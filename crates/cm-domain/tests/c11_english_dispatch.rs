//! C11 — production-path integration test for the exact English
//! Traditional fixture dispatch.
//!
//! Crosses the REAL production entry
//! `World::generate_new_game_season`. Constructs a minimal `World`
//! containing:
//!
//!   * the 5 English competition records (7, 8, 9, 10, 93);
//!   * the requisite stadium records (with alt-stadium cross-links);
//!   * 114 raw club records (20 + 24 + 24 + 24 + 22), each with the
//!     correct division_id at +0x57 and stadium_id at +0x69.
//!
//! Asserts:
//!   1. Each of the five leagues produces its exact fixture count
//!      (Premier 380 / First 552 / Second 552 / Third 552 / Conference 462).
//!   2. Every fixture in the five leagues came from the exact
//!      English engine (`source` string check).
//!   3. Total fixtures 2498 across the five.
//!   4. Negative dispatch: a non-English league (Sweden Premier 38)
//!      + a feeder pool (Isthmian 358) + a bucket (357) route
//!      through the fallback path (their `source` does NOT match
//!      the exact-engine string), or produce no fixtures at all.
//!   5. Subset selection: with only Premier + First selected the
//!      dispatch still produces 380 + 552 = 932 exact-engine
//!      fixtures.

use cm_domain::english_traditional::{
    english_dispatch_decision, is_english_traditional_league,
    EnglishFixtureDispatch, GameMode, ENGLISH_TRADITIONAL_COMP_IDS,
};
use cm_domain::game_rng::{GameRng, GameRngState};
use cm_domain::{
    ClubView, CoreBook, CoreSummary, DomainCompetition, DomainOpaqueRecord,
    DomainStadium, NewGameOptions, ReferenceBook, ReferenceSummary,
    SchemaBook, StaffBook, StaffSummary, World,
};
use std::collections::BTreeSet;

const EXACT_ENGINE_MARKER: &str = "cm0102-gdi exact English engine";

/// Build a raw 0x245-byte club record with the fields the dispatch
/// path reads: `+0x00` id (u32), `+0x04..+0x37` primary name,
/// `+0x37` set flag `0xff`, `+0x57` division_id (i32),
/// `+0x69` stadium_id (i32).
fn raw_club(id: u32, name: &str, division_id: i32, stadium_id: i32) -> DomainOpaqueRecord {
    let mut raw = vec![0u8; ClubView::RECORD_SIZE];
    raw[0..4].copy_from_slice(&id.to_le_bytes());
    let name_bytes = name.as_bytes();
    let take = name_bytes.len().min(51);
    raw[4..4 + take].copy_from_slice(&name_bytes[..take]);
    raw[0x37] = 0xff;
    raw[0x57..0x5B].copy_from_slice(&division_id.to_le_bytes());
    // Secondary/tertiary comp slots at 0x5b/0x60 stay 0 (i.e. -0 = 0,
    // which is a valid comp id via `id_opt` only when != -2; we keep
    // -1 so competition_ids only reports the primary division).
    raw[0x5b..0x5f].copy_from_slice(&(-1_i32).to_le_bytes());
    raw[0x60..0x64].copy_from_slice(&(-1_i32).to_le_bytes());
    raw[0x69..0x6D].copy_from_slice(&stadium_id.to_le_bytes());
    DomainOpaqueRecord {
        ordinal: id,
        id,
        primary_name: Some(name.to_string()),
        secondary_name: None,
        short_name: None,
        text_candidates: Vec::new(),
        raw,
    }
}

fn eng_comp(id: u32, name: &str) -> DomainCompetition {
    DomainCompetition {
        id,
        long_name: name.to_string(),
        short_name: name.to_string(),
        three_letter_name: name.chars().take(3).collect(),
        scope: 2,
        nation_id: 46,
        last_division: -1,
        reserve_division: -1,
        reputation: 100,
        unknown_tail: Vec::new(),
    }
}

fn stadium(id: u32, alt: Option<i32>) -> DomainStadium {
    DomainStadium {
        id,
        name: format!("Stadium-{id}"),
        alt_stadium_id: alt,
        ..Default::default()
    }
}

fn build_world_with_five_english_leagues() -> World {
    let mut clubs: Vec<DomainOpaqueRecord> = Vec::new();
    let mut stadiums: Vec<DomainStadium> = Vec::new();

    let mut next_club_id: u32 = 10_000;
    let mut next_stadium_id: u32 = 20_000;

    for (comp_id, n_clubs, comp_name) in [
        (7u32, 20usize, "English Premier Division"),
        (8, 24, "English First Division"),
        (9, 24, "English Second Division"),
        (10, 24, "English Third Division"),
        (93, 22, "English Conference"),
    ] {
        for i in 0..n_clubs {
            let cid = next_club_id;
            next_club_id += 1;
            let sid = next_stadium_id;
            next_stadium_id += 1;
            clubs.push(raw_club(
                cid,
                &format!("{comp_name}-Club-{i}"),
                comp_id as i32,
                sid as i32,
            ));
            stadiums.push(stadium(sid, None));
        }
        let _ = comp_name;
    }

    // Two derby pairs: give one pair of clubs in Premier a shared
    // stadium (E2), and one pair in First a rival cross-link (E3).
    // The engine handles empty-crosslink cases fine; we exercise
    // the resolver path opportunistically here.
    if clubs.len() >= 2 {
        // Two Prem clubs share stadium 20_000.
        let sid_shared = 20_000i32;
        let v = &mut clubs[1].raw;
        v[0x69..0x6D].copy_from_slice(&sid_shared.to_le_bytes());
    }
    if stadiums.len() >= 22 {
        // Two First-Div stadiums as rivals.
        stadiums[20].alt_stadium_id = Some(21_i32.wrapping_add(20_000));
        stadiums[21].alt_stadium_id = Some(20_i32.wrapping_add(20_000));
    }

    // Add a non-English comp (Sweden Premier 38) + one feeder-pool
    // comp (Isthmian 358) + a bucket comp (357) + 3 clubs in each,
    // to give the negative-dispatch test something to see.
    let extras: [(u32, usize, &str); 3] = [
        (38u32, 20, "Sweden Premier"),
        (358, 10, "English Northern Premier (feeder)"),
        (357, 8,  "A Lower Division (bucket)"),
    ];
    for (comp_id, n_clubs, name) in extras.iter() {
        for i in 0..*n_clubs {
            let cid = next_club_id;
            next_club_id += 1;
            let sid = next_stadium_id;
            next_stadium_id += 1;
            clubs.push(raw_club(cid, &format!("{name}-{i}"), *comp_id as i32, sid as i32));
            stadiums.push(stadium(sid, None));
        }
    }

    // Competition records for the 5 English + 3 extras.
    let club_competitions = vec![
        eng_comp(7,   "English Premier Division"),
        eng_comp(8,   "English First Division"),
        eng_comp(9,   "English Second Division"),
        eng_comp(10,  "English Third Division"),
        eng_comp(93,  "English Conference"),
        eng_comp(38,  "Sweden Premier Division"),
        eng_comp(358, "English Northern Premier"),
        eng_comp(357, "A Lower Division"),
    ];

    World {
        base_data: Vec::new(),
        save: None,
        schema: SchemaBook::default(),
        core: CoreBook {
            clubs,
            nat_clubs: Vec::new(),
            colours: Vec::new(),
            continents: Vec::new(),
            nations: Vec::new(),
        },
        core_summary: CoreSummary::default(),
        references: ReferenceBook {
            cities: Vec::new(),
            officials: Vec::new(),
            first_names: Vec::new(),
            second_names: Vec::new(),
            common_names: Vec::new(),
            stadiums,
            staff_competitions: Vec::new(),
            club_competitions,
            nation_competitions: Vec::new(),
            staff_history: Vec::new(),
            staff_comp_history: Vec::new(),
            club_comp_history: Vec::new(),
            nation_comp_history: Vec::new(),
        },
        reference_summary: ReferenceSummary::default(),
        staff: StaffBook::default(),
        staff_summary: StaffSummary::default(),
        contracts: None,
        squad_numbers: Default::default(),
    }
}

#[test]
fn all_five_english_leagues_route_through_exact_engine() {
    let world = build_world_with_five_english_leagues();
    let comp_ids: BTreeSet<u32> = ENGLISH_TRADITIONAL_COMP_IDS.iter().copied().collect();
    let (fixtures, _proofs, _standings) =
        world.generate_new_game_season(&comp_ids, 2001);

    // Per-league fixture counts. This test crosses the real
    // production dispatch (`World::generate_new_game_season`).
    let mut per_comp: std::collections::BTreeMap<u32, usize> =
        std::collections::BTreeMap::new();
    for f in &fixtures {
        *per_comp.entry(f.competition_id).or_default() += 1;
    }
    assert_eq!(per_comp.get(&7).copied().unwrap_or(0), 380, "Premier");
    assert_eq!(per_comp.get(&8).copied().unwrap_or(0), 552, "First");
    assert_eq!(per_comp.get(&9).copied().unwrap_or(0), 552, "Second");
    assert_eq!(per_comp.get(&10).copied().unwrap_or(0), 552, "Third");
    assert_eq!(per_comp.get(&93).copied().unwrap_or(0), 462, "Conference");

    // Total across the 5 = 2498.
    let total_eng: usize = per_comp
        .iter()
        .filter(|(id, _)| is_english_traditional_league(**id))
        .map(|(_, n)| *n)
        .sum();
    assert_eq!(total_eng, 380 + 552 + 552 + 552 + 462);

    // Every English fixture must carry the exact-engine source string.
    for f in fixtures.iter().filter(|f| is_english_traditional_league(f.competition_id)) {
        assert!(
            f.source.contains(EXACT_ENGINE_MARKER),
            "English fixture for comp {} must come from exact engine, got source: {:?}",
            f.competition_id, f.source
        );
    }
}

#[test]
fn non_english_and_feeder_leagues_do_not_use_the_exact_engine() {
    let world = build_world_with_five_english_leagues();
    // Include Sweden Premier (38), Isthmian feeder (358), and bucket
    // (357) alongside the 5 English leagues.
    let comp_ids: BTreeSet<u32> = [7, 8, 9, 10, 93, 38, 358, 357]
        .iter().copied().collect();
    let (fixtures, _proofs, _standings) =
        world.generate_new_game_season(&comp_ids, 2001);

    // Bucket comp 357 is filtered out by MAX_LEAGUE_CLUBS (it only
    // has 8 clubs in the fixture, well within the cap, so it WILL
    // generate fixtures on the fallback path — but MUST NOT carry
    // the exact-engine source string).
    // Feeder 358 (10 clubs) same: fallback path, non-exact.
    // Sweden Premier (38, 20 clubs): fallback path — non-exact.
    for (id, label) in [(38u32, "Sweden"), (358, "Isthmian feeder"), (357, "bucket")] {
        let saw_any = fixtures.iter().any(|f| f.competition_id == id);
        let saw_exact = fixtures.iter().any(|f|
            f.competition_id == id && f.source.contains(EXACT_ENGINE_MARKER)
        );
        assert!(!saw_exact,
            "comp {} ({label}) must NOT route through the exact English engine",
            id
        );
        // saw_any is informational — the fallback path may or may
        // not produce fixtures depending on other filters.
        let _ = saw_any;
    }
}

#[test]
fn subset_selection_only_dispatches_selected_leagues() {
    let world = build_world_with_five_english_leagues();
    // Premier + First only.
    let comp_ids: BTreeSet<u32> = [7u32, 8].iter().copied().collect();
    let (fixtures, _proofs, _standings) =
        world.generate_new_game_season(&comp_ids, 2001);

    let mut per_comp: std::collections::BTreeMap<u32, usize> = Default::default();
    for f in &fixtures {
        *per_comp.entry(f.competition_id).or_default() += 1;
    }
    assert_eq!(per_comp.get(&7).copied().unwrap_or(0), 380);
    assert_eq!(per_comp.get(&8).copied().unwrap_or(0), 552);
    assert_eq!(per_comp.get(&9).copied().unwrap_or(0), 0,
               "Second not selected — must produce zero fixtures");
    assert_eq!(per_comp.get(&10).copied().unwrap_or(0), 0,
               "Third not selected");
    assert_eq!(per_comp.get(&93).copied().unwrap_or(0), 0,
               "Conference not selected");
    // Both selected leagues must have gone through the exact engine.
    for f in fixtures.iter().filter(|f| f.competition_id == 7 || f.competition_id == 8) {
        assert!(f.source.contains(EXACT_ENGINE_MARKER));
    }
}

#[test]
fn c11_1_shared_rng_state_advances_across_selected_leagues_only() {
    // C11.1 point 12: verify one shared RNG threads through the
    // production dispatch, advances only for GENERATED leagues,
    // and each subsequent league's state resumes from the prior's.
    let world = build_world_with_five_english_leagues();
    // All 5 selected — the exe-order chain: Prem -> First -> Second
    // -> Third -> Conf.
    let comp_ids: BTreeSet<u32> = ENGLISH_TRADITIONAL_COMP_IDS.iter().copied().collect();
    let mut rng_all = GameRng::new(0xC110_ABCDu32);
    let state_before_all = rng_all.snapshot();
    let (_fixtures_all, _, _) =
        world.generate_new_game_season_with_rng(&comp_ids, 2001, &mut rng_all);
    let state_after_all = rng_all.snapshot();
    assert_ne!(state_before_all, state_after_all,
        "RNG must advance when the 5 English leagues generate");

    // Subset (Prem + First only) starts from the SAME initial state
    // must advance LESS than the full chain — proves skipped leagues
    // don't consume RNG.
    let mut rng_two = GameRng::new(0xC110_ABCDu32);
    let comp_ids_two: BTreeSet<u32> = [7u32, 8].iter().copied().collect();
    let (_, _, _) =
        world.generate_new_game_season_with_rng(&comp_ids_two, 2001, &mut rng_two);
    let state_after_two = rng_two.snapshot();
    assert_ne!(state_after_two, state_after_all,
        "subset selection must reach a DIFFERENT final RNG state \
         than full selection (skipped leagues must not silently \
         consume their would-have-been random calls)");

    // Prem-only starts from same seed but produces yet-another state.
    let mut rng_one = GameRng::new(0xC110_ABCDu32);
    let comp_ids_one: BTreeSet<u32> = [7u32].iter().copied().collect();
    let (_, _, _) =
        world.generate_new_game_season_with_rng(&comp_ids_one, 2001, &mut rng_one);
    let state_after_one = rng_one.snapshot();
    assert_ne!(state_after_one, state_after_two);
    assert_ne!(state_after_one, state_after_all);
}

#[test]
fn c11_1_captured_rng_state_can_be_injected_into_production() {
    // C11.1 point 11: prove GameRngState can round-trip through the
    // real production dispatch. We inject a pinned state, run the
    // real season builder, and confirm the RNG advanced (not
    // reset) and the fixture engine produced the expected counts.
    // (Byte-exact-vs-captured-GDI requires matching P1 club_ids —
    // shipped rust-db uses different numeric ids than the captured
    // build; a rust-db renumbering issue, out of C11.1 scope. P1
    // NAME ordering does match, verified separately via
    // captured_p1.json.)
    let world = build_world_with_five_english_leagues();
    let comp_ids: BTreeSet<u32> = ENGLISH_TRADITIONAL_COMP_IDS.iter().copied().collect();
    let pinned = GameRngState { cursor: 1992, jitter: 26340, lcg_state: 1726273615 , dbc340_cli_seed: 0 };
    let mut rng = GameRng::from_state_snapshot(pinned);
    let before = rng.snapshot();
    assert_eq!(before, pinned);
    let (fixtures, _, _) =
        world.generate_new_game_season_with_rng(&comp_ids, 2001, &mut rng);
    assert_eq!(fixtures.len(), 2498);
    assert_ne!(rng.snapshot(), pinned, "RNG must have advanced");
}

#[test]
fn c11_1_bootstrap_from_options_matches_direct_from_state() {
    // Confirm NewGameOptions::bootstrap_game_rng() honors
    // initial_game_rng_state, and skipping it reproducibly derives
    // one from options (no invented magic constants).
    let pinned = GameRngState { cursor: 100, jitter: 200, lcg_state: 300 , dbc340_cli_seed: 0 };
    let opts_pinned = NewGameOptions {
        initial_game_rng_state: Some(pinned),
        ..NewGameOptions::default()
    };
    let opts_derived_a = NewGameOptions {
        selected_nations: vec!["England".into()],
        ..NewGameOptions::default()
    };
    let opts_derived_b = NewGameOptions {
        selected_nations: vec!["England".into()],
        ..NewGameOptions::default()
    };
    let opts_derived_c = NewGameOptions {
        selected_nations: vec!["France".into()],
        ..NewGameOptions::default()
    };
    assert_eq!(opts_pinned.bootstrap_game_rng().snapshot(), pinned);
    let da = opts_derived_a.bootstrap_game_rng().snapshot();
    let db = opts_derived_b.bootstrap_game_rng().snapshot();
    let dc = opts_derived_c.bootstrap_game_rng().snapshot();
    assert_eq!(da, db, "same options -> same bootstrap state");
    assert_ne!(da, dc, "different selected_nations -> different bootstrap state");
}

#[test]
fn c11_1_dispatch_v4_never_routes_to_exact_english_engine() {
    // C11.1 point 9: prove V4 mode is blocked from the exact engine
    // by the dispatcher itself, not by surrounding assumptions.
    for &id in &[7u32, 8, 9, 10, 93] {
        let d = english_dispatch_decision(GameMode::V4, id, 2001);
        assert!(matches!(d, EnglishFixtureDispatch::Skipped { .. }),
            "V4 comp {id} routed as {d:?} — must be Skipped");
    }
    // And Traditional continues to route them exact.
    for &id in &[7u32, 8, 9, 10, 93] {
        assert_eq!(
            english_dispatch_decision(GameMode::Traditional, id, 2001),
            EnglishFixtureDispatch::ExactEnglish
        );
    }
}

#[test]
fn c11_1_wrong_club_count_panics_no_silent_fallback() {
    // C11.1 point 7: build a world where Prem (7) has only 15
    // clubs. The dispatcher routes it to ExactEnglish (Traditional +
    // 2001), the engine returns UnexpectedClubCount, and the
    // production season builder MUST panic rather than silently
    // fall through to Berger.
    let mut world = build_world_with_five_english_leagues();
    // Remove 5 clubs from Prem (comp 7). Use raw division_id at +0x57.
    let mut removed = 0;
    world.core.clubs.retain(|r| {
        let cv = ClubView::new(r);
        if cv.division_id() == Some(7) && removed < 5 {
            removed += 1;
            false
        } else {
            true
        }
    });
    let comp_ids: BTreeSet<u32> = [7u32].iter().copied().collect();
    let mut rng = GameRng::new(0);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = world.generate_new_game_season_with_rng(&comp_ids, 2001, &mut rng);
    }));
    assert!(result.is_err(),
        "production dispatch must panic (not silently fall back) on a \
         Traditional English roster-shape violation");
}

#[test]
fn c11_1_unsupported_base_year_skipped_by_dispatch() {
    // C11.1 point 14: non-2001 base years fall to Skipped and the
    // production dispatch does not run the exact engine at all.
    let world = build_world_with_five_english_leagues();
    let comp_ids: BTreeSet<u32> = ENGLISH_TRADITIONAL_COMP_IDS.iter().copied().collect();
    let mut rng = GameRng::new(0);
    let (fixtures, _, _) =
        world.generate_new_game_season_with_rng(&comp_ids, 2002, &mut rng);
    // None of the fixtures should be the exact-engine source string
    // (the exact engine was skipped; the generic Berger builder may
    // still produce fixtures for these ids in fallback mode).
    assert!(!fixtures.iter().any(|f| f.source.contains(EXACT_ENGINE_MARKER)),
        "base_year 2002 must NOT route through the exact engine — \
         no proven templates for non-2001 seasons.");
}

#[test]
fn top_four_without_conference() {
    let world = build_world_with_five_english_leagues();
    let comp_ids: BTreeSet<u32> = [7u32, 8, 9, 10].iter().copied().collect();
    let (fixtures, _proofs, _standings) =
        world.generate_new_game_season(&comp_ids, 2001);

    let mut per_comp: std::collections::BTreeMap<u32, usize> = Default::default();
    for f in &fixtures {
        *per_comp.entry(f.competition_id).or_default() += 1;
    }
    assert_eq!(per_comp.get(&7).copied().unwrap_or(0), 380);
    assert_eq!(per_comp.get(&8).copied().unwrap_or(0), 552);
    assert_eq!(per_comp.get(&9).copied().unwrap_or(0), 552);
    assert_eq!(per_comp.get(&10).copied().unwrap_or(0), 552);
    assert_eq!(per_comp.get(&93).copied().unwrap_or(0), 0);
}
