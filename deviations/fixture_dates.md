# Deviation: League fixture dates

## OPEN TRADITIONAL DEVIATION — matrix_perturb resolver slot-vs-club-id (2026-09-14, C10.6)

C10.6 five-league differential (capture
`20260914_131616_five_leagues_*`) revealed the production
`matrix_perturb + StadiumClubResolver` combo has a slot-vs-club-id
bug in E2/E3 phases:

| League | Rust P2 mismatches / total |
|---|---|
| Premier | 15/20 |
| First | 21/24 |
| Second | 9/24 |
| Third | 0/24 |
| Conference | 0/22 |

`StadiumClubResolver` maps SLOT → stadium. Phase D shuffles clubs
across slots. E2/E3 then call `resolver.e2_pair_shares_69(i, j)`
with POST-shuffle slots, but the resolver returns pre-shuffle
stadium data → stale.

Driver output remains BehaviourallyExact for all 5 (0/all
ordered mismatches with captured P2), so fixture generation is
correct for the shipped 2001-02 data. But the perturb ALGORITHM
is not reproducing the exe's P2. Third/Conf pass 0 by coincidence
(their Phase-D outcome avoids the E2/E3 mismatch path); do not
read as evidence the port is correct.

**Must be fixed before C11 production wiring closes.** See
`memory/perturb-resolver-slot-vs-club-bug.md` for the fix plan.

## OPEN TRADITIONAL DEVIATION — walker per-call capture gap (2026-09-14, C10.6)

C10.6 walker per-call differential shows 7-9 mismatches per league
across all shapes:

| League | walker Δ / n |
|---|---|
| Premier | 7/38 |
| First | 9/46 |
| Second | 8/46 |
| Third | 9/46 |
| Conference | 9/42 |

Most likely a Frida harness capture gap — `walker_step`'s 8th arg
`special_comp_id` (sp+0x20) is not captured. Diff feeds
`i32::MIN` for that arg, mis-branching on rounds where the exe
passes a non-sentinel value.

Not a C11 blocker: driver output remains 0-diff for all 5. Needed
to promote `walker_confidence` above `StructurallyVerified`. Fix:
extend `gdi_five_league_lineage.py` to capture sp+0x20, re-run
harness. See `memory/walker-capture-gap.md`.

## OPEN TRADITIONAL DEVIATION — Conference-fallback stadium-expansion (2026-09-14, C6)

Traditional Conference-fallback (`sub_0055ec00` port at
`ada5945`) is **decision-exact** but **NOT yet behaviourally exact
on the passing branch**. The exe's stadium-gate `sub_00584150` is a
471-instruction function that does much more than gate — on a
passing candidate it ALSO:

* mutates the stadium record's `+0x3c`, `+0x40`, and `+0x44`
  capacity fields (max / current / peak-ever-required);
* updates 8 club-ledger financial fields via the exe's stadium-cost
  formula (`(need/1000 + 1) * 6000 + need) * 125 + local_14 * 75 +
  iVar10 * 50`);
* fires the "stadium expanded" news broadcast via `sub_0058a310`.

The C5+C6 port in `crates/cm-domain/src/eng_second_fixtures.rs`
covers only the **boolean-return subset** the fallback caller
observes. `stadium_meets_capacity_target` is byte-exact for that
return but the on-pass side effects are missing — meaning a
Traditional save where Conference is not selected and the fallback
fires with a passing candidate currently promotes the candidate
WITHOUT paying the expansion cost, updating stadium capacity, or
firing the "stadium expanded" news.

**This deviation MUST NOT be closed out of C11.** Follow-up commit
required: port the full expansion transaction as a companion helper
(the "apply" layer for the fallback's Promoted outcome). Rough
plan: extract stadium-cost formula → return an
`ExpansionTransaction { capacity_delta, cost, ledger_deltas }`
alongside the boolean → apply-layer writes stadium fields, deducts
cost, fires news.

## Build provenance (2026-09-14, C1′ pass)

TWO source binaries exist:

* **`cm0102_GDI.exe`** — GDI build. **Authoritative for this project.**
  Runtime captures and all byte-exact goldens target this binary. All
  VAs in this file — driver, perturb, walker, eng_second ctor and
  schedule-getter — are GDI VAs unless explicitly prefixed
  `DirectDraw VA:`.
* **`cm0102.exe`** — DirectDraw build. Source of most Ghidra
  decompile reports; used only as a corroborating cross-reference.

The two builds' VAs differ by non-uniform, region-dependent deltas.
Never translate between them by applying a fixed offset. See
`memory/gdi-vs-directdraw-builds.md` for the calibration table.

## Address-labelling correction (2026-09-14, C1 pass)

The round-robin driver is a single outer function at **`0x00668450`**,
2336 bytes / 675 instructions, ending `0x00668d70` (linear-carve
segment `03577_sub_00668450.asm`). Older notes in this file called it
"`FUN_00668890`" — that address is an **internal label / basic-block
inside the same outer function**, not a distinct function. All
`FUN_00668890` mentions below have been mass-substituted to
`FUN_00668450`. The Ghidra decompile file was initially named after
the inner label, so references to `00668890.c` continue to point at
the same source body; treat line numbers there as "line N of the
`sub_00668450` decompile".

The 552/552 ordered-diff pass in `examples/driver_diff_via_p2.rs` was
run against the ported outer function, so byte-exact evidence stands
under the corrected label.

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
  - **`eng_second.cpp` ctor** — GDI `sub_0055f240` / DirectDraw `0x0055f040` (comp id 9). Pillar-15 archaeology 2026-09-14.
  - `eng_third.cpp` at `0x00560b40`  — Third     ctor (comp id 10)
  - `eng_conf.cpp`  at `0x005577a0`  — Conference ctor (comp id 93)
  - `FUN_00560320` — sets `+0xba = vtable+0x3c()` (virtual dispatch to
    schedule-getter)
  - **eng_second schedule-getter** — GDI `sub_0055f540` (3692 bytes) / DirectDraw `0x0055f340` (vtable slot +0x3c on
    vtable `0x00957dc8`).
  - `FUN_0066f3b0` — round-record writer; 9 args
    `(buffer, round_idx, day, month, day_off, flag, type, year, prize)`;
    writes packed date, type, prize into the 65-byte record.
  - `FUN_00533b50` — date encoder (day/month/year → 4-short pack).
  - **`FUN_00533eb0`** — flag-snap helper (weekday snap-back); NOT YET
    PORTED. This is the key remaining pre-buffer step.
  - `FUN_00668450` — round-robin driver (writes the +0x09..+0x0b 0xFF
    sentinels + is presumed to fill team-pair bytes later).
  - `FUN_0066f280` — schedule walker / nominal-date generator.

## VERIFIED EXACT — Runtime capture 2026-09-13

Full report: `reports/fixture_disasm/RUNTIME_CAPTURE_REPORT.md`.
Raw artefacts: `reports/fixture_disasm/runtime/20260913_113106_*`.

Method: Frida in-process attach + direct call to the schedule-getter (GDI `sub_0055f540` / DirectDraw `FUN_0055f340`) with
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

## Update 2 (2026-09-13, later): team-pair pipeline identified

Complete report: `reports/fixture_disasm/TEAM_PAIRING_REPORT.md`.

### Answers

- **FUN_00668450 IS called from eng_second_ctor** at `0x0055f136` (my
  earlier "not called from schedule-getter" was correct but
  misleading — the ctor calls it directly, but I had only direct-
  called the schedule-getter). See ctor decompile line 50.
- **FUN_0066f410** is the second writer function inside FUN_0055f340
  — every writer call is paired with a slot-writer call. Slot-writer
  owns bytes +0x05..+0x0b of each round record, initialising slot 0
  to sentinels (-1, -1, -1, 0). Not previously found because I only
  hooked one writer function; add_writer_hook and re-capture confirm.
- **65-byte round-record layout**: header (5B, FUN_0066f3b0) + 8×7B
  fixture sub-slots (FUN_0066f410) + tail (4B, FUN_0066f3b0). The
  8-sub-slot structure was completely unknown before.
- **Team pairings live in 79-byte TFixture records** (not in the
  schedule buffer). Built on-stack by FUN_00668450 (round-robin
  driver), inserted into a `TFixList` calendar via FUN_00594d00
  (`fix_man.cpp`).
- **TFixList container**: 0x1184 bytes per season year, chained
  linked-list. Indexed by [year][doy][slot 0..2][comp-idx] of
  TList<TFixture*>. Save file: `fixtures.<year>.tmp`.
- **552 fixtures per Div-2 season**: 24 clubs × 23 rounds × 2 legs = 552.

### Rust changes (this update)

- Added `apply_flag_snap` (byte-exact FUN_00533eb0).
- Added `write_slot` (byte-exact FUN_0066f410).
- Added `build_eng_second_schedule` — reproduces the entire 2990-
  byte buffer byte-exact vs runtime capture.
- Test `full_buffer_matches_runtime_capture` compares all 2990 bytes.

### Remaining (still open)

1. Byte-exact port of **FUN_00668450** (~425 lines) — the round-
   robin driver + fixture builder.
2. Byte-exact port of **FUN_00594d00** (fixture inserter, ~250
   lines) and the TFixList container (0x1184-byte struct).
3. Byte-exact port of **FUN_0066f280** walker — done conceptually,
   pending FUN_00668450 to know how it's called.
4. Runtime capture of a concrete pair sequence (blocked on safe
   direct-call of eng_second_ctor — currently crashes the exe).
5. Save-file compatibility with the exe's `fixtures.<year>.tmp`
   format.

- **Player-visible**: YES  |  **Save-affecting**: YES
- **Opened**: 2026-09-13
- **Update 1**: 2026-09-13 (12 Aug → 11 Aug correction, flag-snap decoded)
- **Update 2**: 2026-09-13 (team-pair mechanism located)
- **Resolved**: —
