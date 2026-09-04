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

use cm_render::dispatcher::{dispatch_club, dispatch_global, DispatchResult, DispatcherState};
use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::PixelFont;
use cm_render::packed_widget::{render_widget, WidgetGlobals};
use cm_render::pool_to_render::to_render_widget;
use cm_render::screen_rich_state;
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
        // `AutoRoute` is the widened-fold path — any cmd the ported
        // dispatchers can build a pool for lands here. `cmd_for_screen`
        // passes the cmd through; `dispatch_and_render` tries
        // `dispatch_global` first then `dispatch_club`.
        Screen::AutoRoute { cmd } => *cmd,
        // Rich-state screens (News / Dashboard / LeagueTable /
        // PlayerProfile / ClubFixtures / SelectedLeagues /
        // WidgetPoolDebug) still carry `View` payloads that the
        // dispatchers do not yet accept — porting `to_widget_pool`
        // impls for those views into the fast path is the next
        // widening step. Category A pre-boot (Setup / SelectLeagues /
        // StartSeason / EnterName / SelectClub) has no dispatcher
        // route at all. Both stay on the old renderer.
        _ => return None,
    })
}

/// The set of cmds that `dispatch_global` or `dispatch_club` builds
/// a pool for via one of the LIVE `build_screen_*` routes (i.e. arms
/// that return `DispatchResult::Handled` from a live builder, not a
/// `TodoBuilder` stub). Sourced from a static audit of
/// `crates/cm-render/src/dispatcher.rs` — one entry per `Handled` arm.
///
/// `dispatch_menu_command` consults this to know whether an unmapped
/// menu cmd should route to `Screen::AutoRoute` (new pipeline) or
/// fall back to the "not yet implemented" status message.
pub fn is_dispatch_handled(cmd: i16) -> bool {
    matches!(
        cmd,
        // dispatch_global LIVE routes:
        1000        // News (0x3e8)
        | 0x418     // Latest Scores
        | 0x3f2     // build_screen_698160
        | 0x3ec     // Manager History
        | 0x3ef     // Go on Holiday dialog
        | 0x414     // build_screen_7719b0
        | 0x3f3     // FIFA rankings
        | 0x40c     // UEFA Coefficients
        | 0x3f4 | 0x3f5 | 0x3f6 | 0x3f7 | 0x3f8 | 0x3f9  // shared list/table 1..6
        | 0x3fa     // build_screen_58d000
        | 0x3fc     // Player Waiting sidebar
        | 0x431     // Select Leagues
        | 0x42b     // Game Settings
        | 0x42c     // settings sub-screen
        // dispatch_club LIVE routes:
        | 0x7d1     // Club overview
        | 0x7d3     // Match report
        | 0x7d4     // Club honours
        | 0x7d7 | 0x7e7 | 0x7d8 | 0x7da | 0x7d9 | 0x7db
        | 0x7dc | 0x7dd | 0x7df | 0x7de   // Squad + Staff lists
        | 0x7e5     // Player-in-club profile
        | 0x7e6     // build_screen_46bdf0
    )
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
    let mut result = dispatch_global(&mut pool, &mut state, -1);
    // Club-context cmds (0x7d0..=0x7e7) live on `dispatch_club`; fall
    // through when `dispatch_global` reports `Unhandled` so the
    // widened fold reaches every LIVE builder in either dispatcher.
    if matches!(result, DispatchResult::Unhandled) {
        result = dispatch_club(&mut pool, &mut state, -1);
    }
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

/// Category B rich-state fast path.
///
/// The seven rich-state `Screen` variants carry live cm-domain View
/// payloads that `dispatch_global` / `dispatch_club` do not accept
/// (their builders take default views, so the AutoRoute path renders
/// empty substrates). This function pattern-matches on those variants
/// directly, feeds the live View into a
/// [`cm_render::screen_rich_state`] builder to populate a fresh
/// [`GuiRecordPool`], and paints it through [`render_widget`].
///
/// Returns `true` when a builder ran (Screen was a rich-state variant
/// AND its pool spawn didn't overflow); `false` sends the caller to the
/// old per-screen `screens::` renderer.
///
/// Provenance: every rect + colour spawned by the builders is either
/// from a live exe capture (News → `screen_news`) or from the
/// pre-fold layout in `crates/cm-ui-app/src/screens.rs` (see
/// [`cm_render::screen_rich_state`] module doc). No fabricated
/// geometry.
pub fn try_render_rich_state(
    screen: &Screen,
    out: &mut Surface,
    font: &PixelFont,
) -> bool {
    let mut pool = GuiRecordPool::new();
    let ok = match screen {
        Screen::News { view, .. } => {
            screen_rich_state::build_news_from_view(&mut pool, view).is_some()
        }
        Screen::Dashboard { view, squad_scroll } => {
            screen_rich_state::build_dashboard_from_view(&mut pool, view, *squad_scroll).is_some()
        }
        Screen::LeagueTable { view, scroll } => {
            screen_rich_state::build_league_table_from_view(&mut pool, view, *scroll).is_some()
        }
        Screen::PlayerProfile { view } => {
            screen_rich_state::build_player_profile_from_view(&mut pool, view).is_some()
        }
        Screen::ClubFixtures { view, scroll } => {
            screen_rich_state::build_club_fixtures_from_view(&mut pool, view, *scroll).is_some()
        }
        Screen::SelectedLeagues { rows, options } => {
            screen_rich_state::build_selected_leagues_from_view(
                &mut pool, rows, options.as_ref(),
            ).is_some()
        }
        Screen::WidgetPoolDebug { label, widgets } => {
            screen_rich_state::build_widget_pool_debug_from_view(
                &mut pool, label, widgets,
            ).is_some()
        }
        _ => return false,
    };
    if !ok {
        return false;
    }
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    let n = pool.widgets.len();
    for i in 0..n {
        let mut rw = to_render_widget(&pool.widgets[i]);
        rw.frame_idx = -1;
        render_widget(&mut packed, &mut rw, Some(&pool), font, WidgetGlobals::default(), true);
    }
    blit_packed_to_surface(&packed, out);
    true
}
