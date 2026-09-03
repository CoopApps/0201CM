# News screen capture — evidence bundle

Captured live from a running `cm0102_GDI.exe` on 2026-09-03 using
`tools/gdi_capture/live_log.py` with Frida hooking the 8 primitive
functions at `0x005cd3e0` (line), `0x005cd730` (rect), `0x005cdd60`
(darken), `0x005cda90` (restore), `0x005ceaa0` (glyph), `0x005cf570`
(panel), `0x005d03a0` (wrapped_text), `0x005cccd0` (present).

The user drove the game manually:
1. Started on the club dashboard with sidebar visible
2. Clicked a non-News sidebar button (transitioned away from News)
3. Opened the manager menu (clicked the manager-name button)
4. Clicked "News" — the transition into News is what got captured

## What's in each file

### `news_capture.jsonl.gz` — the primary capture

- **65,701 events** across **60 present-boundaries** (60 discrete frame flips)
- **12,978,677 bytes** raw → 629,847 bytes gzipped
- Contains: 55,080 line + 3,880 panel + 3,540 wrapped + 3,179 glyph +
  1,731 rect + 600 darken + 120 restore + 60 present

### `click_test.jsonl.gz` — a broader multi-screen click sequence

- **75,501 events** across **56 present-boundaries**
- **15,364,646 bytes** raw → 703,279 bytes gzipped
- Broader coverage of what a normal click-through-the-game looks like

## Text strings recovered from the News capture

**News-specific chrome:**
- `"Christoph Olewicz News"` — the News page header
- `"Filter :"` — filter label
- `"Next"`, `"Back"` — nav-bar buttons (from FUN_005d75b0, already
  ported as `screen_nav_back_next.rs`)

**All 8 tab-strip categories** (from `sub_00770170:0x00770539..00770698`):
- `All`
- `Messages`
- `Competitions`
- `Injuries and Bans`
- `Contracts and Media`
- `Transfers`
- `Jobs`
- `Records`

**Actual news items rendered:**
- `Davide Andorno has begun full training following his thigh injury.`
- `Pro Vercelli march on`
- `Genoa march on`
- `Board reaction to Pro Sesto game`
- `World Cup 2002 qualifiers`
- `Carrarese v\nPro Vercelli`

**Shared chrome also present (sidebar):**
- `Wednesday\n10.10.01 EVE` (date)
- `Continue\nGame`
- `Christoph\nOlewicz` (active manager)
- `Nations\n& Clubs`, `Competitions`, `Transfers`, `Find`, `Game
  Options` (menu categories)

**Date strip / status:**
- `Sat 6th Oct EVE`, `Sun 7th Oct PM`, `Mon 8th Oct EVE`, `Tue 9th Oct
  AM`, `10.10.01 EVE`

## What this proves

1. **`sub_00770170` was executed** — the 8 category-tab labels are
   present in the exact order the exe's asm at `0x00770539..00770698`
   packs them. This directly validates the methodology's claim that
   News is recoverable via live capture even though Ghidra couldn't
   identify the function boundary.

2. **Layer 2's hooked primitives cover News' entire paint path** —
   65,701 primitive calls, 60 frame boundaries. No mystery non-hooked
   render path.

3. **From-address hot-spots** (top 3 by call count):
   - `from=0x1cd856` (inside FUN_005cd3e0 line, self-recursion for
     rectangle fill): 39,480 calls
   - `from=0x1d7c42` (FUN_005d7c42 — the widget-renderer's
     `draw_panel` callsite): 2,460 calls
   - `from=0x1d810d` (widget-renderer's `draw_wrapped_text`
     callsite): 2,400 calls

   The widget-renderer FUN_005d7aa0 (which we ported byte-exact in
   commits caaa49d..cae04fe) is confirmed as the hot dispatcher — the
   `0x1d7c42` and `0x1d810d` addresses ARE inside its body.

## Known limitations of this capture

1. **`readCString(256)` bug in `live_log.js`** — the JS hook reads text
   arg with `readCString(256)`. If the buffer isn't NUL-terminated
   within the first 256 bytes (some exe text buffers apparently
   aren't), Frida returns partial junk. Symptom: strings like
   `"Next?????????...(240 chars of junk)"`. The real text is always at
   the front of the string, before the first `?`. Fix: change to
   `readByteArray(N)` and split on 0x00 in Python.

2. **`--- PRESENT ---` op-string** in the JSONL is literal, not the
   `present` op name — a stringification quirk of the JS logger.
   Filter with `op.startswith('---')` or fix in live_log.js.

3. **Not a fixture matrix** — this is one capture at one game state.
   The methodology calls for many captures (empty vs populated news,
   read vs unread, tab-active variants, scrolled vs top). Follow-up
   task.

## How to reproduce

Start the game, get to the dashboard, then run:

```bash
D:/Python312/python.exe D:/cm0102-rs/tools/gdi_capture/live_log.py \
    D:/cm0102-rs/fixtures/news_capture.jsonl \
    --max-seconds 60 --silence-seconds 8
```

Then in the game: nav away → open manager menu → click News.

The recorder auto-exits on 8s of silence. Gzip the output before
committing.

## Next steps

1. Fix the `readCString` bug in `live_log.js` so recovered strings
   are clean.
2. Parse `news_capture.jsonl` into per-present widget groups (the 60
   frame-boundaries divide the capture into discrete redraws).
3. Subtract the shared sidebar+menu widget set (see
   `fixtures/screen_sidebar_and_manager_menu.md`) to isolate
   News-specific widgets.
4. Cross-reference the remaining News-specific widgets with the raw
   asm at `sub_00770170` in
   `/d/cm0102-carve/gdi_carve/functions/00004-PE_section_.text/04550_sub_0076fdb0.asm`
   to produce a byte-exact port of the News screen builder.

This capture is the ground-truth reference for that port.
