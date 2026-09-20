//! Player Profile screen view model (the default page that loads on
//! clicking a squad player). Decode:
//! `reports/player_profile_screen_decode.md`.
//!
//! Attribute values are read from the type10 record at the
//! game-authoritative byte offsets (see
//! `memory/type10-real-attribute-offsets.md`) — the REAL DB values.
//! The original fogs attributes for players the manager doesn't know
//! (the AttributeReveal estimate); that is a separate feature and is
//! NOT applied here.

use serde::{Deserialize, Serialize};

use crate::{RuntimeSaveGame, World};
use crate::general_info::fmt_value_full;

/// One career-stats row (competition label + the 9 stat cells).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CareerRow {
    pub label: String,
    /// Apps, Con, Asts, MoM, Pass, Tck, Drb, Sh Tar, Av R.
    pub cells: [String; 9],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerProfileView {
    pub staff_id: u32,
    /// "1. Glyn Garner (Bury)".
    pub title: String,
    /// "Born 9.12.76 (Age 24). Welsh."
    pub born_line: String,
    /// 31 attribute values in the fixed alphabetical grid order (col1 =
    /// 0..12, col2 = 12..24, col3 = 24..31).
    pub attributes: Vec<String>,
    /// Preferred Foot, Form, Morale, Condition (col-3 rows 8..11).
    pub status: [String; 4],
    /// Full position name, e.g. "Goalkeeper".
    pub position: String,
    /// Six competition rows (Non Competitive..Senior Club).
    pub career: Vec<CareerRow>,
    /// Injuries & Bans subtab: Injury, Type, Condition, Training, Bans.
    pub injuries: [String; 5],
    /// True for a goalkeeper — the career col-2 header is "Con" (goals
    /// conceded) for keepers and "Gls" (goals) for outfielders.
    pub is_goalkeeper: bool,
    /// The player's current club id — the title bar uses its kit colour.
    pub club_id: Option<u32>,
    /// Contract subtab: Type, Wages, Expires, Squad Status, Bonuses,
    /// Clauses, Notes.
    pub contract: [String; 7],
    /// Transfer subtab: Availability, Value, Fluent Languages, Offers,
    /// Interested, Future(line 1), Future(line 2).
    pub transfer: [String; 7],
}

/// Grid attribute labels in display order (col1 top→bottom, then col2,
/// then col3 first 7). These are the 31 visible attributes, alphabetical.
pub const PROFILE_ATTR_LABELS: [&str; 31] = [
    "Acceleration", "Aggression", "Agility", "Anticipation", "Balance",
    "Bravery", "Creativity", "Crossing", "Decisions", "Determination",
    "Dribbling", "Finishing", "Flair", "Handling", "Heading", "Influence",
    "Jumping", "Long Shots", "Marking", "Off The Ball", "Pace", "Passing",
    "Positioning", "Reflexes", "Set Pieces", "Stamina", "Strength",
    "Tackling", "Teamwork", "Technique", "Work Rate",
];

/// The col-3 status labels.
pub const PROFILE_STATUS_LABELS: [&str; 4] =
    ["Preferred Foot", "Form", "Morale", "Condition"];

/// Career-stats competition row labels, top→bottom.
pub const PROFILE_CAREER_LABELS: [&str; 6] = [
    "Non Competitive", "League", "Cup", "Continental",
    "International", "Senior Club",
];

/// Byte offset on the type10 record for each grid attribute, in the same
/// order as `PROFILE_ATTR_LABELS`. Determination (index 9) is NOT on
/// type10 — it lives on Person +0x58 — so it carries sentinel 0 here and
/// is read specially. Table verified against the game's own attribute
/// dispatch FUN_0052d090 (memory/type10-real-attribute-offsets.md).
const PROFILE_ATTR_OFFSETS: [usize; 31] = [
    0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x43, 0x23, 0x24, 0x00 /*Determination*/,
    0x26, 0x27, 0x28, 0x2a, 0x2b, 0x2f, 0x2e, 0x31, 0x32, 0x33,
    0x36, 0x37, 0x39, 0x3a, 0x29, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x44,
];

impl World {
    pub fn player_profile_view_for(
        &self,
        save: &RuntimeSaveGame,
        staff_id: u32,
    ) -> Option<PlayerProfileView> {
        use crate::typed_records::{ClubView, PlayerView, NationView};
        let person = self.staff.type6.iter().find(|p| p.id == staff_id)?;
        let pv = PlayerView::from_split(person.id, &person.body);

        // Name + squad number + club → title line. `squad_numbers` is
        // keyed by the type10 (player-data) id, NOT the person id, so
        // resolve the link first (the Squad screen keys it the same way).
        let name = self.person_display_name(person);
        let attr_link = pv.player_data_id().map(|l| l as u32).unwrap_or(staff_id);
        let number = self.squad_numbers.get(&attr_link).copied()
            .filter(|n| *n > 0);
        let club_name = pv.current_club_id()
            .and_then(|cid| self.core.clubs.iter()
                .find(|c| ClubView::new(c).id() == cid as u32))
            .map(|c| ClubView::new(c).primary_name())
            .unwrap_or_default();
        let title = match (number, club_name.is_empty()) {
            (Some(n), false) => format!("{n}. {name} ({club_name})"),
            (Some(n), true)  => format!("{n}. {name}"),
            (None, false)    => format!("{name} ({club_name})"),
            (None, true)     => name.clone(),
        };

        // Bio: Born D.M.YY (Age NN). Nationality.
        // ~21% of players ship with a sentinel DOB (year 1900) because the
        // exe generates it at game init. Until that init pass is wired, use
        // a deterministic age fallback (the SAME hash the Squad screen uses,
        // so the two screens agree) rather than showing "Born unknown".
        let nationality = pv.nation_id()
            .and_then(|nid| self.core.nations.iter()
                .map(NationView::new)
                .find(|nv| nv.id() as i32 == nid))
            .map(|nv| nv.nationality_name())
            .unwrap_or_default();
        let nat_str = if nationality.is_empty() {
            String::new()
        } else {
            format!(" {nationality}.")
        };
        let age: u8 = person.age_at(2001, crate::day_of_year(2001, 8, 10))
            .unwrap_or_else(|| {
                let mut h = (staff_id as u64).wrapping_mul(0xBF58476D_1CE4E5B9);
                h ^= h >> 27; h = h.wrapping_mul(0x94D049BB_133111EB); h ^= h >> 31;
                17 + (h % 19) as u8
            });
        let born_year = if person.dob_year() > 1900 {
            person.dob_year()
        } else {
            2001u16.saturating_sub(age as u16)
        };
        let dob = crate::typed_records::CmDate {
            day: person.dob_day().max(1), year: born_year, is_leap: 0,
        };
        let (month, day) = dob.to_month_day();
        let yy = born_year % 100;
        let born_line = format!("Born {day}.{month}.{yy:02} (Age {age}).{nat_str}");

        // Attributes — real DB values at game-authoritative offsets.
        let attrs = pv.player_data_id().map(|l| l as u32).unwrap_or(staff_id);
        let a10 = self.staff.type10.iter().find(|a| a.id == attrs);
        let fa = a10.map(|a| a.full_attributes());
        let attributes: Vec<String> = PROFILE_ATTR_LABELS.iter().enumerate()
            .map(|(i, label)| {
                if *label == "Determination" {
                    // Person +0x58, not type10.
                    return (pv.determination() as i32).to_string();
                }
                match &fa {
                    Some(fa) => {
                        let off = PROFILE_ATTR_OFFSETS[i];
                        (fa.get(off - 0x0f).copied().unwrap_or(0) as i32).to_string()
                    }
                    None => "-".to_string(),
                }
            })
            .collect();

        // Status fields.
        let preferred_foot = match &fa {
            Some(fa) => {
                let left = fa.get(0x30 - 0x0f).copied().unwrap_or(0) as i32;
                let right = fa.get(0x3b - 0x0f).copied().unwrap_or(0) as i32;
                preferred_foot_str(left, right)
            }
            None => "-".to_string(),
        };
        let status = [
            preferred_foot,
            "-".to_string(),   // Form — no recent games at season start.
            "Ok".to_string(),  // Morale — neutral default until wired.
            "-".to_string(),   // Condition — runtime fitness, not yet wired.
        ];

        // Position — full name from the dominant aptitude.
        let position = a10.map(position_full_name).unwrap_or_default();
        let is_goalkeeper = a10.map(|a| a.apt_goalkeeper >= 15).unwrap_or(false);

        // Career stats — empty at season start (all "-").
        let career: Vec<CareerRow> = PROFILE_CAREER_LABELS.iter()
            .map(|l| CareerRow {
                label: l.to_string(),
                cells: std::array::from_fn(|i| {
                    if i == 8 { "----".to_string() } else { "-".to_string() }
                }),
            })
            .collect();

        // Injuries & Bans subtab. Injury / Type / Bans come from the
        // runtime injury book; Training from its rehab-training gate;
        // Condition is the init match-fitness seed (156 = 100%) until the
        // match-fitness sim reduces it.
        let injured = save.injuries.rehab_progress(staff_id).is_some();
        let banned = !save.injuries.is_available(staff_id) && !injured;
        // Condition — the exe computes it as (sharpness × fitness)/10000
        // (FUN_00615c70), a match-fitness subsystem we have not ported;
        // that gives ~70-80% at a fresh season, never 100%. Until it is
        // ported, mirror the Squad screen's own condition value so the two
        // screens agree and stay in range (same hash of the player id).
        let mut h = staff_id.wrapping_mul(0x9E3779B9);
        h ^= h >> 13; h = h.wrapping_mul(0xC2B2AE35); h ^= h >> 16;
        let condition_pct = 70 + (h % 11);
        let injuries = [
            if injured { "Injured".to_string() } else { "None".to_string() },
            "-".to_string(),                       // Type — detail TBD.
            format!("{condition_pct}%"),
            if save.injuries.is_training_available(staff_id) {
                "Full".to_string()
            } else {
                "Restricted".to_string()
            },
            if banned { "Suspended".to_string() } else { "None".to_string() },
        ];

        // Contract subtab — Wages / Expires / Clauses from the contract
        // pool (real); Type / Squad Status / Bonuses / Notes are the
        // labelled defaults until their exact sources are wired.
        let ct = self.contracts.as_ref().and_then(|p| p.contract_for_staff(staff_id));
        let wages = match ct.map(|r| r.wage).unwrap_or_else(|| pv.wage()) {
            w if w > 0 => format!("\u{a3}{} per week", w),
            _ => "-".to_string(),
        };
        let expires = ct.map(|r| {
            let dob = crate::typed_records::CmDate {
                day: r.expiry_dayofyear, year: r.expiry_year, is_leap: 0,
            };
            let (m, d) = dob.to_month_day();
            format!("{d}.{m}.{:02}", r.expiry_year % 100)
        }).unwrap_or_else(|| "-".to_string());
        let clauses = match ct {
            Some(r) if r.non_promotion == 0 && r.minimum_fee == 0
                && r.non_playing == 0 && r.relegation == 0 && r.manager_job == 0 => "None",
            Some(_) => "Yes",
            None => "None",
        }.to_string();
        // Squad Status — the exe picks a descriptor from the player's
        // squad-status byte (0x00a6e370.. "…squad rotation system" /
        // "…important first team player" / "…indispensable to the club").
        // The byte source is not wired yet; default to the common
        // first-team descriptor.
        let squad_status = "This player is an important first team player".to_string();
        let contract = [
            "Full Time Contract".to_string(),  // Type — pending club-status source.
            wages,
            expires,
            squad_status,
            "None".to_string(),                // Bonuses — pending source.
            clauses,
            "-".to_string(),                   // Notes — pending source.
        ];

        // Transfer subtab. Value from the contract pool (real); Offers /
        // Interested are None at a fresh start; Availability / Fluent
        // Languages / Future use labelled defaults pending their real
        // sources (transfer-status byte, per-nation language, mood text).
        let value_i = ct.map(|r| r.value).unwrap_or_else(|| pv.value()) as i64;
        let language = if nationality.is_empty() { "-".to_string() } else { nationality.clone() };
        let transfer = [
            "Unknown".to_string(),                 // Availability
            if value_i > 0 { fmt_value_full(value_i) } else { "-".to_string() },
            language,                              // Fluent Languages (nation proxy)
            "None".to_string(),                    // Offers
            "None".to_string(),                    // Interested
            "Happy to stay at the club".to_string(), // Future (mood — default)
            String::new(),                         // Future line 2
        ];

        Some(PlayerProfileView {
            staff_id, title, born_line, attributes, status, position, career,
            injuries, is_goalkeeper,
            club_id: pv.current_club_id().map(|c| c as u32),
            contract, transfer,
        })
    }
}

/// A completed season accrued during play (Playing Career row). Populated
/// at year rollover from `player_ratings` (World-free — all fields come
/// from the rating book), merged with the shipped `staff_history` at
/// display time. Eq-safe for the save's `Eq` derive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccruedSeason {
    pub person_id: u32,
    /// Season start year (label = `{year}/{(year+1)%10}`).
    pub year: u16,
    /// Club id that season (`-1` if unknown).
    pub club_id: i32,
    pub apps: u32,
    pub goals: u32,
}

/// One season row in the History subtab's "Playing Career" table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistorySeasonRow {
    /// "1985/6" — `{year}/{(year+1)%10}`.
    pub season: String,
    /// Club that season — `ClubView::secondary_name()` (record +0x38),
    /// e.g. "Brighton". "Unknown Club" for club id -2; blank otherwise.
    pub club: String,
    /// True when this was a loan spell (`subs == 0xFF` in the record).
    pub is_loan: bool,
    /// Appearances ("22"), or "-" when apps == 0.
    pub apps: String,
    /// Goals ("0"), or "-" when apps == 0.
    pub goals: String,
}

/// Player Profile → History subtab view model. The top "Playing Career"
/// table (season-by-season, newest first) plus a synthesized current
/// season, a Total row, and the selected-season header. Decode:
/// `reports/player_history_screen_decode.md`. Verified against Martin
/// Keown (person 55141): 507 apps / 8 goals total, two Brighton loans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerHistoryView {
    pub title: String,
    pub club_id: Option<u32>,
    pub position: String,
    pub is_goalkeeper: bool,
    /// Season rows, newest first (includes the synthesized current
    /// season), NOT including the Total row.
    pub seasons: Vec<HistorySeasonRow>,
    pub total_apps: String,
    pub total_goals: String,
    /// Bottom-table header, e.g. "  2001/2 Arsenal" (the selected season
    /// — the live current season by default).
    pub selected_label: String,
}

/// The current season's start year (game begins 2001/02).
const HISTORY_CURRENT_YEAR: u16 = 2001;

/// Fixed competition-type rows of the bottom breakdown table.
pub const HISTORY_BREAKDOWN_LABELS: [&str; 6] = [
    "Non Competitive", "League", "Cup", "Continental",
    "International", "Senior Club",
];

impl World {
    pub fn player_history_view_for(
        &self,
        save: &RuntimeSaveGame,
        staff_id: u32,
    ) -> Option<PlayerHistoryView> {
        use crate::typed_records::{ClubView, PlayerView};
        let prof = self.player_profile_view_for(save, staff_id)?;
        let person = self.staff.type6.iter().find(|p| p.id == staff_id)?;
        let pv = PlayerView::from_split(person.id, &person.body);
        let cur_club_id: i64 = pv.current_club_id().map(|c| c as i64).unwrap_or(-1);

        // Club name for a season = club record +0x38 (secondary_name).
        // -2 → "Unknown Club"; any other negative → blank.
        let club_name = |cid: i64| -> String {
            if cid == -2 { return "Unknown Club".to_string(); }
            if cid < 0 { return String::new(); }
            self.core.clubs.iter()
                .find(|c| ClubView::new(c).id() == cid as u32)
                .map(|c| {
                    let cv = ClubView::new(c);
                    let s = cv.secondary_name();
                    if s.is_empty() { cv.primary_name() } else { s }
                })
                .unwrap_or_default()
        };
        let season_label = |year: u16| format!("{}/{}", year, (year + 1) % 10);

        // The current (live) season is the save's calendar year.
        let current_year = save.date.year;

        // Gather this person's seasons: shipped pre-game history
        // (`staff_history`) + seasons accrued during play (`player_seasons`)
        // + the live current season (apps/goals from `player_ratings`).
        struct Rec { year: u16, club: i64, loan: bool, apps: u32, goals: u32, current: bool }
        let mut recs: Vec<Rec> = self.references.staff_history.iter()
            .filter(|h| h.person_id == staff_id)
            .map(|h| Rec {
                year: h.year,
                club: h.competition_id as i32 as i64,
                loan: h.subs == 0xFF,
                apps: h.apps as u32,
                goals: h.goals as u32,
                current: false,
            })
            .collect();
        for s in &save.player_seasons {
            if s.person_id == staff_id && s.year != current_year {
                recs.push(Rec {
                    year: s.year, club: s.club_id as i64, loan: false,
                    apps: s.apps, goals: s.goals, current: false,
                });
            }
        }
        let (cur_apps, cur_goals) = save.player_ratings.season_apps_goals(staff_id);
        recs.push(Rec {
            year: current_year, club: cur_club_id, loan: false,
            apps: cur_apps, goals: cur_goals, current: true,
        });

        // Order: year DESC, then apps ASC within a year.
        recs.sort_by(|a, b| b.year.cmp(&a.year).then(a.apps.cmp(&b.apps)));

        let total_apps: u32 = recs.iter().map(|r| r.apps).sum();
        let total_goals: u32 = recs.iter().map(|r| r.goals).sum();

        let seasons = recs.iter().map(|r| {
            // The current season shows live counters ("0" when none yet);
            // past seasons show "-" for a 0-apps registration.
            let (apps, goals) = if r.current {
                (r.apps.to_string(), r.goals.to_string())
            } else if r.apps == 0 {
                ("-".to_string(), "-".to_string())
            } else {
                (r.apps.to_string(), r.goals.to_string())
            };
            HistorySeasonRow {
                season: season_label(r.year),
                club: club_name(r.club),
                is_loan: r.loan,
                apps, goals,
            }
        }).collect();

        let selected_label = format!("  {} {}",
            season_label(current_year), club_name(cur_club_id));

        Some(PlayerHistoryView {
            title: prof.title,
            club_id: prof.club_id,
            position: prof.position,
            is_goalkeeper: prof.is_goalkeeper,
            seasons,
            total_apps: total_apps.to_string(),
            total_goals: total_goals.to_string(),
            selected_label,
        })
    }
}

impl World {
    /// The current club's first-team squad for the name-bar player-picker
    /// (triangle button): `(person_id, "Surname, Initial")`, alphabetical
    /// by surname — matching the exe's pop-up. Players only (records with a
    /// paired type10 attribute record), filtered to the given club.
    pub fn club_squad_picker(&self, club_id: u32) -> Vec<(u32, String)> {
        use crate::typed_records::PlayerView;
        let mut rows: Vec<(String, String, u32)> = Vec::new();
        for person in &self.staff.type6 {
            let pv = PlayerView::from_split(person.id, &person.body);
            if pv.player_data_id().is_none() { continue; }
            if pv.current_club_id() != Some(club_id as i32) { continue; }
            let first = self.references.first_names
                .get(person.first_name_id() as usize)
                .map(|n| n.text.clone()).unwrap_or_default();
            let second = self.references.second_names
                .get(person.second_name_id() as usize)
                .map(|n| n.text.clone()).unwrap_or_default();
            let initial = first.trim().chars().next()
                .map(|c| c.to_string()).unwrap_or_default();
            let surname = second.trim().to_string();
            if surname.is_empty() { continue; }
            let label = if initial.is_empty() {
                surname.clone()
            } else {
                format!("{surname}, {initial}")
            };
            rows.push((surname.to_lowercase(), label, person.id));
        }
        rows.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        rows.into_iter().map(|(_, label, id)| (id, label)).collect()
    }

    /// History → Injuries rows for a person, newest first:
    /// `(date, injury_name, "Match"/"Training", period_out)`.
    pub fn player_injury_rows(&self, save: &RuntimeSaveGame, staff_id: u32)
        -> Vec<(String, String, String, String)>
    {
        let mut rows: Vec<(u32, String, String, String, String)> = save.injuries.history.iter()
            .filter(|h| h.player_id == staff_id && !h.name.is_empty())
            .map(|h| (
                h.game_day,
                h.date.clone(),
                h.name.clone(),
                if h.from_match { "Match".to_string() } else { "Training".to_string() },
                crate::injury_table::period_out(h.days_out),
            ))
            .collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0));
        rows.into_iter().map(|(_, d, n, t, p)| (d, n, t, p)).collect()
    }

    /// History → Bans rows for a person, newest first:
    /// `(date, "N match <scope> ban", reason)`.
    pub fn player_ban_rows(&self, save: &RuntimeSaveGame, staff_id: u32)
        -> Vec<(String, String, String)>
    {
        let mut rows: Vec<(u32, String, String, String)> = save.injuries.ban_history.iter()
            .filter(|b| b.player_id == staff_id)
            .map(|b| {
                let scope = self.competition_scope_adjective(b.competition_id);
                let ban = format!("{} match {} ban", b.matches, scope);
                (b.game_day, b.date.clone(), ban, b.reason.clone())
            })
            .collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0));
        rows.into_iter().map(|(_, d, b, r)| (d, b, r)).collect()
    }

    /// The ban-scope adjective for a competition: "European" for a
    /// continental competition (no owning nation), else the competition's
    /// nation adjective (e.g. "English" for the English leagues/cups).
    /// A player in England thus gets "English" domestically, "European"
    /// in continental ties — per the exe's scope word (FUN_00546a40).
    fn competition_scope_adjective(&self, comp_id: u32) -> String {
        use crate::typed_records::NationView;
        // Continental (European) competitions handled by dedicated engines.
        const CONTINENTAL: [u32; 6] = [326, 327, 328, 329, 330, 96];
        if CONTINENTAL.contains(&comp_id) {
            return "European".to_string();
        }
        if let Some(c) = self.references.club_competitions.iter().find(|c| c.id == comp_id) {
            if c.nation_id < 0 {
                return "European".to_string();
            }
            if let Some(adj) = self.core.nations.iter()
                .map(NationView::new)
                .find(|nv| nv.id() as i32 == c.nation_id)
                .map(|nv| nv.nationality_name())
                .filter(|s| !s.is_empty())
            {
                return adj;
            }
        }
        // Unknown domestic competition (e.g. an English-traditional runtime
        // id not in the club-competition table): domestic English.
        "English".to_string()
    }

    /// Exact achievement text for a season award, using the decoded
    /// per-nation award-pool table (`FUN_00562580` for England) → the
    /// `staff_competitions` name. English is the demonstrated/simulated
    /// nation and is byte-exact; other nations fall back to the generic
    /// `award_line` until their subsystem tables are ported. Decode:
    /// `reports/awards_engine_decode.md`.
    fn resolve_award_text(
        &self,
        category: crate::awards_engine::AwardCategory,
        comp_name: &str,
        season: &str,
    ) -> String {
        use crate::awards_engine::AwardCategory::*;
        let award_name = |id: u32| -> Option<String> {
            self.references.staff_competitions.iter()
                .find(|c| c.id == id)
                .map(|c| c.long_name.clone())
                .filter(|s| !s.is_empty())
        };
        if comp_name.starts_with("English") {
            match category {
                // Overall PFA awards (award ids 5 / 6 — comp = all-English).
                PlayerOfTheSeason => if let Some(n) = award_name(5) { return n; },
                YoungPlayerOfTheSeason => if let Some(n) = award_name(6) { return n; },
                // Division "Select" (Team of the Season): 7/8/9/10/169.
                TeamOfTheSeason => {
                    let id = if comp_name.contains("Premier") { 7 }
                        else if comp_name.contains("First") { 8 }
                        else if comp_name.contains("Second") { 9 }
                        else if comp_name.contains("Third") { 10 }
                        else if comp_name.contains("Conference") { 169 }
                        else { 7 };
                    if let Some(n) = award_name(id) {
                        return format!("Named in {season} {n}");
                    }
                }
                _ => {}
            }
        }
        crate::player_achievements::award_line(category, comp_name, season)
    }

    /// A person's current club as `(club_id, name)`, or `(0, "")` for a
    /// free agent.
    fn person_current_club(&self, person_id: u32) -> (u32, String) {
        use crate::typed_records::{ClubView, PlayerView};
        let person = match self.staff.type6.iter().find(|p| p.id == person_id) {
            Some(p) => p, None => return (0, String::new()),
        };
        match PlayerView::from_split(person.id, &person.body).current_club_id() {
            Some(c) => {
                let name = self.core.clubs.iter()
                    .find(|cl| ClubView::new(cl).id() == c as u32)
                    .map(|cl| ClubView::new(cl).primary_name())
                    .unwrap_or_default();
                (c as u32, name)
            }
            None => (0, String::new()),
        }
    }

    /// Convert the season awards logged by the (World-free) tick and any
    /// newly-recorded club championships into per-person achievements. The
    /// app calls this after each tick; it resolves clubs/squads (which the
    /// tick cannot) and is idempotent (awards drained, honours cursored).
    pub fn accrue_player_achievements(&self, save: &mut RuntimeSaveGame) {
        use crate::player_achievements::AchievementKind;
        // 1. Individual season awards (drain the log).
        let awards = std::mem::take(&mut save.season_award_log);
        for a in awards {
            let season_span = format!("{}/{:02}", a.year, (a.year + 1) % 100);
            let (club_id, club_name) = self.person_current_club(a.winner_staff_id);
            let text = self.resolve_award_text(a.category, &a.competition_name, &season_span);
            save.player_achievements.record(
                a.winner_staff_id, a.day, a.date, club_id, club_name,
                text, AchievementKind::Award,
            );
        }
        // 2. Club championships recorded since the last pass → credit every
        //    player at the winning club with "<competition> Champions".
        let start = save.honours_credited.min(save.honours.len());
        let new_honours: Vec<(u32, String, String)> = save.honours[start..].iter()
            .map(|h| (h.club_id, h.competition.clone(), h.club_name.clone()))
            .collect();
        save.honours_credited = save.honours.len();
        let day = save.elapsed_days;
        let date = format!("{}.{}.{:02}", save.date.day, save.date.month, save.date.year % 100);
        for (club_id, comp, hclub) in new_honours {
            use crate::typed_records::ClubView;
            let club_name = if hclub.is_empty() {
                self.core.clubs.iter().find(|c| ClubView::new(c).id() == club_id)
                    .map(|c| ClubView::new(c).primary_name()).unwrap_or_default()
            } else { hclub };
            for (pid, _) in self.club_squad_picker(club_id) {
                save.player_achievements.record(
                    pid, day, date.clone(), club_id, club_name.clone(),
                    // Exe honour code 5 ("<competition> champions", lower-case;
                    // cups use code 0 "winners" — only league titles are
                    // recorded here). Decode: reports/achievement_text_decode.md.
                    format!("{comp} champions"), AchievementKind::Competition,
                );
            }
        }
        // 3. Completed transfers (accepted bids) → "Bought by <club> for
        //    <fee>" (exe code 102). Cursor over the append-only resolved
        //    bids so each accepted move is credited once.
        let tstart = save.transfers_credited.min(save.transfers.resolved_bids.len());
        let new_transfers: Vec<(u32, u32, i64)> = save.transfers.resolved_bids[tstart..].iter()
            .filter(|(_, o)| *o == crate::transfer::BidOutcome::Accepted)
            .map(|(b, _)| (b.target_player_id, b.bidding_club_id, b.amount))
            .collect();
        save.transfers_credited = save.transfers.resolved_bids.len();
        for (pid, buyer, fee) in new_transfers {
            use crate::typed_records::ClubView;
            let club_name = self.core.clubs.iter()
                .find(|c| ClubView::new(c).id() == buyer)
                .map(|c| ClubView::new(c).primary_name())
                .unwrap_or_default();
            let text = format!("Bought by {} for {}", club_name,
                crate::general_info::fmt_value_full(fee));
            save.player_achievements.record(
                pid, day, date.clone(), buyer, club_name, text, AchievementKind::Other,
            );
        }
    }
}

/// Preferred-foot label from the two foot ratings (1..20). "Right/Left
/// Only" when one foot is strong and the other weak, "Either" when both
/// are strong, else the dominant foot. Reproduces the Garner capture
/// (right 20 / left 1 → "Right Only").
fn preferred_foot_str(left: i32, right: i32) -> String {
    match (right >= 10, left >= 10) {
        (true, true) => "Either".to_string(),
        (true, false) => "Right Only".to_string(),
        (false, true) => "Left Only".to_string(),
        (false, false) => if right >= left { "Right".to_string() } else { "Left".to_string() },
    }
}

/// The full position string the exe paints on the profile, e.g.
/// "Goalkeeper", "Striker (Centre)", "Defender/Striker (Centre)",
/// "Attacking Midfielder (Left, Right)". Ported from the exe's
/// multi-position formatter `FUN_005289a0`: every position whose aptitude
/// is >= 15 is listed (in GK,SW,D,DM,M,AM,Striker order, joined by "/"),
/// then the qualifying sides (Left, Centre, Right — the exe's left→right
/// order, per FUN_005a1890) in parentheses. The attacker slot prints
/// "Striker" only for a pure central attacker and "Forward" otherwise
/// (the FUN_005289a0 stat test below). Names come from the exe string
/// table (0x009a3b98.. and 0x009a3ae8..). Empty when no aptitude reaches
/// 15 (a regen stub whose position the exe generates at init).
fn position_full_name(a: &crate::DomainStaffType10) -> String {
    // Use the exe-exact eligibility bitmask (sliding threshold 15→10),
    // NOT a fixed >=15 test — this returns 0 (no position) for a player
    // with no aptitudes, so regen stubs stay blank instead of defaulting
    // to a phantom "Sweeper". Bit map (position_eligibility_bits):
    //   0x001 GK  0x004 SW  0x002 D  0x008 DM  0x010 M  0x020 AM  0x040 F
    //   0x080 Left  0x200 Centre  0x800 Right
    const T: i8 = 15;
    let bits = a.position_eligibility_bits();
    if bits == 0 {
        return String::new();
    }
    let mut positions: Vec<&str> = Vec::new();
    let mut has_sweeper = false;
    if bits & 0x001 != 0 { positions.push("Goalkeeper"); }
    if bits & 0x004 != 0 { positions.push("Sweeper"); has_sweeper = true; }
    if bits & 0x002 != 0 { positions.push("Defender"); }
    if bits & 0x008 != 0 { positions.push("Defensive Midfielder"); }
    if bits & 0x010 != 0 { positions.push("Midfielder"); }
    if bits & 0x020 != 0 { positions.push("Attacking Midfielder"); }
    if bits & 0x040 != 0 {
        // FUN_005289a0 lines 851-855: a "Striker" is a PURE central
        // attacker — attacker>=15 with attacking-mid, both flanks and
        // free-role all below 15; any of those makes it a "Forward".
        let striker = a.apt_att_midfielder < T
            && a.apt_right_side < T
            && a.apt_left_side < T
            && a.apt_free_role < T;
        positions.push(if striker { "Striker" } else { "Forward" });
    }
    if positions.is_empty() {
        return String::new();
    }
    // A sweeper always plays Centre — force it regardless of side bits.
    if has_sweeper && positions == ["Sweeper"] {
        return "Sweeper (Centre)".to_string();
    }
    let mut sides: Vec<&str> = Vec::new();
    if bits & 0x080 != 0 { sides.push("Left"); }
    if bits & 0x200 != 0 || has_sweeper { sides.push("Centre"); }
    if bits & 0x800 != 0 { sides.push("Right"); }
    sides.dedup();
    let pos = positions.join("/");
    if sides.is_empty() {
        pos
    } else {
        format!("{pos} ({})", sides.join(", "))
    }
}
