//! Regenerate the four `_history` JSONs with named typed fields via the
//! `*HistoryView` accessors. Reads from `D:/cm0102/data/*.dat` directly.
//!
//! Usage: cargo run -p cm-import --bin regen_histories

use std::path::PathBuf;
use std::io::Write;
use cm_domain::typed_records::{
    StaffHistoryView, ClubCompHistoryView, NationCompHistoryView, StaffCompHistoryView,
};
use serde_json::json;

fn stream<F: FnMut(&[u8], usize) -> serde_json::Map<String, serde_json::Value>>(
    dat_path: &str, out_path: &std::path::Path, stride: usize, mut mapper: F,
) -> Result<usize, String> {
    let bytes = std::fs::read(dat_path).map_err(|e| format!("read {dat_path}: {e}"))?;
    if bytes.len() % stride != 0 {
        eprintln!("[warn] {dat_path}: {} bytes / stride {} = {} rem",
                  bytes.len(), stride, bytes.len() % stride);
    }
    let n = bytes.len() / stride;
    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut w = std::io::BufWriter::with_capacity(4 * 1024 * 1024, file);
    w.write_all(b"[").unwrap();
    for i in 0..n {
        if i > 0 { w.write_all(b",").unwrap(); }
        let raw = &bytes[i * stride .. (i + 1) * stride];
        let m = mapper(raw, i);
        serde_json::to_writer(&mut w, &m).unwrap();
    }
    w.write_all(b"]").unwrap();
    drop(w);
    Ok(n)
}

fn main() {
    let db = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));

    // 1. staff_history.dat — 17B rows.
    {
        let out = db.join("references/staff_history.json");
        let n = stream("D:/cm0102/data/staff_history.dat", &out,
                       StaffHistoryView::RECORD_SIZE, |raw, _| {
            let v = StaffHistoryView::from_bytes(raw);
            let mut m = serde_json::Map::new();
            m.insert("id".into(),             json!(v.id()));
            m.insert("person_id".into(),      json!(v.person_id()));
            m.insert("year".into(),           json!(v.year()));
            m.insert("competition_id".into(), json!(v.competition_id()));
            m.insert("subs".into(),           json!(v.subs()));
            m.insert("apps".into(),           json!(v.apps()));
            m.insert("goals".into(),          json!(v.goals()));
            m
        }).expect("staff_history");
        println!("[regen_histories] staff_history: {n} rows → {}", out.display());
    }

    // 2. club_comp_history.dat — 26B rows.
    {
        let out = db.join("references/club_comp_history.json");
        let n = stream("D:/cm0102/data/club_comp_history.dat", &out,
                       ClubCompHistoryView::RECORD_SIZE, |raw, _| {
            let v = ClubCompHistoryView::from_bytes(raw);
            let mut m = serde_json::Map::new();
            m.insert("id".into(),                    json!(v.id()));
            m.insert("competition_id".into(),        json!(v.competition_id()));
            m.insert("year".into(),                  json!(v.year()));
            m.insert("winner_club_id".into(),        json!(v.winner_club_id()));
            m.insert("runner_up_club_id".into(),     json!(v.runner_up_club_id()));
            m.insert("third_place_club_id".into(),   json!(v.third_place_club_id()));
            m.insert("hosts_club_id".into(),         json!(v.hosts_club_id()));
            m
        }).expect("club_comp_history");
        println!("[regen_histories] club_comp_history: {n} rows → {}", out.display());
    }

    // 3. nation_comp_history.dat — 26B rows.
    {
        let out = db.join("references/nation_comp_history.json");
        let n = stream("D:/cm0102/data/nation_comp_history.dat", &out,
                       NationCompHistoryView::RECORD_SIZE, |raw, _| {
            let v = NationCompHistoryView::from_bytes(raw);
            let mut m = serde_json::Map::new();
            m.insert("id".into(),                    json!(v.id()));
            m.insert("competition_id".into(),        json!(v.competition_id()));
            m.insert("year".into(),                  json!(v.year()));
            m.insert("winner_team_id".into(),        json!(v.winner_team_id()));
            m.insert("runner_up_team_id".into(),     json!(v.runner_up_team_id()));
            m.insert("third_place_team_id".into(),   json!(v.third_place_team_id()));
            m.insert("hosts_team_id".into(),         json!(v.hosts_team_id()));
            m
        }).expect("nation_comp_history");
        println!("[regen_histories] nation_comp_history: {n} rows → {}", out.display());
    }

    // 4. staff_comp_history.dat — 58B rows.
    {
        let out = db.join("references/staff_comp_history.json");
        let n = stream("D:/cm0102/data/staff_comp_history.dat", &out,
                       StaffCompHistoryView::RECORD_SIZE, |raw, _| {
            let v = StaffCompHistoryView::from_bytes(raw);
            let mut m = serde_json::Map::new();
            m.insert("id".into(),        json!(v.id()));
            m.insert("person_id".into(), json!(v.person_id()));
            m.insert("year".into(),      json!(v.year()));
            // 12 trailing u32s (semantics unverified — see StaffCompHistoryView docstring).
            let slots: Vec<u32> = (0..12).map(|i| v.slot(i)).collect();
            m.insert("slots".into(), json!(slots));
            m
        }).expect("staff_comp_history");
        println!("[regen_histories] staff_comp_history: {n} rows → {}", out.display());
    }

    // Spot check: row 0 of club_comp_history — Preston N.E. won 1888-89 English Div 1.
    let bytes = std::fs::read("D:/cm0102/data/club_comp_history.dat").unwrap();
    let v = ClubCompHistoryView::from_bytes(&bytes[..ClubCompHistoryView::RECORD_SIZE]);
    println!("\n=== SPOT CHECK: club_comp_history row 0 ===");
    println!("  comp: {}  year: {}", v.competition_id(), v.year());
    println!("  winner: {:?}  runner_up: {:?}  third: {:?}",
             v.winner_club_id(), v.runner_up_club_id(), v.third_place_club_id());
}
