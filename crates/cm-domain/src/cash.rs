//! Money / currency value type — a port of `cash.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\cash.cpp`, VA `0x00440070..0x0044339f`).
//! Function decode + names: `reports/carve_rename_map.json` (`cash.cpp`).
//!
//! The exe stores monetary amounts as integers and, for display, converts them
//! through a per-currency scale/divisor (`cash_scale_value` `0x00440070`) and
//! formats them with a currency symbol + thousands separators (`cash_format`
//! `0x00442500`). One of 25 currencies is selected by `cash_set_currency`
//! (`0x00440240`), which configures the global scale (`0xac5830`) and divisor
//! (`0xac5748`).
//!
//! This ports the value type and the observable formatting. The 25-currency
//! rate/symbol table is runtime-initialised `.data` in the exe (built by
//! `cash_set_currency`); extracting it is a follow-up. The default here is
//! pounds sterling — the game's base currency.
//!
//! Foundation utility: nothing in the headless model spends money yet, but a
//! club-finances / transfer / wages system will use this as its money type.

use serde::{Deserialize, Serialize};

/// A monetary amount in base units (whole currency units, e.g. pounds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Money(pub i64);

impl Money {
    pub const ZERO: Money = Money(0);

    pub fn amount(self) -> i64 {
        self.0
    }

    /// Group the integer part with thousands separators — the core of
    /// `cash_format` (`0x00442500`). E.g. `1234567 -> "1,234,567"`.
    pub fn grouped(self) -> String {
        let neg = self.0 < 0;
        let digits = self.0.unsigned_abs().to_string();
        let bytes = digits.as_bytes();
        let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
        if neg {
            out.push('-');
        }
        let n = bytes.len();
        for (i, &b) in bytes.iter().enumerate() {
            if i > 0 && (n - i) % 3 == 0 {
                out.push(',');
            }
            out.push(b as char);
        }
        out
    }

    /// Format with a currency symbol prefix (default "£"), as `cash_format`
    /// renders it — e.g. `Money(1_500_000).format("£") == "£1,500,000"`.
    pub fn format(self, symbol: &str) -> String {
        if self.0 < 0 {
            format!("-{symbol}{}", Money(-self.0).grouped())
        } else {
            format!("{symbol}{}", self.grouped())
        }
    }

    /// Format in pounds sterling — the game's base currency.
    pub fn format_gbp(self) -> String {
        self.format("£")
    }
}

impl std::ops::Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Money {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        Money(self.0 - rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_grouping() {
        assert_eq!(Money(0).grouped(), "0");
        assert_eq!(Money(999).grouped(), "999");
        assert_eq!(Money(1_000).grouped(), "1,000");
        assert_eq!(Money(1_234_567).grouped(), "1,234,567");
        assert_eq!(Money(-1_500_000).grouped(), "-1,500,000");
    }

    #[test]
    fn formats_with_symbol() {
        assert_eq!(Money(1_500_000).format_gbp(), "£1,500,000");
        assert_eq!(Money(50_000).format("€"), "€50,000");
        assert_eq!(Money(-2_000).format("£"), "-£2,000");
    }

    #[test]
    fn arithmetic() {
        assert_eq!(Money(1000) + Money(500), Money(1500));
        assert_eq!(Money(1000) - Money(1500), Money(-500));
    }
}
