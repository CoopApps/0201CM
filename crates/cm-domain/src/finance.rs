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
//! * `weekly_wage_bill` — sum of every squad member's weekly wage.
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Chairman/owner group id (club record +0x69 in the exe). Clubs sharing
    /// the same group id share a chairman → the rich-sibling→broke-sibling
    /// £20M rescue mechanism (`FUN_00586ec0`:56-114) fires between them.
    /// `None` when unset — the mechanism no-ops for clubs without an owner
    /// group. Populated at seed time from `ClubView::chairman_group()` if
    /// non-zero, else `None`.
    #[serde(default)]
    pub chairman_group: Option<u32>,
    /// Chairman handout already fired this year? (`param_2+0x6d` in the exe.)
    /// Cleared once per game-year in `end_of_year_rollover`; set by
    /// `chairman_handouts` when a £20M transfer completes.
    #[serde(default)]
    pub chairman_handout_used: bool,
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
        Self::seed_from(club_id, reputation, stadium_flag, 0)
    }

    /// Seed variant that takes the CLUB-SPECIFIC shipped cash from club.dat's
    /// `+0x65` field. If non-zero, that value wins; otherwise the reputation
    /// table is used as fallback (matches the exe's own behaviour for clubs
    /// with no pre-set finance state).
    pub fn seed_from(club_id: u32, reputation: u16, stadium_flag: bool, shipped_cash: i32) -> Self {
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

        // Determine if this club boots in administration (SWFC ships with
        // -£14M cash and rep 6000 → threshold -£11.5M → immediate admin).
        let bootstrap = ClubFinance {
            club_id, balance, weekly_wage_bill: 0, transfer_budget,
            months_in_the_red: 0,
            board_confidence: initial_board_confidence(rep),
            in_administration: false,
            month_wages: 0, month_gate: 0, month_tv_prize: 0,
            chairman_group: None, chairman_handout_used: false,
        };
        let boots_in_admin = matches!(bootstrap.status(reputation), FinanceStatus::Admin);
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
            board_confidence: initial_board_confidence(rep),
            month_wages: 0,
            month_gate: 0,
            month_tv_prize: 0,
            chairman_group: None,
            chairman_handout_used: false,
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

/// Real `FUN_005803d0`:102-110 seed for board confidence at finance+0x166.
fn initial_board_confidence(rep: i64) -> u8 {
    // The `cVar3` there is a chairman-existence byte; without a chairman it's
    // 10. `status ∈ {1,2}` → 5, else 15. We approximate at boot: rep tiers.
    if rep < 2000 { 15 } else if rep < 5000 { 10 } else { 5 }
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
}

impl FinanceBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_from_clubs(clubs: &[crate::DomainOpaqueRecord]) -> Self {
        // Faithful seed (kill #8a): real START_CASH table + rep/status budget
        // cascade from FUN_005803d0. Records the club's reputation alongside
        // for later status classification.
        let mut cf = Vec::with_capacity(clubs.len());
        let mut reps = std::collections::BTreeMap::new();
        let mut att = std::collections::BTreeMap::new();
        for rec in clubs {
            let cv = crate::ClubView::new(rec);
            // Use SHIPPED cash from the club record (+0x65) when non-zero —
            // otherwise fall back to the START_CASH-by-reputation table. This
            // is what makes SWFC boot as bankrupt (their shipped cash is
            // -£14M), same as the game itself.
            let mut cfinance = ClubFinance::seed_from(cv.id(), cv.reputation(), false, cv.cash());
            cfinance.chairman_group = cv.chairman_group();
            cf.push(cfinance);
            att.insert(cv.id(), (cv.attendance_average(), cv.attendance_minimum(), cv.attendance_maximum()));
            reps.insert(cv.id(), cv.reputation());
        }
        Self { clubs: cf, rules: CountryFinanceRules::new(), club_reputation: reps, club_attendance: att }
    }

    /// Chairman £20M handout — port of `FUN_00586ec0`:56-114.
    ///
    /// Fires at the start of the weekly-finance tick, BEFORE wages. When a
    /// rich club (balance > £35M) has a chairman (`chairman_group.is_some()`)
    /// and hasn't fired this handout yet (`!chairman_handout_used`):
    ///
    /// 1. Scan every other club in the SAME chairman group.
    ///    - If a sibling club is `Admin` or `InTheRed` (its `+0x165` flag was
    ///      1 or 2 in the exe) — transfer £20M from rich → poor. Mark rich as
    ///      "used", clear sibling's used-bit (their turn's over). This is the
    ///      "chairman rescues struggling sister club" event (news code 4/5).
    ///    - If a sibling exists but has no finance record — chairman still
    ///      takes £20M from the rich club (goes to "personal use"). News 4.
    /// 2. If no sibling found — chairman still drains £20M from the rich club.
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
    pub fn chairman_handouts(&mut self) {
        const HANDOUT: i64 = 20_000_000;
        const RICH_THRESHOLD: i64 = 35_000_000;
        // Snapshot who is rich enough this tick.
        let candidates: Vec<(u32, u32)> = self
            .clubs
            .iter()
            .filter(|c| !c.chairman_handout_used
                && c.balance > RICH_THRESHOLD
                && c.chairman_group.is_some())
            .map(|c| (c.club_id, c.chairman_group.unwrap()))
            .collect();
        for (rich_id, group) in candidates {
            // Find first sibling in the same group whose handout bit IS set
            // (matches the exe's `piVar15 != param_2 && +0x6d != 0` filter,
            // i.e. "someone who has already had their yearly turn"; per the
            // ported decompile the game reads this as "eligible to receive").
            let sibling: Option<u32> = self.clubs.iter()
                .find(|c| c.club_id != rich_id
                    && c.chairman_group == Some(group)
                    && c.chairman_handout_used)
                .map(|c| c.club_id);
            match sibling {
                None => {
                    // No sibling found — chairman personal use (0058_6ff5).
                    if let Some(r) = self.clubs.iter_mut().find(|c| c.club_id == rich_id) {
                        r.balance = r.balance.saturating_sub(HANDOUT);
                        r.chairman_handout_used = true;
                    }
                }
                Some(sib_id) => {
                    // Sibling in Admin or InTheRed → rescue. Otherwise still
                    // drain the rich club (news code 4 only).
                    let sib_is_stressed = self.clubs.iter()
                        .find(|c| c.club_id == sib_id)
                        .map(|c| {
                            let rep = self.club_reputation.get(&sib_id).copied().unwrap_or(1000);
                            matches!(c.status(rep), FinanceStatus::Admin | FinanceStatus::InTheRed)
                        })
                        .unwrap_or(false);
                    if let Some(r) = self.clubs.iter_mut().find(|c| c.club_id == rich_id) {
                        r.balance = r.balance.saturating_sub(HANDOUT);
                        r.chairman_handout_used = true;
                    }
                    if sib_is_stressed {
                        if let Some(s) = self.clubs.iter_mut().find(|c| c.club_id == sib_id) {
                            s.balance = s.balance.saturating_add(HANDOUT);
                            s.chairman_handout_used = false;
                            // Admin exit (if crossing threshold now):
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

    /// Yearly rollover for the chairman-handout eligibility bit (clears the
    /// exe's `param_2+0x6d` for every club so next season's handout can fire).
    pub fn reset_yearly_chairman_flags(&mut self) {
        for c in &mut self.clubs {
            c.chairman_handout_used = false;
        }
    }

    /// Weekly wage payment tick (kill #8b) — port of `FUN_00586ec0` tail
    /// (:363-421). Three revenue tiers by balance-vs-reputation, RNG-banded
    /// per-rep wage draw; deducted from balance and accumulated into the
    /// month_wages ledger. Skint clubs (balance < rep×3000) pay no wages that
    /// week (matches the exe's `return` at the bottom of the cascade).
    pub fn pay_weekly_wages(&mut self) {
        // Chairman handout fires FIRST (FUN_00586ec0:56-114 sits above the
        // wage cascade in the exe).
        self.chairman_handouts();
        let mut rng = crate::match_engine_exe::MatchRng::new(0x0058_6ec0);
        for c in &mut self.clubs {
            let rep = self.club_reputation.get(&c.club_id).copied().unwrap_or(1000) as i64;
            let rep_i32 = rep as i32;
            // Compute weekly wage draw by tier (FUN_00586ec0:363-421).
            let bal = c.balance;
            let top_gate = (rep * 8000).max(500_000);
            let weekly = if bal >= top_gate {
                // Top tier — rand(0x1F5) + 2000 (or 1500 under 4000 rep).
                if rep < 4000 {
                    (rng.range(0x1F5) as i64 + 1500) * rep
                } else {
                    (rng.range(0x1F5) as i64 + 2000) * rep
                }
            } else if bal >= rep * 6000 {
                if rep < 4000 {
                    (rng.range(0x1F5) as i64 + 1000) * rep
                } else {
                    (rng.range(0x1F5) as i64 + 1500) * rep
                }
            } else if bal >= rep * 3000 {
                if rep < 4000 { rep * 500 } else { rep * 750 }
            } else {
                0 // skint — no wages this week
            };
            let _ = rep_i32;
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
        for c in &mut self.clubs {
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
