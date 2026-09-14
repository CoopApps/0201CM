//! C11.1 — production fixture golden (points 10, 11, 13-17).
//!
//! Runs the REAL production dispatch
//! (`World::generate_new_game_season_with_rng`) against the shipped
//! rust-db, with the captured GDI initial pool RNG state pinned via
//! [`GameRngState`]. Compares the produced (home_name, away_name,
//! round_within_half) tuple for every English fixture ordered
//! against the captured GDI insert trace.
//!
//! Names not numeric IDs — the shipped rust-db uses a different id
//! numbering than the running GDI build (a rust-db-vs-shipped-dat
//! renumbering that is out of C11.1 scope; the club NAME field
//! matches byte-for-byte). Order + names + round + date is what
//! the perturb + walker + driver actually determine — that is what
//! we assert byte-exact here.
//!
//! Required counts:
//!   Premier    0/380 mismatches
//!   First      0/552
//!   Second     0/552
//!   Third      0/552
//!   Conference 0/462

use cm_domain::english_traditional::ENGLISH_TRADITIONAL_COMP_IDS;
use cm_domain::game_rng::{GameRng, GameRngState};
use cm_domain::World;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const CAPTURE_PREFIX: &str =
    "D:/cm0102-rs/reports/fixture_disasm/runtime/20260914_180417_five_leagues";

fn strip_pad(s: &str) -> String {
    s.trim_end_matches(|c: char| c == '\u{fffd}' || c == '\0' || c == ' ')
        .to_string()
}

/// Load captured Prem initial pool RNG state — the boot state for
/// the shared 5-league chain. The exe's continuous stream means
/// this alone pins the rest.
fn load_prem_initial_rng_state() -> Option<GameRngState> {
    let path = format!("{CAPTURE_PREFIX}_prem_lineage.jsonl");
    let text = std::fs::read_to_string(&path).ok()?;
    let mut init: Option<serde_json::Value> = None;
    let mut phase_c_seed: Option<i32> = None;
    for line in text.lines() {
        if line.is_empty() { continue; }
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let op = v.get("op").and_then(|s| s.as_str()).unwrap_or("");
        if op == "ctor_enter" && init.is_none() {
            init = v.get("rng_initial").cloned();
        } else if op == "rng_trace" {
            for c in v.get("calls")?.as_array()? {
                if c.get("phase").and_then(|s| s.as_str()) == Some("PERTURB")
                    && c.get("kind").and_then(|s| s.as_str()) == Some("srand")
                {
                    phase_c_seed = c.get("seed").and_then(|s| s.as_i64()).map(|i| i as i32);
                    break;
                }
            }
        }
    }
    let init = init?;
    // C11.2: DAT_00dbc340 = phase_c_srand_seed - base_year (=2001).
    let dbc340 = phase_c_seed.map(|s| s.wrapping_sub(2001)).unwrap_or(0);
    Some(GameRngState {
        cursor: init.get("cursor_off")?.as_u64()? as u32,
        jitter: init.get("jitter")?.as_u64()? as u32,
        lcg_state: init.get("lcg_state")?.as_u64()? as u32,
        dbc340_cli_seed: dbc340,
    })
}

/// Load captured primary fixtures for one league as ordered
/// tuples `(home_name, away_name, round_within_half)`.
fn load_captured_fixtures(lid: &str) -> Option<Vec<(String, String, i16)>> {
    let path = format!("{CAPTURE_PREFIX}_{lid}_lineage.jsonl");
    let text = std::fs::read_to_string(&path).ok()?;
    let mut id_to_name: BTreeMap<i32, String> = BTreeMap::new();
    let mut primary = None;
    for line in text.lines() {
        if line.is_empty() { continue; }
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let op = v.get("op").and_then(|s| s.as_str()).unwrap_or("");
        if op == "clubs_snap" && v.get("point").and_then(|s| s.as_str()) == Some("P1") {
            for e in v.get("entries")?.as_array()? {
                let cid = e.get("club_id")?.as_i64()? as i32;
                let name = strip_pad(e.get("club_name")?.as_str()?);
                id_to_name.insert(cid, name);
            }
        } else if op == "insert_trace" {
            primary = Some(v.get("inserts")?.as_array()?.clone());
        }
    }
    let inserts = primary?;
    let mut out = Vec::new();
    for f in inserts {
        let site = f.get("site")?.as_str()?;
        if site != "primary" && site != "main" { continue; }
        let home = f.get("home")?.as_i64()? as i32;
        let away = f.get("away")?.as_i64()? as i32;
        let rwh = f.get("rwh")?.as_i64()? as i16;
        let hn = id_to_name.get(&home).cloned().unwrap_or_default();
        let an = id_to_name.get(&away).cloned().unwrap_or_default();
        out.push((hn, an, rwh));
    }
    Some(out)
}

fn run_production_and_diff(league: &str, comp_id: u32, expected_count: usize) {
    let rust_db = Path::new("D:/cm0102-rs/rust-db");
    if !rust_db.exists() {
        eprintln!("rust-db not present; skipping {league}");
        return;
    }
    let Some(initial) = load_prem_initial_rng_state() else {
        eprintln!("captured Prem initial RNG state missing; skipping {league}");
        return;
    };
    let Some(captured) = load_captured_fixtures(league) else {
        eprintln!("captured fixture data for {league} missing; skipping");
        return;
    };
    assert_eq!(captured.len(), expected_count,
        "{league}: capture size mismatch: got {}, want {}",
        captured.len(), expected_count);

    let world = World::read_rust_db_dir(rust_db).expect("read rust-db");
    let comp_ids: BTreeSet<u32> = ENGLISH_TRADITIONAL_COMP_IDS.iter().copied().collect();
    let dbc340 = initial.dbc340_cli_seed;
    let mut rng = GameRng::from_state_snapshot(initial);
    let (fixtures, _proofs, _standings) =
        world.generate_new_game_season_with_rng_and_dbc340(
            &comp_ids, 2001, &mut rng, dbc340,
        );

    // Rust produces all 5 leagues in one call — filter by comp_id.
    let rust: Vec<(String, String, i16)> = fixtures
        .iter()
        .filter(|f| f.competition_id == comp_id)
        .map(|f| {
            let round = infer_round_within_half(f);
            (
                strip_pad(&f.home_club_name),
                strip_pad(&f.away_club_name),
                round,
            )
        })
        .collect();
    assert_eq!(rust.len(), expected_count,
        "{league}: rust produced {} fixtures, expected {expected_count}",
        rust.len());

    let mut mismatches = 0usize;
    let mut first_mismatches: Vec<String> = Vec::new();
    for (i, (r, c)) in rust.iter().zip(captured.iter()).enumerate() {
        if r != c {
            mismatches += 1;
            if first_mismatches.len() < 8 {
                first_mismatches.push(format!(
                    "  slot {i}: rust ({}, {}, r{}) vs exe ({}, {}, r{})",
                    r.0, r.1, r.2, c.0, c.1, c.2
                ));
            }
        }
    }
    if mismatches > 0 {
        println!(
            "{league}: {}/{} mismatches (first samples below); see \
             deviations/c11_1_production_fixture_golden.md for the \
             pending archaeology.\n{}",
            mismatches, expected_count, first_mismatches.join("\n")
        );
    } else {
        println!("{league}: 0/{expected_count} ordered mismatches");
    }
    // C11.2: hard-assert 0/N vs captured GDI. The pieces required
    // — captured pool state via GameRngState, captured
    // DAT_00dbc340 via GameRngState.dbc340_cli_seed, real rust-db
    // P1 in shipped-dat order, native Rust schedule, production
    // resolver — are all wired.
    assert_eq!(mismatches, 0,
        "{league}: {}/{} ordered production-fixture mismatches vs \
         captured GDI",
        mismatches, expected_count);
}

fn infer_round_within_half(f: &cm_domain::HeadlessSeasonFixture) -> i16 {
    // The source string includes "round <N>, outer <M>, last_round=..."
    // — we exposed it via the exact engine's source label. Parse it.
    let s = &f.source;
    let key = "round ";
    if let Some(idx) = s.find(key) {
        let tail = &s[idx + key.len()..];
        let end = tail.find(|c: char| !c.is_ascii_digit()).unwrap_or(tail.len());
        return tail[..end].parse().unwrap_or(-1);
    }
    -1
}

// C11.2 production fixture golden. Hard-asserts 0/N ordered
// mismatches for the real production dispatch fed a captured GDI
// boot's (GameRngState + DAT_00dbc340) via
// `NewGameOptions.initial_game_rng_state.dbc340_cli_seed`.
#[test]
fn premier_ordered_production_fixtures_match_captured() {
    run_production_and_diff("prem", 7, 380);
}
#[test]
fn first_ordered_production_fixtures_match_captured() {
    run_production_and_diff("first", 8, 552);
}
#[test]
fn second_ordered_production_fixtures_match_captured() {
    run_production_and_diff("second", 9, 552);
}
#[test]
fn third_ordered_production_fixtures_match_captured() {
    run_production_and_diff("third", 10, 552);
}
#[test]
fn conference_ordered_production_fixtures_match_captured() {
    run_production_and_diff("conf", 93, 462);
}
