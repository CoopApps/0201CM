//! Validate the Rust round-robin driver INDEPENDENTLY of perturb.
//!
//! Approach: feed the driver GDI's captured POST-perturb clubs (P2)
//! as the input clubs table, replay GDI's captured walker sequence,
//! skip Rust perturb entirely. Then diff fixture emissions against
//! GDI's captured 552 primary fixtures.
//!
//! If diff == 0: driver is byte-exact; only perturb blocks production.
//! If diff > 0: driver has bugs too.
//!
//! Trace: reports/fixture_disasm/runtime/20260913_205743_lineage.jsonl

use cm_domain::eng_second_fixtures::{
    matrix_seed_base, FixtureEmission,
};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Insert {
    idx: i64, site: String,
    #[serde(default)] ret_addr: String,
    #[serde(default)] this_list: String,
    #[serde(default)] fixture_ptr: String,
    #[allow(dead_code)] #[serde(default)] mode: i64,
    cid: i32, home: i32, away: i32,
    year: i16, doy: i16, rwh: i16,
    #[serde(default)] type_byte: u8,
    #[serde(default)] flag_bits: u16,
}

#[derive(Debug, Deserialize)]
struct WalkerCall {
    idx: i32,
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
    slot: i64, club_id: i32,
    #[allow(dead_code)] cptr: String,
}

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
    let n_even: i32 = { let nc = n_clubs as i32; nc + (nc & 1) };
    let mut matrix: Vec<Vec<i32>> = matrix_seed_base(n_even as usize);
    let total_rounds: i32 = (matches_per_pair as i32) * (n_even - 1);
    assert_eq!(total_rounds as usize, walker_returns.len(),
               "walker sequence length mismatch");
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
            } else {
                (home_row, away_row, false)
            };
            out.push(FixtureEmission {
                walker_col,
                home_slot, away_slot,
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
    let trace_path = root.join("20260913_205743_lineage.jsonl");
    let text = std::fs::read_to_string(&trace_path).unwrap();
    let records: Vec<serde_json::Value> = text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    println!("loaded {} trace records", records.len());

    // Extract inputs
    let inserts_rec = records.iter()
        .find(|r| r["op"] == "insert_trace").unwrap();
    let inserts: Vec<Insert> = serde_json::from_value(
        inserts_rec["inserts"].clone()).unwrap();
    let primary: Vec<&Insert> = inserts.iter()
        .filter(|i| i.site == "main").collect();
    println!("GDI primary (site==main) inserts: {}", primary.len());
    assert_eq!(primary.len(), 552);

    let walker_rec = records.iter()
        .find(|r| r["op"] == "walker_trace").unwrap();
    let walkers: Vec<WalkerCall> = serde_json::from_value(
        walker_rec["calls"].clone()).unwrap();
    let walker_seq: Vec<i32> = walkers.iter().map(|w| w.retval).collect();
    println!("captured walker: {} calls", walker_seq.len());

    // P2 = post-perturb slot -> club_id (what the driver actually sees)
    let p2_rec = records.iter()
        .find(|r| r["op"] == "clubs_snap" && r["point"] == "P2").unwrap();
    let p2_entries: Vec<DerefEntry> = serde_json::from_value(
        p2_rec["entries"].clone()).unwrap();
    let slot_to_id: BTreeMap<i64, i32> = p2_entries.iter()
        .map(|e| (e.slot, e.club_id)).collect();
    println!("P2 slot->club_id map: {} entries", slot_to_id.len());
    let p2_ids: Vec<i32> = (0..24).map(|i| *slot_to_id.get(&i).unwrap()).collect();
    println!("P2 ids: {:?}", p2_ids);

    // Driver constants
    let n_clubs: i16 = 24;
    let matches_per_pair: i16 = 2;
    let n_rounds: i16 = 46;
    let year_base: i16 = 2001;
    let weekday_parity_flag: u8 = 3;  // captured comp+0xd9 low byte
    let host_nation: i32 = -1;

    let rust_emissions = replay_driver(
        n_clubs, matches_per_pair, n_rounds,
        year_base, weekday_parity_flag, host_nation,
        &walker_seq,
    );
    println!("\nRust replay emitted {} fixtures\n", rust_emissions.len());

    // Canonicalize
    #[derive(Debug, PartialEq, Eq, Clone)]
    struct C {
        seq: usize, home_id: i32, away_id: i32, rwh: i16, is_last: bool,
    }
    let gdi: Vec<C> = primary.iter().enumerate()
        .map(|(i, f)| C {
            seq: i, home_id: f.home, away_id: f.away, rwh: f.rwh,
            is_last: (f.flag_bits & 0x0800) != 0
                     || f.rwh as i32 == n_rounds as i32 - 1,
        }).collect();
    let rust: Vec<C> = rust_emissions.iter().enumerate()
        .map(|(i, f)| C {
            seq: i,
            home_id: *slot_to_id.get(&(f.home_slot as i64)).unwrap(),
            away_id: *slot_to_id.get(&(f.away_slot as i64)).unwrap(),
            rwh: f.round_within_half,
            is_last: f.is_last_round,
        }).collect();

    // Ordered diff
    println!("=== ORDERED DIFF ===");
    let mut mismatches = 0;
    let mut first_mm: Option<usize> = None;
    let mut fields = BTreeMap::new();
    for i in 0..gdi.len() {
        if gdi[i] != rust[i] {
            mismatches += 1;
            if first_mm.is_none() { first_mm = Some(i); }
            if gdi[i].home_id != rust[i].home_id
                { *fields.entry("home_id").or_insert(0) += 1; }
            if gdi[i].away_id != rust[i].away_id
                { *fields.entry("away_id").or_insert(0) += 1; }
            if gdi[i].rwh != rust[i].rwh
                { *fields.entry("rwh").or_insert(0) += 1; }
            if gdi[i].is_last != rust[i].is_last
                { *fields.entry("is_last").or_insert(0) += 1; }
        }
    }
    println!("ordered mismatches: {}/{}", mismatches, gdi.len());
    println!("field breakdown: {:?}", fields);
    if let Some(fi) = first_mm {
        println!("first mismatch @ idx={}", fi);
        println!("  GDI:  {:?}", gdi[fi]);
        println!("  Rust: {:?}", rust[fi]);
        let lo = fi.saturating_sub(2);
        let hi = (fi + 5).min(gdi.len());
        for i in lo..hi {
            let mark = if gdi[i] == rust[i] { "  " } else { "**" };
            println!("  {} idx={} GDI={:?}", mark, i, gdi[i]);
            println!("     rust={:?}", rust[i]);
        }
    }

    // Set diffs
    println!("\n=== SET DIFF (unordered per rwh) ===");
    let gs: HashSet<(i32,i32,i16)> = gdi.iter().map(|c| {
        let (a,b) = if c.home_id < c.away_id
            { (c.home_id, c.away_id) } else { (c.away_id, c.home_id) };
        (a, b, c.rwh)
    }).collect();
    let rs: HashSet<(i32,i32,i16)> = rust.iter().map(|c| {
        let (a,b) = if c.home_id < c.away_id
            { (c.home_id, c.away_id) } else { (c.away_id, c.home_id) };
        (a, b, c.rwh)
    }).collect();
    println!("gdi set size: {}, rust set size: {}, intersection: {}",
             gs.len(), rs.len(), gs.intersection(&rs).count());

    println!("\n=== SET DIFF (directed (home,away,rwh)) ===");
    let gd: HashSet<(i32,i32,i16)> = gdi.iter().map(|c| (c.home_id, c.away_id, c.rwh)).collect();
    let rd: HashSet<(i32,i32,i16)> = rust.iter().map(|c| (c.home_id, c.away_id, c.rwh)).collect();
    println!("gdi: {}, rust: {}, intersection: {}",
             gd.len(), rd.len(), gd.intersection(&rd).count());

    println!("\n=== VERDICT ===");
    if mismatches == 0 {
        println!("**** ZERO-DIFF PASS ****");
        println!("Driver is byte-exact when fed post-perturb clubs.");
        println!("Only perturb blocks production wiring.");
    } else if gs == rs {
        println!("SAME UNORDERED SET, DIFFERENT ORDER/H-A");
    } else if gd == rd {
        println!("SAME DIRECTED SET, DIFFERENT INSERTION ORDER");
    } else {
        println!("GENUINELY DIFFERENT PAIRINGS — driver also has bugs");
    }
}
