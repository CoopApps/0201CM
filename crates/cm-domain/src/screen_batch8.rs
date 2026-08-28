//! Batch 8: 5 more setup functions — nations/major_clubs/league list,
//! find dialog, written history, selected leagues screen, hall of fame.
//!
//! Decompiles: `d:/cm0102-carve/decompiled/screen_batch8/`.
//! * `0x0058A550.c` — Entity List (cmd 0x3F4/6/7/etc, 486 B) —
//!   nations/major-clubs/league list, mode selects which
//! * `0x0058CDE0.c` — Find dialog (cmd 0x3FA, 544 B) — 4 search
//!   fields each with a 25-byte scratch buffer
//! * `0x005DC5E0.c` — Written History (cmd 0x429, 117 B, 5 slots)
//! * `0x008053D0.c` — Selected Leagues setup (cmd 0x431, 515 B) — the
//!   Setup screen (post-initial-load only)
//! * `0x0080FAC0.c` — Hall of Fame (cmd 0x42A, 79 B, 3 slots)

use serde::{Deserialize, Serialize};

// =====================================================================
// Entity List — cmd 0x3F4/6/7 → FUN_0058A550(mode, focus, extra, filter)
// =====================================================================

/// Entity List view — Nations/Major Clubs/League/etc. list screen.
///
/// `mode` selects which entity family (1=Nations, 3=Major Clubs,
/// 4=League, 5=Non-League, 6=Other Clubs, 7=Under 21s).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityListView {
    /// Slot 0: entity-family mode byte.
    pub mode: i8,
    /// Slot 1, 4, 7, 0x10: focus entity (repeated 4 times in slot writes).
    pub focus: u32,
    /// Slot 10, 0xD: additional filter (param_3).
    pub extra_filter: u32,
    /// Slot 0x13: filter byte (param_4).
    pub filter_byte: i8,
}

impl Default for EntityListView {
    fn default() -> Self { Self { mode: 0, focus: 0, extra_filter: 0, filter_byte: 0 } }
}

/// Direct port of `FUN_0058A550(mode, focus, extra, filter)`. `focus
/// == 0` fires MsgBox `find_screens:0x7A`, but the exe *continues*
/// pushing slots anyway. We return `None` only if registration failed.
pub fn build_entity_list(
    registration_ok: bool,
    mode: i8, focus: u32, extra: u32, filter: i8,
) -> Option<EntityListView> {
    if !registration_ok { return None; }
    Some(EntityListView { mode, focus, extra_filter: extra, filter_byte: filter })
}

// =====================================================================
// Find dialog — cmd 0x3FA → FUN_0058CDE0(mode)
// =====================================================================

/// Find dialog view — 4 search-field scratch buffers, each 25 bytes
/// initialised to spaces + null terminator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindDialogView {
    /// Slot 0: mode byte (0 = default, non-zero = specific search).
    pub mode: i8,
    /// Slots 1, 7, 0xD, 0x13: 4 search-field scratch buffers (each
    /// 25 bytes = 6 dwords of 0x20 + trailing 0 byte). Init'd to 0x20
    /// then zeroed at [0]/[7]/[0xD]/[0x13] via `FUN_007E6EE0(n)`.
    pub search_field: [Vec<u8>; 4],
}

impl Default for FindDialogView {
    fn default() -> Self {
        let mk = || {
            let mut b = vec![b' '; 25];
            b[0] = 0;      // exe: FUN_007E6EE0(n) then *ptr = 0
            b
        };
        Self { mode: 0, search_field: [mk(), mk(), mk(), mk()] }
    }
}

/// Direct port of `FUN_0058CDE0(mode)`.
pub fn build_find_dialog(
    registration_ok: bool,
    mode: i8,
) -> Option<FindDialogView> {
    if !registration_ok { return None; }
    Some(FindDialogView { mode, ..Default::default() })
}

// =====================================================================
// Written History — cmd 0x429 → FUN_005DC5E0(mode)
// =====================================================================

/// Written History view — 5 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrittenHistoryView {
    /// Slot 0: history category (param_1 — matches `HistoryCategory` enum).
    pub category: u32,
    /// Slots 1..=4: zero-init.
    pub reserved: [u32; 4],
}

impl Default for WrittenHistoryView {
    fn default() -> Self { Self { category: 0, reserved: [0; 4] } }
}

/// Direct port of `FUN_005DC5E0(param_1)`.
pub fn build_written_history(
    registration_ok: bool,
    category: u32,
) -> Option<WrittenHistoryView> {
    if !registration_ok { return None; }
    Some(WrittenHistoryView { category, ..Default::default() })
}

// =====================================================================
// Selected Leagues — cmd 0x431 → FUN_008053D0
// =====================================================================

/// Selected-Leagues setup view — 2 slots + big first-time init cascade.
///
/// The exe has a `DAT_00ACD5B0 == 0` branch that runs the full setup
/// pipeline (FUN_00789A40 loader gate, FUN_008FB240 CM3 DATA path,
/// FUN_0050E9B0 config load, FUN_005C2180 UI init, FUN_00811D80 human
/// init, FUN_006508E0 world init). We model that with `first_time_init`
/// and `init_ok` gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedLeaguesView {
    /// Slot 0: 0 (initial-selection cursor).
    pub cursor: u32,
    /// Slot 1: 0 (scroll offset).
    pub scroll_offset: u32,
    /// Whether the exe set `DAT_00A01714 = 1` (config-dirty flag) —
    /// only set on successful registration.
    pub config_dirty_flag: bool,
}

impl Default for SelectedLeaguesView {
    fn default() -> Self { Self { cursor: 0, scroll_offset: 0, config_dirty_flag: false } }
}

/// Direct port of `FUN_008053D0`.
///
/// * `first_time_init` matches exe's `DAT_00ACD5B0 == 0` gate.
/// * `pipeline_ok` matches the full setup pipeline (fires exe's
///   FUN_009349C4(-1) noreturn on failure — we return `None`).
pub fn build_selected_leagues(
    registration_ok: bool,
    first_time_init: bool,
    pipeline_ok: bool,
) -> Option<SelectedLeaguesView> {
    if first_time_init && !pipeline_ok { return None; }
    if !registration_ok { return None; }
    Some(SelectedLeaguesView {
        cursor: 0, scroll_offset: 0, config_dirty_flag: true,
    })
}

// =====================================================================
// Hall of Fame — cmd 0x42A → FUN_0080FAC0 (79 B, 3 slots)
// =====================================================================

/// Hall of Fame view — 3 slots (0, 1, 2), all zero-init.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HallOfFameView {
    pub selected_tab: u32,
    pub scroll_offset: u32,
    pub filter: u32,
}

/// Direct port of `FUN_0080FAC0`.
pub fn build_hall_of_fame(registration_ok: bool) -> Option<HallOfFameView> {
    if !registration_ok { return None; }
    Some(HallOfFameView::default())
}

// =====================================================================
// World-populator companions
// =====================================================================

use crate::world_pools::WorldPools;

/// Populate an Entity-List view (Nations by default: mode 1) focused on
/// the active human's nation. `focus == 0` uses the first nation in the
/// pool as a fallback — matches the exe's "MsgBox but continue" path.
pub fn populate_entity_list(pools: &WorldPools<'_>) -> Option<EntityListView> {
    let focus = pools.nations.first().map(|n| n.id).unwrap_or(0);
    build_entity_list(true, 1, focus, 0, 0)
}

/// Populate a Find dialog in the default (mode 0) search state — 4
/// empty scratch buffers, matches `FUN_0058CDE0(0)`.
pub fn populate_find_dialog(_pools: &WorldPools<'_>) -> Option<FindDialogView> {
    build_find_dialog(true, 0)
}

/// Populate a Written-History view. Category 0 = default. No pool slot
/// names the picker — the user selects it from the sidebar.
pub fn populate_written_history(_pools: &WorldPools<'_>) -> Option<WrittenHistoryView> {
    build_written_history(true, 0)
}

/// Populate a Selected-Leagues setup view. The setup screen runs before
/// any world exists, so `first_time_init=false` (the pipeline was
/// already run by the New-Game boot path when this facade has any pools).
pub fn populate_selected_leagues(pools: &WorldPools<'_>) -> Option<SelectedLeaguesView> {
    let first_time_init = pools.comps.is_empty() && pools.nations.is_empty();
    build_selected_leagues(true, first_time_init, true)
}

/// Populate a Hall-of-Fame view — all defaults.
pub fn populate_hall_of_fame(_pools: &WorldPools<'_>) -> Option<HallOfFameView> {
    build_hall_of_fame(true)
}

#[cfg(test)]
#[allow(dead_code)]
const TODO_POPULATOR_INFO: &str = "\
    EntityListView.focus should use the active human's nation id \
    (via active_human_club_id -> club+0x27); currently uses first \
    nation in the pool. SelectedLeaguesView's real 'first-time init' \
    predicate is DAT_00ACD5B0 == 0 (not on the facade); we approximate \
    with 'no pools loaded'.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_list_carries_all_params() {
        let v = build_entity_list(true, 3, 42, 100, 7).unwrap();
        assert_eq!(v.mode, 3);            // Major Clubs
        assert_eq!(v.focus, 42);
        assert_eq!(v.extra_filter, 100);
        assert_eq!(v.filter_byte, 7);
    }
    #[test]
    fn entity_list_failed_registration_returns_none() {
        assert!(build_entity_list(false, 3, 42, 0, 0).is_none());
    }
    #[test]
    fn entity_list_mode_matches_sidebar_dispatch_values() {
        // From dispatcher: 0x3F4→mode 1, 0x3F6→3, 0x3F7→4.
        assert_eq!(build_entity_list(true, 1, 0, 0, 0).unwrap().mode, 1);
        assert_eq!(build_entity_list(true, 3, 0, 0, 0).unwrap().mode, 3);
        assert_eq!(build_entity_list(true, 4, 0, 0, 0).unwrap().mode, 4);
    }

    #[test]
    fn find_dialog_has_4_search_scratch_buffers() {
        let v = build_find_dialog(true, 0).unwrap();
        assert_eq!(v.search_field.len(), 4);
        // Each is 25 bytes, first byte zeroed (null-terminated empty string).
        for buf in &v.search_field {
            assert_eq!(buf.len(), 25);
            assert_eq!(buf[0], 0);
        }
    }
    #[test]
    fn find_dialog_mode_carried() {
        let v = build_find_dialog(true, 5).unwrap();
        assert_eq!(v.mode, 5);
    }

    #[test]
    fn written_history_category_carried() {
        let v = build_written_history(true, 3).unwrap();
        assert_eq!(v.category, 3);
        assert_eq!(v.reserved, [0; 4]);
    }

    #[test]
    fn selected_leagues_first_time_init_failure_returns_none() {
        assert!(build_selected_leagues(true, true, false).is_none());
    }
    #[test]
    fn selected_leagues_normal_flow_sets_config_dirty() {
        let v = build_selected_leagues(true, false, true).unwrap();
        assert!(v.config_dirty_flag);
    }
    #[test]
    fn selected_leagues_first_time_ok_sets_config_dirty() {
        let v = build_selected_leagues(true, true, true).unwrap();
        assert!(v.config_dirty_flag);
    }

    #[test]
    fn hall_of_fame_defaults_all_zero() {
        let v = build_hall_of_fame(true).unwrap();
        assert_eq!(v.selected_tab, 0);
        assert_eq!(v.scroll_offset, 0);
        assert_eq!(v.filter, 0);
    }
    #[test]
    fn hall_of_fame_failed_registration_returns_none() {
        assert!(build_hall_of_fame(false).is_none());
    }

    // --- Populators ---
    use crate::world_pools::WorldPools;
    use crate::DomainOpaqueRecord;

    #[test]
    fn populate_entity_list_uses_first_nation() {
        let nations = vec![DomainOpaqueRecord {
            ordinal: 0, id: 99, primary_name: Some("Foo".into()),
            secondary_name: None, short_name: None,
            text_candidates: vec![], raw: vec![],
        }];
        let pools = WorldPools { nations: &nations, ..WorldPools::empty() };
        let v = populate_entity_list(&pools).unwrap();
        assert_eq!(v.mode, 1);
        assert_eq!(v.focus, 99);
    }

    #[test]
    fn populate_entity_list_empty_pools_uses_zero_focus() {
        let v = populate_entity_list(&WorldPools::empty()).unwrap();
        assert_eq!(v.focus, 0);
    }

    #[test]
    fn populate_find_dialog_defaults() {
        let v = populate_find_dialog(&WorldPools::empty()).unwrap();
        assert_eq!(v.mode, 0);
        assert_eq!(v.search_field.len(), 4);
    }

    #[test]
    fn populate_written_history_defaults() {
        let v = populate_written_history(&WorldPools::empty()).unwrap();
        assert_eq!(v.category, 0);
    }

    #[test]
    fn populate_selected_leagues_sets_config_dirty() {
        let v = populate_selected_leagues(&WorldPools::empty()).unwrap();
        assert!(v.config_dirty_flag);
    }

    #[test]
    fn populate_hall_of_fame_defaults() {
        let v = populate_hall_of_fame(&WorldPools::empty()).unwrap();
        assert_eq!(v.selected_tab, 0);
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A `_from_pools`)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b8 pool sources: EntityListView.focus should use the active human's \
    nation id (via active_human_club_id -> club+0x27); currently uses \
    the first nation in the pool. SelectedLeaguesView 'first-time init' \
    predicate is DAT_00ACD5B0 == 0 (not on the pools facade); we \
    approximate it as 'no pools loaded'. FindDialog/WrittenHistory/ \
    HallOfFame have no per-caller pool state.";

pub fn populate_entity_list_from_pools(pools: &WorldPools<'_>) -> Option<EntityListView> {
    populate_entity_list(pools)
}

pub fn populate_find_dialog_from_pools(pools: &WorldPools<'_>) -> Option<FindDialogView> {
    populate_find_dialog(pools)
}

pub fn populate_written_history_from_pools(pools: &WorldPools<'_>) -> Option<WrittenHistoryView> {
    populate_written_history(pools)
}

pub fn populate_selected_leagues_from_pools(pools: &WorldPools<'_>) -> Option<SelectedLeaguesView> {
    populate_selected_leagues(pools)
}

pub fn populate_hall_of_fame_from_pools(pools: &WorldPools<'_>) -> Option<HallOfFameView> {
    populate_hall_of_fame(pools)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;
    use crate::DomainOpaqueRecord;

    #[test]
    fn entity_list_uses_first_nation() {
        let nations = vec![DomainOpaqueRecord {
            ordinal: 0, id: 99, primary_name: Some("Foo".into()),
            secondary_name: None, short_name: None,
            text_candidates: vec![], raw: vec![],
        }];
        let pools = WorldPools { nations: &nations, ..WorldPools::empty() };
        let v = populate_entity_list_from_pools(&pools).unwrap();
        assert_eq!(v.focus, 99);
        assert_eq!(v.mode, 1);
    }

    #[test]
    fn entity_list_empty_pools_zero_focus() {
        let v = populate_entity_list_from_pools(&WorldPools::empty()).unwrap();
        assert_eq!(v.focus, 0);
    }

    #[test]
    fn find_dialog_defaults() {
        let v = populate_find_dialog_from_pools(&WorldPools::empty()).unwrap();
        assert_eq!(v.mode, 0);
        assert_eq!(v.search_field.len(), 4);
    }

    #[test]
    fn written_history_defaults() {
        let v = populate_written_history_from_pools(&WorldPools::empty()).unwrap();
        assert_eq!(v.category, 0);
    }

    #[test]
    fn selected_leagues_sets_config_dirty() {
        let v = populate_selected_leagues_from_pools(&WorldPools::empty()).unwrap();
        assert!(v.config_dirty_flag);
    }

    #[test]
    fn hall_of_fame_defaults() {
        let v = populate_hall_of_fame_from_pools(&WorldPools::empty()).unwrap();
        assert_eq!(v.selected_tab, 0);
        assert_eq!(v.filter, 0);
    }
}
