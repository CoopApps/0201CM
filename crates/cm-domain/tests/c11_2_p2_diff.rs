//! C11.2 point 1 — production P2 diff vs captured GDI P2.
//!
//! Runs `matrix_perturb` through the REAL production path
//! (rust-db data + captured initial RNG state) and compares the
//! post-perturb slot -> club_id ordering to the GDI capture,
//! comparing by canonical club NAME (a stable semantic identity;
//! numeric IDs are known to differ between rust-db and captured
//! GDI per C11.1 findings).
//!
//! This isolates whether divergence enters the pipeline in
//! perturb (Fisher-Yates + E1..E4 identity lookups) or later
//! (walker / driver emission).

use cm_domain::eng_second_fixtures::{matrix_perturb, PerturbConstants, StadiumClubResolver};
use cm_domain::english_traditional::{
    english_runtime_spec_for, EnglishClubEntry, ENGLISH_TRADITIONAL_COMP_IDS,
};
use cm_domain::game_rng::{GameRng, GameRngState};
use cm_domain::World;
use std::collections::BTreeMap;
use std::path::Path;

fn strip_pad(s: &str) -> String {
    s.trim_end_matches(|c: char| c == '\u{fffd}' || c == '\0' || c == ' ')
        .to_string()
}

fn load_p1_initial_rng_state(lid: &str) -> Option<GameRngState> {
    let path = format!(
        "D:/cm0102-rs/reports/fixture_disasm/runtime/20260914_180417_five_leagues_{lid}_lineage.jsonl"
    );
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
            // Find the FIRST PERTURB srand seed — this is
            // `year + DAT_00dbc340`.
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
    // C11.2: extract DAT_00dbc340 as `phase_c_srand_seed - base_year`.
    // Default to 0 if missing (production baseline).
    let dbc340 = phase_c_seed.map(|s| s.wrapping_sub(2001)).unwrap_or(0);
    Some(GameRngState {
        cursor: init.get("cursor_off")?.as_u64()? as u32,
        jitter: init.get("jitter")?.as_u64()? as u32,
        lcg_state: init.get("lcg_state")?.as_u64()? as u32,
        dbc340_cli_seed: dbc340,
    })
}

/// Load captured P2 as an ordered `Vec<String>` of canonical
/// (name-normalised) club names.
fn load_p2_names(lid: &str) -> Option<Vec<String>> {
    let path = format!(
        "D:/cm0102-rs/reports/fixture_disasm/runtime/20260914_180417_five_leagues_{lid}_lineage.jsonl"
    );
    let text = std::fs::read_to_string(&path).ok()?;
    for line in text.lines() {
        if line.is_empty() { continue; }
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        if v.get("op").and_then(|s| s.as_str()) == Some("clubs_snap")
            && v.get("point").and_then(|s| s.as_str()) == Some("P2")
        {
            let entries = v.get("entries")?.as_array()?;
            let mut with_slot: Vec<(u64, String)> = Vec::new();
            for e in entries {
                let slot = e.get("slot")?.as_u64()?;
                let name = strip_pad(e.get("club_name")?.as_str()?);
                with_slot.push((slot, name));
            }
            with_slot.sort_by_key(|(s, _)| *s);
            return Some(with_slot.into_iter().map(|(_, n)| n).collect());
        }
    }
    None
}

fn build_entries(world: &World, comp_id: u32) -> Vec<EnglishClubEntry> {
    let stadium_alt: BTreeMap<i32, Option<i32>> = world
        .references
        .stadiums
        .iter()
        .map(|s| (s.id as i32, s.alt_stadium_id))
        .collect();
    world
        .club_members_of_competition(comp_id)
        .into_iter()
        .map(|(cid, name)| {
            let stadium_id = world.core.clubs.iter()
                .find(|c| cm_domain::ClubView::new(c).id() == cid)
                .and_then(|c| cm_domain::ClubView::new(c).stadium_id());
            let alt = stadium_id
                .and_then(|sid| stadium_alt.get(&sid).copied().flatten());
            EnglishClubEntry {
                club_id: cid,
                club_name: name,
                stadium_id,
                alt_stadium_id: alt,
            }
        })
        .collect()
}

fn run_perturb_and_diff(league: &str, comp_id: u32) {
    let rust_db = Path::new("D:/cm0102-rs/rust-db");
    if !rust_db.exists() {
        eprintln!("rust-db not present; skipping {league}");
        return;
    }
    let world = World::read_rust_db_dir(rust_db).expect("read rust-db");
    let spec = english_runtime_spec_for(comp_id).expect("spec");
    let entries = build_entries(&world, comp_id);
    assert_eq!(entries.len(), spec.n_clubs as usize);

    let initial = load_p1_initial_rng_state(league).expect("captured initial rng");
    let dbc340 = initial.dbc340_cli_seed;
    let mut rng = GameRng::from_state_snapshot(initial);

    // Build the same clubs_table + resolver the production dispatch
    // does.
    let mut clubs_table = vec![0u8; entries.len() * 0x3b];
    for (i, e) in entries.iter().enumerate() {
        clubs_table[i * 0x3b..i * 0x3b + 4]
            .copy_from_slice(&(e.club_id as i32).to_le_bytes());
    }
    let resolver = StadiumClubResolver::new_by_club(
        entries.iter().map(|e| (e.club_id as i32, e.stadium_id, e.alt_stadium_id))
    );

    let n_even = spec.n_clubs as i32 + (spec.n_clubs as i32 & 1);
    let pc = PerturbConstants::default();
    matrix_perturb(
        spec.n_clubs as i16,
        2001, // base_year
        spec.walker_flag_byte as u16,
        Some(spec.comp_id as i32),
        &mut clubs_table,
        &resolver,
        &mut rng,
        n_even,
        dbc340,
        pc.dat_009bba9c,
        pc.dat_009bc5a8,
        pc.dat_009bc5ac,
    );

    // Decode post-perturb club_ids → names.
    let mut rust_p2_names: Vec<String> = Vec::new();
    let id_to_name: BTreeMap<i32, String> = entries.iter()
        .map(|e| (e.club_id as i32, e.club_name.clone()))
        .collect();
    for i in 0..spec.n_clubs as usize {
        let cid = i32::from_le_bytes(
            clubs_table[i * 0x3b..i * 0x3b + 4].try_into().unwrap());
        rust_p2_names.push(id_to_name.get(&cid).cloned().unwrap_or_default());
    }

    let captured = load_p2_names(league).expect("captured P2");
    assert_eq!(rust_p2_names.len(), captured.len());

    let mut mismatches = 0;
    let mut first: Vec<String> = Vec::new();
    for (i, (r, c)) in rust_p2_names.iter().zip(captured.iter()).enumerate() {
        if r.trim() != c.trim() {
            mismatches += 1;
            if first.len() < 5 {
                first.push(format!("  slot {i}: rust {r:?} vs exe {c:?}"));
            }
        }
    }
    println!("{league} P2 name-diff: {}/{} mismatches", mismatches, rust_p2_names.len());
    if mismatches > 0 {
        println!("{}", first.join("\n"));
    }
    assert_eq!(mismatches, 0,
        "{league} P2 by-name: {}/{} slots differ; \
         production perturb diverges from captured GDI perturb (see \
         C11.2 identity/resolver investigation).",
        mismatches, rust_p2_names.len());
}

#[test]
fn premier_p2_matches_captured() { run_perturb_and_diff("prem", 7); }
#[test]
fn first_p2_matches_captured()   { run_perturb_and_diff("first", 8); }
#[test]
fn second_p2_matches_captured()  { run_perturb_and_diff("second", 9); }
#[test]
fn third_p2_matches_captured()   { run_perturb_and_diff("third", 10); }
#[test]
fn conference_p2_matches_captured() { run_perturb_and_diff("conf", 93); }
