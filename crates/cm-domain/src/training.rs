//! Training + player development substrate.
//!
//! Ports `training_manager.cpp` + `training_schedule.cpp` + `training_edit_session.cpp`.
//! Every player has a weekly training schedule that grows their CA toward
//! their PA over time. Weekly tick applies the growth.
//!
//! # Growth model
//!
//! Per-week CA growth = `base_growth × schedule_weight × age_factor`.
//! * `base_growth` = 0.05 (100 weeks of full-intensity training = 5 CA points)
//! * `schedule_weight` in [0.5, 1.5] — light rest to heavy training
//! * `age_factor` — 1.0 for players under 21, 0.5 for 21-28, 0.1 for 29+
//!
//! CA never exceeds PA. Players over 30 also see a gradual DECLINE
//! (`decline_rate` = 0.02/week) when not training hard.

use serde::{Deserialize, Serialize};

use crate::player_rating::PlayerRatingBook;

/// A player's training schedule — the "focus" the manager sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainingSchedule {
    Rest,             // recovery — 0.5x growth
    Light,            // 0.75x
    Balanced,         // 1.0x (default)
    Intense,          // 1.25x — slight injury risk uplift
    OverTrain,        // 1.5x — real injury risk
}

impl Default for TrainingSchedule {
    fn default() -> Self { Self::Balanced }
}

impl TrainingSchedule {
    pub fn weight(self) -> f32 {
        match self {
            Self::Rest      => 0.5,
            Self::Light     => 0.75,
            Self::Balanced  => 1.0,
            Self::Intense   => 1.25,
            Self::OverTrain => 1.5,
        }
    }

    /// Per-week extra injury probability from over-training.
    pub fn injury_uplift(self) -> f32 {
        match self {
            Self::Rest | Self::Light | Self::Balanced => 0.0,
            Self::Intense => 0.002,
            Self::OverTrain => 0.008,
        }
    }
}

/// One player's per-week training assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PlayerTraining {
    pub player_id: u32,
    pub schedule: TrainingSchedule,
}

/// Fractional-CA accumulator so growth of 0.05 CA/week doesn't get lost
/// when we round back into the integer stored on `RatedPlayer.ca`. Keyed
/// by player id.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GrowthAccumulator {
    pub player_id: u32,
    /// Accumulated fractional CA that hasn't yet crossed a whole-point
    /// threshold. In `[-1.0, +1.0)`.
    pub fractional: f32,
}

/// Training state — per-player schedules + fractional growth accumulator.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrainingBook {
    pub schedules: Vec<PlayerTraining>,
    #[serde(default)]
    pub accumulators: Vec<GrowthAccumulator>,
    /// The faithful per-attribute development system (kill #TR) — real CM0102
    /// training (coach-gated, schedule-driven per-attribute growth/decline).
    /// The `apply_weekly_growth` CA-drift above is retained only as an interim
    /// CA continuity feed for the awards book until CA is recomputed from these
    /// attributes.
    #[serde(default)]
    pub development: crate::player_development::PlayerDevelopmentBook,
}

impl TrainingBook {
    pub fn new() -> Self { Self::default() }

    /// Set every player to Balanced training initially.
    pub fn seed_from_ratings(ratings: &PlayerRatingBook) -> Self {
        Self {
            schedules: ratings.players.iter()
                .map(|p| PlayerTraining { player_id: p.staff_id,
                                          schedule: TrainingSchedule::Balanced })
                .collect(),
            accumulators: Vec::new(),
            development: Default::default(),
        }
    }

    pub fn set_schedule(&mut self, player_id: u32, schedule: TrainingSchedule) {
        if let Some(row) = self.schedules.iter_mut().find(|r| r.player_id == player_id) {
            row.schedule = schedule;
        } else {
            self.schedules.push(PlayerTraining { player_id, schedule });
        }
    }

    pub fn schedule_for(&self, player_id: u32) -> TrainingSchedule {
        self.schedules.iter().find(|r| r.player_id == player_id)
            .map(|r| r.schedule).unwrap_or_default()
    }

    /// Weekly training pass. Applies CA growth (toward PA) or decline
    /// (past 30) to every player in the rating book. Growth is accumulated
    /// fractionally on `accumulators`; once a whole point crosses over,
    /// the CA integer moves and the fraction stays. Returns how many
    /// players saw a whole-CA change this tick.
    pub fn apply_weekly_growth(&mut self, ratings: &mut PlayerRatingBook) -> usize {
        let mut changed = 0;
        for p in &mut ratings.players {
            let schedule = self.schedules.iter().find(|r| r.player_id == p.staff_id)
                .map(|r| r.schedule).unwrap_or_default();
            let age_factor: f32 = if p.age_est < 21 { 1.0 }
                                  else if p.age_est <= 28 { 0.4 }
                                  else if p.age_est <= 32 { 0.1 }
                                  else { 0.03 };
            let base_growth = 0.06_f32;
            let mut delta = base_growth * schedule.weight() * age_factor;

            // Old players resting = active decline.
            if p.age_est >= 30 && schedule == TrainingSchedule::Rest {
                delta = -0.02;
            } else if (p.ca as i16) >= p.pa {
                delta = 0.0;
            }
            if delta == 0.0 { continue; }

            let acc = if let Some(a) = self.accumulators.iter_mut()
                .find(|a| a.player_id == p.staff_id)
            {
                a
            } else {
                self.accumulators.push(GrowthAccumulator {
                    player_id: p.staff_id, fractional: 0.0,
                });
                self.accumulators.last_mut().unwrap()
            };
            acc.fractional += delta;

            while acc.fractional >= 1.0 && (p.ca as i16) < p.pa {
                p.ca = (p.ca + 1).min(p.pa);
                acc.fractional -= 1.0;
                changed += 1;
            }
            while acc.fractional <= -1.0 && p.ca > 1 {
                p.ca -= 1;
                acc.fractional += 1.0;
                changed += 1;
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player_rating::RatedPlayer;

    fn mk(id: u32, ca: i16, pa: i16, age: u8) -> RatedPlayer {
        RatedPlayer { staff_id: id, club_id: Some(10), division_id: Some(24),
                      ca, pa, goals_est: 10, age_est: age, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] }
    }

    #[test]
    fn young_player_grows_faster_than_old() {
        let mut ratings = PlayerRatingBook { players: vec![
            mk(1, 100, 180, 18),   // young, room to grow
            mk(2, 100, 180, 28),   // older, same room
        ], ..Default::default()};
        let mut book = TrainingBook::seed_from_ratings(&ratings);
        for _ in 0..200 { book.apply_weekly_growth(&mut ratings); }
        assert!(ratings.players[0].ca > ratings.players[1].ca,
                "young player should have grown more");
    }

    #[test]
    fn ca_never_exceeds_pa() {
        let mut ratings = PlayerRatingBook { players: vec![
            mk(1, 195, 200, 20),
        ], ..Default::default()};
        let mut book = TrainingBook::seed_from_ratings(&ratings);
        for _ in 0..1000 { book.apply_weekly_growth(&mut ratings); }
        assert!(ratings.players[0].ca <= 200);
    }

    #[test]
    fn old_resting_player_declines() {
        let mut ratings = PlayerRatingBook { players: vec![mk(1, 150, 180, 34)], ..Default::default() };
        let mut book = TrainingBook::seed_from_ratings(&ratings);
        book.set_schedule(1, TrainingSchedule::Rest);
        let before = ratings.players[0].ca;
        for _ in 0..500 { book.apply_weekly_growth(&mut ratings); }
        assert!(ratings.players[0].ca < before);
    }

    #[test]
    fn intense_schedule_has_injury_uplift() {
        assert!(TrainingSchedule::Intense.injury_uplift()
                > TrainingSchedule::Balanced.injury_uplift());
        assert!(TrainingSchedule::OverTrain.injury_uplift()
                > TrainingSchedule::Intense.injury_uplift());
    }
}
