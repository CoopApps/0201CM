//! Byte-exact verification of `packed_glyph::draw_text` against
//! FUN_005ceaa0 (traditional-font path) from live cm0102_GDI.exe.

use std::collections::HashMap;
use std::path::PathBuf;

use base64::Engine;
use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::{draw_text, Glyph, PixelFont};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    scratch: Scratch,
    fonts: HashMap<String, FontDump>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Scratch { x0: i32, y0: i32, width: i32, height: i32 }

#[derive(Deserialize)]
struct FontDump {
    height: i32,
    glyphs: Vec<Option<GlyphDump>>,
}

#[derive(Deserialize)]
struct GlyphDump {
    width: i32,
    #[serde(default)] kern_a: i32,
    #[serde(default)] kern_b: i32,
    #[serde(default)] kern_c: i32,
    bitmap_b64: String,
}

#[derive(Deserialize)]
struct Case {
    label: String,
    x: i32, y: i32,
    font: u16,
    colour: u16,
    text: String,
    before_b64: String,
    after_b64: String,
}

fn decode_u16(b64: &str) -> Vec<u16> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).expect("b64");
    bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

fn decode_bytes(b64: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD.decode(b64).expect("b64")
}

fn build_font(dump: &FontDump) -> PixelFont {
    let mut f = PixelFont::empty(dump.height);
    for (i, g) in dump.glyphs.iter().enumerate() {
        if let Some(g) = g {
            if i < f.glyphs.len() {
                let bitmap = if g.bitmap_b64.is_empty() {
                    Vec::new()
                } else {
                    decode_bytes(&g.bitmap_b64)
                };
                f.glyphs[i] = Some(Glyph {
                    width: g.width,
                    kern_a: g.kern_a, kern_b: g.kern_b, kern_c: g.kern_c,
                    bitmap,
                });
            }
        }
    }
    f
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..").join("fixtures").join("verify_glyph.json")
}

#[test]
fn draw_text_matches_fun_005ceaa0_byte_exact() {
    let path = fixture_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("SKIPPED — no fixture at {}", path.display()); return;
    };
    let fx: Fixture = serde_json::from_str(&text).expect("parse fixture");
    let (w, h) = (fx.scratch.width, fx.scratch.height);
    let (sh_x, sh_y) = (fx.scratch.x0, fx.scratch.y0);

    let mut passes: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    for case in &fx.cases {
        let font_dump = fx.fonts.get(&case.font.to_string())
            .unwrap_or_else(|| panic!("no font dump for {}", case.font));
        let font = build_font(font_dump);
        let before = decode_u16(&case.before_b64);
        let after_expected = decode_u16(&case.after_b64);
        let mut s = PackedSurface {
            buf: before, width: w, height: h, pitch_pixels: w,
            red_mask: 0x7c00, green_mask: 0x03e0, blue_mask: 0x001f,
        };
        draw_text(&mut s, case.x - sh_x, case.y - sh_y,
                  &font, case.text.as_bytes(), case.colour);
        if s.buf == after_expected {
            passes.push(case.label.clone());
        } else {
            let first = s.buf.iter().zip(after_expected.iter()).enumerate()
                .find(|(_, (a, b))| a != b);
            let (idx, ours, exe) = first.map(|(i, (a, b))| (i, *a, *b)).unwrap_or((0, 0, 0));
            let total: usize = s.buf.iter().zip(after_expected.iter()).filter(|(a,b)| a != b).count();
            failures.push(format!(
                "{:<20} font={} colour={:#06x} text={:?}\n    first diff idx={} (x={}, y={}): exe={:#06x} ours={:#06x}  ({} pixels differ)",
                case.label, case.font, case.colour, case.text,
                idx, idx as i32 % w, idx as i32 / w, exe, ours, total,
            ));
        }
    }
    eprintln!("passes ({}):\n  {}", passes.len(), passes.join("\n  "));
    if !failures.is_empty() {
        panic!("{} of {} cases disagree:\n  {}", failures.len(), fx.cases.len(), failures.join("\n  "));
    }
    eprintln!("PASS — {} glyph cases all match exe FUN_005ceaa0", fx.cases.len());
}
