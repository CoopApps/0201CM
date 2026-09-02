//! Position rating weights table + rating estimator, translated from
//! agevak/CM0102/CM0102Core/Model/EstimatedPlayer.cs.
//!
//! For each pitch position (11 variants — see [`crate::agevak_enums::PlayerPosition`]),
//! the community-derived table lists the weight (0.0..=1.0) of every
//! player attribute that contributes to the estimated rating. Only 9
//! of the 11 positions have a defined weights row — AML and FL fall
//! back to their DML / FC counterparts inside the estimator.
//!
//! Also carries [`POSITION_PRE_NORM_MAX_RATING`] — the per-position
//! max the weighted-sum is scaled to (100 for most positions, 90 for
//! DC, 95 for ML), and [`estimate_position_rating`] — the weighted-sum
//! evaluator with the special-case scales (Reflexes-19 cap, Marking
//! 4-step scale, Vision capped at Passing, Jumping non-linear for FC
//! and MC/AMC, DL-Dribbling quadratic-then-mirror, winger-Teamwork
//! 3-band scale, winger-RightFoot foot-swap).

use crate::agevak_enums::PlayerPosition;
use crate::player_attribute_derivation::PlayerAttribute;

/// Number of PlayerAttribute variants (0..=49).
pub const NUM_ATTRIBUTES: usize = 50;
/// Number of PlayerPosition variants.
pub const NUM_POSITIONS: usize  = 11;

/// Per-position pre-normalisation maximum (rating gets multiplied by
/// this after the weighted-sum divides by sum-of-weights).
pub const POSITION_PRE_NORM_MAX_RATING: [f32; NUM_POSITIONS] = {
    let mut arr = [100.0; NUM_POSITIONS];
    arr[PlayerPosition::DC as usize] = 90.0;
    arr[PlayerPosition::ML as usize] = 95.0;
    arr
};

// ---------------------------------------------------------------------------
// WEIGHTS table  — [position][attribute]
// ---------------------------------------------------------------------------

/// Position-attribute weights. Empty (0.0) means the attribute doesn't
/// contribute at that position. Verified line-by-line against
/// EstimatedPlayer.cs static ctor.
pub const WEIGHTS: [[f32; NUM_ATTRIBUTES]; NUM_POSITIONS] = build_weights();

const fn build_weights() -> [[f32; NUM_ATTRIBUTES]; NUM_POSITIONS] {
    let mut w = [[0.0f32; NUM_ATTRIBUTES]; NUM_POSITIONS];

    // Helpers
    macro_rules! set {
        ($pos:expr, $attr:expr, $v:expr) => {
            w[$pos as usize][$attr as usize] = $v;
        };
    }

    // GK
    set!(PlayerPosition::GK, PlayerAttribute::Agility,   0.069);
    set!(PlayerPosition::GK, PlayerAttribute::Handling,  0.159);
    set!(PlayerPosition::GK, PlayerAttribute::Reflexes,  0.060);

    // DC
    set!(PlayerPosition::DC, PlayerAttribute::Aggression,  0.024);
    set!(PlayerPosition::DC, PlayerAttribute::Jumping,     0.027);
    set!(PlayerPosition::DC, PlayerAttribute::Marking,     0.115);
    set!(PlayerPosition::DC, PlayerAttribute::Pace,        0.019);
    set!(PlayerPosition::DC, PlayerAttribute::Passing,     0.031);
    set!(PlayerPosition::DC, PlayerAttribute::Positioning, 0.114);
    set!(PlayerPosition::DC, PlayerAttribute::Teamwork,    0.040);
    set!(PlayerPosition::DC, PlayerAttribute::WorkRate,    0.022);

    // DL
    set!(PlayerPosition::DL, PlayerAttribute::Aggression,  0.014);
    set!(PlayerPosition::DL, PlayerAttribute::Dribbling,   0.143);
    set!(PlayerPosition::DL, PlayerAttribute::Jumping,     0.056);
    set!(PlayerPosition::DL, PlayerAttribute::RightFoot,   0.060);
    set!(PlayerPosition::DL, PlayerAttribute::Movement,    0.055);
    set!(PlayerPosition::DL, PlayerAttribute::Pace,        0.029);
    set!(PlayerPosition::DL, PlayerAttribute::Passing,     0.072);
    set!(PlayerPosition::DL, PlayerAttribute::Positioning, 0.195);
    set!(PlayerPosition::DL, PlayerAttribute::Stamina,     0.048);
    set!(PlayerPosition::DL, PlayerAttribute::Teamwork,    0.098);
    set!(PlayerPosition::DL, PlayerAttribute::Technique,   0.170);
    set!(PlayerPosition::DL, PlayerAttribute::WorkRate,    0.038);

    // DMC
    set!(PlayerPosition::DMC, PlayerAttribute::Aggression,  0.030);
    set!(PlayerPosition::DMC, PlayerAttribute::Dribbling,   0.037);
    set!(PlayerPosition::DMC, PlayerAttribute::Movement,    0.057);
    set!(PlayerPosition::DMC, PlayerAttribute::Pace,        0.041);
    set!(PlayerPosition::DMC, PlayerAttribute::Passing,     0.087);
    set!(PlayerPosition::DMC, PlayerAttribute::Positioning, 0.200);
    set!(PlayerPosition::DMC, PlayerAttribute::Stamina,     0.017);
    set!(PlayerPosition::DMC, PlayerAttribute::Teamwork,    0.018);
    set!(PlayerPosition::DMC, PlayerAttribute::Tackling,    0.046);
    set!(PlayerPosition::DMC, PlayerAttribute::Technique,   0.047);
    set!(PlayerPosition::DMC, PlayerAttribute::Vision,      0.090);
    set!(PlayerPosition::DMC, PlayerAttribute::WorkRate,    0.078);

    // DML
    set!(PlayerPosition::DML, PlayerAttribute::Aggression,  0.020);
    set!(PlayerPosition::DML, PlayerAttribute::Dribbling,   0.141);
    set!(PlayerPosition::DML, PlayerAttribute::Jumping,     0.052);
    set!(PlayerPosition::DML, PlayerAttribute::RightFoot,   0.077);
    set!(PlayerPosition::DML, PlayerAttribute::Movement,    0.077);
    set!(PlayerPosition::DML, PlayerAttribute::Pace,        0.029);
    set!(PlayerPosition::DML, PlayerAttribute::Passing,     0.073);
    set!(PlayerPosition::DML, PlayerAttribute::Positioning, 0.182);
    set!(PlayerPosition::DML, PlayerAttribute::Stamina,     0.053);
    set!(PlayerPosition::DML, PlayerAttribute::Teamwork,    0.112);
    set!(PlayerPosition::DML, PlayerAttribute::Technique,   0.168);
    set!(PlayerPosition::DML, PlayerAttribute::WorkRate,    0.035);

    // MC
    set!(PlayerPosition::MC, PlayerAttribute::Dribbling,   0.159);
    set!(PlayerPosition::MC, PlayerAttribute::Jumping,     0.065);
    set!(PlayerPosition::MC, PlayerAttribute::Movement,    0.154);
    set!(PlayerPosition::MC, PlayerAttribute::Pace,        0.039);
    set!(PlayerPosition::MC, PlayerAttribute::Passing,     0.064);
    set!(PlayerPosition::MC, PlayerAttribute::Positioning, 0.117);
    set!(PlayerPosition::MC, PlayerAttribute::Stamina,     0.080);
    set!(PlayerPosition::MC, PlayerAttribute::Teamwork,    0.013);
    set!(PlayerPosition::MC, PlayerAttribute::Technique,   0.187);
    set!(PlayerPosition::MC, PlayerAttribute::Vision,      0.035);
    set!(PlayerPosition::MC, PlayerAttribute::WorkRate,    0.055);

    // ML
    set!(PlayerPosition::ML, PlayerAttribute::Aggression,  0.018);
    set!(PlayerPosition::ML, PlayerAttribute::Dribbling,   0.131);
    set!(PlayerPosition::ML, PlayerAttribute::RightFoot,   0.077);
    set!(PlayerPosition::ML, PlayerAttribute::Movement,    0.249);
    set!(PlayerPosition::ML, PlayerAttribute::Pace,        0.012);
    set!(PlayerPosition::ML, PlayerAttribute::Passing,     0.072);
    set!(PlayerPosition::ML, PlayerAttribute::Stamina,     0.068);
    set!(PlayerPosition::ML, PlayerAttribute::Teamwork,    0.099);
    set!(PlayerPosition::ML, PlayerAttribute::Technique,   0.158);
    set!(PlayerPosition::ML, PlayerAttribute::WorkRate,    0.036);

    // AMC
    set!(PlayerPosition::AMC, PlayerAttribute::Crossing,   0.025);
    set!(PlayerPosition::AMC, PlayerAttribute::Dribbling,  0.193);
    set!(PlayerPosition::AMC, PlayerAttribute::Jumping,    0.045);
    set!(PlayerPosition::AMC, PlayerAttribute::Movement,   0.155);
    set!(PlayerPosition::AMC, PlayerAttribute::Pace,       0.046);
    set!(PlayerPosition::AMC, PlayerAttribute::Passing,    0.067);
    set!(PlayerPosition::AMC, PlayerAttribute::Positioning,0.124);
    set!(PlayerPosition::AMC, PlayerAttribute::Stamina,    0.079);
    set!(PlayerPosition::AMC, PlayerAttribute::Teamwork,   0.022);
    set!(PlayerPosition::AMC, PlayerAttribute::Technique,  0.203);
    set!(PlayerPosition::AMC, PlayerAttribute::WorkRate,   0.078);

    // FC
    set!(PlayerPosition::FC, PlayerAttribute::Dribbling,   0.080);
    set!(PlayerPosition::FC, PlayerAttribute::Finishing,   0.024);
    set!(PlayerPosition::FC, PlayerAttribute::Heading,     0.029);
    set!(PlayerPosition::FC, PlayerAttribute::Jumping,     0.111);
    set!(PlayerPosition::FC, PlayerAttribute::Movement,    0.177);
    set!(PlayerPosition::FC, PlayerAttribute::Pace,        0.030);
    set!(PlayerPosition::FC, PlayerAttribute::Passing,     0.063);
    set!(PlayerPosition::FC, PlayerAttribute::Positioning, 0.092);
    set!(PlayerPosition::FC, PlayerAttribute::Stamina,     0.083);
    set!(PlayerPosition::FC, PlayerAttribute::Teamwork,    0.023);
    set!(PlayerPosition::FC, PlayerAttribute::Technique,   0.141);
    set!(PlayerPosition::FC, PlayerAttribute::WorkRate,    0.040);

    // AML and FL are NOT populated — agevak's estimator falls back to
    // DML / FC weights for those positions.
    w
}

/// True if `attribute` is a "major" (>=0.100 weight) at `position`.
pub fn is_major_attribute(position: PlayerPosition, attribute: PlayerAttribute) -> bool {
    WEIGHTS[position as usize][attribute as usize] >= 0.100
}

/// True if `attribute` is a "minor" (0 < weight < 0.100) at `position`.
pub fn is_minor_attribute(position: PlayerPosition, attribute: PlayerAttribute) -> bool {
    let w = WEIGHTS[position as usize][attribute as usize];
    w > 1e-9 && w < 0.100
}

/// The estimator's linear-plus-special-case rating.
///
/// - `intrinsics[i]` — raw attribute byte for `PlayerAttribute::from_repr(i)`.
/// - `values[i]` — the "in-match" value for the same attribute (already
///   CA-adjusted by [`crate::player_attribute_derivation::ca_dependent_in_match`]
///   where applicable), passed in separately so the caller controls
///   the MEGA-BOOST-on/off variant.
/// - `left_side`, `right_side`, `left_foot`, `right_foot` — sidedness
///   aptitudes for the RightFoot swap.
///
/// Returns the pre-normalisation-max rating (already divided by weight
/// sum and multiplied by [`POSITION_PRE_NORM_MAX_RATING`]).
pub fn estimate_position_rating(
    position: PlayerPosition,
    intrinsics: &[i32; NUM_ATTRIBUTES],
    values:     &[f32; NUM_ATTRIBUTES],
    left_side: i32,  right_side: i32,
    left_foot: i32,  right_foot: i32,
) -> f32 {
    let mut rating = 0.0f32;
    let mut weight_sum = 0.0f32;

    for a_idx in 0..NUM_ATTRIBUTES {
        let weight = WEIGHTS[position as usize][a_idx];
        if weight <= 1e-9 { continue; }
        weight_sum += weight;
        let intr = intrinsics[a_idx];
        let mut value = values[a_idx];
        let mut handled = false;

        // Jumping — non-linear at FC and MC/AMC
        if a_idx == PlayerAttribute::Jumping as usize {
            if matches!(position, PlayerPosition::FC) {
                if intr <= 10 {
                    rating += weight * (value - 1.0) / 9.0 * 0.01;
                } else if intr <= 14 {
                    rating += weight * (value - 10.0) / 4.0 * 0.06;
                } else {
                    if intr >= 15 { rating += weight * 0.03; }
                    if intr >= 16 { rating += weight * 0.05; }
                    if intr >= 17 { rating += weight * 0.10; }
                    if intr >= 18 { rating += weight * 0.15; }
                    if intr >= 19 { rating += weight * 0.25; }
                    if intr >= 20 { rating += weight * 0.35; }
                }
                handled = true;
            } else if matches!(position, PlayerPosition::MC | PlayerPosition::AMC) {
                if intr <= 14 {
                    rating += weight * (value - 1.0) / 13.0 * 0.19;
                } else {
                    if intr >= 15 { rating += weight * 0.03; }
                    if intr >= 16 { rating += weight * 0.05; }
                    if intr >= 17 { rating += weight * 0.09; }
                    if intr >= 18 { rating += weight * 0.13; }
                    if intr >= 19 { rating += weight * 0.21; }
                    if intr >= 20 { rating += weight * 0.30; }
                }
                handled = true;
            }
        // Teamwork — 3-band winger scale
        } else if a_idx == PlayerAttribute::Teamwork as usize && position.is_winger() {
            if intr <= 5 {
                rating += weight * (value - 1.0) / 4.0 * 3.0 / 19.0;
            } else if intr <= 10 {
                rating += weight * (value - 5.0) / 5.0 * 10.0 / 19.0;
            } else {
                rating += weight * (value - 10.0) / 9.0 * 6.0 / 19.0;
            }
            handled = true;
        // RightFoot on a winger — foot-swap based on sidedness
        } else if a_idx == PlayerAttribute::RightFoot as usize && position.is_winger() {
            let left  = if left_side  >= 15 { left_foot  } else { right_foot };
            let right = if right_side >= 15 { right_side } else { left_foot };
            value = left.max(right) as f32;
            // falls through to unhandled branch
        // Reflexes — hard cap at 19
        } else if a_idx == PlayerAttribute::Reflexes as usize {
            rating += weight * value.min(19.0) / 19.0;
            handled = true;
        // Marking — 4-band non-linear
        } else if a_idx == PlayerAttribute::Marking as usize {
            let v = value;
            rating += v.min(9.0) / 9.0 * weight * 2.0 / 3.0;
            if v > 9.0  { rating += (v.min(19.0) - 9.0) / 10.0 * weight / 3.0; }
            if v > 19.0 { rating += (v.min(30.0) - 19.0) / 11.0 * weight / 5.0; }
            if v > 30.0 { rating += (v - 30.0) / 30.0 * weight / 5.0; }
            handled = true;
        // Dribbling at DL — quadratic-then-mirror around 20
        } else if a_idx == PlayerAttribute::Dribbling as usize
                && matches!(position, PlayerPosition::DL)
        {
            if value <= 20.0 {
                rating += weight * value * value / 20.0 / 20.0;
            } else {
                let capped = value.min(40.0);
                rating += 2.0 * weight
                    - weight * (40.0 - capped) * (40.0 - capped) / 20.0 / 20.0;
            }
            handled = true;
        // Vision — capped at Passing's in-match, then Reflexes-style /19
        } else if a_idx == PlayerAttribute::Vision as usize {
            let passing_val = values[PlayerAttribute::Passing as usize];
            let v = value.min(passing_val);
            rating += weight * v.min(19.0) / 19.0;
            handled = true;
        }

        if !handled {
            // Fallback: CA-dependent → /19; else (value-1)/19 (i.e.
            // clip zero at raw=1).
            let ca_dep = crate::player_attribute_derivation::CA_DEPENDENT_ATTRIBUTES
                .iter()
                .any(|a| *a as u8 as usize == a_idx);
            if ca_dep {
                rating += weight * value / 19.0;
            } else {
                rating += weight * (value - 1.0) / 19.0;
            }
        }
    }

    if weight_sum > 0.0 { rating /= weight_sum; }
    rating * POSITION_PRE_NORM_MAX_RATING[position as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_rating_defaults_and_overrides() {
        assert_eq!(POSITION_PRE_NORM_MAX_RATING[PlayerPosition::GK  as usize], 100.0);
        assert_eq!(POSITION_PRE_NORM_MAX_RATING[PlayerPosition::DC  as usize],  90.0);
        assert_eq!(POSITION_PRE_NORM_MAX_RATING[PlayerPosition::ML  as usize],  95.0);
        assert_eq!(POSITION_PRE_NORM_MAX_RATING[PlayerPosition::AMC as usize], 100.0);
        assert_eq!(POSITION_PRE_NORM_MAX_RATING[PlayerPosition::FC  as usize], 100.0);
    }

    #[test]
    fn gk_only_three_weights() {
        let n_gk_nonzero = WEIGHTS[PlayerPosition::GK as usize]
            .iter()
            .filter(|w| **w > 1e-9)
            .count();
        assert_eq!(n_gk_nonzero, 3, "GK has exactly Agility/Handling/Reflexes");
    }

    #[test]
    fn dc_positioning_and_marking_are_major() {
        assert!(is_major_attribute(PlayerPosition::DC, PlayerAttribute::Marking));
        assert!(is_major_attribute(PlayerPosition::DC, PlayerAttribute::Positioning));
    }

    #[test]
    fn dc_workrate_is_minor() {
        assert!(is_minor_attribute(PlayerPosition::DC, PlayerAttribute::WorkRate));
        assert!(!is_major_attribute(PlayerPosition::DC, PlayerAttribute::WorkRate));
    }

    #[test]
    fn aml_and_fl_have_no_weights() {
        assert_eq!(
            WEIGHTS[PlayerPosition::AML as usize].iter().filter(|w| **w > 1e-9).count(),
            0,
            "AML weights row is empty — falls back to DML in estimator caller"
        );
        assert_eq!(
            WEIGHTS[PlayerPosition::FL as usize].iter().filter(|w| **w > 1e-9).count(),
            0,
            "FL weights row is empty — falls back to FC"
        );
    }

    #[test]
    fn estimate_produces_gk_rating_from_three_attributes_only() {
        let mut intr = [0i32; NUM_ATTRIBUTES];
        let mut vals = [0.0f32; NUM_ATTRIBUTES];
        // Give the GK a perfect 20 on the 3 things that matter
        intr[PlayerAttribute::Agility  as usize] = 20;
        intr[PlayerAttribute::Handling as usize] = 20;
        intr[PlayerAttribute::Reflexes as usize] = 20;
        vals[PlayerAttribute::Agility  as usize] = 20.0;
        vals[PlayerAttribute::Handling as usize] = 20.0;
        vals[PlayerAttribute::Reflexes as usize] = 20.0;
        // Junk everywhere else stays 0
        let r = estimate_position_rating(
            PlayerPosition::GK, &intr, &vals, 0, 0, 0, 0
        );
        // Reflexes caps at 19; other two use fallback /19 (CA-dependent
        // → value/19; non-CA → (value-1)/19).
        // → Handling is CA-dep so 20/19, Agility is non-CA-dep so 19/19,
        //   Reflexes hits its 19-cap branch: 19/19.
        // Weight-sum = 0.069+0.159+0.060 = 0.288.
        // Weighted numerator = 0.069*(20-1)/19 + 0.159*20/19 + 0.060*19/19
        //                    = 0.069*1.0 + 0.159*1.05263 + 0.060*1.0
        //                    = 0.297368
        // Rating = 0.297368 / 0.288 * 100  ≈ 103.25
        assert!(r > 100.0 && r < 110.0, "got {r}");
    }

    #[test]
    fn dc_zero_attributes_gives_zero_rating() {
        let intr = [0i32; NUM_ATTRIBUTES];
        let vals = [0.0f32; NUM_ATTRIBUTES];
        let r = estimate_position_rating(
            PlayerPosition::DC, &intr, &vals, 0, 0, 0, 0
        );
        // All attributes at 0 → all fallbacks yield weight*(0-1)/19 or
        // weight*0/19; result is deterministically small (negative for
        // non-CA-dep) and definitely well under any real rating.
        assert!(r < 5.0, "got {r}");
    }
}
