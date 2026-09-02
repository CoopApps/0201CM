//! Batch 28: 3 competition-screen setup functions from `comp_screens.cpp`.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//!
//! - `004a0c10.c` — competition-list screen builder. Enumerates 22 slots
//!   (via `FUN_004a5610(i, 0)` returning `'\a'` for a valid row) and
//!   emits one highlighted row for the current selection plus one row
//!   per entry in a linked structure at stride `0x26b` (619 bytes).
//! - `004a1fd0.c` — competition-screen command pre-dispatcher. Reads a
//!   state byte from the seat's per-competition-slot record (`0x18c`
//!   stride) at offset `+0x0` inside the slot. If not `1`, falls
//!   through to the two shared dispatchers `FUN_007491e0` (global) and
//!   `FUN_0074bf60` (club). If `1`, cross-checks fields 1 and 2, emits
//!   a set_field for one of them, and returns the "handled-with-refresh"
//!   sentinel `-0xb` (-11).
//! - `004a3d20.c` — one-row widget for the competition list. Guards on
//!   a null context (raises the shipped `"Error"` MessageBox with the
//!   dev-path string that leaked into the release), caches the widget
//!   ref through `param_1: short*`, defaults an empty name to
//!   `"League"`, and emits the row with the packed id
//!   `param_10 + (row_idx * 5 + 5) * 200`.

use serde::{Deserialize, Serialize};

// =====================================================================
// FUN_004a0c10 — competition-list screen builder
// =====================================================================

/// Maximum number of competition-list rows the exe iterates
/// (`1..=0x16`, i.e. 22 rows). Verified from
/// `if (0x16 < iVar10) { emit_footer; return; }` at 004a0eba.
pub const COMP_LIST_MAX_ROWS: u32 = 22;

/// Byte stride for the per-entry linked structure walked by the second
/// loop in `FUN_004a0c10`. Verified from `psVar11 = (short*)((int)psVar11 + 0x26b)`.
pub const COMP_LIST_ENTRY_STRIDE: usize = 0x26b;

/// The classification `FUN_004a5610` returns for a valid row: `'\a'`.
pub const ROW_VALID_MARKER: u8 = 0x07;

/// One row of the 22-slot upper list — the exe's outer enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompListSlotRow {
    /// Row index in the 1..=22 enumeration.
    pub slot: u32,
    /// True iff `FUN_004a5610(slot, 0) == ROW_VALID_MARKER`.
    pub valid: bool,
    /// True iff the row's `slot` equals `get_field(0x17)` — the
    /// currently-selected competition.
    pub highlighted: bool,
    /// Display name — `FUN_004abbe0(slot, buf, 200)` for valid rows,
    /// left empty for invalid ones.
    pub name: String,
    /// Id returned by `FUN_004abf80(slot)` for valid rows; `0` for
    /// invalid.
    pub id: u32,
}

/// One row of the per-entry linked structure walked at stride `0x26b`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompListEntryRow {
    /// Value of the leading `short` — the guard `*psVar11 != 0`
    /// determines whether a widget row is emitted at all.
    pub head_word: i16,
    /// Formatted name (`FUN_00652e60(*psVar11, ctx, buf, 2000)`).
    pub name: String,
    /// Row index (0-based) among the emitted (non-zero-head) entries.
    pub display_index: u32,
}

/// The finished competition-list screen model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CompListView {
    /// The upper 22-slot enumeration (only valid slots kept — invalid
    /// slots would emit an empty greyed row in the exe).
    pub slots: Vec<CompListSlotRow>,
    /// The per-entry linked-structure rows (only `head_word != 0`).
    pub entries: Vec<CompListEntryRow>,
    /// Screen base x/y as passed to the builder (kept for round-trip).
    pub base_x: i16,
    pub base_y: i16,
}

/// Direct port of `FUN_004a0c10(state, base_x, base_y)`.
///
/// Inputs come pre-collected from the state pool + iteration helpers:
/// - `slot_probe`: for each `i in 1..=22`, returns `Some((id, name))`
///   iff `FUN_004a5610(i, 0) == ROW_VALID_MARKER` (i.e. the row is a
///   populated competition slot). The name should be no longer than
///   200 chars (that's the exe's `FUN_004abbe0` buffer size).
/// - `selected_slot`: the value of `get_field(0x17)` — highlighted row.
/// - `entries`: the linked list walked at stride `0x26b`; each entry's
///   `head_word` and formatted `name` come from the caller's data.
///
/// `registration_ok` mirrors `FUN_00549580` returning a valid parent
/// widget — `false` short-circuits with `None` (matches the exe's
/// null-guard behaviour higher up the call chain).
pub fn build_comp_list_screen(
    registration_ok: bool,
    base_x: i16,
    base_y: i16,
    selected_slot: u32,
    slot_probe: impl Fn(u32) -> Option<(u32, String)>,
    entries: Vec<(i16, String)>,
) -> Option<CompListView> {
    if !registration_ok { return None; }

    let mut slots = Vec::new();
    for slot in 1..=COMP_LIST_MAX_ROWS {
        let (valid, id, name) = match slot_probe(slot) {
            Some((id, name)) => (true, id, name),
            None => (false, 0, String::new()),
        };
        slots.push(CompListSlotRow {
            slot,
            valid,
            highlighted: valid && slot == selected_slot,
            name,
            id,
        });
    }

    // Second loop: only rows with head_word != 0 emit a widget; give
    // them a running 0-based display_index.
    let mut display_index = 0u32;
    let entry_rows = entries
        .into_iter()
        .map(|(head_word, name)| {
            let out = CompListEntryRow {
                head_word,
                name,
                display_index: if head_word != 0 { display_index } else { 0 },
            };
            if head_word != 0 { display_index += 1; }
            out
        })
        .collect();

    Some(CompListView {
        slots,
        entries: entry_rows,
        base_x,
        base_y,
    })
}

// =====================================================================
// FUN_004a1fd0 — competition-screen command pre-dispatcher
// =====================================================================

/// Byte offset inside a per-competition-slot seat record for the
/// gate byte the pre-dispatcher tests. Verified from the read
/// `[seat + 0xba966 + slot * 0x18c]`.
pub const SEAT_SLOT_GATE_OFFSET: usize = 0x0;

/// Offset inside a per-competition-slot seat record for the id-word
/// the exe cross-checks against `get_field(1)`/`get_field(2)`.
/// Verified from the read `[seat + 0xba9a6 + slot * 0x18c]`
/// (`0xba9a6 - 0xba966 = 0x40`).
pub const SEAT_SLOT_ID_OFFSET: usize = 0x40;

/// Per-competition-slot record stride inside the seat.
pub const SEAT_SLOT_STRIDE: usize = 0x18c;

/// The "handled with refresh" sentinel the pre-dispatcher returns.
/// Verified as `-0xb` = `-11`.
pub const HANDLED_REFRESH_SENTINEL: i32 = -0xb;

/// Which sibling get/set_field the pre-dispatcher's normal path
/// operates on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FieldChannel { One = 1, Two = 2 }

/// Outcome of the pre-dispatcher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompCommandOutcome {
    /// Gate byte was not `1` — the exe falls through to the shared
    /// dispatchers `FUN_007491e0` then `FUN_0074bf60`. The caller
    /// runs those and returns their combined result.
    FallThroughShared,
    /// Gate byte was `1` and the slot's id-word matched `get_field(1)`
    /// or `get_field(2)`. The pre-dispatcher sets that field to `0`
    /// and returns the refresh sentinel. `channel` says which field
    /// was cleared.
    ClearField { channel: FieldChannel },
    /// Gate byte was `1` and no field matched. The pre-dispatcher
    /// writes the slot's id-word into `get_field(1)`'s slot with
    /// channel 1 (if `get_field(1) == 0`) or channel 2 otherwise, and
    /// returns the refresh sentinel.
    WriteSlotId { channel: FieldChannel, slot_id: u32 },
}

/// Direct port of `FUN_004a1fd0(param_1: short)`.
///
/// Inputs (all pre-fetched from the seat pool):
/// - `slot`: the short parameter (negative → invalid, seat reads
///   substitute `0`).
/// - `gate_byte`: the byte at `seat + 0xba966 + slot * 0x18c`, or
///   `fallback_gate_byte` when `slot < 0`.
/// - `slot_id_word`: the u32 at `seat + 0xba9a6 + slot * 0x18c`.
/// - `field_1_value`, `field_2_value`: `get_field(1)` / `get_field(2)`
///   snapshots.
pub fn dispatch_comp_command(
    slot: i16,
    gate_byte: u16,
    fallback_gate_byte: u16,
    slot_id_word: u32,
    field_1_value: u32,
    field_2_value: u32,
) -> CompCommandOutcome {
    let effective_gate = if slot < 0 { fallback_gate_byte } else { gate_byte };
    if effective_gate != 1 {
        return CompCommandOutcome::FallThroughShared;
    }

    let id = if slot < 0 { 0 } else { slot_id_word };

    // First check: does field 1 already carry this slot's id?
    if id == field_1_value {
        return CompCommandOutcome::ClearField { channel: FieldChannel::One };
    }
    // Then: does field 2?
    if id == field_2_value {
        return CompCommandOutcome::ClearField { channel: FieldChannel::Two };
    }
    // Otherwise stash the slot's id — pick channel 1 if it's empty,
    // else channel 2. (Matches the exe's `if (get_field(1) == 0)` branch.)
    let channel = if field_1_value == 0 { FieldChannel::One } else { FieldChannel::Two };
    CompCommandOutcome::WriteSlotId { channel, slot_id: id }
}

// =====================================================================
// FUN_004a3d20 — one row of the competition list
// =====================================================================

/// Default fallback string when the row's fetched display name is empty.
/// Verified from the exe reading `s_League_0097d7f4` after the
/// `DAT_00dc723c == '\0'` guard.
pub const DEFAULT_ROW_LABEL: &str = "League";

/// Widget-id packing multiplier: the exe computes
/// `iVar1 = row * 5 + 5; id = base + iVar1 * 200`.
pub const ROW_ID_STRIDE: u32 = 200;

/// Result of one row-builder invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompListRowOutcome {
    /// `param_6 == NULL` — the exe raises `"Error"` MessageBox
    /// carrying the shipped-in dev-path string and clears
    /// `DAT_00b4d5a8`. Returns `0`.
    ErrorDialog,
    /// Normal row emit. `parent_widget` = the cached parent (either
    /// preexisting `*param_1` or freshly built when the input was `-1`);
    /// `packed_id` = the y-stride widget id.
    RowEmitted {
        parent_widget: i16,
        packed_id: u32,
        display_text: String,
    },
}

/// Direct port of `FUN_004a3d20(cached_parent: short*, base_x, base_y, w, h, ctx, name_ctx_1, name_ctx_2, row_idx, base_id)`.
///
/// Inputs:
/// - `ctx_is_null`: `param_6 == NULL` — triggers the error branch.
/// - `cached_parent`: current value of `*param_1`; `-1` means "not
///   yet built, do so now". The builder returns the widget id it
///   used in `parent_widget` (either the input value if it was ≥ 0,
///   or a freshly-allocated one otherwise).
/// - `fresh_parent_id`: the id `FUN_00549580` would have returned had
///   it been called (only consulted when `cached_parent == -1`).
/// - `fetched_text`: the string `FUN_004c63c0(...)` returned; empty
///   string triggers the `"League"` fallback.
/// - `row_idx`: `param_9` — used in the packed-id formula.
/// - `base_id`: `param_10`.
#[allow(clippy::too_many_arguments)]
pub fn build_comp_list_row(
    ctx_is_null: bool,
    cached_parent: i16,
    fresh_parent_id: i16,
    fetched_text: &str,
    row_idx: u32,
    base_id: u32,
) -> CompListRowOutcome {
    if ctx_is_null {
        return CompListRowOutcome::ErrorDialog;
    }
    let parent_widget = if cached_parent == -1 { fresh_parent_id } else { cached_parent };
    let display_text = if fetched_text.is_empty() {
        DEFAULT_ROW_LABEL.to_string()
    } else {
        fetched_text.to_string()
    };
    let packed_id = base_id.wrapping_add((row_idx * 5 + 5) * ROW_ID_STRIDE);
    CompListRowOutcome::RowEmitted { parent_widget, packed_id, display_text }
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- build_comp_list_screen ----

    #[test]
    fn comp_list_produces_22_slot_rows() {
        // No valid slots, no entries.
        let v = build_comp_list_screen(
            true, 100, 200, 0,
            |_| None,
            Vec::new(),
        ).unwrap();
        assert_eq!(v.slots.len(), 22);
        for s in &v.slots {
            assert!(!s.valid);
            assert!(!s.highlighted);
            assert!(s.name.is_empty());
            assert_eq!(s.id, 0);
        }
        assert!(v.entries.is_empty());
        assert_eq!(v.base_x, 100);
        assert_eq!(v.base_y, 200);
    }

    #[test]
    fn comp_list_marks_selected_only_when_valid() {
        let v = build_comp_list_screen(
            true, 0, 0, 5,
            // Only slot 5 is valid; slot 5 is also the selection.
            |i| if i == 5 { Some((42, "Premier".into())) } else { None },
            Vec::new(),
        ).unwrap();
        assert!(v.slots[4].valid);
        assert!(v.slots[4].highlighted);
        assert_eq!(v.slots[4].id, 42);
        assert_eq!(v.slots[4].name, "Premier");
        // Selection pointing at an INVALID slot must NOT highlight.
        let v2 = build_comp_list_screen(
            true, 0, 0, 7,
            |i| if i == 5 { Some((42, "Premier".into())) } else { None },
            Vec::new(),
        ).unwrap();
        for s in &v2.slots {
            assert!(!s.highlighted, "invalid slot {} was highlighted", s.slot);
        }
    }

    #[test]
    fn comp_list_entries_skip_zero_head_but_keep_them_in_output() {
        let entries = vec![
            (0, "hidden".into()),        // head_word 0 → not emitted (display_index = 0)
            (5, "shown A".into()),       // display_index = 0
            (0, "also hidden".into()),
            (9, "shown B".into()),       // display_index = 1
        ];
        let v = build_comp_list_screen(true, 0, 0, 0, |_| None, entries).unwrap();
        assert_eq!(v.entries.len(), 4);
        assert_eq!(v.entries[0].head_word, 0);
        assert_eq!(v.entries[1].head_word, 5);
        assert_eq!(v.entries[1].display_index, 0);
        assert_eq!(v.entries[3].head_word, 9);
        assert_eq!(v.entries[3].display_index, 1);
    }

    #[test]
    fn comp_list_failed_registration_returns_none() {
        let v = build_comp_list_screen(false, 0, 0, 0, |_| None, Vec::new());
        assert!(v.is_none());
    }

    #[test]
    fn comp_list_row_range_is_1_to_22_inclusive() {
        // Verify no off-by-one: slot 1 through 22.
        let v = build_comp_list_screen(true, 0, 0, 0, |_| None, Vec::new()).unwrap();
        assert_eq!(v.slots.first().unwrap().slot, 1);
        assert_eq!(v.slots.last().unwrap().slot, 22);
        assert_eq!(COMP_LIST_MAX_ROWS, 22);
    }

    // ---- dispatch_comp_command ----

    #[test]
    fn dispatch_gate_zero_falls_through_to_shared() {
        let out = dispatch_comp_command(3, 0, 0, 99, 0, 0);
        assert_eq!(out, CompCommandOutcome::FallThroughShared);
    }

    #[test]
    fn dispatch_gate_two_falls_through_to_shared() {
        // Any value != 1 falls through.
        assert_eq!(
            dispatch_comp_command(3, 2, 0, 99, 0, 0),
            CompCommandOutcome::FallThroughShared,
        );
        assert_eq!(
            dispatch_comp_command(3, 255, 0, 99, 0, 0),
            CompCommandOutcome::FallThroughShared,
        );
    }

    #[test]
    fn dispatch_negative_slot_uses_fallback_gate_byte() {
        // slot < 0 substitutes fallback_gate_byte (mirrors exe branch
        // `sVar5 == -1 → sVar1 = DAT_00dbbf7a`).
        // fallback = 1 → the normal-path handler runs.
        let out = dispatch_comp_command(-1, 0, 1, 999, 0, 0);
        // id computed as 0 (since slot < 0 → id substitutes 0).
        // 0 == field_1_value(0) → ClearField channel One.
        assert_eq!(out, CompCommandOutcome::ClearField { channel: FieldChannel::One });
    }

    #[test]
    fn dispatch_gate_one_id_matches_field_1_clears_it() {
        let out = dispatch_comp_command(3, 1, 0, 77, 77, 999);
        assert_eq!(out, CompCommandOutcome::ClearField { channel: FieldChannel::One });
    }

    #[test]
    fn dispatch_gate_one_id_matches_field_2_clears_it() {
        let out = dispatch_comp_command(3, 1, 0, 55, 999, 55);
        assert_eq!(out, CompCommandOutcome::ClearField { channel: FieldChannel::Two });
    }

    #[test]
    fn dispatch_gate_one_no_match_writes_to_empty_field_1() {
        // field_1_value == 0 → write to channel 1.
        let out = dispatch_comp_command(3, 1, 0, 42, 0, 99);
        assert_eq!(out, CompCommandOutcome::WriteSlotId { channel: FieldChannel::One, slot_id: 42 });
    }

    #[test]
    fn dispatch_gate_one_no_match_field_1_taken_writes_to_field_2() {
        // field_1_value != 0 → write to channel 2.
        let out = dispatch_comp_command(3, 1, 0, 42, 88, 99);
        assert_eq!(out, CompCommandOutcome::WriteSlotId { channel: FieldChannel::Two, slot_id: 42 });
    }

    #[test]
    fn dispatch_sentinel_is_minus_eleven() {
        assert_eq!(HANDLED_REFRESH_SENTINEL, -11);
        assert_eq!(HANDLED_REFRESH_SENTINEL, -0xb);
    }

    // ---- build_comp_list_row ----

    #[test]
    fn row_null_ctx_returns_error_dialog() {
        assert_eq!(
            build_comp_list_row(true, -1, 5, "anything", 0, 0),
            CompListRowOutcome::ErrorDialog,
        );
    }

    #[test]
    fn row_empty_text_uses_default_league_label() {
        let out = build_comp_list_row(false, 10, 99, "", 0, 1000);
        match out {
            CompListRowOutcome::RowEmitted { display_text, parent_widget, packed_id } => {
                assert_eq!(display_text, "League");
                assert_eq!(parent_widget, 10);          // used cached (not -1)
                assert_eq!(packed_id, 1000 + 5 * 200);  // (0*5+5)*200 = 1000
            }
            _ => panic!("expected RowEmitted"),
        }
    }

    #[test]
    fn row_uncached_widget_uses_freshly_built_id() {
        let out = build_comp_list_row(false, -1, 42, "Serie A", 3, 500);
        match out {
            CompListRowOutcome::RowEmitted { display_text, parent_widget, packed_id } => {
                assert_eq!(display_text, "Serie A");
                assert_eq!(parent_widget, 42);          // freshly built
                assert_eq!(packed_id, 500 + (3*5+5)*200);  // 500 + 4000 = 4500
            }
            _ => panic!("expected RowEmitted"),
        }
    }

    #[test]
    fn row_packed_id_matches_exe_formula() {
        // Sanity-check the multiplier constant.
        assert_eq!(ROW_ID_STRIDE, 200);
        // (row * 5 + 5) * 200 for a few rows
        for row in [0u32, 1, 5, 21] {
            let out = build_comp_list_row(false, 0, 0, "x", row, 0);
            match out {
                CompListRowOutcome::RowEmitted { packed_id, .. } => {
                    assert_eq!(packed_id, (row * 5 + 5) * 200,
                        "row {row} → wrong packed id");
                }
                _ => panic!(),
            }
        }
    }

    // ---- structural constants ----

    #[test]
    fn seat_slot_offsets_and_stride() {
        assert_eq!(SEAT_SLOT_GATE_OFFSET, 0);
        assert_eq!(SEAT_SLOT_ID_OFFSET, 0x40);      // 0xba9a6 - 0xba966
        assert_eq!(SEAT_SLOT_STRIDE, 0x18c);        // 396 bytes
        assert_eq!(SEAT_SLOT_STRIDE, 396);
    }

    #[test]
    fn comp_list_entry_stride_is_619_bytes() {
        assert_eq!(COMP_LIST_ENTRY_STRIDE, 0x26b);
        assert_eq!(COMP_LIST_ENTRY_STRIDE, 619);
    }
}
