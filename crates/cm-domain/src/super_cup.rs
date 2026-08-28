//! A reusable domestic **super cup** — a single tie between a nation's league
//! champion and its cup winner. This is the shape of `bel_super.cpp`
//! (VA `0x00421a40`, vtable `0x00955aa8`) and every other national super cup
//! (and mirrors the already-ported Asian Super Cup, [`crate::asia_super_cup`]).
//!
//! The two participants are the champions of a league and a cup that finish
//! earlier in the game; the port looks them up in [`crate::honours`] (which the
//! league/cup ports already populate), creates the tie once both are known, and
//! announces the winner + an honour when it is played.

use serde::{Deserialize, Serialize};

use crate::honours::Honour;
use crate::{GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuperCupState {
    pub year: u16,
    /// Display name, e.g. "Belgian Super Cup".
    pub name: String,
    /// Fixtures' competition id.
    pub runtime_comp_id: u32,
    /// Honour title of the league whose champion is one participant.
    pub league_competition: String,
    /// Honour title of the cup whose winner is the other participant.
    pub cup_competition: String,
    pub champion_honour: u32,
    pub created: bool,
    pub announced: bool,
}

impl SuperCupState {
    fn tag(&self) -> String {
        format!("SUPER{}", self.runtime_comp_id)
    }
}

/// Find a recorded champion (club id + name) of `competition` in `year`.
fn champion_in<'a>(honours: &'a [Honour], competition: &str, year: u16) -> Option<(u32, String)> {
    honours
        .iter()
        .find(|h| h.competition == competition && h.year == year)
        .map(|h| (h.club_id, h.club_name.clone()))
}

/// The result of one advancement step.
#[derive(Debug, Default)]
pub struct SuperCupAdvance {
    pub new_fixtures: Vec<HeadlessSeasonFixture>,
    pub news: Vec<(String, String)>,
    pub honours: Vec<Honour>,
    pub created: bool,
    pub announced: bool,
}

/// Create the tie once both champions are known; announce the winner once it is
/// played. Pure — the caller applies the results and flips the state flags.
pub fn advance(
    state: &SuperCupState,
    fixtures: &[HeadlessSeasonFixture],
    honours: &[Honour],
    date: &GameDate,
    next_row: u32,
) -> SuperCupAdvance {
    let mut out = SuperCupAdvance::default();
    if !state.created {
        let home = champion_in(honours, &state.league_competition, state.year);
        let away = champion_in(honours, &state.cup_competition, state.year);
        if let (Some(home), Some(away)) = (home, away) {
            out.news.push((
                "competition".into(),
                format!("{} - {} vs {} will contest the {} {}", state.name, home.1, away.1, state.year, state.name),
            ));
            out.new_fixtures.push(HeadlessSeasonFixture {
                row: next_row,
                competition_id: state.runtime_comp_id,
                competition_name: state.name.clone(),
                // Played two weeks after the later of the two finals.
                date: crate::CmPackedDate::from_game_date(date.clone()).add_days(14).to_game_date(),
                home_club_id: home.0,
                home_club_name: home.1,
                away_club_id: away.0,
                away_club_name: away.1,
                status: HeadlessFixtureStatus::Pending,
                home_score: None,
                away_score: None,
                match_packet: None,
                match_report: None,
                source: format!("{} {} (bel_super.cpp / super_cup; draw to home)", state.tag(), state.year),
            });
            out.created = true;
        }
        return out;
    }
    if !state.announced {
        let tag = state.tag();
        if let Some(f) = fixtures
            .iter()
            .find(|f| f.competition_id == state.runtime_comp_id && f.source.contains(&tag))
            .filter(|f| f.status == HeadlessFixtureStatus::Played)
        {
            if let (Some(hs), Some(as_)) = (f.home_score, f.away_score) {
                let (id, name) = if hs >= as_ {
                    (f.home_club_id, f.home_club_name.clone())
                } else {
                    (f.away_club_id, f.away_club_name.clone())
                };
                out.news.push((
                    "competition".into(),
                    format!("{name} - win the {} {}", state.name, state.year),
                ));
                out.honours.push(Honour::champion(
                    state.year,
                    state.champion_honour,
                    state.name.clone(),
                    id,
                    name,
                ));
                out.announced = true;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ties_league_champ_and_cup_winner_from_honours() {
        let state = SuperCupState {
            year: 2001,
            name: "Belgian Super Cup".into(),
            runtime_comp_id: 0x7f00,
            league_competition: "Belgian First Division".into(),
            cup_competition: "Belgian Cup".into(),
            champion_honour: 0x83c,
            created: false,
            announced: false,
        };
        let honours = vec![
            Honour::champion(2001, 0x7d0, "Belgian First Division", 10, "Club Brugge"),
            Honour::champion(2001, 0x7d0, "Belgian Cup", 20, "Anderlecht"),
        ];
        let date = GameDate { year: 2001, month: 11, day: 20 };
        let adv = advance(&state, &[], &honours, &date, 0);
        assert!(adv.created);
        assert_eq!(adv.new_fixtures.len(), 1);
        let f = &adv.new_fixtures[0];
        assert_eq!((f.home_club_id, f.away_club_id), (10, 20));
    }
}
