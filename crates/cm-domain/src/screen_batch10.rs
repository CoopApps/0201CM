//! Batch 10: 4 screen-setup functions + 1 dispatcher stub.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `0x00417780.c` — Award-screen setup (206 B, 2 slots) — param_1 == 0
//!   fires an error MsgBox and clears DAT_00b4d5a8; otherwise pushes the
//!   param + a "no active manager" flag derived from FUN_00418ca0.
//! * `0x00425e70.c` — Small 3-slot screen setup (dereferences param_1
//!   pointer for slot 0; slots 1 and 2 are zero-init).
//! * `0x0046ad30.c` — SKIPPED: 1650+ byte menu-action dispatcher, not a
//!   screen-setup fn. Reads slots 0..0x15 (FUN_007E6EE0), branches on
//!   DAT_00dbbf7c command codes 0x10..0x18, opens confirmation dialogs
//!   or forwards to other setup fns (e.g. FUN_00472bf0). No initial
//!   FUN_007e6570 gate.
//! * `0x00470bf0.c` — 7-slot setup (462 B). Requires both params non-zero;
//!   slot 3 is a colour-pool pointer keyed off the active human's nation.
//! * `0x00472bf0.c` — 11-slot setup (509 B) with a branch on param_2:
//!   char '\v' takes a variant path; char '\n' allocates a 0xD1-byte
//!   "CM Cup" descriptor pushed into slot 9/10.

use serde::{Deserialize, Serialize};

// =====================================================================
// 0x00417780 — Award-screen setup, 2 slots
// =====================================================================

/// View for `FUN_00417780(param_1)` — small award-related setup.
///
/// Slot 0 = `param_1`. Slot 1 = the boolean `sVar1 == -1` from
/// `FUN_00418ca0(active_human.nation)` — "no active manager for nation".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Award417780View {
    pub award_kind: i32,
    pub no_active_manager: bool,
}

/// Direct port of `FUN_00417780(param_1)`.
///
/// * `param_1 == 0` -> exe pops an error MsgBox and returns without pushing
///   slots. We model that by returning `None`.
/// * `registration_ok` = `FUN_007e6570(...) != 0`.
/// * `no_active_manager` = `FUN_00418ca0(active_nation) == -1`.
pub fn build_award_417780(
    registration_ok: bool,
    param_1: i32,
    no_active_manager: bool,
) -> Option<Award417780View> {
    if param_1 == 0 { return None; }
    if !registration_ok { return None; }
    Some(Award417780View { award_kind: param_1, no_active_manager })
}

// =====================================================================
// 0x00425e70 — 3-slot setup
// =====================================================================

/// View for `FUN_00425e70(param_1)` — dereferences `*param_1` into slot 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen425e70View {
    /// Slot 0: `*param_1` (first dword of the caller-supplied struct).
    pub focus: u32,
    /// Slots 1, 2: zero.
    pub reserved: [u32; 2],
}

/// Direct port of `FUN_00425e70(param_1)`.
pub fn build_screen_425e70(
    registration_ok: bool,
    focus: u32,
) -> Option<Screen425e70View> {
    if !registration_ok { return None; }
    Some(Screen425e70View { focus, reserved: [0, 0] })
}

// =====================================================================
// 0x0046ad30 — SKIPPED (dispatcher, not screen setup)
// =====================================================================

/// Stub for `FUN_0046ad30(param_1)` — this is **not** a screen-setup
/// function. It is a ~1.65 KB menu-action dispatcher that reads existing
/// widget slots via `FUN_007E6EE0`, matches command codes 0x10..0x18
/// from `DAT_00dbbf7c`, and either opens "Please Confirm" dialogs
/// (`FUN_006547c0`) or forwards to other setup fns (e.g. `FUN_00472bf0`).
/// It never calls `FUN_007e6570` as its own top-level registration.
///
/// Intentionally not ported; kept as a documentation stub so the
/// address is not silently forgotten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Dispatcher46ad30Stub;

// =====================================================================
// 0x00470bf0 — 7-slot setup
// =====================================================================

/// View for `FUN_00470bf0(param_1, param_2, param_3)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen470bf0View {
    /// Slot 0.
    pub param_1: i32,
    /// Slot 1.
    pub param_2: i32,
    /// Slot 2: constant 0x221 (widget/category id baked into the exe).
    pub category: u32,
    /// Slot 3: pointer into the colour pool for the active human's nation
    /// (or the fallback pool head `DAT_00ac65e0` when the nation record
    /// is null). We model it as an opaque index / offset.
    pub colour_pool_offset: u32,
    /// Slot 4, 5: zero.
    pub reserved: [u32; 2],
    /// Slot 6.
    pub param_3: u32,
}

/// Direct port of `FUN_00470bf0(param_1, param_2, param_3)`.
///
/// * `param_1 == 0 || param_2 == 0` -> exe pops error MsgBox, returns.
/// * `colour_pool_offset` should be `0` (i.e. use fallback `DAT_00ac65e0`)
///   when the active-human nation record is null; otherwise the caller
///   passes `(nation_id - DAT_00acd56c) * 0x2a` as computed by the exe.
pub fn build_screen_470bf0(
    registration_ok: bool,
    param_1: i32, param_2: i32, param_3: u32,
    colour_pool_offset: u32,
) -> Option<Screen470bf0View> {
    if param_1 == 0 || param_2 == 0 { return None; }
    if !registration_ok { return None; }
    Some(Screen470bf0View {
        param_1, param_2, category: 0x221,
        colour_pool_offset, reserved: [0, 0], param_3,
    })
}

// =====================================================================
// 0x00472bf0 — 11-slot setup with param_2 branch
// =====================================================================

/// CM-Cup descriptor allocated on the `param_2 == '\n'` branch — a 0xD1
/// byte scratch struct with a few offsets initialised to constants.
///
/// Exact byte layout from `FUN_006547c0(pv, "CM Cup"); pv[200]=2;
/// pv[0xc9]=3; pv[0xca]=1; pv[0xcb..0xcf]=0; pv[0xcf]=4;`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmCupDescriptor {
    pub name: String,
    pub byte_200: u8,
    pub byte_c9: u8,
    pub byte_ca: u8,
    pub word_cb: u16,
    pub word_cd: u16,
    pub word_cf: u16,
}

impl Default for CmCupDescriptor {
    fn default() -> Self {
        Self {
            name: "CM Cup".to_string(),
            byte_200: 2, byte_c9: 3, byte_ca: 1,
            word_cb: 0, word_cd: 0, word_cf: 4,
        }
    }
}

/// View for `FUN_00472bf0(param_1, param_2, param_3)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Screen472bf0View {
    /// Slot 0.
    pub param_1: i32,
    /// Slot 1: constant 1.
    pub one: u32,
    /// Slot 2, 3: zero.
    pub reserved_23: [u32; 2],
    /// Slot 4: `param_2 as i32` (the branch char '\v' / '\n' / ...).
    pub param_2: i32,
    /// Slot 5, 6: zero.
    pub reserved_56: [u32; 2],
    /// Slot 7: constant 0x14.
    pub kind: u32,
    /// Slot 8.
    pub param_3: i32,
    /// Slots 9 and 10: `pvVar3` — `Some(CmCupDescriptor)` only when
    /// `param_2 == '\n'`, else `None`.
    pub cm_cup: Option<CmCupDescriptor>,
}

/// Direct port of `FUN_00472bf0(param_1, param_2, param_3)`.
///
/// * `param_1 == 0` -> exe pops error MsgBox, returns.
/// * `param_2 == '\v' && param_3 == 0` -> same error MsgBox path.
/// * The `FUN_007e6570` "arg3" boolean is `0` on the '\v' branch and
///   `1` otherwise — modelled implicitly via `registration_ok` since
///   this fn only reports whether the gate ultimately passed.
pub fn build_screen_472bf0(
    registration_ok: bool,
    param_1: i32, param_2: i8, param_3: i32,
) -> Option<Screen472bf0View> {
    if param_1 == 0 { return None; }
    if param_2 == b'\x0b' as i8 && param_3 == 0 { return None; }
    if !registration_ok { return None; }
    let cm_cup = if param_2 == b'\n' as i8 {
        Some(CmCupDescriptor::default())
    } else {
        None
    };
    Some(Screen472bf0View {
        param_1,
        one: 1,
        reserved_23: [0, 0],
        param_2: param_2 as i32,
        reserved_56: [0, 0],
        kind: 0x14,
        param_3,
        cm_cup,
    })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 0x00417780 ----
    #[test]
    fn award_417780_zero_param_returns_none() {
        assert!(build_award_417780(true, 0, false).is_none());
    }
    #[test]
    fn award_417780_failed_registration_returns_none() {
        assert!(build_award_417780(false, 3, false).is_none());
    }
    #[test]
    fn award_417780_carries_param_and_flag() {
        let v = build_award_417780(true, 7, true).unwrap();
        assert_eq!(v.award_kind, 7);
        assert!(v.no_active_manager);
    }

    // ---- 0x00425e70 ----
    #[test]
    fn screen_425e70_failed_registration_returns_none() {
        assert!(build_screen_425e70(false, 42).is_none());
    }
    #[test]
    fn screen_425e70_zero_inits_reserved_slots() {
        let v = build_screen_425e70(true, 42).unwrap();
        assert_eq!(v.focus, 42);
        assert_eq!(v.reserved, [0, 0]);
    }

    // ---- 0x00470bf0 ----
    #[test]
    fn screen_470bf0_zero_param1_returns_none() {
        assert!(build_screen_470bf0(true, 0, 5, 0, 0).is_none());
    }
    #[test]
    fn screen_470bf0_zero_param2_returns_none() {
        assert!(build_screen_470bf0(true, 5, 0, 0, 0).is_none());
    }
    #[test]
    fn screen_470bf0_failed_registration_returns_none() {
        assert!(build_screen_470bf0(false, 1, 2, 3, 4).is_none());
    }
    #[test]
    fn screen_470bf0_slot2_is_constant_0x221() {
        let v = build_screen_470bf0(true, 1, 2, 3, 0).unwrap();
        assert_eq!(v.category, 0x221);
        assert_eq!(v.param_1, 1);
        assert_eq!(v.param_2, 2);
        assert_eq!(v.param_3, 3);
        assert_eq!(v.reserved, [0, 0]);
    }

    // ---- 0x00472bf0 ----
    #[test]
    fn screen_472bf0_zero_param1_returns_none() {
        assert!(build_screen_472bf0(true, 0, b'\n' as i8, 1).is_none());
    }
    #[test]
    fn screen_472bf0_vt_with_zero_param3_returns_none() {
        assert!(build_screen_472bf0(true, 1, b'\x0b' as i8, 0).is_none());
    }
    #[test]
    fn screen_472bf0_failed_registration_returns_none() {
        assert!(build_screen_472bf0(false, 1, b'\n' as i8, 1).is_none());
    }
    #[test]
    fn screen_472bf0_lf_branch_allocates_cm_cup() {
        let v = build_screen_472bf0(true, 1, b'\n' as i8, 99).unwrap();
        assert_eq!(v.one, 1);
        assert_eq!(v.kind, 0x14);
        assert_eq!(v.param_2, b'\n' as i32);
        assert_eq!(v.param_3, 99);
        let cup = v.cm_cup.expect("LF branch must allocate CM Cup");
        assert_eq!(cup.name, "CM Cup");
        assert_eq!(cup.byte_200, 2);
        assert_eq!(cup.byte_c9, 3);
        assert_eq!(cup.byte_ca, 1);
        assert_eq!(cup.word_cf, 4);
    }
    #[test]
    fn screen_472bf0_vt_branch_no_cm_cup_but_slots_pushed() {
        let v = build_screen_472bf0(true, 1, b'\x0b' as i8, 42).unwrap();
        assert!(v.cm_cup.is_none());
        assert_eq!(v.param_2, b'\x0b' as i32);
        assert_eq!(v.param_3, 42);
    }
    #[test]
    fn screen_472bf0_other_branch_no_cm_cup() {
        let v = build_screen_472bf0(true, 1, 0, 0).unwrap();
        assert!(v.cm_cup.is_none());
        assert_eq!(v.kind, 0x14);
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

/// Facade keys this batch reads. Anything not listed falls back to the
/// exe's null-pool default (usually zero) — see the note at the tail
/// of `world_facade.rs`.
pub const TODO_POPULATOR_INFO: &str = "\
b10: Award417780View reads bytes {award.param_1} + flags {award.no_active_manager}; \
Screen425e70View reads handles {screen_425e70.focus}; \
Screen470bf0View reads bytes {screen_470bf0.param_1, screen_470bf0.param_2} + \
handles {screen_470bf0.param_3, screen_470bf0.colour_pool_offset}; \
Screen472bf0View reads bytes {screen_472bf0.param_1, screen_472bf0.param_2, screen_472bf0.param_3}.\n\
UNKNOWNS: wave-B typed pools not yet available; \
`colour_pool_offset` should ultimately trace back to the colour-record pool \
(FUN_00470bf0 does `world.core.colours` arithmetic).";

use crate::world_facade::WorldFacade;

/// Populator for [`Award417780View`].
pub fn populate_award_417780(world: &WorldFacade) -> Option<Award417780View> {
    build_award_417780(
        world.registration_ok,
        world.byte("award.param_1") as i32,
        world.flag("award.no_active_manager"),
    )
}

/// Populator for [`Screen425e70View`].
pub fn populate_screen_425e70(world: &WorldFacade) -> Option<Screen425e70View> {
    build_screen_425e70(world.registration_ok, world.handle("screen_425e70.focus"))
}

/// Populator for [`Screen470bf0View`].
pub fn populate_screen_470bf0(world: &WorldFacade) -> Option<Screen470bf0View> {
    build_screen_470bf0(
        world.registration_ok,
        world.byte("screen_470bf0.param_1") as i32,
        world.byte("screen_470bf0.param_2") as i32,
        world.handle("screen_470bf0.param_3"),
        world.handle("screen_470bf0.colour_pool_offset"),
    )
}

/// Populator for [`Screen472bf0View`].
pub fn populate_screen_472bf0(world: &WorldFacade) -> Option<Screen472bf0View> {
    build_screen_472bf0(
        world.registration_ok,
        world.byte("screen_472bf0.param_1") as i32,
        world.byte("screen_472bf0.param_2") as i8,
        world.byte("screen_472bf0.param_3") as i32,
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_award_417780_zero_param_returns_none() {
        assert!(populate_award_417780(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_award_417780_carries_kind_and_flag() {
        let w = WorldFacade::ready()
            .with_byte("award.param_1", 7)
            .with_flag("award.no_active_manager", true);
        let v = populate_award_417780(&w).unwrap();
        assert_eq!(v.award_kind, 7);
        assert!(v.no_active_manager);
    }
    #[test]
    fn populate_screen_425e70_defaults_focus_zero() {
        let v = populate_screen_425e70(&WorldFacade::ready()).unwrap();
        assert_eq!(v.focus, 0);
    }
    #[test]
    fn populate_screen_470bf0_zero_params_returns_none() {
        assert!(populate_screen_470bf0(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_screen_470bf0_carries_all_fields() {
        let w = WorldFacade::ready()
            .with_byte("screen_470bf0.param_1", 5)
            .with_byte("screen_470bf0.param_2", 6)
            .with_handle("screen_470bf0.param_3", 0xdead)
            .with_handle("screen_470bf0.colour_pool_offset", 0x40);
        let v = populate_screen_470bf0(&w).unwrap();
        assert_eq!(v.param_1, 5);
        assert_eq!(v.param_3, 0xdead);
        assert_eq!(v.colour_pool_offset, 0x40);
    }
    #[test]
    fn populate_screen_472bf0_zero_param_returns_none() {
        assert!(populate_screen_472bf0(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_screen_472bf0_cm_cup_branch() {
        let w = WorldFacade::ready()
            .with_byte("screen_472bf0.param_1", 1)
            .with_byte("screen_472bf0.param_2", b'\n' as i32);
        let v = populate_screen_472bf0(&w).unwrap();
        assert!(v.cm_cup.is_some());
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A pattern)
// =====================================================================
//
// The WorldFacade populators above use the string-keyed handle facade
// (wave-B). These `*_from_pools` companions use the typed borrow-only
// `WorldPools` facade (wave-A, matches screen_batch8/9). Where a slot's
// exe source is opaque to the pools we currently expose, we fall back
// to a sensible default and document the gap in `TODO_POPULATOR_INFO_POOLS`.

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b10 pool sources: award kind + no_active_manager flag depend on the \
    award-registry pool and FUN_00418ca0 (not on the pools facade); \
    Screen470bf0View's colour_pool_offset needs the colour pool \
    (world.core.colours) keyed off the active human's nation record \
    (club+0x27); Screen472bf0View's param_2 branch needs a caller-supplied \
    char and there is no world-pool source for it. Defaults chosen so \
    every populator returns Some when an active human is seated.";

use crate::world_pools::WorldPools;

pub fn populate_award_417780_from_pools(pools: &WorldPools<'_>) -> Option<Award417780View> {
    // Award kind 1 whenever a human is seated; else exe's None branch.
    let seat = pools.active_human_seat?;
    build_award_417780(true, seat as i32, false)
}

pub fn populate_screen_425e70_from_pools(pools: &WorldPools<'_>) -> Option<Screen425e70View> {
    let focus = pools.active_human_seat.unwrap_or(0);
    build_screen_425e70(true, focus)
}

pub fn populate_screen_470bf0_from_pools(pools: &WorldPools<'_>) -> Option<Screen470bf0View> {
    // Requires two non-zero params; use the seat as both when present.
    let seat = pools.active_human_seat? as i32;
    build_screen_470bf0(true, seat, 1, 0, 0)
}

pub fn populate_screen_472bf0_from_pools(pools: &WorldPools<'_>) -> Option<Screen472bf0View> {
    let seat = pools.active_human_seat? as i32;
    build_screen_472bf0(true, seat, 0, 0)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn all_return_none_on_empty_pools() {
        let p = WorldPools::empty();
        assert!(populate_award_417780_from_pools(&p).is_none());
        assert!(populate_screen_470bf0_from_pools(&p).is_none());
        assert!(populate_screen_472bf0_from_pools(&p).is_none());
        // Screen425e70 has no non-zero requirement.
        assert!(populate_screen_425e70_from_pools(&p).is_some());
    }

    #[test]
    fn seat_enables_all_populators() {
        let p = WorldPools { active_human_seat: Some(3), ..WorldPools::empty() };
        assert!(populate_award_417780_from_pools(&p).is_some());
        assert!(populate_screen_425e70_from_pools(&p).is_some());
        assert!(populate_screen_470bf0_from_pools(&p).is_some());
        assert!(populate_screen_472bf0_from_pools(&p).is_some());
    }
}
