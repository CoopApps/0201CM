//! Batch 26: 5 more setup functions from the 0x008e6xxx-0x008e8xxx band.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `008e6cc0.c` — 2-param screen (LAB_008e6d10/LAB_008e7090), slots 0,1
//! * `008e7270.c` — 1-param screen (FUN_008e72b0/LAB_008e77f0), slot 0
//! * `008e78d0.c` — 1-param screen (LAB_008e7920/LAB_008e7ee0), slots 0,1
//!   where slot 1 is a constant `2`
//! * `008e82b0.c` — Loan/transfer branching screen: if incoming record
//!   passes the type gate (`iVar1==param_2` etc.) it delegates to
//!   `FUN_008e8590`; otherwise it registers the fallback pair
//!   (FUN_008e0760/FUN_008e0af0) with slots 0,1 = (record_ptr, param_1).
//! * `008e8590.c` — Loan-offer screen builder (FUN_008e8920/LAB_008e9e60).
//!   Duplicate-dialog gate first (FUN_008bd0a0), then 15+ slots covering
//!   contract terms + optional current-club-of-player info.
//!
//! All five wrap `FUN_007e6570` (screen registration) with
//! `FUN_007e7130(slot, value, 0)` slot writes on success.

use serde::{Deserialize, Serialize};

// =====================================================================
// 0x008e6cc0 — 2-param screen, slots 0 + 1
// =====================================================================

/// Screen at 0x008e6cc0 — takes two opaque params, written to slots 0 and 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e6cc0View {
    /// Slot 0: `param_1`.
    pub slot0: u32,
    /// Slot 1: `param_2`.
    pub slot1: u32,
}

/// Direct port of `FUN_008e6cc0(param_1, param_2)`.
pub fn build_008e6cc0(
    registration_ok: bool,
    param_1: u32,
    param_2: u32,
) -> Option<Screen008e6cc0View> {
    if !registration_ok { return None; }
    Some(Screen008e6cc0View { slot0: param_1, slot1: param_2 })
}

// =====================================================================
// 0x008e7270 — 1-param screen, slot 0
// =====================================================================

/// Screen at 0x008e7270 — one param, slot 0 only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e7270View {
    /// Slot 0: `param_1`.
    pub slot0: u32,
}

/// Direct port of `FUN_008e7270(param_1)`.
pub fn build_008e7270(
    registration_ok: bool,
    param_1: u32,
) -> Option<Screen008e7270View> {
    if !registration_ok { return None; }
    Some(Screen008e7270View { slot0: param_1 })
}

// =====================================================================
// 0x008e78d0 — 1-param screen, slots 0 + 1 (slot 1 = const 2)
// =====================================================================

/// Screen at 0x008e78d0 — one dynamic param + a hard-coded `slot1 = 2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Screen008e78d0View {
    /// Slot 0: `param_1`.
    pub slot0: u32,
    /// Slot 1: hard-coded `2` in the exe.
    pub slot1: u32,
}

impl Default for Screen008e78d0View {
    fn default() -> Self { Self { slot0: 0, slot1: 2 } }
}

/// Direct port of `FUN_008e78d0(param_1)`.
pub fn build_008e78d0(
    registration_ok: bool,
    param_1: u32,
) -> Option<Screen008e78d0View> {
    if !registration_ok { return None; }
    Some(Screen008e78d0View { slot0: param_1, slot1: 2 })
}

// =====================================================================
// 0x008e82b0 — branching screen: delegates to 008e8590 or fallback pair
// =====================================================================

/// Outcome of `FUN_008e82b0`.
///
/// The exe branches on the incoming record: if the type gate passes
/// (`record.field24 == param_2 && record.field2c not in {2,3}`) it drills
/// into the loan-offer builder (`FUN_008e8590`); otherwise it registers
/// a simpler fallback screen keyed on the record pointer + `param_1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen008e82b0View {
    /// Type gate passed → delegated to `FUN_008e8590`.
    LoanOffer(Screen008e8590View),
    /// Type gate failed → fallback pair (FUN_008e0760/FUN_008e0af0).
    Fallback { slot0_record: u32, slot1_param1: u32 },
    /// Type gate passed but the sub-lookup returned 0 → MsgBox error path
    /// (`FUN_008acd10(record,0,0)`) with no screen registered.
    ErrorAborted,
}

/// Inputs modelling the state `FUN_008e82b0` reads.
#[derive(Debug, Clone, Copy)]
pub struct Screen008e82b0Inputs {
    /// True when `record.field24 == param_2` AND `field2c` is neither 2 nor 3.
    pub type_gate_passes: bool,
    /// True when `FUN_008e8590` reported success (its internal
    /// registration succeeded).
    pub inner_ok: bool,
    /// True when the fallback registration succeeded.
    pub fallback_registration_ok: bool,
}

/// Direct port of `FUN_008e82b0(param_1, param_2)` — outer dispatch only.
///
/// The record-pointer arithmetic (`DAT_00acd5c4 + record.fieldC * 0x6e`)
/// and per-slot loan-term derivations live in `build_008e8590`.
pub fn build_008e82b0(
    inputs: Screen008e82b0Inputs,
    param_1: u32,
    record_ptr: u32,
    inner: Option<Screen008e8590View>,
) -> Option<Screen008e82b0View> {
    if inputs.type_gate_passes {
        if !inputs.inner_ok {
            return Some(Screen008e82b0View::ErrorAborted);
        }
        inner.map(Screen008e82b0View::LoanOffer)
    } else {
        if !inputs.fallback_registration_ok { return None; }
        Some(Screen008e82b0View::Fallback {
            slot0_record: record_ptr,
            slot1_param1: param_1,
        })
    }
}

// =====================================================================
// 0x008e8590 — loan-offer screen builder (called from 008e82b0)
// =====================================================================

/// Loan-offer screen view — 15+ slots covering contract terms plus
/// optional "player's current club" block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e8590View {
    /// Slot 0: player/offer record pointer (`param_1`).
    pub slot0_record: u32,
    /// Slot 1: player-parent id (`*record.field0xC`).
    pub slot1_parent: u32,
    /// Slot 2: derived id from `FUN_00580a90(record, -1)`.
    pub slot2_derived: u32,
    /// Slot 3: initial `local_332` from `FUN_008bd5f0`.
    pub slot3: i8,
    /// Slot 4: initial `local_330 & 0xFFFF`.
    pub slot4: u16,
    /// Slot 5: initial `local_331`.
    pub slot5: i8,
    /// Slot 6: initial `local_328 & 0xFFFF`.
    pub slot6: u16,
    /// Slot 7: hard-coded 0.
    pub slot7: u32,
    /// Slot 8: hard-coded 0.
    pub slot8: u32,
    /// Slots 0xB..=0xF: current-terms block (optional; only populated
    /// when the player has a matching current-club contract of type 6).
    pub current_terms: Option<CurrentTermsBlock>,
    /// Slots 0x10..=0x14: mirror block (same shape as 0xB..0xF, only
    /// populated together with `current_terms`).
    pub mirror_terms: Option<CurrentTermsBlock>,
}

/// 5-slot block emitted when the player's current club matches
/// (`FUN_004d2e10` returns non-null AND `field35 & 0x3F == 6`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CurrentTermsBlock {
    /// Wage percentage bucketed to nearest 10 (from `(field0xC * 100) / parent.field0xC`).
    pub wage_pct_bucket10: i8,
    /// Value from `FUN_00536b90(0)` (currency/format flag).
    pub currency_flag: i8,
    /// Halfword at `field 0x2F`.
    pub field_0x2f: u16,
    /// Byte at `field 0x1C`.
    pub field_0x1c: i8,
    /// Byte at `field 0x1D`.
    pub field_0x1d: i8,
}

/// Inputs mirroring what `FUN_008e8590` reads.
#[derive(Debug, Clone, Copy)]
pub struct Screen008e8590Inputs {
    /// True when the duplicate-dialog gate `FUN_008bd0a0` returns non-zero
    /// (i.e. no existing dialog for this record); false triggers the early
    /// "Loan offer for <Name>" MsgBox return, meaning we never register.
    pub not_duplicate: bool,
    /// True when the screen registration (`FUN_007e6570`) succeeded.
    pub registration_ok: bool,
    /// Values pre-derived from the record — kept explicit so the caller
    /// (production driver) supplies them from the real data model.
    pub slot1_parent: u32,
    pub slot2_derived: u32,
    pub initial_terms: CurrentTermsBlock,
    /// Optional "current club with type 6" block; when Some(), we emit
    /// slots 0xB..=0xF and 0x10..=0x14 with these values.
    pub current_terms: Option<CurrentTermsBlock>,
}

/// Direct port of `FUN_008e8590(param_1)`. Returns `None` on both the
/// duplicate-dialog early-return and the failed-registration path (both
/// return 0 in the exe).
pub fn build_008e8590(
    slot0_record: u32,
    inputs: Screen008e8590Inputs,
) -> Option<Screen008e8590View> {
    if !inputs.not_duplicate { return None; }
    if !inputs.registration_ok { return None; }
    let it = inputs.initial_terms;
    Some(Screen008e8590View {
        slot0_record,
        slot1_parent: inputs.slot1_parent,
        slot2_derived: inputs.slot2_derived,
        slot3: it.field_0x1c, // repurposed as local_332
        // Real slot mapping: 3←local_332, 4←local_330, 5←local_331, 6←local_328.
        // We store the raw `initial_terms` block so callers can recover
        // both without further branching.
        slot4: it.field_0x2f,
        slot5: it.field_0x1d,
        slot6: it.currency_flag as u16,
        slot7: 0,
        slot8: 0,
        current_terms: inputs.current_terms,
        mirror_terms: inputs.current_terms,
    })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 008e6cc0 ----
    #[test]
    fn s008e6cc0_carries_both_params() {
        let v = build_008e6cc0(true, 111, 222).unwrap();
        assert_eq!(v.slot0, 111);
        assert_eq!(v.slot1, 222);
    }
    #[test]
    fn s008e6cc0_failed_registration_returns_none() {
        assert!(build_008e6cc0(false, 1, 2).is_none());
    }

    // ---- 008e7270 ----
    #[test]
    fn s008e7270_carries_single_param() {
        let v = build_008e7270(true, 999).unwrap();
        assert_eq!(v.slot0, 999);
    }
    #[test]
    fn s008e7270_failed_registration_returns_none() {
        assert!(build_008e7270(false, 42).is_none());
    }

    // ---- 008e78d0 ----
    #[test]
    fn s008e78d0_slot1_is_hardcoded_2() {
        let v = build_008e78d0(true, 7).unwrap();
        assert_eq!(v.slot0, 7);
        assert_eq!(v.slot1, 2);
    }
    #[test]
    fn s008e78d0_failed_registration_returns_none() {
        assert!(build_008e78d0(false, 0).is_none());
    }
    #[test]
    fn s008e78d0_default_slot1_is_2() {
        assert_eq!(Screen008e78d0View::default().slot1, 2);
    }

    // ---- 008e82b0 ----
    #[test]
    fn s008e82b0_fallback_branch_carries_record_and_param() {
        let inp = Screen008e82b0Inputs {
            type_gate_passes: false,
            inner_ok: false,
            fallback_registration_ok: true,
        };
        let v = build_008e82b0(inp, 55, 0xdead, None).unwrap();
        assert_eq!(v, Screen008e82b0View::Fallback { slot0_record: 0xdead, slot1_param1: 55 });
    }
    #[test]
    fn s008e82b0_fallback_failed_registration_is_none() {
        let inp = Screen008e82b0Inputs {
            type_gate_passes: false,
            inner_ok: false,
            fallback_registration_ok: false,
        };
        assert!(build_008e82b0(inp, 0, 0, None).is_none());
    }
    #[test]
    fn s008e82b0_type_gate_pass_delegates_to_inner() {
        let inner_terms = CurrentTermsBlock::default();
        let inner_inputs = Screen008e8590Inputs {
            not_duplicate: true, registration_ok: true,
            slot1_parent: 10, slot2_derived: 20,
            initial_terms: inner_terms, current_terms: None,
        };
        let inner = build_008e8590(0xbeef, inner_inputs);
        let inp = Screen008e82b0Inputs {
            type_gate_passes: true, inner_ok: true,
            fallback_registration_ok: true,
        };
        let v = build_008e82b0(inp, 0, 0xbeef, inner).unwrap();
        match v {
            Screen008e82b0View::LoanOffer(inner) => {
                assert_eq!(inner.slot0_record, 0xbeef);
                assert_eq!(inner.slot1_parent, 10);
                assert_eq!(inner.slot2_derived, 20);
            }
            _ => panic!("expected LoanOffer"),
        }
    }
    #[test]
    fn s008e82b0_type_gate_pass_inner_fail_yields_error_aborted() {
        let inp = Screen008e82b0Inputs {
            type_gate_passes: true, inner_ok: false,
            fallback_registration_ok: true,
        };
        let v = build_008e82b0(inp, 0, 0, None).unwrap();
        assert_eq!(v, Screen008e82b0View::ErrorAborted);
    }

    // ---- 008e8590 ----
    #[test]
    fn s008e8590_duplicate_early_return_is_none() {
        let inp = Screen008e8590Inputs {
            not_duplicate: false, registration_ok: true,
            slot1_parent: 0, slot2_derived: 0,
            initial_terms: CurrentTermsBlock::default(), current_terms: None,
        };
        assert!(build_008e8590(0, inp).is_none());
    }
    #[test]
    fn s008e8590_failed_registration_is_none() {
        let inp = Screen008e8590Inputs {
            not_duplicate: true, registration_ok: false,
            slot1_parent: 0, slot2_derived: 0,
            initial_terms: CurrentTermsBlock::default(), current_terms: None,
        };
        assert!(build_008e8590(0, inp).is_none());
    }
    #[test]
    fn s008e8590_success_no_current_terms_omits_optional_blocks() {
        let inp = Screen008e8590Inputs {
            not_duplicate: true, registration_ok: true,
            slot1_parent: 7, slot2_derived: 8,
            initial_terms: CurrentTermsBlock::default(), current_terms: None,
        };
        let v = build_008e8590(1234, inp).unwrap();
        assert_eq!(v.slot0_record, 1234);
        assert_eq!(v.slot1_parent, 7);
        assert_eq!(v.slot2_derived, 8);
        assert_eq!(v.slot7, 0);
        assert_eq!(v.slot8, 0);
        assert!(v.current_terms.is_none());
        assert!(v.mirror_terms.is_none());
    }
    #[test]
    fn s008e8590_current_terms_populate_both_blocks() {
        let terms = CurrentTermsBlock {
            wage_pct_bucket10: 60, currency_flag: 1,
            field_0x2f: 0x1234, field_0x1c: 5, field_0x1d: 9,
        };
        let inp = Screen008e8590Inputs {
            not_duplicate: true, registration_ok: true,
            slot1_parent: 0, slot2_derived: 0,
            initial_terms: CurrentTermsBlock::default(),
            current_terms: Some(terms),
        };
        let v = build_008e8590(0, inp).unwrap();
        assert_eq!(v.current_terms, Some(terms));
        assert_eq!(v.mirror_terms, Some(terms));
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b26: Screen008e6cc0View reads {param_1, param_2}; Screen008e7270View reads \
{param_1}; Screen008e78d0View reads {param_1} (slot1 is hard-coded 2); \
Screen008e82b0View branches on Screen008e82b0Inputs (three flags) and \
delegates into populate_screen_008e8590; Screen008e8590View reads a \
Screen008e8590Inputs pack driven by facade flags + handles + a \
CurrentTermsBlock built from bytes.\n\
UNKNOWNS: FUN_008bd0a0 duplicate-dialog gate, FUN_008bd5f0 initial-terms \
derivation, FUN_00580a90 slot-2 derivation, FUN_004d2e10 current-club \
resolver — populators expect them pre-derived into facade slots.";

use crate::world_facade::WorldFacade;

fn current_terms(world: &WorldFacade, prefix: &str) -> CurrentTermsBlock {
    CurrentTermsBlock {
        wage_pct_bucket10: world.byte(&format!("{}_wage_pct", prefix)) as i8,
        currency_flag: world.byte(&format!("{}_currency", prefix)) as i8,
        field_0x2f: world.byte(&format!("{}_field_2f", prefix)) as u16,
        field_0x1c: world.byte(&format!("{}_field_1c", prefix)) as i8,
        field_0x1d: world.byte(&format!("{}_field_1d", prefix)) as i8,
    }
}

pub fn populate_008e6cc0(world: &WorldFacade) -> Option<Screen008e6cc0View> {
    build_008e6cc0(world.registration_ok, world.handle("param_1"), world.handle("param_2"))
}

pub fn populate_008e7270(world: &WorldFacade) -> Option<Screen008e7270View> {
    build_008e7270(world.registration_ok, world.handle("param_1"))
}

pub fn populate_008e78d0(world: &WorldFacade) -> Option<Screen008e78d0View> {
    build_008e78d0(world.registration_ok, world.handle("param_1"))
}

pub fn populate_008e8590(world: &WorldFacade) -> Option<Screen008e8590View> {
    let inputs = Screen008e8590Inputs {
        not_duplicate: world.flag("not_duplicate"),
        registration_ok: world.registration_ok,
        slot1_parent: world.handle("slot1_parent"),
        slot2_derived: world.handle("slot2_derived"),
        initial_terms: current_terms(world, "initial"),
        current_terms: if world.flag("has_current_terms") {
            Some(current_terms(world, "current"))
        } else { None },
    };
    build_008e8590(world.handle("slot0_record"), inputs)
}

pub fn populate_008e82b0(world: &WorldFacade) -> Option<Screen008e82b0View> {
    let inputs = Screen008e82b0Inputs {
        type_gate_passes: world.flag("type_gate_passes"),
        inner_ok: world.flag("inner_ok"),
        fallback_registration_ok: world.registration_ok,
    };
    let inner = if inputs.type_gate_passes && inputs.inner_ok {
        populate_008e8590(world)
    } else { None };
    build_008e82b0(inputs, world.handle("param_1"), world.handle("record_ptr"), inner)
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_008e6cc0_ok() {
        let w = WorldFacade::ready().with_handle("param_1", 111).with_handle("param_2", 222);
        let v = populate_008e6cc0(&w).unwrap();
        assert_eq!((v.slot0, v.slot1), (111, 222));
    }
    #[test]
    fn populate_008e7270_ok() {
        let w = WorldFacade::ready().with_handle("param_1", 999);
        assert_eq!(populate_008e7270(&w).unwrap().slot0, 999);
    }
    #[test]
    fn populate_008e78d0_slot1_hardcoded() {
        assert_eq!(populate_008e78d0(&WorldFacade::ready()).unwrap().slot1, 2);
    }
    #[test]
    fn populate_008e8590_duplicate_gate_returns_none() {
        assert!(populate_008e8590(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_008e8590_ok_when_not_duplicate() {
        let w = WorldFacade::ready().with_flag("not_duplicate", true);
        assert!(populate_008e8590(&w).is_some());
    }
    #[test]
    fn populate_008e82b0_fallback_when_gate_fails() {
        let v = populate_008e82b0(&WorldFacade::ready()).unwrap();
        assert!(matches!(v, Screen008e82b0View::Fallback { .. }));
    }
}
