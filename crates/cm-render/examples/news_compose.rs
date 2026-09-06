//! THROWAWAY composition harness (2026-09-04): assemble our News screen
//! from the pieces that already exist — real .fnt font, real .RGN photo
//! background, real widget pool via `build_news_screen` + `render_widget` —
//! and dump the raw RGB555 surface so it can be diffed/eyeballed against
//! the live exe capture (`fixtures/news_after.pixels.bin`).
//!
//! Run: cargo run -p cm-render --example news_compose -- out.raw

use cm_render::font::Fonts;
use cm_render::image::Image;
use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::PixelFont;
use cm_render::packed_panel::PanelPalette;
use cm_render::packed_widget::{render_widget, Widget, WidgetGlobals};
use cm_render::screen_news::{build_news_screen, NewsItem, NewsScreenState};
use cm_render::widget_pool::GuiRecordPool;
use std::io::Write;

const W: i32 = 800;
const H: i32 = 600;

/// 565 (RGN file format) -> 555 (live surface format), matching the exe's
/// blit-time conversion.
fn c565_to_555(v: u16) -> u16 {
    let r = (v >> 11) & 0x1f;
    let g = ((v >> 5) & 0x3f) >> 1; // 6-bit green -> 5-bit
    let b = v & 0x1f;
    (r << 10) | (g << 5) | b
}

/// Placeholder data ONLY. This is a graphics-fidelity test — the headlines,
/// manager name, dates and body are all game-state-generated in the real game
/// and must NOT be hardcoded to match the exe. What we compare is the CHROME
/// (sidebar, title bar, tab strips, list-row geometry, panels, photo, buttons,
/// font placement), not the words. Text regions differing is expected.
fn news_state() -> NewsScreenState {
    NewsScreenState {
        header_title: "Manager News".into(),
        active_tab: 0,
        filter_text: String::new(),
        next_unread_enabled: true,
        items: vec![
            NewsItem { headline: "News item one".into(),   unread: true,  date: "Date AM".into() },
            NewsItem { headline: "News item two".into(),   unread: true,  date: "Date PM".into() },
            NewsItem { headline: "News item three".into(), unread: true,  date: "Date PM".into() },
            NewsItem { headline: "News item four".into(),  unread: false, date: "Date AM".into() },
            NewsItem { headline: "News item five".into(),  unread: false, date: "Date EVE".into() },
        ],
        selected_body: "Placeholder story body text.".into(),
        back_disabled: false,
        next_enabled: true,
    }
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "news_ours.raw".into());
    let mut surface = PackedSurface::rgb555(W, H);

    // 1. Real photo background (the exe blits the story RGN as the base layer).
    match Image::load_rgn("D:/cm0102/pictures/eMP_MADRID_FANS_3617_361716.RGN") {
        Ok(img) => {
            eprintln!("RGN {}x{} loaded", img.w, img.h);
            for y in 0..H.min(img.h as i32) {
                for x in 0..W.min(img.w as i32) {
                    let src = img.px[(y as usize) * img.w + (x as usize)];
                    surface.buf[(y * surface.pitch_pixels + x) as usize] = c565_to_555(src);
                }
            }
        }
        Err(e) => eprintln!("RGN load failed: {e} (background will be black)"),
    }

    // 2. Real font (verified 224/224 glyphs earlier this session).
    let mut fonts = Fonts::new("D:/cm0102/Data");
    let font: PixelFont = fonts.pixel_slot(3).clone();

    // 3. Real widget pool.
    let mut pool = GuiRecordPool::new();
    build_news_screen(&mut pool, &news_state()).expect("build_news_screen");
    eprintln!("pool: {} widgets, {} areas", pool.widgets.len(), pool.areas.len());

    // 4. Render every widget on top of the photo.
    let palette = PanelPalette { outer_highlight: 0x7fff, default_bevel: 0x39e7 };
    let n = pool.widgets.len();
    for i in 0..n {
        let d = pool.widgets[i].descriptor.clone();
        let mut w = Widget {
            frame_base: 0, flags: d.kind,
            x0: d.grid_x0, y0: d.grid_y0, x1: d.grid_x1, y1: d.grid_y1,
            style_byte: 0x30, text_style: 0x0c, text_kern: -1,
            saved_bg: None, cached_text: None,
            colour_a: 0x0200, colour_b: 0,
            label_ink: d.label_ink, pattern: 0, frame_idx: -1,
            label: { let mut b = d.text.as_bytes().to_vec(); b.push(0); b },
            detached_glyph_cache: 0, alt_hover: 0,
        };
        render_widget(&mut surface, &mut w, Some(&pool), &font,
                      WidgetGlobals { panel_palette: palette }, true);
    }

    // 5. Dump raw u16 LE.
    let mut f = std::fs::File::create(&out).unwrap();
    for &p in &surface.buf {
        f.write_all(&p.to_le_bytes()).unwrap();
    }
    eprintln!("wrote {out} ({} u16)", surface.buf.len());
}
