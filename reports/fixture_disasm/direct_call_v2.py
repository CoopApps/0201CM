"""Direct-call FUN_0055f340 again, now with hooks on:
- FUN_00533b50 (pack_date)  entry/leave with buffer contents
- FUN_00533eb0 (flag-snap)  entry with (flag, date-buffer-before) and leave with buffer-after
- FUN_00668890 (round-robin driver) entry/leave
- FUN_0066f280 (walker) entry/leave with args
- FUN_0066f3b0 (writer) as before
"""
import frida, sys, json, time
from pathlib import Path

OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime"); OUT.mkdir(parents=True, exist_ok=True)

dev = frida.get_local_device()
pid = next((p.pid for p in dev.enumerate_processes() if p.name.lower()=="cm0102.exe"), None)
if not pid: sys.exit("cm0102.exe not running")
print(f"attaching pid={pid}")
session = frida.attach(pid)

script_src = r"""
const SCHEDGETTER   = 0x0055f340;
const ROUND_WRITER  = 0x0066f3b0;
const PACK_DATE     = 0x00533b50;
const FLAG_SNAP     = 0x00533eb0;
const RR_DRIVER     = 0x00668890;
const WALKER        = 0x0066f280;

// helper: dump 8 bytes as [i16,i16,i16,i16]
function dump4(p) {
    return [p.readS16(), p.add(2).readS16(), p.add(4).readS16(), p.add(6).readS16()];
}

// FUN_00533b50 : __thiscall, this=result(short*), day,month(c),year(us),flag
Interceptor.attach(ptr(PACK_DATE), {
    onEnter(args) {
        const sp = this.context.esp;
        this.result = this.context.ecx;
        // stack args (right of ECX):
        this.day = sp.add(4).readS16();
        this.month = sp.add(8).readS8();
        this.year = sp.add(0x0c).readU16();
        this.flag = sp.add(0x10).readS32();
        send({op:"packdate_enter",
              result: this.result.toString(),
              day:this.day, month:this.month, year:this.year, flag:this.flag,
              ret:this.returnAddress.toString()});
    },
    onLeave() {
        try { send({op:"packdate_leave", result:this.result.toString(),
                    buf: dump4(this.result), flag:this.flag}); } catch(e){}
    }
});

// FUN_00533eb0 : called from pack_date as FUN_00533eb0(flag_or_buf, buf_or_flag).
// Ghidra confused; we'll capture both interpretations.
Interceptor.attach(ptr(FLAG_SNAP), {
    onEnter(args) {
        const sp = this.context.esp;
        this.ecx = this.context.ecx;
        this.stack_arg1 = sp.add(4).readU32();
        // Try to read each as a short buffer
        let ecx_as_buf = null, stk_as_buf = null;
        try { ecx_as_buf = dump4(this.ecx); } catch(e){}
        try { stk_as_buf = dump4(ptr(this.stack_arg1)); } catch(e){}
        this.ecx_as_buf_before = ecx_as_buf;
        this.stk_as_buf_before = stk_as_buf;
        send({op:"flagsnap_enter",
              ecx: this.ecx.toString(),
              stack_arg1: "0x"+this.stack_arg1.toString(16),
              ecx_as_buf, stk_as_buf,
              ret: this.returnAddress.toString()});
    },
    onLeave() {
        let ecx_as_buf = null, stk_as_buf = null;
        try { ecx_as_buf = dump4(this.ecx); } catch(e){}
        try { stk_as_buf = dump4(ptr(this.stack_arg1)); } catch(e){}
        send({op:"flagsnap_leave",
              ecx_as_buf_before: this.ecx_as_buf_before,
              ecx_as_buf_after: ecx_as_buf,
              stk_as_buf_before: this.stk_as_buf_before,
              stk_as_buf_after: stk_as_buf});
    }
});

// FUN_0066f3b0: writer (already characterised)
Interceptor.attach(ptr(ROUND_WRITER), {
    onEnter(args) {
        const sp = this.context.esp;
        const buf = sp.add(4).readPointer();
        const round_idx = sp.add(8).readU16();
        this.buf = buf; this.round_idx = round_idx;
    },
    onLeave() {
        try {
            const rec = this.buf.add(this.round_idx * 0x41).readByteArray(0x41);
            send({op:"writer_done", round_idx:this.round_idx,
                  first8_hex: Array.from(new Uint8Array(rec.slice(0,8))).map(b=>b.toString(16).padStart(2,'0')).join('')}, null);
        } catch(e) {}
    }
});

// FUN_00668890 driver
Interceptor.attach(ptr(RR_DRIVER), {
    onEnter(args) {
        const sp = this.context.esp;
        send({op:"rr_driver_enter",
              ecx: this.context.ecx.toString(),
              a1: sp.add(4).readU32(),
              a2: sp.add(8).readU32(),
              a3: sp.add(0x0c).readU32(),
              a4: sp.add(0x10).readU32(),
              ret: this.returnAddress.toString()});
    },
    onLeave(retval) {
        send({op:"rr_driver_leave", retval: retval.toString()});
    }
});

// FUN_0066f280 walker
let walker_calls = 0;
Interceptor.attach(ptr(WALKER), {
    onEnter(args) {
        const sp = this.context.esp;
        walker_calls++;
        this.idx = walker_calls;
        send({op:"walker_enter", idx:this.idx,
              a1: sp.add(4).readS32(),
              a2: sp.add(8).readU32(),
              a3: sp.add(0x0c).readS32(),
              a4: sp.add(0x10).readS16(),
              a5: sp.add(0x14).readS16(),
              a6: sp.add(0x18).readS16(),
              a7: sp.add(0x1c).readU8(),
              ret: this.returnAddress.toString()});
    },
    onLeave(retval) {
        send({op:"walker_leave", idx:this.idx, retval: retval.toInt32()});
    }
});

send({op:"hooks_installed"});

// Now do the direct call
const fake_this = Memory.alloc(0x300);
fake_this.writeByteArray(new Array(0x300).fill(0));
fake_this.add(0x40).writeU16(2001);

const getter = new NativeFunction(ptr(SCHEDGETTER), 'pointer',
    ['pointer', 'uint8', 'pointer', 'pointer', 'uint32'], 'thiscall');
const p2 = fake_this.add(0xa9);
const p3 = fake_this.add(0x3a);

send({op:"calling"});
try {
    const rb = getter(fake_this, 0xFF, p2, p3, 0);
    send({op:"returned", retval: rb.toString(),
          round_count: fake_this.add(0xa9).readU16()});
} catch(e) { send({op:"crashed", err:e.toString()}); }

send({op:"done"});
"""

records = []; done = [False]
def on_message(m, d):
    if m["type"] == "send":
        p = m["payload"]; records.append(p)
        op = p.get("op")
        if op in ("hooks_installed","calling","returned","crashed","done","rr_driver_enter","rr_driver_leave"):
            print(f"[{op}] {p}")
        elif op == "flagsnap_enter":
            print(f"[flagsnap_enter] ecx={p['ecx']} stk={p['stack_arg1']} ecx_buf={p['ecx_as_buf']} stk_buf={p['stk_as_buf']}")
        elif op == "flagsnap_leave":
            print(f"[flagsnap_leave] ecx {p['ecx_as_buf_before']}->{p['ecx_as_buf_after']}  stk {p['stk_as_buf_before']}->{p['stk_as_buf_after']}")
        elif op == "packdate_enter":
            print(f"[packdate] day={p['day']} mon={p['month']} yr={p['year']} flag={p['flag']}")
        elif op == "walker_enter":
            print(f"[walker#{p['idx']}] a1={p['a1']} a2=0x{p['a2']:x} a3={p['a3']} a4..a7={p['a4']},{p['a5']},{p['a6']},{p['a7']}")
        if op == "done": done[0] = True
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()

t0 = time.time()
while not done[0] and time.time()-t0 < 30: time.sleep(0.1)

ts = time.strftime("%Y%m%d_%H%M%S")
(OUT / f"{ts}_v2_events.jsonl").write_text("\n".join(json.dumps(r) for r in records), encoding="utf-8")
print(f"[+] {len(records)} events -> {ts}_v2_events.jsonl")
session.detach()
