//! C15.1G.1 — Traditional English year-end differential CLI.
//!
//! Runs the REAL production year-end path (`compute_annual_rollover`
//! → `apply_report_to_world_parts` → `YearEndSnapshot::from_apply`)
//! against a GDI-side capture snapshot and prints a layer-by-layer
//! verdict. Exit codes:
//!
//! * `0` — every captured semantic layer clean.
//! * `1` — semantic mismatch on one or more captured layers.
//! * `2` — invalid / missing input or setup error.
//!
//! # Usage
//!
//! ```bash
//! # Default: run against the built-in canonical Traditional
//! # English 2001-02 demo pre-rollover state. Useful for
//! # tooling smoke-checks; NOT the final freeze differential.
//! cargo run -p cm-import --bin c15_year_end_diff -- \
//!     --gdi fixtures/gdi_captures/year_end_2001_02.gdi.json
//!
//! # With a user-supplied Rust pre-rollover config (JSON matching
//! # `AnnualRolloverInputConfig` — an owned/serialisable image of
//! # `AnnualRolloverInput`). This is the sanctioned final-freeze
//! # invocation once a real capture is in hand.
//! cargo run -p cm-import --bin c15_year_end_diff -- \
//!     --gdi fixtures/gdi_captures/year_end_2001_02.gdi.json \
//!     --rust-input fixtures/gdi_captures/year_end_2001_02.rust_input.json
//!
//! # JSON output for CI:
//! cargo run -p cm-import --bin c15_year_end_diff -- \
//!     --gdi <path> --json-output verdict.json
//! ```
//!
//! Load-bearing invariant: the CLI's `--demo` fallback is
//! plainly labelled in the output so `CORE FREEZE: PASS` is
//! never claimed on synthetic Rust input. A green demo run
//! proves the tooling; the final freeze needs a real Rust
//! pre-rollover config built to mirror the GDI capture.

use cm_domain::c13_promotion_apply::{
    ClubFieldWrites, PersonEffect, PersonHistoryEvent,
    RelegationApplyEffects,
};
use cm_domain::c15_1_world_apply::{
    apply_report_to_world_parts, ClubFinanceLedger,
};
use cm_domain::c15_1g_snapshot::{
    diff_snapshots, DifferentialLayerResult, YearEndSnapshot,
};
use cm_domain::c15_english_annual_rollover::{
    AnnualRolloverReport, YearEndMutationEvent,
};
use cm_domain::contract_init::{ContractPool, ContractRecord};
use cm_domain::person_news::PersonNewsMailboxPool;
use cm_domain::{
    CoreBook, CoreSummary, DomainOpaqueRecord, GameDate, ReferenceBook,
    RuntimeEvent, World,
};
use std::path::PathBuf;
use std::process::ExitCode;

/// Layer verdict for CLI printing + JSON output.
#[derive(serde::Serialize)]
struct LayerVerdict {
    layer: String,
    status: String,      // "PASS" | "FAIL" | "NOT_CAPTURED"
    mismatches: usize,
    first_divergence: Option<String>,
}

#[derive(serde::Serialize)]
struct FreezeVerdict {
    status: String,      // "PASS" | "PASS_FOR_OBSERVED" | "FAIL"
    first_failing_layer: Option<String>,
    unobserved_branches: Vec<String>,
}

#[derive(serde::Serialize)]
struct RunReport {
    call_order: LayerVerdict,
    club_movement: LayerVerdict,
    contracts: LayerVerdict,
    squad: LayerVerdict,
    history: LayerVerdict,
    stadium: LayerVerdict,
    finance: LayerVerdict,
    news: LayerVerdict,
    rng: LayerVerdict,
    freeze: FreezeVerdict,
    rust_source: String,   // "demo" | "user_input"
    representation_deviations: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut gdi: Option<PathBuf> = None;
    let mut rust_input: Option<PathBuf> = None;
    let mut json_output: Option<PathBuf> = None;
    let mut verbose = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--gdi" => gdi = it.next().map(PathBuf::from),
            "--rust-input" => rust_input = it.next().map(PathBuf::from),
            "--json-output" => json_output = it.next().map(PathBuf::from),
            "--verbose" | "-v" => verbose = true,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {}", other)),
        }
    }
    let gdi = gdi.ok_or_else(||
        "--gdi <path> is required".to_string())?;
    Ok(Args { gdi, rust_input, json_output, verbose })
}

fn print_help() {
    eprintln!("c15_year_end_diff — Traditional English year-end runtime differential");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    c15_year_end_diff --gdi <path> [--rust-input <path>] [--json-output <path>] [--verbose]");
    eprintln!();
    eprintln!("EXIT CODES:");
    eprintln!("    0    every captured semantic layer clean");
    eprintln!("    1    semantic mismatch");
    eprintln!("    2    invalid / missing input");
}

struct Args {
    gdi: PathBuf,
    rust_input: Option<PathBuf>,
    json_output: Option<PathBuf>,
    verbose: bool,
}

fn load_gdi_snapshot(path: &std::path::Path) -> Result<YearEndSnapshot, String> {
    let s = std::fs::read_to_string(path)
        .map_err(|e| format!("read {}: {}", path.display(), e))?;
    serde_json::from_str(&s)
        .map_err(|e| format!("parse {} as YearEndSnapshot: {}",
                             path.display(), e))
}

/// Canonical Traditional English 2001-02 demo pre-rollover
/// state. Exercises the full production pipeline
/// (`apply_report_to_world_parts`) so the CLI can smoke-check
/// itself in the absence of a real user-supplied config. The
/// resulting `rust_source` field in the JSON output is `"demo"`;
/// the freeze verdict downgrades from `PASS` to `PASS (demo)` so
/// no false claims of state-exactness are emitted.
///
/// Real freeze differentials MUST supply `--rust-input <path>`
/// with a config mirroring the GDI capture's pre-state.
fn run_demo_rollover() -> Result<YearEndSnapshot, String> {
    // One-club Traditional English demo: club 100 in comp 7, a
    // person with an armed relegation clause. Small enough to
    // read; large enough to fire C15.1B/E/F.
    let mut world = build_minimal_world(100, 30_000_000);
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
    let mut ledger = ClubFinanceLedger::new();
    ledger.seed_from_world(&world);

    let report = AnnualRolloverReport {
        events: vec![YearEndMutationEvent::Relegation {
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
        }],
        pyramid_decision: None,
        conference_dispatch: None,
        club_moves: Default::default(),
    };

    let date = GameDate { year: 2002, month: 6, day: 30 };
    let mut pending: Vec<RuntimeEvent> = vec![];
    let applier = apply_report_to_world_parts(
        &mut world, &mut pending, &date, 0, &report,
    );
    for w in &applier.pending_finance {
        ledger.apply_write(w);
    }
    Ok(YearEndSnapshot::from_apply(
        date.year, &world, &ledger.per_club, &applier,
    ))
}

fn build_minimal_world(club_id: u32, cash_seed: i32) -> World {
    let mut raw = vec![0u8; 0x69];
    raw[0x00..0x04].copy_from_slice(&(club_id as i32).to_le_bytes());
    raw[0x65..0x69].copy_from_slice(&cash_seed.to_le_bytes());
    World {
        base_data: Vec::new(),
        save: None,
        schema: Default::default(),
        core: CoreBook {
            clubs: vec![DomainOpaqueRecord {
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
        core_summary: CoreSummary {
            club_count: 1, nat_club_count: 0, colour_count: 0,
            continent_count: 0, nation_count: 0,
            sample_club_record_size: None,
            sample_nat_club_record_size: None,
            sample_colour_record_size: None,
            sample_continent_record_size: None,
            sample_nation_record_size: None,
        },
        references: ReferenceBook {
            cities: Vec::new(), officials: Vec::new(),
            first_names: Vec::new(), second_names: Vec::new(),
            common_names: Vec::new(), stadiums: Vec::new(),
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
        person_news_mailboxes: PersonNewsMailboxPool::default(),
    }
}

/// Validate a GDI snapshot before running the differential.
/// Rejects obviously off-target captures (e.g. the year fields
/// disagree with an included Rust user input).
fn validate_gdi_snapshot(s: &YearEndSnapshot) -> Result<(), String> {
    // Non-zero-length events isn't a hard requirement (empty
    // capture may be valid on quiet years), but a zero-year
    // capture is almost always malformed.
    if s.year == 0 {
        return Err(
            "GDI snapshot has year=0 — likely a malformed capture. \
             Check that the analyser saw a `season` field."
                .to_string()
        );
    }
    Ok(())
}

fn to_verdict(layer: &str, r: &DifferentialLayerResult) -> LayerVerdict {
    let status = if r.mismatches == 0 { "PASS" } else { "FAIL" };
    LayerVerdict {
        layer: layer.to_string(),
        status: status.to_string(),
        mismatches: r.mismatches,
        first_divergence: r.first_divergence.clone(),
    }
}

fn print_layer(v: &LayerVerdict, label: &str, verbose: bool) {
    let dots: String = ".".repeat(30usize.saturating_sub(label.len()));
    println!("  {} {} {}", label, dots, v.status);
    if v.status == "FAIL" {
        if let Some(fd) = &v.first_divergence {
            println!("        first divergence: {}", fd);
        }
        if verbose {
            println!("        mismatch count:   {}", v.mismatches);
        }
    }
}

fn print_per_edge_movement(
    gdi: &YearEndSnapshot, rust: &YearEndSnapshot,
) {
    // Break status transitions into pyramid edges. Edge is
    // classified by (old_comp -> new_comp) inferred from
    // ClubMoved events; when the GDI capture doesn't include
    // ClubMoved (the current analyser doesn't yet), we fall
    // back to counting post-rollover statuses per club and
    // labelling edges NOT AVAILABLE.
    println!();
    println!("  Per-edge movement:");
    let has_moves = gdi.events.iter().any(|e| matches!(
        e, cm_domain::c15_1g_snapshot::YearEndEvent::ClubMoved { .. }
    ));
    if !has_moves {
        println!("        (edge breakdown NOT AVAILABLE — GDI \
                  analyser did not emit ClubMoved events)");
        return;
    }
    let edges = [
        ("Premier ↔ First",         7,  8),
        ("First ↔ Second",          8,  9),
        ("Second ↔ Third",          9, 10),
        ("Third ↔ Conference",     10, 93),
        ("Conference feeder",       93, 93),
    ];
    for (label, a, b) in edges {
        let count_edge = |snap: &YearEndSnapshot|
            snap.events.iter().filter(|e| match e {
                cm_domain::c15_1g_snapshot::YearEndEvent::ClubMoved {
                    old_primary_comp, new_primary_comp, ..
                } => {
                    (*old_primary_comp == a && *new_primary_comp == b)
                    || (*old_primary_comp == b && *new_primary_comp == a)
                }
                _ => false,
            }).count();
        let g = count_edge(gdi);
        let r = count_edge(rust);
        let mism = if g == r { 0 } else { g.max(r) - g.min(r) };
        println!("        {label:<28} gdi={g:>3} rust={r:>3} mismatch={mism}");
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => { eprintln!("error: {}", e); return ExitCode::from(2); }
    };

    let gdi_snap = match load_gdi_snapshot(&args.gdi) {
        Ok(s) => s,
        Err(e) => { eprintln!("error: {}", e); return ExitCode::from(2); }
    };
    if let Err(e) = validate_gdi_snapshot(&gdi_snap) {
        eprintln!("error: {}", e);
        return ExitCode::from(2);
    }

    let (rust_snap, rust_source) = match &args.rust_input {
        Some(_p) => {
            // Real Rust pre-rollover input path — the sanctioned
            // final-freeze invocation. Loading + running is left
            // as a follow-up because it requires a stable
            // AnnualRolloverInputConfig schema that mirrors
            // AnnualRolloverInput without the borrowed RNG. For
            // now, refuse loudly and point the user at the
            // demo path.
            eprintln!("error: --rust-input is reserved for the \
                       forthcoming AnnualRolloverInputConfig \
                       schema (C15.1G.2). Run without it to use \
                       the built-in demo pipeline for tooling \
                       verification.");
            return ExitCode::from(2);
        }
        None => {
            match run_demo_rollover() {
                Ok(s) => (s, "demo".to_string()),
                Err(e) => {
                    eprintln!("error running demo rollover: {}", e);
                    return ExitCode::from(2);
                }
            }
        }
    };

    let per_layer = diff_snapshots(&gdi_snap, &rust_snap);

    // Header.
    println!();
    println!("Traditional English Year-End Differential");
    println!("=========================================");
    println!("  GDI capture:  {}", args.gdi.display());
    println!("  Rust source:  {}", rust_source);
    println!("  GDI year:     {}", gdi_snap.year);
    println!("  Rust year:    {}", rust_snap.year);
    println!();

    // Per-layer summary.
    let find = |name: &str| per_layer.iter()
        .find(|r| r.layer == name).cloned()
        .unwrap_or(DifferentialLayerResult {
            layer: name.to_string(), mismatches: 0,
            first_divergence: None,
        });

    let call_order   = to_verdict("Call order",             &find("call_order"));
    let club_move    = to_verdict("Club movement",          &find("club_movement"));
    let contracts    = to_verdict("Contract clauses",       &find("contract"));
    let squad        = to_verdict("Squad position",         &find("squad_position"));
    let history      = to_verdict("Person history/news",    &find("history"));
    let stadium      = to_verdict("Stadium",                &find("stadium"));
    let finance      = to_verdict("Finance",                &find("finance"));
    let news         = to_verdict("News",                   &find("news"));
    // RNG layer: no capture support yet.
    let rng = LayerVerdict {
        layer: "RNG".to_string(),
        status: "NOT CAPTURED".to_string(),
        mismatches: 0,
        first_divergence: None,
    };

    print_layer(&call_order, "Call order",           args.verbose);
    print_layer(&club_move,  "Club movement",        args.verbose);
    print_layer(&contracts,  "Contract clauses",     args.verbose);
    print_layer(&squad,      "Squad position",       args.verbose);
    print_layer(&history,    "Person history/news",  args.verbose);
    print_layer(&stadium,    "Stadium",              args.verbose);
    print_layer(&finance,    "Finance",              args.verbose);
    print_layer(&news,       "News",                 args.verbose);
    println!("  {} {} {}", "RNG",
             ".".repeat(30usize.saturating_sub(3)),
             rng.status);

    print_per_edge_movement(&gdi_snap, &rust_snap);

    // Representation deviations — always printed, always
    // separate from the pass/fail verdict.
    let rep_devs = vec![
        "history mailbox: DOB-age routing not modelled (semantic parity only)".to_string(),
        "history mailbox: 100-entry ring overwrite not modelled".to_string(),
        "history record: numbered slots 1..=4 held at 0 (pointer chains not modelled)".to_string(),
        "history record: +0x04 severity byte held at 0".to_string(),
        "finance record: 24+ additional accumulator DWORDs deferred to future tranches".to_string(),
    ];
    println!();
    println!("  Known representation deviations (not compared):");
    for d in &rep_devs {
        println!("        - {}", d);
    }

    // Unobserved branches — always listed, never counted as PASS.
    let unobserved = vec![
        "Conference fallback (FUN_0055EA00 path)".to_string(),
        "Owner-refusal stadium path (FUN_00583FC0 refuse branch)".to_string(),
        "Nation-affiliation identity clause (FUN_00843970 second identity check)".to_string(),
    ];
    println!();
    println!("  Unobserved branches (state-exactness NOT claimed):");
    for u in &unobserved {
        println!("        - {}", u);
    }

    // Freeze verdict. Compute first-failing name via a scan
    // over the verdicts (owned lookup, not borrows) so we can
    // still hand the LayerVerdicts to RunReport below.
    let ordered: [(&str, &LayerVerdict); 8] = [
        ("Call order", &call_order),
        ("Club movement", &club_move),
        ("Contract clauses", &contracts),
        ("Squad position", &squad),
        ("Person history/news", &history),
        ("Stadium", &stadium),
        ("Finance", &finance),
        ("News", &news),
    ];
    let first_fail: Option<String> = ordered.iter()
        .find(|(_, v)| v.status == "FAIL")
        .map(|(n, _)| (*n).to_string());
    let any_fail = first_fail.is_some();
    println!();
    println!("=========================================");
    let freeze = if let Some(first) = first_fail.clone() {
        println!("CORE FREEZE: FAIL");
        println!("first failing layer: {}", first);
        FreezeVerdict {
            status: "FAIL".to_string(),
            first_failing_layer: Some(first),
            unobserved_branches: unobserved.clone(),
        }
    } else if rust_source == "demo" {
        println!("CORE FREEZE: PASS (demo run — synthetic Rust \
                  input, NOT a final freeze differential)");
        println!("  Run with --rust-input <config.json> from a \
                  real pre-rollover state to promote to a final \
                  freeze.");
        FreezeVerdict {
            status: "PASS_DEMO_ONLY".to_string(),
            first_failing_layer: None,
            unobserved_branches: unobserved.clone(),
        }
    } else {
        println!("CORE FREEZE: PASS FOR OBSERVED {}→{} PATH",
                 gdi_snap.year - 1, gdi_snap.year);
        println!("unobserved branches (see above) remain at \
                  STRUCTURALLY-PORTED confidence.");
        FreezeVerdict {
            status: "PASS_FOR_OBSERVED".to_string(),
            first_failing_layer: None,
            unobserved_branches: unobserved.clone(),
        }
    };
    println!("=========================================");

    // JSON output.
    let json_report = RunReport {
        call_order, club_movement: club_move, contracts, squad,
        history, stadium, finance, news, rng,
        freeze,
        rust_source,
        representation_deviations: rep_devs,
    };
    if let Some(out) = &args.json_output {
        match serde_json::to_string_pretty(&json_report) {
            Ok(s) => {
                if let Err(e) = std::fs::write(out, s) {
                    eprintln!("warning: could not write --json-output {}: {}",
                              out.display(), e);
                }
            }
            Err(e) => eprintln!("warning: json encode failed: {}", e),
        }
    }

    if any_fail { ExitCode::from(1) }
    else { ExitCode::from(0) }
}

// -----------------------------------------------------------------
// Tests
// -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cm_domain::c15_1g_snapshot::{
        FinanceSnapshotEntry, PersonMailboxEntry, YearEndEvent,
    };

    fn seeded_snapshot() -> YearEndSnapshot {
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
            staff_id: 999, year: 2002, news_id: 0,
        }]);
        s
    }

    #[test]
    fn identical_snapshots_diff_clean() {
        let a = seeded_snapshot();
        for r in diff_snapshots(&a, &a) {
            assert_eq!(r.mismatches, 0,
                       "layer {} nonzero on self-diff", r.layer);
        }
    }

    #[test]
    fn one_club_mismatch_flags_first_divergence() {
        let a = seeded_snapshot();
        let mut b = seeded_snapshot();
        // Overwrite b's finance so club 100 shows a different cash.
        b.finance.get_mut(&100).unwrap().cash = 99;
        let per_layer = diff_snapshots(&a, &b);
        let fin = per_layer.iter().find(|r| r.layer == "finance").unwrap();
        assert_eq!(fin.mismatches, 1);
        assert!(fin.first_divergence.is_some());
    }

    #[test]
    fn representation_only_difference_is_not_a_semantic_mismatch() {
        // The current diff comparator projects mailbox entries
        // down to (category, kind, old_comp, staff_id) — i.e.
        // news_id and year (representation-adjacent labels)
        // are NOT compared. Changing news_id on one side alone
        // must NOT count as a mismatch.
        let a = seeded_snapshot();
        let mut b = seeded_snapshot();
        b.person_mailboxes.get_mut(&555).unwrap()[0].news_id = 42;
        let per_layer = diff_snapshots(&a, &b);
        let hist = per_layer.iter().find(|r| r.layer == "history").unwrap();
        assert_eq!(hist.mismatches, 0,
                   "news_id divergence must not count as semantic mismatch");
    }

    #[test]
    fn demo_rollover_produces_deterministic_snapshot() {
        // The CLI's `--demo` mode drives the real
        // apply_report_to_world_parts pipeline. Two invocations
        // must produce identical snapshots.
        let a = run_demo_rollover().unwrap();
        let b = run_demo_rollover().unwrap();
        for r in diff_snapshots(&a, &b) {
            assert_eq!(r.mismatches, 0,
                       "demo layer {} nonzero on repeat run", r.layer);
        }
    }
}
