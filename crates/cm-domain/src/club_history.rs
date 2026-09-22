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

impl crate::World {
    /// Build the club History view-model for `club_id`.
    ///
    /// DONE (DB-driven, verifiable): honours (from `references.club_comp_history`),
    /// the View-dropdown competitions, the record category lists, and the two
    /// shipped club records "Most League Apps/Goals for Club" (decoded: the club
    /// key is staff_history +0x0a, holder = max summed apps/goals per person;
    /// verified Chester -> 361/124 Stuart Rimmer).
    /// FLAGGED (value left "-"): the game-accrued records (Top Goalscorer,
    /// streaks, transfer-fee records, etc.) — empty at a fresh save, they accrue
    /// live from match/season events as the game is played.
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
        }
    }
}
