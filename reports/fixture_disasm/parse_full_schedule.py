"""Parse ALL 134 calls to FUN_0066f3b0 in eng_second's schedule-getter and
resolve every argument, including register-derived args.

The pattern for each call is:
    push arg9
    push arg8      (usually word ptr [edi+0x40] = year, or eax after mov)
    push arg7
    push arg6
    push arg5
    push arg4      (month, 0..11)
    push arg3      (day, 1..31)
    push arg2      (round_idx, but sometimes register — resolve where possible)
    push arg1      (buffer = esi, or fresh alloc)
    call 0x66f3b0

We walk backward from each call, capturing the 9 push instructions and
resolving 3 kinds of pushes:
  - immediate int:  push 0xNN or push -1
  - register:       push reg — leave the register name symbolic
  - memory:         push [reg+offset] — leave symbolic

We also track `mov <reg>, <imm>` and `mov <reg>, [mem]` in the previous
few instructions so we can resolve which value the register held at the
time of the push.

Then we sort by round_idx and produce a table.
"""
import struct, re
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

EXE = Path(r"D:\cm0102\cm0102.exe")
OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm")

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
            return s["ro"] + (rva - s["va"])
    return None

md = Cs(CS_ARCH_X86, CS_MODE_32); md.detail = True
off = va_to_off(0x0055f340)
blob = exe[off : off + 0x4000]
insns = list(md.disasm(blob, 0x0055f340))

# Track a simple constant register tracker as we walk forward
def parse_int(op):
    op = op.strip()
    try:
        if op.startswith("0x"): return int(op, 16)
        if op == "-1": return -1
        if op.isdigit(): return int(op)
        return None
    except:
        return None

def sign_ext(n, bits=32):
    if n is None: return None
    if n < 0: return (n + (1 << bits)) & ((1 << bits) - 1)
    return n & ((1 << bits) - 1)

# Emulate a very small subset of x86 for constant tracking within short windows
class SimpleTracker:
    def __init__(self):
        self.regs = {r: None for r in ["eax","ebx","ecx","edx","esi","edi","ebp","esp","al","ax","cl","cx","dl","dx"]}
    def step(self, insn):
        m = insn.mnemonic
        op = insn.op_str
        if m == "xor" and "," in op:
            a, b = [x.strip() for x in op.split(",", 1)]
            if a == b and a in self.regs:
                self.regs[a] = 0
        elif m == "mov" and "," in op:
            a, b = [x.strip() for x in op.split(",", 1)]
            if a in self.regs:
                v = parse_int(b)
                if v is not None:
                    self.regs[a] = v
                elif b in self.regs:
                    self.regs[a] = self.regs.get(b)
                else:
                    self.regs[a] = None
        elif m == "movzx" and "," in op:
            a, b = [x.strip() for x in op.split(",", 1)]
            if a in self.regs:
                # dest = zero-extended memory or reg — usually can't resolve
                self.regs[a] = None
        elif m == "movsx" and "," in op:
            a, b = [x.strip() for x in op.split(",", 1)]
            if a in self.regs:
                self.regs[a] = None
        elif m == "lea" and "," in op:
            a = op.split(",", 1)[0].strip()
            if a in self.regs:
                self.regs[a] = None
        elif m in ("add","sub","inc","dec","imul","shl","shr","or","and"):
            # invalidate destination
            first = op.split(",")[0].strip() if "," in op else op.strip()
            if first in self.regs:
                self.regs[first] = None

# Collect all calls to FUN_0066f3b0
tgt = 0x0066f3b0
call_idxs = []
for i, ins in enumerate(insns):
    if ins.mnemonic == "call":
        try:
            if int(ins.op_str, 16) == tgt:
                call_idxs.append(i)
        except: pass

print(f"{len(call_idxs)} calls to FUN_0066f3b0 in eng_second schedule-getter")

# For each call, walk backwards through pushes (last 30 insns) and forward-track
# regs from the start of that window. If we get 9 pushes, decode arg1..arg9.
def resolve_for_call(call_idx):
    lo = max(0, call_idx - 30)
    tracker = SimpleTracker()
    pushes = []       # (addr, resolved_int_or_None, symbolic_str)
    for j in range(lo, call_idx):
        ins = insns[j]
        if ins.mnemonic == "push":
            op = ins.op_str.strip()
            v = parse_int(op)
            if v is not None:
                pushes.append((ins.address, v, op))
            elif op in tracker.regs:
                pushes.append((ins.address, tracker.regs[op], op))
            else:
                pushes.append((ins.address, None, op))
        else:
            tracker.step(ins)
    # Keep only the LAST 9 pushes before the call (the actual args)
    pushes = pushes[-9:]
    return pushes

rounds = []
for k, ci in enumerate(call_idxs):
    ps = resolve_for_call(ci)
    if len(ps) != 9:
        continue
    # arg1 = last push (topmost on stack)
    # pushes[-1] is the LAST push before the call = arg1
    arg = {}
    for a_idx, (addr, val, sym) in enumerate(reversed(ps)):
        arg[f"arg{a_idx+1}"] = (val, sym)
    # extract
    round_idx = arg["arg2"][0]
    day       = arg["arg3"][0]
    month     = arg["arg4"][0]
    day_off   = arg["arg5"][0]
    flag      = arg["arg6"][0]
    typebyte  = arg["arg7"][0]
    year_reg  = arg["arg8"][1]
    prize     = arg["arg9"][0]
    rounds.append((k, insns[ci].address, round_idx, day, month, day_off, flag, typebyte, year_reg, prize))

# Print header
print(f"\n{'call':>4}  {'call_va':>10}  {'round':>5}  {'day':>3}  {'mon':>3}  {'dayoff':>6}  {'flag':>4}  {'type':>4}  {'year':<8}  prize")
for row in rounds[:20]:
    k, va, ri, d, m, df, fl, tb, yr, pr = row
    ri_s = str(ri) if ri is not None else "?"
    d_s  = str(d) if d is not None else "?"
    m_s  = str(m) if m is not None else "?"
    df_s = str(df) if df is not None else "?"
    fl_s = str(fl) if fl is not None else "?"
    tb_s = str(tb) if tb is not None else "?"
    pr_s = str(pr) if pr is not None else "?"
    print(f"{k:>4}  {va:#010x}  {ri_s:>5}  {d_s:>3}  {m_s:>3}  {df_s:>6}  {fl_s:>4}  {tb_s:>4}  {yr:<8}  {pr_s}")

# Write full table
lines = []
lines.append("# All FUN_0066f3b0 calls in eng_second schedule-getter 0x0055f340")
lines.append(f"# {len(rounds)}/{len(call_idxs)} fully resolved")
lines.append(f"# {'call':>4}  {'call_va':>10}  {'round':>5}  {'day':>3}  {'mon':>3}  {'dayoff':>6}  {'flag':>4}  {'type':>4}  {'year':<8}  prize")
for row in rounds:
    k, va, ri, d, m, df, fl, tb, yr, pr = row
    ri_s = str(ri) if ri is not None else "?"
    d_s  = str(d) if d is not None else "?"
    m_s  = str(m) if m is not None else "?"
    df_s = str(df) if df is not None else "?"
    fl_s = str(fl) if fl is not None else "?"
    tb_s = str(tb) if tb is not None else "?"
    pr_s = str(pr) if pr is not None else "?"
    lines.append(f"{k:>4}  {va:#010x}  {ri_s:>5}  {d_s:>3}  {m_s:>3}  {df_s:>6}  {fl_s:>4}  {tb_s:>4}  {yr:<8}  {pr_s}")

(OUT / "eng_second_full_schedule.txt").write_text("\n".join(lines), encoding="utf-8")
print(f"\nwrote {OUT/'eng_second_full_schedule.txt'} ({len(rounds)} rows)")
