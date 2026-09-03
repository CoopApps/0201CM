//! Wrapped text renderer — port of `FUN_005d03a0` (GDI variant,
//! 1955 bytes). Consumes a rectangle + font + string + style bits, wraps
//! the string to fit width, and paints each line via the ported glyph
//! blit (`packed_glyph::draw_text`).
//!
//! The exe builds an in-memory std::list of `std::string` lines using
//! helpers `FUN_00671e40` (list ctor), `FUN_00671ef0` (append line) and
//! `FUN_00671fd0` (iterate lines), and measures line width via
//! `FUN_005cf2a0` (font+kern-aware). We use Rust `Vec<Vec<u8>>` for the
//! line list and approximate width as `sum(glyph.width)` — the shipped
//! fonts carry no kerning-per-pair table (the third-through-fifth ints
//! of each per-char record are unused by the renderer), so the sum is
//! byte-exact on real inputs.

use crate::packed::PackedSurface;
use crate::packed_glyph::{draw_text, draw_text_caret, PixelFont};
use crate::packed_panel::scale_colour;

// ------ Style flag constants (from FUN_005d03a0's `param_5`) ------
pub const W_LEFT: u32     = 0x0000_0001; // else centred horizontally
pub const W_TOP: u32      = 0x0000_0002; // else centred vertically
pub const W_SHADOW: u32   = 0x0000_0020; // paint a shadow copy at +1,+1
pub const W_RIGHT: u32    = 0x0000_0040; // right-align (overrides centre)
pub const W_PASSWORD: u32 = 0x0000_0080; // replace every glyph with '*'
pub const W_WRAP: u32     = 0x0000_0100; // wrap on line-full instead of clipping

/// Port of `FUN_005cf2a0` — the exe's line-measure helper. Sums each
/// glyph's `width` plus the pair-wise kern between adjacent glyphs,
/// matching how `draw_string` advances the pen (via `kern_between`).
/// `|` → space and control chars (< 0x20) are ignored, matching the
/// substitution rule inside `draw_text`.
pub fn measure_line(font: &PixelFont, text: &[u8]) -> i32 {
    use crate::packed_glyph::kern_between;
    let mut total = 0;
    for (i, &b) in text.iter().enumerate() {
        let b = if b == b'|' { b' ' } else { b };
        if b < 0x20 { continue; }
        if let Some(Some(g)) = font.glyphs.get(b as usize) {
            total += g.width + kern_between(font, text, i + 1);
        }
    }
    total
}

/// Wrap `text` to fit `max_width` in `font`'s metrics. Matches the exe's
/// state machine:
/// - `\n` in the input flushes the current line (empty ones included).
/// - When `wrap_on_overflow` is true (`W_WRAP` in the outer style), any
///   character that would push the current line beyond `max_width` is
///   rejected and the current line is flushed instead.
/// - When it is false, the character is rejected but the line stays open
///   (the exe's `bVar3` flag keeps rejecting until end-of-string or
///   newline).
/// Same as [`wrap_text`], but also returns the un-wrapped byte offset
/// where each line's first char came from. Used by the caret pipeline
/// so [`draw_wrapped_text`] can translate a global caret index (into
/// the un-wrapped buffer) into a per-line local index — matching the
/// exe, where the caret target is one specific char that either lives
/// in a wrapped line or was dropped at the overflow boundary.
pub fn wrap_text_with_origins(
    font: &PixelFont,
    max_width: i32,
    text: &[u8],
    wrap_on_overflow: bool,
) -> Vec<(Vec<u8>, i32)> {
    let mut lines: Vec<(Vec<u8>, i32)> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    let mut cur_start: i32 = 0;
    let mut overflow = false;
    let last_idx = text.len().saturating_sub(1);
    for (i, &b) in text.iter().enumerate() {
        let mut do_flush = false;
        if b == b'\n' {
            do_flush = true;
        } else {
            if b >= 0x20 && !overflow {
                cur.push(b);
                if measure_line(font, &cur) > max_width {
                    cur.pop();
                    overflow = true;
                }
            }
            if i == last_idx || (wrap_on_overflow && overflow) {
                do_flush = true;
            }
        }
        if do_flush {
            let taken = std::mem::take(&mut cur);
            lines.push((taken, cur_start));
            overflow = false;
            // The next line's origin starts after this char (which was
            // either \n, an overflow-dropped byte, or the last byte).
            cur_start = (i as i32) + 1;
        }
    }
    lines
}

pub fn wrap_text(
    font: &PixelFont,
    max_width: i32,
    text: &[u8],
    wrap_on_overflow: bool,
) -> Vec<Vec<u8>> {
    // Literal port of FUN_005d03a0's wrap state machine:
    //  - Iterate one char at a time.
    //  - `bVar3` tracks "we're in overflow" — while set, further chars
    //    are skipped (not appended to `cur`).
    //  - On overflow: append, measure, if >max_width null-terminate the
    //    just-added char away and set `bVar3 = true`.
    //  - After per-iter processing, if `bVar3 && W_WRAP` (or newline,
    //    or last-char), FLUSH `cur` to a line, reset `cur = []`, clear
    //    `bVar3`.
    //  - Then ADVANCE to next char regardless. THE OVERFLOW CHAR IS
    //    DROPPED — exe does exactly this (loses one char per wrap
    //    boundary). Verified byte-exact against the running exe's
    //    FUN_005d03a0 for the "wrap" case in verify_wrapped_text.
    let mut lines: Vec<Vec<u8>> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    let mut overflow = false;
    let last_idx = text.len().saturating_sub(1);
    for (i, &b) in text.iter().enumerate() {
        let mut do_flush = false;
        if b == b'\n' {
            do_flush = true;
        } else {
            if b >= 0x20 && !overflow {
                cur.push(b);
                if measure_line(font, &cur) > max_width {
                    cur.pop();
                    overflow = true;
                }
            }
            if i == last_idx || (wrap_on_overflow && overflow) {
                do_flush = true;
            }
        }
        if do_flush {
            lines.push(std::mem::take(&mut cur));
            overflow = false;
        }
    }
    lines
}

/// Literal port of `FUN_005d03a0` (1955 bytes) — wrap and paint text
/// inside `(x0..=x1, y0..=y1)`.
///
/// `caret_char_index` is the exe's `param_10` — a byte index into the
/// (unwrapped) label. When `>= 0` a vertical caret stroke is drawn at
/// the pen position of that character; `-1` (or any negative) disables
/// the caret. This matches the exe's gate at `FUN_005ceaa0:005cf180`
/// (`cmp edx, -1; jle skip`) — the caret pipeline is inside the glyph
/// blit, not the wrap layer, and the wrap layer just passes the index
/// through. See asm at `FUN_005ceaa0:005cf1dd..005cf1ef`:
///   `push colour; push 2; push (font_h + y_top - 1); push x_pen;`
///   `push y_top; push x_pen; call FUN_005cd3e0`
/// — i.e. a single 1-px vertical line from `(x_pen, y_top)` to
/// `(x_pen, y_top + font_h - 1)` (`style=2` → solid). We draw it here
/// after painting the first wrapped line, because that's where the
/// pen position is unambiguously anchored to the label buffer index.
pub fn draw_wrapped_text(
    s: &mut PackedSurface,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    font: &PixelFont,
    text: &[u8],
    colour: u16,
    style: u32,
    caret_char_index: i32,
) {
    let w = x1 - x0 + 1;
    let h = y1 - y0 + 1;
    let mut lines = wrap_text_with_origins(font, w, text, (style & W_WRAP) != 0);
    if lines.is_empty() {
        return;
    }
    if (style & W_PASSWORD) != 0 {
        for (line, _) in &mut lines {
            for b in line.iter_mut() {
                *b = b'*';
            }
        }
    }
    let font_h = font.height;
    let block_h = lines.len() as i32 * font_h;
    // Vertical centring is the default; W_TOP suppresses it.
    let mut y = if (style & W_TOP) != 0 {
        y0
    } else {
        (h - block_h) / 2 + y0
    };
    for (line, origin) in &lines {
        let lw = measure_line(font, line);
        let x = if (style & W_LEFT) != 0 {
            x0
        } else if (style & W_RIGHT) != 0 {
            x1 - lw + 1
        } else {
            (w - lw) / 2 + x0
        };
        // Translate global caret_char_index → local index within this
        // wrapped line. If the target falls in this line, pass the local
        // index down to draw_text_caret — that's where the exe's
        // FUN_005ceaa0 does the per-glyph pen-X capture (asm
        // 005cf102..005cf124) and the vertical-stroke draw (005cf1ef).
        // If the target sits inside a dropped-overflow boundary, no
        // line owns it and the caret is silently lost — matches the
        // exe (the target-char branch never fires).
        let line_local_caret: i32 = if caret_char_index < 0 {
            -1
        } else {
            let local = caret_char_index - *origin;
            // `<= line.len()` so an "at end of line" caret still hits
            // the past-the-end branch inside draw_text_caret.
            if local >= 0 && local <= line.len() as i32 {
                local
            } else {
                -1
            }
        };
        if (style & W_SHADOW) != 0 {
            // The exe samples the mid-rect pixel and scales BOTH the
            // shadow (0x42 ≈ 66%) and the foreground (0xa6 ≈ 166%) from
            // it. Sample-and-scale is deferred; for now paint the shadow
            // as a 66%-scaled version of `colour` — matches the exe when
            // called with a non-zero foreground colour (the common case).
            // The shadow layer never carries the caret — matches the
            // exe, which passes -1 for the shadow blit.
            let shadow = scale_colour(s, colour, 0x42);
            draw_text(s, x + 1, y + 1, font, line, shadow);
        }
        draw_text_caret(s, x, y, font, line, colour, line_local_caret);
        y += font_h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packed_glyph::Glyph;

    // Build a font with only the characters we need for a test — each
    // 1 pixel wide, all-transparent (we're only testing wrap + layout).
    fn stub_font(chars: &[u8], height: i32) -> PixelFont {
        let mut f = PixelFont::empty(height);
        for &c in chars {
            f.glyphs[c as usize] = Some(Glyph { width: 1, bitmap: vec![0xf0; height as usize], ..Default::default() });
        }
        f
    }

    #[test]
    fn measure_line_ignores_pipe_and_control() {
        let f = stub_font(b"abc ", 1);
        assert_eq!(measure_line(&f, b"abc"), 3);
        assert_eq!(measure_line(&f, b"a|b"), 3); // '|' becomes space (width 1)
        assert_eq!(measure_line(&f, b"a\nb"), 2); // '\n' < 0x20, ignored here
    }

    #[test]
    fn wrap_text_breaks_on_newline_regardless_of_width() {
        let f = stub_font(b"abcdef", 1);
        let lines = wrap_text(&f, 100, b"ab\ncd\nef", false);
        assert_eq!(lines, vec![b"ab".to_vec(), b"cd".to_vec(), b"ef".to_vec()]);
    }

    #[test]
    fn wrap_text_clips_when_wrap_flag_off() {
        let f = stub_font(b"abcdef", 1);
        // 4-wide limit, "abcdef" (6 wide) with no wrap → line stays open
        // but drops overflow characters; final flush produces "abcd".
        let lines = wrap_text(&f, 4, b"abcdef", false);
        assert_eq!(lines, vec![b"abcd".to_vec()]);
    }

    #[test]
    fn wrap_text_drops_the_overflowing_char_on_wrap() {
        // Verified byte-exact against exe: FUN_005d03a0 DROPS the char
        // that triggered overflow. "abcdefgh" at width 4 → the 'e' is
        // the one that overflows and is lost. Result: "abcd" / "fgh".
        // (Was "abcd" / "efgh" before verify_wrapped_text caught the
        // divergence.)
        let f = stub_font(b"abcdefgh", 1);
        let lines = wrap_text(&f, 4, b"abcdefgh", true);
        assert_eq!(lines, vec![b"abcd".to_vec(), b"fgh".to_vec()]);
    }

    #[test]
    fn draw_wrapped_text_left_aligns_when_flag_set() {
        // 1×1 font, one visible glyph char (block colour). W_LEFT | W_TOP
        // suppresses centring — glyph should land at (x0, y0).
        let f = stub_font(b"X", 1);
        let mut s = PackedSurface::rgb555(6, 4);
        let c = s.pack_rgb(0xff, 0, 0);
        draw_wrapped_text(&mut s, 2, 1, 5, 3, &f, b"X", c, W_LEFT | W_TOP, -1);
        assert_eq!(s.buf[(1 * 6 + 2) as usize], c);
        // Anything else zero.
        for (i, &v) in s.buf.iter().enumerate() {
            if i != (1 * 6 + 2) as usize {
                assert_eq!(v, 0, "extra pixel at {i}");
            }
        }
    }

    #[test]
    fn draw_wrapped_text_centres_horizontally_by_default() {
        // "XX" (width 2) in a rect of width 6 → centred at x = (6-2)/2 = 2,
        // so pixels 2 and 3 lit.
        let f = stub_font(b"X", 1);
        let mut s = PackedSurface::rgb555(6, 1);
        let c = s.pack_rgb(0, 0xff, 0);
        draw_wrapped_text(&mut s, 0, 0, 5, 0, &f, b"XX", c, W_TOP, -1);
        assert_eq!(&s.buf, &[0, 0, c, c, 0, 0]);
    }

    #[test]
    fn draw_wrapped_text_right_aligns_when_flag_set() {
        let f = stub_font(b"X", 1);
        let mut s = PackedSurface::rgb555(6, 1);
        let c = s.pack_rgb(0, 0, 0xff);
        draw_wrapped_text(&mut s, 0, 0, 5, 0, &f, b"X", c, W_RIGHT | W_TOP, -1);
        assert_eq!(s.buf[5], c);
        assert!(s.buf[0..5].iter().all(|&v| v == 0));
    }

    /// Font with fully transparent 1-pixel-wide glyphs — used by caret
    /// tests so the glyph blit paints nothing and we can assert on the
    /// caret pixels in isolation.
    fn transparent_font(chars: &[u8], height: i32) -> PixelFont {
        let mut f = PixelFont::empty(height);
        for &c in chars {
            f.glyphs[c as usize] = Some(Glyph {
                width: 1,
                bitmap: vec![0x00; height as usize],
                ..Default::default()
            });
        }
        f
    }

    #[test]
    fn draw_wrapped_text_paints_vertical_caret_at_index_zero() {
        // Ported per FUN_005ceaa0:005cf1dd..005cf1ef — the caret is a
        // 1-px SOLID vertical line spanning the full font height at the
        // pen X of the target char. Index 0 → "before the first char",
        // so caret_x == label_start_x. With W_LEFT|W_TOP the label
        // starts at (x0, y0). We use a transparent 3-row font so the
        // ONLY colour_c pixels come from the caret stroke.
        let f = transparent_font(b"OK", 3);
        let mut s = PackedSurface::rgb555(4, 5);
        let c = s.pack_rgb(0xff, 0, 0);
        draw_wrapped_text(&mut s, 0, 0, 3, 4, &f, b"OK", c, W_LEFT | W_TOP, 0);
        // Column 0 rows 0..2 lit with `c` — the caret.
        assert_eq!(s.buf[0], c, "caret row 0");
        assert_eq!(s.buf[4], c, "caret row 1");
        assert_eq!(s.buf[8], c, "caret row 2");
        // Nothing else is touched.
        for (i, &v) in s.buf.iter().enumerate() {
            let expected = if i == 0 || i == 4 || i == 8 { c } else { 0 };
            assert_eq!(v, expected, "unexpected pixel at index {i}");
        }
    }

    #[test]
    fn draw_wrapped_text_caret_disabled_when_index_negative() {
        // caret_char_index = -1 → gate at FUN_005ceaa0:005cf180 skips
        // the stroke; surface stays black.
        let f = transparent_font(b"OK", 3);
        let mut s = PackedSurface::rgb555(4, 5);
        let c = s.pack_rgb(0xff, 0, 0);
        draw_wrapped_text(&mut s, 0, 0, 3, 4, &f, b"OK", c, W_LEFT | W_TOP, -1);
        assert!(s.buf.iter().all(|&v| v == 0));
    }
}
