//! Byte-exact ports of CM 01/02's date-encoding helpers.
//!
//! Sources (all from `D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`):
//!   - `00533b50.c` (`FUN_00533b50`) — packs (day, month, year) into a
//!     4-short record: [day_of_year_0idx, year, is_leap, reserved].
//!   - `0066f3b0.c` (`FUN_0066f3b0`) — writes one 65-byte round record
//!     into the schedule buffer at `buffer + round_idx * 0x41`.
//!
//! Data tables `DAT_009a4b28` (non-leap) and `DAT_009a4b40` (leap) were
//! dumped from `cm0102.exe` .rdata at VA 0x009a4b28 / 0x009a4b40.

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
/// The exe function optionally chains to `FUN_00533eb0(flag, result)`
/// when `flag != -1`. That helper writes additional data into the
/// same buffer; not yet ported. All schedule-getter callers we've
/// seen pass `flag = 5` or `flag = -1`, and the extra path only fires
/// on non-`-1` values. Callers that need the chain must invoke the
/// (future) `apply_flag_chain` function separately.
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
    _flag: i32,
    type_byte: u8,
    year: u16,
    prize_or_int: i32,
) {
    // `day_off + year` — the exe passes an i32 sum; if day_off == 1
    // that yields year + 1.
    let real_year = ((year as i32) + day_off) as u16;
    let packed = pack_date(day as i16, month, real_year);
    // packed[0] = day-of-year, packed[1] = year — write day-of-year
    // and `year - original_year` (year offset relative to season
    // base) into the record.
    let year_offset = packed[1] - year as i16;

    let off = (round_idx as usize) * 0x41;
    let record = &mut buffer[off..off + 0x41];
    record[0..2].copy_from_slice(&packed[0].to_le_bytes());
    record[2..4].copy_from_slice(&year_offset.to_le_bytes());
    record[4] = type_byte;
    record[0x3d..0x41].copy_from_slice(&prize_or_int.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ground truth from FUN_00533b50 dump. Round 0 of eng_second
    /// 2001/02: (day=12, month=7, year=2001).
    ///   MONTH_START_NON_LEAP[7] + 12 = 211 + 12 = 223 (0-indexed day-of-year)
    ///   2001 is not a leap year → leap_flag = 0
    #[test]
    fn round_0_english_second_2001_02() {
        let p = pack_date(12, 7, 2001);
        assert_eq!(p, [223, 2001, 0, 0]);
    }

    #[test]
    fn round_3_midweek_28_aug_2001() {
        let p = pack_date(28, 7, 2001);
        assert_eq!(p, [239, 2001, 0, 0]);
    }

    #[test]
    fn leap_year_2000() {
        let p = pack_date(1, 2, 2000); // 1 March 2000, leap
        // MONTH_START_LEAP[2] + 1 = 59 + 1 = 60
        assert_eq!(p, [60, 2000, 1, 0]);
    }

    #[test]
    fn non_leap_2001() {
        let p = pack_date(1, 2, 2001); // 1 March 2001, non-leap
        // MONTH_START_NON_LEAP[2] + 1 = 58 + 1 = 59
        assert_eq!(p, [59, 2001, 0, 0]);
    }
}
