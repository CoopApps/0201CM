//! Load every shipped Data/*.dat that cm-data can round-trip, write it
//! back to a scratch file with cm-data's writer, and byte-diff the result
//! against the original.
//!
//! Any mismatch = wrong field type / offset in our loader-writer pair —
//! exactly the u16-vs-u32 bug class that gave us the DomainStaffType6
//! surname_id truncation. The intra-record histogram groups every diff
//! by (byte_offset % stride) so one broken field surfaces as thousands
//! of hits at one offset, not scattered noise.
//!
//! The shipped .dat file IS what the editor loads too (we verified the
//! editor's colour records match the shipped file byte-exact). So this
//! round-trip is the strongest data validation we can run.
//!
//! Usage:
//!     cargo run -p cm-import --release --bin roundtrip_all
//!     cargo run -p cm-import --release --bin roundtrip_all -- --data D:/cm0102/Data

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn parse_data_arg() -> PathBuf {
    let mut it = env::args().skip(1);
    let mut data = PathBuf::from("D:/cm0102/Data");
    while let Some(a) = it.next() {
        if a == "--data" {
            if let Some(v) = it.next() { data = PathBuf::from(v); }
        }
    }
    data
}

struct DiffReport {
    name: String, stride: usize,
    orig_size: usize, round_size: usize,
    equal: bool, first_diff: Option<usize>,
    total_diffs: usize,
    per_intra_offset: BTreeMap<usize, usize>,
}

fn diff_bytes(name: &str, stride: usize, orig: &[u8], round: &[u8]) -> DiffReport {
    let mut r = DiffReport {
        name: name.into(), stride,
        orig_size: orig.len(), round_size: round.len(),
        equal: orig == round, first_diff: None, total_diffs: 0,
        per_intra_offset: BTreeMap::new(),
    };
    if r.equal { return r; }
    let n = orig.len().min(round.len());
    for i in 0..n {
        if orig[i] != round[i] {
            if r.first_diff.is_none() { r.first_diff = Some(i); }
            r.total_diffs += 1;
            let intra = if stride > 0 { i % stride } else { i };
            *r.per_intra_offset.entry(intra).or_insert(0) += 1;
        }
    }
    r.total_diffs += orig.len().abs_diff(round.len());
    r
}

fn print_report(r: &DiffReport) {
    let status = if r.equal { "OK      " } else { "MISMATCH" };
    let head = format!("  {:26}  {:>10}B  {}", r.name, r.orig_size, status);
    if r.equal { println!("{head}"); return; }
    let first = r.first_diff.map(|f| format!("+0x{:x}", f)).unwrap_or_default();
    let mut top: Vec<_> = r.per_intra_offset.iter().collect();
    top.sort_by(|a, b| b.1.cmp(a.1));
    let top_str = top.into_iter().take(4)
        .map(|(o, c)| format!("+0x{:x}({})", o, c))
        .collect::<Vec<_>>().join(", ");
    println!("{head}  first={first}  diffs={}  intra: {}",
             r.total_diffs, top_str);
    if r.orig_size != r.round_size {
        println!("    !! size differs: orig={}  round={}",
                 r.orig_size, r.round_size);
    }
}

macro_rules! table_rt {
    ($reports:ident, $name:expr, $stride:expr, $data:ident, $tmp:ident,
     $load:path, $write:path) => {{
        let src = $data.join($name);
        if !src.exists() {
            println!("  {:26}  (missing)  SKIPPED", $name);
        } else {
            let orig = fs::read(&src)?;
            let table = $load(&src)?;
            let entries = table.entries();
            let dst = $tmp.join($name);
            $write(&dst, &entries)?;
            let round = fs::read(&dst)?;
            let r = diff_bytes($name, $stride, &orig, &round);
            print_report(&r);
            $reports.push(r);
        }
    }};
}

fn main() -> std::io::Result<()> {
    let data = parse_data_arg();
    let tmp = std::env::temp_dir().join("cm0102-roundtrip");
    fs::create_dir_all(&tmp)?;
    println!("[+] Data dir: {}", data.display());
    println!("[+] Scratch:  {}", tmp.display());
    println!();
    println!("  {:26}  {:>10}   {}", "TABLE", "SIZE", "STATUS");

    let mut reports: Vec<DiffReport> = Vec::new();

    // Strides from cm-data's own FixedRecordLayout constants — these
    // are the record widths the loader/writer actually assumes.
    table_rt!(reports, "city.dat",              56, data, tmp,
              cm_data::load_city_table,             cm_data::write_city_table);
    table_rt!(reports, "stadium.dat",           78, data, tmp,
              cm_data::load_stadium_table,          cm_data::write_stadium_table);
    table_rt!(reports, "officials.dat",         43, data, tmp,
              cm_data::load_officials_table,        cm_data::write_officials_table);
    table_rt!(reports, "club_comp.dat",        107, data, tmp,
              cm_data::load_club_comp_table,        cm_data::write_club_comp_table);
    table_rt!(reports, "nation_comp.dat",      107, data, tmp,
              cm_data::load_nation_comp_table,      cm_data::write_nation_comp_table);
    table_rt!(reports, "staff_comp.dat",       101, data, tmp,
              cm_data::load_staff_comp_table,       cm_data::write_staff_comp_table);
    table_rt!(reports, "club_comp_history.dat", 26, data, tmp,
              cm_data::load_club_comp_history_table,
              cm_data::write_club_comp_history_table);
    table_rt!(reports, "nation_comp_history.dat", 26, data, tmp,
              cm_data::load_nation_comp_history_table,
              cm_data::write_nation_comp_history_table);
    table_rt!(reports, "staff_comp_history.dat", 58, data, tmp,
              cm_data::load_staff_comp_history_table,
              cm_data::write_staff_comp_history_table);
    table_rt!(reports, "staff_history.dat",    17, data, tmp,
              cm_data::load_staff_history_table,    cm_data::write_staff_history_table);
    table_rt!(reports, "first_names.dat",      60, data, tmp,
              cm_data::load_name_table,             cm_data::write_name_table);
    table_rt!(reports, "second_names.dat",     60, data, tmp,
              cm_data::load_name_table,             cm_data::write_name_table);
    table_rt!(reports, "common_names.dat",     60, data, tmp,
              cm_data::load_name_table,             cm_data::write_name_table);

    // staff.dat is multi-section (type6 157B / type9 68B / type10 70B).
    {
        let src = data.join("staff.dat");
        if src.exists() {
            let orig = fs::read(&src)?;
            let staff = cm_data::load_staff_data(&src)?;
            let dst = tmp.join("staff.dat");
            cm_data::write_staff_data(&dst, &staff)?;
            let round = fs::read(&dst)?;
            let r = diff_bytes("staff.dat", 157, &orig, &round);
            print_report(&r);
            reports.push(r);
        }
    }

    let ok = reports.iter().filter(|r| r.equal).count();
    println!();
    println!("[+] {}/{} tables round-trip byte-perfect.",
             ok, reports.len());

    let json = serde_json::json!({
        "data_dir": data.display().to_string(),
        "tables": reports.iter().map(|r| serde_json::json!({
            "table":            r.name,
            "stride":           r.stride,
            "orig_size":        r.orig_size,
            "round_size":       r.round_size,
            "byte_perfect":     r.equal,
            "first_diff":       r.first_diff,
            "total_diffs":      r.total_diffs,
            "per_intra_offset": r.per_intra_offset,
        })).collect::<Vec<_>>(),
    });
    let out = Path::new("reports/roundtrip_all.json");
    fs::create_dir_all(out.parent().unwrap())?;
    let mut f = fs::File::create(out)?;
    write!(f, "{}", serde_json::to_string_pretty(&json).unwrap())?;
    println!("[+] wrote {}", out.display());
    Ok(())
}
