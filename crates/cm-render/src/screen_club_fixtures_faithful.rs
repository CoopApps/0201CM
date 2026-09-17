//! Club Fixtures screen — top tab #3 on the club preview.
//!
//! Screen builder is `FUN_00460820` in the exe
//! (`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/00460820.c`,
//! ~530 lines). Called from `0x00456f38` inside the club-preview tab
//! dispatch (LAB_004551c0). Everything documented in
//! `memory/fixtures-screen-decoded.md`.
//!
//! What this file OWNS (screen chrome + row renderer):
//!   - Subtitle text formatter (`"Season 2001/2"` — no sub-view prefix)
//!   - Column x-ranges verified from `scratchpad/prelaunch/fixtures_gdi.png`
//!     (Cambridge United 2001/2): Date bevel 112..207, opponent name at
//!     ~220+, nation flag ~405..425, H/A ~440..455, competition name at
//!     ~465..600, result bevel 699..755
//!   - Six-column row renderer per FUN_00460820:279-284 spec
//!
//! What this file DOES NOT do:
//!   - Generate fixtures. The row list must be supplied by callers; the
//!     port intentionally shows an empty body when the fixture generator
//!     hasn't landed yet, rather than invent rows.

use crate::packed::PackedSurface;
use crate::packed_glyph::PixelFont;

/// One row of the fixtures list — matches the 80-byte record at
/// `club + 0xb1 + i * 0x50` (see FUN_005b1840 field decode in the
/// memory doc). Fields present here are the ones the row painter
/// consumes; the record has extras (played-flag reads, competition
/// pointer chase for the comp name).
#[derive(Debug, Clone)]
pub struct FixtureRow {
    /// Formatted date — the exe formats it as e.g. "Wed 25th Jul";
    /// caller formats from the packed day-in-year + year at +0x28/+0x2A.
    pub date: String,
    /// Opponent short name — from opponent club_id via ClubView.
    /// `"Tba"` (0x0097c148) when the fixture is scheduled but the
    /// opponent hasn't been drawn yet (cup rounds).
    pub opponent: String,
    /// Opponent's nation 3-letter code — painted ONLY when the opponent's
    /// nation differs from the viewed club's nation (FUN_00460820:360).
    /// Empty string for domestic games.
    pub nation_flag: String,
    /// `"H"` (0x0097c178) or `"A"` (0x0097c160). Empty for tours.
    pub home_away: &'static str,
    /// Competition name — `"Friendly"` (0x0097c154), `"  Tour Of ..."`
    /// (0x0097c12c), or the live comp name from `local_12c[0x13]+4`.
    pub competition: String,
    /// Result text — empty (0x0097c150 placeholder) when unplayed, or
    /// `"3-1"` / `"1-1 aet"` etc. once played.
    pub result: String,
}

/// Format the subtitle band for the Fixtures screen.
///
/// Exe format string reference: `"Season <%s - e.g. 1998-9>"`
/// (strings.json:611, painted at line 251 of 00460820.c). We match the
/// short trailing-year format used across the club preview family:
/// `2001` -> `"Season 2001/2"`.
pub fn format_subtitle(year: u16) -> String {
    // Short trailing year — 2001 -> "2001/2", 2005 -> "2005/6".
    // Verified from scratchpad/prelaunch/fixtures_gdi.png ("Season 2001/2")
    // and scratchpad/prelaunch/transfers_stalybridge_2006.png ("Season 2005/6").
    let short = (year + 1) % 10;
    format!("Season {}/{}", year, short)
}

/// Paint the six-column body rows.
///
/// Column x-ranges (measured from `scratchpad/prelaunch/fixtures_gdi.png`
/// via PIL colour probing, Cambridge United 2001/2):
///   x=112..207  Date cell  — blue bevel, white text
///   x=220..390  Opponent   — white text, left-aligned
///   x=405..425  Nation     — yellow text, centred (blank if nation matches)
///   x=440..455  H/A marker — yellow text, single char
///   x=465..640  Comp name  — yellow text, left-aligned
///   x=699..755  Result     — purple bevel, yellow text
///
/// Scrollbar rule reused from [[transfers-row-layout]]: show only when
/// `rows.len() > 14`. When shown, the date+result bevels shrink slightly
/// (fee/date columns are ~4 px narrower with scrollbar per that memory).
///
/// Not yet implemented — the row-body geometry is measured but the port
/// intentionally paints nothing until the fixture generator lands, so
/// no invented rows appear in-game. `rows.is_empty()` is the expected
/// case at boot.
pub fn render_fixture_rows(
    surface: &mut PackedSurface,
    font: &PixelFont,
    rows: &[FixtureRow],
) { render_fixture_rows_scrolled(surface, font, rows, 0); }

/// Same as [`render_fixture_rows`] but paints starting at `scroll`
/// entries down the row list. Scrollbar sizing still uses the FULL
/// `rows.len()` so the thumb reflects total-vs-visible correctly.
pub fn render_fixture_rows_scrolled(
    surface: &mut PackedSurface,
    font: &PixelFont,
    rows: &[FixtureRow],
    scroll: usize,
) {
    use crate::packed_text::{draw_wrapped_text, W_LEFT};
    use crate::packed_panel::{draw_panel, PanelPalette,
                              P_SOLID_FILL, P_BEVEL};
    use crate::screen_club_squad_faithful::c_string_latin1;
    use crate::screen_pre_boot_chrome::TS_CENTRE;

    // Geometry from the exe capture fixtures/club_fixtures_screen
    // (structure.txt): body panel (110,190)-(780,500); first date box
    // (112,198)-(208,218); rows on a 21px pitch (text y = 201, 222,
    // 243 … 475). The box is 20px tall (198..218).
    //
    // Was ROW_FIRST_Y=207 / ROW_HEIGHT=19, which pushed all 14 rows 9px
    // too low so the last row sat flush against the panel bottom (500)
    // with no gap. The exe leaves the last box at 471..491, a 9px gap
    // above the panel edge.
    const ROW_FIRST_Y: i32 = 198;
    const ROW_STRIDE:  i32 = 21;
    const ROW_HEIGHT:  i32 = 20;
    // Date box 112..208 and result box 699..756, unchanged by the
    // scrollbar (the exe keeps these x-ranges whether or not the bar is
    // shown — this capture HAS the bar and the date box still ends at
    // 208).
    let (date_x0, date_x1) = (112, 208);
    let (res_x0,  res_x1)  = (699, 756);
    const NAME_X0:     i32 = 220;
    const NAME_X1:     i32 = 405;
    const NATION_X0:   i32 = 410;
    const NATION_X1:   i32 = 435;
    const HA_X0:       i32 = 440;
    const HA_X1:       i32 = 460;
    const COMP_X0:     i32 = 465;
    const COMP_X1:     i32 = 640;

    const BLUE:         u16 = 0x0010;
    const VALUE_PURPLE: u16 = 0x2008;
    const INK_WHITE:    u16 = 0x7fff;
    const INK_YELLOW:   u16 = 0x7380;
    // H/A single-char marker paints in bright cyan on this screen,
    // not yellow — verified from scratchpad/prelaunch/fixtures_gdi.png
    // where "H" and "A" stand out from the yellow competition text.
    const INK_CYAN_BRIGHT: u16 = 0x43ff;

    let palette = PanelPalette::default();

    for (i, r) in rows.iter().skip(scroll).take(14).enumerate() {
        let y0 = ROW_FIRST_Y + (i as i32) * ROW_STRIDE;
        let y1 = y0 + ROW_HEIGHT;

        // Date — blue bevel, white centred.
        draw_panel(surface, date_x0, y0, date_x1, y1,
            P_SOLID_FILL | P_BEVEL, BLUE, 0, palette);
        draw_wrapped_text(surface, date_x0, y0, date_x1, y1,
            font, &c_string_latin1(r.date.as_bytes()),
            INK_WHITE, TS_CENTRE, -1);

        // Opponent — white, left-aligned.
        draw_wrapped_text(surface, NAME_X0, y0, NAME_X1, y1,
            font, &c_string_latin1(r.opponent.as_bytes()),
            INK_WHITE, W_LEFT, -1);

        // Nation flag (yellow, centred) — only when it's non-empty.
        // FUN_00460820:360 paints it only when opponent.nation !=
        // own_club.nation; caller decides.
        if !r.nation_flag.is_empty() {
            draw_wrapped_text(surface, NATION_X0, y0, NATION_X1, y1,
                font, &c_string_latin1(r.nation_flag.as_bytes()),
                INK_YELLOW, TS_CENTRE, -1);
        }

        // H/A marker — cyan (see INK_CYAN_BRIGHT above).
        if !r.home_away.is_empty() {
            draw_wrapped_text(surface, HA_X0, y0, HA_X1, y1,
                font, &c_string_latin1(r.home_away.as_bytes()),
                INK_CYAN_BRIGHT, TS_CENTRE, -1);
        }

        // Competition name.
        draw_wrapped_text(surface, COMP_X0, y0, COMP_X1, y1,
            font, &c_string_latin1(r.competition.as_bytes()),
            INK_YELLOW, W_LEFT, -1);

        // Result — purple bevel, yellow centred. Text is empty when
        // unplayed (per FUN_00460820:342-408 empty placeholder
        // 0x0097c150).
        draw_panel(surface, res_x0, y0, res_x1, y1,
            P_SOLID_FILL | P_BEVEL, VALUE_PURPLE, 0, palette);
        draw_wrapped_text(surface, res_x0, y0, res_x1, y1,
            font, &c_string_latin1(r.result.as_bytes()),
            INK_YELLOW, TS_CENTRE, -1);
    }
}

/// Format a GameDate as the exe's row-cell date — e.g. "Wed 25th Jul".
/// The exe reads the packed short at record +0x28 (day-in-year) and
/// +0x2A (year). Weekday abbreviations 3 chars; day ordinal suffix
/// per English convention (1st/2nd/3rd/…/21st/22nd/…). Month is the
/// standard 3-letter abbreviation.
pub fn format_row_date(day: u16, month: u16, year: u16) -> String {
    let month_abbr = ["Jan","Feb","Mar","Apr","May","Jun",
                      "Jul","Aug","Sep","Oct","Nov","Dec"];
    let suffix = ordinal_suffix(day);
    let weekday = weekday_abbr(day, month, year);
    let m = month_abbr.get(month.saturating_sub(1) as usize)
        .copied().unwrap_or("???");
    format!("{weekday} {day}{suffix} {m}")
}

fn ordinal_suffix(day: u16) -> &'static str {
    match day % 100 {
        11 | 12 | 13 => "th",
        _ => match day % 10 {
            1 => "st", 2 => "nd", 3 => "rd", _ => "th",
        }
    }
}

/// Zeller's congruence — day-of-week from Y-M-D. 0=Sat..6=Fri, but we
/// map to 3-letter abbrev directly.
fn weekday_abbr(day: u16, month: u16, year: u16) -> &'static str {
    let (y, m) = if month < 3 { (year as i32 - 1, month as i32 + 12) }
                 else         { (year as i32,     month as i32) };
    let k = y % 100;
    let j = y / 100;
    let h = (day as i32
        + (13 * (m + 1)) / 5
        + k + k / 4 + j / 4 + 5 * j).rem_euclid(7);
    // Zeller h: 0=Sat, 1=Sun, 2=Mon, 3=Tue, 4=Wed, 5=Thu, 6=Fri
    ["Sat","Sun","Mon","Tue","Wed","Thu","Fri"][h as usize]
}
