//! Save/restore-background pair — direct port of `FUN_005CDAC0` (save) and
//! `FUN_005CDCC0` (restore) from cm0102.exe.
//!
//! Decompile: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/005cdac0.c`,
//! `.../005cdcc0.c`.
//!
//! The exe's job is the classic "save the pixels behind a popup, draw the
//! popup, then put them back when it closes" pattern used throughout the
//! menu-heavy UI (dropdowns, tooltips, modal dialogs). In the original,
//! `FUN_005CDAC0` locks the back surface read-only (`DDLOCK` flag `0x11`),
//! copies the rect row-by-row into a `malloc`'d buffer (reusing a
//! previously-allocated same-size buffer when the caller supplies one — a
//! perf optimisation this port skips: Rust's `Vec` allocation is cheap
//! enough not to need it), and `FUN_005CDCC0` does the mirror image —
//! locks write-only (`0x21`) and copies the buffer back.
//!
//! `FUN_005CDCC0` is also reused generically elsewhere in the exe as a
//! plain "blit this raw pixel buffer to the surface" primitive (e.g. the
//! boot splash-logo sequence in `game.cpp::FUN_005B6F10` draws each
//! `.rgn` logo image through it) — that generic case is already covered
//! by [`crate::image::Image`]/[`crate::Surface::blit_image`]; this module
//! only covers the save/restore round-trip.

use crate::Surface;

/// The captured rectangle — the exe's `{w, h, size, dataPtr}` cache struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedBackground {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u16>, // RGB565, row-major
}

impl Surface {
    /// Direct port of `FUN_005CDAC0(x0, y0, x1, y1)` — capture the
    /// (inclusive) rect into a new buffer. Clips to surface bounds, like
    /// the exe's `FUN_005CD370` clip helper. Returns `None` for a
    /// degenerate (fully-clipped-away) rect, matching the exe's early
    /// return when the clip helper reports nothing to lock.
    pub fn save_background(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> Option<SavedBackground> {
        let cx0 = x0.max(0);
        let cy0 = y0.max(0);
        let cx1 = x1.min(self.w as i32 - 1);
        let cy1 = y1.min(self.h as i32 - 1);
        if cx1 < cx0 || cy1 < cy0 {
            return None;
        }
        let w = (cx1 - cx0 + 1) as usize;
        let h = (cy1 - cy0 + 1) as usize;
        let mut px = Vec::with_capacity(w * h);
        for y in cy0..=cy1 {
            let row = y as usize * self.w + cx0 as usize;
            px.extend_from_slice(&self.buf[row..row + w]);
        }
        Some(SavedBackground { w, h, px })
    }

    /// Direct port of `FUN_005CDCC0(x, y, saved)` — write a previously
    /// captured rect back at `(x, y)`. Clips to surface bounds; a `saved`
    /// wider/taller than what still fits is truncated, matching the
    /// exe's per-row bounded copy.
    pub fn restore_background(&mut self, x: i32, y: i32, saved: &SavedBackground) {
        for row in 0..saved.h {
            let dy = y + row as i32;
            if dy < 0 || dy as usize >= self.h {
                continue;
            }
            let src = &saved.px[row * saved.w..row * saved.w + saved.w];
            for (col, &p) in src.iter().enumerate() {
                let dx = x + col as i32;
                if dx < 0 || dx as usize >= self.w {
                    continue;
                }
                self.buf[dy as usize * self.w + dx as usize] = p;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack565;

    #[test]
    fn save_then_restore_round_trips_exactly() {
        let mut s = Surface::new();
        s.fill_rect(10, 10, 39, 29, (200, 50, 50));
        let saved = s.save_background(10, 10, 39, 29).unwrap();
        assert_eq!(saved.w, 30);
        assert_eq!(saved.h, 20);

        // Overwrite, then restore — should get the original pixels back.
        s.fill_rect(10, 10, 39, 29, (0, 0, 0));
        s.restore_background(10, 10, &saved);
        for y in 10..30 {
            for x in 10..40 {
                assert_eq!(s.get(x, y), pack565(200, 50, 50), "mismatch at ({x},{y})");
            }
        }
    }

    #[test]
    fn save_clips_to_surface_bounds() {
        let s = Surface::new();
        let saved = s.save_background(-5, -5, 10, 10).unwrap();
        // Clipped to (0,0)-(10,10) inclusive = 11x11.
        assert_eq!(saved.w, 11);
        assert_eq!(saved.h, 11);
    }

    #[test]
    fn save_fully_offscreen_rect_returns_none() {
        let s = Surface::new();
        assert!(s.save_background(-100, -100, -50, -50).is_none());
    }

    #[test]
    fn restore_clips_when_placed_near_edge() {
        let mut s = Surface::new();
        let saved = SavedBackground { w: 10, h: 10, px: vec![pack565(1, 2, 3); 100] };
        // Placed so half hangs off the right/bottom edge — should not panic,
        // and the in-bounds portion should still be written.
        let ox = s.w as i32 - 5;
        let oy = s.h as i32 - 5;
        s.restore_background(ox, oy, &saved);
        assert_eq!(s.get(ox, oy), pack565(1, 2, 3));
    }

    #[test]
    fn restore_offscreen_placement_writes_nothing() {
        let mut s = Surface::new();
        let before: Vec<u16> = s.buf.clone();
        let saved = SavedBackground { w: 5, h: 5, px: vec![pack565(9, 9, 9); 25] };
        s.restore_background(-100, -100, &saved);
        assert_eq!(s.buf, before);
    }
}
