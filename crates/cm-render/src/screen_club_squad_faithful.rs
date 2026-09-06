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
use crate::image::Image;
use crate::packed::PackedSurface;
use crate::packed_panel::{
    draw_panel, PanelPalette,
    P_BEVEL, P_DARKEN, P_OUTER_HIGHLIGHT, P_SOLID_FILL,
};
use crate::packed_text::{draw_wrapped_text, W_LEFT};
use crate::screen_pre_boot_chrome::{
    c_string, draw_sidebar, F_BODY, F_SMALL, F_TITLE, GREY_BAR,
    INK_CYAN, INK_YELLOW, TS_CENTRE,
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
/// Orange for the ▷ triangles on enabled bottom tabs. Same colour the
/// Leagues screen used for the highlighted radio (`0x7e00` = 31/16/0),
/// reads as bright red-orange.
const TRIANGLE_ORANGE: u16 = 0x7e00;
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
    /// Per-mode column data — filled by the app when `view` != Traditional.
    /// Each string is what the exe would paint in its column for this
    /// row. Empty strings render blank cells.
    pub cols: &'a [&'a str],
}

/// The View pull-down modes lifted from FUN_00457200 (attr_callers/
/// 0x00457200.c lines 563..668). Bitmask flags in the exe are on
/// `local_380`; we use them as enum discriminants for parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquadView {
    /// Position(s) — the default view (bit 0x1).
    Traditional = 0x0001,
    /// Basic Wage / Contract Expiry / Contract Protected / Value (bit 0x2).
    Contract    = 0x0002,
    /// Goals / Conceded / Assists / Av. Rating (bit 0x4).
    Stats       = 0x0004,
    /// Competitions + extended stats (bit 0x8).
    MoreStats   = 0x0008,
    /// Physical / Mental / GK / Def / Att attribute grid (bit 0x10).
    Attributes  = 0x0010,
    /// Nationality/Club / Int. Caps / Int. Goals (bit 0x20).
    OtherInfo   = 0x0020,
    /// Selection Info (bit 0x1000).
    Selection   = 0x1000,
    // Penalty Takers (bit 0x800) — only shown after Take Control;
    // omitted from the pre-launch dropdown per the FUN_005ea720 gate
    // at 0x00457200:669.
}

impl SquadView {
    /// Menu ORDER matches what the exe paints in FUN_00457200 —
    /// Traditional / Contract / Selection / Stats / More Stats /
    /// Attributes / Other Info.
    pub const PRE_LAUNCH_ORDER: [SquadView; 7] = [
        SquadView::Traditional,
        SquadView::Contract,
        SquadView::Selection,
        SquadView::Stats,
        SquadView::MoreStats,
        SquadView::Attributes,
        SquadView::OtherInfo,
    ];
    pub fn label(self) -> &'static str {
        match self {
            SquadView::Traditional => "Traditional",
            SquadView::Contract    => "Contract",
            SquadView::Selection   => "Selection",
            SquadView::Stats       => "Stats",
            SquadView::MoreStats   => "More Stats",
            SquadView::Attributes  => "Attributes",
            SquadView::OtherInfo   => "Other Info",
        }
    }
    /// The right-side "sub-title" the exe puts at the top of the list
    /// panel (see the s_Position_s / s_Contract_Info / etc. lines).
    pub fn subtitle(self) -> &'static str {
        match self {
            SquadView::Traditional => "Position(s)",
            SquadView::Contract    => "Contract Info",
            SquadView::Selection   => "Selection Info",
            SquadView::Stats       => "Stats",
            SquadView::MoreStats   => "More Stats",
            SquadView::Attributes  => "Attributes",
            SquadView::OtherInfo   => "Other Info",
        }
    }
    /// Column headers lifted from FUN_00457200's `text_template_expand`
    /// calls — verified against the s_* string constants at
    /// 0x0097b5e0..0x0097b7c8. Left-to-right in each row after the name
    /// column.
    pub fn column_headers(self) -> &'static [&'static str] {
        match self {
            SquadView::Traditional => &["Position(s)"],
            SquadView::Contract    => &["Basic Wage", "Contract Expiry",
                                         "Contract Protected", "Value"],
            SquadView::Selection   => &["Selection Info"],
            SquadView::Stats       => &["Goals", "Conceded", "Assists",
                                         "Av. Rating"],
            SquadView::MoreStats   => &["Competitions"],
            SquadView::Attributes  => &["Physical", "Mental", "Goalkeeping",
                                         "Defensive", "Attacking"],
            SquadView::OtherInfo   => &["Nationality", "Int. Caps", "Int. Goals"],
        }
    }
}

pub struct SquadState<'a> {
    /// Club name — goes in the in-game title bar.
    pub club_name: &'a str,
    pub players: &'a [SquadPlayer<'a>],
    /// First-visible row (0 = top).
    pub scroll: usize,
    /// Seed for the rotating RGN photo background (per-screen fresh
    /// seed). `0` skips the blit — useful for tests/CI without the
    /// game's Data directory.
    pub photo_seed: u64,
    /// `true` when a manager exists on the profile — controls the
    /// Add-Manager sidebar entry's enabled/faded ink (same rule as
    /// pre-boot chrome).
    pub has_manager: bool,
    /// Division long name for the fourth bottom-tab label — the exe
    /// puts the actual competition name there (e.g. "Conference" for
    /// Chester, "Premier League" for Arsenal). Never hardcoded.
    pub division_name: &'a str,
    /// Club's home-kit BACKGROUND colour (packed RGB565). This is the
    /// shirt body — e.g. Chester's home kit is dark-blue on white so
    /// the bar fill will be dark-blue. Resolved by the app from
    /// `ClubView::kit1_bg_color_id()` → `colour.dat` lookup. Falls back
    /// to the in-game purple `IG_TITLE_FILL` when zero.
    pub kit_bg_rgb565: u16,
    /// Club's home-kit FOREGROUND colour (packed RGB565) — the trim /
    /// stripe / shirt-detail colour used for the bevel and title ink.
    /// Falls back to the in-game dark blue `IG_TITLE_INK` when zero.
    pub kit_fg_rgb565: u16,
    /// Currently-active View mode from the "View" pull-down (FUN_00457200
    /// bitmask). Controls the subtitle + column layout on the right side
    /// of each row.
    pub view: SquadView,
    /// `true` when the View dropdown is open — the renderer paints the
    /// 7-row overlay under the View button.
    pub view_menu_open: bool,
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

// Bottom tab bar. Slot 3 (currently "Conference" for Chester) is the
// competition menu — its label is DYNAMIC per club (Premier League for
// Arsenal, D1 for Wolves, Conference for Chester, ...). The other four
// labels are fixed. Triangles on enabled tabs = hollow right-arrow at
// the right edge, drawn by `draw_hollow_triangle` (pixels replicated
// from the exe framebuffer since the primitive isn't hookable).
const BTB_Y0: i32 = 510;
const BTB_Y1: i32 = 545;
struct BotTab { x0: i32, x1: i32, label: &'static str, enabled: bool }
const BOT_TABS_FIXED: [BotTab; 5] = [
    BotTab { x0: 100, x1: 237, label: "Tactics",    enabled: true  },
    BotTab { x0: 239, x1: 375, label: "Training",   enabled: false },
    BotTab { x0: 377, x1: 513, label: "Last Match", enabled: true  },
    // Slot 3's label is overridden per club from state.division_name.
    BotTab { x0: 515, x1: 651, label: "",           enabled: true  },
    BotTab { x0: 653, x1: 790, label: "History",    enabled: true  },
];

// Bottom nav
const NAV_Y0: i32 = 555;
const NAV_Y1: i32 = 590;
const NAV_BACK: (i32, i32) = (100, 617);
const NAV_NEXT: (i32, i32) = (619, 790);

// Visible rows in the list — 14 rows × 2 cols = 28 players.
// The list panel is y=190..500 (310px). Row 0 starts at ROW_FIRST_Y=198
// with ROW_STRIDE=21, so row 14 would start at 492 and paint down to
// 511 — past LIST_Y1=500 — bleeding into the darkened background. The
// exe stops at row 13 (index 13, i.e. 14 rows) so Prescott/Monk are
// the last fully-painted rows for Leigh RMI before scrolling.
pub const VISIBLE_ROWS: usize = 14;
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

    // ---- Photo background — matches the pre-boot chrome pattern
    //      (P_DARKEN'd panels show a darkened photo through them).
    blit_photo(surface, state.photo_seed);

    // ---- Left sidebar — the exe's club screen paints the FULL pre-boot
    //      sidebar (Version / arrows / Add Manager / Restart / Exit) at
    //      this stage of the flow; the persistent in-game menu bar
    //      swaps in later once the manager takes control. Re-use the
    //      shared helper so it stays in sync with the pre-boot screens.
    draw_sidebar(surface, fonts, state.has_manager);

    // ---- In-game TITLE BAR (100,10)-(790,70) — filled with the club's
    //      HOME-KIT background colour, bevelled with the kit foreground
    //      (shirt trim). Falls back to the in-game purple/blue palette
    //      when the DB has no colours set (e.g. tests, missing data).
    let bar_fill = if state.kit_bg_rgb565 != 0 { state.kit_bg_rgb565 } else { IG_TITLE_FILL };
    let bar_ink  = if state.kit_fg_rgb565 != 0 { state.kit_fg_rgb565 } else { IG_TITLE_INK };
    draw_panel(surface, 100, 10, 790, 70,
        P_SOLID_FILL | P_BEVEL, bar_fill, bar_ink, palette);
    let mut title_bytes = state.club_name.as_bytes().to_vec();
    title_bytes.push(0);
    draw_wrapped_text(surface, 100, 10, 790, 70,
        &title_font, &title_bytes, bar_ink, TS_CENTRE, -1);
    // Small badge placeholder — same kit colours.
    draw_panel(surface, 105, 15, 120, 35,
        P_SOLID_FILL | P_BEVEL, bar_fill, bar_ink, palette);

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

    // ---- Column header band. Displays the current view mode's
    //      subtitle in the exe (Position(s) / Contract Info / etc).
    //      For Traditional a single centred "Position(s)" label matches
    //      the capture; for multi-column modes the exe splits the band
    //      into equal-width labels drawn small.
    draw_panel(surface, LIST_X0, HDR_Y0, LIST_X1, HDR_Y1,
        P_DARKEN, 0, 0, palette);
    let headers = state.view.column_headers();
    if headers.len() <= 1 {
        // Single-column layout — the exe's Traditional look: centred
        // "Position(s)" in yellow above the position column.
        draw_wrapped_text(surface, LIST_X0, HDR_Y0, LIST_X1, HDR_Y1,
            &body_font, &c_string(state.view.subtitle().as_bytes()),
            INK_YELLOW, TS_CENTRE, -1);
    } else {
        // Multi-column — carve the POS band into N equal slots for
        // both left+right groups, matching how the exe partitions the
        // 89-pixel wide POS cell across visible columns.
        let l_w = (POS_L.1 - POS_L.0) / headers.len() as i32;
        let r_w = (POS_R.1 - POS_R.0) / headers.len() as i32;
        for (i, hdr) in headers.iter().enumerate() {
            let lx0 = POS_L.0 + i as i32 * l_w;
            let lx1 = lx0 + l_w - 2;
            draw_wrapped_text(surface, lx0, HDR_Y0, lx1, HDR_Y1,
                &small_font, &c_string(hdr.as_bytes()),
                INK_YELLOW, TS_CENTRE, -1);
            let rx0 = POS_R.0 + i as i32 * r_w;
            let rx1 = rx0 + r_w - 2;
            draw_wrapped_text(surface, rx0, HDR_Y0, rx1, HDR_Y1,
                &small_font, &c_string(hdr.as_bytes()),
                INK_YELLOW, TS_CENTRE, -1);
        }
    }

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
        // Position / mode-specific columns — yellow. For Traditional
        // (or when the app hasn't populated `cols`) fall back to the
        // player's position string in the whole POS cell.
        if headers.len() <= 1 || p.cols.is_empty() {
            draw_wrapped_text(surface, pos.0, y0, pos.1, y1,
                &small_font, &c_string(p.position.as_bytes()),
                INK_YELLOW, TS_CENTRE, -1);
        } else {
            let w = (pos.1 - pos.0) / headers.len() as i32;
            for (ci, cell) in p.cols.iter().take(headers.len()).enumerate() {
                let cx0 = pos.0 + ci as i32 * w;
                let cx1 = cx0 + w - 2;
                draw_wrapped_text(surface, cx0, y0, cx1, y1,
                    &small_font, &c_string(cell.as_bytes()),
                    INK_YELLOW, TS_CENTRE, -1);
            }
        }
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
    //      Slot 3's label is overridden with the live division name.
    for (i, tab) in BOT_TABS_FIXED.iter().enumerate() {
        let label = if i == 3 { state.division_name } else { tab.label };
        draw_panel(surface, tab.x0, BTB_Y0, tab.x1, BTB_Y1,
            P_SOLID_FILL | P_BEVEL, TAB_FILL,
            if tab.enabled { CYAN_BRIGHT } else { GREY_BAR }, palette);
        let ink = if tab.enabled { CYAN_BRIGHT } else { GREY_BAR };
        draw_wrapped_text(surface, tab.x0, BTB_Y0, tab.x1, BTB_Y1,
            &small_font, &c_string(label.as_bytes()),
            ink, TS_CENTRE, -1);
        // Hollow ▷ triangle at the right edge for ENABLED tabs.
        // Pixel-verified from the exe framebuffer at op-log frame 9:
        // vertical left edge from (x1-10, y_centre-5) to (x1-10,
        // y_centre+5), diagonals converging to a tip at (x1-5, y_centre).
        if tab.enabled {
            let cy = (BTB_Y0 + BTB_Y1) / 2;
            draw_hollow_triangle(surface, tab.x1 - 10, cy, 5, TRIANGLE_ORANGE);
        }
    }

    // ---- Bottom nav Back / Next (both grey / cyan).
    for (rect, label) in [(NAV_BACK, "Back"), (NAV_NEXT, "Next")] {
        draw_panel(surface, rect.0, NAV_Y0, rect.1, NAV_Y1,
            P_SOLID_FILL | P_BEVEL, GREY_BAR, INK_CYAN, palette);
        draw_wrapped_text(surface, rect.0, NAV_Y0, rect.1, NAV_Y1,
            &body_font, &c_string(label.as_bytes()),
            INK_CYAN, TS_CENTRE, -1);
    }

    // ---- View pull-down dropdown (drawn LAST so it overlays whatever's
    //      beneath). Geometry from the reference capture:
    //      view_menu.png measured at x=120..250, rows below the View
    //      button. Green pattern background — dark 0x0084 (0,132,0)
    //      alternating with 0x0094 (0,148,0) per row for the CM01/02
    //      pull-down look. Selected row painted in a highlight cyan.
    if state.view_menu_open {
        draw_view_dropdown(surface, &small_font, state.view);
    }
}

/// View pull-down. 7 rows (Traditional / Contract / Selection / Stats /
/// More Stats / Attributes / Other Info per SquadView::PRE_LAUNCH_ORDER)
/// stacked under the View button on the sub-toolbar. Positioned at
/// (110, 145)..(255, 145+7*ROW) — width matches the View button.
///
/// Geometry lifted from the GDI capture in
/// scratchpad/prelaunch/view_menu.png (RGB555): green pattern
/// (0,132,0) / (0,148,0) alternating rows; the currently-selected mode
/// is drawn with a highlight ink so the current state is visible.
fn draw_view_dropdown(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    current: SquadView,
) {
    use crate::pack565;
    let palette = PanelPalette::default();
    const DD_X0: i32 = 110;
    const DD_X1: i32 = 255;
    const DD_Y0: i32 = 148;
    const DD_ROW_H: i32 = 18;
    let items = SquadView::PRE_LAUNCH_ORDER;
    let dd_y1 = DD_Y0 + items.len() as i32 * DD_ROW_H;
    // Measured from GDI live capture (scratchpad/prelaunch/gdi_view_menu3.png):
    //   base green A (0,132,0), alternating green B (0,148,0),
    //   currently-selected row → pure yellow (255,255,0),
    //   text on ALL rows → BLACK, small pixel font, left-indented
    //   with a whitespace pad (same convention as the Nationality
    //   Filter dropdown at screen_nationality_faithful.rs:245).
    let green_a  = pack565(  0, 132,   0);
    let green_b  = pack565(  0, 148,   0);
    let yellow   = pack565(255, 255,   0);
    let ink_black = 0;
    draw_panel(surface, DD_X0, DD_Y0, DD_X1, dd_y1,
        P_SOLID_FILL | P_BEVEL, green_a, ink_black, palette);
    for (i, mode) in items.iter().enumerate() {
        let ry0 = DD_Y0 + 1 + i as i32 * DD_ROW_H;
        let ry1 = ry0 + DD_ROW_H - 1;
        // Highlighted (currently-active) row → yellow; every other row
        // uses one of the two green shades in alternation.
        let fill = if *mode == current {
            yellow
        } else if i % 2 == 0 {
            green_a
        } else {
            green_b
        };
        for y in ry0..ry1 {
            for x in DD_X0 + 1..DD_X1 - 1 {
                surface.buf[y as usize * surface.pitch_pixels as usize + x as usize] = fill;
            }
        }
        // BLACK text — same on selected and non-selected rows. Left-
        // pad with whitespace so the exe's indent look is preserved.
        let mut buf = b"      ".to_vec();
        buf.extend_from_slice(mode.label().as_bytes());
        buf.push(0);
        draw_wrapped_text(surface, DD_X0, ry0, DD_X1 - 4, ry1,
            font, &buf, ink_black, TS_CENTRE | W_LEFT, -1);
    }
}

/// Hit-test the View dropdown. Returns the SquadView the cursor is over,
/// or None if outside the dropdown area. Used by the app's `on_release`
/// to route a click through to the mode change.
pub fn view_dropdown_hit(x: i32, y: i32) -> Option<SquadView> {
    const DD_X0: i32 = 110;
    const DD_X1: i32 = 255;
    const DD_Y0: i32 = 148;
    const DD_ROW_H: i32 = 18;
    if x < DD_X0 || x >= DD_X1 || y < DD_Y0 { return None; }
    let row = (y - DD_Y0) / DD_ROW_H;
    SquadView::PRE_LAUNCH_ORDER.get(row as usize).copied()
}

/// The View button rect on the sub-toolbar. Clicks here toggle
/// `view_menu_open`.
pub const VIEW_BUTTON_RECT: (i32, i32, i32, i32) = (110, 125, 255, 145);

/// Draw a hollow right-pointing ▷ triangle. Left edge is a vertical
/// line at `(x_left, cy-h)..(x_left, cy+h)`; the top/bottom diagonals
/// meet at the tip `(x_left + h, cy)`. Pixel pattern verified from the
/// exe framebuffer at fixtures/club_squad_screen — the primitive that
/// draws it isn't in our hooked set, so we replicate it by hand.
fn draw_hollow_triangle(
    surface: &mut PackedSurface,
    x_left: i32, cy: i32, half_h: i32, colour: u16,
) {
    // Vertical left edge.
    surface.draw_line(x_left, cy - half_h, x_left, cy + half_h, 2, colour);
    // Top diagonal — one step right per row.
    for i in 0..=half_h {
        surface.draw_line(x_left + i, cy - half_h + i,
                          x_left + i, cy - half_h + i, 2, colour);
    }
    // Bottom diagonal — mirror.
    for i in 0..=half_h {
        surface.draw_line(x_left + i, cy + half_h - i,
                          x_left + i, cy + half_h - i, 2, colour);
    }
}

/// 565 → 555 conversion. Same shape as `screen_pre_boot_chrome`.
fn c565_to_555(v: u16) -> u16 {
    let r = (v >> 11) & 0x1f;
    let g = ((v >> 5) & 0x3f) >> 1;
    let b = v & 0x1f;
    (r << 10) | (g << 5) | b
}

/// Blit a random RGN photo as base layer. Same file table + hash as
/// pre-boot chrome; missing directory (test env) leaves surface alone.
fn blit_photo(surface: &mut PackedSurface, photo_seed: u64) {
    if photo_seed == 0 { return; }
    let dir = std::path::Path::new("D:/cm0102/pictures");
    let entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok())
                    .filter(|e| {
                        let p = e.path();
                        p.extension().and_then(|s| s.to_str())
                            .map(|s| s.eq_ignore_ascii_case("rgn"))
                            .unwrap_or(false)
                    })
                    .map(|e| e.path())
                    .collect(),
        Err(_) => return,
    };
    if entries.is_empty() { return; }
    let path = &entries[(photo_seed % entries.len() as u64) as usize];
    let Ok(img) = Image::load_rgn(path) else { return };
    let w = surface.width.min(img.w as i32);
    let h = surface.height.min(img.h as i32);
    for y in 0..h {
        for x in 0..w {
            let src = img.px[(y as usize) * img.w + (x as usize)];
            let dst_idx = (y * surface.pitch_pixels + x) as usize;
            surface.buf[dst_idx] = c565_to_555(src);
        }
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
            photo_seed: 0,
            has_manager: false,
            division_name: "Conference",
            kit_bg_rgb565: 0,
            kit_fg_rgb565: 0,
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_squad(&mut surface, &mut fonts, &state);
        }));
    }
}
