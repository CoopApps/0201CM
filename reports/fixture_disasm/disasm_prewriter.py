"""Disassemble FUN_0055f340 body from entry to first FUN_0066f3b0 call
(0x0055f340 .. 0x0055f3db) to find who initialises the buffer with 0xFF."""
import struct
from pathlib import Path
try:
    from capstone import Cs, CS_ARCH_X86, CS_MODE_32
except ImportError:
    raise SystemExit("pip install capstone")

EXE = Path(r"D:\cm0102\cm0102.exe")
IMAGE_BASE = 0x00400000
START_VA = 0x0055f340
END_VA   = 0x0055f440  # a bit past the first writer call to be safe

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
        break

start_off = text_ro + (START_VA - text_va)
end_off = text_ro + (END_VA - text_va)
code = b[start_off:end_off]

md = Cs(CS_ARCH_X86, CS_MODE_32)
md.detail = True
for insn in md.disasm(code, START_VA):
    marker = ""
    if insn.mnemonic == "call":
        # Print call target for E8-relative
        if len(insn.bytes) == 5 and insn.bytes[0] == 0xE8:
            rel = struct.unpack_from("<i", insn.bytes, 1)[0]
            tgt = insn.address + 5 + rel
            marker = f"  -> {tgt:#010x}"
    # highlight instructions writing to a memory operand
    if any(op.type == 3 and op.access & 2 for op in insn.operands):  # X86_OP_MEM, CS_AC_WRITE
        marker += "  [MEM WRITE]"
    print(f"{insn.address:#010x}  {insn.mnemonic:8s} {insn.op_str}{marker}")
