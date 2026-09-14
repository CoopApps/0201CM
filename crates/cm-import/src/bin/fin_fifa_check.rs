use cm_domain::{GameDate, NewGameOptions};
fn main() {
    let path = std::path::Path::new("D:/cm0102-rs/rust-db");
    let world = cm_domain::World::read_rust_db_dir(path).expect("read rust-db");
    let options = NewGameOptions { selected_nations: vec!["Finland".to_string()], background_nations: vec![], use_real_players: true, attribute_masking: true, start_year: 2001 , initial_game_rng_state: None };
    let mut save = world.new_game_from_rust_db(path, &options);
    save.tick_to_date(GameDate { year: 2003, month: 8, day: 1 });
    for ev in save.pending_events.iter().filter(|e| e.message.contains("Finnish") || e.message.contains("Confederations")) {
        if ev.message.contains("crowned") || ev.message.contains("win the") || ev.message.contains("champions in") { println!("  [{}-{:02}-{:02}] {}", ev.date.year, ev.date.month, ev.date.day, ev.message); }
    }
}
