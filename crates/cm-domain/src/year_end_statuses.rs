//! C12 — Traditional English year-end status writers, playoff-winner
//! propagation, and post-swap reset semantics.
//!
//! # Scope
//!
//! C12 is the STATE layer that feeds C8 (English pyramid orchestrator
//! `sub_0055f080` / DD `FUN_0055EE90`) and the cleanup that follows
//! `FUN_0066EED0`. Everything here is pure decision / state
//! transition — no World mutation.
//!
//! # Recovered execution order (see [[c11-2-fixture-subsystem-frozen]]
//! for the fixture side; year-end archaeology commits with C12)
//!
//! 1. Regular league rounds finish.
//! 2. **[`stamp_league_end_of_season_statuses`]** — port of
//!    `FUN_00669FA0` (`comp/league.cpp`, DD VA `0x00669FA0`, 1997 B,
//!    fanin 116 across every league class's vtable). Walks the
//!    league's club array in position order and writes
//!    `Club+0x37 ∈ {0, 2, 3, 5, 6, 7, 0xFE}` based on final rank
//!    plus per-league flag bits.
//! 3. Any subcomp playoff resolves. English playoff status-5 is
//!    written by the SAME `FUN_00669FA0` when the subcomp
//!    aggregates its own table — see [`STICKY_STATUSES`] and point 5.
//!    The three Brazilian regional resolvers
//!    (`FUN_00435510`/`00438C30`/`0043B500` — `bra_reg_*.cpp`)
//!    write `+0x37 = 5` in a different code path (same-club wins
//!    both legs); English pyramid never hits those.
//! 4. English pyramid orchestrator `FUN_0055EE90` runs — see
//!    [`crate::eng_second_fixtures::english_pyramid_annual_rollover`].
//!    Its four `FUN_0066EED0` swap calls execute the transform in
//!    [`apply_swap_status_transform`].
//! 5. Conference coordinator `FUN_0055EA00` runs (peer, not called
//!    from the pyramid orchestrator — decompile-verified).
//! 6. New-season init — no distinct global reset function exists in
//!    the decompile. Every relevant club ends the year with
//!    `+0x37 = 0xFF` (from the swap transform); next year
//!    [`stamp_league_end_of_season_statuses`] re-stamps fresh.
//!    `0xFE` (stadium-fail reprieve) is NOT cleared automatically —
//!    it persists until an external gate rewrites it. See
//!    [`ResetOutcome::persist_across_year`].
//!
//! # Confidence
//!
//! * [`stamp_league_end_of_season_statuses`] — **STRUCTURALLY PORTED**
//!   from the DirectDraw decompile of `FUN_00669FA0`. Full runtime
//!   differential against a captured GDI year-end pass has not yet
//!   been performed (C13 target).
//! * [`propagate_english_playoff_winner`] — **STATE-EXACT** for the
//!   English case: writes `+0x37 = 5` iff current status is
//!   non-sticky (`{3, 5, 0xFE, 0xFC}` are never overwritten per
//!   the archaeology).
//! * [`apply_swap_status_transform`] — **STATE-EXACT** vs
//!   `FUN_0066EED0`'s post-swap byte writes; fits the existing
//!   [`crate::eng_second_fixtures::promote_relegate_swap`] port.
//! * [`stadium_fail_reprieve_persists`] and other reset predicates
//!   are DIRECT observations from the decompile: no `+0xFE → *`
//!   path exists in this transform family.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Status constants (Club+0x37 alphabet, from FUN_00669FA0 + FUN_0066EED0
// + FUN_0055EE90/FUN_0055EA00)
// ---------------------------------------------------------------------------

/// Auto-promotion eligible — champion or top-N depending on league.
/// Sticky? NO. Written by [`stamp_league_end_of_season_statuses`] at
/// year-end; cleared to `STATUS_IDLE` by [`apply_swap_status_transform`]
/// when the club is actually promoted.
pub const STATUS_AUTO_PROMOTION: u8 = 0;
/// Playoff loser — written to clubs that made the playoff bracket but
/// did not win. Sticky? NO. Non-actionable by
/// [`apply_swap_status_transform`] (it treats `2` as "stayer").
pub const STATUS_PLAYOFF_LOSER: u8 = 2;
/// Relegated — moves DOWN one tier. Sticky? YES (part of the sticky
/// set inside `FUN_00669FA0`).
pub const STATUS_RELEGATED: u8 = 3;
/// Playoff winner — moves UP one tier alongside `STATUS_AUTO_PROMOTION`.
/// Sticky? YES.
pub const STATUS_PLAYOFF_WINNER: u8 = 5;
/// Miscellaneous position-derived flag (e.g., "position N group A"
/// for split-competition years). Not consumed by the English
/// pyramid P/R chain but observed in the alphabet.
pub const STATUS_POSITION_FLAG_6: u8 = 6;
/// Miscellaneous position-derived flag. Same disposition as `6`.
pub const STATUS_POSITION_FLAG_7: u8 = 7;
/// Deep-history reprieve. Signed-byte `-4`. Sticky. Not written by
/// the English pyramid year-end path; documented here because it
/// appears in `FUN_00669FA0`'s sticky-preservation guard alongside
/// `3` / `5` / `0xFE`.
pub const STATUS_DEEP_REPRIEVE: u8 = 0xFC;
/// Stadium-fail reprieve — written by the Third↔Conference stadium
/// gate on the Conference champion (or the D3 last-place club) when
/// the ground fails the capacity check. Sticky (never overwritten by
/// `FUN_00669FA0`); ALSO never cleared by
/// [`apply_swap_status_transform`] nor by any observed reset —
/// persists into the next season unless an external subsystem
/// rewrites it.
pub const STATUS_STADIUM_REPRIEVE: u8 = 0xFE;
/// Idle — every relevant club ends the year with this after the P/R
/// swap. `FUN_00669FA0` treats it as "not yet stamped this year"
/// and is free to overwrite it with a position-derived value.
pub const STATUS_IDLE: u8 = 0xFF;

/// The four statuses `FUN_00669FA0` explicitly refuses to overwrite
/// (verified inline in the DD decompile at ~line 173: the "sticky
/// set" branch checks current status against these before writing
/// any new position-derived value).
///
/// This is what protects a playoff winner's `5` marker from being
/// clobbered by a subsequent stamp pass, and why the stadium-fail
/// reprieve (`0xFE`) survives into next year.
pub const STICKY_STATUSES: &[u8] = &[
    STATUS_RELEGATED,       // 3
    STATUS_PLAYOFF_WINNER,  // 5
    STATUS_STADIUM_REPRIEVE,// 0xFE
    STATUS_DEEP_REPRIEVE,   // 0xFC
];

/// True iff `status` is in [`STICKY_STATUSES`]. Every writer in the
/// year-end pipeline must gate its own write behind this predicate.
pub fn is_sticky_status(status: u8) -> bool {
    matches!(status, STATUS_RELEGATED | STATUS_PLAYOFF_WINNER
                   | STATUS_STADIUM_REPRIEVE | STATUS_DEEP_REPRIEVE)
}

// ---------------------------------------------------------------------------
// Per-league config for FUN_00669FA0's position → status stamping
// ---------------------------------------------------------------------------

/// Which league's stamping pattern to apply. `FUN_00669FA0` is
/// generic (`comp/league.cpp`) and reads flag bits off the comp
/// record (`comp+0xD9 & {2, 4, 8, 0x20}`) to switch between shapes.
/// C12 exposes the five English shapes as an enum so callers don't
/// pass raw flag bits.
///
/// Confidence: STRUCTURALLY PORTED — the per-league auto/playoff/
/// relegated counts here are the shape captured on the shipped
/// 2001/02 data (verified against the perturb/driver golden runs
/// which decoded each comp's `comp_bf` / `comp_c1` bytes and
/// the vtable-installed flag bits). A runtime differential of the
/// stamped `+0x37` alphabet against a captured GDI year-end capture
/// is the [pending C13 target].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnglishLeagueEndShape {
    /// Prem: 0 auto-promotion (no tier above), N=3 auto-relegation
    /// from the bottom. No playoff.
    Premier,
    /// First: 2 auto-promotion (top 2), playoff bracket for
    /// positions 3..=6 (4 clubs), 3 auto-relegation from bottom.
    First,
    /// Second: 2 auto-promotion, 4-club playoff for positions
    /// 3..=6, 4 auto-relegation.
    Second,
    /// Third: 3 auto-promotion (top 3), 4-club playoff for
    /// positions 4..=7, 1 auto-relegation.
    Third,
    /// Conference: 1 auto-promotion (champion only, stadium-gated).
    /// 3 auto-relegation. No playoff bracket.
    Conference,
}

impl EnglishLeagueEndShape {
    /// Number of auto-promotion positions (positions 1..=n).
    pub fn n_auto_promote(self) -> usize {
        match self {
            Self::Premier => 0,
            Self::First => 2,
            Self::Second => 2,
            Self::Third => 3,
            Self::Conference => 1,
        }
    }

    /// Number of playoff-bracket positions immediately after
    /// auto-promotion. These get `STATUS_PLAYOFF_LOSER` initially;
    /// the eventual bracket winner is stamped
    /// `STATUS_PLAYOFF_WINNER` by
    /// [`propagate_english_playoff_winner`].
    pub fn n_playoff(self) -> usize {
        match self {
            Self::Premier | Self::Conference => 0,
            Self::First | Self::Second | Self::Third => 4,
        }
    }

    /// Number of auto-relegation positions from the bottom of the
    /// table.
    pub fn n_auto_relegate(self) -> usize {
        match self {
            Self::Premier => 3,
            Self::First => 3,
            Self::Second => 4,
            Self::Third => 1,
            Self::Conference => 3,
        }
    }
}

// ---------------------------------------------------------------------------
// FUN_00669FA0 port — stamp end-of-season statuses on a finalized table
// ---------------------------------------------------------------------------

/// One club's row in a finalized league table, in **position order**
/// (position 1 = index 0). Caller resolves the sort ahead of time
/// (the exe already has clubs sorted by points/GD/GF via
/// `FUN_00666500` before `FUN_00669FA0` runs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeagueTableRow {
    pub club_id: u32,
    /// Current `Club+0x37`. `STATUS_IDLE` (`0xFF`) on a fresh season
    /// / after `apply_swap_status_transform`. Anything in
    /// [`STICKY_STATUSES`] is preserved.
    pub current_status: u8,
}

/// Port of `FUN_00669FA0`'s core position-to-status stamping loop
/// for the English pyramid.
///
/// # What the exe does
///
/// Walks the club array (`comp+0xB1`, stride `0x3B`, count
/// `comp+0x3E`), and for each row where `current_status` is NOT in
/// [`STICKY_STATUSES`], overwrites `Club+0x37` based on final rank
/// plus per-league flag bits. Positions that are already sticky
/// (a preserved `3` from a prior stamp, a `5` from a same-year
/// playoff resolver, `0xFE` from the stadium gate, `0xFC` from a
/// deep reprieve) are left alone.
///
/// Returns one [`LeagueTableRow`] per input, in the same order,
/// with `current_status` updated where a non-sticky write fired.
/// The caller applies the writes.
// GDI-REG: 00669fa0 PORTED_PARTIAL
pub fn stamp_league_end_of_season_statuses(
    shape: EnglishLeagueEndShape,
    finalized_table: &[LeagueTableRow],
) -> Vec<LeagueTableRow> {
    let n_auto  = shape.n_auto_promote();
    let n_pl    = shape.n_playoff();
    let n_releg = shape.n_auto_relegate();
    let n = finalized_table.len();
    let releg_start = n.saturating_sub(n_releg);

    finalized_table
        .iter()
        .enumerate()
        .map(|(pos, row)| {
            let mut r = *row;
            if is_sticky_status(r.current_status) {
                // FUN_00669FA0's sticky-set guard — leave alone.
                return r;
            }
            let new_status = if pos < n_auto {
                STATUS_AUTO_PROMOTION
            } else if pos < n_auto + n_pl {
                // Playoff bracket positions — losers get 2; the
                // eventual winner is stamped 5 via
                // `propagate_english_playoff_winner` AFTER the
                // subcomp resolves.
                STATUS_PLAYOFF_LOSER
            } else if pos >= releg_start {
                STATUS_RELEGATED
            } else {
                // Mid-table — no year-end status change.
                r.current_status
            };
            r.current_status = new_status;
            r
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Playoff-winner propagation — `+0x37 = 5` on the parent-league row
// ---------------------------------------------------------------------------

/// Result of applying a playoff outcome to the parent league's table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayoffWinnerPropagation {
    /// The parent league's table AFTER writing `+0x37 = 5` on the
    /// winner. Same order as input.
    pub updated_table: Vec<LeagueTableRow>,
    /// True iff a write actually fired (i.e. the winner existed in
    /// the parent table AND its current status was not sticky).
    pub write_fired: bool,
    /// If the write fired: the winner's post-write position in the
    /// parent table (0-indexed). Diagnostic.
    pub winner_position: Option<usize>,
}

/// Semantic port of the transition:
///
/// > When an English promotion-playoff (D1 / D2 / D3) resolves, its
/// > winner receives `Club+0x37 = 5` in the parent league's table.
///
/// The archaeology shows the actual write happens inside
/// `FUN_00669FA0`'s translator when it re-scans the parent table
/// AFTER the subcomp aggregate lands. That translator applies the
/// [`STICKY_STATUSES`] guard, so a winner already carrying a
/// sticky status is not overwritten — verified against the
/// Brazilian regional resolvers' analogous write pattern.
///
/// This is Traditional-mode only. V4 has its own playoff shape and
/// is not routed through here.
pub fn propagate_english_playoff_winner(
    parent_table: &[LeagueTableRow],
    winner_club_id: u32,
) -> PlayoffWinnerPropagation {
    let mut updated = parent_table.to_vec();
    let mut write_fired = false;
    let mut winner_position = None;
    for (i, row) in updated.iter_mut().enumerate() {
        if row.club_id != winner_club_id {
            continue;
        }
        winner_position = Some(i);
        if is_sticky_status(row.current_status) {
            // Point 4 of the C12 gate: sticky statuses are
            // preserved. A club already carrying `3` (relegated),
            // `5` (double-win?), `0xFE` (reprieve), or `0xFC` is
            // NOT overwritten. This ordering (subcomp-final →
            // parent-table stamp) is why C8 can rely on `5`
            // remaining eligible: it was stamped BEFORE the swap.
            return PlayoffWinnerPropagation {
                updated_table: updated,
                write_fired: false,
                winner_position,
            };
        }
        row.current_status = STATUS_PLAYOFF_WINNER;
        write_fired = true;
        break;
    }
    PlayoffWinnerPropagation {
        updated_table: updated,
        write_fired,
        winner_position,
    }
}

// ---------------------------------------------------------------------------
// FUN_0066EED0 post-swap transform — every moved club → 0xFF
// ---------------------------------------------------------------------------

/// Deterministic mapping between the pre-swap `+0x37` of a club
/// selected for the P/R swap and the post-swap value.
///
/// This is `FUN_0066EED0`'s write behaviour distilled to a pure
/// function. Callers that want to simulate the post-swap state
/// without threading a full apply-layer can fold this over their
/// roster.
///
/// | Pre-swap | Meaning | Moved? | Post-swap |
/// |---|---|---|---|
/// | `0`  | auto-promotion | ✓ up   | `0xFF` |
/// | `5`  | playoff winner | ✓ up   | `0xFF` |
/// | `3`  | relegated      | ✓ down | `0xFF` |
/// | `0xFE` | stadium reprieve | no | unchanged (sticky) |
/// | `0xFF` | idle           | no | unchanged |
/// | anything else | mid-table stayer | no | unchanged |
///
/// Confidence: STATE-EXACT vs the DD decompile of `FUN_0066EED0`.
pub fn apply_swap_status_transform(pre_swap_status: u8) -> u8 {
    match pre_swap_status {
        STATUS_AUTO_PROMOTION
        | STATUS_PLAYOFF_WINNER
        | STATUS_RELEGATED => STATUS_IDLE,
        _ => pre_swap_status,
    }
}

// ---------------------------------------------------------------------------
// Reset semantics — post-year-end cleanup
// ---------------------------------------------------------------------------

/// Outcome of the annual reset pass on a `Club+0x37` value.
///
/// No separate reset function exists in the decompile. This enum
/// records the OBSERVED disposition of each status alphabet member
/// as the year-end pipeline flows from `FUN_00669FA0` stamping →
/// `FUN_0066EED0` swap → next season's `FUN_00669FA0` stamping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetOutcome {
    /// Value ends the year as `0xFF` because the club MOVED (0/5/3
    /// entered the swap and came out as idle).
    ClearedByMove,
    /// Value persists across the year boundary. Next
    /// `FUN_00669FA0` pass will re-stamp any non-sticky value
    /// based on the new season's final table.
    PersistsAcrossYear,
    /// Value persists across the year boundary AND blocks
    /// re-stamping (sticky under `FUN_00669FA0`'s guard).
    PersistsAndBlocksReStamp,
}

/// Report the disposition of a status byte at year rollover — the
/// answer to the point-7 question "what does each recovered status
/// become after reset?".
///
/// * `0` → `ClearedByMove` (auto-promoted club is moved, swap
///   writes `0xFF`).
/// * `2` → `PersistsAcrossYear`; next stamp overwrites it (`2` is
///   NOT sticky).
/// * `3` → `ClearedByMove` (relegated club is moved, swap writes
///   `0xFF`).
/// * `5` → `ClearedByMove` (playoff winner promoted, same swap).
/// * `6`/`7` → `PersistsAcrossYear`; non-sticky, next stamp
///   overwrites.
/// * `0xFC` → `PersistsAndBlocksReStamp` — deep reprieve.
/// * `0xFE` → `PersistsAndBlocksReStamp` — stadium reprieve. The
///   critical gotcha: `0xFE` is NEVER cleared automatically. If
///   an external gate does not rewrite it before the next
///   year-end, the affected club stays reprieved indefinitely.
/// * `0xFF` → `PersistsAcrossYear` (nominal idle; next stamp
///   overwrites).
pub fn reset_outcome_for(status: u8) -> ResetOutcome {
    match status {
        STATUS_AUTO_PROMOTION | STATUS_RELEGATED | STATUS_PLAYOFF_WINNER
            => ResetOutcome::ClearedByMove,
        STATUS_DEEP_REPRIEVE | STATUS_STADIUM_REPRIEVE
            => ResetOutcome::PersistsAndBlocksReStamp,
        _ => ResetOutcome::PersistsAcrossYear,
    }
}

/// True iff the stadium-fail reprieve (`0xFE`) survives into the
/// next season without automatic clearing. Point-8 answer. Kept as
/// a separate predicate because production paths may want to log
/// or diagnose that the reprieve is persisting.
pub fn stadium_fail_reprieve_persists() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: u32, status: u8) -> LeagueTableRow {
        LeagueTableRow { club_id: id, current_status: status }
    }

    fn all_idle(n: usize) -> Vec<LeagueTableRow> {
        (0..n).map(|i| row(i as u32 + 1, STATUS_IDLE)).collect()
    }

    // -------- Sticky-set --------
    #[test]
    fn sticky_set_covers_expected_alphabet() {
        for &s in &[STATUS_RELEGATED, STATUS_PLAYOFF_WINNER,
                    STATUS_STADIUM_REPRIEVE, STATUS_DEEP_REPRIEVE] {
            assert!(is_sticky_status(s), "expected sticky: {s:#x}");
        }
        for &s in &[STATUS_AUTO_PROMOTION, STATUS_PLAYOFF_LOSER,
                    STATUS_POSITION_FLAG_6, STATUS_POSITION_FLAG_7,
                    STATUS_IDLE] {
            assert!(!is_sticky_status(s), "expected non-sticky: {s:#x}");
        }
    }

    // -------- Point 11: final-table status tests per league --------
    #[test]
    fn premier_stamps_zero_promo_three_relegated_no_playoff() {
        // Prem: 20 clubs, 0 auto-promo, 0 playoff, 3 auto-releg.
        let out = stamp_league_end_of_season_statuses(
            EnglishLeagueEndShape::Premier, &all_idle(20));
        // Positions 1..=17 unchanged (still IDLE) — mid-table.
        for i in 0..17 { assert_eq!(out[i].current_status, STATUS_IDLE); }
        for i in 17..20 { assert_eq!(out[i].current_status, STATUS_RELEGATED); }
    }

    #[test]
    fn first_division_stamps_2_auto_4_playoff_3_relegated() {
        // First: 24 clubs, 2 auto, 4 playoff (pos 3..=6), 3 releg.
        let out = stamp_league_end_of_season_statuses(
            EnglishLeagueEndShape::First, &all_idle(24));
        assert_eq!(out[0].current_status, STATUS_AUTO_PROMOTION);
        assert_eq!(out[1].current_status, STATUS_AUTO_PROMOTION);
        for i in 2..6 {
            assert_eq!(out[i].current_status, STATUS_PLAYOFF_LOSER,
                "First pos {} = playoff bracket", i+1);
        }
        for i in 6..21 { assert_eq!(out[i].current_status, STATUS_IDLE); }
        for i in 21..24 {
            assert_eq!(out[i].current_status, STATUS_RELEGATED);
        }
    }

    #[test]
    fn second_division_stamps_2_auto_4_playoff_4_relegated() {
        let out = stamp_league_end_of_season_statuses(
            EnglishLeagueEndShape::Second, &all_idle(24));
        assert_eq!(out[0].current_status, STATUS_AUTO_PROMOTION);
        assert_eq!(out[1].current_status, STATUS_AUTO_PROMOTION);
        for i in 2..6 { assert_eq!(out[i].current_status, STATUS_PLAYOFF_LOSER); }
        for i in 20..24 { assert_eq!(out[i].current_status, STATUS_RELEGATED); }
    }

    #[test]
    fn third_division_stamps_3_auto_4_playoff_1_relegated() {
        let out = stamp_league_end_of_season_statuses(
            EnglishLeagueEndShape::Third, &all_idle(24));
        for i in 0..3 { assert_eq!(out[i].current_status, STATUS_AUTO_PROMOTION); }
        for i in 3..7 { assert_eq!(out[i].current_status, STATUS_PLAYOFF_LOSER); }
        for i in 7..23 { assert_eq!(out[i].current_status, STATUS_IDLE); }
        assert_eq!(out[23].current_status, STATUS_RELEGATED);
    }

    #[test]
    fn conference_stamps_1_auto_no_playoff_3_relegated() {
        let out = stamp_league_end_of_season_statuses(
            EnglishLeagueEndShape::Conference, &all_idle(22));
        assert_eq!(out[0].current_status, STATUS_AUTO_PROMOTION);
        for i in 1..19 { assert_eq!(out[i].current_status, STATUS_IDLE); }
        for i in 19..22 { assert_eq!(out[i].current_status, STATUS_RELEGATED); }
    }

    #[test]
    fn sticky_status_survives_restamp() {
        // C12 point 4: an already-set 5 must not be clobbered by
        // subsequent stamping. Also verify 0xFE reprieve at
        // last-place Prem does NOT become STATUS_RELEGATED.
        let mut table = all_idle(20);
        table[19].current_status = STATUS_STADIUM_REPRIEVE;
        table[17].current_status = STATUS_PLAYOFF_WINNER;
        table[5].current_status = STATUS_DEEP_REPRIEVE;
        let out = stamp_league_end_of_season_statuses(
            EnglishLeagueEndShape::Premier, &table);
        assert_eq!(out[19].current_status, STATUS_STADIUM_REPRIEVE);
        assert_eq!(out[17].current_status, STATUS_PLAYOFF_WINNER);
        assert_eq!(out[5].current_status, STATUS_DEEP_REPRIEVE);
        assert_eq!(out[18].current_status, STATUS_RELEGATED); // 19th pos still relegated
    }

    // -------- Point 12: playoff winner → C8 --------
    #[test]
    fn playoff_winner_writes_status_5_when_non_sticky() {
        let table = vec![
            row(1, STATUS_AUTO_PROMOTION),
            row(2, STATUS_AUTO_PROMOTION),
            row(3, STATUS_PLAYOFF_LOSER),
            row(4, STATUS_PLAYOFF_LOSER),
            row(5, STATUS_PLAYOFF_LOSER),  // winner will be 5
            row(6, STATUS_PLAYOFF_LOSER),
        ];
        let out = propagate_english_playoff_winner(&table, 5);
        assert!(out.write_fired);
        assert_eq!(out.winner_position, Some(4));
        assert_eq!(out.updated_table[4].current_status, STATUS_PLAYOFF_WINNER);
        // Other rows untouched.
        assert_eq!(out.updated_table[0].current_status, STATUS_AUTO_PROMOTION);
        assert_eq!(out.updated_table[3].current_status, STATUS_PLAYOFF_LOSER);
    }

    #[test]
    fn playoff_winner_does_not_overwrite_sticky() {
        // If somehow the winner already has status 3 (should be
        // impossible in a real English playoff — a relegated club
        // can't win a promotion playoff — but the sticky guard is
        // the point). The write must not fire.
        let table = vec![
            row(1, STATUS_PLAYOFF_LOSER),
            row(2, STATUS_RELEGATED),
        ];
        let out = propagate_english_playoff_winner(&table, 2);
        assert!(!out.write_fired);
        assert_eq!(out.updated_table[1].current_status, STATUS_RELEGATED);
    }

    #[test]
    fn playoff_winner_missing_from_table_is_noop() {
        let table = vec![row(1, STATUS_PLAYOFF_LOSER)];
        let out = propagate_english_playoff_winner(&table, 999);
        assert!(!out.write_fired);
        assert_eq!(out.winner_position, None);
        assert_eq!(out.updated_table, table);
    }

    // -------- apply_swap_status_transform --------
    #[test]
    fn swap_transform_covers_documented_alphabet() {
        assert_eq!(apply_swap_status_transform(STATUS_AUTO_PROMOTION), STATUS_IDLE);
        assert_eq!(apply_swap_status_transform(STATUS_PLAYOFF_WINNER), STATUS_IDLE);
        assert_eq!(apply_swap_status_transform(STATUS_RELEGATED), STATUS_IDLE);
        assert_eq!(apply_swap_status_transform(STATUS_STADIUM_REPRIEVE),
                   STATUS_STADIUM_REPRIEVE);
        assert_eq!(apply_swap_status_transform(STATUS_IDLE), STATUS_IDLE);
        assert_eq!(apply_swap_status_transform(STATUS_PLAYOFF_LOSER),
                   STATUS_PLAYOFF_LOSER);
        assert_eq!(apply_swap_status_transform(STATUS_DEEP_REPRIEVE),
                   STATUS_DEEP_REPRIEVE);
        assert_eq!(apply_swap_status_transform(STATUS_POSITION_FLAG_6),
                   STATUS_POSITION_FLAG_6);
        assert_eq!(apply_swap_status_transform(STATUS_POSITION_FLAG_7),
                   STATUS_POSITION_FLAG_7);
    }

    // -------- Point 13: reset mapping --------
    #[test]
    fn reset_outcome_full_alphabet() {
        assert_eq!(reset_outcome_for(STATUS_AUTO_PROMOTION), ResetOutcome::ClearedByMove);
        assert_eq!(reset_outcome_for(STATUS_RELEGATED),      ResetOutcome::ClearedByMove);
        assert_eq!(reset_outcome_for(STATUS_PLAYOFF_WINNER), ResetOutcome::ClearedByMove);
        assert_eq!(reset_outcome_for(STATUS_PLAYOFF_LOSER),  ResetOutcome::PersistsAcrossYear);
        assert_eq!(reset_outcome_for(STATUS_POSITION_FLAG_6),ResetOutcome::PersistsAcrossYear);
        assert_eq!(reset_outcome_for(STATUS_POSITION_FLAG_7),ResetOutcome::PersistsAcrossYear);
        assert_eq!(reset_outcome_for(STATUS_IDLE),           ResetOutcome::PersistsAcrossYear);
        assert_eq!(reset_outcome_for(STATUS_STADIUM_REPRIEVE),
                   ResetOutcome::PersistsAndBlocksReStamp);
        assert_eq!(reset_outcome_for(STATUS_DEEP_REPRIEVE),
                   ResetOutcome::PersistsAndBlocksReStamp);
    }

    #[test]
    fn stadium_fail_reprieve_is_not_cleared_by_reset() {
        // Point 8: prove 0xFE survives the year-boundary in the
        // Rust model, matching the decompile.
        assert!(stadium_fail_reprieve_persists());
        assert_eq!(reset_outcome_for(STATUS_STADIUM_REPRIEVE),
                   ResetOutcome::PersistsAndBlocksReStamp);
    }

    // -------- End-to-end: stamp → playoff → swap-transform --------
    #[test]
    fn end_to_end_first_division_playoff_flow_to_swap() {
        // Simulate First's finalized table (24 clubs), stamp,
        // resolve playoff (club 5 wins), then apply swap transform
        // to verify auto-promoted (1,2) + playoff-winner (5) all
        // end at STATUS_IDLE and relegated (22,23,24) end at IDLE.
        let table = all_idle(24);
        let stamped = stamp_league_end_of_season_statuses(
            EnglishLeagueEndShape::First, &table);
        // Playoff winner = club 5 (position 5 in the input =
        // playoff bracket).
        let after_playoff = propagate_english_playoff_winner(&stamped, 5);
        assert!(after_playoff.write_fired);
        // Post-swap transform: apply per club.
        let final_states: Vec<u8> = after_playoff.updated_table
            .iter()
            .map(|r| apply_swap_status_transform(r.current_status))
            .collect();
        // Moved clubs (auto-promoted + playoff winner + relegated)
        // are now IDLE.
        assert_eq!(final_states[0], STATUS_IDLE);  // auto-promoted 1
        assert_eq!(final_states[1], STATUS_IDLE);  // auto-promoted 2
        assert_eq!(final_states[4], STATUS_IDLE);  // playoff winner (was 5)
        assert_eq!(final_states[21], STATUS_IDLE); // relegated
        assert_eq!(final_states[22], STATUS_IDLE);
        assert_eq!(final_states[23], STATUS_IDLE);
        // Playoff losers (positions 3, 4, 6) still carry
        // STATUS_PLAYOFF_LOSER — they didn't move.
        assert_eq!(final_states[2], STATUS_PLAYOFF_LOSER);
        assert_eq!(final_states[3], STATUS_PLAYOFF_LOSER);
        assert_eq!(final_states[5], STATUS_PLAYOFF_LOSER);
    }

    #[test]
    fn end_to_end_conference_champion_flow_stadium_fail_preserves_status() {
        // Conference: 22 clubs, champion has stadium reprieve
        // written by the pyramid orchestrator. Verify that reprieve
        // is preserved through the year-end pipeline.
        let mut table = all_idle(22);
        table[0].current_status = STATUS_STADIUM_REPRIEVE; // champion reprieved
        let stamped = stamp_league_end_of_season_statuses(
            EnglishLeagueEndShape::Conference, &table);
        // Champion NOT overwritten with STATUS_AUTO_PROMOTION.
        assert_eq!(stamped[0].current_status, STATUS_STADIUM_REPRIEVE);
        // Bottom 3 got STATUS_RELEGATED as usual.
        for i in 19..22 { assert_eq!(stamped[i].current_status, STATUS_RELEGATED); }
        // Swap transform preserves reprieve.
        assert_eq!(
            apply_swap_status_transform(stamped[0].current_status),
            STATUS_STADIUM_REPRIEVE,
        );
    }
}
