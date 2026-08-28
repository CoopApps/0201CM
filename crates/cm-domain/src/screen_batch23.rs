//! Batch 23: 5 more screen-setup functions from the manager screens
//! cluster (~0x008d*).
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `008da210.c` — huge message-dispatch handler (>1500 lines,
//!   dozens of sVar3 command codes: 1, 2, 3..0x27). NOT a screen
//!   setup — stubbed here as a placeholder.
//! * `008dcfd0.c` — thin 3-param builder for the `FUN_008dd030`
//!   screen (slots 0/1/2).
//! * `008def70.c` — builder for the `LAB_008df0c0` screen (slots
//!   0/1/2/4 fixed; optional slot 3 pushes for related staff when
//!   param_3 == 0x12 or 0x20).
//! * `008df8f0.c` — dual-purpose handler: either fires the
//!   confirmation dialog `FUN_008d6510(_, 1)` (when the fetched
//!   record's field @0x24 matches and field @0x2c != 3), or falls
//!   through and builds the `FUN_008e0760` screen (slots 0/1).
//! * `008df9f0.c` — sibling of `008df8f0.c`. Same shape, but the
//!   condition adds `field@0x2c != 2`, and the "matched" branch
//!   calls `FUN_008d6310` (a different action) instead of firing
//!   a confirmation dialog.

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_008da210 — message-dispatch handler (STUB; not a screen setup)
// =====================================================================

/// Marker for the (very large) `FUN_008da210` message-dispatch
/// handler. It is NOT a screen-setup function — it reads the current
/// widget-pool command code (`(&DAT_00b59fe8)[...] + 0xba966 + sVar4*0x18c`)
/// and multiplexes across ~24 sub-commands (0x01..0x27), most of
/// which mutate a person/manager record at offsets 0x1b, 0x35, 0x3a,
/// 0x3e, 0x3f, 0x40 (contract terms and toggles).
///
/// We keep the shape here as a stub so the batch registry stays
/// complete; the real port belongs with the manager-record edit
/// screen, which owns those field offsets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ManagerDispatchStub {
    /// The command code (short) that would be dispatched on.
    pub cmd: i16,
}

/// Stub for `FUN_008da210(param_1)`. Not yet ported.
pub fn dispatch_manager_message_stub(cmd: i16) -> ManagerDispatchStub {
    ManagerDispatchStub { cmd }
}

// =====================================================================
// FUN_008dcfd0 — 3-param builder for the FUN_008dd030 screen
// =====================================================================

/// View built by `FUN_008dcfd0(param_1, param_2, param_3)` on top
/// of the `FUN_008dd030` screen. Three slots pushed via
/// `FUN_007E7130(0/1/2, ...)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008dd030View {
    /// Slot 0: `param_1` (typically the fetched record ptr).
    pub p0: u32,
    /// Slot 1: `param_2` (typically the entity-family / focus id).
    pub p1: u32,
    /// Slot 2: `param_3` (typically a sub-mode / filter code).
    pub p2: u32,
}

/// Direct port of `FUN_008dcfd0`. Returns `None` if the screen
/// registration (`FUN_007E6570`) failed.
pub fn build_screen_008dd030(
    registration_ok: bool,
    p0: u32, p1: u32, p2: u32,
) -> Option<Screen008dd030View> {
    if !registration_ok { return None; }
    Some(Screen008dd030View { p0, p1, p2 })
}

// =====================================================================
// FUN_008def70 — builder for the LAB_008df0c0 screen
// =====================================================================

/// View built by `FUN_008def70(param_1, param_2, param_3, param_4)`
/// on top of the `LAB_008df0c0` screen (draw) / `FUN_008df880`
/// (dtor). Four fixed slots (0/1/2/4) and, when `param_3` is 0x12
/// or 0x20, an extra dynamic sequence of related-staff ids pushed
/// via slot 3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008df0c0View {
    /// Slot 0.
    pub p0: u32,
    /// Slot 1.
    pub p1: u32,
    /// Slot 2 (widened from `short param_3`).
    pub p2: i16,
    /// Slot 4 (widened from `char param_4`).
    pub p4: i8,
    /// Slot 3: iterated related-staff ids (only when p2 == 0x12 or
    /// 0x20). Each is the record's field@8, filtered by
    /// `field@0x2c ∈ {0x0B..=0x0E, 0x13}` (staff-role tags).
    pub related_staff: Vec<u32>,
}

/// Direct port of `FUN_008def70`. Returns `None` if the screen
/// registration failed.
pub fn build_screen_008df0c0(
    registration_ok: bool,
    p0: u32, p1: u32, p2: i16, p4: i8,
    related_staff_for_special: Vec<u32>,
) -> Option<Screen008df0c0View> {
    if !registration_ok { return None; }
    let related_staff = if p2 == 0x12 || p2 == 0x20 {
        related_staff_for_special
    } else {
        Vec::new()
    };
    Some(Screen008df0c0View { p0, p1, p2, p4, related_staff })
}

// =====================================================================
// FUN_008df8f0 — confirm-or-open handler
// =====================================================================

/// Outcome of `FUN_008df8f0(param_1, param_2)`.
///
/// The exe fetches the current pool record via `FUN_008B3240(uVar2)`
/// and inspects two of its fields:
/// * `field@0x24` — the "expected/anchor" id, compared to
///   `FUN_0076D7D0(7)` (a widget-pool slot).
/// * `field@0x2c` — a role tag.
///
/// If the anchor matches and `field@0x2c != 3`, the exe fires the
/// confirmation dialog via `FUN_008D6510(record, 1)` — represented
/// here as `Confirm`. Otherwise it falls through and builds the
/// `FUN_008E0760` screen with slots 0/1 = record & anchor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfirmOrOpen008df8f0 {
    /// Confirmation dialog on the record (payload = record ptr).
    Confirm(u32),
    /// Screen built with slots 0/1.
    Screen(Screen008e0760View),
    /// Registration for the fall-through screen failed.
    Failed,
}

/// Shared 2-slot view for `FUN_008e0760`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Screen008e0760View {
    /// Slot 0: fetched record ptr (`*piVar4`).
    pub record: u32,
    /// Slot 1: the anchor id (`FUN_0076D7D0(7)`).
    pub anchor: u32,
}

/// Direct port of `FUN_008df8f0`.
///
/// * `record` = `*FUN_008B3240(FUN_0076D7D0(6))`
/// * `anchor` = `FUN_0076D7D0(7)`
/// * `field_24`, `field_2c` = the two record fields inspected.
/// * `registration_ok` = fall-through screen registration result.
pub fn handle_screen_008df8f0(
    record: u32, anchor: u32,
    field_24: u32, field_2c: i8,
    registration_ok: bool,
) -> ConfirmOrOpen008df8f0 {
    if field_24 == anchor && field_2c != 0x03 {
        ConfirmOrOpen008df8f0::Confirm(record)
    } else if registration_ok {
        ConfirmOrOpen008df8f0::Screen(Screen008e0760View { record, anchor })
    } else {
        ConfirmOrOpen008df8f0::Failed
    }
}

// =====================================================================
// FUN_008df9f0 — sibling of 008df8f0 with a different action
// =====================================================================

/// Outcome of `FUN_008df9f0(param_1, param_2)`.
///
/// Same shape as `FUN_008df8f0` but:
/// * the "matched" condition also requires `field@0x2c != 2`, and
/// * the matched action is `FUN_008D6310(<nation-block>, record)`
///   — a state mutation rather than a UI confirm dialog. Represented
///   here as `Action`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionOrOpen008df9f0 {
    /// The matched-branch action: apply `FUN_008D6310` on the
    /// nation-block for the record's field@0x0c and the record ptr.
    Action { record: u32, nation_block_ord: u32 },
    /// Fall-through screen with slots 0/1.
    Screen(Screen008e0760View),
    /// Registration for the fall-through screen failed.
    Failed,
}

/// Direct port of `FUN_008df9f0`.
///
/// The `nation_block_ord` is the record's field@0x0c (an ordinal
/// into `DAT_00ACD5C4`'s 0x6e-byte per-nation block table); it is
/// only meaningful for the `Action` variant.
pub fn handle_screen_008df9f0(
    record: u32, anchor: u32,
    field_24: u32, field_2c: i8, nation_block_ord: u32,
    registration_ok: bool,
) -> ActionOrOpen008df9f0 {
    if field_24 == anchor && field_2c != 0x03 && field_2c != 0x02 {
        ActionOrOpen008df9f0::Action { record, nation_block_ord }
    } else if registration_ok {
        ActionOrOpen008df9f0::Screen(Screen008e0760View { record, anchor })
    } else {
        ActionOrOpen008df9f0::Failed
    }
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_stub_carries_cmd() {
        assert_eq!(dispatch_manager_message_stub(0x23).cmd, 0x23);
    }
    #[test]
    fn dispatch_stub_default_is_zero() {
        assert_eq!(ManagerDispatchStub::default().cmd, 0);
    }

    #[test]
    fn screen_008dd030_carries_three_slots() {
        let v = build_screen_008dd030(true, 10, 20, 30).unwrap();
        assert_eq!((v.p0, v.p1, v.p2), (10, 20, 30));
    }
    #[test]
    fn screen_008dd030_failed_registration_returns_none() {
        assert!(build_screen_008dd030(false, 1, 2, 3).is_none());
    }

    #[test]
    fn screen_008df0c0_carries_four_fixed_slots() {
        let v = build_screen_008df0c0(true, 5, 6, 0x10, 42, vec![]).unwrap();
        assert_eq!(v.p0, 5);
        assert_eq!(v.p1, 6);
        assert_eq!(v.p2, 0x10);
        assert_eq!(v.p4, 42);
        assert!(v.related_staff.is_empty());
    }
    #[test]
    fn screen_008df0c0_special_p2_carries_related_staff() {
        let v = build_screen_008df0c0(true, 0, 0, 0x12, 0, vec![100, 200]).unwrap();
        assert_eq!(v.related_staff, vec![100, 200]);
        let v2 = build_screen_008df0c0(true, 0, 0, 0x20, 0, vec![7]).unwrap();
        assert_eq!(v2.related_staff, vec![7]);
    }
    #[test]
    fn screen_008df0c0_ignores_related_staff_on_normal_p2() {
        let v = build_screen_008df0c0(true, 0, 0, 0x11, 0, vec![100]).unwrap();
        assert!(v.related_staff.is_empty());
    }
    #[test]
    fn screen_008df0c0_failed_registration_returns_none() {
        assert!(build_screen_008df0c0(false, 0, 0, 0, 0, vec![]).is_none());
    }

    #[test]
    fn handle_008df8f0_fires_confirm_when_anchor_matches_and_field2c_not_3() {
        let r = handle_screen_008df8f0(0xDEAD, 7, 7, 1, true);
        assert_eq!(r, ConfirmOrOpen008df8f0::Confirm(0xDEAD));
    }
    #[test]
    fn handle_008df8f0_opens_screen_when_field2c_is_3() {
        let r = handle_screen_008df8f0(1, 7, 7, 3, true);
        match r {
            ConfirmOrOpen008df8f0::Screen(v) => {
                assert_eq!(v.record, 1);
                assert_eq!(v.anchor, 7);
            }
            _ => panic!("expected Screen"),
        }
    }
    #[test]
    fn handle_008df8f0_opens_screen_when_anchor_mismatch() {
        let r = handle_screen_008df8f0(1, 7, 8, 1, true);
        assert!(matches!(r, ConfirmOrOpen008df8f0::Screen(_)));
    }
    #[test]
    fn handle_008df8f0_failed_registration_gives_failed() {
        let r = handle_screen_008df8f0(1, 7, 8, 1, false);
        assert_eq!(r, ConfirmOrOpen008df8f0::Failed);
    }

    #[test]
    fn handle_008df9f0_fires_action_when_conditions_met() {
        let r = handle_screen_008df9f0(0xBEEF, 5, 5, 1, 42, true);
        assert_eq!(
            r,
            ActionOrOpen008df9f0::Action { record: 0xBEEF, nation_block_ord: 42 }
        );
    }
    #[test]
    fn handle_008df9f0_screen_when_field2c_is_2() {
        let r = handle_screen_008df9f0(1, 5, 5, 2, 0, true);
        assert!(matches!(r, ActionOrOpen008df9f0::Screen(_)));
    }
    #[test]
    fn handle_008df9f0_screen_when_field2c_is_3() {
        let r = handle_screen_008df9f0(1, 5, 5, 3, 0, true);
        assert!(matches!(r, ActionOrOpen008df9f0::Screen(_)));
    }
    #[test]
    fn handle_008df9f0_failed_registration_gives_failed() {
        let r = handle_screen_008df9f0(1, 5, 6, 1, 0, false);
        assert_eq!(r, ActionOrOpen008df9f0::Failed);
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b23: Screen008dd030View reads {p0, p1, p2}; Screen008df0c0View reads \
{p0, p1} + bytes {p2, p4} + buffer `related_staff` (as u32s decoded from \
`related_staff_ids` handles set 0..N); Screen008e0760View is a fallthrough \
built by populate_handle_008df8f0 and populate_handle_008df9f0. \
ManagerDispatchStub (008da210) is a stub — no populator.\n\
UNKNOWNS: FUN_008B3240 record lookup, FUN_0076D7D0 slot pool, and the \
related-staff iterator (field@0x2c filter over the pool) are not ported.";

use crate::world_facade::WorldFacade;

fn related_staff_from(world: &WorldFacade) -> Vec<u32> {
    (0..)
        .map_while(|i| world.handles.get(&format!("related_staff_{}", i)).copied())
        .collect()
}

pub fn populate_screen_008dd030(world: &WorldFacade) -> Option<Screen008dd030View> {
    build_screen_008dd030(
        world.registration_ok,
        world.handle("p0"), world.handle("p1"), world.handle("p2"),
    )
}

pub fn populate_screen_008df0c0(world: &WorldFacade) -> Option<Screen008df0c0View> {
    build_screen_008df0c0(
        world.registration_ok,
        world.handle("p0"), world.handle("p1"),
        world.byte("p2") as i16,
        world.byte("p4") as i8,
        related_staff_from(world),
    )
}

/// Populator for the fall-through `Screen008e0760View` (used by both
/// FUN_008df8f0 and FUN_008df9f0 handlers). Just carries record+anchor.
pub fn populate_screen_008e0760(world: &WorldFacade) -> Option<Screen008e0760View> {
    if !world.registration_ok { return None; }
    Some(Screen008e0760View {
        record: world.handle("record"),
        anchor: world.handle("anchor"),
    })
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_screen_008dd030_carries_three_slots() {
        let w = WorldFacade::ready()
            .with_handle("p0", 10).with_handle("p1", 20).with_handle("p2", 30);
        let v = populate_screen_008dd030(&w).unwrap();
        assert_eq!((v.p0, v.p1, v.p2), (10, 20, 30));
    }
    #[test]
    fn populate_screen_008df0c0_normal_p2_ignores_related_staff() {
        let w = WorldFacade::ready()
            .with_byte("p2", 0x11)
            .with_handle("related_staff_0", 100);
        let v = populate_screen_008df0c0(&w).unwrap();
        assert!(v.related_staff.is_empty());
    }
    #[test]
    fn populate_screen_008df0c0_special_p2_collects_related_staff() {
        let w = WorldFacade::ready()
            .with_byte("p2", 0x12)
            .with_handle("related_staff_0", 100)
            .with_handle("related_staff_1", 200);
        let v = populate_screen_008df0c0(&w).unwrap();
        assert_eq!(v.related_staff, vec![100, 200]);
    }
    #[test]
    fn populate_screen_008e0760_carries_record_and_anchor() {
        let w = WorldFacade::ready()
            .with_handle("record", 1).with_handle("anchor", 7);
        let v = populate_screen_008e0760(&w).unwrap();
        assert_eq!(v.record, 1); assert_eq!(v.anchor, 7);
    }
    #[test]
    fn populate_screen_008e0760_none_without_registration() {
        assert!(populate_screen_008e0760(&WorldFacade::default()).is_none());
    }
}
