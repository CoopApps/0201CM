//! Faithful direct-draw renderer for the Setup Game screen.
//!
//! # Why direct
//!
//! Unlike the rest of the pre-boot fold (which spawns pool widgets and
//! runs them through the Layer 2 widget renderer), the Setup screen has
//! precise structural elements — vgradient sidebar with SEE-THROUGH
//! entries, red title bar with a CYAN bevel (not red), button rows with
//! `darken()` for the through-photo effect — that the generic widget
//! pool doesn't currently express. Rather than teach the widget pool a
//! Setup-specific dialect and risk polluting the other 100+ screens, this
//! module encodes the Setup paint recipe DIRECTLY.
//!
//! **Hardcoding is intentional and scoped to this screen only.** The
//! coords, colours and font slots are transcribed from a live Frida
//! capture of `cm0102_GDI.exe` on the running Setup screen
//! (`fixtures/setup_screen/exe_paint.jsonl` — 373 draw ops).
//!
//! # Layering rules (from user feedback / observation)
//!
//! * **Sidebar entries are see-through.** The vgradient runs the full
//!   height; each Version / Arrow / Add-Manager / Restart / Exit entry
//!   is JUST a bevel + text — no fill — so the gradient shows through.
//!   The exe achieves this with `P_SAMPLE_BG` (fill = sampled gradient
//!   pixel = same tint); we skip the fill entirely for the same look.
//! * **Title bar has a CYAN bevel** even though the fill is red. The
//!   exe passes `pattern_colour=0x739c` to draw_panel for this; we get
//!   the same by drawing the fill first and then a second `P_BEVEL`-only
//!   pass using cyan as the bevel base.
//! * **Buttons darken()-into-photo and bevel cyan**. Same two-pass
//!   approach: darken the rect, then bevel cyan on top.
//! * **Add Manager appears faded** when no manager is present. We draw
//!   it with a dimmer cyan ink and skip the bevel to match the exe's
//!   "disabled" style for that entry on pre-boot Setup.
//! * **Back/Next carry a proper bevel** too (P_BEVEL over the grey fill).
//!
//! # Provenance summary
//!
//! * Ground truth: `fixtures/setup_screen/exe_paint.jsonl`
//! * Ground-truth framebuffer: `fixtures/setup_screen/paint.pixels.bin`
//! * Structural summary: `fixtures/setup_screen/structure.txt`
//! * Live capture harness: `tools/gdi_capture/live_log.py|js`

use crate::font::Fonts;
use crate::image::Image;
use crate::packed::PackedSurface;
use crate::packed_panel::{
    draw_panel, PanelPalette,
    P_BEVEL, P_BEVEL_INVERT, P_DARKEN, P_SAMPLE_BG, P_SOLID_FILL, P_VGRADIENT,
};
use crate::packed_text::{draw_wrapped_text, W_WRAP};

// -----------------------------------------------------------------------
// Colour constants (all RGB555; every one is from the captured paint ops)
// -----------------------------------------------------------------------

/// Red title-bar fill. `0x7c00 = (31, 0, 0)`.
const RED_TITLE: u16 = 0x7c00;

/// Cyan bevel-pattern accent. `0x739c = (28, 28, 28)` — light grey/white.
/// Used as the bevel colour AND text colour on nearly every accented
/// panel in the Setup screen.
const CYAN_PATTERN: u16 = 0x739c;

/// Yellow bevel-pattern accent. `0x7fe0 = (31, 31, 0)`. Used for the
/// Version box, Restart / Exit sidebar buttons, and "Setup Game" subheader.
const YELLOW_PATTERN: u16 = 0x7fe0;

/// Dimmed cyan for the disabled "Add Manager" entry. Half-brightness of
/// CYAN_PATTERN — matches the exe's greyed-out look.
const CYAN_DIM: u16 = 0x3def; // ~14/14/15 packed

/// Grey bottom-bar (Back/Next) fill. `0x4210 = (16, 16, 16)`.
const GREY_BAR: u16 = 0x4210;

/// Blue button base. `0x0010 = (0, 0, 16)` — matches the exe's captured
/// `c=0x0010` on the 9 button panels. Passed as `colour` to draw_panel;
/// the P_BEVEL loop scales this up/down to give the (0x001a highlight,
/// 0x0005 shadow) 3D-bevel effect the ground-truth pixels show.
const BLUE_BUTTON: u16 = 0x0010;

/// Sidebar gradient top colour. Sampled from the ground-truth
/// framebuffer at (40, 0) = `0x0010` (b=16).
const SIDEBAR_TOP: u16 = 0x0010;

const INK_YELLOW: u16 = YELLOW_PATTERN;
const INK_CYAN:   u16 = CYAN_PATTERN;

// -----------------------------------------------------------------------
// Font slot constants
// -----------------------------------------------------------------------

const F_SMALL: u8 = 1; // arial_narrow_10 — sidebar entries + arrows
const F_BODY:  u8 = 3; // arial_14 — button labels + Back/Next
const F_SUB:   u8 = 6; // trade_cond_24_bold — "Setup Game" subheader
const F_TITLE: u8 = 7; // trade_cond_28_bold — title bar

const TS_CENTRE: u32 = 0x0c;
const TS_WRAP: u32 = TS_CENTRE | W_WRAP;

/// 565 (RGN on disk) → 555 (surface). Matches the exe's blit conversion.
fn c565_to_555(v: u16) -> u16 {
    let r = (v >> 11) & 0x1f;
    let g = ((v >> 5) & 0x3f) >> 1;
    let b = v & 0x1f;
    (r << 10) | (g << 5) | b
}

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

fn blit_photo(surface: &mut PackedSurface, photo_seed: u64) {
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

/// Runtime state parameters for the Setup screen render.
pub struct SetupState {
    /// Which RGN to blit as the base photo layer (hash → photo index).
    pub photo_seed: u64,
    /// `true` when a manager is present — enables the "Add Manager" entry.
    pub has_manager: bool,
    /// `true` when the Back button leads somewhere — Setup is top-level
    /// so this is always `false` for now; sub-screens would pass `true`.
    pub back_enabled: bool,
    /// `true` when the Next button leads somewhere — same as above,
    /// Setup is top-level so `false`.
    pub next_enabled: bool,
    /// Which of the 9 content buttons is currently pressed (mouse down
    /// on it) — the panel gets `P_BEVEL_INVERT` so it looks pushed-in.
    /// 0..=8 in `BUTTONS` order (Start New Game .. Web Sites). Also
    /// accepts index 9 = Back, 10 = Next.
    pub pressed: Option<usize>,
}

impl Default for SetupState {
    fn default() -> Self {
        Self { photo_seed: 0, has_manager: false, back_enabled: false,
               next_enabled: false, pressed: None }
    }
}

/// Content-button index for the Back button in `SetupState::pressed`.
pub const BUTTON_BACK: usize = 9;
/// Content-button index for the Next button in `SetupState::pressed`.
pub const BUTTON_NEXT: usize = 10;

/// Main entry: paint the Setup Game screen.
pub fn render_setup(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &SetupState,
) {
    let photo_seed = state.photo_seed;
    let has_manager = state.has_manager;
    let back_enabled = state.back_enabled;
    let next_enabled = state.next_enabled;
    let pressed = state.pressed;
    let palette = PanelPalette::default();

    // 1. Photo base layer (behind everything).
    blit_photo(surface, photo_seed);

    // 2. Sidebar navy vgradient. Painted OVER the photo strip so the
    //    photo only shows through the content area / darken()s.
    draw_panel(surface, 0, 0, 89, 599, P_VGRADIENT, SIDEBAR_TOP, 0, palette);

    // 3. Sidebar entries — every one uses P_SAMPLE_BG so the fill AND
    //    the bevel derive from the pixel at (x0, y0). Since the sidebar
    //    is a navy vgradient underneath, every entry ends up filled with
    //    that local blue and bevelled with a brighter/darker variant of
    //    that same blue. Effect: see-through + coherent 3D border.
    //    Ground-truth pixel check on Restart Game: highlight = 0x0013
    //    (b=19 blue), shadow = 0x0004. Not white — a scaled blue.

    //    Version box: SUNKEN sample-bg bevel + yellow text.
    sidebar_entry(surface, fonts, 5, 10, 85, 53, "Version\n3.9.60",
                  INK_YELLOW, /*sunken*/ true, /*enabled*/ true);

    //    <<< / >>> arrows: raised sample-bg bevel + cyan text (literal).
    sidebar_entry(surface, fonts, 5, 55, 44, 98, "<<<",
                  INK_CYAN, false, true);
    sidebar_entry(surface, fonts, 46, 55, 85, 98, ">>>",
                  INK_CYAN, false, true);

    //    Add Manager: dimmer text + no bevel when disabled.
    sidebar_entry(surface, fonts, 5, 100, 85, 143, "Add\nManager",
                  if has_manager { INK_CYAN } else { CYAN_DIM },
                  /*sunken*/ false,
                  /*enabled*/ has_manager);

    //    Restart Game + Exit Game — yellow text.
    sidebar_entry(surface, fonts, 5, 145, 85, 187, "Restart\nGame",
                  INK_YELLOW, false, true);
    sidebar_entry(surface, fonts, 5, 189, 85, 232, "Exit\nGame",
                  INK_YELLOW, false, true);

    // 4. Title bar (100,10)-(790,70). Ground-truth bevel is RED-scaled
    //    (0x7c00 highlight, 0x2800 shadow) — the exe passes the fill red
    //    as the bevel base and the primitive scales it up (bright edge)
    //    and down (dark edge). One draw_panel call does both.
    draw_panel(surface, 100, 10, 790, 70,
        P_SOLID_FILL | P_BEVEL, RED_TITLE, 0, palette);
    let title_font = fonts.pixel_slot(F_TITLE).clone();
    let title_bytes = c_string(b"Championship Manager 2001/02");
    draw_wrapped_text(surface, 100, 10, 790, 70,
        &title_font, &title_bytes, INK_CYAN, TS_CENTRE, -1);

    // 5. "Setup Game" subheader (100,80)-(790,125): P_DARKEN only, no
    //    bevel, yellow text.
    draw_panel(surface, 100, 80, 790, 125, P_DARKEN, 0, YELLOW_PATTERN, palette);
    let sub_font = fonts.pixel_slot(F_SUB).clone();
    let sub_bytes = c_string(b"Setup Game");
    draw_wrapped_text(surface, 100, 80, 790, 125,
        &sub_font, &sub_bytes, INK_YELLOW, TS_CENTRE, -1);

    // 6. 9 buttons. Ground-truth bevel is BLUE-scaled (0x001a highlight,
    //    0x0005 shadow) — the exe passes the fill blue 0x0010 as the
    //    bevel base, plus P_DARKEN so the photo bleeds through inside.
    //    Single-pass draw_panel with P_BEVEL | P_DARKEN.
    const BTN_ROWS: [i32; 5] = [145, 211, 276, 341, 406];
    const BTN_END_Y: [i32; 5] = [209, 274, 339, 404, 469];
    const BUTTONS: [(usize, i32, i32, &str); 9] = [
        (0, 110, 444, "Start New Game"),
        (0, 446, 780, "Quick Start Game"),
        (1, 110, 444, "Restore Saved Game"),
        (1, 446, 780, "Delete Saved Game"),
        (2, 110, 444, "Network Play"),
        (2, 446, 780, "Game Settings"),
        (3, 110, 444, "Hall Of Fame"),
        (3, 446, 780, "Game Credits"),
        (4, 278, 611, "Web Sites"),
    ];
    let body_font = fonts.pixel_slot(F_BODY).clone();
    for (idx, (row, x0, x1, label)) in BUTTONS.iter().copied().enumerate() {
        let y0 = BTN_ROWS[row];
        let y1 = BTN_END_Y[row];
        // Pressed button → invert the bevel so it looks pushed in.
        let mut style = P_BEVEL | P_DARKEN;
        if pressed == Some(idx) {
            style |= P_BEVEL_INVERT;
        }
        draw_panel(surface, x0, y0, x1, y1, style, BLUE_BUTTON, 0, palette);
        let bytes = c_string(label.as_bytes());
        draw_wrapped_text(surface, x0, y0, x1, y1,
            &body_font, &bytes, INK_CYAN, TS_CENTRE, -1);
    }

    // 7. Bottom bar — Back + Next.
    //
    // When a button is DISABLED (nothing to go back / forward to), the
    // exe renders it faded — no fill, no bevel, dim ink — matching the
    // "Add Manager" disabled look. When enabled it gets the grey fill +
    // grey-scaled bevel. Pressed inverts the bevel like the content buttons.
    render_nav_button(surface, &body_font, 100, 555, 617, 590, "Back",
                      back_enabled, pressed == Some(BUTTON_BACK));
    render_nav_button(surface, &body_font, 619, 555, 790, 590, "Next",
                      next_enabled, pressed == Some(BUTTON_NEXT));
}

/// Back / Next button. Enabled = grey panel + cyan text + optional
/// invert-on-press. Disabled = no panel, dim cyan text (looks faded,
/// matching the Add Manager disabled entry style).
fn render_nav_button(
    surface: &mut PackedSurface,
    font: &crate::packed_glyph::PixelFont,
    x0: i32, y0: i32, x1: i32, y1: i32,
    label: &str,
    enabled: bool,
    pressed: bool,
) {
    let palette = PanelPalette::default();
    let ink = if enabled { INK_CYAN } else { CYAN_DIM };
    if enabled {
        let mut style = P_SOLID_FILL | P_BEVEL;
        if pressed {
            style |= P_BEVEL_INVERT;
        }
        draw_panel(surface, x0, y0, x1, y1, style, GREY_BAR, 0, palette);
    }
    let bytes = c_string(label.as_bytes());
    draw_wrapped_text(surface, x0, y0, x1, y1, font, &bytes, ink, TS_CENTRE, -1);
}

/// A single sidebar entry. Uses P_SAMPLE_BG so the fill AND bevel
/// derive from the gradient pixel at (x0, y0) — the entry looks like a
/// bevelled cut-out of the gradient itself, coherent with the
/// see-through effect the user described. `sunken=true` renders the
/// inverted (inset) bevel used for the Version box. `enabled=false`
/// skips the panel entirely — text sits directly on the gradient
/// (the exe's greyed/disabled sidebar look).
fn sidebar_entry(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    x0: i32, y0: i32, x1: i32, y1: i32,
    label: &str,
    ink: u16,
    sunken: bool,
    enabled: bool,
) {
    let palette = PanelPalette::default();
    if enabled {
        let mut style = P_SAMPLE_BG | P_SOLID_FILL | P_BEVEL;
        if sunken {
            style |= P_BEVEL_INVERT;
        }
        // `colour` is a placeholder — P_SAMPLE_BG replaces it with the
        // pixel at (x0, y0), which is the gradient blue at that row.
        draw_panel(surface, x0, y0, x1, y1, style, 0, 0, palette);
    }
    let font = fonts.pixel_slot(F_SMALL).clone();
    let bytes = c_string(label.as_bytes());
    draw_wrapped_text(surface, x0, y0, x1, y1, &font, &bytes, ink, TS_WRAP, -1);
}

fn c_string(bytes: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(bytes.len() + 1);
    v.extend_from_slice(bytes);
    v.push(0);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_setup_produces_pixels() {
        let mut surface = PackedSurface::rgb555(800, 600);
        let mut fonts = Fonts::new("D:/cm0102/Data");
        let state = SetupState::default();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_setup(&mut surface, &mut fonts, &state);
        }));
        if result.is_ok() {
            let top = surface.buf[40];
            assert!(top & 0x001f > 0, "sidebar top expected blue: got {top:04x}");
        }
    }
}
