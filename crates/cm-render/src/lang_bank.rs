//! Localisation bank lookup — direct port of `FUN_00654E90`
//! (14,645 bytes decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_glyph_blit/0x00654e90.c`.
//! Full decode: [`reports/gui_glyph_blit_decode.md`](../../../../reports/gui_glyph_blit_decode.md) §lookup_string.
//!
//! # Two paths
//!
//! * **Fast** (`DAT_00B4C658 == 1`) — production shipped this: an
//!   in-memory table of 52-byte `LangSlot` entries, sorted by
//!   normalised source string, resolved via `FUN_009354F4` bsearch.
//! * **Slow** — fopen a hard-coded `C:\dev\CM3 00-01\si\code\Langlib`
//!   path, linear-scan records. Only used when the fast path table
//!   isn't populated; production ships without this dev path active.
//!
//! # Normalisation
//!
//! Both paths first normalise `raw_src`:
//! * Strip `{`, `}`, and space (fast path only; slow path keeps `{}`)
//! * `\n` (backslash-n) → `0x0A`
//! * `\"` → `0x22`
//! * Trim trailing spaces

/// One entry in the fast-path bank. Matches the exe's 52-byte layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangSlot {
    /// `+0x00` — original id / template source (the lookup key).
    pub id_original: String,
    /// `+0x04` — key or context annotation.
    pub key_or_context: String,
    /// `+0x08` — translated text (contains `\x08` reorder markers).
    pub translated_text: String,
    /// `+0x0C` — 10 × i32 payload (format/attr fields).
    pub payload: [i32; 10],
}

impl Default for LangSlot {
    fn default() -> Self {
        Self { id_original: String::new(), key_or_context: String::new(),
               translated_text: String::new(), payload: [0; 10] }
    }
}

/// The complete bank — a sorted vector of slots + a flag matching the
/// exe's `DAT_00B4C658` "fast path active" bit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LangBank {
    /// Sorted by `id_original` (normalised) so binary search resolves.
    pub slots: Vec<LangSlot>,
    /// exe: `DAT_00B4C658`. When true, `lookup` uses bsearch;
    /// otherwise it returns `None` (the exe would fall back to disk).
    pub fast_path_active: bool,
}

/// Normalise a raw source string — port of the `abStack_65e` walker
/// inside `FUN_00654E90`. Fast-path variant (strips `{`, `}`, spaces).
pub fn normalise_fast(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'{' || b == b'}' || b == b' ' { i += 1; continue; }
        if b == b'\\' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            if next == b'n' { out.push(0x0A as char); i += 2; continue; }
            if next == b'"' { out.push(0x22 as char); i += 2; continue; }
        }
        out.push(b as char);
        i += 1;
    }
    // Trim trailing spaces (already stripped above, but keep for parity).
    while out.ends_with(' ') { out.pop(); }
    out
}

/// Normalise for the slow path — same rules but KEEPS `{`, `}`.
pub fn normalise_slow(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b' ' { i += 1; continue; }
        if b == b'\\' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            if next == b'n' { out.push(0x0A as char); i += 2; continue; }
            if next == b'"' { out.push(0x22 as char); i += 2; continue; }
        }
        out.push(b as char);
        i += 1;
    }
    while out.ends_with(' ') { out.pop(); }
    out
}

impl LangBank {
    /// Build a new bank from a Vec of slots. Automatically sorts by
    /// `id_original` so `lookup` can bsearch, and marks the fast path
    /// active.
    pub fn new(mut slots: Vec<LangSlot>) -> Self {
        slots.sort_by(|a, b| a.id_original.cmp(&b.id_original));
        Self { slots, fast_path_active: true }
    }

    /// Direct port of `FUN_00654E90(&out, raw_src, &index)` fast path.
    ///
    /// Returns `Some((slot, index))` on hit, `None` on miss (or when
    /// the fast-path table is not populated — matching the exe's early
    /// return without falling through to disk).
    pub fn lookup(&self, raw_src: &str) -> Option<(&LangSlot, usize)> {
        if !self.fast_path_active { return None; }
        let key = normalise_fast(raw_src);
        match self.slots.binary_search_by(|s| s.id_original.cmp(&key)) {
            Ok(idx) => Some((&self.slots[idx], idx)),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalise_fast_strips_braces_and_spaces() {
        // Exe strips ONLY the brace chars (and all spaces), keeps the
        // content between them.
        assert_eq!(normalise_fast("Hello {name} !"), "Helloname!");
        assert_eq!(normalise_fast("  a b c  "), "abc");
    }

    #[test]
    fn normalise_fast_escapes_backslash_n_and_quote() {
        assert_eq!(normalise_fast("a\\nb"), format!("a{}b", 0x0A as char));
        assert_eq!(normalise_fast("q\\\"q"), format!("q{}q", 0x22 as char));
    }

    #[test]
    fn normalise_slow_keeps_braces() {
        assert_eq!(normalise_slow("Hello {name}"), "Hello{name}");
    }

    #[test]
    fn empty_bank_returns_none() {
        let b = LangBank::default();
        assert_eq!(b.lookup("anything"), None);
    }

    #[test]
    fn lookup_hits_on_normalised_key() {
        let bank = LangBank::new(vec![
            LangSlot { id_original: "hello".to_string(),
                       translated_text: "bonjour".to_string(),
                       ..Default::default() },
            LangSlot { id_original: "world".to_string(),
                       translated_text: "monde".to_string(),
                       ..Default::default() },
        ]);
        let (slot, _) = bank.lookup("{hello}").unwrap();
        assert_eq!(slot.translated_text, "bonjour");
        // Braces + spaces stripped.
        let (slot, _) = bank.lookup("wor ld").unwrap();
        assert_eq!(slot.translated_text, "monde");
    }

    #[test]
    fn lookup_returns_stable_index() {
        let bank = LangBank::new(vec![
            LangSlot { id_original: "a".to_string(), ..Default::default() },
            LangSlot { id_original: "b".to_string(), ..Default::default() },
            LangSlot { id_original: "c".to_string(), ..Default::default() },
        ]);
        assert_eq!(bank.lookup("a").unwrap().1, 0);
        assert_eq!(bank.lookup("b").unwrap().1, 1);
        assert_eq!(bank.lookup("c").unwrap().1, 2);
    }

    #[test]
    fn lookup_miss_returns_none() {
        let bank = LangBank::new(vec![
            LangSlot { id_original: "hello".to_string(), ..Default::default() },
        ]);
        assert!(bank.lookup("missing").is_none());
    }

    #[test]
    fn fast_path_inactive_flag_disables_lookup() {
        let mut bank = LangBank::new(vec![
            LangSlot { id_original: "hello".to_string(), ..Default::default() },
        ]);
        bank.fast_path_active = false;
        assert!(bank.lookup("hello").is_none());
    }

    #[test]
    fn slots_get_sorted_on_construction() {
        let bank = LangBank::new(vec![
            LangSlot { id_original: "c".to_string(), ..Default::default() },
            LangSlot { id_original: "a".to_string(), ..Default::default() },
            LangSlot { id_original: "b".to_string(), ..Default::default() },
        ]);
        assert_eq!(bank.slots[0].id_original, "a");
        assert_eq!(bank.slots[1].id_original, "b");
        assert_eq!(bank.slots[2].id_original, "c");
    }
}
