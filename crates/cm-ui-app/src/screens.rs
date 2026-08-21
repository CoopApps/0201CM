//! Spec-driven screen render — the OLD `screens.rs` (700 lines of hand-ported
//! Rust per screen) is now `screens.rs.bak`. Every screen ships as data,
//! produced by the carver at `D:/cm0102-carve/analysis/screens/<va>.json`,
//! and evaluated by `cm-widget`.
//!
//! This file is deliberately thin. Adding a new screen is:
//!   1. Point the carver at its callback (already done for 95 screens).
//!   2. Point the app at the JSON file (SCREEN_SPECS below).
//!   3. Optionally write a StateProvider for that screen (~50 lines) when the
//!      spec references runtime scratch buffers / palette globals / guards.
//!
//! Old hand-porting per-screen: ~600 lines each. New: ~30 lines here + optional
//! 50-line adapter. Every bug fix propagates through one evaluator, not 150
//! per-screen Rust functions.

use crate::game_state::{ManagerName, SelectLeaguesState, StartSeasonState};
use cm_render::font::Fonts;
use cm_render::image::Image;
use cm_render::layout::rebuild_layout;
use cm_render::panel::{F_BEVEL, F_SOLID_FILL};
use cm_render::Surface;
use cm_widget::{HighlightHint, NullProvider, Palette, ScreenSpec, StateProvider};
use crate::game_state::real_picker_slots;

/// Clicks on the Enter Name screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameClick {
    Field(u8),
    Back,
    Next,
}

/// Row geometry for the three name fields — from the manager-creation decode
/// (FUN_00809cc0). Each row is an AREA at (150, t, 740, b) with flags 0x22
/// (F_TRANSPARENT | F_BEVEL) — the standard blue-bordered input box — split
/// into 3 columns weight {8,1,12}: label / gap / typed text. Disassembled
/// area rects: (150,217)-(740,277), (150,279)-(740,339), (150,341)-(740,401).
const NAME_ROWS: [(i32, i32); 3] = [(217, 277), (279, 339), (341, 401)];
const NAME_LABELS: [&str; 3] = ["   First Name", "   Second Name", "   Nickname"];

/// The edit-text sub-rect (column 2) of a row — where the caret/typed text go
/// and where a click focuses the field.
fn name_field_rects() -> [(i32, i32, i32, i32); 3] {
    let mut out = [(0, 0, 0, 0); 3];
    for (i, &(t, b)) in NAME_ROWS.iter().enumerate() {
        let lo = rebuild_layout((150, t, 740, b), 2, &[8, 1, 12], &[1], false);
        out[i] = (lo.col_left[2], lo.row_top[0], lo.col_right[2], lo.row_bottom[0]);
    }
    out
}

/// Render the Enter Name screen (draw 0x00809cc0). Title banner, "Enter Name"
/// heading, three blue-bordered input rows (label + typed text), and Back/Next.
pub fn enter_name(
    s: &mut Surface,
    fonts: &mut Fonts,
    bg: Option<&Image>,
    manager: &ManagerName,
) {
    use cm_render::panel::{F_TRANSPARENT, F_VGRADIENT};
    let pal = palette();
    if let Some(image) = bg {
        s.blit_image(image, 0, 0);
    } else {
        s.fill(0, 0, 0);
    }
    // Banner + heading.
    s.draw_panel(100, 10, 790, 70, F_SOLID_FILL | F_BEVEL, pal.banner_red);
    {
        let f = fonts.slot(7);
        s.draw_text_box(100, 10, 790, 70, 0, f, pal.highlight_fg, "Championship Manager 2001/02");
    }
    {
        let f = fonts.slot(6);
        s.draw_text_box(100, 80, 790, 125, 0, f, pal.near_white, "Enter Name");
    }
    // Sidebar (mode 4).
    s.draw_panel(0, 0, 89, 599, F_VGRADIENT, pal.sidebar_blue);

    for i in 0..3u8 {
        let (t, b) = (NAME_ROWS[i as usize].0, NAME_ROWS[i as usize].1);
        // The row IS the input box: transparent + blue bevel (area flags 0x22),
        // spanning the whole 150..740 row, exactly as the exe draws it.
        s.draw_panel(150, t, 740, b, F_TRANSPARENT | F_BEVEL, pal.btn_blue);

        let lo = rebuild_layout((150, t, 740, b), 2, &[8, 1, 12], &[1], false);
        let f = fonts.slot(3);
        // Label in column 0 (left-justified).
        s.draw_text_box(
            lo.col_left[0], lo.row_top[0], lo.col_right[0], lo.row_bottom[0],
            0x1, f, pal.near_white, NAME_LABELS[i as usize],
        );
        // Typed text in column 2 (left-justified), white ink (the exe's edit
        // item ink), with a caret on the focused field. Focus is shown by the
        // caret alone — there is NO coloured border on this screen.
        let text = manager.field(i);
        let shown = if manager.focus == i {
            format!("{text}_")
        } else {
            text.to_string()
        };
        let (el, et, er, eb) = (lo.col_left[2], lo.row_top[0], lo.col_right[2], lo.row_bottom[0]);
        s.draw_text_box(el, et, er, eb, 0x1, f, pal.near_white, &shown);
    }

    // Back / Next (grey bevelled buttons — the exe uses F_SOLID_FILL|F_BEVEL,
    // flags 0x30, on these two, distinct from the input rows).
    let nav = rebuild_layout((100, 555, 790, 590), 1, &[3, 1], &[1], false);
    let bf = fonts.slot(3);
    for (rect, label, enabled) in [
        (nav.cell(0, 0), "Back", true),
        (nav.cell(1, 0), "Next", manager.is_valid()),
    ] {
        let (l, t, r, b) = rect;
        s.draw_panel(l, t, r, b, F_SOLID_FILL | F_BEVEL, pal.grey);
        let ink = if enabled { pal.near_white } else { (120, 120, 120) };
        s.draw_text_box(l, t, r, b, 0, bf, ink, label);
    }
}

/// Hit-test the Enter Name screen.
pub fn enter_name_hit(x: i32, y: i32) -> Option<NameClick> {
    for (i, &(l, t, r, b)) in name_field_rects().iter().enumerate() {
        if x >= l && x <= r && y >= t && y <= b {
            return Some(NameClick::Field(i as u8));
        }
    }
    let nav = rebuild_layout((100, 555, 790, 590), 1, &[3, 1], &[1], false);
    if in_rect(x, y, nav.cell(0, 0)) {
        return Some(NameClick::Back);
    }
    if in_rect(x, y, nav.cell(1, 0)) {
        return Some(NameClick::Next);
    }
    None
}
use std::path::PathBuf;

/// Location of the widget-spec JSONs produced by the carver.
///
/// Override with the `CM_SCREENS_DIR` env var when running from a different
/// checkout. Default targets the sibling repo where the carve output lives.
fn spec_dir() -> PathBuf {
    let s = std::env::var("CM_SCREENS_DIR")
        .unwrap_or_else(|_| "D:/cm0102-carve/analysis/screens".to_string());
    PathBuf::from(s)
}

/// Load a screen spec by its callback VA (as it appears in the carver's file
/// naming, e.g. `0x804020` → `00804020.json`). Returns `None` if the file
/// isn't there — the caller can fall back to a blank frame + warning.
fn load_spec(callback_va: u32) -> Option<ScreenSpec> {
    let path = spec_dir().join(format!("{:08x}.json", callback_va));
    ScreenSpec::load(&path).ok()
}

// ------- shared palette across all screens -------

fn palette() -> Palette {
    Palette::default()
}

// ------- Setup Game screen -------

/// The 9 Setup menu buttons: (left, top, right, bottom, label). Kept
/// alongside the spec-driven render for hit-testing until we lift hit-test
/// into cm-widget too. Positions come from FUN_005d6bf0 laid out via
/// `rebuild_layout((110,145,780,535), 1, [1,1], [1,1,1,1,1,1], false)` —
/// same as the spec's item ctors.
pub fn setup_buttons() -> [(i32, i32, i32, i32, &'static str); 9] {
    use cm_render::layout::rebuild_layout;
    let lo = rebuild_layout((110, 145, 780, 535), 1, &[1, 1], &[1, 1, 1, 1, 1, 1], false);
    let labels = [
        ["Start New Game", "Quick Start Game"],
        ["Restore Saved Game", "Delete Saved Game"],
        ["Network Play", "Game Settings"],
        ["Hall Of Fame", "Game Credits"],
    ];
    let mut out = [(0, 0, 0, 0, ""); 9];
    let mut i = 0;
    while i < 8 {
        let (r, c) = (i / 2, i % 2);
        out[i] = (lo.col_left[c], lo.row_top[r], lo.col_right[c], lo.row_bottom[r], labels[r][c]);
        i += 1;
    }
    out[8] = (278, lo.row_top[4], 611, lo.row_bottom[4], "Web Sites");
    out
}

pub fn setup_hit(x: i32, y: i32) -> Option<usize> {
    setup_buttons()
        .iter()
        .position(|&(l, t, r, b, _)| x >= l && x <= r && y >= t && y <= b)
}

/// Render Setup Game from `analysis/screens/00804020.json`.
pub fn setup(s: &mut Surface, fonts: &mut Fonts, bg: Option<&Image>, _pressed: Option<usize>) {
    if let Some(spec) = load_spec(0x804020) {
        spec.render(s, fonts, bg, &palette(), &NullProvider);
    } else {
        blank_with_warning(s, "screens/00804020.json not found");
    }
}

// ------- Select League(s) — spec-driven -------

/// Same click enum kept for main.rs's hit-test dispatch. When we lift
/// hit-test into cm-widget this shrinks further.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaguesClick {
    Back,
    Next,
    RealPlayersYes,
    RealPlayersNo,
    MaskingYes,
    MaskingNo,
    SelectAll,
    DeselectAll,
    /// Row click on the SELECTED cell (col 2) — event 0xc in the exe.
    ToggleSelected(u8),
    /// Row click on the BACKGROUND cell (col 4) — independent toggle per
    /// the user's spec ("click SELECTED to highlight, click again to
    /// un-highlight; same for every option").
    ToggleBackground(u8),
    /// Row click on the secondary league cell (col 6). Only fires when the
    /// country has a secondary competition. Maps to event 0xe in the exe.
    ToggleSecondary(u8),
    /// Row click on the human-controller marker column. Not currently
    /// emitted as a separate cell since col 6 hosts the secondary league.
    ToggleHuman(u8),
}

/// Approximate hit-test — same rects the exe declares in FUN_008055e0.
/// Until cm-widget grows a proper hit tester, this is a code-derived shim
/// (rects and column indexes are from the same spec the render uses).
pub fn leagues_hit(state: &SelectLeaguesState, x: i32, y: i32) -> Option<LeaguesClick> {
    use cm_render::layout::rebuild_layout;
    // Back / Next
    let nav = rebuild_layout((100, 555, 790, 590), 1, &[3, 1], &[1], false);
    if in_rect(x, y, nav.cell(0, 0)) {
        return Some(LeaguesClick::Back);
    }
    if in_rect(x, y, nav.cell(1, 0)) {
        return Some(LeaguesClick::Next);
    }
    // Toggles bar (110,145,525,165) — 9 cols, real weights from the spec.
    let tlo = rebuild_layout(
        (110, 145, 525, 165),
        2,
        &[0x2a, 0xd, 1, 0xd, 0xa, 0x2a, 0xd, 1, 0xd],
        &[1],
        false,
    );
    for (col, click) in [
        (2, LeaguesClick::RealPlayersYes),
        (3, LeaguesClick::RealPlayersNo),
    ] {
        if x >= tlo.col_left[col] && x <= tlo.col_right[col]
            && y >= tlo.row_top[0] && y <= tlo.row_bottom[0]
        {
            return Some(click);
        }
    }
    if state.options.use_real_players {
        for (col, click) in [
            (6, LeaguesClick::MaskingYes),
            (8, LeaguesClick::MaskingNo),
        ] {
            if x >= tlo.col_left[col] && x <= tlo.col_right[col]
                && y >= tlo.row_top[0] && y <= tlo.row_bottom[0]
            {
                return Some(click);
            }
        }
    }
    // Select All / De-Select All (530,145,780,165)
    let sa = rebuild_layout((530, 145, 780, 165), 2, &[1, 1], &[1], false);
    if y >= sa.row_top[0] && y <= sa.row_bottom[0] {
        if x >= sa.col_left[0] && x <= sa.col_right[0] {
            return Some(LeaguesClick::SelectAll);
        }
        if x >= sa.col_left[1] && x <= sa.col_right[1] {
            return Some(LeaguesClick::DeselectAll);
        }
    }
    // List rows (110,170,780,535) — 7 cols × 16 rows. Cols 2/4 = primary toggle.
    let list = rebuild_layout(
        (110, 170, 780, 535),
        2,
        &[0x17, 1, 0xf, 1, 0xf, 1, 0x13],
        &[1; 16],
        false,
    );
    if x >= list.col_left[0] && x <= list.col_right[6] {
        for row in 0..16 {
            if y >= list.row_top[row] && y <= list.row_bottom[row] {
                let idx = state.order.get(row).copied().unwrap_or(row as u8);
                if idx as usize >= state.slots.len() {
                    return None;
                }
                if x >= list.col_left[2] && x <= list.col_right[2] {
                    return Some(LeaguesClick::ToggleSelected(idx));
                }
                if x >= list.col_left[4] && x <= list.col_right[4] {
                    return Some(LeaguesClick::ToggleBackground(idx));
                }
                if x >= list.col_left[6] {
                    // Column 6 hosts the secondary-league button for the six
                    // countries with lower leagues (England/Germany/Italy/
                    // Portugal/Spain/Sweden), else the human-controller
                    // marker for everyone else.
                    let country = state.slots.iter().find(|s| s.index == idx)
                        .map(|s| s.primary_name.as_str())
                        .unwrap_or("");
                    if secondary_league_label(country).is_some() {
                        return Some(LeaguesClick::ToggleSecondary(idx));
                    }
                    return Some(LeaguesClick::ToggleHuman(idx));
                }
            }
        }
    }
    None
}

/// Per-screen provider for Select League(s). Supplies the picker row count
/// (26 real countries) and the country name for column 0. The other columns
/// (SELECTED / BACKGROUND / BACKGROUND / human icon) are scratch buffers the
/// exe fills per-row at runtime; leaving them fall-through prints the last
/// value the scraper captured, which is wrong. For now we return empty
/// strings for cols 1..6 so nothing leaked-stale appears.
pub struct LeaguesProvider {
    pub state: SelectLeaguesState,
    /// Scroll offset in slots — the picker area's `nrows=16` shows 16 at a
    /// time; a scrollbar (not yet spec'd) advances this. For now default 0.
    pub scroll: usize,
}

impl LeaguesProvider {
    pub fn new(state: SelectLeaguesState) -> Self { Self { state, scroll: 0 } }
    pub fn with_scroll(state: SelectLeaguesState, scroll: usize) -> Self { Self { state, scroll } }

    fn slot_for_row(&self, row: usize) -> Option<&crate::game_state::PickerSlot> {
        let idx = self.state.order.get(self.scroll + row).copied()? as usize;
        self.state.slots.get(idx)
    }
}

/// Secondary competition per country, as it appears in the exe's per-country
/// `_comps()` handlers (English → Conference, Italian → Serie C2 A/B/C, etc).
/// Currently a static table because we haven't yet decoded which pool records
/// the secondary IDs live in. NULL means "no secondary league in this
/// country's picker".
/// Countries with an optional lower-league picker cell in Select Leagues.
/// User-confirmed list (2026-08-20): only England / Germany / Italy /
/// Portugal / Spain / Sweden expose a secondary competition here. All
/// others render col 6 as blank.
fn secondary_league_label(country: &str) -> Option<&'static str> {
    match country {
        "England" => Some("Conference"),
        "Germany" => Some("Regionalliga"),
        "Italy" => Some("Serie C2 A/B/C"),
        "Portugal" => Some("Segunda B"),
        "Spain" => Some("Segunda B"),
        "Sweden" => Some("Superettan"),
        _ => None,
    }
}

const LIST_RECT: (i32, i32, i32, i32) = (110, 170, 780, 535);
const TOGGLE_RECT: (i32, i32, i32, i32) = (110, 145, 525, 165);
/// Yellow highlight RGB565 — matches the exe's DAT_00ad6bc4 (approx pack565(255,255,0)).
const YELLOW565: u32 = 0xFFE0;

impl StateProvider for LeaguesProvider {
    fn row_count(&self, area_rect: (i32, i32, i32, i32)) -> Option<usize> {
        // The picker area declared in FUN_008055e0. Show min(16, remaining).
        if area_rect == LIST_RECT {
            let remaining = self.state.slots.len().saturating_sub(self.scroll);
            Some(remaining.min(16))
        } else {
            None
        }
    }

    fn total_rows(&self, area_rect: (i32, i32, i32, i32)) -> Option<usize> {
        if area_rect == LIST_RECT {
            Some(self.state.slots.len())
        } else {
            None
        }
    }

    fn scroll_offset(&self, area_rect: (i32, i32, i32, i32)) -> usize {
        if area_rect == LIST_RECT { self.scroll } else { 0 }
    }

    fn row_text(&self, area_rect: (i32, i32, i32, i32), row: usize, col: usize) -> Option<String> {
        if area_rect != LIST_RECT { return None; }
        let slot = self.slot_for_row(row)?;
        // Column layout from FUN_008055e0: col_weights = [23, 1, 15, 1, 15, 1, 19]
        //   col 0 = country name
        //   col 2 = ALWAYS "SELECTED" button — the choice, not a switch
        //   col 4 = ALWAYS "BACKGROUND" button — the OTHER choice
        //   col 6 = secondary league NAME (Conference / Serie C2 A/B/C etc.),
        //           or blank when the country has no secondary in the picker
        // Which of {SELECTED, BACKGROUND} is the current pick is indicated by
        // cell_box (yellow highlight around the active one), NOT by hiding
        // the other or by ink colour.
        match col {
            0 => Some(slot.primary_name.clone()),
            2 => Some("SELECTED".into()),
            4 => Some("BACKGROUND".into()),
            6 => Some(secondary_league_label(&slot.primary_name).unwrap_or("").into()),
            _ => None,
        }
    }

    fn cell_ink(&self, area_rect: (i32, i32, i32, i32), row: usize, col: usize) -> Option<u32> {
        // Picker row cells — yellow ink follows selection state.
        if area_rect == LIST_RECT {
            let slot = self.slot_for_row(row)?;
            return match col {
                0 => if slot.selected { Some(YELLOW565) } else { None },
                2 => if slot.selected { Some(YELLOW565) } else { None },
                4 => if slot.background_marker { Some(YELLOW565) } else { None },
                6 => if slot.extra && secondary_league_label(&slot.primary_name).is_some() {
                    Some(YELLOW565)
                } else { None },
                _ => None,
            };
        }
        // Toggle strip — Yes/No ink follows the active option.
        //
        // We MUST return an explicit colour for both states (not None-when-
        // inactive), because the spec captured widget 11/17 with aux_b =
        // DAT_00ad6bc4 (yellow — the exe was in the No-selected state when
        // scraped). Falling through would let that yellow paint an inactive
        // "No", which is wrong. Force the grey ink for the inactive side.
        // GREY565 = pack565(200, 200, 200) — matches the desaturated label
        // colour used by the exe when aux_b is DAT_00acdf6e (grey).
        const GREY565: u32 = 0xCE59;
        if area_rect == TOGGLE_RECT {
            let opts = &self.state.options;
            let ink = |on: bool| Some(if on { YELLOW565 } else { GREY565 });
            return match col {
                1 => ink(opts.use_real_players),
                3 => ink(!opts.use_real_players),
                6 => ink(opts.attribute_masking),
                8 => ink(!opts.attribute_masking),
                _ => None,
            };
        }
        None
    }

    // No cell_fill for picker rows: widgets 27/29/30 have flags=0x1, NOT
    // F_SOLID_FILL. They render as plain text on the dimmed list area.

    fn cell_hidden(&self, area_rect: (i32, i32, i32, i32), _row: usize, col: usize) -> bool {
        // FUN_008055e0 (leagues draw) reads [0x9a2051] (use_real_players)
        // and SKIPS emitting the Attribute Masking group entirely when it's
        // 0. Those widgets sit at cols 5-8 of the toggle strip:
        //   col 5 = "Attribute Masking:" label
        //   col 6 = "Yes"
        //   col 8 = "No"
        if area_rect == TOGGLE_RECT && !self.state.options.use_real_players {
            return matches!(col, 5 | 6 | 7 | 8);
        }
        false
    }

    fn cell_highlight(&self, area_rect: (i32, i32, i32, i32), row: usize, col: usize) -> HighlightHint {
        // Picker rows: independent toggles per user description ("click
        // SELECTED to highlight, click again to un-highlight; same for
        // every option"). Initial state is nothing highlighted.
        if area_rect == LIST_RECT {
            let Some(slot) = self.slot_for_row(row) else { return HighlightHint::Default; };
            return match col {
                2 => if slot.selected { HighlightHint::Force(YELLOW565) } else { HighlightHint::Suppress },
                4 => if slot.background_marker { HighlightHint::Force(YELLOW565) } else { HighlightHint::Suppress },
                6 => if slot.extra && secondary_league_label(&slot.primary_name).is_some() {
                    HighlightHint::Force(YELLOW565)
                } else { HighlightHint::Suppress },
                _ => HighlightHint::Default,
            };
        }
        // Toggle strip: highlight follows options, not the static spec flag.
        // Widget 11 (Real Players No, col 3) has flags=0x801 in the spec
        // because scraping captured the 'No' state — we must suppress that
        // by default and drive the highlight from state.
        if area_rect == TOGGLE_RECT {
            let opts = &self.state.options;
            return match col {
                1 => if opts.use_real_players { HighlightHint::Force(YELLOW565) } else { HighlightHint::Suppress },
                3 => if !opts.use_real_players { HighlightHint::Force(YELLOW565) } else { HighlightHint::Suppress },
                6 => if opts.attribute_masking { HighlightHint::Force(YELLOW565) } else { HighlightHint::Suppress },
                8 => if !opts.attribute_masking { HighlightHint::Force(YELLOW565) } else { HighlightHint::Suppress },
                _ => HighlightHint::Default,
            };
        }
        HighlightHint::Default
    }
}

pub fn select_leagues(
    s: &mut Surface,
    fonts: &mut Fonts,
    bg: Option<&Image>,
    _state: &SelectLeaguesState,
    _pressed: Option<LeaguesClick>,
) {
    if let Some(spec) = load_spec(0x8055e0) {
        let provider = LeaguesProvider::new(_state.clone());
        spec.render(s, fonts, bg, &palette(), &provider);
    } else {
        blank_with_warning(s, "screens/008055e0.json not found");
    }
}

// ------- Select Start Season — spec-driven -------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonClick {
    Back,
    Next,
    Select(usize),
}

pub fn season_hit(state: &StartSeasonState, x: i32, y: i32) -> Option<SeasonClick> {
    use cm_render::layout::rebuild_layout;
    let nav = rebuild_layout((100, 555, 790, 590), 1, &[3, 1], &[1], false);
    if in_rect(x, y, nav.cell(0, 0)) {
        return Some(SeasonClick::Back);
    }
    if in_rect(x, y, nav.cell(1, 0)) {
        return Some(SeasonClick::Next);
    }
    // Season list = rebuild_layout((110, ~270, 780, 535), 2, [1,1,1], N rows)
    let n = state.rows.len().max(1);
    let list = rebuild_layout((110, 270, 780, 535), 2, &[1, 1, 1], &vec![1; n], false);
    for r in 0..n {
        if y >= list.row_top[r] && y <= list.row_bottom[r]
            && x >= list.col_left[0] && x <= list.col_right[2]
        {
            return Some(SeasonClick::Select(r));
        }
    }
    None
}

/// Whether a country uses a calendar-year season ("Finland 2002") or a
/// split-year season ("England 01/02"). Ported from the branch at
/// FUN_00807280:0x8073c8 which reads `slot_record[0x17]` (the split-season
/// flag) and picks the format string. Since our port doesn't yet load the
/// real slot records, the table below encodes the same decision by name.
/// Values verified against CM01/02 shipping data.
fn is_calendar_year_season(country: &str) -> bool {
    matches!(country,
        "Argentina" | "Brazil" | "Finland" | "Ireland" | "Japan"
        | "Norway" | "Sweden" | "USA"
    )
}

/// Format a season label the way the exe does.
///
/// From FUN_00807280 disasm:
///   split path (0x8073fe): sprintf(edi, "%s %02d/%02d", name, y, y+1)
///   calendar path (0x80741c): sprintf(edi, "%s %d", name, year)
/// The 2001/02 default season → y=1 for split, y=2002 for calendar.
fn season_label(country: &str, base_year_2digit: u32) -> String {
    if is_calendar_year_season(country) {
        // Format "%s %d" — the exe uses a full 4-digit year here.
        let full = 2000 + base_year_2digit + 1;
        format!("{country} {full}")
    } else {
        // Format "%s %02d/%02d" — start/end 2-digit zero-padded.
        let y = base_year_2digit;
        let n = (y + 1) % 100;
        format!("{country} {y:02}/{n:02}")
    }
}

/// Per-screen provider for Select Start Season (VA 0x807280). The screen's
/// menu_list has label_count = DAT_00acdf04 (selected_count) and labels_ptr
/// built at runtime — one label per SELECTED slot, format decided by that
/// country's season type (mirrors the exe's loop at 0x80739a..0x807458).
pub struct SeasonProvider {
    pub state: StartSeasonState,
    /// Snapshot of the leagues screen state — needed to know which slots
    /// were selected so we can build one label per selection.
    pub leagues: Option<SelectLeaguesState>,
    /// Index of the currently-highlighted season row (0-based). Ports the
    /// exe's DAT_00acdf08 → selected_slot_pointer comparison. Default 0:
    /// the exe seeds this to the top row when Start Season is first shown.
    pub selected_idx: usize,
}

impl SeasonProvider {
    pub fn new(state: StartSeasonState) -> Self { Self { state, leagues: None, selected_idx: 0 } }
    pub fn from_leagues(leagues: SelectLeaguesState) -> Self {
        Self { state: StartSeasonState::default(), leagues: Some(leagues), selected_idx: 0 }
    }
}

impl StateProvider for SeasonProvider {
    fn menu_labels(&self, area_rect: (i32, i32, i32, i32)) -> Option<Vec<String>> {
        // Menu at (110, 145, 780, 535) per FUN_00807280 spec.
        if area_rect != (110, 145, 780, 535) { return None; }

        // Preferred path: build labels from the selected leagues, matching
        // the exe's loop. Base year = 01 (i.e. the "2001/02" season).
        if let Some(leagues) = &self.leagues {
            let mut labels: Vec<String> = leagues
                .slots
                .iter()
                .filter(|s| s.selected)
                .map(|s| season_label(&s.primary_name, 1))
                .collect();
            if labels.is_empty() {
                // Fresh-install / no-save fallback — shipped CM01/02 shows
                // exactly "2001/02" as one row.
                labels.push("2001/02".to_string());
            }
            return Some(labels);
        }
        // Fallback: use whatever StartSeasonState carried in.
        Some(self.state.rows.iter().map(|r| r.year_label.clone()).collect())
    }

    fn menu_selected_index(&self, area_rect: (i32, i32, i32, i32)) -> Option<usize> {
        if area_rect == (110, 145, 780, 535) {
            Some(self.selected_idx)
        } else {
            None
        }
    }
}

pub fn start_season(
    s: &mut Surface,
    fonts: &mut Fonts,
    bg: Option<&Image>,
    state: &StartSeasonState,
    _pressed: Option<SeasonClick>,
) {
    // The real Start Season screen is FUN_00807280 (0x808ae0 is Manager
    // Status / Network Waiting — corrected 2026-08-20 after grep on
    // "Select Start Season" string literal.)
    if let Some(spec) = load_spec(0x807280) {
        let provider = SeasonProvider::new(state.clone());
        spec.render(s, fonts, bg, &palette(), &provider);
    } else {
        blank_with_warning(s, "screens/00807280.json not found");
    }
}

// ------- helpers -------

fn in_rect(x: i32, y: i32, r: (i32, i32, i32, i32)) -> bool {
    x >= r.0 && x <= r.2 && y >= r.1 && y <= r.3
}

/// Paint a solid dark background with a warning message. Fallback when the
/// spec JSON isn't reachable — makes the failure obvious in the UI instead
/// of silently drawing something wrong.
fn blank_with_warning(s: &mut Surface, msg: &str) {
    s.fill(20, 0, 0);
    let _ = msg; // Text render needs a font handle; log to stderr for now.
    eprintln!("[screens] {msg}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_hit_zones_are_all_reachable() {
        for (i, &(l, t, r, b, _)) in setup_buttons().iter().enumerate() {
            let cx = (l + r) / 2;
            let cy = (t + b) / 2;
            assert_eq!(setup_hit(cx, cy), Some(i));
        }
    }
}
