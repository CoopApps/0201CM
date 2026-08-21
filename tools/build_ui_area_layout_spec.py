#!/usr/bin/env python3
"""Emit renderer-ready CM0102 area layout transform facts."""
from __future__ import annotations

import json
from pathlib import Path


ROOT = Path("D:/cm0102-rs/reports/carve_segment_index")
OUT_JSON = ROOT / "fixtures/ui_area_layout_model.json"
OUT_MD = ROOT / "ui_area_layout_model.md"


MODEL = {
    "schema": "cm0102.ui.area_layout_model.v1",
    "source_functions": {
        "0x00402eb0": "area_draw_panel_and_children",
        "0x00403390": "area_rebuild_layout_tables",
        "0x00403640": "area_configure_scrollbar_and_first_visible_child",
        "0x00403240": "area_insert_child_guio_sorted_by_y",
        "0x005cf7b0": "graphics_font_row_height",
    },
    "area_fields": {
        "0x00": "left",
        "0x04": "top",
        "0x08": "right",
        "0x0c": "bottom",
        "0x10": "minimum_child_sort_y_extent",
        "0x14": "maximum_child_sort_y_extent",
        "0x18": "area_flags",
        "0x1c + column*4": "column_left_offsets",
        "0x94 + row*4": "row_top_offsets",
        "0x10c + column*4": "column_right_offsets",
        "0x184 + row*4": "row_bottom_offsets",
        "0x1fc": "scroll_position",
        "0x208": "gui_record_base_pointer",
        "0x20e": "parent_gui_or_selected_child_relation",
        "0x212 + child_slot*2": "sorted_child_gui_indices",
        "0xb74": "child_count",
        "0xb76": "first_visible_child_slot",
        "0xbab": "column_count",
        "0xbac": "row_count",
        "0xbad + column": "column_weight_bytes",
        "0xbcb + row": "row_weight_bytes",
        "0xbed": "scroll_controls_present",
    },
    "gui_child_fields": {
        "0x20": "column_slot",
        "0x24": "sort_y_or_row_payload",
        "0x28": "explicit_left",
        "0x2c": "explicit_right",
        "0x30": "explicit_top",
        "0x34": "explicit_bottom",
        "0x10": "resolved_left_written_before_draw",
        "0x14": "resolved_top_written_before_draw",
        "0x18": "resolved_right_written_before_draw",
        "0x1c": "resolved_bottom_written_before_draw",
    },
    "child_resolution": [
        "first_child_index = area.sorted_child_gui_indices[area.first_visible_child_slot]",
        "visible_row_delta = child.gui_0x24 - first_visible_child.gui_0x24",
        "if visible_row_delta > area.row_count - 1: stop drawing further children",
        "if child.explicit_left != 0: left = child.explicit_left; right = child.explicit_right",
        "else: left = area.left + area.column_left_offsets[child.column_slot]; right = area.left + area.column_right_offsets[child.column_slot]",
        "if child.explicit_top != 0: top = child.explicit_top; bottom = child.explicit_bottom",
        "else: top = area.top + area.row_top_offsets[visible_row_delta + area.minimum_child_sort_y_extent]; bottom = area.top + area.row_bottom_offsets[visible_row_delta + area.minimum_child_sort_y_extent]",
        "write resolved bounds back to gui+0x10/+0x14/+0x18/+0x1c, then call guio_draw_object",
    ],
    "layout_table_generation": {
        "limits": "column_count and row_count must be 1..0x1e; otherwise CM0102 raises an area.cpp error modal",
        "columns": "area_rebuild_layout_tables sums column_weight_bytes, then maps proportional cumulative weights into column_left_offsets and column_right_offsets across available width.",
        "rows": "area_rebuild_layout_tables sums row_weight_bytes, then maps proportional cumulative weights into row_top_offsets and row_bottom_offsets across available height.",
        "gap_rule": "non-first starts use previous end + 2, producing CM0102's grid/panel separation.",
        "last_slot_rule": "last column/row end is available_extent + inset - 1 rather than proportional division.",
    },
    "scroll_resolution": {
        "function": "0x00403640",
        "scroll_position": "area+0x1fc is clamped to [0, area.maximum_child_sort_y_extent]",
        "first_visible_child_slot": "first sorted child whose gui+0x24 is >= scroll_position",
        "scrollbar_fields": "0xb7a..0xba2 hold fixed scrollbar/track geometry derived from area bounds and insets",
    },
    "font_row_height_fallbacks": {
        "0": 15,
        "1": 15,
        "2": 18,
        "3": 21,
        "4": 24,
        "5": 27,
        "6": 39,
        "7": 45,
    },
}


def main() -> int:
    OUT_JSON.write_text(json.dumps(MODEL, indent=2), encoding="utf-8")
    lines = [
        "# CM0102 Area Layout Model",
        "",
        "This is a renderer-ready lift of how CM0102 turns parent areas plus child GUI objects into visible child rectangles.",
        "",
        "## Source Functions",
        "",
    ]
    for addr, name in MODEL["source_functions"].items():
        lines.append(f"- `{addr}` `{name}`")
    lines.extend(["", "## Child Resolution", ""])
    for item in MODEL["child_resolution"]:
        lines.append(f"- {item}")
    lines.extend(["", "## Key Area Fields", "", "| Offset | Meaning |", "|---|---|"])
    for offset, meaning in MODEL["area_fields"].items():
        lines.append(f"| `{offset}` | {meaning} |")
    lines.extend(["", "## Key Child GUI Fields", "", "| Offset | Meaning |", "|---|---|"])
    for offset, meaning in MODEL["gui_child_fields"].items():
        lines.append(f"| `{offset}` | {meaning} |")
    lines.extend(["", "## Table Generation", ""])
    for key, value in MODEL["layout_table_generation"].items():
        lines.append(f"- `{key}`: {value}")
    lines.extend(["", "## Scroll Resolution", ""])
    for key, value in MODEL["scroll_resolution"].items():
        lines.append(f"- `{key}`: {value}")
    lines.extend(["", "## Font Row Heights", ""])
    lines.append("`graphics_font_row_height` (`0x005cf7b0`) supplies the row/text height used by many screen builders before area row counts are calculated. If loaded font metrics are available, CM0102 reads `DAT_00accb9c + slot*0x1404`; otherwise the fallback switch is:")
    lines.extend(["", "| Font slot | Fallback height |", "|---:|---:|"])
    for slot, height in MODEL["font_row_height_fallbacks"].items():
        lines.append(f"| `{slot}` | `{height}` |")
    OUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {OUT_JSON}")
    print(f"wrote {OUT_MD}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
