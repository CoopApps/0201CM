//! English Second Division exact schedule integration + generic
//! cm0102-gdi walker/matrix primitives.
//!
//! **AUTHORITATIVE SPECIFICATION**: `D:/cm0102/cm0102_GDI.exe`.
//! DirectDraw (`cm0102.exe`) VAs appear only as corroborating
//! cross-references.
//!
//! # Contents & status
//!
//! | Item                       | GDI VA        | Status                                  |
//! |----------------------------|---------------|-----------------------------------------|
//! | [`walker_step`]            | `0x0066ee40`  | VERIFIED EXACT — PORTED                 |
//! | [`matrix_seed_base`]       | `0x00669340`  | VERIFIED EXACT — NOT YET PORTED (stub)  |
//! | [`matrix_perturb`]         | `0x0066b900`  | STRUCTURE VERIFIED — SEMANTICS PARTIAL  |
//! | [`round_robin_driver`]     | `0x00668450`  | STRUCTURE VERIFIED — SEMANTICS PARTIAL  |
//! | [`generate_eng_second_dates`] | (composite)  | VERIFIED EXACT — PORTED                 |
//!
//! # Integration architecture
//!
//! The audit at `reports/fixture_disasm/RUST_ARCHITECTURE_AUDIT.md`
//! established the boundary:
//!
//! * **Generic pipeline** — [`walker_step`], matrix seeders, driver,
//!   `exe_date::write_round_record`, `exe_date::write_slot`. Shared
//!   by every competition constructor in `cm0102-gdi`.
//! * **Competition-specific data** — the 46-tuple
//!   `exe_date::ENG_SECOND_2001_TEMPLATE` and its 46-round buffer.
//!   Every English pyramid league will contribute a similar template
//!   (Premier, First, Third, Conference) without cloning code.
//!
//! Do NOT hard-code eng_second constants into generic helpers. Where
//! a value in this file is competition-specific it is either a
//! parameter or a comment says so explicitly.

use crate::exe_date::build_eng_second_schedule;
use crate::game_rng::GameRng;
use crate::{CmPackedDate, GameDate};

/// English Second Division comp id — cm0102-gdi.exe `0x0055f291`
/// (`mov byte ptr [esi+0x50], 9` inside `eng_second_ctor`).
pub const COMP_ID_ENG_SECOND: u32 = 9;

/// Number of clubs in the English Second Division 2001/02 season.
/// Sourced from the roster populator `FUN_005603d0`, which writes
/// exactly 24 club entries into `comp+0xb1`. Verified in the
/// cm0102-gdi disassembly at ctor byte offset `+0xbc`.
pub const ENG_SECOND_CLUB_COUNT: usize = 24;

/// Number of rounds in the English Second Division schedule.
/// Sourced from the schedule-getter body which writes `0x2e` (= 46)
/// to the `comp+0xa9` round-count out-parameter (see
/// `reports/fixture_disasm/gdi_disasm_probe.py`).
pub const ENG_SECOND_ROUND_COUNT: usize = 46;

/// Standard Gregorian leap-year rule (Rust-local; matches
/// `exe_date::pack_date`'s inline check and `lib::is_leap_year`).
fn is_leap_year(year: u16) -> bool {
    let y = u32::from(year);
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

// ---------------------------------------------------------------------------
// Generic cm0102-gdi walker (used by every round-robin driver in the exe)
// ---------------------------------------------------------------------------

/// Byte-exact port of `FUN_0066ee40` — the round walker used by
/// `FUN_00668450` (English round-robin driver) and equivalent
/// competition drivers.
///
/// cm0102-gdi.exe `0x0066ee40`. DirectDraw corroboration:
/// `0x0066f280` — the two bodies differ only in the RNG target
/// (`0x008fbe20` vs `0x008fc4f0`) and the special-case comp id
/// (`[0x009bbaf0]` vs `[0x009bbba8]`).
///
/// Function contract (deduced from static disassembly + agent
/// verification):
///
/// ```text
/// walker(prev_col, state_p, comp_id, n_clubs, matches_per_pair,
///        n_rounds, flag_byte) -> new_col
/// ```
///
/// * `prev_col`: last returned column (init to -1 by driver).
/// * `state_p`: mutable byte state, values in `{0, 1, 2, 3}` (init 0).
/// * `comp_id`: competition id from `comp+0x04`.
/// * `n_clubs`, `matches_per_pair`, `n_rounds`: from comp struct
///   fields `+0x3e`, `+0x3c`, `+0xa9`.
/// * `flag_byte`: from `comp+0xd9`. Bits `0x40` and `0x80` select
///   deterministic overrides that skip the state machine.
///
/// The `special_comp_id` argument (`[0x009bbaf0]` in the exe) is a
/// global constant; passed as a parameter here so the same code
/// serves every competition without re-reading a global.
///
/// # Determinism
///
/// The only randomness is a single `rand_mod(4)` in the `state==0`
/// branch (RNG target `0x008fbe20` in GDI). Everything else is a
/// pure state machine. `rng` may be `None` — the state-0 RNG branch
/// is then treated as if `rand_mod` returned 0 (no perturbation),
/// which yields the pure round-robin. Tests use this to exercise
/// the deterministic subset.
pub fn walker_step(
    prev_col: i32,
    state: &mut u8,
    comp_id: i32,
    n_clubs: i16,
    matches_per_pair: i16,
    n_rounds: i16,
    flag_byte: u8,
    special_comp_id: i32,
    rng: Option<&mut GameRng>,
) -> i32 {
    // cm0102-gdi.exe 0x0066ee40:
    //     if (comp_id == [DAT_009bbaf0]) {
    //         if (prev_col == 2) return 4;
    //         if (prev_col == 5) return 3;
    //     }
    if comp_id == special_comp_id {
        if prev_col == 2 {
            return 4;
        }
        if prev_col == 5 {
            return 3;
        }
    }

    // Guard rails: matches_per_pair < 2 or n_clubs < 8 -> return prev+1.
    // (cm0102-gdi.exe 0x0066ee75, 0x0066ee81)
    if matches_per_pair < 2 || n_clubs < 8 {
        return prev_col + 1;
    }

    // Flag-byte overrides checked BEFORE the state machine.
    if flag_byte & 0x40 != 0 {
        // cm0102-gdi.exe: `return prev_col + 1;` (no wrap)
        return prev_col + 1;
    }
    if flag_byte & 0x80 != 0 {
        // Mirror rule: p1 = prev + 1; if p1 == n/2 -> n-1; if p1 == n -> n/2.
        let p1 = prev_col + 1;
        let half = (n_rounds as i32) >> 1;
        if p1 == half {
            return (n_rounds as i32) - 1;
        }
        if p1 == n_rounds as i32 {
            return half;
        }
        return p1;
    }

    // State machine.
    // States 1, 2, 3 are unconditional; state 0 has an RNG gate.
    match *state {
        1 => {
            *state = 2;
            let x = prev_col - 1;
            if x >= 0 { x } else { (n_rounds as i32) - 1 }
        }
        2 => {
            *state = 3;
            let x = prev_col - 1;
            if x >= 0 { x } else { (n_rounds as i32) - 1 }
        }
        3 => {
            *state = 0;
            let x = prev_col + 3;
            if x < n_rounds as i32 { x } else { x - n_rounds as i32 }
        }
        _ => {
            // state == 0 (or any other value; the exe branches only
            // on values 0..=3, but treats "other" as 0).
            // cm0102-gdi.exe 0x0066eeee..:
            //   if (prev_col < n_rounds - 5 && rand_mod(4) > 1) {
            //       *state = 1;
            //       return (prev_col + 3) mod n_rounds;
            //   }
            //   return (prev_col + 1) mod n_rounds;
            let rng_hit = if prev_col < (n_rounds as i32) - 5 {
                match rng {
                    Some(r) => r.rand_mod(4) > 1,
                    None => false,
                }
            } else {
                false
            };
            if rng_hit {
                *state = 1;
                let x = prev_col + 3;
                if x < n_rounds as i32 { x } else { x - n_rounds as i32 }
            } else {
                let x = prev_col + 1;
                if x < n_rounds as i32 { x } else { x - n_rounds as i32 }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Matrix seeder / perturbation — stubs to be populated in follow-up commits
// ---------------------------------------------------------------------------

/// Byte-exact port of `FUN_00669340` (matrix base seeder).
///
/// cm0102-gdi.exe `0x00669340` (size 0x18f). Called from the driver
/// at `0x00668570` with `(spine_ptr, n_even)`. Seeds a canonical
/// pure round-robin adjacency into `spine[1..=n_even]` where
/// `spine[i]` is a row of `n_even` ints. Non-zero cells at
/// `spine[row][col]` indicate a scheduled pair; the sign selects
/// H/A orientation.
///
/// **Status**: NOT YET PORTED. This function's byte-exact decode
/// remains open and blocks the byte-exact round-robin driver.
///
/// The stub returns an empty matrix so callers can compile; any
/// caller that consumes its output must gate on `!matrix.is_empty()`.
pub fn matrix_seed_base(_n_even: usize) -> Vec<Vec<i32>> {
    Vec::new()
}

/// Byte-exact port of `FUN_0066b900` (matrix perturbation).
///
/// cm0102-gdi.exe `0x0066b900` (size 0x9b1). Called from the driver
/// with `n_even`; guarded by a comp-id check against
/// `[0x009bbba4]` / `[0x009bbbac]`. Skips secondary seed if the
/// comp id matches those constants.
///
/// **Status**: STRUCTURE VERIFIED — SEMANTICS PARTIAL. Body decode
/// pending.
pub fn matrix_perturb(_matrix: &mut [Vec<i32>], _rng: &mut GameRng) {
    // Deliberately empty until byte-exact port lands.
}

/// Byte-exact port of `FUN_00668450` (English round-robin driver).
///
/// cm0102-gdi.exe `0x00668450` (size 0x920). Structural
/// reconstruction in `reports/fixture_disasm/FUN_00668890_DECODE.md`
/// plus the GDI-specific overlay in
/// `reports/fixture_disasm/GDI_CORRECTION_REPORT.md`.
///
/// **Status**: STRUCTURE VERIFIED — SEMANTICS PARTIAL. Full byte-
/// exact behaviour requires the [`matrix_seed_base`] and
/// [`matrix_perturb`] ports to land, plus a runtime capture of a
/// concrete pair sequence for differential validation.
pub fn round_robin_driver_stub_returns_empty() -> Vec<(i32, i32, i32)> {
    Vec::new()
}

// ---------------------------------------------------------------------------
// English Second Division exact-date dispatch
// ---------------------------------------------------------------------------

/// Return the 46 exact `GameDate` values for English Second Division
/// `season_base_year` (2001 in the shipped save).
///
/// These are the actual dates written by the cm0102-gdi schedule-
/// getter at `0x0055f540`, reproduced byte-exact by
/// [`build_eng_second_schedule`]. Byte-exact vs runtime capture
/// (SHA256 `682a5ea6…`) enforced by
/// `exe_date::tests::full_buffer_matches_runtime_capture_gdi`.
///
/// Order corresponds to the 46 round-writer calls inside
/// `0x0055f540` (round index 0..45). Each date has been snapped by
/// `apply_flag_snap` — so index 0 is Sat 11 Aug 2001, index 3 is
/// Mon 27 Aug (Bank Holiday midweek), etc.
///
/// Callers use this to REPLACE the flat `+7 days` stub for the
/// English Second Division. Team pair generation remains the
/// existing Berger add-mod approximation until the round-robin
/// driver + walker + matrix land byte-exact; documented in the
/// integration commit.
pub fn generate_eng_second_dates(season_base_year: u16) -> Vec<GameDate> {
    // 1. Build the exact 2990-byte buffer via the byte-exact
    //    schedule-getter port.
    let buf = build_eng_second_schedule(season_base_year);
    // 2. Extract each round's day-of-year (`+0x00`) and year offset
    //    (`+0x02`) and convert to `GameDate`. The exe stores
    //    0-INDEXED day-of-year; `CmPackedDate::to_game_date` expects
    //    1-INDEXED, so we add 1 before decoding.
    let mut out = Vec::with_capacity(46);
    for round in 0..46 {
        let off = round * 0x41;
        let doy_0idx = i16::from_le_bytes([buf[off], buf[off + 1]]);
        let year_off = i16::from_le_bytes([buf[off + 2], buf[off + 3]]);
        let year = ((season_base_year as i32) + year_off as i32) as u16;
        let packed = CmPackedDate {
            day_of_year: (doy_0idx as u16).saturating_add(1),
            year,
            leap_year: is_leap_year(year),
        };
        out.push(packed.to_game_date());
    }
    out
}

// ---------------------------------------------------------------------------
// Minimal GameRng bridge — abstracts the underlying `game_rng::GameRng`
// so tests can inject captured sequences without touching cm-domain
// wiring elsewhere.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Landmark subset — spot-check a few notable dates.
    #[test]
    fn eng_second_dates_2001_02_landmarks() {
        let dates = generate_eng_second_dates(2001);
        assert_eq!(dates.len(), 46);
        assert_eq!(dates[0],  GameDate { year: 2001, month: 8, day: 11 });
        assert_eq!(dates[3],  GameDate { year: 2001, month: 8, day: 27 });
        assert_eq!(dates[6],  GameDate { year: 2001, month: 9, day: 12 });
        assert_eq!(dates[24], GameDate { year: 2001, month: 12, day: 26 });
        assert_eq!(dates[26], GameDate { year: 2002, month: 1, day: 1 });
        assert_eq!(dates[37], GameDate { year: 2002, month: 3, day: 19 });
        assert_eq!(dates[45], GameDate { year: 2002, month: 5, day: 5 });
    }

    /// Differential test: every one of the 46 dates must match the
    /// canonical list decoded from the runtime capture at
    /// `reports/fixture_disasm/runtime/20260913_131323_gdi_buffer_0.bin`.
    /// This is the stage-level fixture-date reference trace the user
    /// asked for. If it fails, `generate_eng_second_dates` and the
    /// underlying `exe_date` primitives are diverging from the
    /// authoritative cm0102-gdi build.
    #[test]
    fn eng_second_dates_2001_02_full_46() {
        // (round, year, month, day) — decoded from GDI buffer capture
        // via reports/fixture_disasm/decode_buffer.py.
        let expected: &[(usize, u16, u8, u8)] = &[
            (0,  2001,  8, 11), (1,  2001,  8, 18), (2,  2001,  8, 25),
            (3,  2001,  8, 27), (4,  2001,  9,  1), (5,  2001,  9,  8),
            (6,  2001,  9, 12), (7,  2001,  9, 15), (8,  2001,  9, 22),
            (9,  2001,  9, 29), (10, 2001, 10,  6), (11, 2001, 10, 13),
            (12, 2001, 10, 16), (13, 2001, 10, 20), (14, 2001, 10, 23),
            (15, 2001, 10, 27), (16, 2001, 11,  3), (17, 2001, 11, 10),
            (18, 2001, 11, 17), (19, 2001, 11, 24), (20, 2001, 12,  1),
            (21, 2001, 12,  8), (22, 2001, 12, 15), (23, 2001, 12, 22),
            (24, 2001, 12, 26), (25, 2001, 12, 29), (26, 2002,  1,  1),
            (27, 2002,  1, 12), (28, 2002,  1, 19), (29, 2002,  2,  2),
            (30, 2002,  2,  9), (31, 2002,  2, 16), (32, 2002,  2, 19),
            (33, 2002,  2, 23), (34, 2002,  3,  2), (35, 2002,  3,  6),
            (36, 2002,  3,  9), (37, 2002,  3, 19), (38, 2002,  3, 23),
            (39, 2002,  3, 30), (40, 2002,  4,  6), (41, 2002,  4, 13),
            (42, 2002,  4, 15), (43, 2002,  4, 20), (44, 2002,  4, 27),
            (45, 2002,  5,  5),
        ];
        let dates = generate_eng_second_dates(2001);
        assert_eq!(dates.len(), 46);
        for &(idx, y, m, d) in expected {
            let got = &dates[idx];
            assert_eq!(
                got,
                &GameDate { year: y, month: m, day: d },
                "round {idx}: got {got:?} expected {y}-{m}-{d}"
            );
        }
    }

    /// Walker pure round-robin (no RNG, default state) advances +1
    /// each step and wraps at `n_rounds`.
    #[test]
    fn walker_pure_advances_by_one() {
        let mut state = 0u8;
        let mut prev = -1i32;
        let seq: Vec<i32> = (0..46)
            .map(|_| {
                prev = walker_step(prev, &mut state, /*comp*/ 9, 24, 2, 46, 0, /*special*/ i32::MIN, None);
                prev
            })
            .collect();
        // Without RNG hits the walker degenerates to +1; first call
        // returns 0, then 1, 2, ..., 45.
        assert_eq!(seq[0], 0);
        assert_eq!(seq[1], 1);
        assert_eq!(seq[45], 45);
    }

    /// Walker flag_byte & 0x40 override returns prev+1 unconditionally.
    #[test]
    fn walker_flag40_pure_increment() {
        let mut state = 0u8;
        // n_clubs<8 also triggers the shortcut, so avoid that; use
        // 0x40 with valid clubs.
        assert_eq!(walker_step(-1, &mut state, 9, 24, 2, 46, 0x40, i32::MIN, None), 0);
        assert_eq!(walker_step(10, &mut state, 9, 24, 2, 46, 0x40, i32::MIN, None), 11);
    }

    /// Walker guard rail: matches_per_pair < 2 → returns prev+1
    /// even outside 0x40.
    #[test]
    fn walker_guard_low_matches_per_pair() {
        let mut state = 0u8;
        assert_eq!(walker_step(5, &mut state, 9, 24, 1, 46, 0, i32::MIN, None), 6);
    }

    /// Walker special-case comp id (cm0102-gdi `[0x009bbaf0]`):
    /// when comp_id matches, the state machine is bypassed and
    /// specific inputs get magic outputs.
    #[test]
    fn walker_special_comp_id_shortcut() {
        let mut state = 0u8;
        // If comp_id matches the special constant AND prev == 2 → 4.
        assert_eq!(walker_step(2, &mut state, /*comp*/ 42, 24, 2, 46, 0, /*special*/ 42, None), 4);
        // Different prev falls through to the guard/state path.
        assert_eq!(walker_step(5, &mut state, 42, 24, 2, 46, 0, 42, None), 3);
    }

    /// Walker state cycle 1→2→3→0: each state decrements or +3s
    /// with wrap.
    #[test]
    fn walker_state_cycle_advances() {
        let mut state = 1u8;
        // state 1: prev - 1, next state 2
        assert_eq!(walker_step(5, &mut state, 9, 24, 2, 46, 0, i32::MIN, None), 4);
        assert_eq!(state, 2);
        // state 2: prev - 1, next state 3
        assert_eq!(walker_step(4, &mut state, 9, 24, 2, 46, 0, i32::MIN, None), 3);
        assert_eq!(state, 3);
        // state 3: prev + 3, next state 0
        assert_eq!(walker_step(3, &mut state, 9, 24, 2, 46, 0, i32::MIN, None), 6);
        assert_eq!(state, 0);
    }

    /// Walker state 1 wrap: prev=0 → n_rounds-1 (=45).
    #[test]
    fn walker_state1_wrap() {
        let mut state = 1u8;
        assert_eq!(walker_step(0, &mut state, 9, 24, 2, 46, 0, i32::MIN, None), 45);
    }

    /// Walker state 3 wrap: prev=44 + 3 = 47 → 47 - 46 = 1.
    #[test]
    fn walker_state3_wrap() {
        let mut state = 3u8;
        assert_eq!(walker_step(44, &mut state, 9, 24, 2, 46, 0, i32::MIN, None), 1);
        assert_eq!(state, 0);
    }
}
