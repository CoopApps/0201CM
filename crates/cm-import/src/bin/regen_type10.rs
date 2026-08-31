//! Regenerate `rust-db/staff/type10.json` from the source `staff.dat`, with
//! every field named per the editor-verified layout (no more `unknown_bytes_*`).
//! Output JSON is fully typed: each player's ratings, 12 aptitudes, 42
//! attributes, and squad_number are all human-readable fields.
//!
//! Usage: cargo run -p cm-import --bin regen_type10

use std::path::PathBuf;

fn main() {
    let staff_dat = PathBuf::from(std::env::var("CM_STAFF_DAT")
        .unwrap_or_else(|_| "D:/cm0102/data/staff.dat".into()));
    let out_json = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into())).join("staff/type10.json");

    println!("[regen_type10] source: {}", staff_dat.display());
    println!("[regen_type10] output: {}", out_json.display());

    let staff = cm_data::load_staff_data(&staff_dat).expect("load staff.dat");
    println!("[regen_type10] parsed {} type-10 records", staff.type10.len());

    // Convert cm-data entries → cm-domain entries so they serialize with named
    // fields (the cm-data entry has the same shape now, but round-tripping
    // through cm-domain uses that crate's Serialize/serde derive).
    let out: Vec<serde_json::Value> = staff.type10.iter().map(|e| {
        let mut m = serde_json::Map::new();
        macro_rules! ins { ($k:literal, $v:expr) => { m.insert($k.into(), serde_json::json!($v)); } }
        ins!("id", e.id);
        ins!("flags_byte_04", e.flags_byte_04);
        ins!("current_ability", e.current_ability);
        ins!("potential_ability", e.potential_ability);
        ins!("home_reputation", e.home_reputation);
        ins!("current_reputation", e.current_reputation);
        ins!("world_reputation", e.world_reputation);
        ins!("apt_goalkeeper", e.apt_goalkeeper);
        ins!("apt_sweeper", e.apt_sweeper);
        ins!("apt_defender", e.apt_defender);
        ins!("apt_def_midfielder", e.apt_def_midfielder);
        ins!("apt_midfielder", e.apt_midfielder);
        ins!("apt_att_midfielder", e.apt_att_midfielder);
        ins!("apt_attacker", e.apt_attacker);
        ins!("apt_wing_back", e.apt_wing_back);
        ins!("apt_right_side", e.apt_right_side);
        ins!("apt_left_side", e.apt_left_side);
        ins!("apt_central", e.apt_central);
        ins!("apt_free_role", e.apt_free_role);
        ins!("acceleration", e.acceleration); ins!("aggression", e.aggression);
        ins!("agility", e.agility); ins!("anticipation", e.anticipation);
        ins!("balance", e.balance); ins!("bravery", e.bravery);
        ins!("consistency", e.consistency); ins!("corners", e.corners);
        ins!("crossing", e.crossing); ins!("free_kicks", e.free_kicks);
        ins!("handling", e.handling); ins!("heading", e.heading);
        ins!("important_matches", e.important_matches);
        ins!("injury_proneness", e.injury_proneness);
        ins!("jumping", e.jumping); ins!("leadership", e.leadership);
        ins!("left_foot", e.left_foot); ins!("long_shots", e.long_shots);
        ins!("dirtiness", e.dirtiness); ins!("dribbling", e.dribbling);
        ins!("finishing", e.finishing); ins!("flair", e.flair);
        ins!("decisions", e.decisions); ins!("movement", e.movement);
        ins!("natural_fitness", e.natural_fitness); ins!("one_on_ones", e.one_on_ones);
        ins!("marking", e.marking); ins!("pace", e.pace);
        ins!("passing", e.passing); ins!("penalties", e.penalties);
        ins!("positioning", e.positioning); ins!("reflexes", e.reflexes);
        ins!("right_foot", e.right_foot); ins!("stamina", e.stamina);
        ins!("strength", e.strength); ins!("tackling", e.tackling);
        ins!("teamwork", e.teamwork); ins!("throw_ins", e.throw_ins);
        ins!("versatility", e.versatility); ins!("vision", e.vision);
        ins!("work_rate", e.work_rate); ins!("technique", e.technique);
        ins!("squad_number", e.squad_number);
        serde_json::Value::Object(m)
    }).collect();

    // Write compact JSON.
    let json_str = serde_json::to_string(&out).expect("serialize");
    std::fs::write(&out_json, json_str).expect("write");
    println!("[regen_type10] wrote {} records ({} bytes)",
             out.len(), std::fs::metadata(&out_json).unwrap().len());

    // Spot check: Kevin Pressman (staff type-10 id=47735 per the earlier
    // player_data_id join). Show his full record.
    if let Some(p) = staff.type10.iter().find(|e| e.id == 47735) {
        println!("\n=== SPOT CHECK: type-10 record id=47735 (Kevin Pressman via player_data_id) ===");
        println!("  CA/PA: {}/{}", p.current_ability, p.potential_ability);
        println!("  reputations: home={} current={} world={}",
                 p.home_reputation, p.current_reputation, p.world_reputation);
        println!("  aptitudes: GK={} SW={} D={} DM={} M={} AM={} ST={} WB={} R={} L={} C={} FR={}",
                 p.apt_goalkeeper, p.apt_sweeper, p.apt_defender, p.apt_def_midfielder,
                 p.apt_midfielder, p.apt_att_midfielder, p.apt_attacker, p.apt_wing_back,
                 p.apt_right_side, p.apt_left_side, p.apt_central, p.apt_free_role);
        println!("  handling={} reflexes={} one_on_ones={} jumping={} strength={} pace={}",
                 p.handling, p.reflexes, p.one_on_ones, p.jumping, p.strength, p.pace);
        println!("  squad_number: {}", p.squad_number);
    }
}
