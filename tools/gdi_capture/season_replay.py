#!/usr/bin/env python3
"""Partition + summarise a season_capture.jsonl produced by
season_capture.py. Splits into per-group files so each differential
can run independently, and prints a call-count summary.

Usage:
    D:/Python312/python.exe tools/gdi_capture/season_replay.py \
        season_2001_02.jsonl --split-dir season_split/
"""

from __future__ import annotations
import argparse
import collections
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("jsonl", type=Path)
    ap.add_argument("--split-dir", type=Path, default=None)
    return ap.parse_args()


def main() -> int:
    args = parse_args()
    if not args.jsonl.exists():
        print(f"missing: {args.jsonl}"); return 2

    per_group_counts: dict[str, collections.Counter] = collections.defaultdict(
        collections.Counter)
    per_group_files = None
    if args.split_dir:
        args.split_dir.mkdir(parents=True, exist_ok=True)
        per_group_files = {}

    total = 0
    init_records = []
    with args.jsonl.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line: continue
            try: rec = json.loads(line)
            except json.JSONDecodeError:
                continue
            total += 1
            group = rec.get("group", "_meta")
            hook = rec.get("hook", "?")
            phase = rec.get("phase", "?")
            per_group_counts[group][f"{hook}:{phase}"] += 1
            if hook == "__init__":
                init_records.append(rec)
            if per_group_files is not None:
                if group not in per_group_files:
                    per_group_files[group] = (
                        args.split_dir / f"{group}.jsonl"
                    ).open("w", encoding="utf-8")
                per_group_files[group].write(line + "\n")

    print(f"total events: {total}")
    print("per-group / per-hook counts:")
    for g in sorted(per_group_counts):
        print(f"  {g}:")
        for k, v in sorted(per_group_counts[g].items()):
            print(f"      {k:60s} {v:>8d}")

    if init_records:
        print("\ninit summary:")
        for r in init_records:
            for gname, hooks in (r.get("install_report", {})
                                  .get("groups", {}).items()):
                enabled = [h for h in hooks if h.get("hooked")]
                skipped = [h for h in hooks if h.get("skipped")]
                print(f"  {gname}: {len(enabled)} armed, {len(skipped)} skipped")
                for h in skipped:
                    print(f"      skip {h['name']}: {h.get('reason')}")

    if per_group_files is not None:
        for f in per_group_files.values(): f.close()
        print(f"\nsplit → {args.split_dir}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
