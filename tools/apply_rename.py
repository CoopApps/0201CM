"""Apply the consolidated symbol map to every decompiled .c file, producing a
legible copy tree where FUN_<addr> / 0x<addr>(...) references become named.

Writes to D:/cm0102-carve/decompiled_renamed/ mirroring the source layout, plus a
_symbol_map.txt index. Non-destructive: originals are untouched.
"""
import json, re, glob, os, shutil

CARVE = "D:/cm0102-carve"
SRC = f"{CARVE}/decompiled"
DST = f"{CARVE}/decompiled_renamed"
names = json.load(open("D:/cm0102-rs/reports/carve_segment_index/fixtures/symbol_names.json"))

# name lookups keyed by both 8-digit and bare-hex forms of the address
by_hex = {}
for a, nm in names.items():
    h = a[2:].lower().lstrip("0") or "0"
    by_hex[h] = nm
    by_hex[a[2:].lower()] = nm

def repl_fun(m):
    h = m.group(1).lower().lstrip("0") or "0"
    nm = by_hex.get(h) or by_hex.get(m.group(1).lower())
    return nm if nm else m.group(0)

FUN_RE = re.compile(r"FUN_0*([0-9a-fA-F]{5,8})")

if os.path.isdir(DST): shutil.rmtree(DST)
os.makedirs(DST, exist_ok=True)

renamed_files = 0; total_subs = 0
for c in glob.glob(f"{SRC}/**/0x*.c", recursive=True):
    rel = os.path.relpath(c, SRC)
    txt = open(c, encoding="utf-8", errors="replace").read()
    new, n = FUN_RE.subn(repl_fun, txt)
    total_subs += n
    # rename the file itself if its entry address is known
    m = re.match(r"0x0*([0-9a-fA-F]+)\.c$", os.path.basename(c))
    outname = os.path.basename(c)
    if m:
        h = m.group(1).lower()
        nm = by_hex.get(h) or by_hex.get(h.zfill(8))
        if nm: outname = f"{nm}__{os.path.basename(c)}"; renamed_files += 1
    dst = os.path.join(DST, os.path.dirname(rel), outname)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    open(dst, "w", encoding="utf-8").write(new)

with open(f"{DST}/_symbol_map.txt", "w", encoding="utf-8") as f:
    for a in sorted(names):
        f.write(f"{a}  {names[a]}\n")

print(f"files processed, named entries={len(names)}, files given named prefix={renamed_files}")
print(f"FUN_ references rewritten to legible names: {total_subs}")
print(f"-> {DST}")
