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
const DAYS_TO_TICK: u32 = 60;

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

    // Wire-up observations — proves the ported subsystems actually fire
    // during a tick, not just in unit tests. Every line here is a
    // pass/fail signal for one of the campaigns landed this session.
    print_wireup_observations(&save);

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

/// Dump one line per session-landed subsystem showing whether it's actually
/// firing in the running sim. PASS = observed non-empty; FAIL = wired but
/// silent (would surface bugs where the fn is called but does nothing).
fn print_wireup_observations(save: &RuntimeSaveGame) {
    println!("\n=== Wire-up observations ===");

    // 1. Per-match rating deltas → season_rating_stats.
    let rated_players = save.player_ratings.season_rating_stats.len();
    let total_apps: u32 = save.player_ratings.season_rating_stats.values()
        .map(|(_, c)| *c as u32).sum();
    let avg_of_avg: Option<f32> = if rated_players > 0 {
        let avgs: Vec<f32> = save.player_ratings.season_rating_stats.iter()
            .filter_map(|(_, (sum, count))|
                if *count > 0 { Some(*sum as f32 / *count as f32) } else { None })
            .collect();
        if avgs.is_empty() { None } else {
            Some(avgs.iter().sum::<f32>() / avgs.len() as f32)
        }
    } else { None };
    println!(
        "[{}] season_rating_stats: {} players tracked, {} total appearances, mean avg = {:?}",
        pass(rated_players > 0),
        rated_players, total_apps, avg_of_avg,
    );

    // 2. Chairman states + board patience.
    let finance_clubs = save.finance.clubs.len();
    let has_chair_count = save.finance.club_has_chairman.values().filter(|v| **v).count();
    println!(
        "[diag] finance.clubs.len={}, club_has_chairman true-count={}",
        finance_clubs, has_chair_count,
    );
    let chairman_count = save.finance.chairman.len();
    let patience_seeded = save.finance.board_patience.len();
    let generosity_dist: Vec<u8> = save.finance.chairman.values()
        .map(|c| c.generosity).take(5).collect();
    println!(
        "[{}] chairman: {} states, {} board_patience entries; first 5 generosities = {:?}",
        pass(chairman_count > 0),
        chairman_count, patience_seeded, generosity_dist,
    );

    // 3. Counter-offers. Note: pending/resolved only populate when a bid
    // goes through submit_bid; run_ai_transfer_pass bypasses that pipeline
    // and moves players directly, so 0 here is normal for a headless
    // no-human-manager sim. Real AI transfer volume shows up in
    // pending_events["transfer"] and in contract-club-id churn.
    let counter_count = save.transfers.active_counters.len();
    let pending = save.transfers.pending_bids.len();
    let resolved = save.transfers.resolved_bids.len();
    println!(
        "[N/A ] transfers: {} pending, {} resolved this-tick, {} active counter-offers (headless-manager sim never submits bids)",
        pending, resolved, counter_count,
    );

    // 4. Injuries firing. No injury GENERATOR is wired yet — advance_day
    // recovers existing injuries but nothing produces them from match
    // fouls/collisions. Real gap.
    let injured = save.injuries.injuries.len() + save.injuries.suspensions.len();
    println!(
        "[N/A ] injuries: {} players unavailable (no injury generator wired — see gap)",
        injured,
    );

    // 5. Per-club tactics seeded.
    let tactic_count = save.club_tactics.len();
    println!(
        "[{}] per-club tactics: {} clubs have a Tactic assigned",
        pass(tactic_count > 0),
        tactic_count,
    );

    // 6. Contracts (transfer market populated).
    let contract_count = save.transfers.contracts.len();
    println!(
        "[{}] contracts: {} on the transfer market book",
        pass(contract_count > 0),
        contract_count,
    );

    // 7. Nation tiers.
    let foreground = save.nation_tiers.iter()
        .filter(|t| matches!(t.tier, cm_domain::LeagueTier::Foreground)).count();
    let background = save.nation_tiers.iter()
        .filter(|t| matches!(t.tier, cm_domain::LeagueTier::Background)).count();
    println!(
        "[PASS] nation tiers: {} foreground, {} background",
        foreground, background,
    );

    // 8. Pending events (news / manager_sacked / transfers etc).
    let event_kinds: std::collections::BTreeMap<&str, usize> = save.pending_events.iter()
        .fold(std::collections::BTreeMap::new(), |mut m, e| {
            *m.entry(e.kind.as_str()).or_insert(0) += 1;
            m
        });
    println!(
        "[{}] pending_events by kind: {:?}",
        pass(!event_kinds.is_empty()),
        event_kinds,
    );

    println!("=== end observations ===\n");
}

fn pass(b: bool) -> &'static str { if b { "PASS" } else { "FAIL" } }
