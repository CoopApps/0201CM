//! Byte-exact verification of `packed::PackedSurface::pack_rgb` against
//! the running cm0102_GDI.exe's `FUN_005ce240` (pack-RGB). The fixture
//! is captured by `tools/verify/verify_pack_rgb.py` — attach to a live
//! game, call the exe function on a 5,000+ triple test matrix, dump
//! each `(r, g, b) -> packed_u16` result. This test asserts our Rust
//! port produces the identical `packed_u16` for every triple.
//!
//! Fixture path: `fixtures/verify_pack_rgb.json`. Missing fixture =
//! skipped test with a note; the harness cannot spin up an exe on
//! its own, so if the fixture isn't there someone hasn't run it yet.
//!
//! Format:
//! ```json
//! {
//!   "va": "0x005ce240",
//!   "meta": { "red_mask": 31744, "green_mask": 992, "blue_mask": 31 },
//!   "cases": [{"r": 0, "g": 0, "b": 0, "packed": 0}, ...]
//! }
//! ```

use std::path::PathBuf;

use cm_render::packed::PackedSurface;
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    meta: Meta,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Meta {
    red_mask: u16,
    green_mask: u16,
    blue_mask: u16,
}

#[derive(Deserialize)]
struct Case {
    r: u8,
    g: u8,
    b: u8,
    packed: u16,
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .join("verify_pack_rgb.json")
}

#[test]
fn pack_rgb_matches_fun_005ce240_byte_exact() {
    let path = fixture_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!(
            "SKIPPED — no fixture at {}. Run tools/verify/verify_pack_rgb.py against a live cm0102_GDI.exe.",
            path.display()
        );
        return;
    };
    let fx: Fixture = serde_json::from_str(&text).expect("parse fixture");
    // Build a surface with the exe's masks so pack_rgb dispatches to the same branch.
    let s = PackedSurface {
        buf: vec![],
        width: 1, height: 1, pitch_pixels: 1,
        red_mask: fx.meta.red_mask,
        green_mask: fx.meta.green_mask,
        blue_mask: fx.meta.blue_mask,
    };
    let mut mismatches: Vec<String> = Vec::new();
    for c in &fx.cases {
        let ours = s.pack_rgb(c.r, c.g, c.b);
        if ours != c.packed {
            mismatches.push(format!(
                "pack_rgb({:3}, {:3}, {:3}) → exe {:#06x}  ours {:#06x}",
                c.r, c.g, c.b, c.packed, ours,
            ));
            if mismatches.len() >= 10 {
                break;
            }
        }
    }
    if !mismatches.is_empty() {
        panic!(
            "{} of {} cases disagree with the exe. First 10:\n  {}",
            mismatches.len(), fx.cases.len(), mismatches.join("\n  "),
        );
    }
    eprintln!("PASS — {} cases all match exe FUN_005ce240", fx.cases.len());
}
