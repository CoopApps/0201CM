//! Regenerate `rust-db/references/cities.json` with every CityView-decoded
//! field. Reads `D:/cm0102/data/city.dat` (56-byte records).
//!
//! Usage: cargo run -p cm-import --bin regen_cities

use std::path::PathBuf;
use cm_domain::typed_records::CityView;
use serde_json::json;

const RECORD_SIZE: usize = CityView::RECORD_SIZE;

fn main() {
    let dat = std::env::var("CM_CITY_DAT")
        .unwrap_or_else(|_| "D:/cm0102/data/city.dat".into());
    let db_dir = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    let out = db_dir.join("references/cities.json");
    println!("[regen_cities] .dat: {dat}");
    println!("[regen_cities] out:  {}", out.display());

    let bytes = std::fs::read(&dat).expect("read city.dat");
    let (header, n) = [0usize, 4, 8].iter()
        .map(|&h| (h, (bytes.len().saturating_sub(h)) / RECORD_SIZE))
        .find(|&(_, n)| n > 1000 && n < 20000)
        .expect("find header");
    println!("[regen_cities] header={header} records={n}");

    let mut arr = Vec::with_capacity(n);
    for i in 0..n {
        let off = header + i * RECORD_SIZE;
        let raw = &bytes[off..off + RECORD_SIZE];
        let v = CityView::from_bytes(raw);
        let mut m = serde_json::Map::new();
        macro_rules! ins { ($k:literal, $e:expr) => { m.insert($k.into(), json!($e)); } }
        ins!("id",         v.id());
        ins!("name",       v.name());
        ins!("name_set",   v.name_set());
        ins!("nation_id",  v.nation_id());
        ins!("latitude",   v.latitude());
        ins!("longitude",  v.longitude());
        ins!("size_tier",  v.size_tier());
        ins!("region_or_primary_club", v.region_or_primary_club());
        arr.push(serde_json::Value::Object(m));
    }

    let s = serde_json::to_string(&arr).expect("serialize");
    std::fs::write(&out, s).expect("write");
    println!("[regen_cities] wrote {n} records ({} bytes)",
             std::fs::metadata(&out).unwrap().len());

    // Spot check: London.
    for i in 0..n {
        let raw = &bytes[header + i * RECORD_SIZE..header + (i + 1) * RECORD_SIZE];
        let v = CityView::from_bytes(raw);
        if v.name() == "London" {
            println!("\n=== SPOT CHECK: {} (id={}) ===", v.name(), v.id());
            println!("  nation_id: {}", v.nation_id());
            println!("  lat/lon:   {:.3}° / {:.3}°", v.latitude(), v.longitude());
            println!("  size_tier: {}", v.size_tier());
            break;
        }
    }
}
