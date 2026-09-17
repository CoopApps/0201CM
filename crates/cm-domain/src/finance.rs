//! Club finance substrate — balance, wages, transfer budget.
//!
//! Consumed by every country's `*_rules.cpp` file at new-game time to
//! declare the ceilings and defaults for that country's domestic economy
//! (contract lengths, wage caps, transfer window rules, foreigner limits).
//!
//! # Scope
//!
//! This is a MINIMAL working substrate — enough to unblock the country
//! rules TUs and let the tick's monthly-balance / weekly-wage-payment
//! hooks actually mutate club money. It is NOT a faithful port of the
//! exe's transfer-market / bidding / contract-negotiation engine — those
//! are separate TUs (`transfer_manager.cpp`, `contract_manager.cpp`,
//! `staff_contracts.cpp`) still ahead in the walk.
//!
//! # Fields
//!
//! * `balance` — the club's cash reserve (starts from club_comp record's
//!   own balance field once decoded; currently seeded to a plausible
//!   default proportional to reputation).
//! * `weekly_wage_bill` — this week's actual cash outflow. NOT a sum of
//!   individual contract wages; the exe (FUN_00586ec0:363-422) uses a
//!   reputation-scaled formula with cash-tier gating + chairman-satisfaction
//!   adjustment + RNG jitter. Per-contract wage at contract+0xc is only
//!   read by UI/negotiation surfaces, never aggregated for cash flow.
//!   See reports/weekly_wage_bill_decode.md.
//! * `transfer_budget` — cash the manager can commit to signings this
//!   window.
//! * `salary_ceiling` — per-country wage cap (0 = no cap).
//! * `max_foreigners` — non-domestic players allowed in the matchday
//!   squad (0 = no limit).

use serde::{Deserialize, Serialize};

/// Per-country rules declared by that country's `*_rules.cpp`. Passed to
/// [`CountryFinanceRules::register`] at new-game time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CountryRulesSpec {
    /// Nation id (from rust-db `nation_competitions`).
    pub nation_id: i32,
    /// Maximum contract length in years the country's leagues allow.
    /// Real values: 5 (most), 3 (Italy for foreigners pre-Bosman),
    /// 7 (rare — Argentine allowed longer historically).
    pub max_contract_years: u8,
    /// Weekly wage ceiling in local units (0 = no ceiling).
    pub salary_ceiling_weekly: u32,
    /// Foreign-player slots per matchday squad (0 = unlimited).
    pub max_foreigners_matchday: u8,
    /// Transfer-window month opens (1..12). Convention: primary window opens
    /// month, closes ~1 month later. Real: European Aug-Sep, S.American
    /// Jul-Aug, Japanese Feb-Mar.
    pub transfer_window_open_month: u8,
    /// Youth-team recruitment allowed (some leagues restricted).
    pub youth_recruitment: bool,
}

/// Country-level finance rules keyed by nation id.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CountryFinanceRules {
    pub countries: Vec<CountryRulesSpec>,
}

impl CountryFinanceRules {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a country's rules. Duplicate registrations for the same
    /// nation_id overwrite — the last `*_rules.cpp` port to run wins.
    pub fn register(&mut self, spec: CountryRulesSpec) {
        self.countries.retain(|c| c.nation_id != spec.nation_id);
        self.countries.push(spec);
    }

    /// Look up the rules for a nation.
    pub fn for_nation(&self, nation_id: i32) -> Option<CountryRulesSpec> {
        self.countries.iter().find(|c| c.nation_id == nation_id).copied()
    }
}

/// Per-club finance state, keyed by club id.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClubFinance {
    pub club_id: u32,
    /// Cash in the bank. Can go negative — the tick tracks how many months
    /// a club has been in the red for potential administration.
    pub balance: i64,
    /// Weekly wage bill — sum of every contracted staff member's weekly
    /// wage. Currently a proxy: 100 units × squad reputation cluster.
    pub weekly_wage_bill: u32,
    /// Cash the manager can commit to signings this window.
    pub transfer_budget: i64,
    /// Number of consecutive months the balance has been negative.
    pub months_in_the_red: u8,
    /// Board confidence (finance record `+0x166`). Decrements weekly on poor
    /// results; hits 51 (`0x33`) → sacking threshold (`FUN_00588c70`:177).
    /// 15/10/5 initial by reputation tier.
    #[serde(default)]
    pub board_confidence: u8,
    /// **In administration** (formal state, finance record `+0x165 == 1` in the
    /// exe). Triggered when `status() == Admin`. Side-effects: wages cut,
    /// stadium projects halted, players from this club lose the ability to
    /// refuse transfer bids. Cleared when balance climbs back above the admin
    /// threshold. (Kill #8/9 refinement — `FUN_00588c70`.)
    #[serde(default)]
    pub in_administration: bool,
    /// This-month accumulators for the monthly rollover ledger. Reset to 0 at
    /// month end (they mirror the exe's this-month block at finance +0x110).
    #[serde(default)]
    pub month_wages: i64,
    #[serde(default)]
    pub month_gate: i64,
    #[serde(default)]
    pub month_tv_prize: i64,
    /// Season misc operating expense — runtime finance record `+0x8C`
    /// (i32). Reset annually by the season roll (`FUN_00585AE0`).
    /// Written by the C14 stadium-expansion transaction (C15.1A) via
    /// [`FinanceBook::apply_year_end_write`]. Moved here from the former
    /// `ClubFinanceLedger` so the port has ONE runtime finance store.
    #[serde(default)]
    pub season_misc_expense: i32,
    /// Lifetime misc operating expense — `+0x12C` (i32). Never reset.
    #[serde(default)]
    pub lifetime_misc_expense: i32,
    /// Season subsidy income — `+0xB4` (i32). Reset annually.
    #[serde(default)]
    pub season_subsidy_income: i32,
    /// Lifetime subsidy income — `+0x154` (i32). Never reset.
    #[serde(default)]
    pub lifetime_subsidy_income: i32,
    /// Home stadium id (club record +0x69 in the exe). Clubs that share a
    /// stadium (Bayern & 1860 München at Olympiastadion; Alemannia Aachen &
    /// its reserves; many reserve/first-team pairs worldwide) share this
    /// value → they are the ones eligible for the £20M cross-club transfer
    /// event in `FUN_00586ec0`:56-114. Populated at seed time from
    /// `ClubView::home_stadium_id()`. `None` when unset.
    #[serde(default)]
    pub home_stadium_id: Option<i32>,
    /// Stadium-share transfer already fired this year? (`param_2+0x6d` in the
    /// exe.) Cleared once per game-year in `reset_yearly_stadium_flags`; set
    /// by `stadium_share_transfers` when a £20M transfer completes.
    #[serde(default)]
    pub stadium_share_used: bool,
    /// Takeover latch (`+0x82` in the exe). Set when a new-board takeover
    /// (`FUN_005884a0`) fires; prevents re-firing until reset. Also read by
    /// FUN_00588c70 to short-circuit repeat handouts.
    #[serde(default)]
    pub takeover_pending: bool,
    /// Ledger of the most recent takeover injection (for news generation).
    #[serde(default)]
    pub last_takeover_amount: i64,
    /// Month/year ledger of chairman gifts (`puVar+0x2d`, `puVar+0x55` in the
    /// exe — offsets 0xb4 and 0x154 on the ClubFinance record). Accumulates
    /// takeovers + board debt payments. Reset by month/year rollovers.
    #[serde(default)]
    pub month_owner_gift: i64,
    #[serde(default)]
    pub year_owner_gift: i64,
}

/// Starting-cash lookup table extracted from the exe at VA 0x009b48e0
/// (`FUN_005803d0`:58-60). Indexed by `clamp(reputation/500 - 1, 0, 15)`.
/// Values are in pounds and range from a £450k debt for rep-500 minnows to
/// £10M for rep-8000+ giants. This is the actual game data, not a heuristic.
pub const START_CASH: [i64; 16] = [
    -450_000, -375_000, -125_000, 0,
    150_000, 350_000, 750_000, 1_000_000,
    1_500_000, 3_000_000, 5_000_000, 6_000_000,
    7_000_000, 8_000_000, 9_000_000, 10_000_000,
];

/// Board-confidence classification — real thresholds from `FUN_00582870`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinanceStatus {
    Admin,       // 0xFE (-2): balance ≤ -(1500·rep + 2.5M)
    InTheRed,    // 0xFF (-1): rep<5000 && bal ≤ -250·rep; rep≥5000 && bal ≤ (3750-rep)·1000
    Normal,      // 0
    Healthy,     // 1: balance ≥ (25·rep + 87500)·40
    Rich,        // 2: balance ≥ (25·rep + 50000)·100
}

impl FinanceStatus {
    /// Signed byte the exe's `FUN_00582870` classifier returns
    /// (-2..=2). Used by the initial-board-confidence cascade at
    /// `FUN_005803d0`:101-110.
    pub fn to_signed_byte(self) -> i8 {
        match self {
            FinanceStatus::Admin    => -2,
            FinanceStatus::InTheRed => -1,
            FinanceStatus::Normal   =>  0,
            FinanceStatus::Healthy  =>  1,
            FinanceStatus::Rich     =>  2,
        }
    }
}

impl ClubFinance {
    /// Seed a new club's finance. The REAL data uses the SHIPPED `club_cash`
    /// value at club record +0x65 when it's non-zero (287 clubs of 10,580 ship
    /// with a specific balance including all the bankrupt clubs like Sheffield
    /// Wednesday at −£14M and top clubs like Real Madrid at +£100M). For clubs
    /// that ship with cash=0, we fall back to the exe's `FUN_005803d0`
    /// START_CASH table (verified extract from VA 0x009b48e0) which seeds by
    /// reputation band.
    ///
    /// Transfer budget: still generated from the rep/status cascade
    /// (`005803d0.c`:195-238) since the shipped value overlay isn't at a
    /// decoded offset yet.
    pub fn seed_faithful(club_id: u32, reputation: u16, stadium_flag: bool) -> Self {
        // Assumes has_chairman=true — matches the shipped data where every
        // real club carries a chairman staff record. Test-only convenience.
        Self::seed_from(club_id, reputation, stadium_flag, 0, true)
    }

    /// Seed variant that takes the CLUB-SPECIFIC shipped cash from club.dat's
    /// `+0x65` field. If non-zero, that value wins; otherwise the reputation
    /// table is used as fallback (matches the exe's own behaviour for clubs
    /// with no pre-set finance state).
    pub fn seed_from(club_id: u32, reputation: u16, stadium_flag: bool, shipped_cash: i32, has_chairman: bool) -> Self {
        let rep = reputation as i64;
        let band = ((rep / 500).saturating_sub(1)).clamp(0, 15) as usize;
        let balance: i64 = if shipped_cash != 0 {
            shipped_cash as i64
        } else {
            START_CASH[band]
        };

        // Transfer budget cascade — only clubs with rep ≥ 4751 in a top status
        // get a positive budget; the mass of lower-league clubs get 0.
        // Simplified from :195-238: we don't yet have the promoted/relegated
        // status wire-through, so treat every eligible club as status=0.
        let transfer_budget: i64 = if rep < 0x128F {
            0
        } else {
            let rep_adj = if rep > 0x1676 { 2 * rep - 0x1676 } else { rep };
            let cost = if stadium_flag { 1_250 } else { 750 };
            if rep_adj < 0x1676 {
                (rep_adj - cost) * 6
            } else {
                (rep_adj - cost) * 10
            }.max(0)
        };

        // Compute status once against the seed balance so board_confidence
        // and boots_in_admin agree with FUN_00582870.
        let status_at_seed = {
            let r = reputation as i64;
            let bal = balance;
            if bal >= (25 * r + 50_000) * 100 { FinanceStatus::Rich }
            else if bal >= (25 * r + 87_500) * 40 { FinanceStatus::Healthy }
            else if bal <= -(1500 * r + 2_500_000) { FinanceStatus::Admin }
            else if r < 5000 && bal <= -250 * r { FinanceStatus::InTheRed }
            else if r >= 5000 && bal <= (3750 - r) * 1000 { FinanceStatus::InTheRed }
            else { FinanceStatus::Normal }
        };
        let boots_in_admin = matches!(status_at_seed, FinanceStatus::Admin);
        // Wage bill starts at 0 — the tick sums real contracts weekly. (The
        // old rep²/5 heuristic invented a wage bill uncorrelated with the
        // actual signed players.)
        Self {
            club_id,
            balance,
            weekly_wage_bill: 0,
            transfer_budget,
            in_administration: boots_in_admin,
            months_in_the_red: 0,
            board_confidence: initial_board_confidence(has_chairman, status_at_seed),
            month_wages: 0,
            month_gate: 0,
            month_tv_prize: 0,
            home_stadium_id: None,
            stadium_share_used: false,
            takeover_pending: false,
            last_takeover_amount: 0,
            month_owner_gift: 0,
            year_owner_gift: 0,
            // Year-end accumulators (+0x8C / +0x12C / +0xB4 / +0x154):
            // the exe zero-inits the whole 0x167 record before the
            // cash seed lands (FUN_0093543F ctor loop), so these start
            // at 0 regardless of the START_CASH path taken above.
            season_misc_expense: 0,
            lifetime_misc_expense: 0,
            season_subsidy_income: 0,
            lifetime_subsidy_income: 0,
        }
    }

    /// Real-formula status classification (`FUN_00582870`).
    pub fn status(&self, reputation: u16) -> FinanceStatus {
        let r = reputation as i64;
        let bal = self.balance;
        if bal >= (25 * r + 50_000) * 100 { FinanceStatus::Rich }
        else if bal >= (25 * r + 87_500) * 40 { FinanceStatus::Healthy }
        else if bal <= -(1500 * r + 2_500_000) { FinanceStatus::Admin }
        else if r < 5000 && bal <= -250 * r { FinanceStatus::InTheRed }
        else if r >= 5000 && bal <= (3750 - r) * 1000 { FinanceStatus::InTheRed }
        else { FinanceStatus::Normal }
    }
}

/// Real `FUN_005803d0`:101-110 seed for board confidence at finance+0x166.
/// `has_chairman` comes from club-record +0x6d; `status` is the signed byte
/// returned by `FUN_00582870`. VERIFIED via decode agent 2026-08-31.
///
/// Cascade (exact):
///   no chairman OR status == Normal (0)  → 10
///   status ∈ {Healthy=1, Rich=2}          → 5
///   else (InTheRed=-1, Admin=-2)          → 15
///
/// The old rep-tier heuristic was backwards — the exe rewards *good*
/// status (Healthy/Rich) with LOW confidence numbers (5) and penalises
/// *very bad or very good* with 15. Reputation is not read.
fn initial_board_confidence(has_chairman: bool, status: FinanceStatus) -> u8 {
    let s = status.to_signed_byte();
    if !has_chairman || s == 0 { 10 }
    else if (1..=2).contains(&s) { 5 }
    else { 15 }
}

/// Verified chairman-personality gates from
/// [`reports/chairman_gates_decode.md`]. Each byte lives on the
/// chairman-STAFF record at the cited offset; all four are `u8` in 0..=20
/// (rerolled as `rand(20)+1` in `FUN_00588840`).
///
/// This is the state the AI transfer/finance path reads to decide whether
/// the chairman approves a bid, injects funds, or sacks the manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChairmanState {
    /// `chairman.staff[+0x0f]` — AMBITION. Gates the wage-cap uplift
    /// (00580a90.c:477) and cash-injection approval roll (00587f50.c:48).
    pub ambition: u8,
    /// `chairman.staff[+0x16]` — TAKEOVER patience. `rand(x)==0` fires
    /// silent takeover reroll (00588840.c:51).
    pub takeover_patience: u8,
    /// `chairman.staff[+0x1d]` — MANAGER-SACK patience.
    /// `rand(x) <= 4 && rand(20) != 0` fires sack msg (0067fdf0.c:243).
    pub manager_patience: u8,
    /// `chairman.staff[+0x20]` — GENEROSITY / wealth pool. Cap on injection
    /// amount (× £1M), on transfer approval (× £500k). Decremented on use,
    /// floor 5. Cites: 00583fc0.c:123; 00587f50.c:54,63,121.
    pub generosity: u8,
    /// `chairman[+0x57]` — wealth tier, `>15` unlocks wage-cap top branch
    /// (00580a90.c:482). Not decremented.
    pub wealth_tier: u8,
}

impl Default for ChairmanState {
    /// Neutral default when no chairman record is loaded — matches the
    /// `has_chairman=false` fallback used elsewhere in the finance path.
    fn default() -> Self {
        ChairmanState { ambition: 10, takeover_patience: 10,
                        manager_patience: 10, generosity: 10, wealth_tier: 10 }
    }
}

/// Verified port of `FUN_00583fc0`:122-127 — transfer-approval gate.
/// Chairman REJECTS an overrun bid when `amount > generosity × £500,000`;
/// on refusal, generosity is incremented (capped 20). Returns `true` iff
/// the bid is approved.
///
/// `would_go_negative` mirrors the exe's finance-status check that gates
/// the whole path; when the club can pay from cash on hand, no chairman
/// approval is needed.
pub fn chairman_approves_overrun(state: &mut ChairmanState,
                                  amount_gbp: i64,
                                  would_go_negative: bool) -> bool {
    if !would_go_negative { return true; }
    if (state.generosity as i64) * 500_000 <= amount_gbp {
        if state.generosity < 20 { state.generosity += 1; }
        return false;
    }
    true
}

/// Verified port of `FUN_00587f50`:54,63,121 — chairman cash-injection.
/// Returns `Some(amount)` when the chairman injects, `None` when the
/// date-gate blocks. Amount is `rand(generosity × £1_000_000)`. On success,
/// generosity is decremented by 1, floor 5.
///
/// `rng_upto(n)` returns a value in `[0, n)`.
pub fn chairman_cash_inject_cap(state: &mut ChairmanState,
                                current_date_short: i16,
                                rng_upto: impl FnOnce(i32) -> i32) -> Option<i32> {
    // 00587f50.c:63 — `generosity*750 <= date` skips the injection.
    if (state.generosity as i32) * 750 <= current_date_short as i32 {
        return None;
    }
    let cap = (state.generosity as i32) * 1_000_000;
    if cap <= 0 { return None; }
    let amount = rng_upto(cap);
    // 00587f50.c:121-123 — decrement post-inject, floor 5.
    if state.generosity > 5 { state.generosity -= 1; }
    Some(amount)
}

/// Verified port of `FUN_0067fdf0`:243 — chairman sack decision.
/// Sack triggers when `rand(manager_patience) <= 4` AND the 1-in-20
/// floor also fires. Both rolls come from the caller so the same RNG
/// stream is preserved.
pub fn chairman_will_sack(state: &ChairmanState,
                          rand_mod_patience: i32,
                          rand_mod_20: i32) -> bool {
    let _ = state; // read for readability at call sites; roll pre-computed
    rand_mod_patience <= 4 && rand_mod_20 != 0
}

/// Verified port of `FUN_00588840`:51 — silent takeover trigger.
/// Fires when `rand(takeover_patience) == 0`.
pub fn chairman_takeover_fires(state: &ChairmanState,
                                rand_mod_patience: i32) -> bool {
    let _ = state;
    rand_mod_patience == 0
}

/// Verified port of `FUN_00588840`:83-93 — post-takeover chairman-stat
/// reroll. All four bytes go to `rand(20)+1`, with generosity re-rolled
/// once if it lands below 5. Mutates `state` in place.
pub fn reroll_chairman_stats(state: &mut ChairmanState,
                             mut rand20: impl FnMut() -> i32) {
    state.manager_patience  = (rand20() + 1) as u8;
    state.ambition          = (rand20() + 1) as u8;
    state.takeover_patience = (rand20() + 1) as u8;
    let mut ge = (rand20() + 1) as u8;
    if ge < 5 { ge = (rand20() + 1) as u8; }
    state.generosity = ge;
}

/// The finance book — indexed by club id.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FinanceBook {
    pub clubs: Vec<ClubFinance>,
    pub rules: CountryFinanceRules,
    /// club_id → reputation (ClubView::reputation, ×500) — needed by every
    /// per-club tick (wages, status, income) since those all key off it.
    #[serde(default)]
    pub club_reputation: std::collections::BTreeMap<u32, u16>,
    /// club_id → (average, minimum, maximum) attendance — from the shipped
    /// club record at +0x73/+0x77/+0x7b. Fed into gate-receipt calc so a
    /// packed Old Trafford (67k) makes more from a match than Rushden's
    /// 4k-capacity home; this replaces the pure-reputation gate proxy.
    #[serde(default)]
    pub club_attendance: std::collections::BTreeMap<u32, (i32, i32, i32)>,
    /// club_id → nation_id (from `ClubView::nation_id`). Needed by the
    /// takeover-check exempt-nation filter (Wales / ROI / NI are
    /// hardcoded out of `FUN_005884a0`).
    #[serde(default)]
    pub club_nation: std::collections::BTreeMap<u32, i32>,
    /// club_id → has-chairman flag (ClubView::flag_6d() != 0). Item 1
    /// VERIFIED — the exe reads this byte at `005803d0.c:102`,
    /// `00586ec0.c:59/117/118`. Consumed by initial_board_confidence
    /// and by the debt-payment gate (Item 3).
    #[serde(default)]
    pub club_has_chairman: std::collections::BTreeMap<u32, bool>,
    /// club_id → ChairmanState. Populated lazily on first monthly-board
    /// tick from the shipped chairman-staff bytes at +0x0f/+0x16/+0x1d/+0x20.
    /// See [`ChairmanState`] + [`reports/chairman_gates_decode.md`].
    #[serde(default)]
    pub chairman: std::collections::BTreeMap<u32, ChairmanState>,
    /// club_id → board patience byte (`club+0x7f`), 0..=20. Decremented in
    /// the monthly board tick per `FUN_00588c70`:210-243. Reaching 0 arms
    /// the manager-sack roll (see [`chairman_will_sack`]).
    #[serde(default)]
    pub board_patience: std::collections::BTreeMap<u32, u8>,
}

impl FinanceBook {
    /// Monthly board tick — verified port of the patience-decrement +
    /// chairman-decision cascade from `FUN_00588c70`:210-243 and
    /// `FUN_0067fdf0`:243. Returns the list of clubs whose chairman
    /// FIRED the manager this month (caller wires the actual departure).
    ///
    /// For each club:
    ///   1. Look up (or default-init) ChairmanState + board_patience.
    ///   2. Decrement patience by the exe's stepped rule:
    ///      - `> 15` → subtract `rand(5)+1`
    ///      - `> 7`  → subtract 1
    ///      - `< 2`  → no change
    ///      - else   → `rand(patience) != 0 → subtract 1`
    ///   3. If patience is now 0, roll `chairman_will_sack`; on fire,
    ///      push club onto the return list.
    ///   4. Roll `chairman_takeover_fires`; on fire, reroll stats.
    pub fn tick_month_board(&mut self, seed: u64) -> Vec<u32> {
        use crate::match_engine_exe::MatchRng;
        let mut rng = MatchRng::new(seed);
        let club_ids: Vec<u32> = self.clubs.iter().map(|c| c.club_id).collect();
        let mut fired_by: Vec<u32> = Vec::new();
        // Chairman-presence read at raw offset +0xbf (ClubView::has_chairman).
        // ~25% of shipped clubs carry a chairman staff id; the rest are
        // chairman-less and skip the monthly board tick as the exe does.
        for cid in club_ids {
            let has_chairman = self.club_has_chairman.get(&cid).copied().unwrap_or(false);
            if !has_chairman { continue; }
            let cs = self.chairman.entry(cid).or_insert_with(ChairmanState::default);
            let patience = self.board_patience.entry(cid).or_insert(15u8);
            // 1) patience decrement (exe: FUN_00588c70:210-243)
            let p = *patience;
            let dec: u8 = if p > 15 { (rng.range(5) + 1) as u8 }
                          else if p > 7 { 1 }
                          else if p < 2 { 0 }
                          else if rng.range(p as u32) != 0 { 1 } else { 0 };
            *patience = patience.saturating_sub(dec);
            // 2) sack roll (exe: FUN_0067fdf0:243)
            if *patience == 0 {
                let rmp = rng.range(cs.manager_patience.max(1) as u32) as i32;
                let r20 = rng.range(20) as i32;
                if chairman_will_sack(cs, rmp, r20) {
                    fired_by.push(cid);
                    // Reset patience post-sack (exe implicitly resets on new hire).
                    *patience = 15;
                }
            }
            // 3) takeover roll (exe: FUN_00588840:51)
            let rtp = rng.range(cs.takeover_patience.max(1) as u32) as i32;
            if chairman_takeover_fires(cs, rtp) {
                reroll_chairman_stats(cs, || rng.range(20) as i32);
            }
            // 4) chairman cash injection — verified port of
            //    FUN_00587f50:54,63,121. Fires when the club's balance is
            //    negative (finance status In-The-Red / Admin), gated by the
            //    date-vs-generosity check inside chairman_cash_inject_cap.
            //    Amount is `rand(generosity × £1M)`; generosity decrements
            //    by 1 on success, floor 5.
            let club = match self.clubs.iter_mut().find(|c| c.club_id == cid) {
                Some(c) => c, None => continue,
            };
            let rep = self.club_reputation.get(&cid).copied().unwrap_or(1000);
            let status = club.status(rep);
            if matches!(status, FinanceStatus::InTheRed | FinanceStatus::Admin) {
                // Date-short: pack (year-1900)*365 + day-of-year — the exe
                // uses a compact 16-bit day counter from game start. Here
                // we use elapsed_days modulo the i16 range as an
                // approximation; the gate compares to generosity*750 so
                // the ordering is what matters.
                let date_short = (seed & 0x7FFF) as i16;
                let inject = chairman_cash_inject_cap(cs, date_short, |cap| {
                    rng.range(cap.max(1) as u32) as i32
                });
                if let Some(amt) = inject {
                    club.balance = club.balance.saturating_add(amt as i64);
                }
            }
        }
        fired_by
    }
}

impl FinanceBook {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mutable lookup by club id (linear, like [`Self::for_club`]).
    pub fn for_club_mut(&mut self, club_id: u32) -> Option<&mut ClubFinance> {
        self.clubs.iter_mut().find(|c| c.club_id == club_id)
    }

    /// The five fields the year-end pipeline (C14 stadium expansion,
    /// C15 rollover) reads/writes, as a value snapshot. `cash` is
    /// `balance` — the runtime record's `+0x00` i64. Absent club →
    /// all-zero (mirrors the old ledger's `get` default).
    ///
    /// This is THE runtime finance store: `balance` is what the weekly
    /// wage / match-income / board / debt ticks mutate, so year-end
    /// affordability sees the real, wage-depleted cash — the former
    /// `ClubFinanceLedger` field on the save never saw those and was
    /// never seeded in production (see ledger §12a).
    pub fn year_end_state(&self, club_id: u32) -> crate::c15_1_world_apply::ClubFinanceState {
        match self.for_club(club_id) {
            Some(c) => crate::c15_1_world_apply::ClubFinanceState {
                cash: c.balance,
                season_misc_expense: c.season_misc_expense,
                lifetime_misc_expense: c.lifetime_misc_expense,
                season_subsidy_income: c.season_subsidy_income,
                lifetime_subsidy_income: c.lifetime_subsidy_income,
            },
            None => crate::c15_1_world_apply::ClubFinanceState::default(),
        }
    }

    /// Snapshot [`Self::year_end_state`] for a set of clubs (used by
    /// `YearEndSnapshot::from_apply` for the clubs a rollover touched).
    pub fn year_end_states(
        &self,
        club_ids: impl IntoIterator<Item = u32>,
    ) -> std::collections::BTreeMap<u32, crate::c15_1_world_apply::ClubFinanceState> {
        club_ids.into_iter().map(|id| (id, self.year_end_state(id))).collect()
    }

    /// Apply one C14/C15.1A finance write. C14 has already computed the
    /// NEW post-transaction values; this stores them: `balance` ←
    /// `new_cash`, the four accumulators ← `new_*`. A club absent from
    /// the book (only possible in tests — boot seeds all 10,580) is
    /// created on demand so the write is never silently dropped.
    pub fn apply_year_end_write(&mut self, w: &crate::c15_1_world_apply::PendingFinanceWrite) {
        if self.for_club(w.club_id).is_none() {
            self.clubs.push(ClubFinance { club_id: w.club_id, ..Default::default() });
        }
        let c = self.for_club_mut(w.club_id).expect("just ensured present");
        c.balance = w.new_cash;
        c.season_misc_expense = w.new_season_misc_expense;
        c.lifetime_misc_expense = w.new_lifetime_misc_expense;
        c.season_subsidy_income = w.new_season_subsidy_income;
        c.lifetime_subsidy_income = w.new_lifetime_subsidy_income;
    }

    pub fn seed_from_clubs(clubs: &[crate::DomainOpaqueRecord]) -> Self {
        // Faithful seed (kill #8a): real START_CASH table + rep/status budget
        // cascade from FUN_005803d0. Records the club's reputation alongside
        // for later status classification.
        let mut cf = Vec::with_capacity(clubs.len());
        let mut reps = std::collections::BTreeMap::new();
        let mut att = std::collections::BTreeMap::new();
        let mut nats = std::collections::BTreeMap::new();
        let mut chair = std::collections::BTreeMap::new();
        for rec in clubs {
            let cv = crate::ClubView::new(rec);
            if let Some(nid) = cv.nation_id() { nats.insert(cv.id(), nid); }
            // Use SHIPPED cash from the club record (+0x65) when non-zero —
            // otherwise fall back to the START_CASH-by-reputation table. This
            // is what makes SWFC boot as bankrupt (their shipped cash is
            // -£14M), same as the game itself.
            // Real chairman presence: club record has a chairman staff id at
            // raw offset +0xbf (VERIFIED — was earlier misidentified as
            // flag_6d/+0x6d which reads 0 for every club). ~25% of shipped
            // clubs carry a chairman; the rest are chairman-less.
            let has_chair = cv.has_chairman();
            let mut cfinance = ClubFinance::seed_from(cv.id(), cv.reputation(), false, cv.initial_cash_seed(), has_chair);
            cfinance.home_stadium_id = cv.home_stadium_id();
            cf.push(cfinance);
            att.insert(cv.id(), (cv.attendance_average(), cv.attendance_minimum(), cv.attendance_maximum()));
            reps.insert(cv.id(), cv.reputation());
            chair.insert(cv.id(), has_chair);
        }
        Self { clubs: cf, rules: CountryFinanceRules::new(),
               club_has_chairman: chair,
               chairman: std::collections::BTreeMap::new(),
               board_patience: std::collections::BTreeMap::new(),
               club_reputation: reps, club_attendance: att, club_nation: nats }
    }

    /// **New board of directors assumes control** (takeover with debts cleared).
    /// Port of `FUN_005884a0`. Fires from the finance-status dispatcher
    /// (`FUN_00588c70`) every finance tick.
    ///
    /// Gates (verified line-by-line vs decompile):
    ///   * `reputation > 4749` (0x128d)  — top-half clubs only.
    ///   * Nation NOT in the 3-nation exempt list (`DAT_009bb9d0/79c/8a4`).
    ///     `TODO`: those three nation ids need decoding at bootstrap; wired
    ///     here as `nation_takeover_exempt` on the FinanceBook (empty by
    ///     default → mechanism can fire in all nations).
    ///   * `+0x82 == 0` — "no takeover pending" flag (Rust:
    ///     `takeover_pending: false`, defaults on).
    ///   * RNG gate: `FUN_008fc4f0(<nation_prob_table>) == 0`.
    ///
    /// Amount (from lines 31–55, no-chairman branch):
    ///   `raw = (rep * 5 + 15000) * 500`   (rep 5000 → £20,000,000)
    ///   Cap 1: don't inject more than the actual debt (`-balance`).
    ///   Cap 2 (rep < 2000): `min(raw, rep² + 1_500_000)`.
    ///   Cap 2 (rep ≥ 2000): `min(raw, rep² × 0.25 + 4_000_000)` — the exact
    ///     float multiplier `_DAT_009569e0` = 0.25 (VERIFIED via cm-lift from
    ///     shipped exe .rdata); the branch also clamps to
    ///     `4_000_001` / `4_000_000` around the threshold (lines 47–54).
    ///
    /// Chairman branch (lines 68–107):
    ///   RNG against chairman `+0xf` (ambition) then `+0x20` (charisma).
    ///   Amount: `(chairman_charisma × 20 + 150) × rep + 750_000`, same caps.
    ///
    /// Effects on the rich club (lines 56–63):
    ///   * balance += amount
    ///   * month_owner_gift, year_owner_gift ledgers += amount
    ///   * news code 3 → "A new board of directors has assumed control of X
    ///     with £Y of the club's debts being cleared" (0x009b4f80)
    ///   * `+0x165 = 0` — **exits Admin/InTheRed formal state**.
    ///
    /// This is the mechanism the game uses to rescue clubs from administration
    /// via a new owner. NOT the same as the stadium-share £20M transfer below
    /// (`FUN_00586ec0`) which is a rich→sibling in-family transfer.
    pub fn takeover_check(&mut self, rng: &mut crate::match_engine_exe::MatchRng) {
        // Exempt nations (verified — FUN_005f71d0 sets DAT_009bb79c/8a4/9d0 to
        // Republic of Ireland, Northern Ireland, Wales respectively from a
        // strcmpi ladder over nation names during startup):
        const EXEMPT_NATIONS: &[i32] = &[92, 128, 207];
        let candidates: Vec<u32> = self.clubs.iter()
            .filter(|c| {
                let rep = self.club_reputation.get(&c.club_id).copied().unwrap_or(0);
                rep as i32 > 0x128d
                    && !c.takeover_pending
                    && !EXEMPT_NATIONS.contains(&self.club_nation.get(&c.club_id).copied().unwrap_or(0))
                    && (c.in_administration || matches!(
                        c.status(rep),
                        FinanceStatus::InTheRed | FinanceStatus::Admin
                    ))
            })
            .map(|c| c.club_id)
            .collect();
        for club_id in candidates {
            let rep = self.club_reputation.get(&club_id).copied().unwrap_or(0) as i64;
            // Per-status probability denominators (VERIFIED from asm at
            // 0x00588512..0x00588528 loading f64 constants at
            //   0x00958348 = 0.001  (Rich/Healthy/Normal)
            //   0x00955880 = 0.1    (Admin,     status 0xff)
            //   0x009569e0 = 0.25   (InTheRed,  status 0xfe)
            // then computing 4.0/const (no-chairman branch at 0x5886c7) or
            // 30.0/const (chairman branch at 0x5884f). Table of takeover
            // probability = 1/denom per tick:
            //             Rich/Healthy   Admin   InTheRed
            //   no-chair        4000       40         16
            //   chair          30000      300        120
            let status = self.clubs.iter().find(|c| c.club_id == club_id).unwrap()
                .status(rep as u16);
            // Real chairman flag from club record +0x6d (populated in
            // FinanceBook::seed_from_clubs via ClubView::flag_6d()).
            let has_chairman = self.club_has_chairman.get(&club_id).copied().unwrap_or(false);
            let denom: u32 = match (has_chairman, status) {
                (false, FinanceStatus::InTheRed) => 16,
                (false, FinanceStatus::Admin)    => 40,
                (false, _)                        => 4_000,
                (true,  FinanceStatus::InTheRed) => 120,
                (true,  FinanceStatus::Admin)    => 300,
                (true,  _)                        => 30_000,
            };
            if rng.range(denom) != 0 { continue; }
            // Amount — no-chairman branch (line 31 onwards).
            let raw = (rep * 5 + 15_000) * 500;                      // rep 5000 → £20M
            let debt = (-self.clubs.iter().find(|c| c.club_id == club_id).unwrap().balance).max(0);
            let mut amount = raw.min(debt.max(raw));                 // pay AT MOST the debt (line 36-38)
            // Cap 2 (line 39-55):
            let rep2 = rep * rep;
            let cap = if rep < 2000 {
                rep2 + 1_500_000
            } else {
                (rep2 / 4) + 4_000_000                                // × 0.25 (verified _DAT_009569e0=0.25)
            };
            if amount > cap { amount = cap; }
            if let Some(c) = self.clubs.iter_mut().find(|c| c.club_id == club_id) {
                c.balance = c.balance.saturating_add(amount);
                c.month_owner_gift = c.month_owner_gift.saturating_add(amount);
                c.year_owner_gift  = c.year_owner_gift.saturating_add(amount);
                c.in_administration = false;    // +0x165 = 0
                c.takeover_pending = true;      // debounce (mirror +0x82 latch)
                c.last_takeover_amount = amount;
            }
        }
    }

    /// **Full chairman-reroll takeover (news case 2)** — port of `FUN_00588840`
    /// is DEFERRED. That fn clears the whole debt AND randomizes the chairman
    /// record (+0xf/+0x16/+0x1d/+0x20 all reroll to `rand(20)+1`; sometimes
    /// reassigns nation). Requires a chairman-record model I haven't built yet
    /// (would be a follow-up to add ChairmanView + a ChairmanBook keyed by
    /// club_id). Cash arithmetic mirrors takeover_check; only the effect on
    /// board_of_directors composition differs. News template:
    ///   0x009b4ec4 = "A new board of directors has assumed control of X.
    ///                 They have agreed a financial package with the creditors
    ///                 to keep the club from falling into receivership."
    /// (No £X debt figure — the "silent" takeover variant.)

    /// **Board pays some of the club's debts** — port of `FUN_00587c40`.
    /// Smaller, more frequent bailout event that does NOT change ownership.
    ///
    /// Gates (lines 13–29):
    ///   * status ∈ {Rich(0), Healthy(0), InTheRed(-2), Admin(-1)} — line 14/15.
    ///   * `reputation ≤ 3500` (0xdac) OR the balance is more than `rep * 25 * 4e9`
    ///     in debt (line 18–28: signed 64-bit comparison against `rep * 0x19`
    ///     scaled). Small/mid clubs and truly desperate big clubs.
    ///   * RNG(7000): `rand + 3500 > rep` — inverse-reputation gate (smaller
    ///     clubs are MORE likely to be rescued).
    ///
    /// No-chairman branch (34–72):
    ///   * RNG(<nation-prob>) == 0 gate + RNG(10) == 0 gate.
    ///   * If rep < 1250 (0x4e2): amount = `rep × 450`  (rep 800 → £360k)
    ///   * Else: amount = `rep × 300` (rep 3000 → £900k)
    ///
    /// Chairman branch (73–101):
    ///   * RNG(<nation-prob>) == 0.
    ///   * RNG(chairman `+0x20`) == 0 — chairman charisma gate.
    ///   * Same amount formulas, or default `rep × 150` fallback (line 102).
    ///
    /// Effects (LAB_00587ef8, lines 103–108):
    ///   * balance += amount
    ///   * month_owner_gift, year_owner_gift += amount
    ///   * news code 4 → "The board paid £X of the club's debts to keep the
    ///     club from falling into receivership" (0x009b53dc)
    ///   * Does NOT clear admin flag on its own — the balance-based classifier
    ///     will re-evaluate next tick.
    pub fn board_debt_payment(&mut self, rng: &mut crate::match_engine_exe::MatchRng) {
        let candidates: Vec<u32> = self.clubs.iter()
            .filter(|c| {
                let rep = self.club_reputation.get(&c.club_id).copied().unwrap_or(0);
                let status = c.status(rep);
                matches!(status, FinanceStatus::Rich | FinanceStatus::Healthy
                                 | FinanceStatus::InTheRed | FinanceStatus::Admin)
                    && rep <= 3500 || c.balance < -(rep as i64 * 25 * 100_000)
            })
            .map(|c| c.club_id)
            .collect();
        for club_id in candidates {
            let rep = self.club_reputation.get(&club_id).copied().unwrap_or(0) as i64;
            // Inverse-rep gate (line 30-33). Higher rep → less likely.
            if rng.range(7000) as i64 + 3500 <= rep { continue; }
            // Item 3 VERIFIED — per-status RNG denom is `6.0 / status_divisor`
            // via asm 0x00587c9f..0x00587dd8 (no per-nation table exists).
            // Effective denoms: {InTheRed, Admin & rep<3500} → 3; else → 15.
            let status = self.clubs.iter().find(|c| c.club_id == club_id).unwrap()
                .status(rep as u16);
            let denom: u32 = match status {
                FinanceStatus::InTheRed                             => 3,
                FinanceStatus::Admin  if rep < 3500                 => 3,
                _                                                    => 15,
            };
            if rng.range(denom) != 0 { continue; }
            // Second gate: no-chairman path uses RNG(10) — exe SKIPS when
            // RNG returns 0 (`je 0x587f40` at 0x00587def). The previous
            // port had `!= 0` which was inverted; now matches the exe.
            // TODO chairman branch consults chairman.charisma; wire when
            // ChairmanAttrs lands in the state.
            let has_chairman = self.club_has_chairman.get(&club_id).copied().unwrap_or(false);
            if !has_chairman && rng.range(10) == 0 { continue; }
            let amount: i64 = if rep < 1250 { rep * 450 } else { rep * 300 };
            if let Some(c) = self.clubs.iter_mut().find(|c| c.club_id == club_id) {
                c.balance = c.balance.saturating_add(amount);
                c.month_owner_gift = c.month_owner_gift.saturating_add(amount);
                c.year_owner_gift  = c.year_owner_gift.saturating_add(amount);
            }
        }
    }

    /// Stadium-share £20M transfer — port of `FUN_00586ec0`:56-114.
    ///
    /// Fires at the start of the weekly-finance tick, BEFORE wages. When a
    /// rich club (balance > £35M) has a stadium (`home_stadium_id.is_some()`)
    /// and hasn't fired this transfer yet (`!stadium_share_used`):
    ///
    /// 1. Scan every other club that shares the SAME stadium.
    ///    - If a sibling club is `Admin` or `InTheRed` (its `+0x165` flag was
    ///      1 or 2 in the exe) — transfer £20M from rich → struggling
    ///      ground-share partner. Mark rich as "used", clear sibling's
    ///      used-bit. (News codes 4 = rich club drained, 5 = partner rescued.)
    ///    - If a sibling exists but has no finance record — rich club loses
    ///      £20M anyway (news 4 only).
    /// 2. If no ground-share partner found — rich club still drains £20M
    ///    (the exe writes the same subtraction unconditionally).
    ///
    /// Reference (from Ghidra):
    /// ```text
    /// 0058_6f30  if (35000000 < balance)
    /// 0058_6f8f    puVar16[0x23] += 20000000;    // month_owner_take
    /// 0058_6f94    puVar16[0x4b] += 20000000;    // year_owner_take
    /// 0058_6f99    balance -= 20000000;
    /// 0058_6fc0    puVar9[0x2d] += 20000000;     // sibling month_owner_gift
    /// 0058_6fc9    puVar9[0x55] += 20000000;     // sibling year_owner_gift
    /// 0058_6fd1    sibling.balance += 20000000;
    /// ```
    /// Runs on caller's schedule — call from `pay_weekly_wages` prologue.
    pub fn stadium_share_transfers(&mut self) {
        const AMOUNT: i64 = 20_000_000;
        const RICH_THRESHOLD: i64 = 35_000_000;
        let candidates: Vec<(u32, i32)> = self
            .clubs
            .iter()
            .filter(|c| !c.stadium_share_used
                && c.balance > RICH_THRESHOLD
                && c.home_stadium_id.is_some())
            .map(|c| (c.club_id, c.home_stadium_id.unwrap()))
            .collect();
        for (rich_id, stadium) in candidates {
            let sibling: Option<u32> = self.clubs.iter()
                .find(|c| c.club_id != rich_id
                    && c.home_stadium_id == Some(stadium)
                    && c.stadium_share_used)
                .map(|c| c.club_id);
            match sibling {
                None => {
                    if let Some(r) = self.clubs.iter_mut().find(|c| c.club_id == rich_id) {
                        r.balance = r.balance.saturating_sub(AMOUNT);
                        r.stadium_share_used = true;
                    }
                }
                Some(sib_id) => {
                    let sib_is_stressed = self.clubs.iter()
                        .find(|c| c.club_id == sib_id)
                        .map(|c| {
                            let rep = self.club_reputation.get(&sib_id).copied().unwrap_or(1000);
                            matches!(c.status(rep), FinanceStatus::Admin | FinanceStatus::InTheRed)
                        })
                        .unwrap_or(false);
                    if let Some(r) = self.clubs.iter_mut().find(|c| c.club_id == rich_id) {
                        r.balance = r.balance.saturating_sub(AMOUNT);
                        r.stadium_share_used = true;
                    }
                    if sib_is_stressed {
                        if let Some(s) = self.clubs.iter_mut().find(|c| c.club_id == sib_id) {
                            s.balance = s.balance.saturating_add(AMOUNT);
                            s.stadium_share_used = false;
                            let rep = self.club_reputation.get(&sib_id).copied().unwrap_or(1000);
                            if s.in_administration
                                && !matches!(s.status(rep), FinanceStatus::Admin)
                            {
                                s.in_administration = false;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Yearly rollover for the stadium-share eligibility bit
    /// (clears the exe's `+0x6d` for every club).
    pub fn reset_yearly_stadium_flags(&mut self) {
        for c in &mut self.clubs {
            c.stadium_share_used = false;
            c.takeover_pending = false;
            c.year_owner_gift = 0;
        }
    }

    /// Weekly wage payment tick (kill #8b) — port of `FUN_00586ec0` tail
    /// (:363-421). Three revenue tiers by balance-vs-reputation, RNG-banded
    /// per-rep wage draw; deducted from balance and accumulated into the
    /// month_wages ledger. Skint clubs (balance < rep×3000) pay no wages that
    /// week (matches the exe's `return` at the bottom of the cascade).
    pub fn pay_weekly_wages(&mut self) {
        // Stadium-share £20M transfer fires FIRST (FUN_00586ec0:56-114 sits
        // above the wage cascade in the exe).
        self.stadium_share_transfers();
        let mut rng = crate::match_engine_exe::MatchRng::new(0x0058_6ec0);
        for c in &mut self.clubs {
            let rep = self.club_reputation.get(&c.club_id).copied().unwrap_or(1000) as i64;
            // Chairman-satisfaction adjustment (FUN_00586ec0:374-378) —
            // rep_adj is used in the mid/low tier compares. `sat` = board
            // chairman satisfaction byte (+0x59), `expect` = manager
            // expectation byte (+0x20). When either is missing we fall
            // back to raw rep (matches the else-branch at :378).
            let (sat, expect) = self.chairman.get(&c.club_id)
                .map(|ch| (ch.ambition as i32, ch.manager_patience as i32))
                .unwrap_or((7, 0));  // neutral defaults
            let rep_adj = ((rep as i32) + ((7 - sat) * 3 - expect) * 5).max(500) as i64;
            // Compute weekly wage draw by tier (FUN_00586ec0:363-421).
            let bal = c.balance;
            let top_gate = (rep * 8000).max(500_000);
            let flag82 = c.takeover_pending; // proxy for chairman_boost_flag +0x82
            let weekly = if bal > top_gate && flag82 {
                // Top tier — rand(0x1F5) + 2000 (or 1500 under 4000 rep),
                // fully cash-comfortable + chairman-boosted branch.
                if rep < 4000 {
                    ((rng.range(0x1F5) as i64 + 1500) * rep).max(50_000)
                } else {
                    (rng.range(0x1F5) as i64 + 2000) * rep
                }
            } else if bal > rep_adj * 6000 {
                // Mid tier — cash > rep_adj×6000 uses adjusted rep + jitter.
                if rep < 4000 {
                    (rng.range(0x1F5) as i64 + 1000) * rep
                } else {
                    (rng.range(0x1F5) as i64 + 1500) * rep
                }
            } else if bal > rep_adj * 3000 {
                // Low tier — fixed step by rep bracket.
                if rep < 4000 { rep * 500 } else { rep * 750 }
            } else {
                0 // skint — no wages this week (FUN_00586ec0:384-389)
            };
            c.balance = c.balance.saturating_sub(weekly);
            c.month_wages = c.month_wages.saturating_add(weekly);
            c.weekly_wage_bill = weekly.min(u32::MAX as i64) as u32;
        }
    }

    /// End-of-month rollover (kill #8c) — port of `FUN_00586cf0`. Clears the
    /// this-month ledger accumulators; increments/resets the in-red counter;
    /// runs the board-confidence tick per real `FUN_00588c70` dispatch on the
    /// real status classifier.
    pub fn end_of_month(&mut self) {
        // Board-driven cash events fire FIRST at month end, since they can
        // pull a club out of admin before the classifier below reads status.
        let mut rng = crate::match_engine_exe::MatchRng::new(0x0058_84a0);
        self.takeover_check(&mut rng);
        self.board_debt_payment(&mut rng);
        for c in &mut self.clubs {
            // Also reset the this-month owner-gift ledger.
            c.month_owner_gift = 0;
            // In-red counter.
            if c.balance < 0 {
                c.months_in_the_red = c.months_in_the_red.saturating_add(1);
            } else {
                c.months_in_the_red = 0;
            }
            // Board-confidence tick (FUN_00588c70 dispatch).
            let rep = self.club_reputation.get(&c.club_id).copied().unwrap_or(1000);
            let status = c.status(rep);
            match status {
                FinanceStatus::Rich => {
                    if c.board_confidence > 5 { c.board_confidence -= 1; }
                }
                FinanceStatus::Healthy => {
                    if c.board_confidence > 0 { c.board_confidence -= 1; }
                }
                FinanceStatus::Normal => {
                    if c.board_confidence > 10 { c.board_confidence -= 1; }
                }
                FinanceStatus::InTheRed => {
                    c.board_confidence = c.board_confidence.saturating_add(1);
                }
                FinanceStatus::Admin => {
                    c.board_confidence = c.board_confidence.saturating_add(3);
                }
            }
            // Administration state transition (kill #8/9 refinement, real port
            // of `FUN_00588c70`:150-217 Admin branch):
            //   Enter admin when status → Admin AND not already in admin.
            //   Exit admin when status improves back to Normal or better.
            match status {
                FinanceStatus::Admin => {
                    if !c.in_administration {
                        c.in_administration = true;
                        // Wages get cut on next weekly tick (the wage cascade
                        // in pay_weekly_wages naturally lowers when balance is
                        // very negative). The "can't refuse bids" bit is read
                        // from `in_administration` at bid-resolution time.
                    }
                }
                FinanceStatus::Normal | FinanceStatus::Healthy | FinanceStatus::Rich => {
                    // Recovery — release from admin state.
                    if c.in_administration {
                        c.in_administration = false;
                    }
                }
                FinanceStatus::InTheRed => {
                    // Stay in whichever state we're in; the threshold-based
                    // classifier will re-enter/exit admin as balance moves.
                }
            }
            // Rollover ledger: this-month → 0.
            c.month_wages = 0;
            c.month_gate = 0;
            c.month_tv_prize = 0;
        }
    }

    /// True if the given club is in administration and its players must accept
    /// incoming bids. Consumed by `TransferMarket::resolve_bids` and
    /// `run_ai_transfer_pass` (kill #8/9 wire).
    pub fn is_in_administration(&self, club_id: u32) -> bool {
        self.clubs.iter().any(|c| c.club_id == club_id && c.in_administration)
    }

    /// Submit a [`BoardDemand`] to the board — applies the response's
    /// side-effects (transfer_budget/balance/board_confidence) and returns
    /// the response so the caller can render a news template.
    ///
    /// Item 5 wire — call site: human-manager action pipeline when the
    /// manager clicks "Request X" in the board interaction screen.
    ///
    /// # Params
    /// - `club_id`: manager's club
    /// - `demand`: which of the 7 requests
    /// - `recent_form_score`: manager's recent form aggregate (-100..+100)
    pub fn submit_board_demand(
        &mut self,
        club_id: u32,
        demand: BoardDemand,
        recent_form_score: i16,
    ) -> BoardResponse {
        let club = match self.clubs.iter_mut().find(|c| c.club_id == club_id) {
            Some(c) => c,
            None => return BoardResponse::OutOfScope,
        };
        let response = evaluate_board_demand(demand, club, recent_form_score);
        match response {
            BoardResponse::Approved { cash_granted, confidence_delta } => {
                // Route cash: TransferFunds → transfer_budget; else balance.
                match demand {
                    BoardDemand::TransferFunds => {
                        club.transfer_budget = club.transfer_budget
                            .saturating_add(cash_granted);
                    }
                    _ => {
                        club.balance = club.balance.saturating_add(cash_granted);
                    }
                }
                // Bump confidence, clamp 0..100
                let new_conf = (club.board_confidence as i32 + confidence_delta as i32)
                    .clamp(0, 100) as u8;
                club.board_confidence = new_conf;
            }
            BoardResponse::Refused { confidence_delta, .. } => {
                let new_conf = (club.board_confidence as i32 + confidence_delta as i32)
                    .clamp(0, 100) as u8;
                club.board_confidence = new_conf;
            }
            BoardResponse::OutOfScope => {
                // No side-effect (silly requests just refuse silently)
            }
        }
        response
    }

    /// Apply a fine to a player. Deducts cash from the player's contract
    /// (via the transfer market), routes reason to the fine-reaction
    /// dispatcher, and applies any team ripple to the whole squad's
    /// morale via [`crate::transfer::apply_mood_delta`].
    ///
    /// Item 5 wire — call site: human-manager fines a player from the
    /// player screen, or auto-fine triggers on missed-training news.
    ///
    /// # Params
    /// - `market`: transfer market carrying player contracts (mutated
    ///   for the team-ripple + fine-cash deduction)
    /// - `club_id`: fined player's club
    /// - `player_id`: fined player
    /// - `tier`: [`FineTier`]
    /// - `reason`: [`FineReason`]
    /// - `player_popularity`: how well the fined player is liked by the
    ///   squad (0..20; higher → more likely to ripple)
    ///
    /// Returns the [`FineOutcome`] so caller can render a news template.
    pub fn apply_fine(
        &mut self,
        market: &mut crate::transfer::TransferMarket,
        _club_id: u32,
        player_id: u32,
        tier: FineTier,
        reason: FineReason,
        player_popularity: u8,
    ) -> FineOutcome {
        // Find fined player's contract for wage lookup + cash deduction.
        let weekly_wage = market.contracts.iter()
            .find(|c| c.player_id == player_id)
            .map(|c| c.weekly_wage)
            .unwrap_or(0);
        let fine_amount = compute_fine_amount(tier, weekly_wage);

        // Deduct fine from the player's "pay" — modelled here as a
        // negative one-shot mood_delta (exe's fine hits morale as well
        // as the pay packet; the pay-packet deduction flows via the
        // weekly wage tick).
        let fined_delta: i8 = match tier {
            FineTier::OneWeekWages   => -3,
            FineTier::TwoWeeksWages  => -6,
            FineTier::OneMonthWages  => -10,
            FineTier::FixedAmount(_) => {
                // Scale by ratio to a month's wages, clamped
                let month = (weekly_wage as i64) * 4;
                if month > 0 {
                    (-10 * (fine_amount * 100 / month).max(1).min(100) / 100) as i8
                } else { -5 }
            }
        };
        if let Some(c) = market.contracts.iter_mut()
            .find(|c| c.player_id == player_id) {
            crate::transfer::apply_mood_delta(c, fined_delta);
        }

        // Evaluate team reaction
        let outcome = evaluate_fine_reaction(tier, reason, player_popularity);
        if let FineOutcome::TeamRipple { team_delta } = outcome {
            // Apply to EVERY OTHER player at this club (not the fined one).
            let target_club = market.contracts.iter()
                .find(|c| c.player_id == player_id).map(|c| c.club_id);
            if let Some(cid) = target_club {
                for c in market.contracts.iter_mut() {
                    if c.club_id == cid && c.player_id != player_id {
                        crate::transfer::apply_mood_delta(c, team_delta);
                    }
                }
            }
        }
        outcome
    }

    /// Deliver a press statement about a specific player. Applies the
    /// tier-scaled mood delta to the target player's contract.
    ///
    /// Item 5 wire — call site: human-manager press-conference screen.
    ///
    /// # Params
    /// - `market`: for the target player's contract
    /// - `target_player_id`: subject of the statement
    /// - `stmt`: which of the 6 verified statement templates
    /// - `tier`: newspaper coverage tier (National doubles impact)
    pub fn deliver_press_statement(
        &self,
        market: &mut crate::transfer::TransferMarket,
        target_player_id: u32,
        stmt: PressStatement,
        tier: NewspaperTier,
    ) -> i8 {
        let delta = scaled_press_delta(stmt, tier);
        if let Some(c) = market.contracts.iter_mut()
            .find(|c| c.player_id == target_player_id) {
            crate::transfer::apply_mood_delta(c, delta);
        }
        delta
    }

    /// Post-match gate + TV/prize income (kill #8d) — port of `FUN_00584790`.
    /// League gates: rand(250)+rand(250) base; cup gates: rand(400 or 200)
    /// (all in the exe's inline float form, collapsed here). Reputation of the
    /// scoring/hosting side scales it via `FUN_00585060` — `(rep/500 + 3)`
    /// multiplier (league) or `+4` (cup). Awards split between home and away.
    pub fn record_match_income(&mut self, home_club: u32, away_club: u32, is_cup: bool) {
        let mut rng = crate::match_engine_exe::MatchRng::new(
            0x0058_4790 ^ ((home_club as u64) << 16) ^ (away_club as u64));
        let home_rep = self.club_reputation.get(&(home_club as u32)).copied().unwrap_or(1000) as i64;
        let away_rep = self.club_reputation.get(&(away_club as u32)).copied().unwrap_or(1000) as i64;

        // Gate calc — now uses the REAL attendance figures from the shipped
        // club record (kill #8d refinement). Actual attendance = a random
        // draw between the club's minimum and maximum, weighted toward the
        // average. Then × per-ticket price (rep-scaled). This replaces the
        // pure-reputation-derived proxy.
        let gate: i64 = if let Some(&(avg, min_att, max_att)) = self.club_attendance.get(&home_club) {
            let (avg, min_a, max_a) = (avg as i64, min_att as i64, max_att as i64);
            // Draw around average with symmetric spread capped by (min, max).
            let spread = (max_a - min_a).max(1) as u32;
            let draw = (rng.range(spread) as i64) + min_a;
            // Weight toward the average: 60% avg + 40% draw.
            let attendance = (avg * 60 + draw * 40) / 100;
            // Per-ticket price (£): rep-scaled — cup ticket premium.
            let ticket = (home_rep / 500) + if is_cup { 20 } else { 15 };
            attendance.saturating_mul(ticket)
        } else {
            // Fallback: original rep-derived proxy for clubs without shipped attendance.
            let gate_base = if is_cup {
                rng.range(400) as i64 + 50
            } else {
                rng.range(0xFA) as i64 + rng.range(0xFA) as i64
            };
            let mult_home = (home_rep / 500) + if is_cup { 4 } else { 3 };
            gate_base * mult_home * 100
        };

        // TV/prize: rep-scaled base, awarded to both sides in league; home-heavy
        // in cups. Simplified from the FPU switch at :133-166.
        let tv_prize_home = ((home_rep / 500) + 2) * 250;
        let tv_prize_away = ((away_rep / 500) + 2) * 250;

        if let Some(h) = self.clubs.iter_mut().find(|c| c.club_id == home_club) {
            h.balance = h.balance.saturating_add(gate).saturating_add(tv_prize_home);
            h.month_gate = h.month_gate.saturating_add(gate);
            h.month_tv_prize = h.month_tv_prize.saturating_add(tv_prize_home);
        }
        if let Some(a) = self.clubs.iter_mut().find(|c| c.club_id == away_club) {
            a.balance = a.balance.saturating_add(tv_prize_away);
            a.month_tv_prize = a.month_tv_prize.saturating_add(tv_prize_away);
        }
    }

    pub fn for_club(&self, club_id: u32) -> Option<&ClubFinance> {
        self.clubs.iter().find(|c| c.club_id == club_id)
    }
}

/// Standard slate for a top-flight European league — 5-year contracts, no
/// cap (pre-Bosman was different; 2001-02 is post-Bosman), Aug/Feb window,
/// 3 non-EU foreigners allowed. This is the default that most country
/// registrations override with a few fields.
pub const STANDARD_EUROPEAN_TOP_FLIGHT: CountryRulesSpec = CountryRulesSpec {
    nation_id: -1,  // caller overrides
    max_contract_years: 5,
    salary_ceiling_weekly: 0,
    max_foreigners_matchday: 3,
    transfer_window_open_month: 8,
    youth_recruitment: true,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn club(balance: i64, wage_bill: u32, confidence: u8) -> ClubFinance {
        ClubFinance {
            club_id: 1,
            balance,
            weekly_wage_bill: wage_bill,
            transfer_budget: 0,
            months_in_the_red: 0,
            board_confidence: confidence,
            in_administration: false,
            ..Default::default()
        }
    }

    // ---- Year-end seam (single-store consolidation, ledger §12a) ----

    /// `year_end_state` reads the LIVE balance. This is the property the
    /// consolidation exists for: the tick debits `balance`, and year-end
    /// stadium affordability must see that, not a separately-seeded copy.
    #[test]
    fn year_end_state_reads_live_balance_after_tick_mutation() {
        let mut book = FinanceBook::new();
        book.clubs.push(club(30_000_000, 0, 100));
        assert_eq!(book.year_end_state(1).cash, 30_000_000);
        // Any tick-side mutation of balance is visible immediately.
        book.for_club_mut(1).unwrap().balance -= 1_250_000;
        assert_eq!(book.year_end_state(1).cash, 28_750_000);
        // Absent club -> all-zero snapshot (mirrors the old ledger's get).
        let z = book.year_end_state(999);
        assert_eq!((z.cash, z.season_misc_expense, z.lifetime_misc_expense,
                    z.season_subsidy_income, z.lifetime_subsidy_income),
                   (0, 0, 0, 0, 0));
    }

    /// `apply_year_end_write` lands every field of a PendingFinanceWrite on
    /// the book and `year_end_state` reads them back — the FinanceBook
    /// analogue of c15_1a_trace_matches_world_ledger_state_after_apply.
    #[test]
    fn apply_year_end_write_round_trips_all_five_fields() {
        use crate::c15_1_world_apply::PendingFinanceWrite;
        let mut book = FinanceBook::new();
        book.clubs.push(club(50_000_000, 0, 100));
        let w = PendingFinanceWrite {
            club_id: 1,
            new_cash: 42_250_000,
            new_season_misc_expense: 9_750_000,
            new_lifetime_misc_expense: 17_750_000,
            new_season_subsidy_income: 100_000,
            new_lifetime_subsidy_income: 500_000,
        };
        book.apply_year_end_write(&w);
        let s = book.year_end_state(1);
        assert_eq!(s.cash, w.new_cash);
        assert_eq!(s.season_misc_expense, w.new_season_misc_expense);
        assert_eq!(s.lifetime_misc_expense, w.new_lifetime_misc_expense);
        assert_eq!(s.season_subsidy_income, w.new_season_subsidy_income);
        assert_eq!(s.lifetime_subsidy_income, w.new_lifetime_subsidy_income);
        // The write went to `balance` itself (single store, no shadow).
        assert_eq!(book.for_club(1).unwrap().balance, 42_250_000);
        // Other ClubFinance state is untouched by a year-end write.
        assert_eq!(book.for_club(1).unwrap().board_confidence, 100);
    }

    /// A write for a club the book doesn't hold creates it rather than
    /// dropping the write (only reachable in tests — boot seeds all clubs).
    #[test]
    fn apply_year_end_write_creates_absent_club() {
        use crate::c15_1_world_apply::PendingFinanceWrite;
        let mut book = FinanceBook::new();
        assert!(book.for_club(7).is_none());
        book.apply_year_end_write(&PendingFinanceWrite {
            club_id: 7, new_cash: -5, new_season_misc_expense: 1,
            new_lifetime_misc_expense: 2, new_season_subsidy_income: 3,
            new_lifetime_subsidy_income: 4,
        });
        let c = book.for_club(7).expect("created on demand");
        assert_eq!((c.balance, c.season_misc_expense, c.lifetime_misc_expense,
                    c.season_subsidy_income, c.lifetime_subsidy_income),
                   (-5, 1, 2, 3, 4));
        assert_eq!(book.clubs.len(), 1);
    }

    /// `year_end_states` snapshots exactly the requested ids, in a map.
    #[test]
    fn year_end_states_snapshots_requested_clubs_only() {
        let mut book = FinanceBook::new();
        book.clubs.push(club(10, 0, 50));
        let mut other = club(20, 0, 50); other.club_id = 2; book.clubs.push(other);
        let m = book.year_end_states([2u32, 1, 42]);
        assert_eq!(m.len(), 3);
        assert_eq!(m[&1].cash, 10);
        assert_eq!(m[&2].cash, 20);
        assert_eq!(m[&42].cash, 0); // absent -> zeros, still present as a key
    }

    /// seed_from zero-inits the four year-end accumulators (the exe
    /// zero-inits the whole 0x167 record before the cash seed lands).
    #[test]
    fn seed_from_zeroes_year_end_accumulators() {
        let c = ClubFinance::seed_from(9, 4000, false, 1_000_000, false);
        assert_eq!(c.balance, 1_000_000);
        assert_eq!((c.season_misc_expense, c.lifetime_misc_expense,
                    c.season_subsidy_income, c.lifetime_subsidy_income),
                   (0, 0, 0, 0));
    }

    #[test]
    fn half_time_oranges_always_out_of_scope() {
        let c = club(1_000_000_000, 100_000, 100);
        let r = evaluate_board_demand(BoardDemand::HalfTimeOranges, &c, 100);
        assert_eq!(r, BoardResponse::OutOfScope);
    }

    #[test]
    fn time_to_rebuild_needs_confidence_and_form() {
        // Low confidence → refused
        let broke = club(0, 100_000, 20);
        let r = evaluate_board_demand(BoardDemand::TimeToRebuild, &broke, 0);
        assert!(matches!(r, BoardResponse::Refused { .. }));
        // High confidence + decent form → approved
        let confident = club(0, 100_000, 50);
        let r = evaluate_board_demand(BoardDemand::TimeToRebuild, &confident, 10);
        assert!(matches!(r, BoardResponse::Approved { .. }));
        // High confidence but terrible form → refused
        let r = evaluate_board_demand(BoardDemand::TimeToRebuild, &confident, -80);
        assert!(matches!(r, BoardResponse::Refused { .. }));
    }

    #[test]
    fn wage_budget_needs_cash_and_confidence() {
        // Broke → CannotAfford
        let broke = club(-100_000, 100_000, 60);
        let r = evaluate_board_demand(BoardDemand::HigherWageBudget, &broke, 0);
        assert!(matches!(r, BoardResponse::Refused {
            reason: BoardRefusalReason::ClubCannotAfford, .. }));
        // Rich + confident → approved with cash grant
        let rich = club(50_000_000, 100_000, 60);
        let r = evaluate_board_demand(BoardDemand::HigherWageBudget, &rich, 0);
        match r {
            BoardResponse::Approved { cash_granted, .. } => {
                assert!(cash_granted > 0, "grant = {}", cash_granted);
            }
            other => panic!("expected Approved, got {:?}", other),
        }
    }

    #[test]
    fn transfer_funds_scales_with_surplus() {
        let c = club(100_000_000, 100_000, 60);
        let r = evaluate_board_demand(BoardDemand::TransferFunds, &c, 5);
        match r {
            BoardResponse::Approved { cash_granted, .. } => {
                // 100M - 5.2M wages = ~94.8M surplus; grant = surplus / 4
                assert!(cash_granted > 20_000_000);
                assert!(cash_granted < 30_000_000);
            }
            other => panic!("expected Approved, got {:?}", other),
        }
    }

    #[test]
    fn stadium_expansion_high_bar() {
        // Modest cash → refused (needs > £5M surplus + 60 confidence)
        let modest = club(1_000_000, 100_000, 50);
        let r = evaluate_board_demand(BoardDemand::ExpandStadium, &modest, 0);
        assert!(matches!(r, BoardResponse::Refused { .. }));
        // Rich + confident → approved
        let rich = club(20_000_000, 100_000, 70);
        let r = evaluate_board_demand(BoardDemand::ExpandStadium, &rich, 0);
        assert!(matches!(r, BoardResponse::Approved { .. }));
    }

    fn book_with_club(club_id: u32, balance: i64, wage_bill: u32, conf: u8) -> FinanceBook {
        let mut b = FinanceBook::new();
        let mut c = club(balance, wage_bill, conf);
        c.club_id = club_id;
        b.clubs.push(c);
        b
    }

    #[test]
    fn submit_board_demand_wires_approved_transfer_funds_to_budget() {
        // Rich confident club → transfer funds approved → transfer_budget grows.
        let mut b = book_with_club(100, 100_000_000, 100_000, 60);
        let r = b.submit_board_demand(100, BoardDemand::TransferFunds, 5);
        assert!(matches!(r, BoardResponse::Approved { .. }));
        let c = b.clubs.iter().find(|c| c.club_id == 100).unwrap();
        assert!(c.transfer_budget > 20_000_000);
        assert_eq!(c.balance, 100_000_000);  // balance untouched
    }

    #[test]
    fn submit_board_demand_wires_approved_wage_budget_to_balance() {
        // Rich confident club → wage budget approved → balance grows (headroom)
        let mut b = book_with_club(100, 50_000_000, 100_000, 60);
        let start_balance = b.clubs[0].balance;
        let start_budget = b.clubs[0].transfer_budget;
        let r = b.submit_board_demand(100, BoardDemand::HigherWageBudget, 5);
        assert!(matches!(r, BoardResponse::Approved { .. }));
        let c = b.clubs.iter().find(|c| c.club_id == 100).unwrap();
        assert!(c.balance > start_balance);
        assert_eq!(c.transfer_budget, start_budget);  // budget untouched
    }

    #[test]
    fn submit_board_demand_refused_lowers_confidence() {
        // Low-confidence + terrible-form → TimeToRebuild refused →
        // board_confidence drops by refusal delta.
        let mut b = book_with_club(100, 0, 100_000, 25);
        b.submit_board_demand(100, BoardDemand::TimeToRebuild, -50);
        assert!(b.clubs[0].board_confidence < 25);
    }

    #[test]
    fn submit_board_demand_half_time_oranges_no_side_effect() {
        // OutOfScope should not change anything
        let mut b = book_with_club(100, 5_000_000, 100_000, 50);
        let start_conf = b.clubs[0].board_confidence;
        let start_bal = b.clubs[0].balance;
        let r = b.submit_board_demand(100, BoardDemand::HalfTimeOranges, 50);
        assert_eq!(r, BoardResponse::OutOfScope);
        assert_eq!(b.clubs[0].board_confidence, start_conf);
        assert_eq!(b.clubs[0].balance, start_bal);
    }

    #[test]
    fn apply_fine_ripples_to_team_when_dispatcher_fires() {
        use crate::transfer::{TransferMarket, Contract, SquadStatus};
        let mut b = FinanceBook::new();
        let mut m = TransferMarket::default();
        // 3 players on club 100, one gets fined (popular player + heavy fine
        // + PoorPerformance → ripples)
        for pid in [1u32, 2, 3] {
            m.contracts.push(Contract {
                player_id: pid, club_id: 100, weekly_wage: 30_000,
                signed_year: 2001, expires_year: 2004,
                bosman_eligible: false, morale: 10, mood_delta: 0,
                squad_status: SquadStatus::FirstTeam,
                ..Default::default()
            });
        }
        let outcome = b.apply_fine(
            &mut m, 100, /*fined*/ 1,
            FineTier::OneMonthWages, FineReason::PoorPerformance,
            /*popularity*/ 15);
        assert_eq!(outcome, FineOutcome::TeamRipple { team_delta: -3 });
        // Fined player has direct hit (-10 for OneMonth)
        let p1 = m.contracts.iter().find(|c| c.player_id == 1).unwrap();
        assert!(p1.mood_delta <= -10);
        // Other players have the -3 ripple
        for other in [2u32, 3] {
            let c = m.contracts.iter().find(|c| c.player_id == other).unwrap();
            assert_eq!(c.mood_delta, -3);
        }
    }

    #[test]
    fn deliver_press_statement_applies_scaled_delta() {
        use crate::transfer::{TransferMarket, Contract, SquadStatus};
        let b = FinanceBook::new();
        let mut m = TransferMarket::default();
        m.contracts.push(Contract {
            player_id: 42, club_id: 100, weekly_wage: 30_000,
            signed_year: 2001, expires_year: 2004,
            bosman_eligible: false, morale: 10, mood_delta: 0,
            squad_status: SquadStatus::FirstTeam,
            ..Default::default()
        });
        // NotForSale + National → +5 * 2 = +10
        let delta = b.deliver_press_statement(
            &mut m, 42, PressStatement::NotForSale, NewspaperTier::National);
        assert_eq!(delta, 10);
        let c = m.contracts.iter().find(|c| c.player_id == 42).unwrap();
        assert_eq!(c.mood_delta, 10);
    }

    #[test]
    fn fine_reaction_accepted_for_indiscreet_remarks() {
        // IndiscreetRemarks → always accepted (player at fault)
        for tier in [FineTier::OneWeekWages, FineTier::OneMonthWages] {
            for pop in [0u8, 15, 20] {
                let r = evaluate_fine_reaction(tier, FineReason::IndiscreetRemarks, pop);
                assert_eq!(r, FineOutcome::AcceptedWithoutComment);
            }
        }
    }

    #[test]
    fn fine_reaction_ripples_on_heavy_poor_performance_with_popular_player() {
        let r = evaluate_fine_reaction(
            FineTier::OneMonthWages, FineReason::PoorPerformance, 15);
        assert_eq!(r, FineOutcome::TeamRipple { team_delta: -3 });
    }

    #[test]
    fn fine_reaction_no_ripple_for_light_fine_or_unpopular_player() {
        // Light fine + popular player → no ripple
        let r = evaluate_fine_reaction(
            FineTier::OneWeekWages, FineReason::PoorPerformance, 15);
        assert_eq!(r, FineOutcome::AcceptedWithoutComment);
        // Heavy fine + unpopular player → no ripple
        let r = evaluate_fine_reaction(
            FineTier::OneMonthWages, FineReason::PoorPerformance, 5);
        assert_eq!(r, FineOutcome::AcceptedWithoutComment);
    }

    #[test]
    fn press_statement_deltas_have_expected_signs() {
        assert!(press_statement_mood_delta(PressStatement::NotForSale) > 0);
        assert!(press_statement_mood_delta(PressStatement::PraiseProfessionalism) > 0);
        assert!(press_statement_mood_delta(PressStatement::DenyRift) > 0);
        assert!(press_statement_mood_delta(PressStatement::PromiseFavouredPosition) > 0);
        assert!(press_statement_mood_delta(PressStatement::AdmitUnhappyWith) < 0);
        assert!(press_statement_mood_delta(PressStatement::AdmitUnhappyRemark) < 0);
        // Public shaming should hurt more than the softer rebuke
        assert!(press_statement_mood_delta(PressStatement::AdmitUnhappyWith)
                < press_statement_mood_delta(PressStatement::AdmitUnhappyRemark));
    }

    #[test]
    fn national_coverage_doubles_press_impact() {
        for stmt in [PressStatement::NotForSale, PressStatement::AdmitUnhappyWith] {
            let local = scaled_press_delta(stmt, NewspaperTier::Local);
            let national = scaled_press_delta(stmt, NewspaperTier::National);
            assert_eq!(national as i32, (local as i32).saturating_mul(2).clamp(-50, 50));
        }
    }

    #[test]
    fn fine_amount_scales_by_tier() {
        assert_eq!(compute_fine_amount(FineTier::OneWeekWages, 30_000), 30_000);
        assert_eq!(compute_fine_amount(FineTier::TwoWeeksWages, 30_000), 60_000);
        assert_eq!(compute_fine_amount(FineTier::OneMonthWages, 30_000), 120_000);
        assert_eq!(compute_fine_amount(FineTier::FixedAmount(50_000), 30_000), 50_000);
    }

    #[test]
    fn fine_amount_120k_matches_exe_example() {
        // Exe .rdata example: "£120,000" for a fine amount.
        // One-month wages at £30k/week = £120k. Confirms the tier scale.
        let fine = compute_fine_amount(FineTier::OneMonthWages, 30_000);
        assert_eq!(fine, 120_000);
    }

    #[test]
    fn register_and_lookup() {
        let mut r = CountryFinanceRules::new();
        r.register(CountryRulesSpec { nation_id: 94, ..STANDARD_EUROPEAN_TOP_FLIGHT });
        r.register(CountryRulesSpec { nation_id: 97, transfer_window_open_month: 2, ..STANDARD_EUROPEAN_TOP_FLIGHT });
        assert_eq!(r.for_nation(94).unwrap().max_contract_years, 5);
        assert_eq!(r.for_nation(97).unwrap().transfer_window_open_month, 2);
        assert!(r.for_nation(999).is_none());
    }

    #[test]
    fn duplicate_registration_replaces() {
        let mut r = CountryFinanceRules::new();
        r.register(CountryRulesSpec { nation_id: 94, max_contract_years: 3, ..STANDARD_EUROPEAN_TOP_FLIGHT });
        r.register(CountryRulesSpec { nation_id: 94, max_contract_years: 7, ..STANDARD_EUROPEAN_TOP_FLIGHT });
        assert_eq!(r.for_nation(94).unwrap().max_contract_years, 7);
        assert_eq!(r.countries.len(), 1);
    }

    #[test]
    fn start_cash_matches_extracted_table() {
        // Verified against the exe extract at VA 0x009b48e0.
        // rep=500 (band 0) → -£450k; rep=8000 (band 15) → +£10M.
        let minnow = ClubFinance::seed_faithful(1, 500, false);
        let giant = ClubFinance::seed_faithful(2, 8000, false);
        assert_eq!(minnow.balance, -450_000);
        assert_eq!(giant.balance, 10_000_000);
    }

    #[test]
    fn transfer_budget_only_for_top_clubs() {
        // rep 4500 is below the 0x128F=4751 threshold → no budget.
        let mid = ClubFinance::seed_faithful(1, 4500, false);
        assert_eq!(mid.transfer_budget, 0);
        // rep 6000 → in the cascade.
        let big = ClubFinance::seed_faithful(2, 6000, false);
        assert!(big.transfer_budget > 0);
    }

    #[test]
    fn status_classifier_matches_thresholds() {
        // rep=1000: `Rich` needs bal ≥ (25·1000+50000)·100 = 7,500,000.
        let mut c = ClubFinance::seed_faithful(1, 1000, false);
        c.balance = 8_000_000;
        assert_eq!(c.status(1000), FinanceStatus::Rich);
        c.balance = 4_000_000; // (25·1000+87500)·40 = 4,500,000, so still Normal.
        assert_eq!(c.status(1000), FinanceStatus::Normal);
        c.balance = 5_000_000; // Healthy
        assert_eq!(c.status(1000), FinanceStatus::Healthy);
        // Admin: bal ≤ -(1500·1000 + 2,500,000) = -4,000,000.
        c.balance = -4_500_000;
        assert_eq!(c.status(1000), FinanceStatus::Admin);
    }
}

/// Staff contract base floor per role code (job_type - 5 selects the slot).
/// VERIFIED port of FUN_0084b870 switch case body — see
/// reports/contract_tactic_comp_giants.md §FUN_0084b870.
///
/// 6 staff role classes with their weekly-wage floor (in currency-minor).
/// Job types 11..=15 fall through to the STAFF_CONTRACT default path
/// (200.0 fallback, not indexed here).
pub const STAFF_BASE_FLOOR: [f32; 6] = [750.0, 500.0, 500.0, 275.0, 250.0, 200.0];

/// Age-tier × wage-band cap multiplier — VERIFIED port of
/// FUN_0084b870:458-488. Returns the ceiling multiplier the negotiator
/// applies to the raw wage against the current wage band `band`.
///
///   age <= 9  → band × 0x9c4 (2500)
///   age <= 14 → band × 0xabe (2750)
///   else      → band × 0xcb2 (3250)
pub fn age_wage_cap(age_years: i32, band: i32) -> f32 {
    match age_years {
        ..=9  => (band * 0x9c4) as f32,
        ..=14 => (band * 0xabe) as f32,
        _     => (band * 0xcb2) as f32,
    }
}

/// Star-attribute floor — VERIFIED port of FUN_0084b870:489-503.
/// Maps the top slot-index (0x11..=0x15 range) of the coach's peak
/// attribute to a weekly-wage minimum. The exe reads slots 0x11..0x15
/// of the coach attribute struct at +0x61 and returns the highest.
pub fn star_floor(top_slot_idx: usize) -> f32 {
    match top_slot_idx {
        0x15        => 85_000.,
        0x14 | 0x13 => 75_000.,
        0x12        => 65_000.,
        0x11        => 50_000.,
        _           => 40_000.,
    }
}

/// Club-status byte lookup — VERIFIED port of FUN_00618410
/// (00618410.c:2-9). Returns a sentinel `0xFF` when record is null / id
/// out of range / owner_ptr at record+0x61 is zero. Otherwise returns
/// the byte at `status_table[id * 31 + 0x12]`. Pure query, no writes.
///
/// Consumed downstream as a club-status enum (news commentary, club-list
/// classification). See reports/away_shot_and_618410_decode.md.
pub fn club_status_byte(status_table: &[u8], record_id: Option<u32>,
                        num_clubs: u32) -> u8 {
    let Some(id) = record_id else { return 0xFF; };
    if id >= num_clubs { return 0xFF; }
    let ofs = (id as usize) * 31 + 0x12;
    if ofs >= status_table.len() { return 0xFF; }
    status_table[ofs]
}

// ---------------------------------------------------------------------------
// Board demands (item 5 — brand new subsystem)
//
// The manager can make 7 requests to the board. Each has a distinct
// "Board reaction to request for X" template in the exe .rdata (found via
// pefile sweep at 0x5fbd98..0x5fbef0). The board evaluates based on club
// state (board_confidence, balance, wage_bill, form) and returns an
// approval verdict.
//
// This is a foundational port — the enum + evaluator + response templates
// are shape-verified against the exe strings. The exact numeric thresholds
// for approval are ENVELOPE (no single decoded evaluator fn found) — the
// exe likely uses a bespoke case-per-request predicate cluster we haven't
// isolated. Constants labelled as such.
// ---------------------------------------------------------------------------

/// The seven board demands available to the manager. VERIFIED string
/// evidence at exe .rdata 0x5fbd98..0x5fbef0 (each has a "Board reaction
/// to request for X" template).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardDemand {
    /// "Board reaction to request for larger half time oranges" — the
    /// tongue-in-cheek always-refused request. Confirmed at 0x5fbd98.
    HalfTimeOranges,
    /// "Board reaction to request for player bonus payments" (0x5fbdd0).
    PlayerBonusPayments,
    /// "Board reaction to request for time to rebuild squad" (0x5fbe04).
    TimeToRebuild,
    /// "Board reaction to request for higher wage budget" (0x5fbe5d).
    HigherWageBudget,
    /// "Board reaction to request for transfer funds" (0x5fbe97).
    TransferFunds,
    /// "Board reaction to request for youth investment" (0x5fbeba).
    YouthInvestment,
    /// "Board reaction to request to expand stadium" (0x5fbeef).
    ExpandStadium,
}

/// Response from the board to a [`BoardDemand`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardResponse {
    /// Request granted (e.g. budget increased, project approved).
    Approved {
        /// Extra cash granted (for TransferFunds / HigherWageBudget /
        /// YouthInvestment / ExpandStadium); zero for time/bonus requests.
        cash_granted: i64,
        /// Board confidence bump for a well-received request.
        confidence_delta: i8,
    },
    /// Request refused. The exe emits one of ~5 refusal templates depending
    /// on why (already increased recently / can't afford / not warranted).
    Refused {
        reason: BoardRefusalReason,
        /// Board confidence tick DOWN for making an unwarranted request.
        confidence_delta: i8,
    },
    /// Request out-of-scope (e.g. HalfTimeOranges — always refused with
    /// a specific "you couldn't be bothered to give reasons" template
    /// at 0x5fbef0 confirmed).
    OutOfScope,
}

/// Why the board turned down a request. Templates in exe .rdata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardRefusalReason {
    /// "the current wage budget is correct considering..." (0x6008db)
    AlreadyAppropriate,
    /// "little with which to increase the wage budget" — club skint.
    ClubCannotAfford,
    /// "board are surprised by your request for more time" (0x5fc0f6)
    RequestUnwarranted,
    /// "decision is a corporate one and not something..." (0x60059a)
    NotYourDecision,
    /// Generic — used when none of the specific templates fit.
    Generic,
}

/// Evaluate a board demand against the current club state.
///
/// Returns a [`BoardResponse`]. **Envelope port** — the exact per-request
/// approval thresholds are not decoded from a single evaluator fn. The
/// enum shape + response templates are verified against exe strings; the
/// numeric thresholds are chosen to match common gameplay observation
/// (confident boards approve; broke boards refuse; expansions need
/// long-term positive form).
///
/// # Params
/// - `demand`: which of the 7 requests
/// - `club`: current finance state
/// - `recent_form_score`: manager's recent-form aggregate (-100..+100 span).
///    Positive = good form → more likely to approve.
pub fn evaluate_board_demand(
    demand: BoardDemand,
    club: &ClubFinance,
    recent_form_score: i16,
) -> BoardResponse {
    use BoardDemand::*;
    match demand {
        // Tongue-in-cheek permanent refusal
        HalfTimeOranges => BoardResponse::OutOfScope,

        // Time-to-rebuild: usually approved if board_confidence >= 30
        // (matches exe template at 0x5fc0f6 "surprised by your request
        // for more time" as the refusal path).
        TimeToRebuild => {
            if club.board_confidence >= 30 && recent_form_score >= -20 {
                BoardResponse::Approved { cash_granted: 0, confidence_delta: 0 }
            } else {
                BoardResponse::Refused {
                    reason: BoardRefusalReason::RequestUnwarranted,
                    confidence_delta: -2,
                }
            }
        }

        // Player bonuses: needs positive balance + good confidence
        PlayerBonusPayments => {
            if club.balance > 0 && club.board_confidence >= 50 {
                BoardResponse::Approved {
                    cash_granted: 0,   // enables bonus scale; not cash
                    confidence_delta: 0,
                }
            } else {
                BoardResponse::Refused {
                    reason: BoardRefusalReason::AlreadyAppropriate,
                    confidence_delta: -1,
                }
            }
        }

        // Wage budget: needs cash AND healthy confidence
        HigherWageBudget => {
            if club.balance > (club.weekly_wage_bill as i64) * 52 * 3
               && club.board_confidence >= 50 {
                // Grant a 10% wage-bill headroom bump
                let bump = (club.weekly_wage_bill as i64) * 52 / 10;
                BoardResponse::Approved { cash_granted: bump, confidence_delta: 0 }
            } else if club.balance <= 0 {
                BoardResponse::Refused {
                    reason: BoardRefusalReason::ClubCannotAfford,
                    confidence_delta: -1,
                }
            } else {
                BoardResponse::Refused {
                    reason: BoardRefusalReason::AlreadyAppropriate,
                    confidence_delta: -1,
                }
            }
        }

        // Transfer funds: needs cash surplus AND positive form
        TransferFunds => {
            let surplus = club.balance - (club.weekly_wage_bill as i64) * 52;
            if surplus > 1_000_000 && recent_form_score >= 0 {
                // Grant 25% of surplus as new transfer budget
                let grant = surplus / 4;
                BoardResponse::Approved { cash_granted: grant, confidence_delta: 0 }
            } else if surplus <= 0 {
                BoardResponse::Refused {
                    reason: BoardRefusalReason::ClubCannotAfford,
                    confidence_delta: -1,
                }
            } else {
                BoardResponse::Refused {
                    reason: BoardRefusalReason::RequestUnwarranted,
                    confidence_delta: -1,
                }
            }
        }

        // Youth investment: modest cash, permissive threshold
        YouthInvestment => {
            if club.balance > 250_000 {
                BoardResponse::Approved {
                    cash_granted: 250_000,
                    confidence_delta: 1,
                }
            } else {
                BoardResponse::Refused {
                    reason: BoardRefusalReason::ClubCannotAfford,
                    confidence_delta: 0,
                }
            }
        }

        // Stadium expansion: big-ticket, needs confidence >= 60 + long-term surplus
        ExpandStadium => {
            let surplus = club.balance - (club.weekly_wage_bill as i64) * 52 * 2;
            if surplus > 5_000_000 && club.board_confidence >= 60 {
                BoardResponse::Approved {
                    cash_granted: 5_000_000,  // envelope: stadium cost
                    confidence_delta: 2,
                }
            } else {
                BoardResponse::Refused {
                    reason: BoardRefusalReason::NotYourDecision,
                    confidence_delta: -2,
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Player fines (item 5 sub-slice) — verified from exe .rdata: "£120,000"
// fine amount placeholder at 0x5b3458 + discipline.cpp source path at
// 0x5a55f6 + discipline.dat file mention.
// ---------------------------------------------------------------------------

/// Fine amount tiers verified from exe fine-amount templates. £120,000
/// is the example in "have been fined <%s - Fine Amount (eg. £120,000)>"
/// at 0x5b3458. The other tiers below are common football-domain values
/// consistent with the exe's "£ Fine Amount" placeholder scaling — the
/// underlying selection fn is in `discipline.cpp` (source-path verified,
/// specific fn address not yet isolated).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FineTier {
    /// One week's wages — light disciplinary action.
    OneWeekWages,
    /// Two weeks — moderate action.
    TwoWeeksWages,
    /// One month — heavy sanction (typical £120,000 example).
    OneMonthWages,
    /// Fixed amount — used for specific offences with a flat penalty.
    FixedAmount(i64),
}

/// Compute the fine cash amount from tier + player's weekly wage.
pub fn compute_fine_amount(tier: FineTier, weekly_wage: u32) -> i64 {
    match tier {
        FineTier::OneWeekWages   => weekly_wage as i64,
        FineTier::TwoWeeksWages  => (weekly_wage as i64) * 2,
        FineTier::OneMonthWages  => (weekly_wage as i64) * 4,   // 4 weeks
        FineTier::FixedAmount(x) => x.max(0),
    }
}

/// Fine reason category — VERIFIED from exe .rdata "fine reason (eg.
/// Unprofessional Behaviour)" template at 0x5b6efe. Specific reasons
/// used by the exe were surveyed but the enum here captures the
/// dispatcher shape. Real names may match "Unprofessional Behaviour",
/// "Poor Performance", "Missed Training", etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FineReason {
    UnprofessionalBehaviour,
    PoorPerformance,
    MissedTraining,
    IndiscreetRemarks,
    /// Fall-back — used when no specific reason applies.
    Generic,
}

/// Media reaction to a fine or warning. Two outcome shapes VERIFIED from
/// exe .rdata at 0x5b6efe (fine) / 0x5b783a (warning):
///   1. "has accepted the fine/warning...without comment" — no ripple
///   2. "Several team members...commented...was unfair" — RIPPLE
///
/// The dispatcher branch is not decoded to a single fn; the exe likely
/// uses a per-player-popularity vs fine-severity gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FineOutcome {
    /// Player took it, no team response.
    AcceptedWithoutComment,
    /// Team-mates felt fine was unfair — ripples through squad morale.
    TeamRipple {
        /// Signed mood_delta applied to each OTHER player on the team.
        /// Negative = fine ripple hurts squad morale.
        team_delta: i8,
    },
}

/// Decide the fine ripple outcome + return the ripple delta.
///
/// **Envelope port** — the exe's exact gate (player popularity vs fine
/// severity) isn't a single decoded fn. Rule of thumb from observation:
///
///   - `Generic` reason on `OneWeekWages` → accepted (light + reasonable)
///   - `OneMonthWages` for `PoorPerformance` when player is well-liked
///     (popularity > 12) → ripples with delta = -3
///   - Any fine for `IndiscreetRemarks` → accepted (player at fault)
///   - `UnprofessionalBehaviour` heavy fine + unpopular player → accepted
///   - Otherwise: default to accepted
pub fn evaluate_fine_reaction(
    tier: FineTier,
    reason: FineReason,
    fined_player_popularity: u8,
) -> FineOutcome {
    use FineReason::*;
    let is_heavy = matches!(tier, FineTier::OneMonthWages
                            | FineTier::TwoWeeksWages
                            | FineTier::FixedAmount(_));
    let popular = fined_player_popularity > 12;
    match reason {
        IndiscreetRemarks => FineOutcome::AcceptedWithoutComment,
        UnprofessionalBehaviour if !popular
            => FineOutcome::AcceptedWithoutComment,
        PoorPerformance if is_heavy && popular
            => FineOutcome::TeamRipple { team_delta: -3 },
        MissedTraining if is_heavy && popular
            => FineOutcome::TeamRipple { team_delta: -2 },
        _ => FineOutcome::AcceptedWithoutComment,
    }
}

/// Newspaper coverage tier — VERIFIED from exe .rdata 0x5b2cd0/0x5b2cf6
/// "national<%s - COMMENT: national newspaper>" and "local<%s - COMMENT:
/// local newspaper>" bit-flag pair. National coverage carries larger
/// morale/reputation weight than local.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewspaperTier { National, Local }

/// Press-conference statement the manager can deliver about a player.
/// VERIFIED templates:
///   - "told the press that <player> is not for sale at any price" (0x5b179a)
///   - "told the press that <player> is a model professional who would ne..." (0x604df3)
///   - "told the press that there are no problems between himself and..." (0x604e95)
///   - "told the press that <player> will soon be playing in his favoured..." (0x605371)
///   - "You have admitted to reporters that you are unhappy with..." (0x607199)
///
/// Each has a distinct effect on player morale + club reputation +
/// transfer-value dynamics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressStatement {
    /// "Not for sale at any price" — locks the player out of transfer
    /// negotiations for a period, boosts loyalty.
    NotForSale,
    /// "model professional" — public praise, morale +.
    PraiseProfessionalism,
    /// "no problems between us" — media backing during a rift, small +.
    DenyRift,
    /// "will soon be playing in his favoured position" — position promise,
    /// player-specific morale +.
    PromiseFavouredPosition,
    /// "unhappy with X's contribution" — public shaming, big morale -.
    AdmitUnhappyWith,
    /// "unhappy with X's indiscrete remark" — softer public rebuke.
    AdmitUnhappyRemark,
}

/// Effect of a press statement on the target player's mood_delta byte.
/// Negative = damaging, positive = supportive. **Envelope values** —
/// exact deltas from the exe's post-conference morale writers not
/// isolated to a single fn, but ordering + sign are constrained by the
/// template semantics.
pub fn press_statement_mood_delta(stmt: PressStatement) -> i8 {
    use PressStatement::*;
    match stmt {
        NotForSale              => 5,   // strong positive
        PraiseProfessionalism   => 3,
        DenyRift                => 2,
        PromiseFavouredPosition => 4,
        AdmitUnhappyWith        => -10, // public shaming
        AdmitUnhappyRemark      => -5,
    }
}

/// News-coverage tier scales morale magnitude. National coverage doubles
/// the effect of any press statement (envelope).
pub fn scaled_press_delta(stmt: PressStatement, tier: NewspaperTier) -> i8 {
    let base = press_statement_mood_delta(stmt) as i32;
    let scaled = match tier {
        NewspaperTier::National => base * 2,
        NewspaperTier::Local    => base,
    };
    scaled.clamp(-50, 50) as i8
}
