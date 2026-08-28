use cm_domain::aus_nsl::AusNslStage;
use cm_domain::{GameDate, NewGameOptions};
fn main() {
    let dir = "D:/cm0102-rs/rust-db";
    let path = std::path::Path::new(dir);
    let world = cm_domain::World::read_rust_db_dir(path).expect("read rust-db");
    let options = NewGameOptions { selected_nations: vec!["Australia".to_string()], background_nations: vec![], use_real_players: true, attribute_masking: true, start_year: 2001 };
    let mut save = world.new_game_from_rust_db(path, &options);
    match &save.aus_nsl { Some(s) => println!("NSL {}: {} clubs", s.year, s.teams.len()), None => { println!("NSL NOT built"); return; } }
    save.tick_to_date(GameDate { year: 2002, month: 8, day: 1 });
    println!("final stage: {:?}", save.aus_nsl.as_ref().map(|s| s.stage));
    assert_eq!(save.aus_nsl.as_ref().map(|s| s.stage), Some(AusNslStage::Complete));
    for ev in save.pending_events.iter().filter(|e| e.message.contains("NSL") || e.message.contains("Grand Final") || e.message.contains("minor premier")) {
        println!("  [{}-{:02}-{:02}] {}", ev.date.year, ev.date.month, ev.date.day, ev.message);
    }
    for h in save.honours.iter().filter(|h| h.competition == "Australian NSL") { println!("  honour 0x{:x}: {}", h.honour_id, h.club_name); }
    println!("salary cap built: {}", save.aus_salary_cap.is_some());
}
