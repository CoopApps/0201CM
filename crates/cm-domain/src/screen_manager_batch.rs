//! Manager / search / confidence screen builders — batch port of five
//! sidebar-dispatch screens.
//!
//! Decompiles under `d:/cm0102-carve/decompiled/screen_batch2/`:
//! * `0x00693410.c` — Manager Stats (cmd 0x42E, 251 B)
//! * `0x00695E60.c` — Job Information (cmd 0x41E, 244 B)
//! * `0x006977B0.c` — Confidence (cmd 0x420 FA / cmd 0x41F Board, 171 B)
//! * `0x0076FFB0.c` — Staff / News wrapper (cmd 0x3E9, 438 B)
//! * `0x007F0020.c` — Player & Staff Search (cmd 0x3EB, 336 B)
//!
//! All five share the same shape: register callbacks via
//! `FUN_007E6570` (returns 0 on registration fail → no-op), then push
//! a fixed set of slots via `FUN_007E7130` (global) or `FUN_007E7000`
//! (per-entity). Every slot is named and typed here.

use serde::{Deserialize, Serialize};

// =====================================================================
// Manager Stats — cmd 0x42E → FUN_00693410
// =====================================================================

/// Player-resolver input for Manager Stats: exe reads the current
/// human's nation-club at `+0x39`, backup club at `+0x24`, then staff
/// slot at `+0x1A` gated by `+0x47 & 2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ManagerContextInput {
    /// `human+0x39` → nation-primary-club record; if present, use
    /// `club+0x53` (nation ptr) → first int (nation id).
    pub nation_primary_club_nation_id: Option<u32>,
    /// `human+0x24` → backup club nation id (same path).
    pub backup_club_nation_id: Option<u32>,
    /// `human+0x1A` → staff slot ptr; used when its `+0x47 & 2` flag
    /// is set. Returns the staff slot's first int (person id).
    pub staff_slot_person_id: Option<(u32, bool)>,
}

/// Manager Stats screen view — 9 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagerStatsView {
    /// Slot 0: resolved nation/person id, -1 if none.
    pub context_id: Option<u32>,
    /// Slot 3: 1 (sort-order flag).
    pub sort_order: u8,
    /// Slot 4: 1 (season-view flag).
    pub season_view: u8,
    /// Slot 7: 1 (per-comp filter).
    pub comp_filter: u8,
    /// Slot 8: 1 (aggregate-view flag).
    pub aggregate: u8,
}

impl Default for ManagerStatsView {
    fn default() -> Self {
        Self { context_id: None, sort_order: 1, season_view: 1,
               comp_filter: 1, aggregate: 1 }
    }
}

/// Direct port of `FUN_00693410`. Resolves the display context via
/// the exe's 3-priority cascade: nation-club > backup-club > staff-slot.
pub fn build_manager_stats(
    registration_ok: bool,
    ctx: ManagerContextInput,
) -> Option<ManagerStatsView> {
    if !registration_ok { return None; }
    let id = ctx.nation_primary_club_nation_id
        .or(ctx.backup_club_nation_id)
        .or_else(|| ctx.staff_slot_person_id.and_then(
            |(id, gate)| if gate { Some(id) } else { None }));
    Some(ManagerStatsView { context_id: id, ..Default::default() })
}

// =====================================================================
// Job Information — cmd 0x41E → FUN_00695E60
// =====================================================================

/// Job Info view — 8 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobInfoView {
    /// Slot 0: resolved club/nation context, -1 if none.
    pub context_id: Option<u32>,
    /// Slot 3: 1 (view-mode flag).
    pub view_mode: u8,
    /// Slot 4: 1 (season-filter flag).
    pub season_filter: u8,
    /// Slot 7: 1 (comp-filter flag).
    pub comp_filter: u8,
}

impl Default for JobInfoView {
    fn default() -> Self {
        Self { context_id: None, view_mode: 1, season_filter: 1, comp_filter: 1 }
    }
}

/// Job-info resolver input — slightly different priority order from
/// Manager Stats: nation-primary → staff-slot → backup-club.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JobInfoInput {
    pub nation_primary_club_nation_id: Option<u32>,
    /// `human+0x1A` staff slot with gate check.
    pub staff_slot_person_id: Option<(u32, bool)>,
    /// `human+0x24` backup — additionally reads +0x53 nation ptr and
    /// checks `+0x47 & 2` on IT before using.
    pub backup_club_gated: Option<(u32, bool)>,
}

/// Direct port of `FUN_00695E60`.
pub fn build_job_info(
    registration_ok: bool,
    ctx: JobInfoInput,
) -> Option<JobInfoView> {
    if !registration_ok { return None; }
    let id = ctx.nation_primary_club_nation_id
        .or_else(|| ctx.staff_slot_person_id.and_then(
            |(id, gate)| if gate { Some(id) } else { None }))
        .or_else(|| ctx.backup_club_gated.and_then(
            |(id, gate)| if gate { Some(id) } else { None }));
    Some(JobInfoView { context_id: id, ..Default::default() })
}

// =====================================================================
// Confidence (FA / Board) — cmd 0x420 / 0x41F → FUN_006977B0
// =====================================================================

/// FA / Board confidence view — single slot, generic.
///
/// The exe's `FUN_006977B0` fires with either the human's nation id
/// (cmd 0x420 FA) or club id (cmd 0x41F Board). Same body either way.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfidenceView {
    /// Slot 0: the club or nation id whose confidence board to show.
    pub context_id: u32,
    /// Which confidence board: FA (nation) or Board (club).
    pub kind: ConfidenceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceKind { FaNation, BoardClub }

/// Direct port of `FUN_006977B0(param_1)`. Errors on param==0 (exe
/// fires MsgBox `manager_screens:0x5FC`).
pub fn build_confidence(
    registration_ok: bool,
    context_id: u32,
    kind: ConfidenceKind,
) -> Option<ConfidenceView> {
    if context_id == 0 { return None; }
    if !registration_ok { return None; }
    Some(ConfidenceView { context_id, kind })
}

// =====================================================================
// Staff / News wrapper — cmd 0x3E9 → FUN_0076FFB0
// =====================================================================

/// Staff wrapper view — 0x21 slots via per-entity `FUN_007E7000`.
/// The exe allocates a 30-byte "search string" scratch buffer at slot
/// 8, initialised to `"____234________..."` and passed with the
/// slot-write flag=1 (meaning "owned pointer, free on close").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffView {
    /// Slot 0: additional context payload from param_2.
    pub context_param: u32,
    /// Slot 8: search-string scratch (30 bytes, initialised to spaces
    /// then overwritten by `FUN_00933D2F` with an `s_____234...` pattern).
    pub search_scratch: [u8; 30],
    // Slots 1..7 and 9..0x20 are all zero-init.
}

impl Default for StaffView {
    fn default() -> Self {
        let mut scratch = [b' '; 30];
        // The exe writes a specific "____234________..." template.
        // Preserve the first 4 underscores and trailing null.
        scratch[0..4].copy_from_slice(b"____");
        scratch[4..7].copy_from_slice(b"234");
        scratch[7..15].copy_from_slice(b"________");
        scratch[29] = 0;
        Self { context_param: 0, search_scratch: scratch }
    }
}

/// Direct port of `FUN_0076FFB0(param_1, param_2)`.
///
/// * `param_1` is the entity ptr (opaque here — we take a bool for
///   the registration success gate + the caller's context).
/// * `param_2` is passed straight into slot 0.
pub fn build_staff_screen(
    registration_ok: bool,
    allocation_ok: bool,
    context_param: u32,
) -> Option<StaffView> {
    if !registration_ok { return None; }
    if !allocation_ok { return None; }   // exe fires MsgBox news_screens:0x61
    Some(StaffView { context_param, ..Default::default() })
}

// =====================================================================
// Player & Staff Search — cmd 0x3EB → FUN_007F0020
// =====================================================================

/// Player/Staff Search view — 12 slots.
///
/// The exe allocates a 0x3638 (13,880) byte search-context struct on
/// entry and populates slot 0 with its return from `FUN_007EBCA0`
/// (search-context initialiser). Slots 8, 9, 10 are sentinel `-1`s;
/// slot 7 = 1; rest = 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerSearchView {
    /// Slot 0: search-context handle (result of `FUN_007EBCA0`).
    pub search_context: Option<u32>,
    /// Slot 7: 1 (search-active flag).
    pub search_active: u8,
    /// Slot 8: filter1, -1 = no filter.
    pub filter1: Option<u32>,
    /// Slot 9: filter2.
    pub filter2: Option<u32>,
    /// Slot 10: filter3.
    pub filter3: Option<u32>,
}

impl Default for PlayerSearchView {
    fn default() -> Self {
        Self {
            search_context: None, search_active: 1,
            filter1: None, filter2: None, filter3: None,
        }
    }
}

/// Direct port of `FUN_007F0020(param_1)`. Allocates the 13,880-byte
/// search-context (modelled by `allocation_ok`); on alloc failure the
/// search_context is written as 0.
pub fn build_player_search(
    registration_ok: bool,
    allocation_ok: bool,
    param_1: u32,
    context_init: impl FnOnce(u32) -> u32,
) -> Option<PlayerSearchView> {
    if !registration_ok { return None; }
    let ctx = if allocation_ok { Some(context_init(param_1)) } else { None };
    Some(PlayerSearchView {
        search_context: ctx,
        search_active: 1,
        filter1: None, filter2: None, filter3: None,
    })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_stats_nil_registration_returns_none() {
        assert!(build_manager_stats(false, ManagerContextInput::default()).is_none());
    }

    #[test]
    fn manager_stats_nation_primary_wins() {
        let v = build_manager_stats(true, ManagerContextInput {
            nation_primary_club_nation_id: Some(100),
            backup_club_nation_id: Some(200),
            staff_slot_person_id: Some((300, true)),
        }).unwrap();
        assert_eq!(v.context_id, Some(100));
    }

    #[test]
    fn manager_stats_falls_to_backup_when_primary_absent() {
        let v = build_manager_stats(true, ManagerContextInput {
            nation_primary_club_nation_id: None,
            backup_club_nation_id: Some(200),
            staff_slot_person_id: None,
        }).unwrap();
        assert_eq!(v.context_id, Some(200));
    }

    #[test]
    fn manager_stats_staff_slot_gated_by_flag() {
        // gate=false → staff slot is skipped.
        let v = build_manager_stats(true, ManagerContextInput {
            nation_primary_club_nation_id: None,
            backup_club_nation_id: None,
            staff_slot_person_id: Some((300, false)),
        }).unwrap();
        assert_eq!(v.context_id, None);
    }

    #[test]
    fn manager_stats_default_slot_values_match_exe() {
        let v = build_manager_stats(true, ManagerContextInput::default()).unwrap();
        assert_eq!(v.sort_order, 1);
        assert_eq!(v.season_view, 1);
        assert_eq!(v.comp_filter, 1);
        assert_eq!(v.aggregate, 1);
    }

    #[test]
    fn job_info_uses_different_priority_from_manager_stats() {
        // Priority: nation_primary > staff_slot > backup.
        let v = build_job_info(true, JobInfoInput {
            nation_primary_club_nation_id: None,
            staff_slot_person_id: Some((200, true)),
            backup_club_gated: Some((300, true)),
        }).unwrap();
        assert_eq!(v.context_id, Some(200), "staff slot should win over backup");
    }

    #[test]
    fn job_info_backup_gate_respected() {
        let v = build_job_info(true, JobInfoInput {
            nation_primary_club_nation_id: None,
            staff_slot_person_id: None,
            backup_club_gated: Some((300, false)),
        }).unwrap();
        assert_eq!(v.context_id, None);
    }

    #[test]
    fn confidence_nil_context_returns_none() {
        assert!(build_confidence(true, 0, ConfidenceKind::FaNation).is_none());
    }

    #[test]
    fn confidence_carries_kind() {
        let fa = build_confidence(true, 42, ConfidenceKind::FaNation).unwrap();
        let bd = build_confidence(true, 42, ConfidenceKind::BoardClub).unwrap();
        assert_eq!(fa.kind, ConfidenceKind::FaNation);
        assert_eq!(bd.kind, ConfidenceKind::BoardClub);
        assert_eq!(fa.context_id, bd.context_id);
    }

    #[test]
    fn staff_view_default_scratch_is_30_bytes() {
        let d = StaffView::default();
        assert_eq!(d.search_scratch.len(), 30);
        assert_eq!(d.search_scratch[29], 0, "trailing null");
    }

    #[test]
    fn staff_view_alloc_failure_returns_none() {
        assert!(build_staff_screen(true, false, 42).is_none());
    }

    #[test]
    fn staff_view_context_param_carries_through() {
        let v = build_staff_screen(true, true, 999).unwrap();
        assert_eq!(v.context_param, 999);
    }

    #[test]
    fn player_search_alloc_failure_leaves_context_none() {
        let v = build_player_search(true, false, 0, |_| 0).unwrap();
        assert_eq!(v.search_context, None);
        assert_eq!(v.search_active, 1);
    }

    #[test]
    fn player_search_success_populates_context_from_init() {
        let v = build_player_search(true, true, 42,
            |p| { assert_eq!(p, 42); 0xDEADBEEF }).unwrap();
        assert_eq!(v.search_context, Some(0xDEADBEEF));
    }

    #[test]
    fn player_search_default_filters_are_sentinels() {
        let v = build_player_search(true, true, 0, |_| 1).unwrap();
        assert_eq!(v.filter1, None);   // slot 8 = -1
        assert_eq!(v.filter2, None);   // slot 9 = -1
        assert_eq!(v.filter3, None);   // slot 10 = -1
    }
}
