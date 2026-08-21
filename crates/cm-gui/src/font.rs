//! Text pipeline: port of FUN_005d0870 (text box: align/center/shadow) + the .fnt
//! glyph blit (FUN_005ced50). Fonts are CM0102's real decoded .fnt bitmaps.
//! Width = sum of native .fnt advances (validated vs tight in-game text).

use crate::surface::{pack565, unpack565, Surface};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct RawGlyph {
    codepoint: u32,
    advance: i32,
    bitmap_width: i32,
    left_bearing: i32,
    #[serde(default)]
    bitmap_hex: String,
}
#[derive(Deserialize)]
struct RawFont {
    height: i32,
    glyphs: Vec<RawGlyph>,
}

pub struct Glyph {
    pub adv: i32,
    pub w: i32,
    pub lb: i32,
    pub bm: Vec<u8>, // coverage bytes, w*height
}

pub struct Font {
    pub height: i32,
    pub glyphs: HashMap<u32, Glyph>,
}

fn hex_to_bytes(h: &str) -> Vec<u8> {
    (0..h.len() / 2).map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap_or(0)).collect()
}

impl Font {
    pub fn load(path: &str) -> Font {
        let raw: RawFont = serde_json::from_str(&std::fs::read_to_string(path).expect("font json")).expect("font parse");
        let mut glyphs = HashMap::new();
        for g in raw.glyphs {
            glyphs.insert(
                g.codepoint,
                Glyph { adv: g.advance, w: g.bitmap_width, lb: g.left_bearing, bm: hex_to_bytes(&g.bitmap_hex) },
            );
        }
        Font { height: raw.height, glyphs }
    }

    pub fn width(&self, s: &str) -> i32 {
        s.chars().map(|c| self.glyphs.get(&(c as u32)).map(|g| g.adv).unwrap_or(4)).sum()
    }
}

// FUN_005d0870 alignment flags
pub const F_LEFT: u32 = 0x1;
pub const F_RIGHT: u32 = 0x40;

/// Blit one string at (x,y) in a color — FUN_005ced50: per-glyph coverage composite.
fn blit(surf: &mut Surface, font: &Font, x: i32, y: i32, s: &str, rgb: (u8, u8, u8)) {
    let mut cx = x;
    for c in s.chars() {
        let g = match font.glyphs.get(&(c as u32)) {
            Some(g) => g,
            None => {
                cx += 4;
                continue;
            }
        };
        if g.w > 0 && !g.bm.is_empty() {
            for row in 0..font.height {
                for col in 0..g.w {
                    let cov = g.bm[(row * g.w + col) as usize] as u32;
                    if cov != 0 {
                        let dx = cx + g.lb + col;
                        let dy = y + row;
                        let (br, bg, bb) = unpack565(surf.get(dx, dy));
                        let a = cov;
                        let mix = |bc: u8, fc: u8| ((bc as u32 * (255 - a) + fc as u32 * a) / 255) as u8;
                        surf.set(dx, dy, pack565(mix(br, rgb.0), mix(bg, rgb.1), mix(bb, rgb.2)));
                    }
                }
            }
        }
        cx += g.adv;
    }
}

/// FUN_005d0870: place text in a box per alignment (left/right/center). Single line.
pub fn draw_text_box(surf: &mut Surface, font: &Font, l: i32, t: i32, r: i32, s: &str, flags: u32, rgb: (u8, u8, u8)) {
    let tw = font.width(s);
    let x = if flags & F_LEFT != 0 {
        l
    } else if flags & F_RIGHT != 0 {
        r - tw
    } else {
        l + ((r - l) - tw) / 2
    };
    blit(surf, font, x, t, s, rgb);
}
