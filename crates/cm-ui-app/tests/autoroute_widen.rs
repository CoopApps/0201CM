//! Widen-fold coverage — every cmd advertised by
//! `render_new::is_dispatch_handled` must actually route through the new
//! pipeline (a `dispatch_global` / `dispatch_club` LIVE arm builds a
//! non-empty pool and paints it into a `PackedSurface`).
//!
//! Complements the two curated screens (`LatestScores`, `FifaRankings`)
//! that already had per-arm coverage in `cm-render`'s dispatch_pool_render
//! integration test.

use cm_render::dispatcher::{dispatch_club, dispatch_global, DispatchResult, DispatcherState};
use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::PixelFont;
use cm_render::packed_widget::{render_widget, WidgetGlobals};
use cm_render::pool_to_render::to_render_widget;
use cm_render::widget_pool::GuiRecordPool;

/// The complete widened cmd list — must stay in lock-step with
/// `render_new::is_dispatch_handled` in the app binary. Keeping this
/// as a const array in the test rather than depending on the binary's
/// helper keeps this file's compile independent of `main.rs`.
const HANDLED_CMDS: &[i16] = &[
    // dispatch_global LIVE routes
    1000, 0x418, 0x3f2, 0x3ec, 0x3ef, 0x414, 0x3f3, 0x40c,
    0x3f4, 0x3f5, 0x3f6, 0x3f7, 0x3f8, 0x3f9, 0x3fa, 0x3fc,
    0x431, 0x42b, 0x42c,
    // dispatch_club LIVE routes
    0x7d1, 0x7d3, 0x7d4, 0x7d7, 0x7e7, 0x7d8, 0x7da, 0x7d9,
    0x7db, 0x7dc, 0x7dd, 0x7df, 0x7de, 0x7e5, 0x7e6,
];

/// Sanity check on the widened cmd count. Prior commit `89034cd` shipped
/// only 2 curated screens; this list widens it to 34.
#[test]
fn widened_cmd_count_is_at_least_thirty_plus_the_curated_two() {
    // 34 total = 19 global-live + 15 club-live. The two curated ones
    // (0x418, 0x3f3) are in there, so the "net-new" widening is 32.
    assert_eq!(HANDLED_CMDS.len(), 34);
    assert!(HANDLED_CMDS.contains(&0x418));
    assert!(HANDLED_CMDS.contains(&0x3f3));
}

/// Every advertised cmd must succeed through the same code path
/// `render_new::dispatch_and_render` uses: try `dispatch_global`, fall
/// through to `dispatch_club` on `Unhandled`, and paint every widget.
/// A failure here means the fold is broken for that screen.
#[test]
fn every_widened_cmd_populates_pool_and_renders() {
    let font = PixelFont::empty(11);
    for &cmd in HANDLED_CMDS {
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        state.pending_cmd_fallback = cmd;

        let mut result = dispatch_global(&mut pool, &mut state, -1);
        if matches!(result, DispatchResult::Unhandled) {
            result = dispatch_club(&mut pool, &mut state, -1);
        }
        let handled = matches!(
            result,
            DispatchResult::Handled
                | DispatchResult::NavBack
                | DispatchResult::NavNext
        );
        assert!(
            handled,
            "cmd 0x{cmd:x}: dispatcher refused — is_dispatch_handled is out of sync"
        );

        // The pool must carry something for the widget loop to paint.
        assert!(
            !pool.widgets.is_empty() || !pool.areas.is_empty(),
            "cmd 0x{cmd:x}: dispatcher accepted but produced an empty pool"
        );

        // Paint into a fresh surface — this is what `dispatch_and_render`
        // does. A panic here (out-of-bounds indexing, uninitialised
        // widget field) would fail the test.
        let mut packed = PackedSurface::rgb555(800, 600);
        let n = pool.widgets.len();
        for i in 0..n {
            let mut rw = to_render_widget(&pool.widgets[i]);
            rw.frame_idx = -1;
            render_widget(
                &mut packed,
                &mut rw,
                Some(&pool),
                &font,
                WidgetGlobals::default(),
                true,
            );
        }
    }
}
