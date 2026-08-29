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

    # Per identity, collect every burst with its deduped call count.
    groups = {}    # identity -> list of (name, ncalls)
    for f in files:
        d = json.load(open(f, encoding="utf-8"))
        calls = dedup(d["calls"])
        ident = identity(calls)
        name = os.path.basename(f).replace(".json", "")
        groups.setdefault(ident, []).append((name, len(calls)))

    # Canonical burst = the MODE of the deduped call count (the stable state
    # the screen was captured in repeatedly), not the max (which favours
    # transitional captures — a screen caught mid menu-open has more calls and
    # renders as an overlap/ghost). Ties break to the larger count.
    lib_out = {}
    for ident, entries in groups.items():
        from collections import Counter
        counts = Counter(nc for _, nc in entries)
        # most common count; tie -> larger
        best_count = sorted(counts.items(), key=lambda kv: (kv[1], kv[0]))[-1][0]
        rep = next(n for n, nc in entries if nc == best_count)
        lib_out[ident] = {"burst": rep, "calls": best_count,
                          "captured": len(entries)}
    (DRAWS.parent / "library.json").write_text(
        json.dumps(lib_out, indent=1), encoding="utf-8")

    print(f"{len(files)} bursts -> {len(groups)} distinct screens\n")
    for ident, entries in sorted(groups.items(), key=lambda kv: -len(kv[1])):
        best = lib_out[ident]
        print(f"  x{len(entries):2}  {ident[:52]:52}  canonical={best['burst']} ({best['calls']} calls)")
    print(f"\nlibrary.json written: {len(groups)} screens keyed by title.")


if __name__ == "__main__":
    main()
