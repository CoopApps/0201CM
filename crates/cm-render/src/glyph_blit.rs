//! Glyph blitter — direct port of `FUN_005CED50` (11,384 bytes
//! decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_glyph_blit/0x005ced50.c`.
//! Full decode: [`reports/gui_glyph_blit_decode.md`](../../../../reports/gui_glyph_blit_decode.md).
//!
//! # 4-bit-per-pixel packed bitmap
//!
//! Each glyph is a byte array of `bytes_per_row × line_height` bytes.
//! `bytes_per_row = ceil(width_px / 2)`. Within each row:
//!
//! * Even x (0-based, from glyph origin) → nibble = `*p >> 4`
//! * Odd x → nibble = `*p & 0x0F`, then advance p by 1 byte
//! * If the last x in a row was even, advance p one extra byte
//!   (skip the unread low nibble)
//!
//! Nibble semantics:
//!
//! * `0`  → transparent (leave background pixel)
//! * `0xF` → opaque foreground (write `fg_color` raw)
//! * `1..0xE` → alpha level `a`; blend
//!   `r = (fg_r × a + bg_r × (15 - a)) / 15`
//!   for each channel, then re-pack into RGB565/555.
//!
//! # Signature
//!
//! ```pseudo
//! void draw_string(int x, int y, i16 font_slot, u32 fg_color,
//!                   u8* text, int underline_idx)
//! ```

use crate::Surface;

/// Font record — one per font slot. Matches the exe layout at
/// `DAT_00ACCB9C + slot*0x1404`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FontRecord {
    /// `+0x00` — pixel height per row.
    pub line_height: u32,
    /// 256 glyph records; indexed by char code 0..0xFF. Chars 0..0x1F
    /// stay default (width=0, no bitmap).
    pub glyphs: Vec<Glyph>,
}

impl FontRecord {
    pub fn new(line_height: u32) -> Self {
        Self { line_height, glyphs: vec![Glyph::default(); 256] }
    }
}

/// One glyph — 20 bytes in the exe layout (`+0..+0x14` per char).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Glyph {
    /// `+0x00` — width in pixels.
    pub width_px: u32,
    /// `+0x04` — bytes per row = `ceil(width_px / 2)`. Stored in the
    /// file rather than recomputed so tools can generate odd-authored
    /// glyphs safely.
    pub bytes_per_row: u32,
    /// `+0x08` — bearing right (kerning right).
    pub bearing_right: i32,
    /// `+0x0C` — bearing left (kerning left).
    pub bearing_left: i32,
    /// `+0x10` — the raw 4bpp pixel bytes.
    pub pixels: Vec<u8>,
}

/// Alpha blend one channel: `(fg × a + bg × (15 - a)) / 15`. Matches
/// the exe's `0x88888889` magic-multiply which computes `floor(x / 15)`
/// for non-negative x. Values here are all in `[0, 15]` × `[0, 255]` so
/// straight integer division matches exactly.
#[inline]
fn blend_channel(fg: u8, bg: u8, a: u8) -> u8 {
    let ia = 15 - a as u32;
    ((fg as u32 * a as u32 + bg as u32 * ia) / 15) as u8
}

/// Extract 8-bit R/G/B from a packed pixel using the given format's
/// masks. Ports the exe's `((px & mask) << 8) / (mask + 1)` — this
/// gives 0..0xF8 for a 5-bit channel, 0..0xFC for a 6-bit channel.
fn unpack_channels(px: u16, green_mask: u16) -> (u8, u8, u8) {
    let (rm, gm, bm): (u16, u16, u16) = if green_mask == 0x07E0 {
        (0xF800, 0x07E0, 0x001F)
    } else {
        (0x7C00, 0x03E0, 0x001F)
    };
    // exe: `((px & mask) << 8) / (mask + 1)` — expands 5-bit → 0..0xF8
    // and 6-bit → 0..0xFC. Parenthesise explicitly (Rust's `/` binds
    // tighter than `<<`).
    let unpack = |v: u16, mask: u16| -> u8 {
        (((v & mask) as u32 * 256) / (mask as u32 + 1)) as u8
    };
    (unpack(px, rm), unpack(px, gm), unpack(px, bm))
}

/// Repack (r, g, b) into a 16-bit pixel matching the format.
fn pack_channels(r: u8, g: u8, b: u8, green_mask: u16) -> u16 {
    if green_mask == 0x07E0 {
        ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3)
    } else {
        ((r as u16 & 0xF8) << 7) | ((g as u16 & 0xF8) << 2) | (b as u16 >> 3)
    }
}

/// Blit one glyph at (x, y) into `surface` with the given foreground
/// colour. Ports the inner loop of `FUN_005CED50` exactly.
///
/// * Transparent nibbles skip.
/// * `0xF` nibbles write `fg_color` raw.
/// * `1..0xE` nibbles blend against the current surface pixel.
pub fn blit_glyph(
    surface: &mut Surface,
    x: i32, y: i32,
    glyph: &Glyph,
    line_height: u32,
    fg_color: u16,
    fg_channels: (u8, u8, u8),
    green_mask: u16,
) {
    let width = glyph.width_px as i32;
    let height = line_height as i32;
    if width == 0 || glyph.pixels.is_empty() { return; }

    let (fg_r, fg_g, fg_b) = fg_channels;
    let mut p_idx = 0usize;

    for row in 0..height {
        let py = y + row;
        // Track byte cursor per row: reset high-nibble parity.
        let row_start = p_idx;
        let mut byte_ofs = 0usize;
        let mut high_nibble = true;
        for col in 0..width {
            // Guard against short pixel buffers (invariant: buffer
            // has bytes_per_row × line_height bytes).
            let idx = row_start + byte_ofs;
            if idx >= glyph.pixels.len() { break; }
            let px_val = if high_nibble {
                (glyph.pixels[idx] >> 4) & 0x0F
            } else {
                let v = glyph.pixels[idx] & 0x0F;
                byte_ofs += 1;
                v
            };
            high_nibble = !high_nibble;
            let dx = x + col;
            if dx >= 0 && (dx as usize) < surface.w
                && py >= 0 && (py as usize) < surface.h
            {
                match px_val {
                    0 => {}                          // transparent
                    0xF => surface.set(dx, py, fg_color),
                    a => {
                        let bg = surface.get(dx, py);
                        let (bg_r, bg_g, bg_b) = unpack_channels(bg, green_mask);
                        let r = blend_channel(fg_r, bg_r, a);
                        let g = blend_channel(fg_g, bg_g, a);
                        let b = blend_channel(fg_b, bg_b, a);
                        surface.set(dx, py, pack_channels(r, g, b, green_mask));
                    }
                }
            }
        }
        // End of row: advance to start of next row.
        // If last x was even (high_nibble is now false because we toggle
        // after each pixel), we already advanced. Otherwise we need to
        // skip one byte for the unread low nibble.
        if !high_nibble { byte_ofs += 1; }
        p_idx = row_start + glyph.bytes_per_row as usize;
        let _ = byte_ofs;   // consumed
    }
}

/// Direct port of `FUN_005CED50` — walk `text` and blit each glyph.
///
/// * `text` bytes < 0x20 (control) are skipped.
/// * `|` (0x7C) maps to space (0x20).
/// * `underline_idx` = 0-based char index to underline; -1 = none.
///
/// Font records + kerning are provided via the `font` argument.
pub fn draw_string(
    surface: &mut Surface,
    mut x: i32, y: i32,
    font: &FontRecord,
    fg_color: u16,
    text: &[u8],
    underline_idx: i32,
    green_mask: u16,
) {
    if font.glyphs.is_empty() { return; }
    let fg_channels = unpack_channels(fg_color, green_mask);
    let mut underline_x0: Option<i32> = None;
    let mut underline_x1: Option<i32> = None;

    let mut char_idx = 0i32;
    let mut prev_char: Option<u8> = None;
    for (i, &b) in text.iter().enumerate() {
        if b == 0 { break; }
        if b < 0x20 && b != 0x7C { continue; }
        let ch = if b == 0x7C { 0x20u8 } else { b };
        let glyph = &font.glyphs[ch as usize];
        // Kerning before this glyph.
        if let Some(prev) = prev_char {
            let scaled = (font.glyphs[0x20].width_px as i32) * 3 / 4;
            let delta = (scaled
                       - glyph.bearing_left
                       - font.glyphs[prev as usize].bearing_right).max(0);
            x += delta;
        }
        if char_idx == underline_idx {
            underline_x0 = Some(x);
        }
        blit_glyph(surface, x, y, glyph, font.line_height,
                    fg_color, fg_channels, green_mask);
        x += glyph.width_px as i32;
        if char_idx == underline_idx {
            underline_x1 = Some(x - 1);
        }
        char_idx += 1;
        prev_char = Some(ch);
        let _ = i;
    }

    // Underline via draw_line op-mode 2 (solid).
    if let (Some(x0), Some(x1)) = (underline_x0, underline_x1) {
        let baseline = font.line_height.saturating_sub(1) as i32;
        crate::line::draw_line(surface, x0, y + baseline,
                                x1, y + baseline,
                                crate::line::OP_SOLID, fg_color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_2x2_glyph() -> Glyph {
        // 2×2 all-opaque (nibble 0xF each). bytes_per_row = ceil(2/2) = 1.
        // One byte per row: high nibble = col 0, low nibble = col 1.
        Glyph {
            width_px: 2,
            bytes_per_row: 1,
            bearing_right: 0,
            bearing_left: 0,
            pixels: vec![0xFF, 0xFF],   // row 0: 0xFF, row 1: 0xFF
        }
    }

    #[test]
    fn blit_2x2_solid_writes_four_pixels() {
        let mut s = Surface::new();
        let g = solid_2x2_glyph();
        let fg = 0xF800;   // red
        let ch = unpack_channels(fg, 0x07E0);
        blit_glyph(&mut s, 100, 100, &g, 2, fg, ch, 0x07E0);
        assert_eq!(s.get(100, 100), 0xF800);
        assert_eq!(s.get(101, 100), 0xF800);
        assert_eq!(s.get(100, 101), 0xF800);
        assert_eq!(s.get(101, 101), 0xF800);
    }

    #[test]
    fn blit_transparent_nibbles_leave_bg_alone() {
        let mut s = Surface::new();
        // Fill background with green.
        for i in 0..s.buf.len() { s.buf[i] = 0x07E0; }
        // 2×1 glyph with nibble 0 (transparent), 0xF (opaque).
        let g = Glyph {
            width_px: 2, bytes_per_row: 1, bearing_right: 0, bearing_left: 0,
            pixels: vec![0x0F],   // col 0 = 0 transparent, col 1 = 0xF opaque red
        };
        let fg = 0xF800;
        let ch = unpack_channels(fg, 0x07E0);
        blit_glyph(&mut s, 50, 50, &g, 1, fg, ch, 0x07E0);
        assert_eq!(s.get(50, 50), 0x07E0, "transparent nibble preserves bg");
        assert_eq!(s.get(51, 50), 0xF800, "opaque nibble writes fg");
    }

    #[test]
    fn unpack_channels_matches_exe_5_to_8_bit_expansion() {
        // Pure red at RGB565 = 0xF800. Unpack: R = (0xF800 & 0xF800) * 256 / 0xF801 = 0xF800*256/0xF801 ≈ 248.
        let (r, g, b) = unpack_channels(0xF800, 0x07E0);
        assert!(r >= 247, "expected r ≈ 248, got {}", r);
        assert_eq!(g, 0);
        assert_eq!(b, 0);

        // White at RGB565 = 0xFFFF.
        let (r, g, b) = unpack_channels(0xFFFF, 0x07E0);
        assert!(r >= 247, "R should expand to ~248, got {}", r);
        assert!(g >= 251, "G should expand to ~252, got {}", g);
        assert!(b >= 247, "B should expand to ~248, got {}", b);

        // Black = 0.
        let (r, g, b) = unpack_channels(0x0000, 0x07E0);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn blit_alpha_blends_bg_and_fg_exactly() {
        let mut s = Surface::new();
        // Background = pure red (0xF800).
        for i in 0..s.buf.len() { s.buf[i] = 0xF800; }
        // Glyph = 1×1 with nibble 8 (roughly half-blend).
        let g = Glyph {
            width_px: 1, bytes_per_row: 1, bearing_right: 0, bearing_left: 0,
            pixels: vec![0x80],   // nibble 8, high position
        };
        // fg = pure blue (0x001F).
        let fg = 0x001F;
        let ch = unpack_channels(fg, 0x07E0);
        blit_glyph(&mut s, 0, 0, &g, 1, fg, ch, 0x07E0);
        let out = s.get(0, 0);
        // Blend with a=8: R = (0*8 + 248*7)/15 = 115; B = (248*8 + 0*7)/15 = 132.
        // Repacked → non-zero R and non-zero B.
        let (r, g_ch, b) = unpack_channels(out, 0x07E0);
        assert!(r > 100 && r < 130, "expected R ~115, got {}", r);
        assert_eq!(g_ch, 0);
        assert!(b > 120 && b < 140, "expected B ~132, got {}", b);
    }

    #[test]
    fn blend_channel_matches_exe_formula() {
        // a=0 → all bg. a=15 → all fg. a=7 → roughly half.
        assert_eq!(blend_channel(255, 0, 0), 0);
        assert_eq!(blend_channel(255, 0, 15), 255);
        // (255*7 + 0*8) / 15 = 1785/15 = 119
        assert_eq!(blend_channel(255, 0, 7), 119);
    }

    #[test]
    fn draw_string_writes_pixels_for_each_char() {
        let mut font = FontRecord::new(2);
        font.glyphs[b' ' as usize] = solid_2x2_glyph();
        font.glyphs[b'a' as usize] = solid_2x2_glyph();
        let mut s = Surface::new();
        draw_string(&mut s, 10, 20, &font, 0xF800, b"aa", -1, 0x07E0);
        // First 'a' at x=10..11, then kerning gap (`space_width * 3/4 = 1`),
        // second 'a' at x=13..14.
        for x in [10, 11, 13, 14] {
            for y in 20..22 {
                assert_eq!(s.get(x, y), 0xF800, "x={x} y={y}");
            }
        }
        // x=12 is the kerning gap.
        assert_eq!(s.get(12, 20), 0, "kerning gap column should be untouched");
    }

    #[test]
    fn draw_string_skips_control_chars_below_20() {
        let mut font = FontRecord::new(2);
        font.glyphs[b'A' as usize] = solid_2x2_glyph();
        let mut s = Surface::new();
        draw_string(&mut s, 10, 20, &font, 0xF800,
                     b"\x01\x02\x03A", -1, 0x07E0);
        // Only 'A' rendered — at (10, 20)..(11, 21).
        assert_eq!(s.get(10, 20), 0xF800);
        assert_eq!(s.get(11, 20), 0xF800);
    }

    #[test]
    fn pipe_maps_to_space_in_draw_string() {
        let mut font = FontRecord::new(2);
        let mut sp = solid_2x2_glyph();
        sp.pixels[0] = 0xF0;    // col 0 opaque, col 1 transparent
        font.glyphs[b' ' as usize] = sp;
        let mut s = Surface::new();
        draw_string(&mut s, 10, 20, &font, 0x001F, b"|", -1, 0x07E0);
        // Should render as space, not as literal '|' (which has empty glyph).
        assert_eq!(s.get(10, 20), 0x001F);
    }

    #[test]
    fn underline_index_draws_baseline() {
        let mut font = FontRecord::new(2);   // matches glyph height
        font.glyphs[b'A' as usize] = solid_2x2_glyph();
        let mut s = Surface::new();
        // Underline the FIRST char (index 0).
        draw_string(&mut s, 10, 30, &font, 0x001F, b"A", 0, 0x07E0);
        // Underline at y = 30 + (2-1) = 31; from x=10 to x=11.
        assert_eq!(s.get(10, 31), 0x001F);
        assert_eq!(s.get(11, 31), 0x001F);
    }
}
