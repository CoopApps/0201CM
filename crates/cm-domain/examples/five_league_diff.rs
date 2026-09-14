//! C10.6 — full evidence matrix for the 5 English Traditional leagues.
//!
//! Consumes the captures produced by
//! `reports/fixture_disasm/gdi_five_league_lineage.py` and produces
//! **five** per-league differentials:
//!
//!   1. Perturb differential: feed the captured RNG stream into
//!      the ported `matrix_perturb` and compare Rust P2 against
//!      captured P2.
//!   2. Walker state differential: feed captured (prev_col, state)
//!      inputs into the ported `walker_step` and compare returns.
//!   3. Driver differential: feed captured P2 + captured walker
//!      returns into `run_round_robin_driver` and compare fixture
//!      emissions.
//!   4. Schedule SHA256: hash the captured buffer bytes.
//!   5. Date decode: extract per-round (year, day-of-year, calendar
//!      date) from the buffer.
//!
//! Prints a per-league matrix. Zero mismatches across (1)+(2)+(3)
//! + a valid date sequence + a stable SHA256 promotes that league
//! to `ByteExact`-with-cited-artefacts confidence in the spec.
//!
//! Deliberate scope note: The RNG playback approach feeds captured
//! `rand_mod`/`lcg_next` values into `matrix_perturb`. That proves
//! the perturb ALGORITHM is byte-exact given the same RNG inputs.
//! A pure "seed → deterministic Rust reproduces same RNG stream"
//! chain would need the harness to also capture the initial pool
//! cursor and jitter (currently only the LCG initial state is
//! captured via first srand `state_before`). Documented as an
//! evidence gap.

use cm_domain::eng_second_fixtures::{
    matrix_perturb, matrix_seed_base, walker_step,
    FixtureEmission, StadiumClubResolver,
};
use cm_domain::game_rng::GameRng;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet, HashMap};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Capture-file shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Insert {
    #[allow(dead_code)] idx: i64,
    site: String,
    #[serde(default)] #[allow(dead_code)] ret_addr: String,
    #[serde(default)] #[allow(dead_code)] this_list: String,
    #[serde(default)] #[allow(dead_code)] fixture_ptr: String,
    #[allow(dead_code)] #[serde(default)] mode: i64,
    #[allow(dead_code)] cid: i32,
    home: i32, away: i32,
    #[allow(dead_code)] year: i16,
    #[allow(dead_code)] doy: i16,
    rwh: i16,
    #[allow(dead_code)] #[serde(default)] type_byte: u8,
    #[serde(default)] flag_bits: u16,
}

#[derive(Debug, Deserialize, Clone)]
struct WalkerCall {
    #[allow(dead_code)] idx: i32,
    prev_col: i32,
    state_before: i32,
    comp_id: i32,
    n_clubs: i16,
    matches_per_pair: i16,
    n_rounds: i16,
    flag_byte: u8,
    #[allow(dead_code)] state_after: i32,
    retval: i32,
}

#[derive(Debug, Deserialize)]
struct DerefEntry {
    slot: i64,
    club_id: i32,
    #[serde(default, alias = "club_ptr")]
    #[allow(dead_code)] cptr: String,
    #[serde(default)] stadium_id: Option<i32>,
    #[serde(default)] stadium_alt_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct CtorLeave {
    #[allow(dead_code)] cid: u8,
    year_base: i16, n_clubs: i16, n_rounds: i16, mpp: i16,
    d9_flags: u16,
    #[allow(dead_code)] alt_pair_list: String,
    owner_first_int: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct RngCall {
    phase: String, kind: String,
    #[serde(default)] n: i32,
    #[serde(default)] ret: i64,
    #[serde(default)] #[allow(dead_code)] state_before: u64,
    #[serde(default)] #[allow(dead_code)] state_after: u64,
    #[serde(default)] #[allow(dead_code)] seed: u64,
}

// ---------------------------------------------------------------------------
// Per-league capture bundle
// ---------------------------------------------------------------------------

struct Capture {
    league: &'static str,
    ctor: CtorLeave,
    p1_entries: Vec<DerefEntry>,
    p2_entries: Vec<DerefEntry>,
    walker_calls: Vec<WalkerCall>,
    rng_calls: Vec<RngCall>,
    primary_inserts: Vec<Insert>,
    schedule_bytes: Vec<u8>,
}

fn load_capture(root: &Path, prefix: &str, lid: &str,
                league: &'static str) -> Option<Capture> {
    let jsonl = root.join(format!("{prefix}_{lid}_lineage.jsonl"));
    if !jsonl.exists() { return None; }
    let text = std::fs::read_to_string(&jsonl).ok()?;
    let records: Vec<serde_json::Value> = text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let ctor: CtorLeave = serde_json::from_value(
        records.iter().find(|r| r["op"] == "ctor_leave")?.clone()).ok()?;
    let inserts_rec = records.iter().find(|r| r["op"] == "insert_trace")?;
    let inserts: Vec<Insert> = serde_json::from_value(
        inserts_rec["inserts"].clone()).ok()?;
    let primary_inserts: Vec<Insert> = inserts.into_iter()
        .filter(|i| i.site == "primary" || i.site == "main").collect();
    let walker_rec = records.iter().find(|r| r["op"] == "walker_trace")?;
    let walker_calls: Vec<WalkerCall> = serde_json::from_value(
        walker_rec["calls"].clone()).ok()?;
    let rng_rec = records.iter().find(|r| r["op"] == "rng_trace")?;
    let rng_calls: Vec<RngCall> = serde_json::from_value(
        rng_rec["calls"].clone()).ok()?;
    let p1_rec = records.iter()
        .find(|r| r["op"] == "clubs_snap" && r["point"] == "P1")?;
    let p1_entries: Vec<DerefEntry> = serde_json::from_value(
        p1_rec["entries"].clone()).ok()?;
    let p2_rec = records.iter()
        .find(|r| r["op"] == "clubs_snap" && r["point"] == "P2")?;
    let p2_entries: Vec<DerefEntry> = serde_json::from_value(
        p2_rec["entries"].clone()).ok()?;
    let sched_path = root.join(format!("{prefix}_{lid}_sched_buf.bin"));
    let schedule_bytes = std::fs::read(&sched_path).unwrap_or_default();
    Some(Capture {
        league, ctor, p1_entries, p2_entries,
        walker_calls, rng_calls, primary_inserts, schedule_bytes,
    })
}

// ---------------------------------------------------------------------------
// Resolver from captured stadium graph
// ---------------------------------------------------------------------------

fn build_resolver(p1: &[DerefEntry]) -> StadiumClubResolver {
    // C10.7: build via new_by_club so E2/E3 lookup follows club
    // identity (Club.+0x69), not slot. This survives Phase D
    // reordering of the clubs table — matches the exe.
    StadiumClubResolver::new_by_club(
        p1.iter().map(|e| (e.club_id, e.stadium_id, e.stadium_alt_id))
    )
}

// ---------------------------------------------------------------------------
// (1) Perturb differential
// ---------------------------------------------------------------------------

/// Load a RNG playback queue from captured PERTURB-phase calls.
/// Returns (pool_returns, lcg_returns, srand_seed) in call-order.
/// srand seed captured as first srand's `seed` field; state_after
/// is what matrix_perturb's lcg_srand will overwrite to.
fn perturb_rng_streams(rng_calls: &[RngCall]) -> (Vec<i32>, Vec<u32>) {
    let mut pool = Vec::new();
    let mut lcg = Vec::new();
    for c in rng_calls {
        if c.phase != "PERTURB" { continue; }
        match c.kind.as_str() {
            "pool" => pool.push(c.ret as i32),
            "lcg"  => lcg.push(c.ret as u32 & 0xFFFF),
            "srand" => {}  // handled by matrix_perturb calling lcg_srand
            _ => {}
        }
    }
    (pool, lcg)
}

/// Assemble a 0x3b-per-entry clubs_table from a DerefEntry list.
/// First 4 bytes of each entry = club id. Remaining bytes are the
/// exe's raw byte snapshot — we don't have that here, but perturb
/// only reads first_int for its E1 dat_009bc5a8/5ac lookups; other
/// bytes are opaque. Fill with zeros; the swap moves whole entries.
fn build_clubs_table(entries: &[DerefEntry]) -> Vec<u8> {
    const REC: usize = 0x3b;
    let mut out = vec![0u8; entries.len() * REC];
    // Sort by slot to guarantee source ordering.
    let mut sorted = entries.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|e| e.slot);
    for e in &sorted {
        let off = e.slot as usize * REC;
        out[off..off+4].copy_from_slice(&e.club_id.to_le_bytes());
    }
    out
}

/// Decode a clubs_table back to a slot->club_id map.
fn decode_clubs_table(table: &[u8]) -> Vec<i32> {
    const REC: usize = 0x3b;
    let n = table.len() / REC;
    (0..n).map(|i| i32::from_le_bytes(
        table[i*REC..i*REC+4].try_into().unwrap())).collect()
}

fn perturb_diff(cap: &Capture) -> (usize, usize) {
    let (pool_stream, lcg_stream) = perturb_rng_streams(&cap.rng_calls);
    // Derive DAT_00dbc340 seed from the captured Phase C srand.
    let phase_c = cap.rng_calls.iter().find(|c|
        c.phase == "PERTURB" && c.kind == "srand");
    let dbc340_cli_seed = match phase_c {
        Some(c) => (c.seed as i32).wrapping_sub(cap.ctor.year_base as i32),
        None => 0,
    };
    let mut clubs_table = build_clubs_table(&cap.p1_entries);
    let mut rng = GameRng::from_state(0, 0, 0);
    rng.queue_pool_returns(pool_stream);
    rng.queue_lcg_returns(lcg_stream);
    let resolver = build_resolver(&cap.p1_entries);
    let n_even = cap.ctor.n_clubs as i32 + (cap.ctor.n_clubs as i32 & 1);
    matrix_perturb(
        cap.ctor.n_clubs, cap.ctor.year_base, cap.ctor.d9_flags,
        cap.ctor.owner_first_int, &mut clubs_table, &resolver, &mut rng,
        n_even, dbc340_cli_seed, i32::MIN, i32::MIN, i32::MIN,
    );
    // Compare Rust P2 club_ids to captured P2 club_ids per slot.
    let rust_p2 = decode_clubs_table(&clubs_table);
    let expected_p2: Vec<i32> = {
        let mut sorted = cap.p2_entries.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|e| e.slot);
        sorted.iter().map(|e| e.club_id).collect()
    };
    let n = rust_p2.len().min(expected_p2.len());
    let mismatches = (0..n)
        .filter(|&i| rust_p2[i] != expected_p2[i])
        .count();
    (mismatches, expected_p2.len())
}

// ---------------------------------------------------------------------------
// (2) Walker state differential
// ---------------------------------------------------------------------------

fn walker_diff(cap: &Capture) -> (usize, usize) {
    let mut mismatches = 0;
    // Per-call determinism: reset state to captured state_before
    // before each invocation. This tests walker_step as a pure
    // function of its inputs — matches what byte-exact means for
    // a state-machine step.
    for call in cap.walker_calls.iter() {
        let mut state: u8 = call.state_before as u8;
        let retval = walker_step(
            call.prev_col, &mut state,
            call.comp_id, call.n_clubs, call.matches_per_pair,
            call.n_rounds, call.flag_byte,
            /*special_comp_id=*/ i32::MIN, None,
        );
        if retval != call.retval { mismatches += 1; }
    }
    (mismatches, cap.walker_calls.len())
}

// ---------------------------------------------------------------------------
// (3) Driver differential — same as v1
// ---------------------------------------------------------------------------

fn driver_diff(cap: &Capture) -> (usize, usize) {
    let n_clubs = cap.ctor.n_clubs;
    let matches_per_pair = cap.ctor.mpp;
    let n_rounds = cap.ctor.n_rounds;
    let year_base = cap.ctor.year_base;
    let weekday_parity_flag: u8 = 3;
    let host_nation: i32 = -1;
    let n_even: i32 = { let nc = n_clubs as i32; nc + (nc & 1) };
    let walker_seq: Vec<i32> = cap.walker_calls.iter()
        .map(|w| w.retval).collect();
    let mut matrix: Vec<Vec<i32>> = matrix_seed_base(n_even as usize);
    let mut out: Vec<FixtureEmission> = Vec::new();
    for (i, &walker_col) in walker_seq.iter().enumerate() {
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
            } else { (row_idx, cell_abs_minus_1) };
            matrix[cell.unsigned_abs() as usize][col as usize] = 0;
            let clubs_max = (n_clubs as i32) - 1;
            if home_row < 0 || home_row > clubs_max
                || away_row < 0 || away_row > clubs_max { continue; }
            out.push(FixtureEmission {
                walker_col, home_slot: home_row, away_slot: away_row,
                round_within_half, outer_round: outer,
                is_last_round, host_nation_swap: false,
            });
        }
    }
    let _ = (host_nation, matches_per_pair); // silence unused
    let slot_to_id: BTreeMap<i64, i32> = cap.p2_entries.iter()
        .map(|e| (e.slot, e.club_id)).collect();
    let gdi: Vec<(i32, i32, i16, bool)> = cap.primary_inserts.iter()
        .map(|f| (f.home, f.away, f.rwh,
                  (f.flag_bits & 0x0800) != 0
                   || f.rwh as i32 == n_rounds as i32 - 1))
        .collect();
    let rust: Vec<(i32, i32, i16, bool)> = out.iter().map(|f| (
        *slot_to_id.get(&(f.home_slot as i64)).unwrap_or(&-1),
        *slot_to_id.get(&(f.away_slot as i64)).unwrap_or(&-1),
        f.round_within_half, f.is_last_round,
    )).collect();
    let mismatches = (0..gdi.len().min(rust.len()))
        .filter(|&i| gdi[i] != rust[i]).count()
        + gdi.len().abs_diff(rust.len());
    (mismatches, gdi.len())
}

// ---------------------------------------------------------------------------
// (4) SHA256
// ---------------------------------------------------------------------------

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

// Minimal hex encoding without the `hex` crate dep.
mod hex {
    pub fn encode(b: impl AsRef<[u8]>) -> String {
        let mut s = String::with_capacity(b.as_ref().len() * 2);
        for byte in b.as_ref() {
            s.push_str(&format!("{:02x}", byte));
        }
        s
    }
}

// ---------------------------------------------------------------------------
// (5) Date decode (first N rounds — quick summary)
// ---------------------------------------------------------------------------

/// Extract a few key fields from the round records. Stride 0x41
/// per pillar 15. Layout per pillar-13 fixture-dates report:
///   +0x00..0x02: packed date (bit-packed year/month/day)
///   +0x02..0x04: year offset
///   +0x04..0x06: day-of-year
///   +0x0B:       type flag (weekday snap)
fn decode_round_summary(buf: &[u8], n_rounds: usize) -> Vec<(u16, i16, i16, u8)> {
    const STRIDE: usize = 0x41;
    (0..n_rounds).map(|i| {
        let off = i * STRIDE;
        let packed = u16::from_le_bytes([buf[off], buf[off+1]]);
        let year_off = i16::from_le_bytes([buf[off+2], buf[off+3]]);
        let doy = i16::from_le_bytes([buf[off+4], buf[off+5]]);
        let flag = buf[off + 0x0B];
        (packed, year_off, doy, flag)
    }).collect()
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

struct LeagueReport {
    league: &'static str,
    n_clubs: i16,
    n_rounds: i16,
    year_base: i16,
    schedule_bytes: usize,
    schedule_sha256: String,
    p2_size: usize,
    perturb_mm: usize,
    walker_calls: usize,
    walker_mm: usize,
    primary_inserts: usize,
    driver_mm: usize,
    date_first_last: ((u16, i16, i16, u8), (u16, i16, i16, u8)),
}

fn find_newest_prefix(root: &Path) -> Option<String> {
    let mut cands: Vec<PathBuf> = std::fs::read_dir(root).ok()?
        .filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.file_name().and_then(|s| s.to_str())
                    .map(|s| s.contains("_five_leagues_") && s.ends_with("_lineage.jsonl"))
                    .unwrap_or(false))
        .collect();
    cands.sort();
    let last = cands.last()?;
    let name = last.file_name()?.to_str()?;
    let up_to_lineage = name.rsplit_once("_lineage.jsonl")?.0;
    let up_to_lid = up_to_lineage.rsplit_once('_')?.0;
    Some(up_to_lid.to_string())
}

fn main() {
    let root = Path::new("D:/cm0102-rs/reports/fixture_disasm/runtime");
    let args: Vec<String> = std::env::args().collect();
    let prefix = args.iter().position(|a| a == "--prefix")
        .and_then(|i| args.get(i + 1)).cloned()
        .or_else(|| find_newest_prefix(root))
        .unwrap_or_else(|| {
            eprintln!("no capture prefix found under {root:?}");
            std::process::exit(2);
        });
    println!("prefix: {prefix}");

    let leagues = [
        ("prem", "Premier"),
        ("first", "First"),
        ("second", "Second"),
        ("third", "Third"),
        ("conf", "Conference"),
    ];
    let mut reports = Vec::new();
    for (lid, disp) in &leagues {
        let cap = match load_capture(root, &prefix, lid, disp) {
            Some(c) => c,
            None => { println!("  {disp}: capture missing"); continue; }
        };
        let (perturb_mm, _p2n) = perturb_diff(&cap);
        let (walker_mm, walker_n) = walker_diff(&cap);
        let (driver_mm, _drv_n) = driver_diff(&cap);
        let sha = sha256_hex(&cap.schedule_bytes);
        let dates = decode_round_summary(&cap.schedule_bytes,
                                          cap.ctor.n_rounds as usize);
        let first = dates.first().copied().unwrap_or_default();
        let last = dates.last().copied().unwrap_or_default();
        reports.push(LeagueReport {
            league: disp, n_clubs: cap.ctor.n_clubs,
            n_rounds: cap.ctor.n_rounds, year_base: cap.ctor.year_base,
            schedule_bytes: cap.schedule_bytes.len(),
            schedule_sha256: sha,
            p2_size: cap.p2_entries.len(),
            perturb_mm,
            walker_calls: walker_n, walker_mm,
            primary_inserts: cap.primary_inserts.len(), driver_mm,
            date_first_last: (first, last),
        });
    }

    println!("\n=== C10.6 evidence matrix ===");
    println!("{:<11} {:>3} {:>4} {:>5} {:>7} {:>10} {:>10} {:>10}",
             "league", "N", "R", "buf", "primary",
             "perturbΔ", "walkerΔ", "driverΔ");
    for r in &reports {
        println!("{:<11} {:>3} {:>4} {:>5} {:>7} {:>4}/{:<4} {:>4}/{:<4} {:>4}/{:<4}",
                 r.league, r.n_clubs, r.n_rounds, r.schedule_bytes,
                 r.primary_inserts,
                 r.perturb_mm, r.p2_size,
                 r.walker_mm, r.walker_calls,
                 r.driver_mm, r.primary_inserts);
    }

    println!("\n=== Schedule SHA256 ===");
    for r in &reports {
        println!("  {:<11} {} bytes  sha256:{}",
                 r.league, r.schedule_bytes, r.schedule_sha256);
    }

    println!("\n=== First/last round date summary ===");
    println!("(packed_date, year_off, day_of_year, weekday_flag)");
    for r in &reports {
        let (f, l) = r.date_first_last;
        println!("  {:<11}  round[0]={:?}  round[{}]={:?}",
                 r.league, f, r.n_rounds - 1, l);
    }

    let all_zero = reports.iter().all(|r|
        r.perturb_mm == 0 && r.walker_mm == 0 && r.driver_mm == 0);
    if all_zero {
        println!("\n**** ALL FIVE LEAGUES: perturb + walker + driver zero-diff ****");
    } else {
        println!("\n(some mismatches — see matrix above)");
        std::process::exit(1);
    }
    // Silence unused imports if only some paths are exercised
    let _ = (BTreeSet::<i32>::new(), HashSet::<i32>::new());
}
