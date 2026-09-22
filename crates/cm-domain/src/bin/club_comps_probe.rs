//! Check whether a club's division/secondary/tertiary comp slots reproduce the
//! captured game-start View menu (Chester: Conference / FA Trophy / Vans Trophy).
//!
//! Run: cargo run -q -p cm-domain --bin club_comps_probe

use std::path::Path;

use cm_domain::{typed_records::ClubView, World};

fn main() {
    let world = World::read_rust_db_dir(Path::new("D:/cm0102-rs/rust-db")).expect("read rust-db");

    let comp_name = |cid: i32| -> String {
        world
            .references
            .club_competitions
            .iter()
            .chain(world.references.staff_competitions.iter())
            .chain(world.references.nation_competitions.iter())
            .find(|c| c.id as i32 == cid)
            .map(|c| {
                if c.short_name.trim().is_empty() {
                    c.long_name.clone()
                } else {
                    c.short_name.clone()
                }
            })
            .unwrap_or_else(|| format!("comp {cid}"))
    };

    for want in ["Chester", "Liverpool"] {
        let club = world
            .core
            .clubs
            .iter()
            .map(|c| ClubView::new(c))
            .find(|v| v.primary_name().contains(want) && !v.primary_name().contains("field"));
        let Some(v) = club else {
            println!("{want}: not found");
            continue;
        };
        println!("{} (id {}):", v.primary_name(), v.id());
        println!("  division_id   = {:?} -> {:?}", v.division_id(), v.division_id().map(comp_name));
        println!("  secondary_comp= {:?} -> {:?}", v.secondary_comp_id(), v.secondary_comp_id().map(comp_name));
        println!("  tertiary_comp = {:?} -> {:?}", v.tertiary_comp_id(), v.tertiary_comp_id().map(comp_name));
        let names: Vec<String> = v.competition_ids().map(comp_name).collect();
        println!("  => menu comps: {names:?}");
    }
}
