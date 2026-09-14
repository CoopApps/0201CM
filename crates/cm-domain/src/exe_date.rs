//! Byte-exact ports of CM 01/02's date-encoding helpers.
//!
//! **AUTHORITATIVE BINARY**: `D:/cm0102/cm0102_GDI.exe` (the GDI build).
//! Not the DirectDraw `cm0102.exe`. See
//! `reports/fixture_disasm/gdi_va_map.json` and
//! `reports/fixture_disasm/GDI_CORRECTION_REPORT.md`.
//!
//! **Every port in this module has been verified against the GDI spec.**
//! Static disassembly of GDI 0x0066ef70 (round writer), 0x0066efd0 (slot
//! writer), 0x005340e0 (flag-snap), 0x00533d80 (pack_date), and
//! 0x0066ee40 (walker) shows each function is BYTE-IDENTICAL to its
//! DirectDraw equivalent modulo two categories of address relocations:
//!   1. Call targets shifting by -0x440 (fixture-cluster) or +0x230
//!      (date-cluster) between builds.
//!   2. DAT globals shifting by -0xb0/-0xb8 between builds.
//! Neither category changes semantics.
//!
//! Runtime corroboration: schedule-getter direct-called on both
//! running processes produces identical 2990-byte buffers
//! (SHA256 `682a5ea6…`).
//!
//! GDI addresses referenced in this module:
//!   - `0x00533d80` pack_date  (DirectDraw was 0x00533b50)
//!   - `0x005340e0` flag-snap  (DirectDraw was 0x00533eb0)
//!   - `0x0066ef70` round writer (DirectDraw was 0x0066f3b0)
//!   - `0x0066efd0` slot writer (DirectDraw was 0x0066f410)
//!
//! Data tables `DAT_009a4b28` (non-leap) and `DAT_009a4b40` (leap)
//! present in both builds at the same VAs.

/// Cumulative days *before* each month, for a non-leap year.
///
/// `DAT_009a4b28` @ 0x009a4b28 in `cm0102.exe`. Note January is -1 so
/// that `MONTH_START_NON_LEAP[month] + day` yields the **0-indexed**
/// day-of-year (Jan 1 = 0).
pub const MONTH_START_NON_LEAP: [i16; 12] = [
    -1, 30, 58, 89, 119, 150, 180, 211, 242, 272, 303, 333,
];

/// Cumulative days *before* each month, for a leap year.
///
/// `DAT_009a4b40` @ 0x009a4b40. Same convention (Jan 1 → 0).
pub const MONTH_START_LEAP: [i16; 12] = [
    -1, 30, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334,
];

/// Byte-exact port of `FUN_00533eb0` — weekday snap-back / snap-forward.
///
/// Given a date buffer `[doy, year, is_leap, _]` (as produced by
/// `pack_date`) and a target weekday `flag` (0=Mon..6=Sun), rotates
/// `doy` to the nearest occurrence of `flag` (chosen from the signed
/// distance in [-3..3]). If `flag == -1`, no-op (caller path guarded).
///
/// Mechanism (validated against runtime capture 2026-09-13 across all
/// 42 flag-snap calls in the eng_second schedule construction):
///
///   1. Compute `total_days = (year - 1600) * 365 + leap_correction + doy`
///      where `leap_correction = (year-1600)/400 + ((year-1600)+((year-1600)>>31 & 3))/4 - (year-1600)/100`
///      then subtract 1 if `is_leap`.
///   2. `wd = total_days % 7`.  Normalise: `wd == 0 → 6` (Sunday=6);
///      else `wd -= 1` (Monday=0..Saturday=5).
///   3. `delta = wd - flag`.
///      - If `delta < -3`: `delta += 7` (choose forward path).
///      - If `delta > 3`: `delta -= 7` (choose backward path).
///   4. `doy -= delta`. Year rollover is handled by the caller — for
///      all 46 English Second Division rounds the shift is |1|, no
///      overflow.
///
/// CM0102 0x00533eb0.
pub fn apply_flag_snap(buf: &mut [i16; 4], flag: i32) {
    if flag == -1 {
        return;
    }
    let doy = buf[0];
    let year = buf[1] as i32;
    let is_leap = buf[2];
    let y = year - 0x640; // year - 1600
    let leap_correction = y / 400 + ((y + ((y >> 31) & 3)) >> 2) - y / 100;
    let mut total = y * 365 + leap_correction + doy as i32;
    if is_leap != 0 {
        total -= 1;
    }
    let rem = total.rem_euclid(7);
    let wd = if rem == 0 { 6 } else { rem - 1 };
    let mut delta = wd - flag;
    if delta < -3 {
        delta += 7;
    } else if delta > 3 {
        delta -= 7;
    }
    buf[0] = doy - delta as i16;
}

/// Byte-exact port of `FUN_00533b50`.
///
/// Encodes `(day, month, year)` into a 4-short record.
///
/// Layout of the returned array:
///   - `[0]` = 0-indexed day-of-year (Jan 1 = 0)
///   - `[1]` = year
///   - `[2]` = 1 if `year` is a leap year, 0 otherwise
///   - `[3]` = 0 (unused / reserved)
///
/// Validation matches the exe: `day` is clamped to `1..=31` (invalid
/// → 1); `month` is clamped to `0..=11` (invalid → 0). The exe emits
/// an error message via `FUN_00933d8f` in those paths; we silently
/// clamp — the output byte pattern is identical either way.
///
/// **This is only the PRE-SNAP step.** The exe writes date records
/// by chaining `pack_date` then `FUN_00533eb0(flag, result)` where
/// `flag` is a target weekday (0=Mon..6=Sun) or -1 for no-snap. The
/// snap helper rewinds day-of-year to the previous occurrence of the
/// target weekday. `FUN_00533eb0` is not yet ported — the runtime
/// capture at `reports/fixture_disasm/RUNTIME_CAPTURE_REPORT.md`
/// shows all 46 English Second Division rounds and their pre/post-snap
/// dates. Do NOT use `pack_date` output as final buffer bytes without
/// applying the flag-snap chain.
///
/// CM0102 0x00533b50 (`__thiscall`, this = `result` ptr).
pub fn pack_date(day: i16, month: i8, year: u16) -> [i16; 4] {
    let day = if !(1..=31).contains(&day) { 1 } else { day };
    let month = if !(0..=11).contains(&month) { 0 } else { month };

    // Standard Gregorian leap-year rule as encoded in the exe:
    //   NOT leap iff  (year & 1 == 1)             i.e. odd year
    //             OR  (year & 3 != 0)             not /4
    //             OR  (year % 100 == 0 && year % 400 != 0)   century not /400
    let not_leap = (year & 1 == 1)
        || (year & 3 != 0)
        || (year % 100 == 0 && year % 400 != 0);

    let (base, leap_flag) = if not_leap {
        (MONTH_START_NON_LEAP, 0)
    } else {
        (MONTH_START_LEAP, 1)
    };
    let day_of_year = base[month as usize] + day;

    [day_of_year, year as i16, leap_flag, 0]
}

/// Byte-exact port of `FUN_0066f3b0` — writes one round record into a
/// schedule template buffer.
///
/// Original C decompile:
/// ```c
/// FUN_00533b50(day, month, day_off + year, flag);   // -> local_8, local_6
/// puVar1 = buffer + round_idx * 0x41;
/// *puVar1                     = local_8;             // +0x00 day-of-year
/// puVar1[1]                   = local_6 - year;      // +0x02 year offset
/// *(u8*)(puVar1 + 2)          = type_byte;           // +0x04
/// *(u32*)((int)puVar1 + 0x3d) = prize_or_int;        // +0x3d
/// ```
///
/// The buffer is a caller-supplied `Vec<u8>` sized `round_count * 65`.
/// Each round record is 65 (0x41) bytes; this writer touches only
/// bytes `+0x00`, `+0x02`, `+0x04`, and `+0x3d..+0x41`. Other bytes
/// stay zero.
///
/// `day_off` (param_5 in the exe) is normally 0 for round dates in
/// the base year, 1 for dates in `year + 1` (Jan..May of the second
/// half of a Northern-hemisphere season).
///
/// CM0102 0x0066f3b0.
pub fn write_round_record(
    buffer: &mut [u8],
    round_idx: u16,
    day: i8,
    month: i8,
    day_off: i32,
    flag: i32,
    type_byte: u8,
    year: u16,
    prize_or_int: i32,
) {
    // `day_off + year` — the exe passes an i32 sum; if day_off == 1
    // that yields year + 1.
    let real_year = ((year as i32) + day_off) as u16;
    let mut packed = pack_date(day as i16, month, real_year);
    // Apply the flag-snap chain (FUN_00533eb0 in the exe).
    apply_flag_snap(&mut packed, flag);
    // packed[0] = day-of-year (post-snap), packed[1] = year.
    // Store day-of-year at +0x00 and `year - season_base_year` (year
    // offset) at +0x02.
    let year_offset = packed[1] - year as i16;

    let off = (round_idx as usize) * 0x41;
    let record = &mut buffer[off..off + 0x41];
    record[0..2].copy_from_slice(&packed[0].to_le_bytes());
    record[2..4].copy_from_slice(&year_offset.to_le_bytes());
    record[4] = type_byte;
    record[0x3d..0x41].copy_from_slice(&prize_or_int.to_le_bytes());
}

/// Byte-exact port of `FUN_0066f410` — writes one 7-byte "fixture
/// sub-slot" inside a 65-byte round record.
///
/// Each 65-byte round record contains **8 sub-slots** of 7 bytes at
/// offsets `+0x05..+0x3c` (indexed by `sub_slot ∈ 0..=7`). Each slot's
/// layout inside the record at `round_idx * 0x41 + sub_slot * 7`:
///   +0x05 : u32 aux payload (`param_7`)   — semantics not yet decoded
///   +0x09 : i8  slot field a (`param_4` domain [-1..6])
///   +0x0a : i8  slot field b (`param_5` domain [-1..2])
///   +0x0b : i8  slot field c (`param_6` domain [-1..4])
///
/// Domain constraints are ENFORCED by the exe (validation clauses at
/// 0066f410:11..38 emit error messages then fall through to the store).
/// We silently clamp — byte pattern is identical either way.
///
/// **VERIFIED STRUCTURE, SEMANTICS PARTIAL**: The three i8 fields have
/// small enum ranges ([-1..6], [-1..2], [-1..4]) so they are NOT team
/// IDs (English Div 2 has 24 teams). Likely per-round metadata (cup-round
/// overlap flags, postponement state, TV pick). FUN_00668450 (the
/// round-robin driver — older notes called it FUN_00668890 after
/// an inner label at that address; the outer function extent is
/// 0x00668450..0x00668d70) READS +0x0b and checks for values 3 and 4 —
/// suggesting +0x0b is a status flag with those values having meaning.
///
/// The schedule-getter calls this once per round with
/// `(buf, round, 0, -1, -1, -1, 0)` — initialising slot 0 to sentinels
/// and leaving slots 1..7 at malloc-returned bytes.
///
/// CM0102 0x0066f410.
pub fn write_slot(
    buffer: &mut [u8],
    round_idx: u16,
    sub_slot: u8,
    field_a: i8,
    field_b: i8,
    field_c: i8,
    aux_payload: u32,
) {
    let sub_slot = if sub_slot > 7 { 0 } else { sub_slot };
    let base = (round_idx as usize) * 0x41 + (sub_slot as usize) * 7;
    // Aux payload at slot_base + 5
    buffer[base + 5..base + 9].copy_from_slice(&aux_payload.to_le_bytes());
    buffer[base + 9] = field_a as u8;
    buffer[base + 10] = field_b as u8;
    buffer[base + 11] = field_c as u8;
}

/// English Second Division 2001/02 schedule template — the exact
/// `(day, month, day_off, flag, type)` sequence FUN_0055f340 feeds
/// into FUN_0066f3b0 for arg1 = 0xFF (normal construction path).
///
/// Runtime-verified 2026-09-13; every entry corresponds to a distinct
/// call site in `0x0055f3db..0x0055ffca`.
pub const ENG_SECOND_2001_TEMPLATE: [(i8, i8, i32, i32, u8); 46] = [
    (12,  7, 0,  5, 1), (19,  7, 0,  5, 1), (26,  7, 0,  5, 1),
    (28,  7, 0,  0, 2), ( 2,  8, 0,  5, 1), ( 9,  8, 0,  5, 1),
    (13,  8, 0,  2, 2), (16,  8, 0,  5, 1), (23,  8, 0,  5, 1),
    (30,  8, 0,  5, 1), ( 7,  9, 0,  5, 1), (14,  9, 0,  5, 1),
    (17,  9, 0,  1, 2), (21,  9, 0,  5, 1), (24,  9, 0,  1, 2),
    (28,  9, 0,  5, 1), ( 4, 10, 0,  5, 1), (11, 10, 0,  5, 1),
    (18, 10, 0,  5, 1), (25, 10, 0,  5, 1), ( 2, 11, 0,  5, 1),
    ( 9, 11, 0,  5, 1), (16, 11, 0,  5, 1), (22, 11, 0, -1, 1),
    (26, 11, 0, -1, 1), (29, 11, 0, -1, 1), ( 1,  0, 1, -1, 1),
    (13,  0, 1,  5, 1), (20,  0, 1,  5, 1), ( 3,  1, 1,  5, 1),
    (10,  1, 1,  5, 1), (17,  1, 1,  5, 1), (20,  1, 1,  1, 2),
    (24,  1, 1,  5, 1), ( 3,  2, 1,  5, 1), ( 7,  2, 1,  2, 2),
    (10,  2, 1,  5, 1), (17,  2, 1,  1, 2), (24,  2, 1,  5, 1),
    (31,  2, 1,  5, 1), ( 7,  3, 1,  5, 1), (14,  3, 1,  5, 1),
    (16,  3, 1,  0, 2), (21,  3, 1,  5, 1), (28,  3, 1,  5, 1),
    ( 6,  4, 1,  6, 1),
];

/// Build the 2990-byte English Second Division schedule buffer using
/// the exact call sequence FUN_0055f340 uses:
///   for round in 0..46:
///     FUN_0066f3b0(buf, round, day, month, day_off, flag, type, year, 0)
///     FUN_0066f410(buf, round, 0, -1, -1, -1, 0)
///
/// Deterministic; only depends on `season_base_year`.
///
/// Verified byte-exact vs runtime capture for `season_base_year =
/// 2001` — see `full_buffer_matches_runtime_capture` test.
pub fn build_eng_second_schedule(season_base_year: u16) -> Vec<u8> {
    let mut buf = vec![0u8; 46 * 0x41];
    for (idx, &(day, month, day_off, flag, type_byte)) in
        ENG_SECOND_2001_TEMPLATE.iter().enumerate()
    {
        write_round_record(
            &mut buf, idx as u16, day, month, day_off, flag, type_byte,
            season_base_year, 0,
        );
        // Every writer call is followed by a slot-writer that
        // initialises fixture-slot 0 to sentinels.
        write_slot(&mut buf, idx as u16, 0, -1, -1, -1, 0);
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ground truth from FUN_00533b50 dump. Round 0 of eng_second
    /// 2001/02: (day=12, month=7, year=2001).
    ///   MONTH_START_NON_LEAP[7] + 12 = 211 + 12 = 223 (0-indexed day-of-year)
    ///   2001 is not a leap year → leap_flag = 0
    #[test]
    fn round_0_english_second_2001_02_prepack() {
        let p = pack_date(12, 7, 2001);
        assert_eq!(p, [223, 2001, 0, 0]);
    }

    #[test]
    fn round_3_midweek_28_aug_2001_prepack() {
        let p = pack_date(28, 7, 2001);
        assert_eq!(p, [239, 2001, 0, 0]);
    }

    #[test]
    fn leap_year_2000() {
        let p = pack_date(1, 2, 2000); // 1 March 2000, leap
        assert_eq!(p, [60, 2000, 1, 0]);
    }

    #[test]
    fn non_leap_2001() {
        let p = pack_date(1, 2, 2001); // 1 March 2001, non-leap
        assert_eq!(p, [59, 2001, 0, 0]);
    }

    // --- flag-snap tests (runtime capture 2026-09-13) ---

    fn snap(day: i16, mon: i8, year: u16, flag: i32) -> i16 {
        let mut b = pack_date(day, mon, year);
        apply_flag_snap(&mut b, flag);
        b[0]
    }

    #[test]
    fn snap_r0_sun12aug_to_sat11aug() {
        // input (12, Aug, 2001, flag=5) doy 223 (Sun) → 222 (Sat)
        assert_eq!(snap(12, 7, 2001, 5), 222);
    }

    #[test]
    fn snap_r3_tue28aug_to_mon27aug() {
        // Tue → Mon, delta -1
        assert_eq!(snap(28, 7, 2001, 0), 238);
    }

    #[test]
    fn snap_r6_thu13sep_to_wed12sep() {
        assert_eq!(snap(13, 8, 2001, 2), 254);
    }

    #[test]
    fn snap_r37_sun17mar_to_tue19mar_forward() {
        // The forward-snap case: doy 75 (Sun) → 77 (Tue), delta=-2.
        assert_eq!(snap(17, 2, 2002, 1), 77);
    }

    #[test]
    fn snap_r45_mon6may_to_sun5may() {
        // Mon → prev Sun, delta -1. doy 125 → 124.
        assert_eq!(snap(6, 4, 2002, 6), 124);
    }

    /// The full 2990-byte English Second Division schedule buffer
    /// generated from `ENG_SECOND_2001_TEMPLATE` must match the live
    /// runtime capture byte-for-byte across ALL 2990 bytes.
    ///
    /// This now includes the +0x05..+0x0b slot-0 sentinels written by
    /// FUN_0066f410. Bytes +0x0c..+0x3c come from freshly-malloc'd
    /// memory (observed zero in this capture — heap arena chance,
    /// not an initialised state per the exe spec).
    /// Authoritative capture: cm0102_GDI.exe schedule buffer for
    /// English Second Division 2001/02.
    #[test]
    fn full_buffer_matches_runtime_capture_gdi() {
        const CAPTURE: &[u8] = include_bytes!(
            "../../../reports/fixture_disasm/runtime/20260913_131323_gdi_buffer_0.bin"
        );
        assert_eq!(CAPTURE.len(), 2990);
        let ours = super::build_eng_second_schedule(2001);
        assert_eq!(ours.len(), 2990);
        for i in 0..2990 {
            if ours[i] != CAPTURE[i] {
                let round = i / 0x41;
                let off_in_rec = i % 0x41;
                panic!("byte {} (round {} +0x{:02x}): ours={:#04x} gdi_capture={:#04x}",
                       i, round, off_in_rec, ours[i], CAPTURE[i]);
            }
        }
    }

    /// Corroborating capture: DirectDraw build produces byte-identical
    /// buffer. Kept to document that the schedule-getter code is the
    /// same between builds; the GDI capture above is the authoritative
    /// specification.
    #[test]
    fn full_buffer_matches_runtime_capture_dd_corroborates() {
        const CAPTURE: &[u8] = include_bytes!(
            "../../../reports/fixture_disasm/runtime/20260913_113106_direct_buffer_0.bin"
        );
        assert_eq!(CAPTURE.len(), 2990);
        let ours = super::build_eng_second_schedule(2001);
        for i in 0..2990 {
            assert_eq!(ours[i], CAPTURE[i], "byte {} differs (round {} +0x{:02x})",
                       i, i/0x41, i%0x41);
        }
    }

    #[test]
    fn snap_no_op_when_flag_neg1() {
        // pack_date output must survive unchanged with flag = -1
        let mut b = pack_date(22, 11, 2001); // Sat 22 Dec doy 355
        apply_flag_snap(&mut b, -1);
        assert_eq!(b[0], 355);
    }

    /// All 42 flag-snap invocations from the runtime capture must
    /// reproduce byte-exactly. Data grabbed from
    /// runtime/20260913_113814_v2_events.jsonl.
    #[test]
    fn snap_all_42_runtime_calls() {
        // (day, month(0-idx), year, flag, expected_doy_after)
        let cases = [
            (12, 7, 2001, 5, 222), (19, 7, 2001, 5, 229), (26, 7, 2001, 5, 236),
            (28, 7, 2001, 0, 238), (2,  8, 2001, 5, 243), (9,  8, 2001, 5, 250),
            (13, 8, 2001, 2, 254), (16, 8, 2001, 5, 257), (23, 8, 2001, 5, 264),
            (30, 8, 2001, 5, 271), (7,  9, 2001, 5, 278), (14, 9, 2001, 5, 285),
            (17, 9, 2001, 1, 288), (21, 9, 2001, 5, 292), (24, 9, 2001, 1, 295),
            (28, 9, 2001, 5, 299), (4, 10, 2001, 5, 306), (11,10, 2001, 5, 313),
            (18,10, 2001, 5, 320), (25,10, 2001, 5, 327), (2, 11, 2001, 5, 334),
            (9, 11, 2001, 5, 341), (16,11, 2001, 5, 348),
            // -- second half (year=2002) --
            (13, 0, 2002, 5, 11),  (20, 0, 2002, 5, 18),  (3,  1, 2002, 5, 32),
            (10, 1, 2002, 5, 39),  (17, 1, 2002, 5, 46),  (20, 1, 2002, 1, 49),
            (24, 1, 2002, 5, 53),  (3,  2, 2002, 5, 60),  (7,  2, 2002, 2, 64),
            (10, 2, 2002, 5, 67),  (17, 2, 2002, 1, 77),  (24, 2, 2002, 5, 81),
            (31, 2, 2002, 5, 88),  (7,  3, 2002, 5, 95),  (14, 3, 2002, 5, 102),
            (16, 3, 2002, 0, 104), (21, 3, 2002, 5, 109), (28, 3, 2002, 5, 116),
            (6,  4, 2002, 6, 124),
        ];
        for (i, (d, m, y, f, exp)) in cases.iter().enumerate() {
            let got = snap(*d, *m, *y, *f);
            assert_eq!(got, *exp,
                "case {}: pack_date({},{},{}) then flag={} — got doy={}, expected {}",
                i, d, m, y, f, got, exp);
        }
    }
}
