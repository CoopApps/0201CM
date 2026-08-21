from __future__ import annotations

import argparse
import html
import json
import shutil
import stat
import struct
from io import BytesIO
from pathlib import Path

import pefile
from PIL import Image


RGB565_HEADER_SIZE = 48
IMAGE_EXTENSIONS = {".rgn", ".mbr"}
REFERENCE_EXTENSIONS = {".wav", ".fnt", ".t2k", ".ttf", ".pct"}
PE_RESOURCE_TYPES = {
    1: "cursor",
    2: "bitmap",
    3: "icon",
    6: "string_table",
    10: "rcdata",
    12: "group_cursor",
    14: "group_icon",
    16: "version",
}


def rgb565_to_rgb(value: int) -> tuple[int, int, int]:
    red = (value >> 11) & 0x1F
    green = (value >> 5) & 0x3F
    blue = value & 0x1F
    return (
        (red << 3) | (red >> 2),
        (green << 2) | (green >> 4),
        (blue << 3) | (blue >> 2),
    )


def decode_rgb565_header_image(path: Path) -> tuple[Image.Image, dict]:
    data = path.read_bytes()
    if len(data) < RGB565_HEADER_SIZE:
        raise ValueError("file is too small to contain an RGB565 header")

    width = int.from_bytes(data[0:4], "little")
    height = int.from_bytes(data[4:8], "little")
    expected = RGB565_HEADER_SIZE + width * height * 2
    if width <= 0 or height <= 0:
        raise ValueError(f"invalid RGB565 dimensions {width}x{height}")
    if expected != len(data):
        raise ValueError(f"expected {expected} bytes for {width}x{height}, got {len(data)}")

    pixels = data[RGB565_HEADER_SIZE:]
    image = Image.new("RGB", (width, height))
    image.putdata(
        [
            rgb565_to_rgb(int.from_bytes(pixels[index : index + 2], "little"))
            for index in range(0, len(pixels), 2)
        ]
    )
    return image, {
        "width": width,
        "height": height,
        "header_size": RGB565_HEADER_SIZE,
        "pixel_format": "RGB565 little-endian",
        "source_size": len(data),
    }


def resource_name(entry) -> str:
    if entry.name is not None:
        return str(entry.name)
    return str(entry.struct.Id)


def exe_stem(path: Path, source_root: Path) -> str:
    return "_".join(path.relative_to(source_root).with_suffix("").parts)


def dib_to_bmp_bytes(dib: bytes) -> bytes:
    if len(dib) < 16:
        raise ValueError("DIB resource is too small")

    header_size = int.from_bytes(dib[0:4], "little")
    bit_count = int.from_bytes(dib[14:16], "little")
    compression = int.from_bytes(dib[16:20], "little") if len(dib) >= 20 else 0
    clr_used = int.from_bytes(dib[32:36], "little") if len(dib) >= 36 else 0
    palette_entries = (clr_used or (1 << bit_count)) if bit_count <= 8 else 0
    masks_size = 12 if compression == 3 and header_size == 40 else 0
    pixel_offset = 14 + header_size + palette_entries * 4 + masks_size
    file_size = 14 + len(dib)
    return b"BM" + struct.pack("<IHHI", file_size, 0, 0, pixel_offset) + dib


def icon_or_cursor_file(data: bytes, *, cursor: bool) -> bytes:
    if len(data) < 16:
        raise ValueError("icon/cursor resource is too small")

    width = abs(int.from_bytes(data[4:8], "little", signed=True))
    stored_height = abs(int.from_bytes(data[8:12], "little", signed=True))
    height = stored_height // 2 if stored_height > 1 else stored_height
    planes = int.from_bytes(data[12:14], "little")
    bit_count = int.from_bytes(data[14:16], "little")

    width_byte = width if 0 < width < 256 else 0
    height_byte = height if 0 < height < 256 else 0
    kind = 2 if cursor else 1
    header = struct.pack("<HHH", 0, kind, 1)
    if cursor:
        entry = struct.pack("<BBBBHHII", width_byte, height_byte, 0, 0, 0, 0, len(data), 22)
    else:
        entry = struct.pack(
            "<BBBBHHII", width_byte, height_byte, 0, 0, planes, bit_count, len(data), 22
        )
    return header + entry + data


def write_image_preview(image_path: Path, preview_path: Path) -> tuple[int, int] | None:
    try:
        with Image.open(image_path) as image:
            image.save(preview_path)
            return image.size
    except Exception:
        return None


def write_image_bytes_preview(image_bytes: bytes, preview_path: Path) -> tuple[int, int] | None:
    try:
        with Image.open(BytesIO(image_bytes)) as image:
            image.save(preview_path)
            return image.size
    except Exception:
        return None


def iter_pe_resource_data(exe_path: Path):
    pe = pefile.PE(str(exe_path))
    if not hasattr(pe, "DIRECTORY_ENTRY_RESOURCE"):
        return
    for type_entry in pe.DIRECTORY_ENTRY_RESOURCE.entries:
        type_id = type_entry.struct.Id
        type_name = PE_RESOURCE_TYPES.get(type_id, f"type_{type_id}")
        if not hasattr(type_entry, "directory"):
            continue
        for name_entry in type_entry.directory.entries:
            if not hasattr(name_entry, "directory"):
                continue
            for lang_entry in name_entry.directory.entries:
                data_rva = lang_entry.data.struct.OffsetToData
                size = lang_entry.data.struct.Size
                yield {
                    "type_id": type_id,
                    "type_name": type_name,
                    "name": resource_name(name_entry),
                    "language": lang_entry.struct.Id,
                    "data": pe.get_data(data_rva, size),
                }


def export_pe_resources(source_root: Path, output_root: Path, manifest: dict) -> None:
    pe_root = output_root / "pe_resources"
    for exe_path in sorted(source_root.rglob("*.exe")):
        relative_exe = exe_path.relative_to(source_root)
        exe_prefix = exe_stem(exe_path, source_root)
        try:
            resources = list(iter_pe_resource_data(exe_path))
        except pefile.PEFormatError as error:
            manifest["assets"].append(
                {
                    "kind": "non_pe_executable",
                    "source": str(exe_path),
                    "relative_source": relative_exe.as_posix(),
                    "source_size": exe_path.stat().st_size,
                    "note": f"Not a PE resource container: {error}",
                }
            )
            continue
        except Exception as error:
            manifest["failures"].append({"source": str(exe_path), "error": str(error)})
            continue

        for resource in resources:
            resource_stem = (
                f"{exe_prefix}_{resource['type_name']}_{resource['name']}_"
                f"lang{resource['language']}"
            )
            resource_dir = pe_root / exe_prefix / resource["type_name"]
            resource_dir.mkdir(parents=True, exist_ok=True)
            raw_path = resource_dir / f"{resource_stem}.bin"
            raw_path.write_bytes(resource["data"])

            asset = {
                "kind": f"pe_{resource['type_name']}",
                "source": str(exe_path),
                "relative_source": relative_exe.as_posix(),
                "resource_type_id": resource["type_id"],
                "resource_type": resource["type_name"],
                "resource_name": resource["name"],
                "resource_language": resource["language"],
                "source_size": len(resource["data"]),
                "raw_output": str(raw_path),
                "relative_raw_output": raw_path.relative_to(output_root).as_posix(),
            }

            try:
                if resource["type_id"] == 2:
                    bmp_path = resource_dir / f"{resource_stem}.bmp"
                    png_path = resource_dir / f"{resource_stem}.png"
                    bmp_path.write_bytes(dib_to_bmp_bytes(resource["data"]))
                    size = write_image_preview(bmp_path, png_path)
                    if size:
                        asset.update(
                            {
                                "kind": "pe_bitmap_image",
                                "output": str(png_path),
                                "relative_output": png_path.relative_to(output_root).as_posix(),
                                "width": size[0],
                                "height": size[1],
                            }
                        )
                elif resource["type_id"] in {1, 3}:
                    cursor = resource["type_id"] == 1
                    ext = ".cur" if cursor else ".ico"
                    image_path = resource_dir / f"{resource_stem}{ext}"
                    preview_path = resource_dir / f"{resource_stem}.png"
                    image_path.write_bytes(icon_or_cursor_file(resource["data"], cursor=cursor))
                    if cursor:
                        size = write_image_bytes_preview(
                            icon_or_cursor_file(resource["data"], cursor=False), preview_path
                        )
                    else:
                        size = write_image_preview(image_path, preview_path)
                    asset.update(
                        {
                            "container_output": str(image_path),
                            "relative_container_output": image_path.relative_to(
                                output_root
                            ).as_posix(),
                        }
                    )
                    if size:
                        asset.update(
                            {
                                "kind": "pe_cursor_image" if cursor else "pe_icon_image",
                                "output": str(preview_path),
                                "relative_output": preview_path.relative_to(
                                    output_root
                                ).as_posix(),
                                "width": size[0],
                                "height": size[1],
                            }
                        )
            except Exception as error:
                asset["decode_error"] = str(error)

            manifest["assets"].append(asset)


def copy_reference_file(source_root: Path, output_root: Path, path: Path) -> dict:
    relative = path.relative_to(source_root)
    target = output_root / "original_files" / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        target.chmod(target.stat().st_mode | stat.S_IWRITE)
    shutil.copy2(path, target)
    ext = path.suffix.lower()
    kind = {
        ".wav": "sound",
        ".fnt": "font_bitmap_or_custom",
        ".t2k": "font_t2k",
        ".ttf": "font_ttf",
        ".pct": "tactic_binary",
    }[ext]
    return {
        "kind": kind,
        "source": str(path),
        "relative_source": relative.as_posix(),
        "output": str(target),
        "relative_output": target.relative_to(output_root).as_posix(),
        "source_size": path.stat().st_size,
        "note": "Copied verbatim; decode semantics are tracked separately from image export.",
    }


def export_assets(source_root: Path, output_root: Path) -> dict:
    output_root.mkdir(parents=True, exist_ok=True)
    manifest = {
        "format": "cm0102-rs-asset-export",
        "version": 2,
        "source_root": str(source_root),
        "output_root": str(output_root),
        "assets": [],
        "failures": [],
    }

    for path in sorted(source_root.rglob("*")):
        if not path.is_file():
            continue

        suffix = path.suffix.lower()
        relative = path.relative_to(source_root)
        if suffix in IMAGE_EXTENSIONS:
            target = output_root / relative.with_suffix(".png")
            target.parent.mkdir(parents=True, exist_ok=True)
            try:
                image, metadata = decode_rgb565_header_image(path)
                image.save(target)
                manifest["assets"].append(
                    {
                        "kind": "rgb565_image",
                        "source": str(path),
                        "relative_source": relative.as_posix(),
                        "output": str(target),
                        "relative_output": target.relative_to(output_root).as_posix(),
                        **metadata,
                    }
                )
            except Exception as error:
                manifest["failures"].append({"source": str(path), "error": str(error)})
        elif suffix in REFERENCE_EXTENSIONS:
            try:
                manifest["assets"].append(copy_reference_file(source_root, output_root, path))
            except Exception as error:
                manifest["failures"].append({"source": str(path), "error": str(error)})

    export_pe_resources(source_root, output_root, manifest)

    counts_by_kind: dict[str, int] = {}
    for asset in manifest["assets"]:
        counts_by_kind[asset["kind"]] = counts_by_kind.get(asset["kind"], 0) + 1
    image_kinds = {"rgb565_image", "pe_bitmap_image", "pe_cursor_image", "pe_icon_image"}
    manifest["summary"] = {
        "assets": len(manifest["assets"]),
        "decoded_images": sum(1 for asset in manifest["assets"] if asset["kind"] in image_kinds),
        "rgb565_images": counts_by_kind.get("rgb565_image", 0),
        "pe_bitmap_images": counts_by_kind.get("pe_bitmap_image", 0),
        "pe_cursor_images": counts_by_kind.get("pe_cursor_image", 0),
        "pe_icon_images": counts_by_kind.get("pe_icon_image", 0),
        "sounds": counts_by_kind.get("sound", 0),
        "fonts": sum(counts_by_kind.get(kind, 0) for kind in ("font_bitmap_or_custom", "font_t2k", "font_ttf")),
        "tactic_binaries": counts_by_kind.get("tactic_binary", 0),
        "failures": len(manifest["failures"]),
        "counts_by_kind": dict(sorted(counts_by_kind.items())),
    }
    (output_root / "asset_manifest.json").write_text(
        json.dumps(manifest, indent=2), encoding="utf-8"
    )
    return manifest


def cards_for(assets: list[dict], output_root: Path) -> str:
    cards = []
    for asset in assets:
        rel = asset["relative_output"]
        name = Path(rel).name
        cards.append(
            f"""
            <article>
              <img src="{html.escape(rel)}" alt="{html.escape(name)}">
              <strong>{html.escape(name)}</strong>
              <span>{asset.get("width", "?")}x{asset.get("height", "?")}</span>
              <code>{html.escape(asset["relative_source"])}</code>
            </article>
            """
        )
    return "".join(cards)


def reference_list(assets: list[dict]) -> str:
    items = []
    for asset in assets:
        items.append(
            f"<li><code>{html.escape(asset['relative_source'])}</code> "
            f"<span>{asset['source_size']:,} bytes</span></li>"
        )
    return "\n".join(items)


def write_contact_sheet(manifest: dict, output_root: Path) -> None:
    rgb565 = [asset for asset in manifest["assets"] if asset["kind"] == "rgb565_image"]
    pe_images = [
        asset
        for asset in manifest["assets"]
        if asset["kind"] in {"pe_bitmap_image", "pe_cursor_image", "pe_icon_image"}
    ]
    fonts = [asset for asset in manifest["assets"] if asset["kind"].startswith("font_")]
    tactics = [asset for asset in manifest["assets"] if asset["kind"] == "tactic_binary"]
    summary = manifest["summary"]
    html_text = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>CM0102 Complete UI Graphics Inventory</title>
  <style>
    :root {{
      --ink: #1e292c;
      --paper: #f7f1e8;
      --panel: #fffaf2;
      --line: #d1c5b6;
      --green: #00685f;
    }}
    body {{ margin: 24px; font-family: Georgia, serif; background: #ebe5da; color: var(--ink); }}
    h1 {{ font-size: 42px; margin-bottom: 4px; }}
    h2 {{ margin-top: 36px; border-bottom: 2px solid var(--green); padding-bottom: 6px; }}
    .stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 12px; margin: 20px 0; }}
    .stat, article {{ background: var(--panel); border: 1px solid var(--line); border-radius: 12px; box-shadow: 0 10px 24px rgba(0,0,0,.08); }}
    .stat {{ padding: 14px; }}
    .stat strong {{ display: block; font-size: 30px; }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 16px; }}
    article {{ padding: 12px; }}
    img {{ width: 100%; height: 160px; object-fit: contain; background: #111; border-radius: 8px; image-rendering: auto; }}
    strong, span, code {{ display: block; margin-top: 8px; }}
    code {{ font-size: 12px; color: #637073; overflow-wrap: anywhere; }}
    li {{ margin: 6px 0; }}
  </style>
</head>
<body>
  <h1>CM0102 Complete UI Graphics Inventory</h1>
  <p>Decoded original RGB565 image files, embedded PE image resources, and catalogued the visual dependencies that need code-derived render semantics.</p>
  <section class="stats">
    <div class="stat"><strong>{summary["decoded_images"]}</strong><span>decoded image previews</span></div>
    <div class="stat"><strong>{summary["rgb565_images"]}</strong><span>RGN/MBR images</span></div>
    <div class="stat"><strong>{summary["pe_bitmap_images"]}</strong><span>PE bitmaps</span></div>
    <div class="stat"><strong>{summary["pe_icon_images"] + summary["pe_cursor_images"]}</strong><span>PE icons/cursors</span></div>
    <div class="stat"><strong>{summary["fonts"]}</strong><span>font files copied</span></div>
    <div class="stat"><strong>{summary["tactic_binaries"]}</strong><span>tactic PCT binaries</span></div>
    <div class="stat"><strong>{summary["failures"]}</strong><span>failures</span></div>
  </section>

  <h2>Original RGB565 Images</h2>
  <section class="grid">{cards_for(rgb565, output_root)}</section>

  <h2>Executable Resource Images</h2>
  <section class="grid">{cards_for(pe_images, output_root)}</section>

  <h2>Fonts Copied Verbatim</h2>
  <ul>{reference_list(fonts)}</ul>

  <h2>Tactic PCT Binaries Catalogued</h2>
  <p>These are not normal image files; they are tactic/formation data used by the UI and need a separate code-derived decoder.</p>
  <ul>{reference_list(tactics)}</ul>
</body>
</html>
"""
    (output_root / "contact_sheet.html").write_text(html_text, encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Export and inventory CM0102 UI assets.")
    parser.add_argument("--source", default="D:/cm0102", help="CM0102 install root")
    parser.add_argument("--output", default="D:/cm0102-rs/assets/cm0102", help="output asset root")
    args = parser.parse_args()

    manifest = export_assets(Path(args.source), Path(args.output))
    write_contact_sheet(manifest, Path(args.output))
    summary = manifest["summary"]
    print(
        f"decoded {summary['decoded_images']} image(s): {summary['rgb565_images']} RGB565, "
        f"{summary['pe_bitmap_images']} PE bitmap(s), {summary['pe_icon_images']} PE icon(s), "
        f"{summary['pe_cursor_images']} PE cursor(s); copied {summary['fonts']} font(s), "
        f"catalogued {summary['tactic_binaries']} tactic binary file(s), failures {summary['failures']}"
    )
    print(Path(args.output) / "asset_manifest.json")
    print(Path(args.output) / "contact_sheet.html")


if __name__ == "__main__":
    main()
