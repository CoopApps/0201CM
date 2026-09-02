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
// TStaff (110 bytes = 0x6E) — the .sav TStaff record
// ---------------------------------------------------------------------------

/// Agevak's TStaff struct offsets (110 bytes total).
///
/// NOTE this is the in-`.sav` TStaff record, NOT the 157-byte
/// standalone staff.dat record from [[record-layouts-decoded]] — those
/// are two different container formats.
pub struct AgevakTStaffOffsets;

impl AgevakTStaffOffsets {
    pub const SIZE: usize = 0x6E;   // 110

    pub const ID:                 usize = 0x00;   // i32
    pub const FIRST_NAME:         usize = 0x04;   // i32 (index into names table)
    pub const SECOND_NAME:        usize = 0x08;   // i32
    pub const COMMON_NAME:        usize = 0x0C;   // i32
    pub const DATE_OF_BIRTH:      usize = 0x10;   // TCMDate (8 bytes)
    pub const YEAR_OF_BIRTH:      usize = 0x18;   // u16
    pub const NATION:             usize = 0x1A;   // i32
    pub const SECOND_NATION:      usize = 0x1E;   // i32
    pub const INT_APPS:           usize = 0x22;   // u8
    pub const INT_GOALS:          usize = 0x23;   // u8
    pub const NATIONAL_JOB:       usize = 0x24;   // i32
    pub const JOB_FOR_NATION:     usize = 0x28;   // u8
    pub const DATE_JOINED_NATION: usize = 0x29;   // TCMDate
    pub const DATE_EXPIRES_NATION:usize = 0x31;   // TCMDate
    pub const CLUB_JOB:           usize = 0x39;   // i32
    pub const JOB_FOR_CLUB:       usize = 0x3D;   // u8
    pub const DATE_JOINED_CLUB:   usize = 0x3E;   // TCMDate
    pub const DATE_EXPIRES_CLUB:  usize = 0x46;   // TCMDate
    pub const WAGE:               usize = 0x4E;   // i32
    pub const VALUE:              usize = 0x52;   // i32
    // 8 personality attributes
    pub const ADAPTABILITY:    usize = 0x56;
    pub const AMBITION:        usize = 0x57;
    pub const DETERMINATION:   usize = 0x58;
    pub const LOYALTY:         usize = 0x59;
    pub const PRESSURE:        usize = 0x5A;
    pub const PROFESSIONALISM: usize = 0x5B;
    pub const SPORTSMANSHIP:   usize = 0x5C;
    pub const TEMPERAMENT:     usize = 0x5D;
    pub const PLAYING_SQUAD:      usize = 0x5E;   // u8
    pub const CLASSIFICATION:     usize = 0x5F;   // u8
    pub const CLUB_VALUATION:     usize = 0x60;   // u8
    pub const PLAYER:             usize = 0x61;   // i32  (link to TPlayer record)
    pub const STAFF_PREFERENCES:  usize = 0x65;   // i32
    pub const NON_PLAYER:         usize = 0x69;   // i32
    pub const SQUAD_SELECTED_FOR: usize = 0x6D;   // u8
}

// ---------------------------------------------------------------------------
// TContract (variable — TCMDate embeds)
// ---------------------------------------------------------------------------

/// Agevak's TContract layout — describes a staff contract (wage,
/// bonuses, release clauses, expiry date). Byte offsets not explicitly
/// commented in agevak source; sizes are:
/// - 8×i32 head (ID..CleanSheetBonus)         → 32 bytes
/// - 5×u8 release-clause flags               → 5 bytes  (offset 32)
/// - i32 ReleaseFee                          → 4 bytes  (offset 37)
/// - TCMDate DateStarted + ContractExpires   → 16 bytes (offset 41)
/// - u8 ContractType                         → 1 byte   (offset 57)
/// - unknown 18_1..18_4                      → 8+8+2+1 = 19 bytes
/// - u8 LeavingOnBosman + i32 TransferArrangedFor +
///   u8 TransferStatus + u8 SquadStatus      → 7 bytes
///
/// TOTAL: 32 + 5 + 4 + 16 + 1 + 19 + 7 = 84 bytes.
pub struct AgevakTContractOffsets;

impl AgevakTContractOffsets {
    pub const SIZE: usize = 84;

    pub const ID:                usize = 0x00;
    pub const CLUB:              usize = 0x04;
    pub const UNKNOWN:           usize = 0x08;
    pub const WAGE:              usize = 0x0C;
    pub const GOAL_BONUS:        usize = 0x10;
    pub const ASSIST_BONUS:      usize = 0x14;
    pub const CLEAN_SHEET_BONUS: usize = 0x18;
    pub const NON_PROMOTION_RC:  usize = 0x1C;
    pub const MINIMUM_FEE_RC:    usize = 0x1D;
    pub const NON_PLAYING_RC:    usize = 0x1E;
    pub const RELEGATION_RC:     usize = 0x1F;
    pub const MANAGER_JOB_RC:    usize = 0x20;
    pub const RELEASE_FEE:       usize = 0x21;
    pub const DATE_STARTED:      usize = 0x25;    // TCMDate (8)
    pub const CONTRACT_EXPIRES:  usize = 0x2D;    // TCMDate (8)
    pub const CONTRACT_TYPE:     usize = 0x35;
    // 0x36..0x48 are the 4 unknown fields
    pub const LEAVING_ON_BOSMAN:     usize = 0x49;
    pub const TRANSFER_ARRANGED_FOR: usize = 0x4A;
    pub const TRANSFER_STATUS:       usize = 0x4E;
    pub const SQUAD_STATUS:          usize = 0x4F;
}

// ---------------------------------------------------------------------------
// TComp (variable) — the club competition record
// ---------------------------------------------------------------------------

/// Agevak's TComp struct offsets (club competition record).
///
/// Cross-checks our [[record-layouts-decoded]] port
/// (nation@0x5D, three_letter@0x53, reputation@0x69).
/// **Confirmed identical** — this is the same layout.
pub struct AgevakTCompOffsets;

impl AgevakTCompOffsets {
    pub const ID:                 usize = 0x00;
    pub const NAME:               usize = 0x04;   // 51 bytes
    pub const GENDER_NAME:        usize = 0x37;
    pub const SHORT_NAME:         usize = 0x38;   // 26 bytes
    pub const SHORT_GENDER_NAME:  usize = 0x52;
    pub const THREE_LETTER_NAME:  usize = 0x53;   // 4 bytes
    pub const CLUB_COMP_SCOPE:    usize = 0x57;
    pub const CLUB_COMP_SELECTED: usize = 0x58;
    pub const CLUB_COMP_CONTINENT:usize = 0x59;   // i32
    pub const CLUB_COMP_NATION:   usize = 0x5D;   // i32  ← matches our port
    pub const FOREGROUND_COLOUR:  usize = 0x61;   // i32
    pub const BACKGROUND_COLOUR:  usize = 0x65;   // i32
    pub const REPUTATION:         usize = 0x69;   // i16  ← matches our port
}

// ---------------------------------------------------------------------------
// TNation (variable) — cross-checks the 0x71 continent-id
// ---------------------------------------------------------------------------

/// Agevak's TNation struct offsets.
///
/// Cross-checks our memory ledger [[record-layouts-decoded]] and
/// [[african-nations-ported]] which pin continent id at +0x71.
/// **Agevak confirms `Continent` at exactly +0x71.**
pub struct AgevakTNationOffsets;

impl AgevakTNationOffsets {
    pub const ID:                 usize = 0x00;
    pub const NAME:               usize = 0x04;   // 51 bytes
    pub const GENDER_NAME:        usize = 0x37;
    pub const SHORT_NAME:         usize = 0x38;   // 26
    pub const SHORT_GENDER_NAME:  usize = 0x52;
    pub const THREE_LETTER_NAME:  usize = 0x53;   // 4
    pub const NATIONALITY:        usize = 0x57;   // 26
    pub const CONTINENT:          usize = 0x71;   // i32  ← matches our port
    pub const REGION:             usize = 0x75;
    pub const ACTUAL_REGION:      usize = 0x76;
    pub const FIRST_LANGUAGE:     usize = 0x77;
    pub const SECOND_LANGUAGE:    usize = 0x78;
    pub const THIRD_LANGUAGE:     usize = 0x79;
    pub const CAPITAL_CITY:       usize = 0x7A;   // i32
    pub const STATE_OF_DEVELOPMENT: usize = 0x7E;
    pub const GROUP_MEMBERSHIP:   usize = 0x7F;
    pub const NATIONAL_STADIUM:   usize = 0x80;
    pub const GAME_IMPORTANCE:    usize = 0x84;
    pub const LEAGUE_STANDARD:    usize = 0x85;
    pub const NUMBER_CLUBS:       usize = 0x86;   // i16
    pub const NUMBER_STAFF:       usize = 0x88;   // i32
    pub const SEASON_UPDATE_DAY:  usize = 0x8C;   // i16
    pub const REPUTATION:         usize = 0x8E;   // i16
    // FIFA/UEFA coefficients — 6 doubles each (0xA8..0x110)
    pub const FIFA_COEFFICIENT:   usize = 0xA8;   // f64
    pub const UEFA_COEFFICIENT91: usize = 0xE0;   // f64
}

// ---------------------------------------------------------------------------
// TIndex (section-directory entry inside a .sav / .dat container)
// ---------------------------------------------------------------------------

/// Agevak's TIndex layout — one entry in the section directory that
/// precedes the per-subsystem save streams in a .sav container.
///
/// See [[sav-file-format]]: our decode calls the same directory
/// entries `Section {field_a, offset, size, name}` with 0x10c-byte
/// stride. Agevak's TIndex is smaller (67 bytes) and named — this may
/// be the .dat variant, or a version-2 header layout.
pub struct AgevakTIndexOffsets;

impl AgevakTIndexOffsets {
    pub const NAME:      usize = 0x00;   // 51 bytes
    pub const FILE_TYPE: usize = 0x33;   // i32
    pub const COUNT:     usize = 0x37;   // i32
    pub const OFFSET:    usize = 0x3B;   // i32
    pub const VERSION:   usize = 0x3F;   // i32
    pub const SIZE:      usize = 0x43;   // 67
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
    fn tstaff_layout_totals_110() {
        assert_eq!(AgevakTStaffOffsets::SIZE, 110);
        assert_eq!(AgevakTStaffOffsets::ID, 0x00);
        assert_eq!(AgevakTStaffOffsets::ADAPTABILITY, 0x56);
        assert_eq!(AgevakTStaffOffsets::TEMPERAMENT, 0x5D);
        assert_eq!(AgevakTStaffOffsets::PLAYER, 0x61);
        assert_eq!(AgevakTStaffOffsets::SQUAD_SELECTED_FOR, 0x6D);
        // 8 consecutive personality attrs
        assert_eq!(
            AgevakTStaffOffsets::TEMPERAMENT - AgevakTStaffOffsets::ADAPTABILITY + 1,
            8
        );
    }

    #[test]
    fn tcomp_matches_our_ported_offsets() {
        // Cross-check: our port's typed_records.rs decodes
        // ClubComp.nation @ +0x5D, three_letter @ +0x53, reputation
        // @ +0x69. Agevak agrees.
        assert_eq!(AgevakTCompOffsets::THREE_LETTER_NAME, 0x53);
        assert_eq!(AgevakTCompOffsets::CLUB_COMP_NATION, 0x5D);
        assert_eq!(AgevakTCompOffsets::REPUTATION, 0x69);
    }

    #[test]
    fn tnation_continent_at_0x71() {
        // Cross-check: [[record-layouts-decoded]] and
        // [[african-nations-ported]] both pin continent id at +0x71.
        assert_eq!(AgevakTNationOffsets::CONTINENT, 0x71);
        assert_eq!(AgevakTNationOffsets::THREE_LETTER_NAME, 0x53);
        assert_eq!(AgevakTNationOffsets::REPUTATION, 0x8E);
    }

    #[test]
    fn tindex_67_bytes() {
        assert_eq!(AgevakTIndexOffsets::SIZE, 67);
    }

    #[test]
    fn tcmdate_layout() {
        assert_eq!(AgevakTCMDate::SIZE, 8);
        assert_eq!(AgevakTCMDate::OFFSET_DAY,  0);
        assert_eq!(AgevakTCMDate::OFFSET_YEAR, 2);
        assert_eq!(AgevakTCMDate::OFFSET_LEAP_YEAR, 4);
    }
}
