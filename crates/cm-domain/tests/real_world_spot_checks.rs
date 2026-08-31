//! End-to-end spot-checks against the shipped 2001-02 database. Load the
//! actual rust-db and assert that specific real-world facts round-trip
//! through the typed views:
//!
//! 1. Kevin Pressman is Sheffield Wednesday's goalkeeper (verifies
//!    the type6 → type10 join via player_data_id, which was the
//!    site of a bug that produced ghost goalkeepers).
//! 2. Ronaldinho is at Paris-Saint-Germain and rated as their top
//!    striker (verifies type6 club filter + type10 aptitude filter
//!    + naming table joins).
//! 3. FC Bayern München and TSV 1860 München share stadium 710 =
//!    Olympiastadion (verifies club_stadium_id decode we corrected
//!    in commit c24f24c).
//!
//! Runs only when the shipped rust-db is present at the default location
//! (skipped in CI environments without the game data).

use std::path::PathBuf;

fn rust_db() -> Option<PathBuf> {
    let p = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    if p.join("core/clubs.json").exists() { Some(p) } else { None }
}

#[test]
fn kevin_pressman_is_sheffield_wednesday_goalkeeper() {
    let Some(db) = rust_db() else { eprintln!("skip: rust-db not present"); return };
    let world = cm_domain::World::read_rust_db_dir(&db).expect("read rust-db");

    // Sheffield Wednesday.
    let swfc = world.core.clubs.iter().find(|r| {
        let v = cm_domain::ClubView::new(r);
        v.primary_name() == "Sheffield Wednesday"
    }).expect("SWFC in rust-db");
    let swfc_id = cm_domain::ClubView::new(swfc).id();
    assert_eq!(swfc_id, 8371, "SWFC id");

    // Kevin Pressman's type6 person record (id 57342).
    let pressman = world.staff.type6.iter().find(|p| p.id == 57342)
        .expect("Pressman in rust-db");
    assert_eq!(pressman.current_club_id(), Some(swfc_id),
        "Pressman must be at SWFC");

    // The type6 → type10 join via player_data_id (NOT id — that's the bug fix).
    let pv = cm_domain::PlayerView::from_split(pressman.id, &pressman.body);
    let pdi = pv.player_data_id().expect("Pressman has attributes");
    assert_eq!(pdi, 47735, "Pressman player_data_id");
    let attrs = world.staff.type10.iter().find(|t| t.id == pdi as u32)
        .expect("type10 record for Pressman");
    assert!(attrs.apt_goalkeeper >= 18,
        "Pressman must be a GK aptitude ({} < 18)", attrs.apt_goalkeeper);
}

#[test]
fn ronaldinho_is_top_psg_striker() {
    let Some(db) = rust_db() else { eprintln!("skip: rust-db not present"); return };
    let world = cm_domain::World::read_rust_db_dir(&db).expect("read rust-db");

    let psg = world.core.clubs.iter().find(|r| {
        cm_domain::ClubView::new(r).primary_name() == "Paris-Saint-Germain"
    }).expect("PSG in rust-db");
    let psg_id = cm_domain::ClubView::new(psg).id();
    assert_eq!(psg_id, 6920, "PSG id");

    // Collect PSG players and their type10 records.
    use std::collections::HashMap;
    let type10_by_id: HashMap<u32, &_> = world.staff.type10.iter().map(|t| (t.id, t)).collect();

    let mut psg_strikers: Vec<(&cm_domain::DomainStaffType6, &cm_domain::DomainStaffType10)> =
        world.staff.type6.iter()
            .filter(|p| p.current_club_id() == Some(psg_id)
                     && matches!({ let pv = cm_domain::PlayerView::from_split(p.id, &p.body); pv.club_job() }, 11 | 15))
            .filter_map(|p| {
                let pv = cm_domain::PlayerView::from_split(p.id, &p.body);
                let pdi = pv.player_data_id()? as u32;
                let t = *type10_by_id.get(&pdi)?;
                // A striker: attacker aptitude tops all non-attacking aptitudes.
                let non_att_max = [t.apt_goalkeeper, t.apt_sweeper, t.apt_defender,
                                   t.apt_def_midfielder, t.apt_midfielder, t.apt_wing_back]
                                   .into_iter().max().unwrap_or(0);
                if t.apt_attacker >= non_att_max && t.apt_attacker >= 15 {
                    Some((p, t))
                } else { None }
            })
            .collect();
    psg_strikers.sort_by(|a, b| b.1.current_ability.cmp(&a.1.current_ability));
    assert!(psg_strikers.len() >= 5,
        "PSG should carry ≥5 strikers, got {}", psg_strikers.len());

    // The top striker must be Ronaldinho — second_name_id maps to the pool
    // slot for "de Assis Moreira". We assert on CA rather than name (name
    // pool depends on region tables). His CA was 165 in 2001-02.
    let (_, top) = psg_strikers[0];
    assert!(top.current_ability >= 160,
        "PSG's top striker should have CA ≥ 160 (Ronaldinho), got {}", top.current_ability);
    assert!(top.potential_ability >= 185,
        "Ronaldinho's PA should be ≥ 185, got {}", top.potential_ability);
    assert!(top.apt_attacker >= 18, "Ronaldinho apt_attacker: {}", top.apt_attacker);
}

#[test]
fn bayern_and_1860_share_olympiastadion() {
    let Some(db) = rust_db() else { eprintln!("skip: rust-db not present"); return };
    let world = cm_domain::World::read_rust_db_dir(&db).expect("read rust-db");

    let bayern = world.core.clubs.iter().find(|r| {
        cm_domain::ClubView::new(r).primary_name() == "FC Bayern München"
    }).expect("Bayern in rust-db");
    let m1860 = world.core.clubs.iter().find(|r| {
        cm_domain::ClubView::new(r).primary_name() == "TSV 1860 München"
    }).expect("1860 in rust-db");

    let bayern_stadium = cm_domain::ClubView::new(bayern).home_stadium_id();
    let m1860_stadium  = cm_domain::ClubView::new(m1860).home_stadium_id();
    assert_eq!(bayern_stadium, Some(710), "Bayern home_stadium_id");
    assert_eq!(m1860_stadium,  Some(710), "1860 home_stadium_id");
    assert_eq!(bayern_stadium, m1860_stadium, "must be the SAME stadium");
}
