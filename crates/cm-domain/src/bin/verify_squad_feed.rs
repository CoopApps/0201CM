//! KILL #1 verification — match squad feeding.
//!
//! Boots a real RuntimeSaveGame from rust-db (which now generates every
//! player's CA/attributes at boot, kill #6) and measures how often
//! `snapshot_team_for_engine` can build a REAL squad vs falling back — the
//! metric that drives whether fixtures reach the ported engine or the
//! `score_from_goal_events` reputation scorer.
//!
//! Reports, over every club that has at least one scheduled fixture:
//!   * clubs whose snapshot is Some (engine runs) vs None (drops out),
//!   * how many carry a real (non-synthetic) reputation from the club record,
//!   * a sample club's engine snapshot (reputation + first XI attributes),
//!     proving the values are real type10 attributes, not the old constants.
//!
//! Usage: cargo run -p cm-domain --bin verify_squad_feed

use cm_domain::World;

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let db = std::path::Path::new(&dir);
    let world = World::read_rust_db_dir(db).expect("read rust-db");
    let save = world.new_runtime_save_from_rust_db(db);

    let book = &save.player_ratings;
    println!("KILL #1 — squad feeding (rating book now includes generated-CA players)\n");
    println!("rating book players ....... {}", book.players.len());
    println!("clubs with reputation ..... {}", book.club_reputation.len());

    // Coverage across the clubs that actually have players in the book.
    let mut club_ids: Vec<i32> = book.players.iter().filter_map(|p| p.club_id).collect();
    club_ids.sort_unstable();
    club_ids.dedup();

    let mut some = 0usize;
    let mut none = 0usize;
    let mut sizes: Vec<usize> = Vec::new();
    for &cid in &club_ids {
        match save.snapshot_team_for_engine(cid as u32) {
            Some(s) => {
                some += 1;
                sizes.push(s.players.len());
            }
            None => none += 1,
        }
    }
    let total = some + none;
    sizes.sort_unstable();
    let med = sizes.get(sizes.len() / 2).copied().unwrap_or(0);
    println!(
        "\nclubs with >=1 rated player .. {total}\n  snapshot Some (engine runs) . {some}  ({:.1}%)\n  snapshot None (fallback) .... {none}  ({:.1}%)",
        100.0 * some as f64 / total.max(1) as f64,
        100.0 * none as f64 / total.max(1) as f64,
    );
    println!("  median snapshot squad size .. {med}");

    // Sample a mid-table club and show its real engine snapshot.
    if let Some(&cid) = club_ids.get(club_ids.len() / 2) {
        if let Some(s) = save.snapshot_team_for_engine(cid as u32) {
            println!("\nsample club id {cid}: reputation={} squad={}", s.reputation, s.players.len());
            for p in s.players.iter().take(4) {
                println!(
                    "  player {:>6}: CA={:>5} pos={:>2} gk={} | aggr={} brav={} dirt={} inj={} jump={}",
                    p.player_id, p.current_ability, p.position, p.is_first_choice_gk,
                    p.aggression, p.bravery, p.dirtiness, p.injury_proneness, p.jumping_heading,
                );
            }
            let all_hardcoded = s.players.iter().all(|p|
                p.aggression == 8 && p.bravery == 10 && p.dirtiness == 5 && p.injury_proneness == 8);
            println!("  (all-old-constant attrs? {}  — expect false)", all_hardcoded);
        }
    }
}
