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
use cm_render::screen_club_preview_faithful;
use cm_render::screen_name_faithful;
use cm_render::screen_nationality_faithful;
use cm_render::screen_season_faithful;
use cm_render::screen_team_faithful;
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

/// Fast path for `Screen::ClubPreview` — the club's Squad tab, opened
/// from Select Team. Includes the in-game title bar, top / bottom tab
/// bars, sub-toolbar, player list drawn from real DB data, and the
/// Take Control button.
pub fn try_render_club_preview_faithful(
    screen: &Screen,
    world: Option<&cm_domain::World>,
    out: &mut Surface,
    fonts: &mut Fonts,
    photo_seed: u64,
    has_manager: bool,
) -> bool {
    let Screen::ClubPreview { choice } = screen else { return false };
    let Some(world) = world else { return false };

    // Build the exe's "D RC" / "F LC" style position code straight
    // from the 12 aptitude bytes at type10 +0x0f..+0x1a (verified
    // from FUN_00414d5c). The category comes from the HIGHEST-scoring
    // of GK/SW/D/DM/M/AM/ST/WB; the side letters (R/L/C) then read
    // whichever of Right/Left/Central aptitudes are eligible.
    //
    //   +0x0f Goalkeeper    +0x10 Sweeper   +0x11 Defender
    //   +0x12 Def Mid       +0x13 Midfield  +0x14 Att Mid
    //   +0x15 Attacker      +0x16 Wing Back +0x17 Right Side
    //   +0x18 Left Side     +0x19 Central   +0x1a Free Role
    //
    // Aptitudes are on the game's 1..20 scale; >=15 is the exe's
    // "eligible to play here" threshold (matches position_eligibility_
    // bits' initial threshold in FUN_005a2030). Sides use the same
    // 15 bar. Everything runs from the typed struct fields, which are
    // populated during rust-db import.
    fn position_code(a: &cm_domain::DomainStaffType10) -> String {
        // Same sliding-threshold logic as FUN_005a2030's
        // `position_eligibility_bits`: start at 15, walk down until
        // something qualifies (`threshold > 9`). No player ends up
        // uncoded — the exe never shows a blank position column.
        //
        // MULTI-ROLE — corrected 2026-09-06 after Paul Robertson (Leigh
        // RMI) came through as just "WB" when his aptitudes are
        // WB=20 D=15 M=15 left=20; the exe shows him as "D/WB/M L".
        // Rule change: collect EVERY category that qualifies at the
        // highest threshold that produces >=1 hit, joined with '/',
        // in defensive-to-attacking order.
        if a.apt_goalkeeper >= 15 {
            return "GK".into();
        }
        // Display categories the exe actually paints. NB `apt_wing_back`
        // exists in the DB but is NOT a UI code — the match engine
        // consumes it internally to decide who can fill a wing-back
        // slot in a tactic; the squad screen shows those players by
        // their D and/or M aptitudes and lets the manager infer the WB
        // ability. This mirrors real CM 01/02 output.
        // (Corrected 2026-09-06 — Paul Robertson at Leigh RMI shows
        // "D/M L" in the GDI exe with WB=20; earlier code fabricated
        // a "WB" display code from the aptitude name alone.)
        let cats: [(&str, i8); 6] = [
            ("SW", a.apt_sweeper),
            ("D",  a.apt_defender),
            ("DM", a.apt_def_midfielder),
            ("M",  a.apt_midfielder),
            ("AM", a.apt_att_midfielder),
            ("F",  a.apt_attacker),
        ];
        // Walk the threshold down until AT LEAST one category qualifies.
        let mut qualifying_t: i8 = 0;
        let mut qualifying_cats: Vec<&str> = Vec::new();
        for t in (10..=15).rev() {
            if a.apt_goalkeeper >= t { return "GK".into(); }
            let hits: Vec<&str> = cats.iter()
                .filter(|(_, v)| *v >= t)
                .map(|(n, _)| *n)
                .collect();
            if !hits.is_empty() {
                qualifying_t = t;
                qualifying_cats = hits;
                break;
            }
        }
        if qualifying_cats.is_empty() {
            // Genuine rubbish — fall back to argmax as a category name
            // with no sides.
            let mut argmax = "F";
            let mut best_v = i8::MIN;
            for &(n, v) in &cats {
                if v > best_v { argmax = n; best_v = v; }
            }
            return argmax.into();
        }
        let t = qualifying_t;
        let r_eligible = a.apt_right_side >= t;
        let l_eligible = a.apt_left_side  >= t;
        let c_eligible = a.apt_central    >= t;
        let am_eligible = a.apt_att_midfielder >= t;

        // F vs S split — only applies when the SOLE qualifying category
        // is "F" (Attacker). If the player also qualifies as something
        // else, we keep "F" in the multi-role listing (a versatile
        // forward isn't a pure striker).
        if qualifying_cats == ["F"] {
            let is_striker = !am_eligible && !r_eligible && !l_eligible
                             && c_eligible;
            let head = if is_striker { "S" } else { "F" };
            let mut sides = String::new();
            if r_eligible { sides.push('R'); }
            if l_eligible { sides.push('L'); }
            if c_eligible { sides.push('C'); }
            return if sides.is_empty() { head.into() }
                   else { format!("{head} {sides}") };
        }

        // Join categories in the fixed defensive→attacking order.
        let head = qualifying_cats.join("/");
        let mut sides = String::new();
        if r_eligible { sides.push('R'); }
        if l_eligible { sides.push('L'); }
        if c_eligible { sides.push('C'); }
        if sides.is_empty() { head } else { format!("{head} {sides}") }
    }
    /// Sort order for the default Squad view — GK first, then the
    /// exe's actual 7 outfield display codes in defensive→attacking
    /// order. Multi-role players are grouped by their FIRST head code
    /// (e.g. "D/M L" sorts as D), matching the exe's grouping.
    fn position_group(code: &str) -> u8 {
        // Split off sides, then take the first "/"-separated head.
        let head_block = code.split(' ').next().unwrap_or("");
        let head = head_block.split('/').next().unwrap_or("");
        match head {
            "GK" => 0, "SW" => 1, "D" => 2, "DM" => 3,
            "M"  => 4, "AM" => 5, "F" => 6, "S" => 7,
            _ => 8,
        }
    }

    // Build "Surname, F" — the exe's row-label convention (see the
    // capture: 'Rose, M', 'Bagnall, S', ...). First-name and second-
    // name ids resolve into the first/second name pools loaded at boot.
    fn surname_initial(world: &cm_domain::World,
                       person: &cm_domain::DomainStaffType6) -> String {
        let first = world.references.first_names
            .get(person.first_name_id() as usize)
            .map(|n| n.text.as_str())
            .unwrap_or("");
        let second = world.references.second_names
            .get(person.second_name_id() as usize)
            .map(|n| n.text.as_str())
            .unwrap_or("");
        let initial = first.chars().next().unwrap_or(' ');
        if second.is_empty() {
            format!("{first}")
        } else if initial == ' ' {
            second.to_string()
        } else {
            format!("{second}, {initial}")
        }
    }

    // Build the squad list from world.staff.type6 filtered by club id
    // (the exe's genuine data source — type-6 person records with
    // body+0x35 = club id link).
    let start_day = cm_domain::day_of_year(2001, 8, 10);
    struct Row {
        name: String,
        position: String,
        age: Option<u8>,
        marker: char,
    }
    let mut rows: Vec<Row> = Vec::new();
    let attr_by_id: std::collections::BTreeMap<u32, &cm_domain::DomainStaffType10> =
        world.staff.type10.iter().map(|a| (a.id, a)).collect();
    for person in &world.staff.type6 {
        let cc = person.current_club_id();
        if cc != Some(choice.club_id) { continue; }
        let pv = cm_domain::typed_records::PlayerView::from_split(person.id, &person.body);
        if !pv.is_player() { continue; }
        let link = pv.player_data_id().map(|l| l as u32).unwrap_or(person.id);
        let pos = attr_by_id.get(&link).map(|a| position_code(a)).unwrap_or_default();
        let name = surname_initial(world, person);
        // Diagnostic prints the raw ids so we can cross-check against
        // the rust-db JSON when the render disagrees with the exe.
        eprintln!("[squad-in] person_id={} first_name_id={} second_name_id={} current_club_id={:?} name={:?}",
                   person.id, person.first_name_id(), person.second_name_id(), cc, name);
        rows.push(Row {
            name,
            position: pos,
            age: person.age_at(2001, start_day),
            marker: ' ',
        });
    }
    // Sort by position group (GK → SW → D → WB → DM → M → AM → F → S),
    // then alphabetical within each group.
    rows.sort_by(|a, b|
        position_group(&a.position).cmp(&position_group(&b.position))
            .then(a.name.cmp(&b.name)));
    // Diagnostic — helps identify wrong-club leakage the user asked
    // about (e.g. Foday/Ovie showing on Chester).
    eprintln!("[squad] {} (club_id={}): {} players",
        choice.club_name, choice.club_id, rows.len());
    for r in &rows {
        eprintln!("  {:24} {:6}  age={}",
                  r.name, r.position,
                  r.age.map(|a| format!("{a}")).unwrap_or_else(|| "?".into()));
    }

    // Owned strings kept on the stack so the renderer's borrows stay
    // valid across the render_squad call.
    let display: Vec<(String, String, Option<u8>, char)> = rows.into_iter()
        .map(|r| (r.name, r.position, r.age, r.marker))
        .collect();
    let refs: Vec<cm_render::screen_club_squad_faithful::SquadPlayer> = display.iter()
        .map(|(n, p, a, m)| cm_render::screen_club_squad_faithful::SquadPlayer {
            name: n.as_str(),
            position: p.as_str(),
            age: *a,
            marker: *m,
        })
        .collect();

    // Division SHORT name for the fourth bottom-tab label. The exe
    // shows "Prem" / "Div 1" / "Div 2" / "Div 3" / "Conference" for
    // English tiers — read directly from `club_competitions.short_name`
    // (the +0x38 field on the club_comp record) via the choice's
    // division_id. Falls back to the long name if the short field is
    // empty (defensive — every English tier ships with one).
    let short_division: String = world.references.club_competitions.iter()
        .find(|c| c.id == choice.division_id)
        .map(|c| if c.short_name.trim().is_empty() {
            c.long_name.clone()
        } else {
            c.short_name.clone()
        })
        .unwrap_or_else(|| choice.division_name.clone());

    // ---- Home-kit colours for the title bar. Look up the club record,
    //      read kit1_bg / kit1_fg colour ids, resolve them in colour.dat
    //      (`ColourView::id() == kit_id` → rgb() → pack565). Zero when
    //      any step is missing — the renderer then falls back to the
    //      in-game purple/blue defaults.
    let (kit_bg_rgb565, kit_fg_rgb565) = {
        let club_rec = world.core.clubs.iter().find(|c|
            cm_domain::typed_records::ClubView::new(c).id() == choice.club_id);
        match club_rec {
            Some(rec) => {
                let cv = cm_domain::typed_records::ClubView::new(rec);
                let resolve = |opt_id: Option<i32>| -> u16 {
                    let id = match opt_id { Some(v) if v > 0 => v as u32, _ => return 0 };
                    let hit = world.core.colours.iter().find(|c|
                        cm_domain::typed_records::ColourView::new(c).id() == id);
                    match hit {
                        Some(c) => {
                            let (r, g, b) = cm_domain::typed_records::ColourView::new(c).rgb();
                            cm_render::pack565(r, g, b)
                        }
                        None => 0,
                    }
                };
                (resolve(cv.kit1_bg_color_id()), resolve(cv.kit1_fg_color_id()))
            }
            None => (0, 0),
        }
    };
    eprintln!("[kit] {} bg=0x{:04x} fg=0x{:04x}",
              choice.club_name, kit_bg_rgb565, kit_fg_rgb565);

    let state = cm_render::screen_club_squad_faithful::SquadState {
        club_name: &choice.club_name,
        players: &refs,
        scroll: 0,
        photo_seed,
        has_manager,
        // Live division short name — Chester -> "Conference",
        // Arsenal -> "Prem", etc. Never hardcoded.
        division_name: &short_division,
        kit_bg_rgb565,
        kit_fg_rgb565,
    };
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    cm_render::screen_club_squad_faithful::render_squad(&mut packed, fonts, &state);
    blit_packed_to_surface(&packed, out);
    // Keep the import alive — screen_club_preview_faithful is retained
    // for the older placeholder variant while we're bootstrapping.
    let _ = screen_club_preview_faithful::TAKE_CONTROL_RECT;
    true
}

/// Fast path for `Screen::SelectClub` (subheader = "Select Team").
/// The club list already exists on the screen state; this function
/// enriches each row with the nation 3-letter code (looked up from
/// `world.core.nations`) and a division short code (mapped from the
/// division long name), then hands it to the faithful renderer.
pub fn try_render_team_faithful(
    screen: &Screen,
    world: Option<&cm_domain::World>,
    out: &mut Surface,
    fonts: &mut Fonts,
    photo_seed: u64,
    has_manager: bool,
) -> bool {
    let Screen::SelectClub { clubs, scroll, selected } = screen else { return false };
    let Some(world) = world else { return false };

    // Build a club_id → nation 3-letter code map ONCE for the render.
    // ClubView::nation_id gives the nation id; NationView::three_letter_name
    // gives the code.
    let nation_code_of = |club_id: u32| -> String {
        world.core.clubs.iter().find_map(|rec| {
            let cv = cm_domain::typed_records::ClubView::new(rec);
            if cv.id() == club_id {
                let nid = cv.nation_id()?;
                world.core.nations.iter()
                    .map(|n| cm_domain::typed_records::NationView::new(n))
                    .find(|nv| nv.id() as i32 == nid)
                    .map(|nv| nv.three_letter_name().to_uppercase())
            } else { None }
        }).unwrap_or_default()
    };
    // Division long name → 3-letter code. The exe uses PRM / D1 / D2 /
    // D3 / CON for England, DIVn for other simple structures. Fallback
    // to the first 3 uppercase chars of the long name.
    fn division_code(long_name: &str) -> String {
        // Table of well-known English tier long names → codes captured
        // from the exe on 2026-09-05.
        let l = long_name.to_ascii_lowercase();
        if l.contains("premier")         { return "PRM".into(); }
        if l.contains("division one")    { return "D1".into(); }
        if l.contains("division two")    { return "D2".into(); }
        if l.contains("division three")  { return "D3".into(); }
        if l.contains("conference")      { return "CON".into(); }
        // Fallback — first three ASCII-alpha letters of the long name.
        let mut out = String::new();
        for c in long_name.chars() {
            if c.is_ascii_alphabetic() { out.push(c.to_ascii_uppercase()); }
            if out.len() == 3 { break; }
        }
        out
    }

    // Look up each club's SHORT name — the exe shows "Tottenham" not
    // "Tottenham Hotspur", "Sheff Wed" not "Sheffield Wednesday". Falls
    // back to the domain-supplied long name for records that have no
    // short name set (defensive — every English club has one).
    let short_name_of = |club_id: u32, long: &str| -> String {
        world.core.clubs.iter().find_map(|rec| {
            let cv = cm_domain::typed_records::ClubView::new(rec);
            if cv.id() == club_id {
                let s = cv.secondary_name();
                if !s.trim().is_empty() { Some(s) } else { None }
            } else { None }
        }).unwrap_or_else(|| long.to_string())
    };

    // Build the enriched rows. The state's `clubs` Vec has already been
    // sorted alphabetically by short name in `App::goto_select_club`, so
    // the hit-test `clubs[visible_idx]` and this render's row order line
    // up automatically.
    let enriched: Vec<(String, String, String, u32)> = clubs.iter()
        .map(|c| {
            let short = short_name_of(c.club_id, &c.club_name);
            (
                format!("  {}", short),          // exe indents with two spaces
                nation_code_of(c.club_id),
                division_code(&c.division_name),
                c.club_id,
            )
        })
        .collect();
    let refs: Vec<screen_team_faithful::TeamRow> = enriched.iter()
        .map(|(n, nc, dc, id)| screen_team_faithful::TeamRow {
            name: n.as_str(),
            nation: nc.as_str(),
            division: dc.as_str(),
            club_id: *id,
        })
        .collect();

    let state = screen_team_faithful::TeamState {
        photo_seed,
        has_manager,
        rows: &refs,
        scroll: *scroll,
        selected: *selected,
        back_enabled: true,
        // Select Team is instant-commit — clicking a row navigates
        // straight to the club dashboard, so there's no persistent
        // "picked" state to gate Next on. The exe still renders Next
        // in bright cyan (not embossed) — mirror that by passing true;
        // the click handler treats a Next click as a no-op.
        next_enabled: true,
    };
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    screen_team_faithful::render_team(&mut packed, fonts, &state);
    blit_packed_to_surface(&packed, out);
    true
}

/// Fast path for `Screen::SelectNationality`. Reads the nationality
/// list from the loaded `World` (213 nations, `nationality_name` +
/// `actual_region` mapped to a 3-letter continent code).
pub fn try_render_nationality_faithful(
    screen: &Screen,
    world: Option<&cm_domain::World>,
    out: &mut Surface,
    fonts: &mut Fonts,
    photo_seed: u64,
    has_manager: bool,
) -> bool {
    let Screen::SelectNationality { scroll, selected, filter, filter_open } = screen
        else { return false };
    let Some(world) = world else { return false };
    // Continent id → 3-letter code — matches `rust-db/core/continents.json`
    // rows 0..5 (Africa=0, Asia=1, Europe=2, N.America=3, Oceania=4,
    // S.America=5). Anything outside 0..=5 is a data gap (some records
    // have no continent set) — render as spaces so the row stays clean.
    fn continent_code(c: i32) -> &'static str {
        match c {
            0 => "AFR", 1 => "ASI", 2 => "EUR",
            3 => "NAM", 4 => "OCE", 5 => "SAM",
            _ => "",
        }
    }
    // Build the sorted display list. `list_data[i] = (name, code, nation_id)`.
    // continent_id() reads the verified +0x71 byte (Africa=0..S.America=5).
    // The active filter selects which nations qualify — see
    // `crate::filter_nations::nation_passes` for the rules.
    let filter = *filter;
    let mut list_data: Vec<(String, &'static str, u32)> = world.core.nations.iter()
        .map(|n| cm_domain::typed_records::NationView::new(n))
        .filter(|v| crate::nation_passes(v, filter))
        .map(|v| (v.nationality_name().to_string(),
                  continent_code(v.continent_id()),
                  v.id()))
        .filter(|(n, _, _)| !n.is_empty())
        .collect();
    list_data.sort_by(|a, b| a.0.cmp(&b.0));
    // Present with the exe's two-space left-indent.
    let display: Vec<(String, &'static str, u32)> = list_data.into_iter()
        .map(|(name, cont, nid)| (format!("  {name}"), cont, nid))
        .collect();
    let refs: Vec<screen_nationality_faithful::NationalityRow> = display.iter()
        .map(|(name, cont, _)| screen_nationality_faithful::NationalityRow {
            name: name.as_str(),
            continent: cont,
        })
        .collect();
    // Map the persisted nation_id (in `selected`) to the list index.
    let selected_idx: Option<usize> = selected
        .and_then(|nid| display.iter().position(|(_, _, id)| *id == nid));
    // Dropdown options are fixed (order matches the exe capture: All
    // Nations first, Major Nations second).
    let filter_options: &[&str] = &["All Nations", "Major Nations"];
    let filter_highlight = match filter {
        crate::NationalityFilter::AllNations   => 0,
        crate::NationalityFilter::MajorNations => 1,
    };
    let state = screen_nationality_faithful::NationalityState {
        photo_seed,
        has_manager,
        rows: &refs,
        scroll: *scroll,
        selected: selected_idx,
        back_enabled: true,
        next_enabled: selected.is_some(),
        filter_label: filter.label(),
        filter_open: *filter_open,
        filter_options,
        filter_highlight,
    };
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    screen_nationality_faithful::render_nationality(&mut packed, fonts, &state);
    blit_packed_to_surface(&packed, out);
    true
}

/// Fast path for `Screen::EnterName` — direct-draw from
/// `screen_name_faithful`. Four field rows (First / Second / Password /
/// Re-Type). Re-Type dims while Password is empty.
pub fn try_render_name_faithful(
    screen: &Screen,
    manager: Option<&ManagerName>,
    out: &mut Surface,
    fonts: &mut Fonts,
    photo_seed: u64,
    has_manager: bool,
) -> bool {
    if !matches!(screen, Screen::EnterName) { return false; }
    let empty = ManagerName::default();
    let m = manager.unwrap_or(&empty);
    let state = screen_name_faithful::NameState {
        photo_seed,
        has_manager,
        first: &m.first,
        second: &m.second,
        password: &m.password,
        retype: &m.password_confirm,
        focus: m.focus,
        cancel_enabled: true,
        next_enabled: m.is_valid(),
    };
    let mut packed = PackedSurface::rgb555(Surface::W as i32, Surface::H as i32);
    screen_name_faithful::render_name(&mut packed, fonts, &state);
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
            // Fallback widget-pool path — unreachable in normal flow now
            // that `try_render_name_faithful` runs first. The old
            // NameFields struct still expects `nickname`; pass empty
            // (nickname is collected on a later screen).
            let empty = ManagerName::default();
            let m = manager.unwrap_or(&empty);
            let fields = screen_pre_boot::NameFields {
                first: &m.first,
                second: &m.second,
                nickname: "",
                focus: m.focus,
                is_valid: m.is_valid(),
            };
            screen_pre_boot::build_enter_name(&mut pool, &fields).is_some()
        }
        Screen::SelectClub { clubs, scroll, .. } => build_club_from_app(&mut pool, clubs, *scroll),
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
