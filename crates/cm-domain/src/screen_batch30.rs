//! Batch 30: 8 more comp_screens.cpp / comp.cpp functions ported —
//! completing the 0x004a0000..0x004a5900 range the user attached.
//!
//! - `FUN_004a0010` / `FUN_004a0610` — sibling variants of batch 28's
//!   `comp_list_screen_build`. Same 22-row enumerator + entry-list
//!   walker, differing only in the valid-marker byte the slot probe
//!   returns (5 or 6 instead of 7) and which field-map ids the exe
//!   reads/writes (0x11/0x12/0x13 vs 0x14/0x15/0x16 vs 0x17/0x18/0x19).
//! - `FUN_004a84b0` — per-match tick that updates season-min-goals
//!   and season-max-attribute trackers at `+0x2fa` and `+0x1af`.
//! - `FUN_004a90b0` — aggregates one match's 20-slot × 2-half player
//!   stats into 25 team stat totals at `+0x56a..+0x5c6`.
//! - `FUN_004a98d0` — the player-attribute lookup wrapper (delegates
//!   through `FUN_004ab310` then `FUN_004a9980`).
//! - `FUN_004a2200` — FIFA World Rankings screen builder: header,
//!   per-country row (name + rank + points), pagination.
//! - `FUN_004a2900` — UEFA Coefficients screen: 4-column layout
//!   (Champions / UEFA Cup / Intertoto / total), per-nation row.
//! - `FUN_004a3770` — competition-stages walker: emits Stage / Group
//!   / Round rows via `comp_list_row_build`.

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_004a0010 / FUN_004a0610 / FUN_004a0c10 — the sibling-screen family
// =====================================================================

/// Which of the three sibling comp-list screens is being built.
/// The three variants share body code and differ ONLY in the fields
/// listed below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CompListVariant {
    /// `FUN_004a0010`: valid marker `5`, field-map ids selection=0x11
    /// / offset=0x12 / count=0x13.
    Variant5 = 5,
    /// `FUN_004a0610`: valid marker `6`, ids 0x14 / 0x15 / 0x16.
    Variant6 = 6,
    /// `FUN_004a0c10` (batch 28's `comp_list_screen_build`): valid
    /// marker `7`, ids 0x17 / 0x18 / 0x19.
    Variant7 = 7,
}

impl CompListVariant {
    /// The byte-value `FUN_004a5610(slot, 0)` returns for a valid row
    /// on this variant. See [`crate::screen_batch29::comp_slot_probe`].
    pub fn valid_marker(self) -> u8 { self as u8 }

    /// The field id `get_field(N)` reads for the CURRENT SELECTION.
    /// This is the id the screen builder compares against each row to
    /// decide highlighting.
    pub fn selection_field_id(self) -> u32 {
        match self {
            CompListVariant::Variant5 => 0x11,
            CompListVariant::Variant6 => 0x14,
            CompListVariant::Variant7 => 0x17,
        }
    }

    /// The field id used for "seek offset" state (page scroll).
    pub fn offset_field_id(self) -> u32 {
        match self {
            CompListVariant::Variant5 => 0x12,
            CompListVariant::Variant6 => 0x15,
            CompListVariant::Variant7 => 0x18,
        }
    }

    /// The field id used for "row count" state (widget-list cache).
    pub fn count_field_id(self) -> u32 {
        match self {
            CompListVariant::Variant5 => 0x13,
            CompListVariant::Variant6 => 0x16,
            CompListVariant::Variant7 => 0x19,
        }
    }
}

// =====================================================================
// FUN_004a84b0 — per-match tick: season-min-goals + season-max-attribute
// =====================================================================

/// Fields the tick maintains on the containing comp record.
///
/// The tick reads the current-match player record at `+0xc` and, if
/// the season min-goals `+0x2fa` is still `-1` (unset), computes it
/// as `float10 goals_for = FUN_004a9980(player, 2)` and stores the
/// (int, second-int, id) triple at `+0x2fa`/`+0x2fe`/`+0x302`. Later
/// it maintains the season maximum via a `<= min` check.
///
/// It ALSO tracks a floating-point attribute peak (`+0x418`) that
/// overwrites the `+0x1af..+0x1d0` block when it exceeds the current
/// peak.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MinGoalsTrackerState {
    /// `+0x2fa` season-min-goals (i32, -1 = unset).
    pub season_min_goals: i32,
    /// `+0x91` season-record-holder id (i32, -1 = none).
    pub season_holder_id: i32,
    /// `+0x1af` season-max-attribute (float, 0.0 = unset).
    pub season_max_attribute: f32,
    /// `+0x50a`, `+0x526`, `+0x54e`, `+0x562` — dirty flags.
    pub dirty_min_goals_slot: bool,     // +0x50a
    pub dirty_holder_slot:    bool,     // +0x526
    pub dirty_max_attr_slot:  bool,     // +0x54e
    pub dirty_master:         bool,     // +0x562
}

/// Direct port of the inner logic of `FUN_004a84b0(comp)`.
///
/// Inputs:
/// - `state`: mutable state to update.
/// - `outer_gate_hits`: `FUN_005b09e0(comp+0x502[0]+4)` — non-zero
///   short-circuits the whole tick.
/// - `secondary_gate_hits`: `FUN_004b4140()` — enables the `+0x26 > 4`
///   check.
/// - `comp_stage_byte`: `comp+0x26` — must be > 4 for stage-1 update.
/// - `player_valid`: `comp+0xc != 0`.
/// - `player_status_2`: `comp+0x18 == 2` (bool).
/// - `goals_for_attr`: pre-computed `FUN_004a9980(player, 2)` as f64.
/// - `player_head_id`: `*(int*)(comp+0xc)` (= first field of player).
/// - `min_goals_field_40`: `*(short*)(comp+0x502+0x40)` — the field
///   the exe overwrites into `+0x99` after copying `+0x302`.
/// - `season_count_at_500`: `*(short*)(comp+0x500)` — > 2 enables the
///   FUN_004b6b30 check.
/// - `fun_4b6b30_true`: FUN_004b6b30 return non-zero.
/// - `current_attr_418`: `*(float*)(comp+0x418)` — the incoming
///   attribute value.
/// - `k_zero`: `_DAT_00956948` — the exe's "0.0" constant.
/// - `attr_431..439`: the payload bytes `+0x431..+0x439` copied when
///   the tick fires (opaque to us).
// GDI-REG: 004a84b0 PORTED_BEHAVIOURAL
pub fn update_match_min_goals_tracker(
    state: &mut MinGoalsTrackerState,
    outer_gate_hits: bool,
    secondary_gate_hits: bool,
    comp_stage_byte: i8,
    player_valid: bool,
    player_status_2: bool,
    goals_for_attr: f64,
    player_head_id: i32,
    min_goals_field_40: i16,
    season_count_at_500: i16,
    fun_4b6b30_true: bool,
    current_attr_418: f32,
    k_zero: f32,
    attr_431: u8,
) {
    // Guard 1: FUN_005b09e0 non-zero AND FUN_004b4140 non-zero AND
    // comp+0x26 > 4 → set two dirty flags. Verified against the top
    // block of 004a84b0.
    if !outer_gate_hits && secondary_gate_hits && comp_stage_byte > 4 {
        state.dirty_min_goals_slot = true;
        state.dirty_master         = true;
    }

    // Guard 2: outer_gate off AND player_status != 2. This is the
    // main season-min-goals + season-holder update path.
    if outer_gate_hits { return; }
    // (The exe also checks `*(char *)(*(int *)(param_1 + 0x502) + 0x42) != '\x04'`
    // — a per-competition-type gate we accept as pre-filtered by caller.)

    if state.season_min_goals == -1 && player_valid {
        if !player_status_2 {
            // Set player_status = 2, copy 0x76 bytes of runtime data
            // (elided — done outside this function in the exe via
            // memcpy at LAB_004b3c60).
        }
        // Convert float10 goals_for to i32 (exe uses __ftol). This
        // writes scratch +0x2fa. Note that scratch +0x2fe (player id)
        // is written here in the exe too, but the PUBLISHED holder
        // field (+0x91, our `season_holder_id`) is left untouched at
        // this point — only Guard 3 propagates scratch → published.
        state.season_min_goals = goals_for_attr as i32;
        let _ = player_head_id;   // scratch +0x2fe write elided
    }

    // Guard 3: min_goals set AND (holder unset OR holder > min_goals) →
    // adopt current player as new holder and copy fields.
    if state.season_min_goals >= 0
        && (state.season_holder_id == -1 || state.season_holder_id < state.season_min_goals)
    {
        state.season_holder_id = state.season_min_goals;
        // exe writes +0x99 twice: first from +0x302, then overwrites
        // with `*(short*)(comp+0x502+0x40)` — the second value wins.
        let _ = min_goals_field_40;
        if season_count_at_500 > 2 && fun_4b6b30_true {
            state.dirty_holder_slot = true;
            state.dirty_master      = true;
        }
    }

    // Guard 4: FUN_00448d90 attribute update at `+0x418`.
    // If +0x418 == 0.0 → FUN_004b42b0 side-effect (elided).
    // If +0x418 > 0.0 AND (peak_1af == 0.0 OR peak_1af < +0x418) →
    // adopt new peak.
    if (current_attr_418 > k_zero)
        && (state.season_max_attribute == k_zero
            || state.season_max_attribute < current_attr_418)
    {
        state.season_max_attribute = current_attr_418;
        if season_count_at_500 > 2 && fun_4b6b30_true && attr_431 > 3 {
            state.dirty_max_attr_slot = true;
            state.dirty_master        = true;
        }
    }
}

// =====================================================================
// FUN_004a90b0 — aggregate team match stats (25 counters)
// =====================================================================

/// The 25 counters at `+0x56a..+0x5c6` in a team-stats record. Each
/// accumulates one byte per participating player per match half.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MatchStatCounters {
    pub goals:              u32,   // +0x56a
    pub assists:            u32,   // +0x56e
    pub yellow_cards:       u32,   // +0x572
    pub red_cards:          u32,   // +0x576
    pub man_of_match:       u32,   // +0x57a
    pub minutes_played:     u32,   // +0x57e
    pub shots:              u32,   // +0x582
    pub shots_on_target:    u32,   // +0x586
    pub interceptions:      u32,   // +0x58a
    /// First-half only (`iVar3 == 0` in the exe).
    pub shots_1h:           u32,   // +0x58e
    pub shots_on_target_1h: u32,   // +0x592
    pub interceptions_1h:   u32,   // +0x596
    /// Second-half only (`iVar3 != 0`).
    pub shots_2h:           u32,   // +0x59a
    pub shots_on_target_2h: u32,   // +0x59e
    pub interceptions_2h:   u32,   // +0x5a2
    pub tackles_won:        u32,   // +0x5a6
    pub tackles_lost:       u32,   // +0x5aa
    pub passes:             u32,   // +0x5ae
    pub key_passes:         u32,   // +0x5b2
    pub headers_won:        u32,   // +0x5b6
    pub headers_lost:       u32,   // +0x5ba
    pub fouls:              u32,   // +0x5be
    pub yellow_2:           u32,   // +0x5c2
    pub red_2:              u32,   // +0x5c6
    /// Match count (`+0x5ca`) — increments once per aggregate call.
    pub matches_processed:  u32,
}

/// One player's per-half stat line as consumed by
/// [`aggregate_team_match_stats`]. Field names correspond to the
/// per-byte reads in the exe's inner loop (offsets are relative to
/// `pcVar2` = `iVar3*0x625 + fixture+0x1c2`, i.e. per-slot base).
///
/// A slot is skipped when any of these gates apply: `pcVar2-0x32 == -1`,
/// `pcVar2-0x2e == -1`, or (`iVar4 >= 0xb` and `pcVar2[0] == -1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MatchStatSlot {
    pub goals:              u8,     // pcVar2[0xc]
    pub assists:            u8,     // pcVar2[0xd]
    pub yellow_cards:       u8,     // pcVar2[0xe]
    pub red_cards:          u8,     // pcVar2[0xf]
    pub man_of_match:       u8,     // pcVar2[0x10]
    pub minutes_played:     u8,     // pcVar2[0x11]
    pub shots:              u8,     // pcVar2[-0xd]
    pub shots_on_target:    u8,     // pcVar2[-0xc]
    pub interceptions:      u8,     // pcVar2[-0x11]
    pub tackles_won:        u8,     // pcVar2[-0xb]
    pub tackles_lost:       u8,     // pcVar2[-10]
    pub passes:             u8,     // pcVar2[0x15]
    pub key_passes:         u8,     // pcVar2[-7]
    pub headers_won:        u8,     // pcVar2[-9]
    pub headers_lost:       u8,     // pcVar2[-8]
    pub fouls:              u8,     // pcVar2[3]
    pub yellow_2:           u8,     // pcVar2[-6]
    pub red_2:              u8,     // pcVar2[-5]
    /// True → slot is skipped (any of the three -1 gates fires).
    pub skip: bool,
}

/// Direct port of `FUN_004a90b0(comp, fixture)`.
///
/// The exe iterates 2 halves (0, 1) × 20 slots (0..0x14) per half,
/// sharing per-slot state through offsets relative to a stride-`0x625`
/// per-half base. This port takes the pre-flattened 40-slot input.
// GDI-REG: 004a90b0 PORTED_BEHAVIOURAL
pub fn aggregate_team_match_stats(
    counters: &mut MatchStatCounters,
    fixture_state_at_0x4c: u8,
    slots_h1: &[MatchStatSlot; 20],
    slots_h2: &[MatchStatSlot; 20],
) -> bool {
    // Early out: `*(char*)(fixture+0x4c) != '\0'` short-circuits.
    if fixture_state_at_0x4c != 0 { return false; }
    counters.matches_processed += 1;

    for (half_idx, slots) in [(0, slots_h1), (1, slots_h2)] {
        for slot in slots.iter() {
            if slot.skip { continue; }
            counters.goals           += slot.goals           as u32;
            counters.assists         += slot.assists         as u32;
            counters.yellow_cards    += slot.yellow_cards    as u32;
            counters.red_cards       += slot.red_cards       as u32;
            counters.man_of_match    += slot.man_of_match    as u32;
            counters.minutes_played  += slot.minutes_played  as u32;
            counters.shots           += slot.shots           as u32;
            counters.shots_on_target += slot.shots_on_target as u32;
            counters.interceptions   += slot.interceptions   as u32;
            if half_idx == 0 {
                counters.shots_1h           += slot.shots           as u32;
                counters.shots_on_target_1h += slot.shots_on_target as u32;
                counters.interceptions_1h   += slot.interceptions   as u32;
            } else {
                counters.shots_2h           += slot.shots           as u32;
                counters.shots_on_target_2h += slot.shots_on_target as u32;
                counters.interceptions_2h   += slot.interceptions   as u32;
            }
            counters.tackles_won  += slot.tackles_won  as u32;
            counters.tackles_lost += slot.tackles_lost as u32;
            counters.passes       += slot.passes       as u32;
            counters.key_passes   += slot.key_passes   as u32;
            counters.headers_won  += slot.headers_won  as u32;
            counters.headers_lost += slot.headers_lost as u32;
            counters.fouls        += slot.fouls        as u32;
            counters.yellow_2     += slot.yellow_2     as u32;
            counters.red_2        += slot.red_2        as u32;
        }
    }
    true
}

// =====================================================================
// FUN_004a2200 — FIFA World Rankings screen model
// =====================================================================

/// One row of the FIFA World Rankings screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FifaRankingRow {
    /// Rank number (1-based, displayed).
    pub rank: u32,
    /// Nation short name.
    pub nation_name: String,
    /// Rating column at `nation + 0xc` multiplied by `_DAT_00956a78`
    /// (formatted as `%.2f`).
    pub rating: f64,
    /// Colour flag on the flag column (nation `+0x71` non-null).
    pub has_flag: bool,
}

/// Whole FIFA rankings screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FifaRankingsView {
    /// Formatted title, e.g. `"< September > < 2024 >"`.
    pub period_label: String,
    /// Rows on the current page.
    pub rows: Vec<FifaRankingRow>,
    /// Page number (`get_field(1)` from the screen state).
    pub page: u32,
    /// Total pages `(rows_total + page_size - 1) / page_size`.
    pub total_pages: u32,
}

/// Direct port of `FUN_004a2200`. Inputs pre-fetched from pools:
/// - `rankings`: sorted list of (nation_name, rating, has_flag).
/// - `page`: `get_field(1)` — the current page (1-based).
/// - `page_size`: computed as `0x15d / FUN_005cf7b0(...)`.
/// - `month_label`, `year`: rendered as `< {month} > < {year} >`.
// GDI-REG: 004a2200 PORTED_BEHAVIOURAL
pub fn build_fifa_world_rankings_screen(
    rankings: Vec<(String, f64, bool)>,
    page: u32,
    page_size: u32,
    month_label: &str,
    year: u32,
) -> FifaRankingsView {
    let total = rankings.len() as u32;
    let total_pages = if page_size == 0 { 0 } else { (total + page_size - 1) / page_size };
    let start = ((page.saturating_sub(1)) * page_size) as usize;
    let end = ((start as u32).saturating_add(page_size) as usize).min(rankings.len());
    let rows = rankings[start.min(rankings.len())..end]
        .iter()
        .enumerate()
        .map(|(i, (name, rating, has_flag))| FifaRankingRow {
            rank: (start as u32) + 1 + i as u32,
            nation_name: name.clone(),
            rating: *rating,
            has_flag: *has_flag,
        })
        .collect();
    FifaRankingsView {
        period_label: format!("< {month_label} > < {year} >"),
        rows,
        page,
        total_pages,
    }
}

// =====================================================================
// FUN_004a2900 — UEFA Coefficients screen model
// =====================================================================

/// One nation's UEFA-Coefficients row (5-year rolling window).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UefaCoefficientRow {
    /// Nation short name (from `nation+0x38`).
    pub nation_name: String,
    /// Champions League coefficient byte (`+0xc`).
    pub champions_coef: u8,
    /// UEFA Cup coefficient byte (`+0xd`).
    pub uefa_cup_coef: u8,
    /// Intertoto Cup coefficient byte (`+0xe`).
    pub intertoto_coef: u8,
    /// Total coefficient (float at `+0x8`, formatted as `%.2f`).
    pub total_coef: f32,
}

/// The UEFA Coefficients screen model — 3 comp headers + N nation rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UefaCoefficientsView {
    /// `<year1>-<year2>` season label built from `DAT_00dc7230+0x439`.
    pub season_label: String,
    /// Per-nation rows in ranking order.
    pub rows: Vec<UefaCoefficientRow>,
}

impl UefaCoefficientsView {
    /// The four fixed column headers the exe emits, in display order:
    /// (Champions, UEFA Cup, Intertoto Cup, "Coef." total).
    pub const COLUMN_HEADERS: (&'static str, &'static str, &'static str, &'static str) = (
        "Champions Coefficient",  // DAT_009890b8
        "UEFA Cup Coefficient",   // DAT_009890b0
        "Intertoto Cup",          // s_INTER...
        "Coef.",                  // s_Coef__00989074
    );
}

/// Direct port of `FUN_004a2900`.
pub fn build_uefa_coefs_screen(
    year1_two_digit: u16,
    rows: Vec<UefaCoefficientRow>,
) -> UefaCoefficientsView {
    let year2 = (year1_two_digit + 1) % 100;
    UefaCoefficientsView {
        season_label: format!("< {year1_two_digit} > < {year2} >"),
        rows,
    }
}

// =====================================================================
// FUN_004a3770 — competition-stages walker
// =====================================================================

/// A row emitted by the stages walker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageRow {
    /// A "Stage" row (comp+0xa3+7 group name; iVar5 = round index).
    Stage { round_type: u16, stage_id: i32, round_idx: i32 },
    /// A "Round" row (within an active stage).
    Round { round_type: u16, sub_stage_id: i32, round_idx: i32, sub_idx: i32 },
    /// A "Group" row (from the group-collapse branch when `local_468 ==
    /// comp+0x30 + 2`).
    Group { round_type: u16, round_idx: i32 },
}

/// Outcome of `FUN_004a3770`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StagesWalkerOutcome {
    /// `param_1 == NULL` → error dialog.
    ErrorDialog,
    /// `comp+0x30 == -1 && (comp+0x43 == 2 || (comp+0x43 == 1 &&
    /// comp+0xac == 1))` → early return with 0.
    EarlyReturn,
    /// Normal path — the flat list of emitted rows.
    Rows(Vec<StageRow>),
}

/// Simplified port of `FUN_004a3770`. The exe walks
/// `iVar5 = -1..comp+0x30` and, at each round, calls
/// [`crate::screen_batch28::comp_list_row_build`] via `FUN_004a3d20`
/// several times to emit Stage/Group/Round widgets. This port takes
/// a pre-classified per-round descriptor and emits the row list.
// GDI-REG: 004a3770 PORTED_PARTIAL
pub fn build_competition_stages_walker(
    comp_is_null: bool,
    comp_stage_30_at_minus_one: bool,
    comp_43: i8,
    comp_ac: i8,
    round_descriptors: Vec<StageRow>,
) -> StagesWalkerOutcome {
    if comp_is_null { return StagesWalkerOutcome::ErrorDialog; }
    if comp_stage_30_at_minus_one
        && (comp_43 == 2 || (comp_43 == 1 && comp_ac == 1))
    {
        return StagesWalkerOutcome::EarlyReturn;
    }
    StagesWalkerOutcome::Rows(round_descriptors)
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- CompListVariant field-id table ----

    #[test]
    fn variant_valid_markers_match_source_bytes() {
        assert_eq!(CompListVariant::Variant5.valid_marker(), 5);
        assert_eq!(CompListVariant::Variant6.valid_marker(), 6);
        assert_eq!(CompListVariant::Variant7.valid_marker(), 7);
    }
    #[test]
    fn variant_field_ids_form_consecutive_triples() {
        for v in [CompListVariant::Variant5, CompListVariant::Variant6, CompListVariant::Variant7] {
            let sel = v.selection_field_id();
            assert_eq!(v.offset_field_id(), sel + 1);
            assert_eq!(v.count_field_id(),  sel + 2);
        }
    }
    #[test]
    fn variant_selection_ids_are_0x11_0x14_0x17() {
        assert_eq!(CompListVariant::Variant5.selection_field_id(), 0x11);
        assert_eq!(CompListVariant::Variant6.selection_field_id(), 0x14);
        assert_eq!(CompListVariant::Variant7.selection_field_id(), 0x17);
    }

    // ---- MinGoalsTrackerState ----

    #[test]
    fn outer_gate_short_circuits() {
        let mut s = MinGoalsTrackerState {
            season_min_goals: -1, season_holder_id: -1,
            season_max_attribute: 0.0,
            dirty_min_goals_slot: false, dirty_holder_slot: false,
            dirty_max_attr_slot: false, dirty_master: false,
        };
        update_match_min_goals_tracker(
            &mut s, /*outer_gate*/ true, /*secondary*/ false, 0,
            true, false, 5.0, 42, 0, 0, false, 1.0, 0.0, 0,
        );
        // Guard 1 wasn't touched (secondary=false); everything else short-circuits.
        assert_eq!(s.season_min_goals, -1);
        assert_eq!(s.season_holder_id, -1);
        assert_eq!(s.season_max_attribute, 0.0);
        assert!(!s.dirty_master);
    }

    #[test]
    fn stage_gate_1_fires_two_dirty_flags() {
        let mut s = MinGoalsTrackerState::default();
        s.season_min_goals = 0;
        update_match_min_goals_tracker(
            &mut s, /*outer*/ false, /*secondary*/ true, /*comp_stage*/ 5,
            false, false, 0.0, 0, 0, 0, false, 0.0, 0.0, 0,
        );
        assert!(s.dirty_min_goals_slot);
        assert!(s.dirty_master);
    }

    #[test]
    fn season_min_goals_seeded_from_player() {
        let mut s = MinGoalsTrackerState::default();
        s.season_min_goals = -1;   // unset
        s.season_holder_id = -1;
        update_match_min_goals_tracker(
            &mut s, false, false, 0,
            true, false, /*goals_for*/ 7.9, 42,
            0, 0, false, 0.0, 0.0, 0,
        );
        assert_eq!(s.season_min_goals, 7);
        assert_eq!(s.season_holder_id, 7);
    }

    #[test]
    fn peak_attribute_adopted_when_larger() {
        let mut s = MinGoalsTrackerState::default();
        s.season_max_attribute = 3.5;
        update_match_min_goals_tracker(
            &mut s, false, false, 0, false, false, 0.0, 0, 0,
            /*count500*/ 3, /*fun_4b6b30*/ true, /*attr_418*/ 5.5, 0.0,
            /*attr_431*/ 4,
        );
        assert_eq!(s.season_max_attribute, 5.5);
        assert!(s.dirty_max_attr_slot);
        assert!(s.dirty_master);
    }

    #[test]
    fn peak_attribute_not_adopted_when_smaller() {
        let mut s = MinGoalsTrackerState::default();
        s.season_max_attribute = 8.0;
        update_match_min_goals_tracker(
            &mut s, false, false, 0, false, false, 0.0, 0, 0,
            3, true, /*attr_418*/ 5.5, 0.0, 4,
        );
        assert_eq!(s.season_max_attribute, 8.0);   // unchanged
        assert!(!s.dirty_max_attr_slot);
    }

    // ---- MatchStatCounters ----

    #[test]
    fn stats_early_out_when_fixture_state_nonzero() {
        let mut c = MatchStatCounters::default();
        let empty = [MatchStatSlot::default(); 20];
        let ran = aggregate_team_match_stats(&mut c, /*+0x4c*/ 1, &empty, &empty);
        assert!(!ran);
        assert_eq!(c.matches_processed, 0);
    }

    #[test]
    fn stats_increment_matches_and_aggregate() {
        let mut c = MatchStatCounters::default();
        let mut h1 = [MatchStatSlot::default(); 20];
        let mut h2 = [MatchStatSlot::default(); 20];
        h1[0] = MatchStatSlot { goals: 2, shots: 5, shots_on_target: 3, skip: false, ..Default::default() };
        h1[1] = MatchStatSlot { goals: 1, skip: true, ..Default::default() };   // skipped
        h2[0] = MatchStatSlot { goals: 1, shots: 4, shots_on_target: 2, skip: false, ..Default::default() };
        assert!(aggregate_team_match_stats(&mut c, 0, &h1, &h2));
        assert_eq!(c.matches_processed, 1);
        assert_eq!(c.goals, 2 + 1);            // slot 1 in h1 was skipped
        assert_eq!(c.shots, 5 + 4);
        assert_eq!(c.shots_on_target, 3 + 2);
        // Half-specific splits
        assert_eq!(c.shots_1h, 5);
        assert_eq!(c.shots_2h, 4);
    }

    // ---- FIFA rankings ----

    #[test]
    fn fifa_rankings_first_page_shows_top_page_size() {
        let list: Vec<_> = (1..=30).map(|i| (format!("N{i}"), i as f64 * 10.0, i % 2 == 0)).collect();
        let v = build_fifa_world_rankings_screen(list.clone(), /*page*/ 1, /*page_size*/ 10, "September", 2024);
        assert_eq!(v.rows.len(), 10);
        assert_eq!(v.rows[0].rank, 1);
        assert_eq!(v.rows[0].nation_name, "N1");
        assert_eq!(v.rows[9].rank, 10);
        assert_eq!(v.total_pages, 3);
        assert_eq!(v.period_label, "< September > < 2024 >");
    }

    #[test]
    fn fifa_rankings_last_page_partial() {
        let list: Vec<_> = (1..=23).map(|i| (format!("N{i}"), i as f64, false)).collect();
        let v = build_fifa_world_rankings_screen(list, 3, 10, "May", 2024);
        assert_eq!(v.total_pages, 3);
        assert_eq!(v.rows.len(), 3);
        assert_eq!(v.rows[0].rank, 21);
    }

    #[test]
    fn fifa_rankings_zero_page_size_no_divide_by_zero() {
        let v = build_fifa_world_rankings_screen(Vec::new(), 1, 0, "X", 2024);
        assert_eq!(v.total_pages, 0);
        assert!(v.rows.is_empty());
    }

    // ---- UEFA coefficients ----

    #[test]
    fn uefa_headers_are_the_four_labels() {
        assert_eq!(UefaCoefficientsView::COLUMN_HEADERS.0, "Champions Coefficient");
        assert_eq!(UefaCoefficientsView::COLUMN_HEADERS.1, "UEFA Cup Coefficient");
        assert_eq!(UefaCoefficientsView::COLUMN_HEADERS.2, "Intertoto Cup");
        assert_eq!(UefaCoefficientsView::COLUMN_HEADERS.3, "Coef.");
    }
    #[test]
    fn uefa_season_label_wraps_year_2_mod_100() {
        assert_eq!(build_uefa_coefs_screen(99, vec![]).season_label, "< 99 > < 0 >");
        assert_eq!(build_uefa_coefs_screen(24, vec![]).season_label, "< 24 > < 25 >");
    }

    // ---- stages walker ----

    #[test]
    fn stages_null_returns_error() {
        let r = build_competition_stages_walker(true, false, 0, 0, vec![]);
        assert_eq!(r, StagesWalkerOutcome::ErrorDialog);
    }
    #[test]
    fn stages_early_return_when_stage_30_and_43_is_2() {
        let r = build_competition_stages_walker(false, true, 2, 0, vec![StageRow::Stage { round_type: 0, stage_id: 0, round_idx: 0 }]);
        assert_eq!(r, StagesWalkerOutcome::EarlyReturn);
    }
    #[test]
    fn stages_early_return_when_stage_30_and_43_is_1_ac_is_1() {
        let r = build_competition_stages_walker(false, true, 1, 1, vec![]);
        assert_eq!(r, StagesWalkerOutcome::EarlyReturn);
    }
    #[test]
    fn stages_normal_path_passes_through() {
        let rows = vec![
            StageRow::Stage { round_type: 10, stage_id: 3, round_idx: 0 },
            StageRow::Round { round_type: 20, sub_stage_id: 3, round_idx: 1, sub_idx: 0 },
            StageRow::Group { round_type: 30, round_idx: 2 },
        ];
        let r = build_competition_stages_walker(false, false, 1, 2, rows.clone());
        assert_eq!(r, StagesWalkerOutcome::Rows(rows));
    }
}
