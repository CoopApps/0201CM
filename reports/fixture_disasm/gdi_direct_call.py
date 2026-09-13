"""Direct-call the GDI schedule-getter and capture the 2990-byte buffer.
Compare against the DirectDraw buffer to prove GDI == DD for this getter.
"""
import frida, sys, json, time, hashlib
from pathlib import Path
OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime"); OUT.mkdir(parents=True, exist_ok=True)
dev = frida.get_local_device()
pid = next((p.pid for p in dev.enumerate_processes() if p.name.lower()=="cm0102_gdi.exe"), None)
if not pid: sys.exit("cm0102_GDI.exe not running")
print(f"attaching to cm0102_GDI.exe pid={pid}")
session = frida.attach(pid)

script_src = r"""
// GDI addresses (from address discovery)
const SCHEDGETTER = 0x0055f540;
const WRITER      = 0x0066ef70;
const SLOTWRITER  = 0x0066efd0;

Interceptor.attach(ptr(WRITER), {
    onEnter(args){
        const sp = this.context.esp;
        send({op:"W",
              buf: sp.add(4).readPointer().toString(),
              round: sp.add(8).readU16(),
              day: sp.add(0x0c).readS32(),
              month: sp.add(0x10).readS32(),
              day_off: sp.add(0x14).readS32(),
              flag: sp.add(0x18).readS32(),
              type_byte: sp.add(0x1c).readS32(),
              year: sp.add(0x20).readS16(),
              prize: sp.add(0x24).readS32(),
              ret: this.returnAddress.toString()});
    }
});
Interceptor.attach(ptr(SLOTWRITER), {
    onEnter(args){
        const sp = this.context.esp;
        send({op:"S",
              buf: sp.add(4).readPointer().toString(),
              round: sp.add(8).readS16(),
              slot: sp.add(0x0c).readS16(),
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
    const raw = rb.readByteArray(0xbae);
    send({op:"buffer_dump", ptr:rb.toString(), size:0xbae}, raw);
} catch(e) { send({op:"crashed", err:e.toString()}); }
send({op:"done"});
"""

records = []; buffers = []; done = [False]
def on_message(m, d):
    if m["type"] == "send":
        p = m["payload"]; records.append(p)
        op = p.get("op")
        if op == "buffer_dump" and d:
            buffers.append(d)
            print(f"[buffer] {len(d)} bytes captured")
        elif op == "W":
            print(f"[W r{p['round']:2d}] day={p['day']:2d} m={p['month']:2d} do={p['day_off']} fl={p['flag']:3d} t={p['type_byte']}")
        elif op != "S":
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
(OUT / f"{ts}_gdi.jsonl").write_text("\n".join(json.dumps(r) for r in records), encoding="utf-8")
for i,b in enumerate(buffers):
    p = OUT / f"{ts}_gdi_buffer_{i}.bin"
    p.write_bytes(b)
    sha = hashlib.sha256(b).hexdigest()
    print(f"[+] wrote {p}   SHA256 = {sha}")

# Compare with DirectDraw capture
dd_path = OUT / "20260913_113106_direct_buffer_0.bin"
if dd_path.exists() and buffers:
    dd_b = dd_path.read_bytes()
    gdi_b = buffers[0]
    dd_sha = hashlib.sha256(dd_b).hexdigest()
    gdi_sha = hashlib.sha256(gdi_b).hexdigest()
    print(f"\n=== BUFFER COMPARISON ===")
    print(f"DD  buffer SHA256: {dd_sha}  ({len(dd_b)} bytes)")
    print(f"GDI buffer SHA256: {gdi_sha}  ({len(gdi_b)} bytes)")
    if dd_sha == gdi_sha:
        print("[+] GDI == DD  BYTE-EXACT MATCH ✓ port is valid")
    else:
        print("[!] GDI != DD  divergent")
        # Report first diff
        for i in range(min(len(dd_b), len(gdi_b))):
            if dd_b[i] != gdi_b[i]:
                print(f"    First diff at byte {i} (round {i//0x41} +0x{i%0x41:02x}): DD={dd_b[i]:#04x} GDI={gdi_b[i]:#04x}")
                break

session.detach()
