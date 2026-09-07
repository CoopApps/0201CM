//! Smoke test — load the rust-db and verify contracts get generated
//! for known-empty-shipped-dat players (Muggleton, id=56534).

use cm_domain::World;
use std::path::Path;

#[test]
fn muggleton_gets_a_contract() {
    let root = Path::new("D:/cm0102-rs/rust-db");
    if !root.exists() {
        eprintln!("skipping — rust-db not at {root:?}");
        return;
    }
    let world = World::read_rust_db_dir(root).expect("load rust-db");
    let pool = world.contracts.as_ref().expect("contracts populated");
    println!("pool: {} contracts, {} index slots",
             pool.records.len(), pool.by_staff_id.len());
    let c = pool.contract_for_staff(56534).expect("Muggleton has contract");
    println!("Muggleton contract: wage={} value={} np={} rlg={} mj={}",
             c.wage, c.value, c.non_promotion, c.relegation, c.manager_job);

    // Diagnostic — dump Cheltenham reputation + all Cheltenham players
    // with their computed wages/values, so we can see the shape vs the
    // exe's reference numbers (Muggleton £750/£110K NP+Rlg,
    // Higgs £300/£12K, Williamson £150/£26K, Woodman £425/£26K, ...).
    let cv = world.core.clubs.iter()
        .map(|r| cm_domain::typed_records::ClubView::new(r))
        .find(|v| v.id() == 1953).expect("Cheltenham club");
    println!("Cheltenham rep = {}", cv.reputation());

    // ---- Aggregate proof: at world scale, how many staff have DB-real
    //      wage/value/expiry vs synthesised? ----
    let mut n_total = 0;
    let mut n_db_wage   = 0;
    let mut n_db_value  = 0;
    let mut n_db_expiry = 0;
    for p in &world.staff.type6 {
        if p.current_club_id().is_none() { continue; }
        let pv = cm_domain::typed_records::PlayerView::from_split(p.id, &p.body);
        n_total += 1;
        if pv.wage()  > 0                          { n_db_wage   += 1; }
        if pv.value() > 0                          { n_db_value  += 1; }
        if !pv.club_contract_expires().is_placeholder() { n_db_expiry += 1; }
    }
    println!("\n[layered override] of {n_total} staff-with-employer:");
    println!("  db wage    : {n_db_wage} ({:.1}%) — synth for the rest",
             100.0 * n_db_wage   as f32 / n_total as f32);
    println!("  db value   : {n_db_value} ({:.1}%) — synth for the rest",
             100.0 * n_db_value  as f32 / n_total as f32);
    println!("  db expiry  : {n_db_expiry} ({:.1}%) — synth for the rest",
             100.0 * n_db_expiry as f32 / n_total as f32);

    let attr_by_id: std::collections::BTreeMap<u32, &cm_domain::DomainStaffType10> =
        world.staff.type10.iter().map(|a| (a.id, a)).collect();

    let mut chelt: Vec<_> = world.staff.type6.iter()
        .filter(|p| p.current_club_id() == Some(1953))
        .collect();
    chelt.sort_by_key(|p| p.id);
    println!("\n{:>6}  {:>3}/{:>3}  {:>6}  {:>7}  clauses  name",
             "id", "CA", "PA", "wage", "value");
    for p in &chelt {
        let pv = cm_domain::typed_records::PlayerView::from_split(p.id, &p.body);
        let link = pv.player_data_id().map(|l| l as u32).unwrap_or(p.id);
        let (ca, pa) = attr_by_id.get(&link)
            .map(|a| (a.current_ability as i32, a.potential_ability as i32))
            .unwrap_or((0, 0));
        let ct = pool.contract_for_staff(p.id);
        let (w, v, np, rlg, mj) = ct
            .map(|c| (c.wage, c.value, c.non_promotion, c.relegation, c.manager_job))
            .unwrap_or((0, 0, 0, 0, 0));
        let first = world.references.first_names.get(p.first_name_id() as usize)
            .map(|n| n.text.as_str()).unwrap_or("");
        let second = world.references.second_names.get(p.second_name_id() as usize)
            .map(|n| n.text.as_str()).unwrap_or("");
        println!("{:>6}  {:>3}/{:>3}  £{:>5}  £{:>6}  {}/{}/{}  {} {}",
                 p.id, ca, pa, w, v, np, rlg, mj, first, second);
    }
}
