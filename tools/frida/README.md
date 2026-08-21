# CM0102 Frida harness — the ground-truth backbone

Runs the real game and captures what it *actually* does, so the Rust reimplementation
can be **verified against reality** instead of guessed. This is the oracle in the
Ghidra (static) → Frida (dynamic) → carver (ledger) → Rust (differential-tested) loop.

## Parts
- `hooks.js` — Frida hooks by category (absolute addrs, base 0x400000):
  - `draw` — panels/text/colors/images (`0x005cf8e0`/`0x005d0870`/`0x005ce4f0`/`0x005cddc0`/`0x005cdcc0`) → GUI draw stream
  - `font` — Win32 `CreateFontA` → the **traditional** TrueType fonts (face/size/weight)
  - `view` — view-model slot setter `0x007e7130` → which data binds to each screen
  - `match` — `0x006a4020` (final score, matchstate `+0xf5bc/+0xf5f2`, teams `+0x1d6/+0x1d8`), `0x0069d950` setup → **teams → scoreline** ground truth
  - `rng` — `0x008fc4f0` match RNG → verify `cm-rng` is bit-exact
- `capture.py` — spawn/attach, install categories, record to `capture.jsonl`
- `replay.py` — replay a draw capture → pixel-exact PNG (honors alpha `0x2`, composites the
  background image, renders **traditional** Arial TrueType); `--ref` diffs vs a screenshot

## Use
```
# GUI: capture a screen, replay it, diff against a reference screenshot
python capture.py --secs 60 --cats draw,font,view    # navigate the game to the target screen
python replay.py --ref path/to/screenshot.png

# Match engine ground truth: capture real matches (teams -> score)
python capture.py --secs 120 --cats match,rng
```

## Notes
- The game needs ~30s+ to load its DB before any screen renders — use `--secs >= 55`.
- **Traditional vs futuristic fonts:** the `.fnt` files are the *futuristic* bitmaps; the
  *traditional* fonts are Windows TrueType via `CreateFontA`. To capture the traditional set,
  set Game Settings → Fonts → Traditional in-game during a `font` capture. We render everything
  traditional (smooth Arial).
- No ASLR on the 2001 build, so absolute addresses work directly.

## The loop this enables
Ghidra says *where/how* → Frida says *what really happens* → carver *records + prioritises* →
Rust *implements and differential-tests* against the capture. Turns "is my port right?" from a
guess into a diff.
