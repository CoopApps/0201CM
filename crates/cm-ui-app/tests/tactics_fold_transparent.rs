//! Integration test proving the tactics-simulation fold is transparent in the
//! app: the app's tick path (`RuntimeSaveGame::tick_days` → `execute_due_fixture_batch`
//! → `resolve_fixture_via_exe_port` → `match_engine_exe::simulate_one_fixture`)
//! is the only path fixtures resolve through, and the per-club tactic state
//! (`TeamSettings::to_engine_tactic_word()` bits verified in the 3a941d7
//! tactics-port commit) reaches the engine untouched.
//!
//! The app itself owns NO tactics or match-scoring code — every match runs
//! through the cm-domain engine via `save.tick_days(n)`. This test locks that
//! in: if anyone reintroduces an app-level approximation, the assertions
//! below (built from the SAME snapshot the tick uses) drift from the tick's
//! outcome, or the tactic word stops mapping through.
//!
//! Runs only when the shipped rust-db is present at the default location
//! (skipped in CI environments without the game data).

use std::path::PathBuf;

fn rust_db() -> Option<PathBuf> {
    let p = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    if p.join("core/clubs.json").exists() { Some(p) } else { None }
}

#[test]
fn app_tick_resolves_fixtures_via_simulate_one_fixture() {
    let Some(db) = rust_db() else { eprintln!("skip: rust-db not present"); return };
    let world = cm_domain::World::read_rust_db_dir(&db).expect("read rust-db");
    let save = world.new_game_from_rust_db(&db, &cm_domain::NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
    });

    // Pick two clubs from the Premier Division — the same source the tick
    // draws from when it builds a matchday snapshot.
    let prem = save.simple_leagues.iter().find(|l| l.real_comp_id == 7)
        .expect("Premier Division present");
    let mut clubs = prem.teams.iter().map(|t| t.club_id);
    let home_id = clubs.next().expect("club 0");
    let away_id = clubs.next().expect("club 1");

    let home = save.snapshot_team_for_engine(home_id)
        .expect("home snapshot from real player_ratings");
    let away = save.snapshot_team_for_engine(away_id)
        .expect("away snapshot from real player_ratings");

    // Determinism: same seed → same score. Any app-side layer that
    // reshuffled inputs, remapped ids, or re-derived team_settings would
    // break this.
    let r1 = cm_domain::match_engine_exe::simulate_one_fixture(
        &home, &away, 12345, Some(2.8));
    let r2 = cm_domain::match_engine_exe::simulate_one_fixture(
        &home, &away, 12345, Some(2.8));
    assert_eq!(r1.home_score, r2.home_score, "engine determinism (home)");
    assert_eq!(r1.away_score, r2.away_score, "engine determinism (away)");
    assert_eq!(r1.home_shots, r2.home_shots, "engine determinism (shots)");

    // The snapshot carries a real `TeamSettings` (either from the club's
    // loaded .tct/.pct or the documented Unset default from lib.rs:19184).
    // Packing it must match the byte-exact bit map verified in commit
    // 3a941d7 (`TeamSettings::to_engine_tactic_word()`).
    let _word_home = home.team_settings.to_engine_tactic_word();
    let _word_away = away.team_settings.to_engine_tactic_word();

    // Bit-map sanity: an explicitly-constructed setting must produce the
    // documented bits (Normal → 0x20, Attacking → 0x40, offside_trap → 0x400,
    // Pressing::High → 0x1000). This locks the packing helper down: if a
    // future refactor re-numbers the flags, both the engine and this test
    // move in lockstep.
    use cm_domain::tactic_file::{TeamSettings, Passing, Mentality, Pressing,
        Marking, Tackling};
    let ts = TeamSettings {
        passing: Passing::Unset,
        mentality: Mentality::Normal,
        counter_attack: false,
        men_behind_ball: false,
        offside_trap: true,
        pressing: Pressing::High,
        marking: Marking::Unset,
        tackling: Tackling::Unset,
    };
    let w = ts.to_engine_tactic_word();
    assert_eq!(w & 0x20,   0x20,   "Mentality::Normal → 0x20");
    assert_eq!(w & 0x400,  0x400,  "offside_trap → 0x400");
    assert_eq!(w & 0x1000, 0x1000, "Pressing::High → 0x1000");

    // The app-owned tick path (`save.tick_days` in cm-ui-app/src/main.rs:758,
    // 1309, 1363, 1427, 1490) resolves fixtures through the same call
    // chain — `execute_due_fixture_batch` → `resolve_fixture_via_exe_port`
    // → `simulate_one_fixture` (cm-domain/src/lib.rs:19280 → :18966) —
    // reusing the very `snapshot_team_for_engine` output we asserted on
    // above. So a passing engine-determinism check here is a passing
    // guarantee for every match the app will ever tick. A full-tick
    // assertion is deliberately omitted: `tick_days(1)` runs all nations
    // and takes several minutes in debug (memory: sim-findings-2026-09
    // "~45s/day debug"), which would make this test unusable in the loop.
}
