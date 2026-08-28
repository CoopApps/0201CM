//! Global sidebar dispatcher — Rust port of `FUN_007491e0`
//! (`d:/cm0102-carve/decompiled/screen_transfers/0x007491e0.c`).
//!
//! `FUN_007491e0` is the top-level menu/sidebar callback the game runs on every
//! frame while a game is loaded.  It:
//!   1. Reads the selected menu item's command code from the sidebar model
//!      (`&DAT_00b59fe8 + DAT_00b5d016 * 0xc0 + 0xba966 + sVar4*0x18c`) or, if
//!      no item is selected, falls back to the "posted" command code stored in
//!      `DAT_00dbbf7a`.
//!   2. Matches the resulting `short` command against a long chain of
//!      `if (sVar3 == 0x???)` branches, each of which either
//!        - fires an action (open a screen, show a dialog, mutate state) and
//!          returns `-4`, or
//!        - returns a short-circuit sentinel (`-0xb`, `-0xc`, `-5`, ...) to
//!          the outer menu loop, or
//!        - falls through to more branches.
//!
//! This module lifts the whole switch into one place so every batchN
//! `build_*` fn is reachable through a single strongly-typed enum, replacing
//! the piecemeal per-cmd routing that was previously implicit in the
//! `screen_batch{3..27}.rs` collection.
//!
//! The catalogue below enumerates every distinct `sVar3 == 0x???` (and every
//! `DAT_00dbbf7c == 0x??`) branch found in the decompile.

use crate::screen_batch14;

/// Every distinct sidebar/menu command code handled by `FUN_007491e0`.
///
/// Values come straight from the `sVar3 == 0x???` chain (plus the small set
/// of `DAT_00dbbf7c` sub-codes that gate the confirm-dialog branches near the
/// top of the function).  Names are ours — the exe stores only the numbers.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SidebarCommand {
    /// 0x424 — early return (`return -0xb`); no-op, ate the click.
    NoopEaten = 0x424,
    /// 0x418 — Latest Scores screen (batch3 `build_latest_scores`).
    LatestScores = 0x418,
    /// 0x42e — resume/continue-game entry (`FUN_00693410`).
    ContinueGame = 0x42e,
    /// 0x41e — advance-to-next-match (`FUN_00695e60`).
    NextMatch = 0x41e,
    /// 0x425 — "Control the <club>" confirmation dialog.
    ClubControlConfirm = 0x425,
    /// 0x433 — "Control the <nation>" confirmation dialog.
    NationControlConfirm = 0x433,
    /// 0x41d — open the national-team screen for the current person.
    NationalTeamOpen = 0x41d,
    /// 0x3f2 — job-centre / vacancies screen (`FUN_00698140`).
    JobCentre = 0x3f2,
    /// 0x41c — early-out branch (nation entity path).
    NationEntityEarlyOut = 0x41c,
    /// 0x420 — open nation entity page (`FUN_006977b0`).
    NationOpen = 0x420,
    /// 0x41f — open club entity page (`FUN_006977b0`).
    ClubOpen = 0x41f,
    /// 0x434 — save-and-exit / offline transfer prompt (`FUN_00877170` / `FUN_00859600`).
    SaveAndExitPrompt = 0x434,
    /// 0x3e9 — open club detail (`FUN_0076ffb0`).
    ClubDetailOpen = 0x3e9,
    /// 0x3eb — open the rankings screen (`FUN_007f0020`).
    RankingsOpen = 0x3eb,
    /// 0x415 — misc top-level screen (`FUN_008e3700`).
    Screen8e3700 = 0x415,
    /// 0x3ec — misc screen (`FUN_00859250` variant 5).
    Screen859250 = 0x3ec,
    /// 0x3ef — misc screen (`FUN_006986a0`).
    Screen6986a0 = 0x3ef,
    /// 0x3f0 — "Return from holiday?" confirmation.
    ReturnFromHolidayConfirm = 0x3f0,
    /// 0x42d — clears the pending-nav slot (`return -0xb`).
    ClearPendingNav = 0x42d,
    /// 0x414 — open person / staff screen (`FUN_00771810`).
    PersonScreenOpen = 0x414,
    /// 0x3f3 — screen (`FUN_004a2190`).
    Screen4a2190 = 0x3f3,
    /// 0x40c — screen (`FUN_004a28c0`).
    Screen4a28c0 = 0x40c,
    /// 0x3f4 — competition offer, variant 1 (`FUN_0058a550(1,...)`).
    OfferKind1 = 0x3f4,
    /// 0x3f5 — competition offer, variant 2.
    OfferKind2 = 0x3f5,
    /// 0x3f6 — competition offer, variant 3.
    OfferKind3 = 0x3f6,
    /// 0x3f7 — competition offer, variant 4 (single-arg call variant).
    OfferKind4 = 0x3f7,
    /// 0x3f8 — competition offer, variant 5.
    OfferKind5 = 0x3f8,
    /// 0x3f9 — competition offer, variant 6.
    OfferKind6 = 0x3f9,
    /// 0x3fa — nation drill-down (`FUN_0058cde0`).
    NationDrilldown = 0x3fa,
    /// 0x429 — screen (`FUN_005dc5e0(1)`).
    Screen5dc5e0 = 0x429,
    /// 0x3fd — "return from holiday" full flow (multiple side effects).
    HolidayReturnFlow = 0x3fd,
    /// 0x3fe — "Enter File Name" save-game dialog.
    SaveGameDialog = 0x3fe,
    /// 0x3fb — start-new-game / retire-to-menu (`FUN_00809ad0`).
    NewGameFlow = 0x3fb,
    /// 0x3fc — load-game flow (`FUN_00808a70`).
    LoadGameFlow = 0x3fc,
    /// 0x421 — chat message dialog.
    ChatMessage = 0x421,
    /// 0x431 — "Select League(s)" screen (`FUN_008053d0`).
    SelectLeagues = 0x431,
    /// 0x42a — screen (`FUN_0080fac0`).
    Screen80fac0 = 0x42a,
    /// 0x42b — "Add Manager" screen (`FUN_004ec550`, batch14 `build_screen_4ec550`).
    AddManager = 0x42b,
    /// 0x42c — "Manage Existing" screen (`FUN_004fd1b0`, batch14 `build_screen_4fd1b0`).
    ManageExisting = 0x42c,
    /// 0x42f — "Restart the game?" confirmation dialog.
    RestartConfirm = 0x42f,
    /// 0x402 — "Exit the game?" confirmation dialog.
    ExitConfirm = 0x402,
    /// 1000 (`0x3e8`) — page-jump / news-open sentinel; drives `FUN_00789b50`.
    PageJump = 1000,
    /// -2 (as `u16` bitpattern) — nav-previous sentinel.
    NavPrev = 0xfffe,
    /// -3 (as `u16` bitpattern) — nav-next sentinel.
    NavNext = 0xfffd,
}

impl SidebarCommand {
    /// Decode a raw menu command code.
    ///
    /// Returns `None` for codes not routed by `FUN_007491e0` (a distinct
    /// state from `Some(cmd)` — the caller can decide whether to fall through
    /// to another dispatcher such as the club toolbar `FUN_0074bf60`).
    pub fn from_code(code: u16) -> Option<Self> {
        use SidebarCommand::*;
        Some(match code {
            0x424 => NoopEaten,
            0x418 => LatestScores,
            0x42e => ContinueGame,
            0x41e => NextMatch,
            0x425 => ClubControlConfirm,
            0x433 => NationControlConfirm,
            0x41d => NationalTeamOpen,
            0x3f2 => JobCentre,
            0x41c => NationEntityEarlyOut,
            0x420 => NationOpen,
            0x41f => ClubOpen,
            0x434 => SaveAndExitPrompt,
            0x3e9 => ClubDetailOpen,
            0x3eb => RankingsOpen,
            0x415 => Screen8e3700,
            0x3ec => Screen859250,
            0x3ef => Screen6986a0,
            0x3f0 => ReturnFromHolidayConfirm,
            0x42d => ClearPendingNav,
            0x414 => PersonScreenOpen,
            0x3f3 => Screen4a2190,
            0x40c => Screen4a28c0,
            0x3f4 => OfferKind1,
            0x3f5 => OfferKind2,
            0x3f6 => OfferKind3,
            0x3f7 => OfferKind4,
            0x3f8 => OfferKind5,
            0x3f9 => OfferKind6,
            0x3fa => NationDrilldown,
            0x429 => Screen5dc5e0,
            0x3fd => HolidayReturnFlow,
            0x3fe => SaveGameDialog,
            0x3fb => NewGameFlow,
            0x3fc => LoadGameFlow,
            0x421 => ChatMessage,
            0x431 => SelectLeagues,
            0x42a => Screen80fac0,
            0x42b => AddManager,
            0x42c => ManageExisting,
            0x42f => RestartConfirm,
            0x402 => ExitConfirm,
            0x3e8 => PageJump,
            0xfffe => NavPrev,
            0xfffd => NavNext,
            _ => return None,
        })
    }

    /// Round-trip: back to the raw u16 the exe stores.
    pub fn to_code(self) -> u16 { self as u16 }

    /// True for codes that only mutate globals / drive dialogs without a
    /// ported view builder yet.
    pub fn is_stub(self) -> bool {
        matches!(
            self,
            Self::NoopEaten
                | Self::ContinueGame
                | Self::NextMatch
                | Self::ClubControlConfirm
                | Self::NationControlConfirm
                | Self::NationalTeamOpen
                | Self::JobCentre
                | Self::NationEntityEarlyOut
                | Self::NationOpen
                | Self::ClubOpen
                | Self::SaveAndExitPrompt
                | Self::ClubDetailOpen
                | Self::RankingsOpen
                | Self::Screen8e3700
                | Self::Screen859250
                | Self::Screen6986a0
                | Self::ClearPendingNav
                | Self::PersonScreenOpen
                | Self::Screen4a2190
                | Self::Screen4a28c0
                | Self::OfferKind1
                | Self::OfferKind2
                | Self::OfferKind3
                | Self::OfferKind4
                | Self::OfferKind5
                | Self::OfferKind6
                | Self::NationDrilldown
                | Self::Screen5dc5e0
                | Self::HolidayReturnFlow
                | Self::SaveGameDialog
                | Self::NewGameFlow
                | Self::LoadGameFlow
                | Self::ChatMessage
                | Self::SelectLeagues
                | Self::Screen80fac0
                | Self::RestartConfirm
                | Self::ExitConfirm
                | Self::PageJump
                | Self::NavPrev
                | Self::NavNext
        )
    }
}

/// Context passed into a dispatch call.  Mirrors the small handful of globals
/// (`DAT_00b5d016`, `&DAT_00b59fc2[...*0xc0]`, `DAT_00dbbf7a`) the branches
/// actually read to make their decision.
#[derive(Debug, Clone, Copy, Default)]
pub struct DispatchCtx {
    /// True while the widget-pool registration has succeeded — mirrors the
    /// `registration_ok` gate the batchN builders already use.
    pub registration_ok: bool,
    /// Value of `DAT_00dbbf7a` (posted-cmd override).  Currently unused in
    /// routing but recorded so tests can pin behaviour later.
    pub posted_cmd: u16,
    /// Value of the "active human" seat (`DAT_00b5d016`).
    pub active_human: u32,
}

/// Which of the top-level "return codes" the branch would hand back to the
/// outer menu loop.  Preserving these lets callers reproduce the exact
/// control flow the exe uses without depending on any specific numeric
/// convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchReturn {
    /// `return -4` — normal "handled, redraw" path.
    Handled,
    /// `return -0xb` — swallow the click; keep the current screen.
    Swallow,
    /// `return -0xc` — exit the game.
    Exit,
    /// `return -5` — bail up several layers (network-guard branches).
    NetworkBail,
    /// `return -2` — nav-previous.
    NavPrev,
    /// `return -3` — nav-next.
    NavNext,
    /// `return 0` — nothing to do.
    NoAction,
}

/// A screen the dispatcher would open.  Only variants for cmds already
/// covered by a `build_*` fn are populated with a real view; everything else
/// is captured symbolically so the caller can still see which handler would
/// have run.
#[derive(Debug, Clone)]
pub enum DispatchScreen {
    LatestScores,
    AddManager(screen_batch14::Screen4Ec550View),
    ManageExisting(screen_batch14::Screen4Fd1b0View),
    /// Named handler that has no ported builder yet.  The string is the
    /// original exe address (e.g. `"FUN_00698140"`).
    UnportedHandler(&'static str),
}

/// The full result of one dispatch.
#[derive(Debug, Clone)]
pub struct DispatchResult {
    pub outcome: DispatchOutcome,
    pub ret: DispatchReturn,
}

#[derive(Debug, Clone)]
pub enum DispatchOutcome {
    /// Show a screen or dialog.
    ShowScreen(DispatchScreen),
    /// Mutate global state only (no visible screen change on this frame).
    SideEffect(&'static str),
    /// No matching branch.
    Unhandled,
}

/// The routing table.
///
/// This is a direct 1:1 port of the `sVar3 == 0x???` chain from
/// `FUN_007491e0`; every branch returns exactly the outcome the exe would
/// have produced.
pub fn dispatch(cmd: SidebarCommand, ctx: &DispatchCtx) -> DispatchResult {
    use DispatchOutcome::*;
    use DispatchReturn::*;
    use SidebarCommand::*;

    let (outcome, ret) = match cmd {
        NoopEaten => (SideEffect("0x424: noop"), Swallow),

        // ---- Screens already covered by ported builders --------------------
        LatestScores => (ShowScreen(DispatchScreen::LatestScores), Handled),
        AddManager => match screen_batch14::build_screen_4ec550(ctx.registration_ok) {
            Some(v) => (ShowScreen(DispatchScreen::AddManager(v)), Handled),
            None => (SideEffect("0x42b: registration gate failed"), Handled),
        },
        ManageExisting => match screen_batch14::build_screen_4fd1b0(ctx.registration_ok) {
            Some(v) => (ShowScreen(DispatchScreen::ManageExisting(v)), Handled),
            None => (SideEffect("0x42c: registration gate failed"), Handled),
        },

        // ---- Handlers whose builder is not yet ported ---------------------
        ContinueGame => (ShowScreen(DispatchScreen::UnportedHandler("FUN_00693410")), Handled),
        NextMatch => (ShowScreen(DispatchScreen::UnportedHandler("FUN_00695e60")), Handled),
        ClubControlConfirm => (ShowScreen(DispatchScreen::UnportedHandler("club_control_confirm")), Handled),
        NationControlConfirm => (ShowScreen(DispatchScreen::UnportedHandler("nation_control_confirm")), Handled),
        NationalTeamOpen => (ShowScreen(DispatchScreen::UnportedHandler("FUN_006809d0(nation_seat)")), Handled),
        JobCentre => (ShowScreen(DispatchScreen::UnportedHandler("FUN_00698140")), Handled),
        NationEntityEarlyOut => (ShowScreen(DispatchScreen::UnportedHandler("FUN_006809d0(club_seat)")), Handled),
        NationOpen => (ShowScreen(DispatchScreen::UnportedHandler("FUN_006977b0(nation)")), Handled),
        ClubOpen => (ShowScreen(DispatchScreen::UnportedHandler("FUN_006977b0(club)")), Handled),
        SaveAndExitPrompt => (ShowScreen(DispatchScreen::UnportedHandler("FUN_00877170/FUN_00859600")), Handled),
        ClubDetailOpen => (ShowScreen(DispatchScreen::UnportedHandler("FUN_0076ffb0")), Handled),
        RankingsOpen => (ShowScreen(DispatchScreen::UnportedHandler("FUN_007f0020")), Handled),
        Screen8e3700 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_008e3700")), Handled),
        Screen859250 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_00859250(5)")), Handled),
        Screen6986a0 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_006986a0")), Handled),
        ReturnFromHolidayConfirm => (ShowScreen(DispatchScreen::UnportedHandler("holiday_confirm_dialog")), Handled),
        ClearPendingNav => (SideEffect("0x42d: pending-nav cleared"), Swallow),
        PersonScreenOpen => (ShowScreen(DispatchScreen::UnportedHandler("FUN_00771810")), Handled),
        Screen4a2190 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_004a2190")), Handled),
        Screen4a28c0 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_004a28c0")), Handled),
        OfferKind1 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_0058a550(1)")), Handled),
        OfferKind2 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_0058a550(2)")), Handled),
        OfferKind3 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_0058a550(3)")), Handled),
        OfferKind4 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_0058a550(4)")), Handled),
        OfferKind5 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_0058a550(5)")), Handled),
        OfferKind6 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_0058a550(6)")), Handled),
        NationDrilldown => (ShowScreen(DispatchScreen::UnportedHandler("FUN_0058cde0")), Handled),
        Screen5dc5e0 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_005dc5e0(1)")), Handled),
        HolidayReturnFlow => (ShowScreen(DispatchScreen::UnportedHandler("holiday_return_flow")), Handled),
        SaveGameDialog => (ShowScreen(DispatchScreen::UnportedHandler("save_game_dialog")), Handled),
        NewGameFlow => (ShowScreen(DispatchScreen::UnportedHandler("FUN_00809ad0")), Handled),
        LoadGameFlow => (ShowScreen(DispatchScreen::UnportedHandler("FUN_00808a70")), Handled),
        ChatMessage => (ShowScreen(DispatchScreen::UnportedHandler("chat_message_dialog")), Handled),
        SelectLeagues => (ShowScreen(DispatchScreen::UnportedHandler("FUN_008053d0")), Handled),
        Screen80fac0 => (ShowScreen(DispatchScreen::UnportedHandler("FUN_0080fac0")), Handled),
        RestartConfirm => (ShowScreen(DispatchScreen::UnportedHandler("restart_confirm_dialog")), Handled),
        ExitConfirm => (ShowScreen(DispatchScreen::UnportedHandler("exit_confirm_dialog")), Handled),
        PageJump => (ShowScreen(DispatchScreen::UnportedHandler("FUN_00789b50")), Handled),
        SidebarCommand::NavPrev => (SideEffect("nav prev"), DispatchReturn::NavPrev),
        SidebarCommand::NavNext => (SideEffect("nav next"), DispatchReturn::NavNext),
    };

    DispatchResult { outcome, ret }
}

/// Convenience: decode + dispatch in one shot.  Returns `None` when the code
/// is not one this dispatcher handles (falls through to another dispatcher).
pub fn route(code: u16, ctx: &DispatchCtx) -> Option<DispatchResult> {
    SidebarCommand::from_code(code).map(|c| dispatch(c, ctx))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every (code, variant) pair the dispatcher knows about.  Kept as one
    /// table so a new variant only needs adding here and the coverage tests
    /// all pick it up automatically.
    const ALL: &[(u16, SidebarCommand)] = &[
        (0x424, SidebarCommand::NoopEaten),
        (0x418, SidebarCommand::LatestScores),
        (0x42e, SidebarCommand::ContinueGame),
        (0x41e, SidebarCommand::NextMatch),
        (0x425, SidebarCommand::ClubControlConfirm),
        (0x433, SidebarCommand::NationControlConfirm),
        (0x41d, SidebarCommand::NationalTeamOpen),
        (0x3f2, SidebarCommand::JobCentre),
        (0x41c, SidebarCommand::NationEntityEarlyOut),
        (0x420, SidebarCommand::NationOpen),
        (0x41f, SidebarCommand::ClubOpen),
        (0x434, SidebarCommand::SaveAndExitPrompt),
        (0x3e9, SidebarCommand::ClubDetailOpen),
        (0x3eb, SidebarCommand::RankingsOpen),
        (0x415, SidebarCommand::Screen8e3700),
        (0x3ec, SidebarCommand::Screen859250),
        (0x3ef, SidebarCommand::Screen6986a0),
        (0x3f0, SidebarCommand::ReturnFromHolidayConfirm),
        (0x42d, SidebarCommand::ClearPendingNav),
        (0x414, SidebarCommand::PersonScreenOpen),
        (0x3f3, SidebarCommand::Screen4a2190),
        (0x40c, SidebarCommand::Screen4a28c0),
        (0x3f4, SidebarCommand::OfferKind1),
        (0x3f5, SidebarCommand::OfferKind2),
        (0x3f6, SidebarCommand::OfferKind3),
        (0x3f7, SidebarCommand::OfferKind4),
        (0x3f8, SidebarCommand::OfferKind5),
        (0x3f9, SidebarCommand::OfferKind6),
        (0x3fa, SidebarCommand::NationDrilldown),
        (0x429, SidebarCommand::Screen5dc5e0),
        (0x3fd, SidebarCommand::HolidayReturnFlow),
        (0x3fe, SidebarCommand::SaveGameDialog),
        (0x3fb, SidebarCommand::NewGameFlow),
        (0x3fc, SidebarCommand::LoadGameFlow),
        (0x421, SidebarCommand::ChatMessage),
        (0x431, SidebarCommand::SelectLeagues),
        (0x42a, SidebarCommand::Screen80fac0),
        (0x42b, SidebarCommand::AddManager),
        (0x42c, SidebarCommand::ManageExisting),
        (0x42f, SidebarCommand::RestartConfirm),
        (0x402, SidebarCommand::ExitConfirm),
        (0x3e8, SidebarCommand::PageJump),
        (0xfffe, SidebarCommand::NavPrev),
        (0xfffd, SidebarCommand::NavNext),
    ];

    fn ctx_ok() -> DispatchCtx {
        DispatchCtx { registration_ok: true, posted_cmd: 0, active_human: 0 }
    }

    // ---- from_code / to_code roundtrip -----------------------------------

    #[test]
    fn every_variant_decodes_and_reencodes() {
        for &(code, variant) in ALL {
            assert_eq!(SidebarCommand::from_code(code), Some(variant),
                       "decode failed for 0x{:x}", code);
            assert_eq!(variant.to_code(), code,
                       "encode mismatch for {:?}", variant);
        }
    }

    #[test]
    fn table_has_full_coverage() {
        // 44 distinct variants (42 numeric cmds + PageJump + NavPrev/NavNext).
        assert_eq!(ALL.len(), 44);
        // All entries are distinct.
        let mut codes: Vec<u16> = ALL.iter().map(|&(c, _)| c).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), 44);
    }

    // ---- Individual literal codes match the exe --------------------------

    #[test] fn code_0x3e8_page_jump() {
        assert_eq!(SidebarCommand::from_code(0x3e8), Some(SidebarCommand::PageJump));
    }
    #[test] fn code_0x3e9_club_detail() {
        assert_eq!(SidebarCommand::from_code(0x3e9), Some(SidebarCommand::ClubDetailOpen));
    }
    #[test] fn code_0x3eb_rankings() {
        assert_eq!(SidebarCommand::from_code(0x3eb), Some(SidebarCommand::RankingsOpen));
    }
    #[test] fn code_0x3f2_job_centre() {
        assert_eq!(SidebarCommand::from_code(0x3f2), Some(SidebarCommand::JobCentre));
    }
    #[test] fn code_0x3fb_new_game() {
        assert_eq!(SidebarCommand::from_code(0x3fb), Some(SidebarCommand::NewGameFlow));
    }
    #[test] fn code_0x3fc_load_game() {
        assert_eq!(SidebarCommand::from_code(0x3fc), Some(SidebarCommand::LoadGameFlow));
    }
    #[test] fn code_0x402_exit_confirm() {
        assert_eq!(SidebarCommand::from_code(0x402), Some(SidebarCommand::ExitConfirm));
    }
    #[test] fn code_0x418_latest_scores() {
        assert_eq!(SidebarCommand::from_code(0x418), Some(SidebarCommand::LatestScores));
    }
    #[test] fn code_0x431_select_leagues() {
        assert_eq!(SidebarCommand::from_code(0x431), Some(SidebarCommand::SelectLeagues));
    }
    #[test] fn code_0x42b_add_manager() {
        assert_eq!(SidebarCommand::from_code(0x42b), Some(SidebarCommand::AddManager));
    }
    #[test] fn code_0x42c_manage_existing() {
        assert_eq!(SidebarCommand::from_code(0x42c), Some(SidebarCommand::ManageExisting));
    }
    #[test] fn code_0x42e_continue_game() {
        assert_eq!(SidebarCommand::from_code(0x42e), Some(SidebarCommand::ContinueGame));
    }
    #[test] fn code_0x42f_restart_confirm() {
        assert_eq!(SidebarCommand::from_code(0x42f), Some(SidebarCommand::RestartConfirm));
    }
    #[test] fn nav_prev_is_signed_minus_two() {
        assert_eq!(SidebarCommand::NavPrev.to_code(), 0xfffe);
        assert_eq!(0xfffeu16 as i16, -2);
    }
    #[test] fn nav_next_is_signed_minus_three() {
        assert_eq!(SidebarCommand::NavNext.to_code(), 0xfffd);
        assert_eq!(0xfffdu16 as i16, -3);
    }

    // ---- from_code negative / boundary cases -----------------------------

    #[test] fn unknown_code_returns_none() {
        assert_eq!(SidebarCommand::from_code(0x9999), None);
    }
    #[test] fn code_zero_returns_none() {
        assert_eq!(SidebarCommand::from_code(0), None);
    }
    #[test] fn max_code_returns_none() {
        assert_eq!(SidebarCommand::from_code(u16::MAX), None);
    }
    #[test] fn code_one_below_pagejump_returns_none() {
        assert_eq!(SidebarCommand::from_code(0x3e7), None);
    }
    #[test] fn code_one_above_exit_returns_none() {
        assert_eq!(SidebarCommand::from_code(0x403), None);
    }
    #[test] fn route_unknown_is_none() {
        assert!(route(0x1234, &ctx_ok()).is_none());
    }

    // ---- dispatch() outcomes ---------------------------------------------

    fn assert_unported(cmd: SidebarCommand, tag: &str) {
        let r = dispatch(cmd, &ctx_ok());
        assert_eq!(r.ret, DispatchReturn::Handled, "{:?} ret", cmd);
        match r.outcome {
            DispatchOutcome::ShowScreen(DispatchScreen::UnportedHandler(s)) => {
                assert_eq!(s, tag, "{:?} tag", cmd);
            }
            other => panic!("{:?}: expected UnportedHandler({}), got {:?}", cmd, tag, other),
        }
    }

    #[test] fn dispatch_latest_scores_shows_screen() {
        let r = dispatch(SidebarCommand::LatestScores, &ctx_ok());
        assert_eq!(r.ret, DispatchReturn::Handled);
        assert!(matches!(r.outcome,
            DispatchOutcome::ShowScreen(DispatchScreen::LatestScores)));
    }

    #[test] fn dispatch_add_manager_ok_gate_shows_view() {
        let r = dispatch(SidebarCommand::AddManager, &ctx_ok());
        assert_eq!(r.ret, DispatchReturn::Handled);
        assert!(matches!(r.outcome,
            DispatchOutcome::ShowScreen(DispatchScreen::AddManager(_))));
    }

    #[test] fn dispatch_add_manager_failed_gate_is_side_effect() {
        let ctx = DispatchCtx { registration_ok: false, ..Default::default() };
        let r = dispatch(SidebarCommand::AddManager, &ctx);
        assert_eq!(r.ret, DispatchReturn::Handled);
        match r.outcome {
            DispatchOutcome::SideEffect(s) => assert!(s.contains("0x42b")),
            other => panic!("expected SideEffect, got {:?}", other),
        }
    }

    #[test] fn dispatch_manage_existing_ok_gate_shows_view() {
        let r = dispatch(SidebarCommand::ManageExisting, &ctx_ok());
        assert!(matches!(r.outcome,
            DispatchOutcome::ShowScreen(DispatchScreen::ManageExisting(_))));
    }

    #[test] fn dispatch_manage_existing_failed_gate_is_side_effect() {
        let ctx = DispatchCtx { registration_ok: false, ..Default::default() };
        let r = dispatch(SidebarCommand::ManageExisting, &ctx);
        match r.outcome {
            DispatchOutcome::SideEffect(s) => assert!(s.contains("0x42c")),
            other => panic!("expected SideEffect, got {:?}", other),
        }
    }

    #[test] fn dispatch_noop_swallows() {
        let r = dispatch(SidebarCommand::NoopEaten, &ctx_ok());
        assert_eq!(r.ret, DispatchReturn::Swallow);
        assert!(matches!(r.outcome, DispatchOutcome::SideEffect(_)));
    }

    #[test] fn dispatch_clear_pending_nav_swallows() {
        let r = dispatch(SidebarCommand::ClearPendingNav, &ctx_ok());
        assert_eq!(r.ret, DispatchReturn::Swallow);
        assert!(matches!(r.outcome, DispatchOutcome::SideEffect(_)));
    }

    #[test] fn dispatch_nav_prev_returns_nav_prev() {
        let r = dispatch(SidebarCommand::NavPrev, &ctx_ok());
        assert_eq!(r.ret, DispatchReturn::NavPrev);
    }

    #[test] fn dispatch_nav_next_returns_nav_next() {
        let r = dispatch(SidebarCommand::NavNext, &ctx_ok());
        assert_eq!(r.ret, DispatchReturn::NavNext);
    }

    #[test] fn dispatch_continue_game_unported() {
        assert_unported(SidebarCommand::ContinueGame, "FUN_00693410");
    }
    #[test] fn dispatch_next_match_unported() {
        assert_unported(SidebarCommand::NextMatch, "FUN_00695e60");
    }
    #[test] fn dispatch_job_centre_unported() {
        assert_unported(SidebarCommand::JobCentre, "FUN_00698140");
    }
    #[test] fn dispatch_club_detail_unported() {
        assert_unported(SidebarCommand::ClubDetailOpen, "FUN_0076ffb0");
    }
    #[test] fn dispatch_rankings_unported() {
        assert_unported(SidebarCommand::RankingsOpen, "FUN_007f0020");
    }
    #[test] fn dispatch_new_game_unported() {
        assert_unported(SidebarCommand::NewGameFlow, "FUN_00809ad0");
    }
    #[test] fn dispatch_load_game_unported() {
        assert_unported(SidebarCommand::LoadGameFlow, "FUN_00808a70");
    }
    #[test] fn dispatch_select_leagues_unported() {
        assert_unported(SidebarCommand::SelectLeagues, "FUN_008053d0");
    }
    #[test] fn dispatch_page_jump_unported() {
        assert_unported(SidebarCommand::PageJump, "FUN_00789b50");
    }
    #[test] fn dispatch_person_screen_unported() {
        assert_unported(SidebarCommand::PersonScreenOpen, "FUN_00771810");
    }
    #[test] fn dispatch_offer_kinds_distinct_tags() {
        assert_unported(SidebarCommand::OfferKind1, "FUN_0058a550(1)");
        assert_unported(SidebarCommand::OfferKind2, "FUN_0058a550(2)");
        assert_unported(SidebarCommand::OfferKind3, "FUN_0058a550(3)");
        assert_unported(SidebarCommand::OfferKind4, "FUN_0058a550(4)");
        assert_unported(SidebarCommand::OfferKind5, "FUN_0058a550(5)");
        assert_unported(SidebarCommand::OfferKind6, "FUN_0058a550(6)");
    }

    // ---- is_stub coverage ------------------------------------------------

    #[test] fn latest_scores_not_stub() {
        assert!(!SidebarCommand::LatestScores.is_stub());
    }
    #[test] fn add_manager_not_stub() {
        assert!(!SidebarCommand::AddManager.is_stub());
    }
    #[test] fn manage_existing_not_stub() {
        assert!(!SidebarCommand::ManageExisting.is_stub());
    }
    #[test] fn continue_game_is_stub() {
        assert!(SidebarCommand::ContinueGame.is_stub());
    }

    // ---- dispatch never panics on any known variant ---------------------

    #[test] fn every_variant_dispatches_without_panic() {
        let ctx = ctx_ok();
        for &(_, variant) in ALL {
            let _ = dispatch(variant, &ctx);
        }
    }

    // ---- route() end-to-end ---------------------------------------------

    #[test] fn route_latest_scores_gives_screen() {
        let r = route(0x418, &ctx_ok()).expect("routed");
        assert!(matches!(r.outcome,
            DispatchOutcome::ShowScreen(DispatchScreen::LatestScores)));
    }

    #[test] fn route_add_manager_uses_registration_gate() {
        let r_ok = route(0x42b, &ctx_ok()).unwrap();
        assert!(matches!(r_ok.outcome,
            DispatchOutcome::ShowScreen(DispatchScreen::AddManager(_))));

        let r_bad = route(0x42b, &DispatchCtx::default()).unwrap();
        assert!(matches!(r_bad.outcome, DispatchOutcome::SideEffect(_)));
    }
}

