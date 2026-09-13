"""Verify whether the schedule-getter at 0x0055f340 has the same code in
cm0102.exe (DirectDraw build I've been static-analyzing) and
cm0102_GDI.exe (currently running). If yes, we can hook the running
GDI process directly with the same addresses.
"""
import struct
from pathlib import Path

EXES = [Path(r"D:\cm0102\cm0102.exe"), Path(r"D:\cm0102\cm0102_GDI.exe")]
ADDRS = [
    ("eng_second ctor",  0x0055f040),
    ("schedule-getter",  0x0055f340),
    ("FUN_0066f3b0",     0x0066f3b0),
    ("FUN_00668890",     0x00668890),
    ("FUN_0066f280",     0x0066f280),
    ("FUN_00533b50",     0x00533b50),
    ("vtable slot",      0x00957dc8),
]

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

def va_to_off(exe, ib, secs, va):
    for s in secs:
        rva = va - ib
        if s["va"] <= rva < s["va"] + s["rsz"]:
            return s["ro"] + (rva - s["va"])
    return None

blobs = {}
for p in EXES:
    b = p.read_bytes()
    ib, secs = parse_pe(b)
    print(f"\n{p.name}: image_base {ib:#010x}")
    for name, va in ADDRS:
        off = va_to_off(b, ib, secs, va)
        if off is None:
            print(f"  {name:20s} @ {va:#010x} : NOT FOUND")
            continue
        first16 = b[off:off+16].hex()
        print(f"  {name:20s} @ {va:#010x} -> file {off:#010x} first16={first16}")
        blobs.setdefault(name, []).append(first16)

print("\n=== SAME BYTES? ===")
for name, hs in blobs.items():
    print(f"  {name:20s}: {'IDENTICAL' if len(set(hs)) == 1 else 'DIFFER'}")
