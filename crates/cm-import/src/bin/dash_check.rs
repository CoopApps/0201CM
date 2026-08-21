use cm_domain::{ManagerIdentity, NewGameOptions};
use std::path::Path;
fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let world = cm_domain::World::read_rust_db_dir(Path::new(&dir)).expect("db");
    let opts = NewGameOptions { selected_nations: vec!["England".into()], background_nations: vec![],
        use_real_players: true, attribute_masking: true, start_year: 2001 };
    let mut save = world.new_game_from_rust_db(Path::new(&dir), &opts);
    // Add a manager, install at a club (Arsenal id 676).
    let h = save.add_manager(ManagerIdentity { first: "Alex".into(), second: "Ferguson".into(), nickname: "Fergie".into() });
    // Arsenal's nation = England (60).
    save.install_manager_at_club(h, 676, Some(60));
    save.switch_active(h);
    match world.dashboard_for(&save, h) {
        Some(cm_domain::DashboardView::Club(d)) => {
            println!("MANAGER: {}", d.manager_name);
            println!("CLUB: {} ({}), position {}/{}", d.club_name, d.division_name, d.position, d.division_size);
            println!("DATE: {}-{:02}-{:02}", d.date.year, d.date.month, d.date.day);
            if let Some(f) = &d.next_fixture {
                println!("NEXT: {} v {} ({}) {}-{:02}-{:02}", f.home_club_name, f.away_club_name, f.competition_name, f.date.year, f.date.month, f.date.day);
            } else { println!("NEXT: none"); }
            println!("SQUAD: {} players. Top 5 by CA:", d.squad.len());
            for p in d.squad.iter().take(5) {
                println!("  {} (age {:?}) CA {} cond {}", p.name, p.age, p.current_ability, p.condition);
            }
        }
        Some(cm_domain::DashboardView::Unemployed(u)) => println!("UNEMPLOYED: {}", u.message),
        None => println!("no dashboard"),
    }
    // Resign → unemployed.
    save.resign(h);
    match world.dashboard_for(&save, h) {
        Some(cm_domain::DashboardView::Unemployed(_)) => println!("\nafter resign: UNEMPLOYED (correct)"),
        _ => println!("\nafter resign: NOT unemployed (wrong)"),
    }
}
