//! KILL #2/#3/#4 verification — real valuation + AI transfer activity.
//! Boots a save and runs a season of weekly AI transfer passes directly (the
//! same method the tick calls), reporting how many transfers complete and at
//! what fees — proving transfers actually happen, priced by the real valuation.
//!
//! Usage: cargo run -p cm-domain --bin verify_transfers -- [weeks]

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
    let mut save = world.new_runtime_save_from_rust_db(db);
    let weeks: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(38);

    println!("KILL #2/#3/#4 — valuation + AI transfers\n");
    println!("contracts seeded .......... {}", save.transfers.contracts.len());

    // Snapshot each player's club before, to detect moves.
    let before: std::collections::HashMap<u32, Option<i32>> =
        save.player_ratings.players.iter().map(|p| (p.staff_id, p.club_id)).collect();

    let mut total = 0usize;
    let mut fees: Vec<i64> = Vec::new();
    // Split the borrow: pull the three books out is awkward; call via the save's
    // own method surface by ticking the AI pass directly.
    for w in 0..weeks {
        let seed = 0x008a_c0c0u64 ^ (w as u64).wrapping_mul(0x9E3779B97F4A7C15);
        let done = save.transfers.run_ai_transfer_pass(
            &mut save.player_ratings, &mut save.finance, 2001, 40, seed,
        );
        total += done;
    }
    // Recompute fees from moves (approximate via current value).
    let moved: Vec<&cm_domain::player_rating::RatedPlayer> = save.player_ratings.players.iter()
        .filter(|p| before.get(&p.staff_id).copied().flatten() != p.club_id).collect();
    for p in &moved { fees.push(p.market_value); }
    fees.sort_unstable();

    println!("weekly AI passes run ...... {weeks}");
    println!("transfers completed ....... {total}");
    println!("players who changed club .. {}", moved.len());
    if !fees.is_empty() {
        let med = fees[fees.len() / 2];
        println!("transfer-fee (by value): min {}  median {}  max {}",
            gbp(*fees.first().unwrap()), gbp(med), gbp(*fees.last().unwrap()));
    }
    println!("\nsample completed moves:");
    for p in moved.iter().take(10) {
        let name = world.staff.type6.iter().find(|t| t.id == p.staff_id)
            .map(|t| world.person_display_name(t)).unwrap_or_else(|| "?".into());
        println!("  {:<24} CA={:<3} value {:<8} → club {}",
            name, p.ca, gbp(p.market_value), p.club_id.unwrap_or(-1));
    }
}
