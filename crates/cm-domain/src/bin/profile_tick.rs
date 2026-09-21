//! Profile the production daily tick: wall-clock per day + per-subsystem
//! breakdown via `tick_profile`. Task F of the match-engine follow-up
//! (reports/match_engine_observational_gap.md Part E → perf tranche).
//!
//! Run: CM_TICK_PROFILE=1 cargo run -q -p cm-domain --bin profile_tick [days]

use std::path::Path;
use std::time::Instant;

use cm_domain::{NewGameOptions, World};

fn main() {
    let days: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

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
    let t0 = Instant::now();
    let mut save = world.new_game_from_rust_db(rust_db, &options);
    eprintln!("boot: {:.2}s", t0.elapsed().as_secs_f32());
    eprintln!("tick_profile enabled = {}", cm_domain::tick_profile::enabled());

    cm_domain::tick_profile::reset();
    let mut day_times = Vec::new();
    for d in 0..days {
        let td = Instant::now();
        for _ in 0..3 {
            save.tick_cm_phase();
        }
        let secs = td.elapsed().as_secs_f32();
        day_times.push(secs);
        eprintln!(
            "day {:>3}: {:>7.2}s  (date {:04}-{:02}-{:02})",
            d + 1,
            secs,
            save.date.year,
            save.date.month,
            save.date.day
        );
    }
    let total: f32 = day_times.iter().sum();
    eprintln!(
        "\n{days} day(s): total {:.2}s, mean {:.2}s/day",
        total,
        total / days as f32
    );
    println!("{}", cm_domain::tick_profile::report("per-subsystem (all days)"));
}
