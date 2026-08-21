#!/usr/bin/env python3
"""Emit code-derived CM0102 UI color/font constants."""
from __future__ import annotations

import json
from pathlib import Path


def rgb565(r: int, g: int, b: int) -> int:
    return ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3)


def rgb555(r: int, g: int, b: int) -> int:
    return ((r & 0xF8) << 7) | ((g & 0xF8) << 2) | (b >> 3)


COLORS = [
    ("DAT_00acdf9a", "green", (0x00, 0x80, 0x00), "FUN_005ce4f0(0,0x80,0)"),
    ("DAT_00acdf40", "dark_green", (0x00, 0x40, 0x00), "FUN_005ce4f0(0,0x40,0)"),
    ("DAT_00acdf42", "cyan", (0x00, 0xFF, 0xFF), "FUN_005ce4f0(0,0xff,0xff)"),
    ("DAT_00ad6bd8", "light_cyan", (0x80, 0xFF, 0xFF), "FUN_005ce4f0(0x80,0xff,0xff)"),
    ("DAT_00ad6be0", "blue", (0x00, 0x00, 0xFF), "FUN_005ce4f0(0,0,0xff)"),
    ("DAT_00ad6bce", "sky_blue", (0x00, 0x80, 0xFF), "FUN_005ce4f0(0,0x80,0xff)"),
    ("DAT_00ad6bf4", "navy", (0x00, 0x00, 0x80), "FUN_005ce4f0(0,0,0x80)"),
    ("DAT_00ad6bc6", "deep_blue", (0x00, 0x00, 0x60), "FUN_005ce4f0(0,0,0x60)"),
    ("DAT_00acdf38", "purple", (0x80, 0x00, 0x80), "FUN_005ce4f0(0x80,0,0x80)"),
    ("DAT_00acdf82", "dark_purple", (0x40, 0x00, 0x40), "FUN_005ce4f0(0x40,0,0x40)"),
    ("DAT_00acdf90", "brown_red", (0x80, 0x40, 0x40), "FUN_005ce4f0(0x80,0x40,0x40)"),
    ("DAT_00acdf6e", "mid_grey", (0x80, 0x80, 0x80), "FUN_005ce4f0(0x80,0x80,0x80)"),
    ("DAT_00ad6bcc", "light_grey", (0xE0, 0xE0, 0xE0), "FUN_005ce4f0(0xe0,0xe0,0xe0)"),
    ("DAT_00acdf44", "silver", (0xC0, 0xC0, 0xC0), "FUN_005ce4f0(0xc0,0xc0,0xc0)"),
    ("DAT_00acdf6c", "dark_grey", (0x40, 0x40, 0x40), "FUN_005ce4f0(0x40,0x40,0x40)"),
    ("DAT_00acdf30", "teal", (0x00, 0x80, 0x80), "FUN_005ce4f0(0,0x80,0x80)"),
]

FORMULA_COLORS = [
    ("DAT_00acdf92", "white_or_555_masked_white", "format-dependent expression in 0x005ce250", "0xffff", "0x7fff"),
    ("DAT_00acdf74", "yellowish_highlight", "format-dependent expression in 0x005ce250", "0xe71c", "0x739c"),
    ("DAT_00acdf98", "format_blue_variant", "format-dependent expression in 0x005ce250", "0xf800", "0x7c00"),
    ("DAT_00ad6bbc", "format_red_variant", "format-dependent expression in 0x005ce250", "0x8000", "0x4000"),
    ("DAT_00ad6bc4", "format_dark_red_variant", "format-dependent expression in 0x005ce250", "0xfc00", "0x7e00"),
    ("DAT_00acdf80", "format_magenta_variant", "format-dependent expression in 0x005ce250", "0x8200", "0x4100"),
    ("DAT_00ad6bdc", "near_white", "format-dependent expression in 0x005ce250", "0xffe0", "0x7fe0"),
    ("DAT_00ad6bde", "near_white_alt", "format-dependent expression in 0x005ce250", "0xfff0", "0x7ff0"),
    ("DAT_00ad6bda", "black", "literal 0 in 0x005ce250", "0x0000", "0x0000"),
]

FONTS = [
    (0, "t2k_slot_0", "loaded by graphics_load_t2k_font_slot(0)", 0x0f),
    (1, "arial_narrow_10", "SI_DATA/arial_narrow_10.fnt", 0x0f),
    (2, "arial_narrow_11", "SI_DATA/arial_narrow_11.fnt", 0x12),
    (3, "arial_14", "SI_DATA/arial_14.fnt", 0x15),
    (4, "arial_16", "SI_DATA/arial_16.fnt", 0x18),
    (5, "arial_18", "SI_DATA/arial_18.fnt", 0x1b),
    (6, "trade_cond_24_bold", "SI_DATA/trade_cond_24_bold.fnt", 0x27),
    (7, "trade_cond_28_bold", "SI_DATA/trade_cond_28_bold.fnt", 0x2d),
]


def main() -> int:
    out = Path("D:/cm0102-rs/reports/carve_segment_index/fixtures/ui_constants.json")
    colors = []
    for global_name, name, rgb, evidence in COLORS:
        r, g, b = rgb
        colors.append(
            {
                "global": global_name,
                "name": name,
                "rgb": {"r": r, "g": g, "b": b},
                "hex": f"#{r:02x}{g:02x}{b:02x}",
                "rgb565": f"0x{rgb565(r,g,b):04x}",
                "rgb555": f"0x{rgb555(r,g,b):04x}",
                "evidence": evidence,
            }
        )
    for global_name, name, evidence, rgb565_value, rgb555_value in FORMULA_COLORS:
        colors.append({"global": global_name, "name": name, "rgb": None, "hex": None, "rgb565": rgb565_value, "rgb555": rgb555_value, "evidence": evidence})

    payload = {
        "schema": "cm0102.ui.constants.v1",
        "source": "D:/cm0102-carve/decompiled/ui_renderer_bootstrap/0x005ce250.c, 0x005ce750.c, and D:/cm0102-carve/decompiled/ui_font_metrics_lift/0x005cf7b0.c",
        "color_format_note": "Runtime packs colors through graphics_rgb_to_surface_pixel; RGB565 is used when green mask DAT_00acdf5c == 0x7e0, otherwise RGB555-style packing.",
        "colors": colors,
        "font_height_note": "graphics_font_row_height(0x005cf7b0) reads loaded metric dword at DAT_00accb9c + slot*0x1404 when available; these are the exact fallback heights used by the switch path.",
        "fonts": [{"slot": slot, "name": name, "fallback_height_px": height, "evidence": evidence} for slot, name, evidence, height in FONTS],
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"wrote {out} ({len(colors)} colors, {len(FONTS)} fonts)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
