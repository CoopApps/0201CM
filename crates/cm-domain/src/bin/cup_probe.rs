//! Cup probe: drive each PROGRESSIVE English cup's engine directly (draw ->
//! play with deterministic scores -> resolve -> next round), far faster than
//! ticking the UI a full season. Verifies: no fixtures before a round's draw
//! date, staggered entry (Premier enters the FA Cup at round 3), replays, and
//! the two-legged League Cup semi-final.
//!
//! Run: cargo run -q -p cm-domain --bin cup_probe

use std::collections::HashSet;
use std::path::Path;

use cm_domain::domestic_cup::{CupMode, CupState};
use cm_domain::game_rng::GameRng;
use cm_domain::{arg_primera, HeadlessFixtureStatus as St, NewGameOptions, World};

fn main() {
    let rust_db = Path::new("D:/cm0102-rs/rust-db");
    let world = World::read_rust_db_dir(rust_db).expect("read rust-db");
    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
        initial_game_rng_state: None,
    };
    let save = world.new_game_from_rust_db(rust_db, &options);

    // Reputation lookup for tie-breaks + decisive scoring.
    let rep_map: std::collections::HashMap<u32, u16> = save
        .player_ratings
        .club_reputation
        .iter()
        .map(|(k, v)| (*k as u32, *v))
        .collect();
    let rep = |id: u32| rep_map.get(&id).copied().unwrap_or(1000);
    let premier: HashSet<u32> = arg_primera::clubs_in_division(&world.core.clubs, 7)
        .into_iter()
        .map(|t| t.club_id)
        .collect();
    println!("Premier Division clubs: {}", premier.len());

    // No future cup fixtures should exist at boot for the PROGRESSIVE cups.
    let prog_ids: Vec<u32> = save.domestic_cups.iter()
        .filter(|c| c.mode == CupMode::ProgressiveDecoded)
        .map(|c| c.runtime_comp_id).collect();
    let boot_prog_fx = save.season.fixtures.iter()
        .filter(|f| prog_ids.contains(&f.competition_id)).count();
    println!("progressive-cup fixtures in the season list AT BOOT: {boot_prog_fx} (expect 0)\n");

    for cup in save.domestic_cups.iter().filter(|c| c.mode == CupMode::ProgressiveDecoded) {
        drive(cup.clone(), &premier, &rep);
        println!();
    }
}

/// Drive a cup through a MONOTONIC day-by-day timeline (the faithful
/// simulation): each day, progress the cup (draws a round only on/after its
/// draw date once the prior round is resolved), then play any tie whose match
/// date has arrived. Fast because it runs no match engine — just the cup
/// state machine + deterministic scores.
fn drive(mut cup: CupState, premier: &HashSet<u32>, rep: &dyn Fn(u32) -> u16) {
    let rounds = cup.schedule.clone().expect("progressive cup has a schedule").rounds;
    println!("=== {} (comp {}) — {} rounds, pool {} entrants ===",
        cup.name, cup.real_comp_id, rounds.len(), cup.ordered_pool.len());
    let mut rng = GameRng::new(0xC0FFEE ^ cup.real_comp_id as u32);
    let mut fixtures: Vec<cm_domain::HeadlessSeasonFixture> = Vec::new();
    let mut next_row = 900_000u32;
    let mut premier_first: Option<u32> = None;
    let mut forced_draw_done = false;
    let mut last_reported: Option<u32> = None;

    let mut date = rounds.first().map(|r| r.draw_date.clone()).unwrap();
    let end = {
        let mut e = rounds.last().map(|r| r.match_date.clone()).unwrap();
        for _ in 0..60 { e.advance_one_day(); } // headroom for replays/legs
        e
    };
    while date <= end && !cup.complete {
        // 1) progress: draw due rounds / resolve the current one / replays.
        let before = fixtures.len();
        let prog = cup.progress(&date, &fixtures, next_row, &mut rng, rep);
        for f in &prog.new_fixtures { next_row = next_row.max(f.row + 1); }
        fixtures.extend(prog.new_fixtures);
        // Report a newly-drawn round once.
        if let Some(r) = cup.last_drawn_round {
            if last_reported != Some(r) && fixtures.len() > before {
                last_reported = Some(r);
                let spec = &rounds[r as usize];
                let ties = cup.current_ties.len();
                let byes = cup.current_ties.iter().filter(|t| t.away.is_none()).count();
                println!(
                    "R{} drawn {}  match {}  {}leg {} -> {} ties ({} byes)",
                    r, date.iso(), spec.match_date.iso(),
                    if spec.two_leg { "TWO-" } else { "single-" },
                    if spec.replay { "replays" } else { "no-replay" },
                    ties, byes,
                );
                for t in cup.current_ties.iter().take(2) {
                    match &t.away {
                        Some(a) => println!("    {} v {}", t.home.name, a.name),
                        None => println!("    {} (bye)", t.home.name),
                    }
                }
                for f in &fixtures[before..] {
                    if premier.contains(&f.home_club_id) || premier.contains(&f.away_club_id) {
                        premier_first.get_or_insert(r);
                    }
                }
            }
        }
        if fixtures.len() > before && cup.last_drawn_round == last_reported {
            // Newly-added fixtures that are replays/2nd legs of the CURRENT round.
            let added = fixtures.len() - before;
            if cup.current_ties.iter().any(|t| t.replay_row.is_some()) && added > 0 {
                // (only note; replays are visible via the extra fixtures)
            }
        }
        if prog.completed {
            println!("  >>> CHAMPION crowned {}", date.iso());
            break;
        }
        // 2) play any tie whose match date has arrived (deterministic scores).
        for f in fixtures.iter_mut().filter(|f| f.status != St::Played && f.date <= date) {
            let (hs, aw) = decisive(f.home_club_id, f.away_club_id, rep);
            // Force ONE drawn tie (first replay-eligible fixture) to exercise
            // the replay path, then decisive everywhere else.
            let (hs, aw) = if !forced_draw_done && rounds.iter().any(|r| r.replay) {
                forced_draw_done = true;
                println!("    [forcing a 1-1 draw to test the replay path]");
                (1u8, 1u8)
            } else { (hs, aw) };
            f.home_score = Some(hs);
            f.away_score = Some(aw);
            f.status = St::Played;
        }
        date.advance_one_day();
    }

    println!("--- {} : Premier first appears at round {:?} ---", cup.name, premier_first);
    if cup.real_comp_id == 351 {
        println!("FA Cup entry-proof (Premier enters R3 = 3rd Round Proper): {}",
            if premier_first == Some(3) { "PASS" } else { "FAIL" });
    }
}

/// A decisive score (no draw) favouring the higher-reputation club.
fn decisive(home: u32, away: u32, rep: &dyn Fn(u32) -> u16) -> (u8, u8) {
    if rep(home) >= rep(away) { (2, 0) } else { (0, 2) }
}
