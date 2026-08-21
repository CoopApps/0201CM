#!/usr/bin/env python3
"""Run the layout-resolution pass over every UI builder: resolve grid-area
column geometry (final pixel x-bounds) game-wide."""
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
import resolve_layout as rl

BUILDERS = "D:/temp/claude/D--cm0102-rs/1a091c9d-7a7f-448e-b330-6e37867ef521/scratchpad/ui_builders.json"
OUT = "D:/cm0102-rs/reports/carve_segment_index/fixtures/all_layouts.json"

def main():
    builders = json.load(open(BUILDERS))
    db = {}
    areas_total = areas_resolved = 0
    errors = 0
    for i, b in enumerate(builders):
        try:
            r = rl.resolve(int(b, 16))
        except Exception:
            errors += 1
            continue
        areas_total += len(r["areas"])
        if r["resolved_areas"]:
            areas_resolved += len(r["resolved_areas"])
            db[b] = {"resolved_areas": r["resolved_areas"], "child_count": r["child_count"]}
        if i % 500 == 0:
            print(f"  {i}/{len(builders)}  builders-with-resolved-grids={len(db)}  areas_resolved={areas_resolved}")
    summary = {
        "builders_scanned": len(builders),
        "builders_with_resolved_grids": len(db),
        "areas_total_seen": areas_total,
        "areas_resolved": areas_resolved,
        "errors": errors,
    }
    Path(OUT).write_text(json.dumps({"summary": summary, "layouts": db}, indent=1))
    print(json.dumps(summary, indent=1))
    print("wrote", OUT)

if __name__ == "__main__":
    main()
