//! New-game check: create a game from the native database with a realistic
//! picker selection and report what the resulting save contains.
//!
//! This is the Rust counterpart of the exe's post-Next path (`FUN_008120d0`,
//! "Initialising game data") and exists to show — against real data — exactly
//! how much of that initialisation the port currently performs.
//!
//! Usage: cargo run -p cm-import --bin newgame-check -- [rust_db_dir]

use cm_domain::NewGameOptions;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102-rs/rust-db".to_string());
    let world =
        cm_domain::World::read_rust_db_dir(std::path::Path::new(&dir)).expect("read rust-db");

    let options = NewGameOptions {
        selected_nations: vec![
            "England".to_string(),
            "Italy".to_string(),
            "Spain".to_string(),
        ],
        background_nations: vec!["France".to_string(), "Germany".to_string()],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
    };

    println!("picker selection: {:?}", options.selected_nations);
    let nation_ids = world.nation_ids_for_picker_labels(&options.selected_nations);
    println!("  resolved to nation ids: {nation_ids:?}");
    let comp_ids = world.competition_ids_for_nations(&options.selected_nations);
    println!("  matched {} competition(s)", comp_ids.len());
    for competition in world
        .references
        .club_competitions
        .iter()
        .filter(|c| comp_ids.contains(&c.id))
        .take(12)
    {
        println!("    {:<5} {}", competition.id, competition.long_name);
    }
    if comp_ids.len() > 12 {
        println!("    ... and {} more", comp_ids.len() - 12);
    }

    let save = world.new_game_from_rust_db(std::path::Path::new(&dir), &options);

    println!("\nresulting save:");
    println!("  format          : {} v{}", save.format, save.version);
    println!(
        "  start date      : {}-{:02}-{:02}",
        save.date.year, save.date.month, save.date.day
    );
    println!("  fixtures        : {}", save.season.fixtures.len());
    println!("  standings       : {}", save.season.standings.len());
    println!(
        "  schedule proofs : {}",
        save.season.schedule_generation.len()
    );
    println!("  pending events  : {}", save.pending_events.len());
    println!("  options recorded: {:?}", save.new_game.is_some());
    println!(
        "  table counts    : {} clubs, {} nations, {} staff6, {} staff10",
        save.table_counts.clubs,
        save.table_counts.nations,
        save.table_counts.staff_type6,
        save.table_counts.staff_type10
    );

    // Which competitions actually got fixtures?
    let mut by_comp: std::collections::BTreeMap<u32, (String, usize)> = Default::default();
    for fixture in &save.season.fixtures {
        let entry = by_comp
            .entry(fixture.competition_id)
            .or_insert_with(|| (fixture.competition_name.clone(), 0));
        entry.1 += 1;
    }
    println!("\n  scheduled competitions:");
    for (id, (name, count)) in &by_comp {
        println!("    {id:<5} {name:<34} {count} fixtures");
    }

    // Tick a week to prove the created game actually runs.
    let mut running = save;
    let report = running.run_headless_days(7);
    println!("\n  after 7 headless days:");
    println!(
        "    date {}-{:02}-{:02}, elapsed {} days",
        running.date.year, running.date.month, running.date.day, running.elapsed_days
    );
    println!("    phase trace entries: {}", running.phase_trace.len());
    let played = running
        .season
        .fixtures
        .iter()
        .filter(|f| f.home_score.is_some())
        .count();
    println!("    fixtures played    : {played}");
    let _ = report;
}
