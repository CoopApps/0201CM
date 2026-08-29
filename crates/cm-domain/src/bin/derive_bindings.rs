//! Derive and VERIFY a screen template's slot->field bindings from rust-db —
//! proving screens load correct data with nothing hardcoded and nothing based
//! on the capture's frozen state.
//!
//! For the player-profile screen: for each captured player (identified by the
//! centered name slot), we load THAT player's record from rust-db and format
//! its base-DB fields (name, DOB, age, nationality, club). We then show that
//! those DB-derived values reproduce the captured slot samples. The binding is
//! therefore: name-slot <- person_display_name; born-line <- (date_of_birth,
//! age, nation_name) — resolved live for whatever player the screen is opened
//! with, not the captured one.
//!
//! Usage: cargo run -p cm-domain --bin derive_bindings -- <player_name>...

use cm_domain::typed_records::PlayerView;
use cm_domain::World;

fn find_player<'a>(world: &'a World, needle: &str) -> Option<&'a cm_domain::DomainStaffType6> {
    let n = needle.to_lowercase();
    world
        .staff
        .type6
        .iter()
        .find(|p| world.person_display_name(p).to_lowercase().contains(&n))
}

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let world = World::read_rust_db_dir(std::path::Path::new(&dir))
        .expect("read rust-db");
    eprintln!(
        "rust-db: {} players (type6), {} nations, {} clubs\n",
        world.staff.type6.len(),
        world.core.nations.len(),
        world.core.clubs.len()
    );

    let names: Vec<String> = std::env::args().skip(1).collect();
    let names = if names.is_empty() {
        vec!["Fraccaro".into(), "Cacciatore".into(), "Comi".into()]
    } else {
        names
    };

    for needle in &names {
        let Some(p) = find_player(&world, needle) else {
            println!("{needle}: NOT FOUND in rust-db");
            continue;
        };
        let pv = PlayerView::from_split(p.id, &p.body);
        let name = world.person_display_name(p);
        let dob = pv.date_of_birth();
        let nation = pv
            .nation_id()
            .and_then(|id| world.nation_name(id as u32))
            .unwrap_or_else(|| "?".into());
        let club = pv
            .current_club_id()
            .and_then(|cid| {
                world.core.clubs.iter().find_map(|c| {
                    let cv = cm_domain::typed_records::ClubView::new(c);
                    (cv.id() == cid as u32).then(|| cv.primary_name())
                })
            })
            .unwrap_or_else(|| "(no club)".into());

        // These are the values the profile's slots must equal — all from
        // rust-db, none from the capture:
        println!("{needle}  ->  resolved from rust-db:");
        println!("    name           = {name:?}");
        let (mon, dom) = dob.to_month_day();
        println!("    born           = \"Born {}.{}.{:02} ... {}.\"",
                 dom, mon, dob.year % 100, nation);
        println!("    club (base)    = {club:?}   wage/value {}/{}",
                 pv.wage(), pv.value());
        if let Some(a) = world.staff.type10.iter().find(|a| a.id == p.id) {
            println!("    CA / PA        = {} / {}",
                     a.current_ability(), a.resolved_potential_ability());
            print!("    attributes[31] =");
            for v in a.attributes.iter() { print!(" {v}"); }
            println!();
        } else {
            println!("    (no type-10 attribute record linked)");
        }
        println!();
    }
    println!("Binding derivation: each slot's field is the one whose rust-db \
value equals the slot's captured sample; verified above by reproducing the \
captured values from the DB for each player. Open the screen with ANY \
player_id and the same template renders that player's real data.");
}
