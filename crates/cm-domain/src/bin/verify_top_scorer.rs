//! KILL #B (slice 1) verification — real goals feed the top-scorer award.
//!
//! Exercises the SAME public methods the daily tick uses
//! (`snapshot_team_for_engine` → `simulate_one_fixture_token_model` →
//! `record_goal` → `top_scorer`), but drives the season's scheduled fixtures
//! directly in one pass instead of the (very slow) full-world daily tick — so
//! we get concrete goal tallies in seconds. This proves the scorer ids the
//! engine produces are real players and that `top_scorer` now ranks by real
//! accumulated goals, not the CA-proxy `goals_est`.
//!
//! Usage: cargo run -p cm-domain --bin verify_top_scorer

use cm_domain::World;

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let db = std::path::Path::new(&dir);
    let world = World::read_rust_db_dir(db).expect("read rust-db");
    let mut save = world.new_runtime_save_from_rust_db(db);

    // The scheduled fixtures (club pairs) for the default season.
    let pairs: Vec<(u32, u32, u32)> = save.season.fixtures.iter()
        .map(|f| (f.home_club_id, f.away_club_id, f.row))
        .collect();
    println!("KILL #B slice 1 — real goals → top scorer\n");
    println!("season fixtures scheduled . {}", pairs.len());

    let mut played = 0usize;
    let mut engine_goals = 0u32;
    for (home_id, away_id, row) in &pairs {
        let (Some(home), Some(away)) = (
            save.snapshot_team_for_engine(*home_id),
            save.snapshot_team_for_engine(*away_id),
        ) else { continue };
        let seed = (*row as u64).wrapping_mul(0x9E3779B97F4A7C15) ^ 0xC0FFEE;
        let r = cm_domain::match_engine_exe::simulate_one_fixture_token_model(&home, &away, seed);
        // Same accumulation the tick does.
        for id in r.home_scorer_ids.iter().chain(r.away_scorer_ids.iter()) {
            if *id != 0 {
                save.player_ratings.record_goal(*id);
                engine_goals += 1;
            }
        }
        played += 1;
    }

    let book = &save.player_ratings;
    let mut scorers: Vec<&cm_domain::player_rating::RatedPlayer> =
        book.players.iter().filter(|p| p.season_goals > 0).collect();
    scorers.sort_by(|a, b| b.season_goals.cmp(&a.season_goals));
    let total_goals: u32 = book.players.iter().map(|p| p.season_goals as u32).sum();

    println!("fixtures played ........... {played}");
    println!("goals from match events ... {engine_goals}");
    println!("goals accumulated in book . {total_goals}   (must equal above)");
    println!("distinct scorers .......... {}", scorers.len());

    println!("\ntop 12 scorers (REAL accumulated goals — was CA-proxy goals_est):");
    for p in scorers.iter().take(12) {
        let name = world.staff.type6.iter().find(|t| t.id == p.staff_id)
            .map(|t| world.person_display_name(t)).unwrap_or_else(|| "?".into());
        println!("  {:<26} staff#{:<6} club {:<6} CA={:<3} goals={:<3} (old goals_est was {})",
            name, p.staff_id, p.club_id.unwrap_or(-1), p.ca, p.season_goals, p.goals_est);
    }

    let mut divs: Vec<i32> = scorers.iter().filter_map(|p| p.division_id).collect();
    divs.sort_unstable(); divs.dedup();
    println!("\ntop_scorer() by division now returns a real-goals player:");
    for d in divs.iter().take(4) {
        if let Some(p) = book.top_scorer(*d) {
            println!("  div {:<6}: staff#{:<6} real goals={}", d, p.staff_id, p.season_goals);
        }
    }
}
