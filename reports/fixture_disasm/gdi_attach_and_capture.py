"""Attach Frida NOW with all fixture-pipeline hooks and click OK on
the CD dialog. Capture whatever ctor/driver events fire.
"""
import frida, sys, json, time, ctypes
from pathlib import Path
from ctypes import wintypes as wt

OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")
OUT.mkdir(parents=True, exist_ok=True)

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
user32.ShowWindow(found_hwnd, SW_RESTORE)
time.sleep(0.5)
user32.SetForegroundWindow(found_hwnd)
time.sleep(0.5)

class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]
origin = POINT(0, 0)
user32.ClientToScreen(found_hwnd, ctypes.byref(origin))

# Attach Frida
dev = frida.get_local_device()
pid = next((p.pid for p in dev.enumerate_processes() if p.name.lower()=="cm0102_gdi.exe"), None)
if not pid: sys.exit("cm0102_GDI.exe not running")
session = frida.attach(pid)

script_src = r"""
Interceptor.attach(ptr(0x0055f240), {
    onEnter() {
        const t = this.context.ecx;
        send({op:"ctor_enter", this_ptr: t.toString(),
              comp_id: t.add(0x50).readU8(),
              year: t.add(0x40).readU16(),
              n_clubs_field: t.add(0x3e).readS16(),
              schedule_ptr_before: t.add(0xba).readPointer().toString()});
        this.target = t;
    },
    onLeave() {
        try {
            send({op:"ctor_leave",
                  schedule_ptr_after: this.target.add(0xba).readPointer().toString(),
                  clubs_ptr: this.target.add(0xb1).readPointer().toString(),
                  n_clubs_after: this.target.add(0x3e).readS16(),
                  n_rounds: this.target.add(0xa9).readS16(),
                  alt_pair_list: this.target.add(0xea).readPointer().toString()});
            // Dump the schedule buffer
            const sched = this.target.add(0xba).readPointer();
            if (!sched.isNull()) {
                const raw = sched.readByteArray(0xbae);
                send({op:"sched_buffer"}, raw);
            }
            // Dump the clubs table
            const clubs_ptr = this.target.add(0xb1).readPointer();
            const nc = this.target.add(0x3e).readS16();
            if (!clubs_ptr.isNull() && nc > 0 && nc <= 32) {
                const raw = clubs_ptr.readByteArray(nc * 0x3b);
                send({op:"clubs_table"}, raw);
                // For each entry, dereference the Club* and dump its
                // first int (id) + +0x69 nation reference.
                const nation_of = [];
                for (let i = 0; i < nc; i++) {
                    try {
                        const entry_off = i * 0x3b;
                        const club_ptr = clubs_ptr.add(entry_off).readPointer();
                        const club_id = club_ptr.readS32();
                        const nation_field = club_ptr.add(0x69).readS32();
                        nation_of.push({slot: i, club_id, nation_field,
                                        club_ptr: club_ptr.toString()});
                    } catch(e) {
                        nation_of.push({slot: i, error: e.toString()});
                    }
                }
                send({op:"clubs_dereferenced", entries: nation_of});
            }
        } catch(e) { send({op:"ctor_leave_err", err:e.toString()}); }
    }
});
Interceptor.attach(ptr(0x00668450), {
    onEnter() {
        const t = this.context.ecx;
        send({op:"driver_enter", this_ptr: t.toString(),
              comp_id: t.add(0x50).readU8(),
              n_clubs: t.add(0x3e).readS16(),
              alt_pair_list: t.add(0xea).readPointer().toString()});
    },
    onLeave(rv) { send({op:"driver_leave", retval: rv.toInt32()}); }
});
Interceptor.attach(ptr(0x0066b900), {
    onEnter() {
        send({op:"perturb_enter"});
    },
    onLeave() { send({op:"perturb_leave"}); }
});
send({op:"hooks_installed"});
"""

records = []
buffers = {}
last_activity = time.time()
def on_message(m, d):
    global last_activity
    if m["type"] == "send":
        p = m["payload"]
        records.append(p)
        last_activity = time.time()
        op = p.get("op")
        if op != "clubs_table" and op != "sched_buffer" and op != "clubs_dereferenced":
            print(f"[{op}] {p}")
        elif op == "clubs_dereferenced":
            print(f"[clubs_dereferenced] {len(p.get('entries', []))} entries")
        if d:
            buffers.setdefault(op, []).append(d)
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()
time.sleep(0.5)
print("hooks installed")

# Click OK on the CD dialog. Dialog buttons often at center-bottom.
# Looking at screenshot: OK button at approximately client (405, 355)
def click(cx, cy, tag=""):
    sx = origin.x + cx
    sy = origin.y + cy
    print(f"click ({cx},{cy}) {tag}")
    user32.SetCursorPos(sx, sy); time.sleep(0.2)
    user32.mouse_event(0x02, 0, 0, 0, 0); time.sleep(0.05)
    user32.mouse_event(0x04, 0, 0, 0, 0); time.sleep(0.8)

# CD dialog OK button
click(405, 355, "CD dialog OK")

# Wait for ctor or timeout (120s max)
print("waiting for eng_second_ctor to fire...")
t0 = time.time()
while time.time() - t0 < 120:
    time.sleep(0.5)
    # Check for eng_second (comp_id == 9) ctor_enter events
    eng2 = [r for r in records
            if r.get("op") == "ctor_enter" and r.get("comp_id") == 9]
    if eng2:
        print(f"eng_second_ctor fired! elapsed={time.time()-t0:.1f}s")
        # Wait a bit more for the driver + leave events
        time.sleep(5)
        break
    # If no activity for 15s and we've been waiting, timeout early
    if time.time() - last_activity > 15 and time.time() - t0 > 10:
        print(f"no activity for 15s - likely ctor already ran before hooks attached")
        break

ts = time.strftime("%Y%m%d_%H%M%S")
(OUT / f"{ts}_natural2.jsonl").write_text(
    "\n".join(json.dumps(r) for r in records), encoding="utf-8")
for kind, bufs in buffers.items():
    for i, b in enumerate(bufs):
        (OUT / f"{ts}_natural2_{kind}_{i}.bin").write_bytes(b)

print(f"\n{len(records)} events total")
print(f"  ctor_enter: {sum(1 for r in records if r.get('op')=='ctor_enter')}")
print(f"  driver_enter: {sum(1 for r in records if r.get('op')=='driver_enter')}")
print(f"  perturb_enter: {sum(1 for r in records if r.get('op')=='perturb_enter')}")

# Print comp_id distribution of ctor entries
from collections import Counter
c = Counter(r.get("comp_id") for r in records if r.get("op") == "ctor_enter")
print(f"  comp_ids seen: {dict(c)}")

session.detach()
