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
        let injuries = [
            if injured { "Injured".to_string() } else { "None".to_string() },
            "-".to_string(),                       // Type — detail TBD.
            "100%".to_string(),                    // Condition — init seed 156.
            if save.injuries.is_training_available(staff_id) {
                "Full".to_string()
            } else {
                "Restricted".to_string()
            },
            if banned { "Suspended".to_string() } else { "None".to_string() },
        ];

        Some(PlayerProfileView {
            staff_id, title, born_line, attributes, status, position, career,
            injuries, is_goalkeeper,
            club_id: pv.current_club_id().map(|c| c as u32),
        })
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
    const T: i8 = 15;
    let mut positions: Vec<&str> = Vec::new();
    if a.apt_goalkeeper     >= T { positions.push("Goalkeeper"); }
    if a.apt_sweeper        >= T { positions.push("Sweeper"); }
    if a.apt_defender       >= T { positions.push("Defender"); }
    if a.apt_def_midfielder >= T { positions.push("Defensive Midfielder"); }
    if a.apt_midfielder     >= T { positions.push("Midfielder"); }
    if a.apt_att_midfielder >= T { positions.push("Attacking Midfielder"); }
    if a.apt_attacker >= T {
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
    let mut sides: Vec<&str> = Vec::new();
    if a.apt_left_side >= T { sides.push("Left"); }
    if a.apt_central   >= T { sides.push("Centre"); }
    if a.apt_right_side >= T { sides.push("Right"); }
    let pos = positions.join("/");
    if sides.is_empty() {
        pos
    } else {
        format!("{pos} ({})", sides.join(", "))
    }
}
