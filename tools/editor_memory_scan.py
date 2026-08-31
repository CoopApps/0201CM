"""Attach to running cm0102ed.exe and find the PARSED record arrays in memory
by searching for known distinctive strings.

Rationale: the editor's ReadFile hits land in tiny reused scratch buffers, not
in the permanent record structs. Parsed records live elsewhere. Find them by
searching for names/strings we know appear exactly once (or a small number of
times).

Assumes the editor is already running and has finished Import.

Usage:
    D:/Python312/python.exe D:/cm0102-rs/tools/editor_memory_scan.py

Outputs:
    reports/editor_capture/mem_scan.json  — for each search term, all hits
    reports/editor_capture/hits/<label>_<addr>.bin — a 4KB window around each hit
"""
import argparse, json, sys, time
from pathlib import Path

try:
    import frida
except ImportError:
    sys.exit("frida not installed. Run: D:/Python312/python.exe -m pip install frida frida-tools")

REPO = Path(__file__).resolve().parents[1]
AGENT = REPO / "tools/editor_field_agent.js"
OUT_DIR = REPO / "reports/editor_capture"
HIT_DIR = OUT_DIR / "hits"
HIT_DIR.mkdir(parents=True, exist_ok=True)

# Known distinctive markers. Each is (label, needle_bytes, expected_hit_count).
# Delphi often stores strings as ANSI (Windows-1252) with an optional length
# prefix (ShortString) or as null-terminated C strings. We search both patterns.
SEARCH_TARGETS = [
    ("SWFC_short_string", b"\x13Sheffield Wednesday"),   # ShortString: len-prefix 0x13 (=19)
    ("SWFC_cstring",      b"Sheffield Wednesday\x00"),
    ("MUFC_short_string", b"\x11Manchester United"),
    ("MUFC_cstring",      b"Manchester United\x00"),
    ("Pressman_cs",       b"Kevin Pressman\x00"),
    ("TJohnson_cs",       b"Tommy Johnson\x00"),
    ("Barcelona_cs",      b"F.C. Barcelona\x00"),
    ("SWFC_no_delim",     b"Sheffield Wednesday"),        # last resort
]

def hex_str(b: bytes) -> str:
    return b.hex()

def main():
    device = frida.get_local_device()
    pid = next((p.pid for p in device.enumerate_processes()
                if p.name.lower() == "cm0102ed.exe"), None)
    if not pid:
        sys.exit("cm0102ed.exe not running — launch it and complete Import first")
    session = device.attach(pid)
    print(f"[scan] attached pid {pid}")
    script = session.create_script(AGENT.read_text(encoding="utf-8"))
    script.on("message", lambda msg, data: None)
    script.load()

    all_hits = {}
    for label, needle in SEARCH_TARGETS:
        pattern = " ".join(f"{b:02x}" for b in needle)
        try:
            hits = script.exports_sync.find_bytes(pattern, 16)
        except Exception as e:
            print(f"[scan] {label}: findBytes failed: {e}")
            continue
        print(f"[scan] {label:<22} {'':>10} → {len(hits)} hits  needle={needle[:32]!r}")
        for h in hits:
            addr = int(h["addr"], 16) if isinstance(h["addr"], str) else h["addr"]
            print(f"    hit @ 0x{addr:016x}")
        all_hits[label] = [
            {"addr": h["addr"], "size": h["size"]} for h in hits
        ]
        # Dump a 4KB window around each hit for offline structure inspection.
        for h in hits:
            addr_str = h["addr"] if isinstance(h["addr"], str) else f"0x{h['addr']:x}"
            addr = int(addr_str, 16) if isinstance(addr_str, str) else addr_str
            window_start = max(0, addr - 256)
            try:
                hex_data = script.exports_sync.dump_region(f"0x{window_start:x}", 4096)
            except Exception as e:
                print(f"    ! window read failed: {e}"); continue
            if not hex_data: continue
            data = bytes.fromhex(hex_data)
            out = HIT_DIR / f"{label}_{addr:x}.bin"
            out.write_bytes(data)
            print(f"    → 4KB window saved to {out.name}")

    (OUT_DIR / "mem_scan.json").write_text(
        json.dumps(all_hits, indent=2), encoding="utf-8")
    print(f"\n[scan] wrote {OUT_DIR / 'mem_scan.json'}")
    print(f"[scan] {sum(len(v) for v in all_hits.values())} total hits saved to {HIT_DIR}")

    # Follow-up: if SWFC + MUFC both hit exactly once each, compute stride and
    # infer the club array base.
    sw = all_hits.get("SWFC_short_string", []) or all_hits.get("SWFC_cstring", []) or \
         all_hits.get("SWFC_no_delim", [])
    mu = all_hits.get("MUFC_short_string", []) or all_hits.get("MUFC_cstring", [])
    if sw and mu:
        def parse_addr(h): return int(h["addr"], 16) if isinstance(h["addr"], str) else h["addr"]
        # Pick the closest pair — likely the ones inside the club array.
        best = None
        for s in sw:
            for m in mu:
                d = abs(parse_addr(s) - parse_addr(m))
                if best is None or d < best[0]:
                    best = (d, parse_addr(s), parse_addr(m))
        if best:
            d, sa, ma = best
            print(f"\n[scan] SWFC..MUFC delta = {d:,} bytes")
            print(f"       If those are two entries of the club array, stride = {d} bytes")
            print(f"       (or {d}/N for N clubs between them).")

if __name__ == "__main__":
    main()
