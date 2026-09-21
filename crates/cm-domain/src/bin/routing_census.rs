//! Census of the detailed-vs-background match routing (perf/fidelity tranche).
//!
//! Replicates the EXACT production predicate in `resolve_fixture_via_exe_port`
//! (`lib.rs` ~20666): a fixture runs the detailed token model iff
//! `detailed(home_nation) || detailed(away_nation)`, where
//! `detailed(nid) = if nid==0 { true } else { nation_tiers[nid].detailed_matches
//! (unwrap_or true) }`, and `nid = finance.club_nation[club].unwrap_or(0)`.
//!
//! Classifies every scheduled fixture and prints the counts the routing tranche
//! needs. National-team continental fixtures are handled by national_match.rs
//! (marked Played before the club batch), so they are counted separately.
//!
//! Run: cargo run -q -p cm-domain --bin routing_census

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use cm_domain::{NewGameOptions, World};

fn main() {
    let rust_db = Path::new("D:/cm0102-rs/rust-db");
    let world = World::read_rust_db_dir(rust_db).expect("read rust-db");
    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
        initial_game_rng_state: None,
    };
    let save = world.new_game_from_rust_db(rust_db, &options);

    // National-comp ids (handled by national_match.rs, not the club batch).
    let national: BTreeSet<u32> = [
        save.african_nations.as_ref().map(|s| s.competition_id),
        save.asia_cup_of_nations.as_ref().map(|s| s.competition_id),
        save.european_championship.as_ref().map(|s| s.competition_id),
        save.fifa_confederations_cup.as_ref().map(|s| s.competition_id),
        save.concacaf_gold_cup.as_ref().map(|s| s.competition_id),
    ]
    .into_iter()
    .flatten()
    .collect();

    // Domestic-cup comp ids (still club-routed, but flagged as cups).
    let cup_comps: BTreeSet<u32> = save
        .domestic_cups
        .iter()
        .flat_map(|c| [c.runtime_comp_id, c.real_comp_id.max(0) as u32])
        .collect();

    // England's nation id (the one Foreground/detailed nation here).
    let england_id: Option<i32> = save
        .nation_tiers
        .iter()
        .find(|t| t.detailed_matches)
        .map(|t| t.nation_id as i32);

    // OLD (buggy) predicate: nid==0 -> detailed, unwrap_or(true).
    let detailed_old = |nid: i32| -> bool {
        if nid == 0 {
            return true;
        }
        save.nation_tiers
            .iter()
            .find(|t| t.nation_id as i32 == nid)
            .map(|t| t.detailed_matches)
            .unwrap_or(true)
    };
    // NEW (faithful) predicate: selected iff club's nation is Foreground;
    // unknown/foreign -> not selected. Human-club exception handled below.
    let selected_new = |club: u32| -> bool {
        match save.finance.club_nation.get(&club) {
            Some(&nid) => save
                .nation_tiers
                .iter()
                .find(|t| t.nation_id as i32 == nid)
                .map(|t| t.detailed_matches)
                .unwrap_or(false),
            None => false,
        }
    };
    let human_clubs: BTreeSet<u32> = save.humans.iter().filter_map(|h| h.club).collect();
    let club_nid = |club: u32| -> i32 { save.finance.club_nation.get(&club).copied().unwrap_or(0) };

    // BEFORE/AFTER detailed counts over club-routed fixtures.
    let mut old_detailed = 0usize;
    let mut new_detailed = 0usize;

    let mut total = 0usize;
    let mut national_fx = 0usize;
    let mut cup_fx = 0usize;
    let mut routed = 0usize;
    let mut detailed_fx = 0usize;
    let mut background_fx = 0usize;
    // Reasons a fixture is detailed:
    let mut english = 0usize; // at least one side is the England nation
    let mut selected_foreign = 0usize; // detailed via a Foreground nation != England (none here)
    let mut unsel_foreign_default = 0usize; // detailed ONLY because a side's nid==0 (bad default)
    let mut unsel_foreign_tier = 0usize; // detailed via unwrap_or(true): nid not in tiers
    // How the misclassification enters:
    let mut comp_hits: BTreeMap<u32, usize> = BTreeMap::new(); // misclassified detailed comps

    for f in &save.season.fixtures {
        total += 1;
        if national.contains(&f.competition_id) {
            national_fx += 1;
            continue;
        }
        routed += 1;
        if cup_comps.contains(&f.competition_id) {
            cup_fx += 1;
        }
        let hn = club_nid(f.home_club_id);
        let an = club_nid(f.away_club_id);
        let dh = detailed_old(hn);
        let da = detailed_old(an);
        let want = dh || da;
        // New faithful predicate for before/after.
        let want_new = human_clubs.contains(&f.home_club_id)
            || human_clubs.contains(&f.away_club_id)
            || selected_new(f.home_club_id)
            || selected_new(f.away_club_id);
        if want {
            old_detailed += 1;
        }
        if want_new {
            new_detailed += 1;
        }
        if !want {
            background_fx += 1;
            continue;
        }
        detailed_fx += 1;
        // Why detailed? Categorize by the strongest reason.
        let is_english = |nid: i32| england_id == Some(nid) && nid != 0;
        if is_english(hn) || is_english(an) {
            english += 1;
        } else {
            // Detailed but neither side is England. Determine if via nid==0
            // default, via unwrap_or(true) (nid not in tiers), or a genuine
            // second Foreground nation.
            let foreground_nonenglish = |nid: i32| {
                nid != 0
                    && england_id != Some(nid)
                    && save
                        .nation_tiers
                        .iter()
                        .find(|t| t.nation_id as i32 == nid)
                        .map(|t| t.detailed_matches)
                        .unwrap_or(false)
            };
            if foreground_nonenglish(hn) || foreground_nonenglish(an) {
                selected_foreign += 1;
            } else if hn == 0 || an == 0 {
                unsel_foreign_default += 1;
                *comp_hits.entry(f.competition_id).or_insert(0) += 1;
            } else {
                // nid != 0, not England, not foreground, yet detailed(nid)==true
                // => nation missing from nation_tiers -> unwrap_or(true).
                unsel_foreign_tier += 1;
                *comp_hits.entry(f.competition_id).or_insert(0) += 1;
            }
        }
    }

    println!("=== detailed/background routing census (England new-game 2001) ===");
    println!("england nation id (Foreground/detailed): {england_id:?}");
    println!("total scheduled fixtures        : {total}");
    println!("  national-team (national_match): {national_fx}");
    println!("  club-routed (reach resolve)   : {routed}");
    println!("    of which domestic-cup       : {cup_fx}");
    println!("  -> DETAILED (token model)     : {detailed_fx}");
    println!("  -> BACKGROUND (condensed)     : {background_fx}");
    println!();
    println!("detailed breakdown by reason:");
    println!("  English (selected)            : {english}");
    println!("  selected foreign (Foreground) : {selected_foreign}");
    println!("  MISCLASSIFIED nid==0 default  : {unsel_foreign_default}");
    println!("  MISCLASSIFIED not-in-tiers    : {unsel_foreign_tier}");
    println!();
    println!("=== BEFORE/AFTER (club-routed fixtures sent to DETAILED token model) ===");
    println!("  OLD predicate detailed : {old_detailed}");
    println!("  NEW predicate detailed : {new_detailed}");
    println!("  moved to background    : {}", old_detailed.saturating_sub(new_detailed));
    println!();
    println!("top comps misclassified-detailed (comp_id -> #fixtures):");
    let mut v: Vec<(u32, usize)> = comp_hits.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    for (comp, n) in v.into_iter().take(15) {
        let nm = save
            .season
            .fixtures
            .iter()
            .find(|f| f.competition_id == comp)
            .map(|f| f.competition_name.clone())
            .unwrap_or_default();
        println!("  comp {comp:>6}: {n:>5}   {nm}");
    }
}
