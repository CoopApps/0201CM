//! European (UEFA) Super Cup — a port of `eur_super_cup.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\eurocomp\eur_super_cup.cpp`, vtable
//! `0x00957f78`, ctor `0x00563da0`, `club_competition` id 329). Function decode
//! ledger: `reports/carve_rename_map.json` (`eur_super_cup.cpp`).
//!
//! A single tie between the **European Champions Cup** winner (comp 326) and
//! the **UEFA Cup** winner (comp 328) — the same shape as the Asian Super Cup
//! (see [`crate::asia_super_cup`]) and every ported national super cup.
//! This module is a thin wrapper around the shared [`crate::super_cup`] engine;
//! the participant selection is picked up from [`crate::intl_comps`].

use crate::intl_comps;
use crate::super_cup::SuperCupState;

/// `club_competition` id of the European Super Cup.
pub const EURO_SUPER_CUP_COMP_ID: u32 = intl_comps::UEFA_SUPER_CUP_COMP_ID;
pub const EURO_SUPER_CUP_NAME: &str = intl_comps::UEFA_SUPER_CUP_NAME;

/// The preset [`SuperCupState`] for a given year. Fed by the two European club
/// cups' honours; use [`crate::super_cup::advance`] to create the tie and
/// announce the winner.
pub fn state(year: u16) -> SuperCupState {
    intl_comps::uefa_super_cup_state(year)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::honours::Honour;
    use crate::{super_cup, GameDate};

    #[test]
    fn ties_uefa_champions_league_and_uefa_cup_winners() {
        let state = state(2001);
        let honours = vec![
            Honour::champion(2001, 0x83c, "European Champions Cup", 10, "Bayern Munich"),
            Honour::champion(2001, 0x83c, "UEFA Cup", 20, "Liverpool"),
        ];
        let date = GameDate { year: 2001, month: 8, day: 24 };
        let adv = super_cup::advance(&state, &[], &honours, &date, 0);
        assert!(adv.created);
        let f = &adv.new_fixtures[0];
        assert_eq!((f.home_club_id, f.away_club_id), (10, 20));
    }

    #[test]
    fn dormant_when_uefa_cup_champion_missing() {
        let state = state(2001);
        let honours = vec![
            Honour::champion(2001, 0x83c, "European Champions Cup", 10, "Bayern"),
        ];
        let date = GameDate { year: 2001, month: 8, day: 24 };
        let adv = super_cup::advance(&state, &[], &honours, &date, 0);
        assert!(!adv.created);
    }
}
