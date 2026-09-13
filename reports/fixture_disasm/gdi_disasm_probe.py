"""Disassemble known GDI candidate function starts to verify entries."""
import struct
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

GDI = Path(r"D:\cm0102\cm0102_GDI.exe")
b = GDI.read_bytes()

def parse_pe(bin):
    e = struct.unpack_from("<I", bin, 0x3c)[0]; coff = e + 4
    n = struct.unpack_from("<H", bin, coff+2)[0]
    opt = struct.unpack_from("<H", bin, coff+16)[0]; off = coff + 20
    ib = struct.unpack_from("<I", bin, off+28)[0]; so = off + opt
    secs = {}
    for i in range(n):
        base = so + i*40
        name = bin[base:base+8].rstrip(b"\0").decode('latin1','replace')
        secs[name] = dict(va=struct.unpack_from("<I", bin, base+12)[0],
                          ro=struct.unpack_from("<I", bin, base+20)[0],
                          rsz=struct.unpack_from("<I", bin, base+16)[0])
    return ib, secs

ib, secs = parse_pe(b)
text = secs[".text"]
text_ro = text["ro"]
text_va = ib + text["va"]

def voff(va): return text_ro + (va - text_va)

md = Cs(CS_ARCH_X86, CS_MODE_32)

TARGETS = [
    (0x0055f240, "eng_second_ctor (GDI)", 60),
    (0x0055f540, "candidate schedule-getter start (GDI)", 60),
    (0x0055f480, "FUN_0055f480 - what is it?", 60),
    (0x00668450, "round-robin driver (GDI)", 40),
    (0x0066ef70, "round writer (GDI)", 40),
    (0x0066efd0, "slot writer (GDI)", 40),
]
for va, name, count in TARGETS:
    print(f"\n=== {name} @ {va:#010x} ===")
    off = voff(va)
    code = b[off:off + count * 6]
    for ii, insn in enumerate(md.disasm(code, va)):
        if ii >= count: break
        # Highlight call targets
        note = ""
        if insn.mnemonic == "call" and len(insn.bytes) == 5 and insn.bytes[0] == 0xE8:
            rel = struct.unpack_from("<i", insn.bytes, 1)[0]
            tgt = insn.address + 5 + rel
            note = f"  -> {tgt:#010x}"
        print(f"{insn.address:#010x}  {insn.mnemonic:8s} {insn.op_str}{note}")
