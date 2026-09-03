#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Analyse fixtures/news_capture.jsonl.gz — the multi-source correlation
input for porting sub_00770170 (News screen builder).

Usage:
    python3 tools/gdi_capture/analyse_news_capture.py \
        [path/to/news_capture.jsonl.gz]

What it does:
  1. Gunzips + reads the JSONL
  2. Groups events by present-boundary (op == "--- PRESENT ---")
  3. Identifies the "News-open frame" as the modal set of primitives
     (nearly every frame in this capture is identical News-open state).
  4. Prints per-present summary: event counts, unique text strings,
     panel/wrapped rect ranges.
  5. Extracts widget-spec candidates: (rect, style, colour, label) tuples
     for panels; (rect, style, colour, text) for wrapped_text; sorted.

Known caveats:
  * `readCString(256)` bug in live_log.js — old captures have "junk?????"
    tails on some text args. This script slices at the first '?' too, so
    the extracted text is only reliable up to that point.
  * Fixture has no framebuffer snapshots — a byte-exact pixel diff test
    cannot be built from the JSONL alone. Downstream code compares
    primitive sequences instead.

Cite: fixtures/news_capture_analysis.md and reports/cm0102_exact_news_ui_evidence.md.
"""

from __future__ import annotations
import collections
import gzip
import json
import sys
from pathlib import Path


DEFAULT_PATH = Path("fixtures/news_capture.jsonl.gz")


def clean_text(t: str) -> str:
    """Truncate readCString junk. Real exe text is always at the front,
    before the first '?' filler (which comes from Frida reading past a
    non-NUL scratch buffer). Preserves newlines/tabs."""
    if not t:
        return ""
    idx = t.find("?")
    return t[:idx] if idx >= 0 else t


def load(path: Path) -> list[dict]:
    """Read the JSONL, one event per line."""
    with gzip.open(path, "rt", encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def group_by_present(events: list[dict]) -> list[list[dict]]:
    """Split at every '--- PRESENT ---' event. Returns a list of frames;
    each frame is the ordered draw calls BEFORE the present marker."""
    frames: list[list[dict]] = []
    cur: list[dict] = []
    for e in events:
        if e.get("op") == "--- PRESENT ---":
            frames.append(cur)
            cur = []
        else:
            cur.append(e)
    if cur:
        frames.append(cur)  # trailing partial (no present at end)
    return frames


def frame_fingerprint(frame: list[dict]) -> tuple:
    """A hashable summary of a frame's structure, for identifying
    duplicates."""
    return (
        len(frame),
        tuple(e.get("op", "?") for e in frame[:20]),
    )


def summarise_frame(frame: list[dict], label: str) -> None:
    op_counts = collections.Counter(e.get("op") for e in frame)
    texts = sorted({clean_text(e.get("text", "")) for e in frame if e.get("text")})
    texts = [t for t in texts if t]
    panels = [(e["x0"], e["y0"], e["x1"], e["y1"], e.get("style", ""), e.get("colour", 0))
              for e in frame if e.get("op") == "panel"]
    print(f"\n=== {label} ({len(frame)} events) ===")
    print(f"op counts: {dict(op_counts)}")
    print(f"unique text strings ({len(texts)}):")
    for t in texts[:40]:
        s = t.replace("\n", "\\n")
        print(f"   \"{s[:80]}\"")
    if len(texts) > 40:
        print(f"   ... {len(texts) - 40} more")
    if panels:
        xs = sorted({p[0] for p in panels} | {p[2] for p in panels})
        ys = sorted({p[1] for p in panels} | {p[3] for p in panels})
        print(f"panel x-boundaries: {xs[:12]}...{xs[-4:] if len(xs)>16 else ''}")
        print(f"panel y-boundaries: {ys[:12]}...{ys[-4:] if len(ys)>16 else ''}")


def widget_candidates(frame: list[dict]) -> list[dict]:
    """Extract the ordered (rect, style, colour, label) primitives that
    would be output by a spawn_widget call — i.e. panels + wrapped_text
    calls (widget rects) and label text (label content)."""
    cands = []
    for e in frame:
        op = e.get("op")
        if op == "panel":
            cands.append({
                "op": "panel",
                "rect": (e["x0"], e["y0"], e["x1"], e["y1"]),
                "style": e.get("style"),
                "colour": e.get("colour"),
                "from": e.get("from"),
            })
        elif op == "wrapped":
            cands.append({
                "op": "wrapped",
                "rect": (e["x0"], e["y0"], e["x1"], e["y1"]),
                "style": e.get("style"),
                "colour": e.get("colour"),
                "text": clean_text(e.get("text", "")),
                "from": e.get("from"),
            })
        elif op == "glyph":
            cands.append({
                "op": "glyph",
                "at": (e["x"], e["y"]),
                "colour": e.get("colour"),
                "text": clean_text(e.get("text", "")),
                "from": e.get("from"),
            })
    return cands


# Subtract shared chrome. Sidebar area = (5, 10) .. (85, 366) per
# fixtures/screen_sidebar_and_manager_menu.md (paraphrased from
# reports/cm0102_exact_news_ui_evidence.md main_menu_chrome fact:
# x=0, y=0, x2=0x59 (=89), y2=599). Also the top menu bar occupies
# roughly y=0..y=8 across full width.
SIDEBAR_RECT = (0, 0, 89, 599)   # widened per menubar decode


def in_sidebar(rect: tuple[int, int, int, int]) -> bool:
    x0, y0, x1, y1 = rect
    sx0, sy0, sx1, sy1 = SIDEBAR_RECT
    return x0 >= sx0 and y0 >= sy0 and x1 <= sx1 and y1 <= sy1


def report_news_specific(cands: list[dict]) -> None:
    news = [c for c in cands if c["op"] != "glyph" and not in_sidebar(c["rect"])]
    glyphs = [c for c in cands if c["op"] == "glyph" and c["at"][0] > 89]
    print(f"\n=== News-specific widget candidates ({len(news)}) ===")
    # Group by outer x0 to spot the tab-strip columns.
    by_rect = collections.Counter(c["rect"] for c in news)
    for rect, count in sorted(by_rect.items())[:60]:
        matching = [c for c in news if c["rect"] == rect][:1]
        m = matching[0]
        text = m.get("text", "")
        if isinstance(text, str):
            text = text.replace("\n", "\\n")[:40]
        print(f"   {rect} × {count} — {m['op']:8s} style={m.get('style')} col={m.get('colour')} text=\"{text}\" from={m.get('from')}")
    if len(by_rect) > 60:
        print(f"   ... {len(by_rect) - 60} more unique rects")

    # Sorted unique text (from wrapped + glyph) outside sidebar.
    texts = sorted({c.get("text", "") for c in cands
                    if c.get("text") and c["op"] != "panel"
                    and (c["op"] == "wrapped" and not in_sidebar(c["rect"])
                         or c["op"] == "glyph" and c["at"][0] > 89)})
    texts = [t for t in texts if t]
    print(f"\n=== News-specific text strings ({len(texts)}) ===")
    for t in texts[:80]:
        print(f"   \"{t.replace(chr(10), '\\n')[:100]}\"")


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_PATH
    if not path.exists():
        print(f"error: {path} not found", file=sys.stderr)
        return 1
    events = load(path)
    print(f"loaded {len(events)} events from {path}")
    frames = group_by_present(events)
    print(f"{len(frames)} present-frames")

    # Fingerprint frames to find the modal News-open frame.
    fps = collections.Counter(frame_fingerprint(f) for f in frames)
    for fp, n in fps.most_common(5):
        print(f"  {n} frames × op-count={fp[0]} first-ops={fp[1][:6]}")

    # Pick frame 1 (frame 0 may be partial startup residue).
    news_frame = frames[1] if len(frames) > 1 else frames[0]
    summarise_frame(news_frame, "News-open frame (index 1)")

    cands = widget_candidates(news_frame)
    report_news_specific(cands)

    # Dump news-specific panel rects for correlation with asm spawn_area.
    news_panels = sorted({c["rect"]: c["from"] for c in cands
                          if c["op"] == "panel" and not in_sidebar(c["rect"])}
                         .items())
    print(f"\n=== Distinct News-specific panel rects ({len(news_panels)}) ===")
    for rect, from_addr in news_panels:
        print(f"   rect={rect} from={from_addr}")

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
