//! Opaque world facade shared by the `populate_from_world` companions
//! attached to ported screen-builder views (see `screen_batch{18..27}`).
//!
//! Every `build_*` in the screen batches takes concrete primitive
//! parameters that mirror the exe's raw pointer / slot reads. The
//! `populate_*` companions look those values up on this facade so the
//! rest of the game (dashboard driver, tests, replay harness) can
//! construct views without knowing which slot goes where.
//!
//! Fields the facade doesn't know about resolve to sensible defaults
//! — `handle()` returns 0, `flag()` returns false, `entries()` returns
//! the empty 50-slot table — which matches the exe's behaviour when its
//! own pool slot is null. Individual populator docs document what they
//! read; anything they cannot recover from the facade is recorded in
//! the per-batch `TODO_POPULATOR_INFO` const.
//!
//! This shim is the minimum viable facade — it is not the wave-B
//! typed-pool facade (which does not yet exist under
//! `crates/cm-domain/src/world_facade.rs`). When that lands, the
//! populators here should switch to typed getters and drop the string
//! lookup.

use std::collections::HashMap;

use crate::screen_batch18::{FlagEntry, SeatEntry};

/// Opaque handle / flag / byte-pool facade. Empty by default.
#[derive(Debug, Clone)]
pub struct WorldFacade {
    /// Mirrors `FUN_007E6570 != 0` for the current screen registration.
    /// Defaults to `false` so tests must opt in via [`WorldFacade::ready`].
    pub registration_ok: bool,
    /// Named opaque u32 handles (record ptrs, DAT-pool bases, slots).
    pub handles: HashMap<String, u32>,
    /// Named boolean flags (loader OK, guard blocks, etc.).
    pub flags: HashMap<String, bool>,
    /// Named u8-typed slots (record byte fields, i8 params).
    pub bytes: HashMap<String, i32>,
    /// Named raw byte buffers (record slices copied by the exe).
    pub buffers: HashMap<String, Vec<u8>>,
    /// Named UTF-8 strings (labels, article text, dialog messages).
    pub strings: HashMap<String, String>,
    /// The 50-slot flag-entry table used by `TwoSelectorListView`.
    pub flag_entries: [FlagEntry; 50],
    /// Active-human seat (used by `SeatScreenView`).
    pub seat_entry: SeatEntry,
}

impl Default for WorldFacade {
    fn default() -> Self {
        Self {
            registration_ok: false,
            handles: HashMap::new(),
            flags: HashMap::new(),
            bytes: HashMap::new(),
            buffers: HashMap::new(),
            strings: HashMap::new(),
            flag_entries: [FlagEntry::default(); 50],
            seat_entry: SeatEntry::default(),
        }
    }
}

impl WorldFacade {
    /// Facade with `registration_ok=true` and everything else empty.
    pub fn ready() -> Self {
        Self { registration_ok: true, ..Self::default() }
    }
    /// Read a handle by name; 0 when absent.
    pub fn handle(&self, key: &str) -> u32 {
        self.handles.get(key).copied().unwrap_or(0)
    }
    /// Read a flag by name; false when absent.
    pub fn flag(&self, key: &str) -> bool {
        self.flags.get(key).copied().unwrap_or(false)
    }
    /// Read an i32-widened byte slot; 0 when absent.
    pub fn byte(&self, key: &str) -> i32 {
        self.bytes.get(key).copied().unwrap_or(0)
    }
    /// Read a raw byte buffer; empty when absent.
    pub fn buffer(&self, key: &str) -> &[u8] {
        self.buffers.get(key).map(|v| v.as_slice()).unwrap_or(&[])
    }
    /// Read a string by name; `None` when absent.
    pub fn text(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(|s| s.as_str())
    }
    /// Fluent setter for a string.
    pub fn with_text(mut self, key: &str, value: impl Into<String>) -> Self {
        self.strings.insert(key.to_string(), value.into()); self
    }
    /// Fluent setter for a handle.
    pub fn with_handle(mut self, key: &str, value: u32) -> Self {
        self.handles.insert(key.to_string(), value); self
    }
    /// Fluent setter for a flag.
    pub fn with_flag(mut self, key: &str, value: bool) -> Self {
        self.flags.insert(key.to_string(), value); self
    }
    /// Fluent setter for a byte / i8-widened slot.
    pub fn with_byte(mut self, key: &str, value: i32) -> Self {
        self.bytes.insert(key.to_string(), value); self
    }
    /// Fluent setter for a raw byte buffer.
    pub fn with_buffer(mut self, key: &str, value: Vec<u8>) -> Self {
        self.buffers.insert(key.to_string(), value); self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_empty_and_not_ready() {
        let w = WorldFacade::default();
        assert!(!w.registration_ok);
        assert_eq!(w.handle("nope"), 0);
        assert!(!w.flag("nope"));
        assert_eq!(w.byte("nope"), 0);
        assert!(w.buffer("nope").is_empty());
    }
    #[test]
    fn ready_flips_registration() {
        assert!(WorldFacade::ready().registration_ok);
    }
    #[test]
    fn fluent_setters_roundtrip() {
        let w = WorldFacade::ready()
            .with_handle("h", 42)
            .with_flag("f", true)
            .with_byte("b", -7)
            .with_buffer("buf", vec![1, 2, 3]);
        assert_eq!(w.handle("h"), 42);
        assert!(w.flag("f"));
        assert_eq!(w.byte("b"), -7);
        assert_eq!(w.buffer("buf"), &[1, 2, 3]);
    }
}
