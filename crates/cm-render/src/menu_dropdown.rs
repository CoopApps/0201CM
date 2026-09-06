//! Shared drop-down / context-menu painter used by every pull-down in
//! the game (View, Sort By, Filter, Nation-filter, ...). Colours +
//! layout convention are the same across every menu the exe paints;
//! the individual screens supply items + hit-test.
//!
//! Colours measured from the live GDI capture
//! `scratchpad/prelaunch/gdi_view_menu3.png` (RGB555):
//!   MENU_GREEN     0x0200   (0, 132, 0)   base row
//!   MENU_GREEN_HI  0x0240   (0, 148, 0)   alternating row
//!   YELLOW_HL      0x7FE0   (255,255,0)   hover-highlight bg
//!   ink            BLACK on every row
//!
//! Selection uses a tick glyph (2 short line strokes forming a check)
//! at the left of the row — NOT a background-colour change. Yellow is
//! purely a HOVER state that follows the cursor.

use crate::packed::PackedSurface;
use crate::packed_glyph::PixelFont;
use crate::packed_panel::{draw_panel, PanelPalette, P_SOLID_FILL, P_BEVEL};
use crate::packed_text::{draw_wrapped_text, W_LEFT};

/// RGB555 bit pattern for the darker base green. Same value as
/// `screen_nationality_faithful::MENU_GREEN`, kept here so callers
/// don't need to reach into a sibling screen module.
pub const MENU_GREEN:    u16 = 0x0200;
/// One shade brighter, used on alternating rows.
pub const MENU_GREEN_HI: u16 = 0x0240;
/// Hover-highlight bg — pure saturated yellow in RGB555.
pub const YELLOW_HL:     u16 = 0x7FE0;

/// Layout of one dropdown.
#[derive(Debug, Clone, Copy)]
pub struct DropdownRect {
    /// Top-left origin (screen-space).
    pub x0: i32,
    pub y0: i32,
    /// Width in pixels. Height is derived from `row_h * items.len()`.
    pub width: i32,
    /// Row height in pixels — matches the exe's ~18 px rows for the
    /// View / Sort By dropdowns and ~22 for the Nation-filter.
    pub row_h: i32,
}

impl DropdownRect {
    pub fn x1(&self) -> i32 { self.x0 + self.width }
    pub fn y1(&self, item_count: usize) -> i32 {
        self.y0 + self.row_h * item_count as i32
    }
    /// Hit-test: returns the row index the cursor is over, or None
    /// when outside the dropdown.
    pub fn hit(&self, item_count: usize, x: i32, y: i32) -> Option<usize> {
        if x < self.x0 || x > self.x1() || y < self.y0 { return None; }
        let row = (y - self.y0) / self.row_h;
        if (row as usize) < item_count { Some(row as usize) } else { None }
    }
}

/// Paint the standard green/yellow/black dropdown.
///
/// * `items` — one label per row.
/// * `selected` — draw a tick on this row (the currently-active choice).
/// * `cursor` — screen-space; whichever row the cursor is over gets the
///   yellow hover background.
///
/// The label is left-padded with whitespace so the tick has room —
/// same "      " indent the exe uses for its own dropdowns.
pub fn draw_dropdown(
    surface: &mut PackedSurface,
    font: &PixelFont,
    rect: DropdownRect,
    items: &[&str],
    selected: Option<usize>,
    cursor: (i32, i32),
) {
    let palette = PanelPalette::default();
    let ink_black = 0u16;
    let (cx, cy) = cursor;
    let dd_y1 = rect.y1(items.len());
    // Outer panel — solid green with a light bevel.
    draw_panel(surface, rect.x0, rect.y0, rect.x1(), dd_y1,
        P_SOLID_FILL | P_BEVEL, MENU_GREEN, ink_black, palette);
    let hover_row = rect.hit(items.len(), cx, cy);
    for (i, label) in items.iter().enumerate() {
        let ry0 = rect.y0 + 1 + i as i32 * rect.row_h;
        let ry1 = ry0 + rect.row_h - 1;
        let fill = if hover_row == Some(i) {
            YELLOW_HL
        } else if i % 2 == 0 {
            MENU_GREEN
        } else {
            MENU_GREEN_HI
        };
        for y in ry0..ry1 {
            for x in rect.x0 + 1..rect.x1() - 1 {
                surface.buf[y as usize * surface.pitch_pixels as usize + x as usize] = fill;
            }
        }
        if label.is_empty() {
            // Separator row — no text, no tick. Paint a 2-pixel
            // embossed horizontal line centred vertically in the row:
            // a darker upper line + a lighter lower line, matching the
            // Windows-style groove the exe draws between the last club
            // and the national team in the club-jump dropdown.
            let cy = (ry0 + ry1) / 2;
            let lx0 = rect.x0 + 4;
            let lx1 = rect.x1() - 4;
            // Upper pixel: same shade as bar shadow (approx 0,66,0 in
            // RGB555 = 0x0100 — one green step darker than MENU_GREEN).
            let shadow_green = 0x0100u16;
            let hi_green     = 0x02A0u16;   // slightly brighter than MENU_GREEN_HI
            surface.draw_line(lx0, cy - 1, lx1, cy - 1, 2, shadow_green);
            surface.draw_line(lx0, cy,     lx1, cy,     2, hi_green);
        } else {
            // Label — BLACK, left-indented past the tick position.
            let mut buf = b"      ".to_vec();
            buf.extend_from_slice(label.as_bytes());
            buf.push(0);
            draw_wrapped_text(surface, rect.x0, ry0, rect.x1() - 4, ry1,
                font, &buf, ink_black, W_LEFT, -1);
            // Tick on the currently-selected row. Two short line segments
            // meeting at a low point; second-pixel-thick stroke for
            // legibility on the greens.
            if selected == Some(i) {
                let tx = rect.x0 + 5;
                let ty = (ry0 + ry1) / 2;
                surface.draw_line(tx,     ty - 1, tx + 2, ty + 2, 2, ink_black);
                surface.draw_line(tx + 2, ty + 2, tx + 6, ty - 3, 2, ink_black);
                surface.draw_line(tx,     ty,     tx + 2, ty + 3, 2, ink_black);
                surface.draw_line(tx + 2, ty + 3, tx + 6, ty - 2, 2, ink_black);
            }
        }
    }
}
