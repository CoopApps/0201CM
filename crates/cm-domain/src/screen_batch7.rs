//! Batch 7: Match Report screen + fixture-history lookup helper.
//!
//! Decompiles: `d:/cm0102-carve/decompiled/screen_batch7/`.
//! * `0x00701240.c` — Match Report screen setup (cmd 0x7D3, 393 B, 22 slots)
//! * `0x007116B0.c` — Fixture-history extractor (called from dispatch
//!   BEFORE the screen setup to fill the report; 434 B)

use serde::{Deserialize, Serialize};

// =====================================================================
// Match Report — cmd 0x7D3 → FUN_00701240(param_1)
// =====================================================================

/// Match Report view — 22 named slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchReportView {
    /// Slot 0: 0 (view-mode reset).
    pub view_mode: u32,
    /// Slot 2: 6 (default section — "Goals").
    pub default_section: u8,
    /// Slot 9: -1 (unselected event sentinel).
    pub selected_event: Option<u32>,
    /// Slot 10: 3 (event-detail expansion level).
    pub detail_level: u8,
    /// Slot 0xB: -1 (unselected player sentinel).
    pub selected_player: Option<u32>,
    /// Slot 0xD: -1 (unselected event-related sentinel).
    pub selected_event_related: Option<u32>,
    /// Slot 0x1A: -1 (unselected filter sentinel).
    pub selected_filter: Option<u32>,
    /// Slot 0x1C: 1 (show-summary flag).
    pub show_summary: u8,
    /// Slot 0x1E: `param_1` (fixture id or match handle).
    pub fixture_handle: u32,
    /// Slot 0x21: -1 (unselected item sentinel).
    pub selected_item: Option<u32>,
    /// Slot 0x23: -1 (unselected sub-tab sentinel).
    pub selected_sub_tab: Option<u32>,
}

impl Default for MatchReportView {
    fn default() -> Self {
        Self {
            view_mode: 0, default_section: 6,
            selected_event: None, detail_level: 3,
            selected_player: None, selected_event_related: None,
            selected_filter: None, show_summary: 1,
            fixture_handle: 0, selected_item: None, selected_sub_tab: None,
        }
    }
}

/// Direct port of `FUN_00701240(param_1)`.
pub fn build_match_report(
    registration_ok: bool,
    fixture_handle: u32,
) -> Option<MatchReportView> {
    if !registration_ok { return None; }
    Some(MatchReportView { fixture_handle, ..Default::default() })
}

// =====================================================================
// Fixture-history extractor — FUN_007116B0(fixture_idx, out_buf)
// =====================================================================

/// Fixture-history lookup result. Ported from `FUN_007116B0(fixture_idx, out)`
/// which is called BEFORE the match-report screen opens.
///
/// The exe reads `in_ECX + fixture_idx*0xC` to get the fixture record,
/// checks bounds against `in_ECX[7]` (count), then walks the
/// `FUN_00672400()` iterator to find all events matching the fixture's
/// home/away team ids, copying 0x13 (19) dwords per matching event
/// into `out`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureHistoryResult {
    pub found: bool,
    /// Number of events copied (max = out buffer capacity / 19).
    pub event_count: u32,
}

/// Direct port of the fixture-index bounds check + event-count logic
/// from `FUN_007116B0`. `fixture_count` = `in_ECX[7]`.
pub fn extract_fixture_history(
    fixture_idx: i32,
    fixture_count: i32,
    matching_events_available: u32,
) -> FixtureHistoryResult {
    // Exe: `if (param_1 < 0 || in_ECX[7]-1 < param_1) → error`.
    if fixture_idx < 0 || fixture_idx > fixture_count.saturating_sub(1) {
        return FixtureHistoryResult { found: false, event_count: 0 };
    }
    FixtureHistoryResult { found: true, event_count: matching_events_available }
}

// =====================================================================
// World-populator companions
// =====================================================================

use crate::world_pools::WorldPools;

/// Populate a Match-Report view for a caller-supplied fixture handle.
///
/// Fixture ids come from the current-day match list (`FUN_00699640` on
/// the tick), which isn't on the pool facade — the caller supplies it.
/// Returning `None` on a nil handle keeps the exe's zero-guard.
pub fn populate_match_report(
    _pools: &WorldPools<'_>,
    fixture_handle: u32,
) -> Option<MatchReportView> {
    if fixture_handle == 0 { return None; }
    build_match_report(true, fixture_handle)
}

/// Populate a fixture-history extraction with caller-supplied bounds.
/// The exe reads `in_ECX + fixture_idx*0xC` — no pool slot names it,
/// so the caller passes the index + count.
pub fn populate_fixture_history(
    _pools: &WorldPools<'_>,
    fixture_idx: i32,
    fixture_count: i32,
    matching_events_available: u32,
) -> FixtureHistoryResult {
    extract_fixture_history(fixture_idx, fixture_count, matching_events_available)
}

#[cfg(test)]
#[allow(dead_code)]
const TODO_POPULATOR_INFO: &str = "\
    MatchReportView.fixture_handle needs the current-day match list \
    (built by FUN_00699640 during the game tick) surfaced on the pool \
    facade — no fixture pool exists on the facade yet.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_report_default_slots_match_exe_values() {
        let v = build_match_report(true, 42).unwrap();
        assert_eq!(v.default_section, 6);   // slot 2
        assert_eq!(v.detail_level, 3);      // slot 10
        assert_eq!(v.selected_event, None); // slot 9 = -1
        assert_eq!(v.selected_player, None); // slot 0xB = -1
        assert_eq!(v.show_summary, 1);      // slot 0x1C
        assert_eq!(v.fixture_handle, 42);   // slot 0x1E from param_1
        assert_eq!(v.selected_item, None);  // slot 0x21 = -1
        assert_eq!(v.selected_sub_tab, None); // slot 0x23 = -1
    }

    #[test]
    fn match_report_failed_registration_returns_none() {
        assert!(build_match_report(false, 42).is_none());
    }

    #[test]
    fn fixture_history_out_of_bounds_reports_not_found() {
        let r = extract_fixture_history(-1, 10, 5);
        assert!(!r.found);
        // param_1 == count also fails (exe: count-1 must be >= param).
        let r = extract_fixture_history(10, 10, 5);
        assert!(!r.found);
    }

    #[test]
    fn fixture_history_in_bounds_reports_found() {
        let r = extract_fixture_history(0, 10, 5);
        assert!(r.found);
        assert_eq!(r.event_count, 5);
        // Boundary: last valid idx = count - 1.
        let r = extract_fixture_history(9, 10, 3);
        assert!(r.found);
        assert_eq!(r.event_count, 3);
    }

    #[test]
    fn fixture_history_zero_events_still_found() {
        let r = extract_fixture_history(0, 10, 0);
        assert!(r.found);
        assert_eq!(r.event_count, 0);
    }

    // --- Populators ---
    use crate::world_pools::WorldPools;

    #[test]
    fn populate_match_report_nil_handle_yields_none() {
        assert!(populate_match_report(&WorldPools::empty(), 0).is_none());
    }

    #[test]
    fn populate_match_report_carries_handle() {
        let v = populate_match_report(&WorldPools::empty(), 42).unwrap();
        assert_eq!(v.fixture_handle, 42);
        assert_eq!(v.default_section, 6);
    }

    #[test]
    fn populate_fixture_history_delegates() {
        let r = populate_fixture_history(&WorldPools::empty(), 0, 10, 3);
        assert!(r.found);
        assert_eq!(r.event_count, 3);
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A `_from_pools`)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b7 pool sources: MatchReportView.fixture_handle needs the current-day \
    match list (built by FUN_00699640 during the game tick) surfaced on \
    the pools facade — no fixture pool exists on the facade yet, so the \
    caller supplies it directly. FixtureHistoryResult bounds come from \
    in_ECX[7] (fixture-index array count), also not on the facade.";

pub fn populate_match_report_from_pools(
    pools: &WorldPools<'_>,
    fixture_handle: u32,
) -> Option<MatchReportView> {
    populate_match_report(pools, fixture_handle)
}

pub fn populate_fixture_history_from_pools(
    pools: &WorldPools<'_>,
    fixture_idx: i32,
    fixture_count: i32,
    matching_events_available: u32,
) -> FixtureHistoryResult {
    populate_fixture_history(pools, fixture_idx, fixture_count, matching_events_available)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn match_report_nil_handle_yields_none() {
        assert!(populate_match_report_from_pools(&WorldPools::empty(), 0).is_none());
    }

    #[test]
    fn match_report_carries_handle() {
        let v = populate_match_report_from_pools(&WorldPools::empty(), 42).unwrap();
        assert_eq!(v.fixture_handle, 42);
        assert_eq!(v.default_section, 6);
    }

    #[test]
    fn fixture_history_delegates() {
        let r = populate_fixture_history_from_pools(&WorldPools::empty(), 0, 10, 3);
        assert!(r.found);
        assert_eq!(r.event_count, 3);
    }

    #[test]
    fn fixture_history_out_of_bounds_not_found() {
        let r = populate_fixture_history_from_pools(&WorldPools::empty(), -1, 10, 3);
        assert!(!r.found);
    }
}
