//! 5-screen batch: latest_scores, manager_history, go_holiday,
//! fifa_rankings, uefa_coefficients.
//!
//! Decompiles under `d:/cm0102-carve/decompiled/screen_batch3/`:
//! * `0x00700F20.c` — Latest Scores (cmd 0x418, 325 B, per-entity)
//! * `0x00859250.c` — Manager History (cmd 0x3EC, 929 B)
//! * `0x006986A0.c` — Go on Holiday dialog (cmd 0x3EF, 27 B — thinnest wrapper)
//! * `0x004A2190.c` — FIFA Rankings (cmd 0x3F3, 98 B)
//! * `0x004A28C0.c` — UEFA Coefficients (cmd 0x40C, 63 B)

use serde::{Deserialize, Serialize};

// =====================================================================
// Latest Scores — cmd 0x418 → FUN_00700F20(entity, param_2)
// =====================================================================

/// Latest Scores view — 8 slots via per-entity `FUN_007E7000`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestScoresView {
    /// Slot 0: 0 (view-mode reset).
    pub view_mode: u32,
    /// Slot 0xE: param_2 (comp id — the current focus competition).
    pub focus_competition: u32,
    /// Slot 0xF: 0 (day offset).
    pub day_offset: i32,
    /// Slot 0x10: -1 (sentinel for "no comp selected").
    pub selected_comp: Option<u32>,
    /// Slot 0x11: 1 (result-column flag).
    pub result_columns: u8,
    /// Slot 0x12: 0 (scroll-offset).
    pub scroll_offset: u32,
    /// Slot 0x13: 1 (group-by-comp flag).
    pub group_by_comp: u8,
    /// Slot 0x14: 0.
    pub reserved: u32,
}

impl Default for LatestScoresView {
    fn default() -> Self {
        Self {
            view_mode: 0, focus_competition: 0, day_offset: 0,
            selected_comp: None, result_columns: 1, scroll_offset: 0,
            group_by_comp: 1, reserved: 0,
        }
    }
}

/// Direct port of `FUN_00700F20`. Errors on either arg == 0 (exe fires
/// MsgBox `match_screens:0x7A1`).
pub fn build_latest_scores(
    registration_ok: bool,
    entity_id: u32, focus_competition: u32,
) -> Option<LatestScoresView> {
    if entity_id == 0 || focus_competition == 0 { return None; }
    if !registration_ok { return None; }
    Some(LatestScoresView { focus_competition, ..Default::default() })
}

// =====================================================================
// Manager History — cmd 0x3EC → FUN_00859250(person, param_2, param_3, prev_state, param_5)
// =====================================================================

/// Manager History view. The exe function is 929 bytes with a big
/// eligibility check at the top, then registers callbacks and pushes
/// slots. When `prev_state != 0` it restores 60 dwords via `FUN_007E6EE0`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagerHistoryView {
    /// Person id whose career history to show.
    pub person_id: u32,
    /// Slot 1: history mode byte (matches param_2 — 5 = full career).
    pub history_mode: i8,
    /// Slot 2: sub-mode byte (matches param_3 — 1 = default).
    pub sub_mode: i8,
    /// Slot 5: additional filter byte (param_5).
    pub extra_filter: i8,
}

impl Default for ManagerHistoryView {
    fn default() -> Self {
        Self { person_id: 0, history_mode: 5, sub_mode: 1, extra_filter: 0 }
    }
}

/// Direct port of `FUN_00859250` (essential path). The exe's eligibility
/// gate at the top is a complex nested check on `person+0x3D`
/// (job-status) and person id range vs `DAT_00ACD56C` (max staff id).
/// We model the gate as an `eligible` boolean the caller supplies
/// (they've already run the same person-vs-staff-pool check).
pub fn build_manager_history(
    person_id: u32,
    eligible: bool,
    history_mode: i8,
    sub_mode: i8,
    extra_filter: i8,
) -> Option<ManagerHistoryView> {
    if person_id == 0 || !eligible { return None; }
    Some(ManagerHistoryView { person_id, history_mode, sub_mode, extra_filter })
}

/// Port of the exe's eligibility check at lines 21-31 — the person's
/// `+0x3D` (job_status) must NOT equal 2 (unemployed-not-registered)
/// AND their id must fall inside the current staff pool bounds.
pub fn manager_history_eligibility(
    person_id: u32, job_status: i8,
    staff_pool_max_id: u32, staff_pool_window: u32,
) -> bool {
    // Exe: `id < DAT_00acd56c && id >= DAT_00acd56c - 0x10` — inside window.
    let below_max = person_id < staff_pool_max_id;
    let above_min = person_id >= staff_pool_max_id.saturating_sub(staff_pool_window);
    let job_valid = job_status != 2;
    job_valid && above_min && below_max
}

// =====================================================================
// Go on Holiday — cmd 0x3EF → FUN_006986A0 (dialog trigger, 27 B)
// =====================================================================

/// Go-on-Holiday dialog handle — the exe's `FUN_006986A0` just calls
/// `FUN_007E6570(FUN_006986C0, LAB_00698E70, 1, 0, 0)` — one-shot
/// dialog registration with no data slots. There's nothing to populate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GoHolidayDialog { pub opened: bool }

/// Direct port of `FUN_006986A0`. Returns `true` when the dialog
/// registered successfully.
pub fn build_go_holiday_dialog(registration_ok: bool) -> GoHolidayDialog {
    GoHolidayDialog { opened: registration_ok }
}

// =====================================================================
// FIFA Rankings — cmd 0x3F3 → FUN_004A2190 (98 B, 4 slots)
// =====================================================================

/// FIFA Rankings view — 4 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifaRankingsView {
    /// Slot 0: 0 (selected-nation filter, 0 = all).
    pub selected_nation: u32,
    /// Slot 1: 0 (continent filter).
    pub continent_filter: u32,
    /// Slot 2: 1 (sort-by-points flag).
    pub sort_by_points: u8,
    /// Slot 3: 1 (show-continent-averages flag).
    pub show_averages: u8,
}

impl Default for FifaRankingsView {
    fn default() -> Self {
        Self { selected_nation: 0, continent_filter: 0,
               sort_by_points: 1, show_averages: 1 }
    }
}

/// Direct port of `FUN_004A2190`.
pub fn build_fifa_rankings(registration_ok: bool) -> Option<FifaRankingsView> {
    if !registration_ok { return None; }
    Some(FifaRankingsView::default())
}

// =====================================================================
// UEFA Coefficients — cmd 0x40C → FUN_004A28C0 (63 B, 2 slots)
// =====================================================================

/// UEFA Coefficients view — 2 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UefaCoefficientsView {
    /// Slot 0: 0 (selected-nation filter).
    pub selected_nation: u32,
    /// Slot 1: 0 (season offset).
    pub season_offset: i32,
}

impl Default for UefaCoefficientsView {
    fn default() -> Self { Self { selected_nation: 0, season_offset: 0 } }
}

/// Direct port of `FUN_004A28C0`.
pub fn build_uefa_coefficients(registration_ok: bool) -> Option<UefaCoefficientsView> {
    if !registration_ok { return None; }
    Some(UefaCoefficientsView::default())
}

// =====================================================================
// World-populator companions
// =====================================================================
//
// These wrap the `build_*` fns above with a facade lookup on the live
// world pools (see [`crate::world_pools::WorldPools`]) instead of taking
// opaque primitive params. They mirror the exe's own "read from
// DAT_*/pool" step that runs before each screen setup.

use crate::world_pools::WorldPools;

/// Populate a Latest-Scores view from the world.
///
/// * Entity id ← active human's club id (`WorldPools::active_human_club_id`,
///   which reads `staff[seat].current_club_id` at body+0x35).
/// * Focus competition ← the human's manageable-league id
///   (`WorldPools::active_human_focus_competition`).
///
/// Registration is assumed to have succeeded — matches the caller's
/// pattern in the sidebar dispatcher (see the club-toolbar path 004551c0).
pub fn populate_latest_scores(pools: &WorldPools<'_>) -> Option<LatestScoresView> {
    let entity_id = pools.active_human_club_id()?;
    let focus = pools.active_human_focus_competition()?;
    build_latest_scores(true, entity_id, focus)
}

/// Populate a Manager-History view from the world.
///
/// * Person id ← active human seat pointer (`DAT_00b5d016`).
/// * Eligibility ← always `true` here (the active seat is by
///   construction in-window and job_status != 2 — the exe check).
pub fn populate_manager_history(pools: &WorldPools<'_>) -> Option<ManagerHistoryView> {
    let person_id = pools.active_human_seat?;
    build_manager_history(person_id, true, 5, 1, 0)
}

/// Populate a Go-Holiday dialog. The exe's `FUN_006986A0` has no data
/// slots — this is a shape-check with `registration_ok=true`.
pub fn populate_go_holiday(_pools: &WorldPools<'_>) -> GoHolidayDialog {
    build_go_holiday_dialog(true)
}

/// Populate a FIFA-Rankings view. All 4 slots are zero/default — the
/// exe reads no per-caller pool state here.
pub fn populate_fifa_rankings(_pools: &WorldPools<'_>) -> Option<FifaRankingsView> {
    build_fifa_rankings(true)
}

/// Populate a UEFA-Coefficients view. Same shape as FIFA rankings —
/// pure default slots.
pub fn populate_uefa_coefficients(_pools: &WorldPools<'_>) -> Option<UefaCoefficientsView> {
    build_uefa_coefficients(true)
}

#[cfg(test)]
#[allow(dead_code)]
const TODO_POPULATOR_INFO: &str = "\
    LatestScoresView.focus_competition should chase club+0x27 (nation) → \
    nation's primary league id; approximated here by first manageable \
    league. Manager-history eligibility gate (staff_pool_max / window) is \
    not decoded on the facade yet.";

#[cfg(test)]
mod tests {
    use super::*;

    // --- Latest Scores ---
    #[test]
    fn latest_scores_nil_args_returns_none() {
        assert!(build_latest_scores(true, 0, 100).is_none());
        assert!(build_latest_scores(true, 100, 0).is_none());
    }
    #[test]
    fn latest_scores_populates_focus_and_defaults() {
        let v = build_latest_scores(true, 42, 100).unwrap();
        assert_eq!(v.focus_competition, 100);
        assert_eq!(v.selected_comp, None);   // slot 0x10 = -1
        assert_eq!(v.result_columns, 1);     // slot 0x11
        assert_eq!(v.group_by_comp, 1);      // slot 0x13
    }

    // --- Manager History ---
    #[test]
    fn manager_history_eligibility_matches_exe_gate() {
        // job_status == 2 → never eligible.
        assert!(!manager_history_eligibility(50, 2, 100, 16));
        // Within window (max=100, window=16 → id must be ≥ 84 OR < 100).
        assert!(manager_history_eligibility(90, 1, 100, 16));
        // Above max fails.
        assert!(!manager_history_eligibility(150, 1, 100, 16));
    }
    #[test]
    fn manager_history_nil_person_returns_none() {
        assert!(build_manager_history(0, true, 5, 1, 0).is_none());
    }
    #[test]
    fn manager_history_ineligible_returns_none() {
        assert!(build_manager_history(42, false, 5, 1, 0).is_none());
    }
    #[test]
    fn manager_history_populates_all_flags() {
        let v = build_manager_history(42, true, 5, 1, 7).unwrap();
        assert_eq!(v.person_id, 42);
        assert_eq!(v.history_mode, 5);
        assert_eq!(v.sub_mode, 1);
        assert_eq!(v.extra_filter, 7);
    }

    // --- Go Holiday ---
    #[test]
    fn go_holiday_opens_on_registration_ok() {
        assert!(build_go_holiday_dialog(true).opened);
        assert!(!build_go_holiday_dialog(false).opened);
    }

    // --- FIFA Rankings ---
    #[test]
    fn fifa_rankings_defaults_match_exe_slots() {
        let v = build_fifa_rankings(true).unwrap();
        assert_eq!(v.selected_nation, 0);   // slot 0
        assert_eq!(v.continent_filter, 0);  // slot 1
        assert_eq!(v.sort_by_points, 1);    // slot 2
        assert_eq!(v.show_averages, 1);     // slot 3
    }
    #[test]
    fn fifa_rankings_failed_registration_none() {
        assert!(build_fifa_rankings(false).is_none());
    }

    // --- UEFA Coefficients ---
    #[test]
    fn uefa_coefficients_defaults_match_exe_slots() {
        let v = build_uefa_coefficients(true).unwrap();
        assert_eq!(v.selected_nation, 0);
        assert_eq!(v.season_offset, 0);
    }
    #[test]
    fn uefa_coefficients_failed_registration_none() {
        assert!(build_uefa_coefficients(false).is_none());
    }

    // --- Populators ---
    use crate::world_pools::WorldPools;
    use crate::{DomainCompetition, DomainStaffType6};

    fn fixture_pools() -> (Vec<DomainStaffType6>, Vec<DomainCompetition>) {
        // Person id 42 → club 100 via body+0x35 (u32 LE at 0x35).
        let mut body = vec![0u8; 0x60];
        body[0x35..0x39].copy_from_slice(&100u32.to_le_bytes());
        let staff = vec![DomainStaffType6 { id: 42, body }];
        let comps = vec![DomainCompetition {
            id: 7, long_name: "Prem".into(), short_name: "PRM".into(),
            three_letter_name: "PRM".into(), scope: 2, nation_id: 1,
            last_division: -1, reserve_division: -1, reputation: 100,
            unknown_tail: vec![],
        }];
        (staff, comps)
    }

    #[test]
    fn populate_latest_scores_uses_seat_club_and_first_league() {
        let (staff, comps) = fixture_pools();
        let pools = WorldPools {
            clubs: &[], nations: &[], staff: &staff, players: &[],
            comps: &comps, active_human_seat: Some(42),
        };
        let v = populate_latest_scores(&pools).unwrap();
        assert_eq!(v.focus_competition, 7);
        // Sanity check: default view slots still hold.
        assert_eq!(v.result_columns, 1);
        assert_eq!(v.group_by_comp, 1);
    }

    #[test]
    fn populate_latest_scores_empty_world_yields_none() {
        assert!(populate_latest_scores(&WorldPools::empty()).is_none());
    }

    #[test]
    fn populate_manager_history_uses_seat_id() {
        let pools = WorldPools { active_human_seat: Some(42), ..WorldPools::empty() };
        let v = populate_manager_history(&pools).unwrap();
        assert_eq!(v.person_id, 42);
        assert_eq!(v.history_mode, 5);
    }

    #[test]
    fn populate_manager_history_no_seat_yields_none() {
        assert!(populate_manager_history(&WorldPools::empty()).is_none());
    }

    #[test]
    fn populate_go_holiday_opens() {
        assert!(populate_go_holiday(&WorldPools::empty()).opened);
    }

    #[test]
    fn populate_fifa_rankings_defaults() {
        let v = populate_fifa_rankings(&WorldPools::empty()).unwrap();
        assert_eq!(v.sort_by_points, 1);
        assert_eq!(v.show_averages, 1);
    }

    #[test]
    fn populate_uefa_coefficients_defaults() {
        let v = populate_uefa_coefficients(&WorldPools::empty()).unwrap();
        assert_eq!(v.season_offset, 0);
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A `_from_pools`)
// =====================================================================
//
// These mirror the `_from_pools` aliases used by screen_batch{10..17} so
// every wave-A batch exposes both the short `populate_*` name and the
// explicit `populate_*_from_pools` name. They delegate to the existing
// pool-based populators above.

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b3 pool sources: LatestScoresView.focus_competition should chase \
    club+0x27 (nation) -> nation.primary_league; approximated by the \
    first manageable club-comp (three_letter_name populated). \
    ManagerHistoryView eligibility gate (staff_pool_max / window) is \
    not decoded on the pools facade yet — the populator trusts the \
    seat pointer. FIFA/UEFA/GoHoliday have no per-caller pool state.";

pub fn populate_latest_scores_from_pools(pools: &WorldPools<'_>) -> Option<LatestScoresView> {
    populate_latest_scores(pools)
}

pub fn populate_manager_history_from_pools(pools: &WorldPools<'_>) -> Option<ManagerHistoryView> {
    populate_manager_history(pools)
}

pub fn populate_go_holiday_from_pools(pools: &WorldPools<'_>) -> GoHolidayDialog {
    populate_go_holiday(pools)
}

pub fn populate_fifa_rankings_from_pools(pools: &WorldPools<'_>) -> Option<FifaRankingsView> {
    populate_fifa_rankings(pools)
}

pub fn populate_uefa_coefficients_from_pools(pools: &WorldPools<'_>) -> Option<UefaCoefficientsView> {
    populate_uefa_coefficients(pools)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn latest_scores_empty_pools_returns_none() {
        assert!(populate_latest_scores_from_pools(&WorldPools::empty()).is_none());
    }

    #[test]
    fn manager_history_seat_returns_view() {
        let p = WorldPools { active_human_seat: Some(11), ..WorldPools::empty() };
        let v = populate_manager_history_from_pools(&p).unwrap();
        assert_eq!(v.person_id, 11);
        assert_eq!(v.history_mode, 5);
    }

    #[test]
    fn go_holiday_opens_on_empty_pools() {
        assert!(populate_go_holiday_from_pools(&WorldPools::empty()).opened);
    }

    #[test]
    fn fifa_rankings_defaults() {
        let v = populate_fifa_rankings_from_pools(&WorldPools::empty()).unwrap();
        assert_eq!(v.sort_by_points, 1);
        assert_eq!(v.show_averages, 1);
    }

    #[test]
    fn uefa_coefficients_defaults() {
        let v = populate_uefa_coefficients_from_pools(&WorldPools::empty()).unwrap();
        assert_eq!(v.season_offset, 0);
        assert_eq!(v.selected_nation, 0);
    }
}
