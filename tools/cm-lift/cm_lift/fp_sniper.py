"""FPU-stack constant extractor.

For every `_DAT_*` symbol Ghidra flagged as an unnamed float/double
constant, we can either:

  (a) read the raw bytes at that .rdata address and interpret as f32/f64
      (works for ~90% of `_DAT_009569xx` style names where Ghidra just
      didn't infer a type), or

  (b) drive the exe under Unicorn to a `__ftol` call site and read the
      FPU stack at that moment (needed for the ~40 fns where the const
      is only ever materialised on ST(0) via a computed expression).

This module does both. `dump_static_constants()` reads .rdata bytes and
emits every `_DAT_00955xxx..0095Dxxx` VA as f32/f64/i32 candidates.
`snipe_ftol_call()` runs a specific fn under Unicorn, hooks each
`__ftol` (FUN_00934200 / 934290 / 9346d0 — the three variants MSVC
emits) and dumps ST(0) at the call moment.

Together they turn every OPEN GAP marked "FP-unlifted" or "FP-elided
constant" into a mechanical extraction.
"""
from __future__ import annotations
import json
import re
import struct
from pathlib import Path
from typing import Optional
from .util import PeInfo, load_pe, DATA_OUT, iter_decompiled_fns

# --- Known __ftol variants -----------------------------------------------
FTOL_ADDRS = [
    0x00934200,  # __ftol
    0x00934290,  # _ftol2
    0x009346d0,  # _CIpow or similar; verify per version
    0x009346a0,
]


# --- Static constant sweep ------------------------------------------------

def dump_static_constants(pe: Optional[PeInfo] = None,
                          out_path: Optional[Path] = None) -> Path:
    """Scan .rdata for every VA in the 0x00955000..0x0095DFFF range
    (where Ghidra puts unnamed floating-point globals in this exe) and
    emit each 4-byte / 8-byte window as a candidate f32/f64/i32 value.

    Also scans .rdata for double-aligned sequences that look like
    plausible-magnitude game constants (0.0..1e10 range).
    """
    pe = pe or load_pe()
    out = out_path or (DATA_OUT / "static_constants.json")
    entries = {}
    # Sweep the range where cm0102's per-subsystem float constants live.
    lo = 0x00955000
    hi = 0x0095E000
    if lo < pe.rdata_va: lo = pe.rdata_va
    if hi > pe.rdata_va + pe.rdata_size:
        hi = pe.rdata_va + pe.rdata_size

    off = lo - pe.rdata_va
    end = hi - pe.rdata_va
    while off + 4 <= end:
        va = pe.rdata_va + off
        w32 = pe.rdata_bytes[off:off+4]
        as_i32 = struct.unpack("<i", w32)[0]
        as_f32 = struct.unpack("<f", w32)[0]
        row = {
            "va": f"{va:#010x}",
            "i32": as_i32,
            "u32": as_i32 & 0xFFFFFFFF,
            "f32": as_f32,
        }
        if off + 8 <= end:
            w64 = pe.rdata_bytes[off:off+8]
            as_f64 = struct.unpack("<d", w64)[0]
            row["f64"] = as_f64
        entries[row["va"]] = row
        off += 4

    with open(out, "w") as f:
        json.dump({"exe": str(pe.path),
                   "range": [f"{lo:#010x}", f"{hi:#010x}"],
                   "count": len(entries),
                   "entries": list(entries.values())}, f, indent=2,
                  default=lambda o: str(o))
    return out


def lookup_constant(va: int, kind: str = "f64",
                    pe: Optional[PeInfo] = None) -> Optional[float]:
    """Return the value at VA as f32 / f64 / i32."""
    pe = pe or load_pe()
    off = va - pe.rdata_va
    if off < 0 or off >= pe.rdata_size:
        return None
    n = {"i32": 4, "f32": 4, "f64": 8}[kind]
    raw = pe.rdata_bytes[off:off+n]
    if len(raw) < n:
        return None
    return struct.unpack("<" + {"i32": "i", "f32": "f", "f64": "d"}[kind], raw)[0]


# --- Grep decompile for every _DAT_ ref ---------------------------------

DAT_PAT = re.compile(r"_DAT_([0-9a-fA-F]{8})")

def scan_dat_refs(pe: Optional[PeInfo] = None,
                  out_path: Optional[Path] = None) -> Path:
    """For every _DAT_XXXX symbol referenced across the decompile,
    resolve the VA against .rdata and dump the concrete value. This is
    the primary path for lifting the ~840 elided FP constants.
    """
    pe = pe or load_pe()
    seen: dict[str, dict] = {}  # va-hex -> {va, f32, f64, i32, refs: [fn_addrs]}
    for addr, path in iter_decompiled_fns():
        try:
            txt = path.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue
        for m in DAT_PAT.finditer(txt):
            va_hex = m.group(1).lower()
            va = int(va_hex, 16)
            key = f"0x{va_hex}"
            if key not in seen:
                # Resolve constant if it's in .rdata.
                entry = {
                    "va": key,
                    "f32": lookup_constant(va, "f32", pe),
                    "f64": lookup_constant(va, "f64", pe),
                    "i32": lookup_constant(va, "i32", pe),
                    "refs": [],
                }
                seen[key] = entry
            seen[key]["refs"].append(f"{addr:#010x}")

    # Compress refs — keep only distinct + first 20 per constant.
    for e in seen.values():
        uniq = list(dict.fromkeys(e["refs"]))
        e["ref_count"] = len(uniq)
        e["refs"] = uniq[:20]

    out = out_path or (DATA_OUT / "dat_constants.json")
    with open(out, "w") as f:
        json.dump({"count": len(seen),
                   "entries": sorted(seen.values(), key=lambda e: e["va"])},
                  f, indent=2, default=lambda o: str(o))
    return out


# --- Dynamic FTOL snipe (Unicorn-driven) --------------------------------

def snipe_ftol_call(fn_va: int, args: list[int] = None,
                    thiscall: bool = False,
                    pe: Optional[PeInfo] = None,
                    out_path: Optional[Path] = None) -> Path:
    """Run `fn_va` under Unicorn, hook every __ftol variant, and dump
    ST(0) at every hit.

    Use this for the ~40 fns where the FP expression is materialised
    on the FPU stack via `fmul/fadd/fdiv` chains before `__ftol` — the
    static .rdata dump alone doesn't recover those (they're computed).
    """
    from .emulate import make_emulator
    emu = make_emulator(pe)
    hits = []
    def _fpu_dump(emu_ref, hook_addr, size):
        st = emu_ref.read_fpu()
        hits.append({
            "hook_addr": f"{hook_addr:#010x}",
            "st0": st[0] if st else None,
            "st1": st[1] if len(st) > 1 else None,
            "regs": {k: f"{v:#010x}" for k, v in emu_ref.read_regs().items()
                     if k in ("eax", "ecx", "edx", "esp")},
        })
    for ftol in FTOL_ADDRS:
        emu.hook_call(ftol, _fpu_dump)
    args = args or []
    retval = emu.call(fn_va, *args, thiscall=thiscall, max_ticks=1_000_000)
    out = out_path or (DATA_OUT / f"ftol_snipe_{fn_va:08x}.json")
    with open(out, "w") as f:
        json.dump({"fn": f"{fn_va:#010x}", "return_eax": f"{retval:#010x}",
                   "hits": hits, "exc": getattr(emu, "last_exc", None)},
                  f, indent=2, default=lambda o: str(o))
    return out


# --- CLI ------------------------------------------------------------------

def main():
    import argparse
    ap = argparse.ArgumentParser(description="FPU + .rdata constant extractor")
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("static", help="Dump every candidate .rdata value in 0x955000..0x95DFFF")
    sub.add_parser("scan-refs", help="Grep decompile for every _DAT_ ref and resolve it")
    snip = sub.add_parser("snipe", help="Dynamic FTOL snipe on one fn")
    snip.add_argument("fn_va", help="Fn address (hex, e.g. 0x00580a90)")
    snip.add_argument("--arg", action="append", default=[], type=lambda s: int(s, 0),
                      help="arg to pass to fn (repeat)")
    snip.add_argument("--thiscall", action="store_true")
    args = ap.parse_args()

    if args.cmd == "static":
        out = dump_static_constants()
        data = json.loads(out.read_text())
        print(f"wrote {out}  ({data['count']} candidate constants in {data['range']})")
    elif args.cmd == "scan-refs":
        out = scan_dat_refs()
        data = json.loads(out.read_text())
        print(f"wrote {out}  ({data['count']} distinct _DAT_ constants referenced)")
        # top 10 most-referenced
        top = sorted(data["entries"], key=lambda e: -e["ref_count"])[:10]
        print(f"top 10 by ref count:")
        for e in top:
            print(f"  {e['va']}  refs={e['ref_count']:4d}  "
                  f"f32={e['f32']}  f64={e['f64']}  i32={e['i32']}")
    elif args.cmd == "snipe":
        va = int(args.fn_va, 0)
        out = snipe_ftol_call(va, args.arg, thiscall=args.thiscall)
        data = json.loads(out.read_text())
        print(f"wrote {out}  ({len(data['hits'])} __ftol hits)")


if __name__ == "__main__":
    main()
