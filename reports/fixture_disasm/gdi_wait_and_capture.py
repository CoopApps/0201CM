"""Attach Frida. Capture EVERY ctor call at 0x0055f240 and filter on
comp_id at onLeave (after the +0x50 write).

Bug fixed vs previous version: cid gate now runs in onLeave (after the
ctor writes comp_id to +0x50), not onEnter.
"""
import frida, sys, json, time
from pathlib import Path

OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")
OUT.mkdir(parents=True, exist_ok=True)

dev = frida.get_local_device()
pid = next((p.pid for p in dev.enumerate_processes() if p.name.lower()=="cm0102_gdi.exe"), None)
if not pid: sys.exit("cm0102_GDI.exe not running - launch it first")
print(f"attaching to cm0102_GDI.exe pid {pid}")
session = frida.attach(pid)

script_src = r"""
let walker_calls = [];
let rng_calls = [];
let fixture_inserts = [];
let phase = "IDLE";
let cur_target = null;

Interceptor.attach(ptr(0x008fbe20), {
    onEnter() { this.n = this.context.esp.add(4).readS32(); },
    onLeave(rv) {
        if (phase !== "IDLE") {
            rng_calls.push({phase, n: this.n, retval: rv.toInt32()});
        }
    }
});

Interceptor.attach(ptr(0x0055f240), {
    onEnter() {
        // Always begin capturing - filter on cid at onLeave.
        this.target = this.context.ecx;
        this.pre_walker = walker_calls.length;
        this.pre_rng = rng_calls.length;
        this.pre_fixtures = fixture_inserts.length;
        // Enter CTOR phase so driver/perturb/walker/fixture hooks fire
        cur_target = this.target;
        phase = "CTOR";
        send({op:"any_ctor_enter", this_ptr: this.target.toString()});
    },
    onLeave() {
        const t = this.target;
        let cid = -1, year = 0;
        try {
            cid = t.add(0x50).readU8();
            year = t.add(0x40).readU16();
        } catch(e){}
        send({op:"any_ctor_leave", cid, year,
              walker_delta: walker_calls.length - this.pre_walker,
              rng_delta: rng_calls.length - this.pre_rng,
              fixtures_delta: fixture_inserts.length - this.pre_fixtures});
        if (cid === 9) {
            try {
                const nc = t.add(0x3e).readS16();
                const nr = t.add(0xa9).readS16();
                const clubs_ptr = t.add(0xb1).readPointer();
                const sched_ptr = t.add(0xba).readPointer();
                const alt = t.add(0xea).readPointer();
                send({op:"ctor_leave", cid: 9,
                      n_clubs: nc, n_rounds: nr,
                      clubs_ptr: clubs_ptr.toString(),
                      sched_ptr: sched_ptr.toString(),
                      alt_pair_list: alt.toString(),
                      walker_total: walker_calls.length,
                      rng_total: rng_calls.length,
                      fixtures_total: fixture_inserts.length});
                if (!sched_ptr.isNull()) send({op:"sched_buffer"}, sched_ptr.readByteArray(0xbae));
                if (!clubs_ptr.isNull() && nc > 0 && nc <= 32) {
                    send({op:"clubs_table"}, clubs_ptr.readByteArray(nc * 0x3b));
                    const entries = [];
                    for (let i = 0; i < nc; i++) {
                        try {
                            const cptr = clubs_ptr.add(i * 0x3b).readPointer();
                            entries.push({slot:i, club_id: cptr.readS32(),
                                          nation_at_69: cptr.add(0x69).readS32(),
                                          club_ptr: cptr.toString()});
                        } catch(e) { entries.push({slot:i, error:e.toString()}); }
                    }
                    send({op:"clubs_dereferenced", entries});
                }
                send({op:"walker_full_trace", calls: walker_calls});
                send({op:"rng_full_trace", calls: rng_calls});
                send({op:"fixture_full_trace", inserts: fixture_inserts});
            } catch(e) { send({op:"ctor_leave_err", err:e.toString()}); }
        }
        cur_target = null;
        phase = "IDLE";
        // Reset arrays after each ctor so per-comp traces are isolated.
        // Actually keep them if it was eng_second so full_trace has data.
        if (cid !== 9) {
            walker_calls = [];
            rng_calls = [];
            fixture_inserts = [];
        }
    }
});

Interceptor.attach(ptr(0x00668450), {
    onEnter() {
        if (phase === "CTOR") {
            phase = "DRIVER";
            send({op:"driver_enter",
                  walker_before: walker_calls.length,
                  rng_before: rng_calls.length});
        }
    },
    onLeave(rv) {
        if (phase === "DRIVER") {
            phase = "CTOR";
            send({op:"driver_leave", retval: rv.toInt32(),
                  walker_after: walker_calls.length,
                  rng_after: rng_calls.length,
                  fixtures: fixture_inserts.length});
        }
    }
});

Interceptor.attach(ptr(0x0066b900), {
    onEnter() {
        if (phase === "DRIVER") {
            phase = "PERTURB";
            const t = this.context.ecx;
            send({op:"perturb_enter", rng_before: rng_calls.length});
            try {
                const nc = t.add(0x3e).readS16();
                const clubs = t.add(0xb1).readPointer();
                if (!clubs.isNull() && nc > 0) {
                    send({op:"clubs_before_perturb", n_clubs: nc},
                         clubs.readByteArray(nc * 0x3b));
                    const entries = [];
                    for (let i = 0; i < nc; i++) {
                        try {
                            const cptr = clubs.add(i * 0x3b).readPointer();
                            entries.push({slot:i, club_id: cptr.readS32(),
                                          nation_at_69: cptr.add(0x69).readS32(),
                                          club_ptr: cptr.toString()});
                        } catch(e){}
                    }
                    send({op:"clubs_before_perturb_deref", entries});
                }
            } catch(e){}
        }
    },
    onLeave() {
        if (phase === "PERTURB") {
            phase = "DRIVER";
            const t = this.context.ecx;
            send({op:"perturb_leave", rng_after: rng_calls.length});
            try {
                const nc = t.add(0x3e).readS16();
                const clubs = t.add(0xb1).readPointer();
                if (!clubs.isNull() && nc > 0) {
                    send({op:"clubs_after_perturb", n_clubs: nc},
                         clubs.readByteArray(nc * 0x3b));
                    const entries = [];
                    for (let i = 0; i < nc; i++) {
                        try {
                            const cptr = clubs.add(i * 0x3b).readPointer();
                            entries.push({slot:i, club_id: cptr.readS32(),
                                          nation_at_69: cptr.add(0x69).readS32(),
                                          club_ptr: cptr.toString()});
                        } catch(e){}
                    }
                    send({op:"clubs_after_perturb_deref", entries});
                }
            } catch(e){}
        }
    }
});

Interceptor.attach(ptr(0x0066ee40), {
    onEnter(args) {
        if (phase === "DRIVER") {
            const sp = this.context.esp;
            this.wi = {
                prev_col: sp.add(4).readS32(),
                state_before: sp.add(8).readPointer().readU8(),
                comp_id: sp.add(0x0c).readS32(),
                n_clubs: sp.add(0x10).readS16(),
                matches_per_pair: sp.add(0x14).readS16(),
                n_rounds: sp.add(0x18).readS16(),
                flag_byte: sp.add(0x1c).readU8(),
            };
            this.state_ptr = sp.add(8).readPointer();
        }
    },
    onLeave(rv) {
        if (this.wi) {
            walker_calls.push({...this.wi,
                               state_after: this.state_ptr.readU8(),
                               retval: rv.toInt32(),
                               idx: walker_calls.length});
        }
    }
});

Interceptor.attach(ptr(0x00594eb0), {
    onEnter() {
        if (phase === "DRIVER") {
            const fp = this.context.esp.add(4).readPointer();
            try {
                fixture_inserts.push({
                    idx: fixture_inserts.length,
                    cid: fp.readS32(),
                    home: fp.add(0x0c).readS32(),
                    away: fp.add(0x10).readS32(),
                    year: fp.add(0x28).readS16(),
                    doy: fp.add(0x2a).readS16(),
                    rwh: fp.add(0x34).readS16(),
                });
            } catch(e){}
        }
    }
});

send({op:"hooks_installed"});
"""

records = []
buffers = {}
ctor_leaves = []
def on_message(m, d):
    if m["type"] == "send":
        p = m["payload"]
        records.append(p)
        op = p.get("op")
        if op == "any_ctor_leave":
            ctor_leaves.append(p)
            cid = p.get("cid")
            print(f"[any_ctor_leave] cid={cid} year={p.get('year')} "
                  f"walker={p.get('walker_delta')} rng={p.get('rng_delta')} "
                  f"fixtures={p.get('fixtures_delta')}")
        elif op not in ("clubs_table", "sched_buffer",
                       "clubs_before_perturb", "clubs_after_perturb",
                       "walker_full_trace", "rng_full_trace",
                       "fixture_full_trace",
                       "clubs_before_perturb_deref",
                       "clubs_after_perturb_deref",
                       "clubs_dereferenced"):
            print(f"[{op}] {p if len(str(p)) < 300 else str(p)[:300]}")
        else:
            print(f"[{op}] (large payload captured)")
        if d:
            buffers.setdefault(op, []).append(d)
    elif m["type"] == "error":
        print(f"[frida-error] {m}", file=sys.stderr)

script = session.create_script(script_src)
script.on("message", on_message)
script.load()
time.sleep(0.5)

print("\n" + "=" * 60)
print("hooks armed - drive the UI now:")
print("  1. Setup Game -> Start New Game")
print("  2. Select League(s) -> pick England only -> Next")
print("  3. 'CD Not Required' dialog -> OK")
print("  4. Wait for 'Creating Shortlists' to build the DB")
print("Prints each ctor call as it fires. Ctrl-C when done.")
print("=" * 60 + "\n")

t0 = time.time()
last_print = time.time()
try:
    while True:
        time.sleep(1)
        if any(r.get("op") == "ctor_leave" for r in records):
            print(f"\neng_second ctor complete! elapsed={time.time()-t0:.1f}s")
            time.sleep(3)
            break
        if time.time() - last_print > 15 and not any(r.get("op") == "any_ctor_leave" for r in records[-5:]):
            print(f"  [{int(time.time()-t0):4d}s] ctor_leaves={len(ctor_leaves)} "
                  f"unique_cids={sorted(set(l['cid'] for l in ctor_leaves))[:20]}"
                  + ("..." if len(ctor_leaves) > 20 else ""))
            last_print = time.time()
except KeyboardInterrupt:
    print("\nstopped by user")

ts = time.strftime("%Y%m%d_%H%M%S")
(OUT / f"{ts}_manual_natural.jsonl").write_text(
    "\n".join(json.dumps(r) for r in records), encoding="utf-8")
for kind, bufs in buffers.items():
    for i, b in enumerate(bufs):
        (OUT / f"{ts}_manual_{kind}_{i}.bin").write_bytes(b)

print(f"\n{len(records)} events total, saved with prefix {ts}_manual_")
print(f"  ctor_leaves: {len(ctor_leaves)}")
if ctor_leaves:
    print(f"  comp_ids seen: {sorted(set(l['cid'] for l in ctor_leaves))}")
print(f"  eng_second ctor_leave: {sum(1 for r in records if r.get('op')=='ctor_leave')}")
print(f"  driver_leave: {sum(1 for r in records if r.get('op')=='driver_leave')}")
print(f"  perturb_leave: {sum(1 for r in records if r.get('op')=='perturb_leave')}")
print(f"  buffers: {list(buffers.keys())}")

session.detach()
