//! 4-screen batch: retire, send_abuse, compare_players, add_manager.
//!
//! Decompiles under `d:/cm0102-carve/decompiled/screen_batch4/`:
//! * `0x00698140.c` — Retire dialog (cmd 0x3F2, 27 B)
//! * `0x00771810.c` — Send Abuse (cmd 0x414, 109 B)
//! * `0x007DFAC0.c` — Compare-players slot lookup (cmd 0x434, 249 B)
//! * `0x007E7790.c` — Add Manager add-to-seat (cmd 0x3FB, 141 B)

use serde::{Deserialize, Serialize};

// =====================================================================
// Retire dialog — cmd 0x3F2 → FUN_00698140 (27 B)
// =====================================================================

/// Retire dialog handle — the exe's `FUN_00698140` just calls
/// `FUN_007E6570(&LAB_00698160, &LAB_00698480, 1, 0, 0)` with no
/// data-slot pushes. Symmetric with go_holiday_dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RetireDialog { pub opened: bool }

/// Direct port of `FUN_00698140`.
pub fn build_retire_dialog(registration_ok: bool) -> RetireDialog {
    RetireDialog { opened: registration_ok }
}

// =====================================================================
// Send Abuse — cmd 0x414 → FUN_00771810(recipient, note)
// =====================================================================

/// Send-Abuse dialog view — 3 slots.
///
/// The exe allocates a 0x65 (101) byte scratch buffer for the message
/// body via `operator_new`, then calls `FUN_007E6EE0(1)` to get its
/// pointer back and zeros the first byte (empty message).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendAbuseView {
    /// Slot 0: recipient manager id (`param_1`). `None` = "all mgrs".
    pub recipient_id: Option<u32>,
    /// Slot 1: 101-byte message body scratch, initialised to empty.
    pub message_body: Vec<u8>,
    /// Slot 2: associated note/context (`param_2`).
    pub note: u32,
}

impl Default for SendAbuseView {
    fn default() -> Self {
        Self { recipient_id: None, message_body: vec![0; 0x65], note: 0 }
    }
}

/// Direct port of `FUN_00771810(param_1, param_2)`. `param_1 == -1`
/// (0xFFFFFFFF) is the "send to all" sentinel.
pub fn build_send_abuse(
    registration_ok: bool,
    param_1_recipient: u32,
    param_2_note: u32,
) -> Option<SendAbuseView> {
    if !registration_ok { return None; }
    let recipient_id = if param_1_recipient == 0xFFFFFFFF { None }
                       else { Some(param_1_recipient) };
    let mut body = vec![0u8; 0x65];
    body[0] = 0;   // exe: `*puVar3 = 0` — empty string terminator.
    Some(SendAbuseView { recipient_id, message_body: body, note: param_2_note })
}

// =====================================================================
// Compare Players helper — cmd 0x434 → FUN_007DFAC0(mode 1/2)
// =====================================================================

/// Compare-Players slot lookup — this isn't a screen itself, but the
/// exe calls it twice with mode=1 and mode=2 to grab the current
/// human's two "shortlist" player slots, then hands them to the
/// screen setup.
///
/// Exe reads:
/// * If current-human ptr == null → error, return 0
/// * `cVar1 = (human[0] - DAT_00ACD56C) + 0x10` — normalise into
///   0..0x10 window
/// * If `cVar1 >= 0 && cVar1 < 0x10`:
///   * mode == 1 → return `in_ECX + 0x68 + cVar1*4` (first-player table)
///   * mode == 2 → return `in_ECX + 0xA8 + cVar1*4` (second-player table)
/// * Else → error, return 0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareSlotMode { First = 1, Second = 2 }

/// Direct port of `FUN_007DFAC0(mode)`. Caller supplies human's
/// person id and `DAT_00ACD56C` (staff pool max). Returns the byte
/// offset into a per-human table (0x68 base for slot A, 0xA8 for
/// slot B), OR `None` on the two error paths (null human, window
/// overflow).
pub fn compare_players_slot(
    current_human_person_id: Option<i32>,
    staff_pool_max: i32,
    mode: CompareSlotMode,
) -> Option<u32> {
    let id = current_human_person_id?;
    let normalised = (id - staff_pool_max) + 0x10;
    if !(0..0x10).contains(&normalised) { return None; }
    let base = match mode {
        CompareSlotMode::First  => 0x68,
        CompareSlotMode::Second => 0xA8,
    };
    Some(base + (normalised as u32) * 4)
}

// =====================================================================
// Add Manager — cmd 0x3FB → FUN_007E7790(callback_a, callback_b, dialog_state)
// =====================================================================

/// Add Manager view — write-through to the human-seat globals at
/// `+0x261FCC`, `+0x261FD0` (callback slots) plus per-manager state
/// at `human_seat[cur].+0x32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AddManagerState {
    /// `+0x261FCC` — primary callback pointer (opaque).
    pub callback_a: u32,
    /// `+0x261FD0` — secondary callback / context pointer.
    pub callback_b: u32,
    /// Per-seat `+0x32` value written when `dialog_state != 0`.
    pub per_seat_state: Option<u32>,
}

/// Direct port of `FUN_007E7790(param_1, param_2, param_3)`.
///
/// * `param_3 != 0` (dialog opening): writes callback_a=0 first,
///   then callback_b=param_2, refreshes the seat via FUN_007E74E0,
///   writes seat's `+0x32 = param_3`, then writes callback_a=param_1.
/// * `param_3 == 0` (dialog closing): writes callback_b=param_2,
///   callback_a=param_1, refreshes seat.
///
/// Both paths end with the same state (both callbacks + optional
/// seat state); the order is what the exe cares about for the
/// refresh-during-write window.
pub fn build_add_manager_state(
    param_1: u32, param_2: u32, param_3: u32,
) -> AddManagerState {
    if param_3 != 0 {
        AddManagerState {
            callback_a: param_1,
            callback_b: param_2,
            per_seat_state: Some(param_3),
        }
    } else {
        AddManagerState {
            callback_a: param_1,
            callback_b: param_2,
            per_seat_state: None,
        }
    }
}

// =====================================================================
// World-populator companions
// =====================================================================

use crate::world_pools::WorldPools;

/// Populate a Retire dialog — no data slots (see `build_retire_dialog`).
pub fn populate_retire(_pools: &WorldPools<'_>) -> RetireDialog {
    build_retire_dialog(true)
}

/// Populate a Send-Abuse view. With no recipient chosen the exe uses
/// the -1 sentinel ("all managers"); the message body starts empty.
///
/// The exe reads the recipient from a context menu, not from the world
/// — so the world facade only witnesses the "no recipient yet" default.
pub fn populate_send_abuse(_pools: &WorldPools<'_>) -> Option<SendAbuseView> {
    build_send_abuse(true, 0xFFFFFFFF, 0)
}

/// Populate a Compare-Players slot lookup for the active human's
/// primary shortlist slot. Returns `None` when no human is seated or
/// when the seat is outside the compare window (0..0x10 above
/// `DAT_00ACD56C`, the staff-pool max).
///
/// The staff-pool max is inferred from `pools.staff.len()` as `staff.len()
/// as i32` — the exe's `DAT_00ACD56C` is exactly the pool size.
pub fn populate_compare_slot(
    pools: &WorldPools<'_>,
    mode: CompareSlotMode,
) -> Option<u32> {
    let sid = pools.active_human_seat? as i32;
    let staff_pool_max = pools.staff.len() as i32;
    compare_players_slot(Some(sid), staff_pool_max, mode)
}

/// Populate an Add-Manager state for the "dialog opening" transition
/// (param_3 = 1, matching the exe's typical dialog-boot path). Callback
/// pointers come from the exe's global registration table
/// (`DAT_00261FCC`/`DAT_00261FD0`) and are not decoded on the pool
/// facade yet — see `TODO_POPULATOR_INFO` below.
pub fn populate_add_manager(_pools: &WorldPools<'_>) -> AddManagerState {
    build_add_manager_state(0, 0, 1)
}

#[cfg(test)]
#[allow(dead_code)]
const TODO_POPULATOR_INFO: &str = "\
    AddManagerState.callback_{a,b} need the two exe globals at \
    DAT_00261FCC/DAT_00261FD0 (registration slots). SendAbuseView's real \
    recipient comes from a right-click context, not the world pool — the \
    populator here only sets the 'all managers' default.";

#[cfg(test)]
mod tests {
    use super::*;

    // Retire
    #[test]
    fn retire_opens_on_success() {
        assert!(build_retire_dialog(true).opened);
        assert!(!build_retire_dialog(false).opened);
    }

    // Send Abuse
    #[test]
    fn send_abuse_specific_recipient() {
        let v = build_send_abuse(true, 42, 100).unwrap();
        assert_eq!(v.recipient_id, Some(42));
        assert_eq!(v.note, 100);
        assert_eq!(v.message_body.len(), 0x65);
    }
    #[test]
    fn send_abuse_sentinel_recipient_means_all() {
        let v = build_send_abuse(true, 0xFFFFFFFF, 0).unwrap();
        assert_eq!(v.recipient_id, None, "-1 recipient sends to all");
    }
    #[test]
    fn send_abuse_body_is_101_bytes_and_starts_null() {
        let v = build_send_abuse(true, 0, 0).unwrap();
        assert_eq!(v.message_body.len(), 0x65);
        assert_eq!(v.message_body[0], 0);
    }
    #[test]
    fn send_abuse_failed_registration_returns_none() {
        assert!(build_send_abuse(false, 42, 0).is_none());
    }

    // Compare Players
    #[test]
    fn compare_null_human_returns_none() {
        assert!(compare_players_slot(None, 100, CompareSlotMode::First).is_none());
    }
    #[test]
    fn compare_within_window_returns_offset() {
        // human_id = 100, staff_pool_max = 100 → normalised = 0x10.
        // But 0x10 is NOT < 0x10, so should be None. Try 95 → 11 → valid.
        let r = compare_players_slot(Some(95), 100, CompareSlotMode::First);
        assert_eq!(r, Some(0x68 + 11 * 4));
    }
    #[test]
    fn compare_second_slot_uses_0xa8_base() {
        let r = compare_players_slot(Some(95), 100, CompareSlotMode::Second);
        assert_eq!(r, Some(0xA8 + 11 * 4));
    }
    #[test]
    fn compare_out_of_window_returns_none() {
        // Way out: id 50 with max 100 → normalised -34, invalid.
        assert!(compare_players_slot(Some(50), 100, CompareSlotMode::First).is_none());
        // At boundary: id 84 → normalised 0 (valid).
        assert!(compare_players_slot(Some(84), 100, CompareSlotMode::First).is_some());
        // Beyond: id 100 → normalised 16 (0x10), NOT valid (< 0x10).
        assert!(compare_players_slot(Some(100), 100, CompareSlotMode::First).is_none());
    }

    // Add Manager
    #[test]
    fn add_manager_dialog_opening_stores_seat_state() {
        let s = build_add_manager_state(0xABCD, 0x1234, 5);
        assert_eq!(s.callback_a, 0xABCD);
        assert_eq!(s.callback_b, 0x1234);
        assert_eq!(s.per_seat_state, Some(5));
    }
    #[test]
    fn add_manager_dialog_closing_clears_seat_state() {
        let s = build_add_manager_state(0xABCD, 0x1234, 0);
        assert_eq!(s.callback_a, 0xABCD);
        assert_eq!(s.callback_b, 0x1234);
        assert_eq!(s.per_seat_state, None);
    }

    // --- Populators ---
    use crate::world_pools::WorldPools;
    use crate::DomainStaffType6;

    #[test]
    fn populate_retire_opens() {
        assert!(populate_retire(&WorldPools::empty()).opened);
    }

    #[test]
    fn populate_send_abuse_defaults_to_all_recipients() {
        let v = populate_send_abuse(&WorldPools::empty()).unwrap();
        assert_eq!(v.recipient_id, None);
        assert_eq!(v.message_body.len(), 0x65);
        assert_eq!(v.message_body[0], 0);
    }

    #[test]
    fn populate_compare_slot_uses_seat_and_pool_max() {
        // Pool size 100, seat 95 → normalised 11 → 0x68 + 11*4.
        let staff: Vec<DomainStaffType6> = (0..100).map(|i| DomainStaffType6 {
            id: i as u32, body: vec![0u8; 0x60],
        }).collect();
        let pools = WorldPools {
            staff: &staff, active_human_seat: Some(95),
            ..WorldPools::empty()
        };
        assert_eq!(populate_compare_slot(&pools, CompareSlotMode::First), Some(0x68 + 11 * 4));
    }

    #[test]
    fn populate_compare_slot_no_seat_yields_none() {
        assert!(populate_compare_slot(&WorldPools::empty(), CompareSlotMode::First).is_none());
    }

    #[test]
    fn populate_add_manager_defaults_open_state() {
        let s = populate_add_manager(&WorldPools::empty());
        assert_eq!(s.per_seat_state, Some(1));
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A `_from_pools`)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b4 pool sources: RetireDialog + AddManagerState are stateless \
    (callback ids DAT_00261FCC/DAT_00261FD0 not on the pools facade). \
    SendAbuseView recipient comes from a right-click context, not from \
    any pool. CompareSlot uses seat + staff.len() as the pool max, \
    matching the exe's DAT_00ACD56C = staff-pool size.";

pub fn populate_retire_from_pools(pools: &WorldPools<'_>) -> RetireDialog {
    populate_retire(pools)
}

pub fn populate_send_abuse_from_pools(pools: &WorldPools<'_>) -> Option<SendAbuseView> {
    populate_send_abuse(pools)
}

pub fn populate_compare_slot_from_pools(
    pools: &WorldPools<'_>,
    mode: CompareSlotMode,
) -> Option<u32> {
    populate_compare_slot(pools, mode)
}

pub fn populate_add_manager_from_pools(pools: &WorldPools<'_>) -> AddManagerState {
    populate_add_manager(pools)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn retire_opens_on_empty_pools() {
        assert!(populate_retire_from_pools(&WorldPools::empty()).opened);
    }

    #[test]
    fn send_abuse_defaults_to_all_recipients() {
        let v = populate_send_abuse_from_pools(&WorldPools::empty()).unwrap();
        assert_eq!(v.recipient_id, None);
        assert_eq!(v.message_body.len(), 0x65);
    }

    #[test]
    fn compare_slot_no_seat_yields_none() {
        assert!(
            populate_compare_slot_from_pools(&WorldPools::empty(), CompareSlotMode::First)
                .is_none()
        );
    }

    #[test]
    fn compare_slot_uses_seat_and_pool_max() {
        let staff: Vec<crate::DomainStaffType6> = (0..100)
            .map(|i| crate::DomainStaffType6 { id: i as u32, body: vec![0u8; 0x60] })
            .collect();
        let pools = WorldPools {
            staff: &staff,
            active_human_seat: Some(95),
            ..WorldPools::empty()
        };
        assert_eq!(
            populate_compare_slot_from_pools(&pools, CompareSlotMode::First),
            Some(0x68 + 11 * 4)
        );
    }

    #[test]
    fn add_manager_defaults_open_state() {
        let s = populate_add_manager_from_pools(&WorldPools::empty());
        assert_eq!(s.per_seat_state, Some(1));
    }
}
