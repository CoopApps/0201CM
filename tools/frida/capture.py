"""CM0102 Frida capture runner — the ground-truth backbone.

Launches (or attaches to) the game, installs the hook categories, records the
stream to a jsonl, and prints a summary. This is the oracle the whole project
verifies against (GUI draw stream, match-engine I/O, RNG, view-model bindings).

Usage:
  python capture.py --secs 60 --cats draw,font,view       # GUI + fonts
  python capture.py --secs 90 --cats match,rng --attach    # attach to a running game
"""
import argparse, json, os, sys, time
import frida

HERE = os.path.dirname(os.path.abspath(__file__))
EXE = "D:/cm0102/cm0102.exe"
CWD = "D:/cm0102"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--secs", type=float, default=60)
    ap.add_argument("--cats", default="draw,font,view", help="comma list: draw,font,view,match,rng")
    ap.add_argument("--attach", action="store_true", help="attach to a running cm0102.exe instead of spawning")
    ap.add_argument("--out", default=os.path.join(HERE, "capture.jsonl"))
    a = ap.parse_args()
    cfg = {c: True for c in a.cats.split(",") if c}

    records = []
    fb = {}
    def on_message(msg, data):
        if msg.get("type") == "send":
            p = msg["payload"]
            if p.get("t") == "framebuffer":
                fb["meta"] = p; fb["data"] = data
            else:
                records.append(p)
        elif msg.get("type") == "error":
            print("SCRIPT ERROR:", msg.get("description"))

    if a.attach:
        session = frida.attach("cm0102.exe")
        pid = None
    else:
        pid = frida.spawn([EXE], cwd=CWD)
        session = frida.attach(pid)
    script = session.create_script(open(os.path.join(HERE, "hooks.js"), encoding="utf-8").read())
    script.on("message", on_message)
    script.load()
    script.exports_sync.start(cfg)
    if pid is not None:
        frida.resume(pid)
    print(f"capturing {a.secs}s  cats={list(cfg)}  (navigate the game to the target screen now)")
    time.sleep(a.secs)
    # grab the exact framebuffer of the CURRENT frame (same frame as the tail of the draw stream)
    try:
        print("framebuffer grab:", script.exports_sync.grab())
        time.sleep(0.5)
    except Exception as e:
        print("grab failed:", e)
    if pid is not None:
        try: frida.kill(pid)
        except Exception: pass

    with open(a.out, "w", encoding="utf-8") as f:
        for r in records:
            f.write(json.dumps(r) + "\n")
    if fb.get("data"):
        from PIL import Image
        m = fb["meta"]; raw = fb["data"]; pitch = m["pitch"]
        img = Image.new("RGB", (800, 600)); px = img.load()
        for y in range(600):
            base = y * pitch
            for x in range(800):
                i = base + x * 2
                v = raw[i] | (raw[i + 1] << 8)
                r = (v >> 11) & 0x1f; g = (v >> 5) & 0x3f; b = v & 0x1f
                px[x, y] = (r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2)
        fbp = os.path.join(HERE, "capture_framebuffer.png")
        img.save(fbp); print("framebuffer ->", fbp)
    from collections import Counter
    print("captured:", dict(Counter(r["t"] for r in records)), "->", a.out)

if __name__ == "__main__":
    main()
