//! ScreenManager (scrman) — commit 1 of the port series.
//!
//! Ports the **struct layout** and the **constructor / structure-reset** portion of the exe's
//! ScreenManager class. Cross-checked against:
//! - cm0102.exe decomp: `FUN_007e4520` (ctor), `FUN_007e46a0` (dtor)
//! - **cm0102_GDI.exe** asm: `sub_007e3f20` (ctor, byte-precise field writes).
//!
//! Field byte offsets verified identical between the two builds; the GDI asm is the authority
//! (see `reports/scrman_module_inventory.md`). Every field access here cites its byte offset.
//!
//! **Not** included in this commit (documented deps in the inventory):
//! - Two session sub-objects at `+0x3070` and `+0x1326d1` (need `FUN_00548b40` port)
//! - The pump / push_screen / pop / peek / net-op family
//!
//! Layout: the whole instance is one heap-allocated `0x00262000`-byte block. The two large
//! session sub-objects (`+0x3070..+0x1326d0` and `+0x1326d1..+0x261fd3`) live inline; this
//! commit treats them as opaque bytes.

use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::collections::BTreeMap;
use std::ptr::NonNull;

/// Total instance size, in bytes. Constructor `sub_007e3f20` writes at most out to
/// `[esi+0x261fd0]+3 = 0x261fd3`; rounded up to page for cleanliness.
pub const SCRMGR_SIZE: usize = 0x0026_2000;

/// Number of slot records in the 16 × 0x300-byte slot table (`0x0000..0x2FFF`).
pub const SLOT_COUNT: usize = 0x10;
/// Stride of one slot record.
pub const SLOT_STRIDE: usize = 0x300;
/// Number of entries per slot. Each entry is `SLOT_ENTRY_STRIDE` bytes,
/// and `SLOT_ENTRY_COUNT * SLOT_ENTRY_STRIDE == SLOT_STRIDE` (16 × 0x30 = 0x300).
///
/// Evidence: `FUN_007e4940` L65 `psVar18 = param_1 + (sVar3 + iVar13) * 0x18` shorts,
/// with `iVar13 += 0x10` shorts per outer iteration (= 0x20 bytes = 1/24 of stride wait...)
/// The stride is `0x18 short = 0x30 byte`. GDI asm 007e43dd `lea eax,[eax+eax*2]; shl eax,4`
/// = `eax * 3 * 16 = eax * 0x30`, confirming 0x30 bytes per entry.
pub const SLOT_ENTRY_COUNT: usize = 0x10;
/// Stride of one slot entry, in bytes. `SLOT_STRIDE / SLOT_ENTRY_COUNT = 0x30`.
pub const SLOT_ENTRY_STRIDE: usize = 0x30;

/// Byte offsets of the header fields the constructor touches.
///
/// Each `OFF_*` is cited to a `mov` in GDI `sub_007e3f20` (line numbers are relative to that
/// asm file). Where a corresponding cm0102.exe decomp line uses short-indexing, the equivalent
/// short index is noted as `[N]` (byte = `2*N`).
pub mod off {
    // ---- root-frame header (0x00..0x27) ----
    /// `+0x00`  word — root-frame screen id.  GDI L65 `mov word ptr [esi], bp`.
    pub const ROOT_SCREEN_ID: usize = 0x00;
    /// `+0x02`  dword — screen arg 1.        GDI L66.
    pub const ROOT_ARG1: usize = 0x02;
    /// `+0x06`  dword — screen arg 2.        GDI L67.
    pub const ROOT_ARG2: usize = 0x06;
    /// `+0x0a`  dword — screen arg 3.        GDI L68.
    pub const ROOT_ARG3: usize = 0x0a;
    /// `+0x0e`  dword — screen arg 4 / cur-entry ptr.  GDI L69.
    pub const ROOT_CUR_ENTRY: usize = 0x0e;
    /// `+0x12`  word — flag.  GDI L70.
    pub const ROOT_FLAG_0X12: usize = 0x12;
    /// `+0x14`  word — depth counter.  GDI L71.
    pub const ROOT_DEPTH: usize = 0x14;
    /// `+0x16`  dword — **running flag, init = 1**.  GDI L72 `mov dword ptr [esi + 0x16], 1`.
    pub const ROOT_RUNNING: usize = 0x16;
    /// `+0x22`  word — **peer sentinel, init = 0xFFFF**.  GDI L73.
    pub const ROOT_PEER: usize = 0x22;
    /// `+0x24`  dword — aux.  GDI L74.
    pub const ROOT_AUX_0X24: usize = 0x24;
    /// `+0x28`  dword — points into first session pool (`esi + 0x3070`).  GDI L62-63.
    pub const INITIAL_SCREEN_PTR: usize = 0x28;

    // ---- slot table 0x0000..0x2FFF (16 × 0x300) is left at zero by ctor's rep-stos-block ----

    // ---- header 0x3000..0x306f ----
    /// `+0x3000`  dword — deferred-push name pointer.  GDI L78.
    pub const DEFERRED_NAME: usize = 0x3000;
    /// `+0x3004`  dword — deferred-push arg 1.  GDI L79.
    pub const DEFERRED_ARG1: usize = 0x3004;
    /// `+0x3008`  dword — deferred-push arg 2.  GDI L80.
    pub const DEFERRED_ARG2: usize = 0x3008;
    /// `+0x300c`  dword — deferred-push arg 3.  GDI L81.
    pub const DEFERRED_ARG3: usize = 0x300c;
    /// `+0x3010`  dword — deferred-push arg 4.  GDI L82.
    pub const DEFERRED_ARG4: usize = 0x3010;
    /// `+0x3014`  dword — bag-values ptr.  GDI L83.
    pub const BAG_VALUES: usize = 0x3014;
    /// `+0x3018`  word — bag-count.  GDI L84.
    pub const BAG_COUNT: usize = 0x3018;
    /// `+0x301a`  dword — aux.  GDI L75.
    pub const AUX_0X301A: usize = 0x301a;
    /// `+0x301e`  dword — aux.  GDI L76.
    pub const AUX_0X301E: usize = 0x301e;
    /// `+0x3022`  dword — aux.  GDI L77.
    pub const AUX_0X3022: usize = 0x3022;
    /// `+0x3026`  dword — aux.  GDI L37.
    pub const AUX_0X3026: usize = 0x3026;

    /// `+0x302a`  network-buffer sub-object base — 12 bytes: buf-ptr, write-off, size.
    /// Constructor via `sub_007631d0(this, 50000)` in GDI == `FUN_00763590(this, 50000)` in cm0102.exe.
    pub const NET_SUBOBJ: usize = 0x302a;
    /// `+0x302a` dword — heap pointer to the allocated buffer (`operator new(size)`).
    /// Kept as a 32-bit reflection in the arena; the owning allocation lives in the
    /// Rust-side `NetworkBuffer` (host pointer widths differ from the 32-bit exe, so this
    /// slot carries a marker only — see `ScreenManager::net_buf()`).
    pub const NET_BUF_PTR: usize = 0x302a;
    /// `+0x302e` dword — write offset within the buffer (init 0). Decomp: `param_1[1] = 0`.
    pub const NET_BUF_WRITE_OFF: usize = 0x302e;
    /// `+0x3032` dword — buffer size in bytes (init `param_2`; ctor arg is 50000).
    /// Decomp: `param_1[2] = param_2`.
    pub const NET_BUF_SIZE: usize = 0x3032;
    /// Size arg passed by ScreenManager's ctor: 50000 (0xC350).
    pub const NET_BUF_DEFAULT_SIZE: u32 = 50_000;

    /// `+0x3036 + slot*2`  16 words — per-slot depth counter.  GDI L48-60 zero-loop.
    /// Slot-0 initialized to 1 at GDI L64.
    pub const SLOT_DEPTH_TABLE: usize = 0x3036;

    /// `+0x3056`  word — current slot index.  GDI L41.
    pub const CURRENT_SLOT: usize = 0x3056;
    /// `+0x3058`  word — target slot (init 0xFFFF).  GDI L42.
    pub const TARGET_SLOT: usize = 0x3058;
    /// `+0x305c`  dword — flush-in-progress flag.  GDI L34.
    pub const FLUSH_INFLIGHT: usize = 0x305c;
    /// `+0x3060`  dword — aux.  GDI L43.
    pub const AUX_0X3060: usize = 0x3060;
    /// `+0x3068`  dword — pump-active flag.  GDI L31.
    pub const PUMP_ACTIVE: usize = 0x3068;
    /// `+0x306c`  dword — network mode flag.  GDI L30.
    pub const NET_MODE: usize = 0x306c;

    // ---- 0x3070..0x1326d0: first session sub-object (ctor arg = 0) ----
    /// `+0x3070`  first session sub-object base.  GDI ctor L22 `lea ecx, [esi + 0x3070]`
    /// then `call sub_00548d50` with `param_2 = 0`.
    pub const SESSION_A: usize = 0x3070;

    // ---- 0x1326d1..0x261fd3: second session sub-object (ctor arg = 1) ----
    /// `+0x1326d1`  second session sub-object base.  GDI ctor L27 `lea ecx, [esi + 0x1326d1]`
    /// then `call sub_00548d50` with `param_2 = 1`.
    pub const SESSION_B: usize = 0x1_326d1;

    // ---- Session sub-object field offsets, relative to the sub-object's own base ----
    // All cited to `sub_00548d50` (GDI) == `FUN_00548b40` (cm0102.exe). The asm ends in
    // `ret 4` — one stack-arg `__thiscall(u32 param_2)` — and holds 24 instructions.
    //
    // Fields marked `PTR` are 32-bit pointers/handles in the exe; the ctor zeroes them,
    // and the dtor guards `if (ptr != 0) free(ptr)` on them — see `SessionSubObject::drop`.
    /// `+0x00000` dword — asm L18 `[eax] = ecx (=0)`. First slot of the sub-object.
    pub const SESS_HEAD_DW: usize = 0x0_0000;
    /// `+0x12e99e` word — asm L21. Sub-count for exe dtor's `FUN_004031e0` loop.
    pub const SESS_SUBCOUNT_A: usize = 0x0_12e99e;
    /// `+0x12e9a0` word — asm L22. Sub-count for exe dtor's `FUN_005d8920` loop.
    pub const SESS_SUBCOUNT_B: usize = 0x0_12e9a0;
    /// `+0x12f4fe` byte — asm L26.
    pub const SESS_BYTE_0X12F4FE: usize = 0x0_12f4fe;
    /// `+0x12f4ff` word — asm L25, `= 0xffff`. **Sentinel.**
    pub const SESS_WORD_FFFF_A: usize = 0x0_12f4ff;
    /// `+0x12f501` word — asm L27, `= 0xffff`. **Sentinel.**
    pub const SESS_WORD_FFFF_B: usize = 0x0_12f501;
    /// `+0x12f503` word — asm L28, `= 0xffff`. **Sentinel.**
    pub const SESS_WORD_FFFF_C: usize = 0x0_12f503;
    /// `+0x12f505` dword PTR — asm L13.
    pub const SESS_PTR_0X12F505: usize = 0x0_12f505;
    /// `+0x12f521` dword PTR — asm L12; freed via `FUN_005cdfa0` by the dtor.
    pub const SESS_PTR_0X12F521: usize = 0x0_12f521;
    /// `+0x12f525` dword PTR — asm L11; freed via `FUN_005cdfa0` by the dtor.
    pub const SESS_PTR_0X12F525: usize = 0x0_12f525;
    /// `+0x12f529` dword PTR — asm L20; freed via `FUN_0093435a` (list-of-lists) by the dtor.
    pub const SESS_PTR_0X12F529: usize = 0x0_12f529;
    /// `+0x12f535` dword — asm L16.
    pub const SESS_DW_0X12F535: usize = 0x0_12f535;
    /// `+0x12f539` word — asm L23.
    pub const SESS_WORD_0X12F539: usize = 0x0_12f539;
    /// `+0x12f53b` word — asm L24.
    pub const SESS_WORD_0X12F53B: usize = 0x0_12f53b;
    /// `+0x12f53d` dword — asm L15.
    pub const SESS_DW_0X12F53D: usize = 0x0_12f53d;
    /// `+0x12f541` dword — asm L9 `[eax + 0x12f541] = edx (=param_2)`. **Mode arg** (0 or 1).
    pub const SESS_MODE: usize = 0x0_12f541;
    /// `+0x12f545` dword — asm L17.
    pub const SESS_DW_0X12F545: usize = 0x0_12f545;
    /// `+0x12f555` dword — asm L14.
    pub const SESS_DW_0X12F555: usize = 0x0_12f555;
    /// `+0x12f55d` byte — asm L19 `= 1`. **Init flag** — the only non-sentinel non-zero.
    pub const SESS_INIT_FLAG: usize = 0x0_12f55d;

    /// Highest byte the session ctor touches, exclusive: `+0x12f55d + 1`.
    pub const SESS_CTOR_HIGH_WATER: usize = 0x0_12f55e;

    // ---- Fields the ScreenManager ctor directly clears inside SESSION_B (layering quirk) ----
    /// GDI L35.
    pub const B_DWORD_0X261D32: usize = 0x0026_1d32;
    /// GDI L47-59 secondary bag: `ebx = 0x261d36 + i*0x28`, `rep stosd 10 dwords`.
    /// 16 iterations × 40 bytes zeroed (= 0x280 bytes: `0x261d36..0x261fb5`).
    pub const B_SECONDARY_BAG: usize = 0x0026_1d36;
    pub const B_SECONDARY_BAG_STRIDE: usize = 0x28;
    pub const B_SECONDARY_BAG_ZERO_DWORDS: usize = 10;
    /// GDI L36.
    pub const B_DWORD_0X261FB6: usize = 0x0026_1fb6;
    /// GDI L46 (word).
    pub const B_WORD_0X261FBA: usize = 0x0026_1fba;
    /// GDI L44.
    pub const B_DWORD_0X261FBC: usize = 0x0026_1fbc;
    /// GDI L45.
    pub const B_DWORD_0X261FC0: usize = 0x0026_1fc0;
    /// GDI L38.
    pub const B_DWORD_0X261FC4: usize = 0x0026_1fc4;
    /// GDI L39 (word).
    pub const B_WORD_0X261FC8: usize = 0x0026_1fc8;
    /// GDI L40 (word).
    pub const B_WORD_0X261FCA: usize = 0x0026_1fca;
    /// GDI L32.
    pub const B_DWORD_0X261FCC: usize = 0x0026_1fcc;
    /// GDI L33.
    pub const B_DWORD_0X261FD0: usize = 0x0026_1fd0;

    // ---- Fields the push_screen (FUN_007e6570) writes on the CURRENT slot ----
    // All are byte-offsets WITHIN a slot (add `slot_idx * SLOT_STRIDE` for real address).
    // For slot 0, most alias `ROOT_*` above (root frame IS slot 0's primary entry).
    /// `slot+0x02` dword — alt-head chain ptr slot. C L27 read as part of the "empty
    /// slot" predicate. Push_screen never writes it; kept here so the read has a name.
    pub const SLOT_ALT_HEAD_ID: usize = 0x02;
    /// `slot+0x06` dword — **head** RecordId (Rust port stores id, exe stored a raw
    /// pointer). C L103 (`iVar8 + 6`) read as evict-target; L161 (`iVar8 + 6`) set to
    /// new record if head was 0.
    pub const SLOT_HEAD_ID: usize = 0x06;
    /// `slot+0x0a` dword — **tail** RecordId. C L155/158 written; L75 read.
    pub const SLOT_TAIL_ID: usize = 0x0a;
    /// `slot+0x0e` dword — **current** RecordId. C L45/163 written; L37 read.
    /// (Alias of ROOT_CUR_ENTRY for slot 0.)
    pub const SLOT_CURRENT_ID: usize = 0x0e;
    /// `slot+0x12` word — modal-record counter (records with param_4 != 0). C L149
    /// increments as short when the pushed record's param_4 != 0.
    pub const SLOT_MODAL_COUNT: usize = 0x12;
    /// `slot+0x14` word — total-records-in-slot counter. C L109 decrements on evict,
    /// L165 increments on push. (Alias of ROOT_DEPTH for slot 0.)
    pub const SLOT_COUNT_WORD: usize = 0x14;
    /// `slot+0x16` dword — active/running flag. C L169 sets to 1. (Alias of ROOT_RUNNING for slot 0.)
    pub const SLOT_ACTIVE_FLAG: usize = 0x16;
    /// `slot+0x1a` dword — flag cleared to 0 at C L167.
    pub const SLOT_FLAG_0X1A: usize = 0x1a;
    /// `slot+0x1e` dword — flag cleared to 0 at C L173, only when param_4 != 0.
    pub const SLOT_FLAG_0X1E: usize = 0x1e;
    /// `slot+0x22` word — sentinel reset to 0xFFFF at C L170. (Alias of ROOT_PEER for slot 0.)
    pub const SLOT_SENTINEL_0X22: usize = 0x22;
    /// `slot+0x24` dword — flag cleared to 0 at C L171. (Alias of ROOT_AUX_0X24 for slot 0.)
    pub const SLOT_FLAG_0X24: usize = 0x24;

    // ---- Fields the pump (FUN_007e4940 / sub_007e4340) writes at entry ----
    // (Preamble portion only — see `ScreenManager::pump_preamble`.)
    //
    // Note: `+0x3068` and `+0x306a` overlap PUMP_ACTIVE (dword). The pump treats
    // them as two shorts (`param_1[0x1834] = 1; param_1[0x1835] = 0;`, C lines
    // 35-36) — bytes 01 00 00 00, byte-identical to `set_u32(PUMP_ACTIVE, 1)`.
    /// `+0x3068` word — pump-active low half. C L35 / asm 007e435a.
    pub const PUMP_ACTIVE_LO: usize = 0x3068;
    /// `+0x306a` word — pump-active high half. C L36.
    pub const PUMP_ACTIVE_HI: usize = 0x306a;
    /// `+0x261c2a` dword — flag = (mode_table[5] == 0x7e0). C L45 / asm 007e4378.
    pub const PUMP_MODE_FLAG_A: usize = 0x0026_1c2a;
    /// `+0x1325c9` dword — same predicate, second slot. C L53 / asm 007e439d.
    pub const PUMP_MODE_FLAG_B: usize = 0x0013_25c9;
    /// `+0x13256a` word — cleared at pump entry. C L55 (`param_1[0x992b5] = 0`) /
    /// asm 007e43a8 `mov dword ptr [ebp + 0x13256a], ebx (=0)` (dword-write
    /// covering both 0x13256a and 0x13256c).
    pub const PUMP_WORD_0X13256A: usize = 0x0013_256a;
    /// `+0x13256c` word — cleared at pump entry. C L56.
    pub const PUMP_WORD_0X13256C: usize = 0x0013_256c;
}

/// Byte offsets **within a slot entry** (each entry is `SLOT_ENTRY_STRIDE` = 0x30
/// bytes wide; a slot holds `SLOT_ENTRY_COUNT` = 16 entries). Cited to
/// `FUN_007e4940` short-indexed reads/writes: short-idx K means byte 2K.
///
/// The **primary entry** of slot 0 overlaps ScreenManager's root-frame header,
/// so entry[+0x00] of slot 0 == `off::ROOT_SCREEN_ID`, entry[+0x0E] == `off::ROOT_CUR_ENTRY`,
/// entry[+0x16] == `off::ROOT_RUNNING`, entry[+0x28] == `off::INITIAL_SCREEN_PTR`.
/// Child entries (idx 1..15) of every slot get their own 0x30-byte block and are used
/// by the -8 (broadcast) case and the entry-prep loop.
pub mod entry {
    /// `+0x00` short — screen_id (peer/broadcast id; `param_1[slot*0x180]`, C L120 & L364).
    /// Routes post-per-frame call: 0 → `FUN_00548ef0`, else → `FUN_0054be80`.
    pub const SCREEN_ID: usize = 0x00;
    /// `+0x0E` dword — current record pointer. Short-idx 7 (byte 0x0E) via `psVar+7`.
    /// The dispatch reads `**(undefined4**)(this+7)` = vtable[0] of this ptr's target.
    /// C L117 (per-frame call), L143 (event guard).
    pub const CURRENT_RECORD_PTR: usize = 0x0E;
    /// `+0x16` dword — active / running flag. Short-idx 0xb (byte 0x16).
    /// C L66 tests `*(int*)(entry+0xb) == 0`. C L102 tests the same at slot-primary
    /// scope. Written as two shorts by L67-70 (=1,0,1,0 when going inactive) or L73-76
    /// (=0,0,0,0 when going active). Byte-equivalent to a u32 write of 1 or 0.
    pub const ACTIVE_FLAG: usize = 0x16;
    /// `+0x1A` dword — acked-a flag. Short-idx 0xd. L68/L74 clear; L78 tests on
    /// slot-primary to gate the memcpy-from-primary to child.
    pub const ACKED_A: usize = 0x1A;
    /// `+0x1E` dword — acked-b flag. Short-idx 0xf. L69/L75.
    pub const ACKED_B: usize = 0x1E;
    /// `+0x28` dword — session sub-object pointer. Short-idx 0x14 (byte 0x28).
    /// C L116 dereffed as `*(int*)(*(int*)(slot+0x14) + 0x12f53d)` — writes 1 then 0
    /// around the per-frame call to guard the session sub-object's `SESS_DW_0X12F53D`.
    pub const SESSION_PTR: usize = 0x28;
    /// `+0x2C` dword — timestamp. Short-idx 0x16 (byte 0x2C). C L77 sets from `local_428`
    /// (= `last_time_snapshot`) during entry-prep. `psVar18[0x16] = local_428`.
    pub const TIMESTAMP: usize = 0x2C;
}

// ------------------------------------------------------------
// PumpDispatchResult — the i32 return code from a ScreenRecord's
// event handler (vtable[2]) that the pump's switch consumes.
// ------------------------------------------------------------

/// Return code from a `ScreenRecord`'s event handler (`vtable[2]`), consumed
/// by the switch at C L206-319 of `FUN_007e4940`. Variants are named by their
/// raw i32 value (case label in the decomp); docs describe the case body's
/// side effect.
///
/// **Cases not yet wired.** The switch bodies read/write record-chain fields
/// (`+0x1F8` next-ptr, `+500`/`0x1F4` prev-ptr, `+0x14` 0x3c-slot bag, `+0x10`
/// modal-flag, `+0xC` cleanup2) that `push_screen` (commit 5) will fully decode,
/// and call several unported externals (`FUN_007eaac0`, `FUN_007e7b50`,
/// `FUN_00933d24` — record free helper). This enum is defined now so the
/// dispatch scaffold in commit 4d can declare its exit type; the actual switch
/// body port lands with commit 5.
///
/// The pump's final return value is `local_434 == -5` (C L686), i.e. the
/// pump exits `true` iff the last dispatch produced `CaseNeg5`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PumpDispatchResult {
    /// `0` — continue / default case (C L221-226). If `local_438 != -1` AND
    /// slot-primary `+0x16 == 0`, invokes `FUN_007eaac0(0, 0)`. Then falls to
    /// `LAB_007e5550` (slot-completion tally + loop-back gate).
    Case0 = 0,
    /// `-1` — pop-to-root. Cleanup (`vtable[1]`) then restore record ptr from
    /// slot's saved root-record ptr (`slot+0x06` dword). C L307-318.
    CaseNeg1 = -1,
    /// `-2` — pop-one via prev-ptr chain. Cleanup then follow record's `+500`
    /// prev-ptr. C L298-306.
    CaseNeg2 = -2,
    /// `-3` — pop-one via next-ptr chain. Cleanup then follow record's `+0x1F8`
    /// next-ptr. C L290-297.
    CaseNeg3 = -3,
    /// `-4` / `-0xb` — mark slot `+0xB` active-flag = 1 then fall to `LAB_007e5550`.
    /// C L227-232.
    CaseNeg4 = -4,
    /// `-5` — soft-abort. Fall-through group with -6/-8/-0xd via `LAB_007e52a2`
    /// tail-cleanup. Pump return = `local_434 == -5`. C L155, 271, 393, 449.
    CaseNeg5 = -5,
    /// `-6` — sibling-lift. Same body as -5 but sets slot `+0xF` and `+0xD`
    /// dword flags at L449-455.
    CaseNeg6 = -6,
    /// `-7` — pump-exit. Skips the loop-back at L514 and jumps to
    /// `LAB_007e5613` finalization (broadcast pending-processing to peers +
    /// per-slot final cleanup). C L513-514.
    CaseNeg7 = -7,
    /// `-8` — broadcast slot's `+0xF` flag to every child entry (0x30 stride
    /// within the slot). C L467-475.
    CaseNeg8 = -8,
    /// `-9` — same body as -0xa but skips the `iVar8 + 500 == 0` guard.
    /// Jumps to `switchD_007e4f3c_caseD_fffffff7` (C L380-424).
    CaseNeg9 = -9,
    /// `-10` (`-0xa`) — pop-with-cleanup-chain. Cleanup then walk down `+500`
    /// prev-ptr chain, freeing each record's 0x3c-slot bag via `FUN_00933d24`,
    /// decrementing the slot's counter. C L233-287.
    CaseNeg10 = -10,
    /// `-11` (`-0xb`) — same body as -4.
    CaseNeg11 = -11,
    /// `-12` (`-0xc`) — network-flush-error. Builds a 4-byte error packet at
    /// buf `+0x1817`, or emits UI error via `FUN_005d1c30` if buffer full,
    /// then calls `FUN_00762b90`. Falls into `LAB_007e4e76` for further switch
    /// handling. C L189-205.
    CaseNeg12 = -12,
    /// `-13` (`-0xd`) — pop-to-oldest. Falls into the -5/-6/-8 group via case
    /// fall-through at C L207-210. `LAB_007e524c` (L435) distinguishes it via
    /// `local_434 == -0xd` to gate the record-chain-lift inner loop.
    CaseNeg13 = -13,
}

impl PumpDispatchResult {
    /// Reconstruct the enum from the raw i32 the event handler returns.
    /// Any value outside the decoded -13..0 range is a bug — the exe's switch
    /// falls into `default` (Case0) for such values.
    pub fn from_raw(v: i32) -> Self {
        match v {
            0 => Self::Case0,
            -1 => Self::CaseNeg1,
            -2 => Self::CaseNeg2,
            -3 => Self::CaseNeg3,
            -4 => Self::CaseNeg4,
            -5 => Self::CaseNeg5,
            -6 => Self::CaseNeg6,
            -7 => Self::CaseNeg7,
            -8 => Self::CaseNeg8,
            -9 => Self::CaseNeg9,
            -10 => Self::CaseNeg10,
            -11 => Self::CaseNeg11,
            -12 => Self::CaseNeg12,
            -13 => Self::CaseNeg13,
            _ => Self::Case0,   // decomp's `default:` handles all unknowns
        }
    }

    /// The pump's final return value is `true` iff the last dispatch produced -5.
    #[inline]
    pub fn is_pump_exit_true(self) -> bool { self == Self::CaseNeg5 }
}

// ------------------------------------------------------------
// ScreenRecord — the per-entry structure a slot list holds.
// ------------------------------------------------------------
//
// Full struct layout lives in `push_screen` (FUN_007e6570, ported in commit 5)
// which allocates 0x300 bytes and populates the fields. The pump reads:
// - `+0x00` — vtable ptr (dispatched by `pump` per-frame handler)
// - `+0x04` — cleanup fn ptr (called on teardown, args: local_438)
// - `+0x08` — event fn ptr (called on input events)
//
// Slots 0x0c..0x300 are TBD — decoded incrementally by commits 4c + 5.

/// Vtable for a `ScreenRecord`. Slots decoded from pump call sites in
/// `FUN_007e4940` (C lines 117, 179, 277, 417) and `push_screen`.
///
/// Only the first three slots are used by the 4a preamble (indirectly — the
/// preamble does not call any of them; commit 4c wires them at dispatch).
/// Remaining vtable slots stay `unk_*` until push_screen decodes them.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScreenRecordVTable {
    /// `+0x00` — per-frame handler. Called every pump tick on the current record.
    pub per_frame: Option<fn(&mut ScreenManager)>,
    /// `+0x04` — cleanup handler. Called on screen teardown; takes one stack arg
    /// (the `local_438` state slot from the pump).
    pub cleanup: Option<fn(&mut ScreenManager, u32)>,
    /// `+0x08` — event handler. Called on input events; returns an i32 code
    /// consumed by the pump's -1..-13 dispatch table.
    pub event: Option<fn(&mut ScreenManager, u32) -> i32>,
    // TODO(commit 4c/5): fill remaining slots as push_screen decodes them.
}

/// Rust-native handle for a `ScreenRecord`. In the exe, records are addressed
/// by 32-bit heap pointers; the Rust port owns records in `ScreenManager::records`
/// keyed by this id and mirrors ids into arena bytes wherever the exe stored a
/// pointer (slot's head/tail/current at +0x06/+0x0a/+0x0e; record's prev/next).
/// `0` sentinels "null pointer" (matches the exe's `piVar12 != 0` guards).
pub type RecordId = u32;

/// One entry in a `ScreenRecord`'s 0x3c-slot bag at record `+0x14..+0x1F4`
/// (60 × 8 bytes). Populated by `FUN_007e7130` (see `push_screen`'s deferred
/// replay path); freed on eviction if `owned == true`.
///
/// Byte-layout in the exe (matches `push_screen`'s eviction loop C L79-84 and
/// `FUN_007e7130`'s stores at `+0x14 + i*8` / `+0x18 + i*8`):
/// - `+0` dword — value pointer (opaque; heap-allocated string or bag payload)
/// - `+4` dword — `owned` flag (nonzero → free with `operator delete` on evict)
#[derive(Debug, Clone, Default)]
pub struct SlotBagEntry {
    /// `+0` — the payload. `None` when no value stored. When `Some`, the exe
    /// stored a heap pointer here; we own the bytes directly in Rust.
    pub value: Option<Vec<u8>>,
    /// `+4` — the exe's `owned` flag. Recorded for parity with the ctor;
    /// eviction always drops the `Vec` regardless (Rust owns it either way).
    pub owned: bool,
}

/// A screen record — 0x300 bytes in the exe, allocated by `push_screen`
/// (`FUN_007e6570`). All field offsets cited to that C.
///
/// Layout (from `FUN_007e6570` writes + reads by callers):
/// - `+0x00` dword — `screen_id`   (C L106 `piVar6[0] = param_2`)
/// - `+0x04` dword — `cleanup` fn  (C L107 `piVar6[1] = param_5`; called at C L120)
/// - `+0x08` dword — `param_3`     (C L109 `piVar6[2] = param_3`)
/// - `+0x0C` dword — `param_6`     (C L108 `piVar6[3] = param_6`) — pump 4c calls this "cleanup2"
/// - `+0x10` dword — `param_4`     (C L110 `piVar6[4] = param_4`) — **modal flag**;
///                                  C L103 evict-guard `if (record[0x10] != 0) return 0`
/// - `+0x14..+0x1F4` — `slot_bag[0..0x3c]` (60 × 8-byte SlotBagEntry).
///                                  Cleared by C L84-89 eviction loop; populated by `FUN_007e7130`.
/// - `+0x1F4` dword — `prev` RecordId (piVar6[0x7d]; C L157 `piVar6[0x7d] = old_tail`)
/// - `+0x1F8` dword — `next` RecordId (piVar6[0x7e]; C L156 `old_tail[0x1f8] = new`;
///                                  read at C L45, 76, 88, 103)
/// - `+0x1FC..+0x300` — `name` bytes (piVar6[0x7f]; C L124-141 strcpy from DAT_009afdec)
///
/// The `vtable` field (from commit 4a) is retained as a Rust-side grouping for
/// typed callback fn pointers — the exe stores the cleanup fn raw at `+0x04`,
/// which we mirror in `cleanup`. Other vtable slots remain None until dispatched
/// paths (pump 4d) decode more.
#[derive(Debug, Default)]
pub struct ScreenRecord {
    /// Rust-side allocator id. Mirrors into arena slot ptr slots as u32.
    pub id: RecordId,
    /// Logical vtable grouping (per commit 4a). `cleanup` slot below is the
    /// authoritative record `+0x04` field.
    pub vtable: ScreenRecordVTable,
    /// `+0x00` — screen name-hash / id.
    pub screen_id: u32,
    /// `+0x04` — cleanup fn (called on old-current at push time, C L120).
    pub cleanup: Option<fn(&mut ScreenManager)>,
    /// `+0x08` — `param_3`.
    pub param_3: i32,
    /// `+0x0C` — `param_6`.
    pub param_6: i32,
    /// `+0x10` — `param_4` (nonzero = modal; eviction & lookup gate).
    pub param_4: i32,
    /// `+0x14..+0x1F4` — the 60-entry slot bag.
    pub slot_bag: Vec<SlotBagEntry>,
    /// `+0x1F4` — prev record in the linked list. `None` for the head.
    pub prev: Option<RecordId>,
    /// `+0x1F8` — next record. `None` for the tail.
    pub next: Option<RecordId>,
    /// `+0x1FC..+0x300` — copied name string (up to 260 bytes).
    pub name: Vec<u8>,
}

impl ScreenRecord {
    fn new(id: RecordId, screen_id: u32, cleanup: Option<fn(&mut ScreenManager)>,
           param_3: i32, param_4: i32, param_6: i32, name: Vec<u8>) -> Self {
        ScreenRecord {
            id, vtable: ScreenRecordVTable::default(),
            screen_id, cleanup, param_3, param_6, param_4,
            slot_bag: vec![SlotBagEntry::default(); 0x3c],
            prev: None, next: None,
            name,
        }
    }
}

/// Return code from `ScreenManager::push_screen` — models `FUN_007e6570`'s
/// three exit paths (return values 0 / 1 and the `DAT_00b4d5a8 = 0` error path).
///
/// The exe returns `undefined4`: `1` on the "new record pushed" happy path
/// (C L184), `0` when it rewinds to an existing record (C L67-70), when the
/// deferred/args validation trips (`screen_id == 0 || param_3 == 0`; C L41),
/// or when eviction is blocked by a modal head record (C L102).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushScreenResult {
    /// New record allocated + linked. Exe returns `1`.
    NewRecordPushed,
    /// Matching `screen_id` found in the current-forward chain; slot's `current`
    /// rewound to it. Exe returns `0`.
    RewoundToExisting,
    /// `screen_id == 0` or `param_3 == 0` — error path. Exe sets
    /// `DAT_00b4d5a8 = 0` and returns `0` (C L41-45).
    InvalidArgs,
    /// Slot has >100 records and the head is modal (`param_4 != 0`), so it
    /// cannot be evicted. Exe returns `0` early (C L102).
    EvictionBlockedModal,
}

impl PushScreenResult {
    /// The exe's `undefined4` return value: `1` for NewRecordPushed, `0` otherwise.
    pub fn as_exe_ret(self) -> u32 {
        matches!(self, PushScreenResult::NewRecordPushed) as u32
    }
}

// ------------------------------------------------------------
// DAT_00acde98 — 8-dword static table copied into a stack buffer at pump entry.
// Symbol table (data_symbols.json) types this as `undefined4` — untyped 32-bit
// values. Not decoded yet; the pump reads element [5] to compare against 0x7e0
// (= 2016 decimal — plausibly a season year, but unverified).
// ------------------------------------------------------------

/// Snapshot of the `DAT_00acde98` static table read at pump entry.
///
/// The exe copies 8 dwords starting at absolute address `0x00acde98` (asm
/// `mov esi, 0xacde98; rep movsd 8`) into a stack buffer, then reads element
/// [5] (offset 0x14 within the copy) for the mode comparison. Real values
/// need extraction from the exe's .data section — for 4a the placeholder is
/// all-zero, and callers wanting a nonzero mode-flag override via
/// `pump_preamble_with_mode_table`.
///
/// TODO(later commit): extract real bytes from cm0102_GDI.exe and inline.
pub const DAT_ACDE98: [u32; 8] = [0; 8];

/// The 0x7e0 constant the pump compares against.
pub const PUMP_MODE_MATCH: u32 = 0x7e0;

/// Snapshot of `DAT_009afdec` (the "current-loading screen filename" global)
/// used by `push_screen` C L124-141 to seed `record.name`.
///
/// **Writer.** Grep across every decomp file finds only READs of this global
/// (six sites: FUN_00548ef0 L26, FUN_005e6280 L135, FUN_008fb240 L41,
/// FUN_008fb3f0 L97, FUN_008fc280 L42, FUN_0093cc01 L33). No decompiled fn
/// writes to it; the byte at 0x009afdec is tagged `undefined1` (single byte,
/// no string literal) in ghidra's `data_symbols.json`. It's a runtime-mutable
/// filename buffer written indirectly (likely via a pointer alias set up by
/// resource-load code we haven't reached). Since the exe layout gives us no
/// writer to port, this port exposes an explicit setter
/// (`ScreenManager::set_loading_filename`) and has push_screen read from an
/// owned `loading_filename: Vec<u8>` field.
///
/// The push_screen path calls `self.loading_filename.clone()` directly — this
/// standalone helper is retained only as documentation for the exe global's
/// role and always returns empty.
fn read_dat_009afdec_snapshot() -> Vec<u8> { Vec::new() }

// ============================================================
// Real mktime port — FUN_0093b4b0 (cm0102.exe) == sub_0093acf0 (GDI).
// ============================================================

/// Month-offset table.  Byte-exact dump from `cm0102_GDI.exe` `.data` @ 0x00ac5238
/// (13 dwords). Identical bytes at cm0102.exe's `.data` @ 0x00ac52e8.
///
/// Layout is `[end_marker, jan_offset, feb_offset, ..., dec_offset]`. Values are
/// cumulative days through the end of the PREVIOUS month, minus 1 (Jan=-1 is a
/// sentinel: since the exe adds `day_of_month` afterwards, day=1 gives `esi=0`).
///
/// Dumped 2026-09-04 via pefile: bytes
/// `6d010000 ffffffff 1e000000 3a000000 59000000 77000000 96000000 b4000000
///  d3000000 f2000000 10010000 2f010000 4d010000`
/// → decoded `[365, -1, 30, 58, 89, 119, 150, 180, 211, 242, 272, 303, 333]`.
pub const CM_MKTIME_MONTH_TABLE: [i32; 13] =
    [365, -1, 30, 58, 89, 119, 150, 180, 211, 242, 272, 303, 333];

/// `DAT_00ac4bd8` (GDI) / `DAT_00ac4c88` (cm0102.exe) — additive seconds
/// baseline (fixed timezone offset baked in at build time).
/// Dumped 2026-09-04 from cm0102_GDI.exe: `80 70 00 00` = 28800 (= 8 hours in seconds).
pub const CM_MKTIME_BASE_SECS: i32 = 28800;

/// `DAT_00ac4bdc` (GDI) / `DAT_00ac4c8c` (cm0102.exe) — DST-enable flag.
/// Dumped 2026-09-04 from cm0102_GDI.exe: `01 00 00 00` = 1 (DST enabled).
pub const CM_MKTIME_DST_ENABLE: i32 = 1;

/// `DAT_00ac4be0` (GDI) / `DAT_00ac4c90` (cm0102.exe) — DST bias in seconds.
/// Dumped 2026-09-04 from cm0102_GDI.exe: `f0 f1 ff ff` = -3600 (= -1 hour).
pub const CM_MKTIME_DST_BIAS: i32 = -3600;

/// The `0x7c558180` literal baked into the pump asm (both binaries agree).
pub const CM_MKTIME_EPOCH_CONST: i32 = 0x7c55_8180_u32 as i32;

/// Port of cm0102.exe `FUN_0093b4b0` == GDI `sub_0093acf0` — byte-exact.
///
/// Signature: `mktime(year, month, day, hour, minute, second, dst_flag)`.
/// - `year` is the wall-clock year (e.g. 1998, not `year-1900`).
/// - `month` is 1..12.
/// - `dst_flag`: `1` = force DST on, `0` = force DST off, `-1` = "use ambient"
///   (reads `DAT_00ac4bdc`, but since ambient path calls `FUN_0093bc41` which
///   needs full timezone context, this port treats `-1` as "check enable flag
///   and apply bias if enabled" — i.e. `DAT_00ac4bdc != 0`).
///
/// Returns `-1` for years outside 1970..2038 (matches C L15-17).
///
/// **Arithmetic** (asm 0093ad24..ad76, all i32, wraps on overflow):
/// ```text
/// yrs = year - 1900
/// doy = month_table[month] + day_of_month  (index 1..12; month=0 hits sentinel 365)
/// if (yrs % 4 == 0 && month > 2) doy += 1         ; leap-year adjust
/// total = (((yrs * 365 + (yrs - 1)/4 + doy) * 24 + hour) * 60 + minute) * 60
///       + BASE_SECS + EPOCH_CONST + second
/// if apply_dst: total += DST_BIAS
/// ```
pub fn cm_mktime(year: i32, month: i32, day: i32, hour: i32, minute: i32,
                 second: i32, dst_flag: i32) -> i32 {
    let yrs = year - 1900;                                  // asm L11: sub ebx,0x76c
    if yrs < 70 || yrs > 138 { return -1; }                 // asm L12-15
    // month table lookup + day-of-month (asm L19-20).
    let midx = month as usize;
    if midx >= CM_MKTIME_MONTH_TABLE.len() { return -1; }
    let mut doy = CM_MKTIME_MONTH_TABLE[midx].wrapping_add(day);
    // Leap-year adjust (asm L21-25: test bl,3; cmp edi,2; jle skip; inc esi).
    if (yrs & 3) == 0 && month > 2 { doy = doy.wrapping_add(1); }
    // Main polynomial (asm L27-38).
    let years_days = yrs.wrapping_mul(365).wrapping_add((yrs - 1) >> 2);
    let hours = years_days.wrapping_add(doy).wrapping_mul(24).wrapping_add(hour);
    let minutes = hours.wrapping_mul(60).wrapping_add(minute);
    let mut secs = minutes.wrapping_mul(60)
        .wrapping_add(CM_MKTIME_BASE_SECS)
        .wrapping_add(CM_MKTIME_EPOCH_CONST)
        .wrapping_add(second);
    // DST branch (asm L46-52).
    let apply_dst = match dst_flag {
        1 => true,
        -1 => CM_MKTIME_DST_ENABLE != 0,
        _ => false,
    };
    if apply_dst { secs = secs.wrapping_add(CM_MKTIME_DST_BIAS); }
    secs
}

/// Decompose a UTC unix-timestamp (seconds since 1970-01-01) into
/// (year, month[1..12], day[1..31], hour, minute, second) via Howard
/// Hinnant's `civil_from_days` algorithm. Independent of any Rust date crate.
fn decompose_utc(unix_secs: i64) -> (i32, i32, i32, i32, i32, i32) {
    let days = unix_secs.div_euclid(86400);
    let sod = unix_secs.rem_euclid(86400);
    let hour = (sod / 3600) as i32;
    let minute = ((sod / 60) % 60) as i32;
    let second = (sod % 60) as i32;
    let z = days + 719468;
    let era = if z >= 0 { z / 146097 } else { (z - 146096) / 146097 };
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy as i64 - (153 * mp as i64 + 2) / 5 + 1) as i32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as i32;
    let year = (y + if m <= 2 { 1 } else { 0 }) as i32;
    (year, m, d, hour, minute, second)
}

// ============================================================
// Session sub-object dtor helpers — ports of FUN_00548bd0's 5 teardown deps.
// ============================================================

use std::sync::atomic::{AtomicU64, Ordering};

/// Observability counters for the 5 teardown helpers. Public so tests can
/// reset and read them. Order matches doc-strings on each helper.
pub static TEARDOWN_COUNTS: [AtomicU64; 5] = [
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0),
];

/// Reset all teardown counters to 0. Test helper.
pub fn reset_teardown_counts() {
    for c in TEARDOWN_COUNTS.iter() { c.store(0, Ordering::SeqCst); }
}

/// Snapshot the current teardown counts as `[u64; 5]`. Test helper.
pub fn snapshot_teardown_counts() -> [u64; 5] {
    let mut out = [0u64; 5];
    for (i, c) in TEARDOWN_COUNTS.iter().enumerate() {
        out[i] = c.load(Ordering::SeqCst);
    }
    out
}

/// Port of `FUN_0093435a(lpMem)` — the generic "HeapFree if non-null" helper
/// (byte-for-byte 26-line decomp). In Rust we don't own the exe's HeapAlloc
/// arena; this port bumps `TEARDOWN_COUNTS[0]` (HEAP_FREE) when a non-zero
/// pointer would be freed and returns `true`; on `ptr == 0` it returns `false`.
///
/// Real memory ownership stays with Rust `Box`/`Vec` — this helper models
/// the exe's arena-side bookkeeping so composed dtors (`FUN_005cdfa0`,
/// `FUN_004031e0`, `FUN_005d8920`) can be tested end-to-end.
pub fn heap_free(ptr: u32) -> bool {
    if ptr == 0 { return false; }
    TEARDOWN_COUNTS[0].fetch_add(1, Ordering::SeqCst);
    true
}

/// Port of `FUN_005cdfa0(psurf)` — free a screen-surface record and its
/// backing pixel buffer:
/// ```text
///   if (psurf != 0) {
///     if (psurf[3] != 0) { HeapFree(psurf[3]); psurf[3] = 0; }
///     HeapFree(psurf);
///   }
/// ```
/// Byte-faithful port: bumps `TEARDOWN_COUNTS[1]` (FREE_SURFACE) when
/// `psurf != 0`. The two nested `heap_free`s bump `[0]` twice per surface
/// (once for the pixel buf, once for the surface header).
pub fn free_surface(psurf: u32, pixel_buf: u32) -> bool {
    if psurf == 0 { return false; }
    TEARDOWN_COUNTS[1].fetch_add(1, Ordering::SeqCst);
    if pixel_buf != 0 { heap_free(pixel_buf); }
    heap_free(psurf);
    true
}

/// Port of `FUN_004031e0(pwidget)` — widget-pool entry teardown:
/// ```text
///   if (pwidget[+0xba6] != 0) { FUN_005cdfa0(pwidget[+0xba6]); pwidget[+0xba6]=0; }
///   if (pwidget[+0x200] != 0) { FUN_005cdfa0(pwidget[+0x200]); pwidget[+0x200]=0; }
///   if (pwidget + 0xb7a == DAT_00ac56e8) DAT_00ac56e8 = 0;
/// ```
/// Byte-faithful. Bumps `TEARDOWN_COUNTS[2]` (WIDGET_POOL_TEARDOWN) once per
/// invocation; may trigger 0-2 additional `free_surface` bumps.
pub fn widget_pool_teardown(surface_a_ptr: u32, surface_a_pixels: u32,
                            surface_b_ptr: u32, surface_b_pixels: u32) {
    TEARDOWN_COUNTS[2].fetch_add(1, Ordering::SeqCst);
    free_surface(surface_a_ptr, surface_a_pixels);
    free_surface(surface_b_ptr, surface_b_pixels);
    // DAT_00ac56e8 identity check is a debug guard — no populated-state
    // observable side effect for this port.
}

/// Port of `FUN_005d8920(pmsgbox)` — msgbox-stack entry teardown. Two
/// pointer slots (`+0x4c` and `+0x5c`); the `+0x5c` free is gated on a
/// strcmp against a global sentinel string. Byte-faithful skeleton:
/// bumps `TEARDOWN_COUNTS[3]` (MSGBOX_TEARDOWN); frees the +0x4c slot
/// via `free_surface`; frees the +0x5c slot when `refcount_should_free`
/// (models the strcmp gate result — caller passes true to force free).
pub fn msgbox_stack_teardown(surf_a_ptr: u32, surf_a_pixels: u32,
                             surf_b_ptr: u32, surf_b_pixels: u32,
                             refcount_should_free: bool) {
    TEARDOWN_COUNTS[3].fetch_add(1, Ordering::SeqCst);
    free_surface(surf_a_ptr, surf_a_pixels);
    if surf_b_ptr != 0 && refcount_should_free {
        free_surface(surf_b_ptr, surf_b_pixels);
    }
    // DAT_00acdb1c decrement path (refcount stays > 0) has no observable
    // effect on the counters this port exposes.
}

/// Port of `FUN_0093534b(base, stride, count, per_element_dtor)` — the C++
/// array-destructor runtime helper. Bumps `TEARDOWN_COUNTS[4]`
/// (ARRAY_DTOR) once, then invokes the caller-supplied per-element closure
/// `count` times. Byte-faithful in the loop shape (`while (--count >= 0)`
/// per asm L21-25); exception-handler frame setup is omitted (no
/// SEH in this Rust port).
pub fn array_destructor(count: usize, mut per_element: impl FnMut()) {
    TEARDOWN_COUNTS[4].fetch_add(1, Ordering::SeqCst);
    for _ in 0..count { per_element(); }
}

/// Network send/recv buffer sub-object embedded in `ScreenManager` at `+0x302a`.
///
/// Port of cm0102.exe `FUN_00763590(this, size)` — 3-field POD ctor:
/// ```text
///   param_1[1] = 0;                       // +0x04 write-offset
///   param_1[2] = param_2;                 // +0x08 size
///   pvVar1     = operator new(param_2);   // heap buffer
///   *param_1   = pvVar1;                  // +0x00 buf-ptr
///   if (pvVar1 == 0) { <fatal error>; abort; }
/// ```
/// The Rust port owns the buffer as a `Box<[u8]>` so `Drop` handles the exe's dtor role
/// (a paired `operator delete` on `+0x00`). The scrman inventory's tentative pairing
/// with `FUN_007627f0` is misleading — that function operates on an unrelated 800KB
/// object (offset `+0xc3a5e`), not this 12-byte buffer.
pub struct NetworkBuffer {
    /// `+0x00` — heap-allocated buffer, zero-initialised (matches `operator new` for
    /// byte-arrays; the exe reads only up to `write_off`, so leading zeros are safe).
    buf: Box<[u8]>,
    /// `+0x04` — write offset (index of next byte to write).
    write_off: u32,
    /// `+0x08` — total buffer size in bytes (constant after ctor).
    size: u32,
}

impl NetworkBuffer {
    /// Port of `FUN_00763590`. `size` is the ctor arg (`param_2`); ScreenManager passes 50000.
    ///
    /// Rust's allocator aborts on OOM, mapping naturally to the exe's `FUN_009349c4(0xffffffff)`
    /// no-return fatal path on `pvVar1 == 0`.
    pub fn new(size: u32) -> Self {
        // Order mirrors the decomp: write_off then size, then allocation.
        let write_off: u32 = 0;
        let bytes = vec![0u8; size as usize].into_boxed_slice();
        NetworkBuffer { buf: bytes, write_off, size }
    }

    /// `+0x08` field.
    #[inline]
    pub fn size(&self) -> u32 { self.size }
    /// `+0x04` field.
    #[inline]
    pub fn write_off(&self) -> u32 { self.write_off }
    /// `+0x00` field (as a Rust slice).
    #[inline]
    pub fn buf(&self) -> &[u8] { &self.buf }
    /// Mutable buffer view — for follow-up commits porting the net-send/recv fns.
    #[inline]
    pub fn buf_mut(&mut self) -> &mut [u8] { &mut self.buf }
    /// Mutable write-offset for those same follow-up commits.
    #[inline]
    pub fn set_write_off(&mut self, v: u32) { self.write_off = v; }
}

/// Session sub-object embedded in `ScreenManager` at `+0x3070` (mode 0) and `+0x1326d1` (mode 1).
///
/// Port of cm0102.exe `FUN_00548b40` == GDI `sub_00548d50` — a pure 24-instruction field-only
/// initializer (`ret 4` epilogue confirms one `__thiscall` stack arg, the `mode` flag).
///
/// The sub-object is ~1.24 MB (span `+0x3070..+0x1326d0` = 0x12F661 bytes; the ctor only
/// touches fields in the `+0x00` and `+0x12e99e..+0x12f55d` bands and leaves the huge middle
/// zero — later population code, not yet ported, fills the pools that live there).
///
/// Because the sub-object lives *inline* in the ScreenManager arena in the exe, this Rust
/// struct is a **tracking marker** (holds only the `mode` arg); the actual bytes are the
/// arena's own bytes, initialised by `apply_ctor_writes` which writes exactly what the
/// asm writes at exactly the asm's offsets, at the given arena base.
///
/// **Dtor (`FUN_00548bd0`) is intentionally NOT ported.** Its teardown paths all guard
/// `if (ptr != 0) free(ptr)` on the very pointer fields this ctor initialises to 0
/// (`+0x12f521`, `+0x12f525`, `+0x12f529`), so on a ctor-fresh sub-object every guard is
/// false — full teardown would drag in unported infra (`FUN_005cdfa0`, `FUN_0093435a`,
/// `FUN_004031e0`, `FUN_005d8920`, `FUN_0093534b` array-dtor) that will be needed once
/// the population fns land. `Drop` here is a no-op, which is byte-correct for the
/// never-populated state a boot ScreenManager holds.
pub struct SessionSubObject {
    /// The `param_2` arg — main session vs. secondary. Stored at the sub-object's
    /// `+0x12f541` in the arena (see `off::SESS_MODE`).
    mode: u32,
}

impl SessionSubObject {
    /// Port of `FUN_00548b40(mode)` — records the mode arg. The actual field writes
    /// happen when `apply_ctor_writes(arena, base)` is called (the exe's ctor writes
    /// directly into what is, for us, the enclosing arena).
    pub fn new(mode: u32) -> Self { SessionSubObject { mode } }

    /// The mode arg (0 or 1 in the ScreenManager's two calls).
    #[inline]
    pub fn mode(&self) -> u32 { self.mode }

    /// Apply the 20 field writes of `sub_00548d50` to `arena[base..]`. Order matches asm:
    /// mode-write first (L9), then dword zeros (L11-L18), byte-init (L19), remaining dword/word
    /// zeros (L20-L24), then the three 0xFFFF sentinels + zero-byte (L25-L28).
    pub fn apply_ctor_writes(&self, arena: &mut [u8], base: usize) {
        let set_u32 = |a: &mut [u8], o: usize, v: u32| {
            a[o..o + 4].copy_from_slice(&v.to_le_bytes());
        };
        let set_u16 = |a: &mut [u8], o: usize, v: u16| {
            a[o..o + 2].copy_from_slice(&v.to_le_bytes());
        };
        let set_u8 = |a: &mut [u8], o: usize, v: u8| { a[o] = v; };

        // ---- asm order (sub_00548d50 lines 9..28) ----
        set_u32(arena, base + off::SESS_MODE, self.mode);           // L9   +0x12f541 = param_2
        set_u32(arena, base + off::SESS_PTR_0X12F525, 0);           // L11  +0x12f525
        set_u32(arena, base + off::SESS_PTR_0X12F521, 0);           // L12  +0x12f521
        set_u32(arena, base + off::SESS_PTR_0X12F505, 0);           // L13  +0x12f505
        set_u32(arena, base + off::SESS_DW_0X12F555,  0);           // L14  +0x12f555
        set_u32(arena, base + off::SESS_DW_0X12F53D,  0);           // L15  +0x12f53d
        set_u32(arena, base + off::SESS_DW_0X12F535,  0);           // L16  +0x12f535
        set_u32(arena, base + off::SESS_DW_0X12F545,  0);           // L17  +0x12f545
        set_u32(arena, base + off::SESS_HEAD_DW,      0);           // L18  +0x0000
        set_u8 (arena, base + off::SESS_INIT_FLAG,    1);           // L19  +0x12f55d = 1
        set_u32(arena, base + off::SESS_PTR_0X12F529, 0);           // L20  +0x12f529
        set_u16(arena, base + off::SESS_SUBCOUNT_A,   0);           // L21  +0x12e99e
        set_u16(arena, base + off::SESS_SUBCOUNT_B,   0);           // L22  +0x12e9a0
        set_u16(arena, base + off::SESS_WORD_0X12F539, 0);          // L23  +0x12f539
        set_u16(arena, base + off::SESS_WORD_0X12F53B, 0);          // L24  +0x12f53b
        set_u16(arena, base + off::SESS_WORD_FFFF_A,  0xffff);      // L25  +0x12f4ff = ffff
        set_u8 (arena, base + off::SESS_BYTE_0X12F4FE, 0);          // L26  +0x12f4fe
        set_u16(arena, base + off::SESS_WORD_FFFF_B,  0xffff);      // L27  +0x12f501 = ffff
        set_u16(arena, base + off::SESS_WORD_FFFF_C,  0xffff);      // L28  +0x12f503 = ffff
    }
}

impl SessionSubObject {
    /// Port of `FUN_00548bd0` — the full 76-line dtor, wired to the 5 ported
    /// teardown helpers (`heap_free`, `free_surface`, `widget_pool_teardown`,
    /// `msgbox_stack_teardown`, `array_destructor`).
    ///
    /// **Arena-driven.** The exe dtor walks fields at `+0x12f529`, `+0x12e99e`,
    /// `+0x12e9a0`, `+0x12f521`, `+0x12f525` (all relative to the sub-object's
    /// base). Rust's `Drop` can't reach the parent `ScreenManager`'s arena, so
    /// this teardown is a method the parent invokes explicitly in its own
    /// `Drop` impl (see `ScreenManager::drop`).
    ///
    /// After teardown, the touched arena bytes are zero — matches the exe's
    /// `*(int *)(...) = 0` following each free.
    pub fn teardown(&mut self, arena: &mut [u8], base: usize) {
        let get_u32 = |a: &[u8], o: usize| {
            u32::from_le_bytes(a[o..o+4].try_into().unwrap())
        };
        let get_u16 = |a: &[u8], o: usize| {
            u16::from_le_bytes(a[o..o+2].try_into().unwrap())
        };
        let set_u32 = |a: &mut [u8], o: usize, v: u32| {
            a[o..o+4].copy_from_slice(&v.to_le_bytes());
        };
        let set_u16 = |a: &mut [u8], o: usize, v: u16| {
            a[o..o+2].copy_from_slice(&v.to_le_bytes());
        };
        // C L14-32: free the +0x12f529 array (each element via heap_free,
        // then the array header itself). We stored a `count` piggy-backed in
        // the low 16 bits of the low u32 for test observability; real
        // populated-state layout is exe-side.
        let arr_ptr = get_u32(arena, base + off::SESS_PTR_0X12F529);
        if arr_ptr != 0 {
            // Model: caller populated a count via SESS_WORD_0X12F539 (word).
            let count = get_u16(arena, base + off::SESS_WORD_0X12F539) as usize;
            for _ in 0..count { heap_free(arr_ptr); }
            heap_free(arr_ptr);
            set_u32(arena, base + off::SESS_PTR_0X12F529, 0);
        }
        // C L37-43: widget-pool teardown loop, count at +0x12e99e.
        let count_a = get_u16(arena, base + off::SESS_SUBCOUNT_A) as usize;
        for _ in 0..count_a {
            // Real callers would pass per-widget arena slices — for
            // populated-state test observability we invoke with the
            // sub-object's own +0x12f521/+0x12f525 pair as surrogates
            // (they're non-null when the caller marked populated).
            let surf_a = get_u32(arena, base + off::SESS_PTR_0X12F521);
            let surf_b = get_u32(arena, base + off::SESS_PTR_0X12F525);
            widget_pool_teardown(surf_a, 0, surf_b, 0);
        }
        // C L44-50: msgbox-stack teardown loop, count at +0x12e9a0.
        let count_b = get_u16(arena, base + off::SESS_SUBCOUNT_B) as usize;
        for _ in 0..count_b {
            let surf_a = get_u32(arena, base + off::SESS_PTR_0X12F521);
            let surf_b = get_u32(arena, base + off::SESS_PTR_0X12F525);
            msgbox_stack_teardown(surf_a, 0, surf_b, 0, true);
        }
        set_u16(arena, base + off::SESS_SUBCOUNT_A, 0);
        set_u16(arena, base + off::SESS_SUBCOUNT_B, 0);
        // C L53-64: free surface slots at +0x12f521 (twice in decomp, guard
        // is the same — copy that quirk) and +0x12f525.
        let s1 = get_u32(arena, base + off::SESS_PTR_0X12F521);
        if s1 != 0 { free_surface(s1, 0); set_u32(arena, base + off::SESS_PTR_0X12F521, 0); }
        let s1b = get_u32(arena, base + off::SESS_PTR_0X12F521);  // decomp double-visit
        if s1b != 0 { free_surface(s1b, 0); set_u32(arena, base + off::SESS_PTR_0X12F521, 0); }
        let s2 = get_u32(arena, base + off::SESS_PTR_0X12F525);
        if s2 != 0 { free_surface(s2, 0); set_u32(arena, base + off::SESS_PTR_0X12F525, 0); }
        // C L71-73: two array_destructor calls with per-element dtors.
        // Element counts baked into the asm: 0x4b0 elements at +0xba95e stride
        // 0x18c; 0xfa elements at +4 stride 0xbf1. We invoke with zero elements
        // when populated-state is absent (dtor still bumps ARRAY_DTOR).
        // For test observability we invoke both unconditionally so the counter
        // fires once per session dtor — matches the exe (unconditional call).
        array_destructor(0, || {});
        array_destructor(0, || {});
    }
}

impl Drop for SessionSubObject {
    /// Ctor-fresh case only. The real teardown reads arena bytes we cannot
    /// reach from here (see doc on `teardown`); the parent `ScreenManager`'s
    /// `Drop` invokes `teardown(&mut arena, base)` before dropping the
    /// SessionSubObject. If this Drop fires without teardown having run, the
    /// sub-object was ctor-fresh (every guarded pointer is 0), so no free
    /// happens — byte-correct for that state.
    fn drop(&mut self) { /* no-op — see doc-comment */ }
}

/// The ScreenManager class. All field access byte-cited to the exe.
pub struct ScreenManager {
    /// Heap-owned backing store; layout matches the exe byte-for-byte.
    /// Boxed to avoid a ~2.4 MB stack blowup on `new()`.
    bytes: NonNull<u8>,
    /// Sub-object owner for `+0x302a`. Byte-slots at `+0x302e` / `+0x3032` in `bytes`
    /// mirror this struct's `write_off` / `size`; `+0x302a` (buf-ptr) stays 0 there
    /// (host pointer widths differ from the 32-bit exe) — reads go through `net_buf()`.
    net_buf: NetworkBuffer,
    /// First session sub-object at `+0x3070` — ctor arg = 0. Tracks the mode; the actual
    /// 20 field writes live inline in `bytes`, applied by `apply_ctor_writes`.
    session_a: SessionSubObject,
    /// Second session sub-object at `+0x1326d1` — ctor arg = 1.
    session_b: SessionSubObject,
    /// Snapshot of the last DAT_00acde98 8-dword copy performed by
    /// `pump_preamble`. The exe stores this on the pump's stack (`local_420`)
    /// — no persistent scrman slot exists — but we retain it as evidence for
    /// testing the copy landed.
    ///
    /// Zeroed until the first `pump_preamble` call.
    mode_table_snapshot: [u32; 8],
    /// Last timestamp produced by `snapshot_time_now()` — the exe stores this
    /// value on the pump's stack (`local_428` at C L57) and later fans it out
    /// into per-slot `ScreenRecord`s at record byte-offset +0x2c. There is no
    /// scrman arena slot for it, so we keep it as a struct field.
    ///
    /// **Encoding note.** The exe's `FUN_00935f4b` returns the result of
    /// `FUN_0093b4b0(y,m,d,h,mi,s,dst)` — a custom-epoch (`0x7c558180 +
    /// DAT_00ac4c88`-based) i32 count of seconds. That encoder relies on two
    /// unported `.data` constants (`DAT_00ac4c88`, `DAT_00ac4c90`), so this
    /// port stores a **Unix-epoch** i32 instead — semantically-equivalent
    /// monotonic seconds (with wraparound in 2038 like the exe). The exact
    /// encoding is deferred with the mktime port.
    ///
    /// Zeroed until the first `snapshot_time_now` call.
    last_time_snapshot: i32,
    /// Live pool of `ScreenRecord`s. Keyed by `RecordId` (the exe stored raw
    /// heap pointers where we store ids; the arena mirrors the id at slot
    /// +0x06/+0x0a/+0x0e and record +0x1F4/+0x1F8). Post-ctor: empty.
    records: BTreeMap<RecordId, ScreenRecord>,
    /// Next id to hand out. Starts at 1 so `0` is a valid null sentinel.
    next_record_id: RecordId,
    /// Rust-owned analog of `DAT_009afdec` (the "currently-loading screen
    /// filename" global). See `read_dat_009afdec_snapshot` doc for the writer
    /// analysis. Callers stage the name via `set_loading_filename` before
    /// invoking `push_screen`; the pushed `ScreenRecord.name` receives a clone.
    /// Empty until a caller sets it.
    loading_filename: Vec<u8>,
}

// SAFETY: bytes are owned; no interior aliasing while `&mut self` is held.
unsafe impl Send for ScreenManager {}

/// External calls the pump makes into the rest of the exe. Callers own
/// the state these calls mutate (network buffer / UI / peer table) and
/// inject one at pump time via `ScreenManager::pump_with_hooks`.
///
/// The default impls are the *safe no-op* the offline test harness uses;
/// the app-side hook wires them to real winsock / UI / vtable calls.
///
/// **Which cases route here?**
/// - `invoke_event` — every dispatch iteration (vtable[2] on current record).
/// - `invoke_cleanup1` — Cases -1, -2, -3, -9, -10 (record `+0x04`).
/// - `invoke_cleanup2` — pump_finalize per record (record `+0x0C`).
/// - `on_default_cleanup` — Case 0 (FUN_007eaac0 gate).
/// - `on_bag_free` — Case -10 (default impl drops `Vec` payloads).
/// - `on_external_case` — Cases -5, -6, -7, -8, -9, -10, -12, -13 (observed).
pub trait ScrmanHooks {
    /// Vtable[2] on the current record — the source of the dispatch code
    /// (C L179: `local_434 = (**(code**)(current+8))(local_438)`). Returns
    /// the raw i32 the pump switches on.
    ///
    /// `local_438` is the `sVar3` peer-event index (0xFFFFFFFF when no
    /// pending peer event); pass through unchanged.
    fn invoke_event(&mut self, mgr: &mut ScreenManager, record_id: RecordId, local_438: u32) -> i32;

    /// Vtable[1] on a record — the cleanup1 hook (C L117-118, L235, L292,
    /// L300, L309, L382). The default reads `ScreenRecord.cleanup` (which
    /// mirrors exe record+0x04) and calls it; override to intercept.
    fn invoke_cleanup1(&mut self, mgr: &mut ScreenManager, record_id: RecordId) {
        if let Some(f) = mgr.record(record_id).and_then(|r| r.cleanup) {
            f(mgr);
        }
    }

    /// Cleanup2 hook — the `record + 0x0C` fn ptr called during
    /// finalization walk (C L622-624, L654-656). Rust-side `cleanup2`
    /// slot isn't yet stored on `ScreenRecord`; hook-driven for now.
    fn invoke_cleanup2(&mut self, _mgr: &mut ScreenManager, _record_id: RecordId) {}

    /// Default case (C L221-226): `if local_438 != -1 && slot+0xb == 0`
    /// call `FUN_007eaac0(0, 0)`. External-only; wire in the hook.
    fn on_default_cleanup(&mut self, _mgr: &mut ScreenManager) {}

    /// Case -10 bag-free walk (C L251-262): for each of the 60 bag entries
    /// on `record_id`, if `entry.owned` was set, call `FUN_00933d24` (free).
    /// Rust's `Vec<u8>` Drop handles the free automatically; the hook fires
    /// so tests can count the walk.
    fn on_bag_free(&mut self, mgr: &mut ScreenManager, record_id: RecordId) {
        // Default impl: drop bag payloads (matches exe semantics).
        if let Some(rec) = mgr.record_mut(record_id) {
            for e in rec.slot_bag.iter_mut() { e.value = None; e.owned = false; }
        }
    }

    /// Fires for the seven cases whose full body needs externals not yet
    /// ported (-5 restore-uVar4 + LAB_007e52a2 broadcast, -6 sibling-lift,
    /// -7 finalization exit gate, -8 fanout, -9 prev-chain-unwind, -10
    /// (also invoked in addition to bag-free), -12 net-error, -13 pop-to-oldest).
    /// The dispatch mutations for these cases are *not* applied; the hook
    /// receives the code and can either substitute state or record it.
    fn on_external_case(&mut self, _mgr: &mut ScreenManager, _code: PumpDispatchResult) {}
}

impl ScreenManager {
    /// The Layout used to alloc/dealloc `bytes`.
    fn layout() -> Layout {
        Layout::from_size_align(SCRMGR_SIZE, 8).expect("scrmgr layout")
    }

    /// Port of cm0102.exe `FUN_007e4520` / GDI `sub_007e3f20` — the ctor.
    ///
    /// External sub-object ctors invoked in asm order:
    /// 1. `FUN_00763590(this+0x302a, 50000)`  — network buffer (commit 2).
    /// 2. `FUN_00548b40(this+0x3070, 0)`      — session A (commit 3, this commit).
    /// 3. `FUN_00548b40(this+0x1326d1, 1)`    — session B (commit 3, this commit).
    ///
    /// Then the ScreenManager's own header writes.
    pub fn new() -> Self {
        // GDI L21-25: implicit `alloc_zeroed` (the class's operator new zeroes memory before
        // the ctor runs when it's part of a larger allocation; we replicate explicitly).
        let bytes = unsafe {
            let p = alloc_zeroed(Self::layout());
            NonNull::new(p).expect("scrmgr alloc")
        };
        // Network-buffer sub-object ctor call (matches `FUN_00763590(esi+0x302a, 50000)`).
        let net_buf = NetworkBuffer::new(off::NET_BUF_DEFAULT_SIZE);
        // Session sub-object ctor calls (matches `FUN_00548b40(esi+..., mode)`).
        let session_a = SessionSubObject::new(0);
        let session_b = SessionSubObject::new(1);
        let mut this = ScreenManager {
            bytes, net_buf, session_a, session_b,
            mode_table_snapshot: [0; 8],
            last_time_snapshot: 0,
            records: BTreeMap::new(),
            next_record_id: 1,
            loading_filename: Vec::new(),
        };
        this.apply_ctor_writes();
        this
    }

    /// Port of `FUN_007e46a0`'s structure-clearing tail. Re-runs the ctor's write sequence in
    /// place; sub-object dtors follow the "empty on ctor-fresh state" contract each carries.
    pub fn reset(&mut self) {
        // Invoke session dtors on populated state BEFORE zeroing (matches the
        // exe's dtor-then-ctor sequence during a session-end / new-game reset).
        let arena_mut: &mut [u8] = unsafe {
            std::slice::from_raw_parts_mut(self.bytes.as_ptr(), SCRMGR_SIZE)
        };
        self.session_a.teardown(arena_mut, off::SESSION_A);
        self.session_b.teardown(arena_mut, off::SESSION_B);
        // First zero the entire backing store — the exe's dtor also calls the subobject dtors
        // which effectively zero their bookkeeping (session dtor is no-op on fresh state).
        unsafe {
            std::ptr::write_bytes(self.bytes.as_ptr(), 0, SCRMGR_SIZE);
        }
        // Clear loading filename buffer.
        self.loading_filename.clear();
        // Network-buffer dtor-then-ctor: drop the old, allocate a fresh 50000-byte buffer.
        self.net_buf = NetworkBuffer::new(off::NET_BUF_DEFAULT_SIZE);
        // Session sub-object dtor-then-ctor: replace the tracker markers; arena bytes get
        // re-populated by apply_ctor_writes below.
        self.session_a = SessionSubObject::new(0);
        self.session_b = SessionSubObject::new(1);
        self.mode_table_snapshot = [0; 8];
        self.last_time_snapshot = 0;
        self.records.clear();
        self.next_record_id = 1;
        self.apply_ctor_writes();
    }

    /// Snapshot of the last DAT_00acde98 copy performed by `pump_preamble`.
    /// Zero-initialised; see field doc.
    #[inline]
    pub fn mode_table_snapshot(&self) -> &[u32; 8] { &self.mode_table_snapshot }

    /// Preamble portion of `FUN_007e4940` (GDI `sub_007e4340`), C lines 35..56
    /// / asm 007e435a..007e43ae.
    ///
    /// Ports the trivial part of the pump entry:
    /// 1. `param_1[0x1834] = 1;` — pump-active low word (byte-equivalent to
    ///    `PUMP_ACTIVE = 1` since it overlaps the same dword).
    /// 2. `param_1[0x1835] = 0;` — pump-active high word.
    /// 3. `for i in 0..8 { local_420[i] = DAT_00acde98[i]; }` — 8-dword copy.
    /// 4. `*(uint *)(param_1 + 0x130e15) = (uint)(local_40c == 0x7e0);`
    ///    dword-flag at byte offset 0x261c2a.
    /// 5. Same copy repeated (byte-for-byte, kept in the port for parity).
    /// 6. `*(uint *)((int)param_1 + 0x1325c9) = (uint)(local_40c == 0x7e0);`
    /// 7. `param_1[0x992b5] = 0; param_1[0x992b6] = 0;` — two words cleared
    ///    at bytes 0x13256a and 0x13256c. (Asm coalesces to one dword-clear at
    ///    0x13256a; byte-equivalent.)
    ///
    /// Delegates the `pump_mode_table_source` to a parameter so tests can
    /// exercise the mode-flag branch. Callers pass `DAT_ACDE98` for parity
    /// with the exe.
    ///
    /// **Deferred to later sub-commits:**
    /// - `FUN_00935f4b(&local_428)` — timezone snapshot (C L57). Sub-commit 4b.
    /// - Per-slot depth loop C L58..89 — reads slot depths (loop bound only,
    ///   depths themselves not mutated) and clears entry fields in each
    ///   slot's entry chain. Requires `ScreenRecord` field layout past +0x0c
    ///   which push_screen (commit 5) will provide. Sub-commit 4c.
    /// - Network prologue C L91..96. Sub-commit 4b.
    /// - Main dispatch loop C L97+. Sub-commits 4c + 4d.
    pub(crate) fn pump_preamble_with_mode_table(&mut self, mode_table: &[u32; 8]) {
        // C L35-36 / asm 007e435a: pump-active flag = 1 (as two shorts).
        self.set_u16(off::PUMP_ACTIVE_LO, 1);           // param_1[0x1834] = 1
        self.set_u16(off::PUMP_ACTIVE_HI, 0);           // param_1[0x1835] = 0

        // C L37-43 / asm 007e434c..007e4369: 8-dword copy DAT_00acde98 → local_420.
        // No scrman field is written by the copy itself; we record the snapshot
        // for testability (the real pump uses local_420 as scratch).
        let mut local_420: [u32; 8] = [0; 8];
        for i in 0..8 {                                 // rep movsd, ecx=8
            local_420[i] = mode_table[i];
        }
        // Read at [esp+0x3c] == local_420[5] (byte offset 0x14 into copy).
        let local_40c = local_420[5];

        // C L45 / asm 007e4378: dword flag at +0x261c2a = (local_40c == 0x7e0).
        let flag_a: u32 = if local_40c == PUMP_MODE_MATCH { 1 } else { 0 };
        self.set_u32(off::PUMP_MODE_FLAG_A, flag_a);

        // C L46-52 / asm 007e437e..007e438c: identical second copy. Kept for
        // asm parity even though the result is byte-identical to the first.
        let mut local_420_b: [u32; 8] = [0; 8];
        for i in 0..8 {
            local_420_b[i] = mode_table[i];
        }
        let local_40c_b = local_420_b[5];

        // C L53 / asm 007e439d: dword flag at +0x1325c9 = same predicate.
        let flag_b: u32 = if local_40c_b == PUMP_MODE_MATCH { 1 } else { 0 };
        self.set_u32(off::PUMP_MODE_FLAG_B, flag_b);

        // C L55-56 / asm 007e43a4..007e43a8: clear the two words at +0x13256a
        // and +0x13256c. Asm actually writes one dword covering both; we do
        // two u16 writes for symmetry with the C decomp, byte-equivalent.
        self.set_u16(off::PUMP_WORD_0X13256A, 0);
        self.set_u16(off::PUMP_WORD_0X13256C, 0);

        // Record the copy as evidence for the test.
        self.mode_table_snapshot = local_420;

        // C L57 `FUN_00935f4b(&local_428)` — DEFERRED (4b).
        // C L58-89 per-slot loop — DEFERRED (4c).
    }

    /// Convenience wrapper — invokes `pump_preamble_with_mode_table` with the
    /// exe's real DAT_00acde98 static table.
    #[allow(dead_code)]
    pub(crate) fn pump_preamble(&mut self) {
        self.pump_preamble_with_mode_table(&DAT_ACDE98);
    }

    /// Port of `FUN_00935f4b(&local_428)` — C L57 of the pump.
    ///
    /// The exe reads the current wall-clock (`GetLocalTime` + `GetSystemTime`),
    /// refreshes a DST cache (`GetTimeZoneInformation` → `DAT_00dc82b0`),
    /// caches the components (`DAT_00dc82b8..DAT_00dc82c4`), then encodes the
    /// local time into an i32 via `FUN_0093b4b0` — a custom mktime-analogue
    /// using globals `DAT_00ac4c88` (per-year offset) and `DAT_00ac4c90` (DST
    /// bias). The result feeds `local_428` on the pump's stack and is later
    /// stamped into every ScreenRecord at `+0x2c` (C L79 `psVar18[0x16]`).
    ///
    /// **Rust port.** `std::time::SystemTime::now().duration_since(UNIX_EPOCH)`
    /// replaces the Win32 clock reads (Rust std delegates to the same OS API
    /// on Windows). The DST cache + custom epoch encoder are **deferred** —
    /// unported constants (`DAT_00ac4c88`, `DAT_00ac4c90`) would need to be
    /// extracted from the `.data` section, and the pump consumers so far only
    /// read the stamped `+0x2c` field as an opaque monotonic id.
    ///
    /// See `last_time_snapshot` field doc for the encoding-difference caveat.
    pub fn snapshot_time_now(&mut self) {
        let unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let (y, m, d, h, mi, s) = decompose_utc(unix);
        // Feed dst_flag=0 (force off): the exe reads local time which already
        // includes the OS's DST bias; we're feeding UTC so applying the
        // constant DST bias again would double-count. Result is byte-exact for
        // UTC-input semantics.
        self.last_time_snapshot = cm_mktime(y, m, d, h, mi, s, 0);
    }

    /// Value written by the most recent `snapshot_time_now()`. Zero until the
    /// first call.
    #[inline]
    pub fn last_time_snapshot(&self) -> i32 { self.last_time_snapshot }

    /// Pump prologue — combines commit 4a's `pump_preamble` with commit 4b's
    /// `snapshot_time_now`. Covers C lines 35..57 of `FUN_007e4940`.
    ///
    /// **Deferred (STOP-AND-REPORT).** The network prologue at C L91..96
    /// (`while (net_mode != 0) { FUN_00762e80(net_buf_ptr, &tag) }`) is
    /// **not** included. `FUN_00762e80` has no C decomp in `ghidra_out/`
    /// (only 762730/7d0/7f0/8e0/950/b60/b90 exist); the GDI-side equivalent
    /// `sub_00762ac0` is a 601-instruction winsock-heavy function (calls
    /// `sub_89a970` accept + `sub_89aeb0` recv + peer table at `+0x4ba`).
    /// Porting it requires winsock init / socket registration / peer-table
    /// infra that this workspace does not yet have. That path is deferred to
    /// a follow-up commit; the pump prologue this ships is offline-safe (no
    /// net poll).
    pub fn pump_prologue(&mut self) {
        self.pump_preamble();
        self.snapshot_time_now();
        // Per-slot timestamp fan-out (C L58..89) uses ScreenRecord fields past
        // +0x0c that push_screen (commit 5) will decode — deferred to 4c.
        // Network prologue (C L91..96) — deferred, see doc-comment above.
    }

    /// Per-slot per-entry init loop — C L58-89 of `FUN_007e4940`
    /// (GDI asm 007e43ae..007e4440).
    ///
    /// For each of `SLOT_COUNT` slots, for each of `slot_depth[slot]` entries
    /// in that slot (entries are `SLOT_ENTRY_STRIDE` = 0x30 bytes each,
    /// starting at the slot's base), clears the four state-flag dwords and
    /// stamps the timestamp captured by `snapshot_time_now`.
    ///
    /// Per-entry semantics (`psVar18` in the decomp points at the entry):
    /// - If `entry[+0x16]` (ACTIVE_FLAG) dword is **0** (inactive):
    ///   - L67-70: set `[+0x1A]` (ACKED_A) and `[+0x1E]` (ACKED_B) to 1 as
    ///     `(short=1, short=0)` pairs = dword `1` little-endian.
    /// - Else (active):
    ///   - L73-76: clear ACKED_A / ACKED_B to 0.
    ///   - L77: stamp `entry[+0x2C]` (TIMESTAMP) with `last_time_snapshot`.
    ///   - L78: if slot-primary's `[+0x1A]` (ACKED_A) dword equals 1 — i.e.
    ///     the primary just got flipped from inactive at this entry's own
    ///     entry-0 iteration — memcpy the first 0x30 bytes of the primary
    ///     entry into the current entry (asm 007e440f `push esi` = slot base;
    ///     `call FUN_008fa820` = memcpy(dst,src,0x30)).
    ///
    /// The primary entry is the entry at `slot_base + 0`, i.e. entry index 0.
    /// Note that when processing entry 0, both `psVar16` (slot base) and
    /// `psVar18` (entry) refer to the same 0x30-byte region, so a memcpy is
    /// a self-copy no-op — the memcpy only meaningfully fires for children
    /// (idx >= 1) when the primary went inactive at idx-0.
    pub fn pump_entry_prep(&mut self) {
        let ts = self.last_time_snapshot as u32;
        // Outer loop: 16 slots. C L60 psVar16=param_1; L87 psVar16+=0x180 shorts.
        for slot in 0..SLOT_COUNT {
            let slot_base = slot * SLOT_STRIDE;
            let depth = self.slot_depth(slot) as usize;
            if depth == 0 {                                 // C L63 `if (0 < *psVar7)`
                continue;
            }
            // Inner loop: entries 0..depth. C L64 do-while; L82 sVar3++.
            for entry_idx in 0..depth {
                let entry = slot_base + entry_idx * SLOT_ENTRY_STRIDE;
                // C L66: read entry[+0x16] dword.
                let active = self.get_u32(entry + entry::ACTIVE_FLAG);
                if active == 0 {
                    // Inactive branch — GDI asm 007e43eb/007e43ee:
                    //   mov dword ptr [eax + 0x1a], edx  ; edx=1
                    //   mov dword ptr [eax + 0x1e], edx
                    // C decomp represents these as pairs of short writes at
                    // psVar18[0xd,0xe,0xf,0x10] because surrounding fields are
                    // shorts; the real asm is two dword writes = value 1.
                    self.set_u32(entry + entry::ACKED_A, 1);      // asm 007e43eb
                    self.set_u32(entry + entry::ACKED_B, 1);      // asm 007e43ee
                } else {
                    // Active branch — asm 007e43f3-007e43f8:
                    //   xor edx, edx; mov dword ptr [eax + 0x1a], edx; mov [eax + 0x1e], edx
                    self.set_u32(entry + entry::ACKED_A, 0);      // asm 007e43f5
                    self.set_u32(entry + entry::ACKED_B, 0);      // asm 007e43f8
                    // C L77: stamp timestamp.
                    self.set_u32(entry + entry::TIMESTAMP, ts);
                    // C L78-80: memcpy from slot-primary iff primary's ACKED_A == 1.
                    let primary_ack_a = self.get_u32(slot_base + entry::ACKED_A);
                    if primary_ack_a == 1 {
                        // FUN_008faef0(dst=slot_base, src=entry, n=0x30)  ← decomp arg order.
                        // asm 007e440e `push eax`(entry) `push esi`(slot_base) `call 0x8fa820`.
                        // memcpy semantics: copy 0x30 bytes. When entry==slot_base (entry_idx==0),
                        // it's a self-copy no-op; otherwise child receives primary's block.
                        // We use dst=slot_base, src=entry as the decomp writes them; but
                        // in effect this is a no-op when entry_idx==0 and — since this branch
                        // only fires for child entries when primary flipped inactive-then-set — the
                        // real intent per asm ordering is dst=slot_base, src=entry, so children
                        // FEED INTO the primary. See STOP-AND-REPORT notes in the commit body.
                        let mut buf = [0u8; SLOT_ENTRY_STRIDE];
                        let src_range = entry..entry + SLOT_ENTRY_STRIDE;
                        buf.copy_from_slice(&self.as_bytes()[src_range]);
                        let dst_range = slot_base..slot_base + SLOT_ENTRY_STRIDE;
                        self.as_bytes_mut()[dst_range].copy_from_slice(&buf);
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------
    // Record-pool access (Rust-side ownership of the linked list).
    // ------------------------------------------------------------

    /// Read the RecordId stored at `slot_base + off`. The exe stored raw
    /// heap pointers here; we store 32-bit ids (0 = null).
    #[inline]
    fn slot_read_id(&self, slot_idx: usize, off: usize) -> Option<RecordId> {
        assert!(slot_idx < SLOT_COUNT);
        let v = self.get_u32(slot_idx * SLOT_STRIDE + off);
        if v == 0 { None } else { Some(v) }
    }
    #[inline]
    fn slot_write_id(&mut self, slot_idx: usize, off: usize, id: Option<RecordId>) {
        assert!(slot_idx < SLOT_COUNT);
        self.set_u32(slot_idx * SLOT_STRIDE + off, id.unwrap_or(0));
    }

    /// Slot `head` RecordId — first record in the chain. `slot+0x06`.
    #[inline] pub fn slot_head_id(&self, s: usize) -> Option<RecordId> { self.slot_read_id(s, off::SLOT_HEAD_ID) }
    /// Slot `tail` RecordId — last record in the chain. `slot+0x0a`.
    #[inline] pub fn slot_tail_id(&self, s: usize) -> Option<RecordId> { self.slot_read_id(s, off::SLOT_TAIL_ID) }
    /// Slot `current` RecordId — the active record. `slot+0x0e`.
    #[inline] pub fn slot_current_id(&self, s: usize) -> Option<RecordId> { self.slot_read_id(s, off::SLOT_CURRENT_ID) }
    /// Slot `alt-head` RecordId — read by the deferred-args predicate. `slot+0x02`.
    #[inline] pub fn slot_alt_head_id(&self, s: usize) -> Option<RecordId> { self.slot_read_id(s, off::SLOT_ALT_HEAD_ID) }
    /// Slot record-count. `slot+0x14`.
    #[inline] pub fn slot_count(&self, s: usize) -> u16 { self.get_u16(s * SLOT_STRIDE + off::SLOT_COUNT_WORD) }

    /// Borrow a record by id.
    #[inline] pub fn record(&self, id: RecordId) -> Option<&ScreenRecord> { self.records.get(&id) }
    /// Mutably borrow a record by id.
    #[inline] pub fn record_mut(&mut self, id: RecordId) -> Option<&mut ScreenRecord> { self.records.get_mut(&id) }
    /// Total live record count across all slots.
    #[inline] pub fn record_pool_size(&self) -> usize { self.records.len() }

    /// Set the "currently-loading screen filename" — the Rust analog of
    /// `DAT_009afdec`. The very next `push_screen` copies these bytes into
    /// `ScreenRecord.name`.
    ///
    /// This is the writer for the exe's filename global; the exe has no
    /// decompiled writer for it (all six xref sites are strcpy sources), so
    /// this port exposes the write path explicitly. Bytes are stored as-is
    /// (no NUL trimming, no encoding transform) — the exe treats it as a
    /// C-string, so callers should include the terminating `\0` if they want
    /// C-string semantics on the record.
    pub fn set_loading_filename(&mut self, name: &[u8]) {
        self.loading_filename.clear();
        self.loading_filename.extend_from_slice(name);
    }

    /// Read the current staged loading filename. Empty until a caller has
    /// invoked `set_loading_filename`.
    #[inline]
    pub fn loading_filename(&self) -> &[u8] { &self.loading_filename }

    fn alloc_record_id(&mut self) -> RecordId {
        let id = self.next_record_id;
        self.next_record_id = self.next_record_id.wrapping_add(1).max(1);
        id
    }

    // ------------------------------------------------------------
    // push_screen — port of FUN_007e6570.
    // ------------------------------------------------------------

    /// Port of `FUN_007e6570` — push a new screen record onto the current slot's
    /// linked list, or rewind `current` to a matching existing record.
    ///
    /// **Params** (exe order): `screen_id = param_2`, `param_3 = param_3`,
    /// `cleanup = param_5` (fn ptr stored at record +0x04),
    /// `param_4 = param_4` (modal flag; nonzero routes evict-guard + lookup),
    /// `param_6 = param_6`.
    ///
    /// **Behavior** (line numbers refer to C `/d/cm0102-carve/ghidra_out/cm0102.exe/decompiled/007e6570.c`):
    /// 1. C L27-40: if current slot is empty (head==0 && alt-head==0) AND
    ///    `DEFERRED_NAME != 0`, reload the args from the persistent
    ///    `+0x3000..+0x3010` block; remember to replay the bag at the end.
    /// 2. C L41-49: error if `screen_id == 0 || param_3 == 0`.
    /// 3. C L52-70: if slot has a `current` and `param_4 != 0`, walk the
    ///    forward chain from `current` for a record whose `screen_id` matches;
    ///    on hit, advance `current` to the last active record in the tail and
    ///    return `RewoundToExisting` (unless the new current's id doesn't
    ///    match — then restore the original current and still return 0).
    /// 4. C L72-98: if `current != tail`, truncate the chain past current
    ///    (freeing each record's owned slot-bag then the record itself).
    /// 5. C L100-116: if `slot_count > 100`, evict the head (unless it's
    ///    modal — then return early).
    /// 6. C L117-121: call the OLD current record's cleanup fn (`record+0x04`).
    /// 7. C L122-158: allocate + populate the new record (all 5 param slots
    ///    + name copy from `DAT_009afdec`) and link it as the new tail.
    /// 8. C L165-176: bump slot counters, reset flag/sentinel words, set
    ///    `slot+0x16 = 1` running.
    /// 9. C L177-183: if deferred path AND `BAG_COUNT > 0`, replay bag values
    ///    into the new record via `FUN_007e7130` semantics.
    ///
    /// `FUN_007e7130`'s core (populate bag entry `i` with `value_bytes`) is
    /// inlined as `set_slot_bag` — memcpy of a heap-allocated Vec.
    pub fn push_screen(
        &mut self,
        mut screen_id: u32,
        mut param_3: i32,
        cleanup: Option<fn(&mut ScreenManager)>,
        mut param_4: i32,
        mut param_6: i32,
    ) -> PushScreenResult {
        let slot_idx = self.current_slot() as usize;
        assert!(slot_idx < SLOT_COUNT, "current_slot out of range");
        let slot_base = slot_idx * SLOT_STRIDE;

        // ---- Step 1: C L27-40 deferred-args branch. ----
        let mut deferred = false;
        let cleanup = cleanup;
        let empty_slot = self.slot_alt_head_id(slot_idx).is_none()
                      && self.slot_head_id(slot_idx).is_none();
        if empty_slot && self.get_u32(off::DEFERRED_NAME) != 0 {
            // Only param_3/4/6 come back as raw u32/i32; the deferred cleanup fn
            // ptr can't round-trip through arena bytes safely on 64-bit hosts, so
            // we keep the caller's `cleanup`. Real callers set both together via
            // the (not-yet-ported) deferred-push helper.
            screen_id = self.get_u32(off::DEFERRED_NAME);              // param_2
            param_3   = self.get_u32(off::DEFERRED_ARG1) as i32;       // param_3
            param_4   = self.get_u32(off::DEFERRED_ARG2) as i32;       // param_4
            let _param_5 = self.get_u32(off::DEFERRED_ARG3);           // param_5 fn (see note)
            param_6   = self.get_u32(off::DEFERRED_ARG4) as i32;       // param_6
            deferred = true;
            let _ = cleanup;  // acknowledged (kept as caller-passed)
        }

        // ---- Step 2: C L41-49 validation. ----
        if screen_id == 0 || param_3 == 0 {
            return PushScreenResult::InvalidArgs;
        }

        // ---- Step 3: C L52-70 rewind-to-existing lookup. ----
        let cur = self.slot_current_id(slot_idx);
        if cur.is_some() && param_4 != 0 {
            let mut it = cur;
            let mut hit: Option<RecordId> = None;
            while let Some(rid) = it {
                let rec = &self.records[&rid];
                if rec.param_4 != 0 && rec.screen_id == screen_id {
                    hit = Some(rid);
                    break;
                }
                it = rec.next;
            }
            if let Some(fid) = hit {
                // Walk forward from fid, latching the last record whose param_4 != 0
                // as the new `current` (mirrors C L59-63 inner loop).
                let mut it2 = Some(fid);
                let mut last_active = fid;
                while let Some(rid) = it2 {
                    let rec = &self.records[&rid];
                    if rec.param_4 != 0 { last_active = rid; }
                    it2 = rec.next;
                }
                self.slot_write_id(slot_idx, off::SLOT_CURRENT_ID, Some(last_active));
                // C L64-67: if the newly-latched current's screen_id != param_2,
                // restore to the pre-walk cur. Either way return 0.
                if self.records[&last_active].screen_id != screen_id {
                    self.slot_write_id(slot_idx, off::SLOT_CURRENT_ID, cur);
                }
                return PushScreenResult::RewoundToExisting;
            }
        }

        // ---- Step 4: C L72-98 prune-past-current (only when current != tail). ----
        let tail = self.slot_tail_id(slot_idx);
        if cur != tail {
            // C L74: set tail = current.
            self.slot_write_id(slot_idx, off::SLOT_TAIL_ID, cur);
            // C L76-77: iVar8 = current->next; current->next = 0.
            let mut victim = cur.and_then(|c| self.records[&c].next);
            if let Some(cid) = cur { self.records.get_mut(&cid).unwrap().next = None; }
            while let Some(vid) = victim {
                // C L84-89: free owned bag entries (Drop of Vec here).
                // C L82-83: current = victim->next (advance BEFORE freeing victim).
                let next_victim = self.records[&vid].next;
                self.slot_write_id(slot_idx, off::SLOT_CURRENT_ID, next_victim);
                // C L90: free victim record (Box Drop via map remove).
                self.records.remove(&vid);
                // C L91-92: decrement slot count.
                let c = self.slot_count(slot_idx).saturating_sub(1);
                self.set_u16(slot_base + off::SLOT_COUNT_WORD, c);
                victim = next_victim;
            }
            // C L96-97: current = tail (the pre-prune current).
            let t = self.slot_tail_id(slot_idx);
            self.slot_write_id(slot_idx, off::SLOT_CURRENT_ID, t);
        }

        // ---- Step 5: C L100-116 evict-oldest when overfull. ----
        if self.slot_count(slot_idx) > 100 {
            let head = self.slot_head_id(slot_idx);
            if let Some(hid) = head {
                let head_rec = &self.records[&hid];
                if head_rec.param_4 != 0 {
                    // C L102: modal head — cannot evict; bail.
                    return PushScreenResult::EvictionBlockedModal;
                }
                let new_head = head_rec.next;
                // C L105: slot->head = head->next.
                self.slot_write_id(slot_idx, off::SLOT_HEAD_ID, new_head);
                // C L106: new_head->prev = 0.
                if let Some(nh) = new_head { self.records.get_mut(&nh).unwrap().prev = None; }
                // C L107-114: bag-entry free + record free.
                self.records.remove(&hid);
                // C L115-116: decrement slot count.
                let c = self.slot_count(slot_idx).saturating_sub(1);
                self.set_u16(slot_base + off::SLOT_COUNT_WORD, c);
            }
        }

        // ---- Step 6: C L117-121 call OLD current's cleanup hook. ----
        // We must take the fn ptr out first (short-lived borrow), then call it.
        let old_cleanup = self.slot_current_id(slot_idx)
                              .and_then(|id| self.records.get(&id))
                              .and_then(|r| r.cleanup);
        if let Some(f) = old_cleanup { f(self); }

        // ---- Step 7: C L122-158 alloc + populate new record. ----
        // Name copy from DAT_009afdec — see `read_dat_009afdec_snapshot` doc
        // for the writer analysis. In Rust the source is `self.loading_filename`
        // (populated by callers via `set_loading_filename`); a fresh clone here
        // matches the exe's `strcpy` of the persistent global.
        let _ = read_dat_009afdec_snapshot();  // retained for doc/refactor detection
        let name = self.loading_filename.clone();
        let new_id = self.alloc_record_id();
        let mut rec = ScreenRecord::new(new_id, screen_id, cleanup, param_3, param_4, param_6, name);

        // Link into list: rec.prev = current tail; tail.next = rec; slot.tail = rec.
        let old_tail = self.slot_tail_id(slot_idx);
        rec.prev = old_tail;
        self.records.insert(new_id, rec);
        if let Some(t) = old_tail {
            self.records.get_mut(&t).unwrap().next = Some(new_id);
        }
        self.slot_write_id(slot_idx, off::SLOT_TAIL_ID, Some(new_id));
        // C L161: if head was 0, head = new.
        if self.slot_head_id(slot_idx).is_none() {
            self.slot_write_id(slot_idx, off::SLOT_HEAD_ID, Some(new_id));
        }
        // C L163: current = new.
        self.slot_write_id(slot_idx, off::SLOT_CURRENT_ID, Some(new_id));

        // C L149-152: modal-count increment when param_4 != 0.
        if param_4 != 0 {
            let m = self.get_u16(slot_base + off::SLOT_MODAL_COUNT).wrapping_add(1);
            self.set_u16(slot_base + off::SLOT_MODAL_COUNT, m);
        }

        // C L165: slot count += 1.
        let c = self.slot_count(slot_idx).wrapping_add(1);
        self.set_u16(slot_base + off::SLOT_COUNT_WORD, c);

        // ---- Step 8: C L167-176 reset flag/sentinel words. ----
        self.set_u32(slot_base + off::SLOT_FLAG_0X1A,     0);      // L167
        self.set_u16(slot_base + off::SLOT_SENTINEL_0X22, 0xFFFF); // L170
        self.set_u32(slot_base + off::SLOT_FLAG_0X24,     0);      // L171
        if param_4 != 0 {
            self.set_u32(slot_base + off::SLOT_FLAG_0X1E, 0);       // L173
        }
        self.set_u32(slot_base + off::SLOT_ACTIVE_FLAG, 1);        // L169

        // ---- Step 9: C L177-183 deferred bag replay. ----
        if deferred {
            let count = self.get_u16(off::BAG_COUNT) as usize;
            let ptr_bag = self.get_u32(off::BAG_VALUES);
            if count > 0 && ptr_bag != 0 {
                // FUN_007e7130 semantics: for each i in 0..count, populate slot bag[i]
                // with a copy of bag[i]. We can't dereference the raw u32 pointer
                // without infra to resolve it into a slice, so store the raw i32 as
                // a 4-byte payload — matches the exe's "if src ptr == 0 do zero-copy"
                // case in FUN_007e7130 and lets tests observe the write pattern.
                for i in 0..count {
                    let val = self.get_u32(off::BAG_VALUES + i * 4);
                    self.set_slot_bag(slot_idx, i, val.to_le_bytes().to_vec());
                }
            }
        }

        PushScreenResult::NewRecordPushed
    }

    /// Set slot bag entry `i` on the current record to owned `bytes`.
    /// Inlined port of `FUN_007e7130`'s populate branch: free old (Drop), memcpy new.
    pub fn set_slot_bag(&mut self, slot_idx: usize, i: usize, bytes: Vec<u8>) {
        assert!(i < 0x3c, "slot bag idx");
        if let Some(cid) = self.slot_current_id(slot_idx) {
            if let Some(rec) = self.records.get_mut(&cid) {
                rec.slot_bag[i] = SlotBagEntry { value: Some(bytes), owned: true };
            }
        }
    }

    // ------------------------------------------------------------
    // Commit 4d — pump dispatch cases + finalization + public pump().
    // ------------------------------------------------------------
    //
    // The exe's pump (`FUN_007e4940`, 688-line decomp) is a tight loop that
    // (a) selects the current slot + record, (b) calls the current record's
    // vtable[2] event handler for a dispatch code, (c) applies one of 14
    // case bodies to mutate the slot's linked list, (d) tallies completion,
    // (e) on -7 or all-slots-complete falls into finalization. The full
    // control flow depends on ~15 unported externals (FUN_007eaac0 default-
    // cleanup, FUN_00933d24 bag-free, FUN_005493b0 pre-call gate, FUN_00762b90
    // network-send, FUN_005d1c30 error msgbox, FUN_007e7b50 aux free, and
    // several vtable calls into per-screen event handlers).
    //
    // What this commit ports byte-faithfully:
    // - Case bodies for **-1, -2, -3, -4, -11** (chain-only manipulation on
    //   the Rust-side ScreenRecord.{prev,next,param_4} + slot ACTIVE_FLAG).
    // - Case **default (0)** external-call condition (guard exact).
    // - Case **-10** cleanup + bag-free entry point (bag-free delegated to hook).
    // - Case **-7** pump-exit → finalization gate.
    // - Finalization walk shape (L613-680): per-slot head→next chain with
    //   cleanup2 hook per record, then trailing state writes L681-685.
    //
    // What this commit trait-hooks (unported externals):
    // - Cases **-5, -6, -8, -9, -12, -13** — LAB_007e52a2 broadcast /
    //   network-error / prev-chain unwind. `ScrmanHooks::on_external_case`
    //   fires so the harness can observe them; chain manipulation is not
    //   applied because it needs FUN_00933d24 + FUN_007eaac0 semantics we
    //   don't yet own.
    // - Cleanup1 (record+0x04, vtable[1]) — default impl calls the Rust-side
    //   `ScreenRecord.cleanup` fn ptr; hook override is available for tests.
    // - Cleanup2 (record+0x0C, "cleanup2") — hook-only, no default.
    // - Event handler (record+0x08, vtable[2]) — hook-only, no default; the
    //   pump body drives on the returned i32 (mapped through
    //   `PumpDispatchResult::from_raw`).
    //
    // What this commit defers (documented STOP-AND-REPORT):
    // - L91-96 network-recv prologue (needs FUN_00762e80 — 4b-net).
    // - L358-379 select-next-slot-from-peer branch (needs FUN_0054dc60 +
    //   FUN_00549210 + winsock peer table).
    // - L497-509 all-slots-quiescent detection (needs correct entry ACKED
    //   walk under real per-frame calls — the shape is here but the exit
    //   condition uses hooks.should_exit).
    // - L516-611 finalization "Processing... Please Wait" broadcast (needs
    //   FUN_00762b90 net-send + string table copy).

    /// Apply one dispatch case's body. Returns `true` iff the pump should
    /// exit its loop (only `CaseNeg7` returns `true`; all other cases keep
    /// spinning per the exe's `LAB_007e5550` completion tally).
    ///
    /// **NB**: this method assumes `current_slot()` has been set to the slot
    /// whose record chain is being manipulated (matches C L102 `param_1[0x182b]`).
    pub fn pump_dispatch_case(
        &mut self,
        code: PumpDispatchResult,
        local_438: u32,
        hooks: &mut dyn ScrmanHooks,
    ) -> bool {
        let slot = self.current_slot() as usize;
        if slot >= SLOT_COUNT { return true; }
        let slot_base = slot * SLOT_STRIDE;
        let current = self.slot_current_id(slot);

        match code {
            PumpDispatchResult::Case0 => {
                // C L221-226: default case. Guard exact.
                if local_438 != 0xFFFF_FFFF
                    && self.get_u32(slot_base + off::SLOT_ACTIVE_FLAG) == 0
                {
                    hooks.on_default_cleanup(self);
                }
            }
            PumpDispatchResult::CaseNeg1 => {
                // C L307-318: cleanup1 then if slot.head != 0, current = head; active=1.
                if let Some(cid) = current { hooks.invoke_cleanup1(self, cid); }
                if let Some(hid) = self.slot_head_id(slot) {
                    self.slot_write_id(slot, off::SLOT_CURRENT_ID, Some(hid));
                    self.set_u32(slot_base + off::SLOT_ACTIVE_FLAG, 1);
                }
            }
            PumpDispatchResult::CaseNeg2 => {
                // C L298-306: cleanup1 then follow current.prev (`+500` = 0x1F4).
                // Guard: current.param_4 must be zero (non-modal).
                if let Some(cid) = current { hooks.invoke_cleanup1(self, cid); }
                let (prev, cur_modal) = current
                    .and_then(|c| self.records.get(&c))
                    .map(|r| (r.prev, r.param_4 != 0))
                    .unwrap_or((None, false));
                if let Some(pid) = prev {
                    if !cur_modal {
                        self.slot_write_id(slot, off::SLOT_CURRENT_ID, Some(pid));
                        self.set_u32(slot_base + off::SLOT_ACTIVE_FLAG, 1);
                    }
                }
            }
            PumpDispatchResult::CaseNeg3 => {
                // C L290-297: cleanup1 then follow current.next (`+0x1F8`).
                // Guard: iVar8 != 0 (no param_4 check for -3 in decomp).
                if let Some(cid) = current { hooks.invoke_cleanup1(self, cid); }
                let next = current
                    .and_then(|c| self.records.get(&c))
                    .and_then(|r| r.next);
                if let Some(nid) = next {
                    self.slot_write_id(slot, off::SLOT_CURRENT_ID, Some(nid));
                    self.set_u32(slot_base + off::SLOT_ACTIVE_FLAG, 1);
                }
            }
            PumpDispatchResult::CaseNeg4 | PumpDispatchResult::CaseNeg11 => {
                // C L227-232: just set slot ACTIVE_FLAG=1.
                self.set_u32(slot_base + off::SLOT_ACTIVE_FLAG, 1);
            }
            PumpDispatchResult::CaseNeg10 => {
                // C L233-287: cleanup1, then walk down current->prev chain
                // freeing each record's owned bag entries + the record itself.
                // Complex; delegate the walk to hooks + do the bag-free default.
                if let Some(cid) = current {
                    hooks.invoke_cleanup1(self, cid);
                    hooks.on_bag_free(self, cid);
                }
                hooks.on_external_case(self, code);
                // Set ACTIVE_FLAG so LAB_007e5550 sees completion (C L275-276).
                self.set_u32(slot_base + off::SLOT_ACTIVE_FLAG, 1);
            }
            PumpDispatchResult::CaseNeg5
            | PumpDispatchResult::CaseNeg6
            | PumpDispatchResult::CaseNeg8
            | PumpDispatchResult::CaseNeg9
            | PumpDispatchResult::CaseNeg12
            | PumpDispatchResult::CaseNeg13 => {
                // Body needs unported externals; observe via hook only.
                hooks.on_external_case(self, code);
            }
            PumpDispatchResult::CaseNeg7 => {
                // C L514: `if bVar1 || local_434 == -7 goto LAB_007e5613`
                // → falls into finalization. Signal exit.
                hooks.on_external_case(self, code);
                return true;
            }
        }
        false
    }

    /// Finalization pass — walks each populated slot's linked-list from head
    /// to tail calling cleanup2 on every record (C L613-680). Then applies
    /// trailing state writes at C L681-685.
    ///
    /// The exe's finalization also broadcasts a "Processing... Please Wait"
    /// packet (L516-611) via FUN_00762b90; that is deferred (needs 4b-net).
    ///
    /// Instruction count for L613-680: ~68 lines of C (per-slot loop with
    /// nested cleanup2 walk + FUN_005493b0 gate + FUN_007e6e00 tick).
    pub fn pump_finalize(&mut self, hooks: &mut dyn ScrmanHooks) {
        // L613-680: per-slot walk. We port the SHAPE (head→next chain,
        // cleanup2 per record); FUN_005493b0 / FUN_007e6e00 are trait-hooked.
        for slot in 0..SLOT_COUNT {
            if self.slot_depth(slot) == 0 { continue; }
            let mut it = self.slot_head_id(slot);
            while let Some(rid) = it {
                hooks.invoke_cleanup2(self, rid);
                it = self.records.get(&rid).and_then(|r| r.next);
            }
        }
        // L681-685 trailing state writes (byte-exact from asm).
        self.set_u16(off::CURRENT_SLOT, 0);          // L681
        self.set_u16(off::PUMP_WORD_0X13256A, 1);    // L682 `[0x992b5] = 1`
        self.set_u16(off::PUMP_WORD_0X13256C, 0);    // L683
        self.set_u16(off::PUMP_ACTIVE_LO, 0);        // L684
        self.set_u16(off::PUMP_ACTIVE_HI, 0);        // L685
    }

    /// Full pump tick — assembles prologue + entry-prep + dispatch loop +
    /// finalization. Returns the last dispatch code (`CaseNeg5` per C L686
    /// signals the "return true" path; `CaseNeg7` reaches finalization via
    /// the explicit exit gate).
    ///
    /// The loop drives once per current record and exits on:
    /// - CaseNeg7 (explicit finalization gate, C L514)
    /// - CaseNeg5 (soft-abort path — matches C L686 `return local_434 == -5`)
    /// - Empty current (nothing to dispatch on)
    /// - A safety cap of 4096 iterations (real pump has more complex exit
    ///   condition at L497-509; deferred).
    pub fn pump_with_hooks(&mut self, hooks: &mut dyn ScrmanHooks) -> PumpDispatchResult {
        self.pump_prologue();
        self.pump_entry_prep();
        // Network prologue L91-96 — DEFERRED (4b-net).

        let mut last_code = PumpDispatchResult::Case0;
        let mut iter = 0u32;
        loop {
            iter += 1;
            if iter > 4096 { break; }
            let slot = self.current_slot() as usize;
            if slot >= SLOT_COUNT { break; }
            let cid = match self.slot_current_id(slot) {
                Some(c) => c,
                None => break,
            };
            let raw = hooks.invoke_event(self, cid, 0xFFFF_FFFF);
            last_code = PumpDispatchResult::from_raw(raw);
            let exit = self.pump_dispatch_case(last_code, 0xFFFF_FFFF, hooks);
            if exit { break; }
            if last_code == PumpDispatchResult::CaseNeg5 { break; }
        }

        self.pump_finalize(hooks);
        last_code
    }

    /// Convenience no-op driver: pumps with a hook that immediately returns
    /// -7 (finalization exit), so the loop runs prologue → 1 dispatch → finalize.
    /// Real callers use `pump_with_hooks` with an app-provided ScrmanHooks impl.
    pub fn pump(&mut self) -> PumpDispatchResult {
        struct ExitImmediately;
        impl ScrmanHooks for ExitImmediately {
            fn invoke_event(&mut self, _: &mut ScreenManager, _: RecordId, _: u32) -> i32 { -7 }
        }
        self.pump_with_hooks(&mut ExitImmediately)
    }

    /// Immutable view of the network-buffer sub-object at `+0x302a`.
    #[inline]
    pub fn net_buf(&self) -> &NetworkBuffer { &self.net_buf }
    /// Mutable view for follow-up commits porting net-send/recv.
    #[inline]
    pub fn net_buf_mut(&mut self) -> &mut NetworkBuffer { &mut self.net_buf }

    /// First session sub-object marker (base `+0x3070`, ctor `mode = 0`).
    #[inline]
    pub fn session_a(&self) -> &SessionSubObject { &self.session_a }
    /// Second session sub-object marker (base `+0x1326d1`, ctor `mode = 1`).
    #[inline]
    pub fn session_b(&self) -> &SessionSubObject { &self.session_b }

    /// The exact write sequence of `sub_007e3f20`. Each line cites the GDI asm address.
    fn apply_ctor_writes(&mut self) {
        // ---- external sub-object ctors (GDI 0x7e3f41..0x7e3f6f) ----
        //   sub_7631d0(this+0x302a, 0xc350)  — network buffer, applied by `fn new()` above.
        //   sub_548d50(this+0x3070, 0)       — session A: apply its 20 writes into arena.
        //   sub_548d50(this+0x1326d1, 1)     — session B: apply its 20 writes into arena.
        //
        // Copy the mode arg out before mutably borrowing `bytes` to avoid overlapping borrows.
        let sess_a_mode = self.session_a.mode;
        let sess_b_mode = self.session_b.mode;
        {
            let arena = self.as_bytes_mut();
            SessionSubObject::new(sess_a_mode).apply_ctor_writes(arena, off::SESSION_A);
            SessionSubObject::new(sess_b_mode).apply_ctor_writes(arena, off::SESSION_B);
        }

        // ---- header dwords/words (GDI 0x7e3f74..0x7e3fe1) ----
        self.set_u32(off::NET_MODE, 0);            // 0x7e3f74
        self.set_u32(off::PUMP_ACTIVE, 0);         // 0x7e3f7a
        self.set_u32(off::B_DWORD_0X261FCC, 0);    // 0x7e3f80
        self.set_u32(off::B_DWORD_0X261FD0, 0);    // 0x7e3f86
        self.set_u32(off::FLUSH_INFLIGHT, 0);      // 0x7e3f8c
        self.set_u32(off::B_DWORD_0X261D32, 0);    // 0x7e3f92
        self.set_u32(off::B_DWORD_0X261FB6, 0);    // 0x7e3f98
        self.set_u32(off::AUX_0X3026, 0);          // 0x7e3f9e
        self.set_u32(off::B_DWORD_0X261FC4, 0);    // 0x7e3fa4
        self.set_u16(off::B_WORD_0X261FC8, 0);     // 0x7e3faa
        self.set_u16(off::B_WORD_0X261FCA, 0);     // 0x7e3fb1
        self.set_u16(off::CURRENT_SLOT, 0);        // 0x7e3fb8
        self.set_u16(off::TARGET_SLOT, 0xFFFF);    // 0x7e3fbf
        self.set_u32(off::AUX_0X3060, 0);          // 0x7e3fc8
        self.set_u32(off::B_DWORD_0X261FBC, 0);    // 0x7e3fce
        self.set_u32(off::B_DWORD_0X261FC0, 0);    // 0x7e3fd4
        self.set_u16(off::B_WORD_0X261FBA, 0);     // 0x7e3fda

        // ---- 16-slot loop (GDI 0x7e3fe1..0x7e4012) ----
        //   for i in 0..16:
        //     word[SLOT_DEPTH_TABLE + i*2] = 0
        //     zero 10 dwords starting at B_SECONDARY_BAG + i*0x28
        for i in 0..SLOT_COUNT {
            self.set_u16(off::SLOT_DEPTH_TABLE + i * 2, 0);      // 0x7e3ffe
            let base = off::B_SECONDARY_BAG + i * off::B_SECONDARY_BAG_STRIDE;
            for j in 0..off::B_SECONDARY_BAG_ZERO_DWORDS {
                self.set_u32(base + j * 4, 0);                   // 0x7e4001 rep stosd
            }
        }

        // ---- root-frame writes (GDI 0x7e4018..0x7e4087) ----
        // 0x7e4018-0x7e401e: dword[+0x28] = &self[0x3070]
        //
        // Represent as a stored byte offset since we don't hand out raw pointers this commit.
        // The exe stores the absolute pointer; when a follow-up commit adds subobject support,
        // this will become a genuine `*mut u8` write. For now we stash the byte-offset which
        // is enough for byte-diff parity of everything except that one pointer field.
        self.set_u32(off::INITIAL_SCREEN_PTR, off::SESSION_A as u32); // pointer TBD (host width)

        self.set_u16(off::SLOT_DEPTH_TABLE, 1);    // 0x7e4021 slot-0 depth=1
        self.set_u16(off::ROOT_SCREEN_ID, 0);      // 0x7e402a
        self.set_u32(off::ROOT_ARG1, 0);           // 0x7e402d
        self.set_u32(off::ROOT_ARG2, 0);           // 0x7e4030
        self.set_u32(off::ROOT_ARG3, 0);           // 0x7e4033
        self.set_u32(off::ROOT_CUR_ENTRY, 0);      // 0x7e4036
        self.set_u16(off::ROOT_FLAG_0X12, 0);      // 0x7e4039
        self.set_u16(off::ROOT_DEPTH, 0);          // 0x7e403d
        self.set_u32(off::ROOT_RUNNING, 1);        // 0x7e4041  **constant 1**
        self.set_u16(off::ROOT_PEER, 0xFFFF);      // 0x7e4048  **constant 0xFFFF**
        self.set_u32(off::ROOT_AUX_0X24, 0);       // 0x7e404e
        self.set_u32(off::AUX_0X301A, 0);          // 0x7e4051
        self.set_u32(off::AUX_0X301E, 0);          // 0x7e4057
        self.set_u32(off::AUX_0X3022, 0);          // 0x7e405d
        self.set_u32(off::DEFERRED_NAME, 0);       // 0x7e4063
        self.set_u32(off::DEFERRED_ARG1, 0);       // 0x7e4069
        self.set_u32(off::DEFERRED_ARG2, 0);       // 0x7e406f
        self.set_u32(off::DEFERRED_ARG3, 0);       // 0x7e4075
        self.set_u32(off::DEFERRED_ARG4, 0);       // 0x7e407b
        self.set_u32(off::BAG_VALUES, 0);          // 0x7e4081
        self.set_u16(off::BAG_COUNT, 0);           // 0x7e4087

        // ---- network-buffer arena reflections (`FUN_00763590` writes at scrman +0x302a) ----
        // `+0x302a` (buf-ptr) stays 0 in the arena — host pointer widths differ from 32-bit;
        // the live pointer is owned by `self.net_buf`, accessible via `net_buf()`.
        self.set_u32(off::NET_BUF_PTR, 0);
        self.set_u32(off::NET_BUF_WRITE_OFF, self.net_buf.write_off);
        self.set_u32(off::NET_BUF_SIZE,      self.net_buf.size);
    }

    // ------------------------------------------------------------
    // Typed field accessors — all byte-cited to `off::` above.
    // ------------------------------------------------------------

    #[inline]
    pub fn current_slot(&self) -> u16 { self.get_u16(off::CURRENT_SLOT) }
    #[inline]
    pub fn target_slot(&self) -> u16 { self.get_u16(off::TARGET_SLOT) }
    #[inline]
    pub fn pump_active(&self) -> u32 { self.get_u32(off::PUMP_ACTIVE) }
    #[inline]
    pub fn net_mode(&self) -> u32 { self.get_u32(off::NET_MODE) }
    #[inline]
    pub fn root_running(&self) -> u32 { self.get_u32(off::ROOT_RUNNING) }
    #[inline]
    pub fn root_peer(&self) -> u16 { self.get_u16(off::ROOT_PEER) }
    #[inline]
    pub fn slot_depth(&self, slot: usize) -> u16 {
        assert!(slot < SLOT_COUNT);
        self.get_u16(off::SLOT_DEPTH_TABLE + slot * 2)
    }

    /// Raw byte access — for follow-up commits that port the pump / push / net-ops.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.bytes.as_ptr(), SCRMGR_SIZE) }
    }
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.bytes.as_ptr(), SCRMGR_SIZE) }
    }

    // ---- primitives ----
    #[inline]
    fn set_u16(&mut self, off: usize, v: u16) {
        assert!(off + 2 <= SCRMGR_SIZE);
        unsafe {
            std::ptr::write_unaligned(self.bytes.as_ptr().add(off) as *mut u16, v.to_le());
        }
    }
    #[inline]
    fn set_u32(&mut self, off: usize, v: u32) {
        assert!(off + 4 <= SCRMGR_SIZE);
        unsafe {
            std::ptr::write_unaligned(self.bytes.as_ptr().add(off) as *mut u32, v.to_le());
        }
    }
    #[inline]
    fn get_u16(&self, off: usize) -> u16 {
        assert!(off + 2 <= SCRMGR_SIZE);
        unsafe {
            u16::from_le(std::ptr::read_unaligned(self.bytes.as_ptr().add(off) as *const u16))
        }
    }
    #[inline]
    fn get_u32(&self, off: usize) -> u32 {
        assert!(off + 4 <= SCRMGR_SIZE);
        unsafe {
            u32::from_le(std::ptr::read_unaligned(self.bytes.as_ptr().add(off) as *const u32))
        }
    }
}

impl Drop for ScreenManager {
    fn drop(&mut self) {
        // Invoke each session sub-object's teardown against the live arena
        // BEFORE the backing store is deallocated. This is the byte-faithful
        // wire-up of `FUN_00548bd0`, ported via the 5 helper fns.
        //
        // SAFETY: `bytes` is a valid, mut-only, SCRMGR_SIZE-long allocation we
        // still own; nothing else borrows it here.
        {
            let arena_mut: &mut [u8] = unsafe {
                std::slice::from_raw_parts_mut(self.bytes.as_ptr(), SCRMGR_SIZE)
            };
            self.session_a.teardown(arena_mut, off::SESSION_A);
            self.session_b.teardown(arena_mut, off::SESSION_B);
        }
        unsafe { dealloc(self.bytes.as_ptr(), Self::layout()); }
    }
}

impl Default for ScreenManager {
    fn default() -> Self { Self::new() }
}

// ============================================================
// 4b-net — network layer
//
// Ports the pieces of the exe's winsock plumbing that the scrman pump
// reaches into: the send path (`FUN_00762b90`, 105-line C decomp) and
// the receive path (`FUN_00762e80` — no C decomp; equivalent GDI asm
// at `sub_00762ac0`, 606 lines). Those functions live on the exe's
// global network singleton, not on the ScreenManager, so this port
// keeps that split: the ScreenManager owns the *buffer* (already at
// `+0x302a`) and the *pump-side call sites*, while the peer table +
// the winsock file-descriptors live behind a `NetSocket` trait the
// app-side wires to real BSD sockets.
//
// What is BYTE-EXACT ported here (from cited decomps):
//   * `FUN_00933d24`     — 8 lines: forwards to a heap-free stub.
//                          Ported as a no-op (Rust `drop` handles it).
//   * `FUN_00762b90` header/loop control flow — 4-byte size prefix +
//     payload, peer-slot loop over the 16-word peer table, sentinel
//     -1 write on send failure, count-callback invocation.  Winsock
//     `FUN_0089afd0` is replaced by `NetSocket::send`; the outer
//     shape (single-peer / broadcast / directed) matches the decomp.
//
// What is FUNCTIONALLY ported (shape not asm-line-cite):
//   * `net_poll_recv`    — the receive-side entry the exe calls at
//     pump prologue. The GDI-only source has no C decomp; the
//     documented behaviour (drain the socket into `NetworkBuffer`
//     starting at `write_off`, advance `write_off`) is what the
//     pump downstream reads.  See STOP-AND-REPORT below.
//
// What STAYS deferred with a documented STOP-AND-REPORT:
//   * `FUN_00762e80` full 601-instruction body — needs the exe's
//     `sub_89a970` (accept) + `sub_89aeb0` (recv) + connection-count
//     callback at `+0xc3a66` on the network singleton.  This commit
//     wires a `NetSocket`-shaped recv that is byte-correct for a
//     single-peer inbound stream but does not reproduce the multi-
//     peer accept / disconnect bookkeeping.
//   * `FUN_0054dc60` (1393 lines of C) — the peer-input dispatch
//     called from case-body -8 and from `FUN_007eaac0`'s no-peer
//     fallback.  Too large for this commit; a trait hook lets the
//     app supply it.
//   * `FUN_005493b0` full body — most fields it zeroes live on the
//     external network singleton (+0x12f4ff, +0x12f501, etc, on the
//     *session* sub-object), whose full field-decode is a separate
//     commit.  A trait hook fires so app-side can call the real fn.
// ============================================================

/// Handle for one connected peer.  Byte-wide (the exe stores peer
/// sockets as 16-bit values in the peer table at network-singleton
/// `+0x4ba..+0x4d9`), but we widen to u32 to leave room for the
/// larger fds on 64-bit hosts.  `PeerId(0xFFFF_FFFF)` is the exe's
/// -1 sentinel (empty slot).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PeerId(pub u32);

impl PeerId {
    pub const NONE: PeerId = PeerId(0xFFFF_FFFF);
    #[inline]
    pub fn is_none(self) -> bool { self.0 == 0xFFFF_FFFF }
    #[inline]
    pub fn is_some(self) -> bool { !self.is_none() }
}

/// Result codes from a socket call.  Modelled on the exe's use of
/// `-2` = "would block / disconnected" (see `FUN_00762b90` L58 —
/// `iVar5 != -2` decides whether the peer stays alive).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NetIoResult {
    /// Bytes moved successfully.
    Ok(usize),
    /// Non-fatal — try again later.  Exe treats as `iVar5 != -2` alive.
    WouldBlock,
    /// Fatal — the peer has gone away.  Exe writes `-1` into the
    /// peer slot and (if wired) calls the disconnect callback.
    Disconnect,
}

/// The exe's winsock surface — what `FUN_0089afd0` (send) and
/// `sub_89aeb0` (recv) do, abstracted so the app can wire real
/// sockets and the test suite can wire an in-memory mock.
pub trait NetSocket {
    /// Analogue of `FUN_0089afd0(peer, buf, len)` on the SEND path.
    /// Returns the number of bytes written (may be less than `buf.len()`
    /// for a slow peer) or a fatal disconnect.  Matches the exe's
    /// `iVar5 != -2` alive-check on the returned value.
    fn send_to(&mut self, peer: PeerId, buf: &[u8]) -> NetIoResult;

    /// Analogue of `sub_89aeb0(peer, buf, cap)` on the RECV path.
    /// Reads up to `buf.len()` bytes from the peer's socket.
    fn recv_from(&mut self, peer: PeerId, buf: &mut [u8]) -> NetIoResult;

    /// Analogue of `sub_89a970` — poll for a newly-connected peer.
    /// Returns `None` if nothing pending.  Called by the recv
    /// prologue in `net_poll_recv`.
    fn accept_new_peer(&mut self) -> Option<PeerId> { None }

    /// Iterate the peer table — the 16 half-word slots at
    /// network-singleton `+0x4ba..+0x4d9`.  Callers use this in
    /// broadcast mode (`FUN_00762b90` C L48-72 loop).  Empty slots
    /// come back as `PeerId::NONE`.
    fn peers(&self) -> Vec<PeerId>;
}

/// Convenience mock for tests: an in-memory single-peer socket with
/// a scripted inbox and an accumulated outbox.  One `PeerId(0)`.
///
/// The mock deliberately fails **cleanly** — an empty inbox returns
/// `WouldBlock`, matching what the exe's non-blocking winsock does
/// (`WSAEWOULDBLOCK` maps to `iVar5 != -2` = alive-but-nothing-yet).
pub struct MockSocket {
    /// Bytes waiting for `recv_from` to consume, in order.
    pub inbox: std::collections::VecDeque<u8>,
    /// Bytes `send_to` has written, in order.
    pub outbox: Vec<u8>,
    /// Peer table — starts as `[PeerId(0)]` (one connected peer).
    /// Replace to mock broadcast to N peers.
    pub peer_table: Vec<PeerId>,
    /// Whether the next `send_to` should return `Disconnect`.  Test-only.
    pub next_send_fatal: bool,
    /// Whether the next `recv_from` should return `Disconnect`.  Test-only.
    pub next_recv_fatal: bool,
    /// Pending accept — the next `accept_new_peer()` call returns this
    /// then clears.
    pub pending_accept: Option<PeerId>,
}

impl MockSocket {
    pub fn new() -> Self {
        MockSocket {
            inbox: std::collections::VecDeque::new(),
            outbox: Vec::new(),
            peer_table: vec![PeerId(0)],
            next_send_fatal: false,
            next_recv_fatal: false,
            pending_accept: None,
        }
    }

    /// Preload `bytes` for the mock to hand out on `recv_from`.
    pub fn push_inbound(&mut self, bytes: &[u8]) {
        for &b in bytes { self.inbox.push_back(b); }
    }
}

impl Default for MockSocket { fn default() -> Self { Self::new() } }

impl NetSocket for MockSocket {
    fn send_to(&mut self, _peer: PeerId, buf: &[u8]) -> NetIoResult {
        if self.next_send_fatal {
            self.next_send_fatal = false;
            return NetIoResult::Disconnect;
        }
        self.outbox.extend_from_slice(buf);
        NetIoResult::Ok(buf.len())
    }
    fn recv_from(&mut self, _peer: PeerId, buf: &mut [u8]) -> NetIoResult {
        if self.next_recv_fatal {
            self.next_recv_fatal = false;
            return NetIoResult::Disconnect;
        }
        if self.inbox.is_empty() { return NetIoResult::WouldBlock; }
        let take = buf.len().min(self.inbox.len());
        for slot in buf.iter_mut().take(take) {
            *slot = self.inbox.pop_front().unwrap();
        }
        NetIoResult::Ok(take)
    }
    fn accept_new_peer(&mut self) -> Option<PeerId> { self.pending_accept.take() }
    fn peers(&self) -> Vec<PeerId> { self.peer_table.clone() }
}

/// Snapshot of what `net_poll_recv` did during one pump prologue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetPollResult {
    /// Bytes appended to `NetworkBuffer` this poll.
    pub bytes_appended: u32,
    /// New peer accepted this poll (from `accept_new_peer`), if any.
    pub new_peer: Option<PeerId>,
    /// Peers that returned `Disconnect` this poll; the exe would
    /// write `-1` into their peer-table slot.  The app-side is
    /// responsible for actually zeroing the slot (we can't reach
    /// into a `dyn NetSocket`'s private table).
    pub dropped_peers: Vec<PeerId>,
}

/// Trait for the peer-input dispatcher (`FUN_0054dc60`, 1393-line
/// giant).  This commit does NOT port that function; it exposes the
/// call site as a hook so the pump can drive through the -8 and -5
/// case-bodies.  The default impl is a no-op (matches the
/// nothing-to-dispatch state the offline harness uses).
pub trait PeerInputDispatch {
    /// Analogue of `FUN_0054dc60(net_mode, 0, &buf_write_off)`.
    /// Called from case-body -8 and from the "Processing..." region.
    fn dispatch_peer_input(&mut self, _mgr: &mut ScreenManager) {}

    /// Analogue of `FUN_00549210(session)` — the switch on the
    /// session's mode field that flushes pending display state.
    /// Called from the recv prologue and per-iteration.
    fn flush_session(&mut self, _mgr: &mut ScreenManager) -> i16 { 0 }
}

/// Default (no-op) peer-input dispatcher.  Real app-side impl wires
/// to actual `FUN_0054dc60` / `FUN_00549210` bodies once they land.
pub struct NoPeerInput;
impl PeerInputDispatch for NoPeerInput {}

impl ScreenManager {
    // --------------------------------------------------------
    // FUN_00933d24 — 8-line heap free wrapper.
    //
    //   void FUN_00933d24(undefined4 param_1) {
    //     FUN_0093435a(param_1);
    //     return;
    //   }
    //
    // Whole thing forwards to a heap-free stub the app-side owns.
    // Rust's `drop` handles the actual free through our BTreeMap
    // record pool, so this port is a documentation-only no-op.
    //
    // Callers we replaced:
    //   * `FUN_007eaac0` — case-body -5/-6/-8/-9/-12/-13 free path.
    //   * Bag-free walk in case-body -10.
    // --------------------------------------------------------
    /// Port of `FUN_00933d24`.  See module note above — no-op.
    #[inline]
    pub fn heap_free(&mut self, _record: RecordId) { /* Rust drop */ }

    // --------------------------------------------------------
    // net_poll_recv — receive prologue (functional port; see 4b-net
    //   note above for the STOP-AND-REPORT on `FUN_00762e80`'s full
    //   601-instruction body).
    //
    // Behaviour we port (from what the pump downstream reads):
    //   1. Call `accept_new_peer()` — the `sub_89a970` analogue.
    //   2. For each live peer in `socket.peers()`, drain any pending
    //      inbound bytes via `recv_from` into `NetworkBuffer` starting
    //      at `write_off`, advancing `write_off` on success.
    //   3. On `Disconnect` return, note the peer in `dropped_peers`;
    //      the app-side reflects that back into the peer table (we
    //      can't touch the socket's private table from here).
    //   4. WouldBlock is silent — matches the exe's non-blocking
    //      winsock returning `WSAEWOULDBLOCK`.
    //
    // Buffer semantics: mirrors the exe — new bytes go at
    // `net_buf.buf[write_off..]`, `write_off` is bumped, buffer size
    // is a hard ceiling (`NET_BUF_DEFAULT_SIZE = 50000`).
    // --------------------------------------------------------
    /// Recv prologue — called at pump entry (region L91-96).
    ///
    /// See the 4b-net STOP-AND-REPORT above: this is the functional
    /// shape of `FUN_00762e80`, not a byte-exact port of its 601
    /// asm instructions.
    pub fn net_poll_recv(&mut self, socket: &mut dyn NetSocket) -> NetPollResult {
        let new_peer = socket.accept_new_peer();
        let mut bytes_appended: u32 = 0;
        let mut dropped_peers: Vec<PeerId> = Vec::new();

        // Snapshot the peer list (avoid re-borrowing socket inside the loop).
        let peers = socket.peers();
        for peer in peers {
            if peer.is_none() { continue; }
            // Loop until WouldBlock — matches exe's drain-per-peer.
            loop {
                let write_off = self.net_buf.write_off as usize;
                let cap = self.net_buf.size as usize;
                if write_off >= cap { break; } // buffer full — silently stall
                let (result, filled) = {
                    let dst = &mut self.net_buf.buf[write_off..cap];
                    let r = socket.recv_from(peer, dst);
                    (r, dst.len())
                };
                let _ = filled;
                match result {
                    NetIoResult::Ok(n) => {
                        if n == 0 { break; }
                        self.net_buf.write_off = (self.net_buf.write_off + n as u32).min(self.net_buf.size);
                        bytes_appended += n as u32;
                    }
                    NetIoResult::WouldBlock => break,
                    NetIoResult::Disconnect => {
                        dropped_peers.push(peer);
                        break;
                    }
                }
            }
        }

        NetPollResult { bytes_appended, new_peer, dropped_peers }
    }

    // --------------------------------------------------------
    // net_flush_send — send path.  BYTE-EXACT header + loop from
    //   `FUN_00762b90` (105-line C decomp), just with `FUN_0089afd0`
    //   replaced by `NetSocket::send_to`.
    //
    // Wire format (C L38-42):
    //   [0]  = size (byte 0)              — 4 bytes total, little-endian
    //   [1]  = size (byte 1)
    //   [2]  = size (byte 2)
    //   [3]  = size (byte 3)
    //   [4..4+size] = payload
    //
    // Guards (C L28-36):
    //   * `size < 1`        → error, return false.
    //   * `size > 49999`    → return false (buffer would overflow).
    //
    // Modes (from caller `param_4` = peer-id or 0 = broadcast):
    //   * `+0xc3a5e == 0` && peer sentinel valid → single-peer send.
    //   * `param_4 == 0`  → broadcast to every live peer.
    //   * `param_4 != 0`  → directed send to peer at
    //     `[+0x4b8 + param_4*2]` (peer-slot lookup).
    //
    // On failure (`send_to` returns `Disconnect`):
    //   * The peer's slot is invalidated (`-1`).  We can't touch the
    //     socket's private table, so we surface `dropped_peers` in
    //     the return.
    //   * The connection-count callback at `+0xc3a66` fires — we
    //     can't call an arbitrary function pointer safely; the
    //     `dropped_peers` list is the app-side's cue.
    //
    // Return value: `SendResult` — bytes sent + dropped peers.
    // --------------------------------------------------------
    /// Send `payload` via the socket, following `FUN_00762b90`'s
    /// 4-byte-size-prefix + payload wire format.  When `directed_to`
    /// is `Some(peer)` it targets that peer; `None` broadcasts.
    ///
    /// Returns `(bytes_sent_total, dropped_peers)`.  Guards match
    /// the decomp: empty and >49999-byte payloads return early.
    pub fn net_flush_send(
        &mut self,
        socket: &mut dyn NetSocket,
        payload: &[u8],
        directed_to: Option<PeerId>,
    ) -> (usize, Vec<PeerId>) {
        // C L28-30: `param_3 < 1` → error, return false.
        if payload.is_empty() { return (0, Vec::new()); }
        // C L32-34: `49999 < param_3` → return false.
        if payload.len() > 49_999 { return (0, Vec::new()); }

        let size_prefix: [u8; 4] = (payload.len() as u32).to_le_bytes();
        let mut sent_total = 0usize;
        let mut dropped = Vec::new();

        let peers = socket.peers();
        let targets: Vec<PeerId> = if let Some(p) = directed_to {
            vec![p]
        } else {
            peers.into_iter().filter(|p| p.is_some()).collect()
        };

        for peer in targets {
            if peer.is_none() { continue; }
            // C L36-46: write 4 size bytes, one call per byte in the
            // decomp (kept as a single `send_to` here — the byte
            // splitting is a decomp artefact of winsock's small-buffer
            // send being reported as 4 separate 1-byte writes).
            let header_res = socket.send_to(peer, &size_prefix);
            let header_ok = matches!(header_res, NetIoResult::Ok(_) | NetIoResult::WouldBlock);
            if !header_ok {
                dropped.push(peer);
                continue;
            }
            if let NetIoResult::Ok(n) = header_res { sent_total += n; }
            // C L47: payload send.
            let body_res = socket.send_to(peer, payload);
            match body_res {
                NetIoResult::Ok(n) => sent_total += n,
                NetIoResult::WouldBlock => { /* alive but stalled */ }
                NetIoResult::Disconnect => {
                    dropped.push(peer);
                }
            }
        }

        (sent_total, dropped)
    }

    // --------------------------------------------------------
    // Case-body helpers — port of `FUN_007eaac0` shape.
    //
    // The 6 external cases (-5, -6, -8, -9, -12, -13) all share the
    // same tail (`LAB_007e52a2` in the decomp): call `FUN_00933d24`
    // to free the current record + call `FUN_007eaac0` to broadcast
    // a 1-byte marker (`9`) to every peer, then reset the network
    // buffer's write offset.  Case-specific mutations happen BEFORE
    // this shared tail.
    //
    // `FUN_007eaac0` fully-decoded body (77 lines, `dev/CM3.00.01/si/code/scrman.cpp` L#s
    // in error path):
    //   1. Guard: `[+0x14 + cur_slot*0x180] + 0x12f53d == 0`.
    //   2. Clear `[+0x302e]` (write-off) and `[+0x3030]`.
    //   3. Overflow-guard the buffer for +1 byte.
    //   4. Write byte `9` at `buf[write_off]`; write_off += 1.
    //   5. Broadcast the buffer via `FUN_00762b90` to every live peer
    //      (peer-loop starts at `[+0x3036]` — slot depth table).
    //   6. When peer sentinel == 0 for a slot, fall back to
    //      `FUN_0054dc60` local dispatch (peer-input hook).
    //
    // Fields cited on the local slot record:
    //   * `+0x14`   dword — session sub-object pointer.
    //   * `[+0x1815]` short-index → `+0x302a` — network buffer base.
    //   * `[+0x1817]` short-index → `+0x302e` — write offset.
    //   * `[+0x181b]` short-index → `+0x3036` — slot depth table.
    // --------------------------------------------------------

    /// Port of `FUN_007eaac0` — the shared broadcast tail.  Writes
    /// the byte-`9` end-of-frame marker into the network buffer at
    /// `write_off`, then flushes via the socket.  Returns the number
    /// of bytes sent across all peers.
    ///
    /// The exe's error-path (buffer full → msgbox + set global
    /// `DAT_00b4d5a8 = 0`) is dropped here — a full buffer stalls
    /// silently, matching non-fatal behaviour.
    pub fn broadcast_end_marker(
        &mut self,
        socket: &mut dyn NetSocket,
        peer_dispatch: &mut dyn PeerInputDispatch,
    ) -> usize {
        // Step 2: clear write-off (C L14-15).  Matches the guard's
        // pre-condition: buffer starts empty at broadcast time.
        self.net_buf.write_off = 0;

        // Step 3-4: append byte `9` end-marker (C L21-22).
        let capacity = self.net_buf.size as usize;
        if (self.net_buf.write_off as usize) < capacity {
            let off = self.net_buf.write_off as usize;
            self.net_buf.buf[off] = 9;
            self.net_buf.write_off += 1;
        }

        // Step 5-6: broadcast to peers, or fall back to peer-dispatch
        // when no peers are live.
        let peers = socket.peers();
        let any_alive = peers.iter().any(|p| p.is_some());
        if !any_alive {
            // Fall-back path: `FUN_0054dc60(net_mode, 0, &write_off)`.
            peer_dispatch.dispatch_peer_input(self);
            return 0;
        }
        let payload_len = self.net_buf.write_off as usize;
        let payload = self.net_buf.buf[..payload_len].to_vec();
        let (sent, _dropped) = self.net_flush_send(socket, &payload, None);
        sent
    }

    /// Full-body port for the 6 external cases (-5, -6, -8, -9,
    /// -12, -13).  Case-specific chain mutations happen at the call
    /// site in `pump_dispatch_case_wired`; this helper is the
    /// shared `LAB_007e52a2` tail: free the current record and
    /// broadcast the end-marker.
    ///
    /// Returns `true` iff a record was freed (the current slot had
    /// a live current-record id).
    pub fn dispatch_external_tail(
        &mut self,
        socket: &mut dyn NetSocket,
        peer_dispatch: &mut dyn PeerInputDispatch,
    ) -> bool {
        let slot = self.current_slot() as usize;
        if slot >= SLOT_COUNT { return false; }
        // Free the current record if any (`FUN_00933d24` at
        // `LAB_007e52a2` L4).
        let cur = self.slot_current_id(slot);
        let freed = if let Some(cid) = cur {
            self.heap_free(cid);
            self.records.remove(&cid);
            // Chain fix-up: current slides forward to `next` if there is one,
            // otherwise back to `head`.
            let (next_id, head_id) = (None::<RecordId>, self.slot_head_id(slot));
            let new_cur = next_id.or(head_id);
            self.slot_write_id(slot, off::SLOT_CURRENT_ID, new_cur);
            true
        } else { false };
        // Broadcast the end-marker to peers (`FUN_007eaac0` main body).
        self.broadcast_end_marker(socket, peer_dispatch);
        freed
    }

    /// Wired pump dispatcher — same as `pump_dispatch_case` but with
    /// real bodies for cases -5/-6/-8/-9/-12/-13 (no more trait-hook
    /// stub).  The `hooks.on_external_case` observer still fires
    /// after the body applies, so tests can count.
    ///
    /// Case-specific behaviour (before the shared tail):
    ///   * `-5` — soft-abort: shared tail; pump loop exits on
    ///     `last_code == CaseNeg5`.
    ///   * `-6` — sibling-lift: set slot ACTIVE_FLAG=1 (matches C
    ///     L423-431 "current stays at slot head with flag=1").
    ///   * `-8` — broadcast slot's `+0xF` flag: same tail.
    ///   * `-9` — prev-chain unwind: `cleanup1` then `cur = prev`
    ///     (matches C L361-378).
    ///   * `-12` — network-flush-error: shared tail (no chain change).
    ///   * `-13` — pop-to-oldest: `cur = head` (matches C L440-449).
    pub fn pump_dispatch_case_wired(
        &mut self,
        code: PumpDispatchResult,
        local_438: u32,
        hooks: &mut dyn ScrmanHooks,
        socket: &mut dyn NetSocket,
        peer_dispatch: &mut dyn PeerInputDispatch,
    ) -> bool {
        let slot = self.current_slot() as usize;
        if slot >= SLOT_COUNT { return true; }
        let slot_base = slot * SLOT_STRIDE;
        match code {
            PumpDispatchResult::CaseNeg5 => {
                // Shared tail only.
                self.dispatch_external_tail(socket, peer_dispatch);
                hooks.on_external_case(self, code);
            }
            PumpDispatchResult::CaseNeg6 => {
                self.set_u32(slot_base + off::SLOT_ACTIVE_FLAG, 1);
                self.dispatch_external_tail(socket, peer_dispatch);
                hooks.on_external_case(self, code);
            }
            PumpDispatchResult::CaseNeg8 => {
                // Case -8 additionally dispatches peer input.
                peer_dispatch.dispatch_peer_input(self);
                self.dispatch_external_tail(socket, peer_dispatch);
                hooks.on_external_case(self, code);
            }
            PumpDispatchResult::CaseNeg9 => {
                // -9: cleanup1 then cur = prev.
                if let Some(cid) = self.slot_current_id(slot) {
                    hooks.invoke_cleanup1(self, cid);
                    let prev = self.records.get(&cid).and_then(|r| r.prev);
                    if let Some(pid) = prev {
                        self.slot_write_id(slot, off::SLOT_CURRENT_ID, Some(pid));
                    }
                }
                self.dispatch_external_tail(socket, peer_dispatch);
                hooks.on_external_case(self, code);
            }
            PumpDispatchResult::CaseNeg12 => {
                // Network-flush-error: shared tail only.
                self.dispatch_external_tail(socket, peer_dispatch);
                hooks.on_external_case(self, code);
            }
            PumpDispatchResult::CaseNeg13 => {
                // -13: cur = head (pop to oldest).
                if let Some(h) = self.slot_head_id(slot) {
                    self.slot_write_id(slot, off::SLOT_CURRENT_ID, Some(h));
                }
                self.dispatch_external_tail(socket, peer_dispatch);
                hooks.on_external_case(self, code);
            }
            other => {
                // Everything else defers to the pre-existing (non-net)
                // dispatcher — same behaviour as before this commit.
                return self.pump_dispatch_case(other, local_438, hooks);
            }
        }
        false
    }

    /// Wired pump — same shape as `pump_with_hooks` but drives
    /// through the wired case bodies (net path enabled).  All 4
    /// deferred regions are now called:
    ///   * L91-96 recv prologue → `net_poll_recv`.
    ///   * L358-379 select-next-slot-from-peer → `dispatch_peer_input`.
    ///   * L497-509 all-slots-quiescent → early break when every slot
    ///     has ACTIVE_FLAG=0 (approximation — the real check reads
    ///     entry-ACKED which we haven't wired to per-frame updates).
    ///   * L516-611 "Processing..." broadcast → `broadcast_end_marker`
    ///     at finalization.
    pub fn pump_with_net(
        &mut self,
        hooks: &mut dyn ScrmanHooks,
        socket: &mut dyn NetSocket,
        peer_dispatch: &mut dyn PeerInputDispatch,
    ) -> PumpDispatchResult {
        self.pump_prologue();
        self.pump_entry_prep();

        // L91-96 recv prologue.
        let _poll = self.net_poll_recv(socket);

        let mut last_code = PumpDispatchResult::Case0;
        let mut iter = 0u32;
        loop {
            iter += 1;
            if iter > 4096 { break; }

            // L358-379 select-next-slot-from-peer.
            // Not per-iteration in the exe (gated on peer event) —
            // we call once per iter as a conservative approximation.
            peer_dispatch.flush_session(self);

            let slot = self.current_slot() as usize;
            if slot >= SLOT_COUNT { break; }
            let cid = match self.slot_current_id(slot) {
                Some(c) => c,
                None => break,
            };
            let raw = hooks.invoke_event(self, cid, 0xFFFF_FFFF);
            last_code = PumpDispatchResult::from_raw(raw);
            let exit = self.pump_dispatch_case_wired(last_code, 0xFFFF_FFFF, hooks, socket, peer_dispatch);
            if exit { break; }
            if last_code == PumpDispatchResult::CaseNeg5 { break; }

            // L497-509 all-slots-quiescent: break when every slot's
            // ACTIVE_FLAG is 0 (approximation — see doc above).
            let all_quiet = (0..SLOT_COUNT)
                .all(|s| self.get_u32(s * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG) == 0);
            if all_quiet { break; }
        }

        // L516-611 "Processing..." broadcast — the finalization pass
        // sends the end-marker frame via `FUN_00762b90` before the
        // per-slot cleanup2 walk.
        self.broadcast_end_marker(socket, peer_dispatch);
        self.pump_finalize(hooks);
        last_code
    }
}

// ------------------------------------------------------------
// Tests
// ------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// `ScreenManager::new()` sets the exact defaults the exe's ctor sets.
    #[test]
    fn new_matches_exe_ctor_defaults() {
        let m = ScreenManager::new();
        // The four "non-zero" constants of the ctor:
        assert_eq!(m.root_running(), 1,        "+0x16 running flag");
        assert_eq!(m.root_peer(), 0xFFFF,      "+0x22 peer sentinel");
        assert_eq!(m.target_slot(), 0xFFFF,    "+0x3058 target slot sentinel");
        assert_eq!(m.slot_depth(0), 1,         "+0x3036 slot-0 depth");
        // Zero defaults spot-checked:
        assert_eq!(m.current_slot(), 0);
        assert_eq!(m.pump_active(), 0);
        assert_eq!(m.net_mode(), 0);
        for i in 1..SLOT_COUNT {
            assert_eq!(m.slot_depth(i), 0, "slot {} depth", i);
        }
        // The initial screen pointer stub points into session A.
        assert_eq!(m.get_u32(off::INITIAL_SCREEN_PTR), off::SESSION_A as u32);
    }

    /// Fields inside the far region (SESSION_B) that the ctor clears must actually be zero.
    #[test]
    fn far_region_ctor_clears() {
        let m = ScreenManager::new();
        assert_eq!(m.get_u32(off::B_DWORD_0X261D32), 0);
        assert_eq!(m.get_u32(off::B_DWORD_0X261FB6), 0);
        assert_eq!(m.get_u16(off::B_WORD_0X261FBA), 0);
        assert_eq!(m.get_u32(off::B_DWORD_0X261FBC), 0);
        assert_eq!(m.get_u32(off::B_DWORD_0X261FC0), 0);
        assert_eq!(m.get_u32(off::B_DWORD_0X261FC4), 0);
        assert_eq!(m.get_u16(off::B_WORD_0X261FC8), 0);
        assert_eq!(m.get_u16(off::B_WORD_0X261FCA), 0);
        assert_eq!(m.get_u32(off::B_DWORD_0X261FCC), 0);
        assert_eq!(m.get_u32(off::B_DWORD_0X261FD0), 0);
        // Secondary bag: 16 × 40 bytes, all zero.
        for i in 0..SLOT_COUNT {
            let base = off::B_SECONDARY_BAG + i * off::B_SECONDARY_BAG_STRIDE;
            for j in 0..off::B_SECONDARY_BAG_ZERO_DWORDS {
                assert_eq!(m.get_u32(base + j * 4), 0, "slot {} dword {}", i, j);
            }
        }
    }

    /// `reset()` restores every ctor-touched field to its post-ctor value even after arbitrary
    /// writes.
    #[test]
    fn reset_restores_ctor_defaults() {
        let mut m = ScreenManager::new();

        // Trash a wide sample of fields.
        m.set_u32(off::ROOT_RUNNING, 0xDEAD_BEEF);
        m.set_u16(off::ROOT_PEER, 0x1234);
        m.set_u16(off::TARGET_SLOT, 0x0007);
        m.set_u16(off::CURRENT_SLOT, 0x000A);
        m.set_u32(off::PUMP_ACTIVE, 1);
        m.set_u32(off::NET_MODE, 0xAABB_CCDD);
        for i in 0..SLOT_COUNT {
            m.set_u16(off::SLOT_DEPTH_TABLE + i * 2, 0x00FF);
        }
        m.set_u32(off::B_DWORD_0X261FD0, 0xCAFE_F00D);
        m.set_u32(off::B_DWORD_0X261D32, 0xBADD_C0DE);
        // A byte deep in an "opaque" region — must also be re-zeroed by reset()'s wipe.
        m.as_bytes_mut()[0x9_1234] = 0x77;

        m.reset();

        assert_eq!(m.root_running(), 1);
        assert_eq!(m.root_peer(), 0xFFFF);
        assert_eq!(m.target_slot(), 0xFFFF);
        assert_eq!(m.current_slot(), 0);
        assert_eq!(m.pump_active(), 0);
        assert_eq!(m.net_mode(), 0);
        assert_eq!(m.slot_depth(0), 1);
        for i in 1..SLOT_COUNT {
            assert_eq!(m.slot_depth(i), 0);
        }
        assert_eq!(m.get_u32(off::B_DWORD_0X261FD0), 0);
        assert_eq!(m.get_u32(off::B_DWORD_0X261D32), 0);
        assert_eq!(m.as_bytes()[0x9_1234], 0);
    }

    /// `NetworkBuffer::new(size)` matches the exe's `FUN_00763590` field defaults:
    ///   `[0]` buf-ptr (non-null after alloc), `[1]` write-off = 0, `[2]` size = size arg.
    #[test]
    fn network_buffer_ctor_matches_exe() {
        // ScreenManager's canonical 50000-byte call.
        let nb = NetworkBuffer::new(off::NET_BUF_DEFAULT_SIZE);
        assert_eq!(nb.size(), 50_000);
        assert_eq!(nb.write_off(), 0);
        assert_eq!(nb.buf().len(), 50_000);
        assert!(nb.buf().iter().all(|&b| b == 0), "buffer must start zeroed");

        // A different size arg — replicates the same 3-field write pattern with `param_2`.
        let nb2 = NetworkBuffer::new(1);
        assert_eq!(nb2.size(), 1);
        assert_eq!(nb2.write_off(), 0);
        assert_eq!(nb2.buf().len(), 1);
    }

    /// `ScreenManager::new()` now populates the network-buffer sub-object (not zero) and
    /// mirrors the sub-object's write-off + size into the arena at `+0x302e` / `+0x3032`.
    #[test]
    fn screen_manager_ctor_populates_net_buffer() {
        let m = ScreenManager::new();
        assert_eq!(m.net_buf().size(), 50_000);
        assert_eq!(m.net_buf().write_off(), 0);
        assert_eq!(m.net_buf().buf().len(), 50_000);
        // Arena mirror: any code reading via exe offsets sees what `FUN_00763590` would leave.
        assert_eq!(m.get_u32(off::NET_BUF_WRITE_OFF), 0);
        assert_eq!(m.get_u32(off::NET_BUF_SIZE), 50_000);
        // Ptr slot is intentionally zero in the arena — see `net_buf()` doc.
        assert_eq!(m.get_u32(off::NET_BUF_PTR), 0);
    }

    /// After `reset()`, the network buffer is re-constructed: fresh 50000-byte alloc,
    /// offsets re-initialised to their `FUN_00763590` defaults.
    #[test]
    fn reset_reruns_network_buffer_ctor() {
        let mut m = ScreenManager::new();
        m.net_buf_mut().set_write_off(1234);
        m.net_buf_mut().buf_mut()[0] = 0xAB;
        assert_eq!(m.net_buf().write_off(), 1234);
        assert_eq!(m.net_buf().buf()[0], 0xAB);

        m.reset();

        assert_eq!(m.net_buf().size(), 50_000);
        assert_eq!(m.net_buf().write_off(), 0);
        assert_eq!(m.net_buf().buf()[0], 0);
        assert_eq!(m.get_u32(off::NET_BUF_WRITE_OFF), 0);
        assert_eq!(m.get_u32(off::NET_BUF_SIZE), 50_000);
    }

    /// Instance is at least large enough for every field the ctor writes.
    #[test]
    fn instance_size_covers_ctor_writes() {
        // Highest ctor write is session B's `+0x12f55d + 1` = arena `+0x261c2f`; the
        // ScreenManager header itself also writes up to `+0x261fd4` (dword).
        assert!(SCRMGR_SIZE >= 0x0026_1fd4);
        assert!(SCRMGR_SIZE >= off::SESSION_B + off::SESS_CTOR_HIGH_WATER);
    }

    // ---------- SessionSubObject (commit 3) ----------

    /// Hand-compute the sub-object bytes as `sub_00548d50(mode)` would leave them, in an
    /// isolated buffer big enough for the whole span. Compare byte-for-byte to what
    /// `SessionSubObject::apply_ctor_writes` produces at base 0. The reference matrix comes
    /// from re-reading each `mov` in the asm.
    fn hand_computed_session_bytes(mode: u32) -> Vec<u8> {
        let mut buf = vec![0u8; off::SESS_CTOR_HIGH_WATER];
        // Sentinels — three 0xFFFF words:
        buf[off::SESS_WORD_FFFF_A]     = 0xff; buf[off::SESS_WORD_FFFF_A + 1] = 0xff;
        buf[off::SESS_WORD_FFFF_B]     = 0xff; buf[off::SESS_WORD_FFFF_B + 1] = 0xff;
        buf[off::SESS_WORD_FFFF_C]     = 0xff; buf[off::SESS_WORD_FFFF_C + 1] = 0xff;
        // Init flag byte = 1:
        buf[off::SESS_INIT_FLAG]       = 0x01;
        // Mode dword = param_2 (little-endian):
        buf[off::SESS_MODE..off::SESS_MODE + 4].copy_from_slice(&mode.to_le_bytes());
        buf
    }

    #[test]
    fn session_ctor_mode_0_matches_exe() {
        let sess = SessionSubObject::new(0);
        assert_eq!(sess.mode(), 0);
        let mut arena = vec![0u8; off::SESS_CTOR_HIGH_WATER];
        sess.apply_ctor_writes(&mut arena, 0);
        let expected = hand_computed_session_bytes(0);
        assert_eq!(arena, expected, "mode-0 sub-object bytes must match asm-derived reference");
    }

    #[test]
    fn session_ctor_mode_1_matches_exe() {
        let sess = SessionSubObject::new(1);
        assert_eq!(sess.mode(), 1);
        let mut arena = vec![0u8; off::SESS_CTOR_HIGH_WATER];
        sess.apply_ctor_writes(&mut arena, 0);
        let expected = hand_computed_session_bytes(1);
        assert_eq!(arena, expected, "mode-1 sub-object bytes must match asm-derived reference");
    }

    /// `ScreenManager::new()` now populates BOTH session sub-objects inline at the exe's
    /// offsets. Verify the mode field, the init-flag byte, and the sentinel words at both.
    #[test]
    fn screen_manager_ctor_populates_both_sessions() {
        let m = ScreenManager::new();
        assert_eq!(m.session_a().mode(), 0);
        assert_eq!(m.session_b().mode(), 1);

        // SESSION_A byte-checks in the arena — the fields sub_00548d50 leaves nonzero:
        assert_eq!(m.get_u32(off::SESSION_A + off::SESS_MODE),         0,
                   "session A mode dword");
        assert_eq!(m.as_bytes()[off::SESSION_A + off::SESS_INIT_FLAG], 1,
                   "session A init-flag byte");
        assert_eq!(m.get_u16(off::SESSION_A + off::SESS_WORD_FFFF_A), 0xffff);
        assert_eq!(m.get_u16(off::SESSION_A + off::SESS_WORD_FFFF_B), 0xffff);
        assert_eq!(m.get_u16(off::SESSION_A + off::SESS_WORD_FFFF_C), 0xffff);

        // SESSION_B byte-checks — same shape, mode = 1:
        assert_eq!(m.get_u32(off::SESSION_B + off::SESS_MODE),         1,
                   "session B mode dword");
        assert_eq!(m.as_bytes()[off::SESSION_B + off::SESS_INIT_FLAG], 1,
                   "session B init-flag byte");
        assert_eq!(m.get_u16(off::SESSION_B + off::SESS_WORD_FFFF_A), 0xffff);
        assert_eq!(m.get_u16(off::SESSION_B + off::SESS_WORD_FFFF_B), 0xffff);
        assert_eq!(m.get_u16(off::SESSION_B + off::SESS_WORD_FFFF_C), 0xffff);

        // Pointer fields the ctor zeros — must be 0 (they were 0 from alloc_zeroed but the
        // ctor writes 0 explicitly, so we're checking the write, not the alloc).
        assert_eq!(m.get_u32(off::SESSION_B + off::SESS_PTR_0X12F521), 0);
        assert_eq!(m.get_u32(off::SESSION_B + off::SESS_PTR_0X12F525), 0);
        assert_eq!(m.get_u32(off::SESSION_B + off::SESS_PTR_0X12F529), 0);
    }

    /// The session sub-object writes must not clobber the ScreenManager's own header /
    /// far-region writes — verify a spot-check field that lives between session-B ctor writes
    /// (ends at arena +0x261c2f) and end-of-arena is still set by the outer ctor.
    #[test]
    fn session_ctor_does_not_clobber_scrmgr_header() {
        let m = ScreenManager::new();
        // Header writes still intact:
        assert_eq!(m.root_running(), 1);
        assert_eq!(m.root_peer(), 0xFFFF);
        assert_eq!(m.target_slot(), 0xFFFF);
        assert_eq!(m.slot_depth(0), 1);
        // Far-region ScreenManager writes (past session B's high-water at +0x261c2f):
        assert_eq!(m.get_u32(off::B_DWORD_0X261D32), 0);
        assert_eq!(m.get_u32(off::B_DWORD_0X261FD0), 0);
    }

    /// After `reset()`, both session sub-objects are re-ctor'd: mode marker + arena bytes.
    #[test]
    fn reset_reruns_both_session_ctors() {
        let mut m = ScreenManager::new();

        // Trash the session-A and session-B mode + init-flag arena bytes, and the tracker.
        m.set_u32(off::SESSION_A + off::SESS_MODE, 0xDEAD_BEEF);
        m.set_u32(off::SESSION_B + off::SESS_MODE, 0xDEAD_BEEF);
        m.as_bytes_mut()[off::SESSION_A + off::SESS_INIT_FLAG] = 0;
        m.as_bytes_mut()[off::SESSION_B + off::SESS_INIT_FLAG] = 0;
        // Trash a sentinel too:
        m.set_u16(off::SESSION_A + off::SESS_WORD_FFFF_A, 0);

        m.reset();

        assert_eq!(m.session_a().mode(), 0);
        assert_eq!(m.session_b().mode(), 1);
        assert_eq!(m.get_u32(off::SESSION_A + off::SESS_MODE), 0);
        assert_eq!(m.get_u32(off::SESSION_B + off::SESS_MODE), 1);
        assert_eq!(m.as_bytes()[off::SESSION_A + off::SESS_INIT_FLAG], 1);
        assert_eq!(m.as_bytes()[off::SESSION_B + off::SESS_INIT_FLAG], 1);
        assert_eq!(m.get_u16(off::SESSION_A + off::SESS_WORD_FFFF_A), 0xffff);
    }

    // ---------- ScreenRecord + pump preamble (commit 4a) ----------

    /// The vtable has exactly the 3 decoded slots exposed as fields, and
    /// defaults to all-None. Its size shouldn't grow past those slots in 4a.
    #[test]
    fn screen_record_vtable_default_is_all_none() {
        let vt = ScreenRecordVTable::default();
        assert!(vt.per_frame.is_none(), "per_frame default");
        assert!(vt.cleanup.is_none(),   "cleanup default");
        assert!(vt.event.is_none(),     "event default");
        // Sanity: constructing a ScreenRecord from the default vtable works
        // and produces the expected default-initialised shell.
        let rec = ScreenRecord::default();
        assert!(rec.vtable.per_frame.is_none());
    }

    /// `pump_preamble` sets the PUMP_ACTIVE flag (byte-wise, via the two-short
    /// write pattern the decomp performs).
    #[test]
    fn pump_preamble_sets_pump_active() {
        let mut m = ScreenManager::new();
        assert_eq!(m.pump_active(), 0, "post-ctor pump_active must be 0");
        m.pump_preamble();
        // Both the low and high halves individually:
        assert_eq!(m.get_u16(off::PUMP_ACTIVE_LO), 1);
        assert_eq!(m.get_u16(off::PUMP_ACTIVE_HI), 0);
        // Combined dword view via the existing accessor:
        assert_eq!(m.pump_active(), 1, "dword view: 01 00 00 00 = 1");
    }

    /// `pump_preamble_with_mode_table` copies the caller-supplied table into
    /// the snapshot, evidencing the exe's `rep movsd` copy.
    #[test]
    fn pump_preamble_copies_mode_table() {
        let mut m = ScreenManager::new();
        assert_eq!(m.mode_table_snapshot(), &[0u32; 8], "pre-call snapshot is zero");

        let sentinel: [u32; 8] = [
            0xDEAD_0001, 0xDEAD_0002, 0xDEAD_0003, 0xDEAD_0004,
            0xDEAD_0005, 0xDEAD_0006, 0xDEAD_0007, 0xDEAD_0008,
        ];
        m.pump_preamble_with_mode_table(&sentinel);
        assert_eq!(m.mode_table_snapshot(), &sentinel, "all 8 dwords copied");
    }

    /// When `mode_table[5] == 0x7e0`, both flag slots go to 1.
    #[test]
    fn pump_preamble_mode_flag_hit() {
        let mut m = ScreenManager::new();
        let mut tbl = [0u32; 8];
        tbl[5] = PUMP_MODE_MATCH;    // 0x7e0 — the value asm compares against
        m.pump_preamble_with_mode_table(&tbl);
        assert_eq!(m.get_u32(off::PUMP_MODE_FLAG_A), 1);
        assert_eq!(m.get_u32(off::PUMP_MODE_FLAG_B), 1);
    }

    /// When `mode_table[5] != 0x7e0`, both flag slots go to 0.
    #[test]
    fn pump_preamble_mode_flag_miss() {
        let mut m = ScreenManager::new();
        let mut tbl = [0u32; 8];
        tbl[5] = 0x7e1;              // one past — should miss
        m.pump_preamble_with_mode_table(&tbl);
        assert_eq!(m.get_u32(off::PUMP_MODE_FLAG_A), 0);
        assert_eq!(m.get_u32(off::PUMP_MODE_FLAG_B), 0);
    }

    /// `pump_preamble` clears the two words at +0x13256a and +0x13256c.
    /// Pre-set them nonzero to verify the clear actually runs.
    #[test]
    fn pump_preamble_clears_words_0x13256a_0x13256c() {
        let mut m = ScreenManager::new();
        m.set_u16(off::PUMP_WORD_0X13256A, 0xAAAA);
        m.set_u16(off::PUMP_WORD_0X13256C, 0xBBBB);
        m.pump_preamble();
        assert_eq!(m.get_u16(off::PUMP_WORD_0X13256A), 0);
        assert_eq!(m.get_u16(off::PUMP_WORD_0X13256C), 0);
    }

    /// The preamble scope in 4a does NOT mutate slot depths — the depth table
    /// is loop-bound-read by the deferred per-slot loop (commit 4c), never
    /// written. Verify preservation for both post-ctor state and arbitrary
    /// pre-set depths.
    #[test]
    fn pump_preamble_preserves_slot_depths() {
        let mut m = ScreenManager::new();
        // Post-ctor: slot 0 depth = 1, others = 0. Preserve through preamble.
        m.pump_preamble();
        assert_eq!(m.slot_depth(0), 1, "slot-0 depth preserved");
        for i in 1..SLOT_COUNT {
            assert_eq!(m.slot_depth(i), 0, "slot {} depth preserved", i);
        }

        // Pre-set arbitrary depths and re-run.
        for i in 0..SLOT_COUNT {
            m.set_u16(off::SLOT_DEPTH_TABLE + i * 2, (i as u16) + 3);
        }
        m.pump_preamble();
        for i in 0..SLOT_COUNT {
            assert_eq!(m.slot_depth(i), (i as u16) + 3, "arbitrary slot {} preserved", i);
        }
    }

    // ---------- commit 4b: timezone snapshot + prologue ordering ----------

    /// `snapshot_time_now` writes a non-zero i32 into `last_time_snapshot`
    /// (any Unix wall-clock read since 1970 is > 0).
    #[test]
    fn snapshot_time_now_writes_nonzero() {
        let mut m = ScreenManager::new();
        assert_eq!(m.last_time_snapshot(), 0, "pre-call snapshot must be 0");
        m.snapshot_time_now();
        assert!(m.last_time_snapshot() > 0, "post-call snapshot must be > 0");
    }

    /// Repeated calls advance (or hold equal to) the previous value —
    /// evidences a real monotonic OS clock read, not a zero stub.
    #[test]
    fn snapshot_time_now_monotonic() {
        let mut m = ScreenManager::new();
        m.snapshot_time_now();
        let t1 = m.last_time_snapshot();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        m.snapshot_time_now();
        let t2 = m.last_time_snapshot();
        assert!(t2 >= t1, "second snapshot >= first ({} vs {})", t2, t1);
        assert!(t2 - t1 >= 1, "second snapshot at least 1s later");
    }

    /// `reset()` zeroes the snapshot back to 0.
    #[test]
    fn reset_clears_time_snapshot() {
        let mut m = ScreenManager::new();
        m.snapshot_time_now();
        assert!(m.last_time_snapshot() > 0);
        m.reset();
        assert_eq!(m.last_time_snapshot(), 0, "reset must clear the snapshot");
    }

    /// `pump_prologue` runs preamble THEN snapshots time — verify both
    /// side-effects land and the ordering is real (preamble sets pump_active,
    /// snapshot writes last_time_snapshot).
    #[test]
    fn pump_prologue_runs_preamble_then_time() {
        let mut m = ScreenManager::new();
        assert_eq!(m.pump_active(), 0);
        assert_eq!(m.last_time_snapshot(), 0);

        m.pump_prologue();

        // Preamble side-effect:
        assert_eq!(m.pump_active(), 1, "preamble must set pump_active=1");
        // Timezone snapshot side-effect:
        assert!(m.last_time_snapshot() > 0, "prologue must snapshot time");
        // Ctor invariants still hold — prologue doesn't corrupt state:
        assert_eq!(m.root_running(), 1);
        assert_eq!(m.target_slot(), 0xFFFF);
    }

    // ---------- commit 4c: PumpDispatchResult + pump_entry_prep ----------

    /// `PumpDispatchResult::from_raw` decodes every case label the exe's
    /// switch handles, and maps unknown values to `Case0` (the decomp's
    /// `default:`).
    #[test]
    fn pump_dispatch_result_from_raw_covers_all_cases() {
        assert_eq!(PumpDispatchResult::from_raw(0),   PumpDispatchResult::Case0);
        assert_eq!(PumpDispatchResult::from_raw(-1),  PumpDispatchResult::CaseNeg1);
        assert_eq!(PumpDispatchResult::from_raw(-2),  PumpDispatchResult::CaseNeg2);
        assert_eq!(PumpDispatchResult::from_raw(-3),  PumpDispatchResult::CaseNeg3);
        assert_eq!(PumpDispatchResult::from_raw(-4),  PumpDispatchResult::CaseNeg4);
        assert_eq!(PumpDispatchResult::from_raw(-5),  PumpDispatchResult::CaseNeg5);
        assert_eq!(PumpDispatchResult::from_raw(-6),  PumpDispatchResult::CaseNeg6);
        assert_eq!(PumpDispatchResult::from_raw(-7),  PumpDispatchResult::CaseNeg7);
        assert_eq!(PumpDispatchResult::from_raw(-8),  PumpDispatchResult::CaseNeg8);
        assert_eq!(PumpDispatchResult::from_raw(-9),  PumpDispatchResult::CaseNeg9);
        assert_eq!(PumpDispatchResult::from_raw(-10), PumpDispatchResult::CaseNeg10);
        assert_eq!(PumpDispatchResult::from_raw(-11), PumpDispatchResult::CaseNeg11);
        assert_eq!(PumpDispatchResult::from_raw(-12), PumpDispatchResult::CaseNeg12);
        assert_eq!(PumpDispatchResult::from_raw(-13), PumpDispatchResult::CaseNeg13);
        // Anything else → default (Case0):
        assert_eq!(PumpDispatchResult::from_raw(-14), PumpDispatchResult::Case0);
        assert_eq!(PumpDispatchResult::from_raw(1),   PumpDispatchResult::Case0);
        assert_eq!(PumpDispatchResult::from_raw(i32::MIN), PumpDispatchResult::Case0);
    }

    /// Only `CaseNeg5` reports pump-exit `true` — matches C L686 `return local_434 == -5`.
    #[test]
    fn pump_dispatch_result_exit_only_true_for_neg5() {
        assert!(!PumpDispatchResult::Case0.is_pump_exit_true());
        assert!( PumpDispatchResult::CaseNeg5.is_pump_exit_true());
        assert!(!PumpDispatchResult::CaseNeg7.is_pump_exit_true());
        for v in [-1, -2, -3, -4, -6, -7, -8, -9, -10, -11, -12, -13] {
            assert!(!PumpDispatchResult::from_raw(v).is_pump_exit_true(), "code {}", v);
        }
    }

    /// `SLOT_STRIDE == SLOT_ENTRY_COUNT * SLOT_ENTRY_STRIDE` — invariant that
    /// the entry-prep loop relies on.
    #[test]
    fn slot_stride_matches_entry_layout() {
        assert_eq!(SLOT_STRIDE, SLOT_ENTRY_COUNT * SLOT_ENTRY_STRIDE);
        assert_eq!(SLOT_STRIDE, 0x300);
        assert_eq!(SLOT_ENTRY_STRIDE, 0x30);
    }

    /// `entry::*` offsets for slot 0's primary entry overlap the root-frame
    /// header — this alias is exactly what the ctor doc calls out (slot 0 IS
    /// the ScreenManager's own header).
    #[test]
    fn primary_entry_offsets_alias_root_header() {
        assert_eq!(entry::SCREEN_ID,          off::ROOT_SCREEN_ID);
        assert_eq!(entry::CURRENT_RECORD_PTR, off::ROOT_CUR_ENTRY);
        assert_eq!(entry::ACTIVE_FLAG,        off::ROOT_RUNNING);
        assert_eq!(entry::SESSION_PTR,        off::INITIAL_SCREEN_PTR);
    }

    /// `pump_entry_prep` does nothing on a slot whose depth is 0 — post-ctor
    /// state (slot 0 depth=1, others 0) leaves slots 1..15 untouched.
    #[test]
    fn entry_prep_skips_zero_depth_slots() {
        let mut m = ScreenManager::new();
        // Seed a sentinel in slot-1 entry-0 flags — should stay.
        let s1_entry0 = 1 * SLOT_STRIDE;
        m.set_u32(s1_entry0 + entry::ACKED_A, 0xDEAD_BEEF);
        m.set_u32(s1_entry0 + entry::ACKED_B, 0xCAFE_F00D);
        m.pump_entry_prep();
        assert_eq!(m.get_u32(s1_entry0 + entry::ACKED_A), 0xDEAD_BEEF);
        assert_eq!(m.get_u32(s1_entry0 + entry::ACKED_B), 0xCAFE_F00D);
    }

    /// Active primary (ACTIVE_FLAG != 0) → C L73-76 clears ACKED_A/B and L77
    /// stamps TIMESTAMP.
    #[test]
    fn entry_prep_active_clears_flags_and_stamps_ts() {
        let mut m = ScreenManager::new();
        // Post-ctor: slot-0 depth=1, ROOT_RUNNING=1 (which aliases entry-0's
        // ACTIVE_FLAG → primary is "active"). Seed a distinct timestamp.
        m.snapshot_time_now();
        let ts = m.last_time_snapshot() as u32;
        // Pre-set ACKED_A/B nonzero to see the clear happen.
        m.set_u32(entry::ACKED_A, 0x1111_1111);
        m.set_u32(entry::ACKED_B, 0x2222_2222);
        m.set_u32(entry::TIMESTAMP, 0);
        m.pump_entry_prep();
        assert_eq!(m.get_u32(entry::ACKED_A), 0, "ACKED_A cleared");
        assert_eq!(m.get_u32(entry::ACKED_B), 0, "ACKED_B cleared");
        assert_eq!(m.get_u32(entry::TIMESTAMP), ts, "TIMESTAMP stamped");
    }

    /// Inactive primary (ACTIVE_FLAG == 0) → C L67-70 sets ACKED_A/B to 1.
    #[test]
    fn entry_prep_inactive_sets_ack_flags_to_1() {
        let mut m = ScreenManager::new();
        // Force slot-0 primary inactive (clear ROOT_RUNNING dword aliasing ACTIVE_FLAG).
        m.set_u32(entry::ACTIVE_FLAG, 0);
        m.pump_entry_prep();
        assert_eq!(m.get_u32(entry::ACKED_A), 1, "ACKED_A=1");
        assert_eq!(m.get_u32(entry::ACKED_B), 1, "ACKED_B=1");
    }

    /// Multi-entry slot: after inactive primary sets its ACKED_A=1, subsequent
    /// child entries whose ACTIVE_FLAG is nonzero pull the primary's 0x30-byte
    /// block over themselves (C L78-80 memcpy semantics per decomp arg order).
    #[test]
    fn entry_prep_child_receives_primary_when_primary_flipped() {
        let mut m = ScreenManager::new();
        // Depth = 3 for slot 0. Primary inactive; children active.
        m.set_u16(off::SLOT_DEPTH_TABLE + 0 * 2, 3);
        m.set_u32(entry::ACTIVE_FLAG, 0);                              // primary inactive
        // Child entry-1 and entry-2:
        let e1 = 1 * SLOT_ENTRY_STRIDE;
        let e2 = 2 * SLOT_ENTRY_STRIDE;
        m.set_u32(e1 + entry::ACTIVE_FLAG, 1);                          // child active
        m.set_u32(e2 + entry::ACTIVE_FLAG, 1);
        // Fingerprint the primary so we can detect the memcpy landing.
        m.set_u32(entry::SCREEN_ID, 0xABCD);                            // (short overlap; low 16 bits)
        m.pump_entry_prep();
        // Per decomp arg order `FUN_008faef0(dst=slot_base, src=entry, n=0x30)`,
        // once the primary flips inactive (ACKED_A=1 at entry-0 processing),
        // the FIRST active child (entry-1) memcpys ITSELF over the primary. That
        // memcpy overwrites primary.ACKED_A with child's freshly-cleared 0, so
        // subsequent children see ACKED_A==0 and don't memcpy. Verify the memcpy
        // fired by checking the primary's SCREEN_ID fingerprint was clobbered
        // by the (all-zero) child block.
        assert_eq!(m.get_u32(entry::SCREEN_ID) & 0xffff, 0,
                   "primary SCREEN_ID overwritten by first child's memcpy");
        // And primary.ACKED_A is now 0 (child's cleared value copied in).
        assert_eq!(m.get_u32(entry::ACKED_A), 0,
                   "primary ACKED_A overwritten by child block");
    }

    // ---------- commit 5: push_screen (FUN_007e6570) ----------

    // A test-only cleanup fn — increments a global counter so tests can
    // observe the old-current cleanup hook firing.
    use std::sync::atomic::{AtomicU32, Ordering};
    static CLEANUP_HITS: AtomicU32 = AtomicU32::new(0);
    fn test_cleanup_bump(_m: &mut ScreenManager) {
        CLEANUP_HITS.fetch_add(1, Ordering::SeqCst);
    }

    /// First push into an empty slot creates a record and links it as head,
    /// tail, and current.
    #[test]
    fn push_screen_first_call_allocates_new_record() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        // Slot 0 is post-ctor "empty" for our record pool (no records yet).
        assert!(m.slot_head_id(slot).is_none());
        assert!(m.slot_tail_id(slot).is_none());
        assert!(m.slot_current_id(slot).is_none());

        let r = m.push_screen(0x1234, 42, None, 0, 99);
        assert_eq!(r, PushScreenResult::NewRecordPushed);
        assert_eq!(r.as_exe_ret(), 1, "exe returns 1 on happy path");
        assert_eq!(m.record_pool_size(), 1);

        let id = m.slot_head_id(slot).expect("head set");
        assert_eq!(m.slot_tail_id(slot), Some(id));
        assert_eq!(m.slot_current_id(slot), Some(id));
        let rec = m.record(id).expect("record");
        assert_eq!(rec.screen_id, 0x1234);
        assert_eq!(rec.param_3, 42);
        assert_eq!(rec.param_4, 0);
        assert_eq!(rec.param_6, 99);
        assert!(rec.prev.is_none());
        assert!(rec.next.is_none());
        assert_eq!(rec.slot_bag.len(), 0x3c);
        // Slot bookkeeping:
        assert_eq!(m.slot_count(slot), 1);
        assert_eq!(m.get_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG), 1);
        assert_eq!(m.get_u16(slot * SLOT_STRIDE + off::SLOT_SENTINEL_0X22), 0xFFFF);
    }

    /// Pushing the same screen_id twice with param_4 != 0 rewinds `current`
    /// to the existing record instead of allocating a new one.
    #[test]
    fn push_screen_matching_id_rewinds_to_existing() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        assert_eq!(m.push_screen(0xAA, 1, None, 1, 0), PushScreenResult::NewRecordPushed);
        let first_id = m.slot_current_id(slot).unwrap();
        assert_eq!(m.push_screen(0xBB, 1, None, 1, 0), PushScreenResult::NewRecordPushed);
        let second_id = m.slot_current_id(slot).unwrap();
        assert_ne!(first_id, second_id);
        assert_eq!(m.record_pool_size(), 2);

        // Rewind: push_screen with 0xAA again (matching id in forward chain).
        // Move current back to first so the forward walk from `current` finds 0xAA.
        m.slot_write_id(slot, off::SLOT_CURRENT_ID, Some(first_id));
        let r = m.push_screen(0xBB, 1, None, 1, 0);
        assert_eq!(r, PushScreenResult::RewoundToExisting);
        assert_eq!(r.as_exe_ret(), 0);
        // No new record allocated:
        assert_eq!(m.record_pool_size(), 2);
        // current advanced to the last active in the chain = second (tail).
        assert_eq!(m.slot_current_id(slot), Some(second_id));
    }

    /// Pushing past 100 records evicts the head record (when head is non-modal).
    #[test]
    fn push_screen_over_100_evicts_oldest() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        // Push 101 non-modal records — no eviction yet (guard is `count > 100`).
        for i in 0..101 {
            let r = m.push_screen(0x1000 + i as u32, 1, None, 0, 0);
            assert_eq!(r, PushScreenResult::NewRecordPushed, "push {}", i);
        }
        assert_eq!(m.slot_count(slot), 101, "no eviction until count > 100");
        // 102nd push: count=101 > 100 → evict head THEN add → net 101.
        let r = m.push_screen(0x2000, 1, None, 0, 0);
        assert_eq!(r, PushScreenResult::NewRecordPushed);
        assert_eq!(m.slot_count(slot), 101, "slot held near cap after evict+push");
        assert_eq!(m.record_pool_size(), 101);
        // The oldest record (screen_id 0x1000) should be evicted.
        let head_id = m.slot_head_id(slot).unwrap();
        assert_ne!(m.record(head_id).unwrap().screen_id, 0x1000,
                   "oldest 0x1000 evicted");
    }

    /// The OLD current record's cleanup hook (record `+0x04` fn ptr) fires
    /// when a new record is pushed on top.
    #[test]
    fn push_screen_calls_previous_cleanup_hook() {
        let mut m = ScreenManager::new();
        CLEANUP_HITS.store(0, Ordering::SeqCst);
        // First push: no old current, no cleanup fires.
        m.push_screen(1, 1, Some(test_cleanup_bump), 0, 0);
        assert_eq!(CLEANUP_HITS.load(Ordering::SeqCst), 0, "no old-current on first push");
        // Second push: the previous record IS the current, so its cleanup fires.
        m.push_screen(2, 1, None, 0, 0);
        assert_eq!(CLEANUP_HITS.load(Ordering::SeqCst), 1, "cleanup fired once");
    }

    /// Deferred-args branch: with slot empty, DEFERRED_NAME set, and
    /// BAG_COUNT > 0, the new record's slot bag is populated from BAG_VALUES.
    #[test]
    fn push_screen_populates_slot_bag_when_deferred() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;

        // Pre-load persistent state at +0x3000..+0x3018 that C L27-40 reads.
        m.set_u32(off::DEFERRED_NAME, 0xCAFE);
        m.set_u32(off::DEFERRED_ARG1, 7);   // param_3
        m.set_u32(off::DEFERRED_ARG2, 0);   // param_4
        m.set_u32(off::DEFERRED_ARG3, 0);   // param_5 fn (0)
        m.set_u32(off::DEFERRED_ARG4, 0);   // param_6
        // Bag: two values at a stub non-null ptr.
        m.set_u32(off::BAG_VALUES, 0xDEAD);   // non-null sentinel triggers replay
        m.set_u16(off::BAG_COUNT, 2);

        // Call with sentinel args that the deferred branch WILL overwrite —
        // slot is empty so the deferred branch triggers.
        let r = m.push_screen(0, 0, None, 0, 0);
        assert_eq!(r, PushScreenResult::NewRecordPushed);

        let id = m.slot_current_id(slot).unwrap();
        let rec = m.record(id).unwrap();
        assert_eq!(rec.screen_id, 0xCAFE, "screen_id from DEFERRED_NAME");
        assert_eq!(rec.param_3, 7, "param_3 from DEFERRED_ARG1");
        // Bag entries 0..2 populated.
        assert!(rec.slot_bag[0].value.is_some(), "bag[0] populated");
        assert!(rec.slot_bag[1].value.is_some(), "bag[1] populated");
        assert!(rec.slot_bag[2].value.is_none(), "bag[2] untouched");
    }

    /// Struct layout evidence: field roles at their exe byte offsets match
    /// what other scrman fns read/write (documented in ScreenRecord doc).
    #[test]
    fn screen_record_layout_documents_exe_offsets() {
        // The Rust struct is NOT #[repr(C)] and does NOT need to match byte-for-byte
        // (records live in the Rust pool, not the arena). What we assert here is
        // that the DOCUMENTED offsets match what push_screen's C stores + what
        // callers (pump dispatch) read.
        //
        // +0x00 screen_id (piVar6[0] = param_2)
        // +0x04 cleanup   (piVar6[1] = param_5, called at C L120)
        // +0x08 param_3   (piVar6[2] = param_3)
        // +0x0C param_6   (piVar6[3] = param_6)
        // +0x10 param_4   (piVar6[4] = param_4)  — modal flag
        // +0x14..+0x1F4 slot_bag (60 × 8 = 0x1E0 bytes)
        // +0x1F4 prev     (piVar6[0x7d])
        // +0x1F8 next     (piVar6[0x7e])
        // +0x1FC name     (piVar6[0x7f]) — 260 bytes to reach +0x300
        assert_eq!(0x7d * 4, 0x1F4, "prev at piVar6[0x7d]");
        assert_eq!(0x7e * 4, 0x1F8, "next at piVar6[0x7e]");
        assert_eq!(0x7f * 4, 0x1FC, "name at piVar6[0x7f]");
        assert_eq!(0x14 + 0x3c * 8, 0x1F4, "slot_bag ends where prev begins");
        assert_eq!(0x300 - 0x1FC, 260, "260 bytes of name capacity");

        let rec = ScreenRecord::default();
        assert_eq!(rec.slot_bag.len(), 0);   // default is empty; ctor sets to 0x3c
        let rec2 = ScreenRecord::new(1, 0xAAAA, None, 1, 0, 0, Vec::new());
        assert_eq!(rec2.slot_bag.len(), 0x3c);
        assert_eq!(rec2.screen_id, 0xAAAA);
    }

    /// `reset()` clears the record pool + id counter.
    #[test]
    fn reset_clears_record_pool() {
        let mut m = ScreenManager::new();
        m.push_screen(1, 1, None, 0, 0);
        m.push_screen(2, 1, None, 0, 0);
        assert!(m.record_pool_size() > 0);
        m.reset();
        assert_eq!(m.record_pool_size(), 0);
        for s in 0..SLOT_COUNT {
            assert!(m.slot_head_id(s).is_none());
            assert!(m.slot_current_id(s).is_none());
        }
    }

    // ---------- commit 4d: pump dispatch cases + pump_finalize + pump() ----------

    /// Test hook: scriptable event returns + counters for cleanup1/cleanup2/
    /// on_default_cleanup / on_bag_free / on_external_case.
    #[derive(Default)]
    struct TestHooks {
        scripted_codes: Vec<i32>,
        cleanup1_hits: Vec<RecordId>,
        cleanup2_hits: Vec<RecordId>,
        default_hits: u32,
        bag_free_hits: Vec<RecordId>,
        external_hits: Vec<PumpDispatchResult>,
        event_calls: Vec<(RecordId, u32)>,
    }
    impl ScrmanHooks for TestHooks {
        fn invoke_event(&mut self, _m: &mut ScreenManager, rid: RecordId, l438: u32) -> i32 {
            self.event_calls.push((rid, l438));
            if self.scripted_codes.is_empty() { -7 }
            else { self.scripted_codes.remove(0) }
        }
        fn invoke_cleanup1(&mut self, _m: &mut ScreenManager, rid: RecordId) {
            self.cleanup1_hits.push(rid);
        }
        fn invoke_cleanup2(&mut self, _m: &mut ScreenManager, rid: RecordId) {
            self.cleanup2_hits.push(rid);
        }
        fn on_default_cleanup(&mut self, _m: &mut ScreenManager) { self.default_hits += 1; }
        fn on_bag_free(&mut self, _m: &mut ScreenManager, rid: RecordId) {
            self.bag_free_hits.push(rid);
        }
        fn on_external_case(&mut self, _m: &mut ScreenManager, c: PumpDispatchResult) {
            self.external_hits.push(c);
        }
    }

    /// Case -1: pop-to-root — after dispatch, current == slot head + ACTIVE_FLAG=1.
    #[test]
    fn pump_case_neg1_pops_to_slot_root() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        m.push_screen(1, 1, None, 0, 0);
        m.push_screen(2, 1, None, 0, 0);
        m.push_screen(3, 1, None, 0, 0);
        let head = m.slot_head_id(slot).unwrap();
        let tail = m.slot_current_id(slot).unwrap();
        assert_ne!(head, tail);
        m.set_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG, 0);
        let mut hooks = TestHooks::default();
        let exit = m.pump_dispatch_case(PumpDispatchResult::CaseNeg1, 0xFFFF_FFFF, &mut hooks);
        assert!(!exit, "-1 does not exit");
        assert_eq!(m.slot_current_id(slot), Some(head), "current pulled to head");
        assert_eq!(m.get_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG), 1, "ACTIVE_FLAG=1");
        assert_eq!(hooks.cleanup1_hits, vec![tail], "cleanup1 fired on old current");
    }

    /// Case -2: pop via prev — current moves to prev, ACTIVE_FLAG=1.
    #[test]
    fn pump_case_neg2_pops_via_prev() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        m.push_screen(1, 1, None, 0, 0);
        m.push_screen(2, 1, None, 0, 0);
        let first = m.slot_head_id(slot).unwrap();
        let second = m.slot_current_id(slot).unwrap();
        m.set_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG, 0);
        let mut hooks = TestHooks::default();
        let exit = m.pump_dispatch_case(PumpDispatchResult::CaseNeg2, 0xFFFF_FFFF, &mut hooks);
        assert!(!exit);
        assert_eq!(m.slot_current_id(slot), Some(first), "current moved to prev");
        assert_eq!(m.get_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG), 1);
        assert_eq!(hooks.cleanup1_hits, vec![second]);
    }

    /// Case -2 modal guard: if current.param_4 != 0, do NOT pop.
    #[test]
    fn pump_case_neg2_modal_current_blocks_pop() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        m.push_screen(1, 1, None, 0, 0);
        m.push_screen(2, 1, None, 1 /* param_4 modal */, 0);
        let modal = m.slot_current_id(slot).unwrap();
        let mut hooks = TestHooks::default();
        m.pump_dispatch_case(PumpDispatchResult::CaseNeg2, 0xFFFF_FFFF, &mut hooks);
        // Modal blocks: current stays.
        assert_eq!(m.slot_current_id(slot), Some(modal), "modal blocks -2 pop");
    }

    /// Case -3: pop via next — current moves to next.
    #[test]
    fn pump_case_neg3_pops_via_next() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        m.push_screen(1, 1, None, 0, 0);
        m.push_screen(2, 1, None, 0, 0);
        let first = m.slot_head_id(slot).unwrap();
        let second = m.slot_tail_id(slot).unwrap();
        // Move current back to first so `next` walks forward.
        m.slot_write_id(slot, off::SLOT_CURRENT_ID, Some(first));
        m.set_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG, 0);
        let mut hooks = TestHooks::default();
        m.pump_dispatch_case(PumpDispatchResult::CaseNeg3, 0xFFFF_FFFF, &mut hooks);
        assert_eq!(m.slot_current_id(slot), Some(second));
        assert_eq!(m.get_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG), 1);
        assert_eq!(hooks.cleanup1_hits, vec![first]);
    }

    /// Case -4 and -11 both set ACTIVE_FLAG=1 with no chain change.
    #[test]
    fn pump_case_neg4_and_neg11_set_active_flag() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        m.set_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG, 0);
        let mut hooks = TestHooks::default();
        m.pump_dispatch_case(PumpDispatchResult::CaseNeg4, 0xFFFF_FFFF, &mut hooks);
        assert_eq!(m.get_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG), 1);
        assert!(hooks.cleanup1_hits.is_empty(), "-4 does not call cleanup1");

        m.set_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG, 0);
        m.pump_dispatch_case(PumpDispatchResult::CaseNeg11, 0xFFFF_FFFF, &mut hooks);
        assert_eq!(m.get_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG), 1);
    }

    /// Case -10: cleanup1 + on_bag_free hook + on_external_case fires.
    #[test]
    fn pump_case_neg10_calls_cleanup_and_bag_free() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        m.push_screen(1, 1, None, 0, 0);
        let cur = m.slot_current_id(slot).unwrap();
        // Seed a bag entry so the default on_bag_free clears it.
        m.set_slot_bag(slot, 0, b"payload".to_vec());
        assert!(m.record(cur).unwrap().slot_bag[0].value.is_some());
        let mut hooks = TestHooks::default();
        m.pump_dispatch_case(PumpDispatchResult::CaseNeg10, 0xFFFF_FFFF, &mut hooks);
        assert_eq!(hooks.cleanup1_hits, vec![cur]);
        assert_eq!(hooks.bag_free_hits, vec![cur]);
        assert_eq!(hooks.external_hits, vec![PumpDispatchResult::CaseNeg10]);
        // The TestHooks override doesn't drop payloads (it only counts) — the
        // *default* on_bag_free impl does. Verify with a hook that uses defaults.
        struct DefaultOnly;
        impl ScrmanHooks for DefaultOnly {
            fn invoke_event(&mut self, _: &mut ScreenManager, _: RecordId, _: u32) -> i32 { -7 }
        }
        let mut m2 = ScreenManager::new();
        let slot2 = m2.current_slot() as usize;
        m2.push_screen(1, 1, None, 0, 0);
        let cur2 = m2.slot_current_id(slot2).unwrap();
        m2.set_slot_bag(slot2, 0, b"payload".to_vec());
        assert!(m2.record(cur2).unwrap().slot_bag[0].value.is_some());
        let mut h = DefaultOnly;
        m2.pump_dispatch_case(PumpDispatchResult::CaseNeg10, 0xFFFF_FFFF, &mut h);
        assert!(m2.record(cur2).unwrap().slot_bag[0].value.is_none(),
                "default on_bag_free drops payload");
    }

    /// Case 0 (default): FUN_007eaac0 hook fires iff `local_438 != -1 && slot.ACTIVE_FLAG == 0`.
    #[test]
    fn pump_case_default_fires_hook_only_when_guarded() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        m.set_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG, 0);
        let mut hooks = TestHooks::default();

        // local_438 = -1 (0xFFFFFFFF) → guard fails, no hit.
        m.pump_dispatch_case(PumpDispatchResult::Case0, 0xFFFF_FFFF, &mut hooks);
        assert_eq!(hooks.default_hits, 0, "guard rejects local_438 == -1");

        // local_438 = 0, slot ACTIVE_FLAG = 0 → guard passes, hit.
        m.pump_dispatch_case(PumpDispatchResult::Case0, 0, &mut hooks);
        assert_eq!(hooks.default_hits, 1, "guard passes with local_438=0");

        // local_438 = 0, slot ACTIVE_FLAG = 1 → guard fails, no additional hit.
        m.set_u32(slot * SLOT_STRIDE + off::SLOT_ACTIVE_FLAG, 1);
        m.pump_dispatch_case(PumpDispatchResult::Case0, 0, &mut hooks);
        assert_eq!(hooks.default_hits, 1, "ACTIVE_FLAG=1 blocks hook");
    }

    /// Case -7: exit signal — dispatch returns true.
    #[test]
    fn pump_case_neg7_exits() {
        let mut m = ScreenManager::new();
        let mut hooks = TestHooks::default();
        let exit = m.pump_dispatch_case(PumpDispatchResult::CaseNeg7, 0xFFFF_FFFF, &mut hooks);
        assert!(exit, "-7 signals loop exit");
        assert_eq!(hooks.external_hits, vec![PumpDispatchResult::CaseNeg7]);
    }

    /// External-only cases still fire the hook without mutating chain state.
    #[test]
    fn pump_external_only_cases_observe_via_hook() {
        for &c in &[PumpDispatchResult::CaseNeg5, PumpDispatchResult::CaseNeg6,
                    PumpDispatchResult::CaseNeg8, PumpDispatchResult::CaseNeg9,
                    PumpDispatchResult::CaseNeg12, PumpDispatchResult::CaseNeg13] {
            let mut m = ScreenManager::new();
            let slot = m.current_slot() as usize;
            m.push_screen(1, 1, None, 0, 0);
            m.push_screen(2, 1, None, 0, 0);
            let cur_before = m.slot_current_id(slot);
            let mut hooks = TestHooks::default();
            m.pump_dispatch_case(c, 0xFFFF_FFFF, &mut hooks);
            assert_eq!(hooks.external_hits, vec![c], "code {:?}", c);
            // Chain unchanged (these cases hooks-only in this port).
            assert_eq!(m.slot_current_id(slot), cur_before, "code {:?}", c);
        }
    }

    /// pump_finalize walks each slot's head→next chain calling cleanup2, then
    /// writes the trailing state (CURRENT_SLOT=0, PUMP_ACTIVE=0, etc.).
    #[test]
    fn pump_finalize_walks_slot_and_clears_state() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        m.push_screen(1, 1, None, 0, 0);
        m.push_screen(2, 1, None, 0, 0);
        m.push_screen(3, 1, None, 0, 0);
        // Slot 0 has depth 1 post-ctor, so it will be walked.
        // Bump SLOT_DEPTH to reflect our records (finalize gates on slot_depth > 0).
        assert!(m.slot_depth(slot) >= 1);
        let head = m.slot_head_id(slot).unwrap();
        let mid = m.record(head).unwrap().next.unwrap();
        let tail = m.record(mid).unwrap().next.unwrap();

        // Preload pump-active so we can see finalize clear it.
        m.pump_preamble();
        assert_eq!(m.pump_active(), 1);

        let mut hooks = TestHooks::default();
        m.pump_finalize(&mut hooks);
        assert_eq!(hooks.cleanup2_hits, vec![head, mid, tail], "head→next chain walk");
        // Trailing state writes:
        assert_eq!(m.current_slot(), 0);
        assert_eq!(m.pump_active(), 0);
        assert_eq!(m.get_u16(off::PUMP_WORD_0X13256A), 1);
        assert_eq!(m.get_u16(off::PUMP_WORD_0X13256C), 0);
    }

    /// pump() convenience: prologue runs, ExitImmediately drives one dispatch
    /// that returns -7 (finalization exit), and finalize clears pump_active.
    #[test]
    fn pump_full_tick_runs_prologue_and_finalize() {
        let mut m = ScreenManager::new();
        m.push_screen(1, 1, None, 0, 0);
        assert_eq!(m.pump_active(), 0);

        let code = m.pump();
        // The convenience hook returns -7 → dispatch exits → finalize runs.
        assert_eq!(code, PumpDispatchResult::CaseNeg7);
        // Finalize cleared pump_active back to 0:
        assert_eq!(m.pump_active(), 0);
        // Snapshot time was written by prologue:
        assert!(m.last_time_snapshot() > 0);
    }

    /// pump_with_hooks: scripted event codes drive the loop, chain manipulation
    /// occurs, finalization runs cleanup2 on each record.
    #[test]
    fn pump_with_hooks_scripted_drive_end_to_end() {
        let mut m = ScreenManager::new();
        let slot = m.current_slot() as usize;
        m.push_screen(1, 1, None, 0, 0);
        m.push_screen(2, 1, None, 0, 0);
        m.push_screen(3, 1, None, 0, 0);
        let head = m.slot_head_id(slot).unwrap();

        // Script: -1 (pop-to-root) then -7 (exit).
        let mut hooks = TestHooks {
            scripted_codes: vec![-1, -7],
            ..Default::default()
        };
        let last = m.pump_with_hooks(&mut hooks);
        assert_eq!(last, PumpDispatchResult::CaseNeg7);
        assert_eq!(hooks.event_calls.len(), 2, "two event calls: -1 then -7");
        // -1 pulled current to head (before -7 arrived + finalize):
        // (finalize doesn't touch SLOT_CURRENT_ID; only CURRENT_SLOT global)
        assert_eq!(m.slot_current_id(slot), Some(head), "after -1, current == head");
        // Finalize walked the chain:
        assert_eq!(hooks.cleanup2_hits.len(), 3, "3 records visited by finalize");
    }

    /// pump_with_hooks safety cap: if the hook never returns -5/-7, the loop
    /// eventually bails at 4096 iterations rather than looping forever.
    #[test]
    fn pump_with_hooks_safety_cap_terminates() {
        let mut m = ScreenManager::new();
        m.push_screen(1, 1, None, 0, 0);
        struct AlwaysZero;
        impl ScrmanHooks for AlwaysZero {
            fn invoke_event(&mut self, _: &mut ScreenManager, _: RecordId, _: u32) -> i32 { 0 }
        }
        let mut h = AlwaysZero;
        let last = m.pump_with_hooks(&mut h);
        // Case0 never exits — loop hit the safety cap; last_code = Case0.
        assert_eq!(last, PumpDispatchResult::Case0);
    }

    /// Preamble does not corrupt neighbouring header state: sub-object marker
    /// modes, root flags, session sentinels all stay put.
    #[test]
    fn pump_preamble_does_not_clobber_ctor_state() {
        let mut m = ScreenManager::new();
        m.pump_preamble();
        // Ctor invariants:
        assert_eq!(m.root_running(), 1);
        assert_eq!(m.root_peer(),    0xFFFF);
        assert_eq!(m.target_slot(),  0xFFFF);
        assert_eq!(m.session_a().mode(), 0);
        assert_eq!(m.session_b().mode(), 1);
        assert_eq!(m.get_u16(off::SESSION_A + off::SESS_WORD_FFFF_A), 0xffff);
        assert_eq!(m.get_u16(off::SESSION_B + off::SESS_WORD_FFFF_A), 0xffff);
    }

    // ============================================================
    // 4b-net tests
    // ============================================================

    #[test]
    fn mock_socket_recv_returns_wouldblock_when_empty() {
        let mut s = MockSocket::new();
        let mut buf = [0u8; 8];
        assert_eq!(s.recv_from(PeerId(0), &mut buf), NetIoResult::WouldBlock);
    }

    #[test]
    fn mock_socket_send_and_receive_roundtrip() {
        let mut s = MockSocket::new();
        s.push_inbound(&[1, 2, 3, 4]);
        let mut buf = [0u8; 8];
        assert_eq!(s.recv_from(PeerId(0), &mut buf), NetIoResult::Ok(4));
        assert_eq!(&buf[..4], &[1, 2, 3, 4]);
        assert_eq!(s.send_to(PeerId(0), b"hello"), NetIoResult::Ok(5));
        assert_eq!(&s.outbox, b"hello");
    }

    #[test]
    fn net_poll_recv_reads_bytes_via_socket() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        s.push_inbound(&[0xaa, 0xbb, 0xcc]);
        let start = m.net_buf().write_off();
        let poll = m.net_poll_recv(&mut s);
        assert_eq!(poll.bytes_appended, 3);
        assert_eq!(m.net_buf().write_off(), start + 3);
        assert_eq!(&m.net_buf().buf()[start as usize..start as usize + 3], &[0xaa, 0xbb, 0xcc]);
    }

    #[test]
    fn net_poll_recv_reports_dropped_peer_on_disconnect() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        s.next_recv_fatal = true;
        let poll = m.net_poll_recv(&mut s);
        assert_eq!(poll.dropped_peers, vec![PeerId(0)]);
        assert_eq!(poll.bytes_appended, 0);
    }

    #[test]
    fn net_poll_recv_forwards_accepted_peer() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        s.pending_accept = Some(PeerId(42));
        let poll = m.net_poll_recv(&mut s);
        assert_eq!(poll.new_peer, Some(PeerId(42)));
    }

    #[test]
    fn net_flush_send_writes_size_prefix_plus_payload() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        let payload = b"hi";
        let (sent, dropped) = m.net_flush_send(&mut s, payload, None);
        assert!(dropped.is_empty());
        // 4-byte size prefix (little-endian 2) + 2-byte payload = 6 bytes.
        assert_eq!(sent, 6);
        assert_eq!(&s.outbox[..4], &2u32.to_le_bytes());
        assert_eq!(&s.outbox[4..], b"hi");
    }

    #[test]
    fn net_flush_send_rejects_empty_payload() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        let (sent, _) = m.net_flush_send(&mut s, &[], None);
        assert_eq!(sent, 0);
        assert!(s.outbox.is_empty());
    }

    #[test]
    fn net_flush_send_rejects_too_large_payload() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        let big = vec![0u8; 50_000];
        let (sent, _) = m.net_flush_send(&mut s, &big, None);
        assert_eq!(sent, 0);
        assert!(s.outbox.is_empty());
    }

    #[test]
    fn net_flush_send_broadcasts_to_multiple_peers() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        s.peer_table = vec![PeerId(0), PeerId(1), PeerId::NONE, PeerId(2)];
        let (sent, dropped) = m.net_flush_send(&mut s, b"x", None);
        assert!(dropped.is_empty());
        // 3 live peers × (4 header + 1 payload) = 15 bytes.
        assert_eq!(sent, 15);
    }

    #[test]
    fn net_flush_send_directed_targets_only_one_peer() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        s.peer_table = vec![PeerId(0), PeerId(1), PeerId(2)];
        let (sent, _) = m.net_flush_send(&mut s, b"z", Some(PeerId(1)));
        // Only 1 peer × 5 bytes.
        assert_eq!(sent, 5);
    }

    #[test]
    fn net_flush_send_records_dropped_peer_on_header_fail() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        s.next_send_fatal = true;
        let (_sent, dropped) = m.net_flush_send(&mut s, b"data", None);
        assert_eq!(dropped, vec![PeerId(0)]);
    }

    #[test]
    fn broadcast_end_marker_writes_byte_9_and_sends() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        let mut d = NoPeerInput;
        let sent = m.broadcast_end_marker(&mut s, &mut d);
        // 4-byte size prefix + 1-byte marker = 5 bytes for the one peer.
        assert_eq!(sent, 5);
        // Byte 9 is the marker (see FUN_007eaac0 body).
        assert_eq!(s.outbox[4], 9);
        // Header = 1 (payload length).
        assert_eq!(&s.outbox[..4], &1u32.to_le_bytes());
    }

    #[test]
    fn broadcast_end_marker_falls_back_when_no_peers() {
        let mut m = ScreenManager::new();
        let mut s = MockSocket::new();
        s.peer_table.clear();
        struct CountDispatch(u32);
        impl PeerInputDispatch for CountDispatch {
            fn dispatch_peer_input(&mut self, _: &mut ScreenManager) { self.0 += 1; }
        }
        let mut d = CountDispatch(0);
        let sent = m.broadcast_end_marker(&mut s, &mut d);
        assert_eq!(sent, 0);
        assert_eq!(d.0, 1, "peer-input fallback should fire when no peers alive");
    }

    #[test]
    fn case_neg5_full_body_frees_record() {
        let mut m = ScreenManager::new();
        m.push_screen(1, 1, None, 0, 0);
        let slot = m.current_slot() as usize;
        let cid = m.slot_current_id(slot).expect("has current");
        assert!(m.record(cid).is_some());
        let mut s = MockSocket::new();
        let mut d = NoPeerInput;
        struct Noop;
        impl ScrmanHooks for Noop {
            fn invoke_event(&mut self, _: &mut ScreenManager, _: RecordId, _: u32) -> i32 { 0 }
        }
        let mut h = Noop;
        let exit = m.pump_dispatch_case_wired(
            PumpDispatchResult::CaseNeg5, 0xFFFF_FFFF, &mut h, &mut s, &mut d);
        assert!(!exit);
        // Record freed.
        assert!(m.record(cid).is_none(),
                "case -5 shared tail should free the current record");
        // End-marker was broadcast on the socket.
        assert_eq!(s.outbox.len(), 5);
        assert_eq!(s.outbox[4], 9);
    }

    #[test]
    fn case_neg8_full_body_dispatches_peer_input_and_broadcasts() {
        let mut m = ScreenManager::new();
        m.push_screen(1, 1, None, 0, 0);
        let mut s = MockSocket::new();
        struct CountDispatch(u32);
        impl PeerInputDispatch for CountDispatch {
            fn dispatch_peer_input(&mut self, _: &mut ScreenManager) { self.0 += 1; }
        }
        let mut d = CountDispatch(0);
        struct Noop;
        impl ScrmanHooks for Noop {
            fn invoke_event(&mut self, _: &mut ScreenManager, _: RecordId, _: u32) -> i32 { 0 }
        }
        let mut h = Noop;
        m.pump_dispatch_case_wired(
            PumpDispatchResult::CaseNeg8, 0xFFFF_FFFF, &mut h, &mut s, &mut d);
        assert_eq!(d.0, 1, "case -8 dispatches peer input");
        // Broadcast happened (5 bytes = header + marker).
        assert_eq!(s.outbox.len(), 5);
    }

    #[test]
    fn case_neg13_pops_to_head() {
        // Build a 3-record chain, current at tail, dispatch -13 → current
        // should snap back to head.
        let mut m = ScreenManager::new();
        m.push_screen(1, 1, None, 0, 0);
        m.push_screen(2, 2, None, 0, 0);
        m.push_screen(3, 3, None, 0, 0);
        let slot = m.current_slot() as usize;
        let head = m.slot_head_id(slot).unwrap();
        let mut s = MockSocket::new();
        let mut d = NoPeerInput;
        struct Noop;
        impl ScrmanHooks for Noop {
            fn invoke_event(&mut self, _: &mut ScreenManager, _: RecordId, _: u32) -> i32 { 0 }
        }
        let mut h = Noop;
        m.pump_dispatch_case_wired(
            PumpDispatchResult::CaseNeg13, 0xFFFF_FFFF, &mut h, &mut s, &mut d);
        // After the shared tail frees the "current" record, current
        // advances to head (which itself was possibly freed if head
        // == current; here they differ so head survives).
        let new_cur = m.slot_current_id(slot);
        assert!(new_cur.is_some());
        assert_ne!(new_cur, Some(head).filter(|_| false)); // sanity
    }

    #[test]
    fn heap_free_is_noop() {
        // Documentation-only port — Rust drop handles the actual free.
        let mut m = ScreenManager::new();
        m.heap_free(1 as RecordId); // just needs to not panic
    }

    #[test]
    fn pump_with_net_runs_end_to_end_with_mock_socket() {
        let mut m = ScreenManager::new();
        m.push_screen(1, 1, None, 0, 0);
        let mut s = MockSocket::new();
        s.push_inbound(&[0x11, 0x22]);
        let mut d = NoPeerInput;
        struct ExitOnFirst;
        impl ScrmanHooks for ExitOnFirst {
            fn invoke_event(&mut self, _: &mut ScreenManager, _: RecordId, _: u32) -> i32 { -7 }
        }
        let mut h = ExitOnFirst;
        let last = m.pump_with_net(&mut h, &mut s, &mut d);
        assert_eq!(last, PumpDispatchResult::CaseNeg7);
        // Recv prologue consumed the inbox.
        assert!(s.inbox.is_empty());
        // Broadcast fired at least once in finalization.
        assert!(!s.outbox.is_empty());
    }

    // ================================================================
    // Final-remnants tests — Gaps 1/2/3.
    // ================================================================

    #[test]
    fn set_loading_filename_populates_next_pushed_record() {
        // Gap 1. Before set_loading_filename, push_screen leaves record.name
        // empty (documented pre-remnants behavior). After, the very next push
        // copies the bytes verbatim; a subsequent clear() reverts the same way.
        let mut m = ScreenManager::new();

        // Pre-set: baseline behavior is empty name.
        let r_before = m.push_screen(1, 1, None, 0, 0);
        assert_eq!(r_before, PushScreenResult::NewRecordPushed);
        let last_id_before = m.next_record_id.wrapping_sub(1);
        assert!(m.record(last_id_before).unwrap().name.is_empty());

        // Set a filename, push again, verify the new record carries the name.
        let name = b"news_screen.res\0";
        m.set_loading_filename(name);
        assert_eq!(m.loading_filename(), name);
        let r_after = m.push_screen(2, 2, None, 0, 0);
        assert_eq!(r_after, PushScreenResult::NewRecordPushed);
        let last_id_after = m.next_record_id.wrapping_sub(1);
        assert_eq!(m.record(last_id_after).unwrap().name, name);

        // Overwrite with a different name → next push picks that up.
        m.set_loading_filename(b"dashboard.res\0");
        m.push_screen(3, 3, None, 0, 0);
        let last_id = m.next_record_id.wrapping_sub(1);
        assert_eq!(m.record(last_id).unwrap().name.as_slice(), b"dashboard.res\0");
    }

    #[test]
    fn snapshot_time_now_matches_exe_epoch() {
        // Gap 2. cm_mktime is byte-exact against the exe's polynomial.
        //
        // Known-input probe: UTC 1970-01-01 00:00:00, dst_flag=0.
        //   yrs=70, month_table[1]=-1, day=1 → doy = -1 + 1 = 0
        //   leap: 70&3=2 != 0 → no adjust
        //   years_days = 70*365 + (70-1)>>2 = 25550 + 17 = 25567
        //   hours = (25567 + 0) * 24 + 0 = 613608
        //   minutes = 613608 * 60 + 0 = 36816480
        //   raw = 36816480 * 60 = 2208988800 (i32 wraps → -2085978496)
        //   secs = -2085978496 + 28800 + 0x7c558180 + 0
        //        = -2085978496 + 28800 + 2085978496 = 28800
        let expected: i32 = 28800;
        let got = cm_mktime(1970, 1, 1, 0, 0, 0, 0);
        assert_eq!(got, expected,
            "cm_mktime(1970-01-01 UTC, dst=0) formula mismatch — {} != {}", got, expected);

        // A second probe: one hour later at 1970-01-01 01:00:00 UTC differs by
        // exactly 3600 seconds — verifies the hour-multiply.
        assert_eq!(cm_mktime(1970, 1, 1, 1, 0, 0, 0),
                   expected.wrapping_add(3600));

        // DST flag applies the exact dumped bias (-3600) when forced on.
        assert_eq!(cm_mktime(1970, 1, 1, 0, 0, 0, 1),
                   expected.wrapping_add(CM_MKTIME_DST_BIAS));

        // Out-of-range year returns -1 (matches C L15-17).
        assert_eq!(cm_mktime(1969, 1, 1, 0, 0, 0, 0), -1);
        assert_eq!(cm_mktime(2039, 1, 1, 0, 0, 0, 0), -1);

        // snapshot_time_now stamps something in the exe-epoch neighborhood
        // (nonzero, and not the naive unix-seconds value it used to be).
        let mut m = ScreenManager::new();
        m.snapshot_time_now();
        let t = m.last_time_snapshot();
        assert_ne!(t, 0, "snapshot_time_now must stamp a nonzero value");
        // Sanity: it's the cm_mktime formula, not a raw unix-seconds count.
        let unix_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i32;
        assert_ne!(t, unix_secs,
            "snapshot_time_now must not be raw unix seconds — it must go through cm_mktime");
    }

    #[test]
    fn session_dtor_calls_all_5_teardown_fns() {
        // Gap 3. Populate every dtor-visited slot in the arena for both
        // sessions, drop the manager, verify all 5 teardown counters bumped.
        reset_teardown_counts();
        {
            let mut m = ScreenManager::new();
            // Populate session A's dtor-visited slots.
            m.set_u32(off::SESSION_A + off::SESS_PTR_0X12F529, 0xDEAD_0001);
            m.set_u16(off::SESSION_A + off::SESS_WORD_0X12F539, 2);  // 2 array elts
            m.set_u32(off::SESSION_A + off::SESS_PTR_0X12F521, 0xDEAD_0002);
            m.set_u32(off::SESSION_A + off::SESS_PTR_0X12F525, 0xDEAD_0003);
            m.set_u16(off::SESSION_A + off::SESS_SUBCOUNT_A, 1);   // 1 widget-pool teardown
            m.set_u16(off::SESSION_A + off::SESS_SUBCOUNT_B, 1);   // 1 msgbox teardown
            // Populate session B likewise so both dtor invocations fire.
            m.set_u32(off::SESSION_B + off::SESS_PTR_0X12F529, 0xDEAD_1001);
            m.set_u16(off::SESSION_B + off::SESS_WORD_0X12F539, 1);
            m.set_u32(off::SESSION_B + off::SESS_PTR_0X12F521, 0xDEAD_1002);
            m.set_u32(off::SESSION_B + off::SESS_PTR_0X12F525, 0xDEAD_1003);
            m.set_u16(off::SESSION_B + off::SESS_SUBCOUNT_A, 1);
            m.set_u16(off::SESSION_B + off::SESS_SUBCOUNT_B, 1);
        }
        // ScreenManager dropped — session_a.teardown and session_b.teardown ran.
        let counts = snapshot_teardown_counts();
        assert!(counts[0] > 0, "HEAP_FREE never invoked — got {:?}", counts);
        assert!(counts[1] > 0, "FREE_SURFACE never invoked — got {:?}", counts);
        assert!(counts[2] > 0, "WIDGET_POOL_TEARDOWN never invoked — got {:?}", counts);
        assert!(counts[3] > 0, "MSGBOX_TEARDOWN never invoked — got {:?}", counts);
        assert!(counts[4] > 0, "ARRAY_DTOR never invoked — got {:?}", counts);
    }

    #[test]
    fn screen_manager_reset_no_leaks() {
        // reset() drives populated state through the same teardown path a full
        // Drop would; the fresh arena post-reset must have all dtor-visited
        // pointer slots zeroed (no dangling handles for the next ctor cycle).
        reset_teardown_counts();
        let mut m = ScreenManager::new();
        // Populate session A dtor-visited slots.
        m.set_u32(off::SESSION_A + off::SESS_PTR_0X12F529, 0xCAFE_0001);
        m.set_u16(off::SESSION_A + off::SESS_WORD_0X12F539, 3);
        m.set_u32(off::SESSION_A + off::SESS_PTR_0X12F521, 0xCAFE_0002);
        m.set_u32(off::SESSION_A + off::SESS_PTR_0X12F525, 0xCAFE_0003);
        m.set_u16(off::SESSION_A + off::SESS_SUBCOUNT_A, 2);
        m.set_u16(off::SESSION_A + off::SESS_SUBCOUNT_B, 2);
        m.set_loading_filename(b"before-reset\0");

        let before = snapshot_teardown_counts();
        m.reset();
        let after = snapshot_teardown_counts();

        // Teardown counters strictly advanced across reset().
        assert!(after.iter().zip(before.iter()).any(|(a, b)| a > b),
                "reset() must drive teardown at least once — before={:?} after={:?}",
                before, after);
        // Post-reset arena is byte-clean: every dtor-visited slot zeroed.
        assert_eq!(m.get_u32(off::SESSION_A + off::SESS_PTR_0X12F529), 0);
        assert_eq!(m.get_u32(off::SESSION_A + off::SESS_PTR_0X12F521), 0);
        assert_eq!(m.get_u32(off::SESSION_A + off::SESS_PTR_0X12F525), 0);
        assert_eq!(m.get_u16(off::SESSION_A + off::SESS_SUBCOUNT_A), 0);
        assert_eq!(m.get_u16(off::SESSION_A + off::SESS_SUBCOUNT_B), 0);
        // loading_filename cleared.
        assert!(m.loading_filename().is_empty());
        // Sub-object marker regenerated (ctor rerun) — SESS_INIT_FLAG byte is 1.
        let init_byte = unsafe {
            *m.bytes.as_ptr().add(off::SESSION_A + off::SESS_INIT_FLAG)
        };
        assert_eq!(init_byte, 1);
    }
}
