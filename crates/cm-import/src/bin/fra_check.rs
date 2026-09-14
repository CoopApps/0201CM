use cm_domain::{GameDate, NewGameOptions};
fn main() {
    let path = std::path::Path::new("D:/cm0102-rs/rust-db");
    let world = cm_domain::World::read_rust_db_dir(path).expect("read rust-db");
    let options = NewGameOptions { selected_nations: vec!["France".to_string()], background_nations: vec![], use_real_players: true, attribute_masking: true, start_year: 2001 , initial_game_rng_state: None };
    let mut save = world.new_game_from_rust_db(path, &options);
    for l in &save.simple_leagues { if l.name.starts_with("French") { println!("{} ({} clubs)", l.name, l.teams.len()); } }
    for c in &save.domestic_cups { if c.name.contains("French") { println!("{} ({} teams)", c.name, c.teams.len()); } }
    save.tick_to_date(GameDate { year: 2002, month: 8, day: 20 });
    for ev in save.pending_events.iter().filter(|e| e.message.contains("French") || e.message.contains("Trophée")) {
        if ev.message.contains("crowned") || ev.message.contains("win the") { println!("  [{}-{:02}-{:02}] {}", ev.date.year, ev.date.month, ev.date.day, ev.message); }
    }
}
