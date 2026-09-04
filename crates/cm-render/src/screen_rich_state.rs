//! Layer 3 — Category B rich-state screen builders.
//!
//! The dispatcher pipeline (`dispatch_global` / `dispatch_club`) takes
//! a widget pool and a cmd id, no view payload. Rich-state screens
//! (News / Dashboard / League Table / Player Profile / Club Fixtures /
//! Selected Leagues / Widget Pool Debug) carry a live cm-domain view
//! that has to feed the widget spawns — this module is where that
//! feeding happens.
//!
//! # Provenance
//!
//! Every rect + label here is either
//!
//! * from a live exe capture (News → [`crate::screen_news`], which we
//!   re-use verbatim after adapting the cm-domain [`cm_domain::NewsView`]
//!   into the [`crate::screen_news::NewsScreenState`] the builder takes), or
//!
//! * from the pre-fold hand-ported layout in
//!   `crates/cm-ui-app/src/screens.rs` — cited per-widget with the
//!   originating draw function (e.g. `screens::dashboard`,
//!   `screens::league_table`). Those layouts have not been verified
//!   against the exe pixel-for-pixel — see memory
//!   [[wired-screens-are-approximations]] — and this fold preserves
//!   the fidelity level the app already shipped rather than upgrading
//!   it (that is the next widening step).
//!
//! # Widget-pool pattern
//!
//! Every builder returns `Option<(areas_added, widgets_added)>`
//! mirroring `screen_wire_batch3::WireToPool` so tests can assert
//! spawn counts without knowing the shape of the underlying pool.
//! The builders spawn a small canvas of KIND_HEADER / KIND_LABEL /
//! KIND_BUTTON widgets — Layer 2's `packed_widget::render_widget`
//! turns those into pixels.

use crate::screen_news::{build_news_screen, NewsItem as NsItem, NewsScreenState};
use crate::widget_pool::{
    GuiRecordPool, WidgetDescriptor, KIND_BUTTON, KIND_HEADER, KIND_LABEL,
};

// ============================================================================
// Palette constants — from screens.rs's palette.rs default palette. Kept
// here as u16 RGB555 packs so the widget label_ink field (a u16) can carry
// them without a colour-space adapter every call.
// ============================================================================

/// Bright yellow — "highlight_fg" in `cm_widget::Palette::default`.
const INK_YELLOW: u16 = 0x7FE0;
/// Near-white — "near_white" body text.
const INK_NEAR_WHITE: u16 = 0x7FFF;
/// Sub-heading grey.
const INK_GREY: u16 = 0x6318;
/// Standard button ink slot (matches screen_wire_batch3::BUTTON_LABEL_INK).
const INK_BUTTON: u16 = 3;
/// Standard title label_ink (matches screen_wire_batch3::HEADER_LABEL_INK).
const INK_TITLE: u16 = 7;
/// Standard label text_style (matches screen_wire_batch3::HEADER_TEXT_STYLE).
const TS_TITLE: u32 = 12;
/// Panel style_byte (matches every batch3 panel arg8 push).
const PANEL_STYLE: u32 = 0x30;

// ============================================================================
// Local helpers — same shape as screen_wire_batch3.rs's private helpers,
// duplicated so this file does not depend on a private module.
// ============================================================================

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

/// The "sidebar + banner + nav" prelude every ported screen shares.
///
/// * Sidebar area — matches [[news-screen-geometry]] left column
///   `(0, 0, 89, 599)`.
/// * Root content area — the 800x600 canvas widgets attach to.
/// * Title banner header — same 60px band `(100, 10, 790, 70)` every
///   in-game screen paints (see `screens::dashboard`,
///   `screens::league_table`, `screens::selected_leagues`).
/// * Nav-bar Back/Next — same `(100, 555, 790, 590)` band; delegates
///   button spawns.
///
/// Returns the `root_area` handle so callers can attach content
/// widgets to it.
fn spawn_common_prelude(
    pool: &mut GuiRecordPool,
    title: &str,
) -> Option<i16> {
    // Sidebar substrate area — the exe's FUN_00745540 mode 4 owns
    // the actual sidebar; the placeholder here is what
    // `screen_wire_batch3::spawn_sidebar_placeholder` uses too.
    pool.spawn_area(
        0, 70, 99, 599, 0, Vec::new(),
        7, 0, PANEL_STYLE, 0, -1,
    )?;
    // Root content canvas.
    let root = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        7, 0, PANEL_STYLE, 0, -1,
    )? as i16;
    // Title banner header — cited: `screens::dashboard` banner rect
    // (crates/cm-ui-app/src/screens.rs:164).
    spawn_header(pool, root, 100, 10, 790, 70, title)?;
    Some(root)
}

fn spawn_nav_back_next(
    pool: &mut GuiRecordPool,
    parent_area: i16,
    back_enabled: bool,
    next_enabled: bool,
) -> Option<()> {
    // Same rects as `screen_wire_batch3::spawn_navbar_stub` — the
    // Back/Next cells of the nav strip.
    let (bink, bmsg) = if back_enabled { (INK_BUTTON, -2) } else { (INK_GREY, 0) };
    spawn_button(pool, parent_area, 338, 555, 500, 585, "Back", bink, bmsg, 0)?;
    let (nink, nmsg) = if next_enabled { (INK_BUTTON, -3) } else { (INK_GREY, 0) };
    spawn_button(pool, parent_area, 685, 555, 790, 585, "Next", nink, nmsg, 1)?;
    Some(())
}

// ============================================================================
// 1. News — [`cm_domain::NewsView`] → `screen_news::build_news_screen`
// ============================================================================

/// Adapt the cm-domain News view into the exe-decoded News state the
/// live-capture builder in [`crate::screen_news`] consumes.
///
/// * `header_title` — the domain View already stores "Christoph Olewicz
///   News" style titles verbatim.
/// * `active_tab` — the domain View's captured slot code (12 = "All",
///   see memory [[news-screen-geometry]]). The exe decodes it into a
///   0..=7 index via `selected_tab & 7`.
/// * `filter_text` — no dedicated field on `NewsView`; empty (matches
///   the closed-combo capture).
/// * `next_unread_enabled` — every unread item that has not been
///   selected yet keeps the button live.
/// * `items` — first 5 headlines feed the row list.
/// * `selected_body` — body of the currently selected item.
/// * `back_disabled` / `next_enabled` — read from the domain nav
///   flags on the View.
pub fn build_news_from_view(
    pool: &mut GuiRecordPool,
    view: &cm_domain::NewsView,
) -> Option<(usize, usize)> {
    let areas_before = pool.areas.len();
    let widgets_before = pool.widgets.len();
    let items: Vec<NsItem> = view
        .items
        .iter()
        .take(5)
        .map(|it| NsItem {
            headline: it.headline.clone(),
            unread: true, // domain View has no per-item read flag yet
            date: it.date_label.clone(),
        })
        .collect();
    let selected_body = view
        .items
        .get(view.selected_item)
        .map(|it| it.body.clone())
        .unwrap_or_default();
    let unread_remaining = view.items.len() > view.selected_item + 1;
    let state = NewsScreenState {
        header_title: view.title.clone(),
        active_tab: (view.selected_tab & 7) as u8,
        filter_text: String::new(),
        next_unread_enabled: unread_remaining,
        items,
        selected_body,
        back_disabled: !view.nav_back_enabled,
        next_enabled: view.nav_next_enabled,
    };
    build_news_screen(pool, &state)?;
    Some((pool.areas.len() - areas_before, pool.widgets.len() - widgets_before))
}

// ============================================================================
// 2. Dashboard — [`cm_domain::DashboardView`]
// ============================================================================

/// Rects cited from `crates/cm-ui-app/src/screens.rs::dashboard`
/// (banner, sub-heading, next-fixture panel, squad list header, 11
/// squad rows). Every dynamic string comes from the View.
pub fn build_dashboard_from_view(
    pool: &mut GuiRecordPool,
    view: &cm_domain::DashboardView,
    squad_scroll: usize,
) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    match view {
        cm_domain::DashboardView::Unemployed(u) => {
            let title = format!("{} — Unemployed", u.manager_name);
            let root = spawn_common_prelude(pool, &title)?;
            // Sub-heading + message (screens.rs:172-174).
            spawn_label(pool, root, 100, 80, 790, 125, &u.manager_name, INK_NEAR_WHITE, 0, 0)?;
            spawn_label(pool, root, 120, 200, 780, 240, &u.message, INK_NEAR_WHITE, 0, 1)?;
            spawn_nav_back_next(pool, root, true, false)?;
        }
        cm_domain::DashboardView::Club(d) => {
            let root = spawn_common_prelude(pool, &d.club_name)?;
            // Sub-heading (division + position) — screens.rs:182.
            let subhead = format!(
                "{}  —  Position {} of {}",
                d.division_name, d.position, d.division_size
            );
            spawn_label(pool, root, 110, 78, 780, 108, &subhead, INK_NEAR_WHITE, 0, 0)?;
            // Manager + date line — screens.rs:187.
            let mgr_line = format!(
                "Manager: {}    {}.{}.{}",
                d.manager_name, d.date.day, d.date.month, d.date.year
            );
            spawn_label(pool, root, 110, 110, 780, 138, &mgr_line, INK_GREY, 0, 1)?;
            // Next-fixture panel — screens.rs:196 (DASH_FIXTURE_LINK).
            let next_line = match &d.next_fixture {
                Some(f) => format!(
                    "Next match:  {} v {}   ({})",
                    f.home_club_name, f.away_club_name, f.competition_name
                ),
                None => "Next match:  no fixtures scheduled".to_string(),
            };
            spawn_button(pool, root, 110, 150, 780, 200, &next_line, INK_YELLOW,
                /* msg_id — screens.rs promotes the next-fixture link to
                 * the ClubFixtures screen (main.rs 638) */ 0, 2)?;
            // Squad header — screens.rs:207.
            let squad_hdr = format!("Squad ({} players)", d.squad.len());
            spawn_label(pool, root, 110, 220, 780, 248, &squad_hdr, INK_NEAR_WHITE, 0, 3)?;
            // Squad rows — 11 visible per screens.rs::DASH_SQUAD_ROWS.
            const ROW_H: i16 = 25;
            for row in 0..11usize {
                let Some(p) = d.squad.get(squad_scroll + row) else { break };
                let y = 250 + (row as i16) * ROW_H;
                let text = format!(
                    "{}    Age {}    CA {}    Cond {}",
                    p.name,
                    p.age.map(|a| a.to_string()).unwrap_or_else(|| "?".into()),
                    p.current_ability, p.condition,
                );
                spawn_label(pool, root, 110, y, 780, y + ROW_H - 2, &text,
                    INK_NEAR_WHITE, 0, (10 + row) as i32)?;
            }
            spawn_nav_back_next(pool, root, true, false)?;
        }
    }
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// 3. League Table — [`cm_domain::LeagueTableView`]
// ============================================================================

/// Rects cited from `screens::league_table` (TABLE_RECT +
/// LEAGUE_ROWS_VISIBLE = 20).
pub fn build_league_table_from_view(
    pool: &mut GuiRecordPool,
    view: &cm_domain::LeagueTableView,
    scroll: usize,
) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, &view.competition_name)?;
    // Column headers — same 10 columns as screens.rs:408.
    spawn_label(pool, root, 110, 96, 780, 118,
        "Pos   Club                        P    W    D    L    F    A   GD  Pts",
        INK_GREY, 0, 0)?;
    // Body rows — 20 visible per LEAGUE_ROWS_VISIBLE.
    const ROW_H: i16 = 20;
    for row in 0..20usize {
        let Some(r) = view.rows.get(scroll + row) else { break };
        let y = 120 + (row as i16) * ROW_H;
        let ink = if r.is_manager_club { INK_YELLOW } else { INK_NEAR_WHITE };
        let text = format!(
            "{:>3}  {:<24} {:>3} {:>3} {:>3} {:>3} {:>3} {:>3} {:>+3} {:>3}",
            r.position, truncate(&r.club_name, 24),
            r.played, r.won, r.drawn, r.lost,
            r.goals_for, r.goals_against, r.goal_difference, r.points,
        );
        spawn_label(pool, root, 110, y, 780, y + ROW_H - 2, &text, ink, 0, row as i32)?;
    }
    spawn_nav_back_next(pool, root, true, false)?;
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// 4. Player Profile — [`cm_domain::PlayerProfile`]
// ============================================================================

/// Rects cited from `screens::player_profile` (banner, born line,
/// value/wage line, positions strip, attributes grid).
pub fn build_player_profile_from_view(
    pool: &mut GuiRecordPool,
    view: &cm_domain::PlayerProfile,
) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, &view.name)?;
    let age = view.age.map(|a| a.to_string()).unwrap_or_else(|| "?".into());
    let club = view.club_name.clone().unwrap_or_else(|| "Unattached".into());
    // Age + club — screens.rs:564.
    spawn_label(pool, root, 110, 78, 780, 108,
        &format!("Age {age}    {club}"), INK_NEAR_WHITE, 0, 0)?;
    // Born line — screens.rs:567.
    spawn_label(pool, root, 110, 108, 780, 128, &view.born_line, INK_GREY, 0, 1)?;
    // Value / Wage / Caps — screens.rs:568-576.
    let money_line = format!(
        "Value {}    Wage {} p/w    Caps {} ({} goals)",
        cm_domain::cash::Money(view.value as i64).format_gbp(),
        cm_domain::cash::Money(view.wage as i64).format_gbp(),
        view.international_caps, view.international_goals,
    );
    spawn_label(pool, root, 110, 126, 780, 146, &money_line, INK_GREY, 0, 2)?;
    // Positions strip — 12 aptitudes at PROFILE_POS (110,176,780,202).
    // One row of 12 labels, evenly spaced.
    const N_POS: usize = 12;
    let pos_w: i16 = ((780 - 110) / N_POS as i16) - 2;
    for (i, (label, apt)) in view.positions.iter().enumerate() {
        let x0 = 110 + (i as i16) * (pos_w + 2);
        let ink = if *apt >= 20 { INK_YELLOW } else if *apt >= 15 { INK_NEAR_WHITE } else { INK_GREY };
        spawn_label(pool, root, x0, 176, x0 + pos_w, 202,
            &format!("{label} {apt}"), ink, 0, (10 + i) as i32)?;
    }
    // Attributes — 42 rows across 3 columns × 14 rows.
    const ATTR_ROWS: usize = 14;
    const ATTR_ROW_H: i16 = 24;
    for (i, (label, val)) in view.attributes.iter().enumerate() {
        let col = i / ATTR_ROWS;
        let row = i % ATTR_ROWS;
        if col >= 3 { break; }
        let col_w: i16 = (780 - 110) / 3;
        let x0 = 110 + (col as i16) * col_w;
        let y = 232 + (row as i16) * ATTR_ROW_H;
        let text = format!("{label:<20} {val}");
        spawn_label(pool, root, x0, y, x0 + col_w - 4, y + ATTR_ROW_H - 4,
            &text, INK_NEAR_WHITE, 0, (100 + i) as i32)?;
    }
    spawn_nav_back_next(pool, root, true, false)?;
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// 5. Club Fixtures — [`cm_domain::ClubFixturesView`]
// ============================================================================

pub fn build_club_fixtures_from_view(
    pool: &mut GuiRecordPool,
    view: &cm_domain::ClubFixturesView,
    scroll: usize,
) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, &view.club_name)?;
    // Header row — screens.rs:486.
    spawn_label(pool, root, 110, 96, 780, 118,
        "Date              Competition          H/A  Opponent                Result",
        INK_GREY, 0, 0)?;
    const ROW_H: i16 = 20;
    for row in 0..20usize {
        let Some(r) = view.rows.get(scroll + row) else { break };
        let y = 120 + (row as i16) * ROW_H;
        let result = match r.result {
            Some((us, them)) => format!("{us}-{them}"),
            None => String::new(),
        };
        let ink = if r.result.is_some() { INK_YELLOW } else { INK_NEAR_WHITE };
        let text = format!(
            "{:>2}.{:>2}.{:<4}  {:<20}  {:<3}  {:<20}  {:>5}",
            r.date.day, r.date.month, r.date.year,
            truncate(&r.competition_name, 20),
            if r.is_home { "H" } else { "A" },
            truncate(&r.opponent_name, 20), result,
        );
        spawn_label(pool, root, 110, y, 780, y + ROW_H - 2, &text, ink, 0, (10 + row) as i32)?;
    }
    spawn_nav_back_next(pool, root, true, false)?;
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// 6. Selected Leagues — Vec<NationTierAssignment> + Option<NewGameOptions>
// ============================================================================

pub fn build_selected_leagues_from_view(
    pool: &mut GuiRecordPool,
    rows: &[cm_domain::NationTierAssignment],
    options: Option<&cm_domain::NewGameOptions>,
) -> Option<(usize, usize)> {
    use cm_domain::LeagueTier;
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, "Selected Leagues")?;
    // Options block — screens.rs:641-654.
    let (real, mask, year) = options
        .map(|o| (o.use_real_players, o.attribute_masking, o.start_year))
        .unwrap_or((true, true, 0));
    let onoff = |b: bool| if b { "Yes" } else { "No" };
    let season = format!("Season {}/{:02}", year, (year + 1) % 100);
    spawn_label(pool, root, 110, 82, 780, 108, &season, INK_NEAR_WHITE, 0, 0)?;
    let opts_line = format!(
        "Real players: {}     Attribute masking: {}",
        onoff(real), onoff(mask)
    );
    spawn_label(pool, root, 110, 112, 780, 136, &opts_line, INK_GREY, 0, 1)?;
    let fg = rows.iter().filter(|r| r.tier == LeagueTier::Foreground).count();
    let bg = rows.iter().filter(|r| r.tier == LeagueTier::Background).count();
    let counts = format!(
        "{fg} playable league(s), {bg} background nation(s), {} nations loaded",
        rows.len()
    );
    spawn_label(pool, root, 110, 140, 780, 164, &counts, INK_GREY, 0, 2)?;
    // Table header + up to 12 rows (screens.rs:LEAGUES_ROWS = 12).
    spawn_label(pool, root, 110, 174, 780, 196,
        "Nation                       Status              Detailed matches",
        INK_GREY, 0, 3)?;
    let mut sorted: Vec<&cm_domain::NationTierAssignment> = rows.iter().collect();
    sorted.sort_by_key(|r| (
        match r.tier {
            LeagueTier::Foreground => 0,
            LeagueTier::Background => 1,
            _ => 2,
        },
        r.nation_name.clone(),
    ));
    const ROW_H: i16 = 22;
    for (i, r) in sorted.iter().take(12).enumerate() {
        let y = 200 + (i as i16) * ROW_H;
        let (status, ink) = match r.tier {
            LeagueTier::Foreground => ("Playable", INK_YELLOW),
            LeagueTier::Background => ("Background", INK_NEAR_WHITE),
            _ => ("Loaded only", INK_GREY),
        };
        let text = format!(
            "{:<26}  {:<16}  {}",
            truncate(&r.nation_name, 26), status, onoff(r.detailed_matches),
        );
        spawn_label(pool, root, 110, y, 780, y + ROW_H - 2, &text, ink, 0, (10 + i) as i32)?;
    }
    spawn_nav_back_next(pool, root, true, false)?;
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// 7. Widget-pool debug — dump a widget slice as coloured labels
// ============================================================================

/// Re-spawn every input widget into the pool as a KIND_LABEL with the
/// same rect + a describing text, so the packed_widget renderer draws
/// the pool the caller wants to inspect. Not a faithful port of any
/// exe function — this is app-side debug scaffolding (used by
/// `CM_BOOT=news-widgets` / `CM_BOOT=dash-widgets` in
/// `crates/cm-ui-app/src/main.rs`).
pub fn build_widget_pool_debug_from_view(
    pool: &mut GuiRecordPool,
    label: &str,
    widgets: &[crate::widget_pool::Widget],
) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();
    let root = spawn_common_prelude(pool, &format!("Widget pool: {label}"))?;
    let count_line = format!("{} widgets in pool", widgets.len());
    spawn_label(pool, root, 110, 78, 780, 108, &count_line, INK_NEAR_WHITE, 0, 0)?;
    // Show up to 24 widget descriptors as labels at their own rects.
    for (idx, w) in widgets.iter().take(24).enumerate() {
        let (l, t, r, b) = (
            w.left.max(0).min(799) as i16,
            w.top.max(30).min(599) as i16,
            w.right.max(0).min(799) as i16,
            w.bottom.max(0).min(599) as i16,
        );
        if r <= l || b <= t { continue; }
        let text = format!("#{idx} {}", if w.descriptor.text.is_empty() {
            format!("kind={} ({},{})", w.descriptor.kind, l, t)
        } else {
            w.descriptor.text.clone()
        });
        spawn_label(pool, root, l, t, r, b, &text, INK_YELLOW, 0, (10 + idx) as i32)?;
    }
    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}

// ============================================================================
// Utility
// ============================================================================

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_string() }
    else {
        let mut out: String = s.chars().take(max.saturating_sub(3)).collect();
        out.push_str("...");
        out
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget_pool::GuiRecordPool;

    fn dummy_date() -> cm_domain::GameDate {
        cm_domain::GameDate { day: 10, month: 8, year: 2001 }
    }

    #[test]
    fn news_from_view_spawns_widgets_derived_from_state() {
        let view = cm_domain::NewsView {
            title: "Test Manager News".into(),
            items: vec![
                cm_domain::NewsItem {
                    date: dummy_date(),
                    date_label: "Tue 10 Aug".into(),
                    headline: "First headline".into(),
                    body: "Body one".into(),
                    category: cm_domain::NewsCategory::Message,
                    unread: true,
                },
                cm_domain::NewsItem {
                    date: dummy_date(),
                    date_label: "Wed 11 Aug".into(),
                    headline: "Second headline".into(),
                    body: "Body two".into(),
                    category: cm_domain::NewsCategory::Message,
                    unread: true,
                },
            ],
            selected_tab: 12,
            tab_count: 8,
            selected_item: 0,
            nav_back_enabled: true,
            nav_next_enabled: false,
        };
        let mut pool = GuiRecordPool::new();
        let (a, w) = build_news_from_view(&mut pool, &view).unwrap();
        assert!(a >= 9, "News builder must spawn at least the 9 capture areas");
        assert!(w > 5, "News builder must spawn tabs + rows: got {w}");
        // Dynamic state actually reaches the pool.
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "Test Manager News"));
        assert!(pool.widgets.iter().any(|x| x.descriptor.text.contains("First headline")));
    }

    #[test]
    fn dashboard_from_club_view_spawns_squad_rows() {
        let club = cm_domain::ClubDashboard {
            manager_name: "Test Manager".into(),
            club_id: 1,
            club_name: "Test FC".into(),
            division_name: "Div 1".into(),
            position: 3,
            division_size: 20,
            date: dummy_date(),
            next_fixture: None,
            squad: (0..15).map(|i| cm_domain::SquadMember {
                player_id: i,
                name: format!("Player {i}"),
                age: Some(25),
                current_ability: 100,
                condition: 8000,
            }).collect(),
        };
        let view = cm_domain::DashboardView::Club(club);
        let mut pool = GuiRecordPool::new();
        let (_a, w) = build_dashboard_from_view(&mut pool, &view, 0).unwrap();
        assert!(w >= 11 + 4, "dashboard must spawn header + squad rows: got {w}");
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "Test FC"));
        assert!(pool.widgets.iter().any(|x| x.descriptor.text.contains("Player 0")));
    }

    #[test]
    fn dashboard_from_unemployed_view() {
        let u = cm_domain::UnemployedView {
            manager_name: "Nobody".into(),
            date: dummy_date(),
            message: "You have no job".into(),
        };
        let view = cm_domain::DashboardView::Unemployed(u);
        let mut pool = GuiRecordPool::new();
        let (_a, _w) = build_dashboard_from_view(&mut pool, &view, 0).unwrap();
        assert!(pool.widgets.iter().any(|x| x.descriptor.text.contains("Unemployed")));
    }

    #[test]
    fn league_table_rows_carry_manager_club_highlight() {
        let mgr_row = cm_domain::LeagueTableRow {
            position: 1, club_id: 42, club_name: "Manager FC".into(),
            played: 5, won: 3, drawn: 1, lost: 1,
            goals_for: 10, goals_against: 5, goal_difference: 5, points: 10,
            is_manager_club: true,
        };
        let view = cm_domain::LeagueTableView {
            competition_id: 1, competition_name: "Test Div".into(),
            rows: vec![mgr_row],
        };
        let mut pool = GuiRecordPool::new();
        build_league_table_from_view(&mut pool, &view, 0).unwrap();
        let hit = pool.widgets.iter()
            .find(|w| w.descriptor.text.contains("Manager FC"))
            .expect("manager row missing");
        assert_eq!(hit.descriptor.label_ink, INK_YELLOW,
            "manager club row must be yellow-highlighted");
    }

    #[test]
    fn player_profile_populates_positions_and_attributes() {
        let view = cm_domain::PlayerProfile {
            player_id: 1, name: "Test Player".into(),
            age: Some(28), club_name: Some("Test FC".into()),
            born_line: "Born 1.1.73 ... England.".into(),
            nation_name: Some("England".into()),
            wage: 50000, value: 1_000_000,
            international_caps: 10, international_goals: 3,
            positions: vec![("GK", 5), ("D", 20), ("M", 15)],
            attributes: (0..42).map(|i| ("Attr", i as i8)).collect(),
        };
        let mut pool = GuiRecordPool::new();
        let (_a, w) = build_player_profile_from_view(&mut pool, &view).unwrap();
        assert!(w >= 3 + 3 + 42, "profile widgets: got {w}");
        assert!(pool.widgets.iter().any(|x| x.descriptor.text == "Test Player"));
        assert!(pool.widgets.iter().any(|x| x.descriptor.text.contains("Age 28")));
    }

    #[test]
    fn club_fixtures_view_spawns_result_rows() {
        let view = cm_domain::ClubFixturesView {
            club_id: 42, club_name: "Test FC".into(),
            rows: vec![
                cm_domain::ClubFixtureRow {
                    date: dummy_date(),
                    competition_name: "Test Cup".into(),
                    opponent_name: "Away FC".into(),
                    is_home: true, result: Some((2, 1)),
                },
            ],
        };
        let mut pool = GuiRecordPool::new();
        build_club_fixtures_from_view(&mut pool, &view, 0).unwrap();
        let hit = pool.widgets.iter()
            .find(|w| w.descriptor.text.contains("Away FC"))
            .expect("fixture row missing");
        assert_eq!(hit.descriptor.label_ink, INK_YELLOW,
            "played fixtures must be highlighted");
    }

    #[test]
    fn selected_leagues_options_and_rows() {
        use cm_domain::LeagueTier;
        let rows = vec![
            cm_domain::NationTierAssignment {
                nation_id: 1, nation_name: "England".into(),
                tier: LeagueTier::Foreground, detailed_matches: true,
            },
            cm_domain::NationTierAssignment {
                nation_id: 2, nation_name: "Germany".into(),
                tier: LeagueTier::Background, detailed_matches: false,
            },
        ];
        let opts = cm_domain::NewGameOptions {
            selected_nations: vec!["England".into()],
            background_nations: vec!["Germany".into()],
            use_real_players: true,
            attribute_masking: false,
            start_year: 2001,
        };
        let mut pool = GuiRecordPool::new();
        build_selected_leagues_from_view(&mut pool, &rows, Some(&opts)).unwrap();
        assert!(pool.widgets.iter().any(|w| w.descriptor.text.contains("England")));
        assert!(pool.widgets.iter().any(|w| w.descriptor.text.contains("Playable")));
    }

    #[test]
    fn widget_pool_debug_survives_empty_input() {
        let mut pool = GuiRecordPool::new();
        build_widget_pool_debug_from_view(&mut pool, "smoke", &[]).unwrap();
        assert!(pool.widgets.iter().any(|w| w.descriptor.text.contains("Widget pool: smoke")));
    }
}
