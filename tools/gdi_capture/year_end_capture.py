#!/usr/bin/env python3
"""Attach Frida to cm0102_GDI.exe and record the year-end call
chain across a 2001→2002 year boundary.

Usage:
    D:/Python312/python.exe tools/gdi_capture/year_end_capture.py \
        --process-name cm0102_GDI.exe \
        --out year_end.jsonl

    # Then in the game, tick past 30 June 2002 (end of English
    # season). The script emits one JSON record per hooked event.
    # When the year has rolled, kill the process; the JSONL will
    # be complete.

Post-processing:
    Feed the JSONL into the C15.1 differential comparer (see
    reports/c15_1_runtime_differential.md — pending).

Status:
    - Harness READY.
    - Requires the user to run against a real running GDI process.
    - Not executed in-agent (no exe present in this environment).
"""

from __future__ import annotations
import argparse
import json
import sys
from pathlib import Path

try:
    import frida
except ImportError:
    print(
        "ERROR: frida-python not installed. Install with:\n"
        "  D:/Python312/python.exe -m pip install frida",
        file=sys.stderr,
    )
    sys.exit(2)


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--process-name", default="cm0102_GDI.exe")
    ap.add_argument("--out", type=Path, default=Path("year_end.jsonl"))
    ap.add_argument("--script", type=Path,
                    default=Path(__file__).parent / "year_end_capture.js")
    return ap.parse_args()


def main() -> int:
    args = parse_args()
    if not args.script.exists():
        print(f"ERROR: script not found: {args.script}", file=sys.stderr)
        return 2

    print(f"attaching to {args.process_name} …", file=sys.stderr)
    try:
        session = frida.attach(args.process_name)
    except frida.ProcessNotFoundError:
        print(
            f"ERROR: process {args.process_name!r} not running. "
            "Launch cm0102_GDI.exe first.",
            file=sys.stderr,
        )
        return 3

    js = args.script.read_text(encoding="utf-8")
    script = session.create_script(js)

    out_f = args.out.open("w", encoding="utf-8")

    def on_message(msg, _data):
        if msg.get("type") == "send":
            out_f.write(msg["payload"] + "\n")
            out_f.flush()
        elif msg.get("type") == "error":
            print("SCRIPT ERROR:", msg.get("description"), file=sys.stderr)

    script.on("message", on_message)
    script.load()
    print(
        "hooked. tick the game past the 2001-02 season end (30 Jun 2002) "
        "then close it to finalise the capture. output → "
        f"{args.out}", file=sys.stderr,
    )

    try:
        # Idle — hooks run inside the game; we just relay the log.
        sys.stdin.read()
    except KeyboardInterrupt:
        pass
    finally:
        out_f.close()
        try: session.detach()
        except Exception: pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
