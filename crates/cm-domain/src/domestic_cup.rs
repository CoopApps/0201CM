//! A reusable single-elimination **domestic cup** engine — the shape of
//! `bel_fa_cup.cpp` (VA `0x0041dba0`, vtable `0x009558a0`, club base
//! `0x00502320`) and every other national cup. Clubs are seeded into a
//! power-of-two bracket (weaker entrants get byes into later rounds); each round
//! is played through the ordinary phase-2 batch and the winners advance until a
//! single champion remains.
//!
//! Reused across nations by supplying the participant pool + competition id.

use serde::{Deserialize, Serialize};

use crate::arg_primera::ArgTeam;
use crate::honours::Honour;
use crate::{
    CmPackedDate, GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture,
};

/// Runtime competition-id base for ported cups (`+ real_comp_id`).
pub const CUP_RUNTIME_BASE: u32 = 0x7800;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CupState {
    pub year: u16,
    pub real_comp_id: i32,
    pub runtime_comp_id: u32,
    pub name: String,
    /// The bracket teams, strongest first (used for seeding + bye placement).
    pub teams: Vec<ArgTeam>,
    /// The current round number (1 = first round created).
    pub round: u32,
    pub start_date: GameDate,
    /// Optional per-round leg-A dates decoded from the exe's cup
    /// round-helper tables (see `reports/cup_round_schedules.md`).
    /// Index 0 = round 1, index 1 = round 2, ... When set, the
    /// `advance()` function uses `round_dates[next_round - 1]`
    /// instead of the naive `start_date + round * 14` fallback.
    /// `#[serde(default)]` keeps older saves loading unchanged.
    #[serde(default)]
    pub round_dates: Vec<GameDate>,
    pub champion_honour: u32,
    pub complete: bool,
    pub provenance: String,
}

/// Largest power of two that is `<= n` (the bracket size).
fn bracket_size(n: usize) -> usize {
    let mut p = 1;
    while p * 2 <= n {
        p *= 2;
    }
    p
}

impl CupState {
    /// Build from a candidate pool; keeps the strongest `2^k` clubs so the
    /// bracket is clean. `None` if fewer than two clubs.
    pub fn build(
        mut pool: Vec<ArgTeam>,
        real_comp_id: i32,
        name: &str,
        year: u16,
        start_date: GameDate,
        champion_honour: u32,
        source_va: &str,
    ) -> Option<Self> {
        if pool.len() < 2 {
            return None;
        }
        pool.sort_by(|a, b| b.reputation.cmp(&a.reputation).then(a.club_id.cmp(&b.club_id)));
        pool.truncate(bracket_size(pool.len()));
        let count = pool.len();
        Some(Self {
            year,
            real_comp_id,
            runtime_comp_id: CUP_RUNTIME_BASE + real_comp_id as u32,
            name: name.to_string(),
            teams: pool,
            round: 0,
            start_date,
            round_dates: Vec::new(),
            champion_honour,
            complete: false,
            provenance: format!("{name} {year}: {count}-team single-elimination cup; ported from {source_va}."),
        })
    }

    /// Attach the decoded per-round leg-A dates (index 0 = round 1).
    /// Passes-through unchanged; the effect is on the `advance()`
    /// scheduling step.
    pub fn with_round_dates(mut self, dates: Vec<GameDate>) -> Self {
        self.round_dates = dates;
        self
    }

    fn tag(&self, round: u32) -> String {
        format!("CUP{}-R{}", self.runtime_comp_id, round)
    }
}

fn fixture(
    state: &CupState,
    row: u32,
    round: u32,
    date: GameDate,
    home: &ArgTeam,
    away: &ArgTeam,
) -> HeadlessSeasonFixture {
    HeadlessSeasonFixture {
        row,
        competition_id: state.runtime_comp_id,
        competition_name: state.name.clone(),
        date,
        home_club_id: home.club_id,
        home_club_name: home.name.clone(),
        away_club_id: away.club_id,
        away_club_name: away.name.clone(),
        status: HeadlessFixtureStatus::Pending,
        home_score: None,
        away_score: None,
        match_packet: None,
        match_report: None,
        source: format!("{} {} round (tag {})", state.name, state.year, state.tag(round)),
    }
}

/// Generate the first round (seeded 1-v-n, 2-v-(n-1), … so top seeds meet
/// weakest first). Mutates `state.round` to 1.
pub fn generate_first_round(state: &mut CupState, mut next_row: u32) -> Vec<HeadlessSeasonFixture> {
    state.round = 1;
    let n = state.teams.len();
    let date = state.start_date.clone();
    let mut out = Vec::new();
    for i in 0..n / 2 {
        let home = state.teams[i].clone();
        let away = state.teams[n - 1 - i].clone();
        out.push(fixture(state, next_row, 1, date.clone(), &home, &away));
        next_row += 1;
    }
    out
}

/// The winner of a played tie: higher score, draw → higher reputation (a
/// documented stand-in for the cup's replay/extra-time path).
fn winner(f: &HeadlessSeasonFixture, rep: &dyn Fn(u32) -> u16) -> Option<ArgTeam> {
    let (hs, as_) = (f.home_score?, f.away_score?);
    let home = match hs.cmp(&as_) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => rep(f.home_club_id) >= rep(f.away_club_id),
    };
    Some(if home {
        ArgTeam { club_id: f.home_club_id, name: f.home_club_name.clone(), reputation: rep(f.home_club_id) }
    } else {
        ArgTeam { club_id: f.away_club_id, name: f.away_club_name.clone(), reputation: rep(f.away_club_id) }
    })
}

/// The result of a cup advancement step.
#[derive(Debug, Default)]
pub struct CupAdvance {
    pub new_fixtures: Vec<HeadlessSeasonFixture>,
    pub news: Vec<(String, String)>,
    pub honours: Vec<Honour>,
    pub completed: bool,
    /// The new round number, if the cup advanced.
    pub new_round: Option<u32>,
}

/// Advance the cup one round: when the current round's ties are all played,
/// pair the winners into the next round; when one winner remains, crown the
/// champion. Idempotent until the round completes.
pub fn advance(state: &CupState, fixtures: &[HeadlessSeasonFixture], next_row: u32) -> CupAdvance {
    let mut out = CupAdvance::default();
    if state.complete || state.round == 0 {
        return out;
    }
    let rep = |id: u32| state.teams.iter().find(|t| t.club_id == id).map(|t| t.reputation).unwrap_or(0);
    let tag = state.tag(state.round);
    let mut cur: Vec<&HeadlessSeasonFixture> =
        fixtures.iter().filter(|f| f.source.contains(&tag)).collect();
    cur.sort_by_key(|f| f.row);
    if cur.is_empty() || cur.iter().any(|f| f.status != HeadlessFixtureStatus::Played) {
        return out;
    }
    let winners: Vec<ArgTeam> = cur.iter().filter_map(|f| winner(f, &rep)).collect();
    if winners.len() == 1 {
        let champ = &winners[0];
        out.news.push((
            "competition".into(),
            format!("{} - win the {} {}", champ.name, state.name, state.year),
        ));
        out.honours.push(Honour::champion(
            state.year,
            state.champion_honour,
            state.name.clone(),
            champ.club_id,
            champ.name.clone(),
        ));
        out.completed = true;
        return out;
    }
    // Pair winners into the next round.
    let next_round = state.round + 1;
    // Prefer the decoded per-round leg-A date when available
    // (populated from cup_round_schedules.md via
    // `with_round_dates`); otherwise fall back to a +14 days
    // per round approximation.
    let date = state.round_dates.get((next_round as usize) - 1)
        .cloned()
        .unwrap_or_else(|| {
            let base = CmPackedDate::from_game_date(state.start_date.clone());
            base.add_days((state.round as i16) * 14).to_game_date()
        });
    let mut row = next_row;
    for pair in winners.chunks(2) {
        if let [h, a] = pair {
            out.new_fixtures.push(fixture(state, row, next_round, date.clone(), h, a));
            row += 1;
        }
    }
    // Retag the new fixtures for the next round.
    for f in &mut out.new_fixtures {
        f.source = format!("{} {} round (tag {})", state.name, state.year, state.tag(next_round));
    }
    out.new_round = Some(next_round);
    out.news.push((
        "competition".into(),
        format!("{} - round {} of the {} is set", state.name, next_round, state.year),
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(n: u32) -> Vec<ArgTeam> {
        (0..n).map(|i| ArgTeam { club_id: i, name: format!("C{i}"), reputation: (200 - i) as u16 }).collect()
    }

    #[test]
    fn bracket_trims_to_power_of_two() {
        let st = CupState::build(pool(18), 332, "Belgian Cup", 2001, GameDate { year: 2001, month: 9, day: 1 }, 0x7d0, "test").unwrap();
        assert_eq!(st.teams.len(), 16);
    }

    #[test]
    fn cup_runs_to_a_single_champion() {
        let mut st = CupState::build(pool(8), 332, "Belgian Cup", 2001, GameDate { year: 2001, month: 9, day: 1 }, 0x7d0, "test").unwrap();
        let mut fx = generate_first_round(&mut st, 0);
        let play = |fx: &mut Vec<HeadlessSeasonFixture>| {
            for f in fx.iter_mut().filter(|f| f.status == HeadlessFixtureStatus::Pending) {
                f.status = HeadlessFixtureStatus::Played;
                // Lower club id (stronger seed) always wins.
                let (h, a) = (f.home_club_id, f.away_club_id);
                let (hs, as_) = if h < a { (2, 0) } else { (0, 2) };
                f.home_score = Some(hs);
                f.away_score = Some(as_);
            }
        };
        // 8 -> 4 -> 2 -> 1: three advance steps.
        let mut guard = 0;
        loop {
            play(&mut fx);
            let next = fx.iter().map(|f| f.row + 1).max().unwrap_or(0);
            let adv = advance(&st, &fx, next);
            fx.extend(adv.new_fixtures);
            if let Some(r) = adv.new_round {
                st.round = r;
            }
            if adv.completed {
                assert_eq!(adv.honours.len(), 1);
                assert!(adv.news[0].1.contains("C0"), "top seed wins the cup");
                break;
            }
            guard += 1;
            assert!(guard < 6, "cup did not converge");
        }
    }
}
