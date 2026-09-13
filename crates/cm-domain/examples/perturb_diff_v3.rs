//! Test whether Rust `matrix_perturb` reproduces GDI's captured
//! P1→P2 reorder for the 24-club run at
//! `runtime/20260913_203314_clubs_lifecycle.jsonl`.
//!
//! Input: P1 slot->club_id map (24 entries).
//! Expected output: P2 slot->club_id map (24 entries).
//!
//! Method: build a synthetic clubs_table with the club_id in the first
//! 4 bytes of each 0x3b entry (Rust perturb doesn't dereference the
//! Club* pointer — it swaps whole entries). Then run `matrix_perturb`
//! with year=2001 (assumed default) and inspect the first 4 bytes of
//! each entry after.

use cm_domain::eng_second_fixtures::{
    matrix_perturb, ClubResolver, NullResolver, PerturbConstants,
};
use cm_domain::game_rng::GameRng;

/// Resolver that says every slot shares the same nation.
struct AllSameNation(i32);
impl ClubResolver for AllSameNation {
    fn nation_of(&self, _slot: usize) -> Option<i32> { Some(self.0) }
}

fn main() {
    // Captured P1 slot->club_id
    let p1: Vec<i32> = vec![
        587, 689, 767, 848, 1165, 1173, 1185, 1825, 1905, 2784,
        2922, 3179, 3182, 3555, 3574, 3823, 4099, 4100, 4350, 4907,
        4915, 4930, 4980, 4999,
    ];
    // Captured P2 slot->club_id (perturb output)
    let p2_expected: Vec<i32> = vec![
        1185, 587, 2784, 4999, 3555, 4099, 1173, 1825, 4930, 689,
        3574, 4915, 4980, 4100, 4350, 4907, 3179, 3182, 1905, 3823,
        1165, 2922, 767, 848,
    ];
    assert_eq!(p1.len(), 24);
    assert_eq!(p2_expected.len(), 24);
    assert_eq!(p1.iter().copied().collect::<std::collections::HashSet<_>>(),
               p2_expected.iter().copied().collect::<std::collections::HashSet<_>>(),
               "P1 and P2 must be the same set of clubs (just reordered)");

    // Build synthetic 24 * 0x3b clubs table with club_id in first 4 bytes.
    const REC: usize = 0x3b;
    let n_clubs = p1.len() as i16;
    let mut clubs_table = vec![0u8; p1.len() * REC];
    for (i, &cid) in p1.iter().enumerate() {
        clubs_table[i * REC..i * REC + 4].copy_from_slice(&cid.to_le_bytes());
        // Fill remaining bytes with a unique sentinel so we can trace them
        for b in 4..REC {
            clubs_table[i * REC + b] = ((i as u8).wrapping_mul(0x11)).wrapping_add(b as u8);
        }
    }

    // Rust matrix_perturb inputs
    let year: i16 = 2001;
    let d9_flags: u16 = 3;         // matches captured walker_flag_byte
    let owner_comp_first_int: Option<i32> = None;   // gate for E1 - unknown, use None
    let resolver = NullResolver;                     // no nation info, E2/E3 no-op

    // We don't know the initial GameRng state at perturb-entry in the
    // natural run. Phase B's rand_mod(30000) just picks a "saved_pool"
    // value used by Phase F (which srand's the LCG before E-phases).
    // Phase D's shuffle depends ONLY on year (via srand(year) at Phase
    // C). So Phase D output is deterministic given P1 and year.
    //
    // Try a few seeds to see if any Rust output matches P2.
    let seeds_to_try: Vec<u32> = vec![0, 1, 2001, 12345, 14542];

    let n_even = n_clubs as i32 + (n_clubs as i32 & 1);
    let consts = PerturbConstants::default();

    // Also try a config that would match GDI: dbc340_cli_seed = 0 (stock)
    println!("Rust `matrix_perturb` reproduction test");
    println!("=========================================");
    println!("P1 input:  {:?}", p1);
    println!("P2 expect: {:?}", p2_expected);
    println!();

    for seed in &seeds_to_try {
        let mut ct = clubs_table.clone();
        let mut rng = GameRng::new(*seed);
        matrix_perturb(
            n_clubs, year, d9_flags, owner_comp_first_int,
            &mut ct, &resolver, &mut rng,
            n_even, consts.dbc340_cli_seed, consts.dat_009bba9c,
            consts.dat_009bc5a8, consts.dat_009bc5ac,
        );
        let out: Vec<i32> = (0..24)
            .map(|i| i32::from_le_bytes(ct[i*REC..i*REC+4].try_into().unwrap()))
            .collect();
        let matches = out == p2_expected;
        let match_count = out.iter().zip(&p2_expected).filter(|(a,b)| a==b).count();
        println!("seed={:5}: {}  matches_expected={}/{}",
                 seed, if matches { "***MATCH***" } else { "differ" },
                 match_count, 24);
        if !matches {
            println!("           Rust out: {:?}", out);
        }
    }

    // Now try WITHOUT E1-E4 phases (Phase D alone) by using a resolver
    // that ensures no fixups trigger. NullResolver already does this
    // for E2 (no nation info). But E1 fires when owner_comp_first_int
    // matches specific DAT constants — we pass None, so E1 no-op.

    // d9_flags = 0x100 (branch B) — skipped because unimplemented for eng.

    // ==== Test with AllSameNation resolver (English clubs share nation) ====
    println!();
    println!("--- With AllSameNation(100) resolver, various seeds ---");
    let same = AllSameNation(100);
    for seed in &[0u32, 1, 2001, 12345, 14542, 6181] {
        let mut ct = clubs_table.clone();
        let mut rng = GameRng::new(*seed);
        matrix_perturb(
            n_clubs, year, d9_flags, owner_comp_first_int,
            &mut ct, &same, &mut rng,
            n_even, consts.dbc340_cli_seed, consts.dat_009bba9c,
            consts.dat_009bc5a8, consts.dat_009bc5ac,
        );
        let out: Vec<i32> = (0..24)
            .map(|i| i32::from_le_bytes(ct[i*REC..i*REC+4].try_into().unwrap()))
            .collect();
        let match_count = out.iter().zip(&p2_expected).filter(|(a,b)| a==b).count();
        let matches = out == p2_expected;
        println!("seed={:5}: {}  matches={}/24",
                 seed, if matches { "***MATCH***" } else { "differ" },
                 match_count);
        if match_count >= 12 {
            println!("           out: {:?}", out);
        }
    }

    // Additional check: how many positions match if we just apply
    // Phase D's biased-Fisher-Yates in isolation (no E1-E4)?
    // The key question is whether Phase D alone reproduces P2.
    println!();
    println!("--- Isolated Phase D reproduction (custom loop) ---");
    for year_try in &[2001i32, 2002, 2000, 1, 0] {
        let mut ct = clubs_table.clone();
        let mut rng = GameRng::new(0);
        // Phase B: consume pool RNG once (matches GDI's saved_pool draw)
        let _ = rng.rand_mod(30000);
        // Phase C: LCG srand(year)
        rng.lcg_srand(*year_try as u32);
        // Phase D: n_clubs iterations of biased-Fisher-Yates
        for _ in 0..24 {
            let r1 = rng.lcg_next() as i32;
            let idx1 = ((r1 as i64 * 24) / 0x8000) as usize;
            let r2 = rng.lcg_next() as i32;
            let idx2 = ((r2 as i64 * 24) / 0x8000) as usize;
            if idx1 != idx2 && idx1 < 24 && idx2 < 24 {
                let (lo, hi) = if idx1 < idx2 { (idx1, idx2) } else { (idx2, idx1) };
                let (a, b) = ct.split_at_mut(hi * REC);
                a[lo * REC..lo * REC + REC].swap_with_slice(&mut b[0..REC]);
            }
        }
        let out: Vec<i32> = (0..24)
            .map(|i| i32::from_le_bytes(ct[i*REC..i*REC+4].try_into().unwrap()))
            .collect();
        let matches = out == p2_expected;
        let match_count = out.iter().zip(&p2_expected).filter(|(a,b)| a==b).count();
        println!("year={:4}: {}  matches_expected={}/{}",
                 year_try, if matches { "***MATCH***" } else { "differ" },
                 match_count, 24);
        if match_count >= 20 {
            println!("           Rust out: {:?}", out);
        }
    }
}
