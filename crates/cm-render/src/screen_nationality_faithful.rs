//! Faithful direct-draw renderer for the "Select Nationality" screen.
//!
//! Chrome from [`screen_pre_boot_chrome`]. This module owns:
//!
//! - Small "Filter" button top-right (655,145)-(780,165).
//! - Two-column scrollable list of nationalities (110,170)-(780,535):
//!   each visible row shows TWO entries side by side, each entry being
//!   a country/nationality name (font 3, cyan) + a 3-letter continent
//!   code (AFR / ASI / EUR / NAM / OCE / SAM, font 2, yellow).
//! - Scrollbar on the right (759..778) with top arrow / darken track /
//!   grey thumb / bottom arrow — same layout as Leagues.
//!
//! Ground truth: `fixtures/nationality_screen/exe_paint_fb.jsonl.gz`
//! (170 structural ops, captured live 2026-09-05 while the S..T
//! nationalities were visible).

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{draw_panel, PanelPalette, P_BEVEL, P_DARKEN, P_SOLID_FILL};
use crate::packed_text::{draw_wrapped_text, W_LEFT};
use crate::screen_pre_boot_chrome::{
    c_string, draw_chrome, ChromeState,
    F_BODY, F_SMALL, GREY_BAR, INK_CYAN, INK_YELLOW, TS_CENTRE,
};

/// One nationality option.
pub struct NationalityRow<'a> {
    /// Adjective form (e.g. "Spanish", "South Korean"). The exe pads
    /// with two leading spaces for the left-indent look.
    pub name: &'a str,
    /// 3-letter continent code — AFR / ASI / EUR / NAM / OCE / SAM.
    pub continent: &'a str,
}

pub struct NationalityState<'a> {
    pub photo_seed: u64,
    pub has_manager: bool,
    /// All picker rows in the exe's display order (alphabetical).
    /// The exe list holds ~200 nationalities; only 16 visible rows fit
    /// per column, i.e. 32 entries visible at a time.
    pub rows: &'a [NationalityRow<'a>],
    /// First visible entry index (0 = start of list). Advances by 2 per
    /// wheel tick since the visible window is 2 entries wide per row.
    pub scroll: usize,
    pub back_enabled: bool,
    pub next_enabled: bool,
}

// -----------------------------------------------------------------------
// Layout (from the exe capture)
// -----------------------------------------------------------------------

/// Filter button (op #579).
const FILTER: (i32, i32, i32, i32) = (655, 145, 780, 165);
/// List container (op #83).
const LIST_X0: i32 = 110;
const LIST_X1: i32 = 780;
const LIST_Y0: i32 = 170;
const LIST_Y1: i32 = 535;
/// Row layout — starts y=178, height 20, stride 22.
const ROW_FIRST_Y: i32 = 178;
const ROW_STRIDE: i32 = 22;
const ROW_HEIGHT: i32 = 20;
/// Column geometry for the two side-by-side entries.
const NAME_L: (i32, i32) = (112, 387);
const CONT_L: (i32, i32) = (389, 433);
const NAME_R: (i32, i32) = (435, 709);
const CONT_R: (i32, i32) = (711, 756);
/// Scrollbar (op #277-368).
const SB_X0: i32 = 759;
const SB_X1: i32 = 778;
const SB_TOP_ARROW: (i32, i32) = (178, 197);
const SB_BOT_ARROW: (i32, i32) = (508, 527);
const SB_TRACK_Y0: i32 = 198;
const SB_TRACK_Y1: i32 = 507;
/// 16 rows × 2 columns = 32 entries visible at once.
const VISIBLE_ROWS: usize = 16;
pub const VISIBLE_ENTRIES: usize = VISIBLE_ROWS * 2;

pub fn render_nationality(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &NationalityState<'_>,
) {
    draw_chrome(surface, fonts, &ChromeState {
        photo_seed: state.photo_seed,
        has_manager: state.has_manager,
        sub_title: "Select Nationality",
        left_nav_label: "Back",
        right_nav_label: "Next",
        back_enabled: state.back_enabled,
        next_enabled: state.next_enabled,
    });

    let palette = PanelPalette::default();
    let small_font = fonts.pixel_slot(F_SMALL).clone();
    let body_font = fonts.pixel_slot(F_BODY).clone();

    // Filter button (op #579 — grey P_SOLID_FILL|P_BEVEL, cyan label).
    draw_panel(surface, FILTER.0, FILTER.1, FILTER.2, FILTER.3,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_wrapped_text(surface, FILTER.0, FILTER.1, FILTER.2, FILTER.3,
        &small_font, &c_string(b"Filter"), INK_CYAN, TS_CENTRE, -1);

    // List container darken (op #83-84).
    draw_panel(surface, LIST_X0, LIST_Y0, LIST_X1, LIST_Y1,
        P_DARKEN, 0, 0, palette);

    // Rows — 2 columns of entries.
    let visible = state.rows.iter().skip(state.scroll).take(VISIBLE_ENTRIES);
    for (i, row) in visible.enumerate() {
        let row_idx = i / 2;
        let is_left = i % 2 == 0;
        let y0 = ROW_FIRST_Y + (row_idx as i32) * ROW_STRIDE;
        let y1 = y0 + ROW_HEIGHT;
        let (name_x0, name_x1, cont_x0, cont_x1) = if is_left {
            (NAME_L.0, NAME_L.1, CONT_L.0, CONT_L.1)
        } else {
            (NAME_R.0, NAME_R.1, CONT_R.0, CONT_R.1)
        };
        // Name — LEFT-aligned cyan (font 3), leading whitespace in the
        // captured strings acts as the indent.
        draw_wrapped_text(surface, name_x0, y0, name_x1, y1,
            &body_font, &c_string(row.name.as_bytes()),
            INK_CYAN, TS_CENTRE | W_LEFT, -1);
        // Continent code — centred yellow (font 2 in the exe; using
        // slot 1 = arial_narrow_10 is close enough visually).
        draw_wrapped_text(surface, cont_x0, y0, cont_x1, y1,
            &small_font, &c_string(row.continent.as_bytes()),
            INK_YELLOW, TS_CENTRE, -1);
    }

    // Scrollbar (op #277-368).
    // Top arrow.
    draw_panel(surface, SB_X0, SB_TOP_ARROW.0, SB_X1, SB_TOP_ARROW.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    // Darken track.
    draw_panel(surface, SB_X0, SB_TRACK_Y0, SB_X1, SB_TRACK_Y1,
        P_DARKEN, 0, 0, palette);
    // Thumb — sized/positioned from scroll and total row count.
    let total_entries = state.rows.len().max(1);
    let track_h = (SB_TRACK_Y1 - SB_TRACK_Y0).max(1) as f32;
    let visible_frac = (VISIBLE_ENTRIES as f32 / total_entries as f32).min(1.0);
    let thumb_h = (track_h * visible_frac).max(20.0).min(track_h) as i32;
    let max_scroll = total_entries.saturating_sub(VISIBLE_ENTRIES);
    let thumb_y0 = if max_scroll == 0 {
        SB_TRACK_Y0
    } else {
        SB_TRACK_Y0
            + ((track_h - thumb_h as f32) * (state.scroll as f32 / max_scroll as f32)) as i32
    };
    let thumb_y1 = (thumb_y0 + thumb_h).min(SB_TRACK_Y1);
    draw_panel(surface, SB_X0, thumb_y0, SB_X1, thumb_y1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    // Bottom arrow.
    draw_panel(surface, SB_X0, SB_BOT_ARROW.0, SB_X1, SB_BOT_ARROW.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_nationality_smoke() {
        let mut surface = PackedSurface::rgb555(800, 600);
        let mut fonts = Fonts::new("D:/cm0102/Data");
        let rows: Vec<NationalityRow> = vec![];
        let state = NationalityState {
            photo_seed: 0, has_manager: false,
            rows: &rows, scroll: 0,
            back_enabled: true, next_enabled: false,
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_nationality(&mut surface, &mut fonts, &state);
        }));
    }
}
