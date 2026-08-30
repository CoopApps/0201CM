//! Regenerate `rust-db/references/officials.json` with every OfficialView-
//! decoded field. Reads `D:/cm0102/data/officials.dat` (43-byte records).
//!
//! Usage: cargo run -p cm-import --bin regen_officials

use std::path::PathBuf;
use cm_domain::typed_records::OfficialView;
use serde_json::json;

const RECORD_SIZE: usize = OfficialView::RECORD_SIZE;

fn main() {
    let dat = std::env::var("CM_OFFICIALS_DAT")
        .unwrap_or_else(|_| "D:/cm0102/data/officials.dat".into());
    let db_dir = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    let out = db_dir.join("references/officials.json");
    println!("[regen_officials] .dat: {dat}");
    println!("[regen_officials] out:  {}", out.display());

    let bytes = std::fs::read(&dat).expect("read officials.dat");
    let (header, n) = [0usize, 4, 8].iter()
        .map(|&h| (h, (bytes.len().saturating_sub(h)) / RECORD_SIZE))
        .find(|&(_, n)| n > 500 && n < 20000)
        .expect("find header");
    println!("[regen_officials] header={header} records={n}");

    let mut arr = Vec::with_capacity(n);
    for i in 0..n {
        let off = header + i * RECORD_SIZE;
        let raw = &bytes[off..off + RECORD_SIZE];
        let v = OfficialView::from_bytes(raw);
        let mut m = serde_json::Map::new();
        macro_rules! ins { ($k:literal, $e:expr) => { m.insert($k.into(), json!($e)); } }
        ins!("id",             v.id());
        ins!("first_name_id",  v.first_name_id());
        ins!("second_name_id", v.second_name_id());
        ins!("dob_day",        v.dob_day());
        ins!("dob_year",       v.dob_year());
        ins!("flags",          v.flags());
        ins!("nation_id",      v.nation_id());
        ins!("home_city_id",   v.home_city_id());
        ins!("reputation_ca",  v.reputation_ca());
        ins!("reputation_pa",  v.reputation_pa());
        ins!("rating_bytes",   v.rating_bytes().to_vec());
        arr.push(serde_json::Value::Object(m));
    }

    let s = serde_json::to_string(&arr).expect("serialize");
    std::fs::write(&out, s).expect("write");
    println!("[regen_officials] wrote {n} records ({} bytes)",
             std::fs::metadata(&out).unwrap().len());

    // Spot check: id=2 (verified in decode agent report: England, DOB day 155 of 1958).
    for i in 0..n.min(10) {
        let raw = &bytes[header + i * RECORD_SIZE..header + (i + 1) * RECORD_SIZE];
        let v = OfficialView::from_bytes(raw);
        if v.id() == 2 {
            println!("\n=== SPOT CHECK: official id={} ===", v.id());
            println!("  first/second_name: {:?} / {:?}", v.first_name_id(), v.second_name_id());
            println!("  DOB:      day {} of {}", v.dob_day(), v.dob_year());
            println!("  nation:   {:?}", v.nation_id());
            println!("  city:     {:?}", v.home_city_id());
            println!("  ratings:  {:?}", v.rating_bytes());
            break;
        }
    }
}
