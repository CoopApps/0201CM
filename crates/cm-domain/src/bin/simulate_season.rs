//! Run a full CM 00/01 season end-to-end using only already-ported Rust code.
//!
//! Loads `rust-db`, boots a New Game with England selected, ticks the ported
//! phase pump (`RuntimeSaveGame::tick_cm_phase`, the port of
//! `FUN_005B6A90`) for ~300 in-game days, then prints the final Premiership
//! table and FA Cup winner.
//!
//! Wraps the daily tick in `catch_unwind` so a subsystem crash surfaces as
//! "died on day D in phase P" instead of a silent panic.

use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::time::Instant;

use cm_domain::{
    honours::Honour,
    simple_league::SimpleLeagueState,
    HeadlessFixtureStatus, HeadlessSeasonFixture, NewGameOptions, RuntimeSaveGame, World,
};

/// Real Premiership competition id (comp_id 7 in shipped comp_id.dat).
const PREM_COMP_ID: i32 = 7;
/// FA Cup competition id.
const FA_CUP_COMP_ID: i32 = 351;
/// How many in-game days to advance (rough season length; kickoff → mid-May).
const DAYS_TO_TICK: u32 = 300;

fn main() {
    let rust_db = Path::new("D:/cm0102-rs/rust-db");
    if !rust_db.exists() {
        eprintln!("rust-db not found at {}; aborting.", rust_db.display());
        std::process::exit(2);
    }

    let load_started = Instant::now();
    let world = match World::read_rust_db_dir(rust_db) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("read_rust_db_dir({}) failed: {e}", rust_db.display());
            std::process::exit(2);
        }
    };
    println!(
        "Loaded rust-db in {:.2}s",
        load_started.elapsed().as_secs_f32()
    );

    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
    };
    let boot_started = Instant::now();
    let mut save = world.new_game_from_rust_db(rust_db, &options);
    println!(
        "Booted new game in {:.2}s: date={:04}-{:02}-{:02}, fixtures={}, simple_leagues={}, domestic_cups={}",
        boot_started.elapsed().as_secs_f32(),
        save.date.year, save.date.month, save.date.day,
        save.season.fixtures.len(),
        save.simple_leagues.len(),
        save.domestic_cups.len(),
    );

    // Find the Premiership state so we know its runtime_comp_id (fixtures
    // are stored under `simple_league::RUNTIME_BASE + real_comp_id`).
    let prem: Option<SimpleLeagueState> = save
        .simple_leagues
        .iter()
        .find(|s| s.real_comp_id == PREM_COMP_ID)
        .cloned();
    match &prem {
        Some(p) => println!(
            "Top division: {} ({} clubs, runtime comp={:#x})",
            p.name, p.teams.len(), p.runtime_comp_id
        ),
        None => eprintln!(
            "WARN: no SimpleLeagueState with real_comp_id={PREM_COMP_ID}; England not wired?",
        ),
    }
    let fa_cup_runtime =
        save.domestic_cups.iter().find(|c| c.real_comp_id == FA_CUP_COMP_ID).map(|c| c.runtime_comp_id);
    if fa_cup_runtime.is_none() {
        eprintln!("WARN: no CupState with real_comp_id={FA_CUP_COMP_ID}");
    }

    // Tick day-by-day, chunked so we can pinpoint the crash day if any
    // subsystem panics. Every 30 days we print a progress line with the
    // matches-played count against the top-division runtime id.
    let sim_started = Instant::now();
    let mut ok_days: u32 = 0;
    let mut died_at: Option<(u32, u8, String)> = None;
    while ok_days < DAYS_TO_TICK {
        let day_no = ok_days + 1;
        let start_phase = save.simulation.phase;
        let start_date = save.date.clone();
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            // A "day" in the ported pump is 3 phase ticks.
            for _ in 0..3 {
                save.tick_cm_phase();
            }
        }));
        match result {
            Ok(()) => {
                ok_days += 1;
                if ok_days % 30 == 0 {
                    let played = if let Some(p) = &prem {
                        played_count(&save.season.fixtures, p.runtime_comp_id)
                    } else {
                        0
                    };
                    println!(
                        "Day {ok_days:>3}: date={:04}-{:02}-{:02}, Prem matches played = {played}",
                        save.date.year, save.date.month, save.date.day
                    );
                }
            }
            Err(payload) => {
                let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "<non-string panic payload>".to_string()
                };
                eprintln!(
                    "\n*** DIED on day {day_no} (in-game date {:04}-{:02}-{:02}, phase {start_phase}) ***",
                    start_date.year, start_date.month, start_date.day
                );
                eprintln!("panic: {msg}");
                died_at = Some((day_no, start_phase, msg));
                break;
            }
        }
    }
    println!(
        "Ticked {ok_days} day(s) in {:.2}s (elapsed_days on save={})",
        sim_started.elapsed().as_secs_f32(),
        save.elapsed_days,
    );

    // Final report.
    if let Some(p) = &prem {
        print_prem_table(p, &save.season.fixtures);
    }
    if let Some(rt) = fa_cup_runtime {
        print_fa_cup_status(&save, rt);
    }

    if let Some((day, phase, msg)) = died_at {
        eprintln!(
            "\nSubsystem-crash summary: day {day}, phase {phase}, first panic message: {msg}"
        );
        std::process::exit(1);
    }
}

fn played_count(fixtures: &[HeadlessSeasonFixture], runtime_comp_id: u32) -> usize {
    fixtures
        .iter()
        .filter(|f| f.competition_id == runtime_comp_id && f.status == HeadlessFixtureStatus::Played)
        .count()
}

fn print_prem_table(state: &SimpleLeagueState, fixtures: &[HeadlessSeasonFixture]) {
    // Aggregate P/W/D/L/GF/GA/Pts for the top division from played fixtures.
    let mut table: Vec<Row> = state
        .teams
        .iter()
        .map(|t| Row {
            club_id: t.club_id,
            name: t.name.clone(),
            reputation: t.reputation,
            ..Row::default()
        })
        .collect();
    let index: std::collections::HashMap<u32, usize> =
        table.iter().enumerate().map(|(i, r)| (r.club_id, i)).collect();
    let mut any_played = false;
    for f in fixtures
        .iter()
        .filter(|f| f.competition_id == state.runtime_comp_id)
    {
        if f.status != HeadlessFixtureStatus::Played {
            continue;
        }
        let (hs, as_) = match (f.home_score, f.away_score) {
            (Some(h), Some(a)) => (h as i32, a as i32),
            _ => continue,
        };
        any_played = true;
        if let Some(&i) = index.get(&f.home_club_id) {
            let r = &mut table[i];
            r.played += 1;
            r.gf += hs;
            r.ga += as_;
            match hs.cmp(&as_) {
                std::cmp::Ordering::Greater => {
                    r.won += 1;
                    r.pts += 3;
                }
                std::cmp::Ordering::Less => r.lost += 1,
                std::cmp::Ordering::Equal => {
                    r.drawn += 1;
                    r.pts += 1;
                }
            }
        }
        if let Some(&i) = index.get(&f.away_club_id) {
            let r = &mut table[i];
            r.played += 1;
            r.gf += as_;
            r.ga += hs;
            match as_.cmp(&hs) {
                std::cmp::Ordering::Greater => {
                    r.won += 1;
                    r.pts += 3;
                }
                std::cmp::Ordering::Less => r.lost += 1,
                std::cmp::Ordering::Equal => {
                    r.drawn += 1;
                    r.pts += 1;
                }
            }
        }
    }
    println!("\n=== {} {} ===", state.name, state.year);
    if !any_played {
        println!("  (no matches played yet)");
        return;
    }
    table.sort_by(|a, b| {
        b.pts
            .cmp(&a.pts)
            .then((b.gf - b.ga).cmp(&(a.gf - a.ga)))
            .then(b.gf.cmp(&a.gf))
            .then(b.reputation.cmp(&a.reputation))
            .then(a.club_id.cmp(&b.club_id))
    });
    println!(
        "{:>2}  {:<28} {:>3} {:>3} {:>3} {:>3} {:>4} {:>4} {:>4} {:>4}",
        "#", "Club", "P", "W", "D", "L", "GF", "GA", "GD", "Pts"
    );
    for (i, r) in table.iter().enumerate() {
        println!(
            "{:>2}. {:<28} {:>3} {:>3} {:>3} {:>3} {:>4} {:>4} {:>+4} {:>4}",
            i + 1,
            r.name,
            r.played,
            r.won,
            r.drawn,
            r.lost,
            r.gf,
            r.ga,
            r.gf - r.ga,
            r.pts
        );
    }
}

fn print_fa_cup_status(save: &RuntimeSaveGame, runtime_comp_id: u32) {
    // FA Cup winner: look up the champion honour whose competition matches.
    let winners: Vec<&Honour> = save
        .honours
        .iter()
        .filter(|h| h.competition == "English FA Cup")
        .collect();
    println!("\n=== English FA Cup ===");
    if let Some(w) = winners.last() {
        println!("Winner {}: {} (honour_id={:#x})", w.year, w.club_name, w.honour_id);
    } else {
        // Not decided — report how far it got.
        let played = save
            .season
            .fixtures
            .iter()
            .filter(|f| {
                f.competition_id == runtime_comp_id && f.status == HeadlessFixtureStatus::Played
            })
            .count();
        let scheduled = save
            .season
            .fixtures
            .iter()
            .filter(|f| f.competition_id == runtime_comp_id)
            .count();
        let cup = save
            .domestic_cups
            .iter()
            .find(|c| c.runtime_comp_id == runtime_comp_id);
        let round = cup.map(|c| c.round).unwrap_or(0);
        println!(
            "Winner: (not yet decided) — round {round} reached, {played}/{scheduled} ties played"
        );
    }
}

#[derive(Default)]
struct Row {
    club_id: u32,
    name: String,
    reputation: u16,
    played: u32,
    won: u32,
    drawn: u32,
    lost: u32,
    gf: i32,
    ga: i32,
    pts: u32,
}

// Simple smoke test: at least confirm the boot half of the binary runs and
// finds the Premiership wired up. This exercises `new_game_from_rust_db`
// end-to-end for the "England selected" path.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn england_boot_wires_premiership() {
        let rust_db = Path::new("D:/cm0102-rs/rust-db");
        if !rust_db.exists() {
            eprintln!("rust-db not present; skipping");
            return;
        }
        let world = World::read_rust_db_dir(rust_db).expect("read rust-db");
        let opts = NewGameOptions {
            selected_nations: vec!["England".to_string()],
            background_nations: vec![],
            use_real_players: true,
            attribute_masking: true,
            start_year: 2001,
        };
        let save = world.new_game_from_rust_db(rust_db, &opts);
        assert!(
            save.simple_leagues.iter().any(|s| s.real_comp_id == PREM_COMP_ID),
            "Premiership (comp 7) must be wired as a SimpleLeagueState",
        );
        assert!(
            save.domestic_cups.iter().any(|c| c.real_comp_id == FA_CUP_COMP_ID),
            "FA Cup (comp 351) must be wired as a CupState",
        );
    }
}
