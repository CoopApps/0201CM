//! Transfers screen builder — direct port of `FUN_008E3700`
//! (218 bytes decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/screen_transfers_setup/0x008e3700.c`.
//! Dispatched from the sidebar msg 0x415 (Transfers). Registers three
//! callbacks with the screen system:
//!
//! * draw: `FUN_008E37E0`
//! * event: `LAB_008E48B0`
//! * refresh: `LAB_008E5000`
//!
//! Then pushes 10 slot values (0..=9) via `FUN_007E7130(idx, val, 0)`
//! to seed the screen state.

use serde::{Deserialize, Serialize};

/// The 10 typed slots the exe pushes for the Transfers screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransfersView {
    /// Slot 0: currently-selected club id, -1 = none.
    pub selected_club: Option<u32>,
    /// Slot 1: filter flag byte from `FUN_00536B90(0)` — the "show
    /// only bosman" or "show only loans" toggle set at kickoff.
    pub filter_flag: i8,
    /// Slot 2: current year (`DAT_00ACDE92`).
    pub year: u16,
    /// Slot 3: 1 (sort-order flag).
    pub sort_order: u8,
    /// Slot 4: 0 (window-scroll offset).
    pub scroll_offset: u32,
    /// Slot 5: 0 (selected-row index).
    pub selected_row: u32,
    /// Slot 6: 1 (position-filter enable flag).
    pub position_filter: u8,
    /// Slot 7: 1 (nationality-filter enable flag).
    pub nationality_filter: u8,
    /// Slot 8: 0 (max-price filter).
    pub max_price_filter: u32,
    /// Slot 9: 0 (max-wage filter).
    pub max_wage_filter: u32,
}

impl Default for TransfersView {
    fn default() -> Self {
        Self {
            selected_club: None,
            filter_flag: 0,
            year: 0,
            sort_order: 1,
            scroll_offset: 0,
            selected_row: 0,
            position_filter: 1,
            nationality_filter: 1,
            max_price_filter: 0,
            max_wage_filter: 0,
        }
    }
}

/// Direct port of `FUN_008E3700`. The exe first calls `FUN_007E6570`
/// to register callbacks — we treat that as a precondition; when it
/// returns 0 (registration fail), the exe skips all `FUN_007E7130`
/// pushes. We model that by making the whole function optional.
///
/// * `filter_flag` — result of `FUN_00536B90(0)`, the initial filter state.
/// * `date_year` — `DAT_00ACDE92`.
///
/// Returns `Some(view)` on success, `None` if callback registration failed.
pub fn build_transfers_screen(
    registration_ok: bool,
    filter_flag: i8,
    date_year: u16,
) -> Option<TransfersView> {
    if !registration_ok { return None; }
    Some(TransfersView {
        selected_club: None,      // slot 0 = -1
        filter_flag,              // slot 1
        year: date_year,          // slot 2
        sort_order: 1,            // slot 3
        scroll_offset: 0,         // slot 4
        selected_row: 0,          // slot 5
        position_filter: 1,       // slot 6
        nationality_filter: 1,    // slot 7
        max_price_filter: 0,      // slot 8
        max_wage_filter: 0,       // slot 9
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_registration_returns_none() {
        assert!(build_transfers_screen(false, 0, 2001).is_none());
    }

    #[test]
    fn successful_build_matches_exe_slot_defaults() {
        let v = build_transfers_screen(true, 7, 2001).unwrap();
        assert_eq!(v.selected_club, None);   // slot 0 = -1
        assert_eq!(v.filter_flag, 7);        // slot 1 from param
        assert_eq!(v.year, 2001);            // slot 2 from DAT_00ACDE92
        assert_eq!(v.sort_order, 1);         // slot 3
        assert_eq!(v.scroll_offset, 0);      // slot 4
        assert_eq!(v.selected_row, 0);       // slot 5
        assert_eq!(v.position_filter, 1);    // slot 6
        assert_eq!(v.nationality_filter, 1); // slot 7
        assert_eq!(v.max_price_filter, 0);   // slot 8
        assert_eq!(v.max_wage_filter, 0);    // slot 9
    }

    #[test]
    fn default_matches_exe_zero_state() {
        let d = TransfersView::default();
        assert_eq!(d.sort_order, 1);
        assert_eq!(d.position_filter, 1);
        assert_eq!(d.nationality_filter, 1);
    }

    #[test]
    fn filter_flag_carries_negative_bytes() {
        // The exe reads `(char)FUN_00536B90(0)` — signed. Verify -1 flows.
        let v = build_transfers_screen(true, -1, 2001).unwrap();
        assert_eq!(v.filter_flag, -1);
    }
}
