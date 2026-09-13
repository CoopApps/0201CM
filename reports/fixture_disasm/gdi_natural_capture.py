"""Comprehensive natural-execution capture harness for cm0102_GDI.exe.

Steps:
  1. Attach Frida to cm0102_GDI.exe with hooks on the full fixture-
     pipeline (ctor, schedule getter, perturbation, walker, driver,
     RNG, fixture insertion).
  2. Restore the window if minimized, foreground it.
  3. Click Start New Game.
  4. On the League Selection screen, click Next.
  5. Wait for the ctor to fire naturally.
  6. Capture the trace and save to disk.

Hook targets (GDI VAs):
  0x0055f240  eng_second_ctor
  0x0055f540  schedule getter
  0x0066b900  matrix perturbation
  0x00668450  round-robin driver
  0x0066ee40  walker
  0x008fbe20  pool RNG (FUN_008fc4f0)
  0x00594eb0  TFixList insert
  0x00844dc0  venue writer
  0x0066a4d0  replay/reset
  0x0066f660  success finalizer
"""
import frida, sys, json, time, ctypes
from pathlib import Path
from ctypes import wintypes as wt

OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")
OUT.mkdir(parents=True, exist_ok=True)

# --- Locate + restore the CM window ---------------------------------------
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
if not found_hwnd:
    sys.exit("no CM window")
print(f"HWND: 0x{found_hwnd:x}")

SW_RESTORE = 9
user32.ShowWindow(found_hwnd, SW_RESTORE)
time.sleep(0.4)
user32.SetForegroundWindow(found_hwnd)
time.sleep(0.3)

# Get client-to-screen offset
class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]
origin = POINT(0, 0)
user32.ClientToScreen(found_hwnd, ctypes.byref(origin))
print(f"client origin: ({origin.x}, {origin.y})")

# --- Attach Frida ---------------------------------------------------------
dev = frida.get_local_device()
pid = next((p.pid for p in dev.enumerate_processes() if p.name.lower()=="cm0102_gdi.exe"), None)
if not pid: sys.exit("cm0102_GDI.exe not running")
print(f"attaching to pid {pid}")
session = frida.attach(pid)

script_src = r"""
const ADDRS = {
    CTOR:       0x0055f240,
    SCHEDGETTER:0x0055f540,
    PERTURB:    0x0066b900,
    DRIVER:     0x00668450,
    WALKER:     0x0066ee40,
    POOL_RNG:   0x008fbe20,
    FIX_INSERT: 0x00594eb0,
    VENUE:      0x00844dc0,
    REPLAY:     0x0066a4d0,
    FINALIZE:   0x0066f660,
};

let target_comp = null;   // the eng_second comp we're tracking
let phase = "IDLE";
let event_count = 0;
let walker_calls = [];
let rng_calls = [];
let fixture_inserts = [];
let clubs_before_perturb = null;
let clubs_after_perturb = null;
let rng_state_before = null;
let rng_state_after_perturb = null;

// Track pool RNG state via a fake global (we don't have the exe's
// DAT_00dc7238 pointer resolved). Instead just count calls with
// (arg, retval).
Interceptor.attach(ptr(ADDRS.POOL_RNG), {
    onEnter(args) {
        this.n = this.context.esp.add(4).readS32();
    },
    onLeave(rv) {
        rng_calls.push({phase, n: this.n, retval: rv.toInt32(),
                        callsite: this.returnAddress.toString()});
    }
});

Interceptor.attach(ptr(ADDRS.CTOR), {
    onEnter() {
        target_comp = this.context.ecx;
        send({op:"ctor_enter",
              this_ptr: target_comp.toString(),
              comp_id_byte: target_comp.add(0x50).readU8(),
              year: target_comp.add(0x40).readU16()});
        phase = "CTOR";
    },
    onLeave() {
        send({op:"ctor_leave", event_count});
        phase = "IDLE";
    }
});

Interceptor.attach(ptr(ADDRS.SCHEDGETTER), {
    onEnter() { phase = "SCHEDGETTER"; send({op:"sched_getter_enter"}); },
    onLeave(rv) {
        send({op:"sched_getter_leave", retval: rv.toString()});
        // Dump the returned 2990-byte buffer.
        try {
            const raw = rv.readByteArray(0xbae);
            send({op:"sched_buffer", ptr: rv.toString()}, raw);
        } catch(e) {}
    }
});

Interceptor.attach(ptr(ADDRS.PERTURB), {
    onEnter() {
        phase = "PERTURB";
        // param_1 = this (comp), param_2 = n_even
        const this_ptr = this.context.ecx;
        const n_even = this.context.esp.add(4).readS32();
        const clubs_table_ptr = this_ptr.add(0xb1).readPointer();
        // Read 24 × 0x3b bytes
        let table_before = null;
        try {
            table_before = clubs_table_ptr.readByteArray(24 * 0x3b);
        } catch(e) {}
        clubs_before_perturb = clubs_table_ptr;
        this.clubs_ptr = clubs_table_ptr;
        this.n_even = n_even;
        send({op:"perturb_enter", this_ptr: this_ptr.toString(),
              n_even, clubs_table: clubs_table_ptr.toString()}, table_before);
    },
    onLeave() {
        phase = "IDLE";
        try {
            const table_after = this.clubs_ptr.readByteArray(this.n_even * 0x3b);
            send({op:"perturb_leave", n_even: this.n_even,
                  clubs_table: this.clubs_ptr.toString()}, table_after);
        } catch(e) {}
    }
});

Interceptor.attach(ptr(ADDRS.WALKER), {
    onEnter(args) {
        const sp = this.context.esp;
        this.walker_input = {
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
    },
    onLeave(rv) {
        const walker_out = {
            ...this.walker_input,
            state_after: this.state_ptr.readU8(),
            retval: rv.toInt32(),
            call_idx: walker_calls.length,
        };
        walker_calls.push(walker_out);
    }
});

Interceptor.attach(ptr(ADDRS.DRIVER), {
    onEnter() {
        phase = "DRIVER";
        const this_ptr = this.context.ecx;
        send({op:"driver_enter",
              this_ptr: this_ptr.toString(),
              n_clubs: this_ptr.add(0x3e).readS16(),
              matches_per_pair: this_ptr.add(0x3c).readS16(),
              n_rounds: this_ptr.add(0xa9).readS16(),
              year_base: this_ptr.add(0x40).readS16(),
              host_nation: this_ptr.add(0x24).readS32(),
              walker_flag: this_ptr.add(0xd9).readU8(),
              alt_pair_list: this_ptr.add(0xea).readPointer().toString(),
              rng_calls_so_far: rng_calls.length});
    },
    onLeave(rv) {
        send({op:"driver_leave",
              retval: rv.toInt32(),
              walker_calls: walker_calls.length,
              rng_calls: rng_calls.length,
              fixture_inserts: fixture_inserts.length});
    }
});

Interceptor.attach(ptr(ADDRS.FIX_INSERT), {
    onEnter() {
        if (phase !== "DRIVER") return;
        const fixture_ptr = this.context.esp.add(4).readPointer();
        try {
            const rec = fixture_ptr.readByteArray(0x4f);
            const arr = new Uint8Array(rec);
            const comp_id = new Int32Array(rec.slice(0,4))[0];
            const home_id = new Int32Array(rec.slice(0x0c, 0x10))[0];
            const away_id = new Int32Array(rec.slice(0x10, 0x14))[0];
            const year = new Int16Array(rec.slice(0x28, 0x2a))[0];
            const doy = new Int16Array(rec.slice(0x2a, 0x2c))[0];
            const rwh = new Int16Array(rec.slice(0x34, 0x36))[0];
            fixture_inserts.push({
                idx: fixture_inserts.length,
                comp_id, home_id, away_id, year, doy, round_wh: rwh,
                fixture_ptr: fixture_ptr.toString(),
            });
        } catch(e) {}
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
        if op in ("hooks_installed", "ctor_enter", "ctor_leave",
                  "sched_getter_enter", "sched_getter_leave",
                  "perturb_enter", "perturb_leave",
                  "driver_enter", "driver_leave"):
            print(f"[{op}] {p}")
        if op == "sched_buffer" and d:
            buffers["sched"] = d
        if op == "perturb_enter" and d:
            buffers["clubs_before"] = d
        if op == "perturb_leave" and d:
            buffers["clubs_after"] = d
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()
time.sleep(0.5)
print("hooks armed. driving UI...")

# --- Automate the UI clicks ---------------------------------------------
def click_client(cx, cy, tag):
    sx = origin.x + cx
    sy = origin.y + cy
    print(f"  [{tag}] click screen ({sx},{sy}) client ({cx},{cy})")
    user32.SetForegroundWindow(found_hwnd)
    time.sleep(0.15)
    user32.SetCursorPos(sx, sy)
    time.sleep(0.15)
    # MOUSEEVENTF_LEFTDOWN = 0x2, LEFTUP = 0x4
    user32.mouse_event(0x02, 0, 0, 0, 0)
    time.sleep(0.05)
    user32.mouse_event(0x04, 0, 0, 0, 0)
    time.sleep(0.6)

# Step 1: Start New Game — client (280, 205)
click_client(280, 205, "Start New Game")

# Step 2: On the League Selection screen (usually Next) — client (710, 596)
# Actually: Start New Game leads to a manager-name / league-select screen.
# We first need to click Next multiple times, letting each screen fire.
for i in range(6):
    click_client(710, 596, f"Next #{i+1}")

# Wait for ctor to fire
print("waiting for ctor_enter...")
t0 = time.time()
saw_ctor = False
while time.time() - t0 < 60:
    time.sleep(0.5)
    if any(r.get("op") == "ctor_enter" for r in records):
        saw_ctor = True
        print("ctor fired!")
        # give it a bit more to complete driver + inserts
        time.sleep(3)
        break
if not saw_ctor:
    print("TIMEOUT: ctor did not fire within 60s")

# Save trace
ts = time.strftime("%Y%m%d_%H%M%S")
(OUT / f"{ts}_natural_capture.jsonl").write_text(
    "\n".join(json.dumps(r) for r in records), encoding="utf-8")
for name, data in buffers.items():
    (OUT / f"{ts}_natural_{name}.bin").write_bytes(data)
print(f"\ntrace saved to {ts}_natural_*")
print(f"  events: {len(records)}")
print(f"  buffers: {list(buffers.keys())}")

# Fetch walker + rng + fixture arrays from the script state
try:
    walker_arr = script.exports.get_walker_calls()
except Exception:
    walker_arr = None
# We'll just report record-based counts
print(f"  ctor_enter events: {sum(1 for r in records if r.get('op')=='ctor_enter')}")
print(f"  driver_enter events: {sum(1 for r in records if r.get('op')=='driver_enter')}")

session.detach()
