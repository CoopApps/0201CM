#!/usr/bin/env python3
"""Layout-resolution pass: turn area-grid children's runtime bounds into pixels.

CM0102 lays grid children out via area_rebuild_layout_tables (0x00403390):
  inset = 0 if area_flags&1 else 2
  E = (right - 2*inset - left) + 1     (minus 0x15 if a scrollbar is present)
  S = sum(column_weight_bytes)
  left[0]=inset ; left[i]=right[i-1]+2
  right[i]=(cum_i*E)/S - 1 + inset   (i<n-1) ; right[n-1]=E-1+inset
  child x = area.left + column_left_offsets[child.column_slot] .. +right_offsets[...]

The column weights are immediate byte-stores to a stack buffer; the area's weight
pointer is `lea reg,[esp+X]`. Matching them requires tracking ESP through pushes,
so both refer to the same frame offset. This pass disassembles a builder with ESP
tracking, captures areas (0x549790) + their weight arrays + grid children
(0x549580 with column_slot), and computes each child's horizontal pixel bounds.
"""
import argparse, json, re
from pathlib import Path
import pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

EXE = "D:/cm0102/cm0102.exe"
GHIDRA_FNS = "D:/cm0102-carve/ghidra_out/cm0102.exe/functions.json"
GUIO_CREATE, AREA_CREATE = 0x00549580, 0x00549790

# area_create arg order (1-based): left top right bottom col_count weight_ptr row_count ? flags ...
# guio arg order: type left top right bottom col_slot sort_y flags color_pri color_sec tbf font tmode text
_pe = pefile.PE(EXE, fast_load=True); _ib = _pe.OPTIONAL_HEADER.ImageBase
def read(va, n):
    rva = va - _ib
    for s in _pe.sections:
        if s.VirtualAddress <= rva < s.VirtualAddress + max(s.Misc_VirtualSize, s.SizeOfRawData):
            o = rva - s.VirtualAddress; return s.get_data()[o:o + n]
    return b""
_sizes = None
def fn_size(a):
    global _sizes
    if _sizes is None:
        _sizes = {int(f["entry"],16): f["size"] for f in json.load(open(GHIDRA_FNS))}
    return _sizes.get(a, 0x1000)

def compute_columns(weights, left, right, flags, scrollbar=False):
    inset = 0 if (flags & 1) else 2
    iv8 = (right - 2*inset) - left
    E = (iv8 - 0x15) if scrollbar else iv8 + 1
    S = sum(weights) or 1
    n = len(weights); C = 0; out = []; pr = None
    for i, w in enumerate(weights):
        C += w
        l = inset if i == 0 else pr + 2
        r = (E-1+inset) if i == n-1 else (C*E)//S - 1 + inset
        out.append((left+l, left+r)); pr = r
    return out

def resolve(builder_addr):
    data = read(builder_addr, fn_size(builder_addr))
    md = Cs(CS_ARCH_X86, CS_MODE_32); md.detail = False
    # callee-cleanup bytes for known __thiscall/__stdcall targets (they pop their
    # own stack args, with no `add esp` at the callsite). cdecl calls are handled
    # by their explicit `add esp`.
    CALLEE_CLEANUP = {GUIO_CREATE: 0x48, AREA_CREATE: 0x2c}
    esp = 0                       # delta from function entry (push -> -4)
    byte_stores = []              # (esp-relative offset, value) in program order
    regs = {}                     # reg -> ("imm",v)|("frameptr",frame_off)|("glob",a)
    pushes = []                   # (value_desc) in push order for current call
    areas = []; children = []
    for ins in md.disasm(data, builder_addr):
        m, ops = ins.mnemonic, ins.op_str
        # ---- ESP tracking ----
        if m == "push": esp -= 4
        elif m == "pop": esp += 4
        elif m == "sub" and ops.startswith("esp,"): esp -= int(ops.split(",")[1].strip(), 16)
        elif m == "add" and ops.startswith("esp,"): esp += int(ops.split(",")[1].strip(), 16)
        # ---- stack byte stores (weight arrays) ----
        # A weight array is a run of byte-immediate stores to CONSECUTIVE [esp+X]
        # offsets. Collect (offset,value) so runs can be grouped by length below;
        # far more robust than exact esp/frame matching across calling conventions.
        if m == "mov" and ops.startswith("byte ptr [esp"):
            mm = re.match(r"byte ptr \[esp \+ (0x[0-9a-f]+)\], (0x[0-9a-f]+|\d+|[a-z]+)", ops)
            if mm:
                off = int(mm.group(1), 16)
                v = mm.group(2)
                if v.startswith("0x"):
                    val = int(v, 16)
                elif v.isdigit():
                    val = int(v)
                else:  # a register byte (e.g. bl from xor ebx,ebx -> 0)
                    r = regs.get(_rk(v))
                    val = r[1] if r and r[0] == "imm" else 0
                byte_stores.append((off, val))
        # ---- register tracking (imm / frame ptr via lea / global) ----
        if m in ("mov", "movsx", "movzx"):
            p = [x.strip() for x in ops.split(",", 1)]
            if len(p) == 2:
                dst, src = p
                if src.startswith("0x"): regs[_rk(dst)] = ("imm", int(src, 16))
                elif "[0x" in src:
                    g = re.search(r"\[(0x[0-9a-f]+)\]", src); regs[_rk(dst)] = ("glob", int(g.group(1),16)) if g else None
                else: regs.pop(_rk(dst), None)
        elif m == "xor":
            p = [x.strip() for x in ops.split(",",1)]
            if len(p)==2 and p[0]==p[1]: regs[_rk(p[0])] = ("imm", 0)
        elif m == "lea":
            p = [x.strip() for x in ops.split(",",1)]
            if len(p)==2 and p[1].startswith("[esp"):
                mm = re.match(r"\[esp \+ (0x[0-9a-f]+)\]", p[1])
                if mm: regs[_rk(p[0])] = ("frameptr", esp + int(mm.group(1),16))
        # ---- push args ----
        if m == "push":
            if ops.startswith("0x"): pushes.append(("imm", int(ops,16)))
            else: pushes.append(regs.get(_rk(ops), ("reg", ops)))
        elif m == "call":
            tgt = int(ops,16) if ops.startswith("0x") else None
            args = list(reversed(pushes))
            def ai(i): return args[i] if i < len(args) else ("none",None)
            if tgt == AREA_CREATE and len(args) >= 9:
                areas.append({"call": f"{ins.address:#010x}",
                    "left": _iv(ai(0)), "top": _iv(ai(1)), "right": _iv(ai(2)), "bottom": _iv(ai(3)),
                    "col_count": _iv(ai(4)), "weight_ptr": ai(5), "flags": _iv(ai(8))})
            elif tgt == GUIO_CREATE and len(args) >= 7:
                children.append({"call": f"{ins.address:#010x}",
                    "explicit_left": _iv(ai(1)), "explicit_right": _iv(ai(3)),
                    "column_slot": _iv(ai(5)), "sort_y": _iv(ai(6))})
            pushes = []
            for v in ("a","c","d"): regs.pop(v, None)
            esp += CALLEE_CLEANUP.get(tgt, 0)   # callee popped its stack args
    # ---- group byte-stores into runs of consecutive [esp+X] offsets ----
    runs = []
    cur = []
    for off, val in byte_stores:
        if cur and off == cur[-1][0] + 1:
            cur.append((off, val))
        else:
            if len(cur) >= 2:
                runs.append(cur)
            cur = [(off, val)]
    if len(cur) >= 2:
        runs.append(cur)

    # ---- match each area to a weight-run of length == col_count, in order ----
    resolved = []
    used = set()
    for a in areas:
        cc = a["col_count"]
        if not isinstance(cc, int) or not (1 <= cc <= 30):
            continue
        if not isinstance(a["left"], int) or not isinstance(a["right"], int):
            continue
        match = next((i for i, r in enumerate(runs) if i not in used and len(r) == cc), None)
        if match is None:
            continue
        used.add(match)
        weights = [v for _, v in runs[match]]
        flags = a["flags"] if isinstance(a["flags"], int) else 0
        cols = compute_columns(weights, a["left"], a["right"], flags)
        a["weights"] = weights
        a["columns"] = cols
        resolved.append(a)
    return {"builder": f"{builder_addr:#010x}", "areas": areas, "resolved_areas": resolved,
            "child_count": len(children)}

def _rk(r):
    r = r.strip().lower()
    return {"eax":"a","ax":"a","al":"a","ecx":"c","cx":"c","cl":"c","edx":"d","dx":"d",
            "ebx":"b","bx":"b","bl":"b","esi":"si","edi":"di"}.get(r, r)
def _iv(a):
    return a[1] if a and a[0]=="imm" else (a[1] if a and a[0]=="reg" else None)

if __name__ == "__main__":
    ap = argparse.ArgumentParser(); ap.add_argument("builder"); ap.add_argument("--out")
    a = ap.parse_args()
    r = resolve(int(a.builder, 16))
    print(json.dumps(r, indent=1)[:2500])
    if a.out: Path(a.out).write_text(json.dumps(r, indent=1))
