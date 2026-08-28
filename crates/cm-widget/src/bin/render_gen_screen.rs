//! Rasterize a GENERATED screen skeleton (cm-render/src/gen_screens/, from
//! tools/gen_screen_rs.py) — live-capture aware.
//!
//! Supersedes the earlier cm-render bin of the same name. What's real here:
//!   * every non-zero rect is exe-computed geometry, byte-exact;
//!   * text is the live-captured string the game itself constructed;
//!   * the tab strip is drawn by the SAME ported `FUN_005d7070` code the
//!     app uses (`cm_widget::tab_strip`), fed the captured labels and the
//!     captured selection bit (rflags & 0x800);
//!   * the nav row (Cancel/Ok, Back/Next) uses the ported FUN_005d75b0
//!     geometry (cols [3,1] split at x=618, News ground truth).
//! What's still placeholder: per-screen colors (only News is measured) and
//! the table body (needs the layout engine over the captured area grids).
//! Sidebar widgets are excluded by caller address (shared chrome, drawn by
//! FUN_00745540 — a separate render path).
//!
//! Usage: `cargo run -p cm-widget --bin render_gen_screen -- <slug> [out.bmp]`

use cm_render::font::{slot_line_height, Fonts};
use cm_render::gen_screen_types::{GenObj, GenScreen};
use cm_render::layout::rebuild_layout;
use cm_render::panel::{F_BEVEL, F_SOLID_FILL};
use cm_render::Surface;
use cm_widget::tab_strip::{draw_tab_strip, TabRecord};
use cm_widget::Palette;
use std::io::Write;

fn write_bmp(path: &str, w: usize, h: usize, argb: &[u32]) -> std::io::Result<()> {
    let row_bytes = w * 3;
    let padding = (4 - (row_bytes % 4)) % 4;
    let data_size = (row_bytes + padding) * h;
    let file_size = 14 + 40 + data_size;
    let mut f = std::fs::File::create(path)?;
    f.write_all(b"BM")?;
    f.write_all(&(file_size as u32).to_le_bytes())?;
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

// Caller-address attribution (evidence: player_search_live capture).
// Sidebar tree: FUN_00745540 + its helpers, and the comp-menu item helper
// observed at 0x415c86. Tab labels: inside FUN_005d7070 (site 0x5d74e2).
// Nav labels: site 0x415bca (shared Cancel/Ok emitter).
fn is_sidebar(caller: i64) -> bool {
    (0x745540..0x74d500).contains(&caller) || (0x415c60..0x415cb0).contains(&caller)
}
fn is_tab_label(caller: i64) -> bool {
    (0x5d7000..0x5d7600).contains(&caller)
}
fn is_nav_label(caller: i64) -> bool {
    (0x415ba0..0x415be0).contains(&caller)
}

// Placeholder palette (measured News values as stand-ins; see render_news.rs).
const COLOR_PANEL: (u8, u8, u8) = (0, 48, 165);
const COLOR_BUTTON: (u8, u8, u8) = (132, 130, 132);

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(slug) = args.next() else {
        eprintln!("usage: render_gen_screen <slug> [out.bmp]");
        for s in cm_render::gen_screens::all_slugs() {
            eprintln!("  {s}");
        }
        std::process::exit(2);
    };
    let Some(f) = cm_render::gen_screens::lookup(&slug) else {
        eprintln!("no generated screen {slug:?} -- run tools/gen_screen_rs.py");
        std::process::exit(2);
    };
    let scr: GenScreen = f();

    let mut s = Surface::new();
    s.fill(10, 15, 30);
    let fonts_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/cm0102/original_files/Data"
    );
    let mut fonts = Fonts::new(fonts_dir);
    let pal = Palette::default();

    // ---- classify objects by caller ------------------------------------
    let mut screen_objs: Vec<&GenObj> = Vec::new();
    let mut tab_labels: Vec<(u32, &str, bool)> = Vec::new(); // (id, label, selected)
    let mut nav_labels: Vec<(&str, u8)> = Vec::new(); // (label, captured font)
    let mut n_sidebar = 0usize;
    let mut n_state = 0usize;
    for o in &scr.objects {
        if is_sidebar(o.caller) {
            n_sidebar += 1;
        } else if is_tab_label(o.caller) {
            tab_labels.push((o.id, o.text, o.rflags & 0x800 != 0));
        } else if is_nav_label(o.caller) {
            nav_labels.push((o.text, o.font as u8));
        } else if o.r <= o.l || o.b <= o.t {
            n_state += 1;
        } else {
            screen_objs.push(o);
        }
    }

    // ---- areas (backgrounds), exe construction order -------------------
    // Captured flags straight into the ported draw_panel — no reinterpretation.
    // (Fill COLOR is still a placeholder until per-screen colors are sampled
    // live; the flag-driven STYLE — solid/bevel/dim/border/none — is real.)
    for a in &scr.areas {
        if a.r <= a.l || a.b <= a.t || is_sidebar(a.caller) {
            continue;
        }
        s.draw_panel(a.l, a.t, a.r, a.b, a.flags as u32, COLOR_PANEL);
    }

    // ---- screen objects with real rects + real text --------------------
    for o in &screen_objs {
        // Background exactly as the widget's own captured rflags request:
        // draw_panel implements the exe's flag semantics (F_TRANSPARENT
        // dims to 60% via ported FUN_005cdfd0, F_SOLID_FILL/F_BEVEL box,
        // gradients, borders). No per-type hardcoding.
        let rf = o.rflags as u32;
        const BG_BITS: u32 = cm_render::panel::F_TRANSPARENT
            | cm_render::panel::F_HGRADIENT
            | cm_render::panel::F_VGRADIENT
            | cm_render::panel::F_SOLID_FILL
            | cm_render::panel::F_BEVEL
            | cm_render::panel::F_SUNKEN
            | cm_render::panel::F_BORDER;
        if rf & BG_BITS != 0 {
            s.draw_panel(o.l, o.t, o.r, o.b, rf, COLOR_BUTTON);
        }
        let font_id = o.font as u8;
        let font = fonts.slot(font_id);
        let tag;
        let text: &str = if o.text.is_empty() {
            tag = format!("<obj {} font {}>", o.id, o.font);
            &tag
        } else {
            o.text
        };
        let cx = o.l + ((o.r - o.l) - font.text_width(text)) / 2;
        let cy = o.t + ((o.b - o.t) - slot_line_height(font_id)) / 2;
        s.blit_string(cx.max(o.l + 2), cy.max(o.t), font, (230, 233, 240), text);
    }

    // ---- tab strips: CAPTURED strip areas + ported FUN_005d7070 --------
    // The tab builder emits its own background areas (callers 0x5d72f3 /
    // 0x5d7371, flags=2 dim + flags=1 container) with real rects — use
    // those, not reconstructed geometry. Each strip claims the tab-label
    // objects whose capture id falls between its container area's id and
    // the next tab-builder area's id (creation order is the association).
    let strip_areas: Vec<_> = scr
        .areas
        .iter()
        .filter(|a| is_tab_label(a.caller) && a.flags == 1 && a.r > a.l)
        .collect();
    for (si, strip) in strip_areas.iter().enumerate() {
        let next_area_id = strip_areas.get(si + 1).map(|a| a.id).unwrap_or(u32::MAX);
        let claimed: Vec<&(u32, &str, bool)> = tab_labels
            .iter()
            .filter(|(id, _, _)| *id > strip.id && *id < next_area_id)
            .collect();
        if claimed.is_empty() {
            continue;
        }
        let selected_id = claimed
            .iter()
            .position(|(_, _, sel)| *sel)
            .map(|i| i as i32)
            .unwrap_or(-1);
        let recs: Vec<TabRecord> = claimed
            .iter()
            .enumerate()
            .map(|(i, (_, l, _))| TabRecord::simple(i as i32, l))
            .collect();
        let rect = (strip.l, strip.t, strip.r, strip.b);
        let (mut lx, mut rx) = (strip.l, strip.r);
        draw_tab_strip(&mut s, &mut fonts, rect, &recs, selected_id,
                       &mut lx, &mut rx, false, &pal);
    }

    // ---- nav row: CAPTURED panel area + ported FUN_005d75b0 cols -------
    // The nav panel is a captured area from the 0x415acd emitter; only the
    // [3,1] column split comes from the ported nav-bar code (News-verified
    // FUN_005d75b0 semantics), and each label uses its captured font.
    let nav_area = scr
        .areas
        .iter()
        .find(|a| (0x415a00..0x415b00).contains(&a.caller) && a.r > a.l);
    if let (Some(area), false) = (nav_area, nav_labels.is_empty()) {
        let nav_rect = (area.l, area.t, area.r, area.b);
        s.draw_panel(nav_rect.0, nav_rect.1, nav_rect.2, nav_rect.3,
                     F_SOLID_FILL | F_BEVEL, COLOR_BUTTON);
        let lo = rebuild_layout(nav_rect, 1, &[3, 1], &[1], false);
        for (i, (label, font_id)) in nav_labels.iter().enumerate().take(2) {
            let (l, t, r, b) = lo.cell(i, 0);
            if i == 1 {
                // seam between the two cells, one bevel line (News §11)
                s.draw_hollow_rect(l, t, l + 1, b, (90, 90, 90));
            }
            let font = fonts.slot(*font_id);
            let cx = l + ((r - l) - font.text_width(label)) / 2;
            let cy = t + ((b - t) - slot_line_height(*font_id)) / 2;
            s.blit_string(cx, cy, font, (255, 255, 255), label);
        }
    }

    // ---- honest footer --------------------------------------------------
    let font = fonts.slot(3);
    let note = format!(
        "gen:{}  drawn: {} objs + {} tabs + {} nav | excluded: {} sidebar | {} state-dependent (layout engine pending)",
        scr.name, screen_objs.len(), tab_labels.len(), nav_labels.len(),
        n_sidebar, n_state,
    );
    s.blit_string(4, (Surface::H as i32) - 16, font, (150, 155, 170), &note);

    let mut argb = vec![0u32; Surface::W * Surface::H];
    s.to_argb(&mut argb);
    let out = args.next().unwrap_or_else(|| format!("{slug}_gen.bmp"));
    write_bmp(&out, Surface::W, Surface::H, &argb).expect("write bmp");
    eprintln!("wrote {out}");
}
