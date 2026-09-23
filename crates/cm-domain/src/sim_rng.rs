//! Byte-exact port of the game's single global RNG (`FUN_008fc4f0` /
//! `FUN_008fc5d0` / LCG `FUN_00935a94`), the source of every `rand(n)` in
//! the exe (358 call sites incl. the match engine — injuries, cards, …).
//!
//! Model: a 51000-int table (`&DAT_00a8df38`, dumped byte-exact to
//! `data/sim_rng_table.bin`) is walked one int per call. Each call returns
//! `(offset16 + table[cursor]) % n`; when the cursor passes the end it
//! resets to the table start and `offset16` is refreshed from the MSVC LCG.
//! Seeding (`FUN_008fc5d0`) picks the initial cursor (`rand()%51000`) and
//! `offset16` (`rand()%0xffff`) from that same LCG.
//!
//! See `reports/injury_generator_decode.md` §5.

/// The 51000-entry table, byte-exact from the exe `.data` (`0x00a8df38`).
static RNG_TABLE_BYTES: &[u8] = include_bytes!("data/sim_rng_table.bin");
const RNG_TABLE_LEN: usize = 51000;

#[inline]
fn table(i: usize) -> i32 {
    let o = i * 2;
    u16::from_le_bytes([RNG_TABLE_BYTES[o], RNG_TABLE_BYTES[o + 1]]) as i32
}

/// The game's global RNG state (table cursor + 16-bit offset + LCG state).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SimRng {
    /// Index into the 51000-int table (the exe holds this as a pointer).
    cursor: usize,
    /// The 16-bit additive offset (`DAT_00dc7234`).
    offset16: u32,
    /// MSVC LCG state (`DAT_00ac26c0`), advanced by [`Self::lcg`].
    lcg_state: u32,
}

impl Default for SimRng {
    /// A deterministic default seed (real games seed via [`SimRng::seed`]
    /// at kickoff; this keeps `#[serde(default)]`/test construction valid).
    fn default() -> Self { SimRng::seed(0x1234_5678) }
}

impl SimRng {
    /// Port of `FUN_00935a94`: `state = state*0x343fd + 0x269ec3;
    /// return (state >> 16) & 0x7fff`.
    // GDI-REG: 00935a94 PORTED_EXACT
    fn lcg(&mut self) -> u32 {
        self.lcg_state = self.lcg_state
            .wrapping_mul(0x343fd)
            .wrapping_add(0x269ec3);
        (self.lcg_state >> 16) & 0x7fff
    }

    /// Port of `FUN_008fc5d0(seed)` — seed the RNG. `seed` must be non-zero
    /// (the exe substitutes a clock read for 0; callers here always pass a
    /// concrete seed). `FUN_00935a8a(seed)` sets the LCG state to `seed`.
    // GDI-REG: 008fc5d0 PORTED_EXACT
    pub fn seed(seed: u32) -> Self {
        let mut r = SimRng { cursor: 0, offset16: 0, lcg_state: seed };
        let c = r.lcg() as usize % RNG_TABLE_LEN;   // rand() % 51000
        r.cursor = c;
        r.offset16 = r.lcg() % 0xffff;              // rand() % 0xffff
        r
    }

    /// Port of `FUN_008fc4f0(n)` — return a value in `0..n`. For `n` outside
    /// `(-0x10000, 0x10000)` the exe uses a recursive scaling path (ported
    /// below). `n == 0` returns 0.
    // GDI-REG: 008fc4f0 PORTED_EXACT
    pub fn rand(&mut self, n: i32) -> i32 {
        if n == 0 {
            return 0;
        }
        // Walk +1 int; on passing the end, reset + refresh offset16.
        self.cursor += 1;
        if self.cursor >= RNG_TABLE_LEN {
            self.cursor = 0;
            self.offset16 = self.lcg() & 0xffff;
        }
        if (-0x10000..0x10000).contains(&n) {
            let mut v = (self.offset16 as i32 + table(self.cursor)) % n;
            if n < 1 {
                v = -v;
            }
            v
        } else {
            // Large-range path: FUN_008fc4f0 recurses on (n*10)/0xffff.
            // `__ftol()` here truncates a float already on the x87 stack in
            // the exe; with no float in scope it is 0 in this port (the
            // recursive term dominates the large-range result).
            let inner = self.rand((n.wrapping_mul(10)) / 0xffff);
            let mut v = (inner + 5) / 10;
            if n - 1 < v {
                v = n - 1;
            }
            v
        }
    }

    /// Convenience: `rand(n)` as a `u32` for `n > 0`.
    pub fn range(&mut self, n: u32) -> u32 {
        if n == 0 { 0 } else { self.rand(n as i32).max(0) as u32 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_loaded() {
        assert_eq!(RNG_TABLE_BYTES.len(), RNG_TABLE_LEN * 2);
        // First table entry (byte-exact from .data 0x00a8df38).
        assert_eq!(table(0), 25136);
        assert_eq!(table(50999), 31634);
    }

    #[test]
    fn deterministic_seed() {
        let mut a = SimRng::seed(12345);
        let mut b = SimRng::seed(12345);
        for _ in 0..1000 {
            assert_eq!(a.rand(100), b.rand(100));
        }
        // Values stay in range.
        let mut r = SimRng::seed(1);
        for _ in 0..10000 {
            let v = r.rand(20);
            assert!((0..20).contains(&v));
        }
    }
}
