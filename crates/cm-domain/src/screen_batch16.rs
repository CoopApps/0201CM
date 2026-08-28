//! Batch 16: 5 more setup functions — one big history/results table +
//! three "manage" (staff/board interaction) screens + one match-code
//! screen.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `0x005DAB70.c` — history/records screen (cm3_history:0x115),
//!   22 slots (0x0..0x15) with a rectangle + focus id
//! * `0x00696FA0.c` — manage screen (cm3_code_manage:0x51f), 1 slot,
//!   param==0 fires MsgBox
//! * `0x00697390.c` — manage screen (cm3_code_manage:0x588), 1 slot,
//!   param==0 fires MsgBox
//! * `0x00697C30.c` — manage screen (cm3_code_manage:0x67e/0x685),
//!   2 slots, gated by `FUN_0076eb10` and uses per-club record base
//! * `0x006FD6C0.c` — match screen (cm3_code_match:0xcd), area-scoped
//!   via `FUN_007e6430` + `FUN_007e7000`, 4 slots

use serde::{Deserialize, Serialize};

// =====================================================================
// 0x005DAB70 — History / records list (cm3_history:0x115), 22 slots
// =====================================================================

/// History/records view — 22 slots (0..=0x15).
///
/// Direct port of `FUN_005DAB70(param_1)`. Slots follow the exe:
/// * 0     = -1
/// * 1..7  = 0
/// * 8..0xB = -1
/// * 0xC   = param_1 (focus id, flagged with the "1" third-arg)
/// * 0xD   = -1
/// * 0xE   = 0x91, 0xF = 0x4B, 0x10 = 0x2E9, 0x11 = 0x20F
///           (rect: x=0x91, y=0x4B, w=0x2E9, h=0x20F)
/// * 0x12  = 0x115 (string-table line number = context tag)
/// * 0x13  = `s_cm3_history` string ptr — kept as a marker bool
/// * 0x14  = 0x101, 0x15 = 0x14F  (secondary rect / footer coords)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryRecordsView {
    /// Slot 0xC — focus id (param_1). Third arg = 1 means "id-typed".
    pub focus_id: u32,
    /// Slots 0xE..=0x11 — primary rectangle (x, y, w, h).
    pub main_rect: (u32, u32, u32, u32),
    /// Slot 0x12 — string-table line (0x115).
    pub string_line: u32,
    /// Slot 0x13 — non-zero because `s_cm3_history` is a real string ptr.
    pub has_source_marker: bool,
    /// Slots 0x14, 0x15 — secondary coords (0x101, 0x14F).
    pub footer_xy: (u32, u32),
}

impl Default for HistoryRecordsView {
    fn default() -> Self {
        Self {
            focus_id: 0,
            main_rect: (0x91, 0x4B, 0x2E9, 0x20F),
            string_line: 0x115,
            has_source_marker: true,
            footer_xy: (0x101, 0x14F),
        }
    }
}

/// Direct port of `FUN_005DAB70(param_1)`.
pub fn build_history_records(
    registration_ok: bool,
    focus_id: u32,
) -> Option<HistoryRecordsView> {
    if !registration_ok { return None; }
    Some(HistoryRecordsView { focus_id, ..Default::default() })
}

// =====================================================================
// 0x00696FA0 — manage screen (cm3_code_manage:0x51f), 1 slot
// =====================================================================

/// Manage-line-0x51f view — 1 slot carrying the object id.
///
/// Direct port of `FUN_00696FA0(param_1)`. Guards:
/// * `param_1 == 0` fires MsgBox `Error` with `manage:0x51f` and sets
///   `DAT_00b4d5a8 = 0` — we return `None`.
/// * If registration fails after that, slot 0 is *not* written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ManageObj51fView {
    /// Slot 0 — object id (param_1, non-zero).
    pub object_id: i32,
}

/// Direct port of `FUN_00696FA0(param_1)`.
pub fn build_manage_obj_51f(
    registration_ok: bool,
    object_id: i32,
) -> Option<ManageObj51fView> {
    if object_id == 0 { return None; }        // exe: MsgBox + set flag
    if !registration_ok { return None; }
    Some(ManageObj51fView { object_id })
}

// =====================================================================
// 0x00697390 — manage screen (cm3_code_manage:0x588), 1 slot
// =====================================================================

/// Manage-line-0x588 view — identical shape to the 0x51f screen but a
/// different builder registration target, so kept as its own type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ManageObj588View {
    /// Slot 0 — object id.
    pub object_id: i32,
}

/// Direct port of `FUN_00697390(param_1)`.
pub fn build_manage_obj_588(
    registration_ok: bool,
    object_id: i32,
) -> Option<ManageObj588View> {
    if object_id == 0 { return None; }
    if !registration_ok { return None; }
    Some(ManageObj588View { object_id })
}

// =====================================================================
// 0x00697C30 — manage screen (cm3_code_manage:0x67e/0x685), 2 slots
// =====================================================================

/// Manage-line-0x67e/0x685 view — 2 slots, needs a per-club record ptr
/// resolved from a global base + `FUN_0076d7d0(0) * 0x245`.
///
/// Direct port of `FUN_00697C30(param_1, param_2)`. Two error paths:
/// * `param_1 == 0` → MsgBox `manage:0x67e`
/// * `FUN_0076eb10(param_1, param_2, _) == 0` → MsgBox `manage:0x685`
///   (guarded via `record_lookup_ok`)
///
/// The Rust caller supplies the resolved `record_offset` — we don't
/// reinvent the exe's per-club record table here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ManageObj67eView {
    /// Slot 0 — carries param_2 (the *inner* object id).
    pub inner_id: u32,
    /// Slot 1 — resolved per-club record byte offset
    /// (`DAT_00acd5bc + club_index * 0x245`).
    pub record_offset: u32,
}

/// Direct port of `FUN_00697C30(param_1, param_2)`.
///
/// * `container_id` = `param_1`; 0 fires the first MsgBox.
/// * `record_lookup_ok` = whether `FUN_0076eb10` succeeded.
/// * `inner_id` = `param_2`; `record_offset` = the resolved slot-1 value.
pub fn build_manage_obj_67e(
    registration_ok: bool,
    container_id: u32,
    record_lookup_ok: bool,
    inner_id: u32,
    record_offset: u32,
) -> Option<ManageObj67eView> {
    if container_id == 0 { return None; }     // manage:0x67e MsgBox
    if !record_lookup_ok { return None; }     // manage:0x685 MsgBox
    if !registration_ok { return None; }
    Some(ManageObj67eView { inner_id, record_offset })
}

// =====================================================================
// 0x006FD6C0 — match screen (cm3_code_match:0xcd), area-scoped, 4 slots
// =====================================================================

/// Match-screen view — 4 slots, uses the area-scoped setter
/// `FUN_007e7000(area, slot, val, flag)` instead of the global one.
///
/// Direct port of `FUN_006FD6C0(area, match_id)`. Slots:
/// * 1 = match_id      (flag = 0)
/// * 2 = 0             (flag = 0)
/// * 3 = 0             (flag = 0)
/// * 4 = 1             (flag = 0)
///
/// `match_id == 0` fires MsgBox `match:0xcd`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchAreaScreenView {
    /// Slot 1 — match id (param_2, must be non-zero).
    pub match_id: i32,
    /// Slot 2 — unused / cleared.
    pub aux_a: u32,
    /// Slot 3 — unused / cleared.
    pub aux_b: u32,
    /// Slot 4 — always 1 (bool flag: "screen active" / mode-on).
    pub screen_flag: u32,
}

impl Default for MatchAreaScreenView {
    fn default() -> Self {
        Self { match_id: 0, aux_a: 0, aux_b: 0, screen_flag: 1 }
    }
}

/// Direct port of `FUN_006FD6C0(area, param_2)`.
pub fn build_match_area_screen(
    registration_ok: bool,
    match_id: i32,
) -> Option<MatchAreaScreenView> {
    if match_id == 0 { return None; }
    if !registration_ok { return None; }
    Some(MatchAreaScreenView { match_id, ..Default::default() })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- 0x005DAB70 ---
    #[test]
    fn history_records_defaults_match_exe_slots() {
        let v = build_history_records(true, 42).unwrap();
        assert_eq!(v.focus_id, 42);
        assert_eq!(v.main_rect, (0x91, 0x4B, 0x2E9, 0x20F));
        assert_eq!(v.string_line, 0x115);
        assert!(v.has_source_marker);
        assert_eq!(v.footer_xy, (0x101, 0x14F));
    }
    #[test]
    fn history_records_registration_failure_returns_none() {
        assert!(build_history_records(false, 42).is_none());
    }

    // --- 0x00696FA0 ---
    #[test]
    fn manage_51f_zero_id_fires_msgbox_returns_none() {
        assert!(build_manage_obj_51f(true, 0).is_none());
    }
    #[test]
    fn manage_51f_nonzero_id_carried_to_slot_0() {
        let v = build_manage_obj_51f(true, 12345).unwrap();
        assert_eq!(v.object_id, 12345);
    }
    #[test]
    fn manage_51f_registration_failure_returns_none() {
        assert!(build_manage_obj_51f(false, 5).is_none());
    }

    // --- 0x00697390 ---
    #[test]
    fn manage_588_zero_id_returns_none() {
        assert!(build_manage_obj_588(true, 0).is_none());
    }
    #[test]
    fn manage_588_carries_id() {
        let v = build_manage_obj_588(true, 999).unwrap();
        assert_eq!(v.object_id, 999);
    }

    // --- 0x00697C30 ---
    #[test]
    fn manage_67e_zero_container_returns_none() {
        assert!(build_manage_obj_67e(true, 0, true, 5, 0x100).is_none());
    }
    #[test]
    fn manage_67e_record_lookup_failure_returns_none() {
        assert!(build_manage_obj_67e(true, 1, false, 5, 0x100).is_none());
    }
    #[test]
    fn manage_67e_normal_path_carries_both_slots() {
        let v = build_manage_obj_67e(true, 1, true, 7, 0xDEAD).unwrap();
        assert_eq!(v.inner_id, 7);
        assert_eq!(v.record_offset, 0xDEAD);
    }
    #[test]
    fn manage_67e_registration_failure_returns_none() {
        assert!(build_manage_obj_67e(false, 1, true, 7, 0).is_none());
    }

    // --- 0x006FD6C0 ---
    #[test]
    fn match_area_screen_zero_match_returns_none() {
        assert!(build_match_area_screen(true, 0).is_none());
    }
    #[test]
    fn match_area_screen_slot_4_is_always_one() {
        let v = build_match_area_screen(true, 123).unwrap();
        assert_eq!(v.match_id, 123);
        assert_eq!(v.screen_flag, 1);
        assert_eq!(v.aux_a, 0);
        assert_eq!(v.aux_b, 0);
    }
    #[test]
    fn match_area_screen_registration_failure_returns_none() {
        assert!(build_match_area_screen(false, 5).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b16: HistoryRecordsView reads handle {b16.history.focus_id}; \
ManageObj51fView + ManageObj588View read byte {b16.manage_51f.object_id}, \
{b16.manage_588.object_id}; ManageObj67eView reads handle \
{b16.manage_67e.container_id} + flag {b16.manage_67e.record_lookup_ok} + \
handles {b16.manage_67e.inner_id, record_offset}; MatchAreaScreenView reads \
byte {b16.match_area.match_id}.\n\
UNKNOWNS: wave-B typed pools not yet available; the history sub-tables (all-\
time / this season / etc.) are decoded from `world.references.staff_history` \
and family — needs typed accessor to populate the *content* here.";

use crate::world_facade::WorldFacade;

/// Populator for [`HistoryRecordsView`].
pub fn populate_history_records(world: &WorldFacade) -> Option<HistoryRecordsView> {
    build_history_records(world.registration_ok, world.handle("b16.history.focus_id"))
}

/// Populator for [`ManageObj51fView`].
pub fn populate_manage_obj_51f(world: &WorldFacade) -> Option<ManageObj51fView> {
    build_manage_obj_51f(world.registration_ok, world.byte("b16.manage_51f.object_id") as i32)
}

/// Populator for [`ManageObj588View`].
pub fn populate_manage_obj_588(world: &WorldFacade) -> Option<ManageObj588View> {
    build_manage_obj_588(world.registration_ok, world.byte("b16.manage_588.object_id") as i32)
}

/// Populator for [`ManageObj67eView`].
pub fn populate_manage_obj_67e(world: &WorldFacade) -> Option<ManageObj67eView> {
    build_manage_obj_67e(
        world.registration_ok,
        world.handle("b16.manage_67e.container_id"),
        world.flag("b16.manage_67e.record_lookup_ok"),
        world.handle("b16.manage_67e.inner_id"),
        world.handle("b16.manage_67e.record_offset"),
    )
}

/// Populator for [`MatchAreaScreenView`].
pub fn populate_match_area_screen(world: &WorldFacade) -> Option<MatchAreaScreenView> {
    build_match_area_screen(world.registration_ok, world.byte("b16.match_area.match_id") as i32)
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_history_records_ok() {
        assert!(populate_history_records(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_manage_obj_51f_zero_returns_none() {
        assert!(populate_manage_obj_51f(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready().with_byte("b16.manage_51f.object_id", 42);
        assert_eq!(populate_manage_obj_51f(&w).unwrap().object_id, 42);
    }
    #[test]
    fn populate_manage_obj_588_zero_returns_none() {
        assert!(populate_manage_obj_588(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_manage_obj_67e_needs_container_and_lookup() {
        assert!(populate_manage_obj_67e(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready()
            .with_handle("b16.manage_67e.container_id", 7)
            .with_flag("b16.manage_67e.record_lookup_ok", true)
            .with_handle("b16.manage_67e.inner_id", 3);
        let v = populate_manage_obj_67e(&w).unwrap();
        assert_eq!(v.inner_id, 3);
    }
    #[test]
    fn populate_match_area_screen_zero_returns_none() {
        assert!(populate_match_area_screen(&WorldFacade::ready()).is_none());
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A pattern)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b16 pool sources: manage_51f/588 need an object id from cmd_manage \
    codes; we use the active human seat. manage_67e's inner_id + \
    record_lookup come from FUN_0076d7d0(0) * 0x245 lookup (opaque). \
    MatchAreaScreen needs a current-match id from the day-tick engine.";

use crate::world_pools::WorldPools;

pub fn populate_history_records_from_pools(
    pools: &WorldPools<'_>,
) -> Option<HistoryRecordsView> {
    let focus = pools.active_human_seat.unwrap_or(0);
    build_history_records(true, focus)
}

pub fn populate_manage_obj_51f_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ManageObj51fView> {
    let id = pools.active_human_seat? as i32;
    build_manage_obj_51f(true, id)
}

pub fn populate_manage_obj_588_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ManageObj588View> {
    let id = pools.active_human_seat? as i32;
    build_manage_obj_588(true, id)
}

pub fn populate_manage_obj_67e_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ManageObj67eView> {
    let container = pools.active_human_seat?;
    build_manage_obj_67e(true, container, true, 0, 0)
}

pub fn populate_match_area_screen_from_pools(
    _pools: &WorldPools<'_>,
) -> Option<MatchAreaScreenView> {
    // No current-match handle on the pools facade — exe returns None
    // when match_id == 0, so we mirror that.
    build_match_area_screen(true, 0)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn empty_pools_return_expected() {
        let p = WorldPools::empty();
        assert!(populate_history_records_from_pools(&p).is_some());
        assert!(populate_manage_obj_51f_from_pools(&p).is_none());
        assert!(populate_manage_obj_588_from_pools(&p).is_none());
        assert!(populate_manage_obj_67e_from_pools(&p).is_none());
        assert!(populate_match_area_screen_from_pools(&p).is_none());
    }

    #[test]
    fn seat_enables_manage_populators() {
        let p = WorldPools { active_human_seat: Some(4), ..WorldPools::empty() };
        assert_eq!(populate_manage_obj_51f_from_pools(&p).unwrap().object_id, 4);
        assert_eq!(populate_manage_obj_588_from_pools(&p).unwrap().object_id, 4);
        assert!(populate_manage_obj_67e_from_pools(&p).is_some());
    }
}
