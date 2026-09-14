use cm_domain::{GameDate, NewGameOptions};
fn main() {
    let path = std::path::Path::new("D:/cm0102-rs/rust-db");
    let world = cm_domain::World::read_rust_db_dir(path).expect("read rust-db");
    let options = NewGameOptions { selected_nations: vec!["Brazil".to_string()], background_nations: vec![], use_real_players: true, attribute_masking: true, start_year: 2001 , initial_game_rng_state: None };
    let mut save = world.new_game_from_rust_db(path, &options);
    println!("Brazilian competitions built: {} leagues, {} cups", save.simple_leagues.len(), save.domestic_cups.len());
    for l in &save.simple_leagues { if l.name.contains("ampeonato") || l.name.contains("Série") || l.name.contains("Brazilian") { println!("  {} ({} clubs, {} legs)", l.name, l.teams.len(), l.legs); } }
    save.tick_to_date(GameDate { year: 2002, month: 12, day: 20 });
    println!("\nBrazilian champions:");
    for ev in save.pending_events.iter().filter(|e| e.message.contains("Campeonato") || e.message.contains("Série") || e.message.contains("Copa do Brasil") || e.message.contains("Brazilian")) {
        if ev.message.contains("crowned") || ev.message.contains("win the") { println!("  [{}-{:02}-{:02}] {}", ev.date.year, ev.date.month, ev.date.day, ev.message); }
    }
}
