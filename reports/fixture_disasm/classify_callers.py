"""For each of the 86 CALL FUN_00670350 sites, find the enclosing function
and dump the ~80 preceding instructions so we can read off:
  * which `param_7` (schedule template ptr) is being pushed
  * which `param_8` (round count) is being pushed
  * which comp id is being set on `param_1`

Function boundaries are found by walking backward from the CALL until we hit
a run of INT3 (0xCC) padding or a return preceded by RET (0xC3 / 0xC2).
"""
import struct
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

EXE = Path(r"D:\cm0102\cm0102.exe")
OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm")
TARGET = 0x00670350

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
text_bytes = exe[text["ro"] : text["ro"] + text["rsz"]]
text_va = ib + text["va"]
text_end = text_va + text["rsz"]

def va_to_file_off(va):
    return text["ro"] + (va - text_va)

# Find all call sites (raw scan again)
call_sites = []
for i in range(len(text_bytes) - 4):
    if text_bytes[i] == 0xE8:
        rel = struct.unpack_from("<i", text_bytes, i+1)[0]
        va = text_va + i
        tgt = (va + 5 + rel) & 0xffffffff
        if tgt == TARGET:
            call_sites.append(va)

print(f"{len(call_sites)} call sites")

# Find containing function: walk back byte-by-byte looking for the first
# byte after a run of >=4 INT3 padding OR a preceded RET+INT3.
def find_fn_start(call_va):
    # Search backwards up to 8KB
    end_off = va_to_file_off(call_va)
    lo = max(0, end_off - 0x2000)
    # Look for the last transition: CC padding run followed by non-CC byte.
    # Also accept C3 / C2 xx xx (ret) followed by CC*.
    # Walk backwards.
    off = end_off - 1
    while off > lo:
        b = text_bytes[off]
        if b == 0xCC:
            # potential padding — look at the byte just before it
            # keep scanning backwards while CC
            j = off
            while j > lo and text_bytes[j] == 0xCC:
                j -= 1
            # The byte before the CC run is either a ret or something else.
            # The function *after* the CC run starts at off (first non-CC after
            # padding walking forward from j).
            return text_va + (j + 1) if text_bytes[j] in (0xc3, 0xc2, 0xcc) else None if False else text_va + (j + 1 + (1 if text_bytes[j] in (0xc3, 0xc2) else 0))
        off -= 1
    return None

# Simpler heuristic: find nearest preceding `55 8B EC` (push ebp; mov ebp, esp)
# OR nearest preceding `83 EC` (sub esp, imm8) that's after a CC padding.
def find_fn_start_v2(call_va):
    end_off = va_to_file_off(call_va)
    lo = max(0, end_off - 0x4000)
    # Walk back looking for `55 8B EC` (push ebp; mov ebp, esp) with CC before
    for off in range(end_off - 3, lo, -1):
        if (text_bytes[off]   == 0x55
            and text_bytes[off+1] == 0x8b
            and text_bytes[off+2] == 0xec):
            # look at 2 bytes back for CC padding
            if off >= 1 and text_bytes[off-1] == 0xcc:
                return text_va + off
    # Fallback: find any preceding CC run
    for off in range(end_off - 1, lo, -1):
        if text_bytes[off] == 0xcc and off + 1 < len(text_bytes) and text_bytes[off+1] != 0xcc:
            return text_va + off + 1
    return None

md = Cs(CS_ARCH_X86, CS_MODE_32)
md.detail = True

lines_out = []
prior_fn_starts = set()
by_fn = {}
for site in call_sites:
    fn_start = find_fn_start_v2(site)
    if fn_start is not None:
        by_fn.setdefault(fn_start, []).append(site)
    else:
        by_fn.setdefault(0, []).append(site)

print(f"{len(by_fn)} distinct containing functions")

# Dump each function's disassembly from start to the CALL (max 1500 bytes)
lines_out.append(f"# {len(call_sites)} CALL FUN_00670350 sites in {len(by_fn)} enclosing functions\n")
for fn_start, sites in sorted(by_fn.items()):
    lines_out.append(f"\n## Function starting near {fn_start:#010x}")
    lines_out.append(f"  Contains {len(sites)} calls to FUN_00670350: {[hex(s) for s in sites]}")
    start_off = va_to_file_off(fn_start)
    max_call_off = max(va_to_file_off(s) for s in sites)
    slice_len = min(1500, max_call_off - start_off + 200)
    blob = text_bytes[start_off : start_off + slice_len]
    for insn in md.disasm(blob, fn_start):
        lines_out.append(f"  {insn.address:#010x}  {insn.bytes.hex():<18} {insn.mnemonic} {insn.op_str}")
        if insn.address >= max(sites) + 5:
            break

(OUT / "fun00670350_callers.txt").write_text("\n".join(lines_out), encoding="utf-8")
print(f"wrote {OUT/'fun00670350_callers.txt'} ({sum(len(l) for l in lines_out)} bytes)")
