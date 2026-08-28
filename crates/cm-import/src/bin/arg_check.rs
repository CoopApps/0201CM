//! End-to-end check for the Argentine Primera port: build a game, tick through
//! both tournaments, and report champions + promedios relegation.
//!
//! Usage: cargo run -p cm-import --bin arg_check -- [rust_db_dir]

use cm_domain::arg_primera::ArgPhase;
use cm_domain::{GameDate, NewGameOptions};

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102-rs/rust-db".to_string());
    let path = std::path::Path::new(&dir);
    let world = cm_domain::World::read_rust_db_dir(path).expect("read rust-db");

    let options = NewGameOptions {
        selected_nations: vec!["Argentina".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
    };
    let mut save = world.new_game_from_rust_db(path, &options);

    let Some(state) = save.argentine_primera.clone() else {
        println!("NO Argentine Primera built.");
        return;
    };
    println!("Argentine Primera {} — {} clubs:", state.year, state.teams.len());
    for t in &state.teams {
        println!("  {}", t.name);
    }
    let ap = save.season.fixtures.iter().filter(|f| f.competition_name == "Argentine Apertura").count();
    let cl = save.season.fixtures.iter().filter(|f| f.competition_name == "Argentine Clausura").count();
    println!("  Apertura fixtures: {ap}, Clausura fixtures: {cl}");

    // Tick past the Clausura (Feb–Jun 2002).
    let target = GameDate { year: 2002, month: 7, day: 1 };
    let days = save.tick_to_date(target);
    println!("\nticked {days} days to 2002-07-01");
    println!("final phase: {:?}", save.argentine_primera.as_ref().map(|s| s.phase));
    assert_eq!(save.argentine_primera.as_ref().map(|s| s.phase), Some(ArgPhase::Complete));

    println!("\nArgentine news:");
    for ev in save
        .pending_events
        .iter()
        .filter(|e| e.message.contains("Argentine") || e.message.contains("promedios"))
    {
        println!("  [{}-{:02}-{:02}] {}", ev.date.year, ev.date.month, ev.date.day, ev.message);
    }

    println!("\nHonours recorded ({}):", save.honours.len());
    for h in &save.honours {
        println!("  {} {} champion (honour 0x{:x}): {}", h.year, h.competition, h.honour_id, h.club_name);
    }
}
