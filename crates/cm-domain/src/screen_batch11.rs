//! Batch 11: 5 more setup functions in the club/offer screen family.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `00475870.c` — Club sub-screen A (3 slots, copies 0xd1-byte record)
//! * `004776a0.c` — Club/competition setup B (3 slots, computed payload)
//! * `00477dd0.c` — Club/competition setup C (4 slots)
//! * `00478580.c` — Contract/offer screen (19 slots, per-seat)
//! * `004787f0.c` — Contract/offer setup direct (19 slots)

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_00475870 — 3-slot club sub-screen; copies a 0xd1-byte snapshot
// of `param_1`'s record for slot 2. Bails with a MsgBox when
// `param_1 == 0` (also stores 0 into DAT_00b4d5a8) instead of calling
// FUN_007e6570 at all.
// =====================================================================

/// Slot 0=club ptr, slot 1=aux param, slot 2=owned 0xd1 (209) byte copy
/// of the club record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubSubScreenAView {
    pub club_ptr: u32,
    pub aux_param: u32,
    /// Fresh 209-byte copy of the record at `club_ptr`.
    pub record_copy: Vec<u8>,
}

/// Direct port of `FUN_00475870(param_1, param_2)`.
///
/// * `record` is what `param_1` points at (must be at least 0xd1 bytes).
/// * Returns `None` when `record` is empty (matches the null-`param_1`
///   MsgBox branch) or when registration fails.
pub fn build_club_sub_screen_a(
    registration_ok: bool,
    club_ptr: u32,
    aux_param: u32,
    record: &[u8],
) -> Option<ClubSubScreenAView> {
    if record.is_empty() { return None; }        // exe: param_1 == 0
    if !registration_ok { return None; }
    let take = record.len().min(0xd1);
    let mut copy = vec![0u8; 0xd1];
    copy[..take].copy_from_slice(&record[..take]);
    Some(ClubSubScreenAView { club_ptr, aux_param, record_copy: copy })
}

// =====================================================================
// FUN_004776a0 — 3-slot setup, payload built from cmd-line args:
//   arg2 → club-record ptr (base + arg*0x245), arg3+arg4 → FUN_00533ad0,
//   then FUN_005b2b90(uVar2, iVar1). Bails to MsgBox if FUN_0076eb10 == 0.
// =====================================================================

/// 3-slot view — the exe records only the caller args + the computed
/// payload handle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ClubCompSetupBView {
    pub param_1: u32,
    pub param_2: u32,
    /// Slot 2: `FUN_005b2b90(FUN_00533ad0(arg3, arg4), club_record_ptr)`.
    pub computed_payload: u32,
}

/// Direct port of `FUN_004776a0(param_1, param_2)`.
///
/// * `args_ok` matches `FUN_0076eb10(...) != 0` (arg parse).
/// * `computed_payload` is the caller's precomputed result — we hoist
///   the FUN_00533ad0 / FUN_005b2b90 chain to the caller until those
///   are ported.
pub fn build_club_comp_setup_b(
    registration_ok: bool,
    args_ok: bool,
    param_1: u32,
    param_2: u32,
    computed_payload: u32,
) -> Option<ClubCompSetupBView> {
    if !args_ok { return None; }
    if !registration_ok { return None; }
    Some(ClubCompSetupBView { param_1, param_2, computed_payload })
}

// =====================================================================
// FUN_00477dd0 — 4 slots.
//   arg1 → club-record ptr, arg2 → FUN_005b2c40(uVar1, 0).
// =====================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ClubCompSetupCView {
    pub param_1: u32,
    pub param_2: u32,
    /// Slot 2: `FUN_005b2c40(arg2, 0)` — caller-computed.
    pub payload: u32,
    /// Slot 3: `base + arg1 * 0x245`, or 0 when arg1 < 0.
    pub club_record_ptr: u32,
}

/// Direct port of `FUN_00477dd0(param_1, param_2)`.
pub fn build_club_comp_setup_c(
    registration_ok: bool,
    args_ok: bool,
    param_1: u32,
    param_2: u32,
    payload: u32,
    club_record_ptr: u32,
) -> Option<ClubCompSetupCView> {
    if !args_ok { return None; }
    if !registration_ok { return None; }
    Some(ClubCompSetupCView { param_1, param_2, payload, club_record_ptr })
}

// =====================================================================
// FUN_00478580 — Contract/offer screen (per-seat entry point), 19 slots.
// Registers callbacks FUN_00478ca0 / FUN_0047ea60 / LAB_004800f0.
// Loops seats until the first eligible one, then pushes:
//   0: active human's person ptr ((&DAT_00b59fc2)[DAT_00b5d016 * 0xc0])
//   1: -1, 2: 1, 3: 1, 4: 0x41, 5: 8, 6: 6,
//   7..0x11: 0 (with 9,10 = -1), 0x12: iVar1 (target club record)
// =====================================================================

/// Contract/offer view — the 19-slot payload the exe pushes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractOfferView {
    pub active_person: u32,        // slot 0
    pub mode_a: i32,               // slot 1 = -1
    pub mode_b: u32,               // slot 2 = 1
    pub mode_c: u32,               // slot 3 = 1
    pub field_4: u32,              // slot 4 = 0x41 ('A')
    pub field_5: u32,              // slot 5 = 8
    pub field_6: u32,              // slot 6 = 6
    /// Slots 7,8,0xb..=0x11 (all zero); 9,10 = -1.
    pub reserved: [i32; 12],
    pub target_club_record: u32,   // slot 0x12
}

impl ContractOfferView {
    fn defaults_with(active_person: u32, target_club_record: u32) -> Self {
        // reserved indices map to slots: 7,8,9,10,0xb..0x11 (12 entries).
        let mut reserved = [0i32; 12];
        reserved[2] = -1;   // slot 9
        reserved[3] = -1;   // slot 10
        Self {
            active_person,
            mode_a: -1,
            mode_b: 1,
            mode_c: 1,
            field_4: 0x41,
            field_5: 8,
            field_6: 6,
            reserved,
            target_club_record,
        }
    }
}

/// Direct port of `FUN_00478580(param_1)`.
///
/// The eligibility loop is caller-side: pass `seat_eligible` = whether
/// any seat passed the exe's cascading gates (`FUN_00843ef0`, seat
/// filters, `FUN_008815a0`, `FUN_00525450`, `FUN_00843880`, etc.). When
/// eligible, `target_club_record` is the `iVar1` value (person's current
/// club record).
pub fn build_contract_offer(
    registration_ok: bool,
    seat_eligible: bool,
    active_person: u32,
    target_club_record: u32,
) -> Option<ContractOfferView> {
    if !seat_eligible { return None; }        // exe returns 0
    if !registration_ok { return None; }
    Some(ContractOfferView::defaults_with(active_person, target_club_record))
}

// =====================================================================
// FUN_004787f0 — Contract/offer direct entry (arg-driven), 19 slots.
// Same callbacks as FUN_00478580. Bails when FUN_0076eb10 == 0.
// Slots differ from FUN_00478580: slot 0/1 = raw args, slot 3 =
// (local_2ec[0] == 0x18), slot 0x12 = 0.
// =====================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractOfferDirectView {
    pub param_1: u32,          // slot 0
    pub param_2: u32,          // slot 1
    pub mode_b: u32,           // slot 2 = 1
    pub is_18_flag: bool,      // slot 3 = (parsed[0] == 0x18)
    pub field_4: u32,          // slot 4 = 0x41
    pub field_5: u32,          // slot 5 = 8
    pub field_6: u32,          // slot 6 = 6
    /// Slots 7,8,0xb..=0x11 (12 entries); 9,10 = -1.
    pub reserved: [i32; 12],
    pub slot_12: u32,          // slot 0x12 = 0
}

impl ContractOfferDirectView {
    fn defaults_with(param_1: u32, param_2: u32, is_18: bool) -> Self {
        let mut reserved = [0i32; 12];
        reserved[2] = -1;
        reserved[3] = -1;
        Self {
            param_1, param_2, mode_b: 1, is_18_flag: is_18,
            field_4: 0x41, field_5: 8, field_6: 6,
            reserved, slot_12: 0,
        }
    }
}

/// Direct port of `FUN_004787f0(param_1, param_2)`.
///
/// * `args_ok` matches `FUN_0076eb10(...) != 0`.
/// * `parsed_first_word` is `local_2ec[0]` — the exe compares it to
///   `0x18` and pushes the resulting bool into slot 3.
pub fn build_contract_offer_direct(
    registration_ok: bool,
    args_ok: bool,
    param_1: u32,
    param_2: u32,
    parsed_first_word: u32,
) -> Option<ContractOfferDirectView> {
    if !args_ok { return None; }
    if !registration_ok { return None; }
    Some(ContractOfferDirectView::defaults_with(
        param_1, param_2, parsed_first_word == 0x18,
    ))
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn club_sub_screen_a_copies_full_209_bytes() {
        let mut rec = vec![0u8; 0xd1];
        rec[0] = 0xAB; rec[0xd0] = 0xCD;
        let v = build_club_sub_screen_a(true, 0x1000, 7, &rec).unwrap();
        assert_eq!(v.club_ptr, 0x1000);
        assert_eq!(v.aux_param, 7);
        assert_eq!(v.record_copy.len(), 0xd1);
        assert_eq!(v.record_copy[0], 0xAB);
        assert_eq!(v.record_copy[0xd0], 0xCD);
    }
    #[test]
    fn club_sub_screen_a_null_ptr_returns_none() {
        assert!(build_club_sub_screen_a(true, 0, 0, &[]).is_none());
    }
    #[test]
    fn club_sub_screen_a_failed_registration_returns_none() {
        assert!(build_club_sub_screen_a(false, 1, 0, &[0u8; 0xd1]).is_none());
    }

    #[test]
    fn club_comp_setup_b_carries_all_three_slots() {
        let v = build_club_comp_setup_b(true, true, 0x11, 0x22, 0xDEAD).unwrap();
        assert_eq!(v.param_1, 0x11);
        assert_eq!(v.param_2, 0x22);
        assert_eq!(v.computed_payload, 0xDEAD);
    }
    #[test]
    fn club_comp_setup_b_bad_args_returns_none() {
        assert!(build_club_comp_setup_b(true, false, 0, 0, 0).is_none());
    }

    #[test]
    fn club_comp_setup_c_carries_all_four_slots() {
        let v = build_club_comp_setup_c(true, true, 1, 2, 3, 0xBEEF).unwrap();
        assert_eq!(v.param_1, 1);
        assert_eq!(v.param_2, 2);
        assert_eq!(v.payload, 3);
        assert_eq!(v.club_record_ptr, 0xBEEF);
    }
    #[test]
    fn club_comp_setup_c_bad_args_returns_none() {
        assert!(build_club_comp_setup_c(true, false, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn contract_offer_pushes_exact_fixed_values() {
        let v = build_contract_offer(true, true, 0xAAAA, 0xBBBB).unwrap();
        assert_eq!(v.active_person, 0xAAAA);
        assert_eq!(v.mode_a, -1);
        assert_eq!(v.mode_b, 1);
        assert_eq!(v.mode_c, 1);
        assert_eq!(v.field_4, 0x41);
        assert_eq!(v.field_5, 8);
        assert_eq!(v.field_6, 6);
        // slot 9 / 10 are -1 in the exe; others zero.
        assert_eq!(v.reserved[2], -1);
        assert_eq!(v.reserved[3], -1);
        assert_eq!(v.reserved[0], 0);
        assert_eq!(v.reserved[11], 0);
        assert_eq!(v.target_club_record, 0xBBBB);
    }
    #[test]
    fn contract_offer_no_eligible_seat_returns_none() {
        assert!(build_contract_offer(true, false, 0, 0).is_none());
    }
    #[test]
    fn contract_offer_failed_registration_returns_none() {
        assert!(build_contract_offer(false, true, 1, 2).is_none());
    }

    #[test]
    fn contract_offer_direct_slot_3_true_when_first_word_is_0x18() {
        let v = build_contract_offer_direct(true, true, 1, 2, 0x18).unwrap();
        assert!(v.is_18_flag);
        assert_eq!(v.param_1, 1);
        assert_eq!(v.param_2, 2);
        assert_eq!(v.field_4, 0x41);
        assert_eq!(v.slot_12, 0);
    }
    #[test]
    fn contract_offer_direct_slot_3_false_otherwise() {
        let v = build_contract_offer_direct(true, true, 0, 0, 0x17).unwrap();
        assert!(!v.is_18_flag);
    }
    #[test]
    fn contract_offer_direct_bad_args_returns_none() {
        assert!(build_contract_offer_direct(true, false, 0, 0, 0).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b11: ClubSubScreenAView reads handles {b11.club_ptr, b11.aux_param} + \
buffer {b11.club_sub.record}; ClubCompSetupBView reads flags {b11.setup_b.args_ok} + \
handles {b11.setup_b.param_1, param_2, computed_payload}; ClubCompSetupCView reads \
the same plus {b11.setup_c.club_record_ptr}; ContractOfferView reads flag \
{b11.offer.seat_eligible} + handles {b11.offer.active_person, target_club_record}; \
ContractOfferDirectView reads flag {b11.offer_direct.args_ok} + handles \
{b11.offer_direct.param_1, param_2, parsed_first_word}.\n\
UNKNOWNS: wave-B typed pools not yet available; the 0xd1-byte club record slice \
should ultimately be sourced from world.core.clubs[club_ptr].";

use crate::world_facade::WorldFacade;

/// Populator for [`ClubSubScreenAView`].
pub fn populate_club_sub_screen_a(world: &WorldFacade) -> Option<ClubSubScreenAView> {
    build_club_sub_screen_a(
        world.registration_ok,
        world.handle("b11.club_ptr"),
        world.handle("b11.aux_param"),
        world.buffer("b11.club_sub.record"),
    )
}

/// Populator for [`ClubCompSetupBView`].
pub fn populate_club_comp_setup_b(world: &WorldFacade) -> Option<ClubCompSetupBView> {
    build_club_comp_setup_b(
        world.registration_ok,
        world.flag("b11.setup_b.args_ok"),
        world.handle("b11.setup_b.param_1"),
        world.handle("b11.setup_b.param_2"),
        world.handle("b11.setup_b.computed_payload"),
    )
}

/// Populator for [`ClubCompSetupCView`].
pub fn populate_club_comp_setup_c(world: &WorldFacade) -> Option<ClubCompSetupCView> {
    build_club_comp_setup_c(
        world.registration_ok,
        world.flag("b11.setup_c.args_ok"),
        world.handle("b11.setup_c.param_1"),
        world.handle("b11.setup_c.param_2"),
        world.handle("b11.setup_c.payload"),
        world.handle("b11.setup_c.club_record_ptr"),
    )
}

/// Populator for [`ContractOfferView`].
pub fn populate_contract_offer(world: &WorldFacade) -> Option<ContractOfferView> {
    build_contract_offer(
        world.registration_ok,
        world.flag("b11.offer.seat_eligible"),
        world.handle("b11.offer.active_person"),
        world.handle("b11.offer.target_club_record"),
    )
}

/// Populator for [`ContractOfferDirectView`].
pub fn populate_contract_offer_direct(world: &WorldFacade) -> Option<ContractOfferDirectView> {
    build_contract_offer_direct(
        world.registration_ok,
        world.flag("b11.offer_direct.args_ok"),
        world.handle("b11.offer_direct.param_1"),
        world.handle("b11.offer_direct.param_2"),
        world.handle("b11.offer_direct.parsed_first_word"),
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_club_sub_screen_a_empty_record_returns_none() {
        assert!(populate_club_sub_screen_a(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_club_sub_screen_a_with_record_ok() {
        let w = WorldFacade::ready().with_buffer("b11.club_sub.record", vec![1u8; 4]);
        let v = populate_club_sub_screen_a(&w).unwrap();
        assert_eq!(v.record_copy.len(), 0xd1);
        assert_eq!(v.record_copy[0], 1);
    }
    #[test]
    fn populate_club_comp_setup_b_requires_args_ok() {
        assert!(populate_club_comp_setup_b(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready().with_flag("b11.setup_b.args_ok", true);
        assert!(populate_club_comp_setup_b(&w).is_some());
    }
    #[test]
    fn populate_club_comp_setup_c_requires_args_ok() {
        let w = WorldFacade::ready().with_flag("b11.setup_c.args_ok", true);
        assert!(populate_club_comp_setup_c(&w).is_some());
    }
    #[test]
    fn populate_contract_offer_requires_seat_eligible() {
        assert!(populate_contract_offer(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready().with_flag("b11.offer.seat_eligible", true);
        assert!(populate_contract_offer(&w).is_some());
    }
    #[test]
    fn populate_contract_offer_direct_requires_args_ok() {
        let w = WorldFacade::ready().with_flag("b11.offer_direct.args_ok", true);
        assert!(populate_contract_offer_direct(&w).is_some());
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A pattern)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b11 pool sources: ClubSubScreenA needs a 0xd1-byte club record \
    slice; we take clubs[0].raw when present. ClubCompSetupB/C \
    'computed_payload' comes from FUN_005b2b90/FUN_00533ad0 (opaque). \
    ContractOfferView eligibility loop is per-seat and needs \
    FUN_00843ef0/FUN_00525450/FUN_00843880 gates (not on the facade); \
    we treat 'seat present' as 'eligible' for the default populator. \
    ContractOfferDirect's parsed_first_word comes from a cmd-arg parser.";

use crate::world_pools::WorldPools;

pub fn populate_club_sub_screen_a_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ClubSubScreenAView> {
    // Prefer the active human's current club; else first club in pool.
    let club_id = pools.active_human_club_id().or_else(|| pools.clubs.first().map(|c| c.id))?;
    let rec = pools.clubs.iter().find(|c| c.id == club_id)?;
    if rec.raw.is_empty() { return None; }
    build_club_sub_screen_a(true, club_id, 0, &rec.raw)
}

pub fn populate_club_comp_setup_b_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ClubCompSetupBView> {
    // Requires args_ok. Default: any club present -> args_ok.
    let args_ok = !pools.clubs.is_empty();
    let club = pools.active_human_club_id().unwrap_or(0);
    build_club_comp_setup_b(true, args_ok, club, 0, 0)
}

pub fn populate_club_comp_setup_c_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ClubCompSetupCView> {
    let args_ok = !pools.clubs.is_empty();
    let club = pools.active_human_club_id().unwrap_or(0);
    build_club_comp_setup_c(true, args_ok, club, 0, 0, 0)
}

pub fn populate_contract_offer_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ContractOfferView> {
    let seat = pools.active_human_seat?;
    let club = pools.active_human_club_id().unwrap_or(0);
    build_contract_offer(true, true, seat, club)
}

pub fn populate_contract_offer_direct_from_pools(
    pools: &WorldPools<'_>,
) -> Option<ContractOfferDirectView> {
    let args_ok = pools.active_human_seat.is_some();
    let seat = pools.active_human_seat.unwrap_or(0);
    build_contract_offer_direct(true, args_ok, seat, 0, 0)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn all_return_none_or_ok_on_empty_pools() {
        let p = WorldPools::empty();
        assert!(populate_club_sub_screen_a_from_pools(&p).is_none());
        assert!(populate_club_comp_setup_b_from_pools(&p).is_none()); // args_ok false
        assert!(populate_club_comp_setup_c_from_pools(&p).is_none());
        assert!(populate_contract_offer_from_pools(&p).is_none());
        assert!(populate_contract_offer_direct_from_pools(&p).is_none());
    }

    #[test]
    fn seat_enables_contract_offer() {
        let p = WorldPools { active_human_seat: Some(5), ..WorldPools::empty() };
        let v = populate_contract_offer_from_pools(&p).unwrap();
        assert_eq!(v.active_person, 5);
        let d = populate_contract_offer_direct_from_pools(&p).unwrap();
        assert_eq!(d.param_1, 5);
    }
}
