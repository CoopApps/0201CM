//! cm-import — build or verify the native Rust database.
//!
//! Modes:
//!   cm-import import --from D:/cm0102 --to D:/cm0102-rs/rust-db
//!       Read the original install's Data/*.dat and (re)write rust-db.
//!       This is the ONLY place .dat files are read; the game itself
//!       opens rust-db through cm-db and never sees a .dat.
//!
//!   cm-import verify --db D:/cm0102-rs/rust-db
//!       Open the database through cm-db (the same path the game uses),
//!       print per-table record counts against the shipping-data counts,
//!       and exit non-zero if any required table is missing.

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("import") => run_import(&args[1..]),
        Some("verify") => run_verify(&args[1..]),
        _ => {
            eprintln!("usage: cm-import import --from <install_dir> --to <rust_db_dir>");
            eprintln!("       cm-import verify --db <rust_db_dir>");
            ExitCode::from(2)
        }
    }
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn run_import(args: &[String]) -> ExitCode {
    let from = flag(args, "--from").unwrap_or("D:/cm0102");
    let to = flag(args, "--to").unwrap_or("D:/cm0102-rs/rust-db");
    println!("importing {from} -> {to}");

    let world = match cm_domain::World::load_from_install(Path::new(from)) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("FAILED to load install at {from}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = world.write_rust_db_dir(Path::new(to), Some(Path::new(from))) {
        eprintln!("FAILED to write rust-db at {to}: {e}");
        return ExitCode::FAILURE;
    }

    // RNG table — the 51,000-entry precomputed random table the game walks
    // (FUN_008fc4f0), lifted from cm0102.exe .data at 0x00a8df38. Not a .dat
    // record; extracted into rust-db/config/ so the Rust game's RNG is
    // bit-exact without the exe.
    let exe_path = Path::new(from).join("cm0102.exe");
    match extract_rng_table(&exe_path) {
        Ok(bytes) => {
            let cdir = Path::new(to).join("config");
            let _ = std::fs::create_dir_all(&cdir);
            if let Err(e) = std::fs::write(cdir.join("rng_table.bin"), &bytes) {
                eprintln!("WARNING: failed to write rng_table.bin: {e}");
            } else {
                println!("rng table imported: {} bytes ({} entries).", bytes.len(), bytes.len() / 4);
            }
        }
        Err(e) => eprintln!("WARNING: RNG table extraction failed: {e}"),
    }

    // Config-file data (weather.cfg + *.pct tactics) — not part of the .dat
    // record database, imported separately into rust-db/config/.
    let data_dir = Path::new(from).join("Data");
    match cm_db::ConfigData::import_from_data_dir(&data_dir) {
        Ok(cfg) => {
            if let Err(e) = cfg.write_dir(Path::new(to)) {
                eprintln!("WARNING: failed to write config data: {e}");
            } else {
                println!(
                    "config imported: {} weather configs, {} tactic templates.",
                    cfg.weather.len(),
                    cfg.tactics.len()
                );
            }
        }
        Err(e) => eprintln!("WARNING: config import failed: {e}"),
    }

    println!("import complete.");
    // Immediately verify through the game's own open path.
    run_verify(&["--db".to_string(), to.to_string()])
}

/// Extract the RNG ring-buffer table (0x00a8df38 .. 0x00abfc14 inclusive =
/// 51,000 i32 entries) from cm0102.exe by parsing the PE section map.
fn extract_rng_table(exe_path: &Path) -> std::io::Result<Vec<u8>> {
    const TABLE_BASE: u32 = 0x00a8_df38;
    const TABLE_END_INCL: u32 = 0x00ab_fc14;
    let len = (TABLE_END_INCL - TABLE_BASE + 4) as usize; // 204,000 bytes
    let exe = std::fs::read(exe_path)?;
    let u32le = |o: usize| u32::from_le_bytes([exe[o], exe[o + 1], exe[o + 2], exe[o + 3]]);
    let u16le = |o: usize| u16::from_le_bytes([exe[o], exe[o + 1]]);
    let nt = u32le(0x3c) as usize;
    let n_sections = u16le(nt + 6) as usize;
    let opt_size = u16le(nt + 0x14) as usize;
    let image_base = u32le(nt + 0x18 + 0x1c);
    let sect_off = nt + 0x18 + opt_size;
    let rva = TABLE_BASE - image_base;
    for i in 0..n_sections {
        let s = sect_off + i * 0x28;
        let v_size = u32le(s + 8);
        let v_addr = u32le(s + 12);
        let raw_size = u32le(s + 16);
        let raw_ptr = u32le(s + 20);
        let span = v_size.max(raw_size);
        if rva >= v_addr && rva < v_addr + span {
            let file_off = (raw_ptr + (rva - v_addr)) as usize;
            if file_off + len <= exe.len() {
                return Ok(exe[file_off..file_off + len].to_vec());
            }
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "RNG table VA not found in any PE section",
    ))
}

fn run_verify(args: &[String]) -> ExitCode {
    let db_dir = flag(args, "--db").unwrap_or("D:/cm0102-rs/rust-db");
    println!("opening {db_dir} via cm_db::Database::open (the game's path) ...");
    let db = match cm_db::Database::open(Path::new(db_dir)) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("OPEN FAILED: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("{:<24} {:>10} {:>10}  status", "table", "found", "shipped");
    let mut mismatches = 0usize;
    for t in &db.tables {
        let status = if t.matches_shipping() {
            "ok"
        } else {
            mismatches += 1;
            "DIFFERS"
        };
        println!("{:<24} {:>10} {:>10}  {status}", t.name, t.found, t.shipped);
    }
    if db.is_pristine_shipping_data() {
        println!("\nall 22 tables match the shipping v3.9.60 data exactly.");
    } else {
        println!("\n{mismatches} table(s) differ from shipping counts (allowed for edited databases).");
    }
    ExitCode::SUCCESS
}
