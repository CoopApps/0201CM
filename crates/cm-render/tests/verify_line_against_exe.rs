//! Byte-exact verification of `PackedSurface::draw_line` against
//! FUN_005cd3e0 from a live cm0102_GDI.exe.
//!
//! Fixture is captured by `tools/verify/verify_line.py` — for each of
//! 15 line-shape cases (horizontal solid/dashed, vertical solid/dashed,
//! diagonals in all quadrants, x-major and y-major Bresenham, reversed
//! endpoints, single pixel), the harness saves the scratch region, runs
//! the exe's line, reads the resulting pixels, restores. Each case
//! carries (before_pixels, after_pixels).
//!
//! This test loads each case, builds a PackedSurface pre-populated with
//! `before_pixels` at the scratch offset within a 200x40 mini-surface,
//! runs our port on the same inputs, asserts the result matches
//! `after_pixels` byte-for-byte.

use std::path::PathBuf;

use base64::Engine;
use cm_render::packed::PackedSurface;
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    scratch: Scratch,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Scratch {
    x0: i32, y0: i32,
    width: i32, height: i32,
}

#[derive(Deserialize)]
struct Case {
    label: String,
    x0: i32, y0: i32, x1: i32, y1: i32,
    style: u32,
    colour: u16,
    before_b64: String,
    after_b64: String,
}

fn decode(b64: &str) -> Vec<u16> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).expect("b64");
    bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .join("verify_line.json")
}

#[test]
fn draw_line_matches_fun_005cd3e0_byte_exact() {
    let path = fixture_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("SKIPPED — no fixture at {}", path.display());
        return;
    };
    let fx: Fixture = serde_json::from_str(&text).expect("parse fixture");
    let (w, h) = (fx.scratch.width, fx.scratch.height);
    // The exe draws into a 200x40 region at (scratch.x0, scratch.y0) inside
    // an 800x600 backbuffer, but our test surface is JUST that region —
    // remap the exe's absolute coords into our surface coords by shifting.
    // The exe also uses RGB555 masks (same as the live game).
    let sh_x = fx.scratch.x0;
    let sh_y = fx.scratch.y0;
    let mut failures = Vec::new();
    for case in &fx.cases {
        let before = decode(&case.before_b64);
        let after_expected = decode(&case.after_b64);
        assert_eq!(before.len(), (w * h) as usize);
        assert_eq!(after_expected.len(), (w * h) as usize);
        let mut s = PackedSurface {
            buf: before,
            width: w,
            height: h,
            pitch_pixels: w,
            red_mask: 0x7c00,
            green_mask: 0x03e0,
            blue_mask: 0x001f,
        };
        // Shift the exe's absolute coords into our surface's own coord space.
        // The exe drew at (case.x0..case.x1) in the backbuffer, which — since
        // scratch.x0 is 0 in this test — is the same as our surface coords.
        // Kept explicit so this works if scratch is ever moved.
        s.draw_line(case.x0 - sh_x, case.y0 - sh_y, case.x1 - sh_x, case.y1 - sh_y,
                    case.style, case.colour);
        if s.buf != after_expected {
            let mut first_diff = None;
            for (i, (&a, &b)) in s.buf.iter().zip(after_expected.iter()).enumerate() {
                if a != b {
                    first_diff = Some((i, a, b));
                    break;
                }
            }
            let (idx, ours, exe) = first_diff.unwrap();
            failures.push(format!(
                "{:<28} line({},{})→({},{}) style={} colour={:#06x}\n    first diff idx={} (x={}, y={}): exe={:#06x} ours={:#06x}",
                case.label, case.x0, case.y0, case.x1, case.y1, case.style, case.colour,
                idx, idx as i32 % w, idx as i32 / w, exe, ours,
            ));
        }
    }
    if !failures.is_empty() {
        panic!("{} of {} cases disagree with exe:\n  {}",
               failures.len(), fx.cases.len(), failures.join("\n  "));
    }
    eprintln!("PASS — {} cases all match exe FUN_005cd3e0", fx.cases.len());
}
