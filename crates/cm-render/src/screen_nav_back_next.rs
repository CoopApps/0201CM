//! Layer 3 — Back/Next nav-bar screen builder.
//!
//! Direct port of `FUN_005d75b0` from cm0102.exe (decompile at
//! `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/005d75b0.c`,
//! 50 lines of C ≈ ~110 asm instructions).
//!
//! This is the canonical Layer 3 exemplar — a screen builder that
//! populates the [`GuiRecordPool`] via
//! [`spawn_area`][GuiRecordPool::spawn_area] / [`spawn_widget`]
//! [GuiRecordPool::spawn_widget]. Layer 2 ([`render_widget`]) then
//! draws each widget from the pool. Every widget below comes from a
//! spawn call in the original C — no fabricated geometry.
//!
//! # Original C
//!
//! ```c
//! undefined4 FUN_005d75b0(int param_1, int param_2)
//! {
//!   local_7d6 = 3;  local_7d5 = 1;                       // col weights [3,1]
//!   uVar1 = FUN_00549790(100, 0x22b, 0x316, 0x24e, 2,    // area (100,555,790,590)
//!                         &local_7d6, 1, 0, 1, 0, 0xffffffff);
//!   FUN_006547c0(local_7d0, &DAT_00975120);              // strcpy "Back"
//!   if (param_1 == 0) { ...disabled-style Back... }
//!   else            { ...active-style Back... }
//!   FUN_00549580(...Back...);
//!   FUN_006547c0(local_7d0, &DAT_00975118);              // strcpy "Next"
//!   if (param_2 != 0) { FUN_00549580(...active Next...); return 1; }
//!   FUN_00549580(...static Next...);
//!   return 1;
//! }
//! ```
//!
//! Prior analysis (`d:/cm0102-carve/analysis/helpers.json`) decodes
//! the two flags as:
//!
//! * `back_flag == 0` — Back is a normal button (kind=2 button,
//!   `msg_id = 0`, `font_id = 0x2c` = F_SHADOW, `color_a`/`color_b` =
//!   panel byte).
//! * `back_flag != 0` — Back is drawn disabled/greyed (kind=1 label
//!   holder, `msg_id = -2`, `font_id = 0xc`, both colours packed
//!   from panel byte).
//! * `next_flag == 0` — Next is a static label (kind=1, `msg_id = 0`,
//!   `font_id = 0x2c`, `color_a` = panel byte, `color_b` = 0).
//! * `next_flag != 0` — Next is an active button (kind=2, `msg_id = -3`,
//!   `font_id = 0xc`, both colours packed from panel byte).
//!
//! # State inputs
//!
//! * `back_flag` / `next_flag` — the two `param_1` / `param_2` inputs.
//! * `panel_colour_byte` — the `DAT_00acdf6e` byte (u8 palette index).
//!   In the exe this is a global set by the theme; the caller passes
//!   the resolved value.
//! * `label_ink_word` — the `DAT_00acdf74` word (u16 packed RGB555).
//!   Same story — resolved by the caller.
//!
//! # Return
//!
//! The area index that both buttons attach to (matches the exe's
//! `return 1` semantics of "built OK" — we return the actual area
//! handle so callers can chain further widgets into it if they want).

use crate::widget_pool::{
    GuiRecordPool, WidgetDescriptor, KIND_BUTTON, KIND_LABEL,
};

/// Direct port of `FUN_005d75b0`. Spawns one area + two widgets into
/// `pool` and returns the area index.
///
/// Returns `None` iff the pool overflows (widget cap `0x4AF` or area
/// cap `0xF8`) — matches the exe's `-1` overflow return path.
pub fn build_nav_back_next(
    pool: &mut GuiRecordPool,
    back_flag: i32,
    next_flag: i32,
    panel_colour_byte: u8,
    label_ink_word: u16,
) -> Option<u32> {
    // ------------------------------------------------------------------
    // C:  local_7d6 = 3;
    //     local_7d5 = 1;
    //     uVar1 = FUN_00549790(100, 0x22b, 0x316, 0x24e, 2,
    //                          &local_7d6, 1, 0, 1, 0, 0xffffffff);
    //
    // Area geometry: (100, 555, 790, 590) — the bottom nav strip.
    // nchildren_hint = 2 (two columns), extra = [3, 1] (column weights),
    // color_slot = 1, gradient_ptr = 0, border_style = 1, bg_color = 0,
    // parent_area = -1 (root).
    // ------------------------------------------------------------------
    let area = pool.spawn_area(
        100, 0x22b, 0x316, 0x24e,
        2,
        vec![3, 1],
        1,
        0,
        1,
        0,
        -1,
    )?;

    // ------------------------------------------------------------------
    // C:  FUN_006547c0(local_7d0, &DAT_00975120);   // "Back"
    //     if (param_1 == 0) {
    //         uVar2 = (uint)DAT_00acdf6e;   uVar6 = 0;
    //         uVar5 = 0x2c;                 uVar4 = 0;
    //         uVar3 = 1;
    //     } else {
    //         uVar6 = 0xfffffffe;                                  // -2
    //         local_7d4 = CONCAT22(_, DAT_00acdf6e);
    //         uVar5 = 0xc;                  uVar3 = 2;
    //         uVar2 = local_7d4;            uVar4 = local_7d4;
    //     }
    //     FUN_00549580(uVar3, 0, 0, 0, 0, 0, 0, 0x30,
    //                  uVar2, uVar4, uVar5, 3,
    //                  CONCAT22(_, DAT_00acdf74),
    //                  local_7d0, 0, uVar6, 0, uVar1);
    // ------------------------------------------------------------------
    // NB: `helpers.json` records that no observed screen ever sets
    // back_flag != 0 — but we port both branches faithfully, per the C.
    let (back_kind, back_color_a, back_color_b, back_font, back_msg) =
        if back_flag == 0 {
            // Active-button branch (the observed path).
            (KIND_LABEL,           // uVar3 = 1
             panel_colour_byte as u32,  // uVar2 = (uint)DAT_00acdf6e
             0u32,                 // uVar4 = 0
             0x2cu8,               // uVar5 = 0x2c
             0i32)                 // uVar6 = 0
        } else {
            // Disabled/greyed branch. `CONCAT22(hi, DAT_00acdf6e)`
            // in the exe leaves the high 16 bits as stack garbage —
            // the port uses a defined zero-extension since Rust has
            // no uninitialised u32.
            (KIND_BUTTON,          // uVar3 = 2
             panel_colour_byte as u32,  // uVar2 = local_7d4 (low byte)
             panel_colour_byte as u32,  // uVar4 = local_7d4
             0x0cu8,               // uVar5 = 0xc
             -2i32)                // uVar6 = 0xfffffffe
        };

    // C call: FUN_00549580(uVar3, 0, 0, 0, 0, 0, 0, 0x30, uVar2, uVar4,
    //                      uVar5, 3, CONCAT22(_, DAT_00acdf74),
    //                      local_7d0, 0, uVar6, 0, uVar1)
    // Per FUN_005d76c0 field mapping (see pool_to_render.rs docs):
    //   arg1 = kind (widget flags dword, +0x0c)
    //   arg8 = style_byte  (+0x38)
    //   arg9,10 = colour_a, colour_b  (+0x72, +0x74)
    //   arg11 = text_style             (+0x3c) — 0x2c label / 0xc button
    //   arg12 = label_ink              (+0x76) — literal 3 in the C
    //   arg13 = pattern                (+0x78) — DAT_00acdf74 (gold)
    //   arg14 = text                    strcpy → +0x80
    //   arg16 = msg_id                 (+0x08)
    let back = WidgetDescriptor {
        kind: back_kind,
        grid_x0: 0, grid_y0: 0, grid_x1: 0, grid_y1: 0,
        seq: 0,
        row_index: 0,
        style_byte: 0x30,                          // arg8 (C: 0x30)
        colour_a: back_color_a as u16,             // arg9 (C: uVar2)
        colour_b: back_color_b as u16,             // arg10 (C: uVar4)
        text_style: back_font as u32,              // arg11 (C: uVar5)
        label_ink: 3,                              // arg12 (C: literal 3)
        pattern: label_ink_word,                   // arg13 (C: DAT_00acdf74)
        text: "Back".to_string(),                  // arg14 (C: local_7d0)
        slot_40: 0,                                // arg15
        msg_id: back_msg,                          // arg16 (C: uVar6)
        userdata_id: 0,                            // arg17
    };
    pool.spawn_widget(back, area as i16)?;

    // ------------------------------------------------------------------
    // C:  FUN_006547c0(local_7d0, &DAT_00975118);   // "Next"
    //     if (param_2 != 0) {
    //         local_7d4 = CONCAT22(_, DAT_00acdf6e);
    //         FUN_00549580(2, 0, 0, 0, 0, 1, 0, 0x30,
    //                      local_7d4, local_7d4, 0xc, 3,
    //                      CONCAT22(_, DAT_00acdf74), local_7d0,
    //                      0, 0xfffffffd, 0, uVar1);
    //         return 1;
    //     }
    //     FUN_00549580(1, 0, 0, 0, 0, 1, 0, 0x30,
    //                  DAT_00acdf6e, 0, 0x2c, 3,
    //                  CONCAT22(_, DAT_00acdf74), local_7d0,
    //                  0, 0, 0, uVar1);
    //     return 1;
    // ------------------------------------------------------------------
    let (next_kind, next_color_a, next_color_b, next_font, next_msg) =
        if next_flag != 0 {
            // Active-button branch.
            (KIND_BUTTON,           // 2
             panel_colour_byte as u32,
             panel_colour_byte as u32,
             0x0cu8,
             -3i32)                 // 0xfffffffd
        } else {
            // Static-label branch (the observed path).
            (KIND_LABEL,            // 1
             panel_colour_byte as u32,
             0u32,
             0x2cu8,
             0i32)
        };

    let next = WidgetDescriptor {
        kind: next_kind,
        grid_x0: 0, grid_y0: 0, grid_x1: 0, grid_y1: 0,
        seq: 1,                            // arg6 = 1
        row_index: 0,
        style_byte: 0x30,                  // arg8
        colour_a: next_color_a as u16,     // arg9
        colour_b: next_color_b as u16,     // arg10
        text_style: next_font as u32,      // arg11
        label_ink: 3,                      // arg12 (C: literal 3)
        pattern: label_ink_word,           // arg13 (DAT_00acdf74)
        text: "Next".to_string(),          // arg14
        slot_40: 0,                        // arg15
        msg_id: next_msg,                  // arg16
        userdata_id: 0,                    // arg17
    };
    pool.spawn_widget(next, area as i16)?;

    Some(area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packed::PackedSurface;
    use crate::packed_glyph::PixelFont;
    use crate::packed_widget::{render_widget, WidgetGlobals};

    // Bridge lives in `crate::pool_to_render::to_render_widget` — the
    // production version copies every style field from the pool widget
    // to the render widget 1:1.
    use crate::pool_to_render::to_render_widget;

    fn stub_font() -> PixelFont { PixelFont::empty(10) }

    #[test]
    fn builder_spawns_one_area_and_two_widgets_observed_flags() {
        // Observed flags per helpers.json: back_flag = 0, next_flag = 0
        // (the only combination any live screen sets).
        let mut pool = GuiRecordPool::new();
        let area = build_nav_back_next(&mut pool, 0, 0, 0x01, 0x0000)
            .expect("pool should have room");
        assert_eq!(pool.areas.len(), 1);
        assert_eq!(pool.widgets.len(), 2);
        // Both widgets are children of the area (not root).
        assert_eq!(pool.widgets[0].parent_area, area as i16);
        assert_eq!(pool.widgets[1].parent_area, area as i16);
        // Area rect matches (100, 555, 790, 590) after clamp.
        let a = &pool.areas[area as usize];
        assert_eq!((a.x0, a.y0, a.x1, a.y1), (100, 555, 790, 590));
        // First 2 bytes of extra = column weights [3, 1] (the two-byte
        // caller-supplied palette).
        assert_eq!(a.extra[0], 3);
        assert_eq!(a.extra[1], 1);
        // Back = static label (kind=1), Next = static label (kind=1)
        // for the observed-flag combination.
        assert_eq!(pool.widgets[0].descriptor.kind, KIND_LABEL);
        assert_eq!(pool.widgets[0].descriptor.text, "Back");
        assert_eq!(pool.widgets[0].descriptor.seq, 0);
        assert_eq!(pool.widgets[0].descriptor.msg_id, 0);
        assert_eq!(pool.widgets[1].descriptor.kind, KIND_LABEL);
        assert_eq!(pool.widgets[1].descriptor.text, "Next");
        assert_eq!(pool.widgets[1].descriptor.seq, 1);
        assert_eq!(pool.widgets[1].descriptor.msg_id, 0);
    }

    #[test]
    fn back_disabled_branch_matches_c_constants() {
        // back_flag != 0 → kind=2 button, msg_id=-2, text_style=0xc.
        let mut pool = GuiRecordPool::new();
        build_nav_back_next(&mut pool, 1, 0, 0x42, 0x1234).unwrap();
        let back = &pool.widgets[0].descriptor;
        assert_eq!(back.kind, KIND_BUTTON);
        assert_eq!(back.msg_id, -2);
        assert_eq!(back.text_style, 0x0c);
        assert_eq!(back.colour_a, 0x42);   // colour_a from panel byte
        assert_eq!(back.colour_b, 0x42);   // colour_b also from panel byte
        assert_eq!(back.pattern, 0x1234);  // DAT_00acdf74
    }

    #[test]
    fn next_active_branch_matches_c_constants() {
        // next_flag != 0 → kind=2 button, msg_id=-3, text_style=0xc.
        let mut pool = GuiRecordPool::new();
        build_nav_back_next(&mut pool, 0, 1, 0x42, 0x1234).unwrap();
        let next = &pool.widgets[1].descriptor;
        assert_eq!(next.kind, KIND_BUTTON);
        assert_eq!(next.msg_id, -3);
        assert_eq!(next.text_style, 0x0c);
        assert_eq!(next.colour_a, 0x42);
        assert_eq!(next.colour_b, 0x42);
        assert_eq!(next.pattern, 0x1234);
    }

    #[test]
    fn back_active_branch_matches_c_constants() {
        // back_flag == 0 → kind=1 label, msg_id=0, text_style=0x2c,
        // colour_b = 0 (uVar4 = 0).
        let mut pool = GuiRecordPool::new();
        build_nav_back_next(&mut pool, 0, 0, 0x42, 0x1234).unwrap();
        let back = &pool.widgets[0].descriptor;
        assert_eq!(back.kind, KIND_LABEL);
        assert_eq!(back.msg_id, 0);
        assert_eq!(back.text_style, 0x2c);
        assert_eq!(back.colour_a, 0x42);
        assert_eq!(back.colour_b, 0);
    }

    #[test]
    fn built_pool_renders_through_layer_2_without_panic() {
        // Populate → bridge → render every widget through Layer 2 onto
        // a fresh 800×600 RGB555 surface. Proves the wiring is intact:
        // no panic, and the bridge preserves every style field
        // (`build_nav_back_next` passes style_byte=0x30 = bevel+fill).
        //
        // The widgets carry `grid_x0..y1 = 0` (relative to the parent
        // area's cell grid — the real layout engine's job to expand);
        // this test doesn't drive the layout engine, so we force each
        // rect to a small non-empty box before rendering. The bridge
        // itself is production; only the rect-forcing is test-only.
        let mut pool = GuiRecordPool::new();
        build_nav_back_next(&mut pool, 0, 0, 0x01, 0x0000).unwrap();
        let mut s = PackedSurface::rgb555(800, 600);
        let font = stub_font();
        for (i, pw) in pool.widgets.iter().enumerate() {
            let mut rw = to_render_widget(pw);
            // Test-only layout-engine stand-in: give each widget a
            // distinct 40×20 rect so draw_panel has pixels to touch.
            rw.x0 = 100 + (i as i32) * 60;
            rw.y0 = 100;
            rw.x1 = rw.x0 + 40;
            rw.y1 = rw.y0 + 20;
            render_widget(
                &mut s, &mut rw, Some(&pool), &font,
                WidgetGlobals::default(), true,
            );
        }
        // Some pixel painted — the bridge carried style_byte=0x30
        // through, so draw_panel fills the rect.
        assert!(
            s.buf.iter().any(|&p| p != 0),
            "at least one pixel should be painted"
        );
    }
}
