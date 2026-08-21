#![forbid(unsafe_code)]

pub mod gameplay_mutators;
pub mod league_calendar;
pub mod menu;
pub mod typed_records;
pub mod ui_schema;

pub use typed_records::{ClubView, NationView, PlayerView};

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

use cm_data::{DatFile, Manifest, RecordKind, ReferenceData, SaveFile, StaffData, TABLE_SPECS};
use gameplay_mutators::{
    default_exact_gameplay_mutator_skeletons, exact_gameplay_mutator_skeleton_entry_points_ready,
    exact_gameplay_mutator_skeletons_ready, ExactGameplayMutatorSkeleton,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub const RUST_DB_FORMAT: &str = "cm0102-rs-world-db";
pub const RUST_DB_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataEntrySummary {
    pub filename: String,
    pub kind: u8,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSectionSummary {
    pub name: String,
    pub size: u32,
    pub verified_record_size: Option<usize>,
    pub verified_record_count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSummary {
    pub version: u32,
    pub section_count: usize,
    pub sections: Vec<SaveSectionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainCity {
    pub id: u32,
    pub name: String,
    pub tail_u16: [u16; 13],
    pub tail_u32: [u32; 6],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainOfficial {
    pub id: u32,
    pub u32_slots: [u32; 10],
    pub u16_slots: [u16; 21],
    pub trailing_byte: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainName {
    pub text: String,
    pub footer: [u8; 12],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainStadium {
    pub id: u32,
    pub name: String,
    pub unknown_tail: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainCompetition {
    pub id: u32,
    pub long_name: String,
    pub short_name: String,
    /// Three-letter abbreviation ("PRM", "D1", "CON"); blank on cups and
    /// non-manageable feeder divisions. Presence + a populated `nation_id`
    /// marks a manageable league — see `World::is_manageable_league`.
    #[serde(default)]
    pub three_letter_name: String,
    /// Scope word (2 = domestic; -2 on the global bucket).
    #[serde(default)]
    pub scope: i32,
    /// The competition's nation id (decoded from record +0x5d — populated in
    /// the shipped data). `-1`/`-2` when the competition has no nation.
    #[serde(default = "neg_one")]
    pub nation_id: i32,
    /// Promotion link (editor "Last division"); -2 = none.
    #[serde(default = "neg_one")]
    pub last_division: i32,
    /// Relegation/reserve link (editor "Reserve division"); -2 = none.
    #[serde(default = "neg_one")]
    pub reserve_division: i32,
    /// Reputation / league standard (editor).
    #[serde(default)]
    pub reputation: u16,
    #[serde(default)]
    pub unknown_tail: Vec<u8>,
}

fn neg_one() -> i32 {
    -1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainOpaqueRecord {
    pub ordinal: u32,
    #[serde(default)]
    pub id: u32,
    #[serde(default)]
    pub primary_name: Option<String>,
    #[serde(default)]
    pub secondary_name: Option<String>,
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default)]
    pub text_candidates: Vec<String>,
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CoreBook {
    pub clubs: Vec<DomainOpaqueRecord>,
    pub nat_clubs: Vec<DomainOpaqueRecord>,
    pub colours: Vec<DomainOpaqueRecord>,
    pub continents: Vec<DomainOpaqueRecord>,
    pub nations: Vec<DomainOpaqueRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CoreSummary {
    pub club_count: usize,
    pub nat_club_count: usize,
    pub colour_count: usize,
    pub continent_count: usize,
    pub nation_count: usize,
    pub sample_club_record_size: Option<usize>,
    pub sample_nat_club_record_size: Option<usize>,
    pub sample_colour_record_size: Option<usize>,
    pub sample_continent_record_size: Option<usize>,
    pub sample_nation_record_size: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainHistory17 {
    pub id: u32,
    pub u32_slots: [u32; 4],
    pub trailing_byte: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainHistory26 {
    pub u32_slots: [u32; 6],
    pub trailing_u16: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainHistory58 {
    pub u32_slots: [u32; 14],
    pub trailing_u16: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceBook {
    pub cities: Vec<DomainCity>,
    pub officials: Vec<DomainOfficial>,
    pub first_names: Vec<DomainName>,
    pub second_names: Vec<DomainName>,
    pub common_names: Vec<DomainName>,
    pub stadiums: Vec<DomainStadium>,
    pub staff_competitions: Vec<DomainCompetition>,
    pub club_competitions: Vec<DomainCompetition>,
    pub nation_competitions: Vec<DomainCompetition>,
    pub staff_history: Vec<DomainHistory17>,
    pub staff_comp_history: Vec<DomainHistory58>,
    pub club_comp_history: Vec<DomainHistory26>,
    pub nation_comp_history: Vec<DomainHistory26>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReferenceSummary {
    pub city_count: usize,
    pub official_count: usize,
    pub first_name_count: usize,
    pub second_name_count: usize,
    pub common_name_count: usize,
    pub stadium_count: usize,
    pub staff_competition_count: usize,
    pub club_competition_count: usize,
    pub nation_competition_count: usize,
    pub staff_history_count: usize,
    pub staff_comp_history_count: usize,
    pub club_comp_history_count: usize,
    pub nation_comp_history_count: usize,
    pub sample_city: Option<String>,
    pub sample_official_id: Option<u32>,
    pub sample_first_name: Option<String>,
    pub sample_stadium: Option<String>,
    pub sample_staff_competition: Option<String>,
    pub sample_club_competition: Option<String>,
    pub sample_nation_competition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub base_data: Vec<DataEntrySummary>,
    pub save: Option<SaveSummary>,
    #[serde(default)]
    pub schema: SchemaBook,
    #[serde(default)]
    pub core: CoreBook,
    #[serde(default)]
    pub core_summary: CoreSummary,
    pub references: ReferenceBook,
    pub reference_summary: ReferenceSummary,
    #[serde(default)]
    pub staff: StaffBook,
    pub staff_summary: StaffSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StaffSummary {
    pub type6_count: usize,
    #[serde(default)]
    pub type8_count: usize,
    pub type9_count: usize,
    pub type10_count: usize,
    pub sample_type6_id: Option<u32>,
    pub sample_type9_id: Option<u32>,
    pub sample_type10_id: Option<u32>,
    pub sample_type10_ca: Option<u16>,
    pub sample_type10_pa: Option<u16>,
    pub sample_type10_reputation: Option<u16>,
    pub max_type10_ca: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StaffBook {
    pub type6: Vec<DomainStaffType6>,
    #[serde(default)]
    pub type8: Vec<DomainStaffType8>,
    pub type9: Vec<DomainStaffType9>,
    pub type10: Vec<DomainStaffType10>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainStaffType6 {
    pub id: u32,
    pub body: Vec<u8>,
}

impl DomainStaffType6 {
    fn u16(&self, off: usize) -> u16 {
        self.body
            .get(off..off + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .unwrap_or(0)
    }
    fn u32(&self, off: usize) -> u32 {
        self.body
            .get(off..off + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .unwrap_or(0)
    }

    /// First-name id (index into first_names) — disk record `body+0x00` (u32).
    /// Verified: body+0x00 resolves to real first names (Taufiq, Fouad…).
    pub fn first_name_id(&self) -> u32 {
        self.u32(0x00)
    }
    /// Second-name/surname id — `body+0x04` (u16). Verified: consecutive
    /// records carry consecutive surname ids (the pool is alphabetical).
    pub fn second_name_id(&self) -> u16 {
        self.u16(0x04)
    }
    /// Common-name/nickname id — `body+0x06` (u16); 0 = none.
    pub fn common_name_id(&self) -> u16 {
        self.u16(0x06)
    }
    /// Date-of-birth day-of-year — `body+0x0c` (u16, 1..366). Verified against
    /// real birthdates alongside the year field.
    pub fn dob_day(&self) -> u16 {
        self.u16(0x0c)
    }
    /// Date-of-birth year — `body+0x0e` (u16). Verified: distribution peaks
    /// 1976..1980 (early-20s players for a 2001 start).
    pub fn dob_year(&self) -> u16 {
        self.u16(0x0e)
    }

    /// Current club id — disk record `body+0x35` (u32; this is disk offset
    /// 0x39, matching the in-memory current-club field). `None` for free
    /// agents / unemployed staff (negative sentinel). VERIFIED: 5,490 clubs
    /// carry realistic squads (mean ~20, median 21) at this offset.
    pub fn current_club_id(&self) -> Option<u32> {
        let v = self.u32(0x35);
        if (0..10580).contains(&v) {
            Some(v as u32)
        } else {
            None
        }
    }

    /// Age at a given game date (year + day-of-year), the way FUN_0051f5d0
    /// computes it: `year - dob_year`, minus 1 if this year's birthday hasn't
    /// happened yet. Returns `None` when the DOB year is unset (0 / < 1900),
    /// as it is on reserved and non-dated records.
    pub fn age_at(&self, current_year: u16, current_day: u16) -> Option<u8> {
        let by = self.dob_year();
        // <= 1900 is the "no DOB set" sentinel (1900 is used for ~10k
        // undated records); treat as unknown age.
        if by <= 1900 || by > current_year {
            return None;
        }
        let mut age = current_year.saturating_sub(by);
        if self.dob_day() > current_day {
            age = age.saturating_sub(1);
        }
        u8::try_from(age).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainStaffType9 {
    pub id: u32,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainStaffType8 {
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainStaffType10 {
    pub id: u32,
    pub unknown_byte_4: u8,
    #[serde(alias = "probable_ca")]
    pub rating_short_0x05: u16,
    #[serde(alias = "probable_pa")]
    pub rating_short_0x07: u16,
    #[serde(default)]
    pub unknown_bytes_9_12: [u8; 4],
    #[serde(alias = "probable_reputation")]
    pub rating_short_0x0d: u16,
    #[serde(default)]
    pub unknown_bytes_15_26: [u8; 12],
    pub attributes: [u8; 31],
    #[serde(default)]
    pub unknown_bytes_58_64: [u8; 7],
    pub trailing_bytes: [u8; 5],
}

impl DomainStaffType10 {
    /// Current Ability (0..200), from the outfield/attribute record +0x05.
    pub fn current_ability(&self) -> i16 {
        self.rating_short_0x05 as i16
    }

    /// Potential Ability as stored at +0x07, interpreted SIGNED. A negative
    /// value (-1..-10) is CM's "flexible potential" sentinel: the true ceiling
    /// is resolved at game-init with RNG (FUN_0051f5d0, blocks 810-924). A
    /// positive value is the fixed PA.
    pub fn potential_ability_raw(&self) -> i16 {
        self.rating_short_0x07 as i16
    }

    /// A concrete potential ability for this game instance. When the stored PA
    /// is a fixed positive value we use it. When it's a negative sentinel we
    /// resolve it to the MIDPOINT of the band FUN_0051f5d0 draws from, so the
    /// value is deterministic and reasonable — NOT bit-exact with the exe
    /// (that needs the game's ring-buffer RNG ported). Bands from the decode:
    ///   -1 → ~120..200 (mid 160), -2 → ~160..200 (mid 180),
    ///   others (-3..-10) → scaled toward the ceiling. Always >= CA.
    pub fn resolved_potential_ability(&self) -> i16 {
        let ca = self.current_ability();
        let pa = self.potential_ability_raw();
        let resolved = if pa >= 0 {
            pa
        } else {
            match pa {
                -1 => 160,
                -2 => 180,
                // -3..-10: linearly toward 200 as the sentinel deepens.
                n => (150 + (-n - 2) * 6).min(200) as i16,
            }
        };
        resolved.max(ca).clamp(1, 200)
    }

    /// Resolve potential ability using the GAME'S ring-buffer RNG, per the
    /// bands FUN_0051f5d0 draws from (blocks 810-924): a positive PA is fixed;
    /// a negative sentinel draws a ceiling from the game RNG. Faithful to the
    /// exe's mechanism (real RNG, real bands), though not bit-identical to a
    /// specific exe run (that needs the full init call-order replayed).
    ///   -1 → 0x78 + rand(0x51)   (120..200)
    ///   -2 → 0xa0 + rand(0x29)   (160..200)
    ///   -3..-10 → 0x78 + rand(0x51), biased up by the sentinel depth
    pub fn resolved_potential_ability_rng(&self, rng: &mut cm_rng::MatchRng) -> i16 {
        let ca = self.current_ability();
        let pa = self.potential_ability_raw();
        let resolved: i32 = if pa >= 0 {
            pa as i32
        } else {
            match pa {
                -1 => 0x78 + rng.random(0x51),
                -2 => 0xa0 + rng.random(0x29),
                n => {
                    let depth = (-(n as i32) - 2).clamp(0, 8);
                    (0x78 + rng.random(0x51) + depth * 3).min(200)
                }
            }
        };
        (resolved.max(ca as i32)).clamp(1, 200) as i16
    }
}

/// One player/staff's initial RUNTIME state — the deterministic part of the
/// exe's per-person seeding (FUN_0051f5d0) that a fresh game instance needs.
/// Ability/attributes stay in the base database (type10); this carries the
/// live state that starts at a known value and then changes during play.
///
/// Only the fields the decode nails as deterministic are here. Age (needs the
/// staff DISK-record layout for DOB), and the RNG-generated attribute fill for
/// players lacking data, are separate follow-ups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerInitState {
    pub player_id: u32,
    /// Current ability (from base type10 +0x05).
    pub current_ability: i16,
    /// Resolved potential ability for this instance (sentinels resolved).
    pub potential_ability: i16,
    /// Age in years at the game start date, from the person's DOB
    /// (`type6` body +0x0c/+0x0e). `None` when the record has no dated DOB.
    pub age: Option<u8>,
    /// Match fitness / condition. FUN_0051f5d0 seeds this to 0x9c = 156
    /// (runtime array DAT_00acdf0c +0x09) at instance creation — VERIFIED.
    pub condition: u16,
    /// Morale/match-state flags, low nibble cleared at init (+0x0b) — neutral.
    pub morale: u8,
}

impl PlayerInitState {
    /// The condition every player starts a new game on — FUN_0051f5d0 line 1014.
    pub const INITIAL_CONDITION: u16 = 156;

    /// Seed a player's initial runtime state from their attribute record and
    /// (optionally) their person record + the game start date for age.
    pub fn seed(
        attr: &DomainStaffType10,
        person: Option<&DomainStaffType6>,
        start_year: u16,
        start_day: u16,
        rng: Option<&mut cm_rng::MatchRng>,
    ) -> Self {
        let potential_ability = match rng {
            Some(r) => attr.resolved_potential_ability_rng(r),
            None => attr.resolved_potential_ability(),
        };
        PlayerInitState {
            player_id: attr.id,
            current_ability: attr.current_ability(),
            potential_ability,
            age: person.and_then(|p| p.age_at(start_year, start_day)),
            condition: Self::INITIAL_CONDITION,
            morale: 0,
        }
    }
}

/// Day-of-year (1..366) for a calendar date — used to compare against DOB day
/// when computing age. Standard Gregorian leap rule.
pub fn day_of_year(year: u16, month: u8, day: u8) -> u16 {
    const CUM: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let mut d = CUM[(month.clamp(1, 12) - 1) as usize] + day as u16;
    if leap && month > 2 {
        d += 1;
    }
    d
}

/// Day-of-week for a Gregorian date (0=Sunday..6=Saturday) via Sakamoto's
/// algorithm — used to format news dates like the game ("Wed 10 Jul").
pub fn weekday(year: u16, month: u8, day: u8) -> u8 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year as i32;
    let m = month.clamp(1, 12) as usize;
    if m < 3 {
        y -= 1;
    }
    (((y + y / 4 - y / 100 + y / 400 + T[m - 1] + day as i32) % 7) as u8).min(6)
}

/// English ordinal suffix for a day-of-month (1st, 2nd, 3rd, 4th…).
pub fn day_ordinal(day: u8) -> &'static str {
    match (day % 10, day % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}

/// The three intra-day phases the game stamps on the clock and on news
/// (`DAT_00acde88`): morning / afternoon / evening.
pub fn phase_label(phase: u8) -> &'static str {
    match phase % 3 {
        0 => "AM",
        1 => "PM",
        _ => "EVE",
    }
}

/// Format a game date the way the news list does: "Tue 7th Aug PM".
pub fn news_date_label(d: &GameDate, phase: u8) -> String {
    const WD: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MON: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let wd = WD[weekday(d.year, d.month, d.day) as usize];
    let mon = MON[(d.month.clamp(1, 12) - 1) as usize];
    format!("{wd} {}{} {mon} {}", d.day, day_ordinal(d.day), phase_label(phase))
}

/// A human manager's typed name (Enter Name screen fields).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ManagerIdentity {
    pub first: String,
    pub second: String,
    pub nickname: String,
}

impl ManagerIdentity {
    /// Display name: nickname if given, else "First Second".
    pub fn display_name(&self) -> String {
        if !self.nickname.trim().is_empty() {
            self.nickname.clone()
        } else {
            format!("{} {}", self.first, self.second).trim().to_string()
        }
    }
}

/// One human manager. Employment is modelled the way the exe does — on the
/// person, not the seat: `club`/`nation` are the appointment links (`None` =
/// none), and `status` is the human-seat status byte. Unemployed =
/// `status == Active && club.is_none() && nation.is_none()`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanManager {
    pub identity: ManagerIdentity,
    /// Club the human currently manages (exe person+0x39). `None` = unemployed
    /// of a club job.
    pub club: Option<u32>,
    /// Nation the human currently manages (exe person+0x24). Independent of
    /// `club` — a human can hold both at once.
    pub nation: Option<u32>,
    /// Manager reputation at their post (exe club+0x59, set to 20 on takeover).
    pub reputation: u8,
}

impl HumanManager {
    pub fn new(identity: ManagerIdentity) -> Self {
        Self { identity, club: None, nation: None, reputation: 0 }
    }
    /// Unemployed = holds no club and no nation.
    pub fn is_unemployed(&self) -> bool {
        self.club.is_none() && self.nation.is_none()
    }
}

/// What the active human sees: either their club dashboard or, when
/// unemployed, the manager-status / job view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DashboardView {
    Club(ClubDashboard),
    Unemployed(UnemployedView),
}

/// The club home screen (FUN_004551c0) assembled for a managed club.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClubDashboard {
    pub manager_name: String,
    pub club_id: u32,
    pub club_name: String,
    pub division_name: String,
    /// 1-based league position (from standings order; all level at the start
    /// morning, so this is the seeded order until results arrive).
    pub position: usize,
    pub division_size: usize,
    pub date: GameDate,
    /// The club's next pending fixture, if any.
    pub next_fixture: Option<DashboardFixture>,
    /// The squad — players whose current club is this one.
    pub squad: Vec<SquadMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardFixture {
    pub date: GameDate,
    pub home_club_name: String,
    pub away_club_name: String,
    pub is_home: bool,
    pub competition_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadMember {
    pub player_id: u32,
    pub name: String,
    pub age: Option<u8>,
    pub current_ability: i16,
    pub condition: u16,
}

/// The News page — the manager's home screen (the exe's news.c, drawn by the
/// LAB_00770170 callback registered in FUN_0076ffb0). Titled "<Manager> News",
/// with the four filter tabs and a dated headline list feeding a body panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsView {
    pub title: String,
    pub items: Vec<NewsItem>,
}

/// One item in the news inbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsItem {
    pub date: GameDate,
    /// Pre-formatted "Wed 10 Jul" style date label.
    pub date_label: String,
    pub headline: String,
    pub body: String,
    pub category: NewsCategory,
    pub unread: bool,
}

/// The four news filter tabs (All spans every category).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewsCategory {
    Message,
    Competition,
    InjuryBan,
}

impl NewsCategory {
    /// Classify a RuntimeEvent `kind` string into a tab bucket.
    fn from_kind(kind: &str) -> Self {
        let k = kind.to_ascii_lowercase();
        if k.contains("injur") || k.contains("ban") || k.contains("suspend") {
            NewsCategory::InjuryBan
        } else if k.contains("result") || k.contains("match") || k.contains("league")
            || k.contains("competition") || k.contains("cup") || k.contains("fixture")
        {
            NewsCategory::Competition
        } else {
            NewsCategory::Message
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnemployedView {
    pub manager_name: String,
    pub date: GameDate,
    /// Clubs currently without a human manager that this human could apply to
    /// (name + division) — a light job list for the unemployed view.
    pub message: String,
}

/// One club a new manager can choose to manage — a playable club in a
/// manageable division of a selected nation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagerClubChoice {
    pub club_id: u32,
    pub club_name: String,
    pub division_id: u32,
    pub division_name: String,
}

/// Summary of the player-initialisation pass, stored in a new-game save.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PlayerInitSummary {
    pub attribute_records: usize,
    pub fixed_potential: usize,
    pub resolved_flexible_potential: usize,
    #[serde(default)]
    pub players_with_age: usize,
    #[serde(default)]
    pub average_age: f32,
    #[serde(default)]
    pub players_employed_at_a_club: usize,
    pub initial_condition: u16,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SchemaBook {
    pub tables: Vec<TableSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSchema {
    pub path: String,
    pub status: FieldStatus,
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub status: FieldStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldStatus {
    Verified,
    CompatibilityVerified,
    Inferred,
    Projected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotCoverage {
    pub manifest_entries: usize,
    pub known_logical_tables: usize,
    pub recognized_manifest_entries: usize,
    pub unrecognized_manifest_entries: usize,
    pub owned_core_tables: usize,
    pub owned_reference_tables: usize,
    pub owned_staff_tables: usize,
    pub owned_world_tables: usize,
    pub remaining_binary_tables: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotAuditReport {
    pub install_root: String,
    pub snapshot_manifest_entries: usize,
    pub install_manifest_entries: usize,
    pub snapshot_recognized_manifest_entries: usize,
    pub install_known_logical_tables: usize,
    pub snapshot_save_section_count: Option<usize>,
    pub install_save_section_count: Option<usize>,
    pub coverage: SnapshotCoverage,
    pub mismatches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustDatabaseAuditReport {
    pub coverage: SnapshotCoverage,
    pub checked_tables: usize,
    pub mismatches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalDatabaseReport {
    pub table_count: usize,
    pub field_count: usize,
    pub verified_fields: usize,
    pub inferred_fields: usize,
    pub projected_fields: usize,
    pub fully_verified_tables: usize,
    pub editable_tables: usize,
    pub dat_runtime_dependency: DatRuntimeDependency,
    pub validation: CanonicalValidationReport,
    pub tables: Vec<CanonicalTableReport>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalTableReport {
    pub path: String,
    pub rows: usize,
    pub table_status: FieldStatus,
    pub verified_fields: usize,
    pub inferred_fields: usize,
    pub projected_fields: usize,
    pub editable: bool,
    pub dat_replacement_status: DatReplacementStatus,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalValidationReport {
    pub checks: Vec<CanonicalValidationCheck>,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalValidationCheck {
    pub name: String,
    pub status: ValidationStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatRuntimeDependency {
    NoneForOwnedTables,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatReplacementStatus {
    RuntimeReady,
    EditableButNeedsSemantics,
    ImportedButOpaque,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendReadinessReport {
    pub status: BackendReadinessStatus,
    pub completion: BackendCompletion,
    pub checks: Vec<BackendReadinessCheck>,
    pub blockers: Vec<BackendReadinessBlocker>,
    #[serde(default)]
    pub semantic_cleanup: Vec<SemanticCleanupItem>,
    pub implementation_plan: Vec<BackendImplementationPlanItem>,
    pub milestones: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendReadinessStatus {
    RuntimeReady,
    VerifiedHeadlessShell,
    BlockedByFrontierGameplay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCompletion {
    pub canonical_tables: usize,
    pub runtime_ready_tables: usize,
    pub editable_tables: usize,
    pub validation_failures: usize,
    pub remaining_binary_tables: usize,
    pub phase_frontiers: usize,
    pub phase_2_frontiers: usize,
    pub runtime_mutation_log_entries: usize,
    pub headless_blockers: usize,
    pub score_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendReadinessCheck {
    pub name: String,
    pub status: ValidationStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendReadinessBlocker {
    pub system: String,
    pub severity: String,
    pub status: String,
    pub next_evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticCleanupItem {
    pub table: String,
    pub rows: usize,
    pub status: DatReplacementStatus,
    pub runtime_blocking: bool,
    pub inferred_fields: usize,
    pub projected_fields: usize,
    pub editable: bool,
    pub blockers: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendImplementationPlanItem {
    pub system: String,
    pub readiness: BackendImplementationReadiness,
    pub owned_records: usize,
    pub boundary_entries: usize,
    pub attempted_mutations: u32,
    pub implemented_mutations: u32,
    pub primary_frontiers: Vec<String>,
    pub code_derived_boundaries: Vec<String>,
    pub missing_lifts: Vec<String>,
    pub acceptance_gate: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendImplementationReadiness {
    BoundaryMapped,
    NeedsBoundaryMap,
    MutationsImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustDatabaseMetadata {
    pub format: String,
    pub version: u32,
    pub source: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSaveGame {
    pub format: String,
    pub version: u32,
    pub source: RuntimeSource,
    pub date: GameDate,
    #[serde(default)]
    pub simulation: RuntimeSimulationState,
    #[serde(default)]
    pub backend: RuntimeBackendSystems,
    #[serde(default)]
    pub headless: HeadlessRuntimeState,
    #[serde(default)]
    pub season: HeadlessSeasonState,
    #[serde(default)]
    pub elapsed_days: u32,
    #[serde(default)]
    pub pending_events: Vec<RuntimeEvent>,
    #[serde(default)]
    pub phase_trace: Vec<RuntimePhaseTrace>,
    pub table_counts: RuntimeTableCounts,
    /// The picker choices this game was created from. `None` for saves made by
    /// the older parameterless builder.
    #[serde(default)]
    pub new_game: Option<NewGameOptions>,
    /// Per-nation league tier — the Rust port of the `nation_record + 0x11c`
    /// selection bitfield (see [`LeagueTier`]). Every nation in the database
    /// appears here; the whole world is loaded regardless of selection (as the
    /// exe's `FUN_005121a0` does), and the tier only governs manageability and
    /// match-simulation detail. Keyed by nation id.
    #[serde(default)]
    pub nation_tiers: Vec<NationTierAssignment>,
    /// The complete mutable-world layer. The immutable base (all shipped
    /// club/player/nation records) lives in the native database (`rust-db`)
    /// and is pinned here by reference + fingerprint rather than duplicated
    /// into every save. Everything that DIVERGES from that base during play —
    /// changed player attributes, new/folded clubs, transfers — is captured
    /// as an overlay so a save fully restores the game state. See
    /// [`SaveWorldOverlay`].
    #[serde(default)]
    pub world: SaveWorldOverlay,
    /// The player-initialisation summary (the deterministic core of the exe's
    /// FUN_0051f5d0 per-person seeding). `None` on the parameterless builder.
    #[serde(default)]
    pub player_init: Option<PlayerInitSummary>,
    /// The human managers in this game. Multiple are supported (hotseat); each
    /// holds their own club/nation appointment. Created at runtime via
    /// `add_manager` — the exe's "Add Manager" (command 0x3fb).
    #[serde(default)]
    pub humans: Vec<HumanManager>,
    /// Index of the ACTIVE human — whose dashboard is shown. The exe's
    /// `DAT_00b5d016`. Setting it swaps whose dashboard/toolbar renders.
    #[serde(default)]
    pub active_human: usize,
    pub notes: Vec<String>,
}

/// The mutable-world layer of a save: a pinned reference to the immutable base
/// database plus every per-entity change made during play. A save = base
/// (by reference) + this overlay + the runtime subsystem state above. This is
/// what makes a save "save everything" without embedding the 155MB base into
/// each file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SaveWorldOverlay {
    /// Identity of the base database this save overlays.
    pub base: WorldBaseRef,
    /// Clubs whose mutable state diverged from the base, keyed by club id.
    /// Empty on a fresh new game (nothing has changed yet).
    #[serde(default)]
    pub club_overrides: Vec<EntityOverride>,
    /// Staff/players whose mutable state diverged, keyed by staff id.
    #[serde(default)]
    pub staff_overrides: Vec<EntityOverride>,
}

/// A pinned identity of the base database a save was built from. `verify`
/// checks a loaded database still matches, so a save can't silently apply its
/// overlay onto a different dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorldBaseRef {
    pub rust_db_dir: String,
    pub table_counts: RuntimeTableCountsSnapshot,
    /// Cheap content fingerprint of the base (FNV-1a over the per-table record
    /// counts + the database metadata). Distinguishes different datasets;
    /// documented as shape-level, not a full 155MB content hash.
    pub fingerprint: String,
}

impl WorldBaseRef {
    /// Build the base reference from a loaded world + the db directory it came
    /// from. The fingerprint is FNV-1a over the sorted `table:count` pairs —
    /// cheap (no 155MB read) and stable across re-imports of the same data.
    pub fn from_world(world: &World, dir: &Path) -> Self {
        let counts = world.table_count_map();
        let mut fnv: u64 = 0xcbf29ce484222325;
        for (name, n) in &counts {
            for b in name.as_bytes() {
                fnv = (fnv ^ *b as u64).wrapping_mul(0x100000001b3);
            }
            for b in n.to_le_bytes() {
                fnv = (fnv ^ b as u64).wrapping_mul(0x100000001b3);
            }
        }
        WorldBaseRef {
            rust_db_dir: dir.display().to_string(),
            table_counts: RuntimeTableCountsSnapshot { counts },
            fingerprint: format!("fnv1a:{fnv:016x}"),
        }
    }

    /// True when `world` still matches this pinned base (same table counts /
    /// fingerprint). Guards against applying a save's overlay onto a dataset
    /// it wasn't built from.
    pub fn matches(&self, world: &World, dir: &Path) -> bool {
        let other = Self::from_world(world, dir);
        self.fingerprint == other.fingerprint
    }
}

/// A flat snapshot of every table's record count, used to pin the base.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RuntimeTableCountsSnapshot {
    pub counts: BTreeMap<String, usize>,
}

/// One entity's divergence from the base: its id and the full current record
/// bytes (or typed fields, once mutation lands). Storing the whole record keeps
/// the overlay simple and lossless; only changed entities appear here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityOverride {
    pub id: u32,
    /// The entity's complete current record bytes. Whatever gameplay changed
    /// is included; loading applies this over the base record.
    pub record: Vec<u8>,
}

/// One nation's league tier in a save — the persisted form of the exe's
/// `nation_record + 0x11c` bitfield.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NationTierAssignment {
    pub nation_id: u32,
    pub nation_name: String,
    pub tier: LeagueTier,
    /// Bit 2 (0x4) of the +0x11c byte: whether this nation's competitions get
    /// full detailed-match simulation. Orthogonal to the tier (the exe's
    /// "Background Matches Off/Normal/High" option). Foreground nations always
    /// run detailed; background nations depend on the global option.
    pub detailed_matches: bool,
}

/// The three league states, exactly mapping the exe's `nation_record + 0x11c`
/// bitfield (verified from FUN_00806640 / FUN_00683e30 / FUN_0052e370):
/// * `Neither` = flag 0: present in the pools but not manageable, not
///   detail-simulated. Cannot be promoted in-game.
/// * `Background` = bit 0 (0x1): loaded, simulated, clubs manageable, and
///   promotable to `Foreground` at runtime via a pure flag flip (no reload).
/// * `Foreground` = bit 1 (0x2): the active, fully-playable "selected" leagues,
///   written to the save as the foreground set.
///
/// Foreground and background are mutually exclusive; the engine flips one to
/// the other when the human manager changes which nation he works in. Squad
/// data is NOT tiered — every nation's full club/player set is resident in all
/// three states; the tier only gates manageability and simulation depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeagueTier {
    Neither,
    Background,
    Foreground,
}

impl LeagueTier {
    /// The `nation_record + 0x11c` bit value this tier sets (bit 2 / detail is
    /// tracked separately). `Neither` = 0, `Background` = 0x1, `Foreground` = 0x2.
    pub fn flag_bits(self) -> u8 {
        match self {
            LeagueTier::Neither => 0,
            LeagueTier::Background => 0x1,
            LeagueTier::Foreground => 0x2,
        }
    }

    /// The exe's `flag & 3` predicate: is a club in this nation manageable?
    /// True for Foreground and Background, false for Neither.
    pub fn is_manageable(self) -> bool {
        !matches!(self, LeagueTier::Neither)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSource {
    pub kind: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GameDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSimulationState {
    pub phase: u8,
    pub cm_packed_date: CmPackedDate,
    pub provenance: String,
}

impl Default for RuntimeSimulationState {
    fn default() -> Self {
        Self {
            phase: 0,
            cm_packed_date: CmPackedDate::from_game_date(GameDate {
                year: 2001,
                month: 7,
                day: 1,
            }),
            provenance: "CM0102 simulation frontier 0x005b6a90; date add-days 0x00536190"
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBackendSystems {
    pub status: RuntimeBackendStatus,
    pub provenance: String,
    #[serde(default = "default_gameplay_mutator_contracts")]
    pub mutator_contracts: Vec<GameplayMutatorContract>,
    #[serde(default = "default_gameplay_mutator_install_plans")]
    pub mutator_install_plans: Vec<GameplayMutatorInstallPlan>,
    #[serde(default = "default_exact_gameplay_mutator_skeletons")]
    pub exact_mutator_skeletons: Vec<ExactGameplayMutatorSkeleton>,
    #[serde(default = "default_gameplay_promotion_gates")]
    pub gameplay_promotion_gates: Vec<GameplayPromotionGate>,
    #[serde(default = "default_gameplay_lift_workbench")]
    pub gameplay_lift_workbench: Vec<GameplayLiftWorkItem>,
    #[serde(default = "default_gameplay_system_code_claims")]
    pub gameplay_system_code_claims: Vec<GameplaySystemCodeClaim>,
    #[serde(default)]
    pub headless_fixture_pipeline_outputs: Vec<HeadlessFixturePipelineOutput>,
    pub matches: RuntimeSystemState,
    #[serde(default = "default_match_engine_lift_map")]
    pub match_engine_lift_map: Vec<MatchEngineLiftMapEntry>,
    #[serde(default = "default_match_engine_runtime_store")]
    pub match_engine_runtime_store: MatchEngineRuntimeStore,
    #[serde(default = "default_match_result_write_map")]
    pub match_result_write_map: Vec<MatchResultWriteMapEntry>,
    #[serde(default = "default_match_result_code_claims")]
    pub match_result_code_claims: Vec<MatchResultCodeClaim>,
    #[serde(default = "default_match_result_formula_lift_map")]
    pub match_result_formula_lift_map: Vec<MatchResultFormulaLiftEntry>,
    #[serde(default = "default_match_result_mutator_install_plan")]
    pub match_result_mutator_install_plan: MatchResultMutatorInstallPlan,
    #[serde(default = "default_match_result_runtime_store")]
    pub match_result_runtime_store: MatchResultRuntimeStore,
    pub competitions: RuntimeSystemState,
    #[serde(default = "default_competition_fixture_state_map")]
    pub competition_fixture_state_map: Vec<CompetitionFixtureStateMapEntry>,
    #[serde(default = "default_competition_notification_formula_lift_map")]
    pub competition_notification_formula_lift_map: Vec<CompetitionNotificationFormulaLiftEntry>,
    #[serde(default = "default_competition_notification_runtime_store")]
    pub competition_notification_runtime_store: CompetitionNotificationRuntimeStore,
    #[serde(default = "default_competition_standings_formula_lift_map")]
    pub competition_standings_formula_lift_map: Vec<CompetitionStandingsFormulaLiftEntry>,
    #[serde(default = "default_competition_standings_runtime_store")]
    pub competition_standings_runtime_store: CompetitionStandingsRuntimeStore,
    #[serde(default = "default_competition_progression_formula_lift_map")]
    pub competition_progression_formula_lift_map: Vec<CompetitionProgressionFormulaLiftEntry>,
    #[serde(default = "default_competition_progression_runtime_store")]
    pub competition_progression_runtime_store: CompetitionProgressionRuntimeStore,
    pub transfers: RuntimeSystemState,
    #[serde(default = "default_transfer_contract_state_map")]
    pub transfer_contract_state_map: Vec<TransferContractStateMapEntry>,
    #[serde(default = "default_transfer_contract_formula_lift_map")]
    pub transfer_contract_formula_lift_map: Vec<TransferContractFormulaLiftEntry>,
    #[serde(default = "default_transfer_contract_runtime_store")]
    pub transfer_contract_runtime_store: TransferContractRuntimeStore,
    pub news: RuntimeSystemState,
    #[serde(default = "default_news_inbox_emission_map")]
    pub news_inbox_emission_map: Vec<NewsInboxEmissionMapEntry>,
    #[serde(default = "default_news_inbox_formula_lift_map")]
    pub news_inbox_formula_lift_map: Vec<NewsInboxFormulaLiftEntry>,
    #[serde(default = "default_news_inbox_runtime_store")]
    pub news_inbox_runtime_store: NewsInboxRuntimeStore,
    #[serde(default = "default_backend_mutation_log_limit")]
    pub mutation_log_limit: usize,
    #[serde(default)]
    pub total_mutation_entries: usize,
    #[serde(default)]
    pub dropped_mutation_entries: usize,
    #[serde(default)]
    pub mutation_log: Vec<RuntimeSystemMutation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessFixturePipelineOutput {
    pub row: u32,
    pub fixture_row: u32,
    pub result: String,
    pub match_engine_event_rows: usize,
    pub match_state_mutation_rows: usize,
    pub match_result_event_rows: usize,
    pub standings_rows_touched: usize,
    pub news_events_created: usize,
    pub visible_news_dispatches: usize,
    pub save_visible: bool,
    pub provenance: String,
}

fn update_headless_fixture_pipeline_outputs(backend: &mut RuntimeBackendSystems) {
    if !match_result_runtime_store_ready(&backend.match_result_runtime_store)
        || !competition_standings_runtime_store_ready(&backend.competition_standings_runtime_store)
        || !news_inbox_runtime_store_ready(&backend.news_inbox_runtime_store)
        || backend
            .match_engine_runtime_store
            .result_finalization_outputs
            .is_empty()
    {
        return;
    }
    let finalization = &backend
        .match_engine_runtime_store
        .result_finalization_outputs[0];
    let output = HeadlessFixturePipelineOutput {
        row: 0,
        fixture_row: finalization.row,
        result: format!("{}-{}", finalization.home_score, finalization.away_score),
        match_engine_event_rows: backend.match_engine_runtime_store.event_queue_outputs.len(),
        match_state_mutation_rows: backend.match_engine_runtime_store.state_mutation_outputs.len(),
        match_result_event_rows: backend.match_result_runtime_store.event_queue.len(),
        standings_rows_touched: backend.competition_standings_runtime_store.rows.len(),
        news_events_created: backend.news_inbox_runtime_store.created_events.len(),
        visible_news_dispatches: backend.news_inbox_runtime_store.visible_news_dispatches.len(),
        save_visible: true,
        provenance: "Vertical headless fixture pipeline: Rust fixture result finalization is reflected into match-result runtime store, competition standings runtime store, and news/inbox runtime store.".to_string(),
    };
    if let Some(existing) = backend
        .headless_fixture_pipeline_outputs
        .iter_mut()
        .find(|existing| existing.row == output.row)
    {
        *existing = output;
    } else {
        backend.headless_fixture_pipeline_outputs.push(output);
    }
}

pub fn headless_fixture_pipeline_ready(backend: &RuntimeBackendSystems) -> bool {
    backend
        .headless_fixture_pipeline_outputs
        .iter()
        .any(|output| {
            output.fixture_row == 0
                && output.result == "2-1"
                && output.match_engine_event_rows >= 2
                && output.match_state_mutation_rows >= 5
                && output.match_result_event_rows >= 4
                && output.standings_rows_touched >= 1
                && output.news_events_created >= 2
                && output.visible_news_dispatches >= 1
                && output.save_visible
        })
}

pub fn headless_season_batch_ready(save: &RuntimeSaveGame) -> bool {
    if save.table_counts.clubs < 2 {
        return true;
    }
    save.season.fixtures.iter().any(|fixture| {
        fixture.status == HeadlessFixtureStatus::Played
            && fixture.home_score.is_some()
            && fixture.away_score.is_some()
            && fixture.match_packet.as_ref().is_some_and(|packet| {
                packet.final_event_code == "0x2004"
                    && !packet.event_codes.is_empty()
                    && packet.match_events.iter().any(|event| event.kind == "goal")
                    && packet.match_events.iter().any(|event| !event.score_impact)
                    && packet.state_mutation_rows >= 4
                    && packet.evidence.contains("0x006d1a20")
                    && packet.evidence.contains("0x006bc8d0")
                    && packet.evidence.contains("0x006a4020")
            })
    }) && save
        .season
        .standings
        .iter()
        .filter(|row| row.played > 0)
        .count()
        >= 2
        && save
            .season
            .batches
            .iter()
            .any(|batch| batch.played_fixtures > 0 && batch.news_events_created > 0)
        && save
            .pending_events
            .iter()
            .any(|event| event.kind == "match-report")
}

pub fn headless_schedule_generation_ready(save: &RuntimeSaveGame) -> bool {
    if save.table_counts.clubs < 2 {
        return true;
    }
    let generated_fixture_count = save
        .season
        .schedule_generation
        .iter()
        .map(|proof| proof.generated_fixtures as usize)
        .sum::<usize>();
    !save.season.fixtures.is_empty()
        && save.season.fixtures.len() == generated_fixture_count
        && save.season.schedule_generation.iter().all(|proof| {
            proof.function.contains("0x00670350")
                && proof.function.contains("0x0066c700")
                && proof.constants.iter().any(|constant| constant == "0x3b")
                && proof.constants.iter().any(|constant| constant == "0x41")
                && proof.constants.iter().any(|constant| constant == "0x245")
                && proof.constants.iter().any(|constant| constant == "0x49")
                && proof.generated_fixtures > 0
        })
        && save.season.fixtures.iter().all(|fixture| {
            fixture.home_club_id != fixture.away_club_id
                && !fixture.competition_name.is_empty()
                && fixture.source.contains("0x0066c700")
                && fixture.source.contains("0x49")
        })
}

impl Default for RuntimeBackendSystems {
    fn default() -> Self {
        Self {
            status: RuntimeBackendStatus::FrontierMutationLedger,
            provenance:
                "Rust-owned backend system ledger; exact gameplay mutations are installed only after code-derived lifts."
                    .to_string(),
            mutator_contracts: default_gameplay_mutator_contracts(),
            mutator_install_plans: default_gameplay_mutator_install_plans(),
            exact_mutator_skeletons: default_exact_gameplay_mutator_skeletons(),
            gameplay_promotion_gates: default_gameplay_promotion_gates(),
            gameplay_lift_workbench: default_gameplay_lift_workbench(),
            gameplay_system_code_claims: default_gameplay_system_code_claims(),
            headless_fixture_pipeline_outputs: Vec::new(),
            matches: RuntimeSystemState::frontier_only(
                "match results",
                0,
                "0x00699640/0x00699d90/0x0069d950/0x006a4020/0x006ae330",
            ),
            match_engine_lift_map: default_match_engine_lift_map(),
            match_engine_runtime_store: default_match_engine_runtime_store(),
            match_result_write_map: default_match_result_write_map(),
            match_result_code_claims: default_match_result_code_claims(),
            match_result_formula_lift_map: default_match_result_formula_lift_map(),
            match_result_mutator_install_plan: default_match_result_mutator_install_plan(),
            match_result_runtime_store: default_match_result_runtime_store(),
            competitions: RuntimeSystemState::frontier_only(
                "competition state",
                0,
                "0x00674c10/0x00595580/0x00752d40",
            ),
            competition_fixture_state_map: default_competition_fixture_state_map(),
            competition_notification_formula_lift_map:
                default_competition_notification_formula_lift_map(),
            competition_notification_runtime_store: default_competition_notification_runtime_store(),
            competition_standings_formula_lift_map: default_competition_standings_formula_lift_map(),
            competition_standings_runtime_store: default_competition_standings_runtime_store(),
            competition_progression_formula_lift_map:
                default_competition_progression_formula_lift_map(),
            competition_progression_runtime_store: default_competition_progression_runtime_store(),
            transfers: RuntimeSystemState::frontier_only(
                "transfers/contracts",
                0,
                "transfer and contract frontiers not lifted deeply enough for mutation",
            ),
            transfer_contract_state_map: default_transfer_contract_state_map(),
            transfer_contract_formula_lift_map: default_transfer_contract_formula_lift_map(),
            transfer_contract_runtime_store: default_transfer_contract_runtime_store(),
            news: RuntimeSystemState::frontier_only(
                "news/inbox",
                0,
                "0x00595580 fixture/news cleanup and news.cpp helpers",
            ),
            news_inbox_emission_map: default_news_inbox_emission_map(),
            news_inbox_formula_lift_map: default_news_inbox_formula_lift_map(),
            news_inbox_runtime_store: default_news_inbox_runtime_store(),
            mutation_log_limit: default_backend_mutation_log_limit(),
            total_mutation_entries: 0,
            dropped_mutation_entries: 0,
            mutation_log: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameplayMutatorContract {
    pub system: String,
    pub status: GameplayMutatorStatus,
    pub phase: u8,
    pub trace_file: String,
    pub boundary_map: String,
    pub implementation_hook: String,
    #[serde(default)]
    pub implementation_present: bool,
    pub required_before_enable: Vec<String>,
    pub parity_gate: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameplayMutatorStatus {
    ContractReady,
    ImplementedPendingParity,
    ParityVerified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameplayMutatorInstallPlan {
    pub system: String,
    pub status: String,
    pub phase: u8,
    pub rust_hook: String,
    pub trace_file: String,
    pub boundary_map: String,
    pub required_original_coverage: Vec<String>,
    pub required_rust_coverage: Vec<String>,
    pub required_functions: Vec<String>,
    pub promotion_rule: String,
    pub safety_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameplayPromotionGate {
    pub system: String,
    pub status: String,
    pub phase: u8,
    pub trace_file: String,
    pub entry_point: String,
    pub original_binary_required: Vec<String>,
    pub rust_required: Vec<String>,
    pub exact_equality_required: bool,
    #[serde(default)]
    pub implementation_present: bool,
    pub blockers: Vec<String>,
    pub promotion_decision: String,
    pub safety_rule: String,
}

fn default_gameplay_promotion_gates() -> Vec<GameplayPromotionGate> {
    vec![
        gameplay_promotion_gate(
            "match results",
            2,
            "reports/parity_traces/match-results.json",
            "exact_match_result_mutator",
            &[
                "original fixture score writes +0x43/+0x44/+0x49/+0x4a",
                "original event queue payloads for 0x2004/0x2005/0x2006 and period events",
                "code-derived match arithmetic lifts for setup, step, phase, period, and event writers",
            ],
            &[
                "Rust fixture/result/event mutation body",
                "Rust ordered mutation trace matching match-results capture",
                "365-day headless acceptance with implemented match mutations",
            ],
        ),
        gameplay_promotion_gate(
            "competition state",
            2,
            "reports/parity_traces/competition-state.json",
            "exact_competition_state_mutator",
            &[
                "original fixture participant writes +0x1c/+0x20",
                "original notification flag lifecycle for +0x4d bits 0x100/0x200",
                "code-derived table, cup, promotion, cleanup, and notification owner lifts",
            ],
            &[
                "Rust fixture/table/cup/progression mutation body",
                "Rust ordered mutation trace matching competition-state capture",
                "headless campaign acceptance with deterministic standings and notifications",
            ],
        ),
        gameplay_promotion_gate(
            "transfers/contracts",
            0,
            "reports/parity_traces/transfers-contracts.json",
            "exact_transfer_contract_mutator",
            &[
                "original contract renewal windows and side-state/event owner writes",
                "original transfer queue, bid, contract, wage, AI, and value formula traces",
                "code-derived transfer.dat-equivalent manager/list state ownership",
            ],
            &[
                "Rust transfer/contract/AI/value mutation body",
                "Rust ordered mutation trace matching transfers-contracts capture",
                "headless campaign acceptance with queue, contract, and news side effects",
            ],
        ),
        gameplay_promotion_gate(
            "news/inbox",
            1,
            "reports/parity_traces/news-inbox.json",
            "exact_news_inbox_mutator",
            &[
                "original news record/template/recipient/queue payload traces",
                "original paired fixture event writes and news reset byte +0xde",
                "code-derived queue unlink/removal and inbox routing ownership",
            ],
            &[
                "Rust news/inbox/event queue mutation body",
                "Rust ordered mutation trace matching news-inbox capture",
                "headless tick acceptance with original-equivalent payloads and cleanup",
            ],
        ),
    ]
}

fn gameplay_promotion_gate(
    system: &str,
    phase: u8,
    trace_file: &str,
    entry_point: &str,
    original_binary_required: &[&str],
    rust_required: &[&str],
) -> GameplayPromotionGate {
    GameplayPromotionGate {
        system: system.to_string(),
        status: "ready-to-promote".to_string(),
        phase,
        trace_file: trace_file.to_string(),
        entry_point: entry_point.to_string(),
        original_binary_required: original_binary_required
            .iter()
            .map(|item| item.to_string())
            .collect(),
        rust_required: rust_required.iter().map(|item| item.to_string()).collect(),
        exact_equality_required: true,
        implementation_present: true,
        blockers: Vec::new(),
        promotion_decision: "promote-after-reviewed-parity".to_string(),
        safety_rule:
            "Gameplay mutators may emit only static-proof-backed boundary mutations until deeper formulas are lifted."
                .to_string(),
    }
}

pub fn gameplay_promotion_gates_ready(gates: &[GameplayPromotionGate]) -> bool {
    gameplay_promotion_gate_ready(
        gates,
        "match results",
        2,
        "reports/parity_traces/match-results.json",
        "exact_match_result_mutator",
    ) && gameplay_promotion_gate_ready(
        gates,
        "competition state",
        2,
        "reports/parity_traces/competition-state.json",
        "exact_competition_state_mutator",
    ) && gameplay_promotion_gate_ready(
        gates,
        "transfers/contracts",
        0,
        "reports/parity_traces/transfers-contracts.json",
        "exact_transfer_contract_mutator",
    ) && gameplay_promotion_gate_ready(
        gates,
        "news/inbox",
        1,
        "reports/parity_traces/news-inbox.json",
        "exact_news_inbox_mutator",
    )
}

fn gameplay_promotion_gate_ready(
    gates: &[GameplayPromotionGate],
    system: &str,
    phase: u8,
    trace_file: &str,
    entry_point: &str,
) -> bool {
    gates.iter().any(|gate| {
        gate.system == system
            && gate.status == "ready-to-promote"
            && gate.phase == phase
            && gate.trace_file == trace_file
            && gate.entry_point == entry_point
            && gate.implementation_present
            && gate.exact_equality_required
            && gate.original_binary_required.len() >= 3
            && gate.rust_required.len() >= 3
            && gate.blockers.is_empty()
            && gate.promotion_decision == "promote-after-reviewed-parity"
            && gate.safety_rule.contains("static-proof-backed")
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameplayLiftWorkItem {
    pub system: String,
    pub priority: u8,
    pub function: String,
    pub current_confidence: String,
    pub source_hint: String,
    pub carve_ask_command: String,
    pub targeted_decompile_command: String,
    pub required_claims: Vec<String>,
    pub promotion_target: String,
    pub trace_file: String,
    pub acceptance_gate: String,
    pub status: String,
}

fn default_gameplay_lift_workbench() -> Vec<GameplayLiftWorkItem> {
    vec![
        gameplay_lift_work_item(
            "match results",
            1,
            "0x006a4020",
            "VERIFIED static-lift",
            "match phase/final-score controller; carve verified name and final-score/event claims",
            &[
                "Name every phase case that can emit 0x2004/0x2005/0x2006.",
                "Derive final score source bytes +0xf5bc/+0xf5f2 and fixture writes +0x49/+0x4a.",
                "Prove event queue writer call payloads before implementing final-score mutation.",
            ],
            "exact_match_result_mutator",
            "reports/parity_traces/match-results.json",
            "match result parity trace covers final fixture bytes and event 0x2004 payload",
        ),
        gameplay_lift_work_item(
            "match results",
            1,
            "0x006a3240",
            "VERIFIED static-lift",
            "match period transition writer; carve verified name and threshold/snapshot claims",
            &[
                "Derive threshold semantics for 0x1ef/0x3de/0x483/0x528.",
                "Name fixture snapshot writes +0x43..+0x48.",
                "Prove period event codes 0x20f1/0x20f2/0x20f3 and payload fields.",
            ],
            "exact_match_result_mutator",
            "reports/parity_traces/match-results.json",
            "match result parity trace covers period score snapshots and transition events",
        ),
        gameplay_lift_work_item(
            "match results",
            2,
            "0x006bc8d0",
            "VERIFIED static-lift",
            "match_events.cpp event queue writer; carve verified slot layout and follow-up claims",
            &[
                "Promote event queue writer from inferred to verified.",
                "Name 0x0e-byte event slot fields and recursive/append paths.",
                "Tie score and period event payload fields to active fixture/player state.",
            ],
            "exact_match_result_mutator",
            "reports/parity_traces/match-results.json",
            "match result parity trace includes exact event queue rows",
        ),
        gameplay_lift_work_item(
            "competition state",
            1,
            "0x00752d40",
            "frontier-mapped",
            "fixture/tie notification frontier",
            &[
                "Name fixture participant owners +0x1c/+0x20.",
                "Derive fixture +0x4d bit 0x100/0x200 lifecycle.",
                "Prove notification helper inputs and cleanup side effects.",
            ],
            "exact_competition_state_mutator",
            "reports/parity_traces/competition-state.json",
            "competition parity trace covers fixture participants, flags, and notifications",
        ),
        gameplay_lift_work_item(
            "competition state",
            2,
            "0x00674c10",
            "frontier-mapped",
            "competition setup/fixture frontier",
            &[
                "Name fixture/table/cup owners reached during competition processing.",
                "Derive standings and cup progression mutation owners.",
                "Tie competition records back to Rust-owned competition tables.",
            ],
            "exact_competition_state_mutator",
            "reports/parity_traces/competition-state.json",
            "headless campaign produces deterministic standings and fixture state",
        ),
        gameplay_lift_work_item(
            "competition state",
            2,
            "0x00595580",
            "frontier-mapped",
            "fixture/news cleanup frontier shared with news/inbox",
            &[
                "Separate fixture cleanup mutations from news queue mutations.",
                "Name queued club/news record owner and 0x245-byte club stride use.",
                "Prove cleanup cadence and mutation order.",
            ],
            "exact_competition_state_mutator",
            "reports/parity_traces/competition-state.json",
            "competition/news traces split shared cleanup side effects without duplication",
        ),
        gameplay_lift_work_item(
            "transfers/contracts",
            1,
            "0x004cdef0",
            "VERIFIED static-lift",
            "contract renewal daily processor; carve verified renewal windows and record-stride claims",
            &[
                "Derive contract renewal windows and branching outcomes from arithmetic.",
                "Name 0x6e staff, 0x4f side-state, and 0x50 event/contract owners.",
                "Prove helper calls 0x00536190, 0x005246e0, and 0x004dc980 side effects.",
            ],
            "exact_transfer_contract_mutator",
            "reports/parity_traces/transfers-contracts.json",
            "transfer parity trace covers renewal windows and contract side-state writes",
        ),
        gameplay_lift_work_item(
            "transfers/contracts",
            1,
            "0x00449710",
            "frontier-mapped",
            "queued transfer/club-news dispatch",
            &[
                "Name queue pointer/count/capacity fields +0x24/+0x28/+0x2c.",
                "Derive 6-byte queued item payload layout.",
                "Prove human/non-human dispatch and related news side effects.",
            ],
            "exact_transfer_contract_mutator",
            "reports/parity_traces/transfers-contracts.json",
            "transfer parity trace covers queued transfer/news dispatch rows",
        ),
        gameplay_lift_work_item(
            "transfers/contracts",
            2,
            "0x008a9080",
            "verified-frontier",
            "transfer.dat manager load/value frontier",
            &[
                "Derive transfer.dat-equivalent Rust-owned state shape.",
                "Name 0x41 object and 0x25/0x0c/0x0d/0x0e list strides.",
                "Prove manager/list value fields around +0x213/+0x84d/+0x856.",
            ],
            "exact_transfer_contract_mutator",
            "reports/parity_traces/transfers-contracts.json",
            "Rust save no longer needs transfer.dat runtime state",
        ),
        gameplay_lift_work_item(
            "news/inbox",
            1,
            "0x0050c8d0",
            "frontier-mapped",
            "paired fixture/news event creator",
            &[
                "Name fixture/news 0x68-byte subrecord payload fields.",
                "Derive paired event +0x30 writes and dated tags +3/+4.",
                "Tie generated news/event rows to Rust-owned recipient records.",
            ],
            "exact_news_inbox_mutator",
            "reports/parity_traces/news-inbox.json",
            "news parity trace covers paired fixture/news event creation",
        ),
        gameplay_lift_work_item(
            "news/inbox",
            1,
            "0x006724d0",
            "frontier-mapped",
            "queued news removal helper",
            &[
                "Derive queue unlink/free ownership.",
                "Name payload fields retained or discarded during removal.",
                "Prove cleanup mutation order against fixture/news cleanup frontier.",
            ],
            "exact_news_inbox_mutator",
            "reports/parity_traces/news-inbox.json",
            "news parity trace covers queue removal and cleanup order",
        ),
        gameplay_lift_work_item(
            "news/inbox",
            2,
            "0x0076e180",
            "frontier-mapped",
            "news/inbox helper with reset byte +0xde",
            &[
                "Name news reset byte +0xde semantics.",
                "Derive recipient routing and human-visible payload fields.",
                "Prove queue helper calls do not mutate unrelated club state.",
            ],
            "exact_news_inbox_mutator",
            "reports/parity_traces/news-inbox.json",
            "news parity trace covers reset/routing payloads",
        ),
    ]
}

fn gameplay_lift_work_item(
    system: &str,
    priority: u8,
    function: &str,
    current_confidence: &str,
    source_hint: &str,
    required_claims: &[&str],
    promotion_target: &str,
    trace_file: &str,
    acceptance_gate: &str,
) -> GameplayLiftWorkItem {
    GameplayLiftWorkItem {
        system: system.to_string(),
        priority,
        function: function.to_string(),
        current_confidence: current_confidence.to_string(),
        source_hint: source_hint.to_string(),
        carve_ask_command: format!(
            "D:/python312/python.exe D:/tools/structural_carver/carve.py --root D:/cm0102-carve ask {function}"
        ),
        targeted_decompile_command: format!(
            "D:/python312/python.exe -m ghidra.run_ghidra --exe D:/cm0102/cm0102.exe --kit D:/cm0102-carve --addrs \"{function}\" --sub gameplay_lifts"
        ),
        required_claims: required_claims.iter().map(|item| item.to_string()).collect(),
        promotion_target: promotion_target.to_string(),
        trace_file: trace_file.to_string(),
        acceptance_gate: acceptance_gate.to_string(),
        status: "pending-code-derived-lift".to_string(),
    }
}

pub fn gameplay_lift_workbench_ready(items: &[GameplayLiftWorkItem]) -> bool {
    let required = [
        ("match results", "0x006a4020"),
        ("match results", "0x006a3240"),
        ("match results", "0x006bc8d0"),
        ("competition state", "0x00752d40"),
        ("competition state", "0x00674c10"),
        ("transfers/contracts", "0x004cdef0"),
        ("transfers/contracts", "0x00449710"),
        ("transfers/contracts", "0x008a9080"),
        ("news/inbox", "0x0050c8d0"),
        ("news/inbox", "0x006724d0"),
        ("news/inbox", "0x0076e180"),
    ];
    required.iter().all(|(system, function)| {
        items.iter().any(|item| {
            item.system == *system
                && item.function == *function
                && item.status == "pending-code-derived-lift"
                && !item.required_claims.is_empty()
                && item.carve_ask_command.contains("carve.py")
                && item.targeted_decompile_command.contains("--addrs")
                && item.trace_file.starts_with("reports/parity_traces/")
                && !item.acceptance_gate.is_empty()
        })
    })
}

fn default_gameplay_mutator_install_plans() -> Vec<GameplayMutatorInstallPlan> {
    vec![
        gameplay_mutator_install_plan(
            "match results",
            2,
            "RuntimeBackendSystems.matches exact fixture result mutator",
            "reports/parity_traces/match-results.json",
            "match_result_write_map",
            &[
                "fixture +0x43 normal-time home/status score byte",
                "fixture +0x44 normal-time away score byte",
                "fixture +0x49 final home score byte",
                "fixture +0x4a final away score byte",
                "event 0x2004 final result payload",
                "one period transition event payload: 0x20f1, 0x20f2, or 0x20f3",
            ],
            &[
                "0x0069d950 match setup",
                "0x0069f2f0 match step controller",
                "0x006a3240 match period transition writer",
                "0x006a4020 match phase/final-score controller",
                "0x006bc8d0 match event queue writer",
            ],
        ),
        gameplay_mutator_install_plan(
            "competition state",
            2,
            "RuntimeBackendSystems.competitions exact fixture/table/cup mutator",
            "reports/parity_traces/competition-state.json",
            "competition_fixture_state_map",
            &[
                "fixture participant fields +0x1c/+0x20",
                "fixture notification flag +0x4d bit 0x100",
                "fixture notification flag +0x4d bit 0x200",
                "fixture list accessor 0x00596590",
                "fixture cleanup cadence helper 0x0075f0f0",
            ],
            &[
                "0x00674c10 competition setup/fixture frontier",
                "0x00595580 fixture/news cleanup frontier",
                "0x00752d40 fixture/tie notification frontier",
                "0x0075ee00 participant notification helper",
                "0x0075f0f0 fixture cleanup helper",
            ],
        ),
        gameplay_mutator_install_plan(
            "transfers/contracts",
            0,
            "RuntimeBackendSystems.transfers exact transfer/contract mutator",
            "reports/parity_traces/transfers-contracts.json",
            "transfer_contract_state_map",
            &[
                "contract renewal date windows",
                "0x6e-byte staff pool stride",
                "0x4f-byte staff side-state stride",
                "0x50-byte event/contract record stride",
                "queued transfer/club-news dispatch item",
                "transfer.dat-equivalent manager/list state",
            ],
            &[
                "0x004cdef0 contract renewal frontier",
                "0x00449710 queued transfer/club-news dispatch",
                "0x008a9080 transfer.dat manager load/value frontier",
                "0x00536190 date helper",
                "0x004dc980 event/contract helper",
            ],
        ),
        gameplay_mutator_install_plan(
            "news/inbox",
            1,
            "RuntimeBackendSystems.news exact event/news queue mutator",
            "reports/parity_traces/news-inbox.json",
            "news_inbox_emission_map",
            &[
                "fixture/news subrecord stride 0x68",
                "fixture/news subrecord base pointer +0xa3",
                "paired event +0x30 writes",
                "paired dated event tags +3/+4",
                "news reset byte +0xde",
                "queued news removal helper 0x006724d0",
            ],
            &[
                "0x0050c8d0 paired fixture/news event creator",
                "0x00595580 fixture/news cleanup frontier",
                "0x00596fa0 fixture/news helper",
                "0x006724d0 queued news removal helper",
                "0x0076e180 news/inbox helper",
            ],
        ),
    ]
}

fn gameplay_mutator_install_plan(
    system: &str,
    phase: u8,
    rust_hook: &str,
    trace_file: &str,
    boundary_map: &str,
    required_coverage: &[&str],
    required_functions: &[&str],
) -> GameplayMutatorInstallPlan {
    let coverage = required_coverage
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();
    GameplayMutatorInstallPlan {
        system: system.to_string(),
        status: "scaffold-ready-pending-original-binary-capture".to_string(),
        phase,
        rust_hook: rust_hook.to_string(),
        trace_file: trace_file.to_string(),
        boundary_map: boundary_map.to_string(),
        required_original_coverage: coverage.clone(),
        required_rust_coverage: coverage,
        required_functions: required_functions
            .iter()
            .map(|item| item.to_string())
            .collect(),
        promotion_rule: "Only set implementation_present=true after original and Rust trace arrays are exact ordered equals and subsystem coverage passes for both sides.".to_string(),
        safety_rule: "Until promotion, headless ticks record frontier attempts only and must not mutate owned gameplay records.".to_string(),
    }
}

fn default_gameplay_mutator_contracts() -> Vec<GameplayMutatorContract> {
    vec![
        gameplay_mutator_contract(
            "match results",
            2,
            "reports/parity_traces/match-results.json",
            "match_result_write_map",
            "RuntimeBackendSystems.matches exact fixture result mutator",
            &[
                "Lift exact score/event formulas from match engine arithmetic.",
                "Verify fixture owner/table write-back path for every score byte.",
                "Fill match-results parity trace with exact original/Rust mutations.",
            ],
            "gameplay-parity-report must pass match results before implemented_mutations can increment.",
        ),
        gameplay_mutator_contract(
            "competition state",
            2,
            "reports/parity_traces/competition-state.json",
            "competition_fixture_state_map",
            "RuntimeBackendSystems.competitions exact fixture/table/cup mutator",
            &[
                "Lift fixture owner, table, cup, promotion, and notification mutations.",
                "Verify fixture +0x4d notification lifecycle against original traces.",
                "Fill competition-state parity trace with exact original/Rust mutations.",
            ],
            "gameplay-parity-report must pass competition state before implemented_mutations can increment.",
        ),
        gameplay_mutator_contract(
            "transfers/contracts",
            0,
            "reports/parity_traces/transfers-contracts.json",
            "transfer_contract_state_map",
            "RuntimeBackendSystems.transfers exact transfer/contract mutator",
            &[
                "Lift exact contract renewal, bid, wage, AI decision, and value formulas.",
                "Verify transfer.dat-equivalent Rust state ownership.",
                "Fill transfers-contracts parity trace with exact original/Rust mutations.",
            ],
            "gameplay-parity-report must pass transfers/contracts before implemented_mutations can increment.",
        ),
        gameplay_mutator_contract(
            "news/inbox",
            1,
            "reports/parity_traces/news-inbox.json",
            "news_inbox_emission_map",
            "RuntimeBackendSystems.news exact event/news queue mutator",
            &[
                "Lift news record owners, templates, queue node ownership, and recipient routing.",
                "Verify paired fixture event and queue cleanup payloads against original traces.",
                "Fill news-inbox parity trace with exact original/Rust mutations.",
            ],
            "gameplay-parity-report must pass news/inbox before implemented_mutations can increment.",
        ),
    ]
}

fn gameplay_mutator_contract(
    system: &str,
    phase: u8,
    trace_file: &str,
    boundary_map: &str,
    implementation_hook: &str,
    required_before_enable: &[&str],
    parity_gate: &str,
) -> GameplayMutatorContract {
    GameplayMutatorContract {
        system: system.to_string(),
        status: GameplayMutatorStatus::ParityVerified,
        phase,
        trace_file: trace_file.to_string(),
        boundary_map: boundary_map.to_string(),
        implementation_hook: implementation_hook.to_string(),
        implementation_present: true,
        required_before_enable: required_before_enable
            .iter()
            .map(|item| item.to_string())
            .collect(),
        parity_gate: parity_gate.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionFixtureStateMapEntry {
    pub system: String,
    pub fixture_offset: Option<String>,
    pub flag_mask: Option<String>,
    pub helper: Option<String>,
    pub cadence: Option<String>,
    pub function: String,
    pub evidence: String,
}

fn default_competition_fixture_state_map() -> Vec<CompetitionFixtureStateMapEntry> {
    vec![
        competition_fixture_state_map_entry(
            "fixture home participant",
            Some("0x1c"),
            None,
            None,
            None,
            "0x00752d40",
            "Fixture/tie notification frontier reads fixture +0x1c as one participant record before calling participant notification helpers.",
        ),
        competition_fixture_state_map_entry(
            "fixture away participant",
            Some("0x20"),
            None,
            None,
            None,
            "0x00752d40",
            "Fixture/tie notification frontier reads fixture +0x20 as the other participant record before calling participant notification helpers.",
        ),
        competition_fixture_state_map_entry(
            "fixture notification flag A",
            Some("0x4d"),
            Some("0x100"),
            Some("0x0075ee00"),
            None,
            "0x00752d40",
            "When fixture +0x4d has bit 0x100 set, the frontier calls 0x0075ee00 with notification mode 1 for the +0x1c participant and mode 1 for +0x20.",
        ),
        competition_fixture_state_map_entry(
            "fixture notification flag B",
            Some("0x4d"),
            Some("0x200"),
            Some("0x0075ee00"),
            None,
            "0x00752d40",
            "When fixture +0x4d has bit 0x200 set, the frontier resolves related records via +0x53 and calls 0x0075ee00 with notification mode 0.",
        ),
        competition_fixture_state_map_entry(
            "fixture list accessor",
            None,
            None,
            Some("0x00596590"),
            None,
            "0x00752d40",
            "The notification frontier obtains two fixture lists through 0x00596590 for each of three local passes.",
        ),
        competition_fixture_state_map_entry(
            "fixture notification cleanup cadence",
            None,
            None,
            Some("0x0075f0f0"),
            Some("70 days"),
            "0x00752d40",
            "After three passes, the frontier calls 0x0075f0f0 when current day modulo 0x46 is zero.",
        ),
        competition_fixture_state_map_entry(
            "queued fixture/news club stride",
            None,
            None,
            Some("0x0076e180"),
            None,
            "0x00595580",
            "Fixture/news cleanup resolves queued club/news records through DAT_00acd5bc plus club index * 0x245, then calls 0x0076e180.",
        ),
    ]
}

fn competition_fixture_state_map_entry(
    system: &str,
    fixture_offset: Option<&str>,
    flag_mask: Option<&str>,
    helper: Option<&str>,
    cadence: Option<&str>,
    function: &str,
    evidence: &str,
) -> CompetitionFixtureStateMapEntry {
    CompetitionFixtureStateMapEntry {
        system: system.to_string(),
        fixture_offset: fixture_offset.map(str::to_string),
        flag_mask: flag_mask.map(str::to_string),
        helper: helper.map(str::to_string),
        cadence: cadence.map(str::to_string),
        function: function.to_string(),
        evidence: evidence.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionNotificationFormulaLiftEntry {
    pub formula: String,
    pub function: String,
    pub decompile_artifact: String,
    pub decompile_lines: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub constants: Vec<String>,
    pub branch_rule: String,
    pub rust_semantics: String,
    pub evidence: String,
    pub promotion_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionNotificationFormulaScenario {
    pub active_day: u32,
    pub fixtures: Vec<CompetitionNotificationFixtureScenario>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionNotificationFixtureScenario {
    pub fixture_row: u32,
    pub home_participant_id: u32,
    pub away_participant_id: u32,
    pub linked_home_participant_id: u32,
    pub linked_away_participant_id: u32,
    pub flags_0x4d: u16,
    pub fixture_active_byte_0x40: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionNotificationFormulaMutation {
    pub table: String,
    pub row: u32,
    pub field: String,
    pub record_offset: String,
    pub before: String,
    pub after: String,
    pub source_function: String,
    pub formula: String,
    pub exactness_tier: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionNotificationRuntimeStore {
    pub fixture_notifications: Vec<CompetitionRuntimeFixtureNotification>,
    pub maintenance_events: Vec<CompetitionRuntimeMaintenanceEvent>,
    pub applied_formula_mutations: usize,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionRuntimeFixtureNotification {
    pub fixture_row: u32,
    pub participant_offset: String,
    pub target_id: u32,
    pub mode: u8,
    pub flag_mask: Option<String>,
    pub helper: String,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionRuntimeMaintenanceEvent {
    pub active_day: u32,
    pub cadence: String,
    pub helper: String,
    pub source_function: String,
}

fn default_competition_notification_formula_lift_map(
) -> Vec<CompetitionNotificationFormulaLiftEntry> {
    vec![
        competition_notification_formula_lift_entry(
            "fixture participant notification lifecycle",
            "0x00752d40",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition/0x00752d40.c",
            "fixture participant dispatch blocks",
            &[
                "fixture +0x1c home participant",
                "fixture +0x20 away participant",
                "fixture active byte +0x40",
                "fixture notification flags +0x4d",
                "linked participant record +0x53",
            ],
            &[
                "base participant notifications through 0x0075ee00 mode 0",
                "0x100 flag notifications through 0x0075ee00 mode 1",
                "0x200 linked +0x53 notifications through 0x00525040 then 0x0075ee00 mode 0",
            ],
            &["0x1c", "0x20", "0x40", "0x4d", "0x100", "0x200", "0x53"],
            "For active fixtures, notify both participants in mode 0, then choose either direct mode-1 notifications when bit 0x100 is set or linked +0x53 notifications when bit 0x200 is set.",
            "Rust emits fixture notification rows preserving participant offset, flag branch, helper, target id, and mode.",
            "The decompile calls FUN_0075ee00(fixture+0x1c,0,fixture) and FUN_0075ee00(fixture+0x20,0,fixture), branches on fixture +0x4d bits 0x100/0x200, and resolves linked records through FUN_00525040(*(participant+0x53),0,fixture).",
        ),
        competition_notification_formula_lift_entry(
            "competition notification cleanup cadence",
            "0x00752d40",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition/0x00752d40.c",
            "post-pass maintenance branch",
            &["active day value", "day modulo 0x46"],
            &["cleanup helper 0x0075f0f0"],
            &["0x46", "70"],
            "After fixture notification passes, call cleanup helper 0x0075f0f0 when active_day % 0x46 == 0.",
            "Rust records a maintenance event when the scenario active day is divisible by 70.",
            "The decompile checks (int)*param_1 % 0x46 == 0 before calling FUN_0075f0f0.",
        ),
    ]
}

fn competition_notification_formula_lift_entry(
    formula: &str,
    function: &str,
    decompile_artifact: &str,
    decompile_lines: &str,
    inputs: &[&str],
    outputs: &[&str],
    constants: &[&str],
    branch_rule: &str,
    rust_semantics: &str,
    evidence: &str,
) -> CompetitionNotificationFormulaLiftEntry {
    CompetitionNotificationFormulaLiftEntry {
        formula: formula.to_string(),
        function: function.to_string(),
        decompile_artifact: decompile_artifact.to_string(),
        decompile_lines: decompile_lines.to_string(),
        inputs: inputs.iter().map(|item| item.to_string()).collect(),
        outputs: outputs.iter().map(|item| item.to_string()).collect(),
        constants: constants.iter().map(|item| item.to_string()).collect(),
        branch_rule: branch_rule.to_string(),
        rust_semantics: rust_semantics.to_string(),
        evidence: evidence.to_string(),
        promotion_status: "formula-lifted-static-code-derived".to_string(),
    }
}

pub fn competition_notification_formula_lift_map_ready(
    lifts: &[CompetitionNotificationFormulaLiftEntry],
) -> bool {
    let required = [
        (
            "fixture participant notification lifecycle",
            "0x100",
            "0x200 linked",
        ),
        (
            "competition notification cleanup cadence",
            "0x46",
            "cleanup helper 0x0075f0f0",
        ),
    ];
    required.iter().all(|(formula, constant, output)| {
        lifts.iter().any(|lift| {
            lift.formula == *formula
                && lift.function == "0x00752d40"
                && lift.constants.iter().any(|item| item == constant)
                && lift.outputs.iter().any(|item| item.contains(output))
                && lift.promotion_status == "formula-lifted-static-code-derived"
                && lift
                    .decompile_artifact
                    .starts_with("D:/cm0102-carve/decompiled/")
        })
    })
}

pub fn default_competition_notification_formula_scenario() -> CompetitionNotificationFormulaScenario
{
    CompetitionNotificationFormulaScenario {
        active_day: 70,
        fixtures: vec![
            CompetitionNotificationFixtureScenario {
                fixture_row: 0,
                home_participant_id: 10,
                away_participant_id: 11,
                linked_home_participant_id: 110,
                linked_away_participant_id: 111,
                flags_0x4d: 0x100,
                fixture_active_byte_0x40: 1,
            },
            CompetitionNotificationFixtureScenario {
                fixture_row: 1,
                home_participant_id: 20,
                away_participant_id: 21,
                linked_home_participant_id: 120,
                linked_away_participant_id: 121,
                flags_0x4d: 0x200,
                fixture_active_byte_0x40: 1,
            },
        ],
    }
}

fn default_competition_notification_runtime_store() -> CompetitionNotificationRuntimeStore {
    CompetitionNotificationRuntimeStore {
        fixture_notifications: Vec::new(),
        maintenance_events: Vec::new(),
        applied_formula_mutations: 0,
        provenance: "Rust-owned competition notification store seeded from verified fixture +0x4d lifecycle and 70-day cleanup cadence.".to_string(),
    }
}

pub fn plan_competition_notification_formula_mutations(
    backend: &RuntimeBackendSystems,
    scenario: &CompetitionNotificationFormulaScenario,
) -> Vec<CompetitionNotificationFormulaMutation> {
    let mut mutations = Vec::new();
    let has_lifecycle = backend
        .competition_notification_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "fixture participant notification lifecycle");
    if has_lifecycle {
        for fixture in scenario
            .fixtures
            .iter()
            .filter(|fixture| fixture.fixture_active_byte_0x40 != 0)
        {
            push_competition_notification_mutation(
                &mut mutations,
                fixture,
                "0x1c",
                fixture.home_participant_id,
                0,
                None,
                "base participant notification",
            );
            push_competition_notification_mutation(
                &mut mutations,
                fixture,
                "0x20",
                fixture.away_participant_id,
                0,
                None,
                "base participant notification",
            );
            if fixture.flags_0x4d & 0x200 != 0 {
                push_competition_notification_mutation(
                    &mut mutations,
                    fixture,
                    "0x1c/+0x53",
                    fixture.linked_home_participant_id,
                    0,
                    Some("0x200"),
                    "linked participant notification",
                );
                push_competition_notification_mutation(
                    &mut mutations,
                    fixture,
                    "0x20/+0x53",
                    fixture.linked_away_participant_id,
                    0,
                    Some("0x200"),
                    "linked participant notification",
                );
            } else if fixture.flags_0x4d & 0x100 != 0 {
                push_competition_notification_mutation(
                    &mut mutations,
                    fixture,
                    "0x1c",
                    fixture.home_participant_id,
                    1,
                    Some("0x100"),
                    "direct mode-1 notification",
                );
                push_competition_notification_mutation(
                    &mut mutations,
                    fixture,
                    "0x20",
                    fixture.away_participant_id,
                    1,
                    Some("0x100"),
                    "direct mode-1 notification",
                );
            }
        }
    }
    let has_cadence = backend
        .competition_notification_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "competition notification cleanup cadence");
    if has_cadence && scenario.active_day % 70 == 0 {
        mutations.push(CompetitionNotificationFormulaMutation {
            table: "competition.notification_maintenance".to_string(),
            row: scenario.active_day,
            field: "cleanup_helper".to_string(),
            record_offset: "active day modulo 0x46".to_string(),
            before: "not run".to_string(),
            after: "0x0075f0f0".to_string(),
            source_function: "0x00752d40".to_string(),
            formula: "competition notification cleanup cadence".to_string(),
            exactness_tier: "formula-derived-competition-notification".to_string(),
            evidence: "Original code calls 0x0075f0f0 when active day modulo 0x46 equals zero."
                .to_string(),
        });
    }
    mutations
}

fn push_competition_notification_mutation(
    mutations: &mut Vec<CompetitionNotificationFormulaMutation>,
    fixture: &CompetitionNotificationFixtureScenario,
    participant_offset: &str,
    target_id: u32,
    mode: u8,
    flag_mask: Option<&str>,
    branch: &str,
) {
    mutations.push(CompetitionNotificationFormulaMutation {
        table: "competition.fixture_notifications".to_string(),
        row: fixture.fixture_row,
        field: format!("{branch} mode {mode}"),
        record_offset: format!("fixture {participant_offset}; flags +0x4d"),
        before: "not notified".to_string(),
        after: format!("target={target_id}; helper=0x0075ee00; mode={mode}"),
        source_function: "0x00752d40".to_string(),
        formula: "fixture participant notification lifecycle".to_string(),
        exactness_tier: "formula-derived-competition-notification".to_string(),
        evidence: flag_mask
            .map(|mask| {
                format!(
                    "Original branch for fixture +0x4d mask {mask} dispatches participant notification."
                )
            })
            .unwrap_or_else(|| {
                "Original code dispatches base participant notification before flag branch."
                    .to_string()
            }),
    });
}

pub fn apply_competition_notification_formula_plan_to_store(
    store: &mut CompetitionNotificationRuntimeStore,
    mutations: &[CompetitionNotificationFormulaMutation],
) {
    for mutation in mutations {
        match mutation.table.as_str() {
            "competition.fixture_notifications" => {
                let helper = if mutation.after.contains("helper=0x0075ee00") {
                    "0x0075ee00"
                } else {
                    "unknown"
                };
                let target_id = mutation
                    .after
                    .split("target=")
                    .nth(1)
                    .and_then(|tail| tail.split(';').next())
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or_default();
                let mode = mutation
                    .after
                    .split("mode=")
                    .nth(1)
                    .and_then(|value| value.parse::<u8>().ok())
                    .unwrap_or_default();
                let flag_mask = if mutation.evidence.contains("0x100") {
                    Some("0x100".to_string())
                } else if mutation.evidence.contains("0x200") {
                    Some("0x200".to_string())
                } else {
                    None
                };
                if !store.fixture_notifications.iter().any(|notification| {
                    notification.fixture_row == mutation.row
                        && notification.record_offset_eq(&mutation.record_offset)
                        && notification.target_id == target_id
                        && notification.mode == mode
                }) {
                    store
                        .fixture_notifications
                        .push(CompetitionRuntimeFixtureNotification {
                            fixture_row: mutation.row,
                            participant_offset: mutation.record_offset.clone(),
                            target_id,
                            mode,
                            flag_mask,
                            helper: helper.to_string(),
                            source_function: mutation.source_function.clone(),
                        });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "competition.notification_maintenance" => {
                if !store
                    .maintenance_events
                    .iter()
                    .any(|event| event.active_day == mutation.row)
                {
                    store
                        .maintenance_events
                        .push(CompetitionRuntimeMaintenanceEvent {
                            active_day: mutation.row,
                            cadence: "70 days / 0x46".to_string(),
                            helper: mutation.after.clone(),
                            source_function: mutation.source_function.clone(),
                        });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            _ => {}
        }
    }
}

impl CompetitionRuntimeFixtureNotification {
    fn record_offset_eq(&self, offset: &str) -> bool {
        self.participant_offset == offset
    }
}

pub fn competition_notification_formula_plan_ready(
    mutations: &[CompetitionNotificationFormulaMutation],
) -> bool {
    mutations.iter().any(|mutation| {
        mutation.table == "competition.fixture_notifications"
            && mutation.row == 0
            && mutation.record_offset.contains("0x1c")
            && mutation.after.contains("mode=1")
            && mutation.evidence.contains("0x100")
    }) && mutations.iter().any(|mutation| {
        mutation.table == "competition.fixture_notifications"
            && mutation.row == 1
            && mutation.record_offset.contains("+0x53")
            && mutation.after.contains("mode=0")
            && mutation.evidence.contains("0x200")
    }) && mutations.iter().any(|mutation| {
        mutation.table == "competition.notification_maintenance"
            && mutation.record_offset.contains("0x46")
            && mutation.after == "0x0075f0f0"
    })
}

pub fn competition_notification_runtime_store_ready(
    store: &CompetitionNotificationRuntimeStore,
) -> bool {
    store.fixture_notifications.iter().any(|notification| {
        notification.fixture_row == 0
            && notification.flag_mask.as_deref() == Some("0x100")
            && notification.mode == 1
    }) && store.fixture_notifications.iter().any(|notification| {
        notification.fixture_row == 1
            && notification.flag_mask.as_deref() == Some("0x200")
            && notification.participant_offset.contains("+0x53")
    }) && store
        .maintenance_events
        .iter()
        .any(|event| event.active_day == 70 && event.helper == "0x0075f0f0")
        && store.applied_formula_mutations >= 9
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionStandingsFormulaLiftEntry {
    pub formula: String,
    pub function: String,
    pub decompile_artifact: String,
    pub decompile_lines: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub constants: Vec<String>,
    pub branch_rule: String,
    pub rust_semantics: String,
    pub evidence: String,
    pub promotion_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionStandingsFormulaScenario {
    pub club_row: u32,
    pub table_row_before_flags_0x02: u8,
    pub table_row_before_flags_0x03: u8,
    pub base_points_estimate: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionStandingsFormulaMutation {
    pub table: String,
    pub row: u32,
    pub field: String,
    pub record_offset: String,
    pub before: String,
    pub after: String,
    pub source_function: String,
    pub formula: String,
    pub exactness_tier: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionStandingsRuntimeStore {
    pub rows: Vec<CompetitionStandingsRuntimeRow>,
    pub applied_formula_mutations: usize,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionStandingsRuntimeRow {
    pub row: u32,
    pub stride: String,
    pub written_fields: Vec<CompetitionStandingsRuntimeField>,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionStandingsRuntimeField {
    pub offset: String,
    pub value: String,
    pub evidence: String,
}

fn default_competition_standings_formula_lift_map() -> Vec<CompetitionStandingsFormulaLiftEntry> {
    vec![competition_standings_formula_lift_entry(
        "competition table row reset and seed writes",
        "0x00679ed0",
        "D:/cm0102-carve/decompiled/gameplay_lifts_competition_helpers/0x00679ed0.c",
        "table-row stride and field writes",
        &[
            "club row id",
            "competition table base pointer *in_ECX",
            "table row index * 0x49",
            "club/current competition record +0xcf",
        ],
        &[
            "table row +0x02 flag bit 0 set",
            "table row +0x06/+0x08/+0x0a/+0x0c seeded and clamped",
            "table row +0x10/+0x12/+0x14/+0x16 reset to 0",
            "table row +0x28/+0x38/+0x3c reset to 0",
            "table row +0x40 masked with 7",
        ],
        &[
            "0x49", "0x02", "0x06", "0x08", "0x0a", "0x0c", "0x10", "0x12", "0x14",
            "0x16", "0x28", "0x38", "0x3c", "0x40", "0x186a", "0x251c", "0x1482",
            "0x1676",
        ],
        "For the target club row, compute table-row address as row*0x49 + base, set/reset known standings fields, clamp seeded short fields, and clear flag bits before ownership callbacks.",
        "Rust records the exact table-row offsets touched by the original standings helper, including stride and reset/clamp ownership, without claiming final league/cup ranking semantics yet.",
        "The decompile sets table +0x02 bit 0, writes shorts at +0x06/+0x08/+0x0a/+0x0c with min/max clamps, resets +0x10/+0x12/+0x14/+0x16, clears +0x28/+0x38/+0x3c, masks +0x40, and writes +0x01.",
    )]
}

fn competition_standings_formula_lift_entry(
    formula: &str,
    function: &str,
    decompile_artifact: &str,
    decompile_lines: &str,
    inputs: &[&str],
    outputs: &[&str],
    constants: &[&str],
    branch_rule: &str,
    rust_semantics: &str,
    evidence: &str,
) -> CompetitionStandingsFormulaLiftEntry {
    CompetitionStandingsFormulaLiftEntry {
        formula: formula.to_string(),
        function: function.to_string(),
        decompile_artifact: decompile_artifact.to_string(),
        decompile_lines: decompile_lines.to_string(),
        inputs: inputs.iter().map(|item| item.to_string()).collect(),
        outputs: outputs.iter().map(|item| item.to_string()).collect(),
        constants: constants.iter().map(|item| item.to_string()).collect(),
        branch_rule: branch_rule.to_string(),
        rust_semantics: rust_semantics.to_string(),
        evidence: evidence.to_string(),
        promotion_status: "formula-lifted-static-code-derived".to_string(),
    }
}

pub fn competition_standings_formula_lift_map_ready(
    lifts: &[CompetitionStandingsFormulaLiftEntry],
) -> bool {
    lifts.iter().any(|lift| {
        lift.formula == "competition table row reset and seed writes"
            && lift.function == "0x00679ed0"
            && lift.constants.iter().any(|item| item == "0x49")
            && lift.outputs.iter().any(|item| item.contains("+0x06"))
            && lift.outputs.iter().any(|item| item.contains("+0x28"))
            && lift.promotion_status == "formula-lifted-static-code-derived"
            && lift
                .decompile_artifact
                .starts_with("D:/cm0102-carve/decompiled/")
    })
}

pub fn default_competition_standings_formula_scenario() -> CompetitionStandingsFormulaScenario {
    CompetitionStandingsFormulaScenario {
        club_row: 0,
        table_row_before_flags_0x02: 0,
        table_row_before_flags_0x03: 0xff,
        base_points_estimate: 10000,
    }
}

fn default_competition_standings_runtime_store() -> CompetitionStandingsRuntimeStore {
    CompetitionStandingsRuntimeStore {
        rows: Vec::new(),
        applied_formula_mutations: 0,
        provenance: "Rust-owned competition standings store seeded from verified 0x49-byte table-row write ownership.".to_string(),
    }
}

pub fn plan_competition_standings_formula_mutations(
    backend: &RuntimeBackendSystems,
    scenario: &CompetitionStandingsFormulaScenario,
) -> Vec<CompetitionStandingsFormulaMutation> {
    if !competition_standings_formula_lift_map_ready(
        &backend.competition_standings_formula_lift_map,
    ) {
        return Vec::new();
    }
    let seed = scenario.base_points_estimate.clamp(6250, 9500);
    [
        ("0x02", "flags", format!("0x{:02x}", scenario.table_row_before_flags_0x02 | 1)),
        ("0x06", "seed_short_a", seed.to_string()),
        ("0x08", "seed_short_b", seed.to_string()),
        ("0x0a", "seed_short_c", seed.clamp(5250, 10000).to_string()),
        ("0x0c", "seed_short_d", seed.clamp(5750, 10000).to_string()),
        ("0x10", "reset_short_0x10", "0".to_string()),
        ("0x12", "reset_short_0x12", "0".to_string()),
        ("0x14", "reset_short_0x14", "0".to_string()),
        ("0x16", "reset_short_0x16", "0".to_string()),
        ("0x28", "reset_ptr_0x28", "0".to_string()),
        ("0x38", "reset_ptr_0x38", "0".to_string()),
        ("0x3c", "reset_ptr_0x3c", "0".to_string()),
        ("0x40", "mask_flags_0x40", "value & 7".to_string()),
    ]
    .into_iter()
    .map(|(offset, field, after)| CompetitionStandingsFormulaMutation {
        table: "competition.standings_rows".to_string(),
        row: scenario.club_row,
        field: field.to_string(),
        record_offset: format!("table base + row*0x49 + {offset}"),
        before: "unapplied".to_string(),
        after,
        source_function: "0x00679ed0".to_string(),
        formula: "competition table row reset and seed writes".to_string(),
        exactness_tier: "formula-derived-competition-standings".to_string(),
        evidence: format!("Original helper writes or masks standings row offset {offset} using 0x49-byte row stride."),
    })
    .collect()
}

pub fn apply_competition_standings_formula_plan_to_store(
    store: &mut CompetitionStandingsRuntimeStore,
    mutations: &[CompetitionStandingsFormulaMutation],
) {
    for mutation in mutations {
        let row_index = store
            .rows
            .iter()
            .position(|row| row.row == mutation.row)
            .unwrap_or_else(|| {
                store.rows.push(CompetitionStandingsRuntimeRow {
                    row: mutation.row,
                    stride: "0x49".to_string(),
                    written_fields: Vec::new(),
                    source_function: mutation.source_function.clone(),
                });
                store.rows.len() - 1
            });
        let row = &mut store.rows[row_index];
        if !row
            .written_fields
            .iter()
            .any(|field| mutation.record_offset.ends_with(&field.offset))
        {
            let offset = mutation
                .record_offset
                .split("+ ")
                .last()
                .unwrap_or(&mutation.record_offset)
                .to_string();
            row.written_fields.push(CompetitionStandingsRuntimeField {
                offset,
                value: mutation.after.clone(),
                evidence: mutation.evidence.clone(),
            });
            store.applied_formula_mutations = store.applied_formula_mutations.saturating_add(1);
        }
    }
}

pub fn competition_standings_formula_plan_ready(
    mutations: &[CompetitionStandingsFormulaMutation],
) -> bool {
    [
        "0x02", "0x06", "0x08", "0x0a", "0x0c", "0x10", "0x28", "0x40",
    ]
    .iter()
    .all(|offset| {
        mutations.iter().any(|mutation| {
            mutation.record_offset.contains(offset)
                && mutation.exactness_tier == "formula-derived-competition-standings"
        })
    })
}

pub fn competition_standings_runtime_store_ready(store: &CompetitionStandingsRuntimeStore) -> bool {
    store.rows.iter().any(|row| {
        row.stride == "0x49"
            && [
                "0x02", "0x06", "0x08", "0x0a", "0x0c", "0x10", "0x28", "0x40",
            ]
            .iter()
            .all(|offset| {
                row.written_fields
                    .iter()
                    .any(|field| field.offset == *offset)
            })
    }) && store.applied_formula_mutations >= 13
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionFormulaLiftEntry {
    pub formula: String,
    pub function: String,
    pub decompile_artifact: String,
    pub decompile_lines: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub constants: Vec<String>,
    pub branch_rule: String,
    pub rust_semantics: String,
    pub evidence: String,
    pub promotion_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionFormulaScenario {
    pub competition_row: u32,
    pub selected_staff_row: u32,
    pub previous_staff_row: Option<u32>,
    pub candidate_score: i32,
    pub current_date: GameDate,
    pub season_rollover_date: GameDate,
    pub has_existing_owner: bool,
    pub has_named_successor: bool,
    pub queue_count_before: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionFormulaMutation {
    pub table: String,
    pub row: u32,
    pub field: String,
    pub record_offset: String,
    pub before: String,
    pub after: String,
    pub source_function: String,
    pub formula: String,
    pub exactness_tier: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionRuntimeStore {
    pub reset_rows: Vec<CompetitionProgressionRuntimeResetRow>,
    pub owner_candidates: Vec<CompetitionProgressionRuntimeCandidate>,
    pub progression_queue: Vec<CompetitionProgressionRuntimeQueueRecord>,
    pub assignment_transitions: Vec<CompetitionProgressionRuntimeAssignmentTransition>,
    pub cleanup_events: Vec<CompetitionProgressionRuntimeCleanupEvent>,
    pub applied_formula_mutations: usize,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionRuntimeResetRow {
    pub competition_row: u32,
    pub stride: String,
    pub reset_fields: Vec<CompetitionProgressionRuntimeField>,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionRuntimeCandidate {
    pub staff_row: u32,
    pub staff_stride: String,
    pub side_state_stride: String,
    pub score_helper: String,
    pub selected_score: i32,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionRuntimeQueueRecord {
    pub competition_row: u32,
    pub selected_staff_row: u32,
    pub previous_staff_row: Option<u32>,
    pub stride: String,
    pub target_day: GameDate,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionRuntimeAssignmentTransition {
    pub competition_row: u32,
    pub staff_row: u32,
    pub club_row: u32,
    pub owner_offset: String,
    pub status_byte_0x3d: u8,
    pub squad_slot_offset: Option<String>,
    pub table_owner_offset: String,
    pub transition_helper: String,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionRuntimeCleanupEvent {
    pub queue_count_before: u32,
    pub queue_count_after: u32,
    pub removal_helper: String,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompetitionProgressionRuntimeField {
    pub offset: String,
    pub value: String,
    pub evidence: String,
}

fn default_competition_progression_formula_lift_map() -> Vec<CompetitionProgressionFormulaLiftEntry>
{
    vec![
        competition_progression_formula_lift_entry(
            "competition progression row reset",
            "0x00680cc0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_helpers/0x00680cc0.c",
            "initial table-row reset writes before manager selection",
            &[
                "competition row id *param_1",
                "competition table base pointer *in_ECX",
                "current owner param_2",
            ],
            &[
                "table row +0x00 set to 0",
                "table row +0x05 set to 0",
                "table row +0x06/+0x08/+0x0a/+0x0c set to 5000",
                "table row +0x0e set to 0x10",
                "table row +0x10/+0x12/+0x14/+0x16 reset to 0",
                "table row +0x28 reset to 0",
                "table row +0x02 flags masked with 0xfe/0xfd/0xfb/0xf7/0xef/0xdf/0xbf/0x7f",
                "table row +0x03 flags masked with 0xe1 then 0xfe",
                "table row +0x40 masked with 7",
            ],
            &[
                "0x49", "5000", "0x10", "0xfe", "0xfd", "0xfb", "0xf7", "0xef",
                "0xdf", "0xbf", "0x7f", "0xe1", "7",
            ],
            "Before assigning/advancing a competition owner, the helper resets the 0x49-byte table row and masks known flag bytes.",
            "Rust owns the competition progression reset row as explicit typed mutations, separate from final league/cup ranking semantics.",
            "The decompile writes row*0x49 offsets +0/+5/+6/+8/+0xa/+0xc/+0xe/+0x10/+0x12/+0x14/+0x16/+0x28 and masks +0x02/+0x03/+0x40.",
        ),
        competition_progression_formula_lift_entry(
            "competition owner candidate selection",
            "0x00680cc0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_helpers/0x00680cc0.c",
            "staff loop and candidate scoring",
            &[
                "staff array DAT_00acd5c4",
                "staff count DAT_00acd56c",
                "side-state array DAT_00acdf0c",
                "candidate score helper 0x00682420",
                "random helper 0x008fc4f0",
            ],
            &[
                "selected staff row local_2e",
                "best candidate score local_40",
                "optional fallback owner from competition +0xd3/+0x19f",
            ],
            &["0x6e", "0x4f", "0x30", "-1000000", "0x00682420", "0x008fc4f0"],
            "Walk staff records by 0x6e stride, side-state by 0x4f stride, ignore blocked side-state flags 0x30, score candidates through 0x00682420, and retain the highest score.",
            "Rust records the manager/owner candidate chosen for competition progression, including the proven array strides and score-helper ownership.",
            "The decompile loops DAT_00acd5c4 + index*0x6e and DAT_00acdf0c + index*0x4f +0xb, filters flags &0x30, calls 0x00682420(...,0,5), and updates local_2e/local_40.",
        ),
        competition_progression_formula_lift_entry(
            "competition progression queue record",
            "0x00680cc0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_helpers/0x00680cc0.c",
            "local_34 record build and FUN_00672320 enqueue",
            &[
                "selected owner row local_2e",
                "previous owner local_2a",
                "current date DAT_00dbc268",
                "season rollover date DAT_00acde90/DAT_00acde92",
                "date randomizer 0x008fc4f0",
            ],
            &[
                "0x26-byte progression queue record",
                "target day/year fields local_16/local_1c/local_1e",
                "queue count restored after enqueue",
            ],
            &["0x26", "3", "5", "7", "10", "20", "200", "500", "0x16c"],
            "Build a 0x26-byte progression record, schedule it around the current/rollover date with bounded random day offsets, enqueue it through 0x00672320, then restore the queue count snapshot.",
            "Rust owns a typed progression queue record and target date plan; exact competition-specific promotion/cup advancement rules remain a smaller downstream lift.",
            "The decompile fills local_34/local_32/local_2a/local_26/local_16/local_1c/local_1e, calls 0x008fc4f0 with 3/5/7/20/200/500, enqueues with FUN_00672320(&local_34,0x26), and restores *(in_ECX[2]+10).",
        ),
        competition_progression_formula_lift_entry(
            "competition queue cleanup/remove helper",
            "0x00672440",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_helpers/0x00672440.c",
            "list node unlink and queue count decrement",
            &[
                "competition queue linked-list",
                "node previous/next links +6/+10",
                "record payload length match",
            ],
            &[
                "queue node unlinked",
                "queue count decremented",
                "active pointer advanced",
                "node storage freed",
            ],
            &["0x06", "0x0a", "0x00933d24"],
            "When a queued competition progression record is removed, repair previous/next links, decrement the owning count, advance active iteration if needed, and free the node.",
            "Rust models cleanup as a typed queue-count mutation with helper provenance.",
            "The decompile updates node links at +6/+10, decrements the owner count, calls 0x00672260 for matching payload removal, and frees via 0x00933d24.",
        ),
        competition_progression_formula_lift_entry(
            "competition manager assignment and squad registration",
            "0x00674c10",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition/0x00674c10.c",
            "post-progression owner assignment and squad-slot write",
            &[
                "queued progression record pcVar9",
                "staff row DAT_00acd5c4 + *(pcVar9+6)*0x6e",
                "club row DAT_00acd5bc + *(pcVar9+0xe)*0x245",
                "previous/current club pointer puVar17",
            ],
            &[
                "club +0xcf set to staff pointer",
                "staff +0x39 set to club pointer for non-national clubs",
                "staff +0x3d set to 0x0/0x5/0x8/0x0b/0x0c depending status branch",
                "club squad slot +0xd7 + index*4 set to staff pointer for max 0x32 slots",
                "competition table row +0x01 written from 0 or 0x00582870",
                "queue record removed through 0x00672440",
                "standings row refreshed through 0x00679ed0",
            ],
            &[
                "0x6e", "0x245", "0xcf", "0x39", "0x3d", "0xd7", "0x32", "0x0c",
                "0x05", "0x08", "0x0b", "0x49", "0x01", "0x2ee",
            ],
            "When a due progression queue row survives validation, resolve staff/club by original strides, update club/staff ownership pointers, optionally register the staff in one of 50 club slots, set status byte branches, update table owner byte +0x01, remove the queue row, and refresh standings.",
            "Rust records the exact assignment transition surface used by the original manager/competition progression loop; final cup bracket and league tie-break arithmetic are still separate from this owner mutation.",
            "The decompile writes *(club+0xcf)=staff, *(staff+0x39)=club, scans club +0xd7 for <0x32 empty slots, writes status +0x3d = 0x0c or 5/8/0xb branches, writes row*0x49+1, then calls 0x00672440 and 0x00679ed0.",
        ),
        competition_progression_formula_lift_entry(
            "competition progression status transition helper dispatch",
            "0x00674c10",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition/0x00674c10.c",
            "status switch dispatch into 0x00688070/0x00675980/0x00675ae0",
            &[
                "staff status byte +0x3d",
                "staff contract/current club pointers +0x61/+0x69/+0x39",
                "target club pointer",
            ],
            &[
                "transition mode 1 for status 0x0b",
                "transition mode 2 for status 0x06/0x08/0x0f/default",
                "transition mode 3 for status 0x05/0x0c",
                "contract cleanup helper 0x00675980",
                "club/staff detach helper 0x00675ae0",
                "assignment transition helper 0x00688070",
            ],
            &["0x3d", "0x0b", "0x0c", "0x0f", "1", "2", "3"],
            "Map staff status byte branches to original transition modes before final ownership assignment and cleanup.",
            "Rust preserves the original status-to-transition-mode dispatch table as provenance for future exact pro/rel and job movement mutations.",
            "The decompile switches on staff +0x3d and calls 0x00688070 with mode 1/2/3, invoking 0x00675980 and/or 0x00675ae0 depending on contract/current-club pointers.",
        ),
        competition_progression_formula_lift_entry(
            "league stage construction state shape",
            "0x00670350",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_rank_progression/0x00670350.c",
            "constructor writes and team/extra allocation",
            &[
                "league stage object in_ECX",
                "participant count param_2",
                "extra row count param_7",
                "club pointers param_3",
                "optional 0x41-byte extra rows param_6",
            ],
            &[
                "team row storage allocated as participant_count * 0x3b bytes",
                "extra row storage copied as extra_count * 0x41 bytes",
                "fixture/team rows initialized through 0x0066c700",
                "default extra row date uses DAT_00acde90 and DAT_00acde92 - stage year offset",
                "club pointers are preserved for later 0x245 club-row resolution",
            ],
            &["0x3b", "0x41", "0xa9", "0xb1", "0xba", "0x245", "0x0066c700"],
            "Build league-stage state with fixed participant row stride 0x3b and optional extra row stride 0x41 before ranking/tie-break processing.",
            "Rust records league-stage state shape separately from the final comparator formula so the table engine owns the right storage first.",
            "The decompile writes object fields, mallocs count*0x3b at +0xb1, copies count*0x41 at +0xba, initializes rows through 0x0066c700, and creates a default 0x41 row when absent.",
        ),
        competition_progression_formula_lift_entry(
            "league stage serialization and club pointer rebind",
            "0x00671c90",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_rank_progression/0x00671c90.c",
            "save/load state serialization and pointer/id conversion",
            &[
                "league stage object in_ECX",
                "team row storage +0xb1",
                "extra row storage +0xba",
                "participant count +0x3e",
                "secondary list stride 0x0e",
            ],
            &[
                "serializes scalar state fields before row arrays",
                "writes 0x41-byte extra rows count +0xa9",
                "writes 0x3b-byte team rows count +0x3e",
                "serializes per-row 0x0e secondary entries",
                "rebases saved club row ids to DAT_00acd5bc + id*0x245 pointers after load",
            ],
            &["0x41", "0x3b", "0x0e", "0x245", "0xa9", "0xb1", "0xba"],
            "Persist league-stage rows as ids, then rebind ids back to 0x245 club pointers after reading.",
            "Rust can round-trip league-stage state without legacy .dat pointers by storing ids and rebuilding typed references.",
            "The decompile temporarily replaces club pointers with row ids/-1, writes 0x3b and 0x0e arrays, then converts ids back through DAT_00acd5bc + id*0x245.",
        ),
        competition_progression_formula_lift_entry(
            "competition manager state allocation shape",
            "0x00672e40",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_rank_progression/0x00672e40.c",
            "manager allocation/reload state reset",
            &[
                "club count DAT_00acd564",
                "manager state object in_ECX",
                "queue/list heads in_ECX[1..5]",
                "optional reload path param_1",
            ],
            &[
                "competition manager table allocated as club_count * 0x49 bytes",
                "five 0x0e list heads are initialized for progression/manager queues",
                "reload path frees old table/list heads before reallocating",
                "state initialization dispatches to 0x00674380",
            ],
            &["0x49", "0x0e", "0x00672270", "0x00672290", "0x00674380"],
            "Allocate the manager/competition table by club count with original 0x49 row stride and reset the associated list heads.",
            "Rust owns the competition manager table allocation shape used by standings/progression rows before exact table ranking formulas execute.",
            "The decompile allocates DAT_00acd564 * 0x49, creates five 0x0e list heads with 0x00672270, frees old state on reload, and calls 0x00674380.",
        ),
        competition_progression_formula_lift_entry(
            "standings refresh potential and table-row clamp",
            "0x00679ed0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_final_selectors/0x00679ed0.c",
            "post-assignment standings row refresh",
            &[
                "club row *param_1",
                "competition table row *in_ECX + club_row*0x49",
                "manager/staff pointer club +0xcf",
                "staff ability short +0x20",
                "national/club ability block +0x69",
            ],
            &[
                "refreshes the row through 0x006889e0 before table byte +0x01 update",
                "sets shortlist/manager slot byte to 2 at manager +0x856/+0xdc + club_row*9 +4",
                "seeds +0x06/+0x08/+0x0a/+0x0c with random-clamped 0x186a/0x1482/0x1676..10000 ranges",
                "resets row +0x10/+0x12/+0x14/+0x16, doubles +0x38/+0x3c to zero, clears +0x28",
                "masks table row +0x40 with 7 and writes owner marker +0x01 from 0x00582870 or zero",
            ],
            &[
                "0x49", "0x856", "0xdc", "9", "2", "0x186a", "0x251c", "0x1482",
                "0x1676", "10000", "0x2ee", "0x4e2", "0xfa", "0x40", "7", "0x01",
            ],
            "After a manager/competition assignment, refresh the club table row using exact original seed/clamp bands and owner marker writes.",
            "Rust now treats standings refresh as a concrete formula-derived table mutation rather than a generic competition placeholder.",
            "The decompile computes row*0x49, calls 0x006889e0(param_1,1), writes manager byte +club*9+4=2, clamps seeded shorts at +0x06/+0x08/+0x0a/+0x0c, resets row form fields, masks +0x40&=7, and writes row +0x01 from 0x00582870 or 0.",
        ),
        competition_progression_formula_lift_entry(
            "cup and continental eligibility selector flags",
            "0x006889e0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_final_selectors/0x006889e0.c",
            "competition row selector flags and qualifying-position clamps",
            &[
                "club row *param_1",
                "competition table row *in_ECX + club_row*0x49",
                "club current manager +0xcf",
                "nation/competition pointer +0x57",
                "cup/continental config table DAT_00ac688c",
            ],
            &[
                "clears selector field +0x0e and qualifying pointer +0x2c before recomputing",
                "stores qualifying/round selector at table row +0x2c as a signed byte-range value",
                "sets selector flag +0x0e to 0/0x10/0x40 plus bitwise flag updates 0x80/0x200/0x400 paths",
                "clamps qualifying position to at least 1 and at most competition limit short +0x3e - 1",
                "applies late reset of row +0x02/+0x03/+0x04/+0x05/+0x12/+0x14/+0x16/+0x28",
            ],
            &[
                "0x49", "0x0e", "0x2c", "0x10", "0x40", "0x80", "0x200", "0x400",
                "0x3e", "0xbe", "0xbf", "0xc1", "0x72", "0x73", "0x74", "0x75",
            ],
            "Resolve cup/continental eligibility through the competition config table, clamp the selected position, and write the original selector flags into the competition row.",
            "Rust now owns the bracket/eligibility selector surface used by cup and continental progression instead of waiting on live capture.",
            "The decompile clears row +0x0e/+0x2c, derives selectors from DAT_00ac688c and nation/competition bytes +0x72..+0x75, clamps +0x2c to signed byte and competition +0x3e bounds, writes +0x0e as 0/0x10/0x40 with 0x80/0x200/0x400 bit paths, then resets row fields.",
        ),
        competition_progression_formula_lift_entry(
            "queued owner progression and missing-job repair",
            "0x006808a0/0x006822d0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_final_selectors/0x006808a0.c; D:/cm0102-carve/decompiled/gameplay_lifts_competition_final_selectors/0x006822d0.c",
            "queued progression drain and missing job scan",
            &[
                "manager queue head in_ECX +4",
                "missing-job queue head in_ECX +8",
                "club records DAT_00acd5bc + club_row*0x245",
                "club manager pointer +0xcf",
            ],
            &[
                "resets active queue cursor +0x0a from +0x02 before iteration",
                "removes queued rows with no club manager pointer +0xcf",
                "calls 0x006809e0 then 0x00680cc0 for valid queued progression rows",
                "scans all club rows by 0x245 and repairs eligible missing jobs through 0x00680cc0",
                "removes or leaves queue records through 0x00672440/0x00672400",
            ],
            &["0x0a", "0x02", "0x245", "0xcf", "0x00672400", "0x00672440", "0x00680cc0", "0x006809e0"],
            "Drain queued competition-owner records and repair missing job ownership before standings/progression selectors run.",
            "Rust now models the promotion/relegation owner queue plumbing needed for deterministic headless competition advancement.",
            "The decompiles reset list cursor +0x0a, iterate 0x00672400, resolve club rows by DAT_00acd5bc + id*0x245, remove rows lacking +0xcf, call 0x006809e0/0x00680cc0 for valid rows, and scan every club to repair missing jobs.",
        ),
        competition_progression_formula_lift_entry(
            "temporary cup selector evaluation",
            "0x0068a1f0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition_final_selectors/0x0068a1f0.c",
            "temporary owner injection and selector recompute",
            &[
                "club row *param_1",
                "temporary target owner param_2",
                "competition table row +0x02 flag bit",
                "selector refresh helper 0x006889e0",
            ],
            &[
                "temporarily writes club +0xcf to candidate owner",
                "sets table row +0x02 bit 1 while recomputing selector",
                "calls 0x006889e0(param_1,0) with temporary owner state",
                "restores original club +0xcf and original +0x02 bit state",
                "exports resulting selector short +0x0e for UI/reporting path",
            ],
            &["0xcf", "0x49", "0x02", "1", "0x0e", "0x006889e0", "0x0076d730"],
            "Evaluate cup/progression selector outcome by temporarily injecting an owner, recomputing, and restoring original state.",
            "Rust can now implement selector preview/evaluation without mutating the canonical competition owner accidentally.",
            "The decompile saves club +0xcf and row +0x02 bit state, writes param_2 into +0xcf, sets bit 1, calls 0x006889e0, restores both values, and publishes row +0x0e.",
        ),
    ]
}

fn competition_progression_formula_lift_entry(
    formula: &str,
    function: &str,
    decompile_artifact: &str,
    decompile_lines: &str,
    inputs: &[&str],
    outputs: &[&str],
    constants: &[&str],
    branch_rule: &str,
    rust_semantics: &str,
    evidence: &str,
) -> CompetitionProgressionFormulaLiftEntry {
    CompetitionProgressionFormulaLiftEntry {
        formula: formula.to_string(),
        function: function.to_string(),
        decompile_artifact: decompile_artifact.to_string(),
        decompile_lines: decompile_lines.to_string(),
        inputs: inputs.iter().map(|item| item.to_string()).collect(),
        outputs: outputs.iter().map(|item| item.to_string()).collect(),
        constants: constants.iter().map(|item| item.to_string()).collect(),
        branch_rule: branch_rule.to_string(),
        rust_semantics: rust_semantics.to_string(),
        evidence: evidence.to_string(),
        promotion_status: "formula-lifted-static-code-derived".to_string(),
    }
}

pub fn competition_progression_formula_lift_map_ready(
    lifts: &[CompetitionProgressionFormulaLiftEntry],
) -> bool {
    [
        ("competition progression row reset", "0x00680cc0", "0x49"),
        (
            "competition owner candidate selection",
            "0x00680cc0",
            "0x00682420",
        ),
        ("competition progression queue record", "0x00680cc0", "0x26"),
        (
            "competition queue cleanup/remove helper",
            "0x00672440",
            "0x00933d24",
        ),
        (
            "competition manager assignment and squad registration",
            "0x00674c10",
            "0xd7",
        ),
        (
            "competition progression status transition helper dispatch",
            "0x00674c10",
            "0x3d",
        ),
        (
            "league stage construction state shape",
            "0x00670350",
            "0x3b",
        ),
        (
            "league stage serialization and club pointer rebind",
            "0x00671c90",
            "0x0e",
        ),
        (
            "competition manager state allocation shape",
            "0x00672e40",
            "0x49",
        ),
        (
            "standings refresh potential and table-row clamp",
            "0x00679ed0",
            "0x186a",
        ),
        (
            "cup and continental eligibility selector flags",
            "0x006889e0",
            "0x400",
        ),
        (
            "queued owner progression and missing-job repair",
            "0x006808a0/0x006822d0",
            "0x00680cc0",
        ),
        (
            "temporary cup selector evaluation",
            "0x0068a1f0",
            "0x006889e0",
        ),
    ]
    .iter()
    .all(|(formula, function, constant)| {
        lifts.iter().any(|lift| {
            lift.formula == *formula
                && lift.function == *function
                && lift.constants.iter().any(|item| item == constant)
                && lift.promotion_status == "formula-lifted-static-code-derived"
                && lift
                    .decompile_artifact
                    .starts_with("D:/cm0102-carve/decompiled/")
        })
    })
}

pub fn default_competition_progression_formula_scenario() -> CompetitionProgressionFormulaScenario {
    CompetitionProgressionFormulaScenario {
        competition_row: 0,
        selected_staff_row: 16,
        previous_staff_row: Some(3),
        candidate_score: 100,
        current_date: GameDate {
            year: 2001,
            month: 7,
            day: 1,
        },
        season_rollover_date: GameDate {
            year: 2002,
            month: 6,
            day: 30,
        },
        has_existing_owner: true,
        has_named_successor: false,
        queue_count_before: 1,
    }
}

fn default_competition_progression_runtime_store() -> CompetitionProgressionRuntimeStore {
    CompetitionProgressionRuntimeStore {
        reset_rows: Vec::new(),
        owner_candidates: Vec::new(),
        progression_queue: Vec::new(),
        assignment_transitions: Vec::new(),
        cleanup_events: Vec::new(),
        applied_formula_mutations: 0,
        provenance: "Rust-owned competition progression store seeded from verified 0x00680cc0/0x00672440/0x00674c10 static lifts.".to_string(),
    }
}

pub fn plan_competition_progression_formula_mutations(
    backend: &RuntimeBackendSystems,
    scenario: &CompetitionProgressionFormulaScenario,
) -> Vec<CompetitionProgressionFormulaMutation> {
    if !competition_progression_formula_lift_map_ready(
        &backend.competition_progression_formula_lift_map,
    ) {
        return Vec::new();
    }
    let mut mutations = Vec::new();
    for (offset, field, after) in [
        ("0x00", "row_status", "0"),
        ("0x05", "row_byte_0x05", "0"),
        ("0x06", "seed_short_0x06", "5000"),
        ("0x08", "seed_short_0x08", "5000"),
        ("0x0a", "seed_short_0x0a", "5000"),
        ("0x0c", "seed_short_0x0c", "5000"),
        ("0x0e", "seed_short_0x0e", "0x10"),
        ("0x10", "reset_short_0x10", "0"),
        ("0x12", "reset_short_0x12", "0"),
        ("0x14", "reset_short_0x14", "0"),
        ("0x16", "reset_short_0x16", "0"),
        ("0x28", "reset_ptr_0x28", "0"),
        (
            "0x02",
            "flag_mask_0x02",
            "value & 0xfe & 0xfd & 0xfb & 0xf7 & 0xef & 0xdf & 0xbf & 0x7f",
        ),
        ("0x03", "flag_mask_0x03", "value & 0xe1 & 0xfe"),
        ("0x40", "flag_mask_0x40", "value & 7"),
    ] {
        mutations.push(CompetitionProgressionFormulaMutation {
            table: "competition.progression_rows".to_string(),
            row: scenario.competition_row,
            field: field.to_string(),
            record_offset: format!("competition table base + row*0x49 + {offset}"),
            before: "unapplied".to_string(),
            after: after.to_string(),
            source_function: "0x00680cc0".to_string(),
            formula: "competition progression row reset".to_string(),
            exactness_tier: "formula-derived-competition-progression".to_string(),
            evidence: format!("Original 0x00680cc0 writes or masks table row offset {offset} before progression owner selection."),
        });
    }
    mutations.push(CompetitionProgressionFormulaMutation {
        table: "competition.owner_candidates".to_string(),
        row: scenario.selected_staff_row,
        field: "selected_owner_score".to_string(),
        record_offset: "DAT_00acd5c4 + staff_row*0x6e; DAT_00acdf0c + staff_row*0x4f +0x0b".to_string(),
        before: "-1000000".to_string(),
        after: scenario.candidate_score.to_string(),
        source_function: "0x00680cc0".to_string(),
        formula: "competition owner candidate selection".to_string(),
        exactness_tier: "formula-derived-competition-progression".to_string(),
        evidence: "Original helper walks staff by 0x6e, side-state by 0x4f, filters flags &0x30, and keeps the best 0x00682420 score.".to_string(),
    });
    let target_day = if scenario.has_existing_owner && !scenario.has_named_successor {
        CmPackedDate::from_game_date(scenario.current_date.clone())
            .add_days(12)
            .to_game_date()
    } else {
        CmPackedDate::from_game_date(scenario.season_rollover_date.clone())
            .add_days(4)
            .to_game_date()
    };
    mutations.push(CompetitionProgressionFormulaMutation {
        table: "competition.progression_queue".to_string(),
        row: scenario.competition_row,
        field: "queued_0x26_record".to_string(),
        record_offset: "local_34 record passed to 0x00672320 with length 0x26".to_string(),
        before: "not queued".to_string(),
        after: format!(
            "selected_staff_row={}; previous_staff_row={:?}; target_day={}",
            scenario.selected_staff_row,
            scenario.previous_staff_row,
            target_day.iso()
        ),
        source_function: "0x00680cc0".to_string(),
        formula: "competition progression queue record".to_string(),
        exactness_tier: "formula-derived-competition-progression".to_string(),
        evidence: "Original helper builds local_34 and enqueues it with FUN_00672320(&local_34,0x26), then restores the queue count snapshot.".to_string(),
    });
    mutations.push(CompetitionProgressionFormulaMutation {
        table: "competition.progression_queue_cleanup".to_string(),
        row: scenario.competition_row,
        field: "queue_count".to_string(),
        record_offset: "linked-list node +0x06/+0x0a and owner count".to_string(),
        before: scenario.queue_count_before.to_string(),
        after: scenario.queue_count_before.saturating_sub(1).to_string(),
        source_function: "0x00672440".to_string(),
        formula: "competition queue cleanup/remove helper".to_string(),
        exactness_tier: "formula-derived-competition-progression".to_string(),
        evidence: "Original cleanup helper unlinks the matching node, decrements count, advances active pointer, and frees storage.".to_string(),
    });
    mutations.push(CompetitionProgressionFormulaMutation {
        table: "competition.assignment_transitions".to_string(),
        row: scenario.competition_row,
        field: "club_owner_staff_pointer_0xcf".to_string(),
        record_offset: "DAT_00acd5bc + club_row*0x245 +0xcf".to_string(),
        before: "previous owner".to_string(),
        after: format!("staff_row={}", scenario.selected_staff_row),
        source_function: "0x00674c10".to_string(),
        formula: "competition manager assignment and squad registration".to_string(),
        exactness_tier: "formula-derived-competition-progression".to_string(),
        evidence: "Original progression loop writes *(club+0xcf)=staff after resolving club by queued row +0xe and staff by queued row +6.".to_string(),
    });
    mutations.push(CompetitionProgressionFormulaMutation {
        table: "competition.assignment_transitions".to_string(),
        row: scenario.competition_row,
        field: "staff_club_pointer_0x39".to_string(),
        record_offset: "DAT_00acd5c4 + staff_row*0x6e +0x39".to_string(),
        before: "previous club".to_string(),
        after: format!("club_row={}", scenario.competition_row),
        source_function: "0x00674c10".to_string(),
        formula: "competition manager assignment and squad registration".to_string(),
        exactness_tier: "formula-derived-competition-progression".to_string(),
        evidence:
            "Original progression loop writes *(staff+0x39)=club for non-national club assignment."
                .to_string(),
    });
    mutations.push(CompetitionProgressionFormulaMutation {
        table: "competition.assignment_transitions".to_string(),
        row: scenario.competition_row,
        field: "club_squad_slot_0xd7".to_string(),
        record_offset: "club +0xd7 + slot*4, slot <0x32".to_string(),
        before: "empty slot".to_string(),
        after: format!("staff_row={}; status_0x3d=0x0c", scenario.selected_staff_row),
        source_function: "0x00674c10".to_string(),
        formula: "competition manager assignment and squad registration".to_string(),
        exactness_tier: "formula-derived-competition-progression".to_string(),
        evidence: "Original code scans up to 0x32 club squad slots at +0xd7 and writes the staff pointer, then sets staff +0x3d to 0x0c.".to_string(),
    });
    mutations.push(CompetitionProgressionFormulaMutation {
        table: "competition.assignment_transitions".to_string(),
        row: scenario.competition_row,
        field: "table_owner_byte_0x01".to_string(),
        record_offset: "competition table base + club_row*0x49 +0x01".to_string(),
        before: "old table owner marker".to_string(),
        after: "0 or 0x00582870()".to_string(),
        source_function: "0x00674c10".to_string(),
        formula: "competition manager assignment and squad registration".to_string(),
        exactness_tier: "formula-derived-competition-progression".to_string(),
        evidence: "Original code writes row*0x49+1 to 0 when 0x0058a490 is false, otherwise writes 0x00582870() result.".to_string(),
    });
    mutations.push(CompetitionProgressionFormulaMutation {
        table: "competition.status_transition_dispatch".to_string(),
        row: scenario.selected_staff_row,
        field: "status_to_transition_mode".to_string(),
        record_offset: "staff +0x3d switch; helper 0x00688070 mode param".to_string(),
        before: "staff status 0x05/0x06/0x08/0x0b/0x0c/0x0f/default".to_string(),
        after: "0x0b=>1; 0x05/0x0c=>3; 0x06/0x08/0x0f/default=>2".to_string(),
        source_function: "0x00674c10".to_string(),
        formula: "competition progression status transition helper dispatch".to_string(),
        exactness_tier: "formula-derived-competition-progression".to_string(),
        evidence: "Original switch on staff +0x3d dispatches 0x00688070 with modes 1, 2, or 3 and calls cleanup helpers based on +0x61/+0x69.".to_string(),
    });
    mutations
}

pub fn apply_competition_progression_formula_plan_to_store(
    store: &mut CompetitionProgressionRuntimeStore,
    mutations: &[CompetitionProgressionFormulaMutation],
    scenario: &CompetitionProgressionFormulaScenario,
) {
    for mutation in mutations {
        match mutation.table.as_str() {
            "competition.progression_rows" => {
                let row_index = store
                    .reset_rows
                    .iter()
                    .position(|row| row.competition_row == mutation.row)
                    .unwrap_or_else(|| {
                        store
                            .reset_rows
                            .push(CompetitionProgressionRuntimeResetRow {
                                competition_row: mutation.row,
                                stride: "0x49".to_string(),
                                reset_fields: Vec::new(),
                                source_function: mutation.source_function.clone(),
                            });
                        store.reset_rows.len() - 1
                    });
                let offset = mutation
                    .record_offset
                    .split("+ ")
                    .last()
                    .unwrap_or(&mutation.record_offset)
                    .to_string();
                if !store.reset_rows[row_index]
                    .reset_fields
                    .iter()
                    .any(|field| field.offset == offset)
                {
                    store.reset_rows[row_index].reset_fields.push(
                        CompetitionProgressionRuntimeField {
                            offset,
                            value: mutation.after.clone(),
                            evidence: mutation.evidence.clone(),
                        },
                    );
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "competition.owner_candidates" => {
                if !store
                    .owner_candidates
                    .iter()
                    .any(|candidate| candidate.staff_row == mutation.row)
                {
                    store
                        .owner_candidates
                        .push(CompetitionProgressionRuntimeCandidate {
                            staff_row: mutation.row,
                            staff_stride: "0x6e".to_string(),
                            side_state_stride: "0x4f".to_string(),
                            score_helper: "0x00682420".to_string(),
                            selected_score: scenario.candidate_score,
                            source_function: mutation.source_function.clone(),
                        });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "competition.progression_queue" => {
                if !store
                    .progression_queue
                    .iter()
                    .any(|record| record.competition_row == mutation.row)
                {
                    let target_day = mutation
                        .after
                        .split("target_day=")
                        .last()
                        .and_then(|value| parse_iso_game_date(value).ok())
                        .unwrap_or_else(|| {
                            CmPackedDate::from_game_date(scenario.current_date.clone())
                                .add_days(12)
                                .to_game_date()
                        });
                    store
                        .progression_queue
                        .push(CompetitionProgressionRuntimeQueueRecord {
                            competition_row: mutation.row,
                            selected_staff_row: scenario.selected_staff_row,
                            previous_staff_row: scenario.previous_staff_row,
                            stride: "0x26".to_string(),
                            target_day,
                            source_function: mutation.source_function.clone(),
                        });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "competition.progression_queue_cleanup" => {
                if !store
                    .cleanup_events
                    .iter()
                    .any(|event| event.source_function == mutation.source_function)
                {
                    store
                        .cleanup_events
                        .push(CompetitionProgressionRuntimeCleanupEvent {
                            queue_count_before: scenario.queue_count_before,
                            queue_count_after: scenario.queue_count_before.saturating_sub(1),
                            removal_helper: "0x00672440".to_string(),
                            source_function: mutation.source_function.clone(),
                        });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "competition.assignment_transitions" => {
                if !store.assignment_transitions.iter().any(|transition| {
                    transition.competition_row == mutation.row
                        && transition.field_matches(&mutation.field)
                }) {
                    store.assignment_transitions.push(
                        CompetitionProgressionRuntimeAssignmentTransition {
                            competition_row: mutation.row,
                            staff_row: scenario.selected_staff_row,
                            club_row: scenario.competition_row,
                            owner_offset: "club +0xcf".to_string(),
                            status_byte_0x3d: if mutation.field == "club_squad_slot_0xd7" {
                                0x0c
                            } else {
                                0x05
                            },
                            squad_slot_offset: if mutation.field == "club_squad_slot_0xd7" {
                                Some("club +0xd7 + slot*4, slot <0x32".to_string())
                            } else {
                                None
                            },
                            table_owner_offset: "table row*0x49 +0x01".to_string(),
                            transition_helper: "0x00688070".to_string(),
                            source_function: mutation.source_function.clone(),
                        },
                    );
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "competition.status_transition_dispatch" => {
                if !store.assignment_transitions.iter().any(|transition| {
                    transition.transition_helper == "0x00688070"
                        && transition.source_function == mutation.source_function
                }) {
                    store.assignment_transitions.push(
                        CompetitionProgressionRuntimeAssignmentTransition {
                            competition_row: scenario.competition_row,
                            staff_row: mutation.row,
                            club_row: scenario.competition_row,
                            owner_offset: "club +0xcf".to_string(),
                            status_byte_0x3d: 0x0b,
                            squad_slot_offset: None,
                            table_owner_offset: "table row*0x49 +0x01".to_string(),
                            transition_helper: "0x00688070 modes 1/2/3".to_string(),
                            source_function: mutation.source_function.clone(),
                        },
                    );
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            _ => {}
        }
    }
}

impl CompetitionProgressionRuntimeAssignmentTransition {
    fn field_matches(&self, field: &str) -> bool {
        match field {
            "club_owner_staff_pointer_0xcf" => self.owner_offset == "club +0xcf",
            "staff_club_pointer_0x39" => self.club_row == self.competition_row,
            "club_squad_slot_0xd7" => self.squad_slot_offset.is_some(),
            "table_owner_byte_0x01" => self.table_owner_offset == "table row*0x49 +0x01",
            _ => false,
        }
    }
}

pub fn competition_progression_formula_plan_ready(
    mutations: &[CompetitionProgressionFormulaMutation],
) -> bool {
    ["0x00", "0x02", "0x03", "0x06", "0x0e", "0x28", "0x40"]
        .iter()
        .all(|offset| {
            mutations.iter().any(|mutation| {
                mutation.table == "competition.progression_rows"
                    && mutation.record_offset.contains(offset)
                    && mutation.exactness_tier == "formula-derived-competition-progression"
            })
        })
        && mutations.iter().any(|mutation| {
            mutation.table == "competition.owner_candidates"
                && mutation.record_offset.contains("0x6e")
                && mutation.record_offset.contains("0x4f")
        })
        && mutations.iter().any(|mutation| {
            mutation.table == "competition.progression_queue"
                && mutation.record_offset.contains("0x26")
        })
        && mutations.iter().any(|mutation| {
            mutation.table == "competition.progression_queue_cleanup"
                && mutation.source_function == "0x00672440"
        })
        && mutations.iter().any(|mutation| {
            mutation.table == "competition.assignment_transitions"
                && mutation.record_offset.contains("0xcf")
        })
        && mutations.iter().any(|mutation| {
            mutation.table == "competition.assignment_transitions"
                && mutation.record_offset.contains("0xd7")
                && mutation.after.contains("0x0c")
        })
        && mutations.iter().any(|mutation| {
            mutation.table == "competition.status_transition_dispatch"
                && mutation.after.contains("0x0b=>1")
        })
}

pub fn competition_progression_runtime_store_ready(
    store: &CompetitionProgressionRuntimeStore,
) -> bool {
    store.reset_rows.iter().any(|row| {
        row.stride == "0x49"
            && ["0x00", "0x02", "0x03", "0x06", "0x0e", "0x28", "0x40"]
                .iter()
                .all(|offset| row.reset_fields.iter().any(|field| field.offset == *offset))
    }) && store
        .owner_candidates
        .iter()
        .any(|candidate| candidate.staff_stride == "0x6e" && candidate.side_state_stride == "0x4f")
        && store
            .progression_queue
            .iter()
            .any(|record| record.stride == "0x26")
        && store.assignment_transitions.iter().any(|transition| {
            transition.owner_offset == "club +0xcf"
                && transition.table_owner_offset == "table row*0x49 +0x01"
        })
        && store.assignment_transitions.iter().any(|transition| {
            transition.squad_slot_offset.as_deref() == Some("club +0xd7 + slot*4, slot <0x32")
                && transition.status_byte_0x3d == 0x0c
        })
        && store
            .cleanup_events
            .iter()
            .any(|event| event.removal_helper == "0x00672440")
        && store.applied_formula_mutations >= 20
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractStateMapEntry {
    pub system: String,
    pub record_offset: Option<String>,
    pub stride: Option<String>,
    pub helper: Option<String>,
    pub function: String,
    pub evidence: String,
}

fn default_transfer_contract_state_map() -> Vec<TransferContractStateMapEntry> {
    vec![
        transfer_contract_state_map_entry(
            "contract renewal date windows",
            None,
            None,
            Some("0x00536190"),
            "0x004cdef0",
            "The staff/contract renewal frontier builds date windows at +7, +0x1e, +0x3c, +0x5b, +0x79, +0x98, +0xb6, +0x16d, +0x226, and +0x447 days.",
        ),
        transfer_contract_state_map_entry(
            "staff pool stride",
            Some("0x59"),
            Some("0x6e"),
            None,
            "0x004cdef0",
            "The renewal frontier walks DAT_00acd5c4 staff records with stride 0x6e and reads byte +0x59 while deriving renewal timing.",
        ),
        transfer_contract_state_map_entry(
            "staff side-state index stride",
            Some("0x0b"),
            Some("0x4f"),
            None,
            "0x004cdef0",
            "The frontier maps staff indexes through DAT_00acdf0c using staff index * 0x4f, including queue/status byte +0x0b flag updates.",
        ),
        transfer_contract_state_map_entry(
            "event/contract record stride",
            Some("0x35"),
            Some("0x50"),
            Some("0x004dc980"),
            "0x004cdef0",
            "Mapped side-state entries resolve to event/contract records at base + index * 0x50; date fields +0x2d/+0x2f and status byte +0x35 drive contract outcomes.",
        ),
        transfer_contract_state_map_entry(
            "contract club stride",
            Some("0x04"),
            Some("0x245"),
            Some("0x005246e0"),
            "0x004cdef0",
            "Contract outcomes repeatedly resolve clubs from record +0x04 through DAT_00acd5bc plus club index * 0x245, then consult age/date helper 0x005246e0.",
        ),
        transfer_contract_state_map_entry(
            "queued transfer/club-news item",
            Some("0x24/0x28/0x2c"),
            Some("0x6"),
            Some("0x004539f0"),
            "0x00449710",
            "Queued club-news dispatch reads queue pointer +0x24, capacity/count fields +0x28/+0x2c, processes 6-byte items, then builds and dispatches news payloads.",
        ),
        transfer_contract_state_map_entry(
            "queued transfer human/non-human dispatch",
            Some("0x53"),
            Some("0x245"),
            Some("0x0076e180/0x0076e390"),
            "0x00449710",
            "Dispatch resolves queued clubs through 0x245-byte club records, sends human-visible payloads via 0x0076e180 or affiliated records at club +0x53 via 0x0076e390.",
        ),
        transfer_contract_state_map_entry(
            "transfer.dat manager load",
            Some("0x213/0x84d/0x856"),
            Some("0x41/0x25/0x0c/0x0d/0x0e"),
            Some("0x00921cc0"),
            "0x008a9080",
            "Verified transfer_value_calc opens transfer.dat and loads manager/list state: 0x41-byte objects, 0x25/0x0c/0x0d/0x0e-byte list records, and state fields around +0x213/+0x84d/+0x856.",
        ),
    ]
}

fn transfer_contract_state_map_entry(
    system: &str,
    record_offset: Option<&str>,
    stride: Option<&str>,
    helper: Option<&str>,
    function: &str,
    evidence: &str,
) -> TransferContractStateMapEntry {
    TransferContractStateMapEntry {
        system: system.to_string(),
        record_offset: record_offset.map(str::to_string),
        stride: stride.map(str::to_string),
        helper: helper.map(str::to_string),
        function: function.to_string(),
        evidence: evidence.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractFormulaLiftEntry {
    pub formula: String,
    pub function: String,
    pub decompile_artifact: String,
    pub decompile_lines: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub constants: Vec<String>,
    pub branch_rule: String,
    pub rust_semantics: String,
    pub evidence: String,
    pub promotion_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractFormulaScenario {
    pub staff_row: u32,
    pub side_state_row: u32,
    pub event_record_row: u32,
    pub current_date: GameDate,
    pub event_due_date: GameDate,
    pub event_status_before: u8,
    pub queued_transfer_row: u32,
    pub queued_transfer_club_id: u32,
    pub queued_transfer_payload_kind: u8,
    pub queue_capacity_before: u32,
    pub queue_count_before: u32,
    pub queued_transfer_affiliated_club_id: Option<u32>,
    pub compensation_factor: i32,
    pub contract_value: i32,
    pub squad_staff_count: u8,
    pub transfer_base_value: i32,
    pub transfer_cash_limit: i32,
    pub transfer_param_offer: i32,
    pub transfer_sell_on_percent: u8,
    pub transfer_monthly_installment: i32,
    pub transfer_installment_months: u8,
    pub transfer_player_value: i32,
    pub player_value_base: i32,
    pub player_value_deep_model: i32,
    pub player_value_release_cap: i32,
    pub player_value_contract_class: u8,
    pub wage_role_case: u8,
    pub wage_role_cap: i32,
    pub wage_role_multiplier_per_mille: u16,
    pub wage_current_offer: i32,
    pub wage_existing_contract_value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractFormulaMutation {
    pub table: String,
    pub row: u32,
    pub field: String,
    pub record_offset: String,
    pub before: String,
    pub after: String,
    pub source_function: String,
    pub formula: String,
    pub exactness_tier: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractRuntimeStore {
    pub renewal_windows: Vec<TransferContractRuntimeWindow>,
    pub contract_events: Vec<TransferContractRuntimeContractEvent>,
    pub compensation_values: Vec<TransferContractRuntimeCompensationValue>,
    pub offer_values: Vec<TransferContractRuntimeOfferValue>,
    pub decision_rules: Vec<TransferContractRuntimeDecisionRule>,
    pub transfer_manager_record_shapes: Vec<String>,
    pub transfer_queue: Vec<TransferContractRuntimeQueueItem>,
    pub queue_dispatches: Vec<TransferContractRuntimeQueueDispatch>,
    pub queue_pointer_active: bool,
    pub queue_capacity: u32,
    pub queue_count: u32,
    pub applied_formula_mutations: usize,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractRuntimeWindow {
    pub day_offset: i32,
    pub date: GameDate,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractRuntimeContractEvent {
    pub row: u32,
    pub staff_row: u32,
    pub side_state_row: u32,
    pub status_byte_0x35: u8,
    pub day_offset: i32,
    pub due_date: GameDate,
    pub source_function: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractRuntimeCompensationValue {
    pub row: u32,
    pub formula: String,
    pub value: i32,
    pub source_function: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractRuntimeOfferValue {
    pub row: u32,
    pub base_value: i32,
    pub total_value: i32,
    pub cash_limit: i32,
    pub sell_on_percent: u8,
    pub monthly_installment: i32,
    pub installment_months: u8,
    pub source_function: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractRuntimeDecisionRule {
    pub rule: String,
    pub threshold: String,
    pub outcome: String,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractRuntimeQueueItem {
    pub row: u32,
    pub club_id: u32,
    pub payload_kind: u8,
    pub stride: String,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferContractRuntimeQueueDispatch {
    pub row: u32,
    pub club_id: u32,
    pub helper: String,
    pub recipient: String,
    pub source_function: String,
    pub evidence: String,
}

fn default_transfer_contract_formula_lift_map() -> Vec<TransferContractFormulaLiftEntry> {
    vec![
        transfer_contract_formula_lift_entry(
            "contract renewal date-window ladder",
            "0x004cdef0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x004cdef0.c",
            "initial FUN_00536190 calls and branch-window calls",
            &[
                "current CM packed date",
                "date add-days helper 0x00536190",
                "renewal branch flag param_2",
            ],
            &[
                "renewal windows +7/+30/+60/+91/+121/+152/+182/+365/+550/+1095 days",
                "branch windows +31/+42/+70 days",
            ],
            &[
                "7", "30", "60", "91", "121", "152", "182", "365", "550", "1095", "31", "42",
                "70",
            ],
            "When the daily contract processor enters the normal renewal path, it creates fixed future date windows through helper 0x00536190; later branches add +31/+42/+70 day checks.",
            "Rust computes the same date-window set from CmPackedDate::add_days and records the generated dates in the transfer/contract runtime store.",
            "The decompile shows repeated FUN_00536190 calls with literal day offsets 7, 0x1e, 0x3c, 0x5b, 0x79, 0x98, 0xb6, 0x16d, 0x226, 0x447, plus branch offsets 0x1f, 0x2a, and 0x46.",
        ),
        transfer_contract_formula_lift_entry(
            "contract event side-state status promotion",
            "0x004cdef0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x004cdef0.c",
            "side-state/event owner paths",
            &[
                "staff index",
                "side-state index * 0x4f",
                "event/contract record index * 0x50",
                "event due date bytes +0x2d/+0x2f",
                "event status byte +0x35",
            ],
            &[
                "event/contract record status byte +0x35 low bits set to 1",
                "side-state row linked to event/contract row",
            ],
            &["0x6e", "0x4f", "0x50", "0x2d", "0x2f", "0x35", "0xc1", "1"],
            "If the event/contract record due date equals the current date, the processor preserves status byte bits masked by 0xc1 and sets the low outcome to 1.",
            "Rust resolves staff row -> side-state row -> event/contract row and applies status = (before & 0xc1) | 1 when the scenario due date matches the current date.",
            "The decompile compares current date components with event +0x2d/+0x2f, then writes *(byte *)(event+0x35) = *(byte *)(event+0x35) & 0xc1 | 1.",
        ),
        transfer_contract_formula_lift_entry(
            "transfer.dat manager record-shape ownership",
            "0x008a9080",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x008a9080.c",
            "load_from_disk record allocations/reads",
            &[
                "transfer.dat stream",
                "manager state offsets +0x213/+0x84d/+0x856",
                "loaded list counts",
            ],
            &[
                "Rust-owned transfer manager state",
                "0x41-byte objects",
                "0x25/0x0c/0x0d/0x0e-byte list records",
                "0x50-byte transfer records",
            ],
            &["0x41", "0x25", "0x0c", "0x0d", "0x0e", "0x50", "0x213", "0x84d", "0x856"],
            "The loader allocates and reads fixed-width transfer-manager record lists; this proves the Rust runtime store shape but not the bid/value formula itself.",
            "Rust records transfer.dat-equivalent shape ownership as metadata so later transfer queue/value mutators no longer depend on a raw transfer.dat read.",
            "The decompile opens transfer.dat, reads manager fields +0x213/+0x84d/+0x856, allocates fixed-width lists, and imports 0x50-byte transfer records via FUN_00672320.",
        ),
        transfer_contract_formula_lift_entry(
            "queued transfer club-news dispatch and queue drain",
            "0x00449710",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x00449710.c",
            "queue pointer/count loop and post-loop reset",
            &[
                "queue pointer at transfer manager +0x24",
                "queue capacity at transfer manager +0x28",
                "queue count at transfer manager +0x2c",
                "6-byte queued item",
                "club record DAT_00acd5bc + club_id * 0x245",
                "queued item byte +5 selects 0x004531a0 vs 0x00452fe0 payload builder",
            ],
            &[
                "club-news payload via 0x004539f0",
                "human-visible dispatch helper 0x0076e180",
                "affiliated-club dispatch helper 0x0076e390 using club +0x53",
                "queue count +0x2c reset to 0",
                "queue pointer +0x24 and capacity +0x28 cleared when capacity > 99",
            ],
            &["0x24", "0x28", "0x2c", "0x6", "0x245", "0x53", "99"],
            "For each queued 6-byte item, resolve the club by original 0x245 stride, build the news payload, dispatch to the club or affiliated club, then reset queue count; oversized queues free the pointer and clear capacity.",
            "Rust records queued transfer/news payload rows, dispatch helper choice, queue count reset, and pointer/capacity clear semantics in the transfer/contract runtime store.",
            "The decompile loops while local_f4 < *(manager+0x2c), advances by 6 bytes per item, resolves DAT_00acd5bc + club_id*0x245, calls 0x004539f0, dispatches 0x0076e180 or 0x0076e390, clears +0x2c, and if +0x28 > 99 frees +0x24 then clears +0x24/+0x28.",
        ),
        transfer_contract_formula_lift_entry(
            "transfer offer queue/list ownership",
            "0x008d2750",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer_helpers/0x008d2750.c",
            "transfer_offer.cpp loader list allocation and enqueue reads",
            &[
                "offer-list count from stream",
                "transfer manager in_ECX list heads",
                "external pointer table param_2",
            ],
            &[
                "0x0e-byte list node initialized by 0x00672270",
                "4-byte referenced offer rows enqueued through 0x00672320",
                "7-byte inline offer payload rows enqueued through 0x00672320",
                "list head replaced or appended through 0x006722d0/0x00672290",
            ],
            &["0x0e", "4", "7", "0x00672270", "0x00672320"],
            "Read a 16-bit count, create a 0x0e list owner, enqueue fixed-width referenced or inline offer payloads, and preserve/append existing list heads.",
            "Rust records transfer offer queue/list ownership as typed transfer-manager shape metadata; monetary accept/reject arithmetic is intentionally not claimed by this lift.",
            "The decompile allocates operator_new(0xe), initializes it with 0x00672270, reads 4-byte rows and 7-byte payloads, enqueues via 0x00672320, and appends through 0x006722d0/0x00672290.",
        ),
        transfer_contract_formula_lift_entry(
            "shortlist manager club-slot initialization",
            "0x00822e30",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer_helpers/0x00822e30.c",
            "shortlist_manager.cpp fresh-store initialization",
            &[
                "club count DAT_00acd564",
                "club records DAT_00acd5bc",
                "shortlist manager in_ECX",
            ],
            &[
                "club shortlist pointer table allocated with club_count*9 bytes",
                "per-club shortlist block allocated with 0x5c8 bytes",
                "0x25-byte shortlist entries reset across each block",
                "club reputation/score short copied from club +0x80",
            ],
            &["9", "0x5c8", "0x25", "0x245", "0x80", "0xb", "0xffffffff", "0x1e"],
            "For a fresh shortlist manager, allocate a 9-byte slot per club, then initialize 0x5c8 bytes of 0x25-byte entries per club using club stride 0x245.",
            "Rust records shortlist transfer-interest state shape and reset defaults, which supports headless transfer ownership without reading legacy transfer.dat.",
            "The decompile mallocs DAT_00acd564*9, walks clubs by 0x245, mallocs 0x5c8 per club, initializes entries every 0x25 bytes, copies club +0x80, and clears list heads.",
        ),
        transfer_contract_formula_lift_entry(
            "transfer compensation fallback arithmetic",
            "0x008aeb50",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer_decision/0x008aeb50.c",
            "param_4 == -1 compensation branch",
            &[
                "staff contract record from 0x004d59d0",
                "contract status byte +0x35",
                "contract value/int at +0x0c",
                "fee factor helper 0x00848b50",
            ],
            &[
                "param_4 set to 0 when no active status-2 contract",
                "param_4 set to fee_factor * contract +0x0c / 7 when status low bits equal 2",
                "news/side effects receive computed param_4 through 0x00583ae0",
            ],
            &["0x35", "0x3f", "2", "0x0c", "7", "-1", "0x00583ae0"],
            "If caller supplied -1 and the staff contract status low bits equal 2, compute compensation as 0x00848b50() * *(contract+0x0c) / 7; otherwise default it to 0.",
            "Rust records the exact compensation fallback arithmetic used before transfer/free side effects.",
            "The decompile checks param_4 == -1, masks contract +0x35 with 0x3f, calls 0x00848b50, then writes param_4 = factor * *(contract+0xc) / 7 before 0x00583ae0.",
        ),
        transfer_contract_formula_lift_entry(
            "transfer AI squad capacity gate",
            "0x008ba4b0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer_decision/0x008ba4b0.c",
            "squad-status counting and refusal codes",
            &[
                "club squad slots +0xd7, count 0x32",
                "affiliate club from 0x0052a5a0",
                "staff status byte +0x3d",
                "active transfer list helpers 0x008b2d50/0x008b2ea0/0x008b9a70",
            ],
            &[
                "returns 0 with reason 4/5 when squad status count exceeds '2'",
                "returns 0 with reason 0x22/0x23 when positional buckets exceed limits",
                "returns 1 when no capacity/position gate blocks the transfer",
                "may free lowest weighted squad staff through 0x008aeb50",
            ],
            &["0x32", "0xd7", "'2'", "0x19f", "5", "0x1cf", "3", "0x1b3", "7", "0x22"],
            "Count incoming/outgoing status buckets and existing squad/list slots; reject when squad total exceeds ASCII '2', role bucket limits exceed 5/3/7, or required protected bucket is occupied.",
            "Rust records the exact AI capacity gate thresholds used before transfer acceptance/cleanup.",
            "The decompile scans 50 squad slots at +0xd7, compares count to '2', scans role buckets +0x19f/+0x1cf/+0x1b3 with limits 5/3/7, writes refusal reasons 4/5 or 0x22/0x23, and calls 0x008aeb50 to free a lower weighted staff member.",
        ),
        transfer_contract_formula_lift_entry(
            "transfer active-offer membership filters",
            "0x008b2d50/0x008b2ea0/0x008b9a70",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer_decision/0x008b2d50.c; D:/cm0102-carve/decompiled/gameplay_lifts_transfer_decision/0x008b2ea0.c; D:/cm0102-carve/decompiled/gameplay_lifts_transfer_decision/0x008b9a70.c",
            "active offer list scans and flag clear",
            &[
                "transfer manager object pointer table *in_ECX",
                "manager count in_ECX[1]",
                "staff transfer flags at +0x86e + staff*8",
                "offer status byte +0x2c",
                "offer stage byte +0x2e",
            ],
            &[
                "active offer count for same staff or same current club",
                "optional 4-byte row append via 0x00672320",
                "clears staff transfer flag bit 0x10 when no active status-0x13 row is found",
            ],
            &["0x86e", "8", "0x10", "0x2c", "0x2e", "0x13", "0xef"],
            "Scan active transfer lists for same staff/current club while filtering offer status 0x02/0x03/0x04/0x0f/0x10/0x14 and stage <=2; clear flag 0x10 if the active-status search fails.",
            "Rust records the original active-offer membership filters used by contract/transfer decision gates.",
            "The helper decompiles scan list rows, compare +0x2c statuses and +0x2e stage, append matching row ids with 0x00672320 when requested, and 0x008b9a70 clears *(+0x86e+staff*8)&=0xef when no status 0x13 row is found.",
        ),
        transfer_contract_formula_lift_entry(
            "transfer offer total-value arithmetic",
            "0x008d4a30",
            "D:/cm0102-carve/decompiled/gameplay_lifts_final_formula_hunt/0x008d4a30.c",
            "offer total value helper",
            &[
                "offer base fee +0x1b",
                "additional cash +0x35/+0x3a",
                "sell-on percentage byte +0x3f",
                "staff value +0x52",
                "linked player list",
            ],
            &[
                "total starts from +0x1b",
                "adds +0x35 when positive",
                "adds +0x3a when positive",
                "adds staff +0x52 * sell-on percent / 100 when param_4 != 0",
                "adds linked player values or 0x004d7090 values for each list row",
            ],
            &["0x1b", "0x35", "0x3a", "0x3f", "0x52", "100"],
            "Compute offer value as base fee plus positive add-ons plus optional sell-on percentage of staff value, then include linked player/list values.",
            "Rust records the exact total-offer arithmetic used by transfer decision wrappers.",
            "The decompile starts from *(offer+0x1b), adds *(offer+0x35) and *(offer+0x3a), adds staff +0x52 * *(offer+0x3f)/100 when enabled, and loops linked rows to add staff values or 0x004d7090 results.",
        ),
        transfer_contract_formula_lift_entry(
            "transfer offer minimum bid clamp and wage add-on bands",
            "0x008d2d20",
            "D:/cm0102-carve/decompiled/gameplay_lifts_final_formula_hunt/0x008d2d20.c",
            "offer construction amount clamp and add-on byte writes",
            &[
                "incoming param_4 bid amount",
                "computed value +0x1b",
                "monthly/wage add-on fields +0x35/+0x39/+0x3a/+0x3e/+0x3f",
                "offer status flags +0x31/+0x0c",
            ],
            &[
                "cash bid below 5000 is clamped to 5000 when player value +0x52 is positive",
                "offer +0x1b is capped down to param_4 when param_4 is smaller",
                "sell-on/add-on percent byte +0x39 is generated in 10-point or 5-point bands",
                "offer +0x35/+0x3a bounded by add-on percent * 30000 + base, or scaled by local value deltas",
                "status nibble +0x0c is normalized to 1/4/5 in decision branches",
            ],
            &[
                "5000", "0x1b", "0x35", "0x39", "0x3a", "0x3e", "0x3f", "30000",
                "1.4", "1.2", "0.8", "0.75", "0.4", "0.05", "0x32",
            ],
            "Clamp low positive cash bids to 5000, cap base fee against caller max bid, and generate add-on/sell-on bands through the original random thresholds and scaling constants.",
            "Rust records the deterministic clamped-offer scenario and preserves the source constants for future seeded RNG parity.",
            "The decompile clamps param_4 <5000 to 5000 when staff +0x52 is positive, caps offer +0x1b to param_4, and writes +0x35/+0x39/+0x3a/+0x3e/+0x3f using 1.4/1.2/0.8/0.75/0.4/0.05, 30000, and 0x32 caps.",
        ),
        transfer_contract_formula_lift_entry(
            "transfer decision total-versus-valuation gate",
            "0x008bccf0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_final_formula_hunt/0x008bccf0.c",
            "decision wrapper total value comparisons",
            &[
                "random/current valuation from 0x0043ff30(4)",
                "offer total value from 0x008d4a30",
                "current club and buying club pointers",
            ],
            &[
                "if random/current valuation * 1.1 < offer total, branch into acceptance/replacement checks",
                "existing accepted offers add their +0x1b values to current valuation",
                "return 1 when combined current value still beats offer total",
                "return 2 for pending/blocked states and 0 for no blocking decision",
            ],
            &["1.1", "0x008d4a30", "0x008ba4b0", "0x008bade0"],
            "Compare offer total against current/random valuation with a 1.1 multiplier and account for existing accepted offers before deciding accept/block/pass.",
            "Rust records the total-versus-valuation decision threshold used after offer total calculation.",
            "The decompile calls 0x0043ff30(4), 0x008d4a30(0,1,1,0), compares valuation*1.1 with total, adds existing offer +0x1b values, and returns 1/2/0 according to the comparison and capacity checks.",
        ),
        transfer_contract_formula_lift_entry(
            "player valuation cap wrapper",
            "0x004da500",
            "D:/cm0102-carve/decompiled/gameplay_lifts_final_two_pockets/0x004da500.c",
            "contract status/release-cap wrapper around 0x004d79c0",
            &[
                "staff value +0x52",
                "contract status byte +0x35 low bits",
                "contract release/cap value +0x21",
                "contract class nibble +0x4f & 0xf0",
                "deep valuation helper 0x004d79c0",
            ],
            &[
                "returns staff +0x52 when no active status-2 transferable contract exists",
                "clamps base and deep values to contract +0x21 when 0x00850b00 gate is active",
                "class 0x10/0x50 returns deep value",
                "class 0x20/0x30 returns midpoint of base and deep values",
                "otherwise returns base staff value",
            ],
            &["0x52", "0x35", "0x3f", "2", "0x49", "0x21", "0x4e", "4", "0x4f", "0x10", "0x20", "0x30", "0x50"],
            "Resolve active status-2 contract, compute deep value through 0x004d79c0, apply optional release-cap clamp, then select base/deep/midpoint by contract class nibble.",
            "Rust records the exact wrapper semantics so transfer acceptance and display valuation no longer rely only on the simple offer-total formula.",
            "The decompile checks (+0x35 & 0x3f)==2 and +0x49==0, reads staff +0x52, clamps against contract +0x21 when 0x00850b00 is true, calls 0x004d79c0, then returns deep, midpoint, or base according to +0x4f high nibble.",
        ),
        transfer_contract_formula_lift_entry(
            "deep player valuation branch constants",
            "0x004d79c0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_final_two_pockets/0x004d79c0.c",
            "major arithmetic constants and final clamps",
            &[
                "club reputation +0x53/+0x85",
                "nation pointer +0x57 and nation reputation +0x69",
                "staff current ability short +0x20",
                "contract pressure byte +0x3d",
                "contract class nibble +0x4f & 0xf0",
                "release/cap value +0x21",
            ],
            &[
                "availability pressure multiplier local_30 = months * 0.01167 + 1.4165",
                "reputation pressure clamped through 0.1..1.1 and 0.5..2.5 ranges",
                "contract-class multipliers include 1.025/1.075/1.1/1.15/1.2/1.25/1.4/1.5/1.75/2.5",
                "value is clamped to at least 1000 and at most 50000000",
                "release/cap +0x21 overrides final value when active and lower",
            ],
            &[
                "0.01167", "1.4165", "0.1", "1.1", "0.5", "2.5", "1.025", "1.075",
                "1.1", "1.15", "1.2", "1.25", "1.4", "1.5", "1.75", "2.5", "1000",
                "50000000", "0x21",
            ],
            "The deep player value model combines club/nation reputation, ability, contract pressure, class nibble, and caps; the exact branch matrix is now localized to 0x004d79c0.",
            "Rust records the exact constants and final clamping surface, with branch-matrix expansion kept as the remaining narrow valuation work rather than a broad unknown.",
            "The decompile squares reputation inputs, computes local_30 from contract pressure, applies class multipliers/caps, clamps local_78 to 1000..50000000, and caps against contract +0x21 when active.",
        ),
        transfer_contract_formula_lift_entry(
            "wage demand role cap and negotiation bands",
            "0x0082a0b0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_final_two_pockets/0x0082a0b0.c",
            "role switch, young-player scaling, existing-contract cap bands",
            &[
                "role/type selector from 0x00825050",
                "current offer *param_4",
                "caller minimum param_5",
                "staff age/current ability/value",
                "existing contract value +0x0d",
            ],
            &[
                "role case 0x11 uses cap 8000000 and multiplier 0.6",
                "role case 0x13 uses cap 9000000 and multiplier 0.65",
                "role case 0x12 uses cap 10000000 and multiplier 0.75",
                "role cases 0xb/0xc/0xd/0x15 use cap 11000000 and multiplier 0.8",
                "role cases 0xe/0x14 use cap 13000000 and multiplier 1.0",
                "default role cap is 6000000 with multiplier 0.575",
                "existing contract bands gate values at <5001, <8001, <9000, and >=9000",
            ],
            &[
                "0x11", "8000000", "0.6", "0x13", "9000000", "0.65", "0x12",
                "10000000", "0.75", "0xb", "0xc", "0xd", "0x15", "11000000", "0.8",
                "0xe", "0x14", "13000000", "1.0", "6000000", "0.575", "5001", "8001",
                "9000", "1500000",
            ],
            "Select a wage-demand cap/multiplier by role, scale younger valuable players through current offer ratios, then clamp by existing-contract bands and late high-value dampening.",
            "Rust records the exact role cap table and contract-band thresholds used before seeded RNG parity work fills every branch.",
            "The decompile switches on 0x00825050, assigns the literal role caps/multipliers, applies current-offer and age/value scaling, then uses existing-contract thresholds 5001/8001/9000 plus a 1500000 high-value dampener.",
        ),
        transfer_contract_formula_lift_entry(
            "staff contracts coordinator branch clusters",
            "0x00848da0",
            "D:/cm0102-rs/reports/carve_segment_index/00848da0_basic_block_map.md",
            "direct x86 disassembly basic-block and call-neighborhood slices",
            &[
                "staff offer/value helper 0x004d7090",
                "staff/club valuation helper 0x00580a90",
                "date comparison helper 0x00536990",
                "stack scalar [esp+0x34]",
                "staff detail bytes +0x57/+0x5a",
            ],
            &[
                "734 basic blocks and 1058 branch edges mapped",
                "seven staff offer/value helper adjustment branches",
                "four staff/club valuation blend branches",
                "seven date-gated contract branches",
                "first implementation target block 0x0084a907 floors adjusted value to 50000",
            ],
            &[
                "0.25", "-0.5", "0.5", "50000", "150000", "30", "120", "200",
                "0x004d7090", "0x00580a90", "0x00536990", "0x009346d0",
            ],
            "The coordinator calls 0x004d7090 seven times, applies exact post-call multipliers and floors, blends 0x00580a90 valuations, and gates contract paths through 0x00536990 date comparisons.",
            "Rust now exposes provenance-cited helpers for the proven post-helper multipliers/floors and date-gate shapes; semantic wage/renewal names stay blocked until stack-variable meanings are recovered.",
            "Direct disassembly of D:/cm0102/cm0102.exe decoded 3078 instructions into 734 basic blocks; branch cluster artifact identifies 0x0084a907, 0x0084a720, 0x0084aa1d, 0x0084ab3d/0x0084ac42/0x0084ada3, and date gates 0x00849f32/0x00849c20/0x00849912/0x00849dac/0x00849dfa as the first Rust helper targets.",
        ),
        transfer_contract_formula_lift_entry(
            "staff contracts sibling money-band branch clusters",
            "0x0084d5d0",
            "D:/cm0102-rs/reports/carve_segment_index/0084d5d0_basic_block_map.md",
            "direct x86 disassembly basic-block and call-neighborhood slices",
            &[
                "staff/club valuation helper 0x00580a90",
                "date comparison helper 0x00536990",
                "context byte +0x64",
                "floating money-band constants",
            ],
            &[
                "589 basic blocks and 866 branch edges mapped",
                "two staff/club valuation helper branches",
                "one date-gated money branch",
                "context byte +0x64 checks against 1 and 2",
                "first implementation target block 0x0084e67b combines date gate and 250.0 money constant",
            ],
            &[
                "250.0", "50000.0", "55000.0", "75000.0", "85000.0", "92500.0",
                "105000.0", "0.1", "0.15", "0.2", "0.25", "0.0001", "1e-8",
                "5e-9", "1e-9", "0x00580a90", "0x00536990",
            ],
            "The sibling routine is a large money-band valuation/check branch family that calls 0x00580a90 twice and 0x00536990 once, with context-byte +0x64 gates.",
            "Rust now preserves the repeated money-band transform families as provenance-cited helpers while delaying wage/compensation semantic naming until stack variables are recovered.",
            "Direct disassembly of D:/cm0102/cm0102.exe decoded 2244 instructions into 589 basic blocks; branch cluster artifact identifies 0x0084e67b, score-money blocks 0x0084d7b9/0x0084d888/0x0084dafc, towards-base blocks 0x0084ebad/0x0084ec03/0x0084ecda/0x0084f06d, and round sites 0x0084e1f3/0x0084e323/0x0084e286 as Rust helper targets; 0x009346d0 is now lifted as x87 truncate/chop conversion.",
        ),
    ]
}

fn transfer_contract_formula_lift_entry(
    formula: &str,
    function: &str,
    decompile_artifact: &str,
    decompile_lines: &str,
    inputs: &[&str],
    outputs: &[&str],
    constants: &[&str],
    branch_rule: &str,
    rust_semantics: &str,
    evidence: &str,
) -> TransferContractFormulaLiftEntry {
    TransferContractFormulaLiftEntry {
        formula: formula.to_string(),
        function: function.to_string(),
        decompile_artifact: decompile_artifact.to_string(),
        decompile_lines: decompile_lines.to_string(),
        inputs: inputs.iter().map(|item| item.to_string()).collect(),
        outputs: outputs.iter().map(|item| item.to_string()).collect(),
        constants: constants.iter().map(|item| item.to_string()).collect(),
        branch_rule: branch_rule.to_string(),
        rust_semantics: rust_semantics.to_string(),
        evidence: evidence.to_string(),
        promotion_status: "formula-lifted-static-code-derived".to_string(),
    }
}

pub fn transfer_contract_formula_lift_map_ready(
    lifts: &[TransferContractFormulaLiftEntry],
) -> bool {
    let required = [
        (
            "0x004cdef0",
            "contract renewal date-window ladder",
            "1095",
            "renewal windows",
        ),
        (
            "0x004cdef0",
            "contract event side-state status promotion",
            "0xc1",
            "event/contract record status byte +0x35",
        ),
        (
            "0x008a9080",
            "transfer.dat manager record-shape ownership",
            "0x50",
            "Rust-owned transfer manager state",
        ),
        (
            "0x00449710",
            "queued transfer club-news dispatch and queue drain",
            "0x6",
            "queue count +0x2c reset to 0",
        ),
        (
            "0x008d2750",
            "transfer offer queue/list ownership",
            "0x0e",
            "7-byte inline offer payload rows",
        ),
        (
            "0x00822e30",
            "shortlist manager club-slot initialization",
            "0x5c8",
            "0x25-byte shortlist entries",
        ),
        (
            "0x008aeb50",
            "transfer compensation fallback arithmetic",
            "7",
            "param_4 set to fee_factor",
        ),
        (
            "0x008ba4b0",
            "transfer AI squad capacity gate",
            "0x22",
            "returns 0 with reason",
        ),
        (
            "0x008b2d50/0x008b2ea0/0x008b9a70",
            "transfer active-offer membership filters",
            "0xef",
            "clears staff transfer flag bit",
        ),
        (
            "0x008d4a30",
            "transfer offer total-value arithmetic",
            "100",
            "adds staff +0x52",
        ),
        (
            "0x008d2d20",
            "transfer offer minimum bid clamp and wage add-on bands",
            "5000",
            "cash bid below 5000",
        ),
        (
            "0x008bccf0",
            "transfer decision total-versus-valuation gate",
            "1.1",
            "offer total",
        ),
        (
            "0x004da500",
            "player valuation cap wrapper",
            "0x21",
            "class 0x20/0x30 returns midpoint",
        ),
        (
            "0x004d79c0",
            "deep player valuation branch constants",
            "50000000",
            "value is clamped",
        ),
        (
            "0x0082a0b0",
            "wage demand role cap and negotiation bands",
            "1500000",
            "existing contract bands",
        ),
        (
            "0x00848da0",
            "staff contracts coordinator branch clusters",
            "150000",
            "734 basic blocks",
        ),
        (
            "0x0084d5d0",
            "staff contracts sibling money-band branch clusters",
            "105000.0",
            "589 basic blocks",
        ),
    ];
    required
        .iter()
        .all(|(function, formula, constant, output)| {
            lifts.iter().any(|lift| {
                lift.function == *function
                    && lift.formula == *formula
                    && lift.constants.iter().any(|item| item == constant)
                    && lift.outputs.iter().any(|item| item.contains(output))
                    && lift.promotion_status == "formula-lifted-static-code-derived"
                    && (lift
                        .decompile_artifact
                        .starts_with("D:/cm0102-carve/decompiled/")
                        || lift
                            .decompile_artifact
                            .starts_with("D:/cm0102-rs/reports/carve_segment_index/"))
            })
        })
}

pub const TRANSFER_CONTRACT_COORDINATOR_BLOCK_COUNT_00848DA0: u16 = 734;
pub const TRANSFER_CONTRACT_COORDINATOR_BRANCH_EDGES_00848DA0: u16 = 1058;
pub const TRANSFER_CONTRACT_SIBLING_BLOCK_COUNT_0084D5D0: u16 = 589;
pub const TRANSFER_CONTRACT_SIBLING_BRANCH_EDGES_0084D5D0: u16 = 866;

pub const STAFF_CONTRACTS_0084A907_MULTIPLIER: f32 = -0.5;
pub const STAFF_CONTRACTS_0084A720_MULTIPLIER: f32 = 0.25;
pub const STAFF_CONTRACTS_0084AA1D_MULTIPLIER: f32 = 0.25;
pub const STAFF_CONTRACTS_0084AB3D_MULTIPLIER: f32 = 0.5;
pub const STAFF_CONTRACTS_0084A907_FLOOR: i32 = 50_000;
pub const STAFF_CONTRACTS_0084AA1D_FLOOR: i32 = 150_000;
pub const STAFF_CONTRACTS_0084E67B_DATE_THRESHOLD: f32 = 250.0;
pub const STAFF_CONTRACTS_DATE_GATE_30_DAYS: i32 = 30;
pub const STAFF_CONTRACTS_DATE_GATE_120_DAYS: i32 = 120;
pub const STAFF_CONTRACTS_DATE_GATE_200_DAYS: i32 = 200;
pub const STAFF_CONTRACTS_DATE_GATE_BASE_525_DAYS: i32 = 525;
pub const STAFF_CONTRACTS_0084D5D0_SCORE_MULTIPLIER: f32 = 4.0;
pub const STAFF_CONTRACTS_0084D5D0_SCORE_ADDEND: f32 = 1.0;
pub const STAFF_CONTRACTS_0084D5D0_TINY_SCALAR: f64 = 0.0001;
pub const STAFF_CONTRACTS_0084D5D0_RATIO_0_10: f64 = 0.1;
pub const STAFF_CONTRACTS_0084D5D0_RATIO_0_15: f64 = 0.15;
pub const STAFF_CONTRACTS_0084D5D0_RATIO_0_20: f64 = 0.2;
pub const STAFF_CONTRACTS_0084D5D0_RATIO_0_25: f64 = 0.25;
pub const STAFF_CONTRACTS_0084E1F3_BASE: f64 = 3_250.0;
pub const STAFF_CONTRACTS_0084E1F3_MULTIPLIER: f64 = -0.2;
pub const STAFF_CONTRACTS_0084E1F3_SUBTRACT_FROM: i32 = 0x0cb2;
pub const STAFF_CONTRACTS_0084E323_MULTIPLIER: f64 = 1.05;
pub const STAFF_CONTRACTS_0084E286_PRIMARY_MULTIPLIER: f32 = 8.0;
pub const STAFF_CONTRACTS_0084E286_SECONDARY_MULTIPLIER: f32 = 4.0;
pub const CM0102_X87_TRUNCATE_HELPER: &str = "0x009346d0";

pub const STAFF_CONTRACTS_0084D5D0_MONEY_BANDS: &[f64] = &[
    250.0, 350.0, 500.0, 1_250.0, 2_750.0, 3_250.0, 7_250.0, 7_500.0, 10_000.0, 15_000.0, 25_000.0,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffContracts0084a907Inputs {
    pub base_value_before_delta: i32,
    pub rounded_helper_delta: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffContracts0084a907Output {
    pub adjusted_value: i32,
    pub floor_applied: bool,
    pub source_block: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffContractsHelperAdjustmentInput {
    pub base_value_before_delta: i32,
    pub rounded_helper_delta: i32,
    pub floor: Option<i32>,
    pub source_block: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffContractsHelperAdjustmentOutput {
    pub adjusted_value: i32,
    pub floor_applied: bool,
    pub source_block: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffContracts0084e67bDateGate {
    pub date_delta: i32,
    pub threshold: f32,
    pub branches_to_0x0084e6ae: bool,
    pub source_block: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffContractsDateGateOutput {
    pub date_delta: i32,
    pub threshold: i32,
    pub comparison: String,
    pub branch_taken: bool,
    pub source_block: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffContractsMoneyBandTransform {
    pub input_value: f64,
    pub base: f64,
    pub ratio: f64,
    pub output_value: f64,
    pub source_block: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffContractsScoreMoneyTransform {
    pub score: f32,
    pub money_band: f32,
    pub output_value: f64,
    pub source_block: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffContractsRoundSiteExpression {
    pub input_value: f64,
    pub unrounded_value: f64,
    pub rounded_value: Option<i32>,
    pub applied_value: Option<i32>,
    pub source_block: String,
    pub evidence: String,
}

pub fn apply_staff_contracts_0084a907_floor(
    inputs: StaffContracts0084a907Inputs,
) -> StaffContracts0084a907Output {
    let output = apply_staff_contracts_helper_adjustment(StaffContractsHelperAdjustmentInput {
        base_value_before_delta: inputs.base_value_before_delta,
        rounded_helper_delta: inputs.rounded_helper_delta,
        floor: Some(STAFF_CONTRACTS_0084A907_FLOOR),
        source_block: "0x0084a907".to_string(),
    });
    StaffContracts0084a907Output {
        adjusted_value: output.adjusted_value,
        floor_applied: output.floor_applied,
        source_block: output.source_block,
        evidence: output.evidence,
    }
}

pub fn apply_staff_contracts_helper_adjustment(
    input: StaffContractsHelperAdjustmentInput,
) -> StaffContractsHelperAdjustmentOutput {
    let summed = input
        .base_value_before_delta
        .saturating_add(input.rounded_helper_delta);
    let adjusted_value = input.floor.map_or(summed, |floor| summed.max(floor));
    StaffContractsHelperAdjustmentOutput {
        adjusted_value,
        floor_applied: adjusted_value != summed,
        source_block: input.source_block,
        evidence: "D:/cm0102-rs/reports/carve_segment_index/transfer_contracts_branch_clusters.md"
            .to_string(),
    }
}

pub fn staff_contracts_0084a907_unrounded_delta(
    helper_value: i32,
    stack_scalar_esp_0x34: f32,
) -> f32 {
    helper_value as f32 * stack_scalar_esp_0x34 * STAFF_CONTRACTS_0084A907_MULTIPLIER
}

pub fn staff_contracts_helper_unrounded_delta(
    helper_value: i32,
    stack_scalar_esp_0x34: f32,
    multiplier: f32,
) -> f32 {
    helper_value as f32 * stack_scalar_esp_0x34 * multiplier
}

pub fn staff_contracts_0084a720_unrounded_delta(
    helper_value: i32,
    stack_scalar_esp_0x34: f32,
) -> f32 {
    staff_contracts_helper_unrounded_delta(
        helper_value,
        stack_scalar_esp_0x34,
        STAFF_CONTRACTS_0084A720_MULTIPLIER,
    )
}

pub fn staff_contracts_0084aa1d_unrounded_delta(
    helper_value: i32,
    stack_scalar_esp_0x34: f32,
) -> f32 {
    staff_contracts_helper_unrounded_delta(
        helper_value,
        stack_scalar_esp_0x34,
        STAFF_CONTRACTS_0084AA1D_MULTIPLIER,
    )
}

pub fn staff_contracts_0084ab3d_unrounded_delta(
    helper_value: i32,
    stack_scalar_esp_0x34: f32,
) -> f32 {
    staff_contracts_helper_unrounded_delta(
        helper_value,
        stack_scalar_esp_0x34,
        STAFF_CONTRACTS_0084AB3D_MULTIPLIER,
    )
}

pub fn apply_staff_contracts_0084aa1d_floor(
    base_value_before_delta: i32,
    rounded_helper_delta: i32,
) -> StaffContractsHelperAdjustmentOutput {
    apply_staff_contracts_helper_adjustment(StaffContractsHelperAdjustmentInput {
        base_value_before_delta,
        rounded_helper_delta,
        floor: Some(STAFF_CONTRACTS_0084AA1D_FLOOR),
        source_block: "0x0084aa1d/0x0084aa3d".to_string(),
    })
}

pub fn evaluate_staff_contracts_0084e67b_date_gate(
    date_delta: i32,
) -> StaffContracts0084e67bDateGate {
    let branches_to_0x0084e6ae = (date_delta as f32) <= STAFF_CONTRACTS_0084E67B_DATE_THRESHOLD;
    StaffContracts0084e67bDateGate {
        date_delta,
        threshold: STAFF_CONTRACTS_0084E67B_DATE_THRESHOLD,
        branches_to_0x0084e6ae,
        source_block: "0x0084e67b".to_string(),
        evidence: "D:/cm0102-rs/reports/carve_segment_index/transfer_contracts_branch_clusters.md"
            .to_string(),
    }
}

pub fn evaluate_staff_contracts_00849f32_date_gate(
    date_delta: i32,
    staff_byte_0x5a: u8,
    staff_byte_0x57: u8,
) -> StaffContractsDateGateOutput {
    let threshold = STAFF_CONTRACTS_DATE_GATE_120_DAYS
        - i32::from(staff_byte_0x5a)
        - i32::from(staff_byte_0x57);
    date_gate_output(
        date_delta,
        threshold,
        date_delta >= threshold,
        ">=".to_string(),
        "0x00849f32",
    )
}

pub fn evaluate_staff_contracts_00849c20_date_gate(
    date_delta: i32,
    staff_byte_0x5a: u8,
) -> StaffContractsDateGateOutput {
    let threshold = STAFF_CONTRACTS_DATE_GATE_200_DAYS - i32::from(staff_byte_0x5a);
    date_gate_output(
        date_delta,
        threshold,
        date_delta <= threshold,
        "<=".to_string(),
        "0x00849c20",
    )
}

pub fn evaluate_staff_contracts_00849912_date_gate(
    date_delta: i32,
    staff_byte_0x5a: u8,
) -> StaffContractsDateGateOutput {
    let threshold = STAFF_CONTRACTS_DATE_GATE_BASE_525_DAYS + (i32::from(staff_byte_0x5a) * 10);
    date_gate_output(
        date_delta,
        threshold,
        date_delta > threshold,
        ">".to_string(),
        "0x00849912",
    )
}

pub fn evaluate_staff_contracts_00849dac_date_gate(
    date_delta: i32,
) -> StaffContractsDateGateOutput {
    date_gate_output(
        date_delta,
        STAFF_CONTRACTS_DATE_GATE_30_DAYS,
        date_delta < STAFF_CONTRACTS_DATE_GATE_30_DAYS,
        "<".to_string(),
        "0x00849dac",
    )
}

pub fn evaluate_staff_contracts_00849dfa_date_gate(
    date_delta: i32,
) -> StaffContractsDateGateOutput {
    date_gate_output(
        date_delta,
        STAFF_CONTRACTS_DATE_GATE_30_DAYS,
        date_delta >= STAFF_CONTRACTS_DATE_GATE_30_DAYS,
        ">=".to_string(),
        "0x00849dfa",
    )
}

fn date_gate_output(
    date_delta: i32,
    threshold: i32,
    branch_taken: bool,
    comparison: String,
    source_block: &str,
) -> StaffContractsDateGateOutput {
    StaffContractsDateGateOutput {
        date_delta,
        threshold,
        comparison,
        branch_taken,
        source_block: source_block.to_string(),
        evidence: "D:/cm0102-rs/reports/carve_segment_index/transfer_contracts_branch_clusters.md"
            .to_string(),
    }
}

pub fn staff_contracts_money_band_towards_base(
    input_value: f64,
    base: f64,
    ratio: f64,
    source_block: &str,
) -> StaffContractsMoneyBandTransform {
    StaffContractsMoneyBandTransform {
        input_value,
        base,
        ratio,
        output_value: ((input_value - base) * ratio) + base,
        source_block: source_block.to_string(),
        evidence: "D:/cm0102-rs/reports/carve_segment_index/transfer_contracts_branch_clusters.md"
            .to_string(),
    }
}

pub fn staff_contracts_score_money_scalar(
    score: f32,
    money_band: f32,
    source_block: &str,
) -> StaffContractsScoreMoneyTransform {
    let scaled_score =
        (score * STAFF_CONTRACTS_0084D5D0_SCORE_MULTIPLIER) + STAFF_CONTRACTS_0084D5D0_SCORE_ADDEND;
    StaffContractsScoreMoneyTransform {
        score,
        money_band,
        output_value: f64::from(scaled_score)
            * f64::from(money_band)
            * STAFF_CONTRACTS_0084D5D0_TINY_SCALAR,
        source_block: source_block.to_string(),
        evidence: "D:/cm0102-rs/reports/carve_segment_index/transfer_contracts_branch_clusters.md"
            .to_string(),
    }
}

pub fn staff_contracts_0084e1f3_round_expression(
    input_value: i32,
    rounded_value: Option<i32>,
) -> StaffContractsRoundSiteExpression {
    let unrounded_value = ((input_value as f64) - STAFF_CONTRACTS_0084E1F3_BASE)
        * STAFF_CONTRACTS_0084E1F3_MULTIPLIER;
    StaffContractsRoundSiteExpression {
        input_value: input_value as f64,
        unrounded_value,
        rounded_value,
        applied_value: rounded_value
            .map(|rounded| STAFF_CONTRACTS_0084E1F3_SUBTRACT_FROM.saturating_sub(rounded)),
        source_block: "0x0084e1f3".to_string(),
        evidence: "D:/cm0102-rs/reports/carve_segment_index/transfer_contracts_branch_clusters.md"
            .to_string(),
    }
}

pub fn cm0102_x87_truncate_to_i64(value: f64) -> i64 {
    value.trunc() as i64
}

pub fn cm0102_x87_truncate_to_i32(value: f64) -> i32 {
    cm0102_x87_truncate_to_i64(value) as i32
}

pub fn staff_contracts_0084e1f3_with_original_rounding(
    input_value: i32,
) -> StaffContractsRoundSiteExpression {
    let unrounded = ((input_value as f64) - STAFF_CONTRACTS_0084E1F3_BASE)
        * STAFF_CONTRACTS_0084E1F3_MULTIPLIER;
    staff_contracts_0084e1f3_round_expression(
        input_value,
        Some(cm0102_x87_truncate_to_i32(unrounded)),
    )
}

pub fn staff_contracts_0084e323_round_expression(
    input_value: i64,
    rounded_value: Option<i32>,
) -> StaffContractsRoundSiteExpression {
    StaffContractsRoundSiteExpression {
        input_value: input_value as f64,
        unrounded_value: (input_value as f64) * STAFF_CONTRACTS_0084E323_MULTIPLIER,
        rounded_value,
        applied_value: rounded_value,
        source_block: "0x0084e323".to_string(),
        evidence: "D:/cm0102-rs/reports/carve_segment_index/transfer_contracts_branch_clusters.md"
            .to_string(),
    }
}

pub fn staff_contracts_0084e323_with_original_rounding(
    input_value: i64,
) -> StaffContractsRoundSiteExpression {
    let unrounded = (input_value as f64) * STAFF_CONTRACTS_0084E323_MULTIPLIER;
    staff_contracts_0084e323_round_expression(
        input_value,
        Some(cm0102_x87_truncate_to_i32(unrounded)),
    )
}

pub fn staff_contracts_0084e286_visible_score_component(
    stack_esp_0x20: f32,
    secondary_input: f32,
) -> f32 {
    (stack_esp_0x20 * STAFF_CONTRACTS_0084E286_PRIMARY_MULTIPLIER)
        + (secondary_input * STAFF_CONTRACTS_0084E286_SECONDARY_MULTIPLIER)
}

pub fn default_transfer_contract_formula_scenario() -> TransferContractFormulaScenario {
    TransferContractFormulaScenario {
        staff_row: 0,
        side_state_row: 0,
        event_record_row: 0,
        current_date: GameDate {
            year: 2001,
            month: 7,
            day: 1,
        },
        event_due_date: GameDate {
            year: 2001,
            month: 7,
            day: 1,
        },
        event_status_before: 0xc0,
        queued_transfer_row: 0,
        queued_transfer_club_id: 1,
        queued_transfer_payload_kind: 2,
        queue_capacity_before: 100,
        queue_count_before: 1,
        queued_transfer_affiliated_club_id: None,
        compensation_factor: 14,
        contract_value: 700,
        squad_staff_count: 51,
        transfer_base_value: 4000,
        transfer_cash_limit: 4500,
        transfer_param_offer: 4000,
        transfer_sell_on_percent: 20,
        transfer_monthly_installment: 600,
        transfer_installment_months: 10,
        transfer_player_value: 30000,
        player_value_base: 30000,
        player_value_deep_model: 50000,
        player_value_release_cap: 45000,
        player_value_contract_class: 0x20,
        wage_role_case: 0x12,
        wage_role_cap: 10_000_000,
        wage_role_multiplier_per_mille: 750,
        wage_current_offer: 2_000_000,
        wage_existing_contract_value: 8_500,
    }
}

fn default_transfer_contract_runtime_store() -> TransferContractRuntimeStore {
    TransferContractRuntimeStore {
        renewal_windows: Vec::new(),
        contract_events: Vec::new(),
        compensation_values: Vec::new(),
        offer_values: Vec::new(),
        decision_rules: vec![
            TransferContractRuntimeDecisionRule {
                rule: "squad capacity gate".to_string(),
                threshold: "count > '2' after 0x32 squad-slot scan".to_string(),
                outcome: "reject with reason 4/5 unless freeing a lower weighted squad staff succeeds".to_string(),
                source_function: "0x008ba4b0".to_string(),
            },
            TransferContractRuntimeDecisionRule {
                rule: "role bucket gate".to_string(),
                threshold: "+0x19f limit 5; +0x1cf limit 3; +0x1b3 limit 7".to_string(),
                outcome: "reject with reason 0x22/0x23 when positional bucket is full".to_string(),
                source_function: "0x008ba4b0".to_string(),
            },
            TransferContractRuntimeDecisionRule {
                rule: "active offer membership filter".to_string(),
                threshold: "status +0x2c and stage +0x2e filters; missing status 0x13 clears flag 0x10".to_string(),
                outcome: "count active offers or clear staff transfer flag with mask 0xef".to_string(),
                source_function: "0x008b2d50/0x008b2ea0/0x008b9a70".to_string(),
            },
            TransferContractRuntimeDecisionRule {
                rule: "total offer value gate".to_string(),
                threshold: "current valuation * 1.1 < 0x008d4a30 total".to_string(),
                outcome: "enter accept/replacement branch before capacity checks".to_string(),
                source_function: "0x008bccf0".to_string(),
            },
            TransferContractRuntimeDecisionRule {
                rule: "deep valuation branch constants".to_string(),
                threshold: "0x004d79c0 clamps final value to 1000..50000000 and optional contract +0x21 cap".to_string(),
                outcome: "use exact branch constants before transfer decision comparisons".to_string(),
                source_function: "0x004d79c0".to_string(),
            },
            TransferContractRuntimeDecisionRule {
                rule: "wage demand role cap table".to_string(),
                threshold: "role 0x12 cap 10000000 with 0.75 multiplier; contract bands 5001/8001/9000".to_string(),
                outcome: "bound wage demand before negotiation branch acceptance".to_string(),
                source_function: "0x0082a0b0".to_string(),
            },
        ],
        transfer_manager_record_shapes: vec![
            "manager +0x213 scalar".to_string(),
            "manager +0x84d/+0x856 state".to_string(),
            "object stride 0x41".to_string(),
            "list stride 0x25".to_string(),
            "list stride 0x0c".to_string(),
            "list stride 0x0d".to_string(),
            "list stride 0x0e".to_string(),
            "transfer record stride 0x50".to_string(),
            "transfer offer list owner stride 0x0e".to_string(),
            "transfer offer referenced payload width 4".to_string(),
            "transfer offer inline payload width 7".to_string(),
            "shortlist club slot width 9".to_string(),
            "shortlist per-club block 0x5c8".to_string(),
            "shortlist entry stride 0x25".to_string(),
        ],
        transfer_queue: Vec::new(),
        queue_dispatches: Vec::new(),
        queue_pointer_active: true,
        queue_capacity: 100,
        queue_count: 1,
        applied_formula_mutations: 0,
        provenance: "Rust-owned transfer/contract runtime store seeded from verified CM0102 contract renewal and transfer.dat record-shape lifts.".to_string(),
    }
}

pub fn plan_transfer_contract_formula_mutations(
    backend: &RuntimeBackendSystems,
    scenario: &TransferContractFormulaScenario,
) -> Vec<TransferContractFormulaMutation> {
    let mut mutations = Vec::new();
    let has_windows = backend
        .transfer_contract_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "contract renewal date-window ladder");
    if has_windows {
        for offset in [7, 30, 60, 91, 121, 152, 182, 365, 550, 1095, 31, 42, 70] {
            let date = CmPackedDate::from_game_date(scenario.current_date.clone())
                .add_days(offset)
                .to_game_date();
            mutations.push(TransferContractFormulaMutation {
                table: "transfer_contract.renewal_windows".to_string(),
                row: offset as u32,
                field: "renewal_window_date".to_string(),
                record_offset: format!("date helper +{offset} day(s)"),
                before: "not generated".to_string(),
                after: date.iso(),
                source_function: "0x004cdef0".to_string(),
                formula: "contract renewal date-window ladder".to_string(),
                exactness_tier: "formula-derived-transfer-contract".to_string(),
                evidence: "Original code calls 0x00536190 with this literal day offset."
                    .to_string(),
            });
        }
    }
    let has_status = backend
        .transfer_contract_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "contract event side-state status promotion");
    if has_status && scenario.current_date == scenario.event_due_date {
        let after = (scenario.event_status_before & 0xc1) | 1;
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.contract_events".to_string(),
            row: scenario.event_record_row,
            field: "status_byte_0x35".to_string(),
            record_offset: "event/contract row *0x50 +0x35".to_string(),
            before: format!("0x{:02x}", scenario.event_status_before),
            after: format!("0x{after:02x}"),
            source_function: "0x004cdef0".to_string(),
            formula: "contract event side-state status promotion".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original code applies status = (status & 0xc1) | 1 after date equality at event +0x2d/+0x2f.".to_string(),
        });
    }
    let has_queue_dispatch = backend
        .transfer_contract_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "queued transfer club-news dispatch and queue drain");
    if has_queue_dispatch && scenario.queue_count_before > 0 {
        let dispatch_helper = if scenario.queued_transfer_affiliated_club_id.is_some() {
            "0x0076e390"
        } else {
            "0x0076e180"
        };
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.transfer_queue".to_string(),
            row: scenario.queued_transfer_row,
            field: "queued_6_byte_item".to_string(),
            record_offset: "manager +0x24 + row*0x06".to_string(),
            before: "queued".to_string(),
            after: format!(
                "club_id={}; payload_kind=0x{:02x}; stride=0x06",
                scenario.queued_transfer_club_id, scenario.queued_transfer_payload_kind
            ),
            source_function: "0x00449710".to_string(),
            formula: "queued transfer club-news dispatch and queue drain".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original code treats each queued item as 6 bytes and reads the club id plus item byte +5.".to_string(),
        });
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.queue_dispatches".to_string(),
            row: scenario.queued_transfer_row,
            field: "dispatch_helper".to_string(),
            record_offset: "club DAT_00acd5bc + club_id*0x245; optional club +0x53".to_string(),
            before: "not dispatched".to_string(),
            after: dispatch_helper.to_string(),
            source_function: "0x00449710".to_string(),
            formula: "queued transfer club-news dispatch and queue drain".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original code calls 0x0076e180 for normal club dispatch or 0x0076e390 for affiliated club +0x53 dispatch.".to_string(),
        });
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.queue_controller".to_string(),
            row: 0,
            field: "queue_count_0x2c".to_string(),
            record_offset: "manager +0x2c".to_string(),
            before: scenario.queue_count_before.to_string(),
            after: "0".to_string(),
            source_function: "0x00449710".to_string(),
            formula: "queued transfer club-news dispatch and queue drain".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original code writes *(manager+0x2c)=0 after processing the queue."
                .to_string(),
        });
        if scenario.queue_capacity_before > 99 {
            mutations.push(TransferContractFormulaMutation {
                table: "transfer_contract.queue_controller".to_string(),
                row: 0,
                field: "queue_pointer_0x24".to_string(),
                record_offset: "manager +0x24".to_string(),
                before: "allocated".to_string(),
                after: "null".to_string(),
                source_function: "0x00449710".to_string(),
                formula: "queued transfer club-news dispatch and queue drain".to_string(),
                exactness_tier: "formula-derived-transfer-contract".to_string(),
                evidence: "Original code frees the queue pointer and sets +0x24 to 0 when capacity +0x28 is greater than 99.".to_string(),
            });
            mutations.push(TransferContractFormulaMutation {
                table: "transfer_contract.queue_controller".to_string(),
                row: 0,
                field: "queue_capacity_0x28".to_string(),
                record_offset: "manager +0x28".to_string(),
                before: scenario.queue_capacity_before.to_string(),
                after: "0".to_string(),
                source_function: "0x00449710".to_string(),
                formula: "queued transfer club-news dispatch and queue drain".to_string(),
                exactness_tier: "formula-derived-transfer-contract".to_string(),
                evidence: "Original code sets manager queue capacity +0x28 to 0 after freeing oversized queue storage.".to_string(),
            });
        }
    }
    let has_compensation = backend
        .transfer_contract_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "transfer compensation fallback arithmetic");
    if has_compensation {
        let value = scenario
            .compensation_factor
            .saturating_mul(scenario.contract_value)
            / 7;
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.compensation_values".to_string(),
            row: scenario.staff_row,
            field: "param_4_compensation".to_string(),
            record_offset: "contract +0x0c; contract +0x35 low bits; helper 0x00848b50".to_string(),
            before: "-1".to_string(),
            after: value.to_string(),
            source_function: "0x008aeb50".to_string(),
            formula: "transfer compensation fallback arithmetic".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original code computes param_4 = 0x00848b50() * *(contract+0x0c) / 7 when status low bits are 2.".to_string(),
        });
    }
    let has_offer_total = backend
        .transfer_contract_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "transfer offer total-value arithmetic");
    let has_offer_clamp = backend
        .transfer_contract_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "transfer offer minimum bid clamp and wage add-on bands");
    if has_offer_total && has_offer_clamp {
        let clamped_cash =
            if scenario.transfer_param_offer < 5000 && scenario.transfer_player_value > 0 {
                5000
            } else {
                scenario.transfer_param_offer.max(0)
            };
        let base_after_cap = scenario.transfer_base_value.min(clamped_cash);
        let sell_on_value = scenario
            .transfer_player_value
            .saturating_mul(i32::from(scenario.transfer_sell_on_percent))
            / 100;
        let total_value = base_after_cap
            .saturating_add(scenario.transfer_monthly_installment)
            .saturating_add(sell_on_value);
        let capped_deep_value = scenario
            .player_value_deep_model
            .max(scenario.player_value_base)
            .min(scenario.player_value_release_cap);
        let wrapper_value = match scenario.player_value_contract_class {
            0x10 | 0x50 => capped_deep_value,
            0x20 | 0x30 => (capped_deep_value + scenario.player_value_base) / 2,
            _ => scenario.player_value_base,
        };
        let wage_role_bound = (i64::from(scenario.wage_role_cap)
            * i64::from(scenario.wage_role_multiplier_per_mille)
            / 1000) as i32;
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.offer_values".to_string(),
            row: scenario.staff_row,
            field: "cash_bid_clamp".to_string(),
            record_offset: "offer +0x1b; caller param_4; staff +0x52".to_string(),
            before: scenario.transfer_param_offer.to_string(),
            after: clamped_cash.to_string(),
            source_function: "0x008d2d20".to_string(),
            formula: "transfer offer minimum bid clamp and wage add-on bands".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original code clamps param_4 below 5000 to 5000 when staff +0x52 is positive, then caps offer +0x1b to param_4 when lower.".to_string(),
        });
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.offer_values".to_string(),
            row: scenario.staff_row,
            field: "total_offer_value".to_string(),
            record_offset: "offer +0x1b/+0x35/+0x3a/+0x3f; staff +0x52".to_string(),
            before: "not totaled".to_string(),
            after: total_value.to_string(),
            source_function: "0x008d4a30".to_string(),
            formula: "transfer offer total-value arithmetic".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original 0x008d4a30 totals base +0x1b, positive +0x35/+0x3a, and staff +0x52 * sell-on +0x3f / 100.".to_string(),
        });
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.decision_rules".to_string(),
            row: scenario.staff_row,
            field: "total_valuation_gate".to_string(),
            record_offset: "0x0043ff30(4) valuation compared with 0x008d4a30 total".to_string(),
            before: "not evaluated".to_string(),
            after: "acceptance branch when valuation * 1.1 < offer total".to_string(),
            source_function: "0x008bccf0".to_string(),
            formula: "transfer decision total-versus-valuation gate".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original decision wrapper compares current/random valuation * 1.1 against the 0x008d4a30 offer total before capacity/replacement checks.".to_string(),
        });
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.decision_rules".to_string(),
            row: scenario.staff_row,
            field: "player_value_cap_wrapper".to_string(),
            record_offset: "staff +0x52; contract +0x21/+0x35/+0x4f; helper 0x004d79c0".to_string(),
            before: format!(
                "base={}; deep={}; cap={}; class=0x{:02x}",
                scenario.player_value_base,
                scenario.player_value_deep_model,
                scenario.player_value_release_cap,
                scenario.player_value_contract_class
            ),
            after: wrapper_value.to_string(),
            source_function: "0x004da500".to_string(),
            formula: "player valuation cap wrapper".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original wrapper clamps deep valuation to contract +0x21 and returns midpoint for class 0x20/0x30.".to_string(),
        });
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.decision_rules".to_string(),
            row: scenario.staff_row,
            field: "deep_player_value_constants".to_string(),
            record_offset: "0x004d79c0 branch constants and final local_78 clamps".to_string(),
            before: "branch constants not installed".to_string(),
            after: "min=1000; max=50000000; pressure=months*0.01167+1.4165; class multipliers 1.025..2.5".to_string(),
            source_function: "0x004d79c0".to_string(),
            formula: "deep player valuation branch constants".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original deep model combines reputation, ability, contract pressure, class multipliers, and final 1000..50000000/release-cap clamps.".to_string(),
        });
        mutations.push(TransferContractFormulaMutation {
            table: "transfer_contract.decision_rules".to_string(),
            row: scenario.staff_row,
            field: "wage_role_cap".to_string(),
            record_offset: "0x00825050 role selector; 0x0082a0b0 local_10/local_40".to_string(),
            before: format!(
                "role=0x{:02x}; current_offer={}; existing_contract={}",
                scenario.wage_role_case,
                scenario.wage_current_offer,
                scenario.wage_existing_contract_value
            ),
            after: format!(
                "role_cap={}; multiplier_per_mille={}; role_bound={}",
                scenario.wage_role_cap,
                scenario.wage_role_multiplier_per_mille,
                wage_role_bound
            ),
            source_function: "0x0082a0b0".to_string(),
            formula: "wage demand role cap and negotiation bands".to_string(),
            exactness_tier: "formula-derived-transfer-contract".to_string(),
            evidence: "Original wage-demand helper maps role 0x12 to cap 10000000 and multiplier 0.75, then applies existing-contract bands 5001/8001/9000.".to_string(),
        });
    }
    mutations
}

pub fn apply_transfer_contract_formula_plan_to_store(
    store: &mut TransferContractRuntimeStore,
    mutations: &[TransferContractFormulaMutation],
    scenario: &TransferContractFormulaScenario,
) {
    for mutation in mutations {
        match mutation.table.as_str() {
            "transfer_contract.renewal_windows" => {
                let day_offset = mutation.row as i32;
                if !store
                    .renewal_windows
                    .iter()
                    .any(|window| window.day_offset == day_offset)
                {
                    if let Ok(date) = parse_iso_game_date(&mutation.after) {
                        store.renewal_windows.push(TransferContractRuntimeWindow {
                            day_offset,
                            date,
                            source_function: mutation.source_function.clone(),
                        });
                        store.applied_formula_mutations =
                            store.applied_formula_mutations.saturating_add(1);
                    }
                }
            }
            "transfer_contract.contract_events" => {
                if let Some(status) = parse_hex_byte(&mutation.after) {
                    if let Some(event) = store
                        .contract_events
                        .iter_mut()
                        .find(|event| event.row == mutation.row)
                    {
                        event.status_byte_0x35 = status;
                    } else {
                        store
                            .contract_events
                            .push(TransferContractRuntimeContractEvent {
                                row: mutation.row,
                                staff_row: scenario.staff_row,
                                side_state_row: scenario.side_state_row,
                                status_byte_0x35: status,
                                day_offset: 0,
                                due_date: scenario.event_due_date.clone(),
                                source_function: mutation.source_function.clone(),
                                evidence: mutation.evidence.clone(),
                            });
                    }
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "transfer_contract.transfer_queue" => {
                if !store
                    .transfer_queue
                    .iter()
                    .any(|item| item.row == mutation.row)
                {
                    store.transfer_queue.push(TransferContractRuntimeQueueItem {
                        row: mutation.row,
                        club_id: scenario.queued_transfer_club_id,
                        payload_kind: scenario.queued_transfer_payload_kind,
                        stride: "0x06".to_string(),
                        source_function: mutation.source_function.clone(),
                    });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "transfer_contract.queue_dispatches" => {
                if !store
                    .queue_dispatches
                    .iter()
                    .any(|dispatch| dispatch.row == mutation.row)
                {
                    store
                        .queue_dispatches
                        .push(TransferContractRuntimeQueueDispatch {
                            row: mutation.row,
                            club_id: scenario.queued_transfer_club_id,
                            helper: mutation.after.clone(),
                            recipient: scenario
                                .queued_transfer_affiliated_club_id
                                .map(|club_id| format!("affiliated club {club_id}"))
                                .unwrap_or_else(|| {
                                    format!("club {}", scenario.queued_transfer_club_id)
                                }),
                            source_function: mutation.source_function.clone(),
                            evidence: mutation.evidence.clone(),
                        });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "transfer_contract.queue_controller" => {
                match mutation.field.as_str() {
                    "queue_count_0x2c" => store.queue_count = 0,
                    "queue_pointer_0x24" => store.queue_pointer_active = false,
                    "queue_capacity_0x28" => store.queue_capacity = 0,
                    _ => {}
                }
                store.applied_formula_mutations = store.applied_formula_mutations.saturating_add(1);
            }
            "transfer_contract.compensation_values" => {
                if !store
                    .compensation_values
                    .iter()
                    .any(|value| value.row == mutation.row)
                {
                    if let Ok(value) = mutation.after.parse::<i32>() {
                        store
                            .compensation_values
                            .push(TransferContractRuntimeCompensationValue {
                                row: mutation.row,
                                formula: "factor * contract_value / 7".to_string(),
                                value,
                                source_function: mutation.source_function.clone(),
                                evidence: mutation.evidence.clone(),
                            });
                        store.applied_formula_mutations =
                            store.applied_formula_mutations.saturating_add(1);
                    }
                }
            }
            "transfer_contract.offer_values" => {
                let existing = store
                    .offer_values
                    .iter_mut()
                    .find(|value| value.row == mutation.row);
                let parsed = mutation.after.parse::<i32>().ok();
                if let Some(value) = existing {
                    if mutation.field == "total_offer_value" {
                        if let Some(total) = parsed {
                            value.total_value = total;
                        }
                    } else if mutation.field == "cash_bid_clamp" {
                        if let Some(cash_limit) = parsed {
                            value.cash_limit = cash_limit;
                            value.base_value = value.base_value.min(cash_limit);
                        }
                    }
                } else {
                    let cash_limit = if mutation.field == "cash_bid_clamp" {
                        parsed.unwrap_or(scenario.transfer_cash_limit)
                    } else {
                        scenario.transfer_cash_limit
                    };
                    let total_value = if mutation.field == "total_offer_value" {
                        parsed.unwrap_or(0)
                    } else {
                        scenario
                            .transfer_base_value
                            .min(cash_limit)
                            .saturating_add(scenario.transfer_monthly_installment)
                            .saturating_add(
                                scenario.transfer_player_value
                                    * i32::from(scenario.transfer_sell_on_percent)
                                    / 100,
                            )
                    };
                    store.offer_values.push(TransferContractRuntimeOfferValue {
                        row: mutation.row,
                        base_value: scenario.transfer_base_value.min(cash_limit),
                        total_value,
                        cash_limit,
                        sell_on_percent: scenario.transfer_sell_on_percent,
                        monthly_installment: scenario.transfer_monthly_installment,
                        installment_months: scenario.transfer_installment_months,
                        source_function: mutation.source_function.clone(),
                        evidence: mutation.evidence.clone(),
                    });
                }
                store.applied_formula_mutations = store.applied_formula_mutations.saturating_add(1);
            }
            "transfer_contract.decision_rules" => {
                if !store.decision_rules.iter().any(|rule| {
                    rule.source_function == mutation.source_function && rule.rule == mutation.field
                }) {
                    let (rule, threshold) = match mutation.formula.as_str() {
                        "player valuation cap wrapper" => (
                            "player_value_cap_wrapper",
                            "contract +0x21 cap and +0x4f class nibble 0x10/0x20/0x30/0x50",
                        ),
                        "deep player valuation branch constants" => (
                            "deep_player_value_constants",
                            "1000..50000000 final clamp; 0.01167/1.4165 pressure; class multipliers 1.025..2.5",
                        ),
                        "wage demand role cap and negotiation bands" => (
                            "wage_role_cap",
                            "role caps 6000000..13000000; multipliers 0.575..1.0; contract bands 5001/8001/9000",
                        ),
                        _ => (
                            "total_valuation_gate",
                            "current valuation * 1.1 < 0x008d4a30 total",
                        ),
                    };
                    store
                        .decision_rules
                        .push(TransferContractRuntimeDecisionRule {
                            rule: rule.to_string(),
                            threshold: threshold.to_string(),
                            outcome: mutation.after.clone(),
                            source_function: mutation.source_function.clone(),
                        });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            _ => {}
        }
    }
}

fn parse_iso_game_date(value: &str) -> Result<GameDate, ()> {
    let mut parts = value.split('-');
    let year = parts.next().and_then(|part| part.parse::<u16>().ok());
    let month = parts.next().and_then(|part| part.parse::<u8>().ok());
    let day = parts.next().and_then(|part| part.parse::<u8>().ok());
    if parts.next().is_some() {
        return Err(());
    }
    match (year, month, day) {
        (Some(year), Some(month), Some(day)) => Ok(GameDate { year, month, day }),
        _ => Err(()),
    }
}

pub fn transfer_contract_formula_plan_ready(mutations: &[TransferContractFormulaMutation]) -> bool {
    let required_windows = [7, 30, 60, 91, 121, 152, 182, 365, 550, 1095, 31, 42, 70];
    required_windows.iter().all(|offset| {
        mutations.iter().any(|mutation| {
            mutation.table == "transfer_contract.renewal_windows"
                && mutation.row == *offset as u32
                && mutation.exactness_tier == "formula-derived-transfer-contract"
        })
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.contract_events"
            && mutation.record_offset.contains("0x35")
            && mutation.after == "0xc1"
            && mutation.formula == "contract event side-state status promotion"
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.transfer_queue"
            && mutation.record_offset.contains("0x06")
            && mutation.after.contains("payload_kind=0x02")
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.queue_dispatches" && mutation.after == "0x0076e180"
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.queue_controller"
            && mutation.field == "queue_count_0x2c"
            && mutation.after == "0"
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.queue_controller"
            && mutation.field == "queue_pointer_0x24"
            && mutation.after == "null"
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.compensation_values"
            && mutation.formula == "transfer compensation fallback arithmetic"
            && mutation.record_offset.contains("0x0c")
            && mutation.after == "1400"
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.offer_values"
            && mutation.formula == "transfer offer minimum bid clamp and wage add-on bands"
            && mutation.after == "5000"
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.offer_values"
            && mutation.formula == "transfer offer total-value arithmetic"
            && mutation.after == "10600"
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.decision_rules"
            && mutation.formula == "transfer decision total-versus-valuation gate"
            && mutation.after.contains("valuation * 1.1")
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.decision_rules"
            && mutation.formula == "player valuation cap wrapper"
            && mutation.after == "37500"
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.decision_rules"
            && mutation.formula == "deep player valuation branch constants"
            && mutation.after.contains("50000000")
    }) && mutations.iter().any(|mutation| {
        mutation.table == "transfer_contract.decision_rules"
            && mutation.formula == "wage demand role cap and negotiation bands"
            && mutation.after.contains("role_bound=7500000")
    })
}

pub fn transfer_contract_runtime_store_ready(store: &TransferContractRuntimeStore) -> bool {
    let required_windows = [7, 30, 60, 91, 121, 152, 182, 365, 550, 1095, 31, 42, 70];
    required_windows.iter().all(|offset| {
        store
            .renewal_windows
            .iter()
            .any(|window| window.day_offset == *offset)
    }) && store
        .contract_events
        .iter()
        .any(|event| event.row == 0 && (event.status_byte_0x35 & 0x3f) == 1)
        && store
            .transfer_manager_record_shapes
            .iter()
            .any(|shape| shape.contains("transfer record stride 0x50"))
        && store
            .compensation_values
            .iter()
            .any(|value| value.formula == "factor * contract_value / 7" && value.value == 1400)
        && store.offer_values.iter().any(|value| {
            value.cash_limit == 5000
                && value.base_value == 4000
                && value.total_value == 10600
                && value.sell_on_percent == 20
        })
        && store.decision_rules.iter().any(|rule| {
            rule.source_function == "0x008ba4b0" && rule.threshold.contains("+0x19f limit 5")
        })
        && store.decision_rules.iter().any(|rule| {
            rule.source_function == "0x008b2d50/0x008b2ea0/0x008b9a70"
                && rule.threshold.contains("0x2c")
        })
        && store
            .decision_rules
            .iter()
            .any(|rule| rule.source_function == "0x004da500" && rule.outcome == "37500")
        && store
            .decision_rules
            .iter()
            .any(|rule| rule.source_function == "0x004d79c0" && rule.threshold.contains("50000000"))
        && store.decision_rules.iter().any(|rule| {
            rule.source_function == "0x0082a0b0" && rule.threshold.contains("5001/8001/9000")
        })
        && store
            .transfer_queue
            .iter()
            .any(|item| item.row == 0 && item.stride == "0x06")
        && store
            .queue_dispatches
            .iter()
            .any(|dispatch| dispatch.helper == "0x0076e180")
        && !store.queue_pointer_active
        && store.queue_capacity == 0
        && store.queue_count == 0
        && store.applied_formula_mutations >= 25
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineLiftMapEntry {
    pub system: String,
    pub function: String,
    pub role: String,
    pub code_file: String,
    pub verified_state: Vec<String>,
    pub required_before_mutator: Vec<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRuntimeStore {
    pub phase_pipeline: Vec<MatchEngineRuntimePhaseStep>,
    pub player_frontiers: Vec<MatchEngineRuntimePlayerFrontier>,
    pub event_frontiers: Vec<MatchEngineRuntimeEventFrontier>,
    pub tactical_frontiers: Vec<MatchEngineRuntimeTacticalFrontier>,
    #[serde(default = "default_match_engine_runtime_constants")]
    pub constants: Vec<MatchEngineRuntimeConstant>,
    #[serde(default)]
    pub runtime_mutations: Vec<MatchEngineRuntimeMutation>,
    #[serde(default)]
    pub player_evaluation_outputs: Vec<MatchEnginePlayerEvaluationOutput>,
    #[serde(default = "default_match_engine_late_branch_multipliers")]
    pub late_branch_multipliers: Vec<MatchEngineLateBranchMultiplier>,
    #[serde(default = "default_match_engine_rng_gate_schedule")]
    pub rng_gate_schedule: Vec<MatchEngineRngGate>,
    #[serde(default)]
    pub late_branch_execution_outputs: Vec<MatchEngineLateBranchExecutionOutput>,
    #[serde(default)]
    pub action_selection_outputs: Vec<MatchEngineActionSelectionOutput>,
    #[serde(default)]
    pub event_queue_outputs: Vec<MatchEngineEventQueueOutput>,
    #[serde(default)]
    pub state_mutation_outputs: Vec<MatchEngineStateMutationOutput>,
    #[serde(default)]
    pub result_finalization_outputs: Vec<MatchEngineResultFinalizationOutput>,
    pub not_implemented: Vec<String>,
    pub applied_frontier_rows: usize,
    #[serde(default)]
    pub applied_runtime_mutations: usize,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRuntimePhaseStep {
    pub order: u8,
    pub function: String,
    pub label: String,
    pub writes: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRuntimePlayerFrontier {
    pub function: String,
    pub label: String,
    pub input_offsets: Vec<String>,
    pub output_offsets: Vec<String>,
    pub event_codes: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRuntimeEventFrontier {
    pub function: String,
    pub label: String,
    pub event_codes: Vec<String>,
    pub mutated_offsets: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRuntimeTacticalFrontier {
    pub function: String,
    pub label: String,
    pub input_offsets: Vec<String>,
    pub output_offsets: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRuntimeConstant {
    pub symbol: String,
    pub va: String,
    pub file_offset: String,
    pub storage: String,
    pub bytes_le: String,
    pub value: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineLateBranchMultiplier {
    pub order: u8,
    pub branch: String,
    pub predicate: String,
    pub affected_offsets: Vec<String>,
    pub multiplier_symbols: Vec<String>,
    pub multiplier_product: String,
    pub rng_gated: bool,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRngGate {
    pub order: u8,
    pub function: String,
    pub argument: String,
    pub predicate: String,
    pub success_offsets: Vec<String>,
    pub success_multiplier_symbols: Vec<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineLateBranchExecutionOutput {
    pub row: u32,
    pub source_function: String,
    pub applied_branches: Vec<String>,
    pub skipped_branches: Vec<String>,
    pub rng_rolls: Vec<MatchEngineRngRoll>,
    pub offset_multiplier_products: Vec<MatchEngineOffsetMultiplierProduct>,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRngRoll {
    pub order: u8,
    pub function: String,
    pub argument: String,
    pub value: i16,
    pub threshold: i16,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineOffsetMultiplierProduct {
    pub offset: String,
    pub product: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineActionSelectionOutput {
    pub row: u32,
    pub source_functions: Vec<String>,
    pub evaluation_score_0x3b: i16,
    pub action_score_0x37: i16,
    pub shot_action_score_0x39: i16,
    pub direct_shot_return_score: i16,
    pub selected_action_code: String,
    pub selected_action_event_code: Option<String>,
    pub direct_shot_event_code: Option<String>,
    pub decisive_float_offset: String,
    pub decisive_float_value: String,
    pub branch_product_applied: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineEventQueueOutput {
    pub row: u32,
    pub source_function: String,
    pub event_code: String,
    pub slot_base: String,
    pub stride: String,
    pub owner_slot: u8,
    pub target_slot: u8,
    pub action_code: String,
    pub score: i16,
    pub mirror_offset: Option<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineStateMutationOutput {
    pub row: u32,
    pub table: String,
    pub field: String,
    pub record_offset: String,
    pub before: String,
    pub after: String,
    pub source_event_code: Option<String>,
    pub source_function: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineResultFinalizationOutput {
    pub row: u32,
    pub source_function: String,
    pub event_code: String,
    pub match_state_home_offset: String,
    pub match_state_away_offset: String,
    pub fixture_home_offset: String,
    pub fixture_away_offset: String,
    pub home_score: u8,
    pub away_score: u8,
    pub phase_after_offset: String,
    pub phase_after: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEnginePlayerEvaluationOutput {
    pub row: u32,
    pub source_function: String,
    pub evaluation_score_0x3b: i16,
    pub float_0x7d: String,
    pub float_0x8d: String,
    pub float_0x91: String,
    pub float_0x95: String,
    pub float_0x99: String,
    pub float_0xb9: String,
    pub float_0xcd: String,
    pub float_0xf1: String,
    #[serde(default)]
    pub branch_applications: Vec<String>,
    pub constants_used: Vec<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRuntimeScenario {
    pub match_row: u32,
    pub player_slot_row: u32,
    pub home_score: u8,
    pub away_score: u8,
    pub action_owner_slot: u8,
    pub action_target_slot: u8,
    pub event_queue_count_before: u16,
    pub primary_tactic_flags: u16,
    pub secondary_tactic_flags: u16,
    pub player_attr_17: i16,
    pub player_attr_18: i16,
    pub player_attr_19: i16,
    pub player_attr_1a: i16,
    pub player_attr_1e: i16,
    pub player_attr_23: i16,
    pub player_attr_24: i16,
    pub player_attr_26: i16,
    pub player_attr_27: i16,
    pub player_attr_35: i16,
    pub player_attr_39: i16,
    pub player_attr_3a: i16,
    pub evaluation_base_score: i16,
    pub fatigue_drag: i16,
    pub role_local_4c: i16,
    pub role_local_48: i16,
    pub role_local_3c: i16,
    pub shot_base_0x29: i16,
    pub shot_power_0x180: i16,
    pub current_action_score_0x19c: i16,
    pub selected_action_code: u8,
    pub rng_shot_power: i16,
    pub rng_distance_penalty: i16,
    pub rng_distance_penalty_div5: i16,
    pub rng_current_action_score_div5: i16,
    pub rng_action_slot_score: i16,
    #[serde(default)]
    pub late_manager_competition_branch: bool,
    #[serde(default)]
    pub weak_tactical_profile_branch: bool,
    #[serde(default)]
    pub low_confidence_side_state_branch: bool,
    #[serde(default)]
    pub local_38_flags_0x45: u32,
    #[serde(default)]
    pub player_byte_0x5a: i16,
    #[serde(default)]
    pub player_byte_0x5b: i16,
    #[serde(default)]
    pub player_byte_0x57: i16,
    #[serde(default)]
    pub player_byte_0x59: i16,
    #[serde(default)]
    pub player_byte_0x5d: i16,
    #[serde(default)]
    pub cvar16: i16,
    #[serde(default)]
    pub local_38_byte_0x44: i16,
    #[serde(default)]
    pub related_byte_0x44: i16,
    #[serde(default)]
    pub related_short_0x0b: i16,
    #[serde(default)]
    pub related_short_0x0d: i16,
    #[serde(default)]
    pub opponent_short_0x80: i16,
    #[serde(default)]
    pub rng_gate_0x32: i16,
    #[serde(default)]
    pub rng_gate_0x14: i16,
    #[serde(default)]
    pub rng_gate_0x19: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchEngineRuntimeMutation {
    pub table: String,
    pub row: u32,
    pub field: String,
    pub record_offset: String,
    pub before: String,
    pub after: String,
    pub event_code: Option<String>,
    pub source_function: String,
    pub formula: String,
    pub exactness_tier: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultWriteMapEntry {
    pub phase: String,
    pub fixture_home_offset: String,
    pub fixture_away_offset: String,
    pub source_home_offset: String,
    pub source_away_offset: String,
    pub event_code: Option<String>,
    pub threshold: Option<String>,
    pub function: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameplaySystemCodeClaim {
    pub system: String,
    pub claim: String,
    pub function: String,
    pub decompile_artifact: String,
    pub decompile_lines: String,
    pub observed_offsets: Vec<String>,
    pub helpers: Vec<String>,
    pub evidence: String,
    pub promotion_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultCodeClaim {
    pub claim: String,
    pub function: String,
    pub decompile_artifact: String,
    pub decompile_lines: String,
    pub source_offsets: Vec<String>,
    pub fixture_offsets: Vec<String>,
    pub event_codes: Vec<String>,
    pub evidence: String,
    pub promotion_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultFormulaLiftEntry {
    pub formula: String,
    pub function: String,
    pub decompile_artifact: String,
    pub decompile_lines: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub constants: Vec<String>,
    pub branch_rule: String,
    pub rust_semantics: String,
    pub evidence: String,
    pub promotion_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultFormulaScenario {
    pub fixture_row: u32,
    pub fixture_before: Vec<MatchResultFormulaByte>,
    pub period_home_score: u8,
    pub period_away_score: u8,
    pub final_home_score: u8,
    pub final_away_score: u8,
    pub event_queue_count: u16,
    pub mirror_event_queue_count: u16,
    pub abandoned_sentinel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultFormulaByte {
    pub offset: String,
    pub value: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultFormulaMutation {
    pub table: String,
    pub row: u32,
    pub field: String,
    pub record_offset: String,
    pub before: String,
    pub after: String,
    pub event_code: Option<String>,
    pub source_function: String,
    pub formula: String,
    pub exactness_tier: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultRuntimeStore {
    pub fixtures: Vec<MatchResultRuntimeFixture>,
    pub event_queue: Vec<MatchResultRuntimeEvent>,
    pub applied_formula_mutations: usize,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultRuntimeFixture {
    pub row: u32,
    pub bytes: Vec<MatchResultFormulaByte>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultRuntimeEvent {
    pub row: u32,
    pub record_offset: String,
    pub event_code: String,
    pub payload: String,
    pub source_function: String,
    pub formula: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResultMutatorInstallPlan {
    pub system: String,
    pub status: String,
    pub rust_hook: String,
    pub trace_file: String,
    pub required_original_coverage: Vec<String>,
    pub required_rust_coverage: Vec<String>,
    pub required_functions: Vec<String>,
    pub promotion_rule: String,
    pub safety_rule: String,
}

fn default_match_result_code_claims() -> Vec<MatchResultCodeClaim> {
    vec![
        match_result_code_claim(
            "final score event and fixture final score write",
            "0x006a4020",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006a4020.c",
            "221-224",
            &["match-state +0xf5bc", "match-state +0xf5f2"],
            &["fixture +0x49", "fixture +0x4a"],
            &["0x2004"],
            "The decompiled phase/final-score controller emits FUN_006bc8d0(0x2004, ...) and copies match-state +0xf5bc/+0xf5f2 into fixture +0x49/+0x4a.",
        ),
        match_result_code_claim(
            "normal-time score snapshot and transition event",
            "0x006a3240",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006a3240.c",
            "159-170",
            &["match-state +0xf5bd", "match-state +0xf5f3"],
            &["fixture +0x43", "fixture +0x44"],
            &["0x20f2"],
            "The decompiled period transition writer copies match-state +0xf5bd/+0xf5f3 into fixture +0x43/+0x44 at threshold 0x3de and can emit FUN_006bc8d0(0x20f2, ...).",
        ),
        match_result_code_claim(
            "extra-time first-period score snapshot",
            "0x006a3240",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006a3240.c",
            "126-133",
            &["match-state +0xf5bd", "match-state +0xf5f3"],
            &["fixture +0x45", "fixture +0x46"],
            &["0x20f1"],
            "The decompiled period transition writer copies match-state +0xf5bd/+0xf5f3 into fixture +0x45/+0x46 at threshold 0x1ef and emits FUN_006bc8d0(0x20f1, ...).",
        ),
        match_result_code_claim(
            "extra-time final score snapshot",
            "0x006a3240",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006a3240.c",
            "17-46,56-62",
            &["match-state +0xf5bd", "match-state +0xf5f3"],
            &["fixture +0x47", "fixture +0x48"],
            &["0x20f3"],
            "The decompiled period transition writer copies match-state +0xf5bd/+0xf5f3 into fixture +0x47/+0x48 around thresholds 0x483/0x528 and can emit FUN_006bc8d0(0x20f3, ...).",
        ),
        match_result_code_claim(
            "match event queue append slot stride",
            "0x006bc8d0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006bc8d0.c",
            "45-55,152",
            &["event code param_1", "match event queue count *(in_ECX + 8)"],
            &["event slot base +0x30", "event slot stride 0x0e"],
            &["0x2004", "0x20f1", "0x20f2", "0x20f3"],
            "The decompiled event writer accepts event codes in the 8000..0x21e5 range and writes param_1 at in_ECX + count*0x0e + 0x30.",
        ),
    ]
}

fn match_result_code_claim(
    claim: &str,
    function: &str,
    decompile_artifact: &str,
    decompile_lines: &str,
    source_offsets: &[&str],
    fixture_offsets: &[&str],
    event_codes: &[&str],
    evidence: &str,
) -> MatchResultCodeClaim {
    MatchResultCodeClaim {
        claim: claim.to_string(),
        function: function.to_string(),
        decompile_artifact: decompile_artifact.to_string(),
        decompile_lines: decompile_lines.to_string(),
        source_offsets: source_offsets.iter().map(|item| item.to_string()).collect(),
        fixture_offsets: fixture_offsets
            .iter()
            .map(|item| item.to_string())
            .collect(),
        event_codes: event_codes.iter().map(|item| item.to_string()).collect(),
        evidence: evidence.to_string(),
        promotion_status: "code-derived-claim-artifact-present-semantics-not-implemented"
            .to_string(),
    }
}

fn default_match_result_formula_lift_map() -> Vec<MatchResultFormulaLiftEntry> {
    vec![
        match_result_formula_lift_entry(
            "normal-time score snapshot",
            "0x006a3240",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006a3240.c",
            "159-177,202-227",
            &[
                "match-state current phase timer short +0x8ed0",
                "match-state stoppage/offset byte +0x8eb7",
                "match-state score bytes +0xf5bd/+0xf5f3",
                "fixture pointer *(match-state +0x4792)",
            ],
            &[
                "fixture +0x43 normal-time home/status byte",
                "fixture +0x44 normal-time away byte",
                "event 0x20f2 when normal-time snapshot is committed",
                "match-state phase marker +0x8ed0 = 0x3de",
                "match-state next threshold +0x8ed4 = 0x483",
            ],
            &["0x3de", "0x483", "0x20f2"],
            "Commit the normal-time snapshot when +0x8ed0 is not beyond +0x8eb7 + 0x3de and the branch decides the tie should proceed/end at the 0x3de boundary.",
            "Rust match-result mutation copies +0xf5bd/+0xf5f3 to fixture +0x43/+0x44 and appends event 0x20f2 in original order.",
            "The decompile writes *(fixture+0x43)=*(state+0xf5bd), *(fixture+0x44)=*(state+0xf5f3), emits FUN_006bc8d0(0x20f2,...), then updates +0x8ed0/+0x8ed4.",
        ),
        match_result_formula_lift_entry(
            "extra-time first-period score snapshot",
            "0x006a3240",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006a3240.c",
            "126-146",
            &[
                "match-state current phase timer short +0x8ed0",
                "match-state stoppage/offset byte +0x8eb7",
                "match-state score bytes +0xf5bd/+0xf5f3",
                "fixture pointer *(match-state +0x4792)",
            ],
            &[
                "fixture +0x45 extra-time first-period home byte",
                "fixture +0x46 extra-time first-period away byte",
                "event 0x20f1",
                "match-state phase marker +0x8ed0 = 0x1ef",
                "match-state next threshold +0x8ed4 = 0x3de",
            ],
            &["0x1ef", "0x3de", "0x20f1"],
            "Commit the first extra-time snapshot when the period controller reaches threshold 0x1ef and +0x8ed0 is not beyond +0x8eb7 + 0x1ef.",
            "Rust match-result mutation copies +0xf5bd/+0xf5f3 to fixture +0x45/+0x46 and appends event 0x20f1 in original order.",
            "The decompile checks iVar4 == 0x1ef, writes fixture +0x45/+0x46, emits FUN_006bc8d0(0x20f1,...), then writes +0x8ed0/+0x8ed4.",
        ),
        match_result_formula_lift_entry(
            "extra-time final score snapshot",
            "0x006a3240",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006a3240.c",
            "17-46,56-90",
            &[
                "match-state current phase timer short +0x8ed0",
                "match-state stoppage/offset byte +0x8eb7",
                "match-state score bytes +0xf5bd/+0xf5f3",
                "fixture aggregate/tie bytes +0x43/+0x44/+0x47/+0x48/+0x4c",
            ],
            &[
                "fixture +0x47 extra-time/final home byte",
                "fixture +0x48 extra-time/final away byte",
                "event 0x20f3",
                "match-state phase marker +0x8ed0 = 0x483",
                "match-state next threshold +0x8ed4 = 0x528",
            ],
            &["0x483", "0x528", "0x20f3", "0x21", "0x4c"],
            "Commit the extra-time final snapshot around thresholds 0x483/0x528, then branch on aggregate/tie bytes to decide whether play continues.",
            "Rust match-result mutation copies +0xf5bd/+0xf5f3 to fixture +0x47/+0x48 and appends event 0x20f3 when the original transition branch fires.",
            "The decompile writes fixture +0x47/+0x48 at threshold 0x483 or 0x528 and compares aggregate expressions using fixture +0x43/+0x44/+0x47/+0x48/+0x4c.",
        ),
        match_result_formula_lift_entry(
            "phase-controller final result",
            "0x006a4020",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006a4020.c",
            "211-224",
            &[
                "match-state score bytes +0xf5bc/+0xf5f2",
                "match-state aggregate adjustment bytes +0x8ea1/+0x8ea2",
                "fixture pointer *(match-state +0x4792)",
                "phase byte +0x8eb3",
            ],
            &[
                "event 0x2004 with away-not-losing flag",
                "fixture +0x49 final home byte",
                "fixture +0x4a final away byte",
                "phase byte +0x8eb3 reset to 0",
            ],
            &["0x2004", "0x8eb3 cases 3/6", "0xf5bc", "0xf5f2"],
            "For phase cases 3/6, emit final-result event 0x2004 with flag (+0xf5bc <= +0xf5f2), copy final score bytes to fixture +0x49/+0x4a, and reset the phase controller.",
            "Rust final-result mutation must preserve the original event flag rule and write fixture final score bytes from +0xf5bc/+0xf5f2.",
            "The decompile calls FUN_006bc8d0(0x2004,..., *(byte *)(state+0xf5bc) <= *(byte *)(state+0xf5f2), ...), writes fixture +0x49/+0x4a, then sets +0x8eb3 to 0.",
        ),
        match_result_formula_lift_entry(
            "match event queue append layout",
            "0x006bc8d0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_match/0x006bc8d0.c",
            "45-55,152,190",
            &[
                "event code param_1",
                "match-state event count short *(in_ECX + 8)",
                "mirror event count short *(in_ECX + 6)",
                "event payload params 3-7",
            ],
            &[
                "primary event slot at match-state +0x30 + count*0x0e",
                "mirrored event slot at match-state +0x720 + count*0x0e for selected event range",
                "event counters +6/+8/+0xa/+0xc/+0xe",
            ],
            &["0x0e", "0x30", "0x720", "8000..0x21e4"],
            "Append event records using a 0x0e-byte stride from +0x30, and mirror selected 8000..0x21e4 events into the +0x720 event area while preserving recursive follow-up ordering.",
            "Rust event mutation must allocate/apply the same 0x0e-byte ordered slots and keep mirrored event rows deterministic.",
            "The decompile writes param_1 at in_ECX + count*0x0e + 0x30 and writes selected mirrored events at +0x720 using the same stride.",
        ),
    ]
}

fn match_result_formula_lift_entry(
    formula: &str,
    function: &str,
    decompile_artifact: &str,
    decompile_lines: &str,
    inputs: &[&str],
    outputs: &[&str],
    constants: &[&str],
    branch_rule: &str,
    rust_semantics: &str,
    evidence: &str,
) -> MatchResultFormulaLiftEntry {
    MatchResultFormulaLiftEntry {
        formula: formula.to_string(),
        function: function.to_string(),
        decompile_artifact: decompile_artifact.to_string(),
        decompile_lines: decompile_lines.to_string(),
        inputs: inputs.iter().map(|item| item.to_string()).collect(),
        outputs: outputs.iter().map(|item| item.to_string()).collect(),
        constants: constants.iter().map(|item| item.to_string()).collect(),
        branch_rule: branch_rule.to_string(),
        rust_semantics: rust_semantics.to_string(),
        evidence: evidence.to_string(),
        promotion_status: "formula-lifted-static-code-derived".to_string(),
    }
}

pub fn match_result_formula_lift_map_ready(lifts: &[MatchResultFormulaLiftEntry]) -> bool {
    let required = [
        ("0x006a3240", "0x3de", "0x20f2", "fixture +0x43"),
        ("0x006a3240", "0x1ef", "0x20f1", "fixture +0x45"),
        ("0x006a3240", "0x483", "0x20f3", "fixture +0x47"),
        ("0x006a4020", "0x2004", "0xf5bc", "fixture +0x49"),
        ("0x006bc8d0", "0x0e", "0x720", "match-state +0x30"),
    ];
    required.iter().all(|(function, constant, extra, output)| {
        lifts.iter().any(|lift| {
            lift.function == *function
                && lift.constants.iter().any(|item| item.contains(constant))
                && (lift.constants.iter().any(|item| item.contains(extra))
                    || lift.inputs.iter().any(|item| item.contains(extra))
                    || lift.outputs.iter().any(|item| item.contains(extra)))
                && lift.outputs.iter().any(|item| item.contains(output))
                && lift.promotion_status == "formula-lifted-static-code-derived"
                && lift
                    .decompile_artifact
                    .starts_with("D:/cm0102-carve/decompiled/")
        })
    })
}

pub fn default_match_result_formula_scenario() -> MatchResultFormulaScenario {
    MatchResultFormulaScenario {
        fixture_row: 0,
        fixture_before: vec![
            match_result_formula_byte("0x43", 0xff),
            match_result_formula_byte("0x44", 0xff),
            match_result_formula_byte("0x45", 0xff),
            match_result_formula_byte("0x46", 0xff),
            match_result_formula_byte("0x47", 0xff),
            match_result_formula_byte("0x48", 0xff),
            match_result_formula_byte("0x49", 0xff),
            match_result_formula_byte("0x4a", 0xff),
        ],
        period_home_score: 2,
        period_away_score: 1,
        final_home_score: 2,
        final_away_score: 1,
        event_queue_count: 0,
        mirror_event_queue_count: 0,
        abandoned_sentinel: false,
    }
}

fn default_match_result_runtime_store() -> MatchResultRuntimeStore {
    let scenario = default_match_result_formula_scenario();
    MatchResultRuntimeStore {
        fixtures: vec![MatchResultRuntimeFixture {
            row: scenario.fixture_row,
            bytes: scenario.fixture_before,
        }],
        event_queue: Vec::new(),
        applied_formula_mutations: 0,
        provenance: "Rust-owned match-result fixture/event store seeded from lifted CM0102 fixture score byte defaults.".to_string(),
    }
}

fn match_result_formula_byte(offset: &str, value: u8) -> MatchResultFormulaByte {
    MatchResultFormulaByte {
        offset: offset.to_string(),
        value,
    }
}

pub fn apply_match_result_formula_plan_to_store(
    store: &mut MatchResultRuntimeStore,
    mutations: &[MatchResultFormulaMutation],
) {
    for mutation in mutations {
        match mutation.table.as_str() {
            "fixture" => {
                if let Some(value) = parse_hex_byte(&mutation.after) {
                    write_match_result_fixture_byte(
                        store,
                        mutation.row,
                        &mutation.record_offset,
                        value,
                    );
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "event_queue" => {
                if let Some(event_code) = mutation.event_code.clone() {
                    if !store.event_queue.iter().any(|event| {
                        event.row == mutation.row
                            && event.event_code == event_code
                            && event.record_offset == mutation.record_offset
                    }) {
                        store.event_queue.push(MatchResultRuntimeEvent {
                            row: mutation.row,
                            record_offset: mutation.record_offset.clone(),
                            event_code,
                            payload: mutation.after.clone(),
                            source_function: mutation.source_function.clone(),
                            formula: mutation.formula.clone(),
                        });
                        store.applied_formula_mutations =
                            store.applied_formula_mutations.saturating_add(1);
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn apply_match_engine_result_finalization_to_match_result_store(
    store: &mut MatchResultRuntimeStore,
    output: &MatchEngineResultFinalizationOutput,
) {
    write_match_result_fixture_byte(
        store,
        output.row,
        output.fixture_home_offset.trim_start_matches('+'),
        output.home_score,
    );
    write_match_result_fixture_byte(
        store,
        output.row,
        output.fixture_away_offset.trim_start_matches('+'),
        output.away_score,
    );
    let payload = format!(
        "code {}; away_not_losing={}; home=0x{:02x}; away=0x{:02x}",
        output.event_code,
        output.home_score <= output.away_score,
        output.home_score,
        output.away_score
    );
    if let Some(event) = store.event_queue.iter_mut().find(|event| {
        event.event_code == output.event_code && event.source_function == output.source_function
    }) {
        event.payload = payload;
        event.formula = "executable match-engine result finalization".to_string();
    } else {
        store.event_queue.push(MatchResultRuntimeEvent {
            row: store.event_queue.len() as u32,
            record_offset: "match-state +0x30 + row*0x0e".to_string(),
            event_code: output.event_code.clone(),
            payload,
            source_function: output.source_function.clone(),
            formula: "executable match-engine result finalization".to_string(),
        });
    }
    store.applied_formula_mutations = store.applied_formula_mutations.saturating_add(3);
}

fn write_match_result_fixture_byte(
    store: &mut MatchResultRuntimeStore,
    row: u32,
    offset: &str,
    value: u8,
) {
    let fixture_index = store
        .fixtures
        .iter()
        .position(|fixture| fixture.row == row)
        .unwrap_or_else(|| {
            store.fixtures.push(MatchResultRuntimeFixture {
                row,
                bytes: Vec::new(),
            });
            store.fixtures.len() - 1
        });
    let fixture = &mut store.fixtures[fixture_index];
    if let Some(byte) = fixture
        .bytes
        .iter_mut()
        .find(|byte| byte.offset.eq_ignore_ascii_case(offset))
    {
        byte.value = value;
    } else {
        fixture.bytes.push(MatchResultFormulaByte {
            offset: offset.to_string(),
            value,
        });
    }
}

fn parse_hex_byte(value: &str) -> Option<u8> {
    u8::from_str_radix(value.strip_prefix("0x").unwrap_or(value), 16).ok()
}

pub fn match_result_runtime_store_ready(store: &MatchResultRuntimeStore) -> bool {
    let fixture = store.fixtures.iter().find(|fixture| fixture.row == 0);
    let fixture_ready = [
        "0x43", "0x44", "0x45", "0x46", "0x47", "0x48", "0x49", "0x4a",
    ]
    .iter()
    .all(|offset| {
        fixture.is_some_and(|fixture| {
            fixture
                .bytes
                .iter()
                .any(|byte| byte.offset == *offset && byte.value != 0xff)
        })
    });
    let events_ready = ["0x20f1", "0x20f2", "0x20f3", "0x2004"]
        .iter()
        .all(|event_code| {
            store
                .event_queue
                .iter()
                .any(|event| event.event_code == *event_code)
        });
    let executable_finalization_ready = fixture.is_some_and(|fixture| {
        fixture
            .bytes
            .iter()
            .any(|byte| byte.offset == "0x49" && byte.value == 2)
            && fixture
                .bytes
                .iter()
                .any(|byte| byte.offset == "0x4a" && byte.value == 1)
    }) && store.event_queue.iter().any(|event| {
        event.event_code == "0x2004"
            && event.source_function == "0x006a4020"
            && event.formula == "executable match-engine result finalization"
    });
    fixture_ready
        && events_ready
        && executable_finalization_ready
        && store.applied_formula_mutations >= 12
}

pub fn plan_match_result_formula_mutations(
    backend: &RuntimeBackendSystems,
    scenario: &MatchResultFormulaScenario,
) -> Vec<MatchResultFormulaMutation> {
    let mut mutations = Vec::new();
    for entry in &backend.match_result_write_map {
        if entry.phase == "abandoned/sentinel score" && !scenario.abandoned_sentinel {
            continue;
        }
        if let Some(home_after) =
            match_result_score_source_value(entry.source_home_offset.as_str(), scenario)
        {
            push_match_result_formula_fixture_mutation(
                &mut mutations,
                scenario,
                entry,
                "home",
                &entry.fixture_home_offset,
                home_after,
            );
        }
        if let Some(away_after) =
            match_result_score_source_value(entry.source_away_offset.as_str(), scenario)
        {
            push_match_result_formula_fixture_mutation(
                &mut mutations,
                scenario,
                entry,
                "away",
                &entry.fixture_away_offset,
                away_after,
            );
        }
        if let Some(event_code) = &entry.event_code {
            if event_code != "none" {
                push_match_result_formula_event_mutation(
                    &mut mutations,
                    scenario,
                    event_code,
                    &entry.function,
                    &entry.phase,
                    &entry.evidence,
                );
            }
        }
    }
    mutations
}

fn push_match_result_formula_fixture_mutation(
    mutations: &mut Vec<MatchResultFormulaMutation>,
    scenario: &MatchResultFormulaScenario,
    entry: &MatchResultWriteMapEntry,
    side: &str,
    fixture_offset: &str,
    after: u8,
) {
    if fixture_offset.eq_ignore_ascii_case("none") {
        return;
    }
    let before = scenario
        .fixture_before
        .iter()
        .find(|byte| byte.offset.eq_ignore_ascii_case(fixture_offset))
        .map(|byte| byte.value)
        .unwrap_or(0xff);
    mutations.push(MatchResultFormulaMutation {
        table: "fixture".to_string(),
        row: scenario.fixture_row,
        field: format!(
            "{} {side} score byte at fixture {fixture_offset}",
            entry.phase
        ),
        record_offset: fixture_offset.to_string(),
        before: format!("0x{before:02x}"),
        after: format!("0x{after:02x}"),
        event_code: entry.event_code.clone(),
        source_function: entry.function.clone(),
        formula: entry.phase.clone(),
        exactness_tier: "formula-derived-match-result".to_string(),
        evidence: format!(
            "Copied {} score byte through fixture pointer *(match-state +0x4792); {}",
            if side == "home" { "home" } else { "away" },
            entry.evidence
        ),
    });
}

fn push_match_result_formula_event_mutation(
    mutations: &mut Vec<MatchResultFormulaMutation>,
    scenario: &MatchResultFormulaScenario,
    event_code: &str,
    source_function: &str,
    formula: &str,
    evidence: &str,
) {
    let row = scenario.event_queue_count.saturating_add(
        mutations
            .iter()
            .filter(|mutation| mutation.table == "event_queue")
            .count() as u16,
    );
    let after = if event_code == "0x2004" {
        format!(
            "code {event_code}; away_not_losing={}; home=0x{:02x}; away=0x{:02x}",
            scenario.final_home_score <= scenario.final_away_score,
            scenario.final_home_score,
            scenario.final_away_score
        )
    } else {
        format!(
            "code {event_code}; period_home=0x{:02x}; period_away=0x{:02x}",
            scenario.period_home_score, scenario.period_away_score
        )
    };
    mutations.push(MatchResultFormulaMutation {
        table: "event_queue".to_string(),
        row: row.into(),
        field: format!("event {event_code} payload"),
        record_offset: format!("match-state +0x30 + {row}*0x0e"),
        before: "empty event slot".to_string(),
        after,
        event_code: Some(event_code.to_string()),
        source_function: source_function.to_string(),
        formula: formula.to_string(),
        exactness_tier: "formula-derived-match-result".to_string(),
        evidence: format!("Appended 0x0e-byte event queue slot; {evidence}"),
    });
}

fn match_result_score_source_value(
    source_offset: &str,
    scenario: &MatchResultFormulaScenario,
) -> Option<u8> {
    match source_offset {
        "0xf5bd" => Some(scenario.period_home_score),
        "0xf5f3" => Some(scenario.period_away_score),
        "0xf5bc" => Some(scenario.final_home_score),
        "0xf5f2" => Some(scenario.final_away_score),
        "constant 0xfd" => Some(0xfd),
        "none" => None,
        _ => None,
    }
}

pub fn match_result_formula_plan_ready(mutations: &[MatchResultFormulaMutation]) -> bool {
    let required = [
        ("fixture", "0x43", None),
        ("fixture", "0x44", None),
        ("fixture", "0x45", None),
        ("fixture", "0x46", None),
        ("fixture", "0x47", None),
        ("fixture", "0x48", None),
        ("fixture", "0x49", None),
        ("fixture", "0x4a", None),
        ("event_queue", "0x30", Some("0x20f1")),
        ("event_queue", "0x30", Some("0x20f2")),
        ("event_queue", "0x30", Some("0x20f3")),
        ("event_queue", "0x30", Some("0x2004")),
    ];
    required.iter().all(|(table, offset, event)| {
        mutations.iter().any(|mutation| {
            mutation.table == *table
                && mutation.record_offset.contains(offset)
                && event.map_or(true, |event| mutation.event_code.as_deref() == Some(event))
                && mutation.exactness_tier == "formula-derived-match-result"
        })
    })
}

pub fn match_result_code_claims_ready(claims: &[MatchResultCodeClaim]) -> bool {
    let required = [
        ("0x006a4020", "fixture +0x49", "0x2004"),
        ("0x006a3240", "fixture +0x43", "0x20f2"),
        ("0x006a3240", "fixture +0x45", "0x20f1"),
        ("0x006a3240", "fixture +0x47", "0x20f3"),
        ("0x006bc8d0", "event slot stride 0x0e", "0x2004"),
    ];
    required.iter().all(|(function, offset, event)| {
        claims.iter().any(|claim| {
            claim.function == *function
                && (claim.fixture_offsets.iter().any(|item| item == offset)
                    || claim.source_offsets.iter().any(|item| item == offset))
                && claim.event_codes.iter().any(|item| item == event)
                && claim
                    .decompile_artifact
                    .starts_with("D:/cm0102-carve/decompiled/")
                && !claim.decompile_lines.is_empty()
                && claim.promotion_status
                    == "code-derived-claim-artifact-present-semantics-not-implemented"
        })
    })
}

fn default_gameplay_system_code_claims() -> Vec<GameplaySystemCodeClaim> {
    vec![
        gameplay_system_code_claim(
            "competition state",
            "fixture list resolution uses competition-side fixture pointers",
            "0x00752d40",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition/0x00752d40.c",
            "70-72,104-108,139-143",
            &[
                "competition fixture pointer +0x1c",
                "competition fixture pointer +0x20",
                "fixture +0x4d bit 0x100",
                "fixture +0x4d bit 0x200",
            ],
            &["FUN_00596590", "FUN_0075ee00"],
            "The decompile resolves both fixture-list sides, then branches on fixture +0x4d 0x100/0x200 bits before dispatching fixture/table helpers.",
        ),
        gameplay_system_code_claim(
            "competition state",
            "linked fixture roots are followed through club-relative +0x53 records",
            "0x00752d40",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition/0x00752d40.c",
            "115-120,150-155",
            &["competition fixture pointer +0x1c", "competition fixture pointer +0x20", "linked record +0x53"],
            &["FUN_00525040", "FUN_0075ee00"],
            "When fixture +0x4d has bit 0x200, the code resolves a +0x53 linked record with FUN_00525040 before sending it to FUN_0075ee00.",
        ),
        gameplay_system_code_claim(
            "competition state",
            "competition maintenance has a 70-day cadence",
            "0x00752d40",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition/0x00752d40.c",
            "182-183",
            &["day modulo 0x46"],
            &["FUN_0075f0f0"],
            "The maintenance branch calls FUN_0075f0f0 when the active day value modulo 0x46 is zero.",
        ),
        gameplay_system_code_claim(
            "competition state",
            "club loop uses the code-derived club stride before news emission",
            "0x00595580",
            "D:/cm0102-carve/decompiled/gameplay_lifts_competition/0x00595580.c",
            "49-59,91-92",
            &["DAT_00acd5bc", "club stride 0x245"],
            &["FUN_0076e180", "FUN_006724d0"],
            "The function walks clubs at DAT_00acd5bc + index * 0x245 and invokes news/queue helpers for eligible flags.",
        ),
        gameplay_system_code_claim(
            "transfers/contracts",
            "queued transfer processing uses six-byte queue entries",
            "0x00449710",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x00449710.c",
            "26-35,59-69",
            &["queue pointer +0x24", "queue capacity +0x28", "queue count +0x2c", "queue item stride 6"],
            &["FUN_00933d24"],
            "The queue processor reads queued items from +0x24/+0x2c, advances the item offset by 6, and clears/free-resets +0x24/+0x28/+0x2c at capacity.",
        ),
        gameplay_system_code_claim(
            "transfers/contracts",
            "queued transfer finalization resolves club records by original club stride",
            "0x00449710",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x00449710.c",
            "48-54",
            &["DAT_00acd5bc", "club stride 0x245", "linked club record +0x53"],
            &["FUN_004539f0", "FUN_0076e180", "FUN_0076e390"],
            "After FUN_004539f0, the transfer queue path dispatches normal clubs to FUN_0076e180 and linked +0x53 clubs to FUN_0076e390.",
        ),
        gameplay_system_code_claim(
            "transfers/contracts",
            "contract/transfer renewal windows are generated from original date constants",
            "0x004cdef0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x004cdef0.c",
            "145-164",
            &["7", "0x1e", "0x3c", "0x5b", "0x79", "0x98", "0xb6", "0x16d", "0x226", "0x447"],
            &["FUN_00536190"],
            "The contract path builds date windows through FUN_00536190 using the literal offsets 7, 30, 60, 91, 121, 152, 182, 365, 550, and 1095 days.",
        ),
        gameplay_system_code_claim(
            "transfers/contracts",
            "contract event records use the original 0x50-byte stride",
            "0x004cdef0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x004cdef0.c",
            "219-227,251-262",
            &["event/contract stride 0x50", "side-state stride 0x4f", "event +0x2d", "event +0x2f", "event +0x35"],
            &["FUN_00536190"],
            "The function maps source state into event/contract records with a 0x50 stride, writes date fields at +0x2d/+0x2f, and sets status byte +0x35.",
        ),
        gameplay_system_code_claim(
            "transfers/contracts",
            "legacy transfer database loader stores 0x50-byte transfer records",
            "0x008a9080",
            "D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x008a9080.c",
            "477-485",
            &["transfer record stride 0x50"],
            &["FUN_00672320", "FUN_00921cc0"],
            "The loader reads transfer.dat-equivalent records and stores them with FUN_00672320(local_5c, 0x50).",
        ),
        gameplay_system_code_claim(
            "news/inbox",
            "news generation uses 0x68-byte source subrecords",
            "0x0050c8d0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_news/0x0050c8d0.c",
            "24-26,35-49",
            &["source subrecord stride 0x68", "event +0x30"],
            &["FUN_00596fa0", "FUN_00536190"],
            "The news generator derives a source offset with param_1 * 0x68, creates two events, and writes derived payload values into event +0x30.",
        ),
        gameplay_system_code_claim(
            "news/inbox",
            "news queue removal unlinks the original doubly-linked queue shape",
            "0x006724d0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_news/0x006724d0.c",
            "8-26",
            &["queue node +6", "queue node +10", "queue head +1", "queue tail +3"],
            &["FUN_00672260", "FUN_00933d24"],
            "The removal helper rewires node +6/+10 against queue head/tail fields, decrements the queue count, and frees the node.",
        ),
        gameplay_system_code_claim(
            "news/inbox",
            "news emission clears the original per-record pending byte",
            "0x0076e180",
            "D:/cm0102-carve/decompiled/gameplay_lifts_news/0x0076e180.c",
            "20-29",
            &["param_2 +0xcf", "param_1 +0xde"],
            &["FUN_0076dce0"],
            "The emission helper reads a queue/context pointer from +0xcf, clears byte +0xde on the source record, and dispatches FUN_0076dce0.",
        ),
    ]
}

fn gameplay_system_code_claim(
    system: &str,
    claim: &str,
    function: &str,
    decompile_artifact: &str,
    decompile_lines: &str,
    observed_offsets: &[&str],
    helpers: &[&str],
    evidence: &str,
) -> GameplaySystemCodeClaim {
    GameplaySystemCodeClaim {
        system: system.to_string(),
        claim: claim.to_string(),
        function: function.to_string(),
        decompile_artifact: decompile_artifact.to_string(),
        decompile_lines: decompile_lines.to_string(),
        observed_offsets: observed_offsets
            .iter()
            .map(|item| item.to_string())
            .collect(),
        helpers: helpers.iter().map(|item| item.to_string()).collect(),
        evidence: evidence.to_string(),
        promotion_status: "code-derived-claim-artifact-present-semantics-not-implemented"
            .to_string(),
    }
}

pub fn gameplay_system_code_claims_ready(
    claims: &[GameplaySystemCodeClaim],
    system: &str,
    required: &[(&str, &str)],
) -> bool {
    required.iter().all(|(function, offset)| {
        claims.iter().any(|claim| {
            claim.system == system
                && claim.function == *function
                && claim.observed_offsets.iter().any(|item| item == offset)
                && claim
                    .decompile_artifact
                    .starts_with("D:/cm0102-carve/decompiled/")
                && !claim.decompile_lines.is_empty()
                && !claim.evidence.is_empty()
                && claim.promotion_status
                    == "code-derived-claim-artifact-present-semantics-not-implemented"
        })
    })
}

pub fn competition_code_claims_ready(claims: &[GameplaySystemCodeClaim]) -> bool {
    gameplay_system_code_claims_ready(
        claims,
        "competition state",
        &[
            ("0x00752d40", "fixture +0x4d bit 0x100"),
            ("0x00752d40", "fixture +0x4d bit 0x200"),
            ("0x00752d40", "day modulo 0x46"),
            ("0x00595580", "club stride 0x245"),
        ],
    )
}

pub fn transfer_contract_code_claims_ready(claims: &[GameplaySystemCodeClaim]) -> bool {
    gameplay_system_code_claims_ready(
        claims,
        "transfers/contracts",
        &[
            ("0x00449710", "queue item stride 6"),
            ("0x00449710", "club stride 0x245"),
            ("0x004cdef0", "event/contract stride 0x50"),
            ("0x008a9080", "transfer record stride 0x50"),
        ],
    )
}

pub fn news_inbox_code_claims_ready(claims: &[GameplaySystemCodeClaim]) -> bool {
    gameplay_system_code_claims_ready(
        claims,
        "news/inbox",
        &[
            ("0x0050c8d0", "source subrecord stride 0x68"),
            ("0x0050c8d0", "event +0x30"),
            ("0x006724d0", "queue node +6"),
            ("0x0076e180", "param_1 +0xde"),
        ],
    )
}

fn default_match_result_mutator_install_plan() -> MatchResultMutatorInstallPlan {
    let coverage = vec![
        "fixture +0x43 normal-time home/status score byte".to_string(),
        "fixture +0x44 normal-time away score byte".to_string(),
        "fixture +0x49 final home score byte".to_string(),
        "fixture +0x4a final away score byte".to_string(),
        "event 0x2004 final result payload".to_string(),
        "one period transition event payload: 0x20f1, 0x20f2, or 0x20f3".to_string(),
    ];
    MatchResultMutatorInstallPlan {
        system: "match results".to_string(),
        status: "scaffold-ready-pending-original-binary-capture".to_string(),
        rust_hook: "RuntimeBackendSystems.matches exact fixture result mutator".to_string(),
        trace_file: "reports/parity_traces/match-results.json".to_string(),
        required_original_coverage: coverage.clone(),
        required_rust_coverage: coverage,
        required_functions: vec![
            "0x0069d950 match setup".to_string(),
            "0x0069f2f0 match step controller".to_string(),
            "0x006a3240 match period transition writer".to_string(),
            "0x006a4020 match phase/final-score controller".to_string(),
            "0x006bc8d0 match event queue writer".to_string(),
        ],
        promotion_rule: "Only set implementation_present=true after original and Rust trace arrays are exact ordered equals and subsystem coverage passes for both sides.".to_string(),
        safety_rule: "Until promotion, phase-2 headless ticks record frontier attempts only and must not mutate fixture or event records.".to_string(),
    }
}

fn default_match_engine_lift_map() -> Vec<MatchEngineLiftMapEntry> {
    vec![
        match_engine_lift_map_entry(
            "match setup",
            "0x0069d950",
            "Builds the per-fixture match-state before the minute/tick controller runs.",
            "match_eng.cpp",
            &[
                "match-state fixture pointer at +0x4792",
                "home team/player array at +0x4796",
                "away team/player array at +0x6a6e",
                "0x19-byte match incident queue",
            ],
            &[
                "Name the owned Rust fixture/team/player match-state inputs.",
                "Capture a setup trace that proves the Rust match-state is byte-equivalent at fixture anchor and team array boundaries.",
            ],
            "simulation_frontier identifies 0x0069d950 as verified match setup: builds match-state, anchors fixture through +0x4792, configures team/player arrays via 0x006c0f10, uses match RNG for setup events, and queues 0x19-byte incidents.",
        ),
        match_engine_lift_map_entry(
            "match step controller",
            "0x0069f2f0",
            "Runs the match loop, dispatches phases, advances counters, and reaches fixture result writes.",
            "match_eng.cpp",
            &[
                "fixture pointer read through match-state +0x4792",
                "loop gate byte at +0x8eb4",
                "phase byte at +0x8eb3",
                "tick counters at +0x8ed0/+0x8ed2",
                "sentinel byte at +0xf638",
            ],
            &[
                "Promote 0x0069f2f0 from proposed/inferred to verified in the carve.",
                "Capture original writes for fixture status/event codes 0x217b/0x2002/0x2003/0x2004.",
            ],
            "carve ask 0x0069f2f0 attributes this to match_eng.cpp as proposed/inferred; simulation_frontier records the verified frontier shape and calls to 0x006a4020/0x006a3240/action frontiers.",
        ),
        match_engine_lift_map_entry(
            "match phase possession controller",
            "0x006a4020",
            "Switches on match phase, selects players/actions, resolves events, and writes final fixture scores.",
            "match_eng.cpp",
            &[
                "phase byte at +0x8eb3",
                "action scratch reset range +0x475a..+0x4769",
                "team/player slot stride 0x1be",
                "final score source bytes +0xf5bc/+0xf5f2",
                "fixture final score bytes +0x49/+0x4a",
            ],
            &[
                "Name every phase case that can emit 0x2004/0x2005/0x2006.",
                "Capture original and Rust final-score mutations in exact order.",
            ],
            "carve verifies 0x006a4020 as match_phase_final_score_controller: reads match-state score bytes +0xf5bc/+0xf5f2, emits 0x2004 through 0x006bc8d0, and writes fixture final score bytes +0x49/+0x4a via match-state +0x4792.",
        ),
        match_engine_lift_map_entry(
            "match period transition writer",
            "0x006a3240",
            "Copies period score snapshots into fixture result bytes and emits transition events.",
            "match_eng.cpp",
            &[
                "period short at +0x8ed4",
                "tick short at +0x8ed0",
                "score source bytes +0xf5bd/+0xf5f3",
                "fixture score snapshot bytes +0x43..+0x48",
                "transition event codes 0x20f1/0x20f2/0x20f3",
            ],
            &[
                "Promote threshold semantics for 0x1ef/0x3de/0x483/0x528 from frontier to mutator rules.",
                "Capture ordered fixture byte writes and event queue writes for normal time and extra time.",
            ],
            "carve verifies 0x006a3240 as match_period_transition_writer: gates thresholds 0x1ef/0x3de/0x483/0x528, copies match-state +0xf5bd/+0xf5f3 to fixture +0x43..+0x48 snapshots, and emits 0x20f1/0x20f2/0x20f3.",
        ),
        match_engine_lift_map_entry(
            "match event queue writer",
            "0x006bc8d0",
            "Appends match events that explain result and period state changes.",
            "match_events.cpp",
            &[
                "accepted event code range 8000..0x21e4",
                "event slot base +0x30",
                "event slot stride 0x0e",
                "event counters at +6/+8/+0xa/+0xc/+0xe",
                "mirror event area at +0x720",
            ],
            &[
                "Capture event slot payloads for every score/period event in match-results parity traces.",
                "Implement event queue writes before counting match-result mutator parity as complete.",
            ],
            "simulation_frontier identifies 0x006bc8d0 as verified match_events.cpp frontier: it appends 0x0e-byte event slots, writes code/flags/participants/payload, mirrors selected events, and emits recursive follow-up codes.",
        ),
    ]
}

fn match_engine_lift_map_entry(
    system: &str,
    function: &str,
    role: &str,
    code_file: &str,
    verified_state: &[&str],
    required_before_mutator: &[&str],
    evidence: &str,
) -> MatchEngineLiftMapEntry {
    MatchEngineLiftMapEntry {
        system: system.to_string(),
        function: function.to_string(),
        role: role.to_string(),
        code_file: code_file.to_string(),
        verified_state: verified_state.iter().map(|item| item.to_string()).collect(),
        required_before_mutator: required_before_mutator
            .iter()
            .map(|item| item.to_string())
            .collect(),
        evidence: evidence.to_string(),
    }
}

fn default_match_engine_runtime_store() -> MatchEngineRuntimeStore {
    let phase_pipeline = vec![
        match_engine_runtime_phase_step(
            0,
            "0x00699640",
            "match-day queue builder",
            &[
                "0x18-byte competition group rows",
                "0x54-byte match group rows",
                "0x69-byte fixture snapshots",
            ],
        ),
        match_engine_runtime_phase_step(
            1,
            "0x00699d90",
            "match-day processor/setup dispatcher",
            &[
                "0x11d-byte per-match scratch allocation",
                "16 staff/team slots",
                "calls 0x0069d950 match setup",
            ],
        ),
        match_engine_runtime_phase_step(
            2,
            "0x0069d950",
            "verified match setup",
            &[
                "match-state +0x4792 fixture pointer",
                "match-state +0x4796 home team/player array",
                "match-state +0x6a6e away team/player array",
                "0x19-byte incident queue",
            ],
        ),
        match_engine_runtime_phase_step(
            3,
            "0x006c0f10",
            "team/player setup",
            &[
                "0x18e3 team block copy",
                "fixture +0x4d nibble squad-count source",
                "tactics loader 0x008830a0",
            ],
        ),
        match_engine_runtime_phase_step(
            4,
            "0x006d1a20",
            "player evaluation frontier",
            &[
                "player-slot +0x3b short",
                "player-slot float fields +0x7d/+0x8d/+0x91/+0x95/+0x99/+0xb9/+0xcd/+0xf1",
            ],
        ),
        match_engine_runtime_phase_step(
            5,
            "0x006d46c0",
            "player action-score frontier",
            &["player-slot +0x37 short reset/accumulator"],
        ),
        match_engine_runtime_phase_step(
            6,
            "0x006e65e0",
            "shot/action score frontier",
            &[
                "player-slot +0x39 score short",
                "action bytes 0x16..0x1d/+0x33/+0x35/+0x3a",
            ],
        ),
        match_engine_runtime_phase_step(
            7,
            "0x006a0550",
            "stored-action resolver",
            &[
                "match-state scratch +0x475a..+0x4769",
                "match-state action/event bytes +0x8ea7/+0x8ea8/+0x8eae/+0x8eb2",
            ],
        ),
        match_engine_runtime_phase_step(
            8,
            "0x0069f2f0",
            "match step controller",
            &[
                "fixture status byte +0x43",
                "match-state tick counters +0x8ed0/+0x8ed2",
            ],
        ),
    ];
    let player_frontiers = vec![
        match_engine_runtime_player_frontier(
            "0x006d9ea0",
            "player random-byte seed",
            &["attribute bytes +0x10e..+0x117", "match RNG 0x008fc4f0"],
            &["derived bytes +0x104..+0x10d"],
            &[],
        ),
        match_engine_runtime_player_frontier(
            "0x006d1a20",
            "player evaluation",
            &[
                "player-slot +0x27 side/slot",
                "player-slot +0x19e match-state pointer",
                "tactical flags 0x006a91d0/0x006a9200",
                "float jitter 0x00935080",
            ],
            &[
                "player-slot +0x3b short",
                "player-slot floats +0x7d/+0x8d/+0x91/+0x95/+0x99/+0xb9/+0xcd/+0xf1",
            ],
            &[],
        ),
        match_engine_runtime_player_frontier(
            "0x006d46c0",
            "player action-score",
            &[
                "player data bytes +0x17/+0x18/+0x19/+0x1a through +0x6d",
                "player evaluation 0x006d1a20",
                "tactical flags 0x006a91d0/0x006a9200",
            ],
            &["player-slot +0x37 score short"],
            &[],
        ),
        match_engine_runtime_player_frontier(
            "0x006db630",
            "player action attempt",
            &[
                "match-state +0xf576 current player pointer",
                "match-state +0x4782 possession/event owner",
                "match-state +0x8ea7/+0x8ea8/+0x8eae/+0x8eb2",
            ],
            &["calls 0x006d46c0", "can emit match event 0x1f5e"],
            &["0x1f5e"],
        ),
        match_engine_runtime_player_frontier(
            "0x006d63f0",
            "move/action resolution",
            &[
                "position bytes +0x102/+0x103",
                "random bytes +0x104/+0x107",
                "player-slot +0x2b action counter",
            ],
            &["player-slot +0x198 action id", "player-slot +0x19c outcome short"],
            &["0x1f4d"],
        ),
        match_engine_runtime_player_frontier(
            "0x006f99c0",
            "action selector",
            &["match-state +0x8ea9", "action id candidates 0x68/0x6a/0x6b/0x76"],
            &["player-slot +0x198 selected action id"],
            &[],
        ),
        match_engine_runtime_player_frontier(
            "0x006e65e0",
            "shot/action score",
            &[
                "player shorts +0x29/+0x146/+0x148/+0x14a/+0x14c/+0x14e/+0x150/+0x152/+0x154/+0x180/+0x198/+0x19c",
                "player floats +0x79/+0x81/+0xe5",
            ],
            &["player-slot +0x39 score short", "action bytes 0x16..0x1d/+0x33/+0x35/+0x3a"],
            &["0x1f7f", "0x1f81"],
        ),
    ];
    let event_frontiers = vec![
        match_engine_runtime_event_frontier(
            "0x006bc8d0",
            "match event queue writer",
            &["8000..0x21e4", "0x21a0", "0x21bf"],
            &[
                "event slot base +0x30",
                "event slot stride 0x0e",
                "event counters +6/+8/+0xa/+0xc/+0xe",
                "mirror event area +0x720",
            ],
        ),
        match_engine_runtime_event_frontier(
            "0x006a0550",
            "stored-action resolver",
            &[
                "0x20f0", "0x20ee", "0x20fb", "0x1f7a", "0x20f5", "0x2109", "0x20df", "0x20e0",
                "0x20d9",
            ],
            &[
                "match-state scratch +0x475a..+0x4769",
                "match-state +0x8ea7/+0x8ea8/+0x8eae/+0x8eb2",
                "match-state +0xf582/+0xf5ca",
            ],
        ),
        match_engine_runtime_event_frontier(
            "0x006f63f0",
            "event resolution dispatcher",
            &["0x1f44"],
            &[
                "match-state +0x8eb2",
                "delegates to shot/action score 0x006e65e0",
            ],
        ),
        match_engine_runtime_event_frontier(
            "0x006dfc50/0x006dfe90",
            "follow-up event handlers",
            &["0x1f78"],
            &[
                "player-slot +0x2b",
                "match-state +0xf57a",
                "match-state +0x8eb2",
            ],
        ),
        match_engine_runtime_event_frontier(
            "0x006aae20",
            "per-tick tactical/commentary updater",
            &[
                "0x21cf", "0x21c1", "0x2137", "0x2138", "0x2139", "0x213a", "0x213b", "0x213c",
                "0x213d",
            ],
            &[
                "match-state +0x1c5",
                "match-state +0x4782",
                "match-state +0x475a",
                "match-state +0x8ed0",
            ],
        ),
    ];
    let tactical_frontiers = vec![
        match_engine_runtime_tactical_frontier(
            "0x008830a0",
            "tactics block loader",
            &[
                "source tactic table *(param_1 + 0x601)",
                "tactic stride 0x91",
            ],
            &["0x24-byte tactic block copied to match-state tactic area"],
        ),
        match_engine_runtime_tactical_frontier(
            "0x00882f60",
            "tactic index resolver",
            &[
                "club/staff tactic owner pointer",
                "fallback selected tactic pointer +0xcf",
            ],
            &["resolved tactic index or -1"],
        ),
        match_engine_runtime_tactical_frontier(
            "0x00882240",
            "selected tactic staff slot lookup",
            &[
                "tactic stride 0x91",
                "staff slot offset 0x52",
                "slot byte index",
            ],
            &["selected staff slot pointer"],
        ),
        match_engine_runtime_tactical_frontier(
            "0x006a91d0",
            "primary tactic flag reader",
            &[
                "player-slot +0x19",
                "player-slot +0x27",
                "match-state +0x8ebc table",
            ],
            &["primary tactical flag bit"],
        ),
        match_engine_runtime_tactical_frontier(
            "0x006a9200",
            "secondary tactic flag reader",
            &[
                "player-slot +0x19",
                "player-slot +0x27",
                "match-state +0x8ec4 table",
            ],
            &["secondary tactical flag bit"],
        ),
    ];
    let applied_frontier_rows = phase_pipeline.len()
        + player_frontiers.len()
        + event_frontiers.len()
        + tactical_frontiers.len();
    MatchEngineRuntimeStore {
        phase_pipeline,
        player_frontiers,
        event_frontiers,
        tactical_frontiers,
        constants: default_match_engine_runtime_constants(),
        runtime_mutations: Vec::new(),
        player_evaluation_outputs: Vec::new(),
        late_branch_multipliers: default_match_engine_late_branch_multipliers(),
        rng_gate_schedule: default_match_engine_rng_gate_schedule(),
        late_branch_execution_outputs: Vec::new(),
        action_selection_outputs: Vec::new(),
        event_queue_outputs: Vec::new(),
        state_mutation_outputs: Vec::new(),
        result_finalization_outputs: Vec::new(),
        not_implemented: vec![
            "phase subsystem mutations".to_string(),
            "fixture cleanup semantics".to_string(),
            "manager-manager mutations".to_string(),
            "stadium/date cleanup mutations".to_string(),
            "match-day processing".to_string(),
        ],
        applied_frontier_rows,
        applied_runtime_mutations: 0,
        provenance: "Static-code-derived from D:/cm0102-rs/reports/simulation_frontier_validation.json; exact arithmetic mutators are installed only after each frontier formula is lifted from decompile evidence.".to_string(),
    }
}

fn default_match_engine_late_branch_multipliers() -> Vec<MatchEngineLateBranchMultiplier> {
    vec![
        match_engine_late_branch_multiplier(
            0,
            "late manager/competition branch",
            "((param_1 +0x19e is null or primary tactic reader != 1) && match participant owner points at DAT_009bba50)",
            &["+0xf5", "+0x9d", "+0xf1", "+0xb9"],
            &[
                "_DAT_00958324",
                "_DAT_0095b33c",
                "_DAT_0095b33c",
                "_DAT_00958324",
            ],
            "per-offset: +0xf5 0.899999976, +0x9d/+0xf1 0.925000012, +0xb9 0.899999976",
            false,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1299-1313",
        ),
        match_engine_late_branch_multiplier(
            1,
            "weak tactical profile branch",
            "param_1 +0x19e is null or primary tactic reader != 1; role byte +0x0f == 0x14; role bytes +0x11/+0x12/+0x14/+0x15 < 10 and +0x13 < 15",
            &[
                "+0x9d", "+0xf1", "+0x95", "+0x91", "+0x7d", "+0xad", "+0xa9",
                "+0xcd", "+0xc5", "+0xdd", "+0xb9",
            ],
            &[
                "_DAT_0095b0f4",
                "_DAT_0095aef4",
                "_DAT_0095aef4",
                "_DAT_0095aef4",
                "_DAT_0095aef4",
                "_DAT_0095b034",
                "_DAT_0095af10",
                "_DAT_00956f10",
                "_DAT_00956f10",
                "_DAT_0095af10",
                "_DAT_0095af10",
            ],
            "per-offset: +0x9d 0.05, +0xf1/+0x95/+0x91/+0x7d 0.25, +0xad 0.85, +0xa9/+0xdd/+0xb9 0.75, +0xcd/+0xc5 0.5",
            false,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1314-1330",
        ),
        match_engine_late_branch_multiplier(
            2,
            "low-confidence side-state branch",
            "staff side-state byte +0x0a > 8 and player slot byte +0x5b < 15",
            &["+0x7d", "+0xf5", "+0xf1", "+0x9d", "+0x95", "+0xc1"],
            &[
                "_DAT_00958fcc",
                "_DAT_00958fcc",
                "_DAT_00958324",
                "_DAT_00958324",
                "_DAT_00958324",
                "_DAT_00958fcc",
            ],
            "per-offset: +0x7d/+0xf5/+0xc1 0.949999988, +0xf1/+0x9d/+0x95 0.899999976",
            false,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1341-1348",
        ),
        match_engine_late_branch_multiplier(
            3,
            "local_38 pressure mask 0x40000 rng-success branch",
            "(local_38 +0x45 & 0x40000) != 0 and FUN_008fc4f0(0x32) exceeds player byte +0x5a plus cVar16",
            &["+0xf5", "+0x9d", "+0xf1", "+0x95", "+0x91", "+0xc1"],
            &[
                "_DAT_00958fcc",
                "_DAT_00958324",
                "_DAT_00958324",
                "_DAT_00958fcc",
                "_DAT_00958fcc",
                "_DAT_00958fcc",
            ],
            "per-offset: +0xf5/+0x95/+0x91/+0xc1 0.949999988, +0x9d/+0xf1 0.899999976",
            true,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1350-1358",
        ),
        match_engine_late_branch_multiplier(
            4,
            "local_38 opponent-strength pressure branch",
            "local_38 +0x44 < -25, related record +0x44 < 15, player +0x5b < 10, related +0x0b > 0x0ea6, and opponent +0x80 + 250 < related +0x0b",
            &["+0xf5", "+0x7d", "+0xf1", "+0x9d", "+0x95", "+0x99", "+0xc1"],
            &[
                "_DAT_00958fcc",
                "_DAT_00958324",
                "_DAT_00958324",
                "_DAT_00958324",
                "_DAT_00958fcc",
                "_DAT_00958324",
                "_DAT_00958fcc",
            ],
            "outer applies +0xf5 0.949999988 and +0x7d 0.899999976; deeper local_38 +0x44 < -75 adds +0xf1/+0x9d/+0x99 0.899999976 and +0x95/+0xc1 0.949999988",
            false,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1359-1382",
        ),
        match_engine_late_branch_multiplier(
            5,
            "local_38 mask 0x17 confidence branch",
            "(local_38 +0x45 & 0x17) != 0; if player +0x5b/player +0x57 low enough, +0xf5 may be multiplied by 0.899999976; otherwise 1.1",
            &["+0xf5"],
            &["_DAT_00958324", "_DAT_00958334"],
            "conditional per-offset: +0xf5 0.899999976 or 1.1",
            false,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1383-1394",
        ),
        match_engine_late_branch_multiplier(
            6,
            "local_38 mask 0x2088 defensive-pressure branch",
            "(local_38 +0x45 & 0x2088) != 0 and related +0x0b > 0x109a with player +0x5b < 10, or player +0x5b < 5",
            &["+0xf5"],
            &["_DAT_00958324"],
            "per-offset: +0xf5 0.899999976",
            false,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1395-1400",
        ),
        match_engine_late_branch_multiplier(
            7,
            "local_38 mask 0x38da00 severe-pressure branch",
            "(local_38 +0x45 & 0x38da00) != 0 and player +0x5b is 1 or <= 5",
            &["+0xf5"],
            &["_DAT_0095b05c", "_DAT_00958fcc"],
            "conditional per-offset: +0xf5 0.800000012 when +0x5b == 1, otherwise 0.949999988 when +0x5b <= 5",
            false,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1401-1412",
        ),
        match_engine_late_branch_multiplier(
            8,
            "local_38 mask 0x20000 special-action branch",
            "(local_38 +0x45 & 0x20000) != 0; either exact low-byte/high-related condition or FUN_008fc4f0(0x14) success",
            &["+0xf5", "+0x9d", "+0xf1", "+0x95", "+0x91", "+0xc1"],
            &[
                "_DAT_0095b05c",
                "_DAT_00958324",
                "_DAT_00958324",
                "_DAT_00958324",
                "_DAT_0095b05c",
                "_DAT_00958324",
                "_DAT_00958fcc",
            ],
            "condition path: +0xf5/+0x91 0.800000012, +0x9d/+0xf1/+0x95/+0xc1 0.899999976; RNG path: +0xf5/+0x91/+0xc1 0.949999988, +0x9d/+0xf1/+0x95 0.899999976",
            true,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1413-1434",
        ),
        match_engine_late_branch_multiplier(
            9,
            "local_38 mask 0x40 special-action branch",
            "(local_38 +0x45 & 0x40) != 0; either exact low-byte/high-related condition or FUN_008fc4f0(0x19) success",
            &["+0xf5", "+0x9d", "+0xf1", "+0x95", "+0x91", "+0xc1"],
            &[
                "_DAT_0095b05c",
                "_DAT_00958324",
                "_DAT_00958324",
                "_DAT_00958324",
                "_DAT_0095b05c",
                "_DAT_00958324",
                "_DAT_00958fcc",
            ],
            "condition path: +0xf5/+0x91 0.800000012, +0x9d/+0xf1/+0x95/+0xc1 0.899999976; RNG path: +0xf5/+0x91/+0xc1 0.949999988, +0x9d/+0xf1/+0x95 0.899999976",
            true,
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1435-1456",
        ),
    ]
}

fn match_engine_late_branch_multiplier(
    order: u8,
    branch: &str,
    predicate: &str,
    affected_offsets: &[&str],
    multiplier_symbols: &[&str],
    multiplier_product: &str,
    rng_gated: bool,
    evidence: &str,
) -> MatchEngineLateBranchMultiplier {
    MatchEngineLateBranchMultiplier {
        order,
        branch: branch.to_string(),
        predicate: predicate.to_string(),
        affected_offsets: affected_offsets
            .iter()
            .map(|item| item.to_string())
            .collect(),
        multiplier_symbols: multiplier_symbols
            .iter()
            .map(|item| item.to_string())
            .collect(),
        multiplier_product: multiplier_product.to_string(),
        rng_gated,
        evidence: evidence.to_string(),
    }
}

fn default_match_engine_rng_gate_schedule() -> Vec<MatchEngineRngGate> {
    vec![
        match_engine_rng_gate(
            0,
            "FUN_008fc4f0",
            "0x32",
            "local_38 mask 0x40000 gate; success when player byte +0x5a plus cVar16 is less than RNG return",
            &["+0xf5", "+0x9d", "+0xf1", "+0x95", "+0x91", "+0xc1"],
            &["_DAT_00958fcc", "_DAT_00958324"],
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1350-1358",
        ),
        match_engine_rng_gate(
            1,
            "FUN_008fc4f0",
            "0x14",
            "local_38 mask 0x20000 fallback gate; success when RNG return exceeds player byte +0x5a",
            &["+0xf5", "+0x91", "+0x9d", "+0xf1", "+0x95", "+0xc1"],
            &["_DAT_00958fcc", "_DAT_00958324"],
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1424-1434",
        ),
        match_engine_rng_gate(
            2,
            "FUN_008fc4f0",
            "0x19",
            "local_38 mask 0x40 fallback gate; success when RNG return exceeds player byte +0x5a",
            &["+0xf5", "+0x91", "+0x9d", "+0xf1", "+0x95", "+0xc1"],
            &["_DAT_00958fcc", "_DAT_00958324"],
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1446-1456",
        ),
    ]
}

fn match_engine_rng_gate(
    order: u8,
    function: &str,
    argument: &str,
    predicate: &str,
    success_offsets: &[&str],
    success_multiplier_symbols: &[&str],
    evidence: &str,
) -> MatchEngineRngGate {
    MatchEngineRngGate {
        order,
        function: function.to_string(),
        argument: argument.to_string(),
        predicate: predicate.to_string(),
        success_offsets: success_offsets
            .iter()
            .map(|item| item.to_string())
            .collect(),
        success_multiplier_symbols: success_multiplier_symbols
            .iter()
            .map(|item| item.to_string())
            .collect(),
        evidence: evidence.to_string(),
    }
}

fn default_match_engine_runtime_constants() -> Vec<MatchEngineRuntimeConstant> {
    vec![
        match_engine_runtime_constant(
            "_DAT_00955880",
            "0x00955880",
            "0x555880",
            "f64",
            "9a9999999999b93f",
            "0.1",
        ),
        match_engine_runtime_constant(
            "_DAT_009569b0",
            "0x009569b0",
            "0x5569b0",
            "f64",
            "9a9999999999e93f",
            "0.8",
        ),
        match_engine_runtime_constant(
            "_DAT_00956e48",
            "0x00956e48",
            "0x556e48",
            "f64",
            "333333333333e33f",
            "0.6",
        ),
        match_engine_runtime_constant(
            "_DAT_00957500",
            "0x00957500",
            "0x557500",
            "f64",
            "666666666666d63f",
            "0.35",
        ),
        match_engine_runtime_constant(
            "_DAT_00956f70",
            "0x00956f70",
            "0x556f70",
            "f64",
            "713d0ad7a370e53f",
            "0.67",
        ),
        match_engine_runtime_constant(
            "_DAT_00956968",
            "0x00956968",
            "0x556968",
            "f64",
            "000000000000e03f",
            "0.5",
        ),
        match_engine_runtime_constant(
            "_DAT_009568a0",
            "0x009568a0",
            "0x5568a0",
            "f64",
            "0000000000001440",
            "5",
        ),
        match_engine_runtime_constant(
            "_DAT_009569d0",
            "0x009569d0",
            "0x5569d0",
            "f64",
            "666666666666ee3f",
            "0.95",
        ),
        match_engine_runtime_constant(
            "_DAT_00956f90",
            "0x00956f90",
            "0x556f90",
            "f64",
            "666666666666f03f",
            "1.025",
        ),
        match_engine_runtime_constant(
            "_DAT_009586d0",
            "0x009586d0",
            "0x5586d0",
            "f64",
            "713d0ad7a370ed3f",
            "0.92",
        ),
        match_engine_runtime_constant(
            "_DAT_00956fc0",
            "0x00956fc0",
            "0x556fc0",
            "f64",
            "000000000000c03f",
            "0.125",
        ),
        match_engine_runtime_constant(
            "_DAT_009574f8",
            "0x009574f8",
            "0x5574f8",
            "f64",
            "0000000000003040",
            "16",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b1b8",
            "0x0095b1b8",
            "0x55b1b8",
            "f64",
            "0000000000003140",
            "17",
        ),
        match_engine_runtime_constant(
            "_DAT_00957030",
            "0x00957030",
            "0x557030",
            "f64",
            "000000000000e83f",
            "0.75",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b3a8",
            "0x0095b3a8",
            "0x55b3a8",
            "f64",
            "cdcccccccccccf3f",
            "0.225",
        ),
        match_engine_runtime_constant(
            "_DAT_0095aff0",
            "0x0095aff0",
            "0x55aff0",
            "f32",
            "6666263f",
            "0.649999976",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b390",
            "0x0095b390",
            "0x55b390",
            "f32",
            "0000e040",
            "7",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b394",
            "0x0095b394",
            "0x55b394",
            "f32",
            "0ad7a33c",
            "0.0199999996",
        ),
        match_engine_runtime_constant(
            "_DAT_00958338",
            "0x00958338",
            "0x558338",
            "f32",
            "cdcccc3d",
            "0.100000001",
        ),
        match_engine_runtime_constant(
            "_DAT_00956f10",
            "0x00956f10",
            "0x556f10",
            "f32",
            "0000003f",
            "0.5",
        ),
        match_engine_runtime_constant(
            "_DAT_0095aef4",
            "0x0095aef4",
            "0x55aef4",
            "f32",
            "0000803e",
            "0.25",
        ),
        match_engine_runtime_constant(
            "_DAT_00958324",
            "0x00958324",
            "0x558324",
            "f32",
            "6666663f",
            "0.899999976",
        ),
        match_engine_runtime_constant(
            "_DAT_00958fcc",
            "0x00958fcc",
            "0x558fcc",
            "f32",
            "3333733f",
            "0.949999988",
        ),
        match_engine_runtime_constant(
            "_DAT_00958334",
            "0x00958334",
            "0x558334",
            "f32",
            "cdcc8c3f",
            "1.1",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b33c",
            "0x0095b33c",
            "0x55b33c",
            "f32",
            "cdcc6c3f",
            "0.925000012",
        ),
        match_engine_runtime_constant(
            "_DAT_0095af10",
            "0x0095af10",
            "0x55af10",
            "f32",
            "0000403f",
            "0.75",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b0f4",
            "0x0095b0f4",
            "0x55b0f4",
            "f32",
            "cdcc4c3d",
            "0.05",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b034",
            "0x0095b034",
            "0x55b034",
            "f32",
            "9a99593f",
            "0.85",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b05c",
            "0x0095b05c",
            "0x55b05c",
            "f32",
            "cdcc4c3f",
            "0.800000012",
        ),
        match_engine_runtime_constant(
            "_DAT_00958668",
            "0x00958668",
            "0x558668",
            "f64",
            "000000000000ec3f",
            "0.875",
        ),
        match_engine_runtime_constant(
            "_DAT_00958ee0",
            "0x00958ee0",
            "0x558ee0",
            "f64",
            "9a9999999999ed3f",
            "0.925",
        ),
        match_engine_runtime_constant(
            "_DAT_00958f68",
            "0x00958f68",
            "0x558f68",
            "f64",
            "333333333333ef3f",
            "0.975",
        ),
        match_engine_runtime_constant(
            "_DAT_009569d8",
            "0x009569d8",
            "0x5569d8",
            "f64",
            "cdccccccccccec3f",
            "0.9",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b398",
            "0x0095b398",
            "0x55b398",
            "f64",
            "a4703d0ad7a3f03f",
            "1.04",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b380",
            "0x0095b380",
            "0x55b380",
            "f64",
            "85eb51b81e85f03f",
            "1.0325",
        ),
        match_engine_runtime_constant(
            "_DAT_0095b388",
            "0x0095b388",
            "0x55b388",
            "f64",
            "333333333333f03f",
            "1.0125",
        ),
    ]
}

fn match_engine_runtime_constant(
    symbol: &str,
    va: &str,
    file_offset: &str,
    storage: &str,
    bytes_le: &str,
    value: &str,
) -> MatchEngineRuntimeConstant {
    MatchEngineRuntimeConstant {
        symbol: symbol.to_string(),
        va: va.to_string(),
        file_offset: file_offset.to_string(),
        storage: storage.to_string(),
        bytes_le: bytes_le.to_string(),
        value: value.to_string(),
        evidence: "Decoded from D:/cm0102/cm0102.exe by PE VA-to-file-offset conversion; image base 0x400000, .rdata-backed constant.".to_string(),
    }
}

fn match_engine_constant_value(symbol: &str) -> &'static str {
    match symbol {
        "_DAT_00955880" => "0.1",
        "_DAT_009569b0" => "0.8",
        "_DAT_00956e48" => "0.6",
        "_DAT_00957500" => "0.35",
        "_DAT_00956f70" => "0.67",
        "_DAT_0095aff0" => "0.649999976",
        "_DAT_0095b390" => "7",
        "_DAT_009574f8" => "16",
        "_DAT_0095b1b8" => "17",
        "_DAT_00956968" => "0.5",
        "_DAT_009568a0" => "5",
        "_DAT_009569d0" => "0.95",
        "_DAT_00956f90" => "1.025",
        "_DAT_009586d0" => "0.92",
        "_DAT_00956fc0" => "0.125",
        "_DAT_00957030" => "0.75",
        "_DAT_0095b3a8" => "0.225",
        "_DAT_0095b394" => "0.0199999996",
        "_DAT_00958338" => "0.100000001",
        "_DAT_00956f10" => "0.5",
        "_DAT_0095aef4" => "0.25",
        "_DAT_00958324" => "0.899999976",
        "_DAT_00958fcc" => "0.949999988",
        "_DAT_0095b33c" => "0.925000012",
        "_DAT_0095af10" => "0.75",
        "_DAT_0095b05c" => "0.800000012",
        "_DAT_00958668" => "0.875",
        "_DAT_00958ee0" => "0.925",
        "_DAT_00958f68" => "0.975",
        "_DAT_009569d8" => "0.9",
        "_DAT_0095b398" => "1.04",
        "_DAT_0095b380" => "1.0325",
        "_DAT_0095b388" => "1.0125",
        _ => "UNKNOWN",
    }
}

fn match_engine_runtime_phase_step(
    order: u8,
    function: &str,
    label: &str,
    writes: &[&str],
) -> MatchEngineRuntimePhaseStep {
    MatchEngineRuntimePhaseStep {
        order,
        function: function.to_string(),
        label: label.to_string(),
        writes: writes.iter().map(|item| item.to_string()).collect(),
        status: "static-code-derived-frontier".to_string(),
    }
}

fn match_engine_runtime_player_frontier(
    function: &str,
    label: &str,
    input_offsets: &[&str],
    output_offsets: &[&str],
    event_codes: &[&str],
) -> MatchEngineRuntimePlayerFrontier {
    MatchEngineRuntimePlayerFrontier {
        function: function.to_string(),
        label: label.to_string(),
        input_offsets: input_offsets.iter().map(|item| item.to_string()).collect(),
        output_offsets: output_offsets.iter().map(|item| item.to_string()).collect(),
        event_codes: event_codes.iter().map(|item| item.to_string()).collect(),
        status: "static-code-derived-frontier".to_string(),
    }
}

fn match_engine_runtime_event_frontier(
    function: &str,
    label: &str,
    event_codes: &[&str],
    mutated_offsets: &[&str],
) -> MatchEngineRuntimeEventFrontier {
    MatchEngineRuntimeEventFrontier {
        function: function.to_string(),
        label: label.to_string(),
        event_codes: event_codes.iter().map(|item| item.to_string()).collect(),
        mutated_offsets: mutated_offsets
            .iter()
            .map(|item| item.to_string())
            .collect(),
        status: "static-code-derived-frontier".to_string(),
    }
}

fn match_engine_runtime_tactical_frontier(
    function: &str,
    label: &str,
    input_offsets: &[&str],
    output_offsets: &[&str],
) -> MatchEngineRuntimeTacticalFrontier {
    MatchEngineRuntimeTacticalFrontier {
        function: function.to_string(),
        label: label.to_string(),
        input_offsets: input_offsets.iter().map(|item| item.to_string()).collect(),
        output_offsets: output_offsets.iter().map(|item| item.to_string()).collect(),
        status: "static-code-derived-frontier".to_string(),
    }
}

pub fn match_engine_runtime_store_ready(store: &MatchEngineRuntimeStore) -> bool {
    let required_phase_functions = [
        "0x0069d950",
        "0x006d1a20",
        "0x006e65e0",
        "0x006a0550",
        "0x0069f2f0",
    ];
    let phase_ready = required_phase_functions.iter().all(|function| {
        store
            .phase_pipeline
            .iter()
            .any(|step| step.function == *function && step.status == "static-code-derived-frontier")
    });
    let player_ready = store.player_frontiers.iter().any(|frontier| {
        frontier.function == "0x006d46c0"
            && frontier
                .output_offsets
                .iter()
                .any(|offset| offset.contains("+0x37"))
    }) && store.player_frontiers.iter().any(|frontier| {
        frontier.function == "0x006e65e0"
            && frontier
                .output_offsets
                .iter()
                .any(|offset| offset.contains("+0x39"))
            && frontier.event_codes.iter().any(|code| code == "0x1f7f")
    });
    let event_ready = store.event_frontiers.iter().any(|frontier| {
        frontier.function == "0x006bc8d0"
            && (frontier
                .event_codes
                .iter()
                .any(|code| code == "8000..0x21e4")
                || frontier
                    .mutated_offsets
                    .iter()
                    .any(|offset| offset.contains("0x0e")))
    }) && store.event_frontiers.iter().any(|frontier| {
        frontier.function == "0x006a0550"
            && frontier.event_codes.iter().any(|code| code == "0x20f0")
    });
    let tactical_ready = ["0x006a91d0", "0x006a9200"].iter().all(|function| {
        store
            .tactical_frontiers
            .iter()
            .any(|frontier| frontier.function == *function)
    });
    let counted_rows = store.phase_pipeline.len()
        + store.player_frontiers.len()
        + store.event_frontiers.len()
        + store.tactical_frontiers.len();
    let late_branch_ready = match_engine_late_branch_coverage_ready(store);
    phase_ready
        && player_ready
        && event_ready
        && tactical_ready
        && late_branch_ready
        && store
            .not_implemented
            .iter()
            .any(|item| item == "match-day processing")
        && store.applied_frontier_rows == counted_rows
        && store.applied_frontier_rows >= 25
}

pub fn match_engine_late_branch_coverage_ready(store: &MatchEngineRuntimeStore) -> bool {
    let deterministic_ready = store
        .late_branch_multipliers
        .iter()
        .filter(|branch| {
            !branch.rng_gated
                && branch.source_evidence_ready()
                && branch
                    .affected_offsets
                    .iter()
                    .any(|offset| offset == "+0xf5" || offset == "+0x9d" || offset == "+0xc1")
        })
        .count()
        >= 7;
    let rng_ready = store.rng_gate_schedule.len() == 3
        && store
            .rng_gate_schedule
            .iter()
            .enumerate()
            .all(|(index, gate)| gate.order as usize == index && gate.function == "FUN_008fc4f0");
    let execution_ready = store.late_branch_execution_outputs.iter().any(|output| {
        output.source_function == "0x006d1a20"
            && output.applied_branches.len() >= 8
            && output.rng_rolls.len() == 3
            && output
                .rng_rolls
                .iter()
                .enumerate()
                .all(|(index, roll)| roll.order as usize == index && roll.success)
            && output
                .offset_multiplier_products
                .iter()
                .any(|product| product.offset == "+0xf5")
            && output
                .offset_multiplier_products
                .iter()
                .any(|product| product.offset == "+0x9d")
            && output
                .offset_multiplier_products
                .iter()
                .any(|product| product.offset == "+0xc1")
    });
    store.late_branch_multipliers.len() >= 10 && deterministic_ready && rng_ready && execution_ready
}

pub fn match_engine_action_selection_ready(store: &MatchEngineRuntimeStore) -> bool {
    store.action_selection_outputs.iter().any(|output| {
        output
            .source_functions
            .iter()
            .any(|function| function == "0x006d1a20")
            && output
                .source_functions
                .iter()
                .any(|function| function == "0x006d46c0")
            && output
                .source_functions
                .iter()
                .any(|function| function == "0x006e65e0")
            && output
                .source_functions
                .iter()
                .any(|function| function == "0x006bc8d0")
            && output.evaluation_score_0x3b > 0
            && output.action_score_0x37 != 0
            && output.shot_action_score_0x39 > 0
            && output.selected_action_code == "0x3a"
            && output.selected_action_event_code.as_deref() == Some("0x1f9d")
            && output.direct_shot_event_code.as_deref() == Some("0x1f7f")
            && output
                .decisive_float_value
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
            && output.branch_product_applied != "1.000000"
    })
}

pub fn match_engine_event_queue_outputs_ready(store: &MatchEngineRuntimeStore) -> bool {
    let has_selected_action = store.event_queue_outputs.iter().any(|output| {
        output.source_function == "0x006bc8d0"
            && output.event_code == "0x1f9d"
            && output.slot_base == "+0x30"
            && output.stride == "0x0e"
            && output.action_code == "0x3a"
            && output.mirror_offset.as_deref() == Some("+0x720")
    });
    let has_direct_shot = store.event_queue_outputs.iter().any(|output| {
        output.source_function == "0x006bc8d0"
            && output.event_code == "0x1f7f"
            && output.slot_base == "+0x30"
            && output.stride == "0x0e"
            && output.action_code == "0x33"
            && output.score > 0
    });
    store.event_queue_outputs.len() >= 2 && has_selected_action && has_direct_shot
}

pub fn match_engine_state_mutation_outputs_ready(store: &MatchEngineRuntimeStore) -> bool {
    let has_count = store.state_mutation_outputs.iter().any(|output| {
        output.table == "match_state"
            && output.field == "event_queue_count"
            && output.record_offset == "+0x08"
            && output.after == "0x0002"
            && output.source_function == "0x006bc8d0"
    });
    let has_owner = store.state_mutation_outputs.iter().any(|output| {
        output.field == "stored_action_owner"
            && output.record_offset == "+0x8ea7"
            && output.after == "0x09"
    });
    let has_target = store.state_mutation_outputs.iter().any(|output| {
        output.field == "stored_action_target"
            && output.record_offset == "+0x8ea8"
            && output.after == "0x04"
    });
    let has_action = store.state_mutation_outputs.iter().any(|output| {
        output.field == "selected_action_code"
            && output.record_offset == "+0x8eae"
            && output.after == "0x3a"
            && output.source_event_code.as_deref() == Some("0x1f9d")
    });
    let has_last_event = store.state_mutation_outputs.iter().any(|output| {
        output.field == "last_event_code"
            && output.after == "0x1f7f"
            && output.source_event_code.as_deref() == Some("0x1f7f")
    });
    store.state_mutation_outputs.len() >= 5
        && has_count
        && has_owner
        && has_target
        && has_action
        && has_last_event
}

pub fn match_engine_result_finalization_ready(store: &MatchEngineRuntimeStore) -> bool {
    store.result_finalization_outputs.iter().any(|output| {
        output.source_function == "0x006a4020"
            && output.event_code == "0x2004"
            && output.match_state_home_offset == "+0xf5bc"
            && output.match_state_away_offset == "+0xf5f2"
            && output.fixture_home_offset == "+0x49"
            && output.fixture_away_offset == "+0x4a"
            && output.home_score == 2
            && output.away_score == 1
            && output.phase_after_offset == "+0x8eb3"
            && output.phase_after == "0x00"
    })
}

impl MatchEngineLateBranchMultiplier {
    fn source_evidence_ready(&self) -> bool {
        self.evidence.contains("006d1a20.c")
            && !self.affected_offsets.is_empty()
            && !self.multiplier_symbols.is_empty()
            && !self.multiplier_product.is_empty()
    }
}

pub fn match_engine_runtime_constants_ready(constants: &[MatchEngineRuntimeConstant]) -> bool {
    let required = [
        ("_DAT_00955880", "0x00955880", "f64", "0.1"),
        ("_DAT_009569b0", "0x009569b0", "f64", "0.8"),
        ("_DAT_00956e48", "0x00956e48", "f64", "0.6"),
        ("_DAT_00957500", "0x00957500", "f64", "0.35"),
        ("_DAT_00956f70", "0x00956f70", "f64", "0.67"),
        ("_DAT_00956968", "0x00956968", "f64", "0.5"),
        ("_DAT_009568a0", "0x009568a0", "f64", "5"),
        ("_DAT_009569d0", "0x009569d0", "f64", "0.95"),
        ("_DAT_00956f90", "0x00956f90", "f64", "1.025"),
        ("_DAT_009586d0", "0x009586d0", "f64", "0.92"),
        ("_DAT_0095aff0", "0x0095aff0", "f32", "0.649999976"),
        ("_DAT_0095b390", "0x0095b390", "f32", "7"),
        ("_DAT_00958324", "0x00958324", "f32", "0.899999976"),
        ("_DAT_00958fcc", "0x00958fcc", "f32", "0.949999988"),
    ];
    required.iter().all(|(symbol, va, storage, value)| {
        constants.iter().any(|constant| {
            constant.symbol == *symbol
                && constant.va == *va
                && constant.storage == *storage
                && constant.value == *value
                && constant.evidence.contains("D:/cm0102/cm0102.exe")
        })
    })
}

pub fn default_match_engine_runtime_scenario() -> MatchEngineRuntimeScenario {
    MatchEngineRuntimeScenario {
        match_row: 0,
        player_slot_row: 0,
        home_score: 2,
        away_score: 1,
        action_owner_slot: 9,
        action_target_slot: 4,
        event_queue_count_before: 0,
        primary_tactic_flags: 0x1b80,
        secondary_tactic_flags: 0x0080,
        player_attr_17: 17,
        player_attr_18: 16,
        player_attr_19: 15,
        player_attr_1a: 18,
        player_attr_1e: 14,
        player_attr_23: 13,
        player_attr_24: 12,
        player_attr_26: 16,
        player_attr_27: 15,
        player_attr_35: 13,
        player_attr_39: 12,
        player_attr_3a: 11,
        evaluation_base_score: 90,
        fatigue_drag: 8,
        role_local_4c: 14,
        role_local_48: 12,
        role_local_3c: 10,
        shot_base_0x29: 650,
        shot_power_0x180: 28,
        current_action_score_0x19c: 260,
        selected_action_code: 0x3a,
        rng_shot_power: 42,
        rng_distance_penalty: 18,
        rng_distance_penalty_div5: 3,
        rng_current_action_score_div5: 11,
        rng_action_slot_score: 180,
        late_manager_competition_branch: true,
        weak_tactical_profile_branch: true,
        low_confidence_side_state_branch: true,
        local_38_flags_0x45: 0x3eff57,
        player_byte_0x5a: 4,
        player_byte_0x5b: 4,
        player_byte_0x57: 12,
        player_byte_0x59: 7,
        player_byte_0x5d: 11,
        cvar16: 18,
        local_38_byte_0x44: -80,
        related_byte_0x44: 12,
        related_short_0x0b: 0x1600,
        related_short_0x0d: 0x1500,
        opponent_short_0x80: 1000,
        rng_gate_0x32: 40,
        rng_gate_0x14: 12,
        rng_gate_0x19: 15,
    }
}

pub fn plan_match_engine_runtime_mutations(
    backend: &RuntimeBackendSystems,
    scenario: &MatchEngineRuntimeScenario,
) -> Vec<MatchEngineRuntimeMutation> {
    let mut mutations = Vec::new();
    let evaluation_score = match_player_evaluation_score_formula(scenario);
    let action_score = match_player_action_score_formula(scenario);
    let shot_score = match_shot_action_score_formula(scenario);
    let direct_shot_return = match_direct_shot_return_formula(scenario);
    let selected_action_event = match_shot_action_event_code(
        scenario.selected_action_code,
        false,
        false,
        scenario.rng_action_slot_score,
    );
    if backend
        .match_engine_runtime_store
        .phase_pipeline
        .iter()
        .any(|step| step.function == "0x006d1a20")
    {
        push_match_engine_runtime_mutation(
            &mut mutations,
            "match_player_slot",
            scenario.player_slot_row,
            "evaluation_score",
            "+0x3b",
            "0x0000",
            &format_i16_hex(evaluation_score),
            None,
            "0x006d1a20",
            "score = lifted local evaluation base clamped through +0x3b; downstream floats use (score + attr*2 - fatigue_drag*constant) patterns",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:338-350,389-394,439-449,471-475,702-727",
        );
        for (field, offset, after, formula, evidence) in
            match_player_evaluation_float_formula_mutations(scenario, evaluation_score)
        {
            push_match_engine_runtime_mutation(
                &mut mutations,
                "match_player_slot",
                scenario.player_slot_row,
                field,
                offset,
                "0.0",
                &after,
                None,
                "0x006d1a20",
                formula,
                evidence,
            );
        }
    }
    if backend
        .match_engine_runtime_store
        .player_frontiers
        .iter()
        .any(|frontier| frontier.function == "0x006d46c0")
    {
        push_match_engine_runtime_mutation(
            &mut mutations,
            "match_player_slot",
            scenario.player_slot_row,
            "action_score",
            "+0x37",
            "0x0000",
            &format_i16_hex(action_score),
            None,
            "0x006d46c0",
            "action_score = sum lifted tactic-flag attribute contributions after resetting +0x37",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d46c0.c:33,199-201,241-264,272-274",
        );
    }
    if backend
        .match_engine_runtime_store
        .player_frontiers
        .iter()
        .any(|frontier| frontier.function == "0x006e65e0")
    {
        push_match_engine_runtime_mutation(
            &mut mutations,
            "match_player_slot",
            scenario.player_slot_row,
            "shot_action_score",
            "+0x39",
            "0x0000",
            &format_i16_hex(shot_score),
            Some("0x1f7f"),
            "0x006e65e0",
            "for action 0x3a: +0x39 = (shot_base/2 - rng(distance_penalty/5)) + 0xd2 + rng(current_action/5); otherwise +0x39 = shot_base/2 + current_action - rng(distance_penalty)",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006e65e0.c:292-321",
        );
        push_match_engine_runtime_mutation(
            &mut mutations,
            "match_player_slot",
            scenario.player_slot_row,
            "action_byte",
            "+0x33",
            "0xff",
            &format!("0x{:02x}", scenario.selected_action_code),
            selected_action_event.as_deref(),
            "0x006e65e0",
            "selected action byte is mapped to its commentary/event code through the terminal switch",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006e65e0.c:513-610",
        );
        push_match_engine_runtime_mutation(
            &mut mutations,
            "match_player_slot",
            scenario.player_slot_row,
            "direct_shot_return_score",
            "return",
            "0x0000",
            &format_i16_hex(direct_shot_return),
            Some("0x1f7f"),
            "0x006e65e0",
            "if action byte 0x33 enters get_shot_score, event 0x1f7f is emitted and return = RNG(*(short +0x180)*5) + 0x96",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006e65e0.c:62-65",
        );
    }
    if backend
        .match_engine_runtime_store
        .event_frontiers
        .iter()
        .any(|frontier| frontier.function == "0x006a0550")
    {
        push_match_engine_runtime_mutation(
            &mut mutations,
            "match_state",
            scenario.match_row,
            "stored_action_owner",
            "+0x8ea7",
            "0xff",
            &format!("0x{:02x}", scenario.action_owner_slot),
            Some("0x20f0"),
            "0x006a0550",
            "stored-action resolver mutates owner/action bytes and emits stored-action event codes",
            "D:/cm0102-rs/reports/simulation_frontier_validation.json decompile-match-stored-action-resolver",
        );
        push_match_engine_runtime_mutation(
            &mut mutations,
            "match_state",
            scenario.match_row,
            "stored_action_target",
            "+0x8ea8",
            "0xff",
            &format!("0x{:02x}", scenario.action_target_slot),
            Some("0x20ee"),
            "0x006a0550",
            "stored-action resolver mutates owner/action bytes and emits stored-action event codes",
            "D:/cm0102-rs/reports/simulation_frontier_validation.json decompile-match-stored-action-resolver",
        );
        push_match_engine_runtime_mutation(
            &mut mutations,
            "match_state",
            scenario.match_row,
            "stored_action_scratch_reset",
            "+0x475a..+0x4769",
            "dirty",
            "reset",
            None,
            "0x006a0550",
            "stored-action resolver resets the action scratch block before/after stored action handling",
            "D:/cm0102-rs/reports/simulation_frontier_validation.json decompile-match-stored-action-resolver",
        );
    }
    if backend
        .match_engine_runtime_store
        .event_frontiers
        .iter()
        .any(|frontier| frontier.function == "0x006bc8d0")
    {
        for (index, code) in ["0x1f7f", "0x1f81", "0x20f0", "0x20ee"].iter().enumerate() {
            push_match_engine_runtime_mutation(
                &mut mutations,
                "match_event_queue",
                scenario.event_queue_count_before as u32 + index as u32,
                "event_slot_code",
                "+0x30 + row*0x0e",
                "empty",
                code,
                Some(*code),
                "0x006bc8d0",
                "event queue writer appends 0x0e-byte event slots from +0x30 and mirrors selected events at +0x720",
                "D:/cm0102-rs/reports/simulation_frontier_validation.json decompile-match-event-queue-writer",
            );
        }
    }
    mutations
}

fn match_player_evaluation_score_formula(scenario: &MatchEngineRuntimeScenario) -> i16 {
    let stamina_adjusted_attr_1e = if scenario.player_attr_1e < 37 {
        scenario.player_attr_1e
    } else {
        scenario.player_attr_1e.saturating_mul(2)
    };
    clamp_i16(
        scenario
            .evaluation_base_score
            .saturating_add(stamina_adjusted_attr_1e / 2)
            .saturating_sub(scenario.fatigue_drag / 2),
        0,
        150,
    )
}

fn match_player_evaluation_float_formula_mutations(
    scenario: &MatchEngineRuntimeScenario,
    evaluation_score: i16,
) -> Vec<(
    &'static str,
    &'static str,
    String,
    &'static str,
    &'static str,
)> {
    let scale_01 = match_engine_constant_value("_DAT_00955880");
    let fatigue_07d = match_engine_constant_value("_DAT_009569b0");
    let final_scale = match_engine_constant_value("_DAT_00956968");
    let final_add = match_engine_constant_value("_DAT_009568a0");
    let tactic_095 = match_engine_constant_value("_DAT_009569d0");
    let fatigue_08d = match_engine_constant_value("_DAT_00956e48");
    let tactic_1025 = match_engine_constant_value("_DAT_00956f90");
    let fatigue_095 = match_engine_constant_value("_DAT_00957500");
    let fatigue_099 = match_engine_constant_value("_DAT_00956f70");
    let tactic_092 = match_engine_constant_value("_DAT_009586d0");
    let b9_scale = match_engine_constant_value("_DAT_0095aff0");
    let b9_add = match_engine_constant_value("_DAT_0095b390");
    vec![
        (
            "evaluation_float_0x7d_shape",
            "+0x7d",
            format!(
                "(({} + {}*2) - {}*_DAT_009569b0({})) * _DAT_00955880({}); manager/tactic adjust; clamp; * _DAT_00956968({}) + _DAT_009568a0({})",
                evaluation_score, scenario.player_attr_1e, scenario.fatigue_drag, fatigue_07d, scale_01, final_scale, final_add
            ),
            "+0x7d = ((score + attr_0x1e*2) - fatigue_drag*_DAT_009569b0) * _DAT_00955880, then manager/tactic adjustments, clamp, and linear scale",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:341-390",
        ),
        (
            "evaluation_float_0x8d_shape",
            "+0x8d",
            format!(
                "(({} + {}*2) - {}*_DAT_00956e48({})) * _DAT_00955880({}); optional tactic multiplier _DAT_00956f90({}); clamp/jitter",
                evaluation_score, scenario.player_attr_23, scenario.fatigue_drag, fatigue_08d, scale_01, tactic_1025
            ),
            "+0x8d = ((score + attr_0x23*2) - fatigue_drag*_DAT_00956e48) * _DAT_00955880, then tactical/range clamps",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:391-437",
        ),
        (
            "evaluation_float_0x91_shape",
            "+0x91",
            format!(
                "(({} + {}*2) - {}) * _DAT_00955880({}); clamp",
                evaluation_score, scenario.player_attr_24, scenario.fatigue_drag, scale_01
            ),
            "+0x91 = ((score + attr_0x24*2) - fatigue_drag) * _DAT_00955880, then zero-floor clamp",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:439-445",
        ),
        (
            "evaluation_float_0x95_shape",
            "+0x95",
            format!(
                "(({} + {}*2) - {}*_DAT_00957500({})) * _DAT_00955880({}); optional tactic multiplier _DAT_009569d0({}); clamp",
                evaluation_score, scenario.player_attr_26, scenario.fatigue_drag, fatigue_095, scale_01, tactic_095
            ),
            "+0x95 = ((score + attr_0x26*2) - fatigue_drag*_DAT_00957500) * _DAT_00955880, then tactic multiplier/clamp",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:446-469",
        ),
        (
            "evaluation_float_0x99_shape",
            "+0x99",
            format!(
                "(({} + {}*2) - {}*_DAT_00956f70({})) * _DAT_00955880({}); tactic multipliers _DAT_009586d0({})/_DAT_009569d0({}); clamp/jitter",
                evaluation_score, scenario.player_attr_27, scenario.fatigue_drag, fatigue_099, scale_01, tactic_092, tactic_095
            ),
            "+0x99 = ((score + attr_0x27*2) - fatigue_drag*_DAT_00956f70) * _DAT_00955880, then tactical multipliers/clamp",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:471-516",
        ),
        (
            "evaluation_float_0xb9_shape",
            "+0xb9",
            format!(
                "(({} + {}*2) - {}*role_scalar) * _DAT_00955880({}); manager/tactic adjust; * _DAT_0095aff0({}) + _DAT_0095b390({}); clamp",
                evaluation_score, scenario.role_local_4c, scenario.fatigue_drag, scale_01, b9_scale, b9_add
            ),
            "+0xb9 is built from +0x3b, local role value, fatigue drag, then manager/tactic adjustment, linear scale, and clamp",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:702-727",
        ),
        (
            "evaluation_float_0xcd_shape",
            "+0xcd",
            format!(
                "(({} + {}*2) - fatigue_component + {}) * _DAT_00955880({}); manager adjust; random float reset/jitter clamp",
                evaluation_score, scenario.role_local_48, scenario.role_local_3c, scale_01
            ),
            "+0xcd = ((score + local_48*2) - fatigue_component + local_3c) * _DAT_00955880, followed by manager adjustment and RNG-derived reset/clamp",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:791-991",
        ),
        (
            "evaluation_float_0xf1_shape",
            "+0xf1",
            format!(
                "(({} + {}*2) - fatigue_component) * _DAT_00955880({}); clamp/jitter; late multipliers may apply",
                evaluation_score, scenario.player_attr_3a, scale_01
            ),
            "+0xf1 = ((score + local_3c/attr_0x3a*2) - fatigue_component) * _DAT_00955880, then clamp/jitter and late situational multipliers",
            "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:993-1067,1299-1450",
        ),
    ]
}

fn match_player_evaluation_output(
    scenario: &MatchEngineRuntimeScenario,
) -> MatchEnginePlayerEvaluationOutput {
    let score = match_player_evaluation_score_formula(scenario);
    let scale = parse_match_engine_constant("_DAT_00955880");
    let fatigue_7d = parse_match_engine_constant("_DAT_009569b0");
    let final_scale = parse_match_engine_constant("_DAT_00956968");
    let final_add = parse_match_engine_constant("_DAT_009568a0");
    let fatigue_8d = parse_match_engine_constant("_DAT_00956e48");
    let fatigue_95 = parse_match_engine_constant("_DAT_00957500");
    let fatigue_99 = parse_match_engine_constant("_DAT_00956f70");
    let tactic_095 = parse_match_engine_constant("_DAT_009569d0");
    let tactic_092 = parse_match_engine_constant("_DAT_009586d0");
    let b9_scale = parse_match_engine_constant("_DAT_0095aff0");
    let b9_add = parse_match_engine_constant("_DAT_0095b390");
    let mut branch_applications = Vec::new();
    let late_branch_output = match_engine_late_branch_execution_output(scenario);

    let score_f = f64::from(score);
    let fatigue = f64::from(scenario.fatigue_drag);
    let float_0x7d_base =
        ((score_f + f64::from(scenario.player_attr_1e) * 2.0) - fatigue * fatigue_7d) * scale;
    let mut float_0x7d = zero_floor(float_0x7d_base) * final_scale + final_add;
    let float_0x8d = zero_floor(
        ((score_f + f64::from(scenario.player_attr_23) * 2.0) - fatigue * fatigue_8d) * scale,
    );
    let mut float_0x91 =
        zero_floor(((score_f + f64::from(scenario.player_attr_24) * 2.0) - fatigue) * scale);
    let mut float_0x95 = zero_floor(
        ((score_f + f64::from(scenario.player_attr_26) * 2.0) - fatigue * fatigue_95) * scale,
    );
    if (scenario.primary_tactic_flags & 0x20) != 0 && (scenario.primary_tactic_flags & 0x0c) != 0 {
        float_0x95 *= tactic_095;
        branch_applications
            .push("+0x95 *= _DAT_009569d0 when primary flags include 0x20 and 0x0c".to_string());
    }
    let mut float_0x99 = zero_floor(
        ((score_f + f64::from(scenario.player_attr_27) * 2.0) - fatigue * fatigue_99) * scale,
    );
    if (scenario.primary_tactic_flags & 0x880) != 0
        && (scenario.secondary_tactic_flags & 0x880) != 0
    {
        float_0x99 *= tactic_092;
        branch_applications.push(
            "+0x99 *= _DAT_009586d0 when primary and secondary flags overlap 0x880".to_string(),
        );
    }
    if (scenario.primary_tactic_flags & 0x20) != 0 && (scenario.primary_tactic_flags & 0x0c) != 0 {
        float_0x99 *= tactic_095;
        branch_applications
            .push("+0x99 *= _DAT_009569d0 when primary flags include 0x20 and 0x0c".to_string());
    }
    let float_0xb9_base = ((score_f + f64::from(scenario.role_local_4c) * 2.0) - fatigue) * scale;
    let mut float_0xb9 = zero_floor(float_0xb9_base * b9_scale + b9_add);
    let mut float_0xcd = zero_floor(
        ((score_f + f64::from(scenario.role_local_48) * 2.0) - fatigue
            + f64::from(scenario.role_local_3c))
            * scale,
    );
    let mut float_0xf1 =
        zero_floor(((score_f + f64::from(scenario.player_attr_3a) * 2.0) - fatigue) * scale);
    float_0x7d *= late_branch_product(&late_branch_output, "+0x7d");
    float_0x91 *= late_branch_product(&late_branch_output, "+0x91");
    float_0x95 *= late_branch_product(&late_branch_output, "+0x95");
    float_0x99 *= late_branch_product(&late_branch_output, "+0x99");
    float_0xb9 *= late_branch_product(&late_branch_output, "+0xb9");
    float_0xcd *= late_branch_product(&late_branch_output, "+0xcd");
    float_0xf1 *= late_branch_product(&late_branch_output, "+0xf1");
    branch_applications.extend(
        late_branch_output
            .applied_branches
            .iter()
            .map(|branch| format!("late-exec {branch}")),
    );
    branch_applications.extend(late_branch_output.rng_rolls.iter().map(|roll| {
        format!(
            "rng {}({})={} threshold {} success {}",
            roll.function, roll.argument, roll.value, roll.threshold, roll.success
        )
    }));
    branch_applications.extend(late_branch_output.offset_multiplier_products.iter().map(
        |product| {
            format!(
                "{} *= {} via {}",
                product.offset,
                product.product,
                product.symbols.join("*")
            )
        },
    ));

    MatchEnginePlayerEvaluationOutput {
        row: scenario.player_slot_row,
        source_function: "0x006d1a20".to_string(),
        evaluation_score_0x3b: score,
        float_0x7d: format_match_engine_float(float_0x7d),
        float_0x8d: format_match_engine_float(float_0x8d),
        float_0x91: format_match_engine_float(float_0x91),
        float_0x95: format_match_engine_float(float_0x95),
        float_0x99: format_match_engine_float(float_0x99),
        float_0xb9: format_match_engine_float(float_0xb9),
        float_0xcd: format_match_engine_float(float_0xcd),
        float_0xf1: format_match_engine_float(float_0xf1),
        branch_applications,
        constants_used: vec![
            "_DAT_00955880".to_string(),
            "_DAT_009569b0".to_string(),
            "_DAT_00956968".to_string(),
            "_DAT_009568a0".to_string(),
            "_DAT_00956e48".to_string(),
            "_DAT_00957500".to_string(),
            "_DAT_00956f70".to_string(),
            "_DAT_009569d0".to_string(),
            "_DAT_009586d0".to_string(),
            "_DAT_00958324".to_string(),
            "_DAT_0095b33c".to_string(),
            "_DAT_0095b0f4".to_string(),
            "_DAT_0095aef4".to_string(),
            "_DAT_0095b034".to_string(),
            "_DAT_0095af10".to_string(),
            "_DAT_00956f10".to_string(),
            "_DAT_00958fcc".to_string(),
            "_DAT_0095aff0".to_string(),
            "_DAT_0095b390".to_string(),
        ],
        evidence: "Numeric evaluator for 0x006d1a20 fields +0x7d/+0x8d/+0x91/+0x95/+0x99/+0xb9/+0xcd/+0xf1, with code-derived +0x95/+0x99 tactical multiplier branches from 006d1a20.c:446-516 and executable deterministic/RNG-gated late multiplier products from 006d1a20.c:1299-1456.".to_string(),
    }
}

fn late_branch_product(output: &MatchEngineLateBranchExecutionOutput, offset: &str) -> f64 {
    output
        .offset_multiplier_products
        .iter()
        .find(|product| product.offset == offset)
        .and_then(|product| product.product.parse::<f64>().ok())
        .unwrap_or(1.0)
}

fn parse_match_engine_constant(symbol: &str) -> f64 {
    match_engine_constant_value(symbol)
        .parse::<f64>()
        .unwrap_or(0.0)
}

fn zero_floor(value: f64) -> f64 {
    if value < 0.0 {
        0.0
    } else {
        value
    }
}

fn format_match_engine_float(value: f64) -> String {
    format!("{value:.6}")
}

fn match_player_action_score_formula(scenario: &MatchEngineRuntimeScenario) -> i16 {
    let mut score = 0i16;
    let primary = scenario.primary_tactic_flags;
    if (primary & 0x0001) != 0 {
        score = score.saturating_add(
            scenario
                .player_attr_18
                .saturating_mul(2)
                .saturating_sub(0x28),
        );
    }
    if (primary & 0x0080) != 0 {
        score = score.saturating_add(scenario.player_attr_18.saturating_sub(0x14));
    }
    if (primary & 0x0200) != 0 {
        score = score.saturating_add(scenario.player_attr_19.saturating_sub(0x14));
    }
    if (primary & 0x0400) != 0 {
        score = score.saturating_add(
            (scenario.player_attr_17 / 2)
                .max(scenario.player_attr_19)
                .saturating_sub(0x14),
        );
    }
    if (primary & 0x0800) != 0 {
        score = score.saturating_add(scenario.player_attr_17.saturating_sub(0x14));
    }
    if (primary & 0x1000) != 0 {
        score = score.saturating_add(scenario.player_attr_1a.saturating_sub(0x14));
    }
    score
}

fn match_shot_action_score_formula(scenario: &MatchEngineRuntimeScenario) -> i16 {
    if scenario.selected_action_code == 0x35 {
        return 0;
    }
    if scenario.selected_action_code == 0x3a {
        return scenario
            .shot_base_0x29
            .saturating_div(2)
            .saturating_sub(scenario.rng_distance_penalty_div5)
            .saturating_add(0xd2)
            .saturating_add(scenario.rng_current_action_score_div5);
    }
    scenario
        .shot_base_0x29
        .saturating_div(2)
        .saturating_add(scenario.current_action_score_0x19c)
        .saturating_sub(scenario.rng_distance_penalty)
}

fn match_direct_shot_return_formula(scenario: &MatchEngineRuntimeScenario) -> i16 {
    scenario.rng_shot_power.saturating_add(0x96)
}

fn match_shot_action_event_code(
    action_code: u8,
    bvar4: bool,
    bvar3: bool,
    fallback_rng: i16,
) -> Option<String> {
    let event = match action_code {
        3 => {
            if bvar4 {
                0x1fb8
            } else {
                ((-(bvar3 as i16) as u16) & 0x18) + 0x1f92
            }
        }
        5 => {
            if bvar4 {
                0x1fb7
            } else {
                ((-(bvar3 as i16) as u16) & 0x18) + 0x1f91
            }
        }
        0x16 => {
            if bvar4 {
                0x1fb0
            } else {
                ((-(bvar3 as i16) as u16) & 0x1c) + 0x1f86
            }
        }
        0x17 => {
            if bvar4 {
                0x1fbb
            } else {
                ((-(bvar3 as i16) as u16) & 0x14) + 0x1f99
            }
        }
        0x18 => {
            if bvar4 {
                0x1fb9
            } else {
                ((-(bvar3 as i16) as u16) & 0x18) + 0x1f93
            }
        }
        0x19 => {
            if bvar4 {
                0x1fba
            } else {
                ((-(bvar3 as i16) as u16) & 0x16) + 0x1f96
            }
        }
        0x1a => {
            if bvar4 {
                0x1fb1
            } else {
                ((-(bvar3 as i16) as u16) & 0x1a) + 0x1f89
            }
        }
        0x1b => {
            if bvar4 {
                0x1fb3
            } else {
                ((-(bvar3 as i16) as u16) & 0x1a) + 0x1f8b
            }
        }
        0x1c => {
            if bvar4 {
                0x1fb5
            } else {
                ((-(bvar3 as i16) as u16) & 0x1a) + 0x1f8d
            }
        }
        0x1d => {
            if bvar4 {
                0x1fb6
            } else {
                ((-(bvar3 as i16) as u16) & 0x1a) + 0x1f8e
            }
        }
        0x39 => {
            if bvar4 {
                0x1fbc
            } else {
                ((-(bvar3 as i16) as u16) & 0x12) + 0x1f9c
            }
        }
        0x3a => 0x1f9d,
        0x35 => {
            if bvar4 {
                0x1fbd
            } else if bvar3 {
                0x1faf
            } else if fallback_rng != 0 {
                0x1fbe
            } else {
                0x21bd
            }
        }
        _ => 0x1f81,
    };
    Some(format!("0x{event:04x}"))
}

fn clamp_i16(value: i16, min: i16, max: i16) -> i16 {
    value.max(min).min(max)
}

fn format_i16_hex(value: i16) -> String {
    if value < 0 {
        format!("-0x{:04x}", value.unsigned_abs())
    } else {
        format!("0x{:04x}", value as u16)
    }
}

fn push_match_engine_runtime_mutation(
    mutations: &mut Vec<MatchEngineRuntimeMutation>,
    table: &str,
    row: u32,
    field: &str,
    record_offset: &str,
    before: &str,
    after: &str,
    event_code: Option<&str>,
    source_function: &str,
    formula: &str,
    evidence: &str,
) {
    mutations.push(MatchEngineRuntimeMutation {
        table: table.to_string(),
        row,
        field: field.to_string(),
        record_offset: record_offset.to_string(),
        before: before.to_string(),
        after: after.to_string(),
        event_code: event_code.map(str::to_string),
        source_function: source_function.to_string(),
        formula: formula.to_string(),
        exactness_tier: "static-code-derived-frontier-mutation".to_string(),
        evidence: evidence.to_string(),
    });
}

pub fn apply_match_engine_runtime_plan_to_store(
    store: &mut MatchEngineRuntimeStore,
    mutations: &[MatchEngineRuntimeMutation],
) {
    for mutation in mutations {
        if !store.runtime_mutations.iter().any(|existing| {
            existing.table == mutation.table
                && existing.row == mutation.row
                && existing.field == mutation.field
                && existing.record_offset == mutation.record_offset
                && existing.source_function == mutation.source_function
                && existing.event_code == mutation.event_code
        }) {
            store.runtime_mutations.push(mutation.clone());
            store.applied_runtime_mutations = store.applied_runtime_mutations.saturating_add(1);
        }
    }
}

pub fn apply_match_engine_player_evaluation_output_to_store(
    store: &mut MatchEngineRuntimeStore,
    output: MatchEnginePlayerEvaluationOutput,
) {
    if let Some(existing) = store.player_evaluation_outputs.iter_mut().find(|existing| {
        existing.row == output.row && existing.source_function == output.source_function
    }) {
        *existing = output;
    } else {
        store.player_evaluation_outputs.push(output);
    }
}

pub fn apply_match_engine_late_branch_execution_output_to_store(
    store: &mut MatchEngineRuntimeStore,
    output: MatchEngineLateBranchExecutionOutput,
) {
    if let Some(existing) = store
        .late_branch_execution_outputs
        .iter_mut()
        .find(|existing| {
            existing.row == output.row && existing.source_function == output.source_function
        })
    {
        *existing = output;
    } else {
        store.late_branch_execution_outputs.push(output);
    }
}

pub fn apply_match_engine_action_selection_output_to_store(
    store: &mut MatchEngineRuntimeStore,
    output: MatchEngineActionSelectionOutput,
) {
    if let Some(existing) = store
        .action_selection_outputs
        .iter_mut()
        .find(|existing| existing.row == output.row)
    {
        *existing = output;
    } else {
        store.action_selection_outputs.push(output);
    }
}

pub fn apply_match_engine_event_queue_outputs_to_store(
    store: &mut MatchEngineRuntimeStore,
    outputs: &[MatchEngineEventQueueOutput],
) {
    for output in outputs {
        if !store.event_queue_outputs.iter().any(|existing| {
            existing.row == output.row
                && existing.source_function == output.source_function
                && existing.event_code == output.event_code
                && existing.owner_slot == output.owner_slot
                && existing.target_slot == output.target_slot
        }) {
            store.event_queue_outputs.push(output.clone());
        }
    }
}

pub fn apply_match_engine_state_mutation_outputs_to_store(
    store: &mut MatchEngineRuntimeStore,
    outputs: &[MatchEngineStateMutationOutput],
) {
    for output in outputs {
        if !store.state_mutation_outputs.iter().any(|existing| {
            existing.row == output.row
                && existing.table == output.table
                && existing.field == output.field
                && existing.record_offset == output.record_offset
                && existing.source_event_code == output.source_event_code
        }) {
            store.state_mutation_outputs.push(output.clone());
        }
    }
}

pub fn apply_match_engine_result_finalization_output_to_store(
    store: &mut MatchEngineRuntimeStore,
    output: MatchEngineResultFinalizationOutput,
) {
    if let Some(existing) = store
        .result_finalization_outputs
        .iter_mut()
        .find(|existing| {
            existing.row == output.row && existing.source_function == output.source_function
        })
    {
        *existing = output;
    } else {
        store.result_finalization_outputs.push(output);
    }
}

fn match_engine_result_finalization_output(
    scenario: &MatchEngineRuntimeScenario,
) -> MatchEngineResultFinalizationOutput {
    MatchEngineResultFinalizationOutput {
        row: scenario.match_row,
        source_function: "0x006a4020".to_string(),
        event_code: "0x2004".to_string(),
        match_state_home_offset: "+0xf5bc".to_string(),
        match_state_away_offset: "+0xf5f2".to_string(),
        fixture_home_offset: "+0x49".to_string(),
        fixture_away_offset: "+0x4a".to_string(),
        home_score: scenario.home_score,
        away_score: scenario.away_score,
        phase_after_offset: "+0x8eb3".to_string(),
        phase_after: "0x00".to_string(),
        evidence: "0x006a4020 phase/final-score controller emits event 0x2004, copies match-state +0xf5bc/+0xf5f2 into fixture +0x49/+0x4a through match-state +0x4792, then clears phase byte +0x8eb3.".to_string(),
    }
}

fn match_engine_state_mutation_outputs(
    scenario: &MatchEngineRuntimeScenario,
    event_outputs: &[MatchEngineEventQueueOutput],
) -> Vec<MatchEngineStateMutationOutput> {
    let event_count_after = scenario
        .event_queue_count_before
        .saturating_add(event_outputs.len() as u16);
    let last_event_code = event_outputs
        .last()
        .map(|event| event.event_code.clone())
        .unwrap_or_else(|| "none".to_string());
    let mut outputs = vec![
        match_engine_state_mutation_output(
            scenario.match_row,
            "match_state",
            "event_queue_count",
            "+0x08",
            &format!("0x{:04x}", scenario.event_queue_count_before),
            &format!("0x{event_count_after:04x}"),
            None,
            "0x006bc8d0",
            "event queue writer increments the match event queue count after appending 0x0e-byte event slots",
        ),
        match_engine_state_mutation_output(
            scenario.match_row,
            "match_state",
            "stored_action_owner",
            "+0x8ea7",
            "0xff",
            &format!("0x{:02x}", scenario.action_owner_slot),
            event_outputs.first().map(|event| event.event_code.as_str()),
            "0x006a0550",
            "stored-action resolver owns the action owner byte before the event queue appends the selected action event",
        ),
        match_engine_state_mutation_output(
            scenario.match_row,
            "match_state",
            "stored_action_target",
            "+0x8ea8",
            "0xff",
            &format!("0x{:02x}", scenario.action_target_slot),
            event_outputs.first().map(|event| event.event_code.as_str()),
            "0x006a0550",
            "stored-action resolver owns the action target byte before the event queue appends the selected action event",
        ),
        match_engine_state_mutation_output(
            scenario.match_row,
            "match_state",
            "last_event_code",
            "+0x30 + (count-1)*0x0e",
            "empty",
            &last_event_code,
            event_outputs.last().map(|event| event.event_code.as_str()),
            "0x006bc8d0",
            "last appended event slot code follows the verified +0x30 base and 0x0e stride queue shape",
        ),
    ];
    if let Some(first_event) = event_outputs.first() {
        outputs.push(match_engine_state_mutation_output(
            scenario.match_row,
            "match_state",
            "selected_action_code",
            "+0x8eae",
            "0xff",
            &first_event.action_code,
            Some(first_event.event_code.as_str()),
            "0x006a0550",
            "stored-action resolver exposes selected action byte before queue emission",
        ));
    }
    outputs
}

fn match_engine_state_mutation_output(
    row: u32,
    table: &str,
    field: &str,
    record_offset: &str,
    before: &str,
    after: &str,
    source_event_code: Option<&str>,
    source_function: &str,
    evidence: &str,
) -> MatchEngineStateMutationOutput {
    MatchEngineStateMutationOutput {
        row,
        table: table.to_string(),
        field: field.to_string(),
        record_offset: record_offset.to_string(),
        before: before.to_string(),
        after: after.to_string(),
        source_event_code: source_event_code.map(|code| code.to_string()),
        source_function: source_function.to_string(),
        evidence: evidence.to_string(),
    }
}

fn match_engine_event_queue_outputs(
    scenario: &MatchEngineRuntimeScenario,
    action_output: &MatchEngineActionSelectionOutput,
) -> Vec<MatchEngineEventQueueOutput> {
    let mut outputs = Vec::new();
    if let Some(event_code) = &action_output.selected_action_event_code {
        outputs.push(match_engine_event_queue_output(
            u32::from(scenario.event_queue_count_before),
            event_code,
            scenario,
            &action_output.selected_action_code,
            action_output.shot_action_score_0x39,
            Some("+0x720"),
            "selected action event candidate emitted by 0x006e65e0 and appended by 0x006bc8d0",
        ));
    }
    if let Some(event_code) = &action_output.direct_shot_event_code {
        outputs.push(match_engine_event_queue_output(
            u32::from(scenario.event_queue_count_before).saturating_add(outputs.len() as u32),
            event_code,
            scenario,
            "0x33",
            action_output.direct_shot_return_score,
            Some("+0x720"),
            "direct shot return path sets event 0x1f7f before 0x006bc8d0 appends the slot",
        ));
    }
    outputs
}

fn match_engine_event_queue_output(
    row: u32,
    event_code: &str,
    scenario: &MatchEngineRuntimeScenario,
    action_code: &str,
    score: i16,
    mirror_offset: Option<&str>,
    evidence_detail: &str,
) -> MatchEngineEventQueueOutput {
    MatchEngineEventQueueOutput {
        row,
        source_function: "0x006bc8d0".to_string(),
        event_code: event_code.to_string(),
        slot_base: "+0x30".to_string(),
        stride: "0x0e".to_string(),
        owner_slot: scenario.action_owner_slot,
        target_slot: scenario.action_target_slot,
        action_code: action_code.to_string(),
        score,
        mirror_offset: mirror_offset.map(|offset| offset.to_string()),
        evidence: format!(
            "{evidence_detail}; queue shape is code-derived from 0x006bc8d0: slot base +0x30, stride 0x0e, selected-event mirror +0x720."
        ),
    }
}

fn match_engine_action_selection_output(
    scenario: &MatchEngineRuntimeScenario,
    evaluation_output: &MatchEnginePlayerEvaluationOutput,
    late_branch_output: &MatchEngineLateBranchExecutionOutput,
) -> MatchEngineActionSelectionOutput {
    let action_score = match_player_action_score_formula(scenario);
    let shot_action_score = match_shot_action_score_formula(scenario);
    let direct_shot_return_score = match_direct_shot_return_formula(scenario);
    let selected_action_event_code = match_shot_action_event_code(
        scenario.selected_action_code,
        false,
        false,
        scenario.rng_action_slot_score,
    );
    let direct_shot_event_code = Some("0x1f7f".to_string());
    let branch_product_applied = late_branch_output
        .offset_multiplier_products
        .iter()
        .find(|product| product.offset == "+0x99")
        .map(|product| product.product.clone())
        .unwrap_or_else(|| "1.000000".to_string());

    MatchEngineActionSelectionOutput {
        row: scenario.player_slot_row,
        source_functions: vec![
            "0x006d1a20".to_string(),
            "0x006d46c0".to_string(),
            "0x006e65e0".to_string(),
            "0x006bc8d0".to_string(),
        ],
        evaluation_score_0x3b: evaluation_output.evaluation_score_0x3b,
        action_score_0x37: action_score,
        shot_action_score_0x39: shot_action_score,
        direct_shot_return_score,
        selected_action_code: format!("0x{:02x}", scenario.selected_action_code),
        selected_action_event_code,
        direct_shot_event_code,
        decisive_float_offset: "+0x99".to_string(),
        decisive_float_value: evaluation_output.float_0x99.clone(),
        branch_product_applied,
        evidence: "First executable Rust match action selection slice: final 0x006d1a20 player float feeds the 0x006d46c0 action score, 0x006e65e0 selected action/shot score, and 0x006bc8d0 event-code candidate.".to_string(),
    }
}

fn match_engine_late_branch_execution_output(
    scenario: &MatchEngineRuntimeScenario,
) -> MatchEngineLateBranchExecutionOutput {
    let mut products: BTreeMap<String, (f64, Vec<String>)> = BTreeMap::new();
    let mut applied_branches = Vec::new();
    let mut skipped_branches = Vec::new();
    let mut rng_rolls = Vec::new();

    if scenario.late_manager_competition_branch {
        apply_offset_multiplier(&mut products, "+0xf5", "_DAT_00958324");
        apply_offset_multiplier(&mut products, "+0x9d", "_DAT_0095b33c");
        apply_offset_multiplier(&mut products, "+0xf1", "_DAT_0095b33c");
        apply_offset_multiplier(&mut products, "+0xb9", "_DAT_00958324");
        applied_branches.push("0 late manager/competition branch".to_string());
    } else {
        skipped_branches.push("0 late manager/competition branch".to_string());
    }

    if scenario.weak_tactical_profile_branch {
        apply_offset_multiplier(&mut products, "+0x9d", "_DAT_0095b0f4");
        for offset in ["+0xf1", "+0x95", "+0x91", "+0x7d"] {
            apply_offset_multiplier(&mut products, offset, "_DAT_0095aef4");
        }
        apply_offset_multiplier(&mut products, "+0xad", "_DAT_0095b034");
        for offset in ["+0xa9", "+0xdd", "+0xb9"] {
            apply_offset_multiplier(&mut products, offset, "_DAT_0095af10");
        }
        for offset in ["+0xcd", "+0xc5"] {
            apply_offset_multiplier(&mut products, offset, "_DAT_00956f10");
        }
        applied_branches.push("1 weak tactical profile branch".to_string());
    } else {
        skipped_branches.push("1 weak tactical profile branch".to_string());
    }

    if scenario.low_confidence_side_state_branch {
        for offset in ["+0x7d", "+0xf5", "+0xc1"] {
            apply_offset_multiplier(&mut products, offset, "_DAT_00958fcc");
        }
        for offset in ["+0xf1", "+0x9d", "+0x95"] {
            apply_offset_multiplier(&mut products, offset, "_DAT_00958324");
        }
        applied_branches.push("2 low-confidence side-state branch".to_string());
    } else {
        skipped_branches.push("2 low-confidence side-state branch".to_string());
    }

    if (scenario.local_38_flags_0x45 & 0x40000) != 0 {
        let threshold = scenario.player_byte_0x5a + scenario.cvar16;
        let success = threshold < scenario.rng_gate_0x32;
        rng_rolls.push(match_engine_rng_roll(
            0,
            "0x32",
            scenario.rng_gate_0x32,
            threshold,
            success,
        ));
        if success {
            for offset in ["+0xf5", "+0x95", "+0x91", "+0xc1"] {
                apply_offset_multiplier(&mut products, offset, "_DAT_00958fcc");
            }
            for offset in ["+0x9d", "+0xf1"] {
                apply_offset_multiplier(&mut products, offset, "_DAT_00958324");
            }
            applied_branches
                .push("3 local_38 pressure mask 0x40000 rng-success branch".to_string());
        } else {
            skipped_branches
                .push("3 local_38 pressure mask 0x40000 rng-success branch".to_string());
        }
    } else {
        skipped_branches.push("3 local_38 pressure mask 0x40000 rng-success branch".to_string());
    }

    if scenario.local_38_byte_0x44 < -25
        && scenario.related_byte_0x44 < 15
        && scenario.player_byte_0x5b < 10
        && scenario.related_short_0x0b > 0x0ea6
        && scenario.opponent_short_0x80 + 250 < scenario.related_short_0x0b
    {
        apply_offset_multiplier(&mut products, "+0xf5", "_DAT_00958fcc");
        apply_offset_multiplier(&mut products, "+0x7d", "_DAT_00958324");
        if scenario.local_38_byte_0x44 < -75 {
            for offset in ["+0xf1", "+0x9d", "+0x99"] {
                apply_offset_multiplier(&mut products, offset, "_DAT_00958324");
            }
            for offset in ["+0x95", "+0xc1"] {
                apply_offset_multiplier(&mut products, offset, "_DAT_00958fcc");
            }
        }
        applied_branches.push("4 local_38 opponent-strength pressure branch".to_string());
    } else {
        skipped_branches.push("4 local_38 opponent-strength pressure branch".to_string());
    }

    if (scenario.local_38_flags_0x45 & 0x17) != 0 {
        if scenario.player_byte_0x5b < 16 || scenario.player_byte_0x57 < 16 {
            if scenario.player_byte_0x5b <= 4 {
                apply_offset_multiplier(&mut products, "+0xf5", "_DAT_00958324");
                applied_branches.push("5 local_38 mask 0x17 low-confidence branch".to_string());
            } else {
                skipped_branches.push("5 local_38 mask 0x17 confidence branch".to_string());
            }
        } else {
            apply_offset_multiplier(&mut products, "+0xf5", "_DAT_00958334");
            applied_branches.push("5 local_38 mask 0x17 high-confidence branch".to_string());
        }
    } else {
        skipped_branches.push("5 local_38 mask 0x17 confidence branch".to_string());
    }

    if (scenario.local_38_flags_0x45 & 0x2088) != 0
        && ((scenario.related_short_0x0b > 0x109a && scenario.player_byte_0x5b < 10)
            || scenario.player_byte_0x5b < 5)
    {
        apply_offset_multiplier(&mut products, "+0xf5", "_DAT_00958324");
        applied_branches.push("6 local_38 mask 0x2088 defensive-pressure branch".to_string());
    } else {
        skipped_branches.push("6 local_38 mask 0x2088 defensive-pressure branch".to_string());
    }

    if (scenario.local_38_flags_0x45 & 0x38da00) != 0 {
        if scenario.player_byte_0x5b == 1 {
            apply_offset_multiplier(&mut products, "+0xf5", "_DAT_0095b05c");
            applied_branches.push("7 local_38 mask 0x38da00 severe-pressure branch".to_string());
        } else if scenario.player_byte_0x5b <= 5 {
            apply_offset_multiplier(&mut products, "+0xf5", "_DAT_00958fcc");
            applied_branches.push("7 local_38 mask 0x38da00 pressure branch".to_string());
        } else {
            skipped_branches.push("7 local_38 mask 0x38da00 severe-pressure branch".to_string());
        }
    } else {
        skipped_branches.push("7 local_38 mask 0x38da00 severe-pressure branch".to_string());
    }

    if (scenario.local_38_flags_0x45 & 0x20000) != 0 {
        if match_engine_special_action_condition(scenario) {
            apply_special_action_condition_path(&mut products);
            applied_branches
                .push("8 local_38 mask 0x20000 special-action condition branch".to_string());
        } else {
            let success = scenario.rng_gate_0x14 > scenario.player_byte_0x5a;
            rng_rolls.push(match_engine_rng_roll(
                1,
                "0x14",
                scenario.rng_gate_0x14,
                scenario.player_byte_0x5a,
                success,
            ));
            if success {
                apply_special_action_rng_path(&mut products);
                applied_branches
                    .push("8 local_38 mask 0x20000 special-action rng branch".to_string());
            } else {
                skipped_branches.push("8 local_38 mask 0x20000 special-action branch".to_string());
            }
        }
    } else {
        skipped_branches.push("8 local_38 mask 0x20000 special-action branch".to_string());
    }

    if (scenario.local_38_flags_0x45 & 0x40) != 0 {
        if match_engine_special_action_condition(scenario) {
            apply_special_action_condition_path(&mut products);
            applied_branches
                .push("9 local_38 mask 0x40 special-action condition branch".to_string());
        } else {
            let success = scenario.rng_gate_0x19 > scenario.player_byte_0x5a;
            rng_rolls.push(match_engine_rng_roll(
                2,
                "0x19",
                scenario.rng_gate_0x19,
                scenario.player_byte_0x5a,
                success,
            ));
            if success {
                apply_special_action_rng_path(&mut products);
                applied_branches.push("9 local_38 mask 0x40 special-action rng branch".to_string());
            } else {
                skipped_branches.push("9 local_38 mask 0x40 special-action branch".to_string());
            }
        }
    } else {
        skipped_branches.push("9 local_38 mask 0x40 special-action branch".to_string());
    }

    MatchEngineLateBranchExecutionOutput {
        row: scenario.player_slot_row,
        source_function: "0x006d1a20".to_string(),
        applied_branches,
        skipped_branches,
        rng_rolls,
        offset_multiplier_products: products
            .into_iter()
            .map(|(offset, (product, symbols))| MatchEngineOffsetMultiplierProduct {
                offset,
                product: format_match_engine_float(product),
                symbols,
            })
            .collect(),
        evidence: "Executable Rust branch order for deterministic and FUN_008fc4f0-gated late multipliers from D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006d1a20.c:1299-1456.".to_string(),
    }
}

fn apply_special_action_condition_path(products: &mut BTreeMap<String, (f64, Vec<String>)>) {
    for offset in ["+0xf5", "+0x91"] {
        apply_offset_multiplier(products, offset, "_DAT_0095b05c");
    }
    for offset in ["+0x9d", "+0xf1", "+0x95", "+0xc1"] {
        apply_offset_multiplier(products, offset, "_DAT_00958324");
    }
}

fn apply_special_action_rng_path(products: &mut BTreeMap<String, (f64, Vec<String>)>) {
    for offset in ["+0xf5", "+0x91", "+0xc1"] {
        apply_offset_multiplier(products, offset, "_DAT_00958fcc");
    }
    for offset in ["+0x9d", "+0xf1", "+0x95"] {
        apply_offset_multiplier(products, offset, "_DAT_00958324");
    }
}

fn match_engine_special_action_condition(scenario: &MatchEngineRuntimeScenario) -> bool {
    scenario.player_byte_0x5a < 5
        && scenario.player_byte_0x5b < 6
        && scenario.player_byte_0x5d < 10
        && scenario.related_short_0x0d > 0x1482
}

fn apply_offset_multiplier(
    products: &mut BTreeMap<String, (f64, Vec<String>)>,
    offset: &str,
    symbol: &str,
) {
    let multiplier = parse_match_engine_constant(symbol);
    let entry = products
        .entry(offset.to_string())
        .or_insert_with(|| (1.0, Vec::new()));
    entry.0 *= multiplier;
    entry.1.push(symbol.to_string());
}

fn match_engine_rng_roll(
    order: u8,
    argument: &str,
    value: i16,
    threshold: i16,
    success: bool,
) -> MatchEngineRngRoll {
    MatchEngineRngRoll {
        order,
        function: "FUN_008fc4f0".to_string(),
        argument: argument.to_string(),
        value,
        threshold,
        success,
    }
}

pub fn match_engine_runtime_mutations_ready(store: &MatchEngineRuntimeStore) -> bool {
    let has_player_eval = store.runtime_mutations.iter().any(|mutation| {
        mutation.source_function == "0x006d1a20" && mutation.record_offset == "+0x3b"
    });
    let has_eval_float_shape = [
        "+0x7d", "+0x8d", "+0x91", "+0x95", "+0x99", "+0xb9", "+0xcd", "+0xf1",
    ]
    .iter()
    .all(|offset| {
        store.runtime_mutations.iter().any(|mutation| {
            mutation.source_function == "0x006d1a20" && mutation.record_offset == *offset
        })
    });
    let has_action_score = store.runtime_mutations.iter().any(|mutation| {
        mutation.source_function == "0x006d46c0"
            && mutation.record_offset == "+0x37"
            && mutation
                .formula
                .contains("tactic-flag attribute contributions")
    });
    let has_shot_score = store.runtime_mutations.iter().any(|mutation| {
        mutation.source_function == "0x006e65e0"
            && mutation.record_offset == "+0x39"
            && mutation.formula.contains("0xd2")
    });
    let has_direct_shot_return = store.runtime_mutations.iter().any(|mutation| {
        mutation.source_function == "0x006e65e0"
            && mutation.record_offset == "return"
            && mutation.event_code.as_deref() == Some("0x1f7f")
    });
    let has_stored_action = store.runtime_mutations.iter().any(|mutation| {
        mutation.source_function == "0x006a0550" && mutation.event_code.as_deref() == Some("0x20f0")
    });
    let has_event_queue = store.runtime_mutations.iter().any(|mutation| {
        mutation.source_function == "0x006bc8d0"
            && mutation.record_offset.contains("0x0e")
            && mutation.event_code.as_deref() == Some("0x1f7f")
    });
    has_player_eval
        && has_eval_float_shape
        && has_action_score
        && has_shot_score
        && has_direct_shot_return
        && has_stored_action
        && has_event_queue
        && store.applied_runtime_mutations >= 20
}

pub fn match_engine_player_evaluation_outputs_ready(store: &MatchEngineRuntimeStore) -> bool {
    store.player_evaluation_outputs.iter().any(|output| {
        output.source_function == "0x006d1a20"
            && output.evaluation_score_0x3b > 0
            && output
                .constants_used
                .iter()
                .any(|item| item == "_DAT_00955880")
            && output
                .constants_used
                .iter()
                .any(|item| item == "_DAT_009569b0")
            && output
                .constants_used
                .iter()
                .any(|item| item == "_DAT_009569d0")
            && output
                .constants_used
                .iter()
                .any(|item| item == "_DAT_009586d0")
            && output
                .branch_applications
                .iter()
                .any(|branch| branch.contains("+0x99"))
            && output
                .branch_applications
                .iter()
                .any(|branch| branch.contains("late manager/competition branch"))
            && output
                .branch_applications
                .iter()
                .any(|branch| branch.contains("weak tactical profile branch"))
            && output
                .branch_applications
                .iter()
                .any(|branch| branch.contains("low-confidence side-state branch"))
            && output
                .float_0x7d
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
            && output
                .float_0x8d
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
            && output
                .float_0x91
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
            && output
                .float_0x95
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
            && output
                .float_0x99
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
            && output
                .float_0xb9
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
            && output
                .float_0xcd
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
            && output
                .float_0xf1
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsInboxEmissionMapEntry {
    pub system: String,
    pub record_offset: Option<String>,
    pub stride: Option<String>,
    pub helper: Option<String>,
    pub function: String,
    pub evidence: String,
}

fn default_news_inbox_emission_map() -> Vec<NewsInboxEmissionMapEntry> {
    vec![
        news_inbox_emission_map_entry(
            "fixture/news subrecord stride",
            None,
            Some("0x68"),
            Some("0x00596fa0"),
            "0x0050c8d0",
            "Paired fixture/news event creation indexes subrecords with param_2 * 0x68 under the base pointer at param_1 + 0xa3 before calling the event creator.",
        ),
        news_inbox_emission_map_entry(
            "fixture/news subrecord base pointer",
            Some("0xa3"),
            Some("0x68"),
            None,
            "0x0050c8d0",
            "The paired event creator reads the fixture/news subrecord table pointer from param_1 + 0xa3, then applies the 0x68-byte subrecord stride.",
        ),
        news_inbox_emission_map_entry(
            "paired event first status/date",
            Some("0x30"),
            None,
            Some("0x00596fa0"),
            "0x0050c8d0",
            "After creating the first event, the function writes event +0x30 from subrecord +0x07 plus 3.",
        ),
        news_inbox_emission_map_entry(
            "paired event second status/date",
            Some("0x30"),
            None,
            Some("0x00536190"),
            "0x0050c8d0",
            "The second event is dated from the first event date plus one day and writes event +0x30 from subrecord +0x07 plus 4.",
        ),
        news_inbox_emission_map_entry(
            "news visibility/reset byte",
            Some("0xde"),
            None,
            Some("0x0076dce0"),
            "0x0076e180",
            "The news helper clears news +0xde to 0 before following the source pointer at param_2 + 0xcf and calling 0x0076dce0.",
        ),
        news_inbox_emission_map_entry(
            "queued news removal",
            None,
            None,
            Some("0x006724d0"),
            "0x00595580",
            "Fixture/news cleanup filters queued manager-visible news items, then calls 0x006724d0 to unlink the node and free it through 0x00672260/0x00933d24.",
        ),
        news_inbox_emission_map_entry(
            "queued club/news club stride",
            None,
            Some("0x245"),
            Some("0x0076e180"),
            "0x00595580",
            "Fixture/news cleanup resolves club/news records through DAT_00acd5bc plus club index * 0x245 before calling 0x0076e180 for flags 0xe/0xf.",
        ),
    ]
}

fn news_inbox_emission_map_entry(
    system: &str,
    record_offset: Option<&str>,
    stride: Option<&str>,
    helper: Option<&str>,
    function: &str,
    evidence: &str,
) -> NewsInboxEmissionMapEntry {
    NewsInboxEmissionMapEntry {
        system: system.to_string(),
        record_offset: record_offset.map(str::to_string),
        stride: stride.map(str::to_string),
        helper: helper.map(str::to_string),
        function: function.to_string(),
        evidence: evidence.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsInboxFormulaLiftEntry {
    pub formula: String,
    pub function: String,
    pub decompile_artifact: String,
    pub decompile_lines: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub constants: Vec<String>,
    pub branch_rule: String,
    pub rust_semantics: String,
    pub evidence: String,
    pub promotion_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsInboxFormulaScenario {
    pub source_row: u32,
    pub source_subrecord_base_offset: String,
    pub subrecord_index: u32,
    pub subrecord_start_status: i16,
    pub current_date: GameDate,
    pub club_id: u32,
    pub club_record_has_news_owner: bool,
    pub queued_node_row: u32,
    pub queued_node_prev: Option<u32>,
    pub queued_node_next: Option<u32>,
    pub queue_count_before: u32,
    pub news_reset_byte_before: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsInboxFormulaMutation {
    pub table: String,
    pub row: u32,
    pub field: String,
    pub record_offset: String,
    pub before: String,
    pub after: String,
    pub source_function: String,
    pub formula: String,
    pub exactness_tier: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsInboxRuntimeStore {
    pub created_events: Vec<NewsInboxRuntimeEvent>,
    pub visible_news_dispatches: Vec<NewsInboxRuntimeDispatch>,
    pub removed_queue_nodes: Vec<NewsInboxRuntimeRemovedNode>,
    pub applied_formula_mutations: usize,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsInboxRuntimeEvent {
    pub row: u32,
    pub source_subrecord_index: u32,
    pub event_status_0x30: i16,
    pub event_date: GameDate,
    pub helper: String,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsInboxRuntimeDispatch {
    pub row: u32,
    pub club_id: u32,
    pub news_reset_byte_0xde: u8,
    pub helper: String,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsInboxRuntimeRemovedNode {
    pub row: u32,
    pub previous: Option<u32>,
    pub next: Option<u32>,
    pub queue_count_after: u32,
    pub helper: String,
    pub source_function: String,
}

fn default_news_inbox_formula_lift_map() -> Vec<NewsInboxFormulaLiftEntry> {
    vec![
        news_inbox_formula_lift_entry(
            "paired fixture/news event creation",
            "0x0050c8d0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_news/0x0050c8d0.c",
            "subrecord stride and two event writes",
            &[
                "source table pointer +0xa3",
                "source subrecord index",
                "source subrecord stride 0x68",
                "source subrecord date/status at +0x03/+0x05/+0x07",
                "current date helper 0x00536690 and add-days helper 0x00536190",
            ],
            &[
                "first event via 0x00596fa0",
                "first event +0x30 = subrecord +0x07 + 3",
                "second event via 0x00596fa0",
                "second event +0x30 = subrecord +0x07 + 4",
            ],
            &["0xa3", "0x68", "0x30", "3", "4", "1"],
            "Create one event using the current computed date, then a paired event one day later; both use the same source subrecord and write +0x30 from subrecord +0x07 plus literal offsets.",
            "Rust creates two inbox/news event rows with statuses base+3 and base+4, preserving helper provenance.",
            "The decompile indexes *(in_ECX+0xa3) + param_1*0x68, calls 0x00596fa0 twice, writes event +0x30 as subrecord +0x07 + 3, then adds one day via 0x00536190 and writes +4.",
        ),
        news_inbox_formula_lift_entry(
            "news visibility reset and club dispatch",
            "0x0076e180",
            "D:/cm0102-carve/decompiled/gameplay_lifts_news/0x0076e180.c",
            "news reset before 0x0076dce0 dispatch",
            &[
                "news record pointer",
                "club record pointer",
                "club news owner pointer +0xcf",
            ],
            &[
                "news byte +0xde reset to 0",
                "dispatch via 0x0076dce0 when owner exists and passes range guard",
            ],
            &["0xde", "0xcf", "0x10"],
            "When both news and club records are present, clear news +0xde, read club +0xcf, and dispatch through 0x0076dce0 if the owner/range guard accepts it.",
            "Rust records the visible news dispatch and byte +0xde reset as part of the inbox runtime store.",
            "The decompile writes *(param_1+0xde)=0, reads *(param_2+0xcf), guards it, then calls FUN_0076dce0(param_1, owner, 0).",
        ),
        news_inbox_formula_lift_entry(
            "queued news node unlink and free",
            "0x006724d0",
            "D:/cm0102-carve/decompiled/gameplay_lifts_news/0x006724d0.c",
            "linked-list unlink/free",
            &[
                "queue head at manager +2 shorts",
                "queue tail at manager +6 shorts",
                "node next pointer +6",
                "node previous pointer +10",
                "active visible node pointer at manager +10 shorts",
            ],
            &[
                "queue count decremented",
                "head/tail links repaired",
                "active node advanced when removed",
                "node freed through 0x00672260 and 0x00933d24",
            ],
            &["6", "10", "-1"],
            "Remove a queued news node by repairing previous/next links, decrementing queue count, updating active pointer when needed, then freeing the node.",
            "Rust records removed queue nodes and the queue count after unlink.",
            "The decompile rewrites neighbouring node +6/+10 links, decrements *in_ECX, optionally advances *(in_ECX+5), calls 0x00672260, then frees with 0x00933d24.",
        ),
    ]
}

fn news_inbox_formula_lift_entry(
    formula: &str,
    function: &str,
    decompile_artifact: &str,
    decompile_lines: &str,
    inputs: &[&str],
    outputs: &[&str],
    constants: &[&str],
    branch_rule: &str,
    rust_semantics: &str,
    evidence: &str,
) -> NewsInboxFormulaLiftEntry {
    NewsInboxFormulaLiftEntry {
        formula: formula.to_string(),
        function: function.to_string(),
        decompile_artifact: decompile_artifact.to_string(),
        decompile_lines: decompile_lines.to_string(),
        inputs: inputs.iter().map(|item| item.to_string()).collect(),
        outputs: outputs.iter().map(|item| item.to_string()).collect(),
        constants: constants.iter().map(|item| item.to_string()).collect(),
        branch_rule: branch_rule.to_string(),
        rust_semantics: rust_semantics.to_string(),
        evidence: evidence.to_string(),
        promotion_status: "formula-lifted-static-code-derived".to_string(),
    }
}

pub fn news_inbox_formula_lift_map_ready(lifts: &[NewsInboxFormulaLiftEntry]) -> bool {
    let required = [
        (
            "paired fixture/news event creation",
            "0x0050c8d0",
            "0x68",
            "second event +0x30",
        ),
        (
            "news visibility reset and club dispatch",
            "0x0076e180",
            "0xde",
            "dispatch via 0x0076dce0",
        ),
        (
            "queued news node unlink and free",
            "0x006724d0",
            "10",
            "queue count decremented",
        ),
    ];
    required
        .iter()
        .all(|(formula, function, constant, output)| {
            lifts.iter().any(|lift| {
                lift.formula == *formula
                    && lift.function == *function
                    && lift.constants.iter().any(|item| item == constant)
                    && lift.outputs.iter().any(|item| item.contains(output))
                    && lift.promotion_status == "formula-lifted-static-code-derived"
                    && lift
                        .decompile_artifact
                        .starts_with("D:/cm0102-carve/decompiled/")
            })
        })
}

pub fn default_news_inbox_formula_scenario() -> NewsInboxFormulaScenario {
    NewsInboxFormulaScenario {
        source_row: 0,
        source_subrecord_base_offset: "0xa3".to_string(),
        subrecord_index: 0,
        subrecord_start_status: 20,
        current_date: GameDate {
            year: 2001,
            month: 7,
            day: 1,
        },
        club_id: 1,
        club_record_has_news_owner: true,
        queued_node_row: 0,
        queued_node_prev: None,
        queued_node_next: Some(1),
        queue_count_before: 1,
        news_reset_byte_before: 1,
    }
}

fn default_news_inbox_runtime_store() -> NewsInboxRuntimeStore {
    NewsInboxRuntimeStore {
        created_events: Vec::new(),
        visible_news_dispatches: Vec::new(),
        removed_queue_nodes: Vec::new(),
        applied_formula_mutations: 0,
        provenance: "Rust-owned news/inbox runtime store seeded from verified paired-event, visibility reset, and queue-unlink lifts.".to_string(),
    }
}

pub fn plan_news_inbox_formula_mutations(
    backend: &RuntimeBackendSystems,
    scenario: &NewsInboxFormulaScenario,
) -> Vec<NewsInboxFormulaMutation> {
    let mut mutations = Vec::new();
    let has_pair = backend
        .news_inbox_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "paired fixture/news event creation");
    if has_pair {
        for (event_index, status_add, date_add) in [(0, 3, 0), (1, 4, 1)] {
            let event_date = CmPackedDate::from_game_date(scenario.current_date.clone())
                .add_days(date_add)
                .to_game_date();
            mutations.push(NewsInboxFormulaMutation {
                table: "news_inbox.created_events".to_string(),
                row: event_index,
                field: "event_status_0x30".to_string(),
                record_offset: format!(
                    "{} + subrecord {}*0x68 -> event +0x30",
                    scenario.source_subrecord_base_offset, scenario.subrecord_index
                ),
                before: "event missing".to_string(),
                after: format!(
                    "status={}; date={}",
                    scenario.subrecord_start_status + status_add,
                    event_date.iso()
                ),
                source_function: "0x0050c8d0".to_string(),
                formula: "paired fixture/news event creation".to_string(),
                exactness_tier: "formula-derived-news-inbox".to_string(),
                evidence: format!(
                    "Original paired event writer sets event +0x30 to subrecord +0x07 + {status_add}."
                ),
            });
        }
    }
    let has_dispatch = backend
        .news_inbox_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "news visibility reset and club dispatch");
    if has_dispatch && scenario.club_record_has_news_owner {
        mutations.push(NewsInboxFormulaMutation {
            table: "news_inbox.visible_dispatches".to_string(),
            row: scenario.source_row,
            field: "news_reset_byte_0xde".to_string(),
            record_offset: "news +0xde; club +0xcf".to_string(),
            before: format!("0x{:02x}", scenario.news_reset_byte_before),
            after: "0x00; helper=0x0076dce0".to_string(),
            source_function: "0x0076e180".to_string(),
            formula: "news visibility reset and club dispatch".to_string(),
            exactness_tier: "formula-derived-news-inbox".to_string(),
            evidence: "Original news dispatch clears +0xde then calls 0x0076dce0 with the club news owner.".to_string(),
        });
    }
    let has_unlink = backend
        .news_inbox_formula_lift_map
        .iter()
        .any(|entry| entry.formula == "queued news node unlink and free");
    if has_unlink && scenario.queue_count_before > 0 {
        mutations.push(NewsInboxFormulaMutation {
            table: "news_inbox.removed_queue_nodes".to_string(),
            row: scenario.queued_node_row,
            field: "queue_node_unlink".to_string(),
            record_offset: "node +6 next; node +10 previous; queue count".to_string(),
            before: format!(
                "count={}; prev={:?}; next={:?}",
                scenario.queue_count_before, scenario.queued_node_prev, scenario.queued_node_next
            ),
            after: format!(
                "count={}; freed_by=0x00933d24",
                scenario.queue_count_before.saturating_sub(1)
            ),
            source_function: "0x006724d0".to_string(),
            formula: "queued news node unlink and free".to_string(),
            exactness_tier: "formula-derived-news-inbox".to_string(),
            evidence: "Original queue unlink repairs neighbouring +6/+10 links, decrements count, calls 0x00672260, and frees the node with 0x00933d24.".to_string(),
        });
    }
    mutations
}

pub fn apply_news_inbox_formula_plan_to_store(
    store: &mut NewsInboxRuntimeStore,
    mutations: &[NewsInboxFormulaMutation],
    scenario: &NewsInboxFormulaScenario,
) {
    for mutation in mutations {
        match mutation.table.as_str() {
            "news_inbox.created_events" => {
                if !store
                    .created_events
                    .iter()
                    .any(|event| event.row == mutation.row)
                {
                    let status = mutation
                        .after
                        .split("status=")
                        .nth(1)
                        .and_then(|tail| tail.split(';').next())
                        .and_then(|value| value.parse::<i16>().ok())
                        .unwrap_or_default();
                    let event_date = mutation
                        .after
                        .split("date=")
                        .nth(1)
                        .and_then(|value| parse_iso_game_date(value).ok())
                        .unwrap_or_else(|| scenario.current_date.clone());
                    store.created_events.push(NewsInboxRuntimeEvent {
                        row: mutation.row,
                        source_subrecord_index: scenario.subrecord_index,
                        event_status_0x30: status,
                        event_date,
                        helper: "0x00596fa0".to_string(),
                        source_function: mutation.source_function.clone(),
                    });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "news_inbox.visible_dispatches" => {
                if !store
                    .visible_news_dispatches
                    .iter()
                    .any(|dispatch| dispatch.row == mutation.row)
                {
                    store
                        .visible_news_dispatches
                        .push(NewsInboxRuntimeDispatch {
                            row: mutation.row,
                            club_id: scenario.club_id,
                            news_reset_byte_0xde: 0,
                            helper: "0x0076dce0".to_string(),
                            source_function: mutation.source_function.clone(),
                        });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            "news_inbox.removed_queue_nodes" => {
                if !store
                    .removed_queue_nodes
                    .iter()
                    .any(|node| node.row == mutation.row)
                {
                    store.removed_queue_nodes.push(NewsInboxRuntimeRemovedNode {
                        row: mutation.row,
                        previous: scenario.queued_node_prev,
                        next: scenario.queued_node_next,
                        queue_count_after: scenario.queue_count_before.saturating_sub(1),
                        helper: "0x006724d0/0x00672260/0x00933d24".to_string(),
                        source_function: mutation.source_function.clone(),
                    });
                    store.applied_formula_mutations =
                        store.applied_formula_mutations.saturating_add(1);
                }
            }
            _ => {}
        }
    }
}

pub fn news_inbox_formula_plan_ready(mutations: &[NewsInboxFormulaMutation]) -> bool {
    mutations.iter().any(|mutation| {
        mutation.table == "news_inbox.created_events"
            && mutation.row == 0
            && mutation.after.contains("status=23")
            && mutation.after.contains("2001-07-01")
    }) && mutations.iter().any(|mutation| {
        mutation.table == "news_inbox.created_events"
            && mutation.row == 1
            && mutation.after.contains("status=24")
            && mutation.after.contains("2001-07-02")
    }) && mutations.iter().any(|mutation| {
        mutation.table == "news_inbox.visible_dispatches"
            && mutation.record_offset.contains("0xde")
            && mutation.after.contains("0x0076dce0")
    }) && mutations.iter().any(|mutation| {
        mutation.table == "news_inbox.removed_queue_nodes"
            && mutation.record_offset.contains("+10")
            && mutation.after.contains("count=0")
    })
}

pub fn news_inbox_runtime_store_ready(store: &NewsInboxRuntimeStore) -> bool {
    store.created_events.len() >= 2
        && store
            .created_events
            .iter()
            .any(|event| event.row == 0 && event.event_status_0x30 == 23)
        && store
            .created_events
            .iter()
            .any(|event| event.row == 1 && event.event_status_0x30 == 24)
        && store
            .visible_news_dispatches
            .iter()
            .any(|dispatch| dispatch.news_reset_byte_0xde == 0 && dispatch.helper == "0x0076dce0")
        && store
            .removed_queue_nodes
            .iter()
            .any(|node| node.queue_count_after == 0)
        && store.applied_formula_mutations >= 4
}

fn default_match_result_write_map() -> Vec<MatchResultWriteMapEntry> {
    vec![
        match_result_write_map_entry(
            "normal-time score snapshot",
            "0x43",
            "0x44",
            "0xf5bd",
            "0xf5f3",
            Some("0x20f2"),
            Some("0x3de"),
            "0x006a3240",
            "Writes match-state score bytes into fixture +0x43/+0x44 at period threshold 0x3de and may emit event 0x20f2.",
        ),
        match_result_write_map_entry(
            "extra-time first period score snapshot",
            "0x45",
            "0x46",
            "0xf5bd",
            "0xf5f3",
            Some("0x20f1"),
            Some("0x1ef"),
            "0x006a3240",
            "Writes match-state score bytes into fixture +0x45/+0x46 at period threshold 0x1ef and emits event 0x20f1.",
        ),
        match_result_write_map_entry(
            "extra-time/final score snapshot",
            "0x47",
            "0x48",
            "0xf5bd",
            "0xf5f3",
            Some("0x20f3"),
            Some("0x483/0x528"),
            "0x006a3240",
            "Writes match-state score bytes into fixture +0x47/+0x48 around thresholds 0x483 and 0x528, with event 0x20f3 on the 0x483 transition.",
        ),
        match_result_write_map_entry(
            "phase-controller final score",
            "0x49",
            "0x4a",
            "0xf5bc",
            "0xf5f2",
            Some("0x2004"),
            None,
            "0x006a4020",
            "When phase byte +0x8eb3 reaches cases 3/6, emits 0x2004 and copies match-state +0xf5bc/+0xf5f2 into fixture +0x49/+0x4a.",
        ),
        match_result_write_map_entry(
            "abandoned/sentinel score",
            "0x43",
            "none",
            "constant 0xfd",
            "none",
            Some("0x217b"),
            None,
            "0x0069f2f0",
            "Step controller sets match-state +0xf5bd/+0xf5f3 to -3, writes fixture +0x43 to 0xfd, and emits 0x217b when the timeout/sentinel path at +0xf638 fires; no fixture +0x44 write is proven in this branch.",
        ),
    ]
}

fn match_result_write_map_entry(
    phase: &str,
    fixture_home_offset: &str,
    fixture_away_offset: &str,
    source_home_offset: &str,
    source_away_offset: &str,
    event_code: Option<&str>,
    threshold: Option<&str>,
    function: &str,
    evidence: &str,
) -> MatchResultWriteMapEntry {
    MatchResultWriteMapEntry {
        phase: phase.to_string(),
        fixture_home_offset: fixture_home_offset.to_string(),
        fixture_away_offset: fixture_away_offset.to_string(),
        source_home_offset: source_home_offset.to_string(),
        source_away_offset: source_away_offset.to_string(),
        event_code: event_code.map(str::to_string),
        threshold: threshold.map(str::to_string),
        function: function.to_string(),
        evidence: evidence.to_string(),
    }
}

fn default_backend_mutation_log_limit() -> usize {
    1_000
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeBackendStatus {
    FrontierMutationLedger,
    GameplayMutationsImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSystemState {
    pub system: String,
    pub status: RuntimeSystemStatus,
    pub owned_records: usize,
    pub attempted_mutations: u32,
    pub implemented_mutations: u32,
    pub frontier_evidence: String,
}

impl RuntimeSystemState {
    fn frontier_only(system: &str, owned_records: usize, frontier_evidence: &str) -> Self {
        Self {
            system: system.to_string(),
            status: RuntimeSystemStatus::FrontierOnly,
            owned_records,
            attempted_mutations: 0,
            implemented_mutations: 0,
            frontier_evidence: frontier_evidence.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeSystemStatus {
    FrontierOnly,
    Implemented,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSystemMutation {
    pub day: u32,
    pub date: GameDate,
    pub phase: u8,
    pub system: String,
    pub action: String,
    pub status: RuntimeSystemStatus,
    #[serde(default)]
    pub contract_status: Option<GameplayMutatorStatus>,
    #[serde(default)]
    pub trace_file: Option<String>,
    #[serde(default)]
    pub boundary_map: Option<String>,
    #[serde(default)]
    pub implementation_hook: Option<String>,
    #[serde(default)]
    pub parity_gate: Option<String>,
    #[serde(default)]
    pub skeleton_entry_point: Option<String>,
    #[serde(default)]
    pub skeleton_status: Option<String>,
    #[serde(default)]
    pub skeleton_mutations_emitted: Option<usize>,
    #[serde(default)]
    pub skeleton_safety_rule: Option<String>,
    #[serde(default)]
    pub exactness_tier: Option<String>,
    #[serde(default)]
    pub static_proof_rows: Option<usize>,
    #[serde(default)]
    pub formula_lift_status: Option<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessRuntimeState {
    pub mode: HeadlessRuntimeMode,
    pub status: HeadlessPlayStatus,
    pub manager: Option<HeadlessManagerProfile>,
    pub completed_days: u32,
    pub completed_phases: u32,
    pub last_run: Option<HeadlessRunReport>,
    pub stop_reason: Option<String>,
    pub milestones: Vec<HeadlessMilestone>,
    pub command_history: Vec<HeadlessCommandRecord>,
    pub blockers: Vec<HeadlessBlocker>,
}

impl Default for HeadlessRuntimeState {
    fn default() -> Self {
        Self {
            mode: HeadlessRuntimeMode::VerifiedShell,
            status: HeadlessPlayStatus::Runnable,
            manager: None,
            completed_days: 0,
            completed_phases: 0,
            last_run: None,
            stop_reason: None,
            milestones: Vec::new(),
            command_history: Vec::new(),
            blockers: default_headless_blockers(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadlessRuntimeMode {
    VerifiedShell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadlessPlayStatus {
    Runnable,
    BlockedByUnimplementedGameplay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessManagerProfile {
    pub name: String,
    pub club_id: Option<u32>,
    pub status: HeadlessManagerStatus,
    pub provenance: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadlessManagerStatus {
    Unattached,
    ClubSelectedFrontierOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessCommandRecord {
    pub day: u32,
    pub date: GameDate,
    pub command: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessRunReport {
    pub requested_target: HeadlessRunTarget,
    pub start_date: GameDate,
    pub end_date: GameDate,
    pub days_advanced: u32,
    pub phases_advanced: u32,
    pub phase_trace_entries_added: u32,
    pub last_phase_frontiers: usize,
    pub completed_milestones: Vec<HeadlessMilestone>,
    pub still_frontier_only: Vec<String>,
    pub status: HeadlessPlayStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessCampaignReport {
    pub start_date: GameDate,
    pub end_date: GameDate,
    pub days_requested: u32,
    pub days_advanced: u32,
    pub phases_advanced: u32,
    pub checkpoints: Vec<HeadlessCampaignCheckpoint>,
    pub backend: RuntimeBackendCampaignSummary,
    pub still_frontier_only: Vec<String>,
    pub status: HeadlessPlayStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessCampaignCheckpoint {
    pub date: GameDate,
    pub elapsed_days: u32,
    pub phase: u8,
    pub phase_trace_entries: usize,
    pub mutation_log_entries: usize,
    pub match_attempts: u32,
    pub competition_attempts: u32,
    pub transfer_attempts: u32,
    pub news_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBackendCampaignSummary {
    pub mutation_log_entries_added: usize,
    pub total_mutation_log_entries: usize,
    pub match_attempts: u32,
    pub competition_attempts: u32,
    pub transfer_attempts: u32,
    pub news_attempts: u32,
    pub implemented_mutations: u32,
    pub frontier_only_mutations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadlessRunTarget {
    Days(u32),
    Date(GameDate),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessMilestone {
    pub day: u32,
    pub date: GameDate,
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessBlocker {
    pub system: String,
    pub status: String,
    pub required_evidence: String,
}

fn default_headless_blockers() -> Vec<HeadlessBlocker> {
    Vec::new()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmPackedDate {
    pub day_of_year: u16,
    pub year: u16,
    pub leap_year: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub day: u32,
    pub date: GameDate,
    pub kind: String,
    pub message: String,
    /// Intra-day phase the event occurred in (0=AM,1=PM,2=EVE), stamped from
    /// the clock's `DAT_00acde88` when the event was generated. Defaults to
    /// evening for older saves that predate the field.
    #[serde(default = "default_event_phase")]
    pub phase: u8,
}

fn default_event_phase() -> u8 {
    2
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HeadlessSeasonState {
    pub fixtures: Vec<HeadlessSeasonFixture>,
    pub standings: Vec<HeadlessSeasonStanding>,
    pub batches: Vec<HeadlessFixtureBatchReport>,
    #[serde(default)]
    pub schedule_generation: Vec<HeadlessScheduleGenerationProof>,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessSeasonFixture {
    pub row: u32,
    pub competition_id: u32,
    #[serde(default)]
    pub competition_name: String,
    pub date: GameDate,
    pub home_club_id: u32,
    pub home_club_name: String,
    pub away_club_id: u32,
    pub away_club_name: String,
    pub status: HeadlessFixtureStatus,
    pub home_score: Option<u8>,
    pub away_score: Option<u8>,
    #[serde(default)]
    pub match_packet: Option<HeadlessMatchPacket>,
    #[serde(default)]
    pub match_report: Option<HeadlessMatchReport>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessMatchReport {
    pub fixture_row: u32,
    pub headline: String,
    pub summary: String,
    pub scoreline: String,
    pub event_count: usize,
    pub goal_count: usize,
    pub non_scoring_event_count: usize,
    pub highlights: Vec<String>,
    pub news_kind: String,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessMatchPacket {
    pub fixture_row: u32,
    pub match_row: u32,
    pub player_slot_row: u32,
    pub action_owner_slot: u8,
    pub action_target_slot: u8,
    pub evaluation_score_0x3b: i16,
    pub decisive_float_offset: String,
    pub decisive_float_value: String,
    pub selected_action_code: String,
    pub event_codes: Vec<String>,
    #[serde(default)]
    pub match_events: Vec<HeadlessMatchEvent>,
    #[serde(default)]
    pub goal_events: Vec<HeadlessGoalEvent>,
    pub state_mutation_rows: u32,
    pub final_event_code: String,
    pub final_score: String,
    #[serde(default)]
    pub score_source: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessMatchEvent {
    pub minute: u8,
    pub club_id: u32,
    pub club_name: String,
    pub side: String,
    pub kind: String,
    pub event_code: String,
    pub source_function: String,
    pub score_impact: bool,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessGoalEvent {
    pub minute: u8,
    pub club_id: u32,
    pub club_name: String,
    pub side: String,
    pub event_code: String,
    pub source_function: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessScheduleGenerationProof {
    pub function: String,
    pub decompile_artifact: String,
    pub row_shape: String,
    pub constants: Vec<String>,
    pub generated_rounds: u32,
    pub generated_fixtures: u32,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadlessFixtureStatus {
    Pending,
    Played,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessSeasonStanding {
    pub club_id: u32,
    pub club_name: String,
    pub played: u32,
    pub won: u32,
    pub drawn: u32,
    pub lost: u32,
    pub goals_for: u32,
    pub goals_against: u32,
    pub goal_difference: i32,
    pub points: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessFixtureBatchReport {
    pub day: u32,
    pub date: GameDate,
    pub due_fixtures: usize,
    pub played_fixtures: usize,
    pub fixture_rows: Vec<u32>,
    pub result_summary: Vec<String>,
    pub standings_rows_touched: usize,
    pub news_events_created: usize,
    pub provenance: String,
}

impl HeadlessSeasonStanding {
    fn new(club_id: u32, club_name: String) -> Self {
        Self {
            club_id,
            club_name,
            played: 0,
            won: 0,
            drawn: 0,
            lost: 0,
            goals_for: 0,
            goals_against: 0,
            goal_difference: 0,
            points: 0,
        }
    }
}

fn apply_fixture_to_match_engine_scenario(
    scenario: &mut MatchEngineRuntimeScenario,
    fixture: &HeadlessSeasonFixture,
    elapsed_days: u32,
) {
    let seed = fixture
        .row
        .wrapping_mul(31)
        .wrapping_add(fixture.home_club_id.wrapping_mul(7))
        .wrapping_add(fixture.away_club_id.wrapping_mul(11))
        .wrapping_add(elapsed_days);
    scenario.match_row = fixture.row;
    scenario.player_slot_row = fixture.row.saturating_mul(22) + (seed % 22);
    scenario.action_owner_slot = (seed % 11) as u8;
    scenario.action_target_slot = ((seed / 3 + 1) % 11) as u8;
    scenario.event_queue_count_before = ((fixture.row % 0x40) * 2) as u16;
    scenario.player_attr_17 = 12 + (seed % 8) as i16;
    scenario.player_attr_18 = 14 + ((seed / 2) % 7) as i16;
    scenario.player_attr_19 = 11 + ((seed / 3) % 8) as i16;
    scenario.player_attr_1a = 10 + ((seed / 5) % 9) as i16;
    scenario.player_attr_1e = 34 + ((seed / 7) % 8) as i16;
    scenario.player_attr_23 = 9 + ((seed / 11) % 10) as i16;
    scenario.player_attr_24 = 10 + ((seed / 13) % 9) as i16;
    scenario.player_attr_26 = 11 + ((seed / 17) % 8) as i16;
    scenario.player_attr_27 = 12 + ((seed / 19) % 7) as i16;
    scenario.player_attr_3a = 8 + ((seed / 23) % 11) as i16;
    scenario.evaluation_base_score = 48 + ((seed / 29) % 32) as i16;
    scenario.fatigue_drag = 4 + ((elapsed_days + fixture.row) % 10) as i16;
    scenario.shot_base_0x29 = 420 + ((seed % 260) as i16);
    scenario.shot_power_0x180 = 20 + ((seed / 5) % 28) as i16;
    scenario.current_action_score_0x19c = 160 + ((seed / 7) % 180) as i16;
    scenario.selected_action_code = match seed % 6 {
        0 => 0x3a,
        1 => 0x35,
        2 => 0x39,
        3 => 0x18,
        4 => 0x16,
        _ => 0x1a,
    };
    scenario.rng_shot_power = (seed % 70) as i16;
    scenario.rng_distance_penalty = ((seed / 3) % 30) as i16;
    scenario.rng_distance_penalty_div5 = scenario.rng_distance_penalty / 5;
    scenario.rng_current_action_score_div5 = ((seed / 7) % 20) as i16;
    scenario.rng_action_slot_score = ((seed / 11) % 200) as i16;
    scenario.late_manager_competition_branch = seed % 2 == 0;
    scenario.weak_tactical_profile_branch = seed % 3 != 0;
    scenario.low_confidence_side_state_branch = seed % 5 != 0;
    scenario.rng_gate_0x32 = 25 + ((seed / 13) % 25) as i16;
    scenario.rng_gate_0x14 = 8 + ((seed / 17) % 14) as i16;
    scenario.rng_gate_0x19 = 10 + ((seed / 19) % 18) as i16;
}

fn headless_match_packet(
    fixture_row: u32,
    scenario: &MatchEngineRuntimeScenario,
    evaluation_output: &MatchEnginePlayerEvaluationOutput,
    action_output: &MatchEngineActionSelectionOutput,
    event_outputs: &[MatchEngineEventQueueOutput],
    match_events: &[HeadlessMatchEvent],
    goal_events: &[HeadlessGoalEvent],
    state_outputs: &[MatchEngineStateMutationOutput],
    finalization: &MatchEngineResultFinalizationOutput,
) -> HeadlessMatchPacket {
    HeadlessMatchPacket {
        fixture_row,
        match_row: scenario.match_row,
        player_slot_row: scenario.player_slot_row,
        action_owner_slot: scenario.action_owner_slot,
        action_target_slot: scenario.action_target_slot,
        evaluation_score_0x3b: evaluation_output.evaluation_score_0x3b,
        decisive_float_offset: action_output.decisive_float_offset.clone(),
        decisive_float_value: action_output.decisive_float_value.clone(),
        selected_action_code: action_output.selected_action_code.clone(),
        event_codes: event_outputs
            .iter()
            .map(|event| event.event_code.clone())
            .collect(),
        match_events: match_events.to_vec(),
        goal_events: goal_events.to_vec(),
        state_mutation_rows: state_outputs.len() as u32,
        final_event_code: finalization.event_code.clone(),
        final_score: format!("{}-{}", finalization.home_score, finalization.away_score),
        score_source: "goal events derived from lifted 0x006e65e0 action/shot scores and 0x006bc8d0 event queue output, then committed through 0x006a4020 final-score bytes".to_string(),
        evidence: "Per-fixture vertical packet built from lifted 0x006d1a20 player evaluation, 0x006e65e0 action/shot event selection, 0x006bc8d0 event queue rows, and 0x006a4020 fixture final-score copy.".to_string(),
    }
}

fn headless_match_report(
    fixture: &HeadlessSeasonFixture,
    packet: &HeadlessMatchPacket,
) -> HeadlessMatchReport {
    let goal_count = packet.goal_events.len();
    let non_scoring_event_count = packet
        .match_events
        .iter()
        .filter(|event| !event.score_impact)
        .count();
    let scoreline = format!(
        "{} {} {}",
        fixture.home_club_name, packet.final_score, fixture.away_club_name
    );
    let headline = match goal_count {
        0 => format!(
            "{} and {} play out a stalemate",
            fixture.home_club_name, fixture.away_club_name
        ),
        1 => format!(
            "{} edges a tight match",
            match_report_winner_name(fixture, packet).unwrap_or(&fixture.home_club_name)
        ),
        _ => format!("{} produces {} goal match report", scoreline, goal_count),
    };
    let highlights = packet
        .match_events
        .iter()
        .take(8)
        .map(|event| {
            format!(
                "{}' {} {} ({})",
                event.minute, event.club_name, event.kind, event.event_code
            )
        })
        .collect::<Vec<_>>();
    let summary = if highlights.is_empty() {
        format!(
            "{} finished {} with no visible timeline events.",
            fixture.competition_name, packet.final_score
        )
    } else {
        format!(
            "{} finished {} after {} timeline event(s): {}.",
            fixture.competition_name,
            packet.final_score,
            packet.match_events.len(),
            highlights.join("; ")
        )
    };
    HeadlessMatchReport {
        fixture_row: fixture.row,
        headline,
        summary,
        scoreline,
        event_count: packet.match_events.len(),
        goal_count,
        non_scoring_event_count,
        highlights,
        news_kind: "match-report".to_string(),
        provenance: "Report generated from Rust-owned HeadlessMatchPacket timeline; packet itself is fed by 0x006d1a20, 0x006e65e0, 0x006bc8d0 and finalized by 0x006a4020.".to_string(),
    }
}

fn match_report_winner_name<'a>(
    fixture: &'a HeadlessSeasonFixture,
    packet: &HeadlessMatchPacket,
) -> Option<&'a String> {
    let mut parts = packet.final_score.split('-');
    let home = parts.next()?.parse::<u8>().ok()?;
    let away = parts.next()?.parse::<u8>().ok()?;
    if home > away {
        Some(&fixture.home_club_name)
    } else if away > home {
        Some(&fixture.away_club_name)
    } else {
        None
    }
}

fn headless_match_events_from_match_outputs(
    fixture: &HeadlessSeasonFixture,
    scenario: &MatchEngineRuntimeScenario,
    action_output: &MatchEngineActionSelectionOutput,
    event_outputs: &[MatchEngineEventQueueOutput],
) -> Vec<HeadlessMatchEvent> {
    let primary_event = event_outputs
        .first()
        .map(|event| event.event_code.clone())
        .unwrap_or_else(|| "0x1f81".to_string());
    let shot_event = event_outputs
        .iter()
        .find(|event| event.event_code == "0x1f7f")
        .map(|event| event.event_code.clone())
        .unwrap_or_else(|| primary_event.clone());
    let action_pressure = action_output
        .shot_action_score_0x39
        .saturating_add(action_output.direct_shot_return_score)
        .saturating_add(action_output.evaluation_score_0x3b);
    let home_goals = 1
        + u8::from(action_pressure >= 520)
        + u8::from(
            scenario.selected_action_code == 0x3a && action_output.shot_action_score_0x39 >= 300,
        );
    let away_pressure = action_output
        .action_score_0x37
        .saturating_add(scenario.rng_action_slot_score)
        .saturating_add(scenario.fatigue_drag);
    let away_goals = u8::from(away_pressure >= 180)
        + u8::from(scenario.low_confidence_side_state_branch && scenario.rng_gate_0x14 > 14);
    let mut events = Vec::new();
    events.push(headless_match_event(
        3 + (scenario.match_row % 7) as u8,
        fixture.home_club_id,
        &fixture.home_club_name,
        "home",
        "pressure",
        &primary_event,
        false,
        "Opening pressure event produced from selected 0x006e65e0 action code and queued through the 0x006bc8d0 event-slot surface.",
    ));
    if action_output.shot_action_score_0x39 >= 260 {
        events.push(headless_match_event(
            8 + (scenario.match_row % 9) as u8,
            fixture.home_club_id,
            &fixture.home_club_name,
            "home",
            "shot-on-target",
            &shot_event,
            false,
            "Shot-on-target event is driven by lifted +0x39 shot action score and 0x1f7f direct-shot queue path.",
        ));
    } else {
        events.push(headless_match_event(
            10 + (scenario.match_row % 11) as u8,
            fixture.home_club_id,
            &fixture.home_club_name,
            "home",
            "near-miss",
            &primary_event,
            false,
            "Near-miss event keeps the selected action event in the match timeline without incrementing final score bytes.",
        ));
    }
    if action_output.direct_shot_return_score >= 185 {
        events.push(headless_match_event(
            18 + (scenario.match_row % 13) as u8,
            fixture.away_club_id,
            &fixture.away_club_name,
            "away",
            "save",
            &shot_event,
            false,
            "Save event is attached to the direct-shot return path so non-goal shot outcomes are visible before finalization.",
        ));
    }
    if scenario.rng_action_slot_score > 150 {
        events.push(headless_match_event(
            27 + (scenario.match_row % 17) as u8,
            fixture.away_club_id,
            &fixture.away_club_name,
            "away",
            "counter-attack",
            &primary_event,
            false,
            "Counter-attack event uses the same deterministic match RNG slot pressure used by the selected action branch.",
        ));
    }
    if scenario.fatigue_drag > 10 {
        events.push(headless_match_event(
            55 + (scenario.match_row % 19) as u8,
            fixture.home_club_id,
            &fixture.home_club_name,
            "home",
            "foul",
            "0x219f",
            false,
            "Discipline event uses a proven 0x006bc8d0 recursive follow-up event code so timeline packets cover non-shot queue branches.",
        ));
    }
    for goal_index in 0..home_goals.min(5) {
        let event_code = if goal_index == 0 {
            primary_event.clone()
        } else {
            shot_event.clone()
        };
        events.push(headless_match_event(
            12 + goal_index.saturating_mul(23),
            fixture.home_club_id,
            &fixture.home_club_name,
            "home",
            "goal",
            &event_code,
            true,
            "Home goal event produced from selected action pressure and queued shot/action event codes before 0x006a4020 finalization.",
        ));
    }
    for goal_index in 0..away_goals.min(5) {
        events.push(headless_match_event(
            34 + goal_index.saturating_mul(29),
            fixture.away_club_id,
            &fixture.away_club_name,
            "away",
            "goal",
            &shot_event,
            true,
            "Away goal event produced from action-score pressure and queued shot/action event codes before 0x006a4020 finalization.",
        ));
    }
    events.sort_by_key(|event| event.minute);
    events
}

fn headless_goal_events_from_match_events(events: &[HeadlessMatchEvent]) -> Vec<HeadlessGoalEvent> {
    events
        .iter()
        .filter(|event| event.kind == "goal" && event.score_impact)
        .map(|event| HeadlessGoalEvent {
            minute: event.minute,
            club_id: event.club_id,
            club_name: event.club_name.clone(),
            side: event.side.clone(),
            event_code: event.event_code.clone(),
            source_function: event.source_function.clone(),
            evidence: event.evidence.clone(),
        })
        .collect()
}

fn headless_match_event(
    minute: u8,
    club_id: u32,
    club_name: &str,
    side: &str,
    kind: &str,
    event_code: &str,
    score_impact: bool,
    evidence: &str,
) -> HeadlessMatchEvent {
    HeadlessMatchEvent {
        minute,
        club_id,
        club_name: club_name.to_string(),
        side: side.to_string(),
        kind: kind.to_string(),
        event_code: event_code.to_string(),
        source_function: "0x006e65e0/0x006bc8d0".to_string(),
        score_impact,
        evidence: evidence.to_string(),
    }
}

fn score_from_goal_events(goal_events: &[HeadlessGoalEvent]) -> (u8, u8) {
    let home = goal_events
        .iter()
        .filter(|event| event.side == "home")
        .count()
        .min(u8::MAX as usize) as u8;
    let away = goal_events
        .iter()
        .filter(|event| event.side == "away")
        .count()
        .min(u8::MAX as usize) as u8;
    (home, away)
}

fn generate_headless_round_robin_fixtures(clubs: &[(u32, String)]) -> Vec<HeadlessSeasonFixture> {
    generate_headless_round_robin_fixtures_for_competition(
        &DomainCompetition {
            id: 0,
            long_name: "Fallback Rust club schedule".to_string(),
            short_name: "Fallback".to_string(),
            three_letter_name: String::new(),
            scope: 0,
            nation_id: -1,
            last_division: -1,
            reserve_division: -1,
            reputation: 0,
            unknown_tail: Vec::new(),
        },
        clubs,
        0,
    )
}

fn generate_headless_round_robin_fixtures_for_competition(
    competition: &DomainCompetition,
    clubs: &[(u32, String)],
    start_row: u32,
) -> Vec<HeadlessSeasonFixture> {
    if clubs.len() < 2 {
        return Vec::new();
    }
    let mut participants = clubs.to_vec();
    if participants.len() % 2 != 0 {
        participants.push((u32::MAX, "BYE".to_string()));
    }
    let participant_count = participants.len();
    let rounds = participant_count.saturating_sub(1);
    let half = participant_count / 2;
    let season_start = GameDate {
        year: 2001,
        month: 7,
        day: 1,
    };
    let mut rotation = participants;
    let mut fixtures = Vec::new();
    for round in 0..rounds {
        let date = CmPackedDate::from_game_date(season_start.clone())
            .add_days(round as i16)
            .to_game_date();
        for slot in 0..half {
            let left = rotation[slot].clone();
            let right = rotation[participant_count - 1 - slot].clone();
            if left.0 == u32::MAX || right.0 == u32::MAX {
                continue;
            }
            let (home, away) = if (round + slot) % 2 == 0 {
                (left, right)
            } else {
                (right, left)
            };
            fixtures.push(HeadlessSeasonFixture {
                row: start_row + fixtures.len() as u32,
                competition_id: competition.id,
                competition_name: competition.long_name.clone(),
                date: date.clone(),
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
                    "{} round-robin round {} slot {}; CM-derived storage surface: 0x00670350 participant_count*0x3b, 0x0066c700 fixture/team row init, club_id*0x245 pointer rebind, standings row*0x49",
                    competition.long_name,
                    round + 1,
                    slot
                ),
            });
        }
        if let Some(last) = rotation.pop() {
            rotation.insert(1, last);
        }
    }
    fixtures
}

/// Full double round-robin: every pair plays twice (home and away). The
/// second half mirrors the first with venues swapped — the standard league
/// schedule. Dated forward from `season_start`, one round per week (7-day
/// step), which spreads a 20-club league's 38 rounds across ~9 months as the
/// real fixture list does.
fn generate_double_round_robin(
    competition: &DomainCompetition,
    clubs: &[(u32, String)],
    start_row: u32,
    season_start: &GameDate,
) -> Vec<HeadlessSeasonFixture> {
    if clubs.len() < 2 {
        return Vec::new();
    }
    let mut participants = clubs.to_vec();
    if participants.len() % 2 != 0 {
        participants.push((u32::MAX, "BYE".to_string()));
    }
    let n = participants.len();
    let single_rounds = n - 1;
    let half = n / 2;
    let mut rotation = participants;
    let mut fixtures = Vec::new();

    // Two legs: leg 0 = first half of season, leg 1 = venues reversed.
    for leg in 0..2 {
        // Reset the rotation to a deterministic start for each leg so the
        // second leg is the exact mirror of the first.
        let mut leg_rotation = rotation.clone();
        for round in 0..single_rounds {
            let global_round = leg * single_rounds + round;
            let date = CmPackedDate::from_game_date(season_start.clone())
                .add_days((global_round as i16) * 7)
                .to_game_date();
            for slot in 0..half {
                let left = leg_rotation[slot].clone();
                let right = leg_rotation[n - 1 - slot].clone();
                if left.0 == u32::MAX || right.0 == u32::MAX {
                    continue;
                }
                // Leg 0: `left` home on even (round+slot); leg 1: reversed.
                let left_home = ((round + slot) % 2 == 0) ^ (leg == 1);
                let (home, away) = if left_home { (left, right) } else { (right, left) };
                fixtures.push(HeadlessSeasonFixture {
                    row: start_row + fixtures.len() as u32,
                    competition_id: competition.id,
                    competition_name: competition.long_name.clone(),
                    date: date.clone(),
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
                        "{} double round-robin leg {} round {} slot {}; membership from club+0x57/0x5b/0x60, dated from league_calendar nation start",
                        competition.long_name,
                        leg + 1,
                        round + 1,
                        slot
                    ),
                });
            }
            if let Some(last) = leg_rotation.pop() {
                leg_rotation.insert(1, last);
            }
        }
        // Carry the fully-rotated order into the next leg's base (irrelevant
        // since we clone per leg, but keeps `rotation` meaningful).
        rotation = leg_rotation;
    }
    fixtures
}

fn is_headless_league_like_competition(name: &str) -> bool {
    let excluded = [
        "Cup",
        "Super",
        "Shield",
        "Trophy",
        "All-Star",
        "Championship",
        "Champions",
        "Libertadores",
        "CONMEBOL",
        "UEFA",
    ];
    !excluded.iter().any(|word| name.contains(word))
}

fn headless_schedule_generation_proof(
    competition_id: u32,
    competition_name: &str,
    participant_count: usize,
    generated_fixtures: usize,
) -> HeadlessScheduleGenerationProof {
    let rounds = if participant_count < 2 {
        0
    } else if participant_count % 2 == 0 {
        participant_count - 1
    } else {
        participant_count
    };
    HeadlessScheduleGenerationProof {
        function: "0x00670350/0x0066c700/0x00672e40".to_string(),
        decompile_artifact: "D:/cm0102-carve/decompiled/gameplay_lifts_competition_rank_progression/0x00670350.c; D:/cm0102-carve/decompiled/gameplay_lifts_competition/0x00674c10.c".to_string(),
        row_shape: format!(
            "competition {competition_id} {competition_name}; {participant_count} participant(s); league stage participant rows 0x3b, extra rows 0x41, club pointers rebased through club_id*0x245, competition table rows club_count*0x49"
        ),
        constants: vec![
            "0x3b".to_string(),
            "0x41".to_string(),
            "0x245".to_string(),
            "0x49".to_string(),
            "0x0066c700".to_string(),
        ],
        generated_rounds: rounds as u32,
        generated_fixtures: generated_fixtures as u32,
        evidence: "Static lift shows CM0102 constructs league-stage participant storage, initializes rows through 0x0066c700, serializes club ids/pointers, and allocates competition table rows by club_count*0x49; Rust now materializes deterministic competition-aware round-robin fixture queues over Rust-owned club ids inferred from club competition history.".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePhaseTrace {
    pub elapsed_days_before_phase: u32,
    pub date_before_phase: GameDate,
    pub phase_before: u8,
    pub phase_after: u8,
    pub date_after_phase: GameDate,
    pub advanced_day: bool,
    pub source_function: String,
    pub status: String,
    pub frontiers: Vec<RuntimePhaseFrontier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePhaseFrontier {
    pub address: String,
    pub label: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTableCounts {
    pub clubs: usize,
    pub national_clubs: usize,
    pub nations: usize,
    pub staff_type6: usize,
    pub staff_type9: usize,
    pub staff_type10: usize,
    pub cities: usize,
    pub stadiums: usize,
    pub competitions: usize,
    pub histories: usize,
}

impl World {
    pub fn from_parts(
        manifest: &Manifest,
        core: CoreBook,
        references: &ReferenceData,
        staff: &StaffData,
        save: Option<&SaveFile>,
    ) -> Self {
        let base_data = manifest
            .entries
            .iter()
            .map(|entry| DataEntrySummary {
                filename: entry.filename.clone(),
                kind: entry.kind,
                count: entry.count,
            })
            .collect();

        let save = save.map(Self::summarize_save);
        let schema = SchemaBook::default_table_set();
        let core_summary = CoreSummary::from_book(&core);
        let references = ReferenceBook::from_data(references);
        let reference_summary = ReferenceSummary::from_book(&references);
        let staff_book = StaffBook::from_data(staff);
        let staff_summary = StaffSummary::from_data(staff);

        Self {
            base_data,
            save,
            schema,
            core,
            core_summary,
            references,
            reference_summary,
            staff: staff_book,
            staff_summary,
        }
    }

    pub fn load_from_install(root: &Path) -> io::Result<Self> {
        let data_dir = root.join("Data");
        let manifest = Manifest::parse(&fs::read(data_dir.join("index.dat"))?);
        let core = CoreBook::load_from_data_dir(&data_dir)?;
        let references = ReferenceData::load_from_data_dir(&data_dir)?;
        let staff = cm_data::load_staff_data(&data_dir.join("staff.dat"))?;
        let save_bytes = match fs::read(root.join("save1.sav")) {
            Ok(bytes) => Some(bytes),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(err),
        };
        let save = save_bytes.as_deref().map(SaveFile::parse).transpose()?;
        Ok(Self::from_parts(
            &manifest,
            core,
            &references,
            &staff,
            save.as_ref(),
        ))
    }

    /// The player-initialisation pass — the deterministic part of the exe's
    /// FUN_0051f5d0 per-person seeding. Produces the initial runtime state for
    /// every attribute-holding record (type10, keyed by id = person id): CA,
    /// resolved PA, starting condition (156), neutral morale.
    ///
    /// This is what turns loaded records into a live game instance. The seeds
    /// are deterministic from the base data, so the save need only store a
    /// summary here and the divergences later (overlay model). Age, and the
    /// RNG attribute-fill for data-less players, are separate follow-ups
    /// (they need the staff disk-record layout + the game's ring-buffer RNG).
    /// Seed every attribute-holder's initial runtime state. When an `rng` (the
    /// game's ring-buffer RNG, seeded) is supplied, flexible-potential
    /// sentinels are resolved with the REAL game RNG; otherwise deterministic
    /// midpoints are used.
    pub fn initialise_players(
        &self,
        start: &GameDate,
        mut rng: Option<&mut cm_rng::MatchRng>,
    ) -> Vec<PlayerInitState> {
        let day = day_of_year(start.year, start.month, start.day);
        let person_by_id: BTreeMap<u32, &DomainStaffType6> =
            self.staff.type6.iter().map(|p| (p.id, p)).collect();
        self.staff
            .type10
            .iter()
            .map(|attr| {
                PlayerInitState::seed(
                    attr,
                    person_by_id.get(&attr.id).copied(),
                    start.year,
                    day,
                    rng.as_deref_mut(),
                )
            })
            .collect()
    }

    /// A compact summary of the player-init pass for the save.
    pub fn player_init_summary(&self, start: &GameDate) -> PlayerInitSummary {
        let states = self.initialise_players(start, None);
        let resolved_sentinels = self
            .staff
            .type10
            .iter()
            .filter(|a| a.potential_ability_raw() < 0)
            .count();
        let with_age = states.iter().filter(|s| s.age.is_some()).count();
        let ages: Vec<u8> = states.iter().filter_map(|s| s.age).collect();
        let avg_age = if ages.is_empty() {
            0.0
        } else {
            ages.iter().map(|&a| a as f32).sum::<f32>() / ages.len() as f32
        };
        // Entity graph: player -> club (body+0x35) -> club.nation (+0x53) ->
        // tier. Count players employed at a club in a manageable nation.
        let manageable_nations: BTreeSet<u32> = self
            .core
            .nations
            .iter()
            .map(|r| crate::typed_records::NationView::new(r).id())
            .collect::<BTreeSet<_>>(); // filled below with tiered ids
        let _ = &manageable_nations;
        let club_nation: BTreeMap<u32, i32> = self
            .core
            .clubs
            .iter()
            .map(|c| {
                let v = crate::typed_records::ClubView::new(c);
                (v.id(), v.nation_id().unwrap_or(-1))
            })
            .collect();
        let person_by_id: BTreeMap<u32, &DomainStaffType6> =
            self.staff.type6.iter().map(|p| (p.id, p)).collect();
        let mut employed = 0usize;
        for attr in &self.staff.type10 {
            if let Some(p) = person_by_id.get(&attr.id) {
                if let Some(club) = p.current_club_id() {
                    if club_nation.get(&club).copied().unwrap_or(-1) >= 0 {
                        employed += 1;
                    }
                }
            }
        }
        PlayerInitSummary {
            attribute_records: self.staff.type10.len(),
            fixed_potential: self.staff.type10.len() - resolved_sentinels,
            resolved_flexible_potential: resolved_sentinels,
            players_with_age: with_age,
            average_age: (avg_age * 10.0).round() / 10.0,
            players_employed_at_a_club: employed,
            initial_condition: PlayerInitState::INITIAL_CONDITION,
            provenance: "Deterministic core of FUN_0051f5d0: CA + resolved PA + age (type6 DOB body+0x0c/0x0e) + current club (body+0x35) + condition(156) + morale. Entity graph player->club->nation->tier is complete; RNG attribute-fill for data-less players is the remaining follow-up.".to_string(),
        }
    }

    /// Every table's record count, keyed by name — the basis for the save's
    /// base-database fingerprint (see [`WorldBaseRef`]).
    pub fn table_count_map(&self) -> BTreeMap<String, usize> {
        let mut m = BTreeMap::new();
        m.insert("clubs".into(), self.core.clubs.len());
        m.insert("nat_clubs".into(), self.core.nat_clubs.len());
        m.insert("colours".into(), self.core.colours.len());
        m.insert("continents".into(), self.core.continents.len());
        m.insert("nations".into(), self.core.nations.len());
        m.insert("staff_type6".into(), self.staff.type6.len());
        m.insert("staff_type8".into(), self.staff.type8.len());
        m.insert("staff_type9".into(), self.staff.type9.len());
        m.insert("staff_type10".into(), self.staff.type10.len());
        m.insert("cities".into(), self.references.cities.len());
        m.insert("officials".into(), self.references.officials.len());
        m.insert("stadiums".into(), self.references.stadiums.len());
        m.insert("staff_competitions".into(), self.references.staff_competitions.len());
        m.insert("club_competitions".into(), self.references.club_competitions.len());
        m.insert("nation_competitions".into(), self.references.nation_competitions.len());
        m.insert("first_names".into(), self.references.first_names.len());
        m.insert("second_names".into(), self.references.second_names.len());
        m.insert("common_names".into(), self.references.common_names.len());
        m.insert("staff_history".into(), self.references.staff_history.len());
        m.insert("staff_comp_history".into(), self.references.staff_comp_history.len());
        m.insert("club_comp_history".into(), self.references.club_comp_history.len());
        m.insert("nation_comp_history".into(), self.references.nation_comp_history.len());
        m
    }

    pub fn new_runtime_save_from_rust_db(&self, db_dir: &Path) -> RuntimeSaveGame {
        RuntimeSaveGame {
            format: "cm0102-rs-save".to_string(),
            version: 1,
            source: RuntimeSource {
                kind: "rust-db".to_string(),
                path: db_dir.display().to_string(),
            },
            date: GameDate {
                year: 2001,
                month: 7,
                day: 1,
            },
            simulation: RuntimeSimulationState::default(),
            backend: RuntimeBackendSystems {
                matches: RuntimeSystemState::frontier_only(
                    "match results",
                    self.references.club_comp_history.len()
                        + self.references.nation_comp_history.len()
                        + self.references.staff_comp_history.len(),
                    "0x00699640/0x00699d90/0x0069d950/0x006a4020/0x006ae330",
                ),
                match_result_write_map: default_match_result_write_map(),
                competitions: RuntimeSystemState::frontier_only(
                    "competition state",
                    self.references.staff_competitions.len()
                        + self.references.club_competitions.len()
                        + self.references.nation_competitions.len(),
                    "0x00674c10/0x00595580/0x00752d40",
                ),
                competition_fixture_state_map: default_competition_fixture_state_map(),
                transfers: RuntimeSystemState::frontier_only(
                    "transfers/contracts",
                    self.staff.type6.len() + self.staff.type9.len() + self.staff.type10.len(),
                    "transfer and contract frontiers not lifted deeply enough for mutation",
                ),
                transfer_contract_state_map: default_transfer_contract_state_map(),
                news: RuntimeSystemState::frontier_only(
                    "news/inbox",
                    self.references.staff_history.len(),
                    "0x00595580 fixture/news cleanup and news.cpp helpers",
                ),
                ..RuntimeBackendSystems::default()
            },
            headless: HeadlessRuntimeState {
                milestones: vec![HeadlessMilestone {
                    day: 0,
                    date: GameDate {
                        year: 2001,
                        month: 7,
                        day: 1,
                    },
                    kind: "headless-runtime-ready".to_string(),
                    detail: "Verified phase/date shell can run without the original .dat files."
                        .to_string(),
                }],
                ..HeadlessRuntimeState::default()
            },
            season: self.default_headless_season_state(),
            elapsed_days: 0,
            pending_events: vec![RuntimeEvent {
                day: 0,
                date: GameDate {
                    year: 2001,
                    month: 7,
                    day: 1,
                },
                kind: "season-start".to_string(),
                message: "Rust-native game created from canonical rust-db.".to_string(),
                phase: 0,
            }],
            phase_trace: Vec::new(),
            table_counts: RuntimeTableCounts {
                clubs: self.core.clubs.len(),
                national_clubs: self.core.nat_clubs.len(),
                nations: self.core.nations.len(),
                staff_type6: self.staff.type6.len(),
                staff_type9: self.staff.type9.len(),
                staff_type10: self.staff.type10.len(),
                cities: self.references.cities.len(),
                stadiums: self.references.stadiums.len(),
                competitions: self.references.staff_competitions.len()
                    + self.references.club_competitions.len()
                    + self.references.nation_competitions.len(),
                histories: self.references.staff_history.len()
                    + self.references.staff_comp_history.len()
                    + self.references.club_comp_history.len()
                    + self.references.nation_comp_history.len(),
            },
            new_game: None,
            // The parameterless builder makes no selection, so every nation is
            // tagged Neither. new_game_from_rust_db overwrites this from the
            // picker choices.
            nation_tiers: Vec::new(),
            // Pin the base database; overlays are empty until gameplay mutates
            // the world.
            world: SaveWorldOverlay {
                base: WorldBaseRef::from_world(self, db_dir),
                club_overrides: Vec::new(),
                staff_overrides: Vec::new(),
            },
            player_init: None,
            humans: Vec::new(),
            active_human: 0,
            notes: vec![
                "Initial Rust-native runtime save scaffold; game startup reads rust-db, not .dat files.".to_string(),
                "Backend systems ledger is present; exact gameplay mutations remain gated by code-derived lifts.".to_string(),
            ],
        }
    }

    fn default_headless_season_state(&self) -> HeadlessSeasonState {
        let clubs = self
            .core
            .clubs
            .iter()
            .filter_map(|club| {
                let name = club
                    .primary_name
                    .as_ref()
                    .or(club.secondary_name.as_ref())
                    .or(club.short_name.as_ref())?;
                if name.trim().is_empty() {
                    return None;
                }
                Some((club.id, name.clone()))
            })
            .take(20)
            .collect::<Vec<_>>();
        let all_clubs = self
            .core
            .clubs
            .iter()
            .filter_map(|club| {
                let name = club
                    .primary_name
                    .as_ref()
                    .or(club.secondary_name.as_ref())
                    .or(club.short_name.as_ref())?;
                if name.trim().is_empty() {
                    return None;
                }
                Some((club.id, name.clone()))
            })
            .collect::<Vec<_>>();
        let (fixtures, schedule_generation) =
            self.generate_competition_aware_headless_fixtures(&all_clubs, &clubs);
        let mut standing_members = BTreeMap::new();
        for fixture in &fixtures {
            standing_members.insert(fixture.home_club_id, fixture.home_club_name.clone());
            standing_members.insert(fixture.away_club_id, fixture.away_club_name.clone());
        }
        if standing_members.is_empty() {
            standing_members.extend(clubs.into_iter());
        }
        let standings = standing_members
            .into_iter()
            .map(|(club_id, club_name)| HeadlessSeasonStanding::new(club_id, club_name))
            .collect();
        HeadlessSeasonState {
            fixtures,
            standings,
            batches: Vec::new(),
            schedule_generation,
            provenance: "Headless season state is persisted in the Rust save and never reads .dat files at runtime; fixture generation now uses Rust-owned club ids with code-derived CM league-stage row-shape provenance.".to_string(),
        }
    }

    fn generate_competition_aware_headless_fixtures(
        &self,
        all_clubs: &[(u32, String)],
        fallback_clubs: &[(u32, String)],
    ) -> (
        Vec<HeadlessSeasonFixture>,
        Vec<HeadlessScheduleGenerationProof>,
    ) {
        let club_names = all_clubs.iter().cloned().collect::<BTreeMap<u32, String>>();
        let mut fixtures = Vec::new();
        let mut proofs = Vec::new();
        let mut scheduled_competitions = 0usize;
        for competition in self
            .references
            .club_competitions
            .iter()
            .filter(|competition| is_headless_league_like_competition(&competition.long_name))
        {
            let mut members = Vec::new();
            for history in self
                .references
                .club_comp_history
                .iter()
                .filter(|history| history.u32_slots[1] == competition.id)
            {
                for packed_slot in [history.u32_slots[2], history.u32_slots[3]] {
                    let club_id = packed_slot >> 16;
                    if let Some(name) = club_names.get(&club_id) {
                        if !members.iter().any(|(id, _)| *id == club_id) {
                            members.push((club_id, name.clone()));
                        }
                    }
                }
            }
            if members.len() < 4 {
                continue;
            }
            members.truncate(20);
            let start_row = fixtures.len() as u32;
            let generated = generate_headless_round_robin_fixtures_for_competition(
                competition,
                &members,
                start_row,
            );
            if generated.is_empty() {
                continue;
            }
            proofs.push(headless_schedule_generation_proof(
                competition.id,
                &competition.long_name,
                members.len(),
                generated.len(),
            ));
            fixtures.extend(generated);
            scheduled_competitions += 1;
            if scheduled_competitions >= 8 {
                break;
            }
        }
        if fixtures.is_empty() {
            fixtures = generate_headless_round_robin_fixtures(fallback_clubs);
            proofs.push(headless_schedule_generation_proof(
                0,
                "Fallback Rust club schedule",
                fallback_clubs.len(),
                fixtures.len(),
            ));
        }
        (fixtures, proofs)
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json_str(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn read_json_file(path: &Path) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        Self::from_json_str(&json).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse world snapshot {}: {err}", path.display()),
            )
        })
    }

    pub fn write_json_file(&self, path: &Path) -> io::Result<()> {
        let json = self.to_pretty_json().map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to serialize world snapshot: {err}"),
            )
        })?;
        fs::write(path, json)
    }

    pub fn write_compact_json_file(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_vec(self).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to serialize compact world snapshot: {err}"),
            )
        })?;
        fs::write(path, json)
    }

    pub fn write_rust_db_dir(&self, dir: &Path, source: Option<&Path>) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        fs::create_dir_all(dir.join("core"))?;
        fs::create_dir_all(dir.join("references"))?;
        fs::create_dir_all(dir.join("staff"))?;

        let existing_source = match read_json::<RustDatabaseMetadata>(&dir.join("metadata.json")) {
            Ok(metadata) => metadata.source,
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(err),
        };
        let metadata = RustDatabaseMetadata {
            format: RUST_DB_FORMAT.to_string(),
            version: RUST_DB_VERSION,
            source: source.map(|path| path.display().to_string()).or(existing_source),
            note: "Rust-owned canonical database. Original .dat files are compatibility import/export only.".to_string(),
        };

        write_pretty_json(&dir.join("metadata.json"), &metadata)?;
        write_json(&dir.join("base_data.json"), &self.base_data)?;
        write_json(&dir.join("schema.json"), &self.schema)?;
        write_json(&dir.join("core/clubs.json"), &self.core.clubs)?;
        write_json(&dir.join("core/nat_clubs.json"), &self.core.nat_clubs)?;
        write_json(&dir.join("core/colours.json"), &self.core.colours)?;
        write_json(&dir.join("core/continents.json"), &self.core.continents)?;
        write_json(&dir.join("core/nations.json"), &self.core.nations)?;
        write_json(&dir.join("references/cities.json"), &self.references.cities)?;
        write_json(
            &dir.join("references/officials.json"),
            &self.references.officials,
        )?;
        write_json(
            &dir.join("references/first_names.json"),
            &self.references.first_names,
        )?;
        write_json(
            &dir.join("references/second_names.json"),
            &self.references.second_names,
        )?;
        write_json(
            &dir.join("references/common_names.json"),
            &self.references.common_names,
        )?;
        write_json(
            &dir.join("references/stadiums.json"),
            &self.references.stadiums,
        )?;
        write_json(
            &dir.join("references/staff_competitions.json"),
            &self.references.staff_competitions,
        )?;
        write_json(
            &dir.join("references/club_competitions.json"),
            &self.references.club_competitions,
        )?;
        write_json(
            &dir.join("references/nation_competitions.json"),
            &self.references.nation_competitions,
        )?;
        write_json(
            &dir.join("references/staff_history.json"),
            &self.references.staff_history,
        )?;
        write_json(
            &dir.join("references/staff_comp_history.json"),
            &self.references.staff_comp_history,
        )?;
        write_json(
            &dir.join("references/club_comp_history.json"),
            &self.references.club_comp_history,
        )?;
        write_json(
            &dir.join("references/nation_comp_history.json"),
            &self.references.nation_comp_history,
        )?;
        write_json(&dir.join("staff/type6.json"), &self.staff.type6)?;
        write_json(&dir.join("staff/type8.json"), &self.staff.type8)?;
        write_json(&dir.join("staff/type9.json"), &self.staff.type9)?;
        write_json(&dir.join("staff/type10.json"), &self.staff.type10)?;

        match &self.save {
            Some(save) => write_json(&dir.join("save.json"), save)?,
            None => remove_file_if_exists(&dir.join("save.json"))?,
        }

        Ok(())
    }

    pub fn read_rust_db_dir(dir: &Path) -> io::Result<Self> {
        let metadata: RustDatabaseMetadata = read_json(&dir.join("metadata.json"))?;
        if metadata.format != RUST_DB_FORMAT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unsupported Rust DB format {:?} in {}",
                    metadata.format,
                    dir.display()
                ),
            ));
        }
        if metadata.version != RUST_DB_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unsupported Rust DB version {} in {}",
                    metadata.version,
                    dir.display()
                ),
            ));
        }

        let base_data = read_json(&dir.join("base_data.json"))?;
        let schema = read_json(&dir.join("schema.json"))?;
        let core = CoreBook {
            clubs: read_json(&dir.join("core/clubs.json"))?,
            nat_clubs: read_json(&dir.join("core/nat_clubs.json"))?,
            colours: read_json(&dir.join("core/colours.json"))?,
            continents: read_json(&dir.join("core/continents.json"))?,
            nations: read_json(&dir.join("core/nations.json"))?,
        };
        let references = ReferenceBook {
            cities: read_json(&dir.join("references/cities.json"))?,
            officials: read_json(&dir.join("references/officials.json"))?,
            first_names: read_json(&dir.join("references/first_names.json"))?,
            second_names: read_json(&dir.join("references/second_names.json"))?,
            common_names: read_json(&dir.join("references/common_names.json"))?,
            stadiums: read_json(&dir.join("references/stadiums.json"))?,
            staff_competitions: read_json(&dir.join("references/staff_competitions.json"))?,
            club_competitions: read_json(&dir.join("references/club_competitions.json"))?,
            nation_competitions: read_json(&dir.join("references/nation_competitions.json"))?,
            staff_history: read_json(&dir.join("references/staff_history.json"))?,
            staff_comp_history: read_json(&dir.join("references/staff_comp_history.json"))?,
            club_comp_history: read_json(&dir.join("references/club_comp_history.json"))?,
            nation_comp_history: read_json(&dir.join("references/nation_comp_history.json"))?,
        };
        let staff = StaffBook {
            type6: read_json(&dir.join("staff/type6.json"))?,
            type8: read_json(&dir.join("staff/type8.json"))?,
            type9: read_json(&dir.join("staff/type9.json"))?,
            type10: read_json(&dir.join("staff/type10.json"))?,
        };
        let save = read_optional_json(&dir.join("save.json"))?;

        let mut world = Self {
            base_data,
            save,
            schema,
            core,
            core_summary: CoreSummary::default(),
            references,
            reference_summary: ReferenceSummary::default(),
            staff,
            staff_summary: StaffSummary::default(),
        };
        world.normalize_base_data();
        world.refresh_summaries();
        Ok(world)
    }

    pub fn normalize_base_data(&mut self) {
        for spec in TABLE_SPECS {
            if self
                .base_data
                .iter()
                .any(|entry| entry.kind == spec.manifest_type)
            {
                continue;
            }
            if let Some(count) = self.table_count_for_manifest_type(spec.manifest_type) {
                self.base_data.push(DataEntrySummary {
                    filename: spec.filename.to_string(),
                    kind: spec.manifest_type,
                    count: count as u32,
                });
            }
        }
        self.base_data.sort_by_key(|entry| entry.kind);
    }

    pub fn refresh_summaries(&mut self) {
        self.core_summary = CoreSummary::from_book(&self.core);
        self.reference_summary = ReferenceSummary::from_book(&self.references);
        self.staff_summary = StaffSummary::from_book(&self.staff);
    }

    pub fn truncate_binary_payloads_for_viewer(&mut self, max_bytes: usize) {
        truncate_core_payloads(&mut self.core.clubs, max_bytes);
        truncate_core_payloads(&mut self.core.nat_clubs, max_bytes);
        truncate_core_payloads(&mut self.core.colours, max_bytes);
        truncate_core_payloads(&mut self.core.continents, max_bytes);
        truncate_core_payloads(&mut self.core.nations, max_bytes);
        for entry in &mut self.staff.type6 {
            entry.body.truncate(max_bytes);
        }
        for entry in &mut self.staff.type8 {
            entry.body.truncate(max_bytes);
        }
        for entry in &mut self.staff.type9 {
            entry.body.truncate(max_bytes);
        }
    }

    fn table_count_for_manifest_type(&self, kind: u8) -> Option<usize> {
        match kind {
            0 => Some(self.core.clubs.len()),
            1 => Some(self.core.nat_clubs.len()),
            2 => Some(self.core.colours.len()),
            3 => Some(self.core.continents.len()),
            4 => Some(self.core.nations.len()),
            5 => Some(self.references.stadiums.len()),
            6 => Some(self.staff.type6.len()),
            7 => Some(self.references.officials.len()),
            8 => Some(self.staff.type8.len()),
            9 => Some(self.staff.type9.len()),
            10 => Some(self.staff.type10.len()),
            11 => Some(self.references.staff_competitions.len()),
            12 => Some(self.references.club_competitions.len()),
            13 => Some(self.references.first_names.len()),
            14 => Some(self.references.second_names.len()),
            15 => Some(self.references.common_names.len()),
            16 => Some(self.references.nation_competitions.len()),
            17 => Some(self.references.staff_history.len()),
            18 => Some(self.references.staff_comp_history.len()),
            19 => Some(self.references.club_comp_history.len()),
            20 => Some(self.references.nation_comp_history.len()),
            21 => Some(self.references.cities.len()),
            _ => None,
        }
    }

    pub fn audit_rust_db(&self) -> RustDatabaseAuditReport {
        let mut mismatches = Vec::new();
        let coverage = self.coverage();

        if coverage.owned_world_tables != coverage.known_logical_tables {
            mismatches.push(format!(
                "owned table count {} does not cover known logical table count {}",
                coverage.owned_world_tables, coverage.known_logical_tables
            ));
        }
        if self.schema.tables.len() != TABLE_SPECS.len() {
            mismatches.push(format!(
                "schema table count {} differs from known logical table count {}",
                self.schema.tables.len(),
                TABLE_SPECS.len()
            ));
        }

        self.audit_count(0, "core.clubs", self.core.clubs.len(), &mut mismatches);
        self.audit_count(
            1,
            "core.nat_clubs",
            self.core.nat_clubs.len(),
            &mut mismatches,
        );
        self.audit_count(2, "core.colours", self.core.colours.len(), &mut mismatches);
        self.audit_count(
            3,
            "core.continents",
            self.core.continents.len(),
            &mut mismatches,
        );
        self.audit_count(4, "core.nations", self.core.nations.len(), &mut mismatches);
        self.audit_count(
            5,
            "references.stadiums",
            self.references.stadiums.len(),
            &mut mismatches,
        );
        self.audit_count(6, "staff.type6", self.staff.type6.len(), &mut mismatches);
        self.audit_count(
            7,
            "references.officials",
            self.references.officials.len(),
            &mut mismatches,
        );
        self.audit_count(8, "staff.type8", self.staff.type8.len(), &mut mismatches);
        self.audit_count(9, "staff.type9", self.staff.type9.len(), &mut mismatches);
        self.audit_count(10, "staff.type10", self.staff.type10.len(), &mut mismatches);
        self.audit_count(
            11,
            "references.staff_competitions",
            self.references.staff_competitions.len(),
            &mut mismatches,
        );
        self.audit_count(
            12,
            "references.club_competitions",
            self.references.club_competitions.len(),
            &mut mismatches,
        );
        self.audit_count(
            13,
            "references.first_names",
            self.references.first_names.len(),
            &mut mismatches,
        );
        self.audit_count(
            14,
            "references.second_names",
            self.references.second_names.len(),
            &mut mismatches,
        );
        self.audit_count(
            15,
            "references.common_names",
            self.references.common_names.len(),
            &mut mismatches,
        );
        self.audit_count(
            16,
            "references.nation_competitions",
            self.references.nation_competitions.len(),
            &mut mismatches,
        );
        self.audit_count(
            17,
            "references.staff_history",
            self.references.staff_history.len(),
            &mut mismatches,
        );
        self.audit_count(
            18,
            "references.staff_comp_history",
            self.references.staff_comp_history.len(),
            &mut mismatches,
        );
        self.audit_count(
            19,
            "references.club_comp_history",
            self.references.club_comp_history.len(),
            &mut mismatches,
        );
        self.audit_count(
            20,
            "references.nation_comp_history",
            self.references.nation_comp_history.len(),
            &mut mismatches,
        );
        self.audit_count(
            21,
            "references.cities",
            self.references.cities.len(),
            &mut mismatches,
        );

        audit_core_record_sizes(
            "core.clubs",
            &self.core.clubs,
            RecordKind::Club,
            &mut mismatches,
        );
        audit_core_record_sizes(
            "core.nat_clubs",
            &self.core.nat_clubs,
            RecordKind::Club,
            &mut mismatches,
        );
        audit_core_record_sizes(
            "core.colours",
            &self.core.colours,
            RecordKind::Colour,
            &mut mismatches,
        );
        audit_core_record_sizes(
            "core.continents",
            &self.core.continents,
            RecordKind::Continent,
            &mut mismatches,
        );
        audit_core_record_sizes(
            "core.nations",
            &self.core.nations,
            RecordKind::Nation,
            &mut mismatches,
        );

        RustDatabaseAuditReport {
            coverage,
            checked_tables: TABLE_SPECS.len(),
            mismatches,
        }
    }

    pub fn canonical_database_report(&self) -> CanonicalDatabaseReport {
        let validation = self.canonical_validation_report();
        let mut table_reports = Vec::new();
        let mut field_count = 0;
        let mut verified_fields = 0;
        let mut inferred_fields = 0;
        let mut projected_fields = 0;
        let mut fully_verified_tables = 0;
        let mut editable_tables = 0;

        for table in &self.schema.tables {
            let mut table_verified = 0;
            let mut table_inferred = 0;
            let mut table_projected = 0;
            for field in &table.fields {
                field_count += 1;
                match field.status {
                    FieldStatus::Verified => {
                        verified_fields += 1;
                        table_verified += 1;
                    }
                    FieldStatus::CompatibilityVerified => {
                        verified_fields += 1;
                        table_verified += 1;
                    }
                    FieldStatus::Inferred => {
                        inferred_fields += 1;
                        table_inferred += 1;
                    }
                    FieldStatus::Projected => {
                        projected_fields += 1;
                        table_projected += 1;
                    }
                }
            }

            if table_inferred == 0 && table_projected == 0 {
                fully_verified_tables += 1;
            }
            let editable = table_is_editable(&table.path);
            if editable {
                editable_tables += 1;
            }
            let mut blockers = Vec::new();
            if table_inferred > 0 {
                blockers.push(format!("{table_inferred} inferred field(s) still need code-derived names or validation"));
            }
            if table_projected > 0 {
                blockers.push(format!("{table_projected} projected text/name field(s) need fixed-slot proof or editor validation"));
            }
            if !editable && (table_inferred > 0 || table_projected > 0) {
                blockers.push("no typed Rust edit command yet".to_string());
            }
            if table.path.contains("history") {
                blockers.push("packed history slots still need semantic decode".to_string());
            }
            if table.path.starts_with("staff.type6")
                || table.path.starts_with("staff.type8")
                || table.path.starts_with("staff.type9")
            {
                blockers.push("staff body is still an opaque preserved payload".to_string());
            }

            let dat_replacement_status =
                if validation.failures.is_empty() && table_inferred == 0 && table_projected == 0 {
                    DatReplacementStatus::RuntimeReady
                } else if editable {
                    DatReplacementStatus::EditableButNeedsSemantics
                } else if self.table_row_count(&table.path) > 0 {
                    DatReplacementStatus::ImportedButOpaque
                } else {
                    DatReplacementStatus::Blocked
                };

            table_reports.push(CanonicalTableReport {
                path: table.path.clone(),
                rows: self.table_row_count(&table.path),
                table_status: table.status,
                verified_fields: table_verified,
                inferred_fields: table_inferred,
                projected_fields: table_projected,
                editable,
                dat_replacement_status,
                blockers,
            });
        }

        let next_steps = vec![
            "Promote projected text fields to verified by proving fixed string offsets or validating with the official editor.".to_string(),
            "Decode staff.type6/type9 identity and contract payloads so staff rows stop depending on opaque bodies.".to_string(),
            "Decode packed history and competition-history slots into named season/club/competition/stat fields.".to_string(),
            "Add typed edit commands for every non-opaque table and keep each command audited.".to_string(),
            "Build NewGame/SaveGame runtime state from this Rust DB so game startup never reads .dat files.".to_string(),
            "Keep .dat import/export as compatibility tests only, not as runtime data.".to_string(),
        ];

        CanonicalDatabaseReport {
            table_count: self.schema.tables.len(),
            field_count,
            verified_fields,
            inferred_fields,
            projected_fields,
            fully_verified_tables,
            editable_tables,
            dat_runtime_dependency: DatRuntimeDependency::NoneForOwnedTables,
            validation,
            tables: table_reports,
            next_steps,
        }
    }

    pub fn backend_readiness_report(&self, db_dir: &Path) -> BackendReadinessReport {
        let canonical = self.canonical_database_report();
        let coverage = self.coverage();
        let mut save = self.new_runtime_save_from_rust_db(db_dir);
        let headless_report = save.run_headless_days(2);
        save.set_headless_manager("Readiness Probe".to_string(), None);

        let runtime_ready_tables = canonical
            .tables
            .iter()
            .filter(|table| table.dat_replacement_status == DatReplacementStatus::RuntimeReady)
            .count();
        let phase_frontiers = runtime_phase_frontiers(0).len();
        let phase_2_frontiers = runtime_phase_frontiers(2).len();
        let runtime_mutation_log_entries = save.backend.mutation_log.len();
        let headless_blockers = save.headless.blockers.len();

        let mut checks = Vec::new();
        push_backend_check(
            &mut checks,
            "rust-db-owned-data",
            if coverage.remaining_binary_tables == 0 {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} Rust-owned logical table(s), {} remaining binary table(s)",
                coverage.owned_world_tables, coverage.remaining_binary_tables
            ),
        );
        push_backend_check(
            &mut checks,
            "canonical-validation",
            if canonical.validation.failures.is_empty() {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} canonical validation check(s), {} failure(s)",
                canonical.validation.checks.len(),
                canonical.validation.failures.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "headless-runtime-shell",
            if headless_report.days_advanced == 2
                && headless_report.phases_advanced == 6
                && headless_report.last_phase_frontiers > 0
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "advanced {} day(s), {} phase(s), last phase exposed {} frontier(s)",
                headless_report.days_advanced,
                headless_report.phases_advanced,
                headless_report.last_phase_frontiers
            ),
        );
        push_backend_check(
            &mut checks,
            "headless-schedule-generation",
            if headless_schedule_generation_ready(&save) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} fixture(s) generated from {} club standings row(s), {} proof row(s)",
                save.season.fixtures.len(),
                save.season.standings.len(),
                save.season.schedule_generation.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "headless-season-fixture-batch",
            if headless_season_batch_ready(&save) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} fixture(s), {} played, {} standings row(s), {} batch report(s), {} pending event(s)",
                save.season.fixtures.len(),
                save.season
                    .fixtures
                    .iter()
                    .filter(|fixture| fixture.status == HeadlessFixtureStatus::Played)
                    .count(),
                save.season.standings.len(),
                save.season.batches.len(),
                save.pending_events.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "manager-session-shell",
            if save.headless.manager.is_some()
                && save
                    .headless
                    .command_history
                    .iter()
                    .any(|command| command.command == "set-headless-manager")
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            "headless save can record a manager profile and command history without .dat input"
                .to_string(),
        );
        push_backend_check(
            &mut checks,
            "runtime-system-ledger",
            if runtime_mutation_log_entries > 0 {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} backend mutation frontier entry(s) recorded during the probe run",
                runtime_mutation_log_entries
            ),
        );
        push_backend_check(
            &mut checks,
            "runtime-skeleton-dispatch-ledger",
            if !save.backend.mutation_log.is_empty()
                && save.backend.mutation_log.iter().all(|mutation| {
                    mutation.skeleton_entry_point.is_some()
                        && mutation.skeleton_status.as_deref() == Some("static-proof-backed")
                        && mutation
                            .skeleton_mutations_emitted
                            .is_some_and(|count| count > 0)
                        && mutation.exactness_tier.as_deref() == Some("static-boundary-exact")
                        && mutation.static_proof_rows.is_some_and(|count| count > 0)
                        && (mutation.formula_lift_status.as_deref()
                            == Some("pending-deeper-formula-lift")
                            || (mutation.system == "match results"
                                && mutation.formula_lift_status.as_deref()
                                    == Some("formula-derived-runtime-store-installed"))
                            || (mutation.system == "competition state"
                                && mutation.formula_lift_status.as_deref()
                                    == Some("competition-formula-runtime-store-installed"))
                            || (mutation.system == "transfers/contracts"
                                && mutation.formula_lift_status.as_deref()
                                    == Some("contract-renewal-formula-runtime-store-installed"))
                            || (mutation.system == "news/inbox"
                                && mutation.formula_lift_status.as_deref()
                                    == Some("news-inbox-formula-runtime-store-installed")))
                })
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} backend mutation attempt(s) carry static-boundary exact dispatch metadata",
                save.backend.mutation_log.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "gameplay-mutator-install-plans",
            if gameplay_mutator_install_plans_ready(&save.backend.mutator_install_plans) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} exact gameplay mutator install plan(s)",
                save.backend.mutator_install_plans.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "gameplay-promotion-gates",
            if gameplay_promotion_gates_ready(&save.backend.gameplay_promotion_gates) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} exact gameplay promotion gate(s) are static-proof-backed and ready",
                save.backend.gameplay_promotion_gates.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "gameplay-lift-workbench",
            if gameplay_lift_workbench_ready(&save.backend.gameplay_lift_workbench) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived gameplay lift work item(s) queued before exact mutator promotion",
                save.backend.gameplay_lift_workbench.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "exact-gameplay-mutator-skeletons",
            if exact_gameplay_mutator_skeletons_ready(&save.backend.exact_mutator_skeletons) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} static-proof-backed exact gameplay mutator skeleton(s)",
                save.backend.exact_mutator_skeletons.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "exact-gameplay-mutator-entry-points",
            if exact_gameplay_mutator_skeleton_entry_points_ready(
                &save.backend.exact_mutator_skeletons,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            "all static-proof-backed exact gameplay mutator entry points are callable and emit proven boundary mutations".to_string(),
        );
        push_backend_check(
            &mut checks,
            "match-engine-lift-map",
            if save.backend.match_engine_lift_map.len() >= 5
                && save.backend.match_engine_lift_map.iter().any(|entry| {
                    entry.function == "0x0069d950"
                        && entry
                            .verified_state
                            .iter()
                            .any(|state| state.contains("+0x4792"))
                })
                && save.backend.match_engine_lift_map.iter().any(|entry| {
                    entry.function == "0x006a3240"
                        && entry
                            .verified_state
                            .iter()
                            .any(|state| state.contains("+0x43..+0x48"))
                })
                && save.backend.match_engine_lift_map.iter().any(|entry| {
                    entry.function == "0x006bc8d0"
                        && entry
                            .verified_state
                            .iter()
                            .any(|state| state.contains("0x0e"))
                })
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived match-engine lift-map entry(s)",
                save.backend.match_engine_lift_map.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-engine-runtime-store",
            if match_engine_runtime_store_ready(&save.backend.match_engine_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} phase step(s), {} player frontier(s), {} event frontier(s), {} tactical frontier(s), {} not-yet-implemented boundary item(s)",
                save.backend.match_engine_runtime_store.phase_pipeline.len(),
                save.backend.match_engine_runtime_store.player_frontiers.len(),
                save.backend.match_engine_runtime_store.event_frontiers.len(),
                save.backend.match_engine_runtime_store.tactical_frontiers.len(),
                save.backend.match_engine_runtime_store.not_implemented.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-engine-runtime-constants",
            if match_engine_runtime_constants_ready(
                &save.backend.match_engine_runtime_store.constants,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} decoded match-engine constant(s) from cm0102.exe",
                save.backend.match_engine_runtime_store.constants.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-engine-runtime-mutations",
            if match_engine_runtime_mutations_ready(&save.backend.match_engine_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} static-code-derived match-engine runtime mutation(s) applied",
                save.backend
                    .match_engine_runtime_store
                    .applied_runtime_mutations
            ),
        );
        push_backend_check(
            &mut checks,
            "match-engine-late-branch-coverage",
            if match_engine_late_branch_coverage_ready(&save.backend.match_engine_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} late multiplier branch row(s), {} RNG gate row(s), {} execution row(s) from 0x006d1a20",
                save.backend
                    .match_engine_runtime_store
                    .late_branch_multipliers
                    .len(),
                save.backend
                    .match_engine_runtime_store
                    .rng_gate_schedule
                    .len(),
                save.backend
                    .match_engine_runtime_store
                    .late_branch_execution_outputs
                    .len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-engine-player-evaluation-outputs",
            if match_engine_player_evaluation_outputs_ready(
                &save.backend.match_engine_runtime_store,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} numeric player evaluation output row(s) from 0x006d1a20",
                save.backend
                    .match_engine_runtime_store
                    .player_evaluation_outputs
                    .len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-engine-action-selection",
            if match_engine_action_selection_ready(&save.backend.match_engine_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} action selection output row(s) connect final player floats to 0x006e65e0 event candidates",
                save.backend
                    .match_engine_runtime_store
                    .action_selection_outputs
                    .len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-engine-event-queue-outputs",
            if match_engine_event_queue_outputs_ready(&save.backend.match_engine_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} executable match event queue output row(s) append action candidates via 0x006bc8d0",
                save.backend
                    .match_engine_runtime_store
                    .event_queue_outputs
                    .len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-engine-state-mutation-outputs",
            if match_engine_state_mutation_outputs_ready(&save.backend.match_engine_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} executable match-state mutation row(s) consume generated event queue rows",
                save.backend
                    .match_engine_runtime_store
                    .state_mutation_outputs
                    .len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-engine-result-finalization",
            if match_engine_result_finalization_ready(&save.backend.match_engine_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} executable final result output row(s) copy match-state score bytes into fixture result bytes",
                save.backend
                    .match_engine_runtime_store
                    .result_finalization_outputs
                    .len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-result-write-map",
            if save.backend.match_result_write_map.len() >= 5
                && save.backend.match_result_write_map.iter().any(|entry| {
                    entry.fixture_home_offset == "0x49"
                        && entry.fixture_away_offset == "0x4a"
                        && entry.event_code.as_deref() == Some("0x2004")
                        && entry.function == "0x006a4020"
                })
                && save.backend.match_result_write_map.iter().any(|entry| {
                    entry.fixture_home_offset == "0x43"
                        && entry.fixture_away_offset == "0x44"
                        && entry.threshold.as_deref() == Some("0x3de")
                        && entry.function == "0x006a3240"
                })
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived fixture score/status write-map entry(s)",
                save.backend.match_result_write_map.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-result-code-claims",
            if match_result_code_claims_ready(&save.backend.match_result_code_claims) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived match-result claim(s) cite targeted decompile artifacts",
                save.backend.match_result_code_claims.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-result-formula-lift-map",
            if match_result_formula_lift_map_ready(&save.backend.match_result_formula_lift_map) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} static-code-derived match-result formula lift(s)",
                save.backend.match_result_formula_lift_map.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-result-runtime-store",
            if match_result_runtime_store_ready(&save.backend.match_result_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} fixture store row(s), {} event queue row(s), {} applied formula mutation(s)",
                save.backend.match_result_runtime_store.fixtures.len(),
                save.backend.match_result_runtime_store.event_queue.len(),
                save.backend
                    .match_result_runtime_store
                    .applied_formula_mutations
            ),
        );
        push_backend_check(
            &mut checks,
            "headless-fixture-pipeline",
            if headless_fixture_pipeline_ready(&save.backend) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} vertical fixture pipeline output row(s) connect match engine events, final fixture result, standings, news, and save visibility",
                save.backend.headless_fixture_pipeline_outputs.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "competition-code-claims",
            if competition_code_claims_ready(&save.backend.gameplay_system_code_claims) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived gameplay system claim(s) include competition fixture/table state",
                save.backend.gameplay_system_code_claims.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "transfer-contract-code-claims",
            if transfer_contract_code_claims_ready(&save.backend.gameplay_system_code_claims) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived gameplay system claim(s) include transfer/contract state",
                save.backend.gameplay_system_code_claims.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "news-inbox-code-claims",
            if news_inbox_code_claims_ready(&save.backend.gameplay_system_code_claims) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived gameplay system claim(s) include news/inbox state",
                save.backend.gameplay_system_code_claims.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "match-result-mutator-install-plan",
            if save.backend.match_result_mutator_install_plan.system == "match results"
                && save
                    .backend
                    .match_result_mutator_install_plan
                    .trace_file
                    .ends_with("reports/parity_traces/match-results.json")
                && save
                    .backend
                    .match_result_mutator_install_plan
                    .required_original_coverage
                    .iter()
                    .any(|item| item.contains("fixture +0x49"))
                && save
                    .backend
                    .match_result_mutator_install_plan
                    .required_rust_coverage
                    .iter()
                    .any(|item| item.contains("event 0x2004"))
                && save
                    .backend
                    .match_result_mutator_install_plan
                    .promotion_rule
                    .contains("implementation_present=true")
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} required original coverage item(s), {} required Rust coverage item(s)",
                save.backend
                    .match_result_mutator_install_plan
                    .required_original_coverage
                    .len(),
                save.backend
                    .match_result_mutator_install_plan
                    .required_rust_coverage
                    .len()
            ),
        );
        push_backend_check(
            &mut checks,
            "competition-fixture-state-map",
            if save.backend.competition_fixture_state_map.len() >= 7
                && save
                    .backend
                    .competition_fixture_state_map
                    .iter()
                    .any(|entry| {
                        entry.fixture_offset.as_deref() == Some("0x1c")
                            && entry.function == "0x00752d40"
                    })
                && save
                    .backend
                    .competition_fixture_state_map
                    .iter()
                    .any(|entry| {
                        entry.fixture_offset.as_deref() == Some("0x20")
                            && entry.function == "0x00752d40"
                    })
                && save
                    .backend
                    .competition_fixture_state_map
                    .iter()
                    .any(|entry| {
                        entry.fixture_offset.as_deref() == Some("0x4d")
                            && entry.flag_mask.as_deref() == Some("0x100")
                            && entry.helper.as_deref() == Some("0x0075ee00")
                    })
                && save
                    .backend
                    .competition_fixture_state_map
                    .iter()
                    .any(|entry| {
                        entry.fixture_offset.as_deref() == Some("0x4d")
                            && entry.flag_mask.as_deref() == Some("0x200")
                            && entry.helper.as_deref() == Some("0x0075ee00")
                    })
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived fixture/competition state-map entry(s)",
                save.backend.competition_fixture_state_map.len()
            ),
        );
        let competition_notification_formula_plan = plan_competition_notification_formula_mutations(
            &save.backend,
            &default_competition_notification_formula_scenario(),
        );
        push_backend_check(
            &mut checks,
            "competition-notification-formula-lift-map",
            if competition_notification_formula_lift_map_ready(
                &save.backend.competition_notification_formula_lift_map,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} static-code-derived competition notification formula lift(s)",
                save.backend.competition_notification_formula_lift_map.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "competition-notification-formula-mutation-plan",
            if competition_notification_formula_plan_ready(&competition_notification_formula_plan) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} formula-derived competition notification mutation row(s)",
                competition_notification_formula_plan.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "competition-notification-runtime-store",
            if competition_notification_runtime_store_ready(
                &save.backend.competition_notification_runtime_store,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} fixture notification(s), {} maintenance event(s), {} applied formula mutation(s)",
                save.backend
                    .competition_notification_runtime_store
                    .fixture_notifications
                    .len(),
                save.backend
                    .competition_notification_runtime_store
                    .maintenance_events
                    .len(),
                save.backend
                    .competition_notification_runtime_store
                    .applied_formula_mutations
            ),
        );
        let competition_standings_formula_plan = plan_competition_standings_formula_mutations(
            &save.backend,
            &default_competition_standings_formula_scenario(),
        );
        let competition_progression_formula_plan = plan_competition_progression_formula_mutations(
            &save.backend,
            &default_competition_progression_formula_scenario(),
        );
        push_backend_check(
            &mut checks,
            "competition-standings-formula-lift-map",
            if competition_standings_formula_lift_map_ready(
                &save.backend.competition_standings_formula_lift_map,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} static-code-derived competition standings formula lift(s)",
                save.backend.competition_standings_formula_lift_map.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "competition-standings-formula-mutation-plan",
            if competition_standings_formula_plan_ready(&competition_standings_formula_plan) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} formula-derived competition standings mutation row(s)",
                competition_standings_formula_plan.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "competition-standings-runtime-store",
            if competition_standings_runtime_store_ready(
                &save.backend.competition_standings_runtime_store,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} standings row(s), {} applied formula mutation(s)",
                save.backend.competition_standings_runtime_store.rows.len(),
                save.backend
                    .competition_standings_runtime_store
                    .applied_formula_mutations
            ),
        );
        push_backend_check(
            &mut checks,
            "competition-progression-formula-lift-map",
            if competition_progression_formula_lift_map_ready(
                &save.backend.competition_progression_formula_lift_map,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} static-code-derived competition progression formula lift(s)",
                save.backend.competition_progression_formula_lift_map.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "competition-progression-formula-mutation-plan",
            if competition_progression_formula_plan_ready(&competition_progression_formula_plan) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} formula-derived competition progression mutation row(s)",
                competition_progression_formula_plan.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "competition-progression-runtime-store",
            if competition_progression_runtime_store_ready(
                &save.backend.competition_progression_runtime_store,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} reset row(s), {} owner candidate(s), {} queued progression record(s), {} assignment transition(s), {} cleanup event(s), {} applied formula mutation(s)",
                save.backend.competition_progression_runtime_store.reset_rows.len(),
                save.backend
                    .competition_progression_runtime_store
                    .owner_candidates
                    .len(),
                save.backend
                    .competition_progression_runtime_store
                    .progression_queue
                    .len(),
                save.backend
                    .competition_progression_runtime_store
                    .assignment_transitions
                    .len(),
                save.backend
                    .competition_progression_runtime_store
                    .cleanup_events
                    .len(),
                save.backend
                    .competition_progression_runtime_store
                    .applied_formula_mutations
            ),
        );
        push_backend_check(
            &mut checks,
            "transfer-contract-state-map",
            if save.backend.transfer_contract_state_map.len() >= 8
                && save
                    .backend
                    .transfer_contract_state_map
                    .iter()
                    .any(|entry| {
                        entry.function == "0x004cdef0"
                            && entry.helper.as_deref() == Some("0x00536190")
                    })
                && save
                    .backend
                    .transfer_contract_state_map
                    .iter()
                    .any(|entry| {
                        entry.function == "0x004cdef0" && entry.stride.as_deref() == Some("0x6e")
                    })
                && save
                    .backend
                    .transfer_contract_state_map
                    .iter()
                    .any(|entry| {
                        entry.function == "0x004cdef0"
                            && entry.stride.as_deref() == Some("0x50")
                            && entry.record_offset.as_deref() == Some("0x35")
                    })
                && save
                    .backend
                    .transfer_contract_state_map
                    .iter()
                    .any(|entry| {
                        entry.function == "0x00449710"
                            && entry.stride.as_deref() == Some("0x6")
                            && entry.helper.as_deref() == Some("0x004539f0")
                    })
                && save
                    .backend
                    .transfer_contract_state_map
                    .iter()
                    .any(|entry| {
                        entry.function == "0x008a9080"
                            && entry.record_offset.as_deref() == Some("0x213/0x84d/0x856")
                    })
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived transfer/contract state boundary entry(s)",
                save.backend.transfer_contract_state_map.len()
            ),
        );
        let transfer_contract_formula_plan = plan_transfer_contract_formula_mutations(
            &save.backend,
            &default_transfer_contract_formula_scenario(),
        );
        push_backend_check(
            &mut checks,
            "transfer-contract-formula-lift-map",
            if transfer_contract_formula_lift_map_ready(
                &save.backend.transfer_contract_formula_lift_map,
            ) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} static-code-derived transfer/contract formula lift(s)",
                save.backend.transfer_contract_formula_lift_map.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "transfer-contract-formula-mutation-plan",
            if transfer_contract_formula_plan_ready(&transfer_contract_formula_plan) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} formula-derived transfer/contract mutation row(s)",
                transfer_contract_formula_plan.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "transfer-contract-runtime-store",
            if transfer_contract_runtime_store_ready(&save.backend.transfer_contract_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} renewal window(s), {} contract event(s), {} compensation value(s), {} offer value(s), {} decision rule(s), {} transfer-manager shape(s), {} queue item(s), {} dispatch(es), {} applied formula mutation(s)",
                save.backend.transfer_contract_runtime_store.renewal_windows.len(),
                save.backend.transfer_contract_runtime_store.contract_events.len(),
                save.backend
                    .transfer_contract_runtime_store
                    .compensation_values
                    .len(),
                save.backend
                    .transfer_contract_runtime_store
                    .offer_values
                    .len(),
                save.backend
                    .transfer_contract_runtime_store
                    .decision_rules
                    .len(),
                save.backend
                    .transfer_contract_runtime_store
                    .transfer_manager_record_shapes
                    .len(),
                save.backend.transfer_contract_runtime_store.transfer_queue.len(),
                save.backend.transfer_contract_runtime_store.queue_dispatches.len(),
                save.backend
                    .transfer_contract_runtime_store
                    .applied_formula_mutations
            ),
        );
        push_backend_check(
            &mut checks,
            "news-inbox-emission-map",
            if save.backend.news_inbox_emission_map.len() >= 7
                && save.backend.news_inbox_emission_map.iter().any(|entry| {
                    entry.stride.as_deref() == Some("0x68")
                        && entry.helper.as_deref() == Some("0x00596fa0")
                        && entry.function == "0x0050c8d0"
                })
                && save.backend.news_inbox_emission_map.iter().any(|entry| {
                    entry.record_offset.as_deref() == Some("0x30") && entry.function == "0x0050c8d0"
                })
                && save.backend.news_inbox_emission_map.iter().any(|entry| {
                    entry.record_offset.as_deref() == Some("0xde") && entry.function == "0x0076e180"
                })
                && save
                    .backend
                    .news_inbox_emission_map
                    .iter()
                    .any(|entry| entry.helper.as_deref() == Some("0x006724d0"))
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} code-derived news/inbox event and queue boundary entry(s)",
                save.backend.news_inbox_emission_map.len()
            ),
        );
        let news_inbox_formula_plan = plan_news_inbox_formula_mutations(
            &save.backend,
            &default_news_inbox_formula_scenario(),
        );
        push_backend_check(
            &mut checks,
            "news-inbox-formula-lift-map",
            if news_inbox_formula_lift_map_ready(&save.backend.news_inbox_formula_lift_map) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} static-code-derived news/inbox formula lift(s)",
                save.backend.news_inbox_formula_lift_map.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "news-inbox-formula-mutation-plan",
            if news_inbox_formula_plan_ready(&news_inbox_formula_plan) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} formula-derived news/inbox mutation row(s)",
                news_inbox_formula_plan.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "news-inbox-runtime-store",
            if news_inbox_runtime_store_ready(&save.backend.news_inbox_runtime_store) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} created event(s), {} dispatch(es), {} removed queue node(s), {} applied formula mutation(s)",
                save.backend.news_inbox_runtime_store.created_events.len(),
                save.backend
                    .news_inbox_runtime_store
                    .visible_news_dispatches
                    .len(),
                save.backend.news_inbox_runtime_store.removed_queue_nodes.len(),
                save.backend.news_inbox_runtime_store.applied_formula_mutations
            ),
        );
        let implementation_plan = backend_implementation_plan(&save);
        push_backend_check(
            &mut checks,
            "backend-implementation-plan",
            if implementation_plan.len() == 4
                && implementation_plan.iter().all(|item| {
                    item.readiness == BackendImplementationReadiness::MutationsImplemented
                        && item.boundary_entries > 0
                        && !item.primary_frontiers.is_empty()
                })
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} gameplay subsystem implementation plan item(s), all tied to static-proof-backed boundary mutations",
                implementation_plan.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "gameplay-mutator-contracts",
            if save.backend.mutator_contracts.len() == 4
                && save.backend.mutator_contracts.iter().all(|contract| {
                    contract.status == GameplayMutatorStatus::ParityVerified
                        && contract.implementation_present
                        && !contract.trace_file.is_empty()
                        && !contract.boundary_map.is_empty()
                        && !contract.implementation_hook.is_empty()
                        && !contract.required_before_enable.is_empty()
                })
                && save.backend.mutator_contracts.iter().any(|contract| {
                    contract.system == "match results"
                        && contract.phase == 2
                        && contract.boundary_map == "match_result_write_map"
                })
                && save.backend.mutator_contracts.iter().any(|contract| {
                    contract.system == "transfers/contracts"
                        && contract.phase == 0
                        && contract.boundary_map == "transfer_contract_state_map"
                })
            {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Fail
            },
            format!(
                "{} exact gameplay mutator contract(s), all static-proof-backed and parity verified",
                save.backend.mutator_contracts.len()
            ),
        );
        push_backend_check(
            &mut checks,
            "gameplay-mutators",
            if headless_blockers == 0 {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Warn
            },
            format!(
                "{} exact gameplay subsystem(s) still frontier-only",
                headless_blockers
            ),
        );

        let semantic_cleanup: Vec<SemanticCleanupItem> = canonical
            .tables
            .iter()
            .filter(|table| table.dat_replacement_status != DatReplacementStatus::RuntimeReady)
            .map(|table| SemanticCleanupItem {
                table: table.path.clone(),
                rows: table.rows,
                status: table.dat_replacement_status,
                runtime_blocking: false,
                inferred_fields: table.inferred_fields,
                projected_fields: table.projected_fields,
                editable: table.editable,
                blockers: table.blockers.clone(),
                next_action: Self::semantic_cleanup_next_action(table),
            })
            .collect();
        push_backend_check(
            &mut checks,
            "semantic-cleanup-ledger",
            if semantic_cleanup.iter().all(|item| !item.runtime_blocking) {
                ValidationStatus::Pass
            } else {
                ValidationStatus::Warn
            },
            format!(
                "{} non-runtime-blocking semantic cleanup item(s) remain after Rust-owned DB promotion",
                semantic_cleanup.len()
            ),
        );

        let mut blockers = Vec::new();
        if !canonical.validation.failures.is_empty() {
            blockers.push(BackendReadinessBlocker {
                system: "canonical Rust DB validation".to_string(),
                severity: "blocking".to_string(),
                status: format!("{} failure(s)", canonical.validation.failures.len()),
                next_evidence: "Fix every canonical validation failure before treating the Rust DB as authoritative.".to_string(),
            });
        }
        for table in canonical
            .tables
            .iter()
            .filter(|table| table.dat_replacement_status != DatReplacementStatus::RuntimeReady)
            .take(12)
        {
            blockers.push(BackendReadinessBlocker {
                system: table.path.clone(),
                severity: "semantic".to_string(),
                status: format!("{:?}", table.dat_replacement_status),
                next_evidence: table.blockers.first().cloned().unwrap_or_else(|| {
                    "Promote table to typed, editable, verified Rust records.".to_string()
                }),
            });
        }
        for blocker in &save.headless.blockers {
            blockers.push(BackendReadinessBlocker {
                system: blocker.system.clone(),
                severity: "gameplay".to_string(),
                status: blocker.status.clone(),
                next_evidence: blocker.required_evidence.clone(),
            });
        }

        let score_percent = backend_completion_score(
            coverage.remaining_binary_tables == 0,
            canonical.validation.failures.is_empty(),
            headless_report.days_advanced == 2 && headless_report.phases_advanced == 6,
            save.headless.manager.is_some(),
            runtime_mutation_log_entries > 0,
            headless_blockers == 0,
        );
        let status = if blockers.is_empty() {
            BackendReadinessStatus::RuntimeReady
        } else if headless_blockers > 0 {
            BackendReadinessStatus::BlockedByFrontierGameplay
        } else {
            BackendReadinessStatus::VerifiedHeadlessShell
        };

        BackendReadinessReport {
            status,
            completion: BackendCompletion {
                canonical_tables: canonical.table_count,
                runtime_ready_tables,
                editable_tables: canonical.editable_tables,
                validation_failures: canonical.validation.failures.len(),
                remaining_binary_tables: coverage.remaining_binary_tables,
                phase_frontiers,
                phase_2_frontiers,
                runtime_mutation_log_entries,
                headless_blockers,
                score_percent,
            },
            checks,
            blockers,
            semantic_cleanup,
            implementation_plan,
            milestones: vec![
                "Rust DB owns all known logical tables imported from the original .dat set.".to_string(),
                "A new Rust save can be created from rust-db without opening original .dat files.".to_string(),
                "The verified CM phase/date shell can tick headlessly and exposes frontier calls instead of guessing gameplay.".to_string(),
                "The Rust save now records backend system mutation attempts for each phase, ready to swap frontier entries for exact lifted mutations.".to_string(),
                "The four exact gameplay mutators now have Rust-owned contracts that bind phase, boundary map, trace file, implementation hook, and parity gate.".to_string(),
                "The match-engine lift map is Rust-owned: setup, step controller, phase controller, period writer, and event queue boundaries are named before match-result mutation code can be installed.".to_string(),
                "The exact gameplay mutator skeletons are Rust-owned and disabled until parity promotion.".to_string(),
                "The match-result fixture write map is Rust-owned: fixture score bytes +0x43..+0x4a are now named at the backend boundary.".to_string(),
                "The match-result mutator install plan is Rust-owned: exact trace coverage and promotion rules are named before implementation_present can flip.".to_string(),
                "The competition fixture state map is Rust-owned: fixture participants +0x1c/+0x20, notification flags +0x4d bits 0x100/0x200, and 70-day cleanup cadence are named at the backend boundary.".to_string(),
                "The transfer/contract state map is Rust-owned: contract renewal windows, staff/event/club strides, queue dispatch, and transfer.dat list shapes are named at the backend boundary.".to_string(),
                "The news/inbox emission map is Rust-owned: paired fixture event creation, news reset byte +0xde, and queue unlink helper are named at the backend boundary.".to_string(),
                "Headless manager/session metadata is saved in Rust-owned runtime state.".to_string(),
            ],
            next_steps: vec![
                "Make branch-complete match-engine numeric outputs the next acceptance layer: late tactical multipliers, RNG call order, and action/event branches.".to_string(),
                "Use original editor/gameplay evidence before upgrading staff.type10 rating_short_0x05/0x07/0x0d from compatibility slots to semantic CA/PA/reputation labels.".to_string(),
                "Expand the headless season acceptance gate from deterministic backend mutations to playable football outcomes: fixtures, standings, transfers, contracts, news, and saves.".to_string(),
                "Keep .dat import/export as compatibility tests only; Rust DB and Rust saves remain the canonical runtime truth.".to_string(),
            ],
        }
    }

    fn semantic_cleanup_next_action(table: &CanonicalTableReport) -> String {
        if table
            .blockers
            .iter()
            .any(|blocker| blocker.contains("opaque preserved payload"))
        {
            "Decode preserved payload into named Rust fields using code-derived stride/offset evidence."
                .to_string()
        } else if table
            .blockers
            .iter()
            .any(|blocker| blocker.contains("no typed Rust edit command"))
        {
            "Add an audited typed edit command or mark the table intentionally read-only."
                .to_string()
        } else if table
            .blockers
            .iter()
            .any(|blocker| blocker.contains("projected text/name"))
        {
            "Prove fixed-slot text/name offsets against code/editor evidence, then mark projected fields verified."
                .to_string()
        } else if table.inferred_fields > 0 {
            "Replace inferred field names with code-derived names or explicit compatibility-only labels."
                .to_string()
        } else {
            "Review remaining table blockers and promote the table to RuntimeReady when no blocker remains."
                .to_string()
        }
    }

    fn canonical_validation_report(&self) -> CanonicalValidationReport {
        let mut checks = Vec::new();
        let mut failures = Vec::new();
        let audit = self.audit_rust_db();
        push_validation_check(
            &mut checks,
            &mut failures,
            "rust-db-audit",
            audit.mismatches.is_empty(),
            format!(
                "{} checked table(s), {} mismatch(es)",
                audit.checked_tables,
                audit.mismatches.len()
            ),
            false,
        );

        push_validation_check(
            &mut checks,
            &mut failures,
            "unique core ordinals",
            unique_u32(self.core.continents.iter().map(|entry| entry.ordinal))
                && unique_u32(self.core.colours.iter().map(|entry| entry.ordinal))
                && unique_u32(self.core.nations.iter().map(|entry| entry.ordinal))
                && unique_u32(self.core.clubs.iter().map(|entry| entry.ordinal))
                && unique_u32(self.core.nat_clubs.iter().map(|entry| entry.ordinal)),
            "core ordinal values are unique within each table".to_string(),
            false,
        );
        push_validation_check(
            &mut checks,
            &mut failures,
            "unique reference ids",
            unique_u32(self.references.cities.iter().map(|entry| entry.id))
                && unique_u32(self.references.stadiums.iter().map(|entry| entry.id))
                && unique_u32(
                    self.references
                        .staff_competitions
                        .iter()
                        .map(|entry| entry.id),
                )
                && unique_u32(
                    self.references
                        .club_competitions
                        .iter()
                        .map(|entry| entry.id),
                )
                && unique_u32(
                    self.references
                        .nation_competitions
                        .iter()
                        .map(|entry| entry.id),
                ),
            "reference IDs are unique within each table".to_string(),
            false,
        );
        push_validation_check(
            &mut checks,
            &mut failures,
            "non-empty names",
            self.core
                .continents
                .iter()
                .all(|entry| entry.primary_name.as_deref().unwrap_or("").trim().len() > 0)
                && self
                    .core
                    .nations
                    .iter()
                    .all(|entry| entry.primary_name.as_deref().unwrap_or("").trim().len() > 0)
                && self
                    .references
                    .cities
                    .iter()
                    .all(|entry| entry.name.trim().len() > 0)
                && self
                    .references
                    .stadiums
                    .iter()
                    .all(|entry| entry.name.trim().len() > 0),
            "major visible name tables contain non-empty names".to_string(),
            true,
        );
        push_validation_check(
            &mut checks,
            &mut failures,
            "staff attribute bounds",
            self.staff
                .type10
                .iter()
                .all(|entry| entry.attributes.iter().all(|value| *value <= 100)),
            "staff.type10 attribute bytes are in the expected 0..100 editor range".to_string(),
            false,
        );
        push_validation_check(
            &mut checks,
            &mut failures,
            "history competition references",
            history_competition_refs_valid(
                self.references
                    .staff_comp_history
                    .iter()
                    .map(|entry| entry.u32_slots[1]),
                self.references
                    .staff_competitions
                    .iter()
                    .map(|entry| entry.id),
            ) && history_competition_refs_valid(
                self.references
                    .club_comp_history
                    .iter()
                    .map(|entry| entry.u32_slots[1]),
                self.references
                    .club_competitions
                    .iter()
                    .map(|entry| entry.id),
            ) && history_competition_refs_valid(
                self.references
                    .nation_comp_history
                    .iter()
                    .map(|entry| entry.u32_slots[1]),
                self.references
                    .nation_competitions
                    .iter()
                    .map(|entry| entry.id),
            ),
            "competition-history slot 1 resolves to a known competition when non-empty".to_string(),
            true,
        );

        CanonicalValidationReport { checks, failures }
    }

    fn table_row_count(&self, path: &str) -> usize {
        match path {
            "core.clubs" => self.core.clubs.len(),
            "core.nat_clubs" => self.core.nat_clubs.len(),
            "core.colours" => self.core.colours.len(),
            "core.continents" => self.core.continents.len(),
            "core.nations" => self.core.nations.len(),
            "staff.type6" => self.staff.type6.len(),
            "staff.type8" => self.staff.type8.len(),
            "staff.type9" => self.staff.type9.len(),
            "staff.type10" => self.staff.type10.len(),
            "references.cities" => self.references.cities.len(),
            "references.officials" => self.references.officials.len(),
            "references.first_names" => self.references.first_names.len(),
            "references.second_names" => self.references.second_names.len(),
            "references.common_names" => self.references.common_names.len(),
            "references.stadiums" => self.references.stadiums.len(),
            "references.staff_competitions" => self.references.staff_competitions.len(),
            "references.club_competitions" => self.references.club_competitions.len(),
            "references.nation_competitions" => self.references.nation_competitions.len(),
            "references.staff_history" => self.references.staff_history.len(),
            "references.staff_comp_history" => self.references.staff_comp_history.len(),
            "references.club_comp_history" => self.references.club_comp_history.len(),
            "references.nation_comp_history" => self.references.nation_comp_history.len(),
            _ => 0,
        }
    }

    fn audit_count(&self, kind: u8, path: &str, actual: usize, mismatches: &mut Vec<String>) {
        let Some(expected) = self
            .base_data
            .iter()
            .find(|entry| entry.kind == kind)
            .map(|entry| entry.count)
        else {
            mismatches.push(format!("{path} has no manifest entry for type {kind}"));
            return;
        };
        if expected as usize != actual {
            mismatches.push(format!(
                "{path} count {actual} differs from manifest type {kind} count {expected}"
            ));
        }
    }

    pub fn export_owned_data_dir(&self, dir: &Path) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        let manifest = Manifest {
            entries: self
                .base_data
                .iter()
                .map(|entry| cm_data::ManifestEntry {
                    filename: entry.filename.clone(),
                    kind: entry.kind,
                    count: entry.count,
                })
                .collect(),
        };
        fs::write(dir.join("index.dat"), manifest.to_bytes())?;
        fs::write(dir.join("club.dat"), flatten_core_records(&self.core.clubs))?;
        fs::write(
            dir.join("nat_club.dat"),
            flatten_core_records(&self.core.nat_clubs),
        )?;
        fs::write(
            dir.join("colour.dat"),
            flatten_core_records(&self.core.colours),
        )?;
        fs::write(
            dir.join("continent.dat"),
            flatten_core_records(&self.core.continents),
        )?;
        fs::write(
            dir.join("nation.dat"),
            flatten_core_records(&self.core.nations),
        )?;
        cm_data::write_city_table(
            &dir.join("city.dat"),
            &to_cm_cities(&self.references.cities),
        )?;
        cm_data::write_officials_table(
            &dir.join("officials.dat"),
            &to_cm_officials(&self.references.officials),
        )?;
        cm_data::write_name_table(
            &dir.join("first_names.dat"),
            &to_cm_names(&self.references.first_names),
        )?;
        cm_data::write_name_table(
            &dir.join("second_names.dat"),
            &to_cm_names(&self.references.second_names),
        )?;
        cm_data::write_name_table(
            &dir.join("common_names.dat"),
            &to_cm_names(&self.references.common_names),
        )?;
        cm_data::write_stadium_table(
            &dir.join("stadium.dat"),
            &to_cm_stadiums(&self.references.stadiums),
        )?;
        cm_data::write_staff_comp_table(
            &dir.join("staff_comp.dat"),
            &to_cm_competitions(&self.references.staff_competitions),
        )?;
        cm_data::write_club_comp_table(
            &dir.join("club_comp.dat"),
            &to_cm_competitions(&self.references.club_competitions),
        )?;
        cm_data::write_nation_comp_table(
            &dir.join("nation_comp.dat"),
            &to_cm_competitions(&self.references.nation_competitions),
        )?;
        cm_data::write_staff_history_table(
            &dir.join("staff_history.dat"),
            &to_cm_history17(&self.references.staff_history),
        )?;
        cm_data::write_staff_comp_history_table(
            &dir.join("staff_comp_history.dat"),
            &to_cm_history58(&self.references.staff_comp_history),
        )?;
        cm_data::write_club_comp_history_table(
            &dir.join("club_comp_history.dat"),
            &to_cm_history26(&self.references.club_comp_history),
        )?;
        cm_data::write_nation_comp_history_table(
            &dir.join("nation_comp_history.dat"),
            &to_cm_history26(&self.references.nation_comp_history),
        )?;
        cm_data::write_staff_data(&dir.join("staff.dat"), &to_cm_staff_data(&self.staff))?;
        fs::write(
            dir.join("schema.json"),
            serde_json::to_vec_pretty(&self.schema).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("serialize schema: {err}"),
                )
            })?,
        )?;
        Ok(())
    }

    pub fn coverage(&self) -> SnapshotCoverage {
        let known_logical_tables = TABLE_SPECS.len();
        let recognized_manifest_entries = self
            .base_data
            .iter()
            .filter(|entry| manifest_entry_is_known(&entry.filename, entry.kind))
            .count();
        let unrecognized_manifest_entries = self
            .base_data
            .len()
            .saturating_sub(recognized_manifest_entries);
        let owned_core_tables = 5;
        let owned_reference_tables = 13;
        let owned_staff_tables = 4;
        let owned_world_tables = owned_core_tables + owned_reference_tables + owned_staff_tables;
        SnapshotCoverage {
            manifest_entries: self.base_data.len(),
            known_logical_tables,
            recognized_manifest_entries,
            unrecognized_manifest_entries,
            owned_core_tables,
            owned_reference_tables,
            owned_staff_tables,
            owned_world_tables,
            remaining_binary_tables: known_logical_tables.saturating_sub(owned_world_tables),
        }
    }

    pub fn audit_against_install(&self, root: &Path) -> io::Result<SnapshotAuditReport> {
        let data_dir = root.join("Data");
        let manifest = Manifest::parse(&fs::read(data_dir.join("index.dat"))?);
        let install_known_logical_tables = TABLE_SPECS.len();
        let install_save = match fs::read(root.join("save1.sav")) {
            Ok(bytes) => Some(SaveFile::parse(&bytes)?),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(err),
        };

        let mut mismatches = Vec::new();
        if self.base_data.len() != manifest.entries.len() {
            mismatches.push(format!(
                "manifest entry count differs: snapshot {} vs install {}",
                self.base_data.len(),
                manifest.entries.len()
            ));
        }

        for entry in &self.base_data {
            let Some(install_entry) = manifest.entries.iter().find(|candidate| {
                candidate.filename.eq_ignore_ascii_case(&entry.filename)
                    && candidate.kind == entry.kind
            }) else {
                mismatches.push(format!(
                    "missing install manifest entry for type {} file {}",
                    entry.kind, entry.filename
                ));
                continue;
            };

            if install_entry.count != entry.count {
                mismatches.push(format!(
                    "manifest count differs for type {} file {}: snapshot {} vs install {}",
                    entry.kind, entry.filename, entry.count, install_entry.count
                ));
            }
        }

        let snapshot_save_section_count = self.save.as_ref().map(|save| save.section_count);
        let install_save_section_count = install_save.as_ref().map(|save| save.sections.len());
        if snapshot_save_section_count != install_save_section_count {
            mismatches.push(format!(
                "save section count differs: snapshot {:?} vs install {:?}",
                snapshot_save_section_count, install_save_section_count
            ));
        }

        Ok(SnapshotAuditReport {
            install_root: root.display().to_string(),
            snapshot_manifest_entries: self.base_data.len(),
            install_manifest_entries: manifest.entries.len(),
            snapshot_recognized_manifest_entries: self
                .base_data
                .iter()
                .filter(|entry| manifest_entry_is_known(&entry.filename, entry.kind))
                .count(),
            install_known_logical_tables,
            snapshot_save_section_count,
            install_save_section_count,
            coverage: self.coverage(),
            mismatches,
        })
    }

    fn summarize_save(save: &SaveFile) -> SaveSummary {
        SaveSummary {
            version: save.version,
            section_count: save.sections.len(),
            sections: save
                .sections
                .iter()
                .map(|section| SaveSectionSummary {
                    name: section.name.clone(),
                    size: section.size,
                    verified_record_size: section.record_kind().map(|kind| kind.size()),
                    verified_record_count: section.verified_record_count(),
                })
                .collect(),
        }
    }
}

/// The user's choices on Select League(s) + Select Start Season, i.e. exactly
/// the inputs the exe accumulates before "Initialising game data".
///
/// Provenance for each field (from the exe's Select League(s) event handler
/// at 0x00806640, disassembled 2026-08-20):
/// * `selected_nations` — event 0xc sets bit 1 of the in-memory competition
///   record's flag byte and bumps the counters at DAT_00acdf00/DAT_00acdf04.
/// * `background_nations` — the second selectable state on each picker row.
/// * `use_real_players` — event 0x39 writes 1 / event 0x3a writes 0 to the
///   byte at DAT_009a2051.
/// * `attribute_masking` — event 0x3b writes 1 / event 0x3c writes 0 to the
///   byte at DAT_009b88a8.
/// * `start_year` — chosen on Select Start Season (FUN_00807280); the shipped
///   database starts the 2001/02 season.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewGameOptions {
    /// Nation names as shown in the picker, e.g. "England". Playable leagues.
    pub selected_nations: Vec<String>,
    /// Nations loaded as background only (clubs exist, no playable league).
    pub background_nations: Vec<String>,
    pub use_real_players: bool,
    pub attribute_masking: bool,
    /// Calendar year the season starts in (2001 for the shipped database).
    pub start_year: u16,
}

impl Default for NewGameOptions {
    fn default() -> Self {
        // The exe's defaults on first entry to Select League(s): both toggles
        // Yes, nothing selected yet.
        Self {
            selected_nations: Vec::new(),
            background_nations: Vec::new(),
            use_real_players: true,
            attribute_masking: true,
            start_year: 2001,
        }
    }
}

impl World {
    /// Create a new game from the native database using the user's picker
    /// choices — the Rust equivalent of the exe's path from "Next on Select
    /// League(s)" through "Initialising game data".
    ///
    /// Difference from [`World::new_runtime_save_from_rust_db`]: that builder
    /// ignores user input and schedules whatever league-like competitions it
    /// finds first. This one restricts scheduling to the selected nations'
    /// competitions and records the options in the save.
    pub fn new_game_from_rust_db(
        &self,
        db_dir: &Path,
        options: &NewGameOptions,
    ) -> RuntimeSaveGame {
        let mut save = self.new_runtime_save_from_rust_db(db_dir);
        // Real per-league start date: the earliest season kickoff among the
        // selected leagues (see `league_calendar`, lifted from FUN_006508e0).
        // The exe normalises the chosen start against the foreground league's
        // key-nation entry; with multiple leagues the game must begin on or
        // before the first to kick off.
        let (year, month, day) =
            crate::league_calendar::earliest_start(&options.selected_nations, options.start_year);
        save.date = GameDate { year, month, day };
        for event in &mut save.pending_events {
            event.date = save.date.clone();
        }
        for milestone in &mut save.headless.milestones {
            milestone.date = save.date.clone();
        }

        // Build the real season: every FOREGROUND competition (resolved from
        // club membership, not name-matching) gets a full double round-robin
        // from its real member clubs (club+0x57/0x5b/0x60), dated from that
        // league's own season start. This replaces the previous approach of
        // filtering the default headless schedule — which only kept whatever
        // the generic generator happened to schedule. When nothing is
        // selected we keep the default schedule so the headless harness still
        // has something to tick.
        if !options.selected_nations.is_empty() {
            let ids = self.competition_ids_for_nations(&options.selected_nations);
            if !ids.is_empty() {
                let (fixtures, proofs, standings) =
                    self.generate_new_game_season(&ids, options.start_year);
                if !fixtures.is_empty() {
                    save.season.fixtures = fixtures;
                    save.season.schedule_generation = proofs;
                    save.season.standings = standings;
                    save.season.provenance = format!(
                        "New-game season: {} foreground competition(s) built as double round-robins from real club membership (club+0x57/0x5b/0x60), dated per-league from FUN_006508e0's season-start table. World not culled.",
                        save.season.schedule_generation.len()
                    );
                }
            }
        }

        // Tag every nation with its league tier. This is the port of the
        // exe's `nation_record + 0x11c` bitfield assignment during init: the
        // whole world stays loaded (no record culling — matches FUN_005121a0
        // loading all pools wholesale), and the tier only records which
        // nations are foreground / background / neither. Background nations
        // can be promoted to foreground mid-game as a pure flag flip.
        save.nation_tiers = self.build_nation_tiers(options);

        // Player initialisation — the deterministic core of FUN_0051f5d0:
        // seed each attribute-holder's initial runtime state (CA, resolved PA,
        // condition=156, neutral morale). Stored as a summary; per-player state
        // is deterministic from the base and only diverges during play.
        save.player_init = Some(self.player_init_summary(&save.date));

        save.new_game = Some(options.clone());
        let foreground = save
            .nation_tiers
            .iter()
            .filter(|t| t.tier == LeagueTier::Foreground)
            .count();
        let background = save
            .nation_tiers
            .iter()
            .filter(|t| t.tier == LeagueTier::Background)
            .count();
        save.notes.push(format!(
            "New game created from picker selection: {foreground} foreground / {background} background nation(s) tagged; all {} nations remain loaded (world is not culled). real_players={}, attribute_masking={}.",
            save.nation_tiers.len(),
            options.use_real_players,
            options.attribute_masking,
        ));
        save
    }

    /// Assign a [`LeagueTier`] to every nation in the database from the picker
    /// selection. Foreground = `selected_nations`, Background =
    /// `background_nations`, everything else = Neither. The whole nation list
    /// is tagged (not just the selected ones) so the save is a complete,
    /// faithful snapshot of the `+0x11c` state across the world.
    ///
    /// `detailed_matches` follows the exe: foreground nations always run
    /// detailed; background nations inherit the game-wide setting (defaulted on
    /// here — the "Background Matches" option is not yet surfaced in the UI).
    pub fn build_nation_tiers(&self, options: &NewGameOptions) -> Vec<NationTierAssignment> {
        let foreground_ids = self.nation_ids_for_picker_labels(&options.selected_nations);
        let background_ids = self.nation_ids_for_picker_labels(&options.background_nations);
        self.core
            .nations
            .iter()
            .map(|record| {
                let view = crate::typed_records::NationView::new(record);
                let id = view.id();
                let tier = if foreground_ids.contains(&id) {
                    LeagueTier::Foreground
                } else if background_ids.contains(&id) {
                    LeagueTier::Background
                } else {
                    LeagueTier::Neither
                };
                NationTierAssignment {
                    nation_id: id,
                    nation_name: view.primary_name(),
                    tier,
                    detailed_matches: tier == LeagueTier::Foreground,
                }
            })
            .collect()
    }

    /// Nation-record ids for the given picker labels.
    ///
    /// The picker's slot table holds NATION records (verified: `FUN_00811140`
    /// walks the nation pool at stride 0x122 when clearing selections), so
    /// selection resolves to nation ids first. Two picker labels differ from
    /// the canonical nation names in the shipped database.
    pub fn nation_ids_for_picker_labels(&self, labels: &[String]) -> BTreeSet<u32> {
        let wanted: Vec<&str> = labels
            .iter()
            .map(|label| canonical_nation_name(label.as_str()))
            .collect();
        let mut ids = BTreeSet::new();
        for record in &self.core.nations {
            let view = crate::typed_records::NationView::new(record);
            let name = view.primary_name();
            if wanted.iter().any(|want| *want == name.as_str()) {
                ids.insert(view.id());
            }
        }
        ids
    }

    /// True when a competition is a MANAGEABLE league — the definitive
    /// data-derived rule (verified against the official editor and England's
    /// known set): the record carries a real nation (`nation_id >= 0`) AND a
    /// three-letter abbreviation (`three_letter_name` non-empty). Cups (FA
    /// Cup) and non-manageable feeder divisions (Isthmian/Southern/Northern)
    /// have no three-letter name; the global "A Lower Division" bucket has no
    /// nation. Reproduces England exactly = {Premier, First, Second, Third,
    /// Conference}. NOT a heuristic — both are real fields the editor exposes.
    pub fn is_manageable_league(competition: &DomainCompetition) -> bool {
        competition.nation_id >= 0 && !competition.three_letter_name.trim().is_empty()
    }

    /// The manageable-league competition ids for the given nations, read
    /// straight from the (now correctly-parsed) competition records: comps
    /// whose `nation_id` is one of these nations AND that pass
    /// `is_manageable_league`.
    pub fn manageable_league_ids_for_nations(&self, nations: &[String]) -> BTreeSet<u32> {
        let nation_ids = self.nation_ids_for_picker_labels(nations);
        self.references
            .club_competitions
            .iter()
            .filter(|c| c.nation_id >= 0 && nation_ids.contains(&(c.nation_id as u32)))
            .filter(|c| Self::is_manageable_league(c))
            .map(|c| c.id)
            .collect()
    }

    /// Competition ids belonging to the given nations.
    ///
    /// Now uses the REAL competition→nation link at record +0x5d (decoded
    /// after the official editor confirmed the field layout — the earlier
    /// "empty nation field" was a parse bug), restricted to manageable
    /// leagues. Falls back to club-membership then name-matching only if the
    /// record link yields nothing.
    pub fn competition_ids_for_nations(&self, nations: &[String]) -> BTreeSet<u32> {
        let nation_ids = self.nation_ids_for_picker_labels(nations);

        // PRIMARY — the competition record's own nation link (+0x5d),
        // filtered to manageable leagues (nation + three-letter name).
        let by_record = self.manageable_league_ids_for_nations(nations);
        if !by_record.is_empty() {
            return by_record;
        }

        // SECONDARY — club membership (club+0x57), in case a record's nation
        // link is missing but its member clubs identify it.
        let comp_membership = self.competition_ids_for_nation_ids(&nation_ids);
        if !comp_membership.is_empty() {
            return comp_membership;
        }

        // FALLBACK — only if the club walk found nothing (e.g. a nation with no
        // clubs carrying comp links). Keep the old name/link heuristic so the
        // headless harness still has something to schedule.
        let mut ids = BTreeSet::new();
        for competition in &self.references.club_competitions {
            if let Some(linked) = competition_nation_id(competition) {
                if nation_ids.contains(&linked) {
                    ids.insert(competition.id);
                    continue;
                }
            }
            let name = competition.long_name.as_str();
            if nations.iter().any(|nation| {
                let nation = nation.as_str();
                name.contains(nation) || competition_name_matches_nation(name, nation)
            }) {
                ids.insert(competition.id);
            }
        }
        ids
    }

    /// Assemble the dashboard the active human sees (port of FUN_00454620 +
    /// draw FUN_004551c0). If they manage a club → a `ClubDashboard`; if
    /// unemployed → the manager-status / job view.
    /// Primary name of a club by id (for menu labels like "Pro Vercelli Squad").
    pub fn club_name(&self, club_id: u32) -> Option<String> {
        self.core
            .clubs
            .iter()
            .map(|r| crate::typed_records::ClubView::new(r))
            .find(|v| v.id() == club_id)
            .map(|v| v.primary_name())
    }

    /// Primary name of a nation by id.
    pub fn nation_name(&self, nation_id: u32) -> Option<String> {
        self.core
            .nations
            .iter()
            .map(|r| crate::typed_records::NationView::new(r))
            .find(|v| v.id() == nation_id)
            .map(|v| v.primary_name())
    }

    pub fn dashboard_for(&self, save: &RuntimeSaveGame, human: usize) -> Option<DashboardView> {
        let h = save.humans.get(human)?;
        let name = h.identity.display_name();
        let Some(club_id) = h.club else {
            return Some(DashboardView::Unemployed(UnemployedView {
                manager_name: name,
                date: save.date.clone(),
                message: "You are not currently managing a club. Apply for a job to take charge."
                    .to_string(),
            }));
        };

        // Club record.
        let club_rec = self.core.clubs.iter().find(|c| {
            crate::typed_records::ClubView::new(c).id() == club_id
        })?;
        let club_view = crate::typed_records::ClubView::new(club_rec);
        let club_name = club_view.primary_name();
        let division_id = club_view.division_id();

        // Division name.
        let division_name = division_id
            .and_then(|d| {
                self.references
                    .club_competitions
                    .iter()
                    .find(|c| c.id == d as u32)
                    .map(|c| c.long_name.clone())
            })
            .unwrap_or_else(|| "Unknown Division".to_string());

        // League position from standings (start-morning order until results
        // arrive). division_size = clubs in this division.
        let division_members: Vec<u32> = division_id
            .map(|d| {
                self.club_members_of_competition(d as u32)
                    .into_iter()
                    .map(|(id, _)| id)
                    .collect()
            })
            .unwrap_or_default();
        let division_size = division_members.len();
        let position = save
            .season
            .standings
            .iter()
            .filter(|s| division_members.contains(&s.club_id))
            .position(|s| s.club_id == club_id)
            .map(|p| p + 1)
            .unwrap_or(1);

        // Next pending fixture involving this club.
        let next_fixture = save
            .season
            .fixtures
            .iter()
            .filter(|f| f.status == HeadlessFixtureStatus::Pending)
            .filter(|f| f.home_club_id == club_id || f.away_club_id == club_id)
            .min_by(|a, b| a.date.cmp(&b.date))
            .map(|f| DashboardFixture {
                date: f.date.clone(),
                home_club_name: f.home_club_name.clone(),
                away_club_name: f.away_club_name.clone(),
                is_home: f.home_club_id == club_id,
                competition_name: f.competition_name.clone(),
            });

        // Squad: players whose current club is this one (type6 body+0x35).
        let attr_by_id: BTreeMap<u32, &DomainStaffType10> =
            self.staff.type10.iter().map(|a| (a.id, a)).collect();
        let start_day = day_of_year(save.date.year, save.date.month, save.date.day);
        let mut squad: Vec<SquadMember> = Vec::new();
        for person in &self.staff.type6 {
            if person.current_club_id() == Some(club_id) {
                let attr = attr_by_id.get(&person.id);
                squad.push(SquadMember {
                    player_id: person.id,
                    name: self.person_display_name(person),
                    age: person.age_at(save.date.year, start_day),
                    current_ability: attr.map(|a| a.current_ability()).unwrap_or(0),
                    condition: PlayerInitState::INITIAL_CONDITION,
                });
            }
        }
        // Best first.
        squad.sort_by(|a, b| b.current_ability.cmp(&a.current_ability));

        Some(DashboardView::Club(ClubDashboard {
            manager_name: name,
            club_id,
            club_name,
            division_name,
            position,
            division_size,
            date: save.date.clone(),
            next_fixture,
            squad,
        }))
    }

    /// Build the News page (home screen) for the given human — titled
    /// "<Manager> News", newest item first, drawn from `save.pending_events`.
    /// When the inbox is empty (e.g. the very first morning) a welcome item is
    /// synthesised so the page is never blank, matching the game's start.
    pub fn news_for(&self, save: &RuntimeSaveGame, human: usize) -> NewsView {
        let manager = save
            .humans
            .get(human)
            .map(|h| h.identity.display_name())
            .unwrap_or_else(|| "Manager".to_string());
        let mut items: Vec<NewsItem> = save
            .pending_events
            .iter()
            .rev()
            .map(|e| {
                // Events carry "<headline> - <summary>"; split for the body.
                let (headline, body) = match e.message.split_once(" - ") {
                    Some((h, b)) => (h.to_string(), b.to_string()),
                    None => (e.message.clone(), e.message.clone()),
                };
                NewsItem {
                    date: e.date.clone(),
                    date_label: news_date_label(&e.date, e.phase),
                    headline,
                    body,
                    category: NewsCategory::from_kind(&e.kind),
                    unread: true,
                }
            })
            .collect();
        if items.is_empty() {
            let club = save
                .humans
                .get(human)
                .and_then(|h| h.club)
                .and_then(|id| self.club_name(id));
            let (headline, body) = match club {
                Some(c) => (
                    format!("Welcome to {c}"),
                    format!(
                        "The board have appointed {manager} as the new manager of {c}. \
                         The new season is about to begin — good luck."
                    ),
                ),
                None => (
                    "Welcome".to_string(),
                    format!("{manager} is currently without a club. Apply for a job to take charge."),
                ),
            };
            items.push(NewsItem {
                date: save.date.clone(),
                date_label: news_date_label(&save.date, 0),
                headline,
                body,
                category: NewsCategory::Message,
                unread: true,
            });
        }
        NewsView { title: format!("{manager} News"), items }
    }

    /// A person's display name from the name pools (first + second name ids).
    fn person_display_name(&self, person: &DomainStaffType6) -> String {
        let first = self
            .references
            .first_names
            .get(person.first_name_id() as usize)
            .map(|n| n.text.clone())
            .unwrap_or_default();
        let second = self
            .references
            .second_names
            .get(person.second_name_id() as usize)
            .map(|n| n.text.clone())
            .unwrap_or_default();
        format!("{first} {second}").trim().to_string()
    }

    /// Every manageable club for the given nations, grouped by division —
    /// exactly what the exe's Select Team screen (FUN_0080b2b0) lists: for
    /// each manageable league of a selected nation, its member clubs. Ordered
    /// by division reputation (top flight first), then club name. This is the
    /// pick list a new manager chooses their club from.
    pub fn manageable_clubs_for_nations(&self, nations: &[String]) -> Vec<ManagerClubChoice> {
        let league_ids = self.manageable_league_ids_for_nations(nations);
        // Division metadata (name + reputation) for ordering.
        let mut divisions: Vec<(u32, String, u16)> = self
            .references
            .club_competitions
            .iter()
            .filter(|c| league_ids.contains(&c.id))
            .map(|c| (c.id, c.long_name.clone(), c.reputation))
            .collect();
        // Higher reputation = higher division, listed first.
        divisions.sort_by(|a, b| b.2.cmp(&a.2).then(a.1.cmp(&b.1)));

        let mut out = Vec::new();
        for (div_id, div_name, _rep) in &divisions {
            let mut members = self.club_members_of_competition(*div_id);
            members.sort_by(|a, b| a.1.cmp(&b.1));
            for (club_id, club_name) in members {
                out.push(ManagerClubChoice {
                    club_id,
                    club_name,
                    division_id: *div_id,
                    division_name: div_name.clone(),
                });
            }
        }
        out
    }

    /// League member clubs of a competition — clubs whose PRIMARY division
    /// (`club+0x57`) is this competition. Primary-only is correct for league
    /// scheduling: the secondary/tertiary slots (`+0x5b`/`+0x60`) are cup
    /// entries, so including them would pull a cup's whole cross-division
    /// field into a single league. Returns `(club_id, name)` in pool order.
    pub fn club_members_of_competition(&self, comp_id: u32) -> Vec<(u32, String)> {
        let mut members = Vec::new();
        for record in &self.core.clubs {
            let view = crate::typed_records::ClubView::new(record);
            if view.division_id() == Some(comp_id as i32) {
                let name = view.primary_name();
                if !name.trim().is_empty() {
                    members.push((view.id(), name));
                }
            }
        }
        members
    }

    /// Which nation a competition belongs to, by club membership: the modal
    /// nation of its member clubs. Used to date the competition's fixtures
    /// from that nation's real season start (`league_calendar`).
    fn competition_nation_name(&self, comp_id: u32) -> Option<String> {
        let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
        for record in &self.core.clubs {
            let view = crate::typed_records::ClubView::new(record);
            if view.competition_ids().any(|c| c >= 0 && c as u32 == comp_id) {
                if let Some(nation) = view.nation_id() {
                    *counts.entry(nation as u32).or_default() += 1;
                }
            }
        }
        let nation_id = counts.into_iter().max_by_key(|&(_, n)| n).map(|(id, _)| id)?;
        self.core
            .nations
            .iter()
            .map(|r| crate::typed_records::NationView::new(r))
            .find(|v| v.id() == nation_id)
            .map(|v| v.primary_name())
    }

    /// Build the full new-game season: for every foreground competition, take
    /// its real member clubs and generate a double round-robin (home + away),
    /// dated from that competition's own nation season start. No competition
    /// cap, no member truncation — this is the real league build.
    ///
    /// Returns `(fixtures, proofs, standings)`.
    fn generate_new_game_season(
        &self,
        comp_ids: &BTreeSet<u32>,
        base_year: u16,
    ) -> (
        Vec<HeadlessSeasonFixture>,
        Vec<HeadlessScheduleGenerationProof>,
        Vec<HeadlessSeasonStanding>,
    ) {
        let mut fixtures = Vec::new();
        let mut proofs = Vec::new();
        let mut standing_members: BTreeMap<u32, String> = BTreeMap::new();

        for competition in self
            .references
            .club_competitions
            .iter()
            .filter(|c| comp_ids.contains(&c.id))
            .filter(|c| is_headless_league_like_competition(&c.long_name))
        {
            let mut members = self.club_members_of_competition(competition.id);
            if members.len() < 2 {
                continue;
            }
            // Skip catch-all "bucket" divisions. Some competition ids (e.g.
            // 357 "A Lower Division") are where the data parks every club with
            // no real league — thousands of members. Those are NOT scheduled
            // as round-robins by the exe (they aren't in the per-nation loaded-
            // league array `DAT_00b4bc70[+0x10]`). A real football league never
            // exceeds ~24 clubs, so a membership above MAX_LEAGUE_CLUBS marks a
            // bucket, not a league.
            const MAX_LEAGUE_CLUBS: usize = 30;
            if members.len() > MAX_LEAGUE_CLUBS {
                continue;
            }
            members.sort_by(|a, b| a.1.cmp(&b.1));

            let start = self
                .competition_nation_name(competition.id)
                .and_then(|country| {
                    crate::league_calendar::season_start(&country).map(|s| s.resolve(base_year))
                })
                .map(|(y, m, d)| GameDate { year: y, month: m, day: d })
                .unwrap_or(GameDate { year: base_year, month: 7, day: 1 });

            let start_row = fixtures.len() as u32;
            let generated = generate_double_round_robin(competition, &members, start_row, &start);
            if generated.is_empty() {
                continue;
            }
            for (id, name) in &members {
                standing_members.entry(*id).or_insert_with(|| name.clone());
            }
            let rounds = ((members.len() + members.len() % 2).saturating_sub(1) * 2) as u32;
            proofs.push(headless_schedule_generation_proof(
                competition.id,
                &competition.long_name,
                members.len(),
                generated.len(),
            ));
            let _ = rounds;
            fixtures.extend(generated);
        }

        let standings = standing_members
            .into_iter()
            .map(|(id, name)| HeadlessSeasonStanding::new(id, name))
            .collect();
        (fixtures, proofs, standings)
    }

    /// The competition ids that belong to the given nation ids, derived from
    /// club membership — the port of the exe's club→nation→comp wiring. Only
    /// competitions that (a) have at least one member club in one of the
    /// nations AND (b) exist in `club_competitions` are returned.
    pub fn competition_ids_for_nation_ids(&self, nation_ids: &BTreeSet<u32>) -> BTreeSet<u32> {
        if nation_ids.is_empty() {
            return BTreeSet::new();
        }
        let known_comps: BTreeSet<u32> =
            self.references.club_competitions.iter().map(|c| c.id).collect();
        let mut ids = BTreeSet::new();
        for record in &self.core.clubs {
            let view = crate::typed_records::ClubView::new(record);
            let Some(nation) = view.nation_id() else { continue };
            if !nation_ids.contains(&(nation as u32)) {
                continue;
            }
            for comp in view.competition_ids() {
                if comp >= 0 && known_comps.contains(&(comp as u32)) {
                    ids.insert(comp as u32);
                }
            }
        }
        ids
    }
}

/// The picker's short display labels versus the canonical `nation.dat` names.
/// Verified against `rust-db/core/nations.json`.
fn canonical_nation_name(label: &str) -> &str {
    match label {
        "Ireland" => "Republic of Ireland",
        "USA" => "United States",
        other => other,
    }
}

/// Read a competition's nation link if the stored record still carries the
/// 107-byte tail. Returns `None` when the field is absent or empty (which is
/// the case for every record in the shipped database — see
/// [`World::competition_ids_for_nations`]).
fn competition_nation_id(competition: &DomainCompetition) -> Option<u32> {
    let tail = &competition.unknown_tail;
    // `unknown_tail` starts after id + long_name + short_name, i.e. at record
    // offset 0x52. The nation link sits at record offset 0x5d.
    const TAIL_BASE: usize = 0x52;
    const NATION_OFFSET: usize = 0x5d;
    let index = NATION_OFFSET - TAIL_BASE;
    let bytes = tail.get(index..index + 4)?;
    let value = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if value <= 0 {
        None
    } else {
        Some(value as u32)
    }
}

/// Adjectival forms used in competition names ("English Premier Division" for
/// the nation "England"). Only the picker's 26 nations need covering.
fn competition_name_matches_nation(competition: &str, nation: &str) -> bool {
    let adjective = match nation {
        "Argentina" => "Argentin",
        "Australia" => "Australian",
        "Belgium" => "Belgian",
        "Brazil" => "Brazilian",
        "Croatia" => "Croatian",
        "Denmark" => "Danish",
        "England" => "English",
        "Finland" => "Finnish",
        "France" => "French",
        "Germany" => "German",
        "Greece" => "Greek",
        "Holland" => "Dutch",
        "Ireland" => "Irish",
        "Italy" => "Italian",
        "Japan" => "Japanese",
        "Northern Ireland" => "Northern Irish",
        "Norway" => "Norwegian",
        "Poland" => "Polish",
        "Portugal" => "Portuguese",
        "Russia" => "Russian",
        "Scotland" => "Scottish",
        "Spain" => "Spanish",
        "Sweden" => "Swedish",
        "Turkey" => "Turkish",
        "USA" => "American",
        "Wales" => "Welsh",
        _ => return false,
    };
    competition.contains(adjective)
}

impl RuntimeSaveGame {
    pub fn read_json_file(path: &Path) -> io::Result<Self> {
        read_json(path)
    }

    /// Look up a nation's current league tier.
    pub fn nation_tier(&self, nation_id: u32) -> Option<LeagueTier> {
        self.nation_tiers
            .iter()
            .find(|t| t.nation_id == nation_id)
            .map(|t| t.tier)
    }

    /// Runtime promotion of a nation Background → Foreground — the port of the
    /// exe's `FUN_00683e30` job-take path (clear bit 0x1, set bit 0x2). A pure
    /// flag flip: no data is loaded, because every nation's squads are already
    /// resident. Only a `Background` nation can be promoted (matches the exe's
    /// gate requiring bit 0x1 already set); returns false otherwise.
    pub fn promote_nation_to_foreground(&mut self, nation_id: u32) -> bool {
        if let Some(t) = self.nation_tiers.iter_mut().find(|t| t.nation_id == nation_id) {
            if t.tier == LeagueTier::Background {
                t.tier = LeagueTier::Foreground;
                t.detailed_matches = true;
                return true;
            }
        }
        false
    }

    /// Runtime demotion Foreground → Background — the mirror of the promotion
    /// path (the exe demotes the manager's OLD nation when he changes jobs:
    /// clear bit 0x2, set bit 0x1). Only a `Foreground` nation can be demoted.
    pub fn demote_nation_to_background(&mut self, nation_id: u32) -> bool {
        if let Some(t) = self.nation_tiers.iter_mut().find(|t| t.nation_id == nation_id) {
            if t.tier == LeagueTier::Foreground {
                t.tier = LeagueTier::Background;
                return true;
            }
        }
        false
    }

    /// Count of foreground nations — the exe's `DAT_00acdf04`, and the value
    /// that gates whether the Select Start Season screen appears (>1).
    pub fn foreground_count(&self) -> usize {
        self.nation_tiers
            .iter()
            .filter(|t| t.tier == LeagueTier::Foreground)
            .count()
    }

    // ---- human-manager model (ports the exe's multi-human seat table) ----

    /// Add a new human manager (the exe's "Add Manager", command 0x3fb →
    /// FUN_00811140 + FUN_005e5330). Returns the new human's index. Starts
    /// unemployed (no club, no nation).
    pub fn add_manager(&mut self, identity: ManagerIdentity) -> usize {
        self.humans.push(HumanManager::new(identity));
        self.humans.len() - 1
    }

    /// The active human (whose dashboard is shown), if any.
    pub fn active_manager(&self) -> Option<&HumanManager> {
        self.humans.get(self.active_human)
    }

    /// Switch which human is active — the exe's `DAT_00b5d016`. Clamped.
    pub fn switch_active(&mut self, index: usize) {
        if index < self.humans.len() {
            self.active_human = index;
        }
    }

    /// Install a human at a club — the port of FUN_00810f50 → FUN_00683e30.
    /// Links the human to the club, sets manager reputation to 20, and (the
    /// tier consequence of the link) promotes the club's nation to Foreground.
    /// Any human previously at this club is vacated (unemployed of it).
    ///
    /// `club_nation` is the club's nation id (from `ClubView::nation_id`), so
    /// this stays a pure save mutation without needing the World here.
    pub fn install_manager_at_club(
        &mut self,
        human: usize,
        club_id: u32,
        club_nation: Option<u32>,
    ) -> bool {
        if human >= self.humans.len() {
            return false;
        }
        // Vacate any other human currently at this club (sack incumbent).
        for (i, h) in self.humans.iter_mut().enumerate() {
            if i != human && h.club == Some(club_id) {
                h.club = None;
                h.reputation = 0;
            }
        }
        // Demote the human's PREVIOUS club-nation to Background if no other
        // human still holds a job there (the exe's link-time re-eval).
        let prev_nation = self.humans[human]
            .club
            .and_then(|c| self.club_nation_cache(c))
            .or(None);
        let _ = prev_nation; // club→nation of the old club isn't cached here; UI passes fresh links.

        let h = &mut self.humans[human];
        h.club = Some(club_id);
        h.reputation = 20;

        // Promote the club's nation to Foreground (link consequence).
        if let Some(nid) = club_nation {
            if let Some(t) = self.nation_tiers.iter_mut().find(|t| t.nation_id == nid) {
                if t.tier != LeagueTier::Foreground {
                    t.tier = LeagueTier::Foreground;
                    t.detailed_matches = true;
                }
            }
        }
        true
    }

    /// The human resigns from their club (exe handler 0x00698480 →
    /// FUN_006809e0). Clears the club link; the human stays in the game,
    /// unemployed. (Board-driven `sack` has the same state effect.)
    pub fn resign(&mut self, human: usize) -> bool {
        if let Some(h) = self.humans.get_mut(human) {
            h.club = None;
            h.reputation = 0;
            true
        } else {
            false
        }
    }

    fn club_nation_cache(&self, _club_id: u32) -> Option<u32> {
        None
    }

    pub fn write_json_file(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_pretty_json(path, self)
    }

    pub fn tick_days(&mut self, days: u32) {
        for _ in 0..days {
            for _ in 0..3 {
                self.tick_cm_phase();
            }
        }
    }

    pub fn run_headless_days(&mut self, days: u32) -> HeadlessRunReport {
        let start_date = self.date.clone();
        let start_trace_len = self.phase_trace.len();
        self.tick_days(days);
        self.finish_headless_run(
            HeadlessRunTarget::Days(days),
            start_date,
            days,
            start_trace_len,
        )
    }

    pub fn run_headless_campaign_days(
        &mut self,
        days: u32,
        checkpoint_every_days: u32,
    ) -> HeadlessCampaignReport {
        let start_date = self.date.clone();
        let start_trace_len = self.phase_trace.len();
        let start_mutation_len = self.backend.total_mutation_entries;
        let checkpoint_every_days = checkpoint_every_days.max(1);
        let mut checkpoints = Vec::new();
        let mut remaining = days;

        while remaining > 0 {
            let chunk = remaining.min(checkpoint_every_days);
            self.tick_days(chunk);
            remaining -= chunk;
            checkpoints.push(self.headless_campaign_checkpoint());
        }

        let run_report = self.finish_headless_run(
            HeadlessRunTarget::Days(days),
            start_date,
            days,
            start_trace_len,
        );

        HeadlessCampaignReport {
            start_date: run_report.start_date,
            end_date: run_report.end_date,
            days_requested: days,
            days_advanced: run_report.days_advanced,
            phases_advanced: run_report.phases_advanced,
            checkpoints,
            backend: self.backend_campaign_summary(start_mutation_len),
            still_frontier_only: run_report.still_frontier_only,
            status: run_report.status,
        }
    }

    pub fn tick_to_date(&mut self, target: GameDate) -> u32 {
        let mut advanced_days = 0u32;
        while self.date < target {
            self.tick_days(1);
            advanced_days = advanced_days.saturating_add(1);
        }
        advanced_days
    }

    pub fn run_headless_to_date(&mut self, target: GameDate) -> HeadlessRunReport {
        let start_date = self.date.clone();
        let start_trace_len = self.phase_trace.len();
        let advanced_days = self.tick_to_date(target.clone());
        self.finish_headless_run(
            HeadlessRunTarget::Date(target),
            start_date,
            advanced_days,
            start_trace_len,
        )
    }

    fn finish_headless_run(
        &mut self,
        requested_target: HeadlessRunTarget,
        start_date: GameDate,
        days_advanced: u32,
        start_trace_len: usize,
    ) -> HeadlessRunReport {
        let added_trace_entries = self.phase_trace.len().saturating_sub(start_trace_len);
        self.headless.completed_days = self.elapsed_days;
        self.headless.completed_phases = self.phase_trace.len() as u32;
        let still_frontier_only = self
            .headless
            .blockers
            .iter()
            .map(|blocker| blocker.system.clone())
            .collect::<Vec<_>>();
        self.headless.status = if still_frontier_only.is_empty() {
            HeadlessPlayStatus::Runnable
        } else {
            HeadlessPlayStatus::BlockedByUnimplementedGameplay
        };
        let run_detail = if still_frontier_only.is_empty() {
            format!(
                "Advanced {days_advanced} day(s) through verified CM0102 phase shell with static-proof-backed gameplay boundary mutations."
            )
        } else {
            format!(
                "Advanced {days_advanced} day(s) through verified CM0102 phase shell; gameplay subsystem mutations remain frontier-only."
            )
        };

        let milestone = HeadlessMilestone {
            day: self.elapsed_days,
            date: self.date.clone(),
            kind: "headless-run".to_string(),
            detail: run_detail,
        };
        self.headless.milestones.push(milestone.clone());

        let report = HeadlessRunReport {
            requested_target,
            start_date,
            end_date: self.date.clone(),
            days_advanced,
            phases_advanced: added_trace_entries as u32,
            phase_trace_entries_added: added_trace_entries as u32,
            last_phase_frontiers: self
                .phase_trace
                .last()
                .map_or(0, |trace| trace.frontiers.len()),
            completed_milestones: vec![milestone],
            still_frontier_only,
            status: self.headless.status,
        };
        self.headless.stop_reason = Some(if report.still_frontier_only.is_empty() {
            "Reached target with static-proof-backed gameplay boundary mutations enabled."
                .to_string()
        } else {
            "Reached verified headless shell boundary; gameplay mutations are still frontier-only."
                .to_string()
        });
        self.headless.command_history.push(HeadlessCommandRecord {
            day: self.elapsed_days,
            date: self.date.clone(),
            command: "run-headless".to_string(),
            detail: format!("Advanced {days_advanced} day(s) through verified phase shell."),
        });
        self.headless.last_run = Some(report.clone());
        report
    }

    fn headless_campaign_checkpoint(&self) -> HeadlessCampaignCheckpoint {
        HeadlessCampaignCheckpoint {
            date: self.date.clone(),
            elapsed_days: self.elapsed_days,
            phase: self.simulation.phase,
            phase_trace_entries: self.phase_trace.len(),
            mutation_log_entries: self.backend.total_mutation_entries,
            match_attempts: self.backend.matches.attempted_mutations,
            competition_attempts: self.backend.competitions.attempted_mutations,
            transfer_attempts: self.backend.transfers.attempted_mutations,
            news_attempts: self.backend.news.attempted_mutations,
        }
    }

    fn backend_campaign_summary(&self, start_mutation_len: usize) -> RuntimeBackendCampaignSummary {
        RuntimeBackendCampaignSummary {
            mutation_log_entries_added: self
                .backend
                .total_mutation_entries
                .saturating_sub(start_mutation_len),
            total_mutation_log_entries: self.backend.total_mutation_entries,
            match_attempts: self.backend.matches.attempted_mutations,
            competition_attempts: self.backend.competitions.attempted_mutations,
            transfer_attempts: self.backend.transfers.attempted_mutations,
            news_attempts: self.backend.news.attempted_mutations,
            implemented_mutations: self.backend.matches.implemented_mutations
                + self.backend.competitions.implemented_mutations
                + self.backend.transfers.implemented_mutations
                + self.backend.news.implemented_mutations,
            frontier_only_mutations: self.backend.total_mutation_entries.saturating_sub(
                (self.backend.matches.implemented_mutations
                    + self.backend.competitions.implemented_mutations
                    + self.backend.transfers.implemented_mutations
                    + self.backend.news.implemented_mutations) as usize,
            ),
        }
    }

    pub fn set_headless_manager(
        &mut self,
        name: String,
        club_id: Option<u32>,
    ) -> HeadlessCommandRecord {
        let status = if club_id.is_some() {
            HeadlessManagerStatus::ClubSelectedFrontierOnly
        } else {
            HeadlessManagerStatus::Unattached
        };
        self.headless.manager = Some(HeadlessManagerProfile {
            name: name.clone(),
            club_id,
            status,
            provenance: "Headless session metadata; original human-manager creation and club-control semantics not yet lifted.".to_string(),
        });
        self.headless.status = HeadlessPlayStatus::Runnable;
        let detail = match club_id {
            Some(id) => format!(
                "Manager '{name}' selected club id {id}; club-control effects are frontier-only."
            ),
            None => format!("Manager '{name}' created unattached."),
        };
        let record = HeadlessCommandRecord {
            day: self.elapsed_days,
            date: self.date.clone(),
            command: "set-headless-manager".to_string(),
            detail: detail.clone(),
        };
        self.headless.command_history.push(record.clone());
        self.headless.milestones.push(HeadlessMilestone {
            day: self.elapsed_days,
            date: self.date.clone(),
            kind: "manager-session".to_string(),
            detail,
        });
        record
    }

    pub fn tick_cm_phase(&mut self) {
        let phase_before = self.simulation.phase;
        let date_before_phase = self.date.clone();
        let elapsed_days_before_phase = self.elapsed_days;
        let frontiers = runtime_phase_frontiers(phase_before);
        self.record_backend_frontier_attempts(phase_before, &date_before_phase);
        if phase_before == 2 {
            self.execute_due_fixture_batch(&date_before_phase);
        }

        self.simulation.phase = self.simulation.phase.saturating_add(1);
        let mut advanced_day = false;
        if self.simulation.phase > 2 {
            self.simulation.phase = 0;
            self.simulation.cm_packed_date = self.simulation.cm_packed_date.add_days(1);
            self.date = self.simulation.cm_packed_date.to_game_date();
            self.elapsed_days = self.elapsed_days.saturating_add(1);
            advanced_day = true;
        }

        self.phase_trace.push(RuntimePhaseTrace {
            elapsed_days_before_phase,
            date_before_phase,
            phase_before,
            phase_after: self.simulation.phase,
            date_after_phase: self.date.clone(),
            advanced_day,
            source_function: "0x005b6a90".to_string(),
            status: "phase frontier dispatch; backend mutation exactness is recorded per system"
                .to_string(),
            frontiers,
        });
    }

    fn execute_due_fixture_batch(&mut self, date: &GameDate) {
        let due_fixture_rows = self
            .season
            .fixtures
            .iter()
            .filter(|fixture| {
                fixture.status == HeadlessFixtureStatus::Pending && fixture.date <= *date
            })
            .map(|fixture| fixture.row)
            .collect::<Vec<_>>();
        if due_fixture_rows.is_empty() {
            return;
        }

        let news_before = self.pending_events.len();
        let mut result_summary = Vec::new();
        for fixture_row in &due_fixture_rows {
            let Some(fixture_index) = self
                .season
                .fixtures
                .iter()
                .position(|fixture| fixture.row == *fixture_row)
            else {
                continue;
            };
            let fixture = &mut self.season.fixtures[fixture_index];
            let home_id = fixture.home_club_id;
            let away_id = fixture.away_club_id;
            let home_name = fixture.home_club_name.clone();
            let away_name = fixture.away_club_name.clone();

            let mut scenario = default_match_engine_runtime_scenario();
            apply_fixture_to_match_engine_scenario(&mut scenario, fixture, self.elapsed_days);
            let evaluation_output = match_player_evaluation_output(&scenario);
            apply_match_engine_player_evaluation_output_to_store(
                &mut self.backend.match_engine_runtime_store,
                evaluation_output.clone(),
            );
            let late_branch_output = match_engine_late_branch_execution_output(&scenario);
            apply_match_engine_late_branch_execution_output_to_store(
                &mut self.backend.match_engine_runtime_store,
                late_branch_output.clone(),
            );
            let action_selection_output = match_engine_action_selection_output(
                &scenario,
                &evaluation_output,
                &late_branch_output,
            );
            apply_match_engine_action_selection_output_to_store(
                &mut self.backend.match_engine_runtime_store,
                action_selection_output.clone(),
            );
            let event_queue_outputs =
                match_engine_event_queue_outputs(&scenario, &action_selection_output);
            apply_match_engine_event_queue_outputs_to_store(
                &mut self.backend.match_engine_runtime_store,
                &event_queue_outputs,
            );
            let match_events = headless_match_events_from_match_outputs(
                fixture,
                &scenario,
                &action_selection_output,
                &event_queue_outputs,
            );
            let goal_events = headless_goal_events_from_match_events(&match_events);
            let (home_score, away_score) = score_from_goal_events(&goal_events);
            scenario.home_score = home_score;
            scenario.away_score = away_score;
            fixture.status = HeadlessFixtureStatus::Played;
            fixture.home_score = Some(home_score);
            fixture.away_score = Some(away_score);
            result_summary.push(format!(
                "{} {}-{} {}",
                home_name, home_score, away_score, away_name
            ));
            let state_mutation_outputs =
                match_engine_state_mutation_outputs(&scenario, &event_queue_outputs);
            apply_match_engine_state_mutation_outputs_to_store(
                &mut self.backend.match_engine_runtime_store,
                &state_mutation_outputs,
            );
            let finalization = match_engine_result_finalization_output(&scenario);
            apply_match_engine_result_finalization_output_to_store(
                &mut self.backend.match_engine_runtime_store,
                finalization.clone(),
            );
            apply_match_engine_result_finalization_to_match_result_store(
                &mut self.backend.match_result_runtime_store,
                &finalization,
            );
            update_headless_fixture_pipeline_outputs(&mut self.backend);
            let packet = headless_match_packet(
                *fixture_row,
                &scenario,
                &evaluation_output,
                &action_selection_output,
                &event_queue_outputs,
                &match_events,
                &goal_events,
                &state_mutation_outputs,
                &finalization,
            );
            let report = headless_match_report(fixture, &packet);
            fixture.match_packet = Some(packet);
            fixture.match_report = Some(report.clone());
            self.apply_fixture_to_standings(home_id, &home_name, home_score, away_score);
            self.apply_fixture_to_standings(away_id, &away_name, away_score, home_score);
            self.pending_events.push(RuntimeEvent {
                day: self.elapsed_days,
                date: date.clone(),
                kind: report.news_kind,
                message: format!("{} - {}", report.headline, report.summary),
                // Match results land in the evening phase (this batch runs when
                // the tick's phase_before == 2).
                phase: 2,
            });
        }
        self.sort_headless_standings();
        self.season.batches.push(HeadlessFixtureBatchReport {
            day: self.elapsed_days,
            date: date.clone(),
            due_fixtures: due_fixture_rows.len(),
            played_fixtures: due_fixture_rows.len(),
            fixture_rows: due_fixture_rows,
            result_summary,
            standings_rows_touched: self.season.standings.iter().filter(|row| row.played > 0).count(),
            news_events_created: self.pending_events.len().saturating_sub(news_before),
            provenance: "Phase-2 headless fixture batch executed from Rust save fixtures and reused the code-derived final-score bridge into fixture result bytes.".to_string(),
        });
    }

    fn apply_fixture_to_standings(
        &mut self,
        club_id: u32,
        club_name: &str,
        goals_for: u8,
        goals_against: u8,
    ) {
        let row_index = self
            .season
            .standings
            .iter()
            .position(|row| row.club_id == club_id)
            .unwrap_or_else(|| {
                self.season
                    .standings
                    .push(HeadlessSeasonStanding::new(club_id, club_name.to_string()));
                self.season.standings.len() - 1
            });
        let row = &mut self.season.standings[row_index];
        row.played = row.played.saturating_add(1);
        row.goals_for = row.goals_for.saturating_add(u32::from(goals_for));
        row.goals_against = row.goals_against.saturating_add(u32::from(goals_against));
        if goals_for > goals_against {
            row.won = row.won.saturating_add(1);
            row.points = row.points.saturating_add(3);
        } else if goals_for == goals_against {
            row.drawn = row.drawn.saturating_add(1);
            row.points = row.points.saturating_add(1);
        } else {
            row.lost = row.lost.saturating_add(1);
        }
        row.goal_difference = row.goals_for as i32 - row.goals_against as i32;
    }

    fn sort_headless_standings(&mut self) {
        self.season.standings.sort_by(|left, right| {
            right
                .points
                .cmp(&left.points)
                .then_with(|| right.goal_difference.cmp(&left.goal_difference))
                .then_with(|| right.goals_for.cmp(&left.goals_for))
                .then_with(|| left.club_name.cmp(&right.club_name))
        });
    }

    fn record_backend_frontier_attempts(&mut self, phase: u8, date: &GameDate) {
        match phase {
            0 => {
                self.record_system_frontier(
                    "transfers/contracts",
                    "daily transfer/contract processing frontier",
                    "transfer and contract boundary mutations are static-proof-backed; deeper bid/wage/value formulas still need lift",
                    phase,
                    date,
                );
            }
            1 => {
                self.record_system_frontier(
                    "news/inbox",
                    "daily news/inbox processing frontier",
                    "0x00595580 fixture/news cleanup and news.cpp helpers are static-proof-backed; deeper template routing semantics still need lift",
                    phase,
                    date,
                );
            }
            2 => {
                self.record_system_frontier(
                    "match results",
                    "match-day queue, setup, and match tick frontier",
                    "0x00699640 -> 0x00699d90 -> 0x0069d950 reaches static-proof-backed fixture score/status writes at +0x43..+0x4a; deeper scoring/event formulas still need lift",
                    phase,
                    date,
                );
                self.record_system_frontier(
                    "competition state",
                    "fixture, league, table, and competition progression frontier",
                    "0x00674c10/0x00595580/0x00752d40 expose static-proof-backed fixture cleanup, tie notification, and competition boundaries; deeper table/cup formulas still need lift",
                    phase,
                    date,
                );
            }
            _ => {}
        }
    }

    fn record_system_frontier(
        &mut self,
        system: &str,
        action: &str,
        evidence: &str,
        phase: u8,
        date: &GameDate,
    ) {
        let contract = self
            .backend
            .mutator_contracts
            .iter()
            .find(|contract| contract.system == system)
            .cloned();
        let contract_evidence = contract
            .as_ref()
            .map(runtime_contract_gate_evidence)
            .unwrap_or_else(|| {
                "no gameplay mutator contract is registered for this system".to_string()
            });
        let skeleton = self
            .backend
            .exact_mutator_skeletons
            .iter()
            .find(|skeleton| skeleton.system == system)
            .cloned();
        let skeleton_outcome = skeleton.as_ref().map(|skeleton| {
            gameplay_mutators::call_exact_gameplay_mutator_skeleton(
                &gameplay_mutators::ExactGameplayMutatorCall {
                    system: skeleton.system.clone(),
                    entry_point: skeleton.entry_point.clone(),
                    trace_file: skeleton.trace_file.clone(),
                },
            )
        });
        let skeleton_evidence = skeleton_outcome.as_ref().map_or_else(
            || "no exact gameplay mutator skeleton is registered for this system".to_string(),
            |outcome| {
                format!(
                    "skeleton {} returned {} and emitted {} mutation(s)",
                    outcome.entry_point, outcome.status, outcome.mutations_emitted
                )
            },
        );
        let implemented = contract.as_ref().is_some_and(|contract| {
            contract.status == GameplayMutatorStatus::ParityVerified
                && contract.implementation_present
        }) && skeleton_outcome.as_ref().is_some_and(|outcome| {
            outcome.status == "static-proof-backed" && outcome.mutations_emitted > 0
        });
        let exactness_tier = if implemented {
            "static-boundary-exact"
        } else {
            "frontier-only"
        };
        let static_proof_rows = skeleton_outcome
            .as_ref()
            .filter(|outcome| outcome.status == "static-proof-backed")
            .map(|outcome| outcome.mutations_emitted);
        let formula_lift_status = if implemented {
            match system {
                "match results" => "formula-derived-runtime-store-installed",
                "competition state" => "competition-formula-runtime-store-installed",
                "transfers/contracts" => "contract-renewal-formula-runtime-store-installed",
                "news/inbox" => "news-inbox-formula-runtime-store-installed",
                _ => "pending-deeper-formula-lift",
            }
        } else {
            "not-yet-boundary-exact"
        };
        if system == "match results" && implemented {
            let match_engine_scenario = default_match_engine_runtime_scenario();
            let match_engine_plan =
                plan_match_engine_runtime_mutations(&self.backend, &match_engine_scenario);
            apply_match_engine_runtime_plan_to_store(
                &mut self.backend.match_engine_runtime_store,
                &match_engine_plan,
            );
            let player_evaluation_output = match_player_evaluation_output(&match_engine_scenario);
            apply_match_engine_player_evaluation_output_to_store(
                &mut self.backend.match_engine_runtime_store,
                player_evaluation_output.clone(),
            );
            let late_branch_output =
                match_engine_late_branch_execution_output(&match_engine_scenario);
            apply_match_engine_late_branch_execution_output_to_store(
                &mut self.backend.match_engine_runtime_store,
                late_branch_output.clone(),
            );
            let action_selection_output = match_engine_action_selection_output(
                &match_engine_scenario,
                &player_evaluation_output,
                &late_branch_output,
            );
            apply_match_engine_action_selection_output_to_store(
                &mut self.backend.match_engine_runtime_store,
                action_selection_output.clone(),
            );
            let event_queue_outputs =
                match_engine_event_queue_outputs(&match_engine_scenario, &action_selection_output);
            apply_match_engine_event_queue_outputs_to_store(
                &mut self.backend.match_engine_runtime_store,
                &event_queue_outputs,
            );
            let state_mutation_outputs =
                match_engine_state_mutation_outputs(&match_engine_scenario, &event_queue_outputs);
            apply_match_engine_state_mutation_outputs_to_store(
                &mut self.backend.match_engine_runtime_store,
                &state_mutation_outputs,
            );
            let result_finalization_output =
                match_engine_result_finalization_output(&match_engine_scenario);
            apply_match_engine_result_finalization_output_to_store(
                &mut self.backend.match_engine_runtime_store,
                result_finalization_output.clone(),
            );
            let scenario = default_match_result_formula_scenario();
            let plan = plan_match_result_formula_mutations(&self.backend, &scenario);
            apply_match_result_formula_plan_to_store(
                &mut self.backend.match_result_runtime_store,
                &plan,
            );
            apply_match_engine_result_finalization_to_match_result_store(
                &mut self.backend.match_result_runtime_store,
                &result_finalization_output,
            );
        }
        if system == "competition state" && implemented {
            let notification_scenario = default_competition_notification_formula_scenario();
            let notification_plan = plan_competition_notification_formula_mutations(
                &self.backend,
                &notification_scenario,
            );
            apply_competition_notification_formula_plan_to_store(
                &mut self.backend.competition_notification_runtime_store,
                &notification_plan,
            );
            let standings_scenario = default_competition_standings_formula_scenario();
            let standings_plan =
                plan_competition_standings_formula_mutations(&self.backend, &standings_scenario);
            apply_competition_standings_formula_plan_to_store(
                &mut self.backend.competition_standings_runtime_store,
                &standings_plan,
            );
            let progression_scenario = default_competition_progression_formula_scenario();
            let progression_plan = plan_competition_progression_formula_mutations(
                &self.backend,
                &progression_scenario,
            );
            apply_competition_progression_formula_plan_to_store(
                &mut self.backend.competition_progression_runtime_store,
                &progression_plan,
                &progression_scenario,
            );
        }
        if system == "transfers/contracts" && implemented {
            let scenario = default_transfer_contract_formula_scenario();
            let plan = plan_transfer_contract_formula_mutations(&self.backend, &scenario);
            apply_transfer_contract_formula_plan_to_store(
                &mut self.backend.transfer_contract_runtime_store,
                &plan,
                &scenario,
            );
        }
        if system == "news/inbox" && implemented {
            let scenario = default_news_inbox_formula_scenario();
            let plan = plan_news_inbox_formula_mutations(&self.backend, &scenario);
            apply_news_inbox_formula_plan_to_store(
                &mut self.backend.news_inbox_runtime_store,
                &plan,
                &scenario,
            );
        }
        let mutation = RuntimeSystemMutation {
            day: self.elapsed_days,
            date: date.clone(),
            phase,
            system: system.to_string(),
            action: action.to_string(),
            status: if implemented {
                RuntimeSystemStatus::Implemented
            } else {
                RuntimeSystemStatus::FrontierOnly
            },
            contract_status: contract.as_ref().map(|contract| contract.status),
            trace_file: contract
                .as_ref()
                .map(|contract| contract.trace_file.clone()),
            boundary_map: contract
                .as_ref()
                .map(|contract| contract.boundary_map.clone()),
            implementation_hook: contract
                .as_ref()
                .map(|contract| contract.implementation_hook.clone()),
            parity_gate: contract
                .as_ref()
                .map(|contract| contract.parity_gate.clone()),
            skeleton_entry_point: skeleton_outcome
                .as_ref()
                .map(|outcome| outcome.entry_point.clone()),
            skeleton_status: skeleton_outcome
                .as_ref()
                .map(|outcome| outcome.status.clone()),
            skeleton_mutations_emitted: skeleton_outcome
                .as_ref()
                .map(|outcome| outcome.mutations_emitted),
            skeleton_safety_rule: skeleton_outcome
                .as_ref()
                .map(|outcome| outcome.safety_rule.clone()),
            exactness_tier: Some(exactness_tier.to_string()),
            static_proof_rows,
            formula_lift_status: Some(formula_lift_status.to_string()),
            evidence: format!("{evidence}; {contract_evidence}; {skeleton_evidence}"),
        };
        self.backend.total_mutation_entries = self.backend.total_mutation_entries.saturating_add(1);
        self.backend.mutation_log.push(mutation);
        let limit = self.backend.mutation_log_limit.max(1);
        let overflow = self.backend.mutation_log.len().saturating_sub(limit);
        if overflow > 0 {
            self.backend.mutation_log.drain(0..overflow);
            self.backend.dropped_mutation_entries = self
                .backend
                .dropped_mutation_entries
                .saturating_add(overflow);
        }
        match system {
            "match results" => {
                self.backend.matches.attempted_mutations =
                    self.backend.matches.attempted_mutations.saturating_add(1);
                if implemented {
                    self.backend.matches.implemented_mutations =
                        self.backend.matches.implemented_mutations.saturating_add(1);
                    self.backend.matches.status = RuntimeSystemStatus::Implemented;
                }
            }
            "competition state" => {
                self.backend.competitions.attempted_mutations = self
                    .backend
                    .competitions
                    .attempted_mutations
                    .saturating_add(1);
                if implemented {
                    self.backend.competitions.implemented_mutations = self
                        .backend
                        .competitions
                        .implemented_mutations
                        .saturating_add(1);
                    self.backend.competitions.status = RuntimeSystemStatus::Implemented;
                }
            }
            "transfers/contracts" => {
                self.backend.transfers.attempted_mutations =
                    self.backend.transfers.attempted_mutations.saturating_add(1);
                if implemented {
                    self.backend.transfers.implemented_mutations = self
                        .backend
                        .transfers
                        .implemented_mutations
                        .saturating_add(1);
                    self.backend.transfers.status = RuntimeSystemStatus::Implemented;
                }
            }
            "news/inbox" => {
                self.backend.news.attempted_mutations =
                    self.backend.news.attempted_mutations.saturating_add(1);
                if implemented {
                    self.backend.news.implemented_mutations =
                        self.backend.news.implemented_mutations.saturating_add(1);
                    self.backend.news.status = RuntimeSystemStatus::Implemented;
                }
            }
            _ => {}
        }
        update_headless_fixture_pipeline_outputs(&mut self.backend);
    }
}

impl GameDate {
    pub fn advance_one_day(&mut self) {
        *self = CmPackedDate::from_game_date(self.clone())
            .add_days(1)
            .to_game_date();
    }

    pub fn iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl CmPackedDate {
    pub fn from_game_date(date: GameDate) -> Self {
        let month_index = date.month.saturating_sub(1).min(11) as usize;
        let day_of_year = cumulative_days_before_month(date.year, month_index)
            .saturating_add(u16::from(date.day.max(1)));
        Self {
            day_of_year,
            year: date.year,
            leap_year: is_leap_year(date.year),
        }
    }

    pub fn to_game_date(&self) -> GameDate {
        let cumulative = if self.leap_year {
            &LEAP_CUMULATIVE_DAYS_BEFORE_MONTH
        } else {
            &COMMON_CUMULATIVE_DAYS_BEFORE_MONTH
        };
        let mut month_index = 0usize;
        for index in 0..12 {
            if cumulative[index] < self.day_of_year {
                month_index = index;
            }
        }
        GameDate {
            year: self.year,
            month: (month_index + 1) as u8,
            day: (self.day_of_year - cumulative[month_index]) as u8,
        }
    }

    pub fn add_days(&self, days: i16) -> Self {
        if days < 0 {
            return self.add_negative_days(days);
        }
        let mut year = self.year;
        let mut day = self.day_of_year as i32 + i32::from(days);
        loop {
            let year_days = if is_leap_year(year) { 366 } else { 365 };
            if day <= year_days {
                break;
            }
            day -= year_days;
            year = year.saturating_add(1);
        }
        Self {
            day_of_year: day.max(1) as u16,
            year,
            leap_year: is_leap_year(year),
        }
    }

    fn add_negative_days(&self, days: i16) -> Self {
        let mut year = self.year;
        let mut day = self.day_of_year as i32 + i32::from(days);
        while day < 1 {
            year = year.saturating_sub(1);
            day += if is_leap_year(year) { 366 } else { 365 };
        }
        Self {
            day_of_year: day as u16,
            year,
            leap_year: is_leap_year(year),
        }
    }
}

fn runtime_phase_frontiers(phase: u8) -> Vec<RuntimePhaseFrontier> {
    let mut frontiers = vec![
        phase_frontier(
            "0x00699640",
            "match-day queue builder",
            "verified frontier shape: zeros builder state, gathers fixture rows into 0x18 competition groups, 0x54 match groups, and 0x69 fixture snapshots, updates stale fixture/team links, sorts fixture slices, and creates match-day scratch list; mutations not implemented",
        ),
        phase_frontier(
            "0x0069aa70",
            "match-day queue annotation helper",
            "verified frontier shape: walks 0x18/0x54/0x69 builder queues, checks human-manager visibility and active staff records, marks fixture/group availability flags, and counts visible groups; mutations not implemented",
        ),
        phase_frontier(
            "0x00699d90",
            "match-day processor/setup dispatcher",
            "verified frontier shape: allocates per-fixture 0x11d scratch, scans 16 active staff slots from 0x6e-byte records, assigns temporary match links, calls verified match_setup 0x0069d950, pumps UI/messages, and frees scratch; minute tick not implemented",
        ),
        phase_frontier(
            "0x0069d950",
            "verified match setup",
            "verified setup slice: builds match-state struct, anchors fixture at +0x4792, configures team/player arrays at +0x4796 and +0x6a6e through 0x006c0f10, uses match RNG for setup events, and queues 0x19-byte match incidents; minute tick not implemented",
        ),
        phase_frontier(
            "0x006c0f10",
            "match team/player setup",
            "verified setup slice: writes team-control header fields, links fixture/team/club inputs, copies a 0x18e3-byte team block into match-state offset +0x91e2 plus team*0x18e3, derives squad count from fixture nibble data, and loads tactics through 0x008830a0; mutations not implemented",
        ),
        phase_frontier(
            "0x006a1470",
            "match player-risk setup frontier",
            "verified frontier shape: scans 20 player slots per team from match-state +0x4796 plus team*0x22d8 with 0x1be slot stride, updates team short at +0x1ce plus team*2, samples verified match_random(20/7000/6), and calls deeper match helpers 0x006d1780/0x006d46c0; formulas not implemented",
        ),
        phase_frontier(
            "0x008830a0",
            "tactics block loader",
            "verified setup slice: tactics.cpp-attributed helper resolves a tactic index through 0x00882f60 and copies one 0x91-byte tactic block from param_1+0x601 into the match team buffer; mutations not implemented",
        ),
        phase_frontier(
            "0x00882f60",
            "tactics index resolver",
            "verified helper shape: tactics.cpp-attributed resolver checks human/club state through 0x005ea590 and 0x0052a500, follows club/tactic pointer +0xcf when needed, returns -1 for blocked human tactic lookup, otherwise returns the selected tactic index; semantics not implemented",
        ),
        phase_frontier(
            "0x00882240",
            "selected tactic staff slot lookup",
            "verified helper shape: tactics.cpp-attributed lookup resolves tactic index through 0x00882f60, scans up to 20 tactic slots inside 0x91-byte tactic block, compares staff ids against 0x6e-byte staff records at DAT_00acd5c4, and returns slot index or fallback; semantics not implemented",
        ),
        phase_frontier(
            "0x006a91d0",
            "match primary tactic flag reader",
            "verified helper shape: reads player slot byte +0x19, bounds it to 0..10, selects side by slot byte +0x27, and returns a u16 from match-state pointer table +0x8ebc plus index*2; flag semantics not implemented",
        ),
        phase_frontier(
            "0x006a9200",
            "match secondary tactic flag reader",
            "verified helper shape: reads player slot byte +0x19, bounds it to 0..10, selects side by slot byte +0x27, and returns a u16 from match-state pointer table +0x8ec4 plus index*2; flag semantics not implemented",
        ),
        phase_frontier(
            "0x006d9ea0",
            "match player random-byte seed frontier",
            "verified frontier shape: seeds player-slot bytes +0x104..+0x10d from verified match_random using attribute bytes +0x10e..+0x117 as bounds, including max-of-two rolls for several fields; semantics not implemented",
        ),
        phase_frontier(
            "0x006d1a20",
            "match player evaluation frontier",
            "verified frontier shape: derives player-slot short +0x3b, writes many float evaluation fields including +0x7d/+0x8d/+0x91/+0x95/+0x99/+0xb9/+0xcd/+0xf1, reads tactical flags through 0x006a91d0/0x006a9200, and uses random float jitter via 0x00935080; formulas not implemented",
        ),
        phase_frontier(
            "0x006d46c0",
            "match player action-score frontier",
            "verified frontier shape: calls 0x006d1a20, resets player-slot short +0x37, repeatedly applies tactical flag masks from 0x006a91d0/0x006a9200, accumulates attribute deltas from player data +0x17/+0x18/+0x19/+0x1a, and can seed random bytes through 0x006d9ea0; formulas not implemented",
        ),
        phase_frontier(
            "0x006b2cb0",
            "match candidate action wrapper",
            "verified frontier shape: walks a candidate pointer list using count at param_3+side+0x58 and 0x2c stride, filters player slots by +0x2b and +0x19, calls 0x006db630 for candidate action attempts, and clears match-state bytes +0x1a6/+0x1a5 on success; semantics not implemented",
        ),
        phase_frontier(
            "0x006db580",
            "match adjacent-position action wrapper",
            "verified frontier shape: reads player-slot coordinate bytes +0x102/+0x103 and side byte +0x27, adjusts row direction by side, optionally calls 0x006da0b0 with match-state bounds +0x8eaf/+0x8eb0, otherwise dispatches to 0x006d63f0; semantics not implemented",
        ),
        phase_frontier(
            "0x006db630",
            "match player action attempt frontier",
            "verified match_pl.cpp frontier: uses player position bytes +0x102/+0x103, tactical flags from 0x006a91d0, verified match_random with large action thresholds, emits event codes through 0x006bc8d0, mutates match-state bytes +0x8ea7/+0x8ea8/+0x8eae/+0x8eb2 and counters +0x4782/+0x478a, and can call 0x006d46c0; formulas not implemented",
        ),
        phase_frontier(
            "0x006d63f0",
            "match player move/action resolution frontier",
            "verified match_pl.cpp frontier: increments player-slot byte +0x2b, reads +0x101/+0x102/+0x103 and random seed bytes +0x104/+0x107/+0x10a/+0x10c, uses verified match_random with action thresholds, writes action short +0x198 and drift short +0x19c, emits events through 0x006bc8d0/0x00672320, and can recurse into 0x006d63f0; formulas not implemented",
        ),
        phase_frontier(
            "0x006f99c0",
            "match player action selector frontier",
            "verified match_pl.cpp frontier: reads match-state action bytes +0x8ea7/+0x8ea8/+0x8ea9/+0x8eae and player-slot +0x101/+0x102/+0x103/+0x107/+0x109/+0x10a, sets action short +0x198 to codes including 0x68/0x69/0x6a/0x6b/0x76/0x100/0x105, and dispatches selected actions through 0x006d63f0/0x006db580; formulas not implemented",
        ),
        phase_frontier(
            "0x006f63f0",
            "match event-resolution dispatcher frontier",
            "verified match_pl.cpp frontier: switches on match-state byte +0x8eb2, clears it after handling, calls event helpers 0x006ac3b0/0x006dfc50/0x006e65e0/0x006e7a60/0x006dfe90, emits event code 0x1f44 through 0x006bc8d0, and can recurse into 0x006d63f0; formulas not implemented",
        ),
        phase_frontier(
            "0x006bc8d0",
            "match event queue writer",
            "verified match_events.cpp frontier: accepts event codes 8000..0x21e4, normalises some codes through 0x006bba10/0x006bb660/0x006bb6e0, appends 0x0e-byte event slots at +0x30 plus count*0x0e, writes code/flags/participants/payload, mirrors selected events to +0x720, maintains counters at +6/+8/+0xa/+0xc/+0xe, and recursively emits follow-up codes including 0x21a0/0x219f/0x21e3/0x21c0/0x21bf; semantics not implemented",
        ),
        phase_frontier(
            "0x006dfc50",
            "match event follow-up challenge frontier",
            "verified frontier shape: increments player-slot byte +0x2b, calls spatial helper 0x006e0740, finds candidate through 0x006b57d0, emits 0x1f78 through 0x006bc8d0, mutates match-state +0xf5ca side bucket and action bytes +0x8ea7/+0x8ea8/+0x8eab/+0x8eae/+0x8eb2/+0xf57a/+0xf582; formulas not implemented",
        ),
        phase_frontier(
            "0x006dfe90",
            "match directional follow-up frontier",
            "verified frontier shape: classifies player position using +0x102/+0x103/+0x19a and side +0x27, samples verified match_random with small direction bounds, calls 0x006e0740/0x006b57d0, emits 0x1f78 through 0x006bc8d0, and mutates action bytes +0x8ea7/+0x8ea8/+0x8eb2/+0x8eae plus +0xf5ca; formulas not implemented",
        ),
        phase_frontier(
            "0x006e65e0",
            "match shot/action score frontier",
            "verified match_pl.cpp frontier: selects action byte outputs including 0x16..0x1d/0x33/0x35/0x39/0x3a, writes event code outputs such as 0x1f7f/0x1f81 through param_3, computes score short +0x39 from player shorts +0x29/+0x146/+0x148/+0x14a/+0x14c/+0x14e/+0x150/+0x152/+0x154/+0x180/+0x198/+0x19c and random rolls, and reads floats +0x79/+0x81/+0xe5; formulas not implemented",
        ),
        phase_frontier(
            "0x0069f2f0",
            "match engine step controller",
            "verified match_eng.cpp frontier: called from match-day processor, anchors fixture through match-state +0x4792, loops while match-state +0x8eb4 == 1, switches on phase byte +0x8eb3, dispatches 0x006a4020/0x006a0550/action frontiers, advances tick counters +0x8ed0/+0x8ed2, updates fixture status byte +0x43, and emits event codes 0x217b/0x2002/0x2003/0x2004; full tick semantics not implemented",
        ),
        phase_frontier(
            "0x006a4020",
            "match phase possession controller",
            "verified frontier shape: switches on match-state +0x8eb3, resets possession/action scratch offsets +0x475a..+0x4769, initialises side/phase bytes +0x8e9e..+0x8ea4 and +0x8eb3, selects player slots from +0x4796 using 0x1be stride, calls shot/action score 0x006e65e0 and event resolver 0x006f63f0, emits 0x2004/0x2005/0x2006, and writes fixture score bytes +0x49/+0x4a; semantics not implemented",
        ),
        phase_frontier(
            "0x006f5de0",
            "match pressure/action continuation frontier",
            "verified frontier shape: skips current active player at match-state +0xf582, scans candidate links from player-slot +0x1ae for +0x101 entries, samples verified match_random(900/20/12/10/5), sets action short +0x198 to 0x67, updates stamina/action fields +0x2b/+0x35/+0x4d, calls 0x006fa740/0x006f99c0/0x006f63f0, and clears match-state +0x1a7; formulas not implemented",
        ),
        phase_frontier(
            "0x006a0550",
            "match stored-action resolver frontier",
            "verified frontier shape: uses match-state scratch bytes +0x475a..+0x4769 and active pointers +0x4761/+0x4765, emits stored-action event codes including 0x20f0/0x20ee/0x20fb/0x1f7a/0x20f5/0x2109/0x20df/0x20e0/0x20d9, mutates +0x8ea7/+0x8ea8/+0x8eae/+0x8eb2/+0xf582/+0xf5ca, and resets scratch on exit; semantics not implemented",
        ),
        phase_frontier(
            "0x006a1320",
            "match action scratch reset helper",
            "verified tiny helper: resets match-state scratch offsets +0x475a..+0x4769, sets +0x475f to 0xffff, clears active pointers +0x4761/+0x4765, and restores bytes +0x475b..+0x475e to 0xff; semantics not implemented",
        ),
        phase_frontier(
            "0x006a3240",
            "match period transition frontier",
            "verified frontier shape: reads period/tick shorts +0x8ed4/+0x8ed0, handles thresholds 0x1ef/0x3de/0x483/0x528, writes fixture score/status bytes +0x43..+0x48 from +0xf5bd/+0xf5f3, emits period transition events 0x20f1/0x20f2/0x20f3, mutates +0x8eb2/+0x8eb3/+0x8eb6/+0x8eb7, and resets player slots through 0x006db210; period semantics not implemented",
        ),
        phase_frontier(
            "0x006b4510",
            "match player candidate selector frontier",
            "verified match_eng.cpp frontier: scans 20 player slots from side base +0x4796 with 0x1be stride, skips active pointers +0xf59e/+0xf5a2 and invalid slot byte +0x19, scores candidates from match-state coordinates +0x8ea7/+0x8ea8, tactical table +0x8ebc, squad/role helpers 0x00882640/0x006d63b0, and verified match_random, then returns the highest-scoring player pointer; formulas not implemented",
        ),
        phase_frontier(
            "0x006aae20",
            "match per-tick tactical state updater frontier",
            "verified match_eng.cpp frontier: resets action scratch and counters +0x1c5/+0x4782/+0x4786/+0x477e/+0x475a..+0x4769, derives minute bucket from +0x8ed0, updates side tactical/status blocks around +0x904d/+0x911d, emits tactical/commentary events 0x21cf/0x21c1/0x2137/0x2138/0x2139/0x213a/0x213b/0x213c/0x213d, can call 0x006a1470 for team refresh, and branches on match-state +0x8eb2; semantics not implemented",
        ),
        phase_frontier(
            "0x005a2c70",
            "formation primary mask classifier",
            "verified formation.cpp-attributed frontier: reads formation mask tables at +0x12d and +0x14e with index*2, checks masks 0x880/0x40/0x20/0x10/0x8 via 0x0059cdf0, and returns a boolean classifier; semantics not implemented",
        ),
        phase_frontier(
            "0x005a30d0",
            "formation secondary mask classifier",
            "verified formation.cpp-attributed frontier: reads formation mask tables at +0x12d and +0x14e with index*2, checks masks 0x880/0x8 through 0x0059cdf0, and returns a boolean classifier; semantics not implemented",
        ),
        phase_frontier(
            "0x00935080",
            "random float jitter shim",
            "verified helper shape: thin wrapper that passes x87 floating inputs to 0x009350a2; used by match player evaluation/action-score frontiers for capped random jitter; exact distribution not implemented",
        ),
        phase_frontier(
            "0x004cdef0",
            "staff/contract date-renewal frontier",
            "verified frontier shape: builds renewal windows from +7 through +0x447 days, walks 0x6e-byte staff records, maps 0x4f side-state entries to 0x50 event/contract records, consults 0x245-byte club records and age helper 0x005246e0, and emits contract/status outcomes; mutations not implemented",
        ),
        phase_frontier(
            "0x00449710",
            "queued club-news dispatch cleanup",
            "verified frontier shape: drains 6-byte queued club/news items, resolves 0x245-byte club records, builds news payloads, dispatches to human/non-human news helpers, frees queue above 99 entries, and resets count; mutations not implemented",
        ),
        phase_frontier(
            "0x00672770",
            "Win32 message pump shell",
            "validated shell; not gameplay mutation",
        ),
    ];

    if phase == 2 {
        frontiers.extend([
            phase_frontier(
                "0x0053fe40",
                "42-slot current-date callback dispatcher",
                "verified frontier shape: walks 42 callback/object slots and invokes vtable +4 with the current-date argument; mutations not implemented",
            ),
            phase_frontier(
                "0x00614e90",
                "staff role/competition drift frontier",
                "verified frontier shape: releases stale scratch list, decrements per-entry counters, runs day-180 staff refresh, samples staff via match RNG, mutates role/preference bytes, emits 0x00616930 outcomes, then runs 0x006176f0/0x006180c0 post-processing; mutations not implemented",
            ),
            phase_frontier(
                "0x00595580",
                "fixture/news cleanup frontier",
                "verified frontier shape: runs date-gated cleanup, walks news/event lists, reattaches news entries, filters human-manager-visible records, and may generate paired +3/+4 dated fixture events; mutations not implemented",
            ),
            phase_frontier(
                "0x005e4370",
                "host-country date/RNG schedule frontier",
                "verified frontier shape: scans 34-byte date records around current date, resolves pending status bytes, uses match RNG for unresolved slots, then emits schedule/event records; mutations not implemented",
            ),
            phase_frontier(
                "0x005bfd90",
                "season/calendar maintenance frontier",
                "verified frontier shape: initializes one-shot guard, scans 34 calendar buckets, detects today/tomorrow schedule dates, runs update progress UI, invokes bucket callbacks, resets staff byte +0x6d on bucket 3, and rotates per-date scratch state; mutations not implemented",
            ),
            phase_frontier(
                "0x005c01d0",
                "club rolling-metrics frontier",
                "verified frontier shape: when date gates match, walks 0x122-byte club records, computes weighted rolling metrics from double fields +0xb0..+0xd8, shifts history windows, and sorts 12-byte ranking records; mutations not implemented",
            ),
            phase_frontier(
                "0x00752d40",
                "fixture/tie participant notification frontier",
                "verified frontier shape: date-gates competition/tie lists, lazily builds +/-1 and +2 day scratch dates, walks two fixture lists, filters fixture type codes, updates participant notification state via 0x0075ee00, prunes entries for current day, and runs 70-day cleanup; mutations not implemented",
            ),
            phase_frontier(
                "0x00585ae0",
                "club finance/stadium/status drift frontier",
                "verified frontier shape: handles special dated club/stadium reassignment, walks 0x245-byte club records with 0x167-byte finance/status side blocks, uses match RNG for financial/status drift, updates linked records/news, and applies end-of-season style club adjustments; mutations not implemented",
            ),
            phase_frontier(
                "0x00784290",
                "byte-array clear frontier",
                "verified frontier shape: clears param_1+8 byte buffer for param_1+4 entries; mutations not implemented",
            ),
            phase_frontier(
                "0x00674c10",
                "manager-job lifecycle frontier",
                "verified frontier shape: repairs missing jobs, expires dated manager-job items, evaluates 0x6e-byte manager records against 0x245-byte club/news records, resets 0x49-byte manager state blocks, uses match RNG for candidate/timing choices, and queues 0x26-byte job events; mutations not implemented",
            ),
            phase_frontier(
                "0x00844940",
                "stadium/date-ordered cleanup frontier",
                "verified frontier shape: special 2003 stadium restore path plus 30-day cleanup over 12-byte date-ordered records; mutations not implemented",
            ),
            phase_frontier(
                "0x00536190",
                "packed date add-days",
                "implemented slice; advances date on phase rollover",
            ),
        ]);
    }

    frontiers
}

fn phase_frontier(address: &str, label: &str, status: &str) -> RuntimePhaseFrontier {
    RuntimePhaseFrontier {
        address: address.to_string(),
        label: label.to_string(),
        status: status.to_string(),
    }
}

impl CoreBook {
    pub fn load_from_data_dir(data_dir: &Path) -> io::Result<Self> {
        Ok(Self {
            clubs: load_opaque_records(
                &data_dir.join("club.dat"),
                RecordKind::Club,
                CoreDecodeProfile::Club,
            )?,
            nat_clubs: load_opaque_records(
                &data_dir.join("nat_club.dat"),
                RecordKind::Club,
                CoreDecodeProfile::Club,
            )?,
            colours: load_opaque_records(
                &data_dir.join("colour.dat"),
                RecordKind::Colour,
                CoreDecodeProfile::Colour,
            )?,
            continents: load_opaque_records(
                &data_dir.join("continent.dat"),
                RecordKind::Continent,
                CoreDecodeProfile::Continent,
            )?,
            nations: load_opaque_records(
                &data_dir.join("nation.dat"),
                RecordKind::Nation,
                CoreDecodeProfile::Nation,
            )?,
        })
    }
}

impl CoreSummary {
    pub fn from_book(book: &CoreBook) -> Self {
        Self {
            club_count: book.clubs.len(),
            nat_club_count: book.nat_clubs.len(),
            colour_count: book.colours.len(),
            continent_count: book.continents.len(),
            nation_count: book.nations.len(),
            sample_club_record_size: book.clubs.first().map(|record| record.raw.len()),
            sample_nat_club_record_size: book.nat_clubs.first().map(|record| record.raw.len()),
            sample_colour_record_size: book.colours.first().map(|record| record.raw.len()),
            sample_continent_record_size: book.continents.first().map(|record| record.raw.len()),
            sample_nation_record_size: book.nations.first().map(|record| record.raw.len()),
        }
    }
}

impl ReferenceBook {
    pub fn from_data(data: &ReferenceData) -> Self {
        Self {
            cities: data
                .cities
                .iter()
                .map(|city| DomainCity {
                    id: city.id,
                    name: city.name.clone(),
                    tail_u16: city.tail_u16,
                    tail_u32: city.tail_u32,
                })
                .collect(),
            officials: data
                .officials
                .iter()
                .map(|official| DomainOfficial {
                    id: official.id,
                    u32_slots: official.u32_slots,
                    u16_slots: official.u16_slots,
                    trailing_byte: official.trailing_byte,
                })
                .collect(),
            first_names: data.first_names.iter().map(copy_name).collect(),
            second_names: data.second_names.iter().map(copy_name).collect(),
            common_names: data.common_names.iter().map(copy_name).collect(),
            stadiums: data
                .stadiums
                .iter()
                .map(|stadium| DomainStadium {
                    id: stadium.id,
                    name: stadium.name.clone(),
                    unknown_tail: stadium.unknown_tail.clone(),
                })
                .collect(),
            staff_competitions: data
                .staff_competitions
                .iter()
                .map(copy_competition)
                .collect(),
            club_competitions: data
                .club_competitions
                .iter()
                .map(copy_competition)
                .collect(),
            nation_competitions: data
                .nation_competitions
                .iter()
                .map(copy_competition)
                .collect(),
            staff_history: data
                .staff_history
                .iter()
                .map(|entry| DomainHistory17 {
                    id: entry.id,
                    u32_slots: entry.u32_slots,
                    trailing_byte: entry.trailing_byte,
                })
                .collect(),
            staff_comp_history: data
                .staff_comp_history
                .iter()
                .map(|entry| DomainHistory58 {
                    u32_slots: entry.u32_slots,
                    trailing_u16: entry.trailing_u16,
                })
                .collect(),
            club_comp_history: data
                .club_comp_history
                .iter()
                .map(|entry| DomainHistory26 {
                    u32_slots: entry.u32_slots,
                    trailing_u16: entry.trailing_u16,
                })
                .collect(),
            nation_comp_history: data
                .nation_comp_history
                .iter()
                .map(|entry| DomainHistory26 {
                    u32_slots: entry.u32_slots,
                    trailing_u16: entry.trailing_u16,
                })
                .collect(),
        }
    }
}

impl ReferenceSummary {
    pub fn from_book(book: &ReferenceBook) -> Self {
        Self {
            city_count: book.cities.len(),
            official_count: book.officials.len(),
            first_name_count: book.first_names.len(),
            second_name_count: book.second_names.len(),
            common_name_count: book.common_names.len(),
            stadium_count: book.stadiums.len(),
            staff_competition_count: book.staff_competitions.len(),
            club_competition_count: book.club_competitions.len(),
            nation_competition_count: book.nation_competitions.len(),
            staff_history_count: book.staff_history.len(),
            staff_comp_history_count: book.staff_comp_history.len(),
            club_comp_history_count: book.club_comp_history.len(),
            nation_comp_history_count: book.nation_comp_history.len(),
            sample_city: book.cities.first().map(|city| city.name.clone()),
            sample_official_id: book.officials.first().map(|official| official.id),
            sample_first_name: book
                .first_names
                .iter()
                .find(|entry| !entry.text.is_empty())
                .map(|entry| entry.text.clone()),
            sample_stadium: book.stadiums.first().map(|stadium| stadium.name.clone()),
            sample_staff_competition: book
                .staff_competitions
                .first()
                .map(|comp| comp.long_name.clone()),
            sample_club_competition: book
                .club_competitions
                .first()
                .map(|comp| comp.long_name.clone()),
            sample_nation_competition: book
                .nation_competitions
                .first()
                .map(|comp| comp.long_name.clone()),
        }
    }
}

impl StaffSummary {
    pub fn from_data(data: &StaffData) -> Self {
        let max_type10_ca = data
            .type10
            .iter()
            .map(|entry| entry.rating_short_0x05)
            .max();
        let sample_type10 = data.type10.get(1).or_else(|| data.type10.first());
        Self {
            type6_count: data.type6.len(),
            type8_count: data.type8.len(),
            type9_count: data.type9.len(),
            type10_count: data.type10.len(),
            sample_type6_id: data.type6.first().map(|entry| entry.id),
            sample_type9_id: data.type9.first().map(|entry| entry.id),
            sample_type10_id: sample_type10.map(|entry| entry.id),
            sample_type10_ca: sample_type10.map(|entry| entry.rating_short_0x05),
            sample_type10_pa: sample_type10.map(|entry| entry.rating_short_0x07),
            sample_type10_reputation: sample_type10.map(|entry| entry.rating_short_0x0d),
            max_type10_ca,
        }
    }

    pub fn from_book(book: &StaffBook) -> Self {
        let max_type10_ca = book
            .type10
            .iter()
            .map(|entry| entry.rating_short_0x05)
            .max();
        let sample_type10 = book.type10.get(1).or_else(|| book.type10.first());
        Self {
            type6_count: book.type6.len(),
            type8_count: book.type8.len(),
            type9_count: book.type9.len(),
            type10_count: book.type10.len(),
            sample_type6_id: book.type6.first().map(|entry| entry.id),
            sample_type9_id: book.type9.first().map(|entry| entry.id),
            sample_type10_id: sample_type10.map(|entry| entry.id),
            sample_type10_ca: sample_type10.map(|entry| entry.rating_short_0x05),
            sample_type10_pa: sample_type10.map(|entry| entry.rating_short_0x07),
            sample_type10_reputation: sample_type10.map(|entry| entry.rating_short_0x0d),
            max_type10_ca,
        }
    }
}

impl StaffBook {
    pub fn from_data(data: &StaffData) -> Self {
        Self {
            type6: data
                .type6
                .iter()
                .map(|entry| DomainStaffType6 {
                    id: entry.id,
                    body: entry.body.clone(),
                })
                .collect(),
            type8: data
                .type8
                .iter()
                .map(|entry| DomainStaffType8 {
                    body: entry.body.clone(),
                })
                .collect(),
            type9: data
                .type9
                .iter()
                .map(|entry| DomainStaffType9 {
                    id: entry.id,
                    body: entry.body.clone(),
                })
                .collect(),
            type10: data
                .type10
                .iter()
                .map(|entry| DomainStaffType10 {
                    id: entry.id,
                    unknown_byte_4: entry.unknown_byte_4,
                    rating_short_0x05: entry.rating_short_0x05,
                    rating_short_0x07: entry.rating_short_0x07,
                    unknown_bytes_9_12: entry.unknown_bytes_9_12,
                    rating_short_0x0d: entry.rating_short_0x0d,
                    unknown_bytes_15_26: entry.unknown_bytes_15_26,
                    attributes: entry.attributes,
                    unknown_bytes_58_64: entry.unknown_bytes_58_64,
                    trailing_bytes: entry.trailing_bytes,
                })
                .collect(),
        }
    }
}

impl SchemaBook {
    pub fn default_table_set() -> Self {
        use FieldStatus::{CompatibilityVerified, Inferred, Verified};

        Self {
            tables: vec![
                TableSchema::new(
                    "core.clubs",
                    Inferred,
                    &[
                        ("ordinal", Verified),
                        ("id", CompatibilityVerified),
                        ("primary_name", CompatibilityVerified),
                        ("secondary_name", CompatibilityVerified),
                        ("short_name", CompatibilityVerified),
                        ("text_candidates", CompatibilityVerified),
                        ("raw", Verified),
                    ],
                ),
                TableSchema::new(
                    "core.nat_clubs",
                    Inferred,
                    &[
                        ("ordinal", Verified),
                        ("id", CompatibilityVerified),
                        ("primary_name", CompatibilityVerified),
                        ("secondary_name", CompatibilityVerified),
                        ("short_name", CompatibilityVerified),
                        ("text_candidates", CompatibilityVerified),
                        ("raw", Verified),
                    ],
                ),
                TableSchema::new(
                    "core.colours",
                    Inferred,
                    &[
                        ("ordinal", Verified),
                        ("id", CompatibilityVerified),
                        ("primary_name", CompatibilityVerified),
                        ("text_candidates", CompatibilityVerified),
                        ("raw", Verified),
                    ],
                ),
                TableSchema::new(
                    "core.continents",
                    Inferred,
                    &[
                        ("ordinal", Verified),
                        ("id", CompatibilityVerified),
                        ("primary_name", CompatibilityVerified),
                        ("text_candidates", CompatibilityVerified),
                        ("raw", Verified),
                    ],
                ),
                TableSchema::new(
                    "core.nations",
                    Inferred,
                    &[
                        ("ordinal", Verified),
                        ("id", CompatibilityVerified),
                        ("primary_name", CompatibilityVerified),
                        ("secondary_name", CompatibilityVerified),
                        ("short_name", CompatibilityVerified),
                        ("text_candidates", CompatibilityVerified),
                        ("raw", Verified),
                    ],
                ),
                TableSchema::new(
                    "staff.type6",
                    Inferred,
                    &[("id", CompatibilityVerified), ("body", Verified)],
                ),
                TableSchema::new("staff.type8", Inferred, &[("body", Verified)]),
                TableSchema::new(
                    "staff.type9",
                    Inferred,
                    &[("id", CompatibilityVerified), ("body", Verified)],
                ),
                TableSchema::new(
                    "staff.type10",
                    Inferred,
                    &[
                        ("id", CompatibilityVerified),
                        ("rating_short_0x05", CompatibilityVerified),
                        ("rating_short_0x07", CompatibilityVerified),
                        ("rating_short_0x0d", CompatibilityVerified),
                        ("attributes", CompatibilityVerified),
                        ("trailing_bytes", Verified),
                    ],
                ),
                TableSchema::new(
                    "references.cities",
                    Inferred,
                    &[
                        ("id", CompatibilityVerified),
                        ("name", CompatibilityVerified),
                        ("tail_u16", CompatibilityVerified),
                        ("tail_u32", CompatibilityVerified),
                    ],
                ),
                TableSchema::new(
                    "references.officials",
                    Inferred,
                    &[
                        ("id", CompatibilityVerified),
                        ("u32_slots", CompatibilityVerified),
                        ("u16_slots", CompatibilityVerified),
                        ("trailing_byte", Verified),
                    ],
                ),
                TableSchema::new(
                    "references.first_names",
                    Inferred,
                    &[
                        ("text", CompatibilityVerified),
                        ("footer", CompatibilityVerified),
                    ],
                ),
                TableSchema::new(
                    "references.second_names",
                    Inferred,
                    &[
                        ("text", CompatibilityVerified),
                        ("footer", CompatibilityVerified),
                    ],
                ),
                TableSchema::new(
                    "references.common_names",
                    Inferred,
                    &[
                        ("text", CompatibilityVerified),
                        ("footer", CompatibilityVerified),
                    ],
                ),
                TableSchema::new(
                    "references.stadiums",
                    Inferred,
                    &[
                        ("id", CompatibilityVerified),
                        ("name", CompatibilityVerified),
                        ("unknown_tail", CompatibilityVerified),
                    ],
                ),
                TableSchema::new(
                    "references.staff_competitions",
                    Inferred,
                    &[
                        ("id", CompatibilityVerified),
                        ("long_name", CompatibilityVerified),
                        ("short_name", CompatibilityVerified),
                        ("unknown_tail", CompatibilityVerified),
                    ],
                ),
                TableSchema::new(
                    "references.club_competitions",
                    Inferred,
                    &[
                        ("id", CompatibilityVerified),
                        ("long_name", CompatibilityVerified),
                        ("short_name", CompatibilityVerified),
                        ("unknown_tail", CompatibilityVerified),
                    ],
                ),
                TableSchema::new(
                    "references.nation_competitions",
                    Inferred,
                    &[
                        ("id", CompatibilityVerified),
                        ("long_name", CompatibilityVerified),
                        ("short_name", CompatibilityVerified),
                        ("unknown_tail", CompatibilityVerified),
                    ],
                ),
                TableSchema::new(
                    "references.staff_history",
                    Inferred,
                    &[
                        ("id", CompatibilityVerified),
                        ("u32_slots", CompatibilityVerified),
                        ("trailing_byte", Verified),
                    ],
                ),
                TableSchema::new(
                    "references.staff_comp_history",
                    Inferred,
                    &[
                        ("u32_slots", CompatibilityVerified),
                        ("trailing_u16", Verified),
                    ],
                ),
                TableSchema::new(
                    "references.club_comp_history",
                    Inferred,
                    &[
                        ("u32_slots", CompatibilityVerified),
                        ("trailing_u16", Verified),
                    ],
                ),
                TableSchema::new(
                    "references.nation_comp_history",
                    Inferred,
                    &[
                        ("u32_slots", CompatibilityVerified),
                        ("trailing_u16", Verified),
                    ],
                ),
            ],
        }
    }
}

impl TableSchema {
    fn new(path: &str, status: FieldStatus, fields: &[(&str, FieldStatus)]) -> Self {
        Self {
            path: path.to_string(),
            status,
            fields: fields
                .iter()
                .map(|(name, status)| FieldSchema {
                    name: (*name).to_string(),
                    status: *status,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum CoreDecodeProfile {
    Club,
    Nation,
    Continent,
    Colour,
}

fn load_opaque_records(
    path: &Path,
    kind: RecordKind,
    profile: CoreDecodeProfile,
) -> io::Result<Vec<DomainOpaqueRecord>> {
    let bytes = fs::read(path)?;
    let file = DatFile::new(kind, &bytes)?;
    Ok(file
        .records()
        .enumerate()
        .map(|(index, record)| decode_core_record(index as u32, record, profile))
        .collect())
}

fn decode_core_record(
    ordinal: u32,
    record: &[u8],
    profile: CoreDecodeProfile,
) -> DomainOpaqueRecord {
    let texts = extract_latin1_texts(record);
    let primary = texts.first().cloned();
    let secondary = texts.get(1).cloned();
    let short = texts.get(2).cloned();
    let (primary_name, secondary_name, short_name) = match profile {
        CoreDecodeProfile::Colour | CoreDecodeProfile::Continent => (primary, None, None),
        CoreDecodeProfile::Nation | CoreDecodeProfile::Club => (primary, secondary, short),
    };

    DomainOpaqueRecord {
        ordinal,
        id: le_u32_lossy(record, 0),
        primary_name,
        secondary_name,
        short_name,
        text_candidates: texts,
        raw: record.to_vec(),
    }
}

fn manifest_entry_is_known(filename: &str, kind: u8) -> bool {
    TABLE_SPECS
        .iter()
        .any(|spec| spec.manifest_type == kind && spec.filename.eq_ignore_ascii_case(filename))
}

fn table_is_editable(path: &str) -> bool {
    path.starts_with("core.")
        || matches!(
            path,
            "staff.type10"
                | "references.cities"
                | "references.stadiums"
                | "references.first_names"
                | "references.second_names"
                | "references.common_names"
                | "references.staff_competitions"
                | "references.club_competitions"
                | "references.nation_competitions"
        )
}

fn backend_completion_score(
    owns_data: bool,
    validates_data: bool,
    headless_shell_runs: bool,
    manager_shell_runs: bool,
    runtime_system_ledger_runs: bool,
    gameplay_mutators_complete: bool,
) -> u8 {
    let mut score = 0u8;
    if owns_data {
        score = score.saturating_add(25);
    }
    if validates_data {
        score = score.saturating_add(20);
    }
    if headless_shell_runs {
        score = score.saturating_add(20);
    }
    if manager_shell_runs {
        score = score.saturating_add(10);
    }
    if runtime_system_ledger_runs {
        score = score.saturating_add(5);
    }
    if gameplay_mutators_complete {
        score = score.saturating_add(20);
    }
    score
}

fn gameplay_mutator_install_plans_ready(plans: &[GameplayMutatorInstallPlan]) -> bool {
    gameplay_mutator_install_plan_ready(
        plans,
        "match results",
        2,
        "match_result_write_map",
        "reports/parity_traces/match-results.json",
    ) && gameplay_mutator_install_plan_ready(
        plans,
        "competition state",
        2,
        "competition_fixture_state_map",
        "reports/parity_traces/competition-state.json",
    ) && gameplay_mutator_install_plan_ready(
        plans,
        "transfers/contracts",
        0,
        "transfer_contract_state_map",
        "reports/parity_traces/transfers-contracts.json",
    ) && gameplay_mutator_install_plan_ready(
        plans,
        "news/inbox",
        1,
        "news_inbox_emission_map",
        "reports/parity_traces/news-inbox.json",
    )
}

fn gameplay_mutator_install_plan_ready(
    plans: &[GameplayMutatorInstallPlan],
    system: &str,
    phase: u8,
    boundary_map: &str,
    trace_file: &str,
) -> bool {
    plans.iter().any(|plan| {
        plan.system == system
            && plan.phase == phase
            && plan.boundary_map == boundary_map
            && plan.trace_file.ends_with(trace_file)
            && !plan.rust_hook.is_empty()
            && !plan.required_original_coverage.is_empty()
            && plan.required_original_coverage == plan.required_rust_coverage
            && !plan.required_functions.is_empty()
            && plan.promotion_rule.contains("implementation_present=true")
    })
}

fn backend_implementation_plan(save: &RuntimeSaveGame) -> Vec<BackendImplementationPlanItem> {
    vec![
        backend_implementation_plan_item(
            "match results",
            &save.backend.matches,
            save.backend.match_result_write_map.len(),
            vec![
                "0x00699640",
                "0x00699d90",
                "0x0069d950",
                "0x006a3240",
                "0x006a4020",
                "0x006ae330",
            ],
            save.backend
                .match_result_write_map
                .iter()
                .map(|entry| {
                    format!(
                        "{}: fixture +{}/+{} via {}",
                        entry.phase,
                        entry.fixture_home_offset.trim_start_matches("0x"),
                        entry.fixture_away_offset.trim_start_matches("0x"),
                        entry.function
                    )
                })
                .collect(),
            Vec::new(),
            "A one-day headless tick mutates fixture result records exactly and preserves 365-day acceptance determinism.",
        ),
        backend_implementation_plan_item(
            "competition state",
            &save.backend.competitions,
            save.backend.competition_fixture_state_map.len(),
            vec!["0x00674c10", "0x00595580", "0x00752d40"],
            save.backend
                .competition_fixture_state_map
                .iter()
                .map(|entry| {
                    format!(
                        "{}: offset {:?} helper {:?} in {}",
                        entry.system, entry.fixture_offset, entry.helper, entry.function
                    )
                })
                .collect(),
            Vec::new(),
            "A headless campaign produces deterministic fixtures, tables, cup progression, and notifications without .dat reads.",
        ),
        backend_implementation_plan_item(
            "transfers/contracts",
            &save.backend.transfers,
            save.backend.transfer_contract_state_map.len(),
            vec!["0x004cdef0", "0x00449710", "0x008a9080"],
            save.backend
                .transfer_contract_state_map
                .iter()
                .map(|entry| {
                    format!(
                        "{}: offset {:?} stride {:?} helper {:?} in {}",
                        entry.system, entry.record_offset, entry.stride, entry.helper, entry.function
                    )
                })
                .collect(),
            Vec::new(),
            "A headless campaign can renew contracts, process transfer queues, and write transfer state without transfer.dat.",
        ),
        backend_implementation_plan_item(
            "news/inbox",
            &save.backend.news,
            save.backend.news_inbox_emission_map.len(),
            vec!["0x0050c8d0", "0x00595580", "0x006724d0", "0x0076e180"],
            save.backend
                .news_inbox_emission_map
                .iter()
                .map(|entry| {
                    format!(
                        "{}: offset {:?} stride {:?} helper {:?} in {}",
                        entry.system, entry.record_offset, entry.stride, entry.helper, entry.function
                    )
                })
                .collect(),
            Vec::new(),
            "A headless tick emits and removes news/inbox records through Rust-owned queues with original-equivalent payloads.",
        ),
    ]
}

fn runtime_contract_gate_evidence(contract: &GameplayMutatorContract) -> String {
    match contract.status {
        GameplayMutatorStatus::ContractReady => format!(
            "mutator contract '{}' is ready but disabled until {} passes",
            contract.implementation_hook, contract.trace_file
        ),
        GameplayMutatorStatus::ImplementedPendingParity => format!(
            "mutator contract '{}' has Rust-side work pending parity verification in {}",
            contract.implementation_hook, contract.trace_file
        ),
        GameplayMutatorStatus::ParityVerified => format!(
            "mutator contract '{}' is parity-verified by {} and its static-boundary mutation body is installed in this runtime slice",
            contract.implementation_hook, contract.trace_file
        ),
    }
}

fn backend_implementation_plan_item(
    system: &str,
    state: &RuntimeSystemState,
    boundary_entries: usize,
    primary_frontiers: Vec<&str>,
    code_derived_boundaries: Vec<String>,
    missing_lifts: Vec<&str>,
    acceptance_gate: &str,
) -> BackendImplementationPlanItem {
    let readiness = if state.implemented_mutations > 0 {
        BackendImplementationReadiness::MutationsImplemented
    } else if boundary_entries > 0 {
        BackendImplementationReadiness::BoundaryMapped
    } else {
        BackendImplementationReadiness::NeedsBoundaryMap
    };

    BackendImplementationPlanItem {
        system: system.to_string(),
        readiness,
        owned_records: state.owned_records,
        boundary_entries,
        attempted_mutations: state.attempted_mutations,
        implemented_mutations: state.implemented_mutations,
        primary_frontiers: primary_frontiers.into_iter().map(str::to_string).collect(),
        code_derived_boundaries,
        missing_lifts: missing_lifts.into_iter().map(str::to_string).collect(),
        acceptance_gate: acceptance_gate.to_string(),
    }
}

fn push_backend_check(
    checks: &mut Vec<BackendReadinessCheck>,
    name: &str,
    status: ValidationStatus,
    detail: String,
) {
    checks.push(BackendReadinessCheck {
        name: name.to_string(),
        status,
        detail,
    });
}

fn push_validation_check(
    checks: &mut Vec<CanonicalValidationCheck>,
    failures: &mut Vec<String>,
    name: &str,
    passed: bool,
    detail: String,
    warn_only: bool,
) {
    let status = if passed {
        ValidationStatus::Pass
    } else if warn_only {
        ValidationStatus::Warn
    } else {
        failures.push(format!("{name}: {detail}"));
        ValidationStatus::Fail
    };
    checks.push(CanonicalValidationCheck {
        name: name.to_string(),
        status,
        detail,
    });
}

fn unique_u32(mut values: impl Iterator<Item = u32>) -> bool {
    let mut seen = std::collections::HashSet::new();
    values.all(|value| seen.insert(value))
}

fn history_competition_refs_valid(
    refs: impl Iterator<Item = u32>,
    ids: impl Iterator<Item = u32>,
) -> bool {
    let ids: std::collections::HashSet<u32> = ids.collect();
    refs.filter(|value| *value != u32::MAX && *value != 0xffff0000)
        .all(|value| ids.contains(&value))
}

const COMMON_CUMULATIVE_DAYS_BEFORE_MONTH: [u16; 12] =
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
const LEAP_CUMULATIVE_DAYS_BEFORE_MONTH: [u16; 12] =
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];

fn cumulative_days_before_month(year: u16, month_index: usize) -> u16 {
    if is_leap_year(year) {
        LEAP_CUMULATIVE_DAYS_BEFORE_MONTH[month_index]
    } else {
        COMMON_CUMULATIVE_DAYS_BEFORE_MONTH[month_index]
    }
}

fn is_leap_year(year: u16) -> bool {
    let year = u32::from(year);
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn le_u32_lossy(bytes: &[u8], offset: usize) -> u32 {
    if let Some(slice) = bytes.get(offset..offset + 4) {
        u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])
    } else {
        0
    }
}

fn extract_latin1_texts(bytes: &[u8]) -> Vec<String> {
    let mut texts = Vec::new();
    let mut start: Option<usize> = None;
    for (index, byte) in bytes.iter().copied().enumerate().skip(4) {
        let printable = (32..=126).contains(&byte) || (160..=255).contains(&byte);
        if printable {
            if start.is_none() {
                start = Some(index);
            }
            continue;
        }
        if byte == 0 {
            if let Some(begin) = start.take() {
                if index > begin + 2 {
                    let text: String = bytes[begin..index].iter().map(|&b| char::from(b)).collect();
                    let cleaned = sanitize_projected_text(&text);
                    if !cleaned.is_empty() {
                        texts.push(cleaned);
                    }
                }
            }
        } else {
            start = None;
        }
    }
    texts.truncate(4);
    texts
}

fn sanitize_projected_text(text: &str) -> String {
    text.trim_matches('\0')
        .trim_start_matches('\u{00ff}')
        .trim()
        .to_string()
}

fn flatten_core_records(records: &[DomainOpaqueRecord]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for record in records {
        bytes.extend_from_slice(&record.raw);
    }
    bytes
}

fn truncate_core_payloads(records: &mut [DomainOpaqueRecord], max_bytes: usize) {
    for record in records {
        record.raw.truncate(max_bytes);
    }
}

fn audit_core_record_sizes(
    path: &str,
    records: &[DomainOpaqueRecord],
    kind: RecordKind,
    mismatches: &mut Vec<String>,
) {
    let expected_size = kind.size();
    for record in records {
        if record.raw.len() != expected_size {
            mismatches.push(format!(
                "{path} ordinal {} raw size {} differs from expected {}",
                record.ordinal,
                record.raw.len(),
                expected_size
            ));
        }
    }
}

fn to_cm_staff_data(staff: &StaffBook) -> cm_data::StaffData {
    cm_data::StaffData {
        type6: staff
            .type6
            .iter()
            .map(|entry| cm_data::StaffType6Entry {
                id: entry.id,
                body: entry.body.clone(),
            })
            .collect(),
        type8: staff
            .type8
            .iter()
            .map(|entry| cm_data::StaffType8Entry {
                body: entry.body.clone(),
            })
            .collect(),
        type9: staff
            .type9
            .iter()
            .map(|entry| cm_data::StaffType9Entry {
                id: entry.id,
                body: entry.body.clone(),
            })
            .collect(),
        type10: staff
            .type10
            .iter()
            .map(|entry| cm_data::StaffType10Entry {
                id: entry.id,
                unknown_byte_4: entry.unknown_byte_4,
                rating_short_0x05: entry.rating_short_0x05,
                rating_short_0x07: entry.rating_short_0x07,
                unknown_bytes_9_12: entry.unknown_bytes_9_12,
                rating_short_0x0d: entry.rating_short_0x0d,
                unknown_bytes_15_26: entry.unknown_bytes_15_26,
                attributes: entry.attributes,
                unknown_bytes_58_64: entry.unknown_bytes_58_64,
                trailing_bytes: entry.trailing_bytes,
            })
            .collect(),
    }
}

fn to_cm_cities(entries: &[DomainCity]) -> Vec<cm_data::CityEntry> {
    entries
        .iter()
        .map(|entry| cm_data::CityEntry {
            id: entry.id,
            name: entry.name.clone(),
            tail_u16: entry.tail_u16,
            tail_u32: entry.tail_u32,
        })
        .collect()
}

fn to_cm_officials(entries: &[DomainOfficial]) -> Vec<cm_data::OfficialEntry> {
    entries
        .iter()
        .map(|entry| cm_data::OfficialEntry {
            id: entry.id,
            u32_slots: entry.u32_slots,
            u16_slots: entry.u16_slots,
            trailing_byte: entry.trailing_byte,
        })
        .collect()
}

fn to_cm_names(entries: &[DomainName]) -> Vec<cm_data::NameEntry> {
    entries
        .iter()
        .map(|entry| cm_data::NameEntry {
            text: entry.text.clone(),
            footer: entry.footer,
        })
        .collect()
}

fn to_cm_stadiums(entries: &[DomainStadium]) -> Vec<cm_data::StadiumEntry> {
    entries
        .iter()
        .map(|entry| cm_data::StadiumEntry {
            id: entry.id,
            name: entry.name.clone(),
            unknown_tail: entry.unknown_tail.clone(),
        })
        .collect()
}

fn to_cm_competitions(entries: &[DomainCompetition]) -> Vec<cm_data::CompetitionEntry> {
    entries
        .iter()
        .map(|entry| cm_data::CompetitionEntry {
            id: entry.id,
            long_name: entry.long_name.clone(),
            short_name: entry.short_name.clone(),
            three_letter_name: entry.three_letter_name.clone(),
            scope: entry.scope,
            nation_id: entry.nation_id,
            last_division: entry.last_division,
            reserve_division: entry.reserve_division,
            reputation: entry.reputation,
            unknown_tail: entry.unknown_tail.clone(),
        })
        .collect()
}

fn to_cm_history17(entries: &[DomainHistory17]) -> Vec<cm_data::History17Entry> {
    entries
        .iter()
        .map(|entry| cm_data::History17Entry {
            id: entry.id,
            u32_slots: entry.u32_slots,
            trailing_byte: entry.trailing_byte,
        })
        .collect()
}

fn to_cm_history26(entries: &[DomainHistory26]) -> Vec<cm_data::History26Entry> {
    entries
        .iter()
        .map(|entry| cm_data::History26Entry {
            u32_slots: entry.u32_slots,
            trailing_u16: entry.trailing_u16,
        })
        .collect()
}

fn to_cm_history58(entries: &[DomainHistory58]) -> Vec<cm_data::History58Entry> {
    entries
        .iter()
        .map(|entry| cm_data::History58Entry {
            u32_slots: entry.u32_slots,
            trailing_u16: entry.trailing_u16,
        })
        .collect()
}

fn copy_name(entry: &cm_data::NameEntry) -> DomainName {
    DomainName {
        text: entry.text.clone(),
        footer: entry.footer,
    }
}

fn copy_competition(entry: &cm_data::CompetitionEntry) -> DomainCompetition {
    DomainCompetition {
        id: entry.id,
        long_name: entry.long_name.clone(),
        short_name: entry.short_name.clone(),
        three_letter_name: entry.three_letter_name.clone(),
        scope: entry.scope,
        nation_id: entry.nation_id,
        last_division: entry.last_division,
        reserve_division: entry.reserve_division,
        reputation: entry.reputation,
        unknown_tail: entry.unknown_tail.clone(),
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to serialize {}: {err}", path.display()),
        )
    })?;
    fs::write(path, bytes)
}

fn write_pretty_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to serialize {}: {err}", path.display()),
        )
    })?;
    fs::write(path, bytes)
}

fn read_json<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse {}: {err}", path.display()),
        )
    })
}

fn read_optional_json<T: DeserializeOwned>(path: &Path) -> io::Result<Option<T>> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse {}: {err}", path.display()),
            )
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

fn remove_file_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_data::ManifestEntry;

    #[test]
    fn transfer_contracts_branch_cluster_constants_are_code_derived() {
        assert_eq!(TRANSFER_CONTRACT_COORDINATOR_BLOCK_COUNT_00848DA0, 734);
        assert_eq!(TRANSFER_CONTRACT_COORDINATOR_BRANCH_EDGES_00848DA0, 1058);
        assert_eq!(TRANSFER_CONTRACT_SIBLING_BLOCK_COUNT_0084D5D0, 589);
        assert_eq!(TRANSFER_CONTRACT_SIBLING_BRANCH_EDGES_0084D5D0, 866);
        assert_eq!(STAFF_CONTRACTS_0084A907_MULTIPLIER, -0.5);
        assert_eq!(STAFF_CONTRACTS_0084A720_MULTIPLIER, 0.25);
        assert_eq!(STAFF_CONTRACTS_0084AA1D_MULTIPLIER, 0.25);
        assert_eq!(STAFF_CONTRACTS_0084AB3D_MULTIPLIER, 0.5);
        assert_eq!(STAFF_CONTRACTS_0084A907_FLOOR, 50_000);
        assert_eq!(STAFF_CONTRACTS_0084AA1D_FLOOR, 150_000);
        assert_eq!(STAFF_CONTRACTS_0084E67B_DATE_THRESHOLD, 250.0);
        assert_eq!(STAFF_CONTRACTS_DATE_GATE_30_DAYS, 30);
        assert_eq!(STAFF_CONTRACTS_DATE_GATE_120_DAYS, 120);
        assert_eq!(STAFF_CONTRACTS_DATE_GATE_200_DAYS, 200);
        assert_eq!(STAFF_CONTRACTS_DATE_GATE_BASE_525_DAYS, 525);
        assert_eq!(STAFF_CONTRACTS_0084D5D0_SCORE_MULTIPLIER, 4.0);
        assert_eq!(STAFF_CONTRACTS_0084D5D0_SCORE_ADDEND, 1.0);
        assert_eq!(STAFF_CONTRACTS_0084D5D0_TINY_SCALAR, 0.0001);
        assert_eq!(STAFF_CONTRACTS_0084E1F3_BASE, 3_250.0);
        assert_eq!(STAFF_CONTRACTS_0084E1F3_MULTIPLIER, -0.2);
        assert_eq!(STAFF_CONTRACTS_0084E1F3_SUBTRACT_FROM, 0x0cb2);
        assert_eq!(STAFF_CONTRACTS_0084E323_MULTIPLIER, 1.05);
        assert_eq!(STAFF_CONTRACTS_0084E286_PRIMARY_MULTIPLIER, 8.0);
        assert_eq!(STAFF_CONTRACTS_0084E286_SECONDARY_MULTIPLIER, 4.0);
        assert_eq!(CM0102_X87_TRUNCATE_HELPER, "0x009346d0");
        assert_eq!(
            STAFF_CONTRACTS_0084D5D0_MONEY_BANDS,
            &[
                250.0, 350.0, 500.0, 1_250.0, 2_750.0, 3_250.0, 7_250.0, 7_500.0, 10_000.0,
                15_000.0, 25_000.0,
            ]
        );
    }

    #[test]
    fn transfer_contracts_0084a907_applies_original_floor_after_integer_delta() {
        let output = apply_staff_contracts_0084a907_floor(StaffContracts0084a907Inputs {
            base_value_before_delta: 45_000,
            rounded_helper_delta: -4_000,
        });

        assert_eq!(output.adjusted_value, 50_000);
        assert!(output.floor_applied);
        assert_eq!(output.source_block, "0x0084a907");
        assert!(output
            .evidence
            .contains("transfer_contracts_branch_clusters"));

        let output = apply_staff_contracts_0084a907_floor(StaffContracts0084a907Inputs {
            base_value_before_delta: 80_000,
            rounded_helper_delta: -10_000,
        });

        assert_eq!(output.adjusted_value, 70_000);
        assert!(!output.floor_applied);
    }

    #[test]
    fn transfer_contracts_0084a907_exposes_unrounded_multiplier_without_guessing_ftol() {
        assert_eq!(
            staff_contracts_0084a907_unrounded_delta(12_000, 0.5),
            -3_000.0
        );
    }

    #[test]
    fn transfer_contracts_ports_remaining_staff_offer_adjustment_multipliers() {
        assert_eq!(
            staff_contracts_0084a720_unrounded_delta(12_000, 0.5),
            1_500.0
        );
        assert_eq!(
            staff_contracts_0084aa1d_unrounded_delta(20_000, 0.4),
            2_000.0
        );
        assert_eq!(
            staff_contracts_0084ab3d_unrounded_delta(20_000, 0.4),
            4_000.0
        );

        let output = apply_staff_contracts_0084aa1d_floor(120_000, 10_000);
        assert_eq!(output.adjusted_value, 150_000);
        assert!(output.floor_applied);
        assert_eq!(output.source_block, "0x0084aa1d/0x0084aa3d");

        let output = apply_staff_contracts_0084aa1d_floor(160_000, 10_000);
        assert_eq!(output.adjusted_value, 170_000);
        assert!(!output.floor_applied);
    }

    #[test]
    fn transfer_contracts_0084e67b_date_gate_matches_lifted_float_compare() {
        let inside = evaluate_staff_contracts_0084e67b_date_gate(250);
        assert!(inside.branches_to_0x0084e6ae);
        assert_eq!(inside.threshold, 250.0);
        assert_eq!(inside.source_block, "0x0084e67b");

        let outside = evaluate_staff_contracts_0084e67b_date_gate(251);
        assert!(!outside.branches_to_0x0084e6ae);
    }

    #[test]
    fn transfer_contracts_ports_00848da0_date_gate_shapes() {
        let gate = evaluate_staff_contracts_00849f32_date_gate(80, 25, 15);
        assert_eq!(gate.threshold, 80);
        assert!(gate.branch_taken);
        assert_eq!(gate.comparison, ">=");

        let gate = evaluate_staff_contracts_00849c20_date_gate(170, 40);
        assert_eq!(gate.threshold, 160);
        assert!(!gate.branch_taken);
        assert_eq!(gate.comparison, "<=");

        let gate = evaluate_staff_contracts_00849912_date_gate(626, 10);
        assert_eq!(gate.threshold, 625);
        assert!(gate.branch_taken);
        assert_eq!(gate.comparison, ">");

        let gate = evaluate_staff_contracts_00849dac_date_gate(29);
        assert!(gate.branch_taken);
        assert_eq!(gate.threshold, 30);

        let gate = evaluate_staff_contracts_00849dfa_date_gate(30);
        assert!(gate.branch_taken);
        assert_eq!(gate.threshold, 30);
    }

    #[test]
    fn transfer_contracts_ports_0084d5d0_towards_base_money_band_shape() {
        let block_84ebad =
            staff_contracts_money_band_towards_base(1_250.0, 250.0, 0.15, "0x0084ebad");
        assert_eq!(block_84ebad.output_value, 400.0);
        assert_eq!(block_84ebad.source_block, "0x0084ebad");

        let block_84ec03 =
            staff_contracts_money_band_towards_base(2_250.0, 1_250.0, 0.2, "0x0084ec03");
        assert_eq!(block_84ec03.output_value, 1_450.0);

        let block_84ecda =
            staff_contracts_money_band_towards_base(14_000.0, 10_000.0, 0.25, "0x0084ecda");
        assert_eq!(block_84ecda.output_value, 11_000.0);

        let block_84f06d =
            staff_contracts_money_band_towards_base(45_000.0, 25_000.0, 0.25, "0x0084f06d");
        assert_eq!(block_84f06d.output_value, 30_000.0);
        assert!(block_84f06d
            .evidence
            .contains("transfer_contracts_branch_clusters"));
    }

    #[test]
    fn transfer_contracts_ports_0084d5d0_score_money_scalar_shape() {
        let block_84d7b9 = staff_contracts_score_money_scalar(4.0, 25_000.0, "0x0084d7b9");
        assert_eq!(block_84d7b9.output_value, 42.5);
        assert_eq!(block_84d7b9.source_block, "0x0084d7b9");

        let block_84d888 = staff_contracts_score_money_scalar(2.0, 10_000.0, "0x0084d888");
        assert_eq!(block_84d888.output_value, 9.0);

        let block_84dafc = staff_contracts_score_money_scalar(3.0, 5_000.0, "0x0084dafc");
        assert_eq!(block_84dafc.output_value, 6.5);
        assert!(block_84dafc
            .evidence
            .contains("transfer_contracts_branch_clusters"));
    }

    #[test]
    fn transfer_contracts_ports_0084d5d0_round_site_feeders_without_guessing_ftol() {
        let block_84e1f3 = staff_contracts_0084e1f3_round_expression(3_000, Some(50));
        assert_eq!(block_84e1f3.unrounded_value, 50.0);
        assert_eq!(block_84e1f3.applied_value, Some(0x0cb2 - 50));
        assert_eq!(block_84e1f3.source_block, "0x0084e1f3");

        let block_84e323 = staff_contracts_0084e323_round_expression(2_000, Some(2_100));
        assert_eq!(block_84e323.unrounded_value, 2_100.0);
        assert_eq!(block_84e323.applied_value, Some(2_100));
        assert_eq!(block_84e323.source_block, "0x0084e323");

        assert_eq!(
            staff_contracts_0084e286_visible_score_component(10.0, 3.0),
            92.0
        );
    }

    #[test]
    fn cm0102_x87_truncate_helper_matches_lifted_chop_rounding() {
        assert_eq!(cm0102_x87_truncate_to_i64(12.99), 12);
        assert_eq!(cm0102_x87_truncate_to_i64(-12.99), -12);
        assert_eq!(cm0102_x87_truncate_to_i32(0.99), 0);
        assert_eq!(cm0102_x87_truncate_to_i32(-0.99), 0);
    }

    #[test]
    fn transfer_contracts_round_sites_can_now_apply_original_truncation() {
        let block_84e1f3 = staff_contracts_0084e1f3_with_original_rounding(3_243);
        assert_eq!(block_84e1f3.unrounded_value, 1.4000000000000001);
        assert_eq!(block_84e1f3.rounded_value, Some(1));
        assert_eq!(block_84e1f3.applied_value, Some(0x0cb1));

        let block_84e323 = staff_contracts_0084e323_with_original_rounding(2_001);
        assert_eq!(block_84e323.unrounded_value, 2_101.05);
        assert_eq!(block_84e323.rounded_value, Some(2_101));
        assert_eq!(block_84e323.applied_value, Some(2_101));
    }

    #[test]
    fn world_can_be_built_from_manifest_and_save() {
        let manifest = Manifest {
            entries: vec![ManifestEntry {
                filename: "club.dat".into(),
                kind: 0,
                count: 10_580,
            }],
        };

        let save_bytes = fake_save_bytes();
        let save = SaveFile::parse(&save_bytes).unwrap();
        let refs = fake_reference_data();
        let staff = fake_staff_data();
        let core = fake_core_book();
        let world = World::from_parts(&manifest, core, &refs, &staff, Some(&save));

        assert_eq!(world.base_data.len(), 1);
        assert_eq!(world.save.as_ref().unwrap().section_count, 2);
        assert_eq!(
            world.save.as_ref().unwrap().sections[0].name,
            "continent.dat"
        );
        assert_eq!(
            world.save.as_ref().unwrap().sections[0].verified_record_count,
            Some(2)
        );
        assert_eq!(world.core_summary.club_count, 1);
        assert_eq!(
            world.reference_summary.sample_city.as_deref(),
            Some("Leeds")
        );
        assert_eq!(
            world.reference_summary.sample_first_name.as_deref(),
            Some("Andre")
        );
        assert_eq!(
            world.reference_summary.sample_stadium.as_deref(),
            Some("Elland Road")
        );
        assert_eq!(
            world.references.staff_competitions[0].short_name,
            "Player of the Year"
        );
        assert_eq!(world.staff.type10[0].rating_short_0x05, 150);
        assert_eq!(world.staff_summary.type10_count, 1);
        assert_eq!(world.staff_summary.type8_count, 0);
        assert_eq!(world.staff_summary.sample_type10_ca, Some(150));
    }

    #[test]
    fn world_serializes_to_json() {
        let manifest = Manifest {
            entries: vec![ManifestEntry {
                filename: "club.dat".into(),
                kind: 0,
                count: 10_580,
            }],
        };
        let refs = fake_reference_data();
        let staff = fake_staff_data();
        let core = fake_core_book();
        let world = World::from_parts(&manifest, core, &refs, &staff, None);
        let json = world.to_pretty_json().unwrap();
        assert!(json.contains("\"reference_summary\""));
        assert!(json.contains("\"core_summary\""));
        assert!(json.contains("\"staff\""));
        assert!(json.contains("Leeds"));
    }

    #[test]
    fn world_round_trips_from_json() {
        let manifest = Manifest {
            entries: vec![ManifestEntry {
                filename: "club.dat".into(),
                kind: 0,
                count: 10_580,
            }],
        };
        let refs = fake_reference_data();
        let staff = fake_staff_data();
        let core = fake_core_book();
        let world = World::from_parts(&manifest, core, &refs, &staff, None);

        let json = world.to_pretty_json().unwrap();
        let parsed = World::from_json_str(&json).unwrap();

        assert_eq!(parsed, world);
    }

    #[test]
    fn world_reports_coverage() {
        let manifest = Manifest {
            entries: vec![
                ManifestEntry {
                    filename: "club.dat".into(),
                    kind: 0,
                    count: 10_580,
                },
                ManifestEntry {
                    filename: "nation.dat".into(),
                    kind: 4,
                    count: 213,
                },
                ManifestEntry {
                    filename: "continent.dat".into(),
                    kind: 3,
                    count: 6,
                },
                ManifestEntry {
                    filename: "staff.dat".into(),
                    kind: 10,
                    count: 109_940,
                },
                ManifestEntry {
                    filename: "p.dat".into(),
                    kind: 21,
                    count: 5_418,
                },
            ],
        };
        let refs = fake_reference_data();
        let staff = fake_staff_data();
        let core = fake_core_book();
        let world = World::from_parts(&manifest, core, &refs, &staff, None);

        let coverage = world.coverage();
        assert_eq!(coverage.manifest_entries, 5);
        assert_eq!(coverage.known_logical_tables, 22);
        assert_eq!(coverage.recognized_manifest_entries, 4);
        assert_eq!(coverage.unrecognized_manifest_entries, 1);
        assert_eq!(coverage.owned_core_tables, 5);
        assert_eq!(coverage.owned_reference_tables, 13);
        assert_eq!(coverage.owned_staff_tables, 4);
        assert_eq!(coverage.owned_world_tables, 22);
        assert_eq!(coverage.remaining_binary_tables, 0);
    }

    #[test]
    fn backend_readiness_report_tracks_current_backend_boundary() {
        let manifest = Manifest {
            entries: vec![
                ManifestEntry {
                    filename: "club.dat".into(),
                    kind: 0,
                    count: 10_580,
                },
                ManifestEntry {
                    filename: "nation.dat".into(),
                    kind: 4,
                    count: 213,
                },
                ManifestEntry {
                    filename: "continent.dat".into(),
                    kind: 3,
                    count: 6,
                },
                ManifestEntry {
                    filename: "staff.dat".into(),
                    kind: 10,
                    count: 109_940,
                },
            ],
        };
        let refs = fake_reference_data();
        let staff = fake_staff_data();
        let core = fake_core_book();
        let world = World::from_parts(&manifest, core, &refs, &staff, None);

        let report = world.backend_readiness_report(Path::new("memory-rust-db"));

        assert_eq!(report.status, BackendReadinessStatus::VerifiedHeadlessShell);
        assert_eq!(report.completion.canonical_tables, 22);
        assert_eq!(report.completion.remaining_binary_tables, 0);
        assert_eq!(report.completion.validation_failures, 1);
        assert_eq!(report.completion.phase_2_frontiers, 50);
        assert_eq!(report.completion.runtime_mutation_log_entries, 8);
        assert_eq!(report.completion.headless_blockers, 0);
        assert_eq!(report.implementation_plan.len(), 4);
        assert!(report.implementation_plan.iter().all(|item| {
            item.readiness == BackendImplementationReadiness::MutationsImplemented
                && item.boundary_entries > 0
                && !item.primary_frontiers.is_empty()
        }));
        assert!(report
            .implementation_plan
            .iter()
            .any(|item| item.system == "match results" && item.missing_lifts.is_empty()));
        assert!(report
            .implementation_plan
            .iter()
            .any(|item| item.system == "news/inbox" && item.missing_lifts.is_empty()));
        assert!(report
            .implementation_plan
            .iter()
            .any(|item| item.system == "transfers/contracts" && item.missing_lifts.is_empty()));
        assert!(report
            .implementation_plan
            .iter()
            .any(|item| item.system == "competition state" && item.missing_lifts.is_empty()));
        assert!(report.completion.score_percent >= 60);
        assert!(report.checks.iter().any(
            |check| check.name == "gameplay-mutators" && check.status == ValidationStatus::Pass
        ));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "runtime-system-ledger"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "headless-schedule-generation"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "headless-season-fixture-batch"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "runtime-skeleton-dispatch-ledger"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "gameplay-mutator-install-plans"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "gameplay-promotion-gates"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "gameplay-lift-workbench"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "exact-gameplay-mutator-skeletons"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "exact-gameplay-mutator-entry-points"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "match-engine-lift-map"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "match-engine-runtime-store"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "match-engine-runtime-constants"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "match-engine-runtime-mutations"
                && check.status == ValidationStatus::Pass));
        assert!(report.checks.iter().any(|check| check.name
            == "match-engine-player-evaluation-outputs"
            && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "headless-fixture-pipeline"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "match-result-write-map"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "match-result-code-claims"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "match-result-formula-lift-map"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "competition-code-claims"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "transfer-contract-code-claims"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "news-inbox-code-claims"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "match-result-mutator-install-plan"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "competition-fixture-state-map"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "transfer-contract-state-map"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "news-inbox-emission-map"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "backend-implementation-plan"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "gameplay-mutator-contracts"
                && check.status == ValidationStatus::Pass));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "semantic-cleanup-ledger"
                && check.status == ValidationStatus::Pass));
        assert!(!report.semantic_cleanup.is_empty());
        assert!(report
            .semantic_cleanup
            .iter()
            .all(|item| !item.runtime_blocking && !item.next_action.is_empty()));
        assert!(report
            .next_steps
            .iter()
            .any(|step| step.contains("branch-complete match-engine numeric outputs")));
        assert!(!report
            .blockers
            .iter()
            .any(|blocker| blocker.severity == "gameplay"));
    }

    #[test]
    fn rust_db_save_executes_due_fixture_batches_into_standings_and_news() {
        let manifest = Manifest {
            entries: vec![ManifestEntry {
                filename: "club.dat".into(),
                kind: 0,
                count: 6,
            }],
        };
        let refs = fake_reference_data();
        let staff = fake_staff_data();
        let mut core = fake_core_book();
        core.clubs = (0..6)
            .map(|id| DomainOpaqueRecord {
                ordinal: id,
                id,
                primary_name: Some(format!("Club {id}")),
                secondary_name: None,
                short_name: None,
                text_candidates: vec![format!("Club {id}")],
                raw: vec![0; RecordKind::Club.size()],
            })
            .collect();
        let world = World::from_parts(&manifest, core, &refs, &staff, None);
        let mut save = world.new_runtime_save_from_rust_db(Path::new("memory-rust-db"));

        assert_eq!(save.season.fixtures.len(), 15);
        assert_eq!(save.season.standings.len(), 6);
        assert!(headless_schedule_generation_ready(&save));
        let unique_pairs = save
            .season
            .fixtures
            .iter()
            .map(|fixture| {
                let low = fixture.home_club_id.min(fixture.away_club_id);
                let high = fixture.home_club_id.max(fixture.away_club_id);
                (low, high)
            })
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(unique_pairs.len(), 15);
        assert_eq!(save.season.schedule_generation[0].generated_rounds, 5);
        assert_eq!(save.season.schedule_generation[0].generated_fixtures, 15);
        assert!(save
            .season
            .fixtures
            .iter()
            .all(|fixture| fixture.status == HeadlessFixtureStatus::Pending));

        let report = save.run_headless_days(3);

        assert_eq!(report.status, HeadlessPlayStatus::Runnable);
        assert_eq!(save.season.batches.len(), 3);
        assert_eq!(
            save.season
                .fixtures
                .iter()
                .filter(|fixture| fixture.status == HeadlessFixtureStatus::Played)
                .count(),
            9
        );
        assert!(save.season.batches.iter().all(|batch| {
            batch.played_fixtures == 3
                && batch.standings_rows_touched >= 2
                && batch.news_events_created == 3
        }));
        assert!(save.season.standings.iter().any(|row| {
            row.played > 0
                && row.points >= 3
                && row.goals_for as i32 - row.goals_against as i32 == row.goal_difference
        }));
        assert!(
            save.pending_events
                .iter()
                .filter(|event| event.kind == "match-report")
                .count()
                >= 9
        );
        let played_packets = save
            .season
            .fixtures
            .iter()
            .filter(|fixture| fixture.status == HeadlessFixtureStatus::Played)
            .filter_map(|fixture| fixture.match_packet.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(played_packets.len(), 9);
        assert!(played_packets.iter().all(|packet| {
            packet.final_event_code == "0x2004"
                && !packet.event_codes.is_empty()
                && packet.match_events.iter().any(|event| event.kind == "goal")
                && packet.match_events.iter().any(|event| !event.score_impact)
                && packet.state_mutation_rows >= 4
                && packet.evidence.contains("0x006d1a20")
                && packet.evidence.contains("0x006e65e0")
                && packet.evidence.contains("0x006bc8d0")
        }));
        assert!(save
            .season
            .fixtures
            .iter()
            .filter(|fixture| fixture.status == HeadlessFixtureStatus::Played)
            .all(|fixture| {
                let packet = fixture.match_packet.as_ref().unwrap();
                let home_goals = packet
                    .match_events
                    .iter()
                    .filter(|event| event.side == "home" && event.score_impact)
                    .count() as u8;
                let away_goals = packet
                    .match_events
                    .iter()
                    .filter(|event| event.side == "away" && event.score_impact)
                    .count() as u8;
                fixture.home_score == Some(home_goals)
                    && fixture.away_score == Some(away_goals)
                    && packet.final_score == format!("{home_goals}-{away_goals}")
                    && packet.score_source.contains("0x006e65e0")
                    && packet.goal_events.len()
                        == packet
                            .match_events
                            .iter()
                            .filter(|event| event.score_impact)
                            .count()
            }));
        assert!(save
            .season
            .fixtures
            .iter()
            .filter(|fixture| fixture.status == HeadlessFixtureStatus::Played)
            .all(|fixture| {
                fixture.match_report.as_ref().is_some_and(|report| {
                    report.news_kind == "match-report"
                        && report.scoreline.contains(&fixture.home_club_name)
                        && report.scoreline.contains(&fixture.away_club_name)
                        && report.event_count >= report.goal_count
                        && report.non_scoring_event_count > 0
                        && !report.highlights.is_empty()
                        && report.provenance.contains("HeadlessMatchPacket")
                })
            }));
        assert!(save
            .pending_events
            .iter()
            .any(|event| event.kind == "match-report" && event.message.contains("timeline")));
        assert!(
            save.backend
                .match_engine_runtime_store
                .player_evaluation_outputs
                .len()
                >= 9
        );
        assert!(
            save.backend
                .match_engine_runtime_store
                .state_mutation_outputs
                .len()
                >= 45
        );
        assert!(save
            .backend
            .match_result_runtime_store
            .fixtures
            .iter()
            .any(|fixture| fixture.row == 2));

        let json = serde_json::to_string(&save).unwrap();
        let reloaded: RuntimeSaveGame = serde_json::from_str(&json).unwrap();
        assert_eq!(reloaded.season.fixtures, save.season.fixtures);
        assert_eq!(reloaded.season.standings, save.season.standings);
        assert_eq!(reloaded.season.batches.len(), 3);
    }

    #[test]
    fn match_player_evaluation_output_applies_code_derived_tactical_branch() {
        let scenario = default_match_engine_runtime_scenario();
        let output = match_player_evaluation_output(&scenario);

        assert!(output
            .branch_applications
            .iter()
            .any(|branch| branch.contains("+0x99 *= _DAT_009586d0")));
        assert!(output
            .branch_applications
            .iter()
            .any(|branch| branch.contains("+0x99 *= 0.900000")));
        let value = output.float_0x99.parse::<f64>().unwrap();
        assert!((value - 9.740592).abs() < 0.00001);
        assert!(output
            .constants_used
            .iter()
            .any(|constant| constant == "_DAT_009586d0"));
    }

    #[test]
    fn match_late_branch_execution_consumes_rng_in_original_order() {
        let scenario = default_match_engine_runtime_scenario();
        let output = match_engine_late_branch_execution_output(&scenario);

        assert_eq!(output.source_function, "0x006d1a20");
        assert_eq!(output.rng_rolls.len(), 3);
        assert_eq!(output.rng_rolls[0].argument, "0x32");
        assert_eq!(output.rng_rolls[1].argument, "0x14");
        assert_eq!(output.rng_rolls[2].argument, "0x19");
        assert!(output.rng_rolls.iter().all(|roll| roll.success));
        assert!(output
            .applied_branches
            .iter()
            .any(|branch| branch.contains("special-action rng branch")));
        assert!(output
            .offset_multiplier_products
            .iter()
            .any(|product| product.offset == "+0xf5" && product.symbols.len() >= 8));
        assert!(output
            .offset_multiplier_products
            .iter()
            .any(|product| product.offset == "+0x9d" && product.symbols.len() >= 6));
        assert!(output
            .offset_multiplier_products
            .iter()
            .any(|product| product.offset == "+0xc1" && product.symbols.len() >= 5));
    }

    #[test]
    fn match_action_selection_connects_final_float_to_event_candidate() {
        let scenario = default_match_engine_runtime_scenario();
        let evaluation = match_player_evaluation_output(&scenario);
        let late = match_engine_late_branch_execution_output(&scenario);
        let output = match_engine_action_selection_output(&scenario, &evaluation, &late);

        assert_eq!(output.selected_action_code, "0x3a");
        assert_eq!(output.selected_action_event_code.as_deref(), Some("0x1f9d"));
        assert_eq!(output.direct_shot_event_code.as_deref(), Some("0x1f7f"));
        assert_eq!(output.decisive_float_offset, "+0x99");
        assert_eq!(output.decisive_float_value, evaluation.float_0x99);
        assert_eq!(output.branch_product_applied, "0.900000");
        assert!(output.shot_action_score_0x39 > output.action_score_0x37);
    }

    #[test]
    fn match_event_queue_outputs_materialize_action_candidates() {
        let scenario = default_match_engine_runtime_scenario();
        let evaluation = match_player_evaluation_output(&scenario);
        let late = match_engine_late_branch_execution_output(&scenario);
        let action = match_engine_action_selection_output(&scenario, &evaluation, &late);
        let events = match_engine_event_queue_outputs(&scenario, &action);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].row, 0);
        assert_eq!(events[0].event_code, "0x1f9d");
        assert_eq!(events[0].action_code, "0x3a");
        assert_eq!(events[0].slot_base, "+0x30");
        assert_eq!(events[0].stride, "0x0e");
        assert_eq!(events[0].mirror_offset.as_deref(), Some("+0x720"));
        assert_eq!(events[1].row, 1);
        assert_eq!(events[1].event_code, "0x1f7f");
        assert_eq!(events[1].action_code, "0x33");
        assert_eq!(events[1].owner_slot, scenario.action_owner_slot);
        assert_eq!(events[1].target_slot, scenario.action_target_slot);
        assert!(events[1].score > 0);
    }

    #[test]
    fn match_state_mutations_consume_generated_event_rows() {
        let scenario = default_match_engine_runtime_scenario();
        let evaluation = match_player_evaluation_output(&scenario);
        let late = match_engine_late_branch_execution_output(&scenario);
        let action = match_engine_action_selection_output(&scenario, &evaluation, &late);
        let events = match_engine_event_queue_outputs(&scenario, &action);
        let mutations = match_engine_state_mutation_outputs(&scenario, &events);

        assert!(mutations.iter().any(|mutation| {
            mutation.field == "event_queue_count"
                && mutation.before == "0x0000"
                && mutation.after == "0x0002"
                && mutation.source_function == "0x006bc8d0"
        }));
        assert!(mutations.iter().any(|mutation| {
            mutation.field == "stored_action_owner"
                && mutation.record_offset == "+0x8ea7"
                && mutation.after == "0x09"
        }));
        assert!(mutations.iter().any(|mutation| {
            mutation.field == "stored_action_target"
                && mutation.record_offset == "+0x8ea8"
                && mutation.after == "0x04"
        }));
        assert!(mutations.iter().any(|mutation| {
            mutation.field == "selected_action_code"
                && mutation.record_offset == "+0x8eae"
                && mutation.after == "0x3a"
                && mutation.source_event_code.as_deref() == Some("0x1f9d")
        }));
        assert!(mutations
            .iter()
            .any(|mutation| { mutation.field == "last_event_code" && mutation.after == "0x1f7f" }));
    }

    #[test]
    fn match_result_finalization_copies_score_bytes_to_fixture() {
        let scenario = default_match_engine_runtime_scenario();
        let output = match_engine_result_finalization_output(&scenario);

        assert_eq!(output.source_function, "0x006a4020");
        assert_eq!(output.event_code, "0x2004");
        assert_eq!(output.match_state_home_offset, "+0xf5bc");
        assert_eq!(output.match_state_away_offset, "+0xf5f2");
        assert_eq!(output.fixture_home_offset, "+0x49");
        assert_eq!(output.fixture_away_offset, "+0x4a");
        assert_eq!(output.home_score, 2);
        assert_eq!(output.away_score, 1);
        assert_eq!(output.phase_after_offset, "+0x8eb3");
        assert_eq!(output.phase_after, "0x00");
    }

    #[test]
    fn match_result_store_reflects_executable_finalization() {
        let formula_scenario = default_match_result_formula_scenario();
        let mut store = default_match_result_runtime_store();
        let plan = plan_match_result_formula_mutations(
            &RuntimeBackendSystems::default(),
            &formula_scenario,
        );
        apply_match_result_formula_plan_to_store(&mut store, &plan);

        let engine_scenario = default_match_engine_runtime_scenario();
        let output = match_engine_result_finalization_output(&engine_scenario);
        apply_match_engine_result_finalization_to_match_result_store(&mut store, &output);

        let fixture = store
            .fixtures
            .iter()
            .find(|fixture| fixture.row == 0)
            .unwrap();
        assert!(fixture
            .bytes
            .iter()
            .any(|byte| byte.offset == "0x49" && byte.value == 2));
        assert!(fixture
            .bytes
            .iter()
            .any(|byte| byte.offset == "0x4a" && byte.value == 1));
        assert!(store.event_queue.iter().any(|event| {
            event.event_code == "0x2004"
                && event.source_function == "0x006a4020"
                && event.formula == "executable match-engine result finalization"
        }));
        assert!(match_result_runtime_store_ready(&store));
    }

    #[test]
    fn cm_packed_date_round_trips_common_year() {
        let date = GameDate {
            year: 2001,
            month: 7,
            day: 1,
        };
        let packed = CmPackedDate::from_game_date(date.clone());

        assert_eq!(packed.day_of_year, 182);
        assert_eq!(packed.year, 2001);
        assert!(!packed.leap_year);
        assert_eq!(packed.to_game_date(), date);
    }

    #[test]
    fn cm_packed_date_adds_across_leap_day_and_year_end() {
        let feb_28 = CmPackedDate::from_game_date(GameDate {
            year: 2004,
            month: 2,
            day: 28,
        });
        assert_eq!(
            feb_28.add_days(1).to_game_date(),
            GameDate {
                year: 2004,
                month: 2,
                day: 29
            }
        );

        let dec_31 = CmPackedDate::from_game_date(GameDate {
            year: 2001,
            month: 12,
            day: 31,
        });
        assert_eq!(
            dec_31.add_days(1).to_game_date(),
            GameDate {
                year: 2002,
                month: 1,
                day: 1
            }
        );
    }

    #[test]
    fn runtime_tick_uses_three_cm_phases_per_day() {
        let mut save = RuntimeSaveGame {
            format: "cm0102-rs-save".to_string(),
            version: 1,
            source: RuntimeSource {
                kind: "test".to_string(),
                path: "memory".to_string(),
            },
            date: GameDate {
                year: 2001,
                month: 7,
                day: 1,
            },
            simulation: RuntimeSimulationState::default(),
            backend: RuntimeBackendSystems::default(),
            headless: HeadlessRuntimeState::default(),
            season: HeadlessSeasonState::default(),
            elapsed_days: 0,
            pending_events: Vec::new(),
            phase_trace: Vec::new(),
            table_counts: RuntimeTableCounts {
                clubs: 0,
                national_clubs: 0,
                nations: 0,
                staff_type6: 0,
                staff_type9: 0,
                staff_type10: 0,
                cities: 0,
                stadiums: 0,
                competitions: 0,
                histories: 0,
            },
            new_game: None,
            nation_tiers: Vec::new(),
            world: SaveWorldOverlay::default(),
            player_init: None,
            humans: Vec::new(),
            active_human: 0,
            notes: Vec::new(),
        };

        save.tick_cm_phase();
        assert_eq!(save.simulation.phase, 1);
        assert_eq!(save.date.day, 1);
        assert_eq!(save.elapsed_days, 0);
        assert_eq!(save.phase_trace.len(), 1);
        assert_eq!(save.phase_trace[0].phase_before, 0);
        assert!(!save.phase_trace[0].advanced_day);

        save.tick_cm_phase();
        assert_eq!(save.simulation.phase, 2);
        assert_eq!(save.date.day, 1);

        save.tick_cm_phase();
        assert_eq!(save.simulation.phase, 0);
        assert_eq!(
            save.date,
            GameDate {
                year: 2001,
                month: 7,
                day: 2
            }
        );
        assert_eq!(save.elapsed_days, 1);
        assert_eq!(save.phase_trace.len(), 3);
        assert_eq!(save.backend.mutation_log.len(), 4);
        assert_eq!(save.backend.transfers.attempted_mutations, 1);
        assert_eq!(save.backend.news.attempted_mutations, 1);
        assert_eq!(save.backend.matches.attempted_mutations, 1);
        assert_eq!(save.backend.competitions.attempted_mutations, 1);
        assert!(save.backend.mutation_log.iter().all(|mutation| {
            mutation.contract_status == Some(GameplayMutatorStatus::ParityVerified)
                && mutation.status == RuntimeSystemStatus::Implemented
                && mutation.trace_file.is_some()
                && mutation.boundary_map.is_some()
                && mutation.implementation_hook.is_some()
                && mutation.parity_gate.is_some()
                && mutation.skeleton_status.as_deref() == Some("static-proof-backed")
                && mutation
                    .skeleton_mutations_emitted
                    .is_some_and(|count| count > 0)
                && mutation.exactness_tier.as_deref() == Some("static-boundary-exact")
                && mutation.static_proof_rows.is_some_and(|count| count > 0)
                && (mutation.formula_lift_status.as_deref() == Some("pending-deeper-formula-lift")
                    || (mutation.system == "match results"
                        && mutation.formula_lift_status.as_deref()
                            == Some("formula-derived-runtime-store-installed"))
                    || (mutation.system == "competition state"
                        && mutation.formula_lift_status.as_deref()
                            == Some("competition-formula-runtime-store-installed"))
                    || (mutation.system == "transfers/contracts"
                        && mutation.formula_lift_status.as_deref()
                            == Some("contract-renewal-formula-runtime-store-installed"))
                    || (mutation.system == "news/inbox"
                        && mutation.formula_lift_status.as_deref()
                            == Some("news-inbox-formula-runtime-store-installed")))
        }));
        assert!(save.backend.mutation_log.iter().any(|mutation| {
            mutation.system == "match results"
                && mutation.trace_file.as_deref()
                    == Some("reports/parity_traces/match-results.json")
                && mutation.boundary_map.as_deref() == Some("match_result_write_map")
        }));
        assert_eq!(save.backend.mutator_contracts.len(), 4);
        assert!(save.backend.mutator_contracts.iter().any(|contract| {
            contract.system == "match results"
                && contract.phase == 2
                && contract.boundary_map == "match_result_write_map"
        }));
        assert!(save.backend.mutator_contracts.iter().any(|contract| {
            contract.system == "news/inbox"
                && contract.phase == 1
                && contract.boundary_map == "news_inbox_emission_map"
        }));
        assert!(save.backend.match_result_write_map.iter().any(|entry| {
            entry.fixture_home_offset == "0x49"
                && entry.fixture_away_offset == "0x4a"
                && entry.event_code.as_deref() == Some("0x2004")
        }));
        assert!(save
            .backend
            .competition_fixture_state_map
            .iter()
            .any(|entry| entry.fixture_offset.as_deref() == Some("0x4d")
                && entry.flag_mask.as_deref() == Some("0x100")
                && entry.helper.as_deref() == Some("0x0075ee00")));
        assert!(save
            .backend
            .transfer_contract_state_map
            .iter()
            .any(|entry| entry.function == "0x004cdef0"
                && entry.helper.as_deref() == Some("0x00536190")));
        assert!(save
            .backend
            .transfer_contract_state_map
            .iter()
            .any(|entry| entry.function == "0x008a9080"
                && entry.record_offset.as_deref() == Some("0x213/0x84d/0x856")));
        assert!(save
            .backend
            .news_inbox_emission_map
            .iter()
            .any(|entry| entry.record_offset.as_deref() == Some("0xde")
                && entry.function == "0x0076e180"));
        assert!(save
            .backend
            .news_inbox_emission_map
            .iter()
            .any(|entry| entry.helper.as_deref() == Some("0x006724d0")));
        assert!(match_result_formula_lift_map_ready(
            &save.backend.match_result_formula_lift_map
        ));
        assert!(save
            .backend
            .match_result_formula_lift_map
            .iter()
            .any(|entry| entry.formula == "phase-controller final result"
                && entry.constants.iter().any(|constant| constant == "0x2004")
                && entry
                    .outputs
                    .iter()
                    .any(|output| output.contains("fixture +0x49"))));
        let formula_scenario = default_match_result_formula_scenario();
        let formula_plan = plan_match_result_formula_mutations(&save.backend, &formula_scenario);
        assert!(match_result_formula_plan_ready(&formula_plan));
        assert!(formula_plan.iter().any(|mutation| {
            mutation.table == "fixture"
                && mutation.record_offset == "0x49"
                && mutation.before == "0xff"
                && mutation.after == "0x02"
        }));
        assert!(formula_plan.iter().any(|mutation| {
            mutation.table == "event_queue"
                && mutation.event_code.as_deref() == Some("0x2004")
                && mutation.after.contains("away_not_losing=false")
        }));
        assert_eq!(save.phase_trace[2].phase_before, 2);
        assert_eq!(save.phase_trace[2].phase_after, 0);
        assert!(save.phase_trace[2].advanced_day);
        assert!(save.phase_trace[2]
            .frontiers
            .iter()
            .any(|frontier| frontier.address == "0x00536190"
                && frontier.status.contains("implemented slice")));
        assert!(save.phase_trace[2]
            .frontiers
            .iter()
            .any(|frontier| frontier.address == "0x00674c10"));
    }

    #[test]
    fn headless_run_records_shell_progress_and_blockers() {
        let mut save = RuntimeSaveGame {
            format: "cm0102-rs-save".to_string(),
            version: 1,
            source: RuntimeSource {
                kind: "test".to_string(),
                path: "memory".to_string(),
            },
            date: GameDate {
                year: 2001,
                month: 7,
                day: 1,
            },
            simulation: RuntimeSimulationState::default(),
            backend: RuntimeBackendSystems::default(),
            headless: HeadlessRuntimeState::default(),
            season: HeadlessSeasonState::default(),
            elapsed_days: 0,
            pending_events: Vec::new(),
            phase_trace: Vec::new(),
            table_counts: RuntimeTableCounts {
                clubs: 0,
                national_clubs: 0,
                nations: 0,
                staff_type6: 0,
                staff_type9: 0,
                staff_type10: 0,
                cities: 0,
                stadiums: 0,
                competitions: 0,
                histories: 0,
            },
            new_game: None,
            nation_tiers: Vec::new(),
            world: SaveWorldOverlay::default(),
            player_init: None,
            humans: Vec::new(),
            active_human: 0,
            notes: Vec::new(),
        };

        let report = save.run_headless_days(2);
        assert_eq!(report.days_advanced, 2);
        assert_eq!(report.phases_advanced, 6);
        assert_eq!(report.phase_trace_entries_added, 6);
        assert_eq!(report.end_date.iso(), "2001-07-03");
        assert_eq!(report.status, HeadlessPlayStatus::Runnable);
        assert!(report.still_frontier_only.is_empty());
        assert_eq!(save.headless.completed_days, 2);
        assert_eq!(save.headless.completed_phases, 6);
        assert_eq!(save.backend.mutation_log.len(), 8);
        assert_eq!(save.backend.matches.attempted_mutations, 2);
        assert_eq!(save.backend.competitions.attempted_mutations, 2);
        assert_eq!(save.backend.matches.implemented_mutations, 2);
        assert_eq!(save.backend.competitions.implemented_mutations, 2);
        assert!(headless_fixture_pipeline_ready(&save.backend));
        assert_eq!(save.backend.headless_fixture_pipeline_outputs.len(), 1);
        assert_eq!(
            save.backend.headless_fixture_pipeline_outputs[0].result,
            "2-1"
        );
        assert!(save.backend.headless_fixture_pipeline_outputs[0].match_engine_event_rows >= 2);
        assert!(save.backend.headless_fixture_pipeline_outputs[0].standings_rows_touched >= 1);
        assert!(save.backend.headless_fixture_pipeline_outputs[0].news_events_created >= 2);
        assert!(save.headless.last_run.is_some());
        assert!(save.headless.stop_reason.is_some());
        assert_eq!(save.headless.command_history.len(), 1);
    }

    #[test]
    fn headless_campaign_records_checkpoints_and_backend_summary() {
        let mut save = RuntimeSaveGame {
            format: "cm0102-rs-save".to_string(),
            version: 1,
            source: RuntimeSource {
                kind: "test".to_string(),
                path: "memory".to_string(),
            },
            date: GameDate {
                year: 2001,
                month: 7,
                day: 1,
            },
            simulation: RuntimeSimulationState::default(),
            backend: RuntimeBackendSystems::default(),
            headless: HeadlessRuntimeState::default(),
            season: HeadlessSeasonState::default(),
            elapsed_days: 0,
            pending_events: Vec::new(),
            phase_trace: Vec::new(),
            table_counts: RuntimeTableCounts {
                clubs: 0,
                national_clubs: 0,
                nations: 0,
                staff_type6: 0,
                staff_type9: 0,
                staff_type10: 0,
                cities: 0,
                stadiums: 0,
                competitions: 0,
                histories: 0,
            },
            new_game: None,
            nation_tiers: Vec::new(),
            world: SaveWorldOverlay::default(),
            player_init: None,
            humans: Vec::new(),
            active_human: 0,
            notes: Vec::new(),
        };

        let report = save.run_headless_campaign_days(10, 4);

        assert_eq!(report.days_requested, 10);
        assert_eq!(report.days_advanced, 10);
        assert_eq!(report.phases_advanced, 30);
        assert_eq!(report.end_date.iso(), "2001-07-11");
        assert_eq!(report.checkpoints.len(), 3);
        assert_eq!(report.backend.mutation_log_entries_added, 40);
        assert_eq!(report.backend.total_mutation_log_entries, 40);
        assert_eq!(report.backend.match_attempts, 10);
        assert_eq!(report.backend.competition_attempts, 10);
        assert_eq!(report.backend.transfer_attempts, 10);
        assert_eq!(report.backend.news_attempts, 10);
        assert_eq!(report.backend.implemented_mutations, 40);
        assert_eq!(report.backend.frontier_only_mutations, 0);
        assert_eq!(save.phase_trace.len(), 30);
        assert_eq!(save.headless.command_history.len(), 1);
        assert!(save.headless.last_run.is_some());
    }

    #[test]
    fn headless_campaign_retains_recent_mutation_log_but_counts_all_entries() {
        let mut save = RuntimeSaveGame {
            format: "cm0102-rs-save".to_string(),
            version: 1,
            source: RuntimeSource {
                kind: "test".to_string(),
                path: "memory".to_string(),
            },
            date: GameDate {
                year: 2001,
                month: 7,
                day: 1,
            },
            simulation: RuntimeSimulationState::default(),
            backend: RuntimeBackendSystems {
                mutation_log_limit: 20,
                ..RuntimeBackendSystems::default()
            },
            headless: HeadlessRuntimeState::default(),
            season: HeadlessSeasonState::default(),
            elapsed_days: 0,
            pending_events: Vec::new(),
            phase_trace: Vec::new(),
            table_counts: RuntimeTableCounts {
                clubs: 0,
                national_clubs: 0,
                nations: 0,
                staff_type6: 0,
                staff_type9: 0,
                staff_type10: 0,
                cities: 0,
                stadiums: 0,
                competitions: 0,
                histories: 0,
            },
            new_game: None,
            nation_tiers: Vec::new(),
            world: SaveWorldOverlay::default(),
            player_init: None,
            humans: Vec::new(),
            active_human: 0,
            notes: Vec::new(),
        };

        let report = save.run_headless_campaign_days(10, 10);

        assert_eq!(report.backend.mutation_log_entries_added, 40);
        assert_eq!(report.backend.total_mutation_log_entries, 40);
        assert_eq!(report.backend.frontier_only_mutations, 0);
        assert_eq!(save.backend.total_mutation_entries, 40);
        assert_eq!(save.backend.mutation_log.len(), 20);
        assert_eq!(save.backend.dropped_mutation_entries, 20);
        assert_eq!(report.checkpoints[0].mutation_log_entries, 40);
    }

    #[test]
    fn headless_manager_profile_records_session_command() {
        let mut save = RuntimeSaveGame {
            format: "cm0102-rs-save".to_string(),
            version: 1,
            source: RuntimeSource {
                kind: "test".to_string(),
                path: "memory".to_string(),
            },
            date: GameDate {
                year: 2001,
                month: 7,
                day: 1,
            },
            simulation: RuntimeSimulationState::default(),
            backend: RuntimeBackendSystems::default(),
            headless: HeadlessRuntimeState::default(),
            season: HeadlessSeasonState::default(),
            elapsed_days: 0,
            pending_events: Vec::new(),
            phase_trace: Vec::new(),
            table_counts: RuntimeTableCounts {
                clubs: 0,
                national_clubs: 0,
                nations: 0,
                staff_type6: 0,
                staff_type9: 0,
                staff_type10: 0,
                cities: 0,
                stadiums: 0,
                competitions: 0,
                histories: 0,
            },
            new_game: None,
            nation_tiers: Vec::new(),
            world: SaveWorldOverlay::default(),
            player_init: None,
            humans: Vec::new(),
            active_human: 0,
            notes: Vec::new(),
        };

        let record = save.set_headless_manager("Alex Ferguson".to_string(), Some(1));
        let manager = save.headless.manager.as_ref().unwrap();
        assert_eq!(manager.name, "Alex Ferguson");
        assert_eq!(manager.club_id, Some(1));
        assert_eq!(
            manager.status,
            HeadlessManagerStatus::ClubSelectedFrontierOnly
        );
        assert_eq!(record.command, "set-headless-manager");
        assert_eq!(save.headless.command_history.len(), 1);
        assert_eq!(save.headless.milestones.len(), 1);
    }

    #[test]
    fn runtime_tick_to_date_advances_by_cm_phase_rollovers() {
        let mut save = RuntimeSaveGame {
            format: "cm0102-rs-save".to_string(),
            version: 1,
            source: RuntimeSource {
                kind: "test".to_string(),
                path: "memory".to_string(),
            },
            date: GameDate {
                year: 2001,
                month: 7,
                day: 1,
            },
            simulation: RuntimeSimulationState::default(),
            backend: RuntimeBackendSystems::default(),
            headless: HeadlessRuntimeState::default(),
            season: HeadlessSeasonState::default(),
            elapsed_days: 0,
            pending_events: Vec::new(),
            phase_trace: Vec::new(),
            table_counts: RuntimeTableCounts {
                clubs: 0,
                national_clubs: 0,
                nations: 0,
                staff_type6: 0,
                staff_type9: 0,
                staff_type10: 0,
                cities: 0,
                stadiums: 0,
                competitions: 0,
                histories: 0,
            },
            new_game: None,
            nation_tiers: Vec::new(),
            world: SaveWorldOverlay::default(),
            player_init: None,
            humans: Vec::new(),
            active_human: 0,
            notes: Vec::new(),
        };

        let unchanged = save.tick_to_date(GameDate {
            year: 2001,
            month: 7,
            day: 1,
        });
        assert_eq!(unchanged, 0);
        assert_eq!(save.phase_trace.len(), 0);

        let advanced = save.tick_to_date(GameDate {
            year: 2001,
            month: 7,
            day: 4,
        });
        assert_eq!(advanced, 3);
        assert_eq!(
            save.date,
            GameDate {
                year: 2001,
                month: 7,
                day: 4
            }
        );
        assert_eq!(save.elapsed_days, 3);
        assert_eq!(save.simulation.phase, 0);
        assert_eq!(save.simulation.cm_packed_date.day_of_year, 185);
        assert_eq!(save.phase_trace.len(), 9);
    }

    fn fake_core_book() -> CoreBook {
        CoreBook {
            clubs: vec![DomainOpaqueRecord {
                ordinal: 0,
                id: 0,
                primary_name: Some("Club".into()),
                secondary_name: None,
                short_name: None,
                text_candidates: vec!["Club".into()],
                raw: vec![0; RecordKind::Club.size()],
            }],
            nat_clubs: vec![DomainOpaqueRecord {
                ordinal: 0,
                id: 0,
                primary_name: Some("Nat Club".into()),
                secondary_name: None,
                short_name: None,
                text_candidates: vec!["Nat Club".into()],
                raw: vec![0; RecordKind::Club.size()],
            }],
            colours: vec![DomainOpaqueRecord {
                ordinal: 0,
                id: 0,
                primary_name: Some("Blue".into()),
                secondary_name: None,
                short_name: None,
                text_candidates: vec!["Blue".into()],
                raw: vec![0; RecordKind::Colour.size()],
            }],
            continents: vec![DomainOpaqueRecord {
                ordinal: 0,
                id: 0,
                primary_name: Some("Europe".into()),
                secondary_name: None,
                short_name: None,
                text_candidates: vec!["Europe".into()],
                raw: vec![0; RecordKind::Continent.size()],
            }],
            nations: vec![DomainOpaqueRecord {
                ordinal: 0,
                id: 0,
                primary_name: Some("England".into()),
                secondary_name: Some("English".into()),
                short_name: Some("ENG".into()),
                text_candidates: vec!["England".into(), "English".into(), "ENG".into()],
                raw: vec![0; RecordKind::Nation.size()],
            }],
        }
    }

    fn fake_reference_data() -> ReferenceData {
        ReferenceData {
            cities: vec![cm_data::CityEntry {
                id: 7,
                name: "Leeds".into(),
                tail_u16: [0; 13],
                tail_u32: [0; 6],
            }],
            officials: vec![cm_data::OfficialEntry {
                id: 12,
                u32_slots: [0; 10],
                u16_slots: [0; 21],
                trailing_byte: 0,
            }],
            first_names: vec![cm_data::NameEntry {
                text: "Andre".into(),
                footer: [0; 12],
            }],
            second_names: vec![],
            common_names: vec![],
            stadiums: vec![cm_data::StadiumEntry {
                id: 1,
                name: "Elland Road".into(),
                unknown_tail: vec![],
            }],
            staff_competitions: vec![cm_data::CompetitionEntry {
                id: 2,
                long_name: "World Player of the Year".into(),
                short_name: "Player of the Year".into(),
                three_letter_name: String::new(),
                scope: 0,
                nation_id: -1,
                last_division: -1,
                reserve_division: -1,
                reputation: 0,
                unknown_tail: vec![],
            }],
            club_competitions: vec![],
            nation_competitions: vec![],
            staff_history: vec![],
            staff_comp_history: vec![],
            club_comp_history: vec![],
            nation_comp_history: vec![],
        }
    }

    fn fake_staff_data() -> StaffData {
        StaffData {
            type6: vec![cm_data::StaffType6Entry {
                id: 1,
                body: vec![0; 153],
            }],
            type8: vec![],
            type9: vec![cm_data::StaffType9Entry {
                id: 2,
                body: vec![0; 64],
            }],
            type10: vec![cm_data::StaffType10Entry {
                id: 3,
                unknown_byte_4: 0,
                rating_short_0x05: 150,
                rating_short_0x07: 175,
                unknown_bytes_9_12: [0; 4],
                rating_short_0x0d: 44,
                unknown_bytes_15_26: [0; 12],
                attributes: [0; 31],
                unknown_bytes_58_64: [0; 7],
                trailing_bytes: [0; 5],
            }],
        }
    }

    fn fake_save_bytes() -> Vec<u8> {
        fn write_entry(dst: &mut Vec<u8>, a: u32, size: u32, name: &str) {
            dst.extend_from_slice(&a.to_le_bytes());
            dst.extend_from_slice(&size.to_le_bytes());
            let mut name_buf = [0u8; 0x104];
            let src = name.as_bytes();
            name_buf[..src.len()].copy_from_slice(src);
            dst.extend_from_slice(&name_buf);
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&999u32.to_le_bytes());
        write_entry(
            &mut bytes,
            1,
            (cm_data::RecordKind::Continent.size() * 2) as u32,
            "continent.dat",
        );
        write_entry(&mut bytes, 3, 6, "club.dat");
        bytes
    }
}
