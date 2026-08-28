//! Argentine Primera División — a faithful port of `arg_prm.cpp`
//! (VA `0x00404290..0x00406c2f`, vtable `0x00955324`). Function decode +
//! names: `reports/carve_rename_map.json` (`arg_prm.cpp`).
//!
//! The Argentine top flight runs a **split season**: two separate championships
//! per year — **Apertura** (`argprm_build_fixture_template` variant `0xff`, the
//! exe's runtime comp id `0x434`) and **Clausura** (variant `0x00`, comp
//! `0x435`) — each a single round-robin of the 20 clubs, each crowning its own
//! champion. Relegation is by **promedios**: a points-per-game average, not the
//! single-season table.
//!
//! This module reproduces that as real fixtures in the headless engine: both
//! tournaments' matches play through the ordinary phase-2 batch and emit
//! `pending_events` news, and the save-side [`advance`] announces each champion
//! and applies promedios relegation at the right moments — the same shape as the
//! African Cup of Nations port ([`crate::african_nations`]).
//!
//! ## Fidelity notes
//! * **Membership** — exact: the 20 clubs whose primary competition
//!   (`club+0x57`) is the Argentine Premier Division (id 63), the field
//!   `argprm_gather_clubs` (`0x00405580`) filters on.
//! * **Split season / single round-robin** — exact structure: two single
//!   round-robins (19 rounds each), Apertura then Clausura.
//! * **Promedios** — the exe averages points over three seasons
//!   (`argprm_compute_relegation_averages` `0x004065b0`: `fild pts / fidiv
//!   games`). The headless model has one season of results, so the average is
//!   taken over this season's games; the multi-season carry is a documented
//!   follow-up, not invented.
//! * **Dates** — Apertura in the second half of the start year, Clausura in the
//!   first half of the next; exact matchdays come from the season-schedule
//!   function, so a documented weekly cadence stands in.

use serde::{Deserialize, Serialize};

use crate::typed_records::ClubView;
use crate::{
    CmPackedDate, DomainOpaqueRecord, GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture,
};

/// `club_competition` id of the Argentine Premier Division (the exe's
/// `0x9bbb14` membership filter).
pub const ARG_PRIMERA_COMP_ID: i32 = 63;
/// Runtime comp id the exe assigns the Apertura (`argprm_build_fixture_template`
/// variant `0xff`).
pub const APERTURA_COMP_ID: u32 = 0x434;
/// Runtime comp id the exe assigns the Clausura (variant `0x00`).
pub const CLAUSURA_COMP_ID: u32 = 0x435;
pub const ARG_PRIMERA_TEAM_COUNT: usize = 20;
/// Clubs relegated per season by promedios.
pub const ARG_RELEGATION_COUNT: usize = 2;

/// Which half of the split season the stored edition is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArgPhase {
    Apertura,
    Clausura,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgTeam {
    pub club_id: u32,
    pub name: String,
    pub reputation: u16,
}

/// Persisted state of the current Argentine season.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgPrimeraState {
    pub year: u16,
    pub teams: Vec<ArgTeam>,
    pub apertura_start: GameDate,
    pub clausura_start: GameDate,
    pub phase: ArgPhase,
    pub provenance: String,
}

/// The clubs whose primary competition (`ClubView::division_id`, `club+0x57`)
/// is `comp_id` — the exe's `argprm_gather_clubs` / `argsecond_gather_clubs`
/// membership filter.
pub fn clubs_in_division(clubs: &[DomainOpaqueRecord], comp_id: i32) -> Vec<ArgTeam> {
    clubs
        .iter()
        .filter_map(|rec| {
            let club = ClubView::new(rec);
            if club.division_id() == Some(comp_id) {
                Some(ArgTeam {
                    club_id: club.id(),
                    name: club.primary_name(),
                    reputation: club.reputation(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// The 20 Argentine Primera clubs (primary competition [`ARG_PRIMERA_COMP_ID`]).
pub fn argentine_primera_clubs(clubs: &[DomainOpaqueRecord]) -> Vec<ArgTeam> {
    clubs_in_division(clubs, ARG_PRIMERA_COMP_ID)
}

/// Clubs in `comp_id` via ANY of the three competition slots
/// (`club+0x57/0x5b/0x60`). Brazilian state championships, for instance, are a
/// club's *secondary/tertiary* competition, not its primary division.
pub fn clubs_in_any_competition(clubs: &[DomainOpaqueRecord], comp_id: i32) -> Vec<ArgTeam> {
    clubs
        .iter()
        .filter_map(|rec| {
            let club = ClubView::new(rec);
            if club.competition_ids().any(|c| c == comp_id) {
                Some(ArgTeam {
                    club_id: club.id(),
                    name: club.primary_name(),
                    reputation: club.reputation(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Circle-method single round-robin: `n-1` rounds, `n/2` matches each, every
/// team playing every other once. Home/away alternate by (round, slot) so a
/// club isn't always home. Mirrors the rotation in the league builder. An odd
/// `n` gets a phantom BYE, so every team sits out exactly one round.
pub fn single_round_robin(n: usize) -> Vec<Vec<(usize, usize)>> {
    if n < 2 {
        return Vec::new();
    }
    let mut arr: Vec<usize> = (0..n).collect();
    if n % 2 != 0 {
        arr.push(usize::MAX); // BYE
    }
    let m = arr.len();
    let mut rounds = Vec::with_capacity(m - 1);
    for round in 0..m - 1 {
        let mut pairs = Vec::with_capacity(m / 2);
        for i in 0..m / 2 {
            let a = arr[i];
            let b = arr[m - 1 - i];
            if a == usize::MAX || b == usize::MAX {
                continue;
            }
            let (home, away) = if (round + i) % 2 == 0 { (a, b) } else { (b, a) };
            pairs.push((home, away));
        }
        rounds.push(pairs);
        // rotate all but the first.
        if let Some(last) = arr.pop() {
            arr.insert(1, last);
        }
    }
    rounds
}

/// Generate one tournament (single round-robin) as headless fixtures, one round
/// per week from `start`.
fn generate_tournament(
    state: &ArgPrimeraState,
    comp_id: u32,
    comp_name: &str,
    start: &GameDate,
    mut next_row: u32,
) -> Vec<HeadlessSeasonFixture> {
    let base = CmPackedDate::from_game_date(start.clone());
    let mut fixtures = Vec::new();
    for (round, pairs) in single_round_robin(state.teams.len()).into_iter().enumerate() {
        let date = base.add_days((round as i16) * 7).to_game_date();
        for (h, a) in pairs {
            let home = &state.teams[h];
            let away = &state.teams[a];
            fixtures.push(HeadlessSeasonFixture {
                row: next_row,
                competition_id: comp_id,
                competition_name: comp_name.to_string(),
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
                    "{comp_name} {} round {} (argprm_build_fixture_template 0x004045b0)",
                    state.year,
                    round + 1
                ),
            });
            next_row += 1;
        }
    }
    fixtures
}

/// Generate both championships' fixtures for a freshly-built season.
pub fn generate_season_fixtures(
    state: &ArgPrimeraState,
    start_row: u32,
) -> Vec<HeadlessSeasonFixture> {
    let mut fx = generate_tournament(
        state,
        APERTURA_COMP_ID,
        "Argentine Apertura",
        &state.apertura_start,
        start_row,
    );
    let next = fx.iter().map(|f| f.row + 1).max().unwrap_or(start_row);
    fx.extend(generate_tournament(
        state,
        CLAUSURA_COMP_ID,
        "Argentine Clausura",
        &state.clausura_start,
        next,
    ));
    fx
}

// ---- Standings / promedios (save-side, pure) ----

/// Points/games/goal-difference per club id from the played fixtures of one
/// competition. Points 3/1/0.
fn tournament_table(
    fixtures: &[HeadlessSeasonFixture],
    comp_id: u32,
) -> std::collections::BTreeMap<u32, (i32, i32, i32)> {
    let mut tab: std::collections::BTreeMap<u32, (i32, i32, i32)> = std::collections::BTreeMap::new();
    for f in fixtures.iter().filter(|f| f.competition_id == comp_id) {
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
        let h = tab.entry(f.home_club_id).or_insert((0, 0, 0));
        h.0 += hp;
        h.1 += 1;
        h.2 += hs - as_;
        let a = tab.entry(f.away_club_id).or_insert((0, 0, 0));
        a.0 += ap;
        a.1 += 1;
        a.2 += as_ - hs;
    }
    tab
}

fn all_played(fixtures: &[HeadlessSeasonFixture], comp_id: u32) -> bool {
    let mut any = false;
    for f in fixtures.iter().filter(|f| f.competition_id == comp_id) {
        any = true;
        if f.status != HeadlessFixtureStatus::Played {
            return false;
        }
    }
    any
}

/// The champion of a completed tournament: most points, then goal difference,
/// then reputation, then id.
fn champion<'a>(
    state: &'a ArgPrimeraState,
    fixtures: &[HeadlessSeasonFixture],
    comp_id: u32,
) -> Option<&'a ArgTeam> {
    let tab = tournament_table(fixtures, comp_id);
    state
        .teams
        .iter()
        .filter(|t| tab.contains_key(&t.club_id))
        .max_by(|a, b| {
            let ta = tab[&a.club_id];
            let tb = tab[&b.club_id];
            ta.0
                .cmp(&tb.0)
                .then(ta.2.cmp(&tb.2))
                .then(a.reputation.cmp(&b.reputation))
                .then(b.club_id.cmp(&a.club_id))
        })
}

/// The clubs relegated by promedios: lowest points-per-game across both
/// tournaments. Returns the bottom [`ARG_RELEGATION_COUNT`] names, worst first.
pub fn relegated_by_promedios(
    state: &ArgPrimeraState,
    fixtures: &[HeadlessSeasonFixture],
) -> Vec<String> {
    let ap = tournament_table(fixtures, APERTURA_COMP_ID);
    let cl = tournament_table(fixtures, CLAUSURA_COMP_ID);
    let mut avgs: Vec<(f64, u16, u32, String)> = state
        .teams
        .iter()
        .map(|t| {
            let (p1, g1, _) = ap.get(&t.club_id).copied().unwrap_or((0, 0, 0));
            let (p2, g2, _) = cl.get(&t.club_id).copied().unwrap_or((0, 0, 0));
            let games = g1 + g2;
            let avg = if games > 0 {
                (p1 + p2) as f64 / games as f64
            } else {
                0.0
            };
            (avg, t.reputation, t.club_id, t.name.clone())
        })
        .collect();
    // Ascending by average; ties keep the weaker/lower reputation down.
    avgs.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))
    });
    avgs.into_iter()
        .take(ARG_RELEGATION_COUNT)
        .map(|(_, _, _, name)| name)
        .collect()
}

/// The result of one advancement check.
#[derive(Debug, Default)]
pub struct ArgAdvance {
    /// `(news_kind, message)` — message already `"<headline> - <summary>"`.
    pub news: Vec<(String, String)>,
    /// Championship honours awarded this step (port of argentina_awards.cpp).
    pub honours: Vec<crate::honours::Honour>,
    pub new_phase: Option<ArgPhase>,
}

/// Advance the split season: announce the Apertura champion when it finishes,
/// then the Clausura champion + promedios relegation when that finishes. Pure;
/// idempotent until a tournament completes.
pub fn advance(state: &ArgPrimeraState, fixtures: &[HeadlessSeasonFixture]) -> ArgAdvance {
    let mut out = ArgAdvance::default();
    match state.phase {
        ArgPhase::Apertura => {
            if !all_played(fixtures, APERTURA_COMP_ID) {
                return out;
            }
            if let Some(champ) = champion(state, fixtures, APERTURA_COMP_ID) {
                out.news.push((
                    "competition".into(),
                    format!("{} - crowned Argentine Apertura {} champions", champ.name, state.year),
                ));
                out.honours.push(crate::honours::Honour::champion(
                    state.year,
                    crate::honours::ARG_PRIMERA_CHAMPION_HONOUR,
                    "Argentine Apertura",
                    champ.club_id,
                    champ.name.clone(),
                ));
            }
            out.new_phase = Some(ArgPhase::Clausura);
        }
        ArgPhase::Clausura => {
            if !all_played(fixtures, CLAUSURA_COMP_ID) {
                return out;
            }
            if let Some(champ) = champion(state, fixtures, CLAUSURA_COMP_ID) {
                out.news.push((
                    "competition".into(),
                    format!("{} - crowned Argentine Clausura {} champions", champ.name, state.year + 1),
                ));
                out.honours.push(crate::honours::Honour::champion(
                    state.year + 1,
                    crate::honours::ARG_PRIMERA_CHAMPION_HONOUR,
                    "Argentine Clausura",
                    champ.club_id,
                    champ.name.clone(),
                ));
            }
            let releg = relegated_by_promedios(state, fixtures);
            if !releg.is_empty() {
                out.news.push((
                    "competition".into(),
                    format!(
                        "Argentine Primera - {} relegated on promedios (points-average)",
                        releg.join(" and ")
                    ),
                ));
            }
            out.new_phase = Some(ArgPhase::Complete);
        }
        ArgPhase::Complete => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team(id: u32) -> ArgTeam {
        ArgTeam { club_id: id, name: format!("Club{id}"), reputation: (100 - id) as u16 }
    }

    fn state(n: usize) -> ArgPrimeraState {
        ArgPrimeraState {
            year: 2001,
            teams: (0..n as u32).map(team).collect(),
            apertura_start: GameDate { year: 2001, month: 8, day: 10 },
            clausura_start: GameDate { year: 2002, month: 2, day: 10 },
            phase: ArgPhase::Apertura,
            provenance: "test".into(),
        }
    }

    #[test]
    fn single_round_robin_covers_every_pair_once() {
        let rounds = single_round_robin(20);
        assert_eq!(rounds.len(), 19, "20 teams -> 19 rounds");
        assert!(rounds.iter().all(|r| r.len() == 10), "10 matches per round");
        // Every unordered pair appears exactly once.
        let mut seen = std::collections::BTreeSet::new();
        for r in &rounds {
            for &(h, a) in r {
                let key = (h.min(a), h.max(a));
                assert!(seen.insert(key), "pair {key:?} twice");
            }
        }
        assert_eq!(seen.len(), 20 * 19 / 2);
    }

    #[test]
    fn season_generates_both_tournaments() {
        let st = state(20);
        let fx = generate_season_fixtures(&st, 0);
        let ap = fx.iter().filter(|f| f.competition_id == APERTURA_COMP_ID).count();
        let cl = fx.iter().filter(|f| f.competition_id == CLAUSURA_COMP_ID).count();
        assert_eq!(ap, 190); // 19 rounds x 10
        assert_eq!(cl, 190);
        // Rows are unique.
        let rows: std::collections::BTreeSet<u32> = fx.iter().map(|f| f.row).collect();
        assert_eq!(rows.len(), fx.len());
    }

    #[test]
    fn advance_announces_apertura_then_clausura_and_relegation() {
        let mut st = state(4);
        let mut fx = generate_season_fixtures(&st, 0);
        // Play only the Apertura: Club0 wins everything, Club3 loses everything.
        for f in fx.iter_mut().filter(|f| f.competition_id == APERTURA_COMP_ID) {
            f.status = HeadlessFixtureStatus::Played;
            let (h, a) = (f.home_club_id, f.away_club_id);
            let (hs, as_) = if h < a { (2, 0) } else { (0, 2) };
            f.home_score = Some(hs);
            f.away_score = Some(as_);
        }
        let adv = advance(&st, &fx);
        assert_eq!(adv.new_phase, Some(ArgPhase::Clausura));
        assert_eq!(adv.news.len(), 1);
        assert!(adv.news[0].1.contains("Club0"), "lowest id wins the mini-league");

        // Now Clausura too -> champion + relegation.
        st.phase = ArgPhase::Clausura;
        for f in fx.iter_mut().filter(|f| f.competition_id == CLAUSURA_COMP_ID) {
            f.status = HeadlessFixtureStatus::Played;
            let (h, a) = (f.home_club_id, f.away_club_id);
            let (hs, as_) = if h < a { (2, 0) } else { (0, 2) };
            f.home_score = Some(hs);
            f.away_score = Some(as_);
        }
        let adv = advance(&st, &fx);
        assert_eq!(adv.new_phase, Some(ArgPhase::Complete));
        // champion + relegation news.
        assert_eq!(adv.news.len(), 2);
        assert!(adv.news[1].1.contains("relegated"));
    }
}
