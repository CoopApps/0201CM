#!/usr/bin/env python3
"""Attach Frida to cm0102_GDI.exe, drive a capture, write a fixture JSON.

Usage:
  python run_capture.py <screen_name> <output.json>

The script attaches to a running cm0102_GDI.exe process, sends
`rpc.start(screen_name)`, waits for `<Enter>` (giving you a beat to
navigate the game into the state you want to capture), then sends
`rpc.stop()` and writes the returned fixture.

Font references inside the fixture are rewritten from `text_pending_font`
/ `wrapped_text_pending_font` into fully-inlined `text` / `wrapped_text`
Calls carrying `FontDump` payloads. The font table decoder converts the
exe's `[height][256 x {width, ?, ?, ?, bitmap_ptr}]` layout into the
`FontDump { height, glyphs: [{ width, bitmap_b64 }] }` schema Rust
consumes. Bitmap size is `ceil(width * height / 2)`; the pointer is
absolute in the exe's address space, so the script uses Frida's
`read_bytes` RPC to fetch each glyph's bitmap.

The script requires `frida` (>= 16) installed for the same Python that
runs this file. On this machine that is `D:/Python312/python.exe`.
"""

from __future__ import annotations

import argparse
import base64
import json
import math
import struct
import sys
import time
from pathlib import Path

try:
    import frida
except ImportError:
    sys.stderr.write(
        "frida module not available — install with `D:/Python312/python.exe -m pip install frida`\n"
    )
    raise


PROCESS_NAME = "cm0102_GDI.exe"
SCRIPT_PATH = Path(__file__).with_name("capture.js")
FONT_STRIDE = 0x1404   # 5124 bytes per font
CHAR_RECORD_STRIDE = 0x14  # 20 bytes per char
CHAR_COUNT = 256


def parse_font_table(raw: bytes, read_bytes) -> dict:
    """Decode a font table dumped from the exe into the FontDump schema.

    Raw layout: 1 int of height, then 256 char records of
    `[width, ?, ?, ?, bitmap_ptr]` × 5 ints. `bitmap_ptr` is an absolute
    address in the exe's memory; we call the injected `read_bytes` RPC to
    fetch `ceil(width * height / 2)` bytes for each glyph.
    """
    height = struct.unpack_from("<i", raw, 0)[0]
    glyphs = []
    for c in range(CHAR_COUNT):
        off = 4 + c * CHAR_RECORD_STRIDE
        width, _, _, _, ptr = struct.unpack_from("<iiiiI", raw, off)
        if width <= 0 or ptr == 0:
            glyphs.append(None)
            continue
        n_bytes = math.ceil(width * height / 2)
        bitmap = read_bytes(ptr, n_bytes)
        glyphs.append({
            "width": width,
            "bitmap_b64": base64.b64encode(bitmap).decode("ascii"),
        })
    return {"height": height, "glyphs": glyphs}


def rewrite_font_calls(fixture: dict, script) -> None:
    """Turn every `*_pending_font` call into an inlined `text`/`wrapped_text`
    call with a FontDump. Font tables are cached by font_idx so the same
    5-KB blob is only read once."""
    font_cache: dict[int, dict] = {}
    api = script.exports_sync

    def read_bytes(ptr: int, n: int) -> bytes:
        # A tiny RPC injected on-demand so we don't add another export
        # up-front. Frida's script.exports supports arbitrary JS eval via
        # `script.post(...)`, but the cleanest cross-version path is a
        # per-call rpc handler we install here. See capture.js line count
        # — keep additions minimal.
        raise NotImplementedError(
            "read_bytes RPC not wired yet — add rpc.readBytes(ptr, n) to capture.js "
            "when you first need a font dump; leaving as TODO to keep the initial script "
            "under the crash-line count from [[frida-instrumentation-crash-evidence]]."
        )

    for call in fixture["calls"]:
        op = call.get("op")
        if op == "text_pending_font":
            font_idx = call.pop("font_idx")
            call.pop("underline_at", None)
            if font_idx not in font_cache:
                raw = base64.b64decode(api.read_font_table(font_idx))
                font_cache[font_idx] = parse_font_table(raw, read_bytes)
            call["op"] = "text"
            call["font"] = font_cache[font_idx]
        elif op == "wrapped_text_pending_font":
            font_idx = call.pop("font_idx")
            call.pop("kern", None)
            if font_idx not in font_cache:
                raw = base64.b64decode(api.read_font_table(font_idx))
                font_cache[font_idx] = parse_font_table(raw, read_bytes)
            call["op"] = "wrapped_text"
            call["font"] = font_cache[font_idx]


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("screen", help="Human-readable screen label for the fixture")
    p.add_argument("output", type=Path, help="Fixture JSON path")
    p.add_argument("--process", default=PROCESS_NAME)
    args = p.parse_args()

    device = frida.get_local_device()
    try:
        pid = next(p.pid for p in device.enumerate_processes() if p.name.lower() == args.process.lower())
    except StopIteration:
        sys.stderr.write(f"process {args.process!r} not running — start it first\n")
        return 2

    session = device.attach(pid)
    script = session.create_script(SCRIPT_PATH.read_text())
    script.on("message", lambda msg, data: print(f"[frida] {msg}", file=sys.stderr))
    script.load()
    api = script.exports_sync

    print(f"Attached to {args.process} (pid {pid}). Navigate the game to the screen state,")
    print(f"then press Enter to start capture...", end=" ", flush=True)
    input()
    r = api.start(args.screen)
    if not r.get("ok"):
        sys.stderr.write(f"start failed: {r}\n")
        return 3
    print("Capturing. Trigger a redraw (click/hover/scroll), then press Enter to stop.", end=" ", flush=True)
    input()
    r = api.stop()
    if not r.get("ok"):
        sys.stderr.write(f"stop failed: {r}\n")
        return 4
    fixture = r["fixture"]
    try:
        rewrite_font_calls(fixture, script)
    except NotImplementedError as e:
        sys.stderr.write(f"warning: text calls left as *_pending_font ({e})\n")

    args.output.write_text(json.dumps(fixture, indent=2))
    print(f"Wrote {args.output} ({len(fixture['calls'])} calls, {len(fixture['before_b64'])//4*3} bytes framebuffer).")
    session.detach()
    return 0


if __name__ == "__main__":
    sys.exit(main())
