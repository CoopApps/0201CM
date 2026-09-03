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

    let back = WidgetDescriptor {
        kind: back_kind,
        grid_x0: 0, grid_y0: 0, grid_x1: 0, grid_y1: 0,   // grid-relative
        seq: 0,                                              // col 0
        row_index: 0,
        flags: 0x30,
        unk9:  back_color_a as i32,   // arg 9  — color_a
        unk10: back_color_b as i32,   // arg 10 — color_b
        font_id: back_font,           // arg 11 — aux_a
        enabled: true,                // arg 12 — font (== 3 in call, TRUE-ish)
        fg_color: label_ink_word as u32, // arg 13 — aux_b = DAT_00acdf74
        text: "Back".to_string(),     // arg 14 — text_ptr → local_7d0
        extra: Vec::new(),            // arg 15 — 0
        msg_id: back_msg,             // arg 16 — event
        userdata_id: 0,               // arg 17 — 0
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
        seq: 1,                       // col 1  — arg 6 = 1
        row_index: 0,
        flags: 0x30,
        unk9:  next_color_a as i32,
        unk10: next_color_b as i32,
        font_id: next_font,
        enabled: true,
        fg_color: label_ink_word as u32,
        text: "Next".to_string(),
        extra: Vec::new(),
        msg_id: next_msg,
        userdata_id: 0,
    };
    pool.spawn_widget(next, area as i16)?;

    Some(area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packed::PackedSurface;
    use crate::packed_glyph::PixelFont;
    use crate::packed_widget::{render_widget, Widget as RWidget, WidgetGlobals};
    use crate::widget_pool::Widget as PoolWidget;

    /// Bridge a pool widget's spawn-time descriptor into the render-time
    /// [`packed_widget::Widget`]. The two structs live in different
    /// crates (pool = spawner state, render = per-paint state); this
    /// converter is the minimum needed to prove Layer 3 → Layer 2 wiring.
    fn to_render_widget(pw: &PoolWidget) -> RWidget {
        let mut label: Vec<u8> = pw.descriptor.text.as_bytes().to_vec();
        label.push(0); // NUL-terminate per exe convention
        RWidget {
            frame_base: 0,
            flags: pw.descriptor.flags | pw.flags,
            // Post-layout rect — for the test we just use the widget's
            // own left/top/right/bottom (spawn clamps them to 800×600).
            // In a full pipeline layout engine would set these; the exe
            // seeds them to the spawn-time grid coords too.
            x0: pw.left.max(0),
            y0: pw.top.max(0),
            x1: pw.right.max(pw.left + 1),
            y1: pw.bottom.max(pw.top + 1),
            style_byte: 0x10, // P_SOLID_FILL — minimum to paint
            text_style: 0x0c,
            text_kern: -1,
            saved_bg: None,
            cached_text: None,
            colour_a: pw.descriptor.unk9 as u16, // arg9 in the spawn = color_a
            colour_b: pw.descriptor.unk10 as u16,
            label_ink: pw.descriptor.fg_color as u16,
            pattern: 0,
            frame_idx: -1,
            label,
            detached_glyph_cache: 0,
            alt_hover: 0,
        }
    }

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
        // back_flag != 0 → kind=2 button, msg_id=-2, font_id=0xc.
        let mut pool = GuiRecordPool::new();
        build_nav_back_next(&mut pool, 1, 0, 0x42, 0x1234).unwrap();
        let back = &pool.widgets[0].descriptor;
        assert_eq!(back.kind, KIND_BUTTON);
        assert_eq!(back.msg_id, -2);
        assert_eq!(back.font_id, 0x0c);
        assert_eq!(back.unk9,  0x42);   // color_a from panel byte
        assert_eq!(back.unk10, 0x42);   // color_b also from panel byte
        assert_eq!(back.fg_color, 0x1234);
    }

    #[test]
    fn next_active_branch_matches_c_constants() {
        // next_flag != 0 → kind=2 button, msg_id=-3, font_id=0xc.
        let mut pool = GuiRecordPool::new();
        build_nav_back_next(&mut pool, 0, 1, 0x42, 0x1234).unwrap();
        let next = &pool.widgets[1].descriptor;
        assert_eq!(next.kind, KIND_BUTTON);
        assert_eq!(next.msg_id, -3);
        assert_eq!(next.font_id, 0x0c);
        assert_eq!(next.unk9,  0x42);
        assert_eq!(next.unk10, 0x42);
        assert_eq!(next.fg_color, 0x1234);
    }

    #[test]
    fn back_active_branch_matches_c_constants() {
        // back_flag == 0 → kind=1 label, msg_id=0, font_id=0x2c,
        // color_b = 0 (uVar4 = 0).
        let mut pool = GuiRecordPool::new();
        build_nav_back_next(&mut pool, 0, 0, 0x42, 0x1234).unwrap();
        let back = &pool.widgets[0].descriptor;
        assert_eq!(back.kind, KIND_LABEL);
        assert_eq!(back.msg_id, 0);
        assert_eq!(back.font_id, 0x2c);
        assert_eq!(back.unk9,  0x42);
        assert_eq!(back.unk10, 0);
    }

    #[test]
    fn built_pool_renders_through_layer_2_without_panic() {
        // Populate → convert → render every widget through the ported
        // Layer 2 dispatcher onto a fresh 800×600 RGB555 surface.
        // Just proves the wiring is intact: any pixel painted, no panic.
        let mut pool = GuiRecordPool::new();
        build_nav_back_next(&mut pool, 0, 0, 0x01, 0x0000).unwrap();
        let mut s = PackedSurface::rgb555(800, 600);
        let font = stub_font();
        for pw in pool.widgets.iter() {
            let mut rw = to_render_widget(pw);
            render_widget(
                &mut s,
                &mut rw,
                Some(&pool),
                &font,
                WidgetGlobals::default(),
                true,
            );
        }
        // Some pixel painted (block B fills the widget rect via
        // draw_panel with style 0x10 = P_SOLID_FILL).
        assert!(
            s.buf.iter().any(|&p| p != 0),
            "at least one pixel should be painted"
        );
    }
}
