//! Batch 22: 5 more setup functions — tactics/formation event handler
//! (stubbed), a small screen setup, a training-panel setup, and two
//! near-identical player-profile setup functions.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `0x0088A850.c` — NOT a screen setup: 2000+ line tactics/formation
//!   event dispatcher (DAT_00dbbf7c command bus + sidebar menu cascade).
//!   Stubbed with reason.
//! * `0x008A20A0.c` — screen setup (LAB_008A2180). Allocates 0x819-byte
//!   record via FUN_0089D100(param_1), pushes 6 slots
//!   {handle, 0, 0, 0, 1, 0xFFFFFFFF}.
//! * `0x008A6420.c` — training-panel screen setup (LAB_008A6580).
//!   `param_1 == 0` fires MsgBox from `training/` source path and
//!   aborts. Otherwise copies a 0x9B-byte record slice indexed by
//!   `param_1[0x7d9]` into a new heap block, then pushes 3 slots
//!   {param_1, record, record+9 (aux flag)}.
//! * `0x008D6310.c` — player-profile screen setup (FUN_008D6720).
//!   `param_2 == NULL || *param_2 == 0` disables the "has-manager" slot
//!   (slot 3 = 0xFFFFFFFF). Pushes many slots including profile-tab
//!   sentinels (0xF..0x16 = -1).
//! * `0x008D6510.c` — variant profile setup: gated on
//!   `param_2 != 0` (no NULL abort — falls through). Otherwise same
//!   view family as 008D6310.

use serde::{Deserialize, Serialize};

// =====================================================================
// 0x0088A850 — Tactics/formation event dispatcher — STUB
// =====================================================================

/// STUB: `FUN_0088A850` is NOT a screen setup.
///
/// The 2067-line decompile is an event dispatcher for the tactics UI
/// (formation load/save/lock/unlock, preset formations, "auto-select
/// tactic for opponent", etc.). It reads the global command bus
/// `DAT_00dbbf7c`/`DAT_00dbbf80`, and per sidebar menu item id (from
/// `DAT_00b59fe8[DAT_00b5d016*0xc0] + 0xba966 + item*0x18c`) dispatches
/// to file-picker helpers, formation loaders (`FUN_00895C10`), savers
/// (`FUN_00895D40`), lock/unlock, tactic-vs-opponent search, etc.
///
/// It does call `FUN_007E6570` (screen registration) in a couple of
/// arms, but only as leaf actions — the function itself is not a
/// build_* target. Deferred: needs full event-bus model + tactics DB.
pub const FUN_0088A850_STUB_REASON: &str =
    "event dispatcher, not screen setup — needs DAT_00dbbf7c command \
     bus + sidebar item lookup + tactics engine (FUN_00895C10/D40, \
     FUN_005A2910 file picker, FUN_008962F0 opponent-tactics search)";

// =====================================================================
// 0x008A20A0 — screen setup (LAB_008A2180)
// =====================================================================

/// Screen view built by `FUN_008A20A0(param_1)`. The exe allocates a
/// 0x819-byte record via `FUN_0089D100(param_1)` (heap fail → handle=0)
/// and pushes 6 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View008A20A0 {
    /// Slot 0: heap-record handle from `FUN_0089D100(param_1)`
    /// (`Some(id)` on success, `None` on allocation failure — but the
    /// exe still registers the screen and pushes 0 for the handle).
    pub record_handle: u32,
    /// Slots 1, 2, 3: literal 0.
    pub reserved: [u32; 3],
    /// Slot 4: literal 1.
    pub flag: u32,
    /// Slot 5: literal 0xFFFFFFFF (i.e. -1, "no selection" sentinel).
    pub selection: i32,
}

impl Default for View008A20A0 {
    fn default() -> Self {
        Self { record_handle: 0, reserved: [0; 3], flag: 1, selection: -1 }
    }
}

/// Direct port of `FUN_008A20A0(param_1)`.
///
/// * `registration_ok` mirrors `FUN_007E6570(&LAB_008A2180, ...) != 0`.
/// * `record_handle` is the value returned by `FUN_0089D100(param_1)`
///   (or 0 if operator_new(0x819) returned null).
pub fn build_view_008a20a0(
    registration_ok: bool,
    record_handle: u32,
) -> Option<View008A20A0> {
    if !registration_ok { return None; }
    Some(View008A20A0 {
        record_handle,
        reserved: [0; 3],
        flag: 1,
        selection: -1,
    })
}

// =====================================================================
// 0x008A6420 — training panel setup (LAB_008A6580)
// =====================================================================

/// Training-panel view built by `FUN_008A6420(param_1)`. Copies the
/// `param_1[0x7D9]`-selected 0x9B-byte record slice at offset
/// `0xFA + idx*0x9B` into a fresh heap block, then pushes 3 slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingPanelView {
    /// Slot 0: `param_1` (the parent object pointer).
    pub parent: u32,
    /// Slot 1: 0x9B-byte record copy (`puVar4`).
    pub record: Vec<u8>,
    /// Slot 2: pointer to `record + 9` written with `flag=1` (the
    /// 3rd arg to `FUN_007E7130(2, ptr, 1)` is the "as-flag" indicator).
    /// We model just that this aux entry exists.
    pub aux_flag_set: bool,
}

impl Default for TrainingPanelView {
    fn default() -> Self {
        Self { parent: 0, record: vec![0; 0x9B], aux_flag_set: true }
    }
}

/// Direct port of `FUN_008A6420(param_1)`.
///
/// * `param_1 == 0` fires the "training/" MsgBox and returns `None`
///   without registering.
/// * Otherwise copies the selected 0x9B-byte record slice and pushes
///   three slots.
pub fn build_training_panel(
    registration_ok: bool,
    param_1: u32,
    record_source: &[u8],
    record_index: u8,
) -> Option<TrainingPanelView> {
    if param_1 == 0 { return None; }
    if !registration_ok { return None; }
    // exe: puVar5 = param_1 + 0xFA + index*0x9B; memcpy 0x9B bytes
    let start = 0xFA + (record_index as usize) * 0x9B;
    let mut record = vec![0u8; 0x9B];
    if start + 0x9B <= record_source.len() {
        record.copy_from_slice(&record_source[start..start + 0x9B]);
    } // else: heap was allocated but source out-of-range — leave zeros
    Some(TrainingPanelView { parent: param_1, record, aux_flag_set: true })
}

// =====================================================================
// 0x008D6310 / 0x008D6510 — player profile screen setup (FUN_008D6720)
// =====================================================================

/// Player-profile screen view — the family built by both
/// `FUN_008D6310` and `FUN_008D6510`. Both register with the same
/// screen fn pair (FUN_008D6720/FUN_008DA210) and push the same 17-ish
/// slots; they only differ in how they source `param_1`/`param_2`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerProfileView {
    /// Slot 0: primary object pointer (`*param_1` for 008D6310,
    /// `*(*param_1 + 0xC)` for 008D6510).
    pub primary: u32,
    /// Slot 1: secondary pointer (`param_2` for 6310, `param_1` for 6510).
    pub secondary: u32,
    /// Slot 2: 0.
    pub reserved2: u32,
    /// Slot 3: manager reference id
    /// (`*(*param_2 + 0x24)` if param_2 valid, else 0xFFFFFFFF).
    pub manager_ref: i32,
    /// Slot 4: 0.
    pub reserved4: u32,
    /// Slot 5: 0.
    pub reserved5: u32,
    /// Slot 6: 1 for 008D6310, 0 for 008D6510. This flag distinguishes
    /// the two entry variants.
    pub variant_flag: u32,
    /// Slot 7: 1 if the linked person exists AND their type byte at
    /// +0x2C is >= 5 (and, for 6310, != 0x10); else 0.
    pub type_gated_flag: u32,
    /// Slot 8: club-tactic pointer (`FUN_00653270(iVar3,0,0,0)` in 6510;
    /// literal 0 in 6310 — the tactic is computed elsewhere).
    pub club_tactic: u32,
    /// Slot 9: boolean — `FUN_008BBDA0(...) != 0` — whether opposition
    /// tactics were resolvable.
    pub opposition_tactics_ok: bool,
    /// Slots 0xF..=0x16 (8 slots): profile-tab sentinels, all
    /// 0xFFFFFFFF ("no tab preselected").
    pub tab_sentinels: [i32; 8],
}

impl Default for PlayerProfileView {
    fn default() -> Self {
        Self {
            primary: 0, secondary: 0, reserved2: 0, manager_ref: -1,
            reserved4: 0, reserved5: 0, variant_flag: 0,
            type_gated_flag: 0, club_tactic: 0, opposition_tactics_ok: false,
            tab_sentinels: [-1; 8],
        }
    }
}

/// Direct port of `FUN_008D6310(param_1, param_2)`.
///
/// * `player` = `*param_1` (the person id).
/// * `person_ptr` = `param_2` (points to `*person_ptr = person object`).
/// * `person_type_byte` = `*(*param_2 + 0x2C)` if param_2 valid, else 0.
/// * `opposition_tactics_ok` matches
///   `FUN_008BBDA0(uVar3, &DAT_00acde90, param_1, iVar2, 1, 0) != 0`.
pub fn build_player_profile_6310(
    registration_ok: bool,
    player: u32,
    person_ptr: u32,
    person_deref: u32,        // *param_2 (0 if none)
    manager_ref: Option<u32>, // *(*param_2 + 0x24)
    person_type_byte: i8,
    opposition_tactics_ok: bool,
) -> Option<PlayerProfileView> {
    if !registration_ok { return None; }
    // exe: type-gated flag = person_ptr && *person_ptr && byte>=5 && byte!=0x10
    let type_gated = person_ptr != 0
        && person_deref != 0
        && person_type_byte >= 5
        && person_type_byte != 0x10;
    Some(PlayerProfileView {
        primary: player,
        secondary: person_ptr,
        reserved2: 0,
        manager_ref: manager_ref.map(|v| v as i32).unwrap_or(-1),
        reserved4: 0,
        reserved5: 0,
        variant_flag: 1,
        type_gated_flag: if type_gated { 1 } else { 0 },
        club_tactic: 0, // exe pushes literal 0 for slot 8 first
        opposition_tactics_ok,
        tab_sentinels: [-1; 8],
    })
}

/// Direct port of `FUN_008D6510(param_1, param_2)`.
///
/// * `param_2 == 0` short-circuits the `FUN_007E6570` call — the exe
///   pushes slots ANYWAY (the `||` short-circuits registration but the
///   subsequent slot pushes still run). We follow that: return `Some`
///   with `registration_ok=false` too, so long as caller confirmed the
///   short-circuit path.
pub fn build_player_profile_6510(
    registration_ok_or_shortcircuit: bool,
    club_id: u32,             // *(*param_1 + 0xC)
    person_ptr: u32,          // param_1
    manager_ref: u32,         // *(*param_1 + 0x24)
    person_type_byte: i8,     // *(*param_1 + 0x2C)
    club_tactic: u32,         // FUN_00653270(...)
    opposition_tactics_ok: bool,
) -> Option<PlayerProfileView> {
    if !registration_ok_or_shortcircuit { return None; }
    // exe 008D6510: type-gate is byte>=5 only (no !=0x10 guard).
    let type_gated = person_ptr != 0 && person_type_byte >= 5;
    Some(PlayerProfileView {
        primary: club_id,
        secondary: person_ptr,
        reserved2: 0,
        manager_ref: manager_ref as i32,
        reserved4: 0,
        reserved5: 0,
        variant_flag: 0,
        type_gated_flag: if type_gated { 1 } else { 0 },
        club_tactic,
        opposition_tactics_ok,
        tab_sentinels: [-1; 8],
    })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- 008A20A0 -----
    #[test]
    fn view_008a20a0_carries_handle_and_sentinels() {
        let v = build_view_008a20a0(true, 0x12345).unwrap();
        assert_eq!(v.record_handle, 0x12345);
        assert_eq!(v.reserved, [0; 3]);
        assert_eq!(v.flag, 1);
        assert_eq!(v.selection, -1);
    }
    #[test]
    fn view_008a20a0_registration_failure_none() {
        assert!(build_view_008a20a0(false, 42).is_none());
    }
    #[test]
    fn view_008a20a0_null_handle_still_registers() {
        // Heap failure → FUN_0089D100 returns 0, but registration still ran.
        let v = build_view_008a20a0(true, 0).unwrap();
        assert_eq!(v.record_handle, 0);
        assert_eq!(v.flag, 1);
    }

    // ----- 008A6420 -----
    #[test]
    fn training_panel_null_parent_returns_none() {
        assert!(build_training_panel(true, 0, &[0u8; 0x1000], 0).is_none());
    }
    #[test]
    fn training_panel_copies_indexed_slice() {
        let mut src = vec![0u8; 0xFA + 0x9B * 4];
        // Fill record #2 with a marker byte.
        let start = 0xFA + 2 * 0x9B;
        for i in 0..0x9B { src[start + i] = 0xAB; }
        let v = build_training_panel(true, 0xDEAD, &src, 2).unwrap();
        assert_eq!(v.parent, 0xDEAD);
        assert_eq!(v.record.len(), 0x9B);
        assert!(v.record.iter().all(|&b| b == 0xAB));
        assert!(v.aux_flag_set);
    }
    #[test]
    fn training_panel_registration_failure_none() {
        assert!(build_training_panel(false, 1, &[0u8; 0x1000], 0).is_none());
    }

    // ----- 008D6310 -----
    #[test]
    fn profile_6310_null_manager_ref_becomes_minus_one() {
        let v = build_player_profile_6310(true, 1, 0, 0, None, 0, false).unwrap();
        assert_eq!(v.manager_ref, -1);
        assert_eq!(v.type_gated_flag, 0);
        assert_eq!(v.variant_flag, 1);
        assert_eq!(v.tab_sentinels, [-1; 8]);
    }
    #[test]
    fn profile_6310_type_gate_excludes_0x10() {
        // byte >= 5 and != 0x10 → gate open
        let ok = build_player_profile_6310(true, 1, 0x100, 0x200, Some(9), 6, true).unwrap();
        assert_eq!(ok.type_gated_flag, 1);
        assert!(ok.opposition_tactics_ok);
        // byte == 0x10 → gate closed (this is the 6310-specific guard)
        let blocked = build_player_profile_6310(true, 1, 0x100, 0x200, Some(9), 0x10, false).unwrap();
        assert_eq!(blocked.type_gated_flag, 0);
    }
    #[test]
    fn profile_6310_registration_failure_none() {
        assert!(build_player_profile_6310(false, 1, 0, 0, None, 0, false).is_none());
    }

    // ----- 008D6510 -----
    #[test]
    fn profile_6510_variant_flag_is_zero() {
        let v = build_player_profile_6510(true, 5, 6, 7, 8, 9, true).unwrap();
        assert_eq!(v.variant_flag, 0);
        assert_eq!(v.primary, 5);
        assert_eq!(v.secondary, 6);
        assert_eq!(v.manager_ref, 7);
        assert_eq!(v.type_gated_flag, 1); // byte 8 >= 5, no 0x10 guard
        assert_eq!(v.club_tactic, 9);
    }
    #[test]
    fn profile_6510_type_gate_no_0x10_exclusion() {
        // 6510 lacks the != 0x10 check — byte 0x10 still opens the gate
        let v = build_player_profile_6510(true, 5, 6, 7, 0x10, 9, false).unwrap();
        assert_eq!(v.type_gated_flag, 1);
    }
    #[test]
    fn profile_6510_shortcircuit_gate_none() {
        assert!(build_player_profile_6510(false, 5, 6, 7, 8, 9, false).is_none());
    }

    // ----- 0088A850 stub -----
    #[test]
    fn stub_reason_documents_dispatcher_shape() {
        assert!(FUN_0088A850_STUB_REASON.contains("dispatcher"));
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b22: View008A20A0 reads handle `record_handle`; TrainingPanelView reads \
handle `parent`, buffer `record_source`, byte `record_index`; \
PlayerProfileView (both 6310 and 6510 flavours) reads a family of \
handles/bytes documented on `populate_player_profile_6310/_6510`. \
FUN_0088A850 stays a stub (see FUN_0088A850_STUB_REASON).\n\
UNKNOWNS: FUN_008BBDA0 opposition-tactics resolver + FUN_00653270 \
club-tactic pointer are not ported — populators expect flags/handles from \
the facade; production driver must set them from the wave-B tactics DB.";

use crate::world_facade::WorldFacade;

pub fn populate_view_008a20a0(world: &WorldFacade) -> Option<View008A20A0> {
    build_view_008a20a0(world.registration_ok, world.handle("record_handle"))
}

pub fn populate_training_panel(world: &WorldFacade) -> Option<TrainingPanelView> {
    build_training_panel(
        world.registration_ok,
        world.handle("parent"),
        world.buffer("record_source"),
        world.byte("record_index") as u8,
    )
}

pub fn populate_player_profile_6310(world: &WorldFacade) -> Option<PlayerProfileView> {
    let manager_ref = world.handles.get("manager_ref").copied();
    build_player_profile_6310(
        world.registration_ok,
        world.handle("player"),
        world.handle("person_ptr"),
        world.handle("person_deref"),
        manager_ref,
        world.byte("person_type_byte") as i8,
        world.flag("opposition_tactics_ok"),
    )
}

pub fn populate_player_profile_6510(world: &WorldFacade) -> Option<PlayerProfileView> {
    build_player_profile_6510(
        world.registration_ok,
        world.handle("club_id"),
        world.handle("person_ptr"),
        world.handle("manager_ref"),
        world.byte("person_type_byte") as i8,
        world.handle("club_tactic"),
        world.flag("opposition_tactics_ok"),
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_view_008a20a0_ok() {
        let w = WorldFacade::ready().with_handle("record_handle", 0x12345);
        assert_eq!(populate_view_008a20a0(&w).unwrap().record_handle, 0x12345);
    }
    #[test]
    fn populate_training_panel_null_parent_none() {
        assert!(populate_training_panel(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_training_panel_ok() {
        let w = WorldFacade::ready()
            .with_handle("parent", 0xDEAD)
            .with_buffer("record_source", vec![0u8; 0x1000]);
        let v = populate_training_panel(&w).unwrap();
        assert_eq!(v.parent, 0xDEAD);
        assert_eq!(v.record.len(), 0x9B);
    }
    #[test]
    fn populate_player_profile_6310_defaults_manager_ref_minus_one() {
        let v = populate_player_profile_6310(&WorldFacade::ready()).unwrap();
        assert_eq!(v.manager_ref, -1);
        assert_eq!(v.variant_flag, 1);
    }
    #[test]
    fn populate_player_profile_6510_variant_flag_zero() {
        let v = populate_player_profile_6510(&WorldFacade::ready()).unwrap();
        assert_eq!(v.variant_flag, 0);
    }
}
