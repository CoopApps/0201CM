//! Layer 3 → Layer 2 bridge — the production `to_render_widget`.
//!
//! `widget_pool::Widget` (Layer 3 — spawn-time state, holds grid arrays,
//! z-order children, parent-area link) and `packed_widget::Widget`
//! (Layer 2 — render-time state, holds cached bitmaps and hover flags)
//! model the same underlying widget struct at different phases of its
//! lifetime. The 6 style slots (`style_byte`, `text_style`, `colour_a`,
//! `colour_b`, `label_ink`, `pattern`) plus the label buffer must flow
//! through unchanged — the layer-2 draw code reads them verbatim from
//! `[ebp+0x38/0x3c/0x72/0x74/0x76/0x78/0x80]` and every screen builder
//! is expected to have written them at spawn.
//!
//! Previously each screen file (`screen_nav_back_next.rs`,
//! `screen_news.rs`, `screen_menu_bar.rs`) hand-rolled a `to_render_widget`
//! stub in its `#[cfg(test)]` block, filling many fields with placeholders
//! because the descriptor was mislabelled and `spawn_widget` was
//! dropping the style fields on the floor. Both bugs are fixed on this
//! commit; this module is the single canonical bridge those stubs now
//! delegate to.
//!
//! # Why not merge the two Widget structs?
//!
//! They carry disjoint runtime state:
//!
//! * `widget_pool::Widget` — spawn-time & layout-engine fields
//!   (grid weight arrays `col_left_x[8]` / `row_top_y[8]`, z-order
//!   `Vec<u16>`, parent-area link, `descriptor` bundle). Written by
//!   `spawn_widget` and `FUN_00403240` (z-order insert).
//! * `packed_widget::Widget` — render-time & GDI-cache fields
//!   (`saved_bg: Option<SavedRect>`, `cached_text: Option<CachedIcon>`,
//!   `alt_hover`, `frame_base`). Written by `render_widget` on each
//!   paint.
//!
//! Merging them would mean every ported layer-2 draw fn takes a
//! full-fat Layer-3 widget, and the giant fields it doesn't read (the
//! grid weight arrays alone are 240 bytes) would still cost bandwidth.
//! Keeping them separate + bridging is the exe's own split too: the
//! layout engine mutates the pool copy; the draw callback reads through
//! a compact view of the same bytes.
//!
//! If a future refactor unifies them, this module collapses to a no-op
//! and every caller keeps working.

use crate::packed_widget::Widget as RenderWidget;
use crate::widget_pool::Widget as PoolWidget;

/// Copy every layer-2-relevant field from `pool_widget` into a fresh
/// `packed_widget::Widget` ready for `render_widget`.
///
/// * `frame_base` is left at 0 — the render callback re-seeds it from
///   `[ebp+0x00]` on entry, which for our pipeline is the area-pool
///   base pointer supplied by the caller of `render_widget`.
/// * `text_kern` defaults to `-1` (no caret marker), matching the exe's
///   `param_1[0x11] = -1` init at 005d7d13 for widgets without the
///   `flags & 0x18` string-length-cache path.
/// * `saved_bg` / `cached_text` / `detached_glyph_cache` / `alt_hover`
///   are per-paint state; they start empty and the render pass fills them.
/// * `frame_idx` starts at `-1` (`0xffff`) matching the exe's init at
///   005d7d43 (`param_1[0x11] = -1`).
pub fn to_render_widget(pool_widget: &PoolWidget) -> RenderWidget {
    RenderWidget {
        frame_base: 0,
        // +0x0c widget flags word — spawn_widget stores desc.kind there.
        flags: pool_widget.flags,
        x0: pool_widget.left,
        y0: pool_widget.top,
        x1: pool_widget.right,
        y1: pool_widget.bottom,
        style_byte: pool_widget.style_byte,
        text_style: pool_widget.text_style,
        text_kern: -1,
        saved_bg: None,
        cached_text: None,
        colour_a: pool_widget.colour_a,
        colour_b: pool_widget.colour_b,
        label_ink: pool_widget.label_ink,
        pattern: pool_widget.pattern,
        frame_idx: -1,
        label: pool_widget.label.clone(),
        detached_glyph_cache: 0,
        alt_hover: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget_pool::{Widget, WidgetDescriptor, GuiRecordPool};

    #[test]
    fn every_style_field_roundtrips_through_the_bridge() {
        // Build a Widget by hand carrying distinct sentinel values in
        // every style slot the bridge is supposed to copy. Every one
        // must appear verbatim on the resulting RenderWidget.
        let mut w = Widget::default();
        w.left = 100; w.top = 55; w.right = 220; w.bottom = 88;
        w.flags = 0x12345678;
        w.style_byte = 0xAABBCCDD;
        w.text_style = 0x0000000C;
        w.colour_a = 0x7FE0;
        w.colour_b = 0x1234;
        w.label_ink = 0x43FF;
        w.pattern = 0x0200;
        w.label = b"hello world\0".to_vec();

        let r = to_render_widget(&w);
        assert_eq!(r.x0, 100);
        assert_eq!(r.y0, 55);
        assert_eq!(r.x1, 220);
        assert_eq!(r.y1, 88);
        assert_eq!(r.flags, 0x12345678);
        assert_eq!(r.style_byte, 0xAABBCCDD);
        assert_eq!(r.text_style, 0x0000000C);
        assert_eq!(r.colour_a, 0x7FE0);
        assert_eq!(r.colour_b, 0x1234);
        assert_eq!(r.label_ink, 0x43FF);
        assert_eq!(r.pattern, 0x0200);
        assert_eq!(r.label, b"hello world\0");
        // Defaults on the render-time-only slots.
        assert_eq!(r.frame_base, 0);
        assert_eq!(r.text_kern, -1);
        assert!(r.saved_bg.is_none());
        assert!(r.cached_text.is_none());
        assert_eq!(r.frame_idx, -1);
        assert_eq!(r.detached_glyph_cache, 0);
        assert_eq!(r.alt_hover, 0);
    }

    #[test]
    fn spawn_widget_populates_all_style_fields_on_widget() {
        // The bug this commit fixes: spawn_widget used to drop desc.*
        // style fields silently. Verify they now round-trip from
        // descriptor → Widget → bridge → RenderWidget.
        let mut pool = GuiRecordPool::new();
        let mut d = WidgetDescriptor::empty();
        d.kind = 0x400;              // widget flags word
        d.grid_x0 = 10; d.grid_y0 = 20;
        d.grid_x1 = 100; d.grid_y1 = 60;
        d.style_byte = 0x30;
        d.text_style = 0x0C;
        d.colour_a = 0x7FE0;
        d.colour_b = 0x1234;
        d.label_ink = 0x43FF;
        d.pattern = 0x0200;
        d.text = "Test".to_string();
        let idx = pool.spawn_widget(d, -1).unwrap();
        let pw = &pool.widgets[idx as usize];
        assert_eq!(pw.style_byte, 0x30);
        assert_eq!(pw.text_style, 0x0C);
        assert_eq!(pw.colour_a, 0x7FE0);
        assert_eq!(pw.colour_b, 0x1234);
        assert_eq!(pw.label_ink, 0x43FF);
        assert_eq!(pw.pattern, 0x0200);
        assert_eq!(pw.flags, 0x400);
        assert_eq!(&pw.label[..pw.label.len() - 1], b"Test");

        let r = to_render_widget(pw);
        assert_eq!(r.style_byte, 0x30);
        assert_eq!(r.text_style, 0x0C);
        assert_eq!(r.colour_a, 0x7FE0);
        assert_eq!(r.colour_b, 0x1234);
        assert_eq!(r.label_ink, 0x43FF);
        assert_eq!(r.pattern, 0x0200);
        assert_eq!(r.flags, 0x400);
    }
}
