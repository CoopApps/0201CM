"""Capture the REAL DirectDraw primitive call stream from the running
cm0102.exe, so a screen can be reproduced by REPLAYING the game's own draw
commands through our ported primitives — not by hand-drawing or
reconstructing it.

This hooks the low-level 2D primitives (the 10 confirmed-role functions,
all already ported into cm-render) and records every call, in order, with
its exact arguments, during a screen's draw:

    0x005cd840  rect fill/outline   (x0,y0,x1,y1, u8 flags, u16 color)   -> Surface::fill_rect / draw_hollow_rect
    0x005cf8e0  panel/bevel frame   (x0,y0,x1,y1, u32 flags, u16 color, extra) -> Surface::draw_panel
    0x005cdfd0  darken/dim rect     (x0,y0,x1,y1)                        -> Surface::dim_region (60%)
    0x005ced50  glyph string        (x, y, u16 color, u32 a4, char* text, i32 a6) -> draw_string
    0x005d0870  word-wrap text box  (x0,y0,x1,y1, u32 flags, u16 color, u32 a7, char* text, i32 a9) -> draw_text_box
    0x005cd420  line                (x0,y0,x1,y1)                        -> draw_line

(Present-Blt 0x005ccba0 and fade 0x005ccdd0 only flip to screen — not
replayed. Save/restore-background 0x005cdac0/cc0 are popup bookkeeping —
added later if a screen needs them.)

Replaying this list through cm-render reproduces the screen PIXEL-EXACT,
because it IS the game's own draw sequence with the game's own arguments.

Safety + usage identical to tools/frida_live_capture.py: version-guarded
against the reference exe; auto-saves one <burst>.draws.json per screen
build. Text pointers are dereferenced at call time (raw hex; decoded
offline). Run: D:/Python312/python.exe tools/frida_draw_capture.py
"""
import json
import struct
import sys
import time
import threading
from pathlib import Path

import frida

REPO = Path(__file__).resolve().parents[1]
OUT_DIR = REPO / "reports" / "screen_captures" / "draws"
REF_EXE = Path("D:/cm0102/cm0102.exe")
PROC = "cm0102.exe"
IB = 0x400000

# name -> (addr, argc, text_arg_index_or_None)
# LEAF primitives only — the functions that actually write pixels, where the
# color arg is already resolved to RGB565 (composites rect/panel/textbox
# resolve color pointers + font, then call these). Capturing here gives clean
# resolved colors and exact pixel ops; depth-gating drops nested calls
# (e.g. glyph's own underline line).
#   line  005cd420(x0,y0,x1,y1, mode, u16 color)   -- Ghidra under-counted argc; real is 6
#   glyph 005ced50(x, y, a3, u16 color, char* text, font_idx)
#   darken 005cdfd0(x0,y0,x1,y1)
PRIMS = {
    "line":   (0x005CD420, 6, None),
    "glyph":  (0x005CED50, 6, 4),
    "darken": (0x005CDFD0, 4, None),
}
BURST_GAP = 0.4


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
            sys.exit(f"reference exe: 0x{v:08x} not in a section")
    return out


JS = r"""
'use strict';
const PRIMS = %PRIMS%;

const probe = {};
for (const k in PRIMS) probe[k] = ptr(PRIMS[k][0]).readByteArray(8);
send({ kind: 'probe' }, (function () {
    const names = Object.keys(PRIMS).sort();
    const total = new Uint8Array(names.length * 8);
    names.forEach((k, i) => total.set(new Uint8Array(probe[k]), i * 8));
    return total.buffer;
})());

let queue = [];
function flush() { if (queue.length) { const b = queue; queue = []; send({ kind: 'calls', events: b }); } }
setInterval(flush, 120);

recv('arm', function () {
    let seq = 0;
    // Shared call-depth across ALL primitive hooks. rect/panel/textbox are
    // composites that call line/glyph/darken internally (verified via the
    // decompile call graph). We record ONLY the OUTERMOST call (depth 0 on
    // enter) — the semantic draw op the screen's code issued — and replay
    // it through the matching ported composite, which reproduces the exact
    // internal pixel writes. This avoids double-drawing the decomposition.
    let depth = 0;
    for (const name in PRIMS) {
        const spec = PRIMS[name];
        const addr = spec[0], argc = spec[1], textArg = spec[2];
        Interceptor.attach(ptr(addr), {
            onEnter(args) {
                const outer = (depth === 0);
                depth++;
                if (!outer) return;   // nested inside another primitive
                const a = [];
                for (let i = 0; i < argc; i++) a.push(args[i].toInt32());
                let text = null;
                if (textArg !== null) {
                    try {
                        const p = args[textArg];
                        if (!p.isNull()) {
                            const b = new Uint8Array(p.readByteArray(96));
                            let h = '';
                            for (let i = 0; i < b.length; i++) h += b[i].toString(16).padStart(2, '0');
                            text = h;
                        }
                    } catch (e) {}
                }
                queue.push({ fn: name, seq: seq++, args: a, text: text });
                if (queue.length >= 800) flush();
            },
            onLeave() { depth--; }
        });
    }
    send({ kind: 'armed' });
});
"""


class Collector:
    def __init__(self):
        self.lock = threading.Lock()
        self.current = []
        self.last_t = 0.0
        self.n = 0

    def add(self, events):
        now = time.monotonic()
        with self.lock:
            if self.current and now - self.last_t > BURST_GAP:
                self._close()
            self.current.extend(events)
            self.last_t = now

    def _close(self):
        self.n += 1
        burst = sorted(self.current, key=lambda m: m["seq"])
        self.current = []
        counts = {}
        for m in burst:
            counts[m["fn"]] = counts.get(m["fn"], 0) + 1
        name = f"draws_{self.n:03d}"
        (OUT_DIR / f"{name}.json").write_text(
            json.dumps({"screen": name, "calls": burst}, indent=1),
            encoding="utf-8")
        summary = " ".join(f"{k}={v}" for k, v in sorted(counts.items()))
        print(f"draw-burst {self.n}: {summary} ({len(burst)} calls) -> {name}.json",
              flush=True)

    def flush_if_idle(self):
        with self.lock:
            if self.current and time.monotonic() - self.last_t > BURST_GAP:
                self._close()


def main():
    try:
        session = frida.attach(PROC)
    except frida.ProcessNotFoundError:
        sys.exit(f"{PROC} not running -- start the game first.")

    js = JS.replace("%PRIMS%", json.dumps({k: list(v) for k, v in PRIMS.items()}))
    script = session.create_script(js)
    armed = threading.Event()
    expected = ref_bytes_at([v[0] for v in PRIMS.values()])
    col = Collector()

    def on_message(msg, data):
        if msg["type"] == "error":
            print("[js-error]", msg.get("stack", msg), flush=True); return
        p = msg["payload"]
        if p["kind"] == "probe":
            names = sorted(PRIMS)
            ok = all(bytes(data[i*8:(i+1)*8]) == expected[PRIMS[k][0]]
                     for i, k in enumerate(names))
            if not ok:
                for i, k in enumerate(names):
                    live = bytes(data[i*8:(i+1)*8]); want = expected[PRIMS[k][0]]
                    if live != want:
                        print(f"VERSION MISMATCH {k} 0x{PRIMS[k][0]:08x}: "
                              f"live={live.hex()} want={want.hex()}")
                script.unload(); session.detach(); sys.exit(1)
            script.post({"type": "arm"})
        elif p["kind"] == "armed":
            armed.set()
        elif p["kind"] == "calls":
            col.add(p["events"])

    script.on("message", on_message)
    script.load()
    if not armed.wait(timeout=10):
        sys.exit("primitives did not arm")
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    print("draw-primitive capture armed + version-verified. Navigate the game; "
          "each screen's real draw-command stream saves to reports/screen_captures/draws/.",
          flush=True)
    try:
        while True:
            time.sleep(0.15)
            col.flush_if_idle()
    except KeyboardInterrupt:
        pass
    session.detach()
    print("detached -- game keeps running.")


if __name__ == "__main__":
    main()
