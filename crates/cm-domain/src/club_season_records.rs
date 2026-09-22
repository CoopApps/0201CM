//! Per-club, per-season match-records accumulator behind the club History
//! **Results** and **Sequences** tabs.
//!
//! The exe stores per-season SUMMARIES per club (biggest win, best streak, …),
//! not every match — so this keeps an O(clubs × seasons) accumulator updated
//! incrementally as each fixture resolves, never an O(matches) log. Matches must
//! be fed in chronological order (the tick resolves fixtures by date), which is
//! what makes the streak tracking correct.
//!
//! Like the exe's record body, records store IDS + the raw date (not display
//! strings): `opponent_id`, `competition_id`, a small round descriptor, home
//! flag, `GameDate`. Display strings are formatted lazily at view time via
//! closures (`result_row`/`result_rows`), so the ledger stays compact.
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
use crate::GameDate;

/// `D.M.YY` — day and month unpadded, year the 2-digit zero-padded remainder
/// (capture-confirmed: `2.12.09`, `23.9.37`, `6.4.30`).
pub fn fmt_date(d: GameDate) -> String {
    format!("{}.{}.{:02}", d.day, d.month, d.year % 100)
}

/// One resolved match from a club's perspective, fed to the accumulator. The
/// caller supplies ids + the round descriptor (empty for leagues) + is_league.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchInput {
    pub our_goals: u32,
    pub their_goals: u32,
    pub opponent_id: u32,
    /// True when this club played at home.
    pub home: bool,
    pub competition_id: u32,
    /// Round descriptor appended after the competition name, e.g. "3rd Rnd",
    /// "Playoff Semi Final Leg 2"; empty for a plain league fixture.
    pub round: String,
    /// Division-type competition (not a cup / play-off) — decoded `FUN_004b6b30`.
    pub is_league: bool,
    pub date: GameDate,
}

/// A record match kept for the Results tab (ids + raw date; formatted lazily).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchRecord {
    pub our_goals: u32,
    pub their_goals: u32,
    pub opponent_id: u32,
    pub home: bool,
    pub competition_id: u32,
    pub round: String,
    pub date: GameDate,
}

impl MatchRecord {
    fn from_input(m: &MatchInput) -> Self {
        Self {
            our_goals: m.our_goals,
            their_goals: m.their_goals,
            opponent_id: m.opponent_id,
            home: m.home,
            competition_id: m.competition_id,
            round: m.round.clone(),
            date: m.date,
        }
    }
    fn win_margin(&self) -> i64 {
        self.our_goals as i64 - self.their_goals as i64
    }
    fn total(&self) -> u32 {
        self.our_goals + self.their_goals
    }

    /// New win replaces `stored` as Biggest Win? Decoded `FUN_007ccaa0`: larger
    /// MARGIN; equal margin → more goals SCORED; full tie → keep earlier.
    fn beats_win(&self, stored: &MatchRecord) -> bool {
        let (nm, sm) = (self.win_margin(), stored.win_margin());
        if nm != sm { nm > sm } else { self.our_goals > stored.our_goals }
    }
    /// New loss replaces `stored` as Biggest Defeat? Decoded `FUN_007ccb70`:
    /// larger losing margin; equal → more goals CONCEDED; else keep earlier.
    fn beats_defeat(&self, stored: &MatchRecord) -> bool {
        let nm = self.their_goals as i64 - self.our_goals as i64;
        let sm = stored.their_goals as i64 - stored.our_goals as i64;
        if nm != sm { nm > sm } else { self.their_goals > stored.their_goals }
    }
    /// New replaces `stored` as Highest Scoring? Decoded `FUN_007ccc50`: TOTAL
    /// goals, strictly greater; ties keep earlier; exe guard: only compares when
    /// `stored.their_goals + self.our_goals != 0`.
    fn beats_scoring(&self, stored: &MatchRecord) -> bool {
        if stored.their_goals + self.our_goals == 0 {
            return false;
        }
        self.total() > stored.total()
    }

    fn to_result_row(
        &self,
        season: &str,
        club_name: &impl Fn(u32) -> String,
        comp_disp: &impl Fn(u32, &str) -> String,
    ) -> ResultRow {
        ResultRow {
            season: season.to_string(),
            score: format!("{}-{}", self.our_goals, self.their_goals),
            opponent: club_name(self.opponent_id),
            venue: if self.home { "H".to_string() } else { "A".to_string() },
            competition: comp_disp(self.competition_id, &self.round),
            date: fmt_date(self.date),
        }
    }
}

/// A run-length tracker: current run + best run with its date range.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Streak {
    cur_len: u32,
    cur_start: Option<GameDate>,
    best_len: u32,
    best_start: Option<GameDate>,
    best_end: Option<GameDate>,
}

impl Streak {
    /// Extend the current run with a match on `date`; promote to best if longer.
    /// Start is stamped when the counter first reaches 1 (decoded); end is the
    /// match that achieved the best length.
    fn extend(&mut self, date: GameDate) {
        if self.cur_len == 0 {
            self.cur_start = Some(date);
        }
        self.cur_len += 1;
        if self.cur_len > self.best_len {
            self.best_len = self.cur_len;
            self.best_start = self.cur_start;
            self.best_end = Some(date);
        }
    }
    fn reset(&mut self) {
        self.cur_len = 0;
        self.cur_start = None;
    }
    fn to_sequence_row(&self, season: &str) -> SequenceRow {
        // length-1 streak shows just the length (no range) — capture-confirmed.
        let (start, end) = if self.best_len >= 2 {
            (
                self.best_start.map(fmt_date).unwrap_or_default(),
                self.best_end.map(fmt_date).unwrap_or_default(),
            )
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
    biggest_win: Option<MatchRecord>,
    biggest_league_win: Option<MatchRecord>,
    biggest_defeat: Option<MatchRecord>,
    highest_scoring: Option<MatchRecord>,
    highest_scoring_league: Option<MatchRecord>,
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
        let r = MatchRecord::from_input(m);
        let win = r.our_goals > r.their_goals;
        let loss = r.our_goals < r.their_goals;

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
        if loss && self.biggest_defeat.as_ref().map_or(true, |b| r.beats_defeat(b)) {
            self.biggest_defeat = Some(r.clone());
        }
        if self.highest_scoring.as_ref().map_or(true, |b| r.beats_scoring(b)) {
            self.highest_scoring = Some(r.clone());
        }
        if m.is_league
            && self.highest_scoring_league.as_ref().map_or(true, |b| r.beats_scoring(b))
        {
            self.highest_scoring_league = Some(r.clone());
        }

        // Streaks (decoded transitions).
        if win { self.won.extend(r.date); } else { self.won.reset(); }
        if loss { self.lost.extend(r.date); } else { self.lost.reset(); }
        if !loss { self.unbeaten.extend(r.date); } else { self.unbeaten.reset(); }
        if !win { self.winless.extend(r.date); } else { self.winless.reset(); }
    }

    /// Result row for one `RESULTS_VIEW_MODES` mode (formatted via closures).
    pub fn result_row(
        &self,
        mode: &str,
        club_name: &impl Fn(u32) -> String,
        comp_disp: &impl Fn(u32, &str) -> String,
    ) -> Option<ResultRow> {
        let s = season_label(self.season_year);
        let m = match mode {
            "Biggest Win" => &self.biggest_win,
            "Biggest League Win" => &self.biggest_league_win,
            "Biggest Defeat" => &self.biggest_defeat,
            "Highest Scoring Game" => &self.highest_scoring,
            "Highest Scoring League Game" => &self.highest_scoring_league,
            _ => return None,
        };
        m.as_ref().map(|r| r.to_result_row(&s, club_name, comp_disp))
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
pub fn result_rows(
    records: &[ClubSeasonRecords],
    club_id: u32,
    mode: &str,
    club_name: &impl Fn(u32) -> String,
    comp_disp: &impl Fn(u32, &str) -> String,
) -> Vec<ResultRow> {
    let mut rows: Vec<(u16, ResultRow)> = records
        .iter()
        .filter(|r| r.club_id == club_id)
        .filter_map(|r| r.result_row(mode, club_name, comp_disp).map(|row| (r.season_year, row)))
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
