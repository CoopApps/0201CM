//! Batch 13: 5 more setup functions — early alpha/comp lookup + a small
//! cluster of contract/offer screens.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `0x004A17F0.c` — Comp-lookup screen (param_1 = id); validates id
//!   via `FUN_004a5760`, then registers with 5 slots.
//! * `0x004E2B80.c` — Two-key lookup screen (param_1, param_2); resolves
//!   two extra ids via `FUN_0076d7d0(0)` / `FUN_0076d7d0(2)` before
//!   registration, pushes 4 slots.
//! * `0x004E3300.c` — Contract-negotiation dispatcher; picks between
//!   itself (7-slot register), `FUN_004E4580` and `FUN_004E4960`. The
//!   dispatch logic is data-heavy (player attributes, RNG, prior seats)
//!   so we surface only the *registration-path* slot writes here and
//!   expose a `ContractDispatch` enum for the routing decision.
//! * `0x004E4580.c` — Contract-Offer screen (self, no comparator).
//!   ~30 slots including a fixed 0x76c constant (transfer window years)
//!   and computed wage/estimate slots. We model the deterministic
//!   subset — the rest is left as `..Default::default()`.
//! * `0x004E4960.c` — Contract-Offer-with-Comparator screen (same shell
//!   as above, plus a `param_1` comparator record).

use serde::{Deserialize, Serialize};

// =====================================================================
// 0x004A17F0 — Comp-lookup screen
// =====================================================================

/// Comp-lookup screen — 5 slots. Slot 0 = resolved key (`local_204`),
/// 1 = `local_20c`, 2 = `local_208`, 3/4 = 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CompLookupView {
    pub resolved_key: u32,
    pub extra_a: u32,
    pub extra_b: u32,
}

/// Direct port of `FUN_004a17f0(param_1)`. Fires the "Error / find_screens:0x1b40"
/// MsgBox and returns `None` when `param_1 == 0`, or when the lookup
/// (`FUN_004a5760`) returns 0 AND both `local_20c`/`local_208` are 0.
///
/// `lookup` is the tuple `(iVar1, local_20c, local_208)` — a real
/// port also emits `local_204` (the resolved key) as `resolved_key`.
pub fn build_comp_lookup(
    registration_ok: bool,
    param_1: u32,
    lookup: (i32, u32, u32),
    resolved_key: u32,
) -> Option<CompLookupView> {
    if param_1 == 0 { return None; }
    // exe: `9999 < ((p ^ p>>31) - p>>31)` — i.e. `|param_1| > 9999`.
    let magnitude = (param_1 as i32).unsigned_abs();
    if magnitude <= 9999 { return None; }
    let (i_var1, local_20c, local_208) = lookup;
    if i_var1 == 0 && local_20c == 0 && local_208 == 0 { return None; }
    if !registration_ok { return None; }
    Some(CompLookupView { resolved_key, extra_a: local_20c, extra_b: local_208 })
}

// =====================================================================
// 0x004E2B80 — Two-key lookup screen
// =====================================================================

/// Two-key lookup screen. Slots 0/1 = incoming params, 2/3 = extra keys
/// resolved via `FUN_0076d7d0(0)` and `FUN_0076d7d0(2)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwoKeyLookupView {
    pub param_1: u32,
    pub param_2: u32,
    pub extra_0: u32,
    pub extra_2: u32,
}

/// Direct port of `FUN_004e2b80(param_1, param_2)`.
pub fn build_two_key_lookup(
    registration_ok: bool,
    param_1: u32,
    param_2: u32,
    extra_0: u32,
    extra_2: u32,
) -> Option<TwoKeyLookupView> {
    if !registration_ok { return None; }
    Some(TwoKeyLookupView { param_1, param_2, extra_0, extra_2 })
}

// =====================================================================
// 0x004E3300 — Contract negotiation dispatcher
// =====================================================================

/// Where `FUN_004e3300` routes: register itself, delegate to the
/// no-comparator offer screen (`FUN_004E4580`), or the with-comparator
/// offer screen (`FUN_004E4960`), or nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractDispatch {
    /// Self-registered; caller pushes the 7-slot payload.
    Register(ContractNegotiationView),
    /// Fell through to `FUN_004E4580(param_1, bVar1, cVar7, 1)`.
    OfferSelf,
    /// Fell through to `FUN_004E4960(param_2, param_1, bVar1, cVar7, 1)`.
    OfferWithComparator,
    /// Nothing to render (no active manager seat / no club at +0x39).
    None,
}

/// Contract-negotiation self-registration payload — 7-slot writes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractNegotiationView {
    /// Slot 0: player record ptr key (`param_1`).
    pub player_key: u32,
    /// Slot 1: role byte (`bVar1` — either input override or 0).
    pub role: i8,
    /// Slot 6: same as slot 1 (dup-write, exe pushes both).
    pub role_dup: i8,
    /// Slot 2: comparator ptr key (`param_2`) — 0 when absent.
    pub comparator_key: u32,
    /// Slot 3: `param_3` (forced 1 when comparator supplied).
    pub flag_3: u32,
    /// Slot 4: derived age/side byte (`cVar7`).
    pub derived_byte: i8,
    /// Slot 5: `iVar4` — offer record ptr key (0 when none).
    pub offer_key: u32,
    /// Slot 7: comparator's owning-club key or 0.
    pub comparator_club_key: u32,
}

/// Direct port of `FUN_004e3300(param_1, param_2, param_3, param_4)` —
/// the register-path only. `manager_seat_valid` matches the
/// `(&DAT_00b59fc2)[DAT_00b5d016 * 0xc0] != 0` gate; `club_ptr_valid`
/// matches `*(iVar4 + 0x39) != 0`. Returns `None` when either gate
/// fails (exe fires the "contra:0x18a"/"0x192" error MsgBox).
///
/// The exe's fall-through-to-offer paths are represented via
/// `ContractDispatch` variants — the caller (menu dispatcher) picks
/// which builder to invoke based on the routing predicates it already
/// knows (they are recomputed here in Rust, not decoded from the exe's
/// float-heavy sub-branches).
pub fn build_contract_negotiation(
    registration_ok: bool,
    manager_seat_valid: bool,
    club_ptr_valid: bool,
    player_key: u32,
    comparator_key: u32,
    param_3: u32,
    role: i8,
    role_used: i8,
    derived_byte: i8,
    offer_key: u32,
    comparator_club_key: u32,
) -> Option<ContractNegotiationView> {
    if !manager_seat_valid || !club_ptr_valid { return None; }
    if !registration_ok { return None; }
    // Comparator forces param_3 = 1 (exe: `if (param_2 != NULL) param_3 = 1;`).
    let flag_3 = if comparator_key != 0 { 1 } else { param_3 };
    Some(ContractNegotiationView {
        player_key,
        role,
        role_dup: role_used,
        comparator_key,
        flag_3,
        derived_byte,
        offer_key,
        comparator_club_key,
    })
}

// =====================================================================
// 0x004E4580 — Contract-Offer screen (self)
// =====================================================================

/// Fixed "transfer-window / years" constant pushed at slots 0xA and 0xC.
pub const OFFER_YEARS_CONST: u32 = 0x76c;

/// Contract-Offer screen — all deterministic slot writes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractOfferView {
    /// Slot 0: player key (`*param_1`).
    pub player_key: u32,
    /// Slot 1: `param_4` short.
    pub short_flag: i16,
    /// Slot 8: `param_3` low byte.
    pub kind_byte: i8,
    /// Slot 0xA/0xC: constant `0x76c`.
    pub years_const_a: u32,
    pub years_const_b: u32,
    /// Slot 0xD: `param_2` (mode byte).
    pub mode_byte: i8,
    /// Slot 0x15: `FUN_005829c0` result clamped to [0, 5_000_000].
    pub estimate_clamped: u32,
    /// Slot 0x17: `FUN_0043ffd0(4)` wage estimate; 0 when player has
    /// no current contract (`player+0x61 == 0`).
    pub wage_estimate: u32,
    /// Slot 0x1B: club rec's `+0x45` (0 when unresolved).
    pub club_extra: u32,
    /// Slot 0x1C: constant `3`.
    pub sentinel_1c: u32,
}

/// Clamp helper matching the exe's guard:
/// `if (est < 0) est = 0; else if (est > 5_000_000) est = 0x004c4b40;`
pub fn clamp_offer_estimate(raw: i64) -> u32 {
    if raw < 0 { 0 } else if raw > 5_000_000 { 0x004c4b40 } else { raw as u32 }
}

/// Direct port of `FUN_004e4580(param_1, param_2, param_3, param_4)`.
pub fn build_contract_offer(
    registration_ok: bool,
    manager_seat_valid: bool,
    club_ptr_valid: bool,
    player_key: u32,
    mode_byte: i8,
    kind_byte: i8,
    short_flag: i16,
    raw_estimate: i64,
    wage_estimate: u32,
    club_extra: u32,
) -> Option<ContractOfferView> {
    if !manager_seat_valid || !club_ptr_valid { return None; }
    if !registration_ok { return None; }
    Some(ContractOfferView {
        player_key,
        short_flag,
        kind_byte,
        years_const_a: OFFER_YEARS_CONST,
        years_const_b: OFFER_YEARS_CONST,
        mode_byte,
        estimate_clamped: clamp_offer_estimate(raw_estimate),
        wage_estimate,
        club_extra,
        sentinel_1c: 3,
    })
}

// =====================================================================
// 0x004E4960 — Contract-Offer-with-Comparator screen
// =====================================================================

/// Contract-Offer screen with a comparator player (`param_1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractOfferComparatorView {
    /// Slot 0: player key (`*param_2`).
    pub player_key: u32,
    /// Slot 1: `param_5` short.
    pub short_flag: i16,
    /// Slot 3: comparator ptr key (`param_1`) — 0 means "no comparator"
    /// which flips the seat/dup to (0,0)/uVar6=7 in the exe.
    pub comparator_key: u32,
    /// Slot 4: comparator's `+0x20` (0 when absent).
    pub comparator_field_20: u32,
    /// Slot 5: comparator's `+0x24` (0 when absent).
    pub comparator_field_24: u32,
    /// Slot 6: 0/1 — comparator differs from active manager's club.
    pub differs_from_own_club: u8,
    /// Slot 7: comparator's `+0x2d` byte (0 when absent).
    pub comparator_field_2d: i8,
    /// Slot 8: `param_4` low byte.
    pub kind_byte: i8,
    /// Slot 0xA/0xC: constant `0x76c`.
    pub years_const_a: u32,
    pub years_const_b: u32,
    /// Slot 0xD: `param_3` low byte.
    pub mode_byte: i8,
    /// Slot 0x15: comparator-adjusted, clamped estimate.
    pub estimate_clamped: u32,
    /// Slot 0x16: comparator scoring result (0 when player has no
    /// current contract).
    pub compare_score: u32,
    /// Slot 0x17: wage estimate.
    pub wage_estimate: u32,
    /// Slot 0x1B: player's club rec `+0x45` (0 when unresolved).
    pub club_extra: u32,
    /// Slot 0x1C: 3 when the "shortlist gate" fires, else 0.
    pub sentinel_1c: u32,
    /// Slot 0x1D: 1 when the follow-up fallback fires, else 0.
    pub sentinel_1d: u32,
}

/// Result of the exe's comparator adjustment on the raw estimate.
/// exe: `est -= *(iComparator + 0x1b)` before the clamp.
pub fn adjust_and_clamp_comparator_estimate(raw: i64, comparator_delta: i32) -> u32 {
    let adj = raw.saturating_sub(comparator_delta as i64);
    clamp_offer_estimate(adj)
}

/// Direct port of `FUN_004e4960(param_1, param_2, param_3, param_4, param_5)`.
pub fn build_contract_offer_with_comparator(
    registration_ok: bool,
    manager_seat_valid: bool,
    club_ptr_valid: bool,
    player_key: u32,
    comparator_key: u32,
    comparator_fields: Option<(u32, u32, i8, u32, i32)>, // (+0x20, +0x24, +0x2d, owning_club, +0x1b delta)
    active_manager_club: u32,
    mode_byte: i8,
    kind_byte: i8,
    short_flag: i16,
    raw_estimate: i64,
    compare_score: u32,
    wage_estimate: u32,
    club_extra: u32,
    shortlist_gate: bool,
    fallback_gate: bool,
) -> Option<ContractOfferComparatorView> {
    if !manager_seat_valid || !club_ptr_valid { return None; }
    if !registration_ok { return None; }
    let (f20, f24, f2d, owning_club, delta) =
        comparator_fields.unwrap_or((0, 0, 0, 0, 0));
    // exe: slot 6 = 0 when comparator absent, else 0/1 based on
    // comparator's owning club vs active manager's club.
    let differs = if comparator_key == 0 { 0 }
        else if owning_club == active_manager_club { 0 } else { 1 };
    Some(ContractOfferComparatorView {
        player_key,
        short_flag,
        comparator_key,
        comparator_field_20: f20,
        comparator_field_24: f24,
        differs_from_own_club: differs,
        comparator_field_2d: f2d,
        kind_byte,
        years_const_a: OFFER_YEARS_CONST,
        years_const_b: OFFER_YEARS_CONST,
        mode_byte,
        estimate_clamped: adjust_and_clamp_comparator_estimate(raw_estimate, delta),
        compare_score,
        wage_estimate,
        club_extra,
        sentinel_1c: if shortlist_gate { 3 } else { 0 },
        sentinel_1d: if fallback_gate { 1 } else { 0 },
    })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- 0x004A17F0 --------------------------------------------------

    #[test]
    fn comp_lookup_zero_param_returns_none() {
        assert!(build_comp_lookup(true, 0, (0, 0, 0), 0).is_none());
    }
    #[test]
    fn comp_lookup_small_magnitude_returns_none() {
        // exe gate: |param_1| must exceed 9999.
        assert!(build_comp_lookup(true, 42, (1, 2, 3), 99).is_none());
        assert!(build_comp_lookup(true, 9999, (1, 2, 3), 99).is_none());
    }
    #[test]
    fn comp_lookup_all_zero_lookup_returns_none() {
        assert!(build_comp_lookup(true, 12000, (0, 0, 0), 0).is_none());
    }
    #[test]
    fn comp_lookup_carries_slots() {
        let v = build_comp_lookup(true, 12345, (1, 22, 33), 0xdead).unwrap();
        assert_eq!(v.resolved_key, 0xdead);
        assert_eq!(v.extra_a, 22);
        assert_eq!(v.extra_b, 33);
    }
    #[test]
    fn comp_lookup_registration_failure_returns_none() {
        assert!(build_comp_lookup(false, 12345, (1, 22, 33), 0xdead).is_none());
    }

    // ----- 0x004E2B80 --------------------------------------------------

    #[test]
    fn two_key_lookup_carries_all_four_slots() {
        let v = build_two_key_lookup(true, 10, 20, 30, 40).unwrap();
        assert_eq!(v.param_1, 10);
        assert_eq!(v.param_2, 20);
        assert_eq!(v.extra_0, 30);
        assert_eq!(v.extra_2, 40);
    }
    #[test]
    fn two_key_lookup_registration_failure_returns_none() {
        assert!(build_two_key_lookup(false, 1, 2, 3, 4).is_none());
    }

    // ----- 0x004E3300 --------------------------------------------------

    #[test]
    fn contract_neg_no_manager_seat_returns_none() {
        let v = build_contract_negotiation(true, false, true, 1, 0, 0, 0, 0, 0, 0, 0);
        assert!(v.is_none());
    }
    #[test]
    fn contract_neg_no_club_ptr_returns_none() {
        let v = build_contract_negotiation(true, true, false, 1, 0, 0, 0, 0, 0, 0, 0);
        assert!(v.is_none());
    }
    #[test]
    fn contract_neg_comparator_forces_flag_3_to_1() {
        let v = build_contract_negotiation(true, true, true, 10, 999, 0, 2, 2, 5, 42, 77)
            .unwrap();
        assert_eq!(v.flag_3, 1);
        assert_eq!(v.player_key, 10);
        assert_eq!(v.comparator_key, 999);
        assert_eq!(v.role, 2);
        assert_eq!(v.role_dup, 2);
        assert_eq!(v.derived_byte, 5);
        assert_eq!(v.offer_key, 42);
        assert_eq!(v.comparator_club_key, 77);
    }
    #[test]
    fn contract_neg_no_comparator_preserves_param_3() {
        let v = build_contract_negotiation(true, true, true, 10, 0, 0, 0, 0, 0, 0, 0)
            .unwrap();
        assert_eq!(v.flag_3, 0);
        let v2 = build_contract_negotiation(true, true, true, 10, 0, 7, 0, 0, 0, 0, 0)
            .unwrap();
        assert_eq!(v2.flag_3, 7);
    }

    // ----- 0x004E4580 --------------------------------------------------

    #[test]
    fn clamp_offer_estimate_matches_exe_branches() {
        assert_eq!(clamp_offer_estimate(-1), 0);
        assert_eq!(clamp_offer_estimate(0), 0);
        assert_eq!(clamp_offer_estimate(5_000_000), 5_000_000);
        assert_eq!(clamp_offer_estimate(5_000_001), 0x004c4b40);
        assert_eq!(clamp_offer_estimate(1_234_567), 1_234_567);
    }
    #[test]
    fn contract_offer_carries_all_deterministic_slots() {
        let v = build_contract_offer(true, true, true, 55, 3, 11, 1, 800_000, 1500, 0xdead)
            .unwrap();
        assert_eq!(v.player_key, 55);
        assert_eq!(v.mode_byte, 3);
        assert_eq!(v.kind_byte, 11);
        assert_eq!(v.short_flag, 1);
        assert_eq!(v.years_const_a, OFFER_YEARS_CONST);
        assert_eq!(v.years_const_b, OFFER_YEARS_CONST);
        assert_eq!(v.estimate_clamped, 800_000);
        assert_eq!(v.wage_estimate, 1500);
        assert_eq!(v.club_extra, 0xdead);
        assert_eq!(v.sentinel_1c, 3);
    }
    #[test]
    fn contract_offer_gates_return_none() {
        assert!(build_contract_offer(false, true, true, 1, 0, 0, 0, 0, 0, 0).is_none());
        assert!(build_contract_offer(true, false, true, 1, 0, 0, 0, 0, 0, 0).is_none());
        assert!(build_contract_offer(true, true, false, 1, 0, 0, 0, 0, 0, 0).is_none());
    }

    // ----- 0x004E4960 --------------------------------------------------

    #[test]
    fn comparator_estimate_subtracts_delta_then_clamps() {
        assert_eq!(adjust_and_clamp_comparator_estimate(1_000_000, 200_000), 800_000);
        assert_eq!(adjust_and_clamp_comparator_estimate(100, 200), 0);
        assert_eq!(
            adjust_and_clamp_comparator_estimate(10_000_000, 0),
            0x004c4b40
        );
    }
    #[test]
    fn comparator_offer_absent_comparator_forces_differs_zero() {
        let v = build_contract_offer_with_comparator(
            true, true, true, 55, 0, None, 42, 1, 2, 3,
            500_000, 0, 1500, 0xdead, false, false,
        ).unwrap();
        assert_eq!(v.differs_from_own_club, 0);
        assert_eq!(v.comparator_key, 0);
        assert_eq!(v.comparator_field_2d, 0);
        assert_eq!(v.estimate_clamped, 500_000);
        assert_eq!(v.sentinel_1c, 0);
        assert_eq!(v.sentinel_1d, 0);
    }
    #[test]
    fn comparator_offer_same_owning_club_reads_as_zero_diff() {
        let v = build_contract_offer_with_comparator(
            true, true, true, 55, 999,
            Some((11, 22, 4, 42, 100_000)),
            42, // active manager's club = comparator's owning_club
            1, 2, 3,
            600_000, 0xdada, 1500, 0xdead,
            true, true,
        ).unwrap();
        assert_eq!(v.differs_from_own_club, 0);
        assert_eq!(v.comparator_field_20, 11);
        assert_eq!(v.comparator_field_24, 22);
        assert_eq!(v.comparator_field_2d, 4);
        assert_eq!(v.estimate_clamped, 500_000);
        assert_eq!(v.compare_score, 0xdada);
        assert_eq!(v.sentinel_1c, 3);
        assert_eq!(v.sentinel_1d, 1);
    }
    #[test]
    fn comparator_offer_diff_club_flags_one() {
        let v = build_contract_offer_with_comparator(
            true, true, true, 55, 999,
            Some((0, 0, 0, 42, 0)),
            77, // different from comparator owning_club 42
            1, 2, 3,
            600_000, 0, 1500, 0xdead,
            false, false,
        ).unwrap();
        assert_eq!(v.differs_from_own_club, 1);
    }
    #[test]
    fn comparator_offer_gate_failures_return_none() {
        assert!(build_contract_offer_with_comparator(
            false, true, true, 1, 0, None, 0, 0, 0, 0, 0, 0, 0, 0, false, false,
        ).is_none());
        assert!(build_contract_offer_with_comparator(
            true, false, true, 1, 0, None, 0, 0, 0, 0, 0, 0, 0, 0, false, false,
        ).is_none());
        assert!(build_contract_offer_with_comparator(
            true, true, false, 1, 0, None, 0, 0, 0, 0, 0, 0, 0, 0, false, false,
        ).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b13: CompLookupView reads handles {b13.comp.param_1, b13.comp.i_var1, \
b13.comp.local_20c, b13.comp.local_208, b13.comp.resolved_key}; \
TwoKeyLookupView reads handles {b13.two_key.param_1, param_2, extra_0, extra_2}; \
ContractNegotiationView reads flags {b13.negot.manager_seat_valid, \
b13.negot.club_ptr_valid} + handles {b13.negot.player_key, comparator_key, \
param_3, offer_key, comparator_club_key} + bytes {b13.negot.role, role_used, \
derived_byte}; ContractOfferView reads flags {b13.offer.manager_seat_valid, \
b13.offer.club_ptr_valid} + handles {b13.offer.player_key, wage_estimate, club_extra} \
+ bytes {b13.offer.mode_byte, kind_byte, short_flag, raw_estimate_lo, raw_estimate_hi}; \
ContractOfferComparatorView adds comparator handles (b13.offerc.*).\n\
UNKNOWNS: wave-B typed pools not yet available; comparator fields tuple stays \
partially opaque until the transfer/contract-manager typed accessors land.";

use crate::world_facade::WorldFacade;

/// Populator for [`CompLookupView`].
pub fn populate_comp_lookup(world: &WorldFacade) -> Option<CompLookupView> {
    build_comp_lookup(
        world.registration_ok,
        world.handle("b13.comp.param_1"),
        (
            world.byte("b13.comp.i_var1") as i32,
            world.handle("b13.comp.local_20c"),
            world.handle("b13.comp.local_208"),
        ),
        world.handle("b13.comp.resolved_key"),
    )
}

/// Populator for [`TwoKeyLookupView`].
pub fn populate_two_key_lookup(world: &WorldFacade) -> Option<TwoKeyLookupView> {
    build_two_key_lookup(
        world.registration_ok,
        world.handle("b13.two_key.param_1"),
        world.handle("b13.two_key.param_2"),
        world.handle("b13.two_key.extra_0"),
        world.handle("b13.two_key.extra_2"),
    )
}

/// Populator for [`ContractNegotiationView`].
pub fn populate_contract_negotiation(world: &WorldFacade) -> Option<ContractNegotiationView> {
    build_contract_negotiation(
        world.registration_ok,
        world.flag("b13.negot.manager_seat_valid"),
        world.flag("b13.negot.club_ptr_valid"),
        world.handle("b13.negot.player_key"),
        world.handle("b13.negot.comparator_key"),
        world.handle("b13.negot.param_3"),
        world.byte("b13.negot.role") as i8,
        world.byte("b13.negot.role_used") as i8,
        world.byte("b13.negot.derived_byte") as i8,
        world.handle("b13.negot.offer_key"),
        world.handle("b13.negot.comparator_club_key"),
    )
}

/// Populator for [`ContractOfferView`] (batch 13 flavour).
pub fn populate_contract_offer(world: &WorldFacade) -> Option<ContractOfferView> {
    let hi = world.handle("b13.offer.raw_estimate_hi") as i64;
    let lo = world.handle("b13.offer.raw_estimate_lo") as i64;
    let raw = (hi << 32) | (lo & 0xFFFF_FFFF);
    build_contract_offer(
        world.registration_ok,
        world.flag("b13.offer.manager_seat_valid"),
        world.flag("b13.offer.club_ptr_valid"),
        world.handle("b13.offer.player_key"),
        world.byte("b13.offer.mode_byte") as i8,
        world.byte("b13.offer.kind_byte") as i8,
        world.byte("b13.offer.short_flag") as i16,
        raw,
        world.handle("b13.offer.wage_estimate"),
        world.handle("b13.offer.club_extra"),
    )
}

/// Populator for [`ContractOfferComparatorView`].
pub fn populate_contract_offer_with_comparator(
    world: &WorldFacade,
) -> Option<ContractOfferComparatorView> {
    let hi = world.handle("b13.offerc.raw_estimate_hi") as i64;
    let lo = world.handle("b13.offerc.raw_estimate_lo") as i64;
    let raw = (hi << 32) | (lo & 0xFFFF_FFFF);
    let comparator_fields = if world.flag("b13.offerc.comparator_fields_present") {
        Some((
            world.handle("b13.offerc.cmp_f20"),
            world.handle("b13.offerc.cmp_f24"),
            world.byte("b13.offerc.cmp_f2d") as i8,
            world.handle("b13.offerc.cmp_owning_club"),
            world.byte("b13.offerc.cmp_delta") as i32,
        ))
    } else {
        None
    };
    build_contract_offer_with_comparator(
        world.registration_ok,
        world.flag("b13.offerc.manager_seat_valid"),
        world.flag("b13.offerc.club_ptr_valid"),
        world.handle("b13.offerc.player_key"),
        world.handle("b13.offerc.comparator_key"),
        comparator_fields,
        world.handle("b13.offerc.active_manager_club"),
        world.byte("b13.offerc.mode_byte") as i8,
        world.byte("b13.offerc.kind_byte") as i8,
        world.byte("b13.offerc.short_flag") as i16,
        raw,
        world.handle("b13.offerc.compare_score"),
        world.handle("b13.offerc.wage_estimate"),
        world.handle("b13.offerc.club_extra"),
        world.flag("b13.offerc.shortlist_gate"),
        world.flag("b13.offerc.fallback_gate"),
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_comp_lookup_requires_param1_and_magnitude() {
        assert!(populate_comp_lookup(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready()
            .with_handle("b13.comp.param_1", 20000)
            .with_handle("b13.comp.local_20c", 1)
            .with_handle("b13.comp.resolved_key", 42);
        let v = populate_comp_lookup(&w).unwrap();
        assert_eq!(v.resolved_key, 42);
    }
    #[test]
    fn populate_two_key_lookup_ok() {
        assert!(populate_two_key_lookup(&WorldFacade::ready()).is_some());
    }
    #[test]
    fn populate_contract_negotiation_requires_gates() {
        assert!(populate_contract_negotiation(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready()
            .with_flag("b13.negot.manager_seat_valid", true)
            .with_flag("b13.negot.club_ptr_valid", true)
            .with_handle("b13.negot.comparator_key", 5);
        let v = populate_contract_negotiation(&w).unwrap();
        assert_eq!(v.flag_3, 1);
    }
    #[test]
    fn populate_contract_offer_requires_gates() {
        let w = WorldFacade::ready()
            .with_flag("b13.offer.manager_seat_valid", true)
            .with_flag("b13.offer.club_ptr_valid", true);
        assert!(populate_contract_offer(&w).is_some());
    }
    #[test]
    fn populate_contract_offer_with_comparator_ok() {
        let w = WorldFacade::ready()
            .with_flag("b13.offerc.manager_seat_valid", true)
            .with_flag("b13.offerc.club_ptr_valid", true);
        assert!(populate_contract_offer_with_comparator(&w).is_some());
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A pattern)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b13 pool sources: CompLookup requires |param_1|>9999 and a resolved \
    key from FUN_004db... lookup tables (opaque). ContractNegotiation \
    needs manager_seat_valid + club_ptr_valid; both derived from a \
    valid active human seat. Wage/estimate figures come from the \
    contract-value calculator (float-heavy).";

use crate::world_pools::WorldPools;

pub fn populate_comp_lookup_from_pools(_pools: &WorldPools<'_>) -> Option<CompLookupView> {
    // The exe requires |param_1| > 9999 and a non-zero lookup key; the
    // pools facade doesn't carry those, so this returns None until a
    // real comp-lookup handler is wired.
    build_comp_lookup(true, 0, (0, 0, 0), 0)
}

pub fn populate_two_key_lookup_from_pools(
    pools: &WorldPools<'_>,
) -> Option<TwoKeyLookupView> {
    let seat = pools.active_human_seat.unwrap_or(0);
    build_two_key_lookup(true, seat, 0, 0, 0)
}

pub fn populate_contract_negotiation_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ContractNegotiationView> {
    let seat_valid = pools.active_human_seat.is_some();
    let club_valid = pools.active_human_club_id().is_some();
    build_contract_negotiation(true, seat_valid, club_valid,
        0, 0, 0, 0, 0, 0, 0, 0)
}

pub fn populate_contract_offer_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ContractOfferView> {
    let seat_valid = pools.active_human_seat.is_some();
    let club_valid = pools.active_human_club_id().is_some();
    build_contract_offer(true, seat_valid, club_valid,
        0, 0, 0, 0, 0, 0, 0)
}

pub fn populate_contract_offer_with_comparator_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ContractOfferComparatorView> {
    let seat_valid = pools.active_human_seat.is_some();
    let club_valid = pools.active_human_club_id().is_some();
    let active_club = pools.active_human_club_id().unwrap_or(0);
    build_contract_offer_with_comparator(true, seat_valid, club_valid,
        0, 0, None, active_club, 0, 0, 0, 0, 0, 0, 0, false, false)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn empty_pools_return_none() {
        let p = WorldPools::empty();
        assert!(populate_comp_lookup_from_pools(&p).is_none());
        assert!(populate_contract_negotiation_from_pools(&p).is_none());
        assert!(populate_contract_offer_from_pools(&p).is_none());
        assert!(populate_contract_offer_with_comparator_from_pools(&p).is_none());
        // two_key_lookup has no non-zero gate.
        assert!(populate_two_key_lookup_from_pools(&p).is_some());
    }
}
