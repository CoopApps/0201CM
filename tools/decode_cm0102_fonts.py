from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image


FIRST_GLYPH = 0x20
LAST_GLYPH = 0xFF
GLYPH_COUNT = LAST_GLYPH - FIRST_GLYPH + 1


def read_i32(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "little", signed=True)


def decode_font(path: Path) -> dict:
    data = path.read_bytes()
    offset = 0
    height = read_i32(data, offset)
    offset += 4
    glyphs = []
    for codepoint in range(FIRST_GLYPH, LAST_GLYPH + 1):
        advance = read_i32(data, offset)
        bitmap_width = read_i32(data, offset + 4)
        left_bearing = read_i32(data, offset + 8)
        right_bearing = read_i32(data, offset + 12)
        offset += 16
        bitmap_size = bitmap_width * height if bitmap_width > 0 else 0
        bitmap = data[offset : offset + bitmap_size]
        if len(bitmap) != bitmap_size:
            raise ValueError(
                f"{path.name} glyph 0x{codepoint:02x} expected {bitmap_size} bitmap bytes, "
                f"got {len(bitmap)}"
            )
        offset += bitmap_size
        glyphs.append(
            {
                "codepoint": codepoint,
                "char": chr(codepoint),
                "advance": advance,
                "bitmap_width": bitmap_width,
                "height": height,
                "left_bearing": left_bearing,
                "right_bearing": right_bearing,
                "bitmap_offset": offset - bitmap_size,
                "bitmap_size": bitmap_size,
                "has_bitmap": bitmap_size > 0,
                "bitmap_hex": bitmap.hex() if bitmap_size else "",
            }
        )
    if offset != len(data):
        raise ValueError(f"{path.name} consumed {offset} bytes but file length is {len(data)}")
    return {
        "file": path.name,
        "source": str(path),
        "height": height,
        "glyph_range": {"first": FIRST_GLYPH, "last": LAST_GLYPH, "count": GLYPH_COUNT},
        "glyph_count": len(glyphs),
        "non_empty_glyphs": sum(1 for glyph in glyphs if glyph["has_bitmap"]),
        "source_size": len(data),
        "consumed_size": offset,
        "glyphs": glyphs,
    }


def glyph_image(glyph: dict) -> Image.Image:
    width = glyph["bitmap_width"]
    height = glyph["height"]
    image = Image.new("RGBA", (max(width, 1), height), (0, 0, 0, 0))
    if not glyph["bitmap_hex"]:
        return image
    bitmap = bytes.fromhex(glyph["bitmap_hex"])
    pixels = []
    for value in bitmap:
        pixels.append((255, 255, 255, value))
    image.putdata(pixels)
    return image


def write_glyph_sheet(font: dict, output_path: Path) -> None:
    cell_w = max(max(glyph["bitmap_width"] for glyph in font["glyphs"]), 1) + 6
    cell_h = font["height"] + 6
    cols = 16
    rows = (len(font["glyphs"]) + cols - 1) // cols
    sheet = Image.new("RGBA", (cell_w * cols, cell_h * rows), (15, 15, 40, 255))
    for index, glyph in enumerate(font["glyphs"]):
        x = (index % cols) * cell_w + 3
        y = (index // cols) * cell_h + 3
        sheet.alpha_composite(glyph_image(glyph), (x, y))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(output_path)


def sample_width(font: dict, text: str) -> int:
    by_codepoint = {glyph["codepoint"]: glyph for glyph in font["glyphs"]}
    width = 0
    previous_right = 0
    first = True
    for char in text:
        codepoint = ord(" " if char == "|" else char)
        glyph = by_codepoint.get(codepoint)
        if glyph is None or codepoint < FIRST_GLYPH or codepoint > LAST_GLYPH:
            continue
        if not first:
            spacing = ((font["height"] * 3) >> 2) - glyph["left_bearing"] - previous_right
            width += max(spacing, 0)
        width += glyph["advance"]
        previous_right = glyph["right_bearing"]
        first = False
    return width


def export_fonts(source_dir: Path, output_dir: Path) -> dict:
    output_dir.mkdir(parents=True, exist_ok=True)
    fonts = []
    for path in sorted(source_dir.glob("*.fnt")):
        font = decode_font(path)
        font["sample_widths"] = {
            "CM0102": sample_width(font, "CM0102"),
            "Basic Wage": sample_width(font, "Basic Wage"),
            "Contract Expiry": sample_width(font, "Contract Expiry"),
            "Name": sample_width(font, "Name"),
        }
        json_path = output_dir / f"{path.stem}.json"
        sheet_path = output_dir / f"{path.stem}_glyphs.png"
        json_path.write_text(json.dumps(font, indent=2), encoding="utf-8")
        write_glyph_sheet(font, sheet_path)
        fonts.append(
            {
                "file": font["file"],
                "height": font["height"],
                "glyph_count": font["glyph_count"],
                "non_empty_glyphs": font["non_empty_glyphs"],
                "source_size": font["source_size"],
                "consumed_size": font["consumed_size"],
                "metrics_output": str(json_path),
                "relative_metrics_output": json_path.relative_to(output_dir).as_posix(),
                "glyph_sheet_output": str(sheet_path),
                "relative_glyph_sheet_output": sheet_path.relative_to(output_dir).as_posix(),
                "sample_widths": font["sample_widths"],
            }
        )
    manifest = {
        "format": "cm0102-rs-font-metrics",
        "version": 1,
        "source_dir": str(source_dir),
        "output_dir": str(output_dir),
        "evidence": {
            "loader": "0x005ce890",
            "height": "0x005cf7b0",
            "width": "0x005cf610",
            "pair_spacing": "0x005cf840",
        },
        "summary": {
            "fonts": len(fonts),
            "glyphs": sum(font["glyph_count"] for font in fonts),
            "non_empty_glyphs": sum(font["non_empty_glyphs"] for font in fonts),
            "all_consumed_exactly": all(
                font["source_size"] == font["consumed_size"] for font in fonts
            ),
        },
        "fonts": fonts,
    }
    (output_dir / "font_manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    write_contact_sheet(manifest, output_dir)
    return manifest


def write_contact_sheet(manifest: dict, output_dir: Path) -> None:
    cards = []
    for font in manifest["fonts"]:
        rel = font["relative_glyph_sheet_output"]
        widths = ", ".join(f"{key}: {value}" for key, value in font["sample_widths"].items())
        cards.append(
            f"""
            <article>
              <img src="{rel}" alt="{font['file']} glyph sheet">
              <h2>{font['file']}</h2>
              <p>height {font['height']} · {font['non_empty_glyphs']} non-empty glyphs</p>
              <code>{widths}</code>
            </article>
            """
        )
    html = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>CM0102 Font Metrics</title>
  <style>
    body {{ margin: 24px; background: #c0c0c0; color: #050505; font-family: Arial, sans-serif; }}
    h1 {{ margin-bottom: 4px; }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(360px, 1fr)); gap: 16px; }}
    article {{ background: #efefef; border: 2px solid; border-color: #fff #606060 #606060 #fff; padding: 12px; }}
    img {{ width: 100%; image-rendering: pixelated; background: #0f0f28; border: 1px solid #000; }}
    code {{ display: block; white-space: normal; }}
  </style>
</head>
<body>
  <h1>CM0102 Font Metrics</h1>
  <p>{manifest['summary']['fonts']} font(s), {manifest['summary']['glyphs']} glyph records, consumed exactly: {manifest['summary']['all_consumed_exactly']}.</p>
  <section class="grid">{''.join(cards)}</section>
</body>
</html>
"""
    (output_dir / "font_contact_sheet.html").write_text(html, encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Decode CM0102 .fnt metric/bitmap files.")
    parser.add_argument("--source", default="D:/cm0102/Data")
    parser.add_argument("--output", default="D:/cm0102-rs/assets/cm0102/fonts")
    args = parser.parse_args()
    manifest = export_fonts(Path(args.source), Path(args.output))
    summary = manifest["summary"]
    print(
        f"decoded {summary['fonts']} font(s), {summary['glyphs']} glyph record(s), "
        f"{summary['non_empty_glyphs']} non-empty, exact {summary['all_consumed_exactly']}"
    )
    print(Path(args.output) / "font_manifest.json")
    print(Path(args.output) / "font_contact_sheet.html")


if __name__ == "__main__":
    main()
