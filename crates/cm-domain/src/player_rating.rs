//! Player-rating substrate — CA-driven per-season performance ratings.
//!
//! This is what the game's per-country `*_awards.cpp` files consume to
//! decide the annual Player of the Season / Top Scorer / Young Player /
//! Team of the Season awards. Without it, none of the awards TUs can be
//! ported — every one of them just reads back a "who's the best" answer
//! this module produces.
//!
//! # Deliberate scope
//!
//! We do NOT reproduce the exe's full performance-rating algorithm. That
//! would require the match engine (per-fixture stats), the form-tracking
//! subsystem, and the injury/morale/fitness modifiers — none of which are
//! ported yet.
//!
//! What we DO produce is a **CA-driven proxy** that's monotone with
//! current ability + a small per-player deterministic wobble seeded from
//! the player's staff id. Concretely:
//!
//! * `season_rating(staff_id) = CA * 0.8 + wobble(staff_id) * 0.2`
//! * `top_scorer_score(staff_id) = CA * 0.6 + wobble(staff_id) * 0.4`
//!
//! The wobble uses a splitmix-style hash of the staff id so it's stable
//! across ticks and deterministic across saves. Real match performance
//! stats will replace the wobble term once the match engine lands — the
//! caller-facing API (`player_of_the_season`, `top_scorer`,
//! `team_of_the_season`) stays the same.
//!
//! # Substrate role
//!
//! Unblocks ALL `*_awards.cpp` TUs:
//! * `holland_awards.cpp`, `ireland_awards.cpp`, `italy_awards.cpp`,
//!   `japan_awards.cpp`, `international_awards.cpp`
//! * `european_awards.cpp` (partially — champion honours already recorded)
//! * `argentina_awards.cpp`, `australia_awards.cpp` (already registered
//!   at low fidelity)
//! * `award_manager.cpp` — the shared engine that consumes what this
//!   substrate exposes
//! * `hall_of_fame.cpp` slot-selection (which player fills each 27x100
//!   slot every season)

use serde::{Deserialize, Serialize};

use crate::{ClubView, DomainOpaqueRecord, StaffBook};

/// Current year assumed for the initial rating snapshot. The book is
/// rebuilt from world data at new-game creation with the real start year.
const DEFAULT_RATING_YEAR: u16 = 2001;
const DEFAULT_RATING_DAY: u16 = 213;  // ~1 Aug — start-of-season

/// A player's per-season score in a given competition. Higher = better.
/// Range roughly 0..250 given CA is 1..200 and wobble is 0..50.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SeasonScore {
    pub staff_id: u32,
    pub score: f32,
    /// Current ability at the time of scoring — makes debugging easy.
    pub ca: i16,
}

/// Deterministic per-id wobble in the range [0.0, 50.0). Seeded by the
/// staff id so a player's score is stable across ticks + saves.
fn wobble(staff_id: u32) -> f32 {
    // splitmix64-style hash on the id then take the low 8 bits as a 0..255
    // value and scale.
    let mut z = (staff_id as u64).wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^= z >> 31;
    ((z & 0xff) as f32) * (50.0 / 255.0)
}

/// One rated player's snapshot: CA + PA + club-affiliation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RatedPlayer {
    pub staff_id: u32,
    pub club_id: Option<i32>,
    pub division_id: Option<i32>,
    pub ca: i16,
    pub pa: i16,
    /// Estimated goals this season (proxy — CA + wobble).
    pub goals_est: u16,
    /// Approximate age. Placeholder until DOB is wired through.
    pub age_est: u8,
    /// Coarse engine-ordinal position (`DomainStaffType10::engine_position_ordinal`) —
    /// byte-exact from the real position-eligibility formula (`FUN_005a2030`),
    /// reduced to what `EngineTeamPlayer::position` actually consumes.
    pub position_ordinal: u8,
    /// Whether this player's real attributes clear the goalkeeper threshold.
    pub is_gk: bool,
}

/// The book of player ratings — one entry per staff record with a valid
/// type10 outfield attribute record.
///
/// Built once per season-rollover and read by every awards engine call.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerRatingBook {
    pub players: Vec<RatedPlayer>,
}

impl PlayerRatingBook {
    /// Build a fresh rating book from the world's staff pool + clubs.
    ///
    /// Players are represented by paired staff records: type6 (person —
    /// name, DOB, current club) + type10 (outfield attributes including CA/PA),
    /// joined by their shared `id` field (memory [[player-init-decode]]:
    /// "person N ↔ type10[N]").
    pub fn build(
        clubs: &[DomainOpaqueRecord],
        staff: &StaffBook,
    ) -> Self {
        // Index type10 by staff id.
        let mut t10_by_id = std::collections::HashMap::with_capacity(staff.type10.len());
        for t in &staff.type10 {
            t10_by_id.insert(t.id, t);
        }
        // Index club division by club id (as i32 for direct comparison with
        // simple_league.real_comp_id at award-lookup time).
        let mut club_div = std::collections::HashMap::with_capacity(clubs.len());
        for rec in clubs {
            let cv = ClubView::new(rec);
            club_div.insert(cv.id() as i32, cv.division_id());
        }

        let mut rated = Vec::with_capacity(staff.type6.len());
        for p in &staff.type6 {
            let sid = p.id;
            let Some(t10) = t10_by_id.get(&sid) else { continue };
            let ca = t10.current_ability();
            if ca <= 0 { continue }
            let pa = t10.resolved_potential_ability();
            let club_id = p.current_club_id().map(|c| c as i32);
            let div_id = club_id.and_then(|c| club_div.get(&c).copied()).flatten();
            let goals = (ca as f32 * 0.06 + wobble(sid) * 0.2) as u16;
            let age = p.age_at(DEFAULT_RATING_YEAR, DEFAULT_RATING_DAY).unwrap_or(25);
            let (position_ordinal, is_gk) = t10.engine_position_ordinal();
            rated.push(RatedPlayer {
                staff_id: sid,
                club_id,
                division_id: div_id,
                ca,
                pa,
                goals_est: goals,
                age_est: age,
                position_ordinal,
                is_gk,
            });
        }
        Self { players: rated }
    }

    /// The score every award category uses as its base metric.
    pub fn season_rating(&self, p: &RatedPlayer) -> f32 {
        p.ca as f32 * 0.8 + wobble(p.staff_id) * 0.2
    }

    /// Top-scorer specifically weights the wobble higher — a striker with
    /// a lower CA can outscore a higher-CA midfielder.
    pub fn top_scorer_score(&self, p: &RatedPlayer) -> f32 {
        p.ca as f32 * 0.6 + wobble(p.staff_id) * 0.4
    }

    /// The players eligible for a competition (those whose club is in the
    /// competition's division).
    fn eligible_for(&self, division_id: i32) -> impl Iterator<Item = &RatedPlayer> {
        self.players.iter().filter(move |p| p.division_id == Some(division_id))
    }

    /// Player of the Season for a competition. Returns None if the
    /// competition has no eligible players (e.g. a cup that draws from
    /// multiple divisions — the caller should use `player_of_the_season_across`
    /// for that case).
    pub fn player_of_the_season(&self, division_id: i32) -> Option<&RatedPlayer> {
        self.eligible_for(division_id)
            .max_by(|a, b| self.season_rating(a).partial_cmp(&self.season_rating(b))
                .unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Top Scorer for a competition — same eligibility, different score.
    pub fn top_scorer(&self, division_id: i32) -> Option<&RatedPlayer> {
        self.eligible_for(division_id)
            .max_by(|a, b| a.goals_est.cmp(&b.goals_est))
    }

    /// Young Player of the Season — eligible == age < 21.
    pub fn young_player_of_the_season(&self, division_id: i32) -> Option<&RatedPlayer> {
        self.eligible_for(division_id)
            .filter(|p| p.age_est < 21)
            .max_by(|a, b| self.season_rating(a).partial_cmp(&self.season_rating(b))
                .unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Player of the Season across a set of divisions (for cross-division
    /// awards like "Italian Serie B Team of the Year" or "European Footballer
    /// of the Year" that draw from multiple leagues).
    pub fn player_of_the_season_across(&self, division_ids: &[i32]) -> Option<&RatedPlayer> {
        self.players.iter()
            .filter(|p| p.division_id.map_or(false, |d| division_ids.contains(&d)))
            .max_by(|a, b| self.season_rating(a).partial_cmp(&self.season_rating(b))
                .unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Best XI for a division — top 11 players by season rating.
    pub fn team_of_the_season(&self, division_id: i32) -> Vec<&RatedPlayer> {
        let mut v: Vec<&RatedPlayer> = self.eligible_for(division_id).collect();
        v.sort_by(|a, b| self.season_rating(b).partial_cmp(&self.season_rating(a))
            .unwrap_or(std::cmp::Ordering::Equal));
        v.into_iter().take(11).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wobble_is_deterministic() {
        assert_eq!(wobble(42), wobble(42));
        assert!(wobble(1) != wobble(2));
        for id in [0u32, 1, 100, 10_000, u32::MAX] {
            let w = wobble(id);
            assert!((0.0..50.0).contains(&w), "wobble({}) = {}", id, w);
        }
    }

    #[test]
    fn empty_book_yields_no_awards() {
        let b = PlayerRatingBook { players: vec![] };
        assert!(b.player_of_the_season(7).is_none());
        assert!(b.top_scorer(7).is_none());
        assert!(b.young_player_of_the_season(7).is_none());
        assert!(b.team_of_the_season(7).is_empty());
    }

    #[test]
    fn poty_picks_highest_rated_in_division() {
        let a = RatedPlayer { staff_id: 1, club_id: Some(10), division_id: Some(7),
                              ca: 150, pa: 160, goals_est: 10, age_est: 25 };
        let b = RatedPlayer { staff_id: 2, club_id: Some(11), division_id: Some(7),
                              ca: 180, pa: 190, goals_est: 15, age_est: 26 };
        let c = RatedPlayer { staff_id: 3, club_id: Some(20), division_id: Some(8),
                              ca: 200, pa: 200, goals_est: 30, age_est: 27 };  // other division
        let book = PlayerRatingBook { players: vec![a.clone(), b.clone(), c.clone()] };
        assert_eq!(book.player_of_the_season(7).unwrap().staff_id, 2);
        assert_eq!(book.player_of_the_season(8).unwrap().staff_id, 3);
    }

    #[test]
    fn cross_division_award_picks_best_across_set() {
        let a = RatedPlayer { staff_id: 1, club_id: Some(10), division_id: Some(24),
                              ca: 170, pa: 180, goals_est: 12, age_est: 25 };
        let b = RatedPlayer { staff_id: 2, club_id: Some(11), division_id: Some(25),
                              ca: 195, pa: 195, goals_est: 25, age_est: 28 };
        let book = PlayerRatingBook { players: vec![a, b] };
        assert_eq!(book.player_of_the_season_across(&[24, 25]).unwrap().staff_id, 2);
    }
}
