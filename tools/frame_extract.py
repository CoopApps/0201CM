"""Disassemble a VA range of cm0102.exe and, at every call to the GUI object/area
constructors, dump the preceding PUSH sequence (the args) with immediates resolved.
Used to lift band/tab/banner geometry that isn't in the 375 pre-decompiled functions.
"""
import struct, sys
import capstone

EXE = "D:/cm0102/cm0102.exe"
IB = 0x400000
GUIO = 0x00549580   # display_create_guio_object (18 args)
AREA = 0x00549790   # display_create_area_or_emit_area_packet (11 args)

exe = open(EXE, "rb").read()
pe = struct.unpack_from("<I", exe, 0x3c)[0]
nsec = struct.unpack_from("<H", exe, pe + 6)[0]
opt = struct.unpack_from("<H", exe, pe + 20)[0]
so = pe + 24 + opt
secs = []
for i in range(nsec):
    o = so + i * 40
    va = struct.unpack_from("<I", exe, o + 12)[0] + IB
    raw = struct.unpack_from("<I", exe, o + 20)[0]
    rs = struct.unpack_from("<I", exe, o + 16)[0]
    secs.append((va, raw, rs))

def va2off(va):
    for v, r, rs in secs:
        if v <= va < v + rs:
            return r + (va - v)
    return None

def disasm(start, end):
    off = va2off(start)
    code = exe[off:off + (end - start)]
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
    md.detail = True
    return list(md.disasm(code, start))

def extract(start, end, label=""):
    insns = disasm(start, end)
    # map for quick arg back-walk: collect a rolling list of recent PUSH values
    pushes = []   # (addr, value_or_regstr)
    out = []
    for ins in insns:
        m = ins.mnemonic
        if m == "push":
            op = ins.op_str
            if op.startswith("0x") or op.lstrip("-").isdigit():
                try: v = int(op, 0)
                except ValueError: v = op
            else:
                v = op  # register or mem
            pushes.append(v)
        elif m == "call":
            tgt = ins.op_str
            try: t = int(tgt, 0)
            except ValueError: t = None
            if t in (GUIO, AREA):
                n = 18 if t == GUIO else 11
                args = pushes[-n:][::-1] if len(pushes) >= 1 else pushes[::-1]
                kind = "GUIO" if t == GUIO else "AREA"
                out.append((hex(ins.address), kind, args))
            pushes = []   # args consumed
        else:
            # any non-push between pushes can still feed a register arg; keep pushes
            # (regs shown as raw names). Only clear on call.
            pass
    return out

def show(start, end):
    for addr, kind, args in extract(start, end):
        if kind == "GUIO":
            names = ["type", "L", "T", "R", "B", "a6", "a7", "rflags", "colP", "colS",
                     "txtflags", "font", "tmode", "text", "a15", "a16", "a17", "parent"]
        else:
            names = ["L", "T", "R", "B", "cntA", "wA", "cntB", "wB", "flags", "a10", "parent"]
        d = {names[i]: args[i] for i in range(min(len(names), len(args)))}
        key = {k: d.get(k) for k in (("L","T","R","B","rflags","font","parent") if kind=="GUIO"
                                     else ("L","T","R","B","cntB","flags","parent"))}
        print(f"  {addr} {kind:4} {key}")

if __name__ == "__main__":
    a = int(sys.argv[1], 0); b = int(sys.argv[2], 0)
    print(f"== {hex(a)}..{hex(b)} ==")
    show(a, b)
