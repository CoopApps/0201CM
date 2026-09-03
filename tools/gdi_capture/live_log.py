"""Attach the live primitive-call logger, stream events to stdout + a
log file, and exit after N seconds of silence (or a fixed max window).

Usage:
  python live_log.py <out.jsonl> [--max-seconds 10] [--silence-seconds 2]
                                 [--capture-framebuffers]

Framebuffer capture (News Milestone C):
  --capture-framebuffers arms the JS-side snapshotter. Every PRESENT hook
  then emits a `present_fb` event carrying the u16 LE framebuffer as
  base64 (in addition to the existing `--- PRESENT ---` marker, which
  fires first for compatibility). Each frame adds ~800*600*2 = ~960 KB
  raw → ~1.3 MB base64 → ~5 MB gzipped over 60 frames. Keep the capture
  window short.

Terminal print is suppressed for `present_fb` events (would flood the
console with a 1.3 MB line each) — only their op tag + width/height are
shown.
"""
from __future__ import annotations

import argparse
import gzip
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
    p.add_argument("--capture-framebuffers", action="store_true",
                   help="Emit a `present_fb` event with a base64 u16 LE "
                        "framebuffer at every PRESENT hook.")
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
        op = payload.get("op", "meta")
        # Framebuffer events are massive — never print the base64 blob.
        if op == "present_fb":
            print(f"{op:>10}  w={payload.get('width')} h={payload.get('height')} "
                  f"pitch={payload.get('pitch')} data_b64_len={len(payload.get('data_b64', ''))}",
                  flush=True)
            return
        summary = " ".join(f"{k}={v}" for k, v in payload.items() if k != "op")
        print(f"{op:>10}  {summary}", flush=True)

    scr.on("message", on_message)
    scr.load()
    if args.capture_framebuffers:
        # rpc.exports would have needed adding a whole recv/send round;
        # the JS uses a bare `recv('config', ...)` handler.
        scr.post({"type": "config", "capture_framebuffers": True})
        print("[capture-framebuffers ON — every PRESENT will emit a "
              "~1 MB base64 payload]", file=sys.stderr)

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
    text = "\n".join(json.dumps(e) for e in events)
    # Auto-gzip on `.gz` suffix — framebuffer captures are massive.
    if args.out.suffix == ".gz":
        with gzip.open(args.out, "wt", encoding="utf-8") as fp:
            fp.write(text)
    else:
        args.out.write_text(text)
    print(f"wrote {args.out} — {len(events)} event(s)", file=sys.stderr)
    session.detach()
    return 0


if __name__ == "__main__":
    sys.exit(main())
