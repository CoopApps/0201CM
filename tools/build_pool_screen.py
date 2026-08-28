"""Join a live burst's post-layout pool with its constructor capture and
sample real colors from the frame grab -> a renderer-ready screen JSON.

Each output widget carries:
  - L,T,R,B   FINAL post-layout rect (from the pool record, +0x10)
  - font      captured font id (from the constructor object, same index)
  - rflags    captured style flags (constructor)
  - text      real string (constructor, whitespace-preserved)
  - fg        text color sampled from the frame at the widget's centre
  - bg        background color sampled just inside the widget's top-left

Alignment: the pool and constructor lists are the SAME widgets in creation
order (verified 557/558 text match on burst_063). We zip by index; text is
a sanity check, not the join key.

Colors are SAMPLED from the real back-buffer frame (RGB565 screenshot) —
ground truth, no palette-index decode needed (the News method, generalised).

Usage: python tools/build_pool_screen.py <burst_name> [<screen_name>]
"""
import json
import struct
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
LIVE = REPO / "reports" / "screen_captures" / "live"
OUT = REPO / "reports" / "screen_captures"
W, H = 800, 600


def clean(t):
    if not t:
        return ""
    for i, c in enumerate(t):
        if c == "�" or ord(c) < 9:
            return t[:i]
    return t


def load_frame(name):
    p = LIVE / f"{name}.frame.bin"
    if not p.exists():
        return None
    raw = p.read_bytes()
    if len(raw) != W * H * 2:
        return None
    return raw


def px(frame, x, y):
    """RGB888 tuple at (x,y) from the RGB565 frame, or None if OOB."""
    if frame is None or not (0 <= x < W and 0 <= y < H):
        return None
    o = (y * W + x) * 2
    v = frame[o] | (frame[o + 1] << 8)
    r5, g6, b5 = (v >> 11) & 0x1F, (v >> 5) & 0x3F, v & 0x1F
    return [(r5 << 3) | (r5 >> 2), (g6 << 2) | (g6 >> 4), (b5 << 3) | (b5 >> 2)]


def main():
    if len(sys.argv) < 2:
        sys.exit("usage: build_pool_screen.py <burst_name> [screen_name]")
    burst = sys.argv[1]
    screen = sys.argv[2] if len(sys.argv) > 2 else burst

    cons = json.loads((LIVE / f"{burst}.json").read_text(encoding="utf-8"))
    meta = json.loads((LIVE / f"{burst}.pool.json").read_text())
    blob = (LIVE / f"{burst}.pool.bin").read_bytes()
    frame = load_frame(burst)
    na, nw = meta["n_areas"], meta["n_widgets"]
    off = na * 0xBF1

    objs = cons["objects"]
    n = min(len(objs), nw)
    widgets = []
    for i in range(n):
        r = blob[off + i * 0x18C:off + (i + 1) * 0x18C]
        l, t, rr, b = struct.unpack_from("<4i", r, 0x10)
        if rr <= l or b <= t:
            continue  # no post-layout rect (hidden / detached this frame)
        o = objs[i]
        cx, cy = (l + rr) // 2, (t + b) // 2
        widgets.append({
            "L": l, "T": t, "R": rr, "B": b,
            "font": o["font"], "rflags": o["rflags"],
            "text": clean(o.get("text_str")),
            "fg": px(frame, cx, cy),
            "bg": px(frame, l + 2, t + 2),
        })

    # Table areas: real rects + multi-column grid, for row/column overlay.
    areas = []
    for i in range(na):
        r = blob[i * 0xBF1:(i + 1) * 0xBF1]
        l, t, rr, b = struct.unpack_from("<4i", r, 0)
        if rr <= l or b <= t:
            continue
        cols, rows = r[0xBA9], r[0xBAC]
        areas.append({
            "L": l, "T": t, "R": rr, "B": b,
            "flags": struct.unpack_from("<I", r, 0x18)[0],
            "cols": cols, "rows": rows,
            "col_weights": list(r[0xBAB:0xBAB + min(cols, 0x1E)]),
            "row_weights": list(r[0xBCB:0xBCB + min(rows, 0x1E)]),
        })

    out = {"screen": screen, "source_burst": burst,
           "has_frame": frame is not None,
           "widgets": widgets, "table_areas": areas}
    dst = OUT / f"{screen}.pool.json"
    dst.write_text(json.dumps(out, indent=1), encoding="utf-8")
    print(f"{screen}: {len(widgets)} placed widgets, {len(areas)} areas "
          f"(frame={'yes' if frame else 'NO'}) -> {dst.relative_to(REPO)}")


if __name__ == "__main__":
    main()
