#!/usr/bin/env python3
"""Inject `// GDI-REG: <addr> <status>` provenance tags at the Rust items named by
curated registry rows, so the exe<->Rust link lives in the code (greppable,
drift-checkable) as well as in the registry CSV.

Conservative + idempotent: only tags an item when its identifier has EXACTLY ONE
definition site in the cited file, and only if that item is not already tagged.
Comment-only insertions (cannot change behaviour). Reports injected/skipped.

Run: python tools/gdi_registry/inject_tags.py   (then rebuild + validate)
"""
import csv, os, re

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
CURATED = os.path.join(os.path.dirname(__file__), "data", "curated.csv")
PORTED = {"PORTED_EXACT", "PORTED_BEHAVIOURAL", "PORTED_PARTIAL", "REPLACED_BY_RUST"}

def def_regex(name):
    n = re.escape(name)
    # fn / const / static / struct / enum / trait / type definitions of `name`
    return re.compile(
        rf'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+|unsafe\s+|const\s+)*'
        rf'(?:fn\s+{n}\b|const\s+{n}\b|static\s+{n}\b|struct\s+{n}\b|'
        rf'enum\s+{n}\b|trait\s+{n}\b|type\s+{n}\b)'
    )

def main():
    rows = list(csv.DictReader(open(CURATED, encoding="utf-8")))
    injected = skipped_multi = skipped_none = already = 0
    edits = {}  # file -> list of (lineno_0based, text)
    for r in rows:
        status = (r.get("status") or "").strip()
        if status not in PORTED:
            continue
        addr = (r.get("dd_va") or "").strip().lower().replace("0x", "")
        if not addr:
            continue
        addr8 = addr.zfill(8)
        files = [f.strip() for f in (r.get("rust_file") or "").split(";") if f.strip()]
        syms = [s.strip() for s in (r.get("rust_symbol") or "").split(";") if s.strip()]
        for sym in syms:
            ident = sym.split("::")[-1].split("(")[0].strip()
            if not ident or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", ident):
                continue
            rx = def_regex(ident)
            hits = []  # (file, lineno)
            for rf in files:
                fp = os.path.join(ROOT, rf)
                if not os.path.isfile(fp):
                    continue
                lines = open(fp, encoding="utf-8", errors="replace").read().splitlines()
                for i, ln in enumerate(lines):
                    if rx.match(ln):
                        hits.append((rf, i, lines))
            if len(hits) == 0:
                skipped_none += 1; continue
            if len(hits) > 1:
                skipped_multi += 1; continue
            rf, i, lines = hits[0]
            # idempotent: already tagged for this addr in the few lines above?
            above = "\n".join(lines[max(0, i-3):i])
            if f"GDI-REG: {addr8}" in above:
                already += 1; continue
            indent = re.match(r'\s*', lines[i]).group(0)
            edits.setdefault(rf, []).append((i, f"{indent}// GDI-REG: {addr8} {status}"))
            injected += 1

    for rf, ins in edits.items():
        fp = os.path.join(ROOT, rf)
        lines = open(fp, encoding="utf-8", errors="replace").read().splitlines()
        for lineno, text in sorted(ins, key=lambda x: -x[0]):  # bottom-up
            lines.insert(lineno, text)
        open(fp, "w", encoding="utf-8").write("\n".join(lines) + "\n")

    print(f"injected {injected} tags into {len(edits)} files")
    print(f"skipped: {skipped_none} identifier-not-found, {skipped_multi} ambiguous-multi-def, {already} already-tagged")

if __name__ == "__main__":
    main()
