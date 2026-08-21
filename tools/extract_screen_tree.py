#!/usr/bin/env python3
"""guio object-tree extractor.

Given a CM0102 screen-builder function address, disassemble it, walk the GUI
construct calls, and emit the renderable object tree: bounds + resolved fill/text
colors + font + text, per object. Automates the manual RE of a screen.

Construct calls recognised:
  0x00549580  display_create_guio_object   (18 stack args, __thiscall)
  0x00549790  display_create_area          (11 args)
  0x006547c0  set_label                    (copies a string into the label buffer)

Field mapping is the definitive one from guio_init_object_record (0x005d7bd0),
mirrored in tools/decode_ui_constructor_args.py:
  arg1 type | arg2 left | arg3 top | arg4 right | arg5 bottom | arg6 field20
  arg7 field24 | arg8 render_flags(+0x38) | arg9 color_primary(+0x72, FILL)
  arg10 color_secondary(+0x74) | arg11 text_box_flags(+0x3c) | arg12 font_slot(+0x76)
  arg13 text_mode(+0x78, TEXT COLOR) | arg14 text_ptr(+0x80) | ...
"""
import argparse, json
from pathlib import Path
import pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

ROOT = Path("D:/cm0102-rs")
EXE = "D:/cm0102/cm0102.exe"
GHIDRA_FNS = "D:/cm0102-carve/ghidra_out/cm0102.exe/functions.json"
UI_CONSTS = ROOT / "reports/carve_segment_index/fixtures/ui_constants.json"

GUIO_CREATE = 0x00549580
AREA_CREATE = 0x00549790
SET_LABEL = 0x006547c0

GUI_ARGS = ["type", "left", "top", "right", "bottom", "field20", "field24",
            "render_flags", "color_primary", "color_secondary", "text_box_flags",
            "font_slot", "text_mode", "text_ptr", "a15", "a16", "a17", "parent_area"]
FONT_SLOT = {0: "arial_narrow_10", 1: "arial_narrow_10", 2: "arial_narrow_11",
             3: "arial_14", 4: "arial_16", 5: "arial_18"}


def rgb565(v):
    return (round(((v >> 11) & 0x1f) * 255 / 31), round(((v >> 5) & 0x3f) * 255 / 63),
            round((v & 0x1f) * 255 / 31))


def load_color_table():
    """global-address(int) -> (name, rgb). Includes format-dependent globals
    resolved via their RGB565 packed value (0x005ce250 color init, 565 mode)."""
    t = {}
    d = json.load(open(UI_CONSTS))
    for c in d["colors"]:
        g = c.get("global")
        if not g:
            continue
        addr = int(g.replace("DAT_", ""), 16)
        if c.get("rgb"):
            t[addr] = (c["name"], (c["rgb"]["r"], c["rgb"]["g"], c["rgb"]["b"]))
    # format-dependent globals (packed RGB565 from 0x005ce250, mask term = 0 in 565 mode)
    for addr, packed, name in [(0x00acdf92, 0xffff, "white"), (0x00acdf74, 0xe71c, "yellowish_highlight"),
                               (0x00ad6bdc, 0xffe0, "near_white"), (0x00ad6bde, 0xfff0, "near_white_alt"),
                               (0x00ad6bbc, 0x8000, "format_red"), (0x00ad6bc4, 0xfc00, "format_orange"),
                               (0x00ad6bda, 0x0000, "black")]:
        t.setdefault(addr, (name, rgb565(packed)))
    return t


class Exe:
    def __init__(self):
        self.pe = pefile.PE(EXE, fast_load=True)
        self.ib = self.pe.OPTIONAL_HEADER.ImageBase

    def read(self, va, n):
        rva = va - self.ib
        for s in self.pe.sections:
            if s.VirtualAddress <= rva < s.VirtualAddress + max(s.Misc_VirtualSize, s.SizeOfRawData):
                o = rva - s.VirtualAddress
                return s.get_data()[o:o + n]
        return b""

    def cstr(self, va):
        b = self.read(va, 80)
        z = b.find(b"\0")
        s = b[:z if z >= 0 else 80].decode("latin-1", "replace")
        return s if all(32 <= ord(c) < 127 for c in s) else None


_FN_SIZES = None
def fn_size(addr):
    global _FN_SIZES
    if _FN_SIZES is None:
        _FN_SIZES = {int(f["entry"], 16): f["size"] for f in json.load(open(GHIDRA_FNS))}
    return _FN_SIZES.get(addr, 0x1000)


_EXE = None
_COLORS = None
def extract(builder_addr):
    global _EXE, _COLORS
    if _EXE is None:
        _EXE = Exe()
    if _COLORS is None:
        _COLORS = load_color_table()
    exe = _EXE
    colors = _COLORS
    size = fn_size(builder_addr)
    data = exe.read(builder_addr, size)
    md = Cs(CS_ARCH_X86, CS_MODE_32)
    regs = {}          # reg -> ("imm", val) | ("glob", addr) | ("stack", off)
    pending_pushes = []  # list of resolved arg descriptors, in push order
    last_label = None    # most recent set_label string
    objects = []

    def resolve_color(arg):
        if arg["kind"] == "glob" and arg["val"] in colors:
            n, rgb = colors[arg["val"]]
            return {"name": n, "rgb": list(rgb)}
        if arg["kind"] == "imm" and arg["val"] in colors:
            n, rgb = colors[arg["val"]]
            return {"name": n, "rgb": list(rgb)}
        return {"raw": arg}

    for ins in md.disasm(data, builder_addr):
        m, ops = ins.mnemonic, ins.op_str
        # light register tracking
        if m in ("mov", "movsx", "movzx"):
            parts = [p.strip() for p in ops.split(",", 1)]
            if len(parts) == 2:
                dst, src = parts
                import re
                if src.startswith("0x"):
                    regs[_rk(dst)] = ("imm", int(src, 16))
                elif "[0x" in src:
                    mm = re.search(r"\[(0x[0-9a-fA-F]+)\]", src)
                    regs[_rk(dst)] = ("glob", int(mm.group(1), 16)) if mm else regs.pop(_rk(dst), None)
                else:
                    regs.pop(_rk(dst), None)
        elif m == "xor":
            parts = [p.strip() for p in ops.split(",", 1)]
            if len(parts) == 2 and parts[0] == parts[1]:
                regs[_rk(parts[0])] = ("imm", 0)
        elif m == "push":
            if ops.startswith("0x"):
                v = int(ops, 16)
                s = exe.cstr(v)
                pending_pushes.append({"kind": "imm", "val": v, "str": s})
            else:
                rk = _rk(ops)
                if rk in regs:
                    k, val = regs[rk]
                    pending_pushes.append({"kind": k, "val": val, "reg": ops})
                else:
                    pending_pushes.append({"kind": "reg", "reg": ops})
        elif m == "call":
            tgt = None
            if ops.startswith("0x"):
                tgt = int(ops, 16)
            args = list(reversed(pending_pushes))  # arg1 = last pushed
            if tgt == SET_LABEL:
                for a in pending_pushes:
                    if a.get("str"):
                        last_label = a["str"]
            elif tgt == GUIO_CREATE and len(args) >= 13:
                named = {GUI_ARGS[i]: args[i] for i in range(min(len(args), len(GUI_ARGS)))}
                obj = {
                    "call_addr": f"{ins.address:#010x}",
                    "left": _val(named.get("left")), "top": _val(named.get("top")),
                    "right": _val(named.get("right")), "bottom": _val(named.get("bottom")),
                    "render_flags": _val(named.get("render_flags")),
                    "fill_color": resolve_color(named["color_primary"]) if "color_primary" in named else None,
                    "text_color": resolve_color(named["text_mode"]) if "text_mode" in named else None,
                    "font_slot": _val(named.get("font_slot")),
                    "font": FONT_SLOT.get(_val(named.get("font_slot")) if isinstance(_val(named.get("font_slot")), int) else -1, None),
                    "text": (named.get("text_ptr", {}) or {}).get("str") or last_label,
                }
                objects.append(obj)
            pending_pushes = []
            for v in ("a", "c", "d"):  # calls clobber only the volatile regs; keep b/si/di
                regs.pop(v, None)
    return {"builder": f"{builder_addr:#010x}", "object_count": len(objects), "objects": objects}


def _rk(reg):
    # normalise 16/32-bit reg names to a common key (eax/ax -> a, ecx/cx -> c ...)
    r = reg.strip().lower()
    for full, k in [("eax", "a"), ("ax", "a"), ("al", "a"), ("ecx", "c"), ("cx", "c"), ("cl", "c"),
                    ("edx", "d"), ("dx", "d"), ("ebx", "b"), ("bx", "b"), ("esi", "si"), ("edi", "di")]:
        if r == full:
            return k
    return r


def _val(arg):
    if not arg:
        return None
    if arg.get("kind") == "imm":
        return arg["val"]
    return arg.get("reg") or arg.get("raw") or "unresolved"


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("builder", help="builder function address, e.g. 0x00495ad0")
    ap.add_argument("--out")
    a = ap.parse_args()
    tree = extract(int(a.builder, 16))
    js = json.dumps(tree, indent=1)
    if a.out:
        Path(a.out).write_text(js)
        print("wrote", a.out, "-", tree["object_count"], "objects")
    else:
        print(js)
