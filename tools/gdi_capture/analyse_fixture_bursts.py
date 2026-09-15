#!/usr/bin/env python3
"""Analyse a season_capture JSONL's fixture-engine calls into
burst-level summaries + per-burst RNG budgets.

A "burst" is a contiguous cluster of Group-B fixture-engine calls
with < 60 s of wall-clock gap between adjacent calls. The exe
appears to invoke the schedule getter / driver / walker / perturb
in tight groups (per-league schedule generation, year-end regen,
cup-round injection) separated by long quiet stretches.

For each burst the analyser prints:
  wall-clock window
  RNG state at first & last event (delta = RNG consumption)
  hook counts inside the burst
  distinct arg0 values for the schedule_getter / driver

Usage:
    D:/Python312/python.exe tools/gdi_capture/analyse_fixture_bursts.py \\
        season_2001_02.jsonl
"""
from __future__ import annotations
import argparse
import json
from collections import Counter
from pathlib import Path

BURST_GAP_MS = 60_000


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("jsonl", type=Path)
    return ap.parse_args()


def rng_step(a: int | None, b: int | None) -> str:
    if a is None or b is None:
        return "?"
    diff = (b - a) & 0xFFFFFFFF
    return f"{diff:>10d}"


def main() -> int:
    args = parse_args()
    events = []
    for line in args.jsonl.open("r", encoding="utf-8"):
        line = line.strip()
        if not line: continue
        try: rec = json.loads(line)
        except json.JSONDecodeError: continue
        if rec.get("group") != "B_fixtures": continue
        events.append(rec)

    if not events:
        print("no B_fixtures events"); return 1

    print(f"{len(events)} B_fixtures records in {args.jsonl.name}")
    print()

    # Cluster into bursts.
    bursts: list[list[dict]] = []
    cur: list[dict] = []
    for ev in events:
        if cur and ev["ms"] - cur[-1]["ms"] > BURST_GAP_MS:
            bursts.append(cur); cur = []
        cur.append(ev)
    if cur: bursts.append(cur)

    print(f"{len(bursts)} bursts (gap > {BURST_GAP_MS//1000}s)")
    print()

    for i, b in enumerate(bursts, 1):
        entries = [e for e in b if e.get("phase") == "enter"]
        counts = Counter(e["hook"] for e in entries)
        first, last = b[0], b[-1]
        rng_first = first.get("rng")
        rng_last = last.get("rng")
        # Just print entry-phase-only counts to make it readable.
        hooks_line = ", ".join(
            f"{h.split('_', 1)[0]}={n}"
            for h, n in sorted(counts.items(), key=lambda kv: -kv[1])
        )
        # Getter arg0 values in this burst
        getter_args = [
            e.get("arg0") for e in entries
            if e.get("hook") == "sub_0055F540_schedule_getter"
        ]
        driver_args = [
            e.get("comp_ptr") for e in entries
            if e.get("hook") == "FUN_00668450_shared_driver"
        ]
        # Print
        dur_s = (last["ms"] - first["ms"]) / 1000.0
        print(f"burst {i:>2}  ms {first['ms']:>8d} -> {last['ms']:>8d}  "
              f"({dur_s:5.1f}s)  {len(b):>3} events")
        print(f"        rng: {rng_first} -> {rng_last}   "
              f"delta (mod 2^32): {rng_step(rng_first, rng_last)}")
        print(f"        {hooks_line}")
        if getter_args:
            uniq = Counter(getter_args)
            print(f"        getter arg0: "
                  f"{dict(uniq.most_common())}")
        if driver_args:
            uniq = Counter(driver_args)
            top = uniq.most_common(6)
            print(f"        driver arg0 (top {len(top)}): "
                  f"{dict(top)}  ({len(uniq)} distinct)")
        print()

    # Cross-burst summary
    getter_calls = [
        e for e in events
        if e.get("phase") == "enter"
        and e.get("hook") == "sub_0055F540_schedule_getter"
    ]
    print("=" * 60)
    print(f"schedule_getter total: {len(getter_calls)} calls")
    for c in getter_calls:
        print(f"   ms {c['ms']:>8d}  arg0={c.get('arg0'):>10s}  "
              f"rng_at_entry={c.get('rng')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
