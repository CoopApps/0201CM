"""Import D:/cm0102/History/*.his → rust-db/references/written_history.json.

Mirrors what crates/cm-import/src/bin/regen_written_history.rs would do,
run in Python because the monster cm-import crate is currently exhausting
rustc's heap on this host.

Header format (CM01/02): one `{KEY: VALUE}` per line, then a blank line,
then the tab-indented narrative body. `.hsr` bitmap sidecars are not
touched.
"""
from pathlib import Path
import json, sys, os

SRC = Path(os.environ.get("CM_HISTORY_DIR", "D:/cm0102/History"))
DB  = Path(os.environ.get("CM_RUST_DB", "D:/cm0102-rs/rust-db"))
OUT = DB / "references" / "written_history.json"

CANON = {
    "club": "Clubs", "clubs": "Clubs", "0": "Clubs",
    "nation": "Nations", "nations": "Nations", "1": "Nations",
    "competition": "Competitions", "competitions": "Competitions", "comp": "Competitions", "2": "Competitions",
    "league": "Leagues", "leagues": "Leagues", "3": "Leagues",
    "player": "Players", "players": "Players", "4": "Players",
}

def parse_his(text: str):
    rec = {"category": "", "category_raw": "", "title": "", "section": "", "pic": "", "body": ""}
    if not text.lstrip().startswith("{"):
        return rec
    cursor = 0
    lines = text.split("\n")
    for idx, raw in enumerate(lines):
        # Track byte cursor so we can slice `text` from the body's start.
        this_start = cursor
        cursor += len(raw) + 1  # +1 for the '\n' we split on
        l = raw.strip("\r").strip()
        if not l:
            continue
        if l.startswith("{") and l.endswith("}"):
            inner = l[1:-1]
            if ":" not in inner:
                continue
            k, v = inner.split(":", 1)
            key = k.strip().lower()
            val = v.strip()
            if key == "category":
                rec["category_raw"] = val
                rec["category"] = CANON.get(val.strip().lower(), "")
            elif key == "title":   rec["title"] = val
            elif key == "section": rec["section"] = val
            elif key == "pic":     rec["pic"] = val
            continue
        # First non-header, non-blank line ⇒ narrative body.
        rec["body"] = text[this_start:].rstrip("\x00").rstrip()
        break
    return rec

def main():
    if not SRC.is_dir():
        print(f"[regen_written_history] {SRC} not found", file=sys.stderr); sys.exit(1)
    files = sorted(p for p in SRC.iterdir() if p.suffix.lower() == ".his")
    print(f"[regen_written_history] src: {SRC}")
    print(f"[regen_written_history] out: {OUT}")
    print(f"[regen_written_history] found {len(files)} .his files")
    out = []
    counts = {}
    spot_billy = None
    for p in files:
        # Latin-1 for accented player names.
        text = p.read_bytes().decode("latin-1")
        rec = parse_his(text)
        rec["filename"] = p.name
        key = rec["category"] or rec["category_raw"] or "?"
        counts[key] = counts.get(key, 0) + 1
        out.append(rec)
        if p.name == "Billy Bingham.his":
            spot_billy = rec
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(out, ensure_ascii=False), encoding="utf-8")
    print(f"[regen_written_history] wrote {len(out)} records ({OUT.stat().st_size} bytes)")
    print("\nCategory distribution:")
    for k, v in sorted(counts.items()): print(f"  {k:>16}  {v}")
    if spot_billy:
        preview = spot_billy["body"][:140]
        print(f"\n=== SPOT CHECK: Billy Bingham.his ===")
        print(f"  category:     {spot_billy['category']!r}")
        print(f"  category_raw: {spot_billy['category_raw']!r}")
        print(f"  title:        {spot_billy['title']!r}")
        print(f"  section:      {spot_billy['section']!r}")
        print(f"  pic:          {spot_billy['pic']!r}")
        print(f"  body[:140]:   {preview!r}")

if __name__ == "__main__":
    main()
