//! Regenerate `rust-db/core/clubs.json` with every ClubView-decoded field as a
//! named typed value. Retains the 581-byte raw record for fields we haven't
//! semantic-decoded yet (editor decode agent nailing them down).
//!
//! Usage: cargo run -p cm-import --bin regen_clubs

use std::path::PathBuf;
use cm_domain::typed_records::ClubView;
use serde_json::json;

fn main() {
    let db_dir = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    let out = db_dir.join("core/clubs.json");
    println!("[regen_clubs] db_dir: {}", db_dir.display());
    println!("[regen_clubs] out: {}", out.display());

    let world = cm_domain::World::read_rust_db_dir(&db_dir).expect("read rust-db");
    let count = world.core.clubs.len();
    println!("[regen_clubs] loaded {count} club records from rust-db");

    let arr: Vec<serde_json::Value> = world.core.clubs.iter().map(|rec| {
        let cv = ClubView::new(rec);
        let mut m = serde_json::Map::new();
        macro_rules! ins { ($k:literal, $v:expr) => { m.insert($k.into(), json!($v)); } }
        ins!("id",              cv.id());
        ins!("primary_name",    cv.primary_name());
        ins!("secondary_name",  cv.secondary_name());
        ins!("division",        cv.division());
        ins!("nation_id",       cv.nation_id());
        ins!("division_id",     cv.division_id());
        ins!("secondary_comp_id", cv.secondary_comp_id());
        ins!("tertiary_comp_id",  cv.tertiary_comp_id());
        ins!("stadium_id",      cv.stadium_id());
        ins!("reputation",      cv.reputation());
        ins!("reputation_raw_byte", cv.reputation_raw_byte());
        // Loader-confirmed i32 fields with TBD semantics — leave a note.
        ins!("field_5b",        cv.field_5b());
        ins!("field_60",        cv.field_60());
        ins!("field_65",        cv.field_65());
        ins!("field_6e",        cv.field_6e());
        // Cash balance (£) — negative = bankrupt
        ins!("cash",               cv.cash());
        // Attendance figures (scale with stadium capacity)
        ins!("attendance_average", cv.attendance_average());
        ins!("attendance_minimum", cv.attendance_minimum());
        ins!("attendance_maximum", cv.attendance_maximum());
        // Rivals + manager/assistant + status flag
        ins!("rival_club_1",  cv.rival_club_1());
        ins!("rival_club_2",  cv.rival_club_2());
        ins!("rival_club_3",  cv.rival_club_3());
        ins!("manager_id",         cv.manager_id());
        ins!("assistant_manager_id", cv.assistant_manager_id());
        ins!("flag_byte_8b",  cv.flag_byte_8b());
        // Kit colours (VERIFIED via render code +0x37/+0x38/+0x39 on the
        // referenced colour records).
        ins!("kit1_fg_color_id", cv.kit1_fg_color_id());
        ins!("kit1_bg_color_id", cv.kit1_bg_color_id());
        ins!("kit2_fg_color_id", cv.kit2_fg_color_id());
        ins!("kit2_bg_color_id", cv.kit2_bg_color_id());
        ins!("kit3_fg_color_id", cv.kit3_fg_color_id());
        ins!("kit3_bg_color_id", cv.kit3_bg_color_id());
        // Chairman + board (VERIFIED via loader loops).
        ins!("chairman_id",   cv.chairman_id());
        ins!("board_members", (0..3).map(|i| cv.board_member(i)).collect::<Vec<_>>());
        ins!("club_status",   cv.club_status()); // 1=pro / 2=semi-pro / 3=amateur (best guess)
        // Staff assignment slots — verified loop sizes in loader
        // FUN_00537870: squad 50, coach 5, scout 7, physio 3.
        ins!("squad_slots",   (0..50).map(|i| cv.squad_slot(i)).collect::<Vec<_>>());
        ins!("coach_slots",   (0..5).map(|i| cv.coach_slot(i)).collect::<Vec<_>>());
        ins!("scout_slots",   (0..7).map(|i| cv.scout_slot(i)).collect::<Vec<_>>());
        ins!("physio_slots",  (0..3).map(|i| cv.physio_slot(i)).collect::<Vec<_>>());
        // Transfer target watchlist (VERIFIED via FUN_00546e70 cap at 0x13).
        ins!("transfer_targets", (0..20).map(|i| cv.transfer_target(i)).collect::<Vec<_>>());
        // Retain raw for byte-fidelity round-trip; remove once every consumer
        // migrates to named fields.
        ins!("raw", &rec.raw);
        serde_json::Value::Object(m)
    }).collect();

    let s = serde_json::to_string(&arr).expect("serialize");
    std::fs::write(&out, s).expect("write");
    println!("[regen_clubs] wrote {count} records ({} bytes)",
             std::fs::metadata(&out).unwrap().len());

    // Spot check: Sheffield Wednesday.
    if let Some(sw) = world.core.clubs.iter()
        .find(|r| ClubView::new(r).primary_name().contains("Sheffield Wednesday"))
    {
        let cv = ClubView::new(sw);
        println!("\n=== SPOT CHECK: {} (id={}) ===", cv.primary_name(), cv.id());
        println!("  nation_id: {:?}", cv.nation_id());
        println!("  division_id: {:?}", cv.division_id());
        println!("  reputation: {}", cv.reputation());
        println!("  stadium_id: {:?}", cv.stadium_id());
        println!("  cash: £{}", cv.cash());
        println!("  attendance avg/min/max: {} / {} / {}",
                 cv.attendance_average(), cv.attendance_minimum(), cv.attendance_maximum());
        println!("  manager_id: {:?}", cv.manager_id());
        println!("  assistant_manager_id: {:?}", cv.assistant_manager_id());
        println!("  rivals: {:?}, {:?}, {:?}",
                 cv.rival_club_1(), cv.rival_club_2(), cv.rival_club_3());
        println!("  flag_byte_8b: 0x{:02x}", cv.flag_byte_8b());
    }
}
