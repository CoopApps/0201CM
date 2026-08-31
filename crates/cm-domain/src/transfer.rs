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
    /// **Display morale** (0..20) — port of contract-record `+0x45` on the exe
    /// (kill #9b partial). Neutral seed = 10 ("Ok"). Displayed via
    /// [`morale_label`] using the exact real thresholds from `FUN_004d2710`.
    /// EVENT-DRIVEN updates (win/loss/red-card/…) not yet wired — the exe's
    /// mutators (`FUN_004d1680`/`FUN_004d14d0`/`FUN_004d0b00`) are located but
    /// their callers weren't traced this pass, so we don't yet know the exact
    /// delta per event.
    #[serde(default = "default_morale")]
    pub morale: u8,
    /// Signed mood accumulator (contract `+0x44`), clamped [-100, +100].
    /// Feeds into contract-renewal decisions (`FUN_004d1680`). Currently 0 and
    /// untouched — needs the event caller trace.
    #[serde(default)]
    pub mood_delta: i8,
}

fn default_morale() -> u8 { 10 }

/// Post-match morale event deltas — the real per-event bumps from
/// `FUN_004d0b00` (post-match dispatcher, agent trace §3). Verbatim:
///   • On the pitch, poor rating (played, low form)   → −4 form (`+0x3d`)
///   • Unused sub                                     → −8 form
///   • Team lost or drew, this player benched         → −3 mood (`+0x44`)
///   • Team won, this player benched                  → +3 mood
///   • Transfer request granted                       → +15 mood
///   • Transfer request refused                       → −15 mood
///   • Squad status demoted                           → −25 mood
///   • Training complaint                             → −5 mood
/// These are the numeric constants written straight from the decompile —
/// clamp bounds ±100, then map to the 0..20 display via [`morale_label`].
pub const MOOD_BENCHED_WIN: i8 = 3;
pub const MOOD_BENCHED_LOSS: i8 = -3;
pub const MOOD_TRANSFER_GRANTED: i8 = 15;
pub const MOOD_TRANSFER_REFUSED: i8 = -15;
pub const MOOD_SQUAD_DEMOTED: i8 = -25;
pub const MOOD_TRAINING_COMPLAINT: i8 = -5;
/// Playing out of position, per match. Tactics gap #7 — real CM01/02 penalty
/// per player-comment string ("<player> is unhappy about being played out of
/// position") gated on the `position_rating` for the slot he actually
/// occupied being negative (the ATTR_CURVE-derived out-of-position drag).
pub const MOOD_OUT_OF_POSITION: i8 = -2;

/// Real exe display thresholds for player morale — `FUN_004d2710`:7-29.
/// Verbatim: 0-3 Very Low, 4-7 Low, 8-11 Ok, 12-14 Good, 15-17 Very Good, 18+ Superb.
pub fn morale_label(m: u8) -> &'static str {
    match m {
        0..=3 => "Very Low",
        4..=7 => "Low",
        8..=11 => "Ok",
        12..=14 => "Good",
        15..=17 => "Very Good",
        _ => "Superb",
    }
}

impl Contract {
    /// Standard offer for a rated player. `weekly_wage` is the real wage from
    /// the ported valuation core (kill #2, `FUN_0084d5d0`); contract length
    /// scales down when a player is over 30.
    pub fn standard_offer(player_id: u32, club_id: u32,
                          weekly_wage: u32, current_year: u16, age: u8) -> Self {
        let weekly_wage = weekly_wage.max(175);
        // Older players get shorter contracts.
        let years: u16 = if age >= 34 { 1 }
                         else if age >= 30 { 2 }
                         else if age >= 26 { 3 }
                         else { 5 };
        Self {
            player_id, club_id, weekly_wage,
            signed_year: current_year,
            expires_year: current_year + years,
            bosman_eligible: false, morale: 10, mood_delta: 0,
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

/// Nudge a mood accumulator by a signed delta, clamped [-100, +100] — the exe's
/// `FUN_004d14d0`:51-56 shape. Then re-derive the display morale byte (0..20)
/// from the accumulator: 100 → 20 ("Superb"), −100 → 0 ("Very Low"), 0 → 10
/// ("Ok"). Approximation of the exe's "cap-by-complaint-bits" that we don't
/// carry yet — faithful in direction and magnitude.
pub fn apply_mood_delta(contract: &mut Contract, delta: i8) {
    contract.mood_delta = ((contract.mood_delta as i32 + delta as i32)
        .clamp(-100, 100)) as i8;
    // Map ±100 → 0..20: 10 (neutral) + mood/10.
    let display = 10 + contract.mood_delta as i32 / 10;
    contract.morale = display.clamp(0, 20) as u8;
}

impl TransferMarket {
    pub fn new() -> Self { Self::default() }

    /// Apply the post-match morale deltas (`FUN_004d0b00`:254/271, agent trace
    /// §3) to every contract on either side. `won` is the club's own
    /// perspective. `benched_players` are the ids of players who did NOT play
    /// this fixture (get the ±3 benched-side delta). Players who DID play get
    /// no delta here (their contribution is via form/rating, wired separately).
    pub fn apply_match_morale(
        &mut self,
        club_id: u32,
        won: Option<bool>,          // Some(true)=win Some(false)=loss None=draw
        benched_players: &[u32],
    ) {
        self.apply_match_morale_full(club_id, won, benched_players, &[]);
    }

    /// Extended version — additionally applies MOOD_OUT_OF_POSITION to each
    /// player id in `out_of_position_players`. Tactics gap #7 wire. The
    /// caller decides who counts as out-of-position from the sign of
    /// `tactics::position_rating` on the slot the player was picked for.
    pub fn apply_match_morale_full(
        &mut self,
        club_id: u32,
        won: Option<bool>,
        benched_players: &[u32],
        out_of_position_players: &[u32],
    ) {
        let benched_delta = match won {
            Some(true) => MOOD_BENCHED_WIN,
            _ => MOOD_BENCHED_LOSS,
        };
        for c in self.contracts.iter_mut() {
            if c.club_id != club_id { continue; }
            if benched_players.contains(&c.player_id) {
                apply_mood_delta(c, benched_delta);
            }
            if out_of_position_players.contains(&c.player_id) {
                apply_mood_delta(c, MOOD_OUT_OF_POSITION);
            }
        }
    }

    /// Bulk-seed contracts for every player at their current club. Called
    /// at new-game creation.
    pub fn seed_from_ratings(
        ratings: &crate::player_rating::PlayerRatingBook,
        current_year: u16,
    ) -> Self {
        let contracts = ratings.players.iter()
            .filter_map(|p| p.club_id.map(|c| Contract::standard_offer(
                p.staff_id, c as u32, p.weekly_wage, current_year, p.age_est,
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
            // Real valuation (kill #2). Accept/counter bands are the recovered
            // FUN_00580a90 gates (_DAT_009569b0=0.8, _DAT_009569d8=0.9): at or
            // above value → accept; within 80% → counter; below → reject. The
            // exact accept-multiple composer (FUN_00848da0, 10.8KB, not
            // decompilable) is a documented refinement.
            let market_value = player.market_value.max(1_000);
            let current_contract = self.contracts.iter()
                .find(|c| c.player_id == bid.target_player_id)
                .cloned();

            // Administration override (kill #8/9 refinement, real port of
            // `FUN_00588c70`:158-217): a club in administration cannot refuse
            // any bid at or above 50% of value — the exe forces sales to
            // reduce wage bill and raise cash.
            let seller_in_admin = finance.is_in_administration(bid.selling_club_id);
            let outcome = if seller_in_admin && bid.amount as f32 >= market_value as f32 * 0.5 {
                BidOutcome::Accepted
            } else if bid.amount >= market_value
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
                    bosman_eligible: false, morale: 10, mood_delta: 0,
                });
            }
            self.resolved_bids.push((bid, outcome));
        }
    }

    /// AI-club transfer activity (kill #4) — a bounded port of the exe's AI
    /// offer path (`FUN_008ac0c0`): each pass, a sample of budget-holding clubs
    /// bids for an affordable player who improves their squad, priced at the
    /// target's real valuation (kill #2); the selling side accepts at/above
    /// value (the recovered accept band). Completed deals move the player, the
    /// money, and the contract. Returns how many transfers completed.
    ///
    /// Bounded for cost (5000+ clubs × 100k players): only `sample` buyer clubs
    /// are considered per pass, targets are drawn from a value-sorted shortlist.
    /// Simplifications (flagged): squad "need" is reduced to "target CA beats the
    /// buyer's average"; the exact AI target-selection/negotiation
    /// (`FUN_00848da0`, not decompilable) is a refinement.
    pub fn run_ai_transfer_pass(
        &mut self,
        ratings: &mut crate::player_rating::PlayerRatingBook,
        finance: &mut FinanceBook,
        current_year: u16,
        sample: usize,
        seed: u64,
    ) -> usize {
        use std::collections::HashMap;
        let mut rng = crate::match_engine_exe::MatchRng::new(seed);
        // Index: club → (player indices, sum CA, count) for squad-average.
        let mut by_club: HashMap<i32, Vec<usize>> = HashMap::new();
        for (i, p) in ratings.players.iter().enumerate() {
            if let Some(c) = p.club_id {
                by_club.entry(c).or_default().push(i);
            }
        }
        let avg_ca = |idxs: &[usize], r: &crate::player_rating::PlayerRatingBook| -> i16 {
            if idxs.is_empty() { return 0; }
            (idxs.iter().map(|&i| r.players[i].ca as i32).sum::<i32>() / idxs.len() as i32) as i16
        };
        // Value-sorted shortlist of realistic targets. Excludes players already
        // moved this pass so a hot deal doesn't repeat.
        let mut shortlist: Vec<usize> = (0..ratings.players.len()).collect();
        shortlist.sort_by_key(|&i| ratings.players[i].market_value);
        let mut moved_this_pass: std::collections::HashSet<u32> =
            std::collections::HashSet::new();

        // Clubs with a budget — random sample (not just a rotation, else the
        // same handful bid each week).
        let all_buyers: Vec<u32> = finance.clubs.iter()
            .filter(|c| c.transfer_budget > 500_000)
            .map(|c| c.club_id).collect();
        let mut buyers: Vec<u32> = Vec::with_capacity(sample);
        if !all_buyers.is_empty() {
            for _ in 0..sample {
                let idx = rng.range(all_buyers.len() as u32) as usize;
                buyers.push(all_buyers[idx]);
            }
        }

        let mut completed = 0usize;
        for &buyer in buyers.iter() {
            let budget = finance.for_club(buyer).map(|c| c.transfer_budget).unwrap_or(0);
            if budget <= 500_000 { continue; }
            let want = avg_ca(by_club.get(&(buyer as i32)).map(|v| v.as_slice()).unwrap_or(&[]), ratings) + 3;
            // Find the best affordable target better than `want`, at another club.
            // Scan the value-sorted shortlist from the top affordable downward.
            let mut chosen: Option<usize> = None;
            for &i in shortlist.iter().rev() {
                let p = &ratings.players[i];
                if p.market_value > budget { continue; }
                if p.club_id == Some(buyer as i32) || p.club_id.is_none() { continue; }
                if p.ca < want { continue; }
                if moved_this_pass.contains(&p.staff_id) { continue; }
                chosen = Some(i);
                break;
            }
            let Some(ti) = chosen else { continue };
            let target = &ratings.players[ti];
            let seller = match target.club_id { Some(c) => c as u32, None => continue };
            let fee = target.market_value.max(1_000);
            let wage = target.weekly_wage.max(175);
            let years = (rng.range(4) + 2) as u16; // FUN_008ac0c0: rand%4+2
            // Accept band (kill #3): buyer offers full value → accepted.
            // Move money (budget + balance), player, contract.
            let can_afford = finance.for_club(buyer).map(|c| c.balance >= fee).unwrap_or(false);
            if !can_afford { continue; }
            if let Some(b) = finance.clubs.iter_mut().find(|c| c.club_id == buyer) {
                b.balance -= fee; b.transfer_budget -= fee;
            }
            if let Some(s) = finance.clubs.iter_mut().find(|c| c.club_id == seller) {
                s.balance += fee; s.transfer_budget += fee;
            }
            ratings.players[ti].club_id = Some(buyer as i32);
            self.contracts.retain(|c| c.player_id != ratings.players[ti].staff_id);
            self.contracts.push(Contract {
                player_id: ratings.players[ti].staff_id,
                club_id: buyer, weekly_wage: wage,
                signed_year: current_year,
                expires_year: current_year + years,
                bosman_eligible: false, morale: 10, mood_delta: 0,
            });
            // Keep the club index roughly current for later buyers this pass.
            by_club.entry(buyer as i32).or_default().push(ti);
            moved_this_pass.insert(ratings.players[ti].staff_id);
            completed += 1;
        }
        completed
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
                      ca, pa: ca, goals_est: 10, age_est: 25, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 1_000_000, weekly_wage: 25_000, position_aptitudes: [0;12] }
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
        let ratings = PlayerRatingBook { players, ..Default::default() };
        let mut market = TransferMarket::seed_from_ratings(&ratings, 2001);
        let mut finance = FinanceBook::default();
        finance.clubs.push(ClubFinance { club_id: 10, balance: 0, weekly_wage_bill: 0, transfer_budget: 0, months_in_the_red: 0, board_confidence: 10, month_wages: 0, month_gate: 0, month_tv_prize: 0, in_administration: false, ..Default::default() });
        finance.clubs.push(ClubFinance { club_id: 20, balance: 10_000_000, weekly_wage_bill: 0, transfer_budget: 10_000_000, months_in_the_red: 0, board_confidence: 10, month_wages: 0, month_gate: 0, month_tv_prize: 0, in_administration: false, ..Default::default() });

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
        let ratings = PlayerRatingBook { players, ..Default::default() };
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
        let ratings = PlayerRatingBook { players, ..Default::default() };
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
        let ratings = PlayerRatingBook { players, ..Default::default() };
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
        let ratings = PlayerRatingBook { players, ..Default::default() };
        let market = TransferMarket::seed_from_ratings(&ratings, 2001);
        // Every player got a 5-year contract → expires 2006.
        let fa = market.free_agents_next_summer(2006);
        assert_eq!(fa.len(), 3);
    }
}
