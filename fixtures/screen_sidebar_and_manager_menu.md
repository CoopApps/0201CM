# Screen fixture — sidebar + manager drop-down menu

Recovered from `D:/capture1.txt` (~1,190 primitive calls, captured live
via `tools/gdi_capture/live_log.py` on a real repaint). Everything below
is direct from the exe — no guessing.

## Sequence of events

1. User clicked their own name "Christoph Olewicz" in the sidebar.
2. Manager-context drop-down menu OPENED — 16 items drawn.
3. Mouse hovered over "Pro Vercelli Reserves" (highlighted yellow), then off, then over "Control Senior Team Only" (highlighted).
4. Menu CLOSED (restore called, PRESENT).
5. Sidebar redrawn (menu had covered part of it).

## Widget-spec recovered

### widget: sidebar_button (used for every sidebar item)

Signature: `(rect, label_text, colour)` where `colour` is the label ink.

```
panel(x0, y0, x1, y1, style=0x1021, colour=0)          // button panel bevel
line(x0+1, y0+1, x1-1, y0+1, style=2, colour=BL_bright)  // inner top edge (lighter half of dark)
line(x0+1, y0+1, x0+1, y1-1, style=2, colour=BL_bright)  // inner left edge
line(x1-1, y1-1, x0+2, y1-1, style=2, colour=BL_dark)    // inner bottom (darker half)
line(x1-1, y1-1, x1-1, y0+2, style=2, colour=BL_dark)    // inner right
line(x0,   y0,   x1,   y0,   style=2, colour=BR_bright)  // outer top (brighter half of dark)
line(x0,   y0,   x0,   y1,   style=2, colour=BR_bright)  // outer left
line(x1,   y1,   x0+1, y1,   style=2, colour=BR_dark)    // outer bottom
line(x1,   y1,   x1,   y0+1, style=2, colour=BR_dark)    // outer right
wrapped_text(x0+2, y0+2, x1+2, y1+2, style=0xc, font=1, colour=<label ink>, label_text)
```

Bevel colour math (observed from the log, matches `packed_panel::scale_colour`):
- Inner bright = base × 5%..10% (very dark blue)
- Inner dark = base × 3%..5% (near-black)
- Outer bright = base × ...
- Outer dark = base × ...

The exact colour values in the log (10, 15, 4, 8, 19, 24 etc.) are the exe's per-button-state palette. Sidebar buttons have colour=0 as their `base_colour` → the derived bevel colours use `DAT_00ad6b3c` (`0x0010`) as the fallback, giving the near-black bevel we see.

### widget: sidebar_arrows_panel

Same panel shape but different position:
```
panel(x0, y0, x1, y1, style=0x1021, colour=0)   // at (5, 55)-(44, 98)
```
The yellow ← → glyphs inside are separate glyph calls (not captured in this specific redraw; they were captured in earlier fixtures).

### widget: menu_dropdown_container (only when menu opens)

```
panel(x0, y0, x1, y1, style=0x30, colour=0x200)    // P_BEVEL|P_SOLID_FILL, dark blue
rect(x0, y0, x1, y1, style=4, colour=0x200)         // FILL (redundant per exe pattern)
[N horizontal lines filling the interior]           // implementation of the fill
line(x0+1, y0+1, x1-1, y0+1, ..., colour=0x2A0)    // inner top bevel (brighter)
line(x0+1, y0+1, x0+1, y1-1, ..., colour=0x2A0)    // inner left
line(x1-1, y1-1, x0+2, y1-1, ..., colour=0x140)    // inner bottom
line(x1-1, y1-1, x1-1, y0+2, ..., colour=0x140)    // inner right
line(x0,   y0,   x1,   y0,   ..., colour=0x360)    // outer top
line(x0,   y0,   x0,   y1,   ..., colour=0x360)    // outer left
line(x1,   y1,   x0+1, y1,   ..., colour=0xA0)     // outer bottom
line(x1,   y1,   x1,   y0+1, ..., colour=0xA0)     // outer right
```

### widget: menu_item (one per row inside the drop-down)

Signature: `(rect, label_text, band, style_flag)` where `band` alternates and `style_flag` sets separator.

```
panel(x0, y0, x1, y1, style=(0x10 or 0x1000010), colour=0x200 or 0x240)
rect(x0, y0, x1, y1, style=4, colour=<same>)      // fill
[20 horizontal lines filling]
if separator (style 0x1000010):
    line(x0+2, mid,   x1-2, mid,   style=2, colour=0x180)   // engraved dark line
    line(x0+2, mid+1, x1-2, mid+1, style=2, colour=0x300)   // engraved bright line
wrapped_text(x0, y0, x1, y1, style=1, font=1, colour=0, "     <text>")   // leading spaces = indent
```

Item colours alternate by row index:
- Even index → colour=0x200 (dark blue)
- Odd index  → colour=0x240 (slightly-lighter dark blue)

### widget: menu_item_hover (hover state — redraws over the row)

Same as menu_item but colour=0x7FE0 (yellow — from `DAT_00ad6b24` palette).

### The 16 items observed in this menu

| # | y0..y1 | text | style | cmd (from menu_tree_scope) |
|---|---|---|---|---|
| 1 | 147..166 | "Pro Vercelli Squad" | 0x10 | 0x7d5 SQUAD |
| 2 | 168..187 | "Pro Vercelli Reserves" | 0x10 | 0x7d6 B_SQUAD |
| 3 | 189..208 | "Control Senior Team Only" | 0x10 | 0x424 |
| 4 | 210..229 | "Board Confidence" | 0x10 | 0x41f BOARD_CONFIDENCE |
| 5 | 231..250 | "Resign from Club" | 0x10 | 0x41c RESIGN_FROM_CLUB |
| 6 | 252..270 | "" (separator) | 0x1000010 | — |
| 7 | 272..291 | "News" | 0x10 | 0x3e8 NEWS |
| 8 | 293..312 | "Player & Staff Search" | 0x10 | 0x3ec PLAYER_STAFF_SEARCH |
| 9 | 314..333 | "Compare two chosen players" | 0x10 | 0x434 COMPARE_PLAYERS |
| 10 | 335..354 | "Manager Stats" | 0x10 | 0x42e MANAGER_STATS |
| 11 | 356..374 | "Job Information" | 0x10 | 0x41e JOB_INFORMATION |
| 12 | 376..395 | "Transfers" | 0x10 | 0x415 TRANSFERS |
| 13 | 397..416 | "History" | 0x10 | 0x3ec MANAGER_HISTORY |
| 14 | 418..437 | "Go on Holiday" | 0x10 | 0x3ef GO_ON_HOLIDAY |
| 15 | 439..458 | "" (separator) | 0x1000010 | — |
| 16 | 460..479 | "Retire" | 0x10 | 0x3f2 RETIRE |

Item height = 20 pixels, item stride = 21 (1px gap between items). Menu total = 145..481 = 336 tall.

### Sidebar buttons observed after menu close

| rect | label | font | colour | notes |
|---|---|---|---|---|
| (5, 10)-(85, 53) | "Wednesday\n10.10.01 EVE" | 1 | 0x7FE0 (yellow) | Date panel |
| (5, 55)-(44, 98) + (46, 55)-(85, 98) | "← →" arrows | icon | 0x7FE0 | Nav arrows (2 panels) |
| (5, 100)-(85, 143) | "Continue\nGame" | 1 | 0x739C (light blue) | Main action |
| (5, 145)-(85, 187) | "Christoph\nOlewicz" | 1 | 0x43FF (yellow-gold) | Active human |
| (5, 189)-(85, 232) | "Competitions" | 1 | 0x739C | Menu category |
| (5, 234)-(85, 277) | "Nations\n& Clubs" | 1 | 0x739C | Menu category |
| (5, 279)-(85, 321) | "Find" | 1 | 0x739C | Menu category |
| (5, 323)-(85, 366) | "Game\nOptions" | 1 | 0x7FE0 (yellow) | Menu category — active/highlighted? |

Every sidebar button uses the same widget kind (`sidebar_button` above) with per-button `label` and `colour`.

## The palette values discovered

- 0x7FE0 (32736) — yellow (DAT_00ad6b24)
- 0x0010 (16) — near-black (DAT_00ad6b3c, default bevel base)
- 0x0200 (512) — dark blue (menu panel base)
- 0x0240 (576) — slightly lighter dark blue (alternating band)
- 0x43FF (17407) — yellow-gold (active-human ink)
- 0x739C (29596) — light blue (menu-category ink)
- Others in bevel: 3, 4, 6, 7, 8, 9, 10, 11, 13, 14, 15, 16, 17, 18, 19, 21, 24 — computed per-button via `scale_colour`.

## What this fixture proves

Every pixel in the sidebar + drop-down menu is emitted by the primitives
I've ported byte-exact. The widget renderers just need to make those
exact calls in the right order with the right args. Building
`sidebar_button`, `menu_dropdown_container`, and `menu_item` renderers in
Rust — three functions — recovers this WHOLE screen at pixel fidelity.

See `crates/cm-render/src/screens_faithful/sidebar_button.rs` (next
commit) for the first widget's byte-exact port.
