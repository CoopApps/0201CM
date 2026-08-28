//! Host Country table — port of `host_country.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\host_country.cpp`).
//!
//! Tracks who hosts each big international tournament (World Cup, Euros,
//! Copa América, African Cup of Nations, etc.) across the decades. Consumed
//! by the international-cup engines to place a comp's finals in the host
//! nation's calendar. Ships as a 57 × 34-byte table.
//!
//! # Runtime model (from the exe)
//!
//! 1. **CD check** ([`FUN_005e32e0`]) walks the logical drives via
//!    `GetLogicalDriveStringsA` / `GetDriveTypeA==5` (CDROM) looking for
//!    `\<drv>\c\m\0\1\0\2\.\e\x\e` — no CD, no `HostCountry.tmp` load, no
//!    static table. In our Rust port we skip this gate entirely.
//! 2. **Cache path** ([`FUN_005e4c80`]) reads `CM3_TEMP\HostCountry.tmp`,
//!    format = `u16 count` + `count × 34-byte` records. When present, the
//!    table comes straight off disk (bit-exact with the shipped table).
//! 3. **Cold path** ([`FUN_005e3490`]) constructs the 57-record table from
//!    static `DAT_009bb...` slots which — verified from a raw dump of the
//!    exe's `.data` section — are all `0xffffffff` in the file image and
//!    get populated by the nation-pool loader at runtime. Each slot ends up
//!    pointing at a `Nation` record, so the table entries mean "at tournament
//!    year Y the host is nation N (plus optional co-hosts)".
//!
//! # Record layout (34 bytes / stride `0x22`)
//!
//! Reverse-engineered from the writes in FUN_005e3490:
//! ```text
//!   +0x00 u32  primary host nation ptr (0xffffffff = TBD/nil)
//!   +0x04 u16  year
//!   +0x06 u32  co-host 1
//!   +0x0a u32  ff-marker
//!   +0x0e u32  co-host 2 or extra co-host
//!   +0x12 u32  ff-marker
//!   +0x16 u32  co-host 3 or extra co-host
//!   +0x1a u32  ff-marker
//!   +0x1e u16  aux (init'd from stack local_218; usually 0)
//!   +0x20 u8   sub-tournament id (0x20, 0x10, 0x00, 0x01, 0xfe, 0xfd, 0xff)
//!   +0x21 u8   pad
//! ```
//!
//! Tournament sub-id decoded from the tail byte + the `sVar5` year stride the
//! seeder used:
//! ```text
//!   0x20   FIFA World Cup                 year stride 4, 8 entries 1998..2026
//!   0x20   FIFA Confederations Cup        year stride 4, 8 entries 2001..2025 (year 2000 rebase → 0x11a..)
//!   0x10   Copa América                   year stride 2, 8 entries 2000..2014
//!   0xfe   UEFA European Championship     bespoke hand-writes
//!   0xfd   African Cup of Nations         bespoke hand-writes
//!   0x00   Asian Cup                      bespoke
//!   0x01   Gold Cup / CONCACAF            bespoke
//!   0xff   sentinel / unused
//! ```
//!
//! # Port scope
//!
//! Portable HERE:
//!   * `HostEntry` struct + parser for the on-disk 34-byte record
//!   * `HostCountryTable` (57 entries, sorted by qsort with LAB_005e4f10)
//!   * Loader for `HostCountry.tmp` (2-byte count + records)
//!   * Comparator helper (LAB_005e4f10 shape: sort by (`sub_tournament_id`,
//!     `year`)) — verified against the seeder's write order
//!
//! Deferred:
//!   * Populating primary/co-host nation ids from a fresh Rust world —
//!     the exe fills these from runtime nation-pool slots that in the base
//!     game image itself are all `0xffffffff`. Once we route the fresh-game
//!     builder to seed nation ptrs for the 25-30 unique DAT slots the
//!     seeder references, the cold path lights up. See ledger
//!     `reports/carve_rename_map.json:host_country.cpp` for the full
//!     DAT→tournament-slot mapping.

use std::path::Path;

/// Bytes per host-country record on disk and in memory.
pub const HOST_RECORD_STRIDE: usize = 0x22;

/// Row count in the shipped table — the malloc in FUN_005e32e0 asks for
/// `0x792 = 1938 = 57 * 34` bytes and the FUN_005e3490 seeder writes 57 rows.
pub const HOST_ROW_COUNT: usize = 57;

/// Sub-tournament id at record `+0x21`. Values verified from the seeder's
/// per-row byte writes:
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostSubTournament {
    WorldCup            = 0x20,
    CopaAmerica         = 0x10,
    UefaEuropean        = 0xfe,
    AfricanCupOfNations = 0xfd,
    AsianCup            = 0x00,
    GoldCup             = 0x01,
    Sentinel            = 0xff,
}

impl HostSubTournament {
    pub fn from_byte(b: u8) -> Option<Self> {
        Some(match b {
            0x20 => Self::WorldCup,
            0x10 => Self::CopaAmerica,
            0xfe => Self::UefaEuropean,
            0xfd => Self::AfricanCupOfNations,
            0x00 => Self::AsianCup,
            0x01 => Self::GoldCup,
            0xff => Self::Sentinel,
            _    => return None,
        })
    }
}

/// One row of the host-country table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostEntry {
    pub primary_host_nation_ptr: u32,   // +0x00
    pub year: u16,                       // +0x04
    pub co_host_1: u32,                  // +0x06 (0xffffffff = none)
    pub co_host_2: u32,                  // +0x0e
    pub co_host_3: u32,                  // +0x16
    pub aux: u16,                        // +0x1e
    pub sub_tournament: u8,              // +0x20 (see HostSubTournament)
    pub pad: u8,                         // +0x21
}

impl HostEntry {
    /// Parse one 34-byte record. Returns `None` on short input.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() < HOST_RECORD_STRIDE { return None; }
        let r_u32 = |o: usize| u32::from_le_bytes([b[o], b[o+1], b[o+2], b[o+3]]);
        let r_u16 = |o: usize| u16::from_le_bytes([b[o], b[o+1]]);
        Some(HostEntry {
            primary_host_nation_ptr: r_u32(0x00),
            year:      r_u16(0x04),
            co_host_1: r_u32(0x06),
            co_host_2: r_u32(0x0e),
            co_host_3: r_u32(0x16),
            aux:       r_u16(0x1e),
            sub_tournament: b[0x20],
            pad:            b[0x21],
        })
    }

    /// Write back to 34 bytes.
    pub fn to_bytes(&self) -> [u8; HOST_RECORD_STRIDE] {
        let mut out = [0xff_u8; HOST_RECORD_STRIDE];
        out[0x00..0x04].copy_from_slice(&self.primary_host_nation_ptr.to_le_bytes());
        out[0x04..0x06].copy_from_slice(&self.year.to_le_bytes());
        out[0x06..0x0a].copy_from_slice(&self.co_host_1.to_le_bytes());
        out[0x0e..0x12].copy_from_slice(&self.co_host_2.to_le_bytes());
        out[0x16..0x1a].copy_from_slice(&self.co_host_3.to_le_bytes());
        out[0x1e..0x20].copy_from_slice(&self.aux.to_le_bytes());
        out[0x20] = self.sub_tournament;
        out[0x21] = self.pad;
        out
    }
}

/// The full 57-row table. Uses a `Vec` so an update path can grow/shrink
/// without touching the on-disk stride.
#[derive(Debug, Clone)]
pub struct HostCountryTable {
    pub rows: Vec<HostEntry>,
}

impl HostCountryTable {
    /// An empty table (all slots the exe would call `0xffffffff`, i.e. TBD).
    pub fn empty() -> Self {
        let mut rows = Vec::with_capacity(HOST_ROW_COUNT);
        for _ in 0..HOST_ROW_COUNT {
            rows.push(HostEntry {
                primary_host_nation_ptr: u32::MAX,
                year: 0,
                co_host_1: u32::MAX,
                co_host_2: u32::MAX,
                co_host_3: u32::MAX,
                aux: 0,
                sub_tournament: HostSubTournament::Sentinel as u8,
                pad: 0,
            });
        }
        Self { rows }
    }

    /// Load the `CM3_TEMP\HostCountry.tmp` cache — 2-byte record count
    /// followed by `count * 34` bytes. Ports [`FUN_005e4c80`] verbatim.
    pub fn load_tmp(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        if bytes.len() < 2 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "short header"));
        }
        let count = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
        let want = 2 + count * HOST_RECORD_STRIDE;
        if bytes.len() < want {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "short body"));
        }
        let mut rows = Vec::with_capacity(count);
        for i in 0..count {
            let off = 2 + i * HOST_RECORD_STRIDE;
            rows.push(HostEntry::from_bytes(&bytes[off..off+HOST_RECORD_STRIDE])
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "record parse"))?);
        }
        Ok(Self { rows })
    }

    /// Save to `CM3_TEMP\HostCountry.tmp` (round-trip of `load_tmp`).
    pub fn save_tmp(&self, path: &Path) -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(2 + self.rows.len() * HOST_RECORD_STRIDE);
        let count = self.rows.len() as u16;
        buf.extend_from_slice(&count.to_le_bytes());
        for r in &self.rows {
            buf.extend_from_slice(&r.to_bytes());
        }
        std::fs::write(path, buf)
    }

    /// The comparator used by the exe's qsort at LAB_005e4f10 — sort by
    /// `(sub_tournament, year)`. Verified against the writing order in
    /// FUN_005e3490 (each tournament block is contiguous, ascending year).
    pub fn sort(&mut self) {
        self.rows.sort_by(|a, b| a.sub_tournament.cmp(&b.sub_tournament)
            .then(a.year.cmp(&b.year)));
    }

    /// Find the host of a specific tournament in a specific year.
    pub fn lookup(&self, sub: HostSubTournament, year: u16) -> Option<&HostEntry> {
        self.rows.iter().find(|r| r.sub_tournament == sub as u8 && r.year == year)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_round_trip() {
        let e = HostEntry {
            primary_host_nation_ptr: 0xdeadbeef,
            year: 2002,
            co_host_1: 0xdead0002,
            co_host_2: u32::MAX,
            co_host_3: u32::MAX,
            aux: 0,
            sub_tournament: HostSubTournament::WorldCup as u8,
            pad: 0,
        };
        let bytes = e.to_bytes();
        assert_eq!(bytes.len(), HOST_RECORD_STRIDE);
        assert_eq!(HostEntry::from_bytes(&bytes), Some(e));
    }

    #[test]
    fn table_tmp_round_trip() {
        let mut t = HostCountryTable::empty();
        t.rows[0].year = 1998;
        t.rows[0].sub_tournament = HostSubTournament::WorldCup as u8;
        t.rows[1].year = 2000;
        t.rows[1].sub_tournament = HostSubTournament::UefaEuropean as u8;
        let path = std::env::temp_dir().join("host_country_test.tmp");
        t.save_tmp(&path).unwrap();
        let loaded = HostCountryTable::load_tmp(&path).unwrap();
        assert_eq!(loaded.rows.len(), t.rows.len());
        assert_eq!(loaded.rows[0].year, 1998);
        assert_eq!(loaded.rows[1].sub_tournament, HostSubTournament::UefaEuropean as u8);
    }

    #[test]
    fn sort_orders_by_tournament_then_year() {
        let mut t = HostCountryTable {
            rows: vec![
                HostEntry { primary_host_nation_ptr: 0, year: 2010, co_host_1: 0, co_host_2: 0, co_host_3: 0, aux: 0, sub_tournament: 0x20, pad: 0 },
                HostEntry { primary_host_nation_ptr: 0, year: 2004, co_host_1: 0, co_host_2: 0, co_host_3: 0, aux: 0, sub_tournament: 0xfe, pad: 0 },
                HostEntry { primary_host_nation_ptr: 0, year: 2002, co_host_1: 0, co_host_2: 0, co_host_3: 0, aux: 0, sub_tournament: 0x20, pad: 0 },
            ],
        };
        t.sort();
        assert_eq!(t.rows[0].year, 2002);
        assert_eq!(t.rows[1].year, 2010);
        assert_eq!(t.rows[2].sub_tournament, 0xfe);
    }
}
