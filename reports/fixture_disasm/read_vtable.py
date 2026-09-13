"""Read the eng_second vtable at VA 0x00957dc8 and dump slots.
Slot at +0x3c is the schedule-getter virtual method (per FUN_00560320:26).
"""
import struct
from pathlib import Path
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
            return s["ro"] + (rva - s["va"]), s["name"]
    return None, None

VTABLE_VA = 0x00957dc8
off, sec = va_to_off(VTABLE_VA)
print(f"vtable VA {VTABLE_VA:#010x} -> file {off:#010x} in {sec}")
# Dump 40 slots (160 bytes)
print("\nVtable slots:")
for i in range(40):
    slot_va = VTABLE_VA + i*4
    slot_off, _ = va_to_off(slot_va)
    if slot_off is None: break
    ptr = struct.unpack_from("<I", exe, slot_off)[0]
    tag = ""
    if i * 4 == 0x3c: tag = " <-- schedule-getter (+0x3c)"
    print(f"  +0x{i*4:03x} : {ptr:#010x}{tag}")
