//! Regenerate `rust-db/staff/type9.json` — coach/staff attribute records.
//! 68 bytes each; the 21-byte coaching-stat block at body[0x0a..0x1f] is
//! fully named per the editor's staff_np field list.
//!
//! Note: the type-9 record header (ratings + position aptitudes + preferred
//! formation + non_player_id ids at body offsets before/after the coaching
//! block) is not yet fully decoded from the editor. Those bytes are kept as
//! `header_bytes`/`footer_bytes` alongside the named coaching stats until a
//! targeted decode nails them down. All 21 coaching stats named.
//!
//! Usage: cargo run -p cm-import --bin regen_type9

use std::path::PathBuf;
use serde_json::json;

/// The 21 coach-stat names in the exact byte order the editor lays them out
/// (staff_np fields, in DFM order after the five ratings).
const COACH_STAT_NAMES: [&str; 21] = [
    "attacking", "business", "coaching", "coaching_gk", "coaching_technique",
    "directness", "discipline", "free_roles", "interference", "judgement",
    "judging_potential", "man_handling", "marking", "motivating", "offside",
    "patience", "physiotherapy", "pressing", "resources", "tactics",
    "youngsters",
];

fn main() {
    let staff_dat = PathBuf::from(std::env::var("CM_STAFF_DAT")
        .unwrap_or_else(|_| "D:/cm0102/data/staff.dat".into()));
    let out_json = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into())).join("staff/type9.json");
    println!("[regen_type9] source: {}", staff_dat.display());
    println!("[regen_type9] output: {}", out_json.display());

    let staff = cm_data::load_staff_data(&staff_dat).expect("load staff.dat");
    println!("[regen_type9] parsed {} type-9 records", staff.type9.len());

    let out: Vec<serde_json::Value> = staff.type9.iter().map(|e| {
        let mut m = serde_json::Map::new();
        macro_rules! ins { ($k:literal, $v:expr) => { m.insert($k.into(), json!($v)); } }
        ins!("id", e.id);
        // 21 named coach stats at body[0x0a..0x1f].
        let coach_block = if e.body.len() >= 0x1f {
            &e.body[0x0a..0x1f]
        } else { &[][..] };
        for (i, name) in COACH_STAT_NAMES.iter().enumerate() {
            let v = if i < coach_block.len() { coach_block[i] } else { 0 };
            m.insert((*name).into(), json!(v));
        }
        // Header + footer bytes retained until a targeted decode names them.
        if !e.body.is_empty() {
            let header_end = 0x0a.min(e.body.len());
            ins!("header_bytes", &e.body[..header_end]);
            let footer_start = 0x1f.min(e.body.len());
            ins!("footer_bytes", &e.body[footer_start..]);
        }
        serde_json::Value::Object(m)
    }).collect();

    let json_str = serde_json::to_string(&out).expect("serialize");
    std::fs::write(&out_json, json_str).expect("write");
    println!("[regen_type9] wrote {} records ({} bytes)",
             out.len(), std::fs::metadata(&out_json).unwrap().len());

    // Spot check: a coach record from SWFC.
    if let Some(c) = staff.type9.iter().find(|e| e.body.len() >= 0x1f && e.body[0x0a] > 10) {
        println!("\n=== SPOT CHECK: type-9 record id={} ===", c.id);
        for (i, name) in COACH_STAT_NAMES.iter().enumerate() {
            let v = c.body.get(0x0a + i).copied().unwrap_or(0);
            println!("  {:<20} {}", name, v);
        }
    }
}
