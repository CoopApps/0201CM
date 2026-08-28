"""Decode the raw dumps saved by tools/frida_live_capture.py (v2).

For each burst in reports/screen_captures/live/:
  burst_NNN.frame.bin  -> burst_NNN.frame.png   (RGB565 -> true screenshot)
  burst_NNN.pool.bin   -> burst_NNN.final.json  (post-layout widget rects)

Record layouts from reports/gui_layout_engine_decode.md + widget_pool.rs:
  widget (stride 0x18C): +0x00..0x0C rect LTRB (i32), +0x14 max_columns,
      +0x18 flags (u32), +0x80 text copy (<= 0x30 bytes, NUL-terminated)
  area   (stride 0xBF1): +0x00..0x0C rect LTRB (i32), +0x18 flags,
      +0xBA9 cols (u8), +0xBAB..+0xBC9 col weights, +0xBAC rows (u8),
      +0xBCB..+0xBE7 row weights, +0x208 parent (i16)

Only fields with documented offsets are decoded; everything else stays in
the .bin for later. No interpretation beyond the documented map.

Usage: python tools/decode_live_dump.py [burst_name ...]   (default: all)
"""
import json
import struct
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
LIVE = REPO / "reports" / "screen_captures" / "live"
W, H = 800, 600


def rgb565_to_png(raw: bytes, out: Path):
    from PIL import Image
    img = Image.new("RGB", (W, H))
    px = img.load()
    for y in range(H):
        row = y * W * 2
        for x in range(W):
            v = raw[row + x * 2] | (raw[row + x * 2 + 1] << 8)
            r5, g6, b5 = (v >> 11) & 0x1F, (v >> 5) & 0x3F, v & 0x1F
            px[x, y] = ((r5 << 3) | (r5 >> 2), (g6 << 2) | (g6 >> 4),
                        (b5 << 3) | (b5 >> 2))
    img.save(out)


def cstr(b: bytes) -> str:
    return b.split(b"\x00", 1)[0].decode("latin-1", "replace")


def decode_pool(meta: dict, blob: bytes) -> dict:
    na, nw = meta["n_areas"], meta["n_widgets"]
    sa, sw = meta["area_stride"], meta["widget_stride"]
    areas = []
    for i in range(na):
        r = blob[i * sa:(i + 1) * sa]
        l, t, rr, b = struct.unpack_from("<4i", r, 0)
        (flags,) = struct.unpack_from("<I", r, 0x18)
        cols, rows = r[0xBA9], r[0xBAC]
        (parent,) = struct.unpack_from("<h", r, 0x208)
        areas.append({
            "i": i, "L": l, "T": t, "R": rr, "B": b, "flags": flags,
            "cols": cols, "rows": rows, "parent": parent,
            "col_weights": list(r[0xBAB:0xBAB + max(0, min(cols, 0x1E))]),
            "row_weights": list(r[0xBCB:0xBCB + max(0, min(rows, 0x1E))]),
        })
    widgets = []
    off = na * sa
    for i in range(nw):
        r = blob[off + i * sw:off + (i + 1) * sw]
        # Empirical live-dump layout (burst_001 hexdump): the record opens
        # with two pointers (self/link), then post-layout rect at +0x10,
        # constructor-arg echoes around +0x28 (L,R,T,B), rflags at +0x38,
        # tflags at +0x3C, text copy at +0x80. The decode doc's offsets
        # were relative to a base 0x10 earlier.
        l, t, rr, b = struct.unpack_from("<4i", r, 0x10)
        (rflags,) = struct.unpack_from("<I", r, 0x38)
        (tflags,) = struct.unpack_from("<I", r, 0x3C)
        widgets.append({
            "i": i, "L": l, "T": t, "R": rr, "B": b,
            "rflags": rflags, "tflags": tflags,
            "text": cstr(r[0x80:0x80 + 0x30]),
        })
    return {"areas": areas, "widgets": widgets}


def main():
    names = sys.argv[1:]
    if not names:
        names = sorted(p.stem.replace(".pool", "")
                       for p in LIVE.glob("*.pool.bin"))
    done = 0
    for name in names:
        pool_bin = LIVE / f"{name}.pool.bin"
        pool_meta = LIVE / f"{name}.pool.json"
        frame_bin = LIVE / f"{name}.frame.bin"
        if pool_bin.exists() and pool_meta.exists():
            meta = json.loads(pool_meta.read_text())
            dec = decode_pool(meta, pool_bin.read_bytes())
            out = LIVE / f"{name}.final.json"
            out.write_text(json.dumps(dec, indent=1), encoding="utf-8")
            print(f"{name}: {len(dec['areas'])} areas, "
                  f"{len(dec['widgets'])} widgets -> {out.name}")
            done += 1
        if frame_bin.exists():
            raw = frame_bin.read_bytes()
            if len(raw) == W * H * 2:
                rgb565_to_png(raw, LIVE / f"{name}.frame.png")
                print(f"{name}: frame -> {name}.frame.png")
            else:
                print(f"{name}: frame size {len(raw)} != {W*H*2}, skipped")
    if not done:
        print("no pool dumps found -- run tools/frida_live_capture.py first")


if __name__ == "__main__":
    main()
