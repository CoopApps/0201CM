//! Faithful direct-draw renderer for the club Squad page — the exe's
//! default view when you click a club on Select Team (or from any
//! in-game context that opens a club).
//!
//! Ground truth: `fixtures/club_squad_screen/exe_paint_fb.jsonl.gz`
//! (285 structural ops from `cm0102_GDI.exe` on Chester City's Squad
//! tab, captured 2026-09-06). This module ports what the Squad tab
//! itself renders:
//!
//! - In-game title bar (purple `0x331f` fill + dark-blue `0x008c` label
//!   + dark-blue bevel) — DIFFERENT from the pre-boot chrome's red bar.
//! - Small badge icon placeholder at `(105,15)-(120,35)`.
//! - **Take Control** button at `(660,4)-(785,24)` — in-game colours are
//!   INVERTED vs the pre-boot preview: dark-blue fill + purple bevel +
//!   purple text.
//! - Top tab bar `y=80..115`: five tabs — Squad (selected) / Transfers /
//!   Next Match / Fixtures / General Info. Selected tab uses a yellow
//!   `0x7fe0` bevel pattern + a yellow rect outline; unselected use the
//!   cyan `0x739c` pattern.
//! - Sub-toolbar `y=125..145`: View, Sort By, Filter buttons.
//! - Position header band `y=150..185`: darkened + yellow "Position(s)"
//!   label — the group divider the exe puts between position groups.
//! - Player list `(110,190)-(780,500)` darkened, two columns of rows at
//!   stride 21 starting `y=198`. Each row: number-cell (blue-bevelled),
//!   name (`f=3` cyan, LEFT-aligned), position code (`f=2` yellow).
//!   Selected/loaned players use white `0x7fff` text.
//! - Scrollbar `(759..778)`.
//! - Bottom tab bar `y=510..545`: Tactics / Training / Last Match /
//!   Conference / History — same panel shape as the top bar. Enabled
//!   tabs use bright cyan `0x43ff`; disabled Training uses grey `0x4210`.
//! - Bottom nav bar `y=555..590`: Back + Next.
//!
//! Chrome (sidebar / photo) is NOT painted on the in-game club page —
//! the sidebar becomes the persistent menu bar which is a separate
//! (still-unported) mechanism. For now we just paint the content area
//! and leave the left strip black.

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{
    draw_panel, PanelPalette,
    P_BEVEL, P_DARKEN, P_OUTER_HIGHLIGHT, P_SOLID_FILL,
};
use crate::packed_text::{draw_wrapped_text, W_LEFT};
use crate::screen_pre_boot_chrome::{
    c_string, F_BODY, F_SMALL, F_TITLE, GREY_BAR, INK_CYAN,
    INK_YELLOW, TS_CENTRE,
};

// -----------------------------------------------------------------------
// In-game colour palette (from op indices in structure.txt)
// -----------------------------------------------------------------------

/// In-game title-bar / panel fill. `0x331f` = light purple/lavender.
const IG_TITLE_FILL: u16 = 0x331f;
/// In-game title-bar accent / label ink. `0x008c` = dark blue.
const IG_TITLE_INK: u16 = 0x008c;
/// Tab-panel fill (top + bottom tab bars). `0x100c` = dark navy.
const TAB_FILL: u16 = 0x100c;
/// Number-cell blue (row leader square). `0x0010`.
const BLUE: u16 = 0x0010;
/// Bright cyan for enabled bottom-tab labels. `0x43ff`.
const CYAN_BRIGHT: u16 = 0x43ff;
/// White for special player names (loan / on-list markers). `0x7fff`.
const WHITE: u16 = 0x7fff;

// -----------------------------------------------------------------------
// Take Control button (identical to screen_club_preview_faithful, but
// with the in-game colours from THIS capture: dark-blue fill + purple
// bevel + purple text — inverted vs the pre-boot preview).
// -----------------------------------------------------------------------

pub const TAKE_CONTROL_RECT: (i32, i32, i32, i32) = (660, 4, 785, 24);

// -----------------------------------------------------------------------
// State
// -----------------------------------------------------------------------

pub struct SquadPlayer<'a> {
    pub name: &'a str,
    pub position: &'a str,
    /// Age in years; `None` when the DB has no DOB for this person.
    pub age: Option<u8>,
    /// A marker rendered after the name — e.g. "*" for on the transfer
    /// list, empty when nothing special. Rendered in white ink like the
    /// exe capture.
    pub marker: char,
}

pub struct SquadState<'a> {
    /// Club name — goes in the in-game title bar.
    pub club_name: &'a str,
    pub players: &'a [SquadPlayer<'a>],
    /// First-visible row (0 = top).
    pub scroll: usize,
}

// -----------------------------------------------------------------------
// Layout constants — every rect direct from structure.txt.
// -----------------------------------------------------------------------

// Top tab bar
const TAB_Y0: i32 = 80;
const TAB_Y1: i32 = 115;
const TOP_TABS: [(i32, i32, &str); 5] = [
    (100, 237, "Squad"),
    (239, 375, "Transfers"),
    (377, 513, "Next Match"),
    (515, 651, "Fixtures"),
    (653, 790, "General Info"),
];

// Sub-toolbar (View / Sort By / Filter)
const TB_Y0: i32 = 125;
const TB_Y1: i32 = 145;
const TOOLBAR_LEFT_L: (i32, i32) = (110, 234);
const TOOLBAR_LEFT_R: (i32, i32) = (236, 360);
const TOOLBAR_FILTER: (i32, i32) = (656, 780);

// Position header
const HDR_Y0: i32 = 150;
const HDR_Y1: i32 = 185;

// Player list
const LIST_X0: i32 = 110;
const LIST_X1: i32 = 780;
const LIST_Y0: i32 = 190;
const LIST_Y1: i32 = 500;
const ROW_FIRST_Y: i32 = 198;
const ROW_STRIDE: i32 = 21;
const ROW_HEIGHT: i32 = 19;
// Sub-cells per entry
const NUM_L:  (i32, i32) = (112, 144);
const NAME_L: (i32, i32) = (175, 342);
const POS_L:  (i32, i32) = (344, 433);
const NUM_R:  (i32, i32) = (435, 467);
const NAME_R: (i32, i32) = (497, 665);
const POS_R:  (i32, i32) = (667, 756);

// Scrollbar
const SB_X0: i32 = 759;
const SB_X1: i32 = 778;
const SB_TOP_ARROW: (i32, i32) = (198, 217);
const SB_BOT_ARROW: (i32, i32) = (473, 492);
const SB_TRACK_Y0: i32 = 218;
const SB_TRACK_Y1: i32 = 472;

// Bottom tab bar
const BTB_Y0: i32 = 510;
const BTB_Y1: i32 = 545;
const BOT_TABS: [(i32, i32, &str, bool); 5] = [
    (100, 237, "Tactics",    true),   // enabled cyan
    (239, 375, "Training",   false),  // disabled grey
    (377, 513, "Last Match", true),
    (515, 651, "Conference", true),
    (653, 790, "History",    true),
];

// Bottom nav
const NAV_Y0: i32 = 555;
const NAV_Y1: i32 = 590;
const NAV_BACK: (i32, i32) = (100, 617);
const NAV_NEXT: (i32, i32) = (619, 790);

// Visible rows in the list (17 rows × 2 cols = 34 players).
pub const VISIBLE_ROWS: usize = 15;
pub const VISIBLE_ENTRIES: usize = VISIBLE_ROWS * 2;

// -----------------------------------------------------------------------
// Renderer
// -----------------------------------------------------------------------

pub fn render_squad(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &SquadState<'_>,
) {
    let palette = PanelPalette::default();
    let title_font = fonts.pixel_slot(F_TITLE).clone();
    let body_font  = fonts.pixel_slot(F_BODY).clone();
    let small_font = fonts.pixel_slot(F_SMALL).clone();

    // ---- In-game TITLE BAR (100,10)-(790,70) — purple fill + dark-blue bevel.
    draw_panel(surface, 100, 10, 790, 70,
        P_SOLID_FILL | P_BEVEL, IG_TITLE_FILL, IG_TITLE_INK, palette);
    let mut title_bytes = state.club_name.as_bytes().to_vec();
    title_bytes.push(0);
    draw_wrapped_text(surface, 100, 10, 790, 70,
        &title_font, &title_bytes, IG_TITLE_INK, TS_CENTRE, -1);
    // Small badge placeholder — same purple/blue treatment.
    draw_panel(surface, 105, 15, 120, 35,
        P_SOLID_FILL | P_BEVEL, IG_TITLE_FILL, IG_TITLE_INK, palette);

    // ---- Take Control button (660,4)-(785,24) — dark-blue fill,
    //      purple bevel, purple text.
    let (tx0, ty0, tx1, ty1) = TAKE_CONTROL_RECT;
    draw_panel(surface, tx0, ty0, tx1, ty1,
        P_SOLID_FILL | P_BEVEL, IG_TITLE_INK, IG_TITLE_FILL, palette);
    surface.draw_rectangle(tx0, ty0, tx1, ty1, 4, IG_TITLE_INK);
    draw_wrapped_text(surface, tx0, ty0, tx1, ty1,
        &small_font, &c_string(b"Take Control"),
        IG_TITLE_FILL, TS_CENTRE, -1);

    // ---- Top tab bar. Squad (idx 0) is the current tab — yellow
    //      pattern + outer-highlight + explicit yellow rect outline.
    for (i, (x0, x1, label)) in TOP_TABS.iter().copied().enumerate() {
        let selected = i == 0;
        let style = if selected {
            P_SOLID_FILL | P_BEVEL | P_OUTER_HIGHLIGHT
        } else {
            P_SOLID_FILL | P_BEVEL
        };
        let pattern = if selected { INK_YELLOW } else { INK_CYAN };
        draw_panel(surface, x0, TAB_Y0, x1, TAB_Y1, style,
                   TAB_FILL, pattern, palette);
        if selected {
            surface.draw_rectangle(x0 - 1, TAB_Y0 - 1,
                                    x1 + 1, TAB_Y1 + 1,
                                    2, INK_YELLOW);
        }
        let ink = if selected { INK_YELLOW } else { INK_CYAN };
        draw_wrapped_text(surface, x0, TAB_Y0, x1, TAB_Y1,
            &small_font, &c_string(label.as_bytes()),
            ink, TS_CENTRE, -1);
    }

    // ---- Sub-toolbar (View / Sort By / Filter).
    for (rect, label) in [
        (TOOLBAR_LEFT_L, "View"),
        (TOOLBAR_LEFT_R, "Sort By"),
        (TOOLBAR_FILTER, "Filter"),
    ] {
        draw_panel(surface, rect.0, TB_Y0, rect.1, TB_Y1,
            P_SOLID_FILL | P_BEVEL, GREY_BAR, INK_CYAN, palette);
        draw_wrapped_text(surface, rect.0, TB_Y0, rect.1, TB_Y1,
            &small_font, &c_string(label.as_bytes()),
            INK_CYAN, TS_CENTRE, -1);
    }

    // ---- Position header band.
    draw_panel(surface, LIST_X0, HDR_Y0, LIST_X1, HDR_Y1,
        P_DARKEN, 0, 0, palette);
    draw_wrapped_text(surface, LIST_X0, HDR_Y0, LIST_X1, HDR_Y1,
        &body_font, &c_string(b"Position(s)"),
        INK_YELLOW, TS_CENTRE, -1);

    // ---- Player list container.
    draw_panel(surface, LIST_X0, LIST_Y0, LIST_X1, LIST_Y1,
        P_DARKEN, 0, 0, palette);
    let visible = state.players.iter().skip(state.scroll).take(VISIBLE_ENTRIES);
    for (i, p) in visible.enumerate() {
        let row_idx = i / 2;
        let is_left = i % 2 == 0;
        let y0 = ROW_FIRST_Y + (row_idx as i32) * ROW_STRIDE;
        let y1 = y0 + ROW_HEIGHT;
        let (num, name, pos) = if is_left {
            (NUM_L, NAME_L, POS_L)
        } else {
            (NUM_R, NAME_R, POS_R)
        };
        // Number cell — blue bevel.
        draw_panel(surface, num.0, y0, num.1, y1,
            P_SOLID_FILL | P_BEVEL, BLUE, INK_CYAN, palette);
        // Name — cyan (or white for marked players).
        let name_ink = if p.marker != ' ' { WHITE } else { INK_CYAN };
        let mut buf = format!("  {}", p.name);
        if p.marker != ' ' { buf.push(p.marker); }
        buf.push('\0');
        draw_wrapped_text(surface, name.0, y0, name.1, y1,
            &body_font, buf.as_bytes(), name_ink,
            TS_CENTRE | W_LEFT, -1);
        // Position — yellow.
        draw_wrapped_text(surface, pos.0, y0, pos.1, y1,
            &small_font, &c_string(p.position.as_bytes()),
            INK_YELLOW, TS_CENTRE, -1);
    }

    // ---- Scrollbar.
    draw_panel(surface, SB_X0, SB_TOP_ARROW.0, SB_X1, SB_TOP_ARROW.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_panel(surface, SB_X0, SB_TRACK_Y0, SB_X1, SB_TRACK_Y1,
        P_DARKEN, 0, 0, palette);
    let total = state.players.len().max(1);
    let track_h = (SB_TRACK_Y1 - SB_TRACK_Y0).max(1) as f32;
    let visible_frac = (VISIBLE_ENTRIES as f32 / total as f32).min(1.0);
    let thumb_h = (track_h * visible_frac).max(20.0).min(track_h) as i32;
    let max_scroll = total.saturating_sub(VISIBLE_ENTRIES);
    let thumb_y0 = if max_scroll == 0 { SB_TRACK_Y0 } else {
        SB_TRACK_Y0
            + ((track_h - thumb_h as f32) * (state.scroll as f32 / max_scroll as f32)) as i32
    };
    let thumb_y1 = (thumb_y0 + thumb_h).min(SB_TRACK_Y1);
    draw_panel(surface, SB_X0, thumb_y0, SB_X1, thumb_y1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    draw_panel(surface, SB_X0, SB_BOT_ARROW.0, SB_X1, SB_BOT_ARROW.1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);

    // ---- Bottom tab bar (visual only — click handling comes later).
    for (x0, x1, label, enabled) in BOT_TABS.iter().copied() {
        draw_panel(surface, x0, BTB_Y0, x1, BTB_Y1,
            P_SOLID_FILL | P_BEVEL, TAB_FILL,
            if enabled { CYAN_BRIGHT } else { GREY_BAR }, palette);
        let ink = if enabled { CYAN_BRIGHT } else { GREY_BAR };
        draw_wrapped_text(surface, x0, BTB_Y0, x1, BTB_Y1,
            &small_font, &c_string(label.as_bytes()),
            ink, TS_CENTRE, -1);
    }

    // ---- Bottom nav Back / Next (both grey / cyan).
    for (rect, label) in [(NAV_BACK, "Back"), (NAV_NEXT, "Next")] {
        draw_panel(surface, rect.0, NAV_Y0, rect.1, NAV_Y1,
            P_SOLID_FILL | P_BEVEL, GREY_BAR, INK_CYAN, palette);
        draw_wrapped_text(surface, rect.0, NAV_Y0, rect.1, NAV_Y1,
            &body_font, &c_string(label.as_bytes()),
            INK_CYAN, TS_CENTRE, -1);
    }
}

/// Format age line for a player row — "Rose, M · 24" style. Kept
/// separate so callers can plug it into a Sort-By-Age view later.
pub fn name_with_age(name: &str, age: Option<u8>) -> String {
    match age {
        Some(a) => format!("{name} · {a}"),
        None    => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_squad_smoke() {
        let mut surface = PackedSurface::rgb555(800, 600);
        let mut fonts = Fonts::new("D:/cm0102/Data");
        let players: Vec<SquadPlayer> = vec![];
        let state = SquadState {
            club_name: "Chester City",
            players: &players,
            scroll: 0,
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_squad(&mut surface, &mut fonts, &state);
        }));
    }
}
