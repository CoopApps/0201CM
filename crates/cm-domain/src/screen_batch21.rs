//! Batch 21: 5 more screen setup / event-handler decompiles.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `00859600.c` — small dispatcher: FUN_007dfac0(1) gate; if the
//!   staff-loaded check fails and `param_1 == 0` it MsgBox's an error
//!   (from `cm3_code/staff` src:0x1A6), else it registers the screen
//!   via `FUN_007e6570(&LAB_0085e580, FUN_00874a10, 0,
//!   &LAB_00876680, 0)` — the offer/handler screen at 0x0085E580.
//! * `00873040.c` — 1073-line input/event handler for a squad-list
//!   screen. Consumes `DAT_00dbbf7c` key codes, returns negative
//!   error codes. NOT a screen builder — no FUN_007e6570 registration
//!   and only conditional `FUN_007e7130` writes on exit paths. STUB.
//! * `00874a10.c` — 971-line sibling of 00873040 (same shape,
//!   simpler param list). Event handler, not builder. STUB.
//! * `00877170.c` — trivial one-liner: FUN_007e6570(&LAB_00877190,
//!   FUN_008773e0, 1, 0, 0). Registers-and-returns.
//! * `00884700.c` — tactic-screen builder. Takes 6 params, allocates
//!   0xC4E2 bytes via `operator new`, calls FUN_008939c0 to init the
//!   record, registers via FUN_007e6570 (label chosen by param_2),
//!   then pushes 11 slots (0..=10) with FUN_007e7130.

use serde::{Deserialize, Serialize};

// =====================================================================
// Offer/Screen dispatcher — FUN_00859600(param_1)
// =====================================================================

/// Result of the dispatcher. On the staff-not-loaded error path the
/// exe MsgBox's "Error" and sets DAT_00b4d5a8 = 0 — we surface it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfferScreenDispatch {
    /// Staff loader hadn't run and `param_1 == 0` — MsgBox error path.
    StaffLoadError,
    /// Normal register — the exe called FUN_007e6570 with the offer
    /// screen labels (0x0085E580 builder, FUN_00874a10 handler,
    /// 0x00876680 tail).
    Registered,
}

/// Direct port of `FUN_00859600(param_1)`.
///
/// * `staff_loaded` — result of `FUN_007dfac0(1) != 0`.
/// * `param_1` — the caller's `param_1` (an entity id or 0).
pub fn dispatch_offer_screen(staff_loaded: bool, param_1: i32) -> OfferScreenDispatch {
    if !staff_loaded && param_1 == 0 {
        OfferScreenDispatch::StaffLoadError
    } else {
        OfferScreenDispatch::Registered
    }
}

// =====================================================================
// Squad-list key handler — FUN_00873040 (event handler; STUB)
// =====================================================================

/// STUB — `FUN_00873040` is a 1073-line input/event handler, not a
/// screen builder. It reads `DAT_00dbbf7c` (last key code),
/// dispatches by opcode (0x18/0x27/0x42/0x3C/etc), reads slot 0..0x12
/// via `FUN_007e6ee0`, and returns negative error codes (-10, -0xB,
/// ...). There is no `FUN_007e6570` registration and no unconditional
/// slot-push cascade. Porting the full event semantics is out of
/// scope for the screen-builder pattern this batch follows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SquadListKeyHandlerStub;

// =====================================================================
// Sibling squad-list key handler — FUN_00874a10 (event handler; STUB)
// =====================================================================

/// STUB — `FUN_00874a10` is the 971-line sibling of `FUN_00873040`
/// with the same event-handler shape (key opcode dispatch, negative
/// return codes, no registration). Same reasoning: not a builder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SquadListKeyHandlerAltStub;

// =====================================================================
// Simple screen register — FUN_00877170
// =====================================================================

/// The 00877190 screen has no setup parameters — the function only
/// exists to register the (builder, handler) pair. Modelled here as a
/// zero-slot view so callers can hang metadata off it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SimpleScreen00877190View;

/// Direct port of `FUN_00877170`. Always attempts registration; we
/// still gate on `registration_ok` so tests can assert the None path.
pub fn build_simple_screen_00877190(registration_ok: bool)
    -> Option<SimpleScreen00877190View>
{
    if !registration_ok { return None; }
    Some(SimpleScreen00877190View)
}

// =====================================================================
// Tactic screen — FUN_00884700(tactic, mode, extra1, side_a, extra2, side_b)
// =====================================================================

/// Tactic-screen view — 11 slots (0..=10) pushed by FUN_007e7130.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TacticScreenView {
    /// Slot 0: pointer to the freshly allocated 0xC4E2-byte tactic
    /// record (from FUN_008939c0). Modelled as an opaque handle.
    pub tactic_record: u32,
    /// Slot 1: `param_1` — primary tactic id / entity (non-zero).
    pub tactic_id: u32,
    /// Slot 2: `param_3` — extra parameter #1.
    pub extra_a: u32,
    /// Slot 3: `param_2` — mode byte (selects builder label:
    /// LAB_00884B50 when 0, DAT_0088A840 otherwise).
    pub mode: u32,
    /// Slot 4: `param_4` — team/side A id (must be non-zero).
    pub side_a: u32,
    /// Slot 5: 0 (reserved).
    /// Slot 6: 0 (reserved).
    /// Slot 7: `param_5` — extra parameter #2.
    pub extra_b: u32,
    /// Slot 8: `param_6` — team/side B id (must be non-zero).
    pub side_b: u32,
    /// Slot 9: 0xFFFFFFFF (sentinel).
    /// Slot 10: `(iVar3 != 0 ? 6 : 2)` — where `iVar3 = *(tactic+4)`.
    /// The exe computes `(-(iVar3 != 0) & 4U) + 2` → 2 or 6.
    pub slot10_flag: u32,
}

/// Direct port of `FUN_00884700`. Returns `None` on the exe's
/// early-exit paths (all of which either MsgBox an error or bail
/// without pushing slots):
///
/// * `param_1 == 0` — tactic pointer null → error MsgBox.
/// * `param_4 == 0` || `param_6 == 0` — one of the sides missing;
///   the exe branches to a slot-less path that only registers and
///   returns (no slot pushes to observe).
/// * `alloc_ok == false` — `operator new(0xC4E2)` returned null.
/// * `registration_ok == false` — FUN_007e6570 failed.
///
/// `tactic_head_flag` is the value of `*(record + 4)` after init;
/// it selects between the two label groups for slot 10.
pub fn build_tactic_screen(
    registration_ok: bool,
    alloc_ok: bool,
    param_1: u32,
    param_2: u32,
    param_3: u32,
    param_4: u32,
    param_5: u32,
    param_6: u32,
    tactic_record_handle: u32,
    tactic_head_flag: bool,
) -> Option<TacticScreenView> {
    if param_1 == 0 { return None; }
    if param_4 == 0 || param_6 == 0 { return None; }
    if !alloc_ok { return None; }
    if !registration_ok { return None; }
    Some(TacticScreenView {
        tactic_record: tactic_record_handle,
        tactic_id: param_1,
        extra_a: param_3,
        mode: param_2,
        side_a: param_4,
        extra_b: param_5,
        side_b: param_6,
        slot10_flag: if tactic_head_flag { 6 } else { 2 },
    })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- 00859600 dispatcher ---
    #[test]
    fn dispatch_offer_screen_error_when_staff_missing_and_param1_zero() {
        assert_eq!(
            dispatch_offer_screen(false, 0),
            OfferScreenDispatch::StaffLoadError
        );
    }
    #[test]
    fn dispatch_offer_screen_registers_when_staff_loaded() {
        assert_eq!(
            dispatch_offer_screen(true, 0),
            OfferScreenDispatch::Registered
        );
    }
    #[test]
    fn dispatch_offer_screen_registers_when_param1_nonzero() {
        // Non-zero param_1 skips the staff-missing error even if
        // staff hadn't loaded.
        assert_eq!(
            dispatch_offer_screen(false, 42),
            OfferScreenDispatch::Registered
        );
    }

    // --- 00877170 simple register ---
    #[test]
    fn simple_screen_00877190_registers() {
        assert!(build_simple_screen_00877190(true).is_some());
    }
    #[test]
    fn simple_screen_00877190_returns_none_on_failed_register() {
        assert!(build_simple_screen_00877190(false).is_none());
    }

    // --- 00884700 tactic screen ---
    #[test]
    fn tactic_screen_normal_flow_carries_all_slots() {
        let v = build_tactic_screen(
            true, true,
            /*p1*/ 100, /*p2*/ 3, /*p3*/ 7,
            /*p4*/ 501, /*p5*/ 9, /*p6*/ 502,
            /*handle*/ 0xdead_beef,
            /*head_flag*/ false,
        ).unwrap();
        assert_eq!(v.tactic_record, 0xdead_beef);
        assert_eq!(v.tactic_id, 100);
        assert_eq!(v.mode, 3);
        assert_eq!(v.extra_a, 7);
        assert_eq!(v.side_a, 501);
        assert_eq!(v.side_b, 502);
        assert_eq!(v.extra_b, 9);
        assert_eq!(v.slot10_flag, 2);
    }
    #[test]
    fn tactic_screen_head_flag_selects_slot10_6() {
        let v = build_tactic_screen(
            true, true, 1, 0, 0, 2, 0, 3, 0, true,
        ).unwrap();
        assert_eq!(v.slot10_flag, 6);
    }
    #[test]
    fn tactic_screen_null_tactic_returns_none() {
        assert!(build_tactic_screen(
            true, true, 0, 0, 0, 1, 0, 2, 0, false,
        ).is_none());
    }
    #[test]
    fn tactic_screen_null_side_returns_none() {
        assert!(build_tactic_screen(
            true, true, 1, 0, 0, 0, 0, 2, 0, false,
        ).is_none());
        assert!(build_tactic_screen(
            true, true, 1, 0, 0, 2, 0, 0, 0, false,
        ).is_none());
    }
    #[test]
    fn tactic_screen_failed_alloc_returns_none() {
        assert!(build_tactic_screen(
            true, false, 1, 0, 0, 2, 0, 3, 0, false,
        ).is_none());
    }
    #[test]
    fn tactic_screen_failed_registration_returns_none() {
        assert!(build_tactic_screen(
            false, true, 1, 0, 0, 2, 0, 3, 0, false,
        ).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b21: SimpleScreen00877190View needs only registration_ok; TacticScreenView \
reads handles {tactic_id, param_2..param_6, tactic_record_handle} + flags \
{alloc_ok, tactic_head_flag}. StaffLoad dispatcher (00859600) has its own \
enum outcome, not a View — no populator.\n\
UNKNOWNS: FUN_008939c0 tactic-record init not ported; populators pass the \
provided handle through unchanged. FUN_00873040/FUN_00874a10 are stubs \
(event handlers), not screens.";

use crate::world_facade::WorldFacade;

pub fn populate_simple_screen_00877190(world: &WorldFacade) -> Option<SimpleScreen00877190View> {
    build_simple_screen_00877190(world.registration_ok)
}

pub fn populate_tactic_screen(world: &WorldFacade) -> Option<TacticScreenView> {
    build_tactic_screen(
        world.registration_ok,
        world.flag("alloc_ok"),
        world.handle("tactic_id"),
        world.handle("mode"),
        world.handle("extra_a"),
        world.handle("side_a"),
        world.handle("extra_b"),
        world.handle("side_b"),
        world.handle("tactic_record_handle"),
        world.flag("tactic_head_flag"),
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_simple_needs_registration() {
        assert!(populate_simple_screen_00877190(&WorldFacade::default()).is_none());
        assert!(populate_simple_screen_00877190(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_tactic_screen_missing_tactic_id_returns_none() {
        // default: tactic_id=0 → early exit.
        assert!(populate_tactic_screen(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_tactic_screen_happy_path() {
        let w = WorldFacade::ready()
            .with_flag("alloc_ok", true)
            .with_handle("tactic_id", 100)
            .with_handle("side_a", 501)
            .with_handle("side_b", 502);
        let v = populate_tactic_screen(&w).unwrap();
        assert_eq!(v.tactic_id, 100);
        assert_eq!(v.slot10_flag, 2);
    }
}
