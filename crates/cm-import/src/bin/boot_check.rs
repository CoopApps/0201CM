//! C15.1G — minimal boot-check bisector.
//!
//! Three step-timed phases with immediate stdout flush after each,
//! so if any single phase hangs we know which one.
//!
//!   1. read_rust_db_dir              — load the shipped native DB
//!   2. new_game_from_rust_db         — run the new-game init
//!   3. run_headless_days(30)         — tick a month
//!
//! Usage:
//!     cargo run -p cm-import --bin boot_check --release [--] [days]
//!
//! `days` defaults to 30 (one month). Pass 370 for a full season
//! once the boot chain is verified.
use std::io::Write;
use std::time::Instant;

fn tick(msg: &str) {
    println!("{msg}");
    let _ = std::io::stdout().flush();
}

fn main() {
    let days: u32 = std::env::args().nth(1)
        .and_then(|s| s.parse().ok()).unwrap_or(30);
    let dir = "D:/cm0102-rs/rust-db";

    tick("=== boot_check ===");
    tick(&format!("rust-db : {dir}"));
    tick(&format!("days    : {days}"));

    let t0 = Instant::now();
    tick("[1/3] read_rust_db_dir...");
    let world = match cm_domain::World::read_rust_db_dir(
        std::path::Path::new(dir),
    ) {
        Ok(w) => w,
        Err(e) => { tick(&format!("      FAIL: {e}")); std::process::exit(2); }
    };
    tick(&format!("      OK  {:.1}s   clubs={} nations={}",
                  t0.elapsed().as_secs_f64(),
                  world.core.clubs.len(),
                  world.core.nations.len()));

    let t1 = Instant::now();
    tick("[2/3] new_game_from_rust_db (England only)...");
    let options = cm_domain::NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
        initial_game_rng_state: None,
    };
    let mut save = world.new_game_from_rust_db(
        std::path::Path::new(dir), &options,
    );
    tick(&format!("      OK  {:.1}s   start={}-{:02}-{:02}  fixtures={}",
                  t1.elapsed().as_secs_f64(),
                  save.date.year, save.date.month, save.date.day,
                  save.season.fixtures.len()));

    let t2 = Instant::now();
    tick(&format!("[3/3] tick_days({days})..."));
    save.tick_days(days);
    tick(&format!("      OK  {:.1}s   end={}-{:02}-{:02}",
                  t2.elapsed().as_secs_f64(),
                  save.date.year, save.date.month, save.date.day));

    let played = save.season.fixtures.iter()
        .filter(|f| f.status == cm_domain::HeadlessFixtureStatus::Played)
        .count();
    tick(&format!("      played fixtures : {} / {}",
                  played, save.season.fixtures.len()));
    tick(&format!("      finance ledger  : {} clubs",
                  save.finance_ledger.per_club.len()));
    for n in save.notes.iter().filter(|n| n.contains("regen")) {
        tick(&format!("      {n}"));
    }
    tick(&format!("=== boot_check TOTAL: {:.1}s ===",
                  t0.elapsed().as_secs_f64()));
}
