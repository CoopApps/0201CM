//! C15.1E — persistent per-person news mailbox pool.
//!
//! Runtime object in the exe: a per-person mailbox descriptor
//! table at `DAT_00ACD5C4` (stride `0x6E` per person) whose
//! `+0xCF` field points into a shared news-item pool. The pool
//! stores 222-byte news items (stride `0xDF`) organised as
//! per-DOB-age-bucket ring buffers of 100 entries each, growing
//! linearly by 100 slots at a time via `FUN_009346F7` (realloc).
//!
//! ## What lives in this module
//!
//! * [`PersonNewsItem`] — one news-item record with the eight
//!   parameter slots the exe's `FUN_0076D730(item, idx, val)`
//!   populates, plus the season stamp and monotonic news id.
//! * [`PersonNewsMailboxPool`] — the aggregate, keyed by
//!   `person_id`. Append-only from Rust's point of view.
//!
//! ## What this port INTENTIONALLY does not model
//!
//! * DOB-age bucket routing. The exe indexes the pool by
//!   `(person.dob - DAT_00ACD56C) + 0x10`; Rust does not model
//!   DOB at simulation granularity, so keying is directly by
//!   `person_id`. The exe's per-person mailbox descriptor at
//!   `DAT_00ACD5C4 + person_id * 0x6E` is what a downstream
//!   reader hits first anyway — the age bucket is a
//!   cross-person index the port sidesteps.
//! * 100-entry ring overwrite. Rust uses an append-only `Vec`.
//!   The exe's ring behaviour would drop the oldest entry after
//!   100 items in the same age bucket. That failure mode does
//!   not affect the mutation semantics tested here (rollover
//!   emits at most one entry per person per season) and is
//!   documented as a deferred deviation.
//! * Stride-`0xDF` on-disk footprint. The exe reserves 223
//!   bytes per slot for future template growth; the Rust
//!   record only carries the fields actually populated.
//! * Numbered param slots 1..=4 (secondary/reserve/nation
//!   pointer chains) are left zero — the C13/C15 pipeline in
//!   Rust does not currently thread those pointer-derived ids.
//!   Slots 0/5/6/7 (person_id, old_comp_id, kind, staff_id) are
//!   what the two known callers of `FUN_008D0D90` actually
//!   consume via downstream templates, and are populated
//!   faithfully.
//!
//! See `reports/c15_1e_history_archaeology.md` for the exe
//! archaeology and the refutation of the "13-list-per-person"
//! model.

use serde::{Deserialize, Serialize};

/// One entry in a person's news mailbox — the Rust image of the
/// 222-byte news item that `FUN_008D0D90` builds and hands to
/// `FUN_0076E180` / `FUN_0076DCE0` for persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonNewsItem {
    /// News category id (`FUN_00763B90` `param_2`). For the two
    /// known callers of `FUN_008D0D90` (relegation `kind=3` and
    /// retirement `kind=0`) this is always `0xFBF`.
    pub category: u32,
    /// Per-item severity byte at record `+0x04` (initially 0;
    /// the exe overwrites in `FUN_0076CE50` per person). Not
    /// modelled here — held at 0 to keep the record
    /// reader-invariant for downstream template code.
    pub severity: u8,
    /// The eight parameter slots the exe populates via
    /// `FUN_0076D730(item, idx, val)` at offsets `+0x05 + idx*4`.
    /// Slot semantics for the relegation caller
    /// (`FUN_004D3460 → FUN_008D0D90(slot, old_comp, 3, contract_row)`):
    ///
    /// | Slot | Semantic |
    /// | ---- | -------- |
    /// | 0 | Subject person id (`*param_1`)                |
    /// | 1 | Secondary A (`*(u32*)(param_1[1] + 0x33)`)    |
    /// | 2 | Secondary B (`*(u32*)(param_1[2] + 0x33)`)    |
    /// | 3 | Secondary C (`*(u32*)(param_1[3] + 0x33)`)    |
    /// | 4 | Club/nation base id                           |
    /// | 5 | `old_comp_id` (`*param_2`)                    |
    /// | 6 | `kind` (`param_3`; 3 = relegated, 0 = retire) |
    /// | 7 | Staff row id (`*(u32*)(param_4 + 0x21)`)      |
    ///
    /// Slots 1..=4 are held at 0 in this port (see module
    /// deviations).
    pub params: [u32; 8],
    /// Season byte at record `+0xD5` — the exe writes
    /// `DAT_00ACDE88` (game-year low byte). Rust stores the
    /// full year the applier was called with.
    pub year: u16,
    /// Monotonic news-item id assigned by the pool at insert
    /// time. Matches the exe's `param_1_ctx[4]++` at
    /// `FUN_0076DCE0`.
    pub news_id: u32,
}

impl PersonNewsItem {
    /// Convenience readers so tests don't have to remember which
    /// param slot each field lives in.
    pub fn person_id(&self) -> u32 { self.params[0] }
    pub fn old_comp_id(&self) -> u32 { self.params[5] }
    pub fn kind(&self) -> u8 { self.params[6] as u8 }
    pub fn staff_id(&self) -> u32 { self.params[7] }
}

/// The persistent per-person mailbox pool.
///
/// Keyed by `person_id` — matches the exe's mailbox descriptor
/// table indexing (`DAT_00ACD5C4 + person_id * 0x6E`). Each
/// person's mailbox is an append-only `Vec` of
/// [`PersonNewsItem`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonNewsMailboxPool {
    /// Per-person mailbox contents, in insertion order.
    #[serde(default)]
    pub by_person: std::collections::BTreeMap<u32, Vec<PersonNewsItem>>,
    /// Monotonic news-item id counter. Mirrors the exe's
    /// `param_1_ctx[4]` on `FUN_0076DCE0` — one shared counter
    /// across all mailboxes so ids are globally unique.
    #[serde(default)]
    pub next_news_id: u32,
}

impl PersonNewsMailboxPool {
    /// Append one news item to `person_id`'s mailbox. Assigns
    /// the next monotonic `news_id` and returns the mailbox's
    /// pre-append length (so callers can build a trace entry
    /// without a re-lookup).
    ///
    /// Never fails. The exe's ring-overflow / realloc failure
    /// paths are not modelled — see module deviations.
    pub fn push(&mut self, person_id: u32, mut item: PersonNewsItem)
        -> AppendReceipt
    {
        item.news_id = self.next_news_id;
        self.next_news_id = self.next_news_id.wrapping_add(1);
        let mb = self.by_person.entry(person_id).or_default();
        let old_len = mb.len();
        mb.push(item);
        AppendReceipt {
            person_id,
            old_len,
            new_len: mb.len(),
        }
    }

    /// Read-only accessor for tests.
    pub fn mailbox_for(&self, person_id: u32) -> &[PersonNewsItem] {
        self.by_person.get(&person_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Receipt returned by [`PersonNewsMailboxPool::push`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendReceipt {
    pub person_id: u32,
    pub old_len: usize,
    pub new_len: usize,
}

/// News category constant `0xFBF`. Both known callers of
/// `FUN_008D0D90` use this category — the differentiation
/// between relegation and retirement lives in the `kind` param
/// slot, not in a distinct category id.
pub const NEWS_CATEGORY_PERSON_CAREER: u32 = 0x0FBF;

/// `kind` values seen at `FUN_008D0D90` callers.
pub mod kinds {
    /// `FUN_004D3300:48` — retirement path.
    pub const RETIREMENT: u8 = 0;
    /// `FUN_004D3460:35` — relegation path.
    pub const RELEGATED: u8 = 3;
}
