//! Rasterize the News screen from its verified-exact widget geometry
//! (`NewsView::to_widget_pool`), re-verified this session via a LIVE Frida
//! capture of the real, running exe (real save loaded, actually on the
//! News page) rather than the earlier incomplete Unicorn-emulation ground
//! truth. See DIRECTDRAW_CAPTURE_HANDOVER.md §11.
//!
//! This is a geometry + real-text visualization: every rectangle position
//! and every string is real, captured data. Colors are a reasonable
//! placeholder mapping from each widget's real `rflags` bit pattern to a
//! visual treatment (filled+beveled panel / highlighted row / disabled /
//! plain) -- not yet verified against the exe's real palette, which is a
//! separate remaining task. The left sidebar is deliberately not drawn
//! here (separate shared chrome, see [[menu-command-tree]]).
//!
//! Usage: `cargo run -p cm-render --bin render_news -- out.bmp`

use cm_render::font::{slot_line_height, Fonts};
use cm_render::panel::F_SOLID_FILL;
use cm_render::view_render::RenderableView;
use cm_render::widget_pool::{Widget, KIND_LABEL, KIND_ROOT_HOLDER};
use cm_render::Surface;
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
            let r = ((p >> 16) & 0xff) as u8;
            let g = ((p >> 8) & 0xff) as u8;
            let b = (p & 0xff) as u8;
            f.write_all(&[b, g, r])?;
        }
        f.write_all(&pad)?;
    }
    Ok(())
}

// Real measured colors, read directly off the live back buffer while the
// News screen was actually displayed (Lock the real back-buffer surface,
// sample known coordinates, unpack the real RGB565 pixel) -- see
// scratchpad/frida_sample_news_pixels.py and
// DIRECTDRAW_CAPTURE_HANDOVER.md §11. Not guesses: these are what the exe
// itself put on screen.
const COLOR_HEADER: (u8, u8, u8) = (0, 48, 165);
// Tab rows (top AND bottom, selected AND unselected) all measured to the
// SAME fill color -- the "All" tab's visibly different look in the real
// screenshot is a border/highlight effect on top of this fill, not a
// different fill color. An earlier version wrongly assumed a white fill
// for the selected tab.
const COLOR_TAB_FILL: (u8, u8, u8) = (33, 0, 99);
const COLOR_LIST_ROW_SELECTED: (u8, u8, u8) = (132, 0, 0);
// Real measurement, corrected: the date column and headline column are
// DIFFERENT colors (date blue, headline grey) -- every earlier sample
// (x=300, x=700, x=760) landed inside the headline column (x>240), so the
// date column's real color was never actually measured until now. Both
// columns had been wrongly painted with the same grey.
const COLOR_LIST_DATE_COL: (u8, u8, u8) = (0, 0, 132);
const COLOR_LIST_HEADLINE_COL: (u8, u8, u8) = (80, 82, 80);
const COLOR_FILTER_BAR: (u8, u8, u8) = (78, 75, 74);
// Real measurement: Back/Next are grey, NOT the blue used for
// header/tabs -- a flat placeholder color mistake in the first version.
const COLOR_NAV_BUTTON: (u8, u8, u8) = (132, 130, 132);
const COLOR_STORY_PANEL: (u8, u8, u8) = (66, 65, 66); // no photo loaded here; flat placeholder.

fn is_tab_row(t: i32) -> bool {
    t == cm_render::view_render::news_geom::TOP_TABS_T
        || t == cm_render::view_render::news_geom::BOTTOM_TABS_T
}
fn is_nav_button(text: &str) -> bool {
    text == "Back" || text == "Next"
}
fn is_list_row(t: i32) -> bool {
    use cm_render::view_render::news_geom::{LIST_T, LIST_B};
    t >= LIST_T && t < LIST_B
}

fn draw_widget(s: &mut Surface, fonts: &mut Fonts, w: &Widget) {
    let (l, t, r, b) = (w.left, w.top, w.right, w.bottom);
    if r <= l || b <= t {
        return;
    }
    match w.descriptor.kind {
        KIND_ROOT_HOLDER => {
            use cm_render::view_render::news_geom::NAV_T;
            if is_tab_row(t) {
                s.fill_rect(l, t, r, b, COLOR_TAB_FILL);
            } else if t == NAV_T {
                // Real measurement: the nav row is ONE continuous beveled
                // panel (a bevel edge exists only at its outer border plus
                // one internal seam at the real Back/Next split, x=618 --
                // see NAV_SPLIT's comment in view_render.rs) -- not two
                // independently-beveled boxes, which was drawing an extra
                // seam that doesn't exist in the real image.
                s.draw_panel(l, t, r, b, 48, COLOR_NAV_BUTTON);
            } else if w.descriptor.flags & F_SOLID_FILL != 0 {
                s.draw_panel(l, t, r, b, w.descriptor.flags, COLOR_HEADER);
            } else if w.descriptor.flags == 2 {
                use cm_render::view_render::news_geom::STORY_T;
                let color = if t == STORY_T { COLOR_STORY_PANEL } else { COLOR_FILTER_BAR };
                s.fill_rect(l, t, r, b, color);
            }
            // flags==1 containers (header-inner, tab/nav containers): no
            // fill of their own -- purely grouping, matches the exe (no
            // F_SOLID_FILL/F_BEVEL/F_BORDER bit set).
        }
        KIND_LABEL => {
            let font = fonts.slot(w.descriptor.font_id);
            let text = w.descriptor.text.as_str();
            if text.is_empty() {
                return;
            }
            // fg_color doubles as a color override marker for the few
            // widgets whose real color isn't recoverable from rflags alone
            // (currently just the yellow story headline) -- see its set
            // site in view_render.rs for why.
            let text_color = if w.descriptor.fg_color == 0xFFFF00 {
                (240, 220, 40)
            } else if !w.descriptor.enabled {
                (120, 120, 120) // disabled (tflags=44 in the exe): Next / Next Unread.
            } else {
                (255, 255, 255)
            };
            let cx = |text: &str| l + ((r - l) - font.text_width(text)) / 2;
            // Real per-font line height (graphics_font_row_height,
            // 0x005CF7B0 -- see font.rs's slot_line_height doc comment),
            // not a made-up constant. The earlier version used a flat "-8"
            // guess regardless of font, which is why "centered vertically"
            // was never actually true for the header (font 7 = 45px tall,
            // nowhere near an 8px half-height).
            let line_h = slot_line_height(w.descriptor.font_id);
            let cy = t + ((b - t) - line_h) / 2;
            match w.descriptor.flags {
                2096 => {
                    // Selected tab: same real fill as unselected (measured
                    // identical), a lighter border stands in for the real
                    // highlight effect until that's captured separately.
                    s.fill_rect(l, t, r, b, COLOR_TAB_FILL);
                    s.draw_hollow_rect(l, t, r, b, (230, 200, 80));
                    s.blit_string(cx(text), cy, font, (255, 255, 255), text);
                }
                48 if is_nav_button(text) => {
                    // The shared nav-row AREA already drew the one
                    // continuous bevel panel -- just place the text.
                    s.blit_string(cx(text), cy, font, text_color, text);
                }
                48 if is_tab_row(t) => {
                    // Unselected tab label -- real fill, no extra border.
                    s.fill_rect(l, t, r, b, COLOR_TAB_FILL);
                    s.blit_string(cx(text), cy, font, (255, 255, 255), text);
                }
                48 if is_list_row(t) => {
                    // Plain (non-selected) row's DATE column -- real blue
                    // fill (measured separately from the headline column,
                    // which is grey -- see COLOR_LIST_DATE_COL's comment).
                    s.fill_rect(l, t, r, b, COLOR_LIST_DATE_COL);
                    s.blit_string(l + 4, t + 2, font, (255, 255, 255), text);
                }
                48 => {
                    // Header text -- sits on the header area's own fill.
                    s.blit_string(cx(text), cy, font, text_color, text);
                }
                528 => {
                    s.fill_rect(l, t, r, b, COLOR_LIST_ROW_SELECTED);
                    s.blit_string(l + 4, t + 2, font, (255, 255, 255), text);
                }
                2 if is_list_row(t) => {
                    // Plain row's HEADLINE column -- real grey, distinct
                    // from the date column's real blue (see
                    // COLOR_LIST_DATE_COL's comment -- these were wrongly
                    // painted identically before the date column was
                    // actually measured).
                    s.fill_rect(l, t, r, b, COLOR_LIST_HEADLINE_COL);
                    s.blit_string(l + 4, t + 2, font, (255, 255, 255), text);
                }
                _ => {
                    // Plain text (list rows, filter labels, story
                    // headline/body): rflags=1/2, no box of its own.
                    s.blit_string(l + 4, t + 2, font, text_color, text);
                }
            }
        }
        _ => {}
    }
}

fn main() {
    use cm_domain::{GameDate, NewsCategory, NewsItem, NewsView};
    fn item(date_label: &str, headline: &str, body: &str) -> NewsItem {
        NewsItem {
            date: GameDate { year: 2001, month: 10, day: 10 },
            date_label: date_label.to_string(), headline: headline.to_string(),
            body: body.to_string(), category: NewsCategory::Message, unread: false,
        }
    }
    let v = NewsView {
        title: "Christoph Olewicz News".to_string(),
        items: vec![
            item("Wed 10th Oct EVE", "Board reaction to Rhayader game",
                "The Haverfordwest County board of directors are pleased with the 1-0 \
                 Welsh League Cup win against Rhayader Town."),
            item("Wed 10th Oct EVE", "Haverfordwest win in Welsh League Cup Quarter Final", ""),
            item("Mon 8th Oct EVE", "Brazil out for about 2 weeks", ""),
            item("Sun 7th Oct EVE", "Sweden ready for 2002", ""),
            item("Sun 7th Oct EVE", "Burrows resumes full training", ""),
        ],
        selected_tab: 0, tab_count: 8,
        selected_item: 0, nav_back_enabled: true, nav_next_enabled: false,
    };
    let widgets = v.to_widget_pool();

    let mut surface = Surface::new();
    surface.fill(10, 15, 30);

    let fonts_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/cm0102/original_files/Data");
    let mut fonts = Fonts::new(fonts_dir);

    // Draw areas first (backgrounds), then labels on top -- matches the
    // exe's own area-then-widget construction order.
    for w in &widgets {
        if w.descriptor.kind == KIND_ROOT_HOLDER {
            draw_widget(&mut surface, &mut fonts, w);
        }
    }
    for w in &widgets {
        if w.descriptor.kind == KIND_LABEL {
            draw_widget(&mut surface, &mut fonts, w);
        }
    }

    let mut argb = vec![0u32; Surface::W * Surface::H];
    surface.to_argb(&mut argb);

    let out = std::env::args().nth(1).unwrap_or_else(|| "news_render.bmp".to_string());
    write_bmp(&out, Surface::W, Surface::H, &argb).expect("write bmp");
    eprintln!("wrote {out} ({} widgets drawn)", widgets.len());
}
