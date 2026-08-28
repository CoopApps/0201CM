//! Batch 14: 5 more screen-setup functions.
//!
//! Decompiles at `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `0x004EB240.c` — 7-param setup, slots 0..=7 (slots 1/2 dereferenced
//!   from param_1 record, else 0xFFFFFFFF sentinel).
//! * `0x004EBF60.c` — 1-param setup, slot 0.
//! * `0x004EC550.c` — no-arg setup, slots 0 & 1 zero.
//! * `0x004FD1B0.c` — no-arg setup, slots 0 & 1 zero.
//! * `0x00548170.c` — 2-param setup with a 15-field decode cascade
//!   (FUN_0076eb10 + 15× FUN_0076d7d0) feeding ~20 slots including
//!   three indirected record fields at DAT_00acd5c4[index * 0x6e].

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_004EB240 — 7-param setup, 8 slots
// =====================================================================

/// View built by `FUN_004eb240(param_1, p2, p3, p4, p5, p6)`.
///
/// param_1 is a pointer to a record; if non-null and `*param_1 != 0`
/// slots 1 and 2 pull `[record+0x20]` and `[record+0x24]` respectively,
/// otherwise both are set to the sentinel `0xFFFFFFFF`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Screen4Eb240View {
    /// Slot 0: raw param_1 pointer value.
    pub param1_ptr: u32,
    /// Slot 1: `*param_1 + 0x20` or sentinel.
    pub slot1: u32,
    /// Slot 2: `*param_1 + 0x24` or sentinel.
    pub slot2: u32,
    /// Slot 3: param_2 as i8.
    pub p2: i8,
    /// Slot 4: param_3 as i8.
    pub p3: i8,
    /// Slot 5: param_4.
    pub p4: u32,
    /// Slot 6: param_6 as i8 (NB: slot 6 = param_6, slot 7 = param_5 in exe).
    pub p6: i8,
    /// Slot 7: param_5.
    pub p5: u32,
}

/// Direct port of `FUN_004eb240`.
///
/// `record_slots` models the `(param_1 != NULL && *param_1 != 0)`
/// branch — pass `Some((v1, v2))` for the loaded-record path, `None`
/// for the sentinel (0xFFFFFFFF) path.
pub fn build_screen_4eb240(
    registration_ok: bool,
    param1_ptr: u32,
    p2: i8, p3: i8, p4: u32, p5: u32, p6: i8,
    record_slots: Option<(u32, u32)>,
) -> Option<Screen4Eb240View> {
    if !registration_ok { return None; }
    let (slot1, slot2) = record_slots.unwrap_or((0xFFFF_FFFF, 0xFFFF_FFFF));
    Some(Screen4Eb240View { param1_ptr, slot1, slot2, p2, p3, p4, p6, p5 })
}

// =====================================================================
// FUN_004EBF60 — 1-param setup, 1 slot
// =====================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen4Ebf60View {
    /// Slot 0: param_1.
    pub p1: u32,
}

/// Direct port of `FUN_004ebf60(param_1)`.
pub fn build_screen_4ebf60(registration_ok: bool, p1: u32) -> Option<Screen4Ebf60View> {
    if !registration_ok { return None; }
    Some(Screen4Ebf60View { p1 })
}

// =====================================================================
// FUN_004EC550 — no-arg setup, 2 zero slots
// =====================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen4Ec550View {
    /// Slot 0: zero.
    pub slot0: u32,
    /// Slot 1: zero.
    pub slot1: u32,
}

/// Direct port of `FUN_004ec550`.
pub fn build_screen_4ec550(registration_ok: bool) -> Option<Screen4Ec550View> {
    if !registration_ok { return None; }
    Some(Screen4Ec550View::default())
}

// =====================================================================
// FUN_004FD1B0 — no-arg setup, 2 zero slots
// =====================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen4Fd1b0View {
    pub slot0: u32,
    pub slot1: u32,
}

/// Direct port of `FUN_004fd1b0`.
pub fn build_screen_4fd1b0(registration_ok: bool) -> Option<Screen4Fd1b0View> {
    if !registration_ok { return None; }
    Some(Screen4Fd1b0View::default())
}

// =====================================================================
// FUN_00548170 — 2-param setup with 15-field decode cascade, ~20 slots
// =====================================================================

/// Fifteen bytes/shorts/dwords pulled from a serialised blob by fifteen
/// consecutive `FUN_0076d7d0(cursor++)` calls, feeding the slot writes.
///
/// Modelled as a plain struct so callers can construct it directly for
/// tests without emulating the decoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen548170Decode {
    /// Cascade field 0 → slot 0 (i8).
    pub f_113: i8,
    /// Cascade field 1 → slot 0x14 (u32).
    pub f_10c: u32,
    /// Cascade field 2 → slot 5 (i8).
    pub f_115: i8,
    /// Cascade field 3 → slot 6 (i16, sign-extended).
    pub f_110: i16,
    /// Cascade field 4 → slot 8 (u16).
    pub f_slot8: u16,
    /// Cascade field 5 → slot 9 (u16).
    pub f_slot9: u16,
    /// Cascade field 6 → slot 0xA (i8).
    pub f_11c: i8,
    /// Cascade field 7 → slot 0xB (i8).
    pub f_11b: i8,
    /// Cascade field 8 → slot 0xC (i8).
    pub f_11a: i8,
    /// Cascade field 9 → slot 0xD (i8).
    pub f_119: i8,
    /// Cascade field 10 → slot 0xE (i8).
    pub f_111: i8,
    /// Cascade field 11 → slot 0xF (i8).
    pub f_112: i8,
    /// Cascade field 12 → slot 0x10 (u32).
    pub f_108: u32,
    /// Cascade field 13 → slot 0x12 (i8).
    pub f_114: i8,
    /// Cascade field 14 → slot 0x13 (i8).
    pub f_11d: i8,
}

/// Three record-derived slots (1, 2, 3): each is
/// `*(u32*)(DAT_00acd5c4[idx * 0x6e] . ptr[N] + 0x33)` for N in 1..=3.
/// Slot 4 is the raw first pointer at `puVar1[0]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen548170Record {
    /// Slot 4: `puVar1[0]`.
    pub base_ptr: u32,
    /// Slot 1: `*(puVar1[1] + 0x33)`.
    pub field1_at_33: u32,
    /// Slot 2: `*(puVar1[2] + 0x33)`.
    pub field2_at_33: u32,
    /// Slot 3: `*(puVar1[3] + 0x33)`.
    pub field3_at_33: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Screen548170View {
    /// Slot 0.
    pub slot0: i8,
    /// Slots 1..=4 (record indirections).
    pub record: Screen548170Record,
    /// Slot 5.
    pub slot5: i8,
    /// Slot 6.
    pub slot6: i16,
    /// Slot 8.
    pub slot8: u16,
    /// Slot 9.
    pub slot9: u16,
    /// Slots 0xA..=0xF (all bytes from cascade).
    pub slot_a: i8,
    pub slot_b: i8,
    pub slot_c: i8,
    pub slot_d: i8,
    pub slot_e: i8,
    pub slot_f: i8,
    /// Slot 0x10.
    pub slot_10: u32,
    /// Slot 0x11: param_2 (passed through unchanged).
    pub slot_11: u32,
    /// Slot 0x12.
    pub slot_12: i8,
    /// Slot 0x13.
    pub slot_13: i8,
    /// Slot 0x14.
    pub slot_14: u32,
}

/// Direct port of `FUN_00548170(param_1, param_2)`.
///
/// * `decode` is the result of the 15-field `FUN_0076d7d0` cascade
///   (fed from the `FUN_0076eb10(param_1, param_2, buf)` blob).
/// * `record` is the trio+base derived from
///   `DAT_00acd5c4 + local_f8 * 0x6e` (the index chosen by the decoder).
pub fn build_screen_548170(
    registration_ok: bool,
    param_2: u32,
    decode: Screen548170Decode,
    record: Screen548170Record,
) -> Option<Screen548170View> {
    if !registration_ok { return None; }
    Some(Screen548170View {
        slot0: decode.f_113,
        record,
        slot5: decode.f_115,
        slot6: decode.f_110,
        slot8: decode.f_slot8,
        slot9: decode.f_slot9,
        slot_a: decode.f_11c,
        slot_b: decode.f_11b,
        slot_c: decode.f_11a,
        slot_d: decode.f_119,
        slot_e: decode.f_111,
        slot_f: decode.f_112,
        slot_10: decode.f_108,
        slot_11: param_2,
        slot_12: decode.f_114,
        slot_13: decode.f_11d,
        slot_14: decode.f_10c,
    })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 4eb240 ----
    #[test]
    fn s4eb240_failed_registration_returns_none() {
        assert!(build_screen_4eb240(false, 0, 0, 0, 0, 0, 0, None).is_none());
    }
    #[test]
    fn s4eb240_null_record_uses_sentinel_for_slots_1_and_2() {
        let v = build_screen_4eb240(true, 0, 1, 2, 3, 4, 5, None).unwrap();
        assert_eq!(v.slot1, 0xFFFF_FFFF);
        assert_eq!(v.slot2, 0xFFFF_FFFF);
        assert_eq!(v.param1_ptr, 0);
    }
    #[test]
    fn s4eb240_record_slots_pass_through() {
        let v = build_screen_4eb240(true, 0xdead, 9, 8, 100, 200, 7,
            Some((0x1234, 0x5678))).unwrap();
        assert_eq!(v.slot1, 0x1234);
        assert_eq!(v.slot2, 0x5678);
        assert_eq!(v.p2, 9);
        assert_eq!(v.p3, 8);
        assert_eq!(v.p4, 100);
        assert_eq!(v.p5, 200);
        assert_eq!(v.p6, 7);
    }

    // ---- 4ebf60 ----
    #[test]
    fn s4ebf60_carries_param() {
        assert_eq!(build_screen_4ebf60(true, 42).unwrap().p1, 42);
    }
    #[test]
    fn s4ebf60_failed_registration_returns_none() {
        assert!(build_screen_4ebf60(false, 42).is_none());
    }

    // ---- 4ec550 ----
    #[test]
    fn s4ec550_default_zeros() {
        let v = build_screen_4ec550(true).unwrap();
        assert_eq!(v.slot0, 0);
        assert_eq!(v.slot1, 0);
    }
    #[test]
    fn s4ec550_failed_registration_returns_none() {
        assert!(build_screen_4ec550(false).is_none());
    }

    // ---- 4fd1b0 ----
    #[test]
    fn s4fd1b0_default_zeros() {
        let v = build_screen_4fd1b0(true).unwrap();
        assert_eq!(v.slot0, 0);
        assert_eq!(v.slot1, 0);
    }
    #[test]
    fn s4fd1b0_failed_registration_returns_none() {
        assert!(build_screen_4fd1b0(false).is_none());
    }

    // ---- 00548170 ----
    #[test]
    fn s548170_failed_registration_returns_none() {
        let d = Screen548170Decode::default();
        let r = Screen548170Record::default();
        assert!(build_screen_548170(false, 0, d, r).is_none());
    }
    #[test]
    fn s548170_decoder_fields_map_to_correct_slots() {
        let d = Screen548170Decode {
            f_113: 1, f_10c: 2, f_115: 3, f_110: -4, f_slot8: 5, f_slot9: 6,
            f_11c: 7, f_11b: 8, f_11a: 9, f_119: 10, f_111: 11, f_112: 12,
            f_108: 13, f_114: 14, f_11d: 15,
        };
        let r = Screen548170Record { base_ptr: 0xB000, field1_at_33: 0x111,
            field2_at_33: 0x222, field3_at_33: 0x333 };
        let v = build_screen_548170(true, 99, d, r).unwrap();
        assert_eq!(v.slot0, 1);
        assert_eq!(v.record.base_ptr, 0xB000);
        assert_eq!(v.record.field1_at_33, 0x111);
        assert_eq!(v.record.field2_at_33, 0x222);
        assert_eq!(v.record.field3_at_33, 0x333);
        assert_eq!(v.slot5, 3);
        assert_eq!(v.slot6, -4);
        assert_eq!(v.slot8, 5);
        assert_eq!(v.slot9, 6);
        assert_eq!(v.slot_a, 7);
        assert_eq!(v.slot_b, 8);
        assert_eq!(v.slot_c, 9);
        assert_eq!(v.slot_d, 10);
        assert_eq!(v.slot_e, 11);
        assert_eq!(v.slot_f, 12);
        assert_eq!(v.slot_10, 13);
        assert_eq!(v.slot_11, 99); // param_2 passthrough
        assert_eq!(v.slot_12, 14);
        assert_eq!(v.slot_13, 15);
        assert_eq!(v.slot_14, 2);
    }
    #[test]
    fn s548170_param2_passthrough_independent_of_decode() {
        let v = build_screen_548170(true, 0xCAFE,
            Screen548170Decode::default(),
            Screen548170Record::default()).unwrap();
        assert_eq!(v.slot_11, 0xCAFE);
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b14: Screen4Eb240View reads handles {b14.4eb240.param1_ptr, p4, p5} + \
bytes {b14.4eb240.p2, p3, p6} + optional handles {b14.4eb240.slot1, slot2} \
(gated by flag b14.4eb240.record_slots_present); \
Screen4Ebf60View reads handle {b14.4ebf60.p1}; \
Screen4Ec550View + Screen4Fd1b0View are parameterless registration gates; \
Screen548170View reads handle {b14.548170.param_2} + full \
Screen548170Decode + Screen548170Record blocks — decoded from the \
DAT_00acd5c4 record pool. Wave-A: passed as Default::default().\n\
UNKNOWNS: wave-B typed pools not yet available; the 0x6e-stride record \
walk in FUN_00548170 requires the club-record typed accessor to lift.";

use crate::world_facade::WorldFacade;

/// Populator for [`Screen4Eb240View`].
pub fn populate_screen_4eb240(world: &WorldFacade) -> Option<Screen4Eb240View> {
    let record_slots = if world.flag("b14.4eb240.record_slots_present") {
        Some((world.handle("b14.4eb240.slot1"), world.handle("b14.4eb240.slot2")))
    } else {
        None
    };
    build_screen_4eb240(
        world.registration_ok,
        world.handle("b14.4eb240.param1_ptr"),
        world.byte("b14.4eb240.p2") as i8,
        world.byte("b14.4eb240.p3") as i8,
        world.handle("b14.4eb240.p4"),
        world.handle("b14.4eb240.p5"),
        world.byte("b14.4eb240.p6") as i8,
        record_slots,
    )
}

/// Populator for [`Screen4Ebf60View`].
pub fn populate_screen_4ebf60(world: &WorldFacade) -> Option<Screen4Ebf60View> {
    build_screen_4ebf60(world.registration_ok, world.handle("b14.4ebf60.p1"))
}

/// Populator for [`Screen4Ec550View`].
pub fn populate_screen_4ec550(world: &WorldFacade) -> Option<Screen4Ec550View> {
    build_screen_4ec550(world.registration_ok)
}

/// Populator for [`Screen4Fd1b0View`].
pub fn populate_screen_4fd1b0(world: &WorldFacade) -> Option<Screen4Fd1b0View> {
    build_screen_4fd1b0(world.registration_ok)
}

/// Populator for [`Screen548170View`]. Feeds Default decode/record until
/// the typed 0x6e-record accessor lands; only `param_2` is facade-sourced.
pub fn populate_screen_548170(world: &WorldFacade) -> Option<Screen548170View> {
    build_screen_548170(
        world.registration_ok,
        world.handle("b14.548170.param_2"),
        Screen548170Decode::default(),
        Screen548170Record::default(),
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_screen_4eb240_defaults_to_sentinels_when_no_record() {
        let v = populate_screen_4eb240(&WorldFacade::ready()).unwrap();
        assert_eq!(v.slot1, 0xFFFF_FFFF);
        assert_eq!(v.slot2, 0xFFFF_FFFF);
    }
    #[test]
    fn populate_screen_4eb240_uses_slots_when_present() {
        let w = WorldFacade::ready()
            .with_flag("b14.4eb240.record_slots_present", true)
            .with_handle("b14.4eb240.slot1", 0x11)
            .with_handle("b14.4eb240.slot2", 0x22);
        let v = populate_screen_4eb240(&w).unwrap();
        assert_eq!(v.slot1, 0x11);
        assert_eq!(v.slot2, 0x22);
    }
    #[test]
    fn populate_screen_4ebf60_ok() {
        assert!(populate_screen_4ebf60(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_screen_4ec550_ok() {
        assert!(populate_screen_4ec550(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_screen_4fd1b0_ok() {
        assert!(populate_screen_4fd1b0(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_screen_548170_ok() {
        let v = populate_screen_548170(&WorldFacade::ready()).unwrap();
        assert_eq!(v.slot_11, 0);
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A pattern)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b14 pool sources: Screen4Eb240View's record_slots comes from a \
    per-entity record lookup (opaque). Screen548170 needs the full \
    15-field decode from a serialised blob + a record at \
    DAT_00acd5c4 + local_f8 * 0x6e; neither is on the facade. All \
    populators fall back to zero-filled defaults.";

use crate::world_pools::WorldPools;

pub fn populate_screen_4eb240_from_pools(
    pools: &WorldPools<'_>,
) -> Option<Screen4Eb240View> {
    let seat = pools.active_human_seat.unwrap_or(0);
    build_screen_4eb240(true, seat, 0, 0, 0, 0, 0, None)
}

pub fn populate_screen_4ebf60_from_pools(
    pools: &WorldPools<'_>,
) -> Option<Screen4Ebf60View> {
    let seat = pools.active_human_seat.unwrap_or(0);
    build_screen_4ebf60(true, seat)
}

pub fn populate_screen_4ec550_from_pools(
    _pools: &WorldPools<'_>,
) -> Option<Screen4Ec550View> {
    build_screen_4ec550(true)
}

pub fn populate_screen_4fd1b0_from_pools(
    _pools: &WorldPools<'_>,
) -> Option<Screen4Fd1b0View> {
    build_screen_4fd1b0(true)
}

pub fn populate_screen_548170_from_pools(
    pools: &WorldPools<'_>,
) -> Option<Screen548170View> {
    let seat = pools.active_human_seat.unwrap_or(0);
    build_screen_548170(true, seat,
        Screen548170Decode::default(),
        Screen548170Record::default())
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn all_return_some_on_empty_pools() {
        let p = WorldPools::empty();
        assert!(populate_screen_4eb240_from_pools(&p).is_some());
        assert!(populate_screen_4ebf60_from_pools(&p).is_some());
        assert!(populate_screen_4ec550_from_pools(&p).is_some());
        assert!(populate_screen_4fd1b0_from_pools(&p).is_some());
        assert!(populate_screen_548170_from_pools(&p).is_some());
    }

    #[test]
    fn seat_carries_into_slot0() {
        let p = WorldPools { active_human_seat: Some(9), ..WorldPools::empty() };
        assert_eq!(populate_screen_4ebf60_from_pools(&p).unwrap().p1, 9);
    }
}
