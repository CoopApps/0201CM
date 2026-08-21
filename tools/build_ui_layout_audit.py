#!/usr/bin/env python3
"""Audit whether a CmScreen fixture can resolve child GUI bounds via CM0102 area layout."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any
import re


def parse_int(value: Any) -> int | None:
    if isinstance(value, int):
        return value
    if not isinstance(value, str):
        return None
    if re.fullmatch(r"-0x[0-9a-fA-F]+", value):
        return -int(value[3:], 16)
    if re.fullmatch(r"0x[0-9a-fA-F]+", value):
        return int(value[2:], 16)
    try:
        return int(value)
    except ValueError:
        return None


def arg_value(decoded: dict[str, Any], key: str) -> Any:
    value = decoded.get(key)
    if not isinstance(value, dict):
        return None
    if isinstance(value.get("value"), int):
        return value["value"]
    resolved = value.get("resolved")
    parsed = parse_int(resolved)
    if parsed is not None:
        return parsed
    raw = value.get("raw")
    parsed = parse_int(raw)
    if parsed is not None:
        return parsed
    return resolved or value.get("string") or raw


def decoded_bounds(call: dict[str, Any]) -> dict[str, Any]:
    bounds = call.get("decoded_bounds") or {}
    return {key: arg_value(bounds, key) for key in ["left", "top", "right", "bottom"]}


def concrete_bounds(bounds: dict[str, Any]) -> bool:
    return all(isinstance(bounds.get(key), int) for key in ["left", "top", "right", "bottom"])


def call_label(call: dict[str, Any]) -> str | None:
    for value in (call.get("decoded_args") or {}).values():
        if isinstance(value, dict) and value.get("string"):
            return value["string"]
    return None


def collect(fixture: dict[str, Any]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    areas: list[dict[str, Any]] = []
    children: list[dict[str, Any]] = []
    for order, call in enumerate(fixture.get("callsites", [])):
        decoded = call.get("decoded_args") or {}
        if call.get("target_name") == "display_create_area_or_emit_area_packet":
            areas.append(
                {
                    "order": order,
                    "call_addr": call["call_addr"],
                    "bounds": decoded_bounds(call),
                    "column_count": arg_value(decoded, "scroll_count_a"),
                    "row_count": arg_value(decoded, "scroll_count_b"),
                    "column_weights": arg_value(decoded, "scroll_bytes_a"),
                    "row_weights": arg_value(decoded, "scroll_bytes_b"),
                    "area_flags": arg_value(decoded, "area_flags"),
                }
            )
        if call.get("target_name") == "display_create_guio_object":
            parent = arg_value(decoded, "parent_area_index")
            if parent not in (None, -1, "-0x1"):
                children.append(
                    {
                        "order": order,
                        "call_addr": call["call_addr"],
                        "parent": parent,
                        "label": call_label(call),
                        "column_slot": arg_value(decoded, "field_20"),
                        "sort_y": arg_value(decoded, "field_24"),
                        "explicit_bounds": decoded_bounds(call),
                    }
                )
    return areas, children


def audit_child(child: dict[str, Any], area: dict[str, Any] | None) -> dict[str, Any]:
    missing: list[str] = []
    if area is None:
        missing.append("parent area record")
        return {**child, "status": "unresolved", "missing": missing}
    if not concrete_bounds(area["bounds"]):
        missing.append("concrete parent area bounds")
    if not isinstance(area.get("column_count"), int) or area["column_count"] <= 0:
        missing.append("area column count (+0xbab)")
    if not isinstance(area.get("row_count"), int) or area["row_count"] <= 0:
        missing.append("area row count (+0xbac)")
    if not isinstance(child.get("column_slot"), int):
        missing.append("child column slot (gui+0x20)")
    if not isinstance(child.get("sort_y"), int):
        missing.append("child sort/row payload (gui+0x24)")
    if missing:
        return {**child, "status": "unresolved", "parent_area_call": area.get("call_addr") if area else None, "missing": missing}
    return {**child, "status": "resolvable", "parent_area_call": area["call_addr"], "missing": []}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("fixture", type=Path)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()

    fixture = json.loads(args.fixture.read_text(encoding="utf-8"))
    areas, children = collect(fixture)
    audited = []
    for child in children:
        parent_area = None
        if isinstance(child["parent"], int) and 0 <= child["parent"] < len(areas):
            parent_area = areas[child["parent"]]
        elif isinstance(child["parent"], str) and child["parent"].startswith("ret["):
            earlier = [area for area in areas if area["order"] < child["order"]]
            parent_area = earlier[-1] if earlier else None
        audited.append(audit_child(child, parent_area))

    payload = {
        "schema": "cm0102.ui.layout_audit.v1",
        "fixture": str(args.fixture).replace("\\", "/"),
        "screen": fixture.get("screen"),
        "area_count": len(areas),
        "child_count": len(children),
        "resolvable_child_count": sum(1 for row in audited if row["status"] == "resolvable"),
        "unresolved_child_count": sum(1 for row in audited if row["status"] == "unresolved"),
        "areas": areas,
        "children": audited,
    }
    out = args.out or args.fixture.with_name(args.fixture.stem + "_layout_audit.json")
    out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(
        f"wrote {out} ({payload['area_count']} areas, {payload['child_count']} children, "
        f"{payload['resolvable_child_count']} resolvable)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
