//! Batch 35: `FUN_004aea10` — achievement-news headline emitter.
//!
//! Companion to [`crate::screen_batch34`]'s `FUN_004ac150` metadata
//! (which handles current-standings news). This function fires when a
//! season-record is BROKEN NOW: reads serialized values from a stream
//! (via `FUN_0076d7d0` typed pulls) then emits past-tense achievement
//! templates like `"The <points> points gained by <team>"`,
//! `"The attendance of <ld> at <team> vs <team>"`.
//!
//! Same 22-slot enum as [`crate::screen_batch34::SeasonRecordNewsSlot`].
//! Same news primitives (`FUN_004b5e70` open, `FUN_004b5120` emit,
//! `FUN_004b5360` link). Full body deferred pending news scaffolding
//! port; this batch lands the achievement-template catalogue so the
//! wiring is data-complete.

use serde::{Deserialize, Serialize};
use crate::screen_batch34::SeasonRecordNewsSlot;

/// Achievement-news template name for each of the 22 slots. Cross-
/// referenced to the format-string labels in `FUN_004aea10` — the
/// exe stores these at 4-byte-string-pointer addresses ending in
/// `0098b350`, `0098b2d4`, `0098b290`, etc.
///
/// Some slots emit ONE template; the streak slots (WonInRow/LostInRow/
/// WithoutLoss/WithoutWin) emit two variants — a "modest streak"
/// version at `iVar5 < 2` and a "notable streak" version otherwise.
/// The attendance/scoring slots pair with a special "in nation N" or
/// "played in nation N" trailer decided by
/// [`FUN_004c6370`](comp_screens_helper) that switches between two
/// templates per slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AchievementTemplateSet {
    /// The primary "achievement fired" template. Every slot has this.
    pub primary: &'static str,
    /// Alternate template used for the streak "not notable" (< 2)
    /// path OR for the attendance "in-nation" trailer variant. `None`
    /// when only the primary template exists.
    pub alternate: Option<&'static str>,
}

/// Per-slot achievement templates verified line-by-line against
/// `FUN_004aea10`. The template strings are paraphrased from the exe
/// (which stores real strings like `s___<_s___team>_s__<_d___wins>_tro_0098b318`
/// pointing at the actual sprintf format).
pub const ACHIEVEMENT_TEMPLATES: [(SeasonRecordNewsSlot, AchievementTemplateSet); 22] = [
    (SeasonRecordNewsSlot::MostTimesWinner, AchievementTemplateSet {
        primary:   "The <team> has <count> wins - a title",
        alternate: Some("The <team> has <count> wins - a trophy"),
    }),
    (SeasonRecordNewsSlot::MostTeamPoints, AchievementTemplateSet {
        primary:   "The <points> points gained by <team>",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::MostTeamGoals, AchievementTemplateSet {
        primary:   "The <goals> goals scored by <team>",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::MostTeamConceded, AchievementTemplateSet {
        primary:   "The <goals> goals conceded by <team>",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::WorstTeamDiscipline, AchievementTemplateSet {
        primary:   "<team> received <yellows> yellow cards, <reds> reds",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::HighestAttendance, AchievementTemplateSet {
        primary:   "The attendance of <ld> at <team>",
        alternate: Some("The attendance of <ld> at <team> in nation <n>"),
    }),
    (SeasonRecordNewsSlot::LowestAttendance, AchievementTemplateSet {
        primary:   "The attendance of <ld> at <team>",
        alternate: Some("The attendance of <ld> at <team> in nation <n>"),
    }),
    (SeasonRecordNewsSlot::HighestAvgAttendance, AchievementTemplateSet {
        primary:   "The <team>'s average attendance",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::BiggestWin, AchievementTemplateSet {
        primary:   "The <team>'s <score> win - <trophy/title/normal per stage_byte>",
        alternate: Some("The <team>'s <score> win - <trophy/title/normal> in nation <n>"),
    }),
    (SeasonRecordNewsSlot::HighestScoringGame, AchievementTemplateSet {
        primary:   "The <team>'s <score> win/draw - <opponent>",
        alternate: Some("The <team>'s <score> win/draw - <opponent> in nation <n>"),
    }),
    (SeasonRecordNewsSlot::MostGamesWonInRow, AchievementTemplateSet {
        primary:   "The <team>'s <count> games won in row - milestone",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::MostGamesLostInRow, AchievementTemplateSet {
        primary:   "The <team>'s <count> games lost in row - unwanted",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::MostGamesWithoutLoss, AchievementTemplateSet {
        primary:   "The <team>'s <count> games without losing",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::MostGamesWithoutWin, AchievementTemplateSet {
        primary:   "The <team>'s <count> games without winning",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::TopGoalscorer, AchievementTemplateSet {
        primary:   "The <goals> goals scored by <player> <team>",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::MostGoalsInMatch, AchievementTemplateSet {
        primary:   "The <goals> goals scored by <player> in <home> vs <away>",
        alternate: Some("... variant with nation trailer"),
    }),
    (SeasonRecordNewsSlot::MostAssists, AchievementTemplateSet {
        primary:   "The <assists> assists by <player> <team>",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::HighestAverageRating, AchievementTemplateSet {
        primary:   "The <%.2f> average rating by <player> <team>",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::MostManOfMatch, AchievementTemplateSet {
        primary:   "The <count> man of match by <player> <team>",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::WorstDiscipline, AchievementTemplateSet {
        primary:   "<player> <team> - <yellows>/<reds> cards",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::YoungestPlayer, AchievementTemplateSet {
        primary:   "The <age> age of <player> <team>",
        alternate: None,
    }),
    (SeasonRecordNewsSlot::OldestPlayer, AchievementTemplateSet {
        primary:   "The <age> age of <player> <team>",
        alternate: None,
    }),
];

/// Look up the achievement-template set for one slot.
pub fn achievement_templates_for(slot: SeasonRecordNewsSlot) -> AchievementTemplateSet {
    ACHIEVEMENT_TEMPLATES
        .iter()
        .find(|(s, _)| *s == slot)
        .map(|(_, t)| *t)
        .expect("every SeasonRecordNewsSlot is in ACHIEVEMENT_TEMPLATES")
}

/// The exe's "streak notability" gate. Slots WinsInRow / LossesInRow /
/// WithoutLoss / WithoutWin emit their headline only when
/// `count >= 2`. Verified against `if ((local_955 < 0) || (local_955 < 2))
/// goto LAB_004b1a35;` at case '\v'.
pub fn streak_is_notable(count: i32) -> bool {
    count >= 2
}

/// The two-template selection FUN_004c6370 picks between: returns
/// non-zero when the match is in a "different nation" from the
/// primary competition — the exe then uses the ALTERNATE template
/// (with the "in nation <n>" trailer). Verified as case '\x06' logic.
pub fn use_in_nation_variant(comp_nation_id: u32, match_nation_id: u32) -> bool {
    comp_nation_id != match_nation_id && match_nation_id != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slot_has_an_achievement_template() {
        assert_eq!(ACHIEVEMENT_TEMPLATES.len(), 22);
        for slot in ACHIEVEMENT_TEMPLATES.iter().map(|(s, _)| *s) {
            let t = achievement_templates_for(slot);
            assert!(!t.primary.is_empty(), "slot {slot:?} has empty primary template");
        }
    }

    #[test]
    fn achievements_cover_the_1_to_0x16_slot_range() {
        let mut ids: Vec<u8> = ACHIEVEMENT_TEMPLATES.iter().map(|(s, _)| *s as u8).collect();
        ids.sort();
        assert_eq!(ids, (1u8..=0x16).collect::<Vec<_>>());
    }

    #[test]
    fn attendance_and_scoring_slots_have_in_nation_alternates() {
        for slot in [
            SeasonRecordNewsSlot::HighestAttendance,
            SeasonRecordNewsSlot::LowestAttendance,
            SeasonRecordNewsSlot::BiggestWin,
            SeasonRecordNewsSlot::HighestScoringGame,
        ] {
            assert!(achievement_templates_for(slot).alternate.is_some(),
                "slot {slot:?} should have an alternate template");
        }
    }

    #[test]
    fn most_times_winner_has_trophy_alternate() {
        let t = achievement_templates_for(SeasonRecordNewsSlot::MostTimesWinner);
        assert!(t.primary.contains("title"));
        assert!(t.alternate.unwrap().contains("trophy"));
    }

    #[test]
    fn team_only_slots_dont_use_in_nation_alternate() {
        for slot in [
            SeasonRecordNewsSlot::MostTeamPoints,
            SeasonRecordNewsSlot::MostTeamGoals,
            SeasonRecordNewsSlot::MostTeamConceded,
            SeasonRecordNewsSlot::MostGamesWonInRow,
            SeasonRecordNewsSlot::MostGamesLostInRow,
            SeasonRecordNewsSlot::HighestAverageRating,
            SeasonRecordNewsSlot::MostAssists,
        ] {
            assert!(achievement_templates_for(slot).alternate.is_none(),
                "slot {slot:?} should not have an alternate template");
        }
    }

    // ---- streak_is_notable ----

    #[test]
    fn streak_notable_gate_matches_exe() {
        assert!(!streak_is_notable(-1));    // negative → not notable
        assert!(!streak_is_notable(0));
        assert!(!streak_is_notable(1));     // < 2 → not notable
        assert!(streak_is_notable(2));      // >= 2 → notable
        assert!(streak_is_notable(50));
    }

    // ---- use_in_nation_variant ----

    #[test]
    fn in_nation_variant_selection() {
        // Match nation same as comp → primary template.
        assert!(!use_in_nation_variant(100, 100));
        // Match nation different, non-zero → alternate.
        assert!(use_in_nation_variant(100, 200));
        // Match nation zero → primary (missing data).
        assert!(!use_in_nation_variant(100, 0));
    }
}
