//! P1→P2 golden test for `matrix_perturb`.
//!
//! Loads the captured `20260913_205743_lineage.jsonl` and runs Rust
//! matrix_perturb (via a hand-rolled variant that consumes the exact
//! captured LCG/pool sequence) against P1. Diffs slot-by-slot vs P2.
//!
//! Success criterion: 24/24 slots match. Reports mismatch count and
//! per-slot lineage after every substantive change to the algorithm.

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct DerefEntry {
    slot: i64,
    club_id: i32,
    // NEW field names from the tagged-this Phase 12 capture
    // (perturb-actual: Club.+0x69 is stadium ptr not nation)
    #[serde(default)] stadium_ptr: Option<String>,
    #[serde(default)] stadium_alt_ptr: Option<String>,
    // Legacy names for the mislabelled earlier captures
    #[serde(default, alias = "stadium_ptr")]
    nation_ptr: Option<String>,
    #[serde(default, alias = "stadium_alt_ptr")]
    nation_plus_48: Option<String>,
}

impl DerefEntry {
    fn stadium_str(&self) -> String {
        self.stadium_ptr.clone()
            .or_else(|| self.nation_ptr.clone())
            .unwrap_or_else(|| "0x0".into())
    }
    fn stadium_alt_str(&self) -> String {
        self.stadium_alt_ptr.clone()
            .or_else(|| self.nation_plus_48.clone())
            .unwrap_or_else(|| "0x0".into())
    }
}

#[derive(Debug, Deserialize)]
struct RngCall {
    phase: String,
    kind: String,
    #[serde(default)] n: i32,
    #[serde(default)] ret: i64,
    #[serde(default)] state_before: u64,
    #[serde(default)] state_after: u64,
    #[serde(default)] seed: u64,
}

/// Injected-stream RNG mock. Answers rand_mod/lcg_next/lcg_srand from
/// the pre-recorded capture in the order they were made, no arithmetic.
/// Panics on drift so we notice if the algorithm changes its RNG order.
struct StreamRng {
    calls: Vec<RngCall>,
    idx: usize,
}
impl StreamRng {
    fn new(calls: Vec<RngCall>) -> Self { Self { calls, idx: 0 } }
    fn take(&mut self, want_kind: &str) -> &RngCall {
        loop {
            if self.idx >= self.calls.len() {
                panic!("ran out of captured RNG calls, wanted {}", want_kind);
            }
            let c = &self.calls[self.idx];
            self.idx += 1;
            // Only consume calls tagged PERTURB — the trace also holds
            // DRIVER-phase pool calls (walker rand_mod(4)) after perturb
            // completes; ignore them here.
            if c.phase != "PERTURB" { continue; }
            assert_eq!(c.kind, want_kind,
                "RNG order divergence: expected {}, got {} (idx={})",
                want_kind, c.kind, self.idx - 1);
            return c;
        }
    }
    fn rand_mod(&mut self, n: i32) -> i32 {
        let c = self.take("pool");
        assert_eq!(c.n, n, "pool arg mismatch");
        c.ret as i32
    }
    fn lcg_next(&mut self) -> u32 { self.take("lcg").ret as u32 }
    /// srand: We don't care what the caller passes — we validate the
    /// swap algorithm by replaying the captured LCG return stream.
    /// The captured seed is `c.seed` and could feed a byte-exact
    /// reproduction if we knew DAT_00dbc340.
    fn lcg_srand(&mut self, _seed: u32) -> u32 {
        let c = self.take("srand");
        c.seed as u32   // return captured seed for logging
    }
    fn remaining_perturb_calls(&self) -> usize {
        self.calls[self.idx..].iter().filter(|c| c.phase == "PERTURB").count()
    }
}

/// Manually-inline reimplementation of matrix_perturb, phase-by-phase,
/// with per-slot nation + nation.+0x48 data feeding E2/E3.
///
/// `nation_of(club_id)` and `plus48_of(club_id)` are stable per-club
/// lookups; the algorithm indexes them by the CURRENT clubs[i] value
/// at each E-phase step (which changes as Phase D shuffles clubs).
fn perturb_replay_v2(
    n_clubs: i16,
    year: i16,
    d9_flags: u16,
    owner_first: Option<i32>,
    nation_of: &dyn Fn(i32) -> String,
    plus48_of: &dyn Fn(i32) -> String,
    clubs: &mut Vec<i32>,
    rng: &mut StreamRng,
    dbc340_cli_seed: i32,
    dat_009bba9c: i32,
    dat_009bc5a8: i32,
    dat_009bc5ac: i32,
) -> Vec<Vec<i32>> {
    let n = n_clubs as usize;
    let n_even = n_clubs as i32 + (n_clubs as i32 & 1);
    let mut lineage = Vec::new();
    lineage.push(clubs.clone());   // P1

    // Phase B — pool rand_mod(30000)
    let saved_pool = rng.rand_mod(30000);

    // Phase C — LCG srand(year + dbc340). Seed value comes from
    // captured trace; we log what GDI used so we can back-derive
    // DAT_00dbc340 for this run.
    let seed_c_want = (year as i32).wrapping_add(dbc340_cli_seed) as u32;
    let seed_c_gdi = rng.lcg_srand(seed_c_want);
    let derived_dat = (seed_c_gdi as i32).wrapping_sub(year as i32);
    println!("Phase C seed: rust_wants={} gdi_used={} => DAT_00dbc340 derived={}",
             seed_c_want, seed_c_gdi, derived_dat);

    // Phase D — n_clubs biased-Fisher-Yates swaps
    for _ in 0..n {
        let r1 = rng.lcg_next() as i32;
        let idx1 = ((r1 as i64 * n_clubs as i64) / 0x8000) as usize;
        let r2 = rng.lcg_next() as i32;
        let idx2 = ((r2 as i64 * n_clubs as i64) / 0x8000) as usize;
        // Match C code — swap even if idx1 == idx2 (no-op)
        // and no bounds check (guaranteed by alldiv math)
        if idx1 != idx2 {
            clubs.swap(idx1, idx2);
        }
    }
    lineage.push(clubs.clone());   // after Phase D

    // Phase F — LCG srand(pool_rand + 1)
    let _ = rng.lcg_srand((saved_pool + 1) as u32);

    if (d9_flags & 0x100) == 0 {
        // Branch A — Rust's E1..E4 build a scratch buffer, copy back.
        let half = n / 2;
        let mut scratch: Vec<Option<i32>> = vec![None; n];
        let mut used_src = vec![false; n];
        let mut used_dst = vec![false; n];
        let mut local_254: usize = 0;

        // E1 — DAT_009bba9c gate
        if owner_first == Some(dat_009bba9c) {
            let mut idx_a: Option<usize> = None;
            let mut idx_b: Option<usize> = None;
            for i in 0..n {
                if clubs[i] == dat_009bc5a8 { idx_a = Some(i); }
                if clubs[i] == dat_009bc5ac { idx_b = Some(i); }
            }
            if let (Some(a), Some(b)) = (idx_a, idx_b) {
                let flip = rng.rand_mod(2);
                let (pin0_src, pin_n_src) = if flip == 0 { (b, a) } else { (a, b) };
                scratch[0] = Some(clubs[pin0_src]);
                scratch[half] = Some(clubs[pin_n_src]);
                used_dst[0] = true;
                used_dst[half] = true;
                used_src[pin0_src] = true;
                used_src[pin_n_src] = true;
                local_254 = 1;
            }
        }

        // E2 — pair by shared nation pointer (both non-null, equal).
        // Outer i from 0..n-1, inner j from i+1..n-1.
        for i in 0..n.saturating_sub(1) {
            if used_src[i] { continue; }
            if nation_of(clubs[i]) == "0x0" { continue; }
            for j in (i + 1)..n {
                if used_src[j] { continue; }
                if nation_of(clubs[j]) == "0x0" { continue; }
                if nation_of(clubs[i]) != nation_of(clubs[j]) { continue; }
                if local_254 >= half { break; }
                let slot_lo = local_254;
                let slot_hi = half + local_254;
                scratch[slot_lo] = Some(clubs[i]);
                scratch[slot_hi] = Some(clubs[j]);
                used_dst[slot_lo] = true;
                used_dst[slot_hi] = true;
                used_src[i] = true;
                used_src[j] = true;
                local_254 += 1;
                break;
            }
        }
        lineage.push(scratch.iter().map(|x| x.unwrap_or(-1)).collect());   // after E1+E2

        // E3 — pair by nation.+0x48 cross-link (bidirectional).
        // Same outer i / inner j iteration as E2.
        for i in 0..n.saturating_sub(1) {
            if used_src[i] { continue; }
            if nation_of(clubs[i]) == "0x0" { continue; }
            for j in (i + 1)..n {
                if used_src[j] { continue; }
                if nation_of(clubs[j]) == "0x0" { continue; }
                let match_ij = plus48_of(clubs[i]) == nation_of(clubs[j]);
                let match_ji = plus48_of(clubs[j]) == nation_of(clubs[i]);
                if !match_ij && !match_ji { continue; }
                if local_254 >= half { break; }
                let slot_lo = local_254;
                let slot_hi = half + local_254;
                scratch[slot_lo] = Some(clubs[i]);
                scratch[slot_hi] = Some(clubs[j]);
                used_dst[slot_lo] = true;
                used_dst[slot_hi] = true;
                used_src[i] = true;
                used_src[j] = true;
                local_254 += 1;
                break;
            }
        }

        // E4 — flush unconsumed sources
        for i in 0..n {
            if used_src[i] { continue; }
            for j in 0..n {
                if !used_dst[j] {
                    scratch[j] = Some(clubs[i]);
                    used_dst[j] = true;
                    used_src[i] = true;
                    break;
                }
            }
        }
        lineage.push(scratch.iter().map(|x| x.unwrap_or(-1)).collect());   // after E4

        // Phase G — copy scratch back
        for i in 0..n {
            clubs[i] = scratch[i].expect("all scratch slots must be filled");
        }
    } else {
        unimplemented!("branch B");
    }

    let _ = n_even;   // silence
    lineage
}

fn main() {
    // TRUE English Second Division 2001-02 tagged-this lineage (Phase 12).
    let path = Path::new(
        "D:/cm0102-rs/reports/fixture_disasm/runtime/20260914_004421_eng2_true_lineage.jsonl");
    let text = std::fs::read_to_string(path).unwrap();
    let records: Vec<serde_json::Value> = text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    let p1_rec = records.iter()
        .find(|r| r["op"] == "clubs_snap" && r["point"] == "P1").unwrap();
    let p2_rec = records.iter()
        .find(|r| r["op"] == "clubs_snap" && r["point"] == "P2").unwrap();
    let p1_entries: Vec<DerefEntry> = serde_json::from_value(
        p1_rec["entries"].clone()).unwrap();
    let p2_entries: Vec<DerefEntry> = serde_json::from_value(
        p2_rec["entries"].clone()).unwrap();
    let p1: Vec<i32> = (0..24)
        .map(|i| p1_entries.iter().find(|e| e.slot == i).unwrap().club_id).collect();
    let p2: Vec<i32> = (0..24)
        .map(|i| p2_entries.iter().find(|e| e.slot == i).unwrap().club_id).collect();

    // Build club_id -> (nation, plus48) map from P1 (clubs are identical
    // across P1 and P2 — only their slot order changes)
    let mut club_nation = std::collections::HashMap::new();
    let mut club_plus48 = std::collections::HashMap::new();
    for e in &p1_entries {
        club_nation.insert(e.club_id, e.stadium_str());
        club_plus48.insert(e.club_id, e.stadium_alt_str());
    }

    let rng_rec = records.iter().find(|r| r["op"] == "rng_trace").unwrap();
    let rng_calls: Vec<RngCall> = serde_json::from_value(
        rng_rec["calls"].clone()).unwrap();
    let mut rng = StreamRng::new(rng_calls);

    // Captured per-run inputs (from driver_enter record):
    // year (comp+0x40) = 2001
    // d9_flags (comp+0xd9) = 0x3
    // owner_first_int = 8
    // For unknown DAT_009b* constants use MIN sentinels so E1 doesn't fire.
    // Build per-slot nation/plus48 arrays keyed by post-D club order.
    // Since Phase D shuffles in-place, we must rebuild these arrays
    // dynamically after each phase. But E2/E3 read them indexed by the
    // CURRENT clubs[i], so we look up by club_id at each i.
    let nation_lookup: Box<dyn Fn(i32) -> String> = {
        let m = club_nation.clone();
        Box::new(move |cid| m.get(&cid).cloned().unwrap_or_else(|| "0x0".into()))
    };
    let plus48_lookup: Box<dyn Fn(i32) -> String> = {
        let m = club_plus48.clone();
        Box::new(move |cid| m.get(&cid).cloned().unwrap_or_else(|| "0x0".into()))
    };

    let mut clubs = p1.clone();
    // Build initial nation/plus48 arrays; perturb_replay will use these
    // indexed by CURRENT slot which is what post-D reads.
    let nations_by_slot: Vec<String> = (0..24)
        .map(|i| nation_lookup(p1[i])).collect();
    let plus48_by_slot: Vec<String> = (0..24)
        .map(|i| plus48_lookup(p1[i])).collect();

    // Phase D shuffles clubs. E2/E3 then read nations of shuffled clubs.
    // Since club_id -> nation is a stable map, we pass the LOOKUPS and
    // perturb_replay indexes by clubs[i] at each E-phase step.
    let lineage = perturb_replay_v2(
        24, 2001, 0x3, Some(8),
        &nation_lookup, &plus48_lookup,
        &mut clubs, &mut rng,
        0,
        i32::MIN, i32::MIN, i32::MIN,
    );
    let _ = (nations_by_slot, plus48_by_slot);

    println!("P1:            {:?}", p1);
    println!("After Phase D: {:?}", lineage[1]);
    println!("After E1+E2:   {:?}", lineage[2]);
    println!("After E4 :     {:?}", lineage[3]);
    println!("P2 expected:   {:?}", p2);

    let mismatches: Vec<usize> = (0..24).filter(|&i| clubs[i] != p2[i]).collect();
    println!("\n=== Slot lineage table ===");
    println!("{:<5} {:<10} {:<10} {:<10} {:<10} {:<10} {}",
             "slot", "P1", "postD", "postE12", "postE4", "P2exp", "match");
    for i in 0..24 {
        let m = if clubs[i] == p2[i] { "OK" } else { "**" };
        println!("{:<5} {:<10} {:<10} {:<10} {:<10} {:<10} {}",
                 i, p1[i], lineage[1][i], lineage[2][i],
                 lineage[3][i], p2[i], m);
    }
    println!("\nP1→P2 mismatches: {}/24", mismatches.len());
    println!("mismatch slots: {:?}", mismatches);
    println!("RNG stream remaining PERTURB calls: {}",
             rng.remaining_perturb_calls());

    if mismatches.is_empty() {
        println!("**** 24/24 PASS ****");
    }
}
