"""Global raw-byte scan of cm0102.exe for CALL to specific league-comp helpers.

Search:
  * every `call rel32` (0xE8) to FUN_00670350 (0x00670350)  — LeagueComp base ctor
  * every `call rel32` to FUN_005ac250 (0x005ac250)         — Manager container ctor (for sanity)
  * every `call rel32` to FUN_006508e0 (0x006508e0)         — tour table populator
"""
import struct
from pathlib import Path

EXE = Path(r"D:\cm0102\cm0102.exe")
OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm")
OUT.mkdir(parents=True, exist_ok=True)

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
            vsz=struct.unpack_from("<I",b,base+8)[0],
            ro=struct.unpack_from("<I",b,base+20)[0],
            rsz=struct.unpack_from("<I",b,base+16)[0],
        ))
    return ib, secs

exe = EXE.read_bytes()
ib, sections = parse_pe(exe)

text = next(s for s in sections if s["name"] == ".text")
text_bytes = exe[text["ro"]:text["ro"]+text["rsz"]]
text_va = ib + text["va"]
print(f".text at VA {text_va:#010x}, size {len(text_bytes)}")

targets = {
    "FUN_00670350 (LeagueComp base ctor)":       0x00670350,
    "FUN_005ac250 (Manager container ctor)":     0x005ac250,
    "FUN_006508e0 (tour table populator)":       0x006508e0,
    "FUN_00811140 (game-setup driver, calls tour populator)": 0x00811140,
    "FUN_00668890 (round-robin driver)":         0x00668890,
    "FUN_0066f280 (schedule date walker)":       0x0066f280,
    "FUN_00558f60 (FA Cup schedule ctor)":       0x00558f60,
    "FUN_00556150 (League Cup schedule ctor)":   0x00556150,
    "FUN_00669780 (Berger matrix builder)":      0x00669780,
    "FUN_0066bd40 (seeding shuffle)":            0x0066bd40,
}

results = {}
for i in range(len(text_bytes) - 4):
    if text_bytes[i] == 0xE8:
        rel = struct.unpack_from("<i", text_bytes, i+1)[0]
        call_va = text_va + i
        tgt = (call_va + 5 + rel) & 0xffffffff
        for label, addr in targets.items():
            if tgt == addr:
                results.setdefault(addr, []).append(call_va)

lines = []
for label, addr in targets.items():
    hits = results.get(addr, [])
    lines.append(f"\n## {label}  -> {addr:#010x}")
    lines.append(f"total calls: {len(hits)}")
    for va in hits:
        lines.append(f"  call at {va:#010x}")

out = "\n".join(lines)
print(out[:5000])
(OUT / "global_scan.txt").write_text(out, encoding="utf-8")
print(f"\nwrote {OUT/'global_scan.txt'}")
