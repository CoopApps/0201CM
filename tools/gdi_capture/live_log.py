"""Attach the live primitive-call logger, stream events to stdout + a
log file, and exit after N seconds of silence (or a fixed max window).

Usage:
  python live_log.py <out.jsonl> [--max-seconds 10] [--silence-seconds 2]
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

import frida

SCRIPT = Path(__file__).with_name("live_log.js")


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("out", type=Path)
    p.add_argument("--max-seconds", type=float, default=10.0)
    p.add_argument("--silence-seconds", type=float, default=2.0)
    p.add_argument("--process", default="cm0102_GDI.exe")
    args = p.parse_args()

    dev = frida.get_local_device()
    try:
        pid = next(pr.pid for pr in dev.enumerate_processes()
                   if pr.name.lower() == args.process.lower())
    except StopIteration:
        print(f"process {args.process!r} not running", file=sys.stderr); return 2

    session = dev.attach(pid)
    scr = session.create_script(SCRIPT.read_text())

    events: list[dict] = []
    last_event = [time.monotonic()]

    def on_message(msg, data):
        if msg["type"] != "send": return
        payload = msg["payload"]
        events.append(payload)
        last_event[0] = time.monotonic()
        # Compact one-liner for the terminal.
        op = payload.get("op", "meta")
        summary = " ".join(f"{k}={v}" for k, v in payload.items() if k != "op")
        print(f"{op:>10}  {summary}", flush=True)

    scr.on("message", on_message)
    scr.load()

    print(f"logging {args.process} pid={pid} for up to {args.max_seconds}s "
          f"(exit after {args.silence_seconds}s of silence). Interact with the game NOW.",
          file=sys.stderr, flush=True)

    start = time.monotonic()
    while True:
        now = time.monotonic()
        if now - start >= args.max_seconds:
            print("[max-seconds reached]", file=sys.stderr); break
        if now - last_event[0] >= args.silence_seconds and (now - start) > 1.0:
            print(f"[{args.silence_seconds}s silence — stopping]", file=sys.stderr); break
        time.sleep(0.1)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text("\n".join(json.dumps(e) for e in events))
    print(f"wrote {args.out} — {len(events)} event(s)", file=sys.stderr)
    session.detach()
    return 0


if __name__ == "__main__":
    sys.exit(main())
