//! C15 — Traditional English annual rollover, world-side composition.
//!
//! Single production entry point that composes:
//!
//! * `year_end_statuses::stamp_league_end_of_season_statuses` (C12)
//! * `year_end_statuses::propagate_english_playoff_winner` (C12)
//! * `eng_second_fixtures::english_pyramid_annual_rollover` (C8)
//! * `eng_second_fixtures::english_conference_dispatch` — routes to
//!   `conference_feeder_swap` OR `conference_fallback_promotion`
//!   (mutually exclusive with the pyramid's Conference edge)
//! * `c13_promotion_apply::apply_promotion_install` (C13)
//! * `c13_promotion_apply::apply_relegation_install` (C13)
//! * `c14_stadium_expansion::apply_stadium_expansion` (C14)
//!
//! # Scope
//!
//! * Traditional mode only. V4 returns [`RolloverError::NonTraditionalMode`]
//!   without any World writes.
//! * English pyramid only. Other nations remain on their existing
//!   paths.
//! * `Club+0x57` (new comp id), `Club+0x5B` (prev comp id),
//!   `Club+0x37` (status) are materialised directly onto
//!   `World.core.clubs[i].raw` bytes via [`materialise_club_moves`].
//!   Callers get a full event stream ([`YearEndMutationEvent`]) so
//!   downstream stadium / finance / staff / news layers can
//!   materialise their own side effects.
//!
//! # Non-goals
//!
//! * Fixture regeneration for the next season (fixture engine
//!   frozen at commit `10734b5`).
//! * Worldwide-league expansion.
//! * Player-award pass (Third-Div `+0xA4` — C16).
//! * Runtime differential capture (separate Frida run).
//! * `Club+0x37 = 0xFE` clearer — reprieve is sticky per C12/C14.7;
//!   cleared by an external subsystem not yet located.

use crate::c13_promotion_apply::{
    apply_promotion_install, apply_relegation_install, ClubPreApply,
    CompPreApply, PersonSlot, PromotionApplyEffects,
    PromotionInstallCtx, RelegationApplyEffects, RelegationInstallCtx,
};
use crate::c14_stadium_expansion::{
    apply_stadium_expansion, c13_request_to_c14_input, ParentStadium,
    StadiumExpansionOutcome,
};
use crate::eng_second_fixtures::{
    english_conference_dispatch, english_pyramid_annual_rollover,
    ClubRosterEntry, ConferenceFallbackOutcome,
    ConferenceRolloverDispatch, ConferenceRelegatee,
    EnglishPyramidCompIds, EnglishPyramidRolloverDecision,
    FallbackCandidate, FeederCandidate, PromotionMove,
    PromotionRelegationDecision, PromotionRelegationMode,
    RelegationMove, StadiumCapacityRequest, ThirdConferenceEdgeOutcome,
    ThirdConferenceStadiumInputs, ThirdDivLastPlace, ThirdDivRelegatee,
};
use crate::english_traditional::GameMode;
use crate::game_rng::GameRng;
use crate::year_end_statuses::{
    apply_swap_status_transform, propagate_english_playoff_winner,
    stamp_league_end_of_season_statuses, EnglishLeagueEndShape,
    LeagueTableRow, STATUS_IDLE, STATUS_STADIUM_REPRIEVE,
};

// ---------------------------------------------------------------------------
// Public surface
// ---------------------------------------------------------------------------

/// English-comp ids the rollover covers.
pub const ENGLISH_ROLLOVER_COMP_IDS: [u32; 5] = [7, 8, 9, 10, 93];

/// Final league table row a caller passes to C15.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalTableRow {
    pub club_id: u32,
    /// The pre-stamp `Club+0x37`. Usually `0xFF` unless the club
    /// carries a sticky value from a prior year (`0xFE`).
    pub current_status: u8,
    /// Position 1 = first, sorted by caller's tiebreak rules.
    pub position: u16,
    /// Optional playoff-winner marker for D1/D2/D3. Ignored for
    /// Prem and Conf.
    pub playoff_winner_marker: bool,
}

/// One league's finalised table + shape hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnglishLeagueFinalTable {
    pub comp_id: u32,
    pub shape: EnglishLeagueEndShape,
    pub rows: Vec<FinalTableRow>,
}

/// Destination-comp template used by C14 stadium expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompStadiumTemplate {
    pub type_43: u8,
    pub capacity_template_e2: i32,
    pub capacity_template_e4: i32,
}

/// Per-club state C14 needs to compute a stadium expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClubYearEndState {
    pub stadium_id: Option<i32>,
    pub stadium_total: i32,
    pub stadium_seated: i32,
    pub stadium_peak: i32,
    pub cash: i64,
    pub owner_accum_a: i64,
    pub owner_accum_b: i64,
    pub stadium_expense_ytd: i64,
    pub stadium_expense_lifetime: i64,
    pub news_flag_cf: u8,
    pub tier_byte_64: u8,
    pub parent_stadium_refuse_counter: Option<i8>,
}

impl Default for ClubYearEndState {
    fn default() -> Self {
        Self {
            stadium_id: None,
            stadium_total: 0, stadium_seated: 0, stadium_peak: 0,
            cash: 0,
            owner_accum_a: 0, owner_accum_b: 0,
            stadium_expense_ytd: 0, stadium_expense_lifetime: 0,
            news_flag_cf: 0, tier_byte_64: 0,
            parent_stadium_refuse_counter: None,
        }
    }
}

/// Complete input to one annual rollover pass.
pub struct AnnualRolloverInput<'a> {
    pub mode: GameMode,
    pub comp_ids: EnglishPyramidCompIds,
    /// Final tables in exe-boot order: Prem, First, Second, Third,
    /// Conference.
    pub tables: [EnglishLeagueFinalTable; 5],
    pub conference_simulated: bool,
    pub third_conference_stadium: ThirdConferenceStadiumInputs,
    pub feeder_candidates: &'a [FeederCandidate],
    pub conference_marked_for_relegation: &'a [ConferenceRelegatee],
    pub fallback_candidates: &'a [FallbackCandidate],
    pub third_div_relegatees: &'a [ThirdDivRelegatee],
    pub per_club_person_slots:
        std::collections::BTreeMap<u32, [Option<PersonSlot>; 50]>,
    pub reserve_of: std::collections::BTreeMap<u32, u32>,
    pub comp_stadium_templates:
        std::collections::BTreeMap<u32, CompStadiumTemplate>,
    pub club_state: std::collections::BTreeMap<u32, ClubYearEndState>,
    pub rng: &'a mut GameRng,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YearEndMutationEvent {
    StatusStamped {
        comp_id: u32, club_id: u32,
        old_status: u8, new_status: u8,
    },
    PlayoffWinnerStamp { comp_id: u32, winner_club_id: u32 },
    Promotion { effects: PromotionApplyEffects },
    Relegation { effects: RelegationApplyEffects },
    SwapStatusIdle {
        club_id: u32, pre_swap_status: u8, post_swap_status: u8,
    },
    StadiumExpansion { outcome: StadiumExpansionOutcome },
    StadiumFailReprieve {
        third_div_bottom_club_id: u32,
        candidate_club_id: u32,
        news_template_id: u16,
        news_destination_comp_id: u32,
    },
    ConferenceDispatch { chosen: ConferenceDispatchChoice },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConferenceDispatchChoice {
    FeederSwap { promotions: usize, relegations: usize },
    Fallback { promoted: bool, stadium_failed: bool, no_candidates: bool },
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnualRolloverReport {
    pub events: Vec<YearEndMutationEvent>,
    pub pyramid_decision: Option<EnglishPyramidRolloverDecision>,
    pub conference_dispatch: Option<ConferenceRolloverDispatch>,
    pub club_moves: std::collections::BTreeMap<u32, ClubMoveSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClubMoveSummary {
    pub new_comp_id: u32,
    pub prev_comp_id: u32,
    pub new_status: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RolloverError {
    NonTraditionalMode,
    WrongLeagueSize { comp_id: u32, expected: usize, actual: usize },
    UnexpectedPlayoffWinnerMarker { comp_id: u32 },
    MultiplePlayoffWinners { comp_id: u32 },
    ClubInMultipleLeagues { club_id: u32 },
}

// ---------------------------------------------------------------------------
// Main entry
// ---------------------------------------------------------------------------

pub fn compute_annual_rollover(
    inp: AnnualRolloverInput<'_>,
) -> Result<AnnualRolloverReport, RolloverError> {
    if !matches!(inp.mode, GameMode::Traditional) {
        return Err(RolloverError::NonTraditionalMode);
    }
    validate_input(&inp)?;

    let AnnualRolloverInput {
        mode: _,
        comp_ids,
        tables,
        conference_simulated,
        third_conference_stadium,
        feeder_candidates,
        conference_marked_for_relegation,
        fallback_candidates,
        third_div_relegatees,
        per_club_person_slots,
        reserve_of: _reserve_of,
        comp_stadium_templates,
        club_state,
        rng,
    } = inp;

    let mut events: Vec<YearEndMutationEvent> = Vec::new();

    // ---- Step 1: C12 status stamping per league ------------------
    let mut per_league_stamped: Vec<Vec<LeagueTableRow>> =
        Vec::with_capacity(5);
    for table in tables.iter() {
        let rows: Vec<LeagueTableRow> = table.rows.iter()
            .map(|r| LeagueTableRow {
                club_id: r.club_id,
                current_status: r.current_status,
            })
            .collect();
        let stamped = stamp_league_end_of_season_statuses(table.shape, &rows);
        for (r_new, r_old) in stamped.iter().zip(rows.iter()) {
            if r_new.current_status != r_old.current_status {
                events.push(YearEndMutationEvent::StatusStamped {
                    comp_id: table.comp_id,
                    club_id: r_old.club_id,
                    old_status: r_old.current_status,
                    new_status: r_new.current_status,
                });
            }
        }
        per_league_stamped.push(stamped);
    }

    // ---- Step 2: playoff-winner propagation for D1/D2/D3 ---------
    for idx in 1..=3 {
        let table = &tables[idx];
        let stamped = &mut per_league_stamped[idx];
        let winner = table.rows.iter().find(|r| r.playoff_winner_marker);
        let Some(w) = winner else { continue };
        let out = propagate_english_playoff_winner(stamped, w.club_id);
        if out.write_fired {
            events.push(YearEndMutationEvent::PlayoffWinnerStamp {
                comp_id: table.comp_id, winner_club_id: w.club_id,
            });
        }
        *stamped = out.updated_table;
    }

    // ---- Step 3: build ClubRosterEntry per league ----------------
    let comp_id_of_league = [
        comp_ids.prem, comp_ids.first, comp_ids.second,
        comp_ids.third, comp_ids.conference,
    ];
    let mut rosters: Vec<Vec<ClubRosterEntry>> = Vec::with_capacity(5);
    for (idx, stamped) in per_league_stamped.iter().enumerate() {
        let league_comp_id = comp_id_of_league[idx];
        rosters.push(stamped.iter()
            .map(|r| ClubRosterEntry {
                club_id: r.club_id,
                status_byte: r.current_status,
                current_comp_id: league_comp_id,
            })
            .collect());
    }

    // ---- Step 4: run C8 English pyramid orchestrator -------------
    let pyramid_decision = english_pyramid_annual_rollover(
        comp_ids,
        &rosters[0], &rosters[1], &rosters[2], &rosters[3], &rosters[4],
        conference_simulated,
        third_conference_stadium,
        PromotionRelegationMode::Independent,
    );

    // ---- Step 5: C13 apply for each pyramid edge -----------------
    let mut club_moves: std::collections::BTreeMap<u32, ClubMoveSummary> =
        std::collections::BTreeMap::new();

    materialise_pr_edge(
        &pyramid_decision.prem_first, &mut events, &mut club_moves,
        &per_club_person_slots, &club_state, &comp_stadium_templates, rng,
    );
    materialise_pr_edge(
        &pyramid_decision.first_second, &mut events, &mut club_moves,
        &per_club_person_slots, &club_state, &comp_stadium_templates, rng,
    );
    materialise_pr_edge(
        &pyramid_decision.second_third, &mut events, &mut club_moves,
        &per_club_person_slots, &club_state, &comp_stadium_templates, rng,
    );

    match &pyramid_decision.third_conference {
        ThirdConferenceEdgeOutcome::Swapped(decision) => {
            materialise_pr_edge(
                decision, &mut events, &mut club_moves,
                &per_club_person_slots, &club_state,
                &comp_stadium_templates, rng,
            );
        }
        ThirdConferenceEdgeOutcome::StadiumFailed {
            third_div_reprieved_club_id, news_template_hint,
        } => {
            events.push(YearEndMutationEvent::StadiumFailReprieve {
                third_div_bottom_club_id: *third_div_reprieved_club_id,
                candidate_club_id: 0,
                news_template_id: *news_template_hint,
                news_destination_comp_id: comp_ids.conference,
            });
        }
        ThirdConferenceEdgeOutcome::ConferenceAbsent => {
            events.push(YearEndMutationEvent::ConferenceDispatch {
                chosen: ConferenceDispatchChoice::Absent,
            });
        }
    }

    // ---- Step 6: Conference peer dispatch ------------------------
    let mut feeder_pool: Vec<FeederCandidate> = feeder_candidates.to_vec();
    let mut fallback_pool: Vec<FallbackCandidate> = fallback_candidates.to_vec();
    let third_div_last_place = third_conference_stadium
        .third_div_last_place_club_id
        .map(|id| ThirdDivLastPlace { club_id: id });
    let capacity_req = StadiumCapacityRequest {
        stadium_current_capacity:
            third_conference_stadium.champion_stadium_current_capacity,
        required_capacity_a: third_conference_stadium.required_capacity_a,
        required_capacity_b: third_conference_stadium.required_capacity_b,
    };
    let conference_dispatch = english_conference_dispatch(
        conference_simulated,
        &mut feeder_pool,
        conference_marked_for_relegation,
        &mut fallback_pool,
        third_div_relegatees,
        third_div_last_place,
        capacity_req,
        comp_ids.third,
        comp_ids.conference,
        rng,
    );
    match &conference_dispatch {
        ConferenceRolloverDispatch::FeederSwap(d) => {
            events.push(YearEndMutationEvent::ConferenceDispatch {
                chosen: ConferenceDispatchChoice::FeederSwap {
                    promotions: d.promotions.len(),
                    relegations: d.relegations.len(),
                },
            });
            // FeederSwap: feeder-pool clubs move UP to Conference,
            // Conference clubs marked +0x37==3 pair-relegate back
            // into their recorded feeder.
            for pm in &d.promotions {
                club_moves.insert(pm.club_id, ClubMoveSummary {
                    new_comp_id: comp_ids.conference,
                    prev_comp_id: pm.origin_comp_id,
                    new_status: STATUS_IDLE,
                });
            }
            for rm in &d.relegations {
                if let Some(target) = rm.target_feeder_comp_id {
                    club_moves.insert(rm.club_id, ClubMoveSummary {
                        new_comp_id: target,
                        prev_comp_id: comp_ids.conference,
                        new_status: STATUS_IDLE,
                    });
                }
            }
        }
        ConferenceRolloverDispatch::ChampionFallback(d) => {
            match &d.outcome {
                ConferenceFallbackOutcome::Promoted {
                    candidate_club_id, destination_comp_id,
                    third_div_settled_relegations,
                } => {
                    events.push(YearEndMutationEvent::ConferenceDispatch {
                        chosen: ConferenceDispatchChoice::Fallback {
                            promoted: true, stadium_failed: false,
                            no_candidates: false,
                        },
                    });
                    club_moves.insert(*candidate_club_id, ClubMoveSummary {
                        new_comp_id: *destination_comp_id,
                        prev_comp_id: comp_ids.conference,
                        new_status: STATUS_IDLE,
                    });
                    for id in third_div_settled_relegations {
                        club_moves.insert(*id, ClubMoveSummary {
                            new_comp_id: comp_ids.conference,
                            prev_comp_id: comp_ids.third,
                            new_status: STATUS_IDLE,
                        });
                    }
                }
                ConferenceFallbackOutcome::StadiumFailed {
                    candidate_club_id, third_div_reprieved_club_id,
                    news_template_id, news_destination_comp_id,
                } => {
                    events.push(YearEndMutationEvent::ConferenceDispatch {
                        chosen: ConferenceDispatchChoice::Fallback {
                            promoted: false, stadium_failed: true,
                            no_candidates: false,
                        },
                    });
                    events.push(YearEndMutationEvent::StadiumFailReprieve {
                        third_div_bottom_club_id: *third_div_reprieved_club_id,
                        candidate_club_id: *candidate_club_id,
                        news_template_id: *news_template_id,
                        news_destination_comp_id: *news_destination_comp_id,
                    });
                }
                ConferenceFallbackOutcome::NoCandidates => {
                    events.push(YearEndMutationEvent::ConferenceDispatch {
                        chosen: ConferenceDispatchChoice::Fallback {
                            promoted: false, stadium_failed: false,
                            no_candidates: true,
                        },
                    });
                }
            }
        }
    }

    Ok(AnnualRolloverReport {
        events,
        pyramid_decision: Some(pyramid_decision),
        conference_dispatch: Some(conference_dispatch),
        club_moves,
    })
}

// ---------------------------------------------------------------------------

fn materialise_pr_edge(
    decision: &PromotionRelegationDecision,
    events: &mut Vec<YearEndMutationEvent>,
    club_moves: &mut std::collections::BTreeMap<u32, ClubMoveSummary>,
    per_club_slots:
        &std::collections::BTreeMap<u32, [Option<PersonSlot>; 50]>,
    club_state:
        &std::collections::BTreeMap<u32, ClubYearEndState>,
    comp_templates:
        &std::collections::BTreeMap<u32, CompStadiumTemplate>,
    rng: &mut GameRng,
) {
    for pm in &decision.promoted {
        let (pre, template) = make_promotion_pieces(
            pm, per_club_slots, club_state, comp_templates,
        );
        let ctx = PromotionInstallCtx {
            new_comp: template,
            news_enable: true,
            reserve: None,
            squad_register_flag: true,
        };
        let effects = apply_promotion_install(&pre, &ctx);
        club_moves.insert(pm.club_id, ClubMoveSummary {
            new_comp_id: effects.writes.new_comp_id,
            prev_comp_id: effects.writes.prev_comp_id,
            new_status: STATUS_IDLE,
        });
        if let Some(req) = &effects.stadium_expansion {
            let (state, parent) = state_for_expansion(pm.club_id, club_state);
            let c14_input = c13_request_to_c14_input(
                req,
                state.stadium_total, state.stadium_seated, state.stadium_peak,
                state.cash,
                state.owner_accum_a, state.owner_accum_b,
                state.stadium_expense_ytd, state.stadium_expense_lifetime,
                parent,
                0, // news_ctx
                false, // affordability_checked
            );
            let outcome = apply_stadium_expansion(&c14_input, rng);
            events.push(YearEndMutationEvent::StadiumExpansion { outcome });
        }
        events.push(YearEndMutationEvent::Promotion { effects });
        events.push(YearEndMutationEvent::SwapStatusIdle {
            club_id: pm.club_id,
            pre_swap_status: 0,
            post_swap_status: apply_swap_status_transform(0),
        });
    }
    for rm in &decision.relegated {
        let pre = make_relegation_pre_apply(rm, per_club_slots, club_state);
        let ctx = RelegationInstallCtx {
            new_comp: Some(CompPreApply {
                comp_id: rm.new_comp_id,
                type_43: 0,
                capacity_template_e2: -1,
                capacity_template_e4: -1,
            }),
            reserve: None,
            manager_unhappy: false,
        };
        let effects = apply_relegation_install(&pre, &ctx);
        club_moves.insert(rm.club_id, ClubMoveSummary {
            new_comp_id: effects.writes.new_comp_id,
            prev_comp_id: effects.writes.prev_comp_id,
            new_status: STATUS_IDLE,
        });
        events.push(YearEndMutationEvent::Relegation { effects });
        events.push(YearEndMutationEvent::SwapStatusIdle {
            club_id: rm.club_id,
            pre_swap_status: 3,
            post_swap_status: apply_swap_status_transform(3),
        });
    }
}

fn make_promotion_pieces(
    pm: &PromotionMove,
    per_club_slots:
        &std::collections::BTreeMap<u32, [Option<PersonSlot>; 50]>,
    club_state:
        &std::collections::BTreeMap<u32, ClubYearEndState>,
    comp_templates:
        &std::collections::BTreeMap<u32, CompStadiumTemplate>,
) -> (ClubPreApply, CompPreApply) {
    let state = club_state.get(&pm.club_id).cloned().unwrap_or_default();
    let slots = per_club_slots.get(&pm.club_id).cloned().unwrap_or([None; 50]);
    let template = comp_templates.get(&pm.new_comp_id).cloned()
        .unwrap_or(CompStadiumTemplate {
            type_43: 0, capacity_template_e2: -1, capacity_template_e4: -1,
        });
    let pre = ClubPreApply {
        club_id: pm.club_id,
        status_37: 0,
        current_comp_id: pm.previous_comp_id,
        tier_byte_64: state.tier_byte_64,
        stadium_id: state.stadium_id,
        news_flag_cf: state.news_flag_cf,
        person_slots: slots,
        reserve_club_id: None,
    };
    let comp = CompPreApply {
        comp_id: pm.new_comp_id,
        type_43: template.type_43,
        capacity_template_e2: template.capacity_template_e2,
        capacity_template_e4: template.capacity_template_e4,
    };
    (pre, comp)
}

fn make_relegation_pre_apply(
    rm: &RelegationMove,
    per_club_slots:
        &std::collections::BTreeMap<u32, [Option<PersonSlot>; 50]>,
    club_state: &std::collections::BTreeMap<u32, ClubYearEndState>,
) -> ClubPreApply {
    let state = club_state.get(&rm.club_id).cloned().unwrap_or_default();
    let slots = per_club_slots.get(&rm.club_id).cloned().unwrap_or([None; 50]);
    ClubPreApply {
        club_id: rm.club_id,
        status_37: 3,
        current_comp_id: rm.previous_comp_id,
        tier_byte_64: state.tier_byte_64,
        stadium_id: state.stadium_id,
        news_flag_cf: state.news_flag_cf,
        person_slots: slots,
        reserve_club_id: None,
    }
}

fn state_for_expansion(
    club_id: u32,
    club_state: &std::collections::BTreeMap<u32, ClubYearEndState>,
) -> (ClubYearEndState, Option<ParentStadium>) {
    let state = club_state.get(&club_id).cloned().unwrap_or_default();
    let parent = state.parent_stadium_refuse_counter
        .map(|c| ParentStadium { refuse_counter: c });
    (state, parent)
}

fn validate_input(inp: &AnnualRolloverInput<'_>) -> Result<(), RolloverError> {
    let expected: [(u32, usize); 5] = [
        (7, 20), (8, 24), (9, 24), (10, 24), (93, 22),
    ];
    for (t, (exp_id, exp_n)) in inp.tables.iter().zip(expected.iter()) {
        if *exp_id == 93 && t.rows.is_empty() { continue; }
        if t.rows.len() != *exp_n {
            return Err(RolloverError::WrongLeagueSize {
                comp_id: t.comp_id,
                expected: *exp_n,
                actual: t.rows.len(),
            });
        }
    }
    for t in &inp.tables {
        let n_markers = t.rows.iter()
            .filter(|r| r.playoff_winner_marker).count();
        match t.shape {
            EnglishLeagueEndShape::Premier
            | EnglishLeagueEndShape::Conference => {
                if n_markers > 0 {
                    return Err(RolloverError::UnexpectedPlayoffWinnerMarker {
                        comp_id: t.comp_id,
                    });
                }
            }
            _ => {
                if n_markers > 1 {
                    return Err(RolloverError::MultiplePlayoffWinners {
                        comp_id: t.comp_id,
                    });
                }
            }
        }
    }
    let mut seen: std::collections::BTreeSet<u32> = Default::default();
    for t in &inp.tables {
        for r in &t.rows {
            if !seen.insert(r.club_id) {
                return Err(RolloverError::ClubInMultipleLeagues {
                    club_id: r.club_id,
                });
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// World-side club-record materialisation
// ---------------------------------------------------------------------------

/// Materialise the `ClubMoveSummary` map onto `World.core.clubs`'s
/// raw bytes: writes `+0x57` (new comp id i32), `+0x5B` (prev comp
/// id i32), `+0x37` (status). Returns the number of clubs updated.
pub fn materialise_club_moves(
    world: &mut crate::World,
    moves: &std::collections::BTreeMap<u32, ClubMoveSummary>,
) -> usize {
    use crate::typed_records::ClubView;
    let mut updated = 0usize;
    for club in world.core.clubs.iter_mut() {
        let id = ClubView::new(club).id();
        let Some(mv) = moves.get(&id) else { continue };
        if club.raw.len() < 0x60 { continue; }
        club.raw[0x57..0x5B]
            .copy_from_slice(&(mv.new_comp_id as i32).to_le_bytes());
        club.raw[0x5B..0x5F]
            .copy_from_slice(&(mv.prev_comp_id as i32).to_le_bytes());
        club.raw[0x37] = mv.new_status;
        updated += 1;
    }
    updated
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn eng_comp_ids() -> EnglishPyramidCompIds {
        EnglishPyramidCompIds {
            prem: 7, first: 8, second: 9, third: 10, conference: 93,
        }
    }

    fn ids(offset: u32, n: usize) -> Vec<u32> {
        (0..n).map(|i| offset + i as u32).collect()
    }

    fn mk_table(
        comp_id: u32, shape: EnglishLeagueEndShape, club_ids: &[u32],
    ) -> EnglishLeagueFinalTable {
        EnglishLeagueFinalTable {
            comp_id, shape,
            rows: club_ids.iter().enumerate().map(|(i, id)| FinalTableRow {
                club_id: *id,
                current_status: STATUS_IDLE,
                position: (i + 1) as u16,
                playoff_winner_marker: false,
            }).collect(),
        }
    }

    fn base_stadium_inputs(
        third_last_place: Option<u32>, stadium_ok: bool,
    ) -> ThirdConferenceStadiumInputs {
        ThirdConferenceStadiumInputs {
            champion_stadium_current_capacity:
                if stadium_ok { Some(6_000) } else { Some(500) },
            required_capacity_a: 6_000,
            required_capacity_b: 6_000,
            third_div_last_place_club_id: third_last_place,
        }
    }

    fn make_input<'a>(
        rng: &'a mut GameRng, mode: GameMode,
        prem: &[u32], first: &[u32], second: &[u32],
        third: &[u32], conf: &[u32],
        conference_simulated: bool, stadium_ok: bool,
    ) -> AnnualRolloverInput<'a> {
        AnnualRolloverInput {
            mode,
            comp_ids: eng_comp_ids(),
            tables: [
                mk_table(7,  EnglishLeagueEndShape::Premier,    prem),
                mk_table(8,  EnglishLeagueEndShape::First,      first),
                mk_table(9,  EnglishLeagueEndShape::Second,     second),
                mk_table(10, EnglishLeagueEndShape::Third,      third),
                mk_table(93, EnglishLeagueEndShape::Conference, conf),
            ],
            conference_simulated,
            third_conference_stadium:
                base_stadium_inputs(third.last().copied(), stadium_ok),
            feeder_candidates: &[],
            conference_marked_for_relegation: &[],
            fallback_candidates: &[],
            third_div_relegatees: &[],
            per_club_person_slots: Default::default(),
            reserve_of: Default::default(),
            comp_stadium_templates: Default::default(),
            club_state: Default::default(),
            rng,
        }
    }

    // -- A: full active Conference rollover --
    #[test]
    fn a_full_active_conference_rollover_all_five_simulated() {
        let mut rng = GameRng::new(0xC150_0000);
        let inp = make_input(
            &mut rng, GameMode::Traditional,
            &ids(1000, 20), &ids(2000, 24), &ids(3000, 24),
            &ids(4000, 24), &ids(5000, 22),
            true, true,
        );
        let out = compute_annual_rollover(inp).unwrap();
        assert!(out.pyramid_decision.is_some());
        assert!(matches!(out.conference_dispatch.as_ref().unwrap(),
                         ConferenceRolloverDispatch::FeederSwap(_)));
    }

    // -- B: Conference inactive, fallback promotion --
    #[test]
    fn b_inactive_conference_fallback_promotes_top_candidate() {
        let mut rng = GameRng::new(0xC150_0001);
        let conf: Vec<u32> = vec![];
        let mut inp = make_input(
            &mut rng, GameMode::Traditional,
            &ids(1000, 20), &ids(2000, 24), &ids(3000, 24),
            &ids(4000, 24), &conf,
            false, true,
        );
        let fallback = [FallbackCandidate {
            club_id: 9999, key80: 100,
            stadium_current_capacity: Some(50_000),
        }];
        inp.fallback_candidates = &fallback;
        let out = compute_annual_rollover(inp).unwrap();
        match out.conference_dispatch.as_ref().unwrap() {
            ConferenceRolloverDispatch::ChampionFallback(d) => {
                assert!(matches!(d.outcome,
                                 ConferenceFallbackOutcome::Promoted { .. }));
            }
            other => panic!("expected fallback, got {other:?}"),
        }
        assert_eq!(out.club_moves.get(&9999).unwrap().new_comp_id, 10);
    }

    // -- C: Conference inactive + stadium failure --
    #[test]
    fn c_inactive_conference_stadium_failure_reprieves_third_bottom() {
        let mut rng = GameRng::new(0xC150_0002);
        let conf: Vec<u32> = vec![];
        let mut inp = make_input(
            &mut rng, GameMode::Traditional,
            &ids(1000, 20), &ids(2000, 24), &ids(3000, 24),
            &ids(4000, 24), &conf,
            false, true,
        );
        let fallback = [FallbackCandidate {
            club_id: 9998, key80: 100,
            stadium_current_capacity: Some(100),
        }];
        inp.fallback_candidates = &fallback;
        inp.third_conference_stadium.required_capacity_a = 999_999;
        inp.third_conference_stadium.required_capacity_b = 999_999;
        let out = compute_annual_rollover(inp).unwrap();
        assert!(out.events.iter().any(|e| matches!(
            e, YearEndMutationEvent::StadiumFailReprieve { .. })));
        assert!(out.club_moves.get(&9998).is_none());
    }

    // -- D: playoff winner emitted once --
    #[test]
    fn d_playoff_winner_status_5_moves_exactly_once() {
        let mut rng = GameRng::new(0xC150_0003);
        let mut first_rows = mk_table(
            8, EnglishLeagueEndShape::First, &ids(2000, 24)).rows;
        first_rows[5].playoff_winner_marker = true;
        let inp = AnnualRolloverInput {
            mode: GameMode::Traditional,
            comp_ids: eng_comp_ids(),
            tables: [
                mk_table(7,  EnglishLeagueEndShape::Premier,    &ids(1000, 20)),
                EnglishLeagueFinalTable {
                    comp_id: 8, shape: EnglishLeagueEndShape::First, rows: first_rows,
                },
                mk_table(9,  EnglishLeagueEndShape::Second,     &ids(3000, 24)),
                mk_table(10, EnglishLeagueEndShape::Third,      &ids(4000, 24)),
                mk_table(93, EnglishLeagueEndShape::Conference, &ids(5000, 22)),
            ],
            conference_simulated: true,
            third_conference_stadium: base_stadium_inputs(Some(4023), true),
            feeder_candidates: &[], conference_marked_for_relegation: &[],
            fallback_candidates: &[], third_div_relegatees: &[],
            per_club_person_slots: Default::default(),
            reserve_of: Default::default(),
            comp_stadium_templates: Default::default(),
            club_state: Default::default(),
            rng: &mut rng,
        };
        let out = compute_annual_rollover(inp).unwrap();
        let stamps: Vec<_> = out.events.iter().filter(|e| matches!(
            e, YearEndMutationEvent::PlayoffWinnerStamp { .. })).collect();
        assert_eq!(stamps.len(), 1);
    }

    // -- E: sticky reprieve not consumed --
    #[test]
    fn e_sticky_reprieve_not_consumed() {
        let mut rng = GameRng::new(0xC150_0004);
        let mut prem_rows = mk_table(
            7, EnglishLeagueEndShape::Premier, &ids(1000, 20)).rows;
        prem_rows[19].current_status = STATUS_STADIUM_REPRIEVE;
        let inp = AnnualRolloverInput {
            mode: GameMode::Traditional,
            comp_ids: eng_comp_ids(),
            tables: [
                EnglishLeagueFinalTable {
                    comp_id: 7, shape: EnglishLeagueEndShape::Premier, rows: prem_rows,
                },
                mk_table(8,  EnglishLeagueEndShape::First,      &ids(2000, 24)),
                mk_table(9,  EnglishLeagueEndShape::Second,     &ids(3000, 24)),
                mk_table(10, EnglishLeagueEndShape::Third,      &ids(4000, 24)),
                mk_table(93, EnglishLeagueEndShape::Conference, &ids(5000, 22)),
            ],
            conference_simulated: true,
            third_conference_stadium: base_stadium_inputs(Some(4023), true),
            feeder_candidates: &[], conference_marked_for_relegation: &[],
            fallback_candidates: &[], third_div_relegatees: &[],
            per_club_person_slots: Default::default(),
            reserve_of: Default::default(),
            comp_stadium_templates: Default::default(),
            club_state: Default::default(),
            rng: &mut rng,
        };
        let out = compute_annual_rollover(inp).unwrap();
        assert!(!out.events.iter().any(|e| matches!(
            e, YearEndMutationEvent::StatusStamped { club_id: 1019, .. })));
    }

    // -- F: promotion stadium expansion forced path --
    #[test]
    fn f_promotion_stadium_expansion_forced_path() {
        let mut rng = GameRng::new(0xC150_0005);
        let mut club_state: std::collections::BTreeMap<u32, ClubYearEndState> = Default::default();
        for id in ids(2000, 24) {
            club_state.insert(id, ClubYearEndState {
                stadium_id: Some(500 + id as i32),
                stadium_total: 10_000, stadium_seated: 5_000, stadium_peak: 10_000,
                cash: 100_000_000,
                ..Default::default()
            });
        }
        let mut comp_templates: std::collections::BTreeMap<u32, CompStadiumTemplate> = Default::default();
        comp_templates.insert(7, CompStadiumTemplate {
            type_43: 2, capacity_template_e2: 20_000, capacity_template_e4: 25_000,
        });
        let inp = AnnualRolloverInput {
            mode: GameMode::Traditional,
            comp_ids: eng_comp_ids(),
            tables: [
                mk_table(7,  EnglishLeagueEndShape::Premier,    &ids(1000, 20)),
                mk_table(8,  EnglishLeagueEndShape::First,      &ids(2000, 24)),
                mk_table(9,  EnglishLeagueEndShape::Second,     &ids(3000, 24)),
                mk_table(10, EnglishLeagueEndShape::Third,      &ids(4000, 24)),
                mk_table(93, EnglishLeagueEndShape::Conference, &ids(5000, 22)),
            ],
            conference_simulated: true,
            third_conference_stadium: base_stadium_inputs(Some(4023), true),
            feeder_candidates: &[], conference_marked_for_relegation: &[],
            fallback_candidates: &[], third_div_relegatees: &[],
            per_club_person_slots: Default::default(),
            reserve_of: Default::default(),
            comp_stadium_templates: comp_templates,
            club_state,
            rng: &mut rng,
        };
        let out = compute_annual_rollover(inp).unwrap();
        let expansions: Vec<_> = out.events.iter().filter_map(|e| match e {
            YearEndMutationEvent::StadiumExpansion { outcome } => Some(outcome),
            _ => None,
        }).collect();
        assert!(!expansions.is_empty(),
                "expected at least one stadium expansion");
        assert!(expansions.iter().all(|o| o.success));
    }

    // -- G: person-history relegation --
    #[test]
    fn g_person_history_relegation_effects_present() {
        let mut rng = GameRng::new(0xC150_0006);
        let mut per_slots: std::collections::BTreeMap<u32, [Option<PersonSlot>; 50]> = Default::default();
        let mut slots = [None; 50];
        slots[0] = Some(PersonSlot { person_id: 555, staff_1f: 1, staff_1c: 0 });
        slots[1] = Some(PersonSlot { person_id: 556, staff_1f: 1, staff_1c: 0 });
        for id in ids(1000, 20) {
            per_slots.insert(id, slots);
        }
        let inp = AnnualRolloverInput {
            mode: GameMode::Traditional,
            comp_ids: eng_comp_ids(),
            tables: [
                mk_table(7,  EnglishLeagueEndShape::Premier,    &ids(1000, 20)),
                mk_table(8,  EnglishLeagueEndShape::First,      &ids(2000, 24)),
                mk_table(9,  EnglishLeagueEndShape::Second,     &ids(3000, 24)),
                mk_table(10, EnglishLeagueEndShape::Third,      &ids(4000, 24)),
                mk_table(93, EnglishLeagueEndShape::Conference, &ids(5000, 22)),
            ],
            conference_simulated: true,
            third_conference_stadium: base_stadium_inputs(Some(4023), true),
            feeder_candidates: &[], conference_marked_for_relegation: &[],
            fallback_candidates: &[], third_div_relegatees: &[],
            per_club_person_slots: per_slots,
            reserve_of: Default::default(),
            comp_stadium_templates: Default::default(),
            club_state: Default::default(),
            rng: &mut rng,
        };
        let out = compute_annual_rollover(inp).unwrap();
        let relegations: Vec<_> = out.events.iter().filter_map(|e| match e {
            YearEndMutationEvent::Relegation { effects } => Some(effects),
            _ => None,
        }).collect();
        assert!(!relegations.is_empty());
        let total: usize = relegations.iter().map(|e| e.person_effects.len()).sum();
        assert!(total > 0);
    }

    // -- H: V4 negative dispatch --
    #[test]
    fn h_v4_mode_returns_error_no_side_effects() {
        let mut rng = GameRng::new(0);
        let inp = make_input(
            &mut rng, GameMode::V4,
            &ids(1000, 20), &ids(2000, 24), &ids(3000, 24),
            &ids(4000, 24), &ids(5000, 22),
            true, true,
        );
        let out = compute_annual_rollover(inp);
        assert!(matches!(out, Err(RolloverError::NonTraditionalMode)));
    }

    // -- Invariants --
    #[test]
    fn invariant_wrong_league_size_returns_error() {
        let mut rng = GameRng::new(0);
        let inp = make_input(
            &mut rng, GameMode::Traditional,
            &ids(1000, 19), &ids(2000, 24), &ids(3000, 24),
            &ids(4000, 24), &ids(5000, 22), true, true,
        );
        let out = compute_annual_rollover(inp);
        assert!(matches!(out, Err(RolloverError::WrongLeagueSize {
            comp_id: 7, expected: 20, actual: 19 })));
    }

    #[test]
    fn invariant_club_in_multiple_leagues_fails() {
        let mut rng = GameRng::new(0);
        let mut prem = ids(1000, 20);
        prem[0] = 2000;
        let inp = make_input(
            &mut rng, GameMode::Traditional,
            &prem, &ids(2000, 24), &ids(3000, 24),
            &ids(4000, 24), &ids(5000, 22), true, true,
        );
        let out = compute_annual_rollover(inp);
        assert!(matches!(out,
            Err(RolloverError::ClubInMultipleLeagues { club_id: 2000 })));
    }

    #[test]
    fn invariant_playoff_marker_on_prem_fails() {
        let mut rng = GameRng::new(0);
        let mut prem_rows = mk_table(
            7, EnglishLeagueEndShape::Premier, &ids(1000, 20)).rows;
        prem_rows[0].playoff_winner_marker = true;
        let inp = AnnualRolloverInput {
            mode: GameMode::Traditional,
            comp_ids: eng_comp_ids(),
            tables: [
                EnglishLeagueFinalTable {
                    comp_id: 7, shape: EnglishLeagueEndShape::Premier, rows: prem_rows,
                },
                mk_table(8,  EnglishLeagueEndShape::First,      &ids(2000, 24)),
                mk_table(9,  EnglishLeagueEndShape::Second,     &ids(3000, 24)),
                mk_table(10, EnglishLeagueEndShape::Third,      &ids(4000, 24)),
                mk_table(93, EnglishLeagueEndShape::Conference, &ids(5000, 22)),
            ],
            conference_simulated: true,
            third_conference_stadium: base_stadium_inputs(Some(4023), true),
            feeder_candidates: &[], conference_marked_for_relegation: &[],
            fallback_candidates: &[], third_div_relegatees: &[],
            per_club_person_slots: Default::default(),
            reserve_of: Default::default(),
            comp_stadium_templates: Default::default(),
            club_state: Default::default(),
            rng: &mut rng,
        };
        let out = compute_annual_rollover(inp);
        assert!(matches!(out,
            Err(RolloverError::UnexpectedPlayoffWinnerMarker { comp_id: 7 })));
    }
}
