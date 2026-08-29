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
    #[serde(default)]
    images: Vec<ImgBlit>,
}

/// A bindable template (tools/extract_template.py): static chrome + the
/// dynamic data slots whose text is supplied at render time (from the sim).
#[derive(Deserialize)]
struct Template {
    static_calls: Vec<Call>,
    dynamic_slots: Vec<Slot>,
    #[serde(default)]
    region_slots: Vec<RegionSlot>,
}
/// A dynamic slot for centered/aligned variable-length text (e.g. a header
/// name). The renderer places the bound value itself using `align`/`anchor`.
#[derive(Deserialize)]
struct RegionSlot {
    y: i32,
    font: i64,
    color: i64,
    align: String,     // "center" | "right" | "left"
    anchor: i32,
    samples: Vec<String>,
    #[serde(default)]
    value: Option<String>,
}
#[derive(Deserialize)]
struct Slot {
    x: i32,
    y: i32,
    font: i64,
    color: i64,
    samples: Vec<String>,
    #[serde(default)]
    value: Option<String>, // live-bound override; falls back to samples[idx]
}
#[derive(Deserialize)]
struct ImgBlit {
    file: String,
    dst: [i32; 4],
    w: i32,
    h: i32,
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

/// Draw a filled nav-arrow triangle at the glyph origin (x,y is the top-left
/// where the '(' / ')' glyph would sit). Left-pointing for '(', right for ')'.
/// Sized to the ~10px arrow-font cell (matches menu_sidebar's arrows).
fn fill_arrow(s: &mut Surface, x: i32, y: i32, left: bool, rgb: (u8, u8, u8)) {
    // Clean isosceles triangle: at the vertical centre it spans the full
    // width `w`; it tapers linearly to a point at top and bottom.
    let (w, h): (i32, i32) = (9, 12);
    let packed = cm_render::pack565(rgb.0, rgb.1, rgb.2);
    for row in 0..h {
        let d = (2 * row - (h - 1)).abs(); // 0 at centre .. (h-1) at edges
        let span = (w * (h - 1 - d) / (h - 1)).max(0);
        for col in 0..span {
            let px = if left { x + col } else { x + (w - 1 - col) };
            s.set(px, y + row, packed);
        }
    }
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

fn text_to_hex(t: &str) -> String {
    let mut h = String::new();
    for b in t.bytes() {
        h.push_str(&format!("{:02x}", b));
    }
    h.push_str("00");
    h
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| {
        eprintln!("usage: replay_draws <draws_name> [out.bmp]");
        eprintln!("   or: replay_draws template:<slug>[:<sample_index>] [out.bmp]");
        std::process::exit(2);
    });

    // template:<slug>[:<idx>] — render a bindable template with dynamic slots
    // filled from each slot's live `value` (or samples[idx] to demonstrate the
    // same template driving different data). Static chrome replays verbatim.
    let mut region_slots: Vec<RegionSlot> = Vec::new();
    let mut draws: Draws = if let Some(rest) = name.strip_prefix("template:") {
        let mut parts = rest.splitn(2, ':');
        let slug = parts.next().unwrap();
        let idx: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let tp = format!(
            "{}/../../reports/screen_captures/templates/{}.json",
            env!("CARGO_MANIFEST_DIR"),
            slug
        );
        let traw = std::fs::read_to_string(&tp).unwrap_or_else(|e| {
            eprintln!("cannot read {tp}: {e}");
            std::process::exit(2);
        });
        let t: Template = serde_json::from_str(&traw).expect("parse template");
        let mut calls = t.static_calls;
        let pick = |samples: &Vec<String>, value: &Option<String>| -> String {
            value.clone().unwrap_or_else(|| {
                samples.get(idx).or_else(|| samples.last()).cloned().unwrap_or_default()
            })
        };
        for slot in &t.dynamic_slots {
            let val = pick(&slot.samples, &slot.value);
            calls.push(Call {
                fname: "glyph".into(),
                args: vec![slot.x as i64, slot.y as i64, slot.font, slot.color, 0],
                text: Some(text_to_hex(&val)),
            });
        }
        // Region slots need font metrics to place by alignment — render them
        // after the surface fonts exist (below). Carry the bound values here.
        for s in &t.region_slots {
            region_slots.push(RegionSlot {
                y: s.y, font: s.font, color: s.color, align: s.align.clone(),
                anchor: s.anchor, samples: s.samples.clone(),
                value: Some(pick(&s.samples, &s.value)),
            });
        }
        Draws { calls, images: Vec::new() }
    } else {
        let path = format!(
            "{}/../../reports/screen_captures/draws/{}.json",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("cannot read {path}: {e}");
            std::process::exit(2);
        });
        serde_json::from_str(&raw).expect("parse draws json")
    };

    // A burst often captures the screen drawn 2+ times back-to-back (one
    // redraw cycle each). Replaying every pass double-draws — and with darken
    // calls interleaved, earlier passes show through as ghosts. Keep only the
    // LAST full pass: find the last index where the very first call's
    // signature recurs, and replay from there.
    {
        let calls = &draws.calls;
        if calls.len() > 4 {
            let first = (calls[0].fname.clone(), calls[0].args.clone());
            let mut starts: Vec<usize> = (0..calls.len())
                .filter(|&i| calls[i].fname == first.0 && calls[i].args == first.1)
                .collect();
            if starts.len() >= 2 {
                let last_start = *starts.last().unwrap();
                draws.calls = draws.calls.split_off(last_start);
            }
        }
    }

    let mut s = Surface::new();
    s.fill(0, 0, 0);

    // Image blits FIRST (backgrounds/photos), in the order captured, before
    // the draw primitives paint over them — matching the exe's own order
    // (the screen blits its backdrop, then draws widgets on top). Pixels are
    // raw RGB565 saved by the capture's Blt hook.
    let draws_dir = format!("{}/../../reports/screen_captures/draws", env!("CARGO_MANIFEST_DIR"));
    // Image blit dst rects are in SCREEN coords (the game window is offset on
    // screen); the draw primitives use surface coords (0-origin). Derive the
    // window origin from the full-screen (800x600) backdrop blit and subtract
    // it from every image's dst so they land in surface space.
    let origin = draws.images.iter()
        .find(|i| i.w == 800 && i.h == 600)
        .map(|i| (i.dst[0], i.dst[1]))
        .unwrap_or((0, 0));
    let skip_bg = std::env::var("CM_NO_BG").is_ok();
    for img in &draws.images {
        if skip_bg { break; }
        let p = format!("{draws_dir}/{}", img.file);
        let Ok(raw) = std::fs::read(&p) else { continue };
        if raw.len() < (img.w * img.h * 2) as usize {
            continue;
        }
        let (dx, dy) = (img.dst[0] - origin.0, img.dst[1] - origin.1);
        for y in 0..img.h {
            for x in 0..img.w {
                let o = ((y * img.w + x) * 2) as usize;
                let v = raw[o] as u16 | ((raw[o + 1] as u16) << 8);
                s.set(dx + x, dy + y, v);
            }
        }
    }

    let fonts_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/cm0102/original_files/Data"
    );
    let mut fonts = Fonts::new(fonts_dir);

    let (mut n_rect, mut n_line, mut n_glyph, mut n_dark) = (0u32, 0u32, 0u32, 0u32);
    for c in &draws.calls {
        let a = &c.args;
        match c.fname.as_str() {
            "rect" if a.len() >= 6 => {
                // FUN_005cd840: flags&1 or &2 = 4-side outline; else solid
                // DirectDraw colorfill. Reproduce with fill_rect / hollow_rect.
                let flags = a[4] as u32;
                let rgb = unpack565((a[5] & 0xffff) as u16);
                if flags & 0x3 != 0 {
                    s.draw_hollow_rect(a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32, rgb);
                } else {
                    s.fill_rect(a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32, rgb);
                }
                n_rect += 1;
            }
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
            "glyph" if a.len() >= 5 => {
                // FUN_005ced50: param_1=x, param_2=y, param_3=font (sVar16 =
                // (short)param_3, its low 16 bits are the slot), param_4=color
                // (param_4 & 0xffff RGB565), param_5=text. -1 slot = skip.
                let text = c.text.as_ref().map(|h| hex_to_text(h)).unwrap_or_default();
                let slot = (a[2] & 0xffff) as i16;
                if !text.is_empty() && slot >= 0 && slot <= 7 {
                    let rgb = unpack565((a[3] & 0xffff) as u16);
                    // Font 0 is the game's symbol font (not one of our 7 .fnt
                    // atlases): its '(' / ')' glyphs are the nav arrows, drawn
                    // as filled triangles (as the validated menu_sidebar port
                    // does). Reproduce the glyph's actual shape.
                    if slot == 0 && (text == "(" || text == ")") {
                        fill_arrow(&mut s, a[0] as i32, a[1] as i32, text == "(", rgb);
                    } else {
                        let font = fonts.slot(slot as u8);
                        s.blit_string(a[0] as i32, a[1] as i32, font, rgb, &text);
                    }
                    n_glyph += 1;
                }
            }
            _ => {}
        }
    }

    // Region slots: place the bound value by its own measured width + the
    // captured alignment (center/right/left about `anchor`). This is what
    // fixes centered/variable-length dynamic text (header names).
    for slot in &region_slots {
        let val = slot.value.clone().unwrap_or_default();
        if val.is_empty() {
            continue;
        }
        let fid = (slot.font & 0xffff) as u8;
        if fid > 7 {
            continue;
        }
        let font = fonts.slot(fid);
        let w = font.text_width(&val);
        let x = match slot.align.as_str() {
            "center" => slot.anchor - w / 2,
            "right" => slot.anchor - w,
            _ => slot.anchor,
        };
        let rgb = unpack565((slot.color & 0xffff) as u16);
        s.blit_string(x, slot.y, font, rgb, &val);
    }

    let mut argb = vec![0u32; Surface::W * Surface::H];
    s.to_argb(&mut argb);
    let out = args.next().unwrap_or_else(|| format!("{name}_replay.bmp"));
    write_bmp(&out, Surface::W, Surface::H, &argb).expect("write bmp");
    eprintln!("wrote {out}  ({n_rect} rects, {n_line} lines, {n_glyph} glyphs, {n_dark} darken)");
}

