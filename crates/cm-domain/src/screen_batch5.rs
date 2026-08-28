//! Batch 5: awards, competition dashboard, plus dispatch aliases for
//! squad/reserves that resolve to the already-ported club dashboard.
//!
//! Decompiles under `d:/cm0102-carve/decompiled/screen_batch5/`:
//! * `0x00415010.c` — Awards (cmd 0x7D4, 196 B, 2 slots)
//! * `0x00493E10.c` — Competition dashboard (cmd 2000, 1076 B, 55 slots)
//!
//! Plus discovery from `d:/cm0102-carve/decompiled/screen_batch5_club_disp/0x0074BF60.c`:
//! * cmd 0x7D5 (Squad) → `FUN_00454620(club, 0, 0, 0, 0)` = [`crate::screen_club_dashboard::build_club_dashboard`]
//! * cmd 0x7D6 (Reserves/B-Squad) → `FUN_00454620(0, 1, 0, 0, 0)` = same fn with `is_reserves=true`

use serde::{Deserialize, Serialize};

// =====================================================================
// Awards — cmd 0x7D4 → FUN_00415010(comp)
// =====================================================================

/// Awards view — 2 slots.
///
/// Exe: on `param_1 == 0` fires MsgBox `award_screens:0x5A`. Otherwise
/// pushes slot 0 = `FUN_00414E50(comp)` (award-table extractor result)
/// and slot 1 = 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AwardsView {
    /// Slot 0: award-table handle (from `FUN_00414E50`).
    pub award_table: u32,
    /// Slot 1: 0 (view-mode reset).
    pub view_mode: u32,
}

impl Default for AwardsView {
    fn default() -> Self { Self { award_table: 0, view_mode: 0 } }
}

/// Direct port of `FUN_00415010(param_1)`.
///
/// * `comp_id == 0` → returns `None` (matches exe's MsgBox error path).
/// * `award_table_fn` — closure ports `FUN_00414E50(comp)`; it looks
///   up the competition's award table (winner, runner-up, top-scorer)
///   from its record and returns a handle.
pub fn build_awards(
    registration_ok: bool,
    comp_id: u32,
    award_table_fn: impl FnOnce(u32) -> u32,
) -> Option<AwardsView> {
    if comp_id == 0 { return None; }
    if !registration_ok { return None; }
    Some(AwardsView { award_table: award_table_fn(comp_id), view_mode: 0 })
}

// =====================================================================
// Competition Dashboard — cmd 2000 → FUN_00493E10(comp, -2, -2, 0)
// =====================================================================

/// Competition dashboard view — 55 slots (0x00..=0x36).
///
/// Named after the exe's non-zero-default slot values; the many
/// zero-init slots are grouped into `reserved_*` byte arrays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionDashboardView {
    /// Slot 0: comp vtable dispatch (`DAT_00AC688C[comp[0]*4]`).
    pub vtable_dispatch: u32,
    /// Slot 1: 1 (default view mode).
    pub view_mode: u32,
    /// Slot 2: `param_2` — sub-tab (usually 0xFFFFFFFE = -2 for goto-comp).
    pub sub_tab: i32,
    /// Slot 3: `param_3` — secondary tab.
    pub secondary_tab: i32,
    /// Slot 0x1E: 0x11 (17 = default primary-tab index).
    pub primary_tab: u32,
    /// Slot 0x23: 3 (default view-sub-mode).
    pub view_sub_mode: u32,
    /// Slot 0x2A: -1 (season sentinel).
    pub season: Option<u32>,
    /// Slot 0x2D: 1 (default filter flag).
    pub filter_flag: u32,
    /// Slot 0x32: -1 (unselected-item sentinel).
    pub selected_item: Option<u32>,
    /// Slot 0x36: -1 (unselected-secondary sentinel).
    pub selected_secondary: Option<u32>,
}

impl Default for CompetitionDashboardView {
    fn default() -> Self {
        Self {
            vtable_dispatch: 0, view_mode: 1, sub_tab: -2, secondary_tab: -2,
            primary_tab: 0x11, view_sub_mode: 3, season: None,
            filter_flag: 1, selected_item: None, selected_secondary: None,
        }
    }
}

/// Direct port of `FUN_00493E10(comp, param_2, param_3, prev_state)`.
///
/// * `comp_id` — competition record ptr (opaque id).
/// * `sub_tab`, `secondary_tab` — from the caller; usually `-2` for
///   the "Goto Competition" sidebar dispatch.
/// * `prev_state` — `Some(...)` restores slots 1 / 0x1E / 0x23 from
///   the saved view; else uses defaults.
pub fn build_competition_dashboard(
    registration_ok: bool,
    comp_id: u32,
    vtable_dispatch: u32,
    sub_tab: i32,
    secondary_tab: i32,
    prev_state: Option<&CompetitionDashboardView>,
) -> Option<CompetitionDashboardView> {
    if !registration_ok { return None; }
    let mut v = CompetitionDashboardView {
        vtable_dispatch, view_mode: 1,
        sub_tab, secondary_tab,
        primary_tab: 0x11, view_sub_mode: 3,
        season: None,
        filter_flag: 1,
        selected_item: None, selected_secondary: None,
    };
    // Prior-state restore (exe lines 73-77): slot 1 / 0x1E / 0x23.
    if let Some(prior) = prev_state {
        v.view_mode = prior.view_mode;
        v.primary_tab = prior.primary_tab;
        v.view_sub_mode = prior.view_sub_mode;
    }
    let _ = comp_id;
    Some(v)
}

// =====================================================================
// World-populator companions
// =====================================================================

use crate::world_pools::WorldPools;

/// Populate an Awards view for the active human's primary competition.
///
/// * `comp_id` ← `WorldPools::active_human_focus_competition` (first
///   manageable club-comp; the exe reads the entity's award-anchor
///   comp on the toolbar, not the pool — see `TODO_POPULATOR_INFO`).
/// * The award-table extractor (`FUN_00414E50`) is not ported yet, so
///   we hand `build_awards` an opaque `|_| 0` — the field then reads
///   as the "no award decoded" sentinel.
pub fn populate_awards(pools: &WorldPools<'_>) -> Option<AwardsView> {
    let comp_id = pools.active_human_focus_competition()?;
    build_awards(true, comp_id, |_| 0)
}

/// Populate a Competition-Dashboard view for the active human's focus
/// competition. `sub_tab` / `secondary_tab` default to -2 (goto-comp
/// entry). `vtable_dispatch` is the exe's `DAT_00AC688C[comp[0]*4]` and
/// isn't decoded on the facade — passes 0.
pub fn populate_competition_dashboard(
    pools: &WorldPools<'_>,
) -> Option<CompetitionDashboardView> {
    let comp_id = pools.active_human_focus_competition()?;
    build_competition_dashboard(true, comp_id, 0, -2, -2, None)
}

#[cfg(test)]
#[allow(dead_code)]
const TODO_POPULATOR_INFO: &str = "\
    AwardsView.award_table needs FUN_00414E50 (award-table extractor) \
    ported — reads the competition record's award/winner slots. \
    CompetitionDashboardView.vtable_dispatch needs DAT_00AC688C[comp[0]*4] \
    (per-comp vtable table) which is not yet on the facade.";

#[cfg(test)]
mod tests {
    use super::*;

    // Awards
    #[test]
    fn awards_nil_comp_returns_none() {
        assert!(build_awards(true, 0, |_| 0).is_none());
    }
    #[test]
    fn awards_registration_failed_returns_none() {
        assert!(build_awards(false, 42, |_| 0).is_none());
    }
    #[test]
    fn awards_populates_from_extractor() {
        let v = build_awards(true, 42, |c| { assert_eq!(c, 42); 0xDEADBEEF }).unwrap();
        assert_eq!(v.award_table, 0xDEADBEEF);
        assert_eq!(v.view_mode, 0);
    }

    // Competition Dashboard
    #[test]
    fn competition_default_slots_match_exe_values() {
        let v = build_competition_dashboard(true, 42, 0xABCD, -2, -2, None).unwrap();
        assert_eq!(v.view_mode, 1);           // slot 1
        assert_eq!(v.sub_tab, -2);            // slot 2
        assert_eq!(v.secondary_tab, -2);      // slot 3
        assert_eq!(v.primary_tab, 0x11);      // slot 0x1E
        assert_eq!(v.view_sub_mode, 3);       // slot 0x23
        assert_eq!(v.season, None);           // slot 0x2A = -1
        assert_eq!(v.filter_flag, 1);         // slot 0x2D
        assert_eq!(v.selected_item, None);    // slot 0x32 = -1
        assert_eq!(v.selected_secondary, None); // slot 0x36 = -1
    }
    #[test]
    fn competition_vtable_dispatch_carried() {
        let v = build_competition_dashboard(true, 42, 0xABCD, -2, -2, None).unwrap();
        assert_eq!(v.vtable_dispatch, 0xABCD);
    }
    #[test]
    fn competition_prior_state_restores_3_slots() {
        let prior = CompetitionDashboardView {
            view_mode: 5, primary_tab: 0x22, view_sub_mode: 7,
            ..Default::default()
        };
        let v = build_competition_dashboard(true, 42, 0xABCD, 0, 0, Some(&prior)).unwrap();
        assert_eq!(v.view_mode, 5);
        assert_eq!(v.primary_tab, 0x22);
        assert_eq!(v.view_sub_mode, 7);
    }
    #[test]
    fn competition_failed_registration_returns_none() {
        assert!(build_competition_dashboard(false, 42, 0, 0, 0, None).is_none());
    }
    #[test]
    fn competition_default_view_is_symmetric() {
        let v = CompetitionDashboardView::default();
        assert_eq!(v.view_mode, 1);
        assert_eq!(v.primary_tab, 0x11);
        assert_eq!(v.view_sub_mode, 3);
    }

    // --- Populators ---
    use crate::world_pools::WorldPools;
    use crate::DomainCompetition;

    fn one_comp() -> Vec<DomainCompetition> {
        vec![DomainCompetition {
            id: 42, long_name: "Prem".into(), short_name: "Prm".into(),
            three_letter_name: "PRM".into(), scope: 2, nation_id: 1,
            last_division: -1, reserve_division: -1, reputation: 100,
            unknown_tail: vec![],
        }]
    }

    #[test]
    fn populate_awards_uses_focus_comp() {
        let comps = one_comp();
        let pools = WorldPools { comps: &comps, ..WorldPools::empty() };
        let v = populate_awards(&pools).unwrap();
        assert_eq!(v.award_table, 0); // extractor stub
        assert_eq!(v.view_mode, 0);
    }

    #[test]
    fn populate_awards_no_comps_yields_none() {
        assert!(populate_awards(&WorldPools::empty()).is_none());
    }

    #[test]
    fn populate_competition_dashboard_uses_focus_comp() {
        let comps = one_comp();
        let pools = WorldPools { comps: &comps, ..WorldPools::empty() };
        let v = populate_competition_dashboard(&pools).unwrap();
        assert_eq!(v.sub_tab, -2);
        assert_eq!(v.secondary_tab, -2);
        assert_eq!(v.primary_tab, 0x11);
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A `_from_pools`)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b5 pool sources: AwardsView.award_table needs FUN_00414E50 \
    (award-table extractor) to read the competition record's \
    award/winner slots — currently returns 0. \
    CompetitionDashboardView.vtable_dispatch needs \
    DAT_00AC688C[comp[0]*4] (per-comp vtable table), also not on the \
    pools facade — pass 0 until decoded.";

pub fn populate_awards_from_pools(pools: &WorldPools<'_>) -> Option<AwardsView> {
    populate_awards(pools)
}

pub fn populate_competition_dashboard_from_pools(
    pools: &WorldPools<'_>,
) -> Option<CompetitionDashboardView> {
    populate_competition_dashboard(pools)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;
    use crate::DomainCompetition;

    fn one_comp() -> Vec<DomainCompetition> {
        vec![DomainCompetition {
            id: 42, long_name: "Prem".into(), short_name: "Prm".into(),
            three_letter_name: "PRM".into(), scope: 2, nation_id: 1,
            last_division: -1, reserve_division: -1, reputation: 100,
            unknown_tail: vec![],
        }]
    }

    #[test]
    fn awards_empty_pools_returns_none() {
        assert!(populate_awards_from_pools(&WorldPools::empty()).is_none());
    }

    #[test]
    fn awards_focus_comp_used() {
        let comps = one_comp();
        let pools = WorldPools { comps: &comps, ..WorldPools::empty() };
        let v = populate_awards_from_pools(&pools).unwrap();
        assert_eq!(v.award_table, 0);
    }

    #[test]
    fn competition_dashboard_focus_comp_used() {
        let comps = one_comp();
        let pools = WorldPools { comps: &comps, ..WorldPools::empty() };
        let v = populate_competition_dashboard_from_pools(&pools).unwrap();
        assert_eq!(v.sub_tab, -2);
        assert_eq!(v.primary_tab, 0x11);
    }

    #[test]
    fn competition_dashboard_empty_pools_returns_none() {
        assert!(populate_competition_dashboard_from_pools(&WorldPools::empty()).is_none());
    }
}
