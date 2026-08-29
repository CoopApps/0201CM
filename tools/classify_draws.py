"""Turn a directory of raw draw-command bursts (tools/frida_draw_capture.py)
into a NAMED LIBRARY of distinct screens — the basis for rendering every
page 100% from the game's own draw stream.

For each burst it:
  1. DEDUPES the double render pass (the game draws each screen 2+ times per
     burst; keep the last full pass).
  2. Extracts an identity: the largest-font title text (font 6/7 header) plus
     the second-largest (sub-title), e.g. "Italian Serie C Cup / Evening
     Fixtures".
  3. Groups bursts by (identity, geometry signature) so the same screen
     captured many times collapses to one entry, and different states
     (submenus open, different tabs) stay distinct.

Writes reports/screen_captures/library.json — {screen_key: canonical_burst}
— and prints the distinct-screen inventory. The canonical burst is the one
with the most draw calls (most complete state).

Usage: python tools/classify_draws.py
"""
import json
import glob
import os
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
DRAWS = REPO / "reports" / "screen_captures" / "draws"


def clean(h):
    if not h:
        return ""
    b = bytes.fromhex(h).split(b"\x00", 1)[0]
    return b.decode("latin-1", "replace")


def dedup(calls):
    if len(calls) < 4:
        return calls
    first = (calls[0]["fn"], tuple(calls[0]["args"]))
    starts = [i for i, c in enumerate(calls)
              if c["fn"] == first[0] and tuple(c["args"]) == first[1]]
    return calls[starts[-1]:] if len(starts) >= 2 else calls


def identity(calls):
    """Largest-font non-empty texts -> a human screen name."""
    titles = {}  # font -> first text at that font
    for c in calls:
        if c["fn"] != "glyph":
            continue
        t = clean(c.get("text")).strip()
        if not t or len(t) < 2:
            continue
        font = c["args"][2] & 0xffff
        if font > 7:
            continue
        titles.setdefault(font, t)
    # Structural title fonts only (7,6,5 = trade_cond headers / big arial).
    # Fonts 4,3 are list/content text (player names etc.) — dynamic, excluded
    # from identity so a screen collapses across its data states.
    for f in (7, 6, 5):
        if f in titles:
            head = titles[f]
            sub = ""
            for g in (6, 5):
                if g != f and g in titles and titles[g] != head:
                    sub = titles[g]
                    break
            return (head + (" / " + sub if sub else "")).strip()
    # fall back to any text
    for c in calls:
        if c["fn"] == "glyph":
            t = clean(c.get("text")).strip()
            if t:
                return t
    return "(untitled)"


def geom_sig(calls):
    # coarse signature: counts per primitive + rounded title position
    n = {}
    for c in calls:
        n[c["fn"]] = n.get(c["fn"], 0) + 1
    return tuple(sorted(n.items()))


def main():
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    files = sorted(glob.glob(str(DRAWS / "draws_*.json")))
    if not files:
        sys.exit("no draw captures found -- run tools/frida_draw_capture.py")

    library = {}   # key -> (burst_name, ncalls, identity)
    groups = {}    # identity -> set of burst names
    for f in files:
        d = json.load(open(f, encoding="utf-8"))
        calls = dedup(d["calls"])
        ident = identity(calls)
        key = ident + " | " + str(geom_sig(calls))
        name = os.path.basename(f).replace(".json", "")
        groups.setdefault(ident, set()).add(name)
        prev = library.get(key)
        if prev is None or len(calls) > prev[1]:
            library[key] = (name, len(calls), ident)

    # Emit library.json: identity -> best burst file
    lib_out = {}
    for key, (name, nc, ident) in library.items():
        lib_out.setdefault(ident, {"burst": name, "calls": nc, "states": []})
        if nc > lib_out[ident]["calls"]:
            lib_out[ident] = {"burst": name, "calls": nc, "states": []}
    (DRAWS.parent / "library.json").write_text(
        json.dumps(lib_out, indent=1), encoding="utf-8")

    print(f"{len(files)} bursts -> {len(groups)} distinct screens\n")
    for ident, names in sorted(groups.items(), key=lambda kv: -len(kv[1])):
        best = lib_out[ident]
        print(f"  x{len(names):2}  {ident[:52]:52}  best={best['burst']} ({best['calls']} calls)")
    print(f"\nlibrary.json written: {len(groups)} screens keyed by title.")


if __name__ == "__main__":
    main()
