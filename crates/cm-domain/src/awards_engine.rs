//! Per-country annual awards engine — consumes [`crate::player_rating`].
//!
//! This is the substrate the game's `*_awards.cpp` TUs all hook into.
//! Each country picks a subset of the categories below and each year at
//! season end the engine writes one `AwardedHonour` per active category
//! into `RuntimeSaveGame::pending_events` for the news feed.
//!
//! # Categories
//!
//! * [`AwardCategory::PlayerOfTheSeason`] — highest season rating in a
//!   given league division.
//! * [`AwardCategory::TopScorer`] — highest goals estimate in the division.
//! * [`AwardCategory::YoungPlayerOfTheSeason`] — highest rating among
//!   players with `age_est < 21`.
//! * [`AwardCategory::TeamOfTheSeason`] — top-11 by season rating.
//!
//! # Fidelity
//!
//! Awards fire at the end of the season the moment
//! [`crate::player_rating::PlayerRatingBook`] can be built. Because that
//! substrate is currently a CA-driven proxy (not a real season-average of
//! match performances), the exact NAME of the winner won't match the exe.
//! What matches is the SHAPE: one award per category per crowned comp per
//! year, deterministic given the same input world.
//!
//! # Country registrations
//!
//! Each country's `*_awards.cpp` amounts to declaring `AwardCategory`
//! values it participates in. The declarations live in
//! [`country_award_slate`] below.

use serde::{Deserialize, Serialize};

use crate::player_rating::PlayerRatingBook;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AwardCategory {
    PlayerOfTheSeason,
    TopScorer,
    YoungPlayerOfTheSeason,
    TeamOfTheSeason,
    /// From `month_award.cpp` — monthly Player of the Month per top-flight
    /// league. Awarded on the first day of each month for the preceding
    /// month's performances.
    PlayerOfTheMonth,
    /// From `month_award.cpp` — monthly Manager of the Month.
    ManagerOfTheMonth,
    /// From `nation_awards.cpp` — annual international Footballer of the
    /// Year (one per continent).
    NationFootballerOfTheYear,
    /// From `nation_awards.cpp` — annual national-team Player of the Year.
    NationPlayerOfTheYear,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwardedHonour {
    pub year: u16,
    pub competition_id: i32,
    pub competition_name: String,
    pub category: AwardCategory,
    pub winner_staff_id: u32,
    pub winner_score: f32,
}

/// Determine the country's award slate for the given league comp id.
/// The mapping matches the game's per-country `*_awards.cpp` files, ported
/// as data. Every league gets Player of Season + Top Scorer; a subset gets
/// Young Player and/or Team of Season based on real-world coverage.
pub fn country_award_slate(comp_id: i32) -> &'static [AwardCategory] {
    use AwardCategory::*;
    // Most-covered leagues (Serie A, English Premier, etc.) get the full slate;
    // second-division and lower-tier leagues get the minimum two.
    const FULL: &[AwardCategory] = &[PlayerOfTheSeason, TopScorer,
                                      YoungPlayerOfTheSeason, TeamOfTheSeason];
    const MIN: &[AwardCategory] = &[PlayerOfTheSeason, TopScorer];
    const MID: &[AwardCategory] = &[PlayerOfTheSeason, TopScorer, TeamOfTheSeason];
    match comp_id {
        // Top-flight leagues: full slate
        7 | 22 | 24 | 143 | 11 | 16 | 63 | 69 | 114 | 119 | 0 | 151 => FULL,
        // Second-tier leagues: mid
        8 | 23 | 25 | 144 | 12 | 17 | 64 | 100 | 118 | 120 | 1 => MID,
        // Everything else: minimum
        _ => MIN,
    }
}

/// Award every category the given competition covers. Called by the
/// tick at season-end for each crowned league.
pub fn award_league_season_end(
    year: u16,
    comp_id: i32,
    comp_name: &str,
    ratings: &PlayerRatingBook,
) -> Vec<AwardedHonour> {
    let mut out = Vec::new();
    for &cat in country_award_slate(comp_id) {
        let winner = match cat {
            AwardCategory::PlayerOfTheSeason => ratings.player_of_the_season(comp_id),
            AwardCategory::TopScorer => ratings.top_scorer(comp_id),
            AwardCategory::YoungPlayerOfTheSeason => ratings.young_player_of_the_season(comp_id),
            // Team of Season is 11 winners; represent as the captain (top-1)
            AwardCategory::TeamOfTheSeason => ratings.team_of_the_season(comp_id).into_iter().next(),
            // Monthly / national awards are never in the country_award_slate
            // (they're driven by separate hooks: hook_monthly + international
            // season-end). Guard so the match is exhaustive.
            AwardCategory::PlayerOfTheMonth
            | AwardCategory::ManagerOfTheMonth
            | AwardCategory::NationFootballerOfTheYear
            | AwardCategory::NationPlayerOfTheYear => continue,
        };
        if let Some(w) = winner {
            let score = match cat {
                AwardCategory::TopScorer => w.goals_est as f32,
                _ => ratings.season_rating(w),
            };
            out.push(AwardedHonour {
                year,
                competition_id: comp_id,
                competition_name: comp_name.to_string(),
                category: cat,
                winner_staff_id: w.staff_id,
                winner_score: score,
            });
        }
    }
    out
}

/// Monthly Player of the Month for a competition. Called on the first
/// day of each month by the tick's monthly hook. Uses the same rating
/// pipeline as the annual award; the monthly wobble is a slight
/// re-seeding on `(comp_id, year, month)` so a season's 9 monthly winners
/// aren't all the same player.
pub fn award_month_player_of_month(
    year: u16,
    month: u8,
    comp_id: i32,
    comp_name: &str,
    ratings: &PlayerRatingBook,
) -> Option<AwardedHonour> {
    // Small deterministic wobble derived from (year, month, comp_id) so the
    // monthly winner differs from the season winner.
    let seed = (year as u64 * 12 + month as u64) ^ (comp_id as u64).wrapping_mul(0x9e3779b1);
    let candidate = ratings.players.iter()
        .filter(|p| p.division_id == Some(comp_id))
        .max_by(|a, b| {
            let sa = ratings.season_rating(a) + monthly_shift(a.staff_id, seed);
            let sb = ratings.season_rating(b) + monthly_shift(b.staff_id, seed);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })?;
    Some(AwardedHonour {
        year,
        competition_id: comp_id,
        competition_name: format!("{} ({}/{})", comp_name, month, year),
        category: AwardCategory::PlayerOfTheMonth,
        winner_staff_id: candidate.staff_id,
        winner_score: ratings.season_rating(candidate),
    })
}

fn monthly_shift(staff_id: u32, seed: u64) -> f32 {
    let mut z = (staff_id as u64).wrapping_add(seed).wrapping_mul(0x9e3779b97f4a7c15);
    z ^= z >> 30;
    ((z & 0xff) as f32) * 0.05  // small shift so month winner differs from season winner
}

/// Cross-league awards (International Footballer of the Year etc.). Given a
/// set of top-flight divisions, picks one Player of Year across all of them.
pub fn award_international_season_end(
    year: u16,
    label: &str,
    top_flight_ids: &[i32],
    ratings: &PlayerRatingBook,
) -> Option<AwardedHonour> {
    let w = ratings.player_of_the_season_across(top_flight_ids)?;
    Some(AwardedHonour {
        year,
        competition_id: -1,
        competition_name: label.to_string(),
        category: AwardCategory::PlayerOfTheSeason,
        winner_staff_id: w.staff_id,
        winner_score: ratings.season_rating(w),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player_rating::{PlayerRatingBook, RatedPlayer};

    fn book(players: Vec<RatedPlayer>) -> PlayerRatingBook {
        PlayerRatingBook { players, ..Default::default() }
    }

    #[test]
    fn top_flight_gets_full_slate() {
        let s = country_award_slate(24); // Italian Serie A
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn lower_tier_gets_minimum_slate() {
        let s = country_award_slate(30); // Italian Serie C2/C
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn league_season_end_emits_one_award_per_category() {
        let a = RatedPlayer { staff_id: 1, club_id: Some(10), division_id: Some(24),
                              ca: 195, pa: 198, goals_est: 25, age_est: 27, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };
        let b = RatedPlayer { staff_id: 2, club_id: Some(11), division_id: Some(24),
                              ca: 175, pa: 190, goals_est: 20, age_est: 19, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };
        let c = RatedPlayer { staff_id: 3, club_id: Some(12), division_id: Some(24),
                              ca: 190, pa: 195, goals_est: 30, age_est: 30, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };
        let bk = book(vec![a, b, c]);
        let awards = award_league_season_end(2001, 24, "Italian Serie A", &bk);
        assert_eq!(awards.len(), 4); // full slate
        // Top scorer should be player 3 (highest goals_est)
        let top_scorer = awards.iter()
            .find(|h| h.category == AwardCategory::TopScorer).unwrap();
        assert_eq!(top_scorer.winner_staff_id, 3);
        // Young player should be player 2 (only one under 21)
        let young = awards.iter()
            .find(|h| h.category == AwardCategory::YoungPlayerOfTheSeason).unwrap();
        assert_eq!(young.winner_staff_id, 2);
    }

    #[test]
    fn international_award_picks_best_across_leagues() {
        let a = RatedPlayer { staff_id: 100, club_id: None, division_id: Some(24),
                              ca: 180, pa: 195, goals_est: 15, age_est: 27, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };
        let b = RatedPlayer { staff_id: 200, club_id: None, division_id: Some(7),
                              ca: 199, pa: 200, goals_est: 22, age_est: 24, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };
        let bk = book(vec![a, b]);
        let w = award_international_season_end(
            2001, "European Footballer", &[24, 7, 11, 16, 52], &bk,
        ).unwrap();
        assert_eq!(w.winner_staff_id, 200);
    }
}
