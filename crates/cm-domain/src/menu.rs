//! The persistent menu bar — a port of the exe's `FUN_00745540` (`game_mbr`).
//!
//! The original builds the bar with ~55 `FUN_00549580` widget-registration
//! calls, one per item, each carrying a 16-bit command code (the 3rd-from-last
//! argument) and an item-type flag. Three modes are selected by its `param_1`:
//!
//!   * `1` — pre-game / setup bar (Game menu only).
//!   * `2` — full in-game bar (all five menus). **This is what we build.**
//!   * `3` — match-in-progress bar (Paused / Tactics / Commentary Speed).
//!
//! Command codes below are the *literal* values read from the decompile
//! (`00745540.c`), not the data-driven inference in the scope doc. They are
//! dispatched by the two routers `FUN_007491e0` (global) and `FUN_0074bf60`
//! (club) — see [`describe_command`] and `reports/menu_tree_scope.md`.

use crate::RuntimeSaveGame;

/// Menu-bar command codes (from `00745540.c`, verified against the routers).
pub mod cmd {
    // --- Game ---
    pub const ADD_MANAGER: u16 = 0x3fb;
    pub const CONTINUE: u16 = 1000; // 0x3e8 is News; 1000 is the Continue item id
    pub const LATEST_SCORES: u16 = 0x418;
    pub const RESTART_GAME: u16 = 0x42f;
    pub const EXIT_GAME: u16 = 0x402;

    // --- Manager Options ---
    pub const SQUAD: u16 = 0x7d5; // <club>/<nation> Squad -> club dashboard/squad
    pub const B_SQUAD: u16 = 0x7d6; // Reserves / B squad
    pub const CONTROL_ALL_TEAMS: u16 = 0x433;
    pub const CONTROL_RESERVE: u16 = 0x425;
    pub const FA_CONFIDENCE: u16 = 0x420;
    pub const BOARD_CONFIDENCE: u16 = 0x41f;
    pub const RESIGN_FROM_NATION: u16 = 0x41d;
    pub const RESIGN_FROM_CLUB: u16 = 0x41c;
    pub const NEWS: u16 = 0x3e9;
    pub const PLAYER_STAFF_SEARCH: u16 = 0x3eb;
    pub const COMPARE_PLAYERS: u16 = 0x434;
    pub const MANAGER_STATS: u16 = 0x42e;
    pub const JOB_INFORMATION: u16 = 0x41e;
    pub const TRANSFERS: u16 = 0x415;
    pub const MANAGER_HISTORY: u16 = 0x3ec;
    pub const GO_ON_HOLIDAY: u16 = 0x3ef;
    pub const RETURN_FROM_HOLIDAY: u16 = 0x3f0;
    pub const SEND_ABUSE: u16 = 0x414;
    pub const RETIRE: u16 = 0x3f2;

    // --- Competitions ---
    pub const AWARDS: u16 = 0x7d4;
    pub const FIFA_RANKINGS: u16 = 0x3f3;
    pub const UEFA_COEFFICIENTS: u16 = 0x40c;

    // --- Nations & Clubs ---
    pub const NATIONS: u16 = 0x3f4;
    pub const UNDER_21S: u16 = 0x3f5;
    pub const MAJOR_CLUBS: u16 = 0x3f6;
    pub const LEAGUE: u16 = 0x3f7;
    pub const NON_LEAGUE: u16 = 0x3f8;
    pub const OTHER_CLUBS: u16 = 0x3f9;
    pub const FIND_NATION: u16 = 0x3fa;
    pub const WRITTEN_HISTORY: u16 = 0x429;

    // --- Game Options ---
    pub const SAVE_GAME: u16 = 0x3fe;
    pub const MANAGER_STATUS: u16 = 0x3fc;
    pub const GAME_SETTINGS: u16 = 0x423;
    pub const SELECTED_LEAGUES: u16 = 0x431;
    pub const HALL_OF_FAME: u16 = 0x42a;
    pub const GAME_CREDITS: u16 = 0x42b;
    pub const WEB_SITES: u16 = 0x42c;
}

/// One leaf item in a drop-down menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub label: String,
    /// The 16-bit command dispatched when the item is chosen. `0` = inert
    /// (a disabled label; the exe registers these with command 0).
    pub command: u16,
    /// Whether the item is currently actionable. The exe greys items by
    /// swapping the command to 0 and the type flag to `0x2c`; we keep the
    /// label and mark it disabled instead.
    pub enabled: bool,
    /// A separator is drawn above this item (the exe's `0x1000010` flag).
    pub separator_before: bool,
}

impl MenuItem {
    fn new(label: &str, command: u16, enabled: bool) -> Self {
        Self { label: label.to_string(), command, enabled, separator_before: false }
    }
    fn sep(mut self) -> Self {
        self.separator_before = true;
        self
    }
}

/// One top-level entry in the sidebar. Either a direct action (`command` set,
/// no drop-down — e.g. "Continue Game") or a menu header owning a drop-down
/// (`command` = None, `items` populated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuTop {
    pub label: String,
    pub command: Option<u16>,
    pub items: Vec<MenuItem>,
}

impl MenuTop {
    fn action(label: &str, command: u16) -> Self {
        Self { label: label.to_string(), command: Some(command), items: Vec::new() }
    }
    fn menu(label: &str, items: Vec<MenuItem>) -> Self {
        Self { label: label.to_string(), command: None, items }
    }
    /// True for a direct-action top-level (no drop-down).
    pub fn is_action(&self) -> bool {
        self.command.is_some()
    }
}

/// The whole persistent bar for the active human.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBar {
    pub menus: Vec<MenuTop>,
}

impl MenuBar {
    /// Build the full in-game bar (the exe's mode-2 `FUN_00745540`) for the
    /// active human of `save`. Item presence follows the same gates the exe
    /// applies: nation items only when the human holds a nation job, club
    /// items only when they hold a club, resign items only when appointed.
    pub fn in_game(save: &RuntimeSaveGame) -> Self {
        let human = save.humans.get(save.active_human);
        let has_club = human.map(|h| h.club.is_some()).unwrap_or(false);
        let has_nation = human.map(|h| h.nation.is_some()).unwrap_or(false);
        let appointed = has_club || has_nation;
        // Manager Options is labelled with the manager's own name once
        // appointed (the exe's FUN_005276f0 into the header), else the generic
        // "Manager Options".
        let mgr_label = match human {
            Some(h) if appointed => h.identity.display_name(),
            _ => "Manager Options".to_string(),
        };
        let multi_human = save.humans.len() > 1;

        let mut menus = Vec::new();

        // ---- Continue Game (direct action, top of the sidebar) ----
        menus.push(MenuTop::action("Continue Game", cmd::CONTINUE));

        // ---- Manager Options (labelled with the manager's name) ----
        let mut mo = Vec::new();
        if has_nation {
            mo.push(MenuItem::new("Nation Squad", cmd::SQUAD, true));
            mo.push(MenuItem::new("Nation U21 Squad", cmd::SQUAD, true));
            mo.push(MenuItem::new("Control All Teams", cmd::CONTROL_ALL_TEAMS, true));
            mo.push(MenuItem::new("FA Confidence", cmd::FA_CONFIDENCE, true));
            mo.push(MenuItem::new("Resign from Nation", cmd::RESIGN_FROM_NATION, true).sep());
        }
        if has_club {
            mo.push(MenuItem::new("Club Squad", cmd::SQUAD, true));
            mo.push(MenuItem::new("Club Reserves", cmd::B_SQUAD, true));
            mo.push(MenuItem::new("Control Reserve Team", cmd::CONTROL_RESERVE, true));
            mo.push(MenuItem::new("Board Confidence", cmd::BOARD_CONFIDENCE, true));
            mo.push(MenuItem::new("Resign from Club", cmd::RESIGN_FROM_CLUB, true).sep());
        }
        // Always-present manager tools (the exe shows them even when unemployed).
        mo.push(MenuItem::new("News", cmd::NEWS, true));
        mo.push(MenuItem::new("Player & Staff Search", cmd::PLAYER_STAFF_SEARCH, true));
        mo.push(MenuItem::new("Compare two chosen players", cmd::COMPARE_PLAYERS, true));
        mo.push(MenuItem::new("Manager Stats", cmd::MANAGER_STATS, true));
        mo.push(MenuItem::new("Job Information", cmd::JOB_INFORMATION, true));
        mo.push(MenuItem::new("Transfers", cmd::TRANSFERS, true));
        mo.push(MenuItem::new("History", cmd::MANAGER_HISTORY, true));
        mo.push(MenuItem::new("Go on Holiday", cmd::GO_ON_HOLIDAY, appointed));
        mo.push(MenuItem::new("Retire", cmd::RETIRE, appointed).sep());
        menus.push(MenuTop::menu(&mgr_label, mo));

        // ---- Competitions ----
        menus.push(MenuTop::menu(
            "Competitions",
            vec![
                MenuItem::new("Awards", cmd::AWARDS, true),
                MenuItem::new("FIFA Rankings", cmd::FIFA_RANKINGS, true),
                MenuItem::new("UEFA Coefficients", cmd::UEFA_COEFFICIENTS, true),
            ],
        ));

        // ---- Nations & Clubs ----
        menus.push(MenuTop::menu(
            "Nations & Clubs",
            vec![
                MenuItem::new("Nations", cmd::NATIONS, true),
                MenuItem::new("Under 21s", cmd::UNDER_21S, true),
                MenuItem::new("Major Clubs", cmd::MAJOR_CLUBS, true),
                MenuItem::new("League", cmd::LEAGUE, true),
                MenuItem::new("Non-League", cmd::NON_LEAGUE, true),
                MenuItem::new("Other Clubs", cmd::OTHER_CLUBS, true),
            ],
        ));

        // ---- Find (its own top-level; label 0xa46488) ----
        menus.push(MenuTop::menu(
            "Find",
            vec![
                MenuItem::new("Nation", cmd::FIND_NATION, true),
                MenuItem::new("Non-Player", cmd::FIND_NATION, true),
                MenuItem::new("Player", cmd::FIND_NATION, true),
                MenuItem::new("Written History", cmd::WRITTEN_HISTORY, true).sep(),
            ],
        ));

        // ---- Change Player (only when more than one human is in the game) ----
        if multi_human {
            menus.push(MenuTop::menu(
                "Change Player",
                save.humans
                    .iter()
                    .map(|h| MenuItem::new(&h.identity.display_name(), cmd::MANAGER_STATUS, true))
                    .collect(),
            ));
        }

        // ---- Game Options ----
        menus.push(MenuTop::menu(
            "Game Options",
            vec![
                MenuItem::new("Save Game", cmd::SAVE_GAME, true),
                MenuItem::new("Add Manager", cmd::ADD_MANAGER, true),
                MenuItem::new("Manager Status", cmd::MANAGER_STATUS, true),
                MenuItem::new("Game Settings", cmd::GAME_SETTINGS, true),
                MenuItem::new("Selected Leagues", cmd::SELECTED_LEAGUES, true),
                MenuItem::new("Hall of Fame", cmd::HALL_OF_FAME, true),
                MenuItem::new("Game Credits", cmd::GAME_CREDITS, true),
                MenuItem::new("Web Sites", cmd::WEB_SITES, true),
                MenuItem::new("Restart Game", cmd::RESTART_GAME, true).sep(),
                MenuItem::new("Exit Game", cmd::EXIT_GAME, true),
            ],
        ));

        MenuBar { menus }
    }
}

/// Human-readable target for a command code (from the two routers). Used for
/// logging and for the "not yet implemented" note on unwired commands.
pub fn describe_command(command: u16) -> &'static str {
    match command {
        cmd::ADD_MANAGER => "Add Manager (FUN_007e7790)",
        cmd::CONTINUE => "Continue — advance one day (FUN_005b6a90)",
        cmd::LATEST_SCORES => "Latest Scores / Match (FUN_00700f20)",
        cmd::RESTART_GAME => "Restart Game (confirm)",
        cmd::EXIT_GAME => "Exit Game",
        cmd::SQUAD => "Squad / Club Dashboard (FUN_00454620)",
        cmd::B_SQUAD => "Reserves / B Squad (FUN_00454620)",
        cmd::CONTROL_ALL_TEAMS => "Control All Teams (toggle)",
        cmd::CONTROL_RESERVE => "Control Reserve Team (toggle)",
        cmd::FA_CONFIDENCE => "FA Confidence (FUN_00697...)",
        cmd::BOARD_CONFIDENCE => "Board Confidence",
        cmd::RESIGN_FROM_NATION => "Resign from Nation",
        cmd::RESIGN_FROM_CLUB => "Resign from Club",
        cmd::NEWS => "News (FUN_0076f2f0)",
        cmd::PLAYER_STAFF_SEARCH => "Player & Staff Search (FUN_00859250)",
        cmd::COMPARE_PLAYERS => "Compare Players (FUN_007dfac0)",
        cmd::MANAGER_STATS => "Manager Stats",
        cmd::JOB_INFORMATION => "Job Information",
        cmd::TRANSFERS => "Transfers (FUN_008e3700)",
        cmd::MANAGER_HISTORY => "Manager History (FUN_00859250)",
        cmd::GO_ON_HOLIDAY => "Go on Holiday (FUN_00822940)",
        cmd::RETURN_FROM_HOLIDAY => "Return from Holiday",
        cmd::SEND_ABUSE => "Send Abuse To",
        cmd::RETIRE => "Retire",
        cmd::AWARDS => "Awards (FUN_00415010)",
        cmd::FIFA_RANKINGS => "FIFA Rankings (FUN_004a2190)",
        cmd::UEFA_COEFFICIENTS => "UEFA Coefficients (FUN_004a28c0)",
        cmd::NATIONS => "Nations list",
        cmd::UNDER_21S => "Under 21s list",
        cmd::MAJOR_CLUBS => "Major Clubs list",
        cmd::LEAGUE => "League list",
        cmd::NON_LEAGUE => "Non-League list",
        cmd::OTHER_CLUBS => "Other Clubs list",
        cmd::FIND_NATION => "Find (nation/non-player/player)",
        cmd::WRITTEN_HISTORY => "Written History",
        cmd::SAVE_GAME => "Save Game (FUN_00822940)",
        cmd::MANAGER_STATUS => "Manager Status (FUN_00808a70)",
        cmd::GAME_SETTINGS => "Game Settings (FUN_004ec550)",
        cmd::SELECTED_LEAGUES => "Selected Leagues (FUN_008053d0)",
        cmd::HALL_OF_FAME => "Hall of Fame (FUN_0080fac0)",
        cmd::GAME_CREDITS => "Game Credits (FUN_004ec550)",
        cmd::WEB_SITES => "Web Sites (FUN_004fd1b0)",
        _ => "unknown command",
    }
}
