//! Asian Club Championship (AFC) — a port of `asia_club_champ.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\eurocomp\asia_club_champ.cpp`, VA
//! `0x0040ab40..0x0040cbdf`, vtable `0x009554d4`). Function decode + names:
//! `reports/carve_rename_map.json` (`asia_club_champ.cpp`).
//!
//! This is a **club** continental cup (base ctor `0x00502320`, the club-comp
//! base — not the nation base) with the **same 4-stage group + knockout shape**
//! as the African Cup of Nations. It therefore reuses the ACN engine
//! ([`crate::african_nations`]): the seeding, group-stage generation and
//! save-side knockout advancement are shared — this module only supplies the
//! Asian club field and the competition identity.
//!
//! ## Fidelity notes
//! * **Field** — the Asian Club Championship is contested by national champions.
//!   The headless model has no prior-season champions at kickoff, so this
//!   gathers each Asian nation's **strongest club** (by reputation) and draws 16
//!   from those — a documented proxy for the champions, not an invented rule.
//! * **Structure** — 16 clubs, 4 groups of 4, QF/SF/Final, all shared with the
//!   ACN engine (which is the exact structure `asiacc_build_stages`
//!   `0x0040b680` / `asiacc_advance_stage` `0x0040c070` build).

use crate::african_nations::AcnTeam;
use crate::typed_records::{ClubView, NationView};
use crate::DomainOpaqueRecord;

/// Asia's continent id (`continent.dat` id 1). VERIFIED (nation `+0x71`).
pub const ASIA_CONTINENT_ID: i32 = 1;
/// `club_competition` id of the Asian Club Championship.
pub const ASIA_CLUB_CHAMP_COMP_ID: u32 = 110;
pub const ASIA_CLUB_CHAMP_NAME: &str = "Asian Club Championship";
pub const ASIA_CHAMPION_NOUN: &str = "Asian club champions";

/// One candidate club per Asian nation — that nation's strongest club (highest
/// reputation), a proxy for its domestic champion. These feed the ACN engine's
/// draw ([`crate::african_nations::AcnDraw::draw`]), which takes the 16
/// strongest.
pub fn asian_national_champion_clubs(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    let asian: std::collections::BTreeSet<i32> = nations
        .iter()
        .filter(|rec| NationView::new(rec).continent_id() == ASIA_CONTINENT_ID)
        .map(|rec| NationView::new(rec).id() as i32)
        .collect();

    // Best-reputation club per Asian nation.
    let mut best: std::collections::BTreeMap<i32, AcnTeam> = std::collections::BTreeMap::new();
    for rec in clubs {
        let club = ClubView::new(rec);
        let Some(nation_id) = club.nation_id() else { continue };
        if !asian.contains(&nation_id) {
            continue;
        }
        let team = AcnTeam {
            club_id: club.id(),
            nation_id,
            name: club.primary_name(),
            reputation: club.reputation(),
            pot: 0,
        };
        best.entry(nation_id)
            .and_modify(|cur| {
                if team.reputation > cur.reputation {
                    *cur = team.clone();
                }
            })
            .or_insert(team);
    }
    best.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::african_nations::{AcnDraw, ACN_TEAM_COUNT};

    #[test]
    fn draw_takes_sixteen_strongest_of_the_pool() {
        // A synthetic pool of 20 "champions" of ascending strength.
        let pool: Vec<AcnTeam> = (0..20)
            .map(|i| AcnTeam {
                club_id: i,
                nation_id: i as i32,
                name: format!("Champ{i}"),
                reputation: i as u16,
                pot: 0,
            })
            .collect();
        let mut rng = cm_rng::MatchRng::new((0..64).collect(), cm_rng::CrtRand::new(1));
        let drawn = AcnDraw::draw(pool, &mut rng).expect("20 >= 16");
        assert_eq!(drawn.len(), ACN_TEAM_COUNT);
        // Strongest survive: reputations 4..=19.
        assert!(drawn.iter().all(|t| t.reputation >= 4));
    }
}
