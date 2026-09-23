#!/usr/bin/env python3
"""Integrity checks for the GDI function registry.

  python tools/gdi_registry/validate.py

Exit non-zero if any hard error is found. Warnings are printed but non-fatal.
"""
import csv, json, os, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
REG_JSON = os.path.join(ROOT, "docs", "gdi_registry", "gdi_function_registry.json")

STATUSES = {"PORTED_EXACT","PORTED_BEHAVIOURAL","PORTED_PARTIAL","REPLACED_BY_RUST",
    "NOT_YET_PORTED","BLOCKED_DEPENDENCY","FOREIGN_BREADTH","UI_GDI_DOMAIN",
    "OUT_OF_SCOPE","NON_USEFUL","DEAD_OR_UNREACHABLE","UNKNOWN"}
PORTED = {"PORTED_EXACT","PORTED_BEHAVIOURAL","PORTED_PARTIAL"}
CONF = {"BYTE_EXACT","STATE_EXACT","BEHAVIOURALLY_EXACT","STRUCTURALLY_VERIFIED",
        "PARTIAL","HYPOTHESIS","UNVERIFIED","APPROXIMATE"}
REACH = {"YES","NO","TEST_ONLY","INDIRECT","BLOCKED"}

def main():
    if not os.path.exists(REG_JSON):
        sys.exit(f"registry not built: {REG_JSON}")
    rows = json.load(open(REG_JSON, encoding="utf-8"))
    errors, warns = [], []
    seen = {}
    for r in rows:
        va = r["dd_va"]
        if va in seen:
            errors.append(f"duplicate DD VA {va}")
        seen[va] = r
        st = r["status"]
        if st not in STATUSES:
            errors.append(f"{va}: invalid status {st!r}")
        if not st:
            errors.append(f"{va}: missing status")
        if r["confidence"] and r["confidence"] not in CONF:
            warns.append(f"{va}: unusual confidence {r['confidence']!r}")
        if r["reachable"] and r["reachable"] not in REACH:
            warns.append(f"{va}: unusual reachable {r['reachable']!r}")
        if st in PORTED and not (r["rust_file"] or r["rust_symbol"]):
            errors.append(f"{va}: {st} but no Rust file/symbol")
        if st in PORTED and not r["evidence"] and st != "PORTED_BEHAVIOURAL":
            warns.append(f"{va}: {st} without evidence")
        if st == "NOT_YET_PORTED" and r["reachable"] == "YES":
            errors.append(f"{va}: NOT_YET_PORTED but reachable=YES (contradiction)")
        if st == "NON_USEFUL" and r["rust_symbol"]:
            warns.append(f"{va}: NON_USEFUL yet has a Rust symbol ({r['rust_symbol']}) — justify or reclassify")
        # cited Rust files should exist
        for rf in (r["rust_file"] or "").split(";"):
            rf = rf.strip()
            if rf and rf.endswith(".rs") and not os.path.exists(os.path.join(ROOT, rf)):
                warns.append(f"{va}: rust_file not found: {rf}")

    # --- rust_symbol existence: catch drift when a cited symbol is renamed ---
    import re
    def last_ident(sym):
        return sym.split("::")[-1].split("(")[0].strip()
    for r in rows:
        syms = [s.strip() for s in (r["rust_symbol"] or "").split(";") if s.strip()]
        files = [f.strip() for f in (r["rust_file"] or "").split(";") if f.strip()]
        if not syms or not files:
            continue
        blobs = {}
        for rf in files:
            fp = os.path.join(ROOT, rf)
            if os.path.isfile(fp):
                blobs[rf] = open(fp, encoding="utf-8", errors="replace").read()
        for sym in syms:
            ident = last_ident(sym)
            if ident and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", ident):
                if not any(re.search(rf"\b{re.escape(ident)}\b", b) for b in blobs.values()):
                    warns.append(f"{r['dd_va']}: rust_symbol '{ident}' not found in cited file(s) — drift?")

    # --- // GDI-REG: <addr> <status> tags in code must match the registry ---
    by_va = {r["dd_va"]: r for r in rows}
    tag_re = re.compile(r'GDI-REG:\s*([0-9a-fA-F]{6,8})\s+([A-Z_]+)')
    crates = os.path.join(ROOT, "crates")
    for dp, _, fs in os.walk(crates):
        for fn in fs:
            if not fn.endswith(".rs"):
                continue
            for ln in open(os.path.join(dp, fn), encoding="utf-8", errors="replace"):
                m = tag_re.search(ln)
                if not m:
                    continue
                va = "0x%08x" % int(m.group(1), 16)
                tag_status = m.group(2)
                if va not in by_va:
                    errors.append(f"GDI-REG tag {va} in {fn}: not in registry")
                elif by_va[va]["status"] != tag_status:
                    warns.append(f"GDI-REG tag {va} in {fn}: status {tag_status} != registry {by_va[va]['status']} (merge or drift)")

    for w in warns: print(f"WARN {w}")
    for e in errors: print(f"ERR  {e}")
    print(f"\n{len(rows)} rows · {len(errors)} errors · {len(warns)} warnings")
    sys.exit(1 if errors else 0)

if __name__ == "__main__":
    main()
