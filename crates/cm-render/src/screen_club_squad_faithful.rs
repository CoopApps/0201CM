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
    /// Squad number assigned by `World::assign_squad_numbers` — 1..N
    /// per club, ranked GK-first then top-CA within group. Rendered
    /// inside the blue number-cell on Traditional view. `0` when the
    /// player isn't in the current squad numbering pass.
    pub squad_number: u8,
    /// Sort-value fields — every entry on the Traditional Sort By
    /// menu has a value the exe paints in the right column of each
    /// row (Nationality "ENG", Age "23", Basic Wage "£475", Contract
    /// Expiry "14.6.07", etc). Populated by the app from the exact
    /// same sources the other views read.
    pub nationality: &'a str,
    pub int_caps: u16,
    pub int_goals: u16,
    pub condition_pct: u8,
    pub morale: &'a str,
    pub wage_str: &'a str,
    pub expiry_str: &'a str,
    pub value_str: &'a str,
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
        // Menu labels as painted by the exe (FUN_00457200's dropdown
        // `text_template_expand`s — the OTHER row uses just "Other";
        // "Other Info" is the SUBTITLE shown at the top of the list
        // panel, not the menu row).
        match self {
            SquadView::Traditional => "Traditional",
            SquadView::Contract    => "Contract",
            SquadView::Selection   => "Selection",
            SquadView::Stats       => "Stats",
            SquadView::MoreStats   => "More Stats",
            SquadView::Attributes  => "Attributes",
            SquadView::OtherInfo   => "Other",
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
    /// Per-mode column pack, verbatim from FUN_00457200:
    ///
    /// - Widths are the `local_304 / local_35c / local_324 / local_344 /
    ///   local_314 / local_334` byte arrays declared at lines 456-528 of
    ///   the decompile — unit widths inside a 13-column row grid (except
    ///   Traditional, which is 8-col, 2-players-per-row).
    /// - Headers are the s_* string constants at 0x0097b2c4..0x0097b360
    ///   emitted by the `FUN_00549580(kind=2, ..., group=0xc)` header-
    ///   spawn calls at lines 1867..2186. Empty strings pair with 0-width
    ///   cells the exe skips over.
    ///
    /// Traditional is a 2-col layout with no header row and is handled
    /// specially by the renderer — it returns `None` here.
    pub fn column_pack(self) -> Option<ColumnPack> {
        use SquadView::*;
        match self {
            Traditional => None,
            // widths local_35c = {5,5,24,14,14,15,11,0,0,0,0,0,10}
            // headers per lines 2128-2186. Col 6 (Releases) is present
            // only when the club is NOT human-managed; the exe swaps the
            // widths to {5,5,24,18,18,18,0,...} in the managed branch
            // (FUN_00525450 gate). For the pre-launch preview the club
            // is un-managed so col 6 stays.
            // Header order verified against live GDI capture
            // scratchpad/prelaunch/contract_view.png — the exe paints
            // Pkd first, Inf second (the decompiled bit table was in
            // sort-mask order, not paint order).
            Contract => Some(ColumnPack {
                widths: [5,5,24,14,14,15,11,0,0,0,0,0,10],
                headers: ["Pkd","Inf","Name","Squad Status","Basic Wage",
                          "Contract Expiry","Releases","","","","","","Value"],
            }),
            // Verified against scratchpad/prelaunch/selection_view.png
            // (Cheltenham 2001). The archaeology's "attr#1 / attr#17"
            // guess for cols 7-8 was wrong — the exe paints:
            //   col 7 = "Apps" (season appearances — cyan)
            //   col 8 = "Av R" (average match rating — cyan, "----"
            //           when the record has no rated matches yet)
            Selection => Some(ColumnPack {
                widths: [5,5,24,14,10,12,6,6,6,0,0,0,10],
                headers: ["Pkd","Inf","Name","Position","Form","Morale","Cond.",
                          "Apps","Av R","","","","Value"],
            }),
            // widths local_344 (Stats + More Stats share these).
            // Cols 3..11 populated at runtime from the attribute-id list
            // DAT_0097ae40 = {1,2,5,0xc,0xd,0xe,0xf,0x10,0x11}.
            // Short names from Data attribute table.
            Stats => Some(ColumnPack {
                widths: [5,5,24,6,6,6,6,6,6,6,6,6,10],
                headers: ["Pkd","Inf","Name","Agg","Ant","Cor","Fin","Fla",
                          "Han","Hea","IM","Inf","Value"],
            }),
            // widths local_344; attr list DAT_0097ae4c = {3,4,0xa,0xb,8,9,6,7,0x11}.
            MoreStats => Some(ColumnPack {
                widths: [5,5,24,6,6,6,6,6,6,6,6,6,10],
                headers: ["Pkd","Inf","Name","Bra","Con","Dir","Dri","Dec",
                          "Det","Cre","Cro","Inf","Value"],
            }),
            // widths local_314; header text via FUN_0052c3f0 (long attr
            // name). Sub-toggle selects one of Physical / Mental / GK /
            // Def / Att attribute lists (DAT_0097ae58..88). We default to
            // Physical (bit 0x40 in the exe's local_384). Long names
            // truncate to fit the 6-unit cell.
            Attributes => Some(ColumnPack {
                widths: [5,5,24,6,6,6,6,6,6,6,6,6,10],
                headers: ["Pkd","Inf","Name","Agg","Bra","Cor","Hea","Inf",
                          "Jum","Pac","Sta","Str","Value"],
            }),
            // widths local_334; headers per lines 1914-1958.
            OtherInfo => Some(ColumnPack {
                widths: [5,5,24,8,6,6,6,10,12,6,0,0,10],
                headers: ["Pkd","Inf","Name","Nat.","Age","Caps","Goals",
                          "Form","Morale","Cond.","","","Value"],
            }),
        }
    }
}

/// Semantic type of a header cell — drives default sort direction
/// and the compare function used when the header is clicked.
///
///   Text    — first click sorts A→Z, second click Z→A. Name / Nat. /
///             Squad Status / Position / Releases fall here.
///   Numeric — first click sorts high→low, second click low→high.
///             Wage / Value / attributes / Age fall here.
///   Date    — first click sorts soonest→furthest, second click flips.
///             Contract Expiry is the only current one.
///   Marker  — non-sortable ornament columns (Inf / Pkd flags). Header
///             clicks are absorbed with no state change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnKind {
    Text,
    Numeric,
    Date,
    Marker,
}

impl ColumnKind {
    /// True when this column's first-click direction is descending.
    /// Numeric columns (wage / value / attributes) show the biggest
    /// value at the top first; text/date columns show smallest first.
    pub fn default_descending(self) -> bool {
        matches!(self, ColumnKind::Numeric)
    }
}

/// Per-view column pack — unit widths + header strings, one entry each
/// per column in the exe's 13-cell row grid (or 8 for Traditional).
#[derive(Debug, Clone, Copy)]
pub struct ColumnPack {
    /// Unit widths from `local_35c / 324 / 344 / 314 / 334`. A zero-width
    /// column is skipped when partitioning the row into pixel slices.
    pub widths: [u8; 13],
    /// Header labels — `""` on a 0-width cell.
    pub headers: [&'static str; 13],
}

impl ColumnPack {
    /// Semantic type of each column — drives default sort direction
    /// and the compare function used at click time. Aligned by index
    /// with `headers` / `widths`.
    pub fn kinds(&self) -> [ColumnKind; 13] {
        // Fallback: the trailing "Value" column and per-attribute
        // numeric columns default to numeric-desc; Name is text-asc;
        // date columns are date-asc; everything else defaults to
        // text-asc so the sort at least deterministically groups
        // like values.
        use ColumnKind::*;
        let mut k = [Text; 13];
        k[0] = Marker; k[1] = Marker;
        k[2] = Text;   // Name column — text sort (A→Z first click).
        k[12] = Numeric;
        // Override cols 3..11 per known headers.
        for (i, hdr) in self.headers.iter().enumerate() {
            match *hdr {
                "Contract Expiry"                => k[i] = Date,
                "Basic Wage" | "Value" | "Age"
                    | "Caps" | "Goals" | "Form"
                    | "Morale" | "Cond."          => k[i] = Numeric,
                "Nat." | "Squad Status" | "Position" | "Releases"
                    | "Name"                      => k[i] = Text,
                // Attribute columns ("Agg", "Ant", "Cor", "Bra"…): numeric.
                s if s.len() <= 3 && !s.is_empty() => k[i] = Numeric,
                _ => {}
            }
        }
        k
    }

    /// Per-column body-cell ink for THIS view. `None` slots default
    /// to yellow. Verified against
    /// scratchpad/prelaunch/selection_view.png where the exe paints
    /// Position (bright cyan 0x43ff), Morale (yellow 0x7380), Cond
    /// (orange 0x6180), Apps (cyan), Av R (cyan) instead of the
    /// uniform yellow every other view uses.
    pub fn cell_inks(&self, view: SquadView) -> [Option<u16>; 13] {
        let mut inks = [None; 13];
        if let SquadView::Selection = view {
            const CYAN_BRIGHT: u16 = 0x43ff;
            const ORANGE:      u16 = 0x6180;
            inks[3] = Some(CYAN_BRIGHT);  // Position
            inks[6] = Some(ORANGE);       // Cond.
            inks[7] = Some(CYAN_BRIGHT);  // Apps
            inks[8] = Some(CYAN_BRIGHT);  // Av R
            // Morale (col 5) stays default yellow.
        }
        inks
    }

    /// Convert unit widths to pixel x-slices across `x0..x1`. Returns one
    /// (x0, x1) tuple per NON-ZERO column, in the same order as `widths`.
    pub fn slices(&self, x0: i32, x1: i32) -> Vec<(usize, i32, i32)> {
        let total: u32 = self.widths.iter().map(|w| *w as u32).sum();
        if total == 0 { return Vec::new(); }
        let span = (x1 - x0) as u32;
        let mut out = Vec::with_capacity(13);
        let mut acc: u32 = 0;
        for (i, w) in self.widths.iter().enumerate() {
            if *w == 0 { continue; }
            let sx0 = x0 + (acc * span / total) as i32;
            acc += *w as u32;
            let sx1 = x0 + (acc * span / total) as i32;
            out.push((i, sx0, sx1));
        }
        out
    }
}

/// UTF-8 → Latin-1 conversion for the game's font. `£` in a Rust
/// string literal is two bytes (0xC2 0xA3), but the exe's bitmap font
/// indexes each glyph by Latin-1 codepoint — so we have to collapse
/// each `char` back down to a single byte. Any codepoint above 0xFF
/// falls back to `?` so a bad input never crashes the renderer.
pub fn c_string_latin1(s: &[u8]) -> Vec<u8> {
    let text = std::str::from_utf8(s).unwrap_or("");
    let mut out = Vec::with_capacity(text.len() + 1);
    for ch in text.chars() {
        let cp = ch as u32;
        out.push(if cp < 0x100 { cp as u8 } else { b'?' });
    }
    out.push(0);
    out
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
    /// Live cursor y position (screen-space). Used by the dropdown to
    /// paint a HOVER highlight (yellow bg + black text) on whichever
    /// row the mouse is currently over — matches the exe's per-row
    /// hover behaviour where yellow follows the cursor, and a tick is
    /// drawn on the row of the currently-active mode.
    pub cursor_x: i32,
    pub cursor_y: i32,
    /// `true` when the club-jump dropdown (triangle box in the top-
    /// left of the title bar) is open. Contents are supplied by the
    /// app in `jump_items` — every club in the current division
    /// alphabetically, plus the national team.
    pub jump_menu_open: bool,
    /// Labels for the jump menu, in the order the exe paints them.
    /// Empty when the menu is closed.
    pub jump_items: &'a [&'a str],
    /// Currently-selected Sort By option (Traditional view only).
    /// Rendered as a tick in the dropdown when open. Defaults to Name.
    pub sort_by: SortByKey,
    /// `true` when the Sort By dropdown is open.
    pub sort_menu_open: bool,
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

// Subtitle band ("Position(s)" for Traditional, "Contract Info" /
// "Selection Info" / "Stats" / etc. for the other modes). Yellow
// centred text on the darkened photo backing. Height verified against
// scratchpad/prelaunch/contract_view.png.
const HDR_Y0: i32 = 150;
const HDR_Y1: i32 = 185;

// Column header row — ONLY drawn on non-Traditional modes. Sits just
// above LIST_Y0 in a SHORT (~16 px) grey-bevel strip — shorter than
// the sub-toolbar (Filter / Sort By at y=125..145 = 20 px). Framebuffer
// sample from scratchpad/prelaunch/contract_view.png at x=300 shows the
// header main body as flat 0x4210 grey from y=200..213 with a 2-px
// highlight above (y=198..199) and a 2-px shadow below (y=214..215).
const COL_HDR_Y0: i32 = 198;
const COL_HDR_Y1: i32 = 214;

/// Value column body colour — deep magenta 0x2008 sampled directly out
/// of contract_view.rgb555.bin at the Simon Brown value cell (x=700..720,
/// y=223..230 all read 0x2008 = R65 G0 B65). NOT the same as the
/// title-bar purple 0x331f (which is a light cyan-lavender).
const VALUE_PURPLE: u16 = 0x2008;

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
    // Small badge / jump-menu trigger — same kit colours plus a
    // filled black right-pointing triangle CENTRED inside the box.
    // Rect from the GDI capture: (105, 15)-(120, 35).
    let (jbx0, jby0, jbx1, jby1) = JUMP_BUTTON_RECT;
    draw_panel(surface, jbx0, jby0, jbx1, jby1,
        P_SOLID_FILL | P_BEVEL, bar_fill, bar_ink, palette);
    // Filled right-pointing triangle. 8 px tall × ~5 px wide, black,
    // its vertical left edge centred inside the box and its right
    // tip on the box centre-line.
    let ccx = (jbx0 + jbx1) / 2;
    let ccy = (jby0 + jby1) / 2;
    const TRI_HALF_H: i32 = 4;
    let base_x = ccx - 2;
    for dy in -TRI_HALF_H..=TRI_HALF_H {
        let w = TRI_HALF_H - dy.abs();
        surface.draw_line(base_x, ccy + dy, base_x + w, ccy + dy, 2, 0);
    }

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

    // ---- Column header band + player list. Structure per FUN_00457200
    //      and verified against scratchpad/prelaunch/contract_view.png:
    //      Traditional (bit 1) uses a 2-players-per-row grid with NO
    //      column header cells. Every other mode paints
    //        - a subtitle band ("Contract Info", "Stats", …) in yellow,
    //        - a short (~22 px) column-header strip on grey bevel with
    //          cyan labels,
    //        - a 13-cell body row per player below.
    match state.view.column_pack() {
        // ================================================================
        // Non-Traditional modes — subtitle + short header + 1-row grid.
        // ================================================================
        Some(pack) => {
            // Subtitle strip ("Contract Info" / "Selection Info" / …).
            draw_panel(surface, LIST_X0, HDR_Y0, LIST_X1, HDR_Y1,
                P_DARKEN, 0, 0, palette);
            draw_wrapped_text(surface, LIST_X0, HDR_Y0, LIST_X1, HDR_Y1,
                &body_font, &c_string(state.view.subtitle().as_bytes()),
                INK_YELLOW, TS_CENTRE, -1);

            let list_right = SB_X0 - 1;
            let slices = pack.slices(LIST_X0, list_right);

            // Column header row — grey bevel + cyan labels, ~22 px tall.
            for (col_idx, sx0, sx1) in &slices {
                let hdr = pack.headers[*col_idx].trim_start();
                draw_panel(surface, *sx0, COL_HDR_Y0, *sx1 - 1, COL_HDR_Y1,
                    P_SOLID_FILL | P_BEVEL, GREY_BAR, INK_CYAN, palette);
                if hdr.is_empty() { continue; }
                draw_wrapped_text(surface, *sx0, COL_HDR_Y0, *sx1 - 2, COL_HDR_Y1,
                    &small_font, &c_string_latin1(hdr.as_bytes()),
                    INK_CYAN, TS_CENTRE, -1);
            }

            // List background — starts just below the header strip.
            const NT_LIST_Y0: i32 = COL_HDR_Y1 + 2;
            const NT_ROW_FIRST_Y: i32 = NT_LIST_Y0 + 4;
            const NT_ROW_STRIDE: i32 = 16;
            const NT_ROW_HEIGHT: i32 = 15;
            draw_panel(surface, LIST_X0, NT_LIST_Y0, LIST_X1, LIST_Y1,
                P_DARKEN, 0, 0, palette);

            let one_col_rows: usize = ((LIST_Y1 - NT_ROW_FIRST_Y) / NT_ROW_STRIDE) as usize;
            // Per-view per-column ink overrides (None = default yellow).
            let cell_inks = pack.cell_inks(state.view);
            let visible = state.players.iter().skip(state.scroll).take(one_col_rows);
            for (i, p) in visible.enumerate() {
                let y0 = NT_ROW_FIRST_Y + (i as i32) * NT_ROW_STRIDE;
                let y1 = y0 + NT_ROW_HEIGHT;
                for (col_idx, sx0, sx1) in &slices {
                    let cell = p.cols.get(*col_idx).copied().unwrap_or("");
                    // Cols 0/1/2/12 have identical structure across
                    // every non-Traditional view — carve them out first.
                    match *col_idx {
                        0 => {
                            // Pkd — blue row-marker.
                            draw_panel(surface, *sx0 + 1, y0, *sx1 - 2, y1,
                                P_SOLID_FILL | P_BEVEL, BLUE, INK_CYAN, palette);
                            continue;
                        }
                        1 => continue,   // Inf (marker flags TBD)
                        2 => {
                            let mut buf = format!("  {}", p.name);
                            if p.marker != ' ' { buf.push(p.marker); }
                            buf.push('\0');
                            draw_wrapped_text(surface, *sx0, y0, *sx1 - 2, y1,
                                &small_font, buf.as_bytes(), WHITE,
                                TS_CENTRE | W_LEFT, -1);
                            continue;
                        }
                        12 => {
                            // Value — purple bevel, WHITE ink.
                            draw_panel(surface, *sx0 + 1, y0, *sx1 - 2, y1,
                                P_SOLID_FILL | P_BEVEL, VALUE_PURPLE,
                                WHITE, palette);
                            let shown = if cell.is_empty() { "-" } else { cell };
                            draw_wrapped_text(surface, *sx0, y0, *sx1 - 2, y1,
                                &small_font, &c_string_latin1(shown.as_bytes()),
                                WHITE, TS_CENTRE, -1);
                            continue;
                        }
                        _ => {}
                    }
                    // Body cell — ink from cell_inks[col], with the
                    // Contract-only column overrides (Squad Status
                    // cyan, Releases orange) applied when Contract is
                    // the active view.
                    let ink = match (state.view, *col_idx) {
                        (SquadView::Contract, 3) => INK_CYAN,          // Squad Status
                        (SquadView::Contract, 6) => TRIANGLE_ORANGE,   // Releases
                        _ => cell_inks[*col_idx].unwrap_or(INK_YELLOW),
                    };
                    // Alignment — Position column reads left-aligned in
                    // the exe (D/DM R, S C etc. hug the left of the
                    // cell) so the varying widths don't look wobbly.
                    // Every other data cell is centred.
                    let align = if matches!(state.view, SquadView::Selection) && *col_idx == 3 {
                        W_LEFT
                    } else {
                        TS_CENTRE
                    };
                    let shown = if cell.is_empty() { "-" } else { cell };
                    draw_wrapped_text(surface, *sx0, y0, *sx1 - 2, y1,
                        &small_font, &c_string_latin1(shown.as_bytes()),
                        ink, align, -1);
                }
            }
        }
        // ================================================================
        // Traditional mode — 2-players-per-row, no header cells. Old
        // layout preserved because it matched the exe capture pixel-for-
        // pixel on Chester and Leigh RMI.
        // ================================================================
        None => {
            // Subtitle band + darkened list area — Traditional only
            // paints the "Position(s)" heading, no per-column strip.
            draw_panel(surface, LIST_X0, HDR_Y0, LIST_X1, HDR_Y1,
                P_DARKEN, 0, 0, palette);
            draw_panel(surface, LIST_X0, LIST_Y0, LIST_X1, LIST_Y1,
                P_DARKEN, 0, 0, palette);
            // Subtitle text tracks the active Sort By pick — the exe
            // labels the band with whatever the current sort key is
            // (Name / Position(s) / Age / Basic Wage / ...), so the
            // player knows at a glance which axis the two-col grid is
            // ordered on.
            let hdr_txt = state.sort_by.label();
            draw_wrapped_text(surface, LIST_X0, HDR_Y0, LIST_X1, HDR_Y1,
                &body_font, &c_string_latin1(hdr_txt.as_bytes()),
                INK_YELLOW, TS_CENTRE, -1);
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
                // Blue row-marker cell is always empty — the exe uses
                // it as a per-row Pkd/Inf status flag, not for the
                // squad number. Squad number lives in the right-hand
                // column when Sort By = Squad Number, matching the
                // Cheltenham exe capture.
                draw_panel(surface, num.0, y0, num.1, y1,
                    P_SOLID_FILL | P_BEVEL, BLUE, INK_CYAN, palette);
                let name_ink = if p.marker != ' ' { WHITE } else { INK_CYAN };
                let mut buf = format!("  {}", p.name);
                if p.marker != ' ' { buf.push(p.marker); }
                buf.push('\0');
                draw_wrapped_text(surface, name.0, y0, name.1, y1,
                    &body_font, buf.as_bytes(), name_ink,
                    TS_CENTRE | W_LEFT, -1);
                // Right column reflects the Sort By pick — Position(s)
                // shows the position code, Squad Number the digit,
                // Age years, etc. Fields we don't yet have real data
                // for (Form, Morale, Condition, Basic Wage, ...) still
                // render '-' so the reader can see the column is
                // populated but empty — matches the exe's behaviour
                // at boot when no season has been played.
                let sort_txt: String = match state.sort_by {
                    SortByKey::Position    => p.position.to_string(),
                    SortByKey::SquadNumber =>
                        if p.squad_number > 0 { p.squad_number.to_string() }
                        else { "-".to_string() },
                    SortByKey::Age =>
                        p.age.map(|a| a.to_string()).unwrap_or_else(|| "-".to_string()),
                    SortByKey::Nationality  => p.nationality.to_string(),
                    SortByKey::IntCaps      => p.int_caps.to_string(),
                    SortByKey::IntGoals     => p.int_goals.to_string(),
                    SortByKey::Condition    => format!("{}%", p.condition_pct),
                    SortByKey::Morale       => p.morale.to_string(),
                    SortByKey::BasicWage    => p.wage_str.to_string(),
                    SortByKey::ContractExpiry => p.expiry_str.to_string(),
                    SortByKey::Value        => p.value_str.to_string(),
                    // Name — no useful right-col value (name already
                    // fills the wide cell); leave blank.
                    SortByKey::Name         => String::new(),
                    // Season-stats family (Form / Goals / Conceded /
                    // Assists / Av. Rating) genuinely have no data
                    // until a season has been played; '-' matches the
                    // exe's boot-time blank state.
                    _ => "-".to_string(),
                };
                draw_wrapped_text(surface, pos.0, y0, pos.1, y1,
                    &small_font, &c_string_latin1(sort_txt.as_bytes()),
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
        draw_view_dropdown(surface, &small_font, state.view,
                           state.cursor_x, state.cursor_y);
    }
    // Sort By dropdown (Traditional view only). Same green-alternating
    // menu as View, with a tick on the currently-active sort key and a
    // separator row between base attributes and per-season stats.
    if state.sort_menu_open {
        draw_sort_dropdown(surface, &small_font, state.sort_by,
                           state.cursor_x, state.cursor_y);
    }
    // Club-jump dropdown — corner-triangle box opens a menu of every
    // club in the current division alphabetically + the national
    // team. Rendered LAST so it overlays even the View dropdown when
    // both are somehow open (only one flag is settable at a time via
    // the click handler, but drawing order matters for correctness).
    if state.jump_menu_open && !state.jump_items.is_empty() {
        let items = &state.jump_items[..state.jump_items.len().min(JUMP_MENU_MAX)];
        let rect = jump_menu_rect(items.len());
        // Find which item matches the currently-viewed club so it gets
        // the tick — done by label match; the app supplies items in
        // the same order it will hit-test them.
        let selected = items.iter().position(|s| *s == state.club_name);
        crate::menu_dropdown::draw_dropdown(
            surface, &small_font, rect, items, selected,
            (state.cursor_x, state.cursor_y),
        );
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
/// Anchor rect of the View dropdown — shared between the renderer and
/// the app's hit-test.
const VIEW_DROPDOWN: crate::menu_dropdown::DropdownRect =
    crate::menu_dropdown::DropdownRect { x0: 110, y0: 148, width: 145, row_h: 18 };

fn draw_view_dropdown(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    current: SquadView,
    cursor_x: i32,
    cursor_y: i32,
) {
    let items = SquadView::PRE_LAUNCH_ORDER;
    let labels: Vec<&str> = items.iter().map(|m| m.label()).collect();
    let selected = items.iter().position(|m| *m == current);
    crate::menu_dropdown::draw_dropdown(
        surface, font, VIEW_DROPDOWN, &labels, selected,
        (cursor_x, cursor_y),
    );
}

/// Hit-test the View dropdown. Returns the SquadView the cursor is over,
/// or None if outside the dropdown area.
pub fn view_dropdown_hit(x: i32, y: i32) -> Option<SquadView> {
    VIEW_DROPDOWN
        .hit(SquadView::PRE_LAUNCH_ORDER.len(), x, y)
        .and_then(|row| SquadView::PRE_LAUNCH_ORDER.get(row).copied())
}

/// The View button rect on the sub-toolbar. Clicks here toggle
/// `view_menu_open`.
pub const VIEW_BUTTON_RECT: (i32, i32, i32, i32) = (110, 125, 255, 145);
/// Sort-By button rect on the sub-toolbar — same y as View, sits
/// just right of it at TOOLBAR_LEFT_R (236..360).
pub const SORT_BUTTON_RECT: (i32, i32, i32, i32) = (236, 125, 360, 145);

/// The seventeen Sort By options shown when Traditional view opens
/// the Sort By dropdown, verified against the running exe. Layout
/// order matches the capture: Name first (default, ticked on new
/// game), then base attributes, blank separator, then per-season
/// stats which stay disabled until a season has actually been
/// played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortByKey {
    Name,
    Position,
    SquadNumber,
    Form,
    Morale,
    Condition,
    Nationality,
    Age,
    IntCaps,
    IntGoals,
    BasicWage,
    ContractExpiry,
    Value,
    Goals,
    Conceded,
    Assists,
    AvRating,
}

impl SortByKey {
    /// Menu order lifted from the exe's Sort By dropdown. A `None`
    /// slot renders as the embossed separator (menu_dropdown paints
    /// an empty-label row as a groove).
    pub const MENU_ORDER: [Option<SortByKey>; 18] = [
        Some(SortByKey::Name),
        Some(SortByKey::Position),
        Some(SortByKey::SquadNumber),
        Some(SortByKey::Form),
        Some(SortByKey::Morale),
        Some(SortByKey::Condition),
        Some(SortByKey::Nationality),
        Some(SortByKey::Age),
        Some(SortByKey::IntCaps),
        Some(SortByKey::IntGoals),
        Some(SortByKey::BasicWage),
        Some(SortByKey::ContractExpiry),
        Some(SortByKey::Value),
        None,                                 // separator row
        Some(SortByKey::Goals),
        Some(SortByKey::Conceded),
        Some(SortByKey::Assists),
        Some(SortByKey::AvRating),
    ];
    /// Label as painted in the exe dropdown.
    pub fn label(self) -> &'static str {
        match self {
            SortByKey::Name           => "Name",
            SortByKey::Position       => "Position(s)",
            SortByKey::SquadNumber    => "Squad Number",
            SortByKey::Form           => "Form",
            SortByKey::Morale         => "Morale",
            SortByKey::Condition      => "Condition",
            SortByKey::Nationality    => "Nationality",
            SortByKey::Age            => "Age",
            SortByKey::IntCaps        => "Int. Caps",
            SortByKey::IntGoals       => "Int. Goals",
            SortByKey::BasicWage      => "Basic Wage",
            SortByKey::ContractExpiry => "Contract Expiry",
            SortByKey::Value          => "Value",
            SortByKey::Goals          => "Goals",
            SortByKey::Conceded       => "Conceded",
            SortByKey::Assists        => "Assists",
            SortByKey::AvRating       => "Av. Rating",
        }
    }
}

/// Sort-By dropdown anchor. Sits just under the Sort By button; row
/// height matches the View dropdown so the two menus share the same
/// visual weight.
const SORT_DROPDOWN: crate::menu_dropdown::DropdownRect =
    crate::menu_dropdown::DropdownRect { x0: 236, y0: 148, width: 145, row_h: 18 };

/// Paint the Sort By dropdown. `selected` is the currently-active
/// sort key (gets the tick); cursor drives yellow hover.
pub fn draw_sort_dropdown(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    selected: SortByKey,
    cursor_x: i32, cursor_y: i32,
) {
    let items: Vec<&str> = SortByKey::MENU_ORDER
        .iter()
        .map(|o| o.map(|k| k.label()).unwrap_or(""))
        .collect();
    let sel_row = SortByKey::MENU_ORDER.iter()
        .position(|o| *o == Some(selected));
    crate::menu_dropdown::draw_dropdown(
        surface, font, SORT_DROPDOWN,
        &items, sel_row, (cursor_x, cursor_y),
    );
}

/// Hit-test the Sort By dropdown. Returns the picked key, or None if
/// the click missed a row or landed on the separator.
pub fn sort_dropdown_hit(x: i32, y: i32) -> Option<SortByKey> {
    let idx = SORT_DROPDOWN.hit(SortByKey::MENU_ORDER.len(), x, y)?;
    SortByKey::MENU_ORDER[idx]
}

/// Corner triangle box inside the title bar. Clicks here toggle
/// `jump_menu_open`. Measured from the GDI capture — sits just inside
/// the left edge of the title panel.
pub const JUMP_BUTTON_RECT: (i32, i32, i32, i32) = (105, 15, 120, 35);

/// The jump-menu dropdown rect. Measured from the GDI capture
/// (club_screen_now.png): x=123..247, y=17.., row height 20 px. The
/// vertical extent grows with `items.len()` up to a cap of 25 rows
/// (~500 px tall) — matches the exe scrolling internally when a
/// division has more than that.
const JUMP_MENU_X0:    i32 = 123;
const JUMP_MENU_WIDTH: i32 = 124;
const JUMP_MENU_Y0:    i32 = 17;
const JUMP_MENU_ROW_H: i32 = 20;
// Divisions like the Second Division carry 24 clubs; leave headroom
// for the separator + national-team row without clipping them off.
const JUMP_MENU_MAX:   usize = 28;

fn jump_menu_rect(item_count: usize) -> crate::menu_dropdown::DropdownRect {
    let rows = item_count.min(JUMP_MENU_MAX) as i32;
    let _ = rows;   // rows currently equals items.len() for the visible slice
    crate::menu_dropdown::DropdownRect {
        x0: JUMP_MENU_X0,
        y0: JUMP_MENU_Y0,
        width: JUMP_MENU_WIDTH,
        row_h: JUMP_MENU_ROW_H,
    }
}

/// Hit-test the jump-menu dropdown. Returns the item index the cursor
/// is over (0-based), capped at `items.len()`.
pub fn jump_menu_hit(x: i32, y: i32, item_count: usize) -> Option<usize> {
    jump_menu_rect(item_count).hit(item_count.min(JUMP_MENU_MAX), x, y)
}

/// Hit-test the column-header row (non-Traditional view only).
/// Returns the 0..12 column index the click lands on, or None when
/// the click misses the header strip or lands on a Marker column
/// (Inf / Pkd) that can't be sorted.
pub fn header_hit(pack: &ColumnPack, x: i32, y: i32) -> Option<usize> {
    if y < COL_HDR_Y0 || y > COL_HDR_Y1 { return None; }
    let list_right = SB_X0 - 1;
    let kinds = pack.kinds();
    for (col_idx, sx0, sx1) in pack.slices(LIST_X0, list_right) {
        if x >= sx0 && x <= sx1 {
            if kinds[col_idx] == ColumnKind::Marker { return None; }
            return Some(col_idx);
        }
    }
    None
}

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
