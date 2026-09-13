"""Find GDI eng_prm / eng_first / eng_second / eng_third / eng_conf
constructors + their vtable pointers + schedule-getter VAs.

Method:
  1. For each expected comp id (7, 8, 9, 10, 93), find `mov byte ptr
     [esi+0x50], <id>` in .text. Only one caller per id.
  2. Walk backward to the function entry (functions.json).
  3. Disassemble the ctor to find `mov dword ptr [esi], <vtable_va>`.
  4. Read the vtable in .rdata at that VA, extract slot +0x3c
     (schedule-getter VA).
  5. Confirm each getter starts with the same "mov al, byte ptr
     [esp+4]; sub esp, 0x208" prologue we already characterised for
     eng_second at 0x0055f540.
"""
import struct, json
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

GDI = Path(r"D:\cm0102\cm0102_GDI.exe")
FN = Path(r"D:\cm0102-carve\ghidra_out\cm0102_GDI.exe\functions.json")

data = json.loads(FN.read_text())
fns = data if isinstance(data, list) else data.get("functions", [])
records = []
for f in fns:
    if not isinstance(f, dict): continue
    e = f.get("entry") or f.get("va"); s = f.get("size") or 0
    if isinstance(e, str):
        e = int(e, 16) if e.startswith("0x") else int(e)
    if isinstance(s, str):
        s = int(s, 16) if s.startswith("0x") else int(s)
    records.append((e, s, f.get("name") or ""))
records.sort()

def containing(addr):
    lo, hi = 0, len(records) - 1
    while lo <= hi:
        m = (lo + hi) // 2
        e, s, _ = records[m]
        if addr < e: hi = m - 1
        elif addr >= e + s: lo = m + 1
        else: return records[m]
    return None

b = GDI.read_bytes()
e_pe = struct.unpack_from("<I", b, 0x3c)[0]; coff = e_pe + 4
n_sec = struct.unpack_from("<H", b, coff+2)[0]
opt = struct.unpack_from("<H", b, coff+16)[0]; off = coff + 20
so = off + opt
secs = {}
IMAGE_BASE = 0x00400000
for i in range(n_sec):
    base = so + i*40
    name = b[base:base+8].rstrip(b"\0").decode('latin1','replace')
    secs[name] = dict(va=struct.unpack_from("<I", b, base+12)[0],
                      ro=struct.unpack_from("<I", b, base+20)[0],
                      rsz=struct.unpack_from("<I", b, base+16)[0])

text = secs[".text"]
text_va = text["va"] + IMAGE_BASE
text_ro = text["ro"]
text_sz = text["rsz"]
rdata = secs[".rdata"]

def va_to_off(va):
    for s in secs.values():
        rva = va - IMAGE_BASE
        if s["va"] <= rva < s["va"] + s["rsz"]:
            return s["ro"] + (rva - s["va"])
    return None

md = Cs(CS_ARCH_X86, CS_MODE_32)

# Find all `mov byte ptr [esi+0x50], N` sites for each id.
def find_mov_esi_50_imm(imm):
    pat = bytes([0xC6, 0x46, 0x50, imm])
    matches = []
    i = text_ro
    while True:
        i = b.find(pat, i, text_ro + text_sz)
        if i < 0: break
        matches.append(text_va + (i - text_ro))
        i += 1
    return matches

# Read vtable slot: given a ctor entry, disasm forward looking for
# `mov dword ptr [esi], <VA>`. Then read that vtable + 0x3c to get
# schedule-getter VA.
def disasm_find_vtable(entry, span=0x200):
    off = va_to_off(entry)
    if off is None: return None
    code = b[off:off + span]
    for insn in md.disasm(code, entry):
        if insn.mnemonic == "mov" and insn.op_str.startswith("dword ptr [esi], "):
            # imm target
            parts = insn.op_str.split(",")
            imm = parts[1].strip()
            return int(imm, 16) if imm.startswith("0x") else int(imm)
        # Also handle relative variants like "dword ptr [ecx], ..."
        if insn.mnemonic == "mov" and "dword ptr [" in insn.op_str and ", 0x" in insn.op_str:
            # Match [esi] or [reg], N pattern
            if insn.op_str.startswith("dword ptr [esi]"):
                imm = insn.op_str.split(",")[1].strip()
                return int(imm, 16)
    return None

def read_vtable_slot(vtable_va, slot_off):
    off = va_to_off(vtable_va + slot_off)
    if off is None: return None
    return struct.unpack_from("<I", b, off)[0]

DIVISIONS = [
    (7,  "Premier"),
    (8,  "First"),
    (9,  "Second"),
    (10, "Third"),
    (93, "Conference"),
]

result = {}
for cid, name in DIVISIONS:
    print(f"\n=== English {name} (comp id {cid}) ===")
    sites = find_mov_esi_50_imm(cid)
    print(f"  {len(sites)} `mov [esi+0x50], {cid}` sites: {[hex(s) for s in sites]}")
    for site in sites:
        f = containing(site)
        if not f: continue
        entry, size, fn_name = f
        vtable = disasm_find_vtable(entry, size)
        if not vtable: continue
        getter = read_vtable_slot(vtable, 0x3c)
        print(f"  → ctor {entry:#010x} (size {size:#x}), vtable {vtable:#010x}, getter (slot +0x3c) {getter:#010x}")
        # Confirm getter starts with the schedule-getter prologue
        # (`mov al, [esp+4]; sub esp, 0x208`).
        goff = va_to_off(getter)
        if goff:
            g_prologue = b[goff:goff + 12].hex()
            confirmed = g_prologue.startswith("8a442404" + "81ec")  # rough
            print(f"    getter prologue: {g_prologue}  (schedule-getter shape? {confirmed})")
        result[cid] = dict(ctor=hex(entry), vtable=hex(vtable), getter=hex(getter), name=name)
        break

OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm\gdi_english_pyramid_map.json")
OUT.write_text(json.dumps(result, indent=2))
print(f"\nSaved to {OUT}")
