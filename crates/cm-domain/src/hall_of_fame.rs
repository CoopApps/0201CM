//! Hall of Fame — port of `hall_of_fame.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\hall_of_fame.cpp`).
//!
//! Fully-decoded scope:
//!   - Table dimensions from [`FUN_005d8b50`] (accessor bounds): 27 categories
//!     × 100 slots, stride 0x78 = 120 bytes per record. Total body = 0x4f1a0.
//!   - Persistence container from [`FUN_005d9ac0`] (`hof_write_bin`):
//!       * 5-byte header = 24-bit magic `0x3abfb2` + 1 version byte (`0x01`)
//!       * 0x4f1a0-byte obfuscated body
//!       * 4-byte checksum = sum of raw body bytes (as u32)
//!   - Obfuscation is a fixed 0x1e0-byte table `DAT_009ba1b0` used for:
//!       (a) rolling XOR: `body[i] ^= table[i % 0x1e0]`
//!       (b) permutation: for each `i`, swap `body[table[i % 0x1e0] % 0x4f1a0]`
//!           with `body[table[(i+1) % 0x1e0] % 0x4f1a0]`.
//!     The 0x1e0-byte table itself is a `.rdata` blob we have not yet dumped;
//!     until it's captured verbatim from the exe image, we CANNOT emit a
//!     byte-identical hall_of_fame.bin. The container shape is decoded, the
//!     table bytes are the missing input.
//!
//! Blocked scope:
//!   - Per-record 120-byte layout — the writer marshals opaque player fields
//!     (name/age/CA/PA/nationality/club/goals/... unknown packing). Populating
//!     it also requires the player-rating engine (see
//!     [[deferred-awards-engine]]).
//!
//! What this module ports honestly:
//!   * Dimensional constants (categories, slots, stride, body size, magic).
//!   * A bounds-checked accessor mirroring [`FUN_005d8b50`].
//!   * An in-memory table type ([`HallOfFame`]) with the 27×100 shape.
//!   * The obfuscation function shape (accepts the 0x1e0-byte table as a
//!     parameter so it works once the table blob is captured).

/// From [`FUN_005d8b50`]: `param_2 < 0x1b` (27) — number of HoF categories.
pub const HOF_CATEGORIES: usize = 27;
/// From [`FUN_005d8b50`]: `param_3 < 0x64` (100) — slots per category.
pub const HOF_SLOTS_PER_CATEGORY: usize = 100;
/// From [`FUN_005d8b50`]: stride = 0x78 — bytes per record.
pub const HOF_RECORD_STRIDE: usize = 0x78;
/// Total body bytes on disk (matches the writer's `0x4f1a0` constant).
pub const HOF_BODY_SIZE: usize = HOF_CATEGORIES * HOF_SLOTS_PER_CATEGORY * HOF_RECORD_STRIDE;
/// File magic in the 5-byte header (24 bits of `0x003abfb2`).
pub const HOF_MAGIC: u32 = 0x003abfb2;
/// Version byte in the header (5th byte).
pub const HOF_VERSION: u8 = 1;
/// Length of the fixed permutation table (`DAT_009ba1b0`), not yet dumped.
pub const HOF_TABLE_LEN: usize = 0x1e0;

/// The in-memory Hall of Fame: 27 × 100 × 120-byte opaque records.
///
/// Record layout is undecoded — we hold them as raw bytes so we can still
/// round-trip files once the container is complete.
#[derive(Clone)]
pub struct HallOfFame {
    /// Flat body: `[category][slot][byte]` laid out linearly.
    pub body: Vec<u8>,
}

impl HallOfFame {
    /// Empty (all zero) HoF.
    pub fn empty() -> Self {
        Self { body: vec![0u8; HOF_BODY_SIZE] }
    }

    /// Byte offset of `(category, slot)` in the body — ported from
    /// [`FUN_005d8b50`]: `(slot + category*100) * 0x78`.
    /// Returns `None` on out-of-range indices (the exe raises the standard
    /// runtime error at hall_of_fame.cpp line 0xe3).
    pub fn offset(category: u8, slot: u8) -> Option<usize> {
        if (category as usize) >= HOF_CATEGORIES { return None; }
        if (slot as usize) >= HOF_SLOTS_PER_CATEGORY { return None; }
        Some(((slot as usize) + (category as usize) * HOF_SLOTS_PER_CATEGORY) * HOF_RECORD_STRIDE)
    }

    /// Read-only view of a record. Layout is undecoded.
    pub fn record(&self, category: u8, slot: u8) -> Option<&[u8]> {
        let off = Self::offset(category, slot)?;
        Some(&self.body[off .. off + HOF_RECORD_STRIDE])
    }

    /// Mutable view of a record. Layout is undecoded.
    pub fn record_mut(&mut self, category: u8, slot: u8) -> Option<&mut [u8]> {
        let off = Self::offset(category, slot)?;
        Some(&mut self.body[off .. off + HOF_RECORD_STRIDE])
    }
}

/// Apply the exe's obfuscation (XOR + permutation) — [`FUN_005d9ac0`] step 2.
/// The 0x1e0-byte `table` is `DAT_009ba1b0` (not yet dumped from the exe).
///
/// Also returns the raw-byte checksum written into the 4-byte trailer.
pub fn obfuscate(body: &mut [u8], table: &[u8; HOF_TABLE_LEN]) -> u32 {
    assert_eq!(body.len(), HOF_BODY_SIZE);
    // Pass 1: rolling XOR + checksum accumulation over the ORIGINAL bytes.
    let mut checksum: u32 = 0;
    for i in 0 .. HOF_BODY_SIZE {
        checksum = checksum.wrapping_add(body[i] as u32);
        body[i] ^= table[i % HOF_TABLE_LEN];
    }
    // Pass 2: permutation swap; index i uses table[i], (i+1) as pair.
    for i in 0 .. HOF_BODY_SIZE {
        let a = table[i % HOF_TABLE_LEN] as usize % HOF_BODY_SIZE;
        let b = table[(i + 1) % HOF_TABLE_LEN] as usize % HOF_BODY_SIZE;
        body.swap(a, b);
    }
    checksum
}

/// Reverse the obfuscation — reader side (the exe's read function inverts the
/// same sequence). Same table, same checksum contract.
pub fn deobfuscate(body: &mut [u8], table: &[u8; HOF_TABLE_LEN]) -> u32 {
    assert_eq!(body.len(), HOF_BODY_SIZE);
    // Reverse pass 2 by iterating backwards.
    for i in (0 .. HOF_BODY_SIZE).rev() {
        let a = table[i % HOF_TABLE_LEN] as usize % HOF_BODY_SIZE;
        let b = table[(i + 1) % HOF_TABLE_LEN] as usize % HOF_BODY_SIZE;
        body.swap(a, b);
    }
    // Reverse pass 1: XOR back and compute checksum.
    let mut checksum: u32 = 0;
    for i in 0 .. HOF_BODY_SIZE {
        body[i] ^= table[i % HOF_TABLE_LEN];
        checksum = checksum.wrapping_add(body[i] as u32);
    }
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_match_exe() {
        assert_eq!(HOF_BODY_SIZE, 0x4f1a0);
        assert_eq!(HallOfFame::offset(0, 0), Some(0));
        assert_eq!(HallOfFame::offset(26, 99), Some(HOF_BODY_SIZE - HOF_RECORD_STRIDE));
        assert_eq!(HallOfFame::offset(27, 0), None);
        assert_eq!(HallOfFame::offset(0, 100), None);
    }

    #[test]
    fn obfuscate_deobfuscate_is_identity() {
        let table = [0x5au8; HOF_TABLE_LEN]; // placeholder — real bytes from DAT_009ba1b0
        let mut hof = HallOfFame::empty();
        for (i, b) in hof.body.iter_mut().enumerate() { *b = (i as u8).wrapping_mul(37); }
        let orig = hof.body.clone();
        let cs1 = obfuscate(&mut hof.body, &table);
        let cs2 = deobfuscate(&mut hof.body, &table);
        assert_eq!(hof.body, orig, "round-trip must be identity");
        assert_eq!(cs1, cs2, "checksum stable across the round-trip");
    }
}
