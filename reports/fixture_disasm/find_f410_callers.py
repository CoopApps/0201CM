"""Find all callers of FUN_0066f410 (slot-writer)."""
import struct
from pathlib import Path
EXE = Path(r"D:\cm0102\cm0102.exe")
IMAGE_BASE = 0x00400000
TARGETS = {
    0x0066f410: "FUN_0066f410 (slot-writer)",
}
b = EXE.read_bytes()
e = struct.unpack_from("<I", b, 0x3c)[0]; coff = e + 4
n = struct.unpack_from("<H", b, coff+2)[0]
opt = struct.unpack_from("<H", b, coff+16)[0]; off = coff + 20
so = off + opt
for i in range(n):
    base = so + i*40
    name = b[base:base+8].rstrip(b"\0").decode('latin1','replace')
    if name == ".text":
        text_ro = struct.unpack_from("<I", b, base+20)[0]
        text_va = struct.unpack_from("<I", b, base+12)[0] + IMAGE_BASE
        text_size = struct.unpack_from("<I", b, base+16)[0]
        break

found = []
i = text_ro; end = text_ro + text_size
while i < end - 5:
    if b[i] == 0xE8:
        rel = struct.unpack_from("<i", b, i+1)[0]
        call_va = text_va + (i - text_ro)
        tgt_va = call_va + 5 + rel
        if tgt_va in TARGETS:
            found.append(call_va)
    i += 1

print(f"FUN_0066f410 : {len(found)} callsites")
# Bucket into function ranges — buckets of 0x1000
from collections import Counter
buckets = Counter(c & ~0xFFF for c in found)
for bucket, count in sorted(buckets.items()):
    cs_in_bucket = [c for c in found if c & ~0xFFF == bucket]
    print(f"  {bucket:#010x} : {count:3d} callsites  (first: {hex(cs_in_bucket[0])})")

# Also check callsites from within FUN_0055f340 range
in_schedgetter = [c for c in found if 0x0055f340 <= c < 0x00560000]
print(f"\n  {len(in_schedgetter)} of them lie inside FUN_0055f340 (0x0055f340..0x00560000)")
