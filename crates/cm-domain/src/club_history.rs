//! Club History screen view-model — the data behind the club History page
//! (decode: reports/club_history_screen_decode.md, GDI Frida captures in
//! reports/captures/). DB-driven, no heuristics: honours come straight from
//! `references.club_comp_history`; the record category lists are exactly as
//! captured.
//!
//! Coverage so far (captured + decoded): the two view-tab rows, the Records
//! view (all-time + this-season category lists), and the Competitions view
//! (default "Honours"). The other tabs (Landmarks/Positions/Attendances/
//! Results/Sequences/Players/Transfers) are not captured yet.

use serde::{Deserialize, Serialize};

/// Top view-tab row (y=90 in the capture).
pub const HISTORY_TOP_TABS: [&str; 5] =
    ["Competitions", "Landmarks", "Records", "Positions", "Attendances"];
/// Bottom view-tab row (y=520).
pub const HISTORY_BOTTOM_TABS: [&str; 4] = ["Results", "Sequences", "Players", "Transfers"];

/// The Players tab "View" dropdown, in capture order (Liverpool/Italian 2037).
/// These are the per-season player leaders; they mirror the Records tab's
/// PLAYER categories but add "Most Capped Player" and "Fans Player Of The Year"
/// and omit the team/streak + transfer records. Row FORMAT is per-mode (see
/// `PlayerRecordRow`): season-leader (`<player> - <count>`), value-first
/// (`<count> - <player>`), match-event, or with an `extra` qualifier.
pub const PLAYERS_VIEW_MODES: [&str; 13] = [
    "Top Goalscorer",
    "Top League Goalscorer",
    "Most Goals in Match",
    "Most Assists",
    "Highest Average Rating",
    "Most Man of Match",
    "Worst Discipline",
    "Most League Apps for Club",
    "Most League Goals for Club",
    "Youngest Player",
    "Oldest Player",
    "Most Capped Player",
    "Fans Player Of The Year",
];

/// A club honour: `Third Division` · `Runners Up` · `1986, 1994`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HonourRow {
    pub competition: String,
    pub achievement: String,
    pub years: String,
}

/// A Records-view row: label at x=110, value at x=334.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRow {
    pub label: String,
    /// `"-"` when the record hasn't happened yet this period (accrues live).
    pub value: String,
}

/// A Players-tab row: the season's top player for the current View mode. The
/// View dropdown carries the PLAYER record categories of the Records tab (Top
/// Goalscorer, Top League Goalscorer, Most Goals in Match, Most Assists, Highest
/// Average Rating, Most Man of Match, …), LINKED like Transfers↔Records (Records
/// = all-time leader; Players = per-season). EMPTY at game start;
/// runtime-populated from per-season per-player stats.
///
/// TWO row shapes, capture-verified (Liverpool 2037):
///  - season-leader modes (Top Goalscorer, Top League Goalscorer): `<player> -
///    <value>` with FULL player name and empty match fields;
///  - match-event modes (Most Goals in Match, …): `<value> - <abbrev> v
///    <opponent> (H|A) <competition+round> <date>`, player name ABBREVIATED to
///    `Initial.Surname`, all match fields set.
///
/// The DISPLAY ORDER and name style are per-mode formatting rules over the same
/// fields (capture-verified): Top Goalscorer / Top League Goalscorer render
/// `<player> - <value>` (player first, full name); Most Assists renders `<value>
/// - <player>` (value first, full name); Most Goals in Match renders `<value> -
/// <abbrev> v …` (value first, abbreviated); Highest Average Rating renders
/// `<rating> - <player> (<N> apps)` (the apps count in `extra`). The renderer
/// picks the format from the active View mode; the stored fields do not change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerRecordRow {
    pub season: String,      // "2037/8"
    pub player: String,      // "Neil Lake" (season modes) / "R.Atkinson" (match modes)
    pub value: String,       // "24" (season count) / "4" (goals in the match)
    pub opponent: String,    // match modes only, else ""
    pub venue: String,       // "H"/"A" for match modes, else ""
    pub competition: String, // "First Division"/"League Cup 2nd Rnd" (match modes), else ""
    pub date: String,        // "8.11.14" (match modes), else ""
    /// Trailing qualifier for modes that need one, e.g. "(34 apps)" in Highest
    /// Average Rating (`<rating> - <player> (<N> apps)`); else "".
    pub extra: String,
}

/// A Transfers-tab row (History → Transfers, distinct from the Club Transfers
/// activity screen): the season's value for the current View mode. The tab has
/// FOUR View modes, and they are the SAME four transfer categories as the
/// Records tab (must be computed from ONE per-season transfer ledger so the two
/// screens stay linked): Records shows the all-time aggregate (MAX of the two
/// "Highest" records, SUM of the two "Total"s), Transfers shows the per-season
/// value.
///  - Highest Transfer Fee Paid:     `<fee> - <player> from <selling club> - <date>`
///  - Highest Transfer Fee Received: `<fee> - <player> to <buying club> - <date>`
///  - Total Transfer Spending:       per-season total `<£amount>` (no player/club/date)
///  - Total Transfer Income:         per-season total `<£amount>`
/// Empty-season sentinel is view-mode-dependent (capture-confirmed): the two
/// "Highest" modes show `value = "-"`, the two "Total" modes show `value = "£0"`.
/// EMPTY at game start;
/// runtime-populated from transfer events. Structure capture-verified
/// (Scunthorpe 2037).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferRecordRow {
    pub season: String,     // "2037/8"
    pub value: String,      // "£275,000" or "-"
    pub player: String,     // "Daniel Perry" ("" when "-")
    pub other_club: String, // selling club (paid) / buying club (received)
    pub date: String,       // "30.6.37" ("" when "-")
}

/// An Attendances-tab row: season at x=110, detail at x=190. The tab has four
/// View modes (Highest Attendance / Highest Gate Receipts / Lowest Attendance /
/// Average Attendance). For the match-based modes the row names the season's
/// extreme fixture: `<value> v <opponent> <competition+round> <date>`. For
/// Average, only `season` + `value` (mean) are set. EMPTY at game start;
/// runtime-populated from per-match attendance/receipts. Structure
/// capture-verified (Scunthorpe 2037).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendanceRow {
    pub season: String,      // "2037/8"
    pub value: String,       // attendance "7275", receipts, or average
    pub opponent: String,    // "Rushden" ("" for Average)
    pub competition: String, // "Second Division", "FA Cup 3rd Rnd" ("" for Average)
    pub date: String,        // "14.11.37" ("" for Average)
}

/// The Results tab "View" dropdown, in capture order (2037). Per-season record
/// MATCH for the selected kind.
pub const RESULTS_VIEW_MODES: [&str; 5] = [
    "Biggest Win",
    "Biggest League Win",
    "Biggest Defeat",
    "Highest Scoring Game",
    "Highest Scoring League Game",
];

/// A Results-tab row: the season's record match for the current View mode,
/// `<score> v <opponent> (H|A) <competition+round> <date>` (e.g. `9-0 v
/// Liverpool (A) Premier Division 3.4.32`). EMPTY at game start;
/// runtime-populated from per-season match results. Structure capture-verified
/// (2037).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultRow {
    pub season: String,      // "2037/8"
    pub score: String,       // "4-0"
    pub opponent: String,    // "Liverpool"
    pub venue: String,       // "H"/"A"
    pub competition: String, // "Premier Division" / "FA Cup 5th Rnd"
    pub date: String,        // "23.9.37"
}

/// The Sequences tab "View" dropdown, in capture order (Scunthorpe/Liverpool
/// 2037). Per-season best streak for the selected kind.
pub const SEQUENCES_VIEW_MODES: [&str; 4] = [
    "Most Games Won in Row",
    "Most Games Lost in Row",
    "Most Games Without Losing",
    "Most Games Without Winning",
];

/// A Sequences-tab row: the season's best streak for the current View mode,
/// `<length> - <start date> to <end date>` (a length-1 streak shows just
/// `<length>`, no range). EMPTY at game start; runtime-populated from per-season
/// match results. Structure capture-verified (2037).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceRow {
    pub season: String,     // "2037/8"
    pub length: String,     // "6"
    pub start_date: String, // "1.10.37" ("" for a single-game streak)
    pub end_date: String,   // "28.10.37" ("" for a single-game streak)
}

/// A Positions-tab row: season at x=110, `<ordinal> in <division>` at x=190.
/// One row per season the club has completed (+ the live current season),
/// newest-first, across all divisions. Same source as `LeagueSeasonRow`: the
/// save's archived per-season finishes + live standings (runtime, not a static
/// World table). Structure capture-verified (Scunthorpe 2037).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionRow {
    pub season: String,   // "2037/8"
    pub position: String, // ordinal, e.g. "1st", "22nd"
    pub division: String, // league LONG name, e.g. "Second Division"
}

/// A Landmarks-tab row: date at x=110, description at x=194. A per-club
/// milestone event log (manager hired/sacked/resigned, relegation/promotion,
/// …), newest-first. EMPTY at game start; rows accrue from game events during
/// play (runtime, like News). Structure capture-verified (Liverpool 2037).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LandmarkRow {
    /// `D.M.YY` (e.g. `8.5.32`).
    pub date: String,
    /// e.g. `Hired Max Bell`, `Relegation from English Premier Division`.
    pub description: String,
}

/// A per-season league record row (Competitions view, league scope).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeagueSeasonRow {
    pub season: String,   // "2001/2"
    pub position: String, // "3rd"
    pub league: String,   // "Conference"
    pub played: u32,
    pub won: u32,
    pub drawn: u32,
    pub lost: u32,
    pub goals_for: u32,
    pub goals_against: u32,
    pub points: u32,
}

/// The whole club History screen model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubHistoryView {
    pub club_id: u32,
    pub club_name: String,
    /// `"<Club> History"` — the captured title.
    pub title: String,
    pub top_tabs: Vec<String>,
    pub bottom_tabs: Vec<String>,
    /// Competitions tab, default "Honours" view.
    pub honours: Vec<HonourRow>,
    /// The Competitions "View" dropdown items (Honours + the club's competitions).
    pub view_menu: Vec<String>,
    /// Records tab, all-time period.
    pub records_all_time: Vec<RecordRow>,
    /// Records tab, this-season period (`<<<`/`>>>` pager).
    pub records_this_season: Vec<RecordRow>,
    /// Competitions tab, league scope — one row per season.
    pub league_seasons: Vec<LeagueSeasonRow>,
    /// Landmarks tab — per-club milestone event log, newest-first. EMPTY at
    /// game start (capture-confirmed); accrues from game events during play.
    pub landmarks: Vec<LandmarkRow>,
    /// Positions tab — one row per season, newest-first, across all divisions.
    /// The tab has TWO View modes: "Highest League Position" (default) and
    /// "Lowest League Position" — each season's best/worst league position (and
    /// the division held at that moment; captures show the division can differ
    /// between the two views across a promotion/relegation boundary). At game
    /// start only the live current season exists; past seasons accrue at
    /// season-end (same source as `league_seasons`; runtime layer tracks the
    /// per-season position extreme, not this static World view).
    pub positions: Vec<PositionRow>,
    /// Attendances tab — per-season, four View modes (Highest/Lowest Attendance,
    /// Highest Gate Receipts, Average Attendance). EMPTY at game start; the
    /// runtime tracks per-season attendance/receipts + the extreme fixture.
    pub attendances: Vec<AttendanceRow>,
    /// Transfers tab (History) — per-season record transfer for the current View
    /// mode. EMPTY at game start; runtime-populated from transfer events.
    pub transfer_records: Vec<TransferRecordRow>,
    /// Players tab — per-season top player for the current View mode (Top
    /// Goalscorer, etc.). EMPTY at game start; runtime-populated from per-season
    /// player stats.
    pub player_records: Vec<PlayerRecordRow>,
    /// Sequences tab — per-season best streak for the current View mode
    /// (Most Games Won/Lost in Row, Most Games Without Losing/Winning). EMPTY at
    /// game start; runtime-populated from per-season match results.
    pub sequences: Vec<SequenceRow>,
    /// Results tab — per-season record match for the current View mode (Biggest
    /// Win/League Win/Defeat, Highest Scoring [League] Game). EMPTY at game
    /// start; runtime-populated from per-season match results.
    pub results: Vec<ResultRow>,
}

/// The record category labels, per period (captured exactly; see the decode).
pub const RECORDS_ALL_TIME_LABELS: [&str; 17] = [
    "Most Games Without Losing",
    "Most Games Without Winning",
    "Top Goalscorer",
    "Top League Goalscorer",
    "Most Goals in Match",
    "Most Assists",
    "Highest Average Rating",
    "Most Man of Match",
    "Worst Discipline",
    "Most League Apps for Club",
    "Most League Goals for Club",
    "Youngest Player",
    "Oldest Player",
    "Highest Transfer Fee Paid",
    "Highest Transfer Fee Received",
    "Total Transfer Spending",
    "Total Transfer Income",
];
pub const RECORDS_THIS_SEASON_LABELS: [&str; 17] = [
    "Most Games Without Winning",
    "Top Goalscorer",
    "Top League Goalscorer",
    "Most Goals in Match",
    "Most Assists",
    "Highest Average Rating",
    "Most Man of Match",
    "Worst Discipline",
    "Most League Apps for Club",
    "Most League Goals for Club",
    "Youngest Player",
    "Oldest Player",
    "Highest Transfer Fee Paid",
    "Highest Transfer Fee Received",
    "Total Transfer Spending",
    "Total Transfer Income",
    "Fans Player Of The Year",
];

/// English ordinal for a table position: 1→"1st", 2→"2nd", 3→"3rd", 11→"11th",
/// 21→"21st", 22→"22nd", 23→"23rd".
pub fn ordinal_str(n: usize) -> String {
    let suffix = match (n % 100, n % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// Season label from a starting year: 2001 → "2001/2", 2029 → "2029/0".
// exe FUN_00652cd0 (key_nation.cpp): renders the split-year label `"%d/%d",
// year,(year+1)%10`. PARTIAL — the port ports only this split branch; it does
// NOT yet do the per-nation calendar-vs-split decision, the `season_year==0 →
// current year (DAT_00acde92)` fallback, or the split `-1` day-boundary adjust
// (FUN_00652ff0). The per-nation boundary source is FUN_00652a00 (nation →
// DAT_00b4bc70 index; see league_calendar.rs). Needed to bucket records by
// season byte-exactly — see reports/club_history_screen_decode.md TARGET 3.
// GDI-REG: 00652cd0 PORTED_PARTIAL
pub fn season_label(start_year: u16) -> String {
    format!("{}/{}", start_year, (start_year + 1) % 10)
}

/// Build the Positions + league-scope season rows for one club from the save's
/// archived league tables (newest-season first). Pure so it can be tested
/// without constructing a whole `RuntimeSaveGame`. `comp_name` maps a
/// competition id to its display (division) name.
// GDI-REG: 006684e0 PORTED_PARTIAL
// GDI-REG: 00667aa0 PORTED_BEHAVIOURAL
// GDI-REG: 00447d20 PORTED_PARTIAL
pub fn season_rows_for_club(
    tables: &[crate::ArchivedLeagueTable],
    club_id: u32,
    comp_name: impl Fn(u32) -> String,
) -> (Vec<PositionRow>, Vec<LeagueSeasonRow>) {
    // (season_year, PositionRow, LeagueSeasonRow) then sort newest-first.
    let mut acc: Vec<(u16, PositionRow, LeagueSeasonRow)> = Vec::new();
    for tbl in tables {
        // Standard football order to derive the club's final position:
        // points, then goal difference, then goals for (all descending).
        let mut rows: Vec<&crate::HeadlessSeasonStanding> = tbl.rows.iter().collect();
        rows.sort_by(|a, b| {
            b.points
                .cmp(&a.points)
                .then(b.goal_difference.cmp(&a.goal_difference))
                .then(b.goals_for.cmp(&a.goals_for))
        });
        let Some(rank) = rows.iter().position(|r| r.club_id == club_id) else {
            continue;
        };
        let r = rows[rank];
        let season = season_label(tbl.season_year);
        let league = comp_name(tbl.competition_id);
        let pos = ordinal_str(rank + 1);
        acc.push((
            tbl.season_year,
            PositionRow { season: season.clone(), position: pos.clone(), division: league.clone() },
            LeagueSeasonRow {
                season,
                position: pos,
                league,
                played: r.played,
                won: r.won,
                drawn: r.drawn,
                lost: r.lost,
                goals_for: r.goals_for,
                goals_against: r.goals_against,
                points: r.points,
            },
        ));
    }
    acc.sort_by(|a, b| b.0.cmp(&a.0)); // newest season first
    let positions = acc.iter().map(|(_, p, _)| p.clone()).collect();
    let league_seasons = acc.into_iter().map(|(_, _, l)| l).collect();
    (positions, league_seasons)
}

/// Build the Players tab's default "Top Goalscorer" rows for one club from the
/// save's accrued player-seasons: per season, the club's top scorer (most goals
/// among players at the club that season). Pure/testable. `player_name` maps a
/// person id to a display name. Newest-season first; a season with no goals is
/// omitted (the tab shows only seasons with a scorer).
pub fn season_top_scorers_for_club(
    player_seasons: &[crate::player_profile::AccruedSeason],
    club_id: u32,
    player_name: impl Fn(u32) -> String,
) -> Vec<PlayerRecordRow> {
    use std::collections::BTreeMap;
    // year -> (best_goals, person_id)
    let mut best: BTreeMap<u16, (u32, u32)> = BTreeMap::new();
    for s in player_seasons {
        if s.club_id < 0 || s.club_id as u32 != club_id || s.goals == 0 {
            continue;
        }
        let e = best.entry(s.year).or_insert((0, 0));
        if s.goals > e.0 {
            *e = (s.goals, s.person_id);
        }
    }
    let mut rows: Vec<(u16, PlayerRecordRow)> = best
        .into_iter()
        .map(|(year, (goals, pid))| {
            (
                year,
                PlayerRecordRow {
                    season: season_label(year),
                    player: player_name(pid),
                    value: goals.to_string(),
                    opponent: String::new(),
                    venue: String::new(),
                    competition: String::new(),
                    date: String::new(),
                    extra: String::new(),
                },
            )
        })
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0)); // newest first
    rows.into_iter().map(|(_, r)| r).collect()
}

impl crate::World {
    /// Full club History view merging static DB content (honours, view menu,
    /// records apps/goals) with the save's accrued per-season data. Currently
    /// populates `positions` and `league_seasons` from `save.season_history`;
    /// the other runtime tabs (attendances/transfers/players/sequences/results/
    /// landmarks) still require their own per-season ledgers and stay empty.
    pub fn club_history_view_with_save(
        &self,
        save: &crate::RuntimeSaveGame,
        club_id: u32,
    ) -> ClubHistoryView {
        let mut view = self.club_history_view(club_id);
        // Positions/league-scope use the division LONG name (capture-confirmed:
        // "2nd in Premier Division"), unlike Honours' short names.
        let comp_long = |cid: u32| -> String {
            self.references
                .club_competitions
                .iter()
                .chain(self.references.staff_competitions.iter())
                .chain(self.references.nation_competitions.iter())
                .find(|c| c.id == cid)
                .map(|c| if c.long_name.trim().is_empty() { c.short_name.clone() } else { c.long_name.clone() })
                .unwrap_or_else(|| format!("Competition {cid}"))
        };
        let (positions, league_seasons) =
            season_rows_for_club(&save.season_history, club_id, comp_long);
        view.positions = positions;
        view.league_seasons = league_seasons;

        // Players tab, default "Top Goalscorer" mode, from accrued player-seasons.
        let player_name = |pid: u32| -> String {
            self.staff
                .type6
                .iter()
                .find(|p| p.id == pid)
                .map(|p| self.person_display_name(p))
                .unwrap_or_default()
        };
        view.player_records =
            season_top_scorers_for_club(&save.player_seasons, club_id, player_name);
        view
    }

    /// Build the club History view-model for `club_id`.
    ///
    /// DONE (DB-driven, verifiable): honours (from `references.club_comp_history`),
    /// the View-dropdown competitions, the record category lists, and the two
    /// shipped club records "Most League Apps/Goals for Club" (decoded: the club
    /// key is staff_history +0x0a, holder = max summed apps/goals per person;
    /// verified Chester -> 361/124 Stuart Rimmer).
    /// FLAGGED (value left "-"): the game-accrued records (Top Goalscorer,
    /// streaks, transfer-fee records, etc.) — empty at a fresh save, they accrue
    /// live from match/season events as the game is played. NOTE: the four
    /// transfer-fee categories here (Highest Fee Paid/Received, Total Spending/
    /// Income) are LINKED to the Transfers tab's four View modes — both derive
    /// from one per-season transfer ledger (Records = all-time aggregate;
    /// Transfers = per-season), so wire them from the same source.
    // GDI-REG: 007abef0 PORTED_BEHAVIOURAL
    // GDI-REG: 007abd80 PORTED_BEHAVIOURAL
    // GDI-REG: 00449590 PORTED_BEHAVIOURAL
    // GDI-REG: 007a8090 PORTED_BEHAVIOURAL
    // GDI-REG: 007abef0 PORTED_BEHAVIOURAL
    // GDI-REG: 007abd80 PORTED_BEHAVIOURAL
    // GDI-REG: 00449590 PORTED_BEHAVIOURAL
    // GDI-REG: 007a8090 PORTED_BEHAVIOURAL
    // GDI-REG: 007abef0 PORTED_BEHAVIOURAL
    // GDI-REG: 007abd80 PORTED_BEHAVIOURAL
    pub fn club_history_view(&self, club_id: u32) -> ClubHistoryView {
        use crate::typed_records::ClubView;
        let club = self.core.clubs.iter().find(|c| ClubView::new(c).id() == club_id);
        // Full name (club+0x04) for the model; the History title uses the SHORT
        // name (club+0x38 secondary_name, e.g. "Chester") as the exe does.
        let club_name = club.map(|c| ClubView::new(c).primary_name()).unwrap_or_default();
        let short_club = club
            .map(|c| {
                let v = ClubView::new(c);
                let s = v.secondary_name();
                if s.trim().is_empty() { v.primary_name() } else { s }
            })
            .unwrap_or_default();

        // competition_id -> SHORT display name (the exe shows the competition
        // record's short_name, e.g. "Third Division" not "English Third
        // Division"). Falls back to long_name if a comp has no short name.
        let comp_name = |cid: u32| -> String {
            self.references
                .club_competitions
                .iter()
                .chain(self.references.staff_competitions.iter())
                .chain(self.references.nation_competitions.iter())
                .find(|c| c.id == cid)
                .map(|c| if c.short_name.trim().is_empty() { c.long_name.clone() } else { c.short_name.clone() })
                .unwrap_or_else(|| format!("Competition {cid}"))
        };
        // Honours: filter club_comp_history for this club across winner/runner-up/
        // third-place, group by (competition, placing), aggregate the years.
        // Placing text: winner -> "Champions" (league) / "Winners" (cup);
        // runner-up -> "Runners Up" (capture-confirmed); third -> "Third".
        use std::collections::BTreeMap;
        let cid_i = club_id as i32;
        let mut groups: BTreeMap<(u32, u8), Vec<u16>> = BTreeMap::new();
        for h in &self.references.club_comp_history {
            let placing: Option<u8> = if h.winner_club_id == Some(cid_i) {
                Some(0)
            } else if h.runner_up_club_id == Some(cid_i) {
                Some(1)
            } else if h.third_place_club_id == Some(cid_i) {
                Some(2)
            } else {
                None
            };
            if let Some(p) = placing {
                groups.entry((h.competition_id, p)).or_default().push(h.year);
            }
        }
        let mut honours: Vec<HonourRow> = groups
            .into_iter()
            .map(|((cid, p), mut years)| {
                years.sort_unstable();
                years.dedup();
                // Achievement text is capture-exact and UNIFORM across leagues
                // and cups: winner -> "Winners" (Serie A / Premier Division both
                // show "Winners", not "Champions"), runner-up -> "Runners Up",
                // third -> "Third Placed" (Pro Vercelli Serie C1/A capture).
                let achievement = match p {
                    0 => "Winners",
                    1 => "Runners Up",
                    _ => "Third Placed",
                }
                .to_string();
                HonourRow {
                    competition: comp_name(cid),
                    achievement,
                    years: years.iter().map(|y| y.to_string()).collect::<Vec<_>>().join(", "),
                }
            })
            .collect();
        // Ordering: by competition, then PLACING RANK within a competition
        // (Winners → Runners Up → Third Placed) — capture-confirmed (Pro
        // Vercelli shows Serie A Winners above Serie A Runners Up), NOT alpha by
        // achievement text. (Cross-competition order in the exe is by comp
        // prestige/id, not name; approximated here by name — flagged follow-up.)
        let placing_rank = |a: &str| match a {
            "Winners" => 0u8,
            "Runners Up" => 1,
            _ => 2,
        };
        honours.sort_by(|a, b| {
            a.competition
                .cmp(&b.competition)
                .then(placing_rank(&a.achievement).cmp(&placing_rank(&b.achievement)))
        });

        // View-dropdown. Decoded from GDI captures (Chester fresh-2001 +
        // Liverpool/Salisbury 2037): the menu is the competitions the club is
        // ENTERED IN (participation), NOT its honours — Chester's game-start
        // menu is [Honours, Domestic Leagues, Conference, FA Trophy, Vans
        // Trophy] and does NOT list its Third Division honour; by 2037 Liverpool
        // has accreted every comp it entered over the run (First Division,
        // Inter-Toto Cup, …) while defunct comps (Cup Winners Cup) drop off.
        // So: two fixed aggregates + the club's live competition entries.
        //
        // Verifiable now (from the club record's comp slots +0x57/0x5b/0x60):
        // the club's LEAGUE (Chester -> Conference, Liverpool -> Premier
        // Division). The per-season CUP entries (FA Cup/League Cup/FA Trophy/
        // Vans Trophy/Europe) live in the runtime cup-entry pools (domestic_cup
        // boot) + accrete each played season — not in this static World — so
        // they are appended by the caller when the save's cup state is present.
        let mut view_menu = vec!["Honours".to_string(), "Domestic Leagues".to_string()];
        let mut seen = std::collections::BTreeSet::new();
        if let Some(c) = club {
            for cid in ClubView::new(c).competition_ids() {
                if cid < 0 {
                    continue;
                }
                let name = comp_name(cid as u32);
                if seen.insert(name.clone()) {
                    view_menu.push(name);
                }
            }
        }

        // Shipped all-time club records: "Most League Apps/Goals for Club".
        // Decoded (task aa66fc0520829a5..): staff_history +0x0a is the CLUB
        // (resolved via the 581-byte club pool — NOT a competition; my field
        // label `competition_id` is a misnomer, and player_profile already
        // treats it as the club). The holder is max(sum(apps)) / max(sum(goals))
        // per (person, club). VERIFIED: Chester -> 361 / 124 Stuart Rimmer.
        // exe career sums: FUN_007abd80 (apps per competition), FUN_007abef0
        // (goals per competition), FUN_007ac060 (total apps) — same per-person
        // accumulation the exe does over its in-memory history rows.
        use std::collections::HashMap;
        let mut apps_sum: HashMap<u32, u64> = HashMap::new();
        let mut goals_sum: HashMap<u32, u64> = HashMap::new();
        for h in &self.references.staff_history {
            if h.competition_id == club_id {
                *apps_sum.entry(h.person_id).or_default() += h.apps as u64;
                *goals_sum.entry(h.person_id).or_default() += h.goals as u64;
            }
        }
        let holder = |m: &HashMap<u32, u64>| -> Option<String> {
            m.iter()
                .max_by_key(|(_, v)| **v)
                .filter(|(_, v)| **v > 0)
                .map(|(pid, v)| {
                    let name = self
                        .staff
                        .type6
                        .iter()
                        .find(|p| p.id == *pid)
                        .map(|p| self.person_display_name(p))
                        .unwrap_or_default();
                    if name.trim().is_empty() {
                        format!("{v}")
                    } else {
                        format!("{v} - {name}")
                    }
                })
        };
        let most_apps = holder(&apps_sum);
        let most_goals = holder(&goals_sum);

        // Fill a per-period category list. All game-accrued records are "-"
        // (empty at a fresh save; they accrue live). The two shipped club
        // records populate in the ALL-TIME period only; the THIS-SEASON period
        // resets them to "-" (the new season hasn't accrued) — capture-confirmed.
        let rec = |labels: &[&str], all_time: bool| -> Vec<RecordRow> {
            labels
                .iter()
                .map(|l| {
                    let value = match (all_time, *l) {
                        (true, "Most League Apps for Club") => {
                            most_apps.clone().unwrap_or_else(|| "-".to_string())
                        }
                        (true, "Most League Goals for Club") => {
                            most_goals.clone().unwrap_or_else(|| "-".to_string())
                        }
                        _ => "-".to_string(),
                    };
                    RecordRow { label: (*l).to_string(), value }
                })
                .collect()
        };

        ClubHistoryView {
            club_id,
            club_name: club_name.clone(),
            title: format!("{short_club} History"),
            top_tabs: HISTORY_TOP_TABS.iter().map(|s| s.to_string()).collect(),
            bottom_tabs: HISTORY_BOTTOM_TABS.iter().map(|s| s.to_string()).collect(),
            honours,
            view_menu,
            records_all_time: rec(&RECORDS_ALL_TIME_LABELS, true),
            records_this_season: rec(&RECORDS_THIS_SEASON_LABELS, false),
            league_seasons: Vec::new(),
            // Empty at game start (capture 12); the runtime milestone log fills
            // this during play (capture 13 shows the D.M.YY/description shape).
            landmarks: Vec::new(),
            // Populated at the runtime layer from archived season finishes +
            // live standings (capture 14); empty in the static World view.
            positions: Vec::new(),
            // Empty at game start (no matches played); runtime tracks per-season
            // attendance/receipts extremes (capture 16).
            attendances: Vec::new(),
            // Empty at game start (no transfers made); runtime records per-season
            // record transfers (capture 17).
            transfer_records: Vec::new(),
            // Empty at game start (no matches played); runtime records per-season
            // player leaders (capture 20).
            player_records: Vec::new(),
            // Empty at game start (no matches played); runtime records per-season
            // streaks (capture 34).
            sequences: Vec::new(),
            // Empty at game start (no matches played); runtime records per-season
            // record matches (capture 35).
            results: Vec::new(),
        }
    }
}
