//! Integration tests for the global sidebar dispatcher
//! (`crate::sidebar_dispatcher`, port of `FUN_007491e0`).
//!
//! Kept in the integration test suite rather than `#[cfg(test)] mod tests`
//! inside the module so a single failing sibling module can't block
//! sidebar-dispatcher verification.

use cm_domain::sidebar_dispatcher::{
    dispatch, route, DispatchCtx, DispatchOutcome, DispatchReturn, DispatchScreen,
    SidebarCommand,
};

/// Every distinct `sVar3 == 0x???` code the decompile branches on.
const ALL_CODES: &[u16] = &[
    0x424, 0x418, 0x42e, 0x41e, 0x425, 0x433, 0x41d, 0x3f2, 0x41c,
    0x420, 0x41f, 0x434, 0x3e9, 0x3eb, 0x415, 0x3ec, 0x3ef, 0x3f0,
    0x42d, 0x414, 0x3f3, 0x40c, 0x3f4, 0x3f5, 0x3f6, 0x3f7, 0x3f8,
    0x3f9, 0x3fa, 0x429, 0x3fd, 0x3fe, 0x3fb, 0x3fc, 0x421, 0x431,
    0x42a, 0x42b, 0x42c, 0x42f, 0x402, 0x3e8, 0xfffe, 0xfffd,
];

fn ctx() -> DispatchCtx {
    DispatchCtx { registration_ok: true, posted_cmd: 0, active_human: 0 }
}

#[test]
fn from_code_covers_all_documented_branches() {
    for &code in ALL_CODES {
        let cmd = SidebarCommand::from_code(code)
            .unwrap_or_else(|| panic!("code {:#x} not routed", code));
        assert_eq!(cmd.to_code(), code, "round-trip failed for {:#x}", code);
    }
}

#[test]
fn unknown_code_returns_none() {
    assert!(SidebarCommand::from_code(0x0001).is_none());
    assert!(SidebarCommand::from_code(0x1234).is_none());
    assert!(SidebarCommand::from_code(0xffff).is_none());
}

#[test]
fn noop_eaten_swallows_click() {
    assert_eq!(dispatch(SidebarCommand::NoopEaten, &ctx()).ret, DispatchReturn::Swallow);
}

#[test]
fn clear_pending_nav_swallows_click() {
    assert_eq!(dispatch(SidebarCommand::ClearPendingNav, &ctx()).ret, DispatchReturn::Swallow);
}

#[test]
fn latest_scores_shows_screen() {
    let r = dispatch(SidebarCommand::LatestScores, &ctx());
    assert_eq!(r.ret, DispatchReturn::Handled);
    assert!(matches!(r.outcome, DispatchOutcome::ShowScreen(DispatchScreen::LatestScores)));
}

#[test]
fn add_manager_dispatches_to_batch14_builder() {
    let r = route(0x42b, &ctx()).expect("0x42b must route");
    assert_eq!(r.ret, DispatchReturn::Handled);
    assert!(matches!(r.outcome, DispatchOutcome::ShowScreen(DispatchScreen::AddManager(_))));
}

#[test]
fn add_manager_gated_by_registration() {
    let ctx = DispatchCtx { registration_ok: false, ..ctx() };
    let r = dispatch(SidebarCommand::AddManager, &ctx);
    assert!(matches!(r.outcome, DispatchOutcome::SideEffect(_)));
}

#[test]
fn manage_existing_dispatches_to_batch14_builder() {
    let r = route(0x42c, &ctx()).expect("0x42c must route");
    assert!(matches!(r.outcome, DispatchOutcome::ShowScreen(DispatchScreen::ManageExisting(_))));
}

#[test]
fn offer_family_routes_all_six_variants() {
    for (code, want) in [
        (0x3f4u16, "FUN_0058a550(1)"),
        (0x3f5, "FUN_0058a550(2)"),
        (0x3f6, "FUN_0058a550(3)"),
        (0x3f7, "FUN_0058a550(4)"),
        (0x3f8, "FUN_0058a550(5)"),
        (0x3f9, "FUN_0058a550(6)"),
    ] {
        let r = route(code, &ctx()).expect("offer must route");
        match r.outcome {
            DispatchOutcome::ShowScreen(DispatchScreen::UnportedHandler(name)) => {
                assert_eq!(name, want, "wrong offer handler for {:#x}", code);
            }
            other => panic!("offer {:#x} produced {:?}", code, other),
        }
    }
}

#[test]
fn select_leagues_routes() {
    assert_eq!(route(0x431, &ctx()).unwrap().ret, DispatchReturn::Handled);
}

#[test]
fn nav_prev_and_next_carry_dedicated_return_codes() {
    assert_eq!(dispatch(SidebarCommand::NavPrev, &ctx()).ret, DispatchReturn::NavPrev);
    assert_eq!(dispatch(SidebarCommand::NavNext, &ctx()).ret, DispatchReturn::NavNext);
}

#[test]
fn exit_confirm_and_restart_confirm_both_show_dialog() {
    for cmd in [SidebarCommand::ExitConfirm, SidebarCommand::RestartConfirm] {
        let r = dispatch(cmd, &ctx());
        assert!(matches!(r.outcome, DispatchOutcome::ShowScreen(DispatchScreen::UnportedHandler(_))));
        assert_eq!(r.ret, DispatchReturn::Handled);
    }
}

#[test]
fn open_family_covers_club_nation_and_national_team() {
    for cmd in [
        SidebarCommand::ClubOpen,
        SidebarCommand::NationOpen,
        SidebarCommand::NationalTeamOpen,
        SidebarCommand::ClubDetailOpen,
        SidebarCommand::JobCentre,
    ] {
        let r = dispatch(cmd, &ctx());
        assert_eq!(r.ret, DispatchReturn::Handled);
        assert!(matches!(r.outcome, DispatchOutcome::ShowScreen(_)));
    }
}

#[test]
fn save_load_new_game_family_dispatches() {
    for cmd in [
        SidebarCommand::SaveGameDialog,
        SidebarCommand::NewGameFlow,
        SidebarCommand::LoadGameFlow,
    ] {
        assert_eq!(dispatch(cmd, &ctx()).ret, DispatchReturn::Handled);
    }
}

#[test]
fn page_jump_routes_as_show_screen() {
    let r = route(1000, &ctx()).expect("PageJump must route");
    assert!(matches!(r.outcome, DispatchOutcome::ShowScreen(_)));
}

#[test]
fn is_stub_flags_unported_handlers_but_not_latest_scores() {
    assert!(SidebarCommand::JobCentre.is_stub());
    assert!(SidebarCommand::RankingsOpen.is_stub());
    assert!(!SidebarCommand::LatestScores.is_stub());
    assert!(!SidebarCommand::AddManager.is_stub());
    assert!(!SidebarCommand::ManageExisting.is_stub());
}

#[test]
fn every_documented_code_dispatches_without_panic() {
    let ctx = ctx();
    for &code in ALL_CODES {
        let r = route(code, &ctx).unwrap_or_else(|| panic!("route {:#x}", code));
        assert!(!matches!(r.outcome, DispatchOutcome::Unhandled), "code {:#x} produced Unhandled", code);
    }
}
