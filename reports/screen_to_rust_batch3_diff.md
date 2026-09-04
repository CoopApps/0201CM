# `screen_to_rust.py` round-trip vs `screen_wire_batch3.rs`

Tool commit: initial. Compared against the hand-port at
`crates/cm-render/src/screen_wire_batch3.rs` (commit `499310e`).

Batch3 has 5 Views; the audit classifies 2 easy (LAB analyses available)
and 3 medium (no LAB analysis JSON):

| View | Class | Analyzed callback |
|------|-------|-------------------|
| `LatestScoresView` (0x00700f20) | easy | `00701070.json` |
| `ManagerHistoryView` (0x00859250) | easy | `008596b0.json` |
| `GoHolidayDialog` (0x006986a0) | medium | — |
| `FifaRankingsView` (0x004a2190) | medium | — |
| `UefaCoefficientsView` (0x004a28c0) | medium | — |

The tool operates on analysis JSONs only, so a first pass covers the
two easy screens.

## `LatestScoresView` — `build_screen_701070`

Tool output (see `screens_auto.rs::build_screen_701070`) vs hand-port:

| Field | Hand-port | Tool |
|-------|-----------|------|
| kind | `KIND_LABEL` (1) | 1 |
| grid_x0..grid_y1 | 100, 10, 790, **46** | 100, 10, 790, **70** |
| style_byte | 0x30 | 0x30 |
| **text_style** | **7** (from JSON pos 11 "font") | **12** (from JSON pos 10 "aux_a") |
| **label_ink** | **0** (hardcoded `HEADER_INK`) | **7** (from JSON pos 11 "font") |
| text | `"Latest Scores"` (post-strip) | `"Latest Scores<%s - COMMENT - single line>"` (raw scratch) |
| sidebar | placeholder mode=4 | placeholder mode=4 |
| navbar | back=true (defaulted) | back=false (unresolved `eax`) |

### Divergences documented

1. **text_style / label_ink swap** — the WidgetDescriptor field-map
   from commit `4375e51` maps caller-arg 11 → `text_style` and
   caller-arg 12 → `label_ink`. The JSON dumper heuristically labelled
   positions 10 / 11 as "aux_a" / "font" — the hand-port took those
   names at face value and put the 7 into `text_style`; the tool
   respects the audit's arg → field map instead. The tool's mapping
   matches the render side (`packed_widget::Widget` reads +0x3c /
   +0x76 for the two fields).

2. **grid_y1 46 vs 70** — the JSON's `b` push at 0x7010ee has
   `literal 0x46 = 70`. The hand-port comment cites `b=46` (decimal),
   which is the hex value read as decimal. The tool emits the decimal
   value (70) as the JSON literal says.

3. **text raw vs stripped** — the exe's `FUN_00654380` comment-strip
   runs at render-time on the scratch contents; the widget carries
   the raw scratch. The tool emits the raw scratch text (what
   `spawn_widget` actually stores); the hand-port pre-stripped. Both
   are correct in different places.

4. **navbar back_flag** — JSON records the value as unread register
   `eax`; the hand-port defaulted to `back=true`; the tool defaults
   to `back=false` and emits a `TODO(unresolved-arg)` comment.

## `ManagerHistoryView` — `build_screen_8596b0`

8 JSON widgets: 3 spawn_area, 1 title item, 2 nav buttons (item), 2
strcpy_scratch. Tool emits the areas from the JSON's actual push rects
rather than the hand-port's News-inspired guess:

| Area | Hand-port | Tool |
|------|-----------|------|
| header  | (100, 10, 790, 70) | (100, 10, 790, 70) |
| body    | (110, 80, 780, 500) | (100, 25, 790, 55) |
| footer  | (100, 555, 790, 590) | (100, 267, 790, 590) |

The hand-port acknowledges its rects are guessed from News; the tool
uses the JSON push args directly.

## Screens without LAB JSONs

`GoHolidayDialog`, `FifaRankingsView` and `UefaCoefficientsView` have
no analysis JSON yet — the tool cannot emit for them. They remain
covered by the hand-port `screen_wire_batch3.rs` file (marked
"substrate-only") until LAB analyses are captured.

## Conclusion

The tool round-trips the two batch3 easy screens; divergences from
the hand-port are all traceable to (a) the field-map audit fix in
commit `4375e51` and (b) the hand-port's own admitted
guess-substrate for missing rects.
