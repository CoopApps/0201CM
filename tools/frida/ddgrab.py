"""Grab the exact DirectDraw framebuffer via surface Lock. Attach to a running game
(or spawn one), call grab() once the target screen is showing, save as PNG."""
import frida, sys, os, time, struct
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = "D:/cm0102-rs/reports/carve_segment_index/renders/dd_framebuffer.png"

result = {}
def on_message(msg, data):
    if msg.get("type") == "send":
        result["meta"] = msg["payload"]
        result["data"] = data
    elif msg.get("type") == "error":
        print("SCRIPT ERROR:", msg.get("description"))

def main():
    wait = float(sys.argv[1]) if len(sys.argv) > 1 else 80
    attach = "--attach" in sys.argv
    if attach:
        session = frida.attach("cm0102.exe"); pid = None
    else:
        pid = frida.spawn(["D:/cm0102/cm0102.exe"], cwd="D:/cm0102")
        session = frida.attach(pid)
    script = session.create_script(open(os.path.join(HERE, "ddgrab.js"), encoding="utf-8").read())
    script.on("message", on_message)
    script.load()
    if pid is not None:
        frida.resume(pid)
    print(f"waiting {wait}s for the game to reach the target screen...")
    time.sleep(wait)
    print("grab:", script.exports_sync.grab())
    time.sleep(1.0)
    if pid is not None:
        try: frida.kill(pid)
        except Exception: pass

    if "data" in result and result["data"]:
        m = result["meta"]; pitch, W, H = m["pitch"], m["w"], m["h"]
        raw = result["data"]
        img = Image.new("RGB", (W, H)); px = img.load()
        for y in range(H):
            base = y * pitch
            for x in range(W):
                i = base + x * 2
                v = raw[i] | (raw[i + 1] << 8)
                r = (v >> 11) & 0x1f; g = (v >> 5) & 0x3f; b = v & 0x1f
                px[x, y] = (r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2)
        img.save(OUT)
        print("saved EXACT framebuffer ->", OUT)
    else:
        print("no framebuffer captured")

if __name__ == "__main__":
    main()
