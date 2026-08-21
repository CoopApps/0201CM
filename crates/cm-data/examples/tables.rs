//! Load the full logical table catalog from a CM0102 `Data` directory.
//! Usage: cargo run -p cm-data --example tables -- D:/cm0102/Data

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/cm0102/Data".into());
    let data_dir = std::path::Path::new(&path);
    let manifest = cm_data::Manifest::parse(
        &std::fs::read(data_dir.join("index.dat")).expect("read index.dat"),
    );
    let tables = cm_data::DataTables::load_from_install(data_dir, &manifest).expect("load tables");

    println!(
        "{}: {} logical tables\n{:<20} {:>4} {:<24} {:>10} {:>12} {:>12}",
        path,
        tables.tables.len(),
        "logical",
        "type",
        "file",
        "layout",
        "bytes",
        "records"
    );
    for table in &tables.tables {
        let record_info = table
            .fixed_record_count()
            .map(|count| count.to_string())
            .unwrap_or_else(|| "-".into());
        let layout_info = match table.fixed_record_layout() {
            Some(layout) => match layout.confidence {
                cm_data::RecordLayoutConfidence::Verified => format!("v{}", layout.size),
                cm_data::RecordLayoutConfidence::Inferred => format!("i{}", layout.size),
            },
            None => "raw".into(),
        };
        println!(
            "{:<20} {:>4} {:<24} {:>10} {:>12} {:>12}",
            table.spec.logical_name,
            table.spec.manifest_type,
            table.spec.filename,
            layout_info,
            table.byte_len,
            record_info
        );
    }
}
