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
    // -- (1) 3-tier reputation cascade — `FUN_004d3ea0`:reads club rep and
    //    picks a tier-scale via 4 bands. Reproduced verbatim.
    let rep_scale: f64 = if club.reputation < REP_TIER_A      { 0.35 }  // small club
                         else if club.reputation < REP_TIER_B { 0.60 }  // mid club
                         else if club.reputation < REP_TIER_C { 0.85 }  // big club
                         else                                 { 1.00 }; // top club

    // -- (2) Sentinel gate — the `_DAT_009569a0` "declined" test at
    //    0x0084a35b. In asm, this is a double-fcomp on the fitted wage; when
    //    the max of the 4-way rep-mult falls below the sentinel, callers see
    //    no offer. Ported as: if player is >2 tiers above the club's rep
    //    band, reject. Cross-checked vs `FUN_008d2d20`:141 which also refuses
    //    unless player-tier & 0x3f is in {1,3,5}.
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
    // Club-rep tier delta layered on top — the real exe reaches the same
    // effect via `FUN_00580a90` (affordable-wage cap) applied AFTER the
    // formula. Kept as a MIN clamp so a mid-club can't out-bid its band.
    let cap = ((weekly as f64) * rep_scale) as u32;
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
