"""Static widget-spec extractor for CM0102 screen draw callbacks.

Given a callback's virtual address (e.g. FUN_008055e0 = Select League(s) draw),
this tool disassembles it with capstone, tracks register values / stack pushes /
scratch-buffer contents symbolically, and emits a JSON list of every widget the
callback declares (area ctors, item ctors, sidebar/nav-bar calls). Values that
can be resolved statically are concrete; values that depend on runtime state
are tagged symbolic (e.g. `{"reg":"edi","last_set":"xor edi,edi"}`), never
invented.

Output shape (per widget):

    {
      "at_va": "0x008056b8",
      "kind": "item",           # item | area | sidebar | nav_bar | strcpy_scratch
      "call_target": "FUN_00549580",
      "args": [
        {"pos":0, "name":"type",       "value":{"literal":1}},
        {"pos":1, "name":"l",          "value":{"literal":100}},
        ...
        {"pos":13,"name":"text_ptr",   "value":{"scratch_ref":{"addr":"0xdbc380",
                                                                "contents":"Championship Manager 2001/02"}}},
        {"pos":15,"name":"event",      "value":{"literal":0}},
        ...
      ],
      "branch_context": [
        {"at_va":"0x805731","cmp":"DAT_009a2051 != 0"}
      ]
    }

Usage:
    D:/Python312/python.exe tools/scrape_screen.py \
        --exe D:/cm0102/cm0102.exe \
        --callback 0x008055e0 \
        --out reports/screens/select_leagues.json

Then any Rust code can read the JSON and evaluate it against real runtime state,
substituting the symbolic values from the .dat-loaded pools or a small init
snippet library.
"""

from __future__ import annotations

import argparse
import json
import re
import struct
from dataclasses import asdict, dataclass, field
from typing import Any, Optional

import capstone


# ---------- PE loading ----------

IMAGE_BASE = 0x00400000


def load_text_section(exe_bytes: bytes) -> tuple[int, int, int]:
    """Return (text_va, text_raw_offset, text_size) from the PE header."""
    pe = struct.unpack_from("<I", exe_bytes, 0x3C)[0]
    nsec = struct.unpack_from("<H", exe_bytes, pe + 6)[0]
    opt_size = struct.unpack_from("<H", exe_bytes, pe + 20)[0]
    so = pe + 24 + opt_size
    for i in range(nsec):
        o = so + i * 40
        name = exe_bytes[o : o + 8].rstrip(b"\x00")
        if name == b".text":
            va = struct.unpack_from("<I", exe_bytes, o + 12)[0] + IMAGE_BASE
            raw = struct.unpack_from("<I", exe_bytes, o + 20)[0]
            size = struct.unpack_from("<I", exe_bytes, o + 16)[0]
            return va, raw, size
    raise RuntimeError("no .text section")


def load_section_map(exe_bytes: bytes) -> list[tuple[str, int, int, int, int]]:
    """Return list of (name, va, raw_offset, raw_size, virtual_size) for every section."""
    pe = struct.unpack_from("<I", exe_bytes, 0x3C)[0]
    nsec = struct.unpack_from("<H", exe_bytes, pe + 6)[0]
    opt_size = struct.unpack_from("<H", exe_bytes, pe + 20)[0]
    so = pe + 24 + opt_size
    out = []
    for i in range(nsec):
        o = so + i * 40
        name = exe_bytes[o : o + 8].rstrip(b"\x00").decode("ascii", "replace")
        vsz = struct.unpack_from("<I", exe_bytes, o + 8)[0]
        va = struct.unpack_from("<I", exe_bytes, o + 12)[0] + IMAGE_BASE
        rsz = struct.unpack_from("<I", exe_bytes, o + 16)[0]
        raw = struct.unpack_from("<I", exe_bytes, o + 20)[0]
        out.append((name, va, raw, rsz, vsz))
    return out


def va_to_file_off(secs, va: int) -> Optional[int]:
    for _n, v, r, rs, _vs in secs:
        if v <= va < v + rs:
            return r + (va - v)
    return None


def read_latin1_cstring(exe_bytes: bytes, secs, va: int, max_len: int = 96) -> Optional[str]:
    off = va_to_file_off(secs, va)
    if off is None:
        return None
    end = exe_bytes.find(b"\x00", off, off + max_len)
    if end < 0:
        end = off + max_len
    try:
        s = exe_bytes[off:end].decode("latin-1")
    except UnicodeDecodeError:
        return None
    if not s.isprintable() or len(s) < 2:
        return None
    return s


# ---------- known helpers ----------

# Positional arg names + calling convention for each helper.
# `callee_cleans=True` = __thiscall/__stdcall: callee pops the stack args, so we
# have to bump esp_delta by N*4 right after the call.
# `callee_cleans=False` = cdecl: caller pops (a subsequent `add esp, N` handles it).
HELPER_SIGS: dict[int, tuple[str, list[str], bool]] = {
    # FUN_00549580 item ctor — __thiscall, 18 stack args after ECX.
    0x00549580: (
        "item",
        [
            "type", "l", "t", "r", "b", "col", "row", "flags",
            "color_a", "color_b", "aux_a", "font", "aux_b", "text_ptr",
            "aux_c", "event", "aux_d", "area_handle",
        ],
        True,
    ),
    # FUN_00549790 area ctor — __thiscall, 11 stack args after ECX.
    0x00549790: (
        "area",
        [
            "l", "t", "r", "b", "ncols", "col_weights_ptr", "nrows",
            "aux_a", "flags", "aux_b", "terminator",
        ],
        True,
    ),
    # FUN_00745540 sidebar — cdecl (caller emits `add esp, 8` after each call).
    0x00745540: ("sidebar", ["mode", "aux"], False),
    # FUN_005D75B0 nav_bar — cdecl (same pattern as sidebar).
    0x005D75B0: ("nav_bar", ["back_flag", "next_flag"], False),
    # FUN_006547C0 strcpy_scratch — cdecl (dst, src).
    0x006547C0: ("strcpy_scratch", ["dst", "src"], False),
    # FUN_00933D2F sprintf_scratch — cdecl varargs.
    0x00933D2F: ("sprintf_scratch", ["dst", "fmt", "arg0", "arg1", "arg2"], False),
}


# ---------- value type ----------

@dataclass
class Value:
    """A resolved-or-symbolic value. Exactly one field is populated."""

    literal: Optional[int] = None
    literal_hex: Optional[str] = None
    literal_signed: Optional[int] = None
    string: Optional[dict] = None       # {"addr": "0xhex", "text": "..."}
    global_ref: Optional[str] = None     # "DAT_00xxxxxx"
    scratch_ref: Optional[dict] = None   # {"addr":"0x...","contents":"..."}
    stack_addr: Optional[dict] = None    # {"frame_offset": int, "resolved_bytes": [..]}
    reg: Optional[dict] = None           # {"reg":"edi","last_set":"..."} — unresolved
    unknown: Optional[str] = None        # reason

    @staticmethod
    def lit(v: int) -> "Value":
        # Signed 32-bit interpretation for values that look negative.
        signed = v if v < 0x80000000 else v - 0x100000000
        return Value(literal=v, literal_hex=hex(v), literal_signed=signed)

    @staticmethod
    def stack_ref(frame_off: int) -> "Value":
        return Value(stack_addr={"frame_offset": frame_off})

    @staticmethod
    def unresolved_reg(reg: str, why: str) -> "Value":
        return Value(reg={"reg": reg, "last_set": why})

    @staticmethod
    def unresolved(why: str) -> "Value":
        return Value(unknown=why)

    def to_json(self) -> dict:
        d = {k: v for k, v in asdict(self).items() if v is not None}
        return d


# ---------- register/pushes tracker ----------

@dataclass
class TrackerState:
    """Cheap symbolic tracker: reg -> last known Value; stack of Push records;
    stack frame slot-tracker (offset-from-function-entry-esp -> (size, Value))."""

    regs: dict[str, Value] = field(default_factory=dict)
    pushes: list[tuple[int, Value]] = field(default_factory=list)  # (va, value)
    scratch: dict[int, str] = field(default_factory=dict)  # buffer_addr -> current string
    ecx_at_last_call: Value = field(default_factory=Value)
    branch_ctx: list[dict] = field(default_factory=list)
    last_cmp: Optional[tuple[str, int]] = None  # (lhs_desc, imm)
    # Stack tracker. `esp_delta` tracks bytes ESP has moved relative to function
    # entry (negative going into deeper allocation). `stack_frame[abs_off]` holds
    # (size_bytes, Value) for a slot at that absolute offset from entry-esp.
    esp_delta: int = 0
    stack_frame: dict[int, tuple[int, Value]] = field(default_factory=dict)


# Byte/word aliases for the general-purpose registers. When code writes to a
# 32-bit reg, the low 16 / low 8 aliases carry the same value; when code writes
# to a byte alias, only the parent's low byte changes (we ignore high bytes of
# ax/bx/cx/dx — rare in this codebase).
_REG_ALIASES = {
    "eax": ["ax", "al"],
    "ebx": ["bx", "bl"],
    "ecx": ["cx", "cl"],
    "edx": ["dx", "dl"],
    "esi": ["si"],
    "edi": ["di"],
}
# Inverse map: which parent reg backs a byte/word alias.
_ALIAS_PARENT = {alias: parent for parent, aliases in _REG_ALIASES.items() for alias in aliases}


def _write_reg(state: TrackerState, reg: str, value: Value) -> None:
    """Assign a value to a register and propagate to its aliases (or vice versa)."""
    state.regs[reg] = value
    for alias in _REG_ALIASES.get(reg, ()):
        state.regs[alias] = value
    # If we wrote to a byte alias, also update the parent's tracker slot ONLY if
    # the value fits in a byte — otherwise leave parent unchanged.
    parent = _ALIAS_PARENT.get(reg)
    if parent is not None:
        if value.literal is not None and 0 <= value.literal < 0x100:
            state.regs[parent] = value


def _parse_operand_val(op: str, state: TrackerState, exe: bytes, secs) -> Value:
    """Best-effort resolve of a single operand (immediate, register, [mem])."""
    op = op.strip()
    # Hex or decimal immediate
    if op.startswith("0x"):
        v = int(op, 16)
        s = read_latin1_cstring(exe, secs, v)
        if s is not None:
            return Value(string={"addr": hex(v), "text": s})
        return Value.lit(v)
    if op.lstrip("-").isdigit():
        return Value.lit(int(op))
    # Register (return the tracker's current value if we have one)
    if op in state.regs:
        return state.regs[op]
    # Stack-slot read (`[esp+K]`, `dword ptr [esp+K]`, etc)
    stk = _parse_stack_slot(op)
    if stk is not None:
        k, _width = stk
        slot = state.stack_frame.get(state.esp_delta + k)
        if slot is not None:
            _sz, sv = slot
            return sv
        return Value.stack_ref(state.esp_delta + k)
    # Memory dereference of a global? Ghidra-style operands look like
    # "word ptr [0x00acdf74]" or "dword ptr [0xac688c]".
    if "ptr [" in op and op.rstrip("]").endswith("]"):
        try:
            addr_str = op.split("[", 1)[1].rstrip("]")
            if addr_str.startswith("0x"):
                return Value(global_ref=f"DAT_{int(addr_str, 16):08x}")
        except (ValueError, IndexError):
            pass
    return Value.unresolved_reg(op, "unread register")


_STACK_OP_RE = re.compile(r"^(?:(byte|word|dword)\s+ptr\s+)?\[\s*esp\s*(\+\s*(0x[0-9a-f]+|\d+))?\s*\]$")


def _parse_stack_slot(op: str) -> Optional[tuple[int, str]]:
    """Return (K, width_name_or_dword) if op looks like `[esp+K]` (0 if bare)."""
    m = _STACK_OP_RE.match(op.strip())
    if not m:
        return None
    width = m.group(1) or "dword"
    k = 0
    if m.group(3):
        k = int(m.group(3), 16) if m.group(3).startswith("0x") else int(m.group(3))
    return k, width


def _slot_bytes(width: str) -> int:
    return {"byte": 1, "word": 2, "dword": 4}[width]


def _step(ins, state: TrackerState, exe: bytes, secs) -> None:
    """Update state given one x86 instruction. Best-effort; unknowns clear regs."""
    mn = ins.mnemonic
    op = ins.op_str

    if mn == "push":
        v = _parse_operand_val(op, state, exe, secs)
        state.pushes.append((ins.address, v))
        state.esp_delta -= 4
        state.stack_frame[state.esp_delta] = (4, v)
        return

    if mn == "pop":
        state.esp_delta += 4
        # We don't need to reconstruct the popped register — the register-side
        # tracker will overwrite it on any subsequent write.
        return

    if mn == "sub" and "," in op:
        lhs, rhs = [s.strip() for s in op.split(",", 1)]
        if lhs == "esp" and (rhs.startswith("0x") or rhs.lstrip("-").isdigit()):
            imm = int(rhs, 16) if rhs.startswith("0x") else int(rhs)
            state.esp_delta -= imm
            return
        if lhs == rhs and lhs.isalpha():
            state.regs[lhs] = Value.lit(0)
            return

    if mn == "add" and "," in op:
        lhs, rhs = [s.strip() for s in op.split(",", 1)]
        if lhs == "esp" and (rhs.startswith("0x") or rhs.lstrip("-").isdigit()):
            imm = int(rhs, 16) if rhs.startswith("0x") else int(rhs)
            state.esp_delta += imm
            return

    if mn == "xor" and "," in op:
        lhs, rhs = [s.strip() for s in op.split(",", 1)]
        if lhs == rhs and lhs.isalpha():
            _write_reg(state, lhs, Value.lit(0))
            return

    if mn == "mov" and "," in op:
        lhs, rhs = [s.strip() for s in op.split(",", 1)]
        # Stack write: `mov [esp+K], val` or `mov byte/word/dword ptr [esp+K], val`
        stk = _parse_stack_slot(lhs)
        if stk is not None:
            k, width = stk
            v = _parse_operand_val(rhs, state, exe, secs)
            state.stack_frame[state.esp_delta + k] = (_slot_bytes(width), v)
            return
        # Register write
        if lhs.isalpha() and len(lhs) in (2, 3):
            v = _parse_operand_val(rhs, state, exe, secs)
            _write_reg(state, lhs, v)
            if lhs == "ecx":
                state.ecx_at_last_call = v
            return

    if mn in ("movsx", "movzx") and "," in op:
        lhs, rhs = [s.strip() for s in op.split(",", 1)]
        if lhs.isalpha():
            _write_reg(state, lhs, _parse_operand_val(rhs, state, exe, secs))
            return

    if mn == "lea" and "," in op:
        lhs, rhs = [s.strip() for s in op.split(",", 1)]
        if lhs.isalpha():
            stk = _parse_stack_slot(rhs)
            if stk is not None:
                k, _ = stk
                _write_reg(state, lhs, Value.stack_ref(state.esp_delta + k))
            else:
                _write_reg(state, lhs, Value.unresolved_reg(lhs, f"lea {op}"))
            return

    if mn == "cmp" and "," in op:
        lhs, rhs = [s.strip() for s in op.split(",", 1)]
        if rhs.startswith("0x") or rhs.lstrip("-").isdigit():
            imm = int(rhs, 16) if rhs.startswith("0x") else int(rhs)
            state.last_cmp = (lhs, imm)
        return

    if mn == "test" and "," in op:
        lhs, rhs = [s.strip() for s in op.split(",", 1)]
        if lhs == rhs:
            state.last_cmp = (f"{lhs} != 0", 0)
        return

    if mn in ("je", "jne", "jz", "jnz", "jl", "jle", "jg", "jge"):
        if state.last_cmp is not None:
            state.branch_ctx.append(
                {
                    "at_va": hex(ins.address),
                    "cmp": f"{state.last_cmp[0]} {mn} {state.last_cmp[1]}",
                }
            )
        return


# ---------- widget extraction ----------

@dataclass
class Widget:
    at_va: str
    kind: str
    call_target: str
    args: list[dict]
    ecx: dict
    branch_context: list[dict]


def scrape(exe_path: str, callback_va: int, max_bytes: int = 0x1200) -> list[Widget]:
    exe = open(exe_path, "rb").read()
    secs = load_section_map(exe)
    tv, tr, trs = load_text_section(exe)
    if not (tv <= callback_va < tv + trs):
        raise ValueError(f"callback_va {hex(callback_va)} not in .text")
    file_off = tr + (callback_va - tv)
    code = exe[file_off : file_off + max_bytes]

    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    state = TrackerState()
    widgets: list[Widget] = []

    # Pre-decode the whole block, keyed by VA, so we can skip forward at `jmp`s.
    # Address-order walk with a mutable cursor lets us handle if/jmp/else without
    # walking both branches (which was double-counting pushes and shifting later
    # [esp+K] reads).
    all_ins = list(md.disasm(code, callback_va))
    va_to_idx = {ins.address: i for i, ins in enumerate(all_ins)}
    idx = 0

    while idx < len(all_ins):
        ins = all_ins[idx]
        idx += 1
        # Skip past unconditional forward jmp — the target is the merge point of a
        # preceding if/else; walking the else-branch body double-counts pushes.
        if ins.mnemonic == "jmp" and ins.op_str.startswith("0x"):
            target = int(ins.op_str, 16)
            if target > ins.address and target in va_to_idx:
                idx = va_to_idx[target]
                continue


        # Bail at first ret at function scope (naive but works for simple linear callbacks).
        if ins.mnemonic == "ret" and ins.address > callback_va + 0x10:
            widgets.append(Widget(
                at_va=hex(ins.address),
                kind="ret",
                call_target="",
                args=[],
                ecx={},
                branch_context=list(state.branch_ctx),
            ))
            break

        if ins.mnemonic == "call" and ins.op_str.startswith("0x"):
            tgt = int(ins.op_str, 16)
            sig = HELPER_SIGS.get(tgt)
            if sig is not None:
                kind, arg_names, callee_cleans = sig
                n = len(arg_names)
                if len(state.pushes) >= n:
                    taken = state.pushes[-n:]
                    # cdecl push order: RIGHT-to-left → last push = FIRST arg.
                    args_ordered = list(reversed(taken))
                    args_out = []
                    for i, name in enumerate(arg_names):
                        push_va, val = args_ordered[i]
                        # If the value is a StackAddr, resolve it — for
                        # `col_weights_ptr` we peek `ncols` bytes at the frame
                        # offset; for other stack-pointer args (color/text refs)
                        # we peek one dword. Attaches `resolved_bytes` or
                        # `resolved_dword` alongside the raw addr.
                        val_json = val.to_json()
                        if "stack_addr" in val_json:
                            frame_off = val_json["stack_addr"]["frame_offset"]
                            # Special-case: if this arg is a weights_ptr and we
                            # know ncols from a preceding arg, read that many bytes.
                            if name == "col_weights_ptr":
                                ncols_val = None
                                for other in args_out:
                                    if other["name"] == "ncols" and "literal" in other["value"]:
                                        ncols_val = other["value"]["literal"]
                                        break
                                if ncols_val:
                                    weights: list[int] = []
                                    for j in range(ncols_val):
                                        slot = state.stack_frame.get(frame_off + j)
                                        if slot is None:
                                            break
                                        _sz, sv = slot
                                        if sv.literal is None:
                                            break
                                        weights.append(sv.literal & 0xFF)
                                    val_json["stack_addr"]["resolved_bytes"] = weights
                            else:
                                slot = state.stack_frame.get(frame_off)
                                if slot is not None:
                                    _sz, sv = slot
                                    val_json["stack_addr"]["resolved_dword"] = sv.to_json()
                        args_out.append({
                            "pos": i,
                            "name": name,
                            "push_at": hex(push_va),
                            "value": val_json,
                        })
                    ecx_val = state.ecx_at_last_call.to_json() if state.ecx_at_last_call else {}
                    widgets.append(Widget(
                        at_va=hex(ins.address),
                        kind=kind,
                        call_target=f"FUN_{tgt:08x}",
                        args=args_out,
                        ecx=ecx_val,
                        branch_context=list(state.branch_ctx),
                    ))
                    # Update scratch buffer contents on strcpy_scratch calls.
                    if kind == "strcpy_scratch":
                        dst_val = args_out[0]["value"]
                        src_val = args_out[1]["value"]
                        dst_addr = dst_val.get("literal")
                        src_str = None
                        if "string" in src_val:
                            src_str = src_val["string"]["text"]
                        elif "literal" in src_val:
                            src_str = read_latin1_cstring(exe, secs, src_val["literal"])
                        if dst_addr is not None and src_str is not None:
                            state.scratch[dst_addr] = src_str
                    # Pop the consumed pushes.
                    state.pushes = state.pushes[:-n]
                    # If the callee cleans its own stack (thiscall/stdcall), esp
                    # is restored on return. Bump esp_delta so subsequent
                    # [esp+K] references resolve against the correct absolute offset.
                    if callee_cleans:
                        state.esp_delta += n * 4
                else:
                    # Not enough pushes seen — record what we have.
                    widgets.append(Widget(
                        at_va=hex(ins.address),
                        kind=kind,
                        call_target=f"FUN_{tgt:08x}",
                        args=[
                            {"pos": i, "name": name, "value": {"unknown": "no push seen"}}
                            for i, name in enumerate(arg_names)
                        ],
                        ecx=state.ecx_at_last_call.to_json(),
                        branch_context=list(state.branch_ctx),
                    ))
            else:
                # Unknown helper. Clear only volatile regs (eax/ecx/edx per cdecl).
                for r in ("eax", "ecx", "edx"):
                    state.regs.pop(r, None)
            # Regardless of known/unknown, drop pushes we assume the callee consumed.
            # Conservative fallback: don't clear pushes for unknown calls (some are
            # __stdcall consumers, some cdecl caller-cleans — we can't tell without more
            # analysis, so let subsequent pushes accumulate on top).
            continue

        _step(ins, state, exe, secs)

    return widgets


# ---------- CLI ----------

def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", default="D:/cm0102/cm0102.exe")
    ap.add_argument("--callback", required=True, help="virtual address of the draw callback (hex ok)")
    ap.add_argument("--max-bytes", type=lambda s: int(s, 0), default=0x1200)
    ap.add_argument("--out", default=None)
    args = ap.parse_args()

    va = int(args.callback, 16) if args.callback.startswith("0x") else int(args.callback)
    widgets = scrape(args.exe, va, args.max_bytes)

    payload = {
        "exe": args.exe,
        "callback_va": hex(va),
        "widget_count": sum(1 for w in widgets if w.kind not in ("ret",)),
        "widgets": [asdict(w) for w in widgets],
    }
    text = json.dumps(payload, indent=2, ensure_ascii=False)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as fh:
            fh.write(text)
        print(f"wrote {args.out}  ({len(widgets)} records)")
    else:
        print(text)


if __name__ == "__main__":
    main()
