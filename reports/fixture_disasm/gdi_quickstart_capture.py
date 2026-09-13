"""Continue from the Quick Start Game > Select File screen.
Load the ENG quick-start file with Frida already attached and
comprehensive hooks in place.
"""
import frida, sys, json, time, ctypes
from pathlib import Path
from ctypes import wintypes as wt

OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")

user32 = ctypes.windll.user32
EnumWindowsProc = ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)
found_hwnd = None
def enum_cb(hwnd, _):
    global found_hwnd
    length = user32.GetWindowTextLengthW(hwnd)
    if length > 0:
        buf = ctypes.create_unicode_buffer(length + 1)
        user32.GetWindowTextW(hwnd, buf, length + 1)
        if "Championship Manager" in buf.value:
            found_hwnd = hwnd
            return False
    return True
user32.EnumWindows(EnumWindowsProc(enum_cb), 0)
SW_RESTORE = 9
user32.ShowWindow(found_hwnd, SW_RESTORE); time.sleep(0.5)
user32.SetForegroundWindow(found_hwnd); time.sleep(0.5)
class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]
origin = POINT(0, 0)
user32.ClientToScreen(found_hwnd, ctypes.byref(origin))

# Attach Frida
dev = frida.get_local_device()
pid = next((p.pid for p in dev.enumerate_processes() if p.name.lower()=="cm0102_gdi.exe"), None)
if not pid: sys.exit("cm0102_GDI.exe not running")
print(f"attaching pid {pid}")
session = frida.attach(pid)

script_src = r"""
let walker_calls = [];
let rng_calls = [];
let fixture_inserts = [];
let phase = "IDLE";

Interceptor.attach(ptr(0x008fbe20), {
    onEnter() { this.n = this.context.esp.add(4).readS32(); },
    onLeave(rv) {
        if (phase !== "IDLE") {
            rng_calls.push({phase, n: this.n, retval: rv.toInt32()});
        }
    }
});
Interceptor.attach(ptr(0x0055f240), {
    onEnter() {
        const t = this.context.ecx;
        const cid = t.add(0x50).readU8();
        if (cid === 9) {
            phase = "CTOR";
            this.target = t;
            send({op:"ctor_enter", this_ptr: t.toString(),
                  year: t.add(0x40).readU16()});
        }
    },
    onLeave() {
        if (!this.target) return;
        try {
            const t = this.target;
            const nc = t.add(0x3e).readS16();
            const nr = t.add(0xa9).readS16();
            const clubs_ptr = t.add(0xb1).readPointer();
            const sched_ptr = t.add(0xba).readPointer();
            const alt = t.add(0xea).readPointer();
            send({op:"ctor_leave",
                  n_clubs: nc, n_rounds: nr,
                  clubs_ptr: clubs_ptr.toString(),
                  sched_ptr: sched_ptr.toString(),
                  alt_pair_list: alt.toString(),
                  walker_calls: walker_calls.length,
                  rng_calls: rng_calls.length,
                  fixture_inserts: fixture_inserts.length});
            if (!sched_ptr.isNull()) send({op:"sched_buffer"}, sched_ptr.readByteArray(0xbae));
            if (!clubs_ptr.isNull() && nc > 0 && nc <= 32) {
                send({op:"clubs_table"}, clubs_ptr.readByteArray(nc * 0x3b));
                const entries = [];
                for (let i = 0; i < nc; i++) {
                    try {
                        const cptr = clubs_ptr.add(i * 0x3b).readPointer();
                        entries.push({slot:i, club_id:cptr.readS32(),
                                      nation_field:cptr.add(0x69).readS32(),
                                      club_ptr:cptr.toString()});
                    } catch(e) { entries.push({slot:i, error:e.toString()}); }
                }
                send({op:"clubs_dereferenced", entries});
            }
            send({op:"walker_full_trace", calls: walker_calls});
            send({op:"rng_full_trace", calls: rng_calls});
            send({op:"fixture_full_trace", inserts: fixture_inserts});
        } catch(e) { send({op:"ctor_leave_err", err:e.toString()}); }
        phase = "IDLE";
    }
});
Interceptor.attach(ptr(0x00668450), {
    onEnter() {
        if (phase === "CTOR") {
            phase = "DRIVER";
            send({op:"driver_enter",
                  walker_before: walker_calls.length,
                  rng_before: rng_calls.length});
        }
    },
    onLeave(rv) {
        if (phase === "DRIVER") {
            phase = "CTOR";
            send({op:"driver_leave", retval: rv.toInt32(),
                  walker_after: walker_calls.length,
                  rng_after: rng_calls.length,
                  fixtures: fixture_inserts.length});
        }
    }
});
Interceptor.attach(ptr(0x0066b900), {
    onEnter() {
        if (phase === "DRIVER") {
            phase = "PERTURB";
            const t = this.context.ecx;
            send({op:"perturb_enter", rng_before: rng_calls.length});
            try {
                const nc = t.add(0x3e).readS16();
                const clubs = t.add(0xb1).readPointer();
                if (!clubs.isNull() && nc > 0) {
                    send({op:"clubs_before_perturb"}, clubs.readByteArray(nc * 0x3b));
                }
            } catch(e){}
        }
    },
    onLeave() {
        if (phase === "PERTURB") {
            phase = "DRIVER";
            const t = this.context.ecx;
            send({op:"perturb_leave", rng_after: rng_calls.length});
            try {
                const nc = t.add(0x3e).readS16();
                const clubs = t.add(0xb1).readPointer();
                if (!clubs.isNull() && nc > 0) {
                    send({op:"clubs_after_perturb"}, clubs.readByteArray(nc * 0x3b));
                }
            } catch(e){}
        }
    }
});
Interceptor.attach(ptr(0x0066ee40), {
    onEnter(args) {
        if (phase === "DRIVER") {
            const sp = this.context.esp;
            this.wi = {
                prev_col: sp.add(4).readS32(),
                state_before: sp.add(8).readPointer().readU8(),
                comp_id: sp.add(0x0c).readS32(),
                n_clubs: sp.add(0x10).readS16(),
                matches_per_pair: sp.add(0x14).readS16(),
                n_rounds: sp.add(0x18).readS16(),
                flag_byte: sp.add(0x1c).readU8(),
            };
            this.state_ptr = sp.add(8).readPointer();
        }
    },
    onLeave(rv) {
        if (this.wi) {
            walker_calls.push({...this.wi,
                               state_after: this.state_ptr.readU8(),
                               retval: rv.toInt32(),
                               idx: walker_calls.length});
        }
    }
});
Interceptor.attach(ptr(0x00594eb0), {
    onEnter() {
        if (phase === "DRIVER") {
            const fp = this.context.esp.add(4).readPointer();
            try {
                fixture_inserts.push({
                    idx: fixture_inserts.length,
                    cid: fp.readS32(),
                    home: fp.add(0x0c).readS32(),
                    away: fp.add(0x10).readS32(),
                    year: fp.add(0x28).readS16(),
                    doy: fp.add(0x2a).readS16(),
                    rwh: fp.add(0x34).readS16(),
                });
            } catch(e){}
        }
    }
});
send({op:"hooks_installed"});
"""

records = []
buffers = {}
def on_message(m, d):
    if m["type"] == "send":
        p = m["payload"]
        records.append(p)
        op = p.get("op")
        if op in ("hooks_installed","ctor_enter","ctor_leave","driver_enter",
                  "driver_leave","perturb_enter","perturb_leave"):
            print(f"[{op}] {p}")
        if d:
            buffers.setdefault(op, []).append(d)
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()
time.sleep(0.5)
print("hooks installed. clicking ENG then Ok...")

def click(cx, cy, tag, delay=0.8):
    sx = origin.x + cx; sy = origin.y + cy
    print(f"click ({cx},{cy}) {tag}")
    user32.SetCursorPos(sx, sy); time.sleep(0.15)
    user32.mouse_event(0x02, 0, 0, 0, 0); time.sleep(0.05)
    user32.mouse_event(0x04, 0, 0, 0, 0); time.sleep(delay)

# ENG box in Quick Start list — client ~(445, 282)
click(445, 282, "ENG", 1.0)
# Ok button — client ~(707, 596)
click(707, 596, "Ok", 1.5)

print("waiting for eng_second_ctor to fire...")
t0 = time.time()
while time.time() - t0 < 300:
    time.sleep(0.5)
    if any(r.get("op") == "ctor_leave" for r in records):
        print(f"ctor complete! elapsed={time.time()-t0:.1f}s")
        time.sleep(3)
        break

ts = time.strftime("%Y%m%d_%H%M%S")
(OUT / f"{ts}_quickstart.jsonl").write_text(
    "\n".join(json.dumps(r) for r in records), encoding="utf-8")
for kind, bufs in buffers.items():
    for i, b in enumerate(bufs):
        (OUT / f"{ts}_qs_{kind}_{i}.bin").write_bytes(b)

print(f"\n{len(records)} events")
print(f"  ctor: {sum(1 for r in records if r.get('op')=='ctor_leave')}")
print(f"  driver_leave: {sum(1 for r in records if r.get('op')=='driver_leave')}")
print(f"  perturb_leave: {sum(1 for r in records if r.get('op')=='perturb_leave')}")
print(f"  buffers: {list(buffers.keys())}")

session.detach()
