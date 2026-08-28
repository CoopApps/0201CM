//! Australian National Soccer League — a port of `aus_nsl.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\leagues\aus_nsl.cpp`, VA
//! `0x00410d70..0x0041297f`, vtable `0x00955768`). Function decode + names:
//! `reports/carve_rename_map.json` (`aus_nsl.cpp`).
//!
//! A domestic league with a **finals series** — the exe models it as 3 stages
//! (`[esi+0x2c]=3`, `ausnsl_advance_stage` `0x004121f0`): a double round-robin
//! regular season (`ausnsl_build_fixture_template` `0x00411080`, type
//! `0x1a`=26 rounds for the 14 clubs) whose table crowns the **Minor Premier**,
//! then a **finals series** (`ausnsl_build_finals_series` `0x004122a0`) that
//! decides the **Champion** in a Grand Final.
//!
//! This port plays the double round-robin, then a top-4 knockout finals
//! (two semis + a Grand Final). It reuses the Argentine league helpers
//! ([`crate::arg_primera`]).
//!
//! ## Fidelity notes
//! * **Membership / format** — exact: the 14 clubs with primary competition id
//!   151; double round-robin (26 rounds).
//! * **Finals** — the real NSL used a 6-team finals series; this port uses a
//!   top-4 knockout (semis + Grand Final) as a documented simplification. A
//!   drawn final is decided by regular-season standing.

use serde::{Deserialize, Serialize};

use crate::arg_primera::{clubs_in_division, single_round_robin, ArgTeam};
use crate::{
    CmPackedDate, DomainOpaqueRecord, GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture,
};

/// `club_competition` id of the Australian NSL.
pub const AUS_NSL_COMP_ID: i32 = 151;
/// Runtime comp id the exe assigns the regular season (`0x69a`).
pub const AUS_NSL_COMP_RUNTIME: u32 = 0x69a;
pub const AUS_NSL_TEAM_COUNT: usize = 14;
/// Teams contesting the finals series.
pub const AUS_NSL_FINALS_TEAMS: usize = 4;
/// Honour ids from the shared honour-id scheme, VERIFIED against
/// `australia_awards.cpp` (`ausawards_create_honours` `0x00412ac0`): `0x7d0` is
/// the national-league-champion honour (the same id the Argentine Primera uses)
/// and `0x3e8` a secondary honour, used here for the minor premiership.
pub const AUS_NSL_CHAMPION_HONOUR: u32 = 0x7d0;
pub const AUS_NSL_MINOR_PREMIER_HONOUR: u32 = 0x3e8;
pub const TAG_SF: &str = "NSL-SF";
pub const TAG_GF: &str = "NSL-GF";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AusNslStage {
    Regular,
    SemiFinals,
    GrandFinal,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AusNslState {
    pub year: u16,
    pub teams: Vec<ArgTeam>,
    pub season_start: GameDate,
    pub stage: AusNslStage,
    pub provenance: String,
}

/// The 14 NSL clubs (primary competition id 151).
pub fn nsl_clubs(clubs: &[DomainOpaqueRecord]) -> Vec<ArgTeam> {
    clubs_in_division(clubs, AUS_NSL_COMP_ID)
}

/// Generate the regular season: a **double** round-robin (two legs, venues
/// reversed in the second), one round per week.
pub fn generate_regular_season(state: &AusNslState, mut next_row: u32) -> Vec<HeadlessSeasonFixture> {
    let base = CmPackedDate::from_game_date(state.season_start.clone());
    let single = single_round_robin(state.teams.len());
    let rounds_per_leg = single.len();
    let mut fixtures = Vec::new();
    for leg in 0..2 {
        for (r, pairs) in single.iter().enumerate() {
            let global_round = leg * rounds_per_leg + r;
            let date = base.add_days((global_round as i16) * 7).to_game_date();
            for &(h, a) in pairs {
                // Leg 2 swaps home/away.
                let (hi, ai) = if leg == 0 { (h, a) } else { (a, h) };
                let home = &state.teams[hi];
                let away = &state.teams[ai];
                fixtures.push(HeadlessSeasonFixture {
                    row: next_row,
                    competition_id: AUS_NSL_COMP_RUNTIME,
                    competition_name: "Australian NSL".to_string(),
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
                    source: format!(
                        "Australian NSL {} leg {} round {} (ausnsl_build_fixture_template 0x00411080)",
                        state.year,
                        leg + 1,
                        r + 1
                    ),
                });
                next_row += 1;
            }
        }
    }
    fixtures
}

// ---- Table + finals (save-side, pure) ----

fn regular_table(
    state: &AusNslState,
    fixtures: &[HeadlessSeasonFixture],
) -> std::collections::BTreeMap<u32, (i32, i32, i32)> {
    let mut tab: std::collections::BTreeMap<u32, (i32, i32, i32)> =
        state.teams.iter().map(|t| (t.club_id, (0, 0, 0))).collect();
    for f in fixtures.iter().filter(|f| {
        f.competition_id == AUS_NSL_COMP_RUNTIME
            && !f.source.contains(TAG_SF)
            && !f.source.contains(TAG_GF)
    }) {
        if f.status != HeadlessFixtureStatus::Played {
            continue;
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
    tab
}

/// Regular-season ladder, best-first (points, GD, GF, reputation, id).
fn ladder<'a>(state: &'a AusNslState, fixtures: &[HeadlessSeasonFixture]) -> Vec<&'a ArgTeam> {
    let tab = regular_table(state, fixtures);
    let mut v: Vec<&ArgTeam> = state.teams.iter().collect();
    v.sort_by(|a, b| {
        let ta = tab[&a.club_id];
        let tb = tab[&b.club_id];
        tb.0.cmp(&ta.0)
            .then(tb.1.cmp(&ta.1))
            .then(tb.2.cmp(&ta.2))
            .then(b.reputation.cmp(&a.reputation))
            .then(a.club_id.cmp(&b.club_id))
    });
    v
}

fn round_all_played(fixtures: &[HeadlessSeasonFixture], tag: Option<&str>) -> bool {
    let mut any = false;
    for f in fixtures.iter().filter(|f| f.competition_id == AUS_NSL_COMP_RUNTIME) {
        let is_round = match tag {
            Some(t) => f.source.contains(t),
            None => !f.source.contains(TAG_SF) && !f.source.contains(TAG_GF),
        };
        if is_round {
            any = true;
            if f.status != HeadlessFixtureStatus::Played {
                return false;
            }
        }
    }
    any
}

fn finals_fixture(
    state: &AusNslState,
    row: u32,
    date: GameDate,
    tag: &str,
    label: &str,
    home: (u32, String),
    away: (u32, String),
) -> HeadlessSeasonFixture {
    HeadlessSeasonFixture {
        row,
        competition_id: AUS_NSL_COMP_RUNTIME,
        competition_name: "Australian NSL".to_string(),
        date,
        home_club_id: home.0,
        home_club_name: home.1,
        away_club_id: away.0,
        away_club_name: away.1,
        status: HeadlessFixtureStatus::Pending,
        home_score: None,
        away_score: None,
        match_packet: None,
        match_report: None,
        source: format!(
            "{tag} {} {} (ausnsl_build_finals_series 0x004122a0; drawn ties to higher seed)",
            state.year, label
        ),
    }
}

fn winner(state: &AusNslState, f: &HeadlessSeasonFixture, seed_rank: &dyn Fn(u32) -> usize) -> Option<(u32, String)> {
    let (hs, as_) = (f.home_score?, f.away_score?);
    let home_wins = match hs.cmp(&as_) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        // Draw → better regular-season standing advances.
        std::cmp::Ordering::Equal => seed_rank(f.home_club_id) <= seed_rank(f.away_club_id),
    };
    let _ = state;
    Some(if home_wins {
        (f.home_club_id, f.home_club_name.clone())
    } else {
        (f.away_club_id, f.away_club_name.clone())
    })
}

/// The result of one advancement step.
#[derive(Debug, Default)]
pub struct AusNslAdvance {
    pub new_fixtures: Vec<HeadlessSeasonFixture>,
    pub news: Vec<(String, String)>,
    pub honours: Vec<(u32, u32, String)>, // (honour_id, club_id, club_name)
    pub new_stage: Option<AusNslStage>,
}

/// Advance the NSL: regular season → semis (top 4) → Grand Final → champion.
pub fn advance(state: &AusNslState, fixtures: &[HeadlessSeasonFixture], next_row: u32) -> AusNslAdvance {
    let mut out = AusNslAdvance::default();
    let base = CmPackedDate::from_game_date(state.season_start.clone());
    // Seed rank by regular-season ladder for tie-breaks.
    let order = ladder(state, fixtures);
    let rank_of = |id: u32| order.iter().position(|t| t.club_id == id).unwrap_or(usize::MAX);
    let seed_rank = |id: u32| rank_of(id);

    match state.stage {
        AusNslStage::Regular => {
            if !round_all_played(fixtures, None) {
                return out;
            }
            if order.len() < AUS_NSL_FINALS_TEAMS {
                return out;
            }
            // Minor Premier = top of the regular ladder.
            let minor = order[0];
            out.news.push((
                "competition".into(),
                format!("{} - clinch the Australian NSL {} minor premiership", minor.name, state.year),
            ));
            out.honours.push((AUS_NSL_MINOR_PREMIER_HONOUR, minor.club_id, minor.name.clone()));
            // Top-4 semis: 1v4, 2v3 (higher seed at home).
            let date = base.add_days(27 * 7).to_game_date();
            let s = |i: usize| (order[i].club_id, order[i].name.clone());
            out.new_fixtures.push(finals_fixture(state, next_row, date.clone(), TAG_SF, "major semi-final", s(0), s(3)));
            out.new_fixtures.push(finals_fixture(state, next_row + 1, date, TAG_SF, "minor semi-final", s(1), s(2)));
            out.new_stage = Some(AusNslStage::SemiFinals);
        }
        AusNslStage::SemiFinals => {
            if !round_all_played(fixtures, Some(TAG_SF)) {
                return out;
            }
            let mut sfs: Vec<&HeadlessSeasonFixture> =
                fixtures.iter().filter(|f| f.source.contains(TAG_SF)).collect();
            sfs.sort_by_key(|f| f.row);
            let w: Vec<(u32, String)> = sfs.iter().filter_map(|f| winner(state, f, &seed_rank)).collect();
            if w.len() < 2 {
                return out;
            }
            let date = base.add_days(29 * 7).to_game_date();
            // Grand Final: the two semi winners (higher seed at home).
            let (h, a) = if seed_rank(w[0].0) <= seed_rank(w[1].0) {
                (w[0].clone(), w[1].clone())
            } else {
                (w[1].clone(), w[0].clone())
            };
            out.new_fixtures.push(finals_fixture(state, next_row, date, TAG_GF, "Grand Final", h, a));
            out.new_stage = Some(AusNslStage::GrandFinal);
            out.news.push((
                "competition".into(),
                format!("Australian NSL - the {} Grand Final is set", state.year),
            ));
        }
        AusNslStage::GrandFinal => {
            let gf = fixtures.iter().find(|f| f.source.contains(TAG_GF));
            let Some(gf) = gf.filter(|f| f.status == HeadlessFixtureStatus::Played) else {
                return out;
            };
            if let Some((id, name)) = winner(state, gf, &seed_rank) {
                out.news.push((
                    "competition".into(),
                    format!("{name} - win the Australian NSL {} Grand Final", state.year),
                ));
                out.honours.push((AUS_NSL_CHAMPION_HONOUR, id, name));
            }
            out.new_stage = Some(AusNslStage::Complete);
        }
        AusNslStage::Complete => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(n: usize) -> AusNslState {
        AusNslState {
            year: 2001,
            teams: (0..n as u32)
                .map(|i| ArgTeam { club_id: i, name: format!("N{i}"), reputation: (100 - i) as u16 })
                .collect(),
            season_start: GameDate { year: 2001, month: 10, day: 6 },
            stage: AusNslStage::Regular,
            provenance: "test".into(),
        }
    }

    #[test]
    fn regular_season_is_double_round_robin() {
        let st = state(14);
        let fx = generate_regular_season(&st, 0);
        // 14 teams: 13 rounds/leg x 2 legs x 7 matches = 182 fixtures.
        assert_eq!(fx.len(), 14 * 13);
        assert!(fx.iter().all(|f| f.competition_id == AUS_NSL_COMP_RUNTIME));
    }

    #[test]
    fn finals_series_runs_to_a_champion() {
        let mut st = state(6);
        let mut fx = generate_regular_season(&st, 0);
        // Play the regular season: lower club id always wins (so seeds are by id).
        let play = |fx: &mut Vec<HeadlessSeasonFixture>, tag: Option<&str>| {
            for f in fx.iter_mut() {
                let is = match tag {
                    Some(t) => f.source.contains(t),
                    None => !f.source.contains(TAG_SF) && !f.source.contains(TAG_GF),
                };
                if is && f.status == HeadlessFixtureStatus::Pending {
                    f.status = HeadlessFixtureStatus::Played;
                    let (h, a) = (f.home_club_id, f.away_club_id);
                    let (hs, as_) = if h < a { (2, 0) } else { (0, 2) };
                    f.home_score = Some(hs);
                    f.away_score = Some(as_);
                }
            }
        };
        play(&mut fx, None);
        let next = fx.iter().map(|f| f.row + 1).max().unwrap_or(0);
        let adv = advance(&st, &fx, next);
        assert_eq!(adv.new_stage, Some(AusNslStage::SemiFinals));
        assert_eq!(adv.new_fixtures.len(), 2);
        assert_eq!(adv.honours.len(), 1); // minor premier
        fx.extend(adv.new_fixtures);
        st.stage = AusNslStage::SemiFinals;

        play(&mut fx, Some(TAG_SF));
        let next = fx.iter().map(|f| f.row + 1).max().unwrap_or(0);
        let adv = advance(&st, &fx, next);
        assert_eq!(adv.new_stage, Some(AusNslStage::GrandFinal));
        fx.extend(adv.new_fixtures);
        st.stage = AusNslStage::GrandFinal;

        play(&mut fx, Some(TAG_GF));
        let adv = advance(&st, &fx, 0);
        assert_eq!(adv.new_stage, Some(AusNslStage::Complete));
        assert_eq!(adv.honours.len(), 1); // champion
        assert!(adv.news[0].1.contains("Grand Final"));
    }
}
