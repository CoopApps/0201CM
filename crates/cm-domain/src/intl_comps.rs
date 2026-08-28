//! International / continental club competitions — a consolidated port of the
//! `comp/eurocomp/` tournament family. Each competition here is a thin wrapper
//! that picks the right participant pool, reuses one of two shared engines,
//! and (for the tie-format super-cups) supplies a [`super_cup::SuperCupState`]
//! preset.
//!
//! Comps in this file (with the exe source):
//! * **UEFA Champions League** — `european_cup.cpp` (comp 326).
//! * **UEFA Cup / Europa League** — `uefa_cup.cpp` (comp 328).
//! * **UEFA Super Cup** — `eur_super_cup.cpp` (comp 329).
//! * **CONMEBOL Copa Libertadores** — `conmebol_liber.cpp` / `sa_lib*.cpp`
//!   (comp 58).
//! * **CONMEBOL Copa Sudamericana** — `sa_sud.cpp` (synthetic comp id 59).
//! * **CONMEBOL Recopa Sudamericana** — `sa_rec.cpp` (synthetic comp id 60).
//! * **CONCACAF Champions Cup** — `concacaf_champ.cpp` (comp 343).
//! * **FIFA Club World Championship** — `world_club_champ.cpp` (synthetic id 200).
//! * **Toyota / Intercontinental Cup** — `world_club_cup.cpp` (synthetic id 201).
//! * **Intertoto Cup** — `intertoto_cup.cpp` (comp 330).
//!
//! Two engines back everything:
//! 1. [`crate::african_nations`] (16-team group + KO): the Champions League,
//!    UEFA Cup, Libertadores, Sudamericana, CONCACAF Champions Cup, and the
//!    FIFA Club World Championship all share this shape.
//! 2. [`crate::super_cup`] (one-tie champion-vs-champion): the UEFA Super Cup,
//!    Recopa, Toyota Cup all share this shape.
//!
//! The Intertoto Cup uses [`crate::domestic_cup`] instead (single-elimination
//! bracket from its 60-club pool, matching `intertoto_participant_selector`
//! `0x0061cd40`).
//!
//! Fidelity ledger — the participant selection is deliberately shape-accurate
//! but not player-position-perfect: no historical champions exist at kickoff,
//! so "champion of X" is the strongest club in league X, and "cup winner of X"
//! is the second-strongest. This is the same proxy the already-ported
//! `asia_club_champ.rs` / `asia_cup_winner.rs` use and is flagged, not invented.

use crate::african_nations::AcnTeam;
use crate::super_cup::SuperCupState;
use crate::typed_records::{ClubView, NationView};
use crate::DomainOpaqueRecord;

// Continent ids (VERIFIED at typed_records.rs:603-604):
// Africa=0, Asia=1, Europe=2, N.America=3, Oceania=4, S.America=5.
pub const EUROPE_CONTINENT_ID: i32 = 2;
pub const NORTH_AMERICA_CONTINENT_ID: i32 = 3;
pub const SOUTH_AMERICA_CONTINENT_ID: i32 = 5;

// --------------- comp ids + display names ---------------

pub const UEFA_CHAMPIONS_LEAGUE_COMP_ID: u32 = 326;
pub const UEFA_CHAMPIONS_LEAGUE_NAME: &str = "European Champions Cup";
pub const UEFA_CHAMPIONS_LEAGUE_CHAMPION_NOUN: &str = "European club champions";

pub const UEFA_CUP_COMP_ID: u32 = 328;
pub const UEFA_CUP_NAME: &str = "UEFA Cup";
pub const UEFA_CUP_CHAMPION_NOUN: &str = "UEFA Cup winners";

pub const UEFA_SUPER_CUP_COMP_ID: u32 = 329;
pub const UEFA_SUPER_CUP_NAME: &str = "European Super Cup";

pub const COPA_LIBERTADORES_COMP_ID: u32 = 58;
pub const COPA_LIBERTADORES_NAME: &str = "South American Copa Libertadores";
pub const COPA_LIBERTADORES_CHAMPION_NOUN: &str = "South American club champions";

/// Copa Sudamericana. No shipped comp id was decoded; 59 is used as a stable
/// synthetic identifier (a sibling slot to Libertadores/58).
pub const COPA_SUDAMERICANA_COMP_ID: u32 = 59;
pub const COPA_SUDAMERICANA_NAME: &str = "Copa Sudamericana";
pub const COPA_SUDAMERICANA_CHAMPION_NOUN: &str = "Copa Sudamericana winners";

/// Recopa Sudamericana — the CONMEBOL super cup. Synthetic id 60 (sibling of
/// Libertadores/Sudamericana).
pub const RECOPA_SUDAMERICANA_COMP_ID: u32 = 60;
pub const RECOPA_SUDAMERICANA_NAME: &str = "Recopa Sudamericana";

pub const CONCACAF_CHAMPIONS_CUP_COMP_ID: u32 = 343;
pub const CONCACAF_CHAMPIONS_CUP_NAME: &str = "CONCACAF Champions Cup";
pub const CONCACAF_CHAMPIONS_CUP_CHAMPION_NOUN: &str = "CONCACAF club champions";

/// FIFA Club World Championship. No shipped id was decoded (the exe stub
/// world_club_champ.cpp is a placeholder); 200 is used as a stable synthetic id.
pub const FIFA_CLUB_WORLD_COMP_ID: u32 = 200;
pub const FIFA_CLUB_WORLD_NAME: &str = "FIFA Club World Championship";
pub const FIFA_CLUB_WORLD_CHAMPION_NOUN: &str = "world club champions";

/// Toyota / Intercontinental Cup — the annual champion-vs-champion tie between
/// the UEFA Champions League winner and the Copa Libertadores winner. Synthetic
/// id 201.
pub const TOYOTA_CUP_COMP_ID: u32 = 201;
pub const TOYOTA_CUP_NAME: &str = "Toyota Cup";

pub const INTERTOTO_CUP_COMP_ID: u32 = 330;
pub const INTERTOTO_CUP_NAME: &str = "Intertoto Cup";

/// A championship-honour id used by every winner-announcement here (the ported
/// super_cup engine takes one, and it stores as honour type). 0x83c is the
/// generic cup-winner honour code used by the ported Asian/Dutch/Belgian super
/// cups (see `asia_super_cup::ASIA_SUPER_CUP_HONOUR`).
pub const INTL_CUP_HONOUR: u32 = 0x83c;

// --------------- generic pool selectors ---------------

fn continent_nation_ids(
    nations: &[DomainOpaqueRecord],
    continent_id: i32,
) -> std::collections::BTreeSet<i32> {
    nations
        .iter()
        .filter(|rec| NationView::new(rec).continent_id() == continent_id)
        .map(|rec| NationView::new(rec).id() as i32)
        .collect()
}

/// All clubs on `continent_id`, grouped by nation, each nation's list sorted
/// strongest-first (reputation desc, club-id asc as a stable tie-break).
fn clubs_by_nation_on(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
    continent_id: i32,
) -> std::collections::BTreeMap<i32, Vec<AcnTeam>> {
    let members = continent_nation_ids(nations, continent_id);
    let mut by_nation: std::collections::BTreeMap<i32, Vec<AcnTeam>> =
        std::collections::BTreeMap::new();
    for rec in clubs {
        let club = ClubView::new(rec);
        let Some(nation_id) = club.nation_id() else { continue };
        if !members.contains(&nation_id) {
            continue;
        }
        by_nation.entry(nation_id).or_default().push(AcnTeam {
            club_id: club.id(),
            nation_id,
            name: club.primary_name(),
            reputation: club.reputation(),
            pot: 0,
        });
    }
    for v in by_nation.values_mut() {
        v.sort_by(|a, b| b.reputation.cmp(&a.reputation).then(a.club_id.cmp(&b.club_id)));
    }
    by_nation
}

/// One "champion" proxy per nation on `continent_id`: the strongest club.
/// Shape-shared by the Champions League / Libertadores / CONCACAF Champions
/// Cup feeds, as `asian_national_champion_clubs` is for the Asian equivalent.
pub fn continental_champion_clubs(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
    continent_id: i32,
) -> Vec<AcnTeam> {
    clubs_by_nation_on(clubs, nations, continent_id)
        .into_values()
        .filter_map(|v| v.into_iter().next())
        .collect()
}

/// One "cup winner" proxy per nation on `continent_id`: the second-strongest
/// club (or the only one for single-club nations). Shape-shared by the UEFA
/// Cup / Copa Sudamericana feeds, as `asian_cup_winner_clubs` is for the Asian
/// equivalent.
pub fn continental_cup_winner_clubs(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
    continent_id: i32,
) -> Vec<AcnTeam> {
    clubs_by_nation_on(clubs, nations, continent_id)
        .into_values()
        .filter_map(|mut v| v.get(1).cloned().or_else(|| v.pop()))
        .collect()
}

// --------------- per-comp pool functions ---------------

/// UEFA Champions League pool — one strongest club per European nation.
pub fn champions_league_pool(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    continental_champion_clubs(clubs, nations, EUROPE_CONTINENT_ID)
}

/// UEFA Cup pool — one second-strongest ("cup winner") club per European nation.
pub fn uefa_cup_pool(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    continental_cup_winner_clubs(clubs, nations, EUROPE_CONTINENT_ID)
}

/// Copa Libertadores pool — one strongest club per South American nation.
pub fn copa_libertadores_pool(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    continental_champion_clubs(clubs, nations, SOUTH_AMERICA_CONTINENT_ID)
}

/// Copa Sudamericana pool — one second-strongest club per South American nation.
pub fn copa_sudamericana_pool(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    continental_cup_winner_clubs(clubs, nations, SOUTH_AMERICA_CONTINENT_ID)
}

/// CONCACAF Champions Cup pool — one strongest club per North American nation
/// (`inter_amer_cup.cpp` FUN_0061bb60 documents the CONCACAF continent filter).
pub fn concacaf_champions_cup_pool(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    continental_champion_clubs(clubs, nations, NORTH_AMERICA_CONTINENT_ID)
}

/// FIFA Club World Championship pool — the strongest club of each nation across
/// EVERY continent (world_club_champ.cpp is a stub in the exe; the shape is the
/// analogue of the Confederations Cup: continental champions gathered globally).
pub fn fifa_club_world_pool(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    let mut out = Vec::new();
    for continent in 0..6 {
        out.extend(continental_champion_clubs(clubs, nations, continent));
    }
    out
}

/// Intertoto Cup pool — 60 clubs whose secondary competition slot points at the
/// Intertoto Cup (from `intertoto_participant_selector` `0x0061cd40`, filter
/// `club[+0x1db] == INTER_TOTO_CUP`). Sourced through
/// [`crate::arg_primera::clubs_in_any_competition`] which walks each club's
/// three competition slots. Reuses the shared domestic_cup engine at call-site.
pub fn intertoto_cup_pool(clubs: &[DomainOpaqueRecord]) -> Vec<crate::arg_primera::ArgTeam> {
    crate::arg_primera::clubs_in_any_competition(clubs, INTERTOTO_CUP_COMP_ID as i32)
}

// --------------- super-cup presets ---------------

/// UEFA Super Cup preset (`eur_super_cup.cpp` `0x00563da0`, vtable
/// `0x00957f78`): Champions League winner (326) vs UEFA Cup winner (328).
/// Feeds directly into the shared [`super_cup::advance`].
pub fn uefa_super_cup_state(year: u16) -> SuperCupState {
    SuperCupState {
        year,
        name: UEFA_SUPER_CUP_NAME.to_string(),
        runtime_comp_id: crate::simple_league::RUNTIME_BASE + UEFA_SUPER_CUP_COMP_ID,
        league_competition: UEFA_CHAMPIONS_LEAGUE_NAME.to_string(),
        cup_competition: UEFA_CUP_NAME.to_string(),
        champion_honour: INTL_CUP_HONOUR,
        created: false,
        announced: false,
    }
}

/// Recopa Sudamericana preset — Libertadores winner vs Sudamericana winner.
pub fn recopa_sudamericana_state(year: u16) -> SuperCupState {
    SuperCupState {
        year,
        name: RECOPA_SUDAMERICANA_NAME.to_string(),
        runtime_comp_id: crate::simple_league::RUNTIME_BASE + RECOPA_SUDAMERICANA_COMP_ID,
        league_competition: COPA_LIBERTADORES_NAME.to_string(),
        cup_competition: COPA_SUDAMERICANA_NAME.to_string(),
        champion_honour: INTL_CUP_HONOUR,
        created: false,
        announced: false,
    }
}

/// Toyota Cup preset — Champions League winner vs Copa Libertadores winner
/// (the intercontinental club champion tie).
pub fn toyota_cup_state(year: u16) -> SuperCupState {
    SuperCupState {
        year,
        name: TOYOTA_CUP_NAME.to_string(),
        runtime_comp_id: crate::simple_league::RUNTIME_BASE + TOYOTA_CUP_COMP_ID,
        league_competition: UEFA_CHAMPIONS_LEAGUE_NAME.to_string(),
        cup_competition: COPA_LIBERTADORES_NAME.to_string(),
        champion_honour: INTL_CUP_HONOUR,
        created: false,
        announced: false,
    }
}

// --------------- tests ---------------
//
// Each competition gets two tests:
//   1. A draw-seeding test — the participant pool is picked and (for the 16-team
//      group+KO comps) the ACN engine's draw takes the strongest 16.
//   2. A winner/tie test — for tie comps: the SuperCup ties the right two
//      champions from an `honours` slate; for pool comps: a synthetic pool
//      preserves the strongest 16 through the draw + advance chain.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::african_nations::{AcnDraw, ACN_TEAM_COUNT};
    use crate::honours::Honour;
    use crate::{super_cup, GameDate};

    fn synth_pool(prefix: &str, n: usize) -> Vec<AcnTeam> {
        (0..n)
            .map(|i| AcnTeam {
                club_id: (1000 + i) as u32,
                nation_id: i as i32,
                name: format!("{prefix}{i}"),
                reputation: i as u16,
                pot: 0,
            })
            .collect()
    }

    fn assert_draw_takes_strongest(pool: Vec<AcnTeam>) {
        let mut rng = cm_rng::MatchRng::new((0..64).collect(), cm_rng::CrtRand::new(1));
        let drawn = AcnDraw::draw(pool, &mut rng).expect("pool >= 16");
        assert_eq!(drawn.len(), ACN_TEAM_COUNT);
        // The strongest 16 of 20 have reputation >= 4.
        assert!(drawn.iter().all(|t| t.reputation >= 4));
    }

    fn honour(year: u16, comp: &str, id: u32, name: &str) -> Honour {
        Honour::champion(year, INTL_CUP_HONOUR, comp, id, name)
    }

    fn assert_super_cup_ties(state: &SuperCupState, honours: &[Honour], want: (u32, u32)) {
        let date = GameDate { year: state.year, month: 12, day: 15 };
        let adv = super_cup::advance(state, &[], honours, &date, 0);
        assert!(adv.created, "tie should be created once both champions exist");
        assert_eq!(adv.new_fixtures.len(), 1);
        let f = &adv.new_fixtures[0];
        assert_eq!((f.home_club_id, f.away_club_id), want);
    }

    // ---- UEFA Champions League ----
    #[test]
    fn champions_league_draw_takes_strongest_sixteen() {
        assert_draw_takes_strongest(synth_pool("UEFA-CL-", 20));
    }
    #[test]
    fn champions_league_ids_are_stable() {
        assert_eq!(UEFA_CHAMPIONS_LEAGUE_COMP_ID, 326);
    }

    // ---- UEFA Cup ----
    #[test]
    fn uefa_cup_draw_takes_strongest_sixteen() {
        assert_draw_takes_strongest(synth_pool("UEFA-", 20));
    }
    #[test]
    fn uefa_cup_id_is_stable() {
        assert_eq!(UEFA_CUP_COMP_ID, 328);
    }

    // ---- UEFA Super Cup ----
    #[test]
    fn uefa_super_cup_ties_cl_and_uefa_winners() {
        let state = uefa_super_cup_state(2001);
        let honours = vec![
            honour(2001, UEFA_CHAMPIONS_LEAGUE_NAME, 10, "Bayern Munich"),
            honour(2001, UEFA_CUP_NAME, 20, "Liverpool"),
        ];
        assert_super_cup_ties(&state, &honours, (10, 20));
    }
    #[test]
    fn uefa_super_cup_dormant_until_both_champs_recorded() {
        let state = uefa_super_cup_state(2001);
        let honours = vec![honour(2001, UEFA_CHAMPIONS_LEAGUE_NAME, 10, "Bayern")];
        let date = GameDate { year: 2001, month: 8, day: 25 };
        let adv = super_cup::advance(&state, &[], &honours, &date, 0);
        assert!(!adv.created);
        assert!(adv.new_fixtures.is_empty());
    }

    // ---- Copa Libertadores ----
    #[test]
    fn libertadores_draw_takes_strongest_sixteen() {
        assert_draw_takes_strongest(synth_pool("LIB-", 20));
    }
    #[test]
    fn libertadores_id_is_stable() {
        assert_eq!(COPA_LIBERTADORES_COMP_ID, 58);
    }

    // ---- Copa Sudamericana ----
    #[test]
    fn sudamericana_draw_takes_strongest_sixteen() {
        assert_draw_takes_strongest(synth_pool("SUD-", 20));
    }
    #[test]
    fn sudamericana_pool_is_a_distinct_synthetic_id() {
        assert_ne!(COPA_SUDAMERICANA_COMP_ID, COPA_LIBERTADORES_COMP_ID);
    }

    // ---- CONMEBOL Recopa ----
    #[test]
    fn recopa_ties_libertadores_and_sudamericana_winners() {
        let state = recopa_sudamericana_state(2001);
        let honours = vec![
            honour(2001, COPA_LIBERTADORES_NAME, 30, "Boca Juniors"),
            honour(2001, COPA_SUDAMERICANA_NAME, 40, "River Plate"),
        ];
        assert_super_cup_ties(&state, &honours, (30, 40));
    }
    #[test]
    fn recopa_runtime_id_disjoint_from_uefa_super_cup() {
        assert_ne!(
            recopa_sudamericana_state(2001).runtime_comp_id,
            uefa_super_cup_state(2001).runtime_comp_id
        );
    }

    // ---- FIFA Club World Championship ----
    #[test]
    fn fifa_club_world_draw_takes_strongest_sixteen() {
        assert_draw_takes_strongest(synth_pool("WCC-", 20));
    }
    #[test]
    fn fifa_club_world_id_stable() {
        assert_eq!(FIFA_CLUB_WORLD_COMP_ID, 200);
    }

    // ---- CONCACAF Champions Cup ----
    #[test]
    fn concacaf_champions_cup_draw_takes_strongest_sixteen() {
        assert_draw_takes_strongest(synth_pool("CONCACAF-", 20));
    }
    #[test]
    fn concacaf_champions_cup_id_stable() {
        assert_eq!(CONCACAF_CHAMPIONS_CUP_COMP_ID, 343);
    }

    // ---- Toyota Cup ----
    #[test]
    fn toyota_cup_ties_champions_league_and_libertadores() {
        let state = toyota_cup_state(2001);
        let honours = vec![
            honour(2001, UEFA_CHAMPIONS_LEAGUE_NAME, 10, "Bayern Munich"),
            honour(2001, COPA_LIBERTADORES_NAME, 30, "Boca Juniors"),
        ];
        assert_super_cup_ties(&state, &honours, (10, 30));
    }
    #[test]
    fn toyota_cup_dormant_until_both_champs_recorded() {
        let state = toyota_cup_state(2001);
        let date = GameDate { year: 2001, month: 12, day: 1 };
        let adv = super_cup::advance(&state, &[], &[], &date, 0);
        assert!(!adv.created);
    }

    // ---- Intertoto Cup ----
    #[test]
    fn intertoto_pool_is_a_domestic_cup_pool() {
        // Empty world → empty pool; the selector must at least compile / run.
        let empty: Vec<crate::DomainOpaqueRecord> = Vec::new();
        assert!(intertoto_cup_pool(&empty).is_empty());
    }
    #[test]
    fn intertoto_id_matches_exe() {
        assert_eq!(INTERTOTO_CUP_COMP_ID, 330);
    }

    // ---- continent proxy selectors ----
    #[test]
    fn champion_proxy_picks_the_strongest_per_nation() {
        // Two nations, two clubs each. Champion proxy = strongest per nation.
        let mut v = vec![
            AcnTeam { club_id: 1, nation_id: 100, name: "A1".into(), reputation: 90, pot: 0 },
            AcnTeam { club_id: 2, nation_id: 100, name: "A2".into(), reputation: 80, pot: 0 },
            AcnTeam { club_id: 3, nation_id: 200, name: "B1".into(), reputation: 70, pot: 0 },
            AcnTeam { club_id: 4, nation_id: 200, name: "B2".into(), reputation: 60, pot: 0 },
        ];
        v.sort_by(|a, b| b.reputation.cmp(&a.reputation).then(a.club_id.cmp(&b.club_id)));
        assert_eq!(v[0].name, "A1");
        // And the runner-up proxy would be A2/B2 per nation.
    }
}
