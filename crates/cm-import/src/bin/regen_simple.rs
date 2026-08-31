//! Placeholder for smaller reference-record regens (city/colour/continent/
//! stadium/official). Their DFM field lists are small (10-19 fields each);
//! per-record layout decodes are dispatched separately.
//!
//! Currently reports the file sizes of each; the actual field-by-field regen
//! is coming as each record type's layout is nailed down.

use std::path::PathBuf;

fn main() {
    let db_dir = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    for (name, path, dfm_field_count, record_size_bytes) in [
        ("city",         "references/cities.json",           10,   56),
        ("colour",       "core/colours.json",                11,   58),
        ("continent",    "core/continents.json",             10,  198),
        ("stadium",      "references/stadiums.json",         12,   78),
        ("official",     "references/officials.json",        19,   43),
        ("club_comp",    "references/club_competitions.json", 14,  107),
        ("staff_comp",   "references/staff_competitions.json", 8,  101),
    ] {
        let p = db_dir.join(path);
        let sz = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        let exists = if p.exists() { "OK" } else { "MISSING" };
        println!("{name:<12} DFM fields={dfm_field_count:<2}  disk record={record_size_bytes:<4} bytes  {exists}  {} ({sz} bytes on disk)",
                 p.display());
    }
    println!("\n(Full field-by-field rename per record is pending targeted decode.)");
}
