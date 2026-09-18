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
fn role_for_job(job: u8) -> &'static str {
    match job {
        1 => "Chairman",
        5 => "Manager",
        10 => "Physio",
        _ => "Staff",
    }
}

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
            if pv.is_player() { continue; }
            let job = pv.club_job();
            let first = self.references.first_names
                .get(p.first_name_id() as usize).map(|n| n.text.as_str()).unwrap_or("");
            let second = self.references.second_names
                .get(p.second_name_id() as usize).map(|n| n.text.as_str()).unwrap_or("");
            let name = match (first.is_empty(), second.is_empty()) {
                (true, true) => continue,
                (true, false) => second.to_string(),
                (false, true) => first.to_string(),
                (false, false) => format!("{first} {second}"),
            };
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
