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
//! The weights `W0..W3` are runtime `.data` doubles at `0x00956918` /
//! `0x009569b0` / `0x00956e00` / `0x00956e48` — populated by an init function
//! not yet decoded, so their exact values are unknown at rest.
//!
//! **Ported honestly** as a nation-strength calculator: we compute a ranking
//! from data we *do* have (the average reputation of a nation's clubs — a
//! reasonable proxy for national strength). This fixes the "Tahiti wins the
//! Confederations Cup" fidelity gap by giving the international-cup draws
//! a strength-weighted candidate pool. Flagged: not the exact exe algorithm
//! until the runtime weight table and the nation-record history-slot layout
//! (`+0xb0..+0xd8`) are decoded.

use serde::{Deserialize, Serialize};

use crate::typed_records::{ClubView, NationView};
use crate::DomainOpaqueRecord;

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
