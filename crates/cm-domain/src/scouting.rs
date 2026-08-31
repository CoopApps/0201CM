//! Scouting + fog-of-war subsystem.
//!
//! Decoded from the `scout_manager.cpp` (0x007de900 cluster) and
//! `fog_of_war.cpp` (0x00599d60 cluster) function families. Full report:
//! `reports/scouting_decode.md`. Existing minimal `fog_of_war` port only
//! covers the `0x3e036` save-integrity magic — this module is the actual
//! scout-knowledge substrate.
//!
//! # The exe's shape (short summary)
//!
//! Two subsystems share the "scout" name:
//!
//! * **`ScoutManager`** — a fixed `16 mgrs × 10 slots × 0x2b0` array. The
//!   10 slots per manager are typed by index: `0` = ShortList, `1` =
//!   PlayerSearch, `2` = StaffSearch, `3..=9` = the 7 club scouts (matching
//!   the 7 `ClubView::scout_slot(i)` positions). Each 0x2b0-byte slot is
//!   the exe's `ScoutTask` — owner nation, slot index, creation date,
//!   task kind, target key, saved-search blob, result iterator.
//! * **`FogOfWar`** — a per-manager `fog.dat` cache: 16 human slots each
//!   holding `{u8* per_nation[9 bytes], owner_nation}`. The 9-byte
//!   per-nation record layout is not yet decoded — the tick fn that
//!   mutates it lives outside the identified cluster.
//!
//! # Critical finding — visibility gate is HUMAN-ONLY
//!
//! `FUN_00848da0` (transfer composer) has zero scout-cluster references.
//! **The AI transfer logic sees full CA/PA regardless of scout coverage.**
//! Fog only bites on the human path (Scout Club toolbar, player/staff
//! profile screens, transfer/loan UI). This means [`scouted_view`] is a
//! DISPLAY filter, not a market-visibility filter.
//!
//! # Port scope
//!
//! This module ports the substrate as types + traits + a weekly-tick
//! signature. The knowledge-accumulation FORMULA and the 45-byte
//! `ScoutReport` payload are open decode gaps — see [reports/scouting_decode.md §7]
//! — so `weekly_tick` and `apply_report` are placeholders that grow
//! coverage without yet applying it. The DISPLAY filter path
//! ([`scouted_view`], [`AttributeReveal`]) is fully usable now.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What kind of thing a scout is watching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchTargetKind {
    Nation,
    League,
    Competition,
    Club,
    /// A specific player id — the "Send scout to watch <name>" flow.
    Player,
    /// Free-text saved search — the exe stores it as a 0x16b-byte blob
    /// (`FUN_0079af40`). Not yet decoded at the field level; carried as an
    /// opaque payload for round-trip.
    SavedSearch,
    /// Slot present but unused.
    None,
}

/// The exe's `ScoutTask` — one 0x2b0-byte record per (manager_id, slot).
/// Slot semantics: 0 = ShortList, 1 = PlayerSearch, 2 = StaffSearch,
/// 3..=9 = the 7 club scouts (matching `ClubView::scout_slot(i)`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoutAssignment {
    /// Human manager's nation id. Contract `+0x00`.
    pub owner_nation: u32,
    /// 0..=9. Contract `+0x04`. Priority is implicit in slot order (lower
    /// slot ids fire first in the daily queue).
    pub slot_index: u8,
    /// (year, day-of-year) — the exe stores as a packed date at +0x05.
    pub created: (u16, u16),
    /// Contract `+0x0a`.
    pub kind: WatchTargetKind,
    /// Contract `+0x0b`.
    pub enabled: bool,
    /// Target id — a club_id / nation_id / competition_id / player_id
    /// depending on `kind`. Contract `+0x174` (via the embedded PlayerKey).
    pub target_id: Option<i32>,
    /// "Away trip" sentinel `-2` at contract +0x191 — scout is en route,
    /// won't produce a report until they arrive.
    pub away: bool,
}

/// What a human viewer knows about a single player. Per-(viewer_club, player).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PlayerKnowledge {
    /// Overall scouting coverage 0..=100. Feeds every attribute-fog range.
    pub coverage_pct: u8,
    /// How many weeks a scout has actively watched this player.
    pub weeks_watched: u16,
    /// Per-attribute reveal bits — one bit per named attribute in
    /// [`crate::ATTRIBUTE_NAMES`] order. Bit set = precise value revealed;
    /// bit clear = only a range visible. When `coverage_pct` is high enough
    /// the exe converts the range into a precise reveal for that attribute.
    pub attribute_reveal: u64,
}

impl PlayerKnowledge {
    /// The exe converts coverage into a per-attribute range whose half-width
    /// shrinks as knowledge grows. `range_half_width(true_value)` returns
    /// the ± band the manager screen displays. `coverage_pct = 0` → ±10
    /// (full attribute range: 1..20 shown as 1–20), `coverage_pct = 100` →
    /// ±0 (exact value). Formula matches the shape observed in CM01/02
    /// screenshots; exact-formula decode is an open gap
    /// (`reports/scouting_decode.md §7`), so this is a monotonic-in-coverage
    /// linear interpolation that agrees at both endpoints.
    pub fn range_half_width(&self) -> u8 {
        // 10 at coverage 0 → 0 at coverage 100.
        ((100u32.saturating_sub(self.coverage_pct as u32) * 10) / 100) as u8
    }

    /// A single-attribute reveal: precise value or a `[lo..=hi]` band.
    pub fn reveal(&self, attribute_index: usize, true_value: u8) -> AttributeReveal {
        assert!(attribute_index < 64);
        if self.attribute_reveal & (1u64 << attribute_index) != 0 {
            AttributeReveal::Exact(true_value)
        } else {
            let hw = self.range_half_width();
            let lo = true_value.saturating_sub(hw).max(1);
            let hi = true_value.saturating_add(hw).min(20);
            AttributeReveal::Range(lo, hi)
        }
    }
}

/// What the manager sees for a single attribute — precise value or a fog
/// range. The DISPLAY layer reads this instead of hitting raw type10 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeReveal {
    Exact(u8),
    Range(u8, u8),
}

impl AttributeReveal {
    pub fn as_string(self) -> String {
        match self {
            AttributeReveal::Exact(v)   => v.to_string(),
            AttributeReveal::Range(a,b) if a == b => a.to_string(),
            AttributeReveal::Range(a,b) => format!("{}–{}", a, b),
        }
    }
}

/// Per-manager scout book — the ported `ScoutManager` slice for one human.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ManagerScoutBook {
    /// The 10 slots (0=ShortList, 1=PlayerSearch, 2=StaffSearch, 3..9=scouts).
    pub slots: [Option<ScoutAssignment>; 10],
    /// Per-(observed player) knowledge, keyed by player id.
    pub knowledge: BTreeMap<u32, PlayerKnowledge>,
}

/// Top-level scout book, one entry per human manager. The exe carries 16 max.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScoutBook {
    /// Keyed by manager_id (the type6 record id).
    pub managers: BTreeMap<u32, ManagerScoutBook>,
}

impl ScoutBook {
    pub fn new() -> Self { Self::default() }

    /// Assign a scout to watch something. Idempotent on (manager, slot).
    pub fn assign(&mut self, manager_id: u32, assignment: ScoutAssignment) {
        let slot = assignment.slot_index as usize;
        if slot >= 10 { return; }
        let book = self.managers.entry(manager_id).or_default();
        book.slots[slot] = Some(assignment);
    }

    /// Clear a scout slot (recall).
    pub fn recall(&mut self, manager_id: u32, slot: u8) {
        if let Some(book) = self.managers.get_mut(&manager_id) {
            if (slot as usize) < 10 {
                book.slots[slot as usize] = None;
            }
        }
    }

    /// Read the current watch list for one manager.
    pub fn assignments(&self, manager_id: u32) -> Vec<&ScoutAssignment> {
        self.managers.get(&manager_id)
            .map(|b| b.slots.iter().filter_map(|s| s.as_ref()).collect())
            .unwrap_or_default()
    }

    /// Weekly tick — advances knowledge on every observed player. Placeholder
    /// pending the exact formula decode; grows coverage by a constant per
    /// week per active scout and per-attribute-reveal bit-flip at 25 / 50 /
    /// 75% coverage checkpoints.
    ///
    /// TODO: replace with the real per-scout-ability × weeks-watched formula
    /// once `reports/scouting_decode.md §7` open items land.
    pub fn weekly_tick(&mut self) {
        for book in self.managers.values_mut() {
            for slot in book.slots.iter().flatten() {
                if !slot.enabled { continue; }
                if slot.kind != WatchTargetKind::Player { continue; }
                let Some(pid) = slot.target_id else { continue };
                let k = book.knowledge.entry(pid as u32).or_default();
                k.weeks_watched = k.weeks_watched.saturating_add(1);
                k.coverage_pct = (k.coverage_pct + 3).min(100);
                // Reveal-bit checkpoints — approx of the exe's per-attribute
                // reveal cascade. Order: coverage 25 → first 16 attrs, 50 →
                // next 16, 75 → next 16, 100 → last 16.
                let expected_bits = match k.coverage_pct {
                    0..=24  => 0,
                    25..=49 => 16,
                    50..=74 => 32,
                    75..=99 => 48,
                    _       => 64,
                };
                if expected_bits > 0 {
                    let mask = if expected_bits == 64 { u64::MAX }
                               else { (1u64 << expected_bits) - 1 };
                    k.attribute_reveal |= mask;
                }
            }
        }
    }
}

/// Look up what `viewer_manager_id` knows about `player_id`. Returns an
/// all-fog default when there is no knowledge on record.
pub fn knowledge_for<'a>(
    book: &'a ScoutBook, viewer_manager_id: u32, player_id: u32,
) -> std::borrow::Cow<'a, PlayerKnowledge> {
    static NIL: PlayerKnowledge = PlayerKnowledge {
        coverage_pct: 0, weeks_watched: 0, attribute_reveal: 0,
    };
    if let Some(mgr) = book.managers.get(&viewer_manager_id) {
        if let Some(k) = mgr.knowledge.get(&player_id) {
            return std::borrow::Cow::Borrowed(k);
        }
    }
    std::borrow::Cow::Borrowed(&NIL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_coverage_fogs_to_full_1_20_range() {
        let k = PlayerKnowledge::default();
        assert_eq!(k.range_half_width(), 10);
        match k.reveal(0, 15) {
            AttributeReveal::Range(1, 20) => {}
            other => panic!("expected 1..20 range, got {other:?}"),
        }
    }

    #[test]
    fn full_coverage_reveals_exact_when_bit_set() {
        let mut k = PlayerKnowledge {
            coverage_pct: 100, weeks_watched: 40,
            attribute_reveal: 1,   // bit 0 set
        };
        assert_eq!(k.range_half_width(), 0);
        assert_eq!(k.reveal(0, 15), AttributeReveal::Exact(15));
        // bit 1 clear → range even at full coverage (range collapses to ±0
        // so lo==hi==true value); the enum still returns Range, but
        // as_string prints the single value.
        k.attribute_reveal = 1;
        let r = k.reveal(1, 15);
        assert_eq!(r.as_string(), "15");
    }

    #[test]
    fn range_shrinks_monotonically_with_coverage() {
        let mut widths = Vec::new();
        for pct in [0u8, 25, 50, 75, 100] {
            let k = PlayerKnowledge { coverage_pct: pct, ..Default::default() };
            widths.push(k.range_half_width());
        }
        // Monotonic non-increasing.
        for w in widths.windows(2) { assert!(w[0] >= w[1], "widths: {widths:?}"); }
        assert_eq!(widths.first(), Some(&10));
        assert_eq!(widths.last(),  Some(&0));
    }

    #[test]
    fn weekly_tick_grows_knowledge_and_flips_reveal_bits() {
        let mut book = ScoutBook::new();
        book.assign(1, ScoutAssignment {
            owner_nation: 60, slot_index: 3, created: (2001, 8),
            kind: WatchTargetKind::Player, enabled: true,
            target_id: Some(999), away: false,
        });
        // 10 weeks → coverage 30, first 16 reveal bits set.
        for _ in 0..10 { book.weekly_tick(); }
        let k = knowledge_for(&book, 1, 999);
        assert!(k.coverage_pct >= 25);
        assert!(k.coverage_pct < 50);
        assert_eq!(k.attribute_reveal & ((1u64 << 16) - 1), (1u64 << 16) - 1);
        assert_eq!(k.attribute_reveal & !((1u64 << 16) - 1), 0);
    }

    #[test]
    fn recall_clears_slot() {
        let mut book = ScoutBook::new();
        book.assign(1, ScoutAssignment {
            owner_nation: 60, slot_index: 5, created: (2001, 8),
            kind: WatchTargetKind::Player, enabled: true,
            target_id: Some(42), away: false,
        });
        assert_eq!(book.assignments(1).len(), 1);
        book.recall(1, 5);
        assert_eq!(book.assignments(1).len(), 0);
    }
}
