//! Population gate — the fence.
//!
//! Walks every `analysis/screens/*.json` under the carve directory and fails
//! the whole `cargo test` if ANY screen is unsigned or has stale sha. Result:
//! `cargo build` (via `cargo test`) is red until every screen is signed and
//! its contract matches the current spec + helpers.json.
//!
//! What this catches that per-load gating misses:
//!   - New screens added by re-running the carver: they arrive unsigned, red
//!     test immediately surfaces them.
//!   - Helper-table edits (e.g. flipping `aux_b` from ported=true to false)
//!     rehash every dependent contract via the combined sha → mass-invalidate
//!     the previously-signed population → red test tells you what needs
//!     re-verifying.
//!   - "I forgot to run the audit tool" — you can't forget, the build tells
//!     you before you ship.
//!
//! Skip mechanism: `CM_SKIP_POPULATION_GATE=1` — INTENDED ONLY for the case
//! where the carve directory isn't checked out (e.g. a downstream consumer).
//! Anyone in this repo who sets it is cheating themselves.

use std::path::{Path, PathBuf};

fn carve_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("CM_SCREENS_DIR")
            .unwrap_or_else(|_| "D:/cm0102-carve/analysis/screens".into()),
    )
}

fn sha256(bytes: &[u8]) -> String {
    // Mirror of the inline SHA in cm-widget — kept independent so the test
    // catches divergence between test and runtime hashers.
    use std::io::Write;
    let mut cmd = std::process::Command::new("python");
    cmd.arg("-c").arg(
        r#"import sys, hashlib; sys.stdout.write(hashlib.sha256(sys.stdin.buffer.read()).hexdigest())"#,
    );
    cmd.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped());
    let mut child = cmd.spawn().expect("python available");
    child.stdin.as_mut().unwrap().write_all(bytes).unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8(out.stdout).unwrap()
}

fn combined_sha(spec_path: &Path, helpers_bytes: &[u8]) -> String {
    let mut combined = std::fs::read(spec_path).expect("read spec");
    combined.extend_from_slice(b"||");
    combined.extend_from_slice(helpers_bytes);
    sha256(&combined)
}

#[test]
fn every_screen_has_a_signed_contract_that_matches_current_spec_and_helpers() {
    if std::env::var("CM_SKIP_POPULATION_GATE").ok().as_deref() == Some("1") {
        eprintln!("[population_gate] SKIPPED via CM_SKIP_POPULATION_GATE=1");
        return;
    }
    let dir = carve_dir();
    if !dir.exists() {
        panic!(
            "carve directory missing: {} — set CM_SCREENS_DIR or CM_SKIP_POPULATION_GATE=1",
            dir.display()
        );
    }
    let helpers_path = dir.parent().unwrap().join("helpers.json");
    let helpers_bytes = std::fs::read(&helpers_path)
        .unwrap_or_else(|_| panic!("helpers.json missing at {}", helpers_path.display()));

    let mut screens: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .filter(|p| {
            // Contract files live next to their specs; don't audit them as specs.
            !p.file_name().and_then(|s| s.to_str()).unwrap_or("").ends_with(".contract.json")
        })
        .collect();
    screens.sort();

    let mut unsigned: Vec<String> = Vec::new();
    let mut mismatched: Vec<String> = Vec::new();

    for spec in &screens {
        let stem = spec.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let contract = spec.with_extension("contract.json");
        if !contract.exists() {
            unsigned.push(stem);
            continue;
        }
        let bytes = std::fs::read(&contract).unwrap();
        let v: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => {
                mismatched.push(format!("{stem}: contract unreadable: {e}"));
                continue;
            }
        };
        let recorded = v.get("spec_sha").and_then(|s| s.as_str()).unwrap_or("");
        let expected = combined_sha(spec, &helpers_bytes);
        if recorded != expected {
            mismatched.push(format!(
                "{stem}: sha mismatch — spec or helpers.json edited since sign-off"
            ));
        }
    }

    let total = screens.len();
    let signed_and_valid = total - unsigned.len() - mismatched.len();
    eprintln!(
        "[population_gate] {}/{} screens signed & fresh; unsigned={} mismatched={}",
        signed_and_valid, total, unsigned.len(), mismatched.len()
    );

    if !unsigned.is_empty() || !mismatched.is_empty() {
        let mut msg = String::new();
        msg.push_str(&format!(
            "\nPOPULATION GATE HELD — {}/{} screens are not shippable.\n\n",
            unsigned.len() + mismatched.len(),
            total
        ));
        if !unsigned.is_empty() {
            msg.push_str(&format!("UNSIGNED ({}) — run the audit and sign:\n", unsigned.len()));
            for va in unsigned.iter().take(20) {
                let short = va.trim_start_matches('0');
                msg.push_str(&format!(
                    "  python D:/cm0102-carve/tools/screen_audit.py 0x{short}\n"
                ));
            }
            if unsigned.len() > 20 {
                msg.push_str(&format!("  … and {} more\n", unsigned.len() - 20));
            }
        }
        if !mismatched.is_empty() {
            msg.push_str(&format!(
                "\nSTALE ({}) — spec or helpers.json changed since sign-off:\n",
                mismatched.len()
            ));
            for line in mismatched.iter().take(20) {
                msg.push_str(&format!("  {line}\n"));
            }
        }
        panic!("{msg}");
    }
}
