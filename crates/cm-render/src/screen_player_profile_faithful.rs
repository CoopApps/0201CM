//! Player Profile screen (default page on clicking a squad player) —
//! faithful to `fixtures/player_profile_screen/player_profile.json`
//! (decode: `reports/player_profile_screen_decode.md`).
//!
//! Own frame (not the club chrome): navy title bar with the player's
//! name, five player subtabs (Profile active), a bio line, a 3×12
//! attribute grid, a career-stats table, a position band, and Back/Next.

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{draw_panel, PanelPalette, P_DARKEN, P_SOLID_FILL, P_BEVEL, P_OUTER_HIGHLIGHT};
use crate::packed_text::{draw_wrapped_text, W_LEFT};
use crate::screen_pre_boot_chrome::{c_string, blit_photo, draw_sidebar, TS_CENTRE, F_SMALL, F_BODY, F_TITLE};

const INK_GREY:   u16 = 0x739c;
const INK_YELLOW: u16 = 0x7fe0;
const INK_ORANGE: u16 = 0x7e00;
const INK_TEAL:   u16 = 0x43ff;
const INK_WHITE:  u16 = 0x7fff;
const NAVY:       u16 = 0x0090;
const TAB_BLUE:   u16 = 0x100c;
const GREY_BAR:   u16 = 0x4210;
/// Purple Av-R cell box (capture col 8200 = 0x2008).
const AVR_BOX:    u16 = 0x2008;

/// The five player subtabs.
pub const PROFILE_SUBTABS: [&str; 5] =
    ["Profile", "Injuries & Bans", "Contract", "Transfer", "History"];
/// Left x of each subtab (each 137 wide, gap of 2).
const SUBTAB_X: [i32; 5] = [100, 239, 377, 515, 653];

/// Attribute grid row tops (12 rows).
const ATTR_ROW_Y: [i32; 12] =
    [160, 179, 197, 216, 234, 252, 271, 289, 307, 326, 344, 362];
/// Career-stats row tops (6 rows).
const CAREER_ROW_Y: [i32; 6] = [403, 421, 439, 457, 475, 493];
/// Career stat columns (x0,x1) for Apps..Av R — each a boxed cell
/// (capture-verified bounds).
const CAREER_COLS: [(i32, i32); 9] = [
    (317, 367), (369, 418), (420, 470), (472, 521), (523, 573),
    (575, 625), (627, 676), (678, 728), (730, 780),
];
/// The two nav-arrow header boxes.
const NAV_LT: (i32, i32) = (265, 289);
const NAV_GT: (i32, i32) = (291, 315);
/// Header nav-arrow box colour (capture col 16 = 0x0010, dark blue).
const HDR_BLUE: u16 = 0x0010;
const CAREER_HDR: [&str; 9] =
    ["Apps", "Con", "Asts", "MoM", "Pass ", "Tck", "Drb", "Sh Tar", "Av R"];

/// Everything the Profile page paints (borrowed from `PlayerProfileView`).
pub struct PlayerProfileState<'a> {
    pub title: &'a str,
    pub born_line: &'a str,
    /// 31 attribute value strings, grid order (col1 0..12, col2 12..24,
    /// col3 24..31).
    pub attributes: &'a [String],
    /// Preferred Foot, Form, Morale, Condition.
    pub status: &'a [String; 4],
    pub position: &'a str,
    /// Six career rows: (label, 9 cells).
    pub career: &'a [(&'a str, [&'a str; 9])],
    /// Active subtab index (0 = Profile).
    pub active_subtab: usize,
    /// Darkened-photo background seed (matches the other club screens).
    pub photo_seed: u64,
    /// Whether a manager is installed (drives the sidebar's Add-Manager
    /// enabled/faded state).
    pub has_manager: bool,
}

/// The 31 attribute labels, grid order (mirrors
/// `cm_domain::player_profile::PROFILE_ATTR_LABELS`).
const ATTR_LABELS: [&str; 31] = [
    "Acceleration", "Aggression", "Agility", "Anticipation", "Balance",
    "Bravery", "Creativity", "Crossing", "Decisions", "Determination",
    "Dribbling", "Finishing", "Flair", "Handling", "Heading", "Influence",
    "Jumping", "Long Shots", "Marking", "Off The Ball", "Pace", "Passing",
    "Positioning", "Reflexes", "Set Pieces", "Stamina", "Strength",
    "Tackling", "Teamwork", "Technique", "Work Rate",
];
const STATUS_LABELS: [&str; 4] = ["Preferred Foot", "Form", "Morale", "Condition"];

pub fn render_player_profile(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    st: &PlayerProfileState<'_>,
) {
    let palette = PanelPalette::default();
    let small = fonts.pixel_slot(F_SMALL).clone();
    let body  = fonts.pixel_slot(F_BODY).clone();
    let title_font = fonts.pixel_slot(F_TITLE).clone();
    let cell  = fonts.pixel_slot(2).clone();

    // ---- Darkened-photo background (the P_DARKEN panels below show it
    //      through, exactly like the Squad / General Info screens). ----
    blit_photo(surface, st.photo_seed);
    // ---- Left menu sidebar (Version / nav / File menu) — always on. ----
    draw_sidebar(surface, fonts, st.has_manager);

    // ---- Title bar (navy banner + name + Action button) ----
    draw_panel(surface, 100, 10, 790, 70, P_SOLID_FILL | P_BEVEL, NAVY, 0, palette);
    // Name centred across the banner (excluding the left nav button and
    // the Action button's column) so long names/clubs stay centred and
    // unclipped.
    draw_wrapped_text(surface, 120, 20, 790, 60, &title_font,
        &c_string(st.title.as_bytes()), INK_WHITE, TS_CENTRE, -1);
    draw_panel(surface, 660, 4, 785, 24, P_SOLID_FILL | P_BEVEL, INK_WHITE, 0, palette);
    draw_wrapped_text(surface, 660, 4, 785, 24, &small,
        &c_string(b"Action"), 0x0000, TS_CENTRE, -1);

    // ---- Subtabs ----
    for (i, label) in PROFILE_SUBTABS.iter().enumerate() {
        let x0 = SUBTAB_X[i];
        let x1 = x0 + 137;
        let active = i == st.active_subtab;
        let style = if active {
            P_SOLID_FILL | P_BEVEL | P_OUTER_HIGHLIGHT
        } else {
            P_SOLID_FILL | P_BEVEL
        };
        draw_panel(surface, x0, 80, x1, 115, style, TAB_BLUE, 0, palette);
        // Active tab carries a 1px yellow outline 1px outside the tab
        // (capture rect (99,79)-(238,116) col 0x7fe0).
        if active {
            let (ax0, ay0, ax1, ay1) = (x0 - 1, 79, x1 + 1, 116);
            surface.draw_line(ax0, ay0, ax1, ay0, 2, INK_YELLOW);
            surface.draw_line(ax0, ay1, ax1, ay1, 2, INK_YELLOW);
            surface.draw_line(ax0, ay0, ax0, ay1, 2, INK_YELLOW);
            surface.draw_line(ax1, ay0, ax1, ay1, 2, INK_YELLOW);
        }
        let ink = if active { INK_YELLOW } else { INK_GREY };
        draw_wrapped_text(surface, x0, 80, x1, 115, &small,
            &c_string(label.as_bytes()), ink, TS_CENTRE, -1);
    }

    // ---- Bio band ----
    draw_panel(surface, 110, 125, 780, 155, P_DARKEN, 0, 0, palette);
    // Widened right edge so nationality ("English.") is not clipped.
    draw_wrapped_text(surface, 313, 127, 700, 153, &body,
        &c_string(st.born_line.as_bytes()), INK_YELLOW, W_LEFT, -1);

    // ---- Attribute grid ----
    draw_panel(surface, 110, 160, 780, 379, P_DARKEN, 0, 0, palette);
    // Column geometry: (label_x, value_x).
    let cols = [(110i32, 245i32), (334, 468), (558, 692)];
    // Col 1 = attrs 0..12, col 2 = 12..24, col 3 = 24..31 then status.
    let draw_pair = |surface: &mut PackedSurface, lx: i32, vx: i32, y: i32,
                     label: &str, value: &str, val_ink: u16| {
        draw_wrapped_text(surface, lx + 2, y, lx + 133, y + 17, &cell,
            &c_string(format!("  {label}").as_bytes()), INK_GREY, W_LEFT, -1);
        draw_wrapped_text(surface, vx, y, vx + 86, y + 17, &cell,
            &c_string(value.as_bytes()), val_ink, W_LEFT, -1);
    };
    // Col 1 + col 2 (12 rows each, numeric).
    for row in 0..12 {
        let y = ATTR_ROW_Y[row];
        for (col, &(lx, vx)) in cols.iter().enumerate().take(2) {
            let idx = col * 12 + row;
            if idx < st.attributes.len() && idx < ATTR_LABELS.len() {
                draw_pair(surface, lx, vx, y, ATTR_LABELS[idx],
                    &st.attributes[idx], INK_YELLOW);
            }
        }
    }
    // Col 3: rows 0..7 attrs (24..31), rows 7..11 status.
    let (lx3, vx3) = cols[2];
    for row in 0..7 {
        let idx = 24 + row;
        if idx < st.attributes.len() {
            draw_pair(surface, lx3, vx3, ATTR_ROW_Y[row], ATTR_LABELS[idx],
                &st.attributes[idx], INK_YELLOW);
        }
    }
    for (k, slabel) in STATUS_LABELS.iter().enumerate() {
        draw_pair(surface, lx3, vx3, ATTR_ROW_Y[7 + k], slabel,
            &st.status[k], INK_ORANGE);
    }

    // ---- Career-stats table ----
    draw_panel(surface, 110, 384, 780, 510, P_DARKEN, 0, 0, palette);
    // Header row — every cell is a bevelled box with a centred grey
    // label: << / >> on BLUE boxes (col 16), the nine stat headers on
    // GREY boxes (col 0x4210). (Capture verified.)
    let nav = [(NAV_LT, "<<"), (NAV_GT, ">>")];
    for ((cx0, cx1), lbl) in nav {
        draw_panel(surface, cx0, 384, cx1, 401, P_SOLID_FILL | P_BEVEL, HDR_BLUE, 0, palette);
        draw_wrapped_text(surface, cx0, 384, cx1, 401, &small,
            &c_string(lbl.as_bytes()), INK_GREY, TS_CENTRE, -1);
    }
    for (i, h) in CAREER_HDR.iter().enumerate() {
        let (cx0, cx1) = CAREER_COLS[i];
        draw_panel(surface, cx0, 384, cx1, 401, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
        draw_wrapped_text(surface, cx0, 384, cx1, 401, &small,
            &c_string(h.as_bytes()), INK_GREY, TS_CENTRE, -1);
    }
    // Rows — centred values; Av R column carries a purple bevelled box.
    for (r, (label, cells)) in st.career.iter().enumerate().take(6) {
        let y = CAREER_ROW_Y[r];
        // Alternating band colour (even rows yellow, odd orange).
        let ink = if r % 2 == 0 { INK_YELLOW } else { INK_ORANGE };
        draw_wrapped_text(surface, 112, y, 263, y + 16, &cell,
            &c_string(format!("  {label}").as_bytes()), INK_GREY, W_LEFT, -1);
        for (i, cellv) in cells.iter().enumerate() {
            let (cx0, cx1) = CAREER_COLS[i];
            if i == 8 {
                draw_panel(surface, cx0, y, cx1, y + 16, P_SOLID_FILL | P_BEVEL, AVR_BOX, 0, palette);
            }
            let cink = if i == 8 { INK_TEAL } else { ink };
            draw_wrapped_text(surface, cx0, y, cx1, y + 16,
                &small, &c_string(cellv.as_bytes()), cink, TS_CENTRE, -1);
        }
    }

    // ---- Position band ----
    draw_panel(surface, 110, 515, 780, 545, P_DARKEN, 0, 0, palette);
    draw_wrapped_text(surface, 110, 515, 780, 545, &body,
        &c_string(st.position.as_bytes()), INK_TEAL, TS_CENTRE, -1);

    // ---- Back / Next ----
    draw_panel(surface, 100, 555, 617, 590, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_wrapped_text(surface, 100, 555, 617, 590, &body,
        &c_string(b"Back"), INK_GREY, TS_CENTRE, -1);
    draw_panel(surface, 619, 555, 790, 590, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_wrapped_text(surface, 619, 555, 790, 590, &body,
        &c_string(b"Next"), INK_GREY, TS_CENTRE, -1);
}
