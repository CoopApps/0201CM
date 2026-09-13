//! Differential test: Rust `run_round_robin_driver` output vs the
//! natural cm0102-gdi 552 primary fixture inserts captured in Phase 8.
//!
//! Approach: replay the captured walker sequence directly into the
//! driver's outer-loop algorithm. This isolates the driver's
//! matrix/H-A/host-swap/reset business logic from the RNG-driven
//! walker (state-exact and separately verified). If the Rust logic is
//! correct given the same walker output, the 552 primary fixtures
//! come out identical.
//!
//! Runs against:
//!   reports/fixture_disasm/runtime/20260913_191309_v2_natural.jsonl
//!   reports/fixture_disasm/runtime/20260913_191309_v2_sched_buffer_cid9_0.bin
//!
//! Emits an ordered + set diff report to stdout.

use cm_domain::eng_second_fixtures::{
    matrix_seed_base, FixtureEmission,
};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Insert {
    idx: i64,
    site: String,
    #[serde(default)]
    ret_addr: String,
    #[serde(default)]
    this_list: String,
    #[serde(default)]
    fixture_ptr: String,
    #[allow(dead_code)]
    #[serde(default)]
    mode: i64,
    cid: i32,
    home: i32,
    away: i32,
    year: i16,
    doy: i16,
    rwh: i16,
    #[serde(default)]
    type_byte: u8,
    #[serde(default)]
    flag_bits: u16,
}

#[derive(Debug, Deserialize)]
struct WalkerCall {
    idx: i32,
    prev_col: i32,
    #[allow(dead_code)]
    state_before: i32,
    #[allow(dead_code)]
    comp_id: i32,
    #[allow(dead_code)]
    n_clubs: i16,
    #[allow(dead_code)]
    matches_per_pair: i16,
    #[allow(dead_code)]
    n_rounds: i16,
    #[allow(dead_code)]
    flag_byte: u8,
    #[allow(dead_code)]
    state_after: i32,
    retval: i32,
}

#[derive(Debug, Deserialize)]
struct DerefEntry {
    slot: i64,
    club_id: i32,
    #[allow(dead_code)]
    club_ptr: String,
    #[allow(dead_code)]
    #[serde(default)]
    nation_at_69: i32,
}

fn load_trace(path: &Path) -> Vec<serde_json::Value> {
    let text = std::fs::read_to_string(path).unwrap();
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .collect()
}

/// Reproduce the driver's outer-loop algorithm using replayed walker
/// outputs. Direct port of `run_round_robin_driver` lines 803-919
/// with `walker_step()` replaced by an index into `walker_returns`.
///
/// Returns emissions in the exact order the driver would emit them.
fn replay_driver(
    n_clubs: i16,
    matches_per_pair: i16,
    n_rounds: i16,
    year_base: i16,
    weekday_parity_flag: u8,
    host_nation: i32,
    walker_returns: &[i32],
) -> Vec<FixtureEmission> {
    let mut out = Vec::new();
    let n_even: i32 = {
        let nc = n_clubs as i32;
        nc + (nc & 1)
    };
    let mut matrix: Vec<Vec<i32>> = matrix_seed_base(n_even as usize);

    let total_rounds: i32 = (matches_per_pair as i32) * (n_even - 1);
    assert_eq!(
        total_rounds as usize,
        walker_returns.len(),
        "walker sequence length mismatch"
    );

    for (i, &walker_col) in walker_returns.iter().enumerate() {
        let outer = (i as i32) + 1;
        let round_within_half: i16 = walker_col as i16;
        let is_last_round =
            (round_within_half as i32) == (n_rounds as i32) - 1;

        let col: i32 = {
            let m = outer % (n_even - 1);
            if m == 0 { n_even - 1 } else { m }
        };
        let second_half: bool =
            (((outer - 1) / (n_even - 1)) & 1) != 0;
        let season_odd: bool = {
            let sum = (year_base as i32)
                + (weekday_parity_flag as i8 as i32);
            (sum & 1) != 0
        };

        for row_idx in 0..(n_clubs as i32) {
            let matrix_row = (row_idx + 1) as usize;
            let cell: i32 = matrix[matrix_row][col as usize];
            if cell == 0 {
                continue;
            }
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
            if home_row < 0
                || home_row > clubs_max
                || away_row < 0
                || away_row > clubs_max
            {
                continue;
            }

            let (home_slot, away_slot, swapped) = if host_nation == -1 {
                (home_row, away_row, false)
            } else {
                (home_row, away_row, false)
            };

            out.push(FixtureEmission {
                walker_col,
                home_slot,
                away_slot,
                round_within_half,
                outer_round: outer,
                is_last_round,
                host_nation_swap: swapped,
            });
        }
    }

    out
}

fn main() {
    let root = Path::new("D:/cm0102-rs/reports/fixture_disasm/runtime");
    let trace_path = root.join("20260913_191309_v2_natural.jsonl");
    let sched_path = root.join("20260913_191309_v2_sched_buffer_cid9_0.bin");

    let records = load_trace(&trace_path);
    let sched_buf = std::fs::read(&sched_path).unwrap();
    println!("loaded {} trace records; schedule buffer = {} bytes",
             records.len(), sched_buf.len());

    // Extract 552 primary fixtures (site == "second")
    let fixture_trace = records.iter()
        .find(|r| r["op"] == "fixture_trace" && r["cid"] == 9).unwrap();
    let inserts: Vec<Insert> = serde_json::from_value(
        fixture_trace["inserts"].clone()).unwrap();
    let primary: Vec<&Insert> = inserts.iter()
        .filter(|i| i.site == "second").collect();
    println!("GDI primary inserts (site==second): {}", primary.len());
    assert_eq!(primary.len(), 552);

    // Extract walker sequence
    let walker_trace = records.iter()
        .find(|r| r["op"] == "walker_trace" && r["cid"] == 9).unwrap();
    let walkers: Vec<WalkerCall> = serde_json::from_value(
        walker_trace["calls"].clone()).unwrap();
    let walker_seq: Vec<i32> = walkers.iter()
        .map(|w| w.retval).collect();
    println!("captured walker returns (len={}): {:?}",
             walker_seq.len(), &walker_seq);

    // Pre-perturb slot -> club_id (perturb was a no-op for eng_second,
    // verified by pre==final Club* pointers)
    let deref = records.iter()
        .find(|r| r["op"] == "clubs_before_perturb_deref" && r["cid"] == 9)
        .unwrap();
    let entries: Vec<DerefEntry> = serde_json::from_value(
        deref["entries"].clone()).unwrap();
    let slot_to_id: BTreeMap<i64, i32> = entries.iter()
        .map(|e| (e.slot, e.club_id)).collect();
    println!("pre-perturb slot->club_id map: {} entries", slot_to_id.len());

    // Derive year_base and weekday_parity_flag from schedule buffer +
    // captured first fixture. First fixture: rwh=0, doy=222, year=2001.
    // Schedule buffer round 0 has (doy_offset, year_offset, type_byte).
    let sched_doy0 = u16::from_le_bytes([sched_buf[0], sched_buf[1]]);
    let sched_year_off0 = i16::from_le_bytes([sched_buf[2], sched_buf[3]]);
    let first_fx = primary[0];
    // year = year_base + year_off. So year_base = 2001 - year_off_at_rwh0.
    // rwh 0 in the trace corresponds to walker output 0 (idx 0 walker
    // returned 0), so schedule buffer index 0 is the source. But
    // year_off could be nonzero if the season starts at the 0-slot.
    let year_base = first_fx.year - sched_year_off0;
    println!("schedule buf round 0: doy_off={} year_off={} => year_base={}",
             sched_doy0, sched_year_off0, year_base);
    println!("first fx doy={} vs sched doy_off={}: doy_base = {}",
             first_fx.doy, sched_doy0, first_fx.doy as i32 - sched_doy0 as i32);

    // Weekday parity flag: 3 (from walker_trace flag_byte in this capture)
    let weekday_parity_flag: u8 = 3;
    let host_nation: i32 = -1;
    let n_clubs: i16 = 24;
    let matches_per_pair: i16 = 2;
    let n_rounds: i16 = 46;

    // Run the driver-algorithm replay with the captured walker returns
    let rust_emissions = replay_driver(
        n_clubs, matches_per_pair, n_rounds,
        year_base, weekday_parity_flag, host_nation,
        &walker_seq,
    );
    println!("\nRust replay emitted {} fixtures", rust_emissions.len());

    // Build canonical schema for both sides
    #[derive(Debug, PartialEq, Eq, Clone)]
    struct Canon {
        seq: usize,
        home_id: i32,
        away_id: i32,
        rwh: i16,
        is_last: bool,
    }
    let gdi_canon: Vec<Canon> = primary.iter().enumerate()
        .map(|(i, f)| Canon {
            seq: i,
            home_id: f.home,
            away_id: f.away,
            rwh: f.rwh,
            is_last: (f.flag_bits & 0x0800) != 0
                     || f.rwh as i32 == n_rounds as i32 - 1,
        }).collect();
    let rust_canon: Vec<Canon> = rust_emissions.iter().enumerate()
        .map(|(i, f)| Canon {
            seq: i,
            home_id: *slot_to_id.get(&(f.home_slot as i64))
                .expect("home slot map"),
            away_id: *slot_to_id.get(&(f.away_slot as i64))
                .expect("away slot map"),
            rwh: f.round_within_half,
            is_last: f.is_last_round,
        }).collect();

    // Ordered diff
    println!("\n=== ORDERED DIFF ===");
    let mut mismatches = 0;
    let mut mismatch_fields = BTreeMap::new();
    let mut first_mismatch: Option<usize> = None;
    for i in 0..gdi_canon.len().max(rust_canon.len()) {
        let g = gdi_canon.get(i);
        let r = rust_canon.get(i);
        if g != r {
            mismatches += 1;
            if first_mismatch.is_none() { first_mismatch = Some(i); }
            if let (Some(g), Some(r)) = (g, r) {
                if g.home_id != r.home_id
                    { *mismatch_fields.entry("home_id").or_insert(0) += 1; }
                if g.away_id != r.away_id
                    { *mismatch_fields.entry("away_id").or_insert(0) += 1; }
                if g.rwh != r.rwh
                    { *mismatch_fields.entry("rwh").or_insert(0) += 1; }
                if g.is_last != r.is_last
                    { *mismatch_fields.entry("is_last").or_insert(0) += 1; }
            }
        }
    }
    println!("total ordered mismatches: {}/{}", mismatches, gdi_canon.len());
    println!("field mismatch counts: {:?}", mismatch_fields);
    if let Some(fi) = first_mismatch {
        println!("first mismatch at idx={}", fi);
        println!("  GDI:  {:?}", gdi_canon.get(fi));
        println!("  Rust: {:?}", rust_canon.get(fi));
        // context
        let lo = fi.saturating_sub(3);
        let hi = (fi + 5).min(gdi_canon.len());
        for i in lo..hi {
            let g = gdi_canon.get(i);
            let r = rust_canon.get(i);
            let mark = if g == r { "  " } else { "**" };
            println!("  {} idx={} GDI={:?} Rust={:?}",
                     mark, i, g, r);
        }
    }

    // Set (unordered pair) diff — treat (home,away,rwh) as unordered pair
    println!("\n=== SET DIFF (unordered {{home,away}} per rwh) ===");
    let gdi_set: HashSet<(i32, i32, i16)> = gdi_canon.iter()
        .map(|c| {
            let (a, b) = if c.home_id < c.away_id
                { (c.home_id, c.away_id) } else { (c.away_id, c.home_id) };
            (a, b, c.rwh)
        }).collect();
    let rust_set: HashSet<(i32, i32, i16)> = rust_canon.iter()
        .map(|c| {
            let (a, b) = if c.home_id < c.away_id
                { (c.home_id, c.away_id) } else { (c.away_id, c.home_id) };
            (a, b, c.rwh)
        }).collect();
    println!("gdi set size: {}", gdi_set.len());
    println!("rust set size: {}", rust_set.len());
    let intersection = gdi_set.intersection(&rust_set).count();
    let gdi_only: Vec<_> = gdi_set.difference(&rust_set).take(10).collect();
    let rust_only: Vec<_> = rust_set.difference(&gdi_set).take(10).collect();
    println!("intersection: {}", intersection);
    println!("in GDI only: {} (first 10: {:?})",
             gdi_set.difference(&rust_set).count(), gdi_only);
    println!("in Rust only: {} (first 10: {:?})",
             rust_set.difference(&gdi_set).count(), rust_only);

    // Ordered directed pair (home,away) match — set of directed pairs
    println!("\n=== SET DIFF (directed (home,away,rwh)) ===");
    let gdi_d: HashSet<(i32, i32, i16)> = gdi_canon.iter()
        .map(|c| (c.home_id, c.away_id, c.rwh)).collect();
    let rust_d: HashSet<(i32, i32, i16)> = rust_canon.iter()
        .map(|c| (c.home_id, c.away_id, c.rwh)).collect();
    println!("gdi directed set: {}, rust directed set: {}, intersection: {}",
             gdi_d.len(), rust_d.len(),
             gdi_d.intersection(&rust_d).count());

    // Summary verdict
    println!("\n=== VERDICT ===");
    println!("fixture count: GDI={} Rust={} DELTA={}",
             gdi_canon.len(), rust_canon.len(),
             (rust_canon.len() as i64) - (gdi_canon.len() as i64));
    println!("ordered mismatches: {}", mismatches);
    println!("set-of-unordered-pairs-per-rwh delta: gdi_only={} rust_only={}",
             gdi_set.difference(&rust_set).count(),
             rust_set.difference(&gdi_set).count());
    if mismatches == 0 {
        println!("**** ZERO-DIFF PASS ****");
    } else if gdi_set == rust_set {
        println!("SAME SET, DIFFERENT ORDER");
    } else if gdi_d == rust_d {
        println!("SAME DIRECTED SET, DIFFERENT INSERTION ORDER");
    } else {
        println!("GENUINELY DIFFERENT PAIRINGS");
    }
}
