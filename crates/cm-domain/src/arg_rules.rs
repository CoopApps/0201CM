//! Argentine transfer-registration rules — a port of `argentina_rules.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\transfer\argentina_rules.cpp`, VA
//! `0x0040a680..0x0040ab3f`). Function decode + names:
//! `reports/carve_rename_map.json` (`argentina_rules.cpp`).
//!
//! The Argentine registration window caps how many players a club may sign in
//! it. The exe's own text (`0x00974984`) states the rule verbatim: *"clubs are
//! now allowed to buy players again. A maximum of two players can be signed
//! between today and the deadline in Febuary."* [sic]. A per-club signing
//! counter (`DAT_00dc722c[+0x918][club*0x10]`) is reset when the window opens
//! (`argrules_on_window_date` `0x0040a770`), incremented per signing
//! (`argrules_increment_signing_counter` `0x0040a870`), and checked to block a
//! third signing (`argrules_check_signing_allowed` `0x0040a8c0`: `>= 2` →
//! status `0xe`).
//!
//! This module ports that rule as a tested, self-contained constraint the
//! future transfer system calls. The window-open news is emitted on the tick.
//!
//! ## Fidelity notes
//! * **Limit / counter / check** — exact: reset on open, `+1` per signing,
//!   blocked at 2.
//! * **News text** — the exe string verbatim (including its "Febuary" typo), so
//!   the game shows the same line.
//! * **Window date** — the exe derives it from the rule table via
//!   `argrules_resolve_rule_date` (`0x0040a830`) + the season date-builder
//!   `FUN_00533b50` (not yet decoded); a documented January open date stands in.

use serde::{Deserialize, Serialize};

use crate::GameDate;

/// "A maximum of two players can be signed" (`argrules_check_signing_allowed`
/// blocks at `>= 2`).
pub const MAX_SIGNINGS_PER_WINDOW: u8 = 2;

/// The window-open news, rendered from the exe template `0x00974984` with the
/// nationality adjective "Argentine". Kept verbatim (the exe's "Febuary" typo
/// included) so the ported game shows the same headline.
pub const WINDOW_OPEN_NEWS: &str = "Argentine clubs are now allowed to buy players again. A maximum of two players can be signed between today and the deadline in Febuary.";

/// The state of the Argentine registration window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgTransferRuleState {
    /// The date the window opens (the "allowed to buy again" day).
    pub open_date: GameDate,
    /// Whether the window-open news has been emitted yet this window.
    pub announced: bool,
    /// Per-club signings made this window (`DAT_00dc722c[+0x918]`).
    pub signings: std::collections::BTreeMap<u32, u8>,
}

impl ArgTransferRuleState {
    pub fn new(open_date: GameDate) -> Self {
        Self {
            open_date,
            announced: false,
            signings: std::collections::BTreeMap::new(),
        }
    }

    /// `argrules_on_window_date` (`0x0040a770`): reset every club's signing
    /// counter as the window opens.
    pub fn open_window(&mut self) {
        self.signings.clear();
    }

    /// `argrules_check_signing_allowed` (`0x0040a8c0`): a club may sign only
    /// while it has made fewer than [`MAX_SIGNINGS_PER_WINDOW`] signings.
    pub fn can_sign(&self, club_id: u32) -> bool {
        self.signings.get(&club_id).copied().unwrap_or(0) < MAX_SIGNINGS_PER_WINDOW
    }

    /// `argrules_increment_signing_counter` (`0x0040a870`): record a signing if
    /// the rule allows it. Returns whether the signing was permitted.
    pub fn record_signing(&mut self, club_id: u32) -> bool {
        if !self.can_sign(club_id) {
            return false;
        }
        *self.signings.entry(club_id).or_insert(0) += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_signings_at_two_per_window() {
        let mut r = ArgTransferRuleState::new(GameDate { year: 2002, month: 1, day: 1 });
        assert!(r.can_sign(10));
        assert!(r.record_signing(10)); // 1
        assert!(r.record_signing(10)); // 2
        assert!(!r.can_sign(10), "third signing blocked (status 0xe)");
        assert!(!r.record_signing(10));
        // Other clubs are independent.
        assert!(r.can_sign(11));
        // Opening a new window resets the counters.
        r.open_window();
        assert!(r.can_sign(10));
    }
}
