//! Human manager pool — port of `human_manager.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\human_manager.cpp`).
//!
//! Holds the state of every human player-manager in the game: their name,
//! employment, tactics, transfer wishlist, personal mail, options. Up to
//! **16 humans** simultaneously; each takes ~683 KB of RAM.
//!
//! # Slot geometry (from FUN_005e4f50 ctor + accessors 5e5820..5eb250)
//!
//! * **Slot count**: `0x10` = 16.
//! * **Slot stride**: `0xa6ef` = 42735 bytes per human.
//! * **Total pool**: `0xa6ef4` = 683764 bytes (`4 + 16 * SLOT_STRIDE`).
//! * **Active human index**: `DAT_00b5d016` (u8).
//! * **Public seat record**: `DAT_00b59fc2 + active_human * 0xc0`.
//! * **Sub-records**: 5 tiled at 0x1ecd (7885) bytes each; the `SELECTOR`
//!   byte at slot+0x9a69 picks which one is "active" for the field accessors.
//!   Selectors 0 & 1 share the same 50-entry player-ref list (`FUN_005eaa40`
//!   family remaps selector==1 → 0); selectors 2/3/4 each have their own.
//!
//! # Decoded slot layout (compiled from the full FUN_005e5330..FUN_005eb250 sweep)
//!
//! Sub-record layout (relative to `sub_base = slot + selector * 0x1ecd`):
//! ```text
//!   +0x0000..+0x18e2  6371 B  tactics_data_blob (opaque; seeded from FUN_00881220)
//!   +0x18e3          u16      tactic_field_a
//!   +0x18e5          u16      (unnamed)
//!   +0x18e7          u16      tactic_field_b
//!   +0x18e9          u16      (unnamed)
//!   +0x18eb..+0x18ed 3 × u8   tactic_flag_{a,b,c}
//!   +0x18f1..+0x19f4 0x104 B  tactic_name (nul-term)
//!   +0x19f5..+0x1e04 4×0x104  tactic_name_history
//!   +0x1e05..+0x1ecc 50 × i32 ref_id_list (player pool indices; -1 = empty)
//! ```
//!
//! General slot fields (above the 5×sub-record tile):
//! ```text
//!   +0x9a01 u32       current_team_or_entity_id     (getter 5e6d30 / setter 5e6e30)
//!   +0x9a05 u8        flag_a                        (5e6fc0 / 5e70d0)
//!   +0x9a06 u8        flag_b                        (5e71f0 / 5e7350)
//!   +0x9a09..+0x9a58  0x50 B packed block           (single pointer view via 5e7470)
//!   +0x9a59 u32*      runtime_ptr_a                 (freed on init/load — not persisted)
//!   +0x9a5d u32*      runtime_ptr_b                 (freed on init/load — not persisted)
//!   +0x9a61 u32       unknown_counter
//!   +0x9a65 u32       counter_c                     (5e7770 / 5e7870)
//!   +0x9a69 u8        SELECTOR                      (0..4; 5e5940 sets it)
//!   +0x9a6a 0x32 B    array_A                       (5e7980)
//!   +0x9a9c 0x32 B    array_B                       (5e7a80)
//!   +0x9ace u32       current_managed_club_id       (5e7b80 / 5e7c80 — updates mgr[2])
//!   +0x9ad2 u16       week_counter                  (5e6b10 / 5e6c20 — init 0x7d)
//!   +0x9ad4..+0x9b22  0x4f B  person identity block (5e7dd0 compares, 5e8040 writes)
//!         +0x9ad4 u32     person.id
//!         +0x9ad8 u32     person.pool_a_idx
//!         +0x9ae0 u32     person.pool_b_idx_a
//!         +0x9ae4 u32     person.pool_b_idx_b
//!         +0x9ae8 ptr     cached_pool_ptr_a  (recomputed on load from person.id)
//!         +0x9aec ptr     cached_pool_ptr_b  (recomputed from pool_a_idx)
//!         +0x9af0 ptr     cached_pool_ptr_c  (recomputed from pool_b_idx_a)
//!         +0x9af4 ptr     cached_pool_ptr_d  (recomputed from pool_b_idx_b)
//!         +0x9afc i16     person.short_field_a
//!         +0x9afe u16     person.short_field_b
//!         +0x9b08 i16     person.short_field_c
//!         +0x9b13 u8      person.type_or_gender
//!   +0x9b23 u8[5]     comp_mode                     (5e8590 / 5e86a0)
//!   +0x9b28 16×{u32,i32} comp_cache                 (5e8890; populated on comp_mode[3] write)
//!   +0x9ba8..+0x9cab  0x104 B  search_name          (5e60d0 mode 2, 5e6680 mode 2)
//!   +0x9cac..+0xa0bb  4×0x104  search_name_history  (5e6280 mode 2)
//!   +0xa0bc..+0xa1bf  0x104 B  training_name        (5e60d0 mode 3, 5e6680 mode 3)
//!   +0xa1c0..+0xa5cf  4×0x104  training_name_history
//!   +0xa5d0..+0xa6d3  0x104 B  print_name           (5e60d0 mode 0)
//!   +0xa6d4..+0xa6e2  0xf B    misc (u32/u32/u32/u16/u8) (5e8490 exposes as one block)
//!   +0xa6e3 ptr       dyn_records_ptr               (malloc'd on load; 5ea120/5ea280 search)
//!   +0xa6e7 u32       dyn_records_count_mirror
//!   +0xa6eb u32       dyn_records_count             (persisted; each record is 0x1e bytes)
//! ```
//!
//! # Save/load format (FUN_005e8990 load, FUN_005e9520 save)
//!
//! Both open `human_manager.dat` via `FUN_00921770(mgr, "human_manager.dat",
//! mode, version=0x16, compression=4)` — the standard chunked container.
//!
//! ```text
//!   [header written by FUN_00921770; magic = "human_manager.dat", version 0x16]
//!   u32  slot_count                            (== mgr.slots.len())
//!   for i in 0..slot_count:
//!     slot[i][0 .. 2*0x1ecd] raw               (sub-records 0..1)
//!     slot[i][+0x9a01 .. +0x9b22] general      (identity + counters)
//!     slot[i][+0x9b23 .. +0x9b27] comp_mode[5]
//!     slot[i][+0x9b28 .. +0x9ba7] comp_cache[16] {u32,i32}
//!       (legacy: version < 0x3e005 stored only 16*u32; loader synthesises -1 tails)
//!     slot[i][+0x9ba8 .. +0xa6ea] mid-block    (names, misc)
//!     u32 dyn_records_count = slot[i][+0xa6eb]
//!     dyn_records_count * 0x1e bytes           (pointer stored at +0xa6e3)
//!   u32  mgr.counter_a                         (mgr[2])
//!   u32  mgr.counter_b                         (mgr[3])
//!   for i in 0..slot_count:
//!     slot[i][+0x3d9a .. +0x9a00] raw          (sub-records 2..4)
//! ```
//!
//! Loader post-processing: zero `runtime_ptr_a/b` (+0x9a59, +0x9a5d);
//! recompute `cached_pool_ptr_{a,b,c,d}` from person indices when +0x9ae8 != 0
//! (bases: `DAT_00acd5d8` × 0x6b, `DAT_00acd5b8` × 0x4e, `DAT_00acd5bc` × 0x245).
//!
//! Legacy fork (version < 0x3e021): sub-records were 5 × 0x1e75 with a
//! different internal shape; loader translates by copying the head bytes
//! straight and re-materialising an 11-entry list of 8-byte pairs at
//! `sub+0x188b/8f` plus the 400 × u32 trailing block at `sub+0x18e3`.
//!
//! # Port scope
//!
//! Portable HERE (this file):
//!   * Slot geometry constants (see below)
//!   * Sub-record and general-field offset constants (SUB_* / SLOT_*)
//!   * Typed accessors for the ~15 scalar fields with clear semantics
//!   * `HumanSlot::sub_record_at` — resolves the (selector, offset) pair
//!   * `HumanPool::empty()` — zero-init 16-slot pool
//!
//! Deferred (documented, not yet coded):
//!   * The chunked-container reader/writer (`FUN_00921770`) — a save-format
//!     dependency that also blocks all other `.dat`-backed subsystems
//!   * Cached pool-ptr recomputation (needs the pool bases DAT_00acd5b8/bc/d8
//!     wired through)
//!   * Legacy-version fork translations (< 0x3e021, < 0x3e005, < 0x3e001)
//!   * The comp_cache populate side-effect (FUN_008815a0 iteration in
//!     comp_mode[3] setter)
//!   * The person-add validation chain (FUN_005e8040 / 5e8240 / 5ea590 /
//!     5ea720) — needs the club-chain walker substrate

use serde::{Deserialize, Serialize};

// ── Pool geometry ─────────────────────────────────────────────────────────

/// Maximum concurrent human managers — FUN_005e4f50 hard-codes 0x10 = 16.
pub const HUMAN_MAX_SLOTS: usize = 0x10;
/// Bytes per human slot — FUN_005e4f50 mallocs `4 + 16 * SLOT_STRIDE`.
pub const HUMAN_SLOT_STRIDE: usize = 0xa6ef;
/// Bytes per public "seat" record — `DAT_00b59fc2 + active_human * SEAT_STRIDE`.
pub const HUMAN_SEAT_STRIDE: usize = 0xc0;
/// Bytes per sub-record inside a slot — 5 tiled at 0x1ecd each.
pub const HUMAN_SUB_RECORD_STRIDE: usize = 0x1ecd;
/// Number of tiled sub-records per slot.
pub const HUMAN_SUB_RECORDS_PER_SLOT: usize = 5;

// ── Sub-record field offsets (relative to sub_base) ───────────────────────

pub const SUB_TACTICS_DATA: usize     = 0x0000;
pub const SUB_TACTICS_DATA_LEN: usize = 0x18e3;
pub const SUB_TACTIC_FIELD_A: usize   = 0x18e3;
pub const SUB_TACTIC_FIELD_B: usize   = 0x18e7;
pub const SUB_TACTIC_FLAG_A: usize    = 0x18eb;
pub const SUB_TACTIC_FLAG_B: usize    = 0x18ec;
pub const SUB_TACTIC_FLAG_C: usize    = 0x18ed;
pub const SUB_TACTIC_NAME: usize      = 0x18f1;
pub const SUB_TACTIC_NAME_LEN: usize  = 0x104;
pub const SUB_TACTIC_NAME_HISTORY: usize = 0x19f5;
pub const SUB_TACTIC_NAME_HISTORY_SLOTS: usize = 4;
/// 50-entry i32 array; -1 = empty. Selector==1 aliases to selector==0's list.
pub const SUB_REF_ID_LIST: usize      = 0x1e05;
pub const SUB_REF_ID_LIST_LEN: usize  = 0x32;

// ── General slot field offsets ────────────────────────────────────────────

pub const SLOT_CURRENT_TEAM_OR_ENTITY_ID: usize = 0x9a01;
pub const SLOT_FLAG_A: usize                    = 0x9a05;
pub const SLOT_FLAG_B: usize                    = 0x9a06;
pub const SLOT_PACKED_BLOCK: usize              = 0x9a09;
pub const SLOT_PACKED_BLOCK_LEN: usize          = 0x50;
pub const SLOT_RUNTIME_PTR_A: usize             = 0x9a59;
pub const SLOT_RUNTIME_PTR_B: usize             = 0x9a5d;
pub const SLOT_UNKNOWN_COUNTER: usize           = 0x9a61;
pub const SLOT_COUNTER_C: usize                 = 0x9a65;
pub const SLOT_SELECTOR: usize                  = 0x9a69;
pub const SLOT_ARRAY_A: usize                   = 0x9a6a;
pub const SLOT_ARRAY_A_LEN: usize               = 0x32;
pub const SLOT_ARRAY_B: usize                   = 0x9a9c;
pub const SLOT_ARRAY_B_LEN: usize               = 0x32;
pub const SLOT_CURRENT_MANAGED_CLUB_ID: usize   = 0x9ace;
pub const SLOT_WEEK_COUNTER: usize              = 0x9ad2;
pub const SLOT_WEEK_COUNTER_INIT: u16           = 0x7d;
pub const SLOT_PERSON_BLOCK: usize              = 0x9ad4;
pub const SLOT_PERSON_BLOCK_LEN: usize          = 0x4f;
pub const SLOT_PERSON_ID: usize                 = 0x9ad4;
pub const SLOT_PERSON_POOL_A_IDX: usize         = 0x9ad8;
pub const SLOT_PERSON_POOL_B_IDX_A: usize       = 0x9ae0;
pub const SLOT_PERSON_POOL_B_IDX_B: usize       = 0x9ae4;
pub const SLOT_CACHED_POOL_PTR_A: usize         = 0x9ae8;
pub const SLOT_CACHED_POOL_PTR_B: usize         = 0x9aec;
pub const SLOT_CACHED_POOL_PTR_C: usize         = 0x9af0;
pub const SLOT_CACHED_POOL_PTR_D: usize         = 0x9af4;
pub const SLOT_PERSON_SHORT_FIELD_A: usize      = 0x9afc;
pub const SLOT_PERSON_SHORT_FIELD_B: usize      = 0x9afe;
pub const SLOT_PERSON_SHORT_FIELD_C: usize      = 0x9b08;
pub const SLOT_PERSON_TYPE_OR_GENDER: usize     = 0x9b13;
pub const SLOT_COMP_MODE: usize                 = 0x9b23;
pub const SLOT_COMP_MODE_LEN: usize             = 5;
pub const SLOT_COMP_CACHE: usize                = 0x9b28;
pub const SLOT_COMP_CACHE_ENTRIES: usize        = 16;
pub const SLOT_COMP_CACHE_ENTRY_BYTES: usize    = 8;
pub const SLOT_SEARCH_NAME: usize               = 0x9ba8;
pub const SLOT_SEARCH_NAME_LEN: usize           = 0x104;
pub const SLOT_SEARCH_NAME_HISTORY: usize       = 0x9cac;
pub const SLOT_TRAINING_NAME: usize             = 0xa0bc;
pub const SLOT_TRAINING_NAME_LEN: usize         = 0x104;
pub const SLOT_TRAINING_NAME_HISTORY: usize     = 0xa1c0;
pub const SLOT_PRINT_NAME: usize                = 0xa5d0;
pub const SLOT_PRINT_NAME_LEN: usize            = 0x104;
pub const SLOT_MISC_BLOCK: usize                = 0xa6d4;
pub const SLOT_MISC_BLOCK_LEN: usize            = 0x0f;
pub const SLOT_DYN_RECORDS_PTR: usize           = 0xa6e3;
pub const SLOT_DYN_RECORDS_COUNT_MIRROR: usize  = 0xa6e7;
pub const SLOT_DYN_RECORDS_COUNT: usize         = 0xa6eb;
pub const DYN_RECORD_STRIDE: usize              = 0x1e;

// ── Sentinel values ───────────────────────────────────────────────────────

/// Empty ref-id-list slot (from FUN_005e5330 init: fills with `-1`).
pub const REF_ID_EMPTY: i32 = -1;

// ── Data ──────────────────────────────────────────────────────────────────

/// A single human's state — 42735 bytes of slot content plus a lazily-parsed
/// view.
#[derive(Clone, Serialize, Deserialize)]
pub struct HumanSlot {
    pub raw: Vec<u8>,
}

impl HumanSlot {
    /// FUN_005e4f50's memset — allocates a zero-filled slot.
    pub fn zeroed() -> Self {
        Self { raw: vec![0u8; HUMAN_SLOT_STRIDE] }
    }

    /// Value of the sub-record selector byte at `+0x9a69`.
    pub fn selector(&self) -> u8 {
        self.raw[SLOT_SELECTOR]
    }

    /// Sub-record index used for the ref_id_list — matches the exe's
    /// `FUN_005eaa40` family remap: selector 1 aliases to 0.
    pub fn ref_list_selector(&self) -> u8 {
        match self.selector() {
            1 => 0,
            s => s,
        }
    }

    /// Read-only view of one 0x1ecd-byte sub-record.
    pub fn sub_record(&self, i: usize) -> Option<&[u8]> {
        if i >= HUMAN_SUB_RECORDS_PER_SLOT { return None; }
        let off = i * HUMAN_SUB_RECORD_STRIDE;
        Some(&self.raw[off .. off + HUMAN_SUB_RECORD_STRIDE])
    }

    /// Mutable view of one 0x1ecd-byte sub-record.
    pub fn sub_record_mut(&mut self, i: usize) -> Option<&mut [u8]> {
        if i >= HUMAN_SUB_RECORDS_PER_SLOT { return None; }
        let off = i * HUMAN_SUB_RECORD_STRIDE;
        Some(&mut self.raw[off .. off + HUMAN_SUB_RECORD_STRIDE])
    }

    /// The currently active sub-record (chosen by `selector`).
    pub fn active_sub_record(&self) -> &[u8] {
        self.sub_record(self.selector() as usize).unwrap_or(&self.raw[..0])
    }

    // ── Typed general-field accessors ────────────────────────────────────

    pub fn current_managed_club_id(&self) -> u32 { read_u32(&self.raw, SLOT_CURRENT_MANAGED_CLUB_ID) }
    pub fn set_current_managed_club_id(&mut self, v: u32) { write_u32(&mut self.raw, SLOT_CURRENT_MANAGED_CLUB_ID, v); }
    pub fn current_team_or_entity_id(&self) -> u32 { read_u32(&self.raw, SLOT_CURRENT_TEAM_OR_ENTITY_ID) }
    pub fn set_current_team_or_entity_id(&mut self, v: u32) { write_u32(&mut self.raw, SLOT_CURRENT_TEAM_OR_ENTITY_ID, v); }
    pub fn week_counter(&self) -> u16 { read_u16(&self.raw, SLOT_WEEK_COUNTER) }
    pub fn set_week_counter(&mut self, v: u16) { write_u16(&mut self.raw, SLOT_WEEK_COUNTER, v); }
    pub fn flag_a(&self) -> bool { self.raw[SLOT_FLAG_A] != 0 }
    pub fn flag_b(&self) -> bool { self.raw[SLOT_FLAG_B] != 0 }
    pub fn counter_c(&self) -> u32 { read_u32(&self.raw, SLOT_COUNTER_C) }
    pub fn set_counter_c(&mut self, v: u32) { write_u32(&mut self.raw, SLOT_COUNTER_C, v); }
    pub fn person_id(&self) -> u32 { read_u32(&self.raw, SLOT_PERSON_ID) }

    /// `comp_mode[idx]` (0..5).
    pub fn comp_mode(&self, idx: usize) -> Option<u8> {
        if idx >= SLOT_COMP_MODE_LEN { return None; }
        Some(self.raw[SLOT_COMP_MODE + idx])
    }
    pub fn set_comp_mode(&mut self, idx: usize, v: u8) {
        if idx < SLOT_COMP_MODE_LEN { self.raw[SLOT_COMP_MODE + idx] = v; }
    }

    /// Read one entry of `comp_cache` (returns `(comp_id, aux)`).
    pub fn comp_cache_entry(&self, i: usize) -> Option<(u32, i32)> {
        if i >= SLOT_COMP_CACHE_ENTRIES { return None; }
        let off = SLOT_COMP_CACHE + i * SLOT_COMP_CACHE_ENTRY_BYTES;
        Some((read_u32(&self.raw, off), read_u32(&self.raw, off + 4) as i32))
    }

    /// One `i32` from the active sub-record's 50-entry ref_id_list.
    /// Selector==1 aliases to selector==0 (mirrors FUN_005eaa40).
    pub fn ref_id_at(&self, i: usize) -> Option<i32> {
        if i >= SUB_REF_ID_LIST_LEN { return None; }
        let sub = self.sub_record(self.ref_list_selector() as usize)?;
        let off = SUB_REF_ID_LIST + i * 4;
        Some(read_i32(sub, off))
    }
    pub fn set_ref_id_at(&mut self, i: usize, v: i32) {
        if i >= SUB_REF_ID_LIST_LEN { return; }
        let sel = self.ref_list_selector() as usize;
        if let Some(sub) = self.sub_record_mut(sel) {
            let off = SUB_REF_ID_LIST + i * 4;
            write_i32(sub, off, v);
        }
    }

    /// Read the tactic_name of the currently selected sub-record (nul-terminated).
    pub fn active_tactic_name(&self) -> String {
        read_cstring(self.active_sub_record(), SUB_TACTIC_NAME, SUB_TACTIC_NAME_LEN)
    }
    pub fn search_name(&self) -> String { read_cstring(&self.raw, SLOT_SEARCH_NAME, SLOT_SEARCH_NAME_LEN) }
    pub fn training_name(&self) -> String { read_cstring(&self.raw, SLOT_TRAINING_NAME, SLOT_TRAINING_NAME_LEN) }
    pub fn print_name(&self) -> String { read_cstring(&self.raw, SLOT_PRINT_NAME, SLOT_PRINT_NAME_LEN) }

    /// Count of dyn-records at +0xa6e3 (each 0x1e bytes).
    pub fn dyn_records_count(&self) -> u32 { read_u32(&self.raw, SLOT_DYN_RECORDS_COUNT) }
}

impl std::fmt::Debug for HumanSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HumanSlot")
            .field("bytes", &self.raw.len())
            .field("selector", &self.selector())
            .field("current_managed_club_id", &self.current_managed_club_id())
            .field("current_team_or_entity_id", &self.current_team_or_entity_id())
            .field("person_id", &self.person_id())
            .field("week_counter", &self.week_counter())
            .field("dyn_records_count", &self.dyn_records_count())
            .finish()
    }
}

/// The 16-slot human manager pool + the active-human index.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HumanPool {
    pub slots: Vec<HumanSlot>,
    /// Ports `DAT_00b5d016`.
    pub active: u8,
    /// Mirrors mgr[2] — a running counter adjusted by set_current_managed_club_id.
    pub counter_a: u32,
    /// Mirrors mgr[3] — running counter adjusted by set_current_team_or_entity_id.
    pub counter_b: u32,
}

impl HumanPool {
    /// Empty pool with all 16 slots zeroed; each slot pre-initialised with
    /// the FUN_005e5330 defaults for week_counter, ref_id_list, etc.
    pub fn empty() -> Self {
        let mut slots = Vec::with_capacity(HUMAN_MAX_SLOTS);
        for _ in 0..HUMAN_MAX_SLOTS {
            let mut s = HumanSlot::zeroed();
            s.set_week_counter(SLOT_WEEK_COUNTER_INIT);
            // FUN_005e5330 seeds every sub-record's ref_id_list with -1s and
            // sets the tactic flag defaults (a=1, b=1, c=0).
            for i in 0..HUMAN_SUB_RECORDS_PER_SLOT {
                let sub = s.sub_record_mut(i).unwrap();
                sub[SUB_TACTIC_FLAG_A] = 1;
                sub[SUB_TACTIC_FLAG_B] = 1;
                sub[SUB_TACTIC_FLAG_C] = 0;
                for j in 0..SUB_REF_ID_LIST_LEN {
                    write_i32(sub, SUB_REF_ID_LIST + j * 4, REF_ID_EMPTY);
                }
            }
            slots.push(s);
        }
        Self { slots, active: 0, counter_a: 0, counter_b: 0 }
    }

    pub fn active_slot(&self) -> Option<&HumanSlot> {
        self.slots.get(self.active as usize)
    }
    pub fn active_slot_mut(&mut self) -> Option<&mut HumanSlot> {
        self.slots.get_mut(self.active as usize)
    }
}

// ── Byte helpers ──────────────────────────────────────────────────────────

fn read_u32(b: &[u8], o: usize) -> u32 { u32::from_le_bytes([b[o], b[o+1], b[o+2], b[o+3]]) }
fn read_i32(b: &[u8], o: usize) -> i32 { i32::from_le_bytes([b[o], b[o+1], b[o+2], b[o+3]]) }
fn read_u16(b: &[u8], o: usize) -> u16 { u16::from_le_bytes([b[o], b[o+1]]) }
fn write_u32(b: &mut [u8], o: usize, v: u32) { b[o..o+4].copy_from_slice(&v.to_le_bytes()); }
fn write_i32(b: &mut [u8], o: usize, v: i32) { b[o..o+4].copy_from_slice(&v.to_le_bytes()); }
fn write_u16(b: &mut [u8], o: usize, v: u16) { b[o..o+2].copy_from_slice(&v.to_le_bytes()); }
fn read_cstring(b: &[u8], off: usize, max: usize) -> String {
    let end = off + max;
    let s = &b[off..end.min(b.len())];
    let n = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    String::from_utf8_lossy(&s[..n]).into_owned()
}

// ── Public seat pool (DAT_00b59fc0, stride 0xc0) ──────────────────────────
//
// The seat is the *narrow* per-human record used everywhere UI code reads
// "the current human" from — dashboards, toolbars, message routing. It sits
// alongside the 42735-byte HumanSlot but is addressed independently:
//
//   base   = DAT_00b59fc0
//   stride = 0xc0
//   count  = 16   (matches HUMAN_MAX_SLOTS)
//   active = DAT_00b5d016 (u8)
//
// From the memory decode:
//   +0x02  ptr  person_ptr    (DAT_00b59fc2 in the exe: base+2 = seat[0].person)
//   +0x28  ptr  big_ui_ptr    (DAT_00b59fe8 = base+0x28)
//
// The rest of the 0xC0 window holds the seat's cached employment view
// (club, joined/contract dates, wage, club reputation, employment flags).
// The exe keeps the *authoritative* employment on the Person record
// (person+0x39 club link, +0x24 nation link, +0x5f human_status,
// +0x3d club job-status). The seat is the projection UI code renders.

/// Bytes per seat public record (see HUMAN_SEAT_STRIDE).
pub const SEAT_STRIDE: usize = HUMAN_SEAT_STRIDE;
/// Seat count — matches human slot count.
pub const SEAT_MAX: usize = HUMAN_MAX_SLOTS;

// Seat field offsets. Only person_ptr and big_ui_ptr are pinned by decode
// (base+2 = DAT_00b59fc2, base+0x28 = DAT_00b59fe8). The remaining fields
// occupy documented gaps in the 0xC0 window and mirror the Person-record
// employment view.
pub const SEAT_PERSON_PTR: usize        = 0x02;
pub const SEAT_CLUB_ID: usize           = 0x08;
pub const SEAT_NATION_ID: usize         = 0x0c;
pub const SEAT_JOINED_DATE: usize       = 0x10;   // days since epoch
pub const SEAT_CONTRACT_END: usize      = 0x14;   // days since epoch
pub const SEAT_WAGE: usize              = 0x18;   // weekly wage (cash units)
pub const SEAT_REPUTATION: usize        = 0x1c;   // club rep snapshot (u16)
pub const SEAT_EMPLOYMENT_FLAGS: usize  = 0x1e;   // u8 bitfield (see EF_*)
pub const SEAT_JOB_STATUS: usize        = 0x1f;   // u8: 5=manager,6=asst,8=coach,0xb=player,0xc=player-mgr
pub const SEAT_BIG_UI_PTR: usize        = 0x28;

// Employment flag bits (packed on person+0x5f semantics, projected to seat).
pub const EF_OCCUPIED: u8    = 1 << 0;   // this seat is taken by a human
pub const EF_EMPLOYED: u8    = 1 << 1;   // holds a job (club OR nation)
pub const EF_SACKED: u8      = 1 << 2;   // terminated by board; seat kept
pub const EF_RESIGNED: u8    = 1 << 3;   // voluntary resignation
pub const EF_WAITING_JOB: u8 = 1 << 4;   // unemployed + actively applying

/// Job-status codes on person+0x3d / seat+0x1f (from dashboard-manager-model memory).
pub mod job_status {
    pub const NONE: u8         = 0;
    pub const MANAGER: u8      = 5;
    pub const ASSISTANT: u8    = 6;
    pub const COACH: u8        = 8;
    pub const PLAYER: u8       = 0xb;
    pub const PLAYER_MGR: u8   = 0xc;
    pub const PLAYER_NAT: u8   = 0xf;
}

/// Outcome of a `take_control` / seat operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeatError {
    /// `seat_idx >= SEAT_MAX`.
    OutOfRange(usize),
    /// Seat already has EF_OCCUPIED set for a different person.
    AlreadyOccupied,
    /// Seat is empty (no EF_OCCUPIED).
    NotOccupied,
    /// Ported from FUN_005ea720 pre-check: this club is already managed by
    /// another human.
    ClubAlreadyHumanManaged,
}

/// One 0xC0-byte seat record. Backed by a raw byte buffer so the on-disk
/// layout is preserved bit-for-bit; typed accessors on top.
#[derive(Clone, Serialize, Deserialize)]
pub struct SeatSlot {
    pub raw: Vec<u8>,
}

impl SeatSlot {
    pub fn zeroed() -> Self { Self { raw: vec![0u8; SEAT_STRIDE] } }

    // ── pointer/id accessors (person_ptr stored as u32 person-id in Rust) ──
    pub fn person_id(&self) -> u32 { read_u32(&self.raw, SEAT_PERSON_PTR) }
    pub fn set_person_id(&mut self, v: u32) { write_u32(&mut self.raw, SEAT_PERSON_PTR, v); }
    pub fn big_ui_id(&self) -> u32 { read_u32(&self.raw, SEAT_BIG_UI_PTR) }
    pub fn set_big_ui_id(&mut self, v: u32) { write_u32(&mut self.raw, SEAT_BIG_UI_PTR, v); }

    pub fn club_id(&self) -> u32 { read_u32(&self.raw, SEAT_CLUB_ID) }
    pub fn set_club_id(&mut self, v: u32) { write_u32(&mut self.raw, SEAT_CLUB_ID, v); }
    pub fn nation_id(&self) -> u32 { read_u32(&self.raw, SEAT_NATION_ID) }
    pub fn set_nation_id(&mut self, v: u32) { write_u32(&mut self.raw, SEAT_NATION_ID, v); }

    pub fn joined_date(&self) -> u32 { read_u32(&self.raw, SEAT_JOINED_DATE) }
    pub fn set_joined_date(&mut self, v: u32) { write_u32(&mut self.raw, SEAT_JOINED_DATE, v); }
    pub fn contract_end(&self) -> u32 { read_u32(&self.raw, SEAT_CONTRACT_END) }
    pub fn set_contract_end(&mut self, v: u32) { write_u32(&mut self.raw, SEAT_CONTRACT_END, v); }
    pub fn wage(&self) -> u32 { read_u32(&self.raw, SEAT_WAGE) }
    pub fn set_wage(&mut self, v: u32) { write_u32(&mut self.raw, SEAT_WAGE, v); }
    pub fn reputation(&self) -> u16 { read_u16(&self.raw, SEAT_REPUTATION) }
    pub fn set_reputation(&mut self, v: u16) { write_u16(&mut self.raw, SEAT_REPUTATION, v); }

    pub fn employment_flags(&self) -> u8 { self.raw[SEAT_EMPLOYMENT_FLAGS] }
    pub fn set_employment_flags(&mut self, v: u8) { self.raw[SEAT_EMPLOYMENT_FLAGS] = v; }
    pub fn job_status(&self) -> u8 { self.raw[SEAT_JOB_STATUS] }
    pub fn set_job_status(&mut self, v: u8) { self.raw[SEAT_JOB_STATUS] = v; }

    pub fn is_occupied(&self) -> bool { self.employment_flags() & EF_OCCUPIED != 0 }
    pub fn is_employed(&self) -> bool { self.employment_flags() & EF_EMPLOYED != 0 }
    pub fn is_sacked(&self)   -> bool { self.employment_flags() & EF_SACKED   != 0 }
    pub fn is_resigned(&self) -> bool { self.employment_flags() & EF_RESIGNED != 0 }
    pub fn is_unemployed(&self) -> bool {
        self.is_occupied() && self.club_id() == 0 && self.nation_id() == 0
    }
}

impl std::fmt::Debug for SeatSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SeatSlot")
            .field("person_id", &self.person_id())
            .field("club_id", &self.club_id())
            .field("nation_id", &self.nation_id())
            .field("wage", &self.wage())
            .field("reputation", &self.reputation())
            .field("employment_flags", &format_args!("{:#010b}", self.employment_flags()))
            .field("job_status", &self.job_status())
            .finish()
    }
}

/// The 16-seat public human record pool (DAT_00b59fc0 + i * 0xC0) plus the
/// `active_human` index (DAT_00b5d016).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HumanSeatPool {
    pub seats: Vec<SeatSlot>,
    /// Ports `DAT_00b5d016` — index of the human whose UI is currently rendered.
    pub active_human: u8,
}

impl Default for HumanSeatPool {
    fn default() -> Self { Self::empty() }
}

impl HumanSeatPool {
    /// Empty pool: 16 zeroed seats, active_human=0.
    pub fn empty() -> Self {
        Self {
            seats: (0..SEAT_MAX).map(|_| SeatSlot::zeroed()).collect(),
            active_human: 0,
        }
    }

    // ── active_human accessors (DAT_00b5d016) ────────────────────────────
    pub fn active_human(&self) -> u8 { self.active_human }
    /// Set the active human index (mirrors a plain write to DAT_00b5d016).
    /// Returns Err if out of range or seat is not occupied.
    pub fn switch_active(&mut self, seat_idx: u8) -> Result<(), SeatError> {
        let i = seat_idx as usize;
        if i >= SEAT_MAX { return Err(SeatError::OutOfRange(i)); }
        if !self.seats[i].is_occupied() { return Err(SeatError::NotOccupied); }
        self.active_human = seat_idx;
        Ok(())
    }
    pub fn active_seat(&self) -> Option<&SeatSlot> { self.seats.get(self.active_human as usize) }
    pub fn active_seat_mut(&mut self) -> Option<&mut SeatSlot> { self.seats.get_mut(self.active_human as usize) }

    /// First free (not EF_OCCUPIED) seat index, if any.
    pub fn first_free_slot(&self) -> Option<usize> {
        self.seats.iter().position(|s| !s.is_occupied())
    }

    /// "Take Control" — toolbar cmd 100 → FUN_0080bbd0 → FUN_00810f50.
    /// Seats the given person at the club: sets EF_OCCUPIED | EF_EMPLOYED,
    /// stores person/club ids, job_status=MANAGER, joined_date, wage.
    /// If `seat_idx` is `None`, uses the first free slot.
    pub fn take_control(
        &mut self,
        seat_idx: Option<usize>,
        person_id: u32,
        club_id: u32,
        today: u32,
        wage: u32,
        reputation: u16,
    ) -> Result<usize, SeatError> {
        // Reject if another OCCUPIED seat already holds this club.
        if club_id != 0 {
            for (i, s) in self.seats.iter().enumerate() {
                if s.is_occupied() && s.club_id() == club_id
                    && (seat_idx.is_none() || seat_idx != Some(i))
                {
                    return Err(SeatError::ClubAlreadyHumanManaged);
                }
            }
        }
        let idx = match seat_idx {
            Some(i) => {
                if i >= SEAT_MAX { return Err(SeatError::OutOfRange(i)); }
                let s = &self.seats[i];
                if s.is_occupied() && s.person_id() != person_id {
                    return Err(SeatError::AlreadyOccupied);
                }
                i
            }
            None => self.first_free_slot().ok_or(SeatError::AlreadyOccupied)?,
        };
        let s = &mut self.seats[idx];
        s.set_person_id(person_id);
        s.set_club_id(club_id);
        s.set_nation_id(0);
        s.set_joined_date(today);
        s.set_contract_end(0);
        s.set_wage(wage);
        s.set_reputation(reputation);
        s.set_job_status(job_status::MANAGER);
        // Set occupied+employed; clear sacked/resigned/waiting.
        s.set_employment_flags(EF_OCCUPIED | EF_EMPLOYED);
        Ok(idx)
    }

    /// Resign — dialog handler 0x00698480 → FUN_006809e0(_,3,0,0).
    /// Vacates club+nation, marks EF_RESIGNED, clears EF_EMPLOYED. Seat stays
    /// EF_OCCUPIED (person still exists, just unemployed).
    pub fn resign(&mut self, seat_idx: usize) -> Result<(), SeatError> {
        if seat_idx >= SEAT_MAX { return Err(SeatError::OutOfRange(seat_idx)); }
        let s = &mut self.seats[seat_idx];
        if !s.is_occupied() { return Err(SeatError::NotOccupied); }
        s.set_club_id(0);
        s.set_nation_id(0);
        s.set_contract_end(0);
        s.set_wage(0);
        s.set_job_status(job_status::NONE);
        let mut f = s.employment_flags();
        f &= !EF_EMPLOYED;
        f &= !EF_SACKED;
        f |= EF_RESIGNED;
        s.set_employment_flags(f);
        Ok(())
    }

    /// Sack — FUN_006808a0 / FUN_0066eed0 chain via FUN_006809e0 from board
    /// confidence. Same vacate as resign but marks EF_SACKED instead.
    /// Seat is terminated: EF_OCCUPIED is cleared so the slot can be reused.
    pub fn sack(&mut self, seat_idx: usize) -> Result<(), SeatError> {
        if seat_idx >= SEAT_MAX { return Err(SeatError::OutOfRange(seat_idx)); }
        let s = &mut self.seats[seat_idx];
        if !s.is_occupied() { return Err(SeatError::NotOccupied); }
        s.set_club_id(0);
        s.set_nation_id(0);
        s.set_contract_end(0);
        s.set_wage(0);
        s.set_job_status(job_status::NONE);
        let mut f = s.employment_flags();
        f &= !EF_EMPLOYED;
        f &= !EF_RESIGNED;
        f &= !EF_OCCUPIED;   // sack terminates the seat
        f |= EF_SACKED;
        s.set_employment_flags(f);
        // If we sacked the active human, drop back to seat 0.
        if self.active_human as usize == seat_idx {
            self.active_human = 0;
        }
        Ok(())
    }

    /// Returns `true` if any human seat holds this club (ports FUN_005ea590).
    pub fn is_club_human_managed(&self, club_id: u32) -> bool {
        club_id != 0 && self.seats.iter().any(|s| s.is_occupied() && s.club_id() == club_id)
    }

    /// Returns `true` if the *currently active* human manages this club
    /// (ports FUN_005ea720(club, 0, 1)).
    pub fn is_managed_by_active(&self, club_id: u32) -> bool {
        club_id != 0 && self.active_seat().is_some_and(|s| s.is_occupied() && s.club_id() == club_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_geometry() {
        let p = HumanPool::empty();
        assert_eq!(p.slots.len(), HUMAN_MAX_SLOTS);
        assert!(p.slots.iter().all(|s| s.raw.len() == HUMAN_SLOT_STRIDE));
        assert_eq!(HUMAN_SLOT_STRIDE, 0xa6ef);
        assert_eq!(HUMAN_MAX_SLOTS * HUMAN_SLOT_STRIDE + 4, 0xa6ef4);
    }

    #[test]
    fn selector_alias_1_to_0() {
        let mut s = HumanSlot::zeroed();
        s.raw[SLOT_SELECTOR] = 1;
        assert_eq!(s.selector(), 1);
        assert_eq!(s.ref_list_selector(), 0);
    }

    #[test]
    fn scalars_round_trip() {
        let mut s = HumanSlot::zeroed();
        s.set_current_managed_club_id(0x1122_3344);
        s.set_current_team_or_entity_id(0xdead_beef);
        s.set_week_counter(0x0202);
        s.set_counter_c(42);
        assert_eq!(s.current_managed_club_id(), 0x1122_3344);
        assert_eq!(s.current_team_or_entity_id(), 0xdead_beef);
        assert_eq!(s.week_counter(), 0x0202);
        assert_eq!(s.counter_c(), 42);
    }

    #[test]
    fn ref_id_list_defaults_to_neg1_across_all_sub_records() {
        let p = HumanPool::empty();
        let s = &p.slots[0];
        for sub_ix in 0..HUMAN_SUB_RECORDS_PER_SLOT {
            let sub = s.sub_record(sub_ix).unwrap();
            for j in 0..SUB_REF_ID_LIST_LEN {
                assert_eq!(read_i32(sub, SUB_REF_ID_LIST + j * 4), REF_ID_EMPTY,
                    "sub {} slot {}", sub_ix, j);
            }
        }
    }

    #[test]
    fn tactic_flag_defaults() {
        let p = HumanPool::empty();
        let s = &p.slots[0];
        for sub_ix in 0..HUMAN_SUB_RECORDS_PER_SLOT {
            let sub = s.sub_record(sub_ix).unwrap();
            assert_eq!(sub[SUB_TACTIC_FLAG_A], 1);
            assert_eq!(sub[SUB_TACTIC_FLAG_B], 1);
            assert_eq!(sub[SUB_TACTIC_FLAG_C], 0);
        }
    }

    #[test]
    fn week_counter_default_is_0x7d() {
        let p = HumanPool::empty();
        assert_eq!(p.slots[0].week_counter(), SLOT_WEEK_COUNTER_INIT);
    }

    #[test]
    fn comp_cache_shape() {
        let mut s = HumanSlot::zeroed();
        // Write to entry 3 and read back.
        let off = SLOT_COMP_CACHE + 3 * SLOT_COMP_CACHE_ENTRY_BYTES;
        write_u32(&mut s.raw, off, 0xaaaa_bbbb);
        write_i32(&mut s.raw, off + 4, -7);
        assert_eq!(s.comp_cache_entry(3), Some((0xaaaa_bbbb, -7)));
        assert_eq!(s.comp_cache_entry(16), None);
    }

    // ── HumanSeatPool tests ──────────────────────────────────────────────

    #[test]
    fn seat_geometry_matches_exe_constants() {
        assert_eq!(SEAT_STRIDE, 0xc0);
        assert_eq!(SEAT_MAX, 16);
        let p = HumanSeatPool::empty();
        assert_eq!(p.seats.len(), SEAT_MAX);
        assert!(p.seats.iter().all(|s| s.raw.len() == SEAT_STRIDE));
        assert_eq!(p.active_human(), 0);
    }

    #[test]
    fn take_control_fills_seat_and_flags() {
        let mut p = HumanSeatPool::empty();
        let idx = p.take_control(Some(3), 42, 100, 20010801, 5_000, 8_500).unwrap();
        assert_eq!(idx, 3);
        let s = &p.seats[3];
        assert!(s.is_occupied());
        assert!(s.is_employed());
        assert!(!s.is_sacked());
        assert!(!s.is_resigned());
        assert_eq!(s.person_id(), 42);
        assert_eq!(s.club_id(), 100);
        assert_eq!(s.wage(), 5_000);
        assert_eq!(s.reputation(), 8_500);
        assert_eq!(s.job_status(), job_status::MANAGER);
    }

    #[test]
    fn take_control_first_free_when_no_idx() {
        let mut p = HumanSeatPool::empty();
        assert_eq!(p.take_control(None, 1, 10, 0, 0, 0).unwrap(), 0);
        assert_eq!(p.take_control(None, 2, 11, 0, 0, 0).unwrap(), 1);
        assert_eq!(p.take_control(None, 3, 12, 0, 0, 0).unwrap(), 2);
    }

    #[test]
    fn take_control_rejects_club_already_managed() {
        let mut p = HumanSeatPool::empty();
        p.take_control(Some(0), 1, 100, 0, 0, 0).unwrap();
        let err = p.take_control(Some(1), 2, 100, 0, 0, 0).unwrap_err();
        assert_eq!(err, SeatError::ClubAlreadyHumanManaged);
    }

    #[test]
    fn take_control_out_of_range() {
        let mut p = HumanSeatPool::empty();
        assert_eq!(p.take_control(Some(16), 1, 10, 0, 0, 0), Err(SeatError::OutOfRange(16)));
    }

    #[test]
    fn switch_active_requires_occupied_seat() {
        let mut p = HumanSeatPool::empty();
        p.take_control(Some(2), 1, 10, 0, 0, 0).unwrap();
        p.take_control(Some(5), 2, 20, 0, 0, 0).unwrap();
        assert_eq!(p.active_human(), 0);
        p.switch_active(5).unwrap();
        assert_eq!(p.active_human(), 5);
        // Seat 7 is empty; switching there fails.
        assert_eq!(p.switch_active(7), Err(SeatError::NotOccupied));
        assert_eq!(p.active_human(), 5);
    }

    #[test]
    fn resign_clears_employment_but_keeps_seat() {
        let mut p = HumanSeatPool::empty();
        p.take_control(Some(0), 42, 100, 20010801, 5_000, 8_500).unwrap();
        p.resign(0).unwrap();
        let s = &p.seats[0];
        assert!(s.is_occupied(), "seat still held after resign");
        assert!(!s.is_employed());
        assert!(s.is_resigned());
        assert!(!s.is_sacked());
        assert_eq!(s.club_id(), 0);
        assert_eq!(s.nation_id(), 0);
        assert_eq!(s.wage(), 0);
        assert_eq!(s.job_status(), job_status::NONE);
        assert_eq!(s.person_id(), 42, "person link preserved");
        assert!(s.is_unemployed());
    }

    #[test]
    fn sack_terminates_seat() {
        let mut p = HumanSeatPool::empty();
        p.take_control(Some(1), 42, 100, 0, 5000, 7000).unwrap();
        p.switch_active(1).unwrap();
        assert_eq!(p.active_human(), 1);
        p.sack(1).unwrap();
        let s = &p.seats[1];
        assert!(!s.is_occupied(), "seat vacated on sack");
        assert!(!s.is_employed());
        assert!(s.is_sacked());
        assert_eq!(s.club_id(), 0);
        // Active human falls back to 0 when we sacked the active seat.
        assert_eq!(p.active_human(), 0);
    }

    #[test]
    fn sack_then_reseat_reuses_slot() {
        let mut p = HumanSeatPool::empty();
        p.take_control(Some(0), 1, 100, 0, 0, 0).unwrap();
        p.sack(0).unwrap();
        // Same club_id can now be taken over by another human (no longer human-managed).
        assert!(!p.is_club_human_managed(100));
        let idx = p.take_control(None, 2, 100, 0, 0, 0).unwrap();
        assert_eq!(idx, 0);
        assert!(p.seats[0].is_occupied());
        assert!(p.seats[0].is_employed());
        assert!(!p.seats[0].is_sacked(), "flags reset on take_control");
    }

    #[test]
    fn is_managed_by_active_reflects_active_human() {
        let mut p = HumanSeatPool::empty();
        p.take_control(Some(0), 1, 100, 0, 0, 0).unwrap();
        p.take_control(Some(1), 2, 200, 0, 0, 0).unwrap();
        assert!(p.is_managed_by_active(100));
        assert!(!p.is_managed_by_active(200));
        p.switch_active(1).unwrap();
        assert!(!p.is_managed_by_active(100));
        assert!(p.is_managed_by_active(200));
        assert!(p.is_club_human_managed(100));
        assert!(p.is_club_human_managed(200));
    }

    #[test]
    fn resign_and_sack_require_occupied() {
        let mut p = HumanSeatPool::empty();
        assert_eq!(p.resign(0), Err(SeatError::NotOccupied));
        assert_eq!(p.sack(0), Err(SeatError::NotOccupied));
        assert_eq!(p.resign(99), Err(SeatError::OutOfRange(99)));
        assert_eq!(p.sack(99), Err(SeatError::OutOfRange(99)));
    }

    #[test]
    fn all_field_offsets_land_inside_the_slot() {
        // Regression guard: any offset larger than the stride would panic on access.
        for off in [
            SLOT_CURRENT_TEAM_OR_ENTITY_ID, SLOT_FLAG_A, SLOT_FLAG_B, SLOT_PACKED_BLOCK,
            SLOT_RUNTIME_PTR_A, SLOT_RUNTIME_PTR_B, SLOT_UNKNOWN_COUNTER, SLOT_COUNTER_C,
            SLOT_SELECTOR, SLOT_ARRAY_A, SLOT_ARRAY_B, SLOT_CURRENT_MANAGED_CLUB_ID,
            SLOT_WEEK_COUNTER, SLOT_PERSON_BLOCK, SLOT_COMP_MODE, SLOT_COMP_CACHE,
            SLOT_SEARCH_NAME, SLOT_SEARCH_NAME_HISTORY, SLOT_TRAINING_NAME,
            SLOT_TRAINING_NAME_HISTORY, SLOT_PRINT_NAME, SLOT_MISC_BLOCK,
            SLOT_DYN_RECORDS_PTR, SLOT_DYN_RECORDS_COUNT_MIRROR, SLOT_DYN_RECORDS_COUNT,
        ] {
            assert!(off < HUMAN_SLOT_STRIDE, "slot offset {:#x} >= stride {:#x}",
                off, HUMAN_SLOT_STRIDE);
        }
        for off in [
            SUB_TACTICS_DATA, SUB_TACTIC_FIELD_A, SUB_TACTIC_FIELD_B, SUB_TACTIC_FLAG_A,
            SUB_TACTIC_FLAG_B, SUB_TACTIC_FLAG_C, SUB_TACTIC_NAME, SUB_TACTIC_NAME_HISTORY,
            SUB_REF_ID_LIST,
        ] {
            assert!(off < HUMAN_SUB_RECORD_STRIDE, "sub offset {:#x} >= stride {:#x}",
                off, HUMAN_SUB_RECORD_STRIDE);
        }
    }
}
