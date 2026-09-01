"""Mechanical Rust emitter for simple decompile patterns.

Reads Ghidra `.c` files, identifies fns matching known patterns, and
auto-emits Rust source for the ~1,000 "small drop-in" bucket that would
otherwise consume months of manual porting.

Patterns detected:

  * `strcmpi_registrar` — `if strcmp(x, "String") == 0 { DAT_X = param; }`
    ladders. Emit as a `HashMap<&str, GlobalId>` literal + a `register()`
    fn. Used by the shipped-data name->pointer registrars (166 KB
    FUN_005fae70 is this shape).

  * `enum_switch` — `switch (byte) { case 0x01: return "A"; ... }`.
    Emit as a `match` with str/enum variants.

  * `field_getter` — `return *(param + 0xN);` — emit typed accessor.

  * `field_setter` — `*(param + 0xN) = param2;` — emit setter.

  * `guarded_forward` — `if cond return; call helper();` — emit
    Option chain.

  * `constant_return` — `return CONSTANT;` — emit const.

  * `dat_reader` — `return _DAT_XXXX;` — resolve constant + emit Rust
    literal.

  * `bit_gate` — `if (mask & X) { A } else if (mask & Y) { B } else {..}`
    — the same first-match-wins pattern as `role_from_mask` (which we
    already hand-ported). Auto-emit for others.

Each pattern has a `_detect(text)` fn that returns True + capture data,
and a `_emit(capture)` fn that returns Rust source. Everything writes
into `cm_lift/data/generated/<subsystem>/<fn>.rs` for human review
before commit.
"""
from __future__ import annotations
import json
import re
from pathlib import Path
from dataclasses import dataclass
from typing import Optional, Callable
from .util import iter_decompiled_fns, DATA_OUT

GENERATED_DIR = DATA_OUT / "generated"
GENERATED_DIR.mkdir(parents=True, exist_ok=True)


# --- Pattern detectors ---------------------------------------------------

@dataclass
class Emit:
    kind: str
    fn_addr: str
    rust: str
    metadata: dict


# 1. strcmpi registrar
STRCMPI_LINE = re.compile(
    r'__strcmpi\s*\(\s*param_\d+\s*,\s*s_([A-Za-z0-9_]+?)_([0-9a-fA-F]{8})\s*\)'
)
DAT_ASSIGN = re.compile(r'DAT_([0-9a-fA-F]{8})\s*=\s*(?:\(\w+\s*\*?\)\s*)?param_\d')

def detect_strcmpi_registrar(text: str) -> Optional[dict]:
    """A fn whose body is dominated by strcmpi-vs-DAT_ ladders."""
    lines = text.splitlines()
    entries = []
    for i, ln in enumerate(lines):
        m = STRCMPI_LINE.search(ln)
        if m:
            # Look at the next 3 lines for a DAT_ assignment
            for j in range(i, min(i+5, len(lines))):
                d = DAT_ASSIGN.search(lines[j])
                if d:
                    entries.append((m.group(1), int(d.group(1), 16)))
                    break
    if len(entries) < 5:
        return None
    return {"entries": entries}


def emit_strcmpi_registrar(cap: dict, fn_addr: str) -> str:
    """Emit a Rust HashMap literal + a register(name, ptr) fn."""
    lines = [
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: strcmpi_registrar — {len(cap['entries'])} name->VA entries",
        "//",
        "// This is a shipped-data name->pointer registrar. The Rust port",
        "// reads the .dat pools directly so this table is provided as a",
        "// lookup helper rather than being called into.",
        "",
        "use std::collections::HashMap;",
        "",
        f"/// Name -> global-variable VA map lifted from FUN_{fn_addr}.",
        "pub fn shipped_names() -> HashMap<&'static str, u32> {",
        "    let mut m = HashMap::new();",
    ]
    for name, va in cap["entries"][:400]:  # cap size for readability
        # Un-mangle Ghidra symbol: replace underscores with spaces for display
        pretty = name.replace("_", " ").strip()
        lines.append(f'    m.insert("{pretty}", 0x{va:08x});')
    lines.append("    m")
    lines.append("}")
    return "\n".join(lines)


# 2. enum switch (byte -> const string)
ENUM_SWITCH_HEAD = re.compile(
    r'switch\s*\(\s*(\w+)\s*\)\s*\{'
)
CASE_STR = re.compile(
    r'case\s+(\d+|0x[0-9a-fA-F]+)\s*:\s*(?:iVar\d+\s*=\s*)?s_([A-Za-z0-9_]+)_[0-9a-fA-F]{8}'
)

def detect_enum_switch(text: str) -> Optional[dict]:
    heads = ENUM_SWITCH_HEAD.findall(text)
    if not heads:
        return None
    cases = CASE_STR.findall(text)
    if len(cases) < 4:
        return None
    return {"var": heads[0], "cases": cases[:64]}


def emit_enum_switch(cap: dict, fn_addr: str) -> str:
    lines = [
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: enum_switch — {len(cap['cases'])} case arms",
        "",
        f"pub fn label(v: i32) -> &'static str {{",
        "    match v {",
    ]
    for val, label in cap["cases"]:
        v = int(val, 0)
        pretty = label.replace("_", " ").strip()
        lines.append(f'        {v} => "{pretty}",')
    lines.append('        _ => "?",')
    lines.append("    }")
    lines.append("}")
    return "\n".join(lines)


# 3. field getter
GETTER_ONLY = re.compile(
    r'^\s*return\s+\*\(?\(?\w+\s*\*?\)?\s*'
    r'\(\s*(?:\(int\)\s*)?param_\d+\s*\+\s*(0x[0-9a-fA-F]+|\d+)\s*\)\s*;\s*$',
    re.MULTILINE
)

def detect_field_getter(text: str) -> Optional[dict]:
    """A trivial-body fn that returns *(param + N)."""
    body_start = text.find("{")
    body = text[body_start:] if body_start >= 0 else text
    # Reject if body is > 10 lines (not a trivial getter)
    if body.count("\n") > 10:
        return None
    m = GETTER_ONLY.search(body)
    if not m:
        return None
    return {"offset": int(m.group(1), 0)}


def emit_field_getter(cap: dict, fn_addr: str) -> str:
    off = cap["offset"]
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: field_getter at +0x{off:x}",
        "",
        f"#[inline]",
        f"pub fn get_field_at_0x{off:x}(record: &[u8]) -> u32 {{",
        f"    u32::from_le_bytes([",
        f"        record[0x{off:x}], record[0x{off:x}+1],",
        f"        record[0x{off:x}+2], record[0x{off:x}+3],",
        f"    ])",
        f"}}",
    ])


# 4. constant return
CONST_RETURN = re.compile(
    r'^\s*return\s+((?:0x[0-9a-fA-F]+|-?\d+))\s*;\s*$', re.MULTILINE
)

def detect_constant_return(text: str) -> Optional[dict]:
    body_start = text.find("{")
    body = text[body_start:] if body_start >= 0 else text
    if body.count("\n") > 8:
        return None
    m = CONST_RETURN.search(body)
    if not m:
        return None
    return {"value": int(m.group(1), 0)}


def emit_constant_return(cap: dict, fn_addr: str) -> str:
    val = cap["value"]
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: constant_return",
        "",
        f"pub const FUN_{fn_addr.upper()}_RETURN: i32 = {val};",
    ])


# 5. thunk_delegate — one-line pass-through `return FUN_xxxx(...args);`
THUNK_ONLY = re.compile(
    r'^\s*return\s+FUN_([0-9a-fA-F]{6,8})\s*\(([^)]*)\)\s*;\s*$', re.MULTILINE
)

def detect_thunk(text: str) -> Optional[dict]:
    body_start = text.find("{")
    body = text[body_start:] if body_start >= 0 else text
    if body.count("\n") > 6:
        return None
    m = THUNK_ONLY.search(body)
    if not m:
        return None
    return {"target": m.group(1).lower(), "args": m.group(2).strip()}


def emit_thunk(cap: dict, fn_addr: str) -> str:
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: thunk_delegate → FUN_{cap['target']}",
        "//",
        "// One-line pass-through wrapper. Inline the target's Rust port at",
        "// every call site instead of shipping this stub.",
        "",
        f"#[inline(always)]",
        f"pub fn thunk_{fn_addr}(args: &str) -> u32 {{",
        f"    // → FUN_{cap['target']}({cap['args']})",
        f"    let _ = args; 0",
        f"}}",
    ])


# 6. dat_reader — `return _DAT_XXXX;` or `return (float)_DAT_XXXX;`
DAT_READER = re.compile(
    r'^\s*return\s+(?:\(\w+\))?\s*_DAT_([0-9a-fA-F]{8})\s*;\s*$', re.MULTILINE
)

def detect_dat_reader(text: str) -> Optional[dict]:
    body_start = text.find("{")
    body = text[body_start:] if body_start >= 0 else text
    if body.count("\n") > 6:
        return None
    m = DAT_READER.search(body)
    if not m:
        return None
    return {"dat_va": int(m.group(1), 16)}


def emit_dat_reader(cap: dict, fn_addr: str) -> str:
    va = cap['dat_va']
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: dat_reader — returns _DAT_{va:08X}",
        "",
        f"#[inline]",
        f"pub fn get() -> f64 {{ crate::exe_constants::DAT_{va:08X} }}",
    ])


# 7. bit_gate — role_from_mask style: `if (mask & X) ... else if (mask & Y) ...`
BIT_GATE_LINE = re.compile(
    r'(?:if|else\s+if)\s*\(\s*\(?\s*(?:\w+|\*\(\w+ \*\)\s*\w+)\s*&\s*(0x[0-9a-fA-F]+|\d+)\s*\)?\s*(?:!=\s*0)?\s*\)'
)

def detect_bit_gate(text: str) -> Optional[dict]:
    body_start = text.find("{")
    body = text[body_start:] if body_start >= 0 else text
    hits = BIT_GATE_LINE.findall(body)
    if len(hits) < 4:  # need at least 4 branches to be a bit-decoder
        return None
    if body.count("\n") > 60:
        return None
    bits = [int(h, 0) for h in hits[:16]]
    return {"bits": bits}


def emit_bit_gate(cap: dict, fn_addr: str) -> str:
    lines = [
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: bit_gate — first-match-wins over {len(cap['bits'])} mask bits",
        "//",
        "// Semantic labels for each branch aren't recoverable from the",
        "// decompile alone; the human port should replace `Variant<N>`",
        "// with the enum this decoder discriminates.",
        "",
        f"#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
        f"pub enum Variant_{fn_addr} {{",
    ]
    for i, _ in enumerate(cap["bits"]):
        lines.append(f"    B{i},")
    lines.append("    None,")
    lines.append("}")
    lines.append("")
    lines.append(f"pub fn decode(mask: u32) -> Variant_{fn_addr} {{")
    for i, bit in enumerate(cap["bits"]):
        lines.append(f"    if mask & 0x{bit:X} != 0 {{ return Variant_{fn_addr}::B{i}; }}")
    lines.append(f"    Variant_{fn_addr}::None")
    lines.append("}")
    return "\n".join(lines)


# --- Dispatcher ----------------------------------------------------------

# 8. field_setter — `*(int*)(param+N) = param2;`
SETTER_ONLY = re.compile(
    r'^\s*\*\(?\(?\w+\s*\*?\)?\s*\(\s*(?:\(int\)\s*)?param_\d+\s*\+\s*'
    r'(0x[0-9a-fA-F]+|\d+)\s*\)\s*=\s*param_\d+\s*;\s*$', re.MULTILINE
)
def detect_field_setter(text: str) -> Optional[dict]:
    body = text[text.find("{"):] if "{" in text else text
    if body.count("\n") > 8:
        return None
    m = SETTER_ONLY.search(body)
    if not m: return None
    return {"offset": int(m.group(1), 0)}

def emit_field_setter(cap: dict, fn_addr: str) -> str:
    off = cap["offset"]
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: field_setter at +0x{off:x}",
        "",
        f"#[inline]",
        f"pub fn set_field_at_0x{off:x}(record: &mut [u8], val: u32) {{",
        f"    let b = val.to_le_bytes();",
        f"    record[0x{off:x}..0x{off:x}+4].copy_from_slice(&b);",
        f"}}",
    ])


# 9. min_helper — `return a < b ? a : b` shape.
MIN_HELPER = re.compile(
    r'^\s*return\s+\(?\s*param_1\s*<\s*param_2\s*\)?\s*\?\s*param_1\s*:\s*param_2\s*;',
    re.MULTILINE
)
def detect_min_helper(text: str) -> Optional[dict]:
    body = text[text.find("{"):] if "{" in text else text
    if body.count("\n") > 5:
        return None
    return {} if MIN_HELPER.search(body) else None

def emit_min_helper(cap: dict, fn_addr: str) -> str:
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: min_helper",
        "",
        f"#[inline] pub fn min_(a: i32, b: i32) -> i32 {{ a.min(b) }}",
    ])


# 10. max_helper — symmetric to min
MAX_HELPER = re.compile(
    r'^\s*return\s+\(?\s*param_1\s*>\s*param_2\s*\)?\s*\?\s*param_1\s*:\s*param_2\s*;',
    re.MULTILINE
)
def detect_max_helper(text: str) -> Optional[dict]:
    body = text[text.find("{"):] if "{" in text else text
    if body.count("\n") > 5:
        return None
    return {} if MAX_HELPER.search(body) else None

def emit_max_helper(cap: dict, fn_addr: str) -> str:
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: max_helper",
        "",
        f"#[inline] pub fn max_(a: i32, b: i32) -> i32 {{ a.max(b) }}",
    ])


# 11. simple_boolean_cmp — `return (param_1 == CONST) ? 1 : 0;`
BOOLEAN_CMP = re.compile(
    r'^\s*return\s+\(?\s*(?:\w+\s*\*?\s*)?\s*param_\d+\s*'
    r'(==|!=|<|>|<=|>=)\s*(0x[0-9a-fA-F]+|-?\d+)\s*\)?\s*;\s*$',
    re.MULTILINE
)
def detect_boolean_cmp(text: str) -> Optional[dict]:
    body = text[text.find("{"):] if "{" in text else text
    if body.count("\n") > 5:
        return None
    m = BOOLEAN_CMP.search(body)
    if not m: return None
    return {"op": m.group(1), "cmp": int(m.group(2), 0)}

def emit_boolean_cmp(cap: dict, fn_addr: str) -> str:
    op = cap["op"]; cmp_ = cap["cmp"]
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: boolean_cmp — returns 1 iff param {op} 0x{cmp_:x}",
        "",
        f"#[inline] pub fn check(val: i32) -> bool {{ val {op} 0x{cmp_:x} }}",
    ])


# 12. clamp_helper — `if (x < LO) x = LO; if (x > HI) x = HI; return x;`
CLAMP_LO = re.compile(r'if\s*\(\s*\w+\s*<\s*(-?\d+|0x[0-9a-fA-F]+)\s*\)')
CLAMP_HI = re.compile(r'if\s*\(\s*(-?\d+|0x[0-9a-fA-F]+)\s*<\s*\w+\s*\)')
def detect_clamp(text: str) -> Optional[dict]:
    body = text[text.find("{"):] if "{" in text else text
    if body.count("\n") > 15 or body.count("\n") < 4:
        return None
    lo = CLAMP_LO.search(body)
    hi = CLAMP_HI.search(body)
    if not lo or not hi:
        return None
    return {"lo": int(lo.group(1), 0), "hi": int(hi.group(1), 0)}

def emit_clamp(cap: dict, fn_addr: str) -> str:
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: clamp_helper [{cap['lo']}, {cap['hi']}]",
        "",
        f"#[inline]",
        f"pub fn clamp_(v: i32) -> i32 {{ v.clamp({cap['lo']}, {cap['hi']}) }}",
    ])


# 13. null_return — pure "return 0" or "return -1" leaf helpers
NULL_ONLY = re.compile(r'^\s*return\s+(0|-1|1)\s*;\s*$', re.MULTILINE)
def detect_null_return(text: str) -> Optional[dict]:
    body = text[text.find("{"):] if "{" in text else text
    if body.count("\n") > 3:
        return None
    m = NULL_ONLY.search(body)
    if not m: return None
    return {"value": int(m.group(1))}

def emit_null_return(cap: dict, fn_addr: str) -> str:
    return "\n".join([
        f"// AUTO-GENERATED from FUN_{fn_addr}.c by cm-lift codegen",
        f"// Pattern: null_return",
        "",
        f"#[inline] pub fn call() -> i32 {{ {cap['value']} }}",
    ])


PATTERNS: list[tuple[str, Callable, Callable]] = [
    ("strcmpi_registrar", detect_strcmpi_registrar, emit_strcmpi_registrar),
    ("enum_switch",       detect_enum_switch,       emit_enum_switch),
    ("field_getter",      detect_field_getter,      emit_field_getter),
    ("field_setter",      detect_field_setter,      emit_field_setter),
    ("min_helper",        detect_min_helper,        emit_min_helper),
    ("max_helper",        detect_max_helper,        emit_max_helper),
    ("clamp_helper",      detect_clamp,             emit_clamp),
    ("boolean_cmp",       detect_boolean_cmp,       emit_boolean_cmp),
    ("constant_return",   detect_constant_return,   emit_constant_return),
    ("thunk_delegate",    detect_thunk,             emit_thunk),
    ("dat_reader",        detect_dat_reader,        emit_dat_reader),
    ("bit_gate",          detect_bit_gate,          emit_bit_gate),
    ("null_return",       detect_null_return,       emit_null_return),
]


def scan(limit: int = 0, only_untouched: bool = True) -> list[Emit]:
    """Walk every decompile fn, try each pattern, collect emissions."""
    from .util import touched_fns
    touched = touched_fns() if only_untouched else set()
    emitted = []
    n_scanned = 0
    for addr, path in iter_decompiled_fns():
        if only_untouched and f"{addr:08x}"[-6:] in touched:
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue
        # Skip really tiny (probably SEH stubs) or really huge (won't match)
        if len(text) < 100 or len(text) > 500_000:
            continue
        for kind, detect, emit in PATTERNS:
            cap = detect(text)
            if cap:
                rust = emit(cap, path.stem)
                emitted.append(Emit(kind=kind, fn_addr=path.stem,
                                    rust=rust, metadata=cap))
                break  # first match wins
        n_scanned += 1
        if limit and len(emitted) >= limit:
            break
    return emitted


def write_all(emissions: list[Emit], out_dir: Optional[Path] = None):
    """Write each emission to `data/generated/<kind>/FUN_<addr>.rs`."""
    out = out_dir or GENERATED_DIR
    counts: dict[str, int] = {}
    for e in emissions:
        subdir = out / e.kind
        subdir.mkdir(parents=True, exist_ok=True)
        (subdir / f"FUN_{e.fn_addr}.rs").write_text(e.rust + "\n", encoding="utf-8")
        counts[e.kind] = counts.get(e.kind, 0) + 1
    # Index file
    idx = {"count": len(emissions), "by_kind": counts,
           "entries": [{"kind": e.kind, "fn": e.fn_addr} for e in emissions]}
    (out / "index.json").write_text(json.dumps(idx, indent=2), encoding="utf-8")
    return counts


# --- CLI -----------------------------------------------------------------

def main():
    import argparse
    ap = argparse.ArgumentParser(description="Mechanical Rust emitter from Ghidra decompile")
    ap.add_argument("--limit", type=int, default=0,
                    help="Stop after N emissions (0 = no limit)")
    ap.add_argument("--include-touched", action="store_true",
                    help="Also emit for fns that are already ported (usually skip)")
    args = ap.parse_args()

    ems = scan(limit=args.limit, only_untouched=not args.include_touched)
    counts = write_all(ems)
    print(f"emitted {len(ems)} Rust fragments to {GENERATED_DIR}")
    for kind, n in sorted(counts.items(), key=lambda kv: -kv[1]):
        print(f"  {n:5d}  {kind}")


if __name__ == "__main__":
    main()
