"""Zero-UI editor memory dump.

Spawn cm0102ed.exe, watch it read .dat files during Import, and after Import
quiesces (no ReadFile for `--quiet` seconds) dump each file's receiving memory
range back to disk. Byte-exact snapshot of every record as the editor sees it,
no UI walking required.

Output: reports/editor_capture/mem/
  <name>.bin    — raw bytes read from the file, in file order (concatenated)
  <name>.json   — {total_bytes, read_count, addr_span, region_buckets}
  index.json    — per-file summary + provenance

Usage (from any dir):
    D:/Python312/python.exe D:/cm0102-rs/tools/editor_memory_dump.py
    # then in the editor: File → Import → point at D:/cm0102/data
    # this script auto-dumps ~5s after reads stop firing
"""
import argparse, json, os, sys, time
from pathlib import Path

try:
    import frida
except ImportError:
    sys.exit("frida not installed. Run: D:/Python312/python.exe -m pip install frida frida-tools")

REPO = Path(__file__).resolve().parents[1]
AGENT = REPO / "tools/editor_field_agent.js"
OUT_DIR = REPO / "reports/editor_capture/mem"
OUT_DIR.mkdir(parents=True, exist_ok=True)
EDITOR_EXE = r"D:\cm0102\Editor\cm0102ed.exe"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--attach", action="store_true")
    ap.add_argument("--exe", default=EDITOR_EXE)
    ap.add_argument("--quiet", type=float, default=5.0,
                    help="seconds of no reads before we dump (default 5)")
    ap.add_argument("--min-reads", type=int, default=1000,
                    help="don't dump until at least this many reads have fired")
    args = ap.parse_args()

    device = frida.get_local_device()
    if args.attach:
        pid = next((p.pid for p in device.enumerate_processes()
                    if p.name.lower() == "cm0102ed.exe"), None)
        if not pid: sys.exit("cm0102ed.exe not running")
        session = device.attach(pid)
        print(f"[dump] attached pid {pid}")
    else:
        print(f"[dump] spawning {args.exe}")
        pid = device.spawn([args.exe])
        session = device.attach(pid)

    script = session.create_script(AGENT.read_text(encoding="utf-8"))

    state = {"reads": 0, "last_read_at": 0.0, "started": False}

    def on_message(msg, data):
        if msg.get("type") != "send": return
        p = msg["payload"]
        t = p.get("t")
        if t == "hello":
            print("[agent]", p.get("msg"))
            return
        if t == "read":
            state["reads"] += 1
            state["last_read_at"] = time.time()
            if not state["started"] and state["reads"] > 100:
                state["started"] = True
                print(f"[dump] reads flowing — first {state['reads']}...")
            if state["reads"] % 5000 == 0:
                print(f"[dump] reads = {state['reads']}")

    script.on("message", on_message)
    script.load()
    if not args.attach:
        device.resume(pid)

    print()
    print("=========================================================================")
    print("Now in the editor: File → Import → point at D:\\cm0102\\data")
    print(f"Script will auto-dump {args.quiet}s after reads stop.")
    print("=========================================================================")
    print()

    while True:
        time.sleep(0.5)
        if not state["started"]:
            continue
        idle = time.time() - state["last_read_at"]
        if state["reads"] >= args.min_reads and idle > args.quiet:
            print(f"\n[dump] reads quiet for {idle:.1f}s; total {state['reads']}. Dumping.")
            break

    # First: dump every live memory-mapped view. This is how the editor loads
    # club.dat and part of staff.dat — via MapViewOfFile, which bypasses
    # ReadFile entirely. The mapping IS a byte-exact image of the file.
    try:
        maps = script.exports_sync.list_mappings()
    except Exception as e:
        print(f"[dump] list_mappings failed: {e}")
        maps = []
    if maps:
        print(f"[dump] {len(maps)} live file mappings:")
        for m in maps:
            fn = Path(m['file']).name if m.get('file') and m['file'] != '?' else '?'
            print(f"  addr=0x{int(m['addr'],16):x}  size={m['size']:,}  prot={m['prot']}  file={fn}")
            if not fn or fn == '?' or not fn.lower().endswith('.dat'): continue
            size = m['size']
            if size == 0 or size > 128*1024*1024:
                print(f"    skipping (size={size:,})"); continue
            base = int(m['addr'], 16)
            # Chunked dump.
            chunk = 512*1024
            parts = []
            for off in range(0, size, chunk):
                n = min(chunk, size - off)
                try:
                    hx = script.exports_sync.dump_region(f"0x{base+off:x}", n)
                except Exception as e:
                    print(f"    ! chunk read failed: {e}"); hx = None
                parts.append(bytes.fromhex(hx) if hx else b'\0'*n)
            data = b"".join(parts)
            out = OUT_DIR / f"mapped_{fn.replace('.','_')}.bin"
            out.write_bytes(data)
            print(f"    → {out.name} ({len(data):,} bytes)")

    # Then: also dump the ReadFile scratch destinations (may be useful for
    # files that DO come through ReadFile). Keeps existing behaviour.
    loaded = script.exports_sync.loaded_files()
    print(f"[dump] loaded {len(loaded)} .dat files:")
    index = {}
    for name, info in sorted(loaded.items()):
        span_kb = info["span"] // 1024
        print(f"  {name:<28} {info['total_bytes']:>10,} bytes  "
              f"reads={info['read_count']:<5}  addr_span={span_kb:,} KB  "
              f"regions={len(info['region_buckets'])}")
        index[name] = {
            "total_bytes": info["total_bytes"],
            "read_count": info["read_count"],
            "min_addr": info["min_addr"],
            "max_addr": info["max_addr"],
            "span_bytes": info["span"],
            "region_buckets": info["region_buckets"],
        }

    # For each file: dump the contiguous memory range that received its bytes.
    for name, info in sorted(loaded.items()):
        base_addr = info["min_addr"]
        size = info["span"]
        if base_addr is None or size <= 0: continue
        # Cap at 32MB per file to avoid pulling huge irrelevant ranges.
        if size > 32 * 1024 * 1024:
            print(f"[dump] {name}: span {size:,} > 32MB — skipping (probably fragmented)")
            continue
        print(f"[dump] {name}: reading {size:,} bytes @ 0x{base_addr:x}...")
        chunk = 512 * 1024
        parts = []
        for off in range(0, size, chunk):
            n = min(chunk, size - off)
            try:
                hexstr = script.exports_sync.dump_region(f"0x{base_addr + off:x}", n)
            except Exception as e:
                print(f"    ! read failed at +0x{off:x}: {e}")
                hexstr = None
            if hexstr is None:
                parts.append(b'\x00' * n)  # unreadable page
            else:
                parts.append(bytes.fromhex(hexstr))
        data = b"".join(parts)
        out = OUT_DIR / (name.replace('.', '_') + ".bin")
        out.write_bytes(data)
        print(f"    → {out}  ({len(data):,} bytes)")

    (OUT_DIR / "index.json").write_text(
        json.dumps(index, indent=2, default=str), encoding="utf-8")
    print(f"\n[dump] wrote index → {OUT_DIR / 'index.json'}")
    print(f"[dump] done — {len(loaded)} files snapshotted to {OUT_DIR}")

    # Keep script alive briefly in case user wants to keep exploring; then exit.
    time.sleep(1)

if __name__ == "__main__":
    main()
