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
//! We can now compute the REAL ranking formula when history data is
//! available on the nation record. Until the nation-record history-slot
//! layout (+0xb0..+0xd8) is threaded through NationView, we fall back to
//! the average-club-reputation proxy for the strength scalar.

use serde::{Deserialize, Serialize};

use crate::typed_records::{ClubView, NationView};
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
pub fn fifa_score(year_n_4: f64, year_n_3: f64, year_n_2: f64,
                  year_n_0: f64, pending_accrual: f64, year_n_1: f64) -> f64 {
    year_n_0 * FIFA_W_YEAR_MINUS_0
        + year_n_4 * FIFA_W_YEAR_MINUS_4
        + year_n_3 * FIFA_W_YEAR_MINUS_3
        + year_n_2 * FIFA_W_YEAR_MINUS_2
        + pending_accrual
        + year_n_1
}

/// One row of the FIFA ranking table: nation + a computed strength value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NationRanking {
    pub nation_id: u32,
    pub nation_name: String,
    /// Higher = stronger. Currently a proxy (average of the nation's clubs'
    /// reputations); will match the exe once the weight table + history slots
    /// are decoded.
    pub strength: f32,
}

/// Compute the current FIFA-style ranking of every nation, best-first.
///
/// Proxy: `mean(club.reputation)` over the nation's clubs. A nation with no
/// clubs gets strength 0.
pub fn compute(
    nations: &[DomainOpaqueRecord],
    clubs: &[DomainOpaqueRecord],
) -> Vec<NationRanking> {
    // Sum + count club reputations per nation id.
    let mut tally: std::collections::BTreeMap<i32, (u64, u32)> = std::collections::BTreeMap::new();
    for rec in clubs {
        let cv = ClubView::new(rec);
        let Some(nation_id) = cv.nation_id() else { continue };
        let e = tally.entry(nation_id).or_default();
        e.0 += cv.reputation() as u64;
        e.1 += 1;
    }
    let mut out: Vec<NationRanking> = nations
        .iter()
        .map(|n| {
            let v = NationView::new(n);
            let nid = v.id() as i32;
            let strength = tally
                .get(&nid)
                .map(|(sum, count)| *sum as f32 / *count as f32)
                .unwrap_or(0.0);
            NationRanking {
                nation_id: v.id(),
                nation_name: v.primary_name(),
                strength,
            }
        })
        .collect();
    out.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));
    out
}

#[cfg(test)]
mod tests {
    // Testing this without a live World is awkward; the shape is verified via
    // its consumers.
}
