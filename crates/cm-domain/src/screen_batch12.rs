//! Batch 12: 5 more setup functions ported from cm0102.exe.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `00478a30.c` — Club Squad screen (19 slots, 0..0x12) — gated on
//!   `FUN_0076eb10` license/data check; on failure fires the club-load
//!   error MsgBox and clears `DAT_00b4d5a8`.
//! * `00480130.c` — Related club-list screen (10 slots, 0..9); tail slot
//!   9 is the literal `0x154e21` sentinel.
//! * `00490410.c` — Club Rivals screen: builds a `local_18[6]` list of
//!   up-to-6 peer clubs (parent + 5 in slots +0x19f..) from param_2,
//!   filters out `param_1` and clubs with `+0x69 == 0`, computes the
//!   selected index as `1 << pos`, only pushes 3 slots (0..2).
//! * `0049c9a0.c` — **SKIPPED.** Not a screen setup — this is the
//!   post-selection *event handler* for an already-registered screen.
//!   Reads slots via `FUN_007e6ee0` and pushes replacement slots via
//!   `FUN_007e7130`; the only `FUN_007e6570` call inside is a nested
//!   *sub*-screen (LAB_004a1950) launched from one branch. Stub only.
//! * `0049e0d0.c` — Generic filter dialog (26 slots, 0..0x19) with an
//!   optional "restore previous filter" flow when `param_2 != 0` (which
//!   copies 60 dwords out of the scratch pool, then reapplies 7 of them
//!   post-registration).

use serde::{Deserialize, Serialize};

// =====================================================================
// Club Squad — FUN_00478a30(param_1, param_2)
// =====================================================================

/// Club Squad view — 19 slots (0..=0x12).
///
/// Slot map (post-registration, from decompile):
/// * 0: param_1 (club id)
/// * 1: param_2 (secondary id / view mode)
/// * 2: 2 (view = squad)
/// * 3: 0
/// * 4: 0x41
/// * 5: 8
/// * 6: 6
/// * 7, 8: 0
/// * 9, 10: -1 (0xffffffff)
/// * 0xb..0xd: three tri-state flags from `FUN_0076d7d0(2/4/5)`
/// * 0xe..0x12: 0
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubSquadView {
    pub club_id: u32,
    pub secondary: u32,
    pub view_mode: u32,
    pub flag_a: i8,
    pub flag_b: i8,
    pub flag_c: i8,
}

/// Direct port of `FUN_00478a30`.
///
/// * `licensed_ok` — the exe's `FUN_0076eb10` return; on `false` the
///   exe pops the club-load error MsgBox and does NOT push any slots.
/// * `registration_ok` — result of `FUN_007e6570`.
/// * `flag_a/b/c` — results of `FUN_0076d7d0(2/4/5)` (char return).
pub fn build_club_squad(
    licensed_ok: bool,
    registration_ok: bool,
    param_1: u32,
    param_2: u32,
    flag_a: i8,
    flag_b: i8,
    flag_c: i8,
) -> Option<ClubSquadView> {
    if !licensed_ok { return None; }
    if !registration_ok { return None; }
    Some(ClubSquadView {
        club_id: param_1,
        secondary: param_2,
        view_mode: 2,
        flag_a, flag_b, flag_c,
    })
}

// =====================================================================
// Related club-list — FUN_00480130(param_1)
// =====================================================================

/// Related club-list view — 10 slots (0..=9).
///
/// Slot map:
/// * 0: param_1
/// * 1..=3: 0
/// * 4: 0x41
/// * 5: 8
/// * 6: 6
/// * 7, 8: -1 (0xffffffff)
/// * 9: 0x154e21 (magic tail sentinel)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedClubListView {
    pub anchor: u32,
}

pub const RELATED_CLUB_LIST_TAIL_SENTINEL: u32 = 0x154e21;

/// Direct port of `FUN_00480130`.
pub fn build_related_club_list(
    registration_ok: bool,
    param_1: u32,
) -> Option<RelatedClubListView> {
    if !registration_ok { return None; }
    Some(RelatedClubListView { anchor: param_1 })
}

// =====================================================================
// Club Rivals — FUN_00490410(param_1, param_2)
// =====================================================================

/// Club Rivals view — 3 slots (0..=2).
///
/// * Slot 0: `*param_1` (this club's id).
/// * Slot 1: `1 << position_of(param_2 in rivals[])` — bitmask marking
///   the currently-selected rival.
/// * Slot 2: `*peer` where `peer = FUN_0052a5a0(puVar8, ..., 1)` if that
///   returned a valid result; else the last matching rival record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubRivalsView {
    pub club_id: u32,
    pub selected_bitmask: u32,
    pub peer_id: u32,
}

/// Direct port of `FUN_00490410`.
///
/// The exe collects up to 6 candidate rival records (parent-club at
/// `+0xd3` plus 5 slots at `+0x19f..`), filtering out `param_1` itself
/// and any record with `+0x69 == 0`. If no rivals survive, no slots are
/// pushed and we return `None`.
///
/// * `club_id` — `*param_1`.
/// * `selected_position` — index (0..) of `param_2` in the filtered
///   rivals list, or `None` if not present (bitmask stays 0, as in exe).
/// * `rival_count` — length of the filtered list (>=1 required).
/// * `peer_id` — the id pushed to slot 2 (see docstring above).
pub fn build_club_rivals(
    registration_ok: bool,
    club_id: u32,
    selected_position: Option<u8>,
    rival_count: u8,
    peer_id: u32,
) -> Option<ClubRivalsView> {
    if !registration_ok { return None; }
    if rival_count == 0 { return None; }
    let selected_bitmask = match selected_position {
        Some(p) if (p as u8) < rival_count => 1u32 << (p & 0x1f),
        _ => 0,
    };
    Some(ClubRivalsView { club_id, selected_bitmask, peer_id })
}

// =====================================================================
// 0049c9a0 — SKIPPED (event handler, not a screen setup)
// =====================================================================

/// **Stub — not a screen builder.**
///
/// `FUN_0049c9a0(param_1)` is the post-selection event handler for a
/// screen that was registered elsewhere. It reads slots via
/// `FUN_007e6ee0` and mutates them via `FUN_007e7130` in-place; the
/// single `FUN_007e6570` call inside opens a *sub*-screen (LAB_004a1950)
/// from one branch of the state machine. It does not build a top-level
/// view of its own, so there is no `build_*` port here.
#[doc(hidden)]
pub fn _skip_fun_0049c9a0() {}

// =====================================================================
// Generic Filter Dialog — FUN_0049e0d0(param_1, param_2)
// =====================================================================

/// Filter-dialog view — 26 slots (0..=0x19).
///
/// Fixed slot values (post-registration):
/// * 0: `*(DAT_00ac688c + *param_1 * 4)` — resolved entity id.
/// * 1: 1  (may be overridden by restore)
/// * 2, 3: -1
/// * 4, 5: 0
/// * 6: 1  (may be overridden by restore)
/// * 7..0xa: 0
/// * 0xb: 6 (may be overridden by restore)
/// * 0xc, 0xd: 0
/// * 0xe: 9 (may be overridden by restore)
/// * 0xf, 0x10: 0
/// * 0x11: 0xb (may be overridden by restore)
/// * 0x12, 0x13: 0
/// * 0x14: 3 (may be overridden by restore)
/// * 0x15, 0x16: 0
/// * 0x17: 0xf (may be overridden by restore)
/// * 0x18, 0x19: 0
///
/// When `restore_from_scratch` is provided, the 7 saved slots
/// (1, 6, 0xb, 0xe, 0x11, 0x14, 0x17) are overwritten with the caller's
/// snapshot values (the exe pre-reads 60 dwords from the scratch pool
/// and reapplies these seven).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterDialogView {
    pub entity_id: u32,
    pub slot_01: u32,
    pub slot_06: u32,
    pub slot_0b: u32,
    pub slot_0e: u32,
    pub slot_11: u32,
    pub slot_14: u32,
    pub slot_17: u32,
}

/// Restore-slot snapshot: values that override the 7 tunable slots
/// when the dialog is re-opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FilterDialogRestore {
    pub s01: u32, pub s06: u32, pub s0b: u32, pub s0e: u32,
    pub s11: u32, pub s14: u32, pub s17: u32,
}

/// Direct port of `FUN_0049e0d0`.
pub fn build_filter_dialog(
    registration_ok: bool,
    entity_id: u32,
    restore_from_scratch: Option<FilterDialogRestore>,
) -> Option<FilterDialogView> {
    if !registration_ok { return None; }
    let mut v = FilterDialogView {
        entity_id,
        slot_01: 1,
        slot_06: 1,
        slot_0b: 6,
        slot_0e: 9,
        slot_11: 0xb,
        slot_14: 3,
        slot_17: 0xf,
    };
    if let Some(r) = restore_from_scratch {
        v.slot_01 = r.s01;
        v.slot_06 = r.s06;
        v.slot_0b = r.s0b;
        v.slot_0e = r.s0e;
        v.slot_11 = r.s11;
        v.slot_14 = r.s14;
        v.slot_17 = r.s17;
    }
    Some(v)
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Club Squad ---

    #[test]
    fn club_squad_licence_failure_returns_none() {
        assert!(build_club_squad(false, true, 1, 2, 0, 0, 0).is_none());
    }
    #[test]
    fn club_squad_registration_failure_returns_none() {
        assert!(build_club_squad(true, false, 1, 2, 0, 0, 0).is_none());
    }
    #[test]
    fn club_squad_ok_carries_all_slots() {
        let v = build_club_squad(true, true, 42, 7, 1, 2, 3).unwrap();
        assert_eq!(v.club_id, 42);
        assert_eq!(v.secondary, 7);
        assert_eq!(v.view_mode, 2);
        assert_eq!((v.flag_a, v.flag_b, v.flag_c), (1, 2, 3));
    }

    // --- Related Club List ---

    #[test]
    fn related_club_list_registration_failure_returns_none() {
        assert!(build_related_club_list(false, 100).is_none());
    }
    #[test]
    fn related_club_list_ok_carries_anchor() {
        let v = build_related_club_list(true, 999).unwrap();
        assert_eq!(v.anchor, 999);
    }
    #[test]
    fn related_club_list_tail_sentinel_is_0x154e21() {
        assert_eq!(RELATED_CLUB_LIST_TAIL_SENTINEL, 0x154e21);
    }

    // --- Club Rivals ---

    #[test]
    fn club_rivals_no_rivals_returns_none() {
        assert!(build_club_rivals(true, 10, None, 0, 0).is_none());
    }
    #[test]
    fn club_rivals_registration_failure_returns_none() {
        assert!(build_club_rivals(false, 10, Some(0), 3, 20).is_none());
    }
    #[test]
    fn club_rivals_selected_position_encoded_as_bitmask() {
        let v = build_club_rivals(true, 10, Some(2), 4, 20).unwrap();
        assert_eq!(v.selected_bitmask, 0b100);
        assert_eq!(v.club_id, 10);
        assert_eq!(v.peer_id, 20);
    }
    #[test]
    fn club_rivals_no_selection_gives_zero_bitmask() {
        let v = build_club_rivals(true, 10, None, 3, 20).unwrap();
        assert_eq!(v.selected_bitmask, 0);
    }
    #[test]
    fn club_rivals_out_of_range_selection_gives_zero_bitmask() {
        let v = build_club_rivals(true, 10, Some(9), 3, 20).unwrap();
        assert_eq!(v.selected_bitmask, 0);
    }

    // --- Filter Dialog ---

    #[test]
    fn filter_dialog_registration_failure_returns_none() {
        assert!(build_filter_dialog(false, 1, None).is_none());
    }
    #[test]
    fn filter_dialog_fresh_defaults_match_exe_constants() {
        let v = build_filter_dialog(true, 55, None).unwrap();
        assert_eq!(v.entity_id, 55);
        assert_eq!(v.slot_01, 1);
        assert_eq!(v.slot_06, 1);
        assert_eq!(v.slot_0b, 6);
        assert_eq!(v.slot_0e, 9);
        assert_eq!(v.slot_11, 0xb);
        assert_eq!(v.slot_14, 3);
        assert_eq!(v.slot_17, 0xf);
    }
    #[test]
    fn filter_dialog_restore_overrides_seven_tunable_slots() {
        let r = FilterDialogRestore {
            s01: 100, s06: 200, s0b: 300, s0e: 400,
            s11: 500, s14: 600, s17: 700,
        };
        let v = build_filter_dialog(true, 1, Some(r)).unwrap();
        assert_eq!(v.slot_01, 100);
        assert_eq!(v.slot_06, 200);
        assert_eq!(v.slot_0b, 300);
        assert_eq!(v.slot_0e, 400);
        assert_eq!(v.slot_11, 500);
        assert_eq!(v.slot_14, 600);
        assert_eq!(v.slot_17, 700);
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b12: ClubSquadView reads flags {b12.squad.licensed_ok} + handles \
{b12.squad.club_id, b12.squad.secondary} + bytes {b12.squad.flag_a, flag_b, flag_c}; \
RelatedClubListView reads handles {b12.related.anchor}; \
ClubRivalsView reads handles {b12.rivals.club_id, b12.rivals.peer_id} + bytes \
{b12.rivals.rival_count, b12.rivals.selected_position (-1 = None)}; \
FilterDialogView reads handles {b12.filter.entity_id} and — when \
flag {b12.filter.restore_present} is set — bytes \
{b12.filter.s01, s06, s0b, s0e, s11, s14, s17}.\n\
UNKNOWNS: wave-B typed pools not yet available; club/peer records should \
eventually resolve through world.core.clubs.";

use crate::world_facade::WorldFacade;

/// Populator for [`ClubSquadView`].
pub fn populate_club_squad(world: &WorldFacade) -> Option<ClubSquadView> {
    build_club_squad(
        world.flag("b12.squad.licensed_ok"),
        world.registration_ok,
        world.handle("b12.squad.club_id"),
        world.handle("b12.squad.secondary"),
        world.byte("b12.squad.flag_a") as i8,
        world.byte("b12.squad.flag_b") as i8,
        world.byte("b12.squad.flag_c") as i8,
    )
}

/// Populator for [`RelatedClubListView`].
pub fn populate_related_club_list(world: &WorldFacade) -> Option<RelatedClubListView> {
    build_related_club_list(world.registration_ok, world.handle("b12.related.anchor"))
}

/// Populator for [`ClubRivalsView`].
pub fn populate_club_rivals(world: &WorldFacade) -> Option<ClubRivalsView> {
    let raw = world.byte("b12.rivals.selected_position");
    let selected = if raw < 0 { None } else { Some(raw as u8) };
    build_club_rivals(
        world.registration_ok,
        world.handle("b12.rivals.club_id"),
        selected,
        world.byte("b12.rivals.rival_count") as u8,
        world.handle("b12.rivals.peer_id"),
    )
}

/// Populator for [`FilterDialogView`].
pub fn populate_filter_dialog(world: &WorldFacade) -> Option<FilterDialogView> {
    let restore = if world.flag("b12.filter.restore_present") {
        Some(FilterDialogRestore {
            s01: world.byte("b12.filter.s01") as u8,
            s06: world.byte("b12.filter.s06") as u8,
            s0b: world.byte("b12.filter.s0b") as u8,
            s0e: world.byte("b12.filter.s0e") as u8,
            s11: world.byte("b12.filter.s11") as u8,
            s14: world.byte("b12.filter.s14") as u8,
            s17: world.byte("b12.filter.s17") as u8,
        })
    } else {
        None
    };
    build_filter_dialog(
        world.registration_ok,
        world.handle("b12.filter.entity_id"),
        restore,
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_club_squad_requires_license() {
        assert!(populate_club_squad(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready().with_flag("b12.squad.licensed_ok", true);
        assert!(populate_club_squad(&w).is_some());
    }
    #[test]
    fn populate_related_club_list_ok() {
        assert!(populate_related_club_list(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_club_rivals_zero_count_returns_none() {
        assert!(populate_club_rivals(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_club_rivals_selected_bitmask_computed() {
        let w = WorldFacade::ready()
            .with_byte("b12.rivals.rival_count", 8)
            .with_byte("b12.rivals.selected_position", 2);
        let v = populate_club_rivals(&w).unwrap();
        assert_eq!(v.selected_bitmask, 1 << 2);
    }
    #[test]
    fn populate_filter_dialog_defaults_slots() {
        let v = populate_filter_dialog(&WorldFacade::ready()).unwrap();
        assert_eq!(v.slot_01, 1);
        assert_eq!(v.slot_17, 0xf);
    }
    #[test]
    fn populate_filter_dialog_restores_from_scratch() {
        let w = WorldFacade::ready()
            .with_flag("b12.filter.restore_present", true)
            .with_byte("b12.filter.s01", 9)
            .with_byte("b12.filter.s17", 0xaa);
        let v = populate_filter_dialog(&w).unwrap();
        assert_eq!(v.slot_01, 9);
        assert_eq!(v.slot_17, 0xaa);
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A pattern)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b12 pool sources: ClubSquad flags come from FUN_0076d7d0(2/4/5) \
    cmd-arg parser (not pools). RelatedClubList anchor = active human's \
    club. ClubRivals selected_position + rival_count come from the \
    club's rivals[] array at club_record+???; not decoded yet. \
    FilterDialog `restore_from_scratch` is a per-widget scratch pointer.";

use crate::world_pools::WorldPools;

pub fn populate_club_squad_from_pools(pools: &WorldPools<'_>) -> Option<ClubSquadView> {
    let club = pools.active_human_club_id().or_else(|| pools.clubs.first().map(|c| c.id))?;
    build_club_squad(true, true, club, 0, 0, 0, 0)
}

pub fn populate_related_club_list_from_pools(
    pools: &WorldPools<'_>,
) -> Option<RelatedClubListView> {
    let anchor = pools.active_human_club_id().unwrap_or(0);
    build_related_club_list(true, anchor)
}

pub fn populate_club_rivals_from_pools(pools: &WorldPools<'_>) -> Option<ClubRivalsView> {
    // Requires rival_count > 0; without a decoded rivals[] table we
    // return None per the exe's zero-count MsgBox branch.
    let club = pools.active_human_club_id().or_else(|| pools.clubs.first().map(|c| c.id))?;
    build_club_rivals(true, club, None, 0, 0)
}

pub fn populate_filter_dialog_from_pools(
    pools: &WorldPools<'_>,
) -> Option<FilterDialogView> {
    let entity = pools.active_human_club_id().unwrap_or(0);
    build_filter_dialog(true, entity, None)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn empty_pools_return_expected() {
        let p = WorldPools::empty();
        assert!(populate_club_squad_from_pools(&p).is_none());
        assert!(populate_related_club_list_from_pools(&p).is_some());
        assert!(populate_club_rivals_from_pools(&p).is_none());
        assert!(populate_filter_dialog_from_pools(&p).is_some());
    }
}
