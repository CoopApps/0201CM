"""Extract the COMPLETE data-model schema from the CM0102 editor's Delphi form
resource (TFRM_PANELS) — the authoritative field list for every record type.

The editor (cm0102ed.exe) is a Delphi app; its TFRM_PANELS form binds a labelled
control to every editable field of every record, with a canonical id of the form
`<record>_<field>` (e.g. staff_pl_passing, stadium_capacity, club_reputation).
The controls appear in record-field order, so this gives not just the field names
but their ORDER — which lines up with the byte layout confirmed by the raw-.dat
offset scan (players: CA/PA/reputations, then 12 position ratings, then the
attribute block at type10 0x0f..0x44).

Output: reports/editor_schema.json
  { record_type: [field_id, ...ordered...], ... }

This is the reference every screen slot binds against, and the map that names
each raw attribute byte. Nothing hardcoded; the schema is the editor's own.

Usage: python tools/extract_editor_schema.py
"""
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
FORM = REPO / "assets/cm0102/pe_resources/Editor_cm0102ed/rcdata/Editor_cm0102ed_rcdata_TFRM_PANELS_lang0.bin"

# Record prefixes that name real data tables (skip Delphi UI control prefixes
# like button/panel/label/tabsheet/click/edit/sb/pc).
DATA_PREFIXES = {
    "club", "nation", "stadium", "colour", "continent", "city", "official",
    "weather", "staff_pl", "staff_np", "staff_config", "staff_competition",
    "staff_club", "staff_nation", "staff_favourite", "staff_disliked",
    "staff_second", "index", "name", "club_competition", "nation_reputation",
}


def prefix_of(fid: str) -> str:
    parts = fid.split("_")
    two = "_".join(parts[:2])
    if two in ("staff_pl", "staff_np", "staff_config", "staff_competition",
               "staff_club", "staff_nation", "staff_favourite",
               "staff_disliked", "staff_second", "club_competition"):
        return two
    return parts[0]


def main():
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if not FORM.exists():
        sys.exit(f"editor form not found: {FORM}")
    data = FORM.read_bytes()
    runs = [r.decode("latin-1") for r in re.findall(rb"[\x20-\x7e]{3,}", data)]
    ids = [s for s in runs
           if re.fullmatch(r"[a-z][a-z0-9_]{3,50}", s) and "_" in s]

    schema = {}
    order = {}
    for fid in ids:
        pre = prefix_of(fid)
        if pre not in DATA_PREFIXES:
            continue
        schema.setdefault(pre, [])
        if fid not in schema[pre]:
            schema[pre].append(fid)

    out = REPO / "reports" / "editor_schema.json"
    out.write_text(json.dumps(schema, indent=1), encoding="utf-8")

    total = sum(len(v) for v in schema.values())
    print(f"{len(schema)} record types, {total} fields -> {out.relative_to(REPO)}\n")
    for rec, fields in sorted(schema.items(), key=lambda kv: -len(kv[1])):
        print(f"  {rec:20} {len(fields):3} fields")


if __name__ == "__main__":
    main()
