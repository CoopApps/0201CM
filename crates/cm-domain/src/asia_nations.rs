//! Asian Cup of Nations (AFC) — a port of `asia_nations.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\intercomp\asia_nations.cpp`, VA
//! `0x0040e780..0x0041056f`, vtable `0x00955614`). Function decode + names:
//! `reports/carve_rename_map.json` (`asia_nations.cpp`).
//!
//! A **national-team** continental cup in the same `intercomp/` framework as the
//! African Cup of Nations — nation base ctor `0x006674d0`, 16 teams, 4 groups +
//! knockout — so it reuses the ACN engine ([`crate::african_nations`]). The one
//! structural difference is decoded here: the ctor snaps the start year with
//! `(year - 0x7cc) & 0x80000003` (mod **4**, anchor 1996) — the Asian Cup is
//! **quadrennial**, not biennial like the ACN.

use crate::african_nations::{continental_national_teams, next_edition_year_period, AcnTeam};
use crate::DomainOpaqueRecord;

/// Asia's continent id (`continent.dat` id 1). VERIFIED (nation `+0x71`).
pub const ASIA_CONTINENT_ID: i32 = 1;
/// `nation_comp` id of the "Asian Cup".
pub const ASIA_CUP_COMP_ID: u32 = 399;
pub const ASIA_CUP_NAME: &str = "Asian Cup of Nations";
pub const ASIA_CUP_CHAMPION_NOUN: &str = "Asian champions";
/// The exe's `0x7cc` biennial/quadrennial anchor year.
pub const ASIA_CUP_ANCHOR_YEAR: u16 = 1996;
/// The Asian Cup is played every four years.
pub const ASIA_CUP_PERIOD: u16 = 4;

/// The next Asian Cup edition year on or after `year+1`. Port of `asian_ctor`
/// (`0x0040e780`): the quadrennial (mod-4, 1996-anchored) year-snap.
pub fn next_asian_cup_year(year: u16) -> u16 {
    next_edition_year_period(year, ASIA_CUP_ANCHOR_YEAR, ASIA_CUP_PERIOD)
}

/// The candidate pool: every Asian national team (continent 1).
pub fn asian_national_teams(
    nat_clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    continental_national_teams(nat_clubs, nations, ASIA_CONTINENT_ID)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asian_cup_is_quadrennial() {
        // Editions in 1996, 2000, 2004, ... (mod 4 from 1996).
        assert_eq!(next_asian_cup_year(1999), 2000);
        assert_eq!(next_asian_cup_year(2000), 2004);
        assert_eq!(next_asian_cup_year(2001), 2004);
        assert_eq!(next_asian_cup_year(1995), 1996);
    }
}
