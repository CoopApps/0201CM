//! Glyph rendering onto `PackedSurface` — literal port of the traditional-
//! font branch of `FUN_005ceaa0` (GDI variant, 1904 bytes).
//!
//! The exe stores each font as a 5124-byte table at
//! `DAT_00accae4 + font_idx*0x1404`: one int of font height, then 256
//! per-character records of 20 bytes / 5 ints — `[width, ?, ?, ?, bitmap*]`
//! (three middle ints not touched by the renderer, likely metrics for the
//! runtime layout engine and kerning). Each glyph bitmap is 4-bit alpha
//! per pixel, nibble-packed row-major (high nibble first). 0 = transparent,
//! 0xf = solid glyph colour, 1..14 = alpha-blended `(glyph*a + dst*(15-a))
//! / 15` per channel in the surface's pixel format.
//!
//! The full function also implements the futuristic-font branch
//! (`DAT_009b9d54 != 0`, dispatches into `FUN_0059b550` — a separate
//! scalable renderer), the `|` → space substitution, and the trailing
//! underline stroke driven by `param_6`. This first cut lifts the
//! traditional-font inner loop faithfully; futuristic and underline
//! land as follow-ups.

use crate::packed::PackedSurface;

/// One glyph — width in pixels + a 4-bit nibble-packed bitmap of
/// `ceil(width * height / 2)` bytes. `height` is carried by the parent
/// `PixelFont` (the exe stores it once per font as the first int).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glyph {
    pub width: i32,
    /// Row-major, high-nibble first. A row that ends on the high nibble
    /// (odd `width`) advances the byte pointer at row end — matches the
    /// exe's post-loop `if (!bVar25) pbVar22 += 1`.
    pub bitmap: Vec<u8>,
}

/// A pixel font — 256 glyphs at a fixed height. Codes with no glyph
/// (`None`) render as nothing and advance the pen by zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelFont {
    pub height: i32,
    pub glyphs: Vec<Option<Glyph>>,
}

impl PixelFont {
    /// Empty font — every glyph absent. Useful for tests.
    pub fn empty(height: i32) -> Self {
        Self { height, glyphs: (0..256).map(|_| None).collect() }
    }
}

/// Draw one glyph at `(dst_x, dst_y)` onto `surface`. Returns the pen's
/// advance (the exe's `iVar5` — the glyph's width; the surrounding text
/// walker uses this plus a per-char kern from `FUN_005cf4d0`).
pub fn draw_glyph(
    surface: &mut PackedSurface,
    dst_x: i32,
    dst_y: i32,
    height: i32,
    glyph: &Glyph,
    colour: u16,
) -> i32 {
    let w = glyph.width;
    let h = height;
    if w <= 0 || h <= 0 || glyph.bitmap.is_empty() {
        return w.max(0);
    }
    // The exe's inclusive-bound rect for the glyph is
    // (dst_x, dst_y)..(dst_x+w-1, dst_y+h-1).
    let x1 = dst_x + w - 1;
    let y1 = dst_y + h - 1;
    // Unpack colour into 0..255 channels using the same mask arithmetic
    // as the exe (`((c & mask) << 8) / (mask + 1)`). These play the role
    // of `(uVar10 << 8) / uVar1` (glyph R), and same for G, B.
    let rm = surface.red_mask as u32;
    let gm = surface.green_mask as u32;
    let bm = surface.blue_mask as u32;
    let cu = colour as u32;
    let gr = ((cu & rm) << 8) / (rm + 1) & 0xff;
    let gg = ((cu & gm) << 8) / (gm + 1) & 0xff;
    let gb = ((cu & bm) << 8) / (bm + 1) & 0xff;

    // The `local_64` last-dest cache in the exe is a pure optimisation
    // (skip re-unpacking the destination when the pixel hasn't changed
    // between iterations) — dropped for clarity; the arithmetic is
    // identical without it.

    // Walk every glyph row, tracking the byte pointer + nibble parity
    // exactly as the exe does: high nibble on even column, low nibble
    // on odd column with an advance. `col` runs `dst_x..=dst_x+w-1`.
    let mut byte_ptr = 0usize;
    for row in 0..h {
        let y = dst_y + row;
        let row_inside = y >= 0 && y < surface.height;
        for col_off in 0..w {
            let x = dst_x + col_off;
            let parity = col_off & 1;
            let alpha = if parity == 0 {
                // High nibble; do NOT advance.
                if byte_ptr >= glyph.bitmap.len() {
                    return w;
                }
                glyph.bitmap[byte_ptr] >> 4
            } else {
                let n = glyph.bitmap[byte_ptr] & 0x0f;
                byte_ptr += 1;
                n
            };
            if alpha == 0 || !row_inside || x < 0 || x >= surface.width {
                continue;
            }
            let idx = (surface.pitch_pixels * y + x) as usize;
            if alpha == 0xf {
                surface.buf[idx] = colour;
                continue;
            }
            // Alpha-blend with the destination pixel (unpack, mix, repack).
            let dst = surface.buf[idx] as u32;
            let dr = ((dst & rm) << 8) / (rm + 1) & 0xff;
            let dg = ((dst & gm) << 8) / (gm + 1) & 0xff;
            let db = ((dst & bm) << 8) / (bm + 1) & 0xff;
            let a = alpha as u32;
            let ia = 15 - a;
            let mr = (gr * a + dr * ia) / 15;
            let mg = (gg * a + dg * ia) / 15;
            let mb = (gb * a + db * ia) / 15;
            surface.buf[idx] = surface.pack_rgb(mr as u8, mg as u8, mb as u8);
        }
        // Row end: if we finished on the high nibble (i.e. width odd),
        // advance to the next byte. `bVar25 = (col_off & 1) == 0` after
        // the loop — an even `w` leaves `col_off & 1 == 0` on exit AND
        // did advance on the last low nibble; an odd `w` leaves parity 1
        // and did NOT advance. So the exe's `if (!bVar25)` bumps by one
        // in the odd case only. Equivalent test on `w`:
        if (w & 1) != 0 {
            byte_ptr += 1;
        }
    }
    w
}

/// Draw a UTF-8-agnostic byte string (the exe walks bytes, not code
/// points — the shipped fonts are single-byte). Returns the pen's final
/// x. `text` is the exe's `param_5`; `colour` is `param_4`.
///
/// The exe's `|` → space substitution (unusable line-break marker) is
/// applied here. Characters below 0x20 are skipped without drawing —
/// matches the `if (0x1f < bVar7)` gate in the outer loop.
pub fn draw_text(
    surface: &mut PackedSurface,
    dst_x: i32,
    dst_y: i32,
    font: &PixelFont,
    text: &[u8],
    colour: u16,
) -> i32 {
    let mut pen = dst_x;
    for &raw in text {
        let b = if raw == b'|' { b' ' } else { raw };
        if b < 0x20 {
            continue;
        }
        if let Some(Some(g)) = font.glyphs.get(b as usize) {
            let advance = draw_glyph(surface, pen, dst_y, font.height, g, colour);
            pen += advance;
        }
    }
    pen
}

#[cfg(test)]
mod tests {
    use super::*;

    // Solid 2x2 glyph: 4 nibbles = 0xff, 0xff → 1 byte per row × 2 rows.
    fn solid_2x2() -> Glyph {
        Glyph { width: 2, bitmap: vec![0xff, 0xff] }
    }

    // Empty 2x2 glyph: all-zero.
    fn empty_2x2() -> Glyph {
        Glyph { width: 2, bitmap: vec![0x00, 0x00] }
    }

    #[test]
    fn solid_glyph_writes_colour_verbatim() {
        let mut s = PackedSurface::rgb555(4, 4);
        let c = s.pack_rgb(0xff, 0, 0);
        let g = solid_2x2();
        draw_glyph(&mut s, 1, 1, 2, &g, c);
        let z = 0u16;
        let expected: [u16; 16] = [
            z, z, z, z,
            z, c, c, z,
            z, c, c, z,
            z, z, z, z,
        ];
        assert_eq!(s.buf.as_slice(), &expected);
    }

    #[test]
    fn transparent_glyph_leaves_surface_unchanged() {
        let mut s = PackedSurface::rgb555(4, 4);
        let bg = s.pack_rgb(0, 0x80, 0);
        s.buf.iter_mut().for_each(|v| *v = bg);
        let red = s.pack_rgb(0xff, 0, 0);
        draw_glyph(&mut s, 1, 1, 2, &empty_2x2(), red);
        assert!(s.buf.iter().all(|&v| v == bg));
    }

    #[test]
    fn odd_width_glyph_advances_byte_pointer_at_row_end() {
        // 3-wide × 2-tall — odd width, so at each row end the byte
        // pointer advances (unused low nibble becomes padding).
        //   row 0 nibbles = f, f, 0  →  bytes 0..2 = 0xff, 0x00 (pad),
        //   row 1 nibbles = 0, f, f  →  bytes 2..4 = 0x0f, 0xf0 (pad).
        let g = Glyph { width: 3, bitmap: vec![0xff, 0x00, 0x0f, 0xf0] };
        let mut s = PackedSurface::rgb555(4, 3);
        let c = s.pack_rgb(0, 0, 0xff);
        draw_glyph(&mut s, 0, 0, 2, &g, c);
        // Expected:
        //   row 0: c c _ _
        //   row 1: _ c c _
        //   row 2: _ _ _ _
        let z = 0u16;
        let expected: [u16; 12] = [
            c, c, z, z,
            z, c, c, z,
            z, z, z, z,
        ];
        assert_eq!(s.buf.as_slice(), &expected);
    }

    #[test]
    fn half_alpha_blends_toward_glyph_colour() {
        // 1×1 glyph with alpha nibble = 8/15 (~53%). Destination white,
        // glyph colour black → result ≈ (0*8 + 255*7)/15 = 119 per
        // channel, then repacked in RGB555.
        let g = Glyph { width: 1, bitmap: vec![0x80] };
        let mut s = PackedSurface::rgb555(1, 1);
        let white = s.pack_rgb(0xff, 0xff, 0xff);
        s.buf[0] = white;
        let black = s.pack_rgb(0, 0, 0);
        draw_glyph(&mut s, 0, 0, 1, &g, black);
        // The blended value must sit strictly between black and white.
        assert_ne!(s.buf[0], black);
        assert_ne!(s.buf[0], white);
        // And be closer to black than white because alpha=8 slightly
        // favours the glyph colour.
        let unpack_r = |v: u16| ((v as u32 & 0x7c00) << 8) / (0x7c00 + 1) & 0xff;
        let mid = unpack_r(s.buf[0]);
        assert!(mid < 128 && mid > 100, "R channel expected ~119, got {mid}");
    }

    #[test]
    fn draw_text_walks_bytes_and_substitutes_pipe_for_space() {
        // Font with only glyphs for '.' (width 1, solid) and space (width 3, empty).
        let dot = Glyph { width: 1, bitmap: vec![0xf0] };
        let sp = Glyph { width: 3, bitmap: vec![0x00, 0x00] };
        let mut font = PixelFont::empty(1);
        font.glyphs[b'.' as usize] = Some(dot);
        font.glyphs[b' ' as usize] = Some(sp);
        let mut s = PackedSurface::rgb555(10, 1);
        let c = s.pack_rgb(0xff, 0, 0);
        let end = draw_text(&mut s, 0, 0, &font, b".|.", c);
        // pen path: 0 -> +1 (.) -> +3 (| == space) -> +1 (.) = 5.
        assert_eq!(end, 5);
        // Pixel 0 and 4 should be `c`; the rest zero.
        assert_eq!(s.buf[0], c);
        assert_eq!(s.buf[4], c);
        assert!(s.buf[1..4].iter().all(|&v| v == 0));
        assert!(s.buf[5..].iter().all(|&v| v == 0));
    }
}
