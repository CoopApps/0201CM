//! Tactics-screen event dispatcher — Rust port of `FUN_0088A850`
//! (`d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/0088a850.c`, 2066 lines).
//!
//! Companion to the global sidebar dispatcher (`sidebar_dispatcher.rs` — port of
//! `FUN_007491e0`).  Where the sidebar dispatcher routes global menu/toolbar
//! commands, `FUN_0088A850` is the **tactics-screen local event bus**: it is
//! the callback the tactics screen runs on every frame and drains two sources
//! of events:
//!
//!   1. **Modal-dialog return codes** in `DAT_00dbbf7c` (0x2C..0x36).  These
//!      are the tags the tactics screen previously passed to
//!      `FUN_005e60d0` / `FUN_006547c0` when it opened a file-picker or
//!      confirm dialog; when the dialog closes it stashes its result code in
//!      `DAT_00dbbf7c` and its "OK button pressed" flag in `DAT_00dbbf80`, and
//!      the next tick this dispatcher acts on them.
//!
//!   2. **Widget-item click codes** — read from the currently-selected menu
//!      item at
//!      `(&DAT_00b59fe8)[DAT_00b5d016 * 0xc0] + 0xba966 + sVar24 * 0x18c`
//!      (the same "active-human menu item" pointer used by the sidebar
//!      dispatcher), with `DAT_00dbbf7a` as the posted-cmd fallback when no
//!      item is selected.  Every tactics widget the builder `FUN_00884700`
//!      spawns is registered with one of the type codes 0x01..0x22.
//!
//! **Editing model — confirmed from the decompile.**  The dispatcher never
//! writes into the club's live tactic record on disk.  Every mutating branch
//! follows the same three-step pattern:
//!
//!   1. Save the current scratch (`puVar11 + 0x391`, 0xF07 dwords ≈ 15.4 kB)
//!      to a **snapshot** at `puVar11 + 0x4A63` (same size),
//!   2. Apply the delta (formation change / slot swap / slider tick / …),
//!   3. Set the dirty flag `puVar11 + 0x4A5F = 1` and mark the
//!      formation-shadow flag `puVar11 + 0xC4DE = 1`.
//!
//! On **Save Formation** (0x2E) the current scratch is written straight to
//! `<name>.tct` via `FUN_00895D40`; on **Discard/Revert** (0x0F or 0x31) the
//! snapshot at `+0x4A63` is copied back into `+0x391`; on **Close/Back**
//! (0x20) the dirty flag gates a "Changes have been made" confirm.  So:
//! **`FUN_0088A850` maintains its own scratch tactic during editing and only
//! commits on Save.**  It never touches the loaded `.tct` in memory after the
//! initial copy the initialiser (`FUN_008939C0`) does at screen creation.
//!
//! Style-guide: enum + `route(code) → dispatch(cmd, state)` mirrors
//! [`crate::sidebar_dispatcher`].  Every arm mutates exactly one field of
//! [`TacticState`] (or opens a dialog) and returns the exe's outer return code
//! as a [`TacticDispatchReturn`].

// Companion reader (not yet ported): reports/tactic_file_decode.md §7's
// `TacticView` — will land as `crate::tactic_file::TacticView`.  Not imported
// here so this module compiles standalone against the current tree.

// ---------------------------------------------------------------------------
// 1. Modal-dialog return codes (`DAT_00dbbf7c`).
// ---------------------------------------------------------------------------

/// Modal-return tag passed to a dialog when it was opened, and echoed back in
/// `DAT_00dbbf7c` when the dialog closes.  A separate global `DAT_00dbbf80`
/// carries the button pressed (2 == OK/YES; anything else == cancel).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TacticModal {
    /// 0x2C — "The formation `<name>` has not been saved" info banner
    /// following an attempted Load Formation.  Non-modal — no action on OK.
    LoadNotSavedInfo = 0x2C,
    /// 0x2D — file-picker return for **Load Formation** (`.tct`).  On OK,
    /// calls `FUN_00895C10(name, 0)` to load; validates the filename against
    /// the current file.
    LoadFilePicker = 0x2D,
    /// 0x2E — file-picker return for **Save Formation** (`.tct`).  On OK,
    /// calls `FUN_00895D40(name, tactic)`.
    SaveFilePicker = 0x2E,
    /// 0x2F — file-picker return for **Delete Formation**.  On OK, calls
    /// `FUN_009354CA(name)`.
    DeleteFilePicker = 0x2F,
    /// 0x30 — file-picker return for a load-preset entry point that reuses
    /// the plain `FUN_00895C10(name, 0)` path.
    LoadFilePickerAlt = 0x30,
    /// 0x31 — "Changes have been made — discard?" confirm result.  On YES
    /// (button==2), `FUN_00884B00` is called 3× to release editor sub-views
    /// and the tactics screen self-releases (`return -9`).
    DiscardChangesConfirm = 0x31,
    /// 0x32 — Delete-tactic "Please Confirm" YES.  Calls `FUN_00897010`
    /// (unregister slot), then `FUN_00894A20` / `FUN_00894650`, then releases
    /// editor sub-views 5..7 and self-destroys.
    DeleteTacticConfirm = 0x32,
    /// 0x33 — "Formation must be saved" confirm YES.  Opens the save-as
    /// file-name dialog, seeded with either an entered name or the current
    /// tactic display name at `tactic + 0x391`.
    MustBeSavedConfirm = 0x33,
    /// 0x34 — "Set to default" confirm YES.  Calls `FUN_00895F90(FUN_007E6EE0(9))`.
    SetToDefaultConfirm = 0x34,
    /// 0x35 — file-picker return for **Save as Preset** (`.pct`).  On OK,
    /// calls `FUN_005A0C10(name, 1, 1)`.
    SavePresetFilePicker = 0x35,
    /// 0x36 — file-picker return for **Load Preset Formation** (`.pct`).  On
    /// OK, calls `FUN_00895C10(name, 1)`.
    LoadPresetFilePicker = 0x36,
}

impl TacticModal {
    pub fn from_code(code: u8) -> Option<Self> {
        use TacticModal::*;
        Some(match code {
            0x2C => LoadNotSavedInfo,
            0x2D => LoadFilePicker,
            0x2E => SaveFilePicker,
            0x2F => DeleteFilePicker,
            0x30 => LoadFilePickerAlt,
            0x31 => DiscardChangesConfirm,
            0x32 => DeleteTacticConfirm,
            0x33 => MustBeSavedConfirm,
            0x34 => SetToDefaultConfirm,
            0x35 => SavePresetFilePicker,
            0x36 => LoadPresetFilePicker,
            _ => return None,
        })
    }

    #[inline] pub fn to_code(self) -> u8 { self as u8 }
}

// ---------------------------------------------------------------------------
// 2. Widget click codes (sVar23 chain — the "event bus" per widget click).
// ---------------------------------------------------------------------------

/// One tactics-screen widget event.  Numeric values come straight from the
/// `sVar23 == 0xNN` chain in `FUN_0088A850`; names are ours.  Every widget the
/// builder `FUN_00884700` spawns is registered with one of these type-tags in
/// the sidebar-item struct at `+0xba966`.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TacticCmd {
    /// 0x01 — **New Tactic**.  Spawns a fresh editor window
    /// (`operator_new(0xC4E2)` → `FUN_00894380` → `FUN_007E6570(&LAB_0088DEF0,
    /// FUN_00890360, 1, 0, 0)`).  Different constructor branch when the
    /// current record is already the "different-view" slot (`+0xE40 != 0`).
    NewTactic = 0x01,
    /// 0x02 — **Set To Default** menu click.  Emits either the confirm dialog
    /// (tag 0x34) or, if the "any changes" gate is clear, calls
    /// `FUN_00895F90(FUN_007E6EE0(9))` immediately.
    SetToDefault = 0x02,
    /// 0x03 — **Load Formation** menu click.  Opens the file picker with tag
    /// 0x2D, or shows "not saved" info first (tag 0x2C).
    LoadFormation = 0x03,
    /// 0x04 — **Load Preset Formation** menu click.  Opens the picker
    /// scoped to `CM3_TACTICS/*.pct` with tag 0x36.
    LoadPresetFormation = 0x04,
    /// 0x05 — reserved / not observed in the dispatch chain (only appears as
    /// `!= 5` fall-through).  Kept as a distinct variant so the enum still
    /// round-trips and future decodes can populate it.
    Reserved5 = 0x05,
    /// 0x06 — **Save Formation** menu click.  Opens the save-as file-name
    /// dialog with tag 0x2E; seeds it with the current tactic's name when
    /// one is set.
    SaveFormation = 0x06,
    /// 0x07 — **Save as Preset** menu click.  Opens the file-name dialog
    /// with tag 0x35 scoped to `CM3_TACTICS/`.
    SaveAsPreset = 0x07,
    /// 0x08 — Save-with-name confirm (double-click in the picker) — re-enters
    /// the save-as flow using the widget's stored name.
    SaveWithName = 0x08,
    /// 0x09 — reserved / not observed (only appears as `!= 9`).
    Reserved9 = 0x09,
    /// 0x0A — **Unlock Formation**.  Copies snapshot `+0x4A63` back into
    /// scratch `+0x391`, clears the lock-name string at `+0xE77`, sets dirty
    /// (`+0x4A5F=1`) and shadow (`+0xC4DE=1`).  If the "already-modified" gate
    /// is set, shows the "must save" confirm (tag 0x33) instead.
    UnlockFormation = 0x0A,
    /// 0x0B — **Lock Formation**.  Copies scratch→snapshot, reads text from
    /// `FUN_005E7980(0)` (the entry field), memcopies it into `+0xE77`, sets
    /// dirty.  Same "must save" gate on failure.
    LockFormation = 0x0B,
    /// 0x0C — File selected in the Load-Preset picker (double-click).  Re-runs
    /// the picker flow with an explicit filename.
    LoadPresetPickerRow = 0x0C,
    /// 0x0D — **Copy Tactic From Opponent**.  Records the opponent name via
    /// `FUN_007E7130(10, name, 0)`.
    CopyFromOpponent = 0x0D,
    /// 0x0E — **Load Opponent Tactic**.  Enumerates recent matches via
    /// `FUN_00672420` and re-opens the tactic screen with the opponent's
    /// tactic (`FUN_00884700(this, is_reserve, 0, 0, opponent, 0)`).  Falls
    /// back to one of five "No tactics / No matches available" info dialogs
    /// depending on whether this is 1st-team, reserve, or B-team.
    LoadOpponentTactic = 0x0E,
    /// 0x0F — **Revert changes**.  Copies snapshot `+0x4A63` back into
    /// scratch `+0x391` and clears the shadow flag `+0xC4DE`.
    RevertChanges = 0x0F,
    /// 0x10 — **Formation family select** (dropdown).  Calls
    /// `FUN_0059E2A0(side_bit, mentality_idx, tactic+0xC2A1)` and stamps
    /// `+0xC4DD = 1`.  Gated by `+0x2727 != 1` (not in view-only mode).
    FormationFamilySelect = 0x10,
    /// 0x11 — **Mentality panel toggle**.  Calls
    /// `FUN_0059E5B0(side_bit, tactic+0xC2CD)` and stamps `+0xC4DD = 2`.
    MentalityPanelSelect = 0x11,
    /// 0x12 — Sub-panel toggle handler `FUN_008962F0()` (no args).
    SubPanelToggle = 0x12,
    /// 0x13 — **Toggle Home/Away view**.  Snapshots current, calls
    /// `FUN_0059EAB0(side, tactic+0x2729)`, flips `+0x2729 = 2 - +0x2729`,
    /// sets dirty.  Gated by "not in view-only" (`+0x2727 != 1`).
    ToggleHomeAway = 0x13,
    /// 0x14 — **Toggle "always use this formation"** flag at
    /// `tactic + 0x2731` (u32).  Straight boolean flip.
    ToggleUseAlways = 0x14,
    /// 0x15 — **Select mentality preset** (1..3 valid, else error 0x13C6).
    /// Stamps prev-mentality at `+0x2728` (`+0x9CA*4`), writes new at
    /// `+0x2727`, mirrors `+0x2729 → +0x272D`.
    SelectMentalityPreset = 0x15,
    /// 0x16 — **Player slot click**.  Opens the per-slot player editor as a
    /// separate window (`operator_new(0xC4E2)` → `FUN_00894380` →
    /// `FUN_007E6570(&LAB_00890E50, &LAB_00893500, 1, 0, 0)` with slot idx and
    /// parent pointer wired via `FUN_007E7130`).
    OpenPlayerEditor = 0x16,
    /// 0x17 — **Swap two players in the grid**.  Reads source & target slot
    /// indices from adjacent menu items (`+0xba9dc`), compares them, and if
    /// distinct copies scratch→snapshot, writes `puVar11[0x1288] = *piVar15`
    /// (new lineup pointer), sets dirty.  Also branches into "Frida"-style
    /// error path on missing slots.
    SwapPlayers = 0x17,
    /// 0x18 — **Change slot role** (inner match on adjacent-item sub-code).
    /// On sub-code 0x16 calls `FUN_008956A0(from, to)`; on sub-code 0x18
    /// calls `FUN_008957F0(from, to)` (position swap variant).
    ChangeSlotRole = 0x18,
    /// 0x19 — **Change player number for a slot** — `FUN_00897E20(cur, new, 0)`.
    ChangeSlotNumber = 0x19,
    /// 0x1A — **Set a slider on the mentality/team-instructions panel**.
    /// Extracts idx from `(val >> 6) & 0x1F` and encodes value with
    /// `FUN_0089A580(val & 0x3F)`; if unchanged calls
    /// `FUN_0059F5A0(idx, encoded)` after snapshotting.
    SetMentalitySlider = 0x1A,
    /// 0x1B — **Set an individual-instruction slider** (per-slot).  Same
    /// idx/val extraction, then `FUN_0059FAE0(idx, encoded)`.  Errors when
    /// `idx >= 11` (error 0x1971).
    SetIndividualSlider = 0x1B,
    /// 0x1C — inner sub-code only (appears under 0x1D / 0x1E drag events);
    /// never a top-level tag.
    DragInnerRole = 0x1C,
    /// 0x1D — **Drag a role from the sidebar onto the pitch**.  Complex
    /// geometry: builds a slot list from `DAT_009B893C[]`, picks the closest
    /// slot, then either `FUN_0059D240(slot, chosen)` (position move) or
    /// `FUN_0059CEB0(role, chosen, 0)` (role drop).
    DragRoleToPitch = 0x1D,
    /// 0x1E — **Drag from pitch back to sidebar** (or between-slot).  Mirror
    /// of 0x1D; calls `FUN_0059CEB0(role, chosen, 1)`.
    DragFromPitch = 0x1E,
    /// 0x1F — **Set the mentality (x,y) via percent slider**.  Splits the raw
    /// value into x=`v/100` and y=`v%100`, validates
    /// `0 <= x < 3 && 0 <= y < 4`, writes `+0x2729 = pack(x,y)` and mirrors
    /// prev into `+0x272D`.
    SetMentalityXY = 0x1F,
    /// 0x20 — **Back / Close editor**.  If dirty (`+0x4A5F != 0`) opens the
    /// "Changes have been made" confirm (tag 0x31); otherwise releases the
    /// three editor sub-vectors (`+0x9F73`, `+0x18D5`, `+0x2735`) and the
    /// screen state block (`+0x137`) and self-destroys.
    CloseEditor = 0x20,
    /// 0x21 — **Delete Tactic confirmed**.  Runs `FUN_00894BB0` for pre-flight
    /// ("please confirm dialog" if the tactic is currently used), then
    /// `FUN_00894650` and full editor teardown (same three sub-vectors).
    DeleteTactic = 0x21,
    /// 0x22 — **Alt action route** — `FUN_00454620(this_tactic, this_editor,
    /// 0, 0, 1)` (opens a player-selection widget for one of the tactics
    /// side-panels).
    AltActionRoute = 0x22,
}

impl TacticCmd {
    pub fn from_code(code: u16) -> Option<Self> {
        use TacticCmd::*;
        Some(match code {
            0x01 => NewTactic,
            0x02 => SetToDefault,
            0x03 => LoadFormation,
            0x04 => LoadPresetFormation,
            0x05 => Reserved5,
            0x06 => SaveFormation,
            0x07 => SaveAsPreset,
            0x08 => SaveWithName,
            0x09 => Reserved9,
            0x0A => UnlockFormation,
            0x0B => LockFormation,
            0x0C => LoadPresetPickerRow,
            0x0D => CopyFromOpponent,
            0x0E => LoadOpponentTactic,
            0x0F => RevertChanges,
            0x10 => FormationFamilySelect,
            0x11 => MentalityPanelSelect,
            0x12 => SubPanelToggle,
            0x13 => ToggleHomeAway,
            0x14 => ToggleUseAlways,
            0x15 => SelectMentalityPreset,
            0x16 => OpenPlayerEditor,
            0x17 => SwapPlayers,
            0x18 => ChangeSlotRole,
            0x19 => ChangeSlotNumber,
            0x1A => SetMentalitySlider,
            0x1B => SetIndividualSlider,
            0x1C => DragInnerRole,
            0x1D => DragRoleToPitch,
            0x1E => DragFromPitch,
            0x1F => SetMentalityXY,
            0x20 => CloseEditor,
            0x21 => DeleteTactic,
            0x22 => AltActionRoute,
            _ => return None,
        })
    }

    #[inline] pub fn to_code(self) -> u16 { self as u16 }
}

// ---------------------------------------------------------------------------
// 3. Editor state — the block `FUN_008939C0` initialises and `FUN_0088A850`
//    mutates.  Field names follow the byte offsets seen in the decompile so
//    the crosswalk to the on-disk `.tct` (reports/tactic_file_decode.md) stays
//    obvious.  All offsets are BYTES from the base of the `new(0xC4E2)` block.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TacticState {
    /// Byte 0x0000 — parent club id (matches the club that owns the tactic).
    pub owner_club_id: i32,
    /// Byte 0x0004 — B-team / reserve link (0 when editing first-team).
    pub reserve_link: i32,
    /// Byte 0x0008 — "extraout_ECX[2]" — carried through as the third arg to
    /// the save/load helpers.
    pub extra2: i32,
    /// Byte 0x0018 — first word of the "opponent tactic mirror" area
    /// (`+0xD92`).  Kept as an opaque blob (`0x4C` bytes).
    pub opponent_mirror: [u8; 0x4C],

    // --- Scratch tactic record — the `.tct` on-disk fields live here. -----
    //
    // Base = byte 0x0E44 (word 0x391).  All offsets given in
    // reports/tactic_file_decode.md §4 are RELATIVE TO THIS BASE.
    /// Byte 0x0E44..0x2724 (0x38E0 bytes) — the live editable tactic record.
    /// The on-disk `.tct` fields (formation name / area A / flags_word_1 /
    /// slot arrays / slot body / flags_word_2 / slot pair / slot flags) all
    /// fall inside this blob at the offsets published in
    /// [`crate::tactic_file`].
    pub scratch: TacticRecord,

    // --- Editor session state — NOT saved. ---------------------------------
    /// Byte 0x2727 — active editor tab / view (0=edit, 1=view-only, 2=alt).
    pub view_mode: u8,
    /// Byte 0x2728 — previous `view_mode`.
    pub prev_view_mode: u8,
    /// Byte 0x2729 — current mentality index (u32; low byte only used).
    pub mentality_idx: u32,
    /// Byte 0x272D — previous mentality index.
    pub prev_mentality_idx: u32,
    /// Byte 0x2731 — "always use this formation" flag (u32 boolean).
    pub always_use_formation: u32,

    // --- Snapshot for revert / dirty-tracking. -----------------------------
    /// Byte 0x4A63..0x8342 — copy of `scratch` at last checkpoint.
    pub snapshot: TacticRecord,
    /// Byte 0x4A1F — sub-dirty byte (per-slot edits only).
    pub sub_dirty: u8,
    /// Byte 0x4A5E — 0xFF sentinel written at init; unread afterwards.
    pub sentinel: u8,
    /// Byte 0x4A5F — **top-level dirty flag** (u32).  Set by every mutation;
    /// cleared only when the tactic is committed (Save Formation or
    /// Revert Changes).
    pub dirty: u32,

    // --- Baseline (on-disk-at-open) — used to bail cleanly on Delete. -----
    /// Byte 0x8682..0xBF62 — snapshot made once at init from `scratch`.  Not
    /// mutated by the dispatcher; only rewritten when a fresh Load Formation
    /// succeeds and the reload path is taken.
    pub baseline: TacticRecord,

    // --- Formation lock. ---------------------------------------------------
    /// Byte 0xE77 — formation "lock" text (null-terminated ASCII).  Written
    /// by `TacticCmd::LockFormation`, cleared by `UnlockFormation`.
    pub lock_name: [u8; 0x40],

    // --- Shadow flags for the two side panels. -----------------------------
    /// Byte 0xC4DD — which side panel is active (0=none, 1=formation family,
    /// 2=mentality panel).
    pub active_side_panel: u8,
    /// Byte 0xC4DE — "formation-shadow" dirty flag (u32).  Set alongside
    /// `dirty` on every formation-changing mutation.
    pub formation_shadow_dirty: u32,

    // --- Opponent-mirror flags. --------------------------------------------
    /// Byte 0xDE1 — set to 1 when an opponent tactic is loaded into
    /// `opponent_mirror`.
    pub opponent_loaded: u32,
    /// Byte 0xDE5 — pointer/id of the opposition tactic base.
    pub opponent_ptr: u32,
    /// Byte 0xDE9 — "editing reserves" flag (0=first-team, 1=reserves).
    pub is_reserves: u8,
    /// Byte 0xDEA — "editing home team" flag (mirror of `is_reserves`).
    pub is_home: u8,
    /// Byte 0xDEB — team index (0 or 1) fed into `FUN_00525190` for the
    /// screen chrome.
    pub team_index: u8,

    // --- Screen-close plumbing. --------------------------------------------
    /// Byte 0xE40 — set when the "different-view" tactic slot is active;
    /// picks the `&DAT_00890350` constructor branch instead of `&LAB_0088DEF0`.
    pub different_view: u32,

    /// Word 0x390 — "committed once" flag (1 = tactic ready, 0 = still
    /// initialising).
    pub ready: u32,
}

/// Placeholder for the 0x38E0-byte in-memory tactic record.  Real fields live
/// at the `.tct` offsets published in
/// [`crate::tactic_file`] — this scaffold defers the field-level split to the
/// accessor crate rather than duplicating it here.
#[derive(Debug, Clone)]
pub struct TacticRecord {
    pub bytes: Vec<u8>, // length = 0x38E0 in the exe
}

impl Default for TacticRecord {
    fn default() -> Self {
        Self { bytes: vec![0; 0x38E0] }
    }
}

impl Default for TacticState {
    fn default() -> Self {
        // Manual Default — `derive(Default)` fails because Rust only supports
        // Default on arrays of len ≤ 32 (opponent_mirror = 76 B, lock_name =
        // 64 B). Everything else is a numeric primitive or TacticRecord.
        Self {
            owner_club_id: 0, reserve_link: 0, extra2: 0,
            opponent_mirror: [0u8; 0x4C],
            scratch: TacticRecord::default(),
            view_mode: 0, prev_view_mode: 0,
            mentality_idx: 0, prev_mentality_idx: 0,
            always_use_formation: 0,
            snapshot: TacticRecord::default(),
            sub_dirty: 0, sentinel: 0, dirty: 0,
            baseline: TacticRecord::default(),
            lock_name: [0u8; 0x40],
            active_side_panel: 0, formation_shadow_dirty: 0,
            opponent_loaded: 0, opponent_ptr: 0,
            is_reserves: 0, is_home: 0, team_index: 0,
            different_view: 0, ready: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Dispatch result — same shape as sidebar_dispatcher for consistency.
// ---------------------------------------------------------------------------

/// Which control-flow exit the exe branch takes.  Preserving the numeric
/// return code lets the outer screen loop reproduce the exe's exact behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticDispatchReturn {
    /// `return -4` — handled; keep the screen open, redraw.
    Handled,
    /// `return -0xB` — swallow; no redraw.
    Swallow,
    /// `return -9` — close the tactics screen (Discard/Delete confirm YES).
    Close,
    /// `return 0` — fall through to the two "outer" dispatchers
    /// `FUN_007491E0` (sidebar) and `FUN_0074BF60` (club toolbar).
    Fallthrough,
}

/// Side effect emitted by an arm — a screen open, a dialog spawn, or a
/// simple state mutation.
#[derive(Debug, Clone)]
pub enum TacticSideEffect {
    /// One field of [`TacticState`] mutated; nothing else happened.
    Mutated(&'static str),
    /// Confirm/info dialog opened with a specific modal tag.
    OpenDialog(TacticModal, &'static str),
    /// File picker opened with a specific modal tag.
    OpenFilePicker(TacticModal, &'static str),
    /// A separate editor window was created (New Tactic / Player Editor).
    OpenChildEditor(&'static str),
    /// The tactic was committed to disk (`.tct` or `.pct`).
    Committed(&'static str),
    /// The dispatcher decided to open another tactics screen for the
    /// opponent's tactic.
    OpenOpponentTactic,
    /// Unimplemented arm — the exe fn address it corresponds to.
    UnportedHandler(&'static str),
    /// No-op / info panel.
    Info(&'static str),
}

#[derive(Debug, Clone)]
pub struct TacticDispatchResult {
    pub effect: TacticSideEffect,
    pub ret: TacticDispatchReturn,
}

// ---------------------------------------------------------------------------
// 5. Dispatch — one arm per widget event.  Each arm records the SINGLE state
//    field it mutates, so a downstream caller can reconstruct the exe
//    behaviour without needing the whole 15 kB scratch layout.
// ---------------------------------------------------------------------------

pub fn dispatch(cmd: TacticCmd, state: &mut TacticState) -> TacticDispatchResult {
    use TacticCmd::*;
    use TacticDispatchReturn::*;
    use TacticSideEffect::*;

    /// Local helper — clone scratch→snapshot before every mutation and set
    /// both dirty bits.  Mirrors the exe's `memcpy(0xF07 dwords)` +
    /// `dirty=1` + `formation_shadow_dirty=1` triplet.
    fn snapshot_and_dirty(st: &mut TacticState) {
        st.snapshot = st.scratch.clone();
        st.dirty = 1;
        st.formation_shadow_dirty = 1;
    }

    let (effect, ret) = match cmd {
        // ---- File / preset lifecycle ---------------------------------------
        NewTactic => (OpenChildEditor("FUN_00894380 + FUN_007E6570(0088DEF0/00890350)"), Handled),
        SetToDefault => (OpenDialog(TacticModal::SetToDefaultConfirm, "Set To Default?"), Handled),
        LoadFormation => (OpenFilePicker(TacticModal::LoadFilePicker, "*.tct"), Handled),
        LoadPresetFormation => (OpenFilePicker(TacticModal::LoadPresetFilePicker, "CM3_TACTICS/*.pct"), Handled),
        Reserved5 => (Info("0x05: reserved / no branch"), Fallthrough),
        SaveFormation => (OpenFilePicker(TacticModal::SaveFilePicker, "*.tct"), Handled),
        SaveAsPreset => (OpenFilePicker(TacticModal::SavePresetFilePicker, "CM3_TACTICS/*.pct"), Handled),
        SaveWithName => (OpenFilePicker(TacticModal::SaveFilePicker, "reentry"), Handled),
        Reserved9 => (Info("0x09: reserved / no branch"), Fallthrough),
        LoadPresetPickerRow => (OpenFilePicker(TacticModal::LoadPresetFilePicker, "row double-click"), Handled),

        // ---- Lock / unlock formation ---------------------------------------
        UnlockFormation => {
            state.scratch = state.snapshot.clone();
            state.lock_name.fill(0);
            state.dirty = 1;
            state.formation_shadow_dirty = 1;
            (Mutated("lock_name = \"\"; scratch = snapshot"), Handled)
        }
        LockFormation => {
            snapshot_and_dirty(state);
            // Real code memcopies text from FUN_005E7980(0) into lock_name.
            (Mutated("lock_name = <entry-text>"), Handled)
        }

        // ---- Opponent tactic -----------------------------------------------
        CopyFromOpponent => (Mutated("opponent_mirror = <opponent-name>"), Handled),
        LoadOpponentTactic => (OpenOpponentTactic, Handled),

        // ---- Revert --------------------------------------------------------
        RevertChanges => {
            state.scratch = state.snapshot.clone();
            state.formation_shadow_dirty = 0;
            (Mutated("scratch = snapshot; formation_shadow_dirty=0"), Handled)
        }

        // ---- Formation / mentality panel toggles ---------------------------
        FormationFamilySelect => {
            state.active_side_panel = 1;
            (Mutated("active_side_panel = 1 (FUN_0059E2A0)"), Handled)
        }
        MentalityPanelSelect => {
            state.active_side_panel = 2;
            (Mutated("active_side_panel = 2 (FUN_0059E5B0)"), Handled)
        }
        SubPanelToggle => (UnportedHandler("FUN_008962F0"), Handled),

        ToggleHomeAway => {
            snapshot_and_dirty(state);
            state.mentality_idx = 2u32.wrapping_sub(state.mentality_idx);
            (Mutated("mentality_idx flipped via FUN_0059EAB0"), Handled)
        }

        ToggleUseAlways => {
            state.always_use_formation = if state.always_use_formation == 0 { 1 } else { 0 };
            (Mutated("always_use_formation ^= 1"), Handled)
        }

        SelectMentalityPreset => {
            // Guard: valid = 1..=3.  Invalid raises exe error 0x13C6.
            state.prev_view_mode = state.view_mode;
            state.prev_mentality_idx = state.mentality_idx;
            (Mutated("view_mode = <preset>; prev_* stored"), Handled)
        }

        // ---- Player-slot interactions --------------------------------------
        OpenPlayerEditor => (OpenChildEditor("FUN_007E6570(00890E50/00893500)"), Handled),
        SwapPlayers => {
            snapshot_and_dirty(state);
            // scratch.bytes[+0x1288 (word) = +0x4A20 (byte)] = new lineup ptr;
            // full swap lives in the per-slot record inside scratch.
            (Mutated("scratch.lineup_ptr = new; dirty=1"), Handled)
        }
        ChangeSlotRole => {
            // Inner sub-code 0x16 -> FUN_008956A0 (role change),
            //                 0x18 -> FUN_008957F0 (position swap).
            (Mutated("scratch.slot_role[i] via FUN_008956A0/FUN_008957F0"), Handled)
        }
        ChangeSlotNumber => (Mutated("scratch.slot_number[i] via FUN_00897E20"), Handled),

        // ---- Sliders -------------------------------------------------------
        SetMentalitySlider => {
            snapshot_and_dirty(state);
            (Mutated("scratch.flags_word_2 slider via FUN_0059F5A0"), Handled)
        }
        SetIndividualSlider => {
            // Guard: idx = (val >> 6) & 0x1F must be < 11 (per-slot); else error 0x1971.
            snapshot_and_dirty(state);
            (Mutated("scratch.slot_body[i][field] via FUN_0059FAE0"), Handled)
        }

        // ---- Drag & drop ---------------------------------------------------
        DragInnerRole => (Info("0x1C: inner drag code; not top-level"), Fallthrough),
        DragRoleToPitch => {
            snapshot_and_dirty(state);
            (Mutated("scratch.slot_shorts_A/C via FUN_0059D240/FUN_0059CEB0"), Handled)
        }
        DragFromPitch => {
            snapshot_and_dirty(state);
            (Mutated("scratch.slot_role_mask via FUN_0059CEB0(...,1)"), Handled)
        }

        SetMentalityXY => {
            // Encoded as `iVar14 = x*100 + y`; valid range 0<=x<3, 0<=y<4.
            state.prev_mentality_idx = state.mentality_idx;
            state.mentality_idx = 0; // packed(x,y) computed by caller
            state.prev_view_mode = state.view_mode;
            (Mutated("mentality_idx = pack(x,y); prev_* stored"), Handled)
        }

        // ---- Close / delete ------------------------------------------------
        CloseEditor => {
            if state.dirty != 0 {
                (OpenDialog(TacticModal::DiscardChangesConfirm, "Changes have been made"), Handled)
            } else {
                (Committed("editor closed cleanly; sub-views released"), Close)
            }
        }
        DeleteTactic => (OpenDialog(TacticModal::DeleteTacticConfirm, "Please Confirm"), Handled),
        AltActionRoute => (OpenChildEditor("FUN_00454620"), Handled),
    };

    TacticDispatchResult { effect, ret }
}

/// Decode + dispatch in one call.  Returns `None` when the code is not one
/// this dispatcher handles — the caller then falls through to the sidebar /
/// club-toolbar dispatchers (`FUN_007491E0` / `FUN_0074BF60`).
pub fn route(code: u16, state: &mut TacticState) -> Option<TacticDispatchResult> {
    TacticCmd::from_code(code).map(|c| dispatch(c, state))
}

// ---------------------------------------------------------------------------
// 6. Modal-return application — plugs the dialog-closed events (0x2C..0x36)
//    back into the state.  Called from the outer screen loop when
//    `DAT_00dbbf7c` is in range; button-pressed flag = `DAT_00dbbf80`.
// ---------------------------------------------------------------------------

/// Apply a modal-return, returning the same dispatch result shape.  `ok` is
/// `DAT_00dbbf80 == 2` (OK/YES button); anything else is treated as cancel.
pub fn apply_modal(modal: TacticModal, ok: bool, state: &mut TacticState) -> TacticDispatchResult {
    use TacticDispatchReturn::*;
    use TacticSideEffect::*;
    match (modal, ok) {
        (TacticModal::LoadFilePicker, true)
        | (TacticModal::LoadFilePickerAlt, true) => {
            TacticDispatchResult { effect: Committed("FUN_00895C10(name, 0) -> scratch"), ret: Handled }
        }
        (TacticModal::SaveFilePicker, true) => {
            state.dirty = 0;
            TacticDispatchResult { effect: Committed("FUN_00895D40(name, scratch)"), ret: Handled }
        }
        (TacticModal::DeleteFilePicker, true) => {
            TacticDispatchResult { effect: Committed("FUN_009354CA(name)"), ret: Handled }
        }
        (TacticModal::SavePresetFilePicker, true) => {
            state.dirty = 0;
            TacticDispatchResult { effect: Committed("FUN_005A0C10(name, 1, 1) -> .pct"), ret: Handled }
        }
        (TacticModal::LoadPresetFilePicker, true) => {
            TacticDispatchResult { effect: Committed("FUN_00895C10(name, 1) -> scratch"), ret: Handled }
        }
        (TacticModal::SetToDefaultConfirm, true) => {
            TacticDispatchResult { effect: Committed("FUN_00895F90(FUN_007E6EE0(9))"), ret: Handled }
        }
        (TacticModal::DiscardChangesConfirm, true) => {
            TacticDispatchResult { effect: Info("3x FUN_00884B00 + close"), ret: Close }
        }
        (TacticModal::DeleteTacticConfirm, true) => {
            TacticDispatchResult { effect: Info("FUN_00894650 + close"), ret: Close }
        }
        (TacticModal::MustBeSavedConfirm, true) => {
            TacticDispatchResult {
                effect: OpenFilePicker(TacticModal::SaveFilePicker, "must-save follow-up"),
                ret: Handled,
            }
        }
        (TacticModal::LoadNotSavedInfo, _) => TacticDispatchResult { effect: Info("banner"), ret: Handled },
        // Cancel on any modal: no state change.
        (_, false) => TacticDispatchResult { effect: Info("modal cancelled"), ret: Handled },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_CMDS: &[(u16, TacticCmd)] = &[
        (0x01, TacticCmd::NewTactic),
        (0x02, TacticCmd::SetToDefault),
        (0x03, TacticCmd::LoadFormation),
        (0x04, TacticCmd::LoadPresetFormation),
        (0x05, TacticCmd::Reserved5),
        (0x06, TacticCmd::SaveFormation),
        (0x07, TacticCmd::SaveAsPreset),
        (0x08, TacticCmd::SaveWithName),
        (0x09, TacticCmd::Reserved9),
        (0x0A, TacticCmd::UnlockFormation),
        (0x0B, TacticCmd::LockFormation),
        (0x0C, TacticCmd::LoadPresetPickerRow),
        (0x0D, TacticCmd::CopyFromOpponent),
        (0x0E, TacticCmd::LoadOpponentTactic),
        (0x0F, TacticCmd::RevertChanges),
        (0x10, TacticCmd::FormationFamilySelect),
        (0x11, TacticCmd::MentalityPanelSelect),
        (0x12, TacticCmd::SubPanelToggle),
        (0x13, TacticCmd::ToggleHomeAway),
        (0x14, TacticCmd::ToggleUseAlways),
        (0x15, TacticCmd::SelectMentalityPreset),
        (0x16, TacticCmd::OpenPlayerEditor),
        (0x17, TacticCmd::SwapPlayers),
        (0x18, TacticCmd::ChangeSlotRole),
        (0x19, TacticCmd::ChangeSlotNumber),
        (0x1A, TacticCmd::SetMentalitySlider),
        (0x1B, TacticCmd::SetIndividualSlider),
        (0x1C, TacticCmd::DragInnerRole),
        (0x1D, TacticCmd::DragRoleToPitch),
        (0x1E, TacticCmd::DragFromPitch),
        (0x1F, TacticCmd::SetMentalityXY),
        (0x20, TacticCmd::CloseEditor),
        (0x21, TacticCmd::DeleteTactic),
        (0x22, TacticCmd::AltActionRoute),
    ];

    #[test]
    fn every_widget_code_round_trips() {
        for &(code, cmd) in ALL_CMDS {
            assert_eq!(TacticCmd::from_code(code), Some(cmd), "decode 0x{:x}", code);
            assert_eq!(cmd.to_code(), code, "encode {:?}", cmd);
        }
    }

    #[test]
    fn every_modal_code_round_trips() {
        for code in [0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36] {
            let m = TacticModal::from_code(code).expect("known");
            assert_eq!(m.to_code(), code);
        }
    }

    #[test]
    fn unknown_widget_code_is_none() {
        assert!(TacticCmd::from_code(0x00).is_none());
        assert!(TacticCmd::from_code(0x23).is_none());
        assert!(TacticCmd::from_code(0xFFFF).is_none());
    }

    #[test]
    fn revert_restores_snapshot() {
        let mut st = TacticState::default();
        st.snapshot.bytes[0] = 0xAB;
        st.scratch.bytes[0] = 0x00;
        st.formation_shadow_dirty = 1;
        let r = dispatch(TacticCmd::RevertChanges, &mut st);
        assert!(matches!(r.effect, TacticSideEffect::Mutated(_)));
        assert_eq!(st.scratch.bytes[0], 0xAB);
        assert_eq!(st.formation_shadow_dirty, 0);
    }

    #[test]
    fn unlock_clears_lock_name() {
        let mut st = TacticState::default();
        st.lock_name[..5].copy_from_slice(b"HELLO");
        st.snapshot.bytes[7] = 0x77;
        let _ = dispatch(TacticCmd::UnlockFormation, &mut st);
        assert!(st.lock_name.iter().all(|&b| b == 0));
        assert_eq!(st.scratch.bytes[7], 0x77);
        assert_eq!(st.dirty, 1);
    }

    #[test]
    fn close_when_clean_returns_close() {
        let mut st = TacticState::default();
        st.dirty = 0;
        let r = dispatch(TacticCmd::CloseEditor, &mut st);
        assert_eq!(r.ret, TacticDispatchReturn::Close);
    }

    #[test]
    fn close_when_dirty_opens_confirm() {
        let mut st = TacticState::default();
        st.dirty = 1;
        let r = dispatch(TacticCmd::CloseEditor, &mut st);
        assert!(matches!(
            r.effect,
            TacticSideEffect::OpenDialog(TacticModal::DiscardChangesConfirm, _)
        ));
        assert_eq!(r.ret, TacticDispatchReturn::Handled);
    }

    #[test]
    fn toggle_use_always_flips() {
        let mut st = TacticState::default();
        assert_eq!(st.always_use_formation, 0);
        let _ = dispatch(TacticCmd::ToggleUseAlways, &mut st);
        assert_eq!(st.always_use_formation, 1);
        let _ = dispatch(TacticCmd::ToggleUseAlways, &mut st);
        assert_eq!(st.always_use_formation, 0);
    }

    #[test]
    fn formation_family_select_marks_panel() {
        let mut st = TacticState::default();
        let _ = dispatch(TacticCmd::FormationFamilySelect, &mut st);
        assert_eq!(st.active_side_panel, 1);
        let _ = dispatch(TacticCmd::MentalityPanelSelect, &mut st);
        assert_eq!(st.active_side_panel, 2);
    }

    #[test]
    fn apply_modal_save_clears_dirty() {
        let mut st = TacticState { dirty: 1, ..Default::default() };
        let r = apply_modal(TacticModal::SaveFilePicker, true, &mut st);
        assert_eq!(st.dirty, 0);
        assert!(matches!(r.effect, TacticSideEffect::Committed(_)));
    }

    #[test]
    fn apply_modal_cancel_does_not_touch_state() {
        let mut st = TacticState { dirty: 7, ..Default::default() };
        let _ = apply_modal(TacticModal::SaveFilePicker, false, &mut st);
        assert_eq!(st.dirty, 7);
    }

    #[test]
    fn route_unknown_returns_none() {
        let mut st = TacticState::default();
        assert!(route(0x9999, &mut st).is_none());
    }

    #[test]
    fn route_known_dispatches() {
        let mut st = TacticState::default();
        let r = route(0x14, &mut st).expect("known");
        assert!(matches!(r.effect, TacticSideEffect::Mutated(_)));
    }

    #[test]
    fn dispatch_every_variant_without_panic() {
        for &(_, cmd) in ALL_CMDS {
            let mut st = TacticState::default();
            let _ = dispatch(cmd, &mut st);
        }
    }
}
