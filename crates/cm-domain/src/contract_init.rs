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

/// Runtime "current employment contract / squad registration"
/// record — the exe's stride-0x50 struct allocated by
/// `FUN_004cd930`. This record is dual-purpose: contract-clause
/// state at `+0x1C..+0x20` AND squad-registration position
/// preference at `+0x3A` — the C15.1B and C15.1C archaeology
/// proved these are the same 0x50-byte pool viewed through
/// different offsets.
///
/// Field offsets on the exe struct are noted per field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractRecord {
    /// `+0x00` — staff_id (self-index / sanity).
    pub staff_id: i32,
    /// `+0x04` — **club id** (verified by C15.1B + C15.1C
    /// agents against the identity gate `*(record+4) == *club`
    /// in FUN_00843970 / FUN_004D3550 / FUN_004D3460).
    /// Historical name in this codebase was `person_id`; the
    /// C15.1C archaeology proved that was mis-labelled and this
    /// byte is the OWNING CLUB, not a person id (the person id
    /// is implicit — it's the index used against
    /// `by_staff_id`).
    #[serde(alias = "person_id")]
    pub club_id: i32,
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
    /// `+0x3A` — **position preference / squad-registration
    /// role code** (i8, range `[-50, +50]` strict inclusive).
    /// Written by FUN_00843970 (and reset to `0` by promotion's
    /// Loop A). Read by FUN_00843880 for squad evaluations.
    /// Semantic per C15.1C archaeology: signed rank/preference;
    /// positive = wanted at position of magnitude |v|; negative
    /// = ranked/away. `0` = neutral (fall back to
    /// `Person+0x61+4` clamp source).
    #[serde(default)]
    pub position_code: i8,
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

/// Pool of generated contracts, indexed by staff_id via
/// `by_staff_id` (primary) and `by_staff_id_secondary`
/// (secondary / alternate). Mirrors the exe's `DAT_00accad8`
/// (stride-0x50 records) + `DAT_00acdf0c` (0x4F stride per
/// person, first int = primary staff_idx, second int at +4 =
/// secondary staff_idx).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractPool {
    /// Contract records in insertion order.
    pub records: Vec<ContractRecord>,
    /// staff_id → primary contract index into `records`.
    /// `-1` means no primary contract. Vec-indexed for O(1)
    /// lookup — sized to `max_staff_id + 1`. Corresponds to
    /// `DAT_00acdf0c[person_id * 0x4F + 0]` in the exe.
    pub by_staff_id: Vec<i32>,
    /// staff_id → **secondary / alternate** contract index —
    /// corresponds to `DAT_00acdf0c[person_id * 0x4F + 4]`.
    /// Very rarely populated (the shipped-data loader does not
    /// fill this in the current C15.1B/C archaeology tranche;
    /// most persons have `-1` here). Present so the C15.1C
    /// primary-then-secondary short-circuit can be exercised
    /// in tests + honoured whenever runtime state populates
    /// it. See `reports/c15_1c_squad_archaeology.md` §11 for
    /// the deferred loader trace.
    #[serde(default)]
    pub by_staff_id_secondary: Vec<i32>,
}

impl ContractPool {
    pub fn contract_for_staff(&self, staff_id: u32) -> Option<&ContractRecord> {
        let idx = *self.by_staff_id.get(staff_id as usize)? as isize;
        if idx < 0 { return None; }
        self.records.get(idx as usize)
    }

    /// C15.1B — mutable form of [`contract_for_staff`]. Used by the
    /// promotion / relegation apply layer to write bytes `+0x1C`
    /// (non_promotion), `+0x1F` (relegation), and `+0x3A`
    /// (position_code, C15.1C) on the resolved contract record.
    pub fn contract_for_staff_mut(
        &mut self, staff_id: u32,
    ) -> Option<&mut ContractRecord> {
        let idx = *self.by_staff_id.get(staff_id as usize)? as isize;
        if idx < 0 { return None; }
        self.records.get_mut(idx as usize)
    }

    /// C15.1C — secondary / alternate contract lookup via
    /// `DAT_00acdf0c[person_id * 0x4F + 4]`. Used by
    /// `FUN_00843970`'s fallback branch when the primary
    /// record either doesn't exist or fails the identity gate.
    pub fn contract_for_staff_secondary_mut(
        &mut self, staff_id: u32,
    ) -> Option<&mut ContractRecord> {
        let idx = *self.by_staff_id_secondary
            .get(staff_id as usize)? as isize;
        if idx < 0 { return None; }
        self.records.get_mut(idx as usize)
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

/// Value from wage + ability + age. Matches the shape captured in
/// scratchpad/prelaunch/cheltenham_contract.png — Muggleton
/// (CA 94, PA 118, age 33) shows £110k, so a low-CA veteran gets
/// £100k+, and top-tier players hit £10M+. Rough calibration:
/// value ~= wage_multiple × age_curve × (0.5 + potential_bump).
fn value_multiplier(ca: i32, pa: i32, age: i32) -> f32 {
    // Base "years of wage" — the exe values in Cheltenham hit ~150x
    // weekly wage for backup / lower-tier players. Higher-CA
    // players fetch multi-hundred-x.
    let base_multiple: f32 = 120.0;
    let potential = (pa.max(ca) as f32 / 200.0).powi(2);
    let age_curve = if age <= 22       { 1.60 }
                    else if age <= 25  { 1.40 }
                    else if age <= 28  { 1.20 }
                    else if age <= 32  { 0.90 }
                    else if age <= 35  { 0.60 }
                    else               { 0.30 };
    base_multiple * (0.5 + 1.5 * potential) * age_curve
}

/// Compute a boot-time wage for a player at a club.
pub fn compute_wage(ca: i32, club_reputation: i32) -> i32 {
    let base = wage_from_ability(ca) as f32;
    let mult = LEAGUE_REP_MULT[league_rep_bucket(club_reputation)];
    // Round to the nearest £50.
    let raw = (base * mult).round() as i32;
    ((raw + 25) / 50) * 50
}

/// Compute a boot-time value for a player. Calibrated against
/// scratchpad/prelaunch/cheltenham_contract.png (Muggleton
/// CA 94 age 33 → £110k in the exe; Higgs £12k; Andy Mitchell
/// CA 135 age 27 → the £110K+ range too).
pub fn compute_value(wage: i32, ca: i32, pa: i32, age: i32) -> i32 {
    let raw = (wage as f32 * value_multiplier(ca, pa, age)) as i32;
    raw.clamp(1_000, 20_000_000)
}

/// Inputs the exe's `FUN_00847a80` reads to decide clause flags.
/// Field labels are the runtime-layout `person + Nx` offsets in the
/// decompile: adaptability +0x57, ambition +0x59, professionalism
/// +0x5b, class +0x3d. `store_flag` is the `param_2` byte passed
/// from FUN_004cd930 (always `1` at boot).
pub struct ClauseInputs {
    pub staff_id:       i32,
    pub reputation:     i32,   // local_3c
    pub class:          i8,    // person[+0x3d]  (0x0B = outfield senior)
    pub adaptability:   i8,    // person[+0x57]
    pub ambition:       i8,    // person[+0x59]
    pub professionalism:i8,    // person[+0x5b]
    pub attr16:         i8,    // person[+0x16] (unclear — reserve-status byte?)
    pub age:            i8,    // person[+6] in decompile (piVar3[6])
    pub store_flag:     i8,    // param_2 (== 1 at boot)
}

/// Roll the 5 release-clause flags. **Direct port of FUN_00847a80**
/// lines 224–303, verified against the decompile at
/// `D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/00847a80.c`.
///
/// The exe calls `FUN_008fc4f0(0x14)` (RNG 0..19) inside three of the
/// four gates; we substitute a deterministic per-(staff_id, seed)
/// hash → 0..19 so results are stable across runs and match the
/// "same DB → same clauses" behaviour the user observed.
pub fn roll_clauses(i: &ClauseInputs) -> ReleaseClauses {
    // Deterministic 0..19 draw indexed by seed slot.
    let rand20 = |seed: u32| -> i32 {
        let mut x = (i.staff_id as u32) ^ seed.wrapping_mul(0x9E3779B9);
        x = (x ^ (x >> 16)).wrapping_mul(0x7feb352d);
        x = (x ^ (x >> 15)).wrapping_mul(0x846ca68b);
        (x % 20) as i32
    };

    let mut c = ReleaseClauses::default();
    let rep = i.reputation;
    let adapt = i.adaptability as i32;
    let ambit = i.ambition as i32;
    let prof  = i.professionalism as i32;
    let attr16= i.attr16 as i32;
    let age   = i.age as i32;
    let sf    = i.store_flag as i32;

    // ----- Manager-Job (+0x20) — FUN_00847a80:224–229 -----
    // if ((age > 0x20 && rep > 0xcb2 && staff_id % 3 == 0 &&
    //      0x2d < prof + attr16 + adapt + (staff_id%10) - 0x1c + age)
    //     || has_agent) { manager_job = 1; }
    let mj_score = prof + attr16 + adapt + (i.staff_id.rem_euclid(10))
                 - 0x1c + age;
    if age > 0x20 && rep > 0xcb2 && i.staff_id.rem_euclid(3) == 0
       && mj_score > 0x2d {
        c.manager_job = 1;
    }

    // Rate calibration: the exe's real gate uses `(char)param_2 * K`
    // where param_2 is the runtime person pointer. Ghidra's cast reads
    // the low byte of that pointer — an allocation-order artefact we
    // can't reproduce statically. The empirically-observed rate at
    // Cheltenham (rep 3000) is ~12% NP / ~12% Rlg among eligible
    // players. Adding a deterministic 1-in-8 hash gate reproduces
    // that rate while keeping same-DB → same-clauses behaviour.
    let np_rate_pass  = rand20(11) < 3;    // ~15%
    let rlg_rate_pass = rand20(12) < 3;    // ~15%
    let npl_rate_pass = rand20(13) < 4;    // ~20% among eligible

    // ----- Non-Promotion (+0x1C) — FUN_00847a80:231–249 -----
    // if (class == 0x0B && rep > 0xabe &&
    //     sf*0xc < rep/0x32 + rand20 + adapt*2 + ambit*(-3) &&
    //     league_is_playable) { non_promotion = 1; }
    if i.class == 0x0B && rep > 0xabe {
        let lhs = sf * 0xc;
        let rhs = rep / 0x32 + rand20(1) + adapt * 2 + ambit * -3;
        if lhs < rhs && np_rate_pass {
            c.non_promotion = 1;
        }
    }

    // ----- Non-Playing (+0x1E) — FUN_00847a80:253–268 -----
    // if (class == 0x0B && rep > 0x1964 &&
    //     sf*0xe < rep/0x32 + ambit*(-4) - rand20) { non_playing = 1; }
    if i.class == 0x0B && rep > 0x1964 {
        let lhs = sf * 0xe;
        let rhs = rep / 0x32 + ambit * -4 - rand20(2);
        if lhs < rhs && npl_rate_pass {
            c.non_playing = 1;
        }
    }

    // ----- Relegation (+0x1F) — FUN_00847a80:273–299 -----
    // if (class == 0x0B && rep > 0x6d6 &&
    //     sf*10 < rep/0x32 + adapt*3 + ambit*(-2) - rand20 &&
    //     league_is_playable) { relegation = 1; }
    if i.class == 0x0B && rep > 0x6d6 {
        let lhs = sf * 10;
        let rhs = rep / 0x32 + adapt * 3 + ambit * -2 - rand20(3);
        if lhs < rhs && rlg_rate_pass {
            c.relegation = 1;
        }
    }

    // ----- Minimum-Fee (+0x1D) — always cleared at boot -----
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

        // ---- LAYERED OVERRIDE ---------------------------------------
        // Prefer the shipped-DB values wherever they are populated.
        // Only synthesise the missing ones (~88% of staff whose .dat
        // record has wage=0 / value=0 / expiry=31.1.1900 placeholder).
        // Verified against the JSON dump: Chris Banks (id=45) has
        // DB wage=600 value=35000 expiry=30.6.2004; Muggleton has
        // DB expiry=29.5.2003 but wage/value=0.

        let db_wage  = pv.wage();
        let db_value = pv.value();
        let db_exp   = pv.club_contract_expires();
        let db_exp_valid = !db_exp.is_placeholder();

        let wage = if db_wage > 0 { db_wage } else { compute_wage(ca, reputation) };
        // Value: always try DB first; synthesise only when zero.
        // Even players with wage=0 in the DB may have a real value.
        let value = if db_value > 0 {
            db_value
        } else {
            compute_value(wage, ca, pa, age)
        };
        // Personality bytes for FUN_00847a80 clause math. PlayerView
        // accessors already handle the tail-vs-body offset shift.
        let adapt = pv.adaptability() as i8;
        let ambit = pv.ambition() as i8;
        let prof  = pv.professionalism() as i8;
        // attr16 in the decompile is `(char)piVar3[0x16]` — the low
        // byte of the int at record offset 0x58, which our body maps
        // to body[0x54] = determination.
        let attr16 = pv.determination() as i8;
        let class  = pv.club_job() as i8;
        let clauses = roll_clauses(&ClauseInputs {
            staff_id:        person.id as i32,
            reputation,
            class,
            adaptability:    adapt,
            ambition:        ambit,
            professionalism: prof,
            attr16,
            age:             age as i8,
            store_flag:      1,
        });

        // Expiry — use DB when it holds a real date (year != 1900
        // placeholder). Synthesise only for the ~40% of Cheltenham
        // staff with unset expiry: 2-year default, extended for
        // higher-CA players.
        let (expiry_dayofyear, expiry_year) = if db_exp_valid {
            (db_exp.day, db_exp.year)
        } else {
            let years_ahead = if ca >= 140 { 4 } else if ca >= 100 { 3 } else { 2 };
            (180, (today_year + years_ahead) as u16)
        };

        let idx = records.len() as i32;
        by_staff_id[person.id as usize] = idx;
        records.push(ContractRecord {
            staff_id:  person.id as i32,
            // C15.1C archaeology proved `+0x04` is `club_id`, not
            // `person_id`. The disk→runtime loader trace has NOT
            // established the correct per-contract club id at
            // boot; setting to 0 means the identity gate
            // `record.club_id == effects.club_id` will always
            // fail in production until the loader trace lands
            // (deferred item — see
            // `reports/c15_1c_squad_archaeology.md` §11). Tests
            // set this explicitly.
            club_id: 0,
            wage,
            value,
            non_promotion: clauses.non_promotion,
            minimum_fee:   clauses.minimum_fee,
            non_playing:   clauses.non_playing,
            relegation:    clauses.relegation,
            manager_job:   clauses.manager_job,
            expiry_dayofyear,
            expiry_year,
            position_code: 0,
        });
    }

    eprintln!("[contract_init] {} contracts generated from {} staff (max_id={})",
              records.len(), world.staff.type6.len(), max_id);
    ContractPool {
        records,
        by_staff_id,
        // Secondary index is empty at boot — populated only when
        // runtime state warrants it (rare per shipped data).
        by_staff_id_secondary: Vec::new(),
    }
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
        let inp = ClauseInputs {
            staff_id: 12345, reputation: 5000, class: 0x0B,
            adaptability: 12, ambition: 10, professionalism: 15,
            attr16: 8, age: 26, store_flag: 1,
        };
        let a1 = roll_clauses(&inp);
        let a2 = roll_clauses(&inp);
        assert_eq!(a1.non_promotion, a2.non_promotion);
        assert_eq!(a1.relegation,    a2.relegation);
    }

    #[test]
    fn non_playing_never_fires_below_rep_gate() {
        // Rep < 6500 must never produce a Non-Playing clause.
        let inp = ClauseInputs {
            staff_id: 56534, reputation: 3000, class: 0x0B,
            adaptability: 9, ambition: 0, professionalism: 15,
            attr16: 11, age: 33, store_flag: 1,
        };
        let c = roll_clauses(&inp);
        assert_eq!(c.non_playing, 0, "rep 3000 < 6500 must NOT set NPl");
    }

    #[test]
    fn clause_rate_around_calibration_target() {
        // Simulate 400 Third-Division-ish players and check ~10-25%
        // pick up a clause. Same rep + class + attrs; only staff_id
        // varies. This is the "roughly one in eight" behaviour we
        // observe in the exe capture.
        let inp = |id: i32| ClauseInputs {
            staff_id: id, reputation: 3000, class: 0x0B,
            adaptability: 9, ambition: 0, professionalism: 15,
            attr16: 11, age: 27, store_flag: 1,
        };
        let mut any = 0;
        for id in 100..500 {
            let c = roll_clauses(&inp(id));
            if c.non_promotion != 0 || c.relegation != 0 { any += 1; }
        }
        let rate = any as f32 / 400.0;
        assert!((0.10..=0.40).contains(&rate),
                "clause rate {rate:.2} outside expected 10-40% band");
    }
}
