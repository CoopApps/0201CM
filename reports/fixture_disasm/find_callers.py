"""Find every callsite of FUN_00668890 and FUN_0066f280 in cm0102.exe .text."""
import struct
from pathlib import Path

EXE = Path(r"D:\cm0102\cm0102.exe")
IMAGE_BASE = 0x00400000
TEXT_VA = 0x00401000
TEXT_END = 0x00955000  # rdata start
TARGETS = {
    0x00668890: "FUN_00668890 (round-robin driver)",
    0x0066f280: "FUN_0066f280 (walker)",
    0x0055f340: "FUN_0055f340 (schedule-getter)",
    0x0066f3b0: "FUN_0066f3b0 (round writer)",
}

b = EXE.read_bytes()
# Locate .text file offset via PE
e = struct.unpack_from("<I", b, 0x3c)[0]; coff = e + 4
n = struct.unpack_from("<H", b, coff+2)[0]
opt = struct.unpack_from("<H", b, coff+16)[0]; off = coff + 20
so = off + opt
text_ro = None
for i in range(n):
    base = so + i*40
    name = b[base:base+8].rstrip(b"\0").decode('latin1','replace')
    if name == ".text":
        text_ro = struct.unpack_from("<I", b, base+20)[0]
        text_va = struct.unpack_from("<I", b, base+12)[0] + IMAGE_BASE
        text_size = struct.unpack_from("<I", b, base+16)[0]
        break

# Scan for E8 xx xx xx xx (call rel32)
found = {tgt: [] for tgt in TARGETS}
i = text_ro
end = text_ro + text_size
while i < end - 5:
    if b[i] == 0xE8:
        rel = struct.unpack_from("<i", b, i+1)[0]
        call_va = text_va + (i - text_ro)
        tgt_va = call_va + 5 + rel
        if tgt_va in TARGETS:
            found[tgt_va].append(call_va)
    i += 1

for tgt, name in TARGETS.items():
    calls = found[tgt]
    print(f"\n{name} @ {tgt:#010x} — {len(calls)} call sites")
    # bucket by 0x1000 to see clusters
    from collections import Counter
    buckets = Counter(c & ~0xFFF for c in calls)
    for bucket, count in sorted(buckets.items())[:20]:
        print(f"  {bucket:#010x}xxx: {count}  first: {[hex(c) for c in calls if c & ~0xFFF == bucket][:3]}")
