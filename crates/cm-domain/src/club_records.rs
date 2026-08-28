//! Club records — a port of the observable core of `club_records.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\club_records.cpp`, VA `0x00444e10..0x00454baf`).
//! Function decode + names: `reports/carve_rename_map.json` (`club_records.cpp`).
//!
//! The exe's records subsystem loads each club's records (biggest win, heaviest
//! defeat, record attendance, record signing, …) from `club records history.tmp`
//! and displays them. The prior records are shipped data; here we compute the
//! *live* records — biggest win, heaviest defeat, highest-scoring game — from the
//! match results the ported competitions generate, which is the forward half of
//! the subsystem (mirrors [`crate::simple_league`]/history being fed by play).

use serde::{Deserialize, Serialize};

use crate::{GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture};

/// One notable match from a club's perspective.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchRecord {
    pub opponent: String,
    pub goals_for: u8,
    pub goals_against: u8,
    pub home: bool,
    pub date: GameDate,
    pub competition: String,
}

impl MatchRecord {
    fn margin(&self) -> i32 {
        self.goals_for as i32 - self.goals_against as i32
    }
    fn total(&self) -> i32 {
        self.goals_for as i32 + self.goals_against as i32
    }
}

/// A club's records computed from the played fixtures.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubRecords {
    pub biggest_win: Option<MatchRecord>,
    pub heaviest_defeat: Option<MatchRecord>,
    pub highest_scoring: Option<MatchRecord>,
    pub games_played: u32,
}

/// Compute a club's records from every played fixture it appears in.
pub fn compute(club_id: u32, fixtures: &[HeadlessSeasonFixture]) -> ClubRecords {
    let mut rec = ClubRecords::default();
    for f in fixtures {
        if f.status != HeadlessFixtureStatus::Played {
            continue;
        }
        let (Some(hs), Some(as_)) = (f.home_score, f.away_score) else { continue };
        let m = if f.home_club_id == club_id {
            MatchRecord {
                opponent: f.away_club_name.clone(),
                goals_for: hs,
                goals_against: as_,
                home: true,
                date: f.date.clone(),
                competition: f.competition_name.clone(),
            }
        } else if f.away_club_id == club_id {
            MatchRecord {
                opponent: f.home_club_name.clone(),
                goals_for: as_,
                goals_against: hs,
                home: false,
                date: f.date.clone(),
                competition: f.competition_name.clone(),
            }
        } else {
            continue;
        };
        rec.games_played += 1;
        if m.margin() > 0 && rec.biggest_win.as_ref().map_or(true, |b| m.margin() > b.margin()) {
            rec.biggest_win = Some(m.clone());
        }
        if m.margin() < 0
            && rec
                .heaviest_defeat
                .as_ref()
                .map_or(true, |d| m.margin() < d.margin())
        {
            rec.heaviest_defeat = Some(m.clone());
        }
        if rec
            .highest_scoring
            .as_ref()
            .map_or(true, |h| m.total() > h.total())
        {
            rec.highest_scoring = Some(m);
        }
    }
    rec
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fx(row: u32, h: u32, hn: &str, a: u32, an: &str, hs: u8, as_: u8) -> HeadlessSeasonFixture {
        HeadlessSeasonFixture {
            row,
            competition_id: 1,
            competition_name: "L".into(),
            date: GameDate { year: 2001, month: 8, day: 1 },
            home_club_id: h,
            home_club_name: hn.into(),
            away_club_id: a,
            away_club_name: an.into(),
            status: HeadlessFixtureStatus::Played,
            home_score: Some(hs),
            away_score: Some(as_),
            match_packet: None,
            match_report: None,
            source: String::new(),
        }
    }

    #[test]
    fn computes_win_defeat_and_scoring() {
        let fixtures = vec![
            fx(0, 10, "Us", 20, "A", 5, 0), // biggest win 5-0
            fx(1, 30, "B", 10, "Us", 4, 1), // 1-4 away defeat (margin -3)
            fx(2, 10, "Us", 40, "C", 3, 3), // total 6 (highest scoring)
        ];
        let r = compute(10, &fixtures);
        assert_eq!(r.games_played, 3);
        assert_eq!(r.biggest_win.as_ref().unwrap().goals_for, 5);
        assert_eq!(r.heaviest_defeat.as_ref().unwrap().goals_against, 4);
        assert_eq!(r.highest_scoring.as_ref().unwrap().total(), 6);
    }
}
