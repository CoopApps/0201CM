//! Human-manager action facade — the entry point that turns UI events
//! (board meeting request, fine-player click, press-conference statement)
//! into concrete state mutations across FinanceBook + TransferMarket.
//!
//! Item 5 last-mile wiring: [`finance::submit_board_demand`],
//! [`finance::apply_fine`], and [`finance::deliver_press_statement`]
//! exist as end-to-end mutators; this module gives them a single
//! typed action-enum surface a UI layer or scripted event can call
//! without knowing the underlying dispatch details.
//!
//! ## Provenance
//!
//! Each action variant is a real observable manager option in the
//! exe UI:
//! - Board demand: "Meet with Board" menu (exe .rdata 0x57721f) +
//!   seven request types (0x5fbd98..0x5fbef0).
//! - Player fine: "£120,000" fine template (exe .rdata 0x5b3458) +
//!   `discipline.cpp` source path (0x5a55f6).
//! - Press statement: "told the press that..." templates cluster at
//!   exe .rdata 0x5b179a, 0x604df3, 0x604e95, 0x605371, 0x607199,
//!   0x607205 (six verified variants).
//!
//! The evaluator predicates and side-effect wiring were landed in
//! prior commits (see [`crate::finance`]); this module composes them.

use crate::finance::{
    self, BoardDemand, BoardResponse, FineTier, FineReason, FineOutcome,
    PressStatement, NewspaperTier, FinanceBook,
};
use crate::transfer::TransferMarket;

/// A concrete manager-initiated action. One of these fires per UI click,
/// per scripted media prompt, or per periodic AI-manager tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerAction {
    /// Manager asks the board for one of the 7 requests.
    RequestFromBoard {
        club_id: u32,
        demand: BoardDemand,
        /// Manager's recent-form aggregate score (-100..+100). Positive
        /// scores tilt the board toward approval.
        recent_form_score: i16,
    },
    /// Manager fines / warns a player for a specific offence.
    FinePlayer {
        club_id: u32,
        player_id: u32,
        tier: FineTier,
        reason: FineReason,
        /// Player popularity score in the squad (0..20). Higher →
        /// heavier fines more likely to ripple through squad morale.
        player_popularity: u8,
    },
    /// Manager delivers a press statement about a specific player.
    DeliverPressStatement {
        target_player_id: u32,
        statement: PressStatement,
        /// Newspaper coverage tier — National doubles the impact.
        coverage: NewspaperTier,
    },
}

/// Result of executing a [`ManagerAction`]. The variant carries the
/// action-specific response so callers can render UI feedback.
#[derive(Debug, Clone, PartialEq)]
pub enum ManagerActionResult {
    /// Board response — Approved / Refused / OutOfScope.
    Board(BoardResponse),
    /// Fine outcome — AcceptedWithoutComment / TeamRipple.
    Fine {
        outcome: FineOutcome,
        /// Actual cash amount deducted from the player (for UI display).
        fine_amount: i64,
    },
    /// Press statement — actual mood delta applied to target after
    /// newspaper-tier scaling.
    Press { applied_delta: i8 },
    /// Action was well-formed but referenced entities that don't exist
    /// (club or player id not found in the passed books).
    NoOp,
}

/// Execute one [`ManagerAction`] against the current game state.
///
/// This is the item-5 entry point. UI layers (or a headless test
/// harness) call this once per manager-initiated event.
pub fn execute_manager_action(
    action: ManagerAction,
    finance: &mut FinanceBook,
    market: &mut TransferMarket,
) -> ManagerActionResult {
    match action {
        ManagerAction::RequestFromBoard { club_id, demand, recent_form_score } => {
            // Validate club exists first — otherwise the FinanceBook
            // returns OutOfScope which we surface as NoOp instead.
            if !finance.clubs.iter().any(|c| c.club_id == club_id) {
                return ManagerActionResult::NoOp;
            }
            let response = finance.submit_board_demand(
                club_id, demand, recent_form_score);
            ManagerActionResult::Board(response)
        }
        ManagerAction::FinePlayer { club_id, player_id, tier, reason,
                                    player_popularity } => {
            // Validate the player exists on this club
            let weekly_wage = market.contracts.iter()
                .find(|c| c.player_id == player_id && c.club_id == club_id)
                .map(|c| c.weekly_wage);
            let Some(wage) = weekly_wage else {
                return ManagerActionResult::NoOp;
            };
            let fine_amount = finance::compute_fine_amount(tier, wage);
            let outcome = finance.apply_fine(
                market, club_id, player_id, tier, reason, player_popularity);
            ManagerActionResult::Fine { outcome, fine_amount }
        }
        ManagerAction::DeliverPressStatement { target_player_id, statement,
                                              coverage } => {
            // Validate target exists
            if !market.contracts.iter().any(|c| c.player_id == target_player_id) {
                return ManagerActionResult::NoOp;
            }
            let applied_delta = finance.deliver_press_statement(
                market, target_player_id, statement, coverage);
            ManagerActionResult::Press { applied_delta }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transfer::{Contract, SquadStatus};
    use crate::finance::{BoardResponse, ClubFinance};

    fn make_finance(club_id: u32, balance: i64, wage_bill: u32, conf: u8) -> FinanceBook {
        let mut fb = FinanceBook::new();
        let mut c = ClubFinance {
            club_id, balance,
            weekly_wage_bill: wage_bill,
            transfer_budget: 0,
            months_in_the_red: 0,
            board_confidence: conf,
            in_administration: false,
            ..Default::default()
        };
        c.club_id = club_id;
        fb.clubs.push(c);
        fb
    }

    fn make_contract(pid: u32, cid: u32, wage: u32) -> Contract {
        Contract {
            player_id: pid, club_id: cid, weekly_wage: wage,
            signed_year: 2001, expires_year: 2004,
            bosman_eligible: false, morale: 10, mood_delta: 0,
            squad_status: SquadStatus::FirstTeam,
            ..Default::default()
        }
    }

    #[test]
    fn board_request_dispatches_to_finance_book() {
        let mut fb = make_finance(1, 100_000_000, 100_000, 60);
        let mut m = TransferMarket::default();
        let r = execute_manager_action(
            ManagerAction::RequestFromBoard {
                club_id: 1,
                demand: BoardDemand::TransferFunds,
                recent_form_score: 10,
            },
            &mut fb, &mut m,
        );
        match r {
            ManagerActionResult::Board(BoardResponse::Approved { .. }) => {}
            other => panic!("expected Board Approved, got {:?}", other),
        }
        // Transfer budget grew (side effect from submit_board_demand)
        assert!(fb.clubs[0].transfer_budget > 0);
    }

    #[test]
    fn board_request_unknown_club_is_noop() {
        let mut fb = make_finance(1, 100_000, 100_000, 50);
        let mut m = TransferMarket::default();
        let r = execute_manager_action(
            ManagerAction::RequestFromBoard {
                club_id: 999,
                demand: BoardDemand::HigherWageBudget,
                recent_form_score: 0,
            },
            &mut fb, &mut m,
        );
        assert_eq!(r, ManagerActionResult::NoOp);
    }

    #[test]
    fn fine_player_returns_fine_amount_and_outcome() {
        let mut fb = make_finance(1, 1_000_000, 100_000, 50);
        let mut m = TransferMarket::default();
        for pid in [10u32, 11, 12] {
            m.contracts.push(make_contract(pid, 1, 30_000));
        }
        let r = execute_manager_action(
            ManagerAction::FinePlayer {
                club_id: 1,
                player_id: 10,
                tier: FineTier::OneMonthWages,
                reason: FineReason::PoorPerformance,
                player_popularity: 15,
            },
            &mut fb, &mut m,
        );
        match r {
            ManagerActionResult::Fine { outcome, fine_amount } => {
                assert_eq!(fine_amount, 120_000);  // £30k * 4
                assert_eq!(outcome, FineOutcome::TeamRipple { team_delta: -3 });
            }
            other => panic!("expected Fine, got {:?}", other),
        }
        // Other players ripple with -3
        for other in [11u32, 12] {
            let c = m.contracts.iter().find(|c| c.player_id == other).unwrap();
            assert_eq!(c.mood_delta, -3);
        }
    }

    #[test]
    fn fine_unknown_player_is_noop() {
        let mut fb = make_finance(1, 1_000_000, 100_000, 50);
        let mut m = TransferMarket::default();
        let r = execute_manager_action(
            ManagerAction::FinePlayer {
                club_id: 1, player_id: 999,
                tier: FineTier::OneWeekWages,
                reason: FineReason::Generic,
                player_popularity: 10,
            },
            &mut fb, &mut m,
        );
        assert_eq!(r, ManagerActionResult::NoOp);
    }

    #[test]
    fn press_statement_returns_scaled_delta() {
        let mut fb = make_finance(1, 1_000_000, 100_000, 50);
        let mut m = TransferMarket::default();
        m.contracts.push(make_contract(42, 1, 30_000));
        let r = execute_manager_action(
            ManagerAction::DeliverPressStatement {
                target_player_id: 42,
                statement: PressStatement::NotForSale,
                coverage: NewspaperTier::National,
            },
            &mut fb, &mut m,
        );
        match r {
            ManagerActionResult::Press { applied_delta } => {
                assert_eq!(applied_delta, 10);  // +5 * 2 (National)
            }
            other => panic!("expected Press, got {:?}", other),
        }
    }

    #[test]
    fn press_statement_unknown_target_is_noop() {
        let mut fb = make_finance(1, 1_000_000, 100_000, 50);
        let mut m = TransferMarket::default();
        let r = execute_manager_action(
            ManagerAction::DeliverPressStatement {
                target_player_id: 999,
                statement: PressStatement::PraiseProfessionalism,
                coverage: NewspaperTier::Local,
            },
            &mut fb, &mut m,
        );
        assert_eq!(r, ManagerActionResult::NoOp);
    }
}
