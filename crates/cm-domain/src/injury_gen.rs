//! Byte-exact port of the injury GENERATION algorithm:
//! * [`pick_injury`]  ← `FUN_00616820` (choose an injury id within a body
//!   region by "minimum random roll wins", with a recurrence bias);
//! * [`compute_injury`] ← `FUN_00616930` (recovery-days formula + the
//!   pct→floor term).
//!
//! Both draw from the game's real table RNG ([`crate::sim_rng::SimRng`]).
//! Decode: `reports/injury_generator_decode.md` §1–§2.
//!
//! Two inputs to the day formula are not recoverable as pure integers from
//! the decompile and are passed in by the caller:
//!  * `physio` — the best physiotherapist rating at the player's club
//!    (`FUN_0052df60`, x87 body dropped by Ghidra). 0 when the club has no
//!    physio, which is a real exe path.
//! The pct→floor term uses `__ftol` truncation, reproduced here as integer
//! `days*pct/100`.

use crate::injury_table::{INJURY_TYPES, INJURY_REGION_RANGES};
use crate::sim_rng::SimRng;

/// `FUN_00616820` — pick an injury id within `region` (0..11). `sev_class`
/// is the requested severity filter (`-1` = any; the exe never rolls when
/// the *requested* class is 3). `recent_id` is the player's most-recent
/// injury id (`-1` = none) for the recurrence bias. Returns the chosen id,
/// or `None` (0xFF) when nothing rolled below 100.
pub fn pick_injury(region: u8, sev_class: i8, recent_id: i16, rng: &mut SimRng) -> Option<u8> {
    let (min_id, max_id) = match INJURY_REGION_RANGES.get(region as usize) {
        Some(&r) => r,
        None => return None,
    };
    // Recurrence candidate: recent injury whose region matches this one.
    let prev: i16 = if recent_id >= 0
        && (recent_id as usize) < INJURY_TYPES.len()
        && INJURY_TYPES[recent_id as usize].region == region
    {
        recent_id
    } else {
        -1
    };
    let mut min_roll: i32 = 100;
    let mut sel: i16 = -1;
    let mut id = min_id as i16;
    while id <= max_id as i16 {
        let t = &INJURY_TYPES[id as usize];
        // Severity filter (exe: pass when requested class is -1, or table
        // severity matches the requested class, or table severity is -1).
        let filter_pass = sev_class == -1 || t.severity == sev_class || t.severity == -1;
        // The roll is gated on the *requested* class not being 3.
        if filter_pass && sev_class != 3 {
            let roll = if id == prev {
                rng.rand(3) - 1 // recurrence: {-1, 0, 1}
            } else {
                rng.rand(t.weight.max(1) as i32) // rand(weight)
            };
            if roll < min_roll {
                min_roll = roll;
                sel = id;
            }
        }
        id += 1;
    }
    if sel < 0 { None } else { Some(sel as u8) }
}

/// Result of applying an injury: total days out + the "min remaining" floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InjuryOutcome {
    pub id: u8,
    pub total_days: i16,
    pub floor_days: i16,
}

/// `FUN_00616930` — compute the recovery days for injury `id`.
/// `physio` = best club physiotherapist rating (0 = none), `age` = player
/// age, `prone` = Injury Proneness (type10+0x2d), `prev_remaining` = the
/// countdown of an injury being aggravated/replaced (0 otherwise).
pub fn compute_injury(
    id: u8,
    physio: i32,
    age: i32,
    prone: i32,
    prev_remaining: i16,
    rng: &mut SimRng,
) -> InjuryOutcome {
    let t = &INJURY_TYPES[id as usize];
    let var_days = t.var_days as i32;
    let min_days = t.min_days as i32;

    // (b) varDays * (35 - physio) / 20   (integer division)
    let mut days: i32 = (var_days * (35 - physio)) / 20;
    // (c) good-physio shortening: if rand(physio+5) > rand(20) then -rand(days)
    let a = rng.rand(physio + 5);
    let b = rng.rand(20);
    if b < a && days > 0 {
        let s = rng.rand(days);
        days -= s;
    }
    // (d) physiotherapy pseudo-injury 'Q' (0x51) => 0 days
    if id == 0x51 {
        days = 0;
    }
    // (e) aggravation: add the replaced injury's remaining countdown
    days += prev_remaining as i32;
    // (f) age / proneness penalties (only for minDays > 10, id != 'Q')
    if id != 0x51 && min_days > 10 {
        if age > 30 {
            let s = prone - 35 + age;
            if s > 0 {
                days += rng.rand(s);
            }
        }
        if prone > 10 {
            days += rng.rand(prone - 10);
        }
    }
    // (g) base floor
    days += min_days;

    let total = days.max(0) as i16;
    // Min-remaining floor = pct% of total (FUN_00616930 __ftol truncation),
    // clamped so the floor is never more than 60 below the total.
    let mut floor = ((total as i32 * t.pct as i32) / 100) as i16;
    if (floor as i32) + 60 < total as i32 {
        floor = total - 60;
    }
    InjuryOutcome { id, total_days: total, floor_days: floor }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_returns_id_in_region() {
        let mut r = SimRng::seed(42);
        // Region 3 = ids 22..27 (knee injuries).
        for _ in 0..200 {
            if let Some(id) = pick_injury(3, -1, -1, &mut r) {
                assert!((22..=27).contains(&id), "id {id} outside region 3");
            }
        }
    }

    #[test]
    fn days_positive_and_bounded() {
        let mut r = SimRng::seed(7);
        // id 21 = "broken leg" (a long injury) — days should be substantial.
        let o = compute_injury(21, 0, 24, 10, 0, &mut r);
        assert!(o.total_days > 0);
        assert!(o.floor_days <= o.total_days);
    }
}
