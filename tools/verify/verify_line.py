"""Prove `packed::PackedSurface::draw_line` matches `FUN_005cd3e0`
byte-for-byte, without corrupting the running game's display.

Approach: for each test case, save the destination rect, draw the exe's
line, read the resulting pixels, restore. Ship (input, before_pixels,
after_pixels) to a JSON fixture. Rust test replays each case through
our port and asserts byte-exact.

The tests are drawn into a small scratch region at the top-left (rows
0..40, cols 0..200). The exe's own screen is preserved by save/restore
around every test.
"""
from __future__ import annotations

import base64
import json
import sys
from pathlib import Path

import frida

# 30 test cases covering:
#  - horizontal (solid / dashed)
#  - vertical   (solid / dashed)
#  - diagonals  (all 4 quadrants, small & large)
#  - x-major vs y-major Bresenham branches
#  - endpoints reversed
#  - single-pixel line
CASES: list[dict] = [
    # (label, x0, y0, x1, y1, style, colour_555)
    {"label": "horiz solid short",  "x0": 10, "y0": 5,  "x1": 40, "y1": 5,  "style": 2, "colour": 0x7c00},
    {"label": "horiz dashed short", "x0": 10, "y0": 6,  "x1": 40, "y1": 6,  "style": 0, "colour": 0x03e0},
    {"label": "horiz solid long",   "x0": 5,  "y0": 7,  "x1": 190,"y1": 7,  "style": 2, "colour": 0x001f},
    {"label": "horiz dashed long",  "x0": 5,  "y0": 8,  "x1": 190,"y1": 8,  "style": 0, "colour": 0x7fff},
    {"label": "vert solid",         "x0": 50, "y0": 0,  "x1": 50, "y1": 35, "style": 2, "colour": 0x7c00},
    {"label": "vert dashed",        "x0": 55, "y0": 0,  "x1": 55, "y1": 35, "style": 0, "colour": 0x03e0},
    {"label": "diag \\  solid",     "x0": 0,  "y0": 0,  "x1": 39, "y1": 39, "style": 2, "colour": 0x7fff},
    {"label": "diag /  solid",      "x0": 0,  "y0": 39, "x1": 39, "y1": 0,  "style": 2, "colour": 0x7fff},
    {"label": "diag shallow +x",    "x0": 0,  "y0": 0,  "x1": 100,"y1": 30, "style": 2, "colour": 0x7c1f},
    {"label": "diag steep +y",      "x0": 0,  "y0": 0,  "x1": 30, "y1": 100,"style": 2, "colour": 0x03ff}, # y1=100 > 40 scratch; reduce
    {"label": "diag shallow -x",    "x0": 100,"y0": 0,  "x1": 0,  "y1": 30, "style": 2, "colour": 0x7c1f},
    {"label": "diag reversed",      "x0": 30, "y0": 30, "x1": 5,  "y1": 5,  "style": 2, "colour": 0x001f},
    {"label": "single pixel",       "x0": 100,"y0": 20, "x1": 100,"y1": 20, "style": 2, "colour": 0x7fff},
    {"label": "horiz dashed reversed","x0": 190,"y0": 10, "x1": 5,  "y1": 10, "style": 0, "colour": 0x7c00},
    {"label": "vert reversed",      "x0": 60, "y0": 35, "x1": 60, "y1": 0,  "style": 2, "colour": 0x03e0},
]

# Fix cases that go out of scratch region
for c in CASES:
    if c["y1"] > 39: c["y1"] = 39
    if c["y0"] > 39: c["y0"] = 39


SCRATCH_X0, SCRATCH_Y0 = 0, 0
SCRATCH_X1, SCRATCH_Y1 = 199, 39  # 200x40 scratch

SCRIPT = """
'use strict';
const lineFn = new NativeFunction(ptr('0x005cd3e0'), 'int',
    ['int', 'int', 'int', 'int', 'uint', 'uint16'], 'mscdecl');
const saveFn = new NativeFunction(ptr('0x005cd930'), 'pointer',
    ['int', 'int', 'int', 'int', 'pointer'], 'mscdecl');
const restoreFn = new NativeFunction(ptr('0x005cda90'), 'void',
    ['int', 'int', 'pointer'], 'mscdecl');
const B64 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
function b64(bytes) {
    const v = new Uint8Array(bytes); const n = v.length;
    const out = new Array(Math.ceil(n/3)*4); let o=0, i=0;
    for (; i+2 < n; i+=3) {
        const b1=v[i],b2=v[i+1],b3=v[i+2];
        out[o++]=B64[b1>>2]; out[o++]=B64[((b1&3)<<4)|(b2>>4)];
        out[o++]=B64[((b2&15)<<2)|(b3>>6)]; out[o++]=B64[b3&63];
    }
    if (i<n) {
        const b1=v[i], b2=(i+1<n)?v[i+1]:0;
        out[o++]=B64[b1>>2]; out[o++]=B64[((b1&3)<<4)|(b2>>4)];
        if (i+1<n) { out[o++]=B64[(b2&15)<<2]; out[o++]='='; }
        else { out[o++]='='; out[o++]='='; }
    }
    return out.join('');
}

function readScratch(x0, y0, x1, y1) {
    const buf = ptr('0x00ad6b1c').readPointer();
    const pitch = ptr('0x00acdeb8').readU16();
    const w = x1 - x0 + 1;
    const h = y1 - y0 + 1;
    const bytes = new Uint8Array(w * h * 2);
    for (let y = 0; y < h; y++) {
        const src = buf.add(((y0 + y) * pitch + x0) * 2);
        const row = src.readByteArray(w * 2);
        bytes.set(new Uint8Array(row), y * w * 2);
    }
    return b64(bytes.buffer);
}

rpc.exports = {
    runCase(c) {
        // Save the whole scratch region so we can restore after.
        const saved = saveFn(""" + f"{SCRATCH_X0}, {SCRATCH_Y0}, {SCRATCH_X1}, {SCRATCH_Y1}, " + """NULL);
        const before = readScratch(""" + f"{SCRATCH_X0}, {SCRATCH_Y0}, {SCRATCH_X1}, {SCRATCH_Y1}" + """);
        lineFn(c.x0, c.y0, c.x1, c.y1, c.style, c.colour);
        const after = readScratch(""" + f"{SCRATCH_X0}, {SCRATCH_Y0}, {SCRATCH_X1}, {SCRATCH_Y1}" + """);
        // Restore original pixels.
        restoreFn(""" + f"{SCRATCH_X0}, {SCRATCH_Y0}, " + """saved);
        return { before, after };
    },
};
"""


def main() -> int:
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 \
        else Path("D:/cm0102-rs/fixtures/verify_line.json")
    dev = frida.get_local_device()
    procs = [p for p in dev.enumerate_processes() if p.name.lower() == "cm0102_gdi.exe"]
    if not procs:
        print("cm0102_GDI.exe not running", file=sys.stderr); return 2
    s = dev.attach(procs[0].pid)
    scr = s.create_script(SCRIPT)
    scr.on("message", lambda m,_d: print(f"msg: {m}", flush=True))
    scr.load()
    api = scr.exports_sync
    print(f"running {len(CASES)} line cases on 200x40 scratch region...", flush=True)
    out: list[dict] = []
    for c in CASES:
        r = api.run_case(c)
        out.append({**c, "before_b64": r["before"], "after_b64": r["after"]})
        print(f"  {c['label']}", flush=True)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps({
        "va": "0x005cd3e0",
        "scratch": {"x0": SCRATCH_X0, "y0": SCRATCH_Y0, "x1": SCRATCH_X1, "y1": SCRATCH_Y1,
                    "width": SCRATCH_X1 - SCRATCH_X0 + 1, "height": SCRATCH_Y1 - SCRATCH_Y0 + 1},
        "cases": out,
    }))
    print(f"wrote {out_path}", flush=True)
    s.detach()
    return 0


if __name__ == "__main__":
    sys.exit(main())
