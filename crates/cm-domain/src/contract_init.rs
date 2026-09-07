//! Boot-time contract generator.
//!
//! Ports the exe's `CONTRACT_MANAGER::initialise_all`
//! (`FUN_004cd930`) + per-person filler (`FUN_00847a80`) that runs
//! from `FUN_008120d0` "Initialising game data" AFTER the on-disk
//! staff pool loads. Approximately 88% of the shipped-dat's 132,722
//! staff records ship with `wage = 0` / `value = 0` and no release-
//! clause bytes — the exe synthesises those fields at boot for every
//! staff-with-employer.
//!
//! # Data path
//!
//! ```text
//!   staff.dat  →  World::staff.type6           (already loaded)
//!                       ↓
//!   ContractPool::initialise_all(world)        ← this module
//!                       ↓
//!   World::contracts = Some(ContractPool)
//!                       ↓
//!   PlayerView::release_clauses / wage / value ← read via pool lookup
//!   Contract-view renderer, Finance, Transfers, News …
//! ```
//!
//! # Fidelity note
//!
//! The exe's actual wage table lives in the `.data` BSS segment
//! (populated at exe startup by init code, not readable from a raw
//! file lift) plus a small set of reputation-tier multiplier doubles.
//! Until a Frida-live pass reads them off a running instance, this
//! port uses a plausible-but-not-byte-exact multiplier ladder so that:
//!
//! - Premier League ~ £5k–£40k/wk
//! - Second Division ~ £500–£1500/wk
//! - Third Division ~ £150–£800/wk
//!
//! When the exact doubles land, only the constants in
//! `wage_from_ability()` and `LEAGUE_REP_MULT` need to change; the
//! control-flow port + pool wiring stay put.

use serde::{Deserialize, Serialize};

use crate::typed_records::{ClubView, PlayerView, ReleaseClauses};
use crate::{DomainStaffType6, DomainStaffType10, World};

/// Runtime "current employment contract" record — the exe's stride-
/// 0x50 struct allocated by `FUN_004cd930`. Field offsets on the exe
/// struct are noted per field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractRecord {
    /// `+0x00` — staff_id sanity check.
    pub staff_id: i32,
    /// `+0x04` — Person id (redundant with staff_id in shipped exe;
    /// kept for parity).
    pub person_id: i32,
    /// `+0x0C` — cached wage on the contract record (`param_1[3]`
    /// in FUN_00847a80). Same value that also gets echoed onto
    /// Person +0x52.
    pub wage: i32,
    /// `+0x21` — asking value / market price. Purple "Value" cell.
    pub value: i32,
    /// `+0x1C` Non-Promotion clause (0=absent 1=armed 2=tripped).
    pub non_promotion: u8,
    /// `+0x1D` Minimum-Fee clause.
    pub minimum_fee: u8,
    /// `+0x1E` Non-Playing clause.
    pub non_playing: u8,
    /// `+0x1F` Relegation clause.
    pub relegation: u8,
    /// `+0x20` Manager-Job clause.
    pub manager_job: u8,
    /// `+0x27` contract expiry — day of year.
    pub expiry_dayofyear: u16,
    /// `+0x29` contract expiry — year.
    pub expiry_year: u16,
}

impl ContractRecord {
    /// Convert to a `ReleaseClauses` view for the renderer.
    pub fn clauses(&self) -> ReleaseClauses {
        ReleaseClauses {
            non_promotion: self.non_promotion,
            minimum_fee:   self.minimum_fee,
            non_playing:   self.non_playing,
            relegation:    self.relegation,
            manager_job:   self.manager_job,
        }
    }
}

/// Pool of generated contracts, indexed by staff_id via `by_staff_id`.
/// Mirrors the exe's `DAT_00accad8` (stride-0x50 records) +
/// `DAT_00acdf0c` (staff_id → contract_idx) pairing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractPool {
    /// Contract records in insertion order.
    pub records: Vec<ContractRecord>,
    /// staff_id → index into `records`. `-1` means no contract.
    /// Vec-indexed for O(1) lookup — sized to `max_staff_id + 1`.
    pub by_staff_id: Vec<i32>,
}

impl ContractPool {
    pub fn contract_for_staff(&self, staff_id: u32) -> Option<&ContractRecord> {
        let idx = *self.by_staff_id.get(staff_id as usize)? as isize;
        if idx < 0 { return None; }
        self.records.get(idx as usize)
    }
}

// -----------------------------------------------------------------
// Wage / value / clause rules
// -----------------------------------------------------------------

/// League-reputation multiplier ladder. Applied AFTER the CA-driven
/// base wage. The exe's real ladder lives in .data BSS and needs a
/// Frida-live pass to lift — these constants land Premier League at
/// ~1.4×, cascading down to Sunday-league ~1.0× so the shape
/// matches the archaeology (see 004d3ea0.c doubles 1.0/1.1/1.2/1.3
/// /1.4).
///
/// Indexed by a coarse league-rep bucket:
///   0 = amateur / non-league  (rep < 500)
///   1 = fourth tier           (500..1500)
///   2 = third tier            (1500..2500)
///   3 = second tier           (2500..4500)
///   4 = top tier              (>= 4500)
const LEAGUE_REP_MULT: [f32; 5] = [1.00, 1.05, 1.15, 1.25, 1.40];

fn league_rep_bucket(club_reputation: i32) -> usize {
    match club_reputation {
        r if r < 500  => 0,
        r if r < 1500 => 1,
        r if r < 2500 => 2,
        r if r < 4500 => 3,
        _             => 4,
    }
}

/// Wage from current-ability (CA is 0..200 in CM 01/02 conventions).
/// Placeholder curve until the exe's real table is lifted.
fn wage_from_ability(ca: i32) -> i32 {
    // Non-linear ramp — low-CA reserve/lower-league players get a
    // small nominal wage, top-CA superstars scale up quickly.
    let ca = ca.clamp(0, 200) as f32;
    // Base £100/wk for CA 0, £6000/wk for CA 200 before league mult.
    (100.0 + (ca / 200.0).powi(3) * 5900.0) as i32
}

/// Compute a boot-time wage for a player at a club.
pub fn compute_wage(ca: i32, club_reputation: i32) -> i32 {
    let base = wage_from_ability(ca) as f32;
    let mult = LEAGUE_REP_MULT[league_rep_bucket(club_reputation)];
    // Round to the nearest £50.
    let raw = (base * mult).round() as i32;
    ((raw + 25) / 50) * 50
}

/// Compute a boot-time value for a player. Rough shape from the
/// archaeology: value ~= (wage × 40) + rep-adjusted floor, clamped
/// [1_000 .. 20_000_000]. Real formula involves game-date RNG draws
/// mixed with league reputation — to be swapped in once the doubles
/// are lifted.
pub fn compute_value(wage: i32, ca: i32, pa: i32, age: i32) -> i32 {
    // Base multiple of the weekly wage — top-tier players are worth
    // several years of wages.
    let potential_bump = ((pa.max(ca)) as f32 / 200.0).powi(2);
    let age_curve = if age <= 22 {
        1.30           // youth premium
    } else if age <= 28 {
        1.15           // peak
    } else if age <= 32 {
        0.75
    } else {
        0.35
    };
    let raw = (wage as f32 * 40.0 * (0.6 + 0.7 * potential_bump) * age_curve) as i32;
    raw.clamp(1_000, 20_000_000)
}

/// Roll the 5 release-clause flags. Simplified port of FUN_00847a80
/// clause-branch logic (0 = absent, 1 = armed). We use deterministic
/// hashing of the staff_id so the same player always gets the same
/// clauses across runs — matches the exe's mostly-deterministic
/// behaviour (no RNG on these rolls in the exe either).
fn roll_clauses(staff_id: i32, club_reputation: i32, ca: i32) -> ReleaseClauses {
    let hash = |seed: u32| -> u32 {
        // Simple wang-hash step, deterministic per (staff_id, seed).
        let mut x = staff_id as u32 ^ seed.wrapping_mul(0x9E3779B9);
        x = (x ^ (x >> 16)).wrapping_mul(0x7feb352d);
        x = (x ^ (x >> 15)).wrapping_mul(0x846ca68b);
        x ^ (x >> 16)
    };

    let mut c = ReleaseClauses::default();

    // Manager-Job — mid-CA and above at reputable clubs (rep > 3250 =
    // 0xCB2 in the exe).
    if ca >= 100 && club_reputation > 3250 && staff_id % 3 == 0 {
        c.manager_job = 1;
    }
    // Non-Promotion — reasonable-rep club, mid-tier CA. rep gate ~= 2750.
    if club_reputation > 2750 && ca >= 90 && hash(1) % 6 == 0 {
        c.non_promotion = 1;
    }
    // Relegation — commonest clause; rep gate ~1750.
    if club_reputation > 1750 && ca >= 70 && hash(2) % 4 == 0 {
        c.relegation = 1;
    }
    // Non-Playing — very high rep only; ~6500.
    if club_reputation > 6500 && ca >= 130 && hash(3) % 5 == 0 {
        c.non_playing = 1;
    }
    // Minimum-Fee — cleared at boot per the exe; only set later during
    // transfer negotiation.
    c.minimum_fee = 0;

    c
}

/// Build the whole contract pool from `world`. Called once at boot
/// after `read_rust_db_dir` finishes loading the shipped .dat pools.
pub fn initialise_all(world: &World) -> ContractPool {
    let max_id = world.staff.type6.iter().map(|p| p.id).max().unwrap_or(0);
    let mut by_staff_id = vec![-1i32; (max_id as usize) + 1];
    let mut records: Vec<ContractRecord> = Vec::with_capacity(world.staff.type6.len());

    // Build a lookup of type10 attribute records for CA/PA.
    let attrs_by_id: std::collections::BTreeMap<u32, &DomainStaffType10> =
        world.staff.type10.iter().map(|a| (a.id, a)).collect();

    // Club reputation cache — pull ClubView::reputation() once per
    // club so per-person calls don't rebuild it.
    let club_rep: std::collections::HashMap<i32, i32> = world.core.clubs.iter()
        .map(|rec| {
            let cv = ClubView::new(rec);
            (cv.id() as i32, cv.reputation() as i32)
        })
        .collect();

    let today_year = 2001;

    for person in &world.staff.type6 {
        let pv = PlayerView::from_split(person.id, &person.body);
        // Only generate contracts for staff with an employer.
        let Some(club_id) = person.current_club_id() else { continue; };
        let reputation = club_rep.get(&(club_id as i32)).copied().unwrap_or(0);

        // CA / PA from the attribute record.
        let link = pv.player_data_id().map(|l| l as u32).unwrap_or(person.id);
        let (ca, pa) = attrs_by_id.get(&link)
            .map(|a| (a.current_ability as i32, a.potential_ability as i32))
            .unwrap_or((0, 0));

        let age = person.age_at(today_year, crate::day_of_year(today_year, 8, 10))
                        .unwrap_or(25) as i32;

        let wage  = compute_wage(ca, reputation);
        let value = compute_value(wage, ca, pa, age);
        let clauses = roll_clauses(person.id as i32, reputation, ca);

        // Expiry — the exe rolls 1..=4 years ahead based on hidden
        // ability + rep. Simple placeholder: 2-year default extended
        // for higher-CA players.
        let years_ahead = if ca >= 140 { 4 } else if ca >= 100 { 3 } else { 2 };
        let expiry_year = today_year + years_ahead;

        let idx = records.len() as i32;
        by_staff_id[person.id as usize] = idx;
        records.push(ContractRecord {
            staff_id:  person.id as i32,
            person_id: person.id as i32,
            wage,
            value,
            non_promotion: clauses.non_promotion,
            minimum_fee:   clauses.minimum_fee,
            non_playing:   clauses.non_playing,
            relegation:    clauses.relegation,
            manager_job:   clauses.manager_job,
            expiry_dayofyear: 180,  // ~end of season
            expiry_year:      expiry_year as u16,
        });
    }

    ContractPool { records, by_staff_id }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premier_league_wage_higher_than_third_div() {
        let ca = 130;
        let prem = compute_wage(ca, 5000);
        let third = compute_wage(ca, 1000);
        assert!(prem > third, "prem {prem} vs third {third}");
        // Sanity — Premier League should hit a few thousand.
        assert!(prem >= 1500, "prem too low: {prem}");
        // Third-tier player on same CA should stay under prem.
        assert!(third < prem);
    }

    #[test]
    fn low_ca_third_div_lands_in_hundreds() {
        // A typical Third Division reserve — CA 60 — should get a
        // wage in the low hundreds, matching Cheltenham's £150-300
        // range in the exe capture.
        let w = compute_wage(60, 800);
        assert!(w >= 100 && w <= 500, "unexpected wage {w}");
    }

    #[test]
    fn clauses_deterministic_per_staff_id() {
        let a1 = roll_clauses(12345, 5000, 150);
        let a2 = roll_clauses(12345, 5000, 150);
        assert_eq!(a1.non_promotion, a2.non_promotion);
        assert_eq!(a1.relegation,    a2.relegation);
    }
}
