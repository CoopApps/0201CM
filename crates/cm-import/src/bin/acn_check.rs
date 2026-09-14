//! End-to-end check for the African Cup of Nations port: build a new game,
//! tick through the edition, and report the draw, group stage, knockout rounds
//! and champion.
//!
//! Usage: cargo run -p cm-import --bin acn-check -- [rust_db_dir]

use cm_domain::african_nations::{AcnStage, TAG_FINAL, TAG_QF, TAG_SF};
use cm_domain::{GameDate, HeadlessFixtureStatus, NewGameOptions};

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102-rs/rust-db".to_string());
    let path = std::path::Path::new(&dir);
    let world = cm_domain::World::read_rust_db_dir(path).expect("read rust-db");

    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
        initial_game_rng_state: None,
    };

    let mut save = world.new_game_from_rust_db(path, &options);

    let Some(state) = save.african_nations.clone() else {
        println!("NO ACN edition was drawn (build_african_nations_edition returned None).");
        return;
    };

    println!("African Cup of Nations {} — drawn at new game:", state.year);
    println!("  start date : {}-{:02}-{:02}", state.start_date.year, state.start_date.month, state.start_date.day);
    for (gi, group) in state.groups.iter().enumerate() {
        println!("  Group {}:", (b'A' + gi as u8) as char);
        for t in group {
            println!("    pot {:>2}  rep {:>3}  {}", t.pot, t.reputation, t.name);
        }
    }
    let acn_group_fx = save
        .season
        .fixtures
        .iter()
        .filter(|f| f.competition_id == state.competition_id)
        .count();
    println!("  group-stage fixtures generated: {acn_group_fx}");

    // Tick to just past the final (start + ~25 days).
    let target = GameDate { year: state.year, month: 2, day: 20 };
    println!("\nticking to {}-{:02}-{:02} ...", target.year, target.month, target.day);
    let days = save.tick_to_date(target);
    println!("  advanced {days} days");

    let acn = |tag_test: &dyn Fn(&str) -> bool| -> Vec<String> {
        save.season
            .fixtures
            .iter()
            .filter(|f| f.competition_id == state.competition_id && tag_test(&f.source))
            .map(|f| {
                let sc = match (f.home_score, f.away_score) {
                    (Some(h), Some(a)) => format!("{h}-{a}"),
                    _ => "  ".to_string(),
                };
                let played = matches!(f.status, HeadlessFixtureStatus::Played);
                format!(
                    "    {}-{:02}-{:02}  {:<22} {} {:<22} {}",
                    f.date.year, f.date.month, f.date.day,
                    f.home_club_name, sc, f.away_club_name,
                    if played { "" } else { "(pending)" }
                )
            })
            .collect()
    };

    let is_group = |s: &str| !s.contains(TAG_QF) && !s.contains(TAG_SF) && !s.contains(TAG_FINAL);
    println!("\nGroup stage results:");
    for line in acn(&is_group) { println!("{line}"); }
    println!("\nQuarter-finals:");
    for line in acn(&|s| s.contains(TAG_QF)) { println!("{line}"); }
    println!("\nSemi-finals:");
    for line in acn(&|s| s.contains(TAG_SF)) { println!("{line}"); }
    println!("\nFinal:");
    for line in acn(&|s| s.contains(TAG_FINAL)) { println!("{line}"); }

    let final_stage = save.african_nations.as_ref().map(|s| s.stage);
    println!("\nfinal ACN stage: {:?}", final_stage);
    assert_eq!(final_stage, Some(AcnStage::Complete), "tournament should have completed");

    println!("\nACN news items:");
    for ev in save.pending_events.iter().filter(|e| e.message.contains("African") || e.message.contains("African champions")) {
        println!("  [{}-{:02}-{:02}] {}", ev.date.year, ev.date.month, ev.date.day, ev.message);
    }
}
