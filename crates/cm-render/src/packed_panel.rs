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
//! | 0x0001_0000|`_ALT_PRIMITIVE`     | frame/fill via `FUN_005d0ce0` (pattern) — deferred  |
//! | 0x0002_0000|`SHRINK_2`           | `right -= 2; bottom -= 2` before drawing            |
//! | 0x0004_0000|`SHIFT_2`            | `left += 2; top += 2` before drawing                |
//! | 0x0100_0000|`MIDLINE_H`          | horizontal separator — two 1-px lines at mid-y      |
//!
//! `FUN_005d0ce0` (the "pattern" alternate) and the futuristic-font
//! interactions are deferred; every other path is a byte-for-byte port.

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

/// The `DAT_00ad6b24` outer-highlight colour used when `P_OUTER_HIGHLIGHT`
/// is set. The exe caches this from a global palette entry
/// (`palette_reload` `FUN_005cdfa0`); until we wire that palette in, the
/// caller must pass it explicitly via `outer_highlight_colour`.
///
/// The `DAT_00ad6b3c` "default bevel colour" (used when `colour == 0`
/// on some bevel paths) is likewise deferred.
#[derive(Debug, Clone, Copy)]
pub struct PanelPalette {
    pub outer_highlight: u16,
    pub default_bevel: u16,
}

impl Default for PanelPalette {
    fn default() -> Self {
        Self { outer_highlight: 0, default_bevel: 0 }
    }
}

/// Literal port of `FUN_005cf570` (3617 bytes). `x0..=x1, y0..=y1` is the
/// inclusive rect; `style` carries the P_* bits above; `colour` is the
/// primary colour (`param_6`); `pattern_colour` is `param_7` (only used
/// on the `P_ALT_PRIMITIVE` / bevel paths).
pub fn draw_panel(
    s: &mut PackedSurface,
    x0: i32,
    y0: i32,
    mut x1: i32,
    mut y1: i32,
    style: u32,
    mut colour: u16,
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
            // Pattern fill via FUN_005d0ce0 — deferred; no-op stub.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_fill_writes_uniform_colour() {
        let mut s = PackedSurface::rgb555(6, 4);
        let c = s.pack_rgb(0xff, 0, 0);
        draw_panel(&mut s, 1, 1, 4, 2, P_SOLID_FILL, c, PanelPalette::default());
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
        draw_panel(&mut s, 1, 1, 4, 4, P_BEVEL | P_SOLID_FRAME, c, PanelPalette::default());
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
        draw_panel(&mut s, 0, 0, 1, 9, P_VGRADIENT, c, PanelPalette::default());
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
            PanelPalette { outer_highlight: hi, default_bevel: 0 },
        );
        // Interior filled with `c`.
        assert_eq!(s.buf[(2 * 8 + 2) as usize], c);
        // Corners of the outer highlight (one pixel outside on each side).
        for &(x, y) in &[(1, 1), (6, 1), (1, 5), (6, 5)] {
            assert_eq!(s.buf[(y * 8 + x) as usize], hi, "outer corner ({x},{y})");
        }
    }
}
