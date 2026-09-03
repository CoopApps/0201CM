//! Byte-exact verification of `PackedSurface::clip` against
//! `FUN_005cd330` from a live cm0102_GDI.exe. Fixture captured by
//! `tools/verify/verify_clip.py`.

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
    width: i32,
    height: i32,
}

#[derive(Deserialize)]
struct Case {
    input: [i32; 4],
    output: Option<[i32; 4]>,
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .join("verify_clip.json")
}

#[test]
fn clip_matches_fun_005cd330_byte_exact() {
    let path = fixture_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("SKIPPED — no fixture at {}", path.display());
        return;
    };
    let fx: Fixture = serde_json::from_str(&text).expect("parse fixture");
    let s = PackedSurface::rgb555(fx.meta.width, fx.meta.height);
    let mut mismatches: Vec<String> = Vec::new();
    for c in &fx.cases {
        let ours = s.clip(c.input[0], c.input[1], c.input[2], c.input[3]);
        let ours_arr = ours.map(|(l, t, r, b)| [l, t, r, b]);
        if ours_arr != c.output {
            mismatches.push(format!(
                "clip({:>5},{:>5},{:>5},{:>5}) → exe {:?}  ours {:?}",
                c.input[0], c.input[1], c.input[2], c.input[3],
                c.output, ours_arr,
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
    eprintln!("PASS — {} cases all match exe FUN_005cd330", fx.cases.len());
}
