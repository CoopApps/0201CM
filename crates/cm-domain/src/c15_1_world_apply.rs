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
//! Club finance writes           | `Club.raw` finance offsets            | DEFERRED — the cash offset is disputed (`ClubView::cash() -> i32 @ +0x65` vs the C14 `i64 @ Club[0]`); surfaced as `PendingFinanceWrite` until the archaeology is settled
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
/// values in a typed sidecar. When the loader trace lands
/// (planned follow-up), we know whether to move these into
/// `Club.raw[…]` bytes or keep the sidecar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq,
         serde::Serialize, serde::Deserialize)]
pub struct ClubFinanceState {
    /// Cash — i64 signed. Runtime object byte offset `+0x00`.
    pub cash: i64,
    /// Season misc operating expense — i32.
    /// Runtime byte `+0x8C`. Reset annually by season roll.
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

/// Per-club finance ledger. Populated by C15.1A's finance apply
/// pass from C14 outputs. Keyed by club id.
#[derive(Debug, Clone, Default, PartialEq, Eq,
         serde::Serialize, serde::Deserialize)]
pub struct ClubFinanceLedger {
    pub per_club: std::collections::BTreeMap<u32, ClubFinanceState>,
}

impl ClubFinanceLedger {
    pub fn new() -> Self { Self::default() }

    /// Read the club's current finance state (or default if absent).
    pub fn get(&self, club_id: u32) -> ClubFinanceState {
        self.per_club.get(&club_id).copied().unwrap_or_default()
    }

    /// Apply one finance write from C14 (via `PendingFinanceWrite`).
    /// C14 has already computed the NEW post-transaction values
    /// (cash after debit/subsidy, accumulators after add). The
    /// ledger just stores them.
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
/// into `RuntimeSaveGame.finance_ledger`; the pending vector on
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
    // apply pass into the save's finance ledger. Trace-derived:
    // PendingFinanceWrite records what C14 produced; the ledger
    // reflects what actually landed.
    for w in &out.pending_finance {
        save.finance_ledger.apply_write(w);
    }
    // C15.1B: materialise promotion/relegation staff-state
    // writes onto `world.contracts` (the runtime contract pool
    // at `DAT_00accad8`, per memory
    // [[contract-clauses-generated-at-boot]]). Trace-derived:
    // AppliedContractWrite is pushed only after a real byte
    // mutation.
    if let Some(contracts) = world.contracts.as_mut() {
        apply_contract_writes_from_report(contracts, report, &mut out);
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
                        contracts, pe, /*is_promotion=*/true, out,
                    );
                }
            }
            YearEndMutationEvent::Relegation { effects } => {
                for pe in &effects.person_effects {
                    apply_contract_write_for_person(
                        contracts, pe, /*is_promotion=*/false, out,
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
    out: &mut WorldApplyReport,
) {
    // Chain: person_id → contract idx via by_staff_id → record.
    // Note: the exe uses staff_id here, and in the shipped exe
    // the port's `person_id` on `PersonEffect` corresponds to
    // that same identity (staff_id and person_id are aliased —
    // see ContractRecord docstring for +0x00 vs +0x04).
    let Some(record) = contracts.contract_for_staff_mut(pe.person_id)
        else { return };
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
                outcome, club_id, stadium_id,
            } => {
                apply_stadium_expansion_outcome(
                    world, pending_events, &date, day, outcome,
                    *club_id, *stadium_id, &mut out,
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
    fn pool_with_one_contract(
        person_id: u32, relegation: u8, non_promotion: u8,
    ) -> ContractPool {
        let mut pool = ContractPool::default();
        pool.records.push(ContractRecord {
            staff_id: person_id as i32,
            person_id: person_id as i32,
            wage: 0, value: 0,
            non_promotion, minimum_fee: 0, non_playing: 0,
            relegation, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
        });
        // Size by_staff_id to at least person_id + 1.
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
            staff_id: 1, person_id: 1, wage: 0, value: 0,
            non_promotion: 0, minimum_fee: 0, non_playing: 0,
            relegation: 1, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 0,
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
}
