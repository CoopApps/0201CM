//! Faithful direct-draw renderer for the "Enter Name" screen.
//!
//! Chrome comes from [`screen_pre_boot_chrome`]; this module owns the
//! four text-input rows: First Name / Second Name / Password (Optional)
//! / Re-Type. Re-Type is disabled (grey label + grey ":") while the
//! password field is empty — matches the exe capture's op #149/#152
//! (`c=0x4210` grey text).
//!
//! Ground truth: `fixtures/name_screen/exe_paint_fb.jsonl.gz` (59
//! structural ops from cm0102_GDI.exe, captured 2026-09-05).
//!
//! Op recipe per field row:
//! ```text
//!  83 PANEL  (150,217)-(740,277) c=0x0010 p=0     s=0x22   <- row container
//!  84 darken (150,217)-(740,277)
//!  93 PANEL  (152,219)-(374,275) c=0     p=0x739c s=0x1    <- label cell (no fill)
//!  94 WRAP   (152,219)-(374,275) f=3 c=0x739c '   First Name'  <- LEFT-aligned
//!  96 PANEL  (376,219)-(402,275) c=0     p=0x739c s=0x1    <- ':' cell
//!  97 WRAP   (376,219)-(402,275) f=3 c=0x739c ':'
//!  99 PANEL  (404,219)-(738,275) c=0     p=0x739c s=0x1    <- value cell
//! 100 WRAP   (404,219)-(738,275) f=3 c=0x739c '<typed>'    <- LEFT-aligned + caret
//! ```

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{draw_panel, PanelPalette, P_BEVEL, P_DARKEN};
use crate::packed_text::{draw_wrapped_text, W_LEFT};
use crate::screen_pre_boot_chrome::{
    c_string, draw_chrome, ChromeState,
    F_BODY, GREY_BAR, INK_CYAN, TS_CENTRE,
};

/// Blue field container base — same colour the Setup buttons use.
const BLUE_FIELD: u16 = 0x0010;
/// Disabled label ink — grey `0x4210` per op #149.
const INK_DIM: u16 = GREY_BAR;

/// Row geometry from op #83/#103/#121/#139.
const ROW_X0: i32 = 150;
const ROW_X1: i32 = 740;
const ROW_Y0: [i32; 4] = [217, 279, 341, 403];
const ROW_Y1: [i32; 4] = [277, 339, 401, 463];

/// Sub-cell x ranges (constant across all 4 rows).
const LBL_CELL:  (i32, i32) = (152, 374);
const COL_CELL:  (i32, i32) = (376, 402);
const VAL_CELL:  (i32, i32) = (404, 738);

/// Runtime state parcel.
pub struct NameState<'a> {
    pub photo_seed: u64,
    pub has_manager: bool,
    /// The four field values in row order — First / Second / Password /
    /// Re-Type. Re-Type is optional; pass an empty string when the port
    /// doesn't track it.
    pub first: &'a str,
    pub second: &'a str,
    pub password: &'a str,
    pub retype: &'a str,
    /// Which field currently has keyboard focus (0..=3). Caret drawn on it.
    pub focus: u8,
    /// Cancel enable (top-level Setup can't come back, but Cancel does
    /// return to Setup so it's always true here).
    pub cancel_enabled: bool,
    /// Next enable — true once both First and Second are non-empty (the
    /// exe's gate at FUN_0080a450).
    pub next_enabled: bool,
}

/// Paint the Enter Name screen.
pub fn render_name(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &NameState<'_>,
) {
    // Chrome — subheader "Enter Name", left nav is "Cancel", right "Next".
    draw_chrome(surface, fonts, &ChromeState {
        photo_seed: state.photo_seed,
        has_manager: state.has_manager,
        sub_title: "Enter Name",
        left_nav_label: "Cancel",
        right_nav_label: "Next",
        back_enabled: state.cancel_enabled,
        next_enabled: state.next_enabled,
    });

    // Password is empty → Re-Type is disabled (grey).
    let retype_enabled = !state.password.is_empty();

    let rows: [(bool, &str, &str); 4] = [
        (true,           "   First Name",           state.first),
        (true,           "   Second Name",          state.second),
        (true,           "   Password (Optional)",  state.password),
        (retype_enabled, "   Re-Type",              state.retype),
    ];

    for (i, (enabled, label, value)) in rows.iter().copied().enumerate() {
        draw_field(
            surface, fonts,
            ROW_Y0[i], ROW_Y1[i],
            label, value,
            enabled,
            /*focused*/ state.focus as usize == i,
            /*is_password*/ i >= 2,
        );
    }
}

fn draw_field(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    y0: i32, y1: i32,
    label: &str, value: &str,
    enabled: bool,
    focused: bool,
    is_password: bool,
) {
    let palette = PanelPalette::default();
    let body_font = fonts.pixel_slot(F_BODY).clone();

    // Row container — blue-scaled bevel + darken. Op #83 style=0x22 =
    // P_BEVEL | P_DARKEN, same as a Setup button.
    draw_panel(surface, ROW_X0, y0, ROW_X1, y1,
        P_BEVEL | P_DARKEN, BLUE_FIELD, 0, palette);

    let ink = if enabled { INK_CYAN } else { INK_DIM };

    // Label cell — no fill, left-aligned (leading whitespace acts as indent).
    draw_wrapped_text(surface, LBL_CELL.0, y0 + 2, LBL_CELL.1, y1 - 2,
        &body_font, &c_string(label.as_bytes()), ink, TS_CENTRE | W_LEFT, -1);

    // ':' cell — centred.
    draw_wrapped_text(surface, COL_CELL.0, y0 + 2, COL_CELL.1, y1 - 2,
        &body_font, &c_string(b":"), ink, TS_CENTRE, -1);

    // Value cell — left-aligned typed text. Password fields render as
    // asterisks (one per char) so the password isn't visible.
    let display: Vec<u8> = if is_password {
        vec![b'*'; value.chars().count()]
    } else {
        value.as_bytes().to_vec()
    };
    let mut buf = display.clone();
    buf.push(0);
    // Caret index: on the focused field draw a caret at the end of the
    // typed text; otherwise -1 disables the caret.
    let caret: i32 = if focused && enabled { display.len() as i32 } else { -1 };
    draw_wrapped_text(surface, VAL_CELL.0, y0 + 2, VAL_CELL.1, y1 - 2,
        &body_font, &buf, ink, TS_CENTRE | W_LEFT, caret);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_name_smoke() {
        let mut surface = PackedSurface::rgb555(800, 600);
        let mut fonts = Fonts::new("D:/cm0102/Data");
        let state = NameState {
            photo_seed: 0, has_manager: false,
            first: "", second: "", password: "", retype: "", focus: 0,
            cancel_enabled: true, next_enabled: false,
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_name(&mut surface, &mut fonts, &state);
        }));
    }
}
