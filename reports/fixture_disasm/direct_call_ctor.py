"""Direct-call FUN_0055f040 (eng_second_ctor) to trigger the FULL construction
chain: schedule-getter + roster + FUN_00668890 (round-robin driver) + walker.

Hooks:
- FUN_0055f040 (ctor)                entry/leave
- FUN_00560320 (schedule-template)   entry/leave
- FUN_005601d0 (roster populator)    entry/leave
- FUN_00668890 (round-robin driver)  entry/leave
- FUN_0066f280 (walker)              entry/leave with args
- FUN_00667aa0 (post-schedule)       entry/leave
- FUN_004a5900 (final attach)        entry/leave
- FUN_0066f3b0 (round writer)        entry only
- FUN_0066f410 (slot writer)         entry only
- operator_new                       for 0x5ce allocations
- memory watch on schedule buffer for writes after ctor start
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
const CTOR         = 0x0055f040;
const SCHED_TPL    = 0x00560320;
const ROSTER       = 0x005601d0;
const RR_DRIVER    = 0x00668890;
const WALKER       = 0x0066f280;
const POST_SCHED   = 0x00667aa0;
const FINAL_ATTACH = 0x004a5900;
const WRITER       = 0x0066f3b0;
const SLOTWRITER   = 0x0066f410;

let comp_ptr = null;
let sched_buf_ptr = null;
let sched_buf_size = 0xbae;
let walker_count = 0;
let slot_count = 0;
let writer_count = 0;

Interceptor.attach(ptr(CTOR), {
    onEnter(args){
        comp_ptr = this.context.ecx;
        send({op:"ctor_enter", this_ptr: comp_ptr.toString()});
    },
    onLeave(rv){ send({op:"ctor_leave", retval:rv.toString()}); }
});
Interceptor.attach(ptr(SCHED_TPL), {
    onEnter(){ send({op:"sched_tpl_enter"}); },
    onLeave(rv){ send({op:"sched_tpl_leave", retval:rv.toString()});
        // Read the schedule buffer pointer just written to comp+0xba
        if (comp_ptr) {
            try { sched_buf_ptr = comp_ptr.add(0xba).readPointer();
                  send({op:"sched_buf_ptr", ptr: sched_buf_ptr.toString()}); } catch(e){}
        }
    }
});
Interceptor.attach(ptr(ROSTER), {
    onEnter(){ send({op:"roster_enter"}); },
    onLeave(rv){
        // roster likely stored at comp+0xb1 or similar; dump comp+0xa8..+0xd0
        let comp_dump = null;
        try { comp_dump = comp_ptr.add(0xa0).readByteArray(0x60); } catch(e){}
        send({op:"roster_leave", retval:rv.toString()}, comp_dump);
    }
});
Interceptor.attach(ptr(RR_DRIVER), {
    onEnter(args){
        const t = this.context.ecx;
        send({op:"rr_driver_enter", this_ptr:t.toString(), retaddr:this.returnAddress.toString()});
    },
    onLeave(rv){
        send({op:"rr_driver_leave", retval:rv.toString(), walker_count, slot_count, writer_count});
        // After the driver, dump the schedule buffer and everything comp
        if (sched_buf_ptr) {
            try {
                const raw = sched_buf_ptr.readByteArray(sched_buf_size);
                send({op:"sched_buf_after_driver", ptr:sched_buf_ptr.toString(), size:sched_buf_size}, raw);
            } catch(e){}
        }
    }
});
Interceptor.attach(ptr(WALKER), {
    onEnter(args){
        walker_count++;
        const sp = this.context.esp;
        this.idx = walker_count;
        send({op:"walker", idx:this.idx,
              a1:sp.add(4).readS32(),
              a2:sp.add(8).readU32(),
              a3:sp.add(0x0c).readS32(),
              a4:sp.add(0x10).readS16(),
              a5:sp.add(0x14).readS16(),
              a6:sp.add(0x18).readS16(),
              a7:sp.add(0x1c).readU8(),
              ret_addr:this.returnAddress.toString()});
    },
    onLeave(rv){ send({op:"walker_ret", idx:this.idx, retval:rv.toInt32()}); }
});
Interceptor.attach(ptr(POST_SCHED), {
    onEnter(){ send({op:"post_sched_enter"}); },
    onLeave(rv){ send({op:"post_sched_leave", retval:rv.toString()}); }
});
Interceptor.attach(ptr(FINAL_ATTACH), {
    onEnter(){ send({op:"final_enter"}); },
    onLeave(rv){ send({op:"final_leave", retval:rv.toString()}); }
});
Interceptor.attach(ptr(WRITER), {
    onEnter(args){
        writer_count++;
        const sp = this.context.esp;
        // capture minimal args
        send({op:"W", n:writer_count,
              round:sp.add(8).readU16(),
              day:sp.add(0x0c).readS32(),
              mon:sp.add(0x10).readS32(),
              do_:sp.add(0x14).readS32(),
              fl:sp.add(0x18).readS32(),
              t:sp.add(0x1c).readS32(),
              ret:this.returnAddress.toString()});
    }
});
Interceptor.attach(ptr(SLOTWRITER), {
    onEnter(args){
        slot_count++;
        const sp = this.context.esp;
        send({op:"S", n:slot_count,
              round:sp.add(8).readS16(),
              slot:sp.add(0x0c).readS16(),
              p4:sp.add(0x10).readS16(),
              p5:sp.add(0x14).readS8(),
              p6:sp.add(0x18).readS8(),
              payload:sp.add(0x1c).readU32(),
              ret:this.returnAddress.toString()});
    }
});
send({op:"hooks_installed"});

// Allocate a big fake comp record.
const fake_this = Memory.alloc(0x400);
fake_this.writeByteArray(new Array(0x400).fill(0));

// Zero-fill so any pointer field is null-checked
// param_2 = year (short)
// param_3 = undefined4 — pass 0 for now
const ctor = new NativeFunction(ptr(CTOR), 'pointer',
    ['pointer', 'uint16', 'uint32'], 'thiscall');
send({op:"calling_ctor", this_ptr: fake_this.toString()});
try {
    const rv = ctor(fake_this, 2001, 0);
    send({op:"ctor_returned_top", retval: rv.toString()});
} catch(e) {
    send({op:"ctor_crashed", err: e.toString(), stack: e.stack});
}
send({op:"done"});
"""

records = []; buffers = []; done=[False]
def on_message(m, d):
    if m["type"] == "send":
        p = m["payload"]; records.append(p)
        op = p.get("op")
        # Print key events
        if op in ("hooks_installed","calling_ctor","ctor_enter","ctor_leave",
                  "sched_tpl_enter","sched_tpl_leave","sched_buf_ptr",
                  "roster_enter","roster_leave",
                  "rr_driver_enter","rr_driver_leave",
                  "post_sched_enter","post_sched_leave",
                  "final_enter","final_leave",
                  "ctor_returned_top","ctor_crashed","done"):
            print(f"[{op}] {p}")
        elif op == "walker":
            print(f"  [walk#{p['idx']}] a1={p['a1']} a3={p['a3']} a4={p['a4']} a5={p['a5']} a6={p['a6']} a7={p['a7']}")
        elif op == "walker_ret":
            pass
        elif op == "sched_buf_after_driver" and d:
            buffers.append(d)
            print(f"  [sched_buf_after_driver] {len(d)} bytes captured")
        elif op == "S":
            if p['n'] <= 60 or p['n'] % 20 == 0:
                print(f"  [S#{p['n']} r{p['round']} sl{p['slot']}] p4={p['p4']} p5={p['p5']} p6={p['p6']} payload=0x{p['payload']:08x}")
        elif op == "W":
            pass  # skip verbose writer output
        if op == "done": done[0] = True
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()
t0=time.time()
while not done[0] and time.time()-t0 < 60: time.sleep(0.1)
ts = time.strftime("%Y%m%d_%H%M%S")
(OUT / f"{ts}_ctor.jsonl").write_text("\n".join(json.dumps(r) for r in records), encoding="utf-8")
for i,b in enumerate(buffers):
    (OUT / f"{ts}_ctor_buffer_{i}.bin").write_bytes(b)
print(f"[+] {len(records)} events, {len(buffers)} buffers -> {ts}_*")
session.detach()
