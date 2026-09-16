//! Dump the first N fixtures for a given club id from a fresh
//! new-game state. Bypasses the renderer entirely — this is raw
//! `save.season.fixtures` after `new_game_from_rust_db`.
//!
//! Usage:
//!     cargo run -p cm-import --bin dump_fixtures --release [-- <club_id> [<n>]]
//!
//! Example: dump_fixtures 1090 40    (Cambridge United, first 40 fixtures)

use std::io::Write;

fn tick(s: &str) {
    println!("{s}");
    let _ = std::io::stdout().flush();
}

fn main() {
    let mut a = std::env::args().skip(1);
    let club_id: u32 = a.next().and_then(|s| s.parse().ok()).unwrap_or(1090);
    let n: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(30);
    let dir = "D:/cm0102-rs/rust-db";

    tick(&format!("club_id : {club_id}"));
    tick(&format!("first N : {n}"));

    let world = cm_domain::World::read_rust_db_dir(std::path::Path::new(dir)).unwrap();
    let options = cm_domain::NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
        initial_game_rng_state: None,
    };
    let save = world.new_game_from_rust_db(std::path::Path::new(dir), &options);

    // Locate this club's name.
    let name = save.season.fixtures.iter()
        .find(|f| f.home_club_id == club_id)
        .map(|f| f.home_club_name.clone())
        .or_else(|| save.season.fixtures.iter()
                 .find(|f| f.away_club_id == club_id)
                 .map(|f| f.away_club_name.clone()))
        .unwrap_or_else(|| "?".into());
    tick(&format!("club    : {name}"));
    tick("");

    // Filter + sort by date the same way the renderer does.
    let mut mine: Vec<_> = save.season.fixtures.iter()
        .filter(|f| f.home_club_id == club_id || f.away_club_id == club_id)
        .collect();
    mine.sort_by_key(|f| (f.date.year, f.date.month, f.date.day));

    tick(&format!("total fixtures for this club: {}", mine.len()));
    tick("");
    tick("first N fixtures:");
    for (i, f) in mine.iter().take(n).enumerate() {
        let is_home = f.home_club_id == club_id;
        let opp = if is_home { &f.away_club_name } else { &f.home_club_name };
        let ha  = if is_home { "H" } else { "A" };
        tick(&format!(
            "  {:>3}. {:04}-{:02}-{:02}  {}  {:<24}  comp={}  ({})",
            i + 1, f.date.year, f.date.month, f.date.day,
            ha, opp, f.competition_id, f.competition_name,
        ));
    }

    // Count duplicates: same (date, opponent).
    tick("");
    let mut dup_count = 0;
    for w in mine.windows(2) {
        let a = &w[0]; let b = &w[1];
        let a_opp = if a.home_club_id == club_id { a.away_club_id } else { a.home_club_id };
        let b_opp = if b.home_club_id == club_id { b.away_club_id } else { b.home_club_id };
        if a_opp == b_opp && a.competition_id == b.competition_id {
            let a_date = format!("{}-{:02}-{:02}", a.date.year, a.date.month, a.date.day);
            let b_date = format!("{}-{:02}-{:02}", b.date.year, b.date.month, b.date.day);
            let a_ha = if a.home_club_id == club_id { "H" } else { "A" };
            let b_ha = if b.home_club_id == club_id { "H" } else { "A" };
            let a_opp_name = if a.home_club_id == club_id { &a.away_club_name } else { &a.home_club_name };
            tick(&format!("  DUP: {} {} vs {}  <->  {} {}  (comp {})",
                          a_date, a_ha, a_opp_name, b_date, b_ha, a.competition_id));
            dup_count += 1;
        }
    }
    tick(&format!("adjacent same-opponent pairs: {dup_count}"));
}
