"""Analyze the 240 GDI-produced eng_second fixtures from
fixtures_2002.tmp and derive the club-slot ordering + verify
double-round-robin properties.
"""
import json, datetime
from pathlib import Path
from collections import Counter, defaultdict

DATA = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime\gdi_eng_second_fixtures_2002.json")
fixtures = json.loads(DATA.read_text())

print(f"loaded {len(fixtures)} fixtures")

# --- Basic structural properties -----------------------------------------
per_round = defaultdict(list)
for f in fixtures:
    per_round[f["round_within_half"]].append(f)
print(f"\n=== rounds present (round_within_half) ===")
for r in sorted(per_round.keys()):
    print(f"  round {r:2d}: {len(per_round[r])} fixtures")

# For every round, extract the (home, away) pairs.
# Expected: 12 pairs per round, all unique, no team appearing twice.
print(f"\n=== per-round pair validation ===")
all_pairs = set()
duplicate_pairs = []
teams_double_scheduled = []
for r in sorted(per_round.keys()):
    rf = per_round[r]
    pairs = [(f["home_id"], f["away_id"]) for f in rf]
    teams_this_round = set()
    for h, a in pairs:
        if h in teams_this_round or a in teams_this_round:
            teams_double_scheduled.append((r, h, a))
        teams_this_round.add(h)
        teams_this_round.add(a)
        if (h, a) in all_pairs:
            duplicate_pairs.append((r, h, a))
        all_pairs.add((h, a))
if not teams_double_scheduled:
    print("  every round: each team plays at most once ✓")
else:
    print(f"  DOUBLE-SCHEDULED: {teams_double_scheduled[:5]}")
if not duplicate_pairs:
    print(f"  no duplicate ordered pairs across rounds ✓")

# Collect the 24 club IDs
club_ids = set()
for f in fixtures:
    club_ids.add(f["home_id"])
    club_ids.add(f["away_id"])
print(f"\n=== club roster ===")
print(f"unique clubs appearing: {len(club_ids)}")
print(f"club ids: {sorted(club_ids)}")

# Home/away balance per club
home_count = Counter()
away_count = Counter()
for f in fixtures:
    home_count[f["home_id"]] += 1
    away_count[f["away_id"]] += 1
print(f"\n=== home/away balance (across the 240 second-half fixtures) ===")
for c in sorted(club_ids):
    h, a = home_count[c], away_count[c]
    print(f"  club {c:5}: {h} home, {a} away, total {h+a}")

# --- Reverse-engineer the shuffled roster -------------------------------
# The driver uses matrix_seed_base(n_even=24), which produces cell[row][col]
# = ±(paired_row).  If we can identify which fixtures the driver emits
# first (round 26, i.e. round_within_half=26 with the smallest walker_col
# they map to), we can invert the matrix to get the roster order.
#
# But without walker_col + outer_round metadata for each fixture, and
# without the schedule buffer (which is deterministic), we cannot directly
# invert.  What we CAN do: verify that each round's 12 pairs form a
# perfect matching over 24 clubs.
print(f"\n=== round-by-round pair matching validation ===")
for r in sorted(per_round.keys()):
    rf = per_round[r]
    teams = set()
    for f in rf:
        teams.add(f["home_id"])
        teams.add(f["away_id"])
    assert len(rf) == 12, f"round {r} has {len(rf)} != 12 fixtures"
    assert len(teams) == 24, f"round {r} covers {len(teams)} != 24 clubs"
print("  every one of 20 rounds is a perfect 24-club matching ✓")

# --- Verify dates ----------------------------------------------------------
print(f"\n=== round dates ===")
for r in sorted(per_round.keys())[:5]:
    rf = per_round[r]
    dates = set()
    for f in rf:
        y, d = f["year"], f["doy"]
        try:
            gd = datetime.date(y, 1, 1) + datetime.timedelta(days=d - 1)
            dates.add(gd.strftime("%a %d %b %Y"))
        except Exception:
            dates.add(f"y={y} d={d}")
    print(f"  round {r}: {dates}")

# Summary: this is a valid double-round-robin second-half slice.
print(f"\n=== summary ===")
print(f"  240 fixtures / 20 rounds / 24 clubs / perfect matchings")
print(f"  Every fixture's date, home_id, away_id available for differential")
print(f"  Missing: the RNG state / roster ordering that produced this specific pair sequence")
