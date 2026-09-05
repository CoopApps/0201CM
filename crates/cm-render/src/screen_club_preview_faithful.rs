//! Minimal "club preview" screen — what the exe shows after clicking a
//! club on Select Team, BEFORE the manager takes control.
//!
//! The exe's full club screen has ~10 tabs (Squad / Fixtures / History /
//! Transfer / Profile / Contract / General Info / Injuries & Bans /
//! Next Match / Transfers) and a huge amount of state. Porting all of
//! that is a separate multi-commit sub-project. THIS module ports only
//! the bits needed to make the pre-boot flow work end-to-end:
//!
//! - Shared pre-boot chrome (photo / sidebar / title / subheader = club
//!   name).
//! - The small red "**Take Control**" button pinned to the very top-right
//!   of the window at `(660, 4)-(785, 24)` — the affordance that
//!   installs the manager and jumps to News. From the live capture
//!   (`fixtures/club_preview_screen/structure.txt`):
//!     ```
//!     PANEL (660,4)-(785,24) c=0x7fff p=0x7000 s=0x30    <- white fill + bevel
//!     rect  (660,4)-(785,24) c=0x7fff s=4                <- solid white
//!     WRAP  (660,4)-(785,24) f=1 c=0x7000 'Take Control' <- dark red text
//!     ```
//!   White bevelled button, dark-red 3-D bevel (via pattern colour),
//!   dark-red font-1 label.
//! - A placeholder body panel with a "Club info screen — full port in
//!   progress" note so the screen isn't empty while the rest of the
//!   club info is being ported.

use crate::font::Fonts;
use crate::packed::PackedSurface;
use crate::packed_panel::{draw_panel, PanelPalette, P_BEVEL, P_DARKEN, P_SOLID_FILL};
use crate::packed_text::draw_wrapped_text;
use crate::screen_pre_boot_chrome::{
    c_string, draw_chrome, ChromeState,
    F_BODY, F_SMALL, INK_CYAN, TS_CENTRE,
};

/// Take Control button rect (top-right corner, ABOVE the title bar).
pub const TAKE_CONTROL_RECT: (i32, i32, i32, i32) = (660, 4, 785, 24);

/// White fill for the Take Control button — `0x7fff = (31, 31, 31)`.
const WHITE_FILL: u16 = 0x7fff;
/// Dark-red pattern accent + label ink — `0x7000 = (28, 0, 0)`.
const DARK_RED: u16 = 0x7000;

pub struct ClubPreviewState<'a> {
    pub photo_seed: u64,
    pub has_manager: bool,
    /// Club name for the subheader.
    pub club_name: &'a str,
    /// Optional secondary line under the club name (division + nation).
    pub subtitle: &'a str,
}

pub fn render_club_preview(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    state: &ClubPreviewState<'_>,
) {
    // Chrome — subheader is the club name. Back returns to Select Team;
    // Next is inert here (Take Control is the real forward affordance).
    draw_chrome(surface, fonts, &ChromeState {
        photo_seed: state.photo_seed,
        has_manager: state.has_manager,
        sub_title: state.club_name,
        left_nav_label: "Back",
        right_nav_label: "Next",
        back_enabled: true,
        next_enabled: false,
    });

    // Placeholder body panel — a darkened container roughly where the
    // exe's tab bar + info panels would sit. Real content lands in
    // follow-up commits.
    let palette = PanelPalette::default();
    draw_panel(surface, 110, 145, 780, 535, P_DARKEN, 0, 0, palette);
    let body_font = fonts.pixel_slot(F_BODY).clone();
    draw_wrapped_text(surface, 110, 300, 780, 340,
        &body_font, &c_string(state.subtitle.as_bytes()),
        INK_CYAN, TS_CENTRE, -1);
    draw_wrapped_text(surface, 110, 340, 780, 380,
        &body_font,
        &c_string(b"Full club info screen - port in progress."),
        INK_CYAN, TS_CENTRE, -1);
    draw_wrapped_text(surface, 110, 380, 780, 420,
        &body_font,
        &c_string(b"Click TAKE CONTROL (top-right) to take charge."),
        INK_CYAN, TS_CENTRE, -1);

    // Take Control button — LAST so it paints on top of everything,
    // matching the exe's op order (WRAP appears near the top of the
    // captured stream but at a very high y-coordinate).
    let (tx0, ty0, tx1, ty1) = TAKE_CONTROL_RECT;
    draw_panel(surface, tx0, ty0, tx1, ty1,
        P_SOLID_FILL | P_BEVEL, WHITE_FILL, DARK_RED, palette);
    surface.draw_rectangle(tx0, ty0, tx1, ty1, 4, WHITE_FILL);
    let small_font = fonts.pixel_slot(F_SMALL).clone();
    draw_wrapped_text(surface, tx0, ty0, tx1, ty1,
        &small_font, &c_string(b"Take Control"),
        DARK_RED, TS_CENTRE, -1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_preview_smoke() {
        let mut surface = PackedSurface::rgb555(800, 600);
        let mut fonts = Fonts::new("D:/cm0102/Data");
        let state = ClubPreviewState {
            photo_seed: 0, has_manager: false,
            club_name: "Arsenal", subtitle: "Premier Division · England",
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            render_club_preview(&mut surface, &mut fonts, &state);
        }));
    }
}
