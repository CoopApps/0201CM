use std::path::Path;

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn opt_i32(v: Option<i32>) -> String {
    match v {
        Some(x) => x.to_string(),
        None => "null".to_string(),
    }
}

fn main() {
    let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
    let world = cm_domain::World::read_rust_db_dir(Path::new(&dir)).expect("db");
    let mut lines: Vec<String> = Vec::new();
    for record in &world.core.clubs {
        let view = cm_domain::typed_records::ClubView::new(record);
        let id = view.id();
        if id == 0 {
            continue;
        }
        lines.push(format!(
            "  {{\"club_id\": {}, \"club_name\": \"{}\", \"division_id\": {}, \"secondary_comp_id\": {}, \"tertiary_comp_id\": {}}}",
            id,
            json_escape(&view.primary_name()),
            opt_i32(view.division_id()),
            opt_i32(view.secondary_comp_id()),
            opt_i32(view.tertiary_comp_id()),
        ));
    }
    let json = format!("[\n{}\n]\n", lines.join(",\n"));
    let path = Path::new(&dir).join("references/club_current_competition.json");
    std::fs::write(&path, &json).unwrap();
    println!("wrote {} club rows to {}", lines.len(), path.display());
}
