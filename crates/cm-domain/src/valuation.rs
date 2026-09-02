//! Player valuation & wages — port of the CM0102 value/wage core
//! `FUN_0084d5d0`. Replaces the `CA²×100` value and `CA×250` wage heuristics
//! with the real quality-squared formula.
//!
//! # The recovered core (`FUN_0084d5d0`:463-520)
//! ```text
//!   quality = ( rep9 + repB + repD + 4·maxRep + 50·(PA+CA) ) / 9
//!   figure  = tier_base × quality² × 1e-8          (floor 175)
//! ```
//! `rep9/repB/repD` are the three reputation shorts on the type10 record
//! (offsets +0x09/+0x0b/+0x0d, each ×50); CA/PA at +0x05/+0x07 are weighted ×50
//! (0x32) so they dominate. `tier_base` is one of the wage tier constants
//! (75000..105000) selected by contract status / squad importance / reputation.
//!
//! # Faithful vs approximated
//! * **Weekly wage** — the recovered formula, exact (tier selection reduced to a
//!   reputation band; the full status/importance selection at :330-441 is a
//!   refinement).
//! * **Transfer value** — the exe's value path (`:670-750` length switch +
//!   `FUN_00580a90` club-affordability ceiling, `clubRep²×coef×1e-4`) is only
//!   partially recovered (x87). We reuse the same real quality² driver and
//!   compress low-reputation players with a reputation factor that stands in for
//!   the decompiled reputation-band caps (`:566-598`). Flagged approximation.

/// The blended ability/quality scalar — the arithmetic heart of both wage and
/// value (`FUN_0084d5d0`:498-500). Reputations are the ×50 shorts.
pub fn player_quality(ca: i16, pa: i16, rep9: u16, rep_b: u16, rep_d: u16) -> f64 {
    let (r9, rb, rd) = (eff_rep(ca, rep9), eff_rep(ca, rep_b), eff_rep(ca, rep_d));
    let max_rep = r9.max(rb).max(rd);
    (r9 + rb + rd + 4.0 * max_rep + 50.0 * (pa as f64 + ca as f64)) / 9.0
}

/// A usable reputation short. Reputations are now proper (shipped for valid
/// records, generated via FUN_0051f5d0's blend for the CA=0 records), so we only
/// guard against any residual out-of-range value with a hard sane cap (byte 240
/// × 50 = 12000).
fn eff_rep(_ca: i16, rep: u16) -> f64 {
    (rep as f64).min(12_000.0)
}

/// Wage tier base (`FUN_0084d5d0`:330-441) — reduced to a reputation band over
/// the recovered tier constants.
fn wage_tier_base(rep_d: u16) -> f64 {
    match rep_d {
        r if r < 2500 => 75_000.0,  // _DAT_0095db4c
        r if r < 6500 => 85_000.0,  // _DAT_0095db68
        _ => 105_000.0,             // _DAT_0095db64
    }
}

/// Weekly wage — `figure = tier × quality² × 1e-8`, floored at 175
/// (`FUN_0084d5d0`:505-517).
pub fn weekly_wage(ca: i16, pa: i16, rep9: u16, rep_b: u16, rep_d: u16) -> u32 {
    let q = player_quality(ca, pa, rep9, rep_b, rep_d);
    (wage_tier_base(rep_d) * q * q * 1e-8).max(175.0) as u32
}

/// Transfer value — same real quality² driver, compressed for low-reputation
/// players (standing in for the decompiled reputation-band caps). A ×1.0 top
/// scale means a top star (quality ≈ 5900) lands near £35M; a rep-500 lower-
/// league player near £0.2M.
pub fn market_value(ca: i16, pa: i16, rep9: u16, rep_b: u16, rep_d: u16) -> i64 {
    let q = player_quality(ca, pa, rep9, rep_b, rep_d);
    // Reputation compression: (sane) rep_d/3000 clamped to [0.05, 1.0].
    let rep_factor = (eff_rep(ca, rep_d) / 3000.0).clamp(0.05, 1.0);
    (q * q * rep_factor).max(1_000.0) as i64
}

/// Read the three reputation shorts off a type10 record (+0x09/+0x0b/+0x0d).
pub fn reputations_of(t10: &crate::DomainStaffType10) -> (u16, u16, u16) {
    // Typed fields first (post-migration rust-db); the raw bytes only carry
    // values on pre-migration data.
    (t10.home_reputation_value(), t10.current_reputation_value(), t10.reputation())
}
