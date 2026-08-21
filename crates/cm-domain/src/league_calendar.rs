//! Per-league season start dates — the "starting date for each league".
//!
//! Source: `FUN_006508e0` (`key_nation.cpp`), which builds the picker's league
//! table `DAT_00b4bc70` as 0x48-byte entries. Each entry's dates are built by
//! `FUN_00533b50(day, month, year, -1)`; the season **start** is the first such
//! call in the entry. The country behind each entry was identified from the
//! `.cpp` source of the per-league handler function it installs (e.g. entry 14
//! installs `eng_prm.cpp`).
//!
//! The first 8 table entries (indices 0–7) are the international competitions
//! the picker skips (World/African/European cups, Libertadores, etc.); indices
//! 8–33 are the 26 playable countries. Their start dates are reproduced below.
//!
//! Year handling matches the exe: the base year is `DAT_00acde92` (2001 for the
//! shipped database). Countries that run a **calendar-year** season pass
//! `base_year + 1` to `FUN_00533b50`, i.e. they start in 2002. That set was
//! cross-checked here against the `+1` argument in `FUN_006508e0` and agrees
//! with the calendar/split classification used by the season-label formatter.
//!
//! All VERIFIED from the decompile + a disassembly of the raw-asm handler gap.

/// A league's season-start day within its start year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeasonStart {
    /// Picker country label (matches `real_picker_slots`).
    pub country: &'static str,
    pub day: u8,
    /// 1-based month. Brazil's raw month is 0 in the exe (0-indexed January),
    /// normalised to 1 here.
    pub month: u8,
    /// `false` = split season starting in the base year (2001/02);
    /// `true` = calendar-year season starting in `base_year + 1` (2002).
    pub calendar_year: bool,
    /// The per-league handler VA in cm0102.exe, for traceability.
    pub handler_va: u32,
}

impl SeasonStart {
    /// Resolve the concrete `(year, month, day)` for a given base year
    /// (`DAT_00acde92`, 2001 for shipped data).
    pub fn resolve(&self, base_year: u16) -> (u16, u8, u8) {
        let year = if self.calendar_year { base_year + 1 } else { base_year };
        (year, self.month, self.day)
    }
}

/// The 26 playable leagues' season starts, in picker order.
///
/// `calendar_year` is set for the countries whose `FUN_006508e0` entry passes
/// `year + 1` to the date builder — Argentina, Brazil, Finland, Ireland, Japan,
/// Norway, Sweden, USA — matching a February/March/July calendar-year kickoff.
pub const SEASON_STARTS: &[SeasonStart] = &[
    SeasonStart { country: "Argentina",        day: 9,  month: 7,  calendar_year: false, handler_va: 0x0081bc00 },
    SeasonStart { country: "Australia",        day: 2,  month: 9,  calendar_year: false, handler_va: 0x0081be80 },
    SeasonStart { country: "Belgium",          day: 7,  month: 7,  calendar_year: false, handler_va: 0x0081c0d0 },
    SeasonStart { country: "Brazil",           day: 18, month: 1,  calendar_year: true,  handler_va: 0x0081c480 },
    SeasonStart { country: "Croatia",          day: 24, month: 6,  calendar_year: false, handler_va: 0x0081cbb0 },
    SeasonStart { country: "Denmark",          day: 27, month: 6,  calendar_year: false, handler_va: 0x0081cf00 },
    SeasonStart { country: "England",          day: 10, month: 7,  calendar_year: false, handler_va: 0x0081d250 },
    SeasonStart { country: "Finland",          day: 29, month: 3,  calendar_year: true,  handler_va: 0x0081d7f0 },
    SeasonStart { country: "France",           day: 9,  month: 7,  calendar_year: false, handler_va: 0x0081daf0 },
    SeasonStart { country: "Germany",          day: 29, month: 6,  calendar_year: false, handler_va: 0x0081def0 },
    SeasonStart { country: "Greece",           day: 15, month: 7,  calendar_year: false, handler_va: 0x0081e2c0 },
    SeasonStart { country: "Holland",          day: 16, month: 7,  calendar_year: false, handler_va: 0x0081e610 },
    SeasonStart { country: "Ireland",          day: 1,  month: 7,  calendar_year: false, handler_va: 0x0081e960 },
    SeasonStart { country: "Italy",            day: 25, month: 7,  calendar_year: false, handler_va: 0x0081ee20 },
    SeasonStart { country: "Japan",            day: 6,  month: 2,  calendar_year: true,  handler_va: 0x0081f400 },
    SeasonStart { country: "Northern Ireland", day: 12, month: 7,  calendar_year: false, handler_va: 0x0081f790 },
    SeasonStart { country: "Norway",           day: 12, month: 3,  calendar_year: true,  handler_va: 0x0081fb50 },
    SeasonStart { country: "Poland",           day: 17, month: 6,  calendar_year: false, handler_va: 0x0081fe50 },
    SeasonStart { country: "Portugal",         day: 25, month: 7,  calendar_year: false, handler_va: 0x008201e0 },
    SeasonStart { country: "Russia",           day: 25, month: 2,  calendar_year: true,  handler_va: 0x008205c0 },
    SeasonStart { country: "Scotland",         day: 2,  month: 7,  calendar_year: false, handler_va: 0x008208c0 },
    SeasonStart { country: "Spain",            day: 1,  month: 8,  calendar_year: false, handler_va: 0x00820d00 },
    SeasonStart { country: "Sweden",           day: 6,  month: 3,  calendar_year: true,  handler_va: 0x00821190 },
    SeasonStart { country: "Turkey",           day: 8,  month: 7,  calendar_year: false, handler_va: 0x00821510 },
    SeasonStart { country: "USA",              day: 22, month: 2,  calendar_year: true,  handler_va: 0x00821860 },
    SeasonStart { country: "Wales",            day: 19, month: 7,  calendar_year: false, handler_va: 0x00821b50 },
];

/// Look up a league's season start by its picker country label.
pub fn season_start(country: &str) -> Option<&'static SeasonStart> {
    SEASON_STARTS.iter().find(|s| s.country == country)
}

/// The overall game start date = the EARLIEST season start among the selected
/// leagues. The exe normalises the chosen start against the foreground league's
/// entry (`FUN_0081a020`); for multiple leagues the game must begin on or before
/// the first one to kick off, so we take the minimum.
///
/// Returns `(year, month, day)`. Falls back to 1 July `base_year` when no
/// selected country has a known start (matches the pre-init default date).
pub fn earliest_start(selected: &[String], base_year: u16) -> (u16, u8, u8) {
    selected
        .iter()
        .filter_map(|country| season_start(country))
        .map(|start| start.resolve(base_year))
        .min()
        .unwrap_or((base_year, 7, 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_26_playable_countries_present() {
        assert_eq!(SEASON_STARTS.len(), 26);
    }

    #[test]
    fn calendar_year_countries_start_in_2002() {
        let brazil = season_start("Brazil").unwrap();
        assert_eq!(brazil.resolve(2001), (2002, 1, 18));
        let england = season_start("England").unwrap();
        assert_eq!(england.resolve(2001), (2001, 7, 10));
    }

    #[test]
    fn earliest_of_selected_leagues_wins() {
        // England (10 Jul 2001) + Spain (1 Aug 2001) + USA (22 Feb 2002).
        // The earliest is England.
        let selected = vec!["England".to_string(), "Spain".to_string(), "USA".to_string()];
        assert_eq!(earliest_start(&selected, 2001), (2001, 7, 10));
    }

    #[test]
    fn calendar_country_alone_starts_next_year() {
        let selected = vec!["USA".to_string(), "Japan".to_string()];
        // Japan 6 Feb 2002 is earlier than USA 22 Feb 2002.
        assert_eq!(earliest_start(&selected, 2001), (2002, 2, 6));
    }
}
