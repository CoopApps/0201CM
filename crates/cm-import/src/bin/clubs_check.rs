use std::path::Path;
fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let world = cm_domain::World::read_rust_db_dir(Path::new(&dir)).expect("db");
    let clubs = world.manageable_clubs_for_nations(&["England".to_string()]);
    println!("England manageable clubs: {}", clubs.len());
    use std::collections::BTreeMap;
    let mut by_div: BTreeMap<String, usize> = BTreeMap::new();
    for c in &clubs { *by_div.entry(c.division_name.clone()).or_default() += 1; }
    for (d, n) in &by_div { println!("  {d}: {n} clubs"); }
    println!("\nfirst 6 (top division):");
    for c in clubs.iter().take(6) { println!("  [{}] {} — {}", c.club_id, c.club_name, c.division_name); }
}
