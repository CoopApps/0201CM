//! Batch 20: 5 more setup functions clustered in the 0x0080xxxx setup band.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `00809ad0.c` — Human-manager screen setup (1 slot). Runs a first-time
//!   `DAT_00b59fc2[human*0xc0]==0` allocation cascade that walks the
//!   human-manager pool and grabs a free slot, then registers the screen.
//! * `0080bbd0.c` — Simple 1-slot screen builder (passes `param_1`).
//! * `0080cc20.c` — 2-slot screen builder — allocates 0x31B-byte struct
//!   via `operator_new`, seeds it via `FUN_005c1970(DAT_00acdf28)`,
//!   pushes it as slot 0 and `param_1` as slot 1.
//! * `008109c0.c` — 1-slot screen builder gated by `DAT_00dbc3f0` flag.
//! * `00810ca0.c` — 1-slot screen builder gated by `DAT_00dbc3f0` flag
//!   (sister of 008109c0 with a different pair of layout labels).

use serde::{Deserialize, Serialize};

// =====================================================================
// 0x00809ad0 — Human-manager screen (1 slot + first-time init cascade)
// =====================================================================

/// Human-manager screen view.
///
/// The exe reads the current human's seat pointer at
/// `DAT_00b59fc2 + DAT_00b5d016 * 0xc0` (see `dashboard-manager-model`);
/// if 0, it either walks the manager pool for a free slot (found via
/// `iVar3 + 0x5f == 0`) or, when the pool base `DAT_00acd5c4` is 0,
/// pops a MsgBox and *bails* (`DAT_00b4d5a8 = 0`). After the cascade the
/// screen registers with a single slot 0 = 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanManagerSetupView {
    /// Slot 0: always zero (FUN_007E7130(0, 0, 0)).
    pub slot0: u32,
    /// Whether the first-time init cascade ran and found a free seat.
    pub allocated_seat: bool,
    /// Whether the MsgBox-error path fired (pool base was 0).
    pub pool_error: bool,
}

impl Default for HumanManagerSetupView {
    fn default() -> Self {
        Self { slot0: 0, allocated_seat: false, pool_error: false }
    }
}

/// Direct port of `FUN_00809ad0`.
///
/// * `human_seat_already_set` — non-zero `DAT_00b59fc2[human*0xc0]`,
///   skips the entire allocation cascade.
/// * `pool_base_nonzero` — `DAT_00acd5c4 != 0`; when false the exe fires
///   the "Setup" MsgBox error and clears `DAT_00b4d5a8`.
/// * `registration_ok` — `FUN_007e6570` return.
pub fn build_human_manager_setup(
    human_seat_already_set: bool,
    pool_base_nonzero: bool,
    registration_ok: bool,
) -> Option<HumanManagerSetupView> {
    if !registration_ok {
        return None;
    }
    let (allocated_seat, pool_error) = if human_seat_already_set {
        (false, false)
    } else if pool_base_nonzero {
        (true, false)
    } else {
        (false, true)
    };
    Some(HumanManagerSetupView { slot0: 0, allocated_seat, pool_error })
}

// =====================================================================
// 0x0080bbd0 — 1-slot screen builder
// =====================================================================

/// Simple 1-slot screen — pushes `param_1` at slot 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen0080BBD0View {
    /// Slot 0: caller-supplied focus value.
    pub focus: u32,
}

/// Direct port of `FUN_0080bbd0(param_1)`.
pub fn build_screen_0080bbd0(
    registration_ok: bool,
    focus: u32,
) -> Option<Screen0080BBD0View> {
    if !registration_ok {
        return None;
    }
    Some(Screen0080BBD0View { focus })
}

// =====================================================================
// 0x0080cc20 — 2-slot screen builder with 0x31B-byte scratch
// =====================================================================

/// 2-slot screen — allocates a 0x31B-byte scratch object seeded via
/// `FUN_005c1970(DAT_00acdf28)` and pushes it as slot 0 alongside the
/// caller's `param_1` as slot 1. The exe zeroes slot 0's handle when the
/// allocator returns null.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen0080CC20View {
    /// Slot 0: seeded scratch handle (0 when allocation failed).
    pub scratch_handle: u32,
    /// Slot 1: caller-supplied focus.
    pub focus: u32,
}

/// Direct port of `FUN_0080cc20(param_1)`.
///
/// * `alloc_ok` — false when `operator_new(0x31B)` returned null; slot 0
///   is then zero.
/// * `seeded_handle` — the value `FUN_005c1970(DAT_00acdf28)` returned
///   when the allocation succeeded.
pub fn build_screen_0080cc20(
    registration_ok: bool,
    alloc_ok: bool,
    seeded_handle: u32,
    focus: u32,
) -> Option<Screen0080CC20View> {
    if !registration_ok {
        return None;
    }
    let scratch_handle = if alloc_ok { seeded_handle } else { 0 };
    Some(Screen0080CC20View { scratch_handle, focus })
}

// =====================================================================
// 0x008109c0 — 1-slot screen builder (gated by DAT_00dbc3f0)
// =====================================================================

/// 1-slot screen — `FUN_007e6570` receives `DAT_00dbc3f0` as `param_3`
/// (used by the widget registrar as a visibility gate); we mirror it as
/// an explicit gate parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008109C0View {
    /// Slot 0: caller-supplied focus.
    pub focus: u32,
}

/// Direct port of `FUN_008109c0(param_1)`.
pub fn build_screen_008109c0(
    registration_ok: bool,
    focus: u32,
) -> Option<Screen008109C0View> {
    if !registration_ok {
        return None;
    }
    Some(Screen008109C0View { focus })
}

// =====================================================================
// 0x00810ca0 — 1-slot screen builder (sister of 008109c0)
// =====================================================================

/// 1-slot screen — identical shape to `Screen008109C0View` but wired to
/// a different label pair (`LAB_00810ce0`/`LAB_00810e90`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen00810CA0View {
    /// Slot 0: caller-supplied focus.
    pub focus: u32,
}

/// Direct port of `FUN_00810ca0(param_1)`.
pub fn build_screen_00810ca0(
    registration_ok: bool,
    focus: u32,
) -> Option<Screen00810CA0View> {
    if !registration_ok {
        return None;
    }
    Some(Screen00810CA0View { focus })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 0x00809ad0 ----
    #[test]
    fn human_manager_failed_registration_returns_none() {
        assert!(build_human_manager_setup(true, true, false).is_none());
    }
    #[test]
    fn human_manager_seat_already_set_skips_cascade() {
        let v = build_human_manager_setup(true, true, true).unwrap();
        assert_eq!(v.slot0, 0);
        assert!(!v.allocated_seat);
        assert!(!v.pool_error);
    }
    #[test]
    fn human_manager_first_time_with_pool_allocates() {
        let v = build_human_manager_setup(false, true, true).unwrap();
        assert!(v.allocated_seat);
        assert!(!v.pool_error);
    }
    #[test]
    fn human_manager_first_time_no_pool_fires_error() {
        let v = build_human_manager_setup(false, false, true).unwrap();
        assert!(!v.allocated_seat);
        assert!(v.pool_error);
    }

    // ---- 0x0080bbd0 ----
    #[test]
    fn screen_0080bbd0_carries_focus() {
        let v = build_screen_0080bbd0(true, 0xdead_beef).unwrap();
        assert_eq!(v.focus, 0xdead_beef);
    }
    #[test]
    fn screen_0080bbd0_failed_registration_returns_none() {
        assert!(build_screen_0080bbd0(false, 42).is_none());
    }

    // ---- 0x0080cc20 ----
    #[test]
    fn screen_0080cc20_normal_path_carries_both_slots() {
        let v = build_screen_0080cc20(true, true, 0x1234, 0x5678).unwrap();
        assert_eq!(v.scratch_handle, 0x1234);
        assert_eq!(v.focus, 0x5678);
    }
    #[test]
    fn screen_0080cc20_alloc_failure_zeroes_slot0() {
        let v = build_screen_0080cc20(true, false, 0x1234, 0x5678).unwrap();
        assert_eq!(v.scratch_handle, 0);
        assert_eq!(v.focus, 0x5678);
    }
    #[test]
    fn screen_0080cc20_failed_registration_returns_none() {
        assert!(build_screen_0080cc20(false, true, 0, 0).is_none());
    }

    // ---- 0x008109c0 ----
    #[test]
    fn screen_008109c0_carries_focus() {
        let v = build_screen_008109c0(true, 7).unwrap();
        assert_eq!(v.focus, 7);
    }
    #[test]
    fn screen_008109c0_failed_registration_returns_none() {
        assert!(build_screen_008109c0(false, 7).is_none());
    }

    // ---- 0x00810ca0 ----
    #[test]
    fn screen_00810ca0_carries_focus() {
        let v = build_screen_00810ca0(true, 99).unwrap();
        assert_eq!(v.focus, 99);
    }
    #[test]
    fn screen_00810ca0_failed_registration_returns_none() {
        assert!(build_screen_00810ca0(false, 99).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b20: HumanManagerSetupView reads flags {human_seat_already_set, \
pool_base_nonzero}; Screen0080BBD0View/Screen008109C0View/Screen00810CA0View \
each read handle `focus`; Screen0080CC20View reads flag `alloc_ok` + \
handles {seeded_handle, focus}.\n\
UNKNOWNS: the first-time-init cascade (walking DAT_00acd5c4 for a free \
seat and mutating it) is not modelled here — populator only signals the \
outcome. Wire to a real ManagerPool when wave-B lands.";

use crate::world_facade::WorldFacade;

pub fn populate_human_manager_setup(world: &WorldFacade) -> Option<HumanManagerSetupView> {
    build_human_manager_setup(
        world.flag("human_seat_already_set"),
        world.flag("pool_base_nonzero"),
        world.registration_ok,
    )
}

pub fn populate_screen_0080bbd0(world: &WorldFacade) -> Option<Screen0080BBD0View> {
    build_screen_0080bbd0(world.registration_ok, world.handle("focus"))
}

pub fn populate_screen_0080cc20(world: &WorldFacade) -> Option<Screen0080CC20View> {
    build_screen_0080cc20(
        world.registration_ok,
        world.flag("alloc_ok"),
        world.handle("seeded_handle"),
        world.handle("focus"),
    )
}

pub fn populate_screen_008109c0(world: &WorldFacade) -> Option<Screen008109C0View> {
    build_screen_008109c0(world.registration_ok, world.handle("focus"))
}

pub fn populate_screen_00810ca0(world: &WorldFacade) -> Option<Screen00810CA0View> {
    build_screen_00810ca0(world.registration_ok, world.handle("focus"))
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_human_manager_setup_no_pool_fires_error() {
        let v = populate_human_manager_setup(&WorldFacade::ready()).unwrap();
        assert!(v.pool_error);
    }
    #[test]
    fn populate_screen_0080bbd0_carries_focus() {
        let w = WorldFacade::ready().with_handle("focus", 42);
        assert_eq!(populate_screen_0080bbd0(&w).unwrap().focus, 42);
    }
    #[test]
    fn populate_screen_0080cc20_alloc_failure_zeroes_slot0() {
        let w = WorldFacade::ready().with_handle("seeded_handle", 0x1234);
        let v = populate_screen_0080cc20(&w).unwrap();
        assert_eq!(v.scratch_handle, 0);
    }
    #[test]
    fn populate_screen_008109c0_ok() {
        assert!(populate_screen_008109c0(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_screen_00810ca0_ok() {
        assert!(populate_screen_00810ca0(&WorldFacade::ready()).is_some());
    }
}
