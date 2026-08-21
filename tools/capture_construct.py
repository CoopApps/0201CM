"""Execute a cm0102.exe UI builder in Unicorn and capture every GUI object/area it
creates, with EXACT resolved arguments -- the binary computing its own widget list.

Approach: hook the two constructors (0x549580 guio, 0x549790 area); auto-stub every
other CALL (the builder's helpers) so emulation stays inside the one function. A small
map supplies real return values for the handful of helpers whose result drives control
flow (font row height, the view-model getter, loop terminators).
"""
import struct
from unicorn import *
from unicorn.x86_const import *
import capstone

EXE = "D:/cm0102/cm0102.exe"
IB = 0x400000
GUIO = 0x00549580
AREA = 0x00549790
STACK = 0x10000000; STACK_SZ = 0x200000
SCRATCH = 0x10300000; SCRATCH_SZ = 0x200000
STATE = 0x10600000; STATE_SZ = 0x100000   # fake competition/view-model state

md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32); md.detail = True

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

class Cap:
    def __init__(self, func_start, func_size, helper_returns=None, ret_stub_all=True):
        self.fs = func_start; self.fe = func_start + func_size
        self.helpers = helper_returns or {}
        self.objects = []; self.areas = []
        self.next_id = 1
        self.uc = Uc(UC_ARCH_X86, UC_MODE_32)
        load_pe(self.uc)
        self.uc.mem_map(STACK, STACK_SZ); self.uc.mem_map(SCRATCH, SCRATCH_SZ)
        self.uc.mem_map(STATE, STATE_SZ)
        # RET-stub every OTHER function entry so helper calls return immediately (garbage EAX)
        # and execution can't wander. Constructors + key helpers keep their RET stub too, but a
        # code hook captures/sets EAX at their entry before the RET runs -> no EIP-skip fragility.
        self._orig = {}
        if ret_stub_all:
            import json as _json
            fns = _json.load(open("D:/cm0102-carve/ghidra_out/cm0102.exe/functions.json"))
            for f in fns:
                e = int(f["entry"], 16)
                if not (self.fs <= e < self.fe):
                    try:
                        self._orig[e] = bytes(self.uc.mem_read(e, 1))
                        self.uc.mem_write(e, b"\xc3")     # RET
                    except UcError:
                        pass
        # low memory: RET-filled so any null/indirect call to low addresses returns cleanly
        self.uc.mem_map(0, 0x2000)
        self.uc.mem_write(0, b"\xc3" * 0x2000)
        self.uc.hook_add(UC_HOOK_CODE, self._code)
        # lazily map any unmapped page on access (zero-filled) so missing state doesn't fault;
        # constant-derived geometry stays exact, state-derived values fall back to 0.
        self._mapped = set()
        self.uc.hook_add(UC_HOOK_MEM_READ_UNMAPPED | UC_HOOK_MEM_WRITE_UNMAPPED, self._lazy)

    def _lazy(self, uc, access, address, size, value, _):
        page = address & ~0xfff
        if page not in self._mapped:
            try:
                uc.mem_map(page, 0x1000); self._mapped.add(page)
            except UcError:
                pass
        return True   # retry the access

    def _read_args(self, n):
        esp = self.uc.reg_read(UC_X86_REG_ESP)
        raw = self.uc.mem_read(esp + 4, n * 4)
        return [struct.unpack_from("<i", raw, i * 4)[0] for i in range(n)]

    def _code(self, uc, addr, size, _):
        # constructors/helpers carry a 0xC3 RET stub; capture args + set EAX at entry, then
        # the RET runs and returns cleanly (no EIP manipulation -> robust).
        if addr == GUIO:
            a = self._read_args(18); oid = self.next_id; self.next_id += 1
            self.objects.append((oid, a)); uc.reg_write(UC_X86_REG_EAX, oid); return
        if addr == AREA:
            a = self._read_args(11); aid = self.next_id; self.next_id += 1
            self.areas.append((aid, a)); uc.reg_write(UC_X86_REG_EAX, aid); return
        if addr in self.helpers:
            uc.reg_write(UC_X86_REG_EAX, self.helpers[addr] & 0xffffffff); return

    def run(self, entry, ecx=None, stack_args=None):
        esp = STACK + STACK_SZ - 0x8000   # room above for high-offset stack args, below for locals
        stack_args = stack_args or []
        for i, v in enumerate(stack_args):
            self.uc.mem_write(esp + 4 + i * 4, struct.pack("<i", v))
        self.uc.mem_write(esp, struct.pack("<I", 0x1))   # return sentinel (emu stops at 0x1)
        self.uc.reg_write(UC_X86_REG_ESP, esp)
        if ecx is not None: self.uc.reg_write(UC_X86_REG_ECX, ecx)
        try:
            self.uc.emu_start(entry, 0x1, count=500000)
        except UcError as e:
            print("  emu note:", e, "eip", hex(self.uc.reg_read(UC_X86_REG_EIP)))

GUIO_FIELDS = ["type","L","T","R","B","a6","a7","rflags","colP","colS","tflags","font","tmode","text","a15","a16","a17","parent"]
AREA_FIELDS = ["L","T","R","B","cntA","wA","cntB","wB","flags","a10","parent"]

if __name__ == "__main__":
    import json
    fns = json.load(open("D:/cm0102-carve/ghidra_out/cm0102.exe/functions.json"))
    fmap = {int(f["entry"],16): f.get("size",0) for f in fns}
    # competition screen DRAW CALLBACK at 0x494640 (registered by FUN_00494250 via 0x7e6570).
    # This is the code that builds the banner/tabs/bottom bar. League builder 0x495ad0 stubbed.
    START = 0x00494640; SIZE = 0x495ad0 - 0x00494640
    cap = Cap(START, SIZE, helper_returns={
        0x007e6ee0: STATE,       # view-model getter -> competition/state ptr
        0x00525310: 0,           # title formatter -> benign
        0x005cf7b0: 21,          # font row height
    })
    # competition state: byte+0x43==2 selects the league-table case in the dispatch
    for off, val, sz in [(0x43, 2, 1), (0x3a, 0, 2), (0x3c, 1, 2), (0x3e, 12, 2), (4, STATE, 4)]:
        cap.uc.mem_write(STATE + off, val.to_bytes(sz, "little"))
    cap.run(START, ecx=STATE, stack_args=[STATE, STATE, STATE, STATE])
    print(f"captured {len(cap.objects)} objects, {len(cap.areas)} areas")
    for aid,a in cap.areas[:6]:
        print("  AREA", {AREA_FIELDS[i]:a[i] for i in range(11)})
    for oid,a in cap.objects[:14]:
        print("  OBJ ", {k:a[i] for i,k in enumerate(GUIO_FIELDS) if k in ('L','T','R','B','rflags','font','parent')})
