//! DirectDraw primitives — direct ports of cm0102.exe's immediate-mode
//! rectangle-fill, line, and font-metric functions. Every function here
//! is the Rust equivalent of the exe function of the same name (per
//! `reports/gui_primitives_decode.md`).
//!
//! # Coordinate model
//!
//! All rectangles are LTRB in the exe's 800×600 pitch grid. Colors are
//! RGB565 u16 values (see [`crate::pack565`] / [`crate::rgb_to_surface_pixel`]).
//!
//! # Fill-flag semantics — `FUN_005CD840(l, t, r, b, flag, color)`
//!
//! * `flag & 1` — draw 4 outlines only (border), each line via
//!   `FUN_005CD420(x0, y0, x1, y1, 1, color)`.
//! * `flag & 2` — same outlines with `FUN_005CD420(..., 2, color)`
//!   (thicker or dashed).
//! * else — solid fill via `IDirectDrawSurface::Blt(dest, clip_rect, 0,
//!   0, DDBLT_COLORFILL|DDBLT_WAIT, &{fill_color, alpha=100})`.
//!   `DDBLT_COLORFILL|DDBLT_WAIT = 0x01000400`.
//!
//! # Font-height table — `FUN_005CF7B0(font_id)`
//!
//! Two paths:
//! * `DAT_009B88F8 == 0` (windowed / normal mode): reads the height from
//!   the font-record table at `DAT_00ACCB9C + font_id * 0x1404` (first
//!   field of each 0x1404-byte record).
//! * `DAT_009B88F8 != 0` (fullscreen / larger fonts): hardcoded switch:
//!   * font 0/1 → 15
//!   * font 2 → 18
//!   * font 3 → 21
//!   * font 4 → 24
//!   * font 5 → 27
//!   * font 6 → 39
//!   * font 7 → 45

/// Fill-rect flag bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillFlag {
    /// `flag & 1` — draw four 1-pixel outlines only.
    Outline = 1,
    /// `flag & 2` — draw four thicker outlines.
    ThickOutline = 2,
    /// `flag == 0` — solid fill via DDBLT_COLORFILL.
    SolidFill = 0,
}

impl FillFlag {
    /// exe: `if (flag & 1)` → outline; `else if (flag & 2)` → thick.
    pub fn from_exe_byte(b: u8) -> Self {
        if b & 1 != 0 { Self::Outline }
        else if b & 2 != 0 { Self::ThickOutline }
        else { Self::SolidFill }
    }
}

/// DDBLT flag literal — `DDBLT_COLORFILL | DDBLT_WAIT` = `0x01000400`.
pub const DDBLT_COLORFILL_WAIT: u32 = 0x01000400;

/// Direct port of `FUN_005CF7B0(font_id)` — return font height in pixels.
///
/// Modes:
/// * `windowed_or_normal` = the exe's `DAT_009B88F8 == 0` gate. When
///   `true`, uses the runtime font-record table (caller supplies a
///   closure); when `false`, uses the fullscreen hardcoded switch.
///
/// The fullscreen path is the safe default when the font-record table
/// hasn't been loaded yet.
// GDI-REG: 005cf7b0 PORTED_EXACT
pub fn font_height(font_id: i16, fullscreen_mode: bool,
                   font_record_height: Option<u32>) -> u32 {
    if !fullscreen_mode {
        // Read from the runtime font-record table at
        // `DAT_00ACCB9C + font_id * 0x1404`.
        if font_id == -1 { return 0; }
        return font_record_height.unwrap_or(0);
    }
    // Fullscreen: hardcoded heights.
    match font_id {
        0 | 1 => 0x0F,
        2     => 0x12,
        3     => 0x15,
        4     => 0x18,
        5     => 0x1B,
        6     => 0x27,
        7     => 0x2D,
        _     => 0,
    }
}

/// Font-record stride (exe: `0x1404` bytes per record at `DAT_00ACCB9C`).
pub const FONT_RECORD_STRIDE: u32 = 0x1404;

/// Base VA of the font-record table in cm0102.exe (used only to
/// document the layout; the Rust port stores font data via
/// [`crate::font`] and doesn't read this VA at runtime).
pub const FONT_RECORD_TABLE_VA: u32 = 0x00ACCB9C;

/// Direct port of `FUN_005CD840(l, t, r, b, flag, color)` — fill-rect
/// dispatcher. Returns the equivalent action for the host renderer.
///
/// The exe gates on `DAT_00AD6BFC == 0` (DD init done) and
/// `DAT_00AD6BD4 != 0` (back surface exists). Both are captured as
/// input args here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRectAction {
    /// Skip — DD not up or back surface missing.
    Skip,
    /// Draw four outline lines (each via `FUN_005CD420(l, t, ...)`).
    Outlines { thickness: u8, color: u16 },
    /// Blt colour-fill onto the back surface, rect extended +1 by clip
    /// helper. `flags` matches `DDBLT_COLORFILL_WAIT`.
    ColorFill { rect: (i32, i32, i32, i32), color: u16, flags: u32 },
}

// GDI-REG: 005cd840 PORTED_EXACT
pub fn fill_rect(
    left: i32, top: i32, right: i32, bottom: i32,
    flag: u8, color: u16,
    dd_init_done: bool, back_surface_present: bool,
) -> FillRectAction {
    if !dd_init_done { return FillRectAction::Skip; }
    let ff = FillFlag::from_exe_byte(flag);
    match ff {
        FillFlag::Outline => FillRectAction::Outlines { thickness: 1, color },
        FillFlag::ThickOutline => FillRectAction::Outlines { thickness: 2, color },
        FillFlag::SolidFill => {
            if !back_surface_present { return FillRectAction::Skip; }
            // exe: local_6c += 1; local_68 += 1 — rect extended by 1 pixel
            // on right/bottom by the clip helper before the Blt.
            FillRectAction::ColorFill {
                rect: (left, top, right + 1, bottom + 1),
                color,
                flags: DDBLT_COLORFILL_WAIT,
            }
        }
    }
}

/// Direct port of `FUN_005CD370(l, t, r, b, out[4])` — clip helper.
///
/// Sorts LTRB so `l ≤ r` and `t ≤ b`, clamps to the back surface
/// `[0, back_w-1] × [0, back_h-1]`. Returns `Some((l, t, r, b))` if any
/// part of the input rect lies inside the surface, `None` if fully
/// off-screen (or DD-init-in-progress).
///
/// exe: `DAT_00AD6BF8` = back surface width, `DAT_00AD6BC0` = height.
pub fn clip_to_back_surface(
    left: i32, top: i32, right: i32, bottom: i32,
    back_w: i32, back_h: i32,
    dd_init_done: bool,
) -> Option<(i32, i32, i32, i32)> {
    if !dd_init_done { return None; }
    // Sort — exe: `if (right <= left) l = right`.
    let (mut l, mut r) = if right <= left { (right, left) } else { (left, right) };
    let (mut t, mut b) = if bottom <= top { (bottom, top) } else { (top, bottom) };
    // Clamp lower bounds to zero — exe: `if (-1 >= l) l = 0`.
    if l < 0 { l = 0; }
    if t < 0 { t = 0; }
    // Clamp upper bounds — exe: `if (back_w-1 < r) r = back_w-1`.
    let bw1 = back_w - 1;
    let bh1 = back_h - 1;
    if r > bw1 { r = bw1; }
    if b > bh1 { b = bh1; }
    // Success iff all four bounds are within surface after sort+clamp.
    if r < 0 || b < 0 || l > bw1 || t > bh1 { return None; }
    Some((l, t, r, b))
}

// ============================================================================
// Text measurement — port of FUN_005CF610.
// ============================================================================

/// Font-glyph record stride (exe: 0x14 = 20 bytes per glyph).
pub const GLYPH_STRIDE: u32 = 0x14;

/// Base VA of the glyph metric table (exe: `DAT_00ACCBA0 + font_id*0x1404 + char*0x14`).
pub const GLYPH_TABLE_VA: u32 = 0x00ACCBA0;

/// Fullscreen-mode point size for each of the 8 fonts (exe: switch in
/// `FUN_005CF610`'s fullscreen path — dispatches to `FUN_0059BED0(0, size, s)`).
pub const FONT_POINT_SIZE_FULLSCREEN: [u32; 8] = [9, 9, 10, 14, 16, 18, 20, 22];

/// Direct port of `FUN_005CF610(font_id, str)` — measure string in pixels.
///
/// Two paths mirror the exe. Caller supplies:
/// * `windowed_mode`: `DAT_009B88F8 == 0` — use per-glyph metric table.
/// * `glyph_advance`: closure returning the width (in pixels) that a
///   single character advances at `font_id`. Ports the exe's read of
///   `DAT_00ACCBA0 + char*0x14 + font_id*0x1404`.
/// * `kerning_delta`: closure returning `FUN_005CF840(font, str, idx)` —
///   the exe's inter-glyph kerning adjustment.
/// * `fullscreen_measure`: closure returning `FUN_0059BED0(0, point_size, str)`
///   result — the GDI TextOut-style measure used only in fullscreen mode.
// GDI-REG: 005cf7b0 PORTED_EXACT
pub fn measure_string(
    font_id: i16,
    s: &[u8],
    windowed_mode: bool,
    mut glyph_advance: impl FnMut(i16, u8) -> i32,
    mut kerning_delta: impl FnMut(i16, &[u8], usize) -> i32,
    fullscreen_measure: impl FnOnce(u32, &[u8]) -> i32,
) -> i32 {
    if windowed_mode {
        if font_id == -1 { return 0; }
        let mut total = 0i32;
        for (i, ch) in s.iter().enumerate() {
            if *ch == 0 { return total; }
            // exe: '|' (0x7C) maps to space (0x20).
            let effective = if *ch == 0x7C { 0x20u8 } else { *ch };
            // exe: `if (0x1f < c || c == '|') { ... }`.
            if effective >= 0x20 || *ch == 0x7C {
                total += glyph_advance(font_id, effective)
                       + kerning_delta(font_id, s, i + 1);
            }
        }
        total
    } else {
        // Fullscreen: dispatch to GDI-style measure with per-font point size.
        let point_size = FONT_POINT_SIZE_FULLSCREEN
            .get(font_id as usize).copied().unwrap_or(0);
        if point_size == 0 { return 0; }
        fullscreen_measure(point_size, s)
    }
}

// ============================================================================
// Backbuffer save / restore — ports of FUN_005CDAC0 / FUN_005CDCC0.
// Used by MessageBox to preserve the pixels under a popup.
// ============================================================================

/// Saved backbuffer region descriptor — the record `FUN_005CDAC0` returns
/// (heap-allocated struct with pixel data at `+0xC`).
#[derive(Debug, Clone, PartialEq)]
pub struct SavedRegion {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    /// Copied pixel data (row-major, stride = width, RGB565 u16).
    pub pixels: Vec<u16>,
}

impl SavedRegion {
    pub fn width(&self) -> usize { (self.right - self.left + 1) as usize }
    pub fn height(&self) -> usize { (self.bottom - self.top + 1) as usize }
}

// ============================================================================
// Kerning helper — port of FUN_005CF840 (152 bytes).
// ============================================================================

/// Font-record offset of the "base spacing" dword (exe: `DAT_00ACCE20`
/// vs `DAT_00ACCB9C` base). 0x284 into each 0x1404 font record.
pub const FONT_BASE_SPACING_OFFSET: u32 = 0x284;

/// Glyph record offset of the "right kerning" dword (exe: `DAT_00ACCBA8`
/// = `DAT_00ACCBA0 + 8`).
pub const GLYPH_KERN_RIGHT_OFFSET: u32 = 0x08;

/// Glyph record offset of the "left kerning" dword (exe: `DAT_00ACCBAC`
/// = `DAT_00ACCBA0 + 12`).
pub const GLYPH_KERN_LEFT_OFFSET: u32 = 0x0C;

/// Direct port of `FUN_005CF840(font_id, text_ptr, idx)` — inter-glyph
/// kerning delta between characters `text[idx-1]` and `text[idx]`.
///
/// Exe formula (ported line-by-line):
/// ```pseudo
/// if font_id == -1 || text == null || text[idx] == 0: return 0
/// curr = text[idx]; prev = text[idx-1]
/// if curr == '|': curr = ' '
/// if prev == '|': prev = ' '
/// scaled = font.spacing_base * 3 / 4       // toward-zero divide
/// delta = scaled - glyph[curr].kern_right - glyph[prev].kern_left
/// return max(0, delta)
/// ```
// GDI-REG: 005cf7b0 PORTED_EXACT
pub fn kerning_delta(
    font_id: i16,
    prev_char: Option<u8>,
    curr_char: u8,
    font_spacing_base: i32,
    glyph_kern_right: impl FnOnce(u8) -> i32,
    glyph_kern_left: impl FnOnce(u8) -> i32,
) -> i32 {
    if font_id == -1 { return 0; }
    if curr_char == 0 { return 0; }
    let Some(mut prev) = prev_char else { return 0; };
    let mut curr = curr_char;
    if curr == 0x7C { curr = 0x20; }
    if prev == 0x7C { prev = 0x20; }
    let scaled = font_spacing_base.wrapping_mul(3) / 4;
    let delta = scaled - glyph_kern_right(curr) - glyph_kern_left(prev);
    delta.max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_flag_encoding_matches_exe() {
        assert_eq!(FillFlag::from_exe_byte(1), FillFlag::Outline);
        assert_eq!(FillFlag::from_exe_byte(2), FillFlag::ThickOutline);
        assert_eq!(FillFlag::from_exe_byte(0), FillFlag::SolidFill);
        // Exe checks flag&1 first, so 3 → Outline (bit 0 wins).
        assert_eq!(FillFlag::from_exe_byte(3), FillFlag::Outline);
    }

    #[test]
    fn font_height_fullscreen_matches_exe_switch() {
        for (id, want) in [(0, 15u32), (1, 15), (2, 18), (3, 21),
                            (4, 24), (5, 27), (6, 39), (7, 45)] {
            assert_eq!(font_height(id, true, None), want);
        }
        assert_eq!(font_height(99, true, None), 0);
    }

    #[test]
    fn font_height_windowed_reads_from_table_closure() {
        assert_eq!(font_height(3, false, Some(42)), 42);
        assert_eq!(font_height(-1, false, Some(42)), 0);
    }

    #[test]
    fn font_record_layout_matches_exe() {
        // exe: DAT_00ACCB9C + id * 0x1404. Verifying the stride.
        assert_eq!(FONT_RECORD_STRIDE, 5124);
        assert_eq!(FONT_RECORD_TABLE_VA, 0x00ACCB9C);
    }

    #[test]
    fn fill_rect_skip_when_dd_not_up() {
        assert_eq!(fill_rect(0, 0, 100, 100, 0, 0xFFFF, false, true),
                   FillRectAction::Skip);
    }

    #[test]
    fn fill_rect_skip_when_flag_zero_but_no_back_surface() {
        assert_eq!(fill_rect(0, 0, 100, 100, 0, 0xFFFF, true, false),
                   FillRectAction::Skip);
    }

    #[test]
    fn fill_rect_outline_1px_when_flag_bit_0() {
        assert_eq!(fill_rect(0, 0, 100, 100, 1, 0xF800, true, true),
                   FillRectAction::Outlines { thickness: 1, color: 0xF800 });
    }

    #[test]
    fn fill_rect_thick_outline_when_flag_bit_1() {
        assert_eq!(fill_rect(0, 0, 100, 100, 2, 0x07E0, true, true),
                   FillRectAction::Outlines { thickness: 2, color: 0x07E0 });
    }

    #[test]
    fn fill_rect_solid_extends_rect_by_one_pixel() {
        // Exe's local_6c += 1; local_68 += 1 before Blt.
        assert_eq!(fill_rect(10, 20, 30, 40, 0, 0x1F, true, true),
                   FillRectAction::ColorFill {
                       rect: (10, 20, 31, 41),
                       color: 0x1F,
                       flags: DDBLT_COLORFILL_WAIT,
                   });
    }

    #[test]
    fn ddblt_colorfill_wait_matches_exe_literal() {
        // DDBLT_COLORFILL(0x400) | DDBLT_WAIT(0x01000000) = 0x01000400.
        assert_eq!(DDBLT_COLORFILL_WAIT, 0x0100_0400);
    }

    #[test]
    fn rgb_to_surface_pixel_rgb565_matches_pack565() {
        // The unified rgb_to_surface_pixel(mask=0x7E0) must match the
        // existing pack565 exactly on the same inputs.
        for (r, g, b) in [(0, 0, 0), (255, 255, 255), (198, 0, 0),
                           (128, 128, 128), (63, 127, 191)] {
            assert_eq!(crate::rgb_to_surface_pixel(r, g, b, 0x7E0),
                       crate::pack565(r, g, b),
                       "mismatch for RGB565 ({r}, {g}, {b})");
        }
    }

    #[test]
    fn kerning_returns_zero_for_invalid_font() {
        assert_eq!(kerning_delta(-1, Some(b'a'), b'b', 100, |_| 5, |_| 5), 0);
    }

    #[test]
    fn kerning_returns_zero_when_no_previous_char() {
        assert_eq!(kerning_delta(0, None, b'a', 100, |_| 5, |_| 5), 0);
    }

    #[test]
    fn kerning_pipe_maps_to_space() {
        // With curr='|' and prev='|' both mapped to space, kern lookups
        // will be called with 0x20 not 0x7C.
        let mut curr_arg = 0u8;
        let mut prev_arg = 0u8;
        kerning_delta(0, Some(b'|'), b'|', 100,
                      |c| { curr_arg = c; 0 },
                      |p| { prev_arg = p; 0 });
        assert_eq!(curr_arg, 0x20);
        assert_eq!(prev_arg, 0x20);
    }

    #[test]
    fn kerning_formula_is_base_x_3_div_4_minus_kerns_clamped() {
        // base=100, kern_right=10, kern_left=5 → 100*3/4 - 10 - 5 = 75 - 15 = 60
        assert_eq!(kerning_delta(0, Some(b'a'), b'b', 100, |_| 10, |_| 5), 60);
        // Clamp at zero.
        assert_eq!(kerning_delta(0, Some(b'a'), b'b', 20, |_| 100, |_| 100), 0);
    }

    #[test]
    fn kerning_offsets_match_exe_addresses() {
        // The font record's spacing base is at +0x284 into the 0x1404 record.
        // Glyph record's right/left kerns at +0x08 / +0x0C into each 20-byte glyph.
        assert_eq!(FONT_BASE_SPACING_OFFSET, 0x284);
        assert_eq!(GLYPH_KERN_RIGHT_OFFSET, 0x08);
        assert_eq!(GLYPH_KERN_LEFT_OFFSET, 0x0C);
    }

    #[test]
    fn rgb_to_surface_pixel_rgb555_uses_5_5_5_layout() {
        // In 555 mode green is 5 bits (mask 0x03E0), not 6 (0x07E0).
        let px = crate::rgb_to_surface_pixel(0xFF, 0xFF, 0xFF, 0x03E0);
        // 555: 0111 1111 1111 1111 = 0x7FFF.
        assert_eq!(px, 0x7FFF);
        // Pure red at 555: R=0xF8 → 0x7C00.
        assert_eq!(crate::rgb_to_surface_pixel(0xFF, 0, 0, 0x03E0), 0x7C00);
    }
}
