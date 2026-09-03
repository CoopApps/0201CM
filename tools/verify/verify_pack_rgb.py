"""Prove `packed::pack_rgb` matches `FUN_005ce240` byte-for-byte.

Attaches to a running cm0102_GDI.exe, wraps FUN_005ce240 as a native
call, runs a matrix of (r, g, b) inputs, and writes a JSON fixture of
{input, exe_output} pairs. A Rust test (verify_pack_rgb_against_exe.rs)
then loads that fixture and asserts our port produces the same u16 for
every input.

Test matrix: every 4-bit-quantised (r, g, b) — 17^3 = 4913 triples that
cover every relevant bit pattern in the exe's per-channel arithmetic,
plus explicit edge cases (0, 255, and individual bit walks).
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import frida

SCRIPT = """
'use strict';
const packFn = new NativeFunction(ptr('0x005ce240'), 'uint32',
                                  ['uint8', 'uint32', 'uint32', 'pointer'],
                                  'mscdecl');
rpc.exports = {
    // The exe's function takes (r, g, b, pixel_format_desc*). Passing
    // NULL for the descriptor makes it fall back to the global format
    // (DAT_00acde98), i.e. the currently-installed masks.
    packMany(triples) {
        const out = new Array(triples.length);
        for (let i = 0; i < triples.length; i++) {
            const [r, g, b] = triples[i];
            out[i] = packFn(r, g, b, NULL) & 0xffff;
        }
        return out;
    },
    meta() {
        return {
            red_mask:   ptr('0x00acdea8').readU16(),
            green_mask: ptr('0x00acdeac').readU16(),
            blue_mask:  ptr('0x00acdeb0').readU16(),
        };
    },
};
"""


def build_matrix() -> list[tuple[int, int, int]]:
    """4913 quantised triples + 258 axis-aligned + edge/corner cases."""
    quantised = [
        (r, g, b)
        for r in range(0, 256, 16)
        for g in range(0, 256, 16)
        for b in range(0, 256, 16)
    ]
    # 0..255 walks on each channel (holding the others at 0 and 255)
    axes = []
    for c in range(256):
        axes.append((c, 0, 0))
        axes.append((0, c, 0))
        axes.append((0, 0, c))
        axes.append((c, 255, 255))
        axes.append((255, c, 255))
        axes.append((255, 255, c))
    # Explicit corners of the RGB cube
    corners = [
        (0, 0, 0), (255, 0, 0), (0, 255, 0), (0, 0, 255),
        (255, 255, 0), (255, 0, 255), (0, 255, 255), (255, 255, 255),
        # Bit-walk on each channel
        *[(1 << b, 0, 0) for b in range(8)],
        *[(0, 1 << b, 0) for b in range(8)],
        *[(0, 0, 1 << b) for b in range(8)],
    ]
    # De-dupe preserving order
    seen: set[tuple[int, int, int]] = set()
    ordered: list[tuple[int, int, int]] = []
    for t in quantised + axes + corners:
        if t in seen:
            continue
        seen.add(t)
        ordered.append(t)
    return ordered


def main() -> int:
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 \
        else Path("D:/cm0102-rs/fixtures/verify_pack_rgb.json")

    dev = frida.get_local_device()
    procs = [p for p in dev.enumerate_processes()
             if p.name.lower() == "cm0102_gdi.exe"]
    if not procs:
        print("cm0102_GDI.exe not running", file=sys.stderr); return 2
    print(f"attaching pid={procs[0].pid}", flush=True)
    s = dev.attach(procs[0].pid)
    scr = s.create_script(SCRIPT)
    scr.on("message", lambda m, _d: print(f"msg: {m}", flush=True))
    scr.load()
    meta = scr.exports_sync.meta()
    print(f"pixel format: R={meta['red_mask']:#06x} G={meta['green_mask']:#06x} "
          f"B={meta['blue_mask']:#06x}", flush=True)

    triples = build_matrix()
    print(f"calling exe FUN_005ce240 {len(triples)} times...", flush=True)
    outputs = scr.exports_sync.pack_many(triples)
    assert len(outputs) == len(triples)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps({
        "va": "0x005ce240",
        "meta": meta,
        "cases": [
            {"r": r, "g": g, "b": b, "packed": p}
            for (r, g, b), p in zip(triples, outputs)
        ],
    }, indent=1))
    print(f"wrote {out_path} ({len(outputs)} cases)", flush=True)
    s.detach()
    return 0


if __name__ == "__main__":
    sys.exit(main())
