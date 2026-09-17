//! C15.1 — Materialise the C15 `AnnualRolloverReport` onto real
//! `World` + `RuntimeSaveGame` state, so subsequent ticks observe
//! the post-rollover situation identically to the exe.
//!
//! # What C15.1 materialises
//!
//! Layer                          | Storage                              | Applied
//! -------------------------------|--------------------------------------|--------
//! Club movement (`+0x57/0x5B/0x37`) | `World.core.clubs[i].raw`         | yes (via `materialise_club_moves`)
//! Stadium capacity              | `World.references.stadiums[i]`        | yes
//! Person-history events         | `RuntimeSaveGame.pending_events`      | yes (as `RuntimeEvent` kind = `"person_history"`)
//! Squad-position resets         | Surfaced on report; `Club+0xD7` slots | partial — squad-slot storage lives on staff.dat pool not yet decoded to typed accessors; effect is emitted as `PendingSquadReset` for a follow-up tranche
//! Stadium-expansion news        | `RuntimeSaveGame.pending_events`      | yes (news template `0x1780`)
//! Promotion welcome news        | `RuntimeSaveGame.pending_events`      | yes
//! No-league relegation news     | `RuntimeSaveGame.pending_events`      | yes
//! Conference stadium-fail news  | `RuntimeSaveGame.pending_events`      | yes
//! Club finance writes           | `RuntimeSaveGame.finance` (`FinanceBook`) | MATERIALISED — C15.1F resolved the offset dispute (disk `Club+0x65` is a one-time seed; runtime cash is the 0x167-byte record's `+0x00` i64 = `ClubFinance.balance`); writes land via `FinanceBook::apply_year_end_write`
//!
//! Ownership rules:
//!
//! * No parallel membership store. Comp lookups continue to walk
//!   `Club+0x57` on the raw club bytes.
//! * No parallel stadium store. Writes go into
//!   `World.references.stadiums`.
//! * No parallel news queue. Writes go into
//!   `RuntimeSaveGame.pending_events` — the same queue every other
//!   subsystem uses.
//! * Trace derivation: the applier folds each event as it fires,
//!   so `WorldApplyReport.applied` reflects what actually landed on
//!   World, not what was intended.

use crate::c13_promotion_apply::{
    PersonEffect, PersonHistoryEvent, PromotionApplyEffects,
    PromotionWelcomeNews, RelegationApplyEffects,
    RelegationNoLeagueNews,
};
use crate::c14_stadium_expansion::{
    StadiumExpansionOutcome, NEWS_TEMPLATE_STADIUM_EXPANSION,
};
use crate::c15_english_annual_rollover::{
    materialise_club_moves, AnnualRolloverReport,
    YearEndMutationEvent,
};
use crate::{DomainStadium, GameDate, RuntimeEvent, RuntimeSaveGame,
             World};

// ---------------------------------------------------------------------------
// Public surface
// ---------------------------------------------------------------------------

/// One entry per stadium the applier mutated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedStadiumWrite {
    pub stadium_id: u32,
    pub new_total: u32,
    pub new_seated: u32,
    pub new_peak: u32,
}

/// One club's persistent finance state — the materialised
/// destination of C14 stadium-expansion (and, in follow-ups, other
/// finance-cluster) writes.
///
/// C15.1A archaeology (`reports/c15_1a_finance_archaeology.md`)
/// proved these five fields are the FUN_00583FC0 targets, at these
/// widths, at these runtime object offsets. The RUNTIME OBJECT
/// identity (Club record vs a separate finance pool) is not yet
/// fully proven — that requires tracing `FUN_005121A0`'s loader
/// copy path. For now the ledger holds the semantically-correct
/// The canonical carrier for the exe's per-club Runtime Finance
/// record.
///
/// # Provenance (C15.1F archaeology — FROZEN)
///
/// The exe allocates one 0x167-byte (359-byte) record per club
/// in a dedicated heap pool, distinct from both the disk Club
/// record (`Club.raw`, stride 0x245) and from `FUN_005121A0`'s
/// database loader slabs. Construction site: `FUN_00584530`
/// (via `operator_new(DAT_00acd564 * 0x167 + 4)`) called at
/// new-game boot from `008120d0.c:1144` and at save-load from
/// `00814870.c:1413`. Pool base wrapper: `DAT_00acdc38` (Rust
/// counterpart: `RuntimeSaveGame.finance`, a `finance::FinanceBook`
/// whose `ClubFinance.balance` is this record's `+0x00` cash).
///
/// Resolver idiom (16 hits across the finance-cluster writers):
/// `pool_base + club_id * 0x167` (`FUN_0058A490` line 23). This
/// is why the ledger's map is keyed by `club_id`.
///
/// Per-record layout (bytes seen by decompile-proven writers):
///
/// | Runtime offset | Type | Semantic | Rust field |
/// | ---: | --- | --- | --- |
/// | `+0x00` | i64 | Cash | `cash` |
/// | `+0x08` | u32 | Mirrored `club_id` (parity check) | (not stored — key is BTreeMap key) |
/// | `+0x8C` | i32 | Season misc operating expense | `season_misc_expense` |
/// | `+0xB4` | i32 | Season subsidy income | `season_subsidy_income` |
/// | `+0x12C` | i32 | Lifetime misc operating expense | `lifetime_misc_expense` |
/// | `+0x154` | i32 | Lifetime subsidy income | `lifetime_subsidy_income` |
/// | `+0x14, +0x24, +0x2C, +0x34..+0x15C (24 more DWORDs) | i32 | Other accumulators (wages / TV / prize / gate / attendance / …) — NOT modelled in this tranche; future writers land here as they port | – |
/// | `+0x164, +0x165, +0x166` | u8×3 | Status / tickdown bytes | – |
///
/// # New-game seed
///
/// `FUN_005803D0` (per-club constructor called by
/// `FUN_00584530`) reads `Club+0x65` (the shipped disk cash i32)
/// and stores it, via `__ftol`, into runtime `+0x00` as i64.
/// The four accumulators are zero-initialised. `Club+0x65` is
/// **the seed and only the seed** — after boot it is dead data
/// on the disk record. See [`ClubFinanceState::from_disk_seed`].
///
/// # Serialisation
///
/// The exe persists the whole 0x167-byte pool as its own
/// `finance.dat` sub-file inside the `.sav` bundle
/// (`FUN_005854D0`). The Rust port persists the same
/// information via `RuntimeSaveGame.finance` (`FinanceBook`,
/// serde-derived on the containing struct).
///
/// # Scope of this tranche
///
/// The five fields listed above are the ONLY ones any of the
/// year-end / stadium-expansion writers currently touch (C15.1A
/// arithmetic is byte-exact against `FUN_00583FC0`,
/// `FUN_00587C40`, `FUN_00586EC0`, `FUN_00584790`,
/// `FUN_00585AE0`). Adding the other 24+ accumulator DWORDs is
/// deferred to later tranches that port those writers. Because
/// `serde` is derived and adds fields via `#[serde(default)]`,
/// extending this record later will not break existing saves.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq,
         serde::Serialize, serde::Deserialize)]
pub struct ClubFinanceState {
    /// Cash — i64 signed. Runtime object byte offset `+0x00`.
    /// Seeded at new-game boot from `ClubView::initial_cash_seed()`
    /// (disk `Club+0x65` i32, sign-extended to i64).
    pub cash: i64,
    /// Season misc operating expense — i32.
    /// Runtime byte `+0x8C`. Reset annually by season roll
    /// (`FUN_00585AE0` sets it back to 0).
    pub season_misc_expense: i32,
    /// Lifetime misc operating expense — i32.
    /// Runtime byte `+0x12C`. Never reset; monotone-increasing.
    pub lifetime_misc_expense: i32,
    /// Season subsidy income — i32. Runtime byte `+0xB4`.
    /// Reset annually by season roll.
    pub season_subsidy_income: i32,
    /// Lifetime subsidy income — i32. Runtime byte `+0x154`.
    /// Never reset; monotone-increasing.
    pub lifetime_subsidy_income: i32,
}

impl ClubFinanceState {
    /// New-game boot seed. Ports the effect of `FUN_005803D0`
    /// lines 47/96/100: read disk `Club+0x65` (i32 cash), widen
    /// to i64 via `__ftol`, store at runtime `+0x00`. Every
    /// accumulator starts at zero — the exe zero-inits the
    /// whole 0x167-byte record via `FUN_0093543F`'s ctor before
    /// this seed lands.
    pub fn from_disk_seed(disk_cash_seed: i32) -> Self {
        Self {
            cash: disk_cash_seed as i64,
            season_misc_expense: 0,
            lifetime_misc_expense: 0,
            season_subsidy_income: 0,
            lifetime_subsidy_income: 0,
        }
    }
}

/// Per-club finance ledger — a **value-type helper**, NOT runtime
/// state.
///
/// History: C15.1F promoted this to "the canonical carrier" and
/// persisted it as `RuntimeSaveGame.finance_ledger`, not noticing
/// that `finance.rs::FinanceBook` was already the live runtime
/// finance store (seeded at boot, mutated by the weekly-wage /
/// match-income / board / debt ticks). That left two finance
/// stores, and the year-end path read the one that was never
/// seeded in production. The persisted field is gone; production
/// reads/writes go through [`crate::finance::FinanceBook::year_end_state`]
/// and [`crate::finance::FinanceBook::apply_year_end_write`].
///
/// This type survives for the C15.1A/F arithmetic tests and for
/// building a `YearEndSnapshot` finance map (`per_club`) from a
/// hand-seeded state. Keyed by `club_id` like the exe's
/// same-ordinal `pool_base + club_id * 0x167`.
///
/// See [`ClubFinanceState`] for the layout provenance.
#[derive(Debug, Clone, Default, PartialEq, Eq,
         serde::Serialize, serde::Deserialize)]
pub struct ClubFinanceLedger {
    #[serde(default)]
    pub per_club: std::collections::BTreeMap<u32, ClubFinanceState>,
}

impl ClubFinanceLedger {
    pub fn new() -> Self { Self::default() }

    /// Read the club's current finance state (or default if absent).
    pub fn get(&self, club_id: u32) -> ClubFinanceState {
        self.per_club.get(&club_id).copied().unwrap_or_default()
    }

    /// C15.1F — new-game boot seed pass. Walks every club in
    /// the given `World.core.clubs` and seeds a
    /// `ClubFinanceState` from `ClubView::initial_cash_seed()`.
    /// Idempotent: seeding an already-seeded club overwrites
    /// the entry (matches the exe's `FUN_00584530` which
    /// unconditionally constructs the entire pool at boot).
    ///
    /// This is the analogue of the exe's `FUN_00584530 →
    /// FUN_005803D0` per-club constructor loop. Call once at
    /// new-game boot, after clubs are loaded and before any
    /// season tick begins.
    pub fn seed_from_world(&mut self, world: &crate::World) {
        use crate::typed_records::ClubView;
        self.per_club.clear();
        for club in world.core.clubs.iter() {
            let cv = ClubView::new(club);
            let id = cv.id() as u32;
            let seed = cv.initial_cash_seed();
            self.per_club.insert(id, ClubFinanceState::from_disk_seed(seed));
        }
    }

    /// Apply one finance write from C14 (via `PendingFinanceWrite`).
    /// C14 has already computed the NEW post-transaction values
    /// (cash after debit/subsidy, accumulators after add). The
    /// ledger just stores them.
    ///
    /// Note: an unseeded club (no entry yet) still gets its
    /// state written — the entry is created on demand. This
    /// preserves the C15.1A behaviour and lets callers apply
    /// writes without a preceding seed pass, at the cost of
    /// losing the "pre-write cash was the disk seed" invariant
    /// on that path. For a byte-exact new-game boot, call
    /// `seed_from_world` first.
    pub fn apply_write(&mut self, write: &PendingFinanceWrite) {
        let s = self.per_club.entry(write.club_id).or_default();
        s.cash = write.new_cash;
        s.season_misc_expense = write.new_season_misc_expense;
        s.lifetime_misc_expense = write.new_lifetime_misc_expense;
        s.season_subsidy_income = write.new_season_subsidy_income;
        s.lifetime_subsidy_income = write.new_lifetime_subsidy_income;
    }
}

/// Finance writes emitted by the apply layer for post-hoc trace
/// (both intent and result). After C15.1A, these are materialised
/// into `RuntimeSaveGame.finance` (`FinanceBook`); the pending vector on
/// `WorldApplyReport` remains as diagnostic evidence that a
/// finance transaction fired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingFinanceWrite {
    pub club_id: u32,
    /// Cash (i64 signed, runtime object byte `+0x00`).
    pub new_cash: i64,
    /// Season misc operating expense (i32, byte `+0x8C`).
    pub new_season_misc_expense: i32,
    /// Lifetime misc operating expense (i32, byte `+0x12C`).
    pub new_lifetime_misc_expense: i32,
    /// Season subsidy income (i32, byte `+0xB4`).
    pub new_season_subsidy_income: i32,
    /// Lifetime subsidy income (i32, byte `+0x154`).
    pub new_lifetime_subsidy_income: i32,
}

/// Squad-position writes surfaced but not applied — the
/// `Club+0xD7` slot array is a raw byte region that flows through
/// the shipped staff pool; the applier defers per-slot writes
/// pending a typed accessor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSquadReset {
    pub person_id: u32,
    pub club_id: u32,
    /// Byte to write at `SquadRecord+0x3A`.
    pub new_role_pref: i8,
}

/// C15.1B applied contract-record byte write.
///
/// Emitted by the promotion / relegation apply pass when the
/// corresponding `ContractRecord` field actually mutates.
/// Records the byte offset (0x1C or 0x1F), the OLD value, the
/// NEW value, and a semantic tag. Trace-derived-from-state:
/// the record is pushed only after the write successfully lands.
///
/// Runtime object: the exe's 0x50-byte staff-employment /
/// contract record at `DAT_00accad8` (per memory
/// `[[contract-clauses-generated-at-boot]]`), indexed via
/// `DAT_00acdf0c[person_id]`. Rust storage:
/// `world.contracts.records[idx]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedContractWrite {
    pub person_id: u32,
    pub offset: u8,
    pub old_value: u8,
    pub new_value: u8,
    pub kind: ContractWriteKind,
}

/// Which contract byte was written and by which lifecycle event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractWriteKind {
    /// `+0x1F` relegation clause: `1 → 0` (promotion cleared the
    /// pending-relegation tag).
    RelegationClauseDisarmed,
    /// `+0x1F`: `1 → 2` (club actually got relegated; clause
    /// tripped; history event follows).
    RelegationClauseTripped,
    /// `+0x1C` non-promotion clause: `1 → 0` (promotion cleared
    /// the non-promotion tag).
    NonPromotionClauseDisarmed,
}

/// C15.1C — one `SquadRecord+0x3A` (squad-registration position
/// code) write that actually landed on a `ContractRecord`.
///
/// The exe stores this byte on the same 0x50-byte pool at
/// `DAT_00accad8` used by C15.1B (see memory
/// `[[contract-clauses-generated-at-boot]]`). `FUN_00843970`
/// resolves the target record via one of two indexes
/// (`FUN_004D59D0` primary, `FUN_004D5B00` secondary) and gates
/// the write on `record.club_id == club_id`. Promotion
/// (`FUN_004D3550` Loop A) walks the promoted club's 50 own
/// squad slots and calls `FUN_00843970(person, club, 0)` for each
/// occupant — resetting the position code to 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedSquadPreferenceWrite {
    pub person_id: u32,
    pub record_slot: SquadRecordSlot,
    pub old_value: i8,
    pub new_value: i8,
}

/// C15.1E — one persistent per-person news-mailbox append that
/// actually landed on `world.person_news_mailboxes`.
///
/// Runtime object: the exe's per-person mailbox descriptor
/// table at `DAT_00ACD5C4 + person_id * 0x6E`, indirected via
/// `+0xCF` into the shared 222-byte news-item pool
/// (`FUN_0076DCE0`). The Rust port collapses that two-hop
/// indirection into a `BTreeMap<person_id, Vec<PersonNewsItem>>`
/// keyed by the same `person_id` the exe uses to reach the
/// mailbox descriptor. See
/// `reports/c15_1e_history_archaeology.md` for the derivation
/// and for why the previously-assumed "13-list-per-person"
/// model is refuted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppliedPersonHistoryWrite {
    /// Subject person id (slot 0 of the appended news item;
    /// also the mailbox key).
    pub person_id: u32,
    /// News category (always `0x0FBF` for the two `FUN_008D0D90`
    /// callers). Held as a field rather than a constant so
    /// consumers can filter without cross-referencing).
    pub category: u32,
    /// `kind` value (`param_3` on the caller side; stored to
    /// slot 6 of the news item). 3 = relegated (this tranche's
    /// only wired caller), 0 = retirement (out of scope).
    pub kind: u8,
    /// Old-competition id (slot 5). Comes from the C13
    /// `PersonHistoryEvent.old_comp_id`.
    pub old_comp_id: u32,
    /// Staff row id (slot 7). Resolved by the applier via
    /// `ContractPool.contract_for_staff_mut(person_id)`.`staff_id`
    /// — mirrors the exe's `*(u32*)(param_4 + 0x21)` read on
    /// the contract row `param_4` points to.
    pub staff_id: u32,
    /// Mailbox length BEFORE the append.
    pub old_len: usize,
    /// Mailbox length AFTER the append (always `old_len + 1`).
    pub new_len: usize,
    /// Monotonic news id the pool assigned to this item.
    pub news_id: u32,
}

/// C15.1D — one `Stadium+0x20` owner-refuse counter increment
/// that actually landed on a `DomainStadium`.
///
/// The exe writes this byte inside `FUN_00583FC0` line 125
/// (`0055ee90.c` / `0055ea00.c` affordability-checked path)
/// when an owner-backed stadium expansion is refused because
/// the parent club can't cover the cost via its subsidy chain.
/// The write is a plain `+= 1`, saturating via a `< 0x14`
/// (i.e. `< 20`) guard: once the byte reaches 20, no further
/// increment ever fires.
///
/// Runtime object: `Stadium+0x20`, a byte on the parent's
/// stadium record — reached via the owner-parent-club pointer
/// chain `Club[+0xBF] → Stadium[+0x69] → +0x20`. This is a
/// runtime-only field (not part of the shipped 78-byte
/// `stadium.dat` layout), stored in
/// `DomainStadium.owner_refuse_counter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppliedRefuseCounterWrite {
    /// The parent stadium id that received the write. Refers to
    /// `DomainStadium.id` on `world.references.stadiums`.
    pub stadium_id: u32,
    /// Value BEFORE the increment. Always `< 20` — a write with
    /// `old == 20` is skipped and not traced.
    pub old_value: i8,
    /// Value AFTER the increment. Always `old + 1`.
    pub new_value: i8,
}

/// Which of the two contract-record indexes hit for a given
/// `FUN_00843970` call. Primary is tried first and short-circuits
/// on identity match; Secondary is only consulted when primary
/// returned null or its identity gate failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquadRecordSlot {
    /// `FUN_004D59D0` — the primary staff→contract index
    /// (`ContractPool.by_staff_id`).
    Primary,
    /// `FUN_004D5B00` — the secondary index
    /// (`ContractPool.by_staff_id_secondary`), tried only on
    /// primary failure.
    Secondary,
}

/// One person-history entry queued by the applier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedPersonHistory {
    pub person_id: u32,
    pub competition_id: u32,
    pub year: u16,
    pub kind: String,
}

/// One news event queued by the applier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedNews {
    pub template_id: u16,
    pub club_id: Option<u32>,
    pub competition_id: Option<u32>,
    pub kind: String,
}

/// Full applier trace. The rule: this record is derived from what
/// actually landed on World, not what the intent-only C15 report
/// described.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldApplyReport {
    pub club_move_writes: usize,
    pub stadium_writes: Vec<AppliedStadiumWrite>,
    pub news_writes: Vec<AppliedNews>,
    pub person_history: Vec<AppliedPersonHistory>,
    pub pending_finance: Vec<PendingFinanceWrite>,
    pub pending_squad_resets: Vec<PendingSquadReset>,
    /// C15.1B — contract-record byte writes that actually
    /// mutated a `ContractRecord`. Empty if `world.contracts` was
    /// `None` (contract pool not initialised).
    pub applied_contract_writes: Vec<AppliedContractWrite>,
    /// C15.1C — squad-registration position-code (`+0x3A`) writes
    /// that actually mutated a `ContractRecord`. Empty if
    /// `world.contracts` was `None`, or if no promotion event
    /// touched any resolvable record.
    pub applied_squad_preference_writes:
        Vec<AppliedSquadPreferenceWrite>,
    /// C15.1D — parent-stadium `+0x20` owner-refuse counter
    /// increments that actually landed on a `DomainStadium`.
    /// Empty when no stadium-expansion event carried
    /// `refuse_counter_increment == true`, when
    /// `parent_stadium_id` was `None`, when the parent stadium
    /// was not found in `world.references.stadiums`, or when
    /// the counter was already saturated at 20.
    pub applied_refuse_counter_writes:
        Vec<AppliedRefuseCounterWrite>,
    /// C15.1E — per-person news-mailbox appends that actually
    /// landed on `world.person_news_mailboxes`. Empty when no
    /// relegation event carried a `PersonHistoryEvent`, when
    /// the person's staff_id could not be resolved (no
    /// `ContractRecord`), or when the identity gate failed.
    pub applied_person_history_writes:
        Vec<AppliedPersonHistoryWrite>,
    /// Post-rollover status per moved club, taken from the raw
    /// bytes AFTER `materialise_club_moves` writes them.
    pub post_rollover_club_status: std::collections::BTreeMap<u32, u8>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Apply the C15 report onto World + RuntimeSaveGame. Emits a
/// [`WorldApplyReport`] describing every mutation that actually
/// landed.
///
/// # Ordering
///
/// Per the exe's `FUN_00668380` / `FUN_00668470` flow:
///
/// 1. C13 effects for each mover (person effects, history events,
///    news, stadium requests).
/// 2. C14 stadium capacity write for that mover, if requested.
/// 3. Outer swap status `+0x37 = 0xFF` write (materialised last so
///    downstream helpers still see the pre-consumed status).
///
/// The C15 report already produces events in that order; the
/// applier walks the event stream in the received order and
/// preserves it in the queue.
pub fn apply_report_to_world(
    world: &mut World,
    save: &mut RuntimeSaveGame,
    report: &AnnualRolloverReport,
) -> WorldApplyReport {
    let date = save.date.clone();
    let day = save.elapsed_days;
    let mut out = apply_report_to_world_parts(
        world, &mut save.pending_events, &date, day, report,
    );
    // C15.1A: materialise the finance writes surfaced by the
    // apply pass into the ONE runtime finance store
    // (`RuntimeSaveGame.finance`, a `FinanceBook`). Trace-derived:
    // PendingFinanceWrite records what C14 produced; the book
    // reflects what actually landed.
    for w in &out.pending_finance {
        save.finance.apply_year_end_write(w);
    }
    out
}

/// C15.1B — walk the AnnualRolloverReport's Promotion /
/// Relegation events, look up each person_id's contract record
/// in the ContractPool, and materialise the observed byte writes:
///
/// * Promotion: `+0x1F: 1 → 0`, `+0x1C: 1 → 0`
///   (both are independent predicates; both may fire on one
///   record)
/// * Relegation: `+0x1F: 1 → 2` (never touches `+0x1C`)
///
/// Predicate matches the exe: the current byte must equal `1`.
/// If the byte is `0` or `2`, no write happens (matches the
/// silent-skip semantics of `FUN_004D3550` / `FUN_004D3460`).
///
/// See `reports/c15_1b_staff_archaeology.md` for the full
/// derivation.
pub fn apply_contract_writes_from_report(
    contracts: &mut crate::contract_init::ContractPool,
    report: &AnnualRolloverReport,
    out: &mut WorldApplyReport,
) {
    for ev in &report.events {
        match ev {
            YearEndMutationEvent::Promotion { effects } => {
                for pe in &effects.person_effects {
                    apply_contract_write_for_person(
                        contracts, pe, /*is_promotion=*/true,
                        effects.club_id, out,
                    );
                }
            }
            YearEndMutationEvent::Relegation { effects } => {
                for pe in &effects.person_effects {
                    apply_contract_write_for_person(
                        contracts, pe, /*is_promotion=*/false,
                        effects.club_id, out,
                    );
                }
            }
            _ => {}
        }
    }
}

fn apply_contract_write_for_person(
    contracts: &mut crate::contract_init::ContractPool,
    pe: &PersonEffect,
    is_promotion: bool,
    club_id: u32,
    out: &mut WorldApplyReport,
) {
    // Chain: person_id → contract idx via by_staff_id → record.
    // Note: the exe uses staff_id here (`+0x00` on the record),
    // resolved via `DAT_00acdf0c[person_id * 0x4F]`. The port's
    // `person_id` on `PersonEffect` corresponds to that lookup
    // key.
    let Some(record) = contracts.contract_for_staff_mut(pe.person_id)
        else { return };
    // C15.1C identity gate (retroactively wired into C15.1B):
    // exe checks `*(record+4) == *club` in both FUN_004D3550 and
    // FUN_004D3460. Rust: record.club_id == club_id.
    if record.club_id != club_id as i32 { return; }
    if is_promotion {
        // Promotion: independent 1→0 clears on both bytes.
        //
        // Exe order (004d3550.c lines 45-47 then 48-50):
        // check +0x1F first, then +0x1C. Preserve that ordering.
        if pe.new_staff_1f == Some(0) && record.relegation == 1 {
            let old = record.relegation;
            record.relegation = 0;
            out.applied_contract_writes.push(AppliedContractWrite {
                person_id: pe.person_id,
                offset: 0x1F,
                old_value: old,
                new_value: 0,
                kind: ContractWriteKind::RelegationClauseDisarmed,
            });
        }
        if pe.new_staff_1c == Some(0) && record.non_promotion == 1 {
            let old = record.non_promotion;
            record.non_promotion = 0;
            out.applied_contract_writes.push(AppliedContractWrite {
                person_id: pe.person_id,
                offset: 0x1C,
                old_value: old,
                new_value: 0,
                kind: ContractWriteKind::NonPromotionClauseDisarmed,
            });
        }
    } else {
        // Relegation: 1→2 on +0x1F only. Write BEFORE history
        // helper fires (matches 004d3460.c lines 34-35 order).
        // The history event was already queued by
        // `materialise_person_effect`; we intentionally do not
        // reorder that here — the tranche's ordering boundary
        // means we preserve the write-then-history invariant
        // as an EMIT ordering, which downstream consumers observe
        // via the applied_contract_writes vector landing before
        // the pending_events entry (they were pushed in that
        // order by their respective code paths).
        if pe.new_staff_1f == Some(2) && record.relegation == 1 {
            let old = record.relegation;
            record.relegation = 2;
            out.applied_contract_writes.push(AppliedContractWrite {
                person_id: pe.person_id,
                offset: 0x1F,
                old_value: old,
                new_value: 2,
                kind: ContractWriteKind::RelegationClauseTripped,
            });
        }
    }
}

/// C15.1C — walk the report's Promotion events and materialise
/// the `SquadRecord+0x3A` position-code resets onto the runtime
/// contract pool.
///
/// The exe's promotion path is `FUN_004D3550` Loop A: for each of
/// the promoted club's 50 own squad slots (offset `+0xd7` on the
/// club record), if the slot is occupied, call
/// `FUN_00843970(person, club, 0)`. That helper then:
///
/// 1. Range-gates `param_3` to `[-0x32, +0x32]` (silent skip on
///    out-of-range; the exe additionally pops an Error dialog
///    but sets `DAT_00b4d5a8 = 0` and returns).
/// 2. Resolves the primary contract record via `FUN_004D59D0`.
///    If nonzero AND `*(record+4) == *club` (identity match),
///    write `*(record+0x3A) = param_3` and RETURN — the primary
///    short-circuit.
/// 3. Otherwise consult the secondary index via `FUN_004D5B00`.
///    If nonzero AND identity match, write and return; else
///    silent skip.
///
/// This port covers the promotion-side reset (`param_3 == 0`).
/// The Rust API is deliberately parameterised over `new_value`
/// so future callers (transfer window, editor) can share the
/// same primary-then-secondary write path.
///
/// **Scope note (from the tranche directive).** Loop A on the
/// promoted club iterates 50 squad slots on that club record.
/// The Rust port here iterates `effects.person_effects` — the
/// same person set C15.1B walks. This is intentionally
/// self-only: the Loop-B second pass (which walks
/// `FUN_0052a5a0(club, 0, 1)` — the affiliate/reserve club) is
/// NOT reproduced here for the `+0x3A` write, because Loop A is
/// gated on `param_3 != 0` on the exe side and only runs on the
/// promoted club itself. C15.1B tests confirmed
/// `person_effects` is the right event carrier for per-person
/// walks in this pipeline.
///
/// **Nation-based identity check deferred.** `FUN_00843970` has
/// a fallback identity check
/// `DAT_00acd5bc + record.club_id * 0x245 == FUN_0052a5a0(club, 0, 1)`
/// which lets a person owned by an affiliate club still match.
/// The Rust port uses only the direct `club_id == club_id`
/// check. If a record fails direct identity, it is treated as a
/// mismatch (silent skip after primary → try secondary). See
/// `reports/c15_1c_squad_archaeology.md` for the derivation.
pub fn apply_squad_position_writes_from_report(
    contracts: &mut crate::contract_init::ContractPool,
    report: &AnnualRolloverReport,
    out: &mut WorldApplyReport,
) {
    for ev in &report.events {
        if let YearEndMutationEvent::Promotion { effects } = ev {
            for pe in &effects.person_effects {
                write_squad_position(
                    contracts, pe.person_id, effects.club_id,
                    /*new_value=*/0, out,
                );
            }
        }
    }
}

/// Byte-exact port of `FUN_00843970`: range gate → primary
/// resolve+identity+write → secondary resolve+identity+write.
///
/// Returns silently on any gate failure; mutates the pool only
/// when a write actually lands.
fn write_squad_position(
    contracts: &mut crate::contract_init::ContractPool,
    person_id: u32,
    club_id: u32,
    new_value: i8,
    out: &mut WorldApplyReport,
) {
    // Range gate: exe checks `param_3 < -0x32 || 0x32 < param_3`.
    // Rust: strict inclusive `[-50, +50]`.
    if new_value < -50 || new_value > 50 { return; }
    // Primary resolver — FUN_004D59D0.
    // Direct identity: record.club_id == club_id.
    let primary_hit_or_mismatch = {
        if let Some(rec) = contracts.contract_for_staff_mut(person_id) {
            if rec.club_id == club_id as i32 {
                let old = rec.position_code;
                if old != new_value {
                    rec.position_code = new_value;
                    out.applied_squad_preference_writes.push(
                        AppliedSquadPreferenceWrite {
                            person_id,
                            record_slot: SquadRecordSlot::Primary,
                            old_value: old,
                            new_value,
                        },
                    );
                } else {
                    // Idempotent: the exe still writes the same
                    // byte, but we skip the trace entry to keep
                    // the applier report a mutation log (matches
                    // how C15.1B treats already-cleared bytes).
                }
                return; // primary short-circuit
            }
            // Primary resolved but identity mismatch — fall
            // through to secondary. Matches exe control flow.
            true
        } else {
            // Primary null — fall through to secondary.
            false
        }
    };
    let _ = primary_hit_or_mismatch; // retained for readability
    // Secondary resolver — FUN_004D5B00.
    if let Some(rec) =
        contracts.contract_for_staff_secondary_mut(person_id)
    {
        if rec.club_id == club_id as i32 {
            let old = rec.position_code;
            if old != new_value {
                rec.position_code = new_value;
                out.applied_squad_preference_writes.push(
                    AppliedSquadPreferenceWrite {
                        person_id,
                        record_slot: SquadRecordSlot::Secondary,
                        old_value: old,
                        new_value,
                    },
                );
            }
        }
    }
}

/// C15.1E — walk the report's Relegation events and append a
/// persistent news-mailbox entry per person with an
/// `event_emit` payload, mirroring
/// `FUN_004D3460 → FUN_008D0D90(slot, old_comp, 3, contract_row)`.
///
/// # Ordering
///
/// This pass runs AFTER `apply_contract_writes_from_report`
/// (C15.1B). The exe order inside `FUN_004D3460` lines 34-35 is
/// `contract[+0x1F] = 2; FUN_008D0D90(...);` — contract byte
/// write first, then news append. `FUN_008D0D90` does NOT read
/// the just-written `+0x1F` byte (only `+0x21` = staff_id), so
/// the ordering is behaviourally moot at the byte level — but
/// this port preserves it anyway.
///
/// # Identity gate
///
/// Consistent with C15.1B, a person whose contract is not
/// found in the pool (or whose contract's club_id does not
/// match the event's club_id) produces no mailbox append. The
/// exe reaches this write via a squad-slot walk on the
/// relegated club, so the identity gate is a natural
/// consequence of iterating that club's occupants.
pub fn apply_person_history_from_report(
    world: &mut World,
    report: &AnnualRolloverReport,
    year: u16,
    out: &mut WorldApplyReport,
) {
    for ev in &report.events {
        if let YearEndMutationEvent::Relegation { effects } = ev {
            for pe in &effects.person_effects {
                if let Some(h) = &pe.event_emit {
                    append_person_history_entry(
                        world, pe.person_id, effects.club_id,
                        h.old_comp_id, h.kind, year, out,
                    );
                }
            }
        }
    }
}

/// Byte-image port of `FUN_008D0D90`'s effect on the news
/// mailbox pool. Resolves `staff_id` via the current contract
/// pool (populated by C15.1B before this pass) and appends a
/// [`PersonNewsItem`] to the person's mailbox.
///
/// Silent-skips on:
/// * No `ContractPool` (a World with no contract data was
///   still valid before this tranche existed).
/// * `person_id` not resolvable via `by_staff_id` (matches the
///   exe's early return on a null resolver).
/// * Contract's `club_id` mismatches the event's `club_id`.
///   Mirrors the identity gate `*(record+4) == *param_2` the
///   exe applies in the surrounding squad-slot walk.
fn append_person_history_entry(
    world: &mut World,
    person_id: u32,
    club_id: u32,
    old_comp_id: u32,
    kind: u8,
    year: u16,
    out: &mut WorldApplyReport,
) {
    let staff_id = {
        let Some(contracts) = world.contracts.as_ref() else { return };
        let Some(record) = contracts.contract_for_staff(person_id)
            else { return };
        if record.club_id != club_id as i32 { return; }
        record.staff_id as u32
    };
    let item = crate::person_news::PersonNewsItem {
        category: crate::person_news::NEWS_CATEGORY_PERSON_CAREER,
        severity: 0,
        params: [
            person_id,          // slot 0 — subject person
            0, 0, 0,            // slots 1..=3 (pointer chains not modelled)
            0,                  // slot 4 (club/nation base id)
            old_comp_id,        // slot 5
            kind as u32,        // slot 6
            staff_id,           // slot 7
        ],
        year,
        news_id: 0,             // filled by pool.push
    };
    let receipt = world.person_news_mailboxes.push(person_id, item);
    // Read back the pool-assigned monotonic id from the last
    // slot we just wrote.
    let news_id = world.person_news_mailboxes
        .mailbox_for(person_id)
        .last()
        .map(|it| it.news_id)
        .unwrap_or(0);
    out.applied_person_history_writes.push(
        AppliedPersonHistoryWrite {
            person_id,
            category: crate::person_news::NEWS_CATEGORY_PERSON_CAREER,
            kind,
            old_comp_id,
            staff_id,
            old_len: receipt.old_len,
            new_len: receipt.new_len,
            news_id,
        },
    );
}

/// Lower-level entry: mutate World + a supplied event queue,
/// tagged with a specific date + elapsed-days count. Useful in
/// tests where standing up a full `RuntimeSaveGame` would be
/// costly.
pub fn apply_report_to_world_parts(
    world: &mut World,
    pending_events: &mut Vec<RuntimeEvent>,
    date: &GameDate,
    day: u32,
    report: &AnnualRolloverReport,
) -> WorldApplyReport {
    let mut out = WorldApplyReport::default();
    let year = date.year;
    let date = date.clone();

    // ---- Walk the trace in order, materialising as we go --------------
    // Where a variant has cascading effects (Promotion/Relegation), we
    // apply the sub-effects here before the SwapStatusIdle event.
    for ev in &report.events {
        match ev {
            YearEndMutationEvent::StatusStamped { .. }
            | YearEndMutationEvent::PlayoffWinnerStamp { .. }
            | YearEndMutationEvent::SwapStatusIdle { .. }
            | YearEndMutationEvent::ConferenceDispatch { .. } => {
                // Status writes are handled once at the end by
                // `materialise_club_moves` — the C15 report's per-row
                // `club_moves` map already reflects the final `+0x37`
                // byte per moved club, so re-applying it here would
                // double-write.
            }
            YearEndMutationEvent::Promotion { effects } => {
                apply_promotion_effects(
                    pending_events, &date, day, effects, &mut out,
                );
            }
            YearEndMutationEvent::Relegation { effects } => {
                apply_relegation_effects(
                    pending_events, &date, day, year, effects, &mut out,
                );
            }
            YearEndMutationEvent::StadiumExpansion {
                outcome, club_id, stadium_id, parent_stadium_id,
            } => {
                apply_stadium_expansion_outcome(
                    world, pending_events, &date, day, outcome,
                    *club_id, *stadium_id, *parent_stadium_id,
                    &mut out,
                );
            }
            YearEndMutationEvent::StadiumFailReprieve {
                third_div_bottom_club_id: _,
                candidate_club_id,
                news_template_id,
                news_destination_comp_id,
            } => {
                let msg = format!(
                    "year-end: Conference stadium-fail reprieve (template {:#06x}, comp {})",
                    news_template_id, news_destination_comp_id,
                );
                pending_events.push(RuntimeEvent {
                    day, date: date.clone(),
                    kind: "year_end_stadium_fail_reprieve".to_string(),
                    message: msg,
                    phase: 2,
                });
                out.news_writes.push(AppliedNews {
                    template_id: *news_template_id,
                    club_id: if *candidate_club_id != 0 {
                        Some(*candidate_club_id)
                    } else { None },
                    competition_id: Some(*news_destination_comp_id),
                    kind: "stadium_fail_reprieve".to_string(),
                });
                // The reprieve's `+0x37 = 0xFE` write is already in
                // `report.club_moves` if the C8 path emitted it as a
                // ClubMoveSummary; the Conference-fallback path
                // writes it directly via the club_moves map.
            }
        }
    }

    // ---- Core club movement (Club+0x57, +0x5B, +0x37) ----------------
    // C15 already computed `club_moves`; apply now.
    out.club_move_writes = materialise_club_moves(world, &report.club_moves);

    // Snapshot the post-write status on every moved club, so tests
    // and downstream diagnostics have derived-from-state data.
    use crate::typed_records::ClubView;
    for club in world.core.clubs.iter() {
        let id = ClubView::new(club).id();
        if report.club_moves.contains_key(&id) && club.raw.len() > 0x37 {
            out.post_rollover_club_status.insert(id, club.raw[0x37]);
        }
    }

    // ---- C15.1B / C15.1C / C15.1E — persistent-state passes ---------
    //
    // Run against `world.contracts` (C15.1B / C15.1C) and
    // `world.person_news_mailboxes` (C15.1E). All three passes
    // key on the same event stream we just walked above and
    // land on the same World; running them here from
    // `apply_report_to_world_parts` means both the full
    // `apply_report_to_world` entry AND direct callers share
    // one canonical pipeline order:
    //
    //     C15.1B contract writes  (byte-write on contract row)
    //     C15.1C squad position   (byte-write on contract row)
    //     C15.1E history append   (mailbox entry keyed by
    //                              already-written staff_id)
    //
    // This matches the exe order (contract byte then news
    // append inside FUN_004D3460, disjoint offsets in
    // FUN_004D3550), and gives C15.1E the C15.1B-updated
    // contract state to read `staff_id` from — same as the
    // exe's `FUN_008D0D90(param_4 = contract_row)` call.
    if let Some(contracts) = world.contracts.as_mut() {
        apply_contract_writes_from_report(contracts, report, &mut out);
        apply_squad_position_writes_from_report(contracts, report, &mut out);
    }
    apply_person_history_from_report(world, report, date.year, &mut out);

    out
}

// ---------------------------------------------------------------------------
// Sub-effect materialisers
// ---------------------------------------------------------------------------

fn apply_promotion_effects(
    pending_events: &mut Vec<RuntimeEvent>,
    date: &GameDate,
    day: u32,
    effects: &PromotionApplyEffects,
    out: &mut WorldApplyReport,
) {
    // Person walk: history + squad-reset surfacing.
    let club_id = effects.club_id;
    for pe in &effects.person_effects {
        materialise_person_effect(pending_events, date, day, pe, club_id, out);
    }
    // Welcome news.
    if let Some(w) = &effects.welcome_news {
        queue_promotion_welcome(pending_events, date, day, w, out);
    }
    // Stadium expansion request lives in effects but the
    // materialisation happens via the accompanying
    // `StadiumExpansion` event — deliberately not applied here to
    // avoid double-write.
}

fn apply_relegation_effects(
    pending_events: &mut Vec<RuntimeEvent>,
    date: &GameDate,
    day: u32,
    _year: u16,
    effects: &RelegationApplyEffects,
    out: &mut WorldApplyReport,
) {
    let club_id = effects.club_id;
    for pe in &effects.person_effects {
        materialise_person_effect(pending_events, date, day, pe, club_id, out);
    }
    if let Some(n) = &effects.no_league_news {
        queue_no_league(pending_events, date, day, n, out);
    }
}

fn materialise_person_effect(
    pending_events: &mut Vec<RuntimeEvent>,
    date: &GameDate,
    day: u32,
    pe: &PersonEffect,
    club_id: u32,
    out: &mut WorldApplyReport,
) {
    // Person history event (0 or 1 per person, per C13 apply).
    if let Some(h) = &pe.event_emit {
        let kind = describe_history_kind(h.kind);
        pending_events.push(RuntimeEvent {
            day,
            date: date.clone(),
            kind: format!("person_history_{}", kind),
            message: format!(
                "person #{} year-end {} (old comp {})",
                pe.person_id, kind, h.old_comp_id,
            ),
            phase: 2,
        });
        out.person_history.push(AppliedPersonHistory {
            person_id: pe.person_id,
            competition_id: h.old_comp_id,
            year: date.year,
            kind,
        });
    }
    // Squad-position reset — C14.5 established that promotion
    // clears `SquadRecord+0x3A` on eligible slots. The C13
    // envelope currently exposes staff-flag writes rather than a
    // dedicated squad_reset flag; when `new_staff_1f = 0` fires
    // on the promotion side (register slot), the eligible-slot
    // reset is downstream. Surface it as pending; the applier
    // does not touch the raw staff pool.
    if pe.new_staff_1f.is_some() || pe.new_staff_1c.is_some() {
        out.pending_squad_resets.push(PendingSquadReset {
            person_id: pe.person_id,
            club_id,
            new_role_pref: 0,
        });
    }
}

fn describe_history_kind(kind: u8) -> String {
    match kind {
        3 => "relegated".into(),
        _ => format!("kind_{}", kind),
    }
}

fn queue_promotion_welcome(
    pending_events: &mut Vec<RuntimeEvent>,
    date: &GameDate,
    day: u32,
    w: &PromotionWelcomeNews,
    out: &mut WorldApplyReport,
) {
    pending_events.push(RuntimeEvent {
        day,
        date: date.clone(),
        kind: "promotion_welcome".to_string(),
        message: format!(
            "welcome news: club {} promoted into comp {}",
            w.club_id, w.new_comp_id,
        ),
        phase: 2,
    });
    out.news_writes.push(AppliedNews {
        template_id: 0, // promotion welcome uses a non-templated pathway in the exe
        club_id: Some(w.club_id),
        competition_id: Some(w.new_comp_id),
        kind: "promotion_welcome".to_string(),
    });
}

fn queue_no_league(
    pending_events: &mut Vec<RuntimeEvent>,
    date: &GameDate,
    day: u32,
    n: &RelegationNoLeagueNews,
    out: &mut WorldApplyReport,
) {
    pending_events.push(RuntimeEvent {
        day,
        date: date.clone(),
        kind: "relegation_no_league".to_string(),
        message: format!(
            "no-league relegation: club {} manager unhappy",
            n.club_id,
        ),
        phase: 2,
    });
    out.news_writes.push(AppliedNews {
        template_id: 7, // FUN_004938d0 uses id 7 on the no-league branch
        club_id: Some(n.club_id),
        competition_id: None,
        kind: "relegation_no_league".to_string(),
    });
}

fn apply_stadium_expansion_outcome(
    world: &mut World,
    pending_events: &mut Vec<RuntimeEvent>,
    date: &GameDate,
    day: u32,
    outcome: &StadiumExpansionOutcome,
    club_id_hint: u32,
    stadium_id_hint: Option<u32>,
    parent_stadium_id: Option<u32>,
    out: &mut WorldApplyReport,
) {
    // News is emitted whether the transaction succeeded or not
    // (per FUN_0058A310).
    if let Some(n) = &outcome.news_event {
        let kind = if outcome.success {
            "stadium_expansion"
        } else {
            "stadium_expansion_failed"
        };
        pending_events.push(RuntimeEvent {
            day,
            date: date.clone(),
            kind: kind.to_string(),
            message: format!(
                "{} (club {}, template {:#06x}, comp {}, mode {})",
                kind, club_id_hint, n.template_id, n.comp_id, n.mode,
            ),
            phase: 2,
        });
        out.news_writes.push(AppliedNews {
            template_id: n.template_id,
            club_id: if club_id_hint != 0 { Some(club_id_hint) } else { None },
            competition_id: Some(n.comp_id),
            kind: kind.to_string(),
        });
        debug_assert_eq!(n.template_id, NEWS_TEMPLATE_STADIUM_EXPANSION);
    }

    // Finance always surfaces as pending regardless of success.
    if let Some(cw) = &outcome.club_writes {
        out.pending_finance.push(PendingFinanceWrite {
            club_id: club_id_hint,
            new_cash: cw.new_cash,
            new_season_misc_expense: cw.new_season_misc_expense,
            new_lifetime_misc_expense: cw.new_lifetime_misc_expense,
            new_season_subsidy_income: cw.new_season_subsidy_income,
            new_lifetime_subsidy_income: cw.new_lifetime_subsidy_income,
        });
    }

    // C15.1D — parent owner-refuse counter bump. Fires
    // INDEPENDENT of `outcome.success` (in the exe the write is
    // in the failure branch of FUN_00583FC0 — line 125 — so
    // `success == false` is exactly when it can be true; the
    // C14 port also only sets the flag on that path, so the two
    // are consistent). The write is:
    //
    //   if refuse_counter_increment && parent_stadium.owner_refuse_counter < 20:
    //       parent_stadium.owner_refuse_counter += 1
    //
    // A silent skip on any of:
    //   * `refuse_counter_increment == false`
    //   * `parent_stadium_id == None`
    //   * parent stadium not found in world.references.stadiums
    //   * counter already saturated (== 20). Matches the exe's
    //     `cVar1 < '\x14'` gate on line 124.
    if outcome.refuse_counter_increment {
        if let Some(psid) = parent_stadium_id {
            if let Some(parent_stadium) =
                find_stadium_mut(&mut world.references.stadiums, psid)
            {
                let old = parent_stadium.owner_refuse_counter;
                if old < 20 {
                    parent_stadium.owner_refuse_counter = old + 1;
                    out.applied_refuse_counter_writes.push(
                        AppliedRefuseCounterWrite {
                            stadium_id: psid,
                            old_value: old,
                            new_value: old + 1,
                        },
                    );
                }
            }
        }
    }

    // Stadium capacity writes only on success.
    if !outcome.success { return; }
    let Some(sw) = &outcome.stadium_writes else { return };
    let Some(sid) = stadium_id_hint else { return };
    if let Some(stadium) =
        find_stadium_mut(&mut world.references.stadiums, sid)
    {
        stadium.capacity_total = sw.new_total as u32;
        stadium.capacity_seated = sw.new_seated as u32;
        stadium.capacity_expansion = sw.new_peak as u32;
        out.stadium_writes.push(AppliedStadiumWrite {
            stadium_id: sid,
            new_total: sw.new_total as u32,
            new_seated: sw.new_seated as u32,
            new_peak: sw.new_peak as u32,
        });
    }
}

fn find_stadium_mut(
    stadiums: &mut [DomainStadium],
    id: u32,
) -> Option<&mut DomainStadium> {
    stadiums.iter_mut().find(|s| s.id == id)
}

// ---------------------------------------------------------------------------
// Post-rollover snapshot for differential comparison
// ---------------------------------------------------------------------------

/// Deterministic semantic snapshot of the English year-end state,
/// so a Rust run can be compared byte-for-byte against a captured
/// GDI run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnglishYearEndSnapshot {
    pub year: u16,
    pub clubs: Vec<ClubSnapshot>,
    pub stadiums: Vec<StadiumSnapshot>,
    pub news_kinds: Vec<String>,
    pub person_history: Vec<AppliedPersonHistory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClubSnapshot {
    pub club_id: u32,
    pub primary_comp: i32,
    pub previous_comp: i32,
    pub status_37: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StadiumSnapshot {
    pub stadium_id: u32,
    pub total: u32,
    pub seated: u32,
    pub peak: u32,
}

/// Extract a snapshot from the current World state. Restricts to
/// the 5 English comps + their clubs + their stadiums.
pub fn snapshot_english_year_end(
    world: &World, year: u16, applier: &WorldApplyReport,
) -> EnglishYearEndSnapshot {
    use crate::typed_records::ClubView;
    const ENG: [i32; 5] = [7, 8, 9, 10, 93];
    let mut clubs = Vec::new();
    let mut stadium_ids: std::collections::BTreeSet<u32> = Default::default();
    for club in &world.core.clubs {
        if club.raw.len() < 0x60 { continue; }
        let view = ClubView::new(club);
        let primary = i32::from_le_bytes([
            club.raw[0x57], club.raw[0x58], club.raw[0x59], club.raw[0x5A],
        ]);
        if !ENG.contains(&primary) { continue; }
        let previous = i32::from_le_bytes([
            club.raw[0x5B], club.raw[0x5C], club.raw[0x5D], club.raw[0x5E],
        ]);
        clubs.push(ClubSnapshot {
            club_id: view.id() as u32,
            primary_comp: primary,
            previous_comp: previous,
            status_37: club.raw[0x37],
        });
        if club.raw.len() > 0x69 {
            let sid = club.raw[0x69] as u32;
            if sid > 0 { stadium_ids.insert(sid); }
        }
    }
    clubs.sort_by_key(|c| c.club_id);
    let mut stadiums = Vec::new();
    for s in &world.references.stadiums {
        if stadium_ids.contains(&s.id) {
            stadiums.push(StadiumSnapshot {
                stadium_id: s.id,
                total: s.capacity_total,
                seated: s.capacity_seated,
                peak: s.capacity_expansion,
            });
        }
    }
    stadiums.sort_by_key(|s| s.stadium_id);
    let news_kinds = applier.news_writes.iter()
        .map(|n| n.kind.clone()).collect();
    EnglishYearEndSnapshot {
        year,
        clubs,
        stadiums,
        news_kinds,
        person_history: applier.person_history.clone(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c13_promotion_apply::{
        ClubFieldWrites, PersonEffect, PersonHistoryEvent,
        PromotionApplyEffects, PromotionWelcomeNews,
        RelegationApplyEffects, RelegationNoLeagueNews,
    };
    use crate::c14_stadium_expansion::{
        ClubFinanceWrites, ReturnReason, StadiumExpansionNews,
        StadiumExpansionOutcome, StadiumWrites,
    };

    fn mk_promotion_effects(club_id: u32) -> PromotionApplyEffects {
        PromotionApplyEffects {
            club_id,
            writes: ClubFieldWrites {
                new_comp_id: 7, prev_comp_id: 8, tier_byte_64: None,
            },
            set_status_idle: true,
            person_effects: vec![
                PersonEffect {
                    person_id: 555,
                    new_staff_1f: Some(2),
                    new_staff_1c: None,
                    event_emit: Some(PersonHistoryEvent {
                        old_comp_id: 8, kind: 3,
                    }),
                },
            ],
            welcome_news: Some(PromotionWelcomeNews {
                club_id, new_comp_id: 7,
            }),
            stadium_expansion: None,
        }
    }

    fn mk_relegation_effects(club_id: u32) -> RelegationApplyEffects {
        RelegationApplyEffects {
            club_id,
            writes: ClubFieldWrites {
                new_comp_id: 8, prev_comp_id: 7, tier_byte_64: None,
            },
            set_status_idle: true,
            person_effects: vec![
                PersonEffect {
                    person_id: 777,
                    new_staff_1f: Some(2),
                    new_staff_1c: None,
                    event_emit: Some(PersonHistoryEvent {
                        old_comp_id: 7, kind: 3,
                    }),
                },
            ],
            no_league_news: Some(RelegationNoLeagueNews { club_id }),
        }
    }

    fn empty_report_with_events(
        events: Vec<YearEndMutationEvent>,
    ) -> AnnualRolloverReport {
        AnnualRolloverReport {
            events,
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: Default::default(),
        }
    }

    fn today() -> GameDate {
        GameDate { year: 2002, month: 6, day: 30 }
    }

    fn make_world_with_one_club(club_id: u32) -> World {
        let json = r#"{
            "base_data": [],
            "save": null,
            "core": {
                "clubs": [{ "id": 0, "raw": [0] }],
                "nat_clubs": [], "colours": [], "continents": [],
                "nations": []
            },
            "core_summary": {
                "club_count": 1, "nat_club_count": 0,
                "colour_count": 0, "continent_count": 0,
                "nation_count": 0
            },
            "references": {
                "cities": [], "officials": [], "first_names": [],
                "second_names": [], "common_names": [],
                "stadiums": [{
                    "id": 500,
                    "name": "Test",
                    "unknown_tail": [],
                    "name_set": true,
                    "city_id": null,
                    "capacity_total": 10000,
                    "capacity_seated": 5000,
                    "capacity_expansion": 10000,
                    "alt_stadium_id": null
                }],
                "staff_competitions": [], "club_competitions": [],
                "nation_competitions": [], "staff_history": [],
                "staff_comp_history": [], "club_comp_history": [],
                "nation_comp_history": []
            },
            "reference_summary": {
                "city_count": 0, "official_count": 0,
                "first_name_count": 0, "second_name_count": 0,
                "common_name_count": 0, "stadium_count": 1,
                "staff_competition_count": 0,
                "club_competition_count": 0,
                "nation_competition_count": 0,
                "staff_history_count": 0,
                "staff_comp_history_count": 0,
                "club_comp_history_count": 0,
                "nation_comp_history_count": 0
            },
            "staff_summary": {
                "type6_count": 0, "type8_count": 0,
                "type9_count": 0, "type10_count": 0,
                "sample_type6_id": null, "sample_type9_id": null,
                "sample_type10_id": null, "sample_type10_ca": null,
                "sample_type10_pa": null,
                "sample_type10_reputation": null,
                "max_type10_ca": null
            }
        }"#;
        let mut w: World = serde_json::from_str(json).expect("world json");
        // Give the single club a raw record 0x70 long with the
        // requested id at +0x00.
        let mut raw = vec![0u8; 0x70];
        raw[0..4].copy_from_slice(&(club_id as i32).to_le_bytes());
        raw[0x37] = 0xFF;
        w.core.clubs[0].raw = raw;
        w
    }

    #[test]
    fn promotion_event_queues_history_and_welcome_news() {
        let effects = mk_promotion_effects(100);
        let mut world = make_world_with_one_club(100);
        let mut pending: Vec<RuntimeEvent> = vec![]; let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::Promotion { effects },
        ]);
        let out = apply_report_to_world_parts(&mut world, &mut pending, &date, 0, &report);
        // One history + one welcome.
        assert_eq!(out.person_history.len(), 1);
        assert_eq!(out.news_writes.len(), 1);
        assert_eq!(out.news_writes[0].kind, "promotion_welcome");
        assert_eq!(out.pending_squad_resets.len(), 1);
        // Events actually landed in the pending queue.
        assert!(pending.iter()
                .any(|e| e.kind == "promotion_welcome"));
        assert!(pending.iter()
                .any(|e| e.kind.starts_with("person_history_")));
    }

    #[test]
    fn relegation_event_queues_history_and_no_league_news() {
        let effects = mk_relegation_effects(200);
        let mut world = make_world_with_one_club(200);
        let mut pending: Vec<RuntimeEvent> = vec![]; let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::Relegation { effects },
        ]);
        let out = apply_report_to_world_parts(&mut world, &mut pending, &date, 0, &report);
        assert_eq!(out.person_history.len(), 1);
        assert!(out.news_writes.iter()
                .any(|n| n.kind == "relegation_no_league"));
    }

    #[test]
    fn stadium_expansion_success_writes_capacities() {
        let outcome = StadiumExpansionOutcome {
            return_value: 1,
            success: true,
            return_reason: ReturnReason::ForcedSuccess,
            stadium_writes: Some(StadiumWrites {
                new_total: 25_000,
                new_seated: 20_000,
                new_peak: 25_000,
            }),
            club_writes: Some(ClubFinanceWrites {
                new_cash: 50_000_000,
                new_season_misc_expense: 1_000_000,
                new_lifetime_misc_expense: 1_000_000,
                new_season_subsidy_income: 0,
                new_lifetime_subsidy_income: 0,
                owner_subsidised: false,
            }),
            refuse_counter_increment: false,
            news_event: Some(StadiumExpansionNews {
                comp_id: 7, news_ctx: 0, mode: 1,
                template_id: NEWS_TEMPLATE_STADIUM_EXPANSION,
            }),
            cost: 5_000_000,
        };
        let mut world = make_world_with_one_club(300);
        let mut pending: Vec<RuntimeEvent> = vec![]; let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome, club_id: 300, stadium_id: Some(500),
                parent_stadium_id: None,
            },
        ]);
        let out = apply_report_to_world_parts(&mut world, &mut pending, &date, 0, &report);
        assert_eq!(out.stadium_writes.len(), 1);
        assert_eq!(out.stadium_writes[0].new_total, 25_000);
        assert_eq!(out.stadium_writes[0].new_seated, 20_000);
        assert_eq!(out.stadium_writes[0].new_peak, 25_000);
        // Verify it landed on world.references.stadiums.
        let s = world.references.stadiums.iter()
            .find(|s| s.id == 500).unwrap();
        assert_eq!(s.capacity_total, 25_000);
        assert_eq!(s.capacity_seated, 20_000);
        assert_eq!(s.capacity_expansion, 25_000);
        // Finance surfaced as pending.
        assert_eq!(out.pending_finance.len(), 1);
        assert_eq!(out.pending_finance[0].new_cash, 50_000_000);
        // News queued.
        assert!(pending.iter()
                .any(|e| e.kind == "stadium_expansion"));
    }

    #[test]
    fn stadium_expansion_failure_does_not_write_capacity() {
        let outcome = StadiumExpansionOutcome {
            return_value: 0,
            success: false,
            return_reason: ReturnReason::NullStadium,
            stadium_writes: None,
            club_writes: None,
            refuse_counter_increment: false,
            news_event: None,
            cost: 0,
        };
        let mut world = make_world_with_one_club(400);
        let mut pending: Vec<RuntimeEvent> = vec![]; let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome, club_id: 400, stadium_id: Some(500),
                parent_stadium_id: None,
            },
        ]);
        let out = apply_report_to_world_parts(&mut world, &mut pending, &date, 0, &report);
        assert!(out.stadium_writes.is_empty());
        // Stadium capacities unchanged.
        let s = world.references.stadiums.iter()
            .find(|s| s.id == 500).unwrap();
        assert_eq!(s.capacity_total, 10_000);
    }

    #[test]
    fn club_moves_are_materialised_onto_raw_bytes() {
        let mut world = make_world_with_one_club(500);
        let mut pending: Vec<RuntimeEvent> = vec![]; let date = today();
        let mut moves = std::collections::BTreeMap::new();
        moves.insert(500, crate::c15_english_annual_rollover::ClubMoveSummary {
            new_comp_id: 7, prev_comp_id: 8, new_status: 0xFF,
        });
        let report = AnnualRolloverReport {
            events: vec![],
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: moves,
        };
        let out = apply_report_to_world_parts(&mut world, &mut pending, &date, 0, &report);
        assert_eq!(out.club_move_writes, 1);
        // Verify raw bytes.
        let raw = &world.core.clubs[0].raw;
        assert_eq!(i32::from_le_bytes([raw[0x57], raw[0x58], raw[0x59], raw[0x5A]]), 7);
        assert_eq!(i32::from_le_bytes([raw[0x5B], raw[0x5C], raw[0x5D], raw[0x5E]]), 8);
        assert_eq!(raw[0x37], 0xFF);
        // Applier trace reflects the actual byte.
        assert_eq!(out.post_rollover_club_status.get(&500), Some(&0xFF));
    }

    // ======================================================================
    // C15.1A — finance ledger materialisation tests
    // ======================================================================

    use crate::c14_stadium_expansion::{
        apply_stadium_expansion, StadiumExpansionInput,
    };
    use crate::game_rng::GameRng;

    /// Build a `StadiumExpansionInput` with the specified pre-state
    /// finance fields, a fixed cost of £8_000_000 (matching
    /// `bare_input` in c14 tests: seated 12k→20k etc.).
    fn c15_1a_input_for_forced_path(
        club_id: u32, cash: i64,
        expense_ytd: i32, expense_life: i32,
        subsidy_a: i32, subsidy_b: i32,
    ) -> StadiumExpansionInput {
        StadiumExpansionInput {
            club_id,
            stadium_total: 20_000, stadium_seated: 12_000,
            stadium_peak: 20_000,
            club_cash: cash,
            club_season_subsidy_income: subsidy_a,
            club_lifetime_subsidy_income: subsidy_b,
            club_season_misc_expense: expense_ytd,
            club_lifetime_misc_expense: expense_life,
            parent_club_stadium: None,
            desired_seated: 20_000, desired_total: 25_000,
            affordability_checked: false,
            news_ctx: 0, news_comp_id: 7,
        }
    }

    fn to_pending(
        outcome: &crate::c14_stadium_expansion::StadiumExpansionOutcome,
        club_id: u32,
    ) -> PendingFinanceWrite {
        let cw = outcome.club_writes.unwrap();
        PendingFinanceWrite {
            club_id,
            new_cash: cw.new_cash,
            new_season_misc_expense: cw.new_season_misc_expense,
            new_lifetime_misc_expense: cw.new_lifetime_misc_expense,
            new_season_subsidy_income: cw.new_season_subsidy_income,
            new_lifetime_subsidy_income: cw.new_lifetime_subsidy_income,
        }
    }

    #[test]
    fn c15_1a_forced_path_ledger_bytes() {
        // Cost £7,750,000 (bare_input deltas: 8000 seated + 5000
        // standing + 0 stand_to_seat). Starting from cash £50M,
        // expense_ytd £2M, expense_life £10M.
        let mut rng = GameRng::new(0xC151_A000);
        let inp = c15_1a_input_for_forced_path(
            42,
            50_000_000, 2_000_000, 10_000_000, 100_000, 500_000,
        );
        let outcome = apply_stadium_expansion(&inp, &mut rng);
        assert!(outcome.success);
        assert_eq!(outcome.cost, 7_750_000);
        let write = to_pending(&outcome, 42);
        // Byte-exact expected new state.
        assert_eq!(write.new_cash, 50_000_000 - 7_750_000);
        assert_eq!(write.new_season_misc_expense,
                   2_000_000 + 7_750_000);
        assert_eq!(write.new_lifetime_misc_expense,
                   10_000_000 + 7_750_000);
        // Subsidies unchanged (forced path).
        assert_eq!(write.new_season_subsidy_income, 100_000);
        assert_eq!(write.new_lifetime_subsidy_income, 500_000);
        // Ledger applies exactly.
        let mut ledger = ClubFinanceLedger::new();
        ledger.apply_write(&write);
        let s = ledger.get(42);
        assert_eq!(s.cash, 42_250_000);
        assert_eq!(s.season_misc_expense, 9_750_000);
        assert_eq!(s.lifetime_misc_expense, 17_750_000);
        assert_eq!(s.season_subsidy_income, 100_000);
        assert_eq!(s.lifetime_subsidy_income, 500_000);
    }

    #[test]
    fn c15_1a_owner_subsidy_net_cash_zero_but_all_four_accum_bump() {
        // DD FUN_00583FC0 lines 143-153: subsidy credits cash then
        // debits cash by cost (net zero), and bumps BOTH expense
        // and subsidy accumulators (season + lifetime) by cost.
        //
        // Find a seed for which the subsidy RNG gate fires,
        // matching the c14 test pattern.
        use crate::c14_stadium_expansion::ParentStadium;
        let mut inp = c15_1a_input_for_forced_path(
            77, 0, 500_000, 1_500_000, 200_000, 600_000,
        );
        inp.affordability_checked = true;
        inp.parent_club_stadium = Some(ParentStadium { refuse_counter: 20 });
        let mut seed = 0u32;
        let outcome = loop {
            let mut rng = GameRng::new(0xC151_A100u32.wrapping_add(seed));
            let out = apply_stadium_expansion(&inp, &mut rng);
            if let Some(cw) = out.club_writes {
                if cw.owner_subsidised { break out; }
            }
            seed += 1;
            if seed > 5000 { panic!("no subsidy seed"); }
        };
        let write = to_pending(&outcome, 77);
        // Net-zero cash.
        assert_eq!(write.new_cash, 0);
        // Both expense accumulators bumped by cost.
        assert_eq!(write.new_season_misc_expense, 500_000 + 7_750_000);
        assert_eq!(write.new_lifetime_misc_expense, 1_500_000 + 7_750_000);
        // Both subsidy accumulators bumped by cost.
        assert_eq!(write.new_season_subsidy_income, 200_000 + 7_750_000);
        assert_eq!(write.new_lifetime_subsidy_income, 600_000 + 7_750_000);
        // Ledger reflects all four.
        let mut ledger = ClubFinanceLedger::new();
        ledger.apply_write(&write);
        let s = ledger.get(77);
        assert_eq!(s.cash, 0);
        assert_eq!(s.season_misc_expense, 8_250_000);
        assert_eq!(s.lifetime_misc_expense, 9_250_000);
        assert_eq!(s.season_subsidy_income, 7_950_000);
        assert_eq!(s.lifetime_subsidy_income, 8_350_000);
    }

    #[test]
    fn c15_1a_noop_deltas_produce_no_finance_write() {
        // Already at desired capacity → NoOpDeltas → no
        // club_writes. Ledger untouched.
        let mut rng = GameRng::new(0);
        let mut inp = c15_1a_input_for_forced_path(
            99, 25_000_000, 0, 0, 0, 0,
        );
        inp.desired_seated = 12_000; // already at that
        inp.desired_total = 20_000;
        let outcome = apply_stadium_expansion(&inp, &mut rng);
        assert!(outcome.success);
        assert!(outcome.club_writes.is_none());
        // Ledger stays empty.
        let mut ledger = ClubFinanceLedger::new();
        if let Some(cw) = outcome.club_writes {
            let _ = cw; // avoid warning
            ledger.apply_write(&PendingFinanceWrite {
                club_id: 99,
                new_cash: 0,
                new_season_misc_expense: 0,
                new_lifetime_misc_expense: 0,
                new_season_subsidy_income: 0,
                new_lifetime_subsidy_income: 0,
            });
        }
        assert_eq!(ledger.per_club.len(), 0);
    }

    #[test]
    fn c15_1a_refusal_path_produces_no_finance_write() {
        // affordability_checked + broke + big-ticket + rand_mod(5)!=0
        // → reschedule (return 0, no club_writes). Ledger untouched.
        use crate::c14_stadium_expansion::ReturnReason;
        let inp = {
            let mut i = c15_1a_input_for_forced_path(
                123, 0, 0, 0, 0, 0,
            );
            i.affordability_checked = true;
            i.parent_club_stadium = None;
            i
        };
        // Iterate seeds until we hit a refused path.
        let mut seed = 0u32;
        let outcome = loop {
            let mut rng = GameRng::new(0xC151_A200u32.wrapping_add(seed));
            let out = apply_stadium_expansion(&inp, &mut rng);
            if out.return_reason == ReturnReason::RescheduledIndependent {
                break out;
            }
            seed += 1;
            if seed > 5000 { panic!("no refusal seed"); }
        };
        assert!(!outcome.success);
        assert!(outcome.club_writes.is_none());
    }

    #[test]
    fn c15_1a_ledger_overflow_wraps_like_i32() {
        // The exe uses i32 arithmetic for the accumulators; overflow
        // wraps naturally. Verify that a value near i32::MAX bumped
        // by cost wraps rather than saturates.
        let mut rng = GameRng::new(0xC151_A300);
        let inp = c15_1a_input_for_forced_path(
            555,
            50_000_000,
            i32::MAX - 3_000_000,   // near max
            0, 0, 0,
        );
        let outcome = apply_stadium_expansion(&inp, &mut rng);
        assert!(outcome.success);
        let write = to_pending(&outcome, 555);
        // Cost is 7_750_000; season_misc_expense wraps.
        let expected = (i32::MAX - 3_000_000).wrapping_add(7_750_000);
        assert_eq!(write.new_season_misc_expense, expected);
        assert!(expected < 0, "wrapped into negative territory");
    }

    #[test]
    fn c15_1a_negative_cash_is_representable() {
        // The exe permits debt (287 clubs ship bankrupt per
        // memory [[club-record-decoded]]; DD 00587c40 injects
        // chairman rescue when high half <= 0).
        let mut rng = GameRng::new(0xC151_A400);
        let inp = c15_1a_input_for_forced_path(
            999, 1_000_000, 0, 0, 0, 0,   // £1M cash
        );
        let outcome = apply_stadium_expansion(&inp, &mut rng);
        let write = to_pending(&outcome, 999);
        // £1M - £7.75M = -£6.75M
        assert_eq!(write.new_cash, -6_750_000i64);
        let mut ledger = ClubFinanceLedger::new();
        ledger.apply_write(&write);
        assert_eq!(ledger.get(999).cash, -6_750_000);
    }

    #[test]
    fn c15_1a_trace_matches_world_ledger_state_after_apply() {
        // C15.1A trace-vs-state consistency: the PendingFinanceWrite
        // vector on the report reflects EXACTLY what landed in the
        // ledger.
        let mut rng = GameRng::new(0xC151_A500);
        let inp = c15_1a_input_for_forced_path(
            314, 20_000_000, 0, 0, 0, 0,
        );
        let outcome = apply_stadium_expansion(&inp, &mut rng);
        let write = to_pending(&outcome, 314);
        let mut ledger = ClubFinanceLedger::new();
        ledger.apply_write(&write);
        let s = ledger.get(314);
        // Every field of the ledger = corresponding field of the
        // pending write. Byte-exact.
        assert_eq!(s.cash, write.new_cash);
        assert_eq!(s.season_misc_expense, write.new_season_misc_expense);
        assert_eq!(s.lifetime_misc_expense, write.new_lifetime_misc_expense);
        assert_eq!(s.season_subsidy_income, write.new_season_subsidy_income);
        assert_eq!(s.lifetime_subsidy_income, write.new_lifetime_subsidy_income);
    }

    // ======================================================================
    // C15.1B — contract-record staff-state materialisation tests
    // ======================================================================

    use crate::contract_init::{ContractPool, ContractRecord};

    /// Build a `ContractPool` with a single contract for the given
    /// person id with the given clause pre-states.
    /// Build a ContractPool for testing.  param must match
    /// the promotion/relegation report's club_id so the identity
    /// gate  passes.
    fn pool_with_one_contract(
        person_id: u32, relegation: u8, non_promotion: u8,
    ) -> ContractPool {
        pool_with_one_contract_at_club(person_id, 100, relegation, non_promotion)
    }
    fn pool_with_one_contract_at_club(
        person_id: u32, club_id: i32, relegation: u8, non_promotion: u8,
    ) -> ContractPool {
        let mut pool = ContractPool::default();
        pool.records.push(ContractRecord {
            staff_id: person_id as i32,
            club_id,
            wage: 0, value: 0,
            non_promotion, minimum_fee: 0, non_playing: 0,
            relegation, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
            position_code: 0,
        });
        let n = (person_id as usize) + 1;
        pool.by_staff_id = vec![-1; n];
        pool.by_staff_id[person_id as usize] = 0;
        pool
    }

    fn promotion_report(person_id: u32,
                        new_1f: Option<u8>, new_1c: Option<u8>) -> AnnualRolloverReport {
        AnnualRolloverReport {
            events: vec![
                YearEndMutationEvent::Promotion {
                    effects: PromotionApplyEffects {
                        club_id: 100,
                        writes: ClubFieldWrites {
                            new_comp_id: 7, prev_comp_id: 8, tier_byte_64: None,
                        },
                        set_status_idle: true,
                        person_effects: vec![
                            PersonEffect {
                                person_id,
                                new_staff_1f: new_1f,
                                new_staff_1c: new_1c,
                                event_emit: None,
                            },
                        ],
                        welcome_news: None,
                        stadium_expansion: None,
                    },
                },
            ],
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: Default::default(),
        }
    }

    fn relegation_report(person_id: u32,
                         new_1f: Option<u8>) -> AnnualRolloverReport {
        AnnualRolloverReport {
            events: vec![
                YearEndMutationEvent::Relegation {
                    effects: RelegationApplyEffects {
                        club_id: 100,
                        writes: ClubFieldWrites {
                            new_comp_id: 8, prev_comp_id: 7, tier_byte_64: None,
                        },
                        set_status_idle: true,
                        person_effects: vec![
                            PersonEffect {
                                person_id,
                                new_staff_1f: new_1f,
                                new_staff_1c: None,
                                event_emit: Some(PersonHistoryEvent {
                                    old_comp_id: 7, kind: 3,
                                }),
                            },
                        ],
                        no_league_news: None,
                    },
                },
            ],
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: Default::default(),
        }
    }

    #[test]
    fn c15_1b_promotion_disarms_relegation_clause_when_armed() {
        // Person's relegation clause is armed (1). Promotion clears
        // it to 0. Test the +0x1F path in isolation.
        let mut pool = pool_with_one_contract(555, 1, 0);
        let report = promotion_report(555, Some(0), None);
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        // Byte-exact assertion.
        assert_eq!(pool.records[0].relegation, 0);
        assert_eq!(pool.records[0].non_promotion, 0);
        // Trace records the write.
        assert_eq!(out.applied_contract_writes.len(), 1);
        let w = &out.applied_contract_writes[0];
        assert_eq!(w.person_id, 555);
        assert_eq!(w.offset, 0x1F);
        assert_eq!(w.old_value, 1);
        assert_eq!(w.new_value, 0);
        assert_eq!(w.kind, ContractWriteKind::RelegationClauseDisarmed);
    }

    #[test]
    fn c15_1b_promotion_disarms_non_promotion_clause_when_armed() {
        // Person's non-promotion clause is armed (1). Promotion
        // clears it to 0.
        let mut pool = pool_with_one_contract(666, 0, 1);
        let report = promotion_report(666, None, Some(0));
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        assert_eq!(pool.records[0].relegation, 0);
        assert_eq!(pool.records[0].non_promotion, 0);
        assert_eq!(out.applied_contract_writes.len(), 1);
        assert_eq!(out.applied_contract_writes[0].offset, 0x1C);
        assert_eq!(out.applied_contract_writes[0].kind,
                   ContractWriteKind::NonPromotionClauseDisarmed);
    }

    #[test]
    fn c15_1b_promotion_disarms_both_bytes_when_both_armed() {
        // Both clauses armed. Both fire on the same visit.
        // Exe order (004d3550.c L45-47 then L48-50): +0x1F first,
        // then +0x1C. Verify trace order.
        let mut pool = pool_with_one_contract(777, 1, 1);
        let report = promotion_report(777, Some(0), Some(0));
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        assert_eq!(pool.records[0].relegation, 0);
        assert_eq!(pool.records[0].non_promotion, 0);
        assert_eq!(out.applied_contract_writes.len(), 2);
        assert_eq!(out.applied_contract_writes[0].offset, 0x1F);
        assert_eq!(out.applied_contract_writes[1].offset, 0x1C);
    }

    #[test]
    fn c15_1b_relegation_trips_relegation_clause_when_armed() {
        // Relegation walk: +0x1F: 1 → 2. Never touches +0x1C.
        let mut pool = pool_with_one_contract(888, 1, 1);
        let report = relegation_report(888, Some(2));
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        assert_eq!(pool.records[0].relegation, 2);
        // Non-promotion clause untouched by relegation walk.
        assert_eq!(pool.records[0].non_promotion, 1);
        assert_eq!(out.applied_contract_writes.len(), 1);
        let w = &out.applied_contract_writes[0];
        assert_eq!(w.offset, 0x1F);
        assert_eq!(w.old_value, 1);
        assert_eq!(w.new_value, 2);
        assert_eq!(w.kind, ContractWriteKind::RelegationClauseTripped);
    }

    #[test]
    fn c15_1b_no_write_when_relegation_clause_already_zero() {
        // +0x1F == 0: predicate fails; exe skips silently.
        // Rust does the same.
        let mut pool = pool_with_one_contract(1001, 0, 0);
        let report = promotion_report(1001, Some(0), Some(0));
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        assert_eq!(pool.records[0].relegation, 0); // unchanged
        assert_eq!(pool.records[0].non_promotion, 0);
        assert_eq!(out.applied_contract_writes.len(), 0);
    }

    #[test]
    fn c15_1b_no_write_when_relegation_clause_already_two() {
        // +0x1F == 2 (already tripped from a prior season):
        // relegation walk's predicate `== 1` fails; no write.
        // Guards against double-fire.
        let mut pool = pool_with_one_contract(1002, 2, 0);
        let report = relegation_report(1002, Some(2));
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        assert_eq!(pool.records[0].relegation, 2); // unchanged (was 2)
        assert_eq!(out.applied_contract_writes.len(), 0);
    }

    #[test]
    fn c15_1b_no_write_when_non_promotion_clause_zero() {
        // +0x1C == 0: promotion's predicate fails on that byte.
        // +0x1F still fires if armed.
        let mut pool = pool_with_one_contract(1003, 1, 0);
        let report = promotion_report(1003, Some(0), Some(0));
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        assert_eq!(pool.records[0].relegation, 0);
        assert_eq!(pool.records[0].non_promotion, 0);
        assert_eq!(out.applied_contract_writes.len(), 1);
        assert_eq!(out.applied_contract_writes[0].offset, 0x1F);
    }

    #[test]
    fn c15_1b_duplicate_person_second_visit_is_naturally_noop() {
        // Person id 2000 encountered twice (e.g. same person in
        // both first-team and reserve pools). First visit fires
        // 1 → 0. Second visit finds +0x1F == 0; predicate fails;
        // no write. This is the exe's natural dedup — no explicit
        // guard needed.
        let mut pool = pool_with_one_contract(2000, 1, 0);
        // First fire (as if in first-team pass):
        let report = promotion_report(2000, Some(0), None);
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        assert_eq!(out.applied_contract_writes.len(), 1);
        assert_eq!(pool.records[0].relegation, 0);
        // Second fire (as if in reserve pass, same person):
        let mut out2 = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out2);
        assert_eq!(out2.applied_contract_writes.len(), 0);
        assert_eq!(pool.records[0].relegation, 0); // unchanged
    }

    #[test]
    fn c15_1b_missing_contract_is_silent_skip() {
        // Person id 3000 has no contract record (by_staff_id[3000]
        // is out of bounds → returns None from
        // contract_for_staff_mut). Exe skips silently; Rust
        // matches.
        let mut pool = pool_with_one_contract(555, 1, 0);
        // person 3000 has no by_staff_id entry.
        let report = promotion_report(3000, Some(0), None);
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        assert_eq!(out.applied_contract_writes.len(), 0);
        // Existing contract for person 555 is untouched.
        assert_eq!(pool.records[0].relegation, 1);
    }

    #[test]
    fn c15_1b_negative_by_staff_id_sentinel_is_silent_skip() {
        // by_staff_id[person] = -1 means "no contract" per
        // contract_for_staff docstring. contract_for_staff_mut
        // returns None; no write.
        let mut pool = ContractPool::default();
        pool.records.push(ContractRecord {
            staff_id: 1, club_id: 100, wage: 0, value: 0,
            non_promotion: 0, minimum_fee: 0, non_playing: 0,
            relegation: 1, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 0,
            position_code: 0,
        });
        pool.by_staff_id = vec![-1, -1]; // both persons have no contract
        let report = promotion_report(1, Some(0), None);
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        assert_eq!(out.applied_contract_writes.len(), 0);
        assert_eq!(pool.records[0].relegation, 1); // untouched
    }

    #[test]
    fn c15_1b_trace_old_equals_pre_write_new_equals_post_write() {
        // C15.1B point 19: trace-vs-World consistency.
        // Read pool BEFORE apply; capture the byte. Apply. Read
        // pool AFTER. Assert trace.old == pre and trace.new == post.
        let mut pool = pool_with_one_contract(4001, 1, 1);
        let pre_relegation = pool.records[0].relegation;
        let pre_non_promotion = pool.records[0].non_promotion;
        let report = promotion_report(4001, Some(0), Some(0));
        let mut out = WorldApplyReport::default();
        apply_contract_writes_from_report(&mut pool, &report, &mut out);
        let post_relegation = pool.records[0].relegation;
        let post_non_promotion = pool.records[0].non_promotion;
        // Trace order matches exe: +0x1F first, +0x1C second.
        assert_eq!(out.applied_contract_writes[0].offset, 0x1F);
        assert_eq!(out.applied_contract_writes[0].old_value,
                   pre_relegation);
        assert_eq!(out.applied_contract_writes[0].new_value,
                   post_relegation);
        assert_eq!(out.applied_contract_writes[1].offset, 0x1C);
        assert_eq!(out.applied_contract_writes[1].old_value,
                   pre_non_promotion);
        assert_eq!(out.applied_contract_writes[1].new_value,
                   post_non_promotion);
    }

    // ======================================================================
    // C15.1C — SquadRecord +0x3A position-code materialisation
    // ======================================================================
    //
    // Runtime object: same 0x50-byte contract-record pool at
    // `DAT_00accad8`. Promotion resets `+0x3A` to 0 via
    // `FUN_004D3550` Loop A → `FUN_00843970(person, club, 0)`.
    // The helper: range-gates `[-50, +50]`, tries the primary
    // resolver `FUN_004D59D0`, short-circuits on identity match,
    // else tries the secondary resolver `FUN_004D5B00`, else
    // silent skip.
    //
    // Coverage matrix (14 cases):
    //   • Primary-only golden
    //   • Secondary-only golden (primary null)
    //   • Both-records golden (primary wins short-circuit)
    //   • Identity mismatch (primary present but club_id != club)
    //   • Identity mismatch primary → secondary hit
    //   • Range boundaries: -50 pass, +50 pass, -51 skip, +51 skip
    //   • Idempotent second fire (no duplicate trace, no re-write)
    //   • 50-slot walk (mixed valid / missing / mismatch)
    //   • Non-promotion event ignored (relegation should NOT
    //     touch +0x3A per FUN_004D3460)
    //   • Missing pool index (person_id out of by_staff_id range)
    //   • Trace old/new symmetry (old == pre-write, new ==
    //     post-write)

    /// Build a ContractPool with a primary + secondary index.
    /// `primary_person_id` → primary record at slot 0.
    /// `secondary_person_id` → secondary record at slot 1
    /// (via `by_staff_id_secondary`).
    fn pool_with_primary_and_secondary(
        primary_person_id: u32, primary_club: i32,
        secondary_person_id: u32, secondary_club: i32,
        primary_pos: i8, secondary_pos: i8,
    ) -> ContractPool {
        let mut pool = ContractPool::default();
        pool.records.push(ContractRecord {
            staff_id: primary_person_id as i32,
            club_id: primary_club,
            wage: 0, value: 0, non_promotion: 0, minimum_fee: 0,
            non_playing: 0, relegation: 0, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
            position_code: primary_pos,
        });
        pool.records.push(ContractRecord {
            staff_id: secondary_person_id as i32,
            club_id: secondary_club,
            wage: 0, value: 0, non_promotion: 0, minimum_fee: 0,
            non_playing: 0, relegation: 0, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
            position_code: secondary_pos,
        });
        let n = (primary_person_id.max(secondary_person_id) as usize) + 1;
        pool.by_staff_id = vec![-1; n];
        pool.by_staff_id[primary_person_id as usize] = 0;
        pool.by_staff_id_secondary = vec![-1; n];
        pool.by_staff_id_secondary[secondary_person_id as usize] = 1;
        pool
    }

    /// Promotion report with a single person effect and
    /// configurable club_id (used to probe identity gate).
    fn c15c_promotion_report(
        person_id: u32, club_id: u32,
    ) -> AnnualRolloverReport {
        AnnualRolloverReport {
            events: vec![
                YearEndMutationEvent::Promotion {
                    effects: PromotionApplyEffects {
                        club_id,
                        writes: ClubFieldWrites {
                            new_comp_id: 7, prev_comp_id: 8,
                            tier_byte_64: None,
                        },
                        set_status_idle: true,
                        person_effects: vec![
                            PersonEffect {
                                person_id,
                                new_staff_1f: None,
                                new_staff_1c: None,
                                event_emit: None,
                            },
                        ],
                        welcome_news: None,
                        stadium_expansion: None,
                    },
                },
            ],
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: Default::default(),
        }
    }

    #[test]
    fn c15_1c_primary_only_golden() {
        // Primary record exists at club 100 with position_code=7;
        // promotion resets to 0 via primary resolver.
        let mut pool = pool_with_one_contract_at_club(555, 100, 0, 0);
        pool.records[0].position_code = 7;
        let report = c15c_promotion_report(555, 100);
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        assert_eq!(pool.records[0].position_code, 0);
        assert_eq!(out.applied_squad_preference_writes.len(), 1);
        let w = &out.applied_squad_preference_writes[0];
        assert_eq!(w.person_id, 555);
        assert_eq!(w.record_slot, SquadRecordSlot::Primary);
        assert_eq!(w.old_value, 7);
        assert_eq!(w.new_value, 0);
    }

    #[test]
    fn c15_1c_secondary_only_when_primary_null() {
        // person 600 has no primary entry but IS in the secondary
        // index. Secondary resolver hits; write lands there.
        let mut pool = ContractPool::default();
        pool.records.push(ContractRecord {
            staff_id: 600, club_id: 100, wage: 0, value: 0,
            non_promotion: 0, minimum_fee: 0, non_playing: 0,
            relegation: 0, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
            position_code: 3,
        });
        pool.by_staff_id = vec![-1; 601]; // primary all null
        pool.by_staff_id_secondary = vec![-1; 601];
        pool.by_staff_id_secondary[600] = 0;
        let report = c15c_promotion_report(600, 100);
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        assert_eq!(pool.records[0].position_code, 0);
        assert_eq!(out.applied_squad_preference_writes.len(), 1);
        assert_eq!(
            out.applied_squad_preference_writes[0].record_slot,
            SquadRecordSlot::Secondary,
        );
        assert_eq!(
            out.applied_squad_preference_writes[0].old_value, 3
        );
    }

    #[test]
    fn c15_1c_both_records_primary_wins_short_circuit() {
        // Same person present in BOTH primary and secondary
        // indexes. Primary wins; secondary must NOT be written.
        // Matches FUN_00843970 lines 21-27 return-on-primary-hit.
        let mut pool = pool_with_primary_and_secondary(
            700, 100, 700, 100, /*p_pos=*/9, /*s_pos=*/9,
        );
        // Both records reference the same person id 700; both
        // have club_id=100 so both would match identity.
        // We rely on primary short-circuit to leave secondary
        // untouched.
        let report = c15c_promotion_report(700, 100);
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        assert_eq!(pool.records[0].position_code, 0); // primary
        assert_eq!(pool.records[1].position_code, 9); // secondary untouched
        assert_eq!(out.applied_squad_preference_writes.len(), 1);
        assert_eq!(
            out.applied_squad_preference_writes[0].record_slot,
            SquadRecordSlot::Primary,
        );
    }

    #[test]
    fn c15_1c_identity_mismatch_primary_falls_through_to_secondary()
    {
        // Primary record's club_id != promoted club → identity
        // mismatch. Exe control flow: fall through to secondary.
        // If secondary matches, write there.
        let mut pool = pool_with_primary_and_secondary(
            800, 999, 800, 100, /*p_pos=*/5, /*s_pos=*/6,
        );
        // Primary owned by club 999; promoted club is 100.
        // Secondary owned by club 100 → secondary should win.
        let report = c15c_promotion_report(800, 100);
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        assert_eq!(pool.records[0].position_code, 5); // primary untouched
        assert_eq!(pool.records[1].position_code, 0); // secondary written
        assert_eq!(out.applied_squad_preference_writes.len(), 1);
        assert_eq!(
            out.applied_squad_preference_writes[0].record_slot,
            SquadRecordSlot::Secondary,
        );
    }

    #[test]
    fn c15_1c_identity_mismatch_both_is_silent_skip() {
        // Neither primary nor secondary belongs to the promoted
        // club. Exe returns without writing; Rust matches.
        let mut pool = pool_with_primary_and_secondary(
            900, 998, 900, 999, /*p_pos=*/4, /*s_pos=*/4,
        );
        let report = c15c_promotion_report(900, 100);
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        assert_eq!(pool.records[0].position_code, 4);
        assert_eq!(pool.records[1].position_code, 4);
        assert!(out.applied_squad_preference_writes.is_empty());
    }

    // Range gate — 4 boundary tests.
    // The helper is not directly public but we cover the range
    // gate via `write_squad_position`. We drive it through a
    // synthetic promotion event whose position_code delivery
    // isn't 0 by pretending the code path took a non-zero value.
    // Since the current wiring only calls with 0 (in-range), we
    // exercise `write_squad_position` directly. It is a private
    // fn — but the test module can call it.

    #[test]
    fn c15_1c_range_pass_minus_50() {
        let mut pool = pool_with_one_contract_at_club(1100, 100, 0, 0);
        pool.records[0].position_code = 20;
        let mut out = WorldApplyReport::default();
        write_squad_position(&mut pool, 1100, 100, -50, &mut out);
        assert_eq!(pool.records[0].position_code, -50);
        assert_eq!(out.applied_squad_preference_writes.len(), 1);
    }

    #[test]
    fn c15_1c_range_pass_plus_50() {
        let mut pool = pool_with_one_contract_at_club(1101, 100, 0, 0);
        pool.records[0].position_code = 0;
        let mut out = WorldApplyReport::default();
        write_squad_position(&mut pool, 1101, 100, 50, &mut out);
        assert_eq!(pool.records[0].position_code, 50);
        assert_eq!(out.applied_squad_preference_writes.len(), 1);
    }

    #[test]
    fn c15_1c_range_skip_minus_51() {
        let mut pool = pool_with_one_contract_at_club(1102, 100, 0, 0);
        pool.records[0].position_code = 3;
        let mut out = WorldApplyReport::default();
        write_squad_position(&mut pool, 1102, 100, -51, &mut out);
        // Silent skip — record unchanged, no trace.
        assert_eq!(pool.records[0].position_code, 3);
        assert!(out.applied_squad_preference_writes.is_empty());
    }

    #[test]
    fn c15_1c_range_skip_plus_51() {
        let mut pool = pool_with_one_contract_at_club(1103, 100, 0, 0);
        pool.records[0].position_code = 3;
        let mut out = WorldApplyReport::default();
        write_squad_position(&mut pool, 1103, 100, 51, &mut out);
        assert_eq!(pool.records[0].position_code, 3);
        assert!(out.applied_squad_preference_writes.is_empty());
    }

    #[test]
    fn c15_1c_idempotent_second_fire_no_duplicate_trace() {
        // First fire lands the reset. Second fire finds the byte
        // already at 0 → no new trace entry, no re-write.
        let mut pool = pool_with_one_contract_at_club(1200, 100, 0, 0);
        pool.records[0].position_code = 7;
        let report = c15c_promotion_report(1200, 100);
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        assert_eq!(out.applied_squad_preference_writes.len(), 1);
        assert_eq!(pool.records[0].position_code, 0);
        // Second fire, fresh trace vector.
        let mut out2 = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out2,
        );
        assert!(out2.applied_squad_preference_writes.is_empty());
        assert_eq!(pool.records[0].position_code, 0);
    }

    #[test]
    fn c15_1c_missing_pool_index_is_silent_skip() {
        // Person id larger than any by_staff_id entry →
        // contract_for_staff_mut returns None, contract_for_
        // staff_secondary_mut also returns None → silent skip.
        let mut pool = pool_with_one_contract_at_club(1300, 100, 0, 0);
        let report = c15c_promotion_report(9999, 100);
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        assert_eq!(pool.records[0].position_code, 0);
        assert!(out.applied_squad_preference_writes.is_empty());
    }

    #[test]
    fn c15_1c_relegation_does_not_touch_position_code() {
        // FUN_004D3460 (relegation) never calls FUN_00843970 —
        // the +0x3A byte is only written on promotion. Exe:
        // relegation only touches +0x1F (that's C15.1B).
        let mut pool = pool_with_one_contract_at_club(1400, 100, 1, 0);
        pool.records[0].position_code = 6;
        // Relegation event.
        let report = relegation_report(1400, Some(2));
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        assert_eq!(pool.records[0].position_code, 6); // untouched
        assert!(out.applied_squad_preference_writes.is_empty());
    }

    #[test]
    fn c15_1c_fifty_slot_walk_mixed_valid_missing_mismatch() {
        // Simulate the 50-slot Loop-A walk: build a promotion
        // event with 50 person effects — 30 resolve+match,
        // 10 resolve+mismatch, 10 unresolvable — and verify the
        // final trace has exactly 30 entries and only the 30
        // matching records were mutated.
        //
        // Person ids:
        //   1..=30  → primary at club 100 (match)
        //   31..=40 → primary at club 999 (mismatch)
        //   41..=50 → not in by_staff_id (unresolvable)
        let mut pool = ContractPool::default();
        for pid in 1..=30_u32 {
            pool.records.push(ContractRecord {
                staff_id: pid as i32, club_id: 100,
                wage: 0, value: 0, non_promotion: 0,
                minimum_fee: 0, non_playing: 0, relegation: 0,
                manager_job: 0, expiry_dayofyear: 0,
                expiry_year: 2005, position_code: 11,
            });
        }
        for pid in 31..=40_u32 {
            pool.records.push(ContractRecord {
                staff_id: pid as i32, club_id: 999,
                wage: 0, value: 0, non_promotion: 0,
                minimum_fee: 0, non_playing: 0, relegation: 0,
                manager_job: 0, expiry_dayofyear: 0,
                expiry_year: 2005, position_code: 11,
            });
        }
        pool.by_staff_id = vec![-1; 51];
        for pid in 1..=40_u32 {
            pool.by_staff_id[pid as usize] = (pid as i32) - 1;
        }
        // Build a promotion event with all 50 persons.
        let person_effects: Vec<PersonEffect> = (1..=50_u32)
            .map(|pid| PersonEffect {
                person_id: pid, new_staff_1f: None,
                new_staff_1c: None, event_emit: None,
            })
            .collect();
        let report = AnnualRolloverReport {
            events: vec![YearEndMutationEvent::Promotion {
                effects: PromotionApplyEffects {
                    club_id: 100,
                    writes: ClubFieldWrites {
                        new_comp_id: 7, prev_comp_id: 8,
                        tier_byte_64: None,
                    },
                    set_status_idle: true,
                    person_effects,
                    welcome_news: None,
                    stadium_expansion: None,
                },
            }],
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: Default::default(),
        };
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        assert_eq!(
            out.applied_squad_preference_writes.len(), 30,
            "only the 30 matching records should have written",
        );
        for i in 0..30 {
            assert_eq!(pool.records[i].position_code, 0);
        }
        for i in 30..40 {
            assert_eq!(pool.records[i].position_code, 11);
        }
    }

    #[test]
    fn c15_1c_trace_old_equals_pre_write_new_equals_post_write() {
        // Symmetry check: for each write, the trace entry's
        // old_value must match the pool's byte BEFORE the write
        // and new_value must match AFTER.
        let mut pool = pool_with_one_contract_at_club(1500, 100, 0, 0);
        pool.records[0].position_code = -12;
        let pre = pool.records[0].position_code;
        let report = c15c_promotion_report(1500, 100);
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        let post = pool.records[0].position_code;
        assert_eq!(pre, -12);
        assert_eq!(post, 0);
        assert_eq!(out.applied_squad_preference_writes.len(), 1);
        assert_eq!(out.applied_squad_preference_writes[0].old_value,
                   pre);
        assert_eq!(out.applied_squad_preference_writes[0].new_value,
                   post);
    }

    #[test]
    fn c15_1c_range_boundary_writes_are_recorded_faithfully() {
        // Additional coverage: the range gate lets -50 and +50
        // through as ACTUAL writes and records them in the
        // trace with the correct sign.
        let mut pool = pool_with_one_contract_at_club(1600, 100, 0, 0);
        let mut out = WorldApplyReport::default();
        write_squad_position(&mut pool, 1600, 100, -50, &mut out);
        assert_eq!(out.applied_squad_preference_writes.len(), 1);
        assert_eq!(out.applied_squad_preference_writes[0].new_value,
                   -50);
        write_squad_position(&mut pool, 1600, 100, 50, &mut out);
        assert_eq!(out.applied_squad_preference_writes.len(), 2);
        assert_eq!(out.applied_squad_preference_writes[1].old_value,
                   -50);
        assert_eq!(out.applied_squad_preference_writes[1].new_value,
                   50);
    }

    #[test]
    fn c15_1c_trace_vs_world_consistency() {
        // For every entry in applied_squad_preference_writes,
        // the pool's post-state must equal `new_value` on the
        // record identified by that person_id + record_slot.
        let mut pool = pool_with_primary_and_secondary(
            1700, 100, 1701, 100, /*p_pos=*/8, /*s_pos=*/9,
        );
        let report = AnnualRolloverReport {
            events: vec![YearEndMutationEvent::Promotion {
                effects: PromotionApplyEffects {
                    club_id: 100,
                    writes: ClubFieldWrites {
                        new_comp_id: 7, prev_comp_id: 8,
                        tier_byte_64: None,
                    },
                    set_status_idle: true,
                    person_effects: vec![
                        PersonEffect {
                            person_id: 1700,
                            new_staff_1f: None,
                            new_staff_1c: None,
                            event_emit: None,
                        },
                        PersonEffect {
                            person_id: 1701,
                            new_staff_1f: None,
                            new_staff_1c: None,
                            event_emit: None,
                        },
                    ],
                    welcome_news: None,
                    stadium_expansion: None,
                },
            }],
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: Default::default(),
        };
        let mut out = WorldApplyReport::default();
        apply_squad_position_writes_from_report(
            &mut pool, &report, &mut out,
        );
        // person 1700 hits primary (slot 0), person 1701 hits
        // secondary (slot 1).
        assert_eq!(out.applied_squad_preference_writes.len(), 2);
        assert_eq!(pool.records[0].position_code, 0);
        assert_eq!(pool.records[1].position_code, 0);
        for w in &out.applied_squad_preference_writes {
            let idx = match w.record_slot {
                SquadRecordSlot::Primary =>
                    pool.by_staff_id[w.person_id as usize],
                SquadRecordSlot::Secondary =>
                    pool.by_staff_id_secondary[w.person_id as usize],
            };
            assert!(idx >= 0);
            assert_eq!(
                pool.records[idx as usize].position_code,
                w.new_value,
            );
        }
    }

    // ======================================================================
    // C15.1D — Stadium+0x20 owner-refuse counter materialisation
    // ======================================================================
    //
    // The exe writes `parent_stadium.owner_refuse_counter += 1`
    // inside FUN_00583FC0 line 125 when an owner-backed stadium
    // expansion is refused. Saturation gate on line 124:
    // `cVar1 < '\x14'` (i.e. `< 20`).
    //
    // Coverage (10 cases):
    //   * Applier bumps on refuse_counter_increment == true
    //   * Applier records old/new correctly in trace
    //   * Saturation at 20 (no write, no trace)
    //   * refuse_counter_increment == false → no write
    //   * parent_stadium_id == None → silent skip
    //   * Parent stadium not in world.references.stadiums → skip
    //   * Multiple refuse events bump the SAME parent additively
    //   * Bump lands on parent even when success == false
    //   * The stadium.dat receipt shows the byte on the RIGHT id
    //     (isolated from other stadiums)
    //   * Idempotency: replaying the same event bumps again
    //     (mirrors the exe — the write is not state-dependent).

    /// Build a `World` with two stadiums (`primary_sid` and
    /// `parent_sid`) so tests can drive both the capacity write
    /// path AND the refuse-counter path against distinct rows.
    fn world_with_two_stadiums(
        club_id: u32, primary_sid: u32, parent_sid: u32,
        parent_counter: i8,
    ) -> World {
        let mut world = make_world_with_one_club(club_id);
        // Rewire stadium id 500 (from the fixture) to primary_sid,
        // then push the parent stadium as a second row.
        if let Some(s) = world.references.stadiums
            .iter_mut().find(|s| s.id == 500)
        {
            s.id = primary_sid;
        }
        world.references.stadiums.push(DomainStadium {
            id: parent_sid,
            name: format!("Parent-{}", parent_sid),
            unknown_tail: Vec::new(),
            name_set: false,
            city_id: None,
            capacity_total: 5_000,
            capacity_seated: 3_000,
            capacity_expansion: 8_000,
            alt_stadium_id: None,
            owner_refuse_counter: parent_counter,
        });
        world
    }

    /// A `StadiumExpansionOutcome` with `refuse_counter_increment`
    /// set and everything else null/false — this is a pure
    /// refusal outcome, matching the exe's `OwnerRefused`
    /// path (FUN_00583FC0 line 127: return 0).
    fn refused_outcome() -> StadiumExpansionOutcome {
        StadiumExpansionOutcome {
            return_value: 0,
            success: false,
            return_reason: ReturnReason::OwnerRefused,
            stadium_writes: None,
            club_writes: None,
            refuse_counter_increment: true,
            news_event: None,
            cost: 6_000_000,
        }
    }

    #[test]
    fn c15_1d_refuse_bump_lands_on_parent_stadium() {
        let mut world = world_with_two_stadiums(
            700, /*primary=*/500, /*parent=*/501,
            /*counter=*/3,
        );
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700,
                stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        // Byte-exact assertion.
        let parent = world.references.stadiums.iter()
            .find(|s| s.id == 501).unwrap();
        assert_eq!(parent.owner_refuse_counter, 4);
        // Primary stadium's counter untouched.
        let primary = world.references.stadiums.iter()
            .find(|s| s.id == 500).unwrap();
        assert_eq!(primary.owner_refuse_counter, 0);
        // Trace records old/new.
        assert_eq!(out.applied_refuse_counter_writes.len(), 1);
        let w = &out.applied_refuse_counter_writes[0];
        assert_eq!(w.stadium_id, 501);
        assert_eq!(w.old_value, 3);
        assert_eq!(w.new_value, 4);
    }

    #[test]
    fn c15_1d_saturates_at_20() {
        // Parent already at 20 → exe skips the increment (line
        // 124 `if (cVar1 < '\x14')`). Rust matches.
        let mut world = world_with_two_stadiums(
            700, 500, 501, /*counter=*/20,
        );
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let parent = world.references.stadiums.iter()
            .find(|s| s.id == 501).unwrap();
        assert_eq!(parent.owner_refuse_counter, 20);
        assert!(out.applied_refuse_counter_writes.is_empty());
    }

    #[test]
    fn c15_1d_no_write_when_flag_false() {
        // `refuse_counter_increment == false` — nothing to do.
        let mut outcome = refused_outcome();
        outcome.refuse_counter_increment = false;
        let mut world = world_with_two_stadiums(
            700, 500, 501, /*counter=*/3,
        );
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome, club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let parent = world.references.stadiums.iter()
            .find(|s| s.id == 501).unwrap();
        assert_eq!(parent.owner_refuse_counter, 3);
        assert!(out.applied_refuse_counter_writes.is_empty());
    }

    #[test]
    fn c15_1d_no_write_when_parent_stadium_id_none() {
        // The event carries no parent_stadium_id (the club has no
        // owner-parent, or the pre-C15.1D fixture didn't seed it).
        let mut world = world_with_two_stadiums(
            700, 500, 501, /*counter=*/3,
        );
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: None,
            },
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let parent = world.references.stadiums.iter()
            .find(|s| s.id == 501).unwrap();
        assert_eq!(parent.owner_refuse_counter, 3);
        assert!(out.applied_refuse_counter_writes.is_empty());
    }

    #[test]
    fn c15_1d_no_write_when_parent_stadium_missing_in_world() {
        // parent_stadium_id points at an id that does not exist.
        let mut world = world_with_two_stadiums(
            700, 500, 501, /*counter=*/3,
        );
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(9999),
            },
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        assert!(out.applied_refuse_counter_writes.is_empty());
    }

    #[test]
    fn c15_1d_multiple_refuse_events_bump_additively() {
        // Two refuse events against the same parent → counter
        // bumps twice.
        let mut world = world_with_two_stadiums(
            700, 500, 501, /*counter=*/2,
        );
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let parent = world.references.stadiums.iter()
            .find(|s| s.id == 501).unwrap();
        assert_eq!(parent.owner_refuse_counter, 4);
        assert_eq!(out.applied_refuse_counter_writes.len(), 2);
        assert_eq!(out.applied_refuse_counter_writes[0].old_value, 2);
        assert_eq!(out.applied_refuse_counter_writes[0].new_value, 3);
        assert_eq!(out.applied_refuse_counter_writes[1].old_value, 3);
        assert_eq!(out.applied_refuse_counter_writes[1].new_value, 4);
    }

    #[test]
    fn c15_1d_bump_fires_when_success_false() {
        // The exe's write is on the FAILURE branch of
        // FUN_00583FC0 (return 0). success == false is the norm
        // for this event. Also verify no capacity write leaks.
        let mut world = world_with_two_stadiums(
            700, 500, 501, /*counter=*/5,
        );
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let outcome = refused_outcome(); // success = false
        assert!(!outcome.success);
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome, club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let parent = world.references.stadiums.iter()
            .find(|s| s.id == 501).unwrap();
        assert_eq!(parent.owner_refuse_counter, 6);
        // No capacity writes (the outcome carries none).
        assert!(out.stadium_writes.is_empty());
    }

    #[test]
    fn c15_1d_saturation_streak_stops_writes() {
        // Five refuse events against a parent that starts at 18:
        // 18 → 19 → 20 → skip → skip → skip. Trace length is 2.
        let mut world = world_with_two_stadiums(
            700, 500, 501, /*counter=*/18,
        );
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let evs: Vec<_> = (0..5).map(|_| {
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            }
        }).collect();
        let report = empty_report_with_events(evs);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let parent = world.references.stadiums.iter()
            .find(|s| s.id == 501).unwrap();
        assert_eq!(parent.owner_refuse_counter, 20);
        assert_eq!(out.applied_refuse_counter_writes.len(), 2);
        assert_eq!(out.applied_refuse_counter_writes[0].new_value, 19);
        assert_eq!(out.applied_refuse_counter_writes[1].new_value, 20);
    }

    #[test]
    fn c15_1d_trace_isolates_by_stadium_id() {
        // Three refuse events, two targeting parent 501 and one
        // parent 502. Verify each counter ends up on the right row.
        let mut world = world_with_two_stadiums(
            700, 500, 501, /*counter=*/0,
        );
        world.references.stadiums.push(DomainStadium {
            id: 502,
            name: "Parent-502".to_string(),
            unknown_tail: Vec::new(),
            name_set: false,
            city_id: None,
            capacity_total: 5_000,
            capacity_seated: 3_000,
            capacity_expansion: 8_000,
            alt_stadium_id: None,
            owner_refuse_counter: 10,
        });
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(502),
            },
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let p501 = world.references.stadiums.iter()
            .find(|s| s.id == 501).unwrap();
        let p502 = world.references.stadiums.iter()
            .find(|s| s.id == 502).unwrap();
        assert_eq!(p501.owner_refuse_counter, 2);
        assert_eq!(p502.owner_refuse_counter, 11);
        assert_eq!(out.applied_refuse_counter_writes.len(), 3);
        let by_id: std::collections::BTreeMap<u32, Vec<i8>> =
            out.applied_refuse_counter_writes.iter().fold(
                Default::default(),
                |mut acc, w| {
                    acc.entry(w.stadium_id).or_default()
                        .push(w.new_value);
                    acc
                },
            );
        assert_eq!(by_id.get(&501).unwrap(), &vec![1i8, 2]);
        assert_eq!(by_id.get(&502).unwrap(), &vec![11i8]);
    }

    #[test]
    fn c15_1d_trace_new_equals_pool_state() {
        // For every trace entry, world.references.stadiums'
        // owner_refuse_counter for that stadium_id must equal
        // the LAST new_value emitted for it.
        let mut world = world_with_two_stadiums(
            700, 500, 501, /*counter=*/0,
        );
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
            YearEndMutationEvent::StadiumExpansion {
                outcome: refused_outcome(),
                club_id: 700, stadium_id: Some(500),
                parent_stadium_id: Some(501),
            },
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let last_by_sid: std::collections::BTreeMap<u32, i8> =
            out.applied_refuse_counter_writes.iter()
                .fold(Default::default(), |mut m, w| {
                    m.insert(w.stadium_id, w.new_value);
                    m
                });
        for (sid, expected) in &last_by_sid {
            let s = world.references.stadiums.iter()
                .find(|s| s.id == *sid).unwrap();
            assert_eq!(s.owner_refuse_counter, *expected);
        }
    }

    // ======================================================================
    // C15.1E — persistent person-history (news-mailbox) materialisation
    // ======================================================================
    //
    // Storage owner (decompile-proven): the exe's per-person
    // news-mailbox pool at `DAT_00ACD5C4 + person_id * 0x6E →
    // +0xCF → shared news slab (stride 0xDF, age-bucket rings
    // of 100 entries)`. The Rust port collapses the age-bucket
    // routing into a `BTreeMap<person_id, Vec<PersonNewsItem>>`
    // — see `crates/cm-domain/src/person_news.rs` and
    // `reports/c15_1e_history_archaeology.md` for the full
    // derivation (including the refutation of the earlier
    // "13-list-per-person" hypothesis: `kind` is a stored data
    // field on the item, not a list index).

    use crate::person_news::{NEWS_CATEGORY_PERSON_CAREER, kinds};

    /// Build a world whose ContractPool resolves `person_id`
    /// to a contract at `club_id` with the given
    /// `relegation` byte. Mirrors the C15.1B helper so
    /// C15.1E tests can start from a state whose C15.1B
    /// materialisation is meaningful.
    fn world_with_contract(
        club_id: u32, person_id: u32, staff_id: i32,
        relegation: u8,
    ) -> World {
        let mut world = make_world_with_one_club(club_id);
        let mut pool = ContractPool::default();
        pool.records.push(ContractRecord {
            staff_id,
            club_id: club_id as i32,
            wage: 0, value: 0,
            non_promotion: 0, minimum_fee: 0, non_playing: 0,
            relegation, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
            position_code: 0,
        });
        let n = (person_id as usize) + 1;
        pool.by_staff_id = vec![-1; n];
        pool.by_staff_id[person_id as usize] = 0;
        world.contracts = Some(pool);
        world
    }

    fn c15e_relegation_report(
        club_id: u32, people: &[(u32, /*kind=*/u8, /*old_comp=*/u32)],
    ) -> AnnualRolloverReport {
        AnnualRolloverReport {
            events: vec![
                YearEndMutationEvent::Relegation {
                    effects: RelegationApplyEffects {
                        club_id,
                        writes: ClubFieldWrites {
                            new_comp_id: 8, prev_comp_id: 7,
                            tier_byte_64: None,
                        },
                        set_status_idle: true,
                        person_effects: people.iter().map(|&(pid, kind, oc)| {
                            PersonEffect {
                                person_id: pid,
                                new_staff_1f: Some(2),
                                new_staff_1c: None,
                                event_emit: Some(PersonHistoryEvent {
                                    old_comp_id: oc, kind,
                                }),
                            }
                        }).collect(),
                        no_league_news: None,
                    },
                },
            ],
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: Default::default(),
        }
    }

    #[test]
    fn c15_1e_relegation_golden_end_to_end() {
        // Golden: armed clause on a person attached to the
        // relegated club, old_comp=7. Apply. Assert:
        //   * C15.1B contract 1 → 2 landed
        //   * C15.1E mailbox entry landed
        //   * category = 0xFBF, kind = 3, old_comp = 7,
        //     staff_id resolved via ContractPool
        //   * trace old_len=0, new_len=1
        let mut world = world_with_contract(100, 555, 12345, 1);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = c15e_relegation_report(
            100, &[(555, kinds::RELEGATED, 7)],
        );
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        // C15.1B invariant: contract byte transitioned.
        let pool = world.contracts.as_ref().unwrap();
        assert_eq!(pool.records[0].relegation, 2);
        // C15.1E: mailbox populated.
        let mb = world.person_news_mailboxes.mailbox_for(555);
        assert_eq!(mb.len(), 1);
        let item = &mb[0];
        assert_eq!(item.category, NEWS_CATEGORY_PERSON_CAREER);
        assert_eq!(item.person_id(), 555);
        assert_eq!(item.old_comp_id(), 7);
        assert_eq!(item.kind(), kinds::RELEGATED);
        assert_eq!(item.staff_id(), 12345);
        assert_eq!(item.year, date.year);
        assert_eq!(item.news_id, 0);
        // Trace: one entry, old_len 0, new_len 1.
        assert_eq!(out.applied_person_history_writes.len(), 1);
        let w = &out.applied_person_history_writes[0];
        assert_eq!(w.person_id, 555);
        assert_eq!(w.category, NEWS_CATEGORY_PERSON_CAREER);
        assert_eq!(w.kind, kinds::RELEGATED);
        assert_eq!(w.old_comp_id, 7);
        assert_eq!(w.staff_id, 12345);
        assert_eq!(w.old_len, 0);
        assert_eq!(w.new_len, 1);
        assert_eq!(w.news_id, 0);
    }

    #[test]
    fn c15_1e_multiple_persons_one_event_each_ordered() {
        // Three armed persons on the relegated club, distinct
        // ids and staff ids. Assert one mailbox entry per
        // qualifying person in the report's event iteration
        // order. news_id is monotonic.
        let mut world = make_world_with_one_club(100);
        // Build a pool with three contracts at club 100.
        let mut pool = ContractPool::default();
        for (pid, sid) in [(11u32, 1001i32), (22, 1002), (33, 1003)] {
            pool.records.push(ContractRecord {
                staff_id: sid, club_id: 100,
                wage: 0, value: 0, non_promotion: 0,
                minimum_fee: 0, non_playing: 0,
                relegation: 1, manager_job: 0,
                expiry_dayofyear: 0, expiry_year: 2005,
                position_code: 0,
            });
            let n = (pid as usize) + 1;
            if pool.by_staff_id.len() < n {
                pool.by_staff_id.resize(n, -1);
            }
            pool.by_staff_id[pid as usize] =
                (pool.records.len() - 1) as i32;
        }
        world.contracts = Some(pool);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = c15e_relegation_report(100, &[
            (11, kinds::RELEGATED, 7),
            (22, kinds::RELEGATED, 7),
            (33, kinds::RELEGATED, 7),
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        assert_eq!(out.applied_person_history_writes.len(), 3);
        // Ordered by iteration.
        for (i, expected_pid) in [11u32, 22, 33].iter().enumerate() {
            let w = &out.applied_person_history_writes[i];
            assert_eq!(w.person_id, *expected_pid);
            assert_eq!(w.news_id, i as u32);
        }
        // Each mailbox has exactly one entry.
        for pid in [11u32, 22, 33] {
            assert_eq!(
                world.person_news_mailboxes.mailbox_for(pid).len(),
                1,
            );
        }
    }

    #[test]
    fn c15_1e_duplicate_person_only_first_transition_writes_history() {
        // Same person emitted twice in one event. C15.1B
        // transitions +0x1F 1 → 2 on the first visit; the
        // second visit's contract-write predicate fails
        // silently. But — CRITICAL — the PersonHistoryEvent
        // still fires from the C13 payload, so from this
        // tranche's perspective the mailbox appends TWICE.
        //
        // This is the frozen boundary: C15.1E consumes the C13
        // event stream, not the C15.1B mutation outcome. If
        // real capture shows the exe emits only once, that's a
        // C13 filter concern, not a C15.1E one. Pin it here as
        // observed behaviour so future changes are conscious.
        let mut world = world_with_contract(100, 555, 12345, 1);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = c15e_relegation_report(100, &[
            (555, kinds::RELEGATED, 7),
            (555, kinds::RELEGATED, 7),
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let mb = world.person_news_mailboxes.mailbox_for(555);
        assert_eq!(mb.len(), 2, "documented C15.1E semantic — see test comment");
        assert_eq!(out.applied_person_history_writes.len(), 2);
        assert_eq!(mb[0].news_id, 0);
        assert_eq!(mb[1].news_id, 1);
    }

    #[test]
    fn c15_1e_non_qualifying_contract_no_history() {
        // Person id has no contract → applier silent-skip.
        let mut world = world_with_contract(100, 555, 12345, 1);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        // The event carries a different person id (999) with no
        // ContractPool entry.
        let report = c15e_relegation_report(
            100, &[(999, kinds::RELEGATED, 7)],
        );
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        assert!(out.applied_person_history_writes.is_empty());
        assert!(world.person_news_mailboxes
            .mailbox_for(999).is_empty());
    }

    #[test]
    fn c15_1e_identity_mismatch_no_history() {
        // Person's contract belongs to a different club → the
        // C15.1E identity gate mirrors C15.1B's, so no
        // mailbox append fires.
        let mut world = world_with_contract(999, 555, 12345, 1);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = c15e_relegation_report(
            100, &[(555, kinds::RELEGATED, 7)],
        );
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        assert!(out.applied_person_history_writes.is_empty());
        assert!(world.person_news_mailboxes
            .mailbox_for(555).is_empty());
    }

    #[test]
    fn c15_1e_no_contract_pool_is_silent_skip() {
        // World.contracts == None (contract subsystem not
        // initialised) → applier silent-skip. No panic.
        let mut world = make_world_with_one_club(100);
        assert!(world.contracts.is_none());
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = c15e_relegation_report(
            100, &[(555, kinds::RELEGATED, 7)],
        );
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        assert!(out.applied_person_history_writes.is_empty());
    }

    #[test]
    fn c15_1e_multi_season_appends() {
        // Two apply passes back to back, second at a later
        // year. Both should append. Mailbox length after two
        // seasons is 2. news_ids continue monotonically.
        let mut world = world_with_contract(100, 555, 12345, 1);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date_y1 = GameDate { year: 2002, month: 6, day: 30 };
        let date_y2 = GameDate { year: 2003, month: 6, day: 30 };
        let report = c15e_relegation_report(
            100, &[(555, kinds::RELEGATED, 7)],
        );
        let _ = apply_report_to_world_parts(
            &mut world, &mut pending, &date_y1, 0, &report,
        );
        // Re-arm the contract so C15.1B fires again (only for
        // this multi-season exercise; the exe would re-arm as
        // part of a fresh season's C13 pass).
        world.contracts.as_mut().unwrap()
            .records[0].relegation = 1;
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date_y2, 400, &report,
        );
        let mb = world.person_news_mailboxes.mailbox_for(555);
        assert_eq!(mb.len(), 2);
        assert_eq!(mb[0].year, 2002);
        assert_eq!(mb[1].year, 2003);
        assert_eq!(mb[0].news_id, 0);
        assert_eq!(mb[1].news_id, 1);
        assert_eq!(out.applied_person_history_writes.len(), 1);
        assert_eq!(out.applied_person_history_writes[0].news_id, 1);
    }

    #[test]
    fn c15_1e_kind_isolation_between_persons() {
        // Two persons in one event, different old_comp_ids.
        // Each mailbox holds ONLY its own item; no cross-write.
        let mut world = make_world_with_one_club(100);
        let mut pool = ContractPool::default();
        pool.records.push(ContractRecord {
            staff_id: 100, club_id: 100, wage: 0, value: 0,
            non_promotion: 0, minimum_fee: 0, non_playing: 0,
            relegation: 1, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
            position_code: 0,
        });
        pool.records.push(ContractRecord {
            staff_id: 200, club_id: 100, wage: 0, value: 0,
            non_promotion: 0, minimum_fee: 0, non_playing: 0,
            relegation: 1, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
            position_code: 0,
        });
        pool.by_staff_id = vec![-1; 6];
        pool.by_staff_id[3] = 0;
        pool.by_staff_id[5] = 1;
        world.contracts = Some(pool);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = c15e_relegation_report(100, &[
            (3, kinds::RELEGATED, 7),
            (5, kinds::RELEGATED, 8),
        ]);
        let _ = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        let mb3 = world.person_news_mailboxes.mailbox_for(3);
        let mb5 = world.person_news_mailboxes.mailbox_for(5);
        assert_eq!(mb3.len(), 1);
        assert_eq!(mb5.len(), 1);
        assert_eq!(mb3[0].old_comp_id(), 7);
        assert_eq!(mb5[0].old_comp_id(), 8);
        assert_eq!(mb3[0].staff_id(), 100);
        assert_eq!(mb5[0].staff_id(), 200);
    }

    #[test]
    fn c15_1e_boundary_no_max_size_is_an_intentional_deviation() {
        // The exe rings each age-bucket at 100 entries and
        // silently overwrites. Rust uses append-only Vec. This
        // test PINS the deviation: 150 appends produce 150
        // entries, never wrapping.
        //
        // The deviation is safe because at year-end scope the
        // exe emits at most one entry per person per season.
        // If a downstream tranche introduces per-day mailbox
        // dispatches, revisit.
        let mut world = world_with_contract(100, 42, 4200, 1);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        for _ in 0..150 {
            world.contracts.as_mut().unwrap()
                .records[0].relegation = 1;
            let report = c15e_relegation_report(
                100, &[(42, kinds::RELEGATED, 7)],
            );
            let _ = apply_report_to_world_parts(
                &mut world, &mut pending, &date, 0, &report,
            );
        }
        assert_eq!(
            world.person_news_mailboxes.mailbox_for(42).len(),
            150,
        );
    }

    #[test]
    fn c15_1e_trace_matches_world_state() {
        // For every trace entry, mailbox_for(person_id)
        // .last() equals the trace's new_id, kind, old_comp
        // and staff_id.
        let mut world = make_world_with_one_club(100);
        let mut pool = ContractPool::default();
        for (pid, sid) in [(4u32, 40i32), (5, 50), (6, 60), (7, 70)] {
            pool.records.push(ContractRecord {
                staff_id: sid, club_id: 100,
                wage: 0, value: 0, non_promotion: 0,
                minimum_fee: 0, non_playing: 0,
                relegation: 1, manager_job: 0,
                expiry_dayofyear: 0, expiry_year: 2005,
                position_code: 0,
            });
            let n = (pid as usize) + 1;
            if pool.by_staff_id.len() < n {
                pool.by_staff_id.resize(n, -1);
            }
            pool.by_staff_id[pid as usize] =
                (pool.records.len() - 1) as i32;
        }
        world.contracts = Some(pool);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        let report = c15e_relegation_report(100, &[
            (4, kinds::RELEGATED, 7),
            (5, kinds::RELEGATED, 7),
            (6, kinds::RELEGATED, 7),
            (7, kinds::RELEGATED, 7),
        ]);
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        assert_eq!(out.applied_person_history_writes.len(), 4);
        for w in &out.applied_person_history_writes {
            let mb = world.person_news_mailboxes
                .mailbox_for(w.person_id);
            let last = mb.last().unwrap();
            assert_eq!(last.news_id, w.news_id);
            assert_eq!(last.kind(), w.kind);
            assert_eq!(last.old_comp_id(), w.old_comp_id);
            assert_eq!(last.staff_id(), w.staff_id);
            assert_eq!(last.person_id(), w.person_id);
        }
    }

    // ======================================================================
    // C15.1F — canonical runtime-finance mapping tests
    // ======================================================================
    //
    // Freezes `ClubFinanceLedger` as the canonical Rust carrier
    // for the exe's per-club 0x167-byte Runtime Finance record
    // (pool base `*DAT_00acdc38`, allocated by `FUN_00584530`,
    // seeded by `FUN_005803D0` from disk `Club+0x65`, serialised
    // as `finance.dat`). See
    // `reports/c15_1f_finance_loader_archaeology.md`.
    //
    // Coverage:
    //   * from_disk_seed — cash i32 widens to i64, accumulators zero.
    //   * seed_from_world — every club gets a ledger entry keyed
    //     by club_id with cash = disk seed.
    //   * apply_write — post-seed C14 output overwrites cash and
    //     accumulators.
    //   * serde round-trip — JSON-serialise then deserialise a
    //     seeded+applied ledger and assert equality.
    //   * trace-vs-World — every emitted PendingFinanceWrite
    //     lands on the same club_id in the ledger.
    //   * ClubView::initial_cash_seed — reads Club+0x65 exactly.

    #[test]
    fn c15_1f_from_disk_seed_widens_i32_to_i64() {
        // Positive seed.
        let s = ClubFinanceState::from_disk_seed(30_000_000);
        assert_eq!(s.cash, 30_000_000i64);
        assert_eq!(s.season_misc_expense, 0);
        assert_eq!(s.lifetime_misc_expense, 0);
        assert_eq!(s.season_subsidy_income, 0);
        assert_eq!(s.lifetime_subsidy_income, 0);
        // Negative seed (bankrupt-at-start).
        let s2 = ClubFinanceState::from_disk_seed(-14_000_000);
        assert_eq!(s2.cash, -14_000_000i64);
        // Extremes.
        assert_eq!(ClubFinanceState::from_disk_seed(i32::MAX).cash,
                   i32::MAX as i64);
        assert_eq!(ClubFinanceState::from_disk_seed(i32::MIN).cash,
                   i32::MIN as i64);
    }

    #[test]
    fn c15_1f_seed_from_world_populates_every_club() {
        // World has one club (id 0) with a synthetic Club raw
        // byte pattern that puts a known i32 at +0x65.
        let mut world = make_world_with_one_club(0);
        // Inject a known cash seed at Club+0x65 on club id 0.
        // make_world_with_one_club builds an all-zero row; we
        // overwrite bytes 0x65..0x69 with a known i32.
        let seed_val: i32 = 50_000_000;
        {
            let club = &mut world.core.clubs[0];
            let raw = &mut club.raw;
            if raw.len() < 0x69 { raw.resize(0x69, 0); }
            let b = seed_val.to_le_bytes();
            raw[0x65..0x69].copy_from_slice(&b);
        }
        let mut ledger = ClubFinanceLedger::new();
        ledger.seed_from_world(&world);
        assert_eq!(ledger.per_club.len(), 1);
        let state = ledger.get(0);
        assert_eq!(state.cash, 50_000_000i64);
        assert_eq!(state.season_misc_expense, 0);
        assert_eq!(state.lifetime_subsidy_income, 0);
    }

    #[test]
    fn c15_1f_seed_from_world_is_idempotent() {
        // Calling seed_from_world twice yields the same ledger.
        let mut world = make_world_with_one_club(0);
        {
            let raw = &mut world.core.clubs[0].raw;
            if raw.len() < 0x69 { raw.resize(0x69, 0); }
            raw[0x65..0x69].copy_from_slice(&12_345i32.to_le_bytes());
        }
        let mut a = ClubFinanceLedger::new();
        a.seed_from_world(&world);
        let mut b = a.clone();
        b.seed_from_world(&world);
        assert_eq!(a, b);
    }

    #[test]
    fn c15_1f_apply_write_overwrites_seed() {
        // Seed a club, then apply a PendingFinanceWrite; the
        // seed's initial state is replaced with the new values.
        let mut world = make_world_with_one_club(0);
        {
            let raw = &mut world.core.clubs[0].raw;
            if raw.len() < 0x69 { raw.resize(0x69, 0); }
            raw[0x65..0x69].copy_from_slice(&1_000_000i32.to_le_bytes());
        }
        let mut ledger = ClubFinanceLedger::new();
        ledger.seed_from_world(&world);
        assert_eq!(ledger.get(0).cash, 1_000_000);
        ledger.apply_write(&PendingFinanceWrite {
            club_id: 0,
            new_cash: -5_000_000,
            new_season_misc_expense: 6_000_000,
            new_lifetime_misc_expense: 6_000_000,
            new_season_subsidy_income: 0,
            new_lifetime_subsidy_income: 0,
        });
        let state = ledger.get(0);
        assert_eq!(state.cash, -5_000_000);
        assert_eq!(state.season_misc_expense, 6_000_000);
        assert_eq!(state.lifetime_misc_expense, 6_000_000);
    }

    #[test]
    fn c15_1f_serde_round_trip_preserves_state() {
        // A ledger with a mix of seeded and applied clubs
        // should round-trip via JSON without loss.
        let mut ledger = ClubFinanceLedger::new();
        ledger.per_club.insert(0,
            ClubFinanceState::from_disk_seed(30_000_000));
        ledger.apply_write(&PendingFinanceWrite {
            club_id: 5,
            new_cash: 123_456_789,
            new_season_misc_expense: 42,
            new_lifetime_misc_expense: 100,
            new_season_subsidy_income: 7,
            new_lifetime_subsidy_income: 21,
        });
        let json = serde_json::to_string(&ledger).unwrap();
        let restored: ClubFinanceLedger =
            serde_json::from_str(&json).unwrap();
        assert_eq!(ledger, restored);
        // Belt-and-braces: the values on club 5 survive.
        let s5 = restored.get(5);
        assert_eq!(s5.cash, 123_456_789);
        assert_eq!(s5.lifetime_subsidy_income, 21);
    }

    #[test]
    fn c15_1f_serde_default_lets_empty_json_deserialise() {
        // #[serde(default)] on per_club means an old save that
        // didn't carry the map still loads as an empty ledger.
        let empty: ClubFinanceLedger =
            serde_json::from_str("{}").unwrap();
        assert!(empty.per_club.is_empty());
    }

    #[test]
    fn c15_1f_trace_vs_ledger_consistency() {
        // For every entry in ledger.per_club we can identify a
        // PendingFinanceWrite (in the trace) with the same
        // final values, or a `from_disk_seed` origin. This
        // pins the invariant that the ledger IS the canonical
        // finance state.
        let mut ledger = ClubFinanceLedger::new();
        ledger.per_club.insert(1,
            ClubFinanceState::from_disk_seed(200_000));
        let writes = vec![
            PendingFinanceWrite {
                club_id: 1, new_cash: 5,
                new_season_misc_expense: 100,
                new_lifetime_misc_expense: 100,
                new_season_subsidy_income: 0,
                new_lifetime_subsidy_income: 0,
            },
            PendingFinanceWrite {
                club_id: 2, new_cash: 42,
                new_season_misc_expense: 0,
                new_lifetime_misc_expense: 0,
                new_season_subsidy_income: 0,
                new_lifetime_subsidy_income: 0,
            },
        ];
        for w in &writes { ledger.apply_write(w); }
        // Final values match the LAST write per club_id.
        assert_eq!(ledger.get(1).cash, 5);
        assert_eq!(ledger.get(2).cash, 42);
    }

    #[test]
    fn c15_1f_club_view_initial_cash_seed_reads_offset_65() {
        // ClubView reads +0x65 exactly as an i32 LE.
        use crate::typed_records::ClubView;
        let mut world = make_world_with_one_club(7);
        {
            let raw = &mut world.core.clubs[0].raw;
            if raw.len() < 0x69 { raw.resize(0x69, 0); }
            raw[0x65..0x69].copy_from_slice(
                &(-8_500_000i32).to_le_bytes(),
            );
        }
        let cv = ClubView::new(&world.core.clubs[0]);
        assert_eq!(cv.initial_cash_seed(), -8_500_000);
    }

    #[test]
    fn c15_1e_promotion_does_not_touch_mailbox() {
        // Only Relegation events consume this path in the
        // year-end pipeline. Promotion path in the exe uses a
        // different helper chain (FUN_004D3550 → no
        // FUN_008D0D90 call). Rust matches.
        let mut world = world_with_contract(100, 555, 12345, 1);
        let mut pending: Vec<RuntimeEvent> = vec![];
        let date = today();
        // Build a Promotion event with a PersonHistoryEvent
        // payload (unusual but explicit — the walk is
        // event-variant-gated, not payload-gated).
        let report = AnnualRolloverReport {
            events: vec![YearEndMutationEvent::Promotion {
                effects: PromotionApplyEffects {
                    club_id: 100,
                    writes: ClubFieldWrites {
                        new_comp_id: 7, prev_comp_id: 8,
                        tier_byte_64: None,
                    },
                    set_status_idle: true,
                    person_effects: vec![PersonEffect {
                        person_id: 555,
                        new_staff_1f: Some(0),
                        new_staff_1c: None,
                        event_emit: Some(PersonHistoryEvent {
                            old_comp_id: 8, kind: kinds::RELEGATED,
                        }),
                    }],
                    welcome_news: None,
                    stadium_expansion: None,
                },
            }],
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: Default::default(),
        };
        let out = apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        assert!(out.applied_person_history_writes.is_empty());
        assert!(world.person_news_mailboxes
            .mailbox_for(555).is_empty());
    }
}
