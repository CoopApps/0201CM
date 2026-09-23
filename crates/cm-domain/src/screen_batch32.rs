//! Batch 32: 7 more `comp.cpp` functions — the "read/format" pair
//! for player attributes and the season-record catalogue.
//!
//! - `FUN_004a9980` → `read_player_attribute_value`: 19-case numeric
//!   reader for player-history records (the counterpart to
//!   [`crate::screen_batch29::read_team_attribute_value`]).
//! - `FUN_004aa480` → `format_player_attribute`: same 17 cases as
//!   above but returns a formatted string (includes the 5-symbol
//!   result-window formatting for home/away/all form).
//! - `FUN_004aa9a0` → `format_team_attribute`: string form of
//!   [`crate::screen_batch29::read_team_attribute_value`], adds
//!   goals-conceded diff and per-game ratios.
//! - `FUN_004abbe0` → `season_record_column_name`: 22 season-record
//!   column labels ("Most Times Winner", "Top Goalscorer", ...).
//! - `FUN_004abad0` → `history_scope_label`: 2-way "All Time"/"Current".
//! - `FUN_004abf80` → `screen_enable_gate`: 3-way per-tab enable gate.
//! - `FUN_004a9000` → `set_comp_history_flag`: byte-setter at +0x72.
//!
//! Skipped (documented in `carve_rename_map.json`):
//! - `FUN_004a8710` (CompRecord copy-assign) — Rust `#[derive(Clone)]`.
//! - `FUN_004ab310`, `FUN_004ab4f0` — bsearch wrappers.
//! - `FUN_004ab5e0`, `FUN_004ab880` — array filter+copy.
//! - `FUN_004abff0` — dispatch wrapper (needs `FUN_004ac150`).

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_004a9980 — player-history attribute value reader (19 cases)
// =====================================================================

/// The player attributes queried by
/// [`read_player_attribute_value`]. Cases 1..=0x13 in the exe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerHistoryAttr {
    Appearances       = 0x01,   // +0x04 (byte)
    GoalsPerGame      = 0x02,   // (+0x06 i32) / (+0x05 byte), gated on +0x05 != 0
    HomeForm          = 0x03,   // sum(+0x0a..+0x0e)  — 5-char rolling window
    AwayForm          = 0x04,   // sum(+0x0f..+0x13)
    AllForm           = 0x05,   // sum(+0x14..+0x18)
    Shots             = 0x06,   // +0x19 (i32)
    ShotsOnTarget     = 0x07,   // +0x2d (i32)
    Passes            = 0x08,   // +0x41 (i32)
    Interceptions     = 0x09,   // +0x55 (i32)
    Goals             = 0x0A,   // +0x69 (byte)
    Assists           = 0x0B,   // +0x6a (byte)
    ManOfMatch        = 0x0C,   // +0x6b (byte)
    Yellows           = 0x0D,   // +0x6d (byte)
    Reds              = 0x0E,   // +0x6e (byte)
    CleanSheets       = 0x0F,   // +0x6f (byte)
    Fouls             = 0x10,   // +0x70 (byte)
    ManagerOfMonth    = 0x11,   // +0x71 (byte)
    TeamOfMonth       = 0x12,   // +0x72 (byte)
    KeyPasses         = 0x13,   // +0x73 (byte)
}

/// Full 0x74-byte player-history record snapshot as pre-fetched by the
/// caller. The exe walks this record byte-by-byte; we take a snapshot
/// so this stays pure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PlayerHistoryRecord {
    pub appearances:     u8,   // +0x04
    pub games_won_byte:  u8,   // +0x05 (divisor for GoalsPerGame)
    pub goals_int:       i32,  // +0x06
    /// 5-char rolling home-form window at +0x0a..+0x0e.
    pub home_form_window: [i8; 5],
    /// 5-char rolling away-form window at +0x0f..+0x13.
    pub away_form_window: [i8; 5],
    /// 5-char rolling all-form window at +0x14..+0x18.
    pub all_form_window:  [i8; 5],
    pub shots_i32:        i32,  // +0x19
    pub shots_on_i32:     i32,  // +0x2d
    pub passes_i32:       i32,  // +0x41
    pub interceptions_i32: i32, // +0x55
    // Bytes at +0x69..+0x73
    pub bytes_69_to_73: [u8; 11],
}

/// The return value of [`read_player_attribute_value`]. `Sentinel`
/// represents the exe's `_DAT_00956970` "no value" constant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlayerAttributeValue {
    Int(i32),
    Ratio(f64),
    Sentinel,
}

/// Direct port of `FUN_004a9980(record, attr)`.
// GDI-REG: 004a9980 PORTED_BEHAVIOURAL
pub fn read_player_attribute_value(
    rec: &PlayerHistoryRecord,
    attr: PlayerHistoryAttr,
) -> PlayerAttributeValue {
    use PlayerHistoryAttr::*;
    match attr {
        Appearances => PlayerAttributeValue::Int(rec.appearances as i32),
        GoalsPerGame => {
            if rec.games_won_byte == 0 {
                PlayerAttributeValue::Sentinel
            } else {
                PlayerAttributeValue::Ratio(rec.goals_int as f64 / rec.games_won_byte as f64)
            }
        }
        HomeForm => PlayerAttributeValue::Int(rec.home_form_window.iter().map(|&b| b as i32).sum()),
        AwayForm => PlayerAttributeValue::Int(rec.away_form_window.iter().map(|&b| b as i32).sum()),
        AllForm  => PlayerAttributeValue::Int(rec.all_form_window.iter().map(|&b| b as i32).sum()),
        Shots         => PlayerAttributeValue::Int(rec.shots_i32),
        ShotsOnTarget => PlayerAttributeValue::Int(rec.shots_on_i32),
        Passes        => PlayerAttributeValue::Int(rec.passes_i32),
        Interceptions => PlayerAttributeValue::Int(rec.interceptions_i32),
        Goals         => PlayerAttributeValue::Int(rec.bytes_69_to_73[0] as i32),   // +0x69
        Assists       => PlayerAttributeValue::Int(rec.bytes_69_to_73[1] as i32),   // +0x6a
        ManOfMatch    => PlayerAttributeValue::Int(rec.bytes_69_to_73[2] as i32),   // +0x6b
        // +0x6c is skipped in the exe (not a case value).
        Yellows       => PlayerAttributeValue::Int(rec.bytes_69_to_73[4] as i32),   // +0x6d
        Reds          => PlayerAttributeValue::Int(rec.bytes_69_to_73[5] as i32),   // +0x6e
        CleanSheets   => PlayerAttributeValue::Int(rec.bytes_69_to_73[6] as i32),   // +0x6f
        Fouls         => PlayerAttributeValue::Int(rec.bytes_69_to_73[7] as i32),   // +0x70
        ManagerOfMonth=> PlayerAttributeValue::Int(rec.bytes_69_to_73[8] as i32),   // +0x71
        TeamOfMonth   => PlayerAttributeValue::Int(rec.bytes_69_to_73[9] as i32),   // +0x72
        KeyPasses     => PlayerAttributeValue::Int(rec.bytes_69_to_73[10] as i32),  // +0x73
    }
}

// =====================================================================
// FUN_004aa480 — player-attr FORMATTED reader (17 cases + 5-window
//                symbol format)
// =====================================================================

/// The 4 result-window bullet symbols the exe emits when formatting
/// HomeForm / AwayForm / AllForm.
pub mod form_symbols {
    /// Result char = 3 → "W<...won>".
    pub const WON:  &str = "W";
    /// Result char = 2 → "X<...score draw>".
    pub const SCORE_DRAW: &str = "X";
    /// Result char = 1 → "D<...drawn>".
    pub const DRAW: &str = "D";
    /// Result char = -3 → "L<...lost>".
    pub const LOST: &str = "L";
}

/// Format a 5-char rolling form window into the exe's "WXDWL" style
/// symbol string. Reads left-to-right, decoding each result char per
/// `form_symbols`.
pub fn format_form_window(window: [i8; 5]) -> String {
    let mut s = String::new();
    // The exe walks the window from LAST-to-FIRST (reversed). Verified
    // from `pcVar7 = (char *)(param_1 + 0xe); do { ... } while (int(pcVar7 + (-10 - param_1)) < 0);`
    for &b in window.iter().rev() {
        match b {
            3 => s.push_str(form_symbols::WON),
            2 => s.push_str(form_symbols::SCORE_DRAW),
            1 => s.push_str(form_symbols::DRAW),
            -3 => s.push_str(form_symbols::LOST),
            _ => {}   // other values (0, ...) → skip
        }
    }
    s
}

/// String-formatted output for one player-history attribute.
///
/// Matches the exe's `FUN_004aa480` — same 17 cases plus the fifth
/// (KeyPasses) as case 0x11.
// GDI-REG: 004aa480 PORTED_EXACT
pub fn format_player_attribute(
    rec: &PlayerHistoryRecord,
    attr: PlayerHistoryAttr,
    include_secondary: bool,
) -> Option<String> {
    use PlayerHistoryAttr::*;
    let fmt_int  = |v: i32| Some(format!("{v}"));
    let fmt_pair = |a: i32, b: i32| Some(format!("{a} ({b})"));
    match attr {
        Appearances => {
            // If games_won > 0 && include_secondary → "apps (subs)" where
            // subs = appearances - games_won (verified from
            // `FUN_00933d2f(param_3, "%d (%d)", byte_c - byte_d, byte_d)`).
            if rec.games_won_byte != 0 && include_secondary {
                fmt_pair(rec.appearances as i32 - rec.games_won_byte as i32,
                         rec.games_won_byte as i32)
            } else {
                fmt_int(rec.appearances as i32)
            }
        }
        GoalsPerGame => {
            if rec.games_won_byte == 0 { return None; }
            Some(format!("{}", rec.goals_int / rec.games_won_byte as i32))
        }
        HomeForm => Some(format_form_window(rec.home_form_window)),
        AwayForm => Some(format_form_window(rec.away_form_window)),
        AllForm  => Some(format_form_window(rec.all_form_window)),
        // ManOfMatch (0x0C) — if secondary && bytes[3] > 0 → "man (mom_2nd)"
        ManOfMatch => {
            let mom  = rec.bytes_69_to_73[2] as i32;   // +0x6b
            let mom2 = rec.bytes_69_to_73[3] as i32;   // +0x6c
            if mom > 0 && include_secondary && mom2 > 0 {
                fmt_pair(mom, mom2)
            } else {
                fmt_int(mom)
            }
        }
        // Every other case → single-int format
        Shots         => fmt_int(rec.shots_i32),
        ShotsOnTarget => fmt_int(rec.shots_on_i32),
        Passes        => fmt_int(rec.passes_i32),
        Interceptions => fmt_int(rec.interceptions_i32),
        Goals         => fmt_int(rec.bytes_69_to_73[0] as i32),
        Assists       => fmt_int(rec.bytes_69_to_73[1] as i32),
        Yellows       => fmt_int(rec.bytes_69_to_73[4] as i32),
        Reds          => fmt_int(rec.bytes_69_to_73[5] as i32),
        CleanSheets   => fmt_int(rec.bytes_69_to_73[6] as i32),
        Fouls         => fmt_int(rec.bytes_69_to_73[7] as i32),
        ManagerOfMonth=> fmt_int(rec.bytes_69_to_73[8] as i32),
        TeamOfMonth   => fmt_int(rec.bytes_69_to_73[9] as i32),
        KeyPasses     => fmt_int(rec.bytes_69_to_73[10] as i32),
    }
}

// =====================================================================
// FUN_004abbe0 — 22 season-record column names
// =====================================================================

/// The 22 column labels the season-records screen displays.
pub const SEASON_RECORD_COLUMN_NAMES: [&str; 22] = [
    "Most Times Winner",       // 1
    "Most Team Points",        // 2
    "Most Team Goals",         // 3
    "Most Team Conceded",      // 4
    "Worst Team Discipline",   // 5
    "Highest Attendance",      // 6
    "Lowest Attendance",       // 7
    "Highest Average Attendance", // 8
    "Biggest Win",             // 9
    "Highest Scoring Game",    // 10
    "Most Games Won in Row",   // 11
    "Most Games Lost in Row",  // 12
    "Most Games Without Losing", // 13
    "Most Games Without Winning", // 14
    "Top Goalscorer",          // 15
    "Most Goals in Match",     // 16
    "Most Assists",            // 17
    "Highest Average Rating",  // 18
    "Most Man of Match",       // 19
    "Worst Discipline",        // 20
    "Youngest Player",         // 21
    "Oldest Player",           // 22
];

/// Look up a season-record column label by 1-based index. Returns
/// `None` for out-of-range indices.
// GDI-REG: 004abbe0 PORTED_EXACT
pub fn season_record_column_name(idx: u8) -> Option<&'static str> {
    if (1..=22).contains(&idx) {
        Some(SEASON_RECORD_COLUMN_NAMES[(idx - 1) as usize])
    } else {
        None
    }
}

// =====================================================================
// FUN_004abad0 — 2-way history-scope label
// =====================================================================

/// Direct port of `FUN_004abad0(scope, out, out_len)`.
// GDI-REG: 004abad0 PORTED_EXACT
pub fn history_scope_label(scope: u8) -> Option<&'static str> {
    match scope {
        1 => Some("All Time"),
        2 => Some("Current"),
        _ => None,
    }
}

// =====================================================================
// FUN_004abf80 — 3-way per-tab enable gate
// =====================================================================

/// Which UI branch [`screen_enable_gate`] is checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenGateBranch {
    /// `param_2 == 2` — required predicate: `FUN_004b6b30(...) != 0`.
    RequiresRefreshOk,
    /// `param_2 == 8` — required predicate: comp_type_byte != 0x04.
    ExcludesCompType4,
    /// Every other value — no per-branch predicate.
    NoExtraGate,
}

/// Direct port of `FUN_004abf80(comp, tab)`.
///
/// Inputs:
/// - `outer_gate_ok`: `FUN_005b09e0(...) == 0` — outer prerequisite.
/// - `branch`: which tab is being tested.
/// - `refresh_gate_ok`: value of `FUN_004b6b30(...) != 0` (only
///   consulted for `RequiresRefreshOk`).
/// - `comp_type_byte`: value of `+0x502[0]+0x42` (only consulted for
///   `ExcludesCompType4`).
// GDI-REG: 004abf80 PORTED_BEHAVIOURAL
pub fn screen_enable_gate(
    outer_gate_ok: bool,
    branch: ScreenGateBranch,
    refresh_gate_ok: bool,
    comp_type_byte: u8,
) -> bool {
    if !outer_gate_ok { return false; }
    match branch {
        ScreenGateBranch::RequiresRefreshOk    => refresh_gate_ok,
        ScreenGateBranch::ExcludesCompType4    => comp_type_byte != 0x04,
        ScreenGateBranch::NoExtraGate          => true,
    }
}

// =====================================================================
// FUN_004a9000 — competition-history flag byte setter
// =====================================================================

/// Direct port of `FUN_004a9000(team, value)`. Writes `value` to the
/// competition-history record's `+0x72` byte (TeamOfMonth flag —
/// matches [`PlayerHistoryAttr::TeamOfMonth`] in read direction).
// GDI-REG: 004a9000 PORTED_BEHAVIOURAL
pub fn set_comp_history_team_of_month(
    team_history_record: &mut PlayerHistoryRecord,
    value: u8,
) {
    team_history_record.bytes_69_to_73[9] = value;
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- read_player_attribute_value ----

    fn make_rec() -> PlayerHistoryRecord {
        let mut r = PlayerHistoryRecord::default();
        r.appearances = 30;
        r.games_won_byte = 6;
        r.goals_int = 24;
        r.home_form_window = [3, 3, 1, 2, 3];   // WWDXW = 12
        r.away_form_window = [-3, 1, 3, 1, 2];  // LDWDX = 4
        r.all_form_window  = [3, 3, 3, -3, 1];  // WWWLD = 7
        r.shots_i32 = 100;
        r.shots_on_i32 = 45;
        r.passes_i32 = 890;
        r.interceptions_i32 = 25;
        // +0x69..+0x73: Goals, Assists, MoM, MoM2, Yellows, Reds, CS, Fouls, ManOfMonth, TeamOfMonth, KeyPasses
        r.bytes_69_to_73 = [15, 8, 2, 1, 4, 0, 12, 30, 1, 0, 20];
        r
    }

    #[test]
    fn player_attr_appearances_returns_byte() {
        let r = make_rec();
        assert_eq!(read_player_attribute_value(&r, PlayerHistoryAttr::Appearances),
                   PlayerAttributeValue::Int(30));
    }

    #[test]
    fn player_attr_goals_per_game_ratio() {
        let r = make_rec();
        let v = read_player_attribute_value(&r, PlayerHistoryAttr::GoalsPerGame);
        assert_eq!(v, PlayerAttributeValue::Ratio(24.0 / 6.0));
    }

    #[test]
    fn player_attr_goals_per_game_divide_zero_returns_sentinel() {
        let mut r = make_rec();
        r.games_won_byte = 0;
        assert_eq!(read_player_attribute_value(&r, PlayerHistoryAttr::GoalsPerGame),
                   PlayerAttributeValue::Sentinel);
    }

    #[test]
    fn player_attr_form_windows_sum_5_chars() {
        let r = make_rec();
        // HomeForm: 3+3+1+2+3 = 12
        assert_eq!(read_player_attribute_value(&r, PlayerHistoryAttr::HomeForm),
                   PlayerAttributeValue::Int(12));
        // AwayForm: -3+1+3+1+2 = 4
        assert_eq!(read_player_attribute_value(&r, PlayerHistoryAttr::AwayForm),
                   PlayerAttributeValue::Int(4));
        // AllForm: 3+3+3-3+1 = 7
        assert_eq!(read_player_attribute_value(&r, PlayerHistoryAttr::AllForm),
                   PlayerAttributeValue::Int(7));
    }

    #[test]
    fn player_attr_byte_offsets_map_correctly() {
        let r = make_rec();
        assert_eq!(read_player_attribute_value(&r, PlayerHistoryAttr::Goals),
                   PlayerAttributeValue::Int(15));
        assert_eq!(read_player_attribute_value(&r, PlayerHistoryAttr::Yellows),
                   PlayerAttributeValue::Int(4));
        assert_eq!(read_player_attribute_value(&r, PlayerHistoryAttr::KeyPasses),
                   PlayerAttributeValue::Int(20));
    }

    // ---- format_form_window ----

    #[test]
    fn form_window_reversed_wxdxl() {
        // Window [3, 3, 1, 2, 3] read reversed: 3(W) 2(X) 1(D) 3(W) 3(W) → "WXDWW"
        assert_eq!(format_form_window([3, 3, 1, 2, 3]), "WXDWW");
        // -3 = L
        assert_eq!(format_form_window([-3, 1, 3, 1, 2]), "XDWDL");
        // Zeros and unmapped values → skipped
        assert_eq!(format_form_window([0, 1, 0, 3, 0]), "WD");
    }

    // ---- format_player_attribute ----

    #[test]
    fn format_appearances_pair_when_secondary() {
        let r = make_rec();   // apps=30, games_won=6
        assert_eq!(format_player_attribute(&r, PlayerHistoryAttr::Appearances, true),
                   Some("24 (6)".into()));
        assert_eq!(format_player_attribute(&r, PlayerHistoryAttr::Appearances, false),
                   Some("30".into()));
    }

    #[test]
    fn format_mom_secondary_pair_when_both_nonzero() {
        let r = make_rec();   // MoM=2, MoM2=1
        assert_eq!(format_player_attribute(&r, PlayerHistoryAttr::ManOfMatch, true),
                   Some("2 (1)".into()));
        // include_secondary false → single int
        assert_eq!(format_player_attribute(&r, PlayerHistoryAttr::ManOfMatch, false),
                   Some("2".into()));
    }

    #[test]
    fn format_goals_per_game_integer_truncation() {
        let mut r = make_rec();
        r.goals_int = 25;
        r.games_won_byte = 6;   // 25/6 = 4.16 → truncates to 4
        assert_eq!(format_player_attribute(&r, PlayerHistoryAttr::GoalsPerGame, false),
                   Some("4".into()));
    }

    // ---- season_record_column_name ----

    #[test]
    fn season_record_columns_length_and_endpoints() {
        assert_eq!(SEASON_RECORD_COLUMN_NAMES.len(), 22);
        assert_eq!(season_record_column_name(1),  Some("Most Times Winner"));
        assert_eq!(season_record_column_name(22), Some("Oldest Player"));
        assert_eq!(season_record_column_name(0),  None);
        assert_eq!(season_record_column_name(23), None);
        // Middle sanity check
        assert_eq!(season_record_column_name(15), Some("Top Goalscorer"));
    }

    // ---- history_scope_label ----

    #[test]
    fn history_scope_labels() {
        assert_eq!(history_scope_label(1), Some("All Time"));
        assert_eq!(history_scope_label(2), Some("Current"));
        assert_eq!(history_scope_label(0), None);
        assert_eq!(history_scope_label(3), None);
    }

    // ---- screen_enable_gate ----

    #[test]
    fn gate_outer_off_always_false() {
        assert!(!screen_enable_gate(false, ScreenGateBranch::RequiresRefreshOk, true, 0));
        assert!(!screen_enable_gate(false, ScreenGateBranch::NoExtraGate, true, 0));
    }

    #[test]
    fn gate_branch_2_requires_refresh_ok() {
        assert!(screen_enable_gate(true, ScreenGateBranch::RequiresRefreshOk, true, 0));
        assert!(!screen_enable_gate(true, ScreenGateBranch::RequiresRefreshOk, false, 0));
    }

    #[test]
    fn gate_branch_8_excludes_comp_type_4() {
        assert!(screen_enable_gate(true, ScreenGateBranch::ExcludesCompType4, false, 3));
        assert!(!screen_enable_gate(true, ScreenGateBranch::ExcludesCompType4, false, 4));
        assert!(screen_enable_gate(true, ScreenGateBranch::ExcludesCompType4, false, 5));
    }

    #[test]
    fn gate_other_branches_pass_when_outer_ok() {
        assert!(screen_enable_gate(true, ScreenGateBranch::NoExtraGate, false, 4));
    }

    // ---- set_comp_history_team_of_month ----

    #[test]
    fn setter_writes_to_byte_offset_9() {
        let mut r = make_rec();
        set_comp_history_team_of_month(&mut r, 5);
        // Confirm via reader that TeamOfMonth is now 5
        assert_eq!(read_player_attribute_value(&r, PlayerHistoryAttr::TeamOfMonth),
                   PlayerAttributeValue::Int(5));
    }

    // ---- 5-char window boundary ----

    #[test]
    fn form_windows_are_5_chars_each() {
        let r = make_rec();
        assert_eq!(r.home_form_window.len(), 5);
        assert_eq!(r.away_form_window.len(), 5);
        assert_eq!(r.all_form_window.len(),  5);
    }
}
