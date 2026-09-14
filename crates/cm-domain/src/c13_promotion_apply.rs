//! C13 — Traditional English promotion / relegation APPLY layer.
//!
//! Recovers and ports the actual mutation primitives that
//! `FUN_0066EED0` (P/R swap) delegates to. Decision layer (C7 / C8)
//! and status layer (C12) already model WHO moves and what statuses
//! flow through them; C13 models WHAT HAPPENS when that move is
//! applied — the concrete field writes on Club records, staff
//! records, and the news/history side effects.
//!
//! # Scope
//!
//! C13 covers:
//! * `FUN_00668380` — promotion install
//! * `FUN_00668470` — relegation install (Conference coordinator entry)
//! * `FUN_004D3550` — promotion per-person walk
//! * `FUN_004D3460` — relegation per-person walk (NOT `FUN_004D3700`;
//!   the C12 note carried an off-by-one on the address — no such VA
//!   exists in the decompile).
//! * `FUN_00583FC0` — stadium expansion. **Boundary reached here** —
//!   this expands into the C14 stadium-expansion transaction. C13
//!   exposes it as a deferred [`StadiumExpansionRequest`] and stops.
//!
//! # NOT in scope
//!
//! * Actual World mutation. All results are `struct`s the caller
//!   applies.
//! * The full annual English rollover scheduler (C15).
//! * The `+0x37 = 0xFF` post-swap status write itself — that lives
//!   in the outer swap primitive (`FUN_0066EED0`), already ported
//!   as [`crate::eng_second_fixtures::promote_relegate_swap`] +
//!   [`crate::year_end_statuses::apply_swap_status_transform`].
//!
//! # Confidence per helper
//!
//! * `FUN_00668380` port — **STRUCTURALLY PORTED** (DD decompile).
//! * `FUN_00668470` port — **STRUCTURALLY PORTED**.
//! * `FUN_004D3460` port — **STRUCTURALLY PORTED**.
//! * `FUN_004D3550` port — **STRUCTURALLY PORTED**.
//! * `FUN_00583FC0` (C14 boundary marker only) — **DIRECT**.
//!
//! A runtime differential against a captured GDI year-end
//! promotion/relegation pass would upgrade these to `StateExact`;
//! that's a follow-up capture.

use serde::{Deserialize, Serialize};

use crate::year_end_statuses::{
    is_sticky_status, STATUS_IDLE, STATUS_RELEGATED, STATUS_AUTO_PROMOTION,
    STATUS_PLAYOFF_WINNER,
};

// ---------------------------------------------------------------------------
// Input types
// ---------------------------------------------------------------------------

/// One club's pre-apply state — the fields the apply layer reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClubPreApply {
    pub club_id: u32,
    /// `Club+0x37`. C13 does NOT rewrite this — the outer swap does.
    pub status_37: u8,
    /// `Club+0x57` — current comp id.
    pub current_comp_id: u32,
    /// `Club+0x64` — a tier byte read by `FUN_00668380`. Value `3`
    /// gates the `Club+0x64 = 2` write inside the promotion install
    /// (verified in DD line 39–40).
    pub tier_byte_64: u8,
    /// `Club+0x69` — stadium pointer / id. Non-null gates the
    /// stadium-expansion branch inside promotion install.
    pub stadium_id: Option<i32>,
    /// `Club+0xCF` — news-guard flag; `0` allows welcome-news to
    /// fire.
    pub news_flag_cf: u8,
    /// The 50-slot person array at `Club+0xD7`. Slot semantics per
    /// [`PersonSlot`].
    pub person_slots: [Option<PersonSlot>; 50],
    /// Reserve club id resolved via `FUN_0052A5A0(club, &out, 1)`.
    /// `None` when the club has no reserve/parent record.
    pub reserve_club_id: Option<u32>,
}

/// One person slot at `Club+0xD7 + i*4`. `None` means the slot
/// pointer is null (empty). `Some(...)` carries the state the
/// per-person walk actually reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonSlot {
    pub person_id: u32,
    /// Staff record's `+0x1F` byte before the walk. Only `1`
    /// triggers the relegation-side transform.
    pub staff_1f: u8,
    /// Staff record's `+0x1C` byte before the walk. Cleared by
    /// the promotion-side walk when it was `1`.
    pub staff_1c: u8,
}

/// Destination comp's fields the apply layer reads. Present on the
/// promotion side (a real comp being installed into); `None` on
/// the relegation side when the target comp is null (Conference
/// bottom → nothing to install into).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompPreApply {
    pub comp_id: u32,
    /// `Comp+0x43` — comp type byte. `2` gates the stadium-
    /// expansion branch.
    pub type_43: u8,
    /// `Comp+0xE2` / `Comp+0xE4` — stadium capacity template pair.
    /// If either is `-1` (sentinel) the stadium-expansion branch
    /// skips. Stored as `i32` so real capacities (>32767 seats) fit
    /// alongside the `-1` sentinel.
    pub capacity_template_e2: i32,
    pub capacity_template_e4: i32,
}

// ---------------------------------------------------------------------------
// Output effect types
// ---------------------------------------------------------------------------

/// Concrete Club record writes emitted by one promotion or
/// relegation install. All fields are on the club being moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClubFieldWrites {
    /// New `Club+0x57` value (destination comp id).
    pub new_comp_id: u32,
    /// New `Club+0x5B` value = old `Club+0x57`.
    pub prev_comp_id: u32,
    /// When `Some`, write to `Club+0x64`. Value `2` fires on
    /// promotion install when the incoming `+0x64` was `3` and the
    /// destination comp exists (`FUN_00668380` line 39–40).
    pub tier_byte_64: Option<u8>,
}

/// Per-person effect from one call to
/// [`walk_relegation_persons`] or [`walk_promotion_persons`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonEffect {
    /// The person's staff record id.
    pub person_id: u32,
    /// New value of `staff+0x1F`. `None` = unchanged.
    pub new_staff_1f: Option<u8>,
    /// New value of `staff+0x1C`. `None` = unchanged.
    pub new_staff_1c: Option<u8>,
    /// Emit an event record via `FUN_008D0D90(slot, old_comp, kind,
    /// staff)`. `None` = no event fired.
    pub event_emit: Option<PersonHistoryEvent>,
}

/// A history record `FUN_008D0D90` emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonHistoryEvent {
    /// Old comp id passed as arg 2. Diagnostic.
    pub old_comp_id: u32,
    /// Event kind. `3` = relegated. Other codes come from the
    /// promotion side / other subsystems.
    pub kind: u8,
}

/// News/press-event request. `FUN_00680CC0(club, 0)` — the
/// "welcome to Prem"-style press event. Gated by `param_3 != 0`
/// (news enable) AND `Club[0xCF] == 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromotionWelcomeNews {
    pub club_id: u32,
    pub new_comp_id: u32,
}

/// Relegation news request — `FUN_006809E0(club, 1, 7, 0)`, fires
/// only when the new comp is null AND
/// `FUN_005EA590(club, 1, 1, 0, 0)` returns non-zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelegationNoLeagueNews {
    pub club_id: u32,
}

/// Deferred stadium-expansion transaction request — C14 boundary.
/// Populated by promotion install when the destination comp
/// advertises a bigger capacity template than the club's current
/// stadium. C13 does NOT execute the expansion; C14 owns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StadiumExpansionRequest {
    pub club_id: u32,
    /// Comp's `+0xE2` template.
    pub desired_seating: i32,
    /// Comp's `+0xE4` template.
    pub desired_capacity: i32,
    /// Comp id for the finance/news notify.
    pub new_comp_id: u32,
}

/// The full effect envelope from one promotion apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionApplyEffects {
    pub club_id: u32,
    pub writes: ClubFieldWrites,
    /// Post-swap `Club+0x37 = 0xFF` write (the outer swap does this;
    /// C13 exposes it here so a caller running C13 in isolation
    /// still emits the correct byte).
    pub set_status_idle: bool,
    pub person_effects: Vec<PersonEffect>,
    pub welcome_news: Option<PromotionWelcomeNews>,
    /// Populated when the promotion triggers a C14 stadium
    /// expansion. C13 stops at the boundary — the caller passes
    /// this to C14 when C14 lands.
    pub stadium_expansion: Option<StadiumExpansionRequest>,
}

/// The full effect envelope from one relegation apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelegationApplyEffects {
    pub club_id: u32,
    /// `None` when the new comp is null (the exe still writes
    /// `Club+0x57 = null` — but the Rust port models this as
    /// "kicked out of the league").
    pub writes: ClubFieldWrites,
    pub set_status_idle: bool,
    pub person_effects: Vec<PersonEffect>,
    /// Fires when the new comp is null (Conf → nothing). The
    /// manager-probe (`FUN_005EA590`) precedes this; C13 models
    /// the news request; the probe's response drives whether it
    /// actually posts.
    pub no_league_news: Option<RelegationNoLeagueNews>,
}

// ---------------------------------------------------------------------------
// FUN_004D3460 port — relegation per-person walk
// ---------------------------------------------------------------------------

/// Iterate the 50-slot `Club+0xD7` array on both `club` and its
/// reserve counterpart, and for every valid slot whose staff
/// record currently reads `+0x1F == 1`, downgrade to `2` and emit
/// a `kind=3` (relegated) history event tagged with `old_comp_id`.
///
/// Empty-slot / invalid-slot semantics (verified in DD decompile
/// of `FUN_004D3460`):
/// * `Option::None` slot pointer → skipped.
/// * Person id ≥ pool bound → skipped (Rust port: we don't model
///   the pool bound explicitly; the caller resolves valid slots
///   into `Some(PersonSlot)`).
/// * `staff+0x1F != 1` → skipped (state guard).
///
/// Iteration runs across all 50 slots on both `club` and
/// `reserve`; there is no early break. Duplicates between the two
/// clubs are unusual but not defended against — the walk transforms
/// them twice, which is idempotent (`1→2`, then `2→2` no-op).
pub fn walk_relegation_persons(
    club: &ClubPreApply,
    reserve: Option<&ClubPreApply>,
    old_comp_id: u32,
) -> Vec<PersonEffect> {
    let mut out = Vec::new();
    for source in std::iter::once(Some(club)).chain(std::iter::once(reserve)) {
        let Some(c) = source else { continue; };
        for slot in c.person_slots.iter().flatten() {
            if slot.staff_1f != 1 {
                continue;
            }
            out.push(PersonEffect {
                person_id: slot.person_id,
                new_staff_1f: Some(2),
                new_staff_1c: None,
                event_emit: Some(PersonHistoryEvent {
                    old_comp_id,
                    kind: 3,
                }),
            });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// FUN_004D3550 port — promotion per-person walk
// ---------------------------------------------------------------------------

/// Result of the promotion per-person walk.
///
/// The DD decompile shows two loops:
/// * When `register_flag != 0`: 50-slot walk over `club` only,
///   calling `FUN_00843970(slot, club, 0)` per valid person. This
///   is the squad-manager registration into the new comp — modeled
///   as [`PersonEffect::event_emit`] with `kind = 1` (registered).
/// * Always: two-pass walk (self + reserve), for each valid staff
///   record whose id matches the slot:
///     - `staff+0x1F == 1` → reset to `0` (clear relegation flag).
///     - `staff+0x1C == 1` → reset to `0`.
pub fn walk_promotion_persons(
    club: &ClubPreApply,
    reserve: Option<&ClubPreApply>,
    register_flag: bool,
) -> Vec<PersonEffect> {
    let mut out = Vec::new();

    // Loop A — squad registration into the new comp.
    if register_flag {
        for slot in club.person_slots.iter().flatten() {
            out.push(PersonEffect {
                person_id: slot.person_id,
                new_staff_1f: None,
                new_staff_1c: None,
                event_emit: Some(PersonHistoryEvent {
                    old_comp_id: club.current_comp_id,
                    // `1` = squad-registered on promotion install
                    // (docs: FUN_00843970 delegated); Kept
                    // distinct from relegation's `3`.
                    kind: 1,
                }),
            });
        }
    }

    // Loop B — clear per-staff flags across self + reserve.
    for source in std::iter::once(Some(club)).chain(std::iter::once(reserve)) {
        let Some(c) = source else { continue; };
        for slot in c.person_slots.iter().flatten() {
            let clear_1f = slot.staff_1f == 1;
            let clear_1c = slot.staff_1c == 1;
            if !clear_1f && !clear_1c {
                continue;
            }
            out.push(PersonEffect {
                person_id: slot.person_id,
                new_staff_1f: clear_1f.then_some(0),
                new_staff_1c: clear_1c.then_some(0),
                event_emit: None,
            });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// FUN_00668380 port — promotion install
// ---------------------------------------------------------------------------

/// Configuration passed alongside the club. Mirrors `FUN_00668380`
/// param_3 (news enable) and the comp record fields the install
/// dereferences.
#[derive(Debug, Clone, Copy)]
pub struct PromotionInstallCtx<'a> {
    /// Destination comp (the tier ABOVE, being installed into).
    pub new_comp: CompPreApply,
    /// Whether to emit welcome-news / press events. Corresponds to
    /// `FUN_00668380` param_3 != 0 AND `Club[0xCF] == 0`. Caller
    /// resolves the AND; here `true` = fire when other conditions
    /// hold.
    pub news_enable: bool,
    /// Optional reserve club looked up via `FUN_0052A5A0`.
    pub reserve: Option<&'a ClubPreApply>,
    /// Whether the second `FUN_00843ef0(club, 0)` gate succeeded
    /// (returns 0). If so, the person walk's register_flag is `1`.
    /// Callers without a squad-eligibility model can pass `true`.
    pub squad_register_flag: bool,
}

/// Port of `FUN_00668380`. Emits the full effect envelope for one
/// promotion install. Applies no state changes — the caller does.
pub fn apply_promotion_install(
    club: &ClubPreApply,
    ctx: &PromotionInstallCtx<'_>,
) -> PromotionApplyEffects {
    // Club field writes.
    let writes = ClubFieldWrites {
        new_comp_id: ctx.new_comp.comp_id,
        prev_comp_id: club.current_comp_id,
        // Line 39–40: `if (Club[0x64] == 3 && dest != null) write 2`.
        // C13 always has a real dest (`ctx.new_comp` is not-null).
        tier_byte_64: (club.tier_byte_64 == 3).then_some(2),
    };

    // Person walk — FUN_004D3550.
    let person_effects =
        walk_promotion_persons(club, ctx.reserve, ctx.squad_register_flag);

    // Welcome news — gated by news_enable && Club[0xCF] == 0.
    let welcome_news = (ctx.news_enable && club.news_flag_cf == 0).then(|| {
        PromotionWelcomeNews {
            club_id: club.club_id,
            new_comp_id: ctx.new_comp.comp_id,
        }
    });

    // Stadium expansion — deferred to C14.
    // Gates (DD lines 30–33): stadium_id.is_some() AND
    // new_comp.type_43 == 2 AND new_comp.capacity_template_e2 != -1
    // AND ...E4 != -1.
    let stadium_expansion = if club.stadium_id.is_some()
        && ctx.new_comp.type_43 == 2
        && ctx.new_comp.capacity_template_e2 != -1
        && ctx.new_comp.capacity_template_e4 != -1
    {
        Some(StadiumExpansionRequest {
            club_id: club.club_id,
            desired_seating: ctx.new_comp.capacity_template_e2,
            desired_capacity: ctx.new_comp.capacity_template_e4,
            new_comp_id: ctx.new_comp.comp_id,
        })
    } else {
        None
    };

    PromotionApplyEffects {
        club_id: club.club_id,
        writes,
        set_status_idle: true,
        person_effects,
        welcome_news,
        stadium_expansion,
    }
}

// ---------------------------------------------------------------------------
// FUN_00668470 port — relegation install / Conference coordinator entry
// ---------------------------------------------------------------------------

/// Configuration for the relegation install.
#[derive(Debug, Clone, Copy)]
pub struct RelegationInstallCtx<'a> {
    /// Destination comp (the tier BELOW). `None` = "no league" case
    /// — the Conf coordinator's stadium-fail branch relegates a
    /// club into no destination, and the exe fires the
    /// "manager unhappy" probe + news #7.
    pub new_comp: Option<CompPreApply>,
    /// Optional reserve club.
    pub reserve: Option<&'a ClubPreApply>,
    /// Did `FUN_005EA590(club, 1, 1, 0, 0)` return non-zero?
    /// When `new_comp` is `None` AND this is `true`, emit the news.
    pub manager_unhappy: bool,
}

/// Port of `FUN_00668470`.
pub fn apply_relegation_install(
    club: &ClubPreApply,
    ctx: &RelegationInstallCtx<'_>,
) -> RelegationApplyEffects {
    let new_comp_id = ctx.new_comp.map(|c| c.comp_id).unwrap_or(0);
    let writes = ClubFieldWrites {
        new_comp_id,
        prev_comp_id: club.current_comp_id,
        // Not touched by relegation install.
        tier_byte_64: None,
    };

    // Person walk — FUN_004D3460, tagged with the OLD comp id.
    let person_effects =
        walk_relegation_persons(club, ctx.reserve, club.current_comp_id);

    let no_league_news = if ctx.new_comp.is_none() && ctx.manager_unhappy {
        Some(RelegationNoLeagueNews { club_id: club.club_id })
    } else {
        None
    };

    RelegationApplyEffects {
        club_id: club.club_id,
        writes,
        set_status_idle: true,
        person_effects,
        no_league_news,
    }
}

// ---------------------------------------------------------------------------
// Apply order (documented — see [`crate::eng_second_fixtures`]
// FUN_0066EED0 doc for the outer sequencing)
// ---------------------------------------------------------------------------

/// Order in which the caller must apply one promotion apply
/// envelope's fields. Mirrors the DD decompile's execution order.
///
/// (This is data, not code — Rust callers may materialise fields
/// in any order because the effects are pre-computed. But the exe's
/// order is preserved here so runtime differentials can be
/// interpreted correctly.)
pub const PROMOTION_APPLY_ORDER: &[&str] = &[
    "Club+0x5b = old Club+0x57",  // FUN_00668380 line 11
    "Club+0x57 = new_comp",       // FUN_00668380 line 12
    "FUN_00843ef0(club, 0)",      // eligibility probe 1
    "FUN_004D3550 person walk",   // squad register + flag clear
    "FUN_00843ef0(club, 0)",      // eligibility probe 2 (register-flag decision)
    "if Club+0x64 == 3: Club+0x64 = 2",
    "if news_enable && Club+0xCF == 0: FUN_00680CC0 welcome news",
    "if stadium expansion: FUN_00583FC0 (C14)",
    "outer FUN_0066EED0: Club+0x37 = 0xFF",
];

pub const RELEGATION_APPLY_ORDER: &[&str] = &[
    "Club+0x57 = new_comp (may be null)",     // FUN_00668470 line 9
    "Club+0x5b = old Club+0x57",              // FUN_00668470 line 10
    "FUN_004D3460 relegation person walk",
    "if new_comp is null: FUN_005EA590 probe",
    "if probe != 0: FUN_006809E0 no-league news #7",
    "outer FUN_0066EED0: Club+0x37 = 0xFF",
];

// ---------------------------------------------------------------------------
// Full C7-edge-driven apply (composes decision + C13 apply)
// ---------------------------------------------------------------------------

/// Aggregate result from applying all promotions + relegations on
/// one P/R edge (e.g. Prem↔First).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeApplyResult {
    pub promotions: Vec<PromotionApplyEffects>,
    pub relegations: Vec<RelegationApplyEffects>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_person_slots() -> [Option<PersonSlot>; 50] {
        [None; 50]
    }

    fn slot(id: u32, s1f: u8, s1c: u8) -> PersonSlot {
        PersonSlot { person_id: id, staff_1f: s1f, staff_1c: s1c }
    }

    fn bare_club(id: u32, comp: u32) -> ClubPreApply {
        ClubPreApply {
            club_id: id,
            status_37: STATUS_AUTO_PROMOTION,
            current_comp_id: comp,
            tier_byte_64: 0,
            stadium_id: Some(1000),
            news_flag_cf: 0,
            person_slots: empty_person_slots(),
            reserve_club_id: None,
        }
    }

    fn bare_comp(id: u32) -> CompPreApply {
        CompPreApply {
            comp_id: id,
            type_43: 0,
            capacity_template_e2: -1,
            capacity_template_e4: -1,
        }
    }

    // -------- Isolated promotion --------
    #[test]
    fn isolated_promotion_writes_club_fields() {
        let club = bare_club(101, 9); // was in D2
        let dest = bare_comp(8);      // now promoted to D1
        let ctx = PromotionInstallCtx {
            new_comp: dest,
            news_enable: true,
            reserve: None,
            squad_register_flag: true,
        };
        let eff = apply_promotion_install(&club, &ctx);
        assert_eq!(eff.club_id, 101);
        assert_eq!(eff.writes.new_comp_id, 8);
        assert_eq!(eff.writes.prev_comp_id, 9);
        assert_eq!(eff.writes.tier_byte_64, None);
        assert!(eff.set_status_idle);
        assert!(eff.welcome_news.is_some());
        assert_eq!(eff.welcome_news.unwrap().new_comp_id, 8);
        // No person slots -> no per-person effects.
        assert!(eff.person_effects.is_empty());
        // No stadium expansion (comp type_43 != 2).
        assert!(eff.stadium_expansion.is_none());
    }

    #[test]
    fn promotion_writes_tier_64_when_current_is_3() {
        let mut club = bare_club(101, 9);
        club.tier_byte_64 = 3;
        let ctx = PromotionInstallCtx {
            new_comp: bare_comp(8),
            news_enable: false,
            reserve: None,
            squad_register_flag: false,
        };
        let eff = apply_promotion_install(&club, &ctx);
        assert_eq!(eff.writes.tier_byte_64, Some(2));
    }

    #[test]
    fn promotion_suppresses_welcome_when_news_flag_set() {
        let mut club = bare_club(101, 9);
        club.news_flag_cf = 1; // suppression
        let ctx = PromotionInstallCtx {
            new_comp: bare_comp(8),
            news_enable: true,
            reserve: None,
            squad_register_flag: false,
        };
        let eff = apply_promotion_install(&club, &ctx);
        assert!(eff.welcome_news.is_none());
    }

    #[test]
    fn promotion_triggers_stadium_expansion_request_when_gates_pass() {
        let club = bare_club(101, 9);
        let dest = CompPreApply {
            comp_id: 7,
            type_43: 2,
            capacity_template_e2: 40_000,
            capacity_template_e4: 55_000,
        };
        let ctx = PromotionInstallCtx {
            new_comp: dest, news_enable: true, reserve: None,
            squad_register_flag: false,
        };
        let eff = apply_promotion_install(&club, &ctx);
        let sx = eff.stadium_expansion.expect("expected stadium request");
        assert_eq!(sx.club_id, 101);
        assert_eq!(sx.desired_seating, 40_000);
        assert_eq!(sx.desired_capacity, 55_000);
        assert_eq!(sx.new_comp_id, 7);
    }

    #[test]
    fn promotion_does_not_trigger_expansion_when_type_43_not_2() {
        let club = bare_club(101, 9);
        let dest = CompPreApply {
            comp_id: 7, type_43: 0,
            capacity_template_e2: 40_000, capacity_template_e4: 55_000,
        };
        let ctx = PromotionInstallCtx {
            new_comp: dest, news_enable: true, reserve: None,
            squad_register_flag: false,
        };
        let eff = apply_promotion_install(&club, &ctx);
        assert!(eff.stadium_expansion.is_none());
    }

    #[test]
    fn promotion_does_not_trigger_expansion_when_template_is_sentinel() {
        let club = bare_club(101, 9);
        let mut dest = bare_comp(7);
        dest.type_43 = 2;
        dest.capacity_template_e2 = -1;
        dest.capacity_template_e4 = 55_000;
        let ctx = PromotionInstallCtx {
            new_comp: dest, news_enable: true, reserve: None,
            squad_register_flag: false,
        };
        let eff = apply_promotion_install(&club, &ctx);
        assert!(eff.stadium_expansion.is_none());
    }

    // -------- Isolated relegation --------
    #[test]
    fn isolated_relegation_writes_club_fields() {
        let mut club = bare_club(201, 8); // was in D1
        club.status_37 = STATUS_RELEGATED;
        let dest = bare_comp(9); // now relegated to D2
        let ctx = RelegationInstallCtx {
            new_comp: Some(dest), reserve: None, manager_unhappy: false,
        };
        let eff = apply_relegation_install(&club, &ctx);
        assert_eq!(eff.club_id, 201);
        assert_eq!(eff.writes.new_comp_id, 9);
        assert_eq!(eff.writes.prev_comp_id, 8);
        assert_eq!(eff.writes.tier_byte_64, None);
        assert!(eff.set_status_idle);
        assert!(eff.no_league_news.is_none());
    }

    #[test]
    fn relegation_no_new_comp_fires_no_league_news_when_manager_unhappy() {
        let club = bare_club(202, 93);
        let ctx = RelegationInstallCtx {
            new_comp: None, reserve: None, manager_unhappy: true,
        };
        let eff = apply_relegation_install(&club, &ctx);
        assert!(eff.no_league_news.is_some());
        assert_eq!(eff.writes.new_comp_id, 0);
    }

    #[test]
    fn relegation_no_new_comp_no_news_when_manager_happy() {
        let club = bare_club(202, 93);
        let ctx = RelegationInstallCtx {
            new_comp: None, reserve: None, manager_unhappy: false,
        };
        let eff = apply_relegation_install(&club, &ctx);
        assert!(eff.no_league_news.is_none());
    }

    // -------- Person walks --------
    #[test]
    fn relegation_walk_transforms_only_state_1() {
        let mut club = bare_club(300, 8);
        club.person_slots[0] = Some(slot(1, 1, 0)); // state 1 → 2
        club.person_slots[1] = Some(slot(2, 0, 0)); // state 0 skip
        club.person_slots[2] = Some(slot(3, 2, 0)); // state 2 skip
        club.person_slots[3] = None;
        club.person_slots[4] = Some(slot(4, 1, 1));
        let effects = walk_relegation_persons(&club, None, 8);
        assert_eq!(effects.len(), 2, "only state-1 persons emit effects");
        assert!(effects.iter().all(|e| e.new_staff_1f == Some(2)));
        assert!(effects.iter().all(|e| e.event_emit.map(|ev| ev.kind) == Some(3)));
        assert!(effects.iter().all(|e| e.event_emit.map(|ev| ev.old_comp_id) == Some(8)));
        // person ids preserved.
        let ids: Vec<u32> = effects.iter().map(|e| e.person_id).collect();
        assert_eq!(ids, vec![1, 4]);
    }

    #[test]
    fn relegation_walk_covers_both_self_and_reserve() {
        let mut main = bare_club(300, 8);
        main.person_slots[0] = Some(slot(1, 1, 0));
        let mut reserve = bare_club(301, 8);
        reserve.person_slots[0] = Some(slot(2, 1, 0));
        let effects = walk_relegation_persons(&main, Some(&reserve), 8);
        assert_eq!(effects.len(), 2);
        assert_eq!(effects[0].person_id, 1);
        assert_eq!(effects[1].person_id, 2);
    }

    #[test]
    fn promotion_walk_clears_1f_and_1c_flags() {
        let mut club = bare_club(400, 9);
        club.person_slots[0] = Some(slot(1, 1, 0)); // clears 1f
        club.person_slots[1] = Some(slot(2, 0, 1)); // clears 1c
        club.person_slots[2] = Some(slot(3, 1, 1)); // clears both
        club.person_slots[3] = Some(slot(4, 0, 0)); // no effect
        let effects = walk_promotion_persons(&club, None, false);
        assert_eq!(effects.len(), 3);
        // Person 1 — clear 1f only.
        let e1 = effects.iter().find(|e| e.person_id == 1).unwrap();
        assert_eq!(e1.new_staff_1f, Some(0));
        assert_eq!(e1.new_staff_1c, None);
        let e2 = effects.iter().find(|e| e.person_id == 2).unwrap();
        assert_eq!(e2.new_staff_1f, None);
        assert_eq!(e2.new_staff_1c, Some(0));
        let e3 = effects.iter().find(|e| e.person_id == 3).unwrap();
        assert_eq!(e3.new_staff_1f, Some(0));
        assert_eq!(e3.new_staff_1c, Some(0));
    }

    #[test]
    fn promotion_walk_registers_persons_when_flag_set() {
        let mut club = bare_club(400, 9);
        club.person_slots[0] = Some(slot(1, 0, 0));
        club.person_slots[1] = Some(slot(2, 0, 0));
        let effects = walk_promotion_persons(&club, None, true);
        // Register events + zero flag-clear effects (all zero).
        let register_count = effects.iter()
            .filter(|e| e.event_emit.map(|ev| ev.kind) == Some(1))
            .count();
        assert_eq!(register_count, 2);
    }

    // -------- Empty-slot semantics (point 6) --------
    #[test]
    fn null_slots_are_skipped_in_both_walks() {
        let club = bare_club(500, 9);
        // All 50 slots are None → walks emit nothing.
        assert!(walk_relegation_persons(&club, None, 9).is_empty());
        assert!(walk_promotion_persons(&club, None, true).is_empty());
    }

    #[test]
    fn iteration_covers_all_50_slots_no_early_break() {
        let mut club = bare_club(500, 9);
        // Populate slot 0, skip 1..49, populate slot 49.
        club.person_slots[0] = Some(slot(1, 1, 0));
        club.person_slots[49] = Some(slot(50, 1, 0));
        let effects = walk_relegation_persons(&club, None, 9);
        assert_eq!(effects.len(), 2);
        assert_eq!(effects[0].person_id, 1);
        assert_eq!(effects[1].person_id, 50);
    }

    // -------- Status interaction (point 17) --------
    #[test]
    fn status_5_playoff_winner_flows_through_promotion() {
        // C12 guarantees a playoff winner reaches C13 with
        // status_37 == 5. C13 doesn't rewrite +0x37 itself (that's
        // the outer swap's job) but records set_status_idle=true.
        let mut club = bare_club(101, 9);
        club.status_37 = STATUS_PLAYOFF_WINNER;
        let ctx = PromotionInstallCtx {
            new_comp: bare_comp(8), news_enable: false, reserve: None,
            squad_register_flag: false,
        };
        let eff = apply_promotion_install(&club, &ctx);
        assert!(eff.set_status_idle);
        // Verify C12 sticky-guard doesn't misapply here: C13's
        // set_status_idle is unconditional for movers, whereas the
        // outer swap's transform (from year_end_statuses) treats
        // {0,5,3} as movable → 0xFF, so composition is coherent.
        assert!(!is_sticky_status(STATUS_IDLE));
    }

    // -------- Apply order --------
    #[test]
    fn apply_order_constants_are_documented() {
        assert!(!PROMOTION_APPLY_ORDER.is_empty());
        assert!(!RELEGATION_APPLY_ORDER.is_empty());
        assert!(PROMOTION_APPLY_ORDER.iter()
            .any(|s| s.contains("Club+0x37")));
        assert!(RELEGATION_APPLY_ORDER.iter()
            .any(|s| s.contains("Club+0x37")));
    }

    // -------- Point 15 & 16: multi-club edge --------
    #[test]
    fn full_edge_prem_first_apply_all_movers_and_correct_counts() {
        // Prem clubs 1..3 relegated (status 3), First clubs 4..5
        // auto-promoted (status 0), club 6 playoff winner (status 5).
        // Apply the edge: 3 promotions + 3 relegations.
        // We are NOT invoking C7 decision here — the C7 decision
        // proof is in eng_second_fixtures tests. C13 verifies the
        // apply.
        let relegated: Vec<ClubPreApply> = (1..=3).map(|i| {
            let mut c = bare_club(i, 7);
            c.status_37 = STATUS_RELEGATED;
            c
        }).collect();
        let promoted: Vec<ClubPreApply> = (4..=5).map(|i| {
            let mut c = bare_club(i, 8);
            c.status_37 = STATUS_AUTO_PROMOTION;
            c
        }).chain(std::iter::once({
            let mut c = bare_club(6, 8);
            c.status_37 = STATUS_PLAYOFF_WINNER;
            c
        })).collect();

        let mut result = EdgeApplyResult {
            promotions: Vec::new(),
            relegations: Vec::new(),
        };
        for club in &promoted {
            result.promotions.push(apply_promotion_install(club, &PromotionInstallCtx {
                new_comp: bare_comp(7), news_enable: true, reserve: None,
                squad_register_flag: true,
            }));
        }
        for club in &relegated {
            result.relegations.push(apply_relegation_install(club, &RelegationInstallCtx {
                new_comp: Some(bare_comp(8)), reserve: None, manager_unhappy: false,
            }));
        }
        assert_eq!(result.promotions.len(), 3);
        assert_eq!(result.relegations.len(), 3);
        // Every mover receives set_status_idle. No dupes: club_ids all distinct.
        let all_ids: Vec<u32> = result.promotions.iter().map(|e| e.club_id)
            .chain(result.relegations.iter().map(|e| e.club_id))
            .collect();
        let uniq: std::collections::BTreeSet<_> = all_ids.iter().collect();
        assert_eq!(uniq.len(), all_ids.len(),
            "no club may be applied twice on one edge");
        // Promotions all go to comp 7; relegations to comp 8.
        assert!(result.promotions.iter().all(|e| e.writes.new_comp_id == 7));
        assert!(result.relegations.iter().all(|e| e.writes.new_comp_id == 8));
        // Previous-comp fields correct.
        assert!(result.promotions.iter().all(|e| e.writes.prev_comp_id == 8));
        assert!(result.relegations.iter().all(|e| e.writes.prev_comp_id == 7));
    }

    #[test]
    fn edge_third_conference_stadium_fail_reprieve_not_moved() {
        // Point 17: 0xFE reprieved club must not be moved by C13.
        // Simulate: Conf champion has STATUS_STADIUM_REPRIEVE.
        // C13 doesn't select clubs — C7 does — but a caller that
        // hands the champion to apply_promotion_install would be
        // wrong. Verify no separate C13 guard exists: it trusts
        // the caller. This test documents that: STATUS_STADIUM_REPRIEVE
        // reaches C13 iff C7 mis-routes it — and C12's
        // apply_swap_status_transform is what actually enforces
        // "sticky = don't move". Cross-reference test.
        use crate::year_end_statuses::{apply_swap_status_transform,
            STATUS_STADIUM_REPRIEVE};
        assert_eq!(apply_swap_status_transform(STATUS_STADIUM_REPRIEVE),
                   STATUS_STADIUM_REPRIEVE);
    }
}
