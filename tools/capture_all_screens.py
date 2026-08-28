"""Frida capture->replay scaffolding for 40 top-priority CM0102 screens.

Generalises tools/capture_news.py (the one currently-validated screen, News @0x770170)
to any screen whose draw callback address is known. For each screen we RET-stub every
other function in the exe, hook the two GUI constructors (guio 0x549580, area 0x549790)
plus the tab-strip builder (0x5d7070) and nav bar (0x5d75b0), execute the callback in
Unicorn, and dump every captured widget's EXACT resolved geometry to
    reports/screen_captures/<name>.json

Usage:
    python tools/capture_all_screens.py <screen-name>
    python tools/capture_all_screens.py --list
    python tools/capture_all_screens.py --all

The <screen-name> is a key in SCREENS below. Where the draw callback address is not
yet known ('addr' is None) the entry is a stub -- fill in the address from the Ghidra
decompile (see reports/menu_tree_scope.md for the 45 known screen builders) and rerun.

Ground truth = whatever bytes the exe writes via GUIO/AREA/TABS/NAV. Compare against
the cm-render output for the same screen with tools/diff_screens.py.
"""
import argparse
import json
import os
import struct
import sys
from pathlib import Path

from unicorn import *
from unicorn.x86_const import *

REPO = Path(__file__).resolve().parents[1]
EXE = "D:/cm0102/cm0102.exe"
FUNCS_JSON = "D:/cm0102-carve/ghidra_out/cm0102.exe/functions.json"
OUT_DIR = REPO / "reports" / "screen_captures"

IB       = 0x400000
GUIO     = 0x00549580   # guio_ctor  (18 args)
AREA     = 0x00549790   # area_ctor  (11 args)
TABS     = 0x005D7070   # tab-strip builder
NAV      = 0x005D75B0   # bottom nav bar
FONTH    = 0x005CF7B0   # font row height -> 21
NEWSCNT  = 0x0076EF80   # news unread count -> 0
OP_NEW   = 0x00933D81
GUI_ALLOC = 0x005CE4F0
SLOT_PUSH = 0x007E7130  # widget-pool slot push (idx, val, kind)
REG_SCREEN = 0x007E6570 # screen-register (5 stdcall args, returns non-zero on success)
REG_SCREEN2 = 0x007E6430 # alt screen-register (6 stdcall args, returns non-zero on success)
SLOT_PUSH2 = 0x007E7000 # widget-pool slot push variant (4 args)

STACK = 0x10000000; STACK_SZ = 0x200000
STATE = 0x10600000; STATE_SZ = 0x100000
HEAP  = 0x10800000; HEAP_SZ  = 0x400000

# Priority-40 screens. `addr` is the draw callback entry (registered via the
# screen-open helper, usually 0x7e6570). `size` is the byte extent of the callback
# used to decide which foreign function entries to RET-stub. Unknowns are None --
# fill from ghidra decompile / reports/menu_tree_scope.md before capturing.
#
# state_bytes: optional list of (offset, value, size_bytes) writes into the STATE
# page to steer branch selection inside the callback (matches capture_construct.py's
# helper_returns technique).
SCREENS = {
    # Validated
    "news":                 {"addr": 0x00770170, "size": 0x1150, "state": []},
    "competition_league":   {"addr": 0x00494640, "size": 0x1490, "state": [(0x43, 2, 1), (0x3a, 0, 2), (0x3c, 1, 2), (0x3e, 12, 2)]},

    # Priority-40 targets -- fill in addr from ghidra decompile.
    # Club / dashboard family
    "dashboard":            {"addr": 0x004551c0, "size": 0x1e90, "state": []},  # builder 00454620/00454bb0, event 00468d50
    "club_overview":        {"addr": 0x00476ef0, "size": 0x6a0, "state": []},   # builder 00476df0 (cmd 0x7d1, club_s)
    "club_squad":           {"addr": 0x008596b0, "size": 0x4ec0, "state": []},  # builder 00859250 (squad/staff list, staff.c)
    "club_reserves":        {"addr": None, "size": 0, "state": []},
    "club_youth":           {"addr": None, "size": 0, "state": []},
    "club_staff":           {"addr": 0x0085e580, "size": 0x3180, "state": []},  # builder 00859600 (staff page variant)
    "club_finance":         {"addr": None, "size": 0, "state": []},
    "club_history":         {"addr": 0x0046bdf0, "size": 0xb90, "state": []},   # builder 0046ba80 (cmd 0x7e6)
    "club_transfers":       {"addr": None, "size": 0, "state": []},
    "club_fixtures":        {"addr": None, "size": 0, "state": []},

    # Player family
    "player_overview":      {"addr": 0x008a2180, "size": 0x4200, "state": []},  # builder 008a20a0 (player profile)
    "player_stats":         {"addr": None, "size": 0, "state": []},
    "player_history":       {"addr": None, "size": 0, "state": []},
    "player_contract":      {"addr": None, "size": 0, "state": []},

    # Tactics / match
    "tactics":              {"addr": 0x0088def0, "size": 0x2470, "state": []},  # registered in 0088a850
    "team_talk":            {"addr": None, "size": 0, "state": []},
    "match_report":         {"addr": None, "size": 0, "state": []},
    "match_stats":          {"addr": None, "size": 0, "state": []},
    "fixture_history":      {"addr": None, "size": 0, "state": []},

    # Competitions
    "competition_dashboard":{"addr": None, "size": 0, "state": []},
    "competition_fixtures": {"addr": None, "size": 0, "state": []},
    "competition_results":  {"addr": None, "size": 0, "state": []},
    "competition_cup_draw": {"addr": None, "size": 0, "state": []},
    "competition_history":  {"addr": None, "size": 0, "state": []},
    "competition_stats":    {"addr": None, "size": 0, "state": []},

    # World / rankings
    "world_rankings":       {"addr": None, "size": 0, "state": []},
    "fifa_rankings":        {"addr": None, "size": 0, "state": []},
    "uefa_coefficients":    {"addr": None, "size": 0, "state": []},
    "hall_of_fame":         {"addr": None, "size": 0, "state": []},
    "manager_history":      {"addr": None, "size": 0, "state": []},

    # Transfers / scouting
    "transfers_incoming":   {"addr": None, "size": 0, "state": []},
    "transfers_outgoing":   {"addr": None, "size": 0, "state": []},
    "scout_reports":        {"addr": None, "size": 0, "state": []},
    "scout_shortlist":      {"addr": None, "size": 0, "state": []},
    "latest_scores":        {"addr": None, "size": 0, "state": []},

    # Dialogs
    "go_holiday":           {"addr": None, "size": 0, "state": []},
    "contract_offer":       {"addr": None, "size": 0, "state": []},
    "job_offer":            {"addr": None, "size": 0, "state": []},
    "select_league":        {"addr": 0x00810ce0, "size": 0x270, "state": []},   # builder 008053d0/00810ca0
    "select_club":          {"addr": None, "size": 0, "state": []},
}

AREA_FIELDS = ["L", "T", "R", "B", "cntA", "wA", "cntB", "wB", "flags", "a10", "parent"]
GUIO_FIELDS = ["type", "L", "T", "R", "B", "a6", "a7", "rflags",
               "colP", "colS", "tflags", "font", "tmode", "text",
               "a15", "a16", "a17", "parent"]


def load_pe(uc):
    exe = open(EXE, "rb").read()
    pe = struct.unpack_from("<I", exe, 0x3c)[0]
    nsec = struct.unpack_from("<H", exe, pe + 6)[0]
    opt = struct.unpack_from("<H", exe, pe + 20)[0]
    so = pe + 24 + opt
    img_end = 0
    secs = []
    for i in range(nsec):
        o = so + i * 40
        vsz = struct.unpack_from("<I", exe, o + 8)[0]
        va = struct.unpack_from("<I", exe, o + 12)[0]
        rawsz = struct.unpack_from("<I", exe, o + 16)[0]
        rawp = struct.unpack_from("<I", exe, o + 20)[0]
        secs.append((va, rawsz, rawp))
        img_end = max(img_end, va + max(vsz, rawsz))
    uc.mem_map(IB, (img_end + 0xfff) & ~0xfff)
    for va, rawsz, rawp in secs:
        if rawsz:
            uc.mem_write(IB + va, exe[rawp:rawp + rawsz])


class Cap:
    def __init__(self, start, size):
        self.start, self.end = start, start + size
        self.objects, self.areas, self.tabs, self.nav, self.slots = [], [], [], [], []
        self.next_id = 1
        self.heap = HEAP
        self.uc = Uc(UC_ARCH_X86, UC_MODE_32)
        load_pe(self.uc)
        self.uc.mem_map(STACK, STACK_SZ)
        self.uc.mem_map(STATE, STATE_SZ)
        self.uc.mem_map(HEAP, HEAP_SZ)
        keep = {TABS}
        fns = json.load(open(FUNCS_JSON))
        for f in fns:
            e = int(f["entry"], 16)
            if not (self.start <= e < self.end) and e not in keep:
                try:
                    self.uc.mem_write(e, b"\xc3")
                except UcError:
                    pass
        self.uc.mem_map(0, 0x2000)
        self.uc.mem_write(0, b"\xc3" * 0x2000)
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
            a = self._args(6)
            self.tabs.append({"count": a[1], "sel": a[2],
                              "top_y_in": self._deref(a[3]),
                              "bot_y_in": self._deref(a[4]),
                              "split": a[5], "p4": a[3], "p5": a[4]})
            return
        if addr == NAV:
            a = self._args(2)
            self.nav.append({"back": a[0], "next": a[1]})
            uc.reg_write(UC_X86_REG_EAX, 1); return
        if addr == FONTH:
            uc.reg_write(UC_X86_REG_EAX, 21); return
        if addr == NEWSCNT:
            uc.reg_write(UC_X86_REG_EAX, 0); return
        if addr == REG_SCREEN:
            # stdcall(5): return non-zero, pop 5 args + return addr.
            esp = self.uc.reg_read(UC_X86_REG_ESP)
            try:
                ret_addr = struct.unpack("<I", self.uc.mem_read(esp, 4))[0]
            except UcError:
                ret_addr = 1
            uc.reg_write(UC_X86_REG_EAX, 1)
            uc.reg_write(UC_X86_REG_ESP, esp + 4 + 5*4)  # pop return + 5 args
            uc.reg_write(UC_X86_REG_EIP, ret_addr)
            return
        if addr == REG_SCREEN2:
            # stdcall(6): return non-zero, pop 6 args + return addr.
            esp = self.uc.reg_read(UC_X86_REG_ESP)
            try:
                ret_addr = struct.unpack("<I", self.uc.mem_read(esp, 4))[0]
            except UcError:
                ret_addr = 1
            uc.reg_write(UC_X86_REG_EAX, 1)
            uc.reg_write(UC_X86_REG_ESP, esp + 4 + 6*4)
            uc.reg_write(UC_X86_REG_EIP, ret_addr)
            return
        if addr == SLOT_PUSH2:
            # stdcall(4) variant of SLOT_PUSH: (obj, idx, val, kind)
            a = self._args(4)
            if len(self.slots) < 200:
                self.slots.append({"idx": a[1], "val": a[2], "kind": a[3]})
            esp = self.uc.reg_read(UC_X86_REG_ESP)
            try:
                ret_addr = struct.unpack("<I", self.uc.mem_read(esp, 4))[0]
            except UcError:
                ret_addr = 1
            uc.reg_write(UC_X86_REG_EAX, 1)
            uc.reg_write(UC_X86_REG_ESP, esp + 4 + 4*4)
            uc.reg_write(UC_X86_REG_EIP, ret_addr)
            return
        if addr == SLOT_PUSH:
            a = self._args(3)
            if len(self.slots) < 200:
                self.slots.append({"idx": a[0], "val": a[1], "kind": a[2]})
            elif len(self.slots) == 200:
                self.slots.append({"idx": -1, "val": -1, "kind": -1, "note": "truncated"})
            uc.reg_write(UC_X86_REG_EAX, 1); return
        if addr == OP_NEW or addr == GUI_ALLOC:
            n = self._args(1)[0] if addr == OP_NEW else 0x2000
            p = self.heap; self.heap += (n + 0xf) & ~0xf
            uc.reg_write(UC_X86_REG_EAX, p); return

    def run(self, state_writes, abs_writes=None):
        # Prime STATE with a non-zero pattern so pointer-deref chains don't
        # hit early-return NULL checks. Explicit state_writes override this.
        self.uc.mem_write(STATE, b"\x01" * min(STATE_SZ, 0x10000))
        for off, val, sz in state_writes:
            self.uc.mem_write(STATE + off, int(val).to_bytes(sz, "little"))
        for abs_addr, val, sz in (abs_writes or []):
            # Ensure page mapped; exe .data is already mapped by load_pe.
            page = abs_addr & ~0xfff
            if page not in self._mapped:
                try:
                    self.uc.mem_map(page, 0x1000); self._mapped.add(page)
                except UcError:
                    pass  # already mapped
            self.uc.mem_write(abs_addr, int(val).to_bytes(sz, "little", signed=(val < 0)))
        esp = STACK + STACK_SZ - 0x8000
        for i in range(8):
            self.uc.mem_write(esp + 4 + i * 4, struct.pack("<i", STATE))
        self.uc.mem_write(esp, struct.pack("<I", 0x1))
        self.uc.reg_write(UC_X86_REG_ESP, esp)
        self.uc.reg_write(UC_X86_REG_ECX, STATE)
        try:
            self.uc.emu_start(self.start, 0x1, count=2_000_000)
        except UcError as e:
            print(f"  emu note: {e} eip=0x{self.uc.reg_read(UC_X86_REG_EIP):x}")


def as_dict(name, cap):
    return {
        "screen": name,
        "areas":   [dict(id=aid, **{AREA_FIELDS[i]: a[i] for i in range(11)}) for aid, a in cap.areas],
        "objects": [dict(id=oid, **{GUIO_FIELDS[i]: a[i] for i in range(18)}) for oid, a in cap.objects],
        "tabs":    cap.tabs,
        "nav":     cap.nav,
        "slots":   cap.slots,
    }


def capture(name):
    spec = SCREENS[name]
    if spec["addr"] is None:
        print(f"[{name}] SKIP -- draw callback address not filled in. "
              f"Edit tools/capture_all_screens.py SCREENS[{name!r}].")
        return None
    print(f"[{name}] entry=0x{spec['addr']:08x} size=0x{spec['size']:x}")
    cap = Cap(spec["addr"], spec["size"])
    cap.run(spec.get("state", []), spec.get("abs", []))
    out = as_dict(name, cap)
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    dest = OUT_DIR / f"{name}.json"
    dest.write_text(json.dumps(out, indent=2))
    print(f"  areas={len(out['areas'])} objects={len(out['objects'])} tabs={len(out['tabs'])} nav={len(out['nav'])} slots={len(out['slots'])} -> {dest}")
    return dest


def main():
    ap = argparse.ArgumentParser(description="Emulate & capture a cm0102 screen's exact widget geometry.")
    ap.add_argument("screen", nargs="?", help="screen name (see --list)")
    ap.add_argument("--list", action="store_true", help="list known screens + status")
    ap.add_argument("--all", action="store_true", help="capture every screen whose addr is filled in")
    args = ap.parse_args()

    if args.list:
        ready = sum(1 for s in SCREENS.values() if s["addr"] is not None)
        print(f"{ready}/{len(SCREENS)} screens have draw-callback addresses filled in\n")
        for name, spec in SCREENS.items():
            marker = f"0x{spec['addr']:08x}" if spec["addr"] else "TBD"
            print(f"  {marker:>12}  {name}")
        return 0
    if args.all:
        rc = 0
        for name in SCREENS:
            try:
                capture(name)
            except Exception as e:
                print(f"[{name}] FAILED: {e}"); rc = 1
        return rc
    if not args.screen:
        ap.print_help(); return 2
    if args.screen not in SCREENS:
        print(f"unknown screen {args.screen!r}. Use --list."); return 2
    capture(args.screen)
    return 0


if __name__ == "__main__":
    sys.exit(main())
