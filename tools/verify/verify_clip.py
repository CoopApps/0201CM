"""Prove `packed::PackedSurface::clip` matches `FUN_005cd330` byte-exact.

FUN_005cd330 has signature `bool FUN_005cd330(x0, y0, x1, y1, int *out)`.
Returns true and writes 4 ints (l, t, r, b) into `*out` on success;
returns false when the clipped rect is entirely off-screen.

The surface bounds are DAT_00ad6b40 x DAT_00ad6b08 (width, height in
pixels — 800x600 for the game). The clip clamps inclusively to
[0, W-1] x [0, H-1] and swaps reversed endpoints.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import frida

SCRIPT = """
'use strict';
const clipFn = new NativeFunction(ptr('0x005cd330'), 'int',
                                  ['int', 'int', 'int', 'int', 'pointer'],
                                  'mscdecl');
rpc.exports = {
    meta() {
        return {
            width:  ptr('0x00ad6b40').readS32(),
            height: ptr('0x00ad6b08').readS32(),
        };
    },
    clipMany(rects) {
        // Alloc a 16-byte scratch we reuse for every call.
        const scratch = Memory.alloc(16);
        const out = new Array(rects.length);
        for (let i = 0; i < rects.length; i++) {
            const [x0, y0, x1, y1] = rects[i];
            const ret = clipFn(x0, y0, x1, y1, scratch);
            if (ret) {
                out[i] = [
                    scratch.readInt(),
                    scratch.add(4).readInt(),
                    scratch.add(8).readInt(),
                    scratch.add(12).readInt(),
                ];
            } else {
                // Even when the exe returns false it still writes the
                // partially-clipped values; but for our semantics we
                // treat false as "no rect" and pass null through.
                out[i] = null;
            }
        }
        return out;
    },
};
"""


def build_matrix(w: int, h: int) -> list[tuple[int, int, int, int]]:
    """Cover in-bounds, off-top-left, off-bottom-right, reversed,
    zero-size, single-pixel, and fully-outside cases."""
    axes_x = [-100, -1, 0, 1, w // 4, w // 2, w - 2, w - 1, w, w + 1, w + 100]
    axes_y = [-100, -1, 0, 1, h // 4, h // 2, h - 2, h - 1, h, h + 1, h + 100]
    rects: list[tuple[int, int, int, int]] = []
    # Every (x0, y0, x1, y1) picked from a Cartesian product of both axes
    # — 11^4 = 14641 rects. Covers order-swap, clamps, empty, degenerate.
    for x0 in axes_x:
        for y0 in axes_y:
            for x1 in axes_x:
                for y1 in axes_y:
                    rects.append((x0, y0, x1, y1))
    return rects


def main() -> int:
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 \
        else Path("D:/cm0102-rs/fixtures/verify_clip.json")

    dev = frida.get_local_device()
    procs = [p for p in dev.enumerate_processes()
             if p.name.lower() == "cm0102_gdi.exe"]
    if not procs:
        print("cm0102_GDI.exe not running", file=sys.stderr); return 2
    s = dev.attach(procs[0].pid)
    scr = s.create_script(SCRIPT)
    scr.on("message", lambda m, _d: print(f"msg: {m}", flush=True))
    scr.load()
    api = scr.exports_sync
    meta = api.meta()
    print(f"surface: {meta['width']}x{meta['height']}", flush=True)

    rects = build_matrix(meta["width"], meta["height"])
    print(f"calling exe FUN_005cd330 {len(rects)} times...", flush=True)
    outs = api.clip_many(rects)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps({
        "va": "0x005cd330",
        "meta": meta,
        "cases": [
            {"input": list(r), "output": (list(o) if o is not None else None)}
            for r, o in zip(rects, outs)
        ],
    }))
    print(f"wrote {out_path} ({len(outs)} cases, "
          f"{sum(1 for o in outs if o)} clipped in-bounds, "
          f"{sum(1 for o in outs if o is None)} rejected)", flush=True)
    s.detach()
    return 0


if __name__ == "__main__":
    sys.exit(main())
