//! Palette / pixel-format globals read by the widget renderer
//! (`FUN_005d7aa0`, `FUN_005ce2d0`, and the wrapped-text branch of
//! `FUN_005d03a0`) — dumped verbatim from `cm0102_GDI.exe` with `pefile`.
//!
//! Every `DAT_*` VA below is relative to `ImageBase = 0x00400000`. Values
//! that fall inside the exe's `.data` BSS tail (raw size 0) are read back
//! as zero — those are runtime-initialised slots the loader zeroes before
//! the game reaches them. All six BSS-zero slots below stay at 0 on the
//! main-menu path (verified by the prior static dump); the widget
//! renderer's port therefore treats zero as their initial-state value.
//!
//! Regenerate with `dump_globals.py` if the binary changes — do NOT
//! hand-edit.

/// `DAT_009b88f8` — scalable-font flag. Read by `FUN_005d7aa0` block L to
/// decide between the traditional-font path (via `PixelFont`) and the
/// futuristic scalable-font branch (`FUN_0059b550`). Ships as `1` in
/// `cm0102_GDI.exe`, meaning "traditional-font path" is the default.
pub const DAT_009B88F8: u32 = 0x0000_0001;

/// `DAT_00acdec8` — colour-key replacement word. `FUN_005d7aa0` block A
/// swaps `widget.colour_a` for this value when `colour_a == DAT_00ad6b22`.
/// BSS-zero on shipped exe.
pub const DAT_00ACDEC8: u16 = 0x0000;

/// `DAT_00ad6b22` — colour-key match word. `FUN_005d7aa0` block A compares
/// `widget.colour_a` against this; equality triggers the colour-key swap
/// with `DAT_00acdec8`. BSS-zero on shipped exe.
pub const DAT_00AD6B22: u16 = 0x0000;

/// `DAT_00ad6b0c` — palette word blitted by the widget renderer's
/// stipple-block-I and block-J branches (drawn through `draw_stipple`
/// at the pattern's paint step). BSS-zero on shipped exe.
pub const DAT_00AD6B0C: u16 = 0x0000;

/// `DAT_00acdee4` — palette word used by block K (the elongated stipple).
/// BSS-zero on shipped exe.
pub const DAT_00ACDEE4: u16 = 0x0000;

/// `DAT_00acdeac` — surface pixel-format code. `FUN_005ce2d0` (colour
/// scale) and `FUN_005d03a0` (wrapped-text shadow sampling) dispatch on
/// this field of the pixel-format record: `0x7e0 == RGB565`, anything else
/// is RGB555. BSS-zero on shipped exe (i.e. the format record isn't
/// installed until the DDraw init writes it — the software-only paths
/// use the surface's own `green_mask` instead).
pub const DAT_00ACDEAC: u16 = 0x0000;

/// `DAT_00ad6b44` — colour-scale early-exit gate. `FUN_005ce2d0` returns 0
/// immediately when this is non-zero (skipping the scale). Also gates
/// FUN_005d03a0 wrapped-text and FUN_005ceaa0 glyph blit. BSS-zero on
/// shipped exe → the gate lets rendering proceed.
pub const DAT_00AD6B44: u32 = 0x0000_0000;

/// The default pixel-format descriptor pointed at by the exe's global
/// `DAT_00acde98`. `FUN_005ce2d0` (colour_scale) and other primitives
/// take a `fmt_ref` argument and fall back to `mov esi, 0xacde98` when
/// callers pass NULL (asm 005ce2ea..005ce2ec):
///     `test ebx, ebx; jne skip; mov esi, 0xacde98`.
///
/// The record's raw layout is 8 dwords, of which the renderer reads
/// three:
///     +0x10 red_mask, +0x14 green_mask, +0x18 blue_mask.
///
/// FUN_005cc4f0 (GDI init) writes `0x7c00, 0x03e0, 0x001f` at boot into
/// these slots — the RGB555 mask set. The exe's DDraw build overwrites
/// them at device-negotiation time; the GDI/software build keeps them.
/// So the "NULL fmt_ref" branch always resolves to RGB555 on the
/// software renderer we're porting.
///
/// `size_code` is `[fmt+0x14] == 0x7e0` in the exe's 555/565 pack
/// dispatch — a green-mask compare masquerading as a format tag. Kept
/// here as a distinct field so the branch is explicit in the port.
#[derive(Debug, Clone, Copy)]
pub struct PixelFormat {
    pub red_mask: u16,
    pub green_mask: u16,
    pub blue_mask: u16,
}

impl PixelFormat {
    /// Build a PixelFormat mirroring the surface's mask set — matches the
    /// exe's runtime behaviour where callers that own their surface pass
    /// its format descriptor as `fmt_ref`. (Widget-renderer block A
    /// passes NULL, which hits `DAT_00ACDE98` instead — see the static
    /// below.)
    #[inline]
    pub fn from_surface(s: &crate::packed::PackedSurface) -> Self {
        Self {
            red_mask: s.red_mask,
            green_mask: s.green_mask,
            blue_mask: s.blue_mask,
        }
    }
}

// ---------------------------------------------------------------------
// Widget-icon single-slot cache (FUN_005d7aa0 block D + FUN_005cdb50).
//
// The exe caches one icon at a time in a global slot:
//   * DAT_00acda6c  u16 hold-counter (NOT a valid flag — it counts
//                   how many widgets are currently referencing the
//                   cached bitmap; the miss path only replaces the
//                   cache when it reaches 0).
//   * DAT_00acda70  260-byte filename buffer of the cached icon.
//   * DAT_00acdb74  pointer to the cached IconBitmap.
//
// Semantics (asm 005d7b17..005d7bdd):
//   Hit  (filename matches): reuse cache, `inc DAT_00acda6c`.
//   Miss (filename differs):
//     1. `load_icon_bitmap(filename, NULL)` (block D always passes
//        NULL for cache_slot — asm 005d7b7f `push 0`).
//     2. Stash the returned bitmap in `widget.cached_text`.
//     3. If `DAT_00acda6c == 0` (cache free):
//          drop the old cached bitmap, copy filename into
//          DAT_00acda70, install the new bitmap in DAT_00acdb74,
//          set DAT_00acda6c = 1.
//        Else keep the widget's private bitmap, don't touch cache.
//
// The exe never DECREMENTS the counter (searched every text ref to
// 0xacda6c) — it's a one-shot install lock, not a refcount. Once set
// to 1 the cache never reinstalls again in the exe's lifetime. Our
// port matches that (add a `reset_widget_icon_cache()` helper for
// tests only).
// ---------------------------------------------------------------------

use crate::packed_icon_loader::IconBitmap;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Mutex;

/// `DAT_00acda6c` — widget-icon cache hold-counter.
/// BSS-zero on shipped exe → cache is free at startup.
pub static DAT_00ACDA6C: AtomicU16 = AtomicU16::new(0);

/// `DAT_00acda70` — 260-byte filename buffer of the cached icon.
/// BSS-zero on shipped exe.
pub static DAT_00ACDA70: Mutex<[u8; 260]> = Mutex::new([0u8; 260]);

/// `DAT_00acdb74` — pointer to the cached IconBitmap. BSS-zero.
/// The Rust port owns the bitmap directly (rather than pointer +
/// external allocation).
pub static DAT_00ACDB74: Mutex<Option<IconBitmap>> = Mutex::new(None);

/// Test-only: reset the widget-icon cache slot so tests don't leak
/// state across test-runner threads.
#[cfg(test)]
pub fn reset_widget_icon_cache() {
    DAT_00ACDA6C.store(0, Ordering::SeqCst);
    *DAT_00ACDA70.lock().unwrap() = [0u8; 260];
    *DAT_00ACDB74.lock().unwrap() = None;
}

#[cfg(not(test))]
#[allow(dead_code)]
fn _unused_ordering() { let _ = Ordering::SeqCst; }

/// Bytes of `DAT_00acde98` after `FUN_005cc4f0` (GDI init) has run —
/// RGB555 masks. Every primitive that reads `[0xacde98+0x10..+0x18]` on
/// the NULL-fmt_ref branch sees these three words.
pub const DAT_00ACDE98: PixelFormat = PixelFormat {
    red_mask: 0x7c00,
    green_mask: 0x03e0,
    blue_mask: 0x001f,
};
