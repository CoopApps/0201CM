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
use cm_render::font::Fonts;
use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::PixelFont;
use cm_render::packed_panel::{draw_panel, PanelPalette, P_BEVEL, P_DARKEN, P_SOLID_FILL, P_VGRADIENT};
use cm_render::packed_text::{draw_wrapped_text, W_LEFT, W_TOP, W_WRAP};
use cm_render::packed_widget::{render_widget, WidgetGlobals};
use cm_render::pool_to_render::to_render_widget;
use cm_render::screen_leagues_faithful;
use cm_render::screen_season_faithful;
use cm_render::screen_pre_boot;
use cm_render::screen_rich_state;
use cm_render::screen_setup_faithful;
use cm_render::widget_pool::GuiRecordPool;
use cm_render::Surface;

use crate::game_state::{ManagerName, SelectLeaguesState, StartSeasonState};
use crate::Screen;

/// Everything `draw_sidebar_packed` needs to paint the persistent in-game
/// sidebar. Built by `App::render` from `world` + `game` (only present once
/// a game is loaded); pre-boot passes `None` and no sidebar is drawn.
pub struct SidebarCtx {
    pub bar: cm_domain::menu::MenuBar,
    pub open: Option<usize>,
    pub date: cm_domain::GameDate,
    pub phase: u8,
}

/// Draw a horizontal filled triangle centred in `rect` — left-pointing when
/// `left`, else right-pointing — directly into the packed surface. Byte-for-
/// byte the same span layout as `screens::draw_triangle` (real capture:
/// ~8×7 px, `hh = 4`), but writing packed `colour` via clipped horizontal
/// `draw_line` spans instead of `Surface::fill_rect`.
fn draw_triangle_packed(s: &mut PackedSurface, rect: (i32, i32, i32, i32), left: bool, colour: u16) {
    let (l, t, r, b) = rect;
    let hh = 4i32;
    let cx = (l + r) / 2;
    let cy = (t + b) / 2;
    for dy in -hh..=hh {
        let w = hh - dy.abs();
        let (x0, x1) = if left {
            let xr = cx + hh / 2;
            (xr - w, xr)
        } else {
            let xl = cx - hh / 2;
            (xl, xl + w)
        };
        // style=2 → solid horizontal span, pixels x0..=x1 at row cy+dy.
        s.draw_line(x0, cy + dy, x1, cy + dy, 2, colour);
    }
}

/// Port of `screens::menu_sidebar` onto the packed (byte-exact) pipeline.
///
/// Mirrors every draw of the `Surface` reference implementation, translated
/// to the packed primitives:
///   * old `F_VGRADIENT` (0x8) → `P_VGRADIENT`; `F_BEVEL` (0x20) → `P_BEVEL`;
///     `F_SOLID_FILL` (0x10) → `P_SOLID_FILL`; `F_TRANSPARENT` (0x2, dim-60%)
///     → `P_DARKEN` (same bit).
///   * old text `F_NOVCENTER` (0x2) → `W_TOP`; `F_LEFT` (0x1) → `W_LEFT`.
///   * colours are the same RGB triples (`palette()`), packed through the
///     surface's own `pack_rgb`.
///
/// `open` highlights the open top-level entry and draws its drop-down to the
/// right of the sidebar column.
pub fn draw_sidebar_packed(
    surface: &mut PackedSurface,
    fonts: &mut Fonts,
    bar: &cm_domain::menu::MenuBar,
    open: Option<usize>,
    date: &cm_domain::GameDate,
    phase: u8,
) {
    use crate::screens::{menu_nav_rects, menu_top_rect, MENU_DROPDOWN_W, MENU_ITEM_H, SIDEBAR};

    let pal = PanelPalette::from_palette_reload(surface);
    // palette() RGB triples, packed in the surface's own format.
    let sidebar_blue = surface.pack_rgb(0, 0, 132);
    let btn_blue = surface.pack_rgb(0, 0, 132);
    let highlight_fg = surface.pack_rgb(255, 255, 0); // yellow
    let near_white = surface.pack_rgb(231, 227, 231);
    let turquoise = surface.pack_rgb(132, 255, 255); // measured manager-identity ink
    let grey = surface.pack_rgb(132, 130, 132);
    let grey_disabled = surface.pack_rgb(120, 120, 120);

    // Sidebar column (vertical gradient).
    draw_panel(surface, SIDEBAR.0, SIDEBAR.1, SIDEBAR.2, SIDEBAR.3, P_VGRADIENT, sidebar_blue, 0, pal);

    // Date/ticker cell — F_BEVEL only (gradient shows through).
    draw_panel(surface, 0, 10, SIDEBAR.2, 56, P_BEVEL, sidebar_blue, 0, pal);
    {
        let (line1, line2) = cm_domain::sidebar_date_label(date, phase);
        let f = fonts.pixel_slot(1); // arial_narrow_10
        let mut t1 = line1.into_bytes();
        t1.push(0);
        let mut t2 = line2.into_bytes();
        t2.push(0);
        draw_wrapped_text(surface, 2, 19, SIDEBAR.2 - 2, 34, f, &t1, highlight_fg, W_TOP, -1);
        draw_wrapped_text(surface, 2, 34, SIDEBAR.2 - 2, 49, f, &t2, highlight_fg, W_TOP, -1);
    }

    // ◀ ▶ nav arrows — F_BEVEL box + yellow triangle.
    for (i, &(l, t, r, b)) in menu_nav_rects().iter().enumerate() {
        draw_panel(surface, l, t, r, b, P_BEVEL, sidebar_blue, 0, pal);
        draw_triangle_packed(surface, (l, t, r, b), i == 0, highlight_fg);
    }

    // Top-level entries.
    let last_ix = bar.menus.len().saturating_sub(1);
    for (i, top) in bar.menus.iter().enumerate() {
        let (l, t, r, b) = menu_top_rect(i);
        let is_open = open == Some(i);
        draw_panel(surface, l, t, r, b, P_BEVEL, sidebar_blue, 0, pal);
        let ink = if is_open {
            highlight_fg
        } else if i == 1 {
            turquoise
        } else if i == last_ix && top.label == "Game Options" {
            highlight_fg
        } else {
            near_white
        };
        let f = fonts.pixel_slot(2); // arial_narrow_11
        let mut txt = top.label.clone().into_bytes();
        txt.push(0);
        // style 0 + W_WRAP: horizontally + vertically centred, wrapped —
        // matches `draw_wrapped_center`.
        draw_wrapped_text(surface, l, t, r, b, f, &txt, ink, W_WRAP, -1);
    }

    // Open drop-down to the right of the sidebar column.
    if let Some(i) = open {
        if let Some(top) = bar.menus.get(i) {
            if !top.items.is_empty() {
                let (_, ty, _, _) = menu_top_rect(i);
                let x0 = SIDEBAR.2 + 1;
                let x1 = x0 + MENU_DROPDOWN_W;
                let h = top.items.len() as i32 * MENU_ITEM_H;
                let y0 = ty.min(SIDEBAR.3 - h - 4);
                draw_panel(surface, x0, y0, x1, y0 + h + 4, P_SOLID_FILL | P_BEVEL, btn_blue, 0, pal);
                let f = fonts.pixel_slot(1);
                for (j, item) in top.items.iter().enumerate() {
                    let iy = y0 + 2 + j as i32 * MENU_ITEM_H;
                    if item.separator_before && j > 0 {
                        draw_panel(surface, x0 + 4, iy, x1 - 4, iy + 1, P_DARKEN | P_BEVEL, grey, 0, pal);
                    }
                    let ink = if item.enabled { highlight_fg } else { grey_disabled };
                    let mut txt = item.label.clone().into_bytes();
                    txt.push(0);
                    draw_wrapped_text(surface, x0 + 8, iy, x1 - 6, iy + MENU_ITEM_H, f, &txt, ink, W_LEFT, -1);
                }
            }
        }
    }
}

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
pub fn try_render_via_pool(
    cmd: i16,
    out: &mut Surface,
    fonts: &mut Fonts,
    sidebar: Option<&SidebarCtx>,
) -> bool {
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    {
        let font = fonts.pixel_slot(3);
        if !dispatch_and_render(cmd, &mut packed, font) {
            return false;
        }
    }
    if let Some(ctx) = sidebar {
        draw_sidebar_packed(&mut packed, fonts, &ctx.bar, ctx.open, &ctx.date, ctx.phase);
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
    fonts: &mut Fonts,
    sidebar: Option<&SidebarCtx>,
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
    {
        let font = fonts.pixel_slot(3);
        let n = pool.widgets.len();
        for i in 0..n {
            let mut rw = to_render_widget(&pool.widgets[i]);
            rw.frame_idx = -1;
            render_widget(&mut packed, &mut rw, Some(&pool), font, WidgetGlobals::default(), true);
        }
    }
    if let Some(ctx) = sidebar {
        draw_sidebar_packed(&mut packed, fonts, &ctx.bar, ctx.open, &ctx.date, ctx.phase);
    }
    blit_packed_to_surface(&packed, out);
    true
}

/// Pre-boot fast path (Setup / SelectLeagues / StartSeason / EnterName /
/// SelectClub).
///
/// These are the 5 screens shown BEFORE the game world is initialised —
/// no dispatcher route reaches them (no menu cmd), so this function
/// pattern-matches on the `Screen` variant directly, feeds its state
/// into a [`screen_pre_boot`] builder, and paints through
/// [`render_widget`]. Same fold pattern as
/// [`try_render_rich_state`], just for the pre-game flow.
///
/// Returns `true` when a builder ran. `false` sends the caller back to
/// the old per-screen renderer — reserved for safety, no path exercises
/// it after this fold.
/// Fast path for `Screen::Setup` ONLY — the faithful direct-draw
/// renderer from `screen_setup_faithful`. Bypasses the widget pool
/// entirely: coords and colours are transcribed from the live exe
/// paint capture (fixtures/setup_screen/exe_paint.jsonl). Also blits a
/// random `.RGN` photo as the base layer to match the exe's cycling
/// backdrop.
///
/// Returns `true` when it ran. `false` sends the caller to
/// `try_render_pre_boot` for the fallback widget-pool path.
pub fn try_render_setup_faithful(
    screen: &Screen,
    out: &mut Surface,
    fonts: &mut Fonts,
    state: &screen_setup_faithful::SetupState,
) -> bool {
    if !matches!(screen, Screen::Setup) {
        return false;
    }
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    screen_setup_faithful::render_setup(&mut packed, fonts, state);
    blit_packed_to_surface(&packed, out);
    true
}

/// Fast path for `Screen::SelectLeagues` — the faithful direct-draw
/// renderer from `screen_leagues_faithful`. Same pattern as Setup: coords
/// and colours transcribed from live Frida capture
/// (`fixtures/leagues_screen/exe_paint_fb.jsonl.gz`).
pub fn try_render_leagues_faithful(
    screen: &Screen,
    out: &mut Surface,
    fonts: &mut Fonts,
    photo_seed: u64,
    has_manager: bool,
) -> bool {
    let Screen::SelectLeagues(sl) = screen else { return false };
    // Map SelectLeaguesState → LeaguesRow[] in the exe's display order.
    fn sec_label(country: &str) -> Option<&'static str> {
        match country.trim() {
            "England" => Some("Conference Division"),
            "Germany" => Some("Regional Divisions"),
            "Italy" => Some("Serie C2 A, B, C"),
            "Portugal" => Some("Segunda B"),
            "Spain" => Some("Segunda B"),
            "Sweden" => Some("Superettan"),
            _ => None,
        }
    }
    let rows: Vec<screen_leagues_faithful::LeaguesRow> = sl.order.iter()
        .filter_map(|&i| sl.slots.iter().find(|s| s.index == i))
        .map(|s| screen_leagues_faithful::LeaguesRow {
            country: &s.primary_name,
            selected: s.selected,
            background_marker: s.background_marker,
            secondary_label: sec_label(&s.primary_name),
            secondary_active: s.extra,
        })
        .collect();
    let state = screen_leagues_faithful::LeaguesState {
        photo_seed,
        has_manager,
        use_real_players: sl.options.use_real_players,
        attribute_masking: sl.options.attribute_masking,
        rows: &rows,
        scroll: sl.scroll,
        back_enabled: true,
        next_enabled: sl.selected_count() > 0,
        pressed: None,
    };
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    screen_leagues_faithful::render_leagues(&mut packed, fonts, &state);
    blit_packed_to_surface(&packed, out);
    true
}

/// Fast path for `Screen::StartSeason` — direct-draw from
/// `screen_season_faithful`. Two-column button grid on the same layout
/// as Setup; odd-tail button is centred on the row.
pub fn try_render_season_faithful(
    screen: &Screen,
    out: &mut Surface,
    fonts: &mut Fonts,
    photo_seed: u64,
    has_manager: bool,
) -> bool {
    let Screen::StartSeason { season, .. } = screen else { return false };
    let labels: Vec<&str> = season.rows.iter().map(|r| r.year_label.as_str()).collect();
    let state = screen_season_faithful::SeasonState {
        photo_seed,
        has_manager,
        rows: &labels,
        selected: season.selected,
        back_enabled: true,
        next_enabled: true,
        pressed: None,
    };
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    screen_season_faithful::render_season(&mut packed, fonts, &state);
    blit_packed_to_surface(&packed, out);
    true
}

pub fn try_render_pre_boot(
    screen: &Screen,
    manager: Option<&ManagerName>,
    out: &mut Surface,
    font: &PixelFont,
) -> bool {
    let mut pool = GuiRecordPool::new();
    let ok = match screen {
        Screen::Setup => screen_pre_boot::build_setup(&mut pool).is_some(),
        Screen::SelectLeagues(state) => build_leagues_from_app(&mut pool, state),
        Screen::StartSeason { season, .. } => build_season_from_app(&mut pool, season),
        Screen::EnterName => {
            let empty = ManagerName::default();
            let m = manager.unwrap_or(&empty);
            let fields = screen_pre_boot::NameFields {
                first: &m.first,
                second: &m.second,
                nickname: &m.nickname,
                focus: m.focus,
                is_valid: m.is_valid(),
            };
            screen_pre_boot::build_enter_name(&mut pool, &fields).is_some()
        }
        Screen::SelectClub { clubs, scroll } => build_club_from_app(&mut pool, clubs, *scroll),
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

fn build_leagues_from_app(pool: &mut GuiRecordPool, state: &SelectLeaguesState) -> bool {
    // Secondary league table — same six countries screens.rs::
    // secondary_league_label lists.
    fn sec_label(country: &str) -> Option<&'static str> {
        match country {
            "England" => Some("Conference"),
            "Germany" => Some("Regionalliga"),
            "Italy" => Some("Serie C2 A/B/C"),
            "Portugal" => Some("Segunda B"),
            "Spain" => Some("Segunda B"),
            "Sweden" => Some("Superettan"),
            _ => None,
        }
    }
    let rows: Vec<screen_pre_boot::LeaguesRow> = state
        .order
        .iter()
        .filter_map(|&i| state.slots.iter().find(|s| s.index == i))
        .map(|s| screen_pre_boot::LeaguesRow {
            country: &s.primary_name,
            selected: s.selected,
            background_marker: s.background_marker,
            secondary_label: sec_label(&s.primary_name),
            secondary_active: s.extra,
        })
        .collect();
    screen_pre_boot::build_select_leagues(
        pool,
        &rows,
        screen_pre_boot::LeaguesOptions {
            use_real_players: state.options.use_real_players,
            attribute_masking: state.options.attribute_masking,
        },
        0,
    )
    .is_some()
}

fn build_season_from_app(pool: &mut GuiRecordPool, season: &StartSeasonState) -> bool {
    let labels: Vec<String> = season.rows.iter().map(|r| r.year_label.clone()).collect();
    screen_pre_boot::build_start_season(pool, &labels, season.selected).is_some()
}

fn build_club_from_app(
    pool: &mut GuiRecordPool,
    clubs: &[cm_domain::ManagerClubChoice],
    scroll: usize,
) -> bool {
    let mut last_div = String::new();
    let rows: Vec<screen_pre_boot::ClubRow> = clubs
        .iter()
        .map(|c| {
            let div_new = c.division_name != last_div;
            last_div = c.division_name.clone();
            screen_pre_boot::ClubRow {
                club_name: &c.club_name,
                division_name: &c.division_name,
                division_new: div_new,
            }
        })
        .collect();
    screen_pre_boot::build_select_club(pool, &rows, scroll).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::{self, ManagerName, SelectLeaguesState, StartSeasonState};
    use cm_render::packed_glyph::PixelFont;

    fn font() -> PixelFont {
        // Empty pixel font — enough for widget dispatch smoke tests
        // (rects + descriptors go into the pool without needing glyph
        // bitmaps).
        PixelFont::empty(14)
    }

    #[test]
    fn dispatch_setup_populates_widgets() {
        let mut out = Surface::new();
        let f = font();
        let ok = try_render_pre_boot(&Screen::Setup, None, &mut out, &f);
        assert!(ok);
    }

    #[test]
    fn dispatch_select_leagues_populates_widgets() {
        let state = SelectLeaguesState::from_slots(game_state::real_34_slots());
        let mut out = Surface::new();
        let f = font();
        let ok = try_render_pre_boot(
            &Screen::SelectLeagues(state), None, &mut out, &f,
        );
        assert!(ok);
    }

    #[test]
    fn dispatch_start_season_populates_widgets() {
        let leagues = SelectLeaguesState::from_slots(game_state::real_34_slots());
        let season = StartSeasonState::from_leagues(&leagues);
        let screen = Screen::StartSeason { leagues, season };
        let mut out = Surface::new();
        let f = font();
        let ok = try_render_pre_boot(&screen, None, &mut out, &f);
        assert!(ok);
    }

    #[test]
    fn dispatch_enter_name_populates_widgets() {
        let mut m = ManagerName::default();
        m.first = "Alex".into();
        m.second = "Ferguson".into();
        let mut out = Surface::new();
        let f = font();
        let ok = try_render_pre_boot(&Screen::EnterName, Some(&m), &mut out, &f);
        assert!(ok);
    }

    #[test]
    fn dispatch_select_club_populates_widgets() {
        let clubs = vec![cm_domain::ManagerClubChoice {
            club_id: 1,
            club_name: "Arsenal".into(),
            division_id: 1,
            division_name: "Premier".into(),
        }];
        let screen = Screen::SelectClub { clubs, scroll: 0 };
        let mut out = Surface::new();
        let f = font();
        let ok = try_render_pre_boot(&screen, None, &mut out, &f);
        assert!(ok);
    }

    #[test]
    fn app_renders_all_screens_including_pre_boot_without_panic() {
        // Smoke: sanity that every pre-boot Screen variant reaches the
        // fast path. Rich-state + AutoRoute screens still need live
        // domain state so they stay out of this smoke, but pre-boot is
        // pure-state.
        let f = font();
        let mut out = Surface::new();
        for s in &[
            Screen::Setup,
            Screen::SelectLeagues(SelectLeaguesState::from_slots(
                game_state::real_34_slots(),
            )),
            Screen::EnterName,
            Screen::SelectClub { clubs: Vec::new(), scroll: 0 },
        ] {
            assert!(try_render_pre_boot(s, None, &mut out, &f));
        }
        let leagues = SelectLeaguesState::from_slots(game_state::real_34_slots());
        let season = StartSeasonState::from_leagues(&leagues);
        assert!(try_render_pre_boot(
            &Screen::StartSeason { leagues, season }, None, &mut out, &f,
        ));
    }

    /// ACCEPTANCE PROOF (ignored by default; run with
    /// `cargo test -p app --lib sidebar_column_diff -- --ignored --nocapture`).
    ///
    /// Render the persistent sidebar through the packed pipeline
    /// (`draw_sidebar_packed`) into a fresh 800×600 RGB555 surface, then diff
    /// the SIDEBAR COLUMN ONLY (x in 0..90, all y) against the real exe News
    /// framebuffer `fixtures/news_after.pixels.bin`. The sidebar overwrites
    /// the whole column, so the column pixels are independent of screen
    /// content — this isolates the sidebar port.
    ///
    /// Prints the differing-pixel count and the first 10 `(x,y,exp,got)`.
    /// Text pixels are EXPECTED to differ (the real save's date/manager name
    /// differ from these placeholders); the gradient / bevels / arrows should
    /// match. Requires the real `.fnt` fonts (CM_FONT_DIR or D:/cm0102/Data).
    #[test]
    #[ignore]
    fn sidebar_column_diff_vs_news_after() {
        use cm_domain::menu::{MenuBar, MenuTop};
        use cm_domain::GameDate;
        use cm_render::font::Fonts;

        // Real News-sidebar top-level entries (single human): Continue Game,
        // manager identity, Competitions, Nations & Clubs, Find, Game Options.
        let entry = |label: &str| MenuTop { label: label.into(), command: None, items: Vec::new() };
        let bar = MenuBar {
            menus: vec![
                MenuTop { label: "Continue Game".into(), command: Some(1), items: Vec::new() },
                entry("Christoph Olewicz"),
                entry("Competitions"),
                entry("Nations & Clubs"),
                entry("Find"),
                entry("Game Options"),
            ],
        };
        let date = GameDate { year: 2001, month: 10, day: 10 };
        let phase = 2; // EVE-ish placeholder

        let dir = std::env::var("CM_FONT_DIR").unwrap_or_else(|_| "D:/cm0102/Data".to_string());
        let mut fonts = Fonts::new(dir);

        let mut packed = PackedSurface::rgb555(800, 600);
        draw_sidebar_packed(&mut packed, &mut fonts, &bar, None, &date, phase);

        if let Ok(p) = std::env::var("SIDEBAR_DUMP") {
            use std::io::Write;
            let mut f = std::fs::File::create(&p).unwrap();
            for &px in &packed.buf { f.write_all(&px.to_le_bytes()).unwrap(); }
        }

        // Load the real exe framebuffer (800×600 RGB555, u16 LE).
        let fix = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/news_after.pixels.bin");
        let bytes = std::fs::read(&fix)
            .unwrap_or_else(|e| panic!("read {:?}: {}", fix, e));
        assert_eq!(bytes.len(), 800 * 600 * 2, "fixture must be 800×600 u16");
        let expected: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();

        let mut diffs = 0usize;
        let mut first: Vec<(i32, i32, u16, u16)> = Vec::new();
        for y in 0..600i32 {
            for x in 0..90i32 {
                let idx = (y * 800 + x) as usize;
                let exp = expected[idx];
                let got = packed.buf[idx];
                if exp != got {
                    diffs += 1;
                    if first.len() < 10 {
                        first.push((x, y, exp, got));
                    }
                }
            }
        }
        // Split diffs: gradient LSB-dither (|exp-got|<=1) vs structural (>1),
        // bucketed by y-band. Interpretation (measured 2026-09):
        //   * gradient 0..10 and below-entries 370..600: 0 structural diffs —
        //     the P_VGRADIENT column matches the exe exactly except for a
        //     1-LSB ordered dither the exe applies to the blue fade (a
        //     checkerboard of 0x0f/0x10) that this exact-arithmetic port does
        //     not reproduce (~12k px, all |diff|<=1).
        //   * date cell + entries: text content differs (placeholder date /
        //     manager name vs the real save) — expected per the task.
        //   * nav arrows: the ported `draw_triangle` geometry (5×9 triangle
        //     centred at y=73) is smaller/higher than the exe's ~9px triangle
        //     centred ~y=76; the bevel box + yellow ink (0x7fe0) match.
        let mut lsb = 0usize;
        let bands = [
            ("gradient 0..10", 0, 10),
            ("date cell 10..57", 10, 57),
            ("nav arrows 57..90", 57, 90),
            ("entries 90..370", 90, 370),
            ("below entries 370..600", 370, 600),
        ];
        let mut band_struct = [0usize; 5];
        for y in 0..600i32 {
            for x in 0..90i32 {
                let i = (y * 800 + x) as usize;
                let (e, g) = (expected[i] as i32, packed.buf[i] as i32);
                if e != g {
                    if (e - g).abs() <= 1 {
                        lsb += 1;
                    } else {
                        for (k, &(_, lo, hi)) in bands.iter().enumerate() {
                            if y >= lo && y < hi {
                                band_struct[k] += 1;
                            }
                        }
                    }
                }
            }
        }
        println!("sidebar-column diff: {diffs} / {} pixels differ", 90 * 600);
        for (x, y, exp, got) in &first {
            println!("  ({x},{y}) exp=0x{exp:04x} got=0x{got:04x}");
        }
        println!("  of which {lsb} are |exp-got|<=1 (gradient LSB dither)");
        println!("  structural diffs (>1 LSB) by y-band:");
        for (k, &(name, _, _)) in bands.iter().enumerate() {
            println!("    {name}: {}", band_struct[k]);
        }
        // Guard the finding that matters: the pure-gradient bands must match
        // the exe structurally (only the sub-LSB dither may differ).
        assert_eq!(band_struct[0], 0, "top gradient band must match structurally");
        assert_eq!(band_struct[4], 0, "bottom gradient band must match structurally");
    }
}
