#!/usr/bin/env python3
"""Executable reachability graph for the GDI registry.

Turns the static DIRECT call graph into a practical reachability graph that also
carries INDIRECT dispatch edges (function pointers, vtables, menu/AI/competition
dispatch) recovered with provenance. Direct evidence is preserved separately;
indirect evidence is additive and only PROVEN/STRONG edges promote reachability.

Consumed by build_registry.py. Not a standalone entry point (import compute()).

Data sources (build-time only, external carve):
  callgraph.json  direct caller->callee edges
  xrefs.json      per-function referencing addresses (code loads + data-table slots)
  functions.json  entry/size (for containing-function of a code ref)
Curated indirect edges: tools/gdi_registry/data/indirect_edges.csv
"""
import bisect, csv, json, os
from collections import defaultdict, deque

LO, HI = 0x00401000, 0x00922a85          # game .text span
DATA_START = 0x00922a86                    # .rdata/.data/.idata begin after .text

# Named live-entry root sets. Each: list of (addr, semantic, evidence). Addresses
# without a confirmed decode are omitted rather than guessed.
ROOT_SETS = {
    "BOOT": [
        ("0x00803e00", "startup / seed (CM3_QSTART)", "reports:execution-model"),
        ("0x005b6940", "init / DB-load entry", "reports:execution-model"),
        ("0x008120d0", "new-game init", "memory:start-new-game-flow"),
        ("0x005121a0", "load pools", "memory:start-new-game-flow"),
    ],
    "DAILY_TICK": [
        ("0x005b6f10", "per-day tick driver", "cg:calls all 9 tick-step hooks"),
    ],
    "AI": [
        ("0x005b85b0", "daily AI dispatcher (game.cpp step 10)", "memory:sim-findings"),
    ],
    "UI": [
        ("0x007491e0", "menu/widget dispatcher", "memory:menu-command-tree"),
        ("0x0074bf60", "menu/widget sub-dispatcher", "memory:menu-command-tree"),
    ],
    # COMPETITION / MATCH / SAVE_LOAD roots are seeded from curated indirect_edges
    # (root_hint) + these known anchors; extended as agents confirm scheduler roots.
    "MATCH": [
        ("0x00699640", "match build", "curated"),
        ("0x00699d90", "match play", "curated"),
    ],
}

def _h(x):
    s = str(x).lower().replace("0x", "")
    if not s or any(c not in "0123456789abcdef" for c in s):
        return None
    return int(s, 16)
def _fmt(a):
    return "0x%08x" % a

def compute(carve, data_dir):
    fj = json.load(open(os.path.join(carve, "ghidra_out", "cm0102.exe", "functions.json"), encoding="utf-8"))
    cg = json.load(open(os.path.join(carve, "ghidra_out", "cm0102.exe", "callgraph.json"), encoding="utf-8"))
    try:
        xr = json.load(open(os.path.join(carve, "ghidra_out", "cm0102.exe", "xrefs.json"), encoding="utf-8"))
    except Exception:
        xr = []
    # containing-function interval map
    ents = sorted((_h(f["entry"]), int(f.get("size", 0) or 0)) for f in fj if _h(f["entry"]) is not None)
    starts = [e[0] for e in ents]
    def containing(addr):
        i = bisect.bisect_right(starts, addr) - 1
        if i < 0:
            return None
        s, sz = ents[i]
        return s if addr < s + max(sz, 1) else None
    inspan = lambda a: a is not None and LO <= a <= HI

    # direct edges
    direct_out = defaultdict(set); direct_in = defaultdict(set)
    for e in cg:
        a = _h(e.get("from")); b = _h(e.get("to"))
        if a is None or b is None:
            continue
        direct_out[a].add(b); direct_in[b].add(a)

    # address-taken from xrefs: code loads (not a direct call) + data-table slots
    taken_code = defaultdict(list); taken_data = defaultdict(list)
    for x in xr:
        F = _h(x.get("entry"))
        if not inspan(F):
            continue
        for ref in x.get("from", []):
            r = _h(ref)
            if r is None:
                continue
            if LO <= r <= HI:
                G = containing(r)
                if G is not None and F in direct_out.get(G, ()):
                    continue
                taken_code[F].append(_fmt(r))
            elif r >= DATA_START:
                taken_data[F].append(_fmt(r))

    # curated indirect edges
    ind = []  # (src, dst, etype, conf, root_hint, evidence, provenance)
    p = os.path.join(data_dir, "indirect_edges.csv")
    if os.path.exists(p):
        with open(p, encoding="utf-8") as f:
            rd = csv.reader(f)
            for row in rd:
                if not row or not row[0].strip() or row[0].strip().startswith("#"):
                    continue
                row = (row + [""] * 7)[:7]
                s = _h(row[0]); d = _h(row[1])
                if s is None or d is None:
                    continue
                ind.append((s, d, row[2].strip().upper(), row[3].strip().upper(),
                            row[4].strip().upper(), row[5].strip(), row[6].strip()))
    # promoting edges = STRONG|PROVEN; POSSIBLE recorded but non-promoting
    ind_out_promote = defaultdict(list)   # src -> [(dst, etype, conf, prov)]
    ind_in_all = defaultdict(list)        # dst -> [(src, etype, conf, prov)]
    for s, d, et, cf, rh, ev, pv in ind:
        ind_in_all[d].append((s, et, cf, pv))
        if cf in ("PROVEN", "STRONG"):
            ind_out_promote[s].append((d, et, cf, pv))

    # BFS helper over a chosen adjacency, returns depth + parent(for provenance)
    def bfs(roots, use_indirect):
        depth = {}; parent = {}
        dq = deque()
        for r in roots:
            if r not in depth:
                depth[r] = 0; parent[r] = None; dq.append(r)
        while dq:
            u = dq.popleft()
            for v in direct_out.get(u, ()):
                if v not in depth:
                    depth[v] = depth[u] + 1; parent[v] = (u, "DIRECT_CALL"); dq.append(v)
            if use_indirect:
                for (v, et, cf, pv) in ind_out_promote.get(u, ()):
                    if v not in depth:
                        depth[v] = depth[u] + 1; parent[v] = (u, et); dq.append(v)
        return depth, parent

    # Effective roots per set = declared ROOT_SETS + any curated-edge src whose
    # root_hint names that set (lets a proven scheduler/dispatcher seed its set).
    eff_roots = {name: [_h(a) for (a, _, _) in lst if _h(a) is not None]
                 for name, lst in ROOT_SETS.items()}
    for s, d, et, cf, rh, ev, pv in ind:
        if rh and cf in ("PROVEN", "STRONG"):
            eff_roots.setdefault(rh, [])
            if s not in eff_roots[rh]:
                eff_roots[rh].append(s)
    all_roots = [r for rs in eff_roots.values() for r in rs if r is not None]

    d_depth, _ = bfs(all_roots, use_indirect=False)            # DIRECT only
    i_depth, i_parent = bfs(all_roots, use_indirect=True)       # + STRONG/PROVEN
    # per root-set reachability (indirect graph)
    rootset_reach = {}
    for name, rs in eff_roots.items():
        dep, _ = bfs(rs, use_indirect=True)
        rootset_reach[name] = set(dep)

    def provenance(va):
        # reconstruct one path from a root to va over the indirect BFS tree
        if va not in i_parent:
            return ""
        chain = []
        cur = va; guard = 0
        while cur is not None and guard < 64:
            chain.append(_fmt(cur)); pr = i_parent.get(cur)
            if pr is None:
                break
            cur = pr[0]; guard += 1
        return " <- ".join(chain)

    out = {}
    # union of every va we know about (functions + edge endpoints)
    vas = set(_h(f["entry"]) for f in fj if _h(f["entry"]) is not None)
    for va in vas:
        if not inspan(va):
            continue
        dv = _fmt(va)
        direct_reach = va in d_depth
        indirect_reach = (not direct_reach) and (va in i_depth)
        at_code = taken_code.get(va, []); at_data = taken_data.get(va, [])
        addr_taken = bool(at_code or at_data)
        poss_in = ind_in_all.get(va, [])
        # 5-state model (spec): DEAD_OR_UNREACHABLE is NEVER auto-assigned — it
        # requires positive evidence (a curated status), so absence of xrefs maps
        # to UNRESOLVED, not dead. Raw address-taking is POSSIBLE_INDIRECT, never
        # promoted to reachable without a PROVEN/STRONG dispatch edge.
        if direct_reach:
            kind = "DIRECT_REACHABLE"
        elif indirect_reach:
            kind = "INDIRECT_REACHABLE"
        elif addr_taken or poss_in:
            kind = "POSSIBLE_INDIRECT"
        else:
            kind = "UNRESOLVED"
        rsets = [n for n, s in rootset_reach.items() if va in s]
        # incoming indirect edges (why it's indirect-reachable / address-taken)
        in_types = sorted({et for (_, et, _, _) in poss_in})
        in_conf = ""
        confs = {cf for (_, _, cf, _) in poss_in}
        for c in ("PROVEN", "STRONG", "POSSIBLE"):
            if c in confs:
                in_conf = c; break
        at = "+".join(x for x in [("data" if at_data else ""), ("code" if at_code else "")] if x)
        out[dv] = {
            "cg_direct_reach": "Y" if direct_reach else "",
            "cg_indirect_reach": "Y" if indirect_reach else "",
            "cg_reach_kind": kind,
            "cg_root_sets": ";".join(rsets),
            "cg_min_depth": i_depth.get(va, ""),
            "cg_direct_depth": d_depth.get(va, ""),
            "callers": len(direct_in.get(va, ())),
            "callees": len(direct_out.get(va, ())),
            "cg_direct_callers": len(direct_in.get(va, ())),
            "cg_indirect_callers": len(poss_in),
            "cg_address_taken": at,
            "cg_data_ref_count": len(at_data),
            "cg_code_ref_count": len(at_code),
            "cg_edge_types": ";".join(in_types),
            "cg_confidence": in_conf,
            "cg_provenance": provenance(va) if (direct_reach or indirect_reach) else "",
        }
    # roots documentation = declared ROOT_SETS + roots seeded from proven edge hints
    roots_doc = {name: list(lst) for name, lst in ROOT_SETS.items()}
    declared_by_set = {name: {_h(a) for (a, _, _) in lst} for name, lst in ROOT_SETS.items()}
    for s, d, et, cf, rh, ev, pv in ind:
        if rh and cf in ("PROVEN", "STRONG") and s not in declared_by_set.get(rh, set()):
            roots_doc.setdefault(rh, [])
            if s not in {_h(a) for (a, _, _) in roots_doc[rh]}:
                roots_doc[rh].append((_fmt(s), "seeded by %s dispatch edge" % et,
                                       pv[:80] or ev[:80] or "indirect_edges.csv"))
                declared_by_set.setdefault(rh, set()).add(s)
    n_ind = len(ind)
    return out, roots_doc, n_ind
