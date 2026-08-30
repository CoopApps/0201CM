//! Regenerate `rust-db/references/stadiums.json` with every StadiumView-
//! decoded field as named typed values. Reads directly from
//! `D:/cm0102/data/stadium.dat` (78-byte records, `DAT_00acd5b8` pool).
//!
//! Usage: cargo run -p cm-import --bin regen_stadiums

use std::path::PathBuf;
use cm_domain::typed_records::StadiumView;
use serde_json::json;

const RECORD_SIZE: usize = StadiumView::RECORD_SIZE;

fn main() {
    let dat = std::env::var("CM_STADIUM_DAT")
        .unwrap_or_else(|_| "D:/cm0102/data/stadium.dat".into());
    let db_dir = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    let out = db_dir.join("references/stadiums.json");
    println!("[regen_stadiums] .dat: {dat}");
    println!("[regen_stadiums] out:  {}", out.display());

    let bytes = std::fs::read(&dat).expect("read stadium.dat");
    // Try headers 0/4/8 to find the plausible record count.
    let (header, n) = [0usize, 4, 8].iter()
        .map(|&h| (h, (bytes.len().saturating_sub(h)) / RECORD_SIZE))
        .find(|&(_, n)| n > 1000 && n < 20000)
        .expect("find header");
    println!("[regen_stadiums] header={header} records={n}");

    let mut arr = Vec::with_capacity(n);
    for i in 0..n {
        let off = header + i * RECORD_SIZE;
        let raw = &bytes[off..off + RECORD_SIZE];
        let v = StadiumView::from_bytes(raw);
        let mut m = serde_json::Map::new();
        macro_rules! ins { ($k:literal, $e:expr) => { m.insert($k.into(), json!($e)); } }
        ins!("id",                 v.id());
        ins!("name",               v.name());
        ins!("name_set",           v.name_set());
        ins!("city_id",            v.city_id());
        ins!("capacity_total",     v.capacity_total());
        ins!("capacity_seated",    v.capacity_seated());
        ins!("capacity_expansion", v.capacity_expansion());
        ins!("alt_stadium_id",     v.alt_stadium_id());
        arr.push(serde_json::Value::Object(m));
    }

    let s = serde_json::to_string(&arr).expect("serialize");
    std::fs::write(&out, s).expect("write");
    println!("[regen_stadiums] wrote {n} records ({} bytes)",
             std::fs::metadata(&out).unwrap().len());

    // Spot check.
    for i in 0..n {
        let raw = &bytes[header + i * RECORD_SIZE..header + (i + 1) * RECORD_SIZE];
        let v = StadiumView::from_bytes(raw);
        if v.name().contains("Old Trafford") {
            println!("\n=== SPOT CHECK: {} (id={}) ===", v.name(), v.id());
            println!("  city_id:            {:?}", v.city_id());
            println!("  capacity_total:     {}", v.capacity_total());
            println!("  capacity_seated:    {}", v.capacity_seated());
            println!("  capacity_expansion: {}", v.capacity_expansion());
            break;
        }
    }
}
