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

    use cm_domain::MatchDetailMode;
    let _ = &human_clubs; // (human detection now lives inside match_detail_mode)

    let mut total = 0usize;
    let mut national_fx = 0usize;
    let mut cup_fx = 0usize;
    let mut routed = 0usize;
    // AFTER (real 3-way router):
    let mut not_sim = 0usize;
    let mut instant = 0usize;
    let mut detailed = 0usize;
    let mut invalid = 0usize; // both clubs unresolved AND not a national comp
    let mut invalid_sample: Vec<(u32, u32, u32)> = Vec::new();
    // Detailed breakdown by reason:
    let mut d_human = 0usize;
    let mut d_selected = 0usize;
    let mut d_cross = 0usize; // continental promo (cross-nation w/ a selected club)
    // BEFORE (old buggy predicate):
    let mut old_detailed = 0usize;
    // NotSimulated by competition (the fixtures we now skip):
    let mut notsim_by_comp: BTreeMap<u32, usize> = BTreeMap::new();

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
        // BEFORE: old predicate (nid==0 -> detailed, unwrap_or(true)).
        let hn = club_nid(f.home_club_id);
        let an = club_nid(f.away_club_id);
        if detailed_old(hn) || detailed_old(an) {
            old_detailed += 1;
        }
        // Unresolved-identity detection (both clubs missing a nation).
        let h_res = save.finance.club_nation.contains_key(&f.home_club_id);
        let a_res = save.finance.club_nation.contains_key(&f.away_club_id);
        if !h_res && !a_res {
            invalid += 1;
            if invalid_sample.len() < 10 {
                invalid_sample.push((f.competition_id, f.home_club_id, f.away_club_id));
            }
        }
        // AFTER: the REAL production router.
        match save.match_detail_mode(f.home_club_id, f.away_club_id) {
            MatchDetailMode::NotSimulated => {
                not_sim += 1;
                *notsim_by_comp.entry(f.competition_id).or_insert(0) += 1;
            }
            MatchDetailMode::Instant => instant += 1,
            MatchDetailMode::Detailed => {
                detailed += 1;
                // Attribute the reason (for the breakdown).
                if human_clubs.contains(&f.home_club_id) || human_clubs.contains(&f.away_club_id) {
                    d_human += 1;
                } else if selected_new(f.home_club_id) || selected_new(f.away_club_id) {
                    d_selected += 1;
                } else {
                    d_cross += 1; // detailed w/o human or selected => cross-nation promo
                }
            }
        }
    }

    println!("=== match routing census (England new-game 2001) — 3-way ===");
    println!("england nation id (Foreground/selected): {england_id:?}");
    println!("total scheduled fixtures            : {total}");
    println!("  national-team (national_match)    : {national_fx}");
    println!("  club-routed (reach the router)    : {routed}");
    println!("    of which domestic-cup           : {cup_fx}");
    println!();
    println!("AFTER (real match_detail_mode):");
    println!("  NotSimulated (skipped, no engine) : {not_sim}");
    println!("  Instant   (simulated, token model): {instant}");
    println!("  Detailed  (simulated, token model): {detailed}");
    println!("  Invalid/unresolved (both nation-less): {invalid}");
    println!();
    println!("Detailed breakdown by reason:");
    println!("  human-managed club                : {d_human}");
    println!("  selected nation (Foreground)      : {d_selected}");
    println!("  continental promo (cross-nation)  : {d_cross}");
    println!();
    println!("BEFORE (old nid==0->detailed predicate) DETAILED: {old_detailed}");
    println!("AFTER  detailed (token)                        : {detailed}");
    println!("AFTER  instant  (token)                        : {instant}");
    println!("moved OUT of the engine entirely (NotSimulated): {not_sim}");
    println!();
    if !invalid_sample.is_empty() {
        println!("invalid/unresolved sample (comp, home, away): {invalid_sample:?}");
        println!();
    }
    println!("top NotSimulated competitions (comp_id -> #fixtures skipped):");
    let mut v: Vec<(u32, usize)> = notsim_by_comp.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    for (comp, n) in v.into_iter().take(12) {
        let nm = save
            .season
            .fixtures
            .iter()
            .find(|ff| ff.competition_id == comp)
            .map(|ff| ff.competition_name.clone())
            .unwrap_or_default();
        println!("  comp {comp:>6}: {n:>5}   {nm}");
    }
}
