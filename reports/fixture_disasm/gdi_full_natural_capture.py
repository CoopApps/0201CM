"""Full natural-execution capture:
  1. Kill any existing cm0102_GDI.exe
  2. Launch fresh
  3. Wait for the Setup Game menu
  4. Attach Frida with all fixture-pipeline hooks
  5. Drive through: Start New Game -> Select All -> Next -> CD OK
  6. Wait for eng_second_ctor to fire naturally
  7. Capture the full trace
"""
import frida, subprocess, sys, json, time, ctypes
from pathlib import Path
from ctypes import wintypes as wt

OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")
OUT.mkdir(parents=True, exist_ok=True)

user32 = ctypes.windll.user32

# Kill existing
subprocess.call(["taskkill", "/f", "/im", "cm0102_GDI.exe"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
time.sleep(1.5)

# Launch
subprocess.Popen([r"D:\cm0102\cm0102_GDI.exe"], cwd=r"D:\cm0102")
print("launched. waiting for main menu...")
# Wait until window appears
found_hwnd = None
EnumWindowsProc = ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)
def find_cm():
    hwnd_holder = [None]
    def cb(hwnd, _):
        length = user32.GetWindowTextLengthW(hwnd)
        if length > 0:
            buf = ctypes.create_unicode_buffer(length + 1)
            user32.GetWindowTextW(hwnd, buf, length + 1)
            if "Championship Manager" in buf.value:
                hwnd_holder[0] = hwnd
                return False
        return True
    user32.EnumWindows(EnumWindowsProc(cb), 0)
    return hwnd_holder[0]

t0 = time.time()
while time.time() - t0 < 30 and not found_hwnd:
    time.sleep(0.5)
    found_hwnd = find_cm()

if not found_hwnd:
    sys.exit("CM window did not appear")
print(f"HWND: 0x{found_hwnd:x}")
# Let the exe fully initialise
time.sleep(3)

# Foreground
SW_RESTORE = 9
user32.ShowWindow(found_hwnd, SW_RESTORE)
time.sleep(0.5)
user32.SetForegroundWindow(found_hwnd)
time.sleep(0.5)
class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]
origin = POINT(0, 0)
user32.ClientToScreen(found_hwnd, ctypes.byref(origin))
print(f"client origin: ({origin.x}, {origin.y})")

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

// pool RNG hook
Interceptor.attach(ptr(0x008fbe20), {
    onEnter() {
        this.n = this.context.esp.add(4).readS32();
    },
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
            phase = "ENG_SECOND_CTOR";
            this.target = t;
            send({op:"eng_second_ctor_enter", this_ptr: t.toString(),
                  year: t.add(0x40).readU16()});
        }
    },
    onLeave() {
        if (phase === "ENG_SECOND_CTOR" || phase === "ENG_SECOND_AFTER_DRIVER") {
            try {
                const nc = this.target.add(0x3e).readS16();
                const nr = this.target.add(0xa9).readS16();
                const clubs_ptr = this.target.add(0xb1).readPointer();
                const sched_ptr = this.target.add(0xba).readPointer();
                const alt = this.target.add(0xea).readPointer();
                send({op:"eng_second_ctor_leave",
                      n_clubs: nc, n_rounds: nr,
                      clubs_ptr: clubs_ptr.toString(),
                      sched_ptr: sched_ptr.toString(),
                      alt_pair_list: alt.toString(),
                      walker_calls: walker_calls.length,
                      rng_calls: rng_calls.length,
                      fixture_inserts: fixture_inserts.length});
                // Dump sched + clubs
                if (!sched_ptr.isNull()) {
                    const raw = sched_ptr.readByteArray(0xbae);
                    send({op:"sched_buffer"}, raw);
                }
                if (!clubs_ptr.isNull() && nc > 0 && nc <= 32) {
                    const raw = clubs_ptr.readByteArray(nc * 0x3b);
                    send({op:"clubs_table"}, raw);
                    // Dereference each entry's Club* pointer
                    const entries = [];
                    for (let i = 0; i < nc; i++) {
                        try {
                            const cptr = clubs_ptr.add(i * 0x3b).readPointer();
                            const cid = cptr.readS32();
                            const nation69 = cptr.add(0x69).readS32();
                            entries.push({slot: i, club_id: cid,
                                          nation_field: nation69,
                                          club_ptr: cptr.toString()});
                        } catch(e) {
                            entries.push({slot: i, error: e.toString()});
                        }
                    }
                    send({op:"clubs_dereferenced", entries});
                }
                // Send arrays
                send({op:"walker_trace", n: walker_calls.length,
                      first: walker_calls.slice(0, 5),
                      last: walker_calls.slice(-5)});
                send({op:"rng_trace_summary", n: rng_calls.length,
                      first: rng_calls.slice(0, 10),
                      last: rng_calls.slice(-5)});
                send({op:"fixture_trace", n: fixture_inserts.length,
                      all: fixture_inserts.slice(0, 60)});
            } catch(e) { send({op:"ctor_leave_err", err: e.toString()}); }
            phase = "IDLE";
        }
    }
});

Interceptor.attach(ptr(0x00668450), {
    onEnter() {
        if (phase === "ENG_SECOND_CTOR") {
            phase = "ENG_SECOND_DRIVER";
            send({op:"driver_enter", walker_calls_before: walker_calls.length,
                  rng_calls_before: rng_calls.length});
        }
    },
    onLeave(rv) {
        if (phase === "ENG_SECOND_DRIVER") {
            phase = "ENG_SECOND_AFTER_DRIVER";
            send({op:"driver_leave", retval: rv.toInt32(),
                  walker_calls_after: walker_calls.length,
                  rng_calls_after: rng_calls.length,
                  fixture_inserts: fixture_inserts.length});
        }
    }
});

Interceptor.attach(ptr(0x0066b900), {
    onEnter() {
        if (phase === "ENG_SECOND_DRIVER") {
            phase = "ENG_SECOND_PERTURB";
            const t = this.context.ecx;
            send({op:"perturb_enter",
                  rng_calls_before: rng_calls.length});
            // Snapshot clubs before
            try {
                const nc = t.add(0x3e).readS16();
                const clubs_ptr = t.add(0xb1).readPointer();
                if (!clubs_ptr.isNull() && nc > 0) {
                    const raw = clubs_ptr.readByteArray(nc * 0x3b);
                    send({op:"clubs_before_perturb"}, raw);
                }
            } catch(e) {}
        }
    },
    onLeave() {
        if (phase === "ENG_SECOND_PERTURB") {
            phase = "ENG_SECOND_DRIVER";
            const t = this.context.ecx;
            send({op:"perturb_leave",
                  rng_calls_after: rng_calls.length});
            try {
                const nc = t.add(0x3e).readS16();
                const clubs_ptr = t.add(0xb1).readPointer();
                if (!clubs_ptr.isNull() && nc > 0) {
                    const raw = clubs_ptr.readByteArray(nc * 0x3b);
                    send({op:"clubs_after_perturb"}, raw);
                }
            } catch(e) {}
        }
    }
});

Interceptor.attach(ptr(0x0066ee40), {
    onEnter(args) {
        if (phase === "ENG_SECOND_DRIVER") {
            const sp = this.context.esp;
            this.wi = {
                prev_col: sp.add(4).readS32(),
                state_ptr: sp.add(8).readPointer().toString(),
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
        if (phase === "ENG_SECOND_DRIVER") {
            const fp = this.context.esp.add(4).readPointer();
            try {
                const rec = fp.readByteArray(0x4f);
                // Extract fields via native accessors
                const cid = fp.readS32();
                const home = fp.add(0x0c).readS32();
                const away = fp.add(0x10).readS32();
                const year = fp.add(0x28).readS16();
                const doy = fp.add(0x2a).readS16();
                const rwh = fp.add(0x34).readS16();
                fixture_inserts.push({idx: fixture_inserts.length,
                                      cid, home, away, year, doy, rwh,
                                      fp: fp.toString()});
            } catch(e) {}
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
        if op not in ("clubs_table", "sched_buffer", "clubs_before_perturb",
                      "clubs_after_perturb"):
            print(f"[{op}] {p if len(str(p)) < 200 else str(p)[:200]}")
        if d:
            buffers.setdefault(op, []).append(d)
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()
time.sleep(0.5)
print("hooks armed. driving UI...")

def click(cx, cy, tag, delay=0.8):
    sx = origin.x + cx; sy = origin.y + cy
    print(f"click ({cx},{cy}) {tag}")
    user32.SetCursorPos(sx, sy); time.sleep(0.15)
    user32.mouse_event(0x02, 0, 0, 0, 0); time.sleep(0.05)
    user32.mouse_event(0x04, 0, 0, 0, 0); time.sleep(delay)

click(280, 205, "Start New Game", 1.5)
click(595, 181, "Select All", 1.5)
click(710, 596, "Next", 1.5)
# CD Not Required dialog OK - centered dialog around client (405, 355)
click(405, 355, "CD dialog OK", 1.0)

print("waiting for eng_second_ctor...")
t0 = time.time()
saw_ctor = False
last_activity = time.time()
while time.time() - t0 < 300:
    time.sleep(0.5)
    if any(r.get("op") == "eng_second_ctor_leave" for r in records):
        saw_ctor = True
        print(f"eng_second_ctor complete! elapsed={time.time()-t0:.1f}s")
        time.sleep(2)
        break

ts = time.strftime("%Y%m%d_%H%M%S")
(OUT / f"{ts}_full_natural.jsonl").write_text(
    "\n".join(json.dumps(r) for r in records), encoding="utf-8")
for kind, bufs in buffers.items():
    for i, b in enumerate(bufs):
        (OUT / f"{ts}_full_{kind}_{i}.bin").write_bytes(b)

print(f"\n{len(records)} events total")
print(f"  eng_second_ctor: {sum(1 for r in records if r.get('op')=='eng_second_ctor_leave')}")
print(f"  driver_enter: {sum(1 for r in records if r.get('op')=='driver_enter')}")
print(f"  perturb_enter: {sum(1 for r in records if r.get('op')=='perturb_enter')}")
print(f"  buffers saved: {list(buffers.keys())}")

session.detach()
