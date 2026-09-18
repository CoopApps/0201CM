//! General Info screen (top tab #4, "General Info" page) — faithful to
//! the exe capture `fixtures/general_info_screen/` (decode:
//! `reports/general_info_screen_decode.md`).
//!
//! Two darkened blocks: a 6-row detail panel (110,190)-(780,332) and a
//! scrollable Non-Playing Staff panel (110,337)-(780,500). A View button
//! (110,125)-(235,145) opens the General Info / Stats page dropdown.
//! Label col x 112..301 (font 3, grey); value col x 303..778 (font 2,
//! yellow). Staff: name x=112 (font 3), role x=371 (font 2).

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{draw_panel, PanelPalette, P_DARKEN, P_SOLID_FILL, P_BEVEL};
use crate::packed_text::{draw_wrapped_text, W_LEFT};
use crate::screen_pre_boot_chrome::{c_string, TS_CENTRE};

const INK_GREY:   u16 = 0x739c;
const INK_YELLOW: u16 = 0x7fe0;
const INK_ORANGE: u16 = 0x7e00;
const GREY_BAR:   u16 = 0x4210;
/// View dropdown greens (capture: container 0x0200, hover row 0x0240).
const MENU_BG:    u16 = 0x0200;
const MENU_HOVER: u16 = 0x0240;

/// The data this screen paints (borrowed from `GeneralInfoView`).
pub struct GeneralInfoState<'a> {
    pub nation: &'a str,
    pub status: &'a str,
    pub finances: &'a str,
    pub stadium: &'a str,
    pub facilities: &'a str,
    pub training_ground: &'a str,
    /// (name, role) rows.
    pub staff: &'a [(&'a str, &'a str)],
    /// First-visible staff row.
    pub staff_scroll: usize,
    /// The View dropdown is open.
    pub view_menu_open: bool,
    pub club_name: &'a str,
    pub photo_seed: u64,
    pub has_manager: bool,
    pub division_name: &'a str,
    pub kit_bg_rgb565: u16,
    pub kit_fg_rgb565: u16,
}

/// Staff rows visible at once in the (337..500) panel. Rows are at
/// y=388 + n*21; five (388..472) fit above the 500 border — a sixth at
/// y=493 would overrun it. Capture confirms 5 (Terry Smith..Mark Mason).
pub const STAFF_VISIBLE: usize = 5;

/// Draw the General Info body over the shared club chrome.
pub fn render_general_info_body(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    st: &GeneralInfoState<'_>,
) {
    let palette = PanelPalette::default();
    let body_font = fonts.pixel_slot(crate::screen_pre_boot_chrome::F_BODY).clone();
    let small_font = fonts.pixel_slot(crate::screen_pre_boot_chrome::F_SMALL).clone();
    let value_font = fonts.pixel_slot(2).clone();

    // View button — gray bevel + "View" (small font).
    draw_panel(surface, 110, 125, 235, 145, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_wrapped_text(surface, 110, 125, 235, 145, &small_font,
        &c_string(b"View"), INK_GREY, TS_CENTRE, -1);

    // Title band — "General Info", darkened, yellow centred.
    draw_panel(surface, 110, 150, 780, 185, P_DARKEN, 0, 0, palette);
    draw_wrapped_text(surface, 110, 150, 780, 185, &body_font,
        &c_string(b"General Info"), INK_YELLOW, TS_CENTRE, -1);

    // Detail panel (see-through darken, no fill — like the squad body).
    draw_panel(surface, 110, 190, 780, 332, P_DARKEN, 0, 0, palette);
    let label = |s: &mut PackedSurface, y: i32, t: &str| {
        draw_wrapped_text(s, 112, y, 301, y + 19, &body_font,
            &c_string(t.as_bytes()), INK_GREY, W_LEFT, -1);
    };
    let value = |s: &mut PackedSurface, y: i32, t: &str| {
        draw_wrapped_text(s, 303, y, 778, y + 19, &value_font,
            &c_string(t.as_bytes()), INK_YELLOW, W_LEFT, -1);
    };
    label(surface, 198, "  Nation");          value(surface, 198, st.nation);
    label(surface, 220, "  Status");          value(surface, 220, st.status);
    label(surface, 241, "  Finances");        value(surface, 241, st.finances);
    label(surface, 262, "  Stadium");         value(surface, 262, st.stadium);
    label(surface, 283, "  Facilities");      value(surface, 283, st.facilities);
    label(surface, 304, "  Training Ground"); value(surface, 304, st.training_ground);

    // Non-Playing Staff panel.
    draw_panel(surface, 110, 337, 780, 500, P_DARKEN, 0, 0, palette);
    // Header + orange underline (capture: PANEL (115,367)-(267,386)).
    draw_wrapped_text(surface, 115, 345, 300, 364, &body_font,
        &c_string(b"Non-Playing Staff"), INK_ORANGE, W_LEFT, -1);
    let hdr_w = crate::packed_text::measure_line(&body_font, &c_string(b"Non-Playing Staff"));
    draw_panel(surface, 115, 367, 115 + hdr_w.max(1), 368, P_SOLID_FILL, INK_ORANGE, 0, palette);

    // Staff rows from y=388, 21px pitch: name (font 3) + role (font 2).
    const ROW_Y0: i32 = 388;
    const ROW_STRIDE: i32 = 21;
    for (i, (name, role)) in st.staff.iter().skip(st.staff_scroll).take(STAFF_VISIBLE).enumerate() {
        let y = ROW_Y0 + (i as i32) * ROW_STRIDE;
        draw_wrapped_text(surface, 112, y, 360, y + 19, &body_font,
            &c_string(format!("  {name}").as_bytes()), INK_GREY, W_LEFT, -1);
        draw_wrapped_text(surface, 371, y, 756, y + 19, &value_font,
            &c_string(role.as_bytes()), INK_YELLOW, W_LEFT, -1);
    }
    // Staff scrollbar (capture: (759,345)-(778,492)).
    if st.staff.len() > STAFF_VISIBLE {
        crate::scrollbar::Scrollbar::new(345, 492)
            .draw(surface, st.staff.len(), STAFF_VISIBLE, st.staff_scroll, GREY_BAR, palette);
    }

    // View dropdown (drawn last so it overlays).
    if st.view_menu_open {
        draw_view_dropdown(surface, &small_font, StatsPage::Info, palette);
    }
}

/// Which General Info page the tab is showing (the two View items).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatsPage { Info, Stats }

/// The green View dropdown shared by both General Info pages. Bevelled
/// container (capture style 0x130 = P_SOLID_FILL | P_BEVEL, col 512),
/// two flat rows; the `active` page's row carries the hover colour.
fn draw_view_dropdown(
    surface: &mut PackedSurface,
    small_font: &crate::packed_glyph::PixelFont,
    active: StatsPage,
    palette: PanelPalette,
) {
    draw_panel(surface, 110, 148, 235, 190, P_SOLID_FILL | P_BEVEL, MENU_BG, 0, palette);
    let info_bg  = if active == StatsPage::Info  { MENU_HOVER } else { MENU_BG };
    let stats_bg = if active == StatsPage::Stats { MENU_HOVER } else { MENU_BG };
    draw_panel(surface, 112, 150, 233, 168, P_SOLID_FILL, info_bg, 0, palette);
    draw_panel(surface, 112, 170, 233, 188, P_SOLID_FILL, stats_bg, 0, palette);
    // Capture: plain 6-space indent, black font 1, no tick glyph.
    draw_wrapped_text(surface, 112, 150, 233, 168, small_font,
        &c_string(b"      General Info"), 0x0000, W_LEFT, -1);
    draw_wrapped_text(surface, 112, 170, 233, 188, small_font,
        &c_string(b"      Stats"), 0x0000, W_LEFT, -1);
}

/// The 16 fixed Stats-page row labels, top→bottom (two leading spaces,
/// matching the capture).
pub const STATS_LABELS: [&str; 16] = [
    "  Number Of Players",
    "  Number Of Players Injured",
    "  Average Age - First Team",
    "  Average Age - Squad",
    "  Total Wage Bill - First Team (p/w)",
    "  Total Wage Bill - Squad (p/w)",
    "  Average Wage (p/w)",
    "  Highest Wage (p/w)",
    "  Lowest Wage (p/w)",
    "  Oldest Player",
    "  Youngest Player",
    "  Highest Valued Player",
    "  Number Of Current International Players",
    "  Number Of Current Under 21 Players",
    "  Number Of Foreign Players",
    "  Number Of Non-EU Players",
];

/// Row top-y for each Stats row (capture: stride ~17.4, alternating
/// 17/18 from the layout engine).
const STATS_ROW_Y: [i32; 16] = [
    198, 216, 233, 251, 268, 285, 303, 320,
    337, 355, 372, 389, 407, 424, 441, 459,
];

/// The Stats page (View → Stats). Same frame as General Info but a single
/// content panel (110,190)-(780,500) with 16 label/value rows. `values`
/// are the 16 yellow value strings in row order (from
/// `GeneralInfoStatsView::value_rows`).
pub struct GeneralInfoStatsState<'a> {
    pub values: [&'a str; 16],
    pub view_menu_open: bool,
    pub club_name: &'a str,
    pub photo_seed: u64,
    pub has_manager: bool,
    pub division_name: &'a str,
    pub kit_bg_rgb565: u16,
    pub kit_fg_rgb565: u16,
}

/// Draw the Stats body over the shared club chrome.
pub fn render_general_info_stats_body(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    st: &GeneralInfoStatsState<'_>,
) {
    let palette = PanelPalette::default();
    let small_font = fonts.pixel_slot(crate::screen_pre_boot_chrome::F_SMALL).clone();
    // Both label and value columns use font slot 2 on this page (capture).
    let cell_font = fonts.pixel_slot(2).clone();

    // View button — grey bevel + "View".
    draw_panel(surface, 110, 125, 235, 145, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_wrapped_text(surface, 110, 125, 235, 145, &small_font,
        &c_string(b"View"), INK_GREY, TS_CENTRE, -1);

    // Title band — "Stats", darkened, yellow centred.
    draw_panel(surface, 110, 150, 780, 185, P_DARKEN, 0, 0, palette);
    draw_wrapped_text(surface, 110, 150, 780, 185, &cell_font,
        &c_string(b"Stats"), INK_YELLOW, TS_CENTRE, -1);

    // Single darkened content panel (see-through, no fill).
    draw_panel(surface, 110, 190, 780, 500, P_DARKEN, 0, 0, palette);
    for (i, label) in STATS_LABELS.iter().enumerate() {
        let y = STATS_ROW_Y[i];
        draw_wrapped_text(surface, 112, y, 444, y + 16, &cell_font,
            &c_string(label.as_bytes()), INK_GREY, W_LEFT, -1);
        draw_wrapped_text(surface, 446, y, 778, y + 16, &cell_font,
            &c_string(st.values[i].as_bytes()), INK_YELLOW, W_LEFT, -1);
    }

    if st.view_menu_open {
        draw_view_dropdown(surface, &small_font, StatsPage::Stats, palette);
    }
}
