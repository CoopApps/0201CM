"""Extract the exact 46-tuple round-writer input template from the captured .jsonl.
Emit a Rust `pub const` array."""
import json, sys
from pathlib import Path
p = Path(r"D:\cm0102-rs\reports\fixture_disasm\runtime\20260913_113106_direct_call.jsonl")
rows = [json.loads(l) for l in p.read_text().splitlines() if l.strip()]
writers = [r for r in rows if r["op"] == "roundwriter"]
assert len(writers) == 46, len(writers)

# The "year" arg in some calls is stack-garbage (upper 16 bits leak); the writer
# only reads it as a short. Normalise: year is always 2001 (season start year).
print("pub const ENG_SECOND_2001_TEMPLATE: [(i8, i8, i32, i32, u8); 46] = [")
print("    // (day, month, day_off, flag, type)")
for w in writers:
    print(f"    ({w['day']:2d}, {w['month']:2d}, {w['day_off']}, {w['flag']:2d}, {w['type_byte']}),  // round {w['round_idx']:2d} ret=0x{int(w['retaddr'],16):x}")
print("];")
