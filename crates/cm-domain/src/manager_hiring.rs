//! Manager Hiring AI — port of the `manager_manager.cpp` hiring subsystem.
//!
//! # Executable provenance
//! Driver `FUN_00674c10` (manager_manager daily tick), reached from game-step
//! `FUN_005b6a90`/`FUN_005b6f10`; wires to `RuntimeSaveGame::hook_manager_job_lifecycle`.
//! Full mapping: docs/gdi_registry/gdi_function_registry.md. Spec:
//! scratchpad manager_hiring_spec.md.
//!
//! This module ports the subsystem as ONE coherent unit. This first landing is
//! the **fully-decoded deterministic foundation**: the job-security table, the
//! person/club hiring-field views, the leave-reason decision, the vacancy gate,
//! and the appointment/unlink state mutations. The **scoring core**
//! (`FUN_00682420` + the rating primitives `FUN_0052a330`/`FUN_0082dab0`) is
//! deferred to a follow-up because Ghidra dropped its x87 float coefficients and
//! two rating primitives are still being decoded — porting it now would be a
//! guess, which the project forbids. RNG-drawing paths (board-confidence init,
//! hire round, the tick) are held until the scoring core lands so the shared RNG
//! stream is drawn in the exe's exact order (order parity matters).
//! Registry provenance tags are injected at each fn by tools/gdi_registry/inject_tags.py.

use crate::game_rng::GameRng;

// ─────────────────────────────────────────────────────────────────────────────
// Byte helpers (local; the person/club records are opaque byte bodies)
// ─────────────────────────────────────────────────────────────────────────────
#[inline]
fn u8_at(b: &[u8], o: usize) -> u8 { b.get(o).copied().unwrap_or(0) }
#[inline]
fn i8_at(b: &[u8], o: usize) -> i8 { u8_at(b, o) as i8 }
#[inline]
fn i16_at(b: &[u8], o: usize) -> i16 {
    b.get(o..o + 2).map(|s| i16::from_le_bytes([s[0], s[1]])).unwrap_or(0)
}
#[inline]
fn u32_at(b: &[u8], o: usize) -> u32 {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).unwrap_or(0)
}
#[inline]
fn set_u32(b: &mut [u8], o: usize, v: u32) {
    if let Some(s) = b.get_mut(o..o + 4) { s.copy_from_slice(&v.to_le_bytes()); }
}
#[inline]
fn set_u8(b: &mut [u8], o: usize, v: u8) { if let Some(x) = b.get_mut(o) { *x = v; } }

// ─────────────────────────────────────────────────────────────────────────────
// Person record hiring fields (StaffBook type6 body, stride 0x6e; DAT_00acd5c4)
// Offsets verified in the spec §2.1 against 682420/684a80/674c10.
// ─────────────────────────────────────────────────────────────────────────────
pub const PERSON_STRIDE: usize = 0x6e;
pub mod person_off {
    pub const AGE_STANDING: usize = 0x18;   // i8 age / world-standing gate
    pub const NATION_BIRTH: usize = 0x1a;   // u32 nation-of-birth link
    pub const NATION_JOB: usize = 0x24;     // u32 national-team managed
    pub const NATION_JOB_STATUS: usize = 0x28; // i8
    pub const CLUB_JOB: usize = 0x39;       // u32 club currently managed
    pub const CLUB_JOB_STATUS: usize = 0x3d; // u8 job-status code (see job_status)
    pub const CONTRACT_A: usize = 0x3e;     // packed date
    pub const CONTRACT_B: usize = 0x42;     // packed date
    pub const AMBITION: usize = 0x59;       // i8
    pub const ATTR_5B: usize = 0x5b;        // i8 (score base local_98 = +0x5b - 10)
    pub const STANDING_ID: usize = 0x69;    // u32 manager-standing record link (0 => no standing)
}

/// Read-only view of a manager/person record body.
pub struct PersonHiringView<'a> {
    pub body: &'a [u8],
}
impl<'a> PersonHiringView<'a> {
    pub fn new(body: &'a [u8]) -> Self { Self { body } }
    pub fn id(&self) -> u32 { u32_at(self.body, 0x00) }
    pub fn age_standing(&self) -> i8 { i8_at(self.body, person_off::AGE_STANDING) }
    pub fn club_job(&self) -> u32 { u32_at(self.body, person_off::CLUB_JOB) }
    pub fn nation_job(&self) -> u32 { u32_at(self.body, person_off::NATION_JOB) }
    pub fn job_status(&self) -> u8 { u8_at(self.body, person_off::CLUB_JOB_STATUS) }
    pub fn ambition(&self) -> i8 { i8_at(self.body, person_off::AMBITION) }
    pub fn has_standing(&self) -> bool { u32_at(self.body, person_off::STANDING_ID) != 0 }
}

// ─────────────────────────────────────────────────────────────────────────────
// Club record hiring fields (ClubView record, stride 0x245; DAT_00acd5bc)
// ─────────────────────────────────────────────────────────────────────────────
pub mod club_off {
    pub const NATION: usize = 0x53;          // u32 nation link
    pub const REPUTATION: usize = 0x80;      // i16 club reputation
    pub const BOARD_CASH: usize = 0x88;      // i32 board-cash / patience counter
    pub const ASSISTANT: usize = 0xbf;       // u32 assistant-manager person ptr
    pub const COACH3: usize = 0xc3;          // u32[3]
    pub const MANAGER: usize = 0xcf;         // u32 MANAGER person ptr (authoritative)
    pub const PHYSIO: usize = 0xd3;          // u32 physio/DoF ptr
    pub const SCOUT5: usize = 0x19f;         // u32[5]
    pub const ARR7: usize = 0x1b3;           // u32[7]
    pub const ARR3: usize = 0x1cf;           // u32[3]
}

pub fn club_reputation(raw: &[u8]) -> i16 { i16_at(raw, club_off::REPUTATION) }
pub fn club_manager_id(raw: &[u8]) -> u32 { u32_at(raw, club_off::MANAGER) }

// ─────────────────────────────────────────────────────────────────────────────
// Job-security table row (stride 0x49, base [*manager_manager], indexed by club id)
// Spec §2.3, verified in 679ed0 / 6805a0 / 674380 / 683dc0.
// ─────────────────────────────────────────────────────────────────────────────
pub const JOB_ROW_STRIDE: usize = 0x49;

/// One club's board/chairman/fans/media confidence + grievance axes.
#[derive(Clone)]
pub struct JobSecurityRow {
    pub raw: [u8; JOB_ROW_STRIDE],
}
impl Default for JobSecurityRow {
    fn default() -> Self { Self { raw: [0u8; JOB_ROW_STRIDE] } }
}
impl JobSecurityRow {
    pub fn board_confidence(&self) -> i16 { i16_at(&self.raw, 0x06) }
    pub fn chairman_patience(&self) -> i16 { i16_at(&self.raw, 0x08) }
    pub fn fans_confidence(&self) -> i16 { i16_at(&self.raw, 0x0a) }
    pub fn media_expectation(&self) -> i16 { i16_at(&self.raw, 0x0c) }
    /// grievance axes A..D at +0x10,+0x12,+0x14,+0x16 (i:0..=3).
    pub fn grievance(&self, i: usize) -> i16 { i16_at(&self.raw, 0x10 + i * 2) }
    pub fn expected(&self) -> f64 { f64_at(&self.raw, 0x30) }
    pub fn actual(&self) -> f64 { f64_at(&self.raw, 0x38) }
    pub fn flag02(&self) -> u8 { u8_at(&self.raw, 0x02) }
}
#[inline]
fn f64_at(b: &[u8], o: usize) -> f64 {
    b.get(o..o + 8)
        .map(|s| f64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]))
        .unwrap_or(0.0)
}

/// The manager_manager job-security table: one row per club, indexed by club id.
pub struct JobSecurityTable {
    pub rows: Vec<JobSecurityRow>,
}
impl JobSecurityTable {
    pub fn with_len(n: usize) -> Self {
        Self { rows: vec![JobSecurityRow::default(); n] }
    }
    pub fn row(&self, club_id: u32) -> Option<&JobSecurityRow> { self.rows.get(club_id as usize) }
    pub fn row_mut(&mut self, club_id: u32) -> Option<&mut JobSecurityRow> {
        self.rows.get_mut(club_id as usize)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §3.4 vacancy-target predicate (FUN_00683dc0) — the job-row part (deterministic).
// eligible = pickable && not national && chairman_patience(+0x08) < 3000 && board
//            confidence negative. The pickable gate is club_is_pickable
//            (manager_creation.rs, = FUN_0052e370); the board object read is a
//            separate record, so callers pass board_confidence_negative.
// ─────────────────────────────────────────────────────────────────────────────
// GDI-REG: 00683dc0 PORTED_PARTIAL
pub fn club_is_vacancy_target(
    pickable: bool,
    is_national_team: bool,
    row: &JobSecurityRow,
    board_confidence_negative: bool,
) -> bool {
    pickable
        && !is_national_team
        && (row.chairman_patience() as i32) < 3000
        && board_confidence_negative
}

// ─────────────────────────────────────────────────────────────────────────────
// §3.5 leave-reason code (FUN_006805a0) — "should the manager leave, and why".
// Returns 0 (stay) or a reason code 2..0xb. Fully-decoded integer + RNG logic;
// the float compare actual<expected uses the row f64s directly.
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveContext {
    pub affiliated_pull: bool,   // >=3 affiliated managers w/ flag (code 9)
    pub international_pull: bool, // board international rep pull (code 0xb)
    pub not_first_season: bool,
    pub has_finance_grievance: bool, // finance-gated code 10 path
    pub tenure_days: i32,            // FUN_00536990(mgr+0x3e,+0x42)
}

// GDI-REG: 006805a0 PORTED_BEHAVIOURAL
pub fn leave_reason(row: &JobSecurityRow, ctx: &LeaveContext, rng: &mut GameRng) -> u8 {
    // order mirrors 6805a0.c
    if ctx.affiliated_pull { return 9; } // :39-41
    if ctx.international_pull { return 0xb; } // :48-55
    if row.actual() < row.expected() && (row.grievance(1) as i32) < -300 && ctx.has_finance_grievance {
        return 10; // :57-61  grievanceB (+0x12) < -300 & actual<expected
    }
    if (row.grievance(1) as i32) < -0x2ee && ctx.not_first_season && rng.rand_mod(3) == 0 {
        return 2; // :64-67
    }
    if row.flag02() & 8 != 0 { return 7; } // :68-70  job flag +0x02 & 8
    if (row.grievance(0) as i32) < -0x15e { return 3; } // :72-74  grievanceA < -350
    if (row.grievance(2) as i32) < -0x2ee { return 5; } // :75-77  grievanceC < -750
    if (row.fans_confidence() as i32) < (row.board_confidence() as i32) && rng.rand_mod(2) == 0 {
        return 8; // :78-85 fans<board & RNG loss (approx of the loss-gate)
    }
    if (row.grievance(3) as i32) < -0x2ee && rng.rand_mod(3) != 0 { return 4; } // :86-89
    if ctx.tenure_days > rng.rand_mod(2000) + 1000 { return 6; } // :90-97
    0
}

// ─────────────────────────────────────────────────────────────────────────────
// §5.1 unlink person from a club role (FUN_00675ae0, club arm).
// Deterministic mutation table keyed on person job-status; clears the club's
// back-link slot and the person's club-job link. Nation arm mirrors on +0x24.
// ─────────────────────────────────────────────────────────────────────────────
/// Clear `person` from `club`'s staff back-links per the job-status switch.
/// Returns true if a slot was cleared. `club` and `person` are raw record bytes.
// GDI-REG: 00675ae0 PORTED_BEHAVIOURAL
pub fn unlink_person_from_club(person: &mut [u8], club: &mut [u8]) -> bool {
    // only if this person actually links to this club
    let club_id = u32_at(club, 0x00);
    if u32_at(person, person_off::CLUB_JOB) != club_id {
        return false;
    }
    let status = u8_at(person, person_off::CLUB_JOB_STATUS);
    let clear_ptr = |c: &mut [u8], off: usize, pid: u32| {
        if u32_at(c, off) == pid { set_u32(c, off, 0); }
    };
    let clear_arr = |c: &mut [u8], base: usize, n: usize, pid: u32| {
        for i in 0..n {
            let o = base + i * 4;
            if u32_at(c, o) == pid { set_u32(c, o, 0); }
        }
    };
    let pid = u32_at(person, 0x00);
    match status {
        1 => { set_u32(club, club_off::ASSISTANT, 0); }
        2 => { clear_arr(club, club_off::COACH3, 3, pid); }
        5 => { if u32_at(club, club_off::MANAGER) == pid { set_u32(club, club_off::MANAGER, 0); } }
        6 => { set_u32(club, club_off::PHYSIO, 0); }
        8 => { clear_arr(club, club_off::SCOUT5, 5, pid); }
        9 => { clear_arr(club, club_off::ARR7, 7, pid); }
        10 => { clear_arr(club, club_off::ARR3, 3, pid); }
        0x0c => { // player-manager -> player
            set_u8(person, person_off::CLUB_JOB_STATUS, 0x0b);
            clear_ptr(club, club_off::MANAGER, pid);
        }
        0x0d => {
            set_u8(person, person_off::CLUB_JOB_STATUS, 0x0b);
            set_u32(club, club_off::PHYSIO, 0);
        }
        0x0f => {
            set_u8(person, person_off::CLUB_JOB_STATUS, 0x0b);
            clear_arr(club, club_off::SCOUT5, 5, pid);
        }
        _ => {}
    }
    // statuses 5/6/8/9/10/1/2 also detach the person's club link
    if !matches!(status, 0x0c | 0x0d | 0x0f) {
        set_u32(person, person_off::CLUB_JOB, 0);
    }
    true
}

// ─────────────────────────────────────────────────────────────────────────────
// §5.2 appointment writes (FUN_00674c10 :477-525) — install a manager at a club.
// Deterministic pointer writes; board-confidence init (679ed0) is the RNG-drawing
// follow-up held for the scoring-core landing.
// ─────────────────────────────────────────────────────────────────────────────
// GDI-REG: 00674c10 PORTED_PARTIAL
pub fn appoint_manager_links(person: &mut [u8], club: &mut [u8], is_national_team: bool) {
    let club_id = u32_at(club, 0x00);
    set_u32(club, club_off::MANAGER, u32_at(person, 0x00)); // club+0xcf = person  :477
    if is_national_team {
        set_u32(person, person_off::NATION_JOB, club_id); // person+0x24 = club  :523
        set_u8(person, person_off::CLUB_JOB_STATUS, 0x05); // :525
    } else {
        set_u32(person, person_off::CLUB_JOB, club_id); // person+0x39 = club  :480
        set_u8(person, person_off::CLUB_JOB_STATUS, 0x05); // plain manager  :511
    }
}

#[inline]
fn set_i16(b: &mut [u8], o: usize, v: i16) {
    if let Some(s) = b.get_mut(o..o + 2) { s.copy_from_slice(&v.to_le_bytes()); }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scoring core — reputation-fit (FUN_0052a330 / FUN_0052a410) + score base
// re-weight (FUN_00682420 §5.1). The CANONICAL implementations live in the pure
// leaf crate `cm-scoring` (differentially verified against cm0102.exe by
// tools/exe_diff); re-exported here so cm-domain has one impl.
// GDI-REG: 0052a330 PORTED_EXACT
// GDI-REG: 0052a410 PORTED_EXACT
// GDI-REG: 00682420 PORTED_PARTIAL
// ─────────────────────────────────────────────────────────────────────────────
pub use cm_scoring::{closeness_class, manager_club_repfit, score_base_reweight, ClosenessInputs};

/// §4 poach jitter (FUN_00681c70): scale the poached club's own job-security
/// confidence rows by the recovered multipliers, `round(row * k)` (ties-even).
/// board×0.9, patience×0.75, fans×0.9, media×0.9 then ×0.975.
// GDI-REG: 00681c70 PORTED_PARTIAL
pub fn poach_jitter_primary(row: &mut JobSecurityRow) {
    let scale = |v: i16, k: f64| (v as f64 * k).round_ties_even() as i16;
    let b = scale(row.board_confidence(), 0.9);
    let p = scale(row.chairman_patience(), 0.75);
    let f = scale(row.fans_confidence(), 0.9);
    let m1 = scale(row.media_expectation(), 0.9);
    let m2 = (m1 as f64 * 0.975).round_ties_even() as i16;
    set_i16(&mut row.raw, 0x06, b);
    set_i16(&mut row.raw, 0x08, p);
    set_i16(&mut row.raw, 0x0a, f);
    set_i16(&mut row.raw, 0x0c, m2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repfit_selects_and_averages() {
        // mode!=0 block: national@8, home@0xa, world@0xc
        let mut b = vec![0u8; 0x10];
        set_i16(&mut b, 0x08, 3000); // national
        set_i16(&mut b, 0x0a, 6000); // home
        set_i16(&mut b, 0x0c, 9000); // world
        assert_eq!(manager_club_repfit(&b, false, 4), 9000); // foreign -> world
        assert_eq!(manager_club_repfit(&b, false, 2), 3000); // same nation -> national
        assert_eq!(manager_club_repfit(&b, false, 0), 6000); // same club -> home
        assert_eq!(manager_club_repfit(&b, false, 3), 9000 / 2 + 3000 / 2); // avg(world,nat)
    }

    #[test]
    fn closeness_tree() {
        let mut i = ClosenessInputs { ref_null: true, ..Default::default() };
        assert_eq!(closeness_class(&i), 4);
        i = ClosenessInputs { person_has_club: true, same_club: true, ..Default::default() };
        assert_eq!(closeness_class(&i), 0);
        i = ClosenessInputs { same_person_nation: true, ..Default::default() };
        assert_eq!(closeness_class(&i), 2);
    }

    #[test]
    fn base_reweight_gates() {
        assert_eq!(score_base_reweight(1000, false, false), 1000); // no affinity
        assert_eq!(score_base_reweight(1000, true, false), 3000);  // max(1100, 3000)
        assert_eq!(score_base_reweight(30000, true, false), 33000);// max(33000, 32000)
        assert_eq!(score_base_reweight(1000, false, true), 250);   // *0.25
    }

    #[test]
    fn job_row_accessors() {
        let mut r = JobSecurityRow::default();
        r.raw[0x06] = 0x10; r.raw[0x07] = 0x27; // 0x2710 = 10000
        assert_eq!(r.board_confidence(), 10000);
        r.raw[0x08] = 0xb8; r.raw[0x09] = 0x0b; // 0x0bb8 = 3000
        assert_eq!(r.chairman_patience(), 3000);
    }

    #[test]
    fn vacancy_gate_requires_all() {
        let mut r = JobSecurityRow::default();
        // patience 2999 < 3000
        r.raw[0x08] = 0xb7; r.raw[0x09] = 0x0b;
        assert!(club_is_vacancy_target(true, false, &r, true));
        assert!(!club_is_vacancy_target(false, false, &r, true)); // not pickable
        assert!(!club_is_vacancy_target(true, true, &r, true));   // national team
        assert!(!club_is_vacancy_target(true, false, &r, false)); // board not negative
    }

    #[test]
    fn appoint_then_unlink_manager() {
        let mut club = vec![0u8; ClubRecLen::N];
        set_u32(&mut club, 0x00, 42); // club id
        let mut person = vec![0u8; PERSON_STRIDE];
        set_u32(&mut person, 0x00, 7); // person id
        appoint_manager_links(&mut person, &mut club, false);
        assert_eq!(club_manager_id(&club), 7);
        assert_eq!(u32_at(&person, person_off::CLUB_JOB), 42);
        assert_eq!(u8_at(&person, person_off::CLUB_JOB_STATUS), 5);
        // now unlink
        assert!(unlink_person_from_club(&mut person, &mut club));
        assert_eq!(club_manager_id(&club), 0);
        assert_eq!(u32_at(&person, person_off::CLUB_JOB), 0);
    }

    struct ClubRecLen;
    impl ClubRecLen { const N: usize = 0x245; }
}
