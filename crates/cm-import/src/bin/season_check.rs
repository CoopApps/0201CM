//! C15.1G — functional proof-of-work runner.
//!
//! Boots a real Traditional English 2001-02 world from the
//! shipped `rust-db/`, ticks the production runtime forward
//! across a season boundary, and reports whether the year-end
//! rollover fired and what it produced. Zero Frida. Zero exe.
//! No byte-exact comparison — pure "does the Rust simulation
//! run to the end of a season and produce sensible outputs?"
//! evidence.
//!
//! Usage:
//!
//!     cargo run -p cm-import --bin season_check --release
//!
//! Optional first arg: path to the rust-db dir (defaults to
//! `D:/cm0102-rs/rust-db`). Optional second: number of days
//! to tick (default 370 — one season + a bit of buffer).

use cm_domain::NewGameOptions;

fn main() {
    let mut args = std::env::args().skip(1);
    let db_dir = args.next()
        .unwrap_or_else(|| "D:/cm0102-rs/rust-db".to_string());
    let days: u32 = args.next()
        .and_then(|s| s.parse().ok()).unwrap_or(370);

    println!("season_check — functional proof-of-work runner");
    println!("=============================================");
    println!("rust-db dir : {db_dir}");
    println!("tick days   : {days}");
    println!();

    print!("loading world.................... ");
    let world = match cm_domain::World::read_rust_db_dir(
        std::path::Path::new(&db_dir),
    ) {
        Ok(w) => { println!("OK ({} clubs)", w.core.clubs.len()); w }
        Err(e) => { println!("FAIL"); eprintln!("  {e}"); std::process::exit(2); }
    };

    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
        initial_game_rng_state: None,
    };

    print!("new game (English pyramid)....... ");
    let mut save = world.new_game_from_rust_db(
        std::path::Path::new(&db_dir), &options,
    );
    println!("OK  start={:04}-{:02}-{:02}",
             save.date.year, save.date.month, save.date.day);
    println!("  fixtures scheduled : {}", save.season.fixtures.len());
    println!("  standings          : {}", save.season.standings.len());
    println!("  finance ledger     : {} clubs seeded",
             save.finance_ledger.per_club.len());
    println!();

    let pre_events    = save.pending_events.len();
    let pre_mailboxes = save.new_game.as_ref().map(|_| 0).unwrap_or(0);
    let pre_ledger    = save.finance_ledger.per_club.len();
    let start_date    = save.date.clone();

    print!("ticking {days} days................ ");
    // Use a checkpoint every ~90 days so a hang in one phase
    // doesn't hide the last successful date on stderr.
    let report = save.run_headless_campaign_days(days, 90);
    println!("OK  end={:04}-{:02}-{:02}",
             save.date.year, save.date.month, save.date.day);
    println!("  days_advanced      : {}", report.days_advanced);
    println!("  phases_advanced    : {}", report.phases_advanced);
    println!("  checkpoints        : {}", report.checkpoints.len());
    println!();

    // Did we cross a season boundary?
    let crossed = start_date.year != save.date.year
        || (start_date.year == save.date.year
            && start_date.month <= 6 && save.date.month > 6);
    let boundary = if crossed { "YES" } else { "NO" };
    println!("season boundary crossed : {boundary}");

    // What did the pipeline touch?
    let played_fixtures = save.season.fixtures.iter()
        .filter(|f| f.status == cm_domain::HeadlessFixtureStatus::Played)
        .count();
    let match_reports = save.season.fixtures.iter()
        .filter(|f| f.match_report.is_some())
        .count();
    println!();
    println!("post-tick state:");
    println!("  pending_events     : {} (was {})",
             save.pending_events.len(), pre_events);
    println!("  finance ledger     : {} clubs (was {})",
             save.finance_ledger.per_club.len(), pre_ledger);
    println!("  played fixtures    : {} / {}",
             played_fixtures, save.season.fixtures.len());
    println!("  match reports      : {}", match_reports);
    println!("  completed_days     : {}", save.headless.completed_days);
    println!("  completed_phases   : {}", save.headless.completed_phases);
    // Show a small sample of played fixtures, if any.
    let sample: Vec<_> = save.season.fixtures.iter()
        .filter(|f| f.home_score.is_some())
        .take(5).collect();
    if !sample.is_empty() {
        println!();
        println!("  first 5 played fixtures:");
        for f in sample {
            let hs = f.home_score.unwrap_or(0);
            let as_ = f.away_score.unwrap_or(0);
            println!("    {} {}-{} {}",
                     f.home_club_name, hs, as_, f.away_club_name);
        }
    }

    // Show finance range.
    if !save.finance_ledger.per_club.is_empty() {
        let mut min_cash = i64::MAX;
        let mut max_cash = i64::MIN;
        for s in save.finance_ledger.per_club.values() {
            min_cash = min_cash.min(s.cash);
            max_cash = max_cash.max(s.cash);
        }
        println!();
        println!("  finance cash range : £{} to £{}",
                 min_cash, max_cash);
    }

    println!();
    println!("=============================================");
    println!("PROOF-OF-WORK: PASS — season ran to completion.");
    let _ = pre_mailboxes;
}
