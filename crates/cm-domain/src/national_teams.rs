//! Port of `national_teams.cpp` — national team squad management.
//!
//! Handles automatic squad selection for each nation from eligible
//! players (players whose nationality matches the nation), and the
//! call-up cycle for international windows.
//!
//! # Selection algorithm
//!
//! Per nation:
//!   1. Filter [`crate::player_rating::RatedPlayer`] to those whose
//!      nationality matches.
//!   2. Rank by season rating (CA + wobble).
//!   3. Take the top 22 — 3 GKs + 19 outfielders is the FIFA squad shape.
//!   4. Cap the outfielders by preferred_position mix (currently proxied:
//!      we take the top 22 without a per-position guarantee, matching
//!      the exe's post-2001 relaxed selection).
//!
//! Injury availability is now honoured — [`select_squad_filtered`] takes
//! an `is_available` callback that the caller wires to
//! [`crate::injury::InjuryBook::is_available`]. Suspensions remain
//! unmodelled; the exe's ban_manager cluster isn't yet decoded.

use serde::{Deserialize, Serialize};

use crate::player_rating::{PlayerRatingBook, RatedPlayer};

/// FIFA regulation squad size for international tournaments.
pub const SQUAD_SIZE: usize = 22;

/// A single call-up in the national-team squad.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalledUpPlayer {
    pub staff_id: u32,
    pub club_id: Option<i32>,
    pub ca: i16,
    /// 1 = captain, 22 = last on the roster.
    pub rank: u8,
}

/// A national team's current 22-man squad.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NationalSquad {
    pub nation_id: i32,
    pub squad: Vec<CalledUpPlayer>,
}

impl NationalSquad {
    /// The captain — highest-rated eligible player.
    pub fn captain(&self) -> Option<&CalledUpPlayer> {
        self.squad.first()
    }

    /// Average CA of the whole squad — a rough team-strength number the
    /// match engine can consume when this squad plays an international.
    pub fn average_ca(&self) -> u8 {
        if self.squad.is_empty() { return 0 }
        let sum: u32 = self.squad.iter().map(|p| p.ca as u32).sum();
        (sum / self.squad.len() as u32).min(255) as u8
    }
}

/// Select a national squad from the rating book.
///
/// `nationality_lookup(staff_id) -> Option<nation_id>` provides per-player
/// nationality — passed in as a closure so the caller can hook whichever
/// data source they have (staff type6, an overlay, etc.).
pub fn select_squad<F>(
    nation_id: i32,
    ratings: &PlayerRatingBook,
    nationality_lookup: F,
) -> NationalSquad
where
    F: FnMut(u32) -> Option<i32>,
{
    // Back-compat wrapper: no availability filter (all players eligible).
    select_squad_filtered(nation_id, ratings, nationality_lookup, |_| true)
}

/// Full-fidelity variant of [`select_squad`] with an availability filter —
/// pass `|id| injuries.is_available(id)` to exclude injured players from
/// the national-team pool. Kills the "everyone is available" placeholder.
pub fn select_squad_filtered<F, A>(
    nation_id: i32,
    ratings: &PlayerRatingBook,
    mut nationality_lookup: F,
    mut is_available: A,
) -> NationalSquad
where
    F: FnMut(u32) -> Option<i32>,
    A: FnMut(u32) -> bool,
{
    let mut eligible: Vec<&RatedPlayer> = ratings.players.iter()
        .filter(|p| nationality_lookup(p.staff_id) == Some(nation_id))
        .filter(|p| is_available(p.staff_id))
        .collect();
    // Rank by season rating descending.
    eligible.sort_by(|a, b| ratings.season_rating(b)
        .partial_cmp(&ratings.season_rating(a))
        .unwrap_or(std::cmp::Ordering::Equal));

    let squad = eligible.into_iter().take(SQUAD_SIZE).enumerate().map(|(i, p)| {
        CalledUpPlayer {
            staff_id: p.staff_id,
            club_id: p.club_id,
            ca: p.ca,
            rank: (i + 1) as u8,
        }
    }).collect();

    NationalSquad { nation_id, squad }
}

/// Batch-select squads for every nation the callback maps players to.
/// Returns one `NationalSquad` per nation that has at least one eligible
/// player.
pub fn select_all_squads<F>(
    ratings: &PlayerRatingBook,
    mut nationality_lookup: F,
) -> Vec<NationalSquad>
where
    F: FnMut(u32) -> Option<i32> + Clone,
{
    use std::collections::HashSet;
    let nations: HashSet<i32> = ratings.players.iter()
        .filter_map(|p| nationality_lookup(p.staff_id))
        .collect();
    nations.into_iter()
        .map(|n| select_squad(n, ratings, nationality_lookup.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(id: u32, ca: i16, club: i32) -> RatedPlayer {
        RatedPlayer { staff_id: id, club_id: Some(club), division_id: Some(24),
                      ca, pa: ca, goals_est: 10, age_est: 25, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] }
    }

    #[test]
    fn squad_is_at_most_22() {
        let players: Vec<_> = (0..40).map(|i| mk(i, 100 + i as i16, 10)).collect();
        let book = PlayerRatingBook { players, ..Default::default() };
        // Every player is nation 94
        let sq = select_squad(94, &book, |_| Some(94));
        assert_eq!(sq.squad.len(), 22);
    }

    #[test]
    fn squad_takes_top_rated_first() {
        // Player 39 has highest CA (100+39=139), player 0 has lowest (100).
        let players: Vec<_> = (0..40).map(|i| mk(i, 100 + i as i16, 10)).collect();
        let book = PlayerRatingBook { players, ..Default::default() };
        let sq = select_squad(94, &book, |_| Some(94));
        // Captain must be the highest-rated (staff_id 39).
        assert_eq!(sq.captain().unwrap().staff_id, 39);
        // Last rank must be a lower-CA player.
        assert!(sq.squad.last().unwrap().ca < sq.captain().unwrap().ca);
    }

    #[test]
    fn only_own_nationality_selected() {
        let players: Vec<_> = (0..10).map(|i| mk(i, 150, 10)).collect();
        let book = PlayerRatingBook { players, ..Default::default() };
        // Half are nation 94, half are nation 97.
        let sq = select_squad(94, &book, |id| Some(if id % 2 == 0 { 94 } else { 97 }));
        assert!(sq.squad.iter().all(|p| p.staff_id % 2 == 0));
    }

    #[test]
    fn average_ca_reflects_squad_strength() {
        let players: Vec<_> = (0..22).map(|i| mk(i, 150, 10)).collect();
        let book = PlayerRatingBook { players, ..Default::default() };
        let sq = select_squad(94, &book, |_| Some(94));
        assert_eq!(sq.average_ca(), 150);
    }

    #[test]
    fn empty_pool_yields_empty_squad() {
        let book = PlayerRatingBook { players: vec![], ..Default::default() };
        let sq = select_squad(94, &book, |_| Some(94));
        assert!(sq.squad.is_empty());
        assert!(sq.captain().is_none());
        assert_eq!(sq.average_ca(), 0);
    }
}
