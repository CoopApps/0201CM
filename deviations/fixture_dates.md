# Deviation: League fixture dates

## Status: RECOVERY IN PROGRESS

- **Rust sites**:
  - `crates/cm-domain/src/lib.rs::start_new_game` league fixture emission uses
    `SeasonStart::resolve(base_year)` as round-0 date + flat +7-day stride in
    `generate_double_round_robin` (both TEMPORARY STUB).
  - `crates/cm-domain/src/league_calendar.rs::SEASON_STARTS` (season-window
    openers, not real matchdays).

- **Executable functions / addresses (VERIFIED)**:
  - `eng_prm.cpp`   at `0x0055cf20`  — Premier   ctor (comp id 7)
  - `eng_first.cpp` at `0x0055b340`  — First     ctor (comp id 8)
  - **`eng_second.cpp` at `0x0055f040`** — Second ctor (comp id 9)
  - `eng_third.cpp` at `0x00560b40`  — Third     ctor (comp id 10)
  - `eng_conf.cpp`  at `0x005577a0`  — Conference ctor (comp id 93)
  - `FUN_00560320` (`decompiled/00560320.c`) — sets `+0xba = vtable+0x3c()`
    (virtual dispatch to schedule-getter)
  - `FUN_0066f3b0` (`decompiled/0066f3b0.c`) — round-record writer; writes
    packed date at `buffer+round_idx*0x41+0` and `+0x02`, type byte at `+0x04`,
    int at `+0x3d`
  - `FUN_00533b50` (`decompiled/00533b50.c`) — date encoder; validates
    `1 ≤ day ≤ 31` and `0 ≤ month ≤ 11` (MONTHS ARE 0-INDEXED)
  - **`0x0055f340`** — eng_second schedule-getter (vtable slot +0x3c on
    vtable `0x00957dc8`). Not decompiled by Ghidra; disassembled directly.

- **Discrepancy**:
  1. Round 0 for English Second Division IS **Sun 12 Aug 2001**, not the
     Sat 11 Aug 2001 my pragmatic patch would have inserted (contract-forbidden
     historical guess). Port currently writes Tue 10 Jul 2001 (also wrong).
  2. Port uses flat +7-day stride between rounds. Exe uses variable stride
     with midweek rounds (e.g. Round 3 is Tue 28 Aug 2001, Round 6 is
     Thu 13 Sep 2001).
  3. Port never installs any schedule template; exe writes 46 round records
     into a `malloc(0xbae)` = 2990 byte buffer.

- **Evidence RECOVERED (VERIFIED EXACT)**:

  Rounds 0-25 for English Second Division 2001/02, decoded directly from
  the schedule-getter at `0x0055f340..0x0055f823` (calls 0-25 of 134):

  | Round | Day | Month (0-idx) | Year | Date        | Weekday |
  |-------|-----|---------------|------|-------------|---------|
  | 0     | 12  | 7             | 2001 | 12 Aug 2001 | Sun     |
  | 1     | 19  | 7             | 2001 | 19 Aug 2001 | Sun     |
  | 2     | 26  | 7             | 2001 | 26 Aug 2001 | Sun     |
  | 3     | 28  | 7             | 2001 | 28 Aug 2001 | Tue **midweek** |
  | 4     | 2   | 8             | 2001 | 2 Sep 2001  | Sun     |
  | 5     | 9   | 8             | 2001 | 9 Sep 2001  | Sun     |
  | 6     | 13  | 8             | 2001 | 13 Sep 2001 | Thu **midweek** |
  | 7     | 16  | 8             | 2001 | 16 Sep 2001 | Sun     |
  | 8     | 23  | 8             | 2001 | 23 Sep 2001 | Sun     |
  | 9     | 30  | 8             | 2001 | 30 Sep 2001 | Sun     |
  | 10    | 7   | 9             | 2001 | 7 Oct 2001  | Sun     |
  | 11    | 14  | 9             | 2001 | 14 Oct 2001 | Sun     |
  | 12    | 17  | 9             | 2001 | 17 Oct 2001 | Wed **midweek** |
  | 13    | 21  | 9             | 2001 | 21 Oct 2001 | Sun     |
  | 14    | 24  | 9             | 2001 | 24 Oct 2001 | Wed **midweek** |
  | 15    | 28  | 9             | 2001 | 28 Oct 2001 | Sun     |
  | 16    | 4   | 10            | 2001 | 4 Nov 2001  | Sun     |
  | 17    | 11  | 10            | 2001 | 11 Nov 2001 | Sun     |
  | 18    | 18  | 10            | 2001 | 18 Nov 2001 | Sun     |
  | 19    | 25  | 10            | 2001 | 25 Nov 2001 | Sun     |
  | 20    | 2   | 11            | 2001 | 2 Dec 2001  | Sun     |
  | 21    | 9   | 11            | 2001 | 9 Dec 2001  | Sun     |
  | 22    | 16  | 11            | 2001 | 16 Dec 2001 | Sun     |
  | 23    | 20  | 11            | 2001 | 20 Dec 2001 | Thu     |
  | 24    | 27  | 11            | 2001 | 27 Dec 2001 | Thu (Boxing week) |
  | 25    | 30  | 11            | 2001 | 30 Dec 2001 | Sun     |

  Full raw dump at `reports/fixture_disasm/eng_second_full_schedule.txt`.

- **Confidence**: STRONGLY SUPPORTED for rounds 0-25 (register-value tracking
  fully resolved). Rounds 26-45 are in the alternate branches of the
  schedule-getter (calls 26-66 of 134); the constant-tracker needs to walk
  further to resolve all `mov reg, imm` sources.

- **Work required**:
  1. Complete extraction of rounds 26-45 via improved constant tracker
     across the alternate `param_1` branches.
  2. Same disassembly pass for eng_prm (0x0055cf20), eng_first (0x0055b340),
     eng_third (0x00560b40), eng_conf (0x005577a0).
  3. Port `FUN_0066f3b0` (4 effective lines) + `FUN_00533b50` (date encoder,
     ~40 lines with validation).
  4. Encode recovered schedule as a Rust `pub const` per comp.
  5. Wire per-comp schedule into `generate_double_round_robin` so round N uses
     the schedule template's date instead of `season_start + N*7`.
  6. Port `FUN_0066f280` (schedule walker) byte-exact.
  7. Port `FUN_00668890` algorithmic path byte-exact.
  8. Validate: seed with same base year + PRNG state, confirm the port's
     first 46 English Second Division fixture dates match a Frida-captured
     `save.season.fixtures` from a fresh save.

- **Player-visible**: YES  |  **Save-affecting**: YES
- **Opened**: 2026-09-13
- **Resolved**: —
