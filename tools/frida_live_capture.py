"""Live screen capture from the REAL running cm0102.exe via Frida.

Replaces the Unicorn empty-state emulation as the capture source
(DIRECTDRAW_CAPTURE_HANDOVER.md §11: the emulation route only ever sees
the pre-loop chrome; the live game has real state, real text, real
everything). Same 4 light hooks that ran crash-free in the News session:

    GUIO 0x549580   widget constructor   (18 args)
    AREA 0x549790   area constructor     (11 args)
    TABS 0x5D7070   tab-strip builder    (6 args)
    NAV  0x5D75B0   bottom nav bar       (2 args)

Safety: before installing any hook, the first 8 bytes at every hook
address in the LIVE process are compared against the same bytes in the
decompiled reference exe (D:/cm0102/cm0102.exe). Any mismatch = wrong
exe version (e.g. 3.9.68 vs 3.9.60) and the script aborts untouched.

Text pointers are dereferenced AT CONSTRUCTION TIME inside the hook --
many widgets share one scratch buffer that the next construction
overwrites (§11), so reading later shows only the last string.

Usage:
    1) Start the game normally, load a save / navigate anywhere.
    2) D:/Python312/python.exe tools/frida_live_capture.py
    3) Navigate to the screen you want. Every screen (re)build arrives
       as a numbered burst: `burst 3: guio=617 area=65 tabs=2 nav=1`.
    4) Type a name (e.g. `club_squad`) + Enter to save the LAST burst to
       reports/screen_captures/live/<name>.json. Empty Enter re-prints
       the last burst summary. `q` quits.

Output schema matches tools/capture_all_screens.py (areas/objects/tabs/
nav/slots field names) so the downstream pipeline (gen_screen_rs.py ->
verify_screens.py) consumes it unchanged, with two live-only extras per
record: "text_str" (dereferenced text) and "caller" (return address,
for attributing sidebar/chrome noise to its builder).
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

# Field names per capture_all_screens.py.
AREA_FIELDS = ["L", "T", "R", "B", "cntA", "wA", "cntB", "wB", "flags", "a10", "parent"]
GUIO_FIELDS = ["type", "L", "T", "R", "B", "a6", "a7", "rflags",
               "colP", "colS", "tflags", "font", "tmode", "text",
               "a15", "a16", "a17", "parent"]
GUIO_TEXT_ARG = GUIO_FIELDS.index("text")

BURST_GAP = 0.4  # seconds of silence that ends a screen-build burst


def ref_bytes_at(vaddrs, n=8):
    """Read n bytes at each VA from the reference exe on disk."""
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
                off = rawp + (rva - va)
                out[v] = exe[off:off + n]
                break
        else:
            sys.exit(f"reference exe: 0x{v:08x} not in any section")
    return out


JS = r"""
'use strict';
const HOOKS = %HOOKS%;
const GUIO_TEXT_ARG = %TEXT_ARG%;

// Version guard: report live bytes at each hook address; Python compares
// against the reference exe and only then sends {kind:'arm'}.
const probe = {};
for (const k in HOOKS) {
    const addr = ptr(HOOKS[k][0]);
    probe[k] = addr.readByteArray(8);
}
send({ kind: 'probe' }, (function () {
    // concat the 4 byte-arrays in hook-name order for the payload
    const names = Object.keys(HOOKS).sort();
    let total = new Uint8Array(names.length * 8);
    names.forEach((k, i) => total.set(new Uint8Array(probe[k]), i * 8));
    return total.buffer;
})());

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
                    // Raw bytes, hex-encoded: the game's strings are
                    // latin-1 with markup bytes; terminator semantics are
                    // decided offline from the raw evidence, not guessed
                    // here (readCString mangled them as UTF-8).
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
                    // top_y/bot_y arrive as pointers (args 3,4) -- deref now,
                    // keep raw pointers too (p4/p5), matching the emulator.
                    try { a.push(args[3].isNull() ? null : args[3].readS32()); } catch (e) { a.push(null); }
                    try { a.push(args[4].isNull() ? null : args[4].readS32()); } catch (e) { a.push(null); }
                }
                send({ kind: 'call', fn: name, seq: seq++, args: a, text: text,
                       caller: this.returnAddress.toString() });
            }
        });
    }
    send({ kind: 'armed' });
});
"""


class Collector:
    def __init__(self, auto=False):
        self.lock = threading.Lock()
        self.current = []
        self.last_burst = None
        self.last_t = 0.0
        self.burst_no = 0
        self.auto = auto

    def add(self, msg):
        now = time.monotonic()
        with self.lock:
            if self.current and now - self.last_t > BURST_GAP:
                self._close()
            self.current.append(msg)
            self.last_t = now

    def _close(self):
        self.burst_no += 1
        self.last_burst = self.current
        n = {}
        for m in self.current:
            n[m["fn"]] = n.get(m["fn"], 0) + 1
        summary = " ".join(f"{k}={v}" for k, v in sorted(n.items()))
        if self.auto:
            name = f"burst_{self.burst_no:03d}"
            out = OUT_DIR / f"{name}.json"
            out.write_text(
                json.dumps(burst_to_json(name, self.last_burst), indent=1),
                encoding="utf-8")
            print(f"burst {self.burst_no}: {summary} -> saved {out.name}",
                  flush=True)
        else:
            print(f"\nburst {self.burst_no}: {summary}"
                  "   (type a name + Enter to save)", flush=True)
        self.current = []

    def flush_if_idle(self):
        with self.lock:
            if self.current and time.monotonic() - self.last_t > BURST_GAP:
                self._close()

    def take_last(self):
        with self.lock:
            self.flush_if_idle_locked()
            return self.last_burst

    def flush_if_idle_locked(self):
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
                b = bytes.fromhex(raw)
                cut = b.split(b"\x00", 1)[0]
                rec["text_str"] = cut.decode("latin-1", "replace")
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
            "source": "frida-live"}


def main():
    auto = "--auto" in sys.argv
    try:
        session = frida.attach(PROC)
    except frida.ProcessNotFoundError:
        sys.exit(f"{PROC} is not running -- start the game first.")

    js = JS.replace("%HOOKS%", json.dumps(
        {k: [v[0], v[1]] for k, v in HOOKS.items()})).replace(
        "%TEXT_ARG%", str(GUIO_TEXT_ARG))
    script = session.create_script(js)

    col = Collector(auto=auto)
    armed = threading.Event()
    expected = ref_bytes_at([v[0] for v in HOOKS.values()])

    def on_message(msg, data):
        if msg["type"] == "error":
            print("[js-error]", msg.get("stack", msg))
            return
        p = msg["payload"]
        if p["kind"] == "probe":
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
                print("Running exe does not match the decompiled reference "
                      "(D:/cm0102/cm0102.exe). NOT installing hooks.")
                script.unload(); session.detach(); sys.exit(1)
            script.post({"type": "arm"})
        elif p["kind"] == "armed":
            armed.set()
        elif p["kind"] == "call":
            col.add(p)

    script.on("message", on_message)
    script.load()
    if not armed.wait(timeout=10):
        sys.exit("hooks did not arm (no 'armed' message)")
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    print("Hooks armed and version-verified. Navigate the game; each screen "
          "build shows up as a burst"
          + (" and is saved automatically." if auto else "."), flush=True)

    idler = threading.Thread(target=lambda: _idle_loop(col), daemon=True)
    idler.start()

    if auto:
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            pass
        session.detach()
        print("detached -- game keeps running.")
        return

    while True:
        try:
            line = input("> ").strip()
        except (EOFError, KeyboardInterrupt):
            break
        if line == "q":
            break
        with col.lock:
            col.flush_if_idle_locked()
            burst = col.last_burst
        if not line:
            if burst is None:
                print("no burst captured yet")
            else:
                print(f"last burst: {len(burst)} calls")
            continue
        if burst is None:
            print("no burst to save yet -- navigate to a screen first")
            continue
        out = OUT_DIR / f"{line}.json"
        out.write_text(json.dumps(burst_to_json(line, burst), indent=1),
                       encoding="utf-8")
        print(f"saved {out.relative_to(REPO)} "
              f"({len(burst)} calls)")

    session.detach()
    print("detached -- game keeps running.")


def _idle_loop(col):
    while True:
        time.sleep(0.2)
        col.flush_if_idle()


if __name__ == "__main__":
    main()
