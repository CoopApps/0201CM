"""Emulate the NEWS screen draw callback (0x770170) and capture every widget's
EXACT resolved geometry -- the binary computing its own coordinates.

Hooks the two constructors (guio 0x549580, area 0x549790), the tab-list builder
(0x5d7070) and the bottom nav bar (0x5d75b0); RET-stubs every other function so
emulation stays inside the callback. Constant/font-derived geometry resolves
exactly; state-derived values fall back to 0 (lazy zero pages).
"""
import struct, json
from unicorn import *
from unicorn.x86_const import *

EXE = "D:/cm0102/cm0102.exe"
IB = 0x400000
GUIO = 0x00549580
AREA = 0x00549790
TABS = 0x005D7070
NAV  = 0x005D75B0
FONTH = 0x005CF7B0
NEWSCNT = 0x0076EF80
OP_NEW = 0x00933D81
GUI_ALLOC = 0x005CE4F0
STACK = 0x10000000; STACK_SZ = 0x200000
STATE = 0x10600000; STATE_SZ = 0x100000
HEAP = 0x10800000; HEAP_SZ = 0x400000     # bump allocator for operator_new

def load_pe(uc):
    exe = open(EXE, "rb").read()
    pe = struct.unpack_from("<I", exe, 0x3c)[0]
    nsec = struct.unpack_from("<H", exe, pe + 6)[0]; opt = struct.unpack_from("<H", exe, pe + 20)[0]
    so = pe + 24 + opt; img_end = 0; secs = []
    for i in range(nsec):
        o = so + i * 40
        vsz = struct.unpack_from("<I", exe, o + 8)[0]; va = struct.unpack_from("<I", exe, o + 12)[0]
        rawsz = struct.unpack_from("<I", exe, o + 16)[0]; rawp = struct.unpack_from("<I", exe, o + 20)[0]
        secs.append((va, rawsz, rawp)); img_end = max(img_end, va + max(vsz, rawsz))
    uc.mem_map(IB, (img_end + 0xfff) & ~0xfff)
    for va, rawsz, rawp in secs:
        if rawsz: uc.mem_write(IB + va, exe[rawp:rawp + rawsz])

START = 0x00770170; END = 0x007712c0

class Cap:
    def __init__(self):
        self.objects = []; self.areas = []; self.tabs = []; self.nav = []
        self.next_id = 1
        self.heap = HEAP
        self.uc = Uc(UC_ARCH_X86, UC_MODE_32)
        load_pe(self.uc)
        self.uc.mem_map(STACK, STACK_SZ); self.uc.mem_map(STATE, STATE_SZ)
        self.uc.mem_map(HEAP, HEAP_SZ)
        # Let the tab-list builder (0x5d7070) and its allocators run so it
        # advances the top/bottom tab-strip Y cursors; RET-stub everything else.
        keep = {TABS}
        fns = json.load(open("D:/cm0102-carve/ghidra_out/cm0102.exe/functions.json"))
        for f in fns:
            e = int(f["entry"], 16)
            if not (START <= e < END) and e not in keep:
                try:
                    self.uc.mem_write(e, b"\xc3")
                except UcError:
                    pass
        self.uc.mem_map(0, 0x2000); self.uc.mem_write(0, b"\xc3" * 0x2000)
        self.uc.hook_add(UC_HOOK_CODE, self._code)
        self._mapped = set()
        self.uc.hook_add(UC_HOOK_MEM_READ_UNMAPPED | UC_HOOK_MEM_WRITE_UNMAPPED, self._lazy)

    def _lazy(self, uc, access, address, size, value, _):
        page = address & ~0xfff
        if page not in self._mapped:
            try:
                uc.mem_map(page, 0x1000); self._mapped.add(page)
            except UcError:
                pass
        return True

    def _args(self, n):
        esp = self.uc.reg_read(UC_X86_REG_ESP)
        raw = self.uc.mem_read(esp + 4, n * 4)
        return [struct.unpack_from("<i", raw, i * 4)[0] for i in range(n)]

    def _deref(self, ptr):
        try:
            return struct.unpack("<i", self.uc.mem_read(ptr & 0xffffffff, 4))[0]
        except UcError:
            return None

    def _code(self, uc, addr, size, _):
        if addr == GUIO:
            a = self._args(18); oid = self.next_id; self.next_id += 1
            self.objects.append((oid, a)); uc.reg_write(UC_X86_REG_EAX, oid); return
        if addr == AREA:
            a = self._args(11); aid = self.next_id; self.next_id += 1
            self.areas.append((aid, a)); uc.reg_write(UC_X86_REG_EAX, aid); return
        if addr == TABS:
            a = self._args(6)  # capture the strip cursors on ENTRY (pre-advance)
            self.tabs.append({"count": a[1], "sel": a[2],
                              "top_y_in": self._deref(a[3]), "bot_y_in": self._deref(a[4]),
                              "split": a[5], "p4": a[3], "p5": a[4]})
            return  # let the real body run (advances the cursors, builds tab items)
        if addr == NAV:
            a = self._args(2)
            self.nav.append({"back": a[0], "next": a[1]})
            uc.reg_write(UC_X86_REG_EAX, 1); return
        if addr == FONTH:
            uc.reg_write(UC_X86_REG_EAX, 21); return
        if addr == NEWSCNT:
            uc.reg_write(UC_X86_REG_EAX, 0); return   # 0 unread -> skip the row loop
        if addr == OP_NEW or addr == GUI_ALLOC:
            n = self._args(1)[0] if addr == OP_NEW else 0x2000
            p = self.heap; self.heap += (n + 0xf) & ~0xf
            uc.reg_write(UC_X86_REG_EAX, p); return

    def run(self):
        esp = STACK + STACK_SZ - 0x8000
        for i in range(8):
            self.uc.mem_write(esp + 4 + i * 4, struct.pack("<i", STATE))
        self.uc.mem_write(esp, struct.pack("<I", 0x1))
        self.uc.reg_write(UC_X86_REG_ESP, esp)
        self.uc.reg_write(UC_X86_REG_ECX, STATE)
        try:
            self.uc.emu_start(START, 0x1, count=2000000)
        except UcError as e:
            print("  emu note:", e, "eip", hex(self.uc.reg_read(UC_X86_REG_EIP)))

AREA_FIELDS = ["L","T","R","B","cntA","wA","cntB","wB","flags","a10","parent"]
GUIO_FIELDS = ["type","L","T","R","B","a6","a7","rflags","colP","colS","tflags","font","tmode","text","a15","a16","a17","parent"]

if __name__ == "__main__":
    cap = Cap(); cap.run()
    print(f"areas={len(cap.areas)} objects={len(cap.objects)} tabs={len(cap.tabs)} nav={len(cap.nav)}")
    print("\n== AREAS (L,T,R,B, cols, flags) ==")
    for aid, a in cap.areas:
        print(f"  #{aid}: L={a[0]} T={a[1]} R={a[2]} B={a[3]} cntA={a[4]} cntB={a[6]} flags=0x{a[8]&0xffffffff:x}")
    print("\n== TAB LISTS ==")
    for t in cap.tabs:
        print("  ", t)
    print("\n== NAV BARS ==")
    for n in cap.nav:
        print("  ", n)
    print("\n== OBJECTS (type,L,T,R,B,rflags,font,event) ==")
    for oid, a in cap.objects:
        print(f"  #{oid}: type={a[0]} L={a[1]} T={a[2]} R={a[3]} B={a[4]} rflags=0x{a[7]&0xffffffff:x} font={a[11]} event={a[15]}")
