"""Turn multiple captures of the SAME screen (different game data) into a
bindable TEMPLATE — the Phase-3 step that lets a captured screen show LIVE
simulation data instead of the frozen snapshot.

A screen's draw stream splits into:
  * STATIC draws — everything identical across captures (chrome: fills, lines,
    images, and label glyphs like tab names). Replayed verbatim.
  * DYNAMIC slots — glyph positions whose TEXT differs between captures (player
    names, scores, ratings, dates). Same position/font/color every time; only
    the string changes. These are the binding points: at render time their text
    comes from the Rust sim, not the capture.

Identifying dynamic slots needs captures with DIFFERENT data (e.g. two players'
profiles, two clubs' squads). With only same-data captures every slot looks
static — so the capture walk should visit each screen with varied data.

Writes reports/screen_captures/templates/<slug>.json:
  { name, static_calls:[...], dynamic_slots:[{x,y,font,color,samples:[...]}] }

Usage: python tools/extract_template.py <burst_a> <burst_b> [<burst_c> ...] --name <slug>
"""
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
DRAWS = REPO / "reports" / "screen_captures" / "draws"
OUT = REPO / "reports" / "screen_captures" / "templates"


def dedup(calls):
    if len(calls) < 4:
        return calls
    first = (calls[0]["fn"], tuple(calls[0]["args"]))
    starts = [i for i, c in enumerate(calls)
              if c["fn"] == first[0] and tuple(c["args"]) == first[1]]
    return calls[starts[-1]:] if len(starts) >= 2 else calls


def txt(h):
    if not h:
        return ""
    return bytes.fromhex(h).split(b"\x00", 1)[0].decode("latin-1", "replace")


def load(name):
    d = json.load(open(DRAWS / f"{name}.json", encoding="utf-8"))
    return dedup(d["calls"])


def main():
    argv = sys.argv[1:]
    slug = "template"
    if "--name" in argv:
        i = argv.index("--name")
        slug = argv[i + 1]
        del argv[i:i + 2]
    args = [a for a in argv if not a.startswith("--")]
    if len(args) < 2:
        sys.exit("need >=2 burst names (same screen, different data)")

    caps = [load(n) for n in args]
    base = caps[0]

    # Text at each glyph position, per capture.
    def glyph_map(calls):
        m = {}
        for c in calls:
            if c["fn"] == "glyph":
                a = c["args"]
                m[(a[0], a[1])] = (a[2] & 0xffff, a[3] & 0xffff, txt(c.get("text")))
        return m

    maps = [glyph_map(c) for c in caps]

    # --- fixed-position dynamic slots: same (x,y), text varies ---------------
    common = set(maps[0])
    for m in maps[1:]:
        common &= set(m)
    dynamic_pos = {p for p in common if len({m[p][2] for m in maps}) > 1}

    # --- REGION (centered/variable-length) dynamic slots --------------------
    # A glyph that only appears at ONE (x,y) in the base but matches, in the
    # other captures, a glyph at the SAME row y (±2) and SAME font with a
    # DIFFERENT x and DIFFERENT text, is centered/aligned dynamic text (a
    # header name etc.). Bind it as a REGION (row + font) with an inferred
    # alignment, so the renderer places the live value itself.
    def approx_width(text, font):
        # rough per-font average glyph advance (px); only used to infer align.
        avg = {0: 5, 1: 5, 2: 6, 3: 8, 4: 9, 5: 10, 6: 15, 7: 17}
        return len(text) * avg.get(font, 8)

    base_glyphs = [c for c in base if c["fn"] == "glyph"]
    region_slots = {}   # base (x,y) -> slot dict
    for c in base_glyphs:
        a = c["args"]
        bx, by, bfont = a[0], a[1], a[2] & 0xffff
        btext = txt(c.get("text"))
        if (bx, by) in dynamic_pos or not btext.strip():
            continue
        # find, in each other capture, a same-row same-font DIFFERENT-x glyph
        matches = []
        for m in maps[1:]:
            cand = [(x, y, f, col, t) for (x, y), (f, col, t) in m.items()
                    if abs(y - by) <= 2 and f == bfont and x != bx and t != btext]
            # nearest by x
            if cand:
                cand.sort(key=lambda e: abs(e[0] - bx))
                matches.append(cand[0])
        if len(matches) == len(maps) - 1 and matches:
            # infer alignment from base + matches
            xs = [bx] + [mm[0] for mm in matches]
            texts = [btext] + [mm[4] for mm in matches]
            centers = [x + approx_width(t, bfont) / 2 for x, t in zip(xs, texts)]
            rights = [x + approx_width(t, bfont) for x, t in zip(xs, texts)]
            if max(centers) - min(centers) <= 8:
                align, anchor = "center", int(sum(centers) / len(centers))
            elif max(rights) - min(rights) <= 8:
                align, anchor = "right", int(sum(rights) / len(rights))
            else:
                align, anchor = "left", bx
            region_slots[(bx, by)] = {
                "y": by, "font": bfont, "color": a[3] & 0xffff,
                "align": align, "anchor": anchor,
                "samples": texts, "field": None,
            }

    static_calls = []
    dynamic_slots = []
    region_out = []
    seen_dyn = set()
    for c in base:
        if c["fn"] == "glyph":
            a = c["args"]
            pos = (a[0], a[1])
            if pos in dynamic_pos:
                if pos not in seen_dyn:
                    seen_dyn.add(pos)
                    dynamic_slots.append({
                        "x": a[0], "y": a[1],
                        "font": a[2] & 0xffff, "color": a[3] & 0xffff,
                        "samples": [m[pos][2] for m in maps if pos in m],
                        "field": None,
                    })
                continue
            if pos in region_slots:
                region_out.append(region_slots[pos])
                continue  # region-dynamic glyph is not static
        static_calls.append(c)

    OUT.mkdir(parents=True, exist_ok=True)
    tmpl = {"name": slug,
            "sources": args,
            "static_calls": static_calls,
            "dynamic_slots": dynamic_slots,
            "region_slots": region_out}
    (OUT / f"{slug}.json").write_text(json.dumps(tmpl), encoding="utf-8")

    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    print(f"template '{slug}': {len(static_calls)} static, "
          f"{len(dynamic_slots)} fixed slots, {len(region_out)} region slots")
    for s in sorted(dynamic_slots, key=lambda s: (s["y"], s["x"])):
        print(f"  fixed  ({s['x']:3},{s['y']:3}) font{s['font']} samples={s['samples']}")
    for s in sorted(region_out, key=lambda s: s["y"]):
        print(f"  region y={s['y']:3} font{s['font']} {s['align']}@{s['anchor']} samples={s['samples']}")
    print(f"\n-> {(OUT / (slug + '.json')).relative_to(REPO)}")


if __name__ == "__main__":
    main()
