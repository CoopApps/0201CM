//! KILL #2 verification — real player valuation + wages (FUN_0084d5d0 quality²).
//! Boots a save and reports the value/wage distribution + named players, to
//! confirm the ported formula produces realistic figures (stars worth tens of
//! millions on high wages; lower-league players worth thousands) instead of the
//! old CA²×100 / CA×250 heuristics.
//!
//! Usage: cargo run -p cm-domain --bin verify_valuation

use cm_domain::World;

fn gbp(v: i64) -> String {
    if v >= 1_000_000 { format!("£{:.1}M", v as f64 / 1e6) }
    else if v >= 1_000 { format!("£{:.0}k", v as f64 / 1e3) }
    else { format!("£{v}") }
}

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let db = std::path::Path::new(&dir);
    let world = World::read_rust_db_dir(db).expect("read rust-db");
    let save = world.new_runtime_save_from_rust_db(db);
    let book = &save.player_ratings;

    let mut vals: Vec<i64> = book.players.iter().map(|p| p.market_value).collect();
    vals.sort_unstable();
    let n = vals.len();
    let pct = |p: f64| vals.get(((n as f64 * p) as usize).min(n - 1)).copied().unwrap_or(0);
    println!("KILL #2 — real valuation + wages\n");
    println!("players valued ............ {n}");
    println!("value distribution: p50 {}  p90 {}  p99 {}  max {}",
        gbp(pct(0.50)), gbp(pct(0.90)), gbp(pct(0.99)), gbp(*vals.last().unwrap()));

    let mut wages: Vec<u32> = book.players.iter().map(|p| p.weekly_wage).collect();
    wages.sort_unstable();
    let wpct = |p: f64| wages.get(((n as f64 * p) as usize).min(n - 1)).copied().unwrap_or(0);
    println!("wage/wk distribution: p50 £{}  p90 £{}  p99 £{}  max £{}",
        wpct(0.50), wpct(0.90), wpct(0.99), wages.last().unwrap());

    // Top 10 most valuable, with names + CA/PA.
    let mut top: Vec<&cm_domain::player_rating::RatedPlayer> = book.players.iter().collect();
    top.sort_by_key(|p| -p.market_value);
    println!("\ntop 10 most valuable players:");
    for p in top.iter().take(10) {
        let name = world.staff.type6.iter().find(|t| t.id == p.staff_id)
            .map(|t| world.person_display_name(t)).unwrap_or_else(|| "?".into());
        println!("  {:<26} CA={:<3} PA={:<3}  value {:<8} wage {}/wk",
            name, p.ca, p.pa, gbp(p.market_value), p.weekly_wage);
    }
}
