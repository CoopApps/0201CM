"""Find containing function VAs for the anchor sites in cm0102_GDI.exe."""
import json
from pathlib import Path

FN = Path(r"D:\cm0102-carve\ghidra_out\cm0102_GDI.exe\functions.json")
data = json.loads(FN.read_text())
# functions.json is typically {"functions": [{"entry": 0x..., "size": ..., "name": ...}, ...]}
# or a plain list. Let me probe.
if isinstance(data, dict):
    if "functions" in data:
        fns = data["functions"]
    else:
        fns = list(data.values())
else:
    fns = data
print(f"Loaded {len(fns)} functions")
print(f"Sample entry: {fns[0]}")

# Normalise entries
records = []
for f in fns:
    if isinstance(f, dict):
        e = f.get("entry") or f.get("va") or f.get("address")
        s = f.get("size") or 0
        name = f.get("name") or ""
    else:
        continue
    if isinstance(e, str):
        e = int(e, 16) if e.startswith("0x") else int(e)
    if isinstance(s, str):
        s = int(s, 16) if s.startswith("0x") else int(s)
    records.append((e, s, name))
records.sort()
print(f"Normalised {len(records)} function records")

def find_containing(addr):
    """Binary search: find function containing addr."""
    lo, hi = 0, len(records) - 1
    while lo <= hi:
        mid = (lo + hi) // 2
        e, s, n = records[mid]
        if addr < e:
            hi = mid - 1
        elif addr >= e + s:
            lo = mid + 1
        else:
            return records[mid]
    # if no exact match, return the largest entry <= addr
    for i in range(len(records)-1, -1, -1):
        if records[i][0] <= addr:
            return records[i]
    return None

ANCHORS = [
    (0x0055f2c1, "call site of GDI schedule-installer inside eng_second_ctor"),
    (0x00668570, "call site of GDI matrix-seeder inside round-robin driver"),
    (0x00554891, "one of the 10 comp_id=9 write sites"),
    (0x005560e9, "another comp_id=9 site"),
    (0x00557201, "another comp_id=9 site"),
    (0x005579c1, "another comp_id=9 site"),
    (0x00558eb9, "another comp_id=9 site"),
    (0x0055f540, "GDI schedule-getter equivalent (from 48-byte match)"),
]
print("\n=== Containing functions ===")
for addr, desc in ANCHORS:
    r = find_containing(addr)
    if r:
        e, s, n = r
        end = e + s
        offset_in_fn = addr - e
        print(f"  {addr:#010x}: fn entry {e:#010x} (size {s:#x}, end {end:#010x}) name={n!r}  offset_in_fn={offset_in_fn:#x}  ({desc})")
    else:
        print(f"  {addr:#010x}: no containing function found  ({desc})")

# Also find nearest functions around key VAs
print("\n=== Functions in eng_second area 0x0055f000 .. 0x00561000 ===")
for r in records:
    e, s, n = r
    if 0x0055f000 <= e < 0x00561000:
        print(f"  {e:#010x}  size {s:#5x}  {n!r}")
    if e >= 0x00561200:
        break

print("\n=== Functions in fixture-driver area 0x00668000 .. 0x00670000 ===")
for r in records:
    e, s, n = r
    if 0x00668000 <= e < 0x00670000:
        print(f"  {e:#010x}  size {s:#5x}  {n!r}")
    if e >= 0x00670200:
        break
