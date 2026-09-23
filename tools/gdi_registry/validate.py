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

    for w in warns: print(f"WARN {w}")
    for e in errors: print(f"ERR  {e}")
    print(f"\n{len(rows)} rows · {len(errors)} errors · {len(warns)} warnings")
    sys.exit(1 if errors else 0)

if __name__ == "__main__":
    main()
