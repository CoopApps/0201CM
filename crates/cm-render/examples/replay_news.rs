//! THROWAWAY (2026-09-04): replay the exe's OWN captured News draw calls —
//! chrome (line/rect/panel/darken) + text (wrapped) — through our ported
//! packed primitives + real .fnt fonts. No builder, no guessing.
//! Run: cargo run -p cm-render --example replay_news -- chrome.csv text.tsv out.raw

use cm_render::font::Fonts;
use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::PixelFont;
use cm_render::packed_panel::{draw_panel, PanelPalette};
use cm_render::packed_text::draw_wrapped_text;
use std::collections::HashMap;
use std::io::{BufRead, Write};

fn main() {
    let chrome = std::env::args().nth(1).unwrap();
    let text = std::env::args().nth(2).unwrap();
    let out = std::env::args().nth(3).unwrap_or_else(|| "news_replay.raw".into());
    let mut s = PackedSurface::rgb555(800, 600);
    let pal = PanelPalette { outer_highlight: 0x7fe0, default_bevel: 0x0010 };

    // --- chrome ---
    for line in std::io::BufReader::new(std::fs::File::open(&chrome).unwrap()).lines() {
        let line = line.unwrap();
        let p: Vec<&str> = line.split(',').collect();
        if p.len() < 7 { continue; }
        let g = |i: usize| p[i].parse::<i32>().unwrap_or(0);
        let (x0, y0, x1, y1, st, col) = (g(1), g(2), g(3), g(4), g(5) as u32, g(6) as u16);
        match p[0] {
            "line"   => s.draw_line(x0, y0, x1, y1, st, col),
            "rect"   => s.draw_rectangle(x0, y0, x1, y1, st, col),
            "panel"  => draw_panel(&mut s, x0, y0, x1, y1, st, col, 0, pal),
            "darken" => s.darken_rect(x0, y0, x1, y1),
            _ => {}
        }
    }

    // --- fonts: preload the slots the capture references ---
    let mut fonts = Fonts::new("D:/cm0102/Data");
    let mut slots: HashMap<u8, PixelFont> = HashMap::new();
    for id in [1u8, 3, 7] { slots.insert(id, fonts.pixel_slot(id).clone()); }

    // --- text (wrapped ops: x0,y0,x1,y1,style,colour,font,text) ---
    let mut n = 0;
    for line in std::io::BufReader::new(std::fs::File::open(&text).unwrap()).lines() {
        let line = line.unwrap();
        let p: Vec<&str> = line.split('\t').collect();
        if p.len() < 9 || p[0] != "wrapped" { continue; }
        let g = |i: usize| p[i].parse::<i32>().unwrap_or(0);
        let (x0, y0, x1, y1) = (g(1), g(2), g(3), g(4));
        let style = i64::from_str_radix(p[5].trim_start_matches("0x"), 16)
            .or_else(|_| p[5].parse::<i64>()).unwrap_or(0) as u32;
        let colour = g(6) as u16;
        let font_id = g(7) as u8;
        let mut txt = p[8].as_bytes().to_vec(); txt.push(0);
        if let Some(font) = slots.get(&font_id).or_else(|| slots.get(&1)) {
            draw_wrapped_text(&mut s, x0, y0, x1, y1, font, &txt, colour, style, -1);
            n += 1;
        }
    }
    eprintln!("rendered {n} text ops");

    let mut fo = std::fs::File::create(&out).unwrap();
    for &px in &s.buf { fo.write_all(&px.to_le_bytes()).unwrap(); }
    eprintln!("wrote {out}");
}
