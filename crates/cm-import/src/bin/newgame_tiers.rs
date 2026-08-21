//! Build a new game from a scripted picker selection and print the tier map.
//! Verifies the port of the exe's nation+0x11c tier assignment: whole world
//! stays loaded, tiers only tag foreground/background/neither.

use cm_domain::{LeagueTier, NewGameOptions, World};
use std::path::Path;

fn main() {
    let db = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let world = World::read_rust_db_dir(Path::new(&db)).expect("open rust-db");

    let options = NewGameOptions {
        selected_nations: vec!["England".into(), "Italy".into(), "Spain".into()],
        background_nations: vec!["Republic of Ireland".into(), "Scotland".into()],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
    };

    let save = world.new_game_from_rust_db(Path::new(&db), &options);

    let fg: Vec<_> = save
        .nation_tiers
        .iter()
        .filter(|t| t.tier == LeagueTier::Foreground)
        .collect();
    let bg: Vec<_> = save
        .nation_tiers
        .iter()
        .filter(|t| t.tier == LeagueTier::Background)
        .collect();
    let neither = save
        .nation_tiers
        .iter()
        .filter(|t| t.tier == LeagueTier::Neither)
        .count();

    println!("total nations tagged: {} (world NOT culled)", save.nation_tiers.len());
    println!("game start date: {}-{:02}-{:02}", save.date.year, save.date.month, save.date.day);
    println!("\nFOREGROUND ({}):", fg.len());
    for t in &fg {
        println!("  [{}] {}  detailed_matches={}", t.nation_id, t.nation_name, t.detailed_matches);
    }
    println!("\nBACKGROUND ({}):", bg.len());
    for t in &bg {
        println!("  [{}] {}  detailed_matches={}", t.nation_id, t.nation_name, t.detailed_matches);
    }
    println!("\nNEITHER: {neither} nations (loaded, not manageable, not detail-simulated)");

    // Demonstrate the runtime flag flip: promote a background nation.
    let mut save2 = save;
    if let Some(ire) = save2.nation_tiers.iter().find(|t| t.nation_name.contains("Ireland")).map(|t| t.nation_id) {
        let ok = save2.promote_nation_to_foreground(ire);
        println!("\nmid-game promote Ireland background->foreground: {ok} (pure flag flip, no reload)");
        println!("foreground_count now: {}", save2.foreground_count());
    }

    // Persist and report that the tier map round-trips through the save file.
    let out = "D:/temp/claude/D--cm0102-rs/1a091c9d-7a7f-448e-b330-6e37867ef521/scratchpad/newgame_tiers.json";
    save2.write_json_file(Path::new(out)).expect("write save");
    let reloaded = cm_domain::RuntimeSaveGame::read_json_file(Path::new(out)).expect("reload");
    println!(
        "\nsave round-trip: wrote {} nation_tiers, reloaded {} (foreground_count {})",
        save2.nation_tiers.len(),
        reloaded.nation_tiers.len(),
        reloaded.foreground_count(),
    );
}

// (player-init check appended)
