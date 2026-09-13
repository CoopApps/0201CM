"""Raw byte scan for CALL rel32 to specific targets in the gap.

`call rel32` opcode = E8 followed by 4-byte relative offset (little endian).
Relative offset = target_va - (call_insn_va + 5).

We scan the entire gap file for `E8` followed by the exact bytes that would
encode a call to 0x00670350 from each possible position.

Also scan for `call rel32` to *any* target inside .text to get a call count
(sanity check on capstone).
"""
import struct
from pathlib import Path

GAP_PATH = Path(r"D:\cm0102-rs\reports\fixture_disasm\raw_gap.bin")
OUT      = Path(r"D:\cm0102-rs\reports\fixture_disasm")
GAP_START_VA = 0x0081a118
TARGET_FUN_00670350 = 0x00670350

blob = GAP_PATH.read_bytes()
print(f"gap {len(blob)} bytes starting VA {GAP_START_VA:#010x}")

# Every offset where blob[i] == 0xE8 is a possible `call rel32`.
# Compute the target it would encode and record it.
hits_to_target = []
call_targets_in_text = 0
target_hist = {}   # target_va -> count
for i in range(len(blob) - 4):
    if blob[i] == 0xE8:
        rel = struct.unpack_from("<i", blob, i + 1)[0]
        call_va = GAP_START_VA + i
        target_va = (call_va + 5 + rel) & 0xffffffff
        if 0x00401000 <= target_va < 0x00954000:   # inside .text/.rdata roughly
            call_targets_in_text += 1
            target_hist[target_va] = target_hist.get(target_va, 0) + 1
            if target_va == TARGET_FUN_00670350:
                hits_to_target.append((call_va, i))

print(f"Total plausible `call rel32` inside gap: {call_targets_in_text}")
print(f"Calls to FUN_00670350 (0x{TARGET_FUN_00670350:08x}): {len(hits_to_target)}")
for va, off in hits_to_target[:60]:
    print(f"  call at VA {va:#010x} (gap+{off:#06x})")

# Top-50 most-called targets in the gap
top = sorted(target_hist.items(), key=lambda kv: -kv[1])[:20]
print("\nTop-20 most-called targets:")
for tgt, n in top:
    print(f"  {tgt:#010x}  x{n}")

# Write full call table
with open(OUT / "call_scan.txt", "w") as f:
    f.write(f"# raw E8 rel32 scan over gap 0x{GAP_START_VA:08x}..0x{GAP_START_VA + len(blob):08x}\n")
    f.write(f"# {call_targets_in_text} calls total\n")
    f.write(f"# Calls to FUN_00670350: {len(hits_to_target)}\n\n")
    for va, off in hits_to_target:
        f.write(f"call FUN_00670350 at VA {va:#010x} (gap+{off:#06x})\n")
    f.write("\n# All targets by frequency\n")
    for tgt, n in sorted(target_hist.items(), key=lambda kv: -kv[1]):
        f.write(f"{tgt:#010x}  x{n}\n")
print(f"\nwrote {OUT/'call_scan.txt'}")
