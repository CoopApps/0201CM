//! Nation dashboard screen builder — direct port of `FUN_00454BB0`
//! (1,542 bytes decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/screen_nation_dashboard/0x00454bb0.c`.
//! Shares the same 35-slot layout as [`crate::screen_club_dashboard`];
//! differs only in the CLUB-RESOLUTION step at the top (which club to
//! show as the manager's current club, given a nation context).
//!
//! # Nation → club resolution
//!
//! Exe lines 69-78:
//! ```pseudo
//! slot = FUN_0074D010(nation)   // active manager slot lookup
//! if slot != 0 && slot+0xE2 != 0:
//!     club = slot+0xE6          // the manager's current club
//! else:
//!     club = nation+0x39        // primary linked club
//!     if club == 0:
//!         club = nation+0x24    // backup club link
//! ```
//!
//! After that, everything else mirrors [`crate::screen_club_dashboard`]
//! — same [`FinanceSummary`] defaults, same 35 slots, same top-3
//! key-player loop. The only user-visible difference in the exe is
//! which entity's slot table is written (per-entity `FUN_007E7000`
//! vs global `FUN_007E7130`).

use serde::{Deserialize, Serialize};

use crate::screen_club_dashboard::{
    build_club_dashboard, ClubDashboardView, FinanceSummary, SquadPlayerAttr,
};

/// Wraps [`ClubDashboardView`] with the nation-context header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NationDashboardView {
    /// Slot 0: the resolved club (may be 0 if nation-only view).
    pub nation_context: u32,
    /// The rest of the slots (0x00..=0x22), same structure as club dashboard.
    pub club_view: ClubDashboardView,
}

/// The nation-record fields the resolver reads (from exe).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NationResolutionInput {
    /// `nation_ptr + 0x39` — primary linked club.
    pub primary_club: Option<u32>,
    /// `nation_ptr + 0x24` — backup linked club.
    pub backup_club: Option<u32>,
    /// `FUN_0074D010(nation)`-derived active manager slot's current
    /// club (`slot + 0xE6`) if the slot exists and its 0xE2 field is
    /// non-zero.
    pub manager_current_club: Option<u32>,
}

/// Port of the nation-dashboard club-resolution step (exe lines 69-78).
pub fn resolve_nation_club(input: NationResolutionInput) -> Option<u32> {
    // Priority: manager's current club, then primary, then backup.
    input.manager_current_club
        .or(input.primary_club)
        .or(input.backup_club)
}

/// Direct port of `FUN_00454BB0` — builds the nation dashboard view.
///
/// * `nation_id` — must be `Some(id)`; the exe errors on
///   `param_1 == 0` (fires MsgBox `club_screens.c:0x323`).
/// * All other args mirror [`crate::screen_club_dashboard::build_club_dashboard`].
///
/// Returns `None` when `nation_id` is `None` (matches the exe's early
/// return via the assertion / MessageBox path).
pub fn build_nation_dashboard(
    nation_id: Option<u32>,
    screen_arg: i32,
    prev_state: Option<&NationDashboardView>,
    is_reserves: bool,
    is_highlighted: bool,
    is_managed_by_current_human: bool,
    nation_resolution: NationResolutionInput,
    finance: Option<FinanceSummary>,
    date_year: u16,
    squad: &[SquadPlayerAttr],
    is_reserve_context: bool,
) -> Option<NationDashboardView> {
    let nation = nation_id?;
    let club = resolve_nation_club(nation_resolution);

    let club_view = build_club_dashboard(
        club, screen_arg,
        prev_state.map(|v| &v.club_view),
        is_reserves,
        is_highlighted,
        is_managed_by_current_human,
        finance,
        date_year,
        squad,
        is_reserve_context,
    );

    Some(NationDashboardView { nation_context: nation, club_view })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nil_nation_returns_none() {
        let out = build_nation_dashboard(
            None, 0, None, false, false, false,
            NationResolutionInput { manager_current_club: None,
                                     primary_club: None, backup_club: None },
            None, 2001, &[], false);
        assert!(out.is_none());
    }

    #[test]
    fn resolution_prefers_manager_current_club_over_primary() {
        // Exe: slot check wins first.
        let c = resolve_nation_club(NationResolutionInput {
            manager_current_club: Some(999),
            primary_club: Some(111),
            backup_club: Some(222),
        });
        assert_eq!(c, Some(999));
    }

    #[test]
    fn resolution_falls_to_primary_when_no_manager_slot() {
        let c = resolve_nation_club(NationResolutionInput {
            manager_current_club: None,
            primary_club: Some(111),
            backup_club: Some(222),
        });
        assert_eq!(c, Some(111));
    }

    #[test]
    fn resolution_falls_to_backup_when_primary_is_zero() {
        let c = resolve_nation_club(NationResolutionInput {
            manager_current_club: None,
            primary_club: None,
            backup_club: Some(222),
        });
        assert_eq!(c, Some(222));
    }

    #[test]
    fn resolution_returns_none_when_all_zero() {
        let c = resolve_nation_club(NationResolutionInput {
            manager_current_club: None, primary_club: None, backup_club: None,
        });
        assert!(c.is_none());
    }

    #[test]
    fn build_populates_35_slots_via_club_view() {
        let v = build_nation_dashboard(
            Some(42), 0, None, false, false, true,
            NationResolutionInput {
                manager_current_club: Some(1234),
                primary_club: None, backup_club: None,
            },
            None, 2001, &[], false).unwrap();
        assert_eq!(v.nation_context, 42);
        assert_eq!(v.club_view.club_id, Some(1234));
        assert_eq!(v.club_view.year, 2001);
    }

    #[test]
    fn reserves_flag_propagates_to_club_view() {
        let v = build_nation_dashboard(
            Some(42), 0, None, true, false, true,
            NationResolutionInput {
                manager_current_club: Some(1234),
                primary_club: None, backup_club: None,
            },
            None, 2001, &[], false).unwrap();
        assert_eq!(v.club_view.tab_index, 5);
        assert_eq!(v.club_view.team_context, 2);
    }

    #[test]
    fn highlighted_flag_sets_bit_0x800() {
        let f = FinanceSummary { weekly_wages: 500, ..Default::default() };
        let v = build_nation_dashboard(
            Some(42), 0, None, false, true, true,
            NationResolutionInput {
                manager_current_club: Some(1234),
                primary_club: None, backup_club: None,
            },
            Some(f), 2001, &[], false).unwrap();
        assert_eq!(v.club_view.header_indicator, 500 | 0x800);
    }

    #[test]
    fn top_three_key_players_ranked_by_score() {
        let squad = vec![
            SquadPlayerAttr { player_id: 1, ca_primary: 100, ca_reserve: 100, form: 5 },
            SquadPlayerAttr { player_id: 2, ca_primary: 200, ca_reserve: 200, form: 10 },
            SquadPlayerAttr { player_id: 3, ca_primary: 150, ca_reserve: 150, form: 8 },
        ];
        let v = build_nation_dashboard(
            Some(42), 0, None, false, false, true,
            NationResolutionInput {
                manager_current_club: Some(1234),
                primary_club: None, backup_club: None,
            },
            None, 2001, &squad, true).unwrap();
        assert_eq!(v.club_view.key_player_1, Some(2));
        assert_eq!(v.club_view.key_player_2, Some(3));
        assert_eq!(v.club_view.key_player_3, Some(1));
    }
}
