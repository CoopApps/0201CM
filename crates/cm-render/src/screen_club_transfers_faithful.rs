//! Club Transfers screen — top tab #1 on the club preview.
//!
//! Strings decoded from cm0102.exe strings.json lines 611-620 (see
//! archaeology report for the Transfers screen). The seven View
//! dropdown items are the exact verbatim labels the exe paints.
//!
//! At pre-launch (fresh save, no season has been played), every list
//! is empty — nobody has been signed yet. The heading still paints as
//! `"<view> - Season 2001/02"` per the exe's title format string
//! `"<%s - Title(e.g.Players In)> - Season <%s - season e.g. 1998/9>"`.

/// Which sub-view of the Transfers screen is currently active. Order
/// matches the exe's dropdown top-to-bottom (0x004560cb → 0x0045630e
/// in the undecompiled LAB_004551c0 block; see strings.json 612-618).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransfersView {
    PlayersIn,
    PlayersOut,
    FutureTransfers,
    LoansIn,
    LoansOut,
    StaffIn,
    StaffOut,
}

impl TransfersView {
    pub const MENU_ORDER: [TransfersView; 7] = [
        TransfersView::PlayersIn,
        TransfersView::PlayersOut,
        TransfersView::FutureTransfers,
        TransfersView::LoansIn,
        TransfersView::LoansOut,
        TransfersView::StaffIn,
        TransfersView::StaffOut,
    ];

    /// Exe strings (leading space stripped — the shared dropdown
    /// widget already indents the tick column).
    pub fn label(self) -> &'static str {
        match self {
            TransfersView::PlayersIn       => "Players In",
            TransfersView::PlayersOut      => "Players Out",
            // "Future Transfers" is BIDIRECTIONAL — it lists every
            // in-progress negotiation the club is party to (bids
            // made AND bids received) that has not yet been
            // finalised, at any phase (Offer / Negotiation /
            // Contract / Work Permit / Confirmation / Delayed). It
            // ALSO covers future LOAN negotiations — the exe has no
            // separate "Future Loans" menu item. Loans In / Loans
            // Out below list only FINALISED, active loans.
            TransfersView::FutureTransfers => "Future Transfers",
            TransfersView::LoansIn         => "Loans In",
            TransfersView::LoansOut        => "Loans Out",
            TransfersView::StaffIn         => "Staff In",
            TransfersView::StaffOut        => "Staff Out",
        }
    }

    /// The panel-title prefix used in the subtitle band. Format is
    /// exactly the exe's `"<%s> - Season <%s>"` — a caller passes it
    /// through `format!("{} - Season {}", view.subtitle_prefix(),
    /// season_str)`.
    pub fn subtitle_prefix(self) -> &'static str {
        self.label()
    }
}

/// The Transfers View dropdown pops under the View button on the
/// RIGHT of the sub-toolbar (verified against
/// scratchpad/prelaunch/transfers_players_in.png — View lives where
/// Squad's Filter is at x=656..780, NOT where Squad's View is at
/// x=110..234). Right-aligned to keep the 145-px menu on-screen at
/// the 800-pixel surface edge: x0 = 780 - 145 = 635, ending at the
/// button's right edge like the Squad Filter menu.
pub const TRANSFERS_VIEW_DROPDOWN: crate::menu_dropdown::DropdownRect =
    crate::menu_dropdown::DropdownRect {
        x0: 635, y0: 148, width: 145, row_h: 18,
    };

pub fn draw_transfers_view_dropdown(
    surface: &mut crate::packed::PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    selected: TransfersView,
    cursor_x: i32, cursor_y: i32,
) {
    let items: Vec<&str> = TransfersView::MENU_ORDER.iter()
        .map(|v| v.label()).collect();
    let sel = TransfersView::MENU_ORDER.iter().position(|v| *v == selected);
    crate::menu_dropdown::draw_dropdown(
        surface, font, TRANSFERS_VIEW_DROPDOWN,
        &items, sel, (cursor_x, cursor_y),
    );
}

pub fn transfers_view_dropdown_hit(x: i32, y: i32) -> Option<TransfersView> {
    let idx = TRANSFERS_VIEW_DROPDOWN.hit(TransfersView::MENU_ORDER.len(), x, y)?;
    Some(TransfersView::MENU_ORDER[idx])
}

/// Paint the body of the Players In / Players Out list.
///
/// Column x-ranges + row grid are measured from
/// `scratchpad/prelaunch/transfers_players_in.png` (Pro Vercelli,
/// 3 rows, no scrollbar) and
/// `scratchpad/prelaunch/transfers_stalybridge_2006.png` (Stalybridge
/// Celtic, 14 rows, scrollbar shown).
///
/// Layout (Players In):
///   x=140..190  Date cell — blue bevel, white text (DD.M.YY)
///   x=200..470  Name       — white text, no cell
///   x=475..640  Other club — YELLOW text, no cell ("Free Transfer" or club)
///   x=700..770  Fee bevel  — purple bevel, yellow text ("£150K" / "Free")
///                (shifts to 680..755 when >14 rows and scrollbar shows)
///
/// Row stride 21 px starting at y=207; row height 19 px. The exe caps
/// the visible window at 14 rows before the scrollbar appears.
pub fn render_players_in_rows(
    surface: &mut crate::packed::PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    rows: &[TransferRow],
) {
    use crate::packed_text::{draw_wrapped_text, W_LEFT};
    use crate::packed_panel::{draw_panel, P_SOLID_FILL, P_BEVEL, PanelPalette};
    use crate::screen_club_squad_faithful::c_string_latin1;
    use crate::screen_pre_boot_chrome::TS_CENTRE;

    // Column x-ranges measured pixel-exact from GDI captures
    // (transfers_players_in.png y=230, transfers_stalybridge_2006.png
    // y=200) using PIL colour probing:
    //   Blue date box:  no-scrollbar 112..211 (99 wide)
    //                   scrollbar    112..207 (95 wide)
    //   Purple fee box: no-scrollbar 678..777 (99 wide)
    //                   scrollbar    660..755 (95 wide)
    // Row baselines: first row y0 ≈ 207 (scrollbar) / 208 (no).
    const ROW_FIRST_Y: i32 = 207;
    const ROW_STRIDE:  i32 = 21;
    const ROW_HEIGHT:  i32 = 19;
    let scrollbar = rows.len() > 14;
    let (date_x0, date_x1) = if scrollbar { (112, 207) } else { (112, 211) };
    let (fee_x0,  fee_x1)  = if scrollbar { (660, 755) } else { (678, 777) };
    const NAME_X0:     i32 = 220;
    const OTHER_X0:    i32 = 475;
    const OTHER_X1:    i32 = 660;

    const BLUE:         u16 = 0x0010;
    const VALUE_PURPLE: u16 = 0x2008;
    const INK_WHITE:    u16 = 0x7fff;
    const INK_YELLOW:   u16 = 0x7380;   // same yellow the exe uses on this screen

    let palette = PanelPalette::default();

    let visible = rows.iter().take(14);
    for (i, r) in visible.enumerate() {
        let y0 = ROW_FIRST_Y + (i as i32) * ROW_STRIDE;
        let y1 = y0 + ROW_HEIGHT;

        // Date cell — royal-blue bevel, white text.
        draw_panel(surface, date_x0, y0, date_x1, y1,
            P_SOLID_FILL | P_BEVEL, BLUE, 0, palette);
        draw_wrapped_text(surface, date_x0, y0, date_x1, y1,
            font, &c_string_latin1(r.date.as_bytes()),
            INK_WHITE, TS_CENTRE, -1);

        // Name — white text, left-aligned.
        draw_wrapped_text(surface, NAME_X0, y0, 470, y1,
            font, &c_string_latin1(r.name.as_bytes()),
            INK_WHITE, W_LEFT, -1);

        // Other club — yellow text, left-aligned.
        draw_wrapped_text(surface, OTHER_X0, y0, OTHER_X1, y1,
            font, &c_string_latin1(r.other_club.as_bytes()),
            INK_YELLOW, W_LEFT, -1);

        // Fee bevel — purple bevel, yellow text.
        draw_panel(surface, fee_x0, y0, fee_x1, y1,
            P_SOLID_FILL | P_BEVEL, VALUE_PURPLE, 0, palette);
        draw_wrapped_text(surface, fee_x0, y0, fee_x1, y1,
            font, &c_string_latin1(r.fee.as_bytes()),
            INK_YELLOW, TS_CENTRE, -1);
    }
}

/// One row in the Players In / Players Out list.
///
/// At pre-launch every list is empty — the rows type is here so the
/// caller can feed real data once the transfer-history record layout
/// is decoded (the row loop is in the undecompiled LAB_004551c0
/// block; column set + byte offsets on the transfer-history record
/// still need Ghidra disassembly of 0x00455e00..0x00456400).
#[derive(Debug, Clone)]
pub struct TransferRow {
    /// Player name — "Firstname Surname" per the exe's transfer-panel
    /// convention (same as Contract view).
    pub name: String,
    /// Position code — same "D RC" / "GK" format used everywhere.
    pub position: String,
    /// Other club — "from <Club>" for Players In, "to <Club>" for
    /// Players Out. Exe formats via strings.json 604-606: "to {}",
    /// "from {}", "from Free Transfer".
    pub other_club: String,
    /// Fee — full-digit currency string ("£150,000", "Free"). Zero
    /// prints as "Free" per the exe's "Free Transfer" convention.
    pub fee: String,
    /// Transfer completion date — "DD.M.YY" format matching the
    /// Contract Expiry cell.
    pub date: String,
}
