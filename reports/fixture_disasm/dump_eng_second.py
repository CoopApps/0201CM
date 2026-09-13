"""Disassemble eng_second.cpp starting at 0x0055f040 and dump every
instruction up to the first RET. Then locate the CALL to FUN_00670350
and identify the immediates pushed just before it.
"""
import struct
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

EXE = Path(r"D:\cm0102\cm0102.exe")
OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm")
FN_START = 0x0055f040
FN_END_HINT = 0x0055f3a2   # next function in eng_second.cpp per file_attribution

def parse_pe(b):
    e = struct.unpack_from("<I", b, 0x3c)[0]
    coff = e + 4
    n_sec = struct.unpack_from("<H", b, coff+2)[0]
    opt = struct.unpack_from("<H", b, coff+16)[0]
    off = coff + 20
    ib = struct.unpack_from("<I", b, off+28)[0]
    so = off + opt
    secs = []
    for i in range(n_sec):
        base = so + i*40
        secs.append(dict(
            name=b[base:base+8].rstrip(b"\0").decode("latin1","replace"),
            va=struct.unpack_from("<I",b,base+12)[0],
            ro=struct.unpack_from("<I",b,base+20)[0],
        ))
    return ib, secs

exe = EXE.read_bytes()
ib, secs = parse_pe(exe)
text = next(s for s in secs if s["name"] == ".text")
def va_to_off(va): return text["ro"] + (va - (ib + text["va"]))

blob = exe[va_to_off(FN_START) : va_to_off(FN_END_HINT)]
print(f"Disassembling {len(blob)} bytes from {FN_START:#010x} to {FN_END_HINT:#010x}")

md = Cs(CS_ARCH_X86, CS_MODE_32)
md.detail = True
lines = []
call_00670350_sites = []
call_00668890_sites = []
push_immediates = []
for insn in md.disasm(blob, FN_START):
    lines.append(f"{insn.address:#010x}  {insn.bytes.hex():<20}  {insn.mnemonic:<8} {insn.op_str}")
    if insn.mnemonic == "call":
        try:
            tgt = int(insn.op_str, 16)
            if tgt == 0x00670350:
                call_00670350_sites.append((insn.address, len(lines) - 1))
            elif tgt == 0x00668890:
                call_00668890_sites.append((insn.address, len(lines) - 1))
        except (ValueError, TypeError):
            pass
    # collect push immediate values
    if insn.mnemonic == "push":
        op = insn.op_str
        if op.startswith("0x") or op.startswith("-") or op.isdigit():
            try:
                v = int(op, 16) if op.startswith("0x") else int(op)
                push_immediates.append((insn.address, v))
            except ValueError:
                pass

full = "\n".join(lines)
(OUT / "eng_second_disasm.txt").write_text(full, encoding="utf-8")
print(f"wrote eng_second_disasm.txt ({len(lines)} insns)")
print(f"CALL FUN_00670350: {len(call_00670350_sites)} sites at {[hex(s) for s,_ in call_00670350_sites]}")
print(f"CALL FUN_00668890: {len(call_00668890_sites)} sites at {[hex(s) for s,_ in call_00668890_sites]}")

# For each FUN_00670350 call, print the previous 60 lines
print("\n\n### CALL FUN_00670350 context (previous 60 lines each):")
for site_va, idx in call_00670350_sites:
    print(f"\n--- CALL at {site_va:#010x} ---")
    lo = max(0, idx - 60)
    for j in range(lo, idx + 1):
        print(lines[j])
