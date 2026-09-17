//! C15.1G — canonical semantic snapshot of a Traditional English
//! year-end rollover, for cross-tool differential against the
//! authoritative GDI capture.
//!
//! # Why this exists
//!
//! C15.1A–F resolved every World-storage mapping. C15.1G asks:
//! after running the real production pipeline
//! (`apply_annual_rollover → apply_report_to_world`), does the
//! resulting World state match a Frida capture of the same
//! rollover on the authoritative `cm0102_GDI.exe`?
//!
//! To make that answerable, both sides need to produce a
//! **schema-stable, semantic-identity-keyed snapshot** (not raw
//! pointers). This module defines that schema for the Rust side
//! and provides `YearEndSnapshot::from_apply_report` /
//! `from_world` to build one from a real rollover run.
//!
//! The Python-side analyser at
//! `tools/c15_1g/analyse_gdi_capture.py` reads the Frida
//! JSONL and produces a `YearEndSnapshot`-shaped JSON so the
//! two sides diff verbatim.

use serde::{Deserialize, Serialize};

/// A single semantic event captured during year-end rollover.
///
/// The GDI Frida hooks and the Rust applier both emit a
/// sequence of these. Ordering matters for the call-order
/// differential; identity fields are compared for the
/// per-layer differentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum YearEndEvent {
    /// Status stamp (`FUN_00669FA0`) — pre/post `Club+0x37`.
    StatusStamped {
        club_id: u32,
        old_status: u8,
        new_status: u8,
    },
    /// Club movement — pre/post primary comp / previous comp /
    /// status. Captures the `Club+0x57` / `+0x5B` / `+0x37`
    /// bytes together so the differential can key on the
    /// club-move edge.
    ClubMoved {
        club_id: u32,
        old_primary_comp: u32,
        new_primary_comp: u32,
        old_previous_comp: u32,
        new_previous_comp: u32,
        old_status: u8,
        new_status: u8,
    },
    /// Contract clause byte write (C15.1B).
    ContractClauseWrite {
        person_id: u32,
        offset: u8,          // 0x1C or 0x1F
        old_value: u8,
        new_value: u8,
    },
    /// SquadRecord +0x3A write (C15.1C).
    SquadPositionWrite {
        person_id: u32,
        record_slot: SquadRecordSlotWire,
        old_value: i8,
        new_value: i8,
    },
    /// Person news / history append (C15.1E).
    PersonHistoryAppend {
        person_id: u32,
        category: u32,       // 0x0FBF
        kind: u8,            // 0=retire, 3=relegated
        old_comp_id: u32,
        staff_id: u32,
        news_id: u32,        // sequence within the mailbox pool
    },
    /// Stadium capacity write (C15.1A).
    StadiumCapacityWrite {
        stadium_id: u32,
        new_total: u32,
        new_seated: u32,
        new_peak: u32,
    },
    /// Stadium owner-refuse counter bump (C15.1D).
    StadiumRefuseBump {
        stadium_id: u32,
        old_value: i8,
        new_value: i8,
    },
    /// Finance write (C15.1F).
    FinanceWrite {
        club_id: u32,
        new_cash: i64,
        new_season_misc_expense: i32,
        new_lifetime_misc_expense: i32,
        new_season_subsidy_income: i32,
        new_lifetime_subsidy_income: i32,
    },
    /// News event queued (semantic template id only — internal
    /// handles don't compare across GDI vs Rust).
    NewsEmitted {
        template_id: u16,
        club_id: Option<u32>,
        competition_id: Option<u32>,
        kind: String,        // free-form tag ("promotion_welcome", …)
    },
}

/// Wire form of `SquadRecordSlot` for the snapshot — keeps the
/// snapshot schema decoupled from the applier's enum layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SquadRecordSlotWire {
    Primary,
    Secondary,
}

impl From<crate::c15_1_world_apply::SquadRecordSlot>
    for SquadRecordSlotWire
{
    fn from(v: crate::c15_1_world_apply::SquadRecordSlot) -> Self {
        match v {
            crate::c15_1_world_apply::SquadRecordSlot::Primary =>
                Self::Primary,
            crate::c15_1_world_apply::SquadRecordSlot::Secondary =>
                Self::Secondary,
        }
    }
}

/// Full year-end snapshot.
///
/// Compares directly (`==`) against a similarly-shaped JSON
/// produced by the GDI analyser. For a differential, the
/// caller decomposes the equality into per-layer comparisons
/// and reports the first divergence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct YearEndSnapshot {
    /// The year the rollover was run for (i.e. the season that
    /// just ended). Used only for cross-run labelling.
    pub year: u16,
    /// Ordered semantic events. The first divergence in this
    /// vector is the point to trace.
    pub events: Vec<YearEndEvent>,
    /// Post-rollover club statuses (`Club+0x37`) for every
    /// moved club. Keyed by club id.
    pub post_rollover_club_status:
        std::collections::BTreeMap<u32, u8>,
    /// Post-rollover finance state per club (only clubs the
    /// finance ledger actually holds after the rollover). Keyed
    /// by club id.
    pub finance:
        std::collections::BTreeMap<u32, FinanceSnapshotEntry>,
    /// Post-rollover person mailbox contents. Keyed by
    /// person_id. Values are ordered by insertion index within
    /// the mailbox.
    pub person_mailboxes:
        std::collections::BTreeMap<u32, Vec<PersonMailboxEntry>>,
    /// Post-rollover stadium state per stadium (only stadiums
    /// this rollover touched). Keyed by stadium id.
    pub stadiums:
        std::collections::BTreeMap<u32, StadiumSnapshotEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinanceSnapshotEntry {
    pub cash: i64,
    pub season_misc_expense: i32,
    pub lifetime_misc_expense: i32,
    pub season_subsidy_income: i32,
    pub lifetime_subsidy_income: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonMailboxEntry {
    pub category: u32,
    pub kind: u8,
    pub old_comp_id: u32,
    pub staff_id: u32,
    pub year: u16,
    pub news_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StadiumSnapshotEntry {
    pub capacity_total: u32,
    pub capacity_seated: u32,
    pub capacity_expansion: u32,
    pub owner_refuse_counter: i8,
}

impl YearEndSnapshot {
    /// Build a snapshot from a completed rollover: the
    /// `WorldApplyReport` (ordered event trace) plus the post-
    /// rollover `World` state and finance ledger.
    ///
    /// This is the ONLY sanctioned way to build a snapshot on
    /// the Rust side — it drives from the same real production
    /// output the exe would produce, not from ad-hoc test data.
    /// Finance is passed as a plain `club_id → ClubFinanceState` map
    /// so the snapshot is decoupled from whichever store produced
    /// it: production builds it from the ONE runtime store via
    /// `save.finance.year_end_states(ids)` (ids = clubs the
    /// rollover wrote, matching what the GDI analyser emits);
    /// tests can hand it a `ClubFinanceLedger::per_club`.
    pub fn from_apply(
        year: u16,
        world: &crate::World,
        finance_states: &std::collections::BTreeMap<u32, crate::c15_1_world_apply::ClubFinanceState>,
        applier: &crate::c15_1_world_apply::WorldApplyReport,
    ) -> Self {
        use crate::c15_1_world_apply::ContractWriteKind;
        let mut events: Vec<YearEndEvent> = Vec::new();

        // -------- Ordered event stream --------
        // 1. Club-move materialisations (via post-rollover
        //    status snapshot; we don't have pre-state so those
        //    fields are elided — the differential compares the
        //    post_rollover_club_status map instead).
        for (club_id, new_status) in &applier.post_rollover_club_status {
            events.push(YearEndEvent::StatusStamped {
                club_id: *club_id,
                old_status: 0,
                new_status: *new_status,
            });
        }
        // 2. Contract clause writes (C15.1B).
        for w in &applier.applied_contract_writes {
            events.push(YearEndEvent::ContractClauseWrite {
                person_id: w.person_id,
                offset: w.offset,
                old_value: w.old_value,
                new_value: w.new_value,
            });
            let _ = ContractWriteKind::RelegationClauseTripped; // silence
        }
        // 3. Squad position writes (C15.1C).
        for w in &applier.applied_squad_preference_writes {
            events.push(YearEndEvent::SquadPositionWrite {
                person_id: w.person_id,
                record_slot: w.record_slot.into(),
                old_value: w.old_value,
                new_value: w.new_value,
            });
        }
        // 4. Stadium capacity + refuse writes (C15.1A/D).
        for w in &applier.stadium_writes {
            events.push(YearEndEvent::StadiumCapacityWrite {
                stadium_id: w.stadium_id,
                new_total: w.new_total,
                new_seated: w.new_seated,
                new_peak: w.new_peak,
            });
        }
        for w in &applier.applied_refuse_counter_writes {
            events.push(YearEndEvent::StadiumRefuseBump {
                stadium_id: w.stadium_id,
                old_value: w.old_value,
                new_value: w.new_value,
            });
        }
        // 5. Finance writes (C15.1A/F pending log).
        for w in &applier.pending_finance {
            events.push(YearEndEvent::FinanceWrite {
                club_id: w.club_id,
                new_cash: w.new_cash,
                new_season_misc_expense: w.new_season_misc_expense,
                new_lifetime_misc_expense: w.new_lifetime_misc_expense,
                new_season_subsidy_income: w.new_season_subsidy_income,
                new_lifetime_subsidy_income: w.new_lifetime_subsidy_income,
            });
        }
        // 6. Person history appends (C15.1E).
        for w in &applier.applied_person_history_writes {
            events.push(YearEndEvent::PersonHistoryAppend {
                person_id: w.person_id,
                category: w.category,
                kind: w.kind,
                old_comp_id: w.old_comp_id,
                staff_id: w.staff_id,
                news_id: w.news_id,
            });
        }
        // 7. News events.
        for n in &applier.news_writes {
            events.push(YearEndEvent::NewsEmitted {
                template_id: n.template_id,
                club_id: n.club_id,
                competition_id: n.competition_id,
                kind: n.kind.clone(),
            });
        }

        // -------- Post-rollover state maps --------
        let finance = finance_states.iter()
            .map(|(&id, s)| (id, FinanceSnapshotEntry {
                cash: s.cash,
                season_misc_expense: s.season_misc_expense,
                lifetime_misc_expense: s.lifetime_misc_expense,
                season_subsidy_income: s.season_subsidy_income,
                lifetime_subsidy_income: s.lifetime_subsidy_income,
            }))
            .collect();

        let person_mailboxes = world.person_news_mailboxes
            .by_person.iter()
            .map(|(&pid, items)| (pid, items.iter().map(|it|
                PersonMailboxEntry {
                    category: it.category,
                    kind: it.kind(),
                    old_comp_id: it.old_comp_id(),
                    staff_id: it.staff_id(),
                    year: it.year,
                    news_id: it.news_id,
                }
            ).collect::<Vec<_>>()))
            .collect();

        // Only capture stadiums the rollover touched (by id).
        let touched: std::collections::BTreeSet<u32> =
            applier.stadium_writes.iter().map(|w| w.stadium_id)
                .chain(applier.applied_refuse_counter_writes.iter()
                    .map(|w| w.stadium_id))
                .collect();
        let stadiums = world.references.stadiums.iter()
            .filter(|s| touched.contains(&s.id))
            .map(|s| (s.id, StadiumSnapshotEntry {
                capacity_total: s.capacity_total,
                capacity_seated: s.capacity_seated,
                capacity_expansion: s.capacity_expansion,
                owner_refuse_counter: s.owner_refuse_counter,
            }))
            .collect();

        Self {
            year,
            events,
            post_rollover_club_status:
                applier.post_rollover_club_status.clone(),
            finance,
            person_mailboxes,
            stadiums,
        }
    }
}

/// Per-layer differential result.
///
/// `layer` is one of "call_order", "club_movement",
/// "contract", "squad_position", "history", "stadium",
/// "finance", "news", "rng".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifferentialLayerResult {
    pub layer: String,
    pub mismatches: usize,
    /// First divergence — human-readable summary.
    pub first_divergence: Option<String>,
}

/// Diff two snapshots. Returns a per-layer breakdown; the
/// caller decides whether to gate the freeze on
/// `mismatches == 0` per layer.
pub fn diff_snapshots(
    gdi: &YearEndSnapshot, rust: &YearEndSnapshot,
) -> Vec<DifferentialLayerResult> {
    let mut out = Vec::new();

    // A — call order (event kind sequence).
    let gdi_kinds: Vec<&str> = gdi.events.iter().map(event_kind).collect();
    let rust_kinds: Vec<&str> = rust.events.iter().map(event_kind).collect();
    out.push(compare_seq("call_order", &gdi_kinds, &rust_kinds));

    // B — club movement (post_rollover_club_status).
    out.push(compare_map(
        "club_movement",
        &gdi.post_rollover_club_status,
        &rust.post_rollover_club_status,
    ));

    // C — contract writes.
    out.push(compare_events_of("contract",
        &gdi.events, &rust.events,
        |e| matches!(e, YearEndEvent::ContractClauseWrite { .. })));

    // D — squad position writes.
    out.push(compare_events_of("squad_position",
        &gdi.events, &rust.events,
        |e| matches!(e, YearEndEvent::SquadPositionWrite { .. })));

    // E — history/news mailbox contents (compare
    //     PersonMailboxEntry vectors keyed by person_id).
    out.push(compare_mailboxes(
        "history", &gdi.person_mailboxes, &rust.person_mailboxes));

    // F — stadium (post-rollover stadium map).
    out.push(compare_map("stadium",
        &gdi.stadiums, &rust.stadiums));

    // G — finance (post-rollover finance map).
    out.push(compare_map("finance",
        &gdi.finance, &rust.finance));

    // H — news template semantics (ordered).
    out.push(compare_events_of("news",
        &gdi.events, &rust.events,
        |e| matches!(e, YearEndEvent::NewsEmitted { .. })));

    out
}

fn event_kind(e: &YearEndEvent) -> &'static str {
    match e {
        YearEndEvent::StatusStamped { .. } => "StatusStamped",
        YearEndEvent::ClubMoved { .. } => "ClubMoved",
        YearEndEvent::ContractClauseWrite { .. } => "ContractClauseWrite",
        YearEndEvent::SquadPositionWrite { .. } => "SquadPositionWrite",
        YearEndEvent::PersonHistoryAppend { .. } => "PersonHistoryAppend",
        YearEndEvent::StadiumCapacityWrite { .. } => "StadiumCapacityWrite",
        YearEndEvent::StadiumRefuseBump { .. } => "StadiumRefuseBump",
        YearEndEvent::FinanceWrite { .. } => "FinanceWrite",
        YearEndEvent::NewsEmitted { .. } => "NewsEmitted",
    }
}

fn compare_seq<T: PartialEq + std::fmt::Debug>(
    layer: &str, a: &[T], b: &[T],
) -> DifferentialLayerResult {
    let mut mismatches = 0;
    let mut first: Option<String> = None;
    let n = a.len().max(b.len());
    for i in 0..n {
        match (a.get(i), b.get(i)) {
            (Some(x), Some(y)) if x != y => {
                mismatches += 1;
                if first.is_none() {
                    first = Some(format!(
                        "index {i}: gdi={:?}, rust={:?}", x, y,
                    ));
                }
            }
            (Some(x), None) => {
                mismatches += 1;
                if first.is_none() {
                    first = Some(format!(
                        "index {i}: gdi={:?}, rust=<end>", x,
                    ));
                }
            }
            (None, Some(y)) => {
                mismatches += 1;
                if first.is_none() {
                    first = Some(format!(
                        "index {i}: gdi=<end>, rust={:?}", y,
                    ));
                }
            }
            _ => {}
        }
    }
    DifferentialLayerResult {
        layer: layer.to_string(),
        mismatches,
        first_divergence: first,
    }
}

fn compare_map<K, V>(
    layer: &str,
    a: &std::collections::BTreeMap<K, V>,
    b: &std::collections::BTreeMap<K, V>,
) -> DifferentialLayerResult
where
    K: Ord + std::fmt::Debug,
    V: PartialEq + std::fmt::Debug,
{
    let mut mismatches = 0;
    let mut first: Option<String> = None;
    for (k, va) in a.iter() {
        match b.get(k) {
            None => {
                mismatches += 1;
                if first.is_none() {
                    first = Some(format!(
                        "key {:?}: gdi={:?}, rust=<missing>", k, va,
                    ));
                }
            }
            Some(vb) if vb != va => {
                mismatches += 1;
                if first.is_none() {
                    first = Some(format!(
                        "key {:?}: gdi={:?}, rust={:?}", k, va, vb,
                    ));
                }
            }
            _ => {}
        }
    }
    for (k, vb) in b.iter() {
        if !a.contains_key(k) {
            mismatches += 1;
            if first.is_none() {
                first = Some(format!(
                    "key {:?}: gdi=<missing>, rust={:?}", k, vb,
                ));
            }
        }
    }
    DifferentialLayerResult {
        layer: layer.to_string(),
        mismatches,
        first_divergence: first,
    }
}

fn compare_events_of<F: Fn(&YearEndEvent) -> bool>(
    layer: &str, a: &[YearEndEvent], b: &[YearEndEvent], filter: F,
) -> DifferentialLayerResult {
    let av: Vec<&YearEndEvent> = a.iter().filter(|e| filter(e)).collect();
    let bv: Vec<&YearEndEvent> = b.iter().filter(|e| filter(e)).collect();
    compare_seq(layer, &av, &bv)
}

fn compare_mailboxes(
    layer: &str,
    a: &std::collections::BTreeMap<u32, Vec<PersonMailboxEntry>>,
    b: &std::collections::BTreeMap<u32, Vec<PersonMailboxEntry>>,
) -> DifferentialLayerResult {
    // We compare by (person_id -> ordered kind/old_comp/staff_id
    // sequence). news_id is expected to differ across builds
    // (the exe assigns pool-monotonic ids that include
    // pre-year-end mailbox contents Rust doesn't model);
    // ordering + payload identity is what freezes the semantics.
    let project = |v: &Vec<PersonMailboxEntry>|
        v.iter().map(|e| (e.category, e.kind, e.old_comp_id, e.staff_id))
            .collect::<Vec<_>>();
    let pa: std::collections::BTreeMap<u32, Vec<_>> =
        a.iter().map(|(&k, v)| (k, project(v))).collect();
    let pb: std::collections::BTreeMap<u32, Vec<_>> =
        b.iter().map(|(&k, v)| (k, project(v))).collect();
    compare_map(layer, &pa, &pb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c15_1g_snapshot_empty_diffs_zero() {
        let s = YearEndSnapshot::default();
        let d = diff_snapshots(&s, &s);
        for r in d {
            assert_eq!(r.mismatches, 0, "layer {} was nonzero", r.layer);
        }
    }

    #[test]
    fn c15_1g_snapshot_serde_round_trip() {
        let mut s = YearEndSnapshot::default();
        s.year = 2002;
        s.events.push(YearEndEvent::ContractClauseWrite {
            person_id: 555, offset: 0x1F,
            old_value: 1, new_value: 2,
        });
        s.finance.insert(100, FinanceSnapshotEntry {
            cash: 30_000_000, season_misc_expense: 0,
            lifetime_misc_expense: 0,
            season_subsidy_income: 0,
            lifetime_subsidy_income: 0,
        });
        s.person_mailboxes.insert(555, vec![PersonMailboxEntry {
            category: 0x0FBF, kind: 3, old_comp_id: 7,
            staff_id: 12345, year: 2002, news_id: 0,
        }]);
        let json = serde_json::to_string(&s).unwrap();
        let restored: YearEndSnapshot =
            serde_json::from_str(&json).unwrap();
        assert_eq!(s, restored);
    }

    #[test]
    fn c15_1g_diff_flags_call_order_divergence() {
        let mut a = YearEndSnapshot::default();
        let mut b = YearEndSnapshot::default();
        a.events.push(YearEndEvent::ContractClauseWrite {
            person_id: 1, offset: 0x1F, old_value: 1, new_value: 2,
        });
        b.events.push(YearEndEvent::SquadPositionWrite {
            person_id: 1,
            record_slot: SquadRecordSlotWire::Primary,
            old_value: 0, new_value: 0,
        });
        let d = diff_snapshots(&a, &b);
        let call = d.iter().find(|r| r.layer == "call_order").unwrap();
        assert_eq!(call.mismatches, 1);
        assert!(call.first_divergence.is_some());
    }

    // ---------- End-to-end goldens ------------------------------------

    fn today() -> crate::GameDate {
        crate::GameDate { year: 2002, month: 6, day: 30 }
    }

    fn empty_report_with_events(
        events: Vec<crate::c15_english_annual_rollover::YearEndMutationEvent>,
    ) -> crate::c15_english_annual_rollover::AnnualRolloverReport {
        crate::c15_english_annual_rollover::AnnualRolloverReport {
            events,
            pyramid_decision: None,
            conference_dispatch: None,
            club_moves: Default::default(),
        }
    }

    fn seed_world_with_one_finance_club(club_id: u32, seed: i32) -> crate::World {
        // Build a minimally-valid World with one club whose
        // Club+0x00 = club_id and Club+0x65 = seed. Every
        // required container is initialised empty; the ledger
        // only needs `core.clubs` and the mailbox test only
        // needs `contracts` (added by the caller).
        let mut raw = vec![0u8; 0x69];
        raw[0x00..0x04].copy_from_slice(&(club_id as i32).to_le_bytes());
        raw[0x65..0x69].copy_from_slice(&seed.to_le_bytes());
        crate::World {
            base_data: Vec::new(),
            save: None,
            schema: Default::default(),
            core: crate::CoreBook {
                clubs: vec![crate::DomainOpaqueRecord {
                    ordinal: 0, id: club_id,
                    primary_name: None, secondary_name: None,
                    short_name: None, text_candidates: Vec::new(),
                    raw,
                }],
                nat_clubs: Vec::new(),
                colours: Vec::new(),
                continents: Vec::new(),
                nations: Vec::new(),
            },
            core_summary: crate::CoreSummary {
                club_count: 1, nat_club_count: 0, colour_count: 0,
                continent_count: 0, nation_count: 0,
                sample_club_record_size: None,
                sample_nat_club_record_size: None,
                sample_colour_record_size: None,
                sample_continent_record_size: None,
                sample_nation_record_size: None,
            },
            references: crate::ReferenceBook {
                cities: Vec::new(),
                officials: Vec::new(),
                first_names: Vec::new(),
                second_names: Vec::new(),
                common_names: Vec::new(),
                stadiums: Vec::new(),
                staff_competitions: Vec::new(),
                club_competitions: Vec::new(),
                nation_competitions: Vec::new(),
                staff_history: Vec::new(),
                staff_comp_history: Vec::new(),
                club_comp_history: Vec::new(),
                nation_comp_history: Vec::new(),
            },
            reference_summary: Default::default(),
            staff: Default::default(),
            staff_summary: Default::default(),
            contracts: None,
            squad_numbers: std::collections::BTreeMap::new(),
            person_news_mailboxes:
                crate::person_news::PersonNewsMailboxPool::default(),
        }
    }

    /// C15.1G production-path golden: build a rollover report
    /// with one Relegation event, apply through
    /// `apply_report_to_world_parts`, and confirm the
    /// resulting `YearEndSnapshot` reflects both the C15.1B
    /// contract byte write and the C15.1E mailbox append —
    /// exercising the full pipeline the exe's
    /// `apply_annual_rollover → apply_report_to_world` runs.
    #[test]
    fn c15_1g_production_end_to_end_relegation() {
        use crate::c15_english_annual_rollover::*;
        use crate::c13_promotion_apply::*;
        use crate::contract_init::{ContractPool, ContractRecord};

        // Seed a world with one club (id 100) and a contract
        // pool holding a person 555 (staff_id 999) at club 100
        // with an armed relegation clause.
        let mut world = seed_world_with_one_finance_club(100, 30_000_000);
        let mut pool = ContractPool::default();
        pool.records.push(ContractRecord {
            staff_id: 999, club_id: 100, wage: 0, value: 0,
            non_promotion: 0, minimum_fee: 0, non_playing: 0,
            relegation: 1, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
            position_code: 0,
        });
        pool.by_staff_id = vec![-1; 556];
        pool.by_staff_id[555] = 0;
        world.contracts = Some(pool);
        // Seed the finance ledger from the club.
        let mut ledger = crate::c15_1_world_apply::ClubFinanceLedger::new();
        ledger.seed_from_world(&world);
        let date = today();

        // Build a Relegation event with the person.
        let report = empty_report_with_events(vec![
            YearEndMutationEvent::Relegation {
                effects: RelegationApplyEffects {
                    club_id: 100,
                    writes: ClubFieldWrites {
                        new_comp_id: 8, prev_comp_id: 7,
                        tier_byte_64: None,
                    },
                    set_status_idle: true,
                    person_effects: vec![PersonEffect {
                        person_id: 555,
                        new_staff_1f: Some(2),
                        new_staff_1c: None,
                        event_emit: Some(PersonHistoryEvent {
                            old_comp_id: 7, kind: 3,
                        }),
                    }],
                    no_league_news: Some(RelegationNoLeagueNews {
                        club_id: 100,
                    }),
                },
            },
        ]);

        // Production entry — same code path `apply_report_to_world`
        // uses; the test uses `_parts` so we don't need a full
        // save's RuntimeEvent queue.
        let mut pending: Vec<crate::RuntimeEvent> = vec![];
        let applier = crate::c15_1_world_apply::apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        for w in &applier.pending_finance {
            ledger.apply_write(w);
        }

        // Build the snapshot.
        let snap = YearEndSnapshot::from_apply(
            date.year, &world, &ledger.per_club, &applier,
        );

        // C15.1B assertion: contract clause tripped.
        assert!(snap.events.iter().any(|e| matches!(e,
            YearEndEvent::ContractClauseWrite {
                person_id: 555, offset: 0x1F,
                old_value: 1, new_value: 2,
            })
        ), "expected the C15.1B contract 1→2 event");

        // C15.1E assertion: mailbox append landed.
        let mb = snap.person_mailboxes.get(&555).unwrap();
        assert_eq!(mb.len(), 1);
        assert_eq!(mb[0].kind, 3);
        assert_eq!(mb[0].old_comp_id, 7);
        assert_eq!(mb[0].staff_id, 999);
        assert!(snap.events.iter().any(|e| matches!(e,
            YearEndEvent::PersonHistoryAppend { person_id: 555, kind: 3, .. })
        ), "expected C15.1E PersonHistoryAppend event");

        // C15.1F assertion: finance ledger carries the seeded cash.
        let fin = snap.finance.get(&100).unwrap();
        assert_eq!(fin.cash, 30_000_000i64);

        // News assertion: the no_league_news template landed.
        assert!(snap.events.iter().any(|e| matches!(e,
            YearEndEvent::NewsEmitted { .. })
        ), "expected at least one news event");
    }

    /// C15.1G save/load round-trip: after applying a rollover,
    /// serialise `RuntimeSaveGame` + `World`, deserialise, and
    /// assert the C15.1E mailbox pool and C15.1F finance ledger
    /// survive verbatim.
    #[test]
    fn c15_1g_post_rollover_save_load_round_trip() {
        use crate::c15_english_annual_rollover::*;
        use crate::c13_promotion_apply::*;
        use crate::contract_init::{ContractPool, ContractRecord};

        let mut world = seed_world_with_one_finance_club(100, 25_000_000);
        let mut pool = ContractPool::default();
        pool.records.push(ContractRecord {
            staff_id: 999, club_id: 100, wage: 0, value: 0,
            non_promotion: 0, minimum_fee: 0, non_playing: 0,
            relegation: 1, manager_job: 0,
            expiry_dayofyear: 0, expiry_year: 2005,
            position_code: 0,
        });
        pool.by_staff_id = vec![-1; 556];
        pool.by_staff_id[555] = 0;
        world.contracts = Some(pool);
        let mut ledger = crate::c15_1_world_apply::ClubFinanceLedger::new();
        ledger.seed_from_world(&world);
        let date = today();

        let report = empty_report_with_events(vec![
            YearEndMutationEvent::Relegation {
                effects: RelegationApplyEffects {
                    club_id: 100,
                    writes: ClubFieldWrites {
                        new_comp_id: 8, prev_comp_id: 7,
                        tier_byte_64: None,
                    },
                    set_status_idle: true,
                    person_effects: vec![PersonEffect {
                        person_id: 555,
                        new_staff_1f: Some(2),
                        new_staff_1c: None,
                        event_emit: Some(PersonHistoryEvent {
                            old_comp_id: 7, kind: 3,
                        }),
                    }],
                    no_league_news: None,
                },
            },
        ]);
        let mut pending: Vec<crate::RuntimeEvent> = vec![];
        let applier = crate::c15_1_world_apply::apply_report_to_world_parts(
            &mut world, &mut pending, &date, 0, &report,
        );
        for w in &applier.pending_finance {
            ledger.apply_write(w);
        }

        // Round-trip World.
        let world_json = serde_json::to_string(&world).unwrap();
        let world_back: crate::World =
            serde_json::from_str(&world_json).unwrap();
        // Mailbox contents survive.
        let before = world.person_news_mailboxes.mailbox_for(555);
        let after = world_back.person_news_mailboxes.mailbox_for(555);
        assert_eq!(before, after);
        // ContractPool survives.
        assert_eq!(
            world.contracts.as_ref().unwrap().records[0].relegation,
            world_back.contracts.as_ref().unwrap().records[0].relegation,
        );

        // Round-trip the ledger. (Serialise the ledger alone —
        // the full RuntimeSaveGame round-trip is a much broader
        // test; C15.1F already covers the ledger's own serde,
        // and this test proves the C15.1E mailbox survives too.)
        let ledger_json = serde_json::to_string(&ledger).unwrap();
        let ledger_back: crate::c15_1_world_apply::ClubFinanceLedger =
            serde_json::from_str(&ledger_json).unwrap();
        assert_eq!(ledger, ledger_back);
    }

    /// C15.1G snapshot equality: a rollover applied once and
    /// then again on a fresh copy must produce identical
    /// snapshots. Pins determinism of the applier at the
    /// snapshot layer.
    #[test]
    fn c15_1g_snapshot_is_deterministic() {
        use crate::c15_english_annual_rollover::*;
        use crate::c13_promotion_apply::*;
        use crate::contract_init::{ContractPool, ContractRecord};

        fn build_and_snapshot() -> YearEndSnapshot {
            let mut world = seed_world_with_one_finance_club(100, 30_000_000);
            let mut pool = ContractPool::default();
            pool.records.push(ContractRecord {
                staff_id: 999, club_id: 100, wage: 0, value: 0,
                non_promotion: 0, minimum_fee: 0, non_playing: 0,
                relegation: 1, manager_job: 0,
                expiry_dayofyear: 0, expiry_year: 2005,
                position_code: 0,
            });
            pool.by_staff_id = vec![-1; 556];
            pool.by_staff_id[555] = 0;
            world.contracts = Some(pool);
            let mut ledger =
                crate::c15_1_world_apply::ClubFinanceLedger::new();
            ledger.seed_from_world(&world);
            let date = today();
            let report = empty_report_with_events(vec![
                YearEndMutationEvent::Relegation {
                    effects: RelegationApplyEffects {
                        club_id: 100,
                        writes: ClubFieldWrites {
                            new_comp_id: 8, prev_comp_id: 7,
                            tier_byte_64: None,
                        },
                        set_status_idle: true,
                        person_effects: vec![PersonEffect {
                            person_id: 555,
                            new_staff_1f: Some(2),
                            new_staff_1c: None,
                            event_emit: Some(PersonHistoryEvent {
                                old_comp_id: 7, kind: 3,
                            }),
                        }],
                        no_league_news: None,
                    },
                },
            ]);
            let mut pending: Vec<crate::RuntimeEvent> = vec![];
            let applier =
                crate::c15_1_world_apply::apply_report_to_world_parts(
                    &mut world, &mut pending, &date, 0, &report,
                );
            for w in &applier.pending_finance {
                ledger.apply_write(w);
            }
            YearEndSnapshot::from_apply(
                date.year, &world, &ledger.per_club, &applier,
            )
        }
        let a = build_and_snapshot();
        let b = build_and_snapshot();
        assert_eq!(a, b);
        // And a self-diff is clean at every layer.
        for r in diff_snapshots(&a, &b) {
            assert_eq!(r.mismatches, 0, "layer {} nonzero", r.layer);
        }
    }

    #[test]
    fn c15_1g_diff_flags_finance_divergence_by_club() {
        let mut a = YearEndSnapshot::default();
        let mut b = YearEndSnapshot::default();
        a.finance.insert(100, FinanceSnapshotEntry {
            cash: 1_000, season_misc_expense: 0,
            lifetime_misc_expense: 0,
            season_subsidy_income: 0,
            lifetime_subsidy_income: 0,
        });
        b.finance.insert(100, FinanceSnapshotEntry {
            cash: 2_000, season_misc_expense: 0,
            lifetime_misc_expense: 0,
            season_subsidy_income: 0,
            lifetime_subsidy_income: 0,
        });
        let d = diff_snapshots(&a, &b);
        let fin = d.iter().find(|r| r.layer == "finance").unwrap();
        assert_eq!(fin.mismatches, 1);
    }
}
