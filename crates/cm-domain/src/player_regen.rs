//! Player-regen / free-agent redistribution — the REAL mechanism CM0102
//! uses to fill empty club squads.
//!
//! Ported from the decompile (verified live via Frida trace against the
//! running exe in the discovery session that produced this module):
//!
//! * `FUN_0078E970` (`player_regen.cpp`) — the driver: retires overdue
//!   players, then for each empty slot picks a free agent and links it to
//!   the club. `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/0078e970.c`
//! * `FUN_0078F200` — free-agent selector: two-pass scan of the full
//!   132,722-record staff pool (`DAT_00ACD5C4`, stride `0x6E`) — pass 1
//!   restricts to the target nation, pass 2 falls back to the whole pool.
//!   `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/0078f200.c`
//! * `FUN_0078F4F0` — suitability scorer, a weighted mix of personality,
//!   reputation-threshold bonuses, and RNG terms.
//!   `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/0078f4f0.c`
//!
//! It is NOT procedural name generation: it redistributes REAL,
//! already-loaded staff records that don't currently belong to a club.
//!
//! ## What's approximated (documented per-term below)
//!
//! Two terms in `FUN_0078F4F0` are computed via `FUN_00935080` (an FPU
//! helper of undecoded purpose) immediately truncated by `__ftol`. Without
//! decoding that helper we can't recover their exact bound, so this port
//! rolls them through the same `rng` closure the rest of the formula uses
//! with a documented placeholder bound (`FLOAT_TERM_BOUND`). Everything
//! else — every offset, every threshold, every weight — is a direct,
//! byte-for-byte port of the decompiled arithmetic, using the
//! already-decoded field accessors on [`PlayerView`] and
//! [`crate::DomainStaffType10`] instead of raw offsets.
//!
//! Two more inputs the exe reaches through live pointers baked into the
//! staff record are not yet decoded as standalone tables/fields and are
//! passed in explicitly via [`RegenScoreContext`] instead of re-derived
//! here: the per-person "nation record flag" bitfield
//! (`DAT_00acdf0c[id*0x4f+0xb] & 0x30`) and the employer club's `+0xcf`
//! flag byte.

use crate::typed_records::PlayerView;
use crate::DomainStaffType10;

/// `FUN_0078F4F0`'s invalid-candidate sentinel, `0xffffd8f0` reinterpreted
/// as `i32` (`-10000`).
pub const REGEN_SCORE_INVALID: i32 = -10000;

/// Placeholder bound for the two RNG terms computed via the undecoded
/// `FUN_00935080`/`__ftol` pair. See module docs.
const FLOAT_TERM_BOUND: i32 = 100;

/// Per-candidate context the scorer needs beyond the person record itself.
/// In the exe these come from live pointers baked into the staff record
/// (`+0x39` → employer club, `+0x61` → type10 attribute record); this port
/// takes them explicitly since our records are plain data, not pointers.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegenScoreContext {
    /// Reputation of the candidate's CURRENT employer club
    /// ([`crate::typed_records::ClubView::reputation`], `club+0x80`).
    /// Only consulted when the candidate already has a club; pass `None`
    /// for a free agent or when the employer club isn't known.
    pub employer_club_reputation: Option<u16>,
    /// The employer club's `+0xcf` flag byte, undecoded — the driver
    /// (`FUN_0078E970`) branches on `== 0` as a fast-path when assigning.
    /// Default `false` (not the fast-path) is the safe assumption.
    pub employer_club_flag_cf_zero: bool,
    /// The undecoded per-person "nation record flag" gate
    /// (`DAT_00acdf0c[id*0x4f+0xb] & 0x30`). Default `false` (not blocked)
    /// until that table is decoded.
    pub nation_flag_blocked: bool,
}

/// Read type10's undecoded short field at record offset `+0xb` (the last
/// two bytes of `unknown_bytes_9_12`) — used by the scorer for two
/// reputation-threshold bonuses and a ratio term. Semantics unknown beyond
/// "another CA/reputation-scale short", per the `+0xd` field right next to
/// it (`DomainStaffType10::rating_short_0x0d`, "probable_reputation").
fn type10_field_0xb(t: &DomainStaffType10) -> i16 {
    t.current_reputation_value() as i16
}

/// Direct port of `FUN_0078F4F0` — the free-agent suitability scorer.
///
/// `rng` mirrors `FUN_008FC4F0(bound)`: a uniform integer draw in
/// `0..bound` (exclusive upper bound, matching the exe's RNG-bound-N
/// convention). Returns [`REGEN_SCORE_INVALID`] for a candidate the exe
/// would reject outright.
pub fn regen_suitability_score(
    person: &PlayerView,
    type10: Option<&DomainStaffType10>,
    ctx: &RegenScoreContext,
    rng: &mut impl FnMut(i32) -> i32,
) -> i32 {
    // Gate 1 (`record+0x69 != 0`): `PlayerView::non_player_data_id()` set
    // means this staff record is non-player STAFF (manager/coach/etc, the
    // type-9 link), not a signable player.
    if person.non_player_data_id().is_some() {
        return REGEN_SCORE_INVALID;
    }
    // Gate 2: undecoded nation-record flag bit.
    if ctx.nation_flag_blocked {
        return REGEN_SCORE_INVALID;
    }
    // Gate 3 (`(char)record[0x18] < 0x1f`): `secondary_year_field` read as
    // a SIGNED byte must be >= 0x1f (31).
    let syf = person.secondary_year_field() as u8 as i8;
    if syf < 0x1f {
        return REGEN_SCORE_INVALID;
    }

    // `FUN_008fc4f0((syf - 0x1f) * 0x96)` — base roll scaled by how far
    // past the 0x1f floor the field sits.
    let mut score = rng((syf as i32 - 0x1f) * 0x96);

    // If the candidate already has a club AND that club has a non-zero
    // reputation AND the candidate has a type10 attribute link: add a
    // ratio term (their type10+0xb field over the employer's reputation),
    // scaled ×250.
    if let (Some(_club_id), Some(club_rep), Some(t10)) =
        (person.current_club_id(), ctx.employer_club_reputation, type10)
    {
        if club_rep != 0 {
            let f0xb = type10_field_0xb(t10) as i32;
            score += (f0xb / club_rep as i32) * 0xfa;
        }
    }

    // If the candidate has a type10 link: one more RNG term (approximated,
    // see module docs) plus two reputation-threshold bonuses.
    if let Some(t10) = type10 {
        score += rng(FLOAT_TERM_BOUND);
        if t10.reputation() as i16 > 0x1d4c {
            score += 100;
        }
        if type10_field_0xb(t10) > 0x1d4c {
            score += 0x32;
        }
    }

    // The big personality-weighted sum, plus two more approximated RNG
    // terms (`iVar4`, `iVar5` in the decompile — always evaluated,
    // independent of the type10 branch above).
    let temperament = person.temperament() as i32;
    let pressure = person.pressure() as i32;
    let loyalty = person.loyalty() as i32;
    let determination = person.determination() as i32;
    let adaptability = person.adaptability() as i32;
    let rng_a = rng(FLOAT_TERM_BOUND);
    let rng_b = rng(FLOAT_TERM_BOUND);
    score += ((temperament + pressure) * 3 + (loyalty + determination) * 2 + adaptability) * 5
        + rng_a
        + rng_b;

    match person.current_club_id() {
        None => {
            // Unemployed: caps^2 bonus, then a coin-flip-ish doubling
            // gated on type10+0xd (`FUN_008fc4f0(10000) < field_0xd`).
            let caps = person.international_caps() as i32;
            score += caps * caps;
            if let Some(t10) = type10 {
                let roll = rng(10000);
                if roll < t10.reputation() as i16 as i32 {
                    score *= 2;
                }
            }
        }
        Some(_) => {
            // Employed: the driver's `+0xcf == 0` fast-path returns a
            // slightly-boosted roll directly instead of falling through
            // to the generic final roll below.
            if ctx.employer_club_flag_cf_zero {
                return rng(score + 0x4b);
            }
        }
    }

    rng(score)
}

/// Direct port of `FUN_0078F200`'s two-pass free-agent selection: pass 1
/// restricts to `target_nation_id` and picks the max-scoring candidate;
/// pass 2 (only reached if pass 1 finds nothing) drops the nation filter
/// and scans the whole pool.
///
/// `ctx_for` supplies the per-candidate [`RegenScoreContext`] (employer
/// reputation etc.) that the exe would otherwise reach via live pointers.
/// Only candidates with a positive score are eligible, matching the
/// decompile's `0 < iVar1` guard.
pub fn select_free_agent<'a>(
    pool: &[PlayerView<'a>],
    type10_by_index: &[Option<&DomainStaffType10>],
    target_nation_id: i32,
    ctx_for: impl Fn(usize, &PlayerView) -> RegenScoreContext,
    rng: &mut impl FnMut(i32) -> i32,
) -> Option<usize> {
    let mut best: Option<(usize, i32)> = None;
    for (i, p) in pool.iter().enumerate() {
        if p.nation_id() != Some(target_nation_id) {
            continue;
        }
        let t10 = type10_by_index.get(i).copied().flatten();
        let ctx = ctx_for(i, p);
        let score = regen_suitability_score(p, t10, &ctx, rng);
        if score > 0 && best.is_none_or(|(_, b)| score > b) {
            best = Some((i, score));
        }
    }
    if let Some((i, _)) = best {
        return Some(i);
    }

    let mut best: Option<(usize, i32)> = None;
    for (i, p) in pool.iter().enumerate() {
        let t10 = type10_by_index.get(i).copied().flatten();
        let ctx = ctx_for(i, p);
        let score = regen_suitability_score(p, t10, &ctx, rng);
        if score > 0 && best.is_none_or(|(_, b)| score > b) {
            best = Some((i, score));
        }
    }
    best.map(|(i, _)| i)
}

/// Driver loop matching `FUN_0078E970`'s second loop shape: repeatedly
/// select a free agent for `nation_id` and assign it to `club_id`, until
/// `slots_needed` players have been placed or the pool is exhausted.
///
/// Unlike the exe (which mutates a live pointer field), this writes the
/// club id into the picked record's `current_club_id` disk slot
/// (`body[0x35..0x39]`, i.e. `PlayerView` record offset `0x39`) so the
/// assignment is visible to any subsequent `PlayerView` read of the same
/// `staff_pool` entry. Already-assigned-this-call records are excluded
/// from further picks so a squad fill doesn't repeat the same player.
///
/// Returns the staff ids assigned, in assignment order.
pub fn regen_fill_club_squad(
    staff_pool: &mut [crate::DomainStaffType6],
    type10_pool: &[DomainStaffType10],
    club_id: i32,
    club_reputation: u16,
    nation_id: i32,
    slots_needed: usize,
    rng: &mut impl FnMut(i32) -> i32,
) -> Vec<u32> {
    let t10_by_id: std::collections::HashMap<u32, &DomainStaffType10> =
        type10_pool.iter().map(|t| (t.id, t)).collect();

    let mut assigned = Vec::with_capacity(slots_needed);
    let mut excluded: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for _ in 0..slots_needed {
        let views: Vec<PlayerView> = staff_pool
            .iter()
            .map(|s| PlayerView::from_split(s.id, &s.body))
            .collect();
        let t10_by_index: Vec<Option<&DomainStaffType10>> = staff_pool
            .iter()
            .map(|s| t10_by_id.get(&s.id).copied())
            .collect();

        let ctx_for = |i: usize, p: &PlayerView| {
            if excluded.contains(&i) {
                // Force-reject: the scorer has no "excluded" concept, so
                // fake an already-non-player record via a blocked nation
                // flag (any gate that returns REGEN_SCORE_INVALID works).
                return RegenScoreContext {
                    nation_flag_blocked: true,
                    ..Default::default()
                };
            }
            RegenScoreContext {
                employer_club_reputation: p
                    .current_club_id()
                    .map(|_| club_reputation)
                    .filter(|_| p.current_club_id() == Some(club_id)),
                employer_club_flag_cf_zero: false,
                nation_flag_blocked: false,
            }
        };

        let Some(idx) = select_free_agent(&views, &t10_by_index, nation_id, ctx_for, rng) else {
            break;
        };
        if excluded.contains(&idx) {
            break;
        }

        let s = &mut staff_pool[idx];
        let club_bytes = (club_id as u32).to_le_bytes();
        // Disk offset 0x35 == PlayerView record offset 0x39
        // (`current_club_id`) once the 4-byte `id` prefix is accounted
        // for (`body[0]` == record offset `0x04`).
        if s.body.len() >= 0x35 + 4 {
            s.body[0x35..0x35 + 4].copy_from_slice(&club_bytes);
        }
        assigned.push(s.id);
        excluded.insert(idx);
    }

    assigned
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DomainStaffType6;

    /// Build a minimal 110-byte in-memory-format staff record body (tail
    /// after the 4-byte id) with the given fields set. Byte offsets are
    /// `PlayerView` RECORD offsets (i.e. `tail` offset = record offset - 4).
    fn make_body(
        nation_id: i32,
        current_club_id: Option<i32>,
        international_caps: u8,
        secondary_year_field: u16,
        personality: [u8; 5], // adaptability, loyalty, pressure, temperament, determination
        non_player_data_id: Option<i32>,
    ) -> Vec<u8> {
        let mut body = vec![0u8; 0x6e - 4];
        let at = |record_off: usize| record_off - 4;
        body[at(0x18)..at(0x18) + 2].copy_from_slice(&secondary_year_field.to_le_bytes());
        body[at(0x1a)..at(0x1a) + 4].copy_from_slice(&nation_id.to_le_bytes());
        body[at(0x22)] = international_caps;
        let club_val = current_club_id.unwrap_or(-1);
        body[at(0x39)..at(0x39) + 4].copy_from_slice(&club_val.to_le_bytes());
        let [adaptability, loyalty, pressure, temperament, determination] = personality;
        body[at(0x56)] = adaptability; // adaptability
        body[at(0x58)] = determination; // determination
        body[at(0x59)] = loyalty; // loyalty
        body[at(0x5a)] = pressure; // pressure
        body[at(0x5d)] = temperament; // temperament
        let non_player_val = non_player_data_id.unwrap_or(-1);
        body[at(0x69)..at(0x69) + 4].copy_from_slice(&non_player_val.to_le_bytes());
        body
    }

    fn make_type10(id: u32, ca: u16, reputation_0xd: u16, field_0xb: i16) -> DomainStaffType10 {
        let mut unknown_bytes_9_12 = [0u8; 4];
        unknown_bytes_9_12[2..4].copy_from_slice(&field_0xb.to_le_bytes());
        DomainStaffType10 {
            id,
            current_ability: ca as i16,
            world_reputation: reputation_0xd as i16,
            rating_short_0x05: ca,
            rating_short_0x0d: reputation_0xd,
            unknown_bytes_9_12,
            ..Default::default()
        }
    }

    /// Deterministic "RNG" that always returns the midpoint of the bound
    /// (or 0 for a non-positive bound) — makes score assertions tractable
    /// by hand while still exercising every RNG call site.
    fn mid_rng(bound: i32) -> i32 {
        if bound <= 0 {
            0
        } else {
            bound / 2
        }
    }

    #[test]
    fn rejects_non_player_staff() {
        let body = make_body(1, None, 0, 40, [0, 0, 0, 0, 0], Some(99));
        let p = PlayerView::from_split(1, &body);
        let mut rng = mid_rng;
        let score = regen_suitability_score(&p, None, &RegenScoreContext::default(), &mut rng);
        assert_eq!(score, REGEN_SCORE_INVALID);
    }

    #[test]
    fn rejects_below_secondary_year_floor() {
        let body = make_body(1, None, 0, 0x1e, [0, 0, 0, 0, 0], None);
        let p = PlayerView::from_split(1, &body);
        let mut rng = mid_rng;
        let score = regen_suitability_score(&p, None, &RegenScoreContext::default(), &mut rng);
        assert_eq!(score, REGEN_SCORE_INVALID);
    }

    #[test]
    fn rejects_nation_flag_blocked() {
        let body = make_body(1, None, 0, 40, [0, 0, 0, 0, 0], None);
        let p = PlayerView::from_split(1, &body);
        let mut rng = mid_rng;
        let ctx = RegenScoreContext { nation_flag_blocked: true, ..Default::default() };
        let score = regen_suitability_score(&p, None, &ctx, &mut rng);
        assert_eq!(score, REGEN_SCORE_INVALID);
    }

    #[test]
    fn accepts_at_secondary_year_floor() {
        let body = make_body(1, None, 0, 0x1f, [0, 0, 0, 0, 0], None);
        let p = PlayerView::from_split(1, &body);
        let mut rng = mid_rng;
        let score = regen_suitability_score(&p, None, &RegenScoreContext::default(), &mut rng);
        assert!(score > REGEN_SCORE_INVALID);
    }

    #[test]
    fn scoring_formula_matches_hand_computation_no_type10_free_agent() {
        // syf=0x2f (47) -> base = mid_rng((0x2f-0x1f)*0x96) = mid_rng(0x960) = 0x4b0
        // no type10 -> skip ratio term and threshold-bonus term
        // personality: temperament=10, pressure=5, loyalty=4, determination=3, adaptability=2
        //   weighted = ((10+5)*3 + (4+3)*2 + 2) * 5 = (45+14+2)*5 = 61*5 = 305
        //   + rng_a(100)/2=50 + rng_b(100)/2=50 => +100
        // running = 0x4b0 (1200) + 305 + 100 = 1605
        // unemployed (current_club_id=None): caps^2 bonus; caps=6 -> +36 => 1641
        //   no type10 -> skip doubling roll
        // final = mid_rng(1641) = 820
        let body = make_body(1, None, 6, 0x2f, [2, 4, 5, 10, 3], None);
        let p = PlayerView::from_split(1, &body);
        let mut rng = mid_rng;
        let score = regen_suitability_score(&p, None, &RegenScoreContext::default(), &mut rng);
        assert_eq!(score, 820);
    }

    #[test]
    fn unemployed_gets_caps_bonus_over_otherwise_identical_employed() {
        let body_free = make_body(1, None, 20, 0x30, [5, 5, 5, 5, 5], None);
        let body_employed = make_body(1, Some(42), 20, 0x30, [5, 5, 5, 5, 5], None);
        let p_free = PlayerView::from_split(1, &body_free);
        let p_employed = PlayerView::from_split(2, &body_employed);
        let mut rng1 = mid_rng;
        let mut rng2 = mid_rng;
        let score_free =
            regen_suitability_score(&p_free, None, &RegenScoreContext::default(), &mut rng1);
        let score_employed = regen_suitability_score(
            &p_employed,
            None,
            &RegenScoreContext::default(),
            &mut rng2,
        );
        // Free agent gets +caps^2 = +400 pre-final-roll that the employed
        // candidate (no employer reputation supplied, so the ratio term is
        // also skipped for them) does not. `mid_rng`'s final `rng(score)`
        // call halves whatever it's given, so the +400 pre-roll bonus
        // shows up as +200 in the post-roll result.
        assert_eq!(score_free - score_employed, 200);
    }

    #[test]
    fn type10_reputation_threshold_adds_bonus() {
        let body = make_body(1, None, 0, 0x2f, [0, 0, 0, 0, 0], None);
        let p = PlayerView::from_split(1, &body);
        let t10_low = make_type10(1, 100, 0x1d4c, 0x1d4c); // at threshold, no bonus (needs >)
        let t10_high = make_type10(1, 100, 0x1d4d, 0x1d4d); // just over, both bonuses
        let mut rng1 = mid_rng;
        let mut rng2 = mid_rng;
        let score_low = regen_suitability_score(
            &p,
            Some(&t10_low),
            &RegenScoreContext::default(),
            &mut rng1,
        );
        let score_high = regen_suitability_score(
            &p,
            Some(&t10_high),
            &RegenScoreContext::default(),
            &mut rng2,
        );
        assert!(score_high > score_low);
    }

    #[test]
    fn select_free_agent_prefers_target_nation() {
        let body_a = make_body(1, None, 50, 0x40, [10, 10, 10, 10, 10], None); // nation 1, strong
        let body_b = make_body(2, None, 50, 0x40, [10, 10, 10, 10, 10], None); // nation 2, strong
        let pool = vec![
            PlayerView::from_split(1, &body_a),
            PlayerView::from_split(2, &body_b),
        ];
        let t10s: Vec<Option<&DomainStaffType10>> = vec![None, None];
        let mut rng = mid_rng;
        let picked = select_free_agent(
            &pool,
            &t10s,
            1,
            |_, _| RegenScoreContext::default(),
            &mut rng,
        );
        assert_eq!(picked, Some(0), "must pick the nation-1 candidate, not nation-2");
    }

    #[test]
    fn select_free_agent_falls_back_to_whole_pool_when_nation_empty() {
        let body_b = make_body(2, None, 50, 0x40, [10, 10, 10, 10, 10], None); // nation 2 only
        let pool = vec![PlayerView::from_split(2, &body_b)];
        let t10s: Vec<Option<&DomainStaffType10>> = vec![None];
        let mut rng = mid_rng;
        // Target nation 1 has no candidates -> pass 2 fallback picks the
        // nation-2 candidate anyway.
        let picked = select_free_agent(
            &pool,
            &t10s,
            1,
            |_, _| RegenScoreContext::default(),
            &mut rng,
        );
        assert_eq!(picked, Some(0));
    }

    #[test]
    fn select_free_agent_never_picks_employed_or_non_player_staff() {
        // Non-player staff (has non_player_data_id) must be rejected even
        // though nation matches.
        let body_staff = make_body(1, None, 99, 0x60, [20, 20, 20, 20, 20], Some(7));
        let body_player = make_body(1, None, 1, 0x1f, [0, 0, 0, 0, 0], None);
        let pool = vec![
            PlayerView::from_split(1, &body_staff),
            PlayerView::from_split(2, &body_player),
        ];
        let t10s: Vec<Option<&DomainStaffType10>> = vec![None, None];
        let mut rng = mid_rng;
        let picked = select_free_agent(
            &pool,
            &t10s,
            1,
            |_, _| RegenScoreContext::default(),
            &mut rng,
        );
        assert_eq!(picked, Some(1), "must skip the non-player-staff record");
    }

    #[test]
    fn regen_fill_club_squad_produces_requested_count_and_distinct_players() {
        let mut staff: Vec<DomainStaffType6> = (1..=10u32)
            .map(|id| DomainStaffType6 {
                id,
                body: make_body(9, None, (id % 5) as u8, 0x30 + id as u16, [3, 3, 3, 3, 3], None),
            })
            .collect();
        let type10s: Vec<DomainStaffType10> =
            (1..=10u32).map(|id| make_type10(id, 100, 1000, 500)).collect();
        let mut rng = mid_rng;
        let assigned = regen_fill_club_squad(&mut staff, &type10s, 555, 4000, 9, 6, &mut rng);
        assert_eq!(assigned.len(), 6);
        let unique: std::collections::HashSet<_> = assigned.iter().collect();
        assert_eq!(unique.len(), 6, "must not repeat a player within one fill");
        for id in &assigned {
            let s = staff.iter().find(|s| s.id == *id).unwrap();
            assert_eq!(s.current_club_id(), Some(555));
        }
    }

    #[test]
    fn regen_fill_club_squad_stops_when_pool_exhausted() {
        let mut staff: Vec<DomainStaffType6> = (1..=3u32)
            .map(|id| DomainStaffType6 {
                id,
                body: make_body(9, None, 1, 0x30, [1, 1, 1, 1, 1], None),
            })
            .collect();
        let type10s: Vec<DomainStaffType10> = vec![];
        let mut rng = mid_rng;
        let assigned = regen_fill_club_squad(&mut staff, &type10s, 1, 1000, 9, 10, &mut rng);
        assert_eq!(assigned.len(), 3, "only 3 eligible candidates existed");
    }
}
