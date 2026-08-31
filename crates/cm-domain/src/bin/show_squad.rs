//! Show a club's squad from the boot state — CA/PA, position, generated stats,
//! and value/wage. Usage: cargo run -p cm-domain --bin show_squad -- "Sheffield Wednesday"

use cm_domain::{ClubView, World};

fn gbp(v: i64) -> String {
    if v.abs() >= 1_000_000 { format!("£{:.1}M", v as f64 / 1e6) }
    else if v.abs() >= 1_000 { format!("£{:.0}k", v as f64 / 1e3) }
    else { format!("£{v}") }
}
/// Role bucket via the DFM-verified aptitude order (+0x0f..+0x1a):
/// 0 GK, 1 SW, 2 D, 3 DM, 4 M, 5 AM, 6 ST, 7 WB, 8 R-side, 9 L-side, 10 C, 11 FR.
fn pos_label(aptitudes: &[u8; 12]) -> &'static str {
    let (idx, val) = aptitudes.iter().enumerate()
        .fold((0usize, 0u8), |(bi, bv), (i, &v)| if v > bv { (i, v) } else { (bi, bv) });
    if val == 0 { return "?"; }
    match idx {
        0 => "GK",                 // Goalkeeper
        1 | 2 => "DEF",            // Sweeper, Defender
        3 => "DM",                 // Defensive Midfielder
        4 => "MID",                // Midfielder
        5 => "AM",                 // Attacking Midfielder
        6 => "ST",                 // Attacker
        7 => "WB",                 // Wing Back
        _ => "SIDE",               // R/L/C/FR side qualifiers (usually pair with a main role)
    }
}

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let db = std::path::Path::new(&dir);
    let world = World::read_rust_db_dir(db).expect("read rust-db");
    let save = world.new_runtime_save_from_rust_db(db);

    let query = std::env::args().nth(1).unwrap_or_else(|| "Sheffield Wednesday".into());
    let q = query.to_lowercase();

    // Find the club.
    let clubs: Vec<(u32, String, u16)> = world.core.clubs.iter().filter_map(|rec| {
        let cv = ClubView::new(rec);
        let name = cv.primary_name();
        if name.to_lowercase().contains(&q) {
            Some((cv.id(), name, cv.reputation()))
        } else { None }
    }).collect();
    if clubs.is_empty() {
        println!("No club matching '{query}' found."); return;
    }
    if clubs.len() > 1 {
        println!("Multiple matches for '{query}':");
        for (id, n, r) in &clubs { println!("  {id:<6}  rep={r:<5}  {n}"); }
    }
    let (cid, name, rep) = clubs[0].clone();
    println!("\n=== {name} (club {cid}, reputation {rep}) ===\n");

    let mut squad: Vec<&cm_domain::player_rating::RatedPlayer> = save.player_ratings.players.iter()
        .filter(|p| p.club_id == Some(cid as i32)).collect();
    squad.sort_by_key(|p| (pos_label(&p.position_aptitudes), -p.ca));

    if squad.is_empty() {
        println!("(no rated players)"); return;
    }
    // Finance
    if let Some(fin) = save.finance.for_club(cid) {
        println!("Finances: balance {} · transfer budget {} · board conf {}",
            gbp(fin.balance), gbp(fin.transfer_budget), fin.board_confidence);
    }
    println!();
    println!("{:<3} {:<26} {:<3} CA  PA   Age  Value      Wk wage    Morale",
             "Pos", "Name", "GK");
    println!("{}", "-".repeat(90));
    for p in &squad {
        let person = world.staff.type6.iter().find(|t| t.id == p.staff_id);
        let full_name = person.map(|t| world.person_display_name(t)).unwrap_or_else(|| "?".into());
        let contract = save.transfers.contracts.iter().find(|c| c.player_id == p.staff_id);
        let morale = contract.map(|c| cm_domain::transfer::morale_label(c.morale)).unwrap_or("-");
        println!("{:<3} {:<26} {:<3} {:>3} {:>3}   {:>2}   {:<9} {:<10} {}",
            pos_label(&p.position_aptitudes),
            full_name, if p.is_gk {"Y"} else {""},
            p.ca, p.pa, p.age_est, gbp(p.market_value),
            format!("£{}", p.weekly_wage), morale);
    }
    println!("\n{} players; avg CA {}, top CA {}, sum position ratings (as picked XI) via snapshot:",
        squad.len(),
        squad.iter().map(|p| p.ca as i32).sum::<i32>() / squad.len() as i32,
        squad.iter().map(|p| p.ca).max().unwrap_or(0));
    if let Some(snap) = save.snapshot_team_for_engine(cid) {
        println!("  reputation used by engine: {}, sum_position_ratings: {}",
            snap.reputation, snap.sum_position_ratings);
    }
}
