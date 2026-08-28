//! Batch 6: player contract, club history, comp dashboard variant.
//!
//! Decompiles under `d:/cm0102-carve/decompiled/screen_batch6/` +
//! `d:/cm0102-carve/decompiled/competition_shell_owner/`:
//! * `0x00476DF0.c` — Player Contract screen (cmd 0x7D1, 245 B)
//! * `0x0046BA80.c` — Club History screen (cmd 0x7E6, 865 B)
//! * `0x00494250.c` — Competition Dashboard variant (goto-specific-comp)
//!   (1000 B — cmd from toolbar/comp-shell)

use serde::{Deserialize, Serialize};

// =====================================================================
// Player Contract — cmd 0x7D1 → FUN_00476DF0(player, note)
// =====================================================================

/// Player Contract view — 2 slots + optional player-record mutation.
///
/// Exe: on `param_1 == 0` fires MsgBox `club_screens:0x34A9`. Then
/// computes `is_managed = (player->+0xCF == currentHuman)`. If yes,
/// writes `player+0x29 = 1` (contract-negotiation flag) and
/// `player+0x2D = note`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractView {
    /// Slot 0: player ptr (opaque id).
    pub player_id: u32,
    /// Slot 1: `is_managed_by_current_human` byte.
    pub is_managed: bool,
    /// Whether the exe wrote back to `player+0x29` and `+0x2D`.
    pub player_flag_write: Option<(u32, u32)>,
}

/// Direct port of `FUN_00476DF0(player, note)`.
///
/// * `player_id == 0` → returns `None` (matches MsgBox error).
/// * `is_managed_by_current_human` matches the exe's `player->+0xCF
///   == currentHuman` predicate. When true, writes player+0x29=1 and
///   player+0x2D=note (the caller applies the write).
pub fn build_contract(
    registration_ok: bool,
    player_id: u32,
    is_managed_by_current_human: bool,
    note: u32,
) -> Option<ContractView> {
    if player_id == 0 { return None; }
    if !registration_ok { return None; }
    let player_flag_write = if is_managed_by_current_human {
        Some((1, note))
    } else { None };
    Some(ContractView { player_id, is_managed: is_managed_by_current_human,
                         player_flag_write })
}

// =====================================================================
// Club History — cmd 0x7E6 → FUN_0046BA80(club, prev_state, mode)
// =====================================================================

/// Club History view — ~28 slots. Similar shape to club dashboard but
/// with different defaults and no top-3 key-player loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubHistoryView {
    /// Slot 0: club ptr (opaque id).
    pub club_id: u32,
    /// Slot 1: view mode — 3 when `param_3 == 0` (all-time), 8 when
    /// `param_3 != 0` (season-by-season).
    pub view_mode: u32,
    /// Slot 2: 1 (sort-order flag).
    pub sort_order: u8,
    /// Slot 0x1D: -1 (unselected-season sentinel).
    pub selected_season: Option<u32>,
}

impl Default for ClubHistoryView {
    fn default() -> Self {
        Self { club_id: 0, view_mode: 3, sort_order: 1, selected_season: None }
    }
}

/// Direct port of `FUN_0046BA80(club, prev_state, mode)`.
///
/// * `club_id == 0` → `None`.
/// * `prev_state != 0` → the exe restores 60 dwords from the prior
///   state via `FUN_007E6EE0` (we model this via `prev_state` arg).
/// * `mode == 0` → view mode 3 (all-time); else view mode 8 (season).
pub fn build_club_history(
    registration_ok: bool,
    club_id: u32,
    _prev_state: Option<&ClubHistoryView>,
    season_mode: bool,
) -> Option<ClubHistoryView> {
    if club_id == 0 { return None; }
    if !registration_ok { return None; }
    Some(ClubHistoryView {
        club_id,
        view_mode: if season_mode { 8 } else { 3 },
        sort_order: 1,
        selected_season: None,
    })
}

// =====================================================================
// Competition Dashboard variant — FUN_00494250(comp, param_2, param_3, prev_state)
// =====================================================================

/// Competition Dashboard "goto specific competition" variant view.
/// Structurally identical to [`crate::screen_batch5::CompetitionDashboardView`]
/// but with a different view-mode default (2 or 3 depending on
/// `param_4`) and conditional slot assignments for slots 8/9 vs 0x11/0x12.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionSpecificView {
    pub vtable_dispatch: u32,
    /// Slot 1: 3 when `param_4 == 0`, else 2.
    pub view_mode: u32,
    /// Slot 2: -2 (default sub-tab).
    pub sub_tab: i32,
    /// Slot 3: -2.
    pub secondary_tab: i32,
    /// Slot 8: `param_2` when `param_4 == 0`, else 0.
    pub browse_param_2: u16,
    /// Slot 9: `param_3` when `param_4 == 0`, else 0.
    pub browse_param_3: u16,
    /// Slot 0x11: `param_2` when `param_4 != 0`, else 0.
    pub goto_param_2: u16,
    /// Slot 0x12: `param_3` when `param_4 != 0`, else 0.
    pub goto_param_3: u16,
    /// Slot 0x1E: 0x11 (primary tab).
    pub primary_tab: u32,
    /// Slot 0x23: 3.
    pub view_sub_mode: u32,
    /// Slot 0x2A: -1.
    pub season: Option<u32>,
    /// Slot 0x2D: 1.
    pub filter_flag: u32,
    /// Slot 0x32: -1.
    pub selected_item: Option<u32>,
}

impl Default for CompetitionSpecificView {
    fn default() -> Self {
        Self {
            vtable_dispatch: 0, view_mode: 3,
            sub_tab: -2, secondary_tab: -2,
            browse_param_2: 0, browse_param_3: 0,
            goto_param_2: 0, goto_param_3: 0,
            primary_tab: 0x11, view_sub_mode: 3,
            season: None, filter_flag: 1, selected_item: None,
        }
    }
}

/// Direct port of `FUN_00494250(comp, param_2, param_3, prev_state)`.
///
/// * `is_goto_specific` = exe's `param_4 != 0`: switches view_mode from
///   3 → 2 and swaps which slots receive `param_2`/`param_3`.
pub fn build_competition_specific(
    registration_ok: bool,
    vtable_dispatch: u32,
    param_2: u16, param_3: u16,
    is_goto_specific: bool,
) -> Option<CompetitionSpecificView> {
    if !registration_ok { return None; }
    Some(CompetitionSpecificView {
        vtable_dispatch,
        view_mode: if is_goto_specific { 2 } else { 3 },
        sub_tab: -2, secondary_tab: -2,
        browse_param_2: if is_goto_specific { 0 } else { param_2 },
        browse_param_3: if is_goto_specific { 0 } else { param_3 },
        goto_param_2:   if is_goto_specific { param_2 } else { 0 },
        goto_param_3:   if is_goto_specific { param_3 } else { 0 },
        primary_tab: 0x11, view_sub_mode: 3,
        season: None, filter_flag: 1, selected_item: None,
    })
}

// =====================================================================
// World-populator companions
// =====================================================================

use crate::world_pools::WorldPools;

/// Populate a Player-Contract view for the given player id.
///
/// The exe reads the player id from a context menu / right-click, so
/// there's nothing on the world facade that names it — the caller has
/// to supply it. `is_managed_by_current_human` derives from
/// `players[player].current_club_id == active_human_club_id`.
pub fn populate_contract(
    pools: &WorldPools<'_>,
    player_id: u32,
    note: u32,
) -> Option<ContractView> {
    // Look the player's current club up on the staff-type6 pool (that's
    // where body+0x35 lives — see the decoded record layout note in
    // memory[record-layouts-decoded]). Type10 is attributes-only.
    let is_managed = match (
        pools.active_human_club_id(),
        pools.staff.iter().find(|s| s.id == player_id),
    ) {
        (Some(club), Some(s)) => s.current_club_id() == Some(club),
        _ => false,
    };
    build_contract(true, player_id, is_managed, note)
}

/// Populate a Club-History view for the active human's club (all-time
/// mode). Returns `None` when no seat / no staff record.
pub fn populate_club_history(pools: &WorldPools<'_>) -> Option<ClubHistoryView> {
    let club_id = pools.active_human_club_id()?;
    build_club_history(true, club_id, None, false)
}

/// Populate a Competition-Specific view for the active human's focus
/// competition (browse mode; `param_2`=`param_3`=0).
pub fn populate_competition_specific(
    pools: &WorldPools<'_>,
) -> Option<CompetitionSpecificView> {
    let _comp_id = pools.active_human_focus_competition()?;
    build_competition_specific(true, 0, 0, 0, false)
}

#[cfg(test)]
#[allow(dead_code)]
const TODO_POPULATOR_INFO: &str = "\
    ContractView.player_id comes from a right-click context — no pool \
    slot names it, so the caller passes it directly. players[N] uses \
    DomainStaffType10 which does NOT expose current_club_id yet; the \
    'is_managed' check uses the staff-type6 back-pointer via \
    active_human_club_id.";

#[cfg(test)]
mod tests {
    use super::*;

    // Contract
    #[test]
    fn contract_nil_player_returns_none() {
        assert!(build_contract(true, 0, false, 0).is_none());
    }
    #[test]
    fn contract_managed_writes_player_flags() {
        let v = build_contract(true, 42, true, 999).unwrap();
        assert!(v.is_managed);
        assert_eq!(v.player_flag_write, Some((1, 999)));
    }
    #[test]
    fn contract_unmanaged_skips_write() {
        let v = build_contract(true, 42, false, 999).unwrap();
        assert!(!v.is_managed);
        assert!(v.player_flag_write.is_none());
    }
    #[test]
    fn contract_failed_registration_returns_none() {
        assert!(build_contract(false, 42, false, 0).is_none());
    }

    // Club History
    #[test]
    fn club_history_nil_club_returns_none() {
        assert!(build_club_history(true, 0, None, false).is_none());
    }
    #[test]
    fn club_history_all_time_uses_mode_3() {
        let v = build_club_history(true, 42, None, false).unwrap();
        assert_eq!(v.view_mode, 3);
    }
    #[test]
    fn club_history_season_uses_mode_8() {
        let v = build_club_history(true, 42, None, true).unwrap();
        assert_eq!(v.view_mode, 8);
    }
    #[test]
    fn club_history_default_selected_season_sentinel() {
        let v = build_club_history(true, 42, None, false).unwrap();
        assert_eq!(v.selected_season, None);
        assert_eq!(v.sort_order, 1);
    }

    // Competition Specific
    #[test]
    fn comp_specific_goto_uses_mode_2_and_slot_0x11() {
        let v = build_competition_specific(true, 0xABCD, 5, 7, true).unwrap();
        assert_eq!(v.view_mode, 2);
        // param_2/3 land at goto_param_* (slot 0x11/0x12), not browse_param_*.
        assert_eq!(v.goto_param_2, 5);
        assert_eq!(v.goto_param_3, 7);
        assert_eq!(v.browse_param_2, 0);
        assert_eq!(v.browse_param_3, 0);
    }

    #[test]
    fn comp_specific_browse_uses_mode_3_and_slot_8() {
        let v = build_competition_specific(true, 0xABCD, 5, 7, false).unwrap();
        assert_eq!(v.view_mode, 3);
        // Reverse: browse_param_* set, goto_param_* zero.
        assert_eq!(v.browse_param_2, 5);
        assert_eq!(v.browse_param_3, 7);
        assert_eq!(v.goto_param_2, 0);
        assert_eq!(v.goto_param_3, 0);
    }

    #[test]
    fn comp_specific_default_slots_match_exe() {
        let v = build_competition_specific(true, 0, 0, 0, false).unwrap();
        assert_eq!(v.sub_tab, -2);
        assert_eq!(v.secondary_tab, -2);
        assert_eq!(v.primary_tab, 0x11);
        assert_eq!(v.view_sub_mode, 3);
        assert_eq!(v.season, None);
        assert_eq!(v.filter_flag, 1);
        assert_eq!(v.selected_item, None);
    }

    #[test]
    fn comp_specific_failed_registration_returns_none() {
        assert!(build_competition_specific(false, 0, 0, 0, false).is_none());
    }

    // --- Populators ---
    use crate::world_pools::WorldPools;
    use crate::{DomainCompetition, DomainStaffType6};

    fn seat_at_club(seat: u32, club_id: u32) -> Vec<DomainStaffType6> {
        let mut body = vec![0u8; 0x60];
        body[0x35..0x39].copy_from_slice(&club_id.to_le_bytes());
        vec![DomainStaffType6 { id: seat, body }]
    }

    #[test]
    fn populate_contract_managed_when_same_club() {
        let staff = seat_at_club(42, 100);
        let mut body2 = vec![0u8; 0x60];
        body2[0x35..0x39].copy_from_slice(&100u32.to_le_bytes());
        let mut all = staff.clone();
        all.push(DomainStaffType6 { id: 55, body: body2 }); // player 55 also at club 100
        let pools = WorldPools {
            staff: &all, active_human_seat: Some(42),
            ..WorldPools::empty()
        };
        let v = populate_contract(&pools, 55, 7).unwrap();
        assert!(v.is_managed);
        assert_eq!(v.player_flag_write, Some((1, 7)));
    }

    #[test]
    fn populate_contract_unmanaged_when_different_club() {
        let seat = seat_at_club(42, 100);
        let mut body2 = vec![0u8; 0x60];
        body2[0x35..0x39].copy_from_slice(&200u32.to_le_bytes());
        let mut all = seat.clone();
        all.push(DomainStaffType6 { id: 55, body: body2 });
        let pools = WorldPools {
            staff: &all, active_human_seat: Some(42),
            ..WorldPools::empty()
        };
        let v = populate_contract(&pools, 55, 7).unwrap();
        assert!(!v.is_managed);
    }

    #[test]
    fn populate_club_history_uses_seat_club() {
        let staff = seat_at_club(42, 100);
        let pools = WorldPools {
            staff: &staff, active_human_seat: Some(42),
            ..WorldPools::empty()
        };
        let v = populate_club_history(&pools).unwrap();
        assert_eq!(v.club_id, 100);
        assert_eq!(v.view_mode, 3); // all-time
    }

    #[test]
    fn populate_club_history_no_seat_yields_none() {
        assert!(populate_club_history(&WorldPools::empty()).is_none());
    }

    #[test]
    fn populate_competition_specific_defaults() {
        let comps = vec![DomainCompetition {
            id: 7, long_name: "L".into(), short_name: "L".into(),
            three_letter_name: "PRM".into(), scope: 2, nation_id: 1,
            last_division: -1, reserve_division: -1, reputation: 0,
            unknown_tail: vec![],
        }];
        let pools = WorldPools { comps: &comps, ..WorldPools::empty() };
        let v = populate_competition_specific(&pools).unwrap();
        assert_eq!(v.view_mode, 3); // browse
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A `_from_pools`)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b6 pool sources: ContractView.player_id comes from a right-click \
    context — no pool slot names it, so the caller passes it in. \
    ClubHistoryView chases active_human_club_id via staff.type6 \
    body+0x35. CompetitionSpecificView.vtable_dispatch mirrors b5 — \
    needs DAT_00AC688C[comp[0]*4], not on the pools facade.";

pub fn populate_contract_from_pools(
    pools: &WorldPools<'_>,
    player_id: u32,
    note: u32,
) -> Option<ContractView> {
    populate_contract(pools, player_id, note)
}

pub fn populate_club_history_from_pools(pools: &WorldPools<'_>) -> Option<ClubHistoryView> {
    populate_club_history(pools)
}

pub fn populate_competition_specific_from_pools(
    pools: &WorldPools<'_>,
) -> Option<CompetitionSpecificView> {
    populate_competition_specific(pools)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;
    use crate::{DomainCompetition, DomainStaffType6};

    fn seat_at_club(seat: u32, club_id: u32) -> Vec<DomainStaffType6> {
        let mut body = vec![0u8; 0x60];
        body[0x35..0x39].copy_from_slice(&club_id.to_le_bytes());
        vec![DomainStaffType6 { id: seat, body }]
    }

    #[test]
    fn contract_nil_player_returns_none() {
        assert!(populate_contract_from_pools(&WorldPools::empty(), 0, 0).is_none());
    }

    #[test]
    fn contract_unmanaged_when_no_seat() {
        let v = populate_contract_from_pools(&WorldPools::empty(), 42, 7).unwrap();
        assert!(!v.is_managed);
        assert!(v.player_flag_write.is_none());
    }

    #[test]
    fn club_history_no_seat_yields_none() {
        assert!(populate_club_history_from_pools(&WorldPools::empty()).is_none());
    }

    #[test]
    fn club_history_seat_gives_club() {
        let staff = seat_at_club(42, 100);
        let pools = WorldPools { staff: &staff, active_human_seat: Some(42), ..WorldPools::empty() };
        let v = populate_club_history_from_pools(&pools).unwrap();
        assert_eq!(v.club_id, 100);
    }

    #[test]
    fn competition_specific_needs_comp_pool() {
        assert!(populate_competition_specific_from_pools(&WorldPools::empty()).is_none());
        let comps = vec![DomainCompetition {
            id: 7, long_name: "L".into(), short_name: "L".into(),
            three_letter_name: "PRM".into(), scope: 2, nation_id: 1,
            last_division: -1, reserve_division: -1, reputation: 0,
            unknown_tail: vec![],
        }];
        let pools = WorldPools { comps: &comps, ..WorldPools::empty() };
        let v = populate_competition_specific_from_pools(&pools).unwrap();
        assert_eq!(v.view_mode, 3);
    }
}
