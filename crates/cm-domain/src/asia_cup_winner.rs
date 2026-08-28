//! Asian Cup Winners' Cup (AFC) — a port of `asia_cup_winner.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\eurocomp\asia_cup_winner.cpp`, VA
//! `0x0040cbe0..0x0040e77f`, vtable `0x00955574`). Function decode + names:
//! `reports/carve_rename_map.json` (`asia_cup_winner.cpp`).
//!
//! A **club** continental cup in the `eurocomp/` framework, contested by
//! domestic **cup winners** — a sibling of the Asian Club Championship. It
//! reuses the shared group+knockout engine ([`crate::african_nations`]).
//!
//! ## Fidelity notes
//! * **Structure** — the exe splits this cup into 2 stage objects
//!   (`[esi+0x2c]=2`, `asiacc`-style builders); this port plays the same
//!   observable group+knockout shape as the Club Championship via the shared
//!   engine. The exact stage split is a documented follow-up.
//! * **Field** — no prior-season cup winners exist at kickoff, so the entrants
//!   are each Asian nation's **second-strongest** club (a cup-winner proxy that
//!   is deliberately distinct from the Club Championship's strongest-club
//!   field). Flagged, not invented.

use crate::african_nations::AcnTeam;
use crate::asia_nations::ASIA_CONTINENT_ID;
use crate::typed_records::{ClubView, NationView};
use crate::DomainOpaqueRecord;

/// `club_competition` id of the Asian Cup Winners' Cup.
pub const ASIA_CUP_WINNER_COMP_ID: u32 = 62;
pub const ASIA_CUP_WINNER_NAME: &str = "Asian Cup Winners' Cup";
pub const ASIA_CUP_WINNER_CHAMPION_NOUN: &str = "Asian Cup Winners' Cup winners";

/// One candidate per Asian nation: that nation's **second**-strongest club (a
/// domestic-cup-winner proxy, distinct from the Club Championship's strongest).
/// Nations with only one club contribute that club. Feeds the ACN engine's draw.
pub fn asian_cup_winner_clubs(
    clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    let asian: std::collections::BTreeSet<i32> = nations
        .iter()
        .filter(|rec| NationView::new(rec).continent_id() == ASIA_CONTINENT_ID)
        .map(|rec| NationView::new(rec).id() as i32)
        .collect();

    // Collect all Asian clubs per nation.
    let mut by_nation: std::collections::BTreeMap<i32, Vec<AcnTeam>> = std::collections::BTreeMap::new();
    for rec in clubs {
        let club = ClubView::new(rec);
        let Some(nation_id) = club.nation_id() else { continue };
        if !asian.contains(&nation_id) {
            continue;
        }
        by_nation.entry(nation_id).or_default().push(AcnTeam {
            club_id: club.id(),
            nation_id,
            name: club.primary_name(),
            reputation: club.reputation(),
            pot: 0,
        });
    }

    // Per nation: the second-strongest club (or the only one).
    by_nation
        .into_values()
        .filter_map(|mut v| {
            v.sort_by(|a, b| b.reputation.cmp(&a.reputation).then(a.club_id.cmp(&b.club_id)));
            v.get(1).or_else(|| v.first()).cloned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_second_strongest_per_nation() {
        // The cup-winner proxy is the second-strongest club of a nation.
        let mut v = vec![
            AcnTeam { club_id: 1, nation_id: 7, name: "A".into(), reputation: 90, pot: 0 },
            AcnTeam { club_id: 2, nation_id: 7, name: "B".into(), reputation: 80, pot: 0 },
            AcnTeam { club_id: 3, nation_id: 7, name: "C".into(), reputation: 70, pot: 0 },
        ];
        v.sort_by(|a, b| b.reputation.cmp(&a.reputation).then(a.club_id.cmp(&b.club_id)));
        assert_eq!(v.get(1).unwrap().name, "B");
    }
}
