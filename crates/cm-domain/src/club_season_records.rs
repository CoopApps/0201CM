//! Per-club, per-season match-records accumulator behind the club History
//! **Results** and **Sequences** tabs.
//!
//! The exe stores per-season SUMMARIES per club (biggest win, best streak, …),
//! not every match — so this keeps an O(clubs × seasons) accumulator updated
//! incrementally as each fixture resolves, never an O(matches) log. Matches must
//! be fed in chronological order (the tick resolves fixtures by date), which is
//! what makes the streak tracking correct.
//!
//! Rules decoded byte-exact from the exe record engine `FUN_00445220` (driver
//! `FUN_0069b930`), compares `FUN_007ccaa0`/`FUN_007ccb70`/`FUN_007ccc50`, and
//! the league test `FUN_004b6b30` — NOT reconstructed from the display. Biggest
//! Win/Defeat by goal margin with a goals-scored/conceded tiebreak, earliest
//! kept on a full tie; Highest Scoring by total goals (strict, with the exe's
//! `stored.against + new.for != 0` guard); League variants exclude cups AND
//! play-offs. Streaks are continuous across ALL competitions in date order:
//! win→won++/unbeaten++, draw→unbeaten++/winless++, loss→lost++/winless++,
//! each keeping the season max. The engine gates on a real recorded competition
//! (`FUN_005b09e0`/`FUN_00525450`), so the CALLER must feed only recorded
//! competitive fixtures.
//!
//! Decode: reports/club_history_screen_decode.md captures 34 (Sequences) & 35
//! (Results). Row shapes:
//!  - Results:  `<score> v <opponent> (H|A) <competition+round> <date>`
//!  - Sequences: `<length> - <start> to <end>` (a length-1 streak shows just the
//!    length).

use serde::{Deserialize, Serialize};

use crate::club_history::{season_label, ResultRow, SequenceRow};

/// One resolved match from a club's perspective, fed to the accumulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchInput {
    pub our_goals: u32,
    pub their_goals: u32,
    pub opponent: String,
    /// "H" or "A".
    pub venue: String,
    /// Display competition incl. round, e.g. "Premier Division",
    /// "FA Cup 5th Rnd".
    pub competition: String,
    pub is_league: bool,
    /// Display date, e.g. "23.9.37".
    pub date: String,
}

/// A record match kept for the Results tab.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchRef {
    pub our_goals: u32,
    pub their_goals: u32,
    pub opponent: String,
    pub venue: String,
    pub competition: String,
    pub date: String,
}

impl MatchRef {
    fn from_input(m: &MatchInput) -> Self {
        Self {
            our_goals: m.our_goals,
            their_goals: m.their_goals,
            opponent: m.opponent.clone(),
            venue: m.venue.clone(),
            competition: m.competition.clone(),
            date: m.date.clone(),
        }
    }
    fn score(&self) -> String {
        format!("{}-{}", self.our_goals, self.their_goals)
    }
    fn win_margin(&self) -> i64 {
        self.our_goals as i64 - self.their_goals as i64
    }
    fn total(&self) -> u32 {
        self.our_goals + self.their_goals
    }

    /// Should `self` (a new win) replace `stored` as the Biggest Win?
    /// Decoded `FUN_007ccaa0`: larger goal MARGIN wins; on equal margin, more
    /// goals SCORED wins; otherwise keep the earlier-achieved record.
    fn beats_win(&self, stored: &MatchRef) -> bool {
        let (nm, sm) = (self.win_margin(), stored.win_margin());
        if nm != sm { nm > sm } else { self.our_goals > stored.our_goals }
    }

    /// Should `self` (a new loss) replace `stored` as the Biggest Defeat?
    /// Decoded `FUN_007ccb70`: larger losing margin (their-our); on equal margin,
    /// more goals CONCEDED; otherwise keep the earlier record.
    fn beats_defeat(&self, stored: &MatchRef) -> bool {
        let nm = self.their_goals as i64 - self.our_goals as i64;
        let sm = stored.their_goals as i64 - stored.our_goals as i64;
        if nm != sm { nm > sm } else { self.their_goals > stored.their_goals }
    }

    /// Should `self` replace `stored` as the Highest Scoring game? Decoded
    /// `FUN_007ccc50`: by TOTAL goals, strictly greater; ties keep the earlier.
    /// Reproduces the exe's guard exactly: it only compares when
    /// `stored.their_goals + self.our_goals != 0` (a genuine oddity of the
    /// original), otherwise keeps `stored`.
    fn beats_scoring(&self, stored: &MatchRef) -> bool {
        if stored.their_goals + self.our_goals == 0 {
            return false;
        }
        self.total() > stored.total()
    }
    fn to_result_row(&self, season: &str) -> ResultRow {
        ResultRow {
            season: season.to_string(),
            score: self.score(),
            opponent: self.opponent.clone(),
            venue: self.venue.clone(),
            competition: self.competition.clone(),
            date: self.date.clone(),
        }
    }
}

/// A run-length tracker: current run + best run with its date range.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Streak {
    cur_len: u32,
    cur_start: String,
    best_len: u32,
    best_start: String,
    best_end: String,
}

impl Streak {
    /// Extend the current run with a match on `date`; promote to best if longer.
    fn extend(&mut self, date: &str) {
        if self.cur_len == 0 {
            self.cur_start = date.to_string();
        }
        self.cur_len += 1;
        if self.cur_len > self.best_len {
            self.best_len = self.cur_len;
            self.best_start = self.cur_start.clone();
            self.best_end = date.to_string();
        }
    }
    fn reset(&mut self) {
        self.cur_len = 0;
        self.cur_start.clear();
    }
    fn to_sequence_row(&self, season: &str) -> SequenceRow {
        // length-1 streak shows just the length (no range) — capture-confirmed.
        let (start, end) = if self.best_len >= 2 {
            (self.best_start.clone(), self.best_end.clone())
        } else {
            (String::new(), String::new())
        };
        SequenceRow {
            season: season.to_string(),
            length: self.best_len.to_string(),
            start_date: start,
            end_date: end,
        }
    }
}

/// One club's accumulated records for one season.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubSeasonRecords {
    pub season_year: u16,
    pub club_id: u32,
    // Results (Option: None until the club plays a qualifying match).
    biggest_win: Option<MatchRef>,
    biggest_league_win: Option<MatchRef>,
    biggest_defeat: Option<MatchRef>,
    highest_scoring: Option<MatchRef>,
    highest_scoring_league: Option<MatchRef>,
    // Sequences.
    won: Streak,
    lost: Streak,
    unbeaten: Streak,
    winless: Streak,
}

impl ClubSeasonRecords {
    pub fn new(club_id: u32, season_year: u16) -> Self {
        Self { season_year, club_id, ..Default::default() }
    }

    /// Fold one resolved match (called in chronological order).
    pub fn update_with_match(&mut self, m: &MatchInput) {
        let r = MatchRef::from_input(m);
        let win = r.our_goals > r.their_goals;
        let loss = r.our_goals < r.their_goals;

        // Biggest win (all comps) + Biggest League Win (division-type only).
        if win {
            if self.biggest_win.as_ref().map_or(true, |b| r.beats_win(b)) {
                self.biggest_win = Some(r.clone());
            }
            if m.is_league
                && self.biggest_league_win.as_ref().map_or(true, |b| r.beats_win(b))
            {
                self.biggest_league_win = Some(r.clone());
            }
        }
        // Biggest defeat (all comps).
        if loss && self.biggest_defeat.as_ref().map_or(true, |b| r.beats_defeat(b)) {
            self.biggest_defeat = Some(r.clone());
        }
        // Highest scoring (all comps) + league variant.
        if self.highest_scoring.as_ref().map_or(true, |b| r.beats_scoring(b)) {
            self.highest_scoring = Some(r.clone());
        }
        if m.is_league
            && self.highest_scoring_league.as_ref().map_or(true, |b| r.beats_scoring(b))
        {
            self.highest_scoring_league = Some(r.clone());
        }

        // Streaks.
        if win { self.won.extend(&m.date); } else { self.won.reset(); }
        if loss { self.lost.extend(&m.date); } else { self.lost.reset(); }
        if !loss { self.unbeaten.extend(&m.date); } else { self.unbeaten.reset(); }
        if !win { self.winless.extend(&m.date); } else { self.winless.reset(); }
    }

    /// Result row for one `RESULTS_VIEW_MODES` mode, if a qualifying match exists.
    pub fn result_row(&self, mode: &str) -> Option<ResultRow> {
        let s = season_label(self.season_year);
        let m = match mode {
            "Biggest Win" => &self.biggest_win,
            "Biggest League Win" => &self.biggest_league_win,
            "Biggest Defeat" => &self.biggest_defeat,
            "Highest Scoring Game" => &self.highest_scoring,
            "Highest Scoring League Game" => &self.highest_scoring_league,
            _ => return None,
        };
        m.as_ref().map(|r| r.to_result_row(&s))
    }

    /// Sequence row for one `SEQUENCES_VIEW_MODES` mode.
    pub fn sequence_row(&self, mode: &str) -> Option<SequenceRow> {
        let s = season_label(self.season_year);
        let st = match mode {
            "Most Games Won in Row" => &self.won,
            "Most Games Lost in Row" => &self.lost,
            "Most Games Without Losing" => &self.unbeaten,
            "Most Games Without Winning" => &self.winless,
            _ => return None,
        };
        Some(st.to_sequence_row(&s))
    }
}

/// Results rows for a club across seasons (newest-first) for a given View mode.
pub fn result_rows(records: &[ClubSeasonRecords], club_id: u32, mode: &str) -> Vec<ResultRow> {
    let mut rows: Vec<(u16, ResultRow)> = records
        .iter()
        .filter(|r| r.club_id == club_id)
        .filter_map(|r| r.result_row(mode).map(|row| (r.season_year, row)))
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    rows.into_iter().map(|(_, r)| r).collect()
}

/// Sequence rows for a club across seasons (newest-first) for a given View mode.
pub fn sequence_rows(records: &[ClubSeasonRecords], club_id: u32, mode: &str) -> Vec<SequenceRow> {
    let mut rows: Vec<(u16, SequenceRow)> = records
        .iter()
        .filter(|r| r.club_id == club_id)
        .filter_map(|r| r.sequence_row(mode).map(|row| (r.season_year, row)))
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    rows.into_iter().map(|(_, r)| r).collect()
}
