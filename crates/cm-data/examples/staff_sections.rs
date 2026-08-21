//! Load the inferred multi-section `staff.dat` split and print sample data.
//! Usage: cargo run -p cm-data --example staff_sections -- D:/cm0102/Data/staff.dat

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102/Data/staff.dat".into());
    let staff = cm_data::load_staff_data(std::path::Path::new(&path)).expect("load staff.dat");

    println!("{}:", path);
    println!("type 6  records {}", staff.type6.len());
    println!("type 9  records {}", staff.type9.len());
    println!("type 10 records {}", staff.type10.len());

    if let Some(entry) = staff.type10.get(1) {
        println!(
            "sample type10 id {} rating_0x05 {} rating_0x07 {} rating_0x0d {} attr0 {} attr30 {}",
            entry.id,
            entry.rating_short_0x05,
            entry.rating_short_0x07,
            entry.rating_short_0x0d,
            entry.attributes[0],
            entry.attributes[30]
        );
    }
}
