"""Locate the true end of the schedule-getter at 0x0055f340 by scanning for
the first RET followed by INT3 padding OR by watching stack balance.
"""
import struct
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

EXE = Path(r"D:\cm0102\cm0102.exe")

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

md = Cs(CS_ARCH_X86, CS_MODE_32)
off = va_to_off(0x0055f340)
blob = exe[off:off + 0x4000]

# Track first RET followed by CC padding
prev_ret = False
consecutive_int3 = 0
last_end = None
for insn in md.disasm(blob, 0x0055f340):
    if insn.mnemonic == "int3":
        if prev_ret:
            consecutive_int3 += 1
            if consecutive_int3 >= 1:
                last_end = insn.address
                print(f"Function end candidate at {last_end:#010x} (RET + INT3 padding)")
                break
    elif insn.mnemonic == "ret":
        # candidate end - but might be an intermediate ret inside function
        prev_ret = True
        consecutive_int3 = 0
        candidate_ret = insn.address + insn.size
    else:
        prev_ret = False
        consecutive_int3 = 0

# Alternative: look at file_attribution's next fn after 0x0055f340
import json
attr = json.load(open(r"D:/cm0102-carve/ghidra_out/cm0102.exe/file_attribution.json"))
eng_second_fns = [int(a, 16) for a in attr.get("eng_second.cpp", [])]
after = [f for f in eng_second_fns if f > 0x0055f340]
if after:
    print(f"\nNext function in eng_second.cpp after 0x0055f340: {after[0]:#010x}")
