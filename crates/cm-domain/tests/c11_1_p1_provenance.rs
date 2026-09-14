//! C11.1 — P1 provenance test (points 4/5/6).
//!
//! Loads the shipped rust-db and asserts that the production P1
//! roster from `World::club_members_of_competition` matches the
//! captured GDI constructor-entry order **by club name** for all
//! five English simulated leagues.
//!
//! Why name-match not id-match: rust-db's shipped `club.dat`
//! conversion uses different numeric club IDs than the running GDI
//! build's memory record (a rust-db-vs-shipped-dat renumbering
//! documented as a separate scope item — the club NAME field
//! matches byte-for-byte across both). Order fidelity — which is
//! what perturb + Fisher-Yates + driver actually consume — is
//! settled by names.
//!
//! The captured P1 file lives at
//! `reports/fixture_disasm/runtime/20260914_180417_five_leagues_<lid>_lineage.jsonl`
//! and carries per-slot `(club_id, club_name)` from the exe's
//! `comp+0xb1` entry table via `dumpRoster` in the harness.

use cm_domain::World;
use std::path::Path;

fn strip_pad(s: &str) -> &str {
    // The exe's club name is a fixed-width Latin-1 field padded
    // with 0xFF ('�' when rendered in UTF-8) up to 48 bytes.
    // Strip either the sentinel or trailing NULs / spaces.
    s.trim_end_matches(|c: char| c == '\u{fffd}' || c == '\0' || c == ' ')
}

fn load_captured_p1_names(lid: &str) -> Option<Vec<String>> {
    let path = format!(
        "D:/cm0102-rs/reports/fixture_disasm/runtime/20260914_180417_five_leagues_{lid}_lineage.jsonl"
    );
    let text = std::fs::read_to_string(&path).ok()?;
    for line in text.lines() {
        if line.is_empty() { continue; }
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        if v.get("op").and_then(|s| s.as_str()) == Some("clubs_snap")
            && v.get("point").and_then(|s| s.as_str()) == Some("P1")
        {
            let entries = v.get("entries")?.as_array()?;
            let mut out = Vec::with_capacity(entries.len());
            let mut with_slot: Vec<(u64, String)> = Vec::new();
            for e in entries {
                let slot = e.get("slot")?.as_u64()?;
                let name = e.get("club_name")?.as_str()?;
                with_slot.push((slot, strip_pad(name).to_string()));
            }
            with_slot.sort_by_key(|(s, _)| *s);
            for (_, n) in with_slot { out.push(n); }
            return Some(out);
        }
    }
    None
}

fn ordered_name_diff(league: &str, comp_id: u32) {
    let rust_db = Path::new("D:/cm0102-rs/rust-db");
    if !rust_db.exists() {
        eprintln!("rust-db not present; skipping P1 diff for {league}");
        return;
    }
    let world = World::read_rust_db_dir(rust_db).expect("read rust-db");
    let members = world.club_members_of_competition(comp_id);
    let prod_names: Vec<String> = members.iter().map(|(_, n)| n.clone()).collect();
    let captured = match load_captured_p1_names(league) {
        Some(v) => v,
        None => {
            eprintln!("captured P1 file for {league} missing; skipping");
            return;
        }
    };
    assert_eq!(prod_names.len(), captured.len(),
        "{league}: production roster count {} != captured {}",
        prod_names.len(), captured.len());
    let mut mismatches = Vec::new();
    for (i, (p, c)) in prod_names.iter().zip(captured.iter()).enumerate() {
        if p.trim() != c.trim() {
            mismatches.push(format!("  slot {i}: rust {p:?} vs exe {c:?}"));
        }
    }
    if !mismatches.is_empty() {
        panic!("{league}: {} P1 slot(s) disagree by name:\n{}",
            mismatches.len(), mismatches.join("\n"));
    }
    println!("{league}: {}/{} P1 slots match by name", prod_names.len(), captured.len());
}

#[test]
fn premier_p1_names_match_captured() {
    ordered_name_diff("prem", 7);
}
#[test]
fn first_p1_names_match_captured() {
    ordered_name_diff("first", 8);
}
#[test]
fn second_p1_names_match_captured() {
    ordered_name_diff("second", 9);
}
#[test]
fn third_p1_names_match_captured() {
    ordered_name_diff("third", 10);
}
#[test]
fn conference_p1_names_match_captured() {
    ordered_name_diff("conf", 93);
}
