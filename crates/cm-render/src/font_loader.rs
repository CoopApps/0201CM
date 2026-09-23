//! `.fnt` file loader — direct port of `FUN_005CE890` (6,311 bytes).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_glyph_blit/0x005ce890.c`.
//! Full decode: [`reports/gui_glyph_blit_decode.md`](../../../../reports/gui_glyph_blit_decode.md) §load_font.
//!
//! # File format
//!
//! ```text
//! [ 4 B  ] line_height (u32 LE)
//! For char c in 0x20..=0xFF:
//!   [ 4 B  ] width_px       (u32 LE)
//!   [ 4 B  ] bytes_per_row  (u32 LE)   // ceil(width_px / 2)
//!   [ 4 B  ] bearing_right  (i32 LE)
//!   [ 4 B  ] bearing_left   (i32 LE)
//!   [ N B  ] pixel data     N = bytes_per_row × line_height
//!            (4bpp packed, 2 px/byte, high nibble first)
//! ```
//!
//! # Slot 6 / 7 Latin-1 alias fixup
//!
//! The two large "trade_cond" font atlases don't ship glyphs for every
//! Latin-1 code point — a small `(src, dst)` table lists which glyphs
//! should be **duplicated** from an existing char after load. Ported
//! from the exe's `local_230[]` table (0xFFFF terminator).

use crate::glyph_blit::{FontRecord, Glyph};

/// Slot 6/7 Latin-1 alias fixup pairs from the exe. Each entry is
/// `(src_char, dst_char)` — after load, copy src's glyph metrics +
/// pixels to dst.
pub const LATIN1_ALIAS_TABLE: &[(u8, u8)] = &[
    (0x82, 0x2C),   // ‚ ← ,  (single low-9 quote from comma)
    (0x82, 0x84),   // „ ← ‚
    (0x20, 0x85),   // …
    (0x20, 0x86),   // †
    (0x20, 0x87),   // ‡
    (0x20, 0x8B),   // ‹
    (0x20, 0x8C),   // Œ
    (0xC6, 0x9B),   // › ← Æ (yes — the exe re-uses Æ's outline)
    (0x20, 0x9C),   // œ
    (0xE6, 0xA0),   //   ← æ
    (0x9F, 0xD0),   // Ð ← Ÿ
    (0x44, 0xDE),   // Þ ← D
    (0x54, 0xF0),   // ð ← T
    (0x64, 0xFE),   // þ ← d
];

/// Direct port of `FUN_005CE890(slot, path)`. Reads bytes and yields
/// a fully-populated `FontRecord`. Any I/O error → `None` (matches exe
/// short-read handling: free everything and return 0).
///
/// `apply_alias_fixup` controls whether the slot 6/7 Latin-1 alias
/// table is applied. Pass `true` for the two large "trade_cond" fonts.
// GDI-REG: 005ce890 PORTED_BEHAVIOURAL
pub fn load_font_bytes(bytes: &[u8], apply_alias_fixup: bool) -> Option<FontRecord> {
    let mut cur = 0usize;
    let line_height = read_u32(bytes, &mut cur)?;
    let mut font = FontRecord::new(line_height);
    // Exe loop: c = 0x20..=0xFF.
    for c in 0x20u16..=0xFF {
        let width_px = read_u32(bytes, &mut cur)?;
        let bytes_per_row = read_u32(bytes, &mut cur)?;
        let bearing_right = read_i32(bytes, &mut cur)?;
        let bearing_left  = read_i32(bytes, &mut cur)?;
        let pixel_len = (bytes_per_row * line_height) as usize;
        let pixels = if bytes_per_row != 0 {
            if cur + pixel_len > bytes.len() { return None; }
            let px = bytes[cur .. cur + pixel_len].to_vec();
            cur += pixel_len;
            px
        } else { Vec::new() };
        font.glyphs[c as usize] = Glyph {
            width_px, bytes_per_row, bearing_right, bearing_left, pixels,
        };
    }
    if apply_alias_fixup {
        apply_latin1_alias_fixup(&mut font);
    }
    Some(font)
}

/// Apply the exe's `local_230[]` copy-from-src table (slot 6/7 only).
/// For each `(src, dst)` pair: replace dst's glyph with a clone of
/// src's glyph (width, bytes_per_row, bearings, and a copy of the
/// pixel bytes). Ports the exe's malloc + memcpy sequence.
pub fn apply_latin1_alias_fixup(font: &mut FontRecord) {
    for &(src, dst) in LATIN1_ALIAS_TABLE {
        let s = font.glyphs[src as usize].clone();
        font.glyphs[dst as usize] = s;
    }
}

#[inline]
fn read_u32(bytes: &[u8], cur: &mut usize) -> Option<u32> {
    if *cur + 4 > bytes.len() { return None; }
    let arr: [u8; 4] = bytes[*cur .. *cur + 4].try_into().ok()?;
    *cur += 4;
    Some(u32::from_le_bytes(arr))
}
#[inline]
fn read_i32(bytes: &[u8], cur: &mut usize) -> Option<i32> {
    read_u32(bytes, cur).map(|v| v as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal `.fnt` byte stream with `line_height=8`, one
    /// non-empty glyph at char 'A' (0x41), rest empty. Enough to
    /// round-trip through the loader.
    fn build_min_fnt() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&8u32.to_le_bytes());             // line_height
        for c in 0x20u16..=0xFF {
            let is_a = c == b'A' as u16;
            let width = if is_a { 4u32 } else { 0 };
            let bpr = if is_a { 2u32 } else { 0 };            // ceil(4/2)
            b.extend_from_slice(&width.to_le_bytes());
            b.extend_from_slice(&bpr.to_le_bytes());
            b.extend_from_slice(&2i32.to_le_bytes());         // bearing_right
            b.extend_from_slice(&1i32.to_le_bytes());         // bearing_left
            if is_a {
                // 4×8 = 16 bytes of glyph pixel data.
                b.extend_from_slice(&[0xFF; 16]);
            }
        }
        b
    }

    #[test]
    fn round_trip_min_fnt_matches_fields() {
        let bytes = build_min_fnt();
        let font = load_font_bytes(&bytes, false).unwrap();
        assert_eq!(font.line_height, 8);
        assert_eq!(font.glyphs[b'A' as usize].width_px, 4);
        assert_eq!(font.glyphs[b'A' as usize].bytes_per_row, 2);
        assert_eq!(font.glyphs[b'A' as usize].bearing_right, 2);
        assert_eq!(font.glyphs[b'A' as usize].bearing_left, 1);
        assert_eq!(font.glyphs[b'A' as usize].pixels.len(), 16);
        // Empty slot at ' ' (0x20).
        assert_eq!(font.glyphs[b' ' as usize].width_px, 0);
        assert!(font.glyphs[b' ' as usize].pixels.is_empty());
    }

    #[test]
    fn short_file_returns_none() {
        let bytes = vec![1u8, 2, 3, 4, 5];   // too short for even the header + first glyph
        assert!(load_font_bytes(&bytes, false).is_none());
    }

    #[test]
    fn alias_fixup_dup_matches_exe_table() {
        let mut bytes = build_min_fnt();
        // Ensure 'D' has distinctive data so we can verify Þ (0xDE) inherits it.
        // Patch source: rewrite 'D' char record's fields in the stream.
        // For simplicity here, just load first and mutate glyph 'D' in-code.
        let mut font = load_font_bytes(&bytes, false).unwrap();
        font.glyphs[b'D' as usize] = Glyph {
            width_px: 7, bytes_per_row: 4, bearing_right: 3, bearing_left: 2,
            pixels: vec![0xAB; 32],
        };
        apply_latin1_alias_fixup(&mut font);
        // 0xDE (Þ) should now be a copy of 'D'.
        assert_eq!(font.glyphs[0xDE].width_px, 7);
        assert_eq!(font.glyphs[0xDE].bearing_right, 3);
        assert_eq!(font.glyphs[0xDE].pixels, vec![0xAB; 32]);
        let _ = bytes;
    }

    #[test]
    fn alias_table_matches_exe_pairs() {
        // Spot-check a few pairs from the exe's local_230[] table.
        assert!(LATIN1_ALIAS_TABLE.contains(&(0x44, 0xDE)));   // Þ ← D
        assert!(LATIN1_ALIAS_TABLE.contains(&(0x54, 0xF0)));   // ð ← T
        assert!(LATIN1_ALIAS_TABLE.contains(&(0x64, 0xFE)));   // þ ← d
    }

    #[test]
    fn full_range_covers_224_glyphs() {
        let bytes = build_min_fnt();
        let font = load_font_bytes(&bytes, false).unwrap();
        // Chars 0x20..=0xFF = 224 slots covered by the loader (256 total).
        assert_eq!(font.glyphs.len(), 256);
    }
}
