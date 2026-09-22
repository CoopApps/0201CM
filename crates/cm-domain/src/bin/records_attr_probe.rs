//! Verify the club Records apps/goals attribution against the captured Chester
//! answer: "Most League Apps for Club 361 - Stuart Rimmer", "Most League Goals
//! 124 - Stuart Rimmer". Decode (task aa66fc0520829a5..): staff_history +0x0a is
//! the CLUB (resolved via the 581-byte club pool), NOT a competition; the record
//! holder is max(sum(apps)) / max(sum(goals)) per (person, club).
//!
//! Run: cargo run -q -p cm-domain --bin records_attr_probe

use std::collections::HashMap;
use std::path::Path;

use cm_domain::{typed_records::ClubView, World};

fn main() {
    let world = World::read_rust_db_dir(Path::new("D:/cm0102-rs/rust-db")).expect("read rust-db");

    let chester = world
        .core
        .clubs
        .iter()
        .map(|c| ClubView::new(c))
        .find(|v| v.primary_name().contains("Chester") && !v.primary_name().contains("field"))
        .map(|v| (v.id(), v.primary_name()));
    let Some((club_id, club_name)) = chester else {
        eprintln!("Chester not found");
        std::process::exit(2);
    };
    println!("Club: {club_name} (id {club_id})");

    // Sum apps/goals per person for rows whose +0x0a field == this club id.
    let mut apps: HashMap<u32, u64> = HashMap::new();
    let mut goals: HashMap<u32, u64> = HashMap::new();
    for h in &world.references.staff_history {
        if h.competition_id == club_id {
            *apps.entry(h.person_id).or_default() += h.apps as u64;
            *goals.entry(h.person_id).or_default() += h.goals as u64;
        }
    }

    let top = |m: &HashMap<u32, u64>| -> Option<(u32, u64)> {
        m.iter().max_by_key(|(_, v)| **v).map(|(k, v)| (*k, *v))
    };
    let name = |pid: u32| {
        world
            .staff
            .type6
            .iter()
            .find(|p| p.id == pid)
            .map(|p| world.person_display_name(p))
            .unwrap_or_else(|| format!("person {pid}"))
    };

    if let Some((pid, n)) = top(&apps) {
        println!("Most League Apps for Club: {} - {}", n, name(pid));
    } else {
        println!("Most League Apps for Club: (no rows)");
    }
    if let Some((pid, n)) = top(&goals) {
        println!("Most League Goals for Club: {} - {}", n, name(pid));
    } else {
        println!("Most League Goals for Club: (no rows)");
    }

    println!("\nEXPECT: 361 - Stuart Rimmer / 124 - Stuart Rimmer");
}
