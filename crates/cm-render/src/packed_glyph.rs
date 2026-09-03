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

/// One glyph — width in pixels + a 4-bit nibble-packed bitmap.
///
/// The exe's char record is 5 ints wide: `[width, ?, kern_b, kern_c,
/// bitmap_ptr]`. We name the two "unknown" middle ints per their use
/// in the pair-wise kerning function `FUN_005cf4d0`:
/// - `kern_b` is subtracted from the base when THIS char is the
///   next-to-be-drawn (i.e. it's the "left bearing"-like value).
/// - `kern_c` is subtracted from the base when THIS char was the
///   previous char (i.e. it's the "right bearing"-like value).
/// The first "unknown" int (unk_a) is not read by any traditional-
/// font code path we've decoded; kept as raw for future use.
///
/// Row bytes = `ceil(width / 2)` — rows do NOT share bytes across
/// boundaries; a row that ends on the high nibble has one byte of
/// padding. (Was `ceil(width * height / 2)` earlier — that under-reads
/// for odd widths, so 'l' and '.' lost their bottom rows.)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Glyph {
    pub width: i32,
    /// `unk_a` in the record; not touched by draw or kern in the
    /// traditional-font path. Kept so a full re-export matches disk.
    pub kern_a: i32,
    /// `unk_b` in the record; subtracted from kern base when this
    /// char is the next-to-draw.
    pub kern_b: i32,
    /// `unk_c` in the record; subtracted from kern base when this
    /// char was the previous-drawn.
    pub kern_c: i32,
    /// Row-major, high-nibble first, one byte per (width/2 rounded up)
    /// pixel-pair, no cross-row nibble sharing.
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

    /// Space (`0x20`) glyph's width. Used by `kern_between` as the
    /// base for the (space_width * 3) / 4 baseline. Exe reads this at
    /// `*(font_base + 0x284)` — the first int of char record 0x20 —
    /// which IS the space glyph's `width` field.
    pub fn space_width(&self) -> i32 {
        self.glyphs.get(0x20).and_then(|g| g.as_ref()).map(|g| g.width).unwrap_or(0)
    }
}

/// Byte-exact port of `FUN_005cf4d0` (152 bytes, 54 instructions) —
/// the pair-wise kerning function. Called between adjacent glyphs by
/// the exe's `draw_string` and added to the current pen along with the
/// just-drawn glyph's width. Returns 0 at end-of-string / null-string /
/// font -1 / negative result.
///
/// Formula: `max(0, (space_width * 3) / 4 - curr.kern_b - prev.kern_c)`.
/// The `|` → space (`0x20`) substitution matches the exe's asm at
/// `005cf512..005cf526`.
pub fn kern_between(font: &PixelFont, text: &[u8], idx: usize) -> i32 {
    if idx == 0 || idx > text.len() {
        return 0;
    }
    let curr = text.get(idx).copied().unwrap_or(0);
    if curr == 0 {
        return 0;
    }
    let prev = text[idx - 1];
    let sub = |b: u8| -> u8 { if b == b'|' { b' ' } else { b } };
    let curr = sub(curr) as usize;
    let prev = sub(prev) as usize;
    let space_w = font.space_width();
    let base = (space_w * 3) / 4;
    let curr_kb = font.glyphs.get(curr).and_then(|g| g.as_ref()).map(|g| g.kern_b).unwrap_or(0);
    let prev_kc = font.glyphs.get(prev).and_then(|g| g.as_ref()).map(|g| g.kern_c).unwrap_or(0);
    (base - curr_kb - prev_kc).max(0)
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
///
/// Caret-less variant. New code should call [`draw_text_caret`], which
/// carries the exe's caret arg (`param_10` on the wrapped-text call
/// tree; FUN_005ceaa0 records it inside the per-glyph loop). This thin
/// wrapper stays for existing callers that never use the caret.
pub fn draw_text(
    surface: &mut PackedSurface,
    dst_x: i32,
    dst_y: i32,
    font: &PixelFont,
    text: &[u8],
    colour: u16,
) -> i32 {
    draw_text_caret(surface, dst_x, dst_y, font, text, colour, -1)
}

/// Draw one line of text with the exe's per-glyph caret pipeline.
///
/// `caret_char_index` is the exe's `[esp+0xa8]` slot inside
/// `FUN_005ceaa0` — a 0-based byte index into the passed-in `text`
/// slice. When the loop reaches that index it records the pen X into
/// the caret slot; after the loop (at asm 005cf17c..005cf1ef) the
/// recorded X drives a single vertical solid line from
/// `(caret_x, dst_y)` to `(caret_x, dst_y + font.height - 1)`.
///
/// `-1` (or any negative) disables the caret via the exe's gate at
/// 005cf180: `cmp edx, -1; jle skip`.
///
/// Faithfulness citations:
/// * 005cf102..005cf124 — in-loop caret-X capture:
///     ebx = current-char index (1-based, bumped at 005cf15b).
///     eax = caret_char_index + 1.
///     if (ebx == eax) [esp+0x18] = min(prev_pen, new_pen - 1).
/// * 005cf16c..005cf178 — past-the-end capture after loop exit:
///     if (ebx == caret_char_index + 1) [esp+0x18] = pen.
/// * 005cf17c..005cf1ef — vertical-stroke draw when [esp+0x18] > -1.
pub fn draw_text_caret(
    surface: &mut PackedSurface,
    dst_x: i32,
    dst_y: i32,
    font: &PixelFont,
    text: &[u8],
    colour: u16,
    caret_char_index: i32,
) -> i32 {
    // Match the exe's outer loop in FUN_005ceaa0: iterate chars, draw
    // each glyph, then advance pen by `kern_between(prev, curr) +
    // glyph.width`. The exe starts iVar21 (char index) at 1 and
    // increments; the kern call uses iVar21 as the index for
    // `string[idx]` = the just-drawn char, `string[idx-1]` is
    // meaningless on the very first call — but exe reads it anyway
    // and gets whatever's in memory. In practice callers ensure a
    // clean prev so first char's kern is well-defined. We use idx
    // starting at 1 (matches exe) and pass the string; kern_between
    // does the same bytes-based lookback.
    let mut pen = dst_x;
    // Exe: [esp+0x18] init to -1 (no capture); asm 005cf17c reads it.
    let mut caret_x: i32 = -1;
    // Exe: ebx starts at 1 (1-based char count); [esp+0xa8] holds the
    // caller's caret index; the compare is `ebx == caret_char_index+1`.
    // Translated: 0-based index of the char we just processed.
    let mut char_count: i32 = 0;
    for (i, &raw) in text.iter().enumerate() {
        let b = if raw == b'|' { b' ' } else { raw };
        if b < 0x20 {
            continue;
        }
        if let Some(Some(g)) = font.glyphs.get(b as usize) {
            let prev_pen = pen;
            let advance = draw_glyph(surface, pen, dst_y, font.height, g, colour);
            let k = kern_between(font, text, i + 1);
            let new_pen = prev_pen + advance + k;
            // 005cf102..005cf124: caret capture inside the loop —
            //   ecx = prev_pen (old edi at loop head)
            //   eax = new_pen - 1
            //   [esp+0x18] = min(ecx, eax)
            // Guard: exe only takes this branch when target_char just
            // drew, i.e. char_count == caret_char_index (0-based).
            if caret_char_index >= 0 && char_count == caret_char_index {
                let clamp_hi = new_pen.wrapping_sub(1);
                caret_x = if prev_pen > clamp_hi { clamp_hi } else { prev_pen };
            }
            pen = new_pen;
            char_count += 1;
        }
    }
    // 005cf16c..005cf178: past-the-end capture — after loop exits (NUL
    // terminator or end-of-slice) if we haven't already latched the
    // caret and the caller asked for a caret AT the end, record the
    // final pen.
    if caret_char_index >= 0 && char_count == caret_char_index {
        caret_x = pen;
    }
    // 005cf180: `cmp edx, -1; jle skip` — draw only when caret_x > -1.
    if caret_x > -1 {
        // 005cf1e7: `lea eax, [eax + ecx - 1]` — y_bottom = y_top + font_h - 1.
        // 005cf1ef: draw_line(caret_x, y_top, caret_x, y_bottom, style=2).
        surface.draw_line(caret_x, dst_y, caret_x, dst_y + font.height - 1, 2, colour);
    }
    pen
}

#[cfg(test)]
mod tests {
    use super::*;

    // Solid 2x2 glyph: 4 nibbles = 0xff, 0xff → 1 byte per row × 2 rows.
    fn solid_2x2() -> Glyph {
        Glyph { width: 2, bitmap: vec![0xff, 0xff], ..Default::default() }
    }

    // Empty 2x2 glyph: all-zero.
    fn empty_2x2() -> Glyph {
        Glyph { width: 2, bitmap: vec![0x00, 0x00], ..Default::default() }
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
        let g = Glyph { width: 3, bitmap: vec![0xff, 0x00, 0x0f, 0xf0], ..Default::default() };
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
        let g = Glyph { width: 1, bitmap: vec![0x80], ..Default::default() };
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
        let dot = Glyph { width: 1, bitmap: vec![0xf0], ..Default::default() };
        let sp = Glyph { width: 3, bitmap: vec![0x00, 0x00], ..Default::default() };
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
