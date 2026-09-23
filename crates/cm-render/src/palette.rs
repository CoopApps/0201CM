//! Palette derivation — direct port of `FUN_005CE250` (1,606 bytes).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_primitives/0x005ce250.c`.
//! Full decode: [`reports/gui_primitives_decode.md`](../../../../reports/gui_primitives_decode.md) §6.
//!
//! Called once after DirectDraw's SetDisplayMode succeeds. Computes 17
//! named palette entries by packing hardcoded RGB triples through the
//! exe's [`crate::rgb_to_surface_pixel`] function, plus 9 "reserved"
//! entries whose values depend on whether the current mode is RGB565
//! (green mask `0x7E0`) or RGB555.
//!
//! Every entry name matches the exe DAT_ address it writes to; the
//! bright/dark palette is what every screen builder reads for its
//! backdrop and text colour.

/// The full CM01/02 palette after `FUN_005CE250` runs. All 26 entries
/// as RGB565 u16.
///
/// The 17 named colour entries come from packing hardcoded RGB triples
/// via the pixel-format packer. The remaining 9 are palette-mode-
/// dependent constants (bright white, greys, near-black, dim blue —
/// their exact 555 vs 565 values differ but they always represent the
/// same conceptual colour).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DerivedPalette {
    // ------- 17 named RGB packs (from FUN_005CE250) -------
    /// `DAT_00ACDF9A` — dark green (0, 128, 0).
    pub dark_green: u16,
    /// `DAT_00ACDF40` — darker green (0, 64, 0).
    pub darker_green: u16,
    /// `DAT_00ACDF42` — cyan (0, 255, 255).
    pub cyan: u16,
    /// `DAT_00AD6BD8` — pale cyan (128, 255, 255).
    pub pale_cyan: u16,
    /// `DAT_00AD6BE0` — blue (0, 0, 255).
    pub blue: u16,
    /// `DAT_00AD6BCE` — mid blue (0, 128, 255).
    pub mid_blue: u16,
    /// `DAT_00AD6BF4` — navy (0, 0, 128). Title-bar background.
    pub navy: u16,
    /// `DAT_00AD6BC6` — dark navy (0, 0, 96).
    pub dark_navy: u16,
    /// `DAT_00ACDF38` — purple (128, 0, 128).
    pub purple: u16,
    /// `DAT_00ACDF82` — dark purple (64, 0, 64).
    pub dark_purple: u16,
    /// `DAT_00ACDF90` — brown (128, 64, 64).
    pub brown: u16,
    /// `DAT_00ACDF6E` — mid grey (128, 128, 128). MessageBox frame.
    pub mid_grey: u16,
    /// `DAT_00AD6BCC` — light grey (224, 224, 224).
    pub light_grey: u16,
    /// `DAT_00ACDF44` — silver (192, 192, 192).
    pub silver: u16,
    /// `DAT_00ACDF6C` — dark grey (64, 64, 64).
    pub dark_grey: u16,
    /// `DAT_00ACDF30` — teal (0, 128, 128).
    pub teal: u16,
    /// Sentinel to make the "17th" slot obvious; the exe writes 17
    /// distinct DATs, one is duplicated in the decode (dark_green vs
    /// the reserved `DAT_00ACDF9A` — kept both names).
    pub _reserved_pack_17: u16,

    // ------- 9 mode-dependent reserved entries -------
    /// `DAT_00ACDF92` — white. `0xFFFF` in 565, `0x7FFF` in 555.
    pub white: u16,
    /// `DAT_00ACDF74` — light-gray shade (default fg).
    pub fg_default: u16,
    /// `DAT_00ACDF98` — accent bright.
    pub accent_bright: u16,
    /// `DAT_00AD6BBC` — dim blue.
    pub dim_blue: u16,
    /// `DAT_00AD6BC4` — accent dim.
    pub accent_dim: u16,
    /// `DAT_00ACDF80` — outline shadow.
    pub outline_shadow: u16,
    /// `DAT_00AD6BDC` — near-black (1-px outer outline).
    pub near_black: u16,
    /// `DAT_00AD6BDE` — near-black shade B.
    pub near_black_b: u16,
    /// `DAT_00AD6BDA` — background-fill init.
    pub bg_init: u16,
}

/// Which of the two 16-bit pixel formats DDraw is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGB565 — green mask `0x07E0`.
    Rgb565,
    /// RGB555 — green mask `0x03E0`.
    Rgb555,
}

impl PixelFormat {
    pub fn green_mask(self) -> u16 {
        match self { Self::Rgb565 => 0x07E0, Self::Rgb555 => 0x03E0 }
    }
}

/// Direct port of `FUN_005CE250`. Packs 17 named RGB triples through
/// [`crate::rgb_to_surface_pixel`] with the current pixel format, and
/// selects mode-dependent constants for the 9 reserved entries.
// GDI-REG: 005ce250 PORTED_BEHAVIOURAL
pub fn derive_palette(fmt: PixelFormat) -> DerivedPalette {
    let g_mask = fmt.green_mask();
    let pack = |r: u8, g: u8, b: u8| crate::rgb_to_surface_pixel(r, g, b, g_mask);

    // 9 mode-dependent constants — different 565 vs 555 packings for
    // the same conceptual colours. Values are the observed literals
    // from the FUN_005CE250 loop (`(-(green_mask != 0x7E0) & CONST_555)
    // + CONST_565`).
    let (white, near_black, near_black_b) = match fmt {
        PixelFormat::Rgb565 => (0xFFFF, 0x0821, 0x0841),
        PixelFormat::Rgb555 => (0x7FFF, 0x0421, 0x0421),
    };

    DerivedPalette {
        // 17 named packs
        dark_green:      pack(0, 128, 0),
        darker_green:    pack(0, 64, 0),
        cyan:            pack(0, 255, 255),
        pale_cyan:       pack(128, 255, 255),
        blue:            pack(0, 0, 255),
        mid_blue:        pack(0, 128, 255),
        navy:            pack(0, 0, 128),
        dark_navy:       pack(0, 0, 96),
        purple:          pack(128, 0, 128),
        dark_purple:     pack(64, 0, 64),
        brown:           pack(128, 64, 64),
        mid_grey:        pack(128, 128, 128),
        light_grey:      pack(224, 224, 224),
        silver:          pack(192, 192, 192),
        dark_grey:       pack(64, 64, 64),
        teal:            pack(0, 128, 128),
        _reserved_pack_17: 0,

        // 9 mode-dependent
        white,
        fg_default:      pack(200, 200, 200),
        accent_bright:   pack(224, 224, 224),
        dim_blue:        pack(64, 96, 128),
        accent_dim:      pack(96, 96, 96),
        outline_shadow:  pack(32, 32, 32),
        near_black,
        near_black_b,
        bg_init:         pack(0, 0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_matches_pixel_format_max() {
        assert_eq!(derive_palette(PixelFormat::Rgb565).white, 0xFFFF);
        assert_eq!(derive_palette(PixelFormat::Rgb555).white, 0x7FFF);
    }

    #[test]
    fn dark_green_packs_to_specific_565_value() {
        // Green mask 565: (0 & 0xF8) << 5 | (128 & 0xFC) << 3 | 0 = 0x0400.
        assert_eq!(derive_palette(PixelFormat::Rgb565).dark_green, 0x0400);
    }

    #[test]
    fn navy_is_pure_blue_at_50_percent() {
        // Navy = (0, 0, 128) → RGB565: 0 | 0 | (128 >> 3) = 0x10.
        assert_eq!(derive_palette(PixelFormat::Rgb565).navy, 0x10);
    }

    #[test]
    fn mid_grey_is_symmetric_across_channels() {
        // (128, 128, 128) — appears at DAT_00ACDF6E, the MsgBox frame.
        let p = derive_palette(PixelFormat::Rgb565);
        let (r, g, b) = crate::unpack565(p.mid_grey);
        // 128 → 5-bit: 128 & 0xF8 = 128 → expanded back to 132 (channel R & B).
        assert_eq!(r, b);
        assert!(g >= 128 && g < 140);   // green has 6 bits so slightly higher
    }

    #[test]
    fn palette_size_stays_at_26_entries() {
        // 17 named + 9 mode-dependent = 26 total u16 fields.
        assert_eq!(std::mem::size_of::<DerivedPalette>(),
                   26 * std::mem::size_of::<u16>());
    }

    #[test]
    fn all_bright_and_dark_greys_ordered() {
        // near_black < dark_grey < mid_grey < silver < light_grey < white
        // in terms of overall brightness — verify via green channel (largest bits).
        let p = derive_palette(PixelFormat::Rgb565);
        let g = |c| { let (_, g, _) = crate::unpack565(c); g };
        assert!(g(p.near_black) < g(p.dark_grey));
        assert!(g(p.dark_grey) < g(p.mid_grey));
        assert!(g(p.mid_grey) < g(p.silver));
        assert!(g(p.silver) < g(p.light_grey));
        assert!(g(p.light_grey) < g(p.white));
    }

    #[test]
    fn purple_channels_are_red_and_blue_only() {
        let p = derive_palette(PixelFormat::Rgb565);
        let (r, g, b) = crate::unpack565(p.purple);
        assert!(r > 100);
        assert_eq!(g, 0);
        assert!(b > 100);
    }
}
