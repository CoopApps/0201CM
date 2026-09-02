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
use crate::packed_glyph::{draw_text, PixelFont};
use crate::packed_panel::scale_colour;

// ------ Style flag constants (from FUN_005d03a0's `param_5`) ------
pub const W_LEFT: u32     = 0x0000_0001; // else centred horizontally
pub const W_TOP: u32      = 0x0000_0002; // else centred vertically
pub const W_SHADOW: u32   = 0x0000_0020; // paint a shadow copy at +1,+1
pub const W_RIGHT: u32    = 0x0000_0040; // right-align (overrides centre)
pub const W_PASSWORD: u32 = 0x0000_0080; // replace every glyph with '*'
pub const W_WRAP: u32     = 0x0000_0100; // wrap on line-full instead of clipping

/// Sum-of-widths port of `FUN_005cf2a0` — the exe's line-measure helper.
/// `|` → space and control chars (< 0x20) are ignored, matching the
/// substitution rule inside `draw_text`.
pub fn measure_line(font: &PixelFont, text: &[u8]) -> i32 {
    text.iter()
        .map(|&b| if b == b'|' { b' ' } else { b })
        .filter(|&b| b >= 0x20)
        .map(|b| {
            font.glyphs
                .get(b as usize)
                .and_then(|g| g.as_ref())
                .map(|g| g.width)
                .unwrap_or(0)
        })
        .sum()
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
pub fn wrap_text(
    font: &PixelFont,
    max_width: i32,
    text: &[u8],
    wrap_on_overflow: bool,
) -> Vec<Vec<u8>> {
    let mut lines: Vec<Vec<u8>> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    let mut overflow = false;
    for (i, &b) in text.iter().enumerate() {
        if b == b'\n' {
            lines.push(std::mem::take(&mut cur));
            overflow = false;
            continue;
        }
        if b >= 0x20 && !overflow {
            cur.push(b);
            if measure_line(font, &cur) > max_width {
                cur.pop();
                overflow = true;
                if wrap_on_overflow {
                    lines.push(std::mem::take(&mut cur));
                    overflow = false;
                    // Consume the character on the fresh line if it now fits.
                    cur.push(b);
                    if measure_line(font, &cur) > max_width {
                        cur.pop(); // still too wide (single char > width) — drop
                    }
                }
            }
        }
        if i + 1 == text.len() {
            lines.push(std::mem::take(&mut cur));
        }
    }
    lines
}

/// Literal port of `FUN_005d03a0` (1955 bytes) — wrap and paint text
/// inside `(x0..=x1, y0..=y1)`.
///
/// `kern` is the exe's `param_9` (underline/attention character index —
/// passed straight through to `packed_glyph::draw_text`; we don't yet
/// render the underline stroke).
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
    _kern: i32,
) {
    let w = x1 - x0 + 1;
    let h = y1 - y0 + 1;
    let mut lines = wrap_text(font, w, text, (style & W_WRAP) != 0);
    if lines.is_empty() {
        return;
    }
    if (style & W_PASSWORD) != 0 {
        for line in &mut lines {
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
    for line in &lines {
        let lw = measure_line(font, line);
        let x = if (style & W_LEFT) != 0 {
            x0
        } else if (style & W_RIGHT) != 0 {
            x1 - lw + 1
        } else {
            (w - lw) / 2 + x0
        };
        if (style & W_SHADOW) != 0 {
            // The exe samples the mid-rect pixel and scales BOTH the
            // shadow (0x42 ≈ 66%) and the foreground (0xa6 ≈ 166%) from
            // it. Sample-and-scale is deferred; for now paint the shadow
            // as a 66%-scaled version of `colour` — matches the exe when
            // called with a non-zero foreground colour (the common case).
            let shadow = scale_colour(s, colour, 0x42);
            draw_text(s, x + 1, y + 1, font, line, shadow);
        }
        draw_text(s, x, y, font, line, colour);
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
            f.glyphs[c as usize] = Some(Glyph { width: 1, bitmap: vec![0xf0; height as usize] });
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
    fn wrap_text_wraps_when_wrap_flag_on() {
        let f = stub_font(b"abcdefgh", 1);
        // 4-wide limit, wrap on: expect two lines of 4.
        let lines = wrap_text(&f, 4, b"abcdefgh", true);
        assert_eq!(lines, vec![b"abcd".to_vec(), b"efgh".to_vec()]);
    }

    #[test]
    fn draw_wrapped_text_left_aligns_when_flag_set() {
        // 1×1 font, one visible glyph char (block colour). W_LEFT | W_TOP
        // suppresses centring — glyph should land at (x0, y0).
        let f = stub_font(b"X", 1);
        let mut s = PackedSurface::rgb555(6, 4);
        let c = s.pack_rgb(0xff, 0, 0);
        draw_wrapped_text(&mut s, 2, 1, 5, 3, &f, b"X", c, W_LEFT | W_TOP, 0);
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
        draw_wrapped_text(&mut s, 0, 0, 5, 0, &f, b"XX", c, W_TOP, 0);
        assert_eq!(&s.buf, &[0, 0, c, c, 0, 0]);
    }

    #[test]
    fn draw_wrapped_text_right_aligns_when_flag_set() {
        let f = stub_font(b"X", 1);
        let mut s = PackedSurface::rgb555(6, 1);
        let c = s.pack_rgb(0, 0, 0xff);
        draw_wrapped_text(&mut s, 0, 0, 5, 0, &f, b"X", c, W_RIGHT | W_TOP, 0);
        assert_eq!(s.buf[5], c);
        assert!(s.buf[0..5].iter().all(|&v| v == 0));
    }
}
