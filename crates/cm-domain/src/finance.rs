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
}

impl ClubFinance {
    /// Seed a new club's finance from its reputation. Higher rep → more
    /// starting cash and higher wage bill. Placeholder until the club_comp
    /// record's own balance field is decoded and wired through.
    pub fn seed_from_reputation(club_id: u32, reputation: u16) -> Self {
        let rep = reputation as i64;
        let balance = rep * rep * 500;
        let weekly_wage_bill = (rep * rep / 5).max(1000) as u32;
        let transfer_budget = balance / 5;
        Self {
            club_id,
            balance,
            weekly_wage_bill,
            transfer_budget,
            months_in_the_red: 0,
        }
    }
}

/// The finance book — indexed by club id.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FinanceBook {
    pub clubs: Vec<ClubFinance>,
    pub rules: CountryFinanceRules,
}

impl FinanceBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_from_clubs(clubs: &[crate::DomainOpaqueRecord]) -> Self {
        let mut cf = Vec::with_capacity(clubs.len());
        for rec in clubs {
            let cv = crate::ClubView::new(rec);
            cf.push(ClubFinance::seed_from_reputation(cv.id(), cv.reputation()));
        }
        Self { clubs: cf, rules: CountryFinanceRules::new() }
    }

    /// Weekly wage payment tick — subtracts every club's `weekly_wage_bill`
    /// from its `balance`. Called by the tick every 7 days.
    pub fn pay_weekly_wages(&mut self) {
        for c in &mut self.clubs {
            c.balance -= c.weekly_wage_bill as i64;
        }
    }

    /// End-of-month bookkeeping — increments `months_in_the_red` for any
    /// club still negative, resets for any that returned to positive.
    /// Placeholder for the more elaborate administration/receivership logic
    /// the exe's rules layer implements.
    pub fn end_of_month(&mut self) {
        for c in &mut self.clubs {
            if c.balance < 0 {
                c.months_in_the_red = c.months_in_the_red.saturating_add(1);
            } else {
                c.months_in_the_red = 0;
            }
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
    fn seed_scales_with_reputation() {
        // Small: rep=100, big: rep=200. Both above the wage-bill floor
        // so the delta actually shows.
        let c_small = ClubFinance::seed_from_reputation(1, 100);
        let c_big = ClubFinance::seed_from_reputation(2, 200);
        assert!(c_big.balance > c_small.balance);
        assert!(c_big.weekly_wage_bill > c_small.weekly_wage_bill);
        assert!(c_big.transfer_budget > c_small.transfer_budget);
    }

    #[test]
    fn weekly_wages_deplete_balance() {
        let mut b = FinanceBook::new();
        b.clubs.push(ClubFinance {
            club_id: 1, balance: 10_000, weekly_wage_bill: 1_000,
            transfer_budget: 0, months_in_the_red: 0,
        });
        b.pay_weekly_wages();
        assert_eq!(b.for_club(1).unwrap().balance, 9_000);
        for _ in 0..10 { b.pay_weekly_wages(); }
        assert_eq!(b.for_club(1).unwrap().balance, -1_000);
    }

    #[test]
    fn end_of_month_tracks_red() {
        let mut b = FinanceBook::new();
        b.clubs.push(ClubFinance {
            club_id: 1, balance: -100, weekly_wage_bill: 0,
            transfer_budget: 0, months_in_the_red: 0,
        });
        b.end_of_month();
        b.end_of_month();
        assert_eq!(b.for_club(1).unwrap().months_in_the_red, 2);
        b.clubs[0].balance = 500;
        b.end_of_month();
        assert_eq!(b.for_club(1).unwrap().months_in_the_red, 0);
    }
}
