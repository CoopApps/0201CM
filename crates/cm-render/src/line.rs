//! Line drawer — direct port of `FUN_005CD420` (1,049 bytes decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_glyph_blit/0x005cd420.c`.
//!
//! The exe uses this one function for every line: the 4 outline sides of
//! [`crate::primitives::fill_rect`], the per-column and per-row
//! gradients inside [`crate::bevel::bevel_box`], the 3-D bevel inset
//! frames, the middle line dividers.
//!
//! # Signature
//!
//! ```pseudo
//! void draw_line(int x0, int y0, int x1, int y1, uint op_mode, u16 color)
//! ```
//!
//! * `op_mode & 2` (`OP_SOLID`) — draw every pixel.
//! * else — dashed 4-on / 2-off (period 6): pixel written iff
//!   `(i - clip_start) % 6 < 4`.
//!
//! # Three paths
//!
//! 1. Horizontal (`y0 == y1`): Lock, walk from `clip_x0..clip_x1`.
//! 2. Vertical (`x0 == x1`): step by `pitch/2` per row.
//! 3. Bresenham (diagonal): integer Bresenham; the `dx < dy` branch
//!    iterates y (steep), else x (shallow). Both apply the same
//!    dashed-pattern gate.

use crate::Surface;

/// Line op-mode bits.
pub const OP_SOLID: u32 = 0x2;

/// Direct port of `FUN_005CD420(x0, y0, x1, y1, op_mode, color)`.
///
/// Writes into `surface`, matching the exe's Lock-write-Unlock cycle.
/// Clips to surface bounds internally (matches `FUN_005CD370` clip).
pub fn draw_line(
    surface: &mut Surface,
    x0: i32, y0: i32, x1: i32, y1: i32,
    op_mode: u32,
    color: u16,
) {
    let solid = op_mode & OP_SOLID != 0;
    let w = surface.w as i32;
    let h = surface.h as i32;

    if y0 == y1 {
        // Horizontal line.
        if y0 < 0 || y0 >= h { return; }
        let (mut a, mut b) = if x1 < x0 { (x1, x0) } else { (x0, x1) };
        a = a.max(0);
        b = b.min(w - 1);
        if b < a { return; }
        for x in a..=b {
            let i = (x - a) as u32;
            if solid || (i % 6) < 4 {
                surface.set(x, y0, color);
            }
        }
        return;
    }

    if x0 == x1 {
        // Vertical line.
        if x0 < 0 || x0 >= w { return; }
        let (mut a, mut b) = if y1 < y0 { (y1, y0) } else { (y0, y1) };
        a = a.max(0);
        b = b.min(h - 1);
        if b < a { return; }
        for y in a..=b {
            let i = (y - a) as u32;
            if solid || (i % 6) < 4 {
                surface.set(x0, y, color);
            }
        }
        return;
    }

    // Bresenham — the exe branches on |dx| vs |dy|.
    let dx_abs = (x1 - x0).unsigned_abs() as i32;
    let dy_abs = (y1 - y0).unsigned_abs() as i32;
    if dx_abs < dy_abs {
        // Steep — iterate over y.
        let (mut sx, mut sy, ex, ey) = if y1 < y0 { (x1, y1, x0, y0) }
                                        else       { (x0, y0, x1, y1) };
        // step direction for x.
        let step_x = if ex >= sx { 1 } else { -1 };
        let d_abs_x = (ex - sx).abs() * 2;
        let corr = (d_abs_x - (ey - sy)) * 2;
        let mut err = d_abs_x - (ey - sy);
        let mut i = 0u32;
        if sx >= 0 && sx < w && sy >= 0 && sy < h {
            surface.set(sx, sy, color);
            i += 1;
        }
        sy += 1;
        while sy <= ey {
            let bump = if err >= 0 { sx += step_x; corr } else { d_abs_x };
            err += bump;
            if sx >= 0 && sx < w && sy >= 0 && sy < h {
                if solid || (i % 6) < 4 { surface.set(sx, sy, color); }
                i += 1;
            }
            sy += 1;
        }
    } else {
        // Shallow — iterate over x.
        let (mut sx, mut sy, ex, ey) = if x1 < x0 { (x1, y1, x0, y0) }
                                        else       { (x0, y0, x1, y1) };
        let step_y = if ey >= sy { 1 } else { -1 };
        let d_abs_y = (ey - sy).abs() * 2;
        let corr = (d_abs_y - (ex - sx)) * 2;
        let mut err = d_abs_y - (ex - sx);
        let mut i = 0u32;
        if sx >= 0 && sx < w && sy >= 0 && sy < h {
            surface.set(sx, sy, color);
            i += 1;
        }
        sx += 1;
        while sx <= ex {
            let bump = if err >= 0 { sy += step_y; corr } else { d_abs_y };
            err += bump;
            if sx >= 0 && sx < w && sy >= 0 && sy < h {
                if solid || (i % 6) < 4 { surface.set(sx, sy, color); }
                i += 1;
            }
            sx += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_pixels(s: &Surface, want: u16) -> usize {
        s.buf.iter().filter(|&&p| p == want).count()
    }

    #[test]
    fn horizontal_solid_line_fills_all_pixels() {
        let mut s = Surface::new();
        draw_line(&mut s, 10, 50, 50, 50, OP_SOLID, 0xF800);
        assert_eq!(count_pixels(&s, 0xF800), 41);
    }

    #[test]
    fn horizontal_dashed_pattern_is_4_on_2_off() {
        let mut s = Surface::new();
        draw_line(&mut s, 0, 0, 11, 0, 0, 0x001F);
        // 12 pixels: on/on/on/on/off/off/on/on/on/on/off/off = 8 on.
        assert_eq!(count_pixels(&s, 0x001F), 8);
    }

    #[test]
    fn vertical_solid_line_fills_all_pixels() {
        let mut s = Surface::new();
        draw_line(&mut s, 100, 20, 100, 60, OP_SOLID, 0x07E0);
        assert_eq!(count_pixels(&s, 0x07E0), 41);
    }

    #[test]
    fn vertical_dashed_pattern_matches_horizontal() {
        let mut s = Surface::new();
        draw_line(&mut s, 100, 0, 100, 11, 0, 0x07E0);
        assert_eq!(count_pixels(&s, 0x07E0), 8);
    }

    #[test]
    fn diagonal_shallow_line_touches_expected_pixels() {
        let mut s = Surface::new();
        // Shallow: dx=10, dy=5 → iterate over x.
        draw_line(&mut s, 0, 0, 10, 5, OP_SOLID, 0xFFFF);
        // Bresenham should light every x from 0..=10 (11 pixels).
        assert_eq!(count_pixels(&s, 0xFFFF), 11);
    }

    #[test]
    fn diagonal_steep_line_touches_expected_pixels() {
        let mut s = Surface::new();
        draw_line(&mut s, 0, 0, 5, 10, OP_SOLID, 0xFFFF);
        // Steep: iterate over y from 0..=10 (11 pixels).
        assert_eq!(count_pixels(&s, 0xFFFF), 11);
    }

    #[test]
    fn line_out_of_bounds_writes_nothing() {
        let mut s = Surface::new();
        draw_line(&mut s, 900, 700, 950, 750, OP_SOLID, 0xFFFF);
        assert_eq!(count_pixels(&s, 0xFFFF), 0);
    }

    #[test]
    fn line_clips_at_surface_edges() {
        let mut s = Surface::new();
        // Overshoots right edge — should clip cleanly.
        draw_line(&mut s, 790, 50, 810, 50, OP_SOLID, 0xF800);
        // Should write x=790..=799 = 10 pixels.
        assert_eq!(count_pixels(&s, 0xF800), 10);
    }

    #[test]
    fn reversed_endpoints_swap_correctly() {
        let mut s = Surface::new();
        draw_line(&mut s, 50, 30, 10, 30, OP_SOLID, 0xF800);
        // 41 pixels regardless of endpoint order.
        assert_eq!(count_pixels(&s, 0xF800), 41);
    }
}
