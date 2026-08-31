//! KILL #8/#9 verification — real finance + board confidence.
//! Boots a save with the ported finance seed (real START_CASH table + rep/status
//! transfer budget); runs a season of weekly wages, monthly rollovers, and
//! simulated match income directly; reports balance/budget distributions and
//! shows a sample of clubs across the reputation spectrum.
//!
//! Usage: cargo run -p cm-domain --bin verify_finance -- [weeks]

use cm_domain::World;
use cm_domain::finance::FinanceStatus;

fn gbp(v: i64) -> String {
    if v.abs() >= 1_000_000 { format!("£{:.1}M", v as f64 / 1e6) }
    else if v.abs() >= 1_000 { format!("£{:.0}k", v as f64 / 1e3) }
    else { format!("£{v}") }
}

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let db = std::path::Path::new(&dir);
    let world = World::read_rust_db_dir(db).expect("read rust-db");
    let mut save = world.new_runtime_save_from_rust_db(db);
    let weeks: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(38);

    println!("KILL #8/#9 — real finance + board confidence\n");
    println!("clubs with finance state . {}", save.finance.clubs.len());

    // Boot distribution.
    let mut bals: Vec<i64> = save.finance.clubs.iter().map(|c| c.balance).collect();
    bals.sort_unstable();
    let pct = |v: &[i64], p: f64| v.get(((v.len() as f64 * p) as usize).min(v.len()-1)).copied().unwrap_or(0);
    println!("boot balances: min {}  p10 {}  p50 {}  p90 {}  max {}",
        gbp(*bals.first().unwrap()), gbp(pct(&bals, 0.10)), gbp(pct(&bals, 0.50)),
        gbp(pct(&bals, 0.90)), gbp(*bals.last().unwrap()));
    let with_budget = save.finance.clubs.iter().filter(|c| c.transfer_budget > 0).count();
    println!("clubs with transfer budget: {with_budget}");

    // Simulate a season of weekly wages + a match-income event per week + monthly rollover.
    // (Directly, to avoid the very slow full-world daily tick.)
    let club_ids: Vec<u32> = save.finance.clubs.iter().map(|c| c.club_id).take(200).collect();
    for w in 0..weeks {
        save.finance.pay_weekly_wages();
        // Simulate 20 matches per week between paired sample clubs.
        for i in 0..20 {
            let h = club_ids[i % club_ids.len()];
            let a = club_ids[(i + 7) % club_ids.len()];
            save.finance.record_match_income(h, a, false);
        }
        if w % 4 == 3 { save.finance.end_of_month(); }
    }

    // After a season.
    let mut bals: Vec<i64> = save.finance.clubs.iter().map(|c| c.balance).collect();
    bals.sort_unstable();
    println!("\nafter {weeks} weeks:");
    println!("  balances: p10 {}  p50 {}  p90 {}  max {}",
        gbp(pct(&bals, 0.10)), gbp(pct(&bals, 0.50)),
        gbp(pct(&bals, 0.90)), gbp(*bals.last().unwrap()));
    let mut in_red = [0usize; 5];
    for c in &save.finance.clubs {
        let rep = save.finance.club_reputation.get(&c.club_id).copied().unwrap_or(1000);
        in_red[match c.status(rep) {
            FinanceStatus::Rich => 0, FinanceStatus::Healthy => 1,
            FinanceStatus::Normal => 2, FinanceStatus::InTheRed => 3,
            FinanceStatus::Admin => 4,
        }] += 1;
    }
    println!("  status: Rich {}  Healthy {}  Normal {}  InRed {}  Admin {}",
        in_red[0], in_red[1], in_red[2], in_red[3], in_red[4]);

    // Sample clubs.
    let mut sample_ids: Vec<u32> = save.finance.club_reputation.iter()
        .filter(|(_, r)| **r >= 4500).map(|(k, _)| *k).take(5).collect();
    sample_ids.extend(save.finance.club_reputation.iter()
        .filter(|(_, r)| **r < 2000).map(|(k, _)| *k).take(3));
    println!("\nsample clubs:");
    for cid in sample_ids {
        let c = save.finance.clubs.iter().find(|c| c.club_id == cid).unwrap();
        let rep = save.finance.club_reputation[&cid];
        println!("  club {:<5} rep={:<5}  bal {:<10}  budget {:<8}  weekly wage {:<8}  boardConf {}",
            cid, rep, gbp(c.balance), gbp(c.transfer_budget), gbp(c.weekly_wage_bill as i64),
            c.board_confidence);
    }
}
