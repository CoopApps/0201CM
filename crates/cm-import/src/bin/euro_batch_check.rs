use cm_domain::{GameDate, NewGameOptions};
fn main() {
    let path=std::path::Path::new("D:/cm0102-rs/rust-db");
    let world=cm_domain::World::read_rust_db_dir(path).unwrap();
    let mut save=world.new_game_from_rust_db(path,&NewGameOptions{selected_nations:vec!["England".to_string()],background_nations:vec![],use_real_players:true,attribute_masking:true,start_year:2001});
    save.tick_to_date(GameDate{year:2002,month:8,day:1});
    let kws=["La Liga","Copa del Rey","Spanish Super","Eredivisie","Dutch Cup","Dutch Super","Tippeligaen","Norwegian Cup","Russian Prem","Russian Cup","Allsvenskan","Swedish Cup","Turkish Super","Turkish Cup","Welsh Prem","Welsh Cup","Primeira Liga","Portuguese Cup","Portuguese Super","Scottish Prem","Scottish Cup","Scottish League Cup"];
    for ev in &save.pending_events {
        if (ev.message.contains("crowned")||ev.message.contains("win the")) && kws.iter().any(|k| ev.message.contains(k)) {
            println!("  [{}-{:02}-{:02}] {}", ev.date.year, ev.date.month, ev.date.day, ev.message);
        }
    }
}