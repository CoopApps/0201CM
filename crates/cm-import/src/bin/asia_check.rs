//! End-to-end check for the Asian continental cups.
use cm_domain::{GameDate, NewGameOptions};

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "D:/cm0102-rs/rust-db".to_string());
    let path = std::path::Path::new(&dir);
    let world = cm_domain::World::read_rust_db_dir(path).expect("read rust-db");
    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()], background_nations: vec![],
        use_real_players: true, attribute_masking: true, start_year: 2001, initial_game_rng_state: None,
    };
    let mut save = world.new_game_from_rust_db(path, &options);
    for (label, st) in [
        ("Club Championship", &save.asia_club_champ),
        ("Cup Winners' Cup", &save.asia_cup_winner),
        ("Cup of Nations", &save.asia_cup_of_nations),
    ] {
        match st {
            Some(s) => println!("{label}: {} ({} groups) year {}", s.competition_name, s.groups.len(), s.year),
            None => println!("{label}: NOT built"),
        }
    }
    save.tick_to_date(GameDate { year: 2005, month: 1, day: 1 });
    println!("\nAsian cup news (champions):");
    for ev in save.pending_events.iter().filter(|e| e.message.contains("Asian")) {
        if ev.message.contains("crowned") || ev.message.contains("Super Cup") { println!("  [{}-{:02}-{:02}] {}", ev.date.year, ev.date.month, ev.date.day, ev.message); }
    }
}
