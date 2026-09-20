//! Census of `snapshot_team_for_engine` outcomes across the whole club DB and
//! across the clubs that actually play scheduled fixtures in a started game.
//!
//! Measures (does NOT estimate) the failure classes the fidelity review asked
//! for: real-full / free-agent-fill / synthetic-fabrication / none, and crosses
//! them with whether the club's nation runs detailed matches (the player-visible
//! engine path) and whether the club actually appears in scheduled fixtures.
//!
//! Run: cargo run -p cm-domain --bin census_snapshot

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

    // --- Real-player count per club, and the shared free-agent pool size. ---
    let mut real_by_club: BTreeMap<i32, usize> = BTreeMap::new();
    let mut free_agents = 0usize;
    for p in &save.player_ratings.players {
        match p.club_id {
            Some(c) => *real_by_club.entry(c).or_insert(0) += 1,
            None => free_agents += 1,
        }
    }

    // Detailed-nation lookup as PRODUCTION does it (lib.rs:20535): unknown
    // nation (nid==0) or a nation not in nation_tiers defaults to DETAILED.
    let detailed_nation = |nid: i32| -> bool {
        if nid == 0 {
            return true;
        }
        save.nation_tiers
            .iter()
            .find(|t| t.nation_id as i32 == nid)
            .map(|t| t.detailed_matches)
            .unwrap_or(true)
    };
    // STRICT detailed: only a nation explicitly flagged detailed_matches==true
    // in nation_tiers. This is what "the player actually watches" should mean.
    let strict_detailed = |nid: i32| -> bool {
        save.nation_tiers
            .iter()
            .find(|t| t.nation_id as i32 == nid)
            .map(|t| t.detailed_matches)
            .unwrap_or(false)
    };

    // Classification mirrors snapshot_team_for_engine's decision tree:
    //   >=6 real                         -> RealFull
    //   real<6 but free pool >= needed    -> FreeAgentFill (ephemeral non-club real players)
    //   free pool < needed                -> Synthetic (fabricated players)
    //   (post-synthesis always >=6)       -> None never reached
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Class {
        RealFull,
        FreeAgentFill,
        Synthetic,
    }
    let classify = |real_n: usize| -> Class {
        if real_n >= 6 {
            Class::RealFull
        } else {
            let needed = 6 - real_n;
            // Free-agent fill is a non-mutating per-fixture borrow of the WHOLE
            // shared free-agent pool, so every club independently sees all of it.
            if free_agents >= needed {
                Class::FreeAgentFill
            } else {
                Class::Synthetic
            }
        }
    };

    // ---- Whole-DB census over every club with a nation entry (all seeded). ----
    let all_clubs: BTreeSet<u32> = save.finance.club_nation.keys().copied().collect();
    let mut wdb: BTreeMap<(&str, bool), usize> = BTreeMap::new();
    let mut real_hist: BTreeMap<usize, usize> = BTreeMap::new(); // real_n -> #clubs (capped 10+)
    for &c in &all_clubs {
        let nid = save.finance.club_nation.get(&c).copied().unwrap_or(0);
        let det = detailed_nation(nid);
        let real_n = real_by_club.get(&(c as i32)).copied().unwrap_or(0);
        *real_hist.entry(real_n.min(10)).or_insert(0) += 1;
        let name = match classify(real_n) {
            Class::RealFull => "real_full",
            Class::FreeAgentFill => "free_agent_fill",
            Class::Synthetic => "synthetic",
        };
        *wdb.entry((name, det)).or_insert(0) += 1;
    }

    // ---- Clubs that actually play scheduled fixtures in the started game. ----
    let mut fixture_clubs: BTreeSet<u32> = BTreeSet::new();
    for f in &save.season.fixtures {
        fixture_clubs.insert(f.home_club_id);
        fixture_clubs.insert(f.away_club_id);
    }
    let mut fx: BTreeMap<(&str, bool), usize> = BTreeMap::new();
    for &c in &fixture_clubs {
        let nid = save.finance.club_nation.get(&c).copied().unwrap_or(0);
        let det = detailed_nation(nid);
        let real_n = real_by_club.get(&(c as i32)).copied().unwrap_or(0);
        let name = match classify(real_n) {
            Class::RealFull => "real_full",
            Class::FreeAgentFill => "free_agent_fill",
            Class::Synthetic => "synthetic",
        };
        *fx.entry((name, det)).or_insert(0) += 1;
    }

    // -------------------------- Report ----------------------------------
    println!("=== snapshot_team_for_engine census (England new-game, 2001) ===");
    println!("free-agent pool size (shared): {free_agents}");
    println!("total clubs with nation entry: {}", all_clubs.len());
    println!("clubs in real player_ratings : {}", real_by_club.len());
    println!();

    println!("-- real-player-count histogram (whole DB, 10 = 10+) --");
    for (n, c) in &real_hist {
        println!("  real={:>2} : {:>5} clubs", n, c);
    }
    println!();

    let dump = |title: &str, m: &BTreeMap<(&str, bool), usize>, total: usize| {
        println!("-- {title} (total {total}) --");
        for class in ["real_full", "free_agent_fill", "synthetic"] {
            let det = m.get(&(class, true)).copied().unwrap_or(0);
            let bg = m.get(&(class, false)).copied().unwrap_or(0);
            println!(
                "  {:<16} detailed={:>5}  background={:>5}  total={:>5}",
                class,
                det,
                bg,
                det + bg
            );
        }
        println!();
    };
    dump("WHOLE DB", &wdb, all_clubs.len());
    dump("SCHEDULED-FIXTURE CLUBS", &fx, fixture_clubs.len());

    // The decision-relevant number: detailed clubs that actually play but are
    // NOT real_full (i.e. get non-club players on the visible engine path).
    let det_fill = fx.get(&("free_agent_fill", true)).copied().unwrap_or(0)
        + fx.get(&("synthetic", true)).copied().unwrap_or(0);
    let det_total = fx.get(&("real_full", true)).copied().unwrap_or(0) + det_fill;
    println!("=== KEY: detailed + scheduled clubs NOT real_full: {det_fill} of {det_total} ===");
    println!("(these field non-club players on the player-visible engine path)");
    println!();

    // ---- WHY: for the affected clubs, total staff records (the regen gate's
    // measure) vs CA>0 rated players (the engine's measure). The regen boot
    // gate at lib.rs:16511 uses `staff.type6.current_club_id()` counts; if those
    // exceed the 8-threshold while rated<6, the gate never fires. ----
    let mut staff_by_club: BTreeMap<u32, usize> = BTreeMap::new();
    for s in &world.staff.type6 {
        if let Some(c) = s.current_club_id() {
            *staff_by_club.entry(c).or_insert(0) += 1;
        }
    }
    let mut gate_wrong = 0usize; // staff>=8 but rated<6 (gate misses it)
    let mut genuinely_thin = 0usize; // staff<8 too (gate should have fired)
    let mut sample = Vec::new();
    for &c in &fixture_clubs {
        let nid = save.finance.club_nation.get(&c).copied().unwrap_or(0);
        if !detailed_nation(nid) {
            continue;
        }
        let rated = real_by_club.get(&(c as i32)).copied().unwrap_or(0);
        if rated >= 6 {
            continue;
        }
        let staff = staff_by_club.get(&c).copied().unwrap_or(0);
        if staff >= 8 {
            gate_wrong += 1;
        } else {
            genuinely_thin += 1;
        }
        if sample.len() < 12 {
            sample.push((c, staff, rated));
        }
    }
    println!("-- affected detailed+scheduled clubs: staff-record count vs rated(CA>0) count --");
    println!("  gate MISSES (staff>=8 but rated<6): {gate_wrong}");
    println!("  genuinely thin (staff<8 too)       : {genuinely_thin}");
    println!("  sample (club_id, staff_records, rated_CA>0):");
    for (c, staff, rated) in &sample {
        println!("    club {c:>6}: staff={staff:>3}  rated={rated:>2}");
    }
    println!();

    // Which competitions do the 81 empty detailed clubs actually play in?
    let affected: BTreeSet<u32> = fixture_clubs
        .iter()
        .copied()
        .filter(|&c| {
            let nid = save.finance.club_nation.get(&c).copied().unwrap_or(0);
            detailed_nation(nid) && real_by_club.get(&(c as i32)).copied().unwrap_or(0) < 6
        })
        .collect();
    let mut comp_tally: BTreeMap<u32, usize> = BTreeMap::new(); // comp_id -> #fixtures involving an affected club
    for f in &save.season.fixtures {
        if affected.contains(&f.home_club_id) || affected.contains(&f.away_club_id) {
            *comp_tally.entry(f.competition_id).or_insert(0) += 1;
        }
    }
    println!("-- competitions the 81 empty detailed clubs appear in (comp_id -> #fixtures) --");
    for (comp, n) in &comp_tally {
        // Name via the season's own fixture rows (they carry competition_name).
        let nm = save
            .season
            .fixtures
            .iter()
            .find(|f| f.competition_id == *comp)
            .map(|f| f.competition_name.clone())
            .unwrap_or_default();
        println!("    comp {comp:>4}: {n:>5} fixtures   {nm}");
    }
    println!();

    // STRICT detailed re-measure: how many clubs whose nation is EXPLICITLY
    // detailed (i.e. England here) are thin? This is the real "valid club
    // misses the intended path" number, excluding foreign/default-detailed.
    let mut strict_thin = 0usize;
    let mut strict_total = 0usize;
    let mut strict_sample = Vec::new();
    for &c in &fixture_clubs {
        let nid = save.finance.club_nation.get(&c).copied().unwrap_or(0);
        if !strict_detailed(nid) {
            continue;
        }
        strict_total += 1;
        let rated = real_by_club.get(&(c as i32)).copied().unwrap_or(0);
        if rated < 6 {
            strict_thin += 1;
            if strict_sample.len() < 12 {
                strict_sample.push((c, nid, rated));
            }
        }
    }
    println!("=== STRICT detailed (nation explicitly detailed_matches==true) ===");
    println!("  scheduled clubs in strict-detailed nations : {strict_total}");
    println!("  of those, thin (rated<6)                   : {strict_thin}");
    for (c, nid, rated) in &strict_sample {
        println!("    club {c:>6} (nation {nid}): rated={rated}");
    }
}
