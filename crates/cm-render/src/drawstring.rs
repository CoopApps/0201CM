//! Text-layout wrapper — direct port of `FUN_005D0870` (12,893 bytes
//! decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_primitives/0x005d0870.c`.
//! Full decode: [`reports/gui_primitives_decode.md`](../../../../reports/gui_primitives_decode.md) §11.
//!
//! `FUN_005D0870` does NOT rasterize glyphs — that's `FUN_005CED50`
//! (still needs decompile). This module handles the wrapper: word-wrap,
//! alignment, password-mask, and drop-shadow colour derivation. The
//! actual glyph blit is delegated back through a closure.
//!
//! # Flag word
//!
//! | Bit   | Effect                                            |
//! |-------|---------------------------------------------------|
//! | 0x001 | left-align (skip horizontal centering)            |
//! | 0x002 | top-align (skip vertical centering)               |
//! | 0x020 | drop-shadow (2-layer emboss)                      |
//! | 0x040 | right-align                                       |
//! | 0x080 | password mask (replace each char with '*')        |
//! | 0x100 | word-wrap at width                                |

use crate::bevel::{scale_color_by_percent, PCT_66, PCT_166};

pub mod flag {
    pub const LEFT_ALIGN: u32 = 0x001;
    pub const TOP_ALIGN: u32 = 0x002;
    pub const DROP_SHADOW: u32 = 0x020;
    pub const RIGHT_ALIGN: u32 = 0x040;
    pub const PASSWORD_MASK: u32 = 0x080;
    pub const WORD_WRAP: u32 = 0x100;
}

/// The complete plan for one drawstring call — a sequence of glyph
/// blits at exact (x, y) with the exact colour the exe computes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawStringPlan {
    /// One entry per line after wrap + shadow expansion.
    pub blits: Vec<GlyphBlit>,
}

/// One call to the real glyph blitter (`FUN_005CED50`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphBlit {
    pub x: i32,
    pub y: i32,
    pub font_id: i16,
    pub color: u16,
    pub text: String,
    pub extra: i32,
}

/// Break a string into lines. Word-wrap at `max_width` if `wrap`; else
/// only at explicit `\n`. Returns each line's text.
///
/// Ports the exe's inner loop: append chars ≥ 0x20; measure after each;
/// if measured > `max_width` and wrap → force break.
pub fn wrap_lines(
    text: &str,
    max_width: i32,
    wrap: bool,
    mut measure: impl FnMut(&str) -> i32,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut last_space: Option<usize> = None;
    for ch in text.chars() {
        if ch == '\n' {
            lines.push(std::mem::take(&mut current));
            last_space = None;
            continue;
        }
        if (ch as u32) < 0x20 { continue; }   // skip control chars
        current.push(ch);
        if ch == ' ' { last_space = Some(current.len() - 1); }
        if wrap && measure(&current) > max_width {
            // Split at last space if any, else hard break.
            if let Some(idx) = last_space {
                let rest = current[idx + 1..].to_string();
                current.truncate(idx);
                lines.push(std::mem::take(&mut current));
                current = rest;
                last_space = None;
            } else if current.len() > 1 {
                let last = current.pop().unwrap();
                lines.push(std::mem::take(&mut current));
                current.push(last);
                last_space = None;
            }
        }
    }
    if !current.is_empty() { lines.push(current); }
    lines
}

/// Compute the horizontal x for a line under the flag word's alignment.
pub fn align_x(line_width: i32, x0: i32, x1: i32, flags: u32) -> i32 {
    let box_w = x1 - x0 + 1;
    if flags & flag::LEFT_ALIGN != 0 { x0 }
    else if flags & flag::RIGHT_ALIGN != 0 { x1 - line_width + 1 }
    else { x0 + (box_w - line_width) / 2 }
}

/// Compute the vertical starting y for the first line under the top/
/// centered choice.
pub fn align_y(line_count: usize, line_height: i32, y0: i32, y1: i32,
               flags: u32) -> i32 {
    if flags & flag::TOP_ALIGN != 0 { y0 }
    else {
        let box_h = y1 - y0 + 1;
        y0 + (box_h - line_count as i32 * line_height) / 2
    }
}

/// Direct port of `FUN_005D0870` — but delegates the actual pixel work
/// to the `sample_bg` closure (only invoked in DROP_SHADOW mode) and
/// returns a [`DrawStringPlan`] the host renderer executes.
pub fn drawstring(
    x0: i32, y0: i32, x1: i32, y1: i32,
    flags: u32,
    font_id: i16,
    color: u16,
    text: &str,
    extra: i32,
    line_height: i32,
    mut measure: impl FnMut(&str) -> i32 + Copy,
    sample_bg: impl FnOnce(i32, i32) -> u16,
) -> DrawStringPlan {
    let mut plan = DrawStringPlan { blits: Vec::new() };
    let effective = if flags & flag::PASSWORD_MASK != 0 {
        "*".repeat(text.chars().count())
    } else { text.to_string() };

    let box_w = x1 - x0 + 1;
    let lines = wrap_lines(&effective, box_w, flags & flag::WORD_WRAP != 0, measure);
    let effective_lines = if lines.is_empty() && extra >= 0 {
        vec![String::new()]
    } else { lines };

    let start_y = align_y(effective_lines.len(), line_height, y0, y1, flags);

    // Drop-shadow color derivation: sample the background pixel once at
    // the box centre and derive bright + dark tones. Exe: `bg × 0xA6/100`
    // for the highlight, `bg × 0x42/100` for the shadow.
    let (shadow_color, highlight_color) = if flags & flag::DROP_SHADOW != 0 {
        let bg = sample_bg((x0 + x1) / 2, (y0 + y1) / 2);
        (scale_color_by_percent(bg, PCT_66),
         Some(scale_color_by_percent(bg, PCT_166)))
    } else {
        (color, None)
    };

    let mut y = start_y;
    for line in &effective_lines {
        let w = if flags & flag::LEFT_ALIGN == 0 && flags & flag::RIGHT_ALIGN == 0 {
            (measure)(line)
        } else { 0 };
        let x = align_x(w, x0, x1, flags);
        if let Some(hi) = highlight_color {
            // Highlight layer at (x+1, y+1) with brightened color.
            plan.blits.push(GlyphBlit {
                x: x + 1, y: y + 1, font_id, color: hi,
                text: line.clone(), extra,
            });
        }
        plan.blits.push(GlyphBlit {
            x, y, font_id, color: shadow_color,
            text: line.clone(), extra,
        });
        y += line_height;
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_lines_no_wrap_only_splits_on_newlines() {
        let lines = wrap_lines("hello\nworld", 1000, false, |s| s.len() as i32);
        assert_eq!(lines, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn wrap_lines_word_wraps_at_last_space() {
        // Each char = 10 units; max_width = 55 → allows 5 chars.
        let lines = wrap_lines("hello world", 55, true, |s| s.len() as i32 * 10);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("hello"));
    }

    #[test]
    fn align_left_returns_x0() {
        assert_eq!(align_x(50, 10, 100, flag::LEFT_ALIGN), 10);
    }

    #[test]
    fn align_right_returns_x1_minus_width_plus_1() {
        assert_eq!(align_x(50, 10, 100, flag::RIGHT_ALIGN), 100 - 50 + 1);
    }

    #[test]
    fn align_center_puts_text_in_middle() {
        // Box = 91 wide (10..100 inclusive). Text = 50. Slack = 41 → left pad 20.
        assert_eq!(align_x(50, 10, 100, 0), 10 + (91 - 50) / 2);
    }

    #[test]
    fn top_align_returns_y0() {
        assert_eq!(align_y(1, 15, 100, 200, flag::TOP_ALIGN), 100);
    }

    #[test]
    fn center_align_y_centers_multiline() {
        // Box = 101 tall (100..200). 3 lines × 15 = 45. Slack = 56 → top pad 28.
        assert_eq!(align_y(3, 15, 100, 200, 0), 100 + (101 - 45) / 2);
    }

    #[test]
    fn password_mask_replaces_chars_with_star() {
        let p = drawstring(0, 0, 100, 100, flag::PASSWORD_MASK,
                            0, 0, "hello", 0, 15, |s| s.len() as i32, |_,_| 0);
        assert_eq!(p.blits[0].text, "*****");
    }

    #[test]
    fn drop_shadow_emits_two_blits_per_line() {
        let p = drawstring(0, 0, 100, 100, flag::DROP_SHADOW,
                            0, 0xFFFF, "hi", 0, 15, |s| s.len() as i32, |_,_| 0x7BEF);
        assert_eq!(p.blits.len(), 2, "highlight + shadow");
        // Highlight is at (x+1, y+1) of the shadow.
        assert_eq!(p.blits[0].x, p.blits[1].x + 1);
        assert_eq!(p.blits[0].y, p.blits[1].y + 1);
    }

    #[test]
    fn no_shadow_emits_one_blit_per_line() {
        let p = drawstring(0, 0, 100, 100, 0,
                            0, 0xFFFF, "hi\nworld", 0, 15,
                            |s| s.len() as i32, |_,_| 0);
        assert_eq!(p.blits.len(), 2);
    }
}
