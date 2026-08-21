//! `.fnt` bitmap-font typography — faithful port of the CM0102 text pipeline.
//!
//! - format:   the loader `FUN_005ce890` — int32 height, then per glyph from codepoint 0x20:
//!             `[advance, bmw_bytes, field3, field4]` + `bmw*height` bitmap bytes (4-bit
//!             coverage, TWO pixels per byte: high nibble = left pixel).
//! - tracking: `FUN_005cf840` — `max(0, base*3/4 - cur.field3 - prev.field4)`, base = space advance.
//! - blit:     `FUN_005ced50` — composite coverage in the text colour; cursor += tracking + advance.
//! - text box: `FUN_005d0870` — alignment / vertical-centering / shadow.
//!
//! These are the game's own glyphs; the tracking was verified pixel-exact against the grab.

use crate::{pack565, unpack565, Surface};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// text-box flags (FUN_005d0870 param_5)
pub const F_LEFT: u32 = 0x1; // left-align (else centred)
pub const F_NOVCENTER: u32 = 0x2; // top-align (else vertical-centre)
pub const F_SHADOW: u32 = 0x20; // draw a +1,+1 shadow first (futuristic style)
pub const F_RIGHT: u32 = 0x40; // right-align

#[derive(Clone)]
pub struct Glyph {
    pub advance: i32,    // field1: cell width (= 2*bmw px)
    pub bmw: usize,      // field2: bitmap byte-width (2 px per byte)
    pub lb: i32,         // field3: left bearing / kern term
    pub rb: i32,         // field4: right bearing / kern term
    pub bitmap: Vec<u8>, // bmw*height bytes, 4-bit coverage packed 2 px/byte
}

pub struct Font {
    pub height: usize,    // glyph bitmap rows (.fnt @0)
    pub line_height: i32, // vertical advance per line (per-slot metric)
    pub base: i32,        // tracking base = space glyph's advance
    glyphs: HashMap<u32, Glyph>,
}

impl Font {
    /// Parse a `.fnt` file exactly as `FUN_005ce890` reads it.
    pub fn parse(data: &[u8], line_height: i32) -> Font {
        let rd = |o: usize| i32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
        let height = rd(0).max(0) as usize;
        let mut glyphs = HashMap::new();
        let mut off = 4usize;
        let mut cp = 0x20u32;
        while cp < 0x100 && off + 16 <= data.len() {
            let advance = rd(off);
            let bmw = rd(off + 4).max(0) as usize;
            let lb = rd(off + 8);
            let rb = rd(off + 12);
            off += 16;
            let n = bmw * height;
            let bitmap = if n > 0 && off + n <= data.len() {
                data[off..off + n].to_vec()
            } else {
                Vec::new()
            };
            off += n;
            glyphs.insert(cp, Glyph { advance, bmw, lb, rb, bitmap });
            cp += 1;
        }
        let base = glyphs.get(&0x20).map(|g| g.advance).unwrap_or(0);
        Font { height, line_height, base, glyphs }
    }

    pub fn load(path: impl AsRef<Path>, line_height: i32) -> std::io::Result<Font> {
        Ok(Font::parse(&std::fs::read(path)?, line_height))
    }

    #[inline]
    fn glyph(&self, cp: u32) -> &Glyph {
        self.glyphs.get(&cp).or_else(|| self.glyphs.get(&0x20)).expect("font has no space glyph")
    }

    /// `FUN_005cf840` inter-glyph tracking.
    #[inline]
    fn track(&self, prev: Option<&Glyph>, cur: &Glyph) -> i32 {
        match prev {
            None => 0,
            Some(p) => ((self.base * 3) / 4 - cur.lb - p.rb).max(0),
        }
    }

    /// Rendered width: sum of advances plus inter-glyph tracking.
    pub fn text_width(&self, s: &str) -> i32 {
        let mut w = 0;
        let mut prev: Option<&Glyph> = None;
        for ch in s.chars() {
            let g = self.glyph(ch as u32);
            w += self.track(prev, g) + g.advance;
            prev = Some(g);
        }
        w
    }
}

/// slot -> `.fnt` base filename (from graphics_load_all_fonts 0x005ce750).
pub fn slot_file(slot: u8) -> &'static str {
    match slot {
        0 | 1 => "arial_narrow_10",
        2 => "arial_narrow_11",
        3 => "arial_14",
        4 => "arial_16",
        5 => "arial_18",
        6 => "trade_cond_24_bold",
        _ => "trade_cond_28_bold",
    }
}

/// slot -> line height (graphics_font_row_height 0x005cf7b0 traditional metrics).
pub fn slot_line_height(slot: u8) -> i32 {
    match slot {
        0 | 1 => 15,
        2 => 18,
        3 => 21,
        4 => 24,
        5 => 27,
        6 => 39,
        _ => 45,
    }
}

/// Lazily-loaded cache of fonts by slot, from a directory of `.fnt` files.
pub struct Fonts {
    dir: PathBuf,
    cache: HashMap<u8, Font>,
}

impl Fonts {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), cache: HashMap::new() }
    }

    pub fn slot(&mut self, slot: u8) -> &Font {
        if !self.cache.contains_key(&slot) {
            let p = self.dir.join(format!("{}.fnt", slot_file(slot)));
            let f = Font::load(&p, slot_line_height(slot))
                .unwrap_or_else(|e| panic!("load font {:?}: {}", p, e));
            self.cache.insert(slot, f);
        }
        &self.cache[&slot]
    }
}

impl Surface {
    /// `FUN_005ced50`: composite each glyph's 4-bit coverage (2 px/byte) in the text colour.
    pub fn blit_string(&mut self, x: i32, y: i32, font: &Font, rgb: (u8, u8, u8), s: &str) {
        let mut cx = x;
        let mut prev: Option<&Glyph> = None;
        for ch in s.chars() {
            let g = font.glyph(ch as u32);
            cx += font.track(prev, g);
            if g.bmw > 0 && !g.bitmap.is_empty() {
                for row in 0..font.height {
                    let rowoff = row * g.bmw;
                    for bcol in 0..g.bmw {
                        let byte = g.bitmap[rowoff + bcol];
                        if byte == 0 {
                            continue;
                        }
                        for half in 0..2 {
                            let nib = if half == 0 { byte >> 4 } else { byte & 0xf };
                            if nib == 0 {
                                continue;
                            }
                            let cov = nib as i32 * 17;
                            let dx = cx + g.lb + (bcol as i32) * 2 + half;
                            let dy = y + row as i32;
                            if dx >= 0 && dy >= 0 && (dx as usize) < self.w && (dy as usize) < self.h {
                                let idx = dy as usize * self.w + dx as usize;
                                let (br, bg, bb) = unpack565(self.buf[idx]);
                                let bl = |b: u8, f: u8| {
                                    (b as i32 + (f as i32 - b as i32) * cov / 255) as u8
                                };
                                self.buf[idx] = pack565(bl(br, rgb.0), bl(bg, rgb.1), bl(bb, rgb.2));
                            }
                        }
                    }
                }
            }
            cx += g.advance;
            prev = Some(g);
        }
    }

    /// `FUN_005d0870`: place `text` in the box per alignment / vcenter / shadow flags.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text_box(
        &mut self,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        flags: u32,
        font: &Font,
        rgb: (u8, u8, u8),
        text: &str,
    ) {
        let box_w = right - left + 1;
        let box_h = bottom - top + 1;
        let lh = font.line_height;
        let lines: Vec<&str> = text.split('\n').collect();
        let mut y = if flags & F_NOVCENTER != 0 {
            top
        } else {
            (box_h - lines.len() as i32 * lh) / 2 + top
        };
        for line in lines {
            let tw = font.text_width(line);
            let x = if flags & F_LEFT != 0 {
                left
            } else if flags & F_RIGHT != 0 {
                right - tw + 1
            } else {
                (box_w - tw) / 2 + left
            };
            if flags & F_SHADOW != 0 {
                self.blit_string(x + 1, y + 1, font, (0, 0, 0), line);
            }
            self.blit_string(x, y, font, rgb, line);
            y += lh;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Font;

    fn fonts_dir() -> std::path::PathBuf {
        std::env::var("CM_FONT_DIR").unwrap_or_else(|_| "D:/cm0102/Data".to_string()).into()
    }

    /// Locks the traditional `.fnt` decode + metrics for the title font. Verified values:
    /// trade_cond_28_bold has bitmap height 45, space (tracking base) advance 7, and 'C'
    /// advance 14. If the game fonts aren't present (CI without assets) the test no-ops.
    #[test]
    fn trade_cond_28_metrics() {
        let p = fonts_dir().join("trade_cond_28_bold.fnt");
        if !p.exists() {
            eprintln!("skip trade_cond_28_metrics: fonts not present at {:?}", p);
            return;
        }
        let font = Font::load(&p, 45).expect("load trade_cond_28_bold.fnt");
        assert_eq!(font.height, 45, "glyph bitmap height");
        assert_eq!(font.base, 7, "tracking base = space advance");
        // single-char width == that glyph's advance (no tracking): 'C' == 14 (field1)
        assert_eq!(font.text_width("C"), 14, "C advance");
        // a full title has a stable, positive tracked width (regression guard for FUN_005cf840)
        assert!(font.text_width("Championship Manager 2001/02") > 400);
    }

    /// Locks the arial_14 (slot 3, table-body) decode: height 21, space advance 4.
    #[test]
    fn arial_14_metrics() {
        let p = fonts_dir().join("arial_14.fnt");
        if !p.exists() {
            eprintln!("skip arial_14_metrics: fonts not present at {:?}", p);
            return;
        }
        let font = Font::load(&p, 21).expect("load arial_14.fnt");
        assert_eq!(font.height, 21);
        assert_eq!(font.base, 4);
    }
}
