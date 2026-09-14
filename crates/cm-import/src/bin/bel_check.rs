use cm_domain::{GameDate, NewGameOptions};
fn main() {
    let path = std::path::Path::new("D:/cm0102-rs/rust-db");
    let world = cm_domain::World::read_rust_db_dir(path).expect("read rust-db");
    let options = NewGameOptions { selected_nations: vec!["Belgium".to_string()], background_nations: vec![], use_real_players: true, attribute_masking: true, start_year: 2001 , initial_game_rng_state: None };
    let mut save = world.new_game_from_rust_db(path, &options);
    for l in &save.simple_leagues { println!("{}: {} clubs, runtime comp {}", l.name, l.teams.len(), l.runtime_comp_id); }
    save.tick_to_date(GameDate { year: 2002, month: 6, day: 1 });
    for ev in save.pending_events.iter().filter(|e| e.message.contains("Belgian")) {
        println!("  [{}-{:02}-{:02}] {}", ev.date.year, ev.date.month, ev.date.day, ev.message);
    }
}
