//! Multi-season probe: tick a full England game across the year rollover into
//! season 2 and report the cross-season defects (standings-row collision +
//! pre-promotion fixtures) concretely. A before/after harness for the
//! multi-season correctness tranche — NOT a golden yet.
//!
//! Run: RUSTFLAGS="-C debuginfo=0" cargo run -q -p cm-domain --bin multiseason_probe [days]

use std::collections::BTreeMap;
use std::path::Path;

use cm_domain::{HeadlessFixtureStatus, NewGameOptions, World};

const PREM: u32 = 7; // English Premier Division real comp id

fn main() {
    let days: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(400);

    let rust_db = Path::new("D:/cm0102-rs/rust-db");
    let mut world = World::read_rust_db_dir(rust_db).expect("read rust-db");
    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
        initial_game_rng_state: None,
    };
    let mut save = world.new_game_from_rust_db(rust_db, &options);

    let start_year = save.date.year;
    // Record the Premier membership at season 1 (division_id via ClubView is
    // World-side; here we approximate "who is in the Prem" from the season-1
    // standings rows for comp 7 before any rollover).
    let s1_prem: Vec<u32> = save
        .season
        .standings
        .iter()
        .filter(|s| s.competition_id == PREM)
        .map(|s| s.club_id)
        .collect();
    println!("season {start_year}-{:02}: Prem standings rows = {}", (start_year + 1) % 100, s1_prem.len());

    let mut announced_yearend = false;
    let mut announced_drain = false;
    for d in 0..days {
        save.tick_days_bound(&mut world, 1);
        // Progress on stderr (unbuffered) so a long run is observable.
        if (d + 1) % 30 == 0 {
            eprintln!(
                "  day {:>3}: {:04}-{:02}-{:02}  fixtures={}  standings={}  history={}",
                d + 1, save.date.year, save.date.month, save.date.day,
                save.season.fixtures.len(), save.season.standings.len(),
                save.season_history.len()
            );
        }
        if !announced_yearend
            && save.last_english_year_end_applied == Some(save.date.year)
            && save.date.month >= 5
        {
            announced_yearend = true;
            eprintln!("  [year-end applied {}-{:02}-{:02}]", save.date.year, save.date.month, save.date.day);
        }
        if !announced_drain && !save.season_history.is_empty() {
            announced_drain = true;
            eprintln!("  [season-roll drain done {}-{:02}-{:02}; archived {} table(s)]",
                save.date.year, save.date.month, save.date.day, save.season_history.len());
        }
    }
    println!("ticked {days} days -> date {:04}-{:02}-{:02}", save.date.year, save.date.month, save.date.day);
    println!("season_roll_comp_years = {:?}", save.season_roll_comp_years);

    // DEFECT 1/2 probe: standings rows for the Prem.
    let mut per_club_rows: BTreeMap<u32, usize> = BTreeMap::new();
    let mut max_played = 0u32;
    let mut worst: Option<(u32, u32, u32)> = None; // club, played, points
    for s in save.season.standings.iter().filter(|s| s.competition_id == PREM) {
        *per_club_rows.entry(s.club_id).or_insert(0) += 1;
        if s.played > max_played {
            max_played = s.played;
        }
        if worst.map(|(_, p, _)| s.played > p).unwrap_or(true) {
            worst = Some((s.club_id, s.played, s.points));
        }
    }
    let dup_clubs = per_club_rows.values().filter(|&&n| n > 1).count();
    let total_rows = per_club_rows.len();
    println!("\n--- DEFECT 1 (standings collision) ---");
    println!("Prem standings: {} distinct clubs, {} with >1 row (duplicates)", total_rows, dup_clubs);
    println!("max played in any Prem row: {max_played}  (a sane single season is <= 38)");
    if let Some((c, p, pts)) = worst {
        println!("worst row: club {c} played={p} points={pts}");
    }

    // DEFECT 2 (pre-promotion fixtures): count season-2 Prem fixtures and how
    // many involve a club that was NOT in season-1's Prem (i.e. a promoted club
    // correctly appearing, or old clubs wrongly still present).
    let s1_set: std::collections::BTreeSet<u32> = s1_prem.iter().copied().collect();
    let mut s2_prem_fixtures = 0usize;
    let mut s2_clubs: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for f in &save.season.fixtures {
        if f.competition_id != PREM {
            continue;
        }
        // Season-2 fixtures are the Pending ones after the rollover (season 1's
        // are all Played).
        if f.status == HeadlessFixtureStatus::Pending {
            s2_prem_fixtures += 1;
            s2_clubs.insert(f.home_club_id);
            s2_clubs.insert(f.away_club_id);
        }
    }
    let newcomers: Vec<u32> = s2_clubs.iter().copied().filter(|c| !s1_set.contains(c)).collect();
    let dropped: Vec<u32> = s1_set.iter().copied().filter(|c| !s2_clubs.contains(c)).collect();
    println!("\n--- DEFECT 2 (pre-promotion fixtures) ---");
    println!("season-2 Prem Pending fixtures: {s2_prem_fixtures}");
    println!("season-2 Prem clubs: {}", s2_clubs.len());
    println!("newcomers vs season-1 Prem (promoted in): {} {:?}", newcomers.len(), newcomers);
    println!("dropped from season-1 Prem (relegated out): {} {:?}", dropped.len(), dropped);
    println!("(if BOTH lists are empty after a rollover, promotions/relegations did NOT reach season-2 fixtures)");
}
