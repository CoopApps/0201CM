//! Decode-check: run the typed views over the REAL shipped database and print
//! samples. This is how we prove a decoded field map against actual data
//! rather than synthetic test records.
//!
//! Usage: cargo run -p cm-import --bin decode-check -- [rust_db_dir]

use cm_domain::typed_records::{
    ClubView, ColourView, CompetitionView, ContinentView, NationView, PlayerView,
};

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102-rs/rust-db".to_string());
    let world = cm_domain::World::read_rust_db_dir(std::path::Path::new(&dir))
        .expect("read rust-db");

    println!("== colours ({} records) ==", world.core.colours.len());
    for record in world.core.colours.iter().take(6) {
        let v = ColourView::new(record);
        let (r, g, b) = v.rgb();
        println!("  id={:<3} {:<20} rgb=({r:3},{g:3},{b:3})", v.id(), v.name());
    }

    println!("\n== continents ({} records) ==", world.core.continents.len());
    for record in &world.core.continents {
        let v = ContinentView::new(record);
        println!(
            "  id={} {:<16} {:<4} {:<16} {:<10} coeff={:.2}",
            v.id(),
            v.name(),
            v.code(),
            v.adjective(),
            v.confederation_acronym(),
            v.strength_coefficient()
        );
    }

    println!("\n== nations (first 5 of {}) ==", world.core.nations.len());
    for record in world.core.nations.iter().take(5) {
        let v = NationView::new(record);
        println!("  id={:<4} {:<24} {}", v.id(), v.primary_name(), v.secondary_name());
    }

    println!("\n== clubs (first 5 of {}) ==", world.core.clubs.len());
    for record in world.core.clubs.iter().take(5) {
        let v = ClubView::new(record);
        println!(
            "  id={:<6} {:<28} nation={:?} rep={}",
            v.id(),
            v.primary_name(),
            v.nation_id(),
            v.reputation()
        );
    }

    println!(
        "\n== club competitions (first 6 of {}) ==",
        world.references.club_competitions.len()
    );
    for competition in world.references.club_competitions.iter().take(6) {
        println!(
            "  id={:<5} {:<34} {:<10} rep={}",
            competition.id,
            competition.long_name,
            competition.short_name,
            // reputation lives in the tail at record offset 0x69
            competition
                .unknown_tail
                .get(0x69 - 0x52..0x69 - 0x52 + 2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]) as i64)
                .unwrap_or(-1),
        );
    }
    let _ = CompetitionView::RECORD_SIZE;

    // rust-db stores staff records split: `id` as its own field, `body`
    // starting at record offset 0x04. Use the split constructor.
    println!("\n== staff type6 (first 5 of {}) ==", world.staff.type6.len());
    for entry in world.staff.type6.iter().take(5) {
        let v = PlayerView::from_split(entry.id, &entry.body);
        println!(
            "  id={:<7} names=({:?},{:?}) dob=day{}/{} nation={:?} club={:?} player_data={:?} v1={}",
            v.staff_id(),
            v.first_name_id(),
            v.second_name_id(),
            v.date_of_birth().day,
            v.date_of_birth().year,
            v.nation_id(),
            v.current_club_id(),
            v.player_data_id(),
            v.is_disk_v1(),
        );
    }
    // Whole-pool sweep: the year-of-birth duplicate field must agree with the
    // year inside the date struct — that agreement is what proves the offsets.
    let mut players = 0usize;
    let mut dob_agrees = 0usize;
    let mut checked = 0usize;
    let mut with_club = 0usize;
    let mut with_nation = 0usize;
    let mut min_year = u16::MAX;
    let mut max_year = 0u16;
    for entry in &world.staff.type6 {
        let v = PlayerView::from_split(entry.id, &entry.body);
        if v.is_player() {
            players += 1;
        }
        if v.current_club_id().is_some() {
            with_club += 1;
        }
        if v.nation_id().is_some() {
            with_nation += 1;
        }
        let year = v.date_of_birth().year;
        if year != 0 {
            checked += 1;
            if year == v.secondary_year_field() {
                dob_agrees += 1;
            }
            min_year = min_year.min(year);
            max_year = max_year.max(year);
        }
    }
    println!("  full sweep of {} records:", world.staff.type6.len());
    println!("    with player-attribute link : {players}");
    println!("    with club                  : {with_club}");
    println!("    with nation                : {with_nation}");
    println!("    birth years                : {min_year}..={max_year}");
    println!(
        "    +0x18 == dob.year  : {dob_agrees}/{checked} ({:.1}%)",
        if checked > 0 {
            dob_agrees as f64 * 100.0 / checked as f64
        } else {
            0.0
        }
    );

    println!("\n== record body sizes ==");
    if let Some(first) = world.staff.type6.first() {
        println!("  staff type6 body = {} bytes", first.body.len());
    }
    if let Some(first) = world.staff.type9.first() {
        println!("  staff type9 body = {} bytes", first.body.len());
    }
    if let Some(first) = world.core.clubs.first() {
        println!("  club raw        = {} bytes", first.raw.len());
    }
    if let Some(first) = world.core.colours.first() {
        println!("  colour raw      = {} bytes", first.raw.len());
    }
    if let Some(first) = world.core.continents.first() {
        println!("  continent raw   = {} bytes", first.raw.len());
    }
}
