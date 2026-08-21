//! Parse a real `.sav` file and print its section directory.
//! Usage: cargo run -p cm-data --example save_sections -- D:/cm0102/save1.sav

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102/save1.sav".into());
    let bytes = std::fs::read(&path).expect("read save");
    let save = cm_data::SaveFile::parse(&bytes).expect("parse save");

    println!(
        "{}: version {} with {} sections\n{:<24} {:>10} {:>10}",
        path,
        save.version,
        save.sections.len(),
        "section",
        "size",
        "field_a"
    );
    for section in &save.sections {
        println!(
            "{:<24} {:>10} {:>10}",
            section.name, section.size, section.unknown_a
        );
    }
}
