#!/usr/bin/env python3
"""Run the symbolic executor over every UI builder: emit each object's bounds as
formulas, game-wide. The generation-rule database."""
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
import symbolic_extract as se

BUILDERS = "D:/temp/claude/D--cm0102-rs/1a091c9d-7a7f-448e-b330-6e37867ef521/scratchpad/ui_builders.json"
OUT = "D:/cm0102-rs/reports/carve_segment_index/fixtures/all_bound_formulas.json"

def is_formula(s):
    # a bound that is a real expression / named input (not a bare register or 0)
    return s and (any(k in s for k in ("body_", "font", "g[", "+", "-", "*", "<<")) )

def main():
    builders = json.load(open(BUILDERS))
    db = {}; total = with_formula = errors = 0
    from collections import Counter
    input_kinds = Counter()
    for i, b in enumerate(builders):
        try:
            objs = se.extract(int(b, 16))["objects"]
        except Exception:
            errors += 1; continue
        keep = []
        for o in objs:
            total += 1
            bd = [o.get("left"), o.get("top"), o.get("right"), o.get("bottom")]
            if any(is_formula(x) for x in bd):
                with_formula += 1
                for x in bd:
                    if x and "body_" in x: input_kinds["body_region"] += 1
                    if x and "font" in x: input_kinds["font_height"] += 1
                    if x and "g[" in x: input_kinds["data_global"] += 1
                keep.append(o)
        if keep:
            db[b] = keep
        if i % 500 == 0:
            print(f"  {i}/{len(builders)}  builders_with_formulas={len(db)}  objects_with_formulas={with_formula}")
    summary = {"builders_scanned": len(builders), "builders_with_formulas": len(db),
               "objects_total": total, "objects_with_bound_formulas": with_formula,
               "errors": errors, "formula_input_kinds": dict(input_kinds)}
    Path(OUT).write_text(json.dumps({"summary": summary, "builders": db}, indent=1))
    print(json.dumps(summary, indent=1)); print("wrote", OUT)

if __name__ == "__main__":
    main()
