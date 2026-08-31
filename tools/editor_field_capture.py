"""Spawn/attach cm0102ed.exe, install the field-capture Frida agent, stream
events to reports/editor_capture/events.jsonl, then AUTO-DRIVE the editor
through many records per type so the correlator has enough samples.

User does two things: (1) launch the editor + point at data folder for Import,
(2) open the first record type (e.g. Clubs) so the record navigator has focus.
Script does the rest — 500 Down-key presses per record type by default.

Usage:
    D:/Python312/python.exe tools/editor_field_capture.py
        # spawns editor. Do Import → open Clubs → press Enter in this terminal
        # then again for each record type you want to cycle
"""
import argparse, json, os, sys, time
from pathlib import Path

try:
    import frida
except ImportError:
    sys.exit("frida not installed. Run: D:/Python312/python.exe -m pip install frida-tools")

REPO = Path(__file__).resolve().parents[1]
AGENT = REPO / "tools/editor_field_agent.js"
OUT_DIR = REPO / "reports/editor_capture"
OUT_DIR.mkdir(parents=True, exist_ok=True)
LOG = OUT_DIR / "events.jsonl"
EDITOR_EXE = r"D:\cm0102\Editor\cm0102ed.exe"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--attach", action="store_true")
    ap.add_argument("--exe", default=EDITOR_EXE)
    ap.add_argument("--per-type", type=int, default=500,
                    help="records to auto-cycle per record type (default 500)")
    ap.add_argument("--interval-ms", type=int, default=60)
    args = ap.parse_args()

    device = frida.get_local_device()
    if args.attach:
        pid = next((p.pid for p in device.enumerate_processes()
                    if p.name.lower() == "cm0102ed.exe"), None)
        if not pid: sys.exit("cm0102ed.exe not running")
        session = device.attach(pid)
        print(f"[capture] attached pid {pid}")
    else:
        print(f"[capture] spawning {args.exe}")
        pid = device.spawn([args.exe])
        session = device.attach(pid)

    script = session.create_script(AGENT.read_text(encoding="utf-8"))
    total = {"read": 0, "text": 0, "drive_done": 0}
    fh = open(LOG, "w", encoding="utf-8")

    drive_state = {"awaiting": False}

    def on_message(msg, data):
        if msg.get("type") != "send": return
        p = msg["payload"]
        t = p.get("t")
        if t == "hello":
            print("[agent]", p.get("msg"))
            return
        if t == "drive_done":
            drive_state["awaiting"] = False
            print(f"[drive] completed batch — {p.get('pressed')} keys")
            return
        if t in total: total[t] += 1
        # Trim big reads
        if t == "read" and len(p.get("hex","")) > 4*1024:
            p["hex"] = p["hex"][:4*1024]; p["truncated"] = True
        fh.write(json.dumps(p) + "\n")
        n = total["read"] + total["text"]
        if n % 500 == 0:
            print(f"[capture] reads={total['read']}  texts={total['text']}")

    script.on("message", on_message)
    script.load()
    if not args.attach:
        device.resume(pid)

    print(f"[capture] streaming → {LOG}")
    print()
    print("=========================================================================")
    print("STEP 1: In the editor, do File → Import → point at D:/cm0102/data/")
    print("STEP 2: Open the first record type (e.g. View → Clubs, or the toolbar)")
    print("STEP 3: Click into the record navigator (arrow/spinner) so it has focus")
    print(f"STEP 4: press Enter HERE to auto-cycle {args.per_type} records")
    print("        Repeat step 2-4 for each record type you want the schema for.")
    print("        (Type 'quit' + Enter when done.)")
    print("=========================================================================")

    while True:
        try:
            cmd = input("\n[capture] press Enter to auto-cycle (or 'quit'): ").strip().lower()
        except EOFError:
            break
        if cmd == "quit":
            break
        # Ask the agent which windows are visible.
        try:
            wins = script.exports_sync.list_windows()
        except Exception as e:
            print(f"[capture] listWindows failed: {e}")
            continue
        # Print ALL for visibility.
        print(f"[capture] found {len(wins)} visible windows:")
        for w in wins:
            print(f"    hwnd=0x{w['hwnd']:08x}  cls={w['cls']!r:<32} title={w['title']!r}")
        # Filter out Delphi's invisible top-level TApplication. Prefer actual
        # record-editor forms (Delphi form classes usually start with 'TFrm'
        # or the title includes something like "Clubs" / "Player"). Fall back
        # to any T* form with a non-empty title that isn't TApplication.
        skip_cls = {"tapplication", "tstartbar", "ttoolwindow"}
        candidates = [w for w in wins
                      if w.get("cls","").lower() not in skip_cls
                      and w.get("title","")
                      and w.get("cls","").lower().startswith("t")]
        if not candidates:
            print("[capture] NO record-editor form found. Open one first:")
            print("          View menu → Clubs (or Players, Staff, etc.)")
            print("          Then press Enter again.")
            continue
        # If multiple forms are up, use the last one (most-recently-shown).
        target = candidates[-1]
        print(f"[capture] auto-drive → hwnd 0x{target['hwnd']:x} cls={target['cls']!r} title={target['title']!r}")
        drive_state["awaiting"] = True
        script.exports_sync.drive_down(target["hwnd"], args.per_type, args.interval_ms)
        # Wait for completion.
        while drive_state["awaiting"]:
            time.sleep(0.2)

    fh.close()
    print(f"\n[capture] DONE — reads={total['read']} texts={total['text']}  → {LOG}")

if __name__ == "__main__":
    main()
