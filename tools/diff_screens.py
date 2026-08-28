"""Diff a captured cm0102.exe screen (ground truth, from tools/capture_all_screens.py)
against the cm-render port's output for the same screen.

Workflow:
  1) python tools/capture_all_screens.py <name>
        -> reports/screen_captures/<name>.json   (exe-emulated ground truth)
  2) cargo run -p cm-render --bin dump_screen_geometry -- <name>
        -> reports/screen_captures/<name>.render.json  (ported code's output)
  3) python tools/diff_screens.py <name>
        -> summary of matching / mismatched widgets + first N diffs

We compare AREAS and OBJECTS positionally in insertion order (both sources produce
them in the same widget-creation order). A widget is considered matching if every
compared field is equal. Rectangles are compared as (L, T, R, B); the object type
and font are also required to match.

Exit code 0 iff every widget matches. Non-zero + a printed diff otherwise.
"""
import argparse
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
DIR = REPO / "reports" / "screen_captures"

AREA_KEYS = ["L", "T", "R", "B", "cntA", "wA", "cntB", "wB", "flags"]
OBJ_KEYS  = ["type", "L", "T", "R", "B", "rflags", "font"]


def load(p):
    if not p.exists():
        sys.exit(f"missing: {p}")
    return json.loads(p.read_text())


def diff_lists(name, keys, exe_list, render_list, max_show):
    n = max(len(exe_list), len(render_list))
    mismatches = []
    if len(exe_list) != len(render_list):
        mismatches.append(f"COUNT: exe={len(exe_list)} render={len(render_list)}")
    for i in range(min(len(exe_list), len(render_list))):
        e = exe_list[i]; r = render_list[i]
        d = {k: (e.get(k), r.get(k)) for k in keys if e.get(k) != r.get(k)}
        if d:
            mismatches.append(f"  #{i}: " + ", ".join(f"{k}: exe={ev} render={rv}" for k, (ev, rv) in d.items()))
    if mismatches:
        print(f"[{name}] {len(mismatches)} mismatch(es):")
        for m in mismatches[:max_show]:
            print("  " + m)
        if len(mismatches) > max_show:
            print(f"  ... {len(mismatches) - max_show} more")
    else:
        print(f"[{name}] MATCH ({len(exe_list)})")
    return len(mismatches)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("screen")
    ap.add_argument("--max-show", type=int, default=20)
    args = ap.parse_args()

    exe    = load(DIR / f"{args.screen}.json")
    render = load(DIR / f"{args.screen}.render.json")

    a_mm = diff_lists("areas",   AREA_KEYS, exe["areas"],   render["areas"],   args.max_show)
    o_mm = diff_lists("objects", OBJ_KEYS,  exe["objects"], render["objects"], args.max_show)

    if a_mm == 0 and o_mm == 0:
        print("PIXEL-EXACT (geometry) match.")
        return 0
    print(f"MISMATCH: areas={a_mm} objects={o_mm}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
