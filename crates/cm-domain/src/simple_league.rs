//! A reusable plain-league engine — a **double round-robin** with a champion,
//! no finals series or split season. This is the shape of most top divisions
//! (e.g. `bel_first.cpp`, VA `0x0041e890`, vtable `0x00955940`, `[esi+0x2c]=0`
//! meaning a single flat stage). Country league classes that are just a table
//! reuse this instead of a bespoke module.
//!
//! Membership is the clubs whose primary competition (`club+0x57`) is the
//! league's id; the fixtures use a distinct runtime competition id so they never
//! collide with the real id (which can be 0, e.g. the Belgian First Division).

use serde::{Deserialize, Serialize};

use crate::arg_primera::{clubs_in_division, single_round_robin, ArgTeam};
use crate::honours::Honour;
use crate::{
    CmPackedDate, DomainOpaqueRecord, GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture,
};

/// Runtime competition ids for ported plain leagues are `RUNTIME_BASE + real_id`
/// so they never clash with the real id (0 for the Belgian First Division).
pub const RUNTIME_BASE: u32 = 0x7000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleLeagueState {
    pub year: u16,
    /// The real `club_competition` id (membership + generic-path exclusion).
    pub real_comp_id: i32,
    /// The fixtures' competition id (`RUNTIME_BASE + real_comp_id`).
    pub runtime_comp_id: u32,
    pub name: String,
    pub teams: Vec<ArgTeam>,
    pub season_start: GameDate,
    /// Honour id awarded to the champion (shared scheme; `0x7d0` = league champ).
    pub champion_honour: u32,
    /// Round-robin legs: 2 = home+away (default), 1 = single (large leagues like
    /// Brazil's 28-club Série A play a single round-robin).
    #[serde(default = "two")]
    pub legs: u8,
    pub complete: bool,
    pub provenance: String,
}

fn two() -> u8 {
    2
}

impl SimpleLeagueState {
    /// Gather members and build the state. `None` if fewer than two clubs.
    pub fn build(
        clubs: &[DomainOpaqueRecord],
        real_comp_id: i32,
        name: &str,
        year: u16,
        season_start: GameDate,
        champion_honour: u32,
        source_va: &str,
    ) -> Option<Self> {
        let teams = clubs_in_division(clubs, real_comp_id);
        if teams.len() < 2 {
            return None;
        }
        Self::from_teams(teams, real_comp_id, name, year, season_start, champion_honour, 2, source_va)
    }

    /// Build directly from a gathered team list (for leagues whose membership is
    /// not the primary competition, e.g. Brazilian state championships), with a
    /// chosen number of `legs`.
    #[allow(clippy::too_many_arguments)]
    pub fn from_teams(
        teams: Vec<ArgTeam>,
        real_comp_id: i32,
        name: &str,
        year: u16,
        season_start: GameDate,
        champion_honour: u32,
        legs: u8,
        source_va: &str,
    ) -> Option<Self> {
        if teams.len() < 2 {
            return None;
        }
        let team_count = teams.len();
        Some(Self {
            year,
            real_comp_id,
            runtime_comp_id: RUNTIME_BASE + real_comp_id as u32,
            name: name.to_string(),
            teams,
            season_start,
            champion_honour,
            legs: legs.clamp(1, 2),
            complete: false,
            provenance: format!("{name} {year}: {team_count} clubs, {legs}-leg round-robin; ported from {source_va}."),
        })
    }
}

/// Generate the full double round-robin (two legs, venues reversed), one round
/// per week from `season_start`.
pub fn generate(state: &SimpleLeagueState, mut next_row: u32) -> Vec<HeadlessSeasonFixture> {
    let base = CmPackedDate::from_game_date(state.season_start.clone());
    let single = single_round_robin(state.teams.len());
    let rounds_per_leg = single.len();
    let mut fixtures = Vec::new();
    for leg in 0..state.legs as usize {
        for (r, pairs) in single.iter().enumerate() {
            let global_round = leg * rounds_per_leg + r;
            let date = base.add_days((global_round as i16) * 7).to_game_date();
            for &(h, a) in pairs {
                let (hi, ai) = if leg == 0 { (h, a) } else { (a, h) };
                let home = &state.teams[hi];
                let away = &state.teams[ai];
                fixtures.push(HeadlessSeasonFixture {
                    row: next_row,
                    competition_id: state.runtime_comp_id,
                    competition_name: state.name.clone(),
                    date: date.clone(),
                    home_club_id: home.club_id,
                    home_club_name: home.name.clone(),
                    away_club_id: away.club_id,
                    away_club_name: away.name.clone(),
                    status: HeadlessFixtureStatus::Pending,
                    home_score: None,
                    away_score: None,
                    match_packet: None,
                    match_report: None,
                    source: format!("{} {} leg {} round {}", state.name, state.year, leg + 1, r + 1),
                });
                next_row += 1;
            }
        }
    }
    fixtures
}

/// The final table, best-first, once every fixture is played (empty otherwise).
/// Used to seed a playoff (e.g. the Brazilian championship off the Série A
/// table). Ranking: points, goal difference, goals for, reputation, id.
pub fn ranked_table(state: &SimpleLeagueState, fixtures: &[HeadlessSeasonFixture]) -> Vec<ArgTeam> {
    let mut tab: std::collections::BTreeMap<u32, (i32, i32, i32)> =
        state.teams.iter().map(|t| (t.club_id, (0, 0, 0))).collect();
    let mut any = false;
    for f in fixtures.iter().filter(|f| f.competition_id == state.runtime_comp_id) {
        any = true;
        if f.status != HeadlessFixtureStatus::Played {
            return Vec::new();
        }
        let (Some(hs), Some(as_)) = (f.home_score, f.away_score) else { continue };
        let (hs, as_) = (hs as i32, as_ as i32);
        let (hp, ap) = match hs.cmp(&as_) {
            std::cmp::Ordering::Greater => (3, 0),
            std::cmp::Ordering::Less => (0, 3),
            std::cmp::Ordering::Equal => (1, 1),
        };
        if let Some(e) = tab.get_mut(&f.home_club_id) {
            e.0 += hp;
            e.1 += hs - as_;
            e.2 += hs;
        }
        if let Some(e) = tab.get_mut(&f.away_club_id) {
            e.0 += ap;
            e.1 += as_ - hs;
            e.2 += as_;
        }
    }
    if !any {
        return Vec::new();
    }
    let mut ranked = state.teams.clone();
    ranked.sort_by(|a, b| {
        let ta = tab[&a.club_id];
        let tb = tab[&b.club_id];
        tb.0.cmp(&ta.0)
            .then(tb.1.cmp(&ta.1))
            .then(tb.2.cmp(&ta.2))
            .then(b.reputation.cmp(&a.reputation))
            .then(a.club_id.cmp(&b.club_id))
    });
    ranked
}

/// The result of an advancement check.
#[derive(Debug, Default)]
pub struct SimpleLeagueAdvance {
    pub news: Vec<(String, String)>,
    pub honours: Vec<Honour>,
    pub completed: bool,
}

/// Announce the champion (top of the double round-robin table) when every
/// fixture is played.
pub fn advance(state: &SimpleLeagueState, fixtures: &[HeadlessSeasonFixture]) -> SimpleLeagueAdvance {
    let mut out = SimpleLeagueAdvance::default();
    // (points, gd, gf) per club, over played fixtures of this league.
    let mut tab: std::collections::BTreeMap<u32, (i32, i32, i32)> =
        state.teams.iter().map(|t| (t.club_id, (0, 0, 0))).collect();
    let mut any = false;
    for f in fixtures.iter().filter(|f| f.competition_id == state.runtime_comp_id) {
        any = true;
        if f.status != HeadlessFixtureStatus::Played {
            return out; // not finished
        }
        let (Some(hs), Some(as_)) = (f.home_score, f.away_score) else { continue };
        let (hs, as_) = (hs as i32, as_ as i32);
        let (hp, ap) = match hs.cmp(&as_) {
            std::cmp::Ordering::Greater => (3, 0),
            std::cmp::Ordering::Less => (0, 3),
            std::cmp::Ordering::Equal => (1, 1),
        };
        if let Some(e) = tab.get_mut(&f.home_club_id) {
            e.0 += hp;
            e.1 += hs - as_;
            e.2 += hs;
        }
        if let Some(e) = tab.get_mut(&f.away_club_id) {
            e.0 += ap;
            e.1 += as_ - hs;
            e.2 += as_;
        }
    }
    if !any {
        return out;
    }
    let champ = state.teams.iter().max_by(|a, b| {
        let ta = tab[&a.club_id];
        let tb = tab[&b.club_id];
        ta.0
            .cmp(&tb.0)
            .then(ta.1.cmp(&tb.1))
            .then(ta.2.cmp(&tb.2))
            .then(a.reputation.cmp(&b.reputation))
            .then(b.club_id.cmp(&a.club_id))
    });
    if let Some(c) = champ {
        out.news.push((
            "competition".into(),
            format!("{} - crowned {} {} champions", c.name, state.name, state.year),
        ));
        out.honours.push(Honour::champion(
            state.year,
            state.champion_honour,
            state.name.clone(),
            c.club_id,
            c.name.clone(),
        ));
    }
    out.completed = true;
    out
}

/// A knockout playoff seeded from a parent league's final table — the shape of
/// the Brazilian national title (`bra_champ_cup.cpp`, comp 270): the Série A
/// table's top clubs contest a knockout for the championship. Materialised into
/// a [`crate::domestic_cup`] once the parent league finishes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaguePlayoff {
    pub year: u16,
    /// Name of the parent [`SimpleLeagueState`] to read the final table from.
    pub parent_league: String,
    pub name: String,
    pub playoff_comp_id: i32,
    pub top_n: usize,
    pub champion_honour: u32,
    pub start_date: GameDate,
    pub triggered: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(n: usize) -> SimpleLeagueState {
        SimpleLeagueState {
            year: 2001,
            real_comp_id: 0,
            runtime_comp_id: RUNTIME_BASE,
            name: "Belgian First Division".into(),
            teams: (0..n as u32)
                .map(|i| ArgTeam { club_id: i, name: format!("B{i}"), reputation: (100 - i) as u16 })
                .collect(),
            season_start: GameDate { year: 2001, month: 8, day: 1 },
            champion_honour: 0x7d0,
            legs: 2,
            complete: false,
            provenance: "test".into(),
        }
    }

    #[test]
    fn eighteen_teams_double_round_robin() {
        let st = state(18);
        let fx = generate(&st, 0);
        // 18 teams: 17 rounds/leg x 2 x 9 = 306 fixtures.
        assert_eq!(fx.len(), 18 * 17);
    }

    #[test]
    fn champion_is_top_of_table() {
        let st = state(4);
        let mut fx = generate(&st, 0);
        for f in &mut fx {
            f.status = HeadlessFixtureStatus::Played;
            let (h, a) = (f.home_club_id, f.away_club_id);
            let (hs, as_) = if h < a { (2, 0) } else { (0, 2) };
            f.home_score = Some(hs);
            f.away_score = Some(as_);
        }
        let adv = advance(&st, &fx);
        assert!(adv.completed);
        assert_eq!(adv.honours.len(), 1);
        assert!(adv.news[0].1.contains("B0"), "lowest id wins every match");
    }
}
