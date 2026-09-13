# Deviation: League fixture dates

## Status: PARTIALLY RECOVERED — flag-snap chain + walker + driver still to port

- **Rust sites**:
  - `crates/cm-domain/src/lib.rs::start_new_game` league fixture emission uses
    `SeasonStart::resolve(base_year)` as round-0 date + flat +7-day stride in
    `generate_double_round_robin` (both TEMPORARY STUB).
  - `crates/cm-domain/src/league_calendar.rs::SEASON_STARTS` (season-window
    openers, not real matchdays).
  - `crates/cm-domain/src/exe_date.rs` — `pack_date` (byte-exact PRE-SNAP);
    `write_round_record` (byte-exact for +0x00/+0x02/+0x04/+0x3d slots but
    does NOT apply the flag-snap chain).

- **Executable functions / addresses (VERIFIED)**:
  - `eng_prm.cpp`   at `0x0055cf20`  — Premier   ctor (comp id 7)
  - `eng_first.cpp` at `0x0055b340`  — First     ctor (comp id 8)
  - **`eng_second.cpp` at `0x0055f040`** — Second ctor (comp id 9)
  - `eng_third.cpp` at `0x00560b40`  — Third     ctor (comp id 10)
  - `eng_conf.cpp`  at `0x005577a0`  — Conference ctor (comp id 93)
  - `FUN_00560320` — sets `+0xba = vtable+0x3c()` (virtual dispatch to
    schedule-getter)
  - **`0x0055f340`** — eng_second schedule-getter (vtable slot +0x3c on
    vtable `0x00957dc8`).
  - `FUN_0066f3b0` — round-record writer; 9 args
    `(buffer, round_idx, day, month, day_off, flag, type, year, prize)`;
    writes packed date, type, prize into the 65-byte record.
  - `FUN_00533b50` — date encoder (day/month/year → 4-short pack).
  - **`FUN_00533eb0`** — flag-snap helper (weekday snap-back); NOT YET
    PORTED. This is the key remaining pre-buffer step.
  - `FUN_00668890` — round-robin driver (writes the +0x09..+0x0b 0xFF
    sentinels + is presumed to fill team-pair bytes later).
  - `FUN_0066f280` — schedule walker / nominal-date generator.

## VERIFIED EXACT — Runtime capture 2026-09-13

Full report: `reports/fixture_disasm/RUNTIME_CAPTURE_REPORT.md`.
Raw artefacts: `reports/fixture_disasm/runtime/20260913_113106_*`.

Method: Frida in-process attach + direct call to `FUN_0055f340` with
`arg1 = 0xFF` on a synthetic comp record (`this+0x40 = 2001`).

### Answers to standing questions

1. **"134 static call sites — why not 46?"** Because 134 calls to
   FUN_0066f3b0 exist across ALL param_1 branches of FUN_0055f340.
   Runtime with arg1=0xFF hits **exactly 46**. The remaining 88 belong
   to playoff/promotion/reset paths not taken for normal Div-2
   construction.

2. **"Buffer size / layout?"** `malloc(0xbae)` = 2990 bytes = 46 × 65.
   Confirmed the exact bytes and per-record layout (see report §3).

3. **"Round 0 date?"** **Sat 11 Aug 2001** — buffer doy=222, year=2001.
   NOT Sun 12 Aug as my earlier static-analysis claimed. (My static
   pass captured the writer *input* day=12; the buffer stores the
   flag-snapped result day=11.)

4. **"What is `flag`?"** Target weekday for `FUN_00533eb0` snap-back
   (0=Mon..6=Sun, -1=no-snap). Verified across all 46 rounds: nominal
   is always exactly one day past target, and `flag = -1` on rounds
   23–26 (Xmas/New Year holidays) preserves the exact hardcoded date.

### Full recovered fixture list (VERIFIED EXACT from buffer)

| Rd | Date | Type | Notes |
|----|------|------|-------|
| 0 | Sat 11 Aug 2001 | 1 | Opening day |
| 1 | Sat 18 Aug 2001 | 1 | |
| 2 | Sat 25 Aug 2001 | 1 | |
| 3 | Mon 27 Aug 2001 | 2 | Aug Bank Holiday midweek |
| 4 | Sat 01 Sep 2001 | 1 | |
| 5 | Sat 08 Sep 2001 | 1 | |
| 6 | Wed 12 Sep 2001 | 2 | Midweek |
| 7 | Sat 15 Sep 2001 | 1 | |
| 8 | Sat 22 Sep 2001 | 1 | |
| 9 | Sat 29 Sep 2001 | 1 | |
| 10 | Sat 06 Oct 2001 | 1 | |
| 11 | Sat 13 Oct 2001 | 1 | |
| 12 | Tue 16 Oct 2001 | 2 | Midweek |
| 13 | Sat 20 Oct 2001 | 1 | |
| 14 | Tue 23 Oct 2001 | 2 | Midweek |
| 15 | Sat 27 Oct 2001 | 1 | |
| 16 | Sat 03 Nov 2001 | 1 | |
| 17 | Sat 10 Nov 2001 | 1 | |
| 18 | Sat 17 Nov 2001 | 1 | |
| 19 | Sat 24 Nov 2001 | 1 | |
| 20 | Sat 01 Dec 2001 | 1 | |
| 21 | Sat 08 Dec 2001 | 1 | |
| 22 | Sat 15 Dec 2001 | 1 | |
| 23 | Sat 22 Dec 2001 | 1 | flag=-1 |
| 24 | Wed 26 Dec 2001 | 1 | Boxing Day flag=-1 |
| 25 | Sat 29 Dec 2001 | 1 | flag=-1 |
| 26 | Tue 01 Jan 2002 | 1 | New Year flag=-1 |
| 27 | Sat 12 Jan 2002 | 1 | |
| 28 | Sat 19 Jan 2002 | 1 | |
| 29 | Sat 02 Feb 2002 | 1 | |
| 30 | Sat 09 Feb 2002 | 1 | |
| 31 | Sat 16 Feb 2002 | 1 | |
| 32 | Tue 19 Feb 2002 | 2 | Midweek |
| 33 | Sat 23 Feb 2002 | 1 | |
| 34 | Sat 02 Mar 2002 | 1 | |
| 35 | Wed 06 Mar 2002 | 2 | Midweek |
| 36 | Sat 09 Mar 2002 | 1 | |
| 37 | Tue 19 Mar 2002 | 2 | Midweek |
| 38 | Sat 23 Mar 2002 | 1 | |
| 39 | Sat 30 Mar 2002 | 1 | |
| 40 | Sat 06 Apr 2002 | 1 | |
| 41 | Sat 13 Apr 2002 | 1 | |
| 42 | Mon 15 Apr 2002 | 2 | Easter Mon |
| 43 | Sat 20 Apr 2002 | 1 | |
| 44 | Sat 27 Apr 2002 | 1 | |
| 45 | Sun 05 May 2002 | 1 | Final day, flag=6 |

## Remaining work

1. Byte-exact port of **FUN_00533eb0** (flag-snap). This is the last
   date-transform block needed for round-record byte match.
2. Byte-exact port of **FUN_0066f280** (walker / nominal-date stream).
3. Byte-exact port of **FUN_00668890** (round-robin driver — writes the
   0xFF sentinels and is presumed to populate team-pair bytes).
4. Identify where the H/A team pair bytes get written (not by
   FUN_0066f3b0; not in the captured buffer either). Suspect a later
   pass driven by a comp-tick or "start-of-season" hook.
5. Same runtime capture for other English divs (Premier / First /
   Third / Conf) — with contract rule: not yet, English Second Division
   only.
6. Choose the Rust representation only after 1–4 are done.

- **Player-visible**: YES  |  **Save-affecting**: YES
- **Opened**: 2026-09-13
- **Runtime capture**: 2026-09-13 (superseded static claim of "Sun 12 Aug")
- **Resolved**: —
