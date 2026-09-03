//! Byte-exact verification of `PackedSurface::darken_rect` against
//! FUN_005cdd60 from live cm0102_GDI.exe.

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
    before_b64: String,
    after_b64: String,
}

fn decode(b64: &str) -> Vec<u16> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).expect("b64");
    bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..").join("fixtures").join("verify_darken.json")
}

#[test]
fn darken_rect_matches_fun_005cdd60_byte_exact() {
    let path = fixture_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("SKIPPED — no fixture at {}", path.display()); return;
    };
    let fx: Fixture = serde_json::from_str(&text).expect("parse fixture");
    let (w, h) = (fx.scratch.width, fx.scratch.height);
    let mut failures = Vec::new();
    for case in &fx.cases {
        let before = decode(&case.before_b64);
        let after_expected = decode(&case.after_b64);
        let mut s = PackedSurface {
            buf: before, width: w, height: h, pitch_pixels: w,
            red_mask: 0x7c00, green_mask: 0x03e0, blue_mask: 0x001f,
        };
        s.darken_rect(0, 0, w - 1, h - 1);
        if s.buf != after_expected {
            let first = s.buf.iter().zip(after_expected.iter()).enumerate()
                .find(|(_, (a, b))| a != b);
            let (idx, ours, exe) = first.map(|(i, (a, b))| (i, *a, *b))
                .unwrap_or((0, 0, 0));
            let total: usize = s.buf.iter().zip(after_expected.iter()).filter(|(a,b)| a != b).count();
            let before_pixel = decode(&case.before_b64)[idx];
            failures.push(format!(
                "{:<12} first diff idx={} (x={}, y={}) before={:#06x}: exe={:#06x} ours={:#06x}  ({} total)",
                case.label, idx, idx as i32 % w, idx as i32 / w,
                before_pixel, exe, ours, total,
            ));
        }
    }
    if !failures.is_empty() {
        panic!("{} of {} cases disagree:\n  {}",
               failures.len(), fx.cases.len(), failures.join("\n  "));
    }
    eprintln!("PASS — {} cases all match exe FUN_005cdd60", fx.cases.len());
}
