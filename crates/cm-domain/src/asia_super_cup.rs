//! Asian Super Cup — a port of `asia_super_cup.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\eurocomp\asia_super_cup.cpp`, VA
//! `0x00410570..0x00410d6f`, vtable `0x009556c8`). Function decode + names:
//! `reports/carve_rename_map.json` (`asia_super_cup.cpp`).
//!
//! A single tie between the **Asian Club Championship holder** and the **Asian
//! Cup Winners' Cup holder** (`supercup_build_tie` `0x00410970` fetches both via
//! `competition_holder_get` `0x004b61b0`). Unlike the other cups it has **no
//! group stage** (`[esi+0x2c]=0`). This port creates the fixture once both
//! parent cups finish (their champions come from the ACN engine's completed
//! finals) and announces the winner + an honour when it is played.
//!
//! ## Fidelity notes
//! * **Participants** — exact: the two Asian club cups' champions.
//! * **Result** — a single match; a draw is decided by home advantage (the exe
//!   would replay / go to a second leg — a documented placeholder).

use serde::{Deserialize, Serialize};

use crate::{GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture};

/// `club_competition` id of the Asian Super Cup.
pub const ASIA_SUPER_CUP_COMP_ID: u32 = 111;
pub const ASIA_SUPER_CUP_NAME: &str = "Asian Super Cup";
/// Fixture `source` tag so the tie can be found after it is created.
pub const TAG_SUPER: &str = "ASIA-SUPERCUP";
/// Honour-type id for the Asian Super Cup winner.
pub const ASIA_SUPER_CUP_HONOUR: u32 = 0x83c;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsiaSuperCupState {
    pub year: u16,
    /// Whether the tie fixture has been created (both parents finished).
    pub created: bool,
    /// Whether the winner has been announced.
    pub announced: bool,
}

impl AsiaSuperCupState {
    pub fn new(year: u16) -> Self {
        Self { year, created: false, announced: false }
    }
}

/// The tie fixture between the two cup holders. Port of `supercup_build_tie`.
pub fn build_tie(
    year: u16,
    row: u32,
    date: GameDate,
    home: (u32, String),
    away: (u32, String),
) -> HeadlessSeasonFixture {
    HeadlessSeasonFixture {
        row,
        competition_id: ASIA_SUPER_CUP_COMP_ID,
        competition_name: ASIA_SUPER_CUP_NAME.to_string(),
        date,
        home_club_id: home.0,
        home_club_name: home.1,
        away_club_id: away.0,
        away_club_name: away.1,
        status: HeadlessFixtureStatus::Pending,
        home_score: None,
        away_score: None,
        match_packet: None,
        match_report: None,
        source: format!(
            "{TAG_SUPER} {year} (asia_super_cup.cpp supercup_build_tie 0x00410970; draw decided by home advantage pending replay decode)"
        ),
    }
}

/// The played tie, if present, as `&fixture`.
pub fn tie_fixture(fixtures: &[HeadlessSeasonFixture]) -> Option<&HeadlessSeasonFixture> {
    fixtures
        .iter()
        .find(|f| f.competition_id == ASIA_SUPER_CUP_COMP_ID && f.source.contains(TAG_SUPER))
}

/// The winner of a played tie: higher score, draw → home.
pub fn winner(f: &HeadlessSeasonFixture) -> Option<(u32, String)> {
    let (hs, as_) = (f.home_score?, f.away_score?);
    Some(if hs >= as_ {
        (f.home_club_id, f.home_club_name.clone())
    } else {
        (f.away_club_id, f.away_club_name.clone())
    })
}
