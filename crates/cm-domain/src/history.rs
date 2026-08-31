//! History files — port of `history.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\history.cpp`).
//!
//! Contrary to what the name suggests, `cm3_history/*.his` files are
//! **shipped, pre-authored content**, not simulation output. Nothing in the
//! entire cm0102.exe decompile ever opens these files for write — every
//! reference is a READER (five accessors in this TU), the shipped
//! [`history_screen_draw`] callback (FUN_005dc660), the file-picker screen
//! FUN_005dab70, the menu-gate probe at FUN_00745540, or a format helper.
//! Verified by exhaustive grep of `cm3_history` / `CM3_HISTORY` across the
//! full decompile: 8 files, 0 writers.
//!
//! Each `.his` is a text file with a curly-braced sectioned syntax:
//!
//! ```text
//! category { <int> }         # 0=Clubs 1=Nations 2=Competitions 3=Leagues 4=Players
//! title { <display title> }
//! section { <name>
//!   <row0>
//!   <row1>
//!   ...
//! }
//! section { ... }
//! ```
//!
//! Reader recognises three keywords: `title`, `section`, and one more at
//! `DAT_009bb138` (probably `subtitle`). This module ports the parser +
//! in-memory model + category enum.

use std::path::Path;

/// The five History categories exposed by the tab strip in
/// [`history_screen_draw`] (FUN_005dc660): the stack locals
/// `uStack_a0c=0` (Clubs), `uStack_c88=1` (Nations), `uStack_bb4=2`
/// (Competitions), `uStack_ae0=3` (Leagues), `uStack_938=4` (Players).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryCategory {
    Clubs        = 0,
    Nations      = 1,
    Competitions = 2,
    Leagues      = 3,
    Players      = 4,
}

impl HistoryCategory {
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0 => Self::Clubs, 1 => Self::Nations, 2 => Self::Competitions,
            3 => Self::Leagues, 4 => Self::Players,
            _ => return None,
        })
    }
    pub fn label(self) -> &'static str {
        // Order in the tab strip: {Nations, Competitions, Leagues, Clubs, Players}
        // — see FUN_005dc660 lines 88..102 for the string bindings.
        match self {
            Self::Nations      => "Nations",
            Self::Competitions => "Competitions",
            Self::Leagues      => "Leagues",
            Self::Clubs        => "Clubs",
            Self::Players      => "Players",
        }
    }
}

/// One `section { <name> <rows...> }` block (legacy CM3 brace-block format).
#[derive(Debug, Clone, Default)]
pub struct HistorySection {
    pub name: String,
    pub rows: Vec<String>,
}

/// A parsed `.his` file.
#[derive(Debug, Clone, Default)]
pub struct HistoryFile {
    /// From `{CATEGORY: <string>}` (CM01/02) or `category { <int> }` (legacy CM3).
    pub category: Option<HistoryCategory>,
    /// Original category string as it appears in the file
    /// ("CLUB", "PLAYER", "Nation", "Competition", "League").
    pub category_raw: String,
    /// From `{Title: <display>}` / `{TITLE: <display>}`.
    pub title: String,
    /// From `{Section: <string>}` — the single section name in the CM01/02 format.
    pub section: String,
    /// From `{pic: <filename.hsr>}` — associated bitmap.
    pub pic: String,
    /// Trailing narrative body (paragraphs after the header block).
    /// Preserves original line breaks / tab indents.
    pub body: String,
    /// Ordered list of `section { ... }` blocks (only populated for
    /// legacy CM3 brace-block format; CM01/02 `.his` files leave this empty).
    pub sections: Vec<HistorySection>,
}

/// Categorise a raw string ("CLUB", "PLAYER", "Nation", …) to the enum.
fn category_from_raw(raw: &str) -> Option<HistoryCategory> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "club"        | "clubs"        | "0" => Some(HistoryCategory::Clubs),
        "nation"      | "nations"      | "1" => Some(HistoryCategory::Nations),
        "competition" | "competitions" | "2" => Some(HistoryCategory::Competitions),
        "league"      | "leagues"      | "3" => Some(HistoryCategory::Leagues),
        "player"      | "players"      | "4" => Some(HistoryCategory::Players),
        _ => None,
    }
}

/// Parse the text of a `.his` file.
///
/// Handles both the shipped CM01/02 format —
///   ```text
///   {CATEGORY: CLUB}
///   {Title: Arsenal}
///   {Section: Arsenal}
///   {pic: cs_Arsenal1.hsr}
///
///       Formed in 1886, the London club became…
///   ```
/// and the legacy CM3 brace-block format —
///   ```text
///   category { 0 }
///   title { Arsenal }
///   section { Honours
///   1930-31 Champions
///   }
///   ```
///
/// Detection: if the first non-whitespace byte is `{`, it's the CM01/02
/// header-per-line form (and everything after the last `}\n` on a header
/// line is the narrative body). Otherwise fall back to the CM3 parser.
pub fn parse_his(text: &str) -> HistoryFile {
    let stripped = text.trim_start();
    if stripped.starts_with('{') {
        return parse_his_cm0102(text);
    }
    parse_his_cm3(text)
}

/// CM01/02 `.his`: `{KEY: VALUE}` per line + narrative body.
fn parse_his_cm0102(text: &str) -> HistoryFile {
    let mut out = HistoryFile::default();
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
                        out.category = category_from_raw(&val);
                        out.category_raw = val;
                    }
                    "title"   => out.title   = val,
                    "section" => out.section = val,
                    "pic"     => out.pic     = val,
                    _         => {}
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

/// Legacy CM3 `.his`: `keyword { body }` blocks. Ported from history.cpp
/// (FUN_005db8b0's keyword-driven state machine).
fn parse_his_cm3(text: &str) -> HistoryFile {
    let mut out = HistoryFile::default();
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Skip whitespace.
        while i < bytes.len() && (bytes[i] as char).is_whitespace() { i += 1; }
        if i >= bytes.len() { break; }

        // Read a keyword up to `{` or whitespace.
        let kw_start = i;
        while i < bytes.len() && bytes[i] != b'{' && !(bytes[i] as char).is_whitespace() {
            i += 1;
        }
        let keyword = std::str::from_utf8(&bytes[kw_start..i]).unwrap_or("").to_ascii_lowercase();

        // Skip whitespace to `{`.
        while i < bytes.len() && (bytes[i] as char).is_whitespace() { i += 1; }
        if i >= bytes.len() || bytes[i] != b'{' { continue; }
        i += 1; // past '{'

        // Capture the block body up to matching '}'.
        let body_start = i;
        while i < bytes.len() && bytes[i] != b'}' { i += 1; }
        let body = std::str::from_utf8(&bytes[body_start..i]).unwrap_or("");
        if i < bytes.len() { i += 1; } // past '}'

        match keyword.as_str() {
            "title" => {
                out.title = body.trim().to_string();
            }
            "category" => {
                let raw = body.trim();
                out.category = category_from_raw(raw);
                out.category_raw = raw.to_string();
            }
            "section" => {
                let mut lines = body.lines();
                let name = lines.next().map(|s| s.trim().to_string()).unwrap_or_default();
                let rows: Vec<String> = lines
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                out.sections.push(HistorySection { name, rows });
            }
            _ => { /* unknown key — body is already consumed. */ }
        }
    }
    out
}

/// Enumerate the shipped `cm3_history/*.his` files, parsing enough of each to
/// pull its `category` and `title` for the History screen's list — matches
/// [`FUN_005dc2a0`] (verify_all) and the enumeration in
/// [`FUN_005dc660`] (screen draw), plus the field-only fast paths
/// [`FUN_005dbfe0`] (read category) and [`FUN_005dc160`] (read title).
pub fn scan_dir(dir: &Path) -> std::io::Result<Vec<(std::path::PathBuf, HistoryFile)>> {
    let mut out = Vec::new();
    if !dir.is_dir() { return Ok(out); }
    for ent in std::fs::read_dir(dir)? {
        let ent = ent?;
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("his")) != Some(true) {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let hf = parse_his(&text);
        out.push((path, hf));
    }
    // The exe sorts the list at 005dcb44 (FUN_009343c3 quicksort with a
    // strcmp-like comparator LAB_005dcea0). Alphabetical by title.
    out.sort_by(|a, b| a.1.title.to_ascii_lowercase().cmp(&b.1.title.to_ascii_lowercase()));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_shipped_style_his() {
        let src = "\
category { 3 }
title { English League History }
section { Champions
1888-89 Preston North End
1889-90 Preston North End
1890-91 Everton
}
section { Top Scorers
1888-89 John Goodall (Preston)
}
";
        let hf = parse_his(src);
        assert_eq!(hf.category, Some(HistoryCategory::Leagues));
        assert_eq!(hf.title, "English League History");
        assert_eq!(hf.sections.len(), 2);
        assert_eq!(hf.sections[0].name, "Champions");
        assert_eq!(hf.sections[0].rows.len(), 3);
        assert_eq!(hf.sections[1].name, "Top Scorers");
    }

    #[test]
    fn unknown_keyword_is_skipped_not_fatal() {
        let src = "garbage { anything at all }\ntitle { Test }\n";
        let hf = parse_his(src);
        assert_eq!(hf.title, "Test");
    }
}
