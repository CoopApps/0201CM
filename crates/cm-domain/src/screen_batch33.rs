//! Batch 33: completing the `comp.cpp` read/format pair — plus the
//! four history-array helpers the previous batch marked as skipped.
//!
//! - `FUN_004aa9a0` → `format_team_attribute`: the team-history
//!   counterpart to [`crate::screen_batch32::format_player_attribute`]
//!   — I claimed batch 32 covered it in a doc header, but the body
//!   was missing. This lands it for real.
//! - `FUN_004ab310` → `player_history_bsearch`: bsearch over the
//!   team-history's per-player array at stride `0x76`.
//! - `FUN_004ab4f0` → `team_history_bsearch`: bsearch over the
//!   comp-history's per-team array at stride `0x2d`.
//! - `FUN_004ab5e0` → `player_history_filter_copy`: dense-copy of
//!   history records, dropping rows whose queried attribute reads as
//!   the sentinel (empty).
//! - `FUN_004ab880` → `team_history_filter_copy`: dense-copy variant
//!   with an optional flag-mask filter that consults each source
//!   record's per-team flag word.

use serde::{Deserialize, Serialize};
use crate::screen_batch29::TeamAttribute;
use crate::screen_batch32::{PlayerHistoryRecord, PlayerHistoryAttr,
                            format_player_attribute};

// =====================================================================
// FUN_004aa9a0 — format_team_attribute
// =====================================================================

/// The team-history record fields consumed by
/// [`format_team_attribute`]. Kept structural (byte-per-field) so the
/// port stays pure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamHistoryRecord {
    // 14 bytes at +0x0c..=+0x19 (per crate::screen_batch29 conventions)
    pub bytes_c_to_19: [u8; 14],
    // shorts at +0x1a, +0x1d, +0x1f, +0x21, +0x23, +0x25, +0x27
    pub s_1a: i16, pub s_1d: i16, pub s_1f: i16,
    pub s_21: i16, pub s_23: i16, pub s_25: i16, pub s_27: i16,
    /// `+0x29` — 32-bit signed, used as the comparison threshold on
    /// reputation-gated ratios.
    pub int_29: i32,
    /// Second byte at +0x0d (used by Games case for pair formatting).
    pub byte_d: u8,
    /// Second byte at +0x11 (used by HomeForm case for pair formatting).
    pub byte_11: u8,
}

/// Direct port of `FUN_004aa9a0` — team-history attribute formatter.
///
/// The switch selects one of the 17 attributes and applies the same
/// reputation-gated ratio formulas as
/// [`crate::screen_batch29::read_team_attribute_value`], returning a
/// pre-formatted display string. Reputation-gated ratios that don't
/// pass their guards return `None` (matches the exe's behaviour of
/// leaving the pre-loaded header string, e.g. `"Ratio"`, in place).
///
/// - `comp_reputation_ok`: matches
///   [`crate::screen_batch29::read_team_attribute_value`]'s parameter.
/// - `comp_attr1_value`: the reputation-lookup float (returned by
///   `FUN_004a9980(comp, 1)`).
/// - `k1`, `scale_960`: shared constants.
/// - `include_secondary`: `param_5 != 0` — allows "goals (conceded)"
///   pair for Games case and "value (games)" pair for Penalties.
pub fn format_team_attribute(
    rec: &TeamHistoryRecord,
    attr: TeamAttribute,
    include_secondary: bool,
    comp_reputation_ok: bool,
    comp_attr1_value: f64,
    k1: f64,
    _scale_960: f64,
) -> Option<String> {
    use TeamAttribute::*;
    let byte = |idx: usize| rec.bytes_c_to_19[idx];
    let games = byte(0x00);   // +0x0c

    match attr {
        Games => {
            if rec.byte_d != 0 && include_secondary {
                Some(format!("{} ({})", games as i32 - rec.byte_d as i32, rec.byte_d))
            } else {
                Some(format!("{games}"))
            }
        }
        AverageAttendance => Some(format!("{}", byte(0x02))),   // +0x0e
        Ratio             => Some(format!("{}", byte(0x03))),   // +0x0f
        HomeForm => {
            let v = byte(0x05) as i8;   // +0x11
            if v != 0 && include_secondary {
                Some(format!("{v} ({})", rec.byte_11))
            } else {
                Some(format!("{v}"))
            }
        }
        AwayForm         => Some(format!("{}", byte(0x06))),
        GamesWonInRow    => Some(format!("{}", byte(0x07))),
        GamesLostInRow   => Some(format!("{}", byte(0x08))),
        GamesWithoutLoss => Some(format!("{}", byte(0x09))),
        GamesWithoutWin  => Some(format!("{}", byte(0x0A))),
        Goals            => Some(format!("{}", byte(0x0B))),
        Conceded         => Some(format!("{}", byte(0x0C))),
        Penalties        => Some(format!("{}", byte(0x0D))),   // +0x19

        // Reputation-gated ratios — return None when gate fails.
        GoalsFromCorners => {
            if !comp_reputation_ok { return None; }
            if !(comp_attr1_value * k1 <= rec.int_29 as f64) { return None; }
            if rec.s_1d < 0x15 { return None; }
            let sVar3 = rec.s_1d;
            if sVar3 < 0x1f5
                && sVar3 as i32 <= (rec.int_29 + ((rec.int_29 >> 31) & 3)) >> 2
            {
                return None;
            }
            // Exe formats via `__ftol()` then "%.3d" — but the actual
            // value stored comes from
            // `(short at +0x1f) / (short at +0x1d) * scale_960` then
            // truncated. Represent the truncation with `as i64`.
            let v = (rec.s_1f as f64 / rec.s_1d as f64) * _scale_960;
            Some(format!("{}", v as i64))
        }
        GoalsFromIfks => {
            if !comp_reputation_ok { return None; }
            if !(comp_attr1_value * k1 <= rec.int_29 as f64) { return None; }
            if rec.int_29 < 0x96 { return None; }
            if games == 0 { return None; }
            let v = rec.s_21 as f64 / games as f64;
            Some(format!("{v:.1}"))
        }
        GoalsFromDfks => {
            if !comp_reputation_ok { return None; }
            if !(comp_attr1_value * k1 <= rec.int_29 as f64) { return None; }
            if rec.int_29 < 0x96 { return None; }
            if games == 0 { return None; }
            let v = rec.s_23 as f64 / games as f64;
            Some(format!("{v:.1}"))
        }
        YellowCards => {
            if !comp_reputation_ok { return None; }
            if !(comp_attr1_value * k1 <= rec.int_29 as f64) { return None; }
            let sVar3 = rec.s_25;
            if sVar3 < 0x19 {
                if sVar3 < 5 || (sVar3 != 0 && rec.int_29 / sVar3 as i32 > 0x2d) {
                    if rec.int_29 < 0x21d { return None; }
                    if sVar3 < 0xb { return None; }
                }
            }
            let v = (rec.s_27 as f64 / sVar3 as f64) * _scale_960;
            Some(format!("{}", v as i64))
        }
        RedCards => {
            // Case 0x11: cross-reference against attr(1) (Games).
            if !comp_reputation_ok { return None; }
            // The exe compares `attr(1) * K2` (where K2 = _DAT_00956968)
            // to team `attr(1)`. Uses the caller-supplied comp_attr1_value.
            if !(comp_attr1_value * k1 <= rec.int_29 as f64) { return None; }
            if games == 0 { return None; }
            let v = rec.s_1a as f64 / games as f64;
            Some(format!("{v:.2}"))
        }
        GamesRatio => {
            // Note: FUN_004aa9a0 doesn't include a Games-ratio case
            // (it's case 0x12 in the numeric reader but NOT in this
            // formatter). Return None to signal.
            None
        }
    }
}

// =====================================================================
// FUN_004ab310 / FUN_004ab4f0 — bsearch helpers
// =====================================================================

/// Direct port of the semantics of `FUN_004ab310(team_history, key_ref)`.
///
/// The exe's implementation is a `bsearch` call over the player-array
/// at `team_history + 0xc` with stride `0x76` and count `+0x14`. This
/// port takes the array as a `&[T]` and the comparison predicate
/// separately.
///
/// Returns `Some(index)` for a match, `None` for miss. The exe returns
/// a pointer; the caller does `array + index * stride` externally.
pub fn player_history_bsearch<T>(
    array: &[T],
    key: &T,
    cmp: impl Fn(&T, &T) -> std::cmp::Ordering,
) -> Option<usize> {
    array.binary_search_by(|elem| cmp(elem, key)).ok()
}

/// Same shape for the team-history array at stride `0x2d`.
pub fn team_history_bsearch<T>(
    array: &[T],
    key: &T,
    cmp: impl Fn(&T, &T) -> std::cmp::Ordering,
) -> Option<usize> {
    array.binary_search_by(|elem| cmp(elem, key)).ok()
}

// =====================================================================
// FUN_004ab5e0 — player_history_filter_copy
// =====================================================================

/// The exe's `FUN_004ab5e0(team_history, attr, &out_count, requested)`.
///
/// Copies the team-history's player array into a fresh dense buffer,
/// dropping any trailing rows whose queried attribute reads as the
/// `_DAT_00956970` sentinel. Special case: attr values 3/4/5 (form
/// windows) skip the "drop empty" pass — those attrs always render.
///
/// - `records`: source array in current sort order.
/// - `attr`: the attribute the caller will read (drives the empty
///   check).
/// - `requested`: caller's max-count hint (< 0 → use `records.len()`).
///
/// Returns the filtered slice. Length is guaranteed ≤ min(records.len(),
/// requested).
pub fn player_history_filter_copy(
    records: &[PlayerHistoryRecord],
    attr: PlayerHistoryAttr,
    requested: i32,
) -> Vec<PlayerHistoryRecord> {
    if records.is_empty() { return Vec::new(); }
    let mut count = if requested < 0 {
        records.len()
    } else {
        (requested as usize).min(records.len())
    };

    // Skip the empty-drop pass for form-window attrs (cases 3/4/5).
    let skip_empty_drop = matches!(attr,
        PlayerHistoryAttr::HomeForm
        | PlayerHistoryAttr::AwayForm
        | PlayerHistoryAttr::AllForm,
    );

    if !skip_empty_drop {
        while count > 0 {
            let last = &records[count - 1];
            // Empty iff the formatter returns None for this attr.
            if format_player_attribute(last, attr, false).is_none() {
                count -= 1;
            } else {
                break;
            }
        }
    }

    records[..count].to_vec()
}

// =====================================================================
// FUN_004ab880 — team_history_filter_copy with flag mask
// =====================================================================

/// One team-history record as consumed by
/// [`team_history_filter_copy`]. Uses a callback to check the
/// per-team flag word (the exe calls `FUN_004b4fb0(record + 0x40)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamHistoryEntry {
    pub record: TeamHistoryRecord,
    /// The 16-bit flag word `FUN_004b4fb0` returns for this record.
    pub flag_word: u16,
    /// The attribute value cache: `Some(val)` = queryable, `None` =
    /// sentinel. Mirrors `FUN_004a9d10` output.
    pub attr_is_sentinel: bool,
}

/// Direct port of `FUN_004ab880(comp_history, attr, &out_count, requested, mask)`.
///
/// Dense-copies with two filters:
/// 1. If `mask != 0xFFFF` AND `(entry.flag_word & mask) == 0`, skip.
/// 2. If `entry.attr_is_sentinel`, skip.
///
/// Also stops when `requested` records have been emitted.
pub fn team_history_filter_copy(
    entries: &[TeamHistoryEntry],
    requested: i32,
    mask: u16,
) -> Vec<TeamHistoryEntry> {
    let mut out = Vec::new();
    let request_cap = if requested < 0 { i32::MAX } else { requested };
    for e in entries {
        if (out.len() as i32) >= request_cap { break; }
        if mask != 0xFFFF && (e.flag_word & mask) == 0 { continue; }
        if e.attr_is_sentinel { continue; }
        out.push(*e);
    }
    out
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn team_rec() -> TeamHistoryRecord {
        let mut r = TeamHistoryRecord::default();
        r.bytes_c_to_19 = [10, 3, 15, 20, 0, 5, 4, 3, 2, 1, 7, 6, 5, 4];
        r.s_1a = 8; r.s_1d = 100; r.s_1f = 12;
        r.s_21 = 20; r.s_23 = 15; r.s_25 = 25; r.s_27 = 8;
        r.int_29 = 300;
        r.byte_d = 3;
        r.byte_11 = 2;
        r
    }

    // ---- format_team_attribute ----

    #[test]
    fn team_fmt_games_pair_when_secondary() {
        let r = team_rec();   // games=10, byte_d=3
        assert_eq!(format_team_attribute(&r, TeamAttribute::Games, true, false, 0.0, 2.0, 100.0),
                   Some("7 (3)".into()));
        assert_eq!(format_team_attribute(&r, TeamAttribute::Games, false, false, 0.0, 2.0, 100.0),
                   Some("10".into()));
    }

    #[test]
    fn team_fmt_home_form_pair_when_secondary() {
        let r = team_rec();   // byte(0x05) = 5 (positive), byte_11 = 2
        assert_eq!(format_team_attribute(&r, TeamAttribute::HomeForm, true, false, 0.0, 2.0, 100.0),
                   Some("5 (2)".into()));
    }

    #[test]
    fn team_fmt_byte_cases() {
        let r = team_rec();
        assert_eq!(format_team_attribute(&r, TeamAttribute::AverageAttendance, false, false, 0.0, 2.0, 100.0),
                   Some("15".into()));
        assert_eq!(format_team_attribute(&r, TeamAttribute::Penalties, false, false, 0.0, 2.0, 100.0),
                   Some("4".into()));
    }

    #[test]
    fn team_fmt_ratio_returns_none_when_gate_fails() {
        // Fail via comp_reputation_ok=false — universal blocker.
        let r = team_rec();
        assert!(format_team_attribute(&r, TeamAttribute::GoalsFromCorners, false, false, 0.0, 2.0, 100.0).is_none());
        assert!(format_team_attribute(&r, TeamAttribute::GoalsFromIfks, false, false, 0.5, 2.0, 100.0).is_none());
        // Fail via int_29 < 0x96 threshold. Override to trip that path.
        let mut r_low = team_rec();
        r_low.int_29 = 100;   // < 0x96 (150) → IFKs/DFKs gate fails
        assert!(format_team_attribute(&r_low, TeamAttribute::GoalsFromIfks, false, true, 0.5, 2.0, 100.0).is_none());
        assert!(format_team_attribute(&r_low, TeamAttribute::GoalsFromDfks, false, true, 0.5, 2.0, 100.0).is_none());
    }

    #[test]
    fn team_fmt_goals_from_corners_passes_deep_gate() {
        // Need: comp_val * k1 (0.5 * 2 = 1.0) ≤ int_29 (300); s_1d > 0x14
        // AND (s_1d > 500 OR (300+0)>>2 = 75 < s_1d).
        // s_1d = 100 > 75 → pass. Result = (12/100) * 100 = 12
        let r = team_rec();
        let out = format_team_attribute(&r, TeamAttribute::GoalsFromCorners, false, true, 0.5, 2.0, 100.0);
        assert!(out.is_some());
    }

    // ---- bsearch helpers ----

    #[test]
    fn bsearch_finds_and_misses() {
        let arr = [1, 3, 5, 7, 9];
        assert_eq!(player_history_bsearch(&arr, &5, |a, b| a.cmp(b)), Some(2));
        assert_eq!(player_history_bsearch(&arr, &4, |a, b| a.cmp(b)), None);
        assert_eq!(team_history_bsearch(&arr, &1, |a, b| a.cmp(b)), Some(0));
    }

    // ---- player_history_filter_copy ----

    #[test]
    fn player_filter_empty_input_empty_output() {
        assert!(player_history_filter_copy(&[], PlayerHistoryAttr::Goals, -1).is_empty());
    }

    #[test]
    fn player_filter_drops_trailing_empty_rows_for_non_form_attrs() {
        // Rows with games_won_byte != 0 → GoalsPerGame formats to Some.
        // Rows with games_won_byte == 0 → returns None (sentinel).
        let mut a = PlayerHistoryRecord::default();
        a.games_won_byte = 3;
        a.goals_int = 12;
        // Two "kept" rows + one trailing "sentinel" row.
        let recs = vec![a.clone(), a.clone(), PlayerHistoryRecord::default()];
        let out = player_history_filter_copy(&recs, PlayerHistoryAttr::GoalsPerGame, -1);
        // Trailing None row dropped; first two kept.
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn player_filter_form_attrs_skip_empty_drop() {
        // HomeForm always renders (even if empty window sums to 0), so
        // the exe's case-3/4/5 branch SKIPS the drop pass.
        let recs = vec![PlayerHistoryRecord::default(); 3];
        let out = player_history_filter_copy(&recs, PlayerHistoryAttr::HomeForm, -1);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn player_filter_respects_requested_cap() {
        let mut a = PlayerHistoryRecord::default();
        a.bytes_69_to_73[0] = 5;   // Goals = 5
        let recs = vec![a; 10];
        let out = player_history_filter_copy(&recs, PlayerHistoryAttr::Goals, 4);
        assert_eq!(out.len(), 4);
    }

    // ---- team_history_filter_copy ----

    fn te(flags: u16, sentinel: bool) -> TeamHistoryEntry {
        TeamHistoryEntry { record: TeamHistoryRecord::default(), flag_word: flags, attr_is_sentinel: sentinel }
    }

    #[test]
    fn team_filter_mask_ffff_disables_mask_filter() {
        let entries = vec![te(0x0000, false), te(0x0001, false)];
        assert_eq!(team_history_filter_copy(&entries, -1, 0xFFFF).len(), 2);
    }

    #[test]
    fn team_filter_mask_drops_no_match() {
        let entries = vec![te(0x0001, false), te(0x0002, false), te(0x0003, false)];
        // mask = 0x0002 keeps entries 1 and 2 (0x0002 & mask != 0, 0x0003 & mask != 0)
        let out = team_history_filter_copy(&entries, -1, 0x0002);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn team_filter_drops_sentinel_rows() {
        let entries = vec![te(0xFFFF, false), te(0xFFFF, true), te(0xFFFF, false)];
        let out = team_history_filter_copy(&entries, -1, 0xFFFF);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn team_filter_stops_at_requested() {
        let entries = vec![te(0xFFFF, false); 20];
        let out = team_history_filter_copy(&entries, 5, 0xFFFF);
        assert_eq!(out.len(), 5);
    }
}
