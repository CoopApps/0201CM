//! C14.5 — comp_s.cpp squad-manager helpers required by C13
//! promotion apply.
//!
//! # C14.6 RECONCILIATION NOTICE (2026-09-15)
//!
//! A follow-up archaeology pass (C14.6) reconciled two contradictory
//! readings of `Club+0x57` and `DAT_00ac688c`:
//!
//! * **Reading A** (`typed_records::ClubView`, 2026-08-20): `Club+0x57`
//!   is a competition pointer (on disk an `i32 division_id`, in memory
//!   a `Comp*` after loader fixup).
//! * **Reading B** (C14.5 initial archaeology): `param_1+0x57` in
//!   `FUN_00843EF0` was inferred to be a person pointer via
//!   `DAT_00ac688c` being labelled a "person pointer table".
//!
//! **Verdict — Reading A is correct.** C14.6 proved
//! `DAT_00ac688c` is the **Comp pointer table** (indexed by comp_id,
//! stride 4). Evidence: 10+ independent call sites use it with
//! comp_id globals (`DAT_009bbb14`, etc.); one site derefs the
//! resulting pointer's `+0xB1` (comp league-table pointer); another
//! calls its `+0x34` vtable method (a Comp virtual). Person records
//! are addressed by a different base (`DAT_00acd5bc`, stride 0x245).
//!
//! **Implications**:
//!
//! * `Club+0x57` is unchanged from C13: primary comp id (on disk) /
//!   `Comp*` (in memory). **C13's port stays correct.**
//! * `Club+0x5B` is confirmed as secondary/previous comp.
//! * `FUN_00843EF0` is really **`is_club_primary_comp_active`** —
//!   it checks the club's primary comp against the active-comp table
//!   and tests the comp's own `+0x52 & 2` "eligible" flag, NOT a
//!   person eligibility bit.
//! * `appointment_transition_register_flag` is renamed conceptually
//!   to `comp_activation_register_flag`: `register_flag = 1` iff
//!   **old comp was inactive AND new comp is active** — a promotion
//!   into a newly-simulated tier — not a person swap.
//! * The `PersonEligibility` name in this module is retained for
//!   type-signature stability, but it now represents a
//!   **`CompEligibility`** record (a `comp_id` and its `+0x52` flag
//!   byte). Semantic name aliases below.
//! * `FUN_00843970` operates on a **club-staff-slot pointer**
//!   (`Club+0xD7 + i*4`), not a `Comp*`. It looks up per-slot
//!   `SquadRecord` pointers via `FUN_004d59d0`/`FUN_004d5b00`
//!   (primary + fallback/reserve) and writes a signed
//!   **position/role preference byte** at `+0x3A` (range
//!   `-50..=50`; the field is a preference weight, not a slot
//!   index or an XI position).
//! * **`DAT_00acdf0c` is NOT the person → squad-slot table**.
//!   C14.6 grep shows it's a per-person **job/interest tracker**
//!   (fields: `+0x04` target-index with `-1` sentinel, `+0x08`
//!   count, `+0x09` 0..100 rating, `+0x0B` bit flags 0x40/0x80).
//!   Boot-allocated dense array; not a lazy structure. The actual
//!   squad-slot lookup goes through `FUN_004d59d0`/`FUN_004d5b00`
//!   from a club-staff slot pointer, not through this DAT. Retract
//!   the C14.5 provisional recommendation to add
//!   `World.squad_registrations`; C15 does not need it.
//! * `SquadRecord+0x3A` is a **signed position/role preference byte**
//!   in the range `[-50, +50]` (verified by 20+ writers doing
//!   negations, increments, and copies from `Person+0x32` = preferred
//!   position field).
//!
//! Byte-level writes and gates in this module are all still
//! correct against the DD source — only the semantic labels have
//! been corrected.
//!
//! Ports two functions from the DirectDraw decompile:
//!
//! * **`FUN_00843EF0`** (`0x00843EF0`, ~33 lines) — pure person-
//!   eligibility probe. Reads a target pointer at
//!   `param_1+0x57`, dereferences it, treats the first int as a
//!   `person_id`, indexes `DAT_00ac688c[person_id]`, and returns
//!   `1` iff `Person+0x52 & 0x02` is set. Optional second-person
//!   check via `param_2`.
//! * **`FUN_00843970`** (`0x00843970`, ~41 lines) — per-person
//!   comp-registration position writer. Given a comp and a
//!   person, looks up the person's two `DAT_00acdf0c` slot
//!   indices, dereferences `*(Comp+0)[slot*0x50]` to reach a
//!   pre-existing `SquadRecord`, verifies identity, and writes
//!   `slot+0x3a = position_code`.
//!
//! # Corrections vs C13 archaeology
//!
//! C13 documented the promotion-install ordering as:
//!
//! ```text
//! FUN_00843EF0(club, 0)   // probe1
//! FUN_004D3550(...)       // walk
//! FUN_00843EF0(club, 0)   // probe2
//! ```
//!
//! The actual sequence in `FUN_00668380` (verified in the DD
//! source, lines 10–22) is:
//!
//! ```text
//! probe1 = FUN_00843EF0(&target, 0)   // reads OLD occupant
//! target[0x5b] = target[0x57]         // save old
//! target[0x57] = new_pointer          // install new
//! if (!probe1) {
//!     probe2 = FUN_00843EF0(&target, 0)   // reads NEW occupant
//!     if (probe2) register_flag = 1
//! }
//! FUN_004D3550(target, register_flag)     // walk fires ONCE
//! ```
//!
//! So `FUN_00843EF0` is pure — no state mutation — and the
//! double-call pattern exists because the caller mutates
//! `target+0x57` BETWEEN the two probes. `register_flag = 1`
//! iff the swap represents an **invalid → valid transition**.
//!
//! # Registration storage
//!
//! Not `Comp+0xB1`. The actual layout:
//!
//! * **`DAT_00acdf0c`** — global person→slot table. Stride
//!   `0x4f` per person. Two `i32` fields at `+0` (primary) and
//!   `+4` (secondary): each is an index into the comp-side
//!   SquadRecord array, or `< 0` for "not registered".
//! * **`*(Comp+0)`** — comp-side SquadRecord array, stride
//!   `0x50`. Fields:
//!     - `+0x04`: `i32 person_id` — occupant.
//!     - `+0x1C`: `char` — flag cleared by C13 promotion walk.
//!     - `+0x1F`: `char` — flag cleared by C13 promotion walk.
//!     - `+0x3A`: `i8 position_code` — written HERE (range −50..=50).
//! * **`Person+0xD7`** — 50-entry array of `Comp*` iterated by
//!   `FUN_004D3550`.
//!
//! # Confidence
//!
//! Both ports: **STRUCTURALLY PORTED** vs the DD decompile.
//! Runtime differential against a captured GDI event would
//! upgrade to STATE-EXACT.

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// `Person+0x52` bit that means "eligible / staff-role installed".
/// Verified in `00843ef0.c:25`.
pub const PERSON_ELIGIBLE_BIT: u8 = 0x02;

/// Valid range for the `position_code` byte at `SquadRecord+0x3A`.
/// Verified in `00843970.c:12`: `-50 <= p3 <= 50`.
pub const POSITION_CODE_MIN: i8 = -50;
pub const POSITION_CODE_MAX: i8 = 50;

// ---------------------------------------------------------------------------
// FUN_00843EF0 port — pure eligibility probe
// ---------------------------------------------------------------------------

/// A person's decision-relevant state, as `FUN_00843EF0` reads it.
///
/// Only two fields matter: the person's id (used to skip
/// `null`/out-of-pool cases) and the `+0x52` flag byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonEligibility {
    pub person_id: u32,
    /// `Person+0x52`. Bit `0x02` is the "eligible" gate.
    pub flags_52: u8,
}

impl PersonEligibility {
    pub fn is_eligible(&self) -> bool {
        (self.flags_52 & PERSON_ELIGIBLE_BIT) != 0
    }
}

/// Pure port of `FUN_00843EF0`.
///
/// Contract (mirrors the DD control flow):
/// * If `current_person` is `None` (slot pointer at `+0x57` was
///   null in the exe): returns `0`.
/// * If current is not eligible: returns `0`.
/// * If `other_person` is `Some(..)` and NOT eligible: returns `0`.
/// * Otherwise returns `1`.
///
/// Returns `u32` to keep the return-type shape identical to the
/// exe (which returns an `int`).
///
/// Pure. Does NOT mutate anything. The `DAT_00b4d5a8 = 0` write
/// visible in the DD source is on the `param_1 == 0`
/// hard-error path, which cannot be reached from Rust (callers
/// only construct the input when `param_1` was non-null in the
/// source domain).
pub fn probe_person_eligibility(
    current_person: Option<&PersonEligibility>,
    other_person: Option<&PersonEligibility>,
) -> u32 {
    let Some(cur) = current_person else { return 0; };
    if !cur.is_eligible() {
        return 0;
    }
    if let Some(other) = other_person {
        if !other.is_eligible() {
            return 0;
        }
    }
    1
}

// ---------------------------------------------------------------------------
// Semantic aliases (C14.6 correction)
// ---------------------------------------------------------------------------

/// C14.6-correct alias for [`PersonEligibility`]. This is really a
/// **`CompEligibility`** record — `person_id` is the comp id (from
/// `Comp+0`) and `flags_52` is the comp's `+0x52 & 2` "active"
/// flag. Kept as a re-export for callers migrating to the corrected
/// semantic name.
pub type CompEligibility = PersonEligibility;

/// C14.6-correct alias for [`probe_person_eligibility`]. What
/// `FUN_00843EF0` actually asks: *is this club's primary
/// competition active in the live comp table, and (optionally) is
/// a second club's primary comp also active?*
pub use self::probe_person_eligibility as probe_club_primary_comp_active;

/// C14.6-correct alias for
/// [`appointment_transition_register_flag`]. What it computes:
/// *did the club just transition FROM an inactive/absent primary
/// comp TO an active one?* — i.e., a promotion into a newly-
/// simulated tier.
pub use self::appointment_transition_register_flag
    as comp_activation_register_flag;

// ---------------------------------------------------------------------------
// FUN_00668380 double-probe wrapper (register_flag computation)
// ---------------------------------------------------------------------------

/// Emit the register-flag value C13's `FUN_004D3550` walk consumes,
/// following `FUN_00668380`'s appointment-swap pattern:
///
/// ```text
/// probe1 = probe(old_person)
/// (caller writes target+0x5b = old, target+0x57 = new)
/// register_flag = probe1 == 0 && probe(new_person) != 0
/// ```
///
/// `register_flag = 1` iff the transition goes from an invalid
/// occupant to a valid one (a fresh appointment). Idempotent
/// swaps (both valid, or both invalid) yield `0`.
///
/// Callers that don't own an appointment-swap use case can call
/// [`probe_person_eligibility`] directly instead of this.
pub fn appointment_transition_register_flag(
    old_person: Option<&PersonEligibility>,
    new_person: Option<&PersonEligibility>,
) -> u8 {
    let probe1 = probe_person_eligibility(old_person, None);
    if probe1 != 0 {
        // Old already valid — the register walk skips.
        return 0;
    }
    let probe2 = probe_person_eligibility(new_person, None);
    if probe2 != 0 { 1 } else { 0 }
}

// ---------------------------------------------------------------------------
// FUN_00843970 port — write position_code into pre-existing squad slot
// ---------------------------------------------------------------------------

/// Snapshot of one SquadRecord slot, resolved by the caller from
/// `DAT_00acdf0c[person_id*0x4f]` (primary) or
/// `DAT_00acdf0c[person_id*0x4f + 4]` (secondary), then reached
/// via `*(Comp+0) + slot_index * 0x50`.
///
/// `None` means the person doesn't have that slot (`< 0` index) —
/// the corresponding branch in `FUN_00843970` simply doesn't fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SquadSlotView {
    /// Position of this slot in the comp's SquadRecord array.
    /// Same value returned by `FUN_004d59d0` / `FUN_004d5b00`.
    pub slot_index: i32,
    /// `SquadRecord+0x04` — the person_id currently occupying
    /// this slot. Identity-match target.
    pub occupant_person_id: u32,
}

/// One squad-slot write emitted by
/// [`apply_squad_registration`]. Materialises to
/// `*(*Comp+0)[slot_index].position_code = new_position_code`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SquadSlotWrite {
    pub slot_index: i32,
    pub new_position_code: i8,
}

/// Complete input to one `FUN_00843970` call.
///
/// The caller is responsible for pre-resolving:
/// * Which of the person's two slots exist (primary/secondary)
///   by reading `DAT_00acdf0c[person_id * 0x4f + {0, 4}]`.
/// * Each slot's current occupant via `*(Comp+0)[slot*0x50 + 4]`.
/// * `own_staff_record_addr` via `FUN_0052a5a0(person, 0, 1)`
///   (the person's staff-record pointer, used for a secondary
///   identity match).
/// * `staff_records_base` = `DAT_00acd5bc` (the staff-record
///   array base).
#[derive(Debug, Clone, Copy)]
pub struct RegistrationInput {
    /// The person being registered. `person_id` matches
    /// `SquadSlotView::occupant_person_id` for the primary
    /// identity check.
    pub person: PersonEligibility,
    /// Primary slot view (from
    /// `DAT_00acdf0c[person_id*0x4f + 0]`). `None` when the
    /// person's primary slot index is `< 0`.
    pub primary_slot: Option<SquadSlotView>,
    /// Secondary slot view (from
    /// `DAT_00acdf0c[person_id*0x4f + 4]`).
    pub secondary_slot: Option<SquadSlotView>,
    /// Position code (byte range −50..=50). Out-of-range values
    /// cause a silent error-log write.
    pub position_code: i8,
    /// The person's staff-record pointer if they are also a
    /// staff member (`FUN_0052a5a0(person, 0, 1)`). Used for the
    /// staff-alternate identity match when the primary/secondary
    /// slot occupant differs but staff addresses coincide.
    pub own_staff_record_addr: Option<usize>,
    /// `DAT_00acd5bc` — base of the staff-record array (stride
    /// 0x245). Used to reconstruct the address of a slot
    /// occupant's staff record for the alternate identity match.
    pub staff_records_base: usize,
}

/// Effect from one `FUN_00843970` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RegistrationEffect {
    /// Write to the primary slot, if any.
    pub primary_write: Option<SquadSlotWrite>,
    /// Write to the secondary slot, if any.
    pub secondary_write: Option<SquadSlotWrite>,
    /// True iff `position_code` was out of range. When true, the
    /// exe logs an error via `FUN_00933d8f` and clears
    /// `DAT_00b4d5a8`; no slot writes fire.
    pub out_of_range: bool,
}

impl RegistrationEffect {
    pub fn any_write(&self) -> bool {
        self.primary_write.is_some() || self.secondary_write.is_some()
    }
}

/// Pure port of `FUN_00843970`.
///
/// Emits zero, one, or two `position_code` writes into the
/// person's two possible SquadRecord slots. No allocation. No
/// mutation of the person's `+0xD7` comp list (that's populated
/// upstream). Idempotent — a `position_code` write to the same
/// slot with the same value is a no-op on the target byte.
pub fn apply_squad_registration(inp: &RegistrationInput) -> RegistrationEffect {
    // Out-of-range position: silent early-return. Verified in
    // DD line 12: `if (param_3 < -50 || param_3 > 50) { log(); return; }`.
    if inp.position_code < POSITION_CODE_MIN || inp.position_code > POSITION_CODE_MAX {
        return RegistrationEffect {
            primary_write: None,
            secondary_write: None,
            out_of_range: true,
        };
    }

    let identity_matches = |slot: &SquadSlotView| -> bool {
        // Match 1: slot's occupant id == this person's id.
        if slot.occupant_person_id == inp.person.person_id {
            return true;
        }
        // Match 2: staff-alternate. `FUN_0052a5a0(person, 0, 1)`
        // returned the person's staff-record ptr. Compare
        // against `staff_records_base + occupant_person_id
        // * 0x245` — the staff-record address of the slot's
        // current occupant.
        if let Some(own_staff) = inp.own_staff_record_addr {
            let candidate = inp.staff_records_base
                .wrapping_add((slot.occupant_person_id as usize) * 0x245);
            if own_staff == candidate {
                return true;
            }
        }
        false
    };

    let make_write = |slot: &SquadSlotView| -> Option<SquadSlotWrite> {
        if identity_matches(slot) {
            Some(SquadSlotWrite {
                slot_index: slot.slot_index,
                new_position_code: inp.position_code,
            })
        } else {
            None
        }
    };

    let primary_write = inp.primary_slot.as_ref().and_then(make_write);
    let secondary_write = inp.secondary_slot.as_ref().and_then(make_write);

    RegistrationEffect {
        primary_write,
        secondary_write,
        out_of_range: false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- probe_person_eligibility ---

    #[test]
    fn probe_returns_1_when_current_eligible_no_other() {
        let p = PersonEligibility { person_id: 1, flags_52: 0x02 };
        assert_eq!(probe_person_eligibility(Some(&p), None), 1);
    }

    #[test]
    fn probe_returns_0_when_current_missing() {
        assert_eq!(probe_person_eligibility(None, None), 0);
    }

    #[test]
    fn probe_returns_0_when_current_ineligible() {
        let p = PersonEligibility { person_id: 1, flags_52: 0x00 };
        assert_eq!(probe_person_eligibility(Some(&p), None), 0);
        // Other bits of +0x52 set but not the 0x02 bit.
        let p2 = PersonEligibility { person_id: 1, flags_52: 0xFD };
        assert_eq!(probe_person_eligibility(Some(&p2), None), 0);
    }

    #[test]
    fn probe_returns_1_when_both_eligible() {
        let p = PersonEligibility { person_id: 1, flags_52: 0x02 };
        let o = PersonEligibility { person_id: 2, flags_52: 0x03 };
        assert_eq!(probe_person_eligibility(Some(&p), Some(&o)), 1);
    }

    #[test]
    fn probe_returns_0_when_other_ineligible() {
        let p = PersonEligibility { person_id: 1, flags_52: 0x02 };
        let o = PersonEligibility { person_id: 2, flags_52: 0x00 };
        assert_eq!(probe_person_eligibility(Some(&p), Some(&o)), 0);
    }

    // --- appointment_transition_register_flag (FUN_00668380 shape) ---

    #[test]
    fn transition_invalid_to_valid_sets_flag_1() {
        let old = PersonEligibility { person_id: 0, flags_52: 0x00 }; // ineligible
        let new = PersonEligibility { person_id: 1, flags_52: 0x02 }; // eligible
        assert_eq!(
            appointment_transition_register_flag(Some(&old), Some(&new)),
            1,
        );
    }

    #[test]
    fn transition_none_to_valid_sets_flag_1() {
        // old occupant was null (probe1 returns 0).
        let new = PersonEligibility { person_id: 1, flags_52: 0x02 };
        assert_eq!(
            appointment_transition_register_flag(None, Some(&new)),
            1,
        );
    }

    #[test]
    fn transition_valid_to_anything_yields_flag_0() {
        let old = PersonEligibility { person_id: 0, flags_52: 0x02 }; // eligible
        let new = PersonEligibility { person_id: 1, flags_52: 0x02 };
        assert_eq!(
            appointment_transition_register_flag(Some(&old), Some(&new)),
            0,
            "old already valid — walk skips"
        );
    }

    #[test]
    fn transition_invalid_to_invalid_yields_flag_0() {
        let old = PersonEligibility { person_id: 0, flags_52: 0x00 };
        let new = PersonEligibility { person_id: 1, flags_52: 0x00 };
        assert_eq!(
            appointment_transition_register_flag(Some(&old), Some(&new)),
            0,
        );
    }

    #[test]
    fn transition_valid_to_none_yields_flag_0() {
        let old = PersonEligibility { person_id: 0, flags_52: 0x02 };
        assert_eq!(
            appointment_transition_register_flag(Some(&old), None),
            0,
        );
    }

    // --- apply_squad_registration ---

    fn base_input() -> RegistrationInput {
        RegistrationInput {
            person: PersonEligibility { person_id: 100, flags_52: 0x02 },
            primary_slot: Some(SquadSlotView {
                slot_index: 5,
                occupant_person_id: 100, // matches
            }),
            secondary_slot: None,
            position_code: 4,
            own_staff_record_addr: None,
            staff_records_base: 0x00acd5bc,
        }
    }

    #[test]
    fn register_writes_primary_slot_when_person_id_matches() {
        let out = apply_squad_registration(&base_input());
        assert!(!out.out_of_range);
        let w = out.primary_write.expect("expected primary write");
        assert_eq!(w.slot_index, 5);
        assert_eq!(w.new_position_code, 4);
        assert!(out.secondary_write.is_none());
    }

    #[test]
    fn register_writes_both_slots_when_both_match() {
        let mut inp = base_input();
        inp.secondary_slot = Some(SquadSlotView {
            slot_index: 12,
            occupant_person_id: 100,
        });
        let out = apply_squad_registration(&inp);
        let pw = out.primary_write.expect("primary");
        let sw = out.secondary_write.expect("secondary");
        assert_eq!(pw.slot_index, 5);
        assert_eq!(sw.slot_index, 12);
        assert_eq!(pw.new_position_code, 4);
        assert_eq!(sw.new_position_code, 4);
    }

    #[test]
    fn register_skips_slot_when_occupant_mismatch_no_staff_alt() {
        let mut inp = base_input();
        inp.primary_slot = Some(SquadSlotView {
            slot_index: 5,
            occupant_person_id: 999, // NOT this person, no staff addr
        });
        let out = apply_squad_registration(&inp);
        assert!(out.primary_write.is_none());
    }

    #[test]
    fn register_uses_staff_alternate_identity_match() {
        // Person 100 has staff address = base + 999*0x245 (i.e.
        // they are the staff-record occupant even though their
        // person id differs from the SquadRecord's occupant id).
        let mut inp = base_input();
        inp.primary_slot = Some(SquadSlotView {
            slot_index: 5,
            occupant_person_id: 999,
        });
        inp.own_staff_record_addr = Some(
            (0x00acd5bcusize).wrapping_add(999 * 0x245));
        let out = apply_squad_registration(&inp);
        let w = out.primary_write.expect("alt-identity match should write");
        assert_eq!(w.slot_index, 5);
    }

    #[test]
    fn register_out_of_range_position_produces_no_writes() {
        // Below range.
        let mut inp = base_input();
        inp.position_code = -51;
        let out = apply_squad_registration(&inp);
        assert!(out.out_of_range);
        assert!(!out.any_write());
        // Above range.
        inp.position_code = 51;
        let out2 = apply_squad_registration(&inp);
        assert!(out2.out_of_range);
        assert!(!out2.any_write());
        // Exactly on boundary — valid.
        inp.position_code = 50;
        let out3 = apply_squad_registration(&inp);
        assert!(!out3.out_of_range);
        assert!(out3.any_write());
        inp.position_code = -50;
        let out4 = apply_squad_registration(&inp);
        assert!(!out4.out_of_range);
        assert!(out4.any_write());
    }

    #[test]
    fn register_no_slots_is_silent_noop() {
        let mut inp = base_input();
        inp.primary_slot = None;
        inp.secondary_slot = None;
        let out = apply_squad_registration(&inp);
        assert!(!out.out_of_range);
        assert!(out.primary_write.is_none());
        assert!(out.secondary_write.is_none());
    }

    #[test]
    fn register_is_idempotent_same_position_code() {
        let inp = base_input();
        let a = apply_squad_registration(&inp);
        let b = apply_squad_registration(&inp);
        assert_eq!(a, b);
    }

    #[test]
    fn register_different_position_codes_produce_different_writes() {
        let mut inp = base_input();
        inp.position_code = 4;
        let a = apply_squad_registration(&inp).primary_write.unwrap();
        inp.position_code = -7;
        let b = apply_squad_registration(&inp).primary_write.unwrap();
        assert_eq!(a.slot_index, b.slot_index);
        assert_ne!(a.new_position_code, b.new_position_code);
    }

    // --- C14.6 aliases + Reading A semantics ---

    #[test]
    fn c14_6_alias_comp_eligibility_matches_original_type() {
        // CompEligibility is a re-export of PersonEligibility with
        // the corrected semantic name.
        let c: CompEligibility = PersonEligibility {
            person_id: 7, // really: comp_id for Premier
            flags_52: 0x02,
        };
        assert!(c.is_eligible());
        assert_eq!(
            probe_club_primary_comp_active(Some(&c), None),
            1,
        );
    }

    #[test]
    fn c14_6_comp_activation_transition_matches_underlying_predicate() {
        // "old inactive → new active" = register_flag 1.
        let inactive = CompEligibility { person_id: 999, flags_52: 0 };
        let active   = CompEligibility { person_id: 7,   flags_52: 0x02 };
        assert_eq!(
            comp_activation_register_flag(Some(&inactive), Some(&active)),
            1,
        );
        // Both active = flag 0 (no fresh activation).
        assert_eq!(
            comp_activation_register_flag(Some(&active), Some(&active)),
            0,
        );
    }

    // --- 50-slot promotion walk simulation ---

    #[test]
    fn full_50_slot_walk_registers_each_valid_person_once() {
        // Simulate the promotion walk: 50 slots. Populate with
        // 20 valid persons; the rest empty. Each valid person
        // has a primary SquadSlot (occupant match) so
        // apply_squad_registration produces exactly one write.
        let n_valid = 20usize;
        let mut effects = Vec::new();
        for i in 0..50 {
            if i >= n_valid {
                continue; // "None" slot
            }
            let pid = (i as u32) + 100;
            let inp = RegistrationInput {
                person: PersonEligibility {
                    person_id: pid,
                    flags_52: 0x02,
                },
                primary_slot: Some(SquadSlotView {
                    slot_index: (i as i32) + 1,
                    occupant_person_id: pid,
                }),
                secondary_slot: None,
                position_code: 5,
                own_staff_record_addr: None,
                staff_records_base: 0,
            };
            effects.push(apply_squad_registration(&inp));
        }
        assert_eq!(effects.len(), n_valid);
        assert!(effects.iter().all(|e| e.primary_write.is_some()));
        assert!(effects.iter().all(|e| e.secondary_write.is_none()));
    }
}
