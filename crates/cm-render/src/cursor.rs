//! Cursor state port — direct port of `FUN_005D1B10` (set mode) and
//! `FUN_005D1B80` (refresh) from cm0102.exe.
//!
//! Decompiles: `d:/cm0102-carve/decompiled/gui_layout_engine/0x005d1b10.c`
//! and `0x005d1b80.c`. Both are tiny (< 150 bytes each) so the port is
//! trivially 1:1 with the exe's behaviour.
//!
//! # State globals
//!
//! | exe DAT_       | meaning                     |
//! |----------------|-----------------------------|
//! | `DAT_009B9D54` | current cursor mode (0/1/2) |
//! | `DAT_00AD6C14` | HCURSOR for mode 0 (default arrow) |
//! | `DAT_00AD6C0C` | HCURSOR for mode 1 (hand / clickable) |
//! | `DAT_00AD6C10` | HCURSOR for mode 2 (busy / wait) |
//!
//! In the Rust port we model modes as an enum instead of raw HCURSOR
//! pointers; the host renderer picks the actual glyph.

/// The three cursor modes cm0102 uses. Mapping matches the exe:
/// 0 → Default, 1 → Hand, 2 → Busy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMode {
    /// Mode 0 — default arrow. `DAT_00AD6C14`.
    Default = 0,
    /// Mode 1 — hand / clickable. `DAT_00AD6C0C`.
    Hand = 1,
    /// Mode 2 — busy / wait. `DAT_00AD6C10`.
    Busy = 2,
}

impl CursorMode {
    pub fn from_exe_byte(b: i32) -> Self {
        match b { 1 => Self::Hand, 2 => Self::Busy, _ => Self::Default }
    }
    pub fn as_exe_byte(self) -> i32 { self as i32 }
}

/// The single global cursor-state variable. Port of `DAT_009B9D54`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// GDI-REG: 005d1b10 PORTED_BEHAVIOURAL
pub struct CursorState {
    pub mode: CursorMode,
}

impl Default for CursorState {
    fn default() -> Self { Self { mode: CursorMode::Default } }
}

impl CursorState {
    /// Direct port of `FUN_005D1B10(mode)`.
    ///
    /// ```pseudo
    /// if (DAT_009B9D54 != param_1) {
    ///     DAT_009B9D54 = param_1;
    ///     ShowCursor(0);
    ///     if (param_1 == 0)      SetCursor(DAT_00AD6C14);
    ///     else if (param_1 == 1) { SetCursor(DAT_00AD6C0C); ShowCursor(1); return; }
    ///     else if (param_1 == 2) { SetCursor(DAT_00AD6C10); ShowCursor(1); return; }
    ///     ShowCursor(1);
    /// }
    /// ```
    ///
    /// The behaviour is idempotent — if `mode` already matches, nothing
    /// happens (matches the `if (DAT_009B9D54 != param_1)` gate). Any
    /// host renderer sees the return value: `true` if the cursor was
    /// swapped, `false` if this was a no-op.
    pub fn set_mode(&mut self, mode: CursorMode) -> bool {
        if self.mode == mode { return false; }
        self.mode = mode;
        true
    }

    /// Direct port of `FUN_005D1B80` — the refresh helper. Ensures the
    /// system cursor matches `self.mode`, re-issuing the SetCursor call
    /// even if the mode value is the same but the OS forgot (e.g. after
    /// a mode switch or Alt-Tab). No-op in our Rust port because we
    /// don't own the OS cursor state — but the exe unconditionally
    /// re-issues `ShowCursor(0); SetCursor(H); ShowCursor(1)` when the
    /// current HCURSOR differs from the mode's registered handle.
    ///
    /// The Rust equivalent is: renderer redraws the cursor glyph.
    /// Returns `true` if a redraw is warranted.
    pub fn refresh(&self) -> bool {
        // In the exe: `if (GetCursor() != DAT_00AD6C1x)` triggers the
        // redraw. In Rust we assume the renderer handles this and just
        // signal that a redraw MAY be needed.
        true
    }
}

// ============================================================================
// Port of FUN_005CDFA0 — texture / linked-resource free helper.
//
// Decompile: `d:/cm0102-carve/decompiled/gui_layout_engine/0x005cdfa0.c`.
//
// ```c
// void FUN_005cdfa0(int param_1) {
//     if (param_1 != 0) {
//         if (*(int *)(param_1 + 0xc) != 0) {
//             FUN_0093435a(*(int *)(param_1 + 0xc));
//             *(undefined4 *)(param_1 + 0xc) = 0;
//         }
//         FUN_0093435a(param_1);
//     }
// }
// ```
//
// `FUN_0093435A` is the CRT allocator `delete[]`. The struct has an
// owned sub-resource at `+0xC` (typically a linked texture handle); it
// is freed first, then the outer object.
// ============================================================================

/// Port of `FUN_005CDFA0(resource)`. In Rust the equivalent is `Drop`
/// on the resource type. This function is exposed only because the exe
/// calls it explicitly from many teardown paths — we mirror the shape.
pub fn free_texture_resource(resource: Option<Box<TextureResource>>) {
    // Rust's `Box` Drop covers everything the C code does: the linked
    // sub-resource at +0xC is a nested Option<Box<_>> that drops
    // recursively. Retained as a named helper so port sites read like
    // the original C.
    drop(resource);
}

/// Layout mirror of the 16-byte record the exe frees at +0xC. Only the
/// fields the free path touches are named; the rest of the record's
/// bytes are opaque padding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureResource {
    /// `+0x00..+0x0B` — opaque header (owner ptr + type tag).
    pub header: [u8; 12],
    /// `+0x0C` — nested texture/pixel buffer. The exe frees this
    /// separately before the outer object.
    pub inner_buffer: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_mode_returns_true_only_on_change() {
        let mut s = CursorState::default();
        assert_eq!(s.mode, CursorMode::Default);
        assert!(!s.set_mode(CursorMode::Default), "no-op on same mode");
        assert!(s.set_mode(CursorMode::Hand));
        assert_eq!(s.mode, CursorMode::Hand);
        assert!(!s.set_mode(CursorMode::Hand), "second call is idempotent");
    }

    #[test]
    fn exe_byte_round_trips() {
        for m in [CursorMode::Default, CursorMode::Hand, CursorMode::Busy] {
            assert_eq!(CursorMode::from_exe_byte(m.as_exe_byte()), m);
        }
    }

    #[test]
    fn unknown_exe_byte_defaults_to_arrow() {
        // Exe: any value other than 1/2 leaves the else branch that
        // falls through to ShowCursor(1) with no SetCursor — effectively
        // "keep the default arrow visible".
        assert_eq!(CursorMode::from_exe_byte(7), CursorMode::Default);
    }

    #[test]
    fn free_texture_resource_matches_exe_shape() {
        // The exe pattern: if resource->inner is Some, free it, then free
        // the outer. Rust's Drop handles both. Sanity-check that a null
        // resource is a no-op (matches `if (param_1 != 0)` gate).
        free_texture_resource(None);
        let r = TextureResource {
            header: [0; 12],
            inner_buffer: Some(vec![1, 2, 3]),
        };
        free_texture_resource(Some(Box::new(r)));
    }
}
