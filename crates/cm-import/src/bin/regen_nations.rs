//! Regenerate `rust-db/core/nations.json` with every NationView-decoded field
//! as named typed values. The raw 284-byte record is kept intact in the JSON
//! for fields we haven't yet decoded.
//!
//! Usage: cargo run -p cm-import --bin regen_nations

use std::path::PathBuf;
use cm_domain::typed_records::NationView;
use serde_json::json;

fn main() {
    let db_dir = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    let out = db_dir.join("core/nations.json");
    println!("[regen_nations] db_dir: {}", db_dir.display());
    println!("[regen_nations] out: {}", out.display());

    let world = cm_domain::World::read_rust_db_dir(&db_dir).expect("read rust-db");
    let count = world.core.nations.len();
    println!("[regen_nations] loaded {count} nation records");

    let arr: Vec<serde_json::Value> = world.core.nations.iter().map(|rec| {
        let v = NationView::new(rec);
        let mut m = serde_json::Map::new();
        macro_rules! ins { ($k:literal, $e:expr) => { m.insert($k.into(), json!($e)); } }
        ins!("id",                    v.id());
        ins!("primary_name",          v.primary_name());
        ins!("secondary_name",        v.secondary_name());
        ins!("three_letter_name",     v.three_letter_name());
        ins!("nationality_name",      v.nationality_name());
        ins!("region",                v.region());
        ins!("actual_region",         v.actual_region());
        ins!("season_update_day",     v.season_update_day());
        ins!("name_gender",           v.name_gender());
        ins!("short_name_gender",     v.short_name_gender());
        ins!("capital_city_id",       v.capital_city_id());
        ins!("league_standard",       v.league_standard());
        ins!("state_of_development",  v.state_of_development());
        ins!("national_stadium_id",   v.national_stadium_id());
        ins!("number_clubs",          v.number_clubs());
        ins!("number_staff",          v.number_staff());
        ins!("reputation",            v.reputation());
        ins!("foreground_colour_1",   v.foreground_colour_1());
        ins!("foreground_colour_2",   v.foreground_colour_2());
        ins!("foreground_colour_3",   v.foreground_colour_3());
        ins!("background_colour_1",   v.background_colour_1());
        ins!("background_colour_2",   v.background_colour_2());
        ins!("background_colour_3",   v.background_colour_3());
        ins!("fifa_coefficient_1991", v.fifa_coefficient(0));
        ins!("fifa_coefficient_1992", v.fifa_coefficient(1));
        ins!("fifa_coefficient_1993", v.fifa_coefficient(2));
        ins!("fifa_coefficient_1994", v.fifa_coefficient(3));
        ins!("fifa_coefficient_1995", v.fifa_coefficient(4));
        ins!("fifa_coefficient_1996", v.fifa_coefficient(5));
        ins!("fifa_coefficient_1997", v.fifa_coefficient(6));
        ins!("uefa_coefficient_1991", v.uefa_coefficient(0));
        ins!("uefa_coefficient_1992", v.uefa_coefficient(1));
        ins!("uefa_coefficient_1993", v.uefa_coefficient(2));
        ins!("uefa_coefficient_1994", v.uefa_coefficient(3));
        ins!("uefa_coefficient_1995", v.uefa_coefficient(4));
        ins!("uefa_coefficient_1996", v.uefa_coefficient(5));
        ins!("rival_nation_1",        v.rival_nation_1());
        ins!("rival_nation_2",        v.rival_nation_2());
        ins!("rival_nation_3",        v.rival_nation_3());
        ins!("is_background_league",  v.is_background_league());
        ins!("is_foreground_league",  v.is_foreground_league());
        ins!("is_active_nation",      v.is_active_nation());
        // Keep the raw record for the fields we haven't semantic-decoded.
        ins!("raw", &rec.raw);
        serde_json::Value::Object(m)
    }).collect();

    let s = serde_json::to_string(&arr).expect("serialize");
    std::fs::write(&out, s).expect("write");
    println!("[regen_nations] wrote {count} records ({} bytes)",
             std::fs::metadata(&out).unwrap().len());

    // Spot check: England.
    for rec in world.core.nations.iter() {
        let v = NationView::new(rec);
        if v.primary_name() == "England" {
            println!("\n=== SPOT CHECK: {} (id={}) ===", v.primary_name(), v.id());
            println!("  three_letter_name:    {:?}", v.three_letter_name());
            println!("  nationality_name:     {:?}", v.nationality_name());
            println!("  reputation:           {}", v.reputation());
            println!("  state_of_development: {}", v.state_of_development());
            println!("  league_standard:      {}", v.league_standard());
            println!("  capital_city_id:      {:?}", v.capital_city_id());
            println!("  national_stadium_id:  {:?}", v.national_stadium_id());
            println!("  rivals: {:?} / {:?} / {:?}",
                     v.rival_nation_1(), v.rival_nation_2(), v.rival_nation_3());
            break;
        }
    }
}
