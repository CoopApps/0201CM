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

/// Finance writes surfaced but not applied — held pending until
/// the club raw finance offsets are re-confirmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingFinanceWrite {
    pub club_id: u32,
    pub new_cash: i64,
    pub new_stadium_expense_ytd: i64,
    pub new_stadium_expense_lifetime: i64,
    pub new_owner_accum_a: i64,
    pub new_owner_accum_b: i64,
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
    apply_report_to_world_parts(
        world, &mut save.pending_events, &date, day, report,
    )
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
            new_stadium_expense_ytd: cw.new_stadium_expense_ytd,
            new_stadium_expense_lifetime: cw.new_stadium_expense_lifetime,
            new_owner_accum_a: cw.new_owner_accum_a,
            new_owner_accum_b: cw.new_owner_accum_b,
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
                new_stadium_expense_ytd: 1_000_000,
                new_stadium_expense_lifetime: 1_000_000,
                new_owner_accum_a: 0,
                new_owner_accum_b: 0,
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
}
