"""v3 direct-call: hooks FUN_0066f3b0 AND FUN_0066f410 to fully account for
every write to the 2990-byte schedule buffer during 0x0055f340."""
import frida, sys, json, time
from pathlib import Path
OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime"); OUT.mkdir(parents=True, exist_ok=True)
dev = frida.get_local_device()
pid = next((p.pid for p in dev.enumerate_processes() if p.name.lower()=="cm0102.exe"), None)
if not pid: sys.exit("cm0102.exe not running")
print(f"attaching pid={pid}")
session = frida.attach(pid)

script_src = r"""
const SCHEDGETTER = 0x0055f340;
const WRITER      = 0x0066f3b0;
const SLOTWRITER  = 0x0066f410;

Interceptor.attach(ptr(WRITER), {
    onEnter(args) {
        const sp = this.context.esp;
        send({op:"writer",
              buf: sp.add(4).readPointer().toString(),
              round_idx: sp.add(8).readU16(),
              day: sp.add(0x0c).readS32(),
              month: sp.add(0x10).readS32(),
              day_off: sp.add(0x14).readS32(),
              flag: sp.add(0x18).readS32(),
              type_byte: sp.add(0x1c).readS32(),
              year: sp.add(0x20).readS16(),  // read as short - upper is stack garbage
              prize: sp.add(0x24).readS32(),
              ret: this.returnAddress.toString()});
    }
});

Interceptor.attach(ptr(SLOTWRITER), {
    onEnter(args) {
        const sp = this.context.esp;
        // FUN_0066f410(buf, round_idx, sub_slot, p4, p5, p6, u32_payload)
        send({op:"slotwriter",
              buf: sp.add(4).readPointer().toString(),
              round_idx: sp.add(8).readS16(),
              sub_slot: sp.add(0x0c).readS16(),
              p4: sp.add(0x10).readS16(),
              p5: sp.add(0x14).readS8(),
              p6: sp.add(0x18).readS8(),
              payload: sp.add(0x1c).readU32(),
              ret: this.returnAddress.toString()});
    }
});

send({op:"hooks_installed"});

const fake_this = Memory.alloc(0x300);
fake_this.writeByteArray(new Array(0x300).fill(0));
fake_this.add(0x40).writeU16(2001);
const getter = new NativeFunction(ptr(SCHEDGETTER), 'pointer',
    ['pointer','uint8','pointer','pointer','uint32'], 'thiscall');
const p2 = fake_this.add(0xa9); const p3 = fake_this.add(0x3a);
send({op:"calling"});
try {
    const rb = getter(fake_this, 0xFF, p2, p3, 0);
    send({op:"returned", retval: rb.toString(),
          round_count: fake_this.add(0xa9).readU16()});
    // Dump the buffer contents
    const raw = rb.readByteArray(0xbae);
    send({op:"buffer_dump", ptr:rb.toString(), size:0xbae}, raw);
} catch(e) { send({op:"crashed", err:e.toString()}); }

send({op:"done"});
"""

records = []; buffers = []; done=[False]
def on_message(m, d):
    if m["type"] == "send":
        p = m["payload"]; records.append(p)
        op = p.get("op")
        if op == "writer":
            print(f"[W  r{p['round_idx']:2d}] day={p['day']:2d} m={p['month']:2d} do={p['day_off']} fl={p['flag']:3d} t={p['type_byte']} ret=0x{int(p['ret'],16):x}")
        elif op == "slotwriter":
            print(f"[S  r{p['round_idx']:2d} slot{p['sub_slot']}] p4={p['p4']:3d} p5={p['p5']:3d} p6={p['p6']:3d} payload=0x{p['payload']:08x}  ret=0x{int(p['ret'],16):x}")
        elif op == "buffer_dump" and d:
            buffers.append(d)
        elif op in ("hooks_installed","calling","returned","crashed","done"):
            print(f"[{op}] {p}")
        if op == "done": done[0] = True
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()
t0 = time.time()
while not done[0] and time.time()-t0 < 30: time.sleep(0.1)
ts = time.strftime("%Y%m%d_%H%M%S")
(OUT / f"{ts}_v3.jsonl").write_text("\n".join(json.dumps(r) for r in records), encoding="utf-8")
for i,b in enumerate(buffers):
    (OUT / f"{ts}_v3_buffer_{i}.bin").write_bytes(b)
print(f"[+] {len(records)} events, {len(buffers)} buffers -> {ts}_v3_*")
session.detach()
