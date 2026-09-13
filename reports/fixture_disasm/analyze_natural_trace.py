"""Analyze the captured natural trace from cm0102_GDI eng_second_ctor.

Extracts:
  - Pre-perturb and post-perturb 24-club orderings (with dereferenced Club*)
  - The 2990-byte schedule buffer
  - Full walker call sequence
  - Full RNG call sequence
  - 792 (?) fixture insertions

Compares:
  - Rust `build_eng_second_schedule(2001)` vs captured sched buffer
  - Diagnostic: what does "fixtures=792" mean vs the expected 552
"""
import json, hashlib
from pathlib import Path
from collections import Counter, defaultdict

TS = "20260913_175838_manual"
BASE = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")

# Load event log
records = []
with open(BASE / f"{TS}_natural.jsonl") as f:
    for line in f:
        line = line.strip()
        if line: records.append(json.loads(line))

print(f"loaded {len(records)} events")

# Extract each large payload
sched_buf = (BASE / f"{TS}_sched_buffer_0.bin").read_bytes()
clubs_final = (BASE / f"{TS}_clubs_table_0.bin").read_bytes()
clubs_before = (BASE / f"{TS}_clubs_before_perturb_0.bin").read_bytes()
clubs_after = (BASE / f"{TS}_clubs_after_perturb_0.bin").read_bytes()

print(f"schedule buffer: {len(sched_buf)} bytes  SHA={hashlib.sha256(sched_buf).hexdigest()[:16]}")
print(f"clubs_table (final): {len(clubs_final)} bytes")
print(f"clubs_before_perturb: {len(clubs_before)} bytes")
print(f"clubs_after_perturb: {len(clubs_after)} bytes")

# Compare schedule buffer to Rust reference
rust_ref = (BASE / "20260913_131323_gdi_buffer_0.bin").read_bytes()
if sched_buf == rust_ref:
    print("SCHEDULE BUFFER: byte-exact match vs prior GDI capture ✓")
else:
    print(f"SCHEDULE BUFFER: differs vs prior capture; first diff at ??")
    for i in range(min(len(sched_buf), len(rust_ref))):
        if sched_buf[i] != rust_ref[i]:
            print(f"  first diff at byte {i}: capture={sched_buf[i]:#04x} prior={rust_ref[i]:#04x}")
            break

# Extract clubs_before_perturb_deref and clubs_after_perturb_deref
before_deref = next((r["entries"] for r in records
                      if r.get("op") == "clubs_before_perturb_deref"), None)
after_deref = next((r["entries"] for r in records
                     if r.get("op") == "clubs_after_perturb_deref"), None)
final_deref = next((r["entries"] for r in records
                     if r.get("op") == "clubs_dereferenced"), None)

if before_deref:
    print(f"\nPre-perturb roster (24 clubs):")
    for e in before_deref:
        print(f"  slot {e['slot']:2d}: club_id={e.get('club_id')}  nation@0x69={e.get('nation_at_69')}  ptr={e.get('club_ptr')}")

if after_deref:
    print(f"\nPost-perturb roster (24 clubs):")
    for e in after_deref:
        print(f"  slot {e['slot']:2d}: club_id={e.get('club_id')}  nation@0x69={e.get('nation_at_69')}  ptr={e.get('club_ptr')}")

# Verify: is post-perturb a permutation of pre-perturb?
if before_deref and after_deref:
    before_ids = sorted(e.get("club_id") for e in before_deref if "club_id" in e)
    after_ids = sorted(e.get("club_id") for e in after_deref if "club_id" in e)
    if before_ids == after_ids:
        print("\nPost-perturb = permutation of pre-perturb ✓")
    else:
        print("\nPost-perturb ROSTER CHANGED (unexpected)")

# Extract walker trace
walker = next((r["calls"] for r in records if r.get("op") == "walker_full_trace"), [])
print(f"\nWalker calls: {len(walker)}")
if walker:
    print(f"  First 5:")
    for w in walker[:5]:
        print(f"    call {w['idx']:2d}: prev={w['prev_col']:2d} state_in={w['state_before']} "
              f"flag=0x{w['flag_byte']:02x} n_clubs={w['n_clubs']} n_rounds={w['n_rounds']} "
              f"-> retval={w['retval']:2d} state_out={w['state_after']}")

# Extract RNG trace
rng = next((r["calls"] for r in records if r.get("op") == "rng_full_trace"), [])
print(f"\nRNG calls: {len(rng)}")
for r in rng:
    print(f"  {r['phase']:8s}: n={r['n']:6d} -> retval={r['retval']}")

# Extract fixture inserts
inserts = next((r["inserts"] for r in records if r.get("op") == "fixture_full_trace"), [])
print(f"\nFixture inserts total: {len(inserts)}")

# Count comp_ids in inserts
cid_counts = Counter(f.get("cid") for f in inserts)
print(f"  comp_id distribution: {dict(cid_counts.most_common())}")

# Split fixtures by comp id
by_cid = defaultdict(list)
for f in inserts:
    by_cid[f.get("cid")].append(f)

# For comp id 9, count fixtures per round_within_half
if 9 in by_cid:
    eng2 = by_cid[9]
    print(f"\ncomp_id 9 fixtures: {len(eng2)}")
    rwh_counts = Counter(f.get("rwh") for f in eng2)
    print(f"  round_within_half distribution: {dict(sorted(rwh_counts.items()))}")
    print(f"  First 5 fixtures:")
    for f in eng2[:5]:
        print(f"    #{f['idx']:3d}: cid={f['cid']} home={f['home']} away={f['away']} "
              f"year={f['year']} doy={f['doy']} rwh={f['rwh']}")

# Look at driver_leave to see if fixtures came from just one driver run
driver_leaves = [r for r in records if r.get("op") == "driver_leave"]
print(f"\ndriver_leave events: {len(driver_leaves)}")
for d in driver_leaves:
    print(f"  {d}")
