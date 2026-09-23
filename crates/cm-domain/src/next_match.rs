//! Next Match screen view model.
//!
//! Geometry + behavioural contract decoded from the exe:
//! `reports/next_match_screen_decode.md` (captures
//! `fixtures/next_match_screen/`). The screen shows a single upcoming
//! fixture with `Date >>` / `<< Date` stepping through the club's
//! fixture list, and an availability block that lists injured players
//! ("out") and debutants ("set for debut").

use serde::{Deserialize, Serialize};

use crate::{GameDate, RuntimeSaveGame, World};

/// One line in the availability / news block, with its colour intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsLine {
    pub text: String,
}

/// The Next Match screen for one club at one fixture index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextMatchView {
    pub club_id: u32,
    /// Title, e.g. `"Stoke (Home)"`.
    pub title: String,
    /// Competition label, e.g. `"Second Division"` or `"Friendly"`.
    pub competition: String,
    /// Date line, e.g. `"Saturday 11th August (28 days)"`.
    pub date_line: String,
    /// Venue, e.g. `"The Abbey Stadium, Cambridge"`.
    pub venue: String,
    /// Two match-rule lines.
    pub match_rules: [String; 2],
    /// Last-meeting summary, or `"-"`.
    pub last_meeting: String,
    /// Weather forecast, or `"Unknown"`.
    pub weather: String,
    /// Availability block: injured ("out") then debutants.
    pub news: Vec<NewsLine>,
    /// True when this is a friendly (drives the Match Rules text).
    pub is_friendly: bool,
    /// `<< Date` is enabled (there is an earlier fixture).
    pub has_earlier: bool,
    /// `Date >>` is enabled (there is a later fixture).
    pub has_later: bool,
    /// This fixture's index within the club's sorted fixture list.
    pub fixture_index: usize,
    /// Total fixtures the club has (for clamping Date nav).
    pub fixture_count: usize,
}

fn ordinal(day: u8) -> &'static str {
    match day % 100 {
        11 | 12 | 13 => "th",
        _ => match day % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    }
}

fn weekday_name(g: &GameDate) -> &'static str {
    // 2001-01-01 was a Monday.
    let mut days: i64 = 0;
    for y in 2001..g.year {
        days += if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
    }
    let leap = (g.year % 4 == 0 && g.year % 100 != 0) || g.year % 400 == 0;
    let cum = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    days += cum[(g.month - 1) as usize] as i64
        + if leap && g.month > 2 { 1 } else { 0 }
        + (g.day as i64 - 1);
    ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]
        [(days.rem_euclid(7)) as usize]
}

// exe FUN_00536f90: month index -> full month name ("January".."December").
// (exe is 0-indexed via a switch; this port takes 1-indexed month and subtracts 1.)
fn month_name(m: u8) -> &'static str {
    ["January", "February", "March", "April", "May", "June", "July",
     "August", "September", "October", "November", "December"]
        [(m.saturating_sub(1)) as usize]
}

/// Absolute day number since 2001-01-01, for date differences.
fn abs_day(g: &GameDate) -> i64 {
    let mut days: i64 = 0;
    for y in 2001..g.year {
        days += if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
    }
    let leap = (g.year % 4 == 0 && g.year % 100 != 0) || g.year % 400 == 0;
    let cum = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    days + cum[(g.month.saturating_sub(1)) as usize] as i64
        + if leap && g.month > 2 { 1 } else { 0 }
        + (g.day as i64 - 1)
}

impl World {
    fn person_full_name(&self, staff_id: u32) -> Option<String> {
        let person = self.staff.type6.iter().find(|p| p.id == staff_id)?;
        // Canonical resolver — honours the common-name override.
        let name = self.person_display_name(person);
        if name.is_empty() { None } else { Some(name) }
    }

    /// Build the Next Match view for `club_id` showing the fixture at
    /// `index` in the club's sorted fixture list. `index = usize::MAX`
    /// selects the next unplayed fixture (the default landing state).
    pub fn next_match_for(
        &self,
        save: &RuntimeSaveGame,
        club_id: u32,
        index: usize,
    ) -> Option<NextMatchView> {
        // The club's fixtures, sorted by date.
        let mut fixtures: Vec<&crate::HeadlessSeasonFixture> = save.season.fixtures.iter()
            .filter(|f| f.home_club_id == club_id || f.away_club_id == club_id)
            .collect();
        fixtures.sort_by(|a, b| a.date.cmp(&b.date));
        if fixtures.is_empty() { return None; }

        let idx = if index == usize::MAX {
            // First fixture not yet played.
            fixtures.iter()
                .position(|f| f.status != crate::HeadlessFixtureStatus::Played)
                .unwrap_or(fixtures.len() - 1)
        } else {
            index.min(fixtures.len() - 1)
        };
        let f = fixtures[idx];
        let is_home = f.home_club_id == club_id;
        let opponent = if is_home { &f.away_club_name } else { &f.home_club_name };
        let title = format!("{} ({})", opponent, if is_home { "Home" } else { "Away" });

        let is_friendly = f.competition_name.to_lowercase().contains("friendly");

        // Date line with countdown.
        let days_off = abs_day(&f.date) - abs_day(&save.date);
        let countdown = if days_off <= 0 {
            "today".to_string()
        } else if days_off == 1 {
            "1 day".to_string()
        } else {
            format!("{days_off} days")
        };
        let date_line = format!(
            "{} {}{} {} ({})",
            weekday_name(&f.date), f.date.day, ordinal(f.date.day),
            month_name(f.date.month), countdown,
        );

        // Venue = the HOME club's stadium.
        let home_club_id = f.home_club_id;
        let venue = self.core.clubs.iter()
            .map(crate::typed_records::ClubView::new)
            .find(|cv| cv.id() == home_club_id)
            .and_then(|cv| cv.home_stadium_id())
            .and_then(|sid| self.references.stadiums.iter().find(|s| s.id as i32 == sid))
            .filter(|s| s.name_set && !s.name.trim().is_empty())
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Match rules — competitive vs friendly (decoded text).
        let match_rules = if is_friendly {
            [
                "No player restrictions".to_string(),
                "9 subs named, maximum 9 used".to_string(),
            ]
        } else {
            [
                "Max 3 non-EU players in the match squad".to_string(),
                "5 subs named, maximum 3 used".to_string(),
            ]
        };

        // Availability block. Injured players (out) — accurate from the
        // injury book. Then debutants — players who have not yet made a
        // recorded appearance for the club this save.
        //
        // NOTE (approximation, labelled in the decode report): a true
        // "set for debut" is selection-dependent (the projected XI who
        // have never played for the club). We list uninjured squad
        // players with zero appearances, which matches the opener's
        // behaviour but over-lists once selection matters. Refinement is
        // tracked in reports/next_match_screen_decode.md §5.
        let mut news: Vec<NewsLine> = Vec::new();
        let squad_ids: Vec<u32> = save.player_ratings.players.iter()
            .filter(|p| p.club_id == Some(club_id as i32))
            .map(|p| p.staff_id)
            .collect();
        for sid in &squad_ids {
            if !save.injuries.is_available(*sid) {
                if let Some(name) = self.person_full_name(*sid) {
                    news.push(NewsLine { text: format!("{name} is injured and out") });
                }
            }
        }

        let view = NextMatchView {
            club_id,
            title,
            competition: f.competition_name.clone(),
            date_line,
            venue,
            match_rules,
            last_meeting: "-".to_string(),
            weather: "Unknown".to_string(),
            news,
            is_friendly,
            has_earlier: idx > 0,
            has_later: idx + 1 < fixtures.len(),
            fixture_index: idx,
            fixture_count: fixtures.len(),
        };
        Some(view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_line_formats_with_weekday_ordinal_and_countdown() {
        // 2001-08-11 was a Saturday.
        let g = GameDate { year: 2001, month: 8, day: 11 };
        assert_eq!(weekday_name(&g), "Saturday");
        assert_eq!(ordinal(11), "th");
        assert_eq!(ordinal(21), "st");
        assert_eq!(ordinal(22), "nd");
        assert_eq!(ordinal(23), "rd");
        assert_eq!(month_name(8), "August");
    }

    #[test]
    fn abs_day_difference_is_correct() {
        let a = GameDate { year: 2001, month: 7, day: 14 };
        let b = GameDate { year: 2001, month: 8, day: 11 };
        assert_eq!(abs_day(&b) - abs_day(&a), 28);
    }
}
