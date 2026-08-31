//! Import every `D:/cm0102/History/*.his` file into a single JSON pool at
//! `rust-db/references/written_history.json`. Each row = one .his file, with
//! the header (category / title / section / pic) and narrative body as named
//! JSON fields.
//!
//! Parser is inlined here (not `cm_domain::history::parse_his`) to keep this
//! binary buildable without dragging the whole cm-domain crate through
//! rustc — that crate is currently too big to fit in this host's paging
//! space with `codegen-units=1`.
//!
//! Usage: cargo run -p cm-import --bin regen_written_history

use std::path::PathBuf;
use std::io::Write;
use serde_json::json;

#[derive(Default, Clone)]
struct HistoryFile {
    category: String,      // canonical: "Clubs"/"Nations"/"Competitions"/"Leagues"/"Players"
    category_raw: String,  // as-in-file: "CLUB"/"PLAYER"/"Nation"/…
    title: String,
    section: String,
    pic: String,
    body: String,
}

fn category_canon(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "club"        | "clubs"        | "0" => "Clubs",
        "nation"      | "nations"      | "1" => "Nations",
        "competition" | "competitions" | "2" => "Competitions",
        "league"      | "leagues"      | "3" => "Leagues",
        "player"      | "players"      | "4" => "Players",
        _ => "",
    }.to_string()
}

/// Parses both formats: CM01/02 `{KEY: VALUE}` header + narrative body, and
/// legacy CM3 `keyword { body }` blocks (falls back if the file doesn't
/// begin with `{`). Only the CM01/02 branch is exercised by shipped data.
fn parse_his(text: &str) -> HistoryFile {
    let mut out = HistoryFile::default();
    let stripped = text.trim_start();
    if !stripped.starts_with('{') {
        // Legacy CM3 format not present in shipped data — leave fields empty.
        return out;
    }
    let mut cursor = 0usize;
    for line in text.split_inclusive('\n') {
        let this_start = cursor;
        cursor += line.len();
        let l = line.trim_matches(|c: char| c == '\r' || c.is_whitespace());
        if l.is_empty() { continue; }
        if l.starts_with('{') && l.ends_with('}') {
            let inner = &l[1..l.len()-1];
            if let Some((k, v)) = inner.split_once(':') {
                let key = k.trim().to_ascii_lowercase();
                let val = v.trim().to_string();
                match key.as_str() {
                    "category" => {
                        out.category_raw = val.clone();
                        out.category = category_canon(&val);
                    }
                    "title"   => out.title = val,
                    "section" => out.section = val,
                    "pic"     => out.pic = val,
                    _ => {}
                }
            }
            continue;
        }
        // First non-header, non-blank line ⇒ start of the narrative body.
        out.body = text[this_start..]
            .trim_end_matches('\0')
            .trim_end()
            .to_string();
        break;
    }
    out
}

fn main() {
    let src = PathBuf::from(std::env::var("CM_HISTORY_DIR")
        .unwrap_or_else(|_| "D:/cm0102/History".into()));
    let db  = PathBuf::from(std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    let out = db.join("references/written_history.json");
    println!("[regen_written_history] src: {}", src.display());
    println!("[regen_written_history] out: {}", out.display());

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&src).expect("read history dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str())
                     .map(|s| s.eq_ignore_ascii_case("his")).unwrap_or(false))
        .collect();
    entries.sort();
    println!("[regen_written_history] found {} .his files", entries.len());

    let file = std::fs::File::create(&out).expect("open");
    let mut w = std::io::BufWriter::with_capacity(4 * 1024 * 1024, file);
    w.write_all(b"[").unwrap();

    let mut counts = std::collections::BTreeMap::<String, u32>::new();
    let mut spot_billy: Option<serde_json::Value> = None;
    for (i, path) in entries.iter().enumerate() {
        // Latin-1 decode (some player names use accented characters).
        let bytes = std::fs::read(path).expect("read .his");
        let text: String = bytes.iter().map(|&b| b as char).collect();
        let hf = parse_his(&text);
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let key = if hf.category.is_empty() { hf.category_raw.clone() } else { hf.category.clone() };
        *counts.entry(key).or_default() += 1;
        let mut m = serde_json::Map::new();
        m.insert("filename".into(),     json!(filename));
        m.insert("category".into(),     json!(hf.category));
        m.insert("category_raw".into(), json!(hf.category_raw));
        m.insert("title".into(),        json!(hf.title));
        m.insert("section".into(),      json!(hf.section));
        m.insert("pic".into(),          json!(hf.pic));
        m.insert("body".into(),         json!(hf.body));
        if i > 0 { w.write_all(b",").unwrap(); }
        serde_json::to_writer(&mut w, &m).unwrap();
        if path.file_name().and_then(|s| s.to_str()) == Some("Billy Bingham.his") {
            spot_billy = Some(serde_json::Value::Object(m));
        }
    }
    w.write_all(b"]").unwrap();
    drop(w);
    println!("[regen_written_history] wrote {} records ({} bytes)",
             entries.len(), std::fs::metadata(&out).unwrap().len());
    println!("\nCategory distribution:");
    for (k, v) in &counts { println!("  {:>16}  {}", k, v); }

    if let Some(v) = spot_billy {
        println!("\n=== SPOT CHECK: Billy Bingham.his ===");
        println!("  category:     {:?}", v.get("category"));
        println!("  category_raw: {:?}", v.get("category_raw"));
        println!("  title:        {:?}", v.get("title"));
        println!("  section:      {:?}", v.get("section"));
        println!("  pic:          {:?}", v.get("pic"));
        let body = v.get("body").and_then(|s| s.as_str()).unwrap_or("");
        let preview: String = body.chars().take(140).collect();
        println!("  body first 140: {:?}", preview);
    }
}
