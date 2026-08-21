#!/usr/bin/env python3
"""Symbolic executor: extract each GUI object's bounds as FORMULAS.

Abstract-interprets a builder, propagating expression trees (not values) through
registers and stack slots. At each construct call it reads the bound arguments as
formulas in terms of named inputs:
  body_top / body_bottom / body_ext   incoming screen-region args (>0x1000 stack)
  g[addr]                              a global / data read
  font                                 a graphics_font_row_height() result
  <reg>                                an unresolved input (e.g. a loop index)

Combined with the value extractor (colors/text/fonts) this yields, per object,
`bounds = formula(region, data, index)` + fill/text color + font + text — the
generation rule, not a frozen coordinate.
"""
import argparse, json, re
from pathlib import Path
import pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32
import extract_screen_tree as est   # reuse color table, font map, string reader

GUIO, AREA, FONTH = 0x00549580, 0x00549790, 0x005cf7b0
CLEANUP = {GUIO: 0x48, AREA: 0x2c}
BODY = {0x1220: "body_top", 0x1224: "body_bottom", 0x1228: "body_ext",
        0xc10: "body_top", 0xc18: "body_bottom"}

# ---- tiny expression algebra (int | str symbol | (op,a,b)) ----
def mk(op, a, b):
    if isinstance(a, int) and isinstance(b, int):
        if op == "+": return a + b
        if op == "-": return a - b
        if op == "*": return a * b
        if op == "/": return a // b if b else 0
        if op == "<<": return a << b
        if op == ">>": return a >> b
    if op == "+" and a == 0: return b
    if op == "+" and b == 0: return a
    if op == "-" and b == 0: return a
    if op == "*" and (a == 0 or b == 0): return 0
    if op == "*" and a == 1: return b
    if op == "*" and b == 1: return a
    if op == "<<" and b == 0: return a
    # algebraic: (a - b) + b = a ; (a + b) - b = a
    if op == "+" and isinstance(a, tuple) and a[0] == "-" and a[2] == b: return a[1]
    if op == "-" and isinstance(a, tuple) and a[0] == "+" and a[2] == b: return a[1]
    # fold nested (x + c1) + c2  ->  x + (c1+c2)
    if op in ("+", "-") and isinstance(b, int) and isinstance(a, tuple) and a[0] in ("+", "-") and isinstance(a[2], int):
        inner = a[2] if a[0] == "+" else -a[2]
        delta = b if op == "+" else -b
        c = inner + delta
        base = a[1]
        return mk("+", base, c) if c >= 0 else mk("-", base, -c)
    return (op, a, b)

def render(e):
    if isinstance(e, int): return str(e)
    if isinstance(e, str): return e
    op, a, b = e
    return f"({render(a)} {op} {render(b)})"

class Sym:
    def __init__(self, exe):
        self.exe = exe
        self.md = Cs(CS_ARCH_X86, CS_MODE_32)

    def operand(self, op, regs, stack):
        op = op.strip()
        if op.startswith("0x"): return int(op, 16)
        if re.fullmatch(r"-?\d+", op): return int(op)
        m = re.fullmatch(r"(?:dword|word|byte) ptr \[esp \+ (0x[0-9a-f]+)\]", op) or re.fullmatch(r"\[esp \+ (0x[0-9a-f]+)\]", op)
        if m:
            off = int(m.group(1), 16)
            if off in BODY: return BODY[off]
            if off > 0x1000 or off > 0xa00: return f"arg[{hex(off)}]"
            return stack.get(off, f"stk[{hex(off)}]")
        m = re.search(r"\[(0x[0-9a-f]+)\]", op)
        if m: return f"g[{m.group(1)}]"
        return regs.get(est._rk(op), op)   # register expression, default its name

    def mem_expr(self, memop, regs):
        # [base + index*scale + disp]  ->  expression
        inner = memop.strip()[1:-1]
        expr = 0
        for tok in re.findall(r"[+\-]?\s*[^+\-]+", inner):
            t = tok.replace(" ", "")
            neg = t.startswith("-"); t = t.lstrip("+-")
            mm = re.fullmatch(r"([a-z]+)\*(\d+)", t)
            if mm:
                part = mk("*", regs.get(est._rk(mm.group(1)), mm.group(1)), int(mm.group(2)))
            elif re.fullmatch(r"0x[0-9a-f]+", t): part = int(t, 16)
            elif re.fullmatch(r"\d+", t): part = int(t)
            elif re.fullmatch(r"[a-z]+", t): part = regs.get(est._rk(t), t)
            else: part = t
            expr = mk("-", expr, part) if neg else mk("+", expr, part)
        return expr

    def run(self, builder):
        data = self.exe.read(builder, est.fn_size(builder))
        regs, stack, pushes = {}, {}, []
        objects = []
        for ins in self.md.disasm(data, builder):
            m, ops = ins.mnemonic, ins.op_str
            p = [x.strip() for x in ops.split(",", 1)]
            if m in ("mov", "movsx", "movzx") and len(p) == 2:
                val = self.operand(p[1], regs, stack)
                sm = re.fullmatch(r"(?:dword|byte|word) ptr \[esp \+ (0x[0-9a-f]+)\]", p[0])
                if sm: stack[int(sm.group(1), 16)] = val
                else: regs[est._rk(p[0])] = val
            elif m == "lea" and len(p) == 2 and p[1].startswith("["):
                regs[est._rk(p[0])] = self.mem_expr(p[1], regs)
            elif m in ("add", "sub", "imul") and len(p) == 2:
                o = {"add": "+", "sub": "-", "imul": "*"}[m]
                regs[est._rk(p[0])] = mk(o, self.operand(p[0], regs, stack), self.operand(p[1], regs, stack))
            elif m in ("shl", "sal", "sar") and len(p) == 2:
                o = "<<" if m in ("shl", "sal") else ">>"
                regs[est._rk(p[0])] = mk(o, self.operand(p[0], regs, stack), self.operand(p[1], regs, stack))
            elif m == "xor" and len(p) == 2 and p[0] == p[1]:
                regs[est._rk(p[0])] = 0
            elif m in ("inc", "dec"):
                regs[est._rk(ops)] = mk("+" if m == "inc" else "-", regs.get(est._rk(ops), ops), 1)
            elif m == "idiv":
                regs["a"] = mk("/", regs.get("a", "eax"), "font")   # dividend/eax by font-ish divisor
            if m == "push":
                pushes.append(self.operand(ops, regs, stack))
            elif m == "call":
                tgt = int(ops, 16) if ops.startswith("0x") else None
                args = list(reversed(pushes))
                if tgt in (GUIO, AREA) and len(args) >= 5:
                    b = 0 if tgt == AREA else 1   # guio arg1 = type; area has no type arg
                    def A(i): return render(args[i]) if i < len(args) else None
                    objects.append({"call": f"{ins.address:#010x}", "kind": "area" if tgt == AREA else "guio",
                                    "left": A(b + 0), "top": A(b + 1), "right": A(b + 2), "bottom": A(b + 3),
                                    "col_count" if tgt == AREA else "col_slot": A(b + 4),
                                    "sort_y": A(b + 5) if tgt == GUIO else None})
                pushes = []
                for v in ("a", "c", "d"): regs.pop(v, None)
                if tgt == FONTH: regs["a"] = "font"
        return objects


def extract(builder):
    if not hasattr(extract, "_exe"): extract._exe = est.Exe()
    return {"builder": f"{builder:#010x}", "objects": Sym(extract._exe).run(builder)}


if __name__ == "__main__":
    ap = argparse.ArgumentParser(); ap.add_argument("builder"); ap.add_argument("--out")
    a = ap.parse_args()
    r = extract(int(a.builder, 16))
    print(json.dumps(r, indent=1)[:3000])
    if a.out: Path(a.out).write_text(json.dumps(r, indent=1))
