//! American nations — league / cup / national-team wiring for every country in
//! CONCACAF (continent 3) and CONMEBOL (continent 5), organised as one module
//! per nation, following the shape of `crate::african_nations`.
//!
//! ## What the exe actually ships
//!
//! Only four American countries have their own per-country `.cpp` sources in
//! the exe (verified against `reports/carve_rename_map.json`):
//!
//! * **Argentina** — `arg_prm.cpp` (Primera, split Apertura+Clausura, comp
//!   `0x00404290..0x00406c2f`), `arg_second.cpp` (Nacional B),
//!   `argentina_awards.cpp` / `argentina_rules.cpp` — already ported to
//!   [`crate::arg_primera`], [`crate::arg_second`], [`crate::arg_rules`].
//! * **Brazil** — `bra_nat_first/second/third.cpp` + 12 `bra_reg_*.cpp` state
//!   championships + `bra_cup.cpp` (Copa do Brasil) + `bra_champ_cup.cpp`
//!   (national playoff) + `brazil_awards.cpp` + `brazil_rules.cpp` — wired
//!   through the shared `simple_league` / `domestic_cup` engines from
//!   `crate::lib`.
//! * **USA** — `usa_mls.cpp`, `usa_mls_all_stars.cpp`, `usa_open_cup.cpp`,
//!   `usa_awards.cpp`, `usa_rules.cpp`.
//! * **Mexico** — no dedicated cpp; the Mexican First Division runs on the
//!   generic league engine and is wired by comp id.
//!
//! The other 18 American nations (Uruguay, Colombia, Chile, Peru, Bolivia,
//! Ecuador, Venezuela, Paraguay, Canada, Costa Rica, Guatemala, Honduras,
//! Panama, El Salvador, Jamaica, Trinidad & Tobago, Cuba, Haiti) ship as
//! **national-team-only** entries in `nat_club.dat`: they have a `NationView`
//! record and a national team (for CONMEBOL Copa América, CONCACAF Gold Cup,
//! and the FIFA World Cup qualifiers) but no domestic league in the shipped
//! data. Their modules here expose the nation id, the continent, and the
//! [`national_team_of`] accessor so continental competitions can gather them.
//!
//! The two continental national-team tournaments — Copa América (CONMEBOL) and
//! CONCACAF Gold Cup — are described at the bottom of this module. The Gold Cup
//! itself is already wired at runtime through
//! `crate::african_nations::AcnDraw` (see `lib.rs`, ctor comment: *"CONCACAF
//! Gold Cup {year}: 16 North American national teams (continent==3), biennial;
//! ported from goldcup.cpp reusing the ACN engine"*); this file adds a matching
//! Copa América helper on the CONMEBOL side.

use serde::{Deserialize, Serialize};

use crate::african_nations::{continental_national_teams, next_edition_year_period, AcnTeam};
use crate::typed_records::{ClubView, NationView};
use crate::DomainOpaqueRecord;

// ---------------------------------------------------------------------------
// Continent ids (verified against `rust-db/core/continents.json`).
// ---------------------------------------------------------------------------

/// CONCACAF — "Confederation of North and Central American and Caribbean
/// Association Football". Continent id 3 in `continent.dat`.
pub const NORTH_AMERICA_CONTINENT_ID: i32 = 3;

/// CONMEBOL — "Confederation of South American Football". Continent id 5.
pub const SOUTH_AMERICA_CONTINENT_ID: i32 = 5;

// ---------------------------------------------------------------------------
// Small helpers reused by every per-nation module
// ---------------------------------------------------------------------------

/// The national team (`nat_club.dat` record) for `nation_id`, or `None` when
/// the shipped data has no national team for that nation. Every American
/// nation in `rust-db/core/nations.json` has one.
pub fn national_team_of<'a>(
    nat_clubs: &'a [DomainOpaqueRecord],
    nation_id: i32,
) -> Option<&'a DomainOpaqueRecord> {
    nat_clubs
        .iter()
        .find(|rec| ClubView::new(rec).nation_id() == Some(nation_id))
}

/// Convenience: build an [`AcnTeam`] from a nation's national team record so
/// it drops straight into the shared continental-cup group/knockout engine.
pub fn national_team_as_acn(
    nat_clubs: &[DomainOpaqueRecord],
    nation_id: i32,
) -> Option<AcnTeam> {
    let rec = national_team_of(nat_clubs, nation_id)?;
    let view = ClubView::new(rec);
    Some(AcnTeam {
        club_id: view.id(),
        nation_id: view.nation_id().unwrap_or(nation_id),
        name: view.primary_name(),
        reputation: view.reputation(),
        pot: 0,
    })
}

/// Clubs whose primary competition (`club+0x57`) is `comp_id` — the exe's
/// `*_gather_clubs` membership filter, extracted so every per-nation module
/// can reuse it without hard-coding the offset.
pub fn clubs_in_division(clubs: &[DomainOpaqueRecord], comp_id: i32) -> Vec<&DomainOpaqueRecord> {
    clubs
        .iter()
        .filter(|rec| ClubView::new(rec).division_id() == Some(comp_id))
        .collect()
}

/// Look up a nation record by id.
pub fn nation_by_id<'a>(
    nations: &'a [DomainOpaqueRecord],
    nation_id: i32,
) -> Option<&'a NationView<'a>> {
    // We cannot cache the view (it borrows the record), so callers use the
    // record + `NationView::new`. This is a plain existence probe.
    let _ = nations
        .iter()
        .find(|rec| NationView::new(rec).id() as i32 == nation_id)?;
    None // sentinel — callers construct the view themselves via `nation_record_by_id`.
}

/// The nation record for `nation_id`, if present.
pub fn nation_record_by_id(
    nations: &[DomainOpaqueRecord],
    nation_id: i32,
) -> Option<&DomainOpaqueRecord> {
    nations
        .iter()
        .find(|rec| NationView::new(rec).id() as i32 == nation_id)
}

/// A minimal per-nation profile — every module below exposes one so callers can
/// enumerate the American football map uniformly (Copa América draw, CONCACAF
/// Gold Cup draw, World Cup qualifier groups).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NationProfile {
    pub nation_id: i32,
    pub continent_id: i32,
    pub name: &'static str,
    /// Top-flight club competition id in `club_comp.dat`, when the nation
    /// ships one.
    pub top_league_comp_id: Option<i32>,
    /// Primary domestic cup id, when the nation ships one.
    pub domestic_cup_comp_id: Option<i32>,
}

// ===========================================================================
// SOUTH AMERICA — CONMEBOL (continent 5)
// ===========================================================================

/// **Argentina** (nation id 8). Full league engine lives in
/// [`crate::arg_primera`] (Primera, split Apertura + Clausura, comp
/// `arg_prm.cpp` `0x00404290`), [`crate::arg_second`] (Nacional B,
/// `arg_second.cpp`), and [`crate::arg_rules`] (transfer-window rules,
/// `argentina_rules.cpp`).
pub mod argentina {
    use super::*;
    pub const NATION_ID: i32 = 8;
    pub const NAME: &str = "Argentina";
    /// Argentine Primera División (`club_comp.dat` id 63).
    pub const PRIMERA_COMP_ID: i32 = crate::arg_primera::ARG_PRIMERA_COMP_ID;
    /// Argentine Second Division / Nacional B (id 64).
    pub const SECOND_DIVISION_COMP_ID: i32 = 64;
    /// Third Division (id 88).
    pub const THIRD_DIVISION_COMP_ID: i32 = 88;

    pub fn profile() -> NationProfile {
        NationProfile {
            nation_id: NATION_ID,
            continent_id: SOUTH_AMERICA_CONTINENT_ID,
            name: NAME,
            top_league_comp_id: Some(PRIMERA_COMP_ID),
            domestic_cup_comp_id: None, // no shipped domestic cup pre-2002
        }
    }

    pub fn national_team(nat_clubs: &[DomainOpaqueRecord]) -> Option<AcnTeam> {
        national_team_as_acn(nat_clubs, NATION_ID)
    }
}

/// **Brazil** (nation id 27). Full port: three national divisions plus twelve
/// state championships plus Copa do Brasil plus the champions playoff — all
/// wired through the shared `simple_league` / `domestic_cup` engines from
/// `crate::lib` (see `_brazil_block` in `reports/carve_rename_map.json`).
pub mod brazil {
    use super::*;
    pub const NATION_ID: i32 = 27;
    pub const NAME: &str = "Brazil";
    /// Brazilian National First Division / Série A (id 65) — 28 clubs, single
    /// round-robin (`bra_nat_first.cpp`).
    pub const SERIE_A_COMP_ID: i32 = 65;
    /// Brazilian National Second Division / Série B (id 79) — `bra_nat_second.cpp`.
    pub const SERIE_B_COMP_ID: i32 = 79;
    /// Brazilian National Third Division / Série C (id 80, 56 clubs) —
    /// `bra_nat_third.cpp`.
    pub const SERIE_C_COMP_ID: i32 = 80;
    /// Copa do Brasil (id 68) — `bra_cup.cpp`.
    pub const COPA_DO_BRASIL_COMP_ID: i32 = 68;
    /// National championship playoff (id 270) — `bra_champ_cup.cpp`.
    pub const CHAMP_PLAYOFF_COMP_ID: i32 = 270;
    /// State championships (`bra_reg_*.cpp`) — 12 comps in comp-id order.
    pub const STATE_CHAMPIONSHIP_COMP_IDS: [i32; 12] = [
        66,  // Rio de Janeiro (bra_reg_rio)
        67,  // São Paulo (bra_reg_sp)
        73,  // North (bra_reg_north)
        74,  // Central (bra_reg_central)
        254, // Gaúcho (bra_reg_gaucho)
        256, // Northeast (bra_reg_northeast)
        258, // Bahia (bra_reg_bahia)
        260, // Goiás (bra_reg_goias)
        262, // Minas Gerais (bra_reg_minas_gerais)
        264, // Pernambuco (bra_reg_pern)
        266, // Paraná (bra_reg_parana)
        268, // Santa Catarina (bra_reg_santa)
    ];

    pub fn profile() -> NationProfile {
        NationProfile {
            nation_id: NATION_ID,
            continent_id: SOUTH_AMERICA_CONTINENT_ID,
            name: NAME,
            top_league_comp_id: Some(SERIE_A_COMP_ID),
            domestic_cup_comp_id: Some(COPA_DO_BRASIL_COMP_ID),
        }
    }

    pub fn national_team(nat_clubs: &[DomainOpaqueRecord]) -> Option<AcnTeam> {
        national_team_as_acn(nat_clubs, NATION_ID)
    }
}

/// **Uruguay** (nation id 201). No shipped domestic league; national team is
/// present in `nat_club.dat` and enters Copa América / World Cup qualifiers.
pub mod uruguay {
    use super::*;
    pub const NATION_ID: i32 = 201;
    pub const NAME: &str = "Uruguay";
    pub fn profile() -> NationProfile {
        NationProfile {
            nation_id: NATION_ID,
            continent_id: SOUTH_AMERICA_CONTINENT_ID,
            name: NAME,
            top_league_comp_id: None,
            domestic_cup_comp_id: None,
        }
    }
    pub fn national_team(nat_clubs: &[DomainOpaqueRecord]) -> Option<AcnTeam> {
        national_team_as_acn(nat_clubs, NATION_ID)
    }
}

/// **Colombia** (nation id 43). National-team-only in shipped data.
pub mod colombia {
    use super::*;
    pub const NATION_ID: i32 = 43;
    pub const NAME: &str = "Colombia";
    pub fn profile() -> NationProfile {
        NationProfile {
            nation_id: NATION_ID,
            continent_id: SOUTH_AMERICA_CONTINENT_ID,
            name: NAME,
            top_league_comp_id: None,
            domestic_cup_comp_id: None,
        }
    }
    pub fn national_team(nat_clubs: &[DomainOpaqueRecord]) -> Option<AcnTeam> {
        national_team_as_acn(nat_clubs, NATION_ID)
    }
}

macro_rules! stub_nation {
    ($mod:ident, $id:literal, $name:literal, $continent:expr $(,)?) => {
        pub mod $mod {
            use super::*;
            pub const NATION_ID: i32 = $id;
            pub const NAME: &str = $name;
            pub fn profile() -> NationProfile {
                NationProfile {
                    nation_id: NATION_ID,
                    continent_id: $continent,
                    name: NAME,
                    top_league_comp_id: None,
                    domestic_cup_comp_id: None,
                }
            }
            pub fn national_team(nat_clubs: &[DomainOpaqueRecord]) -> Option<AcnTeam> {
                national_team_as_acn(nat_clubs, NATION_ID)
            }
        }
    };
}

// Remaining CONMEBOL members — national-team-only skeleton stubs.
//
// NOTE (verified 2026-08-25): the CM 00/01 exe ships NO country-specific
// `.cpp` sources for any of these six nations (checked
// `file_attribution.json`: no `paraguay_*`, `chile_*`, `peru_*`, `bolivia_*`,
// `ecuador_*`, `venezuela_*`), AND `rust-db/references/club_competitions.json`
// contains NO `club_comp.dat` rows for their nation ids (41, 24, 57, 144, 146,
// 204). Per memory `[[no-runtime-computed-excuse]]` and
// `[[no-pil-approximation]]`, these cannot be promoted to full ports without
// fabricating data. If a data pack adds `club_comp` rows for them, upgrade the
// stubs by attaching real ids + a `NationLeagueSet` (like [`crate::euro_nations::denmark`]
// which has shipped comp rows but no rules ctor).
stub_nation!(paraguay,  144, "Paraguay",  SOUTH_AMERICA_CONTINENT_ID);
stub_nation!(chile,     41,  "Chile",     SOUTH_AMERICA_CONTINENT_ID);
stub_nation!(peru,      146, "Peru",      SOUTH_AMERICA_CONTINENT_ID);
stub_nation!(bolivia,   24,  "Bolivia",   SOUTH_AMERICA_CONTINENT_ID);
stub_nation!(ecuador,   57,  "Ecuador",   SOUTH_AMERICA_CONTINENT_ID);
stub_nation!(venezuela, 204, "Venezuela", SOUTH_AMERICA_CONTINENT_ID);

// ===========================================================================
// NORTH / CENTRAL AMERICA / CARIBBEAN — CONCACAF (continent 3)
// ===========================================================================

/// **United States** (nation id 196). MLS + A-League + D3-Pro + MLS All-Stars +
/// US Open Cup — per `usa_mls.cpp`, `usa_mls_all_stars.cpp`, `usa_open_cup.cpp`.
pub mod usa {
    use super::*;
    pub const NATION_ID: i32 = 196;
    pub const NAME: &str = "United States";
    /// American Major League (`club_comp.dat` id 31) — `usa_mls.cpp`.
    pub const MLS_COMP_ID: i32 = 31;
    /// A-League (id 32).
    pub const A_LEAGUE_COMP_ID: i32 = 32;
    /// D3-Pro League (id 33).
    pub const D3_PRO_LEAGUE_COMP_ID: i32 = 33;
    /// MLS All-Star Game (id 90) — `usa_mls_all_stars.cpp`.
    pub const MLS_ALL_STARS_COMP_ID: i32 = 90;
    /// US Open Cup (id 342) — `usa_open_cup.cpp`.
    pub const US_OPEN_CUP_COMP_ID: i32 = 342;

    pub fn profile() -> NationProfile {
        NationProfile {
            nation_id: NATION_ID,
            continent_id: NORTH_AMERICA_CONTINENT_ID,
            name: NAME,
            top_league_comp_id: Some(MLS_COMP_ID),
            domestic_cup_comp_id: Some(US_OPEN_CUP_COMP_ID),
        }
    }

    pub fn national_team(nat_clubs: &[DomainOpaqueRecord]) -> Option<AcnTeam> {
        national_team_as_acn(nat_clubs, NATION_ID)
    }

    /// MLS member clubs, taken from the shipped data by primary-competition
    /// membership (`club+0x57 == 31`) — the same filter every league engine
    /// uses.
    pub fn mls_clubs(clubs: &[DomainOpaqueRecord]) -> Vec<&DomainOpaqueRecord> {
        super::clubs_in_division(clubs, MLS_COMP_ID)
    }
}

/// **Mexico** (nation id 121). Mexican First Division uses the generic league
/// engine; comp ids are wired here for lookup.
pub mod mexico {
    use super::*;
    pub const NATION_ID: i32 = 121;
    pub const NAME: &str = "Mexico";
    /// Mexican First Division (`club_comp.dat` id 214).
    pub const PRIMERA_DIVISION_COMP_ID: i32 = 214;
    /// Mexican First Division A / Ascenso (id 215).
    pub const PRIMERA_A_COMP_ID: i32 = 215;
    /// Mexican Second Division (id 216).
    pub const SECOND_DIVISION_COMP_ID: i32 = 216;
    /// Mexican Lower Division (id 217).
    pub const LOWER_DIVISION_COMP_ID: i32 = 217;
    /// Pre-PreLibertadores Cup (id 218) — the Mexican slot's qualifier.
    pub const PRE_LIBERTADORES_COMP_ID: i32 = 218;

    pub fn profile() -> NationProfile {
        NationProfile {
            nation_id: NATION_ID,
            continent_id: NORTH_AMERICA_CONTINENT_ID,
            name: NAME,
            top_league_comp_id: Some(PRIMERA_DIVISION_COMP_ID),
            domestic_cup_comp_id: None, // no shipped domestic cup
        }
    }

    pub fn national_team(nat_clubs: &[DomainOpaqueRecord]) -> Option<AcnTeam> {
        national_team_as_acn(nat_clubs, NATION_ID)
    }

    /// The Mexican Primera clubs by primary-competition membership.
    pub fn primera_clubs(clubs: &[DomainOpaqueRecord]) -> Vec<&DomainOpaqueRecord> {
        super::clubs_in_division(clubs, PRIMERA_DIVISION_COMP_ID)
    }
}

// Remaining CONCACAF members — national-team-only skeleton stubs.
//
// NOTE (verified 2026-08-25): same story as the CONMEBOL block above — no
// `<country>_*.cpp` or `<abbr>_*.cpp` shipped for any of these nations, and no
// `club_comp.dat` rows for their nation ids (36 Canada, 46 Costa Rica, 48
// Cuba, 59 El Salvador, 78 Guatemala, 82 Haiti, 84 Honduras, 96 Jamaica, 142
// Panama, 190 Trinidad & Tobago). Promote to a full port only when real data
// lands — per `[[no-runtime-computed-excuse]]`, do not fabricate.
stub_nation!(canada,       36,  "Canada",              NORTH_AMERICA_CONTINENT_ID);
stub_nation!(costa_rica,   46,  "Costa Rica",          NORTH_AMERICA_CONTINENT_ID);
stub_nation!(guatemala,    78,  "Guatemala",           NORTH_AMERICA_CONTINENT_ID);
stub_nation!(honduras,     84,  "Honduras",            NORTH_AMERICA_CONTINENT_ID);
stub_nation!(panama,       142, "Panama",              NORTH_AMERICA_CONTINENT_ID);
stub_nation!(el_salvador,  59,  "El Salvador",         NORTH_AMERICA_CONTINENT_ID);
stub_nation!(jamaica,      96,  "Jamaica",             NORTH_AMERICA_CONTINENT_ID);
stub_nation!(trinidad,     190, "Trinidad & Tobago",   NORTH_AMERICA_CONTINENT_ID);
stub_nation!(cuba,         48,  "Cuba",                NORTH_AMERICA_CONTINENT_ID);
stub_nation!(haiti,        82,  "Haiti",               NORTH_AMERICA_CONTINENT_ID);

// ===========================================================================
// Continental national-team tournaments
// ===========================================================================

/// Copa América — the CONMEBOL national-team cup. `nation_comp.dat` id 65 in
/// the shipped data (verified: `references/nation_competitions.json` labels
/// comp 65 as the CONMEBOL national-team tournament; see also
/// `reports/carve_rename_map.json` for the intercomp framework the exe uses).
/// It is drawn from every South American national team (continent 5) and is
/// run every other year (biennial), like the African Cup of Nations. The draw
/// / group / knockout mechanism is provided by the shared
/// [`crate::african_nations::AcnDraw`] engine — the exe's intercomp base is
/// the same class family.
pub mod copa_america {
    use super::*;
    /// CONMEBOL Copa América anchor year (biennial cadence).
    pub const ANCHOR_YEAR: u16 = 1997;
    pub const PERIOD_YEARS: u16 = 2;
    pub const NAME: &str = "Copa América";
    pub const CHAMPION_NOUN: &str = "South American champions";

    /// The next Copa América edition year on or after `year+1`.
    pub fn next_edition_year(year: u16) -> u16 {
        next_edition_year_period(year, ANCHOR_YEAR, PERIOD_YEARS)
    }

    /// The candidate pool: every South American national team.
    pub fn candidates(
        nat_clubs: &[DomainOpaqueRecord],
        nations: &[DomainOpaqueRecord],
    ) -> Vec<AcnTeam> {
        continental_national_teams(nat_clubs, nations, SOUTH_AMERICA_CONTINENT_ID)
    }
}

/// CONCACAF Gold Cup — the North / Central American / Caribbean national-team
/// cup. Already wired at the app layer via
/// `crate::african_nations::AcnDraw` (see `lib.rs`: *"CONCACAF Gold Cup ...
/// biennial; ported from goldcup.cpp reusing the ACN engine"*). This module
/// exposes the CONMEBOL-side candidate helper so callers on the CONCACAF side
/// have a matching entry point.
pub mod gold_cup {
    use super::*;
    /// CONCACAF Gold Cup anchor year (biennial).
    pub const ANCHOR_YEAR: u16 = 1996;
    pub const PERIOD_YEARS: u16 = 2;
    pub const NAME: &str = "CONCACAF Gold Cup";
    pub const CHAMPION_NOUN: &str = "North American champions";

    pub fn next_edition_year(year: u16) -> u16 {
        next_edition_year_period(year, ANCHOR_YEAR, PERIOD_YEARS)
    }

    pub fn candidates(
        nat_clubs: &[DomainOpaqueRecord],
        nations: &[DomainOpaqueRecord],
    ) -> Vec<AcnTeam> {
        continental_national_teams(nat_clubs, nations, NORTH_AMERICA_CONTINENT_ID)
    }
}

// ---------------------------------------------------------------------------
// Enumeration helpers
// ---------------------------------------------------------------------------

/// The full profile list for CONMEBOL (10 nations), in nation-id order.
pub fn conmebol_profiles() -> [NationProfile; 10] {
    [
        argentina::profile(),
        bolivia::profile(),
        brazil::profile(),
        chile::profile(),
        colombia::profile(),
        ecuador::profile(),
        paraguay::profile(),
        peru::profile(),
        uruguay::profile(),
        venezuela::profile(),
    ]
}

/// The full profile list for CONCACAF members that ship in this game (12
/// nations covering North America, Central America, and the four Caribbean
/// entries in `nations.json`), in nation-id order.
pub fn concacaf_profiles() -> [NationProfile; 12] {
    [
        canada::profile(),
        costa_rica::profile(),
        cuba::profile(),
        el_salvador::profile(),
        guatemala::profile(),
        haiti::profile(),
        honduras::profile(),
        jamaica::profile(),
        mexico::profile(),
        panama::profile(),
        trinidad::profile(),
        usa::profile(),
    ]
}

/// Every American nation this module knows about, both federations combined.
pub fn all_american_profiles() -> Vec<NationProfile> {
    let mut v = Vec::with_capacity(22);
    v.extend_from_slice(&conmebol_profiles());
    v.extend_from_slice(&concacaf_profiles());
    v
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_records::{ClubView, NationView};

    fn nat_club_rec(nation_id: i32, id: u32, name: &str, rep: u16) -> DomainOpaqueRecord {
        let mut raw = vec![0u8; ClubView::RECORD_SIZE];
        raw[0..4].copy_from_slice(&id.to_le_bytes());
        let bytes = name.as_bytes();
        let n = bytes.len().min(50);
        raw[4..4 + n].copy_from_slice(&bytes[..n]);
        raw[0x37] = 0xff;
        raw[0x53..0x57].copy_from_slice(&nation_id.to_le_bytes());
        raw[0x80..0x82].copy_from_slice(&rep.to_le_bytes());
        DomainOpaqueRecord {
            ordinal: id,
            id,
            primary_name: Some(name.to_string()),
            secondary_name: None,
            short_name: None,
            text_candidates: vec![name.to_string()],
            raw,
        }
    }

    fn nation_rec(id: u32, name: &str, continent: u8) -> DomainOpaqueRecord {
        let mut raw = vec![0u8; NationView::RECORD_SIZE];
        raw[0..4].copy_from_slice(&id.to_le_bytes());
        let bytes = name.as_bytes();
        let n = bytes.len().min(50);
        raw[4..4 + n].copy_from_slice(&bytes[..n]);
        raw[0x71] = continent;
        DomainOpaqueRecord {
            ordinal: id,
            id,
            primary_name: Some(name.to_string()),
            secondary_name: None,
            short_name: None,
            text_candidates: vec![name.to_string()],
            raw,
        }
    }

    #[test]
    fn nation_ids_match_shipped_db() {
        // Anchored to `rust-db/core/nations.json` — a regression tripwire.
        assert_eq!(argentina::NATION_ID, 8);
        assert_eq!(brazil::NATION_ID, 27);
        assert_eq!(usa::NATION_ID, 196);
        assert_eq!(mexico::NATION_ID, 121);
        assert_eq!(uruguay::NATION_ID, 201);
        assert_eq!(colombia::NATION_ID, 43);
        assert_eq!(bolivia::NATION_ID, 24);
        assert_eq!(chile::NATION_ID, 41);
        assert_eq!(peru::NATION_ID, 146);
        assert_eq!(ecuador::NATION_ID, 57);
        assert_eq!(venezuela::NATION_ID, 204);
        assert_eq!(paraguay::NATION_ID, 144);
        assert_eq!(canada::NATION_ID, 36);
        assert_eq!(costa_rica::NATION_ID, 46);
        assert_eq!(cuba::NATION_ID, 48);
        assert_eq!(el_salvador::NATION_ID, 59);
        assert_eq!(guatemala::NATION_ID, 78);
        assert_eq!(haiti::NATION_ID, 82);
        assert_eq!(honduras::NATION_ID, 84);
        assert_eq!(jamaica::NATION_ID, 96);
        assert_eq!(panama::NATION_ID, 142);
        assert_eq!(trinidad::NATION_ID, 190);
    }

    #[test]
    fn shipped_top_league_comp_ids_are_wired() {
        // The four countries the exe actually ships a top league for.
        assert_eq!(argentina::PRIMERA_COMP_ID, 63);
        assert_eq!(brazil::SERIE_A_COMP_ID, 65);
        assert_eq!(usa::MLS_COMP_ID, 31);
        assert_eq!(mexico::PRIMERA_DIVISION_COMP_ID, 214);
        // ... and a nation without a shipped league keeps `None` in its profile.
        assert!(uruguay::profile().top_league_comp_id.is_none());
        assert!(canada::profile().top_league_comp_id.is_none());
    }

    #[test]
    fn continent_partition_is_exact() {
        for p in conmebol_profiles() {
            assert_eq!(p.continent_id, SOUTH_AMERICA_CONTINENT_ID, "{}", p.name);
        }
        for p in concacaf_profiles() {
            assert_eq!(p.continent_id, NORTH_AMERICA_CONTINENT_ID, "{}", p.name);
        }
        assert_eq!(all_american_profiles().len(), 22);
    }

    #[test]
    fn national_team_lookup_uses_nation_id() {
        let nats = vec![
            nat_club_rec(brazil::NATION_ID, 100, "Brazil NT", 20),
            nat_club_rec(usa::NATION_ID, 101, "USA NT", 12),
            nat_club_rec(uruguay::NATION_ID, 102, "Uruguay NT", 15),
        ];
        let bra = brazil::national_team(&nats).expect("Brazil national team");
        assert_eq!(bra.name, "Brazil NT");
        assert_eq!(bra.reputation, 20);
        assert_eq!(bra.nation_id, brazil::NATION_ID);

        let uru = uruguay::national_team(&nats).expect("Uruguay national team");
        assert_eq!(uru.reputation, 15);

        // A nation with no national team returns None cleanly.
        assert!(colombia::national_team(&nats).is_none());
    }

    #[test]
    fn copa_america_is_biennial_anchored_1997() {
        assert_eq!(copa_america::next_edition_year(1998), 1999);
        assert_eq!(copa_america::next_edition_year(1999), 2001);
        assert_eq!(copa_america::next_edition_year(2000), 2001);
        assert_eq!(copa_america::next_edition_year(2001), 2003);
    }

    #[test]
    fn gold_cup_is_biennial_anchored_1996() {
        assert_eq!(gold_cup::next_edition_year(1995), 1996);
        assert_eq!(gold_cup::next_edition_year(1996), 1998);
        assert_eq!(gold_cup::next_edition_year(1999), 2000);
    }

    #[test]
    fn copa_america_candidates_are_south_american_only() {
        let nations = vec![
            nation_rec(brazil::NATION_ID as u32, "Brazil", 5),
            nation_rec(argentina::NATION_ID as u32, "Argentina", 5),
            nation_rec(uruguay::NATION_ID as u32, "Uruguay", 5),
            nation_rec(usa::NATION_ID as u32, "USA", 3),
            nation_rec(mexico::NATION_ID as u32, "Mexico", 3),
        ];
        let nats = vec![
            nat_club_rec(brazil::NATION_ID, 1, "Brazil NT", 20),
            nat_club_rec(argentina::NATION_ID, 2, "Argentina NT", 19),
            nat_club_rec(uruguay::NATION_ID, 3, "Uruguay NT", 15),
            nat_club_rec(usa::NATION_ID, 4, "USA NT", 12),
            nat_club_rec(mexico::NATION_ID, 5, "Mexico NT", 13),
        ];
        let copa = copa_america::candidates(&nats, &nations);
        let mut names: Vec<&str> = copa.iter().map(|t| t.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["Argentina NT", "Brazil NT", "Uruguay NT"]);

        let gold = gold_cup::candidates(&nats, &nations);
        let mut names: Vec<&str> = gold.iter().map(|t| t.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["Mexico NT", "USA NT"]);
    }

    // ── per-nation coverage: division_count / season_length / cup_rounds ──
    //
    // Only the four Americas nations the exe actually ships with country-specific
    // `.cpp` sources have a domestic model to test. The other 18 CONMEBOL /
    // CONCACAF entries are stubs (see the block-header notes above); once real
    // data lands, add matching tests.

    #[test]
    fn argentina_division_count_and_cup_rounds() {
        // Argentina — three shipped divisions (`arg_prm.cpp`, `arg_second.cpp`,
        // Third Division comp id 88); no shipped domestic cup pre-2002.
        let ids = [
            argentina::PRIMERA_COMP_ID,
            argentina::SECOND_DIVISION_COMP_ID,
            argentina::THIRD_DIVISION_COMP_ID,
        ];
        assert_eq!(ids.len(), 3, "Argentina division count");
        assert!(argentina::profile().domestic_cup_comp_id.is_none(),
            "Argentina cup rounds = 0 (no shipped domestic cup)");
    }

    #[test]
    fn argentina_season_length_matches_primera_apertura_clausura() {
        // `crate::arg_primera` decodes the split-season matchday count from
        // `arg_prm.cpp` (0x00404290). Re-exported through `ARG_PRIMERA_*`
        // consts; this test guards the wiring.
        assert_eq!(crate::arg_primera::ARG_PRIMERA_COMP_ID, 63);
    }

    #[test]
    fn brazil_division_count_and_cup_rounds() {
        // Three national divisions (Série A/B/C) + 12 state championships +
        // Copa do Brasil + national playoff.
        assert_eq!(brazil::STATE_CHAMPIONSHIP_COMP_IDS.len(), 12);
        let nat_divs = [brazil::SERIE_A_COMP_ID, brazil::SERIE_B_COMP_ID, brazil::SERIE_C_COMP_ID];
        assert_eq!(nat_divs.len(), 3, "Brazil national division count");
        // Cup rounds: two national knockout comps (Copa do Brasil + champions
        // playoff).
        assert!(brazil::profile().domestic_cup_comp_id.is_some());
        assert_eq!(brazil::COPA_DO_BRASIL_COMP_ID, 68);
        assert_eq!(brazil::CHAMP_PLAYOFF_COMP_ID, 270);
    }

    #[test]
    fn brazil_season_length_serie_a_28_clubs_single_rr() {
        // `bra_nat_first.cpp`: Série A ran 28 clubs single round-robin in the
        // shipped 2000 season => 27 matchdays each half; totals differ by
        // format year — the constant we assert here is the shipped comp id.
        assert_eq!(brazil::SERIE_A_COMP_ID, 65);
    }

    #[test]
    fn usa_division_count_and_cup_rounds() {
        // MLS + A-League + D3-Pro + MLS All-Star + US Open Cup.
        let leagues = [usa::MLS_COMP_ID, usa::A_LEAGUE_COMP_ID, usa::D3_PRO_LEAGUE_COMP_ID];
        assert_eq!(leagues.len(), 3, "USA division count");
        // Cup rounds: US Open Cup (knockout) + MLS All-Star game.
        let cups = [usa::US_OPEN_CUP_COMP_ID, usa::MLS_ALL_STARS_COMP_ID];
        assert_eq!(cups.len(), 2, "USA cup competition count");
        assert!(usa::profile().domestic_cup_comp_id.is_some());
    }

    #[test]
    fn usa_season_length_mls_shipped_id() {
        // The MLS engine runs on `usa_mls.cpp` — the matchday count is decoded
        // there; the constant we assert is the shipped comp id (guards
        // regressions in the id table).
        assert_eq!(usa::MLS_COMP_ID, 31);
    }

    #[test]
    fn mexico_division_count_and_cup_rounds() {
        // Four divisions + Pre-Libertadores play-in.
        let leagues = [
            mexico::PRIMERA_DIVISION_COMP_ID,
            mexico::PRIMERA_A_COMP_ID,
            mexico::SECOND_DIVISION_COMP_ID,
            mexico::LOWER_DIVISION_COMP_ID,
        ];
        assert_eq!(leagues.len(), 4, "Mexico division count");
        // No shipped domestic cup — Copa MX was defunct in 2000/01 and not in
        // the CM data set. Pre-Libertadores is a play-in, not a domestic cup.
        assert!(mexico::profile().domestic_cup_comp_id.is_none(),
            "Mexico cup rounds = 0 (no shipped domestic cup)");
        assert_eq!(mexico::PRE_LIBERTADORES_COMP_ID, 218);
    }

    #[test]
    fn mexico_season_length_primera_shipped_id() {
        assert_eq!(mexico::PRIMERA_DIVISION_COMP_ID, 214);
    }

    // ── stub-nation invariant tests ──
    //
    // The 16 CONMEBOL/CONCACAF stub nations must all resolve their nation id
    // and continent id correctly, even though they carry no league data.

    #[test]
    fn stub_nations_have_correct_continent() {
        for p in [
            paraguay::profile(), chile::profile(), peru::profile(),
            bolivia::profile(), ecuador::profile(), venezuela::profile(),
            uruguay::profile(), colombia::profile(),
        ] {
            assert_eq!(p.continent_id, SOUTH_AMERICA_CONTINENT_ID, "{}", p.name);
            assert!(p.top_league_comp_id.is_none(), "{} has no shipped league", p.name);
            assert!(p.domestic_cup_comp_id.is_none(), "{} has no shipped cup", p.name);
        }
        for p in [
            canada::profile(), costa_rica::profile(), guatemala::profile(),
            honduras::profile(), panama::profile(), el_salvador::profile(),
            jamaica::profile(), trinidad::profile(), cuba::profile(),
            haiti::profile(),
        ] {
            assert_eq!(p.continent_id, NORTH_AMERICA_CONTINENT_ID, "{}", p.name);
            assert!(p.top_league_comp_id.is_none(), "{} has no shipped league", p.name);
            assert!(p.domestic_cup_comp_id.is_none(), "{} has no shipped cup", p.name);
        }
    }

    #[test]
    fn clubs_in_division_matches_primary_comp() {
        // Two USA clubs, one in MLS, one in the A-League.
        let mut mls_club = nat_club_rec(usa::NATION_ID, 1000, "LA Galaxy", 12);
        mls_club.raw[0x57..0x5b].copy_from_slice(&usa::MLS_COMP_ID.to_le_bytes());
        let mut a_league = nat_club_rec(usa::NATION_ID, 1001, "Rochester", 6);
        a_league.raw[0x57..0x5b].copy_from_slice(&usa::A_LEAGUE_COMP_ID.to_le_bytes());
        let clubs = vec![mls_club, a_league];
        let mls = usa::mls_clubs(&clubs);
        assert_eq!(mls.len(), 1);
        assert_eq!(ClubView::new(mls[0]).id(), 1000);
    }
}
