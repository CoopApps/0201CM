//! Player/club disputes — a port of `dispute.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\dispute.cpp`, VA `0x005536b0..0x005545ff`,
//! single function `dispute_load_or_save 0x005536b0`).
//!
//! The exe's `dispute.dat` is an array of 4-byte records with a running count
//! at `DAT_00acd56c`. The file is loaded via the shared file API
//! (open `0x00921770`, read `0x00921ea0`, close `0x00921b90`) — the same
//! pattern as `club_history.cpp` / `club_records.cpp`.
//!
//! This ports the record type + the little-endian decoder. `dispute.dat` is
//! **not yet imported** into rust-db, so the runtime table stays empty until
//! `cm-import` picks the file up — the honest stand-in for now.

use serde::{Deserialize, Serialize};

/// One `dispute.dat` record — a 4-byte cross-reference row. The two ids in the
/// record identify the parties in a dispute (player↔club or club↔club); the
/// full semantics are cross-referenced from elsewhere in the engine and are a
/// documented follow-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisputeRecord {
    pub a: u16,
    pub b: u16,
}

impl DisputeRecord {
    /// The record is 4 bytes; `dispute_load_or_save` reads it verbatim.
    pub const SIZE: usize = 4;

    pub fn from_le_bytes(bytes: [u8; 4]) -> Self {
        Self {
            a: u16::from_le_bytes([bytes[0], bytes[1]]),
            b: u16::from_le_bytes([bytes[2], bytes[3]]),
        }
    }

    pub fn to_le_bytes(self) -> [u8; 4] {
        let a = self.a.to_le_bytes();
        let b = self.b.to_le_bytes();
        [a[0], a[1], b[0], b[1]]
    }
}

/// Decode a whole `dispute.dat` blob (as the exe reads it). Extra trailing
/// bytes are ignored.
pub fn read_disputes(bytes: &[u8]) -> Vec<DisputeRecord> {
    bytes
        .chunks_exact(DisputeRecord::SIZE)
        .map(|c| DisputeRecord::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_record() {
        let r = DisputeRecord { a: 0x1234, b: 0xABCD };
        assert_eq!(DisputeRecord::from_le_bytes(r.to_le_bytes()), r);
    }

    #[test]
    fn reads_a_multi_record_blob() {
        let bytes = [
            0x10, 0x00, 0x20, 0x00, // (16, 32)
            0x30, 0x00, 0x40, 0x00, // (48, 64)
            0xAA,                    // trailing byte, ignored
        ];
        let out = read_disputes(&bytes);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], DisputeRecord { a: 16, b: 32 });
        assert_eq!(out[1], DisputeRecord { a: 48, b: 64 });
    }
}
