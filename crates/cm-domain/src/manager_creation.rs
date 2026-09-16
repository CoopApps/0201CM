//! Manager creation flow — port of the post-init "create your manager" screens.
//!
//! Screen sequence (VAs from [[manager-creation-flow]] memory):
//!
//! 1. **Manager Status** — `FUN_00809ad0` allocates a fresh person record and
//!    registers a human seat via `FUN_005e5330` into `DAT_00b59fc2[active*0xc0]`.
//! 2. **Enter Name** — draw `0x00809cc0` / ev `0x0080a450`. Four fields:
//!    First Name, Second Name, Password, Re-Type. Names type in place into
//!    the reserved staff slot; password is validated `password == retype` and
//!    both names must be non-empty. Modelled here by [`ManagerNameEntryView`].
//! 3. **Select Nationality** — draw `0x0080a880` / ev `0x0080b0a0`. Lists
//!    nation records (`DAT_00acd5b0`, stride 0x122) where `nation+0x71 != 0`
//!    (has a continent). Filter has two modes: **Major Nations** (hard-coded
//!    id list at `0x009bb6d8`) or **All Nations**. Pick sets `staff+0x1a`
//!    (the person's nation pointer). Modelled by [`ManagerNationalitySelectView`].
//! 4. **Select Team** — draw `0x0080b2b0` / ev `0x0080ba60`. Walks the club
//!    pool (`DAT_00acd5bc`, stride 0x245) and keeps clubs where
//!    - `club+0x53 == selected_nation` (nation), AND
//!    - `FUN_0052e370(club) != 0` — the league filter: the club's primary
//!      competition (`club+0x57`) must be in the selected-leagues table
//!      (`DAT_00ac688c`) AND the club's nation record must have
//!      `nation+0x11c & 3 != 0` (foreground or background — selected in some
//!      form), AND
//!    - `FUN_00525450(club) == 0` — not a national team (already implicit here
//!      because we walk `world.core.clubs`, not `nat_clubs`).
//!    Modelled by [`ManagerClubSelectView`].
//! 5. **Take Control** — clicking a club fires `FUN_00810f50(staff, club)`:
//!    reputation = 20, wage = attendance/10 + 3000, seat installed via
//!    `FUN_00683e30`. Modelled by [`create_manager`], which sets `world.active_human`
//!    to the newly seated slot (the exe's `DAT_00b5d016 * 0xc0` offset into
//!    the seat pool `DAT_00b59fc2`).

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::human_manager::{HumanSeatPool, SEAT_MAX};
use crate::typed_records::{ClubView, NationView};
use crate::{
    DomainOpaqueRecord, HumanManager, ManagerIdentity, RuntimeSaveGame,
};

// ────────────────────────────────────────────────────────────────────────
// Screen 2 — Enter Name
// ────────────────────────────────────────────────────────────────────────

/// The four editable fields on the Enter Name screen (draw `0x00809cc0`,
/// event `0x0080a450`). Password/re-type live in the human_manager pool
/// (`DAT_00b4bba8`); first/second live in the seat's person record.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagerNameEntryView {
    pub first_name: String,
    pub second_name: String,
    pub password: String,
    pub retype: String,
}

/// Why an Enter-Name submission is rejected. Ported from the exe's Next
/// button gate (empty-name check + password compare).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameEntryError {
    FirstNameEmpty,
    SecondNameEmpty,
    PasswordMismatch,
}

impl ManagerNameEntryView {
    /// Fresh screen — all four fields blank, as `FUN_00809ad0` leaves them
    /// after zeroing `staff+4/+8` and clearing the password slot.
    pub fn new() -> Self {
        Self::default()
    }

    /// The "Next" button gate. Returns `Ok` when both names are non-blank
    /// and `password == retype` (password is allowed to be blank — the
    /// screen labels it "Password (Optional)").
    pub fn validate(&self) -> Result<(), NameEntryError> {
        if self.first_name.trim().is_empty() {
            return Err(NameEntryError::FirstNameEmpty);
        }
        if self.second_name.trim().is_empty() {
            return Err(NameEntryError::SecondNameEmpty);
        }
        if self.password != self.retype {
            return Err(NameEntryError::PasswordMismatch);
        }
        Ok(())
    }

    /// Convenience: build the [`ManagerIdentity`] the world model stores.
    /// Fails with the same rules as [`Self::validate`].
    pub fn into_identity(self) -> Result<ManagerIdentity, NameEntryError> {
        self.validate()?;
        Ok(ManagerIdentity {
            first: self.first_name,
            second: self.second_name,
            nickname: String::new(),
        })
    }
}

// ────────────────────────────────────────────────────────────────────────
// Screen 3 — Select Nationality
// ────────────────────────────────────────────────────────────────────────

/// Filter mode for the nationality picker. Default is Major.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NationalityFilter {
    /// Hard-coded "Major Nations" id list (`0x009bb6d8` in the exe).
    /// The exact list isn't decoded here; caller supplies it.
    Major(BTreeSet<u32>),
    /// All nations with `nation+0x71 != 0` (has a continent).
    All,
}

impl Default for NationalityFilter {
    fn default() -> Self { NationalityFilter::All }
}

/// A single row in the nationality picker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NationalityEntry {
    pub nation_id: u32,
    pub name: String,
    pub continent_id: i32,
    /// True when this nation appears in the Major Nations hard-coded list
    /// (only meaningful if the caller passed one; else always false).
    pub is_major: bool,
}

/// The Select Nationality screen — draw `0x0080a880`, event `0x0080b0a0`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagerNationalitySelectView {
    pub nations: Vec<NationalityEntry>,
    /// The filter that produced this list.
    pub filter: NationalityFilterTag,
}

/// Serializable tag of the filter mode (we don't keep the full id set on the
/// view — it's a build input, not view state).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NationalityFilterTag {
    Major,
    #[default]
    All,
}

impl ManagerNationalitySelectView {
    /// Port of the screen builder: walk the nation pool, keep those with a
    /// continent (exe rule `nation+0x71 != 0`), then apply the filter.
    pub fn build(nations: &[DomainOpaqueRecord], filter: &NationalityFilter) -> Self {
        let mut out = Vec::new();
        for rec in nations {
            if rec.raw.len() < NationView::RECORD_SIZE {
                continue;
            }
            let nv = NationView::new(rec);
            // Exe filter: nation+0x71 != 0 (continent present)
            // Africa is continent id 0 — the exe still lists it because the
            // decompile compares the *byte at +0x71*, which is 0 for Africa,
            // but the actual filter dereferences the swizzled pointer and is
            // "has a continent object at all". Base-data Africa nations DO
            // have +0x71 == 0 and DO appear in the picker, so we do NOT skip
            // continent==0 here; we only require the record to be populated.
            if nv.id() == 0 && nv.primary_name().is_empty() {
                continue;
            }
            let (is_major, keep) = match filter {
                NationalityFilter::All => (false, true),
                NationalityFilter::Major(set) => {
                    let m = set.contains(&nv.id());
                    (m, m)
                }
            };
            if !keep {
                continue;
            }
            out.push(NationalityEntry {
                nation_id: nv.id(),
                name: nv.primary_name(),
                continent_id: nv.continent_id(),
                is_major,
            });
        }
        // Alphabetical by name — the exe orders the list this way (name-sort
        // helper `FUN_005d7710` is called by the picker builder).
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            nations: out,
            filter: match filter {
                NationalityFilter::All => NationalityFilterTag::All,
                NationalityFilter::Major(_) => NationalityFilterTag::Major,
            },
        }
    }
}

// ────────────────────────────────────────────────────────────────────────
// Screen 4 — Select Team (club picker)
// ────────────────────────────────────────────────────────────────────────

/// Port of `FUN_0052e370` — the "is this club pickable" gate that also drives
/// the club-select screen and the club toolbar's "Take Control" button.
///
/// A club passes when:
/// 1. It has a nation (`club+0x53 != 0`).
/// 2. Its primary competition (`club+0x57`) is one of the selected leagues
///    (i.e. present in `selected_league_ids`, the Rust equivalent of the
///    `DAT_00ac688c[comp_id]` table set up from the league picker).
/// 3. That club's nation has `nation+0x11c & 3 != 0` — the selection-flags
///    byte, meaning the nation is Foreground or Background (i.e. selected in
///    some form).
///
/// The two "regular club vs national team" branches of the decompile are
/// collapsed here: we only run against `world.core.clubs`, so the
/// `FUN_00525450(club) == 0` "not a national team" check is implicit.
pub fn club_is_pickable(
    club: &ClubView,
    nation: Option<&NationView>,
    selected_league_ids: &BTreeSet<u32>,
) -> bool {
    let Some(nid) = club.nation_id() else { return false; };
    if nid <= 0 { return false; }
    let Some(div) = club.division_id() else { return false; };
    if div <= 0 { return false; }
    if !selected_league_ids.contains(&(div as u32)) {
        return false;
    }
    // Nation-side gate: selection_flags & 3 (foreground or background).
    match nation {
        Some(nv) => nv.selection_flags() & 3 != 0,
        None => false,
    }
}

/// A single row in the club picker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubPickerEntry {
    pub club_id: u32,
    pub name: String,
    pub division_id: u32,
    pub reputation: u16,
}

/// The Select Team screen — draw `0x0080b2b0`, event `0x0080ba60`.
///
/// Laid out as two clubs per row × three columns in the exe; the view just
/// carries the flat list, sorted alphabetically the way `FUN_005d7710` does.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagerClubSelectView {
    pub nation_id: u32,
    pub clubs: Vec<ClubPickerEntry>,
}

impl ManagerClubSelectView {
    /// Build the pickable club list for `selected_nation_id`, restricting to
    /// the caller's `selected_league_ids` (from the league picker screen).
    pub fn build(
        clubs: &[DomainOpaqueRecord],
        nations: &[DomainOpaqueRecord],
        selected_nation_id: u32,
        selected_league_ids: &BTreeSet<u32>,
    ) -> Self {
        // Look up the nation record once so we can gate on its +0x11c flags.
        let nation_rec = nations.iter().find(|r| {
            r.raw.len() >= NationView::RECORD_SIZE
                && NationView::from_bytes(&r.raw).id() == selected_nation_id
        });
        let nation_view = nation_rec.map(|r| NationView::new(r));

        let mut out = Vec::new();
        for rec in clubs {
            if rec.raw.len() < ClubView::RECORD_SIZE {
                continue;
            }
            let cv = ClubView::new(rec);
            // Club must be in this nation.
            let Some(nid) = cv.nation_id() else { continue; };
            if nid as u32 != selected_nation_id { continue; }
            // FUN_0052e370 gate.
            if !club_is_pickable(&cv, nation_view.as_ref(), selected_league_ids) {
                continue;
            }
            let div = cv.division_id().unwrap_or(0) as u32;
            out.push(ClubPickerEntry {
                club_id: cv.id(),
                name: cv.primary_name(),
                division_id: div,
                reputation: cv.reputation(),
            });
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Self { nation_id: selected_nation_id, clubs: out }
    }
}

// ────────────────────────────────────────────────────────────────────────
// Screen 5 — Take Control (create_manager)
// ────────────────────────────────────────────────────────────────────────

/// The outcome of a full create-manager flow: the new human's index in
/// `world.humans`, the seat index in the (optional) [`HumanSeatPool`] the
/// exe stores at `DAT_00b59fc2[seat * 0xc0]`, and the reputation/wage the
/// take-control logic assigned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerCreationOutcome {
    /// Index into `world.humans` — this is also written to
    /// `world.active_human` (the exe's `DAT_00b5d016`).
    pub human_index: usize,
    /// Seat index in the [`HumanSeatPool`] this call wrote into (0..16),
    /// or `None` if no pool was supplied.
    pub seat_index: Option<usize>,
    /// The reputation the take-control step assigned — always 20 in this
    /// version (no "Sunday League Footballer" ladder).
    pub reputation: u8,
    /// The weekly wage the take-control step computed (attendance/10 + 3000
    /// in the exe; caller supplies the club's average attendance).
    pub wage: u32,
}

/// Errors from [`create_manager`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateManagerError {
    /// The name-entry screen inputs didn't validate.
    Name(NameEntryError),
    /// No free human seat (`FUN_00810f50` returns 0 when the 16-seat table
    /// is full).
    SeatPoolFull,
    /// The chosen club is already managed by another seated human.
    ClubAlreadyHumanManaged,
}

impl From<NameEntryError> for CreateManagerError {
    fn from(e: NameEntryError) -> Self { CreateManagerError::Name(e) }
}

/// Full create-manager flow (screens 2 → 5), collapsed into one call.
///
/// Ports `FUN_00810f50`'s effects onto the ported human-manager model:
/// 1. Validates the name entry.
/// 2. Appends a new [`HumanManager`] to `save.humans` and moves
///    `save.active_human` to that index — the exe's `DAT_00b5d016` write.
/// 3. Calls [`RuntimeSaveGame::install_manager_at_club`], which sets club,
///    reputation (=20), and promotes the club's nation to Foreground.
/// 4. If a raw [`HumanSeatPool`] is supplied, also writes a seat via
///    [`HumanSeatPool::take_control`] at index = the new human index — that
///    seat is the byte-exact `DAT_00b59fc2[i * 0xc0]` record.
///
/// `attendance` is the club's average attendance, from which the wage
/// formula `attendance/10 + 3000` is computed. `today_days` is days-since-
/// epoch to stamp on the seat's `joined_date`.
///
/// Note: named `create_manager(world, …)` in the design doc, but on our
/// data model the humans live on `RuntimeSaveGame` (the save overlay), not
/// on the immutable-base `World` — the two together are "the world" the
/// game sees at runtime.
pub fn create_manager(
    save: &mut RuntimeSaveGame,
    name: ManagerNameEntryView,
    nationality: u32,
    club_id: u32,
    club_nation: Option<u32>,
    attendance: u32,
    today_days: u32,
    seat_pool: Option<&mut HumanSeatPool>,
) -> Result<ManagerCreationOutcome, CreateManagerError> {
    let identity = name.into_identity()?;
    // Wage formula from FUN_00810f50 (short arithmetic, but the effect is
    // just `attendance / 10 + 3000`).
    let wage: u32 = attendance / 10 + 3000;
    const REP_ON_TAKEOVER: u8 = 20;

    // 1) Append & make active.
    let mut human = HumanManager::new(identity);
    human.nation = None; // nationality-of-birth is a person attribute; job
                        // nation stays None until they take a national side.
    let _ = nationality; // kept in signature for future person+0x1a linkage
    save.humans.push(human);
    let human_index = save.humans.len() - 1;
    save.active_human = human_index;

    // 2) Install at club — this sets reputation=20 and promotes the club's
    //    nation to Foreground (the tier-consequence of the link).
    let installed = save.install_manager_at_club(human_index, club_id, club_nation);
    if !installed {
        // Only failure mode is index-out-of-range, which we just avoided.
        // Any future stricter check (already managed etc.) would land here.
    }

    // 3) Optional byte-exact seat write.
    let seat_index = if let Some(pool) = seat_pool {
        let seat_i = human_index.min(SEAT_MAX - 1);
        // Person id — we don't have a person_id here (person records aren't
        // materialised for humans in the ported model), so we use
        // `(human_index + 1) as u32` as a stand-in — deterministic and
        // distinct across humans. Real ports would use the reserved
        // staff-slot id from `FUN_00809ad0`.
        let person_id = (human_index as u32).wrapping_add(1);
        let idx = pool
            .take_control(Some(seat_i), person_id, club_id, today_days, wage, REP_ON_TAKEOVER as u16)
            .map_err(|_e| CreateManagerError::ClubAlreadyHumanManaged)?;
        // Track active seat too (DAT_00b5d016).
        let _ = pool.switch_active(idx as u8);
        Some(idx)
    } else {
        None
    };

    Ok(ManagerCreationOutcome {
        human_index,
        seat_index,
        reputation: REP_ON_TAKEOVER,
        wage,
    })
}

// ────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::human_manager::HumanSeatPool;
    use crate::{
        GameDate, HeadlessRuntimeState, HeadlessSeasonState,
        RuntimeBackendSystems, RuntimeSaveGame, RuntimeSimulationState,
        RuntimeSource, RuntimeTableCounts, SaveWorldOverlay,
    };

    fn blank_save() -> RuntimeSaveGame {
        RuntimeSaveGame {
            format: "cm0102-rs-save".to_string(),
            version: 1,
            source: RuntimeSource {
                kind: "test".to_string(),
                path: "memory".to_string(),
            },
            date: GameDate { year: 2001, month: 7, day: 1 },
            simulation: RuntimeSimulationState::default(),
            backend: RuntimeBackendSystems::default(),
            headless: HeadlessRuntimeState::default(),
            season: HeadlessSeasonState::default(),
            elapsed_days: 0,
            pending_events: Vec::new(),
            phase_trace: Vec::new(),
            table_counts: RuntimeTableCounts {
                clubs: 0, national_clubs: 0, nations: 0,
                staff_type6: 0, staff_type9: 0, staff_type10: 0,
                cities: 0, stadiums: 0, competitions: 0, histories: 0,
            },
            new_game: None,
            nation_tiers: Vec::new(),
            world: SaveWorldOverlay::default(),
            player_init: None,
            club_tactics: Default::default(),
            scouts: Default::default(),
            humans: Vec::new(),
            active_human: 0,
            african_nations: None,
            asia_club_champ: None,
            asia_cup_winner: None,
            asia_cup_of_nations: None,
            european_championship: None,
            fifa_confederations_cup: None,
            concacaf_gold_cup: None,
            asia_super_cup: None,
            aus_nsl: None,
            aus_salary_cap: None,
            simple_leagues: Vec::new(),
            domestic_cups: Vec::new(),
            super_cups: Vec::new(),
            league_playoffs: Vec::new(),
            disputes: Vec::new(),
            friendlies: Vec::new(),
            fifa_rankings: Vec::new(),
            last_year_rollover: 0,
            last_english_year_end_applied: None,
            session_rng_state: None,
            season_roll_scheduler: {
                let mut s = crate::season_roll_scheduler::SeasonRollScheduler::new();
                crate::season_roll_scheduler::register_english_pyramid(&mut s);
                s
            },
            season_roll_comp_years: std::collections::BTreeMap::new(),
            pending_season_roll_regens: std::collections::BTreeMap::new(),
            season_roll_events: Vec::new(),
            finance_ledger: crate::c15_1_world_apply::ClubFinanceLedger::new(),
            argentine_primera: None,
            argentine_second: None,
            honours: Vec::new(),
            argentine_transfer_rules: None,
            notes: Vec::new(),
            finance: Default::default(),
            player_ratings: Default::default(),
            transfers: Default::default(),
            training: Default::default(),
            injuries: Default::default(),
        }
    }

    // ---- helpers to synthesise raw nation/club records ---------------------

    fn make_nation_raw(id: u32, name: &str, continent: u8, flags: u8) -> Vec<u8> {
        let mut raw = vec![0u8; NationView::RECORD_SIZE];
        raw[0..4].copy_from_slice(&id.to_le_bytes());
        let nb = name.as_bytes();
        let n = nb.len().min(50);
        raw[4..4 + n].copy_from_slice(&nb[..n]);
        raw[0x71] = continent;
        raw[0x11c] = flags;
        raw
    }

    fn make_nation(id: u32, name: &str, continent: u8, flags: u8) -> DomainOpaqueRecord {
        DomainOpaqueRecord {
            ordinal: id,
            id,
            primary_name: Some(name.into()),
            secondary_name: None,
            short_name: None,
            text_candidates: vec![],
            raw: make_nation_raw(id, name, continent, flags),
        }
    }

    fn make_club(id: u32, name: &str, nation_id: i32, division_id: i32, reputation: u16) -> DomainOpaqueRecord {
        let mut raw = vec![0u8; ClubView::RECORD_SIZE];
        raw[0..4].copy_from_slice(&id.to_le_bytes());
        let nb = name.as_bytes();
        let n = nb.len().min(50);
        raw[4..4 + n].copy_from_slice(&nb[..n]);
        // division at 0x52 (unused, leave 0). nation_id at 0x53, div_id at 0x57.
        raw[0x53..0x57].copy_from_slice(&nation_id.to_le_bytes());
        raw[0x57..0x5b].copy_from_slice(&division_id.to_le_bytes());
        raw[0x80..0x82].copy_from_slice(&reputation.to_le_bytes());
        DomainOpaqueRecord {
            ordinal: id,
            id,
            primary_name: Some(name.into()),
            secondary_name: None,
            short_name: None,
            text_candidates: vec![],
            raw,
        }
    }

    // ---- Screen 2: Enter Name ---------------------------------------------

    #[test]
    fn manager_creation_name_validate_ok_no_password() {
        let v = ManagerNameEntryView {
            first_name: "Alex".into(),
            second_name: "Ferguson".into(),
            password: String::new(),
            retype: String::new(),
        };
        assert!(v.validate().is_ok());
    }

    #[test]
    fn manager_creation_name_validate_password_match() {
        let v = ManagerNameEntryView {
            first_name: "A".into(), second_name: "B".into(),
            password: "hunter2".into(), retype: "hunter2".into(),
        };
        assert!(v.validate().is_ok());
    }

    #[test]
    fn manager_creation_name_validate_password_mismatch() {
        let v = ManagerNameEntryView {
            first_name: "A".into(), second_name: "B".into(),
            password: "hunter2".into(), retype: "hunter3".into(),
        };
        assert_eq!(v.validate(), Err(NameEntryError::PasswordMismatch));
    }

    #[test]
    fn manager_creation_name_validate_first_blank() {
        let v = ManagerNameEntryView {
            first_name: "   ".into(), second_name: "B".into(),
            password: String::new(), retype: String::new(),
        };
        assert_eq!(v.validate(), Err(NameEntryError::FirstNameEmpty));
    }

    #[test]
    fn manager_creation_name_validate_second_blank() {
        let v = ManagerNameEntryView {
            first_name: "A".into(), second_name: String::new(),
            password: String::new(), retype: String::new(),
        };
        assert_eq!(v.validate(), Err(NameEntryError::SecondNameEmpty));
    }

    #[test]
    fn manager_creation_name_into_identity_carries_names() {
        let v = ManagerNameEntryView {
            first_name: "Ada".into(), second_name: "Lovelace".into(),
            password: String::new(), retype: String::new(),
        };
        let id = v.into_identity().unwrap();
        assert_eq!(id.first, "Ada");
        assert_eq!(id.second, "Lovelace");
        assert_eq!(id.display_name(), "Ada Lovelace");
    }

    // ---- Screen 3: Select Nationality -------------------------------------

    fn nations_pool() -> Vec<DomainOpaqueRecord> {
        vec![
            make_nation(1, "England", 2, 0),
            make_nation(2, "Scotland", 2, 0),
            make_nation(3, "Brazil", 5, 0),
            make_nation(4, "Argentina", 5, 0),
            make_nation(5, "Nigeria", 0, 0),
        ]
    }

    #[test]
    fn manager_creation_nationality_all_lists_every_named_nation_sorted() {
        let pool = nations_pool();
        let v = ManagerNationalitySelectView::build(&pool, &NationalityFilter::All);
        let names: Vec<_> = v.nations.iter().map(|n| n.name.clone()).collect();
        assert_eq!(names, vec!["Argentina", "Brazil", "England", "Nigeria", "Scotland"]);
        assert!(matches!(v.filter, NationalityFilterTag::All));
    }

    #[test]
    fn manager_creation_nationality_major_filters_to_hard_coded_set() {
        let pool = nations_pool();
        let major: BTreeSet<u32> = [1u32, 3, 4].into_iter().collect();
        let v = ManagerNationalitySelectView::build(&pool, &NationalityFilter::Major(major));
        let names: Vec<_> = v.nations.iter().map(|n| n.name.clone()).collect();
        assert_eq!(names, vec!["Argentina", "Brazil", "England"]);
        assert!(v.nations.iter().all(|n| n.is_major));
        assert!(matches!(v.filter, NationalityFilterTag::Major));
    }

    #[test]
    fn manager_creation_nationality_skips_blank_slots() {
        let mut pool = nations_pool();
        pool.push(DomainOpaqueRecord {
            ordinal: 99, id: 0,
            primary_name: None, secondary_name: None, short_name: None,
            text_candidates: vec![],
            raw: vec![0u8; NationView::RECORD_SIZE],
        });
        let v = ManagerNationalitySelectView::build(&pool, &NationalityFilter::All);
        assert_eq!(v.nations.len(), 5);
    }

    #[test]
    fn manager_creation_nationality_carries_continent_id() {
        let pool = nations_pool();
        let v = ManagerNationalitySelectView::build(&pool, &NationalityFilter::All);
        let nig = v.nations.iter().find(|n| n.name == "Nigeria").unwrap();
        assert_eq!(nig.continent_id, 0);
        let eng = v.nations.iter().find(|n| n.name == "England").unwrap();
        assert_eq!(eng.continent_id, 2);
    }

    // ---- Screen 4: Select Team --------------------------------------------

    fn eng_setup() -> (Vec<DomainOpaqueRecord>, Vec<DomainOpaqueRecord>) {
        // England: bit 2 (foreground). France: bit 0 only? actually we want
        // one nation with flags==0 to prove the +0x11c gate rejects it.
        let nations = vec![
            make_nation(1, "England", 2, 0b010), // foreground
            make_nation(2, "France",  2, 0b000), // not selected
            make_nation(3, "Scotland",2, 0b001), // background — still & 3 != 0
        ];
        let clubs = vec![
            make_club(10, "Man Utd",    1, 100, 8000), // English top flight
            make_club(11, "Arsenal",    1, 100, 7500),
            make_club(12, "Everton",    1, 100, 6000),
            make_club(13, "Wrexham",    1, 200, 1500), // English lower — not selected league
            make_club(20, "PSG",        2, 300, 7000), // French top flight
            make_club(30, "Rangers",    3, 400, 5000), // Scottish
            make_club(40, "Nowhere",   -2, 100, 500),  // extinct — nation<=0
        ];
        (clubs, nations)
    }

    #[test]
    fn manager_creation_club_select_english_top_flight_only() {
        let (clubs, nations) = eng_setup();
        let selected: BTreeSet<u32> = [100u32].into_iter().collect();
        let v = ManagerClubSelectView::build(&clubs, &nations, 1, &selected);
        let names: Vec<_> = v.clubs.iter().map(|c| c.name.clone()).collect();
        assert_eq!(names, vec!["Arsenal", "Everton", "Man Utd"]);
        assert_eq!(v.nation_id, 1);
        // Wrexham (division 200, not selected) is out.
        assert!(!names.iter().any(|n| n == "Wrexham"));
    }

    #[test]
    fn manager_creation_club_select_filters_by_selected_leagues() {
        let (clubs, nations) = eng_setup();
        let selected: BTreeSet<u32> = [100u32, 200].into_iter().collect();
        let v = ManagerClubSelectView::build(&clubs, &nations, 1, &selected);
        let names: Vec<_> = v.clubs.iter().map(|c| c.name.clone()).collect();
        assert_eq!(names, vec!["Arsenal", "Everton", "Man Utd", "Wrexham"]);
    }

    #[test]
    fn manager_creation_club_select_gate_rejects_when_nation_flags_zero() {
        // France's clubs must be rejected even though the division is in
        // selected_league_ids: nation flag byte at +0x11c is 0.
        let (clubs, nations) = eng_setup();
        let selected: BTreeSet<u32> = [300u32].into_iter().collect();
        let v = ManagerClubSelectView::build(&clubs, &nations, 2, &selected);
        assert!(v.clubs.is_empty());
    }

    #[test]
    fn manager_creation_club_select_allows_background_nations() {
        // Scotland flags == 1 (background). Bit test is & 3, so it passes.
        let (clubs, nations) = eng_setup();
        let selected: BTreeSet<u32> = [400u32].into_iter().collect();
        let v = ManagerClubSelectView::build(&clubs, &nations, 3, &selected);
        assert_eq!(v.clubs.len(), 1);
        assert_eq!(v.clubs[0].name, "Rangers");
    }

    #[test]
    fn manager_creation_club_is_pickable_matches_fun_0052e370_shape() {
        let (clubs, nations) = eng_setup();
        let cv = ClubView::new(&clubs[0]); // Man Utd
        let nv = NationView::new(&nations[0]); // England
        let selected: BTreeSet<u32> = [100u32].into_iter().collect();
        assert!(club_is_pickable(&cv, Some(&nv), &selected));
        // Same club, but no matching selected league.
        let selected_none: BTreeSet<u32> = BTreeSet::new();
        assert!(!club_is_pickable(&cv, Some(&nv), &selected_none));
        // Same club, but nation missing.
        assert!(!club_is_pickable(&cv, None, &selected));
    }

    // ---- Screen 5: create_manager -----------------------------------------

    #[test]
    fn manager_creation_create_manager_seats_and_activates() {
        let mut world = blank_save();
        let name = ManagerNameEntryView {
            first_name: "Sam".into(), second_name: "Allardyce".into(),
            password: String::new(), retype: String::new(),
        };
        let out = create_manager(
            &mut world, name, /*nationality*/ 1, /*club_id*/ 10,
            /*club_nation*/ Some(1), /*attendance*/ 20_000, /*today*/ 12345,
            None,
        ).unwrap();
        assert_eq!(out.human_index, 0);
        assert_eq!(world.active_human, 0);
        assert_eq!(out.reputation, 20);
        assert_eq!(out.wage, 20_000 / 10 + 3000); // 5000
        assert_eq!(world.humans.len(), 1);
        assert_eq!(world.humans[0].identity.display_name(), "Sam Allardyce");
        assert_eq!(world.humans[0].club, Some(10));
        assert_eq!(world.humans[0].reputation, 20);
        assert!(out.seat_index.is_none());
    }

    #[test]
    fn manager_creation_create_manager_writes_seat_pool_when_given() {
        let mut world = blank_save();
        let mut pool = HumanSeatPool::empty();
        let name = ManagerNameEntryView {
            first_name: "Sam".into(), second_name: "Allardyce".into(),
            password: String::new(), retype: String::new(),
        };
        let out = create_manager(
            &mut world, name, 1, 10, Some(1), 20_000, 12345, Some(&mut pool),
        ).unwrap();
        let idx = out.seat_index.expect("seat_index");
        assert_eq!(idx, 0);
        let seat = &pool.seats[idx];
        assert!(seat.is_occupied());
        assert!(seat.is_employed());
        assert_eq!(seat.club_id(), 10);
        assert_eq!(seat.joined_date(), 12345);
        assert_eq!(seat.wage(), 5000);
        assert_eq!(seat.reputation(), 20);
        assert_eq!(pool.active_human() as usize, idx);
    }

    #[test]
    fn manager_creation_create_manager_rejects_bad_name() {
        let mut world = blank_save();
        let name = ManagerNameEntryView {
            first_name: String::new(), second_name: "X".into(),
            password: String::new(), retype: String::new(),
        };
        let err = create_manager(&mut world, name, 1, 10, Some(1), 20_000, 0, None).unwrap_err();
        assert_eq!(err, CreateManagerError::Name(NameEntryError::FirstNameEmpty));
        assert!(world.humans.is_empty());
    }

    #[test]
    fn manager_creation_create_manager_rejects_password_mismatch() {
        let mut world = blank_save();
        let name = ManagerNameEntryView {
            first_name: "A".into(), second_name: "B".into(),
            password: "x".into(), retype: "y".into(),
        };
        let err = create_manager(&mut world, name, 1, 10, Some(1), 20_000, 0, None).unwrap_err();
        assert_eq!(err, CreateManagerError::Name(NameEntryError::PasswordMismatch));
    }
}
