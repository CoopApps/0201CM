//! Faithful direct-draw renderer for the "Select Team" screen (the
//! last pre-boot pick — chose the club the manager takes charge of).
//! Subheader in the exe capture is literally `'Select Team'`, so the
//! module's named to match.
//!
//! Ground truth: `fixtures/club_screen/exe_paint_fb.jsonl.gz` — 243
//! structural ops from cm0102_GDI.exe with only England selected as
//! the starting league.
//!
//! Screen recipe (all coords direct from the capture):
//!
//! - Darkened container `(110,145)-(780,535)`. First row of clubs
//!   starts at `y=153` (an 8-px top pad from the container top).
//! - Row stride is 22 px. Each visible row shows TWO club entries
//!   side by side. Each entry has THREE sub-cells:
//!     - `NAME` — club name, font 3, cyan `0x739c`.
//!     - `NAT`  — nation 3-letter code (`ENG`, ...), font 2, yellow.
//!     - `DIV`  — division short code (`PRM`, `D1`, `D2`, `D3`,
//!                `CON`), font 2, ORANGE `0x7e00` (same colour the
//!                Leagues screen used for the highlighted `Yes`).
//! - Column x-ranges:
//!     LEFT  entry:  NAME `(112..341)`  NAT `(343..387)`  DIV `(389..433)`
//!     RIGHT entry:  NAME `(435..663)`  NAT `(665..709)`  DIV `(711..756)`
//! - Scrollbar identical shape to Leagues/Nationality — top arrow
//!   `(759,153)-(778,172)`, darken track `(759,173)-(778,507)`, grey
//!   thumb sized/positioned from scroll+total, bottom arrow
//!   `(759,508)-(778,527)`.
//! - Chrome (photo/sidebar/title/subheader=`Select Team`/Back+Next)
//!   comes from [`screen_pre_boot_chrome`].
//!
//! Selection: the exe frames just the NAME cell of the picked club in
//! a 1-px yellow rect (same recipe as Nationality — user's
//! [[nationality]] rule). Text ink stays put — the frame is the
//! only visual cue for "picked".

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{draw_panel, PanelPalette, P_BEVEL, P_DARKEN, P_SOLID_FILL};
use crate::packed_text::{draw_wrapped_text, W_LEFT};
use crate::screen_pre_boot_chrome::{
    c_string, draw_chrome, ChromeState,
    F_BODY, F_SMALL, GREY_BAR, INK_CYAN, INK_YELLOW, TS_CENTRE,
};

/// Division-code orange — same value the Leagues screen used for the
/// active radio's text. `0x7e00 = (31, 16, 0)`.
const INK_DIV: u16 = 0x7e00;

pub struct TeamRow<'a> {
    /// Club name — the exe pads with two leading spaces for the indent.
    pub name: &'a str,
    /// Nation 3-letter code (ENG, ITA, ...).
    pub nation: &'a str,
    /// Division short code (PRM, D1, D2, D3, CON, ...).
    pub division: &'a str,
    /// Club id — used to persist the selection across renders / filter
    /// changes; the renderer doesn't need it, but the caller does.
    pub club_id: u32,
}

pub struct TeamState<'a> {
    pub photo_seed: u64,
    pub has_manager: bool,
    /// All clubs qualifying under the current filter (see the caller's
    /// filter rules — usually all clubs in the selected starting league).
    /// Sorted alphabetically to match the exe.
    pub rows: &'a [TeamRow<'a>],
    /// First-visible-entry index (0 = start of list). Advances by 2 per
    /// wheel tick since the row is two entries wide.
    pub scroll: usize,
    /// The `club_id` of the currently-picked club (`None` until the
    /// user clicks one) — persisted across renders.
    pub selected: Option<u32>,
    pub back_enabled: bool,
    pub next_enabled: bool,
}

// -----------------------------------------------------------------------
// Layout constants — every one from `fixtures/club_screen/structure.txt`
// -----------------------------------------------------------------------

const LIST_X0: i32 = 110;
const LIST_X1: i32 = 780;
const LIST_Y0: i32 = 145;
const LIST_Y1: i32 = 535;
const ROW_FIRST_Y: i32 = 153;
const ROW_STRIDE: i32 = 22;
const ROW_HEIGHT: i32 = 20;

const NAME_L: (i32, i32) = (112, 341);
const NAT_L:  (i32, i32) = (343, 387);
const DIV_L:  (i32, i32) = (389, 433);
const NAME_R: (i32, i32) = (435, 663);
const NAT_R:  (i32, i32) = (665, 709);
const DIV_R:  (i32, i32) = (711, 756);

const SB_X0: i32 = 759;
const SB_X1: i32 = 778;
const SB_TOP_ARROW: (i32, i32) = (153, 172);
const SB_BOT_ARROW: (i32, i32) = (508, 527);
const SB_TRACK_Y0: i32 = 173;
const SB_TRACK_Y1: i32 = 507;

/// 17 rows × 2 columns = 34 entries visible at once (the row-height
/// arithmetic: first row at 153, stride 22, last row at 507 =
/// 153 + 16*22 = 505; 17 rows including the last).
const VISIBLE_ROWS: usize = 17;
pub const VISIBLE_ENTRIES: usize = VISIBLE_ROWS * 2;

pub fn render_team(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &TeamState<'_>,
) {
    draw_chrome(surface, fonts, &ChromeState {
        photo_seed: state.photo_seed,
        has_manager: state.has_manager,
        sub_title: "Select Team",
        left_nav_label: "Back",
        right_nav_label: "Next",
        back_enabled: state.back_enabled,
        next_enabled: state.next_enabled,
    });

    let palette = PanelPalette::default();
    let body_font = fonts.pixel_slot(F_BODY).clone();
    let small_font = fonts.pixel_slot(F_SMALL).clone();

    // Container darken — op #83.
    draw_panel(surface, LIST_X0, LIST_Y0, LIST_X1, LIST_Y1,
        P_DARKEN, 0, 0, palette);

    // Rows.
    let visible = state.rows.iter().skip(state.scroll).take(VISIBLE_ENTRIES);
    for (i, row) in visible.enumerate() {
        let row_idx = i / 2;
        let is_left = i % 2 == 0;
        let y0 = ROW_FIRST_Y + (row_idx as i32) * ROW_STRIDE;
        let y1 = y0 + ROW_HEIGHT;
        let (name_x, nat_x, div_x) = if is_left {
            (NAME_L, NAT_L, DIV_L)
        } else {
            (NAME_R, NAT_R, DIV_R)
        };
        // NAME — font 3, LEFT-aligned (leading spaces = indent).
        // NB: Select Team is INSTANT-COMMIT — a click on a row
        // navigates straight to that club's dashboard. There's no
        // highlight state to draw (the exe never previews a picked
        // club; it goes there immediately).
        draw_wrapped_text(surface, name_x.0, y0, name_x.1, y1,
            &body_font, &c_string(row.name.as_bytes()),
            INK_CYAN, TS_CENTRE | W_LEFT, -1);
        // NAT — centred yellow, font 2 (small).
        draw_wrapped_text(surface, nat_x.0, y0, nat_x.1, y1,
            &small_font, &c_string(row.nation.as_bytes()),
            INK_YELLOW, TS_CENTRE, -1);
        // DIV — centred orange, font 2 (small).
        draw_wrapped_text(surface, div_x.0, y0, div_x.1, y1,
            &small_font, &c_string(row.division.as_bytes()),
            INK_DIV, TS_CENTRE, -1);
    }

    // Scrollbar — same shape as Nationality.
    draw_panel(surface, SB_X0, SB_TOP_ARROW.0, SB_X1, SB_TOP_ARROW.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_panel(surface, SB_X0, SB_TRACK_Y0, SB_X1, SB_TRACK_Y1,
        P_DARKEN, 0, 0, palette);
    let total = state.rows.len().max(1);
    let track_h = (SB_TRACK_Y1 - SB_TRACK_Y0).max(1) as f32;
    let visible_frac = (VISIBLE_ENTRIES as f32 / total as f32).min(1.0);
    let thumb_h = (track_h * visible_frac).max(20.0).min(track_h) as i32;
    let max_scroll = total.saturating_sub(VISIBLE_ENTRIES);
    let thumb_y0 = if max_scroll == 0 {
        SB_TRACK_Y0
    } else {
        SB_TRACK_Y0
            + ((track_h - thumb_h as f32) * (state.scroll as f32 / max_scroll as f32)) as i32
    };
    let thumb_y1 = (thumb_y0 + thumb_h).min(SB_TRACK_Y1);
    draw_panel(surface, SB_X0, thumb_y0, SB_X1, thumb_y1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_panel(surface, SB_X0, SB_BOT_ARROW.0, SB_X1, SB_BOT_ARROW.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_team_smoke() {
        let mut surface = PackedSurface::rgb555(800, 600);
        let mut fonts = Fonts::new("D:/cm0102/Data");
        let rows: Vec<TeamRow> = vec![];
        let state = TeamState {
            photo_seed: 0, has_manager: false,
            rows: &rows, scroll: 0, selected: None,
            back_enabled: true, next_enabled: false,
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_team(&mut surface, &mut fonts, &state);
        }));
    }
}
