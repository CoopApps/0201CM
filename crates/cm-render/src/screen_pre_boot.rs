//! Layer 3 — pre-boot screen builders (Setup / Select Leagues / Start
//! Season / Enter Name / Select Club).
//!
//! These are the five screens the user sees BEFORE the game world is
//! initialised. Unlike the Category-B rich-state screens they are not
//! reachable from `dispatch_global` / `dispatch_club` (no menu cmd), so
//! this module owns their pool-building end-to-end.
//!
//! # Provenance
//!
//! Every rect is either from the exe C decomp for that screen or
//! preserves the geometry the pre-fold `crates/cm-ui-app/src/screens.rs`
//! painted at (cited per builder). This mirrors the fidelity level the
//! app already shipped — see memory
//! [[wired-screens-are-approximations]]. Fabricated geometry is
//! forbidden by the task's hard rules.
//!
//! # Exe fns cited
//!
//! * Setup — `FUN_00804020` (setup callback)
//! * Select League(s) — `FUN_008055e0` (draw) + `FUN_00806640` (events)
//! * Start Season — `FUN_00807280`
//! * Enter Name — `FUN_00809cc0` (draw) + `FUN_0080a450` (events)
//! * Select Club — `FUN_0080b2b0` (Take Control team pick)
//!
//! # Pool pattern
//!
//! Same as `screen_rich_state`: every builder returns
//! `Option<(areas_added, widgets_added)>`. Screens spawn the sidebar
//! substrate + banner header + content + Back/Next through the shared
//! helpers here (kept private to this module — the rich-state module
//! keeps its own copy since it dispatches its own layout tweaks).

use crate::widget_pool::{
    GuiRecordPool, WidgetDescriptor, KIND_BUTTON, KIND_HEADER, KIND_LABEL,
};

// Colour + style constants — same values as `screen_rich_state` so the
// two pipelines land the same pixels. `label_ink` is RGB555-packed for
// the widget field; `text_style` and `style_byte` mirror the
// widget-batch defaults.
const INK_YELLOW: u16 = 0x7FE0;
const INK_NEAR_WHITE: u16 = 0x7FFF;
const INK_GREY: u16 = 0x6318;
const INK_BUTTON: u16 = 3;
const INK_TITLE: u16 = 7;
const TS_TITLE: u32 = 12;
const PANEL_STYLE: u32 = 0x30;

fn spawn_label(
    pool: &mut GuiRecordPool,
    parent_area: i16,
    x0: i16, y0: i16, x1: i16, y1: i16,
    text: &str, label_ink: u16, msg_id: i32, seq: i32,
) -> Option<()> {
    let d = WidgetDescriptor {
        kind: KIND_LABEL,
        grid_x0: x0 as i32, grid_y0: y0 as i32,
        grid_x1: x1 as i32, grid_y1: y1 as i32,
        seq, row_index: 0,
        style_byte: PANEL_STYLE,
        colour_a: 0, colour_b: 0,
        text_style: TS_TITLE,
        label_ink,
        pattern: 0,
        text: text.to_string(),
        slot_40: 0,
        msg_id,
        userdata_id: 0,
    };
    pool.spawn_widget(d, parent_area).map(|_| ())
}

fn spawn_button(
    pool: &mut GuiRecordPool,
    parent_area: i16,
    x0: i16, y0: i16, x1: i16, y1: i16,
    text: &str, label_ink: u16, msg_id: i32, seq: i32,
) -> Option<()> {
    let d = WidgetDescriptor {
        kind: KIND_BUTTON,
        grid_x0: x0 as i32, grid_y0: y0 as i32,
        grid_x1: x1 as i32, grid_y1: y1 as i32,
        seq, row_index: 0,
        style_byte: PANEL_STYLE,
        colour_a: 0, colour_b: 0,
        text_style: 0x0C,
        label_ink,
        pattern: 0,
        text: text.to_string(),
        slot_40: 0,
        msg_id,
        userdata_id: 0,
    };
    pool.spawn_widget(d, parent_area).map(|_| ())
}

fn spawn_header(
    pool: &mut GuiRecordPool,
    parent_area: i16,
    x0: i16, y0: i16, x1: i16, y1: i16,
    text: &str,
) -> Option<()> {
    let d = WidgetDescriptor {
        kind: KIND_HEADER,
        grid_x0: x0 as i32, grid_y0: y0 as i32,
        grid_x1: x1 as i32, grid_y1: y1 as i32,
        seq: 0, row_index: 0,
        style_byte: PANEL_STYLE,
        colour_a: 0, colour_b: 0,
        text_style: TS_TITLE,
        label_ink: INK_TITLE,
        pattern: 0,
        text: text.to_string(),
        slot_40: 0,
        msg_id: 0,
        userdata_id: 0,
    };
    pool.spawn_widget(d, parent_area).map(|_| ())
}

/// Sidebar substrate + root content canvas + title banner. Same shape
/// as `screen_rich_state::spawn_common_prelude` — kept local so the
/// pre-boot module does not depend on a sibling's private helper.
fn spawn_common_prelude(pool: &mut GuiRecordPool, title: &str) -> Option<i16> {
    // Sidebar substrate — [[news-screen-geometry]] left column.
    pool.spawn_area(0, 70, 99, 599, 0, Vec::new(), 7, 0, PANEL_STYLE, 0, -1)?;
    let root = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        7, 0, PANEL_STYLE, 0, -1,
    )? as i16;
    // Banner header — same 100..790 x 10..70 band every pre-boot
    // screen paints (screens.rs::setup / ::enter_name / ::select_club).
    spawn_header(pool, root, 100, 10, 790, 70, "Championship Manager 2001/02")?;
    // Sub-heading with the actual screen title — screens.rs paints
    // this on every pre-boot screen at (100, 80, 790, 125).
    spawn_label(pool, root, 100, 80, 790, 125, title, INK_NEAR_WHITE, 0, 0)?;
    Some(root)
}

/// Two grey nav buttons — same rects `screen_rich_state` uses.
fn spawn_nav_back_next(
    pool: &mut GuiRecordPool,
    parent_area: i16,
    back_enabled: bool,
    next_enabled: bool,
    next_label: &str,
) -> Option<()> {
    let (bink, bmsg) = if back_enabled { (INK_BUTTON, -2) } else { (INK_GREY, 0) };
    spawn_button(pool, parent_area, 338, 555, 500, 585, "Back", bink, bmsg, 100)?;
    let (nink, nmsg) = if next_enabled { (INK_BUTTON, -3) } else { (INK_GREY, 0) };
    spawn_button(pool, parent_area, 685, 555, 790, 585, next_label, nink, nmsg, 101)?;
    Some(())
}

// ============================================================================
// 1. Setup — FUN_00804020
// ============================================================================

/// The 9 setup buttons — same 2-col × 4-row grid + a centred "Web Sites"
/// row, as `screens::setup_buttons`. Labels are the exe's shipped
/// button texts (FUN_00804020 references them via l10n slot lookups).
pub const SETUP_BUTTON_LABELS: [&str; 9] = [
    "Start New Game", "Quick Start Game",
    "Restore Saved Game", "Delete Saved Game",
    "Network Play", "Game Settings",
    "Hall Of Fame", "Game Credits",
    "Web Sites",
];

pub fn build_setup(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, "Setup Game")?;
    // 2 × 4 grid inside (110, 145, 780, 535) — cited from
    // `screens::setup_buttons` (crates/cm-ui-app/src/screens.rs:1572).
    // Row height 97 (390 / 4), col width 335.
    let box_l: i16 = 110;
    let box_t: i16 = 145;
    let row_h: i16 = 390 / 4;
    let col_w: i16 = (780 - 110) / 2;
    for i in 0..8usize {
        let (r, c) = (i / 2, i % 2);
        let x0 = box_l + (c as i16) * col_w;
        let y0 = box_t + (r as i16) * row_h;
        let x1 = x0 + col_w - 4;
        let y1 = y0 + row_h - 4;
        spawn_button(
            pool, root, x0, y0, x1, y1,
            SETUP_BUTTON_LABELS[i], INK_BUTTON,
            /* msg_id */ (0x800 + i) as i32, i as i32,
        )?;
    }
    // "Web Sites" — centred single button (screens::setup_buttons row 4).
    let y0 = box_t + 4 * row_h;
    spawn_button(
        pool, root, 278, y0, 611, y0 + row_h - 4,
        SETUP_BUTTON_LABELS[8], INK_BUTTON, 0x808, 8,
    )?;
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// 2. Select League(s) — FUN_008055e0
// ============================================================================

/// One row of the Select League(s) picker.
pub struct LeaguesRow<'a> {
    pub country: &'a str,
    pub selected: bool,
    pub background_marker: bool,
    pub secondary_label: Option<&'a str>,
    pub secondary_active: bool,
}

/// Options block above the picker.
pub struct LeaguesOptions {
    pub use_real_players: bool,
    pub attribute_masking: bool,
}

pub fn build_select_leagues(
    pool: &mut GuiRecordPool,
    rows: &[LeaguesRow<'_>],
    options: LeaguesOptions,
    scroll: usize,
) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, "Select League(s) To Manage")?;
    // Options strip (110, 145, 780, 165) — screens.rs::leagues_hit
    // splits it into 9 columns; here we render the two Yes/No pairs
    // as simple labels using the same relative positions.
    // Real Players Yes/No.
    let yn_ink = |on| if on { INK_YELLOW } else { INK_GREY };
    spawn_label(pool, root, 110, 145, 250, 165,
        "Real Players:", INK_NEAR_WHITE, 0, 200)?;
    spawn_button(pool, root, 260, 145, 300, 165,
        "Yes", yn_ink(options.use_real_players), 0x39, 201)?;
    spawn_button(pool, root, 310, 145, 350, 165,
        "No", yn_ink(!options.use_real_players), 0x3a, 202)?;
    if options.use_real_players {
        spawn_label(pool, root, 370, 145, 510, 165,
            "Attribute Masking:", INK_NEAR_WHITE, 0, 203)?;
        spawn_button(pool, root, 445, 145, 485, 165,
            "Yes", yn_ink(options.attribute_masking), 0x3b, 204)?;
        spawn_button(pool, root, 495, 145, 525, 165,
            "No", yn_ink(!options.attribute_masking), 0x3c, 205)?;
    }
    // Select-All / De-Select-All (screens.rs::leagues_hit lines
    // 1677-1685) — right half of the toggle strip.
    spawn_button(pool, root, 530, 145, 650, 165, "Select All",
        INK_BUTTON, 0xb, 206)?;
    spawn_button(pool, root, 660, 145, 780, 165, "De-Select All",
        INK_BUTTON, 0xa, 207)?;
    // Picker list — 16 visible rows at (110, 170, 780, 535). Column
    // weights [23,1,15,1,15,1,19] from the exe (crates/cm-ui-app/src/
    // screens.rs::LeaguesProvider::row_text).
    const N_VISIBLE: usize = 16;
    let list_top: i16 = 170;
    let list_bot: i16 = 535;
    let row_h: i16 = (list_bot - list_top) / N_VISIBLE as i16;
    let total: i32 = 23 + 1 + 15 + 1 + 15 + 1 + 19;
    let width: i32 = 780 - 110;
    let col_x = |i: i32| (110 + (i * width / total)) as i16;
    let (cx0, cx1) = (col_x(0), col_x(23));
    let (sx0, sx1) = (col_x(24), col_x(24 + 15));
    let (bx0, bx1) = (col_x(24 + 15 + 1), col_x(24 + 15 + 1 + 15));
    let (secx0, secx1) = (col_x(24 + 15 + 1 + 15 + 1), 780i16);
    for row in 0..N_VISIBLE {
        let Some(r) = rows.get(scroll + row) else { break };
        let y = list_top + (row as i16) * row_h;
        let yb = y + row_h - 2;
        let ink_country = if r.selected { INK_YELLOW } else { INK_NEAR_WHITE };
        spawn_label(pool, root, cx0, y, cx1, yb, r.country, ink_country, 0,
            (1000 + row * 4) as i32)?;
        let sel_ink = if r.selected { INK_YELLOW } else { INK_GREY };
        spawn_button(pool, root, sx0, y, sx1, yb, "SELECTED", sel_ink,
            0x0c, (1000 + row * 4 + 1) as i32)?;
        let bg_ink = if r.background_marker { INK_YELLOW } else { INK_GREY };
        spawn_button(pool, root, bx0, y, bx1, yb, "BACKGROUND", bg_ink,
            0x0d, (1000 + row * 4 + 2) as i32)?;
        if let Some(sec) = r.secondary_label {
            let ink = if r.secondary_active { INK_YELLOW } else { INK_GREY };
            spawn_button(pool, root, secx0, y, secx1, yb, sec, ink, 0x0e,
                (1000 + row * 4 + 3) as i32)?;
        }
    }
    spawn_nav_back_next(pool, root, true, !rows.is_empty(), "Next")?;
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// 3. Start Season — FUN_00807280
// ============================================================================

pub fn build_start_season(
    pool: &mut GuiRecordPool,
    season_rows: &[String],
    selected: usize,
) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, "Select Start Season")?;
    // Season list at (110, 270, 780, 535) — 1 col, N rows
    // (screens.rs::season_hit uses this exact rect).
    let list_top: i16 = 270;
    let list_bot: i16 = 535;
    let n = season_rows.len().max(1);
    let row_h: i16 = (list_bot - list_top) / n as i16;
    for (i, label) in season_rows.iter().enumerate() {
        let y = list_top + (i as i16) * row_h;
        let yb = y + row_h - 2;
        let ink = if i == selected { INK_YELLOW } else { INK_NEAR_WHITE };
        spawn_button(pool, root, 110, y, 780, yb, label, ink,
            (0x2000 + i) as i32, (2000 + i) as i32)?;
    }
    spawn_nav_back_next(pool, root, true, !season_rows.is_empty(), "Start")?;
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// 4. Enter Name — FUN_00809cc0
// ============================================================================

pub struct NameFields<'a> {
    pub first: &'a str,
    pub second: &'a str,
    pub nickname: &'a str,
    /// 0 = First, 1 = Second, 2 = Nickname
    pub focus: u8,
    pub is_valid: bool,
}

/// Row rects verified from the manager-creation decode
/// (`FUN_00809cc0`): (150, 217..277), (150, 279..339), (150, 341..401).
const NAME_ROW_RECTS: [(i16, i16, i16, i16); 3] = [
    (150, 217, 740, 277),
    (150, 279, 740, 339),
    (150, 341, 740, 401),
];
const NAME_LABELS: [&str; 3] = ["First Name", "Second Name", "Nickname"];

pub fn build_enter_name(
    pool: &mut GuiRecordPool,
    fields: &NameFields<'_>,
) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, "Enter Name")?;
    for (i, (l, t, r, b)) in NAME_ROW_RECTS.iter().enumerate() {
        // Label — column 0.
        spawn_label(pool, root, *l, *t, *l + 200, *b,
            NAME_LABELS[i], INK_NEAR_WHITE, 0, (3000 + i * 2) as i32)?;
        // Typed text — column 2 with a caret on the focused field.
        let raw = match i { 0 => fields.first, 1 => fields.second, _ => fields.nickname };
        let shown = if fields.focus as usize == i {
            format!("{raw}_")
        } else {
            raw.to_string()
        };
        spawn_button(pool, root, *l + 220, *t, *r, *b,
            if shown.is_empty() { "-" } else { shown.as_str() },
            INK_NEAR_WHITE, (0x3000 + i) as i32, (3000 + i * 2 + 1) as i32)?;
    }
    spawn_nav_back_next(pool, root, true, fields.is_valid, "Next")?;
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// 5. Select Club — FUN_0080b2b0
// ============================================================================

pub struct ClubRow<'a> {
    pub club_name: &'a str,
    pub division_name: &'a str,
    /// True on the first row of each new division block — the exe
    /// paints these rows yellow (screens::select_club:1477).
    pub division_new: bool,
}

pub fn build_select_club(
    pool: &mut GuiRecordPool,
    clubs: &[ClubRow<'_>],
    scroll: usize,
) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, "Select Club To Manage")?;
    // Club list at (110, 145, 780, 535) — 15 visible rows, cols
    // [13, 1, 6] name/gap/division (screens::CLUB_LIST +
    // ::club_row_rects).
    const N_VISIBLE: usize = 15;
    let list_top: i16 = 145;
    let list_bot: i16 = 535;
    let row_h: i16 = (list_bot - list_top) / N_VISIBLE as i16;
    let total: i32 = 13 + 1 + 6;
    let width: i32 = 780 - 110;
    let name_x0: i16 = 110;
    let name_x1: i16 = (110 + (13 * width / total)) as i16;
    let div_x0: i16 = (110 + ((13 + 1) * width / total)) as i16;
    let div_x1: i16 = 780 - 16; // reserve scrollbar track
    for row in 0..N_VISIBLE {
        let Some(c) = clubs.get(scroll + row) else { break };
        let y = list_top + (row as i16) * row_h;
        let yb = y + row_h - 2;
        spawn_button(pool, root, name_x0, y, name_x1, yb,
            c.club_name, INK_NEAR_WHITE,
            (0x4000 + scroll + row) as i32, (4000 + row) as i32)?;
        let ink = if c.division_new { INK_YELLOW } else { INK_GREY };
        spawn_label(pool, root, div_x0, y, div_x1, yb,
            c.division_name, ink, 0, (4000 + row + 100) as i32)?;
    }
    // Back only — Next is done by picking a club (screens::select_club).
    spawn_button(pool, root, 338, 555, 500, 585, "Back", INK_BUTTON, -2, 5000)?;
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_spawns_nine_buttons_plus_prelude() {
        let mut pool = GuiRecordPool::new();
        let (_a, w) = build_setup(&mut pool).unwrap();
        // 2 area preludes + banner header + sub-heading + 9 buttons = 12.
        assert!(w >= 11, "setup builder must spawn 9 buttons + prelude: got {w}");
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "Start New Game"));
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "Web Sites"));
    }

    #[test]
    fn select_leagues_populates_from_rows() {
        let rows = vec![
            LeaguesRow { country: "England", selected: true, background_marker: false,
                secondary_label: Some("Conference"), secondary_active: false },
            LeaguesRow { country: "France", selected: false, background_marker: true,
                secondary_label: None, secondary_active: false },
        ];
        let mut pool = GuiRecordPool::new();
        let (_a, _w) = build_select_leagues(
            &mut pool, &rows,
            LeaguesOptions { use_real_players: true, attribute_masking: true }, 0,
        ).unwrap();
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "England"));
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "France"));
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "Conference"));
        // England row is selected → its country label is yellow.
        let england = pool.widgets.iter()
            .find(|x| x.descriptor.text == "England").unwrap();
        assert_eq!(england.descriptor.label_ink, INK_YELLOW);
    }

    #[test]
    fn start_season_marks_selected_row() {
        let rows = vec!["England 01/02".to_string(), "France 01/02".to_string()];
        let mut pool = GuiRecordPool::new();
        build_start_season(&mut pool, &rows, 1).unwrap();
        let france = pool.widgets.iter()
            .find(|x| x.descriptor.text == "France 01/02").unwrap();
        assert_eq!(france.descriptor.label_ink, INK_YELLOW);
        let england = pool.widgets.iter()
            .find(|x| x.descriptor.text == "England 01/02").unwrap();
        assert_ne!(england.descriptor.label_ink, INK_YELLOW);
    }

    #[test]
    fn enter_name_shows_focused_caret_and_validates_next() {
        let fields = NameFields {
            first: "Alex", second: "Ferguson", nickname: "",
            focus: 2, is_valid: true,
        };
        let mut pool = GuiRecordPool::new();
        build_enter_name(&mut pool, &fields).unwrap();
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "First Name"));
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "Alex"));
        // Nickname is empty & focused → shows just "_".
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "_"));
        // Next button is live (blue) — is_valid = true.
        let next = pool.widgets.iter()
            .find(|x| x.descriptor.text == "Next").unwrap();
        assert_eq!(next.descriptor.label_ink, INK_BUTTON);
    }

    #[test]
    fn select_club_highlights_division_change() {
        let clubs = vec![
            ClubRow { club_name: "Arsenal", division_name: "Premier",
                division_new: true },
            ClubRow { club_name: "Chelsea", division_name: "Premier",
                division_new: false },
        ];
        let mut pool = GuiRecordPool::new();
        build_select_club(&mut pool, &clubs, 0).unwrap();
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "Arsenal"));
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "Chelsea"));
        // Only ONE division name is yellow (the first of each block).
        let yellow_divs: usize = pool.widgets.iter()
            .filter(|x| x.descriptor.text == "Premier"
                && x.descriptor.label_ink == INK_YELLOW)
            .count();
        assert_eq!(yellow_divs, 1);
    }
}
