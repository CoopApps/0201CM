"""Direct-call the schedule-getter at 0x0055f340 via Frida, bypassing
user interaction. Allocates a scratch comp-record buffer, initialises
the season year at +0x40 to 2001, and calls the getter with
param_1 = 0xFF (standard install path).

We also install pre-call hooks on FUN_0066f3b0 so every round-record
write is captured as we go. After the call returns, dump the 2990-byte
buffer to disk.
"""
import frida, sys, json, time
from pathlib import Path

OUT_DIR = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")
OUT_DIR.mkdir(parents=True, exist_ok=True)

dev = frida.get_local_device()
pid = None
for p in dev.enumerate_processes():
    if p.name.lower() == "cm0102.exe":
        pid = p.pid; break
if pid is None:
    print("cm0102.exe not running", file=sys.stderr); sys.exit(1)
print(f"attaching to cm0102.exe pid={pid}")
session = frida.attach(pid)

# The script installs the FUN_0066f3b0 hook, then calls the getter
# directly on a Memory-alloc'd fake comp record.
script_src = r"""
const SCHEDGETTER = 0x0055f340;
const ROUND_WRITER = 0x0066f3b0;

// hook the writer first
Interceptor.attach(ptr(ROUND_WRITER), {
    onEnter(args) {
        const sp = this.context.esp;
        const buf = sp.add(4).readPointer();
        const round_idx = sp.add(8).readU16();
        const day = sp.add(0x0c).readS32();
        const month = sp.add(0x10).readS32();
        const day_off = sp.add(0x14).readS32();
        const flag = sp.add(0x18).readS32();
        const type_byte = sp.add(0x1c).readS32();
        const year = sp.add(0x20).readS32();
        const p9 = sp.add(0x24).readS32();
        send({op: "roundwriter",
              buf: buf.toString(),
              round_idx, day, month, day_off, flag, type_byte, year, p9,
              retaddr: this.returnAddress.toString()});
    }
});

send({op: "hook_installed"});

// Now allocate a fake comp record, set year at +0x40, call the getter.
// We only need at least 0x50 bytes for it not to crash on internal reads.
const fake_this = Memory.alloc(0x300);
fake_this.writeByteArray(new Array(0x300).fill(0));

// Season year at +0x40 (word) - the ax read in eng_second at 0x0055f3c5:
//   mov ax, word ptr [edi + 0x40]
fake_this.add(0x40).writeU16(2001);

// Set vtable pointer at +0x00 - the ctor uses [edi] a lot; if we
// crash on virtual call we'll come back to this.
// For now leave vtable = 0; if crashes we'll set it.

// Now call the getter. It's __thiscall so `this` in ECX. We use a
// NativeFunction with 'thiscall' calling convention (Frida supports
// 'stdcall'/'thiscall'/'fastcall' abis on x86-32).
// Frida thiscall: argTypes lists ALL args, including `this` first.
const getter = new NativeFunction(ptr(SCHEDGETTER),
    'pointer',                    // returns buffer ptr
    ['pointer', 'uint8', 'pointer', 'pointer', 'uint32'],
    'thiscall');                  // this (first arg) goes in ECX

// Args to the getter (from FUN_00560320:26):
//   param_1 = 0xFF (byte)
//   param_2 = &comp+0xa9 (round count out)
//   param_3 = &comp+0x3a (?)
//   param_4 = 0
const p2 = fake_this.add(0xa9);
const p3 = fake_this.add(0x3a);

send({op: "calling_getter", this_ptr: fake_this.toString(),
      p2: p2.toString(), p3: p3.toString()});

let retbuf = null;
try {
    // On x86 __thiscall, NativeFunction with 'thiscall' abi puts the
    // this pointer in ECX and pushes the remaining args right-to-left
    // on stack (caller cleans up).
    retbuf = getter(fake_this, 0xFF, p2, p3, 0);
    send({op: "getter_returned", retval: retbuf.toString(),
          round_count: fake_this.add(0xa9).readU16()});
} catch (e) {
    send({op: "getter_crashed", err: e.toString()});
}

// If we got a buffer, dump it
if (retbuf !== null && !retbuf.isNull()) {
    try {
        const raw = retbuf.readByteArray(0xbae);
        send({op: "buffer_dump", ptr: retbuf.toString(), size: 0xbae}, raw);
    } catch (e) {
        send({op: "buffer_dump_err", err: e.toString()});
    }
}

send({op: "done"});
"""

records = []
buffers = []
done = [False]

def on_message(msg, data):
    if msg["type"] == "send":
        payload = msg["payload"]
        payload["_ts"] = time.time()
        records.append(payload)
        print(f"[{payload['op']}] {json.dumps({k: v for k, v in payload.items() if k not in ('_ts',)})}")
        if payload.get("op") == "buffer_dump" and data is not None:
            buffers.append(data)
        if payload.get("op") == "done":
            done[0] = True
    elif msg["type"] == "error":
        print(f"[frida-error] {msg}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()

# wait up to 30s for done
start = time.time()
while not done[0] and time.time() - start < 30:
    time.sleep(0.1)

ts = time.strftime("%Y%m%d_%H%M%S")
(OUT_DIR / f"{ts}_direct_call.jsonl").write_text(
    "\n".join(json.dumps(r) for r in records), encoding="utf-8")
for i, b in enumerate(buffers):
    (OUT_DIR / f"{ts}_direct_buffer_{i}.bin").write_bytes(b)
print(f"\n[+] {len(records)} events, {len(buffers)} buffers saved to {OUT_DIR}/{ts}_*")

session.detach()
