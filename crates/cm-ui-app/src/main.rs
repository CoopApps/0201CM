//! 0201CM window shell.
//!
//! A winit window presenting the cm-render surface (the CM0102 design system). This is the
//! bootstrap: a real, resizable, modern window drawing our own pixels — correct from the
//! first commit. GPU (wgpu) components and the screen system grow from here; the design
//! logic (colours, bevels, `.fnt` text, layout) is already lifted and lives in cm-render.

mod game_state;
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
    /// After initialisation: the manager enters their name. `save_path` is the
    /// initialised game this manager will be installed into.
    EnterName {
        manager: game_state::ManagerName,
        save_path: String,
    },
}

/// A generic "some control is being pressed" indicator so the render pass can draw the
/// sunken state. Kept as separate cases because each screen has a different click enum.
enum Pressed {
    None,
    Setup(usize),
    Leagues(LeaguesClick),
    Season(SeasonClick),
    Name(screens::NameClick),
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
}

impl App {
    /// Re-render whatever screen is current into the frame.
    fn render(&mut self) {
        match &self.screen {
            Screen::Setup => {
                let p = match self.pressed {
                    Pressed::Setup(i) => Some(i),
                    _ => None,
                };
                screens::setup(&mut self.frame, &mut self.fonts, self.bg.as_ref(), p);
            }
            Screen::SelectLeagues(state) => {
                let p = match self.pressed {
                    Pressed::Leagues(c) => Some(c),
                    _ => None,
                };
                screens::select_leagues(&mut self.frame, &mut self.fonts, self.bg.as_ref(), state, p);
            }
            Screen::StartSeason { leagues: _, season } => {
                let p = match self.pressed {
                    Pressed::Season(c) => Some(c),
                    _ => None,
                };
                screens::start_season(&mut self.frame, &mut self.fonts, self.bg.as_ref(), season, p);
            }
            Screen::EnterName { manager, .. } => {
                screens::enter_name(&mut self.frame, &mut self.fonts, self.bg.as_ref(), manager);
            }
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
            Screen::EnterName { .. } => match screens::enter_name_hit(x, y) {
                Some(c) => Pressed::Name(c),
                None => Pressed::None,
            },
        }
    }

    /// Convert a mouse-release into a screen transition + state update.
    fn on_release(&mut self, x: i32, y: i32) {
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
                                    // initialise directly and go to the manager
                                    // name screen (Season page is skipped).
                                    let leagues = state.clone();
                                    let season = StartSeasonState::from_leagues(&leagues);
                                    if let Some(next) = initialise_and_enter_name(&leagues, &season) {
                                        self.screen = next;
                                    }
                                }
                                _ => {
                                    let leagues = state.clone();
                                    let season = StartSeasonState::from_leagues(&leagues);
                                    self.screen = Screen::StartSeason { leagues, season };
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
                            // (FUN_008120d0) then advance to the manager name
                            // screen.
                            if let Some(next) = initialise_and_enter_name(leagues, season) {
                                self.screen = next;
                            }
                        }
                        SeasonClick::Select(i) => season.selected = i,
                    }
                }
            }
            Screen::EnterName { manager, .. } => {
                if let Some(click) = screens::enter_name_hit(x, y) {
                    match click {
                        screens::NameClick::Field(i) => manager.focus = i,
                        screens::NameClick::Back => self.screen = Screen::Setup,
                        screens::NameClick::Next => {
                            if manager.is_valid() {
                                eprintln!(
                                    "[manager] {} {} ({}) — ready to install at a club",
                                    manager.first, manager.second, manager.nickname
                                );
                                // Next screen (Select Nationality / Club) is a
                                // follow-up; for now log the created manager.
                            }
                        }
                    }
                }
            }
        }
    }

    /// Keyboard input — only the Enter Name screen consumes it (typing into the
    /// focused field). Tab cycles fields; Enter advances like Next.
    fn key_input(&mut self, ch: Option<char>, named: Option<NamedKeyAction>) {
        if let Screen::EnterName { manager, .. } = &mut self.screen {
            match named {
                Some(NamedKeyAction::Backspace) => manager.backspace(),
                Some(NamedKeyAction::Tab) => manager.focus = (manager.focus + 1) % 3,
                _ => {
                    if let Some(c) = ch {
                        manager.type_char(c);
                    }
                }
            }
        }
    }
}

/// Named (non-character) keys the Enter Name screen reacts to.
#[derive(Debug, Clone, Copy)]
enum NamedKeyAction {
    Backspace,
    Tab,
}

/// Create a new game from the native database using the picker's choices.
///
/// This is the Rust counterpart of the exe's post-Next path: the exe calls
/// `FUN_008120d0` ("Initialising game data"), which fixes up the loaded pools
/// against the selection flags and constructs ~35 runtime subsystems. Our
/// equivalent is `World::new_game_from_rust_db`, which records the same inputs
/// and restricts the season to the selected nations' competitions.
///
/// HONEST SCOPE: the exe's init also builds per-staff/per-club runtime arrays,
/// fictionalises names when Real Players is No (`FUN_0051c970`), and stands up
/// the fixture/finance/transfer/fog-of-war subsystems. Those are separate lifts
/// still in progress — see the backend ledger inside the written save.
/// Initialise the game from the picker choices, then return the EnterName
/// screen for the manager to create their identity. `None` if init failed.
fn initialise_and_enter_name(
    leagues: &game_state::SelectLeaguesState,
    season: &game_state::StartSeasonState,
) -> Option<Screen> {
    match create_new_game(leagues, season) {
        Ok(path) => {
            eprintln!("[start] game initialised: {path} — entering manager name");
            Some(Screen::EnterName {
                manager: game_state::ManagerName::default(),
                save_path: path,
            })
        }
        Err(e) => {
            eprintln!("[start] could not create game: {e}");
            None
        }
    }
}

fn create_new_game(
    leagues: &game_state::SelectLeaguesState,
    season: &game_state::StartSeasonState,
) -> Result<String, String> {
    let db_dir = std::env::var("CM_RUST_DB")
        .unwrap_or_else(|_| "D:/cm0102-rs/rust-db".to_string());
    let world = cm_db::World::read_rust_db_dir(std::path::Path::new(&db_dir))
        .map_err(|e| format!("open database: {e}"))?;

    let options = cm_domain::NewGameOptions {
        selected_nations: leagues
            .slots
            .iter()
            .filter(|slot| slot.selected)
            .map(|slot| slot.primary_name.clone())
            .collect(),
        background_nations: leagues
            .slots
            .iter()
            .filter(|slot| slot.background_marker)
            .map(|slot| slot.primary_name.clone())
            .collect(),
        use_real_players: leagues.options.use_real_players,
        attribute_masking: leagues.options.attribute_masking,
        // The season label is "<Country> 01/02" or "<Country> 2002"; the
        // shipped database starts in 2001 either way.
        start_year: 2001,
    };
    let _ = season;

    let save = world.new_game_from_rust_db(std::path::Path::new(&db_dir), &options);
    let out = std::env::var("CM_SAVE_PATH")
        .unwrap_or_else(|_| "D:/cm0102-rs/saves/new_game.json".to_string());
    save.write_json_file(std::path::Path::new(&out))
        .map_err(|e| format!("write save: {e}"))?;
    let fg = save.foreground_count();
    let bg = save
        .nation_tiers
        .iter()
        .filter(|t| t.tier == cm_domain::LeagueTier::Background)
        .count();
    eprintln!(
        "[start] tiers: {fg} foreground / {bg} background / {} total nations (world not culled); {} fixtures, {} standings, date {}-{:02}-{:02}",
        save.nation_tiers.len(),
        save.season.fixtures.len(),
        save.season.standings.len(),
        save.date.year,
        save.date.month,
        save.date.day,
    );
    Ok(out)
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
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as i32, position.y as i32);
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
                            _ => false,
                        };
                        if same {
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
                    nickname: "Fergie".into(),
                    focus: 1,
                };
                screens::enter_name(&mut frame, &mut fonts, bg.as_ref(), &manager);
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
    let _db = open_database();
    let mut args = std::env::args().skip(1);
    if let Some(first) = args.next() {
        if first == "--dump" {
            let path = args.next().unwrap_or_else(|| "dump.ppm".to_string());
            let which = args.next().unwrap_or_else(|| "setup".to_string());
            dump(&path, &which);
            return;
        }
    }
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
