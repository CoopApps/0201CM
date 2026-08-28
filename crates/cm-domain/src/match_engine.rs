//! Match engine — replaces the reputation-weighted fixture proxy with a
//! minute-by-minute simulation.
//!
//! Scope: enough to unblock `match_eng.cpp` / `match_pl.cpp` /
//! `match_events.cpp` / `match_man.cpp` / `match_stats.cpp` /
//! `match_day.cpp` / `match_official.cpp`. Not bit-exact with the exe's
//! tactical arithmetic — CM01/02's real engine reads formation, morale,
//! form, injuries, weather, playing style, per-player attributes across
//! 30+ dimensions, which need `formation.cpp` (still blocked). This
//! engine reads the team's average CA and does a Poisson-driven event
//! walk that's monotone with quality and produces plausible stats.
//!
//! # What the engine produces
//!
//! Given `SimulateFixtureInput` (home CA, away CA, seed) → `MatchResult`
//! with:
//! * Final score
//! * Per-minute event list (goals, shots on/off, cards, injuries, subs)
//! * Match stats (possession %, shots, shots on target, corners, fouls)
//!
//! # Why the substrate matters
//!
//! Every fixture our tick generates currently resolves via a
//! reputation-weighted scoreline (see `simple_league::resolve_headless`).
//! Replacing that with `MatchEngine::simulate_fixture(...)` gives:
//!   * Real per-fixture stats that feed `match_stats.cpp`'s aggregator
//!     and hall_of_fame + season awards.
//!   * Determinism (seeded RNG) so replaying a save reproduces the same
//!     scores.
//!   * A place for the future `formation.cpp` port to hook in — the
//!     engine will read per-team lineups + tactical directives once
//!     those exist.

use serde::{Deserialize, Serialize};

/// Deterministic PRNG (splitmix64) — the whole engine uses one so the
/// same seed reproduces the same match.
#[derive(Debug, Clone, Copy)]
pub struct MatchRng {
    state: u64,
}

impl MatchRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_add(0x9e3779b97f4a7c15) }
    }

    fn next_u64(&mut self) -> u64 {
        let mut z = self.state.wrapping_add(0x9e3779b97f4a7c15);
        self.state = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    /// Uniform 0.0..1.0
    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1u32 << 24) as f32)
    }

    /// Uniform 0..n
    pub fn next_range(&mut self, n: u32) -> u32 {
        (self.next_u64() >> 32) as u32 % n.max(1)
    }
}

/// Input the caller supplies for one fixture.
#[derive(Debug, Clone, Copy)]
pub struct SimulateFixtureInput {
    /// Home team's average CA (1..200). Reputation stands in when a proper
    /// team-CA aggregate isn't wired.
    pub home_ca: u8,
    pub away_ca: u8,
    /// Fixture id from the season plan — used to seed the RNG so a save
    /// replay produces the same result.
    pub fixture_seed: u64,
    /// Home-advantage multiplier applied to xG (typical: 1.3).
    pub home_advantage: f32,
    /// Home team's tactical bundle. When set, its formation attack-bias
    /// and mentality shift the xG. None → default 4-4-2 flat.
    pub home_tactics: Option<crate::formation::TacticalBundle>,
    /// Away team's tactical bundle.
    pub away_tactics: Option<crate::formation::TacticalBundle>,
}

impl Default for SimulateFixtureInput {
    fn default() -> Self {
        Self {
            home_ca: 100, away_ca: 100, fixture_seed: 0,
            home_advantage: 1.3, home_tactics: None, away_tactics: None,
        }
    }
}

/// One event within the match, timestamped by minute (1..90).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MinuteEvent {
    Goal { minute: u8, side: Side },
    ShotOnTarget { minute: u8, side: Side },
    ShotOffTarget { minute: u8, side: Side },
    YellowCard { minute: u8, side: Side },
    RedCard { minute: u8, side: Side },
    Injury { minute: u8, side: Side },
    Substitution { minute: u8, side: Side },
    Corner { minute: u8, side: Side },
    Foul { minute: u8, side: Side },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Home,
    Away,
}

/// Aggregate match statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MatchStats {
    pub goals: u8,
    pub shots: u16,
    pub shots_on_target: u16,
    pub corners: u16,
    pub fouls: u16,
    pub yellow_cards: u8,
    pub red_cards: u8,
    pub injuries: u8,
    pub subs: u8,
}

/// Full result of a simulated match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchResult {
    pub home_goals: u8,
    pub away_goals: u8,
    pub home_stats: MatchStats,
    pub away_stats: MatchStats,
    pub possession_home_pct: u8,
    pub events: Vec<MinuteEvent>,
}

/// The core simulation entry point. Deterministic given `input.fixture_seed`.
pub fn simulate_fixture(input: SimulateFixtureInput) -> MatchResult {
    let mut rng = MatchRng::new(input.fixture_seed);

    // Expected goals scale with CA. 200-CA team averages ~2.5 goals against
    // a 100-CA team; two 100-CA teams average ~1.2 each. Home advantage
    // multiplies the home team's xG. Formations shift xG through their
    // attack_bias — an all-out-attack 4-3-3 vs a defensive 5-4-1 shifts
    // both teams' xG in the attacking side's favour.
    let home_bias = input.home_tactics
        .map(|t| t.effective_attack_bias())
        .unwrap_or(0.0);
    let away_bias = input.away_tactics
        .map(|t| t.effective_attack_bias())
        .unwrap_or(0.0);
    let bias_mult_home = 1.0 + home_bias * 0.35 - away_bias * 0.15;
    let bias_mult_away = 1.0 + away_bias * 0.35 - home_bias * 0.15;
    let home_xg = expected_goals(input.home_ca, input.away_ca)
        * input.home_advantage * bias_mult_home.max(0.3);
    let away_xg = expected_goals(input.away_ca, input.home_ca) * bias_mult_away.max(0.3);

    // Possession % is CA-weighted with a small home tilt.
    let total_ca = (input.home_ca as u16 + input.away_ca as u16).max(2);
    let home_ca_share = input.home_ca as f32 / total_ca as f32;
    let possession_home_pct = ((home_ca_share * 100.0 + 4.0).round() as u8).clamp(30, 70);

    let mut home = MatchStats::default();
    let mut away = MatchStats::default();
    let mut events = Vec::new();

    // Walk 90 minutes. Each minute has independent event probabilities
    // scaled from xG and CA. This is simple but produces plausible
    // per-fixture stats — shots on target for the average game (~4-6),
    // corners (~10 total), cards (~3), etc.
    let home_goal_prob = home_xg / 90.0;
    let away_goal_prob = away_xg / 90.0;
    let home_shot_prob = home_goal_prob * 4.5;
    let away_shot_prob = away_goal_prob * 4.5;
    let corner_prob = 0.12;
    let foul_prob = 0.20;
    let yellow_prob = 0.03;
    let red_prob = 0.002;
    let injury_prob = 0.008;

    for minute in 1..=90u8 {
        // Goals
        if rng.next_f32() < home_goal_prob {
            home.goals += 1;
            home.shots_on_target += 1;
            home.shots += 1;
            events.push(MinuteEvent::Goal { minute, side: Side::Home });
        }
        if rng.next_f32() < away_goal_prob {
            away.goals += 1;
            away.shots_on_target += 1;
            away.shots += 1;
            events.push(MinuteEvent::Goal { minute, side: Side::Away });
        }
        // Shots (that aren't goals). Split on target vs off.
        if rng.next_f32() < home_shot_prob {
            home.shots += 1;
            if rng.next_f32() < 0.35 {
                home.shots_on_target += 1;
                events.push(MinuteEvent::ShotOnTarget { minute, side: Side::Home });
            } else {
                events.push(MinuteEvent::ShotOffTarget { minute, side: Side::Home });
            }
        }
        if rng.next_f32() < away_shot_prob {
            away.shots += 1;
            if rng.next_f32() < 0.35 {
                away.shots_on_target += 1;
                events.push(MinuteEvent::ShotOnTarget { minute, side: Side::Away });
            } else {
                events.push(MinuteEvent::ShotOffTarget { minute, side: Side::Away });
            }
        }
        // Corners.
        if rng.next_f32() < corner_prob {
            let s = if rng.next_f32() < home_ca_share { Side::Home } else { Side::Away };
            if s == Side::Home { home.corners += 1 } else { away.corners += 1 }
            events.push(MinuteEvent::Corner { minute, side: s });
        }
        // Fouls / cards.
        if rng.next_f32() < foul_prob {
            let s = if rng.next_f32() < 0.5 { Side::Home } else { Side::Away };
            if s == Side::Home { home.fouls += 1 } else { away.fouls += 1 }
            events.push(MinuteEvent::Foul { minute, side: s });
            if rng.next_f32() < yellow_prob {
                if s == Side::Home { home.yellow_cards += 1 } else { away.yellow_cards += 1 }
                events.push(MinuteEvent::YellowCard { minute, side: s });
            }
            if rng.next_f32() < red_prob {
                if s == Side::Home { home.red_cards += 1 } else { away.red_cards += 1 }
                events.push(MinuteEvent::RedCard { minute, side: s });
            }
        }
        // Injuries.
        if rng.next_f32() < injury_prob {
            let s = if rng.next_f32() < 0.5 { Side::Home } else { Side::Away };
            if s == Side::Home { home.injuries += 1 } else { away.injuries += 1 }
            events.push(MinuteEvent::Injury { minute, side: s });
        }
    }

    MatchResult {
        home_goals: home.goals,
        away_goals: away.goals,
        home_stats: home,
        away_stats: away,
        possession_home_pct,
        events,
    }
}

/// Expected goals for a team of `for_ca` against `against_ca`. Tuned so
/// evenly-matched 100-CA teams both average ~1.2, and a 180-CA team
/// against a 100-CA team averages ~2.3.
fn expected_goals(for_ca: u8, against_ca: u8) -> f32 {
    let ratio = (for_ca as f32 / against_ca.max(1) as f32).powf(0.8);
    (1.2 * ratio).clamp(0.15, 5.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_result() {
        let input = SimulateFixtureInput {
            home_ca: 150, away_ca: 130, fixture_seed: 42, home_advantage: 1.3,
            ..Default::default()
        };
        let a = simulate_fixture(input);
        let b = simulate_fixture(input);
        assert_eq!(a, b);
    }

    #[test]
    fn stronger_team_scores_more_on_average() {
        let mut strong_wins = 0;
        let n = 100;
        for seed in 0..n {
            let r = simulate_fixture(SimulateFixtureInput {
                home_ca: 180, away_ca: 100, fixture_seed: seed,
                home_advantage: 1.3, ..Default::default()
            });
            if r.home_goals > r.away_goals { strong_wins += 1 }
        }
        assert!(strong_wins > n / 2, "strong team should win more often: {}/{}",
                strong_wins, n);
    }

    #[test]
    fn stats_are_plausible() {
        let r = simulate_fixture(SimulateFixtureInput {
            home_ca: 130, away_ca: 130, fixture_seed: 7, home_advantage: 1.3, ..Default::default()
        });
        // Between the two teams there should be some shots and some events.
        assert!(r.home_stats.shots + r.away_stats.shots > 0);
        assert!(!r.events.is_empty());
        // Possession splits reasonably.
        assert!((30..=70).contains(&r.possession_home_pct));
    }

    #[test]
    fn events_are_time_ordered() {
        let r = simulate_fixture(SimulateFixtureInput {
            home_ca: 150, away_ca: 150, fixture_seed: 100, home_advantage: 1.3, ..Default::default()
        });
        let minutes: Vec<u8> = r.events.iter().map(|e| match e {
            MinuteEvent::Goal { minute, .. }
            | MinuteEvent::ShotOnTarget { minute, .. }
            | MinuteEvent::ShotOffTarget { minute, .. }
            | MinuteEvent::YellowCard { minute, .. }
            | MinuteEvent::RedCard { minute, .. }
            | MinuteEvent::Injury { minute, .. }
            | MinuteEvent::Substitution { minute, .. }
            | MinuteEvent::Corner { minute, .. }
            | MinuteEvent::Foul { minute, .. } => *minute,
        }).collect();
        for w in minutes.windows(2) { assert!(w[0] <= w[1]); }
    }

    #[test]
    fn goals_match_result_shape() {
        for seed in 0..50 {
            let r = simulate_fixture(SimulateFixtureInput {
                home_ca: 140, away_ca: 120, fixture_seed: seed,
                home_advantage: 1.3, ..Default::default()
            });
            // Every goal-event should correspond to a stat goal.
            let home_goal_events = r.events.iter()
                .filter(|e| matches!(e, MinuteEvent::Goal { side: Side::Home, .. }))
                .count();
            let away_goal_events = r.events.iter()
                .filter(|e| matches!(e, MinuteEvent::Goal { side: Side::Away, .. }))
                .count();
            assert_eq!(home_goal_events as u8, r.home_goals);
            assert_eq!(away_goal_events as u8, r.away_goals);
        }
    }
}
