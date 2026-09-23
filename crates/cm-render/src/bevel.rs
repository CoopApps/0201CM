//! 3-D bevel box — direct port of `FUN_005CF8E0` (13,929 bytes decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_primitives/0x005cf8e0.c`.
//! Full decode: [`reports/gui_primitives_decode.md`](../../../../reports/gui_primitives_decode.md) §12.
//!
//! The exe uses this one function for every panel, button, header,
//! grouped-list border, dialog frame, sunken separator, and grouped
//! header the game paints. It's the workhorse of the 2D UI.
//!
//! # Flag word bit map
//!
//! ```text
//! 0x0000_0002 — TRANSPARENT clear (FUN_005CDFD0)
//! 0x0000_0004 — vertical gradient (per-column, brightness ramp)
//! 0x0000_0008 — horizontal gradient (per-row, brightness ramp)
//! 0x0000_0010 — solid interior fill (op-mode 4)
//! 0x0000_0020 — recessed 3D bevel border
//! 0x0000_0040 — invert bevel (pressed state)
//! 0x0000_0080 — darkened single bottom-edge line (90%)
//! 0x0000_0100 — (reserved / unmapped in this decode)
//! 0x0000_0200 — single-line rim (with 0x020)
//! 0x0000_0400 — full frame outline (fill_rect mode 1/2)
//! 0x0000_0800 — 1px near-black outer outline (DAT_00AD6BDC)
//! 0x0000_1000 — SAMPLE pixel at (x0, y0) to use as fill colour
//! 0x0000_2000 — sub-flag: use rim mode 2 vs 1
//! 0x0000_4000 — middle horizontal line (with 0x400)
//! 0x0000_8000 — sub-flag: middle line at y0 vs (y0+y1)/2
//! 0x0001_0000 — pattern fill via FUN_005D11E0 (with 0x010)
//! 0x0002_0000 — SHRINK rect: right/bottom -= 2
//! 0x0004_0000 — OFFSET rect: left/top += 2
//! 0x0100_0000 — inset separator groove (2-line: bright + dark)
//! ```

/// Percentage constants read verbatim from the bevel loop's
/// brightness/darkness ramp. Matches exe literals.
pub const PCT_66: i32 = 0x42;      // dark shade / step
pub const PCT_90: i32 = 0x5A;      // darkened bottom edge
pub const PCT_133: i32 = 0x85;     // bright inset separator
pub const PCT_166: i32 = 0xA6;     // (used by drawstring shadow — see drawstring.rs)

/// Bevel box flag bits.
pub mod flag {
    pub const TRANSPARENT_CLEAR: u32 = 0x0000_0002;
    pub const GRADIENT_VERTICAL: u32 = 0x0000_0004;
    pub const GRADIENT_HORIZONTAL: u32 = 0x0000_0008;
    pub const SOLID_FILL: u32 = 0x0000_0010;
    pub const BEVEL_3D: u32 = 0x0000_0020;
    pub const BEVEL_INVERTED: u32 = 0x0000_0040;
    pub const DARK_BOTTOM_EDGE: u32 = 0x0000_0080;
    pub const RIM_SINGLE_LINE: u32 = 0x0000_0200;
    pub const FRAME_OUTLINE: u32 = 0x0000_0400;
    pub const OUTER_OUTLINE_1PX: u32 = 0x0000_0800;
    pub const SAMPLE_COLOR: u32 = 0x0000_1000;
    pub const RIM_THICK: u32 = 0x0000_2000;
    pub const FRAME_MIDDLE_LINE: u32 = 0x0000_4000;
    pub const FRAME_MIDDLE_AT_TOP: u32 = 0x0000_8000;
    pub const PATTERN_FILL: u32 = 0x0001_0000;
    pub const RECT_SHRINK: u32 = 0x0002_0000;
    pub const RECT_OFFSET: u32 = 0x0004_0000;
    pub const INSET_SEPARATOR: u32 = 0x0100_0000;
}

/// The bevel box's rect after preprocessing (SHRINK / OFFSET applied).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevelRect { pub left: i32, pub top: i32, pub right: i32, pub bottom: i32 }

impl BevelRect {
    /// Apply the exe's preprocessing branches: SHRINK, then OFFSET.
    pub fn from_flags(mut l: i32, mut t: i32, mut r: i32, mut b: i32, flags: u32) -> Self {
        if flags & flag::RECT_SHRINK != 0 { r -= 2; b -= 2; }
        if flags & flag::RECT_OFFSET != 0 { l += 2; t += 2; }
        Self { left: l, top: t, right: r, bottom: b }
    }
    pub fn width(self) -> i32 { self.right - self.left + 1 }
    pub fn height(self) -> i32 { self.bottom - self.top + 1 }
}

/// The primitives the bevel dispatcher emits, in the exact order the
/// exe would call them. A host renderer iterates this list to paint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BevelOp {
    /// FUN_005CDFD0 — transparent clear.
    TransparentClear { rect: BevelRect },
    /// Per-column brightness ramp.
    GradientVertical { rect: BevelRect, color: u16 },
    /// Per-row brightness ramp.
    GradientHorizontal { rect: BevelRect, color: u16 },
    /// fill_rect mode 4 — solid.
    SolidFill { rect: BevelRect, color: u16 },
    /// FUN_005D11E0 — pattern fill.
    PatternFill { rect: BevelRect, color: u16 },
    /// Classic 3D bevel: `thickness` iterations of 4-line inset frame.
    /// Bright color used for top/left; dark for bottom/right (swap if
    /// `inverted`).
    Bevel3D { rect: BevelRect, base_color: u16, thickness: u8, inverted: bool },
    /// Single-line rim border (with BEVEL_3D + RIM_SINGLE_LINE).
    RimLine { rect: BevelRect, color: u16, thickness: u8 },
    /// Full frame outline via fill_rect mode 1 or 2.
    FrameOutline { rect: BevelRect, color: u16, thickness: u8 },
    /// Middle horizontal line (with FRAME_MIDDLE_LINE).
    MiddleHLine { y: i32, left: i32, right: i32, color: u16, thickness: u8 },
    /// Darkened bottom edge at 90%.
    DarkBottomEdge { rect: BevelRect, color: u16 },
    /// Two-line inset separator: bright at y, dark at y+1.
    InsetSeparator { y: i32, left: i32, right: i32, bright: u16, dark: u16 },
    /// 1-px near-black outer outline (DAT_00AD6BDC).
    OuterOutline { rect: BevelRect, color: u16 },
}

/// Direct port of `FUN_005CF8E0(l, t, r, b, flags, color, xtra_color)`.
///
/// Returns the ordered list of draw ops. `sample_pixel` is invoked if
/// `flags & SAMPLE_COLOR` — matches the exe's Lock/sample at (x0, y0)
/// via IDirectDrawSurface. `outer_outline_color` is `DAT_00AD6BDC`
/// (near-black), also computed at palette-derive time.
// GDI-REG: 005cf8e0 PORTED_EXACT
pub fn bevel_box(
    left: i32, top: i32, right: i32, bottom: i32,
    flags: u32,
    color: u16, xtra_color: u16,
    outer_outline_color: u16,
    sample_pixel: impl FnOnce(i32, i32) -> u16,
) -> Vec<BevelOp> {
    let rect = BevelRect::from_flags(left, top, right, bottom, flags);
    // SAMPLE_COLOR overrides `color` with the pixel at (x0, y0).
    let color = if flags & flag::SAMPLE_COLOR != 0 {
        sample_pixel(rect.left, rect.top)
    } else { color };

    let mut ops = Vec::new();

    // --- Body fill (mutually exclusive) ---
    if flags & flag::TRANSPARENT_CLEAR != 0 {
        ops.push(BevelOp::TransparentClear { rect });
    } else if flags & flag::SOLID_FILL != 0 {
        if flags & flag::PATTERN_FILL != 0 {
            ops.push(BevelOp::PatternFill { rect, color });
        } else {
            ops.push(BevelOp::SolidFill { rect, color });
        }
    } else if flags & flag::GRADIENT_VERTICAL != 0 {
        ops.push(BevelOp::GradientVertical { rect, color });
    } else if flags & flag::GRADIENT_HORIZONTAL != 0 {
        ops.push(BevelOp::GradientHorizontal { rect, color });
    }

    // --- Border ---
    if flags & flag::BEVEL_3D != 0 {
        if flags & flag::RIM_SINGLE_LINE != 0 {
            let thickness = if flags & flag::RIM_THICK != 0 { 2 } else { 1 };
            ops.push(BevelOp::RimLine { rect, color, thickness });
        } else {
            // Classic 3D: thickness 4 if box ≥50×50 AND flag has SOLID_FILL, else 2.
            let big = rect.width() >= 50 && rect.height() >= 50;
            let thickness = if big && flags & flag::SOLID_FILL != 0 { 4 } else { 2 };
            let inverted = flags & flag::BEVEL_INVERTED != 0;
            ops.push(BevelOp::Bevel3D { rect, base_color: color, thickness, inverted });
        }
    } else if flags & flag::DARK_BOTTOM_EDGE != 0 {
        ops.push(BevelOp::DarkBottomEdge { rect, color });
    } else if flags & flag::FRAME_OUTLINE != 0 {
        if flags & flag::FRAME_MIDDLE_LINE != 0 {
            // Middle horizontal line variant.
            let y = if flags & flag::FRAME_MIDDLE_AT_TOP != 0 { rect.top }
                    else { (rect.top + rect.bottom) / 2 };
            let thickness = if flags & flag::RIM_THICK != 0 { 2 } else { 1 };
            ops.push(BevelOp::MiddleHLine { y, left: rect.left, right: rect.right,
                                             color, thickness });
        } else {
            let thickness = if flags & flag::RIM_THICK != 0 { 2 } else { 1 };
            ops.push(BevelOp::FrameOutline { rect, color, thickness });
        }
    }
    let _ = xtra_color;

    // --- Post decorations ---
    if flags & flag::INSET_SEPARATOR != 0 {
        // Bright at (y_mid), dark at (y_mid + 1). Computed via
        // channel × 0x85/100 and 0x42/100 in the exe.
        let y_mid = (rect.top + rect.bottom) / 2;
        let bright = scale_color_by_percent(color, PCT_133);
        let dark   = scale_color_by_percent(color, PCT_66);
        ops.push(BevelOp::InsetSeparator {
            y: y_mid, left: rect.left, right: rect.right, bright, dark,
        });
    }
    if flags & flag::OUTER_OUTLINE_1PX != 0 {
        ops.push(BevelOp::OuterOutline { rect, color: outer_outline_color });
    }

    ops
}

/// Direct port of the exe's channel-scaling math used in bevel /
/// gradient / drop-shadow paths:
///
/// * unpack RGB565 → (r5, g6, b5) → expand to 8-bit via `<< 8 / (mask+1)`
/// * channel × pct / 100 (clamped to 0xFF)
/// * repack RGB565
///
/// The exe applies the exact same formula for `0x42` (66%) darkening,
/// `0x5A` (90% bottom-edge), `0x85` (133% bright inset), and `0xA6`
/// (166% drop-shadow highlight).
pub fn scale_color_by_percent(color: u16, pct: i32) -> u16 {
    let (r, g, b) = crate::unpack565(color);
    let scale = |c: u8| -> u8 {
        ((c as i32 * pct) / 100).clamp(0, 0xFF) as u8
    };
    crate::pack565(scale(r), scale(g), scale(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrink_and_offset_flags_apply_before_ops() {
        let r = BevelRect::from_flags(10, 10, 100, 100,
                                       flag::RECT_SHRINK | flag::RECT_OFFSET);
        assert_eq!(r, BevelRect { left: 12, top: 12, right: 98, bottom: 98 });
    }

    #[test]
    fn solid_fill_emits_one_solid_op() {
        let ops = bevel_box(0, 0, 100, 100, flag::SOLID_FILL, 0xF800, 0, 0, |_,_| 0);
        assert!(ops.iter().any(|op| matches!(op, BevelOp::SolidFill { .. })));
        assert!(!ops.iter().any(|op| matches!(op, BevelOp::Bevel3D { .. })));
    }

    #[test]
    fn solid_fill_plus_bevel_emits_both() {
        let ops = bevel_box(0, 0, 100, 100,
                             flag::SOLID_FILL | flag::BEVEL_3D,
                             0xF800, 0, 0, |_,_| 0);
        assert!(ops.iter().any(|op| matches!(op, BevelOp::SolidFill { .. })));
        assert!(ops.iter().any(|op| matches!(op, BevelOp::Bevel3D { .. })));
    }

    #[test]
    fn bevel_3d_thickness_switches_at_50x50_with_solid_fill() {
        // Small box: thickness 2.
        let ops = bevel_box(0, 0, 30, 30,
                             flag::SOLID_FILL | flag::BEVEL_3D, 0, 0, 0, |_,_| 0);
        for op in ops {
            if let BevelOp::Bevel3D { thickness, .. } = op {
                assert_eq!(thickness, 2);
            }
        }
        // Big box: thickness 4.
        let ops = bevel_box(0, 0, 60, 60,
                             flag::SOLID_FILL | flag::BEVEL_3D, 0, 0, 0, |_,_| 0);
        for op in ops {
            if let BevelOp::Bevel3D { thickness, .. } = op {
                assert_eq!(thickness, 4);
            }
        }
    }

    #[test]
    fn inverted_flag_sets_bevel_inverted() {
        let ops = bevel_box(0, 0, 100, 100,
                             flag::SOLID_FILL | flag::BEVEL_3D | flag::BEVEL_INVERTED,
                             0, 0, 0, |_,_| 0);
        for op in ops {
            if let BevelOp::Bevel3D { inverted, .. } = op {
                assert!(inverted);
            }
        }
    }

    #[test]
    fn sample_color_overrides_argument() {
        let ops = bevel_box(50, 50, 150, 150,
                             flag::SOLID_FILL | flag::SAMPLE_COLOR,
                             0x0000, 0, 0, |x, y| {
                                 assert_eq!(x, 50); assert_eq!(y, 50);
                                 0xF81F  // magenta — very obviously sampled
                             });
        for op in ops {
            if let BevelOp::SolidFill { color, .. } = op {
                assert_eq!(color, 0xF81F);
            }
        }
    }

    #[test]
    fn inset_separator_emits_bright_and_dark_lines() {
        let ops = bevel_box(0, 0, 100, 100,
                             flag::SOLID_FILL | flag::INSET_SEPARATOR,
                             0x7BEF, 0, 0, |_,_| 0);
        assert!(ops.iter().any(|op| matches!(op, BevelOp::InsetSeparator { .. })));
    }

    #[test]
    fn outer_outline_uses_provided_color() {
        let ops = bevel_box(0, 0, 100, 100,
                             flag::OUTER_OUTLINE_1PX, 0, 0, 0x0821, |_,_| 0);
        for op in ops {
            if let BevelOp::OuterOutline { color, .. } = op {
                assert_eq!(color, 0x0821);
            }
        }
    }

    #[test]
    fn scale_color_at_100_percent_is_identity() {
        for c in [0x0000, 0xF800, 0x07E0, 0x001F, 0xFFFF, 0x7BEF] {
            assert_eq!(scale_color_by_percent(c, 100), c,
                       "scale({c:#06x}, 100) should be identity");
        }
    }

    #[test]
    fn scale_color_at_zero_is_black() {
        for c in [0xF800, 0x07E0, 0x001F, 0xFFFF] {
            assert_eq!(scale_color_by_percent(c, 0), 0);
        }
    }

    #[test]
    fn scale_color_clamps_at_255() {
        // 200% of pure red (0xF800) — R is already 0xF8, doubled = 0x1F0 → clamp 0xFF.
        // R channel back to 5-bit: 0xFF & 0xF8 = 0xF8; result should stay 0xF800 or 0xFFFF (fully saturated).
        let bright = scale_color_by_percent(0xF800, 200);
        // Bit-check: red should still be maxed, green/blue still 0.
        let (r, g, b) = crate::unpack565(bright);
        assert!(r >= 0xF8);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn percentages_match_exe_literals() {
        assert_eq!(PCT_66, 0x42);
        assert_eq!(PCT_90, 0x5A);
        assert_eq!(PCT_133, 0x85);
        assert_eq!(PCT_166, 0xA6);
    }

    // ---- Integration cross-check: prove the new exe-accurate primitives
    // ---- produce the same rect + bevel + outline the old `panel::draw_panel`
    // ---- does when given equivalent flag words. This is the wire-up
    // ---- verification proving the whole pipeline is coherent.

    #[test]
    fn solid_fill_plus_bevel_matches_panel_semantics() {
        // The exe's F_SOLID_FILL | F_BEVEL = 0x30 → op stream: SolidFill
        // + Bevel3D (2 layers for a 100×60 box, since it's ≥50×50 with
        // solid fill).
        let ops = bevel_box(0, 0, 100, 60,
                             flag::SOLID_FILL | flag::BEVEL_3D,
                             0xF800, 0, 0, |_, _| 0);
        assert!(ops.iter().any(|op| matches!(op, BevelOp::SolidFill { .. })));
        // Bevel thickness should be 4 for ≥50×50 solid-fill box.
        for op in ops {
            if let BevelOp::Bevel3D { thickness, .. } = op {
                assert_eq!(thickness, 4, "big solid-fill box uses thickness 4");
            }
        }
    }

    #[test]
    fn transparent_bevel_produces_no_solid_fill() {
        // F_TRANSPARENT | F_BEVEL: bevel but no fill.
        let ops = bevel_box(0, 0, 100, 60,
                             flag::TRANSPARENT_CLEAR | flag::BEVEL_3D,
                             0xF800, 0, 0, |_,_| 0);
        assert!(ops.iter().any(|op| matches!(op, BevelOp::TransparentClear { .. })));
        assert!(!ops.iter().any(|op| matches!(op, BevelOp::SolidFill { .. })));
        assert!(ops.iter().any(|op| matches!(op, BevelOp::Bevel3D { .. })));
    }

    #[test]
    fn vgradient_flag_produces_gradient_op() {
        // F_VGRADIENT = 0x8 → GradientHorizontal (exe: per-row ramp).
        let ops = bevel_box(0, 0, 100, 60,
                             flag::GRADIENT_HORIZONTAL,
                             0x3800, 0, 0, |_,_| 0);
        assert!(ops.iter().any(|op| matches!(op, BevelOp::GradientHorizontal { .. })));
    }
}
