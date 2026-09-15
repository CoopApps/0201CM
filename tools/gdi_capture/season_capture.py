#!/usr/bin/env python3
"""Full-season Frida capture driver for cm0102_GDI.exe.

Loads season_capture_manifest.json, attaches Frida to the running
GDI process, and relays every hook emission to a JSONL file.

Usage:
    D:/Python312/python.exe tools/gdi_capture/season_capture.py \
        --out season_2001_02.jsonl

    # Then in the game, holiday through 2001-08 → 2002-06-30.
    # Ctrl-C when done. The JSONL is complete.

Manifest editing:
    Open season_capture_manifest.json and toggle `enabled` on any
    hook. Hooks flagged with `dd_va_hint` (no `va`) are DirectDraw
    addresses that need a live GDI VA before enabling — see
    reports/c15_1_season_capture.md §VA resolution.

Status:
    Groups A + B ready. Groups C/D/E/F need one GDI VA resolution
    pass — inexpensive: attach with the resolver script and read
    the actual GDI addresses off a live IDA/Ghidra of the GDI
    binary (or ask the user).
"""

from __future__ import annotations
import argparse
import json
import sys
from pathlib import Path

try:
    import frida
except ImportError:
    print("ERROR: install frida-python: D:/Python312/python.exe -m pip install frida",
          file=sys.stderr)
    sys.exit(2)


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--process-name", default="cm0102_GDI.exe")
    ap.add_argument("--out", type=Path, default=Path("season_capture.jsonl"))
    ap.add_argument("--script", type=Path,
                    default=Path(__file__).parent / "season_capture.js")
    ap.add_argument("--manifest", type=Path,
                    default=Path(__file__).parent / "season_capture_manifest.json")
    return ap.parse_args()


def main() -> int:
    args = parse_args()
    if not args.script.exists():
        print(f"missing script: {args.script}", file=sys.stderr); return 2
    if not args.manifest.exists():
        print(f"missing manifest: {args.manifest}", file=sys.stderr); return 2

    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))

    # PRE-FLIGHT: every enabled hook must land on a real GDI function
    # start. Interceptor.attach at a mid-function address corrupts
    # nearby instructions and crashes the exe when execution walks
    # through them. See verify_manifest_vas.py.
    verifier = Path(__file__).parent / "verify_manifest_vas.py"
    if verifier.exists():
        import subprocess
        print("preflight: verifying every enabled hook VA …",
              file=sys.stderr)
        rc = subprocess.run(
            [sys.executable, str(verifier)],
            check=False,
        ).returncode
        if rc != 0:
            print("ABORT: at least one enabled hook targets a "
                  "non-function address. Fix the manifest, then rerun.",
                  file=sys.stderr)
            return 4

    print(f"attaching to {args.process_name} …", file=sys.stderr)
    try:
        session = frida.attach(args.process_name)
    except frida.ProcessNotFoundError:
        print(f"process {args.process_name} not running", file=sys.stderr)
        return 3

    js_src = args.script.read_text(encoding="utf-8")
    # Inject the manifest as a top-level `manifest` binding by
    # prepending a var declaration. Cleaner than juggling `send()`
    # round-trips at load time.
    injected = f"const manifest = {json.dumps(manifest)};\n\n" + js_src
    script = session.create_script(injected)

    out_f = args.out.open("w", encoding="utf-8")
    n_written = [0]

    def on_message(msg, _data):
        if msg.get("type") == "send":
            out_f.write(msg["payload"] + "\n")
            out_f.flush()
            n_written[0] += 1
            if n_written[0] % 1000 == 0:
                print(f"  {n_written[0]} events …", file=sys.stderr)
        elif msg.get("type") == "error":
            print("SCRIPT ERROR:", msg.get("description"), file=sys.stderr)

    script.on("message", on_message)
    script.load()
    print(
        "hooks armed. drive the game through the season and Ctrl-C when done.\n"
        f"  out → {args.out}\n"
        f"  manifest → {args.manifest}",
        file=sys.stderr,
    )

    try:
        sys.stdin.read()
    except KeyboardInterrupt:
        pass
    finally:
        out_f.close()
        try: session.detach()
        except Exception: pass
        print(f"finished. {n_written[0]} events written.", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
