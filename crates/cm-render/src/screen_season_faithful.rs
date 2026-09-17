//! Faithful direct-draw renderer for the "Select Start Season" screen.
//!
//! Shared pre-boot chrome comes from [`screen_pre_boot_chrome`]. This
//! module owns only the season-picker content: one button per selected
//! league (up to 10 slots on a 5-row × 2-col grid identical to the Setup
//! screen) with the currently-selected one highlighted in yellow.
//!
//! Ground truth: `fixtures/season_screen/exe_paint_fb.jsonl.gz` (37
//! structural ops; captured live 2026-09-05 on cm0102_GDI.exe after
//! selecting England + Russia).
//!
//! Op recipe (from `fixtures/season_screen/structure.txt`):
//! ```text
//!  80 PANEL  (110,145)-(444,209) c=0x0010 p=0x7fe0 s=0x822   <- SELECTED
//!  81 darken (110,145)-(444,209)
//!  90 rect   (109,144)-(445,210) c=0x7fe0 s=2                 <- yellow frame
//!  95 WRAP   (110,145)-(444,209) f=3 c=0x7fe0 'England 01/02' <- yellow text
//!  97 PANEL  (446,145)-(780,209) c=0x0010 p=0x739c s=0x22    <- UNSELECTED
//!  98 darken (446,145)-(780,209)
//! 107 WRAP   (446,145)-(780,209) f=3 c=0x739c 'Russia 2002'
//! ```
//!
//! Style decode:
//! - `0x22 = P_BEVEL | P_DARKEN` — same recipe as Setup's blue buttons.
//! - `0x822 = 0x800 | 0x22 = P_OUTER_HIGHLIGHT | P_BEVEL | P_DARKEN`
//!   — adds the outer highlight ring on top of the same base.

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{
    draw_panel, PanelPalette,
    P_BEVEL, P_BEVEL_INVERT, P_DARKEN, P_OUTER_HIGHLIGHT,
};
use crate::packed_text::draw_wrapped_text;
use crate::screen_pre_boot_chrome::{
    c_string, draw_chrome, ChromeState,
    F_BODY, INK_CYAN, INK_YELLOW, TS_CENTRE, YELLOW_PATTERN,
};

/// Blue button base — same value the exe passes at op #80/#97.
const BLUE_BUTTON: u16 = 0x0010;

/// Button grid — same as Setup's rows 0..4 for consistency and because
/// the captured exe values match Setup's row 0 exactly.
const BTN_ROWS: [i32; 5] = [145, 211, 276, 341, 406];
const BTN_END_Y: [i32; 5] = [209, 274, 339, 404, 469];
const COL_L_X: (i32, i32) = (110, 444);
const COL_R_X: (i32, i32) = (446, 780);
/// Centred single-button rect (used when a row has only one entry — the
/// exe centres it exactly like Setup's Web Sites button). Op #109 in the
/// 3-button capture: `PANEL (278,211)-(611,274)`.
const COL_C_X: (i32, i32) = (278, 611);

/// Runtime state parcel.
pub struct SeasonState<'a> {
    pub photo_seed: u64,
    pub has_manager: bool,
    /// The season options (one per selected league). Displayed on the
    /// grid in slot order.
    pub rows: &'a [&'a str],
    /// Which row is currently the highlighted selection.
    pub selected: usize,
    pub back_enabled: bool,
    pub next_enabled: bool,
    /// Which button (0..rows.len()) is currently being pressed. Setup
    /// convention: 9 = Back, 10 = Next (but nav buttons don't invert).
    pub pressed: Option<usize>,
}

pub fn render_season(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &SeasonState<'_>,
) {
    draw_chrome(surface, fonts, &ChromeState {
        photo_seed: state.photo_seed,
        has_manager: state.has_manager,
        sub_title: "Select Start Season",
        left_nav_label: "Back",
        right_nav_label: "Next",
        back_enabled: state.back_enabled,
        next_enabled: state.next_enabled,
    });

    let palette = PanelPalette::default();
    let body_font = fonts.pixel_slot(F_BODY).clone();

    for (idx, label) in state.rows.iter().copied().enumerate() {
        // 2-col grid — even idx → left, odd → right — UNLESS this is
        // the last button AND the button count is odd, in which case the
        // exe centres it on its row (like Setup's Web Sites cell).
        let row = idx / 2;
        if row >= BTN_ROWS.len() { break; }
        let is_odd_tail = idx == state.rows.len() - 1 && state.rows.len() % 2 == 1;
        let (x0, x1) = if is_odd_tail {
            COL_C_X
        } else if idx % 2 == 0 {
            COL_L_X
        } else {
            COL_R_X
        };
        let y0 = BTN_ROWS[row];
        let y1 = BTN_END_Y[row];

        let is_selected = idx == state.selected;
        let is_pressed  = state.pressed == Some(idx);

        // Base style: bevel + darken (photo shows through), matching the
        // exe's captured s=0x22. Selected adds outer-highlight to give
        // s=0x822. Pressed adds bevel-invert for the "pushed" look.
        let mut style = P_BEVEL | P_DARKEN;
        if is_selected { style |= P_OUTER_HIGHLIGHT; }
        if is_pressed  { style |= P_BEVEL_INVERT; }
        draw_panel(surface, x0, y0, x1, y1, style, BLUE_BUTTON, 0, palette);

        // Selected variant paints a yellow rect frame just outside the
        // panel (op #90 in the capture: rect c=0x7fe0 s=2 on the
        // (x0-1, y0-1)-(x1+1, y1+1) rect).
        if is_selected {
            surface.draw_rectangle(x0 - 1, y0 - 1, x1 + 1, y1 + 1,
                                    2, YELLOW_PATTERN);
        }

        // Label — yellow when selected, cyan otherwise. Font=3.
        let ink = if is_selected { INK_YELLOW } else { INK_CYAN };
        draw_wrapped_text(surface, x0, y0, x1, y1,
            &body_font, &c_string(label.as_bytes()), ink, TS_CENTRE, -1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_season_smoke() {
        let mut surface = PackedSurface::rgb555(800, 600);
        let mut fonts = Fonts::new("D:/cm0102/Data");
        let rows = ["England 01/02", "Russia 2002"];
        let refs: Vec<&str> = rows.iter().copied().collect();
        let state = SeasonState {
            photo_seed: 0, has_manager: false,
            rows: &refs, selected: 0,
            back_enabled: true, next_enabled: true, pressed: None,
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_season(&mut surface, &mut fonts, &state);
        }));
    }
}
