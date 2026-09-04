//! Byte-exact verification of `PackedSurface::draw_panel` against
//! FUN_005cf570 from live cm0102_GDI.exe.

use std::path::PathBuf;

use base64::Engine;
use cm_render::packed::PackedSurface;
use cm_render::packed_panel::{draw_panel, PanelPalette};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    palette: PaletteDump,
    scratch: Scratch,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct PaletteDump {
    outer_highlight: u16,
    default_bevel: u16,
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
    /// 7th arg of FUN_005cf570 (circle-outline colour). Cases captured
    /// before the field existed passed 0 to the exe, so 0 is the default.
    #[serde(default)]
    pattern: u16,
    before_b64: String,
    after_b64: String,
}

fn decode(b64: &str) -> Vec<u16> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).expect("b64");
    bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..").join("fixtures").join("verify_panel.json")
}

#[test]
fn draw_panel_matches_fun_005cf570_byte_exact() {
    let path = fixture_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("SKIPPED — no fixture at {}", path.display()); return;
    };
    let fx: Fixture = serde_json::from_str(&text).expect("parse fixture");
    let (w, h) = (fx.scratch.width, fx.scratch.height);
    let (sh_x, sh_y) = (fx.scratch.x0, fx.scratch.y0);
    // Palette globals read live from the running exe (DAT_00ad6b24 +
    // DAT_00ad6b3c) must equal what the ported palette-reload
    // (FUN_005cdfa0) derives for the RGB555 surface.
    let live = PanelPalette {
        outer_highlight: fx.palette.outer_highlight,
        default_bevel: fx.palette.default_bevel,
    };
    let derived = PanelPalette::from_palette_reload(&PackedSurface::rgb555(1, 1));
    assert_eq!(derived.outer_highlight, live.outer_highlight,
               "DAT_00ad6b24: FUN_005cdfa0 port vs live exe");
    assert_eq!(derived.default_bevel, live.default_bevel,
               "DAT_00ad6b3c: FUN_005cdfa0 port vs live exe");
    let mut failures: Vec<String> = Vec::new();
    let mut passes: Vec<String> = Vec::new();
    for case in &fx.cases {
        let before = decode(&case.before_b64);
        let after_expected = decode(&case.after_b64);
        let mut s = PackedSurface {
            buf: before, width: w, height: h, pitch_pixels: w,
            red_mask: 0x7c00, green_mask: 0x03e0, blue_mask: 0x001f,
        };
        draw_panel(&mut s,
                   case.x0 - sh_x, case.y0 - sh_y,
                   case.x1 - sh_x, case.y1 - sh_y,
                   case.style, case.colour, case.pattern,
                   live);
        if s.buf == after_expected {
            passes.push(case.label.clone());
        } else {
            let first = s.buf.iter().zip(after_expected.iter()).enumerate()
                .find(|(_, (a, b))| a != b);
            let (idx, ours, exe) = first.map(|(i, (a, b))| (i, *a, *b)).unwrap_or((0, 0, 0));
            let total: usize = s.buf.iter().zip(after_expected.iter()).filter(|(a,b)| a != b).count();
            failures.push(format!(
                "{:<26} style={:#010x} colour={:#06x} rect=({},{})→({},{})\n    first diff idx={} (x={}, y={}): exe={:#06x} ours={:#06x}  ({} pixels differ)",
                case.label, case.style, case.colour,
                case.x0, case.y0, case.x1, case.y1,
                idx, idx as i32 % w, idx as i32 / w, exe, ours, total,
            ));
        }
    }
    eprintln!("passes ({}):\n  {}", passes.len(), passes.join("\n  "));
    if !failures.is_empty() {
        panic!("{} of {} cases disagree with exe:\n  {}",
               failures.len(), fx.cases.len(), failures.join("\n  "));
    }
    eprintln!("PASS — {} panel cases all match exe FUN_005cf570", fx.cases.len());
}
