//! Byte-exact port of the in-match card model — the severity assembly
//! (`FUN_006cf230`/`FUN_006af650`) and the applier verdict (`FUN_006cf040`).
//! Decode: `reports/foul_severity_decode.md`, `reports/inmatch_card_model_decode.md`.
//!
//! LEVEL 1 (algorithm recovered + tested) ONLY. These are pure functions of
//! a *staged foul* (foul code + participants) and the game RNG. They are NOT
//! yet wired to production: the exe stages fouls inside the per-token
//! positional match simulation (frustration accumulator `player+0x31`, pitch
//! cells `FUN_006db520`, action generators `FUN_006d63f0`/`006e0740`,
//! nearest-player `FUN_006ae160`). Our condensed team-CA engine has none of
//! that, so it cannot stage fouls the exe's way — that positional simulation
//! is the real prerequisite for producing cards (recorded, not faked).

use crate::sim_rng::SimRng;

/// The applier verdict for a staged foul.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardVerdict {
    None,
    Yellow,
    /// Straight red, or a second yellow → red.
    Red,
}

/// Discipline map from the opponent's experience/discipline byte
/// (`other+0x25`) → `cVar5`. Exe `FUN_006cf230` C79-94.
// GDI-REG: 006cf040 PORTED_EXACT
pub fn disc_map(exp: i32) -> i32 {
    if exp == 0x14 { 2 }
    else if exp >= 0x12 { 1 }
    else if exp >= 10 { 0 }      // 10..=17
    else if exp >= 5 { -1 }      // 5..=9
    else if exp >= 3 { -1 }      // 3..=4
    else { -3 }                  // <3
}

/// The severity byte for a staged foul of `foul_code` (the ASCII action
/// code, e.g. 0x2c = two-footed tackle). `cards_this_match` = fouler
/// `+0x14`; `opp_exp` = victim `+0x25`; `over_team_limit` = fouler's side
/// already over its card tolerance; `already_booked` = fouler `+0x24 != 0`;
/// `victim_bump` = the connected-tackle victim-reaction add (0/1/3/5).
/// Byte-exact port of the `FUN_006cf230` accumulation (report §2).
#[allow(clippy::too_many_arguments)]
// GDI-REG: 006cf230 PORTED_EXACT
pub fn foul_severity(
    foul_code: u8,
    cards_this_match: i32,
    opp_exp: i32,
    over_team_limit: bool,
    already_booked: bool,
    victim_bump: i32,
    rng: &mut SimRng,
) -> i32 {
    // (c) discipline map + over-limit bump.
    let mut c_var5 = disc_map(opp_exp);
    if over_team_limit {
        rng.rand(4); // consumed in exe order
        c_var5 += 1;
    }
    // (d) per-offence-type accumulation. `rc(b)` = rand(cards_this_match + b).
    let mut sev: i32 = 0;
    let rc = |r: &mut SimRng, bonus: i32| r.rand(cards_this_match + bonus);
    match foul_code {
        0x21 | 0x22 | 0x13 => { sev += rc(rng, 0) + rng.rand(5) + c_var5; }
        0x2a => { sev += rc(rng, 0) + 4; }                 // '*'
        0x2b => { sev += rc(rng, 0) + 3; }                 // '+'
        0x2c => { sev += rc(rng, 0x14) + c_var5 + 9; }     // ','
        0x2d => { sev += rc(rng, 0) + c_var5 + 4; }        // '-'
        0x2e => { sev += rc(rng, 0) + c_var5 + 1; }        // '.'
        0x2f => { sev += rc(rng, 0) + c_var5 + 9; }        // '/'
        0x30 => { sev += rc(rng, 0) + c_var5 + 20; }       // '0'
        0x31 => { sev += rc(rng, 10) + c_var5 + 20; }      // '1'
        0x32 => { sev += rc(rng, 0) + c_var5 + 18; }       // '2'
        0x5f => { sev += rc(rng, 7) + c_var5 + 3; }        // '_'
        0x60 => { sev += rc(rng, 20) + c_var5 + 9; }       // '`'
        0x61 => { sev += rc(rng, 0) + c_var5 + 20; }       // 'a'
        _ => { sev += rc(rng, 0) + c_var5; }
    }
    // already booked → subtract trunc(rand(21 - opp_exp)/5)
    if already_booked {
        let t = rng.rand(0x15 - opp_exp);
        sev -= t / 5;
    }
    // (e) victim-reaction bump for a connected challenge.
    sev += victim_bump;
    sev
}

/// The applier verdict (`FUN_006cf040` C72-80): a staged foul of `severity`,
/// with `already_yellow` = fouler already booked this match (second yellow →
/// red). Draws in exe order: red-test `rand(15)`, then none-test `rand(5)`.
// GDI-REG: 006cf040 PORTED_EXACT
pub fn card_verdict(severity: i32, already_yellow: bool, rng: &mut SimRng) -> CardVerdict {
    if rng.rand(15) + 9 < severity {
        CardVerdict::Red
    } else if severity <= rng.rand(5) + 3 {
        CardVerdict::None
    } else if already_yellow {
        CardVerdict::Red // second yellow
    } else {
        CardVerdict::Yellow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disc_map_bands() {
        assert_eq!(disc_map(20), 2);
        assert_eq!(disc_map(18), 1);
        assert_eq!(disc_map(15), 0);
        assert_eq!(disc_map(7), -1);
        assert_eq!(disc_map(3), -1);
        assert_eq!(disc_map(1), -3);
    }

    #[test]
    fn verdict_thresholds() {
        // Very high severity → red regardless of the two draws.
        let mut r = SimRng::seed(1);
        assert_eq!(card_verdict(100, false, &mut r), CardVerdict::Red);
        // Zero severity → none.
        let mut r = SimRng::seed(2);
        assert_eq!(card_verdict(0, false, &mut r), CardVerdict::None);
        // Deterministic for a fixed seed.
        let mut a = SimRng::seed(9);
        let mut b = SimRng::seed(9);
        for s in 0..40 {
            assert_eq!(card_verdict(s, false, &mut a), card_verdict(s, false, &mut b));
        }
    }

    #[test]
    fn severity_deterministic_and_uses_type() {
        let mut a = SimRng::seed(5);
        let mut b = SimRng::seed(5);
        let sa = foul_severity(0x2c, 0, 12, false, false, 5, &mut a);
        let sb = foul_severity(0x2c, 0, 12, false, false, 5, &mut b);
        assert_eq!(sa, sb);
        // Two-footed ',' with victim bump 5 is a heavy foul.
        assert!(sa >= 9);
    }
}
