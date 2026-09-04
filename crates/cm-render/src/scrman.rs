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

    // ---- 0x3070..0x1326d0: first session sub-object (opaque, ctor arg = 0) ----
    /// `+0x3070`  first session sub-object base.  GDI L22 `lea ecx, [esi + 0x3070]`.
    pub const SESSION_A: usize = 0x3070;

    // ---- 0x1326d1..0x261fd3: second session sub-object (opaque, ctor arg = 1) ----
    /// `+0x1326d1`  second session sub-object base.  GDI L27 `lea ecx, [esi + 0x1326d1]`.
    pub const SESSION_B: usize = 0x1_326d1;

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

/// The ScreenManager class. All field access byte-cited to the exe.
pub struct ScreenManager {
    /// Heap-owned backing store; layout matches the exe byte-for-byte.
    /// Boxed to avoid a ~2.4 MB stack blowup on `new()`.
    bytes: NonNull<u8>,
    /// Sub-object owner for `+0x302a`. Byte-slots at `+0x302e` / `+0x3032` in `bytes`
    /// mirror this struct's `write_off` / `size`; `+0x302a` (buf-ptr) stays 0 there
    /// (host pointer widths differ from the 32-bit exe) — reads go through `net_buf()`.
    net_buf: NetworkBuffer,
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
    /// External sub-object ctors invoked: `FUN_00763590(this+0x302a, 50000)` (network buffer,
    /// this commit). Session sub-object ctors `FUN_00548b40 x2` at `+0x3070` / `+0x1326d1`
    /// remain documented-TODO (follow-up commit); their memory ranges stay zero.
    pub fn new() -> Self {
        // GDI L21-25: implicit `alloc_zeroed` (the class's operator new zeroes memory before
        // the ctor runs when it's part of a larger allocation; we replicate explicitly).
        let bytes = unsafe {
            let p = alloc_zeroed(Self::layout());
            NonNull::new(p).expect("scrmgr alloc")
        };
        // Network-buffer sub-object ctor call (matches `FUN_00763590(esi+0x302a, 50000)`).
        let net_buf = NetworkBuffer::new(off::NET_BUF_DEFAULT_SIZE);
        let mut this = ScreenManager { bytes, net_buf };
        this.apply_ctor_writes();
        this
    }

    /// Port of `FUN_007e46a0`'s structure-clearing tail. Re-runs the ctor's write sequence in
    /// place; safe because the exe's destructor is a "tear-down subobjects then zero header"
    /// sequence, and re-initializing subobjects is out of this commit's scope.
    pub fn reset(&mut self) {
        // First zero the entire backing store — the exe's dtor also calls the subobject dtors
        // which effectively zero their bookkeeping. Session sub-object dtors are still TODO.
        unsafe {
            std::ptr::write_bytes(self.bytes.as_ptr(), 0, SCRMGR_SIZE);
        }
        // Network-buffer dtor-then-ctor: drop the old, allocate a fresh 50000-byte buffer.
        self.net_buf = NetworkBuffer::new(off::NET_BUF_DEFAULT_SIZE);
        self.apply_ctor_writes();
    }

    /// Immutable view of the network-buffer sub-object at `+0x302a`.
    #[inline]
    pub fn net_buf(&self) -> &NetworkBuffer { &self.net_buf }
    /// Mutable view for follow-up commits porting net-send/recv.
    #[inline]
    pub fn net_buf_mut(&mut self) -> &mut NetworkBuffer { &mut self.net_buf }

    /// The exact write sequence of `sub_007e3f20`. Each line cites the GDI asm address.
    fn apply_ctor_writes(&mut self) {
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
        self.set_u32(off::INITIAL_SCREEN_PTR, off::SESSION_A as u32); // marker until commit 3

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
        // Highest ctor write is [+0x261fd0] dword, ends at 0x261fd4.
        assert!(SCRMGR_SIZE >= 0x0026_1fd4);
    }
}
