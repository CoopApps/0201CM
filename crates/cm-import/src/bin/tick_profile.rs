//! Profile the REAL production daily tick.
//!
//! Drives exactly what the playable app drives —
//! `RuntimeSaveGame::tick_days_bound(&mut World, 1)` — after the same
//! boot sequence the app performs, and reports per-day wall time plus a
//! span breakdown.
//!
//! This exists because the project's headline performance figure was
//! measured on `run_headless_days` (which calls `tick_days`, not the
//! World-bound tick the app uses) and was months old.
//!
//! Usage:
//!     cargo run -p cm-import --bin tick_profile --release [--] [days]
//!
//! Set `CM_TICK_PROFILE=1` to enable the span breakdown.

use cm_domain::{NewGameOptions, World};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

fn say(msg: &str) {
    println!("{msg}");
    let _ = std::io::stdout().flush();
}

fn main() {
    let days: u32 = std::env::args().nth(1)
        .and_then(|s| s.parse().ok()).unwrap_or(5);
    let db = PathBuf::from(
        std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));

    say("=== tick_profile — real production daily tick ===");
    say(&format!("rust-db : {}", db.display()));
    say(&format!("days    : {days}"));
    say(&format!("profile : {}", if cm_domain::tick_profile::enabled() {
        "ON" } else { "OFF (set CM_TICK_PROFILE=1)" }));

    let t = Instant::now();
    let mut world = World::read_rust_db_dir(&db).expect("read rust-db");
    say(&format!("[{:>7.1}s] read_rust_db_dir", t.elapsed().as_secs_f64()));

    let t2 = Instant::now();
    world.run_start_game_init(Some(&db.join("config/rng_table.bin")));
    say(&format!("[{:>7.1}s] run_start_game_init", t2.elapsed().as_secs_f64()));

    let t3 = Instant::now();
    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        ..Default::default()
    };
    let mut save = world.new_game_from_rust_db(&db, &options);
    say(&format!("[{:>7.1}s] new_game_from_rust_db", t3.elapsed().as_secs_f64()));
    say(&format!("           fixtures={} standings={} clubs_with_finance={}",
        save.season.fixtures.len(), save.season.standings.len(),
        save.finance.clubs.len()));

    // Warm the span accumulator without counting boot in the per-day table.
    cm_domain::tick_profile::reset();

    let mut day_times: Vec<f64> = Vec::new();
    for d in 0..days {
        let played_before = save.season.fixtures.iter()
            .filter(|f| f.status == cm_domain::HeadlessFixtureStatus::Played)
            .count();
        let t = Instant::now();
        save.tick_days_bound(&mut world, 1);
        let secs = t.elapsed().as_secs_f64();
        day_times.push(secs);
        let played_after = save.season.fixtures.iter()
            .filter(|f| f.status == cm_domain::HeadlessFixtureStatus::Played)
            .count();
        say(&format!(
            "day {:>3}  {}  {:>8.2}s   fixtures played today: {}",
            d + 1, save.date.iso(), secs, played_after - played_before));
    }

    let total: f64 = day_times.iter().sum();
    let mean = total / day_times.len().max(1) as f64;
    let max = day_times.iter().cloned().fold(0.0f64, f64::max);
    say(&format!(
        "\nper-day: mean {:.2}s  max {:.2}s  total {:.1}s over {} day(s)",
        mean, max, total, day_times.len()));
    say(&format!(
        "a 310-day season at this mean = {:.1} minutes ({:.1} hours)",
        mean * 310.0 / 60.0, mean * 310.0 / 3600.0));

    if cm_domain::tick_profile::enabled() {
        say(&cm_domain::tick_profile::report(
            &format!("span breakdown over {} day(s)", days)));
    }
}
