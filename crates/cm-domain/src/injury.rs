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
}

impl InjuryBook {
    pub fn new() -> Self { Self::default() }

    /// Register an injury. Overwrites any existing injury on the same
    /// player (worse-severity wins if the new one is longer).
    pub fn add_injury(&mut self, player_id: u32, severity: InjurySeverity) {
        let days = severity.recovery_days();
        if let Some(existing) = self.injuries.iter_mut().find(|i| i.player_id == player_id) {
            if days > existing.days_remaining {
                existing.severity = severity;
                existing.days_remaining = days;
            }
        } else {
            self.injuries.push(Injury { player_id, severity, days_remaining: days });
        }
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
}
