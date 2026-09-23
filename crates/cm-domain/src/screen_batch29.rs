//! Batch 29: 12 more comp_screens.cpp / comp.cpp functions ported.
//!
//! Continues [`crate::screen_batch28`]. The main additions here are:
//! - `comp_slot_probe` (`FUN_004a5610`) — the 22-slot switch table
//!   [batch 28's `comp_list_screen_build`] queries at each row. Maps
//!   slot index → (return code, sub-row count).
//! - Two more command pre-dispatcher variants (`FUN_004a16a0`,
//!   `FUN_004a3220`) — same pattern as batch 28's `comp_command_dispatch`
//!   but gated on gate byte == 9 or == 2 (instead of == 1).
//! - Screen launchers (`FUN_004a17f0`, `FUN_004a28c0`, `FUN_004a2190`)
//!   that register a new comp/fixture/rankings screen and seed its
//!   field-map.
//! - Fixture team-lookup (`FUN_004a5760`) — resolves (comp, home, away)
//!   from a fixture id + team-list scan.
//! - Two stat-column-name switch tables (`FUN_004a92d0` team stats,
//!   `FUN_004a95d0` player stats) — 17 columns each.
//! - Team attribute-value reader (`FUN_004a9d10`) — 18-case switch
//!   computing raw byte reads, per-game averages, and reputation-
//!   gated ratios.
//! - Pending-comp-news drain (`FUN_004a8f10`) — walks 22 dirty flags
//!   and emits one news item per set flag.

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_004a5610 — 22-slot switch table (comp_slot_probe)
// =====================================================================

/// The return values `FUN_004a5610` produces per slot index. The exe
/// packs two facts into the return + out-param: a "row family code"
/// (the return value) and an optional "sub-row count" (via the out
/// param).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompSlotClassification {
    /// The slot family code:
    /// - `0xFF` for slot 1 (special sentinel)
    /// - `6` for slots 2..=5   (rows-of-`0x14` family)
    /// - `3` for slots 6..=8   (rows-of-`0x0b` family)
    /// - `4` for slots 9..=10  (rows-of-`0x0e` family)
    /// - `5` for slots 11..=14 (rows-of-`0x11` family)
    /// - `7` for slots 15..=22 (rows-of-`0x17` family)
    /// - `3` (fallback) for anything else — the exe also raises an
    ///   "Error" MessageBox on out-of-range slots.
    pub family_code: u8,
    /// The sub-row count the exe writes into `*param_2` when non-null.
    /// `None` for slot 1 (no out-param write).
    pub row_count: Option<u8>,
    /// `true` iff the slot index was outside 1..=22, in which case the
    /// exe raises an "Error" dialog and returns family_code `3` as
    /// fallback.
    pub out_of_range: bool,
}

/// Direct port of `FUN_004a5610(slot: u8, out_row_count: &mut u32)`.
// GDI-REG: 004a5610 PORTED_BEHAVIOURAL
pub fn comp_slot_probe(slot: u8) -> CompSlotClassification {
    match slot {
        1 => CompSlotClassification { family_code: 0xFF, row_count: None,       out_of_range: false },
        2..=5   => CompSlotClassification { family_code: 6, row_count: Some(0x14), out_of_range: false },
        6..=8   => CompSlotClassification { family_code: 3, row_count: Some(0x0b), out_of_range: false },
        9..=10  => CompSlotClassification { family_code: 4, row_count: Some(0x0e), out_of_range: false },
        11..=14 => CompSlotClassification { family_code: 5, row_count: Some(0x11), out_of_range: false },
        15..=22 => CompSlotClassification { family_code: 7, row_count: Some(0x17), out_of_range: false },
        _ => CompSlotClassification { family_code: 3, row_count: None,       out_of_range: true },
    }
}

// =====================================================================
// FUN_004a16a0 / FUN_004a3220 — sibling command pre-dispatchers
// =====================================================================

/// Direct port of `FUN_004a16a0` — a sibling of
/// [`crate::screen_batch28::dispatch_comp_command`]. Same seat pool +
/// stride layout, but gated on gate byte == 9 (instead of 1) and only
/// consults a single sibling field (0x17 by tag).
///
/// Returns [`crate::screen_batch28::HANDLED_REFRESH_SENTINEL`] (-11)
/// when the id at the slot doesn't match the field's current value —
/// otherwise falls through with `0`.
// GDI-REG: 004a16a0 PORTED_PARTIAL
pub fn dispatch_comp_command_val9(
    slot: i16,
    gate_byte: u16,
    fallback_gate_byte: u16,
    slot_id_word: u32,
    field_0x17_value: u32,
) -> i32 {
    let effective_gate = if slot < 0 { fallback_gate_byte } else { gate_byte };
    if effective_gate != 9 { return 0; }
    let id = if slot < 0 { 0 } else { slot_id_word };
    if id == field_0x17_value {
        0   // no change, no refresh
    } else {
        crate::screen_batch28::HANDLED_REFRESH_SENTINEL   // -11: write & refresh
    }
}

/// Outcome of the `FUN_004a3220` dispatcher. Gate byte == 2 triggers a
/// tactic-loader side effect; gate byte == 1 flips two loader flags.
/// Anything else falls through to the shared dispatchers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompLoaderOutcome {
    /// Gate byte == 2: call `FUN_0076eb10` to load; on success emit
    /// two `FUN_0076ecd0(..., 0, 1, 0)` and `(..., 3, 1, 0)` and
    /// return -9. Loader failure → error dialog + return 0.
    LoaderPathOk,
    LoaderPathError,
    /// Gate byte == 1: emit two `FUN_0076ecd0(..., 0, 1, 0)` and
    /// `(..., 3, 0, 0)` (note the second flag differs) and return -9.
    ToggleFlagsPath,
    /// Anything else: fall through to shared dispatchers.
    FallThroughShared,
}

/// Direct port of `FUN_004a3220(slot)`.
// GDI-REG: 004a3220 PORTED_BEHAVIOURAL
pub fn dispatch_comp_loader(
    slot: i16,
    gate_byte: u16,
    fallback_gate_byte: u16,
    loader_ok_when_called: bool,
) -> CompLoaderOutcome {
    let effective_gate = if slot < 0 { fallback_gate_byte } else { gate_byte };
    match effective_gate {
        2 => if loader_ok_when_called { CompLoaderOutcome::LoaderPathOk }
             else { CompLoaderOutcome::LoaderPathError },
        1 => CompLoaderOutcome::ToggleFlagsPath,
        _ => CompLoaderOutcome::FallThroughShared,
    }
}

// =====================================================================
// FUN_004a5760 — extract (comp, home, away) from a fixture id
// =====================================================================

/// Direct port of `FUN_004a5760(fixture, out_comp, out_home, out_away)`.
///
/// Returns [`ExtractFixtureTeamsResult`] with the resolved ids, or a
/// failure classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractFixtureTeamsResult {
    /// All three outputs resolved.
    Resolved { comp: u32, home: u32, away: u32 },
    /// The fixture was invalid: comp id < 0, comp table entry null,
    /// reserves-crossover guard fired, `+0x43` wasn't 2, or team not
    /// found in the competition's team list.
    NotResolved,
    /// Any input pointer was null — the exe raises the "Error" dialog.
    NullInput,
}

/// Direct port of `FUN_004a5760(fixture, out_comp, out_home, out_away)`.
///
/// Inputs (pre-fetched from the fixture record):
/// - `comp_id`: `fixture[0]` — competition slot index (< 0 → NotResolved).
/// - `comp_table_ptr_ok`: `DAT_00ac688c[comp_id] != 0`.
/// - `reserves_guard_blocks`: `FUN_007cf040(fixture[5]) != 0`.
/// - `comp_stage_byte`: `fixture + 0x42` — negative means "special:
///   use the comp record directly"; >= 0 means "index into
///   comp+0xc[stage]" for the round record.
/// - `round_type_is_2`: `round+0x43 == 2`.
/// - `home_found`, `away_found`: outputs of the team-list scan
///   `FUN_006679a0` looking for `fixture[7]` and `fixture[8]`.
// GDI-REG: 004a5760 PORTED_BEHAVIOURAL
pub fn extract_fixture_teams(
    any_input_null: bool,
    comp_id: i32,
    comp_table_ptr_ok: bool,
    reserves_guard_blocks: bool,
    comp_stage_byte: i8,
    resolved_comp: u32,
    round_type_is_2: bool,
    home_found: Option<u32>,
    away_found: Option<u32>,
) -> ExtractFixtureTeamsResult {
    if any_input_null { return ExtractFixtureTeamsResult::NullInput; }
    if comp_id < 0 || !comp_table_ptr_ok || reserves_guard_blocks {
        return ExtractFixtureTeamsResult::NotResolved;
    }
    // `comp_stage_byte < 0` uses the comp record directly; otherwise
    // walks the comp+0xc round table. Either way, the caller supplies
    // the resolved `resolved_comp`.
    let _ = comp_stage_byte;   // kept for callsite symmetry
    if !round_type_is_2 { return ExtractFixtureTeamsResult::NotResolved; }
    match (home_found, away_found) {
        (Some(home), Some(away)) =>
            ExtractFixtureTeamsResult::Resolved { comp: resolved_comp, home, away },
        _ => ExtractFixtureTeamsResult::NotResolved,
    }
}

// =====================================================================
// FUN_004a17f0 / FUN_004a28c0 / FUN_004a2190 — screen launchers
// =====================================================================

/// Slot-map seed for a fixture-view screen (`FUN_004a17f0`). The exe
/// registers a new screen via `FUN_007e6570(builder, dispatcher)` then
/// writes 5 fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FixtureViewSeed {
    pub field_0_comp: u32,
    pub field_1_home: u32,
    pub field_2_away: u32,
    pub field_3_zero: u32,
    pub field_4_zero: u32,
}

/// Direct port of `FUN_004a17f0(fixture_id)`. Guards:
/// - `fixture_id == 0` → error dialog, no screen.
/// - `|fixture_id| <= 9999` → early return without registering.
/// - Fixture-teams extraction fails and all outputs zero → error, no screen.
// GDI-REG: 004a17f0 PORTED_BEHAVIOURAL
pub fn launch_fixture_view(
    fixture_id: i32,
    extraction: ExtractFixtureTeamsResult,
) -> Option<FixtureViewSeed> {
    if fixture_id == 0 { return None; }
    if fixture_id.wrapping_abs() <= 9999 { return None; }
    match extraction {
        ExtractFixtureTeamsResult::Resolved { comp, home, away } =>
            Some(FixtureViewSeed {
                field_0_comp: comp, field_1_home: home, field_2_away: away,
                field_3_zero: 0, field_4_zero: 0,
            }),
        ExtractFixtureTeamsResult::NotResolved =>
            // The exe accepts a partial extraction if either home or
            // away was non-zero; caller passes NotResolved only when
            // both were zero, so we bail here.
            None,
        ExtractFixtureTeamsResult::NullInput => None,
    }
}

/// Direct port of `FUN_004a28c0` — trivial launcher that seeds
/// `field(0) = 0, field(1) = 0`.
// GDI-REG: 004a28c0 PORTED_BEHAVIOURAL
pub fn launch_uefa_coefs_screen() -> (u32, u32) { (0, 0) }

/// Direct port of `FUN_004a2190` — launcher seeding
/// `field(0)=0, field(1)=0, field(2)=1, field(3)=1`.
pub fn launch_fifa_rankings_screen() -> (u32, u32, u32, u32) { (0, 0, 1, 1) }

// =====================================================================
// FUN_004a92d0 — team stat column names (17 columns)
// =====================================================================

/// The exact strings the exe emits for team-stat column labels.
/// Verified string-by-string against the switch table in `FUN_004a92d0`.
pub const TEAM_STAT_COLUMN_NAMES: [&str; 17] = [
    "Games",                 // 1
    "Average Attendance",    // 2
    "Ratio",                 // 3 (DAT_0097b2f0)
    "Home Form",             // 4
    "Away Form",             // 5
    "Games Won in Row",      // 6
    "Games Lost in Row",     // 7
    "Games Without Losing",  // 8
    "Games Without Winning", // 9
    "Goals",                 // 10
    "Conceded",              // 11
    "Penalties",             // 12
    "Goals from Corners",    // 13
    "Goals from IFKs",       // 14
    "Goals from DFKs",       // 15
    "Yellow Cards",          // 16
    "Red Cards",             // 17
];

/// Look up a team-stat column name by 1-based index. Returns `None`
/// for out-of-range indices (the exe raises an "Error" dialog).
// GDI-REG: 004a92d0 PORTED_EXACT
pub fn team_stat_column_name(idx: u8) -> Option<&'static str> {
    if (1..=17).contains(&idx) {
        Some(TEAM_STAT_COLUMN_NAMES[(idx - 1) as usize])
    } else {
        None
    }
}

// =====================================================================
// FUN_004a95d0 — player stat column names (17 columns)
// =====================================================================

pub const PLAYER_STAT_COLUMN_NAMES: [&str; 17] = [
    "Appearances",     // 1
    "Goals",           // 2
    "Conceded",        // 3
    "Penalties",       // 4
    "Assists",         // 5
    "Team Goals",      // 6
    "Team Conceded",   // 7
    "Games Won",       // 8
    "Games Lost",      // 9
    "Yellow Cards",    // 10
    "Red Cards",       // 11
    "Man of Match",    // 12
    "Pass Completion", // 13
    "Tackles / Game",  // 14
    "Dribbles / Game", // 15
    "Shots On Target", // 16
    "Average Rating",  // 17
];

/// Look up a player-stat column name by 1-based index.
// GDI-REG: 004a95d0 PORTED_EXACT
pub fn player_stat_column_name(idx: u8) -> Option<&'static str> {
    if (1..=17).contains(&idx) {
        Some(PLAYER_STAT_COLUMN_NAMES[(idx - 1) as usize])
    } else {
        None
    }
}

// =====================================================================
// FUN_004a9d10 — team attribute value reader (18 cases)
// =====================================================================

/// Which team attribute to read via `read_team_attribute_value`.
/// Cases 0x01..=0x0C read raw bytes; 0x0D..=0x11 read ratios gated on
/// a competition-reputation threshold; 0x12 is a simple goals ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TeamAttribute {
    Games              = 0x01,   // team +0x0c
    AverageAttendance  = 0x02,   // team +0x0e
    Ratio              = 0x03,   // team +0x0f
    HomeForm           = 0x04,   // team +0x11
    AwayForm           = 0x05,   // team +0x12
    GamesWonInRow      = 0x06,   // team +0x13
    GamesLostInRow     = 0x07,   // team +0x14
    GamesWithoutLoss   = 0x08,   // team +0x15
    GamesWithoutWin    = 0x09,   // team +0x16
    Goals              = 0x0A,   // team +0x17
    Conceded           = 0x0B,   // team +0x18
    Penalties          = 0x0C,   // team +0x19
    /// Ratio (short at +0x1f) / (short at +0x1d), gated on:
    /// - `comp_rep * K1 <= team +0x29`
    /// - `team +0x1d > 0x14`
    /// - `team +0x1d > 500 || (team +0x29 + (team +0x29 >> 31 & 3)) >> 2 < team +0x1d`
    /// Result is scaled by the constant at `_DAT_00956960`.
    GoalsFromCorners   = 0x0D,
    /// (short at +0x21) / (byte at +0x0c), gated on:
    /// - `comp_rep * K1 <= team +0x29`
    /// - `team +0x29 > 0x95`
    GoalsFromIfks      = 0x0E,
    /// (short at +0x23) / (byte at +0x0c), same gate as GoalsFromIfks.
    GoalsFromDfks      = 0x0F,
    /// Ratio (short at +0x27) / (short at +0x25), gated on:
    /// - `comp_rep * K1 <= team +0x29`
    /// - `team +0x25 > 0x18` OR
    ///   (`team +0x25 > 4` AND `team +0x29 / team +0x25 < 0x2e`) OR
    ///   (`team +0x29 > 0x21c` AND `team +0x25 > 10`)
    /// Scaled by `_DAT_00956960`.
    YellowCards        = 0x10,
    /// (short at +0x1a) / (byte at +0x0c), gated on:
    /// - `attr(1) * K2 <= comp_rep`
    /// - `team +0x0c != 0`
    RedCards           = 0x11,
    /// Simple: (byte at +0x0f) / (byte at +0x0c). No reputation gate.
    GamesRatio         = 0x12,
}

/// Result of reading a team attribute. Special sentinel value
/// `_DAT_00956970` (default/error) is represented as `None`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TeamAttributeValue {
    /// A byte value (cases 1..=12) — the raw stat count.
    Byte(u8),
    /// A ratio (cases 13..=18) — pre-normalized floating-point result.
    Ratio(f64),
    /// The exe's "no value" sentinel (comp record missing, threshold
    /// gate failed, divide-by-zero).
    Sentinel,
}

/// Direct port of `FUN_004a9d10(team_ptr, param_2)`.
///
/// The exe uses x87 float10 arithmetic throughout; f64 is close
/// enough for reproduction.
///
/// Inputs (pre-extracted from the team record):
/// - `bytes_c_through_19`: the 14 raw bytes at team+0x0c..=+0x19,
///   indexed 0..=13 (offset 0 = +0x0c).
/// - `s_1a_1d_1f_21_23_25_27`: seven shorts at team+0x1a, +0x1d,
///   +0x1f, +0x21, +0x23, +0x25, +0x27.
/// - `int_29`: the i32 at team+0x29.
/// - `comp_reputation_ok`: `DAT_00acd5bc + team[8] * 0x245` non-null
///   AND `FUN_004ab310(comp)` non-null (both branches yield
///   `_DAT_00956970` fallback when either fails).
/// - `comp_attr1_value`: `FUN_004a9980(comp, 1)` — used by RedCards.
/// - `k1`: `_DAT_00956a7c` — the reputation-vs-team-stat comparison
///   constant.
/// - `k2`: `_DAT_00956968` — RedCards-specific comparison constant.
/// - `scale_960`: `_DAT_00956960` — the corners/yellow-cards scaling
///   constant.
// GDI-REG: 004a9d10 PORTED_BEHAVIOURAL
pub fn read_team_attribute_value(
    attr: TeamAttribute,
    bytes_c_through_19: [u8; 14],
    s_1a: i16, s_1d: i16, s_1f: i16, s_21: i16, s_23: i16, s_25: i16, s_27: i16,
    int_29: i32,
    comp_reputation_ok: bool,
    comp_attr1_value: f64,
    k1: f64, k2: f64, scale_960: f64,
) -> TeamAttributeValue {
    let byte_at = |offset_from_c: usize| bytes_c_through_19[offset_from_c];
    use TeamAttribute::*;
    match attr {
        Games             => TeamAttributeValue::Byte(byte_at(0x00)),
        AverageAttendance => TeamAttributeValue::Byte(byte_at(0x02)),
        Ratio             => TeamAttributeValue::Byte(byte_at(0x03)),
        HomeForm          => TeamAttributeValue::Byte(byte_at(0x05)),
        AwayForm          => TeamAttributeValue::Byte(byte_at(0x06)),
        GamesWonInRow     => TeamAttributeValue::Byte(byte_at(0x07)),
        GamesLostInRow    => TeamAttributeValue::Byte(byte_at(0x08)),
        GamesWithoutLoss  => TeamAttributeValue::Byte(byte_at(0x09)),
        GamesWithoutWin   => TeamAttributeValue::Byte(byte_at(0x0A)),
        Goals             => TeamAttributeValue::Byte(byte_at(0x0B)),
        Conceded          => TeamAttributeValue::Byte(byte_at(0x0C)),
        Penalties         => TeamAttributeValue::Byte(byte_at(0x0D)),
        GoalsFromCorners => {
            if !comp_reputation_ok { return TeamAttributeValue::Sentinel; }
            let comp_val = comp_attr1_value;
            let s_1d_i32 = s_1d as i32;
            if comp_val * k1 <= int_29 as f64
                && s_1d > 0x14
                && (s_1d > 500
                    || (int_29 + ((int_29 >> 31) & 3)) >> 2 < s_1d_i32)
            {
                TeamAttributeValue::Ratio(
                    (s_1f as f64 / s_1d as f64) * scale_960
                )
            } else {
                TeamAttributeValue::Sentinel
            }
        }
        GoalsFromIfks => {
            if !comp_reputation_ok { return TeamAttributeValue::Sentinel; }
            if comp_attr1_value * k1 <= int_29 as f64 && int_29 > 0x95 && byte_at(0x00) != 0 {
                TeamAttributeValue::Ratio(s_21 as f64 / byte_at(0x00) as f64)
            } else {
                TeamAttributeValue::Sentinel
            }
        }
        GoalsFromDfks => {
            if !comp_reputation_ok { return TeamAttributeValue::Sentinel; }
            if comp_attr1_value * k1 <= int_29 as f64 && int_29 > 0x95 && byte_at(0x00) != 0 {
                TeamAttributeValue::Ratio(s_23 as f64 / byte_at(0x00) as f64)
            } else {
                TeamAttributeValue::Sentinel
            }
        }
        YellowCards => {
            if !comp_reputation_ok { return TeamAttributeValue::Sentinel; }
            let comp_val = comp_attr1_value;
            if comp_val * k1 <= int_29 as f64
                && (s_25 > 0x18
                    || (s_25 > 4 && (s_25 != 0) && (int_29 / (s_25 as i32) < 0x2e))
                    || (int_29 > 0x21c && s_25 > 10))
            {
                TeamAttributeValue::Ratio(
                    (s_27 as f64 / s_25 as f64) * scale_960
                )
            } else {
                TeamAttributeValue::Sentinel
            }
        }
        RedCards => {
            // Case 0x11: recurse for GamesRatio-like check.
            if !comp_reputation_ok { return TeamAttributeValue::Sentinel; }
            let comp_val = comp_attr1_value;
            if comp_val * k2 <= /* recursive attr(1) result */ byte_at(0x00) as f64
                && byte_at(0x00) != 0
            {
                TeamAttributeValue::Ratio(s_1a as f64 / byte_at(0x00) as f64)
            } else {
                TeamAttributeValue::Sentinel
            }
        }
        GamesRatio => {
            if byte_at(0x00) != 0 {
                TeamAttributeValue::Ratio(byte_at(0x03) as f64 / byte_at(0x00) as f64)
            } else {
                TeamAttributeValue::Sentinel
            }
        }
    }
}

// =====================================================================
// FUN_004a8f10 — drain pending comp-news dirty flags
// =====================================================================

/// Direct port of `FUN_004a8f10(comp_record)`. Walks 22 dirty-flag
/// slots at `comp+0x50a..+0x562` and emits one news item per set flag,
/// then clears them all. `+0x562` is the master dirty flag.
///
/// Returns the list of slot indices (1..=22) that fired.
// GDI-REG: 004a8f10 PORTED_BEHAVIOURAL
pub fn drain_pending_comp_news(
    master_dirty: bool,
    per_slot_dirty: [bool; 22],
) -> Vec<u8> {
    if !master_dirty { return Vec::new(); }
    per_slot_dirty
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| if b { Some((i + 1) as u8) } else { None })
        .collect()
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- comp_slot_probe ----

    #[test]
    fn slot_1_is_ff_sentinel() {
        let c = comp_slot_probe(1);
        assert_eq!(c.family_code, 0xFF);
        assert_eq!(c.row_count, None);
        assert!(!c.out_of_range);
    }

    #[test]
    fn slot_family_boundaries() {
        // 2..=5 → family 6, rows 0x14
        for s in 2..=5 { assert_eq!(comp_slot_probe(s).family_code, 6); }
        // 6..=8 → family 3, rows 0x0b
        for s in 6..=8 { assert_eq!(comp_slot_probe(s).family_code, 3); }
        // 9..=10 → family 4, rows 0x0e
        for s in 9..=10 { assert_eq!(comp_slot_probe(s).family_code, 4); }
        // 11..=14 → family 5, rows 0x11
        for s in 11..=14 { assert_eq!(comp_slot_probe(s).family_code, 5); }
        // 15..=22 → family 7, rows 0x17
        for s in 15..=22 { assert_eq!(comp_slot_probe(s).family_code, 7); }
        assert_eq!(comp_slot_probe(2).row_count,  Some(0x14));
        assert_eq!(comp_slot_probe(6).row_count,  Some(0x0b));
        assert_eq!(comp_slot_probe(9).row_count,  Some(0x0e));
        assert_eq!(comp_slot_probe(11).row_count, Some(0x11));
        assert_eq!(comp_slot_probe(22).row_count, Some(0x17));
    }

    #[test]
    fn slot_out_of_range_returns_fallback() {
        let c = comp_slot_probe(0);
        assert_eq!(c.family_code, 3);
        assert!(c.out_of_range);
        let c = comp_slot_probe(23);
        assert!(c.out_of_range);
    }

    // ---- dispatch_comp_command_val9 ----

    #[test]
    fn val9_wrong_gate_returns_zero() {
        assert_eq!(dispatch_comp_command_val9(3, 5, 0, 42, 42), 0);
    }

    #[test]
    fn val9_id_matches_returns_zero() {
        assert_eq!(dispatch_comp_command_val9(3, 9, 0, 42, 42), 0);
    }

    #[test]
    fn val9_id_differs_returns_refresh_sentinel() {
        let r = dispatch_comp_command_val9(3, 9, 0, 42, 99);
        assert_eq!(r, crate::screen_batch28::HANDLED_REFRESH_SENTINEL);
    }

    #[test]
    fn val9_negative_slot_uses_fallback() {
        // fallback == 9, slot < 0 → id substitutes 0; field must ==0 to match.
        assert_eq!(dispatch_comp_command_val9(-1, 0, 9, 999, 0), 0);
        assert_eq!(
            dispatch_comp_command_val9(-1, 0, 9, 999, 42),
            crate::screen_batch28::HANDLED_REFRESH_SENTINEL,
        );
    }

    // ---- dispatch_comp_loader ----

    #[test]
    fn loader_gate_two_ok() {
        assert_eq!(dispatch_comp_loader(0, 2, 0, true), CompLoaderOutcome::LoaderPathOk);
    }
    #[test]
    fn loader_gate_two_error() {
        assert_eq!(dispatch_comp_loader(0, 2, 0, false), CompLoaderOutcome::LoaderPathError);
    }
    #[test]
    fn loader_gate_one_toggles_flags() {
        assert_eq!(dispatch_comp_loader(0, 1, 0, false), CompLoaderOutcome::ToggleFlagsPath);
    }
    #[test]
    fn loader_other_falls_through() {
        assert_eq!(dispatch_comp_loader(0, 3, 0, false), CompLoaderOutcome::FallThroughShared);
        assert_eq!(dispatch_comp_loader(0, 0, 0, false), CompLoaderOutcome::FallThroughShared);
    }

    // ---- extract_fixture_teams ----

    #[test]
    fn extract_null_input_errors() {
        let r = extract_fixture_teams(true, 0, false, false, 0, 0, false, None, None);
        assert_eq!(r, ExtractFixtureTeamsResult::NullInput);
    }
    #[test]
    fn extract_negative_comp_id_not_resolved() {
        let r = extract_fixture_teams(false, -1, true, false, 0, 100, true, Some(1), Some(2));
        assert_eq!(r, ExtractFixtureTeamsResult::NotResolved);
    }
    #[test]
    fn extract_reserves_guard_blocks() {
        let r = extract_fixture_teams(false, 5, true, true, 0, 100, true, Some(1), Some(2));
        assert_eq!(r, ExtractFixtureTeamsResult::NotResolved);
    }
    #[test]
    fn extract_success() {
        let r = extract_fixture_teams(false, 5, true, false, 0, 500, true, Some(11), Some(22));
        assert_eq!(r, ExtractFixtureTeamsResult::Resolved { comp: 500, home: 11, away: 22 });
    }

    // ---- launch_fixture_view ----

    #[test]
    fn fixture_view_id_zero_returns_none() {
        assert!(launch_fixture_view(0, ExtractFixtureTeamsResult::NullInput).is_none());
    }
    #[test]
    fn fixture_view_id_under_9999_returns_none() {
        // The exe's guard is `9999 < |id|`; so |id| == 9999 is BELOW threshold → None.
        let ok = ExtractFixtureTeamsResult::Resolved { comp: 1, home: 2, away: 3 };
        assert!(launch_fixture_view(9999, ok.clone()).is_none());
        assert!(launch_fixture_view(-9999, ok).is_none());
    }
    #[test]
    fn fixture_view_seeds_state_when_extraction_ok() {
        let ok = ExtractFixtureTeamsResult::Resolved { comp: 100, home: 200, away: 300 };
        let seed = launch_fixture_view(10000, ok).unwrap();
        assert_eq!(seed.field_0_comp, 100);
        assert_eq!(seed.field_1_home, 200);
        assert_eq!(seed.field_2_away, 300);
        assert_eq!(seed.field_3_zero, 0);
        assert_eq!(seed.field_4_zero, 0);
    }

    // ---- launchers with trivial seed ----

    #[test]
    fn uefa_and_fifa_launchers_have_expected_seeds() {
        assert_eq!(launch_uefa_coefs_screen(), (0, 0));
        assert_eq!(launch_fifa_rankings_screen(), (0, 0, 1, 1));
    }

    // ---- stat column names ----

    #[test]
    fn team_stat_column_endpoints_and_boundaries() {
        assert_eq!(team_stat_column_name(1),  Some("Games"));
        assert_eq!(team_stat_column_name(17), Some("Red Cards"));
        assert_eq!(team_stat_column_name(0),  None);
        assert_eq!(team_stat_column_name(18), None);
        assert_eq!(team_stat_column_name(10), Some("Goals"));
    }

    #[test]
    fn player_stat_column_endpoints_and_boundaries() {
        assert_eq!(player_stat_column_name(1),  Some("Appearances"));
        assert_eq!(player_stat_column_name(17), Some("Average Rating"));
        assert_eq!(player_stat_column_name(0),  None);
        assert_eq!(player_stat_column_name(18), None);
        assert_eq!(player_stat_column_name(12), Some("Man of Match"));
    }

    // ---- team attribute reader ----

    #[test]
    fn team_attr_byte_cases_read_correct_offset() {
        let b: [u8; 14] = [10, 0, 20, 30, 0, 40, 50, 60, 70, 80, 90, 100, 110, 120];
        let r = |a| read_team_attribute_value(
            a, b, 0, 0, 0, 0, 0, 0, 0, 0, false, 0.0, 2.0, 2.0, 100.0,
        );
        assert_eq!(r(TeamAttribute::Games),             TeamAttributeValue::Byte(10));
        assert_eq!(r(TeamAttribute::AverageAttendance), TeamAttributeValue::Byte(20));
        assert_eq!(r(TeamAttribute::Ratio),             TeamAttributeValue::Byte(30));
        assert_eq!(r(TeamAttribute::HomeForm),          TeamAttributeValue::Byte(40));
        assert_eq!(r(TeamAttribute::AwayForm),          TeamAttributeValue::Byte(50));
        assert_eq!(r(TeamAttribute::Penalties),         TeamAttributeValue::Byte(120));
    }

    #[test]
    fn team_attr_games_ratio_no_gate() {
        // GamesRatio has NO reputation gate — just needs byte_at(0x0c) != 0.
        let mut b: [u8; 14] = [0; 14];
        b[0x00] = 5;
        b[0x03] = 20;
        let r = read_team_attribute_value(
            TeamAttribute::GamesRatio, b, 0, 0, 0, 0, 0, 0, 0, 0, false, 0.0, 2.0, 2.0, 100.0,
        );
        assert_eq!(r, TeamAttributeValue::Ratio(20.0 / 5.0));
    }

    #[test]
    fn team_attr_games_ratio_divide_by_zero_returns_sentinel() {
        let b: [u8; 14] = [0; 14];   // Games = 0
        let r = read_team_attribute_value(
            TeamAttribute::GamesRatio, b, 0, 0, 0, 0, 0, 0, 0, 0, false, 0.0, 2.0, 2.0, 100.0,
        );
        assert_eq!(r, TeamAttributeValue::Sentinel);
    }

    #[test]
    fn team_attr_gated_case_falls_through_without_comp_rep() {
        let b: [u8; 14] = [1; 14];
        // GoalsFromCorners without comp_reputation_ok → sentinel regardless.
        let r = read_team_attribute_value(
            TeamAttribute::GoalsFromCorners, b, 0, 21, 5, 0, 0, 0, 0, 200,
            false /* comp_reputation_ok */, 0.0, 2.0, 2.0, 100.0,
        );
        assert_eq!(r, TeamAttributeValue::Sentinel);
    }

    #[test]
    fn team_attr_goals_from_corners_passes_gate() {
        let b: [u8; 14] = [1; 14];
        // Set: comp_val * k1 = 0.5 * 2 = 1.0 ≤ 200 (int_29); s_1d=21 > 0x14; 21 <= 500 but
        // (200 + 0) >> 2 = 50, 50 < 21 → FALSE, so gate should still fail.
        // Adjust: s_1d = 100 (>0x14 && >(200+0)>>2 = 50 → 100 > 50 → TRUE)
        let r = read_team_attribute_value(
            TeamAttribute::GoalsFromCorners, b, 0, 100, 10, 0, 0, 0, 0, 200,
            true, 0.5, 2.0, 2.0, 100.0,
        );
        // Expected: (s_1f / s_1d) * scale_960 = (10/100) * 100 = 10.0
        match r {
            TeamAttributeValue::Ratio(v) => assert!((v - 10.0).abs() < 1e-9),
            _ => panic!("expected Ratio, got {r:?}"),
        }
    }

    // ---- drain_pending_comp_news ----

    #[test]
    fn drain_master_off_returns_empty() {
        let dirty = [true; 22];
        assert!(drain_pending_comp_news(false, dirty).is_empty());
    }

    #[test]
    fn drain_emits_only_set_slots() {
        let mut dirty = [false; 22];
        dirty[0]  = true;   // slot 1
        dirty[5]  = true;   // slot 6
        dirty[21] = true;   // slot 22
        assert_eq!(drain_pending_comp_news(true, dirty), vec![1, 6, 22]);
    }

    #[test]
    fn drain_all_flags_emits_all_22_slots() {
        let dirty = [true; 22];
        assert_eq!(drain_pending_comp_news(true, dirty), (1u8..=22u8).collect::<Vec<_>>());
    }
}
