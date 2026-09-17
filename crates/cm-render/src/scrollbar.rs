//! The canonical GDI list scrollbar — geometry, thumb maths, hit
//! testing and drawing, in ONE place.
//!
//! Every list screen must use this. Hand-rolling a scrollbar per screen
//! is what produced a thumb that never moved.
//!
//! # Where the numbers come from
//!
//! Two independent captures of the original executable, which agree
//! exactly:
//!
//! ```text
//! club_squad_screen/structure.txt     club_screen/structure.txt
//!   up    PANEL (759,198)-(778,217)     up    PANEL (759,153)-(778,172)
//!   track PANEL (759,218)-(778,472)     track PANEL (759,173)-(778,507)
//!         darken(759,218)-(778,472)           darken(759,173)-(778,507)
//!   thumb PANEL (759,250)-(778,472)     thumb PANEL (759,173)-(778,271)
//!   down  PANEL (759,473)-(778,492)     down  PANEL (759,508)-(778,527)
//! ```
//!
//! From which:
//!
//! * **x spans 759..=778** — 20 px wide. With one pixel of gap either
//!   side that is the `0x15` (21 px) gutter the layout engine reserves
//!   (`reports/gui_layout_engine_decode.md` §1, `inner_w -= 0x15`).
//! * **Arrow buttons are 20 px tall**, one at each end of the bar.
//! * **The track runs arrow-to-arrow**: `top+20 ..= bottom-20`.
//!   Confirmed on both captures.
//! * Arrows and thumb are drawn identically — `PANEL` with
//!   `P_SOLID_FILL | P_BEVEL` in `GREY_BAR` (`c=0x4210`), plus the
//!   bevel `rect s=4`. The track is a `P_DARKEN` panel.
//!
//! # Thumb maths (verified to the pixel)
//!
//! ```text
//! thumb_len = track_len * visible / total
//! thumb_top = track_y0 + (track_len - thumb_len) * scroll / (total - visible)
//! ```
//!
//! Checked against the squad capture: track 218..=472 (len 255), 28 of
//! 32 rows visible, scrolled to the bottom →
//! `255 * 28 / 32 = 223` (observed 223) and
//! `218 + (255 - 223) = 250` (observed 250). Exact.
//!
//! Checked against the club-select capture: track 173..=507 (len 335),
//! 28 visible, scroll 0 → `335 * 28 / 94 = 99` (observed 99).
//!
//! # UNKNOWN / UNCAPTURED
//!
//! * The **minimum thumb length** has not been observed — no capture
//!   shows a list long enough to drive the thumb below 20 px. We clamp
//!   at 20 (the arrow height) as the least-surprising choice. Mark this
//!   resolved only when a capture shows otherwise.
//! * Arrow **auto-repeat** on press-and-hold is not captured.
//! * Whether the GDI build responds to the **mouse wheel** at all is not
//!   captured.

use crate::packed::PackedSurface;
use crate::packed_panel::{draw_panel, PanelPalette, P_BEVEL, P_DARKEN, P_SOLID_FILL};

/// Bar width in pixels (759..=778 inclusive).
pub const WIDTH: i32 = 20;
/// Arrow button height, both ends.
pub const ARROW: i32 = 20;
/// Left edge used by every captured list screen.
pub const X0: i32 = 759;
/// Right edge used by every captured list screen.
pub const X1: i32 = 778;

/// Minimum thumb length. **Assumption, not captured** — see module docs.
pub const MIN_THUMB: i32 = 20;

/// What the pointer landed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    /// The up arrow — scroll one row back.
    Up,
    /// The down arrow — scroll one row on.
    Down,
    /// Track above the thumb — page back.
    PageUp,
    /// Track below the thumb — page on.
    PageDown,
    /// The thumb itself. `grab` is the offset from the thumb's top to
    /// the click, so dragging does not jump the thumb under the cursor.
    Thumb { grab: i32 },
}

/// A scrollbar occupying the full height of a list body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scrollbar {
    pub x0: i32,
    pub x1: i32,
    /// Top of the up arrow.
    pub top: i32,
    /// Bottom of the down arrow.
    pub bottom: i32,
}

impl Scrollbar {
    /// A bar spanning `top..=bottom` at the captured x position.
    pub const fn new(top: i32, bottom: i32) -> Self {
        Self { x0: X0, x1: X1, top, bottom }
    }

    /// The squad / fixtures / transfers list bar — capture
    /// `club_squad_screen`.
    pub const SQUAD: Scrollbar = Scrollbar::new(198, 492);
    /// The club-select list bar — capture `club_screen`.
    pub const CLUB_SELECT: Scrollbar = Scrollbar::new(153, 527);

    pub const fn up_arrow(&self) -> (i32, i32) { (self.top, self.top + ARROW - 1) }
    pub const fn down_arrow(&self) -> (i32, i32) { (self.bottom - ARROW + 1, self.bottom) }
    pub const fn track(&self) -> (i32, i32) { (self.top + ARROW, self.bottom - ARROW) }

    fn track_len(&self) -> i32 {
        let (a, b) = self.track();
        (b - a + 1).max(1)
    }

    /// Thumb extent for a list state, or `None` when everything fits
    /// (the exe draws no thumb then — the caller should also skip the
    /// whole bar).
    pub fn thumb(&self, total: usize, visible: usize, scroll: usize) -> Option<(i32, i32)> {
        if total <= visible || visible == 0 {
            return None;
        }
        let (ty0, _) = self.track();
        let track_len = self.track_len();
        let len = ((track_len as i64 * visible as i64) / total as i64) as i32;
        let len = len.clamp(MIN_THUMB.min(track_len), track_len);
        let max_scroll = (total - visible) as i64;
        let scroll = (scroll as i64).min(max_scroll);
        let span = (track_len - len) as i64;
        let top = ty0 + ((span * scroll) / max_scroll.max(1)) as i32;
        Some((top, top + len - 1))
    }

    /// Which part of the bar a point falls on. `None` if outside.
    pub fn hit(&self, x: i32, y: i32, total: usize, visible: usize, scroll: usize) -> Option<Hit> {
        if x < self.x0 || x > self.x1 || y < self.top || y > self.bottom {
            return None;
        }
        let (ua0, ua1) = self.up_arrow();
        if y >= ua0 && y <= ua1 {
            return Some(Hit::Up);
        }
        let (da0, da1) = self.down_arrow();
        if y >= da0 && y <= da1 {
            return Some(Hit::Down);
        }
        match self.thumb(total, visible, scroll) {
            Some((t0, t1)) if y >= t0 && y <= t1 => Some(Hit::Thumb { grab: y - t0 }),
            Some((t0, _)) if y < t0 => Some(Hit::PageUp),
            Some(_) => Some(Hit::PageDown),
            None => None,
        }
    }

    /// The scroll position that puts the thumb's top at `thumb_top` —
    /// the inverse of [`Self::thumb`], for dragging.
    pub fn scroll_for_thumb_top(&self, thumb_top: i32, total: usize, visible: usize) -> usize {
        if total <= visible || visible == 0 {
            return 0;
        }
        let (ty0, _) = self.track();
        let track_len = self.track_len();
        let len = ((track_len as i64 * visible as i64) / total as i64) as i32;
        let len = len.clamp(MIN_THUMB.min(track_len), track_len);
        let span = (track_len - len).max(1) as i64;
        let max_scroll = (total - visible) as i64;
        let off = (thumb_top - ty0).clamp(0, span as i32) as i64;
        (((off * max_scroll) + span / 2) / span).clamp(0, max_scroll) as usize
    }

    /// Paint the bar. Draws nothing when the content fits, matching the
    /// exe (no bar on a short list).
    pub fn draw(
        &self,
        surface: &mut PackedSurface,
        total: usize,
        visible: usize,
        scroll: usize,
        grey: u16,
        palette: PanelPalette,
    ) {
        let Some((thumb_y0, thumb_y1)) = self.thumb(total, visible, scroll) else {
            return;
        };
        let (ua0, ua1) = self.up_arrow();
        let (da0, da1) = self.down_arrow();
        let (ty0, ty1) = self.track();
        // Draw order matches the capture: up arrow, track, thumb, down
        // arrow. The thumb sits ON the darkened track, so the track must
        // be painted first.
        draw_panel(surface, self.x0, ua0, self.x1, ua1,
            P_SOLID_FILL | P_BEVEL, grey, 0, palette);
        draw_panel(surface, self.x0, ty0, self.x1, ty1, P_DARKEN, 0, 0, palette);
        draw_panel(surface, self.x0, thumb_y0, self.x1, thumb_y1,
            P_SOLID_FILL | P_BEVEL, grey, 0, palette);
        draw_panel(surface, self.x0, da0, self.x1, da1,
            P_SOLID_FILL | P_BEVEL, grey, 0, palette);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Geometry must reproduce both captures exactly.
    #[test]
    fn geometry_matches_both_exe_captures() {
        // club_squad_screen
        let sb = Scrollbar::SQUAD;
        assert_eq!(sb.up_arrow(), (198, 217));
        assert_eq!(sb.track(), (218, 472));
        assert_eq!(sb.down_arrow(), (473, 492));
        assert_eq!((sb.x0, sb.x1), (759, 778));

        // club_screen
        let sb = Scrollbar::CLUB_SELECT;
        assert_eq!(sb.up_arrow(), (153, 172));
        assert_eq!(sb.track(), (173, 507));
        assert_eq!(sb.down_arrow(), (508, 527));
    }

    /// The squad capture shows 28 of 32 rows, scrolled to the bottom,
    /// with the thumb at exactly (250, 472).
    #[test]
    fn thumb_reproduces_the_squad_capture() {
        let sb = Scrollbar::SQUAD;
        assert_eq!(sb.thumb(32, 28, 4), Some((250, 472)));
    }

    /// The club-select capture shows scroll 0 and a 99 px thumb.
    #[test]
    fn thumb_reproduces_the_club_select_capture() {
        let sb = Scrollbar::CLUB_SELECT;
        let (t0, t1) = sb.thumb(94, 28, 0).expect("thumb");
        assert_eq!(t0, 173, "at scroll 0 the thumb sits at the track top");
        assert_eq!(t1 - t0 + 1, 99, "thumb length");
    }

    #[test]
    fn no_thumb_and_no_bar_when_everything_fits() {
        assert_eq!(Scrollbar::SQUAD.thumb(10, 14, 0), None);
        assert_eq!(Scrollbar::SQUAD.hit(765, 300, 10, 14, 0), None);
    }

    #[test]
    fn hit_regions_partition_the_bar() {
        let sb = Scrollbar::SQUAD;
        assert_eq!(sb.hit(765, 200, 40, 14, 0), Some(Hit::Up));
        assert_eq!(sb.hit(765, 480, 40, 14, 0), Some(Hit::Down));
        // At scroll 0 the thumb starts at the track top, so a point well
        // below it is a page-down.
        assert_eq!(sb.hit(765, 460, 40, 14, 0), Some(Hit::PageDown));
        // Inside the thumb, grab offset is measured from its top.
        let (t0, _) = sb.thumb(40, 14, 0).unwrap();
        assert_eq!(sb.hit(765, t0 + 3, 40, 14, 0), Some(Hit::Thumb { grab: 3 }));
        // Outside the bar entirely.
        assert_eq!(sb.hit(700, 300, 40, 14, 0), None);
    }

    /// Dragging must round-trip: a thumb placed for scroll N must read
    /// back as scroll N.
    #[test]
    fn drag_round_trips_every_scroll_position() {
        let sb = Scrollbar::SQUAD;
        let (total, visible) = (40usize, 14usize);
        for scroll in 0..=(total - visible) {
            let (t0, _) = sb.thumb(total, visible, scroll).unwrap();
            assert_eq!(
                sb.scroll_for_thumb_top(t0, total, visible),
                scroll,
                "scroll {scroll} must survive a thumb round trip"
            );
        }
    }

    #[test]
    fn drag_clamps_at_both_ends() {
        let sb = Scrollbar::SQUAD;
        assert_eq!(sb.scroll_for_thumb_top(-999, 40, 14), 0);
        assert_eq!(sb.scroll_for_thumb_top(9999, 40, 14), 26);
    }
}
