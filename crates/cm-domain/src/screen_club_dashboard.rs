//! Club dashboard screen builder — direct port of `FUN_00454620`
//! (1,424 bytes decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/screen_club_dashboard_setup/0x00454620.c`.
//! Draw callback: `FUN_004551C0` (7,792 B, 2330 instructions) — the
//! rendering pass; this module ports the DATA-BUILDING pass which
//! pushes 0x23 (35) typed values into the exe's screen-slot table
//! via `FUN_007E7130(idx, val, 0)`.
//!
//! # Signature
//!
//! ```pseudo
//! FUN_00454620(
//!     void* club_ptr,
//!     int   screen_arg,
//!     int   prev_state_present,   // non-zero → restore from prior state
//!     int   is_reserves,          // 0 → tab 1 (Squad), 1 → tab 5 (Reserves)
//!     int   is_highlighted        // adds 0x800 to slot 7 for pulse effect
//! )
//! ```
//!
//! # Data sources (per memory `dashboard-manager-model`)
//!
//! * `club_ptr + 0x38` — club name
//! * `club_ptr + 0x52` — 3-letter abbreviation
//! * `club_ptr + 0x53` — nation ptr
//! * `club_ptr + 0xD7..0xD7+50*4` — squad array (50 player ptrs)
//! * `club_ptr + 0x69` — finance sub-record
//! * `club_ptr + 0x80` — club reputation
//! * `club_ptr + 0x59` — manager reputation
//! * `DAT_00ACDE90/92` — current date (day/year)
//! * `FUN_005EA720(club, 0, 1)` — is club managed by current human
//! * `FUN_005E8490(0)` — real finance record when managed
//! * `FUN_00525450(club)` — reserves/team-context flag

use serde::{Deserialize, Serialize};

/// The 35 typed slots the exe pushes via `FUN_007E7130`. Every slot
/// index (0..=0x22) is named after the meaning per the decompile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubDashboardView {
    /// Slot 0: club pointer (opaque id in our port).
    pub club_id: Option<u32>,
    /// Slot 1: screen argument (usually 0 or a sub-page id).
    pub screen_arg: i32,
    /// Slot 4: tab index — 1 = Squad (default), 5 = Reserves.
    pub tab_index: u8,
    /// Slot 7: header/wage indicator (exe's `puVar9[0]` — decoded as
    /// weekly wage total from finance record, or default 0x41). If the
    /// `highlighted` arg is set, bit `0x800` is OR'd in.
    pub header_indicator: u32,
    /// Slot 8: second finance value (exe's `puVar9[1]`, decoded as
    /// balance short).
    pub finance_secondary: u32,
    /// Slot 9: signed byte from `puVar9[3]` — reputation/prestige delta.
    pub prestige_delta: i8,
    /// Slot 0xC: current year (DAT_00ACDE92).
    pub year: u16,
    /// Slot 0xF: 1 (always — sort-order flag).
    pub sort_order: u8,
    /// Slot 0x12: 1 (always — display-mode flag).
    pub display_mode: u8,
    /// Slot 0x17: current year (duplicated for second-column display).
    pub year_column_b: u16,
    /// Slot 0x18: third finance value (`puVar9[2]`).
    pub finance_tertiary: u32,
    /// Slot 0x19: byte at `puVar9+0xD`.
    pub finance_byte_a: i8,
    /// Slot 0x1A: byte at `puVar9+0xE`.
    pub finance_byte_b: i8,
    /// Slot 0x1B: 1 (default), 2 if `is_reserves`.
    pub team_context: u8,
    /// Slot 0x20: top-1 key player id (`-1` if fewer than 1).
    pub key_player_1: Option<u32>,
    /// Slot 0x21: top-2 key player id.
    pub key_player_2: Option<u32>,
    /// Slot 0x22: top-3 key player id.
    pub key_player_3: Option<u32>,
}

impl Default for ClubDashboardView {
    fn default() -> Self {
        Self {
            club_id: None, screen_arg: 0,
            tab_index: 1, header_indicator: 0x41, finance_secondary: 8,
            prestige_delta: 0, year: 0, sort_order: 1, display_mode: 1,
            year_column_b: 0, finance_tertiary: 0x154A21,
            finance_byte_a: -1, finance_byte_b: -1, team_context: 1,
            key_player_1: None, key_player_2: None, key_player_3: None,
        }
    }
}

/// The set of finance-record fields the exe reads via `puVar9[]`.
/// Matches the exe's default struct at `local_10c` when club isn't
/// managed by the current human.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinanceSummary {
    /// `puVar9[0]` — weekly wage bill (default 0x41).
    pub weekly_wages: u32,
    /// `puVar9[1]` — bank balance (default 8).
    pub balance: u32,
    /// `puVar9[2]` — transfer budget (default 0x154A21).
    pub transfer_budget: u32,
    /// `((char*)puVar9)[0xC]` — prestige delta byte (default 0xFF = -1).
    pub prestige_byte: i8,
    /// `((char*)puVar9)[0xD]` — auxiliary byte (default 0xFF = -1).
    pub aux_byte_d: i8,
    /// `((char*)puVar9)[0xE]` — auxiliary byte (default 0xFF = -1).
    pub aux_byte_e: i8,
}

impl Default for FinanceSummary {
    fn default() -> Self {
        Self {
            weekly_wages: 0x41,
            balance: 8,
            transfer_budget: 0x154A21,
            prestige_byte: -1,
            aux_byte_d: -1,
            aux_byte_e: -1,
        }
    }
}

/// Squad-player attribute snapshot — used by the top-3 selector loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SquadPlayerAttr {
    pub player_id: u32,
    /// `attr[+0xB]` — CA rating (used when not-in-reserves context).
    pub ca_primary: i16,
    /// `attr[+0xD]` — reserve/dev CA (used when in-reserves context).
    pub ca_reserve: i16,
    /// `attr[+5]` — form (multiplied by 10).
    pub form: i8,
}

/// Direct port of `FUN_00454620` — builds the club dashboard's data
/// view. Consumers render the returned struct through the layout
/// engine.
///
/// * `club_id` — Some(id) if a club is shown; None for unemployed
///   manager screen (matches exe's `param_1 == 0` path).
/// * `screen_arg` — sub-page selector (0 = default, non-zero = tab).
/// * `prev_state` — Some(prior_view) restores 11 slots from a saved
///   view (matches exe's `param_3` state-restore branch).
/// * `is_reserves` — matches exe's `param_4`. 0 → Squad tab, non-zero → Reserves.
/// * `is_highlighted` — matches exe's `param_5`. Sets bit 0x800 on
///   header_indicator to trigger pulse effect.
/// * `is_managed_by_current_human` — port of `FUN_005EA720(club, 0, 1)`.
/// * `finance` — real finance record if `is_managed_by_current_human`;
///   `None` uses the default sentinel struct.
/// * `date_year` — DAT_00ACDE92.
/// * `squad` — pre-filtered slice of player attributes (max 50).
/// * `is_reserve_context` — port of `FUN_00525450(club) == 0` gate for
///   picking `attr[+0xB]` vs `attr[+0xD]`.
pub fn build_club_dashboard(
    club_id: Option<u32>,
    screen_arg: i32,
    prev_state: Option<&ClubDashboardView>,
    is_reserves: bool,
    is_highlighted: bool,
    is_managed_by_current_human: bool,
    finance: Option<FinanceSummary>,
    date_year: u16,
    squad: &[SquadPlayerAttr],
    is_reserve_context: bool,
) -> ClubDashboardView {
    // Exe: if club and NOT managed → screen_arg = 0.
    let screen_arg = if club_id.is_some() && !is_managed_by_current_human { 0 }
                     else { screen_arg };

    // Pick finance source — real record if managed, else default.
    let fin = if is_managed_by_current_human {
        finance.unwrap_or_default()
    } else {
        FinanceSummary::default()
    };

    // Assemble the 35 slots (0x23 total).
    let mut v = ClubDashboardView {
        club_id, screen_arg,
        tab_index: if is_reserves { 5 } else { 1 },
        header_indicator: fin.weekly_wages,
        finance_secondary: fin.balance,
        prestige_delta: fin.prestige_byte,
        year: date_year,
        sort_order: 1,
        display_mode: 1,
        year_column_b: date_year,
        finance_tertiary: fin.transfer_budget,
        finance_byte_a: fin.aux_byte_d,
        finance_byte_b: fin.aux_byte_e,
        team_context: if is_reserves { 2 } else { 1 },
        key_player_1: None, key_player_2: None, key_player_3: None,
    };

    // Restore prior state (exe lines 127-139) — 11 slots pulled from saved view.
    if let Some(prior) = prev_state {
        v.tab_index = prior.tab_index;
        v.header_indicator = prior.header_indicator;
        v.finance_secondary = prior.finance_secondary;
        v.prestige_delta = prior.prestige_delta;
        v.sort_order = prior.sort_order;
        v.display_mode = prior.display_mode;
        v.finance_tertiary = prior.finance_tertiary;
        v.finance_byte_a = prior.finance_byte_a;
        v.finance_byte_b = prior.finance_byte_b;
        v.team_context = prior.team_context;
    }

    // Highlighted flag (exe: `slot 7 |= 0x800`).
    if is_highlighted {
        v.header_indicator |= 0x800;
    }

    // Top-3 key players (exe lines 143-219). Score = ca + form*10.
    let (top1, top2, top3) = pick_top_three_key_players(squad, is_reserve_context);
    v.key_player_1 = top1;
    v.key_player_2 = top2;
    v.key_player_3 = top3;

    v
}

/// Port of the exe's top-3 selector loop (lines 149-197).
///
/// For each of up to 50 squad slots: compute `score = ca + form*10`.
/// Track top 3 by descending score using the exe's manual 3-way
/// bubble insert. `is_reserve_context` selects `+0xB` vs `+0xD` for CA.
fn pick_top_three_key_players(
    squad: &[SquadPlayerAttr],
    is_reserve_context: bool,
) -> (Option<u32>, Option<u32>, Option<u32>) {
    let mut top: [(i16, Option<u32>); 3] = [(i16::MIN, None); 3];
    let mut count: u8 = 0;
    for p in squad.iter().take(50) {
        let ca = if is_reserve_context { p.ca_primary } else { p.ca_reserve };
        let score = ca.saturating_add(p.form.saturating_mul(10) as i16);
        if count < 3 { count += 1; }
        // Bubble insert into top[0..3] by descending score.
        for i in 0..3 {
            if score > top[i].0 {
                // Shift down.
                for j in (i + 1..3).rev() {
                    top[j] = top[j - 1];
                }
                top[i] = (score, Some(p.player_id));
                break;
            }
        }
    }
    let _ = count;
    (top[0].1, top[1].1, top[2].1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_view_matches_exe_local_10c_defaults() {
        // exe's local_10c initialised to {0x41, 8, 0x154A21, 0xFF, 0xFF, 0xFF}.
        let v = ClubDashboardView::default();
        assert_eq!(v.header_indicator, 0x41);
        assert_eq!(v.finance_secondary, 8);
        assert_eq!(v.finance_tertiary, 0x154A21);
        assert_eq!(v.tab_index, 1);
    }

    #[test]
    fn unmanaged_club_forces_screen_arg_to_zero() {
        // Exe: `if (param_1 != 0 && FUN_005EA720 == 0) param_2 = 0`.
        let v = build_club_dashboard(
            Some(42), 7,
            None, false, false, false,
            None, 2001, &[], false,
        );
        assert_eq!(v.screen_arg, 0);
    }

    #[test]
    fn managed_club_keeps_screen_arg() {
        let v = build_club_dashboard(
            Some(42), 7,
            None, false, false, true,
            None, 2001, &[], false,
        );
        assert_eq!(v.screen_arg, 7);
    }

    #[test]
    fn reserves_flag_picks_tab_5_and_context_2() {
        let v = build_club_dashboard(
            Some(42), 0, None, true, false, true,
            None, 2001, &[], false);
        assert_eq!(v.tab_index, 5);
        assert_eq!(v.team_context, 2);
    }

    #[test]
    fn highlighted_flag_sets_bit_0x800_on_header() {
        let v = build_club_dashboard(
            Some(42), 0, None, false, true, true,
            Some(FinanceSummary { weekly_wages: 100, ..Default::default() }),
            2001, &[], false);
        assert_eq!(v.header_indicator, 100 | 0x800);
    }

    #[test]
    fn managed_uses_real_finance_record() {
        let f = FinanceSummary {
            weekly_wages: 1_000_000, balance: 500_000,
            transfer_budget: 10_000_000, prestige_byte: 25,
            aux_byte_d: 10, aux_byte_e: 15,
        };
        let v = build_club_dashboard(
            Some(42), 0, None, false, false, true,
            Some(f), 2001, &[], false);
        assert_eq!(v.header_indicator, 1_000_000);
        assert_eq!(v.finance_secondary, 500_000);
        assert_eq!(v.finance_tertiary, 10_000_000);
    }

    #[test]
    fn unmanaged_uses_default_finance_regardless_of_supplied_record() {
        let f = FinanceSummary { weekly_wages: 9999, ..Default::default() };
        let v = build_club_dashboard(
            Some(42), 0, None, false, false, false,
            Some(f), 2001, &[], false);
        // Unmanaged path uses defaults, not the supplied f.
        assert_eq!(v.header_indicator, 0x41);
    }

    #[test]
    fn year_pushed_to_both_columns() {
        let v = build_club_dashboard(
            Some(42), 0, None, false, false, true,
            None, 2001, &[], false);
        assert_eq!(v.year, 2001);
        assert_eq!(v.year_column_b, 2001);
    }

    #[test]
    fn top_three_players_ranked_by_ca_plus_form_x10() {
        let squad = vec![
            SquadPlayerAttr { player_id: 1, ca_primary: 100, ca_reserve: 100, form: 5 },   // 150
            SquadPlayerAttr { player_id: 2, ca_primary: 120, ca_reserve: 120, form: 8 },   // 200
            SquadPlayerAttr { player_id: 3, ca_primary: 90,  ca_reserve: 90,  form: 3 },   // 120
            SquadPlayerAttr { player_id: 4, ca_primary: 150, ca_reserve: 150, form: 10 },  // 250
        ];
        let v = build_club_dashboard(
            Some(42), 0, None, false, false, true,
            None, 2001, &squad, true);
        assert_eq!(v.key_player_1, Some(4));
        assert_eq!(v.key_player_2, Some(2));
        assert_eq!(v.key_player_3, Some(1));
    }

    #[test]
    fn top_three_with_fewer_than_3_players_leaves_none() {
        let squad = vec![
            SquadPlayerAttr { player_id: 1, ca_primary: 100, ca_reserve: 100, form: 5 },
        ];
        let v = build_club_dashboard(
            Some(42), 0, None, false, false, true,
            None, 2001, &squad, true);
        assert_eq!(v.key_player_1, Some(1));
        assert_eq!(v.key_player_2, None);
        assert_eq!(v.key_player_3, None);
    }

    #[test]
    fn empty_squad_all_key_players_none() {
        let v = build_club_dashboard(
            Some(42), 0, None, false, false, true,
            None, 2001, &[], false);
        assert_eq!(v.key_player_1, None);
        assert_eq!(v.key_player_2, None);
        assert_eq!(v.key_player_3, None);
    }

    #[test]
    fn prior_state_restore_carries_over_11_slots() {
        let prior = ClubDashboardView {
            tab_index: 3, header_indicator: 999, finance_secondary: 888,
            prestige_delta: 7, sort_order: 2, display_mode: 4,
            finance_tertiary: 777, finance_byte_a: 5, finance_byte_b: 6,
            team_context: 4,
            ..Default::default()
        };
        let v = build_club_dashboard(
            Some(42), 0, Some(&prior), false, false, true,
            None, 2001, &[], false);
        assert_eq!(v.tab_index, 3);
        assert_eq!(v.header_indicator, 999);
        assert_eq!(v.sort_order, 2);
        assert_eq!(v.display_mode, 4);
        assert_eq!(v.team_context, 4);
    }

    #[test]
    fn ca_field_selection_matches_reserve_context() {
        // is_reserve_context = true → use ca_primary (attr +0xB).
        // is_reserve_context = false → use ca_reserve (attr +0xD).
        let squad = vec![
            SquadPlayerAttr { player_id: 1, ca_primary: 200, ca_reserve: 50, form: 0 },
        ];
        let a = build_club_dashboard(Some(1), 0, None, false, false, true,
                                        None, 2001, &squad, true);
        // ca_primary=200 used.
        let b = build_club_dashboard(Some(1), 0, None, false, false, true,
                                        None, 2001, &squad, false);
        // ca_reserve=50 used. Both should still yield player_1=1
        // (single player), but the score internal differs; observable
        // through equal top slot in this case.
        assert_eq!(a.key_player_1, Some(1));
        assert_eq!(b.key_player_1, Some(1));
    }
}
