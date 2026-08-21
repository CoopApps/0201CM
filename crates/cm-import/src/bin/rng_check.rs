//! Verify the game RNG is bit-exact and reproducible with the real table.

use cm_rng::{CrtRand, MatchRng};
use std::path::Path;

fn main() {
    // 1. CRT LCG known-answer (MSVC srand(1)).
    let mut c = CrtRand::new(1);
    let seq: Vec<u16> = (0..5).map(|_| c.next()).collect();
    println!("CRT rand() srand(1): {seq:?}  (expect [41, 18467, 6334, 26500, 19169])");
    assert_eq!(seq, [41, 18467, 6334, 26500, 19169]);

    // 2. Load the real 51,000-entry table extracted from cm0102.exe.
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let table = cm_db::ConfigData::read_rng_table(Path::new(&dir)).expect("read rng table");
    println!("\nRNG table: {} entries", table.len());
    assert_eq!(table.len(), 51_000, "expected the full 51,000-entry table");

    // 3. Seeded init (faithful port of FUN_008fc5d0) is reproducible.
    let draws_a: Vec<i32> = {
        let mut r = MatchRng::new_seeded(table.clone(), 12345);
        (0..10).map(|_| r.random(200)).collect()
    };
    let draws_b: Vec<i32> = {
        let mut r = MatchRng::new_seeded(table.clone(), 12345);
        (0..10).map(|_| r.random(200)).collect()
    };
    println!("seed 12345, random(200) x10: {draws_a:?}");
    assert_eq!(draws_a, draws_b, "same seed must replay identically");
    assert!(draws_a.iter().all(|&x| (0..200).contains(&x)), "draws in range");

    // 4. Different seed diverges.
    let draws_c: Vec<i32> = {
        let mut r = MatchRng::new_seeded(table, 999);
        (0..10).map(|_| r.random(200)).collect()
    };
    println!("seed 999,   random(200) x10: {draws_c:?}");
    assert_ne!(draws_a, draws_c, "different seed must differ");

    println!("\nRNG is bit-exact (LCG known-answer passes), the real table loads, and");
    println!("MatchRng::new_seeded is reproducible per seed. Ready for game generation.");
}
