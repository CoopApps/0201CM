#!/usr/bin/env python3
"""Emit code-derived CM0102 UI renderer flag semantics."""
from __future__ import annotations

import json
from pathlib import Path


GUI_FLAGS = [
    ("0x00000400", "image_resource_object", "guio_draw_object loads/caches image named by object text through graphics_load_image_buffer when object cache is empty."),
    ("0x00000800", "marker_or_timed_text", "guio_draw_object scans text until marker byte 1; timed updater rewrites flagged text objects."),
    ("0x00001000", "blink_color_swap", "timed update swaps color shorts every ~100ms for blink objects."),
    ("0x00002000", "left_embedded_indicator", "guio_draw_object draws small left-side bitmap DAT_009b9c4c, color flips from black to highlight if primary is black."),
    ("0x00004000", "right_embedded_plus", "draws right-side bitmap DAT_009b9c80, or fallback '+' text if bitmap mode disabled."),
    ("0x00020000", "right_embedded_comma", "draws right-side bitmap DAT_009b9cc4, or fallback ',' text if bitmap mode disabled."),
    ("0x00040000", "right_embedded_minus", "draws right-side bitmap DAT_009b9d0c, or fallback '-' text if bitmap mode disabled."),
]

PANEL_FLAGS = [
    ("0x00000002", "shade_or_disabled_fill", "graphics_draw_panel_rect calls graphics shade/fade fill helper over the rect."),
    ("0x00000004", "skip_primary_fill_branch", "suppresses normal gradient/fill path before border rendering."),
    ("0x00000008", "horizontal_gradient_fill", "fills by horizontal lines with stepped color changes."),
    ("0x00000010", "outline_only_or_border_mode", "draws rectangle outline via graphics_draw_rect_outline unless combined with 0x10000."),
    ("0x00000020", "raised_sunken_panel", "enters beveled panel branch with highlight/shadow lines."),
    ("0x00000040", "inset_pressed_offset", "guio_draw_object offsets text/icons by 2px when set; panel swaps highlight/shadow sides."),
    ("0x00000080", "suppress_lower_edge", "skips/changes lower edge path in panel renderer."),
    ("0x00000200", "simple_outline_in_panel_branch", "uses outline path rather than beveled corner detail in panel branch."),
    ("0x00000400", "line_or_divider_mode", "draws line/divider variants instead of full panel in non-beveled branch."),
    ("0x00000800", "outer_white_outline", "draws an extra 1px outline around rect using DAT_00ad6bdc."),
    ("0x00001000", "sample_surface_color", "samples current back-surface pixel at rect before drawing and uses it as panel color."),
    ("0x00002000", "dashed_or_alternate_line_mode", "switches line mode argument from 2 to 1 for line/outline helpers."),
    ("0x00004000", "center_horizontal_line", "line/divider mode draws center horizontal line instead of full edge."),
    ("0x00008000", "top_horizontal_line", "with center-horizontal-line flag, keeps y at top instead of vertical center."),
    ("0x00010000", "alternate_gradient_helper", "uses graphics gradient helper 0x005d11e0 instead of normal outline/bevel path."),
    ("0x00020000", "shrink_bottom_right_2px", "pre-adjusts right/bottom by -2 before drawing."),
    ("0x00040000", "shift_top_left_2px", "pre-adjusts left/top by +2 before drawing."),
    ("0x01000000", "center_double_rule", "draws two horizontal rules through vertical center using supplied/default colors."),
]


def main() -> int:
    out = Path("D:/cm0102-rs/reports/carve_segment_index/fixtures/ui_visual_flags.json")
    payload = {
        "schema": "cm0102.ui.visual_flags.v1",
        "source": [
            "D:/cm0102-carve/decompiled/ui_composition_layer/0x005d7fb0.c",
            "D:/cm0102-carve/decompiled/ui_renderer_primitives/0x005cf8e0.c",
        ],
        "gui_object_flags": [{"flag": flag, "name": name, "meaning": meaning} for flag, name, meaning in GUI_FLAGS],
        "panel_flags": [{"flag": flag, "name": name, "meaning": meaning} for flag, name, meaning in PANEL_FLAGS],
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"wrote {out} ({len(GUI_FLAGS)} GUI flags, {len(PANEL_FLAGS)} panel flags)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
