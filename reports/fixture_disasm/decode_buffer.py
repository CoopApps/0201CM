"""Decode the 2990-byte schedule buffer captured live from cm0102.exe."""
import struct, json, sys
from pathlib import Path

RUNTIME = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime")
bufs = sorted(RUNTIME.glob("*_direct_buffer_*.bin"))
if not bufs:
    sys.exit("no buffer files")
buf = bufs[-1].read_bytes()
print(f"buffer: {bufs[-1].name}  {len(buf)} bytes  (expected 2990 = 46*65)")

DAYS = "Sun Mon Tue Wed Thu Fri Sat".split()
# Base: 12 Aug 2001 is a Sunday (verified). day-of-year 0-indexed = 223.
# In 2001 (non-leap), 12 Aug is Sun -> Jan 1 2001 was a Monday. So
# weekday(day_of_year_0_idx, year) computed from a known anchor.
import datetime
def weekday(doy0, year):
    return (datetime.date(year, 1, 1) + datetime.timedelta(days=doy0)).strftime("%a %d %b %Y")

print("\nround | doy | yr_off | type | +0x05..+0x3c (hex) | +0x3d..+0x40 (u32) | date")
print("-" * 120)
for r in range(46):
    off = r * 0x41
    rec = buf[off:off + 0x41]
    doy = struct.unpack_from("<h", rec, 0)[0]
    yr_off = struct.unpack_from("<h", rec, 2)[0]
    typ = rec[4]
    mid = rec[5:0x3d].hex()
    tail = struct.unpack_from("<i", rec, 0x3d)[0]
    year = 2001 + yr_off
    try:
        date = weekday(doy, year)
    except Exception:
        date = f"invalid doy={doy} yr={year}"
    # Only show non-zero mid bytes summary
    mid_nz = sum(1 for c in rec[5:0x3d] if c != 0)
    print(f"  {r:2d}  | {doy:4d} |  {yr_off:2d}   |  {typ:2d}  | ({mid_nz} nz bytes) | {tail:11d} | {date}")

# Also dump non-zero mid bytes for a few rounds
print("\n=== non-zero bytes in +0x05..+0x3c for each round ===")
for r in range(46):
    off = r * 0x41
    rec = buf[off + 5:off + 0x3d]
    nz = [(i + 5, b) for i, b in enumerate(rec) if b != 0]
    if nz:
        print(f"  round {r:2d}: {nz}")
