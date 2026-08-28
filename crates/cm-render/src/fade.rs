//! Rect fade transition — direct port of `FUN_005CCDD0` from cm0102.exe.
//!
//! Decompile: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/005ccdd0.c`.
//!
//! Traced via its only two call sites (`functions.json` call graph) to
//! `game.cpp::FUN_005B6F10`, the boot splash-logo sequence: it draws each
//! publisher logo (`logo.rgn`/`eidos.rgn`/`kio.rgn`/`savechip.rgn`, all
//! loaded from the same `SI_DATA` resource container as the fonts) via
//! [`crate::Surface::blit_image`], then calls this function with `+0x46`
//! (70) to fade it in and `-0x32` (-50) to fade it out.
//!
//! Unlike every other primitive in this crate, the exe's function is not
//! a single stateless pixel operation — it's a small **blocking animation
//! loop**: it captures the rect's current pixels once, then for each of
//! `abs(fade_amount) + 1` steps it locks the surface, remaps every
//! captured pixel toward (or away from) black by that step's percentage,
//! unlocks, and calls the dirty-rect present routine (`FUN_005CCBA0`,
//! see [`crate::blit`]) before moving to the next step — i.e. `|fade_amount|`
//! is literally the animation's frame count, not a percentage. The sign
//! selects direction: positive ramps 0%→100% (fade in, revealing what's
//! already drawn); negative ramps 100%→0% (fade out, to black). This is
//! an inferred reconstruction of the exact ramp direction from the
//! calling convention (fade-in before the hold, fade-out after) rather
//! than a bit-for-bit trace of both decompiled branches — flag as
//! `INFERRED` if it needs re-verification later.
//!
//! Because Rust doesn't need the exe's per-pixel-value `malloc`'d blend
//! cache (that was a perf optimisation for 2001-era hardware; recomputing
//! per pixel is cheap here), this port skips it and just scales each
//! captured pixel directly per step — same visible result, simpler code.
//! The per-channel scale math matches [`crate::panel`]'s `dim_region`
//! (itself `FUN_005CDFD0`), which is the same "scale to pct% brightness"
//! operation applied to a fixed 60%.

use crate::{pack565, unpack565, Surface};

#[inline]
fn scale_pct(rgb: (u8, u8, u8), pct: i32) -> (u8, u8, u8) {
    let s = |c: u8| ((c as i32 * pct / 100).clamp(0, 255)) as u8;
    (s(rgb.0), s(rgb.1), s(rgb.2))
}

impl Surface {
    /// Direct port of `FUN_005CCDD0(x0, y0, x1, y1, fade_amount)`.
    ///
    /// Captures the rect once, then drives `abs(fade_amount) + 1` steps,
    /// writing each blended frame into `self` and invoking `present`
    /// after each one (standing in for the exe's per-frame `Unlock` +
    /// `FUN_005CCBA0` present call). No-ops if the rect is fully clipped
    /// away, matching the exe's early return when its clip helper finds
    /// nothing to lock.
    pub fn fade_rect(
        &mut self,
        x0: i32, y0: i32, x1: i32, y1: i32,
        fade_amount: i32,
        mut present: impl FnMut(&Surface),
    ) {
        let Some(captured) = self.save_background(x0, y0, x1, y1) else { return };
        let cx0 = x0.max(0);
        let cy0 = y0.max(0);
        let steps = fade_amount.unsigned_abs() as i32;
        let fade_in = fade_amount >= 0;

        for step in 0..=steps {
            let pct = if fade_in {
                step * 100 / steps.max(1)
            } else {
                100 - step * 100 / steps.max(1)
            };
            for row in 0..captured.h {
                let dy = cy0 + row as i32;
                let src_row = &captured.px[row * captured.w..row * captured.w + captured.w];
                for (col, &p) in src_row.iter().enumerate() {
                    let dx = cx0 + col as i32;
                    let (r, g, b) = unpack565(p);
                    let (r, g, b) = scale_pct((r, g, b), pct);
                    self.set(dx, dy, pack565(r, g, b));
                }
            }
            present(self);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack565;

    #[test]
    fn fade_in_ramps_from_black_to_full_color() {
        let mut s = Surface::new();
        s.fill_rect(0, 0, 9, 9, (200, 100, 40));
        let mut frames = Vec::new();
        s.fade_rect(0, 0, 9, 9, 4, |surf| frames.push(surf.get(0, 0)));

        assert_eq!(frames.len(), 5); // steps 0..=4
        assert_eq!(frames[0], pack565(0, 0, 0), "step 0 should be black");
        assert_eq!(frames[4], pack565(200, 100, 40), "final step should be full colour");
    }

    #[test]
    fn fade_out_ramps_from_full_color_to_black() {
        let mut s = Surface::new();
        s.fill_rect(0, 0, 9, 9, (200, 100, 40));
        let mut frames = Vec::new();
        s.fade_rect(0, 0, 9, 9, -4, |surf| frames.push(surf.get(0, 0)));

        assert_eq!(frames.len(), 5);
        assert_eq!(frames[0], pack565(200, 100, 40), "step 0 should be full colour");
        assert_eq!(frames[4], pack565(0, 0, 0), "final step should be black");
    }

    #[test]
    fn present_called_once_per_step_inclusive() {
        let mut s = Surface::new();
        s.fill_rect(0, 0, 4, 4, (10, 20, 30));
        let mut count = 0;
        s.fade_rect(0, 0, 4, 4, 10, |_| count += 1);
        assert_eq!(count, 11); // 0..=10
    }

    #[test]
    fn fully_offscreen_rect_calls_present_zero_times() {
        let mut s = Surface::new();
        let mut count = 0;
        s.fade_rect(-100, -100, -50, -50, 10, |_| count += 1);
        assert_eq!(count, 0);
    }

    #[test]
    fn zero_fade_amount_does_not_panic_and_yields_one_frame() {
        let mut s = Surface::new();
        s.fill_rect(0, 0, 4, 4, (10, 20, 30));
        let mut count = 0;
        s.fade_rect(0, 0, 4, 4, 0, |_| count += 1);
        assert_eq!(count, 1);
    }
}
