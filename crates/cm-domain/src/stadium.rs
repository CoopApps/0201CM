//! Stadium record from `Data/stadium.dat`.
//!
//! Fixed-size 78-byte records (`STADIUM_RECORD_SIZE`), 7099 of them in
//! the shipped database. See `reports/stadium_record_layout.md` for
//! the full field-by-field derivation. Field offsets are cross-checked
//! against the match-engine attendance helpers in
//! [`crate::match_engine_exe::stadium_offsets`].

use core::convert::TryInto;

/// On-disk size of a single stadium record.
pub const STADIUM_RECORD_SIZE: usize = 78;

/// Length of the null-padded name field.
pub const STADIUM_NAME_LEN: usize = 52;

/// Sentinel value stored in `city_id` when no city is linked.
pub const NO_CITY: u32 = 0xFFFF_FFFE;

/// Sentinel value stored in `expansion_status` when no build is active.
pub const EXPANSION_IDLE: i32 = -2;

/// `flags_b` bit set on all-seater / top-tier grounds.
pub const FLAG_ALL_SEATER: u8 = 0xFF;

/// A single row from `Data/stadium.dat` decoded into typed fields.
///
/// Serialises back to bytes byte-for-byte when the name fits inside
/// [`STADIUM_NAME_LEN`]; longer names are truncated at write time.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StadiumRecord {
    /// Sequential record id (0..7098 in the shipped DB).
    pub id: u32,
    /// Null-padded Latin-1 name, stored raw so round-trips are lossless.
    pub name: [u8; STADIUM_NAME_LEN],
    /// City the ground is in; links to `city.dat`. `NO_CITY` = none.
    pub city_id: u32,
    /// Total ground capacity (spectators).
    pub capacity: u32,
    /// All-seated capacity (invariant: `<= capacity`).
    pub seating: u32,
    /// Planned post-expansion capacity; `0` = no expansion planned.
    pub expansion_capacity: u32,
    /// Build progress / days remaining; `EXPANSION_IDLE` (-2) = idle.
    pub expansion_status: i32,
    /// Rarely-set byte (~24 records); provenance unknown.
    pub flags_a: u8,
    /// `FLAG_ALL_SEATER` on top-tier all-seater grounds; else `0`.
    pub flags_b: u8,
}

impl StadiumRecord {
    /// Decode a stadium record from its 78-byte on-disk representation.
    pub fn read_from_bytes(bytes: &[u8; STADIUM_RECORD_SIZE]) -> Self {
        let id = u32::from_le_bytes(bytes[0x00..0x04].try_into().unwrap());
        let mut name = [0u8; STADIUM_NAME_LEN];
        name.copy_from_slice(&bytes[0x04..0x04 + STADIUM_NAME_LEN]);
        let city_id = u32::from_le_bytes(bytes[0x38..0x3c].try_into().unwrap());
        let capacity = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap());
        let seating = u32::from_le_bytes(bytes[0x40..0x44].try_into().unwrap());
        let expansion_capacity =
            u32::from_le_bytes(bytes[0x44..0x48].try_into().unwrap());
        let expansion_status =
            i32::from_le_bytes(bytes[0x48..0x4c].try_into().unwrap());
        let flags_a = bytes[0x4c];
        let flags_b = bytes[0x4d];
        Self {
            id,
            name,
            city_id,
            capacity,
            seating,
            expansion_capacity,
            expansion_status,
            flags_a,
            flags_b,
        }
    }

    /// Re-encode into the exact 78-byte on-disk representation.
    pub fn write_to_bytes(&self) -> [u8; STADIUM_RECORD_SIZE] {
        let mut out = [0u8; STADIUM_RECORD_SIZE];
        out[0x00..0x04].copy_from_slice(&self.id.to_le_bytes());
        out[0x04..0x04 + STADIUM_NAME_LEN].copy_from_slice(&self.name);
        out[0x38..0x3c].copy_from_slice(&self.city_id.to_le_bytes());
        out[0x3c..0x40].copy_from_slice(&self.capacity.to_le_bytes());
        out[0x40..0x44].copy_from_slice(&self.seating.to_le_bytes());
        out[0x44..0x48].copy_from_slice(&self.expansion_capacity.to_le_bytes());
        out[0x48..0x4c].copy_from_slice(&self.expansion_status.to_le_bytes());
        out[0x4c] = self.flags_a;
        out[0x4d] = self.flags_b;
        out
    }

    /// Extract the name as a `String`, trimming trailing NULs.
    pub fn name_string(&self) -> String {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(STADIUM_NAME_LEN);
        // Latin-1 → String
        self.name[..end].iter().map(|&b| b as char).collect()
    }

    /// True when the ground is entirely seated (per `flags_b`).
    pub fn is_all_seater(&self) -> bool {
        self.flags_b == FLAG_ALL_SEATER
    }

    /// True when an expansion build is currently in progress.
    pub fn expansion_in_progress(&self) -> bool {
        self.expansion_status != EXPANSION_IDLE
    }
}

/// Helper: pack a Latin-1 string into a `[u8; STADIUM_NAME_LEN]`,
/// truncating if longer than the fixed field. Trailing bytes are zero.
pub fn pack_name(s: &str) -> [u8; STADIUM_NAME_LEN] {
    let mut out = [0u8; STADIUM_NAME_LEN];
    for (i, ch) in s.chars().take(STADIUM_NAME_LEN).enumerate() {
        out[i] = ch as u32 as u8; // Latin-1 fold
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real record #2 from `Data/stadium.dat` (Highbury), copied verbatim.
    const HIGHBURY: [u8; STADIUM_RECORD_SIZE] = [
        0x02, 0x00, 0x00, 0x00, // id = 2
        // name: "Highbury" + 44 NULs
        b'H', b'i', b'g', b'h', b'b', b'u', b'r', b'y',
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // city_id = 1010 (0x3f2)
        0xf2, 0x03, 0x00, 0x00,
        // capacity = 38420 (0x9614)
        0x14, 0x96, 0x00, 0x00,
        // seating = 38420
        0x14, 0x96, 0x00, 0x00,
        // expansion_capacity = 0
        0x00, 0x00, 0x00, 0x00,
        // expansion_status = 119 (0x77)
        0x77, 0x00, 0x00, 0x00,
        // flags_a = 0, flags_b = 0xff
        0x00, 0xff,
    ];

    #[test]
    fn size_matches_disk_layout() {
        assert_eq!(STADIUM_RECORD_SIZE, 78);
        assert_eq!(HIGHBURY.len(), STADIUM_RECORD_SIZE);
    }

    #[test]
    fn decodes_highbury_correctly() {
        let r = StadiumRecord::read_from_bytes(&HIGHBURY);
        assert_eq!(r.id, 2);
        assert_eq!(r.name_string(), "Highbury");
        assert_eq!(r.city_id, 1010);
        assert_eq!(r.capacity, 38420);
        assert_eq!(r.seating, 38420);
        assert_eq!(r.expansion_capacity, 0);
        assert_eq!(r.expansion_status, 119);
        assert!(r.is_all_seater());
        assert!(r.expansion_in_progress());
    }

    #[test]
    fn round_trip_highbury_bytes_exact() {
        let r = StadiumRecord::read_from_bytes(&HIGHBURY);
        let bytes = r.write_to_bytes();
        assert_eq!(bytes, HIGHBURY, "round-trip must be byte-exact");
    }

    #[test]
    fn boundary_values_round_trip() {
        let r = StadiumRecord {
            id: u32::MAX,
            name: pack_name(""),
            city_id: NO_CITY,                // 0xfffffffe
            capacity: 0,
            seating: 0,
            expansion_capacity: u32::MAX,
            expansion_status: i32::MIN,
            flags_a: 0xff,
            flags_b: 0xff,
        };
        let round = StadiumRecord::read_from_bytes(&r.write_to_bytes());
        assert_eq!(round, r);
        assert_eq!(round.city_id, NO_CITY);
        assert_eq!(round.expansion_status, i32::MIN);
    }

    #[test]
    fn packed_flags_decode() {
        // idle, non-all-seater (Dean Court-like)
        let idle = StadiumRecord {
            id: 42,
            name: pack_name("Dean Court"),
            city_id: 1009,
            capacity: 10_440,
            seating: 8_500,
            expansion_capacity: 15_000,
            expansion_status: EXPANSION_IDLE,
            flags_a: 0,
            flags_b: 0,
        };
        assert!(!idle.is_all_seater());
        assert!(!idle.expansion_in_progress());

        // all-seater with active build (Villa Park-like)
        let live = StadiumRecord {
            id: 3,
            name: pack_name("Villa Park"),
            city_id: 1011,
            capacity: 42_790,
            seating: 42_790,
            expansion_capacity: 60_000,
            expansion_status: 9,
            flags_a: 0,
            flags_b: FLAG_ALL_SEATER,
        };
        assert!(live.is_all_seater());
        assert!(live.expansion_in_progress());
    }

    #[test]
    fn name_truncates_and_pads() {
        let long = "A".repeat(80);
        let packed = pack_name(&long);
        assert_eq!(packed.len(), STADIUM_NAME_LEN);
        assert!(packed.iter().all(|&b| b == b'A'));

        let short = pack_name("X");
        assert_eq!(short[0], b'X');
        assert!(short[1..].iter().all(|&b| b == 0));

        let r = StadiumRecord {
            id: 1,
            name: short,
            city_id: 0,
            capacity: 0,
            seating: 0,
            expansion_capacity: 0,
            expansion_status: 0,
            flags_a: 0,
            flags_b: 0,
        };
        assert_eq!(r.name_string(), "X");
    }

    #[test]
    fn matches_match_engine_attendance_offsets() {
        // Capacity offset must line up with what the ported match-engine
        // helper reads. `stadium_offsets::CAPACITY` = 0x38 on the runtime
        // venue struct; on the *static* record here capacity lives at
        // 0x3c (0x38 is city_id). Guard against accidental drift.
        assert_eq!(crate::match_engine_exe::stadium_offsets::CAPACITY, 0x38);
        // Sanity: seating never exceeds capacity in a well-formed record.
        let r = StadiumRecord::read_from_bytes(&HIGHBURY);
        assert!(r.seating <= r.capacity);
    }
}
