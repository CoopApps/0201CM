//! General Info screen view model (top tab #4, default "General Info"
//! page — the not-managed view). Decode:
//! `reports/general_info_screen_decode.md`.

use serde::{Deserialize, Serialize};

use crate::{RuntimeSaveGame, World};

/// One non-playing-staff row: person name + role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffRow {
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralInfoView {
    pub club_id: u32,
    pub nation: String,
    pub status: String,
    pub finances: String,
    pub stadium: String,
    pub facilities: String,
    pub training_ground: String,
    pub non_playing_staff: Vec<StaffRow>,
}

/// `club_job` byte → role name. Derived from the General Info capture
/// cross-referenced with rust-db (see the decode report):
///   1 = Chairman, 5 = Manager, 10 = Physio.
/// Other bytes are not yet resolved (the sample club's coach/scouts are
/// missing from rust-db), so they render as "Staff" rather than a
/// guessed role — honest until the mapping is decoded.
/// `club_job` byte → role name. AUTHORITATIVE — transcribed from the
/// exe's own role-name switch `FUN_00524850` (cases 0..0xd), confirmed
/// against Burnley's staff in rust-db (job 6 = Sam Ellis / Assistant
/// Manager, job 8 = coaches, job 9 = scouts).
// GDI-REG: 00524850 PORTED_EXACT
fn role_for_job(job: u8) -> &'static str {
    match job {
        0 => "Unemployed",
        1 => "Chairman",
        2 => "Managing Director",
        3 => "General Manager",
        4 => "Director of Football",
        5 => "Manager",
        6 => "Assistant Manager",
        7 => "Reserve Team Manager",
        8 => "Coach",
        9 => "Scout",
        10 => "Physio",
        11 => "Player",
        12 => "Player/Manager",
        13 => "Player/Assistant Manager",
        _ => "Staff",
    }
}

/// `club_job` byte for an ordinary squad player (everyone the General
/// Info staff list must EXCLUDE). Verified against Chester: every squad
/// player is job 11; the non-playing roles (Chairman 1, Manager 5,
/// Physio 10, Player/Assistant Manager 13) are the other bytes.
const JOB_ORDINARY_PLAYER: u8 = 11;

impl World {
    pub fn general_info_for(
        &self,
        save: &RuntimeSaveGame,
        club_id: u32,
    ) -> Option<GeneralInfoView> {
        use crate::typed_records::{ClubView, PlayerView};
        let club_rec = self.core.clubs.iter().find(|c| ClubView::new(c).id() == club_id)?;
        let cv = ClubView::new(club_rec);

        let nation = cv.nation_id()
            .and_then(|n| self.nation_name(n as u32))
            .unwrap_or_else(|| "Unknown".to_string());

        // Status — most league clubs are Professional. The exact club
        // status byte is not yet located; this is a labelled default
        // (see decode report). Semi-pro/amateur lower tiers differ.
        let status = "Professional".to_string();

        // Finances — coarse word from the runtime finance status.
        let finances = match save.finance.for_club(club_id).map(|c| {
            let rep = self.core.clubs.iter().find(|r| ClubView::new(r).id() == club_id)
                .map(|r| ClubView::new(r).reputation()).unwrap_or(1000);
            c.status(rep)
        }) {
            Some(crate::finance::FinanceStatus::Rich) => "Very healthy",
            Some(crate::finance::FinanceStatus::Healthy) => "Healthy",
            Some(crate::finance::FinanceStatus::Normal) => "Ok",
            Some(crate::finance::FinanceStatus::InTheRed) => "In trouble",
            Some(crate::finance::FinanceStatus::Admin) => "In administration",
            None => "Ok",
        }.to_string();

        // Stadium + facilities from the home stadium record.
        let (stadium, facilities) = cv.home_stadium_id()
            .and_then(|sid| self.references.stadiums.iter().find(|s| s.id as i32 == sid))
            .map(|s| {
                let name = if s.name_set && !s.name.trim().is_empty() {
                    s.name.clone()
                } else { "Unknown".to_string() };
                let fac = format!("{} ({} seated)", s.capacity_total, s.capacity_seated);
                (name, fac)
            })
            .unwrap_or_else(|| ("Unknown".to_string(), "Unknown".to_string()));

        // Training ground — descriptor not yet in rust-db; labelled default.
        let training_ground = "Adequate facilities".to_string();

        // Non-playing staff at this club. Chairman first, Manager
        // second, then the rest (physio, etc.), matching the exe's
        // ordering. Sourced from whatever rust-db holds — a known gap is
        // that some clubs' coach/scouts are absent (see decode).
        let mut staff: Vec<(u8, StaffRow)> = Vec::new();
        for p in &self.staff.type6 {
            let pv = PlayerView::from_split(p.id, &p.body);
            if pv.current_club_id() != Some(club_id as i32) { continue; }
            let job = pv.club_job();
            // Exclude ordinary squad players (job 11) — but KEEP players
            // who also hold a staff role (e.g. job 13 Player/Assistant
            // Manager, Dean Spink at Chester), which `is_player()` would
            // wrongly filter out.
            if job == JOB_ORDINARY_PLAYER { continue; }
            // Canonical resolver — honours the common-name override (e.g.
            // Burnley scout Liz Catlow, whose first_name_id 12731 is "L."
            // but common_name_id 1584 is "Liz Catlow", what the original
            // renders).
            let name = self.person_display_name(p);
            if name.is_empty() { continue; }
            staff.push((job, StaffRow { name, role: role_for_job(job).to_string() }));
        }
        // Order: Chairman (1), Manager (5), then others by job byte.
        staff.sort_by_key(|(job, _)| match job { 1 => 0, 5 => 1, j => 2 + *j as u16 });
        let non_playing_staff = staff.into_iter().map(|(_, r)| r).collect();

        Some(GeneralInfoView {
            club_id, nation, status, finances, stadium, facilities,
            training_ground, non_playing_staff,
        })
    }
}

/// The 16 computed values of the General Info → Stats page, in display
/// order. Labels are fixed chrome (held by the renderer); these are the
/// yellow value-column strings. Sourced from the SAME runtime feed the
/// Squad screen uses (contract-pool wage/value, `age_at`, injury book),
/// so Stats agrees with Squad by construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralInfoStatsView {
    pub club_id: u32,
    pub number_of_players: String,
    pub number_injured: String,
    pub avg_age_first_team: String,
    pub avg_age_squad: String,
    pub wage_bill_first_team: String,
    pub wage_bill_squad: String,
    pub average_wage: String,
    pub highest_wage: String,
    pub lowest_wage: String,
    pub oldest_player: String,
    pub youngest_player: String,
    pub highest_valued: String,
    pub num_international: String,
    pub num_under21: String,
    pub num_foreign: String,
    pub num_non_eu: String,
}

impl GeneralInfoStatsView {
    /// The 16 value strings in top-to-bottom display order — the order the
    /// renderer paints them against the fixed labels.
    pub fn value_rows(&self) -> [&str; 16] {
        [
            &self.number_of_players, &self.number_injured,
            &self.avg_age_first_team, &self.avg_age_squad,
            &self.wage_bill_first_team, &self.wage_bill_squad,
            &self.average_wage, &self.highest_wage, &self.lowest_wage,
            &self.oldest_player, &self.youngest_player, &self.highest_valued,
            &self.num_international, &self.num_under21,
            &self.num_foreign, &self.num_non_eu,
        ]
    }
}

/// Wage-style money: `<1000` plain, else K/M with up to one decimal
/// (capture: £150, £700, £2K, £18.5K). £ is CP1252 0xA3.
fn fmt_wage(n: i64) -> String {
    if n < 1000 {
        format!("\u{a3}{n}")
    } else if n < 1_000_000 {
        let k = n as f64 / 1000.0;
        if (k - k.round()).abs() < 0.05 {
            format!("\u{a3}{}K", k.round() as i64)
        } else {
            format!("\u{a3}{k:.1}K")
        }
    } else {
        let m = n as f64 / 1_000_000.0;
        if (m - m.round()).abs() < 0.05 {
            format!("\u{a3}{}M", m.round() as i64)
        } else {
            format!("\u{a3}{m:.1}M")
        }
    }
}

/// Full comma-grouped money (capture: Highest Valued = £350,000).
pub(crate) fn fmt_value_full(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    format!("\u{a3}{}{}", if n < 0 { "-" } else { "" }, out)
}

/// EU/EEA membership as of the 2001 game start, keyed by CM nationality
/// name. The exe stores a per-nation work-permit/EU flag we have not yet
/// surfaced from `nation.dat`; until it is decoded, Non-EU is derived
/// from the real-world EU-15 + EEA set (a documented approximation, not a
/// game value). Brighton's only foreign player, Dirk Lehmann (German),
/// is EU, so the capture's Non-EU = 0 is reproduced.
fn is_eu_nation_name(name: &str) -> bool {
    const EU: &[&str] = &[
        // EU-15 (2001)
        "England", "Scotland", "Wales", "Northern Ireland",
        "Republic of Ireland", "Ireland", "France", "Germany", "Italy",
        "Spain", "Portugal", "Netherlands", "Holland", "Belgium",
        "Luxembourg", "Denmark", "Sweden", "Finland", "Austria", "Greece",
        // EEA (work-permit-exempt)
        "Iceland", "Norway", "Liechtenstein",
    ];
    EU.iter().any(|n| n.eq_ignore_ascii_case(name))
}

impl World {
    /// Build the General Info → Stats page for `club_id`. Values come from
    /// runtime state so they match the Squad screen; see the decode report
    /// for the ground-truth Brighton capture and the rows whose exe
    /// definition is still pending.
    pub fn general_info_stats_for(
        &self,
        save: &RuntimeSaveGame,
        club_id: u32,
    ) -> Option<GeneralInfoStatsView> {
        use crate::typed_records::{ClubView, PlayerView};
        let club_rec = self.core.clubs.iter().find(|c| ClubView::new(c).id() == club_id)?;
        let club_nation = ClubView::new(club_rec).nation_id();

        // Season-start reference date — the same one the Squad screen's
        // age column uses, so both screens agree.
        const REF_YEAR: u16 = 2001;
        let ref_day = crate::day_of_year(2001, 8, 10);

        struct P {
            id: u32,
            name: String,
            wage: i64,
            value: i64,
            age: Option<u8>,
            nation: Option<i32>,
            nation_name: Option<String>,
        }
        let mut players: Vec<P> = Vec::new();
        for p in &self.staff.type6 {
            let pv = PlayerView::from_split(p.id, &p.body);
            if pv.current_club_id() != Some(club_id as i32) { continue; }
            // Players (11) and player-staff (Player/Manager 12,
            // Player/Assistant Manager 13) count; pure non-players don't.
            if !matches!(pv.club_job(), 11 | 12 | 13) { continue; }
            let ct = self.contracts.as_ref().and_then(|pool| pool.contract_for_staff(p.id));
            let wage = ct.map(|r| r.wage).unwrap_or_else(|| pv.wage()) as i64;
            let value = ct.map(|r| r.value).unwrap_or_else(|| pv.value()) as i64;
            let nation = pv.nation_id();
            let nation_name = nation.and_then(|n| self.nation_name(n as u32));
            players.push(P {
                id: p.id,
                name: self.person_display_name(p),
                wage, value,
                age: p.age_at(REF_YEAR, ref_day),
                nation, nation_name,
            });
        }

        let count = players.len();
        let attribute = |v: String, name: &str| format!("{v} - {name}");

        // Injured — the runtime injury book (rehab in progress).
        let injured = players.iter()
            .filter(|p| save.injuries.rehab_progress(p.id).is_some())
            .count();

        // Wages.
        let wage_bill: i64 = players.iter().map(|p| p.wage).sum();
        let avg_wage = if count > 0 { wage_bill / count as i64 } else { 0 };
        let highest_wage = players.iter().max_by_key(|p| p.wage)
            .map(|p| attribute(fmt_wage(p.wage), &p.name)).unwrap_or_else(|| "-".into());
        let lowest_wage = players.iter().min_by_key(|p| p.wage)
            .map(|p| attribute(fmt_wage(p.wage), &p.name)).unwrap_or_else(|| "-".into());

        // Ages (only players with a known DOB contribute).
        let aged: Vec<&P> = players.iter().filter(|p| p.age.is_some()).collect();
        let avg_age_squad = if aged.is_empty() {
            "-".to_string()
        } else {
            let mean = aged.iter().map(|p| p.age.unwrap() as f64).sum::<f64>() / aged.len() as f64;
            format!("{mean:.2}")
        };
        let oldest = aged.iter().max_by_key(|p| p.age.unwrap())
            .map(|p| attribute(p.age.unwrap().to_string(), &p.name)).unwrap_or_else(|| "-".into());
        let youngest = aged.iter().min_by_key(|p| p.age.unwrap())
            .map(|p| attribute(p.age.unwrap().to_string(), &p.name)).unwrap_or_else(|| "-".into());

        // Value.
        let highest_valued = players.iter().max_by_key(|p| p.value)
            .map(|p| attribute(fmt_value_full(p.value), &p.name)).unwrap_or_else(|| "-".into());

        // Nationality counts.
        let foreign = players.iter()
            .filter(|p| p.nation.is_some() && p.nation != club_nation).count();
        let non_eu = players.iter()
            .filter(|p| match &p.nation_name { Some(n) => !is_eu_nation_name(n), None => false })
            .count();

        // First-team rows and current call-ups: the not-managed view has
        // no selected XI (First-Team avg age "-", wage bill £0), and no
        // international/U21 call-ups have been made in this state. These
        // wire to the selection / national-squad engines once those drive
        // this screen (see decode report).
        Some(GeneralInfoStatsView {
            club_id,
            number_of_players: count.to_string(),
            number_injured: injured.to_string(),
            avg_age_first_team: "-".to_string(),
            avg_age_squad,
            wage_bill_first_team: fmt_wage(0),
            wage_bill_squad: fmt_wage(wage_bill),
            average_wage: fmt_wage(avg_wage),
            highest_wage,
            lowest_wage,
            oldest_player: oldest,
            youngest_player: youngest,
            highest_valued,
            num_international: "0".to_string(),
            num_under21: "0".to_string(),
            num_foreign: foreign.to_string(),
            num_non_eu: non_eu.to_string(),
        })
    }
}

#[cfg(test)]
mod stats_format_tests {
    use super::{fmt_wage, fmt_value_full, is_eu_nation_name};
    #[test]
    fn wages_match_capture() {
        assert_eq!(fmt_wage(0), "\u{a3}0");
        assert_eq!(fmt_wage(150), "\u{a3}150");
        assert_eq!(fmt_wage(700), "\u{a3}700");
        assert_eq!(fmt_wage(2000), "\u{a3}2K");
        assert_eq!(fmt_wage(18500), "\u{a3}18.5K");
    }
    #[test]
    fn value_is_comma_grouped() {
        assert_eq!(fmt_value_full(350000), "\u{a3}350,000");
        assert_eq!(fmt_value_full(0), "\u{a3}0");
        assert_eq!(fmt_value_full(1_250_000), "\u{a3}1,250,000");
    }
    #[test]
    fn german_is_eu_brazilian_is_not() {
        assert!(is_eu_nation_name("Germany"));
        assert!(is_eu_nation_name("England"));
        assert!(!is_eu_nation_name("Brazil"));
    }
}

#[cfg(test)]
mod common_name_tests {
    use std::path::PathBuf;

    fn rust_db_dir() -> Option<PathBuf> {
        let dir = std::env::var("CM_RUST_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rust-db")
            });
        if dir.join("metadata.json").exists() { Some(dir) } else { None }
    }

    /// The Burnley scout's disk record carries first_name_id 12731 ("L.")
    /// and second_name_id 19094 ("Catlow"), but common_name_id 1584
    /// ("Liz Catlow"). The original renders the common name, so our
    /// resolver must too — never "L. Catlow".
    #[test]
    fn liz_catlow_uses_common_name() {
        let Some(dir) = rust_db_dir() else {
            eprintln!("rust-db not present locally; skipping Liz Catlow check");
            return;
        };
        let world = crate::World::read_rust_db_dir(&dir).expect("read rust-db");
        let person = world.staff.type6.iter()
            .find(|p| p.id == 52277)
            .expect("Catlow staff record 52277");
        let name = world.person_display_name(person);
        assert_eq!(name, "Liz Catlow", "common-name override must win over first+second");
    }
}
