//! KILL #TR verification — real per-attribute training.
//!
//! Boots a save (which builds the development book from generated attributes +
//! club coaching), snapshots every attribute, runs a season of weekly training
//! ticks, and reports how attributes moved — proving training is now
//! per-attribute + coach/schedule driven, not a single CA nudge.
//!
//! Usage: cargo run -p cm-domain --bin verify_training -- [weeks]

use cm_domain::World;
use cm_domain::player_development::CATEGORY_ATTRS;

const NAMES: [&str; 42] = [
    "Acceleration","Aggression","Agility","Anticipation","Balance","Bravery",
    "Consistency","Corners","Crossing","Decisions","Dirtiness","Dribbling",
    "Finishing","Flair","SetPieces","Handling","Heading","ImportantMatches",
    "InjuryProneness","Jumping","Influence","LeftFoot","LongShots","Marking",
    "OffTheBall","NaturalFitness","OneOnOnes","Pace","Passing","Penalties",
    "Positioning","Reflexes","RightFoot","Stamina","Strength","Tackling",
    "Teamwork","Technique","ThrowIns","Versatility","Creativity","WorkRate",
];

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let db = std::path::Path::new(&dir);
    let world = World::read_rust_db_dir(db).expect("read rust-db");
    let mut save = world.new_runtime_save_from_rust_db(db);
    let weeks: u32 = std::env::args().nth(2).and_then(|s| s.parse().ok())
        .or_else(|| std::env::args().nth(1).and_then(|s| s.parse().ok())).unwrap_or(40);

    let dev = &save.training.development;
    println!("KILL #TR — real per-attribute training\n");
    println!("players with development state . {}", dev.players.len());
    println!("clubs with coaching data ...... {}", dev.club_coaching.len());
    // A well-coached club sample (highest fitness coaching).
    let best = dev.club_coaching.iter().max_by_key(|(_, c)| c[0]).map(|(k, c)| (*k, *c));
    if let Some((club, c)) = best {
        println!("best-coaching club {club}: coachAvg per cat (Fit/Tac/Sho/Ski/Gk) = {:?}", c);
    }

    // Snapshot every attribute before.
    let before: std::collections::HashMap<u32, Vec<u8>> =
        dev.players.iter().map(|p| (p.staff_id, p.attributes.clone())).collect();

    // Run a season of weekly training.
    let seed = 0x0089_de50u64;
    let mut rng = cm_domain::match_engine_exe::MatchRng::new(seed);
    for _ in 0..weeks {
        save.training.development.weekly_tick(1, &mut rng);
    }

    // Aggregate change.
    let dev = &save.training.development;
    let (mut up, mut down, mut same) = (0u64, 0u64, 0u64);
    let mut per_cat_net = [0i64; 5];
    let mut movers = 0u64;
    for p in &dev.players {
        let b = &before[&p.staff_id];
        let mut moved = false;
        for i in 0..42 {
            let d = p.attributes[i] as i64 - b[i] as i64;
            if d > 0 { up += 1; moved = true; } else if d < 0 { down += 1; moved = true; } else { same += 1; }
            for cat in 0..5 { if CATEGORY_ATTRS[cat].contains(&i) { per_cat_net[cat] += d; } }
        }
        if moved { movers += 1; }
    }
    println!("\nafter {weeks} weeks of training:");
    println!("  players whose attributes moved . {movers} / {}", dev.players.len());
    println!("  attribute cells up / down / flat {up} / {down} / {same}");
    println!("  net change by category (Fit/Tac/Sho/Ski/Gk) = {per_cat_net:?}");

    // A concrete young, well-coached outfielder: show the biggest movers.
    if let Some((club, _)) = best {
        if let Some(p) = dev.players.iter().find(|p| p.club_id == Some(club) && p.gk_aptitude < 0x13) {
            let b = &before[&p.staff_id];
            let name = world.staff.type6.iter().find(|t| t.id == p.staff_id)
                .map(|t| world.person_display_name(t)).unwrap_or_else(|| "?".into());
            println!("\nsample outfielder {name} (staff#{}, club {club}) attribute changes:", p.staff_id);
            let mut changes: Vec<(usize, i64)> = (0..42)
                .map(|i| (i, p.attributes[i] as i64 - b[i] as i64)).filter(|(_, d)| *d != 0).collect();
            changes.sort_by_key(|(_, d)| -d.abs());
            for (i, d) in changes.iter().take(12) {
                println!("  {:<16} {:>2} -> {:>2}  ({:+})", NAMES[*i], b[*i], p.attributes[*i], d);
            }
        }
    }
}
