"""Live screen capture from the REAL running cm0102.exe via Frida — v2.

v1 hooked the 4 constructors and saved each screen build as a burst.
v2 adds, at the end of every burst (the screen is finished drawing):

  1. WIDGET-POOL DUMP — raw bytes of the GUI record pool (base 0xB59FE8;
     areas at +4 stride 0xBF1 count@+0x12E99E; widgets at +0xBA95E stride
     0x18C count@+0x12E9A0 — reports/gui_layout_engine_decode.md). This
     holds every widget's FINAL post-layout rect: table cells, pagers,
     dropdowns — everything the pre-layout constructor args can't show.
     Decoded offline; the capture stores raw slabs, zero interpretation.
  2. FRAME GRAB — the full back buffer (DAT_00AD6BD4) via its own Lock
     vtable method (idx 25, DDSURFACEDESC dwSize=0x6C, lpSurface@0x24,
     lPitch@0x10, RGB565), the exact recipe validated in the News color
     session (DIRECTDRAW_CAPTURE_HANDOVER.md §11). A true screenshot of
     what the exe rendered — ground truth for colors and pixel diffs.
  3. Constructor events are BATCHED (one send per ~150ms, not per call)
     to keep long sessions light on the game process.
  4. Text is captured as raw bytes (hex), decoded offline — v1's
     readCString run mangled non-ASCII tails.

Safety: before installing anything, the first 8 bytes at every hook
address in the LIVE process are compared against the reference exe
(D:/cm0102/cm0102.exe). Mismatch = wrong build; abort untouched.

Usage:
    1) Start the game, load a save.
    2) D:/Python312/python.exe tools/frida_live_capture.py --auto
    3) Play. Every screen build auto-saves:
         reports/screen_captures/live/burst_NNN.json        constructor stream
         reports/screen_captures/live/burst_NNN.pool.bin    raw pool slabs
         reports/screen_captures/live/burst_NNN.frame.bin   RGB565 frame
    4) Ctrl+C when done; the game keeps running.
"""
import json
import struct
import sys
import time
import threading
from pathlib import Path

import frida

REPO = Path(__file__).resolve().parents[1]
OUT_DIR = REPO / "reports" / "screen_captures" / "live"
REF_EXE = Path("D:/cm0102/cm0102.exe")
PROC = "cm0102.exe"
IB = 0x400000

HOOKS = {
    "guio": (0x00549580, 18),
    "area": (0x00549790, 11),
    "tabs": (0x005D7070, 6),
    "nav":  (0x005D75B0, 2),
}

AREA_FIELDS = ["L", "T", "R", "B", "cntA", "wA", "cntB", "wB", "flags", "a10", "parent"]
GUIO_FIELDS = ["type", "L", "T", "R", "B", "a6", "a7", "rflags",
               "colP", "colS", "tflags", "font", "tmode", "text",
               "a15", "a16", "a17", "parent"]
GUIO_TEXT_ARG = GUIO_FIELDS.index("text")

BURST_GAP = 0.4   # seconds of silence that ends a screen-build burst
FRAME_W, FRAME_H = 800, 600


def ref_bytes_at(vaddrs, n=8):
    exe = REF_EXE.read_bytes()
    pe = struct.unpack_from("<I", exe, 0x3C)[0]
    nsec = struct.unpack_from("<H", exe, pe + 6)[0]
    opt = struct.unpack_from("<H", exe, pe + 20)[0]
    so = pe + 24 + opt
    secs = []
    for i in range(nsec):
        o = so + i * 40
        vsz, va, rawsz, rawp = struct.unpack_from("<IIII", exe, o + 8)
        secs.append((va, vsz, rawsz, rawp))
    out = {}
    for v in vaddrs:
        rva = v - IB
        for va, vsz, rawsz, rawp in secs:
            if va <= rva < va + rawsz:
                out[v] = exe[rawp + (rva - va):rawp + (rva - va) + n]
                break
        else:
            sys.exit(f"reference exe: 0x{v:08x} not in any section")
    return out


JS = r"""
'use strict';
const HOOKS = %HOOKS%;
const GUIO_TEXT_ARG = %TEXT_ARG%;

// Pool globals (reports/gui_layout_engine_decode.md).
const POOL_BASE   = ptr('0xB59FE8');
const AREA_OFF    = 4,        AREA_STRIDE = 0xBF1, AREA_CNT_OFF = 0x12E99E, AREA_CAP = 0xF8;
const WIDGET_OFF  = 0xBA95E,  WIDGET_STRIDE = 0x18C, WIDGET_CNT_OFF = 0x12E9A0, WIDGET_CAP = 0x4AF;
// Back buffer (DIRECTDRAW_CAPTURE_HANDOVER.md §11).
const BACK_SURF_PP = ptr('0xAD6BD4');
const VTBL_LOCK = 25, VTBL_UNLOCK = 32;
const W = 800, H = 600;

// --- version guard ---------------------------------------------------------
const probe = {};
for (const k in HOOKS) probe[k] = ptr(HOOKS[k][0]).readByteArray(8);
send({ kind: 'probe' }, (function () {
    const names = Object.keys(HOOKS).sort();
    const total = new Uint8Array(names.length * 8);
    names.forEach((k, i) => total.set(new Uint8Array(probe[k]), i * 8));
    return total.buffer;
})());

// --- batched constructor events -------------------------------------------
let queue = [];
function flush() {
    if (!queue.length) return;
    const batch = queue; queue = [];
    send({ kind: 'calls', events: batch });
}
setInterval(flush, 150);

recv('arm', function () {
    let seq = 0;
    for (const name in HOOKS) {
        const [addr, nargs] = HOOKS[name];
        Interceptor.attach(ptr(addr), {
            onEnter(args) {
                const a = [];
                for (let i = 0; i < nargs; i++) a.push(args[i].toInt32());
                let text = null;
                if (name === 'guio') {
                    // Raw hex bytes; terminator decoded offline (v1 lesson:
                    // readCString's UTF-8 decode mangled latin-1 strings).
                    try {
                        const p = args[GUIO_TEXT_ARG];
                        if (!p.isNull()) {
                            const b = new Uint8Array(p.readByteArray(128));
                            let h = '';
                            for (let i = 0; i < b.length; i++)
                                h += b[i].toString(16).padStart(2, '0');
                            text = h;
                        }
                    } catch (e) {}
                }
                if (name === 'tabs') {
                    try { a.push(args[3].isNull() ? null : args[3].readS32()); } catch (e) { a.push(null); }
                    try { a.push(args[4].isNull() ? null : args[4].readS32()); } catch (e) { a.push(null); }
                }
                queue.push({ fn: name, seq: seq++, args: a, text: text,
                             caller: this.returnAddress.toString() });
                if (queue.length >= 500) flush();
            }
        });
    }
    send({ kind: 'armed' });
});

// --- burst-end dump: raw pool slabs + back-buffer frame --------------------
recv('dump', function onDump(msg) {
    const burst = msg.burst;
    try {
        // DAT_00B59FE8 holds a POINTER to the pool struct (verified live:
        // *0xB59FE8 = 0xB5D030, counts through it match the constructor
        // stream exactly).
        const pool = POOL_BASE.readPointer();
        let na = pool.add(AREA_CNT_OFF).readS16();
        let nw = pool.add(WIDGET_CNT_OFF).readS16();
        if (na < 0 || na > AREA_CAP) na = 0;
        if (nw < 0 || nw > WIDGET_CAP) nw = 0;
        const areas = na ? pool.add(AREA_OFF).readByteArray(na * AREA_STRIDE) : new ArrayBuffer(0);
        const widgets = nw ? pool.add(WIDGET_OFF).readByteArray(nw * WIDGET_STRIDE) : new ArrayBuffer(0);
        const total = new Uint8Array(areas.byteLength + widgets.byteLength);
        total.set(new Uint8Array(areas), 0);
        total.set(new Uint8Array(widgets), areas.byteLength);
        send({ kind: 'pool', burst: burst, n_areas: na, n_widgets: nw,
               area_stride: AREA_STRIDE, widget_stride: WIDGET_STRIDE }, total.buffer);
    } catch (e) {
        send({ kind: 'pool-error', burst: burst, error: '' + e });
    }
    try {
        const surf = BACK_SURF_PP.readPointer();
        const vtbl = surf.readPointer();
        const Lock = new NativeFunction(vtbl.add(VTBL_LOCK * 4).readPointer(),
            'int', ['pointer', 'pointer', 'pointer', 'uint', 'pointer'], 'stdcall');
        const Unlock = new NativeFunction(vtbl.add(VTBL_UNLOCK * 4).readPointer(),
            'int', ['pointer', 'pointer'], 'stdcall');
        const desc = Memory.alloc(0x6C);
        desc.writeU32(0x6C);
        // DDLOCK_WAIT(1) | DDLOCK_READONLY(0x10)
        const hr = Lock(surf, NULL, desc, 0x11, NULL);
        if (hr === 0) {
            const lp = desc.add(0x24).readPointer();
            const pitch = desc.add(0x10).readS32();
            const frame = new Uint8Array(W * H * 2);
            for (let y = 0; y < H; y++) {
                frame.set(new Uint8Array(lp.add(y * pitch).readByteArray(W * 2)), y * W * 2);
            }
            Unlock(surf, NULL);
            send({ kind: 'frame', burst: burst, w: W, h: H }, frame.buffer);
        } else {
            send({ kind: 'frame-error', burst: burst, error: 'Lock hr=0x' + (hr >>> 0).toString(16) });
        }
    } catch (e) {
        send({ kind: 'frame-error', burst: burst, error: '' + e });
    }
    recv('dump', onDump);   // re-arm for the next dump request
});
"""


class Collector:
    def __init__(self, script):
        self.lock = threading.Lock()
        self.script = script
        self.current = []
        self.last_t = 0.0
        self.burst_no = 0

    def add_batch(self, events):
        now = time.monotonic()
        with self.lock:
            if self.current and now - self.last_t > BURST_GAP:
                self._close()
            self.current.extend(events)
            self.last_t = now

    def _close(self):
        self.burst_no += 1
        burst = self.current
        self.current = []
        n = {}
        for m in burst:
            n[m["fn"]] = n.get(m["fn"], 0) + 1
        name = f"burst_{self.burst_no:03d}"
        out = OUT_DIR / f"{name}.json"
        out.write_text(json.dumps(burst_to_json(name, burst), indent=1),
                       encoding="utf-8")
        summary = " ".join(f"{k}={v}" for k, v in sorted(n.items()))
        print(f"burst {self.burst_no}: {summary} -> {out.name} (+pool +frame)",
              flush=True)
        # The screen has been idle >= BURST_GAP: layout has run and the back
        # buffer holds the finished frame. Ask the agent for both dumps.
        self.script.post({"type": "dump", "burst": name})

    def flush_if_idle(self):
        with self.lock:
            if self.current and time.monotonic() - self.last_t > BURST_GAP:
                self._close()


def burst_to_json(name, burst):
    areas, objects, tabs, nav = [], [], [], []
    next_id = 1
    for m in sorted(burst, key=lambda m: m["seq"]):
        fn, a = m["fn"], m["args"]
        if fn == "area":
            rec = dict(id=next_id, **{AREA_FIELDS[i]: a[i] for i in range(11)})
            rec["caller"] = m["caller"]
            areas.append(rec); next_id += 1
        elif fn == "guio":
            rec = dict(id=next_id, **{GUIO_FIELDS[i]: a[i] for i in range(18)})
            raw = m.get("text")
            rec["text_hex"] = raw
            if raw:
                b = bytes.fromhex(raw).split(b"\x00", 1)[0]
                rec["text_str"] = b.decode("latin-1", "replace")
            else:
                rec["text_str"] = None
            rec["caller"] = m["caller"]
            objects.append(rec); next_id += 1
        elif fn == "tabs":
            tabs.append({"count": a[1], "sel": a[2],
                         "top_y_in": a[6], "bot_y_in": a[7],
                         "split": a[5], "p4": a[3], "p5": a[4],
                         "caller": m["caller"]})
        elif fn == "nav":
            nav.append({"back": a[0], "next": a[1], "caller": m["caller"]})
    return {"screen": name, "areas": areas, "objects": objects,
            "tabs": tabs, "nav": nav, "slots": [],
            "source": "frida-live-v2"}


def main():
    try:
        session = frida.attach(PROC)
    except frida.ProcessNotFoundError:
        sys.exit(f"{PROC} is not running -- start the game first.")

    js = JS.replace("%HOOKS%", json.dumps(
        {k: [v[0], v[1]] for k, v in HOOKS.items()})).replace(
        "%TEXT_ARG%", str(GUIO_TEXT_ARG))
    script = session.create_script(js)

    armed = threading.Event()
    expected = ref_bytes_at([v[0] for v in HOOKS.values()])
    col = Collector(script)

    def on_message(msg, data):
        if msg["type"] == "error":
            print("[js-error]", msg.get("stack", msg), flush=True)
            return
        p = msg["payload"]
        kind = p["kind"]
        if kind == "probe":
            names = sorted(HOOKS)
            ok = True
            for i, k in enumerate(names):
                live = bytes(data[i * 8:(i + 1) * 8])
                want = expected[HOOKS[k][0]]
                if live != want:
                    ok = False
                    print(f"VERSION MISMATCH at {k} 0x{HOOKS[k][0]:08x}: "
                          f"live={live.hex()} expected={want.hex()}")
            if not ok:
                print("Running exe does not match the decompiled reference. "
                      "NOT installing hooks.")
                script.unload(); session.detach(); sys.exit(1)
            script.post({"type": "arm"})
        elif kind == "armed":
            armed.set()
        elif kind == "calls":
            col.add_batch(p["events"])
        elif kind == "pool":
            meta = {k: p[k] for k in ("n_areas", "n_widgets", "area_stride",
                                      "widget_stride")}
            (OUT_DIR / f"{p['burst']}.pool.json").write_text(
                json.dumps(meta), encoding="utf-8")
            (OUT_DIR / f"{p['burst']}.pool.bin").write_bytes(bytes(data))
        elif kind == "frame":
            (OUT_DIR / f"{p['burst']}.frame.bin").write_bytes(bytes(data))
        elif kind in ("pool-error", "frame-error"):
            print(f"[{kind}] {p['burst']}: {p['error']}", flush=True)

    script.on("message", on_message)
    script.load()
    if not armed.wait(timeout=10):
        sys.exit("hooks did not arm")
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    print("v2 armed and version-verified. Play the game; every screen build "
          "saves constructor stream + post-layout pool dump + frame grab.",
          flush=True)

    try:
        while True:
            time.sleep(0.2)
            col.flush_if_idle()
    except KeyboardInterrupt:
        pass
    session.detach()
    print("detached -- game keeps running.")


if __name__ == "__main__":
    main()
