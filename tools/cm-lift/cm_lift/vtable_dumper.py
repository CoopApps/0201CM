"""Static C++ vtable extractor for cm0102.exe.

Reads the .rdata section and locates every table of consecutive u32
pointers that all point into .text (the classic MSVC vtable shape).
Emits a JSON with the entries so downstream tooling can bind virtual
dispatch slots like `cup+0x9c` (the penalty-shootout resolver we've
been unable to lift).

Approach:
  1. Walk .rdata in 4-byte steps.
  2. At each position, check whether the next N u32s all fall into
     .text — that's a candidate vtable.
  3. Filter out spurious runs (need ≥ 3 consecutive pointers).
  4. When possible, look at the u32 immediately BEFORE the vtable start
     (MSVC vtables have an RTTI COL / typeinfo pointer at [-1]) — this
     lets us name the class.

Not a full C++ ABI reconstructor — just "for these vtable regions, here
are the fn pointers per slot". That's enough to bind the ~30 known
virtual dispatches in the cup / competition / manager clusters.
"""
from __future__ import annotations
import json
import struct
from pathlib import Path
from dataclasses import dataclass
from .util import PeInfo, load_pe, DATA_OUT

MIN_VTABLE_SLOTS = 3
MAX_SEARCH_SLOTS = 512


@dataclass
class Vtable:
    va: int                # RVA of slot 0
    slots: list[int]       # list of fn addresses
    rtti_col: int | None   # value at [-1], if it looks like a valid pointer

    def to_dict(self):
        return {
            "va": f"{self.va:#010x}",
            "slots": [f"{s:#010x}" for s in self.slots],
            "rtti_col": f"{self.rtti_col:#010x}" if self.rtti_col else None,
            "slot_count": len(self.slots),
        }


def is_text_ptr(pe: PeInfo, va: int) -> bool:
    return pe.text_va <= va < pe.text_va + pe.text_size


def _load_fn_entries() -> set[int]:
    """Load Ghidra's set of fn entry addresses from functions.json.

    A real C++ vtable slot points to the FIRST byte of a fn — not to
    an arbitrary mid-fn offset. Cross-referencing every candidate slot
    against Ghidra's fn-entry set eliminates ~all the false vtable
    detections that were previously running for 512 slots into jump
    tables and unrelated .rdata regions.
    """
    from .util import DECOMPILE
    fns_json = DECOMPILE.parent / "functions.json"
    if not fns_json.exists():
        return set()
    import json
    with open(fns_json, "rb") as f:
        data = json.loads(f.read().decode("utf-8", errors="replace"))
    out: set[int] = set()
    if isinstance(data, list):
        for e in data:
            va = e.get("entry")
            if isinstance(va, str):
                try: out.add(int(va, 0))
                except ValueError: pass
            elif isinstance(va, int):
                out.add(va)
    return out


_FN_ENTRIES_CACHE: set[int] | None = None
def fn_entries() -> set[int]:
    global _FN_ENTRIES_CACHE
    if _FN_ENTRIES_CACHE is None:
        _FN_ENTRIES_CACHE = _load_fn_entries()
    return _FN_ENTRIES_CACHE


def scan_vtables(pe: PeInfo) -> list[Vtable]:
    """Sweep .rdata for consecutive fn-entry-pointer runs.

    Slot validation: each pointer must be a real Ghidra fn entry, not
    just any address in .text. This eliminates the jump-table false
    positives that previously gave 128+/512+ slot 'vtables'.
    """
    entries = fn_entries()
    if not entries:
        # Fallback: use loose text-ptr check
        _accept = lambda p: is_text_ptr(pe, p)
    else:
        _accept = lambda p: p in entries

    tables: list[Vtable] = []
    data = pe.rdata_bytes
    base = pe.rdata_va
    length = len(data)
    i = 0
    while i + 4 <= length:
        ptr = struct.unpack_from("<I", data, i)[0]
        if _accept(ptr):
            slots = []
            j = i
            while j + 4 <= length and len(slots) < MAX_SEARCH_SLOTS:
                p = struct.unpack_from("<I", data, j)[0]
                if not _accept(p):
                    break
                slots.append(p)
                j += 4
            if len(slots) >= MIN_VTABLE_SLOTS:
                rtti = None
                if i >= 4:
                    ptr_prev = struct.unpack_from("<I", data, i - 4)[0]
                    if (pe.rdata_va <= ptr_prev < pe.rdata_va + pe.rdata_size
                        or pe.data_va <= ptr_prev < pe.data_va + pe.data_size):
                        rtti = ptr_prev
                tables.append(Vtable(va=base + i, slots=slots, rtti_col=rtti))
                i = j
                continue
        i += 4
    return tables


def dump_json(pe: PeInfo | None = None, out_path: Path | None = None) -> Path:
    pe = pe or load_pe()
    tables = scan_vtables(pe)
    out = out_path or (DATA_OUT / "vtables.json")
    with open(out, "w") as f:
        json.dump(
            {"exe": str(pe.path), "count": len(tables),
             "tables": [t.to_dict() for t in tables]},
            f, indent=2,
        )
    return out


def find_slot(pe: PeInfo, vtable_va: int, slot_index: int) -> int | None:
    """Convenience: read the u32 at vtable_va + slot_index*4. Returns the
    fn address at that slot, or None if it doesn't look like a text ptr.
    """
    va = vtable_va + slot_index * 4
    raw = pe.read_va(va, 4)
    if raw is None or len(raw) < 4:
        return None
    ptr = struct.unpack("<I", raw)[0]
    return ptr if is_text_ptr(pe, ptr) else None


# --- CLI ------------------------------------------------------------------

def main():
    import argparse, sys
    ap = argparse.ArgumentParser(description="Dump C++ vtables from cm0102.exe .rdata")
    ap.add_argument("--slot", nargs=2, metavar=("VTABLE_VA", "IDX"),
                    help="Print the fn at vtable_va + idx*4")
    ap.add_argument("-o", "--out", default=None)
    args = ap.parse_args()

    pe = load_pe()
    if args.slot:
        vt = int(args.slot[0], 0)
        idx = int(args.slot[1], 0)
        fn = find_slot(pe, vt, idx)
        print(f"vtable {vt:#010x} + {idx}*4 = {fn:#010x}" if fn
              else f"vtable {vt:#010x} slot {idx} — no fn")
        return

    out = dump_json(pe, Path(args.out) if args.out else None)
    tables = json.loads(out.read_text())
    print(f"wrote {out}")
    print(f"  {tables['count']} vtables found in .rdata")
    # Show largest 5
    biggest = sorted(tables['tables'], key=lambda t: -t['slot_count'])[:5]
    print(f"  largest 5 by slot count:")
    for t in biggest:
        rtti = t['rtti_col'] or '(no rtti)'
        print(f"    {t['va']}  {t['slot_count']:3d} slots  rtti={rtti}")


if __name__ == "__main__":
    main()
