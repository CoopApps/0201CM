"""Disassemble the schedule-getter at 0x0055f340 (vtable slot +0x3c on
the eng_second vtable). Small function — likely just returns a pointer
into .rdata to a static schedule template blob.
"""
import struct
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

EXE = Path(r"D:\cm0102\cm0102.exe")
FN_START = 0x0055f340
FN_MAX_END = 0x0055f3a2   # next fn boundary per file_attribution

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
off = va_to_off(FN_START)
blob = exe[off:va_to_off(FN_MAX_END)]
print(f"disasm {len(blob)} bytes from {FN_START:#010x}:\n")
for insn in md.disasm(blob, FN_START):
    print(f"  {insn.address:#010x}  {insn.bytes.hex():<20}  {insn.mnemonic:<8} {insn.op_str}")
    if insn.mnemonic == "ret":
        break
