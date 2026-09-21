//! Save/load golden differential (the strongest equivalence test):
//!
//!   stateA -> save -> load -> tick 1 day     (TEST)
//!   stateA ->              -> tick 1 day     (CONTROL)
//!
//! must produce byte-identical serialized state (RuntimeSaveGame AND the
//! mutated World). Since serde_json output is deterministic for identical data,
//! equal state <=> equal JSON. Also asserts the season_roll_scheduler survives
//! the round-trip (the #[serde(default)] danger field).
//!
//! Runs the real production tick path `tick_days_bound(&mut World, 1)`.
//!
//! Run: RUSTFLAGS="-C debuginfo=0" cargo run -q -p cm-domain --bin save_load_golden

use std::path::{Path, PathBuf};

use cm_domain::{NewGameOptions, RuntimeSaveGame, World};

fn scratch() -> PathBuf {
    std::env::var("CM_SCRATCH")
        .unwrap_or_else(|_| "D:/temp/claude/save_load_golden".to_string())
        .into()
}

fn save_bundle(dir: &Path, save: &RuntimeSaveGame, world: &World) {
    std::fs::create_dir_all(dir).unwrap();
    // Stream straight to file (serde_json::to_writer) — a full World JSON is
    // ~1 GB and materializing it in memory (to_vec) OOMs on a low-RAM host.
    let ww = std::io::BufWriter::new(std::fs::File::create(dir.join("world.json")).unwrap());
    serde_json::to_writer(ww, world).unwrap();
    let sw = std::io::BufWriter::new(std::fs::File::create(dir.join("save.json")).unwrap());
    serde_json::to_writer(sw, save).unwrap();
}

fn load_bundle(dir: &Path) -> (RuntimeSaveGame, World) {
    let wr = std::io::BufReader::new(std::fs::File::open(dir.join("world.json")).unwrap());
    let world: World = serde_json::from_reader(wr).unwrap();
    let sr = std::io::BufReader::new(std::fs::File::open(dir.join("save.json")).unwrap());
    let save: RuntimeSaveGame = serde_json::from_reader(sr).unwrap();
    (save, world)
}

/// Stream-compare two files chunk-by-chunk (never loads either fully — the
/// World JSON is ~1 GB, so reading both into memory would OOM). Returns
/// (equal, len_a, len_b, first_diff_byte).
fn compare_files(a: &Path, b: &Path) -> (bool, u64, u64, Option<u64>) {
    use std::io::Read;
    let mut fa = std::io::BufReader::new(std::fs::File::open(a).unwrap());
    let mut fb = std::io::BufReader::new(std::fs::File::open(b).unwrap());
    let (mut ba, mut bb) = ([0u8; 65536], [0u8; 65536]);
    let mut pos: u64 = 0;
    let (mut la, mut lb) = (0u64, 0u64);
    let mut first: Option<u64> = None;
    loop {
        let na = fa.read(&mut ba).unwrap();
        let nb = fb.read(&mut bb).unwrap();
        la += na as u64;
        lb += nb as u64;
        let n = na.min(nb);
        if first.is_none() {
            for i in 0..n {
                if ba[i] != bb[i] {
                    first = Some(pos + i as u64);
                    break;
                }
            }
            if first.is_none() && na != nb {
                first = Some(pos + n as u64);
            }
        }
        pos += n as u64;
        if na == 0 && nb == 0 {
            break;
        }
        // If lengths diverged mid-stream, keep draining to get true lengths.
        if na != nb && (na == 0 || nb == 0) {
            // one file ended; drain the other for its length
            let mut tmp = [0u8; 65536];
            if na == 0 {
                while let Ok(k) = fb.read(&mut tmp) { if k == 0 { break } lb += k as u64; }
            } else {
                while let Ok(k) = fa.read(&mut tmp) { if k == 0 { break } la += k as u64; }
            }
            break;
        }
    }
    (first.is_none() && la == lb, la, lb, first)
}

fn main() {
    let rust_db = Path::new("D:/cm0102-rs/rust-db");
    let world0 = World::read_rust_db_dir(rust_db).expect("read rust-db");
    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
        initial_game_rng_state: None,
    };
    let mut world = world0;
    let mut save = world.new_game_from_rust_db(rust_db, &options);

    // Advance to stateA (a few days past kickoff so fixtures/finance/player
    // state have actually mutated).
    const WARMUP_DAYS: u32 = 4;
    for _ in 0..WARMUP_DAYS {
        save.tick_days_bound(&mut world, 1);
    }
    let sched_a = save.season_roll_scheduler.total_registered();
    println!(
        "stateA: date {:04}-{:02}-{:02}, fixtures={}, scheduler_registered={}",
        save.date.year, save.date.month, save.date.day,
        save.season.fixtures.len(), sched_a
    );

    let base = scratch();
    let a_dir = base.join("stateA");
    let control_dir = base.join("control");
    let test_dir = base.join("test");

    // Persist stateA (the save under test).
    save_bundle(&a_dir, &save, &world);

    // CONTROL: continue the ORIGINAL in-memory stateA one more day.
    save.tick_days_bound(&mut world, 1);
    save_bundle(&control_dir, &save, &world);
    // Free the control world before reloading, to keep peak memory to one World.
    drop(world);
    drop(save);

    // TEST: reload stateA from disk, continue one more day.
    let (mut save2, mut world2) = load_bundle(&a_dir);
    let sched_b = save2.season_roll_scheduler.total_registered();
    save2.tick_days_bound(&mut world2, 1);
    save_bundle(&test_dir, &save2, &world2);
    drop(world2);
    drop(save2);

    // Compare.
    let mut ok = true;
    if sched_a != sched_b {
        println!("FAIL: scheduler registrations changed across save/load: {sched_a} -> {sched_b}");
        ok = false;
    } else {
        println!("scheduler round-trip OK ({sched_a} registrations preserved)");
    }
    for name in ["save.json", "world.json"] {
        let (eq, la, lb, first) = compare_files(&control_dir.join(name), &test_dir.join(name));
        if eq {
            println!("MATCH: {name} identical ({la} bytes)");
        } else {
            ok = false;
            println!(
                "FAIL: {name} differs (control {la} bytes, test {lb} bytes, first diff at {:?})",
                first
            );
        }
    }
    if ok {
        println!("\nGOLDEN PASS: save -> load -> tick == tick (observable state identical)");
    } else {
        println!("\nGOLDEN FAIL");
        std::process::exit(1);
    }
}
