//! Integration test — verify the Layer-4 → Layer-3 → Layer-2 fold
//! actually paints something.
//!
//! For each dispatcher arm we expose to `cm-ui-app`, drive
//! `dispatch_global` with the cmd as the pending-cmd fallback, expect
//! `Handled`, expect the pool to be non-empty (a real screen was
//! built), and expect the rendered `PackedSurface` to carry at least
//! one non-zero pixel (widgets actually painted).

use cm_render::dispatcher::{dispatch_global, DispatchResult, DispatcherState};
use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::PixelFont;
use cm_render::packed_widget::{render_widget, WidgetGlobals};
use cm_render::pool_to_render::to_render_widget;
use cm_render::widget_pool::GuiRecordPool;

fn drive(cmd: i16) -> (DispatchResult, GuiRecordPool, PackedSurface) {
    let mut pool = GuiRecordPool::new();
    let mut state = DispatcherState::default();
    state.pending_cmd_fallback = cmd;
    let r = dispatch_global(&mut pool, &mut state, -1);

    let mut surface = PackedSurface::rgb555(800, 600);
    // Pre-fill with a sentinel that no rendered pixel is likely to
    // legitimately equal after paint — 0xFFFF (white in 555 packed).
    // Then "was painted" == "any pixel changed from the sentinel".
    for p in surface.buf.iter_mut() {
        *p = 0xFFFF;
    }
    let font = PixelFont::empty(11);
    let n = pool.widgets.len();
    for i in 0..n {
        let mut rw = to_render_widget(&pool.widgets[i]);
        rw.frame_idx = -1; // avoid area-out-of-range in block G
        render_widget(
            &mut surface,
            &mut rw,
            Some(&pool),
            &font,
            WidgetGlobals::default(),
            true,
        );
    }
    (r, pool, surface)
}

fn assert_non_empty(surface: &PackedSurface) {
    let painted = surface.buf.iter().any(|&p| p != 0xFFFF);
    assert!(painted, "surface should have at least one pixel that
        differs from the pre-fill sentinel (widgets should have painted)");
}

#[test]
fn dispatch_0x418_latest_scores_builds_pool_and_renders() {
    let (r, pool, surface) = drive(0x418);
    assert_eq!(r, DispatchResult::Handled);
    assert!(!pool.widgets.is_empty(), "0x418 should populate the pool");
    assert_non_empty(&surface);
}

#[test]
fn dispatch_0x3f3_fifa_rankings_builds_pool_and_renders() {
    let (r, pool, surface) = drive(0x3f3);
    assert_eq!(r, DispatchResult::Handled);
    assert!(!pool.widgets.is_empty(), "0x3f3 should populate the pool");
    assert_non_empty(&surface);
}

#[test]
fn dispatch_0x3ec_manager_history_handled() {
    let (r, pool, _surface) = drive(0x3ec);
    assert_eq!(r, DispatchResult::Handled);
    assert!(!pool.widgets.is_empty(), "0x3ec should populate the pool");
}

#[test]
fn dispatch_unknown_cmd_is_unhandled() {
    let (r, pool, _surface) = drive(0x0001);
    // 0x0001 has no arm, so the fallback resolution returns 0 (empty
    // widget_slot with pending_cmd_fallback pre-cleared inside the
    // dispatcher's ancestry) — the outer arm falls to Unhandled.
    // Either Unhandled or one of the Todo* variants is acceptable
    // (some low cmds have no live route yet); the constraint here
    // is: not Handled.
    assert!(
        !matches!(r, DispatchResult::Handled),
        "unmapped cmd should not be Handled, got {:?}",
        r
    );
    assert!(pool.widgets.is_empty(), "no arm ran → pool stays empty");
}
