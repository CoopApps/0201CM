//! Faithful direct-draw renderer for the Select League(s) screen.
//!
//! Shared pre-boot chrome (photo / sidebar / title bar / subheader /
//! nav bar) lives in [`screen_pre_boot_chrome`]. This module owns the
//! Select-Leagues-specific content: the options bar (Real Players / Attribute
//! Masking toggles + Select-All / De-Select-All), the scrollable country
//! list with its SELECTED / BACKGROUND per-row toggles + secondary-league
//! label column, and the right-side scrollbar.
//!
//! **Hardcoded coords are all from the exe capture.** Ground truth:
//! `fixtures/leagues_screen/exe_paint_fb.jsonl.gz` (781 ops per repaint,
//! recorded live from cm0102_GDI.exe on 2026-09-05). Structural summary:
//! `fixtures/leagues_screen/structure.txt`. Op indices in the comments
//! below refer to that structure file.

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{
    draw_panel, PanelPalette,
    P_BEVEL, P_DARKEN, P_OUTER_HIGHLIGHT, P_SOLID_FILL,
};
use crate::packed_text::draw_wrapped_text;
use crate::screen_pre_boot_chrome::{
    c_string, draw_chrome, ChromeState,
    F_BODY, F_SMALL, GREY_BAR, INK_CYAN, INK_YELLOW, TS_CENTRE, YELLOW_PATTERN,
};

// -----------------------------------------------------------------------
// Leagues-specific colour constants (all from the capture)
// -----------------------------------------------------------------------

/// Orange text used for the currently-highlighted radio option ("Yes"
/// when Real Players is on, etc.). Op #90 captured: `c=0x7e00`.
/// `0x7e00 = (31, 16, 0)` — red 31, green 16, blue 0.
const INK_HIGHLIGHT: u16 = 0x7e00;

/// The exe's darkening for the OFF/disabled radio-button text.
/// `c=0x4210 = (16, 16, 16)` — same as `GREY_BAR`, used here as the
/// "not-selected" ink for radio buttons + toggles.
const INK_DISABLED: u16 = GREY_BAR;

// -----------------------------------------------------------------------
// Leagues state
// -----------------------------------------------------------------------

/// One row in the picker.
pub struct LeaguesRow<'a> {
    /// Country / competition name (e.g. "  Argentina", "  England").
    pub country: &'a str,
    /// Whether the primary SELECTED toggle is on.
    pub selected: bool,
    /// Whether the BACKGROUND toggle is on.
    pub background_marker: bool,
    /// Optional secondary league label — "Conference Division",
    /// "Regional Divisions", "Serie C2 A, B, C", etc. Absent for
    /// countries that have no reserve tier.
    pub secondary_label: Option<&'a str>,
    /// Whether the secondary toggle is currently active.
    pub secondary_active: bool,
}

/// Runtime state parcel — the equivalent of `SetupState` for Leagues.
pub struct LeaguesState<'a> {
    pub photo_seed: u64,
    pub has_manager: bool,
    /// Real Players option (top-left radio pair).
    pub use_real_players: bool,
    /// Attribute Masking option (right of Real Players).
    pub attribute_masking: bool,
    /// The picker rows in display order. Only the first ~16 are visible
    /// at once; the rest are reached via the scrollbar.
    pub rows: &'a [LeaguesRow<'a>],
    /// First visible row index (0 = top of list).
    pub scroll: usize,
    /// Back / Next enable — clickable on this screen; both true means
    /// bright cyan text.
    pub back_enabled: bool,
    pub next_enabled: bool,
    /// Content-button press indices — for future press-invert feedback.
    /// Same convention as SetupState.
    pub pressed: Option<usize>,
}

// -----------------------------------------------------------------------
// Layout constants (from exe capture)
// -----------------------------------------------------------------------

/// Options bar band (op #79 darken).
const OPTS_Y0: i32 = 145;
const OPTS_Y1: i32 = 165;

/// Options radio-group row (labels + Yes/No cells share this vertical span).
const RADIO_Y0: i32 = 147;
const RADIO_Y1: i32 = 163;

/// Real Players label / Yes / No rects (ops #81-93).
const RP_LBL: (i32, i32) = (112, 227);
const RP_YES: (i32, i32) = (229, 264);
const RP_NO:  (i32, i32) = (268, 303);

/// Attribute Masking rects (ops #95-107).
const AM_LBL: (i32, i32) = (332, 447);
const AM_YES: (i32, i32) = (449, 484);
const AM_NO:  (i32, i32) = (488, 523);

/// Select All / De-Select All (ops #110-174).
const SEL_ALL:    (i32, i32) = (530, 654);
const DESEL_ALL:  (i32, i32) = (656, 780);

/// Country list container (ops #176-177 darken).
const LIST_X0: i32 = 110;
const LIST_X1: i32 = 780;
const LIST_Y0: i32 = 170;
const LIST_Y1: i32 = 535;

/// Row layout (op #178 onward, rows step by 22px).
const ROW_STRIDE: i32 = 22;
const ROW_FIRST_Y: i32 = 178;
const ROW_HEIGHT: i32 = 20;

const COL_COUNTRY:   (i32, i32) = (112, 308);
const COL_SELECTED:  (i32, i32) = (319, 446);
const COL_BG:        (i32, i32) = (457, 584);
const COL_SECONDARY: (i32, i32) = (594, 756);

/// Scrollbar (ops #336-569).
const SB_X0: i32 = 759;
const SB_X1: i32 = 778;
const SB_TOP_ARROW: (i32, i32) = (178, 197);
const SB_BOT_ARROW: (i32, i32) = (508, 527);
const SB_TRACK_Y0: i32 = 198;
const SB_TRACK_Y1: i32 = 507;

/// Max visible rows before we clip.
const VISIBLE_ROWS: usize = 16;

// -----------------------------------------------------------------------
// Renderer
// -----------------------------------------------------------------------

/// Paint the Select League(s) screen.
pub fn render_leagues(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &LeaguesState<'_>,
) {
    draw_chrome(surface, fonts, &ChromeState {
        photo_seed: state.photo_seed,
        has_manager: state.has_manager,
        sub_title: "Select League(s)",
        back_enabled: state.back_enabled,
        next_enabled: state.next_enabled,
    });

    let palette = PanelPalette::default();
    let small_font = fonts.pixel_slot(F_SMALL).clone();
    let body_font  = fonts.pixel_slot(F_BODY).clone();

    // -------- Options bar (darken band + radio groups + button pair) --
    // Op #79-80: darken the whole (110..525, 145..165) band.
    draw_panel(surface, LIST_X0, OPTS_Y0, 525, OPTS_Y1, P_DARKEN, 0, 0, palette);

    // Real Players label (op #81-82).
    label_cell(surface, &small_font, RP_LBL.0, RADIO_Y0, RP_LBL.1, RADIO_Y1,
               "  Use Real Players:", INK_CYAN);
    // Yes/No pair — one is highlighted (op #84-93 pattern).
    radio_option(surface, &small_font, RP_YES.0, RADIO_Y0, RP_YES.1, RADIO_Y1,
                 "Yes", state.use_real_players);
    radio_option(surface, &small_font, RP_NO.0, RADIO_Y0, RP_NO.1, RADIO_Y1,
                 "No", !state.use_real_players);

    // Attribute Masking label (op #95-96).
    label_cell(surface, &small_font, AM_LBL.0, RADIO_Y0, AM_LBL.1, RADIO_Y1,
               "  Attribute Masking:", INK_CYAN);
    radio_option(surface, &small_font, AM_YES.0, RADIO_Y0, AM_YES.1, RADIO_Y1,
                 "Yes", state.attribute_masking);
    radio_option(surface, &small_font, AM_NO.0, RADIO_Y0, AM_NO.1, RADIO_Y1,
                 "No", !state.attribute_masking);

    // Select All / De-Select All (op #109-174).
    action_button(surface, &small_font, SEL_ALL.0, OPTS_Y0, SEL_ALL.1, OPTS_Y1,
                  "Select All");
    action_button(surface, &small_font, DESEL_ALL.0, OPTS_Y0, DESEL_ALL.1, OPTS_Y1,
                  "De-Select All");

    // -------- League list (darken container + rows) -------------------
    // Op #176-177.
    draw_panel(surface, LIST_X0, LIST_Y0, LIST_X1, LIST_Y1, P_DARKEN, 0, 0, palette);

    // Rows — starting at ROW_FIRST_Y, stepping ROW_STRIDE.
    let visible = state.rows.iter().skip(state.scroll).take(VISIBLE_ROWS);
    for (i, row) in visible.enumerate() {
        let y0 = ROW_FIRST_Y + (i as i32) * ROW_STRIDE;
        let y1 = y0 + ROW_HEIGHT;
        // Country name — cyan text, no fill (op #178-179).
        draw_wrapped_text(surface, COL_COUNTRY.0, y0, COL_COUNTRY.1, y1,
            &body_font, &c_string(row.country.as_bytes()),
            INK_CYAN, TS_CENTRE, -1);
        // SELECTED toggle — highlighted when on.
        toggle_cell(surface, &small_font, COL_SELECTED.0, y0, COL_SELECTED.1, y1,
                    "SELECTED", row.selected);
        // BACKGROUND toggle.
        toggle_cell(surface, &small_font, COL_BG.0, y0, COL_BG.1, y1,
                    "BACKGROUND", row.background_marker);
        // Optional secondary label.
        if let Some(sec) = row.secondary_label {
            toggle_cell(surface, &small_font,
                        COL_SECONDARY.0, y0, COL_SECONDARY.1, y1,
                        sec, row.secondary_active);
        }
    }

    // -------- Scrollbar ----------------------------------------------
    // Top arrow (op #336-337).
    draw_panel(surface, SB_X0, SB_TOP_ARROW.0, SB_X1, SB_TOP_ARROW.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    // Darken track (op #366-367).
    draw_panel(surface, SB_X0, SB_TRACK_Y0, SB_X1, SB_TRACK_Y1,
        P_DARKEN, 0, 0, palette);
    // Thumb (op #368-369). Sized by scroll: for a full-list-visible
    // capture the exe shows thumb from y=198 to y=387 (~62% of track).
    // Compute the thumb rect as a linear function of `scroll` and the
    // total row count.
    let total = state.rows.len().max(1);
    let track_h = (SB_TRACK_Y1 - SB_TRACK_Y0).max(1) as f32;
    let thumb_h = (track_h * (VISIBLE_ROWS as f32 / total as f32))
                        .max(20.0)
                        .min(track_h) as i32;
    let max_scroll = total.saturating_sub(VISIBLE_ROWS);
    let thumb_y0 = if max_scroll == 0 {
        SB_TRACK_Y0
    } else {
        SB_TRACK_Y0
            + ((track_h - thumb_h as f32) * (state.scroll as f32 / max_scroll as f32)) as i32
    };
    let thumb_y1 = (thumb_y0 + thumb_h).min(SB_TRACK_Y1);
    draw_panel(surface, SB_X0, thumb_y0, SB_X1, thumb_y1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    // Bottom arrow (op #568-569).
    draw_panel(surface, SB_X0, SB_BOT_ARROW.0, SB_X1, SB_BOT_ARROW.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);

    let _ = state.pressed; // press-invert on toggles TBD
}

// -----------------------------------------------------------------------
// Sub-cell primitives
// -----------------------------------------------------------------------

/// Right-aligned option label (grey fill, cyan text) — used for
/// "  Use Real Players:" and "  Attribute Masking:".
fn label_cell(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    x0: i32, y0: i32, x1: i32, y1: i32,
    label: &str,
    ink: u16,
) {
    let palette = PanelPalette::default();
    // Op capture: PANEL c=0x4210 p=0x739c s=0x1 → P_SOLID_FILL grey.
    draw_panel(surface, x0, y0, x1, y1, P_SOLID_FILL, GREY_BAR, 0, palette);
    draw_wrapped_text(surface, x0, y0, x1, y1, font,
        &c_string(label.as_bytes()), ink, TS_CENTRE, -1);
}

/// One "Yes" / "No" radio pill — highlighted variant has a yellow rect
/// frame around it and orange text; off variant is grey text on grey.
fn radio_option(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    x0: i32, y0: i32, x1: i32, y1: i32,
    label: &str,
    active: bool,
) {
    let palette = PanelPalette::default();
    if active {
        // Op #84-90 pattern for the highlighted variant:
        //   PANEL c=0x4210 p=0x7e00 s=0x801   (SOLID_FILL | OUTER_HIGHLIGHT)
        //   rect (x0-1..x1+1, y0-1..y1+1) c=0x7fe0 s=2  (yellow outline)
        //   WRAP … c=0x7e00 orange
        draw_panel(surface, x0, y0, x1, y1,
            P_SOLID_FILL | P_OUTER_HIGHLIGHT, GREY_BAR, INK_HIGHLIGHT, palette);
        surface.draw_rectangle(x0 - 1, y0 - 1, x1 + 1, y1 + 1, 2, YELLOW_PATTERN);
        draw_wrapped_text(surface, x0, y0, x1, y1, font,
            &c_string(label.as_bytes()), INK_HIGHLIGHT, TS_CENTRE, -1);
    } else {
        // Op #92-93: grey panel + grey text.
        draw_panel(surface, x0, y0, x1, y1, P_SOLID_FILL, GREY_BAR, 0, palette);
        draw_wrapped_text(surface, x0, y0, x1, y1, font,
            &c_string(label.as_bytes()), INK_DISABLED, TS_CENTRE, -1);
    }
    let _ = INK_YELLOW;
}

/// Grey bevelled action button (Select All / De-Select All / etc.).
fn action_button(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    x0: i32, y0: i32, x1: i32, y1: i32,
    label: &str,
) {
    let palette = PanelPalette::default();
    // Op #110-141: PANEL c=0x4210 p=0x739c s=0x30 + rect c=0x4210 s=4
    // → grey fill + grey bevel. Text cyan.
    draw_panel(surface, x0, y0, x1, y1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_wrapped_text(surface, x0, y0, x1, y1, font,
        &c_string(label.as_bytes()), INK_CYAN, TS_CENTRE, -1);
}

/// A per-row toggle cell (SELECTED / BACKGROUND / secondary label). ON
/// = yellow-highlighted like the Yes radios; OFF = flat grey text.
fn toggle_cell(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    x0: i32, y0: i32, x1: i32, y1: i32,
    label: &str,
    active: bool,
) {
    if active {
        // Same highlighted pattern as the Yes radio.
        let palette = PanelPalette::default();
        draw_panel(surface, x0, y0, x1, y1,
            P_SOLID_FILL | P_OUTER_HIGHLIGHT, GREY_BAR, INK_HIGHLIGHT, palette);
        surface.draw_rectangle(x0 - 1, y0 - 1, x1 + 1, y1 + 1, 2, YELLOW_PATTERN);
        draw_wrapped_text(surface, x0, y0, x1, y1, font,
            &c_string(label.as_bytes()), INK_HIGHLIGHT, TS_CENTRE, -1);
    } else {
        // Off: op #181-182 pattern — PANEL c=0 p=0x4210 s=0x1 (no fill)
        // + grey text. The exe leaves the darkened container showing
        // through the "no fill" panel.
        draw_wrapped_text(surface, x0, y0, x1, y1, font,
            &c_string(label.as_bytes()), INK_DISABLED, TS_CENTRE, -1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_leagues_produces_pixels() {
        let mut surface = PackedSurface::rgb555(800, 600);
        let mut fonts = Fonts::new("D:/cm0102/Data");
        let rows: Vec<LeaguesRow> = vec![];
        let state = LeaguesState {
            photo_seed: 0, has_manager: false,
            use_real_players: true, attribute_masking: true,
            rows: &rows, scroll: 0,
            back_enabled: true, next_enabled: true,
            pressed: None,
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_leagues(&mut surface, &mut fonts, &state);
        }));
    }
}
