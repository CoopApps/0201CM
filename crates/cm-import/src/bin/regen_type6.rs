//! Regenerate `rust-db/staff/type6.json` with every PlayerView-decoded field
//! written out as a named JSON key — not a `body: [1,2,3,…]` byte array.
//!
//! Streams one record at a time through `serde_json::to_writer` to avoid the
//! OOM that killed the first attempt (132k × ~40-field `serde_json::Value`
//! Map ate all RAM before serialisation). File will be larger than the slim
//! `id + body` form, but that is the point — the "no bytes" refactor's goal
//! is that a human reading the JSON can see who a person is.
//!
//! The raw body is kept as a fallback `body` field so existing `PlayerView`
//! consumers still work. Once every consumer migrates to reading named
//! fields directly, drop the `body` field from this writer.
//!
//! Usage: cargo run -p cm-import --bin regen_type6

use std::path::PathBuf;
use std::io::Write;
use cm_domain::typed_records::{PlayerView, CmDate};
use serde_json::json;

fn dt(d: CmDate) -> serde_json::Value {
    let (m, dom) = d.to_month_day();
    json!({"year": d.year, "month": m, "day": dom})
}

fn record_map(e: &cm_data::StaffType6Entry) -> serde_json::Map<String, serde_json::Value> {
    let pv = PlayerView::from_split(e.id, &e.body);
    let mut m = serde_json::Map::new();
    macro_rules! ins { ($k:literal, $v:expr) => { m.insert($k.into(), json!($v)); } }
    ins!("id", e.id);
    // Identity
    ins!("first_name_id",   pv.first_name_id());
    ins!("second_name_id",  pv.second_name_id());
    ins!("common_name_id",  pv.common_name_id());
    ins!("classification",  pv.classification());
    ins!("date_of_birth",   dt(pv.date_of_birth()));
    ins!("secondary_year_field", pv.secondary_year_field());
    // Nationality
    ins!("nation_id",       pv.nation_id());
    ins!("second_nation_id", pv.second_nation_id());
    ins!("international_caps",  pv.international_caps());
    ins!("international_goals", pv.international_goals());
    // National team career
    ins!("national_team_id",         pv.national_team_id());
    ins!("national_job",             pv.national_job());
    ins!("date_joined_national_job", dt(pv.date_joined_national_job()));
    ins!("national_contract_expires",dt(pv.national_contract_expires()));
    // Club career
    ins!("current_club_id",       pv.current_club_id());
    ins!("club_job",              pv.club_job());
    ins!("date_joined_club",      dt(pv.date_joined_club()));
    ins!("club_contract_expires", dt(pv.club_contract_expires()));
    ins!("wage",                  pv.wage());
    ins!("value",                 pv.value());
    // Personality (hidden mentals)
    ins!("adaptability",   pv.adaptability());
    ins!("ambition",       pv.ambition());
    ins!("determination",  pv.determination());
    ins!("loyalty",        pv.loyalty());
    ins!("pressure",       pv.pressure());
    ins!("professionalism",pv.professionalism());
    ins!("sportsmanship",  pv.sportsmanship());
    ins!("temperament",    pv.temperament());
    // Extras + join keys
    ins!("squad_flags",          pv.squad_flags());
    ins!("club_valuation",       pv.club_valuation());
    ins!("player_data_id",       pv.player_data_id());
    ins!("non_player_data_id",   pv.non_player_data_id());
    if let Some(prefs) = pv.preference_ids() {
        m.insert("preferences".into(), json!(prefs));
    }
    // Backward-compat: keep the raw bytes until every consumer migrates off
    // PlayerView-on-body. Drop this line once nothing reads it.
    ins!("body", e.body);
    m
}

fn main() {
    let staff_dat = PathBuf::from(std::env::var("CM_STAFF_DAT")
        .unwrap_or_else(|_| "D:/cm0102/data/staff.dat".into()));
    let db = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    let out_json = db.join("staff/type6.json");
    println!("[regen_type6] source: {}", staff_dat.display());
    println!("[regen_type6] output: {}", out_json.display());

    let staff = cm_data::load_staff_data(&staff_dat).expect("load staff.dat");
    println!("[regen_type6] parsed {} type-6 records", staff.type6.len());

    // Stream: one map at a time so peak memory stays flat, no matter how big.
    let file = std::fs::File::create(&out_json).expect("open");
    let mut w = std::io::BufWriter::with_capacity(8 * 1024 * 1024, file);
    w.write_all(b"[").unwrap();
    for (i, e) in staff.type6.iter().enumerate() {
        if i > 0 { w.write_all(b",").unwrap(); }
        let m = record_map(e);
        serde_json::to_writer(&mut w, &m).expect("serialise record");
    }
    w.write_all(b"]").unwrap();
    drop(w);
    println!("[regen_type6] wrote {} named-field records ({} bytes)",
             staff.type6.len(), std::fs::metadata(&out_json).unwrap().len());

    // Spot check: Kevin Pressman (type6.id=57342).
    if let Some(p) = staff.type6.iter().find(|e| e.id == 57342) {
        let pv = PlayerView::from_split(p.id, &p.body);
        println!("\n=== SPOT CHECK: type-6 record 57342 (Kevin Pressman) ===");
        println!("  first_name_id: {:?}",  pv.first_name_id());
        println!("  second_name_id: {:?}", pv.second_name_id());
        println!("  DOB: {:?}",            pv.date_of_birth());
        println!("  nation_id: {:?}",      pv.nation_id());
        println!("  current_club_id: {:?}", pv.current_club_id());
        println!("  club_job: {}",         pv.club_job());
        println!("  wage: {}",             pv.wage());
        println!("  player_data_id: {:?}", pv.player_data_id());
        println!("  determination: {}",    pv.determination());
        println!("  loyalty: {}",          pv.loyalty());
    }
}
