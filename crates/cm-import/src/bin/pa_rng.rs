//! Show real-RNG potential-ability resolution on the flexible-potential players.
use cm_rng::MatchRng;
use std::path::Path;
fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let world = cm_domain::World::read_rust_db_dir(Path::new(&dir)).expect("db");
    let table = cm_db::ConfigData::read_rng_table(Path::new(&dir)).expect("rng");
    let date = cm_domain::GameDate { year: 2001, month: 7, day: 1 };
    // Deterministic (midpoints) vs real-RNG resolution.
    let det = world.initialise_players(&date, None);
    let mut rng = MatchRng::new_seeded(table, 2001);
    let rng_states = world.initialise_players(&date, Some(&mut rng));
    // Compare a few flexible-potential players (where det != raw CA path).
    let mut shown = 0;
    for (d, r) in det.iter().zip(rng_states.iter()) {
        // flexible = the two resolutions differ
        if d.potential_ability != r.potential_ability {
            println!("player {}: CA={} det_PA={} rng_PA={}", d.player_id, d.current_ability, d.potential_ability, r.potential_ability);
            shown += 1;
            if shown >= 8 { break; }
        }
    }
    let diff = det.iter().zip(&rng_states).filter(|(d,r)| d.potential_ability != r.potential_ability).count();
    println!("\n{diff} players resolved differently by real RNG vs midpoints (the flexible-potential set).");
}
