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

    // Assertion vs capture: honours row.
    let ru = v.honours.iter().any(|h| h.achievement == "Runners Up" && h.years.contains("1986") && h.years.contains("1994"));
    println!("\nCAPTURE CHECK — a Runners-Up honour with years 1986 & 1994: {}",
        if ru { "PASS" } else { "FAIL (check comp-id space / club id)" });

    // Assertion vs capture: the two shipped club records.
    let apps = v.records_all_time.iter().find(|r| r.label == "Most League Apps for Club").map(|r| r.value.as_str()).unwrap_or("");
    let goals = v.records_all_time.iter().find(|r| r.label == "Most League Goals for Club").map(|r| r.value.as_str()).unwrap_or("");
    println!("CAPTURE CHECK — Most League Apps '361 - Stuart Rimmer': {}",
        if apps == "361 - Stuart Rimmer" { "PASS" } else { "FAIL" });
    println!("CAPTURE CHECK — Most League Goals '124 - Stuart Rimmer': {}",
        if goals == "124 - Stuart Rimmer" { "PASS" } else { "FAIL" });
    // This-season period resets both to "-" (capture-confirmed).
    let ts_apps = v.records_this_season.iter().find(|r| r.label == "Most League Apps for Club").map(|r| r.value.as_str()).unwrap_or("");
    println!("CAPTURE CHECK — this-season Most League Apps '-': {}",
        if ts_apps == "-" { "PASS" } else { "FAIL" });

    // View menu: fixed aggregates + the club's league (participation, not
    // honours). Chester -> [Honours, Domestic Leagues, Conference] (cups appended
    // by the caller from runtime cup state).
    let vm_ok = v.view_menu.first().map(|s| s == "Honours").unwrap_or(false)
        && v.view_menu.iter().any(|s| s == "Domestic Leagues")
        && v.view_menu.iter().any(|s| s == "Conference")
        && !v.view_menu.iter().any(|s| s == "Third Division");
    println!("CAPTURE CHECK — View menu is participation-based (Conference, not the Third Division honour): {}",
        if vm_ok { "PASS" } else { "FAIL" });

    // Pro Vercelli (capture 11, 2037 save): Serie A · Winners · 1908.. ;
    // Serie C1/A · Third Placed · 2034. Locks the placing text: leagues use
    // "Winners" (NOT "Champions") and third place is "Third Placed".
    if let Some(pv) = world.core.clubs.iter().map(|c| ClubView::new(c))
        .find(|v| v.primary_name().contains("Vercelli")).map(|v| v.id())
    {
        let pvh = world.club_history_view(pv);
        println!("\nPro Vercelli HONOURS:");
        for h in &pvh.honours { println!("  {} | {} | {}", h.competition, h.achievement, h.years); }
        let serie_a_winners = pvh.honours.iter().any(|h| h.competition.contains("Serie A") && h.achievement == "Winners" && h.years.contains("1908"));
        let no_champions = !pvh.honours.iter().any(|h| h.achievement == "Champions");
        println!("CAPTURE CHECK — Serie A 'Winners' incl 1908 (leagues use Winners, not Champions): {}",
            if serie_a_winners && no_champions { "PASS" } else { "FAIL" });
    }
}
