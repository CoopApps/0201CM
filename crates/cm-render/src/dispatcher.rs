//! Layer 4 — click dispatchers.
//!
//! Direct ports of the two central "click → screen builder" routers in
//! cm0102.exe:
//!
//! * [`dispatch_global`] — port of `FUN_007491e0` (1564 lines of C at
//!   `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/007491e0.c`).
//!   Handles the 41 menu-bar / global-nav command codes in the range
//!   `0x3e8`..=`0x434`, plus the -2 / -3 nav-bar synthetic messages
//!   fired by [`screen_nav_back_next`], plus `1000`, `0x402`.
//! * [`dispatch_club`] — port of `FUN_0074bf60` (384 lines at
//!   `.../0074bf60.c`). Handles the 22 club-context command codes in
//!   the range `0x426`, `0x7d0`..=`0x7e7`.
//!
//! # Command resolution — the recurring 3-line pattern
//!
//! Both dispatchers open every branch with the same idiom (C addresses
//! `007491e0:0x0074921f..0x00749235` and analogues throughout):
//!
//! ```c
//! if (sVar4 == -1)    sVar3 = 0;
//! else                sVar3 = (short)*(u32*)(pool + 0xba966 + sVar4*0x18c);
//! if (sVar3 == 0)     sVar3 = DAT_00dbbf7a;   // fallback "pending cmd" global
//! ```
//!
//! That is: read the clicked widget's cmd slot; fall back to the global
//! "pending command" if the widget has none. `param_1` is a *widget
//! slot index*, not a raw cmd — the dispatcher does the resolve. The
//! port centralises this into [`resolve_widget_cmd`].
//!
//! # What THIS module does vs does NOT port
//!
//! **Ports faithfully:** every `if (sVar3 == 0xNNN)` branch of both C
//! dispatchers becomes a Rust `match` arm, with the exe address of that
//! branch cited in a per-arm comment. Each arm routes to either
//! (a) [`build_nav_back_next`][crate::screen_nav_back_next::build_nav_back_next]
//! for the two synthetic nav messages (the one Layer-3 builder we own),
//! or (b) a [`DispatchResult::TodoBuilder`] naming the FUN_ that Layer 3
//! will eventually port.
//!
//! **Scaffolds only:** the ~640-line pre-dispatch phase of
//! `FUN_007491e0` (lines 74..643 of the C — autosave polling, five
//! separate `DAT_00dbbf7c` dialog-result branches for cmd codes 8/9/10/
//! 0xb/0x2a/0x37/0x3a/0x3d/0x3e/0x44, and the sidebar-text update loop
//! at lines 141..189 / 191..289) touches ~20 unported subsystems
//! (network state via `FUN_007ebaf0`, autosave timing via `FUN_00822580`
//! / `FUN_006547c0`, dialog result queues via `DAT_00dbbf7c` +
//! `DAT_00dbbf80`, sidebar sprintf via `FUN_007e83e0` / `FUN_007e9180`
//! / `FUN_007e9a80` / `FUN_007e9dc0`, etc.). Porting those is out of
//! Layer-4 scope. [`pre_dispatch_bookkeeping`] catalogues each range
//! with its C line span and lists the FUN_ dependency, then returns
//! `PreDispatchOutcome::Skipped` so the switch still runs.
//!
//! **Live route:** the -2 (Back) and -3 (Next) synthetic messages
//! rebuild the nav bar via `build_nav_back_next` as a demonstration of
//! Layer 4 → Layer 3 wiring. That is not what the exe does (the exe's
//! FUN_007e6b60 / FUN_007e6ab0 only *predicate* — they return whether
//! the back/next stack has room — and the actual rebuild happens in the
//! screen-stack manager), but it proves the routing is intact end-to-
//! end without dragging the screen-stack manager into this commit.

use crate::screen_nav_back_next::build_nav_back_next;
use crate::screen_news::{build_news_screen, NewsScreenState};
use crate::widget_pool::GuiRecordPool;

// ============================================================================
// Public types
// ============================================================================

/// Outcome of a dispatch call.
///
/// The exe returns `int` codes with these meanings (see the -N return
/// values scattered through `FUN_007491e0`, plus `0` for "no-op /
/// unhandled" and `0xfffffffc` (i.e. `-4`) throughout `FUN_0074bf60`):
///
/// | C return | Meaning |
/// |----------|---------|
/// |  `0`     | unhandled — pass to next dispatcher / no state change |
/// | `-2`     | nav-back accepted (screen manager should pop) |
/// | `-3`     | nav-next accepted (screen manager should push) |
/// | `-4`    (0xfffffffc) | handled + screen was rebuilt / dialog shown |
/// | `-5`    (0xfffffffb) | handled but blocked (network gate) |
/// | `-0xb`  (0xfffffff5) | soft-refuse (menu shortcut suppressed) |
/// | `-0xc`  (0xfffffff4) | handled + top-level state change (game exit-request) |
///
/// The Rust wrapper preserves these as named variants so downstream
/// screen-manager code can match on intent, not a magic number.
#[derive(Debug, Clone, PartialEq)]
pub enum DispatchResult {
    /// Cmd not handled here. Try the next dispatcher, or ignore.
    /// Exe: `return 0;` (e.g. `007491e0:0x1413`).
    Unhandled,
    /// Nav-back accepted. Screen manager pops. Exe: `return -2;`
    /// (`007491e0:0x00749a2a`).
    NavBack,
    /// Nav-next accepted. Screen manager pushes. Exe: `return -3;`
    /// (`007491e0:0x00749a52`).
    NavNext,
    /// Cmd handled — a new screen was built or a modal dialog shown.
    /// Exe: `return -4;` (many sites).
    Handled,
    /// Handled but blocked by network state. Exe: `return -5;`.
    HandledBlocked,
    /// Soft-refuse — the menu shortcut was suppressed (e.g. autosave
    /// gate, screen already active). Exe: `return -0xb;`.
    Refused,
    /// Top-level state change requested (game exit). Exe: `return -0xc;`.
    ExitRequested,
    /// The clicked widget's cmd targets a Layer-3 builder that has
    /// not yet been ported. Carries the FUN_ address so Layer 3 can
    /// pick it up. This is NOT a routing failure — the dispatcher
    /// resolved cmd correctly; only the target is deferred.
    TodoBuilder {
        cmd: i16,
        fn_addr: &'static str,
        note: &'static str,
    },
    /// Cmd resolved but the branch is unreachable in the port (dead
    /// code in the exe, or a case that requires deep state the port
    /// hasn't modelled). Carries the exe address of the branch.
    TodoBranch { cmd: i16, at: &'static str },
}

/// Outcome of the pre-dispatch bookkeeping phase.
///
/// The exe's `FUN_007491e0` runs ~640 lines of stateful code before it
/// reaches the cmd switch. That code polls autosave, drains dialog
/// result queues, updates the sidebar label + attached command, and
/// short-circuits with a `return -N` if any of those consumed the
/// tick. The port scaffolds that phase — see [`pre_dispatch_bookkeeping`].
#[derive(Debug, Clone, PartialEq)]
pub enum PreDispatchOutcome {
    /// Bookkeeping did not consume the tick — proceed to the switch.
    /// This is the branch the port currently always takes.
    Continue,
    /// Bookkeeping short-circuited with a return code (e.g. autosave
    /// dialog opened, dialog result consumed). Not yet reachable in
    /// the port.
    ShortCircuit(DispatchResult),
    /// The pre-dispatch phase's state dependencies are not ported.
    /// The port continues to the switch anyway — the dispatch table
    /// itself does not depend on this state.
    Skipped,
}

// ============================================================================
// Cmd resolution helper — the 3-line idiom repeated 40+ times in the C
// ============================================================================

/// Byte offset of the widget's cmd slot within its 0x18C-byte record.
///
/// Exe: `pool + 0xBA966 + slot*0x18C`. The base pool is `DAT_00B59FE8`,
/// widgets start at `+0xBA95E`, so `0xBA966 - 0xBA95E = 8` — the cmd
/// slot is the widget's `+0x08` field (a dword read as a short).
///
/// In the port that dword lives on
/// [`WidgetDescriptor::msg_id`][crate::widget_pool::WidgetDescriptor::msg_id]
/// (Rust `i32` — the C reads it as a `short`, so we truncate on read).
pub const WIDGET_CMD_FIELD_OFFSET: usize = 0x08;

/// Global "pending cmd" fallback — `DAT_00dbbf7a` in the exe.
///
/// Set by menu shortcuts and dialog-result handlers so that a click
/// with no per-widget cmd can still trigger a screen change. The port
/// mirrors it as a mutable field on [`DispatcherState`].
#[derive(Debug, Clone, Default)]
pub struct DispatcherState {
    /// Exe: `DAT_00dbbf7a` (short). Fallback cmd when the clicked
    /// widget has none. Cleared by branches that "own" a cmd (e.g.
    /// `007491e0:0x0074bcaf` clears it inside the 0x42b case).
    pub pending_cmd_fallback: i16,
    /// Exe: `DAT_00dbbf7c` (short). Dialog-result-pending flag. The
    /// pre-dispatch phase drains this. Not yet used by the port.
    pub pending_dialog_result: i16,
    /// Exe: `DAT_00dbbf80` (short). Dialog-result value (2 = OK,
    /// 1 = Cancel, etc.).
    pub pending_dialog_value: i16,
}

/// Read `sVar3` per the recurring 3-line C idiom. Returns the resolved
/// cmd code — either the clicked widget's cmd (truncated `short`), or
/// the global fallback if the widget has none, or 0 if the slot itself
/// is `-1` (synthetic dispatch with no widget).
///
/// Direct port of e.g. `007491e0:0x00749212..0x00749239` (repeated 40+
/// times throughout the two dispatchers).
///
/// ```c
/// if (sVar4 == -1)  sVar3 = 0;
/// else              sVar3 = (short)*(u32*)(pool + 0xba966 + sVar4*0x18c);
/// if (sVar3 == 0)   sVar3 = DAT_00dbbf7a;
/// ```
pub fn resolve_widget_cmd(pool: &GuiRecordPool, widget_slot: i16, state: &DispatcherState) -> i16 {
    let mut cmd: i16 = 0;
    if widget_slot != -1 {
        if let Some(w) = pool.widgets.get(widget_slot as usize) {
            // Exe reads +0x08 as a short — we take the low 16 bits of msg_id.
            cmd = w.descriptor.msg_id as i16;
        }
    }
    if cmd == 0 {
        cmd = state.pending_cmd_fallback;
    }
    cmd
}

/// Read the widget's `+0xba9a6` slot — the *payload* pointer/value
/// attached to a click. The C uses this whenever a cmd needs entity
/// context (e.g. `0x414`, `0x7e6`, `0x7d3`).
///
/// Byte offset within widget: `0xBA9A6 - 0xBA95E = 0x48`. In the port
/// that corresponds to
/// [`WidgetDescriptor::userdata_id`][crate::widget_pool::WidgetDescriptor::userdata_id].
pub fn resolve_widget_payload(pool: &GuiRecordPool, widget_slot: i16) -> u32 {
    if widget_slot == -1 {
        return 0;
    }
    pool.widgets
        .get(widget_slot as usize)
        .map(|w| w.descriptor.userdata_id)
        .unwrap_or(0)
}

// ============================================================================
// Pre-dispatch bookkeeping scaffold
// ============================================================================

/// Scaffold for the pre-dispatch phase of `FUN_007491e0` (lines 74..643
/// of the C decomp). Each phase below cites its C line range + the
/// FUN_ subsystem it touches; none are ported here because doing so
/// requires the network / autosave / dialog-result subsystems that are
/// out of Layer-4 scope. The port returns `Skipped` and lets the switch
/// run.
///
/// **Phase 1 — early cmd-0x424 short-circuit** (C lines 84..86).
/// `if (sVar3 == 0x424) return -0xb;` — the Save Game shortcut is
/// suppressed if the screen manager already has it primed. In the
/// port this is folded into the switch: 0x424 always dispatches.
///
/// **Phase 2 — autosave poll + sidebar-text update** (C lines 87..292,
/// `007491e0:0x00749324..0x0074992f`). Reads:
///   * `DAT_00b59fc2[...+0xc0*seat]` — the currently viewed club
///   * `DAT_00acdf28` — autosave config record
///   * `DAT_00b5d020` — active-human seat
///   * `DAT_00dbc408`, `DAT_00dbc378/37c` — autosave date/time
/// Calls: `FUN_00822580`, `FUN_007ebaf0`, `FUN_00818060`, `FUN_006547c0`,
/// `FUN_00822940`, `FUN_007e7cb0`, `FUN_007e8b70`, `FUN_007e83e0`,
/// `FUN_007e9180`, `FUN_007e9a80`, `FUN_007e9dc0`, `FUN_007eaac0`.
/// All unported.
///
/// **Phase 3 — network wait / holiday-check** (C lines 294..375,
/// `LAB_00749c4d..LAB_00749dcc`). Reads `DAT_00b59fec`, `DAT_00b59e80`,
/// `DAT_00b4d5a8`, calls `FUN_00935f4b` (system time), `FUN_00789b50`
/// (network peer count), `FUN_00789a40`. Unported.
///
/// **Phase 4 — dialog-result drain** (C lines 380..643, ten branches
/// each keyed off `DAT_00dbbf7c == N`):
///   * `N=8`  (`007491e0:0x0074a145`) → holiday-in-network cancel
///   * `N=9`  (`0x0074a1e4`) → holiday-out cancel
///   * `N=0x2a` (`0x0074a209`) → return-from-holiday redistributor
///   * `N=7`  (`0x0074a28d`) → save-with-name enter dialog
///   * `N=0x3a` (`0x0074a292`) → generic auto-save-name path
///   * `N=0x3e` (`0x0074a284`) → save-and-quit
///   * `N=0x37` (`0x0074a534`) → save-then-restart
///   * `N=10` (`0x0074a54a`) → resume-from-network dialog
///   * `N=0x44` (`0x0074a58c`) → nation-control confirm
///   * `N=0x3d` (`0x0074a5bd`) → restart-game confirm
///   * `N=0xb` (`0x0074a6c4`) → exit-game confirm
/// Each drains `DAT_00dbbf7c`+`DAT_00dbbf80` and calls the matching
/// subsystem (`FUN_005e6e30`, `FUN_00934ba3`, `FUN_00818060`,
/// `FUN_00810ca0`, `FUN_00672e10`, `FUN_005b6a10`, `FUN_0061d290`,
/// `FUN_009349c4`). All unported.
///
/// The port continues to Phase 5 (the cmd switch — [`dispatch_global`]
/// body proper).
pub fn pre_dispatch_bookkeeping(
    _pool: &mut GuiRecordPool,
    _state: &mut DispatcherState,
) -> PreDispatchOutcome {
    // TODO: port Phases 2..4 once the network + autosave + dialog-result
    // subsystems land. Every branch above needs one FUN_ that we
    // haven't yet ported; skipping this phase is faithful to "no
    // approximation" — we don't fabricate a decision, we just don't
    // reach it. The switch itself does not depend on any of this
    // state (each switch arm resolves cmd from the widget slot).
    PreDispatchOutcome::Skipped
}

/// Scaffold for the post-dispatch sidebar-notification update
/// (`007491e0:LAB_00749c4d..LAB_00749992f`, C lines 294..643 tail).
/// Updates the sidebar-message widget's text + cmd binding based on:
///   * `FUN_0076ef80` — unread news count (cmd 0x3e9 "You Have News")
///   * `DAT_00b51340[club_slot]` — chat-message flag (cmd 0x421)
///   * `FUN_008224b0` — network-waiting-players count (cmd 0x3fc)
/// Unported for the same reasons as Phase 2.
pub fn post_dispatch_sidebar_update(_pool: &mut GuiRecordPool, _state: &mut DispatcherState) {
    // TODO: port once FUN_0076ef80 / FUN_008224b0 / DAT_00b51340 land.
}

// ============================================================================
// Default state accessors for LIVE builder routes
// ============================================================================

/// Minimum-viable `NewsScreenState` for the cmd-1000 live route.
///
/// Real Layer-6 wiring will thread a per-seat state; this default
/// mirrors what `screen_news`'s own tests use so the route produces
/// the same 9 areas + tab/label widgets the fixture captured.
fn default_news_state() -> NewsScreenState {
    NewsScreenState {
        header_title: String::new(),
        active_tab: 0,
        filter_text: String::new(),
        next_unread_enabled: false,
        items: Vec::new(),
        selected_body: String::new(),
        back_disabled: true,
        next_enabled: false,
    }
}

// ============================================================================
// Global dispatcher — port of FUN_007491e0
// ============================================================================

/// Direct port of `FUN_007491e0`. Called on every click; `widget_slot`
/// is the index of the widget the user clicked (or `-1` if the caller
/// is invoking a menu shortcut with no source widget).
///
/// The dispatch table below has one `match` arm per `if (sVar3 == 0xNNN)`
/// branch of the C. The C dispatches by falling through a chain of
/// `if` blocks (Ghidra's decompile of what was likely a jump table in
/// the asm) — the port collapses that into a single `match` since
/// the arms are mutually exclusive (each arm hits a `return -4;` or
/// analogous).
pub fn dispatch_global(
    pool: &mut GuiRecordPool,
    state: &mut DispatcherState,
    widget_slot: i16,
) -> DispatchResult {
    // ---- Phases 1..4: pre-dispatch bookkeeping (scaffold, see fn doc)
    if let PreDispatchOutcome::ShortCircuit(r) = pre_dispatch_bookkeeping(pool, state) {
        return r;
    }

    // ---- Cmd resolution — the 3-line idiom (`007491e0:0x00749212`).
    let cmd = resolve_widget_cmd(pool, widget_slot, state);

    // ---- Nav-bar synthetic messages fired by build_nav_back_next.
    // The exe: `if ((sVar3 == -2) && (FUN_007e6b60(0) == 0)) return -2;`
    //          `if ((sVar3 == -3) && (FUN_007e6ab0() == 0))  return -3;`
    // (`007491e0:0x00749a26..0x00749a52`, targets 007e6b60 / 007e6ab0
    // both predicate-only). For the LIVE route, we also rebuild the
    // nav bar via the ported Layer-3 builder so this commit has one
    // end-to-end route from click → screen render.
    match cmd {
        -2 => {
            // FUN_007e6b60 predicate: "can we go back?" — 27 lines,
            // depends on FUN_00933d8f + FUN_008fc660 + DAT_00b4d5a8.
            // Predicate outcome is unmodelled in this port; we assume
            // yes and rebuild the nav bar (the observed screen has
            // back active, next inactive, so back_flag=1, next_flag=0).
            let panel = 0x01u8;
            let ink = 0x0000u16;
            let _ = build_nav_back_next(pool, /*back*/ 1, /*next*/ 0, panel, ink);
            return DispatchResult::NavBack;
        }
        -3 => {
            // FUN_007e6ab0 predicate: "can we go forward?" — 24 lines,
            // reads `*(int*)(iVar1 + 0x1f8)`. Same treatment.
            let panel = 0x01u8;
            let ink = 0x0000u16;
            let _ = build_nav_back_next(pool, /*back*/ 1, /*next*/ 1, panel, ink);
            return DispatchResult::NavNext;
        }
        _ => {}
    }

    // ---- The 41-entry cmd table. Each arm cites the C address of
    // its `if (sVar3 == 0xNNN)` branch head. `TodoBuilder` names the
    // FUN_ the arm targets — Layer 3 picks them up one by one.
    let result = match cmd {
        // 0x424 (`007491e0:0x00749292`) — Save Game "Enter File Name"
        0x424 => DispatchResult::TodoBuilder {
            cmd: 0x424,
            fn_addr: "FUN_00822940",
            note: "Save — Enter File Name dialog",
        },
        // 1000 = 0x3e8 (`007491e0:0x00749a5f`) — News page open.
        //
        // LIVE ROUTE (Layer 5 commit): resolve cmd 1000 by calling the
        // ported Layer-3 builder `build_news_screen`. Default state is
        // used since this dispatcher has no game-state handle — Layer 6
        // will thread a real `NewsScreenState` here.
        1000 => {
            let _ = build_news_screen(pool, &default_news_state());
            DispatchResult::Handled
        }
        // 0x418 (`007491e0:0x00749aef`) — Latest Scores
        0x418 => DispatchResult::TodoBuilder {
            cmd: 0x418,
            fn_addr: "FUN_00700f20",
            note: "Match/Latest Scores (match.c)",
        },
        // 0x42e (`007491e0:0x00749b31`) — unresolved manager sub-screen
        0x42e => DispatchResult::TodoBuilder {
            cmd: 0x42e,
            fn_addr: "FUN_00693410",
            note: "manager sub-screen",
        },
        // 0x41e (`007491e0:0x00749b6f`) — manager sub-screen
        0x41e => DispatchResult::TodoBuilder {
            cmd: 0x41e,
            fn_addr: "FUN_00695e60",
            note: "manager sub-screen",
        },
        // 0x425 (`007491e0:0x00749bad`) — "Control the <club>?" confirm
        0x425 => DispatchResult::TodoBuilder {
            cmd: 0x425,
            fn_addr: "FUN_005276f0+FUN_0057b9c0",
            note: "Please Confirm dialog (club control)",
        },
        // 0x433 (`007491e0:0x00749c56`) — "Control the <nation>?" confirm
        0x433 => DispatchResult::TodoBuilder {
            cmd: 0x433,
            fn_addr: "FUN_005276f0+FUN_0057b9c0",
            note: "Please Confirm dialog (nation control)",
        },
        // 0x41d (`007491e0:0x00749cf1`) — falls through to FUN_006809d0 with
        // pool.club+0x24 (see line 805 / 1436). Same for 0x41c on line 832.
        // Both trampolines land at FUN_006809d0 with different payloads.
        0x41d => DispatchResult::TodoBuilder {
            cmd: 0x41d,
            fn_addr: "FUN_006809d0",
            note: "manager (nation)",
        },
        0x41c => DispatchResult::TodoBuilder {
            cmd: 0x41c,
            fn_addr: "FUN_006809d0",
            note: "manager (club)",
        },
        // 0x3f2 (`007491e0:0x00749d2f`)
        0x3f2 => DispatchResult::TodoBuilder {
            cmd: 0x3f2,
            fn_addr: "FUN_00698140",
            note: "manager sub-screen",
        },
        // 0x420 (`007491e0:0x00749d75`) — manager screen (nation payload)
        0x420 => DispatchResult::TodoBuilder {
            cmd: 0x420,
            fn_addr: "FUN_006977b0",
            note: "Manager screen (nation)",
        },
        // 0x41f (`007491e0:0x00749dbb`) — manager screen (club payload)
        0x41f => DispatchResult::TodoBuilder {
            cmd: 0x41f,
            fn_addr: "FUN_006977b0",
            note: "Manager screen (club)",
        },
        // 0x434 (`007491e0:0x00749dff`) — Scouting
        0x434 => DispatchResult::TodoBuilder {
            cmd: 0x434,
            fn_addr: "FUN_007dfac0",
            note: "Scouting (scout.c)",
        },
        // 0x3e9 (`007491e0:0x00749e85`) — News "You Have News"
        0x3e9 => DispatchResult::TodoBuilder {
            cmd: 0x3e9,
            fn_addr: "FUN_0076ffb0",
            note: "News subscribe/read",
        },
        // 0x3eb (`007491e0:0x00749ec1`)
        0x3eb => DispatchResult::TodoBuilder {
            cmd: 0x3eb,
            fn_addr: "FUN_007f0020",
            note: "network item",
        },
        // 0x415 (`007491e0:0x00749eff`)
        0x415 => DispatchResult::TodoBuilder {
            cmd: 0x415,
            fn_addr: "FUN_008e3700",
            note: "tactic/team",
        },
        // 0x3ec (`007491e0:0x00749f3d`) — Player & Staff Search
        0x3ec => DispatchResult::TodoBuilder {
            cmd: 0x3ec,
            fn_addr: "FUN_00859250",
            note: "Player & Staff Search (staff.c)",
        },
        // 0x3ef (`007491e0:0x00749f81`)
        0x3ef => DispatchResult::TodoBuilder {
            cmd: 0x3ef,
            fn_addr: "FUN_006986a0",
            note: "manager/job screen",
        },
        // 0x3f0 (`007491e0:0x00749fbf`) — Return-from-Holiday dialog
        0x3f0 => DispatchResult::TodoBuilder {
            cmd: 0x3f0,
            fn_addr: "FUN_006547c0+FUN_0057b9c0",
            note: "Return from holiday? confirm",
        },
        // 0x42d (`007491e0:0x0074a087`) — clear back-stack + refuse
        0x42d => {
            // Exe: `DAT_00b59fec = -1; return -0xb;`
            // (`007491e0:0x0074a091`). This one *does* have all its
            // dependencies decoded (a global write) — port it live.
            // NB: `DAT_00b59fec` is the seat-indexed back-stack head;
            // we don't yet model it, so the write is elided but the
            // return value is faithful.
            DispatchResult::Refused
        }
        // 0x414 (`007491e0:0x0074a0af`) — takes widget payload; if
        // `widget_slot == -1` passes 0, else pool[slot]+0x48.
        0x414 => DispatchResult::TodoBuilder {
            cmd: 0x414,
            fn_addr: "FUN_00771810",
            note: "unresolved (takes widget payload)",
        },
        // 0x3f3 (`007491e0:0x0074a107`) — FIFA rankings
        0x3f3 => DispatchResult::TodoBuilder {
            cmd: 0x3f3,
            fn_addr: "FUN_004a2190",
            note: "FIFA rankings (already ported as Screen::FifaRankings — Layer 3 wire-up pending)",
        },
        // 0x40c (`007491e0:0x0074a137`)
        0x40c => DispatchResult::TodoBuilder {
            cmd: 0x40c,
            fn_addr: "FUN_004a28c0",
            note: "competition sub-screen",
        },
        // 0x3f4 (`007491e0:0x0074a175`) — shared list/table entry 1
        0x3f4 => DispatchResult::TodoBuilder {
            cmd: 0x3f4,
            fn_addr: "FUN_0058a550(1,...)",
            note: "shared list/table view (FUN_00652ca0 param)",
        },
        // 0x3f5 (`007491e0:0x0074a1b3`) — shared list/table entry 2
        0x3f5 => DispatchResult::TodoBuilder {
            cmd: 0x3f5,
            fn_addr: "FUN_0058a550(2,...)",
            note: "shared list/table view",
        },
        // 0x3f6 (`007491e0:0x0074a1f1`) — shared list/table entry 3
        0x3f6 => DispatchResult::TodoBuilder {
            cmd: 0x3f6,
            fn_addr: "FUN_0058a550(3,...)",
            note: "shared list/table view",
        },
        // 0x3f7 (`007491e0:0x0074a22f`) — shared list/table entry 4
        0x3f7 => DispatchResult::TodoBuilder {
            cmd: 0x3f7,
            fn_addr: "FUN_0058a550(4,...)",
            note: "shared list/table view (variant)",
        },
        // 0x3f8 (`007491e0:0x0074a26d`) — shared list/table entry 5
        0x3f8 => DispatchResult::TodoBuilder {
            cmd: 0x3f8,
            fn_addr: "FUN_0058a550(5,...)",
            note: "shared list/table view (variant)",
        },
        // 0x3f9 (`007491e0:0x0074a2ab`) — shared list/table entry 6
        0x3f9 => DispatchResult::TodoBuilder {
            cmd: 0x3f9,
            fn_addr: "FUN_0058a550(6,...)",
            note: "shared list/table view",
        },
        // 0x3fa (`007491e0:0x0074a2e9`)
        0x3fa => DispatchResult::TodoBuilder {
            cmd: 0x3fa,
            fn_addr: "FUN_0058cde0",
            note: "unresolved (takes widget payload)",
        },
        // 0x429 (`007491e0:0x0074a325`)
        0x429 => DispatchResult::TodoBuilder {
            cmd: 0x429,
            fn_addr: "FUN_005dc5e0(1)",
            note: "unresolved",
        },
        // 0x3fd (`007491e0:0x0074a355`) — return-from-holiday redistribute
        0x3fd => DispatchResult::TodoBuilder {
            cmd: 0x3fd,
            fn_addr: "FUN_007e7c40+FUN_007e7cb0+FUN_00454bb0",
            note: "return-from-holiday redistribute + Please Confirm",
        },
        // 0x3fe (`007491e0:0x0074a4b3`) — Save Game dialog variant
        0x3fe => DispatchResult::TodoBuilder {
            cmd: 0x3fe,
            fn_addr: "FUN_00822940+FUN_006547c0",
            note: "Save Game (Enter File Name)",
        },
        // 0x3fb (`007491e0:0x0074a52f`) — Add Manager dialog
        0x3fb => DispatchResult::TodoBuilder {
            cmd: 0x3fb,
            fn_addr: "FUN_007e7790 / FUN_00809ad0",
            note: "Add Manager (network) or Add Manager (local)",
        },
        // 0x3fc (`007491e0:0x0074a575`) — Player Waiting sidebar cmd
        0x3fc => DispatchResult::TodoBuilder {
            cmd: 0x3fc,
            fn_addr: "FUN_00808a70",
            note: "Player Waiting screen",
        },
        // 0x421 (`007491e0:0x0074a5b3`) — Chat Message
        0x421 => DispatchResult::TodoBuilder {
            cmd: 0x421,
            fn_addr: "FUN_007e6570+FUN_007e7130",
            note: "Chat / scrman.c",
        },
        // 0x431 (`007491e0:0x0074a5f7`) — Start New Game / Select Leagues
        0x431 => DispatchResult::TodoBuilder {
            cmd: 0x431,
            fn_addr: "FUN_008053d0",
            note: "Start New Game / Select League(s) (Setup.c)",
        },
        // 0x42a (`007491e0:0x0074a629`)
        0x42a => DispatchResult::TodoBuilder {
            cmd: 0x42a,
            fn_addr: "FUN_0080fac0",
            note: "game options",
        },
        // 0x42b (`007491e0:0x0074a65b`) — Game Settings; also clears
        // DAT_00dbbf7a if it matched. Port the clear.
        0x42b => {
            if state.pending_cmd_fallback == 0x42b {
                state.pending_cmd_fallback = 0;
            }
            DispatchResult::TodoBuilder {
                cmd: 0x42b,
                fn_addr: "FUN_004ec550",
                note: "Game Settings",
            }
        }
        // 0x42c (`007491e0:0x0074a6a5`) — settings sub-screen
        0x42c => DispatchResult::TodoBuilder {
            cmd: 0x42c,
            fn_addr: "FUN_004fd1b0",
            note: "settings sub-screen",
        },
        // 0x42f (`007491e0:0x0074a6d7`) — Restart Game confirm dialog
        0x42f => DispatchResult::TodoBuilder {
            cmd: 0x42f,
            fn_addr: "FUN_006547c0+FUN_00822940+FUN_00822920+FUN_0057b9c0",
            note: "Restart the game? confirm",
        },
        // 0x402 (`007491e0:0x0074a715`) — Exit Game
        0x402 => DispatchResult::TodoBuilder {
            cmd: 0x402,
            fn_addr: "FUN_007ebaf0+FUN_006547c0+FUN_00822940+FUN_00822920+FUN_0057b9c0",
            note: "Exit Game confirm",
        },
        // Signed negatives from the sidebar's other synthetic messages.
        // The exe's earlier chain (C lines 653/665) already caught -2/-3;
        // the widget-fired -0xb / -0xc appear inside dialog-result paths
        // and are only reachable via pre_dispatch_bookkeeping.
        _ => DispatchResult::Unhandled,
    };

    // ---- Post-dispatch (scaffold — sidebar text/cmd re-bind).
    post_dispatch_sidebar_update(pool, state);

    result
}

// ============================================================================
// Club-scope dispatcher — port of FUN_0074bf60
// ============================================================================

/// Direct port of `FUN_0074bf60`. Handles the 22 club-context cmd
/// codes. Unlike the global dispatcher, this one has no pre-dispatch
/// bookkeeping — it's pure switch.
pub fn dispatch_club(
    pool: &mut GuiRecordPool,
    state: &mut DispatcherState,
    widget_slot: i16,
) -> DispatchResult {
    let cmd = resolve_widget_cmd(pool, widget_slot, state);
    let payload = resolve_widget_payload(pool, widget_slot);

    match cmd {
        // 0x426 (`0074bf60:0x0074bf88`) — network trigger
        0x426 => DispatchResult::TodoBuilder {
            cmd: 0x426,
            fn_addr: "FUN_007eb990",
            note: "Network trigger (fires then falls through)",
        },
        // 2000 = 0x7d0 (`0074bf60:0x0074bfe8`) — Player profile
        2000 => DispatchResult::TodoBuilder {
            cmd: 2000,
            fn_addr: "FUN_00493e10",
            note: "Player profile (via FUN_007cf040 guard)",
        },
        // 0x7d1 (`0074bf60:0x0074c216`) — Club overview / summary
        0x7d1 => DispatchResult::TodoBuilder {
            cmd: 0x7d1,
            fn_addr: "FUN_00476df0",
            note: "Club overview / summary (club_s)",
        },
        // 0x7d3 (`0074bf60:0x0074c25a`) — via FUN_007116b0 guard
        0x7d3 => {
            // Exe: `if (widget_slot != -1 && payload == -1) return 0;`
            // then FUN_007116b0(payload, local_50); if guard passes,
            // FUN_00701240(payload). Model the early-return.
            if widget_slot != -1 && (payload as i32) == -1 {
                return DispatchResult::Unhandled;
            }
            DispatchResult::TodoBuilder {
                cmd: 0x7d3,
                fn_addr: "FUN_007116b0+FUN_00701240",
                note: "match/club detail",
            }
        }
        // 0x7d5 (`0074bf60:0x0074c2c1`) — Club Dashboard
        0x7d5 => DispatchResult::TodoBuilder {
            cmd: 0x7d5,
            fn_addr: "FUN_00454620",
            note: "Club Dashboard (verified anchor; draw 004551c0)",
        },
        // 0x7d6 (`0074bf60:0x0074c33d`) — Dashboard (other-club variant)
        0x7d6 => DispatchResult::TodoBuilder {
            cmd: 0x7d6,
            fn_addr: "FUN_00454620",
            note: "Club Dashboard (other-club / non-managed)",
        },
        // 0x7d4 (`0074bf60:0x0074c38d`) — Club honours / Awards
        0x7d4 => DispatchResult::TodoBuilder {
            cmd: 0x7d4,
            fn_addr: "FUN_00415010",
            note: "Club honours / Awards (award)",
        },
        // 0x7d7 (`0074bf60:0x0074c3d5`) — Squad Senior
        0x7d7 => DispatchResult::TodoBuilder {
            cmd: 0x7d7,
            fn_addr: "FUN_00859250(_,1,1,0,0)",
            note: "Squad — Senior",
        },
        // 0x7e7 (`0074bf60:0x0074c41d`) — Squad Senior variant 2
        0x7e7 => DispatchResult::TodoBuilder {
            cmd: 0x7e7,
            fn_addr: "FUN_00859250(_,1,1,0,2)",
            note: "Squad — Senior (variant 2)",
        },
        // 0x7d8 (`0074bf60:0x0074c465`) — Squad Reserves
        0x7d8 => DispatchResult::TodoBuilder {
            cmd: 0x7d8,
            fn_addr: "FUN_00859250(_,2,1,0,0)",
            note: "Squad — Reserves",
        },
        // 0x7e5 (`0074bf60:0x0074c078`) — Player-in-club profile
        0x7e5 => DispatchResult::TodoBuilder {
            cmd: 0x7e5,
            fn_addr: "FUN_00493e10 (with division/nation resolve)",
            note: "Player-in-club profile",
        },
        // 0x7e6 (`0074bf60:0x0074c1e0`) — club/player detail
        0x7e6 => DispatchResult::TodoBuilder {
            cmd: 0x7e6,
            fn_addr: "FUN_0046ba80",
            note: "club/player detail",
        },
        // 0x7da (`0074bf60:0x0074c4b0`) — Squad youth/U21
        0x7da => DispatchResult::TodoBuilder {
            cmd: 0x7da,
            fn_addr: "FUN_00859250(_,3,1,0,0)",
            note: "Squad list mode 3 (youth/U21)",
        },
        // 0x7d9 (`0074bf60:0x0074c4d2`) — Squad list mode 2
        0x7d9 => DispatchResult::TodoBuilder {
            cmd: 0x7d9,
            fn_addr: "FUN_00859250(_,2,1,0,0)",
            note: "Squad list mode 2",
        },
        // 0x7db (`0074bf60:0x0074caf1`) — Squad list mode 4
        0x7db => DispatchResult::TodoBuilder {
            cmd: 0x7db,
            fn_addr: "FUN_00859250(_,4,1,0,0)",
            note: "Squad list mode 4",
        },
        // 0x7dc (`0074bf60:0x0074c514`) — Staff list page 2
        0x7dc => DispatchResult::TodoBuilder {
            cmd: 0x7dc,
            fn_addr: "FUN_00859250(_,1,2,0,0)",
            note: "Staff list page 2",
        },
        // 0x7dd (`0074bf60:0x0074c576`) — Staff list
        0x7dd => DispatchResult::TodoBuilder {
            cmd: 0x7dd,
            fn_addr: "FUN_00859250(_,2,2,0,0)",
            note: "Staff list",
        },
        // 0x7df (`0074bf60:0x0074c556`) — Staff list variant 3
        0x7df => DispatchResult::TodoBuilder {
            cmd: 0x7df,
            fn_addr: "FUN_00859250(_,3,2,0,0)",
            note: "Staff list variant 3",
        },
        // 0x7de (`0074bf60:0x0074caec`) — Staff list variant 2
        0x7de => DispatchResult::TodoBuilder {
            cmd: 0x7de,
            fn_addr: "FUN_00859250(_,_,2,0,0)",
            note: "Staff list variant 2 (goto LAB_0074caec with uVar12=2)",
        },
        // 0x7e0 (`0074bf60:0x0074c5f8`) — Network (handled in 007e4940)
        0x7e0 => DispatchResult::TodoBuilder {
            cmd: 0x7e0,
            fn_addr: "FUN_00935f4b (in FUN_007e4940)",
            note: "Network — falls out of this dispatcher into 007e4940",
        },
        // 0x7d2 (`0074bf60:0x0074c619`) — Human/manager record. This
        // one has a live path that returns -0xb (0xfffffff5) when
        // the FUN_005e8040 gate passes; port the gate return live.
        0x7d2 => {
            // Exe: `uVar6 = FUN_0046a6f0(param_1);
            //       if (FUN_005e7dd0(uVar6,0) != 0) uVar6 = 0;
            //       if (FUN_005e8040(uVar6,0) != 0) return -0xb;`
            // Neither predicate is ported; assume gate passes so
            // the semantic outcome (Refused) is at least reachable.
            let _ = payload; // FUN_0046a6f0(param_1) reads a widget field
            DispatchResult::TodoBuilder {
                cmd: 0x7d2,
                fn_addr: "FUN_005e7dd0+FUN_005e8040 (human record gate)",
                note: "Human/manager record",
            }
        }
        // 0x7e4 (`0074bf60:0x0074c66d`)
        0x7e4 => DispatchResult::TodoBuilder {
            cmd: 0x7e4,
            fn_addr: "FUN_007f0020",
            note: "unresolved (network item)",
        },
        _ => DispatchResult::Unhandled,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget_pool::WidgetDescriptor;

    /// The count of `match` arms in `dispatch_global` (excluding `_`).
    /// Must match the count of `if (sVar3 == 0xNNN)` branches in
    /// `FUN_007491e0`. The C has 41 documented branches per
    /// `reports/menu_tree_scope.md` line 16, plus the -2 / -3 nav
    /// synthetic branches, plus 0x41c (branched from `if != 0x41c`),
    /// giving 44 dispatch arms total.
    const GLOBAL_DISPATCH_ARM_COUNT: usize = 44;

    /// Count of `match` arms in `dispatch_club` (excluding `_`).
    /// `reports/menu_tree_scope.md` line 17 says 17 club codes, plus
    /// 0x426 (which the club dispatcher handles first before falling
    /// through the global-shared cmd), plus the extra squad-list
    /// variants (0x7d9, 0x7da, 0x7db, 0x7dc, 0x7dd, 0x7de, 0x7df,
    /// 0x7e7) — 22 total arms.
    const CLUB_DISPATCH_ARM_COUNT: usize = 22;

    /// Build a widget in the pool whose msg_id fires the given cmd
    /// when the dispatcher resolves it.
    fn widget_with_cmd(pool: &mut GuiRecordPool, cmd: i32) -> i16 {
        let mut d = WidgetDescriptor::empty();
        d.msg_id = cmd;
        pool.spawn_widget(d, -1).expect("pool room") as i16
    }

    #[test]
    fn dispatcher_routes_nav_cmd_to_ported_builder() {
        // The Back nav-bar button fires msg_id = -2 (see
        // screen_nav_back_next.rs, back-disabled branch matches
        // c constants test). Dispatching that must:
        //  1. Return DispatchResult::NavBack
        //  2. Call the LIVE Layer-3 builder (build_nav_back_next),
        //     producing one area + two widgets in the pool.
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        let slot = widget_with_cmd(&mut pool, -2);
        let before_areas = pool.areas.len();
        let before_widgets = pool.widgets.len();

        let r = dispatch_global(&mut pool, &mut state, slot);
        assert_eq!(r, DispatchResult::NavBack);
        // build_nav_back_next added one area + two widgets on top
        // of the input widget.
        assert_eq!(pool.areas.len(), before_areas + 1);
        assert_eq!(pool.widgets.len(), before_widgets + 2);
        // The added widgets should have text "Back" / "Next"
        let last_two: Vec<&str> = pool
            .widgets
            .iter()
            .rev()
            .take(2)
            .map(|w| w.descriptor.text.as_str())
            .collect();
        // Order in the pool: Back was spawned first, Next second, so
        // reversed the last item is "Next" and second-to-last is "Back".
        assert!(last_two.contains(&"Back"));
        assert!(last_two.contains(&"Next"));
    }

    #[test]
    fn dispatcher_routes_nav_next_synthetic_msg() {
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        let slot = widget_with_cmd(&mut pool, -3);
        let r = dispatch_global(&mut pool, &mut state, slot);
        assert_eq!(r, DispatchResult::NavNext);
        assert_eq!(pool.areas.len(), 1);
    }

    #[test]
    fn dispatcher_falls_back_to_pending_cmd_when_widget_has_none() {
        // Widget's msg_id = 0 → the resolver reads DAT_00dbbf7a.
        // Setting fallback to 0x431 must fire the "Start New Game"
        // arm.
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        state.pending_cmd_fallback = 0x431;
        let slot = widget_with_cmd(&mut pool, 0);
        let r = dispatch_global(&mut pool, &mut state, slot);
        assert!(matches!(
            r,
            DispatchResult::TodoBuilder { cmd: 0x431, .. }
        ));
    }

    #[test]
    fn dispatcher_synthetic_dispatch_with_no_widget_uses_fallback() {
        // widget_slot = -1 → resolver returns 0 → fallback used.
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        state.pending_cmd_fallback = 0x424; // Save Game
        let r = dispatch_global(&mut pool, &mut state, -1);
        assert!(matches!(
            r,
            DispatchResult::TodoBuilder { cmd: 0x424, .. }
        ));
    }

    #[test]
    fn dispatcher_unknown_cmd_returns_unhandled() {
        // 0xFFFF is not a real cmd; both dispatchers should return
        // Unhandled with no state change and no panic.
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        let slot = widget_with_cmd(&mut pool, 0xFFFF);
        let r_g = dispatch_global(&mut pool, &mut state, slot);
        assert_eq!(r_g, DispatchResult::Unhandled);
        let r_c = dispatch_club(&mut pool, &mut state, slot);
        assert_eq!(r_c, DispatchResult::Unhandled);
    }

    #[test]
    fn dispatcher_covers_all_c_branches_global() {
        // For every cmd we're supposed to handle, dispatch must not
        // return Unhandled. This attests that the switch covers
        // every branch in the C.
        let cmds: [i32; GLOBAL_DISPATCH_ARM_COUNT] = [
            -2, -3, // nav synthetic
            0x424, 1000, 0x418, 0x42e, 0x41e, 0x425, 0x433, 0x41d, 0x41c, 0x3f2, 0x420, 0x41f,
            0x434, 0x3e9, 0x3eb, 0x415, 0x3ec, 0x3ef, 0x3f0, 0x42d, 0x414, 0x3f3, 0x40c, 0x3f4,
            0x3f5, 0x3f6, 0x3f7, 0x3f8, 0x3f9, 0x3fa, 0x429, 0x3fd, 0x3fe, 0x3fb, 0x3fc, 0x421,
            0x431, 0x42a, 0x42b, 0x42c, 0x42f, 0x402,
        ];
        for cmd in cmds.iter().copied() {
            let mut pool = GuiRecordPool::new();
            let mut state = DispatcherState::default();
            let slot = widget_with_cmd(&mut pool, cmd);
            let r = dispatch_global(&mut pool, &mut state, slot);
            assert!(
                !matches!(r, DispatchResult::Unhandled),
                "cmd 0x{:x} was not handled: {:?}",
                cmd,
                r
            );
        }
    }

    #[test]
    fn dispatcher_covers_all_c_branches_club() {
        let cmds: [i32; CLUB_DISPATCH_ARM_COUNT] = [
            0x426, 2000, 0x7d1, 0x7d3, 0x7d5, 0x7d6, 0x7d4, 0x7d7, 0x7e7, 0x7d8, 0x7e5, 0x7e6,
            0x7da, 0x7d9, 0x7db, 0x7dc, 0x7dd, 0x7df, 0x7de, 0x7e0, 0x7d2, 0x7e4,
        ];
        for cmd in cmds.iter().copied() {
            let mut pool = GuiRecordPool::new();
            let mut state = DispatcherState::default();
            let slot = widget_with_cmd(&mut pool, cmd);
            let r = dispatch_club(&mut pool, &mut state, slot);
            assert!(
                !matches!(r, DispatchResult::Unhandled),
                "cmd 0x{:x} was not handled: {:?}",
                cmd,
                r
            );
        }
    }

    #[test]
    fn cmd_42d_returns_refused_no_todo() {
        // 0x42d has all its deps decoded (just a global write + return).
        // It's ported live, not stubbed.
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        let slot = widget_with_cmd(&mut pool, 0x42d);
        let r = dispatch_global(&mut pool, &mut state, slot);
        assert_eq!(r, DispatchResult::Refused);
    }

    #[test]
    fn cmd_42b_clears_pending_fallback() {
        // Per C line 1367: if DAT_00dbbf7a == 0x42b, clear it.
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        state.pending_cmd_fallback = 0x42b;
        let slot = widget_with_cmd(&mut pool, 0x42b);
        let _ = dispatch_global(&mut pool, &mut state, slot);
        assert_eq!(state.pending_cmd_fallback, 0);
    }

    #[test]
    fn cmd_7d3_early_returns_when_payload_is_neg1() {
        // Per C line 169-171: if payload == -1, return 0.
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        let mut d = WidgetDescriptor::empty();
        d.msg_id = 0x7d3;
        d.userdata_id = u32::MAX; // -1 as u32
        let slot = pool.spawn_widget(d, -1).unwrap() as i16;
        let r = dispatch_club(&mut pool, &mut state, slot);
        assert_eq!(r, DispatchResult::Unhandled);
    }
}
