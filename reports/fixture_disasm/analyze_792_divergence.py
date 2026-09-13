"""Exhaustive analysis of the 792 fixture insertion records from the
natural cm0102-gdi trace at 20260913_175838. Answers:

  1. How many unique logical fixtures? (by comp_id/home/away/year/doy/rwh)
  2. What differentiates the 240 excess inserts?
  3. Cross-check the extra 240 against fixtures_2002.tmp.
  4. Round-numbering semantics.
  5. Byte 474 mutation source.

The Frida hook did NOT capture the TFixture* pointer nor caller
return address — those will be added to the next capture if the
logical-key analysis leaves the pointer-uniqueness question open.
"""
import json
from pathlib import Path
from collections import Counter, defaultdict

BASE = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")
TS = "20260913_175838_manual"

# Load event log
records = []
with open(BASE / f"{TS}_natural.jsonl") as f:
    for line in f:
        line = line.strip()
        if line: records.append(json.loads(line))

inserts = next((r["inserts"] for r in records if r.get("op") == "fixture_full_trace"), [])
print(f"total insert calls: {len(inserts)}")

# --- 1. Logical uniqueness ------------------------------------------------
def key(f):
    """Canonical logical key — comp_id + ordered pair + date + round."""
    return (f["cid"], f["home"], f["away"], f["year"], f["doy"], f["rwh"])

key_counts = Counter(key(f) for f in inserts)
uniq_ordered = len(key_counts)
duped = [(k, c) for k, c in key_counts.items() if c > 1]
print(f"\n=== 1. Logical uniqueness ===")
print(f"unique (cid, home, away, year, doy, rwh): {uniq_ordered}")
print(f"total insert calls: {len(inserts)}")
print(f"logical duplicates: {len(duped)}")
if duped:
    print(f"  first 5 duplicated tuples: {duped[:5]}")

# By UNORDERED pair (a,b) same as (b,a)
def undir_key(f):
    a, b = sorted([f["home"], f["away"]])
    return (f["cid"], a, b, f["year"], f["doy"], f["rwh"])

undir_counts = Counter(undir_key(f) for f in inserts)
uniq_undir = len(undir_counts)
print(f"unique (cid, {{home,away}}, year, doy, rwh) [H/A collapsed]: {uniq_undir}")

# --- 2. What's in the extra 240? ------------------------------------------
first_half = [f for f in inserts if f["rwh"] < 26]
second_half = [f for f in inserts if f["rwh"] >= 26]
print(f"\n=== 2. First vs second half ===")
print(f"first half (rwh 0..25):  {len(first_half)} inserts  = {len(first_half)/26:.1f}/round")
print(f"second half (rwh 26..45): {len(second_half)} inserts  = {len(second_half)/20:.1f}/round")

# Within each second-half round, are the 24 inserts 12 + 12 mirror
# pairings, or 24 different fixtures, or 12 fixtures + 12 duplicates?
print(f"\n=== Per-round analysis (round 26) ===")
rd26 = [f for f in inserts if f["rwh"] == 26]
print(f"insertion count: {len(rd26)}")
# Ordered pairs
ordered = set((f["home"], f["away"]) for f in rd26)
print(f"unique ordered (home, away) pairs: {len(ordered)}")
# Unordered pairs
unordered = set(frozenset([f["home"], f["away"]]) for f in rd26)
print(f"unique unordered {{home, away}} pairs: {len(unordered)}")
# List every insert in round 26
print(f"\nround 26 fixtures (all {len(rd26)}, in insertion order):")
for i, f in enumerate(rd26):
    print(f"  {i:2d}: home={f['home']:4d} away={f['away']:4d} doy={f['doy']:3d} idx={f['idx']:3d}")

# --- Round 27 same analysis ---
print(f"\n=== Per-round analysis (round 27) ===")
rd27 = [f for f in inserts if f["rwh"] == 27]
ordered27 = set((f["home"], f["away"]) for f in rd27)
unordered27 = set(frozenset([f["home"], f["away"]]) for f in rd27)
print(f"insertion count: {len(rd27)}  unique ordered: {len(ordered27)}  unique unordered: {len(unordered27)}")

# --- Round 45 same analysis ---
print(f"\n=== Per-round analysis (round 45) ===")
rd45 = [f for f in inserts if f["rwh"] == 45]
ordered45 = set((f["home"], f["away"]) for f in rd45)
unordered45 = set(frozenset([f["home"], f["away"]]) for f in rd45)
print(f"insertion count: {len(rd45)}  unique ordered: {len(ordered45)}  unique unordered: {len(unordered45)}")

# --- 3. Compare vs fixtures_2002.tmp -------------------------------------
print(f"\n=== 3. Cross-check vs fixtures_2002.tmp ===")
tmp_path = BASE / "gdi_eng_second_fixtures_2002.json"
if tmp_path.exists():
    tmp_fx = json.loads(tmp_path.read_text())
    print(f"loaded {len(tmp_fx)} fixtures from fixtures_2002.tmp")
    # Canonical key for comparison
    def tmp_key(t):
        return (t["comp_id"], t["home_id"], t["away_id"], t["year"], t["doy"], t["round_within_half"])
    tmp_set = set(tmp_key(t) for t in tmp_fx)
    trace_set = set(key(f) for f in inserts)
    print(f"unique keys in tmp: {len(tmp_set)}")
    print(f"unique keys in trace: {len(trace_set)}")
    common = tmp_set & trace_set
    print(f"intersection (exact match): {len(common)}")
    tmp_only = tmp_set - trace_set
    trace_only = trace_set - tmp_set
    print(f"in tmp only:   {len(tmp_only)}")
    print(f"in trace only: {len(trace_only)}")

    # Are the 240 extras (second-half doublings) also in tmp?
    if second_half:
        # In second half, each rwh has 24 inserts. The FIRST 12 and
        # LAST 12 might be different sets. Split them.
        by_rwh = defaultdict(list)
        for f in inserts:
            if f["rwh"] >= 26:
                by_rwh[f["rwh"]].append(f)
        # First 12 per rwh
        first12 = []
        last12 = []
        for rwh in sorted(by_rwh):
            fs = by_rwh[rwh]
            first12.extend(fs[:12])
            last12.extend(fs[12:])
        print(f"\nsecond-half split: first-12/round = {len(first12)}, last-12/round = {len(last12)}")
        f12_set = set(key(f) for f in first12)
        l12_set = set(key(f) for f in last12)
        # Are the last-12 identical to the first-12 (dup inserts) or different?
        f12_l12_overlap = f12_set & l12_set
        print(f"  first-12/second-half UNIQUE keys: {len(f12_set)}")
        print(f"  last-12/second-half UNIQUE keys:  {len(l12_set)}")
        print(f"  keys in BOTH first-12 & last-12: {len(f12_l12_overlap)}")
        # Compare to tmp
        print(f"  first-12 keys in tmp: {len(f12_set & tmp_set)}")
        print(f"  last-12 keys in tmp:  {len(l12_set & tmp_set)}")
        # Sample fixtures
        print(f"\n  Round 26 first-12 vs last-12 side-by-side:")
        for i in range(12):
            f = by_rwh[26][i]
            l = by_rwh[26][i + 12]
            print(f"    [{i:2d}] first: h={f['home']:4d} a={f['away']:4d} doy={f['doy']:3d}  "
                  f"| last: h={l['home']:4d} a={l['away']:4d} doy={l['doy']:3d}")
else:
    print("no tmp reference to cross-check")

# --- 4. Round numbering semantics ----------------------------------------
print(f"\n=== 4. Round numbering ===")
# Group by doy to see how insertion rwh maps to actual date order
by_doy = defaultdict(list)
for f in inserts:
    by_doy[(f["year"], f["doy"])].append(f)
print(f"unique (year, doy) values: {len(by_doy)}")
print(f"rwh distribution for each date:")
sorted_dates = sorted(by_doy.keys())
for (y, d) in sorted_dates[:10] + sorted_dates[-5:]:
    fs = by_doy[(y, d)]
    rwh_dist = Counter(f["rwh"] for f in fs)
    print(f"  year={y} doy={d:3d}  count={len(fs):3d}  rwh: {dict(rwh_dist)}")
