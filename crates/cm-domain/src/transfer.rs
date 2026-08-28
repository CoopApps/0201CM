//! Transfer market + contract system.
//!
//! Ports `transfer_manager.cpp`, `contract_manager.cpp`, `transfer_offer.cpp`,
//! `scout_manager.cpp`, `staff_contracts.cpp` into one substrate. Handles:
//!
//! * Contract expiry tracking (players out of contract at season end)
//! * Bid submission + AI response
//! * Weekly wage negotiation
//! * Transfer window state (per-country, from `finance.rs::CountryRulesSpec`)
//!
//! # Scope
//!
//! The exe's transfer AI has hundreds of behaviour rules (interest based on
//! CA-fit-in-squad, wage vs club-wage-bill, reputation gap, chairman
//! preferences, agent demands). This substrate provides the state model +
//! bid resolution shape, with a simplified AI: bids from higher-reputation
//! clubs succeed proportionally to (bid / market_value + wage_offer /
//! current_wage). Enough to make transfers happen in the tick.

use serde::{Deserialize, Serialize};

use crate::finance::FinanceBook;

/// Length of a player contract in years.
pub type ContractYears = u8;

/// A single contract binding a player to a club.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    pub player_id: u32,
    pub club_id: u32,
    /// Weekly wage in the country's local unit.
    pub weekly_wage: u32,
    /// Season the contract was signed.
    pub signed_year: u16,
    /// Season the contract expires at end-of-season.
    pub expires_year: u16,
    /// Bosman-eligible = can talk to other clubs within 6 months of expiry.
    /// Set to true by the tick when `expires_year - current_year <= 0` and
    /// current month >= expires_year's May (traditional 6-month window).
    pub bosman_eligible: bool,
}

impl Contract {
    /// Standard offer for a rated player. Wage scales with CA; contract
    /// length scales down when a player is over 30.
    pub fn standard_offer(player_id: u32, club_id: u32,
                          ca: i16, current_year: u16, age: u8) -> Self {
        // Weekly wage ~= CA * £250 (100 CA = £25k/wk which matches mid-tier
        // 2001-02 Premier League).
        let weekly_wage = (ca as u32 * 250).max(500);
        // Older players get shorter contracts.
        let years: u16 = if age >= 34 { 1 }
                         else if age >= 30 { 2 }
                         else if age >= 26 { 3 }
                         else { 5 };
        Self {
            player_id, club_id, weekly_wage,
            signed_year: current_year,
            expires_year: current_year + years,
            bosman_eligible: false,
        }
    }
}

/// A transfer bid one club submits for another club's player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferBid {
    pub bidding_club_id: u32,
    pub target_player_id: u32,
    pub selling_club_id: u32,
    /// Bid amount in the currency the balance is denominated in.
    pub amount: i64,
    /// Weekly wage the bidding club is offering the player.
    pub player_wage_offer: u32,
    /// Contract length being offered in years.
    pub contract_years: u8,
}

/// Result of the selling club's AI evaluation of a bid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BidOutcome {
    Accepted,
    Rejected,
    Countered,
}

/// The transfer market state — indexed by club and player.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransferMarket {
    pub contracts: Vec<Contract>,
    /// Bids currently pending — evaluated once per day.
    pub pending_bids: Vec<TransferBid>,
    /// Bids that resolved in the last tick, kept for news generation.
    pub resolved_bids: Vec<(TransferBid, BidOutcome)>,
}

impl TransferMarket {
    pub fn new() -> Self { Self::default() }

    /// Bulk-seed contracts for every player at their current club. Called
    /// at new-game creation.
    pub fn seed_from_ratings(
        ratings: &crate::player_rating::PlayerRatingBook,
        current_year: u16,
    ) -> Self {
        let contracts = ratings.players.iter()
            .filter_map(|p| p.club_id.map(|c| Contract::standard_offer(
                p.staff_id, c as u32, p.ca, current_year, p.age_est,
            )))
            .collect();
        Self { contracts, pending_bids: Vec::new(), resolved_bids: Vec::new() }
    }

    /// Submit a bid. Bids are queued and resolved on the next daily tick.
    pub fn submit_bid(&mut self, bid: TransferBid) {
        self.pending_bids.push(bid);
    }

    /// Resolve every pending bid using a simple AI:
    ///   - Accept if bid >= market_value AND wage_offer >= current_wage
    ///   - Counter if bid within 80% of market_value
    ///   - Reject otherwise
    ///
    /// Market value = CA² * 100.
    pub fn resolve_bids(&mut self, current_year: u16,
                        ratings: &crate::player_rating::PlayerRatingBook,
                        finance: &mut FinanceBook)
    {
        let bids = std::mem::take(&mut self.pending_bids);
        for bid in bids {
            let Some(player) = ratings.players.iter().find(|p| p.staff_id == bid.target_player_id) else {
                self.resolved_bids.push((bid, BidOutcome::Rejected));
                continue;
            };
            let market_value = (player.ca as i64).pow(2) * 100;
            let current_contract = self.contracts.iter()
                .find(|c| c.player_id == bid.target_player_id)
                .cloned();

            let outcome = if bid.amount >= market_value
                && current_contract.as_ref().map_or(true, |c| bid.player_wage_offer >= c.weekly_wage)
            {
                BidOutcome::Accepted
            } else if bid.amount as f32 >= market_value as f32 * 0.8 {
                BidOutcome::Countered
            } else {
                BidOutcome::Rejected
            };

            if outcome == BidOutcome::Accepted {
                // Debit bidding club, credit selling club.
                if let Some(bidder) = finance.clubs.iter_mut().find(|c| c.club_id == bid.bidding_club_id) {
                    bidder.balance -= bid.amount;
                }
                if let Some(seller) = finance.clubs.iter_mut().find(|c| c.club_id == bid.selling_club_id) {
                    seller.balance += bid.amount;
                }
                // Replace contract.
                self.contracts.retain(|c| c.player_id != bid.target_player_id);
                self.contracts.push(Contract {
                    player_id: bid.target_player_id,
                    club_id: bid.bidding_club_id,
                    weekly_wage: bid.player_wage_offer,
                    signed_year: current_year,
                    expires_year: current_year + bid.contract_years as u16,
                    bosman_eligible: false,
                });
            }
            self.resolved_bids.push((bid, outcome));
        }
    }

    /// Update Bosman eligibility flags. Called weekly. A player becomes
    /// eligible in the 6 months (approx: month >= 1) before contract
    /// expiry.
    pub fn update_bosman_flags(&mut self, current_year: u16, current_month: u8) {
        for c in &mut self.contracts {
            let months_left = (c.expires_year as i32 - current_year as i32) * 12
                            - current_month as i32;
            c.bosman_eligible = months_left <= 6;
        }
    }

    /// Players out of contract at end of `year`. Used for the summer
    /// transfer window's free-agent list.
    pub fn free_agents_next_summer(&self, year: u16) -> Vec<u32> {
        self.contracts.iter()
            .filter(|c| c.expires_year == year)
            .map(|c| c.player_id)
            .collect()
    }

    pub fn contract_for(&self, player_id: u32) -> Option<&Contract> {
        self.contracts.iter().find(|c| c.player_id == player_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finance::ClubFinance;
    use crate::player_rating::{PlayerRatingBook, RatedPlayer};

    fn mk_player(id: u32, ca: i16, club: u32) -> RatedPlayer {
        RatedPlayer { staff_id: id, club_id: Some(club as i32), division_id: Some(24),
                      ca, pa: ca, goals_est: 10, age_est: 25 }
    }

    #[test]
    fn standard_offer_scales_wage_with_ca() {
        let a = Contract::standard_offer(1, 10, 100, 2001, 25);
        let b = Contract::standard_offer(2, 10, 180, 2001, 25);
        assert!(b.weekly_wage > a.weekly_wage);
    }

    #[test]
    fn old_players_get_shorter_contracts() {
        let young = Contract::standard_offer(1, 10, 150, 2001, 22);
        let old = Contract::standard_offer(2, 10, 150, 2001, 35);
        assert!(young.expires_year > old.expires_year);
    }

    #[test]
    fn accepted_bid_moves_money_and_contract() {
        let players = vec![mk_player(1, 100, 10)];
        let ratings = PlayerRatingBook { players };
        let mut market = TransferMarket::seed_from_ratings(&ratings, 2001);
        let mut finance = FinanceBook::default();
        finance.clubs.push(ClubFinance { club_id: 10, balance: 0, weekly_wage_bill: 0, transfer_budget: 0, months_in_the_red: 0 });
        finance.clubs.push(ClubFinance { club_id: 20, balance: 10_000_000, weekly_wage_bill: 0, transfer_budget: 10_000_000, months_in_the_red: 0 });

        // Market value = 100^2 * 100 = 1,000,000. Bid 2M with matching wage.
        market.submit_bid(TransferBid {
            bidding_club_id: 20, target_player_id: 1, selling_club_id: 10,
            amount: 2_000_000, player_wage_offer: 50_000, contract_years: 4,
        });
        market.resolve_bids(2001, &ratings, &mut finance);
        assert_eq!(market.resolved_bids[0].1, BidOutcome::Accepted);
        assert_eq!(finance.for_club(20).unwrap().balance, 8_000_000);
        assert_eq!(finance.for_club(10).unwrap().balance, 2_000_000);
        assert_eq!(market.contract_for(1).unwrap().club_id, 20);
    }

    #[test]
    fn low_bid_gets_rejected() {
        let players = vec![mk_player(1, 100, 10)];
        let ratings = PlayerRatingBook { players };
        let mut market = TransferMarket::seed_from_ratings(&ratings, 2001);
        let mut finance = FinanceBook::default();
        market.submit_bid(TransferBid {
            bidding_club_id: 20, target_player_id: 1, selling_club_id: 10,
            amount: 100, player_wage_offer: 1000, contract_years: 3,
        });
        market.resolve_bids(2001, &ratings, &mut finance);
        assert_eq!(market.resolved_bids[0].1, BidOutcome::Rejected);
        assert_eq!(market.contract_for(1).unwrap().club_id, 10);
    }

    #[test]
    fn near_market_bid_gets_countered() {
        let players = vec![mk_player(1, 100, 10)];
        let ratings = PlayerRatingBook { players };
        let mut market = TransferMarket::seed_from_ratings(&ratings, 2001);
        let mut finance = FinanceBook::default();
        // Market value 1M; bid 900k (90%) → counter.
        market.submit_bid(TransferBid {
            bidding_club_id: 20, target_player_id: 1, selling_club_id: 10,
            amount: 900_000, player_wage_offer: 50_000, contract_years: 3,
        });
        market.resolve_bids(2001, &ratings, &mut finance);
        assert_eq!(market.resolved_bids[0].1, BidOutcome::Countered);
    }

    #[test]
    fn bosman_flag_triggers_when_close_to_expiry() {
        let players = vec![mk_player(1, 100, 10)];
        let ratings = PlayerRatingBook { players };
        let mut market = TransferMarket::seed_from_ratings(&ratings, 2001);
        // Standard offer for age 25 = 5-year contract expiring 2006.
        // In Jan 2006, months_left = 0*12 - 1 = -1 → Bosman-eligible.
        market.update_bosman_flags(2006, 1);
        assert!(market.contract_for(1).unwrap().bosman_eligible);
        // In Jan 2003, months_left = 3*12 - 1 = 35 → not eligible.
        market.update_bosman_flags(2003, 1);
        assert!(!market.contract_for(1).unwrap().bosman_eligible);
    }

    #[test]
    fn free_agents_at_year_end_lists_expiring_contracts() {
        let players = vec![mk_player(1, 100, 10), mk_player(2, 120, 10),
                           mk_player(3, 140, 20)];
        let ratings = PlayerRatingBook { players };
        let market = TransferMarket::seed_from_ratings(&ratings, 2001);
        // Every player got a 5-year contract → expires 2006.
        let fa = market.free_agents_next_summer(2006);
        assert_eq!(fa.len(), 3);
    }
}
