"""Find GDI equivalents by unique structural signatures.

Signatures used:
- comp_id write: mov byte ptr [reg + 0x50], 9   → eng_second_ctor
  (opcode: C6 40/41/42/43 50 09 for [eax/ecx/edx/ebx+0x50]=9)
  and via ecx variant: C6 41 50 09
- Callers of the schedule-installer at 0x00560520 → ctors of English divs
- Callers of pack_date at 0x00533d80 → flag-snap and round-writer
- Callers of round-writer at 0x0066ef70 → schedule-getters

Then use these anchors to triangulate FUN_00668890 (fixture generator) as
"the function called by eng_second_ctor after FUN_00560520 returns".
"""
import struct, json
from pathlib import Path

GDI = Path(r"D:\cm0102\cm0102_GDI.exe")
IMAGE_BASE = 0x00400000

def parse_pe(b):
    e = struct.unpack_from("<I", b, 0x3c)[0]; coff = e + 4
    n = struct.unpack_from("<H", b, coff+2)[0]
    opt = struct.unpack_from("<H", b, coff+16)[0]; off = coff + 20
    ib = struct.unpack_from("<I", b, off+28)[0]; so = off + opt
    secs = {}
    for i in range(n):
        base = so + i*40
        name = b[base:base+8].rstrip(b"\0").decode('latin1','replace')
        secs[name] = dict(va=struct.unpack_from("<I", b, base+12)[0],
                          ro=struct.unpack_from("<I", b, base+20)[0],
                          rsz=struct.unpack_from("<I", b, base+16)[0])
    return ib, secs

b = GDI.read_bytes()
ib, secs = parse_pe(b)
text = secs[".text"]
text_va = ib + text["va"]
text_ro = text["ro"]
text_sz = text["rsz"]

def va_of(off):
    return text_va + (off - text_ro)

def find_all(pat):
    matches = []
    i = text_ro
    while True:
        i = b.find(pat, i, text_ro + text_sz)
        if i < 0: break
        matches.append(va_of(i))
        i += 1
    return matches

# 1) Find comp_id=9 write patterns
# c6 41 50 09  = mov byte ptr [ecx+0x50], 9
# c6 40 50 09  = mov byte ptr [eax+0x50], 9  (rare)
patterns = {
    "mov [ecx+0x50], 9": bytes([0xC6, 0x41, 0x50, 0x09]),
    "mov [eax+0x50], 9": bytes([0xC6, 0x40, 0x50, 0x09]),
    "mov [ebx+0x50], 9": bytes([0xC6, 0x43, 0x50, 0x09]),
    "mov [edi+0x50], 9": bytes([0xC6, 0x47, 0x50, 0x09]),
    "mov [esi+0x50], 9": bytes([0xC6, 0x46, 0x50, 0x09]),
}
print("=== comp_id = 9 write patterns in GDI .text ===")
for name, pat in patterns.items():
    ms = find_all(pat)
    if ms:
        print(f"  {name}: {len(ms)} sites  first 5: {[hex(m) for m in ms[:5]]}")

# 2) Callers of schedule-installer 0x00560520
def callers_of(target_va):
    """Find E8-rel32 callers of target_va within .text."""
    results = []
    i = text_ro
    end = text_ro + text_sz
    while i < end - 5:
        if b[i] == 0xE8:
            rel = struct.unpack_from("<i", b, i+1)[0]
            call_va = va_of(i)
            tgt = call_va + 5 + rel
            if tgt == target_va:
                results.append(call_va)
        i += 1
    return results

print("\n=== callers of GDI schedule-installer 0x00560520 (= English division ctors) ===")
for c in callers_of(0x00560520):
    print(f"  called from {c:#010x}")

print("\n=== callers of GDI pack_date 0x00533d80 (or 0x00533f40) ===")
for tgt in [0x00533d80, 0x00533f40]:
    cs = callers_of(tgt)
    if cs:
        print(f"  target {tgt:#010x}: {len(cs)} callers  first 8: {[hex(x) for x in cs[:8]]}")

print("\n=== callers of GDI round-writer 0x0066ef70 (= schedule-getters) ===")
cs = callers_of(0x0066ef70)
print(f"  {len(cs)} total  first 5: {[hex(x) for x in cs[:5]]}")

print("\n=== callers of GDI slot-writer 0x0066efd0 ===")
cs = callers_of(0x0066efd0)
print(f"  {len(cs)} total")

print("\n=== callers of GDI matrix base seeder 0x00669340 (candidate for driver) ===")
cs = callers_of(0x00669340)
print(f"  {len(cs)} total  first 10: {[hex(x) for x in cs[:10]]}")
