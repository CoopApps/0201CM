"""Direct-call cm0102-gdi.exe FUN_0066b900 (matrix perturbation) and
capture:
  * RNG call sequence (every FUN_008fbe20 + every FUN_0093556c LCG hit)
  * clubs-table state before + after
  * matrix state before + after
  * final RNG state

Then dump to JSON so Rust tests can differential.

Comp record we synthesize:
    +0x3e (short) n_clubs = 24
    +0x40 (word)  year_base = 2001
    +0xb1 (u32*)  clubs_table = Memory.alloc(24 * 0x3b), populated with
                    slot_id at +0x00 (i32) and nation_id at +0x69 (u16)
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
const PERTURB      = 0x0066b900;
const SEED         = 0x00669340;
const POOL_RNG     = 0x008fbe20;
const LCG_RAND     = 0x0093556c;   // FUN_00935a94 equivalent in GDI (needs verification)
const LCG_SRAND    = 0x00935562;   // FUN_00935a8a equivalent in GDI
const DAT_LCG_STATE = 0x00ac26c0;  // conventional MSVC srand state global (DirectDraw location, unclear if GDI same)

let pool_rng_calls = [];
let lcg_calls = [];
let lcg_srand_calls = [];

Interceptor.attach(ptr(POOL_RNG), {
    onEnter(args) {
        const sp = this.context.esp;
        this.n = sp.add(4).readS32();
    },
    onLeave(rv) {
        pool_rng_calls.push({n: this.n, retval: rv.toInt32()});
    }
});
// The LCG functions live inside the CRT — hooking may be risky; wrap
// in try. We'll match on the exact GDI VAs and skip if absent.
try {
    Interceptor.attach(ptr(LCG_RAND), {
        onLeave(rv) { lcg_calls.push(rv.toInt32() & 0x7fff); }
    });
} catch (e) { send({op:"warn_lcg_rand_hook_failed", err:e.toString()}); }
try {
    Interceptor.attach(ptr(LCG_SRAND), {
        onEnter(args) {
            const sp = this.context.esp;
            lcg_srand_calls.push(sp.add(4).readS32());
        }
    });
} catch (e) { send({op:"warn_lcg_srand_hook_failed", err:e.toString()}); }

send({op: "hooks_installed"});

// Set up the synthetic comp record.
const N_CLUBS = 24;
const CLUB_STRIDE = 0x3b;
const comp = Memory.alloc(0x300);
comp.writeByteArray(new Array(0x300).fill(0));
comp.add(0x3e).writeU16(N_CLUBS);
comp.add(0x40).writeU16(2001);

// Allocate club table: N_CLUBS * 0x3b bytes, populated so we can
// track which slot moved where. Write:
//   +0x00 (i32) = 1000 + slot_id (unique marker)
//   +0x04..+0x68 = 0
//   +0x69 (u16) = 900 + slot_id (nation-check safe distinct from host_nation=-1)
const clubs = Memory.alloc(N_CLUBS * CLUB_STRIDE);
for (let s = 0; s < N_CLUBS; s++) {
    const base = clubs.add(s * CLUB_STRIDE);
    for (let i = 0; i < CLUB_STRIDE; i++) base.add(i).writeU8(0);
    base.writeS32(1000 + s);
    base.add(0x69).writeU16(900 + s);
}
comp.add(0xb1).writePointer(clubs);

// Also seed the adjacency matrix via matrix_seed_base first, so that
// perturb has something to work with (perturb reads the matrix if
// its post-shuffle phases touch it).
const spine = Memory.alloc((N_CLUBS + 1) * 4);
for (let i = 0; i <= N_CLUBS; i++) spine.add(i * 4).writeU32(0);
const rows = [];
for (let i = 1; i <= N_CLUBS; i++) {
    const row = Memory.alloc(N_CLUBS * 4);
    for (let c = 0; c < N_CLUBS; c++) row.add(c * 4).writeU32(0);
    rows.push(row);
    spine.add(i * 4).writePointer(row);
}
const seed_fn = new NativeFunction(ptr(SEED), 'uint32', ['pointer', 'int32']);
seed_fn(spine, N_CLUBS);

// Snapshot the matrix BEFORE perturb.
function snapshotMatrix() {
    const out = [[]];
    for (let i = 0; i < N_CLUBS; i++) {
        const row = [];
        for (let c = 0; c < N_CLUBS; c++) row.push(rows[i].add(c * 4).readS32());
        out.push(row);
    }
    return out;
}
function snapshotClubs() {
    const out = [];
    for (let s = 0; s < N_CLUBS; s++) {
        const b = clubs.add(s * CLUB_STRIDE);
        out.push({id: b.readS32(), nation: b.add(0x69).readU16()});
    }
    return out;
}
const matrix_before = snapshotMatrix();
const clubs_before = snapshotClubs();

// Call the perturbation. Signature: __thiscall(comp, n_even).
// Frida places `this` in ECX; second arg on stack.
const perturb_fn = new NativeFunction(ptr(PERTURB), 'void',
    ['pointer', 'int32'], 'thiscall');
send({op: "calling"});
try {
    perturb_fn(comp, N_CLUBS);
    send({op: "returned"});
} catch (e) {
    send({op: "crashed", err: e.toString()});
}

const matrix_after = snapshotMatrix();
const clubs_after = snapshotClubs();

send({op: "capture",
      matrix_before,
      matrix_after,
      clubs_before,
      clubs_after,
      pool_rng_calls,
      lcg_calls,
      lcg_srand_calls});

send({op: "done"});
"""

records = []
done = [False]
def on_message(m, d):
    if m["type"] == "send":
        p = m["payload"]
        records.append(p)
        if p.get("op") in ("hooks_installed", "calling", "returned", "crashed", "done"):
            print(f"[{p['op']}] " + (p.get('err') or ""))
        if p.get("op") == "done":
            done[0] = True
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()
t0 = time.time()
while not done[0] and time.time()-t0 < 30: time.sleep(0.1)

ts = time.strftime("%Y%m%d_%H%M%S")
for r in records:
    if r.get("op") == "capture":
        p = OUT / f"{ts}_gdi_perturb.json"
        p.write_text(json.dumps(r, indent=2))
        print(f"[+] {len(r['pool_rng_calls'])} pool RNG calls, "
              f"{len(r['lcg_calls'])} LCG rand hits, "
              f"{len(r['lcg_srand_calls'])} srand calls -> {p}")
        # Print a small summary
        print("  Pool RNG calls (n, returned):")
        for c in r['pool_rng_calls'][:5]:
            print(f"    n={c['n']:<8} -> {c['retval']}")
        if r['pool_rng_calls'][5:]:
            print(f"    ... and {len(r['pool_rng_calls']) - 5} more")
        print("\n  Clubs table order after shuffle (first 8 slots):")
        for i, c in enumerate(r['clubs_after'][:8]):
            b = r['clubs_before'][i]
            print(f"    slot {i}: id={c['id']} (was {b['id']}), nation={c['nation']}")

session.detach()
