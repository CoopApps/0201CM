//! 0201CM design-system renderer.
//!
//! This is the CM0102 look captured as a modern, extensible component library. Fidelity
//! lives in the *design system* (the exact colour pipeline, panel/bevel math, `.fnt`
//! typography, layout rules — all lifted from cm0102.exe), not in replaying an old
//! framebuffer. The base game is calibrated pixel-exact against the original; new screens
//! are authored from the same primitives and inherit the look for free.
//!
//! The surface carries the original's **RGB565** pipeline so quantised colours match the
//! exe to the bit (e.g. red `(198,0,0)` is `0xC000` packed then unpacked). It presents as
//! RGBA8 for a modern GPU window.

pub mod area;
pub mod background;
pub mod bevel;
pub mod blit;
pub mod cursor;
pub mod drawstring;
pub mod fade;
pub mod font;
pub mod font_loader;
pub mod gen_screen_types;
pub mod gen_screens;
pub mod glyph_blit;
pub mod hsr;
pub mod image;
pub mod inflection;
pub mod l10n;
pub mod lang_bank;
pub mod layout;
pub mod line;
pub mod packed;
pub mod packed_glyph;
pub mod packed_panel;
pub mod packed_capture;
pub mod packed_widget;
pub mod packed_frame_lookup;
pub mod packed_stipples;
pub mod packed_icon_loader;
pub mod packed_widget_globals;
pub mod pool_to_render;
pub mod scrman;
pub mod screen_menu_bar;
pub mod screen_nav_back_next;
pub mod screen_news;
pub mod screen_wire_batch3;
pub mod dispatcher;
pub mod screens_faithful;
pub mod packed_text;
pub mod packed_sprintf;
pub mod msgbox;
pub mod palette;
pub mod panel;
pub mod primitives;
pub mod sidebar_msg;
pub mod view_render;
pub mod widget_pool;
pub mod window_init;

/// Exact port of graphics_rgb_to_surface_pixel (0x005ce4f0), RGB565 path (green mask 0x7e0).
#[inline]
pub fn pack565(r: u8, g: u8, b: u8) -> u16 {
    ((((r as u16 & 0xf8) << 5 | (g as u16 & 0xfc)) << 3) | (b as u16 & 0xff) >> 3) & 0xffff
}

/// Full port of `FUN_005CE4F0` (129 bytes). Selects RGB565 vs RGB555
/// based on the pixel-format descriptor's green mask (exe's `param_4[5]`
/// = green mask; `0x7E0` = 565, else 555).
///
/// When the DD-init flag `DAT_00AD6BFC` is non-zero the exe returns 0 —
/// the port takes that as `None`.
#[inline]
pub fn rgb_to_surface_pixel(r: u8, g: u8, b: u8, green_mask: u16) -> u16 {
    if green_mask == 0x7E0 {
        // RGB565: (r & 0xF8) << 5 | (g & 0xFC) → << 3, or (b & 0xFF) >> 3.
        let r5 = ((r as u16) & 0xF8) << 5;
        let g6 = (g as u16) & 0xFC;
        let b5 = (b as u16) >> 3;
        ((r5 | g6) << 3) | b5
    } else {
        // RGB555: (r & 0xF8) << 5 | (g & 0xF8) → << 2, or (b & 0xFF) >> 3.
        let r5 = ((r as u16) & 0xF8) << 5;
        let g5 = (g as u16) & 0xF8;
        let b5 = (b as u16) >> 3;
        ((r5 | g5) << 2) | b5
    }
}

/// Inverse of [`pack565`]: expand 5/6/5 to 8-bit by bit-replication.
#[inline]
pub fn unpack565(v: u16) -> (u8, u8, u8) {
    let r5 = (v >> 11) & 0x1f;
    let g6 = (v >> 5) & 0x3f;
    let b5 = v & 0x1f;
    (
        ((r5 << 3) | (r5 >> 2)) as u8,
        ((g6 << 2) | (g6 >> 4)) as u8,
        ((b5 << 3) | (b5 >> 2)) as u8,
    )
}

/// An 800x600 16-bit indexed surface — the original's DirectDraw surface, in memory.
pub struct Surface {
    pub w: usize,
    pub h: usize,
    pub buf: Vec<u16>,
}

impl Surface {
    pub const W: usize = 800;
    pub const H: usize = 600;

    pub fn new() -> Self {
        Self { w: Self::W, h: Self::H, buf: vec![0; Self::W * Self::H] }
    }

    #[inline]
    pub fn fill(&mut self, r: u8, g: u8, b: u8) {
        let p = pack565(r, g, b);
        self.buf.iter_mut().for_each(|x| *x = p);
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, packed: u16) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.buf[y as usize * self.w + x as usize] = packed;
        }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> u16 {
        self.buf[y as usize * self.w + x as usize]
    }

    /// Present buffer: convert the RGB565 surface to RGBA8 for a GPU (wgpu/pixels) window.
    pub fn to_rgba(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), self.w * self.h * 4);
        for (i, &v) in self.buf.iter().enumerate() {
            let (r, g, b) = unpack565(v);
            let o = i * 4;
            out[o] = r;
            out[o + 1] = g;
            out[o + 2] = b;
            out[o + 3] = 0xff;
        }
    }

    /// Present buffer for softbuffer: 0x00RRGGBB per pixel.
    pub fn to_argb(&self, out: &mut [u32]) {
        debug_assert_eq!(out.len(), self.w * self.h);
        for (i, &v) in self.buf.iter().enumerate() {
            let (r, g, b) = unpack565(v);
            out[i] = (r as u32) << 16 | (g as u32) << 8 | b as u32;
        }
    }
}

impl Default for Surface {
    fn default() -> Self {
        Self::new()
    }
}
