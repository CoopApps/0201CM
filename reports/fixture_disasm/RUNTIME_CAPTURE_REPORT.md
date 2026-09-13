# English Second Division schedule — runtime capture report

Date: 2026-09-13. Executable: `D:/cm0102/cm0102.exe` (SHA identified in
compare_exes.py). Method: Frida in-process attach with direct call to
`FUN_0055f340` on a synthetic comp record (`this+0x40 = 2001`), bypassing
the UI menu path. Script: `reports/fixture_disasm/direct_call_schedgetter.py`.

Raw artefacts (do not edit — captured verbatim):

- `runtime/20260913_113106_direct_call.jsonl` — 51 events (hook_installed,
  calling_getter, 46 × roundwriter, getter_returned, buffer_dump, done)
- `runtime/20260913_113106_direct_buffer_0.bin` — the exact 2990-byte
  schedule buffer returned by 0x0055f340.

## 1. Full FUN_0066f3b0 call trace

Exactly **46 calls** fire during a normal-path construction (arg1 = 0xFF).
Every call is at a distinct return address in `0x0055f3db..0x0055ffca`
(the schedule-getter body). Each call writes exactly one 65-byte round
record. No writer call fires outside this range and no round is written
twice.

Full log with args + return addresses is in the `.jsonl` above.

## 2. Why "134 static call sites" but 46 runtime calls

The static `bl` scan found 134 references to FUN_0066f3b0 inside 0x0055f340.
Runtime with `arg1 = 0xFF` (the normal English Second Division construction
path, taken from FUN_00560320 line 26) traverses exactly one branch, which
contains 46 of those call sites. The other 88 sit on:

- Other `param_1` byte discriminants (playoff-only, restart-only, promotion
  swap paths) — not hit for standard construction.
- Fall-through paths for other constants selected inside the same body.

**134 = 46 (normal) + 88 (other-mode branches).** Direct-call proof: 46
calls, one per round record, in-order round_idx 0..45. Nothing else fires.

## 3. The 2990-byte buffer — decoded

`bufsize = 46 × 0x41 = 2990`. Per-record layout **as verified from live
capture**:

| Offset | Size | Meaning | Written by |
|--------|------|---------|-----------|
| +0x00  | i16  | day-of-year (0-indexed, Jan 1 = 0) | FUN_0066f3b0 |
| +0x02  | i16  | year offset (0 for pre-Jan 1, 1 after) | FUN_0066f3b0 |
| +0x04  | u8   | type: 1 = league round, 2 = midweek | FUN_0066f3b0 |
| +0x05..+0x08 | 4 × u8 | zero (unused) | — |
| +0x09..+0x0b | 3 × u8 | 0xFF sentinels (per-team? per-fixture?) | not FUN_0066f3b0 — some post-writer init loop I have not yet identified |
| +0x0c..+0x3c | zero | (unused so far) | — |
| +0x3d..+0x40 | i32  | prize / round int, all 0 in our capture | FUN_0066f3b0 |

**Every** round record shows the same 0xFF triple at +0x09..+0x0b. That
signature is stamped by code that runs **inside FUN_0055f340 but not
inside FUN_0066f3b0**. Two candidates from the static disassembly:
`FUN_00668890` (round-robin driver — most likely, since it iterates all
rounds) or a small inline loop between call chains. Task E will resolve.

## 4. The 46 round dates (buffer-truth)

```
round | date            | type | meaning
------|-----------------|------|--------
   0  | Sat 11 Aug 2001 | 1    | League round  1
   1  | Sat 18 Aug 2001 | 1    | League round  2
   2  | Sat 25 Aug 2001 | 1    | League round  3
   3  | Mon 27 Aug 2001 | 2    | Midweek (Bank Holiday Mon)
   4  | Sat 01 Sep 2001 | 1    | League round  4
   5  | Sat 08 Sep 2001 | 1    | League round  5
   6  | Wed 12 Sep 2001 | 2    | Midweek
   7  | Sat 15 Sep 2001 | 1    | League round  6
   8  | Sat 22 Sep 2001 | 1    | League round  7
   9  | Sat 29 Sep 2001 | 1    | League round  8
  10  | Sat 06 Oct 2001 | 1    | League round  9
  11  | Sat 13 Oct 2001 | 1    | League round 10
  12  | Tue 16 Oct 2001 | 2    | Midweek
  13  | Sat 20 Oct 2001 | 1    | League round 11
  14  | Tue 23 Oct 2001 | 2    | Midweek
  15  | Sat 27 Oct 2001 | 1    | League round 12
  16  | Sat 03 Nov 2001 | 1    | League round 13
  17  | Sat 10 Nov 2001 | 1    | League round 14
  18  | Sat 17 Nov 2001 | 1    | League round 15
  19  | Sat 24 Nov 2001 | 1    | League round 16
  20  | Sat 01 Dec 2001 | 1    | League round 17
  21  | Sat 08 Dec 2001 | 1    | League round 18
  22  | Sat 15 Dec 2001 | 1    | League round 19
  23  | Sat 22 Dec 2001 | 1    | League round 20 (flag=-1)
  24  | Wed 26 Dec 2001 | 1    | Boxing Day (flag=-1)
  25  | Sat 29 Dec 2001 | 1    | League round 21 (flag=-1)
  26  | Tue 01 Jan 2002 | 1    | New Year (flag=-1)
  27  | Sat 12 Jan 2002 | 1    | League round 22
  28  | Sat 19 Jan 2002 | 1    | League round 23
  29  | Sat 02 Feb 2002 | 1    | League round 24
  30  | Sat 09 Feb 2002 | 1    | League round 25
  31  | Sat 16 Feb 2002 | 1    | League round 26
  32  | Tue 19 Feb 2002 | 2    | Midweek
  33  | Sat 23 Feb 2002 | 1    | League round 27
  34  | Sat 02 Mar 2002 | 1    | League round 28
  35  | Wed 06 Mar 2002 | 2    | Midweek
  36  | Sat 09 Mar 2002 | 1    | League round 29
  37  | Tue 19 Mar 2002 | 2    | Midweek
  38  | Sat 23 Mar 2002 | 1    | League round 30
  39  | Sat 30 Mar 2002 | 1    | League round 31
  40  | Sat 06 Apr 2002 | 1    | League round 32
  41  | Sat 13 Apr 2002 | 1    | League round 33
  42  | Mon 15 Apr 2002 | 2    | Easter Mon
  43  | Sat 20 Apr 2002 | 1    | League round 34
  44  | Sat 27 Apr 2002 | 1    | League round 35
  45  | Sun 05 May 2002 | 1    | League round 36 (final day, flag=6)
```

**Round 0 = Sat 11 Aug 2001** — matches the real-world English Second
Division 2001/02 opening weekend. My earlier static "exe stores Sun 12
Aug" claim was WRONG (see §6).

## 5. `flag` semantics — recovered

The writer receives `(day, month, day_off, flag, type, year, prize)`.
`flag` is the **target weekday** to snap to (0=Mon..6=Sun); the exe helper
`FUN_00533eb0(flag, packed_date)` snaps the packed day-of-year **back** to
the previous occurrence of that weekday. Evidence:

| Round | Input (day,mon,flag) | Nominal weekday | Snapped date | Δ days |
|-------|----------------------|-----------------|--------------|-------|
| 0     | (12, Aug, 5=Sat) | Sun 12 Aug | Sat 11 Aug | -1 |
| 3     | (28, Aug, 0=Mon) | Tue 28 Aug | Mon 27 Aug | -1 |
| 6     | (13, Sep, 2=Wed) | Thu 13 Sep | Wed 12 Sep | -1 |
| 12    | (17, Oct, 1=Tue) | Wed 17 Oct | Tue 16 Oct | -1 |
| 14    | (24, Oct, 1=Tue) | Wed 24 Oct | Tue 23 Oct | -1 |
| 22    | (16, Dec, 5=Sat) | Sun 16 Dec | Sat 15 Dec | -1 |
| 32    | (20, Feb, 1=Tue) | Wed 20 Feb | Tue 19 Feb | -1 |
| 35    | (7, Mar, 2=Wed)  | Thu 7 Mar  | Wed 6 Mar  | -1 |
| 42    | (16, Apr, 0=Mon) | Tue 16 Apr | Mon 15 Apr | -1 |
| 45    | (6, May, 6=Sun)  | Mon 6 May  | Sun 5 May  | -1 |

All snaps are exactly -1 day (nominal is always one day past target
weekday). Rounds 23–26 use `flag = -1` (no snap) — those are the
holiday period fixtures (22 Dec, Boxing Day, 29 Dec, New Year) whose
input date IS the target date.

**Interpretation of the nominal date stream**: someone (the walker
FUN_0066f280?) generates a target date roughly on the correct weekday and
then the writer's flag-snap corrects it back to the exact intended
weekday. The nominal-plus-snap trick lets the walker use a single
"forward N days" rule and let the snap fix small drift.

Recovering FUN_00533eb0 byte-exact is task E prerequisite work.

## 6. Correction to earlier static-analysis claim

Previous deviation register said:
> Round 0 = Sun 12 Aug 2001 (exe stores Sunday, real football played Sat 11 Aug)

**This was wrong.** The exe stores **Sat 11 Aug 2001**. My port's
`pack_date(12, 7, 2001) = [223, ...]` gives the pre-snap value. The
writer then chains through `FUN_00533eb0(flag=5, ...)` to produce the
stored `[222, ...]`. My exe_date.rs is **incomplete** — the flag-chain
is not ported. Any test that consumed `pack_date` output as final was
implicitly testing pre-snap data.

## 7. Rounds using `flag = -1` (no-snap holiday overrides)

Rounds 23 (Sat 22 Dec), 24 (Wed 26 Dec), 25 (Sat 29 Dec), 26 (Tue 1 Jan)
all use `flag = -1`. Ret addresses are `0x55f8f8`, `0x55f922`, `0x55f94c`,
`0x55fb94` — a distinct sub-branch of the schedule-getter (roughly 400
bytes later than the flag-based path). This is the **holiday hardcode
block**: December 22, Boxing Day, December 29, and New Year's Day are
fixed calendar dates, not week-relative, and get bypassed around the
snap. That's exactly what a real English football fixture calendar
requires.

## 8. What still needs decoding

- **FUN_00533eb0** (the flag-snap helper) — task E prerequisite. Snap
  direction and boundary rules (e.g. what if the nominal date is already
  the target weekday?) need byte-exact reproduction. My table above
  suggests it is always -1 for our capture, but the exe may snap forward
  for some inputs.
- **FUN_00668890** (round-robin driver) — writes the +0x09..+0x0b `0xFF`
  triples and, we suspect, populates the (home_id, away_id) pairs into
  the remaining bytes of each 65-byte record **at a later stage** (not
  during the schedule-getter). The captured buffer contains only the
  date + type stubs; pairings will fire when the competition's per-round
  driver runs. Static analysis of FUN_00668890 required to prove this.
- **FUN_0066f280** (walker / nominal-date generator) — determines the
  `(day, month, day_off, flag, type)` stream that the schedule-getter
  feeds to FUN_0066f3b0. Static analysis in progress; hooks are already
  installed in `hook_schedule.py`.
- **Where the 65 bytes actually get filled** with match data (home team,
  away team, kick-off time, result, TV flag). None of the captured
  bytes at +0x05..+0x3c look like team ids yet. The buffer as returned
  is a date/type template only.

## 9. Rust code impact

- `crates/cm-domain/src/exe_date.rs::pack_date` is byte-exact **only**
  for the pre-snap step. It must NOT be advertised as "produces buffer
  bytes"; the buffer bytes come from `pack_date` **then**
  `FUN_00533eb0` (unported).
- Comment in exe_date.rs is updated to point at this report.
- No wiring into `generate_double_round_robin` yet, per user rule.

## 10. Provenance

Every claim above is derivable from:
- The .jsonl trace (46 writer calls, args, return addresses)
- The .bin buffer (2990 bytes, byte-exact)
- Static return-address arithmetic (all in 0x0055f3db..0x0055ffca)
- Cross-check with real 2001/02 English Second Division fixture list
  (used **only** as corroboration, never as source of truth)

Nothing above depends on football-domain assumption.
