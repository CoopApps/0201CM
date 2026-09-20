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
/// Bright cyan used for the History Loan marker and Total row (col 1023).
const INK_HIST_CYAN: u16 = 0x03ff;
/// Selected-season header blue in the History bottom table (col 543).
const HIST_SEL_BLUE: u16 = 0x021f;

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
/// Injuries & Bans detail labels (col at x=110), top→bottom.
const INJURY_LABELS: [&str; 5] = ["Injury", "Type", "Condition", "Training", "Bans"];
/// Contract detail labels (col at x=110), top→bottom.
const CONTRACT_LABELS: [&str; 7] =
    ["Type", "Wages", "Expires", "Squad Status", "Bonuses", "Clauses", "Notes"];
/// Transfer detail labels (col at x=110); the last is blank — the Future
/// text wraps onto a second, label-less row.
const TRANSFER_LABELS: [&str; 7] =
    ["Availability", "Value", "Fluent Languages", "Offers", "Interested", "Future", ""];

/// Which History top-table view is active (View dropdown). Values match
/// the exe's View-menu order.
pub const HV_ACHIEVEMENTS: usize = 0;
pub const HV_PLAYING_CAREER: usize = 1;
pub const HV_INJURIES: usize = 2;
pub const HV_BANS: usize = 3;

/// Which pop-up menu is open over the Player Profile screen.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProfileMenu {
    None,
    /// Name-bar triangle → club squad list.
    Picker,
    /// View dropdown (Achievements / Playing Career / Injuries / Bans).
    View,
    /// Filter dropdown (All Records / Competitions / Awards).
    Filter,
    /// Action button menu.
    Action,
    /// Action → Compare Players submenu.
    Compare,
}

/// One row of the History → Achievements list.
pub struct AchievementRow<'a> {
    pub date: &'a str,
    pub club: &'a str,
    pub text: &'a str,
}

/// One row of the History → Injuries list.
pub struct InjuryRow<'a> {
    pub date: &'a str,
    pub injury: &'a str,
    pub kind: &'a str,     // "Match" / "Training"
    pub period: &'a str,   // "3 weeks" / "10 days"
}

/// One row of the History → Bans list.
pub struct BanRow<'a> {
    pub date: &'a str,
    pub ban: &'a str,      // "3 match English ban"
    pub reason: &'a str,   // "Red card" / "5 yellow cards"
}

/// One season row for the History subtab's "Playing Career" table.
pub struct HistoryRow<'a> {
    pub season: &'a str,
    pub club: &'a str,
    pub is_loan: bool,
    pub apps: &'a str,
    pub goals: &'a str,
}

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
    /// Active subtab index (0 = Profile, 1 = Injuries & Bans).
    pub active_subtab: usize,
    /// Injuries & Bans values: Injury, Type, Condition, Training, Bans.
    pub injuries: &'a [String; 5],
    /// Contract values: Type, Wages, Expires, Squad Status, Bonuses,
    /// Clauses, Notes.
    pub contract: &'a [String; 7],
    /// Transfer values: Availability, Value, Fluent Languages, Offers,
    /// Interested, Future(1), Future(2).
    pub transfer: &'a [String; 7],
    /// True for a goalkeeper (career col-2 header = "Con" not "Gls").
    pub is_goalkeeper: bool,
    /// Title-bar fill = the player's club kit colour (navy for Bury,
    /// red for Dag & Red, …); 0 falls back to navy.
    pub kit_bg: u16,
    /// Darkened-photo background seed (matches the other club screens).
    pub photo_seed: u64,
    /// Whether a manager is installed (drives the sidebar's Add-Manager
    /// enabled/faded state).
    pub has_manager: bool,
    // ---- History subtab (active_subtab == 4) ----
    /// Season rows for the "Playing Career" table (newest first).
    pub history_rows: &'a [HistoryRow<'a>],
    /// Total apps / goals (teal Total row).
    pub history_total: (&'a str, &'a str),
    /// Bottom-table header, e.g. "  2001/2 Arsenal" (selected season).
    pub history_selected_label: &'a str,
    /// Index of the selected season (drives the bottom table).
    pub history_selected_idx: usize,
    /// First list index shown in the top table's scroll window.
    pub history_scroll: usize,
    /// Top-table stat page (0 = Apps/Gls…, 1 = Con/Pens…). The two tables
    /// page INDEPENDENTLY via their own `<<`/`>>` buttons.
    pub history_page: usize,
    /// Bottom-table (selected-season breakdown) stat page.
    pub history_bot_page: usize,
    /// Active top-table view (HV_* — Achievements/Playing Career/…).
    pub history_view: usize,
    /// Achievements Filter mode (0 All Records, 1 Competitions, 2 Awards).
    pub history_filter: usize,
    /// Which pop-up menu is currently open (drawn last, over everything).
    pub open_menu: ProfileMenu,
    /// Player-picker rows ("Surname, Initial"), club squad, alphabetical.
    pub picker_items: &'a [String],
    /// Achievements-view rows (Date · Club · text), newest first. Empty
    /// until honours accrue during play.
    pub achievements: &'a [AchievementRow<'a>],
    /// Injuries-view rows (Date · Injury · Type · Period Out), newest first.
    pub injuries_list: &'a [InjuryRow<'a>],
    /// Bans-view rows (Date · Ban · Reason), newest first.
    pub bans_list: &'a [BanRow<'a>],
}

/// Green pop-up menu fills (exe cols 512 / 576) and text.
const MENU_GREEN_A: u16 = 0x0200;
const MENU_GREEN_B: u16 = 0x0240;
const MENU_INK: u16 = 0x0000;
/// The name-bar triangle button that opens the squad picker.
const PICKER_BTN: (i32, i32, i32, i32) = (103, 15, 120, 35);

/// Top "Playing Career" table — 10 visible row slots (verified capture
/// y's; stride ~19.4). The list is `history_rows` + a Total row.
const HIST_ROW_Y: [i32; 10] =
    [185, 205, 225, 244, 264, 283, 303, 322, 342, 361];
/// Top-table cell columns (x0,x1): Season, Club, Loan, `<<`, `>>`, then
/// Apps, Gls, Asts, MoM, Pass, Tck, Drb, Sh Tar, Av R.
const HIST_SEASON_COL: (i32, i32) = (110, 178);
const HIST_CLUB_COL:   (i32, i32) = (180, 317);
const HIST_LOAN_COL:   (i32, i32) = (319, 377);
const HIST_STAT_COLS: [(i32, i32); 9] = [
    (426, 461), (463, 498), (500, 535), (537, 572), (574, 609),
    (611, 646), (648, 683), (685, 720), (722, 758),
];
/// Scrollbar geometry (x=761-780).
const HIST_SB_X0: i32 = 761;
const HIST_SB_X1: i32 = 780;
const HIST_SB_UP:   (i32, i32) = (185, 204);
const HIST_SB_TRK:  (i32, i32) = (205, 359);
const HIST_SB_DOWN: (i32, i32) = (360, 379);
/// The `<<` / `>>` stat-page nav boxes in the top-table header row.
const HIST_HDR_LT: (i32, i32) = (379, 401);
const HIST_HDR_GT: (i32, i32) = (403, 424);
/// Selected-season season-box fill (grey; the text then turns navy). The
/// non-selected fill is `HDR_BLUE` with grey text — the two invert.
const SEASON_SEL_BG: u16 = 0x739c;

/// The nine stat-column headers for the History tables, decoded from the
/// exe's per-(page,position) column-code arrays at `.data` 0xa706bc..e0
/// (see `reports/player_history_builder_decode.md`). `<<`/`>>` page
/// between the two. Codes → labels: 1 Apps, 2 Gls, 3 Con, 4 Pens, 5 Asts,
/// 6 Tgls, 7 Tcon, 8 Won, 9 Lost, a Yel, b Red, c MoM, d Pass, e Tck,
/// f Drb, 10 Sh Tar, 11 Av R.
pub fn history_headers(is_goalkeeper: bool, page: usize) -> [&'static str; 9] {
    match (is_goalkeeper, page % 2) {
        // Outfield page 0 (default) / page 1.
        (false, 0) => ["Apps", "Gls", "Asts", "MoM", "Pass", "Tck", "Drb", "Sh Tar", "Av R"],
        (false, _) => ["Con", "Pens", "Yel", "Red", "Won", "Lost", "Tgls", "Tcon", "Av R"],
        // Goalkeeper page 0 (Con in the Gls slot) / page 1.
        (true, 0)  => ["Apps", "Con", "Asts", "MoM", "Pass", "Tck", "Drb", "Sh Tar", "Av R"],
        (true, _)  => ["Gls", "Pens", "Yel", "Red", "Won", "Lost", "Tgls", "Tcon", "Av R"],
    }
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

    // ---- Title bar (kit-coloured banner + name + Action button) ----
    let bar = if st.kit_bg != 0 { st.kit_bg } else { NAVY };
    draw_panel(surface, 100, 10, 790, 70, P_SOLID_FILL | P_BEVEL, bar, 0, palette);
    // Name centred across the banner (excluding the left nav button and
    // the Action button's column) so long names/clubs stay centred and
    // unclipped.
    draw_wrapped_text(surface, 120, 20, 790, 60, &title_font,
        &c_string(st.title.as_bytes()), INK_WHITE, TS_CENTRE, -1);
    draw_panel(surface, 660, 4, 785, 24, P_SOLID_FILL | P_BEVEL, INK_WHITE, 0, palette);
    draw_wrapped_text(surface, 660, 4, 785, 24, &small,
        &c_string(b"Action"), 0x0000, TS_CENTRE, -1);
    // Name-bar triangle button (opens the club squad player-picker).
    {
        let (bx0, by0, bx1, by1) = PICKER_BTN;
        draw_panel(surface, bx0, by0, bx1, by1, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
        // Right-pointing triangle glyph.
        let cx = (bx0 + bx1) / 2 - 2;
        let cy = (by0 + by1) / 2;
        for i in 0..5i32 {
            let h = 4 - i;
            surface.draw_line(cx + i, cy - h, cx + i, cy + h, 2, INK_GREY);
        }
    }

    // ---- Subtabs (all panels + labels first) ----
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
        let ink = if active { INK_YELLOW } else { INK_GREY };
        draw_wrapped_text(surface, x0, 80, x1, 115, &small,
            &c_string(label.as_bytes()), ink, TS_CENTRE, -1);
    }
    // Active-tab yellow outline drawn AFTER every tab panel, so the next
    // tab's panel cannot paint over its right edge (capture rect
    // (99,79)-(238,116) col 0x7fe0).
    {
        let x0 = SUBTAB_X[st.active_subtab.min(4)];
        let (ax0, ay0, ax1, ay1) = (x0 - 1, 79, x0 + 138, 116);
        surface.draw_line(ax0, ay0, ax1, ay0, 2, INK_YELLOW);
        surface.draw_line(ax0, ay1, ax1, ay1, 2, INK_YELLOW);
        surface.draw_line(ax0, ay0, ax0, ay1, 2, INK_YELLOW);
        surface.draw_line(ax1, ay0, ax1, ay1, 2, INK_YELLOW);
    }

    if st.active_subtab == 4 {
        render_history(surface, &small, &body, &title_font, &cell, st, palette);
    } else {
    // ---- Bio band ----
    draw_panel(surface, 110, 125, 780, 155, P_DARKEN, 0, 0, palette);
    // Widened right edge so nationality ("English.") is not clipped.
    draw_wrapped_text(surface, 313, 127, 700, 153, &body,
        &c_string(st.born_line.as_bytes()), INK_YELLOW, W_LEFT, -1);

    // ---- Middle content: per-cell darkened grid (the 2px gaps between
    //      cells show the photo brighter, giving the table's grid lines).
    //      One darkened cell per label and value box, exactly like the
    //      capture (no solid fill — the photo shows through). Each cell's
    //      bottom is the next row's top minus 2, so EVERY row has a 2px
    //      divider gap (the row stride alternates 18/19, so a fixed cell
    //      height would only divide every other row). ----
    let cell_bottom = |row: usize| -> i32 {
        if row + 1 < ATTR_ROW_Y.len() { ATTR_ROW_Y[row + 1] - 2 } else { ATTR_ROW_Y[row] + 17 }
    };
    let spans: &[(i32, i32)] = if (1..=3).contains(&st.active_subtab) {
        // Injuries / Contract / Transfer: 2 columns, label + value.
        &[(110, 258), (260, 780)]
    } else {
        // Profile: 6 cells per row — label+value for each of 3 columns.
        &[(110, 243), (245, 332), (334, 466), (468, 556), (558, 690), (692, 780)]
    };
    for (row, &y) in ATTR_ROW_Y.iter().enumerate() {
        let y1 = cell_bottom(row);
        for &(x0, x1) in spans {
            draw_panel(surface, x0, y, x1, y1, P_DARKEN, 0, 0, palette);
        }
    }
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
    // Detail-page row: grey label at x=112, WIDE yellow value at x=260
    // running to the panel edge (values like "Part Time Contract" or
    // "This player is an important first team player" must not clip).
    let draw_detail = |s: &mut PackedSurface, y: i32, label: &str, value: &str| {
        draw_wrapped_text(s, 112, y, 258, y + 17, &cell,
            &c_string(format!("  {label}").as_bytes()), INK_GREY, W_LEFT, -1);
        draw_wrapped_text(s, 260, y, 778, y + 17, &cell,
            &c_string(format!("  {value}").as_bytes()), INK_YELLOW, W_LEFT, -1);
    };
    if st.active_subtab == 1 {
        // ---- Injuries & Bans: 2-column detail block, 5 rows. ----
        for (row, (label, value)) in INJURY_LABELS.iter().zip(st.injuries.iter()).enumerate() {
            draw_detail(surface, ATTR_ROW_Y[row], label, value);
        }
    } else if st.active_subtab == 2 {
        // ---- Contract: 2-column detail block, 7 rows. ----
        for (row, (label, value)) in CONTRACT_LABELS.iter().zip(st.contract.iter()).enumerate() {
            draw_detail(surface, ATTR_ROW_Y[row], label, value);
        }
    } else if st.active_subtab == 3 {
        // ---- Transfer: 2-column detail block, 7 rows. ----
        for (row, (label, value)) in TRANSFER_LABELS.iter().zip(st.transfer.iter()).enumerate() {
            draw_detail(surface, ATTR_ROW_Y[row], label, value);
        }
    } else {
        // ---- Profile: 3-column attribute grid. ----
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
    }
    } // end subtabs 0..3 middle content

    // ---- Career-stats table (shared: profile career, or — for the
    //      History subtab — the selected season's per-competition
    //      breakdown). Rows come from `st.career`. ----
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
    // History subtab: the nine headers follow the bottom table's OWN stat
    // page (it pages independently of the top table). Other subtabs use
    // the fixed profile-career headers.
    let hist_hdr = history_headers(st.is_goalkeeper, st.history_bot_page);
    for i in 0..9 {
        let (cx0, cx1) = CAREER_COLS[i];
        let label: &str = if st.active_subtab == 4 {
            hist_hdr[i]
        } else if i == 1 && !st.is_goalkeeper {
            "Gls"
        } else {
            CAREER_HDR[i]
        };
        draw_panel(surface, cx0, 384, cx1, 401, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
        draw_wrapped_text(surface, cx0, 384, cx1, 401, &small,
            &c_string(label.as_bytes()), INK_GREY, TS_CENTRE, -1);
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
    // History subtab: the bottom table's header carries the selected
    // season label ("  2001/2 Arsenal", blue) at the left of the header
    // row, where the profile career table leaves it blank.
    if st.active_subtab == 4 {
        draw_wrapped_text(surface, 112, 384, 263, 401, &cell,
            &c_string(st.history_selected_label.as_bytes()), HIST_SEL_BLUE, W_LEFT, -1);
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

    // ---- Pop-up menus (drawn LAST so they overlay everything) ----
    if st.active_subtab == 4 && st.open_menu != ProfileMenu::None {
        draw_profile_menu(surface, &small, st, palette);
    }
}

/// Draw a green pop-up menu at (x0,y0) with the given items and an
/// optional checked index. Rows are ~21px, zebra 512/576, 5-space indent.
#[allow(clippy::too_many_arguments)]
fn draw_menu_box(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    x0: i32, y0: i32, x1: i32,
    items: &[&str],
    checked: Option<usize>,
    palette: PanelPalette,
) {
    let pitch = 21;
    let y1 = y0 + 2 + pitch * items.len() as i32;
    draw_panel(surface, x0, y0, x1, y1, P_SOLID_FILL | P_BEVEL, MENU_GREEN_A, 0, palette);
    for (i, item) in items.iter().enumerate() {
        let iy = y0 + 2 + pitch * i as i32;
        let fill = if i % 2 == 0 { MENU_GREEN_A } else { MENU_GREEN_B };
        draw_panel(surface, x0 + 2, iy, x1 - 2, iy + 19, P_SOLID_FILL, fill, 0, palette);
        // Check mark on the active item (in the 5-space indent gap): a
        // small two-stroke tick drawn in the menu ink.
        if checked == Some(i) {
            let (tx, ty) = (x0 + 10, iy + 10);
            surface.draw_line(tx, ty, tx + 3, ty + 3, 2, MENU_INK);
            surface.draw_line(tx + 3, ty + 3, tx + 9, ty - 4, 2, MENU_INK);
            surface.draw_line(tx, ty + 1, tx + 3, ty + 4, 2, MENU_INK);
            surface.draw_line(tx + 3, ty + 4, tx + 9, ty - 3, 2, MENU_INK);
        }
        draw_wrapped_text(surface, x0 + 26, iy, x1 - 2, iy + 19, font,
            &c_string(item.as_bytes()), MENU_INK, W_LEFT, -1);
    }
}

/// Render whichever pop-up menu is open over the History subtab.
fn draw_profile_menu(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    st: &PlayerProfileState<'_>,
    palette: PanelPalette,
) {
    match st.open_menu {
        ProfileMenu::Picker => {
            // Club squad list, anchored at the triangle button, auto-sized.
            let items: Vec<&str> = st.picker_items.iter().map(|s| s.as_str()).collect();
            draw_menu_box(surface, font, 123, 0, 248, &items, None, palette);
        }
        ProfileMenu::View => {
            // Order matches the exe: Achievements / Playing Career /
            // Injuries / Bans. Check the active view.
            let items = ["Achievements", "Playing Career", "Injuries", "Bans"];
            let checked = match st.history_view {
                HV_ACHIEVEMENTS => 0, HV_INJURIES => 2, HV_BANS => 3, _ => 1,
            };
            draw_menu_box(surface, font, 110, 148, 235, &items, Some(checked), palette);
        }
        ProfileMenu::Filter => {
            let items = ["All Records", "Competitions", "Awards"];
            draw_menu_box(surface, font, 236, 148, 361, &items, Some(st.history_filter.min(2)), palette);
        }
        ProfileMenu::Action => {
            // Non-manager player Action menu (separator handled by a blank).
            let items = ["Add To Shortlist", "Set Nickname", "", "Create Manager Note", "Compare Players"];
            draw_menu_box(surface, font, 635, 27, 785, &items, None, palette);
        }
        ProfileMenu::Compare => {
            // Action menu + its Compare submenu (opening to the left).
            let items = ["Add To Shortlist", "Set Nickname", "", "Create Manager Note", "Compare Players"];
            draw_menu_box(surface, font, 635, 27, 785, &items, None, palette);
            let sub = ["Set as 1st player in comparison", "Set as 2nd player in comparison", "Compare two chosen players"];
            draw_menu_box(surface, font, 427, 111, 634, &sub, None, palette);
        }
        ProfileMenu::None => {}
    }
}

/// A small filled arrow triangle centred on `cx`, 5px tall, pointing up
/// (`up`) or down, top at `top`.
fn draw_tri(surface: &mut PackedSurface, cx: i32, top: i32, up: bool, colour: u16) {
    for i in 0..5i32 {
        let w = if up { i } else { 4 - i };
        surface.draw_line(cx - w, top + i, cx + w, top + i, 2, colour);
    }
}

/// History subtab: View dropdown, "Playing Career" top table (scrollable
/// season list + Total), and the scrollbar. The bottom per-competition
/// breakdown reuses the shared career table (driven by `st.career`).
#[allow(clippy::too_many_arguments)]
fn render_history(
    surface: &mut PackedSurface,
    small: &crate::packed_glyph::PixelFont,
    body: &crate::packed_glyph::PixelFont,
    _title_font: &crate::packed_glyph::PixelFont,
    cell: &crate::packed_glyph::PixelFont,
    st: &PlayerProfileState<'_>,
    palette: PanelPalette,
) {
    // ---- View dropdown (bevelled grey box + label + down arrow) ----
    draw_panel(surface, 110, 125, 234, 145, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_wrapped_text(surface, 122, 125, 210, 145, cell,
        &c_string(b"View"), INK_GREY, W_LEFT, -1);
    draw_tri(surface, 222, 132, false, INK_GREY);
    // Filter dropdown — only in the Achievements view.
    if st.history_view == HV_ACHIEVEMENTS {
        draw_panel(surface, 236, 125, 360, 145, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
        draw_wrapped_text(surface, 248, 125, 336, 145, cell,
            &c_string(b"Filter"), INK_GREY, W_LEFT, -1);
        draw_tri(surface, 348, 132, false, INK_GREY);
    }

    // ---- Title (per view) ----
    let title = match st.history_view {
        HV_ACHIEVEMENTS => "Achievements",
        HV_INJURIES => "Injuries",
        HV_BANS => "Bans",
        _ => "Playing Career",
    };
    draw_wrapped_text(surface, 110, 150, 780, 170, body,
        &c_string(title.as_bytes()), INK_YELLOW, TS_CENTRE, -1);

    // ---- Top table background ----
    draw_panel(surface, 110, 185, 780, 379, P_DARKEN, 0, 0, palette);

    // Records views. Achievements draws its accrued rows (Date | Club |
    // text); Injuries/Bans draw their header boxes + (empty until the data
    // is modelled) list. All get a scrollbar.
    if st.history_view != HV_PLAYING_CAREER {
        // Injuries/Bans header row (list-row 0); Achievements has none.
        let first_data_slot = match st.history_view {
            HV_INJURIES => {
                for (bx0, bx1, lbl) in [(460, 608, "Type"), (610, 758, "Period Out")] {
                    draw_panel(surface, bx0, 185, bx1, 203, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
                    draw_wrapped_text(surface, bx0, 185, bx1, 203, small,
                        &c_string(lbl.as_bytes()), INK_GREY, TS_CENTRE, -1);
                }
                1
            }
            HV_BANS => {
                draw_panel(surface, 610, 185, 758, 203, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
                draw_wrapped_text(surface, 610, 185, 758, 203, small,
                    &c_string(b"Reason"), INK_GREY, TS_CENTRE, -1);
                1
            }
            _ => 0, // Achievements: rows fill from the top slot
        };
        // Data rows fill from `first_data_slot` (0 for Achievements, 1 for
        // Injuries/Bans which reserve slot 0 for the header). Yellow row text.
        let visible = HIST_ROW_Y.len() - first_data_slot as usize;
        let rows_len = match st.history_view {
            HV_ACHIEVEMENTS => st.achievements.len(),
            HV_INJURIES => st.injuries_list.len(),
            HV_BANS => st.bans_list.len(),
            _ => 0,
        };
        let sc = st.history_scroll.min(rows_len.saturating_sub(visible));
        for slot in first_data_slot as usize..HIST_ROW_Y.len() {
            let y = HIST_ROW_Y[slot];
            let li = sc + (slot - first_data_slot as usize);
            if li >= rows_len { break; }
            let y1 = y + 18;
            // Common: date box + a darken strip across the row.
            draw_panel(surface, HIST_SEASON_COL.0, y, HIST_SEASON_COL.1, y1,
                P_SOLID_FILL | P_BEVEL, HDR_BLUE, 0, palette);
            draw_panel(surface, HIST_CLUB_COL.0, y, 758, y1, P_DARKEN, 0, 0, palette);
            let date = match st.history_view {
                HV_ACHIEVEMENTS => st.achievements[li].date,
                HV_INJURIES => st.injuries_list[li].date,
                _ => st.bans_list[li].date,
            };
            draw_wrapped_text(surface, HIST_SEASON_COL.0 + 13, y, HIST_SEASON_COL.1, y1, small,
                &c_string(date.as_bytes()), INK_GREY, W_LEFT, -1);
            match st.history_view {
                HV_ACHIEVEMENTS => {
                    let row = &st.achievements[li];
                    draw_wrapped_text(surface, HIST_CLUB_COL.0 + 2, y, 332, y1, cell,
                        &c_string(format!("  {}", row.club).as_bytes()), INK_GREY, W_LEFT, -1);
                    draw_wrapped_text(surface, 335, y, 758, y1, cell,
                        &c_string(format!("  {}", row.text).as_bytes()), INK_YELLOW, W_LEFT, -1);
                }
                HV_INJURIES => {
                    let row = &st.injuries_list[li];
                    draw_wrapped_text(surface, HIST_CLUB_COL.0 + 5, y, 458, y1, cell,
                        &c_string(format!("  {}", row.injury).as_bytes()), INK_YELLOW, W_LEFT, -1);
                    draw_wrapped_text(surface, 460, y, 608, y1, small,
                        &c_string(row.kind.as_bytes()), INK_ORANGE, TS_CENTRE, -1);
                    draw_wrapped_text(surface, 610, y, 758, y1, small,
                        &c_string(row.period.as_bytes()), INK_ORANGE, TS_CENTRE, -1);
                }
                _ => {
                    let row = &st.bans_list[li];
                    draw_wrapped_text(surface, HIST_CLUB_COL.0 + 5, y, 608, y1, cell,
                        &c_string(format!("  {}", row.ban).as_bytes()), INK_YELLOW, W_LEFT, -1);
                    draw_wrapped_text(surface, 610, y, 758, y1, small,
                        &c_string(row.reason.as_bytes()), INK_ORANGE, W_LEFT, -1);
                }
            }
        }
        // Scrollbar: thumb only when the list overflows.
        draw_panel(surface, HIST_SB_X0, HIST_SB_UP.0, HIST_SB_X1, HIST_SB_UP.1,
            P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
        draw_tri(surface, (HIST_SB_X0 + HIST_SB_X1) / 2, HIST_SB_UP.0 + 7, true, INK_GREY);
        draw_panel(surface, HIST_SB_X0, HIST_SB_TRK.0, HIST_SB_X1, HIST_SB_TRK.1, P_DARKEN, 0, 0, palette);
        if rows_len > visible {
            let trk_h = HIST_SB_TRK.1 - HIST_SB_TRK.0;
            let thumb_h = ((trk_h as usize * visible / rows_len) as i32).max(14);
            let max_scroll = rows_len - visible;
            let thumb_top = HIST_SB_TRK.0 + ((trk_h - thumb_h) as usize * sc / max_scroll) as i32;
            draw_panel(surface, HIST_SB_X0, thumb_top, HIST_SB_X1, thumb_top + thumb_h,
                P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
        }
        draw_panel(surface, HIST_SB_X0, HIST_SB_DOWN.0, HIST_SB_X1, HIST_SB_DOWN.1,
            P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
        draw_tri(surface, (HIST_SB_X0 + HIST_SB_X1) / 2, HIST_SB_DOWN.0 + 7, false, INK_GREY);
        return;
    }

    // The scrollable list is: [column header] + season rows + [Total].
    // The header is list index 0 and scrolls with the content.
    let headers = history_headers(st.is_goalkeeper, st.history_page);
    let n = st.history_rows.len();
    let list_len = n + 2; // header + seasons + total
    let visible = HIST_ROW_Y.len();
    let max_scroll = list_len.saturating_sub(visible);
    let scroll = st.history_scroll.min(max_scroll);

    for (slot, &y) in HIST_ROW_Y.iter().enumerate() {
        let y1 = y + 18;
        let li = scroll + slot;
        if li >= list_len { break; }
        let is_header = li == 0;
        let is_total = li == list_len - 1;
        let season_idx = li.wrapping_sub(1); // valid when 1..=n

        if is_header {
            // Header row: blank season/club/loan, `<<`/`>>` blue boxes,
            // nine grey stat-header boxes with the page's labels.
            draw_panel(surface, HIST_SEASON_COL.0, y, HIST_SEASON_COL.1, y1, P_DARKEN, 0, 0, palette);
            draw_panel(surface, HIST_CLUB_COL.0, y, HIST_CLUB_COL.1, y1, P_DARKEN, 0, 0, palette);
            draw_panel(surface, HIST_LOAN_COL.0, y, HIST_LOAN_COL.1, y1, P_DARKEN, 0, 0, palette);
            for (&(bx0, bx1), lbl) in [HIST_HDR_LT, HIST_HDR_GT].iter().zip(["<<", ">>"]) {
                draw_panel(surface, bx0, y, bx1, y1, P_SOLID_FILL | P_BEVEL, HDR_BLUE, 0, palette);
                draw_wrapped_text(surface, bx0, y, bx1, y1, small,
                    &c_string(lbl.as_bytes()), INK_GREY, TS_CENTRE, -1);
            }
            for (i, &(cx0, cx1)) in HIST_STAT_COLS.iter().enumerate() {
                draw_panel(surface, cx0, y, cx1, y1, P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
                draw_wrapped_text(surface, cx0, y, cx1, y1, small,
                    &c_string(headers[i].as_bytes()), INK_GREY, TS_CENTRE, -1);
            }
            continue;
        }

        // ---- Cell boxes for a data / total row ----
        if !is_total {
            let sel = season_idx == st.history_selected_idx;
            let bg = if sel { SEASON_SEL_BG } else { HDR_BLUE };
            draw_panel(surface, HIST_SEASON_COL.0, y, HIST_SEASON_COL.1, y1,
                P_SOLID_FILL | P_BEVEL, bg, 0, palette);
        }
        draw_panel(surface, HIST_CLUB_COL.0, y, HIST_CLUB_COL.1, y1, P_DARKEN, 0, 0, palette);
        draw_panel(surface, HIST_LOAN_COL.0, y, HIST_LOAN_COL.1, y1, P_DARKEN, 0, 0, palette);
        draw_panel(surface, HIST_HDR_LT.0, y, HIST_HDR_LT.1, y1, P_DARKEN, 0, 0, palette);
        draw_panel(surface, HIST_HDR_GT.0, y, HIST_HDR_GT.1, y1, P_DARKEN, 0, 0, palette);
        for (i, &(cx0, cx1)) in HIST_STAT_COLS.iter().enumerate() {
            if i == 8 {
                draw_panel(surface, cx0, y, cx1, y1, P_SOLID_FILL | P_BEVEL, AVR_BOX, 0, palette);
            } else {
                draw_panel(surface, cx0, y, cx1, y1, P_DARKEN, 0, 0, palette);
            }
        }

        // Per-column stat kind: ratio columns (Pass/Tck/Drb/Sh Tar) show
        // "-" when empty, the rating column "----", counters "0".
        let empty_for = |label: &str| -> &'static str {
            match label {
                "Pass" | "Tck" | "Drb" | "Sh Tar" => "-",
                "Av R" => "----",
                _ => "0",
            }
        };
        // Whether this header column is backed by stored history (only
        // Apps and Gls-for-outfield / Con-for-keeper are recorded).
        let stored = |label: &str| -> bool {
            label == "Apps"
                || (label == "Gls" && !st.is_goalkeeper)
                || (label == "Con" && st.is_goalkeeper)
        };

        if is_total {
            draw_wrapped_text(surface, HIST_CLUB_COL.0 + 2, y, HIST_CLUB_COL.1, y1, cell,
                &c_string(b"  Total"), INK_HIST_CYAN, W_LEFT, -1);
            let (ta, tg) = st.history_total;
            for (i, &(cx0, cx1)) in HIST_STAT_COLS.iter().enumerate() {
                let v = match headers[i] {
                    "Apps" => ta,
                    l if stored(l) => tg,
                    l => empty_for(l),
                };
                draw_wrapped_text(surface, cx0, y, cx1, y1, small,
                    &c_string(v.as_bytes()), INK_HIST_CYAN, TS_CENTRE, -1);
            }
        } else if season_idx < n {
            let row = &st.history_rows[season_idx];
            let sel = season_idx == st.history_selected_idx;
            // Selected inverts: grey box + navy text; else blue box + grey.
            let sink = if sel { HDR_BLUE } else { INK_GREY };
            draw_wrapped_text(surface, HIST_SEASON_COL.0 + 15, y, HIST_SEASON_COL.1, y1, small,
                &c_string(row.season.as_bytes()), sink, W_LEFT, -1);
            draw_wrapped_text(surface, HIST_CLUB_COL.0 + 2, y, HIST_CLUB_COL.1, y1, cell,
                &c_string(format!("  {}", row.club).as_bytes()), INK_GREY, W_LEFT, -1);
            if row.is_loan {
                draw_wrapped_text(surface, HIST_LOAN_COL.0 + 16, y, HIST_LOAN_COL.1, y1, cell,
                    &c_string(b"Loan"), INK_HIST_CYAN, W_LEFT, -1);
            }
            // The newest row (index 0) is the live current season: its
            // counters read 0, ratios "-", rating "----" at game start.
            // Past seasons only carry Apps + Gls/Con; everything else is
            // blank (not stored), never fabricated.
            let is_current = season_idx == 0;
            for (i, &(cx0, cx1)) in HIST_STAT_COLS.iter().enumerate() {
                let label = headers[i];
                // Apps + Gls/Con carry stored/live values (the domain formats
                // the current row's "0"/"N" and past rows' "-"/"N"). The
                // other columns only fill on the live current row (counters
                // "0", ratio cols "-", rating "----"); past rows leave them
                // blank (not stored historically).
                let v: &str = if label == "Apps" {
                    row.apps
                } else if stored(label) {
                    row.goals
                } else if is_current {
                    empty_for(label)
                } else {
                    ""
                };
                if !v.is_empty() {
                    draw_wrapped_text(surface, cx0, y, cx1, y1, small,
                        &c_string(v.as_bytes()), INK_YELLOW, TS_CENTRE, -1);
                }
            }
        }
    }

    // ---- Scrollbar (x=761-780) ----
    draw_panel(surface, HIST_SB_X0, HIST_SB_UP.0, HIST_SB_X1, HIST_SB_UP.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_tri(surface, (HIST_SB_X0 + HIST_SB_X1) / 2, HIST_SB_UP.0 + 7, true, INK_GREY);
    draw_panel(surface, HIST_SB_X0, HIST_SB_TRK.0, HIST_SB_X1, HIST_SB_TRK.1, P_DARKEN, 0, 0, palette);
    if list_len > visible {
        let trk_h = HIST_SB_TRK.1 - HIST_SB_TRK.0;
        let thumb_h = ((trk_h as usize * visible / list_len) as i32).max(14);
        let thumb_top = HIST_SB_TRK.0 + ((trk_h - thumb_h) as usize * scroll / max_scroll) as i32;
        draw_panel(surface, HIST_SB_X0, thumb_top, HIST_SB_X1, thumb_top + thumb_h,
            P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    }
    draw_panel(surface, HIST_SB_X0, HIST_SB_DOWN.0, HIST_SB_X1, HIST_SB_DOWN.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_tri(surface, (HIST_SB_X0 + HIST_SB_X1) / 2, HIST_SB_DOWN.0 + 7, false, INK_GREY);
}
