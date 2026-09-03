#!/usr/bin/env python3
"""Phase-1 triage for Layer 3 screen builders.

Parses dispatcher.rs for every `TodoBuilder` arm, extracts the target
`FUN_00xxxxxx` addresses, and classifies each target by:
- decomp availability (cm0102.exe/decompiled/00xxxxxx.c size & body)
- GDI asm availability (gdi_carve/.../00NNN_sub_00xxxxxx.asm size)
- functions.json size

Emits reports/layer3_screen_triage.md with A/B/C classification.
"""

from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DISPATCHER = ROOT / "crates/cm-render/src/dispatcher.rs"
DECOMP_DIR = Path("D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled")
GDI_ASM_DIR = Path("D:/cm0102-carve/gdi_carve/functions/00004-PE_section_.text")
FUNCTIONS_JSON = Path("D:/cm0102-carve/ghidra_out/cm0102.exe/functions.json")
OUT = ROOT / "reports/layer3_screen_triage.md"

# Class-A: clean decomp under this size (bytes of source .c file)
CLASS_A_MAX_C_BYTES = 40_000
# Class-C: no useful signal at either source
FUN_RE = re.compile(r"FUN_([0-9a-fA-F]{8})")


def load_functions_json() -> dict[str, int]:
    """addr(hex-lower) -> size."""
    data = json.loads(FUNCTIONS_JSON.read_text())
    out: dict[str, int] = {}
    for row in data:
        entry = row["entry"].lower().replace("0x", "")
        out[entry] = int(row["size"])
    return out


def gdi_index() -> dict[str, Path]:
    """addr(hex-lower) -> path to asm file."""
    out: dict[str, Path] = {}
    if not GDI_ASM_DIR.is_dir():
        return out
    for f in GDI_ASM_DIR.iterdir():
        m = re.search(r"sub_([0-9a-fA-F]{8})\.asm$", f.name)
        if m:
            out[m.group(1).lower()] = f
    return out


def parse_dispatcher() -> list[dict]:
    """Return one dict per TodoBuilder arm."""
    text = DISPATCHER.read_text()
    rows: list[dict] = []
    # crude but sufficient: iterate line-by-line and pick up cmd + fn_addr + note per arm
    lines = text.splitlines()
    i = 0
    cur_scope = "global"  # switches at dispatch_club fn
    while i < len(lines):
        ln = lines[i]
        if "fn dispatch_club" in ln:
            cur_scope = "club"
        # match a cmd match arm followed by TodoBuilder
        m_arm = re.match(r"\s*(0x[0-9a-fA-F]+|\d+)\s*=>\s*DispatchResult::TodoBuilder\s*\{", ln)
        if m_arm:
            cmd = m_arm.group(1)
            # peek up to 6 lines for fn_addr and note
            fn_addr = None
            note = None
            for j in range(i, min(i + 8, len(lines))):
                m_fa = re.search(r'fn_addr:\s*"([^"]+)"', lines[j])
                m_no = re.search(r'note:\s*"([^"]+)"', lines[j])
                if m_fa:
                    fn_addr = m_fa.group(1)
                if m_no:
                    note = m_no.group(1)
            rows.append({
                "cmd": cmd,
                "scope": cur_scope,
                "fn_addr": fn_addr or "?",
                "note": note or "",
            })
        # nested TodoBuilders inside if-branches — pick up their fn_addr too by scanning any
        # line that reads `DispatchResult::TodoBuilder {` where the preceding line does NOT
        # already carry the match arm
        i += 1
    return rows


def classify(rows: list[dict], fn_sizes: dict[str, int], gdi: dict[str, Path]) -> list[dict]:
    for r in rows:
        # split combined addrs like "FUN_005276f0+FUN_0057b9c0"
        addrs = FUN_RE.findall(r["fn_addr"])
        if not addrs:
            r["class"] = "C"
            r["reason"] = "no FUN_ in fn_addr string"
            r["decomp_bytes"] = 0
            r["asm_bytes"] = 0
            r["fn_size"] = 0
            r["primary_addr"] = ""
            continue
        primary = addrs[0].lower()
        r["primary_addr"] = primary
        decomp_path = DECOMP_DIR / f"{primary}.c"
        decomp_bytes = decomp_path.stat().st_size if decomp_path.exists() else 0
        asm_path = gdi.get(primary)
        asm_bytes = asm_path.stat().st_size if asm_path else 0
        fn_size = fn_sizes.get(primary, 0)

        r["decomp_bytes"] = decomp_bytes
        r["asm_bytes"] = asm_bytes
        r["fn_size"] = fn_size

        if decomp_bytes == 0 and asm_bytes == 0:
            r["class"] = "C"; r["reason"] = "no decomp AND no GDI asm"
        elif decomp_bytes == 0:
            r["class"] = "B"; r["reason"] = "GDI asm only"
        elif decomp_bytes > CLASS_A_MAX_C_BYTES:
            r["class"] = "C"; r["reason"] = f"decomp too large ({decomp_bytes} bytes)"
        else:
            r["class"] = "A"; r["reason"] = f"decomp {decomp_bytes} bytes"
    return rows


def render(rows: list[dict]) -> str:
    a = [r for r in rows if r["class"] == "A"]
    b = [r for r in rows if r["class"] == "B"]
    c = [r for r in rows if r["class"] == "C"]
    out: list[str] = []
    out.append("# Layer 3 screen-builder triage\n")
    out.append(f"Total TodoBuilder arms parsed: **{len(rows)}**\n")
    out.append(f"- **Class A (decomp-ready, ≤{CLASS_A_MAX_C_BYTES} bytes .c)**: {len(a)}")
    out.append(f"- **Class B (GDI asm only, no decomp)**: {len(b)}")
    out.append(f"- **Class C (skipped)**: {len(c)}\n")

    def tbl(label: str, xs: list[dict]) -> None:
        out.append(f"\n## {label} ({len(xs)})\n")
        out.append("| cmd | scope | primary FUN_ | decomp .c | GDI asm | fn size | note |")
        out.append("|-----|-------|--------------|-----------|---------|---------|------|")
        for r in sorted(xs, key=lambda x: (x["scope"], x["cmd"])):
            out.append(
                f"| `{r['cmd']}` | {r['scope']} | `{r['primary_addr']}` | "
                f"{r['decomp_bytes']} | {r['asm_bytes']} | {r['fn_size']} | "
                f"{r['note'][:70]} |"
            )

    tbl("Class A — port these first", a)
    tbl("Class B — asm walk required", b)
    tbl("Class C — skipped (unbounded / no source)", c)
    return "\n".join(out) + "\n"


def main() -> int:
    fn_sizes = load_functions_json()
    gdi = gdi_index()
    rows = parse_dispatcher()
    rows = classify(rows, fn_sizes, gdi)
    OUT.write_text(render(rows), encoding="utf-8")
    print(f"wrote {OUT}: {len(rows)} arms")
    a = sum(1 for r in rows if r["class"] == "A")
    b = sum(1 for r in rows if r["class"] == "B")
    c = sum(1 for r in rows if r["class"] == "C")
    print(f"  A={a}  B={b}  C={c}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
