//! Enums + gate formulas translated from agevak/CM0102/CM0102Core/Model/*.
//!
//! Companion to [`crate::agevak_reference`]. Sources:
//! - `Model/PlayerPosition.cs` — 11-variant position enum + proper-fit gate
//! - `Model/AttributeValueType.cs` — attribute reveal-mode
//! - `Model/SquadStatus.cs` — 9-variant status (community naming)
//! - `Model/TransferStatus.cs` — 4-variant transfer flag
//! - `Model/Range.cs` — inclusive integer/double range
//!
//! Where agevak's naming disagrees with our existing port (e.g.
//! `crate::transfer::SquadStatus` uses "KeyPlayer/FirstTeam/...",
//! agevak uses "Uncertain/Indispensable/..."), this module keeps the
//! agevak names so both mappings are visible side-by-side.

use serde::{Serialize, Deserialize};

// ---------------------------------------------------------------------------
// PlayerPosition (11 variants)
// ---------------------------------------------------------------------------

/// The 11 pitch positions agevak recognises, indexed 0..=10.
///
/// Enum order MUST NOT change — a couple of methods depend on the
/// discriminant modulo 2 (winger detection, UI-name /R suffix).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerPosition {
    GK  = 0,
    DC  = 1,
    DL  = 2,
    DMC = 3,
    DML = 4,
    MC  = 5,
    ML  = 6,
    AMC = 7,
    AML = 8,
    FC  = 9,
    FL  = 10,
}

/// Per-position aptitude thresholds the agevak project uses to decide
/// whether a player is a "proper" fit for the slot. Verified against
/// `PlayerPositionExtensions.IsPlayerProperForPosition`.
///
/// Every input is the raw aptitude byte (0..=20). Numbers are:
/// - GK: Goalkeeper ≥ 20 (i.e. natural only)
/// - all other position groups: 15 in the group aptitude AND 15 in
///   the sidedness aptitude (Central for centre slots; max(Left,
///   Right) for wide slots).
///
/// The two-of-two structure is why wide slots accept a WingBack
/// alternative and defensive-mid slots accept a DefensiveMidfielder
/// OR WingBack alternative.
pub fn is_player_proper_for_position(
    pos: PlayerPosition,
    goalkeeper: i8, sweeper: i8, defender: i8,
    defensive_mid: i8, midfielder: i8, attacking_mid: i8,
    attacker: i8, wing_back: i8,
    left_side: i8, right_side: i8, central: i8,
) -> bool {
    let side = left_side.max(right_side);
    match pos {
        PlayerPosition::GK  => goalkeeper >= 20,
        PlayerPosition::DC  => (sweeper >= 15 || defender >= 15) && central >= 15,
        PlayerPosition::DL  => (defender >= 15 || wing_back >= 15) && side >= 15,
        PlayerPosition::DMC => defensive_mid >= 15 && central >= 15,
        PlayerPosition::DML => (defensive_mid >= 15 || wing_back >= 15) && side >= 15,
        PlayerPosition::MC  => midfielder >= 15 && central >= 15,
        PlayerPosition::ML  => midfielder >= 15 && side >= 15,
        PlayerPosition::AMC => attacking_mid >= 15 && central >= 15,
        PlayerPosition::AML => (attacking_mid >= 15 || wing_back >= 15) && side >= 15,
        PlayerPosition::FC  => attacker >= 15 && central >= 15,
        PlayerPosition::FL  => attacker >= 15 && side >= 15,
    }
}

impl PlayerPosition {
    /// True for slots that are essentially wide roles (DL/DML/ML/AML/FL).
    /// Encoded exactly as agevak: not GK and `(discriminant - 1) % 2 == 1`.
    pub fn is_winger(self) -> bool {
        self != PlayerPosition::GK && ((self as u8 as i32) - 1) % 2 == 1
    }

    /// UI label — appends "/R" to any wide slot (agevak's convention:
    /// left-only labels are shown as e.g. "DL/R" meaning left-or-right).
    pub fn ui_name(self) -> String {
        let s = format!("{:?}", self);
        if self != PlayerPosition::GK && (self as u8 as i32) % 2 == 0 {
            format!("{s}/R")
        } else {
            s
        }
    }
}

// ---------------------------------------------------------------------------
// AttributeValueType (attribute-reveal mode)
// ---------------------------------------------------------------------------

/// How an attribute value should be surfaced to a UI.
///
/// - `Normalized` — divide by the current-CA cap
/// - `PotentialNormalized` — divide by the PA cap
/// - `Intrinsic` — the raw byte, no scaling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeValueType {
    Normalized,
    PotentialNormalized,
    Intrinsic,
}

// ---------------------------------------------------------------------------
// SquadStatus — the agevak-community naming (9 variants).
// ---------------------------------------------------------------------------

/// **Different from [`crate::transfer::SquadStatus`]** — this is the
/// naming the agevak/CM0102 community uses. Our port carries a
/// 7-variant version (KeyPlayer/FirstTeam/FirstTeamSquad/…) that seems
/// to reflect the exe strings; both target the same underlying byte
/// but tag it differently. Cross-reference before mixing the two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AgevakSquadStatus {
    Uncertain      = 0,
    Indispensable  = 1,
    FirstTeam      = 2,
    SquadRotation  = 3,
    Backup         = 4,
    HotProspect    = 5,
    DecentYoung    = 6,
    NotNeeded      = 7,
    OnTrial        = 8,
}

// ---------------------------------------------------------------------------
// TransferStatus (4 variants)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AgevakTransferStatus {
    Unknown         = 0,
    ListedByClub    = 1,
    ListedByRequest = 2,
    ListedForLoan   = 3,
}

// ---------------------------------------------------------------------------
// Range<i32> and Range<f64> with inclusive-with-epsilon match.
// ---------------------------------------------------------------------------

/// Inclusive integer range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntRange { pub lower: i32, pub upper: i32 }

impl IntRange {
    pub const fn new(lower: i32, upper: i32) -> Self { Self { lower, upper } }
    pub const fn matches(&self, v: i32) -> bool { self.lower <= v && v <= self.upper }
    pub fn matches_f64(&self, v: f64) -> bool {
        (self.lower as f64) - 1e-9 <= v && v <= (self.upper as f64) + 1e-9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gk_needs_natural_20() {
        // Only 20 in Goalkeeper passes; 19 fails.
        assert!(is_player_proper_for_position(
            PlayerPosition::GK, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ));
        assert!(!is_player_proper_for_position(
            PlayerPosition::GK, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ));
    }

    #[test]
    fn dc_needs_15_in_sweeper_or_defender_and_15_central() {
        // 15 defender + 15 central → pass
        assert!(is_player_proper_for_position(
            PlayerPosition::DC, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 15
        ));
        // 15 sweeper + 15 central → pass
        assert!(is_player_proper_for_position(
            PlayerPosition::DC, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 15
        ));
        // 14 both aptitudes → fail
        assert!(!is_player_proper_for_position(
            PlayerPosition::DC, 0, 14, 14, 0, 0, 0, 0, 0, 0, 0, 15
        ));
        // 15 defender but only 14 central → fail
        assert!(!is_player_proper_for_position(
            PlayerPosition::DC, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 14
        ));
    }

    #[test]
    fn dl_accepts_wingback_alternative() {
        // 15 wing_back + 15 left → pass (defender=0)
        assert!(is_player_proper_for_position(
            PlayerPosition::DL, 0, 0, 0, 0, 0, 0, 0, 15, 15, 0, 0
        ));
    }

    #[test]
    fn wingers_detected_correctly() {
        // Wide slots: DL(2), DML(4), ML(6), AML(8), FL(10).
        // Discriminant even and ≠ 0 → is_winger true.
        assert!(!PlayerPosition::GK.is_winger());
        assert!(!PlayerPosition::DC.is_winger());
        assert!(PlayerPosition::DL.is_winger());
        assert!(!PlayerPosition::DMC.is_winger());
        assert!(PlayerPosition::DML.is_winger());
        assert!(!PlayerPosition::MC.is_winger());
        assert!(PlayerPosition::ML.is_winger());
        assert!(PlayerPosition::AML.is_winger());
        assert!(PlayerPosition::FL.is_winger());
    }

    #[test]
    fn ui_name_appends_r_to_wide_slots() {
        assert_eq!(PlayerPosition::GK.ui_name(),  "GK");
        assert_eq!(PlayerPosition::DC.ui_name(),  "DC");
        assert_eq!(PlayerPosition::DL.ui_name(),  "DL/R");
        assert_eq!(PlayerPosition::DMC.ui_name(), "DMC");
        assert_eq!(PlayerPosition::DML.ui_name(), "DML/R");
        assert_eq!(PlayerPosition::FL.ui_name(),  "FL/R");
    }

    #[test]
    fn range_matches_endpoints() {
        let r = IntRange::new(5, 10);
        assert!(!r.matches(4));
        assert!(r.matches(5));
        assert!(r.matches(10));
        assert!(!r.matches(11));
        assert!(r.matches_f64(5.0));
        assert!(r.matches_f64(10.0));
        assert!(r.matches_f64(4.999999999));   // within 1e-9 epsilon
        assert!(r.matches_f64(10.000000001));  // within 1e-9 epsilon
    }
}
