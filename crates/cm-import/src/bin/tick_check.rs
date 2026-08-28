use cm_domain::{GameDate, NewGameOptions};
fn main() {
    let path = std::path::Path::new("D:/cm0102-rs/rust-db");
    let world = cm_domain::World::read_rust_db_dir(path).expect("read rust-db");
    let options = NewGameOptions { selected_nations: vec!["England".to_string()], background_nations: vec![], use_real_players: true, attribute_masking: true, start_year: 2001 };
    let mut save = world.new_game_from_rust_db(path, &options);
    println!("FIFA rankings after new_game: {} nations", save.fifa_rankings.len());
    if !save.fifa_rankings.is_empty() {
        println!("Top 8 nations by strength:");
        for (i, r) in save.fifa_rankings.iter().take(8).enumerate() {
            println!("  {}. {} ({:.1})", i+1, r.nation_name, r.strength);
        }
    }
    // Tick a year, verify rankings refreshed
    if let Some(g)=&save.concacaf_gold_cup { println!("Gold Cup initial year: {}", g.year); }
    save.tick_to_date(GameDate { year: 2003, month: 8, day: 1 });
    if save.fifa_rankings.is_empty() {
        println!("\nAfter year rollover: rankings DIRTY (need refresh_after_tick)");
        world.refresh_after_tick(&mut save);
        println!("After refresh: {} nations, top = {}",
            save.fifa_rankings.len(),
            save.fifa_rankings.first().map(|r| r.nation_name.as_str()).unwrap_or("?"));
    } else {
        println!("\nRankings still cached (no year rollover yet)");
    }
    println!("\nTick trace entries: {}, elapsed days: {}", save.phase_trace.len(), save.elapsed_days);
}
