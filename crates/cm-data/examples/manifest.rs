//! Parse a real `index.dat` and print its type→file→count map.
//! Usage: cargo run -p cm-data --example manifest -- D:/cm0102/Data/index.dat

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102/Data/index.dat".into());
    let bytes = std::fs::read(&path).expect("read index.dat");
    let m = cm_data::Manifest::parse(&bytes);
    println!(
        "{}: {} entries\n{:<24} {:>4} {:>10}",
        path,
        m.entries.len(),
        "file",
        "type",
        "count"
    );
    for e in &m.entries {
        println!("{:<24} {:>4} {:>10}", e.filename, e.kind, e.count);
    }
    // spot-check against the carve's findings
    if let Some(club) = m.by_kind(0) {
        println!("\ntype 0 -> {} (expect club.dat, 10580)", club.filename);
    }
    if let Some(attrs) = m.by_kind(10) {
        println!(
            "type 10 -> {} count {} (the 70-byte attribute section)",
            attrs.filename, attrs.count
        );
    }
}
