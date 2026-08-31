//! KILL #T verification — real position rating + team score.
//! Boots a save and shows that team score (from ported FUN_006c8930 position
//! rating × sum + FUN_006c5c40 opp-rep normalisation) differentiates good and
//! bad squads — not just avg-CA, but position-fit × opponent quality.

use cm_domain::World;

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let db = std::path::Path::new(&dir);
    let world = World::read_rust_db_dir(db).expect("read rust-db");
    let save = world.new_runtime_save_from_rust_db(db);

    println!("KILL #T — position rating + team score\n");

    // Pick a few clubs across the reputation spectrum.
    let mut samples: Vec<(u32, u16)> = save.player_ratings.club_reputation.iter()
        .map(|(k, v)| (*k as u32, *v)).collect();
    samples.sort_by_key(|(_, r)| *r);
    let n = samples.len();
    let picks: Vec<(u32, u16)> = [n/4, n/2, 3*n/4, n-100, n-10, n-1].iter()
        .filter_map(|&i| samples.get(i).copied()).collect();

    println!("club_id  rep     avg_ca  sum_pos_rat   team_score-vs-2000rep");
    for (cid, rep) in &picks {
        let Some(snap) = save.snapshot_team_for_engine(*cid) else { continue };
        let avg_ca = if snap.players.is_empty() { 0 } else {
            snap.players.iter().map(|p| p.current_ability as i32).sum::<i32>() / snap.players.len() as i32
        };
        let vs_mid = cm_domain::tactics::team_score(snap.sum_position_ratings, 2000);
        println!("{:<7}  {:<6}  {:<6}  {:<12}  {}",
            cid, rep, avg_ca, snap.sum_position_ratings, vs_mid);
    }

    // Head-to-head: a top club vs a bottom club.
    if let (Some(top), Some(bot)) = (picks.last(), picks.first()) {
        if let (Some(s_top), Some(s_bot)) = (
            save.snapshot_team_for_engine(top.0), save.snapshot_team_for_engine(bot.0),
        ) {
            let top_vs_bot = cm_domain::tactics::team_score(s_top.sum_position_ratings, bot.1);
            let bot_vs_top = cm_domain::tactics::team_score(s_bot.sum_position_ratings, top.1);
            println!("\nhead-to-head:");
            println!("  top club (rep {}) vs bottom (rep {}): team_score = {}",
                top.1, bot.1, top_vs_bot);
            println!("  bottom (rep {}) vs top (rep {}):     team_score = {}",
                bot.1, top.1, bot_vs_top);
            println!("  ratio: {:.2}× — should reflect the class gap",
                top_vs_bot as f32 / bot_vs_top.max(1) as f32);
        }
    }
}
