# cm_render — faithful port of CM0102's draw pipeline

Goal: produce pixels the way the game does — real object records → faithful ports of the
game's own draw functions → 800×600 16-bit surface. **No hand-placement, no eyeballed colors,
no screenshot-copying.** If a behavior isn't lifted from the binary, it isn't drawn.

This exists because the earlier `render_cm_screen.py` output was NOT authentic: it hand-composed
rectangles and matched a screenshot by eye. That is theater, not what the game produces.

## Status

| Primitive | Source fn | State |
|--|--|--|
| Color pack/unpack (`surface.py`) | `FUN_005ce4f0` graphics_rgb_to_surface_pixel | **Ported + VERIFIED**: reproduces all 16 lifted color globals' packed RGB565 values exactly |
| 800×600 16-bit surface (`surface.py`) | ui_renderer_map native mode | Ported |
| Text box layout (`text.py`) | `FUN_005d0870` graphics_draw_text_box | Ported: box width/height, vertical-center by line-count×line-height, per-line align (bit0 left / bit6 right / else center), shadow (bit5) |
| Glyph blit (`text.py`) | `FUN_005ced50` + real `.fnt` bitmaps | Ported: advance-based, real coverage bitmaps. **Validated visually** vs the tight text in a real CM0102 screenshot |
| Text width (`text.py`) | `FUN_005cf610` (.fnt path) | Advance-sum. See caveat below |

### Honest caveat on text width

The decoder's `sample_widths` in `arial_14.json` use a `(height*3//4) − bearings` inter-glyph
spacing formula. Matching it produces **absurdly loose tracking** — wrong. Advance-only matches
the real game's tight text. So `sample_widths` is a bad lift and is NOT the ground truth;
"verifying" against it proves nothing. The real 16.16 fixed-point width fn `FUN_0059bed0`
(via `FUN_0059c1d0`) is a *separate loaded-metrics mode*, not the bitmap path — resolving which
mode the league screen uses at runtime (`DAT_009b88f8`) is a pending lift.

## Still needed for a full authentic screen

1. **Panel primitive** — lift the object-background fill + bevel draw (`0x005cf8e0` region).
2. **`guio_draw_object`** (`0x005d7fb0`) — the per-object orchestrator: which color field, panel
   vs text vs image path, clip rect. Large, dense function.
3. **Object-record construction with real bounds** — the table grid is fully lifted; the chrome
   objects' bounds are runtime-computed and only partially traced. Draw only what's lifted;
   leave the rest blank.
4. **Composition** — run construction → layout (`area_rebuild_layout_tables`, already lifted) →
   draw, over the real world DB.

Lesson recorded in memory: a verification is only as good as its ground truth. Match the game,
not a tool's guess.
