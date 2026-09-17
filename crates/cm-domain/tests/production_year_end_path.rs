//! Production-path integration proofs for the Traditional English
//! year-end (Phase A / Phase Q of the integration audit).
//!
//! These tests deliberately do NOT call the C15 helpers directly. They
//! run the sequence the playable app runs:
//!
//! ```text
//!   read_rust_db_dir              (load master DB)
//!   run_start_game_init           (FUN_008120D0 — player init, squad
//!                                  numbers, contract pool)
//!   new_game_from_rust_db         (fixtures, finance seed, regen)
//!   should_fire_english_year_end  (end-of-season detector)
//!   run_english_year_end          (compute_annual_rollover →
//!                                  apply_report_to_world)
//! ```
//!
//! and assert on what actually landed on `World` / `RuntimeSaveGame`.
//!
//! Each test skips (rather than fails) when `rust-db` is absent, so the
//! suite still runs on a machine without the database.

use cm_domain::{NewGameOptions, World};
use std::path::{Path, PathBuf};

fn db_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()),
    )
}

fn rng_table(db: &Path) -> PathBuf {
    db.join("config/rng_table.bin")
}

/// The app's exact boot sequence, up to a started game.
fn boot_england() -> Option<(World, cm_domain::RuntimeSaveGame)> {
    let db = db_dir();
    if !db.exists() {
        eprintln!("skipping — rust-db not at {db:?}");
        return None;
    }
    let mut world = World::read_rust_db_dir(&db).expect("read rust-db");
    // The step the app skipped for a whole session (see
    // `App::ensure_world_initialised`).
    world.run_start_game_init(Some(&rng_table(&db)));
    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        ..Default::default()
    };
    let save = world.new_game_from_rust_db(&db, &options);
    Some((world, save))
}

/// The boot init pass must actually produce the contract pool, because
/// three shipped tranches (C15.1B clause writes, C15.1C squad-position
/// writes, C15.1E person-news appends) all early-return without it.
#[test]
fn start_game_init_builds_the_contract_pool() {
    let db = db_dir();
    if !db.exists() {
        eprintln!("skipping — rust-db not at {db:?}");
        return;
    }
    let mut world = World::read_rust_db_dir(&db).expect("read rust-db");
    assert!(
        world.contracts.is_none(),
        "the DB load must NOT build the pool — the exe builds it in the \
         post-league-selection init pass, and code that assumes otherwise \
         is what hid this bug"
    );
    world.run_start_game_init(Some(&rng_table(&db)));
    let pool = world
        .contracts
        .as_ref()
        .expect("run_start_game_init must build the contract pool");
    assert!(
        pool.records.len() > 10_000,
        "expected a world-scale contract pool, got {}",
        pool.records.len()
    );
    assert!(
        !world.squad_numbers.is_empty(),
        "run_start_game_init must also assign squad numbers"
    );
}

/// The C13 per-person walk reads a 50-slot person array per club. The
/// runtime builds it from the contract pool; if that build is missing
/// or the pool is absent, every per-person year-end effect silently
/// produces nothing. This proves the slots are derivable for real
/// English clubs after the live boot.
#[test]
fn booted_world_has_person_slots_for_english_clubs() {
    let Some((world, _save)) = boot_england() else { return };
    let pool = world.contracts.as_ref().expect("pool after boot");

    // Count contracts per club for the Premier League's members.
    let members = world.club_members_of_competition(7);
    assert!(!members.is_empty(), "comp 7 must have member clubs");
    let mut with_people = 0usize;
    for (cid, _) in &members {
        let n = pool
            .records
            .iter()
            .filter(|r| r.club_id == *cid as i32)
            .count();
        if n > 0 {
            with_people += 1;
        }
    }
    assert!(
        with_people >= members.len() / 2,
        "expected most comp-7 clubs to have contracted staff; {} of {} did",
        with_people,
        members.len()
    );
}

/// End-to-end: force the season to a finished state and run the live
/// year-end entry point. Asserts the apply layer saw a contract pool —
/// i.e. that the production path reaches the proven C15.1B/C/E code
/// rather than skipping it.
#[test]
fn live_year_end_runs_with_a_contract_pool() {
    let Some((mut world, mut save)) = boot_england() else { return };

    // Drive the season to "all English fixtures played" without
    // simulating (the match batch is far too slow for a test): mark
    // every English fixture Played and move the clock past May.
    let eng = [7u32, 8, 9, 10, 93];
    let mut played = 0usize;
    for f in save.season.fixtures.iter_mut() {
        if eng.contains(&f.competition_id) {
            f.status = cm_domain::HeadlessFixtureStatus::Played;
            played += 1;
        }
    }
    assert!(played > 0, "England boot must schedule English fixtures");
    save.date.month = 6;

    assert!(
        save.should_fire_english_year_end(&save.date.clone()),
        "end-of-season detector must fire once every English fixture is Played"
    );

    let mut rng = cm_domain::game_rng::GameRng::new(0xC15_0000);
    save.run_english_year_end(&mut world, &mut rng);

    // The breadcrumb events tell us which path ran.
    let errs: Vec<&cm_domain::RuntimeEvent> = save
        .pending_events
        .iter()
        .filter(|e| e.kind == "year_end_error")
        .collect();
    for e in &errs {
        eprintln!("[year_end_error] {}", e.message);
    }
    assert!(
        !errs.iter().any(|e| e.message.contains("contract pool not initialised")),
        "the production year-end path must reach the C15.1B/C/E passes \
         with a live contract pool"
    );
    assert!(
        save.pending_events.iter().any(|e| e.kind == "year_end_applied"),
        "year-end must report that it applied"
    );
    assert_eq!(
        save.last_english_year_end_applied,
        Some(save.date.year),
        "year-end must latch so it cannot fire twice in one season"
    );
}

/// Phase E: divisions 1/2/3 promote a FOURTH club through the
/// promotion play-offs.
///
/// `compute_annual_rollover` only propagates a play-off winner when a
/// table row carries `playoff_winner_marker`. `run_english_year_end`
/// used to hardcode that to `false` on every row, so the fourth
/// promotion place was never filled in a real game. This asserts the
/// live path both plays the play-offs and promotes their winner.
#[test]
fn live_year_end_promotes_a_playoff_winner() {
    let Some((mut world, mut save)) = boot_england() else { return };

    let division_of = |w: &World, club_id: u32| -> Option<i32> {
        w.core
            .clubs
            .iter()
            .map(cm_domain::typed_records::ClubView::new)
            .find(|v| v.id() == club_id)
            .and_then(|v| v.division_id())
    };

    let eng = [7u32, 8, 9, 10, 93];
    for f in save.season.fixtures.iter_mut() {
        if eng.contains(&f.competition_id) {
            f.status = cm_domain::HeadlessFixtureStatus::Played;
        }
    }
    save.date.month = 6;

    let mut rng = cm_domain::game_rng::GameRng::new(0xC15_0000);
    save.run_english_year_end(&mut world, &mut rng);

    // One play-off final per division that has a bracket (8, 9, 10).
    let finals: Vec<&cm_domain::RuntimeEvent> = save
        .pending_events
        .iter()
        .filter(|e| e.message.contains("play-offs and are promoted"))
        .collect();
    for e in &finals {
        eprintln!("{}", e.message);
    }
    assert_eq!(
        finals.len(),
        3,
        "divisions 1, 2 and 3 must each resolve a promotion play-off"
    );

    // Each winner must actually be promoted on the live World.
    for e in &finals {
        let cid: u32 = e
            .message
            .split("club #")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse().ok())
            .expect("event names the winning club");
        let now = division_of(&world, cid).expect("winner has a division");
        let comp: i32 = e
            .message
            .split("comp ")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse().ok())
            .expect("event names the comp");
        assert_ne!(
            now, comp,
            "play-off winner {cid} must have LEFT comp {comp}, but is still there"
        );
    }
}

/// Phase D: the season roll must produce a SECOND season of English
/// fixtures.
///
/// `hook_season_roll_scheduler` queues a regen for `year + 1` each
/// Jan 1, and `apply_pending_season_roll_regens` runs the exact engine
/// for it. That engine used to be gated on
/// `EXACT_SUPPORTED_BASE_YEARS == [2001]`, so the 2002 roll returned
/// nothing and the drain skipped it silently — a save could never have a
/// second season of English football. This drives the real drain method.
#[test]
fn season_roll_produces_a_second_season_of_fixtures() {
    let Some((world, mut save)) = boot_england() else { return };

    // A season spans two calendar years (Aug 2001 → May 2002), so count
    // comp-7 fixtures in total rather than by year.
    let total = |s: &cm_domain::RuntimeSaveGame| {
        s.season.fixtures.iter().filter(|f| f.competition_id == 7).count()
    };
    assert_eq!(total(&save), 380, "first season built");

    // Exactly what the Jan-1 hook queues.
    save.pending_season_roll_regens = [(7u32, 2002u16)].into_iter().collect();
    let mut rng = cm_domain::game_rng::GameRng::new(0xC15_0000);
    let applied = save.apply_pending_season_roll_regens(&world, &mut rng, 0);

    assert_eq!(applied, 1, "the 2002 season roll must materialise comp 7");
    assert_eq!(
        total(&save),
        760,
        "comp 7 must hold a full SECOND season alongside the first"
    );
    assert!(
        !save.pending_events.iter().any(|e| e.kind == "season_roll_error"),
        "no league may be left without a schedule"
    );
}

/// Phases F/G/J: the year-end must actually MOVE clubs between
/// divisions on the live World — status stamped (C12), promotion /
/// relegation applied (C7/C8/C13), and the club's own record rewritten
/// (`Club+0x57` current comp, `+0x5B` previous comp, `+0x37` status).
///
/// `ClubView::division_id()` reads `+0x57`, and that is what the league
/// tables and club screens read, so this proves the rollover is visible
/// to the rest of the game rather than confined to a report object.
#[test]
fn live_year_end_moves_clubs_between_divisions() {
    let Some((mut world, mut save)) = boot_england() else { return };

    let division_of = |w: &World, club_id: u32| -> Option<i32> {
        w.core
            .clubs
            .iter()
            .map(cm_domain::typed_records::ClubView::new)
            .find(|v| v.id() == club_id)
            .and_then(|v| v.division_id())
    };

    // Snapshot every English club's division before the rollover.
    let eng = [7u32, 8, 9, 10, 93];
    let mut before: std::collections::BTreeMap<u32, i32> = Default::default();
    for comp in eng {
        for (cid, _) in world.club_members_of_competition(comp) {
            if let Some(d) = division_of(&world, cid) {
                before.insert(cid, d);
            }
        }
    }
    assert!(!before.is_empty(), "England boot must populate the pyramid");

    for f in save.season.fixtures.iter_mut() {
        if eng.contains(&f.competition_id) {
            f.status = cm_domain::HeadlessFixtureStatus::Played;
        }
    }
    save.date.month = 6;

    let mut rng = cm_domain::game_rng::GameRng::new(0xC15_0000);
    save.run_english_year_end(&mut world, &mut rng);

    let moved: Vec<(u32, i32, i32)> = before
        .iter()
        .filter_map(|(cid, old)| {
            division_of(&world, *cid).and_then(|now| {
                if now != *old { Some((*cid, *old, now)) } else { None }
            })
        })
        .collect();

    for (cid, old, now) in moved.iter().take(10) {
        eprintln!("club {cid}: comp {old} -> {now}");
    }
    assert!(
        !moved.is_empty(),
        "the year-end must move at least one club between divisions on the \
         live World — an empty result means the rollover ran but nothing \
         reached Club+0x57"
    );
}
