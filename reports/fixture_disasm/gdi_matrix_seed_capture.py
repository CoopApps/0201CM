"""Direct-call cm0102-gdi.exe FUN_00669340 (matrix base seeder) for
n_even values 4, 6, 8, 24, and dump each returned matrix to JSON so
Rust tests can compare byte-exact.

Signature (from decompile):
    FUN_00669780(int *spine, int n_even)   // GDI equivalent 0x00669340
    - spine[0] guard
    - spine[1..=n_even] each point to `n_even` ints (the rows)
    - After the call, cells are populated per the round-robin seeder.
"""
import frida, sys, json, time
from pathlib import Path

OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime"); OUT.mkdir(parents=True, exist_ok=True)
dev = frida.get_local_device()
pid = next((p.pid for p in dev.enumerate_processes() if p.name.lower()=="cm0102_gdi.exe"), None)
if not pid: sys.exit("cm0102_GDI.exe not running")
print(f"attaching to cm0102_GDI.exe pid={pid}")
session = frida.attach(pid)

script_src = r"""
const SEED = 0x00669340;

// C signature: FUN_00669340(int* spine, int n_even) — cdecl, 2 args
const seed_fn = new NativeFunction(ptr(SEED), 'uint32', ['pointer', 'int32']);

function capture(n_even) {
    // Allocate (n_even + 1) pointers for the spine
    const spine = Memory.alloc((n_even + 1) * 4);
    // Zero-fill spine
    for (let i = 0; i <= n_even; i++) {
        spine.add(i * 4).writeU32(0);
    }
    // Allocate row buffers, store into spine[1..=n_even]
    const rows = [];
    for (let i = 1; i <= n_even; i++) {
        const row = Memory.alloc(n_even * 4);
        for (let c = 0; c < n_even; c++) row.add(c * 4).writeU32(0);
        rows.push(row);
        spine.add(i * 4).writePointer(row);
    }
    // Call the seeder
    seed_fn(spine, n_even);
    // Read back rows
    const out = [];
    // Guard row [0]
    out.push([]);
    for (let i = 0; i < n_even; i++) {
        const row = [];
        for (let c = 0; c < n_even; c++) {
            row.push(rows[i].add(c * 4).readS32());
        }
        out.push(row);
    }
    return out;
}

const dumps = {};
for (const n of [4, 6, 8, 10, 24]) {
    dumps[n] = capture(n);
}
send({op: "dump", data: dumps});
send({op: "done"});
"""

records = []
done = [False]
def on_message(m, d):
    if m["type"] == "send":
        p = m["payload"]
        records.append(p)
        if p.get("op") == "done":
            done[0] = True
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()
t0 = time.time()
while not done[0] and time.time()-t0 < 15: time.sleep(0.1)

for r in records:
    if r.get("op") == "dump":
        ts = time.strftime("%Y%m%d_%H%M%S")
        p = OUT / f"{ts}_gdi_matrix_seed.json"
        p.write_text(json.dumps(r["data"], indent=2))
        print(f"[+] wrote {p}")
        # Also print a small one for quick inspection
        print("\nn_even=4 matrix (guard [0] + rows [1..4]):")
        for i, row in enumerate(r["data"]["4"]):
            print(f"  [{i}]: {row}")
        print("\nn_even=6 matrix:")
        for i, row in enumerate(r["data"]["6"]):
            print(f"  [{i}]: {row}")

session.detach()
