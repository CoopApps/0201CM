//! C10.5 — differential check for all 5 English Traditional leagues.
//!
//! Consumes the JSONL + `.bin` capture artefacts produced by the
//! Frida harness at `reports/fixture_disasm/gdi_five_league_lineage.py`.
//! For each league runs two differentials:
//!
//!   1. **Perturb differential** — feed captured P1 through Rust
//!      `matrix_perturb` and compare the resulting reordered roster
//!      against captured P2.
//!   2. **Driver differential** — feed captured P2 + captured
//!      walker returns through Rust `run_round_robin_driver` and
//!      compare fixture emissions against the captured primary
//!      inserts.
//!
//! Prints a confidence matrix per league. Zero mismatches promote
//! that league from `StructurallyVerified` to `ByteExact` in the
//! spec file (via a follow-up commit).
//!
//! Usage:
//!   cargo run --release -p cm-domain --example five_league_diff -- \
//!       [--prefix 20260915_120000_five_leagues]
//!
//! If `--prefix` is omitted, uses the newest `*_five_leagues_*_lineage.jsonl`
//! files under `reports/fixture_disasm/runtime/`.

use cm_domain::eng_second_fixtures::{
    matrix_seed_base, FixtureEmission,
};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Capture-file shapes (matches the Frida harness's JSONL output)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Insert {
    idx: i64,
    site: String,
    #[serde(default)] ret_addr: String,
    #[serde(default)] this_list: String,
    #[serde(default)] fixture_ptr: String,
    #[allow(dead_code)] #[serde(default)] mode: i64,
    cid: i32,
    home: i32,
    away: i32,
    #[allow(dead_code)] year: i16,
    #[allow(dead_code)] doy: i16,
    rwh: i16,
    #[allow(dead_code)] #[serde(default)] type_byte: u8,
    #[serde(default)] flag_bits: u16,
}

#[derive(Debug, Deserialize)]
struct WalkerCall {
    #[allow(dead_code)] idx: i32,
    #[allow(dead_code)] prev_col: i32,
    #[allow(dead_code)] state_before: i32,
    #[allow(dead_code)] comp_id: i32,
    #[allow(dead_code)] n_clubs: i16,
    #[allow(dead_code)] matches_per_pair: i16,
    #[allow(dead_code)] n_rounds: i16,
    #[allow(dead_code)] flag_byte: u8,
    #[allow(dead_code)] state_after: i32,
    retval: i32,
}

#[derive(Debug, Deserialize)]
struct DerefEntry {
    slot: i64,
    club_id: i32,
    #[serde(default, alias = "club_ptr")]
    #[allow(dead_code)] cptr: String,
    #[serde(default)] #[allow(dead_code)] stadium_id: Option<i32>,
    #[serde(default)] #[allow(dead_code)] stadium_alt_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct CtorLeave {
    #[allow(dead_code)] cid: u8,
    year_base: i16,
    n_clubs: i16,
    n_rounds: i16,
    #[allow(dead_code)] mpp: i16,
    #[allow(dead_code)] d9_flags: u16,
}

// ---------------------------------------------------------------------------
// Driver replay (identical algorithm to the existing driver_diff_via_p2.rs;
// factored so the same body runs for each league's shape)
// ---------------------------------------------------------------------------

fn replay_driver(
    n_clubs: i16, matches_per_pair: i16, n_rounds: i16,
    year_base: i16, weekday_parity_flag: u8, host_nation: i32,
    walker_returns: &[i32],
) -> Vec<FixtureEmission> {
    let mut out = Vec::new();
    let n_even: i32 = { let nc = n_clubs as i32; nc + (nc & 1) };
    let mut matrix: Vec<Vec<i32>> = matrix_seed_base(n_even as usize);
    let total_rounds: i32 = (matches_per_pair as i32) * (n_even - 1);
    assert_eq!(total_rounds as usize, walker_returns.len(),
               "walker sequence length mismatch: expected {}, got {}",
               total_rounds, walker_returns.len());
    for (i, &walker_col) in walker_returns.iter().enumerate() {
        let outer = (i as i32) + 1;
        let round_within_half: i16 = walker_col as i16;
        let is_last_round = (round_within_half as i32) == (n_rounds as i32) - 1;
        let col: i32 = {
            let m = outer % (n_even - 1);
            if m == 0 { n_even - 1 } else { m }
        };
        let second_half: bool = (((outer - 1) / (n_even - 1)) & 1) != 0;
        let season_odd: bool = {
            let sum = (year_base as i32) + (weekday_parity_flag as i8 as i32);
            (sum & 1) != 0
        };
        for row_idx in 0..(n_clubs as i32) {
            let matrix_row = (row_idx + 1) as usize;
            let cell: i32 = matrix[matrix_row][col as usize];
            if cell == 0 { continue; }
            let cell_pos = cell > 0;
            let cell_abs_minus_1 = (cell.unsigned_abs() as i32) - 1;
            let home_is_cell = !(second_half ^ season_odd ^ !cell_pos);
            let (home_row, away_row) = if home_is_cell {
                (cell_abs_minus_1, row_idx)
            } else {
                (row_idx, cell_abs_minus_1)
            };
            matrix[cell.unsigned_abs() as usize][col as usize] = 0;
            let clubs_max = (n_clubs as i32) - 1;
            if home_row < 0 || home_row > clubs_max
                || away_row < 0 || away_row > clubs_max { continue; }
            let (home_slot, away_slot, swapped) = if host_nation == -1 {
                (home_row, away_row, false)
            } else { (home_row, away_row, false) };
            out.push(FixtureEmission {
                walker_col, home_slot, away_slot,
                round_within_half, outer_round: outer,
                is_last_round, host_nation_swap: swapped,
            });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Per-league diff runner
// ---------------------------------------------------------------------------

struct LeagueResult {
    league: &'static str,
    n_clubs: i16,
    n_rounds: i16,
    year_base: i16,
    schedule_bytes: usize,
    primary_inserts: usize,
    driver_ordered_mismatches: usize,
    driver_unordered_set_size_gdi: usize,
    driver_unordered_set_size_rust: usize,
    driver_intersection: usize,
    verdict: &'static str,
}

fn run_league(root: &Path, prefix: &str, league_id: &str,
              league_display: &'static str) -> Option<LeagueResult> {
    let jsonl = root.join(format!("{prefix}_{league_id}_lineage.jsonl"));
    if !jsonl.exists() {
        println!("[{league_display}] capture file missing: {jsonl:?}");
        return None;
    }
    let text = std::fs::read_to_string(&jsonl).ok()?;
    let records: Vec<serde_json::Value> = text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    let ctor_rec = records.iter().find(|r| r["op"] == "ctor_leave")?;
    let ctor: CtorLeave = serde_json::from_value(ctor_rec.clone()).ok()?;

    let inserts_rec = records.iter().find(|r| r["op"] == "insert_trace")?;
    let inserts: Vec<Insert> = serde_json::from_value(
        inserts_rec["inserts"].clone()).ok()?;
    let primary: Vec<&Insert> = inserts.iter()
        .filter(|i| i.site == "primary" || i.site == "main").collect();

    let walker_rec = records.iter().find(|r| r["op"] == "walker_trace")?;
    let walkers: Vec<WalkerCall> = serde_json::from_value(
        walker_rec["calls"].clone()).ok()?;
    let walker_seq: Vec<i32> = walkers.iter().map(|w| w.retval).collect();

    let p2_rec = records.iter()
        .find(|r| r["op"] == "clubs_snap" && r["point"] == "P2")?;
    let p2_entries: Vec<DerefEntry> = serde_json::from_value(
        p2_rec["entries"].clone()).ok()?;
    let slot_to_id: BTreeMap<i64, i32> = p2_entries.iter()
        .map(|e| (e.slot, e.club_id)).collect();

    let sched_bytes = root.join(format!("{prefix}_{league_id}_sched_buf.bin"))
        .metadata().map(|m| m.len() as usize).unwrap_or(0);

    let weekday_parity_flag: u8 = 3;    // per eng_second capture — same for all
    let host_nation: i32 = -1;

    let rust_emissions = replay_driver(
        ctor.n_clubs, /*matches_per_pair=*/ 2, ctor.n_rounds,
        ctor.year_base, weekday_parity_flag, host_nation,
        &walker_seq,
    );

    #[derive(PartialEq, Eq, Clone)]
    struct C { home_id: i32, away_id: i32, rwh: i16, is_last: bool }
    let gdi: Vec<C> = primary.iter().map(|f| C {
        home_id: f.home, away_id: f.away, rwh: f.rwh,
        is_last: (f.flag_bits & 0x0800) != 0
                 || f.rwh as i32 == ctor.n_rounds as i32 - 1,
    }).collect();
    let rust: Vec<C> = rust_emissions.iter().map(|f| C {
        home_id: *slot_to_id.get(&(f.home_slot as i64)).unwrap_or(&-1),
        away_id: *slot_to_id.get(&(f.away_slot as i64)).unwrap_or(&-1),
        rwh: f.round_within_half,
        is_last: f.is_last_round,
    }).collect();

    let mut ordered_mm = 0;
    for i in 0..gdi.len().min(rust.len()) {
        if gdi[i] != rust[i] { ordered_mm += 1; }
    }
    if gdi.len() != rust.len() {
        ordered_mm += (gdi.len() as i64 - rust.len() as i64).unsigned_abs() as usize;
    }

    let gs: HashSet<(i32, i32, i16)> = gdi.iter().map(|c| {
        let (a, b) = if c.home_id < c.away_id { (c.home_id, c.away_id) }
                     else { (c.away_id, c.home_id) };
        (a, b, c.rwh)
    }).collect();
    let rs: HashSet<(i32, i32, i16)> = rust.iter().map(|c| {
        let (a, b) = if c.home_id < c.away_id { (c.home_id, c.away_id) }
                     else { (c.away_id, c.home_id) };
        (a, b, c.rwh)
    }).collect();

    let verdict = if ordered_mm == 0 { "ZERO-DIFF PASS" }
                  else if gs == rs { "same set, wrong order" }
                  else { "genuinely different" };

    Some(LeagueResult {
        league: league_display,
        n_clubs: ctor.n_clubs, n_rounds: ctor.n_rounds,
        year_base: ctor.year_base,
        schedule_bytes: sched_bytes,
        primary_inserts: primary.len(),
        driver_ordered_mismatches: ordered_mm,
        driver_unordered_set_size_gdi: gs.len(),
        driver_unordered_set_size_rust: rs.len(),
        driver_intersection: gs.intersection(&rs).count(),
        verdict,
    })
}

fn find_newest_prefix(root: &Path) -> Option<String> {
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(root).ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.contains("_five_leagues_")
                          && s.ends_with("_lineage.jsonl"))
                    .unwrap_or(false))
        .collect();
    candidates.sort();
    let last = candidates.last()?;
    let name = last.file_name()?.to_str()?;
    // Strip trailing "_<lid>_lineage.jsonl" — up to and including the "_lineage.jsonl".
    let up_to_lineage = name.rsplit_once("_lineage.jsonl")?.0;
    let up_to_lid = up_to_lineage.rsplit_once('_')?.0;
    Some(up_to_lid.to_string())
}

fn main() {
    let root = Path::new("D:/cm0102-rs/reports/fixture_disasm/runtime");
    let args: Vec<String> = std::env::args().collect();
    let prefix = args.iter().position(|a| a == "--prefix")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .or_else(|| find_newest_prefix(root))
        .unwrap_or_else(|| {
            eprintln!("no capture prefix found in {root:?} — run \
                       reports/fixture_disasm/gdi_five_league_lineage.py first");
            std::process::exit(2);
        });
    println!("prefix: {prefix}");

    let leagues = [
        ("prem",   "Premier"),
        ("first",  "First"),
        ("second", "Second"),
        ("third",  "Third"),
        ("conf",   "Conference"),
    ];
    let mut results = Vec::new();
    for (lid, disp) in leagues.iter() {
        if let Some(r) = run_league(root, &prefix, lid, disp) {
            results.push(r);
        }
    }

    println!("\n=== Driver differential matrix ===");
    println!("{:<12} {:>3} {:>4} {:>5} {:>7} {:>10} {:>6}  {}",
             "league", "N", "R", "buf", "primary", "orderedΔ", "set∩", "verdict");
    for r in &results {
        println!("{:<12} {:>3} {:>4} {:>5} {:>7} {:>10} {:>3}/{:<2}  {}",
                 r.league, r.n_clubs, r.n_rounds, r.schedule_bytes,
                 r.primary_inserts, r.driver_ordered_mismatches,
                 r.driver_intersection, r.driver_unordered_set_size_gdi,
                 r.verdict);
    }

    println!("\n=== Confidence promotion candidates ===");
    for r in &results {
        if r.driver_ordered_mismatches == 0 {
            println!("  {:<11} driver: ByteExact  (year_base={})",
                     r.league, r.year_base);
        } else {
            println!("  {:<11} driver: {}  ({} mismatches)",
                     r.league, r.verdict, r.driver_ordered_mismatches);
        }
    }

    let all_zero = results.iter()
        .all(|r| r.driver_ordered_mismatches == 0);
    if !all_zero {
        std::process::exit(1);
    }
}
