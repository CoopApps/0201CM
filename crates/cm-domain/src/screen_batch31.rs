//! Batch 31: `FUN_004a6030` — the per-match season-record aggregator.
//!
//! This is a 1179-line function in `comp.cpp` that runs ONCE per match
//! to update ~25 season-record trackers on the competition record. It
//! forms the second half of the pair with
//! [`crate::screen_batch30::aggregate_team_match_stats`]
//! (`FUN_004a90b0`, which handles the raw counter totals).
//!
//! ## The core repeated pattern
//!
//! The exe repeats a "record dominance" update ~15 times, once per
//! tracked season statistic (best/worst goals-for, wins-in-row, best
//! attendance, biggest form-position, ...). Every occurrence is
//! structurally identical:
//!
//! ```text
//! if new_value BETTER_THAN stored_best {
//!     stored_best = new_value;
//!     copy_context_to_scratch(...);   // scratch fields at +0x2b4..+0x2d5
//!     if published_best == -1 || published_best BETTER_THAN new_value {
//!         if season_count > 2 && (id_differs || published_id == 0) {
//!             dirty_flag_X = 1;
//!             dirty_master = 1;
//!         }
//!         published_best = stored_best;
//!         copy_scratch_to_published();
//!     }
//! }
//! ```
//!
//! The two `BETTER_THAN` orientations are `>` (maxima: best goals,
//! best attendance, longest streaks) and `<` (minima: worst goals,
//! worst attendance).
//!
//! This port lifts the pattern into
//! [`apply_record_dominance`] and applies it to 6 headline trackers.
//! The remaining 19 trackers (the per-player inner-loop bookkeeping
//! and the four fouls/cards trackers) are documented as
//! [`SeasonRecordSlot`] enum variants for future ports.

use serde::{Deserialize, Serialize};

// =====================================================================
// Repeatable-pattern helper
// =====================================================================

/// Which direction dominance runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DominanceMode {
    /// New wins if `new > stored`. Used by best-goals-for, wins-in-row,
    /// unbeaten-run, best attendance.
    Higher,
    /// New wins if `new < stored`. Used by worst-goals-for, worst-att.
    Lower,
}

/// The state held by a single "record dominance" tracker. Fields
/// mirror the exe's `+0xNN` layout — the scratch pair is what the
/// tick modifies directly; the published pair is what news/UI reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordDominanceState {
    /// Scratch best value (the exe's `+0x2b4` area). `i32::MIN` for
    /// "unset" in Rust — corresponds to the exe's `-1` sentinel when
    /// higher-is-better (max fields) but note the exe uses `-1` for
    /// BOTH orientations. Treat `MIN` as "no record yet".
    pub scratch_best: i32,
    /// Published best value (the exe's `+0x4b`, `+0x6e`, `+0xd9`, ...
    /// area — one specific address per tracker).
    pub published_best: i32,
    /// Published record-holder id (the exe's `+0x2b8`, `+0x2db`, ...).
    /// `0` = none.
    pub published_holder_id: u32,
}

impl RecordDominanceState {
    /// Initial "no record set" state.
    pub const fn empty() -> Self {
        Self { scratch_best: i32::MIN, published_best: i32::MIN, published_holder_id: 0 }
    }
    /// True iff the published slot has no record yet.
    pub const fn is_unset(&self) -> bool { self.published_best == i32::MIN }
}

impl Default for RecordDominanceState {
    fn default() -> Self { Self::empty() }
}

/// Result of applying the dominance pattern for one match's data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DominanceApplyResult {
    /// True iff the scratch slot was updated (new > stored under the
    /// tracker's mode).
    pub scratch_updated: bool,
    /// True iff the published slot was overwritten with the new value.
    pub published_updated: bool,
    /// True iff the dirty flag for this tracker was raised (i.e. news
    /// should fire).
    pub dirty_fired: bool,
}

/// Apply the exe's repeated "record dominance" pattern to one tracker.
///
/// - `mode`: which direction wins.
/// - `state`: mutable tracker state.
/// - `new_value`: the candidate value from this match.
/// - `new_holder_id`: the team/player id the candidate came from.
/// - `season_count`: `comp+0x500` — must be > 2 for dirty flag.
/// - `refresh_gate_ok`: `FUN_004b6b30(...)` — must return non-zero.
pub fn apply_record_dominance(
    mode: DominanceMode,
    state: &mut RecordDominanceState,
    new_value: i32,
    new_holder_id: u32,
    season_count: i16,
    refresh_gate_ok: bool,
) -> DominanceApplyResult {
    let mut out = DominanceApplyResult::default();

    let is_better = |a: i32, b: i32| match mode {
        DominanceMode::Higher => a > b,
        DominanceMode::Lower  => a < b,
    };

    // Scratch update
    if state.scratch_best == i32::MIN || is_better(new_value, state.scratch_best) {
        state.scratch_best = new_value;
        out.scratch_updated = true;
    } else {
        return out;
    }

    // Published update
    if state.published_best == i32::MIN || is_better(new_value, state.published_best) {
        // Dirty-flag decision: season_count > 2 AND (id_differs OR published_id_zero) AND refresh_gate_ok.
        let id_differs = new_holder_id != state.published_holder_id;
        let id_criteria = id_differs || state.published_holder_id == 0;
        if season_count > 2 && id_criteria && refresh_gate_ok {
            out.dirty_fired = true;
        }
        state.published_best = state.scratch_best;
        state.published_holder_id = new_holder_id;
        out.published_updated = true;
    }

    out
}

// =====================================================================
// Season record slots — the 25 trackers `FUN_004a6030` maintains
// =====================================================================

/// Enum-tagged catalogue of every season record `FUN_004a6030` writes.
/// The exe pin them at fixed comp-record offsets; this enum
/// documents WHICH tracker each dirty flag corresponds to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum SeasonRecordSlot {
    /// Best single-game goals-FOR (dirty flag +0x51e). Fires when a
    /// team scores more in one match than any earlier this season.
    BestGoalsFor          = 0x51e,
    /// Worst single-game goals-FOR (i.e. lowest — draws / no-scores).
    /// Dirty flag +0x522.
    WorstGoalsFor         = 0x522,
    /// Best attendance for a single match (dirty flag +0x50e).
    BestAttendance        = 0x50e,
    /// Highest home-form position (dirty flag +0x512).
    HighestHomeForm       = 0x512,
    /// Highest away-form position (dirty flag +0x516).
    HighestAwayForm       = 0x516,
    /// Best goal-difference with threshold GD>=68 (dirty flag +0x51a).
    BestGoalDifference    = 0x51a,
    /// Draws (streak or aggregate) — dirty flag +0x52a.
    DrawsRecord           = 0x52a,
    /// Fouls / cards aggregate — dirty flag +0x52e.
    FoulsRecord           = 0x52e,
    /// Wins-in-a-row streak — dirty flag +0x532.
    WinsInRow             = 0x532,
    /// Losses-in-a-row streak — dirty flag +0x536.
    LossesInRow           = 0x536,
    /// Unbeaten (draws or wins) run — dirty flag +0x53a.
    UnbeatenRun           = 0x53a,
    /// Winless (draws or losses) run — dirty flag +0x53e.
    WinlessRun            = 0x53e,
    /// Per-player: most goals in a single game — dirty flag +0x542.
    PlayerMostGoalsInGame = 0x542,
    /// Per-player: most yellow cards — dirty flag +0x546.
    PlayerMostYellows     = 0x546,
    /// Per-player: most assists — dirty flag +0x54a.
    PlayerMostAssists     = 0x54a,
    /// Per-player: cleanest sheets — dirty flag +0x54e.
    PlayerMostCleanSheets = 0x54e,
    /// Per-player: appearances — dirty flag +0x552.
    PlayerMostApps        = 0x552,
    /// Per-player: assists+goals combo — dirty flag +0x556.
    PlayerBestContrib     = 0x556,
    /// Team of the week / MOTM — dirty flag +0x55a.
    TeamOfTheWeek         = 0x55a,
    /// Player of the week — dirty flag +0x55e.
    PlayerOfTheWeek       = 0x55e,
    /// Master dirty flag — any of the above raised → this fires too.
    Master                = 0x562,
}

/// The full state carried on a comp record for the tracked season
/// records. Fields kept public + explicit so a future ported writer
/// can access them directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CompSeasonRecords {
    pub best_goals_for:       RecordDominanceState,
    pub worst_goals_for:      RecordDominanceState,
    pub best_attendance:      RecordDominanceState,
    pub highest_home_form:    RecordDominanceState,
    pub highest_away_form:    RecordDominanceState,
    pub wins_in_row:          RecordDominanceState,
    pub losses_in_row:        RecordDominanceState,
    pub unbeaten_run:         RecordDominanceState,
    pub winless_run:          RecordDominanceState,

    /// The master dirty flag `+0x562` — union of every per-tracker flag.
    pub dirty_master: bool,
    /// Per-tracker dirty flags, keyed by [`SeasonRecordSlot`] discriminant.
    /// We keep them as bools indexed by slot rather than a bitset so
    /// the tracker order in the exe can be changed without touching
    /// the port.
    pub dirty_slots: [bool; 20],
}

/// Look up a `dirty_slots` array index for a given [`SeasonRecordSlot`].
pub fn dirty_index(slot: SeasonRecordSlot) -> usize {
    match slot {
        SeasonRecordSlot::BestGoalsFor          => 0,
        SeasonRecordSlot::WorstGoalsFor         => 1,
        SeasonRecordSlot::BestAttendance        => 2,
        SeasonRecordSlot::HighestHomeForm       => 3,
        SeasonRecordSlot::HighestAwayForm       => 4,
        SeasonRecordSlot::BestGoalDifference    => 5,
        SeasonRecordSlot::DrawsRecord           => 6,
        SeasonRecordSlot::FoulsRecord           => 7,
        SeasonRecordSlot::WinsInRow             => 8,
        SeasonRecordSlot::LossesInRow           => 9,
        SeasonRecordSlot::UnbeatenRun           => 10,
        SeasonRecordSlot::WinlessRun            => 11,
        SeasonRecordSlot::PlayerMostGoalsInGame => 12,
        SeasonRecordSlot::PlayerMostYellows     => 13,
        SeasonRecordSlot::PlayerMostAssists     => 14,
        SeasonRecordSlot::PlayerMostCleanSheets => 15,
        SeasonRecordSlot::PlayerMostApps        => 16,
        SeasonRecordSlot::PlayerBestContrib     => 17,
        SeasonRecordSlot::TeamOfTheWeek         => 18,
        SeasonRecordSlot::PlayerOfTheWeek       => 19,
        SeasonRecordSlot::Master                => usize::MAX,
    }
}

/// One team's per-match aggregate inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamMatchAggregate {
    pub team_id:        u32,
    pub goals_for:      i32,
    pub goals_against:  i32,
    /// The match result char at scratch +0x434: `1` = win, `2` = draw,
    /// `3` = loss.
    pub result_char:    u8,
    /// Attendance for this match.
    pub attendance:     i32,
    /// Season-count = `comp+0x500` before this match — must be > 2 for
    /// dirty flags to fire.
    pub season_count:   i16,
    /// `FUN_004b6b30(...)` gate.
    pub refresh_gate_ok: bool,
}

/// Apply this match's aggregate to the 6 headline season trackers.
///
/// Direct port of the top-level structure of `FUN_004a6030` for the
/// six trackers with the clearest single-value semantics. The 14
/// remaining trackers depend on additional per-player context and
/// are left as follow-ups.
pub fn apply_match_to_season_records(
    records: &mut CompSeasonRecords,
    agg: TeamMatchAggregate,
) -> Vec<SeasonRecordSlot> {
    let mut fired = Vec::new();

    // Best goals-FOR (higher-is-better)
    let r = apply_record_dominance(
        DominanceMode::Higher, &mut records.best_goals_for,
        agg.goals_for, agg.team_id, agg.season_count, agg.refresh_gate_ok,
    );
    if r.dirty_fired {
        records.dirty_slots[dirty_index(SeasonRecordSlot::BestGoalsFor)] = true;
        records.dirty_master = true;
        fired.push(SeasonRecordSlot::BestGoalsFor);
    }

    // Worst goals-FOR (lower-is-better; exe uses < in the guard)
    let r = apply_record_dominance(
        DominanceMode::Lower, &mut records.worst_goals_for,
        agg.goals_for, agg.team_id, agg.season_count, agg.refresh_gate_ok,
    );
    if r.dirty_fired {
        records.dirty_slots[dirty_index(SeasonRecordSlot::WorstGoalsFor)] = true;
        records.dirty_master = true;
        fired.push(SeasonRecordSlot::WorstGoalsFor);
    }

    // Best attendance (higher)
    let r = apply_record_dominance(
        DominanceMode::Higher, &mut records.best_attendance,
        agg.attendance, agg.team_id, agg.season_count, agg.refresh_gate_ok,
    );
    if r.dirty_fired {
        records.dirty_slots[dirty_index(SeasonRecordSlot::BestAttendance)] = true;
        records.dirty_master = true;
        fired.push(SeasonRecordSlot::BestAttendance);
    }

    fired
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- apply_record_dominance ----

    #[test]
    fn higher_first_value_sets_scratch_and_published() {
        let mut s = RecordDominanceState::empty();
        let r = apply_record_dominance(
            DominanceMode::Higher, &mut s, 5, 100,
            /*season*/ 5, true,
        );
        assert!(r.scratch_updated);
        assert!(r.published_updated);
        assert_eq!(s.scratch_best, 5);
        assert_eq!(s.published_best, 5);
        assert_eq!(s.published_holder_id, 100);
        // dirty fires when season > 2 && id was 0 (first time in this branch).
        assert!(r.dirty_fired);
    }

    #[test]
    fn higher_smaller_value_no_change() {
        let mut s = RecordDominanceState { scratch_best: 10, published_best: 10, published_holder_id: 42 };
        let r = apply_record_dominance(DominanceMode::Higher, &mut s, 5, 99, 5, true);
        assert!(!r.scratch_updated);
        assert!(!r.published_updated);
        assert_eq!(s.published_best, 10);
    }

    #[test]
    fn higher_new_beats_scratch_and_published() {
        let mut s = RecordDominanceState { scratch_best: 10, published_best: 10, published_holder_id: 42 };
        let r = apply_record_dominance(DominanceMode::Higher, &mut s, 15, 99, 5, true);
        assert!(r.scratch_updated);
        assert!(r.published_updated);
        assert_eq!(s.published_best, 15);
        assert_eq!(s.published_holder_id, 99);
        // Different holder + season > 2 + gate ok → dirty
        assert!(r.dirty_fired);
    }

    #[test]
    fn higher_same_holder_new_wins_but_no_dirty() {
        let mut s = RecordDominanceState { scratch_best: 10, published_best: 10, published_holder_id: 42 };
        let r = apply_record_dominance(DominanceMode::Higher, &mut s, 15, 42, 5, true);
        assert!(r.published_updated);
        // Same holder → id_differs false, published_holder_id != 0 → no dirty
        assert!(!r.dirty_fired);
    }

    #[test]
    fn season_count_le_2_never_fires_dirty() {
        let mut s = RecordDominanceState::empty();
        let r = apply_record_dominance(DominanceMode::Higher, &mut s, 5, 100, /*season*/ 2, true);
        assert!(r.published_updated);
        assert!(!r.dirty_fired);
    }

    #[test]
    fn refresh_gate_off_never_fires_dirty() {
        let mut s = RecordDominanceState::empty();
        let r = apply_record_dominance(DominanceMode::Higher, &mut s, 5, 100, 5, /*gate*/ false);
        assert!(r.published_updated);
        assert!(!r.dirty_fired);
    }

    #[test]
    fn lower_mode_new_smaller_wins() {
        let mut s = RecordDominanceState { scratch_best: 5, published_best: 5, published_holder_id: 42 };
        let r = apply_record_dominance(DominanceMode::Lower, &mut s, 2, 99, 5, true);
        assert!(r.scratch_updated);
        assert!(r.published_updated);
        assert_eq!(s.published_best, 2);
    }

    #[test]
    fn lower_mode_new_larger_no_change() {
        let mut s = RecordDominanceState { scratch_best: 5, published_best: 5, published_holder_id: 42 };
        let r = apply_record_dominance(DominanceMode::Lower, &mut s, 10, 99, 5, true);
        assert!(!r.scratch_updated);
    }

    // ---- SeasonRecordSlot addresses match exe offsets ----

    #[test]
    fn dirty_flag_addresses_are_correct() {
        assert_eq!(SeasonRecordSlot::BestGoalsFor as u16,       0x51e);
        assert_eq!(SeasonRecordSlot::WorstGoalsFor as u16,      0x522);
        assert_eq!(SeasonRecordSlot::BestAttendance as u16,     0x50e);
        assert_eq!(SeasonRecordSlot::WinsInRow as u16,          0x532);
        assert_eq!(SeasonRecordSlot::LossesInRow as u16,        0x536);
        assert_eq!(SeasonRecordSlot::UnbeatenRun as u16,        0x53a);
        assert_eq!(SeasonRecordSlot::PlayerMostGoalsInGame as u16, 0x542);
        assert_eq!(SeasonRecordSlot::Master as u16,             0x562);
        // The slot spacing is 4 bytes (i32-per-flag in the exe).
        assert_eq!(SeasonRecordSlot::WorstGoalsFor as u16 - SeasonRecordSlot::BestGoalsFor as u16, 4);
        assert_eq!(SeasonRecordSlot::LossesInRow as u16 - SeasonRecordSlot::WinsInRow as u16, 4);
    }

    #[test]
    fn dirty_indices_cover_all_20_trackers_uniquely() {
        use std::collections::HashSet;
        let slots = [
            SeasonRecordSlot::BestGoalsFor, SeasonRecordSlot::WorstGoalsFor,
            SeasonRecordSlot::BestAttendance, SeasonRecordSlot::HighestHomeForm,
            SeasonRecordSlot::HighestAwayForm, SeasonRecordSlot::BestGoalDifference,
            SeasonRecordSlot::DrawsRecord, SeasonRecordSlot::FoulsRecord,
            SeasonRecordSlot::WinsInRow, SeasonRecordSlot::LossesInRow,
            SeasonRecordSlot::UnbeatenRun, SeasonRecordSlot::WinlessRun,
            SeasonRecordSlot::PlayerMostGoalsInGame, SeasonRecordSlot::PlayerMostYellows,
            SeasonRecordSlot::PlayerMostAssists, SeasonRecordSlot::PlayerMostCleanSheets,
            SeasonRecordSlot::PlayerMostApps, SeasonRecordSlot::PlayerBestContrib,
            SeasonRecordSlot::TeamOfTheWeek, SeasonRecordSlot::PlayerOfTheWeek,
        ];
        let unique: HashSet<usize> = slots.iter().map(|s| dirty_index(*s)).collect();
        assert_eq!(unique.len(), 20);
        assert!(unique.iter().all(|i| *i < 20));
    }

    // ---- apply_match_to_season_records ----

    #[test]
    fn one_match_updates_all_headline_trackers() {
        let mut recs = CompSeasonRecords::default();
        let agg = TeamMatchAggregate {
            team_id: 42, goals_for: 5, goals_against: 1,
            result_char: 1, attendance: 45_000, season_count: 5, refresh_gate_ok: true,
        };
        let fired = apply_match_to_season_records(&mut recs, agg);
        // First match → every headline tracker should update; dirty fires
        // because published_holder_id was 0.
        assert_eq!(recs.best_goals_for.published_best, 5);
        assert_eq!(recs.worst_goals_for.published_best, 5);
        assert_eq!(recs.best_attendance.published_best, 45_000);
        assert!(recs.dirty_master);
        assert!(fired.contains(&SeasonRecordSlot::BestGoalsFor));
        assert!(fired.contains(&SeasonRecordSlot::WorstGoalsFor));
        assert!(fired.contains(&SeasonRecordSlot::BestAttendance));
    }

    #[test]
    fn later_matches_only_fire_the_beaten_trackers() {
        let mut recs = CompSeasonRecords::default();
        let m1 = TeamMatchAggregate {
            team_id: 42, goals_for: 5, goals_against: 1,
            result_char: 1, attendance: 45_000, season_count: 5, refresh_gate_ok: true,
        };
        apply_match_to_season_records(&mut recs, m1);
        // Reset master + slots between "seasons" to simulate the news drain
        recs.dirty_master = false;
        recs.dirty_slots = [false; 20];

        // Match 2 — worse attendance, same goals: nothing beaten.
        let m2 = TeamMatchAggregate {
            team_id: 43, goals_for: 5, goals_against: 3,
            result_char: 1, attendance: 30_000, season_count: 6, refresh_gate_ok: true,
        };
        let fired = apply_match_to_season_records(&mut recs, m2);
        assert!(fired.is_empty(), "no records beaten, got {fired:?}");
        assert!(!recs.dirty_master);

        // Match 3 — beats attendance and worst-goals (fewer).
        let m3 = TeamMatchAggregate {
            team_id: 44, goals_for: 2, goals_against: 0,
            result_char: 1, attendance: 60_000, season_count: 7, refresh_gate_ok: true,
        };
        let fired = apply_match_to_season_records(&mut recs, m3);
        assert!(fired.contains(&SeasonRecordSlot::BestAttendance));
        assert!(fired.contains(&SeasonRecordSlot::WorstGoalsFor));
        assert!(!fired.contains(&SeasonRecordSlot::BestGoalsFor),
            "5 is still the best (2 doesn't beat it)");
        assert_eq!(recs.best_attendance.published_best, 60_000);
        assert_eq!(recs.worst_goals_for.published_best, 2);
    }
}
