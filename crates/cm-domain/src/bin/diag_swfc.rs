//! Diagnostic — show the raw data for the Sheffield Wednesday squad bugs.

use cm_domain::{ClubView, World};
use cm_domain::typed_records::PlayerView;

fn main() {
    let db = std::path::Path::new("D:/cm0102-rs/rust-db");
    let world = World::read_rust_db_dir(db).expect("read rust-db");

    // Find Sheffield Wednesday.
    let sw = world.core.clubs.iter().find(|r| {
        let cv = ClubView::new(r);
        cv.primary_name().to_lowercase().contains("sheffield wednesday")
    }).expect("SWFC");
    let sw_id = ClubView::new(sw).id() as i32;
    println!("SWFC club id: {sw_id}\n");

    println!("=== BUG 1: staff mixed with players ===");
    println!("Sampling type6 records where current_club_id = SWFC AND has a type10 record.");
    println!("The type6 body +0x39 (`job_type` in the editor: 0=player, 5..10=staff roles)\n");
    for name in ["Shreeves", "Yorath", "Baker", "Hodge", "Hinchcliffe", "Pressman", "Tommy Johnson"] {
        let hit = world.staff.type6.iter().find(|p| {
            world.person_display_name(p).to_lowercase().contains(&name.to_lowercase())
                && PlayerView::from_split(p.id, &p.body).current_club_id() == Some(sw_id)
        });
        if let Some(p) = hit {
            let pv = PlayerView::from_split(p.id, &p.body);
            // Read the job-type byte at record +0x3d (body offset 0x39 after 4-byte id prefix)
            let jt = p.body.get(0x39).copied().unwrap_or(0xff);
            let has_t10 = world.staff.type10.iter().any(|t| t.id == p.id);
            let has_t9 = world.staff.type9.iter().any(|t| t.id == p.id);
            println!(
                "  {:<26} id={:<6} jobtype={:>3}  type10={:<5} type9={:<5} club={:?}",
                world.person_display_name(p), p.id, jt, has_t10, has_t9, pv.current_club_id(),
            );
        }
    }

    println!("\n=== BUG 2: position_ordinal reduction ===");
    println!("Real position eligibility bits + engine_position_ordinal → mapped to bucket:");
    let by_id: std::collections::HashMap<u32, &cm_domain::DomainStaffType10> =
        world.staff.type10.iter().map(|t| (t.id, t)).collect();
    for name in ["Pressman", "Tommy Johnson", "Ekoku", "Maddix", "Hinchcliffe"] {
        let hit = world.staff.type6.iter().find(|p|
            world.person_display_name(p).to_lowercase().contains(&name.to_lowercase())
                && PlayerView::from_split(p.id, &p.body).current_club_id() == Some(sw_id));
        if let Some(p) = hit {
            if let Some(t10) = by_id.get(&p.id) {
                let bits = t10.position_eligibility_bits();
                let (ord, is_gk) = t10.engine_position_ordinal();
                let bucket = match ord { 12 => "GK", n if n <= 4 => "DEF", n if n <= 8 => "MID", _ => "ATK" };
                let a = &t10.unknown_bytes_15_26;
                println!(
                    "  {:<20} id={:<6} bits=0x{:03x} ord={:<2} gk={:<5} bucket={:<3}  aptitudes(+0x0f..+0x1a) = {:?}",
                    world.person_display_name(p), p.id, bits, ord, is_gk, bucket, a,
                );
            }
        }
    }

    println!("\n=== BUG 3: SWFC finances ===");
    let cv = ClubView::new(sw);
    let rep = cv.reputation();
    let band = (((rep as i64) / 500).saturating_sub(1)).clamp(0, 15) as usize;
    println!("  reputation           = {rep}");
    println!("  START_CASH band      = {band}");
    println!("  band cash            = £{}", cm_domain::finance::START_CASH[band]);
    println!("  → this is the CTOR path (FUN_005803d0 — new-game seed)");
    println!("  finance.dat exists?  = {}", db.join("references/finance.dat").exists()
        || db.join("finance.dat").exists()
        || db.join("core/finance.dat").exists());
    println!("  → we're NEVER loading Sheffield Wednesday's REAL stored balance.");
    println!("    Real CM01/02: SWFC was newly relegated with massive debt.");
    // Also list a few common finance-record locations:
    for p in ["finance.dat", "core/finance.dat", "references/finance.dat"] {
        println!("    D:/cm0102/data/{p} exists? {}", std::path::Path::new(&format!("D:/cm0102/data/{p}")).exists());
    }
}
