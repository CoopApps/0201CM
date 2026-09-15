#!/usr/bin/env python3
"""Decode the 34-slot season-roll scheduler table snapshot emitted
by season_capture.js's snapshotSlotTable() function.

For each slot, prints:
  slot index, trigger_day (0..366), last_processed_year, count,
  each pooled competition's identity (id / name / year / vtable ptr).

Groups slots by trigger_day so it's obvious which competitions
share a fire moment (e.g. all English pyramid comps on the same
day-of-year).

Usage:
    D:/Python312/python.exe tools/gdi_capture/decode_slot_table.py \\
        fixtures/gdi_captures/season_2001_02_v4_slot_table.jsonl
"""
from __future__ import annotations
import argparse
import json
import struct
import sys
from collections import defaultdict
from pathlib import Path


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("jsonl", type=Path)
    return ap.parse_args()


def decode_latin1_cstr(b: bytes) -> str:
    end = b.find(b"\x00")
    if end == -1: end = len(b)
    try:
        return b[:end].decode("latin-1", errors="replace")
    except Exception:
        return "?"


def decode_comp_header(hex128: str) -> dict:
    if not hex128 or len(hex128) < 256:
        return {}
    try:
        raw = bytes.fromhex(hex128)
    except ValueError:
        return {"decode_err": "not hex"}
    if len(raw) < 128:
        return {}
    # Layout confirmed from prior v3 capture:
    #   +0x00 vtable ptr (u32)
    #   +0x40 season year (u16 LE)
    #   +0x54 name (Latin-1, up to 0x71)
    return {
        "vtable":  f"0x{struct.unpack_from('<I', raw, 0x00)[0]:08x}",
        "year":    struct.unpack_from("<H", raw, 0x40)[0],
        "name":    decode_latin1_cstr(raw[0x54:0x54 + 40]),
    }


def dow_label(day: int) -> str:
    # Very rough day-of-year -> approximate month label. Non-leap.
    md = [(31,"Jan"),(28,"Feb"),(31,"Mar"),(30,"Apr"),(31,"May"),
          (30,"Jun"),(31,"Jul"),(31,"Aug"),(30,"Sep"),(31,"Oct"),
          (30,"Nov"),(31,"Dec")]
    d = day
    for length, m in md:
        if d < length: return f"{m}-{d + 1}"
        d -= length
    return f"day{day}"


def main() -> int:
    args = parse_args()
    if not args.jsonl.exists():
        print(f"missing: {args.jsonl}"); return 2

    slots: list[dict] = []
    probes: list[dict] = []
    snapshot_done = None
    for line in args.jsonl.open("r", encoding="utf-8"):
        line = line.strip()
        if not line: continue
        try: rec = json.loads(line)
        except json.JSONDecodeError: continue
        hook = rec.get("hook", "")
        if hook == "__slot_table_probe__":
            probes.append(rec)
        elif hook == "__slot_table_entry__":
            slots.append(rec)
        elif hook == "__slot_table_snapshot_done__":
            snapshot_done = rec

    print(f"{len(probes)} candidate probe(s):")
    for p in probes:
        print(f"    {p.get('candidate'):>35s}  va={p.get('va'):<12s}  "
              f"count={p.get('slot0_count')}  "
              f"trigger={p.get('slot0_trigger')}  "
              f"last_year={p.get('slot0_last_year')}  "
              f"looks_real={p.get('looks_real')}")
    if snapshot_done:
        print(f"    winner: {snapshot_done.get('winning_candidate')}")
    print()

    if not slots:
        print("No slot entries captured. If probe results above show")
        print("no candidate flagged looks_real, the GDI slot-table VA")
        print("is different from the 3 candidates. Extend the JS.")
        return 1

    print(f"{len(slots)} slot entries dumped:")
    print()

    # Print each slot with its comps.
    slots_sorted = sorted(slots, key=lambda s: s["slot"])
    for s in slots_sorted:
        idx = s["slot"]; trig = s["trigger_day"]
        count = s["count"]; last_year = s["last_processed_year"]
        print(f"slot {idx:>2}  trigger_day={trig:>3d} ({dow_label(trig)})  "
              f"count={count:>3d}  last_year={last_year}  "
              f"pool={s['pool_ptr']}")
        for c in s.get("comps", []):
            if c.get("err"):
                print(f"    idx={c.get('idx')}: ERROR {c['err']}")
                continue
            if c.get("comp_ptr") is None:
                print(f"    idx={c.get('idx')}: null")
                continue
            info = decode_comp_header(c.get("hex128", ""))
            print(f"    idx={c.get('idx'):>3d}  ptr={c['comp_ptr']:<12s}  "
                  f"vtable={info.get('vtable','?'):<12s}  "
                  f"year={info.get('year','?')!s:>4s}  "
                  f"name={info.get('name','?')!r}")
        print()

    # Group by trigger_day → helps see which comps share a fire moment.
    print("=" * 60)
    print("Trigger-day grouping (comps sharing a fire day):")
    by_day: dict[int, list[dict]] = defaultdict(list)
    for s in slots_sorted:
        if s["count"] > 0:
            by_day[s["trigger_day"]].append(s)
    for day in sorted(by_day):
        n_comps = sum(sl["count"] for sl in by_day[day])
        print(f"\n  day-of-year {day:>3d} ({dow_label(day)}): "
              f"{n_comps} comps across {len(by_day[day])} slot(s)")
        for s in by_day[day]:
            for c in s.get("comps", []):
                info = decode_comp_header(c.get("hex128", ""))
                name = info.get('name', '?')
                if name and name.strip():
                    print(f"        {name!r}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
