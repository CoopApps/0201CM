//! FIFA nation rankings — port of the recompute function that actually lives
//! inline in `game.cpp` at `FUN_005c01d0` (`game_recompute_fifa_rankings`, 645
//! bytes). The full analysis of the algorithm is in
//! `reports/game_cpp_analysis.md` §4.
//!
//! Algorithm (from the Ghidra decompile `decompiled/005c01d0.c`):
//!   for each of DAT_00acd558 nations at stride 0x122:
//!       ranking = nation[+0xc8] * W3          // most recent (highest weight)
//!               + nation[+0xb0] * W2
//!               + nation[+0xb8] * W1
//!               + nation[+0xc0] * W0
//!               + nation[+0xd8]
//!               + nation[+0xd0]
//!   qsort by ranking
//!
//! The weights `W0..W3` are `.rdata` doubles at `0x00956918` (0.2) /
//! `0x009569b0` (0.8) / `0x00956e00` (0.4) / `0x00956e48` (0.6) —
//! LIFTED via cm-lift from the shipped exe's .rdata:
//!   W(N-4, most decayed) = 0.2   (year N-4 weight)
//!   W(N-3)               = 0.4
//!   W(N-2)               = 0.6
//!   W(N-0, current year) = 0.8
//! Combined with the +0xD0 (year N-1) and +0xD8 (pending accrual) which
//! are added at unit weight, this is a 5-year weighted-history sum
//! (steadily-increasing weight, current year heaviest).
//!
//! Shipped data populates only `+0xA8`; `FUN_005c01d0`'s one-time init
//! (lines 20-36) fans that coefficient out into all five history slots
//! `+0xB0..+0xD0` on first run, so the formula's day-0 value is `3 × coef`.
//!
//! The TABLE the FIFA Rankings screen shows is a separate runtime record
//! (`FUN_00577b00`, comp_fifa): `points(+0xc) = coef(+0xA8) × 1/35`, sorted
//! descending (`sub_00577ff0`), displayed `× 10` with "%.2f". [`compute`]
//! builds that table exactly. Its monthly/annual evolution
//! (`FUN_00577210` / `FUN_00577d70`) is not yet ported.

use serde::{Deserialize, Serialize};

use crate::typed_records::NationView;
use crate::DomainOpaqueRecord;

/// VERIFIED FIFA ranking weights lifted from cm0102.exe .rdata via
/// cm-lift `fp_sniper scan-refs`. Consumed at FUN_005c01d0:57-61.
pub const FIFA_W_YEAR_MINUS_4: f64 = crate::exe_constants::DAT_00956918; // 0.2
pub const FIFA_W_YEAR_MINUS_3: f64 = crate::exe_constants::DAT_00956E00; // 0.4
pub const FIFA_W_YEAR_MINUS_2: f64 = crate::exe_constants::DAT_00956E48; // 0.6
pub const FIFA_W_YEAR_MINUS_0: f64 = crate::exe_constants::DAT_009569B0; // 0.8

/// Real FIFA ranking score — VERIFIED formula from FUN_005c01d0:57-61.
///
/// score = year_N_0 * 0.8
///       + year_N_4 * 0.2
///       + year_N_3 * 0.4
///       + year_N_2 * 0.6
///       + pending_accrual (+0xD8)
///       + year_N_1 (+0xD0)
///
/// Caller supplies the 6 per-nation history-slot values (as stored on
/// nation record at +0xB0..+0xD8). Returns the exact score the shipped
/// exe would sort by.
#[inline]
// GDI-REG: 005c01d0 PORTED_EXACT
pub fn fifa_score(year_n_4: f64, year_n_3: f64, year_n_2: f64,
                  year_n_0: f64, pending_accrual: f64, year_n_1: f64) -> f64 {
    year_n_0 * FIFA_W_YEAR_MINUS_0
        + year_n_4 * FIFA_W_YEAR_MINUS_4
        + year_n_3 * FIFA_W_YEAR_MINUS_3
        + year_n_2 * FIFA_W_YEAR_MINUS_2
        + pending_accrual
        + year_n_1
}

/// Number of continent records the shipped world has (Africa=0 … S.America=5).
/// A nation whose `+0x71` byte is outside this range has no continent object
/// at runtime, and `FUN_005773c0` leaves it out of the ranking array.
const CONTINENT_COUNT: i32 = 6;

/// One row of the runtime FIFA table — the exe's 0x2c-byte per-nation record
/// built by `FUN_00577b00` (comp_fifa): `{nation index @+0, points: f32 @+0xc,
/// rank: i16 @+0x10, prev_rank: i16 @+0x12, history: [i32; 6] @+0x14}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// GDI-REG: 00577890 REPLACED_BY_RUST
pub struct NationRanking {
    pub nation_id: u32,
    pub nation_name: String,
    /// The exe's `+0xc` float. At game start it is
    /// `nation coefficient (+0xA8) × DAT_00958340 (1/35)` — VERIFIED from the
    /// `FUN_00577b00` x87 sequence (`fld [nation+0xa8]; fmul [0x958340]; …;
    /// fstp [rec+0xc]`). The FIFA Rankings screen shows
    /// `points × DAT_00956A78_F32 (10.0)` with "%.2f".
    ///
    /// Kept under the historical field name `strength` so consumers don't
    /// churn; it is NOT a club-reputation proxy any more.
    pub strength: f32,
    /// 1-based rank after the descending sort (`+0x10`).
    pub rank: u16,
    /// Continent id (`+0x71`, 0..5) — the screen's third column shows the
    /// continent name for ranked nations (`FUN_004A2200` reads `+0x71 → +4`).
    pub continent_id: i32,
    /// The six per-nation history ints (`+0x14..+0x2c`), all seeded with
    /// `(int)(points × DAT_00956908 (1000))` by `FUN_00577b00`. They roll
    /// annually and feed the monthly points update (`FUN_00577210` /
    /// `FUN_00577d70`) — not yet ported, carried so the state exists.
    pub history: [i32; 6],
}

/// Build the start-of-game FIFA table exactly as `FUN_00577b00` + the display
/// sort `FUN_005773c0` (comparator `sub_00577ff0`: descending by `+0xc`) do:
///
/// * every nation with a valid continent (`+0x71` in `0..CONTINENT_COUNT`) —
///   the array builder skips nations whose continent pointer is null;
/// * `points = coefficient(+0xA8) × 1/35`, as an f32 (the exe stores a float);
/// * sorted descending by points. The exe uses C `qsort` (unstable); ties keep
///   nation-record order here, which is the deterministic choice.
///
/// Later evolution of `points` (monthly maintenance on the 18th, annual
/// history shift, match-result feed) is the runtime FIFA subsystem
/// (`FUN_00577210`, `FUN_00577d70`) — a tick-side port still to do.
// GDI-REG: 00577ff0 PORTED_EXACT
// GDI-REG: 005773c0 PORTED_BEHAVIOURAL
// GDI-REG: 00577b00 PORTED_EXACT
// GDI-REG: 00577550 REPLACED_BY_RUST
// GDI-REG: 00577ff0 PORTED_EXACT
// GDI-REG: 005773c0 PORTED_BEHAVIOURAL
// GDI-REG: 00577b00 PORTED_EXACT
pub fn compute(nations: &[DomainOpaqueRecord]) -> Vec<NationRanking> {
    let mut out: Vec<NationRanking> = nations
        .iter()
        .filter_map(|n| {
            let v = NationView::new(n);
            let continent_id = v.continent_id();
            if !(0..CONTINENT_COUNT).contains(&continent_id) {
                return None;
            }
            let coefficient = v.fifa_coefficient(0).unwrap_or(0.0);
            let points = (coefficient * crate::exe_constants::DAT_00958340) as f32;
            let seed = (points as f64 * crate::exe_constants::DAT_00956908) as i32;
            Some(NationRanking {
                nation_id: v.id(),
                nation_name: v.primary_name(),
                strength: points,
                rank: 0,
                continent_id,
                history: [seed; 6],
            })
        })
        .collect();
    // sub_00577ff0: `fld [a+0xc]; fcomp [b+0xc]` — a < b ⇒ a sorts after b.
    out.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));
    for (i, r) in out.iter_mut().enumerate() {
        r.rank = (i + 1) as u16;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal 0x122-byte nation record: id @0, name @4 (51 B), continent
    /// byte @0x71, FIFA coefficient f64 @0xa8 — the only fields `compute` reads.
    fn nation(id: u32, name: &str, continent: u8, coefficient: f64) -> DomainOpaqueRecord {
        let mut raw = vec![0u8; 0x122];
        raw[0..4].copy_from_slice(&id.to_le_bytes());
        raw[4..4 + name.len()].copy_from_slice(name.as_bytes());
        raw[0x71] = continent;
        raw[0xa8..0xb0].copy_from_slice(&coefficient.to_le_bytes());
        DomainOpaqueRecord {
            ordinal: id,
            id,
            primary_name: Some(name.to_string()),
            secondary_name: None,
            short_name: None,
            text_candidates: Vec::new(),
            raw,
        }
    }

    #[test]
    fn points_are_coefficient_over_35_and_sorted_descending() {
        let rows = compute(&[
            nation(1, "Lowland", 2, 700.0),
            nation(2, "Topland", 5, 1750.0),
            nation(3, "Midland", 0, 1050.0),
        ]);
        let names: Vec<&str> = rows.iter().map(|r| r.nation_name.as_str()).collect();
        assert_eq!(names, ["Topland", "Midland", "Lowland"]);
        assert!((rows[0].strength - 50.0).abs() < 1e-5, "1750/35 = 50");
        assert!((rows[1].strength - 30.0).abs() < 1e-5, "1050/35 = 30");
        assert!((rows[2].strength - 20.0).abs() < 1e-5, "700/35 = 20");
        assert_eq!(rows.iter().map(|r| r.rank).collect::<Vec<_>>(), [1, 2, 3]);
        // Africa (continent 0) is a valid continent and IS ranked.
        assert_eq!(rows[1].continent_id, 0);
    }

    #[test]
    fn history_ints_seed_at_points_times_1000() {
        let rows = compute(&[nation(9, "X", 1, 1750.0)]);
        assert_eq!(rows[0].history, [50_000; 6]);
    }

    #[test]
    fn nations_without_a_continent_are_not_ranked() {
        // 0xFF at +0x71 = no continent object at runtime → skipped by FUN_005773c0.
        let rows = compute(&[nation(1, "Ghost", 0xff, 9999.0), nation(2, "Real", 3, 35.0)]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].nation_name, "Real");
        assert!((rows[0].strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn display_value_is_points_times_ten() {
        // The screen prints points × DAT_00956A78_F32 with "%.2f": 1750/35 × 10 = 500.00.
        let rows = compute(&[nation(1, "N", 2, 1750.0)]);
        let shown = rows[0].strength * crate::exe_constants::DAT_00956A78_F32;
        assert_eq!(format!("{shown:.2}"), "500.00");
    }
}
