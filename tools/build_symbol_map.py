"""Consolidate every verified address->legible-name mapping we've established
(findings.json + lift markdown tables + decompiled-file header comments) into one
symbol table, so the FUN_xxxxxxxx decompiles can be renamed to something legible.
"""
import json, re, glob, os

ROOT = "D:/cm0102-rs"
CARVE = "D:/cm0102-carve"
pairs = {}   # 0xaddr(8) -> name

def norm(a):
    a = a.lower()
    if not a.startswith("0x"): a = "0x" + a
    return "0x%08x" % int(a, 16)

# single-word English fragments that leak in from decoded-argument tables
_NOISE = {"left","right","top","bottom","then","fans","have","coach","caller",
          "watch","competition","area","club_screens","width","height","color",
          "flags","font","text","parent_area_index","kind","payload"}

def add(addr, name):
    if not addr or not name: return
    if not re.fullmatch(r"[a-z_][a-z0-9_]{2,}", name): return
    if name.startswith("fun_") or name in _NOISE: return
    # real lifted symbol names are multi-token (verb_noun...). Require an underscore
    # to reject stray table-cell words; this keeps every curated name we have.
    if "_" not in name: return
    pairs.setdefault(norm(addr), name)

# 0) curated supplement: render-pipeline functions verified in the lift docs
#    (surface.py/panel.py/text.py headers + master-architecture), address-cited there.
for a, nm in {
    "0x007e6ee0": "view_model_get_slot",
    "0x007e7130": "view_model_set_slot",
    "0x005cf7b0": "graphics_font_row_height",
    "0x005cf8e0": "graphics_draw_panel",
    "0x005ced50": "graphics_blit_string",
    "0x005d0870": "graphics_draw_text_box",
    "0x005cf610": "graphics_text_width",
    "0x005cdfd0": "graphics_dim_region_to_pct",
    "0x005cd840": "graphics_fill_rect",
    "0x0059bed0": "graphics_font_width_fixed16",
}.items():
    add(a, nm)

# 1) findings.json: names live under maps keyed BY ADDRESS, e.g.
#    verified: { "0x008fc4f0": {"name": "match_random", ...}, ... }
fj = json.load(open(f"{CARVE}/findings.json"))
def walk(o):
    if isinstance(o, dict):
        # address-keyed entry: {"0x....": {"name": ...}}
        for k, v in o.items():
            if re.fullmatch(r"0x[0-9a-fA-F]{6,8}", str(k)) and isinstance(v, dict):
                nm = v.get("name") or v.get("verified_name") or v.get("symbol")
                if isinstance(nm, str): add(k, nm)
        # or a self-describing object with both fields
        addr = None
        for k in ("address", "addr", "ea", "func", "function", "va", "entry"):
            val = o.get(k)
            if isinstance(val, str) and re.fullmatch(r"0x[0-9a-fA-F]{6,8}", val):
                addr = val
        nm = o.get("name") or o.get("verified_name") or o.get("symbol") or o.get("rename")
        if addr and isinstance(nm, str): add(addr, nm)
        for v in o.values(): walk(v)
    elif isinstance(o, list):
        for v in o: walk(v)
walk(fj)

# 2) lift markdown tables: | `0x....` | `name` | ...   and  0x.... `name`
for md in glob.glob(f"{ROOT}/reports/**/*.md", recursive=True):
    try: t = open(md, encoding="utf-8").read()
    except Exception: continue
    for m in re.finditer(r"`?(0x[0-9a-fA-F]{6,8})`?\s*\|\s*`?([a-z_][a-z0-9_]{2,})`?", t):
        add(m.group(1), m.group(2))
    for m in re.finditer(r"`(0x[0-9a-fA-F]{6,8})`\s+`?([a-z_][a-z0-9_]{3,})`?", t):
        add(m.group(1), m.group(2))

# 3) decompiled file header comments: "// 0xADDR  verified_name  (...)"
for c in glob.glob(f"{CARVE}/decompiled/**/0x*.c", recursive=True):
    try: first = open(c, encoding="utf-8").readline()
    except Exception: continue
    m = re.match(r"//\s*(0x[0-9a-fA-F]{6,8})\s+([A-Za-z_][A-Za-z0-9_]+)", first)
    if m and not m.group(2).upper().startswith("FUN_"):
        add(m.group(1), m.group(2).lower())

os.makedirs(f"{ROOT}/reports/carve_segment_index/fixtures", exist_ok=True)
out = f"{ROOT}/reports/carve_segment_index/fixtures/symbol_names.json"
json.dump(pairs, open(out, "w"), indent=1, sort_keys=True)
print("total mapped names:", len(pairs), "->", out)
for a in sorted(pairs)[:25]:
    print(" ", a, pairs[a])
