//! Australian salary-cap rule — a port of `australia_rules.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\transfer\australia_rules.cpp`, VA
//! `0x004130c0..0x004143cf`). Function decode + names:
//! `reports/carve_rename_map.json` (`australia_rules.cpp`).
//!
//! Unlike the Argentine registration rule, the Australian rule is a **salary
//! cap**. `ausrules_apply_salary_cap` (`0x00413340`) tracks a per-club spend
//! counter (`[club+0x61][+0xd]`): each contribution adds **5%** of an amount
//! (the double `0.05` at `0x00955888`) and the total is **clamped to `0x2710`
//! = 10000** (the cap). `ausrules_enforce_salary_cap_league` (`0x004139f0`)
//! sweeps the league applying it.
//!
//! This ports the rule as a tested, self-contained constraint. Flagged: the
//! headless model has no club-finances / transfer-spend system to feed it yet,
//! so it is a ready API rather than an active gate — the same honest stance as
//! the Argentine [`crate::arg_rules`] max-signings check.

use serde::{Deserialize, Serialize};

/// The salary-cap ceiling (`0x2710`), the clamp in `ausrules_apply_salary_cap`.
pub const AUS_SALARY_CAP: i32 = 0x2710;
/// The per-contribution multiplier (the double `0.05` at `0x00955888`).
pub const AUS_SALARY_CAP_INCREMENT: f64 = 0.05;

/// Per-club salary-cap spend tracker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AusSalaryCap {
    /// Accumulated (capped) spend per club id.
    pub spend: std::collections::BTreeMap<u32, i32>,
}

impl AusSalaryCap {
    /// `ausrules_apply_salary_cap` (`0x00413340`): add `amount * 0.05` to the
    /// club's counter and clamp to [`AUS_SALARY_CAP`]. Returns the new total.
    pub fn apply(&mut self, club_id: u32, amount: i32) -> i32 {
        let inc = (amount as f64 * AUS_SALARY_CAP_INCREMENT) as i32;
        let e = self.spend.entry(club_id).or_insert(0);
        *e = (*e + inc).min(AUS_SALARY_CAP).max(0);
        *e
    }

    /// The club's current capped spend.
    pub fn spend_of(&self, club_id: u32) -> i32 {
        self.spend.get(&club_id).copied().unwrap_or(0)
    }

    /// Head-room left under the cap.
    pub fn remaining(&self, club_id: u32) -> i32 {
        AUS_SALARY_CAP - self.spend_of(club_id)
    }

    /// Whether the club has hit the cap.
    pub fn at_cap(&self, club_id: u32) -> bool {
        self.spend_of(club_id) >= AUS_SALARY_CAP
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_five_percent_and_clamps_to_cap() {
        let mut c = AusSalaryCap::default();
        // 5% of 100000 = 5000.
        assert_eq!(c.apply(1, 100_000), 5000);
        assert_eq!(c.remaining(1), 5000);
        assert!(!c.at_cap(1));
        // Another 5% of 100000 → 10000, clamped exactly at the cap.
        assert_eq!(c.apply(1, 100_000), AUS_SALARY_CAP);
        // Further contributions cannot exceed the cap.
        assert_eq!(c.apply(1, 100_000), AUS_SALARY_CAP);
        assert!(c.at_cap(1));
        assert_eq!(c.remaining(1), 0);
        // Other clubs are independent.
        assert_eq!(c.spend_of(2), 0);
    }
}
