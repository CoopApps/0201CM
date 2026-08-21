"""Emulate the league-table builder to capture the REAL table area record (bounds +
column/row counts + weight arrays), then run the real layout engine on it -> the exact
row and column rectangles. Fixes row spacing by executing the binary, not guessing.
"""
import struct, importlib.util
spec = importlib.util.spec_from_file_location("cc", "D:/cm0102-rs/tools/capture_construct.py")
cc = importlib.util.module_from_spec(spec); spec.loader.exec_module(cc)
spec2 = importlib.util.spec_from_file_location("emu", "D:/cm0102-rs/tools/emu.py")
emu = importlib.util.module_from_spec(spec2); spec2.loader.exec_module(emu)
import json

fns = json.load(open("D:/cm0102-carve/ghidra_out/cm0102.exe/functions.json"))
fmap = {int(f["entry"], 16): f.get("size", 0) for f in fns}

START = 0x00495ad0
cap = cc.Cap(START, fmap.get(START, 0x1a00), helper_returns={
    0x005cf7b0: 21,        # graphics_font_row_height(slot 3) -> arial_14 height 21
    0x007e6ee0: cc.STATE,  # view-model getter -> state
})
# competition state: teams and schedule so the builder walks the full table path
for off, val, sz in [(0x3e, 20, 2), (0x3c, 1, 2), (0x3a, 0, 2), (0xb6, 0xffff, 2),
                     (0x45, 1, 4), (0x42, 0, 1), (0xbe, 0, 1), (0xbf, 0, 1),
                     (0xc0, 3, 1), (0xc1, 0, 1), (0xd9, 0, 1)]:
    cap.uc.mem_write(cc.STATE + off, val.to_bytes(sz, "little"))
# give STATE a valid vtable so indirect (**vtable[n])() calls resolve to a stub (auto-stubbed
# to return 0) instead of jumping through a null pointer.
STUB = 0x4000
cap.uc.mem_map(STUB & ~0xfff, 0x1000); cap.uc.mem_write(STUB, b"\xc3")
VT = cc.STATE + 0x800
for i in range(128):
    cap.uc.mem_write(VT + i * 4, struct.pack("<I", STUB))
cap.uc.mem_write(cc.STATE + 0, struct.pack("<I", VT))   # *STATE = vtable
# the builder reads its inputs at high stack offsets (Ghidra in_stack_00001200/1204/1208):
#   +0x1200 = competition ptr ; +0x1204 = body top (0x50=80) ; +0x1208 = body bottom (0x221=545)
ESP = cc.STACK + cc.STACK_SZ - 0x8000
cap.uc.mem_write(ESP + 0x1200, struct.pack("<i", cc.STATE))
cap.uc.mem_write(ESP + 0x1204, struct.pack("<i", 80))
cap.uc.mem_write(ESP + 0x1208, struct.pack("<i", 545))
cap.run(START, ecx=cc.STATE, stack_args=[cc.STATE] * 64)

print(f"captured {len(cap.areas)} areas, {len(cap.objects)} objects")
def rd_bytes(addr, n):
    try: return bytes(cap.uc.mem_read(addr & 0xffffffff, n))
    except Exception: return b""

for aid, a in cap.areas:
    L, T, R, B, cntA, wA, cntB, wB, flags = a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8]
    if cntA == 11:   # the league table grid
        col_w = list(rd_bytes(wA, cntA)) if wA else []
        row_w = list(rd_bytes(wB, cntB)) if wB else []
        print(f"\nTABLE AREA id{aid}: L={L} T={T} R={R} B={B} flags={flags:#x}")
        print(f"  columns={cntA} weights={col_w}")
        print(f"  rows={cntB} weights={row_w}")
        # run the REAL layout engine on this exact area
        rec = emu.run_layout(emu.build_area(L, T, R, B, flags, col_w or [1]*cntA, row_w or [1]*cntB))
        cl, cr, rt, rb = emu.read_tables(rec, cntA, cntB)
        print(f"  -> row pitch (first rows, absolute y): "
              + ", ".join(f"[{T+rt[i]},{T+rb[i]}]" for i in range(min(6, cntB))))
        if cntB > 1:
            print(f"  -> row height = {rb[0]-rt[0]+1}px, pitch = {rt[1]-rt[0]}px")
