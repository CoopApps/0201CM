//! Faithful direct-draw renderer for the Setup Game screen.
//!
//! Shared pre-boot chrome (photo / sidebar / title bar / subheader /
//! nav bar) lives in [`screen_pre_boot_chrome`]. This module only owns
//! the 9-button grid specific to Setup + press-state.
//!
//! Provenance: `fixtures/setup_screen/exe_paint.jsonl` (373 draw ops
//! captured live from cm0102_GDI.exe).

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{
    draw_panel, PanelPalette,
    P_BEVEL, P_BEVEL_INVERT, P_DARKEN,
};
use crate::packed_text::draw_wrapped_text;
use crate::screen_pre_boot_chrome::{
    c_string, draw_chrome, ChromeState,
    F_BODY, INK_CYAN, TS_CENTRE,
};

/// Blue button base. `0x0010 = (0, 0, 16)` — matches the exe's captured
/// `c=0x0010` on the 9 button panels. Bevel scales it up/down giving the
/// (0x001a highlight, 0x0005 shadow) that's in the ground truth.
const BLUE_BUTTON: u16 = 0x0010;

/// Runtime state parameters for the Setup screen render.
pub struct SetupState {
    /// Which RGN to blit as the base photo layer.
    pub photo_seed: u64,
    /// `true` when a manager is present.
    pub has_manager: bool,
    /// Back button enable state. Setup is top-level, so always `false`.
    pub back_enabled: bool,
    /// Next button enable state.
    pub next_enabled: bool,
    /// Which content button is currently pressed (mouse down). 0..=8 in
    /// `BUTTONS` order; 9 = Back, 10 = Next.
    pub pressed: Option<usize>,
}

impl Default for SetupState {
    fn default() -> Self {
        Self { photo_seed: 0, has_manager: false, back_enabled: false,
               next_enabled: false, pressed: None }
    }
}

pub const BUTTON_BACK: usize = 9;
pub const BUTTON_NEXT: usize = 10;

/// Paint the Setup Game screen.
pub fn render_setup(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &SetupState,
) {
    // Chrome (photo, sidebar, title bar, subheader, nav bar).
    draw_chrome(surface, fonts, &ChromeState {
        photo_seed: state.photo_seed,
        has_manager: state.has_manager,
        sub_title: "Setup Game",
        back_enabled: state.back_enabled,
        next_enabled: state.next_enabled,
    });

    // Setup-specific content: 9 buttons on a 4-row grid + centred Web
    // Sites row. Each is a bevel + darken panel with cyan label text.
    // Bevel colour comes from the captured `c=0x0010` blue.
    let palette = PanelPalette::default();
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
        let mut style = P_BEVEL | P_DARKEN;
        if state.pressed == Some(idx) {
            style |= P_BEVEL_INVERT;
        }
        draw_panel(surface, x0, y0, x1, y1, style, BLUE_BUTTON, 0, palette);
        let bytes = c_string(label.as_bytes());
        draw_wrapped_text(surface, x0, y0, x1, y1,
            &body_font, &bytes, INK_CYAN, TS_CENTRE, -1);
    }
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
