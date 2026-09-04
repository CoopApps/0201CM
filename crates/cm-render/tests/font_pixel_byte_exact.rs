//! Byte-exact verification of the `.fnt` → `PixelFont` load path against
//! a captured render fixture.
//!
//! `fixtures/verify_glyph.json` is a live capture from cm0102_GDI.exe:
//! its `"fonts": {"3": {...}}` field is the game's own in-memory copy of
//! font slot 3 (arial_14 — the traditional table-body font). We load the
//! same font off disk via [`cm_render::font::Font::parse`] +
//! [`cm_render::font::Font::to_pixel_font`] and assert every printable
//! glyph's `width`, `kern_b`, `kern_c`, and packed 4-bit bitmap match
//! byte-for-byte.
//!
//! `kern_a` (the exe's second int per glyph record — `bmw`, unused by
//! draw or kern) is a re-derivation from `ceil(width / 2)` on the
//! capture side, so we don't demand equality on it here; the disk
//! loader records the file's stored bmw verbatim (see `Font::parse`).
//!
//! When either input file is missing (a lean CI without assets) the
//! test no-ops with a note — matches the pattern in `font.rs::tests`.

use base64::Engine;
use cm_render::font::Font;
use serde_json::Value;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // Cargo runs tests with CWD = the crate directory; walk up one level
    // to the workspace root where `fixtures/` and `assets/` live.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // .../crates
    p.pop(); // repo root
    p
}

#[test]
fn arial_14_disk_load_matches_captured_font() {
    let root = repo_root();
    let fnt_path = root.join("assets/cm0102/original_files/Data/arial_14.fnt");
    let fx_path = root.join("fixtures/verify_glyph.json");
    if !fnt_path.exists() || !fx_path.exists() {
        eprintln!(
            "skip font byte-exact test: missing {} or {}",
            fnt_path.display(),
            fx_path.display()
        );
        return;
    }

    let font = Font::load(&fnt_path, 21).expect("load arial_14.fnt");
    let pf = font.to_pixel_font();
    assert_eq!(pf.height, 21, "arial_14 bitmap height");

    let fx_text = std::fs::read_to_string(&fx_path).expect("read fixture");
    let v: Value = serde_json::from_str(&fx_text).expect("parse fixture");
    let captured = &v["fonts"]["3"];
    assert_eq!(captured["height"].as_i64().unwrap(), 21);
    let cap_glyphs = captured["glyphs"].as_array().expect("glyphs array");
    assert_eq!(cap_glyphs.len(), 256, "capture has full 256 slot table");

    // Compare every printable code point (0x20..=0xFF). `kern_a`
    // is the bmw field, which the capture re-derives — skip it.
    let mut checked = 0usize;
    for cp in 0x20u8..=0xFF {
        let cap = &cap_glyphs[cp as usize];
        let want_width = cap["width"].as_i64().unwrap() as i32;
        let want_kb = cap["kern_b"].as_i64().unwrap() as i32;
        let want_kc = cap["kern_c"].as_i64().unwrap() as i32;
        let want_bitmap_b64 = cap["bitmap_b64"].as_str().unwrap_or("");
        let want_bitmap = base64::engine::general_purpose::STANDARD
            .decode(want_bitmap_b64)
            .expect("decode bitmap_b64");

        let got = pf.glyphs[cp as usize]
            .as_ref()
            .unwrap_or_else(|| panic!("cp {:#x} missing in loaded font", cp));
        assert_eq!(got.width, want_width, "cp {:#x} width mismatch", cp);
        assert_eq!(got.kern_b, want_kb, "cp {:#x} kern_b mismatch", cp);
        assert_eq!(got.kern_c, want_kc, "cp {:#x} kern_c mismatch", cp);
        assert_eq!(
            got.bitmap, want_bitmap,
            "cp {:#x} bitmap mismatch ({} vs {} bytes)",
            cp,
            got.bitmap.len(),
            want_bitmap.len()
        );
        checked += 1;
    }
    // 0x20..=0xFF = 224 glyphs.
    assert_eq!(checked, 224, "checked full printable range");
}
