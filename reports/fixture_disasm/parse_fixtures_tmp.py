"""Parse the cm0102-gdi.exe fixtures.<year>.tmp file to extract real
GDI-produced fixture records.

Format (from agent 3's fix_man.cpp reconstruction):
  offset 0: i32 count
  offset 4: count × 79-byte TFixture records

TFixture layout (bytes we care about here):
  +0x00 i32   competition_id (first int of Comp id-record; sometimes an index)
  +0x04 i32   referee slot
  +0x08 i32   result / match id
  +0x0c i32   home_club_id
  +0x10 i32   away_club_id
  +0x14 u32   Comp* (pointer, garbage on-disk)
  +0x18 u32   Person* referee (pointer)
  +0x1c u32   Club* home
  +0x20 u32   Club* away
  +0x24 u16   venue short
  +0x26 u16   venue short
  +0x28 i16   year
  +0x2a i16   doy
  +0x2c i16   season base year (comp+0x40)
  +0x2e i16   unused
  +0x30 u16   0
  +0x32 u16   comp+0x3a (cup round / week)
  +0x34 i16   round_within_half
  +0x36 u16   comp+0xdb
  +0x38 u16   comp+0xab flag
  +0x3a u8    comp+0xc4
  +0x3b u8    0xff sentinel
  +0x3c u8    weekday code
  +0x3d u8    home has-no-manager
  +0x3e u8    away has-no-manager
  +0x3f u8    weekday (0..6)
  ...
"""
import struct, json, sys, datetime
from pathlib import Path
from collections import Counter, defaultdict

TMP = Path(r"D:\cm0102\fixtures_2002.tmp")
b = TMP.read_bytes()
count = struct.unpack_from("<I", b, 0)[0]
print(f"file: {TMP.name}  {len(b)} bytes  header count={count}  "
      f"record math: {(len(b) - 4) // 79} records = {(len(b) - 4) % 79} rem")

# Sanity: 4 + count*79 should equal len(b)
assert 4 + count * 79 == len(b), f"size mismatch: {4 + count * 79} vs {len(b)}"

def parse(rec):
    return dict(
        comp_id = struct.unpack_from("<i", rec, 0x00)[0],
        referee_idx = struct.unpack_from("<i", rec, 0x04)[0],
        match_id = struct.unpack_from("<i", rec, 0x08)[0],
        home_id = struct.unpack_from("<i", rec, 0x0c)[0],
        away_id = struct.unpack_from("<i", rec, 0x10)[0],
        year = struct.unpack_from("<h", rec, 0x28)[0],
        doy = struct.unpack_from("<h", rec, 0x2a)[0],
        base_year = struct.unpack_from("<h", rec, 0x2c)[0],
        cup_round = struct.unpack_from("<H", rec, 0x32)[0],
        round_within_half = struct.unpack_from("<h", rec, 0x34)[0],
        weekday_flag = rec[0x3c],
        home_no_mgr = rec[0x3d],
        away_no_mgr = rec[0x3e],
        weekday = rec[0x3f],
    )

fixtures = []
for i in range(count):
    off = 4 + i * 79
    fixtures.append(parse(b[off:off + 79]))

# Cluster by comp_id — how many unique comps?
comp_counts = Counter(f["comp_id"] for f in fixtures)
print(f"\ncomp_id distribution (top 20):")
for cid, cnt in comp_counts.most_common(20):
    print(f"  comp {cid:>6}: {cnt} fixtures")

# Focus on comp_id == 9 (English Second Division candidate)
print(f"\n=== fixtures with comp_id == 9 ===")
eng2 = [f for f in fixtures if f["comp_id"] == 9]
print(f"count: {len(eng2)}")

if eng2:
    # Group by round_within_half, then order by doy
    rounds = defaultdict(list)
    for f in eng2:
        rounds[f["round_within_half"]].append(f)
    print(f"unique rounds: {len(rounds)}, indices: {sorted(rounds.keys())}")
    print(f"\nFirst 5 fixtures:")
    for f in eng2[:5]:
        y, d = f["year"], f["doy"]
        date = "?"
        try:
            gd = datetime.date(y, 1, 1) + datetime.timedelta(days=d - 1)
            date = gd.strftime("%a %d %b %Y")
        except Exception:
            pass
        print(f"  home={f['home_id']} away={f['away_id']} year={y} doy={d} "
              f"round_wh={f['round_within_half']} weekday={f['weekday']} date={date}")

# Save the parsed fixture list to json for Rust to consume
out = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime\gdi_fixtures_2002.json")
out.write_text(json.dumps(fixtures, indent=1))
print(f"\nSaved {out}")

# Also filter to eng_second only
out2 = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime\gdi_eng_second_fixtures_2002.json")
out2.write_text(json.dumps(eng2, indent=1))
print(f"Saved {out2}")
