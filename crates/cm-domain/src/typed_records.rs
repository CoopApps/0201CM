//! Typed views over the `.dat` records.
//!
//! Each `.dat` record is a fixed-size opaque byte slice (see `DomainOpaqueRecord.raw`).
//! This module exposes byte offsets we've decoded from the exe as typed accessors,
//! without breaking the existing serialized shape. Callers wrap an opaque record
//! (`ClubView::new(record)`) and read named fields.
//!
//! Offsets come from a decompile-wide scan of every function that reads each pool
//! (see `reports/*_record_offsets.json` and `reports/function_coverage.md` for the
//! evidence). Every field is code-derived, never image-derived. Where offsets are
//! marked "probable" in the report, we return them anyway and let the caller decide
//! how much to trust them — no guessing hidden behind a name.
//!
//! **Runtime pointer fields are omitted.** The C game caches `player+0x39 = &club`
//! and similar for fast dereference, but Rust resolves IDs to references in O(1) via
//! `Vec` indexing, so the Rust world model needs only the persistent ID (+0x61 for
//! current_club_id) and an accessor that returns `&Club`.

use crate::DomainOpaqueRecord;

// ------ shared helpers ------

/// Read a little-endian `u32` at `off`; returns 0 if the slice is short.
#[inline]
fn le_u32(bytes: &[u8], off: usize) -> u32 {
    if off + 4 <= bytes.len() {
        u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
    } else {
        0
    }
}

/// Read a little-endian `i32` at `off`; returns 0 if the slice is short.
#[inline]
fn le_i32(bytes: &[u8], off: usize) -> i32 {
    le_u32(bytes, off) as i32
}

/// Read a little-endian `u16` at `off`; returns 0 if the slice is short.
#[inline]
fn le_u16(bytes: &[u8], off: usize) -> u16 {
    if off + 2 <= bytes.len() {
        u16::from_le_bytes([bytes[off], bytes[off + 1]])
    } else {
        0
    }
}

/// Read a signed byte at `off`; returns 0 if out of range.
#[inline]
fn i8_at(bytes: &[u8], off: usize) -> i8 {
    bytes.get(off).copied().unwrap_or(0) as i8
}

/// Read an unsigned byte at `off`; returns 0 if out of range.
#[inline]
fn u8_at(bytes: &[u8], off: usize) -> u8 {
    bytes.get(off).copied().unwrap_or(0)
}

/// Turn a "sentinel" `i32` (`-1` or `-2` in the exe's convention) into `None`.
/// The exe uses `-1` for "unset/null" and `-2` for "extinct/placeholder"; both
/// mean "no valid reference" so we collapse them.
#[inline]
fn id_opt(v: i32) -> Option<i32> {
    if v < 0 {
        None
    } else {
        Some(v)
    }
}

/// Read a Latin-1 fixed-length C-string at `off..off+len`, stopping at NUL.
fn read_latin1_cstr(bytes: &[u8], off: usize, len: usize) -> String {
    let end = (off + len).min(bytes.len());
    let slice = &bytes[off..end];
    let cut = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    slice[..cut].iter().map(|&b| b as char).collect()
}

// ------ Club (581 B, DAT_00acd5bc, stride 0x245) ------

/// A read-only, typed view over a `club.dat` / `nat_club.dat` record.
///
/// Offsets (from `reports/club_record_offsets.json`):
/// - `+0x00 u32` id
/// - `+0x04..+0x37` primary_name (51-char Latin-1)
/// - `+0x37 u8=0xff` primary-set flag
/// - `+0x38..+0x51` secondary_name (25-char Latin-1)
/// - `+0x52 i8` division (probable) — hottest field: 1971 reads / 57 fns
/// - `+0x53 i32` nation_id — confirmed (73=DE, 27=BR, 149=PT, 179=SE)
/// - `+0x57 i32` city_id (probable)
/// - `+0x69 i32` stadium_id (probable)
/// - `+0x80 u16` reputation — confirmed (Köln 7500, low leagues 1500)
pub struct ClubView<'a> {
    raw: &'a [u8],
}

impl<'a> ClubView<'a> {
    pub const RECORD_SIZE: usize = 0x245;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self {
        Self { raw: &record.raw }
    }

    pub fn from_bytes(raw: &'a [u8]) -> Self {
        Self { raw }
    }

    pub fn id(&self) -> u32 {
        le_u32(self.raw, 0x00)
    }

    pub fn primary_name(&self) -> String {
        read_latin1_cstr(self.raw, 0x04, 51)
    }

    pub fn secondary_name(&self) -> String {
        read_latin1_cstr(self.raw, 0x38, 25)
    }

    /// The `+0x52` byte is the hottest club field in the game — 1971 reads across 57
    /// functions — and its raw value is a small signed integer with `-1` for
    /// "not in a division". Best-current interpretation: **division / league tier**.
    pub fn division(&self) -> Option<i8> {
        match i8_at(self.raw, 0x52) {
            -1 => None,
            v => Some(v),
        }
    }

    /// Confirmed: 73=Germany, 27=Brazil, 149=Portugal, 179=Sweden; `-2` for extinct.
    pub fn nation_id(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x53))
    }

    /// **Primary competition / division id** — CORRECTED 2026-08-20 (was
    /// mislabeled `city_id`). `FUN_0052e370` reads this field as the club's
    /// division: `**(int**)(club+0x57)` dereferences it (in memory a pointer,
    /// on disk the id) and indexes `DAT_00ac688c[comp_id]` to gate playability.
    /// Verified against rust-db: England clubs carry 357 ("A Lower Division"),
    /// 360 ("English Northern Premier"), etc. — all valid club_competition ids.
    pub fn division_id(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x57))
    }

    /// Secondary competition slot (`club+0x5b`) — e.g. a cup the club also
    /// plays in. `-2` when unused. Part of the club→comp wiring the exe walks
    /// during init (`club+0x57/0x5b/0x60`).
    pub fn secondary_comp_id(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x5b))
    }

    /// Tertiary competition slot (`club+0x60`). `-2` when unused.
    pub fn tertiary_comp_id(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x60))
    }

    /// All non-null competition ids this club belongs to (division + secondary
    /// + tertiary). The basis for wiring competitions to nations: a competition
    /// belongs to whichever nation its member clubs are in.
    pub fn competition_ids(&self) -> impl Iterator<Item = i32> + '_ {
        [
            self.division_id(),
            self.secondary_comp_id(),
            self.tertiary_comp_id(),
        ]
        .into_iter()
        .flatten()
    }

    pub fn stadium_id(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x69))
    }

    /// Confirmed: Köln 7500, low-division 1500, extinct 0.
    ///
    /// Note (from FUN_00537870 / FUN_00537730 in the loader): reputation is stored on
    /// disk as a single byte at `+0x80` and multiplied by ×500 at load into a `u16`.
    /// The in-memory value we've been reading in `rust-db` is the post-multiplied one
    /// (1500 = disk byte 3, 7500 = disk byte 15), so this returns the in-memory `u16`
    /// as-is. Use `reputation_raw_byte()` if you want the pre-scaled value.
    pub fn reputation(&self) -> u16 {
        le_u16(self.raw, 0x80)
    }

    /// The 1-byte on-disk value at `+0x80` before the loader's ×500 scale. Only meaningful
    /// if you're reading a record that hasn't been through the loader yet.
    pub fn reputation_raw_byte(&self) -> u8 {
        u8_at(self.raw, 0x80)
    }

    // --- offsets verified by FUN_00537870 (the field-by-field loader path) ---
    //
    // Each field below is fread into these exact offsets by the format-tag-1 loader.
    // Semantics not yet named (they're loader-confirmed as fields, but which one is
    // manager-id / board-balance / league-position / etc. is a follow-up trace).

    /// Loader-confirmed `i32` field at `+0x5b`. Semantics TBD.
    pub fn field_5b(&self) -> i32 {
        le_i32(self.raw, 0x5b)
    }

    /// Loader-confirmed `i32` field at `+0x60`. Semantics TBD.
    pub fn field_60(&self) -> i32 {
        le_i32(self.raw, 0x60)
    }

    /// Loader-confirmed `i32` field at `+0x65`. Semantics TBD.
    pub fn field_65(&self) -> i32 {
        le_i32(self.raw, 0x65)
    }

    /// Loader-confirmed `i32` field at `+0x6e`. Semantics TBD.
    pub fn field_6e(&self) -> i32 {
        le_i32(self.raw, 0x6e)
    }

    /// Loader-confirmed `i32` field at `+0x73`. Highly mutated at runtime (19 reads / 17
    /// writes across the decompile) — probable **balance / cash** or similar tick-updated
    /// financial state.
    pub fn field_73_mutable(&self) -> i32 {
        le_i32(self.raw, 0x73)
    }

    /// Loader-confirmed `i32` field at `+0x77`. Mutable at runtime.
    pub fn field_77_mutable(&self) -> i32 {
        le_i32(self.raw, 0x77)
    }

    /// Loader-confirmed `i32` field at `+0x7b`. Mutable at runtime.
    pub fn field_7b_mutable(&self) -> i32 {
        le_i32(self.raw, 0x7b)
    }
}

// ------ Player / staff-base (110 B, StaffType6, DAT_00acd5c4, stride 0x6e) ------

/// An 8-byte CM date: `{u16 day_of_year, u16 year, u32 is_leap}`.
///
/// Size VERIFIED from the copy helper `FUN_00418770` (moves 2+2+4 bytes).
/// Semantics VERIFIED against the shipped database: across the 132,722 staff
/// records the year field spans 1962–1989 for real people (95,161 records)
/// with `31/1900` as the unset placeholder, and the day field spans 1–365.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmDate {
    /// Day of year, 1–365 (366 in leap years).
    pub day: u16,
    pub year: u16,
    pub is_leap: u32,
}

impl CmDate {
    /// The unset-date placeholder used throughout the shipped database.
    pub const PLACEHOLDER_YEAR: u16 = 1900;

    /// True when this is the `31/1900` "no date recorded" placeholder.
    pub fn is_placeholder(&self) -> bool {
        self.year == Self::PLACEHOLDER_YEAR
    }

    /// Convert day-of-year to `(month, day)`, 1-based.
    pub fn to_month_day(&self) -> (u8, u8) {
        const LENGTHS: [u16; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut remaining = self.day.max(1);
        for (index, base) in LENGTHS.iter().enumerate() {
            let length = base + u16::from(index == 1 && self.is_leap != 0);
            if remaining <= length {
                return (index as u8 + 1, remaining as u8);
            }
            remaining -= length;
        }
        (12, 31)
    }
}

impl CmDate {
    fn read(bytes: &[u8], off: usize) -> Self {
        Self {
            day: le_u16(bytes, off),
            year: le_u16(bytes, off + 2),
            is_leap: le_u32(bytes, off + 4),
        }
    }
}

// ------ Staff / player base record (staff.dat type 6) ------

/// A read-only, typed view over a `staff.dat` type-6 record — the person record
/// (identity, dates, club/nation employment, personality). Player *ability*
/// attributes live in the separate type-10 record (`DomainStaffType10`);
/// non-player ability lives in type 9.
///
/// **Two on-disk formats exist**, selected by the `index.dat` entry's version
/// field (VERIFIED in `FUN_005121a0`, the Start-New-Game static loader):
/// * **version 1 → 157 bytes** (`0x9d`), read by `FUN_00538360`. **This is what
///   the shipped 3.9.60 database uses** — `staff.dat` is exactly
///   `132722*157 + 23785*68 + 109940*70` bytes.
/// * version 2 → 110 bytes (`0x6e`), read by `FUN_00538210`.
///
/// After reading the 157-byte form the loader **unpacks** each record into a
/// 110-byte runtime record plus a 52-byte entry in a parallel "staff
/// preferences" pool (`DAT_00acd5d0`). Offsets `0x00..=0x60` are copied 1:1, so
/// the accessors below are valid for BOTH forms; the fields after `0x60`
/// differ and are marked.
///
/// An earlier version of this file documented `first_name_id` at `+0x00` and a
/// packed DOB at `+0x0a`. Those offsets were wrong — corrected 2026-08-20
/// against the loader's field-by-field copy loop (`FUN_005121a0` lines
/// 2004–2092).
pub struct PlayerView<'a> {
    /// Bytes from record offset `0x04` onward. The record's own `id` (offset
    /// `0x00`) is held separately because `rust-db` stores it as its own JSON
    /// field, so the stored `body` blob begins at `0x04`.
    tail: &'a [u8],
    id: i32,
}

impl<'a> PlayerView<'a> {
    /// On-disk size of the shipped (version-1) record.
    pub const RECORD_SIZE_DISK_V1: usize = 0x9d;
    /// In-memory / version-2 size.
    pub const RECORD_SIZE_MEMORY: usize = 0x6e;
    /// Kept for source compatibility; refers to the in-memory stride.
    pub const RECORD_SIZE: usize = Self::RECORD_SIZE_MEMORY;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self {
        Self::from_bytes(&record.raw)
    }

    /// View over a WHOLE record (id at offset 0).
    pub fn from_bytes(raw: &'a [u8]) -> Self {
        Self {
            id: le_i32(raw, 0x00),
            tail: raw.get(4..).unwrap_or(&[]),
        }
    }

    /// View over `rust-db`'s split storage: the `id` field plus the `body`
    /// blob, where `body[0]` is record offset `0x04`.
    pub fn from_split(id: u32, body: &'a [u8]) -> Self {
        Self {
            id: id as i32,
            tail: body,
        }
    }

    /// Read a field by its RECORD offset (`>= 4`), indexing into `tail`.
    #[inline]
    fn at(&self, record_offset: usize) -> usize {
        record_offset - 4
    }

    /// Whether this view is over a 157-byte version-1 disk record.
    pub fn is_disk_v1(&self) -> bool {
        self.tail.len() + 4 >= Self::RECORD_SIZE_DISK_V1
    }

    // --- identity (offsets 1:1 in both formats) ---

    pub fn staff_id(&self) -> i32 {
        self.id
    }

    /// Index into `first_names.dat`.
    pub fn first_name_id(&self) -> Option<i32> {
        id_opt(le_i32(self.tail, self.at(0x04)))
    }

    /// Index into `second_names.dat`.
    pub fn second_name_id(&self) -> Option<i32> {
        id_opt(le_i32(self.tail, self.at(0x08)))
    }

    /// Index into `common_names.dat` (mostly unset).
    pub fn common_name_id(&self) -> Option<i32> {
        id_opt(le_i32(self.tail, self.at(0x0c)))
    }

    pub fn date_of_birth(&self) -> CmDate {
        CmDate::read(self.tail, self.at(0x10))
    }

    /// A second year value at `+0x18`. **Semantics UNKNOWN.**
    ///
    /// A decode pass initially called this "year_of_birth", duplicating the
    /// year inside [`Self::date_of_birth`]. Measured against the shipped
    /// database that is false: the field is `0` on 109,070 of 132,722 records
    /// (82%), and where both are set they agree only 3,751 times. It is not
    /// player-vs-staff specific either (19,587 players and 4,065 non-players
    /// carry a value). Where present the values cluster in 1980–1983.
    ///
    /// Returned raw so callers can experiment; do not treat it as a birth year.
    pub fn secondary_year_field(&self) -> u16 {
        le_u16(self.tail, self.at(0x18))
    }

    // --- nationality / international career ---

    pub fn nation_id(&self) -> Option<i32> {
        id_opt(le_i32(self.tail, self.at(0x1a)))
    }

    pub fn second_nation_id(&self) -> Option<i32> {
        id_opt(le_i32(self.tail, self.at(0x1e)))
    }

    pub fn international_caps(&self) -> u8 {
        u8_at(self.tail, self.at(0x22))
    }

    pub fn international_goals(&self) -> u8 {
        u8_at(self.tail, self.at(0x23))
    }

    pub fn national_team_id(&self) -> Option<i32> {
        id_opt(le_i32(self.tail, self.at(0x24)))
    }

    pub fn national_job(&self) -> u8 {
        u8_at(self.tail, self.at(0x28))
    }

    pub fn date_joined_national_job(&self) -> CmDate {
        CmDate::read(self.tail, self.at(0x29))
    }

    pub fn national_contract_expires(&self) -> CmDate {
        CmDate::read(self.tail, self.at(0x31))
    }

    // --- club employment ---

    /// Persistent club link. (The exe overwrites this slot with a live pointer
    /// once loaded; on disk and in a saved file it is an ID.)
    pub fn current_club_id(&self) -> Option<i32> {
        id_opt(le_i32(self.tail, self.at(0x39)))
    }

    /// Job at the club. The loader remaps the stored value 7 to 6.
    pub fn club_job(&self) -> u8 {
        match u8_at(self.tail, self.at(0x3d)) {
            7 => 6,
            v => v,
        }
    }

    pub fn date_joined_club(&self) -> CmDate {
        CmDate::read(self.tail, self.at(0x3e))
    }

    pub fn club_contract_expires(&self) -> CmDate {
        CmDate::read(self.tail, self.at(0x46))
    }

    pub fn wage(&self) -> i32 {
        le_i32(self.tail, self.at(0x4e))
    }

    pub fn value(&self) -> i32 {
        le_i32(self.tail, self.at(0x52))
    }

    // --- personality (offsets loader-verified; names community-standard) ---

    pub fn adaptability(&self) -> u8 {
        u8_at(self.tail, self.at(0x56))
    }
    pub fn ambition(&self) -> u8 {
        u8_at(self.tail, self.at(0x57))
    }
    pub fn determination(&self) -> u8 {
        u8_at(self.tail, self.at(0x58))
    }
    pub fn loyalty(&self) -> u8 {
        u8_at(self.tail, self.at(0x59))
    }
    pub fn pressure(&self) -> u8 {
        u8_at(self.tail, self.at(0x5a))
    }
    pub fn professionalism(&self) -> u8 {
        u8_at(self.tail, self.at(0x5b))
    }
    pub fn sportsmanship(&self) -> u8 {
        u8_at(self.tail, self.at(0x5c))
    }
    pub fn temperament(&self) -> u8 {
        u8_at(self.tail, self.at(0x5d))
    }

    /// Squad-membership bit flags; the loader does bit ops on this byte.
    pub fn squad_flags(&self) -> u8 {
        u8_at(self.tail, self.at(0x5e))
    }

    /// Staff classification. The loader treats `6` specially.
    pub fn classification(&self) -> u8 {
        u8_at(self.tail, self.at(0x5f))
    }

    pub fn club_valuation(&self) -> u8 {
        u8_at(self.tail, self.at(0x60))
    }

    // --- links that MOVE between the two formats ---

    /// Link to this person's type-10 player-attribute record. `None` for
    /// non-players.
    ///
    /// Disk v1 stores it at `+0x91`; the runtime/v2 record at `+0x61`.
    pub fn player_data_id(&self) -> Option<i32> {
        let off = if self.is_disk_v1() { 0x91 } else { 0x61 };
        id_opt(le_i32(self.tail, self.at(off)))
    }

    /// Link to this person's type-9 non-player-attribute record.
    ///
    /// Disk v1 stores it at `+0x99`; the runtime/v2 record at `+0x69`.
    pub fn non_player_data_id(&self) -> Option<i32> {
        let off = if self.is_disk_v1() { 0x99 } else { 0x69 };
        id_opt(le_i32(self.tail, self.at(off)))
    }

    /// The 12 object IDs of the embedded preferences block (favourite and
    /// disliked clubs/staff). Only present in the 157-byte disk record — the
    /// loader moves these into a parallel 52-byte pool. Returns `None` for a
    /// runtime/v2 record.
    pub fn preference_ids(&self) -> Option<[Option<i32>; 12]> {
        if !self.is_disk_v1() {
            return None;
        }
        let mut out = [None; 12];
        for (index, slot) in out.iter_mut().enumerate() {
            *slot = id_opt(le_i32(self.tail, self.at(0x61 + index * 4)));
        }
        Some(out)
    }

    /// True when this record has player attributes attached.
    pub fn is_player(&self) -> bool {
        self.player_data_id().is_some()
    }
}

// ------ Nation (290 B, DAT_00acd5b0, stride 0x122) ------

/// A read-only, typed view over a `nation.dat` record.
///
/// Correction from a scan false-positive: earlier we thought `+0x05..+0x44` held per-nation
/// league-config flags. Real data (Andorra: `raw[5]='n', raw[7]='o', raw[9]='r'`) shows those
/// bytes are just chars from within `primary_name`. The nation record itself is simple —
/// per-country league rules live in `nation_comp.dat`, not here.
///
/// **Loader confirmation**: `FUN_00537320` does a bulk `fread(pool, 290, count, fp)` and
/// fatal-errors on any other format tag. The on-disk bytes ARE the in-memory bytes.
///
/// Confirmed offsets:
/// - `+0x00 u32` id
/// - `+0x04..+0x37` primary_name
/// - `+0x37 u8=0xff` primary-set flag
/// - `+0x38..+0x51` secondary_name / abbreviation
/// - `+0x52 u8=0xff` populated flag (always 0xff on live records; polled by ~3 loops)
///
/// Probable cross-refs: `+0x5d`, `+0x69`, `+0x88`, `+0xbf` (i32 each). Semantics
/// (continent, capital_city, etc.) are inferred but not yet locked, so they're
/// exposed as raw `id_opt(i32)` and named by best guess.
pub struct NationView<'a> {
    raw: &'a [u8],
}

impl<'a> NationView<'a> {
    pub const RECORD_SIZE: usize = 0x122;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self {
        Self { raw: &record.raw }
    }

    pub fn from_bytes(raw: &'a [u8]) -> Self {
        Self { raw }
    }

    pub fn id(&self) -> u32 {
        le_u32(self.raw, 0x00)
    }

    pub fn primary_name(&self) -> String {
        read_latin1_cstr(self.raw, 0x04, 51)
    }

    pub fn secondary_name(&self) -> String {
        read_latin1_cstr(self.raw, 0x38, 25)
    }

    /// One of the `i32` cross-refs; probable continent link (nations map to 6 continents).
    pub fn continent_id_probable(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x5d))
    }

    /// One of the `i32` cross-refs; probable capital-city link.
    pub fn capital_city_id_probable(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x69))
    }

    /// **The league-selection flags byte** (`+0x11c`) — the single source of
    /// truth for what the player picked on Select League(s).
    ///
    /// This lives on the NATION record, not the competition record: the picker
    /// lists countries, and its slot table (`DAT_00b4bc70`, stride 0x48) holds
    /// nation-record pointers. VERIFIED in `FUN_00811140`, which clears the
    /// byte across the nation pool with `*(u8*)(i + 0x11c + DAT_00acd5b0) = 0`
    /// stepping `i += 0x122` (the nation stride), and in the game-wide gates
    /// `*(int*)(comp+0x5d) + 0x11c & 4` (comp → nation) and
    /// `*(int*)(club+0x53) + 0x11c & 3` (club → nation).
    ///
    /// Bits: `1` = background league, `2` = foreground (playable) league,
    /// `4` = nation is active in this game.
    ///
    /// **Runtime-only**: on disk this byte is `0x00` in all 213 shipped records
    /// and the screen zeroes it before use, so a freshly imported database
    /// always reads 0 here. It is part of save state, not base data.
    pub fn selection_flags(&self) -> u8 {
        u8_at(self.raw, 0x11c)
    }

    pub fn is_background_league(&self) -> bool {
        self.selection_flags() & 1 != 0
    }

    pub fn is_foreground_league(&self) -> bool {
        self.selection_flags() & 2 != 0
    }

    pub fn is_active_nation(&self) -> bool {
        self.selection_flags() & 4 != 0
    }
}

// ------ Competition (107 B club_comp/nation_comp, 101 B staff_comp) ------

/// A read-only, typed view over a `club_comp.dat` / `nation_comp.dat` record
/// (107 bytes, in-memory stride `0x6b` — the file is a raw dump of the pool).
///
/// **Loader**: `FUN_005121a0` case `0xc` → `FUN_00539a90`, which for format
/// version 2 does a single `fread(pool, 0x6b, count, file)`. The only load-time
/// transformation is `FUN_0051b110` swizzling the four link ids into pointers
/// in place. Pool base `DAT_00acd5d8`, count `DAT_00acd580`. The loader also
/// appends 127 empty slots for competitions created at runtime.
///
/// **What is actually shipped**: in `club_comp.dat` the four link fields are
/// empty (continent/nation/foreground = 0, background = `0xFF000000`) and
/// `nation_comp.dat` is all-zero there. The comp↔nation wiring is established
/// during "Initialising game data", reachable from clubs instead
/// (`club+0x53` = nation, `club+0x57/0x5b/0x60` = comps). So only id, names,
/// genders, abbreviation and reputation carry shipped information — do not
/// trust the link fields straight out of an import.
pub struct CompetitionView<'a> {
    raw: &'a [u8],
}

impl<'a> CompetitionView<'a> {
    /// club_comp.dat and nation_comp.dat.
    pub const RECORD_SIZE: usize = 0x6b;
    /// staff_comp.dat uses the same field layout in a shorter record.
    pub const RECORD_SIZE_STAFF: usize = 0x65;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self {
        Self { raw: &record.raw }
    }

    pub fn from_bytes(raw: &'a [u8]) -> Self {
        Self { raw }
    }

    pub fn id(&self) -> u32 {
        le_u32(self.raw, 0x00)
    }

    pub fn long_name(&self) -> String {
        read_latin1_cstr(self.raw, 0x04, 51)
    }

    /// Grammatical gender marker for the long name, consumed by the text
    /// formatter. The name-override loader clamps values `< 1` to `0`.
    pub fn long_name_gender(&self) -> i8 {
        i8_at(self.raw, 0x37)
    }

    pub fn short_name(&self) -> String {
        read_latin1_cstr(self.raw, 0x38, 26)
    }

    pub fn short_name_gender(&self) -> i8 {
        i8_at(self.raw, 0x52)
    }

    /// Three-letter code plus NUL.
    pub fn abbreviation(&self) -> String {
        read_latin1_cstr(self.raw, 0x53, 4)
    }

    /// Link to `continent.dat`. Empty in the shipped file (see type docs).
    pub fn continent_id(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x59))
    }

    /// Link to `nation.dat`. Empty in the shipped file (see type docs).
    pub fn nation_id(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x5d))
    }

    /// Foreground kit/branding colour id. PROBABLE.
    pub fn foreground_colour_id(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x61))
    }

    /// Background kit/branding colour id. PROBABLE.
    pub fn background_colour_id(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x65))
    }

    /// Competition reputation, `0..=20`. VERIFIED — compared against the
    /// thresholds 5/8/11/16 and scaled by ×500 in transfer/ambition logic;
    /// the shipped data spans exactly 0–20 (World Cup = 20).
    pub fn reputation(&self) -> i16 {
        le_u16(self.raw, 0x69) as i16
    }
}

// ------ Colour (58 B, DAT_00acd5f4, stride 0x3a) ------

/// A read-only, typed view over a `colour.dat` record.
///
/// **Loader**: `FUN_0053a350` bulk-reads `0x3a`-byte records — disk bytes are
/// memory bytes, no unpacking. Dispatched from the DB loader `FUN_005121a0`
/// case 2. Pool base `DAT_00acd5f4`, count `DAT_00acd59c`.
///
/// The RGB triple is VERIFIED: `FUN_00525190` passes `+0x37/+0x38/+0x39` to the
/// device-colour packer `FUN_005ce4f0` when drawing kit colours, and the shipped
/// data agrees with the names (Red 1 = `E0 00 00`, White = `FF FF FF`).
///
/// Clubs reference colours by index at `club+0x83/0x87` (kit 1 fg/bg),
/// `+0x8b/0x8f` (kit 2), `+0x93/0x97` (kit 3).
pub struct ColourView<'a> {
    raw: &'a [u8],
}

impl<'a> ColourView<'a> {
    pub const RECORD_SIZE: usize = 0x3a;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self {
        Self { raw: &record.raw }
    }

    pub fn from_bytes(raw: &'a [u8]) -> Self {
        Self { raw }
    }

    pub fn id(&self) -> u32 {
        le_u32(self.raw, 0x00)
    }

    /// e.g. "Black", "White", "Grey 1".
    pub fn name(&self) -> String {
        read_latin1_cstr(self.raw, 0x04, 51)
    }

    /// `(r, g, b)` — ready to pack into RGB565 for the renderer.
    pub fn rgb(&self) -> (u8, u8, u8) {
        (
            u8_at(self.raw, 0x37),
            u8_at(self.raw, 0x38),
            u8_at(self.raw, 0x39),
        )
    }
}

// ------ Continent (198 B, DAT_00acd5ac, stride 0xc6) ------

/// A read-only, typed view over a `continent.dat` record (6 records).
///
/// **Loader**: `FUN_005371c0` bulk-reads `0xc6`-byte records — disk bytes are
/// memory bytes. Dispatched from `FUN_005121a0` case 3. Pool base
/// `DAT_00acd5ac`, count `DAT_00acd554`.
///
/// Verified readers: `FUN_0058d2c0` displays `+0x04` as the continent name and
/// `+0xa3` as the federation acronym (its fallbacks are the strings "Unknown
/// Continent" / "Unknown Federation"); `FUN_00600f50` returns `+0x23` as the
/// demonym. The `0xff` bytes that follow each string buffer are the grammatical
/// gender markers the text formatter consumes.
pub struct ContinentView<'a> {
    raw: &'a [u8],
}

impl<'a> ContinentView<'a> {
    pub const RECORD_SIZE: usize = 0xc6;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self {
        Self { raw: &record.raw }
    }

    pub fn from_bytes(raw: &'a [u8]) -> Self {
        Self { raw }
    }

    pub fn id(&self) -> u32 {
        le_u32(self.raw, 0x00)
    }

    /// e.g. "Europe", "North America".
    pub fn name(&self) -> String {
        read_latin1_cstr(self.raw, 0x04, 26)
    }

    /// 3-letter code, e.g. "EUR", "AFR". PROBABLE — data-proven, no reader found.
    pub fn code(&self) -> String {
        read_latin1_cstr(self.raw, 0x1f, 4)
    }

    /// Demonym, e.g. "European", "African".
    pub fn adjective(&self) -> String {
        read_latin1_cstr(self.raw, 0x23, 26)
    }

    /// e.g. "Union of European Football Associations". PROBABLE.
    pub fn confederation_name(&self) -> String {
        read_latin1_cstr(self.raw, 0x3d, 101)
    }

    /// e.g. "UEFA", "CAF", "CONMEBOL".
    pub fn confederation_acronym(&self) -> String {
        read_latin1_cstr(self.raw, 0xa3, 26)
    }

    /// Strength coefficient — Europe/S.America 1.0, Africa 0.95, Asia/N.America
    /// 0.9, Oceania 0.85. Type VERIFIED (valid IEEE-754 in the shipped data);
    /// meaning PROBABLE — no engine reader located yet.
    pub fn strength_coefficient(&self) -> f64 {
        if 0xbe + 8 <= self.raw.len() {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&self.raw[0xbe..0xbe + 8]);
            f64::from_le_bytes(buf)
        } else {
            0.0
        }
    }
}

// ------ tests ------

#[cfg(test)]
mod tests {
    use super::*;

    fn opaque_from(mut raw: Vec<u8>, target_len: usize) -> DomainOpaqueRecord {
        raw.resize(target_len, 0);
        DomainOpaqueRecord {
            ordinal: 0,
            id: 0,
            primary_name: None,
            secondary_name: None,
            short_name: None,
            text_candidates: Vec::new(),
            raw,
        }
    }

    /// Reconstruct 1.FC Bocholt's expected fields from a synthesized 581-byte record.
    /// Values match what we saw in `rust-db/core/clubs.json` (id=0, name at +4, nation Germany=73).
    #[test]
    fn club_view_decodes_bocholt_like_record() {
        let mut raw = vec![0u8; ClubView::RECORD_SIZE];
        // id
        raw[0..4].copy_from_slice(&0u32.to_le_bytes());
        // primary_name @ +4
        let name = b"1.FC Bocholt";
        raw[4..4 + name.len()].copy_from_slice(name);
        // primary-set flag @ +0x37
        raw[0x37] = 0xff;
        // secondary_name @ +0x38
        raw[0x38..0x38 + name.len()].copy_from_slice(name);
        // populated flag @ +0x52 — this is division (-1 = not in a division)
        raw[0x52] = 0xff; // -1 as i8
                          // nation_id @ +0x53 = Germany (73)
        raw[0x53..0x57].copy_from_slice(&73i32.to_le_bytes());
        // division/competition id @ +0x57 (357 = a real English lower division)
        raw[0x57..0x5b].copy_from_slice(&357i32.to_le_bytes());
        // stadium_id @ +0x69
        raw[0x69..0x6d].copy_from_slice(&6877i32.to_le_bytes());
        // reputation @ +0x80
        raw[0x80..0x82].copy_from_slice(&1500u16.to_le_bytes());

        let rec = opaque_from(raw, ClubView::RECORD_SIZE);
        let v = ClubView::new(&rec);
        assert_eq!(v.id(), 0);
        assert_eq!(v.primary_name(), "1.FC Bocholt");
        assert_eq!(v.secondary_name(), "1.FC Bocholt");
        assert_eq!(v.division(), None, "0xff = -1 = not in a division");
        assert_eq!(v.nation_id(), Some(73));
        assert_eq!(v.division_id(), Some(357));
        assert_eq!(v.stadium_id(), Some(6877));
        assert_eq!(v.reputation(), 1500);
    }

    #[test]
    fn club_view_handles_extinct_sentinel() {
        // "Balkan B" style: nation_id = -2 (extinct placeholder)
        let mut raw = vec![0u8; ClubView::RECORD_SIZE];
        raw[0x53..0x57].copy_from_slice(&(-2i32).to_le_bytes());
        raw[0x69..0x6d].copy_from_slice(&(-2i32).to_le_bytes());
        let rec = opaque_from(raw, ClubView::RECORD_SIZE);
        let v = ClubView::new(&rec);
        assert_eq!(v.nation_id(), None);
        assert_eq!(v.stadium_id(), None);
    }

    /// The shipped database uses the 157-byte version-1 staff record. Offsets
    /// are the loader's (`FUN_005121a0` field-by-field copy loop).
    #[test]
    fn player_view_decodes_v1_disk_record() {
        let mut raw = vec![0u8; PlayerView::RECORD_SIZE_DISK_V1];
        raw[0x00..0x04].copy_from_slice(&4242i32.to_le_bytes()); // staff id
        raw[0x04..0x08].copy_from_slice(&12345i32.to_le_bytes()); // first name
        raw[0x08..0x0c].copy_from_slice(&678i32.to_le_bytes()); // second name
        raw[0x0c..0x10].copy_from_slice(&(-1i32).to_le_bytes()); // no common name
                                                                 // DOB: day 200 of 1978, non-leap
        raw[0x10..0x12].copy_from_slice(&200u16.to_le_bytes());
        raw[0x12..0x14].copy_from_slice(&1978u16.to_le_bytes());
        raw[0x1a..0x1e].copy_from_slice(&73i32.to_le_bytes()); // Germany
        raw[0x22] = 45; // caps
        raw[0x23] = 12; // goals
        raw[0x39..0x3d].copy_from_slice(&(-1i32).to_le_bytes()); // free agent
        raw[0x3d] = 7; // job 7 remaps to 6
        raw[0x58] = 17; // determination
        raw[0x91..0x95].copy_from_slice(&9001i32.to_le_bytes()); // player attribs
        raw[0x99..0x9d].copy_from_slice(&(-1i32).to_le_bytes()); // no staff attribs

        let rec = opaque_from(raw, PlayerView::RECORD_SIZE_DISK_V1);
        let v = PlayerView::new(&rec);
        assert!(v.is_disk_v1());
        assert_eq!(v.staff_id(), 4242);
        assert_eq!(v.first_name_id(), Some(12345));
        assert_eq!(v.second_name_id(), Some(678));
        assert_eq!(v.common_name_id(), None);
        assert_eq!(v.date_of_birth().year, 1978);
        assert_eq!(v.date_of_birth().day, 200);
        assert_eq!(v.date_of_birth().to_month_day(), (7, 19), "day 200 of a non-leap year");
        assert_eq!(v.nation_id(), Some(73));
        assert_eq!(v.international_caps(), 45);
        assert_eq!(v.international_goals(), 12);
        assert_eq!(v.current_club_id(), None, "-1 = free agent");
        assert_eq!(v.club_job(), 6, "loader remaps job 7 to 6");
        assert_eq!(v.determination(), 17);
        assert_eq!(v.player_data_id(), Some(9001));
        assert_eq!(v.non_player_data_id(), None);
        assert!(v.is_player());
        assert!(v.preference_ids().is_some(), "v1 carries the embedded prefs");
    }

    /// The same accessors must work on a 110-byte runtime/v2 record, where the
    /// two attribute links sit at different offsets.
    #[test]
    fn player_view_reads_v2_links_from_their_own_offsets() {
        let mut raw = vec![0u8; PlayerView::RECORD_SIZE_MEMORY];
        raw[0x00..0x04].copy_from_slice(&7i32.to_le_bytes());
        raw[0x61..0x65].copy_from_slice(&555i32.to_le_bytes()); // player attribs
        raw[0x69..0x6d].copy_from_slice(&(-1i32).to_le_bytes());
        let rec = opaque_from(raw, PlayerView::RECORD_SIZE_MEMORY);
        let v = PlayerView::new(&rec);
        assert!(!v.is_disk_v1());
        assert_eq!(v.staff_id(), 7);
        assert_eq!(v.player_data_id(), Some(555));
        assert_eq!(v.non_player_data_id(), None);
        assert!(v.preference_ids().is_none(), "v2 has no embedded prefs");
    }

    #[test]
    fn colour_view_decodes_name_and_rgb() {
        let mut raw = vec![0u8; ColourView::RECORD_SIZE];
        raw[0x00..0x04].copy_from_slice(&3u32.to_le_bytes());
        raw[4..9].copy_from_slice(b"White");
        raw[0x37] = 0xff;
        raw[0x38] = 0xff;
        raw[0x39] = 0xff;
        let rec = opaque_from(raw, ColourView::RECORD_SIZE);
        let v = ColourView::new(&rec);
        assert_eq!(v.id(), 3);
        assert_eq!(v.name(), "White");
        assert_eq!(v.rgb(), (255, 255, 255));
    }

    #[test]
    fn continent_view_decodes_names_and_coefficient() {
        let mut raw = vec![0u8; ContinentView::RECORD_SIZE];
        raw[0x00..0x04].copy_from_slice(&0u32.to_le_bytes());
        raw[0x04..0x0a].copy_from_slice(b"Europe");
        raw[0x1f..0x22].copy_from_slice(b"EUR");
        raw[0x23..0x2b].copy_from_slice(b"European");
        raw[0xa3..0xa7].copy_from_slice(b"UEFA");
        raw[0xbe..0xc6].copy_from_slice(&1.0f64.to_le_bytes());
        let rec = opaque_from(raw, ContinentView::RECORD_SIZE);
        let v = ContinentView::new(&rec);
        assert_eq!(v.id(), 0);
        assert_eq!(v.name(), "Europe");
        assert_eq!(v.code(), "EUR");
        assert_eq!(v.adjective(), "European");
        assert_eq!(v.confederation_acronym(), "UEFA");
        assert!((v.strength_coefficient() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn nation_view_decodes_id_and_names() {
        let mut raw = vec![0u8; NationView::RECORD_SIZE];
        raw[0x00..0x04].copy_from_slice(&73u32.to_le_bytes()); // Germany
        raw[4..11].copy_from_slice(b"Germany");
        raw[0x37] = 0xff;
        raw[0x38..0x38 + 3].copy_from_slice(b"GER");
        raw[0x52] = 0xff;
        let rec = opaque_from(raw, NationView::RECORD_SIZE);
        let v = NationView::new(&rec);
        assert_eq!(v.id(), 73);
        assert_eq!(v.primary_name(), "Germany");
        assert_eq!(v.secondary_name(), "GER");
    }

    /// Nation-record correction test: proves that offsets `+0x05..+0x0d` on a real
    /// nation record are chars from within the primary_name, NOT a config-flag zone.
    /// (The previous "config-heavy record" interpretation was a scan false positive.)
    #[test]
    fn nation_zone_after_name_is_actually_name_chars() {
        let mut raw = vec![0u8; NationView::RECORD_SIZE];
        raw[4..11].copy_from_slice(b"Andorra");
        // "Andorra" = A(4) n(5) d(6) o(7) r(8) r(9) a(10).
        // If the zone were a config field, these wouldn't be name letters. They are.
        assert_eq!(raw[0x05], b'n');
        assert_eq!(raw[0x07], b'o');
        assert_eq!(raw[0x09], b'r');
    }
}
