//! Scenario-golden test — walks every `fixtures/*.json` at the repo root
//! and asserts each fixture replays byte-exact through the packed
//! renderer. This is the byte-exact contract of milestone 6.
//!
//! Empty `fixtures/` is not a failure: the test reports "no fixtures
//! found" and passes. Adding a fixture (from the Frida capture harness
//! in `tools/gdi_capture/`) automatically enrols it in the suite.

use std::path::{Path, PathBuf};

use cm_render::packed_capture::{replay, Fixture};

fn fixtures_dir() -> PathBuf {
    // Repo layout: cm-render/tests -> ../.. = repo root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("../../fixtures"))
}

fn list_fixtures(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
        .collect();
    out.sort();
    out
}

#[test]
fn every_fixture_replays_byte_exact() {
    let dir = fixtures_dir();
    let paths = list_fixtures(&dir);
    if paths.is_empty() {
        eprintln!("no fixtures found in {} — skipping (add captures via tools/gdi_capture)", dir.display());
        return;
    }
    let mut failures = Vec::new();
    for path in &paths {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("{}: read error: {e}", path.display()));
                continue;
            }
        };
        let fixture: Fixture = match serde_json::from_str(&text) {
            Ok(f) => f,
            Err(e) => {
                failures.push(format!("{}: parse error: {e}", path.display()));
                continue;
            }
        };
        match replay(&fixture) {
            Ok(_) => {
                eprintln!(
                    "OK  {} ({} calls, {}x{})",
                    path.file_name().unwrap().to_string_lossy(),
                    fixture.calls.len(),
                    fixture.meta.width,
                    fixture.meta.height,
                );
            }
            Err(mismatch) => {
                failures.push(format!("{}: {mismatch}", path.display()));
            }
        }
    }
    if !failures.is_empty() {
        panic!(
            "{} of {} fixture(s) failed byte-exact replay:\n  {}",
            failures.len(),
            paths.len(),
            failures.join("\n  ")
        );
    }
}
