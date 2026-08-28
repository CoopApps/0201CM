//! Argentine Second Division (Primera B Nacional) — a faithful port of
//! `arg_second.cpp` (VA `0x00406c30..0x0040a43f`, vtable `0x009553d8`).
//! Function decode + names: `reports/carve_rename_map.json` (`arg_second.cpp`).
//!
//! The second tier is a single long championship of the **25 clubs** whose
//! primary competition (`club+0x57`) is the Argentine Second Division (id 64),
//! the field `argsecond_gather_clubs` (`0x00407d10`) filters on. Its champion
//! **promotes to the Primera**; the bottom clubs go down on **promedios**
//! (points-per-game average). This mirrors the [`crate::arg_primera`] port; it
//! reuses that module's round-robin and team helpers.
//!
//! ## Fidelity notes
//! * **Membership** — exact: the 25 clubs with primary competition id 64.
//! * **Format** — the exe runs a rich 6-stage structure (`[esi+0x2c]=6`: zonal
//!   groups + promotion/final playoffs, `argsecond_build_zonal_stage`
//!   `0x00408210` etc.). This port plays the single round-robin the fixture
//!   template builds (`argsecond_build_fixture_template` `0x00406f40`,
//!   type `0x19`=25 rounds) and awards promotion to its winner; the zonal and
//!   playoff sub-stages are a documented follow-up, not invented.
//! * **Promotion gate** — the exe advances the Second Division only once the
//!   Primera is ready (`argsecond_advance_stage` `0x00408120`); here promotion
//!   is simply announced when the tournament completes.

use serde::{Deserialize, Serialize};

use crate::arg_primera::{clubs_in_division, single_round_robin, ArgTeam};
use crate::{
    CmPackedDate, DomainOpaqueRecord, GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture,
};

/// `club_competition` id of the Argentine Second Division (the exe's `0x9bbb18`
/// membership filter).
pub const ARG_SECOND_COMP_ID: i32 = 64;
/// Runtime comp id the exe assigns the tournament (`argsecond_build_fixture_template`).
pub const ARG_SECOND_RUNTIME_COMP_ID: u32 = 0x659;
pub const ARG_SECOND_TEAM_COUNT: usize = 25;
/// Clubs relegated per season by promedios.
pub const ARG_SECOND_RELEGATION_COUNT: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgSecondState {
    pub year: u16,
    pub teams: Vec<ArgTeam>,
    pub start_date: GameDate,
    pub complete: bool,
    pub provenance: String,
}

/// The 25 Argentine Second Division clubs.
pub fn argentine_second_clubs(clubs: &[DomainOpaqueRecord]) -> Vec<ArgTeam> {
    clubs_in_division(clubs, ARG_SECOND_COMP_ID)
}

/// Generate the Second Division fixtures (single round-robin, one round/week).
pub fn generate_fixtures(state: &ArgSecondState, mut next_row: u32) -> Vec<HeadlessSeasonFixture> {
    let base = CmPackedDate::from_game_date(state.start_date.clone());
    let mut fixtures = Vec::new();
    for (round, pairs) in single_round_robin(state.teams.len()).into_iter().enumerate() {
        let date = base.add_days((round as i16) * 7).to_game_date();
        for (h, a) in pairs {
            let home = &state.teams[h];
            let away = &state.teams[a];
            fixtures.push(HeadlessSeasonFixture {
                row: next_row,
                competition_id: ARG_SECOND_RUNTIME_COMP_ID,
                competition_name: "Argentine Primera B Nacional".to_string(),
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
                    "Argentine Primera B Nacional {} round {} (argsecond_build_fixture_template 0x00406f40)",
                    state.year,
                    round + 1
                ),
            });
            next_row += 1;
        }
    }
    fixtures
}

fn table(fixtures: &[HeadlessSeasonFixture]) -> std::collections::BTreeMap<u32, (i32, i32, i32)> {
    let mut tab = std::collections::BTreeMap::new();
    for f in fixtures
        .iter()
        .filter(|f| f.competition_id == ARG_SECOND_RUNTIME_COMP_ID)
    {
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

fn all_played(fixtures: &[HeadlessSeasonFixture]) -> bool {
    let mut any = false;
    for f in fixtures
        .iter()
        .filter(|f| f.competition_id == ARG_SECOND_RUNTIME_COMP_ID)
    {
        any = true;
        if f.status != HeadlessFixtureStatus::Played {
            return false;
        }
    }
    any
}

/// The result of one advancement check.
#[derive(Debug, Default)]
pub struct ArgSecondAdvance {
    pub news: Vec<(String, String)>,
    /// Championship honours awarded this step (port of argentina_awards.cpp).
    pub honours: Vec<crate::honours::Honour>,
    pub completed: bool,
}

/// Announce the champion (promoted to the Primera) and the promedios relegations
/// when the tournament finishes. Pure; idempotent until then.
pub fn advance(state: &ArgSecondState, fixtures: &[HeadlessSeasonFixture]) -> ArgSecondAdvance {
    let mut out = ArgSecondAdvance::default();
    if state.complete || !all_played(fixtures) {
        return out;
    }
    let tab = table(fixtures);
    // Champion: most points, then goal difference, then reputation, then id.
    let ranked = |worst_first: bool| -> Vec<&ArgTeam> {
        let mut v: Vec<&ArgTeam> = state.teams.iter().filter(|t| tab.contains_key(&t.club_id)).collect();
        v.sort_by(|a, b| {
            let ta = tab[&a.club_id];
            let tb = tab[&b.club_id];
            let ord = ta
                .0
                .cmp(&tb.0)
                .then(ta.2.cmp(&tb.2))
                .then(a.reputation.cmp(&b.reputation))
                .then(b.club_id.cmp(&a.club_id));
            if worst_first { ord } else { ord.reverse() }
        });
        v
    };
    if let Some(champ) = ranked(false).first() {
        out.news.push((
            "competition".into(),
            format!("{} - win the Argentine Primera B Nacional {} and promote to the Primera", champ.name, state.year),
        ));
        out.honours.push(crate::honours::Honour::champion(
            state.year,
            crate::honours::ARG_SECOND_CHAMPION_HONOUR,
            "Argentine Primera B Nacional",
            champ.club_id,
            champ.name.clone(),
        ));
    }
    let releg: Vec<String> = ranked(true)
        .into_iter()
        .take(ARG_SECOND_RELEGATION_COUNT)
        .map(|t| t.name.clone())
        .collect();
    if !releg.is_empty() {
        out.news.push((
            "competition".into(),
            format!(
                "Argentine Primera B Nacional - {} relegated on promedios",
                releg.join(" and ")
            ),
        ));
    }
    out.completed = true;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(n: usize) -> ArgSecondState {
        ArgSecondState {
            year: 2001,
            teams: (0..n as u32)
                .map(|i| ArgTeam { club_id: i, name: format!("B{i}"), reputation: (100 - i) as u16 })
                .collect(),
            start_date: GameDate { year: 2001, month: 8, day: 10 },
            complete: false,
            provenance: "test".into(),
        }
    }

    #[test]
    fn twenty_five_teams_single_round_robin() {
        let st = state(25);
        let fx = generate_fixtures(&st, 0);
        // 25 teams (odd) -> 25 rounds, 12 matches each = 300 fixtures.
        assert_eq!(fx.len(), 25 * 24 / 2);
        assert!(fx.iter().all(|f| f.competition_id == ARG_SECOND_RUNTIME_COMP_ID));
    }

    #[test]
    fn advance_promotes_champion_and_relegates() {
        let st = state(6);
        let mut fx = generate_fixtures(&st, 0);
        for f in &mut fx {
            f.status = HeadlessFixtureStatus::Played;
            let (h, a) = (f.home_club_id, f.away_club_id);
            let (hs, as_) = if h < a { (2, 0) } else { (0, 2) };
            f.home_score = Some(hs);
            f.away_score = Some(as_);
        }
        let adv = advance(&st, &fx);
        assert!(adv.completed);
        assert_eq!(adv.news.len(), 2);
        assert!(adv.news[0].1.contains("B0") && adv.news[0].1.contains("promote"));
        assert!(adv.news[1].1.contains("relegated"));
    }
}
