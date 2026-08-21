#!/usr/bin/env python3
"""Extract CM0102 UI constructor callsites from Ghidra instruction exports.

This is intentionally mechanical: it does not guess widget semantics. It records
the pushes leading into known UI creation functions so renderer work can start
from executable evidence.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

TARGETS = {
    "0x00549580": "display_create_guio_object",
    "0x00549790": "display_create_area_or_emit_area_packet",
    "0x004a3d20": "comp_screen_create_selector_tab",
    "0x00402b00": "area_init_record",
    "0x0045da30": "display_create_guio_object_passthrough_wrapper",
    "0x00415b10": "display_create_child_guio_zero_bounds_wrapper",
}


def norm_addr(value: str) -> str:
    value = value.lower()
    if value.startswith("0x"):
        return "0x" + value[2:].rjust(8, "0")
    return value


def load_strings(path: Path) -> dict[str, str]:
    if not path.exists():
        return {}
    rows = json.loads(path.read_text(encoding="utf-8", errors="replace"))
    return {norm_addr(row["addr"]): row.get("s", "") for row in rows}


def collect_pushes(instructions: list[dict], call_index: int, max_back: int) -> list[dict]:
    pushes: list[dict] = []
    for idx in range(call_index - 1, max(-1, call_index - max_back), -1):
        ins = instructions[idx]
        mnemonic = ins["mnemonic"].lower()
        if mnemonic.startswith("call"):
            break
        if mnemonic == "ret":
            break
        if mnemonic == "push":
            pushes.append({"addr": ins["addr"], "value": ins["operands"]})
    pushes.reverse()
    return pushes


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("instruction_dir", type=Path)
    parser.add_argument("--strings", type=Path, default=Path("D:/cm0102-carve/ghidra_out/cm0102.exe/strings.json"))
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--max-back", type=int, default=70)
    args = parser.parse_args()

    strings = load_strings(args.strings)
    callsites = []

    for path in sorted(args.instruction_dir.glob("*.instructions.json")):
        function = path.name.replace(".instructions.json", "")
        instructions = json.loads(path.read_text(encoding="utf-8", errors="replace"))
        for idx, ins in enumerate(instructions):
            if not ins["mnemonic"].lower().startswith("call"):
                continue
            targets = [target.lower() for target in ins.get("targets", [])]
            if not targets:
                continue
            target = targets[0]
            if target not in TARGETS:
                continue
            pushes = collect_pushes(instructions, idx, args.max_back)
            for push in pushes:
                push["string"] = strings.get(norm_addr(push["value"]))
            callsites.append(
                {
                    "function": function,
                    "call_addr": ins["addr"],
                    "target": target,
                    "target_name": TARGETS[target],
                    "push_count": len(pushes),
                    "pushes": pushes,
                }
            )

    args.out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "cm0102.ui.callsites.v1",
        "source": str(args.instruction_dir).replace("\\", "/"),
        "note": "Raw executable-derived constructor callsites. Register arguments are unresolved until the calling convention/signatures are fully typed.",
        "callsites": callsites,
    }
    args.out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"wrote {args.out} ({len(callsites)} callsites)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
