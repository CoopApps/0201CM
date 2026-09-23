//! Panel + bevel + gradient — literal port of `FUN_005cf570` (GDI variant,
//! 3617 bytes). This is the primitive every screen chrome (banner,
//! button, group box, scrollbar track, dashboard tile) actually calls;
//! the current cm-render/src/panel.rs is a hand-tuned approximation
//! (invented Bayer dithering) — this module is the ground truth.
//!
//! Style bits ("param_5" in the exe):
//!
//! | bit      | mnemonic              | effect                                              |
//! | -------- | --------------------- | --------------------------------------------------- |
//! | 0x0000_2 | `DARKEN`              | interior = `FUN_005cdd60` darken (60/100 LUT)       |
//! | 0x0000_4 | `HGRADIENT`           | interior = per-column colour scaled by `100+d`      |
//! | 0x0000_8 | `VGRADIENT`           | interior = per-row colour scaled by `100+d`         |
//! | 0x0000_10| `SOLID_FILL`          | interior = solid rect fill                          |
//! | 0x0000_20| `BEVEL`               | 3D bevel around edge (thickness 2 or 4)             |
//! | 0x0000_40| `BEVEL_INVERT`        | swaps sunken/raised — see the ±cVar3 pairing        |
//! | 0x0000_80| `BOTTOM_SHADOW`       | one-pixel shadow line at bottom, scaled 0x5a/100    |
//! | 0x0000_200|`SOLID_FRAME`         | 1-px frame (or dashed if 0x2000 also set)           |
//! | 0x0000_400|`RIGHT_EDGE`          | vertical/right edge line, variant per 0x2000/0x4000 |
//! | 0x0000_800|`OUTER_HIGHLIGHT`     | 1-px frame ONE PIXEL outside the rect               |
//! | 0x0000_1000|`SAMPLE_BG`          | read the (x0,y0) pixel as `param_6` (transparent bg)|
//! | 0x0000_2000|`_DASH_VARIANT`      | switches frame/edge to the dashed style             |
//! | 0x0000_4000|`_EDGE_MIDLINE`      | RIGHT_EDGE runs from midheight instead of top       |
//! | 0x0000_8000|`_EDGE_FULL_HEIGHT`  | ignore mid-height for RIGHT_EDGE                    |
//! | 0x0001_0000|`_ALT_PRIMITIVE`     | interior / frame drawn as a CIRCLE via `FUN_005d0ce0` |
//! | 0x0002_0000|`SHRINK_2`           | `right -= 2; bottom -= 2` before drawing            |
//! | 0x0004_0000|`SHIFT_2`            | `left += 2; top += 2` before drawing                |
//! | 0x0100_0000|`MIDLINE_H`          | horizontal separator — two 1-px lines at mid-y      |
//!
//! Every callee of `sub_005cf570` is ported: `FUN_005cd330` (clip),
//! `FUN_005cdd60` (darken), `FUN_005cd730` (rect), `FUN_005cd3e0` (line)
//! live in `packed.rs`; `FUN_005d0ce0` + its octant plotter `FUN_005d0ec0`
//! are [`draw_circle`] / [`plot_circle_octants`] below. The two panel
//! call sites of the circle primitive are:
//!
//! * `005cf652..005cf666` — `SOLID_FILL | _ALT_PRIMITIVE`:
//!   `FUN_005d0ce0(x0, y0, x1, y1, 4, colour)` — filled disc in `param_6`.
//! * `005cf9db..005cfa23` — `BEVEL | _ALT_PRIMITIVE`:
//!   `FUN_005d0ce0(x0, y0, x1, y1, DASH ? 1 : 2, param_7)` — outline in
//!   the panel's 7th argument (`[esp+0x80]`, the "pattern colour").

use crate::packed::PackedSurface;

// ------ Style flag constants ------

pub const P_DARKEN: u32          = 0x0000_0002;
pub const P_HGRADIENT: u32       = 0x0000_0004;
pub const P_VGRADIENT: u32       = 0x0000_0008;
pub const P_SOLID_FILL: u32      = 0x0000_0010;
pub const P_BEVEL: u32           = 0x0000_0020;
pub const P_BEVEL_INVERT: u32    = 0x0000_0040;
pub const P_BOTTOM_SHADOW: u32   = 0x0000_0080;
pub const P_SOLID_FRAME: u32     = 0x0000_0200;
pub const P_RIGHT_EDGE: u32      = 0x0000_0400;
pub const P_OUTER_HIGHLIGHT: u32 = 0x0000_0800;
pub const P_SAMPLE_BG: u32       = 0x0000_1000;
pub const P_DASH_VARIANT: u32    = 0x0000_2000;
pub const P_EDGE_MIDLINE: u32    = 0x0000_4000;
pub const P_EDGE_FULL_HEIGHT: u32= 0x0000_8000;
pub const P_ALT_PRIMITIVE: u32   = 0x0001_0000;
pub const P_SHRINK_2: u32        = 0x0002_0000;
pub const P_SHIFT_2: u32         = 0x0004_0000;
pub const P_MIDLINE_H: u32       = 0x0100_0000;

/// The two palette globals `draw_panel` reads:
///
/// * `DAT_00ad6b24` — outer-highlight colour (`P_OUTER_HIGHLIGHT`),
///   read at `005d0369` (`mov cx, word ptr [0xad6b24]`).
/// * `DAT_00ad6b3c` — "default bevel colour", substituted whenever the
///   caller's `colour == 0` on the bevel / bottom-shadow / midline paths
///   (`005cfb15`, `005cfe89`, `005d0077`: `mov ecx, dword ptr [0xad6b3c]`).
///
/// Both live in the exe's `.data` BSS tail (VA `0x00ad6b24` / `0x00ad6b3c`
/// are past `.data`'s `SizeOfRawData = 0x152000`, so `pefile` reads them
/// back as absent — they are loader-zero at process start). They are
/// written at runtime by the palette-reload routine `FUN_005cdfa0`
/// (GDI `sub_005cdfa0`, 658 bytes): see [`PanelPalette::from_palette_reload`],
/// which ports exactly those two stores. `Default` is the pre-reload
/// (loader-zero) state.
///
/// Live read-back from the running `cm0102_GDI.exe` (fixtures/verify_panel.json
/// `palette`): `outer_highlight = 0x7fe0`, `default_bevel = 0x0010` — which
/// is what `from_palette_reload` computes for the RGB555 surface.
#[derive(Debug, Clone, Copy)]
pub struct PanelPalette {
    pub outer_highlight: u16,
    pub default_bevel: u16,
}

impl Default for PanelPalette {
    /// Loader-zero BSS state — what the exe holds before `FUN_005cdfa0` runs.
    fn default() -> Self {
        Self { outer_highlight: 0, default_bevel: 0 }
    }
}

impl PanelPalette {
    /// Port of the two `FUN_005cdfa0` (GDI `sub_005cdfa0`) stores that feed
    /// `draw_panel`. The routine as a whole re-derives ~24 palette words from
    /// the surface pixel format after `FUN_005cc4f0` has installed it; only
    /// these two are consumed here.
    ///
    /// ```text
    /// 005cdfa0  mov  eax, [0xad6b44]      ; DAT_00ad6b44 gate
    /// 005cdfa5  test eax, eax
    /// 005cdfa7  jne  005ce231             ; non-zero → ret, nothing written
    /// 005cdfad  mov  eax, [0xacdeac]      ; pixel-format green mask
    /// ...
    /// 005ce080  mov  ecx, eax
    /// 005ce082  sub  ecx, 0x7e0           ; ecx = green_mask - 0x7e0
    /// 005ce088  neg  ecx
    /// 005ce08a  sbb  ecx, ecx             ; ecx = (green_mask != 0x7e0) ? -1 : 0
    /// 005ce091  and  ecx, 0xffff8000      ; → 0xffff8000 (555) or 0 (565)
    /// 005ce097  add  ecx, 0xffe0          ; → cx = 0x7fe0 (555) or 0xffe0 (565)
    /// 005ce0a1  mov  [0xad6b24], cx       ; DAT_00ad6b24 = outer highlight
    /// ...
    /// 005ce137  push 0                    ; FUN_005ce240 arg4 (unused alpha)
    /// 005ce139  push 0x80                 ; b = 0x80
    /// 005ce13e  push 0                    ; g = 0
    /// 005ce140  push 0                    ; r = 0
    /// 005ce148  call 0x5ce240             ; pack_rgb(0, 0, 0x80)
    /// 005ce155  mov  [0xad6b3c], ax       ; DAT_00ad6b3c = default bevel
    /// ```
    ///
    /// `FUN_005ce240` is `PackedSurface::pack_rgb` (dispatching on the same
    /// green-mask compare), so the surface's mask set selects the format.
    pub fn from_palette_reload(s: &PackedSurface) -> Self {
        // 005cdfa0..005cdfa7 — DAT_00ad6b44 == 0 on the GDI build
        // (packed_widget_globals::DAT_00AD6B44); a non-zero gate leaves the
        // loader-zero palette untouched.
        if crate::packed_widget_globals::DAT_00AD6B44 != 0 {
            return Self::default();
        }
        // 005ce080..005ce0a1
        let outer_highlight: u16 = if s.green_mask == 0x7e0 { 0xffe0 } else { 0x7fe0 };
        // 005ce137..005ce155
        let default_bevel: u16 = s.pack_rgb(0, 0, 0x80);
        Self { outer_highlight, default_bevel }
    }
}

/// Literal port of `FUN_005cf570` (3617 bytes). `x0..=x1, y0..=y1` is the
/// inclusive rect; `style` carries the P_* bits above; `colour` is the
/// primary colour (`param_6`, `[esp+0x7c]`); `pattern_colour` is `param_7`
/// (`[esp+0x80]`) — consumed only by the `P_BEVEL | P_ALT_PRIMITIVE`
/// circle-outline path at `005cf9db`.
// GDI-REG: 005cf8e0 PORTED_EXACT
pub fn draw_panel(
    s: &mut PackedSurface,
    x0: i32,
    y0: i32,
    mut x1: i32,
    mut y1: i32,
    style: u32,
    mut colour: u16,
    pattern_colour: u16,
    palette: PanelPalette,
) {
    // --- preamble: shrink / shift / sample-bg ---
    if (style & P_SHRINK_2) != 0 {
        x1 -= 2;
        y1 -= 2;
    }
    let orig_right = x1;
    let (mut x0, mut y0) = (x0, y0);
    if (style & P_SHIFT_2) != 0 {
        x0 += 2;
        y0 += 2;
    }
    if (style & P_SAMPLE_BG) != 0 {
        // Read the pixel at (x0, y0) as the operative colour — the exe
        // uses this for "transparent background" panels that pick up the
        // colour of whatever they're painted over.
        if let Some((cx, cy, _, _)) = s.clip(x0, y0, x0, y0) {
            let idx = (s.pitch_pixels * cy + cx) as usize;
            colour = s.buf[idx];
        } else {
            colour = 0;
        }
    }

    // --- interior fill dispatch ---
    if (style & P_DARKEN) != 0 {
        s.darken_rect(x0, y0, x1, y1);
    } else if (style & P_SOLID_FILL) != 0 {
        if (style & P_ALT_PRIMITIVE) != 0 {
            // 005cf652..005cf666: push colour; push 4; push y1; push x1;
            // push y0; push x0; call 0x5d0ce0 — filled disc in `param_6`.
            draw_circle(s, x0, y0, x1, y1, CIRCLE_FILL, colour);
        } else {
            // Rect style=4 in the exe = neither bit 0 nor bit 1 set = FILL.
            s.draw_rectangle(x0, y0, x1, y1, 0, colour);
        }
    } else if (style & P_HGRADIENT) != 0 {
        // Per-column colour: 100 + (i / (x1-x0)) percent — a downward
        // ramp because `local_54` is initialised to 0 then decremented
        // by 100 each column, giving `local_54 / (x1-x0)` a value that
        // matches `-i * 100 / (x1-x0)`.
        if x0 <= x1 {
            let span = (x1 - x0).max(1);
            let mut local_54: i32 = 0;
            let mut ix = x0;
            while ix <= x1 {
                let scale = (local_54.wrapping_div(span) as i8).wrapping_add(100) as u8;
                let c = scale_colour(s, colour, scale as u32);
                s.draw_line(ix, y0, ix, y1, 2, c);
                ix += 1;
                local_54 = local_54.wrapping_sub(100);
            }
        }
    } else if (style & P_VGRADIENT) != 0 {
        if y0 <= y1 {
            let span = (y1 - y0).max(1);
            let mut local_54: i32 = 0;
            let mut iy = y0;
            while iy <= y1 {
                let scale = (local_54.wrapping_div(span) as i8).wrapping_add(100) as u8;
                let c = scale_colour(s, colour, scale as u32);
                s.draw_line(x0, iy, x1, iy, 2, c);
                iy += 1;
                local_54 = local_54.wrapping_sub(100);
            }
        }
    }

    // --- frame / bevel / edge dispatch ---
    if (style & P_BEVEL) == 0 {
        if (style & P_BOTTOM_SHADOW) != 0 {
            // Single bottom shadow line: colour scaled 0x5a/100 = 90%.
            let c = scale_colour(s, if colour == 0 { palette.default_bevel } else { colour }, 0x5a);
            // exe: iVar13 = param_1 + 2; param_3 -= 2; iVar14 = param_4;
            //      FUN_005cd3e0(iVar13, iVar14, param_3, iVar5, 1, c);
            //      style=1 (dashed) — bit 0 set in line.
            s.draw_line(x0 + 2, y1, x1 - 2, y1, 1, c);
        } else if (style & P_RIGHT_EDGE) != 0 {
            // Vertical right-edge line (with midline / full-height /
            // solid-vs-dashed variants). Faithful transcription of the
            // exe's iVar13/iVar14/iVar5 dance.
            let mut iv14 = y0;
            let mut iv5 = y1;
            let iv13 = x0;
            let line_style = if (style & P_DASH_VARIANT) != 0 { 1 } else { 2 };
            if (style & P_EDGE_MIDLINE) != 0 {
                if (style & P_EDGE_FULL_HEIGHT) == 0 {
                    iv14 = (y1 - y0) / 2 + y0;
                }
                iv5 = iv14;
            }
            let _ = iv14;
            s.draw_line(iv13, y0, x1, iv5, line_style, colour);
        }
    } else if (style & P_ALT_PRIMITIVE) == 0 {
        if (style & P_SOLID_FRAME) == 0 {
            // The 3D bevel. Thickness = 4 iff the rect is >= 50 wide,
            // >= 50 tall, y1 < 100, AND `P_SOLID_FILL` set. Else 2.
            let dx = x1 - x0;
            let dy = y1 - y0;
            let thick = if dx < 0x32 || dy < 0x32 || y1 > 99 || (style & P_SOLID_FILL) == 0 {
                2
            } else {
                4
            };
            if thick != 0 {
                // Exact variable names from the exe's bevel loop
                // (FUN_005cf570 near LAB after `if ((param_5 & 0x200) == 0)`).
                // local_44 = column counter, starts at right - thick, increments
                //            each iteration up to `right`.
                // iVar14 = row counter, starts at top + thick, becomes iVar12 = iVar14-1
                //          each iteration, walking down from top+thick-1 to top.
                // iVar15 = constant shear: `(thick + x0) - (y0 + thick) = x0 - y0`.
                // iVar13 = CONSTANT: `(bottom - thick) - (right - thick) = bottom - right`.
                //          Every iteration computes `iVar2 = iVar13 + local_44`, so the
                //          bottom-edge y coord is `bottom + (col - right)`. Earlier
                //          revision of this port wrote `let iv2 = iv14 + iv13` — used
                //          the row counter where the constant belonged — so no bevel
                //          line landed on the intended pixels. Fixed here + verified
                //          against exe (verify_panel_against_exe.rs).
                let mut lc: i32 = 0x42; // per-step scale accumulator
                let shear = x0 - y0;                        // was iVar15
                let mut col_counter = x1 - thick;           // was local_44
                let mut row_counter = y0 + thick;           // was iVar14
                let bottom_offset = (y1 - thick) - col_counter; // was iVar13 CONSTANT
                let base = if colour == 0 { palette.default_bevel } else { colour };
                for _ in 0..thick {
                    col_counter += 1;
                    let inner_row = row_counter - 1;        // was iVar12
                    let cv3 = (lc / thick) as i8;
                    // Two ramps: 100+cv3 brightens, 100-cv3 darkens. Paired so
                    // top-left is bright + bottom-right dark → raised look; swap
                    // for sunken (P_BEVEL_INVERT).
                    let bright = scale_colour(s, base, (cv3.wrapping_add(100) as u8) as u32);
                    let dark   = scale_colour(s, base, (100i8.wrapping_sub(cv3) as u8) as u32);
                    let (mut top_left, mut bot_right) = (bright, dark);
                    if (style & P_BEVEL_INVERT) != 0 {
                        std::mem::swap(&mut top_left, &mut bot_right);
                    }
                    let x_left = shear + inner_row;             // was iVar1
                    let y_bottom = bottom_offset + col_counter; // was iVar2
                    // 1. TOP edge (bright): horizontal at y=inner_row from x_left to col_counter.
                    s.draw_line(x_left, inner_row, col_counter, inner_row, 2, top_left);
                    // 2. LEFT edge (bright): vertical at x=x_left from y=inner_row to y=y_bottom.
                    s.draw_line(x_left, inner_row, x_left, y_bottom, 2, top_left);
                    // 3. BOTTOM edge (dark): horizontal at y=y_bottom from col_counter back to shear+row_counter.
                    s.draw_line(col_counter, y_bottom, shear + row_counter, y_bottom, 2, bot_right);
                    // 4. RIGHT edge (dark): vertical at x=col_counter from y_bottom to row_counter.
                    s.draw_line(col_counter, y_bottom, col_counter, row_counter, 2, bot_right);
                    lc += 0x42;
                    row_counter = inner_row;
                }
            }
        } else if (style & P_DASH_VARIANT) == 0 {
            // Solid frame around the rect.
            s.draw_rectangle(x0, y0, x1, y1, 2, colour);
        } else {
            // Dashed frame.
            s.draw_rectangle(x0, y0, x1, y1, 1, colour);
        }
    } else {
        // 005cf9d3..005cfa23: BEVEL | _ALT_PRIMITIVE → circle OUTLINE in
        // `param_7` ([esp+0x80], pushed at 005cf9e5). `test ch, 0x20`
        // (= style & 0x2000, P_DASH_VARIANT) selects the circle style:
        //   set   → 005cf9f8 `push 1` → dashed arc (neither bit 2 nor 4)
        //   clear → 005cfa18 `push 2` → solid arc
        // then push y1/x1/y0/x0 ([esp+0x78/0x74/0x70/0x6c] after the
        // first push) and `call 0x5d0ce0`; `jmp 0x5d005b` rejoins the
        // midline check below.
        let circle_style = if (style & P_DASH_VARIANT) != 0 { 1 } else { CIRCLE_SOLID };
        draw_circle(s, x0, y0, x1, y1, circle_style, pattern_colour);
    }

    // --- LAB_005d0062: midline separator ---
    if (style & P_MIDLINE_H) != 0 {
        let base = if colour == 0 { palette.default_bevel } else { colour };
        // Two lines at mid-height: top line scaled 0x85 (133% clamped to 255),
        // bottom line scaled 0x42 (66%).
        let bright = scale_colour(s, base, 0x85);
        let dark = scale_colour(s, colour, 0x42);
        let mid = (y1 - y0) / 2 + y0;
        s.draw_line(x0 + 2, mid, orig_right - 2, mid, 2, dark);
        s.draw_line(x0 + 2, mid + 1, orig_right - 2, mid + 1, 2, bright);
    }
    // --- outer highlight ---
    if (style & P_OUTER_HIGHLIGHT) != 0 {
        s.draw_rectangle(x0 - 1, y0 - 1, orig_right + 1, y1 + 1, 2, palette.outer_highlight);
    }
}

/// Apply a whole-number percentage scale to a packed 16-bit colour, in
/// the surface's pixel format. Ports the exe's arithmetic exactly:
/// unpack channels via `((v & mask) << 8) / (mask + 1)`, scale by
/// `pct/100` with `>0xfe → 0xff` clamps, repack via the same 555/565
/// dispatch as `pack_rgb`.
pub fn scale_colour(s: &PackedSurface, colour: u16, pct: u32) -> u16 {
    let rm = s.red_mask as u32;
    let gm = s.green_mask as u32;
    let bm = s.blue_mask as u32;
    let cu = colour as u32;
    let r = ((cu & rm) << 8) / (rm + 1) & 0xff;
    let g = ((cu & gm) << 8) / (gm + 1) & 0xff;
    let b = ((cu & bm) << 8) / (bm + 1) & 0xff;
    let r = ((r * pct) / 100).min(0xff);
    let g = ((g * pct) / 100).min(0xff);
    let b = ((b * pct) / 100).min(0xff);
    s.pack_rgb(r as u8, g as u8, b as u8)
}

// ------ FUN_005d0ce0 / FUN_005d0ec0 — the circle primitive ------

/// `FUN_005d0ce0` style bit 2: solid arc (every midpoint step plotted).
pub const CIRCLE_SOLID: u32 = 0x2;
/// `FUN_005d0ce0` style bit 4: filled disc (per-row span list + span fill).
pub const CIRCLE_FILL: u32 = 0x4;
// Neither bit set: dashed arc — a step is plotted iff `step % 6 < 4`.

/// One entry of the per-row span list `FUN_005d0ce0` mallocs for the
/// fill mode: 8 bytes `{min_x, max_x}`, initialised to `{0x320, 0}`
/// (`005d0d42`/`005d0d48`). Entry `k` covers surface row `y0 + k`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub min: i32,
    pub max: i32,
}

/// Literal port of `FUN_005d0ce0` (GDI, 469 bytes / 176 instructions,
/// cdecl, 6 args — `add esp, 0x18` at both panel call sites).
///
/// Draws a circle inscribed in the height of `(x0, y0)..(x1, y1)`
/// (radius derives from the height only; the x-extent is clipped to the
/// rect) using the midpoint algorithm, in one of three modes selected by
/// `style`: [`CIRCLE_FILL`] (bit 4), [`CIRCLE_SOLID`] (bit 2), or dashed
/// (neither). Stack-slot names from the asm are given as `[esp+..]`.
pub fn draw_circle(
    s: &mut PackedSurface,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    style: u32,
    colour: u16,
) {
    // 005d0ce5/005d0cf9/005d0cfa: ebp = x0; inc → [esp+0x24] = x0 + 1 (l)
    let l = x0.wrapping_add(1);
    // 005d0cfe/005d0d02: ebp = y0 + 1 (t)
    let t = y0.wrapping_add(1);
    // 005d0cea/005d0d03: esi = x1 - 1 → [esp+0x2c] (r)
    let r = x1.wrapping_sub(1);
    // 005d0cee/005d0d04: ebx = y1 - 1 (b)
    let b = y1.wrapping_sub(1);
    // 005d0cf3/005d0d05/005d0d10: [esp+0x14] = style & 4 (fill flag)
    let fill = (style & CIRCLE_FILL) != 0;
    // 005d0cf7/005d0d08: edi = 0 → [esp+0x10] = NULL (span list)
    let mut spans: Option<Vec<Span>> = None;
    // 005d0d14: je 005d0d51 — no list unless filling
    if fill {
        // 005d0d16..005d0d2c: malloc(eax*8 + 0x18) with eax = (t<<29) - t + b,
        // i.e. (mod 2^32) 8*(b - t) + 24 = 8 * ((b - t) + 3) bytes.
        // 005d0d37..005d0d40: ecx = (b - t) + 3 entries; jle → skip init.
        let count = b.wrapping_sub(t).wrapping_add(3);
        // 005d0d42..005d0d4f: [eax] = 0x320; [eax+4] = 0; eax += 8; dec ecx
        spans = Some(vec![Span { min: 0x320, max: 0 }; count.max(0) as usize]);
    }
    // 005d0d51: [esp+0x18] = DAT_00ad6b1c — the surface buffer (`s`).
    // 005d0d56..005d0d5e: [esp+0x1c] = b - t (ih)
    let ih = b.wrapping_sub(t);
    // 005d0d62: [esp+0x30] = 0 — the arc step counter (dash phase)
    let mut step: i32 = 0;
    // 005d0d66..005d0d6e: esi = (ih + 1) / 2   (cdq; sub; sar → trunc toward 0)
    let mut y: i32 = ih.wrapping_add(1) / 2;
    // 005d0d70..005d0d7f: [esp+0x28] = (3 - 2*y) as i16 — midpoint decision var
    let mut d: i32 = (3i32.wrapping_sub(y.wrapping_mul(2)) as i16) as i32;
    // edi (x) is still 0 from 005d0cf7.
    let mut x: i32 = 0;
    // 005d0d7d/005d0d83: test esi, esi; jle 005d0e19 — skip arc loop if y <= 0
    if y > 0 {
        loop {
            // 005d0d89..005d0da7: plot gate (fill | style&2 | step%6 < 4)
            if circle_step_visible(fill, style, step) {
                // 005d0da9..005d0dcb: FUN_005d0ec0(buf, x, y, l, t, r, b, colour, list)
                plot_circle_octants(s, x, y, l, t, r, b, colour, &mut spans);
            }
            // 005d0dce..005d0dd4: test d
            if d < 0 {
                // 005d0dd6..005d0de2: d += (4*x + 6) as i16
                d = d.wrapping_add((x.wrapping_mul(4).wrapping_add(6) as i16) as i32);
            } else {
                // 005d0de8..005d0dfd: d += (4*(x - y) + 10) as i16; y -= 1
                d = d.wrapping_add(
                    (x.wrapping_sub(y).wrapping_mul(4).wrapping_add(10) as i16) as i32,
                );
                y -= 1;
            }
            // 005d0e01..005d0e0d: step += 1; x += 1; jl while x < y
            step += 1;
            x += 1;
            if x >= y {
                break;
            }
        }
    }
    // 005d0e17/005d0e19: test esi, esi; je 005d0e64 — the 45° point
    if y != 0 {
        // 005d0e1b..005d0e39: same plot gate on the current step count
        if circle_step_visible(fill, style, step) {
            // 005d0e3b..005d0e5d: FUN_005d0ec0(buf, y, y, l, t, r, b, colour, list)
            plot_circle_octants(s, y, y, l, t, r, b, colour, &mut spans);
        }
    }
    // 005d0e64..005d0e6a: list == NULL → done
    if let Some(sp) = spans {
        // 005d0e6c..005d0e73: ebx = ih + 3 (count); jle → skip
        // 005d0e75..005d0e9e: for k in 0..count: if entry.max != 0:
        //   FUN_005cd3e0(entry.min, k + t - 1, entry.max, k + t - 1, 2, colour)
        for (k, e) in sp.iter().enumerate() {
            if e.max != 0 {
                // 005d0e82: lea eax, [esi + ebp - 1] → row = k + (y0+1) - 1
                let row = (k as i32).wrapping_add(t).wrapping_sub(1);
                s.draw_line(e.min, row, e.max, row, 2, colour);
            }
        }
        // 005d0ea0..005d0eaa: free(list) — `sp` drops here.
    }
}

/// The plot gate shared by the arc loop (`005d0d89..005d0da7`) and the
/// 45° tail (`005d0e1b..005d0e39`):
///
/// ```text
/// mov  eax, [esp+0x14]      ; fill flag
/// test eax, eax ; jne PLOT
/// test byte [esp+0x34], 2   ; style & 2 (solid)
/// jne  PLOT
/// mov  eax, [esp+0x30]      ; step
/// mov  ecx, 6 ; cdq ; idiv ecx
/// cmp  edx, 4 ; jge SKIP    ; dashed: plot iff step % 6 < 4
/// ```
#[inline]
fn circle_step_visible(fill: bool, style: u32, step: i32) -> bool {
    if fill {
        return true;
    }
    if (style & CIRCLE_SOLID) != 0 {
        return true;
    }
    (step % 6) < 4
}

/// Literal port of `FUN_005d0ec0` (GDI, 1107 bytes / 389 instructions,
/// cdecl, 9 args — `add esp, 0x24` at both call sites):
/// `(buf, x, y, l, t, r, b, colour, spans)`.
///
/// Plots the eight symmetric points of midpoint-circle step `(x, y)`
/// around the centre of the inner rect `(l, t)..(r, b)`, gated per point
/// on the ORIGINAL rect `[l-1, r+1] × [t-1, b+1]` (= `[x0, x1] × [y0, y1]`),
/// and — when `spans` is non-null — widens that row's `{min, max}` span
/// entry so the caller's span fill covers the interior.
///
/// The asm is two single-iteration loops (`for i in x..(x+1)` and
/// `for j in y..(y+1)`, the bounds truncated through `movsx ..., ax`)
/// each plotting four points with an asymmetric jump chain; the port keeps
/// the loop shape and the chain exactly.
#[allow(clippy::too_many_arguments)]
pub fn plot_circle_octants(
    s: &mut PackedSurface,
    x: i32,
    y: i32,
    l: i32,
    t: i32,
    r: i32,
    b: i32,
    colour: u16,
    spans: &mut Option<Vec<Span>>,
) {
    // 005d0ed6/005d0edd/005d0ee4: [esp+0x20] = (y + 1) as i16   (second-loop bound)
    let y_limit = (y.wrapping_add(1) as i16) as i32;
    // 005d0ed3/005d0eeb/005d0ef1: [esp+0x24] = (x + 1) as i16   (first-loop bound)
    let x_limit = (x.wrapping_add(1) as i16) as i32;
    // 005d0ee0..005d0efd: ebp = (r - l + 1) / 2 + l → [esp+0x10] (cx)
    let cx = r.wrapping_sub(l).wrapping_add(1) / 2 + l;
    // 005d0eff..005d0f11: edx = (b - t + 1) / 2 + t → [esp+0x18] (cy)
    let cy = b.wrapping_sub(t).wrapping_add(1) / 2 + t;
    // Bounds the four tests use: [l-1, r+1] × [t-1, b+1].
    let (lx, rx, ty, by) = (l - 1, r + 1, t - 1, b + 1);

    // ---- first pass: 005d0f13 `cmp ebx, ecx` (x vs x_limit); jge → skip ----
    let mut i = x;
    if i < x_limit {
        // 005d0f23..005d0f36: [esp+0x1c] = cy - y (py_u); [esp+0x4c] = cy + y (py_d);
        //                     esi = cx - x (px_l); edi = x + cx (px_r)
        let py_u = cy.wrapping_sub(y);
        let py_d = cy.wrapping_add(y);
        let mut px_l = cx.wrapping_sub(x);
        let mut px_r = cx.wrapping_add(x);
        loop {
            // A 005d0f3e..005d0fa8: (px_r, py_d) — all four bounds
            if py_d >= ty && py_d <= by && px_r >= lx && px_r <= rx {
                circle_point(s, spans, t, px_r, py_d, colour);
            }
            // B 005d0fb0..005d1010: (px_r, py_u)
            //   cmp eax, t-1 ; jl 005d1075  → D   (py_u < t-1)
            //   cmp eax, b+1 ; jg 005d1018  → C
            //   cmp edi, l-1 ; jl 005d1018  → C
            //   cmp edi, r+1 ; jg 005d1018  → C
            if py_u >= ty {
                if !(py_u > by || px_r < lx || px_r > rx) {
                    circle_point(s, spans, t, px_r, py_u, colour);
                }
                // C 005d1018..005d106d: (px_l, py_u) — NO `py_u >= t-1` test
                //   cmp eax, b+1 ; jg 005d1075 → D
                //   cmp esi, l-1 ; jl 005d1075 → D
                //   cmp esi, r+1 ; jg 005d1075 → D
                if !(py_u > by || px_l < lx || px_l > rx) {
                    circle_point(s, spans, t, px_l, py_u, colour);
                }
            }
            // D 005d1075..005d10e3: (px_l, py_d) — all four bounds
            if py_d >= ty && py_d <= by && px_l >= lx && px_l <= rx {
                circle_point(s, spans, t, px_l, py_d, colour);
            }
            // 005d10eb..005d10fc: i += 1; px_r += 1; px_l -= 1; jl while i < x_limit
            i += 1;
            px_r += 1;
            px_l -= 1;
            if i >= x_limit {
                break;
            }
        }
    }

    // ---- second pass: 005d110a..005d1114 `cmp esi(y), eax(y_limit)`; jge → ret ----
    let mut j = y;
    if j < y_limit {
        // 005d111a..005d1130: ebx = cy - x (py_u2); [esp+0x1c] = cy + x (py_d2);
        //                     esi = cx - y (px_l2); edi = y + cx (px_r2)
        let py_u2 = cy.wrapping_sub(x);
        let py_d2 = cy.wrapping_add(x);
        let mut px_l2 = cx.wrapping_sub(y);
        let mut px_r2 = cx.wrapping_add(y);
        loop {
            // E 005d1139..005d11a3: (px_r2, py_d2) — all four bounds
            if py_d2 >= ty && py_d2 <= by && px_r2 >= lx && px_r2 <= rx {
                circle_point(s, spans, t, px_r2, py_d2, colour);
            }
            // F 005d11b0..005d1212: (px_r2, py_u2)
            //   cmp ebx, t-1 ; jl 005d1281 → H
            //   cmp ebx, b+1 ; jg 005d121f → G ; cmp edi, l-1 ; jl → G ; cmp edi, r+1 ; jg → G
            if py_u2 >= ty {
                if !(py_u2 > by || px_r2 < lx || px_r2 > rx) {
                    circle_point(s, spans, t, px_r2, py_u2, colour);
                }
                // G 005d121f..005d1274: (px_l2, py_u2) — NO `py_u2 >= t-1` test
                //   cmp ebx, b+1 ; jg 005d1281 → H ; cmp esi, l-1 ; jl → H ; cmp esi, r+1 ; jg → H
                if !(py_u2 > by || px_l2 < lx || px_l2 > rx) {
                    circle_point(s, spans, t, px_l2, py_u2, colour);
                }
            }
            // H 005d1281..005d12eb: (px_l2, py_d2) — all four bounds
            if py_d2 >= ty && py_d2 <= by && px_l2 >= lx && px_l2 <= rx {
                circle_point(s, spans, t, px_l2, py_d2, colour);
            }
            // 005d12f4..005d1305: j += 1; px_r2 += 1; px_l2 -= 1; jl while j < y_limit
            j += 1;
            px_r2 += 1;
            px_l2 -= 1;
            if j >= y_limit {
                break;
            }
        }
    }
}

/// One accepted point of `FUN_005d0ec0`: the span-widen + pixel store that
/// every one of the eight branches ends in (point A's copy is
/// `005d0f62..005d0fa8`; the others are byte-identical modulo registers).
///
/// ```text
/// test ecx, ecx ; je STORE              ; spans == NULL → no span update
/// mov  eax, cy ; sub eax, t ; add edx, y ; (edx = py - t)
/// cmp  edi, [ecx + edx*8 + 8] ; jge .1  ; entry[(py - t) + 1].min
/// mov  [ecx + edx*8 + 8], edi
/// .1: cmp edi, [ecx + edx*8 + 0xc] ; jle .2 ; entry[(py - t) + 1].max
/// mov  [ecx + edx*8 + 0xc], edi
/// .2: lea edx, [edx + edx*4] ; lea eax, [edx + edx*4] ; shl eax, 5   ; py * 800
///     add eax, x ; add eax, cx ; mov [buf + eax*2], colour            ; + px
/// ```
///
/// The pixel store hardcodes the 800-pixel stride of the exe's 800×600
/// surface (`DAT_00acdeb8 == 800`) and performs NO clip against the
/// surface — the rect tests in the caller are the only gate. The port
/// uses `pitch_pixels` (== 800 on the game surface) and refuses only
/// out-of-buffer indices, which the exe would have written past the
/// allocation.
#[inline]
fn circle_point(
    s: &mut PackedSurface,
    spans: &mut Option<Vec<Span>>,
    t: i32,
    px: i32,
    py: i32,
    colour: u16,
) {
    if let Some(sp) = spans {
        // entry index = (py - t) + 1  (the +8 / +0xc displacements)
        let k = py.wrapping_sub(t).wrapping_add(1);
        if k >= 0 && (k as usize) < sp.len() {
            let e = &mut sp[k as usize];
            if px < e.min {
                e.min = px;
            }
            if px > e.max {
                e.max = px;
            }
        }
    }
    let idx = (s.pitch_pixels as i64) * (py as i64) + (px as i64);
    if idx >= 0 && (idx as usize) < s.buf.len() {
        s.buf[idx as usize] = colour;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_fill_writes_uniform_colour() {
        let mut s = PackedSurface::rgb555(6, 4);
        let c = s.pack_rgb(0xff, 0, 0);
        draw_panel(&mut s, 1, 1, 4, 2, P_SOLID_FILL, c, 0, PanelPalette::default());
        let z = 0u16;
        let expected: [u16; 24] = [
            z, z, z, z, z, z,
            z, c, c, c, c, z,
            z, c, c, c, c, z,
            z, z, z, z, z, z,
        ];
        assert_eq!(s.buf.as_slice(), &expected);
    }

    #[test]
    fn solid_frame_draws_border_only() {
        let mut s = PackedSurface::rgb555(6, 6);
        let c = s.pack_rgb(0, 0, 0xff);
        draw_panel(&mut s, 1, 1, 4, 4, P_BEVEL | P_SOLID_FRAME, c, 0, PanelPalette::default());
        // Border only; interior (2..=3, 2..=3) must be zero.
        for y in 2..=3 {
            for x in 2..=3 {
                assert_eq!(s.buf[(y * 6 + x) as usize], 0, "interior ({x},{y})");
            }
        }
        // All 4 corners of the frame are `c`.
        for &(x, y) in &[(1, 1), (4, 1), (1, 4), (4, 4)] {
            assert_eq!(s.buf[(y * 6 + x) as usize], c, "corner ({x},{y})");
        }
    }

    #[test]
    fn vgradient_first_and_last_row_scale_correctly() {
        // A V-gradient over rows 0..=9: local_54 starts 0, decreases by
        // 100 per row; scale = local_54/9 + 100 in 8-bit signed. Row 0
        // scale = 100 (unchanged), row 9 scale = 100 - 900/9 = 0 (black).
        let mut s = PackedSurface::rgb555(2, 10);
        let c = s.pack_rgb(0xff, 0, 0);
        draw_panel(&mut s, 0, 0, 1, 9, P_VGRADIENT, c, 0, PanelPalette::default());
        let top = s.buf[0];
        let bottom = s.buf[s.buf.len() - 1];
        assert_eq!(top, c, "top row must equal source colour (scale=100)");
        // Bottom row = scale_colour(c, 0) which zeroes the R channel.
        assert_eq!(bottom, scale_colour(&s, c, 0));
    }

    #[test]
    fn scale_colour_black_stays_black_and_100_pct_is_identity() {
        let s = PackedSurface::rgb555(1, 1);
        let c = s.pack_rgb(0x88, 0x44, 0x22);
        assert_eq!(scale_colour(&s, c, 100), c);
        assert_eq!(scale_colour(&s, 0, 50), 0);
        // 0% zeroes every channel.
        assert_eq!(scale_colour(&s, c, 0), 0);
    }

    #[test]
    fn outer_highlight_wraps_the_rect_by_one_pixel() {
        let mut s = PackedSurface::rgb555(8, 6);
        let c = s.pack_rgb(0, 0, 0xff);
        let hi = s.pack_rgb(0xff, 0xff, 0);
        draw_panel(
            &mut s,
            2, 2, 5, 4,
            P_SOLID_FILL | P_OUTER_HIGHLIGHT,
            c,
            0,
            PanelPalette { outer_highlight: hi, default_bevel: 0 },
        );
        // Interior filled with `c`.
        assert_eq!(s.buf[(2 * 8 + 2) as usize], c);
        // Corners of the outer highlight (one pixel outside on each side).
        for &(x, y) in &[(1, 1), (6, 1), (1, 5), (6, 5)] {
            assert_eq!(s.buf[(y * 8 + x) as usize], hi, "outer corner ({x},{y})");
        }
    }

    /// `FUN_005cdfa0`'s two panel stores on the RGB555 surface must equal
    /// the words read back live from the running exe (fixtures/verify_panel.json
    /// `palette`: outer_highlight=32736=0x7fe0, default_bevel=16=0x0010).
    #[test]
    fn palette_reload_matches_live_exe_globals() {
        let s555 = PackedSurface::rgb555(1, 1);
        let p = PanelPalette::from_palette_reload(&s555);
        assert_eq!(p.outer_highlight, 0x7fe0, "DAT_00ad6b24 on RGB555");
        assert_eq!(p.default_bevel, 0x0010, "DAT_00ad6b3c = pack_rgb(0,0,0x80) on RGB555");
        // 565 branch of 005ce080..005ce0a1: cx = 0xffe0; pack_rgb(0,0,0x80) = 0x0010.
        let s565 = PackedSurface::rgb565(1, 1);
        let p = PanelPalette::from_palette_reload(&s565);
        assert_eq!(p.outer_highlight, 0xffe0, "DAT_00ad6b24 on RGB565");
        assert_eq!(p.default_bevel, 0x0010, "DAT_00ad6b3c on RGB565");
    }

    /// Hand-traced `FUN_005d0ce0` solid outline on a 9×9 rect (0,0)..(8,8):
    /// l=1,t=1,r=7,b=7 → ih=6, y=(6+1)/2=3, d=3-6=-3, cx=cy=(7-1+1)/2+1=4.
    ///
    /// step0 (x=0,y=3): A(4,7) B(4,1) C(4,1) D(4,7) | E(7,4) F(7,4) G(1,4) H(1,4);
    ///                  d<0 → d=3; x=1.
    /// step1 (x=1,y=3): A(5,7) B(5,1) C(3,1) D(3,7) | E(7,5) F(7,3) G(1,3) H(1,5);
    ///                  d>=0 → d=3+(4*(1-3)+10)=5, y=2; x=2 → x<y fails.
    /// tail  (y=2)    : A(6,6) B(6,2) C(2,2) D(2,6) | E(6,6) F(6,2) G(2,2) H(2,6).
    #[test]
    fn circle_solid_outline_matches_hand_trace() {
        let mut s = PackedSurface::rgb555(9, 9);
        let c = s.pack_rgb(0, 0xff, 0);
        draw_circle(&mut s, 0, 0, 8, 8, CIRCLE_SOLID, c);
        let expected: [(i32, i32); 16] = [
            (4, 7), (4, 1), (7, 4), (1, 4),
            (5, 7), (5, 1), (3, 1), (3, 7), (7, 5), (7, 3), (1, 3), (1, 5),
            (6, 6), (6, 2), (2, 2), (2, 6),
        ];
        for y in 0..9 {
            for x in 0..9 {
                let want = if expected.contains(&(x, y)) { c } else { 0 };
                assert_eq!(s.buf[(y * 9 + x) as usize], want, "pixel ({x},{y})");
            }
        }
    }

    /// Same trace with bit 4: the span list widens per row to
    /// row1:[3,5] row2:[2,6] row3..5:[1,7] row6:[2,6] row7:[3,5]; rows 0 and 8
    /// keep `max == 0` and are skipped (`005d0e7c je`).
    #[test]
    fn circle_fill_matches_hand_traced_spans() {
        let mut s = PackedSurface::rgb555(9, 9);
        let c = s.pack_rgb(0xff, 0, 0);
        draw_circle(&mut s, 0, 0, 8, 8, CIRCLE_FILL, c);
        let spans: [(i32, i32, i32); 7] = [
            (1, 3, 5), (2, 2, 6), (3, 1, 7), (4, 1, 7), (5, 1, 7), (6, 2, 6), (7, 3, 5),
        ];
        for y in 0..9 {
            for x in 0..9 {
                let inside = spans.iter().any(|&(row, lo, hi)| row == y && x >= lo && x <= hi);
                let want = if inside { c } else { 0 };
                assert_eq!(s.buf[(y * 9 + x) as usize], want, "pixel ({x},{y})");
            }
        }
    }

    /// Dashed mode (`style` with neither bit 2 nor 4) drops every step whose
    /// `step % 6 >= 4`, so a radius large enough for ≥ 6 arc steps paints
    /// strictly fewer pixels than the solid arc over the same rect.
    #[test]
    fn circle_dashed_skips_steps_4_and_5_of_each_6() {
        let mut solid = PackedSurface::rgb555(40, 40);
        let mut dashed = PackedSurface::rgb555(40, 40);
        let c = solid.pack_rgb(0xff, 0xff, 0xff);
        draw_circle(&mut solid, 0, 0, 39, 39, CIRCLE_SOLID, c);
        draw_circle(&mut dashed, 0, 0, 39, 39, 0, c);
        let n_solid = solid.buf.iter().filter(|&&v| v == c).count();
        let n_dashed = dashed.buf.iter().filter(|&&v| v == c).count();
        assert!(n_dashed < n_solid, "dashed {n_dashed} must be < solid {n_solid}");
        // Every dashed pixel is also a solid pixel (same arc, subset of steps).
        for (i, &v) in dashed.buf.iter().enumerate() {
            if v == c {
                assert_eq!(solid.buf[i], c, "dashed pixel {i} not on the solid arc");
            }
        }
    }

    /// `P_BEVEL | P_ALT_PRIMITIVE` draws the outline in `param_7`, never in
    /// `colour` (005cf9db `mov edx, [esp+0x80]`), and `SOLID_FILL |
    /// ALT_PRIMITIVE` fills in `colour` (005cf652 `push eax` = [esp+0x7c]).
    #[test]
    fn panel_alt_primitive_routes_colours_per_callsite() {
        let mut s = PackedSurface::rgb555(9, 9);
        let fill = s.pack_rgb(0xff, 0, 0);
        let pat = s.pack_rgb(0, 0, 0xff);
        draw_panel(&mut s, 0, 0, 8, 8, P_SOLID_FILL | P_BEVEL | P_ALT_PRIMITIVE,
                   fill, pat, PanelPalette::default());
        // Outline point (4,1) is `pat`; interior (4,4) is `fill`.
        assert_eq!(s.buf[(1 * 9 + 4) as usize], pat, "outline in param_7");
        assert_eq!(s.buf[(4 * 9 + 4) as usize], fill, "disc in param_6");
        // Row 1 span [3,5]: (3,1) and (5,1) are outline points → `pat` on top of fill.
        assert_eq!(s.buf[(1 * 9 + 3) as usize], pat);
        assert_eq!(s.buf[(1 * 9 + 5) as usize], pat);
    }
}
