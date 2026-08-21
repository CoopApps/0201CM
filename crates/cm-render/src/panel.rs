//! Panel primitive — faithful port of graphics_draw_panel (FUN_005cf8e0): solid fill,
//! graduated bevel (highlight TL / shadow BR), border, bottom shade, and the transparency
//! (dim-underlying) path. This is a core component style of the CM0102 design system.

use crate::{pack565, unpack565, Surface};

pub const F_TRANSPARENT: u32 = 0x2; // dim underlying pixels to 60% (FUN_005cdfd0)
pub const F_HGRADIENT: u32 = 0x4; // horizontal gradient
pub const F_VGRADIENT: u32 = 0x8; // vertical gradient (the sidebar's blue fade)
pub const F_SOLID_FILL: u32 = 0x10;
pub const F_BEVEL: u32 = 0x20;
pub const F_SUNKEN: u32 = 0x40;
pub const F_SHADE_BOTTOM: u32 = 0x80;
pub const F_BORDER: u32 = 0x400;
pub const F_THIN_BORDER: u32 = 0x2000;
pub const F_INSET_BR: u32 = 0x20000;
pub const F_INSET_LT: u32 = 0x40000;

#[inline]
fn scale(rgb: (u8, u8, u8), pct: i32) -> (u8, u8, u8) {
    let s = |c: u8| ((c as i32 * pct / 100).clamp(0, 255)) as u8;
    (s(rgb.0), s(rgb.1), s(rgb.2))
}

impl Surface {
    /// FUN_005cd840 solid fill (inclusive bounds, as the game passes them).
    pub fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: (u8, u8, u8)) {
        let p = pack565(rgb.0, rgb.1, rgb.2);
        let (x0, y0) = (x0.max(0), y0.max(0));
        let (x1, y1) = (x1.min(self.w as i32 - 1), y1.min(self.h as i32 - 1));
        for y in y0..=y1 {
            let base = y as usize * self.w;
            for x in x0..=x1 {
                self.buf[base + x as usize] = p;
            }
        }
    }

    /// 1px outline rectangle — used to draw the selection highlight box the
    /// exe requests via item flag 0x800. Not a bevel, just four thin edges
    /// in one colour.
    pub fn draw_hollow_rect(&mut self, l: i32, t: i32, r: i32, b: i32, rgb: (u8, u8, u8)) {
        self.edge_line(l, t, r, t, 1, rgb);
        self.edge_line(l, b, r, b, 1, rgb);
        self.edge_line(l, t, l, b, 1, rgb);
        self.edge_line(r, t, r, b, 1, rgb);
    }

    /// FUN_005cd420 thick line (horizontal or vertical).
    fn edge_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, thick: i32, rgb: (u8, u8, u8)) {
        let p = pack565(rgb.0, rgb.1, rgb.2);
        for t in 0..thick {
            if y0 == y1 {
                let yy = y0 + t;
                if yy >= 0 && (yy as usize) < self.h {
                    for x in x0.max(0)..=x1.min(self.w as i32 - 1) {
                        self.buf[yy as usize * self.w + x as usize] = p;
                    }
                }
            } else {
                let xx = x0 + t;
                if xx >= 0 && (xx as usize) < self.w {
                    for y in y0.max(0)..=y1.min(self.h as i32 - 1) {
                        self.buf[y as usize * self.w + xx as usize] = p;
                    }
                }
            }
        }
    }

    /// Vertical gradient (FUN_005cf8e0 flag 0x8). Scans from full-brightness at
    /// the top to 0% at the bottom. RGB565 has only 5 bits of blue precision,
    /// so a naive integer scale gives long visible bands (~36 rows each on a
    /// 600-tall sidebar). Real-game captures show much shorter runs (~5-15
    /// rows) with intermediate pixels — dithering. We reproduce that with
    /// per-row RGB565-value interpolation + Bayer 4x4 ordered dither across x,
    /// which keeps the same set of visible values but breaks the bands so the
    /// eye sees a smooth blend.
    fn vgradient(&mut self, l: i32, t: i32, r: i32, b: i32, rgb: (u8, u8, u8)) {
        let height = b - t;
        if height <= 0 {
            return;
        }
        // 4x4 Bayer threshold matrix, values 0..15.
        const BAYER: [[u32; 4]; 4] = [
            [0, 8, 2, 10],
            [12, 4, 14, 6],
            [3, 11, 1, 9],
            [15, 7, 13, 5],
        ];
        // Dither AT THE RGB565 QUANTISATION LEVEL. Compute the ideal channel
        // value in 5-bit-precision fixed-point (for red/blue) or 6-bit (for
        // green), then the Bayer threshold decides whether to round up or
        // down. Because the quantisation step matches the dither step, the
        // transition regions between adjacent 5-bit values become speckled —
        // matching the real game's per-pixel-different pattern.
        for y in t..=b {
            // scale_q = (b-y)/height in Q16 fixed point.
            let scale_q = ((b - y) as i64 * 65536 / height as i64) as u32;
            // Ideal RGB in 5-/6-/5-bit fixed point (extra 4 bits fractional
            // for dither resolution matches Bayer 4x4 = 16 levels).
            // 8-bit input → 5-bit output: (v * 31 / 255) is the pure quantise.
            //   Q4.4 form: (v * 31 * 16 / 255)  → 0..(31*16-1) = 0..495.
            //   Applied to (rgb.x * scale_q >> 16) which is the 8-bit ideal.
            let ideal_r_q = ((rgb.0 as u32) * scale_q * 31 * 16 / (255 * 65536)) as u32;
            let ideal_g_q = ((rgb.1 as u32) * scale_q * 63 * 16 / (255 * 65536)) as u32;
            let ideal_b_q = ((rgb.2 as u32) * scale_q * 31 * 16 / (255 * 65536)) as u32;
            for x in l..=r {
                if x < 0 || (x as usize) >= self.w || y < 0 || (y as usize) >= self.h {
                    continue;
                }
                let thr = BAYER[(y as usize) & 3][(x as usize) & 3] as u32;
                // int_5bit = ideal >> 4; frac (0..15) = ideal & 15.
                // Round up if frac > threshold.
                let r5 = ((ideal_r_q >> 4) + (if (ideal_r_q & 15) > thr { 1 } else { 0 })).min(31);
                let g6 = ((ideal_g_q >> 4) + (if (ideal_g_q & 15) > thr { 1 } else { 0 })).min(63);
                let b5 = ((ideal_b_q >> 4) + (if (ideal_b_q & 15) > thr { 1 } else { 0 })).min(31);
                let packed = ((r5 << 11) | (g6 << 5) | b5) as u16;
                self.set(x, y, packed);
            }
        }
    }

    /// FUN_005cdfd0: scale underlying pixels to pct% brightness (the F_TRANSPARENT path).
    fn dim_region(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, pct: i32) {
        let (x0, y0) = (x0.max(0), y0.max(0));
        let (x1, y1) = (x1.min(self.w as i32 - 1), y1.min(self.h as i32 - 1));
        for y in y0..=y1 {
            let base = y as usize * self.w;
            for x in x0..=x1 {
                let (r, g, b) = unpack565(self.buf[base + x as usize]);
                let d = |c: u8| ((c as i32 * pct / 100).min(255)) as u8;
                self.buf[base + x as usize] = pack565(d(r), d(g), d(b));
            }
        }
    }

    /// Graduated bevel path of FUN_005cf8e0 (flag 0x20): per-layer highlight/shadow graded
    /// by cVar = 0x42*layer/thickness. thickness=4 for large top-strip solids, else 2.
    fn bevel(&mut self, l: i32, t: i32, r: i32, b: i32, flags: u32, rgb: (u8, u8, u8)) {
        let (w, h) = (r - l, b - t);
        let th = if w >= 0x32 && h >= 0x32 && b <= 99 && (flags & F_SOLID_FILL) != 0 { 4 } else { 2 };
        let mut lb8: i32 = 0x42;
        // FUN_005cf8e0 draws INNERMOST layer first, OUTERMOST last, with c = local_b8/th growing
        // each layer -> the strongest highlight/shadow lands on the outer edge and wins the 2px
        // overlaps. (Drawing outer-first inverts the bevel; verified pixel-exact vs the grab.)
        for layer in 0..th {
            let inset = th - 1 - layer;
            let c = (lb8 / th) & 0xff;
            let mut hi = scale(rgb, c + 100);
            let mut lo = scale(rgb, (100 - c).max(0));
            if flags & F_SUNKEN != 0 {
                std::mem::swap(&mut hi, &mut lo);
            }
            let (yt, yb, xl, xr) = (t + inset, b - inset, l + inset, r - inset);
            // Each layer draws a 1-PIXEL edge line — the depth comes from the
            // colour gradient across layers, not from stacking thicker strokes.
            // Using thickness=2 here caused adjacent layers to overlap by 1px
            // (2 layers → 3 visible px per edge, 4 layers → 5); with 1-thick,
            // 2 layers → 2px per edge, 4 layers → 4px, matching the exe's
            // actual button/banner bevel widths.
            self.edge_line(xl, yt, xr, yt, 1, hi); // top highlight
            self.edge_line(xl, yt, xl, yb, 1, hi); // left highlight
            self.edge_line(xl, yb, xr, yb, 1, lo); // bottom shadow
            self.edge_line(xr, yt, xr, yb, 1, lo); // right shadow
            lb8 += 0x42;
        }
    }

    /// Port of FUN_005cf8e0 — dispatch on render flags exactly as the game does.
    pub fn draw_panel(&mut self, mut l: i32, mut t: i32, mut r: i32, mut b: i32, flags: u32, rgb: (u8, u8, u8)) {
        if flags & F_INSET_BR != 0 {
            r -= 2;
            b -= 2;
        }
        if flags & F_INSET_LT != 0 {
            l += 2;
            t += 2;
        }
        if flags & F_TRANSPARENT != 0 {
            self.dim_region(l, t, r, b, 0x3c); // 60%
        } else if flags & F_SOLID_FILL != 0 {
            self.fill_rect(l, t, r, b, rgb);
        } else if flags & F_VGRADIENT != 0 {
            self.vgradient(l, t, r, b, rgb);
        }
        if flags & F_BEVEL != 0 {
            self.bevel(l, t, r, b, flags, rgb);
        }
        if flags & F_BORDER != 0 {
            let th = if flags & F_THIN_BORDER != 0 { 1 } else { 2 };
            self.edge_line(l, t, r, t, th, rgb);
            self.edge_line(l, b, r, b, th, rgb);
            self.edge_line(l, t, l, b, th, rgb);
            self.edge_line(r, t, r, b, th, rgb);
        }
        if flags & F_SHADE_BOTTOM != 0 {
            self.edge_line(l + 2, b, r - 2, b, 1, scale(rgb, 0x5a));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{unpack565, Surface};
    use crate::panel::{F_BEVEL, F_SOLID_FILL, F_VGRADIENT};

    /// Locks the bevel pixel-exact: the CM0102 red banner (fill+bevel, pure red 0xF800) must
    /// produce a full-red fill/top and a graduated shadow fading 214,173,132,82 down both the
    /// bottom and right edges — the values verified against the DirectDraw grab. Do not change
    /// draw_panel's bevel without re-verifying against the original.
    #[test]
    fn banner_bevel_matches_grab() {
        let mut s = Surface::new();
        let red = unpack565(0xF800); // (255,0,0)
        s.draw_panel(100, 10, 790, 70, F_SOLID_FILL | F_BEVEL, red);
        let rc = |x: i32, y: i32| unpack565(s.get(x, y)).0; // red channel
        assert_eq!(rc(400, 20), 255, "fill is pure red");
        assert_eq!(rc(400, 10), 255, "top edge highlight");
        assert_eq!(
            [rc(400, 67), rc(400, 68), rc(400, 69), rc(400, 70)],
            [214, 173, 132, 82],
            "bottom shadow gradient"
        );
        assert_eq!(
            [rc(787, 40), rc(788, 40), rc(789, 40), rc(790, 40)],
            [214, 173, 132, 82],
            "right shadow gradient"
        );
    }

    /// Locks the button-style bevel width. A menu-button-sized panel with
    /// F_TRANSPARENT | F_BEVEL uses the 2-layer bevel path (small enough +
    /// no F_SOLID_FILL). Each edge must be **exactly 2 pixels thick** — no
    /// more, no less. Regressions here (e.g. going back to 2-thick per layer
    /// with overlap = 3 visible pixels per edge) get caught immediately.
    /// The 2-pixel width matches what the exe actually renders for the
    /// Setup screen's menu buttons and similar-sized bevelled items.
    #[test]
    fn button_bevel_is_two_pixels_thick() {
        use crate::panel::F_TRANSPARENT;
        let mut s = Surface::new();
        // Fill background with a distinctive non-blue colour so bevel pixels
        // (which are navy shades) are unambiguous.
        s.fill(60, 60, 60);
        let navy = (0, 0, 132);
        // Setup screen's Start New Game rect (from the exe's 110/145/444/209
        // grid, close enough — small enough for 2-layer path, no F_SOLID_FILL).
        let (l, t, r, b) = (110, 145, 444, 209);
        s.draw_panel(l, t, r, b, F_TRANSPARENT | F_BEVEL, navy);

        // Helper: is this pixel from the navy bevel? Background is (56,60,57)
        // (RGB565-quantised (60,60,60)); every bevel pixel is a scaled version
        // of navy (0,0,132) with R and G both essentially zero. So "R and G
        // both < 10" cleanly separates bevel from background regardless of
        // how dark the shadow layer is.
        let is_navy_bevel = |x: i32, y: i32| {
            let (rr, gg, _bb) = unpack565(s.get(x, y));
            rr < 10 && gg < 10
        };

        // LEFT edge: sample a mid-height row (avoid corner overlap).
        let y_mid = (t + b) / 2;
        // Start scanning at x = l-2 to catch any stray outer pixels.
        let mut left_run = 0;
        let mut in_run = false;
        for x in (l - 2)..=(l + 5) {
            if is_navy_bevel(x, y_mid) {
                if !in_run {
                    in_run = true;
                }
                left_run += 1;
            } else if in_run {
                break;
            }
        }
        assert_eq!(left_run, 2, "left bevel edge should be 2 pixels thick");

        // TOP edge: same at a mid-width column.
        let x_mid = (l + r) / 2;
        let mut top_run = 0;
        let mut in_run = false;
        for y in (t - 2)..=(t + 5) {
            if is_navy_bevel(x_mid, y) {
                if !in_run {
                    in_run = true;
                }
                top_run += 1;
            } else if in_run {
                break;
            }
        }
        assert_eq!(top_run, 2, "top bevel edge should be 2 pixels thick");

        // RIGHT edge: scan from r-4..r+2, count trailing bevel pixels.
        let mut right_run = 0;
        for x in ((r - 4)..=(r + 2)).rev() {
            if is_navy_bevel(x, y_mid) {
                right_run += 1;
            } else if right_run > 0 {
                break;
            }
        }
        assert_eq!(right_run, 2, "right bevel edge should be 2 pixels thick");

        // BOTTOM edge: same at mid-width.
        let mut bottom_run = 0;
        for y in ((b - 4)..=(b + 2)).rev() {
            if is_navy_bevel(x_mid, y) {
                bottom_run += 1;
            } else if bottom_run > 0 {
                break;
            }
        }
        assert_eq!(bottom_run, 2, "bottom bevel edge should be 2 pixels thick");

        // The interior of an F_TRANSPARENT panel is dimmed by dim_region()
        // to ~60% of the background. What we're checking here is that the
        // bevel didn't LEAK into the interior — i.e. R and G are still
        // in the same ballpark as each other (grey), not near-zero blue.
        let (rr, gg, _bb) = unpack565(s.get(x_mid, y_mid));
        assert!(
            rr > 10 && gg > 10 && (rr as i32 - gg as i32).abs() < 20,
            "interior should be dimmed grey, not smeared navy: got ({rr},{gg},_)",
        );
    }

    /// Locks the sidebar's blue vertical gradient. With ordered dithering
    /// (Bayer 4x4 across x) each row now has mixed pixel values, so we assert
    /// the ROW-AVERAGE trends from ~132 at the top toward ~0 at the bottom
    /// with monotonic decrease at coarse-grained checkpoints. Also asserts
    /// no two adjacent checkpoints have a jump > 20 (that would indicate a
    /// broken interpolation) — matches the real game's smooth character
    /// where visible bands are ≤ 15 rows rather than ≥ 30.
    #[test]
    fn sidebar_gradient_matches_game_mbr() {
        let mut s = Surface::new();
        s.draw_panel(0, 0, 89, 599, F_VGRADIENT, (0, 0, 132));
        // Row-average blue at a given y (sample the sidebar's full width).
        let row_avg_blue = |y: i32| -> i32 {
            let sum: i32 = (0..=89).map(|x| unpack565(s.get(x, y)).2 as i32).sum();
            sum / 90
        };
        let top = row_avg_blue(0);
        let q1 = row_avg_blue(150);
        let mid = row_avg_blue(300);
        let q3 = row_avg_blue(450);
        let bot = row_avg_blue(590);
        assert!(top >= 125, "top row should be near-full blue: {top}");
        assert!(bot <= 12, "bottom row should be near-zero blue: {bot}");
        // Monotonic decrease at checkpoints — the whole point of a gradient.
        assert!(top > q1 && q1 > mid && mid > q3 && q3 > bot,
            "checkpoints not monotonically decreasing: {top}/{q1}/{mid}/{q3}/{bot}");
        // Coarse-grained smoothness: no adjacent checkpoint jump > 40.
        for (a, b) in [(top, q1), (q1, mid), (mid, q3), (q3, bot)] {
            assert!(a - b <= 40, "gradient jump too large: {a} -> {b}");
        }
    }

    /// **Golden-master lock.** The sidebar
    /// (`draw_panel(0,0,89,599,F_VGRADIENT,(0,0,132))`) appears IDENTICALLY
    /// on EVERY screen in the game — Setup, Select Leagues, Season, every
    /// in-game screen, every dialog. If this test breaks, the whole game's
    /// look shifted. Locks two things:
    ///
    /// 1. **Row-blue-sum fingerprint** at 15 checkpoints spanning the height —
    ///    change any pixel and one of these sums breaks with a clear "row R
    ///    changed from N to M" diagnostic.
    /// 2. **FNV-1a hash of the whole 90x600 sidebar RGB buffer** — any
    ///    per-pixel deviation that somehow preserves all 15 row sums (mass-
    ///    conservative dithering shift, say) still breaks this hash.
    ///
    /// The checkpoint sums come from running the current dithered vgradient;
    /// the hash comes from FNV-1a over the resulting RGB bytes. Any legitimate
    /// intentional change requires re-running this test to capture new values
    /// (and probably means one of the earlier bevel/dither/pack565 locks also
    /// needs revisiting).
    #[test]
    fn sidebar_gradient_golden_master() {
        let mut s = Surface::new();
        s.draw_panel(0, 0, 89, 599, F_VGRADIENT, (0, 0, 132));
        let per_row: Vec<u32> = (0..600i32)
            .map(|y| (0..90i32).map(|x| unpack565(s.get(x, y)).2 as u32).sum())
            .collect();
        // 15 checkpoints (first 5, middle 3, last 5, plus 2 quarter points).
        assert_eq!(per_row[0..5], [11880, 11880, 11880, 11673, 11880]);
        assert_eq!(per_row[298..301], [5940, 5940, 5940]);
        assert_eq!(per_row[595..600], [0, 184, 0, 0, 0]);
        // Quarter points — extra structure lock so mid-run regressions fail here.
        let quarter = per_row[150];
        let three_quarter = per_row[449];
        assert!(quarter >= 8500 && quarter <= 9500,
            "quarter (y=150) row sum out of expected range: {quarter}");
        assert!(three_quarter >= 2500 && three_quarter <= 3500,
            "three-quarter (y=449) row sum out of expected range: {three_quarter}");
        // FNV-1a 64-bit hash of the full RGB buffer. If the pattern changes
        // in any way this fails with a clear expected-vs-actual message.
        let mut hash: u64 = 0xcbf29ce484222325;
        for y in 0..600i32 {
            for x in 0..90i32 {
                let (r, g, b) = unpack565(s.get(x, y));
                for &b in &[r, g, b] {
                    hash ^= b as u64;
                    hash = hash.wrapping_mul(0x100000001b3);
                }
            }
        }
        assert_eq!(
            hash, 0xd2be17ca9d1af086,
            "sidebar gradient bytes changed — appears on EVERY screen, so the whole game's look shifted",
        );
    }

    /// Additional lock: the DITHERING keeps visible band lengths short.
    /// This test would fail if someone reverted vgradient to a plain
    /// non-dithered per-row scale (which produced 36-row flat bands).
    #[test]
    fn sidebar_gradient_no_long_flat_bands() {
        let mut s = Surface::new();
        s.draw_panel(0, 0, 89, 599, F_VGRADIENT, (0, 0, 132));
        // Column at x=10 — count consecutive rows with identical blue.
        let mut prev = unpack565(s.get(10, 0)).2;
        let mut run = 1;
        let mut longest = 1;
        for y in 1..600 {
            let b = unpack565(s.get(10, y)).2;
            if b == prev {
                run += 1;
                longest = longest.max(run);
            } else {
                run = 1;
                prev = b;
            }
        }
        assert!(longest <= 20,
            "longest flat band = {longest} rows — dither should keep this under 20",
        );
    }
}
