//! New-game player generation — algorithm-exact port of the exe's
//! `FUN_0051f5d0` (decode: `reports/player_generation_decode.md`).
//! Generates position aptitudes, footedness and the 42 attributes for
//! players whose base record ships with no data (~half the DB). Every
//! step's logic is the exe's; no invented formulas. It is NOT the exe's
//! byte-identical RNG sequence (the pre-pipeline — names/DOB/personality/
//! reputation — is not replayed here), but each generated player is built
//! by the real process.
//!
//! Replaces the old flat `rand(7)+CA/10-3` attribute fill (which made
//! regen players' stats harsh and gave them no positions).

use cm_rng::MatchRng;

// Aptitude indices (record offset - 0x0f), record order.
const GK: usize = 0;
const SW: usize = 1;
const D: usize = 2;
const DM: usize = 3;
const M: usize = 4;
const AM: usize = 5;
const F: usize = 6;
const WB: usize = 7;
const RIGHT: usize = 8;
const LEFT: usize = 9;
const CENTRAL: usize = 10;

/// A real player used as an attribute donor: CA, 12 positions, 42 attrs.
pub struct Donor {
    pub ca: i16,
    pub positions: [i8; 12],
    pub attributes: [i8; 42],
}

/// Result of position generation: the 12 aptitudes plus the two
/// footedness attributes (record +0x30 left, +0x3b right → attr indices
/// 21 and 32).
pub struct GeneratedPositions {
    pub aptitudes: [i8; 12],
    pub left_foot: i8,
    pub right_foot: i8,
}

#[inline]
fn rand(rng: &mut MatchRng, n: i32) -> i32 {
    if n <= 0 { 0 } else { rng.random(n) }
}
/// `rand(6)+15` → 15..20 (one draw).
#[inline]
fn r15_20(rng: &mut MatchRng) -> i8 { (rand(rng, 6) + 15) as i8 }
/// Nested `rand(rand(0x14))+1` (TWO draws), for footedness.
#[inline]
fn rand_rand20(rng: &mut MatchRng) -> i8 {
    let t = rand(rng, 0x14);
    (rand(rng, t) + 1) as i8
}
#[inline]
fn valid(v: i8) -> bool { (1..=20).contains(&v) }

/// Generate positions + footedness (FUN_0051f5d0 lines 1049-1343).
pub fn generate_positions(rng: &mut MatchRng) -> GeneratedPositions {
    let mut p = [1i8; 12];
    // rand(20)==0 → pure keeper.
    if rand(rng, 0x14) == 0 {
        p[GK] = 20;
        // Footedness pass still runs for keepers below? No — Pass A-C are
        // gated on GK != 20 (line 1218). A pure keeper skips them; feet
        // stay at the record default (kept/generated in the attr pass).
        return GeneratedPositions { aptitudes: p, left_foot: 0, right_foot: 0 };
    }
    // Role table (lines 1067-1216). "r" = r15_20 (one draw each, in order).
    let role = rand(rng, 0xb6);
    match role {
        0 => { p[SW] = 20; }
        1..=6 => { p[SW] = r15_20(rng); p[D] = 20; }
        7..=10 => { p[SW] = r15_20(rng); p[DM] = r15_20(rng); p[D] = 20; }
        11..=14 => { p[SW] = r15_20(rng); p[WB] = r15_20(rng); p[D] = 20; }
        15 => { p[SW] = r15_20(rng); p[DM] = r15_20(rng); p[M] = r15_20(rng); p[D] = 20; }
        16 => { p[SW] = r15_20(rng); p[WB] = r15_20(rng); p[M] = r15_20(rng); p[D] = 20; }
        17..=36 => { p[D] = 20; }
        37..=48 => { p[DM] = r15_20(rng); p[D] = 20; }
        49..=60 => { p[WB] = r15_20(rng); p[D] = 20; }
        61..=72 => { p[WB] = r15_20(rng); p[M] = r15_20(rng); p[D] = 20; }
        73..=84 => { p[DM] = r15_20(rng); p[M] = r15_20(rng); p[D] = 20; }
        85 => { p[F] = r15_20(rng); p[D] = 20; }
        86..=100 => { p[DM] = r15_20(rng); p[M] = 20; }
        101..=102 => { p[DM] = r15_20(rng); p[M] = 20; p[AM] = r15_20(rng); }
        103..=105 => { p[WB] = r15_20(rng); p[AM] = r15_20(rng); p[M] = 20; }
        106..=125 => { p[M] = 20; }
        126..=139 => { p[M] = 20; p[AM] = r15_20(rng); }
        140..=141 => { p[F] = r15_20(rng); p[AM] = r15_20(rng); p[M] = 20; }
        142 => { p[F] = r15_20(rng); p[M] = 20; }
        143..=148 => { p[M] = 20; p[AM] = 20; }
        149..=154 => { p[AM] = 20; p[F] = r15_20(rng); }
        155..=160 => { p[F] = 20; p[AM] = r15_20(rng); }
        _ => { p[F] = 20; }
    }

    // ---- Pass A: Right / Left / Central (lines 1218-1245). ----
    // GK != 20 here (pure keeper returned above).
    let mut enter_loop;
    if p[SW] < 15 {
        if p[WB] < 15 && p[RIGHT] <= 14 && p[LEFT] <= 14 {
            let r = rand(rng, 10);
            if r < 3 { p[LEFT] = 20; }
            enter_loop = true;
        } else {
            enter_loop = true;
        }
    } else {
        p[CENTRAL] = 20;
        enter_loop = true;
    }
    if enter_loop {
        let mut set_right = true;
        loop {
            if p[RIGHT] > 19 || p[LEFT] > 19 || p[CENTRAL] > 19 { set_right = false; break; }
            let r = rand(rng, 11);
            if r < 4 || rand(rng, p[CENTRAL] as i32) > 5 { p[CENTRAL] = 20; }
            let r = rand(rng, 11);
            if r > 8 || rand(rng, p[LEFT] as i32) > 5 { p[LEFT] = 20; }
            let r = rand(rng, 11);
            if !(r > 3 && rand(rng, p[RIGHT] as i32) < 6) { break; }
        }
        if set_right { p[RIGHT] = 20; }
    }
    let _ = &mut enter_loop;

    // ---- Pass B: main-position adjacency + AttMid/Fwd carry (1247-1276). ----
    if p[AM] > 14 && p[M] < 15 && p[CENTRAL] < 15 { p[M] = p[AM]; }
    p[M] = (p[AM] / 2).max(p[M]);
    p[D] = (p[SW] / 2).max(p[D]);
    if p[F] > 14 && p[CENTRAL] < 15 {
        let a = rand(rng, 11);
        let b = rand(rng, 7);
        p[AM] = ((b + 14) as i8).max(p[AM]);
        p[M] = ((a + 10) as i8).max(p[M]);
    }

    // ---- Pass C: footedness rec+0x30 (left) / +0x3b (right) (1277-1343). ----
    let mut lf = 0i8; // regen record ships both feet invalid (0)
    let mut rf = 0i8;
    // Both invalid → the `else` branch's both-invalid sub-branch:
    if p[RIGHT] < p[LEFT] { lf = 20; rf = rand_rand20(rng); }
    else { rf = 20; lf = rand_rand20(rng); }
    // (recheck rf / lf-invalid / rand(4) side-pick are all no-ops now, since
    //  exactly one foot is 20 and the other is valid.)

    GeneratedPositions { aptitudes: p, left_foot: lf, right_foot: rf }
}

/// Buffer traversal order: record offsets 0x1b..0x44 as `offset-0x1b`,
/// in the codec's slot order (FUN_00524160). The first 20 entries are the
/// CA-scaled copy slots (buffer 0xC..0x1F); the last 22 are the
/// template+jitter slots (0x20..0x35).
const ORDER: [usize; 42] = [
    21, 32, 3, 8, 9, 11, 12, 15, 16, 22, 23, 24, 26, 28, 29, 30, 31, 35, 38, 40,
    13, 7, 14, 37, 34, 33, 27, 19, 20, 10, 4, 5, 6, 0, 1, 2, 17, 18, 39, 36, 41, 25,
];
const CA_SCALED_COUNT: usize = 20;

/// Per-attribute GK flag (indexed by `offset-0x1b`): 1 = the goalkeeping
/// attribute (+0x2a), 2 = an outfield technical/physical attribute,
/// 0 = neutral. From the codec FUN_00524160 (see decode report Q2).
fn gk_flag(idx: usize) -> u8 {
    const FLAG2: [usize; 17] = [0, 7, 8, 11, 12, 13, 16, 22, 23, 24, 28, 29, 35, 36, 37, 40, 41];
    if idx == 15 { 1 } else if FLAG2.contains(&idx) { 2 } else { 0 }
}

/// Generate the 42 attributes (FUN_0051f5d0 lines 1407-1624). `our_attrs`
/// carries any already-valid attributes (e.g. the feet just set by
/// `generate_positions`), which are kept; `age` drives the young-defensive
/// tweak. Returns the 42 attributes indexed by `offset-0x1b`.
pub fn generate_attributes(
    our_ca: i16,
    our_positions: &[i8; 12],
    our_attrs: &[i8; 42],
    age: i32,
    donors: &[Donor],
    rng: &mut MatchRng,
) -> [i8; 42] {
    // 250 draws over the FULL donor set (invalids skipped, draw consumed);
    // keep the lowest-scoring donor. Fixed 250, no early exit.
    let mut best: Option<usize> = None;
    let mut best_score = i32::MAX;
    for _ in 0..250 {
        let j = rand(rng, donors.len() as i32) as usize;
        if donors.is_empty() { continue; }
        let cand = &donors[j];
        if !(1..=200).contains(&cand.ca) { continue; }
        let mut score = (our_ca as i32 - cand.ca as i32).abs() * 5 / 2;
        for p in 0..12 {
            let cp = cand.positions[p];
            if !valid(cp) {
                score += 100;
            } else if valid(our_positions[p]) {
                let d = (our_positions[p] as i32 - cp as i32).abs();
                if p == GK { score += d * 1000; }
                else if p < 9 { score += d * 25; }
                else { score += d * 16; }
            }
        }
        if score < best_score || best.is_none() {
            best_score = score;
            best = Some(j);
        }
    }

    let gk_apt = our_positions[GK];
    let mut out = [0i8; 42];
    for (k, &idx) in ORDER.iter().enumerate() {
        let flag = gk_flag(idx);
        if valid(our_attrs[idx]) {
            // KEPT: only the GK-flag decay (subtract 10, floor 1) fires.
            let mut v = our_attrs[idx] as i32;
            if (gk_apt < 15 && flag == 1) || (gk_apt >= 15 && flag == 2) {
                v = (v - 10).max(1);
            }
            out[idx] = v.clamp(1, 20) as i8;
            continue;
        }
        // GENERATE — three distinct paths (exe lines 1480-1564):
        //   copy (donor slot valid): CA-scaled or template+adjust, + jitter
        //   donor slot invalid:      rand(0xb)+CA/10-5   (no jitter)
        //   no donor at all:         rand(7)+CA/10-3     (no jitter)
        let mut v: i32 = match best {
            Some(b) if valid(donors[b].attributes[idx]) => {
                let t = donors[b].attributes[idx] as i32;
                let mut vv = if k < CA_SCALED_COUNT {
                    (our_ca as i32 - donors[b].ca as i32) / 10 + t
                } else {
                    let mut x = t;
                    if x < 6 { x += rand(rng, 5); } else if x > 14 { x -= rand(rng, 4); }
                    x
                };
                vv += rand(rng, 6) - 3; // jitter, copy path only
                vv
            }
            Some(_) => rand(rng, 0xb) + (our_ca as i32) / 10 - 5,
            None => rand(rng, 7) + (our_ca as i32) / 10 - 3,
        };
        if v < 1 { v = rand(rng, 8) + 1; } else if v >= 21 { v = 20; }
        // tweaks: consistency/important-matches boost (buf 0x9c/0x9f = k 40/41).
        if k == 40 || k == 41 {
            if v < 12 { v += rand(rng, 13 - v); }
        } else if (k == 24 || k == 25 || k == 28) && age < 22 && v > 15 {
            // young-defensive reduction (buf 0x6c/0x6f/0x78).
            v -= rand(rng, 5);
        }
        // GK-flag decay (generated path): reset to rand(10)+1.
        if (gk_apt < 15 && flag == 1) || (gk_apt >= 15 && flag == 2) {
            v = rand(rng, 10) + 1;
        }
        out[idx] = v.clamp(1, 20) as i8;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rng() -> MatchRng {
        MatchRng::new_seeded((0..4096).collect(), 0x0051_f5d0)
    }

    #[test]
    fn positions_are_valid_and_sweepers_rare() {
        let mut r = rng();
        let (mut gk, mut sweeper, mut valid_main) = (0, 0, 0);
        const N: usize = 4000;
        for _ in 0..N {
            let gp = generate_positions(&mut r);
            for &v in &gp.aptitudes {
                assert!((1..=20).contains(&v), "aptitude out of range: {v}");
            }
            if gp.aptitudes[GK] == 20 { gk += 1; }
            // a "sweeper" = SW is the top main aptitude and >= a defender
            if gp.aptitudes[SW] >= 15 { sweeper += 1; }
            if gp.aptitudes.iter().take(7).any(|&v| v == 20) { valid_main += 1; }
        }
        // Every generated player has a real main position.
        assert_eq!(valid_main, N, "some players got no main position");
        // Keepers ~1/20 of the pool. Sweeper aptitude is set by exactly
        // 17 of the 182 outfield role-buckets (≈9% of outfielders) — the
        // exe's real rate. (The synthetic test RNG table skews the modulo
        // upward; the shipped 3.4MB table hits ~9%.) The point is it is a
        // normal minority role, not the old "every zero-aptitude player is
        // a sweeper" display bug.
        let gk_pct = gk * 100 / N;
        let sw_pct = sweeper * 100 / N;
        assert!((2..=9).contains(&gk_pct), "keeper rate off: {gk_pct}%");
        assert!(sw_pct < 20, "sweeper rate implausibly high: {sw_pct}%");
    }

    #[test]
    fn attributes_copy_from_nearest_donor() {
        // A single donor at CA 90 with a distinctive profile; a generated
        // CA-90 player of the same position must come out CLOSE to it
        // (nearest-neighbour copy), not a flat low spread.
        let mut donor_attrs = [10i8; 42];
        donor_attrs[20] = 18; // finishing (+0x2f-ish slot) high
        let donors = vec![Donor {
            ca: 90,
            positions: {
                let mut p = [1i8; 12];
                p[F] = 20; // attacker
                p
            },
            attributes: donor_attrs,
        }];
        let mut our_pos = [1i8; 12];
        our_pos[F] = 20;
        let mut r = rng();
        let attrs = generate_attributes(90, &our_pos, &[0i8; 42], 25, &donors, &mut r);
        for &v in &attrs {
            assert!((1..=20).contains(&v), "attr out of range: {v}");
        }
        // With one CA-matched donor the copy should track ~10 (donor base)
        // plus small jitter for most slots — mean well above the old flat
        // CA/10-3 = 6 harshness.
        let mean: i32 = attrs.iter().map(|&v| v as i32).sum::<i32>() / 42;
        assert!(mean >= 7, "attributes too harsh, mean {mean}");
    }
}
