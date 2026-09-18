//! Next Match screen renderer — faithful to the exe capture
//! `fixtures/next_match_screen/` (decode:
//! `reports/next_match_screen_decode.md`).
//!
//! Geometry, verbatim from the capture:
//! - Nav row y 125..145: `<< Date` (110-234), `Date >>` (236-360),
//!   `Progress` (530-654, competitive only), `Past Meetings` (656-780).
//! - Title band: opponent "(Home/Away)" WRAP (110,150)-(780,185),
//!   yellow 0x7FE0, centred.
//! - Competition label: WRAP (115,198)-, orange 0x7E00, left.
//! - Detail block: label col x 112..352 grey 0x739C, value col
//!   x 354..756 yellow 0x7FE0, 21px row pitch from y 241.
//! - Availability rows: value col x 355..758, 21px pitch from y 367.
//!
//! Enabled/disabled control rule (also from the decode): an enabled
//! control draws its text ONCE, flat, at the control colour; a disabled
//! one draws twice — a light shadow 0x6F7A at (x+1,y+1) then a dark main
//! 0x294A at (x,y).

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{draw_panel, PanelPalette, P_DARKEN, P_SOLID_FILL, P_BEVEL};
use crate::packed_text::{draw_wrapped_text, W_LEFT};
use crate::screen_pre_boot_chrome::{c_string, TS_CENTRE};

const INK_GREY:   u16 = 0x739c;
const INK_YELLOW: u16 = 0x7fe0;
const INK_ORANGE: u16 = 0x7e00;
const DIS_SHADOW: u16 = 0x6f7a;
const DIS_MAIN:   u16 = 0x294a;
const GREY_BAR:   u16 = 0x4210;

/// The data this screen paints. Mirrors `cm_domain::next_match::NextMatchView`
/// but as borrowed strings so the app can pass its live view without cloning.
pub struct NextMatchState<'a> {
    pub title: &'a str,
    pub competition: &'a str,
    pub date_line: &'a str,
    pub venue: &'a str,
    pub match_rules: [&'a str; 2],
    pub last_meeting: &'a str,
    pub weather: &'a str,
    pub news: &'a [&'a str],
    pub is_friendly: bool,
    pub has_earlier: bool,
    pub has_later: bool,
    /// First-visible availability row (wheel/scroll offset).
    pub news_scroll: usize,
    pub club_name: &'a str,
    pub photo_seed: u64,
    pub has_manager: bool,
    pub division_name: &'a str,
    /// Home-kit background / foreground for the title bar (0 = fallback).
    pub kit_bg_rgb565: u16,
    pub kit_fg_rgb565: u16,
}

/// One nav-row control button. From the capture these are GRAY
/// bevelled panels (`c=0x4210 s=P_SOLID_FILL|P_BEVEL`), same style as the
/// bottom Back/Next bar — not bare text. Enabled text is flat cyan; a
/// disabled control uses the engraved two-pass shadow/dark-main rule.
fn nav_button(
    surface: &mut PackedSurface, font: &crate::packed_glyph::PixelFont,
    palette: PanelPalette, x0: i32, x1: i32, label: &str, enabled: bool,
) {
    let y0 = 125;
    let y1 = 145;
    draw_panel(surface, x0, y0, x1, y1, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    if enabled {
        draw_wrapped_text(surface, x0, y0, x1, y1, font,
            &c_string(label.as_bytes()), INK_GREY, TS_CENTRE, -1);
    } else {
        draw_wrapped_text(surface, x0 + 1, y0 + 1, x1 + 1, y1 + 1, font,
            &c_string(label.as_bytes()), DIS_SHADOW, TS_CENTRE, -1);
        draw_wrapped_text(surface, x0, y0, x1, y1, font,
            &c_string(label.as_bytes()), DIS_MAIN, TS_CENTRE, -1);
    }
}

/// Draw the Next Match content over the shared club chrome.
///
/// `render_chrome` is expected to have painted the photo, sidebar, red
/// title bar with `club_name`, the top tab row (Next Match selected),
/// and the bottom tab + Back/Next bar — the same chrome the Squad screen
/// uses. This function paints only the screen-specific middle.
pub fn render_next_match_body(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &NextMatchState<'_>,
) {
    let palette = PanelPalette::default();
    let body_font = fonts.pixel_slot(crate::screen_pre_boot_chrome::F_BODY).clone();
    let small_font = fonts.pixel_slot(crate::screen_pre_boot_chrome::F_SMALL).clone();

    // ---- Background structure, verbatim from the capture. Two SEPARATE
    // darkened blocks over the photo (NOT the whole screen): the title
    // band and the main content panel. The 5px gap between them (185 →
    // 190) is what visually separates the opponent-name block from the
    // detail block.
    //   darken (110,150)-(780,185)   title band
    //   PANEL  (110,190)-(780,500) s=2 + darken   main content panel
    draw_panel(surface, 110, 150, 780, 185, P_DARKEN, 0, 0, palette);
    draw_panel(surface, 110, 190, 780, 500, P_SOLID_FILL | P_BEVEL, 0, 0, palette);
    draw_panel(surface, 110, 190, 780, 500, P_DARKEN, 0, 0, palette);

    // Nav row buttons — gray bevelled panels.
    nav_button(surface, &body_font, palette, 110, 234, "<< Date", state.has_earlier);
    nav_button(surface, &body_font, palette, 236, 360, "Date >>", state.has_later);
    if !state.is_friendly {
        nav_button(surface, &body_font, palette, 530, 654, "Progress", true);
    }
    nav_button(surface, &body_font, palette, 656, 780, "Past Meetings", true);

    // Title band — opponent (Home/Away), yellow, centred.
    draw_wrapped_text(surface, 110, 150, 780, 185, &body_font,
        &c_string(state.title.as_bytes()), INK_YELLOW, TS_CENTRE, -1);

    // Competition label — orange, left — with the orange underline bar
    // beneath it (capture: PANEL (115,220)-(252,239) c=0x7e00). The bar
    // width tracks the label text so it underlines exactly, as the exe
    // sizes it to the competition name.
    draw_wrapped_text(surface, 115, 198, 400, 218, &body_font,
        &c_string(state.competition.as_bytes()), INK_ORANGE, W_LEFT, -1);
    let comp_w = crate::packed_text::measure_line(&body_font,
        &c_string(state.competition.as_bytes()));
    draw_panel(surface, 115, 220, 115 + comp_w.max(1), 224,
        P_SOLID_FILL, INK_ORANGE, 0, palette);

    // Detail block. Labels carry two leading spaces INSIDE the string
    // (from the capture), so we pass them literally.
    let label = |surface: &mut PackedSurface, y: i32, text: &str| {
        draw_wrapped_text(surface, 112, y, 352, y + 19, &small_font,
            &c_string(text.as_bytes()), INK_GREY, W_LEFT, -1);
    };
    let value = |surface: &mut PackedSurface, y: i32, text: &str| {
        draw_wrapped_text(surface, 354, y, 756, y + 19, &small_font,
            &c_string(text.as_bytes()), INK_YELLOW, W_LEFT, -1);
    };

    label(surface, 241, "  Date");
    value(surface, 241, state.date_line);
    label(surface, 262, "  Venue");
    value(surface, 262, state.venue);
    label(surface, 283, "  Match Rules");
    value(surface, 283, state.match_rules[0]);
    value(surface, 304, state.match_rules[1]);
    label(surface, 325, "  Last Meeting");
    value(surface, 325, state.last_meeting);
    label(surface, 346, "  Weather Forecast");
    value(surface, 346, state.weather);

    // News label uses the club short name.
    let news_label = format!("  {} News", state.club_name);
    label(surface, 367, &news_label);

    // Availability rows: value column x 355..758, 21px pitch from 367.
    // The panel bottom is ~492; 6 rows fit (367..492).
    const NEWS_Y0: i32 = 367;
    const NEWS_STRIDE: i32 = 21;
    const NEWS_VISIBLE: usize = 6;
    for (i, line) in state.news.iter().skip(state.news_scroll).take(NEWS_VISIBLE).enumerate() {
        let y = NEWS_Y0 + (i as i32) * NEWS_STRIDE;
        draw_wrapped_text(surface, 355, y, 758, y + 19, &small_font,
            &c_string(line.as_bytes()), INK_YELLOW, W_LEFT, -1);
    }

    // Availability scrollbar when the list overflows.
    if state.news.len() > NEWS_VISIBLE {
        // The exe puts a scrollbar in the right margin of the news
        // panel. Reuse the canonical primitive at this screen's bar
        // position (from the capture: x ~760..778, y ~198..492).
        let bar = crate::scrollbar::Scrollbar::new(198, 492);
        bar.draw(surface, state.news.len(), NEWS_VISIBLE, state.news_scroll,
            GREY_BAR, palette);
    }

    // Consume the unused-import lint paths for panel drawing helpers that
    // the chrome (drawn by the caller) already used; keep them in scope
    // for future field panels.
    let _ = (P_DARKEN, P_SOLID_FILL, P_BEVEL, &palette);
}
