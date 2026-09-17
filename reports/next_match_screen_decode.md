# Next Match screen — GDI decode

**Captured:** 2026-09-17 from a live `cm0102_GDI.exe` (Version 3.9.60),
Bristol City, pre-season friendly v Crystal Palace (Away).

**Artifacts**

| File | What |
| --- | --- |
| `fixtures/next_match_screen/next_match.json` | Full fixture: 1,968 primitive calls + 800×600 RGB555 framebuffer |
| `fixtures/next_match_screen/structure.txt` | Human-readable op listing |
| `fixtures/next_match_screen/next_match_reference.png` | The framebuffer rendered, for eyeballing |

Masks: `r=0x7C00 g=0x03E0 b=0x001F` (RGB555), pitch 800.

---

## 1. How it was captured — a change to the harness

**CM 01/02 does not read synthetic mouse input.** Posting
`WM_MOUSEMOVE` / `WM_LBUTTONDOWN` / `WM_LBUTTONUP` to the game's
top-level HWND produced **zero** primitive calls — twice, including with
the window foregrounded, and with coordinates verified to land inside an
enabled button. The game polls the mouse directly rather than reading the
Windows message queue, so `tools/gdi_capture/capture_screen.py`'s
click-driven mode cannot work as written.

**`InvalidateRect` + `UpdateWindow` does work.** It drives the exe's own
paint path and emits the complete primitive stream for whatever screen is
displayed. Added as `rpc.forceRedraw(hwnd)` in `capture.js` and exposed
as `run_capture.py --auto`.

```bash
# Navigate the game to the state you want, then:
D:/Python312/python.exe tools/gdi_capture/run_capture.py \
    next_match fixtures/next_match_screen/next_match.json --auto
```

So the workflow is now: **a human navigates to the state once; capture
needs no clicking and no timing window.** Navigating *between* states
still requires a real human click.

**Frame boundary:** one forced redraw emits **two** paint passes of 984
calls each (the fixture holds 1,968). A consumer should treat calls
`[0..984)` as one complete frame.

Op mix per frame: 803 line, 58 panel, 52 text, 46 wrapped_text, 19 rect,
4 darken, 2 restore.

---

## 2. Layout geometry (exact, from the capture)

All coordinates are inclusive, in the 800×600 surface.

### Navigation row — y 125..145

| Control | Rect | State |
| --- | --- | --- |
| `<< Date` | (110,125)-(234,145) | **disabled** (first match) |
| `Date >>` | (236,125)-(360,145) | **enabled** |
| `Past Meetings` | (656,125)-(780,145) | enabled |

### Title and competition

| Element | Rect / position | Colour |
| --- | --- | --- |
| Opponent + venue, e.g. `Crystal Palace (Away)` | WRAP (110,150)-(780,185), text centred at (353,157) | `0x7FE0` yellow |
| Competition/type, e.g. `Friendly` | WRAP (115,198)-(187,218), left-aligned | `0x7E00` orange |

### Detail block — two columns, 21 px row pitch

* **Label column:** x 112..361, colour `0x739C` (grey)
* **Value column:** x 363..778, colour `0x7FE0` (yellow)
* **Row pitch: 21 px**, first row at y 241

| y | Label | Value observed |
| --- | --- | --- |
| 241 | `  Date` | `Thursday 19th July (5 days)` |
| 262 | `  Venue` | `Selhurst Park, London` |
| 283 | `  Match Rules` | `No player restrictions` |
| 304 | *(none — continuation)* | `9 subs named, maximum 9 used` |
| 325 | `  Last Meeting` | `-` |
| 346 | `  Weather Forecast` | `Dry, 21°C` |
| 367 | `  <Club> News` | first news line |
| 388 | | second news line |
| 409 | | third news line |

**Labels carry two leading spaces inside the string** (`'  Date'`, not
an x offset). Reproduce the spaces, not a shifted rectangle.

The news/availability rows use value column x **362**..778 (one pixel
left of the other rows) — reproduce as captured.

Observed news content, which is the availability list:

```
Lee Peacock out (broken shoulder)
Paul Holland out (damaged cruciate ligaments)
Billy Mercer out (torn groin muscle)
```

### Shared chrome (matches the other screen captures)

* Top tabs y 80..115: Squad (100..237), Transfers (239..375),
  **Next Match** (377..513, selected), Fixtures (515..651),
  General Info (653..790).
* Bottom tabs y 510..545: Tactics, Training *(disabled, `0x4210`)*,
  Last Match, `<Division>`, History — enabled ones in `0x43FF` cyan.
* Selected top tab draws in `0x7FE0`; unselected in `0x739C`.

---

## 3. Enabled vs disabled rendering — a general rule

Worth extracting because it applies to every control in the game.

| State | How the text is drawn |
| --- | --- |
| **Enabled** | ONE text call, flat, at the control colour (`0x739C` for these buttons) |
| **Disabled** | TWO text calls — a light shadow `0x6F7A` at (x+1, y+1), then the dark main `0x294A` at (x, y). The Win9x "engraved" look |

Evidence, from this capture:

```text
605 WRAP (110,125)-(234,145) c=0x739c '<< Date'
606 TEXT (149,129) c=0x6f7a '<< Date'     <- shadow
607 TEXT (148,128) c=0x294a '<< Date'     <- dark main   => DISABLED

639 WRAP (236,125)-(360,145) c=0x739c 'Date >>'
640 TEXT (274,128) c=0x739c 'Date >>'     <- single flat  => ENABLED
```

Confirmed visually by cropping the framebuffer: `<< Date` is engraved,
`Date >>` is flat.

Note the **WRAP call's colour is the same (`0x739C`) either way** — the
enabled/disabled distinction lives in the TEXT calls, not the layout
call. Do not infer state from the WRAP.

**The disabled reading is confirmed, not inferred.** `<< Date` is
disabled here *because this is the club's first fixture — there is no
earlier match to step back to* (confirmed by the user against the
original game). So the engraved two-pass rendering above IS the disabled
rendering, and the `0x294A` dark main is its signature.

*(Open detail, not needed for this screen: the sidebar's `Add Manager` /
`Restart Game` / `Exit Game` also draw twice, with near-black colour
pairs — `(0x0013, 0x0007)`, `(0x0012, 0x0007)`, `(0x0010, 0x0006)` — and
they render dim in the framebuffer. That may be the same disabled
treatment in a different palette, or a separate shadow technique. It does
not affect the rule for list/navigation controls, which is what §3
covers.)*

---

## 4. Behavioural contract

From the user's description of the original, plus what the capture shows:

1. Opens on the **next upcoming fixture** for the club.
2. **`Date >>`** advances to the following match in the fixture list;
   **`<< Date`** steps back.

   **Enablement rule (confirmed):** each arrow is enabled only when a
   fixture exists in that direction. `<< Date` is disabled here because
   this is the club's first fixture — there is no earlier match. By the
   same rule `Date >>` should disable on the last fixture of the list
   (not yet observed — §5).

   So the port must gate these two controls on the club's position
   within its own fixture list, and render the disabled state with the
   engraved treatment in §3 — not simply grey the text.
3. The `<Club> News` block lists **players unavailable for this match**,
   with the reason in parentheses — observed (three injuries).
4. It also lists **players who would make their debut** — *not observed
   in this capture*; no eligible player in a pre-season friendly.
5. `Past Meetings` opens the head-to-head history.

---

## 4b. Competitive fixture + debut list (captured 2026-09-17)

Second capture: **Cambridge United v Stoke (Home), Second Division**,
the opening league fixture. `fixtures/next_match_screen/next_match_debut.json`
+ `structure_debut.txt` + `next_match_debut_reference.png`. This resolves
most of the §5 unknowns.

### The News block lists debuts AND injuries together

The `<Club> News` block is the **availability list**: it shows every
player unavailable (injury, `X out (reason)`) *and* every player set to
debut (`X set for debut`). Same style as the injury lines — yellow
`0x7FE0`, value column x **355**..758, 21 px pitch from y 367:

```text
755 WRAP (355,367)-(758,386) c=0x7fe0 'Tony Scully set for debut'
760 WRAP (355,388)-(758,407) c=0x7fe0 'Phil Warner set for debut'
765 WRAP (355,409)-(758,428) c=0x7fe0 'Stevland Angus set for debut'
770 WRAP (355,430)-(758,449) c=0x7fe0 'Armand One set for debut'
775 WRAP (355,451)-(758,470) c=0x7fe0 'Alex Revell set for debut'
780 WRAP (355,472)-(758,492) c=0x7fe0 'Warren Goodhind set for debut'
```

Line format: **`<Full Name> set for debut`** (first + last name, not the
squad-list "Surname, I" form). When the block overflows the panel the
shared scrollbar appears (see §2 chrome; this capture has it).

### Competitive vs friendly differences

| Field | Friendly (Bristol) | Competitive (Cambridge, Div 2) |
| --- | --- | --- |
| Match Rules line 1 | `No player restrictions` | `Max 3 non-EU players in the match squad` |
| Match Rules line 2 | `9 subs named, maximum 9 used` | `5 subs named, maximum 3 used` |
| Weather | `Dry, 21°C` | `Unknown` (far-future fixture) |
| Extra button | — | **`Progress`** at (530,125)-(654,145) |
| Venue | away side's ground | home side's ground (`The Abbey Stadium, Cambridge`) |

So the button row has FIVE slots on a competitive fixture:
`<< Date` (110-234), `Date >>` (236-360), `Progress` (530-654),
`Past Meetings` (656-780). The friendly capture had no `Progress`.

### `<< Date` disabled confirmed again

Cambridge v Stoke is the season opener, so `<< Date` is disabled (engraved
two-pass, `0x6F7A`+`0x294A`) — same rule as the friendly. `Date >>` is
enabled (single `0x739C`).

## 5. UNKNOWN / UNCAPTURED

Per the project rule, these are recorded as unknown rather than guessed:

* **The debut line's wording, colour and position.** Requires a capture
  of a match where a player would debut.
* **`<< Date` in its ENABLED state.** Expected to match `Date >>` (one
  flat call), but unobserved. Low priority now that the disabled case is
  confirmed.
* **`Date >>` on the LAST fixture**, to confirm it disables by the same
  rule.
* **A competitive fixture.** Everything here is a friendly; league and
  cup matches may populate `Match Rules`, `Last Meeting` and the
  competition label differently.
* **Pressed states** for any of these buttons.
* **What happens at the end of the fixture list** — whether `Date >>`
  disables.

All of these need a human to navigate the original game to the state;
the capture itself is then a single `--auto` command.
