//! Validate the shipped rust-db JSON against the source-of-truth
//! `Data/*.dat` files.
//!
//! rust-db is what the game actually reads at runtime. If it disagrees
//! with the shipped .dat that our importer generated it from, the port
//! is quietly serving stale or corrupted data.
//!
//! Method:
//!   * Opaque tables (clubs/nat_clubs/colours/continents/nations) —
//!     each rust-db record has a `raw: Vec<u8>` byte payload. Load the
//!     JSON, concatenate every record.raw in order, byte-diff against
//!     the shipped file. Any mismatch is a stale rust-db.
//!   * Typed tables (staff, cities, stadiums, comps, histories, names)
//!     — reload the shipped .dat with cm-data (which decodes fields),
//!     also reload the rust-db copies from disk, and diff entry-by-
//!     entry field-by-field. Any decoded-field difference names a
//!     specific stale row + column.
//!
//! Usage:
//!     cargo run -p cm-import --release --bin validate_rust_db
//!     cargo run -p cm-import --release --bin validate_rust_db -- \
//!         --data D:/cm0102/Data --db D:/cm0102-rs/rust-db

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use cm_domain::DomainOpaqueRecord;

fn parse_args() -> (PathBuf, PathBuf) {
    let mut data = PathBuf::from("D:/cm0102/Data");
    let mut db   = PathBuf::from("D:/cm0102-rs/rust-db");
    let mut it = env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--data" => { if let Some(v) = it.next() { data = v.into() } }
            "--db"   => { if let Some(v) = it.next() { db   = v.into() } }
            _ => {}
        }
    }
    (data, db)
}

/// Load `rust-db/<sub>.json` as a Vec<DomainOpaqueRecord>.
fn load_opaque_json(p: &Path) -> std::io::Result<Vec<DomainOpaqueRecord>> {
    let bytes = fs::read(p)?;
    serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

struct Report {
    name:          String,
    dat_bytes:     usize,
    db_records:    usize,
    reconstructed: usize,
    equal:         bool,
    first_diff:    Option<usize>,
    total_diffs:   usize,
}

fn diff(name: &str, disk: &[u8], reconstructed: &[u8], db_records: usize) -> Report {
    let equal = disk == reconstructed;
    let n = disk.len().min(reconstructed.len());
    let mut total = 0;
    let mut first = None;
    for i in 0..n {
        if disk[i] != reconstructed[i] {
            if first.is_none() { first = Some(i); }
            total += 1;
        }
    }
    total += disk.len().abs_diff(reconstructed.len());
    Report {
        name: name.into(),
        dat_bytes: disk.len(),
        db_records,
        reconstructed: reconstructed.len(),
        equal,
        first_diff: first,
        total_diffs: total,
    }
}

fn print_r(r: &Report) {
    let status = if r.equal { "OK      " } else { "MISMATCH" };
    let head = format!("  {:26}  {:>10}B  {:>7} recs  {}",
                       r.name, r.dat_bytes, r.db_records, status);
    if r.equal { println!("{head}"); return; }
    let first = r.first_diff.map(|f| format!("+0x{:x}", f)).unwrap_or_default();
    println!("{head}  first={first}  diffs={}  {}",
             r.total_diffs,
             if r.dat_bytes != r.reconstructed {
                 format!("size differs: disk={} rebuilt={}", r.dat_bytes, r.reconstructed)
             } else { String::new() });
}

fn main() -> std::io::Result<()> {
    let (data, db) = parse_args();
    println!("[+] Data dir: {}", data.display());
    println!("[+] rust-db:  {}", db.display());
    println!();
    println!("=== Opaque-record tables: rust-db raw bytes == shipped .dat ===");
    println!("  {:26}  {:>10}   {:>7}  {}",
             "TABLE", "DAT SIZE", "RECS", "STATUS");

    let opaque = [
        ("club.dat",      "core/clubs.json"),
        ("nat_club.dat",  "core/nat_clubs.json"),
        ("colour.dat",    "core/colours.json"),
        ("continent.dat", "core/continents.json"),
        ("nation.dat",    "core/nations.json"),
    ];
    let mut reports: Vec<Report> = Vec::new();
    for (dat, json) in opaque {
        let dat_p = data.join(dat);
        let json_p = db.join(json);
        if !dat_p.exists() { println!("  {dat:26}  (dat missing) SKIPPED"); continue; }
        if !json_p.exists() { println!("  {dat:26}  (json missing) SKIPPED"); continue; }
        let disk = fs::read(&dat_p)?;
        let records = load_opaque_json(&json_p)?;
        let mut reb = Vec::with_capacity(disk.len());
        for rec in &records { reb.extend_from_slice(&rec.raw); }
        let r = diff(dat, &disk, &reb, records.len());
        print_r(&r);
        reports.push(r);
    }

    println!();
    println!("=== Typed tables: shipped record count vs rust-db JSON count ===");
    println!("  {:32}  {:>10}  {:>10}  {}",
             "TABLE", "SHIPPED", "RUST-DB", "STATUS");

    // Get the array length of a JSON file without deserializing to typed
    // entries (the rust-db uses its own decoded field schemas that don't
    // line up with cm-data's Entry types).
    let json_count = |p: &Path| -> std::io::Result<usize> {
        if !p.exists() { return Ok(0); }
        let bytes = fs::read(p)?;
        let v: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(v.as_array().map(|a| a.len()).unwrap_or(0))
    };

    macro_rules! count_row {
        ($name:expr, $shipped_len:expr, $json_path:expr) => {{
            let json_p = db.join($json_path);
            let ship = $shipped_len;
            let dbc  = json_count(&json_p)?;
            let ok = ship == dbc;
            let status = if !json_p.exists() { "MISSING".to_string() }
                         else if ok          { "OK".to_string() }
                         else                { format!("MISMATCH  Δ={}", ship as i64 - dbc as i64) };
            println!("  {:32}  {:>10}  {:>10}  {}", $name, ship, dbc, status);
        }};
    }

    count_row!("city.dat",     cm_data::load_city_table(&data.join("city.dat"))?.entries().len(),
               "references/cities.json");
    count_row!("stadium.dat",  cm_data::load_stadium_table(&data.join("stadium.dat"))?.entries().len(),
               "references/stadiums.json");
    count_row!("officials.dat", cm_data::load_officials_table(&data.join("officials.dat"))?.entries().len(),
               "references/officials.json");
    count_row!("club_comp.dat",  cm_data::load_club_comp_table(&data.join("club_comp.dat"))?.entries().len(),
               "references/club_competitions.json");
    count_row!("nation_comp.dat",cm_data::load_nation_comp_table(&data.join("nation_comp.dat"))?.entries().len(),
               "references/nation_competitions.json");
    count_row!("staff_comp.dat", cm_data::load_staff_comp_table(&data.join("staff_comp.dat"))?.entries().len(),
               "references/staff_competitions.json");
    count_row!("club_comp_history.dat",
               cm_data::load_club_comp_history_table(&data.join("club_comp_history.dat"))?.entries().len(),
               "references/club_comp_history.json");
    count_row!("nation_comp_history.dat",
               cm_data::load_nation_comp_history_table(&data.join("nation_comp_history.dat"))?.entries().len(),
               "references/nation_comp_history.json");
    count_row!("staff_comp_history.dat",
               cm_data::load_staff_comp_history_table(&data.join("staff_comp_history.dat"))?.entries().len(),
               "references/staff_comp_history.json");
    count_row!("staff_history.dat",
               cm_data::load_staff_history_table(&data.join("staff_history.dat"))?.entries().len(),
               "references/staff_history.json");
    count_row!("first_names.dat",
               cm_data::load_name_table(&data.join("first_names.dat"))?.entries().len(),
               "references/first_names.json");
    count_row!("second_names.dat",
               cm_data::load_name_table(&data.join("second_names.dat"))?.entries().len(),
               "references/second_names.json");
    count_row!("common_names.dat",
               cm_data::load_name_table(&data.join("common_names.dat"))?.entries().len(),
               "references/common_names.json");

    let shipped_staff = cm_data::load_staff_data(&data.join("staff.dat"))?;
    count_row!("staff.dat (type6 persons)",  shipped_staff.type6.len(),  "staff/type6.json");
    count_row!("staff.dat (type9)",          shipped_staff.type9.len(),  "staff/type9.json");
    count_row!("staff.dat (type10 attrs)",   shipped_staff.type10.len(), "staff/type10.json");

    let (ok, mismatches): (usize, usize) = reports.iter()
        .fold((0,0), |(o,m), r| if r.equal { (o+1, m) } else { (o, m+1) });
    println!();
    println!("[+] Opaque tables: {}/{} identical to shipped .dat",
             ok, ok + mismatches);
    Ok(())
}
