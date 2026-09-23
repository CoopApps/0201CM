//! CM0102 manager/AI scoring primitives — **pure**, no deps, so they build
//! without cm-domain's LLVM-OOM burden and can be differentially verified against
//! `cm0102.exe` by the harness in `tools/exe_diff/`. cm-domain re-exports these as
//! the single canonical implementation (`manager_hiring.rs`).
//!
//! Every function here is byte-exact against the DirectDraw build at the cited VA.
//! Float constants: docs/manager_hiring/rating_constants.md. Diff report:
//! reports/manager_hiring_diff.md.

#[inline]
pub fn i16_at(b: &[u8], o: usize) -> i16 {
    b.get(o..o + 2).map(|s| i16::from_le_bytes([s[0], s[1]])).unwrap_or(0)
}

/// Resolved manager↔club relationship inputs for the closeness classifier
/// `FUN_0052a410`. The cross-record lookups (nation/league equality, and
/// `ref_known_in_* = FUN_005274d0`) are resolved by the caller — the harness
/// captures FUN_005274d0's actual return via `hook_call`, so this stays
/// byte-exact and independent of the game-data pools.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClosenessInputs {
    pub ref_null: bool,
    pub person_has_club: bool,
    pub same_club: bool,
    pub same_club_nation: bool,
    pub same_person_nation: bool,
    pub ref_known_in_person_nation: bool,
    pub ref_known_in_person_club_nation: bool,
    pub regional_rep_pass: bool,
}

/// `FUN_0052a410` — closeness code {0,2,3,4}. Decision tree verbatim from the
/// disassembly (docs/manager_hiring/rating_constants.md §7, hiring spec §1).
pub fn closeness_class(i: &ClosenessInputs) -> u8 {
    if i.ref_null { return 4; }
    if i.person_has_club {
        if i.same_club { return 0; }
        if i.same_club_nation { return 0; }
    }
    if i.same_person_nation { return 2; }
    if !i.ref_known_in_person_nation { return 2; }
    if i.person_has_club && !i.ref_known_in_person_club_nation { return 3; }
    if i.regional_rep_pass { return 3; }
    4
}

/// `FUN_0052a330` — reputation-fit base rating. Picks/averages one short from the
/// person's 3-entry reputation vector `block`, by closeness code `c`.
/// `mode0=true` → +0x61 block, odd offsets 9/0xb/0xd; else +0x69 block, even
/// 8/0xa/0xc. Slots national/home/world. Codes 3 and 1 average two slots, each
/// `/2` as a short before summing. Pure integer — confirmed zero x87.
pub fn manager_club_repfit(block: &[u8], mode0: bool, c: u8) -> i32 {
    let (nat, home, world) = if mode0 { (9usize, 0xb, 0xd) } else { (8usize, 0xa, 0xc) };
    let s = |o: usize| i16_at(block, o) as i32;
    match c {
        4 => s(world),
        2 => s(nat),
        0 => s(home),
        3 => (s(world) / 2) + (s(nat) / 2),
        _ => (s(home) / 2) + (s(nat) / 2), // c == 1
    }
}

/// Truncation toward zero — the exe's float→int helper `0x009346d0` is `__ftol`
/// (C `(long)` cast), which CHOPS toward zero, NOT round-to-nearest. Differentially
/// confirmed against cm0102.exe (3000×1.0025=3007.5 → 3007, not 3008). Rust `as i32`
/// truncates toward zero for in-range values; this wrapper documents intent.
#[inline]
pub fn ftol(x: f64) -> i32 { x.trunc() as i32 }

/// `FUN_00682420` lines 88/92 — affinity base re-weight. Positive affinity →
/// `max(L*1.1, L+2000)`, negative → `L*0.25`, else unchanged; then `__ftol`
/// (truncate). Constants 1.1/2000/0.25 (rating_constants §5.1).
pub fn score_base_reweight(l: i32, affinity_positive: bool, affinity_negative: bool) -> i32 {
    let lf = l as f64;
    let out = if affinity_positive {
        (lf * 1.1).max(lf + 2000.0)
    } else if affinity_negative {
        lf * 0.25
    } else {
        return l;
    };
    ftol(out)
}

/// `FUN_00682420` attr term: `L = ftol( L * (1.0 + attr * w) )` — one signed-byte
/// attribute re-weight (attr-block 0x682bc4+ and the person-attribute terms). The
/// weight `w` is a recovered reciprocal (rating_constants §2). Differentially
/// verified against the exe L-trajectory.
#[inline]
pub fn score_attr_term(l: i32, attr: i8, w: f64) -> i32 {
    ftol(l as f64 * (1.0 + attr as f64 * w))
}

/// `FUN_00682420` integer **skeleton** (pre-switch + the case-5/0xc no-op path):
/// base rating (from repfit, supplied as `base`), the standing bonus, the
/// incumbency bonus, then the two affinity base re-weights. With every attribute
/// source byte zero and status 5 the per-jobtype float factors all collapse to
/// 1.0, so the function returns exactly this. Verified differentially against
/// cm0102.exe (docs/manager_hiring/diff_report.md). The per-jobtype term products
/// are added in `score_apply_terms` (chunked, still in progress) — NOT here.
///
/// Args mirror the decompile: `status`=local_8e, `club_rep`=*(club+0x80),
/// `standing_c/_4/_6` = *(standing+0xc/+4/+6), `incumbent`= person+0x39==club,
/// `incumbent_active`= person.id < epoch-0x10, `ambition`= person+0x59,
/// `aff_a/_b` = the two affinity gate outcomes (Some(true)=positive → max(L*1.1,
/// L+2000); Some(false)=negative → L*0.25; None = gate not taken).
#[allow(clippy::too_many_arguments)]
pub fn score_skeleton(
    base: i32, status: u8, club_rep: i32, standing_c: i16, standing_4: i16, standing_6: i16,
    incumbent: bool, incumbent_active: bool, ambition: i32,
    aff_a: Option<bool>, aff_b: Option<bool>,
) -> i32 {
    let mut l = base;
    if (status == 0x0c || status == 0x05) && club_rep > 0x109a && (standing_c as i32) > 0xcb2 {
        l += standing_4 as i32 * 0x19;   // *(standing+4) * 25
        l += standing_6 as i32 * 5;      // *(standing+6) * 5
    }
    if incumbent {
        l += if incumbent_active { ambition * ambition } else { 100 };
    }
    if let Some(pos) = aff_a { l = score_base_reweight(l, pos, !pos); }
    if let Some(pos) = aff_b { l = score_base_reweight(l, pos, !pos); }
    l
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repfit_basic() {
        let mut b = [0u8; 0x10];
        b[0x08..0x0a].copy_from_slice(&3000i16.to_le_bytes());
        b[0x0a..0x0c].copy_from_slice(&6000i16.to_le_bytes());
        b[0x0c..0x0e].copy_from_slice(&9000i16.to_le_bytes());
        assert_eq!(manager_club_repfit(&b, false, 4), 9000);
        assert_eq!(manager_club_repfit(&b, false, 3), 9000 / 2 + 3000 / 2);
    }
}
