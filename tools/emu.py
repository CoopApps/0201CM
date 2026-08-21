"""Emulate cm0102.exe functions with Unicorn to get EXACT runtime-computed geometry
(no reconstruction, no guessing -- the binary computes its own values).

Stage 1 (this file): PE loader + run area_rebuild_layout_tables (0x00403390) on a
constructed area record, validate the emulated column table matches the known-good
league geometry [112,164]..[726,778]. Proves the harness before it drives the frame.
"""
import struct
from unicorn import *
from unicorn.x86_const import *

EXE = "D:/cm0102/cm0102.exe"
IB = 0x400000

def load_pe(uc):
    exe = open(EXE, "rb").read()
    pe = struct.unpack_from("<I", exe, 0x3c)[0]
    nsec = struct.unpack_from("<H", exe, pe + 6)[0]
    opt = struct.unpack_from("<H", exe, pe + 20)[0]
    so = pe + 24 + opt
    # map full image region rounded to page, zero-filled, then copy raw section bytes
    img_end = 0
    secs = []
    for i in range(nsec):
        o = so + i * 40
        vsz = struct.unpack_from("<I", exe, o + 8)[0]
        va = struct.unpack_from("<I", exe, o + 12)[0]
        rawsz = struct.unpack_from("<I", exe, o + 16)[0]
        rawp = struct.unpack_from("<I", exe, o + 20)[0]
        secs.append((va, vsz, rawsz, rawp))
        img_end = max(img_end, va + max(vsz, rawsz))
    total = (img_end + 0xfff) & ~0xfff
    uc.mem_map(IB, total)
    for va, vsz, rawsz, rawp in secs:
        data = exe[rawp:rawp + rawsz]
        if data:
            uc.mem_write(IB + va, data)
    return total

STACK = 0x10000000       # well above the mapped image (image ends ~0xdcc000)
STACK_SZ = 0x100000
SCRATCH = 0x10200000     # where we build the area record
SCRATCH_SZ = 0x100000

def new_uc():
    uc = Uc(UC_ARCH_X86, UC_MODE_32)
    load_pe(uc)
    uc.mem_map(STACK, STACK_SZ)
    uc.mem_map(SCRATCH, SCRATCH_SZ)
    return uc

def run_layout(area_bytes, entry=0x00403390):
    """Call FUN_00403390(area) [__fastcall: ECX=area ptr]. Returns the area record
    after execution so we can read the computed layout tables."""
    uc = new_uc()
    area_addr = SCRATCH
    uc.mem_write(area_addr, area_bytes)
    # stub any sub-call (e.g. scrollbar FUN_00403640, error paths) by trapping CALLs
    # to keep the run self-contained: we hook code and, on a call out of the function,
    # emulate a simple 'ret' by not following. Simplest: set a big return addr sentinel
    # and stop at it; stub FUN_00403640 by mapping a RET there.
    for stub in (0x00403640, 0x00933d8f, 0x008fc660, 0x00933d2f, 0x005d1c30):
        uc.mem_write(stub, b"\xc3")     # RET -- neutralize helper calls
    RET = 0x1000
    uc.mem_map(RET & ~0xfff, 0x1000)
    uc.reg_write(UC_X86_REG_ECX, area_addr)          # __fastcall arg1
    uc.reg_write(UC_X86_REG_ESP, STACK + STACK_SZ - 0x400)
    # push return address
    esp = STACK + STACK_SZ - 0x400
    uc.mem_write(esp, struct.pack("<I", RET))
    try:
        uc.emu_start(entry, RET, count=200000)
    except UcError as e:
        print("emu stopped:", e, "eip", hex(uc.reg_read(UC_X86_REG_EIP)))
    return uc.mem_read(area_addr, len(area_bytes))

def build_area(left, top, right, bottom, flags, col_w, row_w):
    """Construct an area record with the fields FUN_00403390 reads."""
    rec = bytearray(0x1000)
    struct.pack_into("<i", rec, 0x00, left)
    struct.pack_into("<i", rec, 0x04, top)
    struct.pack_into("<i", rec, 0x08, right)
    struct.pack_into("<i", rec, 0x0c, bottom)
    struct.pack_into("<i", rec, 0x14, 0)   # param_1[5]: visible-row capacity; 0 => no scrollbar
    struct.pack_into("<I", rec, 0x18, flags)            # param_1[6] area flags
    struct.pack_into("<H", rec, 0x20e, 0xffff)          # -1 : skip the +0x20e branch
    rec[0xbab] = len(col_w)                              # column count
    for i, w in enumerate(col_w): rec[0xbad + i] = w
    rec[0xbac] = len(row_w)                             # row count (byte at 0xbac = param_1[0x2eb])
    for i, w in enumerate(row_w): rec[0xbcb + i] = w
    return bytes(rec)

def read_tables(rec, ncol, nrow):
    def i32(off): return struct.unpack_from("<i", rec, off)[0]
    col_left = [i32(0x1c + 4 * i) for i in range(ncol)]   # param_1[7+i]
    col_right = [i32(0x10c + 4 * i) for i in range(ncol)] # param_1[0x43+i]
    row_top = [i32(0x94 + 4 * i) for i in range(nrow)]    # param_1[0x25+i]
    row_bot = [i32(0x184 + 4 * i) for i in range(nrow)]   # param_1[0x61+i]
    return col_left, col_right, row_top, row_bot

if __name__ == "__main__":
    # league table VARIANT: left=110 top=80 right=780 bottom=545 flags=2, 11 cols
    cw = [4, 2, 12, 4, 4, 4, 4, 4, 4, 4, 4]
    rec = run_layout(build_area(110, 80, 780, 545, 2, cw, [1]))
    cl, cr, rt, rb = read_tables(rec, 11, 1)
    print("emulated col x-bounds:")
    for i in range(11):
        print(f"  col {i}: [{110+cl[i]}, {110+cr[i]}]" if False else f"  col {i}: [{cl[i]}, {cr[i]}]")
    print("expected (VARIANT): [112,164][166,191][193,351][353,404][406,457]...")
