"""Frida driver for capturing FUN_0066f3b0 calls during construction of the
English Second Division schedule.

Attach to cm0102.exe (must already be running). Instrument:
  * 0x0055f040  eng_second ctor entry
  * 0x0055f340  eng_second schedule-getter entry / return
  * 0x0066f3b0  round-record writer  - capture args + dest before/after
  * 0x00668890  round-robin driver
  * 0x0066f280  schedule date walker
  * 0x00933ed6  malloc  - to grab the 2990-byte buffer address

The user drives the exe: Start New Game -> pick English leagues -> Next.
When save init runs the eng_second ctor, we log every call and dump the
completed buffer.

Output written to:
  D:\cm0102-rs\reports\fixture_disasm\runtime\<timestamp>_frida.jsonl
  D:\cm0102-rs\reports\fixture_disasm\runtime\<timestamp>_buffer.bin
"""
import frida, sys, json, time, os
from pathlib import Path

OUT_DIR = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")
OUT_DIR.mkdir(parents=True, exist_ok=True)

# Locate cm0102.exe
dev = frida.get_local_device()
pid = None
for p in dev.enumerate_processes():
    if p.name.lower() == "cm0102.exe":
        pid = p.pid; break
if pid is None:
    print("cm0102.exe not running - launch it first (via the GDI wrapper is fine)", file=sys.stderr)
    sys.exit(1)
print(f"attaching to cm0102.exe pid={pid}")
session = frida.attach(pid)

script_src = r"""
const IMAGE_BASE = 0x00400000;

// address helpers
function va(addr) { return ptr(addr); }

// small state
let sched_getter_active = false;
let sched_getter_call_idx = 0;
let last_malloc_bufs = [];  // last few malloc-returned pointers (with size)
let ctor_this = null;

// 1. hook eng_second ctor entry
Interceptor.attach(va(0x0055f040), {
    onEnter(args) {
        // __thiscall: this in ECX, first stack arg at [esp+4]
        // ECX is not in args; grab from this.context
        ctor_this = this.context.ecx.toString();
        send({op: "eng_second_ctor_enter", this_ptr: ctor_this,
              retaddr: this.returnAddress.toString(),
              ts: Date.now()});
    },
    onLeave(retval) {
        send({op: "eng_second_ctor_leave",
              retval: retval.toString(),
              ts: Date.now()});
        ctor_this = null;
    }
});

// 2. hook schedule-getter entry/leave
Interceptor.attach(va(0x0055f340), {
    onEnter(args) {
        // __thiscall: this in ECX; first stack arg (byte) at [esp+4]
        sched_getter_active = true;
        sched_getter_call_idx = 0;
        send({op: "schedgetter_enter",
              this_ptr: this.context.ecx.toString(),
              retaddr: this.returnAddress.toString(),
              arg1_byte: this.context.esp.add(4).readU8(),
              ts: Date.now()});
    },
    onLeave(retval) {
        // retval is the buffer pointer that becomes comp+0xba
        let buf = retval;
        send({op: "schedgetter_leave",
              retval: buf.toString(),
              ts: Date.now()});
        // Dump the completed 2990-byte buffer
        try {
            const raw = buf.readByteArray(0xbae);
            send({op: "schedbuffer_dump", buffer_ptr: buf.toString(),
                  size: 0xbae}, raw);
        } catch (e) {
            send({op: "schedbuffer_error", err: e.toString()});
        }
        sched_getter_active = false;
    }
});

// 3. hook malloc so we can associate returned pointers with sizes
Interceptor.attach(va(0x00933ed6), {
    onEnter(args) {
        this.size = this.context.esp.add(4).readU32();
    },
    onLeave(retval) {
        // Keep last 4 mallocs
        last_malloc_bufs.push({ptr: retval.toString(), size: this.size});
        if (last_malloc_bufs.length > 8) last_malloc_bufs.shift();
    }
});

// 4. hook round-record writer
Interceptor.attach(va(0x0066f3b0), {
    onEnter(args) {
        // 9 stack args (cdecl); read them
        const sp = this.context.esp;
        // arg1 = [esp+4] = buffer, arg2 = [esp+8] = round_idx (short), ...
        const buf = sp.add(4).readPointer();
        const round_idx = sp.add(8).readU16();
        const day = sp.add(0x0c).readS32();
        const month = sp.add(0x10).readS32();
        const day_off = sp.add(0x14).readS32();
        const flag = sp.add(0x18).readS32();
        const type_byte = sp.add(0x1c).readS32();
        const year = sp.add(0x20).readS32();
        const p9 = sp.add(0x24).readS32();

        // Snapshot the destination record before
        this.buf = buf; this.round_idx = round_idx;
        this.dest_off = round_idx * 0x41;
        let before = null;
        try {
            before = buf.add(this.dest_off).readByteArray(0x41);
        } catch (e) {}

        send({op: "roundwriter_enter",
              call_idx: sched_getter_call_idx,
              buf: buf.toString(),
              round_idx, day, month, day_off, flag, type_byte, year, p9,
              retaddr: this.returnAddress.toString(),
              sched_getter_active,
              ts: Date.now()},
             before);
    },
    onLeave(retval) {
        let after = null;
        try {
            after = this.buf.add(this.dest_off).readByteArray(0x41);
        } catch (e) {}
        send({op: "roundwriter_leave",
              call_idx: sched_getter_call_idx,
              buf: this.buf.toString(),
              round_idx: this.round_idx,
              dest_off: this.dest_off},
             after);
        sched_getter_call_idx += 1;
    }
});

// 5. hook round-robin driver
Interceptor.attach(va(0x00668890), {
    onEnter(args) {
        send({op: "round_robin_enter",
              this_ptr: this.context.ecx.toString(),
              retaddr: this.returnAddress.toString(),
              ts: Date.now()});
    },
    onLeave(retval) {
        send({op: "round_robin_leave", retval: retval.toString(),
              ts: Date.now()});
    }
});

// 6. hook walker
Interceptor.attach(va(0x0066f280), {
    onEnter(args) {
        const sp = this.context.esp;
        const p1 = sp.add(4).readS32();
        const p2_ptr = sp.add(8).readPointer();
        const p2_state = p2_ptr.readU8();
        send({op: "walker_enter",
              p1_slot: p1, p2_state,
              p3: sp.add(0x0c).readS32(),
              p4: sp.add(0x10).readS16(),
              p5: sp.add(0x14).readS16(),
              p6: sp.add(0x18).readS16(),
              p7: sp.add(0x1c).readU8()});
    },
    onLeave(retval) {
        send({op: "walker_leave", retval: retval.toInt32()});
    }
});

send({op: "hooks_installed"});
"""

script = session.create_script(script_src)

# Timestamped output files
ts = time.strftime("%Y%m%d_%H%M%S")
jsonl_path = OUT_DIR / f"{ts}_frida.jsonl"
buffer_path = OUT_DIR / f"{ts}_buffer.bin"
records = []
buffers = []

def on_message(msg, data):
    if msg["type"] == "send":
        payload = msg["payload"]
        payload["_ts_recv"] = time.time()
        if data is not None:
            payload["_data_hex"] = data.hex()
        records.append(payload)
        if payload.get("op") == "schedbuffer_dump" and data is not None:
            buffers.append((payload["buffer_ptr"], data))
            # save immediately
            b_out = OUT_DIR / f"{ts}_buffer_{len(buffers)}.bin"
            b_out.write_bytes(data)
            print(f"[+] wrote {b_out}")
        if payload.get("op") in ("schedgetter_enter", "schedgetter_leave",
                                  "round_robin_enter", "round_robin_leave",
                                  "eng_second_ctor_enter", "eng_second_ctor_leave"):
            print(f"[{payload['op']}] {payload}")
    elif msg["type"] == "error":
        print(f"[frida-error] {msg}", file=sys.stderr)

script.on("message", on_message)
script.load()

print(f"[+] hooks installed; run through Start New Game -> pick English leagues -> Next")
print(f"[+] logging to {jsonl_path}")

# Poll until we see a schedgetter_leave or the user Ctrl+Cs
last_flush = 0
try:
    while True:
        time.sleep(0.5)
        # Flush records periodically
        if len(records) != last_flush:
            with open(jsonl_path, "w") as f:
                for r in records:
                    f.write(json.dumps(r) + "\n")
            last_flush = len(records)
except KeyboardInterrupt:
    pass

print(f"[+] {len(records)} events recorded")
session.detach()
