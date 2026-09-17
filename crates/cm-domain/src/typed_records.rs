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

fn le_i16(bytes: &[u8], off: usize) -> i16 {
    let mut buf = [0u8; 2];
    if let Some(chunk) = bytes.get(off..off + 2) { buf.copy_from_slice(chunk); }
    i16::from_le_bytes(buf)
}

fn le_f64(bytes: &[u8], off: usize) -> f64 {
    let mut buf = [0u8; 8];
    if let Some(chunk) = bytes.get(off..off + 8) { buf.copy_from_slice(chunk); }
    f64::from_le_bytes(buf)
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

    /// **New-game cash seed (i32, £)** at disk `Club+0x65`.
    ///
    /// C15.1F archaeology confirmed this is the ONE-TIME boot
    /// seed the exe uses to initialise runtime cash: the sole
    /// reader is `FUN_005803D0` (per-club finance ctor,
    /// `005803d0.c:47/96/100`), which reads this i32 and stores
    /// it — via `__ftol` — into the runtime finance record's
    /// i64 cash at `+0x00`. After boot, live cash lives on the
    /// separate 0x167-byte runtime finance pool
    /// (`RuntimeSaveGame.finance`, a `FinanceBook`), and `Club+0x65`
    /// becomes dead data on the disk record.
    ///
    /// Verified against the shipped 2001-02 database: Real
    /// Madrid £100M, Man Utd £30M, Sheffield Wednesday -£14M
    /// (bankrupt), Sheffield United -£8M. Distribution across
    /// all 10,580 clubs: p50=£0, p90=£260k, max=£102M,
    /// min=-£22.5M; 287 clubs (2.7%) ship with negative
    /// balances — those are the "Bankrupt" ones the editor
    /// shows.
    pub fn initial_cash_seed(&self) -> i32 {
        le_i32(self.raw, 0x65)
    }

    /// **Deprecated** alias for [`Self::initial_cash_seed`].
    ///
    /// Prior to C15.1F this method was named `cash`, which
    /// suggested it returned live runtime cash. It never did —
    /// the value is the disk seed only. Existing callers
    /// (importer, forced-path finance seed helper) are correct
    /// but misnamed; new code should read
    /// `initial_cash_seed()` for clarity and, when live cash is
    /// needed, look it up via
    /// `RuntimeSaveGame.finance.for_club(club_id).balance`
    /// instead.
    #[deprecated(
        since = "C15.1F",
        note = "reads the disk SEED at +0x65 only; for live \
                runtime cash query RuntimeSaveGame.finance (FinanceBook). \
                Rename to initial_cash_seed()."
    )]
    pub fn cash(&self) -> i32 { self.initial_cash_seed() }

    // --- newly confirmed offsets (editor decode agent, 2026-08-30) ---
    // The three fields at +0x73/+0x77/+0x7b were previously flagged as
    // "probable finance" — cross-club sampling shows they scale exactly with
    // stadium size (Old Trafford, Emirates, Hillsborough, Rushden). So they
    // are attendance figures, not cash balances.

    /// Average / expected attendance. Scaled from stadium capacity.
    pub fn attendance_average(&self) -> i32 { le_i32(self.raw, 0x73) }
    /// Minimum expected attendance (typical low-attendance fixture).
    pub fn attendance_minimum(&self) -> i32 { le_i32(self.raw, 0x77) }
    /// Maximum/capacity attendance for the club at its home stadium.
    pub fn attendance_maximum(&self) -> i32 { le_i32(self.raw, 0x7b) }

    /// Rival clubs — three staff/club id slots. Verified: Sheffield Wednesday's
    /// `rival_1` = 8370 (Sheffield United — the cross-town derby).
    pub fn rival_club_1(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xb3)) }
    pub fn rival_club_2(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xb7)) }
    pub fn rival_club_3(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xbb)) }

    /// Manager (staff id). Very heavily used in the game (46 reads across 16
    /// functions per the club record-offset corpus). Verified: SWFC = 58080
    /// (Peter Shreeves).
    pub fn manager_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xcf)) }
    /// Assistant manager (staff id). Verified: SWFC = 59310 (Terry Yorath).
    pub fn assistant_manager_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xd3)) }

    /// Flag byte at +0x8b — unknown semantics but distinctive per-club. SWFC=0x11,
    /// Arsenal=0x0d, Man Utd/Rushden=0x00. Candidate for `club_professional_status`
    /// / financial state flag (the "Bankrupt" indicator the editor shows).
    pub fn flag_byte_8b(&self) -> u8 { u8_at(self.raw, 0x8b) }

    /// Home stadium id (+0x69). Verified against stadium.dat: Bayern München
    /// AND TSV 1860 München both read 710 = "Olympiastadion"; Alemannia
    /// Aachen and their reserves both read 709 (their shared ground).
    /// `-2` and `0` are unset sentinels.
    ///
    /// This is what `FUN_00586ec0`:56-114 keys off — the £20M event fires
    /// between clubs that share a stadium, NOT between clubs under a shared
    /// chairman (Bayern's chairman is Beckenbauer, 1860's is Wildmoser; the
    /// only thing they share is the ground).
    pub fn home_stadium_id(&self) -> Option<i32> {
        let v = le_i32(self.raw, 0x69);
        if v <= 0 { None } else { Some(v) }
    }

    // --- Kit colours (VERIFIED via FUN_00525190 & FUN_006ba1e0) ---
    // Each field is a colour-record id (i32); the render code resolves it to
    // the colour pool and reads (r, g, b) at +0x37/+0x38/+0x39 on the pointed
    // record. Kit 1 = home, kit 2 = away, kit 3 = third. `-2` = unused;
    // when either kit-2 slot is 0/unset the match-kit picker falls back to
    // REVERSED kit 1 (bg swapped with fg).
    pub fn kit1_fg_color_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x83)) }
    pub fn kit1_bg_color_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x87)) }
    pub fn kit2_fg_color_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x8b)) }
    pub fn kit2_bg_color_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x8f)) }
    pub fn kit3_fg_color_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x93)) }
    pub fn kit3_bg_color_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x97)) }

    // --- People: chairman + 3 board members (loader loop at 0xc3, size 3) ---

    /// Chairman (staff id). VERIFIED via FUN_00583fc0 (gate-receipts rent),
    /// which dereferences the pointed record at +0x69. Sample: Man Utd = 180,
    /// Bayern = 67647, Sheff Wed = 54811, PSG = 66210. -1 in ~85% of clubs
    /// (small clubs with no named chairman).
    pub fn chairman_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xbf)) }

    /// Board members — a fixed 3-slot array at +0xc3 read in a loop by the
    /// loader (`00537870.c` lines 349-362). Slot semantics (director /
    /// vice-chair / treasurer) best-guess.
    pub fn board_member(&self, i: usize) -> Option<i32> {
        if i >= 3 { return None; }
        id_opt(le_i32(self.raw, 0xc3 + i * 4))
    }

    // --- Roster arrays (loader loops at 0xd7 / 0x19f / 0x1b3 / 0x1cf) ---
    // Sizes 50 / 5 / 7 / 3 are LOADER-VERIFIED (`00537870.c` lines 381-440).
    // Slot semantics (players / coaches / scouts / physios) are best-guess
    // from typical CM roster hierarchies; the loader does not distinguish
    // roles, it just reads back-to-back i32 arrays.

    pub const SQUAD_SLOTS: usize = 50;
    pub const COACH_SLOTS: usize = 5;
    pub const SCOUT_SLOTS: usize = 7;
    pub const PHYSIO_SLOTS: usize = 3;

    pub fn squad_slot(&self, i: usize) -> Option<i32> {
        if i >= Self::SQUAD_SLOTS { return None; }
        id_opt(le_i32(self.raw, 0xd7 + i * 4))
    }
    pub fn coach_slot(&self, i: usize) -> Option<i32> {
        if i >= Self::COACH_SLOTS { return None; }
        id_opt(le_i32(self.raw, 0x19f + i * 4))
    }
    pub fn scout_slot(&self, i: usize) -> Option<i32> {
        if i >= Self::SCOUT_SLOTS { return None; }
        id_opt(le_i32(self.raw, 0x1b3 + i * 4))
    }
    pub fn physio_slot(&self, i: usize) -> Option<i32> {
        if i >= Self::PHYSIO_SLOTS { return None; }
        id_opt(le_i32(self.raw, 0x1cf + i * 4))
    }

    /// All squad-array staff ids, in loader order, skipping -1 sentinels.
    pub fn squad_ids(&self) -> impl Iterator<Item = i32> + '_ {
        (0..Self::SQUAD_SLOTS).filter_map(move |i| self.squad_slot(i))
    }

    // --- Six loader-confirmed staff-refs at 0x9b..0xaf (semantics TBD) ---
    // Values are in the staff-id range (0..132,714). Almost every shipped
    // record has -1 in most of these slots; big clubs populate a few.
    // Candidates: club legends / notable ex-managers / historical
    // player-of-the-year references — not yet cross-referenced.
    pub fn staff_ref_9b(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x9b)) }
    pub fn staff_ref_9f(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x9f)) }
    pub fn staff_ref_a3(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xa3)) }
    pub fn staff_ref_a7(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xa7)) }
    pub fn staff_ref_ab(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xab)) }
    pub fn staff_ref_af(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xaf)) }

    // --- Small-enum bytes / flags (loader-confirmed as u8 fields) ---

    /// Three-value enum {1, 2, 3} across the whole DB (5287 / 1705 / 3588
    /// records respectively). Best-guess: 1 = professional, 2 = semi-pro,
    /// 3 = amateur / reserve. Man Utd / Bayern / PSG / Sheff Wed = 1;
    /// "1.FC Synot B" (reserve) and "SV Arminia Hannover" = 3.
    pub fn club_status(&self) -> u8 { u8_at(self.raw, 0x64) }

    /// Rare-set flag byte at +0x82 — 0xff on ~128 clubs (Ajax, Aston Villa,
    /// Kashima Antlers, Alania Vladikavkaz, Antalyaspor, ...). Semantics TBD.
    pub fn flag_82(&self) -> u8 { u8_at(self.raw, 0x82) }

    pub fn flag_5f(&self) -> u8 { u8_at(self.raw, 0x5f) }
    pub fn flag_6d(&self) -> u8 { u8_at(self.raw, 0x6d) }
    pub fn flag_72(&self) -> u8 { u8_at(self.raw, 0x72) }
    pub fn flag_7f(&self) -> u8 { u8_at(self.raw, 0x7f) }

    /// True when this club has a shipped chairman staff record. Reads the
    /// existing [`Self::chairman_id`] at raw offset +0xbf. Supersedes the
    /// earlier misidentification `flag_6d() != 0` as the "has chairman"
    /// flag — +0x6d is 0 for every shipped club record in rust-db.
    /// Approximately 25% of shipped clubs (2625/10580) have a chairman.
    pub fn has_chairman(&self) -> bool { self.chairman_id().is_some() }

    // --- Runtime state (all -1 / 0 in shipped clubs; kept for save-file
    //     round-trip and for later runtime read/write). ---

    /// Paired (ref, type) — set together by the pending-event code path.
    /// Both slots are -1 in every shipped record; heavy runtime reads/writes.
    pub fn pending_ref(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x1db)) }
    pub fn pending_ref_type(&self) -> u8 { u8_at(self.raw, 0x1df) }

    /// Runtime transfer-target watchlist. 20-slot array of player pointers.
    /// VERIFIED via FUN_00546e70 which walks it capped at index 0x13.
    /// All -1 in the shipped database.
    pub const TRANSFER_TARGET_SLOTS: usize = 20;
    pub fn transfer_target(&self, i: usize) -> Option<i32> {
        if i >= Self::TRANSFER_TARGET_SLOTS { return None; }
        id_opt(le_i32(self.raw, 0x1e0 + i * 4))
    }

    /// Runtime 4-slot pending-bid / negotiation array. Semantics best-guess.
    pub fn pending_bid(&self, i: usize) -> Option<i32> {
        if i >= 4 { return None; }
        id_opt(le_i32(self.raw, 0x230 + i * 4))
    }

    /// Runtime misc-pending single ref. -1 in every shipped record.
    pub fn misc_pending(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x240)) }
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

/// 5 release-clause flags on a staff contract. Semantics per
/// FUN_00850fd0: 0 = absent, 1 = armed, 2 = tripped. Field order
/// verified against `agevak::AgevakTContractOffsets` (0x1C..0x20).
#[derive(Debug, Clone, Copy, Default)]
pub struct ReleaseClauses {
    pub non_promotion: u8,
    pub minimum_fee:   u8,
    pub non_playing:   u8,
    pub relegation:    u8,
    pub manager_job:   u8,
}

impl ReleaseClauses {
    /// Squad-view "Releases" column short-code. Direct port of the
    /// precedence tree in FUN_00850fd0 (00850fd0.c), verified against
    /// scratchpad/prelaunch/cheltenham_contract.png (Muggleton →
    /// "NP & Rlg", Keith Hill → "Rlg."). Returns `""` when no clause
    /// is set — the caller renders a cyan '-' in that case.
    ///
    /// The five short codes are literals from the exe's `.rdata`:
    ///   0x00A6F658 "Man."
    ///   0x00A6F69C "NP & Rlg"
    ///   0x00A6F6F4 "Rlg."
    ///   0x00A6F734 "Non Pro."
    ///   0x00A6F77C "Min.Fee"
    ///
    /// The exe NEVER joins codes at runtime with " & " — the combined
    /// "NP & Rlg" is a single pre-canned string. Precedence:
    ///   1. minimum_fee     → "Min.Fee"
    ///   2. NP && Rlg both set:
    ///        NP == 2 → "Non Pro."   (tripped)
    ///        Rlg == 2 → "Rlg."      (tripped)
    ///        else   → "NP & Rlg"
    ///   3. NP alone        → "Non Pro."
    ///   4. Rlg alone       → "Rlg."
    ///   5. manager_job     → "Man."
    ///   6. else            → ""
    pub fn short_code(&self) -> &'static str {
        if self.minimum_fee != 0 { return "Min.Fee"; }
        let (np, rlg) = (self.non_promotion, self.relegation);
        if np != 0 && rlg != 0 {
            if np  == 2 { return "Non Pro."; }
            if rlg == 2 { return "Rlg."; }
            return "NP & Rlg";
        }
        if np  != 0 { return "Non Pro."; }
        if rlg != 0 { return "Rlg."; }
        if self.manager_job != 0 { return "Man."; }
        ""
    }
    /// Whether the short code should be painted in the HIGHLIGHT
    /// colour (orange). Matches the return-1 / return-2 test the exe
    /// makes on FUN_00850fd0 to pick DAT_00acdf98 (red/orange) vs
    /// DAT_00ad6bc4 (default). ANY set clause colours the cell.
    pub fn is_active(&self) -> bool {
        !self.short_code().is_empty()
    }
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

    /// Five release-clause flags. Byte semantics per FUN_00850fd0:
    /// 0 = absent, 1 = armed, 2 = tripped.
    ///
    /// # Data source (not yet available)
    ///
    /// The clauses do NOT live on the Person record. They live on a
    /// separate 80-byte (0x50) Contract record allocated at game boot
    /// by `CONTRACT_MANAGER::initialise_all` (FUN_004cd930), one per
    /// staff-with-employer. The lookup is:
    ///
    ///   contract_ptr = (*DAT_00accad8) + DAT_00acdf0c[staff_id] * 0x50
    ///   clause_byte  = contract_ptr[0x1C + N]      // N=0..4
    ///
    /// The record layout on that 0x50 array is:
    ///   +0x00  staff_id (i32 sanity check)
    ///   +0x04  linked Person id
    ///   +0x0C  wage (i32)
    ///   +0x1C  Non-Promotion flag       ← reads
    ///   +0x1D  Minimum-Fee flag         ← reads
    ///   +0x1E  Non-Playing flag         ← reads
    ///   +0x1F  Relegation flag          ← reads
    ///   +0x20  Manager-Job flag         ← reads
    ///   +0x25  contract-start date word (short)
    ///   +0x27  contract-end date word
    ///   +0x2D / +0x2F  additional date words
    ///
    /// The generator (`FUN_00847a80`, called from FUN_004cd930 during
    /// boot) rolls each clause using reputation + position + RNG:
    ///
    ///   Manager-Job    (`+0x20`) → age gate + rep > 0xCB2 + id % 3 == 0
    ///   Non-Promotion  (`+0x1C`) → class == 0x0B + rep > 0xABE  + roll
    ///   Non-Playing    (`+0x1E`) → class == 0x0B + rep > 0x1964 + roll
    ///   Relegation     (`+0x1F`) → class == 0x0B + rep > 0x6D6  + roll
    ///   Minimum-Fee    (`+0x1D`) → cleared here; only ever set during
    ///                              transfer negotiation
    ///
    /// # Blocker
    ///
    /// Our rust-db import stops at the shipped .dat pools; ~88% of
    /// staff (Muggleton included) leave the shipped dat with no
    /// contract data at all — the exe generates them post-load via the
    /// path above. Wiring this correctly needs a real port of
    /// FUN_004cd930 + FUN_00847a80 + FUN_004d7090 (wage computer)
    /// into a boot-time contract-init subsystem. Not a byte-offset
    /// fix.
    ///
    /// Until that subsystem lands, this returns all zeros and the
    /// Releases column stays blank. The FORMATTER path (short_code +
    /// orange ink) is correct and will light up as soon as data flows.
    pub fn release_clauses(&self) -> ReleaseClauses {
        ReleaseClauses::default()
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
    ///
    /// SUPERSEDED — this offset holds text, not the continent. Use
    /// [`Self::continent_id`] (`+0x71`) instead.
    pub fn continent_id_probable(&self) -> Option<i32> {
        id_opt(le_i32(self.raw, 0x5d))
    }

    /// **The nation's continent id** (`+0x71`). VERIFIED against the shipped
    /// data — 14/14 known nations map correctly (Africa=0, Asia=1, Europe=2,
    /// N.America=3, Oceania=4, S.America=5), and it is exactly the field the
    /// African Cup of Nations draw reads: `afrcup_draw_qualifiers`
    /// (`FUN_00402300`) compares `*(*(nation+0x71))` against the Africa
    /// continent object `DAT_009bbeb8`. On disk `+0x71` is the continent id
    /// (a single byte, high bytes zero); at runtime the loader swizzles it into
    /// a continent-object pointer. Read as the low byte so it works pre-swizzle.
    pub fn continent_id(&self) -> i32 {
        u8_at(self.raw, 0x71) as i32
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

    // ---- Nation record remainder (VERIFIED via value-pattern scan across
    //      all 213 shipped nations, editor decode agent 2026-08-30) ----

    /// 3-letter FIFA code ("ENG", "BRA", "FRO", "SMR"…).
    pub fn three_letter_name(&self) -> String {
        read_latin1_cstr(self.raw, 0x53, 4)
    }
    /// Nationality adjective ("English", "Brazilian"…).
    pub fn nationality_name(&self) -> String {
        read_latin1_cstr(self.raw, 0x57, 26)
    }
    /// Finer regional grouping (0..24).
    pub fn region(&self) -> u8 { u8_at(self.raw, 0x75) }
    /// Coarser continental region (0..14).
    pub fn actual_region(&self) -> u8 { u8_at(self.raw, 0x76) }
    /// Day-of-the-cycle when the nation's season updates (0..63).
    pub fn season_update_day(&self) -> u8 { u8_at(self.raw, 0x77) }
    /// Grammatical gender of full name (0..3).
    pub fn name_gender(&self) -> u8 { u8_at(self.raw, 0x7e) }
    /// Grammatical gender of short name (0..2).
    pub fn short_name_gender(&self) -> u8 { u8_at(self.raw, 0x7f) }
    /// Capital city id (into city.dat). None for -1/-2 sentinels.
    pub fn capital_city_id(&self) -> Option<i32> {
        let v = le_i32(self.raw, 0x80);
        if v < 0 { None } else { Some(v) }
    }
    /// League-quality tier (0..4). Top-flight nations = 1; minnows = 4.
    pub fn league_standard(&self) -> u8 { u8_at(self.raw, 0x84) }
    /// State of development (0..20). Classic CM slider — Afg=1, Eng=19, Spa=20.
    pub fn state_of_development(&self) -> u8 { u8_at(self.raw, 0x85) }
    /// National stadium id (into stadium.dat). None when unset.
    pub fn national_stadium_id(&self) -> Option<i32> {
        let v = le_i16(self.raw, 0x86) as i32;
        if v <= 0 { None } else { Some(v) }
    }
    /// Total affiliated clubs (Italy=13059, San Marino=4).
    pub fn number_clubs(&self) -> i32 { le_i32(self.raw, 0x88) }
    /// Total affiliated staff records (0 in shipped data; populated at runtime).
    pub fn number_staff(&self) -> i16 { le_i16(self.raw, 0x8c) }
    /// Reputation 0..9500 in 500-point steps. VERIFIED value pattern.
    pub fn reputation(&self) -> u16 { le_u16(self.raw, 0x8e) }
    /// Foreground colour ids 1..3 (into colour.dat).
    pub fn foreground_colour_1(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x90)) }
    pub fn foreground_colour_2(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x94)) }
    pub fn foreground_colour_3(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x98)) }
    /// Background colour ids 1..3 (into colour.dat).
    pub fn background_colour_1(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x9c)) }
    pub fn background_colour_2(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xa0)) }
    pub fn background_colour_3(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0xa4)) }
    /// FIFA coefficient by year_index 0=1991..6=1997. Only slot 0 populated in shipped data.
    pub fn fifa_coefficient(&self, year_index: usize) -> Option<f64> {
        if year_index >= 7 { return None; }
        Some(le_f64(self.raw, 0xa8 + year_index * 8))
    }
    /// UEFA coefficient by year_index 0=1991..5=1996. Non-UEFA nations read as 0.
    pub fn uefa_coefficient(&self, year_index: usize) -> Option<f64> {
        if year_index >= 6 { return None; }
        Some(le_f64(self.raw, 0xe0 + year_index * 8))
    }
    /// Rival nation ids 1..3 (into nation.dat).
    pub fn rival_nation_1(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x110)) }
    pub fn rival_nation_2(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x114)) }
    pub fn rival_nation_3(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x118)) }
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
/// **What is actually shipped** (VERIFIED via byte scan across all 390 shipped
/// club_comp records, 2026-08-30): `continent_id` is populated on 258/390
/// records (usually `2` = Europe); `nation_id` is populated on 345/390.
/// Foreground/background colour ids are still empty in `club_comp.dat` and
/// `nation_comp.dat`, so those two links do get patched up at runtime through
/// clubs (`club+0x53/0x57/0x5b/0x60`) — but continent/nation are real on disk
/// and safe to trust straight out of the loader.
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
    ///
    /// This offset is ONLY valid for the 107-byte `club_comp.dat` /
    /// `nation_comp.dat` layout. `staff_comp.dat` records are 101 bytes with no
    /// abbreviation field — use [`StaffCompetitionView::reputation`] for those.
    pub fn reputation(&self) -> i16 {
        le_u16(self.raw, 0x69) as i16
    }
}

/// A read-only, typed view over a `staff_comp.dat` record (101 bytes).
///
/// Same field ordering as [`CompetitionView`] BUT with no abbreviation field:
/// everything from `continent_id` onwards shifts down by 6 bytes.
pub struct StaffCompetitionView<'a> {
    raw: &'a [u8],
}

impl<'a> StaffCompetitionView<'a> {
    pub const RECORD_SIZE: usize = 0x65;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self {
        Self { raw: &record.raw }
    }

    pub fn from_bytes(raw: &'a [u8]) -> Self {
        Self { raw }
    }

    pub fn id(&self) -> u32 { le_u32(self.raw, 0x00) }
    pub fn long_name(&self) -> String { read_latin1_cstr(self.raw, 0x04, 51) }
    pub fn long_name_gender(&self) -> i8 { i8_at(self.raw, 0x37) }
    pub fn short_name(&self) -> String { read_latin1_cstr(self.raw, 0x38, 26) }
    pub fn short_name_gender(&self) -> i8 { i8_at(self.raw, 0x52) }
    // No abbreviation field. Tail from here is 6 bytes earlier than CompetitionView.
    pub fn continent_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x53)) }
    pub fn nation_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x57)) }
    pub fn foreground_colour_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x5b)) }
    pub fn background_colour_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x5f)) }
    /// Competition reputation, 0..=20.
    pub fn reputation(&self) -> i16 { le_u16(self.raw, 0x63) as i16 }
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

// ------ Tactic file (.tct / .pct, 1428/1476 B) ------

/// Zero-copy view over a memory-mapped `.tct` or `.pct` tactic file.
/// Assumes the file has already been version-normalised to canonical v5E
/// (1472/1476 B). Full format decode in `reports/tactic_file_decode.md`.
pub struct TacticView<'a> {
    bytes: &'a [u8],
    is_packaged: bool,      // .pct = true, .tct = false
}

impl<'a> TacticView<'a> {
    pub const VERSION_TAG_V5E: u32 = 0x0098EC5E;
    pub const OBFUSCATE_MASK: u32 = 0x075BCD15;

    /// Wrap bytes assumed to be canonical v5E layout.
    /// `is_packaged` = true for `.pct`, false for `.tct` — controls whether
    /// the version tag is XOR-obfuscated (a `.pct` marker only).
    pub fn from_bytes(bytes: &'a [u8], is_packaged: bool) -> Self {
        Self { bytes, is_packaged }
    }

    pub fn version(&self) -> u32 {
        let raw = le_u32(self.bytes, 0);
        if self.is_packaged { raw.wrapping_sub(Self::OBFUSCATE_MASK) } else { raw }
    }

    /// Formation display name ("3-5-2", "4-4-2"…). Stored bit-inverted on
    /// disk; runtime memory holds it plain — this accessor undoes the
    /// inversion.
    pub fn formation_name(&self) -> String {
        let raw = &self.bytes[0x04..0x36];
        raw.iter()
            .map(|&b| if b == 0xff { 0xff } else { !b })
            .take_while(|&b| b != 0)
            .map(|b| b as char)
            .collect()
    }

    /// Author ASCII string embedded at file `+0x39` inside "area A".
    /// Stock presets are "Marc Vaughan" or "Paul Collyer".
    pub fn author(&self) -> String {
        let raw = &self.bytes[0x39..0x39 + 0x40];
        let n = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        String::from_utf8_lossy(&raw[..n]).to_string()
    }

    // --- Team-wide flag words ---
    pub fn team_flags_1(&self) -> u32 { le_u32(self.bytes, 0x00FE) }
    pub fn team_flags_2(&self) -> u32 { le_u32(self.bytes, 0x0559) }

    /// Team mentality 1..5 (Ultra-Def / Def / Normal / Att / All-Out-Att) —
    /// bits 0..4 of `team_flags_2`, verified from the loader's force-set code.
    pub fn mentality(&self) -> u8 { (self.team_flags_2() & 0x1F) as u8 }

    // --- Per-position accessors (11 slots) ---

    /// Position/role bitmask for slot `i` (one-hot over 12 roles GK/SW/D/DM/
    /// M/AM/ST/WB/RS/LS/C/FR). Matches `crate::tactics::position_rating`'s
    /// `position_mask` parameter.
    pub fn slot_role_mask(&self, i: usize) -> u16 {
        assert!(i < 11);
        le_u16(self.bytes, 0x0102 + i * 2)
    }
    pub fn slot_aux_role(&self, i: usize) -> u8 {
        assert!(i < 11);
        u8_at(self.bytes, 0x0118 + i)
    }
    pub fn slot_depth(&self, i: usize) -> u16 {
        assert!(i < 11);
        le_u16(self.bytes, 0x0123 + i * 2)
    }
    /// 96-byte per-slot instructions block (48 u16s: run-from-position,
    /// closing-down, marking-tightness, distribution, free-role, playmaker,
    /// target-man, forward-runs, hold-up-ball, passing-focus, cross-from,
    /// cross-target, long-shots — exact per-field u16 index still TBD).
    pub fn slot_body(&self, i: usize) -> &'a [u8] {
        assert!(i < 11);
        &self.bytes[0x0139 + i * 96 .. 0x0139 + (i + 1) * 96]
    }
    /// 8-byte per-slot pair — (u32 movement token, u32 flag=10 on legacy).
    pub fn slot_pair(&self, i: usize) -> (u32, u32) {
        assert!(i < 11);
        let o = 0x055D + i * 8;
        (le_u32(self.bytes, o), le_u32(self.bytes, o + 4))
    }
    /// Per-slot flag byte at `+0x5B5`. `0x11` = normal; higher bits toggled
    /// for free-role / attacking-fullback (verified on `3-5-2 AWE.tct`).
    pub fn slot_flag(&self, i: usize) -> u8 {
        assert!(i < 11);
        u8_at(self.bytes, 0x05B5 + i)
    }
}

// ------ Stadium (78 B, DAT_00acd5b8, stride 0x4e) ------

/// Typed view over a `stadium.dat` record (78 bytes).
///
/// Verified via loader `FUN_005121a0` case 5 + `FUN_0051b110` swizzle map.
/// The on-disk bytes are raw pool bytes with no unpacking; the runtime maps
/// `city_id`/`alt_stadium_id` into pool pointers, but the disk form is just IDs.
///
/// - `+0x00 u32` id
/// - `+0x04..+0x36` name (51-char Latin-1)
/// - `+0x37 u8` name_set_flag
/// - `+0x38 i32` city_id (swizzled to city pool)
/// - `+0x3c u32` capacity_total (verified: Old Trafford = 67800)
/// - `+0x40 u32` capacity_seated
/// - `+0x44 u32` capacity_expansion (verified: Old Trafford → 100000)
/// - `+0x48 i32` alt_stadium_id (self-referential, "replacement stadium" slot)
pub struct StadiumView<'a> {
    raw: &'a [u8],
}

impl<'a> StadiumView<'a> {
    pub const RECORD_SIZE: usize = 0x4e;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self { Self { raw: &record.raw } }
    pub fn from_bytes(raw: &'a [u8]) -> Self { Self { raw } }

    pub fn id(&self) -> i32 { le_i32(self.raw, 0x00) }
    pub fn name(&self) -> String { read_latin1_cstr(self.raw, 0x04, 0x33) }
    pub fn name_set(&self) -> bool { u8_at(self.raw, 0x37) == 0xff }
    pub fn city_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x38)) }
    pub fn capacity_total(&self) -> u32 { le_u32(self.raw, 0x3c) }
    pub fn capacity_seated(&self) -> u32 { le_u32(self.raw, 0x40) }
    pub fn capacity_expansion(&self) -> u32 { le_u32(self.raw, 0x44) }
    pub fn alt_stadium_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x48)) }
}

// ------ City (56 B, DAT_00acd5b4, stride 0x38) ------

/// Typed view over a `city.dat` record (56 bytes). No swizzle points; all
/// fields are already values on disk.
///
/// - `+0x00 u32` id
/// - `+0x04..+0x1c` name (25-char Latin-1)
/// - `+0x1e u8` name_set_flag
/// - `+0x1f u8` nation_id (verified: London=60=England)
/// - `+0x23 f64` latitude (unaligned; verified: London 51.519°N)
/// - `+0x2b f64` longitude (unaligned; verified: London -0.102°E)
/// - `+0x33 u8` size_tier (0..20, label unverified)
/// - `+0x34 i32` region_or_primary_club (label unverified)
pub struct CityView<'a> {
    raw: &'a [u8],
}

impl<'a> CityView<'a> {
    pub const RECORD_SIZE: usize = 0x38;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self { Self { raw: &record.raw } }
    pub fn from_bytes(raw: &'a [u8]) -> Self { Self { raw } }

    pub fn id(&self) -> i32 { le_i32(self.raw, 0x00) }
    pub fn name(&self) -> String { read_latin1_cstr(self.raw, 0x04, 0x1a) }
    pub fn name_set(&self) -> bool { u8_at(self.raw, 0x1e) == 0xff }
    pub fn nation_id(&self) -> u8 { u8_at(self.raw, 0x1f) }
    pub fn latitude(&self) -> f64 { le_f64(self.raw, 0x23) }
    pub fn longitude(&self) -> f64 { le_f64(self.raw, 0x2b) }
    pub fn size_tier(&self) -> u8 { u8_at(self.raw, 0x33) }
    pub fn region_or_primary_club(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x34)) }
}

// ------ Chairman attributes subrecord (at chairman_staff[+0x69]) ------

/// Read-only view over the chairman-attribute subrecord.
///
/// The chairman itself is a staff pointer at `ClubView::chairman_id`
/// (club record `+0xBF`). The exe stores the chairman's personality bytes
/// on a subrecord at `staff[+0x69]`, and rerolls them in the silent-
/// takeover event (`FUN_00588840`:82-95) as `rand(20)+1` (uniform 1..20).
/// Every byte is `i8` in the 1..=20 range.
///
/// Semantics decoded from reads elsewhere in the finance cluster
/// (`reports/finance_chairman_decode.md` §4):
/// - `+0x0f` = ambition. VERIFIED (`FUN_005884a0`:0058856d gates
///   whether the chairman initiates a takeover-of-another-club).
/// - `+0x20` = charisma / persuasion. VERIFIED (`FUN_005884a0`:0058858b
///   `FUN_008fc4f0(5) + 10 <= chairman[+0x20]` — negotiation gate; also
///   `FUN_00587c40`:0x00587d33 board-debt-payment gate; also
///   `FUN_00586ec0`:0x00587375 wage-cap decisions).
/// - `+0x16` = OPEN GAP — not read by any decoded finance fn. Rerolled
///   1..20. Speculative: cash-consciousness / patience.
/// - `+0x1d` = OPEN GAP — not read by any decoded finance fn. Rerolled
///   1..20. Speculative: resolve / loyalty.
///
/// Post-reroll charisma (`+0x20`) has an implicit floor: if it lands
/// 1..4 the reroll fires once more, biasing upward but still allowing
/// low values with p = (4/20)² = 4%.
pub struct ChairmanAttrsView<'a> {
    raw: &'a [u8],
}

impl<'a> ChairmanAttrsView<'a> {
    pub fn from_bytes(raw: &'a [u8]) -> Self { Self { raw } }

    /// `+0x0f` — ambition (1..=20). VERIFIED.
    pub fn ambition(&self) -> i8 { i8_at(self.raw, 0x0f) }
    /// `+0x16` — OPEN GAP (no decoded consumer). Rerolled 1..=20.
    pub fn attr_16(&self)  -> i8 { i8_at(self.raw, 0x16) }
    /// `+0x1d` — OPEN GAP (no decoded consumer). Rerolled 1..=20.
    pub fn attr_1d(&self)  -> i8 { i8_at(self.raw, 0x1d) }
    /// `+0x20` — charisma / persuasion (1..=20). VERIFIED.
    pub fn charisma(&self) -> i8 { i8_at(self.raw, 0x20) }
}

// ------ Official (43 B, DAT_00acd5f0, stride 0x2b) ------

/// Typed view over an `officials.dat` record (43 bytes) — a match referee.
///
/// - `+0x00 u32` id
/// - `+0x04 i32` first_name_id  (swizzled to first-name pool)
/// - `+0x08 i32` second_name_id (swizzled to second-name pool)
/// - `+0x0c u16` dob_day (0..364)
/// - `+0x0e u16` dob_year (verified vs `0068f0d0`: `year = current_year - age`)
/// - `+0x10 u32` flags (=1 in shipped data)
/// - `+0x16 i32` nation_id
/// - `+0x1a i32` home_city_id
/// - `+0x1e u32` reputation_ca (label unverified)
/// - `+0x22 u16` reputation_pa (label unverified)
/// - `+0x24..+0x2a` seven 0..20 rating bytes (individual meanings unresolved)
pub struct OfficialView<'a> {
    raw: &'a [u8],
}

impl<'a> OfficialView<'a> {
    pub const RECORD_SIZE: usize = 0x2b;

    pub fn new(record: &'a DomainOpaqueRecord) -> Self { Self { raw: &record.raw } }
    pub fn from_bytes(raw: &'a [u8]) -> Self { Self { raw } }

    pub fn id(&self) -> i32 { le_i32(self.raw, 0x00) }
    pub fn first_name_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x04)) }
    pub fn second_name_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x08)) }
    pub fn dob_day(&self) -> u16 { le_u16(self.raw, 0x0c) }
    pub fn dob_year(&self) -> u16 { le_u16(self.raw, 0x0e) }
    pub fn flags(&self) -> u32 { le_u32(self.raw, 0x10) }
    pub fn nation_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x16)) }
    pub fn home_city_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x1a)) }
    pub fn reputation_ca(&self) -> u32 { le_u32(self.raw, 0x1e) }
    pub fn reputation_pa(&self) -> u16 { le_u16(self.raw, 0x22) }
    pub fn rating_bytes(&self) -> [u8; 7] {
        let s = self.raw;
        [u8_at(s, 0x24), u8_at(s, 0x25), u8_at(s, 0x26),
         u8_at(s, 0x27), u8_at(s, 0x28), u8_at(s, 0x29),
         u8_at(s, 0x2a)]
    }
}

// ------ History records (staff_history, club_comp_history, nation_comp_history, staff_comp_history) ------
//
// Full decode evidence in `reports/history_records_decode.md`. Loader: the
// four adjacent sections in `FUN_005176c0` around lines 1660–2530 that call
// per-record readers `FUN_00539a30` (0x11), `FUN_00539f00` (0x1a) and
// `FUN_0053a1a0` (0x3a). Type ids in `index.dat` are 0x11 (staff_history),
// 0x12 (staff_comp_history), 0x13 (club_comp_history), 0x14 (nation_comp_history).

/// A read-only, typed view over a `staff_history.dat` record (17 bytes).
///
/// One row per (person, competition, season). Semantics VERIFIED from the
/// consumer at `FUN_007a8090` around lines 260–360: the sort key is `+4`
/// (person), then short `+8` (season year), then `+0` (record id); the
/// per-record totals `local_830 += *(byte*)(+0xf)` and
/// `local_82c += *(byte*)(+0x10)` group into `FUN_00449590(person, comp,
/// apps_sum, goals_sum)`. The "Too many goals for goalkeeper" guard
/// (`5 < *(byte*)(+0x10)`) fixes `+0x10 = goals`, so `+0xf = apps` and
/// `+0xe` is the substitute-appearance / unused byte.
pub struct StaffHistoryView<'a> {
    raw: &'a [u8],
}

impl<'a> StaffHistoryView<'a> {
    pub const RECORD_SIZE: usize = 0x11; // 17

    pub fn new(record: &'a DomainOpaqueRecord) -> Self { Self { raw: &record.raw } }
    pub fn from_bytes(raw: &'a [u8]) -> Self { Self { raw } }

    /// Row id, unique per file (0..N-1 in shipped data).
    pub fn id(&self) -> u32 { le_u32(self.raw, 0x00) }
    /// `person.dat` / `staff.dat` id (game code resolves this to a person
    /// pointer, then reads `+0x61` for the club that season).
    pub fn person_id(&self) -> u32 { le_u32(self.raw, 0x04) }
    /// Season start year (e.g. 1984 = 0x07C0).
    pub fn year(&self) -> u16 { le_u16(self.raw, 0x08) }
    /// `club_comp.dat` id (competition).
    pub fn competition_id(&self) -> u32 { le_u32(self.raw, 0x0a) }
    /// Byte at `+0xe`. Substitute-appearances is the strong guess (byte-wide
    /// counter beside apps and goals); UNVERIFIED.
    pub fn subs(&self) -> u8 { u8_at(self.raw, 0x0e) }
    /// Appearances. VERIFIED (sanity clamp at consumer: apps==0 forces goals=0).
    pub fn apps(&self) -> u8 { u8_at(self.raw, 0x0f) }
    /// Goals. VERIFIED (goalkeeper cap: `5 < goals` triggers a data-error).
    pub fn goals(&self) -> u8 { u8_at(self.raw, 0x10) }
}

/// A read-only, typed view over a `club_comp_history.dat` record (26 bytes).
///
/// One row per (competition, season) with the top-4 finishers. Semantics
/// VERIFIED from the honours screen `FUN_0049eb30`: it sorts by
/// `**(int**)(+4)` (competition pointer) then short at `+8` (year), then
/// prints `local_10 + 10 → Winners`, `+ 0xe → Runners-up`,
/// `+ 0x12 → Third Place` / "Minor Premiers" (Australian domestic), and
/// `+ 0x16 → Hosts` — each field is a club pointer, `-1` = "unused".
///
/// SPOT CHECK (verified): row 0 = comp 7 (English Premier Division), year
/// 0x0761 = 1889, winner club 7269 = Preston North End, runner-up club
/// 730 = Aston Villa. Matches the historical 1888-89 Football League: Preston
/// "The Invincibles" won it, Aston Villa were runners-up.
pub struct ClubCompHistoryView<'a> {
    raw: &'a [u8],
}

impl<'a> ClubCompHistoryView<'a> {
    pub const RECORD_SIZE: usize = 0x1a; // 26

    pub fn new(record: &'a DomainOpaqueRecord) -> Self { Self { raw: &record.raw } }
    pub fn from_bytes(raw: &'a [u8]) -> Self { Self { raw } }

    pub fn id(&self) -> u32 { le_u32(self.raw, 0x00) }
    pub fn competition_id(&self) -> u32 { le_u32(self.raw, 0x04) }
    pub fn year(&self) -> u16 { le_u16(self.raw, 0x08) }
    pub fn winner_club_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x0a)) }
    pub fn runner_up_club_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x0e)) }
    /// Third place, or "Minor Premiers" for competitions that use that
    /// concept (Australian A-League etc.).
    pub fn third_place_club_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x12)) }
    pub fn hosts_club_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x16)) }
}

/// A read-only, typed view over a `nation_comp_history.dat` record (26 bytes).
///
/// Identical byte layout to [`ClubCompHistoryView`] — the same 26-byte reader
/// (`FUN_00539f00`, stride `0x1a`) and the same honours screen
/// `FUN_0049eb30` consume both. The only difference is that the pointer
/// fields refer to national-team entities rather than clubs. In the shipped
/// data those winner values (e.g. row 0 = comp 408 African Cup of Nations,
/// year 0x07A5 = 1957, winner id 10638) are neither `nation.dat` ids nor
/// `club.dat` ids (max club id = 10579), so the referenced pool is the
/// runtime national-teams table built on top of the base data.
pub struct NationCompHistoryView<'a> {
    raw: &'a [u8],
}

impl<'a> NationCompHistoryView<'a> {
    pub const RECORD_SIZE: usize = 0x1a; // 26

    pub fn new(record: &'a DomainOpaqueRecord) -> Self { Self { raw: &record.raw } }
    pub fn from_bytes(raw: &'a [u8]) -> Self { Self { raw } }

    pub fn id(&self) -> u32 { le_u32(self.raw, 0x00) }
    pub fn competition_id(&self) -> u32 { le_u32(self.raw, 0x04) }
    pub fn year(&self) -> u16 { le_u16(self.raw, 0x08) }
    pub fn winner_team_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x0a)) }
    pub fn runner_up_team_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x0e)) }
    pub fn third_place_team_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x12)) }
    pub fn hosts_team_id(&self) -> Option<i32> { id_opt(le_i32(self.raw, 0x16)) }
}

/// A read-only, typed view over a `staff_comp_history.dat` record (58 bytes).
///
/// Loader: `FUN_0053a1a0` bulk-reads `0x3a` bytes; base pointer
/// `DAT_00acd5ec`, count `DAT_00acd594`. The header (`+0x00..+0x0a`) is
/// verified from the same file family (id / person / year) and matches the
/// on-disk pattern of the shipped rows (record ids sequential 0..N-1, the
/// second u32 groups adjacent rows by person, the u16 at `+8` is a plausible
/// year, e.g. `0x07A4 = 1956`).
///
/// The 48-byte tail is a bank of twelve u32s that in the shipped data mixes
/// counters and `0xFFFF_FFFF` "empty" sentinels — very likely a
/// per-season managerial stats block (played / won / drawn / lost / for /
/// against split by home / away or by league / cup / continental). We did
/// not find a consumer site of `DAT_00acd5ec` in the decompile that pins
/// individual byte offsets, so the twelve u32s are exposed as an array and
/// left named `slot_N` until a downstream screen (Manager history) is
/// decoded to bind them.
pub struct StaffCompHistoryView<'a> {
    raw: &'a [u8],
}

impl<'a> StaffCompHistoryView<'a> {
    pub const RECORD_SIZE: usize = 0x3a; // 58

    pub fn new(record: &'a DomainOpaqueRecord) -> Self { Self { raw: &record.raw } }
    pub fn from_bytes(raw: &'a [u8]) -> Self { Self { raw } }

    /// Row id, sequential in the shipped file.
    pub fn id(&self) -> u32 { le_u32(self.raw, 0x00) }
    /// `person.dat` / `staff.dat` id (rows are grouped by this in the shipped file).
    pub fn person_id(&self) -> u32 { le_u32(self.raw, 0x04) }
    /// Season start year.
    pub fn year(&self) -> u16 { le_u16(self.raw, 0x08) }
    /// Read one of the twelve trailing u32 slots at `+0x0a + i*4`.
    /// Sentinel `0xFFFFFFFF` means "empty" for the id-shaped slots.
    pub fn slot(&self, i: usize) -> u32 {
        assert!(i < 12);
        le_u32(self.raw, 0x0a + i * 4)
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
