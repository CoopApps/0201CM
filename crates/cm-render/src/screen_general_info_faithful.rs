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

    // View dropdown (drawn last so it overlays). Green menu with two
    // pages: General Info (checked) + Stats.
    if st.view_menu_open {
        // Container is a raised bevel (capture style 0x130 =
        // P_SOLID_FILL | P_BEVEL); the two rows are flat fill inside it.
        draw_panel(surface, 110, 148, 235, 190, P_SOLID_FILL | P_BEVEL, MENU_BG, 0, palette);
        draw_panel(surface, 112, 150, 233, 168, P_SOLID_FILL, MENU_BG, 0, palette);
        draw_panel(surface, 112, 170, 233, 188, P_SOLID_FILL, MENU_HOVER, 0, palette);
        // Capture: plain 6-space indent, black font 1, no tick glyph.
        draw_wrapped_text(surface, 112, 150, 233, 168, &small_font,
            &c_string(b"      General Info"), 0x0000, W_LEFT, -1);
        draw_wrapped_text(surface, 112, 170, 233, 188, &small_font,
            &c_string(b"      Stats"), 0x0000, W_LEFT, -1);
    }
}
