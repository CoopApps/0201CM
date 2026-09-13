"""Parse the schedule-getter at 0x0055f340 and extract each `call 0x66f3b0`
with the arguments pushed just before it. Hypothesis: each such call fills
one round record in the malloc'd 2990-byte buffer, so we should see 46
calls matching the round count for English Second Division.

Also count `call 0x533b50` (date encoder) invocations. If those are one
per round, they encode (day, month, year_off) into a packed short.
"""
import struct, re
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

EXE = Path(r"D:\cm0102\cm0102.exe")
OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm")

def parse_pe(b):
    e = struct.unpack_from("<I", b, 0x3c)[0]; coff = e + 4
    n_sec = struct.unpack_from("<H", b, coff+2)[0]
    opt = struct.unpack_from("<H", b, coff+16)[0]; off = coff + 20
    ib = struct.unpack_from("<I", b, off+28)[0]; so = off + opt
    secs = []
    for i in range(n_sec):
        base = so + i*40
        secs.append(dict(
            name=b[base:base+8].rstrip(b"\0").decode("latin1","replace"),
            va=struct.unpack_from("<I",b,base+12)[0],
            ro=struct.unpack_from("<I",b,base+20)[0],
            rsz=struct.unpack_from("<I",b,base+16)[0],
        ))
    return ib, secs

exe = EXE.read_bytes()
ib, secs = parse_pe(exe)
def va_to_off(va):
    for s in secs:
        rva = va - ib
        if s["va"] <= rva < s["va"] + s["rsz"]:
            return s["ro"] + (rva - s["va"])
    return None

md = Cs(CS_ARCH_X86, CS_MODE_32); md.detail = True
off = va_to_off(0x0055f340)
blob = exe[off : off + 0x4000]   # 16KB

# First pass: linear disasm and collect insn list up to end of function
insns = []
for insn in md.disasm(blob, 0x0055f340):
    insns.append(insn)
    # A rough function end: first ret followed by any 0xCC padding
    # But schedule-getter has no early ret so keep going.

# Also parse the whole function - schedule-getter apparently ends around 0x005633a0 based on
# earlier output.
print(f"Parsed {len(insns)} instructions")

# Find all sites where target = 0x66f3b0, 0x66f280, or 0x533b50 (candidate helpers)
targets = {0x0066f3b0: "FUN_0066f3b0", 0x00533b50: "FUN_00533b50 (date encoder)"}
sites = {addr: [] for addr in targets}

for i, insn in enumerate(insns):
    if insn.mnemonic == "call":
        try:
            tgt = int(insn.op_str, 16)
            if tgt in targets:
                sites[tgt].append(i)
        except (ValueError, TypeError):
            pass

for addr, name in targets.items():
    print(f"{name}: {len(sites[addr])} call sites")

# For each call to 0x66f3b0, extract the preceding pushes (going back until we
# see a non-push instruction). Argument order for a cdecl call: first push =
# LAST arg, last push = FIRST arg. But this looks like this-call:
#   push ...; push ...; ...; mov ecx, X (rare); call ...
# We'll dump the raw push sequence in reverse (first-arg-first).
def extract_pushes_for_call(call_idx):
    pushes = []
    i = call_idx - 1
    max_walkback = 25
    while i >= 0 and max_walkback > 0:
        ins = insns[i]
        if ins.mnemonic == "push":
            pushes.append((ins.address, ins.op_str))
            i -= 1
            max_walkback -= 1
            continue
        # allow interspersed movs/leas/xors
        if ins.mnemonic in ("mov", "lea", "xor", "movzx", "movsx", "and"):
            i -= 1
            max_walkback -= 1
            continue
        break
    # first push found is closest to call = LAST arg; reverse to give first-arg first
    pushes.reverse()
    return pushes

# Dump the first 20 calls
print("\n\n=== First 20 CALL 0x66f3b0 with preceding pushes ===\n")
for k, idx in enumerate(sites[0x0066f3b0][:20]):
    call_addr = insns[idx].address
    ps = extract_pushes_for_call(idx)
    print(f"\ncall #{k:2d} at {call_addr:#010x}  ({len(ps)} pushes)")
    for pa, po in ps:
        print(f"    {pa:#010x}  push {po}")

# Full dump to file
with open(OUT / "round_record_calls.txt", "w") as f:
    f.write(f"# {len(sites[0x0066f3b0])} calls to 0x66f3b0 in schedule-getter\n\n")
    for k, idx in enumerate(sites[0x0066f3b0]):
        call_addr = insns[idx].address
        ps = extract_pushes_for_call(idx)
        f.write(f"call #{k:2d} at {call_addr:#010x}  ({len(ps)} pushes)\n")
        for pa, po in ps:
            f.write(f"    {pa:#010x}  push {po}\n")
        f.write("\n")
print(f"\nwrote {OUT/'round_record_calls.txt'}")
