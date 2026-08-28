//! CONCACAF Champions Cup — a port of `concacaf_champ.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\eurocomp\concacaf_champ.cpp`, comp 343).
//! Function decode ledger: `reports/carve_rename_map.json` (`concacaf_champ.cpp`).
//!
//! A **club** continental cup contested by North American nations' champions,
//! same shape as the Asian Club Championship (see [`crate::asia_club_champ`]).
//! Feeds the Inter-American Cup (`inter_amer_cup.cpp` `0x0061b760`) which ties
//! the CONCACAF Champions Cup winner against the Copa Libertadores winner.
//!
//! This module is a thin wrapper: participant selection comes from
//! [`crate::intl_comps::concacaf_champions_cup_pool`] and the tournament runs
//! through the shared [`crate::african_nations`] group + KO engine.

use crate::african_nations::AcnTeam;
use crate::intl_comps;
use crate::DomainOpaqueRecord;

/// `club_competition` id of the CONCACAF Champions Cup.
pub const CONCACAF_CHAMPIONS_CUP_COMP_ID: u32 = intl_comps::CONCACAF_CHAMPIONS_CUP_COMP_ID;
pub const CONCACAF_CHAMPIONS_CUP_NAME: &str = intl_comps::CONCACAF_CHAMPIONS_CUP_NAME;
pub const CONCACAF_CHAMPIONS_CUP_CHAMPION_NOUN: &str =
    intl_comps::CONCACAF_CHAMPIONS_CUP_CHAMPION_NOUN;

/// One "champion" proxy club per North American nation. Feeds the ACN engine's
/// draw ([`crate::african_nations::AcnDraw::draw`]).
pub fn pool(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    intl_comps::concacaf_champions_cup_pool(clubs, nations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::african_nations::{AcnDraw, ACN_TEAM_COUNT};

    #[test]
    fn draw_takes_sixteen_strongest_of_the_pool() {
        let pool: Vec<AcnTeam> = (0..20)
            .map(|i| AcnTeam {
                club_id: (300 + i) as u32,
                nation_id: i as i32,
                name: format!("CONCACAF-{i}"),
                reputation: i as u16,
                pot: 0,
            })
            .collect();
        let mut rng = cm_rng::MatchRng::new((0..64).collect(), cm_rng::CrtRand::new(7));
        let drawn = AcnDraw::draw(pool, &mut rng).expect("20 >= 16");
        assert_eq!(drawn.len(), ACN_TEAM_COUNT);
        assert!(drawn.iter().all(|t| t.reputation >= 4));
    }

    #[test]
    fn pool_on_empty_world_is_empty() {
        let empty: Vec<DomainOpaqueRecord> = Vec::new();
        assert!(pool(&empty, &empty).is_empty());
    }
}
