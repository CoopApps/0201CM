//! Batch 19: 5 more setup functions ported from cm0102.exe.
//!
//! Decompiles live at `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//!
//! * `0x007CDB80.c` — Person/Squad-type banner setup (3 slots): a
//!   record + a category byte derived from `FUN_00618410(record)`
//!   (mapped via two switch tables) + `**(record+0x39)`.
//! * `0x007E7820.c` — **NOT a screen setup**: this is the widget-pool
//!   pop/close helper (`DAT_00B5CFF6[]` slot compaction, resets the
//!   pool on the last close). Stubbed here for completeness.
//! * `0x007FAEC0.c` — Search-result detail setup (3 slots): input
//!   pointer, a newly-allocated 0x99-byte sub-record copy taken from
//!   `param_1 + record[0]*0x2B0 + 0x1FB`, and a status byte at
//!   `+0x2A4`. Errors out (via MsgBox) if `param_1 == NULL`.
//! * `0x00803E00.c` — Setup / QSTART bootstrap (1 slot): checks the
//!   `CM3_QSTART` config, does the seed/nonetwork resource pulls, then
//!   registers a screen whose slot 0 is `*(DAT_00ACDF28 + 0x213) == 0`
//!   (a "no-network / offline" boolean). Ends by handing off to the
//!   0x00808A70 persistent-registration path.
//! * `0x00808A70.c` — 5-slot all-zero screen (persistent handoff target
//!   from the setup bootstrap).

use serde::{Deserialize, Serialize};

// =====================================================================
// 0x007CDB80 — Person/Squad-type banner (3 slots)
// =====================================================================

/// Person/Squad-type banner view — 3 slots.
///
/// * Slot 0: the first dword of the input record (`*param_1`).
/// * Slot 1: derived category byte — when `mode == 0` the exe reads
///   `FUN_00618410(record)` and remaps via two switch tables (0x4E for
///   the first group, 0x4F for the second, 0x4D default); when
///   `mode != 0` the byte is hard-coded to `0x51`.
/// * Slot 2: `**(record + 0x39)` — first byte at the pointer stored at
///   offset 0x39 in the record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PersonBannerView {
    pub record_head: u32,
    pub category_byte: u8,
    pub sub_head_byte: u8,
}

/// Direct port of `FUN_007CDB80(record, mode)`.
///
/// * `registration_ok` matches `FUN_007E6570(...) != 0`.
/// * `record_present` matches the exe's `param_1 != NULL &&
///   *(int*)(param_1 + 0x39) != NULL` gate — both branches fall through
///   with no widget slots pushed on failure, so we return `None`.
/// * `raw_kind` is the value of `FUN_00618410(record)`.
pub fn build_person_banner(
    registration_ok: bool,
    record_present: bool,
    record_head: u32,
    sub_head_byte: u8,
    mode: i8,
    raw_kind: u8,
) -> Option<PersonBannerView> {
    if !record_present { return None; }
    if !registration_ok { return None; }
    let category_byte = if mode != 0 {
        0x51
    } else {
        match raw_kind {
            0x0b | 0x11 | 0x17 | 0x1e | 0x23 | 0x29 | 0x3b => 0x4e,
            0x0c | 0x0d | 0x12 | 0x14 | 0x19 | 0x1a | 0x1b | 0x1c
            | 0x1f | 0x24 | 0x25 | 0x2c | 0x2d => 0x4f,
            _ => 0x4d,
        }
    };
    Some(PersonBannerView { record_head, category_byte, sub_head_byte })
}

// =====================================================================
// 0x007E7820 — widget-pool pop/close (NOT a screen setup)
// =====================================================================

/// `FUN_007E7820` is **not** a screen builder — it is the widget-pool
/// pop/close routine used from the layout engine (compacts
/// `DAT_00B5CFF6[]` and, on the last pop, resets the pool from the
/// snapshot at `param_1 + 0x3000`). Ported into the render layer, not
/// as a `build_*` view here. This stub records the classification.
pub fn build_widget_pop_stub(_registration_ok: bool) -> Option<()> {
    // Intentionally always None: this decompile is out-of-scope for
    // the screen-batch view registry.
    None
}

// =====================================================================
// 0x007FAEC0 — Search-result detail (3 slots)
// =====================================================================

/// Search-result detail view — 3 slots.
///
/// * Slot 0: the input pointer (record head byte carried here).
/// * Slot 1: a freshly-allocated 0x99-byte sub-record copy taken from
///   `param_1 + param_1[0]*0x2B0 + 0x1FB` (25 dwords + trailing byte,
///   matching the exe's 0x26-iteration loop plus tail byte copy).
/// * Slot 2: the status byte at `param_1[0]*0x2B0 + 0x2A4`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResultDetailView {
    pub record_head_byte: i8,
    pub sub_record: Vec<u8>, // 0x99 = 153 bytes
    pub status_byte: i8,
}

impl Default for SearchResultDetailView {
    fn default() -> Self {
        Self { record_head_byte: 0, sub_record: vec![0u8; 0x99], status_byte: 0 }
    }
}

/// Direct port of `FUN_007FAECO(param_1)`.
///
/// * `param_1_present` matches the `param_1 != NULL` gate (the exe
///   fires a MsgBox and returns on NULL — we return `None`).
/// * `sub_record` must be 0x99 bytes long (the copy source).
pub fn build_search_result_detail(
    registration_ok: bool,
    param_1_present: bool,
    record_head_byte: i8,
    sub_record: Vec<u8>,
    status_byte: i8,
) -> Option<SearchResultDetailView> {
    if !param_1_present { return None; }
    if !registration_ok { return None; }
    let mut buf = sub_record;
    buf.resize(0x99, 0);
    Some(SearchResultDetailView { record_head_byte, sub_record: buf, status_byte })
}

// =====================================================================
// 0x00803E00 — Setup / QSTART bootstrap (1 slot + handoff)
// =====================================================================

/// Setup / QSTART bootstrap view — 1 slot.
///
/// The exe:
/// 1. Checks the `CM3_QSTART` config file exists (`FUN_008FB0B0` +
///    `FUN_008FB3F0`); if missing but the parent path is present it
///    fires the noreturn `FUN_009349C4(-1)` fatal.
/// 2. Pulls the `_seed=` and `_nonetwork=` config values.
/// 3. Loops on `FUN_007E6430` registering the screen whose only slot
///    is the boolean `*(DAT_00ACDF28 + 0x213) == 0`.
/// 4. Hands off to the `0x00808A70` persistent-registration path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SetupBootstrapView {
    /// Slot 0: `*(DAT_00ACDF28 + 0x213) == 0` — "offline / no-network"
    /// flag exposed to the Setup screen.
    pub offline_flag: bool,
    /// Cached `_seed=` value pulled from the QSTART config.
    pub seed: u32,
    /// Whether the `_nonetwork=` key was present (clears
    /// `DAT_00A6BEC8`).
    pub nonetwork: bool,
}

/// Direct port of `FUN_00803E00`.
///
/// * `qstart_present` matches `FUN_008FB3F0(CM3_QSTART) == 0` (config
///   present) — if `qstart_missing_and_pathexists` is true, the exe
///   fatals via `FUN_009349C4(-1)` and we return `None`.
/// * `registration_ok` matches the eventual `FUN_007E6430(...) != 0`.
pub fn build_setup_bootstrap(
    registration_ok: bool,
    qstart_present: bool,
    qstart_missing_and_pathexists: bool,
    offline_flag: bool,
    seed: u32,
    nonetwork: bool,
) -> Option<SetupBootstrapView> {
    if !qstart_present && qstart_missing_and_pathexists { return None; }
    if !registration_ok { return None; }
    Some(SetupBootstrapView { offline_flag, seed, nonetwork })
}

// =====================================================================
// 0x00808A70 — 5-slot all-zero screen (persistent handoff target)
// =====================================================================

/// 5-slot all-zero screen — the persistent-registration target that
/// the Setup bootstrap hands off to (`FUN_007E6A20(&LAB_00808AE0, ...,
/// 5)`), then the initial registration itself here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FiveZeroSlotView {
    pub slot0: u32,
    pub slot1: u32,
    pub slot2: u32,
    pub slot3: u32,
    pub slot4: u32,
}

/// Direct port of `FUN_00808A70`.
pub fn build_five_zero_slot(registration_ok: bool) -> Option<FiveZeroSlotView> {
    if !registration_ok { return None; }
    Some(FiveZeroSlotView::default())
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- 0x007CDB80 --
    #[test]
    fn person_banner_mode_nonzero_forces_0x51() {
        let v = build_person_banner(true, true, 0xDEADBEEF, 0x77, 1, 0x0b).unwrap();
        assert_eq!(v.category_byte, 0x51);
        assert_eq!(v.record_head, 0xDEADBEEF);
        assert_eq!(v.sub_head_byte, 0x77);
    }
    #[test]
    fn person_banner_first_group_maps_to_0x4e() {
        for k in [0x0b, 0x11, 0x17, 0x1e, 0x23, 0x29, 0x3b] {
            let v = build_person_banner(true, true, 0, 0, 0, k).unwrap();
            assert_eq!(v.category_byte, 0x4e, "kind {:#x}", k);
        }
    }
    #[test]
    fn person_banner_second_group_maps_to_0x4f() {
        for k in [0x0c, 0x0d, 0x12, 0x14, 0x19, 0x1a, 0x1b, 0x1c,
                  0x1f, 0x24, 0x25, 0x2c, 0x2d] {
            let v = build_person_banner(true, true, 0, 0, 0, k).unwrap();
            assert_eq!(v.category_byte, 0x4f, "kind {:#x}", k);
        }
    }
    #[test]
    fn person_banner_default_maps_to_0x4d() {
        let v = build_person_banner(true, true, 0, 0, 0, 0xFF).unwrap();
        assert_eq!(v.category_byte, 0x4d);
    }
    #[test]
    fn person_banner_missing_record_returns_none() {
        assert!(build_person_banner(true, false, 0, 0, 0, 0).is_none());
    }
    #[test]
    fn person_banner_failed_registration_returns_none() {
        assert!(build_person_banner(false, true, 0, 0, 0, 0).is_none());
    }

    // -- 0x007E7820 --
    #[test]
    fn widget_pop_is_out_of_scope_stub() {
        // Not a screen — always None regardless of input.
        assert!(build_widget_pop_stub(true).is_none());
        assert!(build_widget_pop_stub(false).is_none());
    }

    // -- 0x007FAEC0 --
    #[test]
    fn search_result_detail_null_param_returns_none() {
        assert!(build_search_result_detail(true, false, 0, vec![], 0).is_none());
    }
    #[test]
    fn search_result_detail_carries_all_slots() {
        let buf = vec![0xAB; 0x99];
        let v = build_search_result_detail(true, true, 3, buf.clone(), 7).unwrap();
        assert_eq!(v.record_head_byte, 3);
        assert_eq!(v.status_byte, 7);
        assert_eq!(v.sub_record.len(), 0x99);
        assert_eq!(v.sub_record[0], 0xAB);
    }
    #[test]
    fn search_result_detail_pads_short_sub_record() {
        let v = build_search_result_detail(true, true, 0, vec![1, 2, 3], 0).unwrap();
        assert_eq!(v.sub_record.len(), 0x99);
        assert_eq!(&v.sub_record[..3], &[1, 2, 3]);
    }
    #[test]
    fn search_result_detail_failed_registration_returns_none() {
        assert!(build_search_result_detail(false, true, 0, vec![0; 0x99], 0).is_none());
    }

    // -- 0x00803E00 --
    #[test]
    fn setup_bootstrap_qstart_fatal_returns_none() {
        assert!(build_setup_bootstrap(true, false, true, false, 0, false).is_none());
    }
    #[test]
    fn setup_bootstrap_normal_flow_carries_slot() {
        let v = build_setup_bootstrap(true, true, false, true, 0x1234_5678, true).unwrap();
        assert!(v.offline_flag);
        assert_eq!(v.seed, 0x1234_5678);
        assert!(v.nonetwork);
    }
    #[test]
    fn setup_bootstrap_qstart_missing_but_no_path_ok() {
        // If the parent path check doesn't fire the fatal, we proceed.
        let v = build_setup_bootstrap(true, false, false, false, 0, false).unwrap();
        assert!(!v.offline_flag);
    }
    #[test]
    fn setup_bootstrap_failed_registration_returns_none() {
        assert!(build_setup_bootstrap(false, true, false, false, 0, false).is_none());
    }

    // -- 0x00808A70 --
    #[test]
    fn five_zero_slot_all_zero() {
        let v = build_five_zero_slot(true).unwrap();
        assert_eq!(v, FiveZeroSlotView::default());
    }
    #[test]
    fn five_zero_slot_failed_registration_returns_none() {
        assert!(build_five_zero_slot(false).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b19: PersonBannerView reads {record_head, sub_head_byte, raw_kind} + \
flags {record_present} + byte `mode`; SearchResultDetailView reads \
{record_head_byte, status_byte} + buffer `sub_record` + flag \
{param_1_present}; SetupBootstrapView reads {seed} + flags \
{qstart_present, qstart_missing_and_pathexists, offline_flag, nonetwork}; \
FiveZeroSlotView needs only registration_ok. widget_pop_stub is \
intentionally non-populatable (not a screen).\n\
UNKNOWNS: raw_kind → category_byte remap table is fully decoded, but the \
producing function (FUN_00618410) has not been ported — populators pass \
the facade byte through unchanged.";

use crate::world_facade::WorldFacade;

pub fn populate_person_banner(world: &WorldFacade) -> Option<PersonBannerView> {
    build_person_banner(
        world.registration_ok,
        world.flag("record_present"),
        world.handle("record_head"),
        world.byte("sub_head_byte") as u8,
        world.byte("mode") as i8,
        world.byte("raw_kind") as u8,
    )
}

pub fn populate_search_result_detail(world: &WorldFacade) -> Option<SearchResultDetailView> {
    build_search_result_detail(
        world.registration_ok,
        world.flag("param_1_present"),
        world.byte("record_head_byte") as i8,
        world.buffer("sub_record").to_vec(),
        world.byte("status_byte") as i8,
    )
}

pub fn populate_setup_bootstrap(world: &WorldFacade) -> Option<SetupBootstrapView> {
    build_setup_bootstrap(
        world.registration_ok,
        world.flag("qstart_present"),
        world.flag("qstart_missing_and_pathexists"),
        world.flag("offline_flag"),
        world.handle("seed"),
        world.flag("nonetwork"),
    )
}

pub fn populate_five_zero_slot(world: &WorldFacade) -> Option<FiveZeroSlotView> {
    build_five_zero_slot(world.registration_ok)
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_person_banner_none_without_record() {
        assert!(populate_person_banner(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_person_banner_with_record_maps_default_to_0x4d() {
        let w = WorldFacade::ready()
            .with_flag("record_present", true)
            .with_handle("record_head", 0xAB);
        let v = populate_person_banner(&w).unwrap();
        assert_eq!(v.category_byte, 0x4d);
    }
    #[test]
    fn populate_search_result_detail_none_without_param_present() {
        assert!(populate_search_result_detail(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_search_result_detail_pads_buffer() {
        let w = WorldFacade::ready().with_flag("param_1_present", true);
        let v = populate_search_result_detail(&w).unwrap();
        assert_eq!(v.sub_record.len(), 0x99);
    }
    #[test]
    fn populate_setup_bootstrap_qstart_present_ok() {
        let w = WorldFacade::ready().with_flag("qstart_present", true);
        assert!(populate_setup_bootstrap(&w).is_some());
    }
    #[test]
    fn populate_setup_bootstrap_qstart_fatal_none() {
        let w = WorldFacade::ready()
            .with_flag("qstart_missing_and_pathexists", true);
        assert!(populate_setup_bootstrap(&w).is_none());
    }
    #[test]
    fn populate_five_zero_slot_needs_registration() {
        assert!(populate_five_zero_slot(&WorldFacade::default()).is_none());
        assert!(populate_five_zero_slot(&WorldFacade::ready()).is_some());
    }
}
