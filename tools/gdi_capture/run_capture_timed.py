#!/usr/bin/env python3
"""Non-interactive capture — attach, start, sleep N seconds, stop, write.

Usage:
  python run_capture_timed.py <screen_name> <output.json> [seconds]

Unlike run_capture.py (which drives start/stop from Enter presses), this
version just captures whatever primitive calls the exe emits during a
fixed window. Good for automation and for cases where the tester can't
send keystrokes over the channel driving the script.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

import frida  # noqa: E402

PROCESS_DEFAULT = "cm0102_GDI.exe"
SCRIPT_PATH = Path(__file__).with_name("capture.js")


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("screen")
    p.add_argument("output", type=Path)
    p.add_argument("seconds", nargs="?", type=float, default=2.0,
                   help="Capture window duration in seconds (default 2.0)")
    p.add_argument("--process", default=PROCESS_DEFAULT)
    args = p.parse_args()

    device = frida.get_local_device()
    try:
        pid = next(pr.pid for pr in device.enumerate_processes()
                   if pr.name.lower() == args.process.lower())
    except StopIteration:
        sys.stderr.write(f"process {args.process!r} not running\n")
        return 2

    session = device.attach(pid)
    script = session.create_script(SCRIPT_PATH.read_text())

    messages = []
    script.on("message", lambda msg, data: messages.append(msg))
    script.load()
    api = script.exports_sync

    r = api.start(args.screen)
    if not r.get("ok"):
        sys.stderr.write(f"start failed: {r}\n")
        session.detach()
        return 3

    print(f"Capturing {args.screen!r} for {args.seconds}s...", file=sys.stderr)
    time.sleep(args.seconds)

    r = api.stop()
    if not r.get("ok"):
        sys.stderr.write(f"stop failed: {r}\n")
        session.detach()
        return 4
    fixture = r["fixture"]

    # Text calls stay as *_pending_font unless we've wired up the byte
    # reader — flag but don't fail.
    pending = sum(1 for c in fixture["calls"]
                  if c.get("op") in ("text_pending_font", "wrapped_text_pending_font"))
    if pending:
        print(f"note: {pending} text call(s) left as *_pending_font "
              f"(no rpc.readBytes yet — see README)", file=sys.stderr)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(fixture, indent=2))
    fb_bytes = len(fixture["before_b64"]) // 4 * 3
    print(f"wrote {args.output}: {len(fixture['calls'])} calls, "
          f"{fixture['meta']['width']}x{fixture['meta']['height']} @ "
          f"pitch={fixture['meta']['pitch_pixels']}, framebuffer={fb_bytes} bytes",
          file=sys.stderr)
    session.detach()
    return 0


if __name__ == "__main__":
    sys.exit(main())
