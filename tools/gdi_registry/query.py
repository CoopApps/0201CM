#!/usr/bin/env python3
"""Query the GDI function registry both ways.

  python tools/gdi_registry/query.py 00669340        # GDI/DD address -> Rust
  python tools/gdi_registry/query.py season_label    # Rust symbol/file -> GDI fns
  python tools/gdi_registry/query.py domestic_cup    # subsystem / file substring
  python tools/gdi_registry/query.py --status NOT_YET_PORTED   # list by status
  python tools/gdi_registry/query.py --indirect-only     # INDIRECT_REACHABLE rows
  python tools/gdi_registry/query.py --possible-indirect # POSSIBLE_INDIRECT (address-taken, unproven)
  python tools/gdi_registry/query.py --unresolved        # UNRESOLVED (no reference found)
  python tools/gdi_registry/query.py --zero-direct-caller# 0 direct callers
  python tools/gdi_registry/query.py --root UI_ROOTS     # reachable from a named root set

Reads docs/gdi_registry/gdi_function_registry.json (run build_registry.py first).
An address query also prints direct callers/callees counts, reachability kind,
root sets, indirect edge types/confidence, and the provenance path.
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
    # executable reachability (call-graph axis, separate from Rust `reachable`)
    print(f"    exe-reach : {r.get('cg_reach_kind','?')}  depth={r.get('cg_min_depth','')}  "
          f"roots=[{r.get('cg_root_sets','')}]")
    print(f"    callers   : direct={r.get('cg_direct_callers', r.get('callers',''))} "
          f"indirect={r.get('cg_indirect_callers','')}   callees={r.get('callees','')}")
    if r.get('cg_address_taken'):
        print(f"    addr-taken: {r['cg_address_taken']}  (data_refs={r.get('cg_data_ref_count','')} "
              f"code_refs={r.get('cg_code_ref_count','')})")
    if r.get('cg_edge_types'):
        print(f"    ind-edges : {r['cg_edge_types']} (conf {r.get('cg_confidence','')})")
    if r.get('cg_provenance'): print(f"    provenance: {r['cg_provenance']}")

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
    if args[0] == "--root":
        rs = args[1] if len(args) > 1 else ""
        rsn = rs.upper().replace("_ROOTS", "")
        hits = [r for r in rows if rsn in (r.get("cg_root_sets", "") or "").upper().split(";")]
        for r in sorted(hits, key=lambda x: (x.get("cg_min_depth", 9999) if x.get("cg_min_depth") != "" else 9999, x["dd_va"])):
            print(f"{r['dd_va']}  d{r.get('cg_min_depth','')}  [{r.get('cg_reach_kind',''):18}]  "
                  f"{r['subsystem']:18}  {r['semantic_name'] or r['symbol']}")
        print(f"\n{len(hits)} rows reachable from root set {rsn}")
        return
    KINDMAP = {"--indirect-only": "INDIRECT_REACHABLE", "--possible-indirect": "POSSIBLE_INDIRECT",
               "--unresolved": "UNRESOLVED", "--direct": "DIRECT_REACHABLE",
               "--dead": "DEAD_OR_UNREACHABLE"}
    if args[0] in KINDMAP or args[0] in ("--zero-caller", "--zero-direct-caller"):
        if args[0] in ("--zero-caller", "--zero-direct-caller"):
            hits = [r for r in rows if str(r.get("cg_direct_callers", r.get("callers", ""))) in ("0", "")]
        else:
            hits = [r for r in rows if r.get("cg_reach_kind") == KINDMAP[args[0]]]
        for r in sorted(hits, key=lambda x: (x["subsystem"], x["dd_va"])):
            print(f"{r['dd_va']}  [{r['status']:16}]  {r.get('cg_reach_kind',''):20}  "
                  f"{r['subsystem']:18}  {r['semantic_name'] or r['symbol']}")
        print(f"\n{len(hits)} rows for {args[0]}")
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
