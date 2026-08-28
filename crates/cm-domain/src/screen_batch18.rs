//! Batch 18: 5 more setup functions.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `00761620.c` — Two-selector list screen with 50-slot flag scan
//!   (counts entries whose +0x5e flag bit is clear). 8 slots.
//! * `00763660.c` — Two-slot dialog gated by loader call
//!   (`FUN_0076eb10`); on failure shows Error msgbox and aborts.
//! * `00771880.c` — Two-selector screen; allocates a 0x65-byte scratch
//!   buffer (first byte zeroed). 3 slots.
//! * `0077e1d0.c` — Active-human seat screen; reads
//!   `DAT_00b59fc2[active_human*0xc0]`, looks up an entry via
//!   `FUN_0077d770`, computes a state code, writes 9 slots.
//! * `0078ad80.c` — Trivial two-pointer setup; null-guarded, writes
//!   two dereferenced ptr values into slots 0 and 1.

use serde::{Deserialize, Serialize};

// =====================================================================
// 00761620 — two-selector list with flag scan (8 slots)
// =====================================================================

/// One entry in the exe's 50-slot table starting at
/// `(DAT_00acd5bc + selector0*0x245) + 0xd7`. Each entry is a pointer;
/// non-null entries carry a byte at `+0x5e` whose bit 3 (selector1==0)
/// or bit 2 (selector1!=0) determines whether the entry is *skipped*.
///
/// We model the exact predicate: `hidden = (flags & mask) != 0`; the
/// running count is of entries where `hidden == false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FlagEntry {
    /// True when the pointer at this table slot is non-null.
    pub present: bool,
    /// Byte at entry +0x5e (only meaningful when `present`).
    pub flags: u8,
}

/// View for `FUN_00761620(param_1, param_2)`.
///
/// Slot layout, matching `FUN_007E7130(n, …)` calls:
/// 0: `iVar6` (selector0 row address / handle)
/// 1: `iVar3` (selector1 value)
/// 2: `param_1`
/// 3: `param_2`
/// 4: `local_ed` (visible count, u8)
/// 5, 6, 7: extra selectors (`FUN_0076d7d0(5|6|7)`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwoSelectorListView {
    pub row_handle: u32,
    pub selector1: u32,
    pub param_1: u32,
    pub param_2: u32,
    pub visible_count: u8,
    pub selector5: u32,
    pub selector6: u32,
    pub selector7: u32,
}

/// Direct port of `FUN_00761620`.
///
/// * `entries` is the 50-slot pointer table (`piVar5` walk).
/// * `selector0` picks the 0x245-stride row; `row_handle` is
///   `DAT_00acd5bc + selector0*0x245` (caller supplies as `row_base`).
/// * `selector1` chooses the mask (`0` → 8, otherwise → 4).
pub fn build_two_selector_list(
    registration_ok: bool,
    row_base: u32,
    selector0: u32,
    selector1: u32,
    param_1: u32,
    param_2: u32,
    selector5: u32,
    selector6: u32,
    selector7: u32,
    entries: &[FlagEntry; 50],
) -> Option<TwoSelectorListView> {
    if !registration_ok {
        return None;
    }
    let mask: u8 = if selector1 == 0 { 8 } else { 4 };
    let mut visible: u8 = 0;
    for e in entries.iter() {
        if e.present && (e.flags & mask) == 0 {
            visible = visible.wrapping_add(1);
        }
    }
    let row_handle = row_base.wrapping_add(selector0.wrapping_mul(0x245));
    Some(TwoSelectorListView {
        row_handle,
        selector1,
        param_1,
        param_2,
        visible_count: visible,
        selector5,
        selector6,
        selector7,
    })
}

// =====================================================================
// 00763660 — loader-gated dialog (2 slots)
// =====================================================================

/// View for `FUN_00763660(param_1, param_2)`.
///
/// Slot 0 is a constant `1` marker; slot 1 is `FUN_0076d7d0(0)` when
/// `param_1 != 0`, otherwise `0`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LoaderGatedDialogView {
    pub marker: u32,
    pub selector0: u32,
}

/// Direct port of `FUN_00763660`.
///
/// * `param_1 == 0` skips the loader call and emits `selector0 = 0`.
/// * When `param_1 != 0`, `loader_ok` matches the exe's
///   `FUN_0076eb10 != 0` guard — on failure the exe shows an Error
///   message and aborts; we return `None`.
pub fn build_loader_gated_dialog(
    registration_ok: bool,
    param_1: u32,
    loader_ok: bool,
    selector0: u32,
) -> Option<LoaderGatedDialogView> {
    if !registration_ok {
        return None;
    }
    if param_1 != 0 && !loader_ok {
        return None;
    }
    let s0 = if param_1 == 0 { 0 } else { selector0 };
    Some(LoaderGatedDialogView { marker: 1, selector0: s0 })
}

// =====================================================================
// 00771880 — two-selector screen with 0x65 scratch buffer (3 slots)
// =====================================================================

/// View for `FUN_00771880(param_1, param_2)`.
///
/// Slot 0 is `0` when `selector1 == 0`, otherwise
/// `DAT_00acd5c4 + selector0*0x6e`. Slot 1 is a fresh 0x65-byte scratch
/// (first byte zeroed via `FUN_007E6EE0(1)`). Slot 2 is `param_2`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualSelectorScratchView {
    pub row_handle: u32,
    pub scratch: Vec<u8>,
    pub param_2: u32,
}

impl Default for DualSelectorScratchView {
    fn default() -> Self {
        let mut scratch = vec![0u8; 0x65];
        scratch[0] = 0;
        Self { row_handle: 0, scratch, param_2: 0 }
    }
}

/// Direct port of `FUN_00771880`.
pub fn build_dual_selector_scratch(
    registration_ok: bool,
    row_base: u32,
    selector0: u32,
    selector1: u32,
    param_2: u32,
) -> Option<DualSelectorScratchView> {
    if !registration_ok {
        return None;
    }
    let row_handle = if selector1 == 0 {
        0
    } else {
        row_base.wrapping_add(selector0.wrapping_mul(0x6e))
    };
    let mut scratch = vec![0u8; 0x65];
    scratch[0] = 0;
    Some(DualSelectorScratchView { row_handle, scratch, param_2 })
}

// =====================================================================
// 0077e1d0 — active-human seat screen (9 slots)
// =====================================================================

/// Result of the exe's `FUN_00536990(entry+0x103, entry+0x107)` state
/// lookup. Values 7 and 0x1c both map to the "flagged" state (-1);
/// anything else stays 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SeatEntry {
    /// True when `FUN_0077d770` returned non-null (entry exists).
    pub present: bool,
    /// Result of `FUN_00536990(dates)`.
    pub state_code: u32,
    /// Byte at entry +0x113.
    pub sub_flag: i8,
    /// Pointer emitted at slot 6 (entry + 0x10b); modelled as opaque u32.
    pub slot6_handle: u32,
    /// Pointer emitted at slot 7 (entry + 0x103); modelled as opaque u32.
    pub slot7_handle: u32,
    /// Value emitted at slot 5 via `FUN_0077da00(seat)`.
    pub slot5_value: u32,
}

/// View for `FUN_0077e1d0(param_1)`.
///
/// Slot layout, matching `FUN_007E7130(n, …)` calls:
/// 0: `*param_1`
/// 1: `&DAT_00acde90` (opaque; caller passes as `dat_acde90_handle`)
/// 2: state (`i8`): -1 when state_code is 7 or 0x1c, else 0
/// 3: `sub_flag`
/// 4: `entry_present` (1 or 0)
/// 5: `FUN_0077da00(seat)` return
/// 6: entry+0x10b or 0
/// 7: entry+0x103 or 0
/// 8: state (same as slot 2)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SeatScreenView {
    pub param_1_deref: u32,
    pub dat_acde90_handle: u32,
    pub state: i8,
    pub sub_flag: i8,
    pub entry_present: u8,
    pub slot5_value: u32,
    pub slot6_handle: u32,
    pub slot7_handle: u32,
}

/// Direct port of `FUN_0077e1d0`.
pub fn build_seat_screen(
    registration_ok: bool,
    param_1_deref: u32,
    dat_acde90_handle: u32,
    entry: &SeatEntry,
) -> Option<SeatScreenView> {
    if !registration_ok {
        return None;
    }
    let (state, sub_flag, entry_present, slot6, slot7) = if !entry.present {
        (0i8, 0i8, 0u8, 0u32, 0u32)
    } else {
        let state = if entry.state_code == 7 || entry.state_code == 0x1c {
            -1i8
        } else {
            0i8
        };
        (state, entry.sub_flag, 1u8, entry.slot6_handle, entry.slot7_handle)
    };
    Some(SeatScreenView {
        param_1_deref,
        dat_acde90_handle,
        state,
        sub_flag,
        entry_present,
        slot5_value: entry.slot5_value,
        slot6_handle: slot6,
        slot7_handle: slot7,
    })
}

// =====================================================================
// 0078ad80 — trivial two-pointer setup (2 slots)
// =====================================================================

/// View for `FUN_0078ad80(param_1, param_2)`. Both params must be
/// non-null. Slots 0 and 1 hold the dereferenced values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwoPointerView {
    pub p1_deref: u32,
    pub p2_deref: u32,
}

/// Direct port of `FUN_0078ad80`. Returns `None` when either pointer is
/// null (matches exe's null guard) or when registration failed.
pub fn build_two_pointer(
    registration_ok: bool,
    p1: Option<u32>,
    p2: Option<u32>,
) -> Option<TwoPointerView> {
    let p1 = p1?;
    let p2 = p2?;
    if !registration_ok {
        return None;
    }
    Some(TwoPointerView { p1_deref: p1, p2_deref: p2 })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 00761620 --------------------------------------------------
    #[test]
    fn two_selector_list_registration_gate() {
        let entries = [FlagEntry::default(); 50];
        assert!(build_two_selector_list(false, 0, 0, 0, 0, 0, 0, 0, 0, &entries).is_none());
    }
    #[test]
    fn two_selector_list_selector1_zero_uses_mask_8() {
        let mut entries = [FlagEntry::default(); 50];
        // Three present entries: flags 0, 8, 4.
        // With mask=8 (selector1==0), only flags==0 and flags==4 are visible.
        entries[0] = FlagEntry { present: true, flags: 0 };
        entries[1] = FlagEntry { present: true, flags: 8 };
        entries[2] = FlagEntry { present: true, flags: 4 };
        let v = build_two_selector_list(true, 0x1000, 2, 0, 0, 0, 0, 0, 0, &entries).unwrap();
        assert_eq!(v.visible_count, 2);
        assert_eq!(v.row_handle, 0x1000 + 2 * 0x245);
        assert_eq!(v.selector1, 0);
    }
    #[test]
    fn two_selector_list_selector1_nonzero_uses_mask_4() {
        let mut entries = [FlagEntry::default(); 50];
        entries[0] = FlagEntry { present: true, flags: 0 };
        entries[1] = FlagEntry { present: true, flags: 8 };
        entries[2] = FlagEntry { present: true, flags: 4 };
        // With mask=4, flags==0 and flags==8 are visible; flags==4 is hidden.
        let v = build_two_selector_list(true, 0, 0, 1, 0, 0, 0, 0, 0, &entries).unwrap();
        assert_eq!(v.visible_count, 2);
    }
    #[test]
    fn two_selector_list_ignores_absent_slots() {
        let entries = [FlagEntry::default(); 50]; // all absent
        let v = build_two_selector_list(true, 0, 0, 0, 0, 0, 0, 0, 0, &entries).unwrap();
        assert_eq!(v.visible_count, 0);
    }

    // ---- 00763660 --------------------------------------------------
    #[test]
    fn loader_gated_param1_zero_short_circuits() {
        let v = build_loader_gated_dialog(true, 0, false, 999).unwrap();
        assert_eq!(v.marker, 1);
        assert_eq!(v.selector0, 0);
    }
    #[test]
    fn loader_gated_param1_nonzero_ok_carries_selector() {
        let v = build_loader_gated_dialog(true, 42, true, 7).unwrap();
        assert_eq!(v.selector0, 7);
    }
    #[test]
    fn loader_gated_param1_nonzero_fail_returns_none() {
        assert!(build_loader_gated_dialog(true, 42, false, 7).is_none());
    }
    #[test]
    fn loader_gated_registration_failure_returns_none() {
        assert!(build_loader_gated_dialog(false, 0, true, 0).is_none());
    }

    // ---- 00771880 --------------------------------------------------
    #[test]
    fn dual_selector_scratch_selector1_zero_zeros_handle() {
        let v = build_dual_selector_scratch(true, 0x1000, 5, 0, 42).unwrap();
        assert_eq!(v.row_handle, 0);
        assert_eq!(v.scratch.len(), 0x65);
        assert_eq!(v.scratch[0], 0);
        assert_eq!(v.param_2, 42);
    }
    #[test]
    fn dual_selector_scratch_selector1_nonzero_computes_row() {
        let v = build_dual_selector_scratch(true, 0x1000, 3, 1, 0).unwrap();
        assert_eq!(v.row_handle, 0x1000 + 3 * 0x6e);
    }
    #[test]
    fn dual_selector_scratch_registration_failure_returns_none() {
        assert!(build_dual_selector_scratch(false, 0, 0, 1, 0).is_none());
    }

    // ---- 0077e1d0 --------------------------------------------------
    #[test]
    fn seat_screen_absent_entry_zeroes_all() {
        let entry = SeatEntry::default();
        let v = build_seat_screen(true, 0xabc, 0xdef, &entry).unwrap();
        assert_eq!(v.param_1_deref, 0xabc);
        assert_eq!(v.dat_acde90_handle, 0xdef);
        assert_eq!(v.state, 0);
        assert_eq!(v.sub_flag, 0);
        assert_eq!(v.entry_present, 0);
        assert_eq!(v.slot6_handle, 0);
        assert_eq!(v.slot7_handle, 0);
    }
    #[test]
    fn seat_screen_state_code_7_flags_state_minus_one() {
        let entry = SeatEntry {
            present: true, state_code: 7, sub_flag: 3,
            slot6_handle: 0x100, slot7_handle: 0x200, slot5_value: 9,
        };
        let v = build_seat_screen(true, 0, 0, &entry).unwrap();
        assert_eq!(v.state, -1);
        assert_eq!(v.sub_flag, 3);
        assert_eq!(v.entry_present, 1);
        assert_eq!(v.slot5_value, 9);
        assert_eq!(v.slot6_handle, 0x100);
        assert_eq!(v.slot7_handle, 0x200);
    }
    #[test]
    fn seat_screen_state_code_0x1c_also_flags_state_minus_one() {
        let entry = SeatEntry { present: true, state_code: 0x1c, ..Default::default() };
        let v = build_seat_screen(true, 0, 0, &entry).unwrap();
        assert_eq!(v.state, -1);
    }
    #[test]
    fn seat_screen_other_state_code_leaves_state_zero() {
        let entry = SeatEntry { present: true, state_code: 5, ..Default::default() };
        let v = build_seat_screen(true, 0, 0, &entry).unwrap();
        assert_eq!(v.state, 0);
        assert_eq!(v.entry_present, 1);
    }
    #[test]
    fn seat_screen_registration_failure_returns_none() {
        assert!(build_seat_screen(false, 0, 0, &SeatEntry::default()).is_none());
    }

    // ---- 0078ad80 --------------------------------------------------
    #[test]
    fn two_pointer_carries_derefs() {
        let v = build_two_pointer(true, Some(0x11), Some(0x22)).unwrap();
        assert_eq!(v.p1_deref, 0x11);
        assert_eq!(v.p2_deref, 0x22);
    }
    #[test]
    fn two_pointer_null_p1_returns_none() {
        assert!(build_two_pointer(true, None, Some(1)).is_none());
    }
    #[test]
    fn two_pointer_null_p2_returns_none() {
        assert!(build_two_pointer(true, Some(1), None).is_none());
    }
    #[test]
    fn two_pointer_registration_failure_returns_none() {
        assert!(build_two_pointer(false, Some(1), Some(2)).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

/// Facade keys this batch reads. Anything not listed falls back to the
/// exe's null-pool default (usually zero).
pub const TODO_POPULATOR_INFO: &str = "\
b18: TwoSelectorListView reads handles \
{row_base_00acd5bc, selector0, selector1, param_1, param_2, sel5, sel6, sel7} \
and world.flag_entries; LoaderGatedDialogView reads {param_1, selector0} + \
flag `loader_ok`; DualSelectorScratchView reads \
{row_base_00acd5c4, selector0, selector1, param_2}; SeatScreenView reads \
{param_1_deref, dat_acde90_handle} + world.seat_entry; TwoPointerView reads \
{p1, p2} — treated as null when absent (matches exe null-guard).\n\
UNKNOWNS: wave-B typed pools not yet available; string-keyed lookup will \
be replaced when world_facade grows typed record accessors.";

use crate::world_facade::WorldFacade;

/// Populator for `TwoSelectorListView`.
pub fn populate_two_selector_list(world: &WorldFacade) -> Option<TwoSelectorListView> {
    build_two_selector_list(
        world.registration_ok,
        world.handle("row_base_00acd5bc"),
        world.handle("selector0"),
        world.handle("selector1"),
        world.handle("param_1"),
        world.handle("param_2"),
        world.handle("selector5"),
        world.handle("selector6"),
        world.handle("selector7"),
        &world.flag_entries,
    )
}

/// Populator for `LoaderGatedDialogView`.
pub fn populate_loader_gated_dialog(world: &WorldFacade) -> Option<LoaderGatedDialogView> {
    build_loader_gated_dialog(
        world.registration_ok,
        world.handle("param_1"),
        world.flag("loader_ok"),
        world.handle("selector0"),
    )
}

/// Populator for `DualSelectorScratchView`.
pub fn populate_dual_selector_scratch(world: &WorldFacade) -> Option<DualSelectorScratchView> {
    build_dual_selector_scratch(
        world.registration_ok,
        world.handle("row_base_00acd5c4"),
        world.handle("selector0"),
        world.handle("selector1"),
        world.handle("param_2"),
    )
}

/// Populator for `SeatScreenView`.
pub fn populate_seat_screen(world: &WorldFacade) -> Option<SeatScreenView> {
    build_seat_screen(
        world.registration_ok,
        world.handle("param_1_deref"),
        world.handle("dat_acde90_handle"),
        &world.seat_entry,
    )
}

/// Populator for `TwoPointerView`. Absent handles map to `None` so
/// the null-guard trips (matches the exe).
pub fn populate_two_pointer(world: &WorldFacade) -> Option<TwoPointerView> {
    let p1 = world.handles.get("p1").copied();
    let p2 = world.handles.get("p2").copied();
    build_two_pointer(world.registration_ok, p1, p2)
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_two_selector_list_defaults_to_zero_visible() {
        let w = WorldFacade::ready();
        let v = populate_two_selector_list(&w).unwrap();
        assert_eq!(v.visible_count, 0);
    }
    #[test]
    fn populate_loader_gated_dialog_short_circuits_on_zero_param() {
        let v = populate_loader_gated_dialog(&WorldFacade::ready()).unwrap();
        assert_eq!(v.marker, 1);
        assert_eq!(v.selector0, 0);
    }
    #[test]
    fn populate_dual_selector_scratch_zero_selector1_zeroes_handle() {
        let v = populate_dual_selector_scratch(&WorldFacade::ready()).unwrap();
        assert_eq!(v.row_handle, 0);
    }
    #[test]
    fn populate_seat_screen_uses_default_seat() {
        let v = populate_seat_screen(&WorldFacade::ready()).unwrap();
        assert_eq!(v.entry_present, 0);
    }
    #[test]
    fn populate_two_pointer_missing_handles_returns_none() {
        assert!(populate_two_pointer(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_two_pointer_present_handles_ok() {
        let w = WorldFacade::ready().with_handle("p1", 1).with_handle("p2", 2);
        let v = populate_two_pointer(&w).unwrap();
        assert_eq!((v.p1_deref, v.p2_deref), (1, 2));
    }
}
