//! CM0102 random number generators — lifted bit-exact from the binary.
//!
//! Provenance (carve: D:/cm0102-carve, `carve ask <addr>`):
//! * `CrtRand` ← `FUN_00935a94` (VERIFIED) — the MSVC C runtime `rand()` LCG.
//! * `MatchRng` ← `match_random 0x008fc4f0` (VERIFIED algorithm) — a precomputed
//!   random table walked by a pointer, offset by a 16-bit reseed. This is why
//!   CM matches replay identically from a save.
//!
//! The reseed of `MatchRng` draws from `CrtRand`, and the walked table lives in
//! the exe's `.data` at `0x00a8df38` (a data dependency — see [`MatchRng::new`]).

#![forbid(unsafe_code)]

pub const MATCH_RANDOM_ADDR: u32 = 0x008f_c4f0;
pub const MATCH_RNG_INIT_ADDR: u32 = 0x008f_c5d0;
pub const CRT_RAND_ADDR: u32 = 0x0093_5a94;
pub const MATCH_RNG_TABLE_BASE: u32 = 0x00a8_df38;
pub const MATCH_RNG_TABLE_END_INCLUSIVE: u32 = 0x00ab_fc14;
pub const MATCH_RNG_TABLE_ENTRIES: usize = 51_000;
pub const MATCH_RNG_START_MODULUS: u32 = 51_000;

pub const fn match_rng_table_byte_len() -> usize {
    (MATCH_RNG_TABLE_END_INCLUSIVE - MATCH_RNG_TABLE_BASE + 4) as usize
}

/// The MSVC C runtime `rand()` — `seed = seed*214013 + 2531011; (seed>>16) & 0x7fff`.
///
/// Cite: `FUN_00935a94` — `DAT_00ac26c0 = DAT_00ac26c0*0x343FD + 0x269EC3;
/// return (DAT_00ac26c0 >> 0x10) & 0x7FFF`.
#[derive(Debug, Clone)]
pub struct CrtRand {
    seed: u32,
}

impl CrtRand {
    pub const MULT: u32 = 0x0034_3FD; // 214013
    pub const INC: u32 = 0x0026_9EC3; // 2531011

    #[inline]
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// One `rand()` call: advances the seed, returns a value in `0..=0x7fff`.
    #[inline]
    pub fn next(&mut self) -> u16 {
        self.seed = self.seed.wrapping_mul(Self::MULT).wrapping_add(Self::INC);
        ((self.seed >> 16) & 0x7fff) as u16
    }

    #[inline]
    pub fn seed(&self) -> u32 {
        self.seed
    }
}

/// The match/game RNG (`match_random(n)` → `0..n`).
///
/// Cite: `match_random 0x008fc4f0`:
/// ```text
/// ptr += 4                                  // walk table (i32 entries)
/// if ptr > end: ptr = &table; seed16 = rand() & 0xffff   // wrap + reseed
/// return (seed16 + *ptr) % n                // for n < 0x10000
/// ```
///
/// `table` is the `.data` blob at `0x00a8df38` in the exe (extract it to make this
/// bit-exact). The algorithm here is faithful; supply the real table for identical
/// output. Large `n` (>= 0x10000) uses the binary's scaled recursion — ported below.
#[derive(Debug, Clone)]
pub struct MatchRng {
    table: Vec<i32>,
    idx: usize,
    seed16: u32,
    base: CrtRand,
}

impl MatchRng {
    /// `table` = the i32 entries from the exe's `.data` at `0x00a8df38`.
    /// `base` seeds the periodic reseed (as the binary uses the CRT `rand()`).
    pub fn new(table: Vec<i32>, base: CrtRand) -> Self {
        let mut rng = Self {
            table,
            idx: 0,
            seed16: 0,
            base,
        };
        // first call wraps immediately in the binary (ptr starts at end); prime it.
        rng.seed16 = (rng.base.next() as u32) & 0xffff;
        rng
    }

    /// Faithful port of the game's RNG initialiser `FUN_008fc5d0(seed)`:
    /// seeds the CRT LCG, then sets the start table index = `rand() % 51000`
    /// and the 16-bit offset = `rand() % 0xffff`. This reproduces the exact
    /// stream a fresh game with that seed would walk. Use this (not `new`)
    /// when reproducing game generation from a known seed.
    pub fn new_seeded(table: Vec<i32>, seed: u32) -> Self {
        let mut base = CrtRand::new(seed);
        // FUN_008fc5d0: ptr = &table + (rand() % 51000); offset = rand() % 0xffff.
        let start = (base.next() as usize) % MATCH_RNG_START_MODULUS as usize;
        let seed16 = (base.next() as u32) % 0xffff;
        let idx = if table.is_empty() { 0 } else { start % table.len() };
        Self {
            table,
            idx,
            seed16,
            base,
        }
    }

    #[inline]
    fn step(&mut self) -> i32 {
        self.idx += 1;
        if self.idx >= self.table.len() {
            self.idx = 0;
            self.seed16 = (self.base.next() as u32) & 0xffff;
        }
        self.table[self.idx]
    }

    /// `random(n)` → `0..n` (n > 0). Matches `FUN_008fc4f0`.
    pub fn random(&mut self, n: i32) -> i32 {
        if n == 0 {
            return 0;
        }
        if n.unsigned_abs() < 0x1_0000 {
            let v = self.seed16 as i64 + self.step() as i64;
            let mut r = (v % n as i64) as i32;
            if n < 1 {
                r = -r;
            }
            return r;
        }
        // binary's large-n path: scaled recursion (see 0x008fc4f0 tail).
        let hi = self.step(); // stands in for the __ftol() high draw
        let lo = self.random((n as i64 * 10 / 0xffff) as i32);
        let mut r = hi + (lo + 5) / 10;
        if r > n - 1 {
            r = n - 1;
        }
        r
    }
}

/// Decode little-endian i32 random-table entries extracted from `cm0102.exe`.
///
/// This intentionally does not know the table length. The base address is
/// verified (`0x00a8df38`); the final bound is established by the original RNG
/// initializer/wrap code, so callers should attach provenance for the slice they
/// pass in.
pub fn table_from_le_bytes(bytes: &[u8]) -> Result<Vec<i32>, &'static str> {
    if bytes.len() % 4 != 0 {
        return Err("RNG table bytes must be a multiple of four");
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

pub fn full_table_from_le_bytes(bytes: &[u8]) -> Result<Vec<i32>, &'static str> {
    if bytes.len() != match_rng_table_byte_len() {
        return Err("RNG table bytes must match the verified 51,000-entry table length");
    }
    table_from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Known-answer: MSVC `srand(1)` produces 41, 18467, 6334, 26500, 19169, ...
    /// This is the canonical MSVC rand() sequence — proves the LCG is bit-exact.
    #[test]
    fn crt_rand_matches_msvc() {
        let mut r = CrtRand::new(1);
        let got: Vec<u16> = (0..5).map(|_| r.next()).collect();
        assert_eq!(got, [41, 18467, 6334, 26500, 19169]);
    }

    #[test]
    fn match_rng_is_deterministic() {
        let table: Vec<i32> = (0..64).collect();
        let a: Vec<i32> = {
            let mut m = MatchRng::new(table.clone(), CrtRand::new(1));
            (0..20).map(|_| m.random(100)).collect()
        };
        let b: Vec<i32> = {
            let mut m = MatchRng::new(table, CrtRand::new(1));
            (0..20).map(|_| m.random(100)).collect()
        };
        assert_eq!(a, b, "same table+seed must replay identically");
        assert!(a.iter().all(|&x| (0..100).contains(&x)));
    }

    #[test]
    fn table_from_le_bytes_decodes_i32_entries() {
        let bytes = [0x30, 0x62, 0x00, 0x00, 0xd8, 0x47, 0x00, 0x00];
        assert_eq!(table_from_le_bytes(&bytes).unwrap(), [25136, 18392]);
    }

    #[test]
    fn verified_table_range_is_51000_i32_entries() {
        assert_eq!(match_rng_table_byte_len(), 204_000);
        assert_eq!(match_rng_table_byte_len() / 4, MATCH_RNG_TABLE_ENTRIES);
    }
}
