//! Shared chrome painted by every pre-boot screen — photo background,
//! navy vgradient sidebar with 5 entries, red title bar, "Championship
//! Manager 2001/02" text, subheader band + screen title, Back / Next
//! buttons at the bottom.
//!
//! The user confirmed (Setup → Select Leagues capture): the sidebar and
//! banner geometry are identical across pre-boot screens. Factoring this
//! chrome out means each screen's `screen_*_faithful.rs` only encodes
//! the screen-specific content.
//!
//! Every constant / rect here traces back to the exe paint captures:
//! `fixtures/setup_screen/exe_paint.jsonl` and
//! `fixtures/leagues_screen/exe_paint_fb.jsonl.gz`. Op indices in the
//! comments are into `structure.txt` in the matching fixture directory.

use crate::font::Fonts;
use crate::image::Image;
use crate::packed::PackedSurface;
use crate::packed_panel::{
    draw_panel, PanelPalette,
    P_BEVEL, P_BEVEL_INVERT, P_DARKEN, P_SAMPLE_BG, P_SOLID_FILL,
};
use crate::packed_text::{draw_wrapped_text, W_SHADOW, W_WRAP};

// -----------------------------------------------------------------------
// Shared palette + fonts
// -----------------------------------------------------------------------

/// Red title-bar fill. `0x7c00 = (31, 0, 0)`.
pub const RED_TITLE: u16 = 0x7c00;
/// Yellow accent (Version box, sub-title text). `0x7fe0 = (31, 31, 0)`.
pub const YELLOW_PATTERN: u16 = 0x7fe0;
/// Near-white cyan text / cyan bevel accent. `0x739c = (28, 28, 28)`.
pub const INK_CYAN: u16 = 0x739c;
pub const INK_YELLOW: u16 = YELLOW_PATTERN;
/// Faded mid-cyan for disabled sidebar text. Half-brightness of INK_CYAN.
pub const CYAN_FADED: u16 = 0x39ce;
/// Grey bottom-bar (Back/Next) fill. `0x4210 = (16, 16, 16)`.
pub const GREY_BAR: u16 = 0x4210;
/// Sidebar gradient top colour. `0x0010` — b channel maxed, everything
/// else zero. Fades to black at y=599.
pub const SIDEBAR_TOP: u16 = 0x0010;

/// Font slot 1 — arial_narrow_10. Sidebar entries + small in-content labels.
pub const F_SMALL: u8 = 1;
/// Font slot 3 — arial_14. Button labels + Back/Next.
pub const F_BODY:  u8 = 3;
/// Font slot 6 — trade_cond_24_bold. Sub-header title.
pub const F_SUB:   u8 = 6;
/// Font slot 7 — trade_cond_28_bold. Main title bar.
pub const F_TITLE: u8 = 7;

/// Wrapped-text base style: centred horizontally + vertically.
pub const TS_CENTRE: u32 = 0x0c;
/// Same as `TS_CENTRE` but with word wrap enabled — used for the sidebar
/// two-line entries like "Version\n4.0.00" and "Restart\nGame".
pub const TS_WRAP: u32 = TS_CENTRE | W_WRAP;

// -----------------------------------------------------------------------
// Sidebar spec (constant across pre-boot screens)
// -----------------------------------------------------------------------

/// Vertical span the sidebar occupies (x in 0..=89).
pub const SIDEBAR_X_END: i32 = 89;

// -----------------------------------------------------------------------
// Shared runtime state
// -----------------------------------------------------------------------

/// Everything the pre-boot chrome needs to know about the current session
/// to paint itself correctly.
pub struct ChromeState<'a> {
    /// Selects the rotating RGN photo background (per-screen fresh seed).
    pub photo_seed: u64,
    /// Enables the "Add Manager" sidebar entry (present only after the
    /// user has added a manager to the profile). Setup with no manager
    /// yet ⇒ `false` → the entry renders faded.
    pub has_manager: bool,
    /// Subheader title: "Setup Game", "Select League(s)", "Enter Name" etc.
    pub sub_title: &'a str,
    /// Enable state for the bottom Back button.
    pub back_enabled: bool,
    /// Enable state for the bottom Next button.
    pub next_enabled: bool,
}

// -----------------------------------------------------------------------
// Public helpers — each pre-boot screen wraps a call to `draw_chrome`
// then paints its screen-specific content on top.
// -----------------------------------------------------------------------

/// Paint everything a pre-boot screen shares — photo, sidebar, banner,
/// nav bar. After this returns, the screen-specific content area is
/// (100..790, 145..535) minus whatever the screen wants inside.
pub fn draw_chrome(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &ChromeState<'_>,
) {
    // 1. Photo background.
    blit_photo(surface, state.photo_seed);

    // 2. Sidebar gradient + 5 entries.
    draw_sidebar(surface, fonts, state.has_manager);

    // 3. Title bar (red panel with red-scaled bevel) + title text.
    let palette = PanelPalette::default();
    draw_panel(surface, 100, 10, 790, 70,
        P_SOLID_FILL | P_BEVEL, RED_TITLE, 0, palette);
    let title_font = fonts.pixel_slot(F_TITLE).clone();
    let title = c_string(b"Championship Manager 2001/02");
    draw_wrapped_text(surface, 100, 10, 790, 70,
        &title_font, &title, INK_CYAN, TS_CENTRE, -1);

    // 4. Subheader (P_DARKEN band + yellow text).
    draw_panel(surface, 100, 80, 790, 125, P_DARKEN, 0, YELLOW_PATTERN, palette);
    let sub_font = fonts.pixel_slot(F_SUB).clone();
    let sub = c_string(state.sub_title.as_bytes());
    draw_wrapped_text(surface, 100, 80, 790, 125,
        &sub_font, &sub, INK_YELLOW, TS_CENTRE, -1);

    // 5. Bottom Back + Next.
    let body_font = fonts.pixel_slot(F_BODY).clone();
    draw_nav_button(surface, &body_font, 100, 555, 617, 590, "Back",
                    state.back_enabled);
    draw_nav_button(surface, &body_font, 619, 555, 790, 590, "Next",
                    state.next_enabled);
}

// -----------------------------------------------------------------------
// Photo background
// -----------------------------------------------------------------------

/// 565 → 555 conversion. The exe blits RGN photos (on-disk 565) through
/// this same downcast at present time to match the 555 back-buffer.
fn c565_to_555(v: u16) -> u16 {
    let r = (v >> 11) & 0x1f;
    let g = ((v >> 5) & 0x3f) >> 1;
    let b = v & 0x1f;
    (r << 10) | (g << 5) | b
}

/// Pick one of the `D:/cm0102/pictures/*.RGN` photos by hashing the seed.
fn choose_photo(photo_seed: u64) -> Option<std::path::PathBuf> {
    let dir = std::path::Path::new("D:/cm0102/pictures");
    let entries: Vec<_> = std::fs::read_dir(dir).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension().and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("rgn"))
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();
    if entries.is_empty() { return None; }
    let n = entries.len() as u64;
    Some(entries[(photo_seed % n) as usize].clone())
}

/// Blit a random RGN photo as the base layer. Missing directory (test
/// environment) leaves the surface untouched.
pub fn blit_photo(surface: &mut PackedSurface, photo_seed: u64) {
    let Some(path) = choose_photo(photo_seed) else { return };
    let Ok(img) = Image::load_rgn(&path) else { return };
    let w = surface.width.min(img.w as i32);
    let h = surface.height.min(img.h as i32);
    for y in 0..h {
        for x in 0..w {
            let src = img.px[(y as usize) * img.w + (x as usize)];
            let dst_idx = (y * surface.pitch_pixels + x) as usize;
            surface.buf[dst_idx] = c565_to_555(src);
        }
    }
}

// -----------------------------------------------------------------------
// Sidebar
// -----------------------------------------------------------------------

/// Full sidebar strip (0..89) — navy vgradient + Version + arrows +
/// Add Manager + Restart Game + Exit Game.
pub fn draw_sidebar(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    has_manager: bool,
) {
    // Smooth per-pixel navy gradient — the exe's P_VGRADIENT primitive
    // bands every ~6 rows because it uses integer division; the exe's
    // real sidebar gradient is drawn by a helper we don't hook, so we
    // paint our own smooth version.
    draw_smooth_vgradient(surface, 0, 0, SIDEBAR_X_END, 599, SIDEBAR_TOP);

    // Sidebar entries — each is a P_SAMPLE_BG panel (fill AND bevel
    // colour derive from the pixel at (x0, y0), which is the local
    // gradient blue) with wrapped text on top.
    sidebar_entry(surface, fonts, 5, 10, 85, 53, "Version\n4.0.00",
                  INK_YELLOW, /*sunken*/ true);
    sidebar_entry(surface, fonts, 5, 55, 44, 98, "<<<",
                  INK_CYAN, false);
    sidebar_entry(surface, fonts, 46, 55, 85, 98, ">>>",
                  INK_CYAN, false);
    // Add Manager — bevel always visible; ink flips between bright
    // and faded based on whether a manager exists yet.
    sidebar_entry(surface, fonts, 5, 100, 85, 143, "Add\nManager",
                  if has_manager { INK_CYAN } else { CYAN_FADED }, false);
    sidebar_entry(surface, fonts, 5, 145, 85, 187, "Restart\nGame",
                  INK_YELLOW, false);
    sidebar_entry(surface, fonts, 5, 189, 85, 232, "Exit\nGame",
                  INK_YELLOW, false);
}

/// One sidebar entry — sample-bg bevel panel + wrapped text.
fn sidebar_entry(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    x0: i32, y0: i32, x1: i32, y1: i32,
    label: &str,
    ink: u16,
    sunken: bool,
) {
    let palette = PanelPalette::default();
    let mut style = P_SAMPLE_BG | P_SOLID_FILL | P_BEVEL;
    if sunken { style |= P_BEVEL_INVERT; }
    draw_panel(surface, x0, y0, x1, y1, style, 0, 0, palette);
    let font = fonts.pixel_slot(F_SMALL).clone();
    let bytes = c_string(label.as_bytes());
    draw_wrapped_text(surface, x0, y0, x1, y1, &font, &bytes, ink, TS_WRAP, -1);
}

/// Smooth per-pixel vertical gradient from `top_colour` at y0 to black
/// at y1. Linear per-channel scale.
fn draw_smooth_vgradient(
    surface: &mut PackedSurface,
    x0: i32, y0: i32, x1: i32, y1: i32,
    top_colour: u16,
) {
    let r_top = ((top_colour >> 10) & 0x1f) as u32;
    let g_top = ((top_colour >>  5) & 0x1f) as u32;
    let b_top = ( top_colour        & 0x1f) as u32;
    let span = (y1 - y0).max(1) as u32;
    for y in y0..=y1 {
        let fade = (span - (y - y0) as u32).min(span);
        let r = ((r_top * fade) / span) as u16;
        let g = ((g_top * fade) / span) as u16;
        let b = ((b_top * fade) / span) as u16;
        let c = (r << 10) | (g << 5) | b;
        for x in x0..=x1 {
            if let Some((cx, cy, _, _)) = surface.clip(x, y, x, y) {
                surface.buf[(cy * surface.pitch_pixels + cx) as usize] = c;
            }
        }
    }
}

// -----------------------------------------------------------------------
// Bottom bar
// -----------------------------------------------------------------------

/// Back / Next button. Panel is ALWAYS painted (grey fill + bevel).
/// Enabled → cyan text. Disabled → embossed text via `W_SHADOW` — the
/// exe's classic disabled look. Bevels do not invert on press — clicking
/// navigates instantly.
pub fn draw_nav_button(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    x0: i32, y0: i32, x1: i32, y1: i32,
    label: &str,
    enabled: bool,
) {
    let palette = PanelPalette::default();
    draw_panel(surface, x0, y0, x1, y1,
        P_SOLID_FILL | P_BEVEL, GREY_BAR, 0, palette);
    let bytes = c_string(label.as_bytes());
    if enabled {
        draw_wrapped_text(surface, x0, y0, x1, y1, font, &bytes,
                          INK_CYAN, TS_CENTRE, -1);
    } else {
        // W_SHADOW derives ink from the background pixel; the ink arg is
        // ignored, so 0 is fine as a "don't-care" cue.
        draw_wrapped_text(surface, x0, y0, x1, y1, font, &bytes,
                          0, TS_CENTRE | W_SHADOW, -1);
    }
}

// -----------------------------------------------------------------------
// String helper
// -----------------------------------------------------------------------

pub fn c_string(bytes: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(bytes.len() + 1);
    v.extend_from_slice(bytes);
    v.push(0);
    v
}
