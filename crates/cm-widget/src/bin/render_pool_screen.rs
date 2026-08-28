//! Reconstruct a screen from its POST-LAYOUT pool join
//! (reports/screen_captures/<screen>.pool.json, built by
//! tools/build_pool_screen.py) using the ported primitives + real fonts.
//!
//! This is items 2+3 of the GUI plan proven end to end: every widget is
//! drawn at its FINAL post-layout rect (from the exe's widget pool), filled
//! with the color SAMPLED from the real back-buffer frame, and its text
//! blitted in the real game font. No geometry or color is invented —
//! rects come from the pool, colors from the screenshot, fonts from the
//! captured font id. The output is a reconstruction to diff against the
//! frame grab (reports/screen_captures/live/<burst>.frame.png).
//!
//! Usage: render_pool_screen <screen> [out.bmp]

use cm_render::font::{slot_line_height, Fonts};
use cm_render::Surface;
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize)]
struct PoolScreen {
    widgets: Vec<PoolWidget>,
}

#[derive(Deserialize)]
struct PoolWidget {
    #[serde(rename = "L")] l: i32,
    #[serde(rename = "T")] t: i32,
    #[serde(rename = "R")] r: i32,
    #[serde(rename = "B")] b: i32,
    font: i64,
    rflags: i64,
    text: String,
    fg: Option<[u8; 3]>,
    bg: Option<[u8; 3]>,
}

fn write_bmp(path: &str, w: usize, h: usize, argb: &[u32]) -> std::io::Result<()> {
    let row_bytes = w * 3;
    let padding = (4 - (row_bytes % 4)) % 4;
    let data_size = (row_bytes + padding) * h;
    let mut f = std::fs::File::create(path)?;
    f.write_all(b"BM")?;
    f.write_all(&((14 + 40 + data_size) as u32).to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&54u32.to_le_bytes())?;
    f.write_all(&40u32.to_le_bytes())?;
    f.write_all(&(w as i32).to_le_bytes())?;
    f.write_all(&(h as i32).to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&24u16.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&(data_size as u32).to_le_bytes())?;
    f.write_all(&0i32.to_le_bytes())?;
    f.write_all(&0i32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    let pad = vec![0u8; padding];
    for y in (0..h).rev() {
        for x in 0..w {
            let p = argb[y * w + x];
            f.write_all(&[(p & 0xff) as u8, ((p >> 8) & 0xff) as u8, ((p >> 16) & 0xff) as u8])?;
        }
        f.write_all(&pad)?;
    }
    Ok(())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let screen = args.next().unwrap_or_else(|| {
        eprintln!("usage: render_pool_screen <screen> [out.bmp]");
        std::process::exit(2);
    });
    let path = format!(
        "{}/../../reports/screen_captures/{}.pool.json",
        env!("CARGO_MANIFEST_DIR"),
        screen
    );
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(2);
    });
    let scr: PoolScreen = serde_json::from_str(&raw).expect("parse pool json");

    let mut s = Surface::new();
    // Neutral ground so any unpainted widget is obvious (the real bg image
    // is a separate plan item — item 4).
    s.fill(18, 20, 24);
    let fonts_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/cm0102/original_files/Data"
    );
    let mut fonts = Fonts::new(fonts_dir);

    for w in &scr.widgets {
        if w.r <= w.l || w.b <= w.t {
            continue;
        }
        // Fill with the color sampled from the real frame at this rect.
        if let Some(bg) = w.bg {
            let rf = w.rflags as u32;
            const BG: u32 = cm_render::panel::F_SOLID_FILL
                | cm_render::panel::F_BEVEL
                | cm_render::panel::F_TRANSPARENT
                | cm_render::panel::F_BORDER;
            if rf & BG != 0 {
                s.draw_panel(w.l, w.t, w.r, w.b, rf, (bg[0], bg[1], bg[2]));
            } else {
                s.fill_rect(w.l, w.t, w.r, w.b, (bg[0], bg[1], bg[2]));
            }
        }
        let txt = w.text.trim();
        if txt.is_empty() {
            continue;
        }
        let fid = w.font as u8;
        let font = fonts.slot(fid);
        let ink = w.fg.map(|c| (c[0], c[1], c[2])).unwrap_or((255, 255, 255));
        // Left-aligned with a small inset, vertically centred — the exe's
        // default text placement for list cells (tmode-driven alignment is
        // a later refinement).
        let cy = w.t + ((w.b - w.t) - slot_line_height(fid)) / 2;
        for (li, line) in txt.split('\n').enumerate() {
            s.blit_string(w.l + 3, cy + li as i32 * slot_line_height(fid),
                          font, ink, line);
        }
    }

    let mut argb = vec![0u32; Surface::W * Surface::H];
    s.to_argb(&mut argb);
    let out = args.next().unwrap_or_else(|| format!("{screen}_pool.bmp"));
    write_bmp(&out, Surface::W, Surface::H, &argb).expect("write bmp");
    eprintln!("wrote {out} ({} widgets)", scr.widgets.len());
}
