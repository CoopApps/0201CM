#!/usr/bin/env python3
"""Run the guio object-tree extractor over EVERY UI builder in the exe.

A builder = any function that calls the GUI/area constructors. Emits one combined
database of screen object trees + a summary, so the whole game's UI is extracted
in a single pass instead of one screen at a time.
"""
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
import extract_screen_tree as est

BUILDERS = "D:/temp/claude/D--cm0102-rs/1a091c9d-7a7f-448e-b330-6e37867ef521/scratchpad/ui_builders.json"
OUT = "D:/cm0102-rs/reports/carve_segment_index/fixtures/all_screen_trees.json"

def main():
    builders = json.load(open(BUILDERS))
    db = {}
    errors = 0
    for i, b in enumerate(builders):
        try:
            tree = est.extract(int(b, 16))
            if tree["object_count"] > 0:
                db[b] = tree
        except Exception:
            errors += 1
        if i % 500 == 0:
            print(f"  {i}/{len(builders)} scanned, {len(db)} with objects, {errors} errors")
    # summary
    total_objs = sum(t["object_count"] for t in db.values())
    with_text = sum(1 for t in db.values() for o in t["objects"] if o.get("text"))
    big = sorted(db.items(), key=lambda kv: -kv[1]["object_count"])[:20]
    summary = {
        "builders_scanned": len(builders),
        "builders_with_objects": len(db),
        "total_objects": total_objs,
        "objects_with_text": with_text,
        "errors": errors,
        "largest_screens": [{"builder": k, "objects": v["object_count"]} for k, v in big],
    }
    Path(OUT).write_text(json.dumps({"summary": summary, "screens": db}, indent=1))
    print(json.dumps(summary, indent=1))
    print("wrote", OUT)

if __name__ == "__main__":
    main()
