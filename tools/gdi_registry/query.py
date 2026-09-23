#!/usr/bin/env python3
"""Query the GDI function registry both ways.

  python tools/gdi_registry/query.py 00669340        # GDI/DD address -> Rust
  python tools/gdi_registry/query.py season_label    # Rust symbol/file -> GDI fns
  python tools/gdi_registry/query.py domestic_cup    # subsystem / file substring
  python tools/gdi_registry/query.py --status NOT_YET_PORTED   # list by status

Reads docs/gdi_registry/gdi_function_registry.json (run build_registry.py first).
"""
import json, os, re, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
REG = os.path.join(ROOT, "docs", "gdi_registry", "gdi_function_registry.json")

def load():
    if not os.path.exists(REG):
        sys.exit(f"registry not found: {REG}\nrun: python tools/gdi_registry/build_registry.py")
    return json.load(open(REG, encoding="utf-8"))

def show(r):
    print(f"{r['dd_va']}  [{r['status']}]  {r['semantic_name'] or r['symbol']}")
    print(f"    subsystem : {r['subsystem']}   source: {r['source_file']}")
    print(f"    rust      : {r['rust_symbol'] or '-'}  ({r['rust_file'] or '-'})")
    print(f"    reach     : {r['reachable']}   relevance: {r['relevance']}   conf: {r['confidence']}")
    if r['evidence']: print(f"    evidence  : {r['evidence']}")
    if r['notes']: print(f"    notes     : {r['notes']}")
    if r['superseded_prev']: print(f"    superseded: was '{r['superseded_prev']}' -> {r['superseded_why']}")

def main():
    rows = load()
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    if args[0] == "--status":
        st = args[1].upper()
        hits = [r for r in rows if r["status"] == st]
        for r in sorted(hits, key=lambda x: x["subsystem"]):
            print(f"{r['dd_va']}  {r['subsystem']:20}  {r['semantic_name'] or r['symbol']}")
        print(f"\n{len(hits)} functions with status {st}")
        return
    q = args[0]
    m = re.fullmatch(r'(?:0x|FUN_|sub_)?([0-9a-fA-F]{6,8})', q)
    if m:
        va = "0x%08x" % int(m.group(1), 16)
        hit = [r for r in rows if r["dd_va"] == va or r["gdi_va"].lower() == va]
        if not hit:
            print(f"no registry row for {va}")
        for r in hit:
            show(r)
        return
    ql = q.lower()
    hits = [r for r in rows if ql in (r["rust_symbol"] or "").lower()
            or ql in (r["rust_file"] or "").lower()
            or ql in (r["semantic_name"] or "").lower()
            or ql in (r["subsystem"] or "").lower()
            or ql in (r["source_file"] or "").lower()]
    for r in sorted(hits, key=lambda x: (x["subsystem"], x["dd_va"])):
        print(f"{r['dd_va']}  [{r['status']:18}]  {r['semantic_name'] or r['symbol']:40}  {r['rust_symbol'] or r['rust_file']}")
    print(f"\n{len(hits)} match(es) for '{q}'")

if __name__ == "__main__":
    main()
