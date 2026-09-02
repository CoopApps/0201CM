//! Player attribute derivation — displayed vs in-match values.
//!
//! Ported from the agevak/CM0102 project's
//! `CM0102Core/Model/PlayerAttribute.cs` — a reverse-engineered
//! reference for the CM01/02 formulas that turn a player's INTRINSIC
//! stored attribute into their IN-MATCH attribute value.
//!
//! **Critical fidelity finding**: the exe stores INTRINSIC attributes
//! (0..20 range shown in the UI) but the match engine uses DERIVED
//! IN-MATCH values that scale with Current Ability. The two can differ
//! substantially — a Passing=15 player at CA=200 has a DIFFERENT
//! effective passing in a match than the same intrinsic at CA=80.
//!
//! Per the agevak project README:
//! > "Correctly calculated in-match attributes. What the game shows are
//! > cosmetic values, calculated via completely different formulas and
//! > not actually used in matches."
//!
//! Two attribute groups:
//! - **CA-dependent** (18 attrs): value blends intrinsic with (CA/2 + 80),
//!   plus per-attribute exponential-above-threshold bonuses for CA > 300
//!   (the `deservesBonus` gate at scaled-ability > 150).
//! - **CA-independent** (rest): direct scale + offset per attribute.

use serde::{Serialize, Deserialize};

/// The 50 player attributes present in CM01/02. First 42 are TPlayer
/// (physical / technical / mental); last 8 are TStaff (personality).
///
/// Order matches the C# `PlayerAttribute` enum in
/// `agevak/CM0102Core/Model/PlayerAttribute.cs`. That order is chosen
/// alphabetically for consistency and does NOT match the on-disk byte
/// order (which we already have decoded — see
/// [`crate::editor_is_ground_truth`] memory for the on-disk 42-attr
/// layout at type10 +0x1b..+0x44).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerAttribute {
    Acceleration    = 0,
    Aggression      = 1,
    Agility         = 2,
    Anticipation    = 3,
    Balance         = 4,
    Bravery         = 5,
    Consistency     = 6,
    Corners         = 7,
    Crossing        = 8,
    Decisions       = 9,
    Dirtiness       = 10,
    Dribbling       = 11,
    Finishing       = 12,
    Flair           = 13,
    FreeKicks       = 14,
    Handling        = 15,
    Heading         = 16,
    ImportantMatches = 17,
    InjuryProneness = 18,
    Jumping         = 19,
    Leadership      = 20,
    LeftFoot        = 21,
    LongShots       = 22,
    Marking         = 23,
    Movement        = 24,
    NaturalFitness  = 25,
    OneOnOnes       = 26,
    Pace            = 27,
    Passing         = 28,
    Penalties       = 29,
    Positioning     = 30,
    Reflexes        = 31,
    RightFoot       = 32,
    Stamina         = 33,
    Strength        = 34,
    Tackling        = 35,
    Teamwork        = 36,
    Technique       = 37,
    ThrowIns        = 38,
    Versatility     = 39,
    Vision          = 40,
    WorkRate        = 41,
    // TStaff begins.
    Adaptability    = 42,
    Ambition        = 43,
    Determination   = 44,
    Loyalty         = 45,
    Pressure        = 46,
    Professionalism = 47,
    Sportsmanship   = 48,
    Temperament     = 49,
}

/// The 18 attributes whose in-match value depends on Current Ability.
/// VERIFIED via port of the C# `CA_DEPENDENT_ATTRIBUTES` array.
pub const CA_DEPENDENT_ATTRIBUTES: [PlayerAttribute; 18] = [
    PlayerAttribute::Anticipation,
    PlayerAttribute::Crossing,
    PlayerAttribute::Decisions,
    PlayerAttribute::Dribbling,
    PlayerAttribute::Finishing,
    PlayerAttribute::Handling,
    PlayerAttribute::Heading,
    PlayerAttribute::LongShots,
    PlayerAttribute::Marking,
    PlayerAttribute::Movement,
    PlayerAttribute::OneOnOnes,
    PlayerAttribute::Passing,
    PlayerAttribute::Penalties,
    PlayerAttribute::Positioning,
    PlayerAttribute::Reflexes,
    PlayerAttribute::Tackling,
    PlayerAttribute::ThrowIns,
    PlayerAttribute::Vision,
];

impl PlayerAttribute {
    /// True iff the attribute's in-match value depends on Current Ability.
    pub fn is_ca_dependent(self) -> bool {
        CA_DEPENDENT_ATTRIBUTES.contains(&self)
    }

    /// UI label (matches game screen). E.g. `Vision` displays as "Creativity".
    pub fn ui_name(self) -> &'static str {
        match self {
            Self::FreeKicks         => "Set pieces",
            Self::ImportantMatches  => "Imp matches",
            Self::InjuryProneness   => "Injury prone",
            Self::Leadership        => "Influence",
            Self::Movement          => "Off the ball",
            Self::NaturalFitness    => "Nat fitness",
            Self::Vision            => "Creativity",
            _ => match self {
                Self::Acceleration  => "Acceleration",
                Self::Aggression    => "Aggression",
                Self::Agility       => "Agility",
                Self::Anticipation  => "Anticipation",
                Self::Balance       => "Balance",
                Self::Bravery       => "Bravery",
                Self::Consistency   => "Consistency",
                Self::Corners       => "Corners",
                Self::Crossing      => "Crossing",
                Self::Decisions     => "Decisions",
                Self::Dirtiness     => "Dirtiness",
                Self::Dribbling     => "Dribbling",
                Self::Finishing     => "Finishing",
                Self::Flair         => "Flair",
                Self::Handling      => "Handling",
                Self::Heading       => "Heading",
                Self::Jumping       => "Jumping",
                Self::LeftFoot      => "Left foot",
                Self::LongShots     => "Long shots",
                Self::Marking       => "Marking",
                Self::OneOnOnes     => "One on ones",
                Self::Pace          => "Pace",
                Self::Passing       => "Passing",
                Self::Penalties     => "Penalties",
                Self::Positioning   => "Positioning",
                Self::Reflexes      => "Reflexes",
                Self::RightFoot     => "Right foot",
                Self::Stamina       => "Stamina",
                Self::Strength      => "Strength",
                Self::Tackling      => "Tackling",
                Self::Teamwork      => "Teamwork",
                Self::Technique     => "Technique",
                Self::ThrowIns      => "Throw ins",
                Self::Versatility   => "Versatility",
                Self::WorkRate      => "Work rate",
                Self::Adaptability  => "Adaptability",
                Self::Ambition      => "Ambition",
                Self::Determination => "Determination",
                Self::Loyalty       => "Loyalty",
                Self::Pressure      => "Pressure",
                Self::Professionalism => "Professionalism",
                Self::Sportsmanship => "Sportsmanship",
                Self::Temperament   => "Temperament",
                _ => "?", // unreachable
            }
        }
    }
}

/// Apply an exponential-above-threshold bonus. Used by CA-dependent
/// attribute derivation when the scaled-ability > 150 gate fires.
///
/// - Below `threshold`: unchanged
/// - Above `threshold`: `threshold + (v - threshold).powf(power)`
///
/// Ported byte-for-byte from `ApplyExpBonusToInMatchValue`.
#[inline]
pub fn apply_exp_bonus(deserves: bool, val: f32, threshold: f32, power: f32) -> f32 {
    if deserves && val > threshold {
        threshold + (val - threshold).powf(power)
    } else {
        val
    }
}

/// Optional per-player context for the attribute derivation formulas.
/// Only Movement and Vision use these fields; supply `None` for the
/// rest to get baseline behavior.
#[derive(Debug, Clone, Copy, Default)]
pub struct AttrCtx {
    /// Player's year-of-birth offset (see agevak `staff.YearOfBirth > 36`).
    /// The `> 36` gate scales Movement intrinsic × 0.75 for older players.
    pub year_of_birth_gt_36: bool,
    /// Player's DefensiveMidfielder aptitude — if exactly 20 (specialist),
    /// scales Movement intrinsic × 0.85. (agevak: `player.DefensiveMidfielder == 20`)
    pub defensive_mid_20: bool,
}

/// Compute the in-match value for a CA-dependent attribute.
///
/// Direct port of `GetCaDepInMatchValue`. Steps:
/// 1. Scale ability: `ability = (ability / 2 + 80) as u8`
/// 2. Base blend: `v = (2 * intrinsic + ability) * 0.1`
/// 3. `deservesBonus = ability > 150`
/// 4. Per-attribute switch applying the specific formula (exponential
///    bonuses, adjustments for age/DM/creativity intrinsic).
///
/// Returns the in-match value (typically 0..~40 for elite CA).
pub fn ca_dependent_in_match(
    attr: PlayerAttribute,
    intrinsic: i8,
    ability: i32,
    ctx: AttrCtx,
) -> f32 {
    let original_ability = ability;
    // Scale to 80..180 range with u8 wrap (matches C# `(byte)(ability/2 + 80)`).
    let ability = ((ability / 2 + 80) & 0xFF) as f32;
    let mut intrinsic_f = intrinsic as f32;
    let mut v = (2.0 * intrinsic_f + ability) * 0.1;
    let deserves = ability > 150.0;

    use PlayerAttribute as A;
    match attr {
        A::Anticipation => {
            v = ((v + 0.23).max(0.0)) / 2.0 + 5.0;
        }
        A::Crossing | A::Finishing => {
            v = apply_exp_bonus(deserves, v, 16.0, 1.25);
        }
        A::Heading => {
            v = apply_exp_bonus(deserves, v, 18.0, 1.2);
        }
        A::Marking | A::Positioning => {
            v = apply_exp_bonus(deserves, v, 16.0, 1.2);
        }
        A::Movement => {
            // Age/DM adjustments before the blend
            if ctx.year_of_birth_gt_36 {
                intrinsic_f *= 0.75;
            } else if ctx.defensive_mid_20 {
                intrinsic_f *= 0.85;
            }
            v = (2.0 * intrinsic_f + ability) * 0.1;
            v = (v + 0.13) * 0.65 + 7.0;
        }
        A::Passing => {
            v = apply_exp_bonus(deserves, v, 18.0, 1.25);
        }
        A::Vision => {
            // "Creativity" derived: intrinsic > 10 gets scaled DOWN for
            // low-CA players (a low-CA "flair" player is less effective).
            if intrinsic > 10 {
                if original_ability < 80 {
                    intrinsic_f = 10.0 + (intrinsic - 10) as f32 * 0.75;
                } else if original_ability < 125 {
                    intrinsic_f = 10.0 + (intrinsic - 10) as f32 * 0.85;
                }
            }
            v = (2.0 * intrinsic_f + ability) * 0.1;
            v = apply_exp_bonus(deserves, v, 17.0, 1.2);
        }
        _ => {
            // Other CA-dependent attributes (Decisions/Dribbling/Handling/
            // LongShots/OneOnOnes/Penalties/Reflexes/Tackling/ThrowIns) use
            // the plain base blend with no bonus adjustment.
        }
    }
    v.max(0.0)
}

/// Compute the in-match value for a NON-CA-dependent attribute.
///
/// Direct port of `GetNonCaDepInMatchValue`. Simple scale + offset per
/// attribute; only Acceleration and Pace get the exponential bonus
/// (unconditionally — `deservesBonus=true` for these two).
pub fn non_ca_dependent_in_match(
    attr: PlayerAttribute,
    intrinsic: i8,
) -> f32 {
    let mut v = intrinsic as f32;
    use PlayerAttribute as A;
    match attr {
        A::Acceleration | A::Pace => {
            v = (intrinsic as f32) * (2.0 / 3.0) + 0.2;
            v = apply_exp_bonus(true, v, 12.0, 1.25);
        }
        A::Agility | A::Balance | A::Bravery | A::Corners | A::Strength => {
            v = (intrinsic as f32) * 0.5 + 0.2;
        }
        A::Aggression => {
            v = (intrinsic as f32) * 0.5;
        }
        A::FreeKicks => {
            v = (intrinsic as f32) * 0.57 + 0.2;
        }
        A::Stamina => {
            v = (intrinsic as f32) * 0.65 + 7.25;
        }
        A::Flair | A::Jumping | A::Teamwork | A::Technique | A::Versatility => {
            v = intrinsic as f32;
        }
        _ => {
            // Others (Consistency, Dirtiness, ImportantMatches, InjuryProneness,
            // Leadership, LeftFoot/RightFoot, NaturalFitness, WorkRate,
            // TStaff attrs) — leave unchanged from intrinsic.
        }
    }
    v.max(0.0)
}

/// Top-level dispatcher: compute in-match value for any attribute.
/// Chooses CA-dependent or CA-independent path based on the enum.
pub fn in_match_value(
    attr: PlayerAttribute,
    intrinsic: i8,
    ability: i32,
    ctx: AttrCtx,
) -> f32 {
    if attr.is_ca_dependent() {
        ca_dependent_in_match(attr, intrinsic, ability, ctx)
    } else {
        non_ca_dependent_in_match(attr, intrinsic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ca_dependent_set_matches_agevak() {
        // 18 CA-dependent attributes per the agevak project
        assert_eq!(CA_DEPENDENT_ATTRIBUTES.len(), 18);
        assert!(PlayerAttribute::Passing.is_ca_dependent());
        assert!(PlayerAttribute::Marking.is_ca_dependent());
        assert!(PlayerAttribute::Vision.is_ca_dependent());
        // These are NOT CA-dependent (they're physical / mental)
        assert!(!PlayerAttribute::Pace.is_ca_dependent());
        assert!(!PlayerAttribute::Strength.is_ca_dependent());
        assert!(!PlayerAttribute::Determination.is_ca_dependent());
    }

    #[test]
    fn ui_name_matches_game_screen() {
        // The ~7 attributes with non-default UI names
        assert_eq!(PlayerAttribute::FreeKicks.ui_name(),        "Set pieces");
        assert_eq!(PlayerAttribute::ImportantMatches.ui_name(), "Imp matches");
        assert_eq!(PlayerAttribute::InjuryProneness.ui_name(),  "Injury prone");
        assert_eq!(PlayerAttribute::Leadership.ui_name(),       "Influence");
        assert_eq!(PlayerAttribute::Movement.ui_name(),         "Off the ball");
        assert_eq!(PlayerAttribute::NaturalFitness.ui_name(),   "Nat fitness");
        assert_eq!(PlayerAttribute::Vision.ui_name(),           "Creativity");
    }

    #[test]
    fn passing_in_match_scales_with_ca() {
        // Passing at intrinsic 15, CA 100 vs CA 200
        let ctx = AttrCtx::default();
        let low_ca  = in_match_value(PlayerAttribute::Passing, 15, 100, ctx);
        let high_ca = in_match_value(PlayerAttribute::Passing, 15, 200, ctx);
        // High CA gives strictly higher in-match value
        assert!(high_ca > low_ca,
                "high_ca={} vs low_ca={}", high_ca, low_ca);
        // Both should be well above intrinsic (blend with ability push)
        assert!(low_ca > 10.0);
    }

    #[test]
    fn pace_gets_exponential_bonus_above_threshold() {
        // Pace = 15 intrinsic → base 10.2, +exp bonus (always fires for Pace)
        let low = non_ca_dependent_in_match(PlayerAttribute::Pace, 10);
        let high = non_ca_dependent_in_match(PlayerAttribute::Pace, 20);
        // Pace 10 → 6.87, threshold 12 → no bonus
        assert!(low < 12.0);
        // Pace 20 → 13.53, above threshold 12 → exp bonus applied
        assert!(high > 12.0);
        // Bonus formula: 12 + (v - 12)^1.25, so high value should exceed
        // 12 by less than the raw exceed (power<1... wait, 1.25 is >1
        // meaning the bonus AMPLIFIES). Actually 1.25>1 means bigger.
    }

    #[test]
    fn stamina_uses_specific_scale_and_offset() {
        // Stamina: v = intrinsic * 0.65 + 7.25
        assert!((non_ca_dependent_in_match(PlayerAttribute::Stamina, 10) - 13.75).abs() < 0.01);
        assert!((non_ca_dependent_in_match(PlayerAttribute::Stamina, 20) - 20.25).abs() < 0.01);
    }

    #[test]
    fn agility_uses_half_plus_02() {
        // v = intrinsic * 0.5 + 0.2
        assert!((non_ca_dependent_in_match(PlayerAttribute::Agility, 10) - 5.2).abs() < 0.01);
        assert!((non_ca_dependent_in_match(PlayerAttribute::Agility, 20) - 10.2).abs() < 0.01);
    }

    #[test]
    fn movement_age_penalty_matches_agevak() {
        // Movement with year_of_birth_gt_36 → intrinsic scaled × 0.75
        let young = in_match_value(PlayerAttribute::Movement, 20, 150,
            AttrCtx { year_of_birth_gt_36: false, defensive_mid_20: false });
        let old = in_match_value(PlayerAttribute::Movement, 20, 150,
            AttrCtx { year_of_birth_gt_36: true, defensive_mid_20: false });
        assert!(old < young, "old={} vs young={}", old, young);
    }

    #[test]
    fn vision_creativity_low_ca_scaling() {
        // High-intrinsic Vision scales DOWN for low-CA players
        let ctx = AttrCtx::default();
        let low_ca_high_vision  = in_match_value(PlayerAttribute::Vision, 20, 79, ctx);
        let mid_ca_high_vision  = in_match_value(PlayerAttribute::Vision, 20, 150, ctx);
        assert!(mid_ca_high_vision > low_ca_high_vision,
                "mid={} vs low={}", mid_ca_high_vision, low_ca_high_vision);
    }
}
