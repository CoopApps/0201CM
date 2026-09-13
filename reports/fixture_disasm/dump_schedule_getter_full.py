"""Disassemble the schedule-getter at 0x0055f340 in full, plus the branch
target 0x0055ffeb.
"""
import struct
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

# Full function 0x0055f340. Continue until INT3 padding run.
off = va_to_off(0x0055f340)
end_off = off + 0x4000   # up to 16KB
blob = exe[off:end_off]
lines = []
# Emit up to the first CC-CC pair (function end padding)
prev_was_ret = False
consecutive_int3 = 0
last_ins_addr = 0
for insn in md.disasm(blob, 0x0055f340):
    lines.append(f"{insn.address:#010x}  {insn.bytes.hex():<20}  {insn.mnemonic:<8} {insn.op_str}")
    last_ins_addr = insn.address + insn.size
    if insn.mnemonic == "int3":
        consecutive_int3 += 1
        if consecutive_int3 >= 2 and prev_was_ret:
            break
    else:
        consecutive_int3 = 0
    prev_was_ret = (insn.mnemonic == "ret")

(OUT / "eng_second_schedule_getter.txt").write_text("\n".join(lines), encoding="utf-8")
print(f"wrote {OUT/'eng_second_schedule_getter.txt'} ({len(lines)} insns, ended at {last_ins_addr:#010x})")
print("\nFirst 50 lines:")
for ln in lines[:50]:
    print(ln)
print("\nLast 40 lines:")
for ln in lines[-40:]:
    print(ln)
