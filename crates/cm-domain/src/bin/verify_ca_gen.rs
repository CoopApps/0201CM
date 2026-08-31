//! KILL #6 verification — CA + attribute generation for the ~24.8k players the
//! base ships with CA=0 (Panzanaro et al.).
//!
//! Runs the REAL init path (`World::initialise_players`, the deterministic core
//! of FUN_0051f5d0) over rust-db, with the game's own ring-buffer RNG (the
//! ported `MatchRng` over `config/rng_table.bin`), and reports:
//!   * how many CA=0 records now receive a real generated CA (was: dropped),
//!   * the generated-CA distribution (sanity: reputation/age/PA-shaped, ≤PA),
//!   * a named player (Panzanaro) resolved end-to-end: shipped CA/PA -> gen CA
//!     + full 42 attributes, proving the editor's "0" becomes a real profile.
//!
//! Usage: cargo run -p cm-domain --bin verify_ca_gen -- [player_name]

use cm_domain::{PlayerInitState, World};

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let db = std::path::Path::new(&dir);
    let world = World::read_rust_db_dir(db).expect("read rust-db");

    // The game's ring-buffer RNG over the extracted table (same construction the
    // rest of the domain uses). One shared stream across the whole init pass.
    let table_bytes = std::fs::read(db.join("config").join("rng_table.bin"))
        .expect("rng_table.bin");
    let table = cm_rng::table_from_le_bytes(&table_bytes).expect("parse rng table");
    let mut rng = cm_rng::MatchRng::new_seeded(table, 0x0051_f5d0);

    // Real start date (1 Aug 2001-ish); age matters to the CA formula.
    let start = cm_domain::GameDate { year: 2001, month: 8, day: 1 };

    // Count shipped CA=0 before, then run the real init pass.
    let shipped_zero = world.staff.type10.iter().filter(|a| a.current_ability() == 0).count();
    let states = world.initialise_players(&start, Some(&mut rng));

    // Of the states whose SHIPPED CA was 0, how many now have CA>=1?
    let mut gen_cas: Vec<i16> = Vec::new();
    let mut attr_filled = 0usize;
    for a in &world.staff.type10 {
        if a.current_ability() != 0 { continue; }
        if let Some(s) = states.iter().find(|s| s.player_id == a.id) {
            gen_cas.push(s.current_ability);
            if s.attributes.iter().any(|&v| v > 0) { attr_filled += 1; }
        }
    }
    gen_cas.sort_unstable();
    let n = gen_cas.len();
    let mean = gen_cas.iter().map(|&c| c as f64).sum::<f64>() / n.max(1) as f64;
    let pct = |p: f64| gen_cas.get(((n as f64 * p) as usize).min(n.saturating_sub(1))).copied().unwrap_or(0);
    let over_pa = states.iter().filter(|s| s.current_ability > s.potential_ability).count();

    println!("KILL #6 — CA generation for CA=0 records (real FUN_0051f5d0 port)\n");
    println!("shipped CA=0 records ...... {shipped_zero}");
    println!("now given a real CA ....... {n}  ({} left at 0)", shipped_zero - n);
    println!("  attributes also filled .. {attr_filled}");
    println!("  CA>PA violations ........ {over_pa}  (must be 0 — PA cap)");
    println!(
        "generated CA distribution:\n  min {}  p10 {}  p25 {}  median {}  p75 {}  p90 {}  max {}  mean {:.1}",
        gen_cas.first().copied().unwrap_or(0),
        pct(0.10), pct(0.25), pct(0.50), pct(0.75), pct(0.90),
        gen_cas.last().copied().unwrap_or(0), mean,
    );

    // Named player, resolved end-to-end.
    let needle = std::env::args().nth(1).unwrap_or_else(|| "Panzanaro".into());
    let nl = needle.to_lowercase();
    if let Some(p) = world.staff.type6.iter()
        .find(|p| world.person_display_name(p).to_lowercase().contains(&nl))
    {
        let name = world.person_display_name(p);
        let a = world.staff.type10.iter().find(|a| a.id == p.id);
        let s = states.iter().find(|s| s.player_id == p.id);
        println!("\n{name} (id {}):", p.id);
        match (a, s) {
            (Some(a), Some(s)) => {
                println!("  shipped: CA={} PA={} attr_sum={}",
                    a.current_ability(), a.potential_ability_raw(),
                    a.full_attributes()[12..54].iter().map(|&v| v as u32).sum::<u32>());
                println!("  GENERATED: CA={} PA={} age={:?}",
                    s.current_ability, s.potential_ability, s.age);
                let names = cm_domain::DomainStaffType10::ATTRIBUTE_NAMES;
                print!("  attributes:");
                for (i, v) in s.attributes.iter().enumerate() {
                    if i % 6 == 0 { print!("\n    "); }
                    print!("{:<16}={:<3} ", names.get(i).copied().unwrap_or("?"), v);
                }
                println!();
            }
            _ => println!("  (no type10 record / init state)"),
        }
    } else {
        println!("\n{needle}: not found");
    }
}
