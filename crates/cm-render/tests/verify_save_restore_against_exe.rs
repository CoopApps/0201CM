//! Byte-exact verification of save_rect and restore_rect against the
//! exe's FUN_005cd930 / FUN_005cda90.
//!
//! Two independent tests:
//!   - save_produces_identical_bytes: given known input pixels, our
//!     port's SavedRect.data must equal the exe's saved buffer byte-
//!     for-byte.
//!   - restore_reproduces_pattern_a: an exe restore of savedA over
//!     patternB yields patternA; our port must do the same when fed
//!     the exe's saved bytes as its SavedRect.

use std::path::PathBuf;

use base64::Engine;
use cm_render::packed::{PackedSurface, SavedRect};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    scratch: Scratch,
    save_case: SaveCase,
    restore_case: RestoreCase,
}

#[derive(Deserialize)]
struct Scratch {
    x0: i32, y0: i32,
    width: i32, height: i32,
}

#[derive(Deserialize)]
struct SaveCase {
    input_b64: String,
    saved_b64: String,
    saved_width: i32,
    saved_height: i32,
}

#[derive(Deserialize)]
struct RestoreCase {
    pattern_a_b64: String,
    pattern_b_b64: String,
    restored_b64: String,
    exe_saved_bytes_b64: String,
}

fn decode(b64: &str) -> Vec<u16> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).expect("b64");
    bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..").join("fixtures").join("verify_save_restore.json")
}

fn load() -> Option<Fixture> {
    let path = fixture_path();
    let text = std::fs::read_to_string(&path).ok()?;
    Some(serde_json::from_str(&text).expect("parse fixture"))
}

#[test]
fn save_produces_identical_bytes_to_fun_005cd930() {
    let Some(fx) = load() else {
        eprintln!("SKIPPED — no fixture"); return;
    };
    let (w, h) = (fx.scratch.width, fx.scratch.height);
    let input = decode(&fx.save_case.input_b64);
    let s = PackedSurface {
        buf: input, width: w, height: h, pitch_pixels: w,
        red_mask: 0x7c00, green_mask: 0x03e0, blue_mask: 0x001f,
    };
    let saved = s.save_rect(0, 0, w - 1, h - 1).expect("in-bounds save");
    assert_eq!(saved.width, fx.save_case.saved_width, "width");
    assert_eq!(saved.height, fx.save_case.saved_height, "height");
    let exe_saved = decode(&fx.save_case.saved_b64);
    assert_eq!(saved.data.len(), exe_saved.len(), "byte count");
    assert_eq!(saved.data, exe_saved, "save data must match exe byte-for-byte");
    eprintln!("PASS — save_rect matches exe on {}x{} scratch", w, h);
}

#[test]
fn restore_reproduces_pattern_a_from_exe_saved_bytes() {
    let Some(fx) = load() else {
        eprintln!("SKIPPED — no fixture"); return;
    };
    let (w, h) = (fx.scratch.width, fx.scratch.height);
    let pattern_a = decode(&fx.restore_case.pattern_a_b64);
    let pattern_b = decode(&fx.restore_case.pattern_b_b64);
    let exe_restored = decode(&fx.restore_case.restored_b64);
    let exe_saved = decode(&fx.restore_case.exe_saved_bytes_b64);
    // Sanity: exe's saved bytes ARE pattern A (roundtrip through the exe worked).
    assert_eq!(exe_saved, pattern_a,
        "exe's saved buffer must equal pattern A — sanity check on the fixture");
    // And the exe's restore-over-pattern-B did yield pattern A.
    assert_eq!(exe_restored, pattern_a,
        "exe's restore over pattern B must yield pattern A — fixture sanity");
    // Now our port: start from pattern B, restore pattern A via SavedRect.
    let mut s = PackedSurface {
        buf: pattern_b, width: w, height: h, pitch_pixels: w,
        red_mask: 0x7c00, green_mask: 0x03e0, blue_mask: 0x001f,
    };
    let saved = SavedRect { width: w, height: h, data: exe_saved };
    s.restore_rect(0, 0, &saved);
    assert_eq!(s.buf, pattern_a,
        "our restore must yield pattern A byte-for-byte");
    eprintln!("PASS — restore_rect matches exe on {}x{} scratch", w, h);
}
