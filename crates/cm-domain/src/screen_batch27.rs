//! Batch 27: 3 more setup functions.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `008ea760.c` — 8-slot screen, param_1/-1/param_2..param_7 pattern.
//! * `008eaef0.c` — Recall/terminate player loan; guard fires an error
//!   MessageBox on same-manager, otherwise registers a 2-slot screen.
//! * `008f5800.c` — Guarded builder: `param_1 != NULL &&
//!   *(param_1+0x39) != 0` and `FUN_0076eb10` loader must succeed;
//!   otherwise raises "Error" dialog and returns without a screen.

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_008ea760 — 8-slot generic screen
// =====================================================================

/// Direct port of `FUN_008ea760(p1, p2:i8, p3:u16, p4:i8, p5, p6, p7)`.
///
/// Slot layout from the exe (FUN_007e7130 calls):
///   0 = param_1, 1 = 0xffffffff, 2 = param_2 (i8→i32),
///   3 = param_3, 4 = param_4 (i8→i32), 5 = param_5,
///   6 = param_6, 7 = param_7.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericEightSlotView {
    /// Slot 0.
    pub param1: u32,
    /// Slot 1: hard-coded `0xffffffff` in the exe.
    pub sentinel: u32,
    /// Slot 2.
    pub param2: i32,
    /// Slot 3.
    pub param3: u16,
    /// Slot 4.
    pub param4: i32,
    /// Slot 5.
    pub param5: u32,
    /// Slot 6.
    pub param6: u32,
    /// Slot 7.
    pub param7: u32,
}

impl Default for GenericEightSlotView {
    fn default() -> Self {
        Self {
            param1: 0, sentinel: 0xffffffff, param2: 0, param3: 0,
            param4: 0, param5: 0, param6: 0, param7: 0,
        }
    }
}

/// Direct port of `FUN_008ea760`.
#[allow(clippy::too_many_arguments)]
pub fn build_generic_eight_slot(
    registration_ok: bool,
    param1: u32, param2: i8, param3: u16, param4: i8,
    param5: u32, param6: u32, param7: u32,
) -> Option<GenericEightSlotView> {
    if !registration_ok { return None; }
    Some(GenericEightSlotView {
        param1,
        sentinel: 0xffffffff,
        param2: param2 as i32,
        param3,
        param4: param4 as i32,
        param5, param6, param7,
    })
}

// =====================================================================
// FUN_008eaef0 — Recall/terminate loan
// =====================================================================

/// Result of `FUN_008eaef0`. The exe has a same-manager guard that
/// short-circuits to an error message ("Cannot terminate/recall <s>")
/// via FUN_005276f0/FUN_00525190/FUN_0057b9c0 — no screen registered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecallLoanOutcome {
    /// Same-manager guard fired; error dialog shown, kind selected by
    /// whether the recall target IS the active-manager club (terminate)
    /// or another club (recall).
    ErrorDialog { kind: RecallLoanErrorKind },
    /// Normal path: 2-slot screen registered.
    Screen(RecallLoanView),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecallLoanErrorKind {
    /// `*(iVar2+0x39) == *(param_1+0x39)` — active manager's own player.
    CannotTerminate,
    /// Otherwise — recall from another manager.
    CannotRecall,
}

/// Direct port of `FUN_008eaef0`'s 2-slot screen: slot 0 = param_1,
/// slot 1 = `*(param_1 + 0x39)` (the club/person id at +0x39).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RecallLoanView {
    pub player_record: u32,
    pub club_id: u32,
}

/// Direct port of `FUN_008eaef0(param_1)`.
///
/// * `active_manager_club_at_39` = `(&DAT_00b59fc2)[DAT_00b5d016*0xc0] + 0x39`.
/// * `guard_blocks` matches the exe's `FUN_008bd590 == 0` early-out.
/// * `same_manager` picks the error template.
pub fn build_recall_loan(
    registration_ok: bool,
    guard_blocks: bool,
    same_manager: bool,
    player_record: u32,
    club_id: u32,
) -> Option<RecallLoanOutcome> {
    if guard_blocks {
        return Some(RecallLoanOutcome::ErrorDialog {
            kind: if same_manager {
                RecallLoanErrorKind::CannotTerminate
            } else {
                RecallLoanErrorKind::CannotRecall
            },
        });
    }
    if !registration_ok { return None; }
    Some(RecallLoanOutcome::Screen(RecallLoanView {
        player_record, club_id,
    }))
}

// =====================================================================
// FUN_008f5800 — Guarded 4-slot screen (loader-gated)
// =====================================================================

/// Direct port of `FUN_008f5800`'s 4-slot screen.
///
/// Slots: 0 = `*param_1`, 1 = `FUN_0076d7d0(1)`, 2 = `**(param_1+0x39)`,
/// 3 = `param_2`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GuardedLoaderView {
    pub head: u32,
    pub loader_token: u32,
    pub nested: u32,
    pub param2: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardedLoaderOutcome {
    /// `param_1` was null or `*(param_1+0x39) == 0` — silent no-op.
    Skipped,
    /// `FUN_0076eb10` loader failed — the exe shows "Error" MessageBox
    /// (with format `"v%s %s %s %d"`) and clears `DAT_00b4d5a8`.
    LoaderError,
    /// Normal path.
    Screen(GuardedLoaderView),
}

/// Direct port of `FUN_008f5800(param_1, param_2)`.
///
/// * `param1_null_or_zero_at_39` matches
///   `param_1 == NULL || *(param_1+0x39) == 0`.
/// * `loader_ok` matches `FUN_0076eb10 != 0`.
pub fn build_guarded_loader(
    registration_ok: bool,
    param1_null_or_zero_at_39: bool,
    loader_ok: bool,
    head: u32, loader_token: u32, nested: u32, param2: u32,
) -> Option<GuardedLoaderOutcome> {
    if param1_null_or_zero_at_39 { return Some(GuardedLoaderOutcome::Skipped); }
    if !loader_ok { return Some(GuardedLoaderOutcome::LoaderError); }
    if !registration_ok { return None; }
    Some(GuardedLoaderOutcome::Screen(GuardedLoaderView {
        head, loader_token, nested, param2,
    }))
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_eight_slot_carries_all_params_and_sentinel() {
        let v = build_generic_eight_slot(true, 10, -3, 0x1234, 7, 100, 200, 300).unwrap();
        assert_eq!(v.param1, 10);
        assert_eq!(v.sentinel, 0xffffffff);
        assert_eq!(v.param2, -3);
        assert_eq!(v.param3, 0x1234);
        assert_eq!(v.param4, 7);
        assert_eq!(v.param5, 100);
        assert_eq!(v.param6, 200);
        assert_eq!(v.param7, 300);
    }
    #[test]
    fn generic_eight_slot_failed_registration_returns_none() {
        assert!(build_generic_eight_slot(false, 0, 0, 0, 0, 0, 0, 0).is_none());
    }
    #[test]
    fn generic_eight_slot_sign_extends_i8_params() {
        let v = build_generic_eight_slot(true, 0, -1, 0, -128, 0, 0, 0).unwrap();
        assert_eq!(v.param2, -1);
        assert_eq!(v.param4, -128);
    }

    #[test]
    fn recall_loan_normal_path_registers_screen() {
        let out = build_recall_loan(true, false, false, 42, 99).unwrap();
        match out {
            RecallLoanOutcome::Screen(v) => {
                assert_eq!(v.player_record, 42);
                assert_eq!(v.club_id, 99);
            }
            _ => panic!("expected Screen"),
        }
    }
    #[test]
    fn recall_loan_same_manager_guard_fires_terminate_error() {
        let out = build_recall_loan(true, true, true, 0, 0).unwrap();
        assert_eq!(
            out,
            RecallLoanOutcome::ErrorDialog { kind: RecallLoanErrorKind::CannotTerminate }
        );
    }
    #[test]
    fn recall_loan_other_manager_guard_fires_recall_error() {
        let out = build_recall_loan(true, true, false, 0, 0).unwrap();
        assert_eq!(
            out,
            RecallLoanOutcome::ErrorDialog { kind: RecallLoanErrorKind::CannotRecall }
        );
    }
    #[test]
    fn recall_loan_failed_registration_returns_none() {
        assert!(build_recall_loan(false, false, false, 0, 0).is_none());
    }

    #[test]
    fn guarded_loader_normal_path_registers_screen() {
        let out = build_guarded_loader(true, false, true, 1, 2, 3, 4).unwrap();
        match out {
            GuardedLoaderOutcome::Screen(v) => {
                assert_eq!(v.head, 1);
                assert_eq!(v.loader_token, 2);
                assert_eq!(v.nested, 3);
                assert_eq!(v.param2, 4);
            }
            _ => panic!("expected Screen"),
        }
    }
    #[test]
    fn guarded_loader_null_param_skips_silently() {
        let out = build_guarded_loader(true, true, true, 0, 0, 0, 0).unwrap();
        assert_eq!(out, GuardedLoaderOutcome::Skipped);
    }
    #[test]
    fn guarded_loader_loader_failure_shows_error_dialog() {
        let out = build_guarded_loader(true, false, false, 0, 0, 0, 0).unwrap();
        assert_eq!(out, GuardedLoaderOutcome::LoaderError);
    }
    #[test]
    fn guarded_loader_failed_registration_returns_none() {
        assert!(build_guarded_loader(false, false, true, 0, 0, 0, 0).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b27: GenericEightSlotView reads {param1, param5, param6, param7} handles + \
{param2, param3, param4} bytes; RecallLoanView is behind a \
RecallLoanOutcome enum driven by flags {guard_blocks, same_manager} + \
handles {player_record, club_id}; GuardedLoaderView is behind a \
GuardedLoaderOutcome enum driven by flags {param1_null_or_zero_at_39, \
loader_ok} + handles {head, loader_token, nested, param2}.\n\
UNKNOWNS: FUN_008bd590 recall guard, FUN_0076eb10 loader — populators \
consume pre-derived facade flags; production driver must set them.";

use crate::world_facade::WorldFacade;

pub fn populate_generic_eight_slot(world: &WorldFacade) -> Option<GenericEightSlotView> {
    build_generic_eight_slot(
        world.registration_ok,
        world.handle("param1"),
        world.byte("param2") as i8,
        world.byte("param3") as u16,
        world.byte("param4") as i8,
        world.handle("param5"),
        world.handle("param6"),
        world.handle("param7"),
    )
}

pub fn populate_recall_loan(world: &WorldFacade) -> Option<RecallLoanOutcome> {
    build_recall_loan(
        world.registration_ok,
        world.flag("guard_blocks"),
        world.flag("same_manager"),
        world.handle("player_record"),
        world.handle("club_id"),
    )
}

pub fn populate_guarded_loader(world: &WorldFacade) -> Option<GuardedLoaderOutcome> {
    build_guarded_loader(
        world.registration_ok,
        world.flag("param1_null_or_zero_at_39"),
        world.flag("loader_ok"),
        world.handle("head"),
        world.handle("loader_token"),
        world.handle("nested"),
        world.handle("param2"),
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_generic_eight_slot_carries_sentinel() {
        let v = populate_generic_eight_slot(&WorldFacade::ready()).unwrap();
        assert_eq!(v.sentinel, 0xffffffff);
    }
    #[test]
    fn populate_recall_loan_default_registers_screen() {
        let out = populate_recall_loan(&WorldFacade::ready()).unwrap();
        assert!(matches!(out, RecallLoanOutcome::Screen(_)));
    }
    #[test]
    fn populate_recall_loan_guard_fires_error() {
        let w = WorldFacade::ready()
            .with_flag("guard_blocks", true).with_flag("same_manager", true);
        assert!(matches!(
            populate_recall_loan(&w).unwrap(),
            RecallLoanOutcome::ErrorDialog { kind: RecallLoanErrorKind::CannotTerminate }
        ));
    }
    #[test]
    fn populate_guarded_loader_loader_failure_is_error() {
        // default: registration_ok=true, loader_ok=false → LoaderError.
        let out = populate_guarded_loader(&WorldFacade::ready()).unwrap();
        assert_eq!(out, GuardedLoaderOutcome::LoaderError);
    }
    #[test]
    fn populate_guarded_loader_skipped_when_param_null() {
        let w = WorldFacade::ready().with_flag("param1_null_or_zero_at_39", true);
        assert_eq!(populate_guarded_loader(&w).unwrap(), GuardedLoaderOutcome::Skipped);
    }
}
