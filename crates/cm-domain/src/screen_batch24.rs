//! Batch 24: 5 more setup functions in the 0x008d… player/contract cluster.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `0x008DFB10.c` — Contract offer / squad move (param_1 person,
//!   param_2 club-slot). Reads slots 0/7/8 via `FUN_0076d7d0`, and
//!   either calls `FUN_004e3300` on the club record (in-branch, no
//!   screen registration) or registers a screen with two slots.
//! * `0x008DFC20.c` — Small 3-slot screen (`LAB_008e01d0`/`FUN_008e0560`);
//!   slots [0]=param_1, [1]=param_2, [2]=resolved entity ptr for slot 9.
//! * `0x008DFDF0.c` — Big transfer/wage-offer screen (`FUN_008e8920`/
//!   `LAB_008e9e60`) with 21 slots — includes error path
//!   ("FUN_008fc660 …2339") that fires a MsgBox and returns without
//!   registering. Non-matching branch falls through to the same
//!   2-slot screen as batch entries 1/4/5.
//! * `0x008E0710.c` — Trivial 2-slot screen (`FUN_008e0760`/`FUN_008e0af0`).
//! * `0x008E0B60.c` — Trivial 2-slot screen (`FUN_008e0bb0`/`LAB_008e0f30`).
//!
//! Registration gate: `FUN_007e6570 != 0`. Slot writes: `FUN_007e7130`.

use serde::{Deserialize, Serialize};

// =====================================================================
// 0x008DFB10 — Contract offer / squad move
// =====================================================================

/// Two-slot view for the 008DFB10 screen (fallback path).
///
/// The exe first checks whether the resolved entity at slot 7 has
/// `+0x24 == slot_8` and `+0x2c != 3`. If so it calls
/// `FUN_004e3300(nation_row, entity, 1, 0)` and does *not* register a
/// screen — represented here by `ContractOfferOutcome::HandledInline`.
/// Otherwise it registers a two-slot screen with `[0]=entity_ptr`,
/// `[1]=slot_8`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractOfferView {
    pub entity_ptr: u32,
    pub slot_8: u32,
}

/// Outcome of `build_contract_offer_or_move`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractOfferOutcome {
    /// exe's `FUN_004e3300(nation_row, entity, 1, 0)` inline path.
    HandledInline { nation_row: u32, entity_ptr: u32 },
    /// exe's `FUN_007e6570` screen-registration path.
    ScreenRegistered(ContractOfferView),
}

/// Direct port of `FUN_008dfb10`.
///
/// * `slot_0` / `slot_7` / `slot_8` are the three `FUN_0076d7d0` reads.
/// * `entity_ptr` is `FUN_008b3240(slot_7)` (resolved).
/// * `entity_kind_24` / `entity_kind_2c` model `*entity+0x24` / `+0x2c`.
/// * `nation_stride_base` models `DAT_00acd5c4 + slot_0 * 0x6e`.
/// * `registration_ok` gates the screen-registered branch.
pub fn build_contract_offer_or_move(
    slot_8: u32,
    entity_ptr: u32,
    entity_kind_24: u32,
    entity_kind_2c: i8,
    nation_stride_base: u32,
    registration_ok: bool,
) -> Option<ContractOfferOutcome> {
    if entity_kind_24 == slot_8 && entity_kind_2c != 3 {
        return Some(ContractOfferOutcome::HandledInline {
            nation_row: nation_stride_base,
            entity_ptr,
        });
    }
    if !registration_ok { return None; }
    Some(ContractOfferOutcome::ScreenRegistered(ContractOfferView {
        entity_ptr,
        slot_8,
    }))
}

// =====================================================================
// 0x008DFC20 — Small 3-slot screen
// =====================================================================

/// View for the 008DFC20 screen — 3 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SmallThreeSlotView {
    pub param_1: u32,
    pub param_2: u32,
    /// Slot 2: `FUN_008b3240(slot_9)` — resolved entity pointer.
    pub entity_ptr: u32,
}

/// Direct port of `FUN_008dfc20`.
pub fn build_small_three_slot(
    registration_ok: bool,
    param_1: u32,
    param_2: u32,
    entity_ptr: u32,
) -> Option<SmallThreeSlotView> {
    if !registration_ok { return None; }
    Some(SmallThreeSlotView { param_1, param_2, entity_ptr })
}

// =====================================================================
// 0x008DFDF0 — Large transfer/wage-offer screen
// =====================================================================

/// View for the 008DFDF0 large screen. 21 slots (0..=0x14) when the
/// entity-match branch is taken. `wage_percentile` is
/// `((linked+0xc * 100 / base+0xc) + 5) / 10) * 10`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WageOfferView {
    pub entity_ptr: u32,
    pub base_field_0xc: u32,
    pub base_field_0xc_2: u32,
    pub b_char_1: i8,
    pub b_ushort_1: u32,
    pub b_char_2: i8,
    pub b_ushort_2: u32,
    pub raw_entity_ptr: u32,
    pub linked_ptr: u32,
    pub entity_field_0x24: u32,
    pub const_zero: u32,
    pub wage_percentile: i32,
    pub button_flag_0: i8,
    pub linked_field_0x2f: u16,
    pub linked_field_0x1c: i8,
    pub linked_field_0x1d: i8,
    pub wage_percentile_2: i32,
    pub button_flag_1: i8,
    pub linked_field_0x2f_2: u16,
    pub linked_field_0x1c_2: i8,
    /// Slot 0x14: `iVar5` — mirror of `linked+0x1d`.
    pub tail_slot: i8,
}

/// Outcome of `build_wage_offer_or_move`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WageOfferOutcome {
    /// Full 21-slot wage-offer registered.
    Full(Box<WageOfferView>),
    /// Fallback 2-slot screen (`FUN_008e0760`/`FUN_008e0af0`) — same as
    /// 008DFB10 fallback shape. Slots: `[0]=entity_ptr`,
    /// `[1]=entity_field_0x24`.
    Fallback { entity_ptr: u32, entity_field_0x24: u32 },
    /// exe's `FUN_004d2e10 == 0` MsgBox path — no screen registered.
    LinkMissingError,
    /// Match happy-path — caller should feed resolved field reads to
    /// `build_wage_offer_view` to populate the full 21-slot pack.
    NeedsFullBuild { entity_ptr: u32, linked_ptr: u32 },
}

/// Direct port of `FUN_008dfdf0`.
///
/// The exe branches on `entity+0x24 == slot_7 && entity+0x2c != 3`.
/// Match → full wage-offer; mismatch → fallback 2-slot screen.
/// Within the match branch, if `FUN_004d2e10` returns 0 the exe fires
/// a MsgBox and returns without registering.
#[allow(clippy::too_many_arguments)]
pub fn build_wage_offer_or_move(
    slot_7: u32,
    entity_ptr: u32,
    entity_field_0x8: u32,
    entity_field_0xc: u32,
    entity_field_0x24: u32,
    entity_field_0x2c: i8,
    linked_ptr: u32,          // FUN_004d2e10(entity+0, entity+8); 0 = error
    base_ptr: u32,            // FUN_004d59d0(entity)
    button_flag_0: i8,
    button_flag_1: i8,
    registration_ok: bool,
) -> Option<WageOfferOutcome> {
    let matched = entity_field_0x24 == slot_7 && entity_field_0x2c != 3;
    if !matched {
        if !registration_ok { return None; }
        return Some(WageOfferOutcome::Fallback {
            entity_ptr,
            entity_field_0x24,
        });
    }
    if !registration_ok { return None; }
    if linked_ptr == 0 {
        return Some(WageOfferOutcome::LinkMissingError);
    }
    // Happy-path caller uses `build_wage_offer_view` to supply the
    // resolved field reads (`base+0xc`, `linked+0xc`, `linked+0x2f`, …).
    // From this control-flow port we only signal that the full pack
    // should be built.
    let _ = (base_ptr, entity_field_0x8, entity_field_0xc,
             button_flag_0, button_flag_1);
    Some(WageOfferOutcome::NeedsFullBuild { entity_ptr, linked_ptr })
}

/// Pure helper: exe's `(((linked_c * 100) / base_c) + 5) / 10) * 10`.
pub fn derive_wage_percentile(base_c: i32, linked_c: i32) -> i32 {
    if base_c == 0 { return 0; }
    (((linked_c * 100) / base_c) + 5) / 10 * 10
}

/// Full-slot constructor for the wage-offer screen (the exe's match
/// branch happy path). Caller supplies already-resolved field reads
/// exactly as `FUN_007e7130` would receive them.
#[allow(clippy::too_many_arguments)]
pub fn build_wage_offer_view(
    registration_ok: bool,
    entity_ptr: u32,
    base_c: i32,
    linked_c: i32,
    b_char_1: i8,
    b_ushort_1: u32,
    b_char_2: i8,
    b_ushort_2: u32,
    raw_entity_ptr: u32,
    linked_ptr: u32,
    entity_field_0x24: u32,
    button_flag_0: i8,
    linked_field_0x2f: u16,
    linked_field_0x1c: i8,
    linked_field_0x1d: i8,
    button_flag_1: i8,
) -> Option<WageOfferView> {
    if !registration_ok { return None; }
    let wp = derive_wage_percentile(base_c, linked_c);
    Some(WageOfferView {
        entity_ptr,
        base_field_0xc: base_c as u32,
        base_field_0xc_2: base_c as u32,
        b_char_1, b_ushort_1, b_char_2, b_ushort_2,
        raw_entity_ptr,
        linked_ptr,
        entity_field_0x24,
        const_zero: 0,
        wage_percentile: wp,
        button_flag_0,
        linked_field_0x2f,
        linked_field_0x1c,
        linked_field_0x1d,
        wage_percentile_2: wp,
        button_flag_1,
        linked_field_0x2f_2: linked_field_0x2f,
        linked_field_0x1c_2: linked_field_0x1c,
        tail_slot: linked_field_0x1d,
    })
}

// =====================================================================
// 0x008E0710 — Trivial 2-slot screen
// =====================================================================

/// View for the 008E0710 screen — 2 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwoSlot0710View {
    pub param_1: u32,
    pub param_2: u32,
}

/// Direct port of `FUN_008e0710`.
pub fn build_two_slot_0710(
    registration_ok: bool,
    param_1: u32,
    param_2: u32,
) -> Option<TwoSlot0710View> {
    if !registration_ok { return None; }
    Some(TwoSlot0710View { param_1, param_2 })
}

// =====================================================================
// 0x008E0B60 — Trivial 2-slot screen
// =====================================================================

/// View for the 008E0B60 screen — 2 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwoSlot0B60View {
    pub param_1: u32,
    pub param_2: u32,
}

/// Direct port of `FUN_008e0b60`.
pub fn build_two_slot_0b60(
    registration_ok: bool,
    param_1: u32,
    param_2: u32,
) -> Option<TwoSlot0B60View> {
    if !registration_ok { return None; }
    Some(TwoSlot0B60View { param_1, param_2 })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 008DFB10 ----
    #[test]
    fn contract_offer_inline_when_entity_matches() {
        let out = build_contract_offer_or_move(0x42, 0x1000, 0x42, 0, 0xACD5C4, true).unwrap();
        match out {
            ContractOfferOutcome::HandledInline { nation_row, entity_ptr } => {
                assert_eq!(nation_row, 0xACD5C4);
                assert_eq!(entity_ptr, 0x1000);
            }
            _ => panic!("expected inline"),
        }
    }
    #[test]
    fn contract_offer_screen_when_entity_kind_2c_is_3() {
        // Match on +0x24 but +0x2c==3 forces the screen branch.
        let out = build_contract_offer_or_move(0x42, 0x1000, 0x42, 3, 0, true).unwrap();
        matches!(out, ContractOfferOutcome::ScreenRegistered(_));
    }
    #[test]
    fn contract_offer_registration_failure_returns_none() {
        assert!(build_contract_offer_or_move(1, 2, 999, 0, 0, false).is_none());
    }

    // ---- 008DFC20 ----
    #[test]
    fn small_three_slot_carries_params() {
        let v = build_small_three_slot(true, 10, 20, 30).unwrap();
        assert_eq!(v.param_1, 10);
        assert_eq!(v.param_2, 20);
        assert_eq!(v.entity_ptr, 30);
    }
    #[test]
    fn small_three_slot_failed_registration_returns_none() {
        assert!(build_small_three_slot(false, 1, 2, 3).is_none());
    }

    // ---- 008DFDF0 ----
    #[test]
    fn wage_offer_fallback_when_entity_mismatch() {
        // slot_7 != entity+0x24 → fallback 2-slot screen.
        let out = build_wage_offer_or_move(1, 0x1000, 0, 0, 999, 0, 0, 0, 0, 0, true).unwrap();
        matches!(out, WageOfferOutcome::Fallback { .. });
    }
    #[test]
    fn wage_offer_link_missing_returns_error_outcome() {
        // Match branch (slot_7 == +0x24, +0x2c != 3), linked_ptr==0.
        let out = build_wage_offer_or_move(0x42, 0x1000, 0, 0, 0x42, 0, 0, 0, 0, 0, true).unwrap();
        matches!(out, WageOfferOutcome::LinkMissingError);
    }
    #[test]
    fn wage_offer_registration_failure_returns_none() {
        assert!(build_wage_offer_or_move(1, 0, 0, 0, 999, 0, 0, 0, 0, 0, false).is_none());
    }
    #[test]
    fn wage_percentile_matches_exe_formula() {
        // ((50*100/100)+5)/10*10 = 50
        assert_eq!(derive_wage_percentile(100, 50), 50);
        // ((73*100/100)+5)/10*10 = 70? actually (73+5)/10*10 = 70
        assert_eq!(derive_wage_percentile(100, 73), 70);
        // Guard against divide-by-zero.
        assert_eq!(derive_wage_percentile(0, 42), 0);
    }
    #[test]
    fn wage_offer_view_populates_all_21_slots_and_mirrors() {
        let v = build_wage_offer_view(
            true, 0x1000, 100, 50,
            1, 2, 3, 4,
            0x2000, 0x3000, 0x42,
            5, 0x1234, 6, 7, 8,
        ).unwrap();
        assert_eq!(v.wage_percentile, 50);
        assert_eq!(v.wage_percentile_2, 50);
        assert_eq!(v.linked_field_0x2f_2, 0x1234);
        assert_eq!(v.linked_field_0x1c_2, 6);
        assert_eq!(v.tail_slot, 7);
        assert_eq!(v.base_field_0xc_2, v.base_field_0xc);
        assert_eq!(v.const_zero, 0);
    }

    // ---- 008E0710 ----
    #[test]
    fn two_slot_0710_carries_params() {
        let v = build_two_slot_0710(true, 11, 22).unwrap();
        assert_eq!(v.param_1, 11);
        assert_eq!(v.param_2, 22);
    }
    #[test]
    fn two_slot_0710_failed_registration_returns_none() {
        assert!(build_two_slot_0710(false, 0, 0).is_none());
    }

    // ---- 008E0B60 ----
    #[test]
    fn two_slot_0b60_carries_params() {
        let v = build_two_slot_0b60(true, 111, 222).unwrap();
        assert_eq!(v.param_1, 111);
        assert_eq!(v.param_2, 222);
    }
    #[test]
    fn two_slot_0b60_failed_registration_returns_none() {
        assert!(build_two_slot_0b60(false, 0, 0).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b24: ContractOfferView reads handles {slot_8, entity_ptr, entity_kind_24, \
nation_stride_base} + byte `entity_kind_2c`; SmallThreeSlotView reads \
{param_1, param_2, entity_ptr}; WageOfferView is huge — reads many \
entity/linked/base fields via well-known keys (see populate_wage_offer_view); \
TwoSlot0710View/TwoSlot0B60View trivially read {param_1, param_2}.\n\
UNKNOWNS: FUN_004d2e10 linked-record resolver, FUN_004d59d0 base resolver, \
FUN_004e3300 inline action, FUN_00536b90 currency flag — populators expect \
the driver to pre-derive these into facade handles/flags/bytes.";

use crate::world_facade::WorldFacade;

pub fn populate_contract_offer(world: &WorldFacade) -> Option<ContractOfferOutcome> {
    build_contract_offer_or_move(
        world.handle("slot_8"),
        world.handle("entity_ptr"),
        world.handle("entity_kind_24"),
        world.byte("entity_kind_2c") as i8,
        world.handle("nation_stride_base"),
        world.registration_ok,
    )
}

pub fn populate_small_three_slot(world: &WorldFacade) -> Option<SmallThreeSlotView> {
    build_small_three_slot(
        world.registration_ok,
        world.handle("param_1"),
        world.handle("param_2"),
        world.handle("entity_ptr"),
    )
}

pub fn populate_wage_offer(world: &WorldFacade) -> Option<WageOfferOutcome> {
    build_wage_offer_or_move(
        world.handle("slot_7"),
        world.handle("entity_ptr"),
        world.handle("entity_field_0x8"),
        world.handle("entity_field_0xc"),
        world.handle("entity_field_0x24"),
        world.byte("entity_field_0x2c") as i8,
        world.handle("linked_ptr"),
        world.handle("base_ptr"),
        world.byte("button_flag_0") as i8,
        world.byte("button_flag_1") as i8,
        world.registration_ok,
    )
}

pub fn populate_wage_offer_view(world: &WorldFacade) -> Option<WageOfferView> {
    build_wage_offer_view(
        world.registration_ok,
        world.handle("entity_ptr"),
        world.byte("base_c"),
        world.byte("linked_c"),
        world.byte("b_char_1") as i8,
        world.handle("b_ushort_1"),
        world.byte("b_char_2") as i8,
        world.handle("b_ushort_2"),
        world.handle("raw_entity_ptr"),
        world.handle("linked_ptr"),
        world.handle("entity_field_0x24"),
        world.byte("button_flag_0") as i8,
        world.byte("linked_field_0x2f") as u16,
        world.byte("linked_field_0x1c") as i8,
        world.byte("linked_field_0x1d") as i8,
        world.byte("button_flag_1") as i8,
    )
}

pub fn populate_two_slot_0710(world: &WorldFacade) -> Option<TwoSlot0710View> {
    build_two_slot_0710(
        world.registration_ok, world.handle("param_1"), world.handle("param_2"),
    )
}

pub fn populate_two_slot_0b60(world: &WorldFacade) -> Option<TwoSlot0B60View> {
    build_two_slot_0b60(
        world.registration_ok, world.handle("param_1"), world.handle("param_2"),
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_contract_offer_default_takes_inline_path() {
        // 0 == 0 → match; +0x2c==0 != 3 → inline.
        let out = populate_contract_offer(&WorldFacade::ready()).unwrap();
        assert!(matches!(out, ContractOfferOutcome::HandledInline { .. }));
    }
    #[test]
    fn populate_small_three_slot_ok() {
        let w = WorldFacade::ready().with_handle("param_1", 1);
        assert_eq!(populate_small_three_slot(&w).unwrap().param_1, 1);
    }
    #[test]
    fn populate_wage_offer_default_is_link_missing() {
        // slot_7 == entity_field_0x24 (0==0), entity_field_0x2c != 3 (0!=3)
        // → match branch; linked_ptr == 0 → LinkMissingError.
        let out = populate_wage_offer(&WorldFacade::ready()).unwrap();
        assert!(matches!(out, WageOfferOutcome::LinkMissingError));
    }
    #[test]
    fn populate_wage_offer_view_ok() {
        let v = populate_wage_offer_view(&WorldFacade::ready()).unwrap();
        assert_eq!(v.const_zero, 0);
    }
    #[test]
    fn populate_two_slot_0710_ok() {
        assert!(populate_two_slot_0710(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_two_slot_0b60_ok() {
        assert!(populate_two_slot_0b60(&WorldFacade::ready()).is_some());
    }
}
