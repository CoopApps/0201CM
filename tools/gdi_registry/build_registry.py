#!/usr/bin/env python3
"""Build the canonical GDI function registry that bridges the decompiled
cm0102 executable to the Rust port.

Sources merged (curated always wins over auto-derived):
  1. functions.json + carver atlas   (D:/cm0102-carve; build-time only)
  2. live grep of crates/**/*.rs      (FUN_/sub_ citations, // GDI-REG: tags)
  3. tools/gdi_registry/data/*.csv    (curated overlays, hand-authored)

Outputs (committed): docs/gdi_registry/gdi_function_registry.{md,csv,json},
gdi_function_coverage.md, gdi_globals.md.

Run: python tools/gdi_registry/build_registry.py
"""
import bisect, csv, json, os, re, sys
from collections import defaultdict, Counter

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
CARVE = os.environ.get("CM_CARVE", "D:/cm0102-carve")
DATA = os.path.join(os.path.dirname(__file__), "data")
OUT = os.path.join(ROOT, "docs", "gdi_registry")
CRATES = os.path.join(ROOT, "crates")

FUNCS_JSON = os.path.join(CARVE, "ghidra_out", "cm0102.exe", "functions.json")
ATLAS_JSON = os.path.join(CARVE, "newcarve", "atlas", "subsystem_map.json")
CALLGRAPH_JSON = os.path.join(CARVE, "ghidra_out", "cm0102.exe", "callgraph.json")

# Live entry roots for exe-reachability (distinct from the Rust `reachable` field).
# 0x005b6f10 is the per-day tick driver (only fn calling all 9 known tick-step
# hooks); the rest are boot / new-game / message-pump entries.
CG_ROOTS = ["0x005b6f10", "0x005b6a90", "0x005b6940", "0x008120d0",
            "0x005121a0", "0x00803e00", "0x00672770"]

STATUSES = {"PORTED_EXACT","PORTED_BEHAVIOURAL","PORTED_PARTIAL","REPLACED_BY_RUST",
    "NOT_YET_PORTED","BLOCKED_DEPENDENCY","FOREIGN_BREADTH","UI_GDI_DOMAIN",
    "OUT_OF_SCOPE","NON_USEFUL","DEAD_OR_UNREACHABLE","UNKNOWN"}
PORTED = {"PORTED_EXACT","PORTED_BEHAVIOURAL","PORTED_PARTIAL"}

CITE_RE = re.compile(r'\b(?:FUN|sub)_([0-9a-fA-F]{6,8})\b')
GDIREG_RE = re.compile(r'GDI-REG:\s*([0-9a-fA-F]{6,8})\s+([A-Z_]+)')

def norm(h):
    h = h.lower().lstrip("0x")
    return "0x%08x" % int(h, 16)

def load_json(p):
    try:
        return json.load(open(p, encoding="utf-8"))
    except Exception as e:
        print(f"[warn] could not load {p}: {e}", file=sys.stderr)
        return None

def load_csv(name):
    p = os.path.join(DATA, name)
    if not os.path.exists(p):
        return []
    with open(p, encoding="utf-8") as f:
        return list(csv.DictReader(f))

def main():
    funcs = load_json(FUNCS_JSON) or []
    by_va = {}
    for f in funcs:
        try:
            by_va[norm(f["entry"])] = f
        except Exception:
            pass
    # Useful game-logic universe = functions inside the game .text span
    # 0x00401000..0x00922a85 (strictly alphabetical by .cpp). Everything OUTSIDE
    # this span is CRT/MSVC-runtime/STL/Win32-DirectDraw-wrapper plumbing and is
    # deliberately excluded. See reports/decompile_coverage_review.md methodology.
    SPAN_LO, SPAN_HI = 0x00401000, 0x00922a85
    span_useful = set()
    for va in by_va:
        try:
            if SPAN_LO <= int(va, 16) <= SPAN_HI:
                span_useful.add(va)
        except Exception:
            pass
    atlas_raw = load_json(ATLAS_JSON) or {}
    atlas = {}
    _BS = chr(92)  # backslash; atlas paths use \code\...\<file>.cpp (single or doubled)
    def _cpp_basename(s):
        # Robust: no fragile backslash regex. Take the <name>.cpp token after the
        # last path separator. Returns lowercase basename or "".
        low = s.lower()
        i = low.rfind(".cpp")
        if i < 0:
            return ""
        start = 0
        for k in range(i - 1, -1, -1):
            if s[k] in (_BS, "/"):
                start = k + 1
                break
        return s[start:i + 4].lower()
    anchors = []  # (int_va, cpp) where the function's own strings name a .cpp
    for k, v in atlas_raw.items():
        try:
            va = norm(k)
        except Exception:
            continue
        cpp = ""
        for s in v.get("string_hits", []):
            if ".cpp" in s.lower():
                cpp = _cpp_basename(s)
                if cpp:
                    break
        atlas[va] = {"subsystem": v.get("subsystem", ""), "cpp": cpp}
        if cpp:
            anchors.append((int(va, 16), cpp))
    # .text is strictly alphabetical by .cpp: attribute every function to the
    # nearest preceding anchor so uncited functions still carry a source file.
    anchors.sort()
    _avas = [a for a, _ in anchors]
    def attr_cpp(va_str):
        try:
            iva = int(va_str, 16)
        except Exception:
            return ""
        j = bisect.bisect_right(_avas, iva) - 1
        return anchors[j][1] if j >= 0 else ""

    # --- call graph: adjacency, degrees, reachability + depth from live roots ---
    cg = load_json(CALLGRAPH_JSON) or []
    callees = defaultdict(set); indeg = Counter(); outdeg = Counter()
    def _cgnorm(h):
        s = str(h).lower().replace("0x", "")
        if not s or any(c not in "0123456789abcdef" for c in s):
            return None
        return "0x%08x" % int(s, 16)
    for e in cg:
        fr = _cgnorm(e.get("from")); to = _cgnorm(e.get("to"))
        if fr is None or to is None or to in callees[fr]:
            continue
        callees[fr].add(to); outdeg[fr] += 1; indeg[to] += 1
    cg_depth = {}
    _dq = __import__("collections").deque()
    for r in CG_ROOTS:
        rn = _cgnorm(r)
        if rn is not None:
            cg_depth[rn] = 0; _dq.append(rn)
    while _dq:
        u = _dq.popleft()
        for v in callees.get(u, ()):
            if v not in cg_depth:
                cg_depth[v] = cg_depth[u] + 1; _dq.append(v)

    # --- live citations from crates/ ---
    cites = defaultdict(list)       # va -> [(file, line, text)]
    gdireg = {}                     # va -> status token from // GDI-REG:
    rs_paths = {}                   # basename.rs -> repo-relative path (for resolving bare names)
    for dirpath, _, files in os.walk(CRATES):
        for fn in files:
            if not fn.endswith(".rs"):
                continue
            fp = os.path.join(dirpath, fn)
            rel = os.path.relpath(fp, ROOT).replace("\\", "/")
            rs_paths.setdefault(fn, rel)
            try:
                lines = open(fp, encoding="utf-8", errors="replace").read().splitlines()
            except Exception:
                continue
            for i, ln in enumerate(lines, 1):
                for m in CITE_RE.finditer(ln):
                    va = norm(m.group(1))
                    cites[va].append((rel, i, ln.strip()[:200]))
                g = GDIREG_RE.search(ln)
                if g:
                    gdireg[norm(g.group(1))] = g.group(2)

    # Curated rows: MERGE multiple rows for the same DD VA (one exe fn may map to
    # several Rust symbols / carry conflicting interpretations). Status by
    # precedence; rust/evidence joined; notes preserved (incl. conflicts).
    STATUS_PREC = ["PORTED_EXACT","PORTED_BEHAVIOURAL","PORTED_PARTIAL","REPLACED_BY_RUST",
        "BLOCKED_DEPENDENCY","NOT_YET_PORTED","FOREIGN_BREADTH","UI_GDI_DOMAIN",
        "OUT_OF_SCOPE","NON_USEFUL","DEAD_OR_UNREACHABLE","UNKNOWN"]
    def sprec(s): return STATUS_PREC.index(s) if s in STATUS_PREC else len(STATUS_PREC)
    cur_groups = defaultdict(list)
    for r in load_csv("curated.csv"):
        if r.get("dd_va"):
            cur_groups[norm(r["dd_va"])].append(r)
    def joinf(rowset, key):
        seen, out = set(), []
        for r in rowset:
            v = (r.get(key) or "").strip()
            for part in v.split(";"):
                part = part.strip()
                if part and part not in seen:
                    seen.add(part); out.append(part)
        return ";".join(out)
    curated = {}
    for va, rs in cur_groups.items():
        rs = sorted(rs, key=lambda r: sprec((r.get("status") or "UNKNOWN").strip()))
        primary = rs[0]
        notes = " | ".join(dict.fromkeys(r.get("notes","").strip() for r in rs if r.get("notes","").strip()))
        if len(rs) > 1:
            notes = (notes + " | MULTI-ROLE: " + "; ".join(
                f"{(r.get('status') or '').strip()}={r.get('semantic_name','') or r.get('rust_symbol','')}"
                for r in rs)).strip(" |")
        m = dict(primary)
        m["rust_file"] = joinf(rs, "rust_file")
        m["rust_symbol"] = joinf(rs, "rust_symbol")
        m["evidence"] = joinf(rs, "evidence")
        m["notes"] = notes
        curated[va] = m
    def _base(p): return p.strip().lower().replace("\\", "/").split("/")[-1]
    groups = {_base(r["source_file"]): r for r in load_csv("frontier_groups.csv") if r.get("source_file")}
    superseded = {norm(r["dd_va"]): r for r in load_csv("superseded.csv") if r.get("dd_va")}
    frontier = set()
    fp = os.path.join(DATA, "frontier_addresses.txt")
    if os.path.exists(fp):
        for l in open(fp):
            l = l.strip()
            if l:
                try: frontier.add(norm(l))
                except Exception: pass

    universe = set(cites) | set(curated) | frontier | set(gdireg) | span_useful

    NEG = ("not implemented","not ported","not yet","frontier","unported","todo",
           "deferred","approximate","stub","see fun_","see the exe","blocked",
           "gap:","missing")
    def cpp_of(va):
        return atlas.get(va, {}).get("cpp") or attr_cpp(va)
    def derive_status(va):
        if va in curated and curated[va].get("status"):
            return curated[va]["status"].strip(), "curated"
        if va in gdireg:
            return gdireg[va], "gdi-reg-tag"
        if va in cites:
            low = " ".join(t for _, _, t in cites[va]).lower()
            # A bare mention in a "not implemented"/frontier/see-also comment is a
            # REFERENCE, not a port -> stay UNKNOWN (cited, pending curation).
            if any(n in low for n in NEG):
                return "UNKNOWN", "cite-reference"
            if "partial" in low: return "PORTED_PARTIAL", "cite-text"
            if "out-of-scope" in low or "out of scope" in low:
                return "OUT_OF_SCOPE", "cite-text"
            # Explicit provenance markers = a real port; otherwise still auto,
            # flagged UNVERIFIED below (confidence), pending curation.
            return "PORTED_BEHAVIOURAL", "cite-default"
        # Uncited/uncurated: apply the frontier audit's per-.cpp file-level default.
        # A function that was individually in the audited frontier set keeps the
        # group's confidence ("frontier-group"); a function swept in only by
        # file-level extension is flagged UNVERIFIED ("cpp-group").
        cpp = cpp_of(va)
        if cpp in groups and groups[cpp].get("status"):
            return groups[cpp]["status"].strip(), ("frontier-group" if va in frontier else "cpp-group")
        if va in frontier:
            return "UNKNOWN", "frontier-unclassified"
        if va in span_useful:
            return "UNKNOWN", "span-useful-unclassified"
        return "UNKNOWN", "no-signal"

    rows = []
    for va in sorted(universe):
        f = by_va.get(va, {})
        a = atlas.get(va, {})
        cu = curated.get(va, {})
        status, how = derive_status(va)
        rustfiles = cu.get("rust_file") or ";".join(sorted({c[0] for c in cites.get(va, [])}))
        # Resolve bare "foo.rs" basenames (agents emit these) to repo paths.
        if rustfiles:
            parts = []
            for p in rustfiles.split(";"):
                p = p.strip()
                if p and "/" not in p and p in rs_paths:
                    p = rs_paths[p]
                if p:
                    parts.append(p)
            rustfiles = ";".join(dict.fromkeys(parts))
        rustsyms = cu.get("rust_symbol", "")
        # HONESTY GUARD: never assert PORTED_* without a Rust link. An
        # auto/group classification that claims ported but has no rust file and
        # is not cited is downgraded to NOT_YET_PORTED.
        if status in PORTED and how != "curated" and not (rustfiles or va in cites):
            status, how = "NOT_YET_PORTED", how + "+no-rust-downgrade"
        # Per-.cpp group metadata (only when the status came from a group default).
        gg = groups.get(cpp_of(va), {}) if how in ("frontier-group", "cpp-group") else {}
        # Auto (non-curated) classifications are UNVERIFIED until a human curates.
        # A file-level group extension (cpp-group) is UNVERIFIED for THIS function;
        # an individually-audited frontier-group row keeps the group's confidence.
        if cu.get("confidence"):
            conf = cu["confidence"]
        elif how == "frontier-group":
            conf = gg.get("confidence") or "STRUCTURALLY_VERIFIED"
        elif how == "cpp-group":
            conf = "UNVERIFIED"
        else:
            conf = "UNVERIFIED" if how != "curated" else "STRUCTURALLY_VERIFIED"
        reach = cu.get("reachable") or ("YES" if status in PORTED and cites.get(va) else
                ("NO" if status in {"NON_USEFUL","OUT_OF_SCOPE","UI_GDI_DOMAIN","DEAD_OR_UNREACHABLE","FOREIGN_BREADTH","NOT_YET_PORTED","BLOCKED_DEPENDENCY"} else "INDIRECT"))
        rel = cu.get("relevance") or gg.get("relevance") or ("IMPLEMENTATION_ONLY" if status == "NON_USEFUL" else
              ("UI_RENDERING" if status == "UI_GDI_DOMAIN" else
               ("" if status == "UNKNOWN" else "SIMULATION_RELEVANT")))
        sup = superseded.get(va, {})
        rows.append({
            "dd_va": va,
            "gdi_va": cu.get("gdi_va", ""),
            "source_file": cu.get("source_file") or a.get("cpp") or attr_cpp(va),
            "symbol": f.get("name", ""),
            "semantic_name": cu.get("semantic_name", ""),
            "subsystem": cu.get("subsystem") or gg.get("subsystem") or a.get("subsystem", ""),
            "status": status,
            "rust_file": rustfiles,
            "rust_symbol": rustsyms,
            "reachable": reach,
            "relevance": rel,
            "confidence": conf,
            "evidence": cu.get("evidence", ""),
            "notes": (cu.get("notes") or ((gg.get("notes", "") + (
                " | file-level group default (not per-function verified)"
                if how == "cpp-group" else "")) if gg else "")),
            "superseded_prev": sup.get("prev", ""),
            "superseded_why": sup.get("why", ""),
            "callers": indeg.get(va, 0),
            "callees": outdeg.get(va, 0),
            "cg_reach": "Y" if va in cg_depth else "",
            "cg_depth": cg_depth.get(va, ""),
            "_how": how,
            "_size": f.get("size", ""),
            "_cited": "Y" if va in cites else "",
        })

    os.makedirs(OUT, exist_ok=True)
    cols = ["dd_va","gdi_va","source_file","symbol","semantic_name","subsystem","status",
            "rust_file","rust_symbol","reachable","relevance","confidence","evidence",
            "notes","superseded_prev","superseded_why","callers","callees","cg_reach","cg_depth"]
    with open(os.path.join(OUT, "gdi_function_registry.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=cols, extrasaction="ignore")
        w.writeheader()
        for r in rows: w.writerow(r)
    json.dump([{k: r[k] for k in cols} for r in rows],
              open(os.path.join(OUT, "gdi_function_registry.json"), "w", encoding="utf-8"),
              indent=1)

    # --- coverage ---
    by_status = Counter(r["status"] for r in rows)
    by_sub = defaultdict(Counter)
    for r in rows: by_sub[r["subsystem"] or "?"][r["status"]] += 1
    ported = sum(by_status[s] for s in PORTED)
    ported_curated = sum(1 for r in rows if r["status"] in PORTED and r["_how"] == "curated")
    ported_auto = ported - ported_curated
    # Classification provenance — how each row's status was decided (honesty).
    def _prov(h):
        if h == "curated": return "curated (hand-verified)"
        if h == "gdi-reg-tag": return "curated (hand-verified)"
        if h == "frontier-group": return "frontier-audit (per-fn audited)"
        if h == "cpp-group": return "file-level default (UNVERIFIED)"
        if h.startswith("cite"): return "auto-cited (UNVERIFIED)"
        if "unclassified" in h or h == "no-signal": return "UNKNOWN (uningested/awaiting)"
        return "auto (UNVERIFIED)"
    prov = Counter(_prov(r["_how"]) for r in rows)
    with open(os.path.join(OUT, "gdi_function_coverage.md"), "w", encoding="utf-8") as f:
        f.write("# GDI function coverage (generated)\n\n")
        f.write(f"Registered functions: **{len(rows)}**  ·  cited in Rust: "
                f"**{sum(1 for r in rows if r['_cited'])}**\n\n")
        n_unknown = by_status["UNKNOWN"]
        n_classified = len(rows) - n_unknown
        f.write(f"**Complete-picture denominator.** The registry now ingests the entire "
                f"useful game-logic universe (the `.text` span `0x00401000..0x00922a85`; "
                f"CRT/runtime plumbing outside the span is excluded). Of **{len(rows)}** useful "
                f"functions, **{n_classified}** are classified and **{n_unknown}** remain "
                f"`UNKNOWN` (ingested, awaiting classification). Drive UNKNOWN to zero via "
                f"mechanical bucketing (NON_USEFUL/FOREIGN_BREADTH/DEAD) + subsystem curation "
                f"waves.\n\n")
        f.write("> **Read carefully.** A `PORTED_*` row derived automatically from a Rust "
                "citation is `confidence: UNVERIFIED` — it means *an exe address is cited in "
                "Rust without a 'not-implemented' marker*, NOT that the port was human-verified. "
                f"Of **{ported}** PORTED_* rows, only **{ported_curated}** are curated/verified; "
                f"**{ported_auto}** are auto-cited and PENDING CURATION. Do not headline the auto "
                "number as real coverage (see the coverage-vs-fidelity antipattern).\n\n")
        f.write("## Classification provenance\n\nHow each row's status was decided — a "
                "file-level default is a defensible per-`.cpp` guess (the audit's group "
                "default extended across the file), NOT a per-function verification.\n\n"
                "| provenance | count |\n|---|---|\n")
        for p in sorted(prov, key=lambda x: -prov[x]):
            f.write(f"| {p} | {prov[p]} |\n")
        f.write("\n## By status\n\n| status | count |\n|---|---|\n")
        for s in sorted(by_status, key=lambda x: -by_status[x]):
            f.write(f"| {s} | {by_status[s]} |\n")
        f.write(f"\nported (EXACT+BEHAVIOURAL+PARTIAL): **{ported}** · "
                f"genuine missing (NOT_YET_PORTED): **{by_status['NOT_YET_PORTED']}** · "
                f"blocked: **{by_status['BLOCKED_DEPENDENCY']}** · "
                f"replaced/out-of-scope/ui: **{by_status['REPLACED_BY_RUST']+by_status['OUT_OF_SCOPE']+by_status['UI_GDI_DOMAIN']}** · "
                f"non-useful: **{by_status['NON_USEFUL']}** · "
                f"foreign-breadth: **{by_status['FOREIGN_BREADTH']}** · "
                f"unknown: **{by_status['UNKNOWN']}**\n")
        f.write("\n## By subsystem\n\n| subsystem | total | ported | missing | out/ui/replaced | non-useful | unknown |\n|---|---|---|---|---|---|---|\n")
        for sub in sorted(by_sub, key=lambda s: -sum(by_sub[s].values())):
            c = by_sub[sub]
            f.write(f"| {sub} | {sum(c.values())} | {sum(c[s] for s in PORTED)} | {c['NOT_YET_PORTED']} | "
                    f"{c['OUT_OF_SCOPE']+c['UI_GDI_DOMAIN']+c['REPLACED_BY_RUST']} | {c['NON_USEFUL']} | {c['UNKNOWN']} |\n")

    # --- port order (call-graph driven backlog plan) ---
    nyp = [r for r in rows if r["status"] == "NOT_YET_PORTED"]
    reach = [r for r in nyp if r["cg_reach"]]
    unreach = [r for r in nyp if not r["cg_reach"]]
    dead_hint = [r for r in unreach if not r["callers"]]
    def _porder(r):
        return (r["cg_depth"] if r["cg_depth"] != "" else 9999, -r["callers"])
    sub_reach = defaultdict(lambda: [0, 0])
    for r in nyp:
        s = sub_reach[r["subsystem"] or "?"]; s[0] += 1
        if r["cg_reach"]: s[1] += 1
    with open(os.path.join(OUT, "gdi_port_order.md"), "w", encoding="utf-8") as f:
        f.write("# GDI port order (generated — call-graph driven)\n\n")
        f.write("Ranks the NOT_YET_PORTED backlog by **exe-reachability** from the live "
                "roots (per-day tick driver 0x005b6f10 + boot/new-game/message-pump) and "
                "depth from those roots. This is the order in which porting unblocks the "
                "running game.\n\n")
        f.write("> **Caveat.** The static call graph misses indirect/vtable/function-pointer "
                "calls, so `cg-unreachable` is a HINT, not proof of dead code. A 0-caller "
                "unreachable function is a candidate to review, not an automatic "
                "DEAD_OR_UNREACHABLE.\n\n")
        f.write(f"NOT_YET_PORTED: **{len(nyp)}** · reachable from live roots: **{len(reach)}** "
                f"· unreachable: **{len(unreach)}** · of those 0-caller (review as dead/indirect): "
                f"**{len(dead_hint)}**\n\n")
        f.write("## Reachable backlog by subsystem (reachable / total)\n\n"
                "| subsystem | reachable | total |\n|---|---|---|\n")
        for s, (tot, rc) in sorted(sub_reach.items(), key=lambda x: -x[1][1]):
            if rc:
                f.write(f"| {s} | {rc} | {tot} |\n")
        f.write("\n## Top 60 reachable targets (shallow depth, many callers first)\n\n"
                "| DD VA | depth | callers | subsystem | semantic |\n|---|---|---|---|---|\n")
        for r in sorted(reach, key=_porder)[:60]:
            f.write(f"| {r['dd_va']} | {r['cg_depth']} | {r['callers']} | {r['subsystem']} | "
                    f"{(r['semantic_name'] or r['symbol'])[:60]} |\n")

    # --- globals ---
    globs = load_csv("globals.csv")
    with open(os.path.join(OUT, "gdi_globals.md"), "w", encoding="utf-8") as f:
        f.write("# GDI global data symbols (generated from data/globals.csv)\n\n")
        f.write("| address | meaning | Rust equivalent | lifetime | used by | confidence |\n|---|---|---|---|---|---|\n")
        for g in globs:
            f.write(f"| {g.get('address','')} | {g.get('meaning','')} | {g.get('rust_equiv','')} | "
                    f"{g.get('lifetime','')} | {g.get('used_by','')} | {g.get('confidence','')} |\n")

    # --- human-readable registry (grouped by subsystem then status) ---
    with open(os.path.join(OUT, "gdi_function_registry.md"), "w", encoding="utf-8") as f:
        f.write("# GDI function registry (generated — do not hand-edit)\n\n")
        f.write("Regenerate: `python tools/gdi_registry/build_registry.py`. Curated rows in "
                "`tools/gdi_registry/data/*.csv`; see docs/reverse_engineering_conventions.md.\n\n")
        f.write(f"{len(rows)} functions registered. Coverage: docs/gdi_registry/gdi_function_coverage.md. "
                "Globals: docs/gdi_registry/gdi_globals.md.\n\n")
        cur = None
        for r in sorted(rows, key=lambda x: (x["subsystem"] or "~", x["source_file"], x["dd_va"])):
            sub = r["subsystem"] or "(unclassified)"
            if sub != cur:
                f.write(f"\n## {sub}\n\n| DD VA | source | semantic | status | Rust | reach | conf | evidence |\n|---|---|---|---|---|---|---|---|\n")
                cur = sub
            f.write(f"| {r['dd_va']} | {r['source_file']} | {r['semantic_name'] or r['symbol']} | "
                    f"{r['status']} | {r['rust_symbol'] or r['rust_file']} | {r['reachable']} | "
                    f"{r['confidence']} | {r['evidence']} |\n")
        sup_rows = [r for r in rows if r["superseded_prev"]]
        if sup_rows:
            f.write("\n## Superseded interpretations\n\n| DD VA | previous | why superseded | current |\n|---|---|---|---|\n")
            for r in sup_rows:
                f.write(f"| {r['dd_va']} | {r['superseded_prev']} | {r['superseded_why']} | {r['semantic_name']} |\n")

    print(f"registered {len(rows)} functions -> {OUT}")
    print("status:", dict(by_status))
    print(f"cited-in-rust: {sum(1 for r in rows if r['_cited'])}  ported: {ported}")

if __name__ == "__main__":
    main()
