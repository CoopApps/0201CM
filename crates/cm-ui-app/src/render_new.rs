//! The new-pipeline render path — Layer 2 (`packed_widget::render_widget`)
//! painting into a `PackedSurface` from a `GuiRecordPool` built by a
//! Layer-4 dispatcher (`dispatch_global` / `dispatch_club`).
//!
//! This is the fold-in commit's rendering plumbing. The old `Surface` +
//! `view_render` path in `main.rs::App::render` still owns every screen
//! it always did; for the handful of `Screen` variants that have a
//! matching dispatcher arm we now build the pool + render the widget
//! records via the byte-exact `packed_widget` pipeline, then blit the
//! resulting `PackedSurface` into the app's `Surface` for softbuffer
//! presentation. Screens without a dispatcher arm keep falling back
//! to the old renderer.
//!
//! Hit-testing is deliberately untouched — this fold is rendering only
//! (see the task doc, Part C's hard rule).
//!
//! One integration test lives in
//! `crates/cm-render/tests/dispatch_pool_render.rs`.

use cm_render::dispatcher::{dispatch_global, DispatchResult, DispatcherState};
use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::PixelFont;
use cm_render::packed_widget::{render_widget, WidgetGlobals};
use cm_render::pool_to_render::to_render_widget;
use cm_render::widget_pool::GuiRecordPool;
use cm_render::Surface;

use crate::Screen;

/// Screen variants that route through the new pipeline.
///
/// This is the mapping we can honour without fabricating widget spawns:
/// every arm here corresponds to a live `Handled` route in
/// `cm_render::dispatcher::dispatch_global`. Anything not listed falls
/// through to the old `App::render` path.
pub fn cmd_for_screen(screen: &Screen) -> Option<i16> {
    Some(match screen {
        Screen::LatestScores { .. } => 0x418,
        Screen::FifaRankings { .. } => 0x3f3,
        // Every other Screen variant either has no dispatcher-live arm
        // yet (News uses `1000` but the app already draws it via the
        // richer Layer-3 NewsView path; leave that on the old renderer
        // until the app carries the ported NewsScreenState) or has no
        // corresponding dispatcher route at all (Setup / SelectLeagues /
        // StartSeason / EnterName / SelectClub / Dashboard / LeagueTable
        // / PlayerProfile / ClubFixtures / SelectedLeagues /
        // WidgetPoolDebug).
        _ => return None,
    })
}

/// Build a widget pool by driving `dispatch_global` with `cmd` as the
/// pending-cmd fallback, then paint every widget into `packed` via
/// `render_widget`. Returns `true` when the dispatcher accepted the cmd
/// (Handled / NavBack / NavNext) — i.e. a real screen was built.
///
/// * `widget_slot = -1` matches the exe's synthetic-dispatch idiom (no
///   clicked widget); the resolver then reads `pending_cmd_fallback`.
///   See `resolve_widget_cmd` in `dispatcher.rs`.
/// * The pool is fresh on entry — dispatcher arms rebuild it from
///   scratch via their `build_screen_*` / `to_widget_pool` targets.
pub fn dispatch_and_render(cmd: i16, packed: &mut PackedSurface, font: &PixelFont) -> bool {
    let mut pool = GuiRecordPool::new();
    let mut state = DispatcherState::default();
    state.pending_cmd_fallback = cmd;
    let result = dispatch_global(&mut pool, &mut state, -1);
    let handled = matches!(
        result,
        DispatchResult::Handled | DispatchResult::NavBack | DispatchResult::NavNext
    );
    if !handled {
        return false;
    }
    // Snapshot the pool once, then iterate by index — render_widget
    // needs `&mut Widget` (block A/D/L mutate the widget), and reading
    // `pool.widgets.len()` up front avoids re-borrowing the pool during
    // the loop.
    let n = pool.widgets.len();
    for i in 0..n {
        let mut rw = to_render_widget(&pool.widgets[i]);
        // `frame_idx` in the pool copy is 0-init for descriptor-only
        // spawns (the builders don't currently thread a real area
        // index in). Force it to -1 so block G's frame overlay skips
        // rather than reading a bogus area.
        rw.frame_idx = -1;
        render_widget(packed, &mut rw, Some(&pool), font, WidgetGlobals::default(), true);
    }
    true
}

/// Blit a `PackedSurface` (555 or 565, own masks) into the app's
/// legacy RGB565 `Surface` — the bridge that lets softbuffer present
/// what the new pipeline produced without reworking the window loop.
pub fn blit_packed_to_surface(packed: &PackedSurface, out: &mut Surface) {
    let w = (packed.width as usize).min(Surface::W);
    let h = (packed.height as usize).min(Surface::H);
    let pitch = packed.pitch_pixels as usize;
    for y in 0..h {
        for x in 0..w {
            let p = packed.buf[y * pitch + x];
            let (r, g, b) = packed.unpack(p);
            out.buf[y * Surface::W + x] = cm_render::pack565(r, g, b);
        }
    }
}

/// Convenience: build+render the given `cmd` via the new pipeline and
/// blit into the app's `Surface`. Returns `true` when the dispatcher
/// handled the cmd; on `false` the caller should fall back to the old
/// per-screen render.
pub fn try_render_via_pool(cmd: i16, out: &mut Surface, font: &PixelFont) -> bool {
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    if !dispatch_and_render(cmd, &mut packed, font) {
        return false;
    }
    blit_packed_to_surface(&packed, out);
    true
}
