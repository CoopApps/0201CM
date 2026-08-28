//! Batch 25: 5 more setup functions from the 0x008e**** cluster.
//!
//! All follow the standard shape: `FUN_007e6570(...)` registers the
//! screen (0 = failure → early return, we return `None`), then a run
//! of `FUN_007e7130(slot, value, 0)` writes to the slot table.
//!
//! Decompiles: `D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `008e10c0.c` — dispatcher: picks the "detail" (6-slot) or
//!   "list" (1-slot) variant based on a data-record peek.
//! * `008e26a0.c` — simple 2-slot screen: `(*param_1, param_2)`.
//! * `008e26f0.c` — simple 2-slot screen: `(*param_1, param_3==1)`.
//! * `008e3320.c` — trivial 1-slot screen: `(param_1)`.
//! * `008e50e0.c` — 16-slot screen with a state-lookup cascade.

use serde::{Deserialize, Serialize};

// =====================================================================
// 008e10c0 — Dispatcher: chooses between two screen shapes.
// =====================================================================

/// Which variant the exe's `008e10c0` dispatcher selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen008e10c0View {
    /// Taken when the lookup's inner ptr is null OR (inner+0x24 == p2)
    /// AND *(inner+0x2c) != 3 — 6 slots + trailing zero at slot 6.
    Detail {
        /// Slot 0: raw pointer (modeled as an id).
        record_ptr: u32,
        /// Slot 1.
        param_1: u32,
        /// Slot 2.
        param_2: u32,
        /// Slot 3: literal 0.
        reserved3: u32,
        /// Slot 4: *(inner+0x24) — a linked-entity id.
        linked_entity: u32,
        /// Slot 5: *(inner+0x0c).
        secondary: u32,
        /// Slot 6: trailing 0 (exe writes `FUN_007e7130(6, 0, 0)`).
        trailing6: u32,
    },
    /// Otherwise — 1 slot (inner-record ptr) + trailing zero at slot 1.
    List {
        inner_record: u32,
        trailing1: u32,
    },
}

/// Inputs to `FUN_008e10c0` — the exe reads `piVar3 = FUN_008b3240(uVar1)`
/// (a lookup keyed on `FUN_0076d7d0(6)`) and `iVar2 = FUN_0076d7d0(7)`.
/// We collapse the whole record chain to the three concrete fields the
/// branch uses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e10c0Lookup {
    /// The pointer returned by the lookup — 0 means the "detail"
    /// branch takes the null path.
    pub inner_record_ptr: u32,
    /// `*(inner + 0x24)` — compared against `iVar2` (id 7).
    pub inner_plus_24: u32,
    /// `*(inner + 0x0c)`.
    pub inner_plus_0c: u32,
    /// `*(inner + 0x2c)` — must not be `3` to take the detail branch.
    pub inner_plus_2c: u8,
    /// `FUN_0076d7d0(7)`.
    pub context_id_7: u32,
}

/// Direct port of `FUN_008e10c0(param_1, param_2)`.
pub fn build_screen_008e10c0(
    registration_ok: bool,
    param_1: u32,
    param_2: u32,
    lookup: &Screen008e10c0Lookup,
) -> Option<Screen008e10c0View> {
    if !registration_ok { return None; }
    let inner = lookup.inner_record_ptr;
    let take_detail = (inner == 0 || lookup.inner_plus_24 == lookup.context_id_7)
        && lookup.inner_plus_2c != 3;
    if take_detail {
        Some(Screen008e10c0View::Detail {
            record_ptr: inner,
            param_1,
            param_2,
            reserved3: 0,
            linked_entity: lookup.inner_plus_24,
            secondary: lookup.inner_plus_0c,
            trailing6: 0,
        })
    } else {
        Some(Screen008e10c0View::List { inner_record: inner, trailing1: 0 })
    }
}

// =====================================================================
// 008e26a0 — 2-slot screen: (*param_1, param_2).
// =====================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e26a0View {
    /// Slot 0: `*param_1`.
    pub primary: u32,
    /// Slot 1: `param_2`.
    pub secondary: u32,
}

/// Direct port of `FUN_008e26a0(param_1, param_2)`.
pub fn build_screen_008e26a0(
    registration_ok: bool,
    primary: u32,
    secondary: u32,
) -> Option<Screen008e26a0View> {
    if !registration_ok { return None; }
    Some(Screen008e26a0View { primary, secondary })
}

// =====================================================================
// 008e26f0 — 2-slot screen: (*param_1, param_3 == 1).
// =====================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e26f0View {
    /// Slot 0: `*param_1`.
    pub primary: u32,
    /// Slot 1: `param_3 == 1` as a bool.
    pub flag: bool,
}

/// Direct port of `FUN_008e26f0(param_1, param_2, param_3)`. Note the exe
/// ignores `param_2` entirely.
pub fn build_screen_008e26f0(
    registration_ok: bool,
    primary: u32,
    param_3: i8,
) -> Option<Screen008e26f0View> {
    if !registration_ok { return None; }
    Some(Screen008e26f0View { primary, flag: param_3 == 1 })
}

// =====================================================================
// 008e3320 — 1-slot screen: (param_1).
// =====================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e3320View {
    /// Slot 0: `param_1`.
    pub value: u32,
}

/// Direct port of `FUN_008e3320(param_1)`.
pub fn build_screen_008e3320(
    registration_ok: bool,
    value: u32,
) -> Option<Screen008e3320View> {
    if !registration_ok { return None; }
    Some(Screen008e3320View { value })
}

// =====================================================================
// 008e50e0 — 16-slot screen; state-lookup cascade drives 4/5/6/A/D/E.
// =====================================================================

/// State fed into `FUN_008e50e0` — models the two lookup chains the exe
/// does through the active-human seat (`DAT_00b59fc2[DAT_00b5d016*0xc0]`)
/// and the sub-record found by `FUN_004d5a20`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e50e0State {
    /// Slot 1: `*(param_1 + 0x52)`.
    pub param1_plus_52: u32,
    /// Slot B: `*(active_seat + 0x39)` — id of the current entity.
    pub active_entity: u32,
    /// Slot C: derived from `FUN_0058a490(active_entity)[0x165] == 1`.
    pub active_flag_set: bool,
    /// Return value of `FUN_004d5a20(param_1, active_entity)` — 0 means
    /// the "no sub-record" branch runs.
    pub sub_record_ptr: u32,
    /// `*(sub + 0x45)` — used by slot E as `iVar1 != 0`.
    pub sub_plus_45: u32,
    /// `*(sub + 0x4e)` — bit 3 (`>>3 & 1`) drives slots 6 and D, bit 2
    /// (`>>2 & 1`) drives slot A.
    pub sub_plus_4e: u8,
    /// `*(sub + 0x4f) >> 4` — drives slot 3.
    pub sub_plus_4f_hi: u8,
    /// `FUN_0052dff0(param_1, 0)` — slot 4.
    pub position_4: u32,
    /// `FUN_0052e0e0(param_1, 0)` — slot 5.
    pub position_5: u32,
    /// `*(param_1 + 0x61)` — 0 → slot F = 0; else slot F =
    /// `FUN_004d7090(...)`.
    pub param1_plus_61: u32,
    /// `FUN_004d7090(param_1, active_entity, 0, -1, 1)` — slot F when
    /// `param1_plus_61 != 0`.
    pub slot_f_value: u32,
}

/// The 16 slot writes `008e50e0` performs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e50e0View {
    pub slot0: u32,   // *param_1
    pub slot1: u32,   // *(param_1 + 0x52)
    pub slot2: u32,   // 0
    pub slot3: u32,   // sub? >>4 of *(sub+0x4f) : 7
    pub slot4: u32,   // sub? FUN_0052dff0 : 0
    pub slot5: u32,   // sub? FUN_0052e0e0 : 0
    pub slot6: u32,   // sub? bit3 of *(sub+0x4e) : 0
    pub slot7: u32,   // param_2
    pub slot8: u32,   // param_3
    pub slot9: u32,   // param_4
    pub slot_a: u32,  // sub? bit2 of *(sub+0x4e) : 0
    pub slot_b: u32,  // *(active_seat + 0x39)
    pub slot_c: u32,  // active-flag test
    pub slot_d: u32,  // sub? bit3 : 0
    pub slot_e: u32,  // sub? sub_plus_45 != 0 : 0
    pub slot_f: u32,  // *(param_1+0x61) ? FUN_004d7090 : 0
}

/// Direct port of `FUN_008e50e0(param_1, param_2, param_3, param_4)`.
/// `*param_1` is passed as `primary`.
pub fn build_screen_008e50e0(
    registration_ok: bool,
    primary: u32,
    param_2: u32,
    param_3: u32,
    param_4: u32,
    state: &Screen008e50e0State,
) -> Option<Screen008e50e0View> {
    if !registration_ok { return None; }
    let bit3 = ((state.sub_plus_4e & 0x08) >> 3) as u32;
    let bit2 = ((state.sub_plus_4e & 0x04) >> 2) as u32;
    let has_sub = state.sub_record_ptr != 0;
    let (slot3, slot4, slot5, slot6, slot_a, slot_d, slot_e) = if has_sub {
        (
            state.sub_plus_4f_hi as u32,
            state.position_4,
            state.position_5,
            bit3,
            bit2,
            bit3,
            (state.sub_plus_45 != 0) as u32,
        )
    } else {
        (7, 0, 0, 0, 0, 0, 0)
    };
    let slot_f = if state.param1_plus_61 == 0 { 0 } else { state.slot_f_value };
    Some(Screen008e50e0View {
        slot0: primary,
        slot1: state.param1_plus_52,
        slot2: 0,
        slot3,
        slot4,
        slot5,
        slot6,
        slot7: param_2,
        slot8: param_3,
        slot9: param_4,
        slot_a,
        slot_b: state.active_entity,
        slot_c: state.active_flag_set as u32,
        slot_d,
        slot_e,
        slot_f,
    })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s008e10c0_failed_registration_returns_none() {
        assert!(build_screen_008e10c0(false, 1, 2, &Screen008e10c0Lookup::default()).is_none());
    }
    #[test]
    fn s008e10c0_null_inner_takes_detail_branch() {
        let lk = Screen008e10c0Lookup { inner_record_ptr: 0, ..Default::default() };
        let v = build_screen_008e10c0(true, 11, 22, &lk).unwrap();
        match v {
            Screen008e10c0View::Detail { param_1, param_2, .. } => {
                assert_eq!(param_1, 11); assert_eq!(param_2, 22);
            }
            _ => panic!("expected Detail"),
        }
    }
    #[test]
    fn s008e10c0_matching_ids_and_non_three_takes_detail() {
        let lk = Screen008e10c0Lookup {
            inner_record_ptr: 0xdead, inner_plus_24: 7, inner_plus_0c: 99,
            inner_plus_2c: 1, context_id_7: 7,
        };
        let v = build_screen_008e10c0(true, 1, 2, &lk).unwrap();
        assert!(matches!(v, Screen008e10c0View::Detail { linked_entity: 7, secondary: 99, .. }));
    }
    #[test]
    fn s008e10c0_type_three_forces_list_branch() {
        let lk = Screen008e10c0Lookup {
            inner_record_ptr: 0xbeef, inner_plus_24: 7, inner_plus_0c: 5,
            inner_plus_2c: 3, context_id_7: 7,
        };
        assert!(matches!(build_screen_008e10c0(true, 1, 2, &lk).unwrap(),
                         Screen008e10c0View::List { inner_record: 0xbeef, .. }));
    }
    #[test]
    fn s008e10c0_mismatched_ids_takes_list_branch() {
        let lk = Screen008e10c0Lookup {
            inner_record_ptr: 0xf00d, inner_plus_24: 1, inner_plus_0c: 0,
            inner_plus_2c: 0, context_id_7: 2,
        };
        assert!(matches!(build_screen_008e10c0(true, 0, 0, &lk).unwrap(),
                         Screen008e10c0View::List { .. }));
    }

    #[test]
    fn s008e26a0_carries_both_params() {
        let v = build_screen_008e26a0(true, 42, 7).unwrap();
        assert_eq!(v.primary, 42); assert_eq!(v.secondary, 7);
    }
    #[test]
    fn s008e26a0_failed_registration_returns_none() {
        assert!(build_screen_008e26a0(false, 0, 0).is_none());
    }

    #[test]
    fn s008e26f0_flag_true_when_param3_is_one() {
        assert!(build_screen_008e26f0(true, 99, 1).unwrap().flag);
    }
    #[test]
    fn s008e26f0_flag_false_for_other_values() {
        assert!(!build_screen_008e26f0(true, 99, 0).unwrap().flag);
        assert!(!build_screen_008e26f0(true, 99, 2).unwrap().flag);
    }
    #[test]
    fn s008e26f0_failed_registration_returns_none() {
        assert!(build_screen_008e26f0(false, 0, 1).is_none());
    }

    #[test]
    fn s008e3320_carries_value() {
        assert_eq!(build_screen_008e3320(true, 0xabc).unwrap().value, 0xabc);
    }
    #[test]
    fn s008e3320_failed_registration_returns_none() {
        assert!(build_screen_008e3320(false, 1).is_none());
    }

    #[test]
    fn s008e50e0_no_sub_record_uses_fallback_defaults() {
        let st = Screen008e50e0State {
            param1_plus_52: 10, active_entity: 5, active_flag_set: true,
            sub_record_ptr: 0, param1_plus_61: 0,
            ..Default::default()
        };
        let v = build_screen_008e50e0(true, 1, 2, 3, 4, &st).unwrap();
        assert_eq!(v.slot0, 1);
        assert_eq!(v.slot1, 10);
        assert_eq!(v.slot2, 0);
        assert_eq!(v.slot3, 7);   // fallback
        assert_eq!(v.slot4, 0);
        assert_eq!(v.slot5, 0);
        assert_eq!(v.slot6, 0);
        assert_eq!(v.slot7, 2);
        assert_eq!(v.slot8, 3);
        assert_eq!(v.slot9, 4);
        assert_eq!(v.slot_a, 0);
        assert_eq!(v.slot_b, 5);
        assert_eq!(v.slot_c, 1);
        assert_eq!(v.slot_d, 0);
        assert_eq!(v.slot_e, 0);
        assert_eq!(v.slot_f, 0);
    }
    #[test]
    fn s008e50e0_with_sub_record_extracts_bits() {
        let st = Screen008e50e0State {
            param1_plus_52: 0, active_entity: 0, active_flag_set: false,
            sub_record_ptr: 0x1000, sub_plus_45: 9,
            sub_plus_4e: 0b0000_1100, // bit3=1, bit2=1
            sub_plus_4f_hi: 0x3,
            position_4: 100, position_5: 200,
            param1_plus_61: 1, slot_f_value: 0x777,
        };
        let v = build_screen_008e50e0(true, 0, 0, 0, 0, &st).unwrap();
        assert_eq!(v.slot3, 3);
        assert_eq!(v.slot4, 100);
        assert_eq!(v.slot5, 200);
        assert_eq!(v.slot6, 1);
        assert_eq!(v.slot_a, 1);
        assert_eq!(v.slot_d, 1);
        assert_eq!(v.slot_e, 1);
        assert_eq!(v.slot_f, 0x777);
    }
    #[test]
    fn s008e50e0_failed_registration_returns_none() {
        assert!(build_screen_008e50e0(false, 0, 0, 0, 0, &Screen008e50e0State::default())
            .is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b25: Screen008e10c0View reads {param_1, param_2} + a Screen008e10c0Lookup \
sourced from handles/bytes (inner_record_ptr, inner_plus_24, inner_plus_0c, \
inner_plus_2c, context_id_7); Screen008e26a0View reads {primary, secondary}; \
Screen008e26f0View reads {primary} + byte `param_3`; Screen008e3320View \
reads {value}; Screen008e50e0View reads {primary, param_2..param_4} + a \
Screen008e50e0State pulled from a dozen well-known slot names.\n\
UNKNOWNS: FUN_008b3240 record lookup, FUN_0058a490 active-flag lookup, \
FUN_004d5a20 sub-record lookup, FUN_004d7090 slot-F derivation — all \
expected as pre-derived handles from the wave-B driver.";

use crate::world_facade::WorldFacade;

fn lookup_from(world: &WorldFacade) -> Screen008e10c0Lookup {
    Screen008e10c0Lookup {
        inner_record_ptr: world.handle("inner_record_ptr"),
        inner_plus_24: world.handle("inner_plus_24"),
        inner_plus_0c: world.handle("inner_plus_0c"),
        inner_plus_2c: world.byte("inner_plus_2c") as u8,
        context_id_7: world.handle("context_id_7"),
    }
}

fn state_50e0_from(world: &WorldFacade) -> Screen008e50e0State {
    Screen008e50e0State {
        param1_plus_52: world.handle("param1_plus_52"),
        active_entity: world.handle("active_entity"),
        active_flag_set: world.flag("active_flag_set"),
        sub_record_ptr: world.handle("sub_record_ptr"),
        sub_plus_45: world.handle("sub_plus_45"),
        sub_plus_4e: world.byte("sub_plus_4e") as u8,
        sub_plus_4f_hi: world.byte("sub_plus_4f_hi") as u8,
        position_4: world.handle("position_4"),
        position_5: world.handle("position_5"),
        param1_plus_61: world.handle("param1_plus_61"),
        slot_f_value: world.handle("slot_f_value"),
    }
}

pub fn populate_screen_008e10c0(world: &WorldFacade) -> Option<Screen008e10c0View> {
    build_screen_008e10c0(
        world.registration_ok,
        world.handle("param_1"), world.handle("param_2"),
        &lookup_from(world),
    )
}

pub fn populate_screen_008e26a0(world: &WorldFacade) -> Option<Screen008e26a0View> {
    build_screen_008e26a0(
        world.registration_ok, world.handle("primary"), world.handle("secondary"),
    )
}

pub fn populate_screen_008e26f0(world: &WorldFacade) -> Option<Screen008e26f0View> {
    build_screen_008e26f0(
        world.registration_ok,
        world.handle("primary"),
        world.byte("param_3") as i8,
    )
}

pub fn populate_screen_008e3320(world: &WorldFacade) -> Option<Screen008e3320View> {
    build_screen_008e3320(world.registration_ok, world.handle("value"))
}

pub fn populate_screen_008e50e0(world: &WorldFacade) -> Option<Screen008e50e0View> {
    build_screen_008e50e0(
        world.registration_ok,
        world.handle("primary"),
        world.handle("param_2"), world.handle("param_3"), world.handle("param_4"),
        &state_50e0_from(world),
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_screen_008e10c0_null_inner_takes_detail() {
        let v = populate_screen_008e10c0(&WorldFacade::ready()).unwrap();
        assert!(matches!(v, Screen008e10c0View::Detail { .. }));
    }
    #[test]
    fn populate_screen_008e26a0_ok() {
        let w = WorldFacade::ready().with_handle("primary", 42);
        assert_eq!(populate_screen_008e26a0(&w).unwrap().primary, 42);
    }
    #[test]
    fn populate_screen_008e26f0_flag_from_byte() {
        let w = WorldFacade::ready().with_byte("param_3", 1);
        assert!(populate_screen_008e26f0(&w).unwrap().flag);
    }
    #[test]
    fn populate_screen_008e3320_ok() {
        let w = WorldFacade::ready().with_handle("value", 7);
        assert_eq!(populate_screen_008e3320(&w).unwrap().value, 7);
    }
    #[test]
    fn populate_screen_008e50e0_no_sub_uses_fallback_slot3() {
        let v = populate_screen_008e50e0(&WorldFacade::ready()).unwrap();
        assert_eq!(v.slot3, 7);
    }
}
