//! Prints how many `club.dat` records have their primary competition field
//! (`club+0x57`, `ClubView::division_id`) equal to each given comp id.
//! Usage: `cargo run -p cm-import --bin comp-counts -- <comp_id> [<comp_id> ...]`
//! Used to verify/derive `TOP_DIVISION_TEAMS`-style constants against the
//! DECODED club->competition binding instead of a real-world-season guess.
use std::path::Path;

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let world = cm_domain::World::read_rust_db_dir(Path::new(&dir)).expect("db");
    let ids: Vec<i64> = std::env::args()
        .skip(1)
        .map(|s| s.parse().unwrap())
        .collect();
    for id in ids {
        let n = world.club_members_of_competition(id as u32).len();
        println!("comp {id}: {n} clubs");
    }
}
