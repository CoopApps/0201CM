//! Batch 15: 5 more screen setup functions.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `00574960.c` — simple 3-slot screen (param, 0, 0), takes ptr to entity id
//! * `005792a0.c` — complex confirm/edit dialog (14 params, 19 slots, param
//!   validation MsgBox on `param_1|param_2|param_4 == 0`)
//! * `0057b9c0.c` — 8-slot screen with 3-entity + short pair + extra
//!   (validates non-zero param_1/2/3)
//! * `0057bb30.c` — variant of 0057b9c0: slot 2 forced 0, slot 6 is a
//!   helper object built from `FUN_004b5080(param_3)`
//! * `005928e0.c` — simple 3-slot screen (param, 1, [skip 2], 0 at slot 3)

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_00574960(param_1) — simple 3-slot screen
// =====================================================================

/// Screen 0x00574960 — 3 slots: (entity_id, 0, 0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen574960View {
    /// Slot 0: `*param_1` (dereferenced entity id).
    pub entity_id: u32,
    /// Slot 1: 0.
    pub reserved1: u32,
    /// Slot 2: 0.
    pub reserved2: u32,
}

/// Direct port of `FUN_00574960(param_1)`.
pub fn build_screen_574960(
    registration_ok: bool,
    entity_id: u32,
) -> Option<Screen574960View> {
    if !registration_ok { return None; }
    Some(Screen574960View { entity_id, reserved1: 0, reserved2: 0 })
}

// =====================================================================
// FUN_005792a0 — complex confirm dialog (19 slots)
// =====================================================================

/// Screen 0x005792a0 — confirm/edit dialog. 14 params → 19 slots, plus
/// three heap-allocated scratch buffers for text fields.
///
/// Exe fires MsgBox `find_screens:0x73` if `param_1|param_2|param_4 == 0`
/// and returns without registering — we mirror that by returning `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Screen5792A0View {
    /// Slots 0, 1: entity handles (param_1, param_2), both refcounted.
    pub entity_a: u32,
    pub entity_b: u32,
    /// Slot 2: mode/kind byte (param_3).
    pub mode: u32,
    /// Slot 3: raw text pointer (param_4) — same string used to fill
    /// `text_a_buf`.
    pub text_a_ptr: u32,
    /// Slot 4: optional refcounted handle (refcount only if non-zero).
    pub optional_handle: u32,
    /// Slot 5: param_7.
    pub extra5: u32,
    /// Slot 6: `_Dest` — 0x104-byte buffer, copy of `param_4` truncated
    /// to 0x103 + null.
    pub text_a_buf: Vec<u8>,
    /// Slot 7: `_Dest_00` — 0x100-byte buffer. If `mode==1 && param_6!=0`
    /// copy param_6 truncated 0xff+null, else empty.
    pub text_b_buf: Vec<u8>,
    /// Slot 8: `puVar2` — 0x104-byte scratch buffer starting with 0.
    pub text_c_buf: Vec<u8>,
    /// Slot 9: 0.
    pub reserved9: u32,
    /// Slot 0xa: `(int)param_8` (sign-extended short).
    pub short_a: i32,
    /// Slot 0xb: `(int)param_9`.
    pub short_b: i32,
    /// Slot 0xc: param_10.
    pub extra_c: u32,
    /// Slot 0xd: param_11.
    pub extra_d: u32,
    /// Slot 0xe, 0xf: 0.
    pub reserved_e: u32,
    pub reserved_f: u32,
    /// Slot 0x10, 0x11, 0x12: param_12/13/14.
    pub extra_10: u32,
    pub extra_11: u32,
    pub extra_12: u32,
}

/// Direct port of `FUN_005792a0(...)`.
#[allow(clippy::too_many_arguments)]
pub fn build_screen_5792a0(
    registration_ok: bool,
    entity_a: u32,
    entity_b: u32,
    mode: u32,
    text_a: Option<&str>,
    optional_handle: u32,
    text_b: Option<&str>,
    extra5: u32,
    short_a: i16,
    short_b: i16,
    extra_c: u32,
    extra_d: u32,
    extra_10: u32,
    extra_11: u32,
    extra_12: u32,
) -> Option<Screen5792A0View> {
    // Exe: `param_1 == 0 || param_2 == 0 || param_4 == 0` → MsgBox, no register.
    let text_a_src = text_a?;
    if entity_a == 0 || entity_b == 0 { return None; }
    if !registration_ok { return None; }

    // Slot 6: strncpy(_Dest, param_4, 0x104); _Dest[0x103] = 0;
    let mut text_a_buf = vec![0u8; 0x104];
    let src = text_a_src.as_bytes();
    let n = src.len().min(0x103);
    text_a_buf[..n].copy_from_slice(&src[..n]);
    // last byte already 0

    // Slot 7: if mode==1 && param_6 → strncpy 0x100; else *_Dest_00 = 0.
    let mut text_b_buf = vec![0u8; 0x100];
    if mode == 1 {
        if let Some(s) = text_b {
            let sb = s.as_bytes();
            let n = sb.len().min(0xff);
            text_b_buf[..n].copy_from_slice(&sb[..n]);
        }
    }

    // Slot 8: operator_new(0x104); *puVar2 = 0;
    let text_c_buf = vec![0u8; 0x104];

    // text_a_ptr is the raw pointer param_4 pushed at slot 3; we model
    // the string identity by echoing text_a's presence — the exact
    // pointer value isn't meaningful cross-address-space.
    Some(Screen5792A0View {
        entity_a, entity_b, mode,
        text_a_ptr: if text_a_src.is_empty() { 0 } else { 1 },
        optional_handle,
        extra5,
        text_a_buf, text_b_buf, text_c_buf,
        reserved9: 0,
        short_a: short_a as i32, short_b: short_b as i32,
        extra_c, extra_d,
        reserved_e: 0, reserved_f: 0,
        extra_10, extra_11, extra_12,
    })
}

// =====================================================================
// FUN_0057b9c0 — 8-slot screen, requires 3 non-zero entities
// =====================================================================

/// Screen 0x0057b9c0 — 8 slots. Refcounts entities at slots 0/1/2.
///
/// Exe fires MsgBox `find_screens:0x532` if any of param_1/2/3 is 0 and
/// skips registration → we return `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Screen57B9C0View {
    /// Slots 0, 1, 2: three refcounted entity handles.
    pub entity_a: u32,
    pub entity_b: u32,
    pub entity_c: u32,
    /// Slot 3: param_4.
    pub extra3: u32,
    /// Slot 4: `(int)param_5` (sign-extended short).
    pub short_a: i32,
    /// Slot 5: `(int)param_6`.
    pub short_b: i32,
    /// Slot 6: 0.
    pub reserved6: u32,
    /// Slot 7: param_7.
    pub extra7: u32,
}

/// Direct port of `FUN_0057b9c0(...)`.
#[allow(clippy::too_many_arguments)]
pub fn build_screen_57b9c0(
    registration_ok: bool,
    entity_a: u32, entity_b: u32, entity_c: u32,
    extra3: u32, short_a: i16, short_b: i16, extra7: u32,
) -> Option<Screen57B9C0View> {
    if entity_a == 0 || entity_b == 0 || entity_c == 0 { return None; }
    if !registration_ok { return None; }
    Some(Screen57B9C0View {
        entity_a, entity_b, entity_c, extra3,
        short_a: short_a as i32, short_b: short_b as i32,
        reserved6: 0, extra7,
    })
}

// =====================================================================
// FUN_0057bb30 — variant of 57b9c0 with slot 2 = 0, slot 6 = helper obj
// =====================================================================

/// Screen 0x0057bb30 — same 8-slot shape as 0x0057b9c0 but:
/// * slot 2 is forced to 0 (entity_c is only consumed by the helper),
/// * slot 6 is a helper object `FUN_004b5080(param_3)` boxed via
///   `operator_new(8)` (null-alloc → helper is 0).
///
/// Same validation: MsgBox `find_screens:0x55b` on any zero of
/// param_1/2/3 → return `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Screen57BB30View {
    /// Slot 0: refcounted param_1.
    pub entity_a: u32,
    /// Slot 1: refcounted param_2.
    pub entity_b: u32,
    /// Slot 2: 0 (unlike 0x57b9c0).
    pub reserved2: u32,
    /// Slot 3: param_4.
    pub extra3: u32,
    /// Slot 4/5: sign-extended shorts.
    pub short_a: i32,
    pub short_b: i32,
    /// Slot 6: derived helper — `FUN_004b5080(param_3)` when alloc
    /// succeeded, else 0. We model with a bool + carried source id.
    pub helper_from_entity_c: u32,
    /// Slot 7: param_7.
    pub extra7: u32,
}

/// Direct port of `FUN_0057bb30(...)`. `alloc_ok` mirrors the
/// `operator_new(8) != NULL` gate — false zeros the helper.
#[allow(clippy::too_many_arguments)]
pub fn build_screen_57bb30(
    registration_ok: bool,
    entity_a: u32, entity_b: u32, entity_c: u32,
    extra3: u32, short_a: i16, short_b: i16, extra7: u32,
    alloc_ok: bool,
) -> Option<Screen57BB30View> {
    if entity_a == 0 || entity_b == 0 || entity_c == 0 { return None; }
    let helper = if alloc_ok { entity_c } else { 0 };
    if !registration_ok { return None; }
    Some(Screen57BB30View {
        entity_a, entity_b, reserved2: 0, extra3,
        short_a: short_a as i32, short_b: short_b as i32,
        helper_from_entity_c: helper, extra7,
    })
}

// =====================================================================
// FUN_005928e0(param_1) — 3-slot screen (slot 2 skipped)
// =====================================================================

/// Screen 0x005928e0 — 3 slots: (entity, 1, [skip 2], 0 at slot 3).
/// Note the exe writes slots 0/1/3 — slot 2 is intentionally not written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen5928E0View {
    /// Slot 0: `*param_1`.
    pub entity_id: u32,
    /// Slot 1: 1 (constant tag).
    pub tag: u32,
    /// Slot 3: 0.
    pub reserved3: u32,
}

/// Direct port of `FUN_005928e0(param_1)`.
pub fn build_screen_5928e0(
    registration_ok: bool,
    entity_id: u32,
) -> Option<Screen5928E0View> {
    if !registration_ok { return None; }
    Some(Screen5928E0View { entity_id, tag: 1, reserved3: 0 })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- FUN_00574960 ----
    #[test]
    fn screen_574960_carries_entity_id() {
        let v = build_screen_574960(true, 12345).unwrap();
        assert_eq!(v.entity_id, 12345);
        assert_eq!(v.reserved1, 0);
        assert_eq!(v.reserved2, 0);
    }
    #[test]
    fn screen_574960_failed_registration() {
        assert!(build_screen_574960(false, 12345).is_none());
    }

    // ---- FUN_005792a0 ----
    #[test]
    fn screen_5792a0_zero_entity_a_returns_none() {
        assert!(build_screen_5792a0(true, 0, 2, 1, Some("hi"), 0, None, 0, 0, 0, 0, 0, 0, 0, 0).is_none());
    }
    #[test]
    fn screen_5792a0_zero_entity_b_returns_none() {
        assert!(build_screen_5792a0(true, 1, 0, 1, Some("hi"), 0, None, 0, 0, 0, 0, 0, 0, 0, 0).is_none());
    }
    #[test]
    fn screen_5792a0_null_text_a_returns_none() {
        assert!(build_screen_5792a0(true, 1, 2, 1, None, 0, None, 0, 0, 0, 0, 0, 0, 0, 0).is_none());
    }
    #[test]
    fn screen_5792a0_normal_flow_populates_buffers() {
        let v = build_screen_5792a0(true, 1, 2, 1, Some("hello"),
            0, Some("world"), 7, -3, 4, 10, 11, 12, 13, 14).unwrap();
        assert_eq!(v.entity_a, 1);
        assert_eq!(v.entity_b, 2);
        assert_eq!(v.mode, 1);
        assert_eq!(v.text_a_buf.len(), 0x104);
        assert_eq!(&v.text_a_buf[..5], b"hello");
        assert_eq!(v.text_a_buf[0x103], 0);
        assert_eq!(v.text_b_buf.len(), 0x100);
        assert_eq!(&v.text_b_buf[..5], b"world");
        assert_eq!(v.text_c_buf.len(), 0x104);
        assert_eq!(v.text_c_buf[0], 0);
        assert_eq!(v.short_a, -3);
        assert_eq!(v.short_b, 4);
        assert_eq!(v.extra_c, 10);
        assert_eq!(v.extra_12, 14);
    }
    #[test]
    fn screen_5792a0_text_b_empty_when_mode_ne_1() {
        let v = build_screen_5792a0(true, 1, 2, 2, Some("hi"), 0, Some("world"),
            0, 0, 0, 0, 0, 0, 0, 0).unwrap();
        assert_eq!(v.text_b_buf[0], 0);
    }

    // ---- FUN_0057b9c0 ----
    #[test]
    fn screen_57b9c0_requires_all_three_entities() {
        assert!(build_screen_57b9c0(true, 0, 2, 3, 0, 0, 0, 0).is_none());
        assert!(build_screen_57b9c0(true, 1, 0, 3, 0, 0, 0, 0).is_none());
        assert!(build_screen_57b9c0(true, 1, 2, 0, 0, 0, 0, 0).is_none());
    }
    #[test]
    fn screen_57b9c0_carries_shorts_and_slots() {
        let v = build_screen_57b9c0(true, 1, 2, 3, 99, -5, 7, 42).unwrap();
        assert_eq!(v.entity_a, 1);
        assert_eq!(v.entity_c, 3);
        assert_eq!(v.short_a, -5);
        assert_eq!(v.short_b, 7);
        assert_eq!(v.reserved6, 0);
        assert_eq!(v.extra7, 42);
    }

    // ---- FUN_0057bb30 ----
    #[test]
    fn screen_57bb30_slot2_is_zero_and_helper_from_entity_c() {
        let v = build_screen_57bb30(true, 1, 2, 3, 0, 0, 0, 0, true).unwrap();
        assert_eq!(v.reserved2, 0);
        assert_eq!(v.helper_from_entity_c, 3);
    }
    #[test]
    fn screen_57bb30_alloc_fail_zeros_helper() {
        let v = build_screen_57bb30(true, 1, 2, 3, 0, 0, 0, 0, false).unwrap();
        assert_eq!(v.helper_from_entity_c, 0);
    }
    #[test]
    fn screen_57bb30_zero_entity_returns_none() {
        assert!(build_screen_57bb30(true, 1, 2, 0, 0, 0, 0, 0, true).is_none());
    }

    // ---- FUN_005928e0 ----
    #[test]
    fn screen_5928e0_tag_constant_is_one() {
        let v = build_screen_5928e0(true, 555).unwrap();
        assert_eq!(v.entity_id, 555);
        assert_eq!(v.tag, 1);
        assert_eq!(v.reserved3, 0);
    }
    #[test]
    fn screen_5928e0_failed_registration() {
        assert!(build_screen_5928e0(false, 1).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b15: Screen574960View reads handle {b15.574960.entity_id}; \
Screen5792A0View reads handles {b15.5792a0.entity_a, entity_b, mode, \
optional_handle, extra5, extra_c, extra_d, extra_10, extra_11, extra_12} + \
strings {b15.5792a0.text_a, b15.5792a0.text_b} + bytes {b15.5792a0.short_a, \
short_b}; Screen57B9C0View reads handles {b15.57b9c0.entity_a, entity_b, \
entity_c, extra3, extra7} + bytes {b15.57b9c0.short_a, short_b}; \
Screen57BB30View reads the same plus flag {b15.57bb30.alloc_ok}; \
Screen5928E0View reads handle {b15.5928e0.entity_id}.\n\
UNKNOWNS: wave-B typed pools not yet available; text_a / text_b originate \
from the string-pool caller — the typed accessor doesn't yet expose it.";

use crate::world_facade::WorldFacade;

/// Populator for [`Screen574960View`].
pub fn populate_screen_574960(world: &WorldFacade) -> Option<Screen574960View> {
    build_screen_574960(world.registration_ok, world.handle("b15.574960.entity_id"))
}

/// Populator for [`Screen5792A0View`].
pub fn populate_screen_5792a0(world: &WorldFacade) -> Option<Screen5792A0View> {
    build_screen_5792a0(
        world.registration_ok,
        world.handle("b15.5792a0.entity_a"),
        world.handle("b15.5792a0.entity_b"),
        world.handle("b15.5792a0.mode"),
        world.text("b15.5792a0.text_a"),
        world.handle("b15.5792a0.optional_handle"),
        world.text("b15.5792a0.text_b"),
        world.handle("b15.5792a0.extra5"),
        world.byte("b15.5792a0.short_a") as i16,
        world.byte("b15.5792a0.short_b") as i16,
        world.handle("b15.5792a0.extra_c"),
        world.handle("b15.5792a0.extra_d"),
        world.handle("b15.5792a0.extra_10"),
        world.handle("b15.5792a0.extra_11"),
        world.handle("b15.5792a0.extra_12"),
    )
}

/// Populator for [`Screen57B9C0View`].
pub fn populate_screen_57b9c0(world: &WorldFacade) -> Option<Screen57B9C0View> {
    build_screen_57b9c0(
        world.registration_ok,
        world.handle("b15.57b9c0.entity_a"),
        world.handle("b15.57b9c0.entity_b"),
        world.handle("b15.57b9c0.entity_c"),
        world.handle("b15.57b9c0.extra3"),
        world.byte("b15.57b9c0.short_a") as i16,
        world.byte("b15.57b9c0.short_b") as i16,
        world.handle("b15.57b9c0.extra7"),
    )
}

/// Populator for [`Screen57BB30View`].
pub fn populate_screen_57bb30(world: &WorldFacade) -> Option<Screen57BB30View> {
    build_screen_57bb30(
        world.registration_ok,
        world.handle("b15.57bb30.entity_a"),
        world.handle("b15.57bb30.entity_b"),
        world.handle("b15.57bb30.entity_c"),
        world.handle("b15.57bb30.extra3"),
        world.byte("b15.57bb30.short_a") as i16,
        world.byte("b15.57bb30.short_b") as i16,
        world.handle("b15.57bb30.extra7"),
        world.flag("b15.57bb30.alloc_ok"),
    )
}

/// Populator for [`Screen5928E0View`].
pub fn populate_screen_5928e0(world: &WorldFacade) -> Option<Screen5928E0View> {
    build_screen_5928e0(world.registration_ok, world.handle("b15.5928e0.entity_id"))
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_screen_574960_ok() {
        assert!(populate_screen_574960(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_screen_5792a0_missing_text_returns_none() {
        assert!(populate_screen_5792a0(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_screen_5792a0_needs_nonzero_entities() {
        let w = WorldFacade::ready().with_text("b15.5792a0.text_a", "hi");
        assert!(populate_screen_5792a0(&w).is_none()); // entities still zero
        let w = w
            .with_handle("b15.5792a0.entity_a", 1)
            .with_handle("b15.5792a0.entity_b", 2);
        let v = populate_screen_5792a0(&w).unwrap();
        assert_eq!(v.entity_a, 1);
    }
    #[test]
    fn populate_screen_57b9c0_zero_entity_returns_none() {
        assert!(populate_screen_57b9c0(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_screen_57bb30_alloc_ok_flag_passes_through() {
        let w = WorldFacade::ready()
            .with_handle("b15.57bb30.entity_a", 1)
            .with_handle("b15.57bb30.entity_b", 2)
            .with_handle("b15.57bb30.entity_c", 3)
            .with_flag("b15.57bb30.alloc_ok", true);
        let v = populate_screen_57bb30(&w).unwrap();
        assert_eq!(v.helper_from_entity_c, 3);
    }
    #[test]
    fn populate_screen_5928e0_ok() {
        assert!(populate_screen_5928e0(&WorldFacade::ready()).is_some());
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A pattern)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b15 pool sources: Screen574960/5928e0 want any entity id — we use \
    the active human seat. Screen5792A0 needs text_a from a user prompt \
    (cmd-arg). Screen57b9c0/57bb30 require three simultaneously-nonzero \
    entity handles; without more decoded pools we return None on empty.";

use crate::world_pools::WorldPools;

pub fn populate_screen_574960_from_pools(
    pools: &WorldPools<'_>,
) -> Option<Screen574960View> {
    let id = pools.active_human_seat.unwrap_or(0);
    build_screen_574960(true, id)
}

pub fn populate_screen_5792a0_from_pools(
    pools: &WorldPools<'_>,
) -> Option<Screen5792A0View> {
    // Requires two non-zero entities + a non-empty text_a. Without the
    // cmd-arg source we default to the seat + a placeholder title.
    let a = pools.active_human_seat?;
    let b = pools.active_human_club_id().unwrap_or(a);
    build_screen_5792a0(true, a, b, 0, Some(""), 0, None,
        0, 0, 0, 0, 0, 0, 0, 0)
}

pub fn populate_screen_57b9c0_from_pools(
    pools: &WorldPools<'_>,
) -> Option<Screen57B9C0View> {
    let a = pools.active_human_seat?;
    let b = pools.active_human_club_id()?;
    let c = pools.active_human_focus_competition()?;
    build_screen_57b9c0(true, a, b, c, 0, 0, 0, 0)
}

pub fn populate_screen_57bb30_from_pools(
    pools: &WorldPools<'_>,
) -> Option<Screen57BB30View> {
    let a = pools.active_human_seat?;
    let b = pools.active_human_club_id()?;
    let c = pools.active_human_focus_competition()?;
    build_screen_57bb30(true, a, b, c, 0, 0, 0, 0, true)
}

pub fn populate_screen_5928e0_from_pools(
    pools: &WorldPools<'_>,
) -> Option<Screen5928E0View> {
    let id = pools.active_human_seat.unwrap_or(0);
    build_screen_5928e0(true, id)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn empty_pools_return_expected() {
        let p = WorldPools::empty();
        assert!(populate_screen_574960_from_pools(&p).is_some());
        assert!(populate_screen_5792a0_from_pools(&p).is_none());
        assert!(populate_screen_57b9c0_from_pools(&p).is_none());
        assert!(populate_screen_57bb30_from_pools(&p).is_none());
        assert!(populate_screen_5928e0_from_pools(&p).is_some());
    }
}
