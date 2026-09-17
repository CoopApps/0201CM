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
