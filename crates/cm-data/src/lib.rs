//! CM0102 data model: `.dat` containers, `index.dat`, `.sav` sections, and table catalog.
//!
//! Provenance (carve: `D:/cm0102-carve`):
//! * Record I/O convention <- `db_files.cpp` wrappers (VERIFIED): `.dat` files are
//!   headerless arrays of fixed-size records; `fread(buf, RECSIZE, count, file)`.
//! * Record sizes <- the `fread` literals, cross-checked by exact file division where closed.
//! * `index.dat` manifest <- reverse-engineered entry format (see [`Manifest`]).
//! * `.sav` container <- `EXECUTION_MODEL.md`: version + named sections whose payloads
//!   reuse the base database record layouts.
//!
//! HONEST SCOPE:
//! * Every logical data table now has an explicit Rust identity in [`TableId`].
//! * Tables with verified fixed record sizes use [`RecordKind`].
//! * Tables whose field layout or exact record size is still unresolved are loaded as
//!   named raw blobs rather than guessed structs.

#![forbid(unsafe_code)]

use std::fs;
use std::io;
use std::path::Path;

const SAVE_SECTION_ENTRY_SIZE: usize = 0x10c;
const SAVE_SECTION_NAME_SIZE: usize = 0x104;

/// A base `.dat` record type and its VERIFIED on-disk size (bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    Club,      // 581 (0x245) <- 0x00537730, club.dat, 10580 recs
    Nation,    // 290 (0x122) <- 0x00537320, nation.dat, 213 recs
    Continent, // 198 (0xc6)  <- 0x005371c0, continent.dat, 6 recs
    Colour,    // 58  (0x3a)  <- corrected current workspace value, colour.dat, 34 recs
}

impl RecordKind {
    pub const fn size(self) -> usize {
        match self {
            RecordKind::Club => 581,
            RecordKind::Nation => 290,
            RecordKind::Continent => 198,
            RecordKind::Colour => 58,
        }
    }

    pub fn from_section_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("club.dat") || name.eq_ignore_ascii_case("nat_club.dat") {
            Some(Self::Club)
        } else if name.eq_ignore_ascii_case("nation.dat") {
            Some(Self::Nation)
        } else if name.eq_ignore_ascii_case("continent.dat") {
            Some(Self::Continent)
        } else if name.eq_ignore_ascii_case("colour.dat") {
            Some(Self::Colour)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordLayoutConfidence {
    Verified,
    Inferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedRecordLayout {
    pub size: usize,
    pub confidence: RecordLayoutConfidence,
}

impl FixedRecordLayout {
    pub const fn verified(kind: RecordKind) -> Self {
        Self {
            size: kind.size(),
            confidence: RecordLayoutConfidence::Verified,
        }
    }

    pub const fn inferred(size: usize) -> Self {
        Self {
            size,
            confidence: RecordLayoutConfidence::Inferred,
        }
    }
}

/// A headerless fixed-size-record `.dat` file: `count` records back to back.
#[derive(Debug)]
pub struct DatFile<'a> {
    kind: RecordKind,
    bytes: &'a [u8],
}

impl<'a> DatFile<'a> {
    pub fn new(kind: RecordKind, bytes: &'a [u8]) -> io::Result<Self> {
        if bytes.len() % kind.size() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} bytes is not a multiple of record size {}",
                    bytes.len(),
                    kind.size()
                ),
            ));
        }
        Ok(Self { kind, bytes })
    }

    pub fn count(&self) -> usize {
        self.bytes.len() / self.kind.size()
    }

    pub fn record(&self, i: usize) -> Option<&'a [u8]> {
        let sz = self.kind.size();
        self.bytes.get(i * sz..i * sz + sz)
    }

    pub fn records(&self) -> impl Iterator<Item = &'a [u8]> + '_ {
        (0..self.count()).map(move |i| self.record(i).unwrap())
    }
}

/// A fixed-size record table whose record meaning is not yet fully lifted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedRecordTable<'a> {
    layout: FixedRecordLayout,
    bytes: &'a [u8],
}

impl<'a> FixedRecordTable<'a> {
    pub fn new(layout: FixedRecordLayout, bytes: &'a [u8]) -> io::Result<Self> {
        if bytes.len() % layout.size != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} bytes is not a multiple of inferred record size {}",
                    bytes.len(),
                    layout.size
                ),
            ));
        }
        Ok(Self { layout, bytes })
    }

    pub fn layout(&self) -> FixedRecordLayout {
        self.layout
    }

    pub fn count(&self) -> usize {
        self.bytes.len() / self.layout.size
    }

    pub fn record(&self, i: usize) -> Option<&'a [u8]> {
        let sz = self.layout.size;
        self.bytes.get(i * sz..i * sz + sz)
    }

    pub fn records(&self) -> impl Iterator<Item = &'a [u8]> + '_ {
        (0..self.count()).map(move |i| self.record(i).unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CityRecord<'a> {
    pub bytes: &'a [u8],
}

impl<'a> CityRecord<'a> {
    pub const NAME_RANGE: std::ops::Range<usize> = 4..30;
    pub const TAIL_START: usize = 30;

    pub fn id(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn name(&self) -> String {
        read_c_string_latin1(&self.bytes[Self::NAME_RANGE])
    }

    pub fn unknown_tail(&self) -> &'a [u8] {
        &self.bytes[Self::NAME_RANGE.end..]
    }

    pub fn tail_u16(&self, index: usize) -> Option<u16> {
        let off = Self::TAIL_START + index * 2;
        let bytes = self.bytes.get(off..off + 2)?;
        Some(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn tail_u32(&self, index: usize) -> Option<u32> {
        let off = 32 + index * 4;
        let bytes = self.bytes.get(off..off + 4)?;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn to_entry(&self) -> CityEntry {
        let mut tail_u16 = [0u16; 13];
        for (i, slot) in tail_u16.iter_mut().enumerate() {
            *slot = self.tail_u16(i).unwrap_or(0);
        }

        let mut tail_u32 = [0u32; 6];
        for (i, slot) in tail_u32.iter_mut().enumerate() {
            *slot = self.tail_u32(i).unwrap_or(0);
        }

        CityEntry {
            id: self.id(),
            name: self.name(),
            tail_u16,
            tail_u32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfficialRecord<'a> {
    pub bytes: &'a [u8],
}

impl<'a> OfficialRecord<'a> {
    pub fn id(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn raw_fields(&self) -> &'a [u8] {
        &self.bytes[4..]
    }

    pub fn u32_slot(&self, index: usize) -> Option<u32> {
        let off = index * 4;
        let bytes = self.bytes.get(off..off + 4)?;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn u16_slot(&self, index: usize) -> Option<u16> {
        let off = index * 2;
        let bytes = self.bytes.get(off..off + 2)?;
        Some(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn trailing_byte(&self) -> u8 {
        self.bytes[self.bytes.len() - 1]
    }

    pub fn to_entry(&self) -> OfficialEntry {
        let mut u32_slots = [0u32; 10];
        for (i, slot) in u32_slots.iter_mut().enumerate() {
            *slot = self.u32_slot(i).unwrap_or(0);
        }

        let mut u16_slots = [0u16; 21];
        for (i, slot) in u16_slots.iter_mut().enumerate() {
            *slot = self.u16_slot(i).unwrap_or(0);
        }

        OfficialEntry {
            id: self.id(),
            u32_slots,
            u16_slots,
            trailing_byte: self.trailing_byte(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameRecord<'a> {
    pub bytes: &'a [u8],
}

impl<'a> NameRecord<'a> {
    pub const TEXT_END: usize = 48;

    pub fn text(&self) -> String {
        read_c_string_latin1(&self.bytes[..Self::TEXT_END])
    }

    pub fn unknown_footer(&self) -> &'a [u8] {
        &self.bytes[Self::TEXT_END..]
    }

    pub fn to_entry(&self) -> NameEntry {
        let mut footer = [0u8; 12];
        footer.copy_from_slice(self.unknown_footer());
        NameEntry {
            text: self.text(),
            footer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StadiumRecord<'a> {
    pub bytes: &'a [u8],
}

impl<'a> StadiumRecord<'a> {
    pub fn id(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn name(&self) -> String {
        read_c_string_latin1(&self.bytes[4..56])
    }

    pub fn unknown_tail(&self) -> &'a [u8] {
        &self.bytes[56..]
    }

    pub fn to_entry(&self) -> StadiumEntry {
        StadiumEntry {
            id: self.id(),
            name: self.name(),
            unknown_tail: self.unknown_tail().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompetitionRecord<'a, const N: usize, const NAME_B: usize, const SHORT_B: usize> {
    pub bytes: &'a [u8],
}

impl<'a, const N: usize, const NAME_B: usize, const SHORT_B: usize>
    CompetitionRecord<'a, N, NAME_B, SHORT_B>
{
    pub fn id(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn long_name(&self) -> String {
        read_c_string_latin1(&self.bytes[4..NAME_B])
    }

    pub fn short_name(&self) -> String {
        read_c_string_latin1(&self.bytes[56..SHORT_B])
    }

    pub fn unknown_tail(&self) -> &'a [u8] {
        &self.bytes[SHORT_B..N]
    }

    fn i32_at(&self, off: usize) -> i32 {
        self.bytes
            .get(off..off + 4)
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .unwrap_or(0)
    }

    fn u16_at(&self, off: usize) -> u16 {
        self.bytes
            .get(off..off + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .unwrap_or(0)
    }

    /// Three-letter abbreviation at +0x53 (fixed 3-byte field).
    pub fn three_letter_name(&self) -> String {
        read_c_string_latin1(&self.bytes[0x53..0x56.min(N)])
    }

    /// Decode the numeric field block for the 107-byte club/nation-comp
    /// layout. Offsets (verified vs raw bytes + official editor field
    /// labels): scope +0x59, nation +0x5d, last_division +0x61,
    /// reserve_division +0x65, reputation +0x69. Only valid when N == 107;
    /// the 101-byte staff-comp layout differs and returns zeros here (its
    /// fields aren't used for league scheduling).
    pub fn to_entry(&self) -> CompetitionEntry {
        let is_107 = N == 107;
        CompetitionEntry {
            id: self.id(),
            long_name: self.long_name(),
            short_name: self.short_name(),
            three_letter_name: if is_107 { self.three_letter_name() } else { String::new() },
            scope: if is_107 { self.i32_at(0x59) } else { 0 },
            nation_id: if is_107 { self.i32_at(0x5d) } else { -1 },
            last_division: if is_107 { self.i32_at(0x61) } else { -1 },
            reserve_division: if is_107 { self.i32_at(0x65) } else { -1 },
            reputation: if is_107 { self.u16_at(0x69) } else { 0 },
            unknown_tail: self.unknown_tail().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct History17Record<'a> {
    pub bytes: &'a [u8],
}

impl<'a> History17Record<'a> {
    pub fn u32_slot(&self, index: usize) -> Option<u32> {
        let off = index * 4;
        let bytes = self.bytes.get(off..off + 4)?;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn trailing_byte(&self) -> u8 {
        self.bytes[16]
    }

    pub fn to_entry(&self) -> History17Entry {
        let mut u32_slots = [0u32; 4];
        for (i, slot) in u32_slots.iter_mut().enumerate() {
            *slot = self.u32_slot(i).unwrap_or(0);
        }
        History17Entry {
            id: u32_slots[0],
            u32_slots,
            trailing_byte: self.trailing_byte(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct History26Record<'a> {
    pub bytes: &'a [u8],
}

impl<'a> History26Record<'a> {
    pub fn u32_slot(&self, index: usize) -> Option<u32> {
        let off = index * 4;
        let bytes = self.bytes.get(off..off + 4)?;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn trailing_u16(&self) -> u16 {
        u16::from_le_bytes([self.bytes[24], self.bytes[25]])
    }

    pub fn to_entry(&self) -> History26Entry {
        let mut u32_slots = [0u32; 6];
        for (i, slot) in u32_slots.iter_mut().enumerate() {
            *slot = self.u32_slot(i).unwrap_or(0);
        }
        History26Entry {
            u32_slots,
            trailing_u16: self.trailing_u16(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct History58Record<'a> {
    pub bytes: &'a [u8],
}

impl<'a> History58Record<'a> {
    pub fn u32_slot(&self, index: usize) -> Option<u32> {
        let off = index * 4;
        let bytes = self.bytes.get(off..off + 4)?;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn trailing_u16(&self) -> u16 {
        u16::from_le_bytes([self.bytes[56], self.bytes[57]])
    }

    pub fn to_entry(&self) -> History58Entry {
        let mut u32_slots = [0u32; 14];
        for (i, slot) in u32_slots.iter_mut().enumerate() {
            *slot = self.u32_slot(i).unwrap_or(0);
        }
        History58Entry {
            u32_slots,
            trailing_u16: self.trailing_u16(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffType6Record<'a> {
    pub bytes: &'a [u8],
}

impl<'a> StaffType6Record<'a> {
    pub fn id(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn body(&self) -> &'a [u8] {
        &self.bytes[4..]
    }

    pub fn to_entry(&self) -> StaffType6Entry {
        StaffType6Entry {
            id: self.id(),
            body: self.body().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffType9Record<'a> {
    pub bytes: &'a [u8],
}

impl<'a> StaffType9Record<'a> {
    pub fn id(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn body(&self) -> &'a [u8] {
        &self.bytes[4..]
    }

    pub fn to_entry(&self) -> StaffType9Entry {
        StaffType9Entry {
            id: self.id(),
            body: self.body().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffType10Record<'a> {
    pub bytes: &'a [u8],
}

impl<'a> StaffType10Record<'a> {
    pub fn id(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn unknown_byte_4(&self) -> u8 {
        self.bytes[4]
    }

    pub fn rating_short_0x05(&self) -> u16 {
        u16::from_le_bytes([self.bytes[5], self.bytes[6]])
    }

    pub fn rating_short_0x07(&self) -> u16 {
        u16::from_le_bytes([self.bytes[7], self.bytes[8]])
    }

    pub fn unknown_bytes_9_12(&self) -> [u8; 4] {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.bytes[9..13]);
        bytes
    }

    pub fn rating_short_0x0d(&self) -> u16 {
        u16::from_le_bytes([self.bytes[13], self.bytes[14]])
    }

    pub fn unknown_bytes_15_26(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        bytes.copy_from_slice(&self.bytes[15..27]);
        bytes
    }

    pub fn attributes(&self) -> [u8; 31] {
        let mut attrs = [0u8; 31];
        attrs.copy_from_slice(&self.bytes[0x1b..0x1b + 31]);
        attrs
    }

    pub fn unknown_bytes_58_64(&self) -> [u8; 7] {
        let mut bytes = [0u8; 7];
        bytes.copy_from_slice(&self.bytes[58..65]);
        bytes
    }

    pub fn trailing_bytes(&self) -> [u8; 5] {
        let mut tail = [0u8; 5];
        tail.copy_from_slice(&self.bytes[65..70]);
        tail
    }

    pub fn to_entry(&self) -> StaffType10Entry {
        StaffType10Entry {
            id: self.id(),
            unknown_byte_4: self.unknown_byte_4(),
            rating_short_0x05: self.rating_short_0x05(),
            rating_short_0x07: self.rating_short_0x07(),
            unknown_bytes_9_12: self.unknown_bytes_9_12(),
            rating_short_0x0d: self.rating_short_0x0d(),
            unknown_bytes_15_26: self.unknown_bytes_15_26(),
            attributes: self.attributes(),
            unknown_bytes_58_64: self.unknown_bytes_58_64(),
            trailing_bytes: self.trailing_bytes(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityTable<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> CityTable<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(56);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn record(&self, i: usize) -> Option<CityRecord<'a>> {
        self.inner.record(i).map(|bytes| CityRecord { bytes })
    }

    pub fn entries(&self) -> Vec<CityEntry> {
        self.inner
            .records()
            .map(|bytes| CityRecord { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialsTable<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> OfficialsTable<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(43);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn record(&self, i: usize) -> Option<OfficialRecord<'a>> {
        self.inner.record(i).map(|bytes| OfficialRecord { bytes })
    }

    pub fn entries(&self) -> Vec<OfficialEntry> {
        self.inner
            .records()
            .map(|bytes| OfficialRecord { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameTable<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> NameTable<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(60);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn record(&self, i: usize) -> Option<NameRecord<'a>> {
        self.inner.record(i).map(|bytes| NameRecord { bytes })
    }

    pub fn entries(&self) -> Vec<NameEntry> {
        self.inner
            .records()
            .map(|bytes| NameRecord { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StadiumTable<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> StadiumTable<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(78);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn record(&self, i: usize) -> Option<StadiumRecord<'a>> {
        self.inner.record(i).map(|bytes| StadiumRecord { bytes })
    }

    pub fn entries(&self) -> Vec<StadiumEntry> {
        self.inner
            .records()
            .map(|bytes| StadiumRecord { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompetitionTable<'a, const N: usize, const NAME_B: usize, const SHORT_B: usize> {
    inner: FixedRecordTable<'a>,
}

impl<'a, const N: usize, const NAME_B: usize, const SHORT_B: usize>
    CompetitionTable<'a, N, NAME_B, SHORT_B>
{
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(N);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn record(&self, i: usize) -> Option<CompetitionRecord<'a, N, NAME_B, SHORT_B>> {
        self.inner
            .record(i)
            .map(|bytes| CompetitionRecord { bytes })
    }

    pub fn entries(&self) -> Vec<CompetitionEntry> {
        self.inner
            .records()
            .map(|bytes| CompetitionRecord::<N, NAME_B, SHORT_B> { bytes }.to_entry())
            .collect()
    }
}

pub type StaffCompTable<'a> = CompetitionTable<'a, 101, 56, 101>;
pub type ClubCompTable<'a> = CompetitionTable<'a, 107, 56, 104>;
pub type NationCompTable<'a> = CompetitionTable<'a, 107, 56, 104>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History17Table<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> History17Table<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(17);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn entries(&self) -> Vec<History17Entry> {
        self.inner
            .records()
            .map(|bytes| History17Record { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History26Table<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> History26Table<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(26);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn entries(&self) -> Vec<History26Entry> {
        self.inner
            .records()
            .map(|bytes| History26Record { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History58Table<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> History58Table<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(58);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn entries(&self) -> Vec<History58Entry> {
        self.inner
            .records()
            .map(|bytes| History58Record { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffType6Table<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> StaffType6Table<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(157);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn entries(&self) -> Vec<StaffType6Entry> {
        self.inner
            .records()
            .map(|bytes| StaffType6Record { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffType9Table<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> StaffType9Table<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(68);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn entries(&self) -> Vec<StaffType9Entry> {
        self.inner
            .records()
            .map(|bytes| StaffType9Record { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffType10Table<'a> {
    inner: FixedRecordTable<'a>,
}

impl<'a> StaffType10Table<'a> {
    pub const LAYOUT: FixedRecordLayout = FixedRecordLayout::inferred(70);

    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        Ok(Self {
            inner: FixedRecordTable::new(Self::LAYOUT, bytes)?,
        })
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn entries(&self) -> Vec<StaffType10Entry> {
        self.inner
            .records()
            .map(|bytes| StaffType10Record { bytes }.to_entry())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityEntry {
    pub id: u32,
    pub name: String,
    pub tail_u16: [u16; 13],
    pub tail_u32: [u32; 6],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialEntry {
    pub id: u32,
    pub u32_slots: [u32; 10],
    pub u16_slots: [u16; 21],
    pub trailing_byte: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameEntry {
    pub text: String,
    pub footer: [u8; 12],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StadiumEntry {
    pub id: u32,
    pub name: String,
    pub unknown_tail: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompetitionEntry {
    pub id: u32,
    pub long_name: String,
    pub short_name: String,
    /// Three-letter abbreviation (e.g. "PRM", "D1", "CON"). Present only on
    /// ranked domestic leagues — cups and non-manageable feeder divisions
    /// leave it blank. Verified against the official editor field
    /// `club_competition_three_letter_name` and raw bytes at record +0x53.
    pub three_letter_name: String,
    /// Scope word at +0x59 (2 = domestic, -2 on the global "A Lower Division"
    /// bucket which has no nation). Editor: "Scope".
    pub scope: i32,
    /// Nation id at +0x5d — the competition's country. POPULATED in the
    /// shipped data (e.g. England Premier = 60). Editor: "Nation".
    pub nation_id: i32,
    /// Promotion link at +0x61 (editor: "Last division"); -2 = none.
    pub last_division: i32,
    /// Relegation/reserve link at +0x65 (editor: "Reserve division"); -2 = none.
    pub reserve_division: i32,
    /// Reputation / league standard at +0x69 (editor: "Reputation" /
    /// "League standard"). England: Premier 18, First 12, Second 8, Third 4,
    /// Conference 3, feeders 2, bucket 1.
    pub reputation: u16,
    /// Kept for round-trip compatibility with the old parse (the final bytes).
    pub unknown_tail: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History17Entry {
    pub id: u32,
    pub u32_slots: [u32; 4],
    pub trailing_byte: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History26Entry {
    pub u32_slots: [u32; 6],
    pub trailing_u16: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History58Entry {
    pub u32_slots: [u32; 14],
    pub trailing_u16: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceData {
    pub cities: Vec<CityEntry>,
    pub officials: Vec<OfficialEntry>,
    pub first_names: Vec<NameEntry>,
    pub second_names: Vec<NameEntry>,
    pub common_names: Vec<NameEntry>,
    pub stadiums: Vec<StadiumEntry>,
    pub staff_competitions: Vec<CompetitionEntry>,
    pub club_competitions: Vec<CompetitionEntry>,
    pub nation_competitions: Vec<CompetitionEntry>,
    pub staff_history: Vec<History17Entry>,
    pub staff_comp_history: Vec<History58Entry>,
    pub club_comp_history: Vec<History26Entry>,
    pub nation_comp_history: Vec<History26Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffType6Entry {
    pub id: u32,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffType8Entry {
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffType9Entry {
    pub id: u32,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffType10Entry {
    pub id: u32,
    pub unknown_byte_4: u8,
    pub rating_short_0x05: u16,
    pub rating_short_0x07: u16,
    pub unknown_bytes_9_12: [u8; 4],
    pub rating_short_0x0d: u16,
    pub unknown_bytes_15_26: [u8; 12],
    pub attributes: [u8; 31],
    pub unknown_bytes_58_64: [u8; 7],
    pub trailing_bytes: [u8; 5],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffData {
    pub type6: Vec<StaffType6Entry>,
    pub type8: Vec<StaffType8Entry>,
    pub type9: Vec<StaffType9Entry>,
    pub type10: Vec<StaffType10Entry>,
}

/// One entry of `index.dat`: a data file the game loads, with its type and count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    pub filename: String,
    pub kind: u8,
    pub count: u32,
}

/// The parsed `Data/index.dat` manifest.
#[derive(Debug, Default)]
pub struct Manifest {
    pub entries: Vec<ManifestEntry>,
}

impl Manifest {
    /// Validated parse against the reverse-engineered `index.dat` entry mark:
    /// `3c f3 12 <type> 00 00 00 <count-le-u32>`.
    pub fn parse(bytes: &[u8]) -> Self {
        const MARK: &[u8] = &[0x3c, 0xf3, 0x12];
        let mut entries = Vec::new();
        let mut cursor = 0usize;
        while let Some(offset) = find(&bytes[cursor..], MARK).map(|p| cursor + p) {
            if offset + 11 > bytes.len() {
                break;
            }
            let kind = bytes[offset + 3];
            let count = u32::from_le_bytes([
                bytes[offset + 7],
                bytes[offset + 8],
                bytes[offset + 9],
                bytes[offset + 10],
            ]);

            if let Some(filename) = find_manifest_filename(bytes, offset) {
                entries.push(ManifestEntry {
                    filename,
                    kind,
                    count,
                });
            }

            cursor = offset + MARK.len();
        }

        // Keep the first entry for each known logical `(type, filename)` pair.
        // This drops ghost scans such as the old `p.dat` artifact without inventing
        // new tables.
        let mut deduped = Vec::with_capacity(entries.len());
        for entry in entries {
            let expected_spec = table_spec_by_manifest_type(entry.kind);
            if let Some(spec) = expected_spec {
                if !spec.filename.eq_ignore_ascii_case(&entry.filename) {
                    continue;
                }
            }
            let is_known = expected_spec.is_some();
            if is_known
                && deduped.iter().any(|seen: &ManifestEntry| {
                    seen.kind == entry.kind && seen.filename.eq_ignore_ascii_case(&entry.filename)
                })
            {
                continue;
            }
            deduped.push(entry);
        }

        Manifest { entries: deduped }
    }

    pub fn by_kind(&self, kind: u8) -> Option<&ManifestEntry> {
        self.entries.iter().find(|entry| entry.kind == kind)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        const MARK_PREFIX: [u8; 7] = [0x3c, 0xf3, 0x12, 0, 0, 0, 0];
        let mut bytes = Vec::new();
        for entry in &self.entries {
            bytes.extend_from_slice(entry.filename.as_bytes());
            bytes.push(0);
            let mut mark = MARK_PREFIX;
            mark[3] = entry.kind;
            bytes.extend_from_slice(&mark);
            bytes.extend_from_slice(&entry.count.to_le_bytes());
            bytes.push(0);
        }
        bytes
    }
}

/// The logical CM0102 data tables we know about from the manifest and carve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableId {
    Club,
    NatClub,
    Colour,
    Continent,
    Nation,
    Stadium,
    Officials,
    StaffType6,
    StaffType8,
    StaffType9,
    StaffType10,
    StaffComp,
    ClubComp,
    FirstNames,
    SecondNames,
    CommonNames,
    NationComp,
    StaffHistory,
    StaffCompHistory,
    ClubCompHistory,
    NationCompHistory,
    City,
}

/// How honest the current Rust model is for a table's record structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableEncoding {
    FixedRecord(FixedRecordLayout),
    Raw,
}

/// Static metadata for one logical table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableSpec {
    pub id: TableId,
    pub manifest_type: u8,
    pub filename: &'static str,
    pub logical_name: &'static str,
    pub encoding: TableEncoding,
}

impl TableSpec {
    pub fn fixed_record_layout(self) -> Option<FixedRecordLayout> {
        match self.encoding {
            TableEncoding::FixedRecord(layout) => Some(layout),
            TableEncoding::Raw => None,
        }
    }
}

pub const TABLE_SPECS: [TableSpec; 22] = [
    TableSpec {
        id: TableId::Club,
        manifest_type: 0,
        filename: "club.dat",
        logical_name: "clubs",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::verified(RecordKind::Club)),
    },
    TableSpec {
        id: TableId::NatClub,
        manifest_type: 1,
        filename: "nat_club.dat",
        logical_name: "national clubs",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::verified(RecordKind::Club)),
    },
    TableSpec {
        id: TableId::Colour,
        manifest_type: 2,
        filename: "colour.dat",
        logical_name: "colours",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::verified(RecordKind::Colour)),
    },
    TableSpec {
        id: TableId::Continent,
        manifest_type: 3,
        filename: "continent.dat",
        logical_name: "continents",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::verified(RecordKind::Continent)),
    },
    TableSpec {
        id: TableId::Nation,
        manifest_type: 4,
        filename: "nation.dat",
        logical_name: "nations",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::verified(RecordKind::Nation)),
    },
    TableSpec {
        id: TableId::Stadium,
        manifest_type: 5,
        filename: "stadium.dat",
        logical_name: "stadiums",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(78)),
    },
    TableSpec {
        id: TableId::StaffType6,
        manifest_type: 6,
        filename: "staff.dat",
        logical_name: "staff type 6",
        encoding: TableEncoding::Raw,
    },
    TableSpec {
        id: TableId::Officials,
        manifest_type: 7,
        filename: "officials.dat",
        logical_name: "officials",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(43)),
    },
    TableSpec {
        id: TableId::StaffType8,
        manifest_type: 8,
        filename: "staff.dat",
        logical_name: "staff type 8",
        encoding: TableEncoding::Raw,
    },
    TableSpec {
        id: TableId::StaffType9,
        manifest_type: 9,
        filename: "staff.dat",
        logical_name: "staff type 9",
        encoding: TableEncoding::Raw,
    },
    TableSpec {
        id: TableId::StaffType10,
        manifest_type: 10,
        filename: "staff.dat",
        logical_name: "staff type 10",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(70)),
    },
    TableSpec {
        id: TableId::StaffComp,
        manifest_type: 11,
        filename: "staff_comp.dat",
        logical_name: "staff competitions",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(101)),
    },
    TableSpec {
        id: TableId::ClubComp,
        manifest_type: 12,
        filename: "club_comp.dat",
        logical_name: "club competitions",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(107)),
    },
    TableSpec {
        id: TableId::FirstNames,
        manifest_type: 13,
        filename: "first_names.dat",
        logical_name: "first names",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(60)),
    },
    TableSpec {
        id: TableId::SecondNames,
        manifest_type: 14,
        filename: "second_names.dat",
        logical_name: "second names",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(60)),
    },
    TableSpec {
        id: TableId::CommonNames,
        manifest_type: 15,
        filename: "common_names.dat",
        logical_name: "common names",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(60)),
    },
    TableSpec {
        id: TableId::NationComp,
        manifest_type: 16,
        filename: "nation_comp.dat",
        logical_name: "nation competitions",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(107)),
    },
    TableSpec {
        id: TableId::StaffHistory,
        manifest_type: 17,
        filename: "staff_history.dat",
        logical_name: "staff history",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(17)),
    },
    TableSpec {
        id: TableId::StaffCompHistory,
        manifest_type: 18,
        filename: "staff_comp_history.dat",
        logical_name: "staff competition history",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(58)),
    },
    TableSpec {
        id: TableId::ClubCompHistory,
        manifest_type: 19,
        filename: "club_comp_history.dat",
        logical_name: "club competition history",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(26)),
    },
    TableSpec {
        id: TableId::NationCompHistory,
        manifest_type: 20,
        filename: "nation_comp_history.dat",
        logical_name: "nation competition history",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(26)),
    },
    TableSpec {
        id: TableId::City,
        manifest_type: 21,
        filename: "city.dat",
        logical_name: "cities",
        encoding: TableEncoding::FixedRecord(FixedRecordLayout::inferred(56)),
    },
];

pub fn table_spec_by_id(id: TableId) -> Option<&'static TableSpec> {
    TABLE_SPECS.iter().find(|spec| spec.id == id)
}

pub fn table_spec_by_manifest_type(kind: u8) -> Option<&'static TableSpec> {
    TABLE_SPECS.iter().find(|spec| spec.manifest_type == kind)
}

/// One loaded table from a CM0102 `Data` directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedTable {
    pub spec: TableSpec,
    pub manifest_count: u32,
    pub byte_len: usize,
}

impl LoadedTable {
    pub fn fixed_record_layout(&self) -> Option<FixedRecordLayout> {
        self.spec.fixed_record_layout()
    }

    pub fn fixed_record_count(&self) -> Option<usize> {
        let layout = self.fixed_record_layout()?;
        if self.byte_len % layout.size != 0 {
            return None;
        }
        Some(self.byte_len / layout.size)
    }
}

/// All known logical tables loaded from a CM0102 `Data` directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTables {
    pub tables: Vec<LoadedTable>,
}

impl DataTables {
    pub fn load_from_install(data_dir: &Path, manifest: &Manifest) -> io::Result<Self> {
        let mut tables = Vec::with_capacity(TABLE_SPECS.len());
        for spec in TABLE_SPECS {
            let entry = manifest.by_kind(spec.manifest_type).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "manifest missing type {} for {}",
                        spec.manifest_type, spec.filename
                    ),
                )
            })?;

            if !entry.filename.eq_ignore_ascii_case(spec.filename) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "manifest type {} expected {}, found {}",
                        spec.manifest_type, spec.filename, entry.filename
                    ),
                ));
            }

            let bytes = fs::read(data_dir.join(spec.filename))?;
            tables.push(LoadedTable {
                spec,
                manifest_count: entry.count,
                byte_len: bytes.len(),
            });
        }
        Ok(Self { tables })
    }

    pub fn table(&self, id: TableId) -> Option<&LoadedTable> {
        self.tables.iter().find(|table| table.spec.id == id)
    }
}

pub fn load_city_table(path: &Path) -> io::Result<CityTable<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    CityTable::new(leaked)
}

pub fn load_officials_table(path: &Path) -> io::Result<OfficialsTable<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    OfficialsTable::new(leaked)
}

pub fn load_name_table(path: &Path) -> io::Result<NameTable<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    NameTable::new(leaked)
}

pub fn load_stadium_table(path: &Path) -> io::Result<StadiumTable<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    StadiumTable::new(leaked)
}

pub fn load_staff_comp_table(path: &Path) -> io::Result<StaffCompTable<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    StaffCompTable::new(leaked)
}

pub fn load_club_comp_table(path: &Path) -> io::Result<ClubCompTable<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    ClubCompTable::new(leaked)
}

pub fn load_nation_comp_table(path: &Path) -> io::Result<NationCompTable<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    NationCompTable::new(leaked)
}

pub fn load_staff_history_table(path: &Path) -> io::Result<History17Table<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    History17Table::new(leaked)
}

pub fn load_staff_comp_history_table(path: &Path) -> io::Result<History58Table<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    History58Table::new(leaked)
}

pub fn load_club_comp_history_table(path: &Path) -> io::Result<History26Table<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    History26Table::new(leaked)
}

pub fn load_nation_comp_history_table(path: &Path) -> io::Result<History26Table<'static>> {
    let bytes = fs::read(path)?;
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    History26Table::new(leaked)
}

pub fn load_staff_data(path: &Path) -> io::Result<StaffData> {
    const TYPE6_COUNT: usize = 132_722;
    const TYPE8_COUNT: usize = 0;
    const TYPE9_COUNT: usize = 23_785;
    const TYPE10_COUNT: usize = 109_940;

    let bytes = fs::read(path)?;
    let type6_len = TYPE6_COUNT * StaffType6Table::LAYOUT.size;
    let type8_len = TYPE8_COUNT;
    let type9_len = TYPE9_COUNT * StaffType9Table::LAYOUT.size;
    let type10_len = TYPE10_COUNT * StaffType10Table::LAYOUT.size;
    let expected = type6_len + type8_len + type9_len + type10_len;
    if bytes.len() != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "staff.dat size {} did not match inferred split {}",
                bytes.len(),
                expected
            ),
        ));
    }

    let type6 = StaffType6Table::new(&bytes[0..type6_len])?.entries();
    let type8 = Vec::new();
    let type9_start = type6_len + type8_len;
    let type10_start = type9_start + type9_len;
    let type9 = StaffType9Table::new(&bytes[type9_start..type10_start])?.entries();
    let type10 = StaffType10Table::new(&bytes[type10_start..])?.entries();

    Ok(StaffData {
        type6,
        type8,
        type9,
        type10,
    })
}

pub fn write_staff_data(path: &Path, staff: &StaffData) -> io::Result<()> {
    let mut bytes = Vec::new();
    for entry in &staff.type6 {
        bytes.extend_from_slice(&entry.id.to_le_bytes());
        bytes.extend_from_slice(&entry.body);
    }
    for entry in &staff.type8 {
        bytes.extend_from_slice(&entry.body);
    }
    for entry in &staff.type9 {
        bytes.extend_from_slice(&entry.id.to_le_bytes());
        bytes.extend_from_slice(&entry.body);
    }
    for entry in &staff.type10 {
        bytes.extend_from_slice(&entry.id.to_le_bytes());
        bytes.push(entry.unknown_byte_4);
        bytes.extend_from_slice(&entry.rating_short_0x05.to_le_bytes());
        bytes.extend_from_slice(&entry.rating_short_0x07.to_le_bytes());
        bytes.extend_from_slice(&entry.unknown_bytes_9_12);
        bytes.extend_from_slice(&entry.rating_short_0x0d.to_le_bytes());
        bytes.extend_from_slice(&entry.unknown_bytes_15_26);
        bytes.extend_from_slice(&entry.attributes);
        bytes.extend_from_slice(&entry.unknown_bytes_58_64);
        bytes.extend_from_slice(&entry.trailing_bytes);
    }
    fs::write(path, bytes)
}

pub fn write_city_table(path: &Path, entries: &[CityEntry]) -> io::Result<()> {
    let mut bytes = Vec::with_capacity(entries.len() * CityTable::LAYOUT.size);
    for entry in entries {
        bytes.extend_from_slice(&entry.id.to_le_bytes());
        write_latin1_c_string(&mut bytes, &entry.name, 26);
        for value in entry.tail_u16 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    fs::write(path, bytes)
}

pub fn write_officials_table(path: &Path, entries: &[OfficialEntry]) -> io::Result<()> {
    let mut bytes = Vec::with_capacity(entries.len() * OfficialsTable::LAYOUT.size);
    for entry in entries {
        for value in entry.u16_slots {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.push(entry.trailing_byte);
    }
    fs::write(path, bytes)
}

pub fn write_name_table(path: &Path, entries: &[NameEntry]) -> io::Result<()> {
    let mut bytes = Vec::with_capacity(entries.len() * NameTable::LAYOUT.size);
    for entry in entries {
        write_latin1_c_string(&mut bytes, &entry.text, NameRecord::TEXT_END);
        bytes.extend_from_slice(&entry.footer);
    }
    fs::write(path, bytes)
}

pub fn write_stadium_table(path: &Path, entries: &[StadiumEntry]) -> io::Result<()> {
    let mut bytes = Vec::with_capacity(entries.len() * StadiumTable::LAYOUT.size);
    for entry in entries {
        bytes.extend_from_slice(&entry.id.to_le_bytes());
        write_latin1_c_string(&mut bytes, &entry.name, 52);
        bytes.extend_from_slice(&entry.unknown_tail);
    }
    fs::write(path, bytes)
}

pub fn write_staff_comp_table(path: &Path, entries: &[CompetitionEntry]) -> io::Result<()> {
    write_competition_table::<101, 56, 101>(path, entries)
}

pub fn write_club_comp_table(path: &Path, entries: &[CompetitionEntry]) -> io::Result<()> {
    write_competition_table::<107, 56, 104>(path, entries)
}

pub fn write_nation_comp_table(path: &Path, entries: &[CompetitionEntry]) -> io::Result<()> {
    write_competition_table::<107, 56, 104>(path, entries)
}

pub fn write_staff_history_table(path: &Path, entries: &[History17Entry]) -> io::Result<()> {
    let mut bytes = Vec::with_capacity(entries.len() * History17Table::LAYOUT.size);
    for entry in entries {
        for value in entry.u32_slots {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.push(entry.trailing_byte);
    }
    fs::write(path, bytes)
}

pub fn write_staff_comp_history_table(path: &Path, entries: &[History58Entry]) -> io::Result<()> {
    let mut bytes = Vec::with_capacity(entries.len() * History58Table::LAYOUT.size);
    for entry in entries {
        for value in entry.u32_slots {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&entry.trailing_u16.to_le_bytes());
    }
    fs::write(path, bytes)
}

pub fn write_club_comp_history_table(path: &Path, entries: &[History26Entry]) -> io::Result<()> {
    write_history26_table(path, entries)
}

pub fn write_nation_comp_history_table(path: &Path, entries: &[History26Entry]) -> io::Result<()> {
    write_history26_table(path, entries)
}

impl ReferenceData {
    pub fn load_from_data_dir(data_dir: &Path) -> io::Result<Self> {
        let cities = load_city_table(&data_dir.join("city.dat"))?.entries();
        let officials = load_officials_table(&data_dir.join("officials.dat"))?.entries();
        let first_names = load_name_table(&data_dir.join("first_names.dat"))?.entries();
        let second_names = load_name_table(&data_dir.join("second_names.dat"))?.entries();
        let common_names = load_name_table(&data_dir.join("common_names.dat"))?.entries();
        let stadiums = load_stadium_table(&data_dir.join("stadium.dat"))?.entries();
        let staff_competitions = load_staff_comp_table(&data_dir.join("staff_comp.dat"))?.entries();
        let club_competitions = load_club_comp_table(&data_dir.join("club_comp.dat"))?.entries();
        let nation_competitions =
            load_nation_comp_table(&data_dir.join("nation_comp.dat"))?.entries();
        let staff_history =
            load_staff_history_table(&data_dir.join("staff_history.dat"))?.entries();
        let staff_comp_history =
            load_staff_comp_history_table(&data_dir.join("staff_comp_history.dat"))?.entries();
        let club_comp_history =
            load_club_comp_history_table(&data_dir.join("club_comp_history.dat"))?.entries();
        let nation_comp_history =
            load_nation_comp_history_table(&data_dir.join("nation_comp_history.dat"))?.entries();

        Ok(Self {
            cities,
            officials,
            first_names,
            second_names,
            common_names,
            stadiums,
            staff_competitions,
            club_competitions,
            nation_competitions,
            staff_history,
            staff_comp_history,
            club_comp_history,
            nation_comp_history,
        })
    }
}

/// One named section in a `.sav` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSection {
    pub unknown_a: u32,
    pub size: u32,
    pub name: String,
}

impl SaveSection {
    pub fn record_kind(&self) -> Option<RecordKind> {
        RecordKind::from_section_name(&self.name)
    }

    pub fn verified_record_count(&self) -> Option<usize> {
        let kind = self.record_kind()?;
        if self.size as usize % kind.size() != 0 {
            return None;
        }
        Some(self.size as usize / kind.size())
    }
}

/// A parsed `.sav` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveFile {
    pub version: u32,
    pub sections: Vec<SaveSection>,
}

impl SaveFile {
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "save file shorter than header",
            ));
        }

        let version = le_u32(bytes, 0)?;
        let section_count = le_u32(bytes, 4)? as usize;
        let dir_len = 12 + section_count * SAVE_SECTION_ENTRY_SIZE;
        if bytes.len() < dir_len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "save directory truncated: need {dir_len} bytes, got {}",
                    bytes.len()
                ),
            ));
        }

        let mut sections = Vec::with_capacity(section_count);
        for idx in 0..section_count {
            let base = 12 + idx * SAVE_SECTION_ENTRY_SIZE;
            let unknown_a = le_u32(bytes, base)?;
            let size = le_u32(bytes, base + 4)?;
            let name = read_c_string(&bytes[base + 8..base + 8 + SAVE_SECTION_NAME_SIZE]);

            sections.push(SaveSection {
                unknown_a,
                size,
                name,
            });
        }

        Ok(Self { version, sections })
    }

    pub fn section(&self, name: &str) -> Option<&SaveSection> {
        self.sections
            .iter()
            .find(|section| section.name.eq_ignore_ascii_case(name))
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn find_manifest_filename(bytes: &[u8], mark_offset: usize) -> Option<String> {
    let search_start = mark_offset.saturating_sub(80);
    let window = &bytes[search_start..mark_offset];
    let dot = window
        .windows(4)
        .enumerate()
        .filter_map(|(index, slice)| (slice == b".dat").then_some(index))
        .last()?;
    let dot_abs = search_start + dot;
    let mut start = dot_abs;
    while start > search_start {
        let byte = bytes[start - 1];
        if byte == 0 || !byte.is_ascii_graphic() {
            break;
        }
        start -= 1;
    }
    let name = String::from_utf8_lossy(&bytes[start..dot_abs + 4]).into_owned();
    if name.ends_with(".dat") {
        Some(name)
    } else {
        None
    }
}

fn write_latin1_c_string(dst: &mut Vec<u8>, text: &str, width: usize) {
    let mut buf = vec![0u8; width];
    for (index, ch) in text.chars().take(width.saturating_sub(1)).enumerate() {
        buf[index] = if (ch as u32) <= 0xff { ch as u8 } else { b'?' };
    }
    dst.extend_from_slice(&buf);
}

fn write_competition_table<const N: usize, const NAME_B: usize, const SHORT_B: usize>(
    path: &Path,
    entries: &[CompetitionEntry],
) -> io::Result<()> {
    let mut bytes = Vec::with_capacity(entries.len() * N);
    for entry in entries {
        bytes.extend_from_slice(&entry.id.to_le_bytes());
        write_latin1_c_string(&mut bytes, &entry.long_name, NAME_B - 4);
        write_latin1_c_string(&mut bytes, &entry.short_name, SHORT_B - 56);
        bytes.extend_from_slice(&entry.unknown_tail);
    }
    fs::write(path, bytes)
}

fn write_history26_table(path: &Path, entries: &[History26Entry]) -> io::Result<()> {
    let mut bytes = Vec::with_capacity(entries.len() * History26Table::LAYOUT.size);
    for entry in entries {
        for value in entry.u32_slots {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&entry.trailing_u16.to_le_bytes());
    }
    fs::write(path, bytes)
}

fn le_u32(bytes: &[u8], off: usize) -> io::Result<u32> {
    let slice = bytes.get(off..off + 4).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("missing u32 at offset {off:#x}"),
        )
    })?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn read_c_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

fn read_c_string_latin1(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    bytes[..end].iter().map(|&b| char::from(b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_sizes_are_the_verified_values() {
        assert_eq!(RecordKind::Club.size(), 581);
        assert_eq!(RecordKind::Nation.size(), 290);
        assert_eq!(RecordKind::Continent.size(), 198);
        assert_eq!(RecordKind::Colour.size(), 58);
    }

    #[test]
    fn dat_container_counts_records() {
        let bytes = vec![0u8; 198 * 6];
        let f = DatFile::new(RecordKind::Continent, &bytes).unwrap();
        assert_eq!(f.count(), 6);
        assert_eq!(f.record(5).unwrap().len(), 198);
        assert!(f.record(6).is_none());
    }

    #[test]
    fn dat_container_rejects_misaligned() {
        assert!(DatFile::new(RecordKind::Colour, &[0u8; 100]).is_err());
    }

    #[test]
    fn table_specs_cover_expected_logical_tables() {
        assert_eq!(TABLE_SPECS.len(), 22);
        assert_eq!(
            table_spec_by_manifest_type(10).unwrap().id,
            TableId::StaffType10
        );
        assert_eq!(
            table_spec_by_id(TableId::Club).unwrap().filename,
            "club.dat"
        );
    }

    #[test]
    fn manifest_round_trips_through_writer() {
        let manifest = Manifest {
            entries: vec![
                ManifestEntry {
                    filename: "club.dat".into(),
                    kind: 0,
                    count: 10_580,
                },
                ManifestEntry {
                    filename: "staff.dat".into(),
                    kind: 10,
                    count: 109_940,
                },
                ManifestEntry {
                    filename: "city.dat".into(),
                    kind: 21,
                    count: 5_418,
                },
            ],
        };

        let bytes = manifest.to_bytes();
        let parsed = Manifest::parse(&bytes);

        assert_eq!(parsed.entries, manifest.entries);
    }

    #[test]
    fn loaded_table_reports_verified_count_when_possible() {
        let table = LoadedTable {
            spec: *table_spec_by_id(TableId::Continent).unwrap(),
            manifest_count: 6,
            byte_len: 198 * 6,
        };
        assert_eq!(table.fixed_record_count(), Some(6));
        assert_eq!(
            table.fixed_record_layout().unwrap().confidence,
            RecordLayoutConfidence::Verified
        );
    }

    #[test]
    fn save_file_parses_directory() {
        fn write_entry(dst: &mut Vec<u8>, a: u32, size: u32, name: &str) {
            dst.extend_from_slice(&a.to_le_bytes());
            dst.extend_from_slice(&size.to_le_bytes());
            let mut name_buf = [0u8; SAVE_SECTION_NAME_SIZE];
            let src = name.as_bytes();
            name_buf[..src.len()].copy_from_slice(src);
            dst.extend_from_slice(&name_buf);
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&999u32.to_le_bytes());
        write_entry(&mut bytes, 111, 4, "continent.dat");
        write_entry(&mut bytes, 333, 6, "club.dat");

        let save = SaveFile::parse(&bytes).unwrap();
        assert_eq!(save.version, 4);
        assert_eq!(save.sections.len(), 2);
        assert_eq!(save.sections[0].name, "continent.dat");
        assert_eq!(save.sections[0].size, 4);
        assert_eq!(save.section("club.dat").unwrap().unknown_a, 333);
    }

    #[test]
    fn save_section_reports_verified_record_counts() {
        let section = SaveSection {
            unknown_a: 0,
            size: 198 * 6,
            name: "continent.dat".into(),
        };
        assert_eq!(section.record_kind(), Some(RecordKind::Continent));
        assert_eq!(section.verified_record_count(), Some(6));

        let bad = SaveSection {
            unknown_a: 0,
            size: 123,
            name: "club.dat".into(),
        };
        assert_eq!(bad.verified_record_count(), None);
    }

    #[test]
    fn city_table_wraps_inferred_fixed_records() {
        let bytes = vec![0u8; CityTable::LAYOUT.size * 3];
        let table = CityTable::new(&bytes).unwrap();
        assert_eq!(table.count(), 3);
        assert_eq!(table.record(1).unwrap().bytes.len(), CityTable::LAYOUT.size);
    }

    #[test]
    fn officials_table_wraps_inferred_fixed_records() {
        let bytes = vec![0u8; OfficialsTable::LAYOUT.size * 2];
        let table = OfficialsTable::new(&bytes).unwrap();
        assert_eq!(table.count(), 2);
        assert_eq!(
            table.record(0).unwrap().bytes.len(),
            OfficialsTable::LAYOUT.size
        );
    }

    #[test]
    fn name_table_wraps_inferred_fixed_records() {
        let bytes = vec![0u8; NameTable::LAYOUT.size * 4];
        let table = NameTable::new(&bytes).unwrap();
        assert_eq!(table.count(), 4);
        assert_eq!(table.record(3).unwrap().bytes.len(), NameTable::LAYOUT.size);
    }

    #[test]
    fn city_record_exposes_id_and_name() {
        let mut bytes = vec![0u8; CityTable::LAYOUT.size];
        bytes[0..4].copy_from_slice(&7u32.to_le_bytes());
        bytes[4..9].copy_from_slice(b"Leeds");
        bytes[30..32].copy_from_slice(&5119u16.to_le_bytes());
        bytes[32..36].copy_from_slice(&123u32.to_le_bytes());
        let rec = CityRecord { bytes: &bytes };
        assert_eq!(rec.id(), 7);
        assert_eq!(rec.name(), "Leeds");
        assert_eq!(rec.unknown_tail().len(), 26);
        assert_eq!(rec.tail_u16(0), Some(5119));
        assert_eq!(rec.tail_u32(0), Some(123));
    }

    #[test]
    fn official_record_exposes_id() {
        let mut bytes = vec![0u8; OfficialsTable::LAYOUT.size];
        bytes[0..4].copy_from_slice(&12u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&76u32.to_le_bytes());
        bytes[42] = 9;
        let rec = OfficialRecord { bytes: &bytes };
        assert_eq!(rec.id(), 12);
        assert_eq!(rec.raw_fields().len(), OfficialsTable::LAYOUT.size - 4);
        assert_eq!(rec.u32_slot(1), Some(76));
        assert_eq!(rec.u16_slot(2), Some(76));
        assert_eq!(rec.trailing_byte(), 9);
    }

    #[test]
    fn name_record_exposes_text_and_footer() {
        let mut bytes = vec![0u8; NameTable::LAYOUT.size];
        bytes[0..5].copy_from_slice(b"Andre");
        bytes[48..52].copy_from_slice(&[0, 0, 0, 1]);
        let rec = NameRecord { bytes: &bytes };
        assert_eq!(rec.text(), "Andre");
        assert_eq!(rec.unknown_footer(), &[0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn staff_type10_record_exposes_compatibility_rating_slots() {
        let mut bytes = vec![0u8; StaffType10Table::LAYOUT.size];
        bytes[0..4].copy_from_slice(&123u32.to_le_bytes());
        bytes[4] = 9;
        bytes[5..7].copy_from_slice(&150u16.to_le_bytes());
        bytes[7..9].copy_from_slice(&175u16.to_le_bytes());
        bytes[13..15].copy_from_slice(&44u16.to_le_bytes());
        for i in 0..31 {
            bytes[0x1b + i] = i as u8;
        }
        bytes[65..70].copy_from_slice(&[1, 2, 3, 4, 5]);

        let rec = StaffType10Record { bytes: &bytes };
        assert_eq!(rec.id(), 123);
        assert_eq!(rec.unknown_byte_4(), 9);
        assert_eq!(rec.rating_short_0x05(), 150);
        assert_eq!(rec.rating_short_0x07(), 175);
        assert_eq!(rec.rating_short_0x0d(), 44);
        assert_eq!(rec.attributes()[0], 0);
        assert_eq!(rec.attributes()[30], 30);
        assert_eq!(rec.trailing_bytes(), [1, 2, 3, 4, 5]);
    }
}
