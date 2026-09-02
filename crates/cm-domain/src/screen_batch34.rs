//! Batch 34: `FUN_004ae9e0` + season-record news-emit metadata.
//!
//! - `FUN_004ae9e0` → `two_short_equals`: 2-line equality check on a
//!   packed short-pair (used by [`FUN_004ac150`] cases 13-14 to test
//!   whether a streak is "still active" for the current match).
//! - `FUN_004ac150` — 1380-line news-headline emitter for the 22
//!   season-record slots. Rather than port the full body (which would
//!   drag in FUN_004b5120 news-list-add, FUN_004b5360 news-link, and
//!   ~24 sprintf format string constants), this batch lands a typed
//!   metadata catalogue: each slot's field-offset schema, headline
//!   template, and entity-link topology. The port can be completed
//!   once the news-primitive scaffolding lands.

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_004ae9e0 — packed-short-pair equality
// =====================================================================

/// Direct port of `FUN_004ae9e0(pair_ptr, packed_u32)`.
///
/// The exe compares:
/// - `packed_u32` high 16 bits == `pair_ptr[1]`
/// - `packed_u32` low 16 bits == `pair_ptr[0]`
///
/// Returns 1 iff BOTH match, else 0. Used to test whether two
/// TCMDate-shaped `{day, year}` pairs are the same date, and by
/// [`SeasonRecordNewsSlot::MostGamesWithoutLosing/Winning`] to gate
/// the "still active" trailer on a streak headline.
pub fn two_short_equals(pair: [i16; 2], packed: u32) -> bool {
    let lo = (packed & 0xFFFF) as i16;
    let hi = ((packed >> 16) & 0xFFFF) as i16;
    pair[0] == lo && pair[1] == hi
}

// =====================================================================
// FUN_004ac150 — season-record news-headline metadata
// =====================================================================

/// The 22 news-emit slots `FUN_004ac150` handles, one per
/// [`crate::screen_batch31::SeasonRecordSlot`]. Discriminant values
/// are the `param_2` switch-cases in the exe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SeasonRecordNewsSlot {
    MostTimesWinner       = 0x01,
    MostTeamPoints        = 0x02,
    MostTeamGoals         = 0x03,
    MostTeamConceded      = 0x04,
    WorstTeamDiscipline   = 0x05,
    HighestAttendance     = 0x06,
    LowestAttendance      = 0x07,
    HighestAvgAttendance  = 0x08,
    BiggestWin            = 0x09,
    HighestScoringGame    = 0x0A,
    MostGamesWonInRow     = 0x0B,
    MostGamesLostInRow    = 0x0C,
    MostGamesWithoutLoss  = 0x0D,   // Uses two_short_equals for "active streak" trailer
    MostGamesWithoutWin   = 0x0E,   // Uses two_short_equals for "active streak" trailer
    TopGoalscorer         = 0x0F,
    MostGoalsInMatch      = 0x10,
    MostAssists           = 0x11,
    HighestAverageRating  = 0x12,
    MostManOfMatch        = 0x13,
    WorstDiscipline       = 0x14,
    YoungestPlayer        = 0x15,
    OldestPlayer          = 0x16,
}

/// Descriptor: what a slot's record layout looks like + which
/// entities the emitted news gets linked to.
///
/// - `guard_offset`: the record field that gates "has data" — if
///   `*(record + guard_offset)` reads as `-1` (or `< 0` for i32
///   fields), no news is emitted.
/// - `guard_is_i32`: `true` if the guard field is an i32 (some slots
///   guard on the char-width discipline count, most on int).
/// - `nation_ref_offset`: which record field holds the team's nation
///   pointer (for FUN_004b5360 nation-link on the news article).
///   `None` means the slot emits no per-nation link.
/// - `team_ref_offset`: which record field holds the team pointer for
///   the primary team news-link. `None` for slots that don't
///   reference a team.
/// - `secondary_team_offset`: for slots that reference TWO teams
///   (e.g. attendance highs where home + away both link).
/// - `player_ref_offset`: for player-scoped slots.
/// - `template_name`: the shipped format-string constant this slot
///   emits (from the exe's `s_<_d___...` string block).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeasonRecordNewsSchema {
    pub guard_offset:          usize,
    pub guard_is_i32:           bool,
    pub team_ref_offset:        Option<usize>,
    pub secondary_team_offset:  Option<usize>,
    pub nation_ref_offset:      Option<usize>,
    pub player_ref_offset:      Option<usize>,
    pub template_name:          &'static str,
    pub uses_active_streak_gate: bool,
}

/// Metadata table for all 22 slots, verified against `FUN_004ac150`
/// case-by-case. Offsets are from the news-emit record base (the
/// `param_1` pointer the exe walks).
pub const NEWS_SCHEMAS: [(SeasonRecordNewsSlot, SeasonRecordNewsSchema); 22] = [
    (SeasonRecordNewsSlot::MostTimesWinner, SeasonRecordNewsSchema {
        guard_offset: 0x00, guard_is_i32: false,
        team_ref_offset: Some(0x01), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: None,
        template_name: "<team> - <count> wins",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::MostTeamPoints, SeasonRecordNewsSchema {
        guard_offset: 0x05, guard_is_i32: false,   // short
        team_ref_offset: Some(0x07), secondary_team_offset: None,
        nation_ref_offset: Some(0x0b), player_ref_offset: None,
        template_name: "<points> points - <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::MostTeamGoals, SeasonRecordNewsSchema {
        guard_offset: 0x0D, guard_is_i32: false,
        team_ref_offset: Some(0x0F), secondary_team_offset: None,
        nation_ref_offset: Some(0x13), player_ref_offset: None,
        template_name: "<goals> goals - <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::MostTeamConceded, SeasonRecordNewsSchema {
        guard_offset: 0x15, guard_is_i32: false,
        team_ref_offset: Some(0x17), secondary_team_offset: None,
        nation_ref_offset: Some(0x1b), player_ref_offset: None,
        template_name: "<goals> conceded - <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::WorstTeamDiscipline, SeasonRecordNewsSchema {
        guard_offset: 0x1D, guard_is_i32: false,
        team_ref_offset: Some(0x1F), secondary_team_offset: None,
        nation_ref_offset: Some(0x23), player_ref_offset: None,
        template_name: "<yellows> yellows, <reds> reds - <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::HighestAttendance, SeasonRecordNewsSchema {
        guard_offset: 0x25, guard_is_i32: true,
        team_ref_offset: Some(0x3A), secondary_team_offset: Some(0x3E),
        nation_ref_offset: Some(0x31), player_ref_offset: None,
        template_name: "<attendance> attendance - <team> vs <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::LowestAttendance, SeasonRecordNewsSchema {
        guard_offset: 0x48, guard_is_i32: true,
        team_ref_offset: Some(0x5D), secondary_team_offset: Some(0x61),
        nation_ref_offset: Some(0x54), player_ref_offset: None,
        template_name: "<attendance> attendance - <team> vs <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::HighestAvgAttendance, SeasonRecordNewsSchema {
        guard_offset: 0x6B, guard_is_i32: true,
        team_ref_offset: Some(0x6F), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: None,
        template_name: "<avg> avg attendance - <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::BiggestWin, SeasonRecordNewsSchema {
        guard_offset: 0x7D, guard_is_i32: true,
        team_ref_offset: Some(0x86), secondary_team_offset: Some(0x8A),
        nation_ref_offset: Some(0x7D), player_ref_offset: None,
        template_name: "<score> - <home> vs <away>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::HighestScoringGame, SeasonRecordNewsSchema {
        guard_offset: 0x9C, guard_is_i32: true,
        team_ref_offset: Some(0xA5), secondary_team_offset: Some(0xA9),
        nation_ref_offset: Some(0x9C), player_ref_offset: None,
        template_name: "<score> - <home> vs <away>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::MostGamesWonInRow, SeasonRecordNewsSchema {
        guard_offset: 0xB3, guard_is_i32: true,
        team_ref_offset: Some(0xC7), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: None,
        template_name: "<count> games won in row - <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::MostGamesLostInRow, SeasonRecordNewsSchema {
        guard_offset: 0xCB, guard_is_i32: true,
        team_ref_offset: Some(0xDF), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: None,
        template_name: "<count> games lost in row - <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::MostGamesWithoutLoss, SeasonRecordNewsSchema {
        guard_offset: 0xE3, guard_is_i32: true,
        team_ref_offset: Some(0xF7), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: None,
        template_name: "<count> games without losing - <team>",
        uses_active_streak_gate: true,   // uses two_short_equals
    }),
    (SeasonRecordNewsSlot::MostGamesWithoutWin, SeasonRecordNewsSchema {
        guard_offset: 0xFB, guard_is_i32: true,
        team_ref_offset: Some(0x10F), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: None,
        template_name: "<count> games without winning - <team>",
        uses_active_streak_gate: true,   // uses two_short_equals
    }),
    (SeasonRecordNewsSlot::TopGoalscorer, SeasonRecordNewsSchema {
        guard_offset: 0x113, guard_is_i32: false,
        team_ref_offset: Some(0x12C), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: Some(0x120),
        template_name: "<player> <team> - <goals> goals",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::MostGoalsInMatch, SeasonRecordNewsSchema {
        guard_offset: 0x132, guard_is_i32: false,
        team_ref_offset: Some(0x15C), secondary_team_offset: Some(0x160),
        nation_ref_offset: Some(0x153), player_ref_offset: Some(0x13F),
        template_name: "<goals> goals - <player> in <home> vs <away>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::MostAssists, SeasonRecordNewsSchema {
        guard_offset: 0x16A, guard_is_i32: false,
        team_ref_offset: Some(0x183), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: Some(0x177),
        template_name: "<assists> assists - <player> <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::HighestAverageRating, SeasonRecordNewsSchema {
        guard_offset: 0x189, guard_is_i32: false,   // float compared to _DAT_00956948 (0.0)
        team_ref_offset: Some(0x1A6), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: Some(0x199),
        template_name: "<%.2f> avg rating - <player> <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::MostManOfMatch, SeasonRecordNewsSchema {
        guard_offset: 0x1AC, guard_is_i32: false,
        team_ref_offset: Some(0x1C5), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: Some(0x1B9),
        template_name: "<count> man of match - <player> <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::WorstDiscipline, SeasonRecordNewsSchema {
        guard_offset: 0x1CB, guard_is_i32: false,
        team_ref_offset: Some(0x1E5), secondary_team_offset: None,
        nation_ref_offset: None, player_ref_offset: Some(0x1D9),
        template_name: "<yellows>/<reds> - <player> <team>",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::YoungestPlayer, SeasonRecordNewsSchema {
        guard_offset: 0x213, guard_is_i32: true,
        team_ref_offset: Some(0x21C), secondary_team_offset: Some(0x220),
        nation_ref_offset: Some(0x213), player_ref_offset: Some(0x1FF),
        template_name: "<player> <team> - age",
        uses_active_streak_gate: false,
    }),
    (SeasonRecordNewsSlot::OldestPlayer, SeasonRecordNewsSchema {
        guard_offset: 0x252, guard_is_i32: true,
        team_ref_offset: Some(0x25B), secondary_team_offset: Some(0x25F),
        nation_ref_offset: Some(0x252), player_ref_offset: Some(0x23E),
        template_name: "<player> <team> - age",
        uses_active_streak_gate: false,
    }),
];

/// Look up the news-emit schema for one slot.
pub fn news_schema_for(slot: SeasonRecordNewsSlot) -> SeasonRecordNewsSchema {
    NEWS_SCHEMAS
        .iter()
        .find(|(s, _)| *s == slot)
        .map(|(_, sc)| *sc)
        .expect("every SeasonRecordNewsSlot is in NEWS_SCHEMAS")
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- two_short_equals ----

    #[test]
    fn two_short_equals_both_halves_match() {
        assert!(two_short_equals([0x1234, 0x5678], 0x5678_1234));
    }
    #[test]
    fn two_short_equals_low_only_matches() {
        assert!(!two_short_equals([0x1234, 0x5678], 0x0000_1234));
    }
    #[test]
    fn two_short_equals_high_only_matches() {
        assert!(!two_short_equals([0x1234, 0x5678], 0x5678_0000));
    }
    #[test]
    fn two_short_equals_negatives_via_sign_extension() {
        // -1 as i16 = 0xFFFF; packed = 0xFFFF_FFFF
        assert!(two_short_equals([-1, -1], 0xFFFF_FFFF));
    }
    #[test]
    fn two_short_equals_zero() {
        assert!(two_short_equals([0, 0], 0));
    }

    // ---- news schemas ----

    #[test]
    fn schemas_cover_every_slot_uniquely() {
        // 22 entries, 22 unique slots.
        assert_eq!(NEWS_SCHEMAS.len(), 22);
        let mut slots: Vec<u8> = NEWS_SCHEMAS.iter().map(|(s, _)| *s as u8).collect();
        slots.sort();
        slots.dedup();
        assert_eq!(slots.len(), 22);
        // Discriminants form 1..=0x16 (case values in the exe).
        assert_eq!(slots, (1u8..=0x16).collect::<Vec<u8>>());
    }

    #[test]
    fn active_streak_gate_only_fires_for_wo_loss_and_wo_win() {
        for (slot, sch) in NEWS_SCHEMAS.iter() {
            let expected = matches!(slot,
                SeasonRecordNewsSlot::MostGamesWithoutLoss
                | SeasonRecordNewsSlot::MostGamesWithoutWin,
            );
            assert_eq!(sch.uses_active_streak_gate, expected,
                "slot {slot:?} streak gate mismatch");
        }
    }

    #[test]
    fn attendance_slots_reference_two_teams_each() {
        assert!(news_schema_for(SeasonRecordNewsSlot::HighestAttendance)
            .secondary_team_offset.is_some());
        assert!(news_schema_for(SeasonRecordNewsSlot::LowestAttendance)
            .secondary_team_offset.is_some());
        // But HighestAvg (single-team stat) does NOT.
        assert!(news_schema_for(SeasonRecordNewsSlot::HighestAvgAttendance)
            .secondary_team_offset.is_none());
    }

    #[test]
    fn player_slots_all_carry_a_player_ref() {
        for slot in [
            SeasonRecordNewsSlot::TopGoalscorer,
            SeasonRecordNewsSlot::MostGoalsInMatch,
            SeasonRecordNewsSlot::MostAssists,
            SeasonRecordNewsSlot::HighestAverageRating,
            SeasonRecordNewsSlot::MostManOfMatch,
            SeasonRecordNewsSlot::WorstDiscipline,
            SeasonRecordNewsSlot::YoungestPlayer,
            SeasonRecordNewsSlot::OldestPlayer,
        ] {
            assert!(news_schema_for(slot).player_ref_offset.is_some(),
                "slot {slot:?} should carry a player ref");
        }
    }

    #[test]
    fn team_only_slots_have_no_player_ref() {
        for slot in [
            SeasonRecordNewsSlot::MostTimesWinner,
            SeasonRecordNewsSlot::MostTeamPoints,
            SeasonRecordNewsSlot::MostGamesWonInRow,
            SeasonRecordNewsSlot::MostGamesWithoutLoss,
        ] {
            assert!(news_schema_for(slot).player_ref_offset.is_none(),
                "slot {slot:?} should have no player ref");
        }
    }
}
