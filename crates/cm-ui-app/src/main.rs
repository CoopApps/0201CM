//! 0201CM window shell.
//!
//! A winit window presenting the cm-render surface (the CM0102 design system). This is the
//! bootstrap: a real, resizable, modern window drawing our own pixels — correct from the
//! first commit. GPU (wgpu) components and the screen system grow from here; the design
//! logic (colours, bevels, `.fnt` text, layout) is already lifted and lives in cm-render.

mod game_state;
mod render_new;
mod screens;

use std::num::NonZeroU32;
use std::rc::Rc;

use cm_render::font::Fonts;
use cm_render::image::Image;
use cm_render::Surface;
use game_state::{real_34_slots, SelectLeaguesState, StartSeasonState};
use screens::{LeaguesClick, SeasonClick};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

/// Which screen the app is currently showing. State that belongs to a specific screen
/// hangs off its enum variant so it's obvious what's persisted across transitions.
enum Screen {
    Setup,
    SelectLeagues(SelectLeaguesState),
    StartSeason { leagues: SelectLeaguesState, season: StartSeasonState },
    /// After initialisation: the manager enters their name. The working game
    /// (and the name being typed) lives in `App.game`, not here.
    EnterName,
    /// Select Nationality — the manager picks which nationality they are.
    /// Sits between EnterName and SelectClub in the exe's flow.
    SelectNationality {
        scroll: usize,
        selected: Option<u32>,
        /// Which subset the picker is currently showing.
        filter: NationalityFilter,
        /// `true` while the Filter dropdown is popped open.
        filter_open: bool,
    },
    /// Pick the club to manage — every playable club in the chosen country's
    /// manageable divisions. The exe subheader calls this "Select Team".
    SelectClub {
        clubs: Vec<cm_domain::ManagerClubChoice>,
        scroll: usize,
        /// club_id of the currently-picked entry (`None` until the user
        /// clicks one). Enables Next when set.
        selected: Option<u32>,
    },
    /// Club-info PREVIEW — reached from Select Team. The exe shows the
    /// full club screen (with all its tabs) but with a Take Control
    /// button in the top-right corner. Clicking Take Control installs
    /// the manager and jumps to News. The full club-tabs port is a
    /// separate follow-up; this variant carries the picked club id so
    /// the Take Control button knows what to take control of.
    ClubPreview {
        choice: cm_domain::ManagerClubChoice,
        /// Squad-list scroll offset in ENTRIES (0 = show entries
        /// 0..30). Renderer skips this many players from the sorted
        /// list; mouse-wheel handler shifts by 2 per notch (= 1 row
        /// across both columns).
        scroll: usize,
    },
    /// The News page — the game's actual home screen (the exe's news.c). This
    /// is what the manager lands on each morning.
    News {
        view: cm_domain::NewsView,
        selected: usize,
        scroll: usize,
        tab: screens::NewsTab,
    },
    /// The club Squad / Information screen (reached from the "Squad" menu item,
    /// cmd 0x7d5) — the squad overview with the player table.
    Dashboard {
        view: cm_domain::DashboardView,
        squad_scroll: usize,
    },
    /// League Table — the exe's competition dashboard table, reached from the
    /// club screen's division link. Built by `World::league_table_for` from
    /// `save.season.standings` filtered to the division's member clubs.
    LeagueTable {
        view: cm_domain::LeagueTableView,
        scroll: usize,
    },
    /// Player Profile — reached from a Dashboard squad row. Built by
    /// `World::player_profile_for` from the type-6 person + type-10 attributes.
    PlayerProfile {
        view: cm_domain::PlayerProfile,
    },
    /// Club Fixtures — reached from the Dashboard's next-fixture panel. Built
    /// by `World::club_fixtures_for` from `save.season.fixtures`.
    ClubFixtures {
        view: cm_domain::ClubFixturesView,
        scroll: usize,
    },
    /// Selected Leagues (menu cmd 0x431 → exe FUN_008053D0 in view mode):
    /// the nation tiers of the working game (`save.nation_tiers`) plus the
    /// new-game options it was started with (`save.new_game`).
    SelectedLeagues {
        rows: Vec<cm_domain::NationTierAssignment>,
        options: Option<cm_domain::NewGameOptions>,
    },
    /// Latest Scores (menu cmd 0x418 → exe FUN_00700F20). Every played
    /// fixture of the working game, most-recent first, built by
    /// `cm_domain::latest_scores` from `save.season.fixtures`.
    LatestScores {
        rows: Vec<cm_domain::LatestScoreRow>,
        scroll: usize,
    },
    /// FIFA World Rankings (menu cmd 0x3f3 → exe launcher FUN_004A2190 +
    /// body FUN_004A2200). Rows come from `save.fifa_rankings`, computed at
    /// new-game and each year rollover by `fifa_rankings::compute`.
    FifaRankings {
        view: cm_domain::screen_batch30::FifaRankingsView,
        scroll: usize,
    },
    /// Widget-pool debug view — draws the raw widget records produced by
    /// `impl RenderableView::to_widget_pool()` as coloured rectangles labelled
    /// by kind, for visual confirmation that a ported View lays out where the
    /// exe capture said it should. Reached via `CM_BOOT=news-widgets` /
    /// `CM_BOOT=dash-widgets` — no user path yet.
    WidgetPoolDebug {
        label: String,
        widgets: Vec<cm_render::widget_pool::Widget>,
    },
    /// A cmd handled by `dispatch_global` / `dispatch_club` via one of
    /// the LIVE `build_screen_*` routes (see `render_new::is_dispatch_handled`).
    /// Rendered by the new pipeline (`render_new::try_render_via_pool`).
    /// Widens the Layer 2 fold from a curated 2 screens to every cmd
    /// the ported dispatchers can build a pool for.
    AutoRoute { cmd: i16 },
}

/// The subset the Nationality picker is currently showing. Matches the
/// exe's Filter dropdown: the two options are "All Nations" (every
/// nation record with a valid name + continent) and "Major Nations"
/// (the 100-odd FIFA/UEFA-recognised entries — `state_of_development >
/// 0 && continent_id ∈ 0..=5`). Major Nations is the exe's default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NationalityFilter {
    MajorNations,
    AllNations,
}

impl NationalityFilter {
    pub fn label(self) -> &'static str {
        match self {
            NationalityFilter::MajorNations => "Major Nations",
            NationalityFilter::AllNations   => "All Nations",
        }
    }
}

/// Shared filter predicate — decides whether a nation qualifies for
/// display given the picker's current filter. Render, click and wheel
/// paths ALL call this so the visible list, hit indices, and scroll
/// range agree.
pub fn nation_passes(
    v: &cm_domain::typed_records::NationView,
    filter: NationalityFilter,
) -> bool {
    if v.nationality_name().is_empty() { return false; }
    match filter {
        NationalityFilter::MajorNations => {
            // Real FIFA nations: valid continent id AND non-zero
            // state-of-development. Drops West Germany, East Germany,
            // Soviet Union, CIS, Basque, Czechoslovakia (all devel=0
            // AND continent=0xFE).
            v.state_of_development() > 0 && (0..=5).contains(&v.continent_id())
        }
        NationalityFilter::AllNations => {
            // Loosest sensible rule — still require SOMETHING sensible
            // so we don't paint blank rows. A valid continent (0..=5)
            // is the minimum; anything else is a data artefact.
            (0..=5).contains(&v.continent_id())
        }
    }
}

/// A generic "some control is being pressed" indicator so the render pass can draw the
/// sunken state. Kept as separate cases because each screen has a different click enum.
enum Pressed {
    None,
    Setup(usize),
    Leagues(LeaguesClick),
    Season(SeasonClick),
    Name(screens::NameClick),
    Club(screens::ClubClick),
}

/// A working game instance — held in memory, never auto-written. The master
/// database (`rust-db`) is LOCKED and read-only; a new game produces one of
/// these in memory, and it only becomes a file when the user explicitly saves.
struct GameInstance {
    /// The runtime save state (tiers, schedule, player init, overlay, …). This
    /// references the immutable base by fingerprint, not by copying it.
    save: cm_domain::RuntimeSaveGame,
    /// The manager the user is creating for this game.
    manager: game_state::ManagerName,
    /// The file this game has been saved to, once the user saves. `None` until
    /// the first save.
    saved_path: Option<String>,
    /// True once the game state has advanced (a half-day tick) since the last
    /// save. The exe won't let you quit dirty without saving — we mirror that.
    dirty: bool,
}

struct App {
    window: Option<Rc<Window>>,
    context: Option<softbuffer::Context<Rc<Window>>>,
    sb: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    frame: Surface,
    fonts: Fonts,
    bg: Option<Image>,
    cursor: (i32, i32),
    pressed: Pressed,
    screen: Screen,
    /// The LOCKED master database — loaded once at startup, read-only, shared
    /// by every new game. Never modified or written back.
    world: Option<cm_domain::World>,
    /// The current in-memory working game (temporary until saved). `None` on
    /// the menu screens before a game is started.
    game: Option<GameInstance>,
    /// Which top-level menu of the persistent sidebar is open (`FUN_00745540`),
    /// or `None` when no drop-down is showing.
    menu_open: Option<usize>,
    /// Transient status line shown under the content (e.g. "not yet
    /// implemented" for menu commands without a ported screen).
    status: Option<String>,
    /// Seed for the Setup screen's rotating photo background — hashed
    /// into a photo index in `screen_setup_faithful::render_setup`. The
    /// exe picks a new one on each screen open; we do the same by
    /// bumping this whenever we return to Setup.
    setup_photo_seed: u64,
    /// Bottom-of-screen "Loading database" progress bar. `Some` while
    /// the overlay is active; the transition to `pending` fires when
    /// `progress` reaches 1.0. Matches the exe's grey bar that shows
    /// after Select Leagues → Next while the DB refinement runs.
    loading: Option<LoadingOverlay>,
}

/// State for the bottom "Loading database" progress bar.
struct LoadingOverlay {
    label:     String,
    started:   std::time::Instant,
    duration:  std::time::Duration,
    /// Next screen to transition to when progress reaches 1.0. Boxed
    /// to keep the Screen enum unbloated.
    pending:   Box<Screen>,
}

impl LoadingOverlay {
    fn progress(&self) -> f32 {
        let elapsed = self.started.elapsed().as_secs_f32();
        (elapsed / self.duration.as_secs_f32()).clamp(0.0, 1.0)
    }
    fn done(&self) -> bool { self.progress() >= 1.0 }
}

impl App {
    /// Re-render whatever screen is current into the frame.
    fn render(&mut self) {
        // New-pipeline fast path: if the current screen has a live
        // dispatcher route (see `render_new::cmd_for_screen`), build
        // the widget pool via `dispatch_global` and paint every widget
        // through the byte-exact `packed_widget` renderer. On success
        // we skip the old per-screen render entirely — the overlay
        // menu bar still runs so the sidebar stays visible.
        // The persistent in-game sidebar now draws through the SAME packed
        // pipeline as screen content (inside `try_render_*`, before the blit)
        // rather than being painted onto the old `Surface` afterwards. Build
        // its context here from `world` + `game` (only present once a game is
        // loaded); pre-boot passes `None` and draws no sidebar.
        let sidebar = match (self.world.as_ref(), self.game.as_ref()) {
            (Some(world), Some(game)) => Some(render_new::SidebarCtx {
                bar: cm_domain::menu::MenuBar::in_game(world, &game.save),
                open: self.menu_open,
                date: game.save.date.clone(),
                phase: game.save.simulation.phase,
            }),
            _ => None,
        };
        if let Some(cmd) = render_new::cmd_for_screen(&self.screen) {
            // `try_render_via_pool` pulls slot 3 (table-body arial_14) for
            // widget draws and slots 1/2 for the sidebar internally.
            if render_new::try_render_via_pool(cmd, &mut self.frame, &mut self.fonts, sidebar.as_ref()) {
                self.overlay_menu_bar();
                return;
            }
        }
        // Category B rich-state fast path (News / Dashboard / League
        // Table / Player Profile / Club Fixtures / Selected Leagues /
        // Widget Pool Debug). These carry a live cm-domain View and
        // route through `render_new::try_render_rich_state`, which
        // feeds the View into a `screen_rich_state::*` pool builder
        // and paints through `packed_widget::render_widget` — the same
        // byte-exact Layer 2 that the AutoRoute path uses.
        {
            if render_new::try_render_rich_state(
                &self.screen, &mut self.frame, &mut self.fonts, sidebar.as_ref(),
            ) {
                self.overlay_menu_bar();
                return;
            }
        }
        // Setup screen — dedicated faithful direct-draw renderer. The
        // Setup screen has too much structural detail (blue vgradient
        // sidebar, red title bar, darken-buttons over a rotating photo)
        // for the generic widget pool to reproduce pixel-accurately, so
        // this bypasses the pool and transcribes the exe's paint
        // sequence directly. See `screen_setup_faithful` for the
        // provenance and hardcoded coord table.
        {
            // Build the state parcel — Setup is top-level, so Back/Next
            // are always disabled here; `pressed` extracts the currently
            // mouse-down content button (0..8) when in the Setup arm.
            let state = cm_render::screen_setup_faithful::SetupState {
                photo_seed: self.setup_photo_seed,
                has_manager: self.game.is_some(),
                back_enabled: false,
                next_enabled: false,
                pressed: match self.pressed {
                    Pressed::Setup(i) => Some(i),
                    _ => None,
                },
            };
            if render_new::try_render_setup_faithful(
                &self.screen, &mut self.frame, &mut self.fonts, &state,
            ) {
                self.overlay_menu_bar();
                return;
            }
        }
        // Select Leagues screen — dedicated faithful direct-draw renderer,
        // same pattern as Setup. See `screen_leagues_faithful`.
        {
            let has_manager = self.game.is_some();
            if render_new::try_render_leagues_faithful(
                &self.screen, &mut self.frame, &mut self.fonts,
                self.setup_photo_seed, has_manager,
            ) {
                self.overlay_menu_bar();
                return;
            }
        }
        // Start Season — same faithful pattern (screen_season_faithful).
        {
            let has_manager = self.game.is_some();
            if render_new::try_render_season_faithful(
                &self.screen, &mut self.frame, &mut self.fonts,
                self.setup_photo_seed, has_manager,
            ) {
                self.overlay_menu_bar();
                return;
            }
        }
        // Enter Name — same faithful pattern (screen_name_faithful).
        {
            let has_manager = self.game.is_some();
            let manager = self.game.as_ref().map(|g| &g.manager);
            if render_new::try_render_name_faithful(
                &self.screen, manager, &mut self.frame, &mut self.fonts,
                self.setup_photo_seed, has_manager,
            ) {
                self.overlay_menu_bar();
                return;
            }
        }
        // Select Nationality — needs the World for the nation list.
        {
            let has_manager = self.game.is_some();
            if render_new::try_render_nationality_faithful(
                &self.screen, self.world.as_ref(),
                &mut self.frame, &mut self.fonts,
                self.setup_photo_seed, has_manager,
            ) {
                self.overlay_menu_bar();
                return;
            }
        }
        // Select Team (subheader) — the club picker; needs the World
        // for club → nation code + club record lookups.
        {
            let has_manager = self.game.is_some();
            if render_new::try_render_team_faithful(
                &self.screen, self.world.as_ref(),
                &mut self.frame, &mut self.fonts,
                self.setup_photo_seed, has_manager,
            ) {
                self.overlay_menu_bar();
                return;
            }
        }
        // Club preview — the Squad tab of the picked club with Take
        // Control button. See screen_club_squad_faithful.
        {
            let has_manager = self.game.is_some();
            if render_new::try_render_club_preview_faithful(
                &self.screen, self.world.as_ref(),
                &mut self.frame, &mut self.fonts,
                self.setup_photo_seed, has_manager,
            ) {
                self.overlay_menu_bar();
                return;
            }
        }
        // Pre-boot fast path — SelectLeagues / StartSeason /
        // EnterName / SelectClub. Closes the Layer 2 fold: EVERY screen
        // now paints through the byte-exact `packed_widget` pipeline
        // (see `screen_pre_boot` for exe fn cites per screen).
        {
            let manager = self.game.as_ref().map(|g| &g.manager);
            let font = self.fonts.pixel_slot(3);
            if render_new::try_render_pre_boot(&self.screen, manager, &mut self.frame, font) {
                // Pre-boot screens don't have an in-game menu bar, but
                // this fold keeps them plain — the overlay only paints
                // when a game is loaded (see `overlay_menu_bar`).
                self.overlay_menu_bar();
                return;
            }
        }
        match &self.screen {
            Screen::News { view, selected, scroll, tab } => {
                screens::news(&mut self.frame, &mut self.fonts, self.bg.as_ref(), view, *selected, *scroll, *tab);
                self.overlay_menu_bar();
            }
            Screen::Dashboard { view, squad_scroll } => {
                screens::dashboard(&mut self.frame, &mut self.fonts, self.bg.as_ref(), view, *squad_scroll);
                self.overlay_menu_bar();
            }
            Screen::FifaRankings { view, scroll } => {
                screens::fifa_rankings(&mut self.frame, &mut self.fonts, self.bg.as_ref(), view, *scroll);
                self.overlay_menu_bar();
            }
            Screen::LatestScores { rows, scroll } => {
                screens::latest_scores(&mut self.frame, &mut self.fonts, self.bg.as_ref(), rows, *scroll);
                self.overlay_menu_bar();
            }
            Screen::SelectedLeagues { rows, options } => {
                screens::selected_leagues(&mut self.frame, &mut self.fonts, self.bg.as_ref(), rows, options.as_ref());
                self.overlay_menu_bar();
            }
            Screen::LeagueTable { view, scroll } => {
                screens::league_table(&mut self.frame, &mut self.fonts, self.bg.as_ref(), view, *scroll);
                self.overlay_menu_bar();
            }
            Screen::PlayerProfile { view } => {
                screens::player_profile(&mut self.frame, &mut self.fonts, self.bg.as_ref(), view);
                self.overlay_menu_bar();
            }
            Screen::ClubFixtures { view, scroll } => {
                screens::club_fixtures(&mut self.frame, &mut self.fonts, self.bg.as_ref(), view, *scroll);
                self.overlay_menu_bar();
            }
            Screen::WidgetPoolDebug { label, widgets } => {
                screens::draw_widget_pool_debug(
                    &mut self.frame, &mut self.fonts, widgets, label,
                );
            }
            Screen::AutoRoute { cmd } => {
                // The new pipeline is expected to have handled this via
                // `render_new::cmd_for_screen` above. If we fell through
                // the dispatcher refused the cmd unexpectedly — show a
                // hint. The overlay menu bar still runs.
                self.status = Some(format!(
                    "AutoRoute cmd 0x{cmd:x}: dispatcher did not build a pool"
                ));
                self.overlay_menu_bar();
            }
            // Pre-boot variants (Setup / SelectLeagues / StartSeason /
            // EnterName / SelectClub) always paint via
            // `render_new::try_render_pre_boot` above, so these arms
            // are unreachable in practice. Fall back to a blank frame
            // if a bug ever routes here.
            Screen::Setup
            | Screen::SelectLeagues(_)
            | Screen::StartSeason { .. }
            | Screen::EnterName
            | Screen::SelectNationality { .. }
            | Screen::SelectClub { .. }
            | Screen::ClubPreview { .. } => {
                self.frame.fill(0, 0, 0);
                self.status = Some(
                    "pre-boot fast path refused to build a pool".into()
                );
            }
        }
    }

    /// Draw the transient status line on top of the current screen.
    ///
    /// The persistent sidebar (game_mbr / FUN_00745540) is NO LONGER painted
    /// here — it now draws through the packed pipeline inside
    /// `render_new::try_render_via_pool` / `try_render_rich_state` (see
    /// `render_new::draw_sidebar_packed`), so screen content + sidebar go out
    /// through ONE renderer instead of two. Only the status line remains on
    /// the old `Surface`.
    fn overlay_menu_bar(&mut self) {
        if let Some(msg) = self.status.clone() {
            screens::status_line(&mut self.frame, &mut self.fonts, &msg);
        }
        // Loading overlay — draws on top of whatever's already on the
        // frame, so it survives every render fast-path.
        self.draw_loading_bar();
    }

    /// Paint the bottom-of-screen "Loading database" progress bar when
    /// an overlay is active. Grey strip across the width, label on the
    /// left, blue fill growing left-to-right on the right.
    fn draw_loading_bar(&mut self) {
        let Some(overlay) = &self.loading else { return };
        let progress = overlay.progress();
        let label = overlay.label.clone();
        use cm_render::pack565;
        use cm_render::panel::{F_SOLID_FILL, F_BEVEL};
        // Re-measured from the live GDI framebuffer (RGB555):
        //   bar rect     y=555..590, x=100..790
        //   bar fill     (132,132,132) medium grey  ← was wrong (had cream)
        //   top highlight y=555  (222,222,214) cream — comes from the
        //                        panel's built-in bevel
        //   well rect    y=565..580, x=270..770  (16 px tall × 500 wide,
        //                                        centred horizontally
        //                                        in the bar's right 2/3)
        //   well bevel   SUNKEN: (41,41,41) + (90,82,82) top-left,
        //                        (173,173,173) + (222,222,214) bottom-right
        //   well fill    (132,132,132) same as bar (before progress)
        //   blue fill    (0,0,132) dark navy
        //   label ink    (231,231,231) near-white  ← was wrong (had dark)
        const Y0: i32 = 555;
        const Y1: i32 = 590;
        const X0: i32 = 100;
        const X1: i32 = 790;
        let grey_rgb     = (132u8, 132u8, 132u8);
        let label_ink    = (231u8, 231u8, 231u8);
        let navy_blue    = pack565(  0,   0, 132);
        let bevel_dark0  = pack565( 41,  41,  41);
        let bevel_dark1  = pack565( 90,  82,  82);
        let bevel_lite0  = pack565(173, 173, 173);
        let bevel_lite1  = pack565(222, 222, 214);
        // 1. Grey bar with panel-drawn bevel (top highlight, bottom
        //    shadow — matches the cream top edge seen in the capture).
        self.frame.draw_panel(X0, Y0, X1, Y1, F_SOLID_FILL | F_BEVEL, grey_rgb);
        // 2. Sunken well — 2-px double bevel around a grey interior.
        //    Top-left = DARK (sunk into the surface), bottom-right = LIGHT.
        const W_X0: i32 = 270;
        const W_X1: i32 = 770;
        const W_Y0: i32 = 565;
        const W_Y1: i32 = 580;
        // Top edge — 2 rows of increasingly dark grey.
        for x in W_X0..=W_X1 { self.frame.set(x, W_Y0,     bevel_dark0); }
        for x in W_X0..=W_X1 { self.frame.set(x, W_Y0 + 1, bevel_dark1); }
        // Bottom edge — 2 rows of light grey.
        for x in W_X0..=W_X1 { self.frame.set(x, W_Y1 - 1, bevel_lite0); }
        for x in W_X0..=W_X1 { self.frame.set(x, W_Y1,     bevel_lite1); }
        // Left / right edges: dark-left, light-right (single px each).
        for y in W_Y0..=W_Y1 { self.frame.set(W_X0, y, bevel_dark0); }
        for y in W_Y0..=W_Y1 { self.frame.set(W_X1, y, bevel_lite1); }
        // Well interior fill — same medium grey as the bar (before the
        // blue fills over it).
        let interior_y0 = W_Y0 + 2;
        let interior_y1 = W_Y1 - 2;
        let interior_x0 = W_X0 + 1;
        let interior_x1 = W_X1 - 1;
        for y in interior_y0..=interior_y1 {
            for x in interior_x0..=interior_x1 {
                self.frame.set(x, y, pack565(132, 132, 132));
            }
        }
        // 3. Blue fill inside the well interior.
        let fill_w = ((interior_x1 - interior_x0) as f32 * progress) as i32;
        for y in interior_y0..=interior_y1 {
            for x in interior_x0..(interior_x0 + fill_w).min(interior_x1) {
                self.frame.set(x, y, navy_blue);
            }
        }
        // 4. Label — near-white text on the grey bar, LEFT of the well.
        //    Vertically centred in the bar's height.
        let font = self.fonts.slot(2);
        self.frame.draw_text_box(110, Y0 + 4, 265, Y1 - 4, 0x1,
            font, label_ink, &label);
    }

    /// Progress the loading overlay: bump animation, fire the pending
    /// screen transition when it finishes. Returns true if a redraw is
    /// needed (either because it's still running or because we just
    /// transitioned). Callers should also `request_redraw()` when they
    /// see a `true` result.
    fn tick_loading(&mut self) -> bool {
        let Some(overlay) = &self.loading else { return false };
        if overlay.done() {
            let next = *self.loading.take().unwrap().pending;
            self.screen = next;
            true
        } else {
            true   // still animating — need another frame
        }
    }

    /// Compute the pressed indicator for the current screen from a cursor position.
    fn hit_test_at(&self, x: i32, y: i32) -> Pressed {
        match &self.screen {
            Screen::Setup => match screens::setup_hit(x, y) {
                Some(i) => Pressed::Setup(i),
                None => Pressed::None,
            },
            Screen::SelectLeagues(state) => match screens::leagues_hit(state, x, y) {
                Some(c) => Pressed::Leagues(c),
                None => Pressed::None,
            },
            Screen::StartSeason { season, .. } => match screens::season_hit(season, x, y) {
                Some(c) => Pressed::Season(c),
                None => Pressed::None,
            },
            Screen::EnterName => match screens::enter_name_hit(x, y) {
                Some(c) => Pressed::Name(c),
                None => Pressed::None,
            },
            // SelectClub uses its own geometry (matching the faithful
            // `screen_team_faithful` renderer); the on_release handler
            // owns the hit-test, so we always return None here and rely
            // on the in-game bypass to let the release fire.
            Screen::SelectClub { .. } => Pressed::None,
            // Nationality has no press-invert state yet — content buttons
            // are grid rows without a bevel to invert.
            Screen::SelectNationality { .. } => Pressed::None,
            // ClubPreview owns its own hit-test (Take Control button +
            // Back nav) — no per-widget press tracking.
            Screen::ClubPreview { .. } => Pressed::None,
            Screen::Dashboard { .. }
            | Screen::News { .. }
            | Screen::FifaRankings { .. }
            | Screen::LatestScores { .. }
            | Screen::SelectedLeagues { .. }
            | Screen::LeagueTable { .. }
            | Screen::PlayerProfile { .. }
            | Screen::ClubFixtures { .. }
            | Screen::WidgetPoolDebug { .. }
            | Screen::AutoRoute { .. } => Pressed::None,
        }
    }

    /// Convert a mouse-release into a screen transition + state update.
    fn on_release(&mut self, x: i32, y: i32) {
        // The persistent menu bar (sidebar) is global on in-game screens and
        // takes clicks before the screen's own controls. Handle it first.
        if matches!(
            self.screen,
            Screen::Dashboard { .. }
                | Screen::News { .. }
                | Screen::FifaRankings { .. }
                | Screen::LatestScores { .. }
                | Screen::SelectedLeagues { .. }
                | Screen::LeagueTable { .. }
                | Screen::PlayerProfile { .. }
                | Screen::ClubFixtures { .. }
                | Screen::AutoRoute { .. }
        ) {
            if let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_ref()) {
                let bar = cm_domain::menu::MenuBar::in_game(world, &game.save);
                match screens::menu_sidebar_hit(&bar, self.menu_open, x, y) {
                    Some(screens::SidebarHit::Top(i)) => {
                        // Toggle the drop-down; a direct-action top-level fires
                        // its command immediately (e.g. Continue Game).
                        if let Some(cmd) = bar.menus[i].command {
                            self.menu_open = None;
                            self.dispatch_menu_command(cmd);
                        } else {
                            self.menu_open = if self.menu_open == Some(i) { None } else { Some(i) };
                        }
                        return;
                    }
                    Some(screens::SidebarHit::Item { top, item }) => {
                        let it = &bar.menus[top].items[item];
                        let (cmd, enabled) = (it.command, it.enabled);
                        self.menu_open = None;
                        if enabled {
                            self.dispatch_menu_command(cmd);
                        }
                        return;
                    }
                    Some(screens::SidebarHit::PrevScreen)
                    | Some(screens::SidebarHit::NextScreen) => {
                        // Screen history (◀▶) is not modelled yet.
                        self.status = Some("Screen history not yet implemented".into());
                        self.menu_open = None;
                        return;
                    }
                    None => {
                        // A click elsewhere closes an open drop-down.
                        if self.menu_open.is_some() {
                            self.menu_open = None;
                            return;
                        }
                    }
                }
            }
        }
        // Deferred new-game start: the click handlers below borrow `self.screen`,
        // so they set this instead of building the game inline (which needs
        // `self.world` + `self.game`). Processed after the match releases the
        // screen borrow.
        let mut start_game: Option<(SelectLeaguesState, StartSeasonState)> = None;
        // Deferred league-table open (club id) — set by the Dashboard arm.
        let mut open_table: Option<u32> = None;
        // Deferred player-profile open (player id) — set by a Dashboard squad row.
        let mut open_profile: Option<u32> = None;
        // Deferred club-fixtures open (club id) — set by the Dashboard's fixture panel.
        let mut open_fixtures: Option<u32> = None;
        // Deferred: Enter Name -> Select Club (needs self.world + self.game).
        let mut goto_select_club = false;
        // Deferred: club picked on Select Team -> show ClubPreview.
        let mut goto_club_preview: Option<cm_domain::ManagerClubChoice> = None;
        // Deferred: ClubPreview Back -> reopen the Select Team list.
        let mut goto_reopen_select_team = false;
        // Deferred: Take Control -> install manager + Dashboard/News.
        let mut install_club: Option<cm_domain::ManagerClubChoice> = None;
        // Deferred: a News control without a ported target was clicked.
        let mut news_note = false;
        match &mut self.screen {
            Screen::Setup => {
                if let Some(idx) = screens::setup_hit(x, y) {
                    // Setup command 0 = Start New Game (mirrors the exe's cmp ax,1
                    // dispatch at 0x804ef9). Every other setup button is a no-op for now.
                    if idx == 0 {
                        // 34 picker slots from the traced LAB_0081a120..0x00821b50 setup
                        // handlers. LOW-confidence country names carry a "?" suffix so
                        // the inference is visible in the UI.
                        self.screen = Screen::SelectLeagues(
                            SelectLeaguesState::from_slots(real_34_slots()),
                        );
                    }
                }
            }
            Screen::SelectLeagues(state) => {
                if let Some(click) = screens::leagues_hit(state, x, y) {
                    match click {
                        LeaguesClick::Back => self.screen = Screen::Setup,
                        LeaguesClick::Next => {
                            // Exe gate (FUN_008070a3): the Select Start Season
                            // screen is registered ONLY when more than one
                            // nation is selected (DAT_00acdf04 > 1). With
                            // exactly one league the exe falls straight through
                            // to FUN_008120d0 ("Initialising game data").
                            //
                            //  - 0 selected → rejected (no-op; exe shows a dialog)
                            //  - 1 selected → skip Season, initialise now
                            //  - >1 selected → show Season screen (one box/league)
                            match state.selected_count() {
                                0 => {}
                                1 => {
                                    // Single league: its season is fixed, so
                                    // initialise directly (Season page skipped).
                                    let leagues = state.clone();
                                    let season = StartSeasonState::from_leagues(&leagues);
                                    start_game = Some((leagues, season));
                                }
                                _ => {
                                    // Multi-league flow — the exe shows a
                                    // "Loading database" bar while it refines
                                    // the DB down to the selected leagues.
                                    // Match that UX: queue a Loading overlay
                                    // that transitions to Start Season after
                                    // ~2.5s.
                                    let leagues = state.clone();
                                    let season = StartSeasonState::from_leagues(&leagues);
                                    self.loading = Some(LoadingOverlay {
                                        label: "Loading database".into(),
                                        started: std::time::Instant::now(),
                                        duration: std::time::Duration::from_millis(2500),
                                        pending: Box::new(Screen::StartSeason { leagues, season }),
                                    });
                                }
                            }
                        }
                        LeaguesClick::RealPlayersYes => {
                            state.options.use_real_players = true;
                        }
                        LeaguesClick::RealPlayersNo => {
                            state.options.use_real_players = false;
                            // When Real Players goes to No the exe skips the masking group;
                            // keep the flag as-is so it re-shows on toggle back to Yes.
                        }
                        LeaguesClick::MaskingYes => {
                            state.options.attribute_masking = true;
                        }
                        LeaguesClick::MaskingNo => {
                            state.options.attribute_masking = false;
                        }
                        LeaguesClick::SelectAll => state.select_all(),
                        LeaguesClick::DeselectAll => state.deselect_all(),
                        LeaguesClick::ToggleSelected(i) => state.toggle_primary(i),
                        LeaguesClick::ToggleBackground(i) => state.toggle_background_marker(i),
                        LeaguesClick::ToggleSecondary(i) => state.toggle_secondary(i),
                        LeaguesClick::ToggleHuman(i) => state.toggle_human(i),
                    }
                }
            }
            Screen::StartSeason { leagues, season } => {
                if let Some(click) = screens::season_hit(season, x, y) {
                    match click {
                        SeasonClick::Back => {
                            self.screen = Screen::SelectLeagues(leagues.clone());
                        }
                        SeasonClick::Next => {
                            // The "Start" action — initialise the game
                            // (FUN_008120d0) then advance to the manager name.
                            start_game = Some((leagues.clone(), season.clone()));
                        }
                        SeasonClick::Select(i) => season.selected = i,
                    }
                }
            }
            Screen::EnterName => {
                if let Some(click) = screens::enter_name_hit(x, y) {
                    let Some(game) = self.game.as_mut() else { return };
                    match click {
                        screens::NameClick::Field(i) => game.manager.focus = i,
                        screens::NameClick::Back => {
                            // Abandon the in-memory game and return to menu.
                            self.game = None;
                            self.screen = Screen::Setup;
                        }
                        screens::NameClick::Next => {
                            if game.manager.is_valid() {
                                // Advance to Select Nationality (the
                                // exe's screen order — Nationality sits
                                // between name and club).
                                self.screen = Screen::SelectNationality {
                                    scroll: 0,
                                    selected: None,
                                    filter: NationalityFilter::MajorNations,
                                    filter_open: false,
                                };
                            }
                        }
                    }
                }
            }
            Screen::SelectClub { clubs, scroll, selected: _ } => {
                // Select Team is INSTANT-COMMIT: clicking a club name
                // takes you straight to that club's dashboard — no
                // preview/highlight, no Next button. Matches the exe's
                // FUN_0080b2b0 handler (single click → install). Back
                // still returns to the Nationality picker.
                if y >= 555 && y <= 590 {
                    if x >= 100 && x <= 617 {
                        self.screen = Screen::SelectNationality {
                            scroll: 0, selected: None,
                            filter: NationalityFilter::MajorNations,
                            filter_open: false,
                        };
                    }
                    // Next area is dead — nothing to commit; the exe
                    // renders both nav buttons but Next is a no-op on
                    // this screen.
                } else if x >= 112 && x <= 756 && y >= 153 && y <= 527 {
                    let row = ((y - 153) / 22) as usize;
                    if row < 17 {
                        let col_left = x <= 433;
                        let visible_idx = *scroll + row * 2
                            + if col_left { 0 } else { 1 };
                        if let Some(c) = clubs.get(visible_idx) {
                            eprintln!("[team] picked {:?} -> club_id {}",
                                       c.club_name, c.club_id);
                            // Deferred: switch to ClubPreview after the
                            // match releases the &mut self.screen borrow.
                            goto_club_preview = Some(c.clone());
                        }
                    }
                }
            }
            Screen::ClubPreview { choice, .. } => {
                // Take Control button — top-right, above the title bar.
                // From screen_club_preview_faithful::TAKE_CONTROL_RECT.
                if x >= 660 && x <= 785 && y >= 4 && y <= 24 {
                    install_club = Some(choice.clone());
                } else if y >= 555 && y <= 590 && x >= 100 && x <= 617 {
                    // Back → return to Select Team. The list still has
                    // the same clubs so no reload needed.
                    goto_reopen_select_team = true;
                }
            }
            Screen::SelectNationality { scroll, selected, filter, filter_open } => {
                // The arms are mutually exclusive — an `if / else if`
                // chain instead of early `return`s so deferred flags
                // like `goto_select_club` still reach the handler at
                // the end of `on_release`.
                if *filter_open {
                    // (1) Dropdown open: rows pick a filter, anywhere
                    //     else dismisses.
                    if x >= 657 && x <= 778 && y >= 170 && y <= 188 {
                        *filter = NationalityFilter::AllNations;
                        *filter_open = false;
                        *scroll = 0;
                    } else if x >= 657 && x <= 778 && y >= 190 && y <= 208 {
                        *filter = NationalityFilter::MajorNations;
                        *filter_open = false;
                        *scroll = 0;
                    } else {
                        *filter_open = false;
                    }
                } else if x >= 655 && x <= 780 && y >= 145 && y <= 165 {
                    // (2) Filter button toggle.
                    *filter_open = true;
                } else if y >= 555 && y <= 590 {
                    // (3) Back / Next.
                    if x >= 100 && x <= 617 {
                        self.screen = Screen::EnterName;
                    } else if x >= 619 && x <= 790 && selected.is_some() {
                        if let (Some(game), Some(nid)) =
                            (self.game.as_mut(), *selected)
                        {
                            game.manager.nationality = Some(nid);
                        }
                        // Fall through so the deferred handler at the
                        // bottom of `on_release` builds the club list
                        // and switches the screen.
                        goto_select_club = true;
                    }
                } else if x >= 112 && x <= 756 && y >= 178 && y <= 527 {
                    // (4) List entries — 16 rows × 2 cols starting at y=178.
                    let row = ((y - 178) / 22) as usize;
                    if row < 16 {
                        let col_left = x <= 433;
                        let visible_idx = *scroll + row * 2
                            + if col_left { 0 } else { 1 };
                        // Rebuild the same sorted+filtered list the
                        // render uses via the shared `nation_passes`
                        // predicate — click index MUST match render.
                        if let Some(world) = self.world.as_ref() {
                            let f = *filter;
                            let mut nations: Vec<(String, u32)> = world.core.nations.iter()
                                .map(|n| cm_domain::typed_records::NationView::new(n))
                                .filter(|v| nation_passes(v, f))
                                .map(|v| (v.nationality_name(), v.id()))
                                .collect();
                            nations.sort_by(|a, b| a.0.cmp(&b.0));
                            if let Some((name, nid)) = nations.get(visible_idx) {
                                *selected = Some(*nid);
                                eprintln!("[nationality] picked {name:?} -> nation_id {nid}");
                            }
                        }
                    }
                }
            }
            Screen::News { view, selected, scroll, tab } => {
                match screens::news_hit(x, y, view, *scroll, *tab) {
                    Some(screens::NewsClick::Tab(i)) => {
                        *tab = screens::NewsTab::ALL[i];
                        *scroll = 0;
                        // Move selection to the first item of the new tab.
                        if let Some(&first) = screens::news_visible_indices(view, *tab).first() {
                            *selected = first;
                        }
                    }
                    Some(screens::NewsClick::Row(item_ix)) => {
                        if let Some(item) = view.items.get_mut(item_ix) {
                            item.unread = false;
                        }
                        *selected = item_ix;
                    }
                    Some(screens::NewsClick::BottomTab(_))
                    | Some(screens::NewsClick::Back)
                    | Some(screens::NewsClick::Next) => {
                        news_note = true;
                    }
                    None => {}
                }
            }
            Screen::Dashboard { view: cm_domain::DashboardView::Club(d), squad_scroll } => {
                // The division line under the club name is the exe's entity link
                // to the competition dashboard — open this division's table.
                let (l, t, r, b) = screens::DASH_DIVISION_LINK;
                if x >= l && x <= r && y >= t && y <= b {
                    // Deferred: `self.screen` is borrowed by this match.
                    open_table = Some(d.club_id);
                } else if screens::in_rect(x, y, screens::DASH_FIXTURE_LINK) {
                    // The next-fixture panel is the entity link to the fixture list.
                    open_fixtures = Some(d.club_id);
                } else if let Some(row) = screens::dash_squad_row_at(x, y) {
                    // A squad row is the entity link to that player's profile.
                    if let Some(p) = d.squad.get(*squad_scroll + row) {
                        open_profile = Some(p.player_id);
                    }
                }
            }
            Screen::Dashboard { .. } => {
                // Unemployed view — no non-menu controls.
            }
            Screen::FifaRankings { .. } => {
                // Read-only table; navigation is via the menu bar (handled above).
            }
            Screen::LatestScores { .. } => {
                // Read-only table; navigation is via the menu bar (handled above).
            }
            Screen::SelectedLeagues { .. } => {
                // Read-only in-game view of the setup choices; menu bar navigates.
            }
            Screen::LeagueTable { .. } => {
                // Read-only table; club rows are not entity links yet.
            }
            Screen::PlayerProfile { .. } => {
                // Read-only; navigation is via the menu bar (handled above).
            }
            Screen::ClubFixtures { .. } => {
                // Read-only list; navigation is via the menu bar (handled above).
            }
            Screen::WidgetPoolDebug { .. } => {
                // Debug view — clicks are inert.
            }
            Screen::AutoRoute { .. } => {
                // Auto-transliterated screen — no hand-authored click
                // handlers yet; the sidebar handles menu clicks above.
            }
        }
        if news_note {
            self.status = Some("That news control is not yet implemented".into());
        }
        // The screen borrow is released here — safe to build the working game.
        if let Some(club_id) = open_table {
            self.open_league_table(club_id);
        }
        if let Some(player_id) = open_profile {
            self.open_player_profile(player_id);
        }
        if let Some(club_id) = open_fixtures {
            self.open_club_fixtures(club_id);
        }
        if let Some((leagues, season)) = start_game {
            self.start_new_game(&leagues, &season);
        }
        if goto_select_club {
            self.goto_select_club();
        }
        if let Some(choice) = goto_club_preview {
            self.screen = Screen::ClubPreview { choice, scroll: 0 };
        }
        if goto_reopen_select_team {
            self.goto_select_club();
        }
        if let Some(choice) = install_club {
            self.take_control_of_club(&choice);
        }
    }

    /// Install the active game's manager at the picked club (port of the exe's
    /// Take Control → FUN_00810f50), then open the club dashboard.
    fn take_control_of_club(&mut self, choice: &cm_domain::ManagerClubChoice) {
        // Club's nation (for the tier promote) — resolved from the locked world.
        let club_nation = self.world.as_ref().and_then(|world| {
            world
                .core
                .clubs
                .iter()
                .find(|c| cm_db::ClubView::new(c).id() == choice.club_id)
                .and_then(|c| cm_db::ClubView::new(c).nation_id())
                .map(|n| n as u32)
        });
        {
            let Some(game) = self.game.as_mut() else { return };
            let identity = cm_domain::ManagerIdentity {
                first: game.manager.first.clone(),
                second: game.manager.second.clone(),
                // Nickname is collected on a later screen — Enter Name
                // now takes password + password_confirm instead.
                nickname: String::new(),
            };
            let human = game.save.add_manager(identity);
            game.save.install_manager_at_club(human, choice.club_id, club_nation);
            game.save.switch_active(human);
            eprintln!(
                "[manager] {} {} takes control of {} ({})",
                game.manager.first, game.manager.second, choice.club_name, choice.division_name
            );
        }
        // Land on the News page — the game's home screen for the new morning.
        self.open_news();
    }

    /// Build the News page (home screen) for the active human and show it.
    fn open_news(&mut self) {
        if let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_ref()) {
            let view = world.news_for(&game.save, game.save.active_human);
            let selected = 0;
            self.screen = Screen::News { view, selected, scroll: 0, tab: screens::NewsTab::All };
        }
    }

    /// Open `club_id`'s fixture list (`save.season.fixtures`).
    fn open_club_fixtures(&mut self, club_id: u32) {
        if let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_ref()) {
            match world.club_fixtures_for(&game.save, club_id) {
                Some(view) => {
                    eprintln!("[fixtures] {} — {} fixtures", view.club_name, view.rows.len());
                    self.screen = Screen::ClubFixtures { view, scroll: 0 };
                }
                None => {
                    self.status = Some("No fixtures for this club".into());
                }
            }
        }
    }

    /// Open `player_id`'s profile (type-6 person + type-10 attributes).
    fn open_player_profile(&mut self, player_id: u32) {
        if let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_ref()) {
            match world.player_profile_for(&game.save, player_id) {
                Some(view) => {
                    eprintln!("[profile] {} ({})", view.name, player_id);
                    self.screen = Screen::PlayerProfile { view };
                }
                None => {
                    self.status = Some("No attribute record for this player".into());
                }
            }
        }
    }

    /// Open the league table of the division `club_id` plays in — the exe's
    /// competition dashboard table, reached via the club screen's division
    /// link. Rows: `save.season.standings` filtered to the division.
    fn open_league_table(&mut self, club_id: u32) {
        if let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_ref()) {
            match world.league_table_for(&game.save, club_id) {
                Some(view) => {
                    eprintln!("[table] {} — {} clubs", view.competition_name, view.rows.len());
                    self.screen = Screen::LeagueTable { view, scroll: 0 };
                }
                None => {
                    self.status = Some("No league table for this club's division".into());
                }
            }
        }
    }

    /// Open the Selected Leagues view (menu cmd 0x431 → FUN_008053D0). In-game
    /// this is the read-only view of the setup choices: every nation in the
    /// working game with its tier (Foreground = selected league, Background,
    /// Neither) and detailed-match flag, plus the new-game options.
    fn open_selected_leagues(&mut self) {
        if let Some(game) = self.game.as_ref() {
            let rows = game.save.nation_tiers.clone();
            let options = game.save.new_game.clone();
            eprintln!(
                "[leagues] {} nations ({} foreground)",
                rows.len(),
                game.save.foreground_count()
            );
            self.screen = Screen::SelectedLeagues { rows, options };
        }
    }

    /// Open the Latest Scores table (menu cmd 0x418 → FUN_00700F20). Rows are
    /// every played fixture of the working game, built by `latest_scores` from
    /// `save.season.fixtures`, most-recent first, the manager's own club's
    /// results highlighted.
    fn open_latest_scores(&mut self) {
        if let Some(game) = self.game.as_ref() {
            let manager_club = game
                .save
                .humans
                .get(game.save.active_human)
                .and_then(|h| h.club);
            let rows = cm_domain::latest_scores(&game.save, manager_club);
            eprintln!("[scores] {} played fixtures", rows.len());
            self.screen = Screen::LatestScores { rows, scroll: 0 };
        }
    }

    /// Open the FIFA World Rankings table (menu cmd 0x3f3). The exe's
    /// launcher FUN_004A2190 seeds the screen fields and its body
    /// FUN_004A2200 pages the nation table — ported as
    /// `screen_batch30::build_fifa_world_rankings_screen`. The rows are the
    /// working game's `save.fifa_rankings` (computed at new-game and at each
    /// year rollover), so the table reflects THIS game, not the master DB.
    fn open_fifa_rankings(&mut self) {
        if let Some(game) = self.game.as_ref() {
            let view = screens::fifa_view_from_save(&game.save);
            eprintln!("[fifa] {} nations ranked, period {}", view.rows.len(), view.period_label);
            self.screen = Screen::FifaRankings { view, scroll: 0 };
        }
    }

    /// Route a menu-bar command (the Rust side of the exe's two dispatchers
    /// FUN_007491e0 / FUN_0074bf60). Commands with a ported screen act; the
    /// rest surface a "not yet implemented" status naming their exe target.
    fn dispatch_menu_command(&mut self, command: u16) {
        use cm_domain::menu::cmd;
        self.status = None;
        match command {
            cmd::CONTINUE => self.advance_active_day(),
            cmd::NEWS => self.open_news(),
            cmd::SQUAD | cmd::B_SQUAD => {
                // Open the club Squad / Information screen (0x7d5 -> FUN_00454620).
                if let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_ref()) {
                    if let Some(view) = world.dashboard_for(&game.save, game.save.active_human) {
                        self.screen = Screen::Dashboard { view, squad_scroll: 0 };
                    }
                }
            }
            cmd::FIFA_RANKINGS => self.open_fifa_rankings(),
            cmd::LATEST_SCORES => self.open_latest_scores(),
            cmd::SELECTED_LEAGUES => self.open_selected_leagues(),
            cmd::ADD_MANAGER => {
                // Add a new (unemployed) human — the exe's cmd 0x3fb. They join
                // the hotseat; appointment happens via Apply/Take Control later.
                if let Some(game) = self.game.as_mut() {
                    let n = game.save.humans.len();
                    let identity = cm_domain::ManagerIdentity {
                        first: "Manager".into(),
                        second: format!("{}", n + 1),
                        nickname: String::new(),
                    };
                    let h = game.save.add_manager(identity);
                    game.dirty = true;
                    self.status = Some(format!("Added manager #{} (unemployed)", h + 1));
                }
            }
            cmd::RESIGN_FROM_CLUB | cmd::RESIGN_FROM_NATION => {
                if let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_mut()) {
                    let human = game.save.active_human;
                    game.save.resign(human);
                    game.dirty = true;
                    let _ = world;
                    self.open_news();
                    self.status = Some("Resigned. You are now unemployed.".into());
                }
            }
            cmd::EXIT_GAME => {
                self.status = Some("Exit: quit the window (unsaved progress is guarded)".into());
            }
            other => {
                // If this cmd routes through the ported dispatcher
                // (`dispatch_global` or `dispatch_club`) to one of the
                // LIVE `build_screen_*` targets, hand it to the new
                // pipeline — this is what widens the Layer 2 fold from
                // a curated 2 screens to ~30+.
                if render_new::is_dispatch_handled(other as i16) {
                    self.screen = Screen::AutoRoute { cmd: other as i16 };
                } else {
                    self.status = Some(format!(
                        "'{}' not yet implemented ({})",
                        cm_domain::menu::describe_command(other),
                        format_args!("cmd 0x{other:x}"),
                    ));
                }
            }
        }
    }

    /// Continue — advance the game one calendar day (the exe's FUN_005b6a90
    /// driver, three phases per day: build+play the day's matches, dispatch the
    /// subsystems, roll the calendar via the add-days primitive FUN_00536190).
    /// Completing any part of a day dirties the game, so the quit guard now
    /// refuses to close without a save.
    fn advance_active_day(&mut self) {
        {
            let Some(game) = self.game.as_mut() else { return };
            let before = game.save.date.clone();
            game.save.tick_days(1);
            game.dirty = true;
            eprintln!(
                "[tick] {} -> {} ({} days elapsed)",
                before.iso(),
                game.save.date.iso(),
                game.save.elapsed_days
            );
        }
        // After a day advances the game shows the News page (any results /
        // events that arrived) — unless the manager is currently looking at the
        // Squad screen, which we refresh in place.
        if matches!(self.screen, Screen::Dashboard { .. }) {
            if let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_ref()) {
                if let Some(view) = world.dashboard_for(&game.save, game.save.active_human) {
                    self.screen = Screen::Dashboard { view, squad_scroll: 0 };
                }
            }
        } else {
            self.open_news();
        }
    }

    /// Build the Select Club pick list from the current game's chosen country
    /// (its manageable divisions) and advance to that screen.
    fn goto_select_club(&mut self) {
        let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_ref()) else {
            return;
        };
        let nations: Vec<String> = game
            .save
            .new_game
            .as_ref()
            .map(|o| o.selected_nations.clone())
            .unwrap_or_default();
        let mut clubs = world.manageable_clubs_for_nations(&nations);

        // The exe's Select Team screen mixes clubs from every
        // manageable division ALPHABETICALLY (Arsenal PRM, Aston Villa
        // PRM, Barnet CON, Barnsley D1, Birmingham D1 …). The upstream
        // `manageable_clubs_for_nations` groups by division; re-sort
        // the list flat here using the SHORT (secondary) name so
        // "Tottenham" sorts after "Sheff Wed" (not after "Tottenham
        // Hotspur" which would place it later in a full-name sort).
        let short_name_of = |club_id: u32, fallback: &str| -> String {
            world.core.clubs.iter().find_map(|rec| {
                let cv = cm_domain::typed_records::ClubView::new(rec);
                if cv.id() == club_id {
                    let s = cv.secondary_name();
                    if !s.trim().is_empty() { Some(s) } else { None }
                } else { None }
            }).unwrap_or_else(|| fallback.to_string())
        };
        clubs.sort_by(|a, b| {
            let an = short_name_of(a.club_id, &a.club_name);
            let bn = short_name_of(b.club_id, &b.club_name);
            an.cmp(&bn)
        });

        eprintln!(
            "[club] {} playable clubs across {}'s manageable divisions",
            clubs.len(),
            nations.join(", ")
        );
        self.screen = Screen::SelectClub { clubs, scroll: 0, selected: None };
    }

    /// Build a NEW working game IN MEMORY from the locked master database and
    /// the picker choices — the Rust counterpart of the exe's "Initialising
    /// game data" (FUN_008120d0). Nothing is written to disk: the instance is
    /// temporary until the user explicitly saves it. The master `rust-db` is
    /// never modified. Advances to the Enter Name screen.
    fn start_new_game(
        &mut self,
        leagues: &game_state::SelectLeaguesState,
        season: &game_state::StartSeasonState,
    ) {
        // Ensure the locked master database is loaded (once).
        if self.world.is_none() {
            let dir = std::env::var("CM_RUST_DB")
                .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".to_string());
            match cm_db::World::read_rust_db_dir(std::path::Path::new(&dir)) {
                Ok(w) => self.world = Some(w),
                Err(e) => {
                    eprintln!("[start] cannot open master database: {e}");
                    return;
                }
            }
        }
        let world = self.world.as_ref().unwrap();
        let options = new_game_options(leagues, season);
        let db_dir = std::env::var("CM_RUST_DB")
            .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".to_string());
        let save = world.new_game_from_rust_db(std::path::Path::new(&db_dir), &options);
        eprintln!(
            "[start] game initialised IN MEMORY: {} foreground nation(s), {} fixtures, date {}-{:02}-{:02} — not saved to disk yet",
            save.foreground_count(),
            save.season.fixtures.len(),
            save.date.year, save.date.month, save.date.day,
        );
        self.game = Some(GameInstance {
            save,
            manager: game_state::ManagerName::default(),
            saved_path: None,
            dirty: false,
        });
        self.screen = Screen::EnterName;
    }

    /// Dev shortcut (CM_BOOT=news-widgets / dash-widgets): render the raw
    /// widget-pool produced by a ported View's `to_widget_pool()` as coloured,
    /// labelled rectangles. Lets a human eyeball "yes, N widgets in the right
    /// positions" against the golden test (News = 11 for tab_count=8).
    fn boot_widget_pool_debug(&mut self, which: &str) {
        use cm_render::view_render::RenderableView;
        let (label, widgets) = match which {
            "news-widgets" => {
                // Build a NewsView. If the world+game are present we use the
                // real one (an already-installed manager); otherwise fall back
                // to a hand-built sample so the debug view works standalone.
                let view = if let (Some(world), Some(game)) = (self.world.as_ref(), self.game.as_ref()) {
                    world.news_for(&game.save, game.save.active_human)
                } else {
                    cm_domain::NewsView {
                        title: "News".into(),
                        items: vec![],
                        selected_tab: 0,
                        tab_count: 8,
                        selected_item: 0,
                        nav_back_enabled: true,
                        nav_next_enabled: false,
                    }
                };
                let widgets = view.to_widget_pool();
                ("NewsView".to_string(), widgets)
            }
            "dash-widgets" => {
                let view = cm_domain::screen_club_dashboard::ClubDashboardView::default();
                let widgets = view.to_widget_pool();
                ("ClubDashboardView".to_string(), widgets)
            }
            _ => return,
        };
        eprintln!(
            "[boot] widget-pool debug: {} produced {} widgets",
            label,
            widgets.len()
        );
        self.screen = Screen::WidgetPoolDebug { label, widgets };
    }

    /// Dev shortcut (CM_BOOT=dashboard): skip the setup flow — build an England
    /// game, install a default manager at Arsenal, and open the dashboard.
    fn boot_dashboard(&mut self) {
        let Some(world) = self.world.as_ref() else { return };
        let db_dir =
            std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".to_string());
        let options = cm_domain::NewGameOptions {
            selected_nations: vec!["England".into()],
            background_nations: vec![],
            use_real_players: true,
            attribute_masking: true,
            start_year: 2001,
        };
        let mut save = world.new_game_from_rust_db(std::path::Path::new(&db_dir), &options);
        let identity = cm_domain::ManagerIdentity {
            first: "Alex".into(),
            second: "Ferguson".into(),
            nickname: "Fergie".into(),
        };
        let h = save.add_manager(identity);
        // Arsenal (club id 676); nation England (60).
        let club_nation = world.club_name(676).and(Some(60u32));
        save.install_manager_at_club(h, 676, club_nation);
        save.switch_active(h);
        let mut manager = game_state::ManagerName::default();
        manager.first = "Alex".into();
        manager.second = "Ferguson".into();
        // Password fields default empty (no save protection).
        self.game = Some(GameInstance { save, manager, saved_path: None, dirty: false });
        self.open_news();
        eprintln!("[boot] jumped straight to the News page (Arsenal / Fergie)");
    }

    /// Keyboard input — only the Enter Name screen consumes it (typing into the
    /// focused field of the working game's manager).
    fn key_input(&mut self, ch: Option<char>, named: Option<NamedKeyAction>) {
        if matches!(self.screen, Screen::EnterName) {
            if let Some(game) = self.game.as_mut() {
                match named {
                    Some(NamedKeyAction::Backspace) => game.manager.backspace(),
                    Some(NamedKeyAction::Tab) => {
                        game.manager.focus = (game.manager.focus + 1) % 3
                    }
                    _ => {
                        if let Some(c) = ch {
                            game.manager.type_char(c);
                        }
                    }
                }
            }
        } else if matches!(self.screen, Screen::Dashboard { .. }) {
            // Continue = advance one day (Enter or Space, matching the exe's
            // Continue button and spacebar shortcut).
            if matches!(named, Some(NamedKeyAction::Continue)) || ch == Some(' ') {
                self.advance_active_day();
            }
        }
    }
}

/// Named (non-character) keys the app reacts to outside plain text entry.
#[derive(Debug, Clone, Copy)]
enum NamedKeyAction {
    Backspace,
    Tab,
    Continue,
}

/// Build the picker choices into NewGameOptions (the exe's accumulated
/// Select-League(s)/Season inputs). Reading only — the master DB is untouched.
fn new_game_options(
    leagues: &game_state::SelectLeaguesState,
    _season: &game_state::StartSeasonState,
) -> cm_domain::NewGameOptions {
    cm_domain::NewGameOptions {
        selected_nations: leagues.slots.iter().filter(|s| s.selected)
            .map(|s| s.primary_name.clone()).collect(),
        background_nations: leagues.slots.iter().filter(|s| s.background_marker)
            .map(|s| s.primary_name.clone()).collect(),
        use_real_players: leagues.options.use_real_players,
        attribute_masking: leagues.options.attribute_masking,
        start_year: 2001,
    }
}

/// Load the first Pictures/*.RGN as a menu background (the original randomises it).
fn load_background() -> Option<Image> {
    let dir = std::env::var("CM_PICTURES_DIR").unwrap_or_else(|_| "D:/cm0102/Pictures".to_string());
    let mut entries: Vec<_> = std::fs::read_dir(&dir).ok()?.flatten().map(|e| e.path()).collect();
    entries.sort();
    let rgn = entries.into_iter().find(|p| {
        p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("rgn")).unwrap_or(false)
    })?;
    Image::load_rgn(&rgn).ok()
}

impl Default for App {
    fn default() -> Self {
        let dir = std::env::var("CM_FONT_DIR").unwrap_or_else(|_| "D:/cm0102/Data".to_string());
        Self {
            window: None,
            context: None,
            sb: None,
            frame: Surface::new(),
            fonts: Fonts::new(dir),
            bg: load_background(),
            cursor: (0, 0),
            pressed: Pressed::None,
            screen: Screen::Setup,
            world: None,
            game: None,
            menu_open: None,
            status: None,
            setup_photo_seed: {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now().duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0)
            },
            loading: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("0201CM")
            .with_inner_size(LogicalSize::new(Surface::W as u32, Surface::H as u32))
            .with_resizable(true);
        let window = Rc::new(el.create_window(attrs).unwrap());
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let sb = softbuffer::Surface::new(&context, window.clone()).unwrap();
        self.render();
        window.request_redraw();
        self.window = Some(window);
        self.context = Some(context);
        self.sb = Some(sb);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                // The exe won't let you quit a game that has advanced (a
                // half-day ticked) without saving. Mirror that: block the
                // close while the working game is dirty. (Once a Save Game
                // screen exists, this will prompt to save; for now it refuses
                // and logs.)
                if self.game.as_ref().map(|g| g.dirty).unwrap_or(false) {
                    eprintln!("[quit] game has unsaved progress — save before quitting (close refused)");
                } else {
                    el.exit();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as i32, position.y as i32);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => (p.y / 20.0) as f32,
                };
                let mut changed = false;
                match &mut self.screen {
                    Screen::SelectClub { clubs, scroll, .. } => {
                        // 2-col grid × 17 rows = 34 entries per screen.
                        // Wheel steps 2 entries (one row) at a time to
                        // match the faithful renderer's row-major layout.
                        const VISIBLE_ENTRIES: usize = 34;
                        let max = clubs.len().saturating_sub(VISIBLE_ENTRIES);
                        *scroll = if dy > 0.0 {
                            scroll.saturating_sub(2)
                        } else {
                            (*scroll + 2).min(max)
                        };
                        changed = true;
                    }
                    Screen::Dashboard { view: cm_domain::DashboardView::Club(d), squad_scroll } => {
                        let max = d.squad.len().saturating_sub(screens::DASH_SQUAD_ROWS);
                        *squad_scroll = if dy > 0.0 { squad_scroll.saturating_sub(1) } else { (*squad_scroll + 1).min(max) };
                        changed = true;
                    }
                    Screen::ClubPreview { choice, scroll } => {
                        // Squad screen: 2 cols × 14 rows = 28 entries
                        // visible (matches screen_club_squad_faithful::
                        // VISIBLE_ENTRIES). Renderer uses scroll as an
                        // ENTRY skip count, so we shift by 2 per wheel
                        // notch (one row across both columns).
                        const VISIBLE_ENTRIES: usize = 28;
                        let total = self.world.as_ref().map(|w| {
                            w.staff.type6.iter()
                                .filter(|p| p.current_club_id() == Some(choice.club_id))
                                .filter(|p| cm_domain::typed_records::PlayerView::from_split(
                                    p.id, &p.body).is_player())
                                .count()
                        }).unwrap_or(0);
                        let max = total.saturating_sub(VISIBLE_ENTRIES);
                        *scroll = if dy > 0.0 { scroll.saturating_sub(2) }
                                  else        { (*scroll + 2).min(max) };
                        changed = true;
                    }
                    Screen::FifaRankings { view, scroll } => {
                        let max = view.rows.len().saturating_sub(screens::FIFA_ROWS_VISIBLE);
                        *scroll = if dy > 0.0 { scroll.saturating_sub(1) } else { (*scroll + 1).min(max) };
                        changed = true;
                    }
                    Screen::LatestScores { rows, scroll } => {
                        let max = rows.len().saturating_sub(screens::LATEST_SCORES_ROWS_VISIBLE);
                        *scroll = if dy > 0.0 { scroll.saturating_sub(1) } else { (*scroll + 1).min(max) };
                        changed = true;
                    }
                    Screen::LeagueTable { view, scroll } => {
                        let max = view.rows.len().saturating_sub(screens::LEAGUE_ROWS_VISIBLE);
                        *scroll = if dy > 0.0 { scroll.saturating_sub(1) } else { (*scroll + 1).min(max) };
                        changed = true;
                    }
                    Screen::ClubFixtures { view, scroll } => {
                        let max = view.rows.len().saturating_sub(screens::FIXTURE_ROWS_VISIBLE);
                        *scroll = if dy > 0.0 { scroll.saturating_sub(1) } else { (*scroll + 1).min(max) };
                        changed = true;
                    }
                    Screen::SelectLeagues(state) => {
                        // 34 total, 16 visible → max scroll = 18.
                        // Match the same-signed convention above (wheel-up
                        // = show earlier rows).
                        const VISIBLE: usize = 16;
                        let max = state.slots.len().saturating_sub(VISIBLE);
                        state.scroll = if dy > 0.0 {
                            state.scroll.saturating_sub(1)
                        } else {
                            (state.scroll + 1).min(max)
                        };
                        changed = true;
                    }
                    Screen::SelectNationality { scroll, filter, .. } => {
                        // 2-col grid × 16 rows = 32 entries per screen.
                        // Wheel steps 2 entries (one row) at a time.
                        // Count against the CURRENTLY-FILTERED list so
                        // scroll-max lines up with whatever the render
                        // is showing.
                        const VISIBLE_ENTRIES: usize = 32;
                        let f = *filter;
                        let total = self.world.as_ref().map(|w| {
                            w.core.nations.iter()
                                .map(|n| cm_domain::typed_records::NationView::new(n))
                                .filter(|v| nation_passes(v, f))
                                .count()
                        }).unwrap_or(0);
                        let max = total.saturating_sub(VISIBLE_ENTRIES);
                        *scroll = if dy > 0.0 {
                            scroll.saturating_sub(2)
                        } else {
                            (*scroll + 2).min(max)
                        };
                        changed = true;
                    }
                    _ => {}
                }
                if changed {
                    self.render();
                    if let Some(w) = self.window.as_ref() {
                        w.request_redraw();
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. }
                if button == winit::event::MouseButton::Left =>
            {
                match state {
                    winit::event::ElementState::Pressed => {
                        self.pressed = self.hit_test_at(self.cursor.0, self.cursor.1);
                    }
                    winit::event::ElementState::Released => {
                        // Only fire the action if release lands on the same target as press;
                        // this matches the classic click semantics (press-and-drag-off cancels).
                        let release_hit = self.hit_test_at(self.cursor.0, self.cursor.1);
                        let same = match (&self.pressed, &release_hit) {
                            (Pressed::None, _) => false,
                            (Pressed::Setup(a), Pressed::Setup(b)) => a == b,
                            (Pressed::Leagues(a), Pressed::Leagues(b)) => a == b,
                            (Pressed::Season(a), Pressed::Season(b)) => a == b,
                            (Pressed::Name(a), Pressed::Name(b)) => a == b,
                            (Pressed::Club(a), Pressed::Club(b)) => a == b,
                            _ => false,
                        };
                        // In-game screens (News / Dashboard) hit-test internally
                        // via their own click enums rather than the Pressed
                        // mechanism, so they always process a release.
                        // SelectNationality also owns its own hit-testing (list
                        // rows + Filter dropdown + Back/Next), so it goes here.
                        let in_game = matches!(
                            self.screen,
                            Screen::Dashboard { .. }
                                | Screen::News { .. }
                                | Screen::FifaRankings { .. }
                                | Screen::LatestScores { .. }
                                | Screen::SelectedLeagues { .. }
                                | Screen::LeagueTable { .. }
                                | Screen::PlayerProfile { .. }
                                | Screen::ClubFixtures { .. }
                                | Screen::AutoRoute { .. }
                                | Screen::SelectNationality { .. }
                                | Screen::SelectClub { .. }
                                | Screen::ClubPreview { .. }
                        );
                        if same || in_game {
                            self.on_release(self.cursor.0, self.cursor.1);
                        }
                        self.pressed = Pressed::None;
                    }
                }
                self.render();
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event: key_event, .. }
                if key_event.state == winit::event::ElementState::Pressed =>
            {
                use winit::keyboard::{Key, NamedKey};
                let named = match &key_event.logical_key {
                    Key::Named(NamedKey::Backspace) => Some(NamedKeyAction::Backspace),
                    Key::Named(NamedKey::Tab) => Some(NamedKeyAction::Tab),
                    Key::Named(NamedKey::Enter) => Some(NamedKeyAction::Continue),
                    _ => None,
                };
                // Character text (letters, space, punctuation) from the key.
                let ch = key_event
                    .text
                    .as_ref()
                    .and_then(|t| t.chars().next())
                    .filter(|c| !c.is_control());
                self.key_input(ch, named);
                self.render();
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                // Loading overlay animation — tick before the render so
                // the progress bar's frame reflects the CURRENT elapsed
                // time. If the tick fires a screen transition we re-
                // render the new screen straight away.
                let loading_active = self.loading.is_some();
                if loading_active {
                    self.tick_loading();   // may consume the overlay
                    self.render();
                }
                let (Some(window), Some(sb)) = (self.window.as_ref(), self.sb.as_mut()) else {
                    return;
                };
                let size = window.inner_size();
                let (ww, wh) = (size.width.max(1), size.height.max(1));
                sb.resize(NonZeroU32::new(ww).unwrap(), NonZeroU32::new(wh).unwrap()).unwrap();
                let mut buffer = sb.buffer_mut().unwrap();
                for y in 0..(wh as usize).min(Surface::H) {
                    for x in 0..(ww as usize).min(Surface::W) {
                        let (r, g, b) = cm_render::unpack565(self.frame.buf[y * Surface::W + x]);
                        buffer[y * ww as usize + x] = (r as u32) << 16 | (g as u32) << 8 | b as u32;
                    }
                }
                buffer.present().unwrap();
                // Keep the animation running — request another redraw
                // while the overlay is still active.
                if self.loading.is_some() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

/// Headless render — for calibration / verifying a screen without opening a window.
/// `cargo run -p app -- --dump <path> [setup|leagues|season|spec:<va>]`
/// The `spec:<va>` variant loads `analysis/screens/<va>.json` and renders it
/// directly via cm-widget (bypassing the app's hand-wired screen enum).
fn dump(path: &str, which: &str) {
    let dir = std::env::var("CM_FONT_DIR").unwrap_or_else(|_| "D:/cm0102/Data".to_string());
    let mut frame = Surface::new();
    let mut fonts = Fonts::new(dir);
    let bg = load_background();
    if let Some(va_str) = which.strip_prefix("spec:") {
        let va = u32::from_str_radix(va_str.trim_start_matches("0x"), 16).unwrap_or(0);
        // Load the spec directly and render — no app-side screen enum needed.
        let spec_path = std::env::var("CM_SCREENS_DIR")
            .unwrap_or_else(|_| "D:/cm0102-carve/analysis/screens".to_string());
        let file = std::path::PathBuf::from(spec_path).join(format!("{va:08x}.json"));
        match cm_widget::ScreenSpec::load(&file) {
            Ok(spec) => {
                // Screen-specific providers here. Add more as we grow screens
                // that need runtime substitution (scratch buffers, row-count,
                // per-row column text). The picker screen (0x8055e0) needs a
                // country-name provider — else the row-0 template renders the
                // last strcpy'd scratch value ("De-Select All") for every row.
                if va == 0x8055e0 {
                    // Initial state: everything unselected. Defaults are
                    // RP=Yes and AM=Yes. To demonstrate the RP=No path
                    // (which hides the Attribute Masking group via
                    // cell_hidden), respond to CM_DEMO_STATE:
                    //   rp-no    → RP toggled to No, masking group vanishes
                    //   sel3     → 3 leagues seeded selected
                    //   bg2      → 2 rows with BACKGROUND clicked
                    // Multiple hints comma-separated.
                    let mut state = SelectLeaguesState::from_slots(real_34_slots());
                    for hint in std::env::var("CM_DEMO_STATE").unwrap_or_default().split(',') {
                        match hint.trim() {
                            "rp-no" => state.options.use_real_players = false,
                            "am-no" => state.options.attribute_masking = false,
                            "sel3" => {
                                for name in ["England", "Germany", "Italy"] {
                                    if let Some(s) = state.slots.iter_mut()
                                        .find(|s| s.primary_name == name) { s.selected = true; }
                                }
                            }
                            "bg2" => {
                                for name in ["Denmark", "France"] {
                                    if let Some(s) = state.slots.iter_mut()
                                        .find(|s| s.primary_name == name) { s.background_marker = true; }
                                }
                            }
                            _ => {}
                        }
                    }
                    let provider = screens::LeaguesProvider::new(state);
                    spec.render(
                        &mut frame,
                        &mut fonts,
                        bg.as_ref(),
                        &cm_widget::Palette::default(),
                        &provider,
                    );
                } else if va == 0x807280 {
                    // Select Start Season — labels_ptr is runtime-built; the
                    // SeasonProvider supplies one label per selected slot.
                    // For the dump, seed the five leagues from the user's
                    // reference screenshot so the multi-league path is
                    // exercised (Denmark, England, Finland, France, Germany).
                    let mut leagues = SelectLeaguesState::from_slots(real_34_slots());
                    let seeded = ["Denmark", "England", "Finland", "France", "Germany"];
                    for slot in leagues.slots.iter_mut() {
                        if seeded.contains(&slot.primary_name.as_str()) {
                            slot.selected = true;
                        }
                    }
                    let provider = screens::SeasonProvider::from_leagues(leagues);
                    spec.render(
                        &mut frame,
                        &mut fonts,
                        bg.as_ref(),
                        &cm_widget::Palette::default(),
                        &provider,
                    );
                } else {
                    spec.render(
                        &mut frame,
                        &mut fonts,
                        bg.as_ref(),
                        &cm_widget::Palette::default(),
                        &cm_widget::NullProvider,
                    );
                }
            }
            Err(e) => {
                eprintln!("[dump] failed to load spec {}: {e}", file.display());
                frame.fill(40, 0, 0);
            }
        }
    } else {
        match which {
            "leagues" => {
                let state = SelectLeaguesState::from_slots(real_34_slots());
                screens::select_leagues(&mut frame, &mut fonts, bg.as_ref(), &state, None);
            }
            "season" => {
                let state = StartSeasonState::default();
                screens::start_season(&mut frame, &mut fonts, bg.as_ref(), &state, None);
            }
            "name" => {
                let manager = game_state::ManagerName {
                    first: "Alex".into(),
                    second: "Ferguson".into(),
                    password: String::new(),
                    password_confirm: String::new(),
                    focus: 1,
                    nationality: None,
                };
                screens::enter_name(&mut frame, &mut fonts, bg.as_ref(), &manager);
            }
            "club" => {
                let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
                let clubs = cm_db::World::read_rust_db_dir(std::path::Path::new(&dir))
                    .map(|w| w.manageable_clubs_for_nations(&["England".to_string()]))
                    .unwrap_or_default();
                screens::select_club(&mut frame, &mut fonts, bg.as_ref(), &clubs, 0);
            }
            "dashboard" => {
                let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
                if let Ok(world) = cm_db::World::read_rust_db_dir(std::path::Path::new(&dir)) {
                    let opts = cm_domain::NewGameOptions {
                        selected_nations: vec!["England".into()],
                        background_nations: vec![],
                        use_real_players: true,
                        attribute_masking: true,
                        start_year: 2001,
                    };
                    let mut save = world.new_game_from_rust_db(std::path::Path::new(&dir), &opts);
                    let h = save.add_manager(cm_domain::ManagerIdentity {
                        first: "Alex".into(), second: "Ferguson".into(), nickname: "Fergie".into(),
                    });
                    save.install_manager_at_club(h, 676, Some(60)); // Arsenal
                    save.switch_active(h);
                    // Optional: advance N days to verify the tick (CM_ADV_DAYS).
                    if let Ok(n) = std::env::var("CM_ADV_DAYS").unwrap_or_default().parse::<u32>() {
                        save.tick_days(n);
                        eprintln!("[dump] advanced {n} days -> {}", save.date.iso());
                    }
                    if let Some(view) = world.dashboard_for(&save, h) {
                        screens::dashboard(&mut frame, &mut fonts, bg.as_ref(), &view, 0);
                        let bar = cm_domain::menu::MenuBar::in_game(&world, &save);
                        // Open a drop-down for the screenshot if CM_MENU_OPEN=N.
                        let open = std::env::var("CM_MENU_OPEN").ok().and_then(|v| v.parse::<usize>().ok());
                        screens::menu_sidebar(&mut frame, &mut fonts, &bar, open, &save.date, save.simulation.phase);
                    }
                }
            }
            "fifa" => {
                // Headless render of the FIFA World Rankings table for the same
                // England / Arsenal test game the other dumps use.
                let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
                if let Ok(world) = cm_db::World::read_rust_db_dir(std::path::Path::new(&dir)) {
                    let opts = cm_domain::NewGameOptions {
                        selected_nations: vec!["England".into()],
                        background_nations: vec![], use_real_players: true,
                        attribute_masking: true, start_year: 2001,
                    };
                    let mut save = world.new_game_from_rust_db(std::path::Path::new(&dir), &opts);
                    let h = save.add_manager(cm_domain::ManagerIdentity {
                        first: "Alex".into(), second: "Ferguson".into(), nickname: "Fergie".into(),
                    });
                    save.install_manager_at_club(h, 676, Some(60));
                    save.switch_active(h);
                    let view = screens::fifa_view_from_save(&save);
                    let scroll = std::env::var("CM_SCROLL").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
                    eprintln!("[dump] fifa: {} nations ranked, period {}", view.rows.len(), view.period_label);
                    screens::fifa_rankings(&mut frame, &mut fonts, bg.as_ref(), &view, scroll);
                    let bar = cm_domain::menu::MenuBar::in_game(&world, &save);
                    let open = std::env::var("CM_MENU_OPEN").ok().and_then(|v| v.parse::<usize>().ok());
                    screens::menu_sidebar(&mut frame, &mut fonts, &bar, open, &save.date, save.simulation.phase);
                }
            }
            "fixtures" => {
                // Headless render of Arsenal's fixture list (CM_ADV_DAYS ticks
                // first so some rows carry results; CM_SCROLL scrolls).
                let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
                if let Ok(world) = cm_db::World::read_rust_db_dir(std::path::Path::new(&dir)) {
                    let opts = cm_domain::NewGameOptions {
                        selected_nations: vec!["England".into()],
                        background_nations: vec![], use_real_players: true,
                        attribute_masking: true, start_year: 2001,
                    };
                    let mut save = world.new_game_from_rust_db(std::path::Path::new(&dir), &opts);
                    let h = save.add_manager(cm_domain::ManagerIdentity {
                        first: "Alex".into(), second: "Ferguson".into(), nickname: "Fergie".into(),
                    });
                    save.install_manager_at_club(h, 676, Some(60));
                    save.switch_active(h);
                    if let Ok(n) = std::env::var("CM_ADV_DAYS").unwrap_or_default().parse::<u32>() {
                        save.tick_days(n);
                        eprintln!("[dump] advanced {n} days -> {}", save.date.iso());
                    }
                    if let Some(view) = world.club_fixtures_for(&save, 676) {
                        let scroll = std::env::var("CM_SCROLL").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
                        eprintln!("[dump] fixtures: {} — {} rows", view.club_name, view.rows.len());
                        screens::club_fixtures(&mut frame, &mut fonts, bg.as_ref(), &view, scroll);
                        let bar = cm_domain::menu::MenuBar::in_game(&world, &save);
                        let open = std::env::var("CM_MENU_OPEN").ok().and_then(|v| v.parse::<usize>().ok());
                        screens::menu_sidebar(&mut frame, &mut fonts, &bar, open, &save.date, save.simulation.phase);
                    }
                }
            }
            "profile" => {
                // Headless render of the first Arsenal squad member's profile
                // (CM_PLAYER_ROW picks another dashboard squad row).
                let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
                if let Ok(world) = cm_db::World::read_rust_db_dir(std::path::Path::new(&dir)) {
                    let opts = cm_domain::NewGameOptions {
                        selected_nations: vec!["England".into()],
                        background_nations: vec![], use_real_players: true,
                        attribute_masking: true, start_year: 2001,
                    };
                    let mut save = world.new_game_from_rust_db(std::path::Path::new(&dir), &opts);
                    let h = save.add_manager(cm_domain::ManagerIdentity {
                        first: "Alex".into(), second: "Ferguson".into(), nickname: "Fergie".into(),
                    });
                    save.install_manager_at_club(h, 676, Some(60));
                    save.switch_active(h);
                    let row = std::env::var("CM_PLAYER_ROW").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
                    let dash = world.dashboard_for(&save, save.active_human);
                    let player_id = match &dash {
                        Some(cm_domain::DashboardView::Club(d)) => d.squad.get(row).map(|p| p.player_id),
                        _ => None,
                    };
                    match player_id.and_then(|id| world.player_profile_for(&save, id)) {
                        Some(view) => {
                            eprintln!("[dump] profile: {} age={:?} club={:?}", view.name, view.age, view.club_name);
                            screens::player_profile(&mut frame, &mut fonts, bg.as_ref(), &view);
                            let bar = cm_domain::menu::MenuBar::in_game(&world, &save);
                            let open = std::env::var("CM_MENU_OPEN").ok().and_then(|v| v.parse::<usize>().ok());
                            screens::menu_sidebar(&mut frame, &mut fonts, &bar, open, &save.date, save.simulation.phase);
                        }
                        None => eprintln!("[dump] profile: no player at squad row {row}"),
                    }
                }
            }
            "table" => {
                // Headless render of Arsenal's division table (day 0 unless
                // CM_ADV_DAYS advances the tick so results populate it).
                let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
                if let Ok(world) = cm_db::World::read_rust_db_dir(std::path::Path::new(&dir)) {
                    let opts = cm_domain::NewGameOptions {
                        selected_nations: vec!["England".into()],
                        background_nations: vec![], use_real_players: true,
                        attribute_masking: true, start_year: 2001,
                    };
                    let mut save = world.new_game_from_rust_db(std::path::Path::new(&dir), &opts);
                    let h = save.add_manager(cm_domain::ManagerIdentity {
                        first: "Alex".into(), second: "Ferguson".into(), nickname: "Fergie".into(),
                    });
                    save.install_manager_at_club(h, 676, Some(60));
                    save.switch_active(h);
                    if let Ok(n) = std::env::var("CM_ADV_DAYS").unwrap_or_default().parse::<u32>() {
                        save.tick_days(n);
                        eprintln!("[dump] advanced {n} days -> {}", save.date.iso());
                    }
                    if let Some(view) = world.league_table_for(&save, 676) {
                        let scroll = std::env::var("CM_SCROLL").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
                        eprintln!("[dump] table: {} — {} clubs", view.competition_name, view.rows.len());
                        // Diagnostics: why a division might resolve but match no standings rows.
                        let members = world.club_members_of_competition(view.competition_id);
                        let member_ids: std::collections::BTreeSet<u32> = members.iter().map(|(id, _)| *id).collect();
                        let overlap = save.season.standings.iter().filter(|s| member_ids.contains(&s.club_id)).count();
                        let fg_ids = world.competition_ids_for_nations(&["England".to_string()]);
                        eprintln!(
                            "[dump] division_id={} members={} standings={} overlap={} foreground_comp_ids={:?} first_standing_ids={:?}",
                            view.competition_id, members.len(), save.season.standings.len(), overlap, fg_ids,
                            save.season.standings.iter().take(5).map(|s| (s.club_id, s.club_name.clone())).collect::<Vec<_>>()
                        );
                        screens::league_table(&mut frame, &mut fonts, bg.as_ref(), &view, scroll);
                        let bar = cm_domain::menu::MenuBar::in_game(&world, &save);
                        let open = std::env::var("CM_MENU_OPEN").ok().and_then(|v| v.parse::<usize>().ok());
                        screens::menu_sidebar(&mut frame, &mut fonts, &bar, open, &save.date, save.simulation.phase);
                    }
                }
            }
            "selected" => {
                // Headless render of the Selected Leagues view for the England /
                // Arsenal test game (no tick needed).
                let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
                if let Ok(world) = cm_db::World::read_rust_db_dir(std::path::Path::new(&dir)) {
                    let opts = cm_domain::NewGameOptions {
                        selected_nations: vec!["England".into()],
                        background_nations: vec![], use_real_players: true,
                        attribute_masking: true, start_year: 2001,
                    };
                    let mut save = world.new_game_from_rust_db(std::path::Path::new(&dir), &opts);
                    let h = save.add_manager(cm_domain::ManagerIdentity {
                        first: "Alex".into(), second: "Ferguson".into(), nickname: "Fergie".into(),
                    });
                    save.install_manager_at_club(h, 676, Some(60));
                    save.switch_active(h);
                    eprintln!("[dump] selected leagues: {} nations, {} foreground", save.nation_tiers.len(), save.foreground_count());
                    screens::selected_leagues(&mut frame, &mut fonts, bg.as_ref(), &save.nation_tiers, save.new_game.as_ref());
                    let bar = cm_domain::menu::MenuBar::in_game(&world, &save);
                    let open = std::env::var("CM_MENU_OPEN").ok().and_then(|v| v.parse::<usize>().ok());
                    screens::menu_sidebar(&mut frame, &mut fonts, &bar, open, &save.date, save.simulation.phase);
                }
            }
            "scores" => {
                // Headless render of the Latest Scores table after advancing the
                // same England / Arsenal test game so fixtures are played.
                let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
                if let Ok(world) = cm_db::World::read_rust_db_dir(std::path::Path::new(&dir)) {
                    let opts = cm_domain::NewGameOptions {
                        selected_nations: vec!["England".into()],
                        background_nations: vec![], use_real_players: true,
                        attribute_masking: true, start_year: 2001,
                    };
                    let mut save = world.new_game_from_rust_db(std::path::Path::new(&dir), &opts);
                    let h = save.add_manager(cm_domain::ManagerIdentity {
                        first: "Alex".into(), second: "Ferguson".into(), nickname: "Fergie".into(),
                    });
                    save.install_manager_at_club(h, 676, Some(60));
                    save.switch_active(h);
                    let n = std::env::var("CM_ADV_DAYS").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(40);
                    save.tick_days(n);
                    let rows = cm_domain::latest_scores(&save, Some(676));
                    let scroll = std::env::var("CM_SCROLL").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
                    eprintln!("[dump] scores after {n} days: {} played fixtures", rows.len());
                    screens::latest_scores(&mut frame, &mut fonts, bg.as_ref(), &rows, scroll);
                    let bar = cm_domain::menu::MenuBar::in_game(&world, &save);
                    let open = std::env::var("CM_MENU_OPEN").ok().and_then(|v| v.parse::<usize>().ok());
                    screens::menu_sidebar(&mut frame, &mut fonts, &bar, open, &save.date, save.simulation.phase);
                }
            }
            "news" => {
                let dir = std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into());
                if let Ok(world) = cm_db::World::read_rust_db_dir(std::path::Path::new(&dir)) {
                    let opts = cm_domain::NewGameOptions {
                        selected_nations: vec!["England".into()],
                        background_nations: vec![], use_real_players: true,
                        attribute_masking: true, start_year: 2001,
                    };
                    let mut save = world.new_game_from_rust_db(std::path::Path::new(&dir), &opts);
                    let h = save.add_manager(cm_domain::ManagerIdentity {
                        first: "Alex".into(), second: "Ferguson".into(), nickname: "Fergie".into(),
                    });
                    save.install_manager_at_club(h, 676, Some(60));
                    save.switch_active(h);
                    // Advance to generate news (default 40 days; CM_ADV_DAYS overrides).
                    let n = std::env::var("CM_ADV_DAYS").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(40);
                    save.tick_days(n);
                    eprintln!("[dump] news after {n} days: {} events", save.pending_events.len());
                    let view = world.news_for(&save, h);
                    let tab = screens::NewsTab::All;
                    screens::news(&mut frame, &mut fonts, bg.as_ref(), &view, 0, 0, tab);
                    let bar = cm_domain::menu::MenuBar::in_game(&world, &save);
                    let open = std::env::var("CM_MENU_OPEN").ok().and_then(|v| v.parse::<usize>().ok());
                    screens::menu_sidebar(&mut frame, &mut fonts, &bar, open, &save.date, save.simulation.phase);
                }
            }
            _ => {
                let pressed = std::env::args().nth(4).and_then(|a| a.parse::<usize>().ok());
                screens::setup(&mut frame, &mut fonts, bg.as_ref(), pressed);
            }
        }
    }
    let mut out = format!("P6\n{} {}\n255\n", Surface::W, Surface::H).into_bytes();
    for &v in &frame.buf {
        let (r, g, b) = cm_render::unpack565(v);
        out.extend_from_slice(&[r, g, b]);
    }
    std::fs::write(path, out).expect("write ppm");
    println!("dumped {}", path);
}

/// Open the native database and cross-check the 26 picker countries against
/// real nation records. The 26-country LIST is code-derived (the exe hardcodes
/// one `_comps()` handler per country — see `game_state::real_picker_slots`);
/// the DATA behind each country comes from rust-db. This check proves at
/// startup that every picker country resolves to a nation record.
fn open_database() -> Option<cm_db::Database> {
    let dir = std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".to_string());
    match cm_db::Database::open(std::path::Path::new(&dir)) {
        Ok(db) => {
            let pristine = if db.is_pristine_shipping_data() {
                "pristine shipping data"
            } else {
                "modified database"
            };
            eprintln!(
                "[db] opened {dir}: {} nations, {} clubs, {} players ({pristine})",
                db.world.core.nations.len(),
                db.world.core.clubs.len(),
                db.world.staff.type10.len(),
            );
            // Picker-country cross-check ("Holland", "Ireland", "USA" are the
            // picker spellings; nation.dat may differ — report, don't fail).
            let nation_names: Vec<String> = db
                .world
                .core
                .nations
                .iter()
                .map(|n| cm_db::NationView::from_bytes(&n.raw).primary_name())
                .collect();
            // Picker labels are the exe's short display strings; two differ
            // from the canonical nation-record names (verified against
            // rust-db/core/nations.json 2026-08-20):
            fn canonical(label: &str) -> &str {
                match label {
                    "Ireland" => "Republic of Ireland",
                    "USA" => "United States",
                    other => other,
                }
            }
            for slot in game_state::real_picker_slots() {
                let want = canonical(&slot.primary_name).to_string();
                if !nation_names.iter().any(|n| n == &want) {
                    eprintln!(
                        "[db] picker country {:?} has no nation record (looked for {:?})",
                        slot.primary_name, want
                    );
                }
            }
            Some(db)
        }
        Err(e) => {
            eprintln!("[db] WARNING: native database unavailable ({e}); run cm-import first");
            None
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    if let Some(first) = args.next() {
        if first == "--dump" {
            let path = args.next().unwrap_or_else(|| "dump.ppm".to_string());
            let which = args.next().unwrap_or_else(|| "setup".to_string());
            dump(&path, &which);
            return;
        }
    }
    // Load the LOCKED master database ONCE and hand it to the app; every new
    // game builds an in-memory instance from it without re-reading disk.
    let world = open_database().map(|db| db.world);
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    app.world = world;
    // Dev shortcut: CM_BOOT selects a jump target past the setup flow.
    //   dashboard      → News page for an England / Arsenal / Fergie test game
    //   news-widgets   → raw NewsView widget-pool debug (colour-coded rects)
    //   dash-widgets   → raw ClubDashboardView widget-pool debug
    //   fifa           → same test game, straight to the FIFA World Rankings table
    match std::env::var("CM_BOOT").ok().as_deref() {
        Some("dashboard") => app.boot_dashboard(),
        Some("fifa") => {
            app.boot_dashboard();
            app.open_fifa_rankings();
        }
        Some("table") => {
            app.boot_dashboard();
            app.open_league_table(676); // Arsenal's division
        }
        Some("fixtures") => {
            app.boot_dashboard();
            app.open_club_fixtures(676); // Arsenal
        }
        Some("profile") => {
            // First squad row of the Arsenal dashboard.
            app.boot_dashboard();
            let first = match &app.screen {
                Screen::Dashboard { view: cm_domain::DashboardView::Club(d), .. } => {
                    d.squad.first().map(|p| p.player_id)
                }
                _ => None,
            };
            if let Some(id) = first {
                app.open_player_profile(id);
            }
        }
        Some("selected") => {
            app.boot_dashboard();
            app.open_selected_leagues();
        }
        Some("scores") => {
            // Same England / Arsenal test game, advanced so fixtures are played.
            app.boot_dashboard();
            if let Some(game) = app.game.as_mut() {
                let n = std::env::var("CM_ADV_DAYS").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(40);
                game.save.tick_days(n);
            }
            app.open_latest_scores();
        }
        Some(v @ ("news-widgets" | "dash-widgets")) => app.boot_widget_pool_debug(v),
        _ => {}
    }
    event_loop.run_app(&mut app).unwrap();
}
