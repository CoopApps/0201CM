//! Verify the club History view-model against the Frida-captured Chester values:
//! Third Division · Runners Up · 1986, 1994; the record category lists; the
//! View-dropdown competitions.
//!
//! Run: cargo run -q -p cm-domain --bin club_history_probe

use std::path::Path;

use cm_domain::{typed_records::ClubView, World};

fn main() {
    let world = World::read_rust_db_dir(Path::new("D:/cm0102-rs/rust-db")).expect("read rust-db");

    // Find Chester by name.
    let chester = world
        .core
        .clubs
        .iter()
        .map(|c| ClubView::new(c))
        .find(|v| v.primary_name().contains("Chester") && !v.primary_name().contains("field"))
        .map(|v| v.id());
    let Some(club_id) = chester else {
        eprintln!("Chester not found");
        std::process::exit(2);
    };

    let v = world.club_history_view(club_id);
    println!("title: {}", v.title);
    println!("top tabs: {:?}", v.top_tabs);
    println!("bottom tabs: {:?}", v.bottom_tabs);
    println!("\nHONOURS ({} rows):", v.honours.len());
    for h in &v.honours {
        println!("  {} | {} | {}", h.competition, h.achievement, h.years);
    }
    println!("\nVIEW menu: {:?}", v.view_menu);
    println!("\nRecords (all-time) labels: {}", v.records_all_time.len());
    for r in &v.records_all_time {
        println!("  {} = {}", r.label, r.value);
    }
    println!("\nRecords (this season) labels: {}", v.records_this_season.len());

    // Assertion vs capture.
    let ru = v.honours.iter().any(|h| h.achievement == "Runners Up" && h.years.contains("1986") && h.years.contains("1994"));
    println!("\nCAPTURE CHECK — a Runners-Up honour with years 1986 & 1994: {}",
        if ru { "PASS" } else { "FAIL (check comp-id space / club id)" });
}
