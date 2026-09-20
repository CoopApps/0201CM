//! Per-person achievements — the accrued list behind the Player Profile →
//! History → **Achievements** view (Date · Club · Achievement text).
//!
//! Empty at game start (the exe ships no pre-game player honours); it fills
//! during simulation from two authentic, already-ported sources fired at
//! season end (`hook_year_rollover`):
//!  * club championships (`honours::Honour`, credited to the winning club's
//!    current squad) — kind `Competition`;
//!  * individual season awards (`awards_engine::AwardedHonour`, which carries
//!    `winner_staff_id`) — kind `Award`.
//!
//! Decode of the on-screen panel: `reports/player_history_view_panels_decode.md`.

use serde::{Deserialize, Serialize};

/// Which Filter bucket an achievement falls in (the exe's Filter dropdown:
/// All Records / Competitions / Awards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementKind {
    /// A club competition honour (championship / cup).
    Competition,
    /// An individual award (Player of the Season, Team of the Season, …).
    Award,
    /// Anything else (e.g. a transfer event) — shown only under All Records.
    Other,
}

/// One accrued achievement for one person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerAchievement {
    pub person_id: u32,
    /// Elapsed sim day when recorded (for stable newest-first ordering).
    pub day: u32,
    /// Display date "D.M.YY" (the sim date the honour was recorded).
    pub date: String,
    /// Club the person was at when it happened.
    pub club_id: u32,
    pub club_name: String,
    /// The achievement line, e.g. "English Premier Division Champions".
    pub text: String,
    pub kind: AchievementKind,
}

/// A season award logged by the (World-free) tick, awaiting the World-aware
/// accrual pass. Eq-safe (no `f32` — unlike `AwardedHonour`) so the save can
/// still derive `Eq`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggedAward {
    pub day: u32,
    /// "D.M.YY" — the sim date when awarded.
    pub date: String,
    pub year: u16,
    pub winner_staff_id: u32,
    pub competition_name: String,
    pub category: crate::awards_engine::AwardCategory,
}

/// Append-only per-person achievement store, held on the save.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerAchievementBook {
    pub entries: Vec<PlayerAchievement>,
}

impl PlayerAchievementBook {
    pub fn new() -> Self { Self { entries: Vec::new() } }

    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        person_id: u32,
        day: u32,
        date: String,
        club_id: u32,
        club_name: String,
        text: String,
        kind: AchievementKind,
    ) {
        self.entries.push(PlayerAchievement {
            person_id, day, date, club_id, club_name, text, kind,
        });
    }

    /// This person's achievements, newest first, optionally filtered by the
    /// Filter dropdown (0 = All, 1 = Competitions, 2 = Awards).
    pub fn for_person(&self, person_id: u32, filter: usize) -> Vec<&PlayerAchievement> {
        let mut out: Vec<&PlayerAchievement> = self.entries.iter()
            .filter(|a| a.person_id == person_id)
            .filter(|a| match filter {
                1 => a.kind == AchievementKind::Competition,
                2 => a.kind == AchievementKind::Award,
                _ => true,
            })
            .collect();
        out.sort_by(|a, b| b.day.cmp(&a.day));
        out
    }
}

/// The on-screen label for an award category (the History Achievements
/// line). `season` is the "YYYY/YY" span string.
pub fn award_line(category: crate::awards_engine::AwardCategory, comp: &str, season: &str) -> String {
    use crate::awards_engine::AwardCategory::*;
    match category {
        PlayerOfTheSeason => format!("{comp} Player of the Season"),
        TopScorer => format!("{comp} Top Scorer"),
        YoungPlayerOfTheSeason => format!("{comp} Young Player of the Season"),
        TeamOfTheSeason => format!("Named in {season} {comp} Select"),
        PlayerOfTheMonth => format!("{comp} Player of the Month"),
        ManagerOfTheMonth => format!("{comp} Manager of the Month"),
        NationFootballerOfTheYear => format!("{comp} Footballer of the Year"),
        NationPlayerOfTheYear => format!("{comp} Player of the Year"),
    }
}
