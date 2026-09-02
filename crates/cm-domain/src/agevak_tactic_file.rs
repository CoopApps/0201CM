//! Tactic file (.tct / .pct) constants translated from
//! agevak/CM0102/CM0102Core/Save/Model/Tactic.cs.
//!
//! **This module cross-checks [`crate::tactic_file`] — my primary
//! tactic decoder** — and provides the AI-preset filename catalogue.
//!
//! ## Marker cross-check
//!
//! agevak defines `PCT_MARKER = {0x73, 0xB9, 0xF4, 0x07}` and
//! `TCT_MARKER = {0x5E, 0xEC, 0x98, 0x00}` as raw byte prefixes.
//! My port defines the same thing as a version-tag u32 at file offset
//! 0, with `.pct` files XOR-obfuscated by ADDing `0x075BCD15`.
//!
//! **The two decodes agree byte-for-byte:**
//! - `TCT_MARKER` little-endian = `0x0098EC5E` = my `v5E` version tag
//! - `PCT_MARKER` little-endian = `0x07F4B973` = `0x0098EC5E + 0x075BCD15`
//!
//! ## Size cross-check
//!
//! agevak accepts total file sizes 1428, 1432, and 1476 bytes. My port
//! has:
//! - v5C/v5D body = 1428 B  ← agevak's 1428 (marker inline) / 1432 (with
//!   4-byte trailer)
//! - v5E body = 1472 B  → 1476 total including agevak's 4-byte tail
//!   `{0xF2, 0xFF, 0xFF, 0xF2}` sentinel
//!
//! Confirmed consistent.
//!
//! ## New info from agevak (NOT in my port)
//!
//! - AI_PACK_FILENAMES / AI_PACK_DEFAULT_NAMES — the 45-entry catalogue
//!   the exe uses to load AI-manager tactic presets (see [`AI_PACK_FILENAMES`]).
//! - .sav-embed offsets — when a Tactic is written INTO a .sav file
//!   (not as a standalone .tct/.pct), the sub-blocks land at fixed
//!   offsets relative to the tactic's base: name at basePos, First250
//!   at basePos+51, Next88 at basePos+0x188B, Last11 at basePos+0x5B4,
//!   the "unmodified" flag at basePos+0x187F (1 = clean, 0 = user-
//!   edited, shows an asterisk in-game).

// ---------------------------------------------------------------------------
// Marker constants (agevak's byte view)
// ---------------------------------------------------------------------------

/// Raw file-prefix bytes for a `.pct` (AI-preset) tactic file. Little-
/// endian encoding of `0x07F4B973` — equal to `0x0098EC5E + 0x075BCD15`,
/// where `0x0098EC5E` is my v5E version tag and `0x075BCD15` is the
/// .pct XOR-obfuscation offset.
pub const PCT_MARKER: [u8; 4] = [0x73, 0xB9, 0xF4, 0x07];

/// Raw file-prefix bytes for a `.tct` (user-authored) tactic file.
/// Little-endian encoding of `0x0098EC5E` — my v5E version tag,
/// unobfuscated.
pub const TCT_MARKER: [u8; 4] = [0x5E, 0xEC, 0x98, 0x00];

/// The 4-byte trailer sentinel agevak appends when writing a tactic
/// file (my port notes it as optional — some saves write it, some
/// don't).
pub const FILE_TAIL_SENTINEL: [u8; 4] = [0xF2, 0xFF, 0xFF, 0xF2];

/// Maximum tactic name length in the name field.
pub const NAME_MAX_LENGTH: usize = 50;

// ---------------------------------------------------------------------------
// Sub-block sizes (agevak's decomposition)
// ---------------------------------------------------------------------------

/// After the 4-byte marker, agevak reads: First250 (250) + Crc (4) +
/// Next1115 (1115) + Next88 (88) + Last11 (11) = 1468 B body, plus
/// the 4-byte trailer for 1476 total. My port decodes many of these
/// bytes into typed fields; agevak keeps them as opaque byte blobs.
pub const FIRST250_LEN:  usize = 250;
pub const CRC_LEN:       usize = 4;
pub const NEXT1115_LEN:  usize = 1115;
pub const NEXT88_LEN:    usize = 88;
pub const LAST11_LEN:    usize = 11;

/// The 3 acceptable total file sizes per agevak.
pub const VALID_TOTAL_SIZES: [usize; 3] = [1428, 1432, 1476];

// ---------------------------------------------------------------------------
// .sav-embedded tactic offsets (agevak's WriteToSavFile method)
// ---------------------------------------------------------------------------

/// When a tactic is embedded inside a `.sav` file, its sub-blocks land
/// at these offsets relative to the tactic's base position.
///
/// (These are NEW offsets not in [`crate::tactic_file`] — my port so
/// far only handles standalone `.tct` / `.pct` files, not the inlined
/// .sav variant. Useful for a future .sav tactic reader.)
pub struct SavEmbedOffsets;

impl SavEmbedOffsets {
    /// Name field starts at basePos.
    pub const NAME:            usize = 0x0000;
    /// First250 block starts at basePos + NAME_MAX_LENGTH + 1 = 51.
    pub const FIRST250:        usize = 51;
    /// Next88 block starts at basePos + 0x188B.
    pub const NEXT88:          usize = 0x188B;
    /// Last11 block starts at basePos + 0x5B4.
    pub const LAST11:          usize = 0x5B4;
    /// "Unmodified" flag byte. Set to 1 for a clean preset, 0 for a
    /// user-edited tactic (game shows an asterisk on the tactic name
    /// when this is 0).
    pub const UNMODIFIED_FLAG: usize = 0x187F;
}

// ---------------------------------------------------------------------------
// AI preset catalogue — 45 shipped .pct files
// ---------------------------------------------------------------------------

/// The 45 tactic filenames the exe loads when populating an AI
/// manager's preset library. Some entries appear more than once — this
/// is intentional (agevak preserves duplicates from the original
/// exe's table).
pub const AI_PACK_FILENAMES: [&str; 45] = [
    "352_v1.pct", "442_v1.pct", "532_v1.pct", "442_v4.pct", "532_v2.pct",
    "532_v3.pct", "532_v4.pct", "352_v2.pct", "532_v5.pct", "532_v6.pct",
    "532_v7.pct", "532_v9.pct", "442_v8.pct", "sweeper_v1.pct",
    "442_push.pct", "defensive_counter.pct", "4312_v1.pct", "424_v1.pct",
    "442_v10.pct", "442_wide.pct", "4132.pct", "343_default.pct",
    "451_default.pct", "442_wide.pct", "sweeper_default.pct",
    "451_defensive.pct", "343_defensive.pct", "451_norway.pct",
    "442_default.pct", "442_defensive_default.pct", "442_push.pct",
    "442_diamond_default.pct", "343_default.pct", "352_default.pct",
    "352_defensive_default.pct", "352_attacking_default.pct",
    "41212_default.pct", "424_default.pct", "433_default.pct",
    "451_default.pct", "532_default.pct", "532_defensive_default.pct",
    "532_attacking_default.pct", "sweeper_default.pct", "4132.pct",
];

/// The display names shown in the AI-preset picker UI. Same order as
/// [`AI_PACK_FILENAMES`]; entries `[0..28]` are the raw preset labels
/// (`352_v1`, `442_v1`, ...) and entries `[28..45]` are the polished
/// display names (`4-4-2`, `4-4-2 Defensive`, ...).
pub const AI_PACK_DEFAULT_NAMES: [&str; 45] = [
    "352_v1", "442_v1", "532_v1", "442_v4", "532_v2",
    "532_v3", "532_v4", "352_v2", "532_v5", "532_v6",
    "532_v7", "532_v9", "442_v8", "sweeper_v1",
    "442_push", "defensive_counter", "4312_v1", "424_v1",
    "442_v10", "442_wide", "4132", "343_default",
    "451_default", "442_wide", "sweeper_default",
    "451_defensive", "343_defensive", "451_norway", "4-4-2",
    "4-4-2 Defensive", "4-4-2 Attacking", "4-4-2 Diamond",
    "3-4-3", "3-5-2", "3-5-2 Defensive", "3-5-2 Attacking",
    "4-1-2-1-2", "4-2-4", "4-3-3", "4-5-1", "5-3-2",
    "5-3-2 Defensive", "5-3-2 Attacking", "Sweeper", "4-1-3-2",
];

/// The 39 preset filenames the exe loads when the user clicks "Start
/// New Game" (from agevak's SaveGameTacticsEditor/Docs/StartNewGamePctFiles.txt).
/// This is a deduplicated SUBSET of [`AI_PACK_FILENAMES`] — the same
/// files, minus the 6 duplicate entries in the AI-manager preset table.
pub const START_NEW_GAME_PCT_FILES: [&str; 39] = [
    "343_default.pct", "343_defensive.pct", "352_attacking_default.pct",
    "352_default.pct", "352_defensive_default.pct", "352_v1.pct",
    "352_v2.pct", "41212_default.pct", "4132.pct", "424_default.pct",
    "424_v1.pct", "4312_v1.pct", "433_default.pct", "442_default.pct",
    "442_defensive_default.pct", "442_diamond_default.pct", "442_push.pct",
    "442_v1.pct", "442_v10.pct", "442_v4.pct", "442_v8.pct",
    "442_wide.pct", "451_default.pct", "451_defensive.pct",
    "451_norway.pct", "532_attacking_default.pct", "532_default.pct",
    "532_defensive_default.pct", "532_v1.pct", "532_v2.pct", "532_v3.pct",
    "532_v4.pct", "532_v5.pct", "532_v6.pct", "532_v7.pct", "532_v9.pct",
    "defensive_counter.pct", "sweeper_default.pct", "sweeper_v1.pct",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pct_marker_is_v5e_tag_plus_xor_offset() {
        // TCT_MARKER LE → my v5E version tag 0x0098EC5E
        let tct_le = u32::from_le_bytes(TCT_MARKER);
        assert_eq!(tct_le, 0x0098EC5E);

        // PCT_MARKER LE → v5E tag + 0x075BCD15 (the .pct XOR-obfuscation add)
        let pct_le = u32::from_le_bytes(PCT_MARKER);
        assert_eq!(pct_le, 0x0098EC5E + 0x075BCD15);
        assert_eq!(pct_le, 0x07F4B973);
    }

    #[test]
    fn sub_block_sizes_total_1476() {
        let total = 4 + FIRST250_LEN + CRC_LEN + NEXT1115_LEN + NEXT88_LEN
                  + LAST11_LEN + FILE_TAIL_SENTINEL.len();
        assert_eq!(total, 1476);
    }

    #[test]
    fn valid_sizes_cover_v5c_v5d_v5e() {
        assert!(VALID_TOTAL_SIZES.contains(&1428));  // v5C/v5D unmarked
        assert!(VALID_TOTAL_SIZES.contains(&1432));  // v5C/v5D + trailer
        assert!(VALID_TOTAL_SIZES.contains(&1476));  // v5E + trailer
    }

    #[test]
    fn ai_pack_catalogues_have_45_entries_each() {
        assert_eq!(AI_PACK_FILENAMES.len(), 45);
        assert_eq!(AI_PACK_DEFAULT_NAMES.len(), 45);
    }

    #[test]
    fn ai_pack_default_names_first_row_matches_filename_stem() {
        // Entries 0..28 are raw preset labels — the filename stem
        // without the .pct extension.
        for i in 0..28 {
            let stem = AI_PACK_FILENAMES[i].trim_end_matches(".pct");
            assert_eq!(stem, AI_PACK_DEFAULT_NAMES[i], "row {i} mismatch");
        }
    }

    #[test]
    fn start_new_game_list_is_subset_of_ai_pack_deduped() {
        // Every file in the start-new-game list appears at least once
        // in the AI pack list, and the start-new-game list has no
        // duplicates.
        for f in START_NEW_GAME_PCT_FILES.iter() {
            assert!(AI_PACK_FILENAMES.contains(f), "{f} missing from AI pack");
        }
        // No duplicates in the new-game list.
        let mut sorted: Vec<&str> = START_NEW_GAME_PCT_FILES.to_vec();
        sorted.sort();
        for i in 1..sorted.len() {
            assert_ne!(sorted[i-1], sorted[i], "duplicate {} in start-new-game list", sorted[i]);
        }
        // AI pack has exactly 6 duplicate entries (45 total, 39 unique).
        let mut ai_sorted: Vec<&str> = AI_PACK_FILENAMES.to_vec();
        ai_sorted.sort();
        ai_sorted.dedup();
        assert_eq!(ai_sorted.len(), 39, "AI pack has {} unique entries", ai_sorted.len());
    }

    #[test]
    fn sav_embed_offsets_are_stable() {
        assert_eq!(SavEmbedOffsets::NAME, 0);
        assert_eq!(SavEmbedOffsets::FIRST250, 51);
        assert_eq!(SavEmbedOffsets::UNMODIFIED_FLAG, 0x187F);
        assert_eq!(SavEmbedOffsets::NEXT88, 0x188B);
        // NEXT88 - UNMODIFIED_FLAG = 12 (the flag lives just before Next88 slot)
        assert_eq!(SavEmbedOffsets::NEXT88 - SavEmbedOffsets::UNMODIFIED_FLAG, 12);
    }
}
