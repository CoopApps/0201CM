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

/// A screen record — 0x300 bytes in the exe. Allocated by `push_screen`
/// (FUN_007e6570, ported in commit 5). Referenced by pump via slot's
/// `+0x28` entry pointer.
///
/// Known byte offsets from live-decode + inventory:
/// - `+0x00`  — vtable pointer (dereffed by pump)
/// - `+0x04`  — cleanup fn (dereffed at pump lines 179/277/417)
/// - `+0x0c`  — cleanup2 fn
/// - `+0x10`  — modal flag byte
/// - `+0x14 + slot_index*8` — slot bag values (from FUN_007e7130)
/// - `+0x1f8` — next-ptr (linked list)
/// - `+0x1fc` — prev-ptr
/// - `+0x7f`  — name string (~0x100 bytes)
///
/// The 4a commit adds only the type shell — no writers, no reads-through-vtable
/// yet. Fields past what commit 4c uses stay `unk_*` until push_screen decodes
/// the full 0x300-byte layout.
#[derive(Debug, Default)]
pub struct ScreenRecord {
    /// `+0x00` — the vtable (owned, since we've moved off raw pointers).
    pub vtable: ScreenRecordVTable,
    // TODO(commit 4c/5): fill remaining fields as they're decoded.
    // Placeholder to reserve struct identity; will be replaced with typed fields.
    pub unk_tail: (),
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

impl Drop for SessionSubObject {
    /// Port of `FUN_00548bd0`, restricted to the ctor-fresh case. The exe dtor's teardown
    /// is empty when `+0x12f521 == +0x12f525 == +0x12f529 == 0` and the two sub-counts at
    /// `+0x12e99e`/`+0x12e9a0` are 0 — all of which hold for a never-populated sub-object.
    /// Full port waits on the population fns (see struct doc).
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
}

// SAFETY: bytes are owned; no interior aliasing while `&mut self` is held.
unsafe impl Send for ScreenManager {}

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
        };
        this.apply_ctor_writes();
        this
    }

    /// Port of `FUN_007e46a0`'s structure-clearing tail. Re-runs the ctor's write sequence in
    /// place; sub-object dtors follow the "empty on ctor-fresh state" contract each carries.
    pub fn reset(&mut self) {
        // First zero the entire backing store — the exe's dtor also calls the subobject dtors
        // which effectively zero their bookkeeping (session dtor is no-op on fresh state).
        unsafe {
            std::ptr::write_bytes(self.bytes.as_ptr(), 0, SCRMGR_SIZE);
        }
        // Network-buffer dtor-then-ctor: drop the old, allocate a fresh 50000-byte buffer.
        self.net_buf = NetworkBuffer::new(off::NET_BUF_DEFAULT_SIZE);
        // Session sub-object dtor-then-ctor: replace the tracker markers; arena bytes get
        // re-populated by apply_ctor_writes below.
        self.session_a = SessionSubObject::new(0);
        self.session_b = SessionSubObject::new(1);
        self.mode_table_snapshot = [0; 8];
        self.last_time_snapshot = 0;
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
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i32)  // wraps in 2038, same as exe's i32
            .unwrap_or(0);
        self.last_time_snapshot = secs;
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
        // No subobject dtors this commit (documented in inventory) — just free the arena.
        unsafe { dealloc(self.bytes.as_ptr(), Self::layout()); }
    }
}

impl Default for ScreenManager {
    fn default() -> Self { Self::new() }
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
}
