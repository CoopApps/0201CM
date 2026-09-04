//! Injury + suspension substrate.
//!
//! Ports `physio.cpp` + `discipline.cpp`. Handles:
//! * Injuries triggered by match events (from `match_engine::MinuteEvent::Injury`)
//! * Recovery per-day (injuries have a duration; player unavailable until it expires)
//! * Card-based suspensions (5 yellows in a season = 1-match ban; red = 3-match ban)

use serde::{Deserialize, Serialize};

/// Injury severity → recovery days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjurySeverity {
    Knock,    // 1-3 days
    Minor,    // 1-2 weeks
    Moderate, // 3-8 weeks
    Major,    // 2-6 months
    Career,   // ends career
}

impl InjurySeverity {
    pub fn recovery_days(self) -> u16 {
        match self {
            Self::Knock => 3,
            Self::Minor => 10,
            Self::Moderate => 35,
            Self::Major => 100,
            Self::Career => u16::MAX,
        }
    }

    /// Return-to-training threshold: number of `days_remaining` at or below
    /// which the player rejoins general training (still not match-fit). The
    /// exe's rehab model gates full training the same way — knock-tier
    /// injuries stay in training throughout; heavier ones only when the
    /// rehab clock winds down to near-recovery. Pure derived helper; no
    /// formula fabricated beyond the "training available in last ~30 % of
    /// window" convention this port has used consistently for gates.
    pub fn training_threshold_days(self) -> u16 {
        match self {
            Self::Knock => u16::MAX,               // always trainable
            Self::Minor => 3,                       // last 3d of a 10d recovery
            Self::Moderate => 10,                   // last 10d of a 35d
            Self::Major => 30,                      // last 30d of ~100
            Self::Career => 0,                      // never
        }
    }
}

/// A completed injury retained for career-history purposes. Written by
/// [`InjuryBook::add_injury`] on onset and never removed by the daily tick
/// — kept even after full recovery so the profile screen and season stats
/// can list what a player has been through.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricInjury {
    pub player_id: u32,
    pub severity: InjurySeverity,
    /// Days out at onset — matches [`InjurySeverity::recovery_days`] at add
    /// time; the actual served time may be less if a worse injury later
    /// replaces it in the active list, but the entry stays as-recorded.
    pub days_out: u16,
    /// In-game day the injury was registered (from
    /// [`RuntimeSaveGame::elapsed_days`]). 0 when the caller doesn't stamp.
    pub game_day: u32,
}

/// A live injury record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Injury {
    pub player_id: u32,
    pub severity: InjurySeverity,
    /// Days remaining until fully recovered.
    pub days_remaining: u16,
}

/// A live suspension record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suspension {
    pub player_id: u32,
    /// Matches still to serve.
    pub matches_remaining: u8,
}

/// Discipline tally for one player across a season.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DisciplineTally {
    pub player_id: u32,
    pub yellow_cards_this_season: u8,
    pub red_cards_this_season: u8,
}

/// Full physio/discipline state, keyed per player.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InjuryBook {
    pub injuries: Vec<Injury>,
    pub suspensions: Vec<Suspension>,
    pub discipline: Vec<DisciplineTally>,
    /// Append-only career-history log. Written by `add_injury` and
    /// [`InjuryBook::add_injury_stamped`]; never cleared. See
    /// [`HistoricInjury`].
    #[serde(default)]
    pub history: Vec<HistoricInjury>,
}

impl InjuryBook {
    pub fn new() -> Self { Self::default() }

    /// Register an injury. Overwrites any existing injury on the same
    /// player (worse-severity wins if the new one is longer). Appends a
    /// [`HistoricInjury`] entry for career stats.
    pub fn add_injury(&mut self, player_id: u32, severity: InjurySeverity) {
        self.add_injury_stamped(player_id, severity, 0);
    }

    /// Like [`add_injury`] but stamps the entry with the current
    /// `game_day` (typically [`RuntimeSaveGame::elapsed_days`]) so the
    /// career history is ordered on the timeline.
    pub fn add_injury_stamped(&mut self, player_id: u32, severity: InjurySeverity, game_day: u32) {
        let days = severity.recovery_days();
        if let Some(existing) = self.injuries.iter_mut().find(|i| i.player_id == player_id) {
            if days > existing.days_remaining {
                existing.severity = severity;
                existing.days_remaining = days;
            }
        } else {
            self.injuries.push(Injury { player_id, severity, days_remaining: days });
        }
        self.history.push(HistoricInjury { player_id, severity, days_out: days, game_day });
    }

    /// Record a yellow card. Every 5 yellows in a season = 1-match ban.
    pub fn record_yellow(&mut self, player_id: u32) {
        let row = self.discipline.iter_mut().find(|r| r.player_id == player_id);
        let tally = match row {
            Some(r) => { r.yellow_cards_this_season += 1; r.yellow_cards_this_season },
            None => {
                self.discipline.push(DisciplineTally {
                    player_id, yellow_cards_this_season: 1, red_cards_this_season: 0,
                });
                1
            }
        };
        if tally % 5 == 0 {
            self.add_suspension(player_id, 1);
        }
    }

    /// Record a red card = 3-match ban.
    pub fn record_red(&mut self, player_id: u32) {
        if let Some(r) = self.discipline.iter_mut().find(|r| r.player_id == player_id) {
            r.red_cards_this_season += 1;
        } else {
            self.discipline.push(DisciplineTally {
                player_id, yellow_cards_this_season: 0, red_cards_this_season: 1,
            });
        }
        self.add_suspension(player_id, 3);
    }

    fn add_suspension(&mut self, player_id: u32, matches: u8) {
        if let Some(s) = self.suspensions.iter_mut().find(|s| s.player_id == player_id) {
            s.matches_remaining = s.matches_remaining.saturating_add(matches);
        } else {
            self.suspensions.push(Suspension { player_id, matches_remaining: matches });
        }
    }

    /// Daily tick — advance injury recovery. Removes cleared injuries.
    pub fn advance_day(&mut self) {
        self.injuries.retain_mut(|i| {
            if i.days_remaining == u16::MAX { return true; }  // career-ender
            i.days_remaining = i.days_remaining.saturating_sub(1);
            i.days_remaining > 0
        });
    }

    /// A player played a match — deduct one from their suspension (if
    /// any) and remove the row if cleared.
    pub fn served_match(&mut self, player_id: u32) {
        self.suspensions.retain_mut(|s| {
            if s.player_id == player_id {
                s.matches_remaining = s.matches_remaining.saturating_sub(1);
                s.matches_remaining > 0
            } else {
                true
            }
        });
    }

    pub fn is_available(&self, player_id: u32) -> bool {
        !self.injuries.iter().any(|i| i.player_id == player_id)
        && !self.suspensions.iter().any(|s| s.player_id == player_id
                                            && s.matches_remaining > 0)
    }

    /// Fraction of recovery completed, `0.0` at onset → `1.0` at return.
    /// Returns `None` for uninjured players and `Some(0.0)` for
    /// career-ending injuries (never advances).
    pub fn rehab_progress(&self, player_id: u32) -> Option<f32> {
        let inj = self.injuries.iter().find(|i| i.player_id == player_id)?;
        if inj.severity == InjurySeverity::Career {
            return Some(0.0);
        }
        let total = inj.severity.recovery_days().max(1) as f32;
        let remaining = inj.days_remaining as f32;
        Some(((total - remaining) / total).clamp(0.0, 1.0))
    }

    /// Whether an injured player has returned to general training. True for
    /// uninjured players and for those whose remaining days are within the
    /// severity's [`InjurySeverity::training_threshold_days`] window.
    pub fn is_training_available(&self, player_id: u32) -> bool {
        match self.injuries.iter().find(|i| i.player_id == player_id) {
            None => true,
            Some(inj) => inj.days_remaining <= inj.severity.training_threshold_days(),
        }
    }

    /// Injury-history entries for one player, chronological (oldest first).
    pub fn history_for(&self, player_id: u32) -> Vec<&HistoricInjury> {
        self.history.iter().filter(|h| h.player_id == player_id).collect()
    }

    /// Weekly aggregate — count of active injuries + suspensions after
    /// [`InjuryBook::advance_day`] has run through the week. Cheap
    /// snapshot for the reporting layer (news / dashboard). The real
    /// per-day countdown is unchanged; this is a read-only summary and
    /// does NOT mutate state.
    pub fn weekly_summary(&self) -> WeeklyInjurySummary {
        let mut minor = 0u32;
        let mut moderate = 0u32;
        let mut major = 0u32;
        let mut knock = 0u32;
        let mut career = 0u32;
        for i in &self.injuries {
            match i.severity {
                InjurySeverity::Knock => knock += 1,
                InjurySeverity::Minor => minor += 1,
                InjurySeverity::Moderate => moderate += 1,
                InjurySeverity::Major => major += 1,
                InjurySeverity::Career => career += 1,
            }
        }
        WeeklyInjurySummary {
            active_injuries: self.injuries.len() as u32,
            active_suspensions: self.suspensions.iter()
                .filter(|s| s.matches_remaining > 0).count() as u32,
            knock, minor, moderate, major, career,
        }
    }
}

/// Snapshot of the injury book at a point in time. Emitted by
/// [`InjuryBook::weekly_summary`] for reporting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeeklyInjurySummary {
    pub active_injuries: u32,
    pub active_suspensions: u32,
    pub knock: u32,
    pub minor: u32,
    pub moderate: u32,
    pub major: u32,
    pub career: u32,
}

impl InjuryBook {

    /// Season-end reset — clears yellow/red tallies. Suspensions stay.
    pub fn reset_season(&mut self) {
        for r in &mut self.discipline {
            r.yellow_cards_this_season = 0;
            r.red_cards_this_season = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injury_recovers_day_by_day() {
        let mut b = InjuryBook::new();
        b.add_injury(1, InjurySeverity::Knock);
        for _ in 0..2 { b.advance_day(); }
        assert!(!b.is_available(1));
        b.advance_day();
        assert!(b.is_available(1));
    }

    #[test]
    fn fifth_yellow_triggers_suspension() {
        let mut b = InjuryBook::new();
        for _ in 0..4 { b.record_yellow(1); }
        assert!(b.is_available(1));
        b.record_yellow(1);
        assert!(!b.is_available(1));
    }

    #[test]
    fn red_card_is_three_match_ban() {
        let mut b = InjuryBook::new();
        b.record_red(1);
        assert!(!b.is_available(1));
        b.served_match(1);
        b.served_match(1);
        assert!(!b.is_available(1));
        b.served_match(1);
        assert!(b.is_available(1));
    }

    #[test]
    fn worse_injury_replaces_lighter_one() {
        let mut b = InjuryBook::new();
        b.add_injury(1, InjurySeverity::Knock);
        b.add_injury(1, InjurySeverity::Major);
        assert_eq!(b.injuries[0].severity, InjurySeverity::Major);
    }

    #[test]
    fn career_ending_never_recovers() {
        let mut b = InjuryBook::new();
        b.add_injury(1, InjurySeverity::Career);
        for _ in 0..10_000 { b.advance_day(); }
        assert!(!b.is_available(1));
    }

    #[test]
    fn season_reset_clears_card_tallies() {
        let mut b = InjuryBook::new();
        for _ in 0..4 { b.record_yellow(1); }
        b.reset_season();
        assert_eq!(b.discipline[0].yellow_cards_this_season, 0);
    }

    #[test]
    fn add_injury_appends_history_entry() {
        let mut b = InjuryBook::new();
        b.add_injury_stamped(42, InjurySeverity::Moderate, 100);
        assert_eq!(b.history.len(), 1);
        assert_eq!(b.history[0].player_id, 42);
        assert_eq!(b.history[0].severity, InjurySeverity::Moderate);
        assert_eq!(b.history[0].days_out, 35);
        assert_eq!(b.history[0].game_day, 100);
    }

    #[test]
    fn history_persists_after_recovery() {
        let mut b = InjuryBook::new();
        b.add_injury(1, InjurySeverity::Knock);
        for _ in 0..5 { b.advance_day(); }
        assert!(b.is_available(1));
        assert_eq!(b.history.len(), 1);
    }

    #[test]
    fn rehab_progress_monotone_from_zero_to_one() {
        let mut b = InjuryBook::new();
        b.add_injury(1, InjurySeverity::Minor);          // 10d
        assert_eq!(b.rehab_progress(1), Some(0.0));
        b.advance_day(); b.advance_day(); b.advance_day(); b.advance_day(); b.advance_day();
        // 5/10 done — halfway.
        let p = b.rehab_progress(1).unwrap();
        assert!((0.45..=0.55).contains(&p), "midpoint ~= 0.5, got {p}");
        for _ in 0..5 { b.advance_day(); }
        assert!(b.is_available(1));
        assert_eq!(b.rehab_progress(1), None);
    }

    #[test]
    fn rehab_progress_none_when_uninjured() {
        let b = InjuryBook::new();
        assert_eq!(b.rehab_progress(999), None);
    }

    #[test]
    fn career_ender_is_never_at_rehab_end() {
        let mut b = InjuryBook::new();
        b.add_injury(7, InjurySeverity::Career);
        assert_eq!(b.rehab_progress(7), Some(0.0));
        for _ in 0..500 { b.advance_day(); }
        assert_eq!(b.rehab_progress(7), Some(0.0));
    }

    #[test]
    fn training_gate_opens_near_recovery() {
        let mut b = InjuryBook::new();
        // Minor injury: 10d out, training threshold = 3.
        b.add_injury(1, InjurySeverity::Minor);
        assert!(!b.is_training_available(1));
        for _ in 0..7 { b.advance_day(); }  // 3d remaining
        assert!(b.is_training_available(1));
    }

    #[test]
    fn training_gate_always_open_for_knock() {
        let mut b = InjuryBook::new();
        b.add_injury(1, InjurySeverity::Knock);
        assert!(b.is_training_available(1));
    }

    #[test]
    fn training_gate_never_opens_for_career_ender() {
        let mut b = InjuryBook::new();
        b.add_injury(1, InjurySeverity::Career);
        assert!(!b.is_training_available(1));
    }

    #[test]
    fn training_gate_open_for_uninjured() {
        let b = InjuryBook::new();
        assert!(b.is_training_available(1234));
    }

    #[test]
    fn weekly_summary_counts_by_severity() {
        let mut b = InjuryBook::new();
        b.add_injury(1, InjurySeverity::Knock);
        b.add_injury(2, InjurySeverity::Minor);
        b.add_injury(3, InjurySeverity::Moderate);
        b.add_injury(4, InjurySeverity::Major);
        b.add_injury(5, InjurySeverity::Career);
        b.record_red(6);
        let s = b.weekly_summary();
        assert_eq!(s.active_injuries, 5);
        assert_eq!(s.active_suspensions, 1);
        assert_eq!(s.knock, 1);
        assert_eq!(s.minor, 1);
        assert_eq!(s.moderate, 1);
        assert_eq!(s.major, 1);
        assert_eq!(s.career, 1);
    }

    #[test]
    fn history_for_filters_to_player() {
        let mut b = InjuryBook::new();
        b.add_injury_stamped(1, InjurySeverity::Knock, 1);
        b.add_injury_stamped(2, InjurySeverity::Minor, 2);
        b.add_injury_stamped(1, InjurySeverity::Major, 3);
        assert_eq!(b.history_for(1).len(), 2);
        assert_eq!(b.history_for(2).len(), 1);
        assert_eq!(b.history_for(99).len(), 0);
    }
}
