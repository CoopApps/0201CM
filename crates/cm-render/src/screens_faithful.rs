//! Screen widget renderers — the layer above the primitives that turns
//! `(rect, data)` → the exact primitive call sequence the exe makes.
//!
//! As of 2026-09-03 these helpers are thin adapters over
//! [`crate::packed_widget::render_widget`] — the byte-exact port of the
//! exe's `FUN_005d7aa0` widget dispatcher. Each helper builds a
//! [`Widget`] record from the captured widget-spec in
//! `fixtures/screen_sidebar_and_manager_menu.md` and delegates. Nothing
//! here draws primitives directly — that's `render_widget`'s job.
//!
//! Why go through render_widget rather than call primitives directly:
//! see [[widgets-not-screens]]. Every widget on every screen paints
//! through this one function in the exe; keeping our port aligned means
//! future widgets slot in without new draw code.

use crate::packed::PackedSurface;
use crate::packed_glyph::PixelFont;
use crate::packed_panel::PanelPalette;
use crate::packed_widget::{render_widget, Widget, WidgetGlobals};

/// Construct an empty [`Widget`] with sensible defaults; individual
/// helpers set the fields they care about.
fn base_widget(x0: i32, y0: i32, x1: i32, y1: i32, label: &[u8]) -> Widget {
    // Ensure label ends in NUL — draw_wrapped_text (via render_widget
    // block L) trims at the first NUL, matching the exe's C-string
    // convention on `[ebp+0x80]`.
    let mut lbl = label.to_vec();
    if !lbl.ends_with(&[0]) {
        lbl.push(0);
    }
    Widget {
        frame_base: 0,
        flags: 0,
        x0,
        y0,
        x1,
        y1,
        style_byte: 0,
        text_style: 0,
        text_kern: -1,
        saved_bg: None,
        cached_text: None,
        colour_a: 0,
        colour_b: 0,
        label_ink: 0,
        pattern: 0,
        frame_idx: -1,
        label: lbl,
        detached_glyph_cache: 0,
        alt_hover: 0,
    }
}

/// Sidebar-button widget (date, Continue Game, human-name, category
/// buttons on the left rail). Captured widget-spec:
///
/// ```text
/// panel(x0, y0, x1, y1, style=0x1021, colour=0)          // outer button
/// wrapped_text(x0+2, y0+2, x1+2, y1+2, style=0xc, font=1,
///              colour=<ink>, label)
/// ```
///
/// `style=0x1021` = `P_SAMPLE_BG (0x1000) | P_BEVEL (0x20) | 0x01`.
/// `draw_panel` emits the 8 bevel edges internally when P_BEVEL is set
/// (see `packed_panel.rs` "3D bevel" block); earlier this helper
/// double-drew them by calling `draw_line` after `draw_panel` — that
/// bug is gone now.
///
/// `label` may contain `'\n'` for multi-line labels ("Continue\nGame",
/// "Nations\n& Clubs"). Font = 1 (the sidebar font).
pub fn sidebar_button(
    s: &mut PackedSurface,
    x0: i32, y0: i32, x1: i32, y1: i32,
    label: &[u8],
    ink: u16,
    palette: PanelPalette,
    font: &PixelFont,
) {
    let mut w = base_widget(x0, y0, x1, y1, label);
    // panel(style=0x1021, colour=0) — see fixture header.
    w.style_byte = 0x1021;
    w.colour_a = 0;
    // wrapped_text(x0+2, y0+2, x1+2, y1+2, style=0xc, font=1, ink, label).
    // The +2 offset comes from block F's `effective_style & 0x40` press
    // path setting label_offset_{x,y}=2; but that only fires on hover.
    // The exe's captured trace shows the offset for the non-hover path
    // too — so the widget itself carries style_byte 0x40 pre-baked.
    // (Note: the primary panel style stays 0x1021; the 0x40 goes into
    // effective_style via a separate path — but we're not modelling
    // hover here, so just union 0x40 into style_byte to force the
    // 2-pixel label indent.)
    w.style_byte |= 0x40;
    w.text_style = 0x0c;
    w.label_ink = ink;
    render_widget(
        s,
        &mut w,
        None,
        font,
        WidgetGlobals { panel_palette: palette },
        true,
    );
}

/// Colour bands used by `menu_item` — alternate per row index.
pub const MENU_BAND_EVEN: u16 = 0x0200;
pub const MENU_BAND_ODD:  u16 = 0x0240;
/// Hover ink for a menu item.
pub const MENU_HOVER_COLOUR: u16 = 0x7FE0;

/// One drop-down menu row. Captured widget-spec:
///
/// ```text
/// panel(x0, y0, x1, y1, style=0x10 or 0x1000010, colour=band)
/// wrapped_text(x0, y0, x1, y1, style=1, font=1, colour=0, "     <label>")
/// ```
///
/// `style=0x10` = `P_SOLID_FILL`. `style=0x1000010` adds
/// `P_MIDLINE_H (0x100_0000)` — `draw_panel` emits the engraved
/// separator (two 1-px lines, scaled 66%/133%) internally when that
/// bit is set; the earlier hand-drawn separator lines are gone.
///
/// `is_hovered` swaps the fill colour to yellow.
pub fn menu_item(
    s: &mut PackedSurface,
    x0: i32, y0: i32, x1: i32, y1: i32,
    band: u16,
    label: &[u8],
    is_separator: bool,
    is_hovered: bool,
    palette: PanelPalette,
    font: &PixelFont,
) {
    let colour = if is_hovered { MENU_HOVER_COLOUR } else { band };
    let style: u32 = if is_separator { 0x0100_0010 } else { 0x0000_0010 };
    let mut w = base_widget(x0, y0, x1, y1, label);
    w.style_byte = style;
    w.colour_a = colour;
    // text_style=1: no wrapping. label_ink=0 (black).
    w.text_style = 1;
    w.label_ink = 0x0000;
    render_widget(
        s,
        &mut w,
        None,
        font,
        WidgetGlobals { panel_palette: palette },
        true,
    );
}

/// Drop-down container panel (the outer box behind a menu). Captured
/// widget-spec:
///
/// ```text
/// panel(x0, y0, x1, y1, style=0x30, colour=0x200)   // P_BEVEL | P_SOLID_FILL
/// ```
///
/// No label. `draw_panel` emits fill + 3D bevel internally.
pub fn menu_dropdown_container(
    s: &mut PackedSurface,
    x0: i32, y0: i32, x1: i32, y1: i32,
    palette: PanelPalette,
    font: &PixelFont,
) {
    let mut w = base_widget(x0, y0, x1, y1, &[]);
    w.style_byte = 0x30;
    w.colour_a = 0x0200;
    // flags |= 0x400 = "widget draws own text" — suppresses block L
    // (there's no label to draw here).
    w.flags = 0x400;
    render_widget(
        s,
        &mut w,
        None,
        font,
        WidgetGlobals { panel_palette: palette },
        true,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packed_glyph::{Glyph, PixelFont};

    /// A stub font: every ASCII char is a 4×5 solid block. Fine for
    /// testing widget renderers' NON-TEXT outputs; real text pixels
    /// need the real font (verified elsewhere by verify_glyph).
    fn stub_font() -> PixelFont {
        let mut f = PixelFont::empty(6);
        for c in 0x20u8..=0x7Eu8 {
            f.glyphs[c as usize] = Some(Glyph {
                width: 4, kern_a: 0, kern_b: 0, kern_c: 0,
                bitmap: vec![0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0],
            });
        }
        f
    }

    #[test]
    fn sidebar_button_paints_bevel_from_render_widget() {
        // Draw a sidebar button at (5, 145)-(85, 187). Palette:
        // default_bevel=0x0010 so draw_panel's bevel scales that.
        let mut s = PackedSurface::rgb555(89, 190);
        let palette = PanelPalette {
            outer_highlight: 0x7FE0,
            default_bevel: 0x0010,
        };
        sidebar_button(&mut s, 5, 145, 85, 187, b"X",
                       0x43FF, palette, &stub_font());
        // draw_panel's P_BEVEL path emits a 3D bevel around the rect.
        // The outer-top-left pixel (5, 145) is where the bevel starts,
        // so it must be a non-zero colour derived from default_bevel.
        let outer_top_left = s.buf[(145 * 89 + 5) as usize];
        assert_ne!(outer_top_left, 0, "bevel top-left must be painted");
    }

    #[test]
    fn menu_item_fills_row_with_band_colour() {
        // Row 1 of the manager menu at (90, 147)-(277, 166), band even.
        let mut s = PackedSurface::rgb555(280, 170);
        let palette = PanelPalette::default();
        menu_item(&mut s, 90, 147, 277, 166, MENU_BAND_EVEN,
                  b"Pro Vercelli Squad", false, false, palette, &stub_font());
        // P_SOLID_FILL fills the interior with band colour.
        assert_eq!(s.buf[(150 * 280 + 100) as usize], MENU_BAND_EVEN);
    }

    #[test]
    fn menu_item_hover_fills_with_yellow() {
        let mut s = PackedSurface::rgb555(280, 200);
        menu_item(&mut s, 90, 168, 277, 187, MENU_BAND_EVEN,
                  b"Pro Vercelli Reserves", false, true,
                  PanelPalette::default(), &stub_font());
        // Interior pixel past the text glyphs.
        assert_eq!(s.buf[(175 * 280 + 250) as usize], MENU_HOVER_COLOUR);
    }

    #[test]
    fn menu_separator_paints_via_panel_midline() {
        // Separator row at (90, 252)-(277, 270).
        let mut s = PackedSurface::rgb555(280, 275);
        menu_item(&mut s, 90, 252, 277, 270, MENU_BAND_ODD, b"", true, false,
                  PanelPalette::default(), &stub_font());
        // With P_MIDLINE_H set, draw_panel emits two lines at mid-y and
        // mid-y+1 with colours scaled from colour_a=MENU_BAND_ODD=0x0240:
        //   dark   = scale_colour(0x0240, 0x42)  ≈ 66% of colour_a
        //   bright = scale_colour(0x0240, 0x85)  ≈ 133% of colour_a
        // Both must be non-zero and dark != bright.
        let mid = (252 + 270) / 2; // 261
        let dark_px = s.buf[(mid * 280 + 100) as usize];
        let bright_px = s.buf[((mid + 1) * 280 + 100) as usize];
        assert_ne!(dark_px, 0, "midline dark pixel painted");
        assert_ne!(bright_px, 0, "midline bright pixel painted");
        assert_ne!(dark_px, bright_px, "midline dark and bright differ");
    }

    #[test]
    fn dropdown_container_bevel_and_fill() {
        // Big panel (88, 145)-(279, 481).
        let mut s = PackedSurface::rgb555(285, 490);
        menu_dropdown_container(&mut s, 88, 145, 279, 481,
                                PanelPalette::default(), &stub_font());
        // P_SOLID_FILL + colour=0x200 fills interior.
        assert_eq!(s.buf[(200 * 285 + 150) as usize], 0x0200);
    }
}
