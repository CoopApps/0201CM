//! Replay the game's captured draw-primitive stream
//! (reports/screen_captures/draws/<name>.json, from tools/frida_draw_capture.py)
//! through the ported cm-render primitives — reproducing the screen
//! PIXEL-EXACT, because we re-issue the game's own draw commands with the
//! game's own resolved arguments.
//!
//! Mapping (each capture record -> its ported primitive):
//!   line   (x0,y0,x1,y1, mode, u16 color)  -> cm_render::line::draw_line   (FUN_005cd420, 1:1)
//!   darken (x0,y0,x1,y1)                    -> Surface::draw_panel F_TRANSPARENT (FUN_005cdfd0)
//!   glyph  (x, y, _, u16 color, text, font) -> Fonts::blit_string          (FUN_005ced50)
//!
//! The glyph font arrives as -1 (the exe reads a global current-font, not an
//! arg); until that's captured separately, font is chosen by a line-height
//! heuristic. Lines and darken are fully exact regardless.
//!
//! Usage: replay_draws <draws_name> [out.bmp]

use cm_render::font::Fonts;
use cm_render::line::draw_line;
use cm_render::panel::F_TRANSPARENT;
use cm_render::{unpack565, Surface};
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize)]
struct Draws {
    calls: Vec<Call>,
}
#[derive(Deserialize)]
struct Call {
    #[serde(rename = "fn")]
    fname: String,
    args: Vec<i64>,
    text: Option<String>,
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

fn hex_to_text(h: &str) -> String {
    let mut bytes = Vec::with_capacity(h.len() / 2);
    let hb = h.as_bytes();
    let mut i = 0;
    while i + 1 < hb.len() {
        let hi = (hb[i] as char).to_digit(16).unwrap_or(0);
        let lo = (hb[i + 1] as char).to_digit(16).unwrap_or(0);
        let b = (hi * 16 + lo) as u8;
        if b == 0 {
            break;
        }
        bytes.push(b);
        i += 2;
    }
    bytes.iter().map(|&b| b as char).collect()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| {
        eprintln!("usage: replay_draws <draws_name> [out.bmp]");
        std::process::exit(2);
    });
    let path = format!(
        "{}/../../reports/screen_captures/draws/{}.json",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(2);
    });
    let draws: Draws = serde_json::from_str(&raw).expect("parse draws json");

    let mut s = Surface::new();
    s.fill(0, 0, 0);
    let fonts_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/cm0102/original_files/Data"
    );
    let mut fonts = Fonts::new(fonts_dir);

    let (mut n_line, mut n_glyph, mut n_dark) = (0u32, 0u32, 0u32);
    for c in &draws.calls {
        let a = &c.args;
        match c.fname.as_str() {
            "line" if a.len() >= 6 => {
                draw_line(&mut s, a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32,
                          a[4] as u32, (a[5] & 0xffff) as u16);
                n_line += 1;
            }
            "darken" if a.len() >= 4 => {
                s.draw_panel(a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32,
                             F_TRANSPARENT, (0, 0, 0));
                n_dark += 1;
            }
            "glyph" if a.len() >= 4 => {
                let text = c.text.as_ref().map(|h| hex_to_text(h)).unwrap_or_default();
                if !text.is_empty() {
                    let rgb = unpack565((a[3] & 0xffff) as u16);
                    // Font heuristic until the current-font global is captured:
                    // the sidebar/most UI text is slot 1 (arial_narrow_10).
                    let font = fonts.slot(1);
                    s.blit_string(a[0] as i32, a[1] as i32, font, rgb, &text);
                    n_glyph += 1;
                }
            }
            _ => {}
        }
    }

    let mut argb = vec![0u32; Surface::W * Surface::H];
    s.to_argb(&mut argb);
    let out = args.next().unwrap_or_else(|| format!("{name}_replay.bmp"));
    write_bmp(&out, Surface::W, Surface::H, &argb).expect("write bmp");
    eprintln!("wrote {out}  ({n_line} lines, {n_glyph} glyphs, {n_dark} darken)");
}

