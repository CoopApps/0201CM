//! Reference struct layouts from the agevak/CM0102 project.
//!
//! Source: <https://github.com/agevak/CM0102/tree/master/CM0102Core/Save/Model>
//!
//! Community-reverse-engineered CM01/02 save-file struct layouts.
//! Used to CROSS-CHECK our own decoded field offsets in
//! [`crate::typed_records`] — when the two disagree, we treat it as a
//! bug candidate needing verification against real .dat files.
//!
//! These are DOCUMENTATION-ONLY structs — they define byte offsets as
//! constants, without displacing the existing port's field access.

use serde::{Serialize, Deserialize};

// ---------------------------------------------------------------------------
// TCMDate (8 bytes)
// ---------------------------------------------------------------------------

/// Agevak's TCMDate layout — how dates are stored in .sav.
///
/// Fields (byte-exact):
/// - `+0` day (i16, 0-indexed day-of-year)
/// - `+2` year (i16)
/// - `+4` leap_year (i32, 0 or 1)
///
/// Total: 8 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgevakTCMDate {
    pub day: i16,
    pub year: i16,
    pub leap_year: i32,
}

impl AgevakTCMDate {
    pub const SIZE: usize = 8;
    pub const OFFSET_DAY:       usize = 0;
    pub const OFFSET_YEAR:      usize = 2;
    pub const OFFSET_LEAP_YEAR: usize = 4;
}

// ---------------------------------------------------------------------------
// TPlayer (70 bytes = 0x46) — the type-10 attribute record
// ---------------------------------------------------------------------------

/// Agevak's TPlayer field-offset constants. Cross-checks our port's
/// type-10 attribute-record decode.
///
/// **IMPORTANT DISCREPANCY vs our port**:
/// - Our port ([`crate::PlayerAttrs::squad_number`] at footer +0x45)
/// - Agevak (SquadNumber at +0x04, PlayerMorale at +0x45)
///
/// Both are 70-byte structs. Both are for the type-10 attribute pool.
/// Only one can be right for the SAME real byte. This is flagged as a
/// bug candidate — needs verification against a real cm0102 .sav file
/// where a specific player's known squad number can be located at
/// either offset.
pub struct AgevakTPlayerOffsets;

impl AgevakTPlayerOffsets {
    pub const SIZE: usize = 0x46;

    // Identity + squad
    pub const ID:            usize = 0x00;  // i32
    pub const SQUAD_NUMBER:  usize = 0x04;  // u8  ← agevak says HERE, not +0x45
    // Ratings (i16)
    pub const CURRENT_ABILITY:   usize = 0x05;
    pub const POTENTIAL_ABILITY: usize = 0x07;
    // Reputations (u16)
    pub const HOME_REPUTATION:    usize = 0x09;
    pub const CURRENT_REPUTATION: usize = 0x0B;
    pub const WORLD_REPUTATION:   usize = 0x0D;
    // Position aptitudes (sbyte × 12) — matches our port's decode
    pub const GOALKEEPER:               usize = 0x0F;
    pub const SWEEPER:                  usize = 0x10;
    pub const DEFENDER:                 usize = 0x11;
    pub const DEFENSIVE_MIDFIELDER:     usize = 0x12;
    pub const MIDFIELDER:               usize = 0x13;
    pub const ATTACKING_MIDFIELDER:     usize = 0x14;
    pub const ATTACKER:                 usize = 0x15;
    pub const WING_BACK:                usize = 0x16;
    pub const RIGHT_SIDE:               usize = 0x17;
    pub const LEFT_SIDE:                usize = 0x18;
    pub const CENTRAL:                  usize = 0x19;
    pub const FREE_ROLE:                usize = 0x1A;
    // Attributes (sbyte × 42) — matches our port's decode
    pub const ACCELERATION:      usize = 0x1B;
    pub const AGGRESSION:        usize = 0x1C;
    pub const AGILITY:           usize = 0x1D;
    pub const ANTICIPATION:      usize = 0x1E;
    pub const BALANCE:           usize = 0x1F;
    pub const BRAVERY:           usize = 0x20;
    pub const CONSISTENCY:       usize = 0x21;
    pub const CORNERS:           usize = 0x22;
    pub const CROSSING:          usize = 0x23;
    pub const DECISIONS:         usize = 0x24;
    pub const DIRTINESS:         usize = 0x25;
    pub const DRIBBLING:         usize = 0x26;
    pub const FINISHING:         usize = 0x27;
    pub const FLAIR:             usize = 0x28;
    pub const FREE_KICKS:        usize = 0x29;
    pub const HANDLING:          usize = 0x2A;
    pub const HEADING:           usize = 0x2B;
    pub const IMPORTANT_MATCHES: usize = 0x2C;
    pub const INJURY_PRONENESS:  usize = 0x2D;
    pub const JUMPING:           usize = 0x2E;
    pub const LEADERSHIP:        usize = 0x2F;
    pub const LEFT_FOOT:         usize = 0x30;
    pub const LONG_SHOTS:        usize = 0x31;
    pub const MARKING:           usize = 0x32;
    pub const MOVEMENT:          usize = 0x33;
    pub const NATURAL_FITNESS:   usize = 0x34;
    pub const ONE_ON_ONES:       usize = 0x35;
    pub const PLAYER_PACE:       usize = 0x36;
    pub const PASSING:           usize = 0x37;
    pub const PENALTIES:         usize = 0x38;
    pub const POSITIONING:       usize = 0x39;
    pub const REFLEXES:          usize = 0x3A;
    pub const RIGHT_FOOT:        usize = 0x3B;
    pub const STAMINA:           usize = 0x3C;
    pub const STRENGTH:          usize = 0x3D;
    pub const TACKLING:          usize = 0x3E;
    pub const TEAMWORK:          usize = 0x3F;
    pub const TECHNIQUE:         usize = 0x40;
    pub const THROW_INS:         usize = 0x41;
    pub const VERSATILITY:       usize = 0x42;
    pub const VISION:            usize = 0x43;
    pub const WORK_RATE:         usize = 0x44;
    // Footer — agevak says PlayerMorale (our port says squad_number)
    pub const PLAYER_MORALE_AGEVAK: usize = 0x45;
}

// ---------------------------------------------------------------------------
// TStaff (partial — key offsets)
// ---------------------------------------------------------------------------

/// Agevak's TStaff struct offsets. Full 157-byte layout per our
/// [[record-layouts-decoded]] memory — this reference covers the
/// personality attributes at the tail end.
pub struct AgevakTStaffOffsets;

impl AgevakTStaffOffsets {
    pub const SIZE: usize = 157;
    // 8 staff-only "personality" attributes at the end of the struct
    pub const ADAPTABILITY:    usize = 0x94;   // approx — verify
    pub const AMBITION:        usize = 0x95;
    pub const DETERMINATION:   usize = 0x96;
    pub const LOYALTY:         usize = 0x97;
    pub const PRESSURE:        usize = 0x98;
    pub const PROFESSIONALISM: usize = 0x99;
    pub const SPORTSMANSHIP:   usize = 0x9A;
    pub const TEMPERAMENT:     usize = 0x9B;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tplayer_offsets_match_agevak_c_sharp_layout() {
        // Cross-check the byte offsets match agevak's TPlayer C# struct
        // (verified line-by-line against
        // https://github.com/agevak/CM0102/blob/master/CM0102Core/Save/Model/Structures.cs
        // TPlayer class, /*NN*/ comments).
        assert_eq!(AgevakTPlayerOffsets::SIZE, 0x46);
        assert_eq!(AgevakTPlayerOffsets::SQUAD_NUMBER,  0x04);
        assert_eq!(AgevakTPlayerOffsets::CURRENT_ABILITY, 0x05);
        assert_eq!(AgevakTPlayerOffsets::POTENTIAL_ABILITY, 0x07);
        assert_eq!(AgevakTPlayerOffsets::GOALKEEPER,    0x0F);
        assert_eq!(AgevakTPlayerOffsets::ACCELERATION,  0x1B);
        assert_eq!(AgevakTPlayerOffsets::WORK_RATE,     0x44);
        assert_eq!(AgevakTPlayerOffsets::PLAYER_MORALE_AGEVAK, 0x45);
    }

    #[test]
    fn ratings_are_all_i16_at_5_7_9_b_d() {
        // 5 ratings, i16 each, consecutive from 0x05..0x0F
        assert_eq!(AgevakTPlayerOffsets::CURRENT_ABILITY,   0x05);
        assert_eq!(AgevakTPlayerOffsets::POTENTIAL_ABILITY, 0x07);
        assert_eq!(AgevakTPlayerOffsets::HOME_REPUTATION,   0x09);
        assert_eq!(AgevakTPlayerOffsets::CURRENT_REPUTATION, 0x0B);
        assert_eq!(AgevakTPlayerOffsets::WORLD_REPUTATION,  0x0D);
    }

    #[test]
    fn twelve_position_aptitudes_at_0f_to_1a() {
        // 12 aptitudes at consecutive offsets 0x0F..0x1B (exclusive)
        assert_eq!(AgevakTPlayerOffsets::GOALKEEPER,           0x0F);
        assert_eq!(AgevakTPlayerOffsets::FREE_ROLE,            0x1A);
        assert_eq!(AgevakTPlayerOffsets::FREE_ROLE - AgevakTPlayerOffsets::GOALKEEPER + 1, 12);
    }

    #[test]
    fn forty_two_attributes_at_1b_to_44() {
        // 42 attributes at consecutive offsets 0x1B..0x45 (exclusive)
        assert_eq!(AgevakTPlayerOffsets::ACCELERATION, 0x1B);
        assert_eq!(AgevakTPlayerOffsets::WORK_RATE,    0x44);
        assert_eq!(AgevakTPlayerOffsets::WORK_RATE - AgevakTPlayerOffsets::ACCELERATION + 1, 42);
    }

    #[test]
    fn tcmdate_layout() {
        assert_eq!(AgevakTCMDate::SIZE, 8);
        assert_eq!(AgevakTCMDate::OFFSET_DAY,  0);
        assert_eq!(AgevakTCMDate::OFFSET_YEAR, 2);
        assert_eq!(AgevakTCMDate::OFFSET_LEAP_YEAR, 4);
    }
}
