"""Map cm0102.exe VAs to cm0102_GDI.exe VAs.

Strategy: for each candidate function in cm0102.exe (e.g. eng_second_ctor
at 0x0055f040), extract its first 40 bytes, find that byte pattern in
cm0102_GDI.exe's .text, and report the GDI VA.

Also print anchor addresses:
- String literal 's_C__dev_CM3_00_01_cm3_code_comp_l_009dd618' in .rdata
- DAT_009bba80 initial value (for both builds)
- Cross-referencing xrefs.json for common calls
"""
import struct, json
from pathlib import Path

DD  = Path(r"D:\cm0102\cm0102.exe")
GDI = Path(r"D:\cm0102\cm0102_GDI.exe")
IMAGE_BASE = 0x00400000

def parse_pe(b):
    e = struct.unpack_from("<I", b, 0x3c)[0]; coff = e + 4
    n = struct.unpack_from("<H", b, coff+2)[0]
    opt = struct.unpack_from("<H", b, coff+16)[0]; off = coff + 20
    ib = struct.unpack_from("<I", b, off+28)[0]; so = off + opt
    secs = {}
    for i in range(n):
        base = so + i*40
        name = b[base:base+8].rstrip(b"\0").decode('latin1','replace')
        secs[name] = dict(
            va=struct.unpack_from("<I", b, base+12)[0],
            ro=struct.unpack_from("<I", b, base+20)[0],
            rsz=struct.unpack_from("<I", b, base+16)[0],
            vsz=struct.unpack_from("<I", b, base+8)[0])
    return ib, secs

def va_to_off(secs, ib, va):
    rva = va - ib
    for s in secs.values():
        if s["va"] <= rva < s["va"] + s["rsz"]:
            return s["ro"] + (rva - s["va"])
    return None

def off_to_va(secs, ib, off):
    for s in secs.values():
        if s["ro"] <= off < s["ro"] + s["rsz"]:
            return ib + s["va"] + (off - s["ro"])
    return None

def load(p):
    b = p.read_bytes()
    ib, secs = parse_pe(b)
    return b, ib, secs

def find_pattern(b, section, pat):
    """Find `pat` in a specific section."""
    s = section
    seg = b[s["ro"]:s["ro"]+s["rsz"]]
    matches = []
    i = 0
    while True:
        i = seg.find(pat, i)
        if i < 0: break
        matches.append(i)
        i += 1
    return matches

dd_b, dd_ib, dd_secs = load(DD)
gdi_b, gdi_ib, gdi_secs = load(GDI)

print(f"DD  image_base={dd_ib:#010x}")
print(f"GDI image_base={gdi_ib:#010x}")
for name in [".text", ".rdata", ".data"]:
    if name in dd_secs and name in gdi_secs:
        print(f"  {name:8s} DD va={dd_secs[name]['va']:#010x} sz={dd_secs[name]['rsz']:#010x}  "
              f"GDI va={gdi_secs[name]['va']:#010x} sz={gdi_secs[name]['rsz']:#010x}")

# Candidate functions in DD (cm0102.exe) — VAs known
TARGETS = {
    0x0055f040: "eng_second ctor",
    0x0055f340: "schedule-getter",
    0x00560320: "schedule-installer FUN_00560320",
    0x005601d0: "roster populator",
    0x00668890: "round-robin driver FUN_00668890",
    0x0066f280: "walker FUN_0066f280",
    0x0066f3b0: "round writer FUN_0066f3b0",
    0x0066f410: "slot writer FUN_0066f410",
    0x00533b50: "pack_date FUN_00533b50",
    0x00533eb0: "flag-snap FUN_00533eb0",
    0x00594d00: "TFixList insert FUN_00594d00",
    0x00669780: "matrix base seeder FUN_00669780",
    0x0066bd40: "matrix perturb FUN_0066bd40",
    0x00845380: "venue writer FUN_00845380",
    0x0066a910: "replay/reset FUN_0066a910",
}

FIRST_BYTES = 48  # signature length

print("\n=== Function pattern matching (DD -> GDI) ===")
print(f"{'DD VA':11s} | {'Function':38s} | {'GDI matches':30s} | notes")
print("-"*130)
gdi_text = gdi_secs.get(".text")
if not gdi_text:
    raise SystemExit("no .text in GDI")

results = {}
for va, name in sorted(TARGETS.items()):
    off = va_to_off(dd_secs, dd_ib, va)
    if off is None:
        print(f"{va:#010x}  | {name:38s} | NOT FOUND in DD")
        continue
    sig = dd_b[off:off+FIRST_BYTES]
    # Search for signature in GDI .text
    matches = find_pattern(gdi_b, gdi_text, sig)
    if not matches:
        # Try shorter patterns
        for shorter in [32, 24, 16]:
            matches = find_pattern(gdi_b, gdi_text, sig[:shorter])
            if matches:
                note = f"partial match at {shorter} bytes"
                break
        else:
            note = "NO MATCH"
            matches = []
    else:
        note = f"exact {FIRST_BYTES} bytes"
    match_vas = [off_to_va(gdi_secs, gdi_ib, gdi_text["ro"] + m) for m in matches]
    match_str = ", ".join(f"{m:#010x}" for m in match_vas[:3])
    if len(match_vas) > 3: match_str += f" (+{len(match_vas)-3} more)"
    if not match_vas:
        match_str = "-"
    print(f"{va:#010x}  | {name:38s} | {match_str:30s} | {note}")
    if len(match_vas) == 1:
        results[va] = match_vas[0]

print("\n=== VA translation table (unambiguous only) ===")
for dd_va, gdi_va in sorted(results.items()):
    delta = gdi_va - dd_va
    print(f"  {dd_va:#010x} -> {gdi_va:#010x}  (delta {delta:+#x})  {TARGETS[dd_va]}")

# Save to JSON
out = Path(r"D:\cm0102-rs\reports\fixture_disasm\gdi_va_map.json")
out.write_text(json.dumps({f"{k:#010x}": {"gdi_va": f"{v:#010x}", "delta": v-k, "name": TARGETS[k]}
                            for k,v in results.items()}, indent=2))
print(f"\nSaved to {out}")
