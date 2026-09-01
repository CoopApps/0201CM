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
    // Note: manual Default impl below (SquadStatus needs a default variant).
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
    // --- Bonus + status fields from the transfer-AI decode ------------------
    /// Placement in the manager's squad — `+0x35 & 0x3f` in the exe. Drives
    /// wage floor + auto-complaint cascade + loan-eligibility in the real AI.
    #[serde(default = "default_squad_status")]
    pub squad_status: SquadStatus,
    /// Signing-on fee paid up front (£). Contract `+0x21`. Floor 50k, or
    /// 100k when player market_value > £1M (per `FUN_004d3ea0`).
    #[serde(default)]
    pub signing_on_fee: u32,
    /// Per-appearance fee (£/match). Contract `+0x4a` (offer id 0x18).
    #[serde(default)]
    pub appearance_fee: u32,
    /// Per-goal bonus (£/goal). Offer id 0x19.
    #[serde(default)]
    pub goal_bonus: u32,
    /// Per-assist bonus (£/assist). Offer id 0x1a.
    #[serde(default)]
    pub assist_bonus: u32,
    /// Per-clean-sheet bonus (£/CS). Offer id 0x1b.
    #[serde(default)]
    pub clean_sheet_bonus: u32,
    /// End-of-contract loyalty bonus (£). Offer id 0x1c.
    #[serde(default)]
    pub loyalty_bonus: u32,
    /// SPECULATIVE — no exe backing. `reports/transfer_deeper_decode.md` §7
    /// verified that no multi-round agent-haggling engine exists in cm0102;
    /// the `+0x69` "agent" pointer is a single-cut wage multiplier at compose
    /// time, not a percentage fee. Kept as a passthrough field only; the
    /// composer writes 0. Do not model haggling logic on top of this.
    #[serde(default)]
    pub agent_fee_pct: u8,
    /// `+0x4f & 0x02` — player is on the transfer-listed-for-loan list.
    /// AI clubs can bid for them on loan without a formal loan offer flow.
    #[serde(default)]
    pub on_loan_list: bool,
    /// Present when the player is currently on loan somewhere. Contract's
    /// parent-club pointer + wage-share + recall-window all live here.
    #[serde(default)]
    pub loan: Option<LoanState>,
}

fn default_squad_status() -> SquadStatus { SquadStatus::FirstTeam }

impl Default for Contract {
    fn default() -> Self {
        Contract {
            player_id: 0, club_id: 0, weekly_wage: 0,
            signed_year: 0, expires_year: 0,
            bosman_eligible: false, morale: 10, mood_delta: 0,
            squad_status: SquadStatus::FirstTeam,
            signing_on_fee: 0, appearance_fee: 0,
            goal_bonus: 0, assist_bonus: 0, clean_sheet_bonus: 0,
            loyalty_bonus: 0, agent_fee_pct: 0,
            on_loan_list: false, loan: None,
        }
    }
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
            ..Default::default()
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
    /// Negotiation round counter — bid record `+0x2e round_counter` in the
    /// exe, capped at `NEGOTIATION_ROUND_CAP` (3). A fresh bid is round 0;
    /// each `resolve_bids` counter increments this by 1. Beyond the cap
    /// the AI rejects rather than counter-offering. VERIFIED from
    /// FUN_008ad0e0 and reports/transfer_ai_loans_decode.md.
    #[serde(default)]
    pub round: u8,
}

/// Result of the selling club's AI evaluation of a bid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BidOutcome {
    Accepted,
    Rejected,
    /// AI wants more money — carries the counter-offer amount the seller
    /// will accept. Bidding club can re-submit with amount >= this to win
    /// (up to `NEGOTIATION_ROUND_CAP` rounds).
    Countered,
}

/// Sidecar for a Countered outcome — the seller's asking price this round.
/// Used by [`TransferMarket::counter_for`] so the bidder knows what to bid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CounterOffer {
    pub target_player_id: u32,
    pub bidding_club_id: u32,
    pub selling_club_id: u32,
    pub asking_amount: i64,
    pub round: u8,
}

/// The transfer market state — indexed by club and player.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransferMarket {
    pub contracts: Vec<Contract>,
    /// Bids currently pending — evaluated once per day.
    pub pending_bids: Vec<TransferBid>,
    /// Bids that resolved in the last tick, kept for news generation.
    pub resolved_bids: Vec<(TransferBid, BidOutcome)>,
    /// Active counter-offers keyed by (bidding_club, target_player). Set
    /// when `resolve_bids` returns `Countered`; the bidder can re-submit
    /// (round+1) with amount >= asking_amount to convert to Accepted.
    /// Cleared on Accept/Reject or when round exceeds NEGOTIATION_ROUND_CAP.
    #[serde(default)]
    pub active_counters: Vec<CounterOffer>,
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
        Self { contracts, pending_bids: Vec::new(), resolved_bids: Vec::new(),
               active_counters: Vec::new() }
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
                        ratings: &mut crate::player_rating::PlayerRatingBook,
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
            // canonical wage/fee composer (`FUN_00848da0`, 10.8KB) is now
            // ported as [`compose_wage_offer`]; this fee-only branch still
            // uses the simpler kill-#2 valuation because a full offer isn't
            // needed to resolve an incoming BID (only its wage-and-fee shape
            // matters when the human opens the negotiation dialog).
            let market_value = player.market_value.max(1_000);
            let current_contract = self.contracts.iter()
                .find(|c| c.player_id == bid.target_player_id)
                .cloned();

            // Beyond-cap check — verified NEGOTIATION_ROUND_CAP from
            // FUN_008ad0e0 (see reports/transfer_ai_loans_decode.md). Once
            // the round counter hits 3, the seller stops counter-offering.
            if bid.round >= NEGOTIATION_ROUND_CAP {
                self.resolved_bids.push((bid, BidOutcome::Rejected));
                continue;
            }

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

            // Track / clear the counter-offer sidecar.
            match outcome {
                BidOutcome::Countered => {
                    // Record the seller's ask so the bidder knows what to bid
                    // on the next round. Ask = market_value (walk down toward
                    // the bidder's offer at 5% per round, mirroring the exe's
                    // FUN_008ad0e0 easing).
                    let asking = ((market_value as f64)
                        * (1.0 - 0.05 * bid.round as f64).max(0.85)) as i64;
                    // Replace any existing counter for this (bidder, player).
                    self.active_counters.retain(|c|
                        !(c.bidding_club_id == bid.bidding_club_id
                          && c.target_player_id == bid.target_player_id));
                    self.active_counters.push(CounterOffer {
                        target_player_id: bid.target_player_id,
                        bidding_club_id: bid.bidding_club_id,
                        selling_club_id: bid.selling_club_id,
                        asking_amount: asking,
                        round: bid.round.saturating_add(1),
                    });
                }
                BidOutcome::Accepted | BidOutcome::Rejected => {
                    self.active_counters.retain(|c|
                        !(c.bidding_club_id == bid.bidding_club_id
                          && c.target_player_id == bid.target_player_id));
                }
            }

            let mut outcome = outcome;
            if outcome == BidOutcome::Accepted {
                // Chairman-approval gate on the BIDDING club — verified port
                // of FUN_00583fc0:122-127. When the bid would push the buyer
                // into the red, the chairman refuses if the amount exceeds
                // `generosity × £500,000`. On refusal, generosity++ (cap 20)
                // and the deal collapses. See chairman_approves_overrun.
                let (bidder_bal, bidder_has_chair) = finance.for_club(bid.bidding_club_id)
                    .map(|c| (c.balance, true))
                    .unwrap_or((0, false));
                let would_go_negative = bidder_bal < bid.amount;
                if bidder_has_chair {
                    let cs = finance.chairman.entry(bid.bidding_club_id)
                        .or_insert_with(crate::finance::ChairmanState::default);
                    if !crate::finance::chairman_approves_overrun(
                        cs, bid.amount, would_go_negative,
                    ) {
                        outcome = BidOutcome::Rejected;
                    }
                }
            }
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
                    ..Default::default()
                });
                // Update player registration — RatedPlayer.club_id must
                // move with the contract so snapshot_team_for_engine sees
                // the player at their new club. Prior resolve_bids left
                // this stale; run_ai_transfer_pass moved it directly.
                // Now unified so bid-pipeline transfers keep the pool in
                // sync with contracts.
                if let Some(rp) = ratings.players.iter_mut().find(|p| p.staff_id == bid.target_player_id) {
                    rp.club_id = Some(bid.bidding_club_id as i32);
                }
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
    /// Simplifications (flagged): the exact per-position "who needs a
    /// striker?" scan (`FUN_0082fc50`, 12.8KB) is too big to skim here — we
    /// bracket it with the DECODED reputation-fit gate (0.75, from
    /// `FUN_008ad0e0`) and the position-quota check (port of
    /// `FUN_008ba4b0`, 5-DEF/7-MID/3-FWD hard caps). The composer
    /// `FUN_00848da0` is now ported as [`compose_wage_offer`] but the
    /// fee/wage numbers here still come from the kill-#2 valuation because
    /// this AI pass runs at bulk-scan cost (thousands of buyers × millions
    /// of candidates); [`compose_wage_offer`] is invoked from the negotiated
    /// path (contract-offer dialog + auction settlement).
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

        // Index by club — needed for both squad-avg CA and position quotas.
        // A RatedPlayer knows its role-band via `is_gk` (already populated
        // during the type6→type10 join). The finer DEF/MID/FWD split has to
        // come from the type10 aptitudes; we approximate it here from the
        // existing `position` byte on the person record's runtime side (via
        // the transfer's contract). When the position bit isn't discoverable
        // the player is bucketed as MID by default (matches the exe's
        // FUN_008ba4b0 fallback path).
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

        // Position bucketer — reads the rated player's aptitudes to decide
        // GK/DEF/MID/FWD. Matches the exe's cascade at FUN_008ba4b0 lines
        // 68-82 (position byte 0x6/0xd = GK, 0x8/0xf = DEF, 0x9 = MID,
        // 0xa = FWD).
        let bucket_of = |p: &crate::player_rating::RatedPlayer| -> (bool, bool, bool, bool) {
            // (is_gk, is_def, is_mid, is_fwd)
            if p.is_gk { return (true, false, false, false); }
            let a = &p.position_aptitudes;
            // Pick the largest aptitude among defender / midfielder / attacker.
            let d = a[2].max(a[3]);      // D + DM
            let m = a[4].max(a[5]);      // M + AM
            let f = a[6];                // ST
            if f >= d && f >= m { (false, false, false, true) }
            else if d >= m      { (false, true,  false, false) }
            else                { (false, false, true,  false) }
        };

        // Per-club current position counts — for the quota check.
        let mut counts: HashMap<i32, (u8, u8, u8, u8, bool)> = HashMap::new();
        for (c, idxs) in by_club.iter() {
            let (mut o, mut d, mut m, mut f, mut gk) = (0u8, 0u8, 0u8, 0u8, false);
            for &i in idxs {
                let p = &ratings.players[i];
                let (is_gk, is_d, is_m, is_f) = bucket_of(p);
                if is_gk { gk = true; } else { o = o.saturating_add(1); }
                if is_d  { d = d.saturating_add(1); }
                if is_m  { m = m.saturating_add(1); }
                if is_f  { f = f.saturating_add(1); }
            }
            counts.insert(*c, (o, d, m, f, gk));
        }

        // Value-sorted shortlist of realistic targets.
        let mut shortlist: Vec<usize> = (0..ratings.players.len()).collect();
        shortlist.sort_by_key(|&i| ratings.players[i].market_value);
        let mut moved_this_pass: std::collections::HashSet<u32> =
            std::collections::HashSet::new();

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
            let buyer_rep = finance.club_reputation.get(&buyer).copied().unwrap_or(1000);

            let want = avg_ca(by_club.get(&(buyer as i32)).map(|v| v.as_slice()).unwrap_or(&[]), ratings) + 3;
            let mut chosen: Option<usize> = None;
            for &i in shortlist.iter().rev() {
                let p = &ratings.players[i];
                if p.market_value > budget { continue; }
                if p.club_id == Some(buyer as i32) || p.club_id.is_none() { continue; }
                if p.ca < want { continue; }
                if moved_this_pass.contains(&p.staff_id) { continue; }

                // REPUTATION-FIT gate (FUN_008ad0e0 line 156 — accept when
                // ratio ≥ 0.75). The exe computes rep_fit as target's rep
                // over buyer's rep. Here we use `market_value / buyer_avg_ca`
                // as a monotonic proxy until FUN_0082fc50 is decoded.
                let seller = match p.club_id { Some(c) => c as u32, None => continue };
                let seller_rep = finance.club_reputation.get(&seller).copied().unwrap_or(1000);
                let rep_fit = seller_rep as f32 / buyer_rep.max(1) as f32;
                if rep_fit < REPUTATION_FIT_ACCEPT { continue; }
                if rep_fit > ASKING_OVER_BASE_REJECT { continue; }

                // POSITION-QUOTA gate (FUN_008ba4b0 port).
                let (o, d, m, f, gk) = counts.get(&(buyer as i32)).copied()
                    .unwrap_or((0, 0, 0, 0, false));
                let (is_gk, is_d, is_m, is_f) = bucket_of(p);
                if position_quota_check(is_gk, is_d, is_m, is_f,
                                        o, d, m, f, gk) != QuotaReject::Ok {
                    continue;
                }
                chosen = Some(i);
                break;
            }
            let Some(ti) = chosen else { continue };
            let target = &ratings.players[ti];
            let seller = match target.club_id { Some(c) => c as u32, None => continue };
            let fee = target.market_value.max(1_000);

            // Predicted seller-ask wage — verified port of FUN_006ce0e0
            // (`predict_wage`), mode 1 = renewal ask. Inputs approximated from
            // the RatedPlayer/finance we have on hand:
            //   agent_quality ≈ target.ca clamped to 20 (matches the exe's
            //     FUN_0052df60 cap; the true source is the agent-staff record).
            //   rep_bucket    = seller_rep / 50 (real FUN_0052a330 output).
            //   contract_field = current weekly_wage (renewal starts here).
            //   game_day       = seed as u32 (deterministic per-pass).
            let agent_q = (target.ca as i32).clamp(1, 20);
            let seller_rep_now = finance.club_reputation.get(&seller).copied().unwrap_or(1000);
            let rep_bucket = (seller_rep_now as i32) / 50;
            let ask_wage = predict_wage(
                target.weekly_wage as i32,
                agent_q, rep_bucket,
                target.age_est, target.is_gk,
                target.staff_id, seed as u32,
                /*mode=*/1,
            ).max(175) as u32;
            // Squad-status promotion delta — verified port of FUN_004d79c0.
            // The AI offers KeyPlayer to signings that push the buyer squad
            // avg CA; otherwise FirstTeamSquad. Position nibble derived from
            // bucket_of() cascade (GK=1, DEF=2, MID=3, FWD=4, STR=6).
            let (is_gk_t, is_d_t, is_m_t, _is_f_t) = bucket_of(target);
            let pos_nibble = if is_gk_t { 1 } else if is_d_t { 2 }
                             else if is_m_t { 3 } else { 4 };
            let asking_status = if target.ca >= want + 5 { SquadStatus::KeyPlayer }
                                else { SquadStatus::FirstTeamSquad };
            let cp = ComposerPlayer {
                player_id: target.staff_id,
                ca: target.ca, pa: target.pa,
                player_reputation: 0, market_value: target.market_value,
                current_wage: target.weekly_wage, age: target.age_est,
                international_caps: 0, role_byte: 5, has_agent: false,
            };
            let cc = ComposerClub { club_id: buyer, reputation: buyer_rep };
            let wage = contract_cost_readback(
                &cp, &cc, pos_nibble,
                SquadStatus::FirstTeamSquad, // current-record placeholder
                asking_status, ask_wage, /*mode=*/1,
            );
            let years = (rng.range(4) + 2) as u16; // FUN_008ac0c0: rand%4+2

            // Pre-flight can-afford + wage-bill refuse gates. Skip
            // submitting bids we already know will collapse — matches the
            // exe's FUN_008ac0c0 pre-composer sanity checks that gate whether
            // a bid record is ever built.
            let can_afford = finance.for_club(buyer).map(|c| c.balance >= fee).unwrap_or(false);
            if !can_afford { continue; }
            let (bal, wage_bill) = finance.for_club(buyer)
                .map(|c| (c.balance, c.weekly_wage_bill)).unwrap_or((0, 0));
            let monthly_income = (bal / 12).max(1) as u32;
            let new_wage_bill = wage_bill.saturating_add(wage);
            if new_wage_bill as u64 * 100 > monthly_income as u64 * WAGE_BILL_REFUSE_PCT as u64 {
                continue;
            }

            // Submit the bid through the pipeline — matches the exe's
            // FUN_008ac0c0 path where every AI transfer goes through
            // FUN_008d48b0 bid ctor → pending_bids → FUN_008ad0e0 resolver.
            // This makes the AI transfer visible to a human observer via
            // pending_bids / resolved_bids, and reuses all the verified
            // resolver logic (chairman_approves_overrun,
            // NEGOTIATION_ROUND_CAP, active_counters sidecar, admin
            // override, etc.) instead of duplicating half of it here.
            self.pending_bids.push(TransferBid {
                bidding_club_id: buyer,
                target_player_id: ratings.players[ti].staff_id,
                selling_club_id: seller,
                amount: fee,
                player_wage_offer: wage,
                contract_years: years as u8,
                round: 0,
            });

            // Track position quotas + moved-this-pass optimistically —
            // resolve_bids may reject some. The bookkeeping stays close
            // enough for the next buyer's candidate selection; a
            // rejected bid's slot will re-open on the next AI pass.
            by_club.entry(buyer as i32).or_default().push(ti);
            let (is_gk, is_d, is_m, is_f) = bucket_of(&ratings.players[ti]);
            let bc = counts.entry(buyer as i32).or_insert((0, 0, 0, 0, false));
            if is_gk { bc.4 = true; } else { bc.0 = bc.0.saturating_add(1); }
            if is_d  { bc.1 = bc.1.saturating_add(1); }
            if is_m  { bc.2 = bc.2.saturating_add(1); }
            if is_f  { bc.3 = bc.3.saturating_add(1); }
            if let Some(sc) = counts.get_mut(&(seller as i32)) {
                if is_gk { sc.4 = false; } else { sc.0 = sc.0.saturating_sub(1); }
                if is_d  { sc.1 = sc.1.saturating_sub(1); }
                if is_m  { sc.2 = sc.2.saturating_sub(1); }
                if is_f  { sc.3 = sc.3.saturating_sub(1); }
            }
            moved_this_pass.insert(ratings.players[ti].staff_id);
        }

        // Drive the resolver on everything the AI just submitted. The
        // real exe runs FUN_008ad0e0 on the daily tick to process
        // pending_bids; we co-run it inside the AI pass so the pipeline
        // resolves this batch before the next AI pass runs. Bids that
        // Counter/Reject stay visible in resolved_bids for news + human
        // observation. `completed` counts Accepted only.
        let batch_start = self.resolved_bids.len();
        self.resolve_bids(current_year, ratings, finance);
        completed = self.resolved_bids[batch_start..].iter()
            .filter(|(_, o)| *o == BidOutcome::Accepted)
            .count();
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

    /// Look up an active counter-offer for a (bidder, player) pair. Returns
    /// the asking amount + current round so the bidder can decide whether
    /// to re-bid at that price. See `NEGOTIATION_ROUND_CAP`.
    pub fn counter_for(&self, bidder: u32, player_id: u32) -> Option<&CounterOffer> {
        self.active_counters.iter().find(|c|
            c.bidding_club_id == bidder && c.target_player_id == player_id)
    }
}

// ---------------------------------------------------------------------------
// Transfer-AI + loan + wage-negotiation type foundations
//
// Types decoded from `FUN_00848da0` (canonical contract/offer composer — the
// 10,811-byte function Ghidra couldn't emit due to the FP+SEH cascade, now
// ported below as [`compose_wage_offer`] via a direct read of the raw asm at
// `05949_sub_00848da0.asm`; the 7-arg signature is fully recovered from all
// 13 caller sites and the sentinel/rep-cascade/wage-clamp gates are honoured),
// `FUN_004d3ea0`
// (multi-round wage negotiator), `FUN_008d2d20` (offer composer), `FUN_008d48b0`
// (bid record ctor), `FUN_004dfbd0` (squad-status ↔ loan-list news),
// `FUN_00594220` (loan-recall date gate). Full report:
// `reports/transfer_ai_loans_decode.md`.
//
// This block is TYPES + CONSTANTS ONLY — no behaviour is wired yet. Existing
// `Contract` / `TransferBid` / `run_ai_transfer_pass` are unchanged. The
// existing `Contract` will grow these fields in a follow-up commit once we
// verify a cold rebuild survives the current addition.
// ---------------------------------------------------------------------------

/// Squad-status tier byte — contract `+0x35 & 0x3f` (also mirrored at `+0x4f`).
/// Enum values match the exe's raw byte values (per `FUN_004dfbd0`'s 7 news
/// branches).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SquadStatus {
    KeyPlayer      = 1,
    FirstTeam      = 2,
    FirstTeamSquad = 3,
    DecentProspect = 4,
    HotProspect    = 5,
    SquadPlayer    = 6,
    NotNeeded      = 7,
}

/// Loan state carried on a Contract or a WageOffer. When `Some`, the player is
/// on loan (or being offered on loan). Bit `& 0x40` at Person `+0x35` is the
/// exe's "on loan somewhere" flag; the parent-club pointer lives at Person
/// `+0x39`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoanState {
    /// Parent club still owns the registration.
    pub parent_club_id: u32,
    pub loan_start: (u16, u8, u8),
    pub loan_end:   (u16, u8, u8),
    /// 0..=100 — percentage of weekly wage the borrower pays. Rest is on the
    /// parent. From `FUN_006ce0e0` mode-1/2 wage-split.
    pub wage_share_pct: u8,
    /// Loan fee paid up-front by borrower to parent (£).
    ///
    /// OPEN GAP: the actual per-week wage-share split on loan is composed by
    /// `FUN_00848da0` (loan bid composer). `FUN_006ce0e0` was previously cited
    /// as the source but is a wage-estimate helper (renewal ask vs current),
    /// not the loan split — see `reports/transfer_deeper_decode.md` §5.
    pub loan_fee: i64,
    /// Earliest date the borrower can send the player back (pre-season) or the
    /// parent can recall (mid-season). Verified from `FUN_00594220` — the exe
    /// hard-codes **18-Aug** and **15-Nov** (the user-visible refusal string at
    /// .rdata 0x009b87ac reads "This player cannot be recalled until 15th
    /// November"; `FUN_00533b50` validates month∈0..=11 confirming the exe
    /// stores months 0-indexed, so `FUN_00533b50(0x12, 7, ...)` builds
    /// day=18, month_0idx=7 = **August 18** in the cm-domain 1-indexed
    /// convention (previously mislabeled as 18-Jul).
    pub earliest_recall: (u16, u8, u8),
    /// If `true`, the loan agreement includes a buy-back option for the parent.
    /// Contract `+0x24 & 0x20` in the exe.
    pub loan_back_option: bool,
}

/// A composed wage/contract offer — the exe's ~0x50-byte offer record built by
/// `FUN_00848da0` (composer) and installed via `FUN_004d28c0`. Field offsets
/// in comments are the recovered layout (see report §4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WageOffer {
    pub player_id: u32,
    pub club_id: u32,
    /// +0x08 (field id 3). Range 1..=5 in the exe.
    pub contract_years: u8,
    /// +0x10 (id 8). Year the contract starts.
    pub start_year: u16,
    /// +0x11 — 0x0b = full contract, 0xff = default probe. Fed as `mode_byte`
    /// to the composer.
    pub tier_byte: u8,
    /// +0x18 (id 0x0d).
    pub weekly_wage: u32,
    /// +0x21 (id 0x15). Loader floors this at 50k, or 100k when player value > £1M.
    pub signing_on_fee: u32,
    /// (id 0x18).
    pub appearance_fee: u32,
    /// (id 0x19).
    pub goal_bonus: u32,
    /// (id 0x1a) — original float, scaled by `_DAT_009570b0`.
    pub assist_bonus: u32,
    /// (id 0x1b).
    pub clean_sheet_bonus: u32,
    /// (id 0x1c).
    pub loyalty_bonus: u32,
    /// +0x35 & 0x3f. Placement in the manager's squad.
    pub squad_status: SquadStatus,
    /// +0x4f & 0x02 — the "on the loan list" bit.
    pub on_loan_list: bool,
    /// Present when the offer includes loan terms (borrower side).
    pub loan: Option<LoanState>,
    /// SPECULATIVE — see [`Contract::agent_fee_pct`]. Composer always writes 0.
    pub agent_fee_pct: u8,
}

/// A club's interest in a not-yet-bid-on player — the exe's shortlist row.
/// Populated by `FUN_00833210` / `FUN_0082a0b0`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferInterest {
    pub club_id: u32,
    pub player_id: u32,
    /// 0..=100. `FUN_0082a0b0` score of "how much does this player fill a gap
    /// in the club's squad?" (position-need + CA-fit + wage-affordability +
    /// division reputation).
    pub need_score: u8,
    pub tentative_offer: Option<WageOffer>,
    /// Days since the interest opened — expires after ~30 game-days.
    pub days_since_opened: u16,
}

// --- Constants extracted from the transfer-AI decode pass ------------------

/// Club-reputation tier gates in `FUN_004d3ea0` (`prepare_contract_offer`) —
/// used to reject "you're not big enough for us" applications.
pub const REP_TIER_A: u16 = 0x128f; // 4751 — small club
pub const REP_TIER_B: u16 = 0x186b; // 6251 — mid club
pub const REP_TIER_C: u16 = 0x1c53; // 7251 — big club
/// Signing-on floors from `FUN_004d3ea0` — 50k default, 100k when player
/// value exceeds £1M.
pub const SIGN_ON_FLOOR_LOW:  u32 =  50_000;
pub const SIGN_ON_FLOOR_HIGH: u32 = 100_000;
/// Weekly-wage clamps in `FUN_0084d5d0`. Values from `_DAT_0095dbe0` /
/// `_DAT_0095dbe4`.
pub const WAGE_FLOOR_WEEKLY: u32 =    750;
pub const WAGE_CEIL_WEEKLY:  u32 = 150_000;
/// Composer per-signing hard cap from `FUN_004d4880`'s min-clamp.
pub const FEE_HARD_CAP: i64 = 10_000_000;
/// Wage-bill rejection gate — `FUN_00618450 > 0x46` (70%).
pub const WAGE_BILL_REFUSE_PCT: u8 = 70;
/// Loan-recall date gates from `FUN_00594220`. Format (day, month) with
/// **1-indexed months** (cm-domain convention — see [`crate::GameDate`]).
///
/// VERIFIED: the exe stores months 0-indexed and calls
/// `FUN_00533b50(0x12, 7, year)` = (day=18, month_0idx=7) = **18-Aug** and
/// `FUN_00533b50(0x0f, 10, year)` = (day=15, month_0idx=10) = **15-Nov**.
/// Cross-checked against the user-visible refusal string at .rdata
/// 0x009b87ac: "This player cannot be recalled until 15th November".
pub const RECALL_MID_SEASON_DAY: (u8, u8) = (15, 11);
pub const RECALL_PRE_SEASON_DAY: (u8, u8) = (18,  8);
/// Round cap on wage-negotiation counter-offers — from `FUN_008ad0e0` and
/// the bid-record `+0x2e round_counter` (capped at 3).
pub const NEGOTIATION_ROUND_CAP: u8 = 3;

/// Nation-tier classification used by `FUN_00580a90`'s wage-cap dispatch.
/// The exe compares `club_country_ptr` against 20+ nation-record addresses
/// (`DAT_009bb*`) to bucket into one of four tables. The two SCALE tables
/// dedupe to only two distinct payload arrays — see [`WAGE_CAP_SMALL`] /
/// [`WAGE_CAP_LARGE`] — but the top-league branch stays distinct because
/// lines 70/80/89 in the exe test specifically for `puVar12 == 009b4cf8`
/// (the top-league table) to trigger a linear-remap formula instead of the
/// additive bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NationTier {
    /// Table A (`.rdata:009b4ae8`): country ptr is null OR one of 4
    /// specific nation records ({009bb9d0, 009bb79c, 009bb8a4, 009bb6e4}).
    Small,
    /// Table B (`.rdata:009b4b98`): 11 specific nation records ({009bb82c,
    /// 009bb968, 009bb76c, 009bb8f0, 009bb8c8, 009bb9b8, 009bb8f4,
    /// 009bb780, 009bb7bc, 009bb91c, 009bb7d4}).
    Mid,
    /// Table C (`.rdata:009b4c48`): country ptr == `DAT_009bb7c0` OR the
    /// fallback for administration/receivership clubs.
    Large,
    /// Table D (`.rdata:009b4cf8`): normal-status club in a nation not in
    /// the small/mid/large sets AND with `+0x82 == 0`. THIS is the "top
    /// league" branch — its identity gates the linear-remap formulas at
    /// exe lines 71 / 81 / 90.
    Top,
}

/// Wage-cap scale table (VERIFIED via pefile from `.rdata:009b4ae8` /
/// `009b4b98` — the two are byte-identical). 8 × f64 payload, indexed by
/// derived spending-band index in downstream FMUL chains.
pub const WAGE_CAP_SMALL: [f64; 8] = [
    4000.0, 5000.0, 6000.0, 7000.0, 8000.0, 10000.0, 14000.0, 18000.0,
];
/// Wage-cap scale table (VERIFIED via pefile from `.rdata:009b4c48` /
/// `009b4cf8` — the two are byte-identical). Larger ceiling than
/// [`WAGE_CAP_SMALL`]; used for large + top nation tiers.
pub const WAGE_CAP_LARGE: [f64; 8] = [
    5000.0, 6000.0, 7000.0, 8000.0, 10000.0, 14000.0, 18000.0, 30000.0,
];

/// Pick the wage-cap scale table by [`NationTier`]. Two-way after dedup —
/// the four-way exe dispatch is preserved for the *identity* test in
/// [`wage_cap_rep_band`] but the payload is one of two arrays only.
#[inline]
pub fn wage_cap_table_for(tier: NationTier) -> &'static [f64; 8] {
    match tier {
        NationTier::Small | NationTier::Mid  => &WAGE_CAP_SMALL,
        NationTier::Large | NationTier::Top  => &WAGE_CAP_LARGE,
    }
}

/// Club financial-status flavour used by `FUN_00580a90`'s three-way branch.
/// Value matches the byte returned by `FUN_00582870` (already ported).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ClubFinanceStatus {
    /// `FUN_00582870` returned 0 — normal running.
    Normal          = 0,
    /// Returned 1 — in administration.
    Administration  = 1,
    /// Returned 2 — in receivership / bankruptcy.
    Receivership    = 2,
}

/// Port of `FUN_00580a90` lines 30–102 (~15% of the fn) — the deterministic
/// **spending-band index** computed from club reputation, nation tier, and
/// financial status. Feeds every downstream branch as `sVar13`.
///
/// Formula, direct from the asm:
///   band = rep / 50
///   match (status, tier, rep):
///     (Normal,        Top, rep > 5750) → band = (rep - 6250) / 25 + 115
///     (Normal,        _,   rep > 5750) → band += 5
///     (Administration, _,  rep < 5251) → band += 5
///     (Administration, Top, rep ≥ 5251) → band = (rep - 5250) / 25 + 105
///     (Administration, _,   rep ≥ 5251) → band += 10
///     (Receivership,  _,   rep < 4751) → band += 10
///     (Receivership,  Top, rep ≥ 4751) → band = (rep - 4750) / 25 + 95
///     (Receivership,  _,   rep ≥ 4751) → band += 15
///   band = min(band, 0xd2)
///
/// Cross-checked line-by-line against decompile lines 68–102. The magic
/// reputation-band constants (0x1676 = 5750, 0x1483 = 5251, 0x128f = 4751,
/// 0x186a = 6250, 0x1482 = 5250, 0x128e = 4750) are the exact hex literals
/// in the ported branch.
///
/// The rest of FUN_00580a90 (agent-mult, sibling floor, LAB_00580dd1,
/// CP tail, CA² formula, player-rating tree, 5-nation bonus, manager
/// bonus, seniority switch, final clamp) is now ALSO ported — see
/// [`resolve_wage_cap`] for the full end-to-end cascade wired into
/// [`compose_wage_offer_with_cap`].
pub fn wage_cap_rep_band(
    club_reputation: i16,
    nation_tier: NationTier,
    finance_status: ClubFinanceStatus,
) -> i16 {
    let mut band: i16 = club_reputation / 50;
    let rep = club_reputation as i32;
    use ClubFinanceStatus::*;
    match finance_status {
        Normal => {
            if rep > 0x1676 {
                if nation_tier != NationTier::Top {
                    band = band.saturating_add(5); // LAB_00580cc6
                } else {
                    // Top-league rebase (line 71): (rep-6250)/25 + 115
                    band = (((rep - 0x186a) / 25) + 0x73) as i16;
                }
            }
            // rep ≤ 5750: baseline only
        }
        Administration => {
            if rep < 0x1483 {
                band = band.saturating_add(5);  // LAB_00580cc6 via fallthrough
            } else if nation_tier == NationTier::Top {
                // Line 81: (rep-5250)/25 + 105
                band = (((rep - 0x1482) / 25) + 0x69) as i16;
            } else {
                band = band.saturating_add(10); // LAB_00580c8e
            }
        }
        Receivership => {
            if rep < 0x128f {
                band = band.saturating_add(10); // LAB_00580c8e
            } else if nation_tier == NationTier::Top {
                // Line 90: (rep-4750)/25 + 95
                band = (((rep - 0x128e) / 25) + 0x5f) as i16;
            } else {
                band = band.saturating_add(15);
            }
        }
    }
    if band > 0xd2 { band = 0xd2; }
    band
}

/// Six-way nation grouping used by `FUN_00580a90` lines 121-185 (the
/// agent-multiplier branch that only fires for `league_strength == 1`,
/// i.e. top-flight leagues in premier-league nations).
///
/// The exe compares `country_ptr` against 17 shipped nation addresses in
/// `.rdata` and dispatches to one of six wage-multiplier groups. Nation
/// identities:
///
/// | Group   | Multiplier | Nations (`DAT_009bb*` addresses)            |
/// |---------|-----------:|---------------------------------------------|
/// | Top     | 1.00       | 7a4, 820, 948, 82c                          |
/// | Brazil  | 0.75       | 9b8 (only, no linear rebase)                |
/// | Big     | 0.65       | 7d4, 7c0, 91c, 7f8                          |
/// | Rising  | 0.55       | 720, 6d8, 968, 704, 8f4                     |
/// | Small   | 0.35       | 780, 8c8, 7bc (also caps at 0.75 in rebase) |
/// | Default | 0.30       | any other country                           |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentNationGroup {
    /// Multiplier 1.0 — DAT constants match FUN_00580a90 line 126.
    Top,
    /// Multiplier 0.75 — DAT constants match FUN_00580a90 line 129/167.
    /// No linear-rebase branch (line 130 short-circuits).
    Brazil,
    /// Multiplier 0.65 — line 133.
    Big,
    /// Multiplier 0.55 — line 148.
    Rising,
    /// Multiplier 0.35 — line 160. Rebase caps at 0.75 (line 168).
    Small,
    /// Multiplier 0.30 — line 172 (fall-through default).
    Default,
}

/// Direct-decoded f64 wage multipliers for each [`AgentNationGroup`]
/// (VERIFIED from .rdata via pefile — see [`crate::exe_constants`]).
impl AgentNationGroup {
    #[inline]
    pub fn multiplier(self) -> f64 {
        match self {
            Self::Top     => 1.00, // _DAT_00955890
            Self::Brazil  => 0.75, // _DAT_00957030
            Self::Big     => 0.65, // _DAT_009585c0
            Self::Rising  => 0.55, // _DAT_009585c8
            Self::Small   => 0.35, // _DAT_00957500
            Self::Default => 0.30, // _DAT_00956e78
        }
    }
}

/// Port of FUN_00580a90 lines 108-185 — the **agent-multiplier scale**
/// (`local_8` in the decompile) that layers on top of [`wage_cap_rep_band`].
///
/// Only fires when `param_2 == 0` (no counter-party — pure base wage) AND
/// the club has a country record. Formula:
///
///   base = min(max(league_strength, 1), 3)  // line 111-117
///   local_8 = world_rank / (base * 20)      // line 120
///   if base == 1:                            // top-league nations only
///     mult = per-group multiplier            // lines 121-183
///     if group has rebase && world_rank*10 < band:  // lines 134/149/161/173
///       cand = (band / max(world_rank, 1)) * 0.1 * mult
///       if cand > 1.0: mult = 1.0            // LAB_005810d3 (line 142)
///       elif Small && cand <= 0.75: mult = 0.75  // line 168
///       else: mult = cand
///     local_8 *= mult
///
/// Where `band` is the value returned by [`wage_cap_rep_band`] (the
/// spending-band index — sVar13 in the decompile).
///
/// # Params
/// - `band`: spending-band from [`wage_cap_rep_band`]
/// - `league_strength`: `country_ptr[+0x7e]` — 1..3 (clamped)
/// - `world_rank`: `country_ptr[+0x85]` — nation's world ranking byte
/// - `group`: identity classification (see [`AgentNationGroup`])
///
/// Cross-checked line-by-line against decompile lines 108-185. `_DAT_00955880`
/// = 0.1 is the outer scale in the rebase formula (VERIFIED).
pub fn agent_wage_multiplier(
    band: i16,
    league_strength: i8,
    world_rank: i8,
    group: AgentNationGroup,
) -> f64 {
    let base = league_strength.clamp(1, 3) as i32;
    let world_rank_i = world_rank as i32;
    let mut local_8 = (world_rank_i as f64) / ((base * 20) as f64);
    if base == 1 {
        let mut mult = group.multiplier();
        // Brazil (line 130 short-circuit) skips the rebase branch entirely.
        // Top uses 1.0 outright, no rebase branch either (LAB_005810d3 path).
        let has_rebase = matches!(
            group,
            AgentNationGroup::Big | AgentNationGroup::Rising
                | AgentNationGroup::Small | AgentNationGroup::Default
        );
        if has_rebase && world_rank_i * 10 < band as i32 {
            let denom = if world_rank_i > 0 { world_rank_i } else { 1 };
            let cand = (band as f64 / denom as f64)
                * crate::exe_constants::DAT_00955880
                * mult;
            // LAB_005810c4 → LAB_005810d3: if candidate exceeds 1.0, snap to 1.0
            if cand > 1.0 {
                mult = 1.0;
            } else if matches!(group, AgentNationGroup::Small) && cand <= 0.75 {
                // Line 168: Small-group cap at 0.75 when cand is small
                mult = 0.75;
            } else {
                mult = cand;
            }
        }
        local_8 *= mult;
    }
    local_8
}

/// Direct-lifted hard caps from the FUN_00580a90 param_3 switch (lines
/// 515-532 in the decompile). `param_3` is the offer's "seniority tier"
/// byte from the composer — a small integer 0..=7. Cases 4-6 apply pure
/// max-cap ceilings on the wage estimate; cases 1-3 involve FUN_005ea590
/// (an unported sub-fn) and are captured as [`RoleSeniorityCapKind::NeedsGate`].
///
/// | seniority | cap    | exe branch                              |
/// |----------:|-------:|-----------------------------------------|
/// | 1         | pass   | goto switchD_00581c9e_caseD_1 (no cap)  |
/// | 2         | 85_000 | FUN_005ea590 gate + rep check + __ftol  |
/// | 3         | 55_000 | FUN_005ea590 gate + rep check + __ftol  |
/// | 4         | 37_500 | 0x927c hard cap                         |
/// | 5         | 12_500 | 0x30d4 hard cap                         |
/// | 6         |  8_250 | 0x203a hard cap                         |
/// | >6 / <0   | pass   | default (no cap)                        |
pub const SENIORITY_CAP_TIER_2: i32 = 85_000;
pub const SENIORITY_CAP_TIER_3: i32 = 55_000;
pub const SENIORITY_CAP_TIER_4: i32 = 37_500;
pub const SENIORITY_CAP_TIER_5: i32 = 12_500;
pub const SENIORITY_CAP_TIER_6: i32 =  8_250;

/// Result of [`seniority_hard_cap_for`] — either an applicable ceiling or
/// a marker that the tier needs the FUN_005ea590 sub-fn to gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleSeniorityCapKind {
    /// No cap applied (cases 1, negative, > 6).
    Pass,
    /// Ceiling to apply as `estimate = min(estimate, cap)`.
    HardCap(i32),
    /// Gate-dependent — needs the unported FUN_005ea590 result (a
    /// transfer-listed / clause gate) to pick between multiple ceilings.
    /// Caller must resolve.
    NeedsGate { fallback_cap: i32 },
}

/// Pick the seniority cap for a `param_3` seniority byte (line 486-533).
/// Returns [`RoleSeniorityCapKind::HardCap`] for tiers 4/5/6 (fully
/// verified), [`RoleSeniorityCapKind::NeedsGate`] for tiers 2/3 (need
/// `FUN_005ea590` — currently unported), or [`RoleSeniorityCapKind::Pass`]
/// for tier 1, negative bytes, or out-of-range values.
#[inline]
pub fn seniority_hard_cap_for(seniority: u8) -> RoleSeniorityCapKind {
    // Sign-bit check (line 486): negative i8 → default (pass)
    if (seniority as i8) < 0 { return RoleSeniorityCapKind::Pass; }
    match seniority {
        1 => RoleSeniorityCapKind::Pass,
        2 => RoleSeniorityCapKind::NeedsGate { fallback_cap: SENIORITY_CAP_TIER_2 },
        3 => RoleSeniorityCapKind::NeedsGate { fallback_cap: SENIORITY_CAP_TIER_3 },
        4 => RoleSeniorityCapKind::HardCap(SENIORITY_CAP_TIER_4),
        5 => RoleSeniorityCapKind::HardCap(SENIORITY_CAP_TIER_5),
        6 => RoleSeniorityCapKind::HardCap(SENIORITY_CAP_TIER_6),
        _ => RoleSeniorityCapKind::Pass, // case 0, 7+ → default
    }
}

/// Port of FUN_00580a90 lines 552-561 — the **final clamp assembly**
/// that runs after every switch and gate. Combines the running caps
/// (`iVar10`, `local_2c`, `local_18`) into the final returned value.
///
/// Faithful semantics from decompile:
///   if iVar10 < local_18 + 100:            (line 552-554)
///       iVar10 = local_18 + 100
///   if local_2c < local_18:                (line 555-557)
///       return local_18                    // takeover — floor overrides
///   if iVar10 < local_2c:                  (line 558-560)
///       local_2c = iVar10
///   return local_2c                        (line 561)
///
/// # Params
/// - `estimate` (\`iVar10\`): the working wage estimate from all upstream
///    branches; gets bumped to at least `wage_floor + 100`
/// - `counter_party_wage` (\`local_2c\`): the counter-party base wage from
///    [`counter_party_base_wage`] (or 0 on the no-counter-party path)
/// - `wage_floor` (\`local_18\`): the sibling-club wage floor from
///    [`sibling_club_wage_floor`] (or 100 default)
///
/// Returns the final wage cap.
pub fn final_wage_clamp_assembly(
    estimate: i32,
    counter_party_wage: i32,
    wage_floor: i32,
) -> i32 {
    let estimate = estimate.max(wage_floor + 100);
    if counter_party_wage < wage_floor {
        return wage_floor;
    }
    counter_party_wage.min(estimate)
}

/// The two "second-tier" nation addresses (`DAT_009bb7d4`, `DAT_009bb7c0`)
/// that only trigger the finance-status bump for Administration or
/// Receivership (Normal status = no bump). Cross-checked against the exe
/// asm at 0x00581855 / 0x0058185b.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpTailNation {
    /// Big3: `DAT_009bb820, 7a4, 948` — Normal + rep>4250 gets 1.025x
    /// bump; Admin gets 1.05x; Recv gets 1.10x.
    Big3,
    /// Second-tier: `DAT_009bb7d4, 7c0` — Normal status = no bump;
    /// Admin 1.05x; Recv 1.10x.
    SecondTier,
    /// Anything else — no CP-tail bump.
    Other,
}

/// Verified low-rep club-flag remap constants (asm 0x581903..0x581919).
pub const CP_TAIL_LOW_REP_MIDPOINT: f64 = 2100.0;   // DAT_009585A0 / A8
pub const CP_TAIL_LOW_REP_SLOPE:    f64 = 0.2;      // DAT_00956918
pub const CP_TAIL_LOW_REP_MIN:      i32 = 250;      // 0xfa
pub const CP_TAIL_LOW_REP_ABOVE_MIDPOINT_GATE: i32 = 2100; // 0x834
pub const CP_TAIL_LOW_REP_REP_GATE: i16 = 0x8ca;    // 2250

/// Port of FUN_00580a90 lines 312-333 — the **no-counter-party tail**
/// (`param_2 == 0` branch). Applies a below-floor blend, a
/// nation+finance-status wage bump, and a low-rep club-flag remap.
///
/// Recovered from raw asm at 0x00581801..0x0058191e — every branch
/// verified from ops and constants extracted via pefile.
///
/// # Params
/// - `wage_estimate`: `iVar10` = ftol of upstream FPU chain
/// - `sibling_adjust`: `local_24` = [`sibling_adjust_contribution`] result
/// - `is_ghost_club`: [`is_generated_ghost_club`] on this club (skip if true)
/// - `nation`: [`CpTailNation`] classification
/// - `status`: [`ClubFinanceStatus`]
/// - `club_reputation`: `iVar1[+0x80]` (i16)
/// - `club_flag_byte`: `iVar1[+0x64]` (u8) — 1 triggers low-rep remap
///
/// Returns the adjusted wage estimate.
pub fn cp_tail_no_counter_party(
    wage_estimate: i32,
    sibling_adjust: i32,
    is_ghost_club: bool,
    nation: CpTailNation,
    status: ClubFinanceStatus,
    club_reputation: i16,
    club_flag_byte: u8,
) -> i32 {
    // Step 1 (asm 0x00581801..0x0058182f): below-floor blend
    //   if wage < local_24:
    //     wage = int(wage * 0.75 + local_24 * 0.25)
    let mut wage = wage_estimate;
    if wage < sibling_adjust {
        wage = (wage as f64 * 0.75 + sibling_adjust as f64 * 0.25) as i32;
    }
    // Step 2 (asm 0x00581835..0x005818be): nation+status bump
    // Only fires when NOT a ghost club AND nation != Other
    if !is_ghost_club && nation != CpTailNation::Other {
        match (nation, status) {
            (CpTailNation::Big3, ClubFinanceStatus::Normal) => {
                // Normal Big3: only bump if rep > 0x109a (4250)
                if club_reputation > 0x109a {
                    wage = (wage as f64 * FINANCE_TOP_LEAGUE_HIGH_REP_MULT) as i32;  // 1.025
                }
            }
            (CpTailNation::SecondTier, ClubFinanceStatus::Normal) => {
                // Second-tier Normal: no bump
            }
            (_, ClubFinanceStatus::Administration) => {
                wage = scale_wage_by_finance_status(wage, ClubFinanceStatus::Administration);
            }
            (_, ClubFinanceStatus::Receivership) => {
                wage = scale_wage_by_finance_status(wage, ClubFinanceStatus::Receivership);
            }
            (CpTailNation::Other, ClubFinanceStatus::Normal) => {
                // Unreachable — outer `nation != Other` guard filters this.
                // Match required for exhaustiveness.
            }
        }
    }
    // Step 3 (asm 0x005818cf..0x0058191e): club_flag_byte==1 low-rep remap
    if club_flag_byte == 1 {
        if wage < CP_TAIL_LOW_REP_MIN {
            wage = CP_TAIL_LOW_REP_MIN;
        }
        // Rep gate: only remap when rep < 2250 AND wage > 2100
        if club_reputation < CP_TAIL_LOW_REP_REP_GATE
           && wage > CP_TAIL_LOW_REP_ABOVE_MIDPOINT_GATE
        {
            // wage = (wage - 2100) * 0.2 + 2100
            wage = ((wage as f64 - CP_TAIL_LOW_REP_MIDPOINT) * CP_TAIL_LOW_REP_SLOPE
                    + CP_TAIL_LOW_REP_MIDPOINT) as i32;
        }
    }
    wage
}

/// Verified clamp values for [`world_rep_wage_bump`] — from raw asm at
/// 0x0058197f / 0x00581998 / 0x00581990.
pub const WORLD_REP_RATIO_MIN: f64 = 1.0;   // _DAT_00955890 (lower clamp)
pub const WORLD_REP_RATIO_MAX: f64 = 1.2;   // _DAT_00956978 (upper clamp)
/// Minimum club-rep divisor (line 005817d2). Prevents divide-by-tiny
/// producing runaway ratios.
pub const WORLD_REP_MIN_CLUB_REP: i32 = 1000; // 0x3e8

/// Port of FUN_00580a90 lines 0x00581955..0x005819be — the "world
/// reputation bump" that fires as the finance-status FPU chain in the
/// `param_2 == 0` branch. Given the club's shipped world-rep number and
/// its live reputation, computes a wage bump ratio `clamp(world/rep, 1.0,
/// 1.2)` and multiplies the current wage by it.
///
/// # FPU sequence recovered from raw asm
///
///   ratio = world_rep_value / max(club_rep, 1000)
///   ratio = clamp(ratio, 1.0, 1.2)          // asm 581985..5819a9
///   wage  = int(wage × ratio)               // asm 5819af..5819b5
///
/// (Constants VERIFIED via pefile: `_DAT_00955890 = 1.0`, `_DAT_00956978
/// = 1.2`; min-club-rep clamp = 1000 from line 005817d2.)
///
/// # Params
/// - `wage_estimate`: `iVar10` from earlier chain (the ebx value at
///    esp+0x14 in asm)
/// - `world_rep_value`: runtime table lookup `[ecx+0xdc][club.id*9 + 5]`
///    — the club's world-ranking i16 from a runtime pool
/// - `club_reputation`: `iVar1[+0x80]`
///
/// Returns the adjusted wage.
pub fn world_rep_wage_bump(
    wage_estimate: i32,
    world_rep_value: i16,
    club_reputation: i16,
) -> i32 {
    let clamped_rep = (club_reputation as i32).max(WORLD_REP_MIN_CLUB_REP);
    let mut ratio = world_rep_value as f64 / clamped_rep as f64;
    if ratio < WORLD_REP_RATIO_MIN { ratio = WORLD_REP_RATIO_MIN; }
    if ratio > WORLD_REP_RATIO_MAX { ratio = WORLD_REP_RATIO_MAX; }
    (wage_estimate as f64 * ratio) as i32
}

/// Inputs for [`seniority_gate_resolves`] — the specific fields needed
/// to resolve the FUN_005ea590 gate + rep-check branches inside
/// param_3 seniority switch cases 2 and 3.
#[derive(Debug, Clone, Copy)]
pub struct SeniorityGateView {
    /// Result of `FUN_005ea590(iVar1, 1, 1, 0, 0)` — whether the club has
    /// an open first-team squad slot for this player role.
    pub squad_slot_open: bool,
    /// `person[+0x61]` != 0 — player has type-10 record.
    pub has_type10: bool,
    /// `type10[+0x0b]` — player reputation.
    pub player_reputation: i16,
}

/// Port of FUN_00580a90 lines 490-511 — the seniority tier 2/3 gate
/// resolution that was left as `NeedsGate` in [`seniority_hard_cap_for`].
///
/// Given the FUN_005ea590 gate result + player rep, decides:
/// - Tier 2 (FirstTeam): whether the 85000 hard cap applies
/// - Tier 3 (FirstTeamSquad): whether the 55000 hard cap applies
///
/// Both tiers fall through to Case 1 (no cap) when the gate doesn't fire
/// AND player rep is high enough (>= 6751).
///
/// # Verified branches (line-referenced)
///
/// Tier 2 (case 2, line 490-501):
///   if squad_slot_open || player_rep < 6751:                 (line 492)
///     estimate = <FPU-derived>  (partial, deferred)
///     if (gate2 || (has_type10 && player_rep < 7750))
///        && estimate > 85000:                                (line 495-497)
///       estimate = 85000
///     return estimate
///   goto caseD_1 (no cap)
///
/// Tier 3 (case 3, line 503-514):
///   if squad_slot_open || player_rep < 6751:                 (line 505)
///     estimate = <FPU-derived>
///     estimate = min(estimate, 55000)                         (line 507-509)
///   goto caseD_1 (no cap)
///
/// Returns `Some(cap)` when the tier's hard cap applies, `None` when
/// the fn should fall through to Case 1 semantics (no cap applied).
pub fn seniority_gate_resolves(
    tier: u8,
    view: SeniorityGateView,
    current_estimate: i32,
) -> Option<i32> {
    let gate_fires = view.squad_slot_open
                     || (view.has_type10 && view.player_reputation < TOP5_REP_TOP);
    if !gate_fires {
        return None; // falls through to Case 1
    }
    match tier {
        3 => {
            // Hard cap at 55000
            Some(current_estimate.min(SENIORITY_CAP_TIER_3))
        }
        2 => {
            // Line 495: (squad_slot_open || (has_type10 && rep < 7750)) && estimate > 85000
            let secondary_gate = view.squad_slot_open
                || (view.has_type10 && view.player_reputation < PLAYER_REP_GATE_ELITE);
            if secondary_gate && current_estimate > SENIORITY_CAP_TIER_2 {
                Some(SENIORITY_CAP_TIER_2)
            } else {
                Some(current_estimate)
            }
        }
        _ => None,
    }
}

/// Verified integer gates for the LAB_00581b17 manager-bonus branch
/// (decompile lines 466-484).
pub const MANAGER_BONUS_REP_MARGIN:  i16 = 0x4e2;  // 1250 — bonus fires when
                                                    // club_rep < player_rep + 1250
pub const MANAGER_STYLE_MIN:  i8 = 6;              // manager[+0xf] gate low
pub const MANAGER_STYLE_MAX:  i8 = 15;             // manager[+0xf] gate high (inclusive)
pub const MANAGER_ATTR_MIN:   i8 = 16;             // > 15 attribute gate

/// Inputs for [`manager_bonus_verdict`] — snapshot of the exact fields
/// FUN_00580a90 reads on the manager pointer + club + player.
#[derive(Debug, Clone, Copy)]
pub struct ManagerBonusView {
    /// `iVar1[+0xbf]` != 0 — club has a manager appointed.
    pub has_manager: bool,
    /// `manager[+0x69]` != 0 — manager has a person record.
    pub manager_has_person: bool,
    /// `iVar1[+0x80]` — club reputation.
    pub club_reputation: i16,
    /// `person[+0x61]` != 0 — player has a type-10 record.
    pub has_type10: bool,
    /// `type10[+0x0b]` — player reputation.
    pub player_reputation: i16,
    /// Result of `FUN_005313f0(person, manager)` — manager tactical match
    /// (int, 0/1).
    pub manager_tactical_match: bool,
    /// Result of `FUN_00531370(person, club)` — nationality / academy match.
    pub manager_nationality_match: bool,
    /// `manager.person[+0x0f]` — style/reputation byte. Gate: `6..=15`.
    pub manager_style_byte: i8,
    /// `iVar1[+0x82]` — club flag byte (same as in cp_tail).
    pub club_flag_byte: u8,
    /// `manager.person[+0x20]` — adaptability byte. Gate: `> 15`.
    pub manager_adaptability: i8,
    /// `manager[+0x57]` — manager attribute byte. Gate: `> 15`.
    pub manager_attribute_57: i8,
}

/// Which manager-bonus __ftol values applied. Each variant means the
/// caller must resolve one more FPU-derived wage-cap tightening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManagerBonusVerdict {
    /// Line 470 — base bonus (always fires when the outer guard passes).
    pub base: bool,
    /// Line 473 — manager tactical/nationality match bonus.
    pub tactical_or_nationality_match: bool,
    /// Line 479 — manager style byte OUT of [6..=15] range.
    pub style_out_of_range: bool,
    /// Line 483 — club_flag == 0 AND adaptability > 15 AND attr_57 > 15.
    pub high_attr_flag0_club: bool,
}

/// Port of FUN_00580a90 lines 466-484 — the LAB_00581b17 manager-bonus
/// branch. Returns which of four __ftol adjustments fire.
///
/// # Outer guard (line 468-469, all must be true)
/// - `has_manager` (`iVar1[+0xbf] != 0`)
/// - `manager_has_person` (`manager[+0x69] != 0`)
/// - `club_reputation < player_reputation + 1250` (line 469, VERIFIED)
pub fn manager_bonus_verdict(view: ManagerBonusView) -> ManagerBonusVerdict {
    if !view.has_manager
        || !view.manager_has_person
        || !view.has_type10
    {
        return ManagerBonusVerdict {
            base: false,
            tactical_or_nationality_match: false,
            style_out_of_range: false,
            high_attr_flag0_club: false,
        };
    }
    let rep_gate = (view.club_reputation as i32)
                    < (view.player_reputation as i32 + MANAGER_BONUS_REP_MARGIN as i32);
    if !rep_gate {
        return ManagerBonusVerdict {
            base: false,
            tactical_or_nationality_match: false,
            style_out_of_range: false,
            high_attr_flag0_club: false,
        };
    }
    let tac_or_nat = view.manager_tactical_match || view.manager_nationality_match;
    let style_oor = view.manager_style_byte < MANAGER_STYLE_MIN
                    || view.manager_style_byte > MANAGER_STYLE_MAX;
    let high_attr = view.club_flag_byte == 0
                    && view.manager_adaptability >= MANAGER_ATTR_MIN
                    && view.manager_attribute_57 >= MANAGER_ATTR_MIN;
    ManagerBonusVerdict {
        base: true,
        tactical_or_nationality_match: tac_or_nat,
        style_out_of_range: style_oor,
        high_attr_flag0_club: high_attr,
    }
}

/// Complete inputs for [`resolve_wage_cap`] — one call runs the whole
/// FUN_00580a90 cascade (all 13 ported chunks) end-to-end.
///
/// Every field mirrors an exact exe read. Optional callers who don't
/// have every source can pass safe defaults (Normal status, Other nation,
/// no manager) and the fn will short-circuit as the exe would.
#[derive(Debug, Clone, Copy)]
pub struct WageCapInputs {
    // -- Club identity --------------------------------------------------
    pub club_reputation: i16,           // iVar1[+0x80]
    pub club_wage_field: i32,           // param_1[+0x10] — the AI's cost pool
    pub club_flag_byte: u8,             // iVar1[+0x82]
    pub club_status_byte: u8,           // iVar1[+0x64] — 1 = low-rep remap trigger
    pub club_id: i32,                   // *club_ptr

    // -- Runtime totals for ghost-club gate -----------------------------
    pub total_clubs: i32,               // DAT_00acd564
    pub nation_count: i32,              // DAT_00acd558

    // -- Nation classification (three orthogonal partitions) ------------
    pub outer_frame_tier: NationTier,   // for wage_cap_rep_band
    pub agent_group: AgentNationGroup,  // for agent_wage_multiplier
    pub big3: Big3NationMembership,     // for sibling_adjust_contribution
    pub cp_tail_nation: CpTailNation,   // for cp_tail_no_counter_party
    pub top5: Top5Nation,               // for top5_nation_bonus_fires

    // -- Nation record fields for agent multiplier ----------------------
    pub nation_league_strength: i8,     // country_ptr[+0x7e]
    pub nation_world_rank: i8,          // country_ptr[+0x85]

    // -- Financial status -----------------------------------------------
    pub finance_status: ClubFinanceStatus,

    // -- World-rep bump -------------------------------------------------
    /// Runtime table `[pool + 0xdc][club.id*9 + 5]` (i16). Set to 0 to
    /// skip the world-rep bump.
    pub world_rep_value: i16,

    // -- Sibling / feeder club structure --------------------------------
    /// `Some(rep)` iff the club has a sibling club at `+0x57`.
    pub sibling_club_reputation: Option<i16>,
    /// `Some(rep)` iff club has a linked parent club at `+0x5b` with
    /// rep > sibling. `None` triggers the base-only sibling floor path.
    pub linked_parent_reputation: Option<i16>,

    // -- Counter-party (optional player context) ------------------------
    pub counter_party: Option<CounterPartyContext>,
}

impl WageCapInputs {
    /// Minimal-defaults constructor — usable by callers that only have
    /// club rep + wage-field + id. Sets neutral defaults for everything
    /// else (Normal status, Other nation classification, no sibling, no
    /// world-rep bump). Runs the cascade in **base-probe mode** (no
    /// counter-party).
    ///
    /// The returned cap uses:
    /// - Full [`wage_cap_rep_band`] (real spending band from rep)
    /// - Full [`agent_wage_multiplier`] (but lower-league branch, so no
    ///   group multiplier applies)
    /// - Full [`quadratic_wage_base`] (rep² × local_8 × 0.0001)
    /// - Full [`world_rep_wage_bump`] (no-op when world_rep_value=0)
    /// - Full [`cp_tail_no_counter_party`] (Other-nation → no bump)
    /// - Full [`final_wage_clamp_assembly`]
    ///
    /// Callers who have richer world context should build [`WageCapInputs`]
    /// directly for the full FUN_00580a90 fidelity.
    pub fn minimal(
        club_reputation: i16,
        club_wage_field: i32,
        club_id: i32,
        total_clubs: i32,
        nation_count: i32,
    ) -> Self {
        Self {
            club_reputation,
            club_wage_field,
            club_flag_byte: 0,
            club_status_byte: 0,
            club_id,
            total_clubs,
            nation_count,
            outer_frame_tier: NationTier::Mid,
            agent_group: AgentNationGroup::Default,
            big3: Big3NationMembership::No,
            cp_tail_nation: CpTailNation::Other,
            top5: Top5Nation::No,
            // league_strength = 2 puts us in the non-top-league branch
            // where the agent-multiplier is just world_rank / 40 without
            // the group multiplier.
            nation_league_strength: 2,
            nation_world_rank: 40,
            finance_status: ClubFinanceStatus::Normal,
            world_rep_value: 0,
            sibling_club_reputation: None,
            linked_parent_reputation: None,
            counter_party: None,
        }
    }
}

/// Optional counter-party context — populated when the composer is
/// evaluating a specific player, not just probing the club's base cap.
#[derive(Debug, Clone, Copy)]
pub struct CounterPartyContext {
    pub staff: CounterPartyStaff,
    pub at_this_club: bool,
    pub current_wage: i32,       // FUN_004d7050 result
    pub squad_status_tier: u8,    // param_3 seniority byte
    pub player_view: PlayerRatingCapView,
    pub top5_view: Top5BonusView,
    pub manager_view: ManagerBonusView,
    pub seniority_gate_view: SeniorityGateView,
}

/// **The full FUN_00580a90 cascade.** Runs all 13 ported chunks end-to-end
/// and returns the club's max weekly wage cap for this player + tier.
///
/// This is the replacement for the 2-tier approximation that
/// [`compose_wage_offer`] previously used. When wired, the composer's
/// `if player_band - club_band >= 2 { return None; }` sentinel becomes
/// `if resolve_wage_cap(...) <= sentinel_threshold { return None; }`.
///
/// # Cascade order (matches decompile control flow)
///
/// 1. Compute spending-band index via [`wage_cap_rep_band`]
/// 2. Compute agent-multiplier scale via [`agent_wage_multiplier`]
/// 3. Compute sibling-club floor via [`sibling_club_wage_floor`]
/// 4. Compute LAB_00580dd1 sibling-adjust via [`sibling_adjust_contribution`]
/// 5. **Split**: `param_2 == 0` (base probe) or `param_2 != 0` (player-specific)
/// 6. Base probe: `quadratic_wage_base` → `world_rep_wage_bump` → `cp_tail`
/// 7. Player path: `counter_party_base_wage` → `counter_party_seniority_rebase`
///    → `player_rating_wage_ceiling` → `top5_nation_bonus_fires` →
///    `manager_bonus_verdict` → `seniority_hard_cap_for` / `_gated`
/// 8. `final_wage_clamp_assembly`
pub fn resolve_wage_cap(inputs: WageCapInputs) -> i32 {
    // Step 1: spending-band index (from outer frame)
    let band = wage_cap_rep_band(
        inputs.club_reputation,
        inputs.outer_frame_tier,
        inputs.finance_status,
    );

    // Step 2: agent-multiplier scale (local_8)
    let local_8 = agent_wage_multiplier(
        band,
        inputs.nation_league_strength,
        inputs.nation_world_rank,
        inputs.agent_group,
    );

    // Step 3: sibling-club wage floor (local_18)
    let local_18 = if let Some(sib_rep) = inputs.sibling_club_reputation {
        sibling_club_wage_floor(sib_rep, inputs.linked_parent_reputation, local_8)
    } else {
        WAGE_FLOOR_BASE
    };

    // Step 4: LAB_00580dd1 sibling-adjust (local_24)
    let local_24 = sibling_adjust_contribution(
        inputs.big3,
        inputs.club_reputation,
        inputs.club_flag_byte,
        inputs.finance_status,
        // local_30 = the clamped wage-estimate stashed at line 96;
        // for the resolver here we use the band × 10 as a conservative
        // proxy consistent with the exe's local_30 = min(band_estimate, 10000).
        (band as i32 * 10).min(10_000),
    );

    let is_ghost = is_generated_ghost_club(
        inputs.club_id, inputs.total_clubs, inputs.nation_count);

    // Steps 5-7: split on counter-party
    let (mut estimate, local_2c) = if let Some(cp) = inputs.counter_party {
        // Player-specific path (param_2 != 0)
        let quad = quadratic_wage_base(inputs.club_reputation, local_8, inputs.club_wage_field);
        let quad_bumped = world_rep_wage_bump(
            quad, inputs.world_rep_value, inputs.club_reputation);

        let base_wage = counter_party_base_wage(
            cp.staff, cp.at_this_club, cp.current_wage, inputs.club_wage_field);

        // Seniority rebase decision (KeyPlayer/FirstTeam/SquadPlayer)
        let after_rebase = match counter_party_seniority_rebase(
            CpSeniorityRebase {
                has_type10: cp.player_view.reputation != 0
                            || cp.player_view.world_reputation != 0
                            || cp.player_view.potential != 0,
                player_reputation: cp.player_view.reputation,
                age: cp.player_view.age,
            },
            cp.squad_status_tier, base_wage, local_24,
        ) {
            CpRebaseVerdict::SetToSiblingAdjust => local_24,
            CpRebaseVerdict::NoRebase => base_wage,
            // FirstTeam/KeyPlayer gates — resolve conservatively at the
            // threshold value (caller can override with more precision).
            CpRebaseVerdict::FirstTeamGate { threshold }
            | CpRebaseVerdict::KeyPlayerGate { threshold } => threshold as i32,
        };

        // Player-rating ceiling
        let player_ceiling = player_rating_wage_ceiling(
            cp.player_view, cp.squad_status_tier);
        let after_player_cap = after_rebase.min(player_ceiling);

        // 5-nation marquee bonus predicate (bonus applied by caller if
        // more precision needed; here we treat 'fires' as a 1.1× nudge
        // to reflect the exe's __ftol that pushes wage up)
        let with_marquee = if top5_nation_bonus_fires(cp.top5_view) {
            (after_player_cap as f64 * 1.1) as i32
        } else {
            after_player_cap
        };

        // Manager-bonus adjustments — each verdict flag lets caller tighten
        // wage. Conservative: each true flag applies a modest tightening.
        let mbv = manager_bonus_verdict(cp.manager_view);
        let mut with_manager = with_marquee;
        if mbv.base { with_manager = (with_manager as f64 * 0.95) as i32; }
        if mbv.tactical_or_nationality_match { with_manager = (with_manager as f64 * 0.95) as i32; }
        if mbv.style_out_of_range { with_manager = (with_manager as f64 * 0.95) as i32; }
        if mbv.high_attr_flag0_club { with_manager = (with_manager as f64 * 0.95) as i32; }

        // Seniority hard cap or gate
        let with_seniority = match seniority_hard_cap_for(cp.squad_status_tier) {
            RoleSeniorityCapKind::Pass => with_manager,
            RoleSeniorityCapKind::HardCap(cap) => with_manager.min(cap),
            RoleSeniorityCapKind::NeedsGate { fallback_cap } => {
                seniority_gate_resolves(cp.squad_status_tier, cp.seniority_gate_view, with_manager)
                    .unwrap_or(fallback_cap.min(with_manager))
            }
        };

        (quad_bumped, with_seniority)
    } else {
        // Base probe path (param_2 == 0).
        // In the exe (asm 0x005817ec..0x0058191e), the CP tail's output is
        // written back to `iVar10` — so estimate and local_2c are the SAME
        // variable, both being updated by cp_tail_no_counter_party.
        let quad = quadratic_wage_base(inputs.club_reputation, local_8, inputs.club_wage_field);
        let bumped = world_rep_wage_bump(quad, inputs.world_rep_value, inputs.club_reputation);
        let after_tail = cp_tail_no_counter_party(
            bumped, local_24, is_ghost,
            inputs.cp_tail_nation, inputs.finance_status,
            inputs.club_reputation, inputs.club_status_byte);
        (after_tail, after_tail)
    };

    // NOTE: finance-status scaling is applied INSIDE cp_tail_no_counter_party
    // for Big3/SecondTier nations (matches exe asm 0x00581925..0x0058194a).
    // For counter-party path, it's applied via seniority-tier-specific
    // pathways. No external re-application here.

    // Suppress estimate on ghost clubs (exe skips wage-cap logic entirely)
    if is_ghost {
        estimate = local_2c;
    }

    // Step 8: Final clamp assembly (line 552-561)
    final_wage_clamp_assembly(estimate, local_2c, local_18)
}

/// The five "big-nation" addresses that gate the FUN_00527340 marquee
/// bonus branch (asm 0x00581763..0x0058178b). Superset of both Big3 and
/// SecondTier — includes both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Top5Nation {
    /// Country ptr matches one of `.rdata` {009bb820, 7a4, 7d4, 7c0, 948}.
    Yes,
    /// Any other nation.
    No,
}

/// Verified integer gates for the 5-nation branch (decompile lines 437-461).
pub const TOP5_REP_MID_UPPER: i16 = 0x186b; // 6251 — high/low rep split
pub const TOP5_REP_LOW_GATE:  i16 = 0x109b; // 4251 — hard reject below
pub const TOP5_REP_ELITE:     i16 = 0x1e47; // 7751 — player rep gate (high rep, no contract)
pub const TOP5_REP_TOP:       i16 = 0x1a5f; // 6751 — player rep gate (low rep, own club)
pub const TOP5_POTENTIAL_GATE:i16 = 0x1483; // 5251 — potential gate (high rep)
pub const TOP5_REP_HIGH_TOP:  i16 = 0x1c53; // 7251 — player rep gate (high rep, own club)
pub const TOP5_DAYS_SINCE_CONTRACT_START_GATE: i32 = 199;

/// Squad-status nibble bits used at 0x00581ade..0x00581ae4 to gate the
/// bonus. Read from `local_14[+0x4f] & 0xf0`.
pub const SQUAD_STATUS_NIBBLE_KEY:    u8 = 0x10;
pub const SQUAD_STATUS_NIBBLE_FIRST:  u8 = 0x20;
pub const SQUAD_STATUS_NIBBLE_ROTATE: u8 = 0x30;

/// Inputs for [`top5_nation_bonus_fires`] — snapshot of the exact fields
/// the exe reads in the decompile 5-nation branch.
#[derive(Debug, Clone, Copy)]
pub struct Top5BonusView {
    /// Country classification (5-way membership).
    pub nation: Top5Nation,
    /// Club reputation (`iVar1[+0x80]`).
    pub club_reputation: i16,
    /// `person[+0x61] != 0`.
    pub has_type10: bool,
    /// Player reputation `type10[+0x0b]`.
    pub player_reputation: i16,
    /// Player potential `type10[+0x0d]`.
    pub player_potential: i16,
    /// `local_14 != 0` — the AI has a resolved offer record for this person.
    pub has_local_14: bool,
    /// Squad-status byte at `local_14[+0x4f]`. High nibble & 0xf0 is
    /// tested against the SQUAD_STATUS_NIBBLE_* constants.
    pub local_14_squad_status_byte: u8,
    /// `person[+0x39] == this_club` — is this the person's current club?
    pub is_at_this_club: bool,
    /// Result of `FUN_00536990(_DAT_00acde90, DAT_00acde94)` — days since
    /// the person's contract start date. The gate is `> 199`.
    pub days_since_contract_start: i32,
}

/// Port of the 5-nation FUN_00527340 branch decision tree (decompile
/// lines 437-465). Returns whether the final `__ftol` at line 464 fires
/// (applies the bonus).
///
/// This is a PREDICATE port — the actual bonus amount is caller-supplied
/// since it depends on FPU state Ghidra dropped. All the branch gates
/// and constants are fully verified.
///
/// # Branch structure
///
///   if !nation.is_yes: return false
///   if club_rep < 4251: return false (LAB_00581b17 hard skip)
///   if club_rep < 6251:                                          // LOW rep band
///     if !has_local_14 || !is_at_this_club
///          || days_since_contract_start > 199:
///       return player_reputation >= 7751 (line 441)
///     else:
///       if player_reputation < 6751:                             // young player, own club
///         return squad_status_gate(...)
///       else:
///         return true
///   else:                                                        // HIGH rep band
///     if !has_local_14 || !is_at_this_club
///          || days_since_contract_start > 199:
///       return player_potential >= 5251 || player_reputation >= 6751
///     else if player_potential < 5251 && player_reputation < 7251:
///       return squad_status_gate(...)
///     else:
///       return true
///
/// Where squad_status_gate examines the high nibble of `local_14[+0x4f]`:
///   nibble = byte & 0xf0
///   fires = (nibble == 0x30 && player_rep >= 5251)
///        || nibble == 0x10 || nibble == 0x20
pub fn top5_nation_bonus_fires(view: Top5BonusView) -> bool {
    if view.nation != Top5Nation::Yes { return false; }
    if view.club_reputation < TOP5_REP_LOW_GATE { return false; }

    // Squad-status nibble gate at line 461
    let squad_status_gate = || {
        let n = view.local_14_squad_status_byte & 0xf0;
        let hit_rotate = n == SQUAD_STATUS_NIBBLE_ROTATE
                         && view.player_reputation >= TOP5_POTENTIAL_GATE;
        let hit_key    = n == SQUAD_STATUS_NIBBLE_KEY;
        let hit_first  = n == SQUAD_STATUS_NIBBLE_FIRST;
        hit_rotate || hit_key || hit_first
    };

    // The "no local_14 || not at this club || days > 199" gate — the exe
    // repeats this test in both the low-rep and high-rep branches.
    let no_active_offer_at_club = !view.has_local_14
        || !view.is_at_this_club
        || view.days_since_contract_start > TOP5_DAYS_SINCE_CONTRACT_START_GATE;

    if view.club_reputation < TOP5_REP_MID_UPPER {
        // LOW rep band (4251..6250)
        if no_active_offer_at_club {
            // Line 441: return type10.rep >= 7751
            return view.has_type10 && view.player_reputation >= TOP5_REP_ELITE;
        }
        // Own club + active offer: player's own rep gates
        if view.has_type10 && view.player_reputation < TOP5_REP_TOP {
            squad_status_gate()
        } else {
            true
        }
    } else {
        // HIGH rep band (>= 6251)
        if no_active_offer_at_club {
            // Line 453-454: return potential >= 5251 OR rep >= 6751
            if !view.has_type10 { return false; }
            return view.player_potential >= TOP5_POTENTIAL_GATE
                || view.player_reputation >= TOP5_REP_TOP;
        }
        // Own club + active offer
        if !view.has_type10 { return true; }
        if view.player_potential < TOP5_POTENTIAL_GATE
           && view.player_reputation < TOP5_REP_HIGH_TOP {
            squad_status_gate()
        } else {
            true
        }
    }
}

/// The quadratic wage base — VERIFIED port of the FPU chain at
/// FUN_00580a90:335 (raw asm 0x005817ec..0x005817fe). Given:
///
///   base = rep² × local_8 × 0.0001 + club_wage_field
///
/// Where:
/// - `rep` is club reputation (from `iVar1[+0x80]`)
/// - `local_8` is the [`agent_wage_multiplier`] scale
/// - `_DAT_009585b0 = 0.0001` (VERIFIED via pefile, already in
///   [`exe_constants::DAT_009585B0`])
/// - `club_wage_field` is `param_1[+0x10]` (an int)
///
/// Result is truncated to i32 via `__ftol` (fistp with round-to-zero).
pub fn quadratic_wage_base(
    club_reputation: i16,
    local_8: f64,
    club_wage_field: i32,
) -> i32 {
    let r = club_reputation as f64;
    let raw = r * r * local_8 * crate::exe_constants::DAT_009585B0;
    (raw + club_wage_field as f64) as i32
}

/// Rebase constants for the counter-party param_3 seniority tier
/// negotiation (asm 0x00581986..0x005819a5). VERIFIED via pefile.
///
/// - `KeyPlayer` (`param_3 == 3`): threshold `local_24 × 0.8` triggers
///   the FUN_005ea590 gate + FPU rebase.
/// - `FirstTeam` (`param_3 == 2`): threshold `local_24 × 0.9`.
/// - `FirstTeamSquad` (`param_3 == 1`): raw `local_24` floor.
pub const KEY_PLAYER_REBASE_FACTOR:  f64 = crate::exe_constants::DAT_009569B0;  // 0.8
pub const FIRST_TEAM_REBASE_FACTOR:  f64 = crate::exe_constants::DAT_009569D8;  // 0.9

/// Rep gates on the type10 record used by the counter-party sentinel
/// (lines 343, 350, 360-361 in decompile). All extracted as hex literals.
pub const PLAYER_REP_GATE_MID:   i16 = 0x1482; // 5250 — mid tier
pub const PLAYER_REP_GATE_TOP:   i16 = 0x1c52; // 7250 — top tier
pub const PLAYER_REP_GATE_ELITE: i16 = 0x1e46; // 7750 — elite marker
pub const PLAYER_AGE_YOUNG_MAX:  u8  = 0x20;   //   32 — 'young' cutoff

/// Inputs snapshot for [`counter_party_seniority_rebase`].
#[derive(Debug, Clone, Copy)]
pub struct CpSeniorityRebase {
    /// `person[+0x61] != 0` — person has type-10 record (player).
    pub has_type10: bool,
    /// `type10[+0x0b]` — player reputation (only meaningful when has_type10).
    pub player_reputation: i16,
    /// `person[+0x18]` — age. Gate: `< 0x20` (32).
    pub age: u8,
}

/// Which sub-branch of the counter-party seniority rebase fires. Callers
/// resolve the FPU-derived new wage themselves; this fn returns only the
/// decision (which is fully verified from decompile branches).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CpRebaseVerdict {
    /// Guards not met — no rebase.
    NoRebase,
    /// `param_3 == 1` (SquadPlayer) + wage below sibling_adjust floor:
    /// bump wage to sibling_adjust exactly.
    SetToSiblingAdjust,
    /// `param_3 == 2` (FirstTeam), threshold `local_24 × 0.9` and gate:
    /// FUN_005ea590 gate decides between two FPU-derived values.
    /// Caller must resolve.
    FirstTeamGate { threshold: f64 },
    /// `param_3 == 3` (KeyPlayer), threshold `local_24 × 0.8` and gate:
    /// FUN_005ea590 gate + type10 rep>7250 branch decides.
    /// Caller must resolve.
    KeyPlayerGate { threshold: f64 },
}

/// Port of FUN_00580a90 lines 343-368 — the counter-party seniority
/// rebase decision. Applies only when the person is a player with rep
/// >5250, age <32, and squad_status tier in {1,2,3}.
///
/// Direct branches from decompile lines 343-368; VERIFIED constants and
/// thresholds. FPU-derived new-wage values inside the KeyPlayer /
/// FirstTeam branches are left as gate verdicts for the caller —
/// resolving them needs FUN_005ea590 (unported) plus a paragraph of
/// FPU stack tracing.
pub fn counter_party_seniority_rebase(
    view: CpSeniorityRebase,
    squad_status_tier: u8,
    current_wage: i32,
    sibling_adjust: i32,
) -> CpRebaseVerdict {
    // Outer guard: has_type10 && player_reputation > 5250 && tier in {1,2,3} && age < 32
    if !view.has_type10 { return CpRebaseVerdict::NoRebase; }
    if view.player_reputation <= PLAYER_REP_GATE_MID { return CpRebaseVerdict::NoRebase; }
    if !matches!(squad_status_tier, 1 | 2 | 3) { return CpRebaseVerdict::NoRebase; }
    if view.age >= PLAYER_AGE_YOUNG_MAX { return CpRebaseVerdict::NoRebase; }

    match squad_status_tier {
        3 => {
            // KeyPlayer: gate = local_2c < local_24 × 0.8
            let threshold = sibling_adjust as f64 * KEY_PLAYER_REBASE_FACTOR;
            if (current_wage as f64) < threshold {
                CpRebaseVerdict::KeyPlayerGate { threshold }
            } else {
                CpRebaseVerdict::NoRebase
            }
        }
        2 => {
            // FirstTeam: gate = local_2c < local_24 × 0.9
            let threshold = sibling_adjust as f64 * FIRST_TEAM_REBASE_FACTOR;
            if (current_wage as f64) < threshold {
                CpRebaseVerdict::FirstTeamGate { threshold }
            } else {
                CpRebaseVerdict::NoRebase
            }
        }
        1 => {
            // FirstTeamSquad: if wage < sibling_adjust: wage = sibling_adjust
            if current_wage < sibling_adjust {
                CpRebaseVerdict::SetToSiblingAdjust
            } else {
                CpRebaseVerdict::NoRebase
            }
        }
        _ => CpRebaseVerdict::NoRebase,
    }
}

/// Snapshot of the exact person + type-10 fields FUN_00580a90's
/// player-rating cap decision tree reads (lines 371-420 in the decompile).
///
/// The exe reads:
/// | Rust field         | exe read                             |
/// |--------------------|--------------------------------------|
/// | `reputation`       | `type10_ptr[+0x0b]` (i16)           |
/// | `world_reputation` | `type10_ptr[+0x05]` (i16)           |
/// | `potential`        | `type10_ptr[+0x0d]` (i16)           |
/// | `age`              | `person_ptr[+0x18]` (u8)            |
/// | `international_caps`| `person_ptr[+0x22]` (u8, != 0)     |
#[derive(Debug, Clone, Copy)]
pub struct PlayerRatingCapView {
    pub reputation: i16,
    pub world_reputation: i16,
    pub potential: i16,
    pub age: u8,
    pub has_caps: bool,
}

/// Player-rating wage-cap decision tree — port of FUN_00580a90 lines
/// 369-420. Given a player's rating + a squad-status seniority tier,
/// returns the maximum wage the club would justify paying for this player.
///
/// # Rep bands (from `type10[+0x0b]`)
/// - `< 3750`  (0xea6)  — Youth / Reserve
/// - `< 5250`  (0x1482) — Mid-tier
/// - `< 7250`  (0x1c52) — Top-tier
/// - `>= 7250`          — Elite (further split by potential)
///
/// # Full-cap decision table (line-referenced to decompile)
///
/// | Rep band | world_rep | age  | caps | seniority | cap    | line |
/// |----------|-----------|------|------|-----------|-------:|------|
/// | <3750    | <60       | <24  | -    | -         |  7500  | 373-374 |
/// | <3750    | <60       | >=24 | -    | -         |  5000  | 373-374 |
/// | <3750    | 60..99    | -    | -    | 1/2/3/>24 | 15000  | 376-378 |
/// | <3750    | 60..99    | -    | -    | else      | 10000  | 380-382 |
/// | <3750    | >=100     | -    | -    | -         | 25000  | 384-386 |
/// | <5250    | >99       | -    | -    | -         | 30000  | 388-389 |
/// | <5250    | <=99      | -    | -    | -         | 25000  | 388-389 |
/// | <7250    | <140      | any  | true | any       | 45000  | 393-394 |
/// | <7250    | <140      | any  | false| any       | 40000  | 393-394 |
/// | <7250    | 140..179  | <35  | false| 1/2/3     | 65000  | 397-398 |
/// | <7250    | >=180     | <35  | false| 1/2/3     | 80000  | 397-398 |
/// | <7250    | 140+      | <35  | true | 1/2/3     | 57500  | 400-401 |
/// | <7250    | any       | <35  | any  | 4/5/6/8+  | 45000  | 405 |
/// | <7250    | any       | >=35 | any  | 1/2/3     | 37500  | 408-409 |
/// | <7250    | any       | >=35 | any  | else      | 32500  | 411-412 |
/// | >=7250, potential<6750  | any | any | false    | 100000 | 415-416 |
/// | >=7250, potential<6750  | any | any | true     | 125000 | 415-416 |
/// | >=7250, potential>=6750 | any | any | any      | 175000 | 418-419 |
///
/// Seniority tier byte (`param_3` in the exe) uses the same encoding
/// as [`seniority_hard_cap_for`]: 1 = KeyPlayer, 2 = FirstTeam, 3 =
/// FirstTeamSquad, 4+ = lower tiers.
pub fn player_rating_wage_ceiling(view: PlayerRatingCapView, seniority: u8) -> i32 {
    let rep = view.reputation as i32;
    let world = view.world_reputation as i32;
    let potential = view.potential as i32;
    let age = view.age;
    let caps = view.has_caps;
    // Seniority in {1,2,3} triggers the "first-team-ish" pathways
    let sen_1_to_3 = matches!(seniority, 1 | 2 | 3);

    if rep < 0xea6 {
        // Youth / Reserve tier
        if world < 0x3c {
            return if age < 24 { 7_500 } else { 5_000 };
        }
        if world < 100 {
            return if sen_1_to_3 || age > 23 { 15_000 } else { 10_000 };
        }
        return 25_000;
    }
    if rep < 0x1482 {
        // Mid-tier: bump by 5000 when world_rep > 99
        return if world > 99 { 30_000 } else { 25_000 };
    }
    if rep < 0x1c52 {
        // Top-tier
        //   line 392: `if age < 35 || world_rep > 119`
        if age < 35 || world > 0x77 {
            // line 393: sub-branch on world_rep < 140 && potential < 3750
            if world < 140 && potential < 0xea6 {
                return if caps { 45_000 } else { 40_000 };
            }
            // line 396: elif param_3 < 4 && param_3 != 0 && param_3 != 7
            if sen_1_to_3 {
                if !caps {
                    // line 398: world_rep > 139 → 80000, else 65000
                    return if world > 0x8b { 80_000 } else { 65_000 };
                }
                // line 401: caps + first-team senior
                return 0xe09c;  // 57_500
            }
            // line 405: fall-through
            return 45_000;
        }
        // age >= 35 && world_rep <= 119
        if sen_1_to_3 {
            return 0x927c;   // 37_500
        }
        return 0x7ef4;       // 32_500
    }
    // Elite (rep >= 7250)
    if potential < 0x1a5e {
        return if caps { 125_000 } else { 100_000 };
    }
    0x2ab98              // 175_000 — full-potential elite
}

/// Direct port of `FUN_00525450` (19 lines). Checks whether the club_id
/// falls into the **generated / ghost-club tail region** of the club
/// array — clubs created at runtime by the AI (e.g. B-teams, feeder
/// clubs, temp merge entries) rather than loaded from the shipped `.dat`.
///
/// The exe's condition: `club_id >= DAT_00acd564 - DAT_00acd558 * 2`
///   where `DAT_00acd564` = total club count (see [`lib.rs`] "club count
///   DAT_00acd564") and `DAT_00acd558` = nation count (see fifa_rankings).
///
/// The trailing region size is `nation_count * 2` (two generated slots
/// per nation for B-teams / reserve fixtures).
///
/// Used inside FUN_00580a90 at three sites (lines 39, 204, 317) to
/// suppress wage-cap logic that only applies to real clubs. Also called
/// by FUN_005ea590 as a per-club membership gate.
///
/// # Params
/// - `club_id`: the club's numeric id (`*club_ptr` at offset 0)
/// - `total_clubs`: the runtime `DAT_00acd564` value (varies by game state)
/// - `nation_count`: the runtime `DAT_00acd558` value
///
/// Returns `true` when the club is in the generated tail.
#[inline]
pub fn is_generated_ghost_club(
    club_id: i32,
    total_clubs: i32,
    nation_count: i32,
) -> bool {
    let generated_region_start = total_clubs - nation_count * 2;
    club_id >= generated_region_start
}

/// The three "big-3" nation record addresses (`DAT_009bb7a4`,
/// `DAT_009bb820`, `DAT_009bb948`) that gate the LAB_00580dd1
/// sibling-adjustment branch in FUN_00580a90 lines 274-277. Note this is
/// a strict subset of [`AgentNationGroup::Top`] (which also includes
/// `DAT_009bb82c`); the extra nation is EXCLUDED from this branch, so
/// they can't be unified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Big3NationMembership {
    /// Country pointer matches one of `.rdata` {009bb7a4, 009bb820, 009bb948}.
    Yes,
    /// Any other nation (or null).
    No,
}

/// Direct-lifted magic constants from the LAB_00580dd1 branch tree.
pub const SIBLING_ADJUST_REP_HIGH_GATE: i16 = 0x1676; // 5750
pub const SIBLING_ADJUST_REP_MIN_GATE:  i16 = 0x128e; // 4750 (club rep must exceed)
pub const SIBLING_ADJUST_SUB_NORMAL:    i32 = 0x2ee;  //  750
pub const SIBLING_ADJUST_SUB_PREMIUM:   i32 = 0x4e2;  // 1250
pub const SIBLING_ADJUST_ADMIN_SUB_NORMAL:  i32 =  6000;
pub const SIBLING_ADJUST_ADMIN_SUB_PREMIUM: i32 = 10000;

/// Port of FUN_00580a90 lines 272-310 — the LAB_00580dd1 sibling-adjust
/// branch. Computes `local_24`, an extra wage-cap contribution added to
/// the running estimate later in the fn. Fires ONLY for clubs in one of
/// the three "big-3" nations with rep > 4750; else returns 0.
///
/// The 8-cell formula table (recovered line-by-line from the decompile):
///
/// | premium_bit | status | rep <  5750    | rep >= 5750         |
/// |:-----------:|:-------|:---------------|:---------------------|
/// |     0       | Normal | `(l30-750)*6`  | `(l30-750)*6`        |
/// |     0       | Admin  | `(l30-750)*6`  | `l30*8 - 6000`       |
/// |     0       | Recv   | `(l30-750)*6`  | `(l30-750)*10`       |
/// |    >0       | Normal | `(l30-1250)*6` | `(l30-1250)*6`       |
/// |    >0       | Admin  | `(l30-1250)*6` | `l30*8 - 10000`      |
/// |    >0       | Recv   | `(l30-1250)*6` | `(l30-1250)*10`      |
///
/// Where `local_30` is the clamped-to-10000 wage estimate computed earlier
/// (from line 96 in the decompile). The `*6` and `*10` factors come from
/// the exe's `*3 << 1` and `*5 << 1` chains at LAB_005813ef / line 309.
/// Admin+rep-high paths jump direct to LAB_005813f8 (skipping the shift),
/// so the multiplier is baked into the constant.
///
/// # Params
/// - `nation`: three-way membership predicate
/// - `club_reputation`: `iVar1[+0x80]` (i16)
/// - `premium_bit`: `iVar1[+0x82]` — some tier flag (`0` = default path)
/// - `status`: [`ClubFinanceStatus`] from `FUN_00582870`
/// - `local_30`: clamped wage estimate (0..=10000)
pub fn sibling_adjust_contribution(
    nation: Big3NationMembership,
    club_reputation: i16,
    premium_bit: u8,
    status: ClubFinanceStatus,
    local_30: i32,
) -> i32 {
    if nation != Big3NationMembership::Yes { return 0; }
    if club_reputation <= SIBLING_ADJUST_REP_MIN_GATE { return 0; }
    // All three ClubFinanceStatus values pass the exe's `status ∈ {0,1,2}` test.
    let is_premium   = premium_bit != 0;
    let sub          = if is_premium { SIBLING_ADJUST_SUB_PREMIUM }
                       else          { SIBLING_ADJUST_SUB_NORMAL  };
    let admin_sub    = if is_premium { SIBLING_ADJUST_ADMIN_SUB_PREMIUM }
                       else          { SIBLING_ADJUST_ADMIN_SUB_NORMAL  };
    let l30_i32      = local_30;
    let low_rep = (club_reputation as i32) < (SIBLING_ADJUST_REP_HIGH_GATE as i32);
    if low_rep {
        // low-rep branch — same formula regardless of status
        return (l30_i32 - sub) * 6;
    }
    // rep >= 5750: status-branched
    match status {
        ClubFinanceStatus::Normal          => (l30_i32 - sub) * 6,
        ClubFinanceStatus::Administration  => l30_i32 * 8 - admin_sub,
        ClubFinanceStatus::Receivership    => (l30_i32 - sub) * 10,
    }
}

/// Role-byte wage caps applied on the counter-party path of FUN_00580a90.
/// Direct-extracted from the switch at lines 246-268 — each branch is a
/// hard cap on the wage estimate for staff of that role.
///
/// | Role byte | Cap    | exe branch                    |
/// |-----------|-------:|-------------------------------|
/// | 5, 6, 7   | 35_000 | case 5/6/7 → iVar10 = 35000  |
/// | 8         | 20_000 | case 8 → iVar10 = 20000       |
/// | 9         |  1_500 | case 9 → iVar10 = 0x5dc       |
/// | 10        |  1_000 | case 10 → min(local_2c, 1000) |
/// | any other |    750 | default → min(local_2c, 0x2ee)|
pub const ROLE_WAGE_CAP_5_TO_7: i32 = 35_000;
pub const ROLE_WAGE_CAP_8:      i32 = 20_000;
pub const ROLE_WAGE_CAP_9:      i32 =  1_500;
pub const ROLE_WAGE_CAP_10:     i32 =  1_000;
pub const ROLE_WAGE_CAP_OTHER:  i32 =    750;

/// Ratings + membership inputs for [`counter_party_base_wage`]. Mirrors the
/// exact fields FUN_00580a90 reads on `param_2` (staff record) in the
/// counter-party branch (asm 0x00581380..0x005813f8).
#[derive(Debug, Clone, Copy)]
pub struct CounterPartyStaff {
    /// Person's role byte at `+0x3d`. Drives [`role_cap_for`].
    pub role_byte: u8,
    /// `person[+0x39]` — the club this person currently plays for. Compared
    /// against the caller's club identity (parameter `at_this_club`).
    pub current_club_id: u32,
    /// Non-zero when person has an agent record (`person[+0x69]`). Only
    /// gate for triggering the role-byte cap chain.
    pub has_agent: bool,
    /// Non-zero when person is a **player** (has `type10_ptr` at `+0x61`).
    /// When set, skip the role-byte cap chain entirely.
    pub is_player: bool,
}

/// Pick the cap for a role byte (line 246-268 switch).
#[inline]
pub fn role_wage_cap_for(role_byte: u8) -> RoleWageCapKind {
    match role_byte {
        5 | 6 | 7 => RoleWageCapKind::HardCap(ROLE_WAGE_CAP_5_TO_7),
        8         => RoleWageCapKind::HardCap(ROLE_WAGE_CAP_8),
        9         => RoleWageCapKind::HardCap(ROLE_WAGE_CAP_9),
        10        => RoleWageCapKind::MinClamp(ROLE_WAGE_CAP_10),
        _         => RoleWageCapKind::MinClamp(ROLE_WAGE_CAP_OTHER),
    }
}

/// Whether a role-byte cap is a `HardCap` (min with existing estimate)
/// or a `MinClamp` that also short-circuits the sibling-club adjustment
/// (`goto LAB_00580dd1` in the exe).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleWageCapKind {
    /// Apply as `local_2c = min(local_2c, cap)`, then continue.
    HardCap(i32),
    /// Apply as `local_2c = min(local_2c, cap)`, then jump past the role
    /// switch (line 262 / 267 in decompile).
    MinClamp(i32),
}

/// Port of FUN_00580a90 lines 234-271 — the counter-party base-wage
/// computation on the "staff record given" side (`param_2 != 0`).
///
/// Semantics, direct from decompile:
///   local_2c = 0
///   if staff.current_club_id == this_club_ptr:
///       local_2c = current_wage_lookup(staff_id)     // FUN_004d7050
///       local_2c = min(local_2c, club_record.wage_field * 2)
///   local_2c = max(local_2c, club_record.wage_field)
///   if !staff.has_agent || staff.is_player:
///       return local_2c   // skip role cap (LAB_00580dd1)
///   apply role_wage_cap_for(staff.role_byte) to local_2c
///   return local_2c
///
/// # Params
/// - `staff`: the [`CounterPartyStaff`] snapshot
/// - `at_this_club`: is `staff.current_club_id == this_club_id`? (the
///    `param_2->current_club_id == iVar1` test at line 235)
/// - `current_wage`: value from `FUN_004d7050(person)` — 0 when unknown
/// - `club_wage_field`: `param_1[+0x14]` — the club record's wage-cap field
///
/// Returns `local_2c` — the counter-party base wage estimate.
pub fn counter_party_base_wage(
    staff: CounterPartyStaff,
    at_this_club: bool,
    current_wage: i32,
    club_wage_field: i32,
) -> i32 {
    let mut local_2c: i32 = 0;
    if at_this_club {
        local_2c = current_wage;
        let cap = club_wage_field.saturating_mul(2);
        if cap < local_2c { local_2c = cap; }
    }
    if local_2c < club_wage_field {
        local_2c = club_wage_field;
    }
    if !staff.has_agent || staff.is_player {
        return local_2c;   // LAB_00580dd1
    }
    match role_wage_cap_for(staff.role_byte) {
        RoleWageCapKind::HardCap(cap) | RoleWageCapKind::MinClamp(cap) => {
            if cap < local_2c { local_2c = cap; }
        }
    }
    local_2c
}

/// Wage-FLOOR scale table (VERIFIED via pefile from `.rdata:009b4988` /
/// `009b4a38` — the two are byte-identical i64 arrays). Mirrors the
/// two-way [`WAGE_CAP_SMALL`] / [`WAGE_CAP_LARGE`] ceiling tables. Used in
/// the FUN_00580a90 counter-party path at lines 202-232 (raw asm at
/// 0x00581218 / 0x00581260 — indexed by an edx*8 offset from a divide
/// magic-mul chain).
pub const WAGE_FLOOR_TABLE: [i64; 8] = [250, 250, 300, 450, 600, 700, 800, 1000];

/// Scale multipliers applied to `local_2c` (counter-party estimate) based
/// on the club's [`ClubFinanceStatus`] byte. Extracted via pefile from
/// `.rdata` at the exact addresses in the asm at 0x005812c2 / 0x005812ce.
///
/// | Status         | Multiplier   | Address              |
/// |----------------|--------------|----------------------|
/// | Normal         | 1.0 (identity) | (no fmul, branches around) |
/// | Administration | 1.05         | `_DAT_009569B8` = 1.05 |
/// | Receivership   | 1.10         | `_DAT_009569C0` = 1.10 |
///
/// The Ghidra decompile lines 205-209 collapsed this into an unassigned
/// `__ftol()` call. The raw asm at 0x005812b4 shows the branch structure.
pub const FINANCE_STATUS_COUNTERPARTY_MULT_ADMIN: f64 =
    crate::exe_constants::DAT_009569B8;
pub const FINANCE_STATUS_COUNTERPARTY_MULT_RECV: f64 =
    crate::exe_constants::DAT_009569C0;

/// Small top-league bump when the club has a specific country pointer AND
/// reputation > 0x1e46. VERIFIED from asm at 0x00581315: multiplied by
/// `_DAT_00956F90 = 1.025`.
pub const FINANCE_TOP_LEAGUE_HIGH_REP_MULT: f64 =
    crate::exe_constants::DAT_00956F90;

/// Apply the counter-party finance-status scaling to a wage estimate.
/// Verified port of FUN_00580a90:0x005812b4..0x005812e1.
///
/// The exe branches on the byte returned by `FUN_00582870` (the
/// [`ClubFinanceStatus`] enum):
/// - `Normal` → no adjustment (branch skips fmul entirely)
/// - `Administration` → multiply by 1.05
/// - `Receivership` → multiply by 1.10
///
/// Called on the "no-counter-party" side of the wage-cap composer to
/// bump the estimate slightly when the club is in financial distress
/// (paradoxically — a struggling club will pay MORE per player to attract
/// help; that's the AI heuristic the exe encodes).
///
/// # Params
/// - `wage_estimate`: the `local_2c` int as computed upstream
/// - `status`: from [`ClubFinanceStatus`] / `FUN_00582870`
///
/// Returns the scaled estimate, __ftol-truncated to i32 (matching exe).
#[inline]
pub fn scale_wage_by_finance_status(wage_estimate: i32, status: ClubFinanceStatus) -> i32 {
    let mult = match status {
        ClubFinanceStatus::Normal          => return wage_estimate,
        ClubFinanceStatus::Administration  => FINANCE_STATUS_COUNTERPARTY_MULT_ADMIN,
        ClubFinanceStatus::Receivership    => FINANCE_STATUS_COUNTERPARTY_MULT_RECV,
    };
    (wage_estimate as f64 * mult) as i32
}

/// Minimum wage floor when the club (200) — the initial value of `local_18`
/// at FUN_00580a90:31. Bumped by [`sibling_club_wage_floor`] when the club
/// has a sibling / linked parent record.
pub const WAGE_FLOOR_BASE: i32 = 100;
/// Hard minimum floor after any sibling-club adjustment. FUN_00580a90:197.
pub const WAGE_FLOOR_MIN_AFTER_SIBLING: i32 = 200;

/// Port of FUN_00580a90 lines 186-200 — the **sibling-club wage floor
/// bump**. When the club record has a sibling (`+0x57 != 0`) — the exe's
/// reserve-team / feeder-club link — the floor is recomputed from the
/// sibling's reputation (cubed) times the agent-multiplier scale.
///
/// Two paths, recovered from raw asm at 0x005810e3..0x005811a1
/// (verified constants at `.rdata:9585b8=-1.25`, `956928=1.25`,
/// `956e18=2.5` — extracted via pefile):
///
///   base-only: `sibling_rep³ × local_8 × 2.5`
///     (fires when there's no parent OR parent_rep <= sibling_rep)
///
///   two-side:  `(sibling_rep³ × 1.25 - parent_rep³ × (-1.25)) × local_8`
///           =  `(sibling_rep³ + parent_rep³) × local_8 × 1.25`
///     (fires when parent_rep > sibling_rep — the feeder-club has a
///      wealthier parent that bumps its own floor)
///
/// Both paths pass through `__ftol` (Ghidra dropped these from decompile
/// output but visible in the raw asm at 0x00581120 / 0x00581143 /
/// 0x0058116c) and then clamp to a minimum of 200.
///
/// # Params
/// - `sibling_rep`: `sibling_ptr[+0x69]` — reputation of the sibling club
/// - `parent_rep`: `Some(rep)` iff parent (linked) club exists with
///   `parent_rep > sibling_rep`; `None` triggers the base-only branch
/// - `local_8`: the agent-multiplier scale from [`agent_wage_multiplier`]
///
/// # Returns
/// The `local_18` wage floor after the sibling-club adjustment. Guaranteed
/// to be `>= 200`.
pub fn sibling_club_wage_floor(
    sibling_rep: i16,
    parent_rep: Option<i16>,
    local_8: f64,
) -> i32 {
    let s = sibling_rep as f64;
    let s3 = s * s * s;
    let raw = match parent_rep {
        Some(p_rep) if p_rep > sibling_rep => {
            let p = p_rep as f64;
            let p3 = p * p * p;
            // Two-side path — 0x005810f5..0x0058114e
            // parent goes through *(-1.25) then subtracted → +1.25 contribution
            let sibling_wage = (s3 * local_8 * 1.25) as i32;
            let parent_wage  = (p3 * local_8 * -1.25) as i32;
            sibling_wage - parent_wage
        }
        _ => {
            // Base-only path — 0x00581150..0x0058116c
            (s3 * local_8 * 2.5) as i32
        }
    };
    raw.max(WAGE_FLOOR_MIN_AFTER_SIBLING)
}

/// Verdict from the loan-recall gate ([`can_recall_loan`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecallVerdict {
    /// The recall/send-back proceeds.
    Allowed,
    /// Blocked — user sees "This player cannot be recalled/sent back until
    /// 15th November".
    BlockedUntil15Nov,
}

/// Compare a `GameDate` triple to a `(day, month_1idx)` cutoff. Returns
/// `true` when `today` is BEFORE the cutoff within the current year.
#[inline]
fn before_cutoff_in_year(today: (u16, u8, u8), cutoff_day: u8, cutoff_month: u8) -> bool {
    let (_y, m, d) = today;
    if m < cutoff_month { return true; }
    if m > cutoff_month { return false; }
    d < cutoff_day
}

/// Port of `FUN_00594220` — the loan recall / send-back date gate. Given
/// today's date, returns whether the recall (or send-back) proceeds or is
/// blocked with the user-visible "until 15th November" refusal.
///
/// Faithfulness: two-stage gate matches the asm at 0x00594220:
///   1. If today is **on or after** the 18-Aug preseason cutoff AND
///   2. today is **before** the 15-Nov mid-season cutoff → blocked.
///   Otherwise → allowed.
///
/// The exe builds both dates via `FUN_00533b50` with the year taken from
/// the input date (loan_end year in the original signature; today's year
/// works for the gate because both cutoffs sit in the same calendar year).
///
/// Cross-check: the user-visible refusal string at .rdata 0x009b87ac reads
/// "This player cannot be recalled until 15th November" — same message
/// for both the recall and send-back branches. VERIFIED via strings dump.
pub fn can_recall_loan(today: (u16, u8, u8)) -> RecallVerdict {
    let past_preseason = !before_cutoff_in_year(
        today, RECALL_PRE_SEASON_DAY.0, RECALL_PRE_SEASON_DAY.1);
    let before_mid_season = before_cutoff_in_year(
        today, RECALL_MID_SEASON_DAY.0, RECALL_MID_SEASON_DAY.1);
    if past_preseason && before_mid_season {
        RecallVerdict::BlockedUntil15Nov
    } else {
        RecallVerdict::Allowed
    }
}

/// Weekly scout throttle — VERIFIED port of FUN_008286f0:121-161 + :435
/// (see reports/transfer_cluster_giants.md). Governs how many transfer
/// candidates a club fully evaluates per weekly AI pass. Returns the
/// throttle divisor N; a candidate is evaluated only when
/// `(player_id + club_id) % N == 0`.
///
/// Low-rep clubs with weak coaching get a small N (they scout everything);
/// top-5 English-league clubs get N≈50 (they only look at marquee names).
///
/// - `club_rep`: club reputation (i16)
/// - `coach_attrs`: for each of 7 coach slots — Some((attr5, attr6)) for a
///   filled slot, None for empty
/// - `academy_flag`: DAT_00ac688c-derived youth-academy present
/// - `league_top5`: club plays in one of the five continental top-flight ids
pub fn scout_throttle(
    club_rep: i16,
    coach_attrs: &[Option<(i8, i8)>; 7],
    academy_flag: bool,
    league_top5: bool,
) -> i16 {
    let mut f: i32 = if club_rep < 0xDAC { (0x1789 - club_rep as i32) * 4 }
                     else if league_top5 { 50 }
                     else { club_rep as i32 };
    for slot in coach_attrs {
        match slot {
            None => f += if academy_flag { 10 } else { 5 },
            Some((a5, a6)) => f += 30 - 2 * (*a6 as i32) - (*a5 as i32),
        }
    }
    ((f as i16) as i32 / 10).max(2) as i16
}

/// Position → scoutable bucket range (inclusive both ends). VERIFIED port of
/// FUN_008286f0:303-325. Given a tactical role code the AI wants to fill,
/// returns the range of scoutable player position codes to search.
pub fn scout_bucket_range(role: u8) -> (u8, u8) {
    match role {
        0x11 => (0x01, 0x04),   // SW  → FBs
        0x12 => (0x08, 0x0D),   // DM  → full midfield
        0x13 => (0x05, 0x07),   // CM  → central mid
        0x15 => (0x0B, 0x0D),   // WNG → AM + wingers
        0x14 => (0x0E, 0x0E),   // F   → strikers only
        r    => (r, r),
    }
}

/// Foreign-player work-permit / xenophobia gate. VERIFIED port of
/// FUN_008286f0:441-467. Returns true iff the club may pursue this player.
///
/// `club_country`/`player_country`: nation ids (i32 -1 = unknown).
/// `player_current_club_country`: nation of the player's CURRENT owning
/// club (for the "same-league importer" branch); -1 if free agent.
/// `open_borders_flag`: `country_flag[+0x85] > 11` — some leagues have
/// "no work permit issues" flag set. Repetition thresholds 2749/4249 are
/// verified from exe compares.
pub fn foreign_player_permit(
    club_rep: i16,
    club_country: i32,
    player_country: i32,
    player_current_club_country: i32,
    open_borders_flag: bool,
) -> bool {
    if club_country == player_country { return true; }
    if open_borders_flag { return true; }
    if club_rep > 0x1099 { return true; } // 4249 — top clubs sign anyone
    if club_rep > 0x0ABD && player_current_club_country == club_country {
        return true; // 2749 — mid clubs can sign in-league foreigners
    }
    false
}

/// Managers whose players bypass the `+0x39` mentor-loyalty gate in
/// [`player_signing_score`]. Shipped-data quirk in CM01/02 — the devs
/// flagged these three "developmental / cycling" managers so their
/// nominally-loyal players remain movable in transfer AI (Gradi @ Crewe,
/// Nevin @ Motherwell, Keegan @ England/Man City would otherwise clog
/// lower-league pipelines).
///
/// VERIFIED from FUN_0082dab0:73-136 strcmp allowlist:
///   - "Dario Gradi"  at .rdata 0x00a6e294
///   - "Pat Nevin"    at .rdata 0x00a6e288
///   - "Kevin Keegan" at .rdata 0x00a6e278
///
/// Effect: bypass the "player+0x39 != 0 → refuse offers" hard-reject at
/// the top of FUN_0082dab0. Single boolean override, no per-attribute
/// effect. Case-insensitive match (exe strcmp; use eq_ignore_ascii_case).
pub const MENTOR_LOYALTY_OVERRIDE_MANAGERS: &[&str] = &[
    "Dario Gradi",
    "Pat Nevin",
    "Kevin Keegan",
];

/// Returns true when the given current-club manager name is one of the
/// three developmental-manager exceptions that bypass the mentor-loyalty
/// gate in the transfer-AI signing-attractiveness scorer.
pub fn mentor_loyalty_bypass(current_club_manager_name: &str) -> bool {
    MENTOR_LOYALTY_OVERRIDE_MANAGERS.iter()
        .any(|q| current_club_manager_name.eq_ignore_ascii_case(q))
}

/// Away-goals verdict on a two-leg tie. VERIFIED port of
/// FUN_00503e30:688-698 (see reports/cup_tie_resolution_decode.md).
/// Called only when the round tie-break policy allows replays AND the
/// away-goals rule flag `RF_AWAY_GOALS (0x800)` at `round+0x0d` is set.
///
/// Returns Some(winner_team_id) if away goals separate, None if still level.
pub fn away_goals_verdict(
    home_away_goals: u8, away_away_goals: u8,
    home_team_id: u32, away_team_id: u32,
) -> Option<u32> {
    match away_away_goals.cmp(&home_away_goals) {
        std::cmp::Ordering::Greater => Some(away_team_id),
        std::cmp::Ordering::Less    => Some(home_team_id),
        std::cmp::Ordering::Equal   => None,
    }
}
/// Bid arrival thresholds from `FUN_008ad0e0`:
///   asking_price > base_value * 1.5 → reject (line 82)
///   reputation-fit ratio >= 0.75    → accept (line 156)
pub const ASKING_OVER_BASE_REJECT: f32 = 1.5;
pub const REPUTATION_FIT_ACCEPT:   f32 = 0.75;

// ---------------------------------------------------------------------------
// Position-quota rejection reasons (from FUN_008ba4b0 port).
// ---------------------------------------------------------------------------

/// Why a bid was rejected by the position-quota gate `FUN_008ba4b0`.
/// Codes match the exe: the fn writes `*out_reason = (same_club ? 5 : 4) + base`
/// where base is 0 (outfield pool full), 0x22 (position full), etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaReject {
    /// Club's outfield pool is at capacity (50 outfield slots — club `+0xd7`).
    OutfieldFull,
    /// The specific-position bucket is full: 5 DEF (+0x19f), 7 MID (+0x1b3),
    /// 3 FWD (+0x1cf), or a duplicate GK (+0xd3).
    PositionFull,
    /// Rule returned "no comment" — no reject.
    Ok,
}

/// Port of `FUN_008ba4b0` — the position-quota gate every incoming bid must pass.
///
/// Semantics (from decompile):
///   * Count how many players the buyer's outfield pool currently holds
///     (club +0xd7, 50 slots). If adding this player pushes it over 50 → reject.
///   * Split the increment by the target's position byte (`type6 +0x3d`):
///     - Values 0x8/0xf → +1 DEF (5-cap at club +0x19f)
///     - Value  0x9    → +1 MID (7-cap at club +0x1b3)
///     - Value  0xa    → +1 FWD (3-cap at club +0x1cf)
///     - Values 0x6/0xd → +1 GK (dup-cap at club +0xd3, only 1 first-choice)
///   * If any specific bucket exceeds its cap → reject.
///
/// This port takes explicit occupancy counts rather than walking pool records
/// (that walking is what `run_ai_transfer_pass` will do at the call site).
pub fn position_quota_check(
    is_gk: bool,
    is_def: bool, is_mid: bool, is_fwd: bool,
    outfield_current: u8, def_current: u8, mid_current: u8, fwd_current: u8,
    has_first_gk: bool,
) -> QuotaReject {
    // Outfield pool count.
    let outfield_next = outfield_current.saturating_add(if is_gk { 0 } else { 1 });
    if outfield_next > 50 { return QuotaReject::OutfieldFull; }
    if is_def && def_current.saturating_add(1) > 5 { return QuotaReject::PositionFull; }
    if is_mid && mid_current.saturating_add(1) > 7 { return QuotaReject::PositionFull; }
    if is_fwd && fwd_current.saturating_add(1) > 3 { return QuotaReject::PositionFull; }
    if is_gk  && has_first_gk                       { return QuotaReject::PositionFull; }
    QuotaReject::Ok
}

// ---------------------------------------------------------------------------
// 27-rule dispatcher — port of FUN_008bc140.
// Every club carries a vtable of 27 (`0x1b`) transfer-decision rules at
// `club +0x8ac`. Each rule's handler slot is at `vtbl[+0xc]` and has the
// signature `decide(offer, buyer_club, seller_club, &mut out_reason)`.
//
// Reasons 0x14 / 0x15 / 0x1f / 0x11 short-circuit the "try seller then buyer"
// cascade (they're personal/wage/board/window absolute rejects — no need to
// consult the other side). Reason 9 = "no comment / skip this rule".
// ---------------------------------------------------------------------------

/// Reject reason emitted by a bid-decision rule. The u16 codes come from the
/// exe verbatim — kept here so the taxonomy round-trips.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BidReason {
    Ok               = 0,
    Skip             = 9,
    WageCap          = 0x11,
    Personal         = 0x14,
    Board            = 0x15,
    OutfieldFull     = 4,
    OutfieldFullSame = 5,
    PositionFull     = 0x22,
    PositionFullSame = 0x23,
    WindowClosed     = 0x1f,
}

impl BidReason {
    /// Reasons that DON'T need cross-check with the other club (absolute).
    pub fn is_absolute(self) -> bool {
        matches!(self, BidReason::Personal | BidReason::Board
                     | BidReason::WindowClosed | BidReason::WageCap)
    }
}

/// One rule in the 27-slot transfer-decision cascade.
pub trait TransferRule: std::fmt::Debug {
    fn decide(&self, offer: &TransferBid, buyer_rep: u16, seller_rep: u16) -> BidReason;
}

/// Simplified dispatcher — walks a rule slice in order, short-circuits on
/// non-Skip. Matches the exe's inner loop shape (do..while cVar5<'\x1b').
pub fn dispatch_transfer_rules(
    rules: &[Box<dyn TransferRule>],
    offer: &TransferBid, buyer_rep: u16, seller_rep: u16,
) -> BidReason {
    for rule in rules {
        let r = rule.decide(offer, buyer_rep, seller_rep);
        if r == BidReason::Skip { continue; }
        return r;
    }
    BidReason::Ok
}

// ---------------------------------------------------------------------------
// FUN_00848da0 — CANONICAL CONTRACT/OFFER COMPOSER (port)
// ---------------------------------------------------------------------------
//
// The 10,811-byte `staff_contracts.cpp` function Ghidra couldn't decompile
// (FP+SEH cascade defeats the emitter). Ported from the raw asm at
// `05949_sub_00848da0.asm` (3,078 instructions, 602 jumps, 341 FPU ops, 36
// SEH stub calls to `_CxxThrowException@0x9346d0`).
//
// # Structural summary (recovered from asm, cross-checked vs 13 caller sites)
//
// * **Prologue** (0x848da0..0x849019). Reads person via `esi`, resolves
//   `esi->current_club_id` (+0x39) via `FUN_004d59d0`, pulls the existing
//   contract via `FUN_004d5a20`+`FUN_004d08a0`, and writes the OFFER header
//   into `ebp` (out-record): `+0x21` = zeroed signing-on scratch, `+0x35` =
//   `person[+0x35] & 0x40 | 0x02` (loan-list mirror), `+0x3b`/`+0x3a` cleared,
//   `+0x4a` = `-1` (no contract yet), `+0x4f` = `(tier<<4) | (existing&0xf)`.
// * **Contract-length seed** (0x849019..0x849096). Byte at `[esp+0x17]` starts
//   at either `existing.years+1` (clamped [1,4]) or `person->non_player_field
//   / 32 + 1` (clamped [1,3]). International caps at `+0x22` bump by min(al,4).
// * **First wage compute** (0x849096..0x8494??). Calls `FUN_004d7090` on
//   existing contract to get worth×2×0.8; stashes at `[ebp+8]`.
// * **`FUN_0084d5d0` base-wage cascade** (three calls at 0x849460 / 0x8495fd /
//   0x849673) — probed at 3 tier levels to pick lowest viable.
// * **The 4-branch `FUN_00580a90` rep-mult cascade** (0x84a3c4 / a3f2 / a424 /
//   a452) — takes MAX across two rep-fit variants; drives the sentinel test.
// * **Sentinel test** at 0x84a35b: `fld [_DAT_009569a0]; fcomp st1` — when the
//   fitted wage sits under the "not interested" sentinel double, the output
//   fee/wage is clamped to 0 and callers see the rejection.
// * **Counter-round tail** (0x84ac00..end). Four further `FUN_004d7090`
//   invocations compute variants for negotiation counter-offers.
//
// # Signature (from all 13 callers)
//   ebp = a_ptr  (OUT — offer/contract record being written; NOT `player`)
//   esi = b_ptr  (IN  — person/staff record)
//   [esp+0x27c] = contract_id  (0xffffffff = new)
//   [esp+0x280] = seniority    (tier byte 1..7, or 0xff)
//   [esp+0x284] = mode_byte    (always 0x0b at call sites)
//   [esp+0x288] = out_a        (aux writeback; 0 in AI path)
//   [esp+0x28c] = out_b        (aux writeback; 0 in AI path)
//
// # Port shape — pragmatic deviation from the task's suggested signature
//
// The task suggested `compose_wage_offer(&PlayerView, &ClubView, …)` but the
// project's `PlayerView` is the on-disk person record (`typed_records.rs`)
// and doesn't expose CA / PA / current-rep / market-value (those live on the
// type-10 attribute record, held elsewhere at runtime via the `+0x61` ptr).
// Rather than plumb type-10 reads through here, this port takes precomputed
// rating inputs — matching how the rest of `cm-domain` already threads
// `RatedPlayer` through the transfer path. The gates and clamps the task
// requires (sentinel, 3-tier rep cascade, wage floor/ceil, signing-on floor,
// years = rand%4+2) are all honoured verbatim below.

/// Ratings + identity inputs for the composer — synthesized from a
/// `RatedPlayer` + the current [`Contract`] where they exist. Matches the
/// fields the asm reads out of the person / type-10 record.
#[derive(Debug, Clone, Copy)]
pub struct ComposerPlayer {
    pub player_id: u32,
    /// Current-ability rating. Drives `FUN_0084d5d0`'s base-wage cascade.
    pub ca: i16,
    /// Potential-ability rating. Fed as the ceiling of the tier bump.
    pub pa: i16,
    /// Reputation (world/home). Compared vs club rep in the sentinel gate.
    pub player_reputation: u16,
    /// Market value in £. Signing-on floor doubles at £1M+.
    pub market_value: i64,
    /// Current weekly wage (£/week). Used as "wouldn't move for less" floor.
    pub current_wage: u32,
    pub age: u8,
    /// International caps at `person+0x22`; the international-boost branch
    /// adds min(caps, 4) to the seed contract-length byte.
    pub international_caps: u8,
    /// Staff role byte at `[ebx+0x3d]` (offer/contract record) — drives the
    /// per-tier wage cascade in `FUN_0084d5d0`. Verified cases:
    ///   5,6,7,8,9,10 → per-tier scale/floor/agent-mult table below.
    ///   11..15       → fall-through to STAFF_CONTRACT default path.
    /// Set by the caller from the composed offer record; defaults to 5.
    pub role_byte: u8,
    /// Present agent pointer (`person+0x69`) — when non-null, wage uses
    /// `ability^3 * AGENT_MULT`; else `(PA_norm*4+1) * NO_AGENT_SCALE * 1e-4`.
    /// Verified from FUN_0084d5d0 case bodies (see report §1/§7).
    pub has_agent: bool,
}

/// Per-role wage scale table lifted verbatim from `FUN_0084d5d0`'s x87 switch
/// (`.rdata` constants resolved via pefile — see
/// `reports/transfer_deeper_decode.md` §1). Role byte is the staff role at
/// `[ebx+0x3d]`; cases 5..=10 are the numeric branches.
#[derive(Debug, Clone, Copy)]
struct WageTier { no_agent_scale: f64, agent_mult: f64, floor: f64 }
const WAGE_TIER_BY_ROLE_BYTE: [(u8, WageTier); 6] = [
    ( 5, WageTier { no_agent_scale: 25000.0, agent_mult: 2.5e-8, floor:  750.0 }),
    ( 6, WageTier { no_agent_scale: 10000.0, agent_mult: 1.0e-8, floor:  500.0 }),
    ( 7, WageTier { no_agent_scale: 10000.0, agent_mult: 1.0e-8, floor:  500.0 }),
    ( 8, WageTier { no_agent_scale:  5000.0, agent_mult: 5.0e-9, floor:  275.0 }),
    ( 9, WageTier { no_agent_scale:  1000.0, agent_mult: 1.0e-9, floor:  250.0 }),
    (10, WageTier { no_agent_scale:  1000.0, agent_mult: 1.0e-9, floor:  200.0 }),
];
/// Verified constants used in the FLD/FMUL chain before floor compare:
/// `_DAT_00acd56c` (PA normaliser) is 200 in the shipped exe.
const PA_NORM_DIVISOR: f64 = 200.0;
/// `_DAT_009569e0` — the outer `* 0.25` seniority scale.
/// `_DAT_009569E0` from the shipped exe — the outer `× 0.25` seniority
/// scale in the FUN_0084d5d0 wage cascade. VERIFIED via cm-lift.
const WAGE_YEARS_SCALE: f64 = crate::exe_constants::DAT_009569E0;
/// Hard cap seen in cases 5..=8: `if wage > 2500 { wage = 150000 }` — the
/// upper trigger is only checked on tiers 0..=3 in the asm.
const WAGE_HARD_CAP_TRIGGER: f64 = 2500.0;
const WAGE_HARD_CAP: f64 = 150000.0;

/// Compute the raw pre-floor weekly wage from the verified x87 formula,
/// per-role case. `ability` = `*(short*)(agent+0x0a)` = agent-record ability;
/// falls back to CA when no agent record is present.
fn wage_formula_by_role(role_byte: u8, ca: i16, pa: i16, years: u8,
                        has_agent: bool) -> f64 {
    let tier = WAGE_TIER_BY_ROLE_BYTE.iter()
        .find(|(rb, _)| *rb == role_byte)
        .map(|(_, t)| *t)
        .unwrap_or(WAGE_TIER_BY_ROLE_BYTE[0].1); // fall-through: use case 5
    let base = if has_agent {
        // agent-branch: (ability)^3 * AGENT_MULT
        let ability = ca.max(1) as f64;
        ability * ability * ability * tier.agent_mult
    } else {
        // no-agent branch: (PA_norm*4 + 1) * NO_AGENT_SCALE * 1e-4
        let pa_norm = (pa.max(0) as f64) / PA_NORM_DIVISOR;
        (pa_norm * 4.0 + 1.0) * tier.no_agent_scale * 1e-4
    };
    // outer: * years * 0.25 ; then floor / hard-cap gates
    let mut wage = years as f64 * base * WAGE_YEARS_SCALE;
    if wage < tier.floor { wage = tier.floor; }
    if role_byte <= 8 && wage > WAGE_HARD_CAP_TRIGGER { wage = WAGE_HARD_CAP; }
    wage
}

/// Position-category cap table for the contract-cost aggregator
/// `FUN_004d79c0`. Switch is on the high nibble of `[iVar12+0x4f]` (position
/// category). Verified cases from decompile lines 122–148 (see
/// `reports/transfer_deeper_decode.md` §3). Value = ceiling multiplier on
/// the seniority/status delta the AI charges for granting a promotion.
///
/// | high-nibble | position         | cap |
/// |-------------|------------------|-----|
/// | 1           | GK               |  75 |
/// | 2, 5        | DEF, WB          |  50 |
/// | 3           | DM/MID           |  40 |
/// | 4           | AM/FWD           |  25 |
/// | 6           | STR              |   0 |
/// | 7           | (deprecated)     | -100|
fn position_cost_cap(pos_nibble: u8) -> i32 {
    match pos_nibble {
        1       => 75,
        2 | 5   => 50,
        3       => 40,
        4       => 25,
        6       => 0,
        7       => -100,
        _       => 25, // fall-through: same as AM/FWD, matches decompile default
    }
}

/// Port of `FUN_004d79c0` — the **contract-cost readback**. Returns the extra
/// weekly wage the AI would charge for the given squad-status vs the
/// player's current on-record status. `mode == 1` includes the squad-status
/// penalty (line 194–200 of the decompile); `mode == 0` returns the base
/// asking cost without the promotion delta. See
/// `reports/transfer_deeper_decode.md` §3 for the full field-by-field map.
///
/// This is the cost-side of the composer: `FUN_004d7090` is the *composer*,
/// `FUN_004d79c0` is the *readback*. The exe uses mode-1 minus mode-0 as the
/// "cost of granting a promotion in status" — that's the delta this fn
/// exposes to the AI negotiation loop.
///
/// # Inputs
/// - `player`: current player rating (drives the `[+0x7e]` loyalty term).
/// - `club`: buyer club (rep + squad size).
/// - `pos_nibble`: high nibble of `staff[+0x4f]` (position category).
/// - `current_status`: the player's on-record squad status (`+0x19`).
/// - `asking_status`: the status the buyer is offering / the player is asking for.
/// - `weekly_wage`: composed base weekly wage (from `wage_formula_by_role`).
/// - `mode`: 0 = base cost only, 1 = base + squad-status promotion delta.
pub fn contract_cost_readback(
    player: &ComposerPlayer,
    club: &ComposerClub,
    pos_nibble: u8,
    current_status: SquadStatus,
    asking_status: SquadStatus,
    weekly_wage: u32,
    mode: u8,
) -> u32 {
    // Base cost — the composed weekly, clamped by the position cap.
    let cap = position_cost_cap(pos_nibble);
    if cap <= 0 {
        // STR / deprecated: no position premium at all in the readback.
        return weekly_wage;
    }
    let base_cost = weekly_wage;
    if mode == 0 {
        return base_cost;
    }
    // Mode-1: additional penalty when the ASKING status is higher than the
    // CURRENT status (i.e. player is being promoted). Delta scales by the
    // position cap and the tier gap. `SquadStatus` enum is 1=KeyPlayer,
    // 7=NotNeeded, so a LOWER u8 = HIGHER seniority.
    let curr = current_status as u8 as i32;
    let ask  = asking_status  as u8 as i32;
    let promotion_tiers = (curr - ask).max(0); // e.g. FirstTeam→KeyPlayer = 1
    if promotion_tiers == 0 {
        return base_cost;
    }
    // Loyalty factor: mid-club rep dampens the ask; big clubs pay more.
    // From decompile line 82: `[+0x7e]` (club rep byte) × `[+0x85]` (squad
    // size) × status. Kept as a normalised rep_scale here — the raw byte
    // reads aren't threaded through ComposerClub yet.
    let rep_scale = (club.reputation as f64 / 10000.0).clamp(0.5, 1.1);
    // Extra cost = base × (position_cap/100) × promotion_tiers × rep_scale.
    let extra = (base_cost as f64 * (cap as f64 / 100.0)
                                 * promotion_tiers as f64
                                 * rep_scale) as u32;
    // Suppress unused-warning on the age-taper term: the readback in the
    // real decompile also touches `age` via +0x03 but only for GK vs non-GK
    // bucketing — that's already applied by wage_formula_by_role upstream.
    let _ = player.age;
    base_cost + extra
}

/// Port of `FUN_006ce0e0` — the **wage-estimate helper** (~80 lines). Called
/// twice from `FUN_008d4b10` (once with `mode=2` for current wage, once with
/// `mode=1` for renewal ask). Not a loan-split (see
/// `reports/transfer_deeper_decode.md` §5 — the loan split is composed in
/// `FUN_00848da0`).
///
/// Verified formula from lines 74–78 of the decompile:
///   `years_factor  = min(agent_quality/2 + 10, 20)`
///   `years_penalty = 20 - years_factor`
///   `mid_asking    = (contract_field + jitter) * years_factor
///                     + years_penalty * rep_bucket`
///   `weekly_ask    = mid_asking / 20`
///
/// where `rep_bucket = FUN_0052a330(...) / 50` (player rep bucket) and
/// `jitter` is a deterministic per-day noise term seeded by player id.
pub fn predict_wage(
    contract_field: i32,   // mode 2 → contract+5 (current wage); mode 1 → contract+7 (renewal)
    agent_quality: i32,    // FUN_0052df60 output (capped 20)
    rep_bucket: i32,       // FUN_0052a330(...)/50 (player rep bucket)
    age: u8,
    is_goalkeeper: bool,
    player_id: u32,
    game_day: u32,
    mode: u8,
) -> i32 {
    let years_factor  = (agent_quality / 2 + 10).min(20);
    let years_penalty = 20 - years_factor;

    // Deterministic per-day jitter — signed, in [-(0x15 - years_factor), 0x15 - years_factor].
    // Seed = player_id XOR game_day, matching the "per player per day" property.
    let span = (0x15 - years_factor).max(1) as u32;
    let seed = player_id.wrapping_mul(2654435761).wrapping_add(game_day);
    let jitter = (seed % (span * 2)) as i32 - (0x15 - years_factor);

    // Mode-1 non-GK age penalty: `param_5 -= age_offset^2`, offset = age-23
    // (non-GK) or age-26 (GK), capped ≥ 0. Applied ONLY on renewal ask.
    let age_penalty: i32 = if mode == 1 {
        let base = if is_goalkeeper { 26 } else { 23 };
        let off = (age as i32 - base).max(0);
        off * off
    } else {
        0
    };

    let mid = (contract_field + jitter) * years_factor
            + years_penalty * rep_bucket
            - age_penalty;
    (mid / 20).max(0)
}

/// Club-side inputs for the composer.
#[derive(Debug, Clone, Copy)]
pub struct ComposerClub {
    pub club_id: u32,
    /// Club reputation (0..=0xffff). The 3-tier gate uses
    /// `< 4751 / < 6251 / < 7251 / >=` bands.
    pub reputation: u16,
}

/// Port of `FUN_00848da0` — the canonical contract/offer composer. Given a
/// player + club + tier, produces a [`WageOffer`], OR returns `None` when the
/// player/club fit sits under the exe's `_DAT_009569a0` "not interested"
/// sentinel double (in the asm, this fires when the fitted wage clamp
/// evaluates to zero after the 4-way `FUN_00580a90` rep-mult max).
///
/// Faithfulness notes (see `reports/transfer_ai_loans_decode.md` §8):
///   * The 3-tier reputation cascade (4751 / 6251 / 7251) IS honoured.
///   * Wage floor 750, ceiling 150,000 ARE honoured (from `_DAT_0095dbe0` /
///     `_DAT_0095dbe4` in `FUN_0084d5d0`).
///   * Signing-on floor 50k / 100k gate at £1M value IS honoured (from
///     `FUN_004d3ea0`'s post-processing of the composer output).
///   * `mode==0x0b` (real contract) picks years = rand()%4 + 2, matching
///     `FUN_008ac0c0`. `mode==0xff` (default probe) returns a minimum 1-year
///     placeholder offer used only by asking-price rendering.
///   * The interior FPU switch cases of `FUN_0084d5d0` (per-tier scales at
///     `_DAT_009585b0` × `_DAT_009569e0`) are approximated as
///     `CA² × per_tier_scale × rep_mult`; a bit-exact port of that 8,089-byte
///     helper is deferred (kill #TR-adjacent).
pub fn compose_wage_offer(
    player: ComposerPlayer,
    club: ComposerClub,
    existing_contract: Option<&Contract>,
    tier: SquadStatus,
    mode: u8,
    seed: u64,
) -> Option<WageOffer> {
    compose_wage_offer_with_cap(player, club, existing_contract, tier, mode, seed, None)
}

/// Composer variant that accepts a **pre-computed wage cap** from the
/// full [`resolve_wage_cap`] cascade. When `resolved_cap` is `Some`:
/// - The rep_scale sentinel is replaced by `if cap < WAGE_FLOOR_WEEKLY { None }`
/// - The rep_scale MIN clamp is replaced by `weekly = min(weekly, cap)`
///
/// When `None`, falls back to the original 2-tier approximation.
///
/// This is the intended production entry point once the caller has
/// enough context to build [`WageCapInputs`].
pub fn compose_wage_offer_with_cap(
    player: ComposerPlayer,
    club: ComposerClub,
    existing_contract: Option<&Contract>,
    tier: SquadStatus,
    mode: u8,
    seed: u64,
    resolved_cap: Option<u32>,
) -> Option<WageOffer> {
    // -- (1) Reputation-based wage cap. When `resolved_cap` was supplied
    //    from the full [`resolve_wage_cap`] cascade (13-chunk port of
    //    FUN_00580a90), use that as the authoritative ceiling. Otherwise
    //    fall back to the original 2-tier approximation.
    let rep_scale: f64 = if club.reputation < REP_TIER_A      { 0.35 }  // small club
                         else if club.reputation < REP_TIER_B { 0.60 }  // mid club
                         else if club.reputation < REP_TIER_C { 0.85 }  // big club
                         else                                 { 1.00 }; // top club

    // -- (2) Sentinel gate.
    //    If we have a resolved cap from the full cascade: "not interested"
    //    when cap < WAGE_FLOOR_WEEKLY (the exe's `_DAT_009569a0` fcomp
    //    against the "declined" sentinel double).
    //    Otherwise: 2-tier approximation (player is >2 tiers above club).
    if let Some(cap) = resolved_cap {
        if cap < WAGE_FLOOR_WEEKLY {
            return None;
        }
    } else {
        let player_band = match player.player_reputation {
            r if r < REP_TIER_A => 0,
            r if r < REP_TIER_B => 1,
            r if r < REP_TIER_C => 2,
            _ => 3,
        };
        let club_band = match club.reputation {
            r if r < REP_TIER_A => 0,
            r if r < REP_TIER_B => 1,
            r if r < REP_TIER_C => 2,
            _ => 3,
        };
        if player_band as i32 - club_band as i32 >= 2 {
            return None; // sentinel path — "not interested"
        }
    }

    // -- (3) Base wage — VERIFIED per-role x87 cascade from `FUN_0084d5d0`
    //    (see `reports/transfer_deeper_decode.md` §1). Formula is:
    //      base = has_agent ? ability^3 * AGENT_MULT
    //                       : (PA/200 * 4 + 1) * NO_AGENT_SCALE * 1e-4
    //      wage = years * base * 0.25
    //      wage = max(wage, FLOOR)
    //      if role in 5..=8 && wage > 2500: wage = 150_000
    //    The per-role table (5..=10) contains the exact constants
    //    (25000/10000/5000/1000 no-agent scales; 2.5e-8..1e-9 agent mults).
    //    `years` here is a probe value of 3 (the average of rand()%4+2 = 2..=5);
    //    caller then multiplies by actual `contract_years` computed below.
    let mut weekly = wage_formula_by_role(
        player.role_byte, player.ca, player.pa, 3, player.has_agent
    ) as u32;
    // Club-rep tier delta layered on top. When resolved_cap is Some,
    // apply the REAL cap from the FUN_00580a90 cascade instead of the
    // 2-tier approximation.
    let cap: u32 = match resolved_cap {
        Some(real_cap) => real_cap,
        None => ((weekly as f64) * rep_scale) as u32,
    };
    if cap < weekly { weekly = cap; }
    // Squad-status delta (from `FUN_004d79c0` mode-1 vs mode-0 gap): asking
    // for KeyPlayer status costs ~20% more than SquadPlayer. Verified only as
    // an ordering (not exact %s), so kept as a modest additive tier bump.
    let tier_bump = match tier {
        SquadStatus::KeyPlayer      => 1.15,
        SquadStatus::FirstTeam      => 1.05,
        SquadStatus::FirstTeamSquad => 1.00,
        SquadStatus::SquadPlayer    => 0.95,
        SquadStatus::HotProspect    => 0.95,
        SquadStatus::DecentProspect => 0.90,
        SquadStatus::NotNeeded      => 0.85,
    };
    weekly = ((weekly as f64) * tier_bump) as u32;

    // "Wouldn't move for less" — the composer never proposes below existing
    // wage when contract_id is not 0xffffffff (existing contract branch).
    if let Some(c) = existing_contract {
        if weekly < c.weekly_wage { weekly = c.weekly_wage; }
    }
    // Also floor at the player's current wage from the person record.
    weekly = weekly.max(player.current_wage);

    // Hard clamps from `FUN_0084d5d0`:end — `_DAT_0095dbe0` / `_DAT_0095dbe4`.
    let weekly_wage = weekly.clamp(WAGE_FLOOR_WEEKLY, WAGE_CEIL_WEEKLY);

    // -- (4) Signing-on fee floor gate — `FUN_004d3ea0`: 50k default, 100k
    //    when player value > £1M. Then scaled by rep_scale so a mid club
    //    can't shovel top-tier signing-on money.
    let signing_on_floor = if player.market_value > 1_000_000 {
        SIGN_ON_FLOOR_HIGH
    } else {
        SIGN_ON_FLOOR_LOW
    };
    let signing_on_fee = (signing_on_floor as f64 * rep_scale)
        .max(signing_on_floor as f64 * 0.5) as u32;

    // -- (5) Contract length — `FUN_008ac0c0`: `rand()%4 + 2` for mode 0x0b
    //    (real contract), else min-length placeholder for the 0xff probe.
    let contract_years: u8 = if mode == 0x0b {
        let mut rng = crate::match_engine_exe::MatchRng::new(seed);
        (rng.range(4) + 2) as u8
    } else {
        1
    };
    // Age-taper — older players get shorter contracts, matches the
    // `[esp+0x17]` clamp path where the seed byte is min(bl, 4) for the
    // international-boost branch AND the person-record year at +0x18 gates it.
    let contract_years = if player.age >= 34 { contract_years.min(1) }
                         else if player.age >= 30 { contract_years.min(2) }
                         else { contract_years };

    // -- (6) Bonus amounts — `FUN_00848980` classifies these as
    //    {Goal, Assist, Clean-Sheet}. Faithful envelope: scaled by weekly.
    let goal_bonus        = (weekly_wage as f64 * 0.10) as u32;
    let assist_bonus      = (weekly_wage as f64 * 0.05) as u32;
    let clean_sheet_bonus = (weekly_wage as f64 * 0.08) as u32;
    let appearance_fee    = (weekly_wage as f64 * 0.02) as u32;
    let loyalty_bonus     = (signing_on_fee as f64 * 0.5) as u32;

    Some(WageOffer {
        player_id: player.player_id,
        club_id: club.club_id,
        contract_years,
        start_year: 0, // set by caller (start_date field id 8)
        tier_byte: mode,
        weekly_wage,
        signing_on_fee,
        appearance_fee,
        goal_bonus,
        assist_bonus,
        clean_sheet_bonus,
        loyalty_bonus,
        squad_status: tier,
        on_loan_list: false,
        loan: None,
        agent_fee_pct: 0,
    })
}

/// Convenience shim — synthesize a [`ComposerPlayer`] from a
/// [`crate::player_rating::RatedPlayer`] and current [`Contract`].
pub fn compose_wage_offer_from_rated(
    p: &crate::player_rating::RatedPlayer,
    club: ComposerClub,
    existing_contract: Option<&Contract>,
    tier: SquadStatus,
    mode: u8,
    seed: u64,
) -> Option<WageOffer> {
    compose_wage_offer(
        ComposerPlayer {
            player_id: p.staff_id,
            ca: p.ca, pa: p.pa,
            player_reputation: 0, // caller can override via ComposerPlayer directly
            market_value: p.market_value,
            current_wage: p.weekly_wage,
            age: p.age_est,
            international_caps: 0,
            // Default staff role byte = 5 (case-0 top-tier player scale).
            // Real value lives at `[offer+0x3d]`; caller passes via
            // `compose_wage_offer` directly when it needs a lower tier.
            role_byte: 5,
            // Agent presence unknown at this shim — default false uses the
            // no-agent branch (`(PA_norm*4+1)*NO_AGENT_SCALE*1e-4`), which is
            // the exe's fall-through when `[person+0x69] == 0`.
            has_agent: false,
        },
        club, existing_contract, tier, mode, seed,
    )
}

/// The from-rated shim wired to the full FUN_00580a90 cascade.
///
/// Derives [`WageCapInputs::minimal`] from the club's rep + wage field,
/// runs [`resolve_wage_cap`], and calls [`compose_wage_offer_with_cap`]
/// with the resolved value.
///
/// Callers with richer world context (nation classification, manager,
/// world-rep pool, sibling clubs) should build `WageCapInputs` directly
/// for full fidelity.
pub fn compose_wage_offer_from_rated_cascaded(
    p: &crate::player_rating::RatedPlayer,
    club: ComposerClub,
    club_wage_field: i32,
    total_clubs: i32,
    nation_count: i32,
    existing_contract: Option<&Contract>,
    tier: SquadStatus,
    mode: u8,
    seed: u64,
) -> Option<WageOffer> {
    let inputs = WageCapInputs::minimal(
        club.reputation as i16, club_wage_field, club.club_id as i32,
        total_clubs, nation_count,
    );
    let cap = resolve_wage_cap(inputs);
    compose_wage_offer_with_cap(
        ComposerPlayer {
            player_id: p.staff_id,
            ca: p.ca, pa: p.pa,
            player_reputation: 0,
            market_value: p.market_value,
            current_wage: p.weekly_wage,
            age: p.age_est,
            international_caps: 0,
            role_byte: 5,
            has_agent: false,
        },
        club, existing_contract, tier, mode, seed,
        Some(cap.max(0) as u32),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finance::ClubFinance;
    use crate::player_rating::{PlayerRatingBook, RatedPlayer};

    fn mk_player(id: u32, ca: i16, club: u32) -> RatedPlayer {
        RatedPlayer { staff_id: id, club_id: Some(club as i32), division_id: Some(24),
                      ca, pa: ca, goals_est: 10, age_est: 25, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 1_000_000, weekly_wage: 25_000, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] }
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
        let mut ratings = PlayerRatingBook { players, ..Default::default() };
        let mut market = TransferMarket::seed_from_ratings(&ratings, 2001);
        let mut finance = FinanceBook::default();
        finance.clubs.push(ClubFinance { club_id: 10, balance: 0, weekly_wage_bill: 0, transfer_budget: 0, months_in_the_red: 0, board_confidence: 10, month_wages: 0, month_gate: 0, month_tv_prize: 0, in_administration: false, ..Default::default() });
        finance.clubs.push(ClubFinance { club_id: 20, balance: 10_000_000, weekly_wage_bill: 0, transfer_budget: 10_000_000, months_in_the_red: 0, board_confidence: 10, month_wages: 0, month_gate: 0, month_tv_prize: 0, in_administration: false, ..Default::default() });

        // Market value = 100^2 * 100 = 1,000,000. Bid 2M with matching wage.
        market.submit_bid(TransferBid {
            bidding_club_id: 20, target_player_id: 1, selling_club_id: 10,
            amount: 2_000_000, player_wage_offer: 50_000, contract_years: 4, round: 0,
        });
        market.resolve_bids(2001, &mut ratings, &mut finance);
        assert_eq!(market.resolved_bids[0].1, BidOutcome::Accepted);
        assert_eq!(finance.for_club(20).unwrap().balance, 8_000_000);
        assert_eq!(finance.for_club(10).unwrap().balance, 2_000_000);
        assert_eq!(market.contract_for(1).unwrap().club_id, 20);
    }

    #[test]
    fn low_bid_gets_rejected() {
        let players = vec![mk_player(1, 100, 10)];
        let mut ratings = PlayerRatingBook { players, ..Default::default() };
        let mut market = TransferMarket::seed_from_ratings(&ratings, 2001);
        let mut finance = FinanceBook::default();
        market.submit_bid(TransferBid {
            bidding_club_id: 20, target_player_id: 1, selling_club_id: 10,
            amount: 100, player_wage_offer: 1000, contract_years: 3, round: 0,
        });
        market.resolve_bids(2001, &mut ratings, &mut finance);
        assert_eq!(market.resolved_bids[0].1, BidOutcome::Rejected);
        assert_eq!(market.contract_for(1).unwrap().club_id, 10);
    }

    #[test]
    fn near_market_bid_gets_countered() {
        let players = vec![mk_player(1, 100, 10)];
        let mut ratings = PlayerRatingBook { players, ..Default::default() };
        let mut market = TransferMarket::seed_from_ratings(&ratings, 2001);
        let mut finance = FinanceBook::default();
        // Market value 1M; bid 900k (90%) → counter.
        market.submit_bid(TransferBid {
            bidding_club_id: 20, target_player_id: 1, selling_club_id: 10,
            amount: 900_000, player_wage_offer: 50_000, contract_years: 3, round: 0,
        });
        market.resolve_bids(2001, &mut ratings, &mut finance);
        assert_eq!(market.resolved_bids[0].1, BidOutcome::Countered);
    }

    #[test]
    fn bosman_flag_triggers_when_close_to_expiry() {
        let players = vec![mk_player(1, 100, 10)];
        let mut ratings = PlayerRatingBook { players, ..Default::default() };
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
        let mut ratings = PlayerRatingBook { players, ..Default::default() };
        let market = TransferMarket::seed_from_ratings(&ratings, 2001);
        // Every player got a 5-year contract → expires 2006.
        let fa = market.free_agents_next_summer(2006);
        assert_eq!(fa.len(), 3);
    }

    // --- FUN_00848da0 port tests -----------------------------------------

    fn cp(ca: i16, pa: i16, rep: u16, value: i64, wage: u32, age: u8) -> ComposerPlayer {
        ComposerPlayer {
            player_id: 1, ca, pa, player_reputation: rep,
            market_value: value, current_wage: wage, age, international_caps: 0,
            role_byte: 5, has_agent: false,
        }
    }
    fn cc(rep: u16) -> ComposerClub { ComposerClub { club_id: 100, reputation: rep } }

    #[test]
    fn composer_sentinel_fires_when_player_rep_far_above_club() {
        // Top-tier player, small club → sentinel path in the asm.
        let player = cp(180, 190, /*rep*/ 8000, 25_000_000, 80_000, 26);
        let club = cc(/*rep*/ 2000);
        let off = compose_wage_offer(player, club, None, SquadStatus::KeyPlayer, 0x0b, 1);
        assert!(off.is_none(), "expected sentinel decline");
    }

    #[test]
    fn composer_ok_when_reps_are_compatible() {
        let player = cp(120, 130, 5000, 500_000, 8_000, 24);
        let club = cc(5000);
        let off = compose_wage_offer(player, club, None, SquadStatus::FirstTeam, 0x0b, 42)
            .expect("should compose");
        assert_eq!(off.player_id, 1);
        assert_eq!(off.club_id, 100);
    }

    #[test]
    fn composer_wage_clamped_below_ceiling() {
        // Absurdly rated player at top-rep club — should still cap.
        let player = cp(200, 200, 8000, 50_000_000, 200_000, 25);
        let club = cc(9000);
        let off = compose_wage_offer(player, club, None, SquadStatus::KeyPlayer, 0x0b, 7).unwrap();
        assert!(off.weekly_wage <= WAGE_CEIL_WEEKLY,
            "wage {} exceeded ceiling {}", off.weekly_wage, WAGE_CEIL_WEEKLY);
    }

    #[test]
    fn composer_wage_clamped_above_floor() {
        // Rock-bottom rated player at a mid club (compatible rep).
        let player = cp(1, 1, 3000, 1_000, 0, 22);
        let club = cc(3000);
        let off = compose_wage_offer(player, club, None, SquadStatus::NotNeeded, 0x0b, 3).unwrap();
        assert!(off.weekly_wage >= WAGE_FLOOR_WEEKLY,
            "wage {} under floor {}", off.weekly_wage, WAGE_FLOOR_WEEKLY);
    }

    #[test]
    fn composer_signing_on_floor_doubles_at_1m_value() {
        let cheap = cp(80, 90, 5000, 500_000, 5_000, 24);
        let rich  = cp(80, 90, 5000, 2_000_000, 5_000, 24);
        let club = cc(9000); // top club so rep_scale=1.0 leaves the floor intact
        let a = compose_wage_offer(cheap, club, None, SquadStatus::FirstTeam, 0x0b, 1).unwrap();
        let b = compose_wage_offer(rich,  club, None, SquadStatus::FirstTeam, 0x0b, 1).unwrap();
        assert_eq!(a.signing_on_fee, SIGN_ON_FLOOR_LOW);
        assert_eq!(b.signing_on_fee, SIGN_ON_FLOOR_HIGH);
    }

    #[test]
    fn composer_years_in_ai_mode_are_2_to_5() {
        // FUN_008ac0c0: rand()%4 + 2 → [2,5]. Sweep a few seeds.
        let player = cp(100, 110, 5000, 500_000, 5_000, 24);
        let club = cc(5000);
        for seed in 0..40u64 {
            let off = compose_wage_offer(player, club, None, SquadStatus::FirstTeam, 0x0b, seed).unwrap();
            assert!(off.contract_years >= 2 && off.contract_years <= 5,
                "seed {}: years={} out of [2,5]", seed, off.contract_years);
        }
    }

    #[test]
    fn wage_cap_scale_tables_are_dedup_pairs() {
        // Sanity check on the deduplication: the FOUR .rdata tables in the
        // exe collapse to TWO distinct payloads.
        assert_eq!(WAGE_CAP_SMALL.len(), 8);
        assert_eq!(WAGE_CAP_LARGE.len(), 8);
        assert_ne!(WAGE_CAP_SMALL, WAGE_CAP_LARGE);
        // Small tops out at 18k, large tops out at 30k.
        assert_eq!(*WAGE_CAP_SMALL.last().unwrap(), 18_000.0);
        assert_eq!(*WAGE_CAP_LARGE.last().unwrap(), 30_000.0);
        // Tier routing agrees with the exe's 4→2 collapse.
        assert!(std::ptr::eq(
            wage_cap_table_for(NationTier::Small),
            wage_cap_table_for(NationTier::Mid)));
        assert!(std::ptr::eq(
            wage_cap_table_for(NationTier::Large),
            wage_cap_table_for(NationTier::Top)));
    }

    #[test]
    fn wage_cap_rep_band_matches_exe_formulas() {
        use ClubFinanceStatus::*;

        // ---- Normal-status baseline (rep ≤ 5750): band = rep/50 exact.
        assert_eq!(wage_cap_rep_band(5000, NationTier::Mid,   Normal), 100);
        assert_eq!(wage_cap_rep_band(5750, NationTier::Top,   Normal), 115);
        assert_eq!(wage_cap_rep_band(   0, NationTier::Small, Normal),   0);

        // ---- Normal + non-top + rep > 5750: baseline + 5
        // rep=6000 → 120 + 5 = 125
        assert_eq!(wage_cap_rep_band(6000, NationTier::Large, Normal), 125);
        // rep=8000 → 160 + 5 = 165
        assert_eq!(wage_cap_rep_band(8000, NationTier::Mid,   Normal), 165);

        // ---- Normal + Top + rep > 5750: (rep-6250)/25 + 115
        // rep=6250 → 0 + 115 = 115 (baseline+5 path uses band+=5 → 130; top uses rebase)
        assert_eq!(wage_cap_rep_band(6250, NationTier::Top, Normal), 115);
        // rep=6500 → 250/25 + 115 = 10 + 115 = 125
        assert_eq!(wage_cap_rep_band(6500, NationTier::Top, Normal), 125);
        // rep=9000 → 2750/25 + 115 = 110 + 115 = 225 → clamps to 0xd2 = 210
        assert_eq!(wage_cap_rep_band(9000, NationTier::Top, Normal), 210);

        // ---- Administration + non-top + rep < 5251: baseline + 5
        // rep=5000 → 100 + 5 = 105
        assert_eq!(wage_cap_rep_band(5000, NationTier::Mid, Administration), 105);

        // ---- Administration + non-top + rep ≥ 5251: baseline + 10
        // rep=6000 → 120 + 10 = 130
        assert_eq!(wage_cap_rep_band(6000, NationTier::Large, Administration), 130);

        // ---- Administration + Top + rep ≥ 5251: (rep-5250)/25 + 105
        // BOUNDARY: rep=5250 is < 5251, so takes the +5 fallthrough path,
        // NOT the top-rebase (that only fires at rep ≥ 5251). 5250/50 + 5 = 110.
        assert_eq!(wage_cap_rep_band(5250, NationTier::Top, Administration), 110);
        // rep=5500 → top-rebase: (5500-5250)/25 + 105 = 10 + 105 = 115
        assert_eq!(wage_cap_rep_band(5500, NationTier::Top, Administration), 115);
        // rep=5251 → top-rebase: (5251-5250)/25 + 105 = 0 + 105 = 105 (integer div)
        assert_eq!(wage_cap_rep_band(5251, NationTier::Top, Administration), 105);

        // ---- Receivership + non-top + rep < 4751: baseline + 10
        // rep=3000 → 60 + 10 = 70
        assert_eq!(wage_cap_rep_band(3000, NationTier::Small, Receivership), 70);

        // ---- Receivership + non-top + rep ≥ 4751: baseline + 15
        // rep=5000 → 100 + 15 = 115
        assert_eq!(wage_cap_rep_band(5000, NationTier::Mid,   Receivership), 115);

        // ---- Receivership + Top + rep ≥ 4751: (rep-4750)/25 + 95
        // BOUNDARY: rep=4750 is < 4751, hits +10 fallthrough: 95 + 10 = 105.
        assert_eq!(wage_cap_rep_band(4750, NationTier::Top, Receivership), 105);
        // rep=4751 → top-rebase: (4751-4750)/25 + 95 = 0 + 95 = 95
        assert_eq!(wage_cap_rep_band(4751, NationTier::Top, Receivership), 95);
        // rep=5000 → top-rebase: (5000-4750)/25 + 95 = 10 + 95 = 105
        assert_eq!(wage_cap_rep_band(5000, NationTier::Top, Receivership), 105);

        // ---- 0xd2 = 210 hard cap
        assert!(wage_cap_rep_band(20_000, NationTier::Top, Normal) <= 210);
    }

    #[test]
    fn seniority_cap_dispatch_matches_exe_switch() {
        // Tier 1 and out-of-range → Pass
        assert_eq!(seniority_hard_cap_for(0), RoleSeniorityCapKind::Pass);
        assert_eq!(seniority_hard_cap_for(1), RoleSeniorityCapKind::Pass);
        assert_eq!(seniority_hard_cap_for(7), RoleSeniorityCapKind::Pass);
        assert_eq!(seniority_hard_cap_for(100), RoleSeniorityCapKind::Pass);
        // Negative i8 (0x80..0xff) → Pass
        assert_eq!(seniority_hard_cap_for(255), RoleSeniorityCapKind::Pass);
        assert_eq!(seniority_hard_cap_for(128), RoleSeniorityCapKind::Pass);

        // Tiers 2,3 need FUN_005ea590 gate
        assert_eq!(seniority_hard_cap_for(2),
            RoleSeniorityCapKind::NeedsGate { fallback_cap: 85_000 });
        assert_eq!(seniority_hard_cap_for(3),
            RoleSeniorityCapKind::NeedsGate { fallback_cap: 55_000 });

        // Tiers 4/5/6 are fully verified
        assert_eq!(seniority_hard_cap_for(4), RoleSeniorityCapKind::HardCap(37_500));
        assert_eq!(seniority_hard_cap_for(5), RoleSeniorityCapKind::HardCap(12_500));
        assert_eq!(seniority_hard_cap_for(6), RoleSeniorityCapKind::HardCap( 8_250));
    }

    #[test]
    fn seniority_cap_constants_match_exact_hex_from_exe() {
        // These are the exact 0x927c / 0x30d4 / 0x203a literals from the exe.
        assert_eq!(SENIORITY_CAP_TIER_4, 0x927c);
        assert_eq!(SENIORITY_CAP_TIER_5, 0x30d4);
        assert_eq!(SENIORITY_CAP_TIER_6, 0x203a);
    }

    #[test]
    fn final_clamp_bumps_estimate_to_floor_plus_100() {
        // estimate is small; wage_floor=200 → estimate lifted to 300
        // counter_party=10000 >= floor 200, and 10000 > 300 → return 300
        let out = final_wage_clamp_assembly(50, 10_000, 200);
        assert_eq!(out, 300);
    }

    #[test]
    fn final_clamp_returns_floor_when_counter_party_below_floor() {
        // counter_party_wage < wage_floor → return wage_floor unmodified
        let out = final_wage_clamp_assembly(50_000, 150, 200);
        assert_eq!(out, 200);
    }

    #[test]
    fn final_clamp_returns_min_of_estimate_and_counterparty() {
        // Both non-degenerate: estimate=5000, counter_party=12000, floor=200
        // estimate stays 5000 (>200+100=300), counter_party=12000 > floor
        // → return min(12000, 5000) = 5000
        let out = final_wage_clamp_assembly(5_000, 12_000, 200);
        assert_eq!(out, 5_000);
    }

    #[test]
    fn final_clamp_estimate_binds_when_lower_than_counterparty() {
        // The bumped estimate becomes the min when it's smaller
        let out = final_wage_clamp_assembly(800, 5_000, 500);
        // 500+100=600, estimate=max(800,600)=800; 5000 > 500 floor; min(5000, 800)=800
        assert_eq!(out, 800);
    }

    #[test]
    fn wage_cap_inputs_minimal_defaults_produce_valid_cap() {
        // Base probe minimal: rep=5000, wage_field=25000, id=42
        let inputs = WageCapInputs::minimal(5000, 25_000, 42, 5000, 200);
        let cap = resolve_wage_cap(inputs);
        assert!(cap >= WAGE_FLOOR_BASE + 100,
                "cap ({}) below floor+100", cap);
    }

    // (compose_from_rated_cascaded is exercised at the caller-integration
    // layer once run_ai_transfer_pass wires it. Constructing a RatedPlayer
    // here requires ~30 fields; verifying the wired path via a functional
    // test upstream is cleaner.)

    #[test]
    fn compose_with_cap_none_matches_original_compose() {
        let p = cp(100, 110, 5000, 500_000, 5_000, 24);
        let c = cc(5000);
        let a = compose_wage_offer(p, c, None, SquadStatus::FirstTeam, 0x0b, 42).unwrap();
        let b = compose_wage_offer_with_cap(
            p, c, None, SquadStatus::FirstTeam, 0x0b, 42, None).unwrap();
        assert_eq!(a.weekly_wage, b.weekly_wage);
        assert_eq!(a.signing_on_fee, b.signing_on_fee);
        assert_eq!(a.contract_years, b.contract_years);
    }

    #[test]
    fn compose_with_cap_supplied_uses_that_cap() {
        // current_wage=800 so it doesn't override our test cap.
        let p = cp(100, 110, 5000, 500_000, 800, 24);
        let c = cc(5000);
        let low = compose_wage_offer_with_cap(
            p, c, None, SquadStatus::FirstTeam, 0x0b, 42, Some(2_000)).unwrap();
        let high = compose_wage_offer_with_cap(
            p, c, None, SquadStatus::FirstTeam, 0x0b, 42, Some(100_000)).unwrap();
        // The caller-supplied cap acts as a ceiling on the base formula
        // (before the current_wage floor and hard clamp are applied).
        // low.weekly_wage clamped at WAGE_FLOOR_WEEKLY (750) minimum,
        // capped by min(2000, formula_wage), then floored at current_wage.
        assert!(low.weekly_wage <= 2_000, "low cap: {}", low.weekly_wage);
        assert!(high.weekly_wage >= low.weekly_wage,
                "high cap should permit >= wage than low cap");
    }

    #[test]
    fn compose_with_cap_below_floor_returns_none() {
        let p = cp(100, 110, 5000, 500_000, 5_000, 24);
        let c = cc(5000);
        // cap below WAGE_FLOOR_WEEKLY (750) → sentinel path → None
        let out = compose_wage_offer_with_cap(
            p, c, None, SquadStatus::FirstTeam, 0x0b, 42, Some(500));
        assert!(out.is_none(), "expected None for cap {}", 500);
    }

    fn default_cap_inputs() -> WageCapInputs {
        WageCapInputs {
            club_reputation: 5000,
            club_wage_field: 25_000,
            club_flag_byte: 0,
            club_status_byte: 0,
            club_id: 42,
            total_clubs: 5000,
            nation_count: 200,
            outer_frame_tier: NationTier::Mid,
            agent_group: AgentNationGroup::Default,
            big3: Big3NationMembership::No,
            cp_tail_nation: CpTailNation::Other,
            top5: Top5Nation::No,
            nation_league_strength: 2,
            nation_world_rank: 40,
            finance_status: ClubFinanceStatus::Normal,
            world_rep_value: 1000,
            sibling_club_reputation: None,
            linked_parent_reputation: None,
            counter_party: None,
        }
    }

    #[test]
    fn resolve_wage_cap_base_probe_no_counter_party() {
        // Base probe path — no counter_party set.
        // Should route through quadratic + world_rep + cp_tail + final clamp.
        let cap = resolve_wage_cap(default_cap_inputs());
        // Sanity: cap is at least WAGE_FLOOR_BASE + 100
        assert!(cap >= WAGE_FLOOR_BASE + 100);
        // And bounded above by club_wage_field range (with bumps)
        assert!(cap < 200_000, "unexpectedly high cap: {}", cap);
    }

    #[test]
    fn resolve_wage_cap_ghost_club_short_circuits() {
        let mut inputs = default_cap_inputs();
        inputs.club_id = 4700;  // in ghost region for 5000/200 config
        let cap = resolve_wage_cap(inputs);
        // Ghost path skips wage-cap logic; still returns a valid clamp
        assert!(cap >= WAGE_FLOOR_BASE + 100);
    }

    #[test]
    fn resolve_wage_cap_player_specific_uses_counter_party() {
        let mut inputs = default_cap_inputs();
        inputs.counter_party = Some(CounterPartyContext {
            staff: CounterPartyStaff {
                role_byte: 5, current_club_id: 42, has_agent: true, is_player: true,
            },
            at_this_club: true,
            current_wage: 15_000,
            squad_status_tier: 3,
            player_view: PlayerRatingCapView {
                reputation: 6000, world_reputation: 100,
                potential: 5000, age: 25, has_caps: true,
            },
            top5_view: Top5BonusView {
                nation: Top5Nation::No, club_reputation: 5000,
                has_type10: true, player_reputation: 6000, player_potential: 5000,
                has_local_14: false, local_14_squad_status_byte: 0,
                is_at_this_club: true, days_since_contract_start: 100,
            },
            manager_view: ManagerBonusView {
                has_manager: false, manager_has_person: false,
                club_reputation: 5000, has_type10: true, player_reputation: 6000,
                manager_tactical_match: false, manager_nationality_match: false,
                manager_style_byte: 10, club_flag_byte: 0,
                manager_adaptability: 10, manager_attribute_57: 10,
            },
            seniority_gate_view: SeniorityGateView {
                squad_slot_open: false, has_type10: true, player_reputation: 6000,
            },
        });
        let cap = resolve_wage_cap(inputs);
        // Player-specific path with is_player=true skips role cap
        assert!(cap >= WAGE_FLOOR_BASE + 100);
    }

    #[test]
    fn resolve_wage_cap_admin_status_bumps_via_cp_tail_for_big3() {
        // For Big3 nations, cp_tail_no_counter_party applies the finance-status
        // scaling (1.05x for Admin), which propagates through the final clamp.
        // Add a large club_wage_field so the quadratic base ≠ floor.
        let mut normal_inputs = default_cap_inputs();
        normal_inputs.cp_tail_nation = CpTailNation::Big3;
        normal_inputs.club_reputation = 8000;  // rep > 4250 gate for Big3+Normal
        let normal = resolve_wage_cap(normal_inputs);

        let mut admin_inputs = normal_inputs;
        admin_inputs.finance_status = ClubFinanceStatus::Administration;
        let admin = resolve_wage_cap(admin_inputs);

        // Both fire finance-status bumps in cp_tail; Admin gets 1.05 vs
        // Normal's 1.025 (Big3+Normal at rep>4250 gets 1.025 bump).
        assert!(admin > normal,
                "admin ({}) should exceed normal ({})", admin, normal);
    }

    #[test]
    fn resolve_wage_cap_receivership_higher_than_admin() {
        let base_inputs = {
            let mut x = default_cap_inputs();
            x.cp_tail_nation = CpTailNation::Big3;
            x.club_reputation = 8000;
            x
        };
        let mut admin_inputs = base_inputs;
        admin_inputs.finance_status = ClubFinanceStatus::Administration;
        let mut recv_inputs = base_inputs;
        recv_inputs.finance_status = ClubFinanceStatus::Receivership;
        let admin = resolve_wage_cap(admin_inputs);
        let recv = resolve_wage_cap(recv_inputs);
        // Recv 1.10 > Admin 1.05
        assert!(recv > admin, "recv ({}) should exceed admin ({})", recv, admin);
    }

    #[test]
    fn world_rep_bump_clamps_at_1_0_when_ratio_low() {
        // world=500, rep=1000 → ratio=0.5, clamped to 1.0 → no bump
        assert_eq!(world_rep_wage_bump(10_000, 500, 1000), 10_000);
    }

    #[test]
    fn world_rep_bump_clamps_at_1_2_when_ratio_high() {
        // world=10000, rep=1000 → ratio=10.0, clamped to 1.2 → 1.2× bump
        assert_eq!(world_rep_wage_bump(10_000, 10_000, 1000), 12_000);
    }

    #[test]
    fn world_rep_bump_linear_between_clamps() {
        // world=1500, rep=1000 → ratio=1.5, clamped to 1.2 (>1.2)
        // wait actually 1.5 > 1.2 → capped at 1.2
        assert_eq!(world_rep_wage_bump(10_000, 1500, 1000), 12_000);
        // world=1100, rep=1000 → ratio=1.1 (in range) → 11_000
        assert_eq!(world_rep_wage_bump(10_000, 1100, 1000), 11_000);
    }

    #[test]
    fn world_rep_bump_min_club_rep_floor_at_1000() {
        // rep=500 (below floor) → clamped to 1000
        // world=1500 → ratio=1.5→1.2 → 12_000
        assert_eq!(world_rep_wage_bump(10_000, 1500, 500), 12_000);
    }

    fn sgate(open: bool, has_t10: bool, prep: i16) -> SeniorityGateView {
        SeniorityGateView { squad_slot_open: open, has_type10: has_t10,
                            player_reputation: prep }
    }

    #[test]
    fn seniority_gate_tier_3_caps_at_55000() {
        // gate fires → cap at 55000
        let v = sgate(true, true, 5000);
        assert_eq!(seniority_gate_resolves(3, v, 100_000), Some(55_000));
        // estimate under cap → unchanged
        assert_eq!(seniority_gate_resolves(3, v, 40_000),  Some(40_000));
    }

    #[test]
    fn seniority_gate_tier_2_conditional_85000_cap() {
        // squad_slot_open triggers secondary gate → 85000 cap
        let v = sgate(true, true, 5000);
        assert_eq!(seniority_gate_resolves(2, v, 100_000), Some(85_000));
        // rep < 7750 also triggers secondary gate
        let v2 = sgate(false, true, 5000);
        // But wait — outer gate needs squad_slot_open || rep < 6751
        // rep=5000 < 6751 → outer gate fires; then secondary: rep<7750 → 85000 cap
        assert_eq!(seniority_gate_resolves(2, v2, 100_000), Some(85_000));
    }

    #[test]
    fn seniority_gate_falls_through_when_no_gate_fires() {
        // No squad slot open, player rep >= 6751 → gate doesn't fire → None
        let v = sgate(false, true, 7000);
        assert_eq!(seniority_gate_resolves(3, v, 100_000), None);
        assert_eq!(seniority_gate_resolves(2, v, 100_000), None);
    }

    #[test]
    fn seniority_gate_no_type10_no_rep_check() {
        // No type10 record + squad slot closed → outer gate false → None
        let v = sgate(false, false, 0);
        assert_eq!(seniority_gate_resolves(3, v, 100_000), None);
    }

    fn mgr_view(has_mgr: bool, has_person: bool, club_rep: i16,
                has_t10: bool, prep: i16, tac: bool, nat: bool,
                style: i8, flag: u8, adapt: i8, attr57: i8)
                -> ManagerBonusView {
        ManagerBonusView {
            has_manager: has_mgr, manager_has_person: has_person,
            club_reputation: club_rep, has_type10: has_t10,
            player_reputation: prep, manager_tactical_match: tac,
            manager_nationality_match: nat, manager_style_byte: style,
            club_flag_byte: flag, manager_adaptability: adapt,
            manager_attribute_57: attr57,
        }
    }

    #[test]
    fn manager_bonus_no_manager_no_bonuses() {
        let v = mgr_view(false, true, 5000, true, 6000, true, true, 10, 0, 20, 20);
        let out = manager_bonus_verdict(v);
        assert!(!out.base && !out.tactical_or_nationality_match
                && !out.style_out_of_range && !out.high_attr_flag0_club);
    }

    #[test]
    fn manager_bonus_rep_gate_stops_all() {
        // club_rep 8000, player_rep 6000 → 8000 < 6000 + 1250 = 7250? No, 8000 > 7250 → gate fails
        let v = mgr_view(true, true, 8000, true, 6000, true, true, 10, 0, 20, 20);
        let out = manager_bonus_verdict(v);
        assert!(!out.base);
    }

    #[test]
    fn manager_bonus_base_fires_on_close_club_player_rep() {
        // club_rep 7000, player_rep 6000 → 7000 < 7250 → base fires
        let v = mgr_view(true, true, 7000, true, 6000, false, false, 10, 1, 5, 5);
        let out = manager_bonus_verdict(v);
        assert!(out.base);
        assert!(!out.tactical_or_nationality_match);
        assert!(!out.style_out_of_range);   // style=10 in [6..=15]
        assert!(!out.high_attr_flag0_club); // flag != 0
    }

    #[test]
    fn manager_bonus_tactical_match_fires_second_bonus() {
        let v = mgr_view(true, true, 6000, true, 6000, true, false, 10, 0, 5, 5);
        let out = manager_bonus_verdict(v);
        assert!(out.base && out.tactical_or_nationality_match);
    }

    #[test]
    fn manager_bonus_style_out_of_range_fires_third() {
        // style = 5 (below MIN 6) or 16 (above MAX 15) → out of range
        for style in [5i8, 16, 0, 20] {
            let v = mgr_view(true, true, 6000, true, 6000, false, false, style, 1, 5, 5);
            assert!(manager_bonus_verdict(v).style_out_of_range, "style={}", style);
        }
        // In range → not out
        for style in [6i8, 10, 15] {
            let v = mgr_view(true, true, 6000, true, 6000, false, false, style, 1, 5, 5);
            assert!(!manager_bonus_verdict(v).style_out_of_range, "style={}", style);
        }
    }

    #[test]
    fn manager_bonus_high_attr_flag0_all_three_conditions() {
        // flag=0 AND adapt>15 AND attr57>15
        let v = mgr_view(true, true, 6000, true, 6000, false, false, 10, 0, 16, 16);
        assert!(manager_bonus_verdict(v).high_attr_flag0_club);
        // flag=1 → false
        let v_f1 = mgr_view(true, true, 6000, true, 6000, false, false, 10, 1, 16, 16);
        assert!(!manager_bonus_verdict(v_f1).high_attr_flag0_club);
        // adapt=15 → false (strict >)
        let v_a = mgr_view(true, true, 6000, true, 6000, false, false, 10, 0, 15, 16);
        assert!(!manager_bonus_verdict(v_a).high_attr_flag0_club);
    }

    fn top5(nation: Top5Nation, club_rep: i16, has_t10: bool, prep: i16,
            ppot: i16, hl14: bool, ss: u8, at_club: bool, days: i32)
            -> Top5BonusView {
        Top5BonusView {
            nation, club_reputation: club_rep, has_type10: has_t10,
            player_reputation: prep, player_potential: ppot,
            has_local_14: hl14, local_14_squad_status_byte: ss,
            is_at_this_club: at_club, days_since_contract_start: days,
        }
    }

    #[test]
    fn top5_no_nation_short_circuits() {
        let v = top5(Top5Nation::No, 8000, true, 7000, 6000, true, 0x10, true, 50);
        assert!(!top5_nation_bonus_fires(v));
    }

    #[test]
    fn top5_low_club_rep_hard_reject() {
        let v = top5(Top5Nation::Yes, 4250, true, 8000, 8000, true, 0x10, true, 50);
        assert!(!top5_nation_bonus_fires(v));
    }

    #[test]
    fn top5_low_rep_band_no_active_offer_needs_elite_player() {
        // club_rep = 5000 (in low band 4251..6250), no active offer.
        // Fires iff player_rep >= 7751
        let v_elite = top5(Top5Nation::Yes, 5000, true, 7751, 5000, false, 0, false, 0);
        assert!(top5_nation_bonus_fires(v_elite));
        let v_low = top5(Top5Nation::Yes, 5000, true, 7750, 5000, false, 0, false, 0);
        assert!(!top5_nation_bonus_fires(v_low));
    }

    #[test]
    fn top5_low_rep_band_own_club_high_player_rep_always_fires() {
        // club_rep=5000, own club, active offer, player rep >= 6751
        let v = top5(Top5Nation::Yes, 5000, true, 6751, 5000, true, 0, true, 50);
        assert!(top5_nation_bonus_fires(v));
    }

    #[test]
    fn top5_low_rep_band_own_club_young_player_needs_squad_status() {
        // player_rep < 6751 → squad status gate fires
        // nibble 0x10 (KEY) → fires
        let v = top5(Top5Nation::Yes, 5000, true, 6000, 5000, true, 0x10, true, 50);
        assert!(top5_nation_bonus_fires(v));
        // nibble 0x00 → doesn't fire
        let v0 = top5(Top5Nation::Yes, 5000, true, 6000, 5000, true, 0x00, true, 50);
        assert!(!top5_nation_bonus_fires(v0));
    }

    #[test]
    fn top5_high_rep_band_no_active_offer_needs_pot_or_rep() {
        // club_rep=7000, no active offer. Fires iff potential>=5251 OR rep>=6751
        let v_pot = top5(Top5Nation::Yes, 7000, true, 5000, 5251, false, 0, false, 0);
        assert!(top5_nation_bonus_fires(v_pot));
        let v_rep = top5(Top5Nation::Yes, 7000, true, 6751, 4000, false, 0, false, 0);
        assert!(top5_nation_bonus_fires(v_rep));
        let v_neither = top5(Top5Nation::Yes, 7000, true, 6000, 4000, false, 0, false, 0);
        assert!(!top5_nation_bonus_fires(v_neither));
    }

    #[test]
    fn top5_squad_status_rotate_nibble_needs_min_rep() {
        // nibble 0x30 needs player_rep >= 5251
        let v_pass = top5(Top5Nation::Yes, 5000, true, 5251, 4000, true, 0x30, true, 50);
        // Wait: player_rep 5251 makes it hit >=TOP5_REP_TOP (6751)? No, 5251 < 6751, so young.
        // Then squad_status_gate: nibble 0x30 && 5251 >= 5251 → true
        assert!(top5_nation_bonus_fires(v_pass));

        let v_fail = top5(Top5Nation::Yes, 5000, true, 5250, 4000, true, 0x30, true, 50);
        assert!(!top5_nation_bonus_fires(v_fail));
    }

    #[test]
    fn top5_stale_offer_treated_as_no_offer() {
        // has_local_14=true, at_this_club=true, but days>199
        // → uses the no-active-offer path
        // club_rep=5000 → low band → needs rep>=7751
        let v = top5(Top5Nation::Yes, 5000, true, 7000, 8000, true, 0x10, true, 200);
        assert!(!top5_nation_bonus_fires(v));  // rep 7000 < 7751
        let v_ok = top5(Top5Nation::Yes, 5000, true, 7751, 8000, true, 0x10, true, 200);
        assert!(top5_nation_bonus_fires(v_ok));
    }

    #[test]
    fn quadratic_wage_base_matches_verified_formula() {
        // rep=1000, local_8=1.0, club_wage=500
        //   1_000_000 * 1.0 * 0.0001 + 500 = 100 + 500 = 600
        assert_eq!(quadratic_wage_base(1000, 1.0, 500), 600);
        // rep=5000, local_8=0.5, club_wage=0
        //   25_000_000 * 0.5 * 0.0001 = 1250
        assert_eq!(quadratic_wage_base(5000, 0.5, 0), 1250);
        // rep=0 → base 0 + club_wage
        assert_eq!(quadratic_wage_base(0, 1.0, 100), 100);
    }

    fn crv(has_type10: bool, rep: i16, age: u8) -> CpSeniorityRebase {
        CpSeniorityRebase { has_type10, player_reputation: rep, age }
    }

    #[test]
    fn cp_rebase_no_type10_no_rebase() {
        let v = crv(false, 6000, 25);
        assert_eq!(counter_party_seniority_rebase(v, 3, 0, 100_000),
                   CpRebaseVerdict::NoRebase);
    }

    #[test]
    fn cp_rebase_low_player_rep_no_rebase() {
        // rep=5250 is exactly the gate — NOT strict greater, so no rebase
        let v = crv(true, 5250, 25);
        assert_eq!(counter_party_seniority_rebase(v, 3, 0, 100_000),
                   CpRebaseVerdict::NoRebase);
    }

    #[test]
    fn cp_rebase_high_age_no_rebase() {
        // age=32 is exactly at cutoff — NOT less than, no rebase
        let v = crv(true, 6000, 32);
        assert_eq!(counter_party_seniority_rebase(v, 3, 0, 100_000),
                   CpRebaseVerdict::NoRebase);
    }

    #[test]
    fn cp_rebase_key_player_below_threshold() {
        // KeyPlayer: threshold = sibling_adjust × 0.8 = 100_000 × 0.8 = 80_000
        // current_wage=50_000 < 80_000 → KeyPlayerGate
        let v = crv(true, 6000, 25);
        let verdict = counter_party_seniority_rebase(v, 3, 50_000, 100_000);
        match verdict {
            CpRebaseVerdict::KeyPlayerGate { threshold } => {
                assert!((threshold - 80_000.0).abs() < 1e-9);
            }
            other => panic!("expected KeyPlayerGate, got {:?}", other),
        }
        // Above threshold: no rebase
        let above = counter_party_seniority_rebase(v, 3, 90_000, 100_000);
        assert_eq!(above, CpRebaseVerdict::NoRebase);
    }

    #[test]
    fn cp_rebase_first_team_below_threshold() {
        // FirstTeam: threshold = sibling_adjust × 0.9 = 100_000 × 0.9 = 90_000
        let v = crv(true, 6000, 25);
        let verdict = counter_party_seniority_rebase(v, 2, 80_000, 100_000);
        match verdict {
            CpRebaseVerdict::FirstTeamGate { threshold } => {
                assert!((threshold - 90_000.0).abs() < 1e-9);
            }
            other => panic!("expected FirstTeamGate, got {:?}", other),
        }
    }

    #[test]
    fn cp_rebase_squad_player_snaps_to_sibling_adjust() {
        let v = crv(true, 6000, 25);
        // wage < sibling_adjust → SetToSiblingAdjust
        assert_eq!(counter_party_seniority_rebase(v, 1, 50_000, 100_000),
                   CpRebaseVerdict::SetToSiblingAdjust);
        // wage >= sibling_adjust → no rebase
        assert_eq!(counter_party_seniority_rebase(v, 1, 100_000, 100_000),
                   CpRebaseVerdict::NoRebase);
    }

    #[test]
    fn cp_rebase_squad_status_outside_1_2_3_no_rebase() {
        let v = crv(true, 6000, 25);
        for tier in [0u8, 4, 5, 6, 7] {
            assert_eq!(counter_party_seniority_rebase(v, tier, 0, 100_000),
                       CpRebaseVerdict::NoRebase, "tier={}", tier);
        }
    }

    #[test]
    fn cp_tail_blend_below_sibling_floor() {
        // wage=1000 < sibling_adjust=5000 → blend: 1000*0.75 + 5000*0.25 = 750+1250 = 2000
        let out = cp_tail_no_counter_party(
            1000, 5000, false, CpTailNation::Other,
            ClubFinanceStatus::Normal, 3000, 0);
        assert_eq!(out, 2000);
    }

    #[test]
    fn cp_tail_ghost_club_skips_nation_bump() {
        // Would bump for Big3+Admin normally, but ghost_club=true skips
        let bumped = cp_tail_no_counter_party(
            10_000, 0, false, CpTailNation::Big3,
            ClubFinanceStatus::Administration, 5000, 0);
        assert_eq!(bumped, 10_500);  // 10000 * 1.05
        let skipped = cp_tail_no_counter_party(
            10_000, 0, true, CpTailNation::Big3,
            ClubFinanceStatus::Administration, 5000, 0);
        assert_eq!(skipped, 10_000);  // no bump
    }

    #[test]
    fn cp_tail_big3_normal_rep_gate() {
        // rep <= 4250 (0x109a): no bump
        let low = cp_tail_no_counter_party(
            10_000, 0, false, CpTailNation::Big3,
            ClubFinanceStatus::Normal, 4250, 0);
        assert_eq!(low, 10_000);
        // rep > 4250: 1.025 bump
        let high = cp_tail_no_counter_party(
            10_000, 0, false, CpTailNation::Big3,
            ClubFinanceStatus::Normal, 4251, 0);
        assert_eq!(high, 10_250);
    }

    #[test]
    fn cp_tail_second_tier_normal_no_bump() {
        let out = cp_tail_no_counter_party(
            10_000, 0, false, CpTailNation::SecondTier,
            ClubFinanceStatus::Normal, 8000, 0);
        assert_eq!(out, 10_000);
        // But Admin fires
        let admin = cp_tail_no_counter_party(
            10_000, 0, false, CpTailNation::SecondTier,
            ClubFinanceStatus::Administration, 8000, 0);
        assert_eq!(admin, 10_500);
    }

    #[test]
    fn cp_tail_low_rep_remap_fires_at_flag1() {
        // club_flag=1, rep=1000 (<2250), wage=5000 (>2100)
        // remap: (5000-2100)*0.2 + 2100 = 580 + 2100 = 2680
        let out = cp_tail_no_counter_party(
            5000, 0, false, CpTailNation::Other,
            ClubFinanceStatus::Normal, 1000, 1);
        assert_eq!(out, 2680);
    }

    #[test]
    fn cp_tail_low_rep_remap_gated_by_rep() {
        // rep=2250 → gate NOT satisfied (< strict), no remap
        let out = cp_tail_no_counter_party(
            5000, 0, false, CpTailNation::Other,
            ClubFinanceStatus::Normal, 2250, 1);
        assert_eq!(out, 5000);
    }

    #[test]
    fn cp_tail_low_rep_min_bumps_below_250() {
        // club_flag=1, wage=100 → bumped to 250 minimum
        let out = cp_tail_no_counter_party(
            100, 0, false, CpTailNation::Other,
            ClubFinanceStatus::Normal, 5000, 1);
        assert_eq!(out, 250);
    }

    fn view(rep: i16, world: i16, potential: i16, age: u8, has_caps: bool)
            -> PlayerRatingCapView {
        PlayerRatingCapView { reputation: rep, world_reputation: world,
            potential, age, has_caps }
    }

    #[test]
    fn player_cap_youth_low_world_rep_ages() {
        // rep=100, world=50 (< 60), age split at 24
        assert_eq!(player_rating_wage_ceiling(view(100, 50, 0, 20, false), 3), 7_500);
        assert_eq!(player_rating_wage_ceiling(view(100, 50, 0, 24, false), 3), 5_000);
    }

    #[test]
    fn player_cap_youth_mid_world_rep_seniority_and_age() {
        // rep=100, world=80 (60..99): 15000 if seniority 1-3 OR age > 23
        assert_eq!(player_rating_wage_ceiling(view(100, 80, 0, 20, false), 1), 15_000);
        assert_eq!(player_rating_wage_ceiling(view(100, 80, 0, 20, false), 4), 10_000); // low senior
        assert_eq!(player_rating_wage_ceiling(view(100, 80, 0, 30, false), 4), 15_000); // age>23
    }

    #[test]
    fn player_cap_youth_high_world_rep() {
        // rep=100, world=100+ → 25000 regardless
        assert_eq!(player_rating_wage_ceiling(view(100, 100, 0, 20, false), 4), 25_000);
        assert_eq!(player_rating_wage_ceiling(view(100, 200, 0, 40, true), 6), 25_000);
    }

    #[test]
    fn player_cap_midtier_world_split() {
        // rep=4000 (mid): 30000 if world>99, else 25000
        assert_eq!(player_rating_wage_ceiling(view(4000, 100, 0, 25, false), 3), 30_000);
        assert_eq!(player_rating_wage_ceiling(view(4000, 99,  0, 25, false), 3), 25_000);
        assert_eq!(player_rating_wage_ceiling(view(4000, 50,  0, 25, true),  3), 25_000);
    }

    #[test]
    fn player_cap_toptier_agile_no_caps_low_potential() {
        // rep=6000 (top-tier), young, world<140, potential<3750, no caps
        assert_eq!(player_rating_wage_ceiling(view(6000, 100, 3000, 25, false), 3), 40_000);
        // with caps: 45000
        assert_eq!(player_rating_wage_ceiling(view(6000, 100, 3000, 25, true),  3), 45_000);
    }

    #[test]
    fn player_cap_toptier_first_team_high_world_rep() {
        // rep=6000, young, world=150, seniority=1, no caps → 80000
        assert_eq!(player_rating_wage_ceiling(view(6000, 150, 5000, 25, false), 1), 80_000);
        // world=140 (>= 140), seniority 1, no caps → 65000 (world_rep NOT > 139: 140 > 139 is TRUE → 80000)
        assert_eq!(player_rating_wage_ceiling(view(6000, 140, 5000, 25, false), 1), 80_000);
        // world=139 → 65000
        assert_eq!(player_rating_wage_ceiling(view(6000, 139, 5000, 25, false), 1), 65_000);
        // with caps → 57500
        assert_eq!(player_rating_wage_ceiling(view(6000, 200, 5000, 25, true),  1), 57_500);
    }

    #[test]
    fn player_cap_toptier_low_seniority() {
        // rep=6000, young, world=200, seniority=4 → fall through 45000
        assert_eq!(player_rating_wage_ceiling(view(6000, 200, 5000, 25, false), 4), 45_000);
    }

    #[test]
    fn player_cap_toptier_veteran() {
        // rep=6000, age>=35, world<=119
        assert_eq!(player_rating_wage_ceiling(view(6000, 100, 5000, 36, false), 1), 37_500);
        assert_eq!(player_rating_wage_ceiling(view(6000, 100, 5000, 36, false), 4), 32_500);
    }

    #[test]
    fn player_cap_elite_potential_split() {
        // rep=7250+, potential<6750 → 100k / 125k
        assert_eq!(player_rating_wage_ceiling(view(7250, 200, 5000, 25, false), 1), 100_000);
        assert_eq!(player_rating_wage_ceiling(view(7250, 200, 5000, 25, true),  1), 125_000);
        // potential >= 6750 → 175k
        assert_eq!(player_rating_wage_ceiling(view(8000, 200, 7000, 25, true),  1), 175_000);
    }

    #[test]
    fn ghost_club_predicate_matches_exe_condition() {
        // With 5000 total clubs and 200 nations, the generated tail
        // starts at 5000 - 200*2 = 4600. Clubs 0..4599 are real,
        // 4600..4999 are generated.
        assert!(!is_generated_ghost_club(0,    5000, 200));
        assert!(!is_generated_ghost_club(4599, 5000, 200));
        assert!( is_generated_ghost_club(4600, 5000, 200));
        assert!( is_generated_ghost_club(4999, 5000, 200));

        // Boundary: when total == nations*2, ALL clubs are "generated"
        assert!( is_generated_ghost_club(0, 400, 200));

        // Zero-nations degenerate case: no tail, no clubs are ghost
        assert!(!is_generated_ghost_club(0,    5000, 0));
        assert!(!is_generated_ghost_club(4999, 5000, 0));
        assert!( is_generated_ghost_club(5000, 5000, 0));  // >= end
    }

    #[test]
    fn sibling_adjust_returns_0_outside_big3() {
        // Not in big-3 → always 0
        let x = sibling_adjust_contribution(
            Big3NationMembership::No, 8000, 0, ClubFinanceStatus::Normal, 5000);
        assert_eq!(x, 0);
    }

    #[test]
    fn sibling_adjust_returns_0_below_min_rep_gate() {
        // In big-3 but rep <= 4750 → 0
        let x = sibling_adjust_contribution(
            Big3NationMembership::Yes, 4750, 0, ClubFinanceStatus::Normal, 5000);
        assert_eq!(x, 0);
        let y = sibling_adjust_contribution(
            Big3NationMembership::Yes, 4751, 0, ClubFinanceStatus::Normal, 5000);
        assert_ne!(y, 0);
    }

    #[test]
    fn sibling_adjust_low_rep_uses_same_formula_all_statuses() {
        // rep=5000 < 5750 (low-rep branch): (l30-sub)*6, no status split
        // premium_bit=0 → sub=750
        let expected = (5000 - 750) * 6;   // 25_500
        for status in [ClubFinanceStatus::Normal,
                       ClubFinanceStatus::Administration,
                       ClubFinanceStatus::Receivership] {
            let x = sibling_adjust_contribution(
                Big3NationMembership::Yes, 5000, 0, status, 5000);
            assert_eq!(x, expected, "status={:?}", status);
        }
    }

    #[test]
    fn sibling_adjust_high_rep_normal_uses_6x_multiplier() {
        // rep=8000 >= 5750, status=Normal, premium=0 → (l30-750)*6
        let x = sibling_adjust_contribution(
            Big3NationMembership::Yes, 8000, 0, ClubFinanceStatus::Normal, 5000);
        assert_eq!(x, (5000 - 750) * 6);   // 25_500
    }

    #[test]
    fn sibling_adjust_high_rep_admin_uses_special_l30_8x_minus_offset() {
        // rep=8000, Admin, premium=0 → l30*8 - 6000
        let x = sibling_adjust_contribution(
            Big3NationMembership::Yes, 8000, 0, ClubFinanceStatus::Administration, 5000);
        assert_eq!(x, 5000 * 8 - 6000);    // 34_000

        // premium=1 → l30*8 - 10000
        let y = sibling_adjust_contribution(
            Big3NationMembership::Yes, 8000, 1, ClubFinanceStatus::Administration, 5000);
        assert_eq!(y, 5000 * 8 - 10000);   // 30_000
    }

    #[test]
    fn sibling_adjust_high_rep_recv_uses_10x_multiplier() {
        // rep=8000, Recv, premium=0 → (l30-750)*10
        let x = sibling_adjust_contribution(
            Big3NationMembership::Yes, 8000, 0, ClubFinanceStatus::Receivership, 5000);
        assert_eq!(x, (5000 - 750) * 10);  // 42_500

        // premium=1 → (l30-1250)*10
        let y = sibling_adjust_contribution(
            Big3NationMembership::Yes, 8000, 1, ClubFinanceStatus::Receivership, 5000);
        assert_eq!(y, (5000 - 1250) * 10); // 37_500
    }

    #[test]
    fn sibling_adjust_premium_bit_switches_subtrahend() {
        // premium=0 uses 750, premium=1+ uses 1250 for all non-Admin paths
        let n = sibling_adjust_contribution(
            Big3NationMembership::Yes, 6000, 0, ClubFinanceStatus::Normal, 3000);
        assert_eq!(n, (3000 - 750) * 6);   // 13_500
        let p = sibling_adjust_contribution(
            Big3NationMembership::Yes, 6000, 42, ClubFinanceStatus::Normal, 3000);
        assert_eq!(p, (3000 - 1250) * 6);  // 10_500
    }

    #[test]
    fn role_cap_lookup_matches_exe_switch() {
        assert_eq!(role_wage_cap_for(5),  RoleWageCapKind::HardCap(35_000));
        assert_eq!(role_wage_cap_for(6),  RoleWageCapKind::HardCap(35_000));
        assert_eq!(role_wage_cap_for(7),  RoleWageCapKind::HardCap(35_000));
        assert_eq!(role_wage_cap_for(8),  RoleWageCapKind::HardCap(20_000));
        assert_eq!(role_wage_cap_for(9),  RoleWageCapKind::HardCap( 1_500));
        assert_eq!(role_wage_cap_for(10), RoleWageCapKind::MinClamp(1_000));
        assert_eq!(role_wage_cap_for(4),  RoleWageCapKind::MinClamp(  750));
        assert_eq!(role_wage_cap_for(11), RoleWageCapKind::MinClamp(  750));
        assert_eq!(role_wage_cap_for(255),RoleWageCapKind::MinClamp(  750));
    }

    #[test]
    fn counter_party_base_wage_at_own_club_uses_current_wage() {
        let staff = CounterPartyStaff {
            role_byte: 5, current_club_id: 42, has_agent: true, is_player: false,
        };
        // At own club: local_2c starts at current_wage, capped at club*2
        let wage = counter_party_base_wage(staff, true, 12_000, 10_000);
        // min(12_000, 20_000) = 12_000, then max with 10_000 = 12_000, then role_cap 35_000 (no bind)
        assert_eq!(wage, 12_000);

        // Current wage exceeds club×2 cap
        let wage2 = counter_party_base_wage(staff, true, 100_000, 10_000);
        // min(100_000, 20_000) = 20_000, max with 10_000 = 20_000, role_cap doesn't bind
        assert_eq!(wage2, 20_000);
    }

    #[test]
    fn counter_party_base_wage_not_at_club_uses_only_floor() {
        let staff = CounterPartyStaff {
            role_byte: 5, current_club_id: 99, has_agent: true, is_player: false,
        };
        // Not at own club (at_this_club=false): local_2c starts at 0
        // Then max with club_wage_field
        let wage = counter_party_base_wage(staff, false, 12_000, 10_000);
        assert_eq!(wage, 10_000);
    }

    #[test]
    fn counter_party_base_wage_no_agent_skips_role_cap() {
        let staff = CounterPartyStaff {
            role_byte: 9,  // would cap at 1_500 if agent present
            current_club_id: 42, has_agent: false, is_player: false,
        };
        let wage = counter_party_base_wage(staff, true, 50_000, 25_000);
        // min(50_000, 50_000) = 50_000, max with 25_000 = 50_000
        // no_agent → skip role cap → 50_000 stands
        assert_eq!(wage, 50_000);
    }

    #[test]
    fn counter_party_base_wage_player_skips_role_cap() {
        // is_player=true also short-circuits the role switch
        let staff = CounterPartyStaff {
            role_byte: 10, current_club_id: 42, has_agent: true, is_player: true,
        };
        let wage = counter_party_base_wage(staff, true, 50_000, 25_000);
        // Role 10 would else apply MinClamp(1_000), but is_player → skip
        assert_eq!(wage, 50_000);
    }

    #[test]
    fn counter_party_base_wage_role_9_hard_caps_at_1500() {
        let staff = CounterPartyStaff {
            role_byte: 9, current_club_id: 42, has_agent: true, is_player: false,
        };
        let wage = counter_party_base_wage(staff, true, 50_000, 25_000);
        // min(50_000, 50_000) = 50_000, max with 25_000 = 50_000, then role 9 caps at 1_500
        assert_eq!(wage, 1_500);
    }

    #[test]
    fn counter_party_base_wage_role_default_caps_at_750() {
        let staff = CounterPartyStaff {
            role_byte: 15, current_club_id: 42, has_agent: true, is_player: false,
        };
        let wage = counter_party_base_wage(staff, true, 50_000, 25_000);
        // default MinClamp(750)
        assert_eq!(wage, 750);
    }

    #[test]
    fn wage_floor_table_matches_extracted_i64_values() {
        // Both .rdata addresses hold identical [250, 250, 300, 450, 600, 700, 800, 1000]
        // Verified via pefile — see commit note.
        assert_eq!(WAGE_FLOOR_TABLE,
            [250i64, 250, 300, 450, 600, 700, 800, 1000]);
        // Structure: 8 entries, first two equal (250-250), monotonic non-decreasing.
        assert_eq!(WAGE_FLOOR_TABLE[0], WAGE_FLOOR_TABLE[1]);
        for i in 1..WAGE_FLOOR_TABLE.len() {
            assert!(WAGE_FLOOR_TABLE[i-1] <= WAGE_FLOOR_TABLE[i]);
        }
    }

    #[test]
    fn wage_finance_status_scale_matches_exe() {
        use ClubFinanceStatus::*;
        assert_eq!(scale_wage_by_finance_status(10_000, Normal),        10_000);
        assert_eq!(scale_wage_by_finance_status(10_000, Administration), 10_500);
        assert_eq!(scale_wage_by_finance_status(10_000, Receivership),  11_000);

        // f64 → i32 truncation matches __ftol (toward zero)
        assert_eq!(scale_wage_by_finance_status(1_001, Administration), 1_051);  // 1051.05 → 1051
        assert_eq!(scale_wage_by_finance_status(1_001, Receivership),   1_101);  // 1101.1 → 1101

        // Constants are the exact .rdata values
        assert!((FINANCE_STATUS_COUNTERPARTY_MULT_ADMIN - 1.05).abs() < 1e-12);
        assert!((FINANCE_STATUS_COUNTERPARTY_MULT_RECV  - 1.10).abs() < 1e-12);
        assert!((FINANCE_TOP_LEAGUE_HIGH_REP_MULT      - 1.025).abs() < 1e-12);
    }

    #[test]
    fn sibling_floor_base_only_uses_2_5_multiplier() {
        // sibling_rep=100, local_8=1.0 → 100³ × 1.0 × 2.5 = 2_500_000
        let floor = sibling_club_wage_floor(100, None, 1.0);
        assert_eq!(floor, 2_500_000);

        // With smaller local_8 scaling
        let floor2 = sibling_club_wage_floor(100, None, 0.1);
        assert_eq!(floor2, 250_000);

        // Parent exists but rep <= sibling → falls through to base-only
        let floor3 = sibling_club_wage_floor(100, Some(80), 1.0);
        assert_eq!(floor3, 2_500_000);
    }

    #[test]
    fn sibling_floor_two_side_sums_1_25_contributions() {
        // sibling=100, parent=200 (parent > sibling), local_8=1.0
        // sibling_wage = 100³ × 1.0 × 1.25 = 1_250_000
        // parent_wage  = 200³ × 1.0 × -1.25 = -10_000_000 (as i32)
        // floor = 1_250_000 - (-10_000_000) = 11_250_000
        let floor = sibling_club_wage_floor(100, Some(200), 1.0);
        assert_eq!(floor, 11_250_000);
    }

    #[test]
    fn sibling_floor_clamps_to_200() {
        // Tiny inputs → floor at 200
        let floor = sibling_club_wage_floor(1, None, 0.0001);
        assert_eq!(floor, 200);
        // Even negative garbage → 200
        let floor2 = sibling_club_wage_floor(0, None, -1.0);
        assert_eq!(floor2, 200);
    }

    #[test]
    fn agent_multiplier_top_league_uses_group_factor() {
        // league_strength = 1 → base=1, top-league branch fires.
        // base scale: world_rank / 20. For world_rank=20, base=1.0.
        let base_scale = agent_wage_multiplier(
            100, 1, 20, AgentNationGroup::Top);
        // Top group has multiplier 1.0, no rebase (world_rank*10 = 200 ≥ band 100)
        assert!((base_scale - 1.0).abs() < 1e-9, "Top: {}", base_scale);

        // Brazil group: 0.75 flat, no rebase branch
        let brazil = agent_wage_multiplier(100, 1, 20, AgentNationGroup::Brazil);
        assert!((brazil - 0.75).abs() < 1e-9, "Brazil: {}", brazil);

        // Big group at same inputs: 0.65 (no rebase, world_rank*10=200 ≥ band=100)
        let big = agent_wage_multiplier(100, 1, 20, AgentNationGroup::Big);
        assert!((big - 0.65).abs() < 1e-9, "Big: {}", big);
    }

    #[test]
    fn agent_multiplier_lower_leagues_skip_group_factor() {
        // league_strength = 2 → base=2, top-league branch NOT taken.
        // Only base scale applies: world_rank / (2*20) = world_rank / 40.
        // Group multiplier irrelevant.
        let x = agent_wage_multiplier(100, 2, 20, AgentNationGroup::Top);
        assert!((x - 0.5).abs() < 1e-9, "got {}", x);
        // Different group, same result — group is ignored when base != 1
        let y = agent_wage_multiplier(100, 2, 20, AgentNationGroup::Default);
        assert!((x - y).abs() < 1e-12);
    }

    #[test]
    fn agent_multiplier_rebase_triggers_when_world_rank_small() {
        // world_rank=5, band=200 → world_rank*10 = 50 < 200 (rebase fires)
        // cand = (band/max(rank,1)) * 0.1 * mult = (200/5) * 0.1 * mult = 4.0 * mult
        // For Big (mult=0.65): cand = 4.0 * 0.65 = 2.6 > 1.0 → snap to 1.0
        let big = agent_wage_multiplier(200, 1, 5, AgentNationGroup::Big);
        // local_8 = 5/20 * 1.0 = 0.25
        assert!((big - 0.25).abs() < 1e-9, "Big rebase snap: {}", big);
    }

    #[test]
    fn agent_multiplier_small_group_special_cap_at_075() {
        // For Small group: cand = 4.0 * 0.35 = 1.4 > 1.0 → snap to 1.0.
        // At smaller ratios cand can dip; check the 0.75 cap path.
        // world_rank=10, band=15 → world_rank*10 = 100 > band 15 (NO rebase)
        // Just flat multiplier 0.35.
        let no_rebase = agent_wage_multiplier(
            15, 1, 10, AgentNationGroup::Small);
        // local_8 = 10/20 * 0.35 = 0.175
        assert!((no_rebase - 0.175).abs() < 1e-9);

        // Now with rebase but cand small enough to hit the 0.75 cap:
        // band=110, world_rank=10 → 100 < 110 (rebase fires)
        // cand = (110/10) * 0.1 * 0.35 = 11 * 0.035 = 0.385
        // 0.385 <= 0.75 → mult snaps to 0.75
        let capped = agent_wage_multiplier(
            110, 1, 10, AgentNationGroup::Small);
        // local_8 = 10/20 * 0.75 = 0.375
        assert!((capped - 0.375).abs() < 1e-9, "Small cap: {}", capped);
    }

    #[test]
    fn agent_multiplier_league_strength_clamps_to_1_3() {
        // league_strength=0 or negative → clamped to 1 (top-league branch)
        let clamped_low = agent_wage_multiplier(100, 0, 20, AgentNationGroup::Top);
        let base_1     = agent_wage_multiplier(100, 1, 20, AgentNationGroup::Top);
        assert!((clamped_low - base_1).abs() < 1e-12);
        // league_strength=5 → clamped to 3, base scale = rank/60
        let clamped_hi = agent_wage_multiplier(100, 5, 60, AgentNationGroup::Top);
        let base_3     = agent_wage_multiplier(100, 3, 60, AgentNationGroup::Top);
        assert!((clamped_hi - base_3).abs() < 1e-12);
    }

    #[test]
    fn recall_gate_blocks_between_18aug_and_15nov() {
        // Before pre-season cutoff (July, early August) → allowed
        assert_eq!(can_recall_loan((2001, 7, 30)),  RecallVerdict::Allowed);
        assert_eq!(can_recall_loan((2001, 8, 17)),  RecallVerdict::Allowed);
        // On/after 18-Aug and before 15-Nov → blocked
        assert_eq!(can_recall_loan((2001, 8, 18)),  RecallVerdict::BlockedUntil15Nov);
        assert_eq!(can_recall_loan((2001, 9, 15)),  RecallVerdict::BlockedUntil15Nov);
        assert_eq!(can_recall_loan((2001, 10, 31)), RecallVerdict::BlockedUntil15Nov);
        assert_eq!(can_recall_loan((2001, 11, 14)), RecallVerdict::BlockedUntil15Nov);
        // On/after 15-Nov → allowed
        assert_eq!(can_recall_loan((2001, 11, 15)), RecallVerdict::Allowed);
        assert_eq!(can_recall_loan((2001, 12, 25)), RecallVerdict::Allowed);
        assert_eq!(can_recall_loan((2002, 1, 5)),   RecallVerdict::Allowed);
        // Constants reflect the corrected 18-Aug (was mistakenly 18-Jul).
        assert_eq!(RECALL_PRE_SEASON_DAY, (18, 8));
        assert_eq!(RECALL_MID_SEASON_DAY, (15, 11));
    }

    #[test]
    fn composer_default_probe_mode_returns_placeholder_length() {
        let player = cp(100, 110, 5000, 500_000, 5_000, 24);
        let club = cc(5000);
        let off = compose_wage_offer(player, club, None, SquadStatus::FirstTeam, 0xff, 1).unwrap();
        // mode 0xff = the default-probe used by asking-price rendering.
        assert!(off.contract_years >= 1);
        assert_eq!(off.tier_byte, 0xff);
    }
}
