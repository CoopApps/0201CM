#!/usr/bin/env python3
"""Build a CM0102 cup-draw screen fixture from lifted competition UI evidence."""
from __future__ import annotations

import json
from pathlib import Path


ROOT = Path("D:/cm0102-rs/reports/carve_segment_index")
DECODED = ROOT / "fixtures/competition_family_decoded_callsites.json"
STRINGS = Path("D:/cm0102-carve/ghidra_out/cm0102.exe/strings.json")
OUT = ROOT / "fixtures/competition_cup_draw_screen.json"


STRING_ADDRS = {
    "0x00988a80": "The draw will be made on {}<%s - date >.",
    "0x00988aac": " <%d - Number(e.g.10)> teams to draw",
    "0x00988ad4": "Teams given bye",
    "0x00988ae4": "    %s",
    "0x00988aec": "%s    ",
    "0x00988b0c": "{}<%s - club name>{} or",
    "0x00988b24": "Automatic Draw",
    "0x00988b34": "Draw All Teams",
    "0x00988b44": "Draw Next Team",
    "0x00988b54": "The draw has not yet been made.",
}


def value_expr(value: dict | None):
    if not value:
        return None
    if isinstance(value.get("value"), int):
        return value["value"]
    return value.get("resolved") or value.get("string") or value.get("raw")


def bounds_from(call: dict) -> dict:
    bounds = call.get("decoded_bounds") or {}
    return {key: value_expr(bounds.get(key)) for key in ["left", "top", "right", "bottom"]}


def arg(call: dict, name: str):
    return value_expr((call.get("decoded_args") or {}).get(name))


def confidence(call: dict, parent_area_index=None) -> str:
    bounds = bounds_from(call)
    if all(isinstance(bounds[key], int) for key in bounds):
        if parent_area_index not in (None, -1, "-0x1") and all(bounds[key] == 0 for key in bounds):
            return "code_derived_constructor_bounds_pending_area_layout"
        return "code_derived_constructor_bounds"
    return "code_derived_dynamic_bounds"


def object_for_call(call: dict, index: int) -> dict:
    target = call["target_name"]
    parent = arg(call, "parent_area_index")
    if target == "display_create_area_or_emit_area_packet":
        kind = "area"
        object_id = f"cup_draw.area.{index:02d}"
    else:
        kind = "child_guio" if parent not in (None, -1, "-0x1") else "guio"
        object_id = f"cup_draw.{kind}.{index:02d}"
    return {
        "id": object_id,
        "kind": kind,
        "bounds": bounds_from(call),
        "text": None,
        "runtime_text_source": arg(call, "text_ptr"),
        "evidence_callsites": [call["call_addr"]],
        "confidence": confidence(call, parent),
        "target": target,
        "parent_area_index": arg(call, "parent_area_index"),
        "object_flags_or_type": arg(call, "object_flags_or_type"),
        "render_flags": arg(call, "render_flags"),
        "text_box_flags": arg(call, "text_box_flags"),
        "font_slot": arg(call, "font_slot"),
        "text_mode": arg(call, "text_mode"),
        "sort_or_payload_field_24": arg(call, "field_24"),
    }


def main() -> int:
    data = json.loads(DECODED.read_text(encoding="utf-8"))
    calls = [call for call in data["callsites"] if call["function"] == "0x00497520"]
    objects = [object_for_call(call, i) for i, call in enumerate(calls)]
    literal_refs = []
    strings = json.loads(STRINGS.read_text(encoding="utf-8", errors="replace"))
    for row in strings:
        if "0x00497520" in (row.get("in") or []) and row["addr"] in STRING_ADDRS:
            literal_refs.append({"addr": row["addr"], "text": row.get("s", STRING_ADDRS[row["addr"]])})

    fixture = {
        "schema": "cm0102.screen.v1",
        "screen": "competition.cup_draw",
        "native_size": {"width": 800, "height": 600},
        "source_function": "0x00497520",
        "source_report": "D:/cm0102-rs/reports/carve_segment_index/competition_ui_family_map.md",
        "decoded_callsites": str(DECODED).replace("\\", "/"),
        "area_layout_model": "D:/cm0102-rs/reports/carve_segment_index/fixtures/ui_area_layout_model.json",
        "notes": [
            "This is the second competition vertical fixture and is derived from 0x00497520 constructor evidence.",
            "Literal strings prove this builder owns the cup draw view: Automatic Draw, Draw All Teams, Draw Next Team, Teams given bye, and draw-not-made text.",
            "Many child GUI objects have 0,0,0,0 constructor bounds because child layout is resolved by parent area ordering and draw transforms.",
            "Dynamic expressions are preserved rather than guessed.",
        ],
        "strings": {addr: {"addr": addr, "text": text} for addr, text in STRING_ADDRS.items()},
        "literal_refs": literal_refs,
        "objects": objects,
        "callsites": calls,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(fixture, indent=2), encoding="utf-8")
    print(f"wrote {OUT} ({len(objects)} objects, {len(calls)} callsites, {len(literal_refs)} string refs)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
