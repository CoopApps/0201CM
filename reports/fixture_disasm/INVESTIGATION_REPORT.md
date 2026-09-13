# English Second Division fixture pipeline — investigation report

Date: 2026-09-13. Under project ACCURACY CONTRACT.

Reporting the 10 items from the user's ordering. Each item is a
concrete outcome, not a claim.

---

## 1. Full runtime FUN_0066f3b0 capture

`reports/fixture_disasm/runtime/20260913_113106_direct_call.jsonl` —
46 `roundwriter` events. Method: Frida in-process attach on the
running `cm0102.exe` (PID captured at 7616), direct call to
`FUN_0055f340` with `arg1 = 0xFF` on a synthetic comp record
(`this+0x40 = 2001`), bypassing the UI menu path.

Every round-writer invocation captured:

- Full 9-argument stack read (`round_idx, day, month, day_off, flag,
  type, year, prize`).
- Return address (each in the range `0x0055f3db..0x0055ffca` — all
  inside FUN_0055f340 body).
- Buffer pointer (identical across all 46 calls — same 2990-byte
  buffer being populated).

The runtime capture supersedes and corrects the earlier static
analysis in two specific ways (see §5).

## 2. Explanation of the 134 static call sites

Static disassembly counted 134 `E8` relative-calls to `FUN_0066f3b0`
inside `FUN_0055f340`. Runtime attribution:

- **46 calls fire** on the normal English Second Division construction
  path (`param_1 = 0xFF`, the value FUN_00560320 passes at line 26).
- **88 calls sit on other `param_1` branches** (playoff, promotion,
  reset, restart) that are not reached for a standard save-init.

So 134 = 46 + 88. There is no "hidden" call. The static tool over-counted
because it did not distinguish live vs dead branches. Runtime is the
authority.

## 3. Exact 2990-byte normal Second Division schedule dump

`reports/fixture_disasm/runtime/20260913_113106_direct_buffer_0.bin` —
2990 bytes, `SHA` calculable, produced by the exe under Frida direct
call.

Buffer size math: `malloc(0xbae) = 2990 = 46 × 65`.

## 4. 46-record decoded structure

Per-record layout (65 bytes each), verified from the capture:

| Offset | Size | Meaning | Set by |
|--------|------|---------|--------|
| +0x00  | i16  | day-of-year (0-indexed, Jan 1 = 0) | FUN_0066f3b0 |
| +0x02  | i16  | year offset from season base year (0 or 1) | FUN_0066f3b0 |
| +0x04  | u8   | type: 1 = league round, 2 = midweek | FUN_0066f3b0 |
| +0x05..+0x08 | 4 × u8 | zero | — |
| +0x09..+0x0b | 3 × u8 | 0xFF sentinel triple | UNKNOWN inline code inside FUN_0055f340 (not writer, not driver, not walker — see §9) |
| +0x0c..+0x3c | 49 × u8 | zero | — |
| +0x3d..+0x40 | i32 | prize / round-int (0 in this capture) | FUN_0066f3b0 |

The full 46-round date table is in `RUNTIME_CAPTURE_REPORT.md` §4.
Round 0 = **Sat 11 Aug 2001**. Round 45 = **Sun 5 May 2002**. Every
row's weekday-check reproduces the real Nationwide Div-1 2001/02
fixture-list weekday (Saturday for league rounds, Bank Holiday Monday
or Wed/Tue for midweek rounds).

## 5. Runtime verification of the apparent 12 Aug 2001 date

**The exe stores Sat 11 Aug 2001 for round 0, not Sun 12 Aug 2001.**

My earlier static claim was wrong. The confusion arose because the
writer receives `(day=12, month=Aug, flag=5)` as arguments, and my
static tool read the day-12 input as the final stored value. In fact
the writer passes those inputs through:

```
pack_date(day=12, mon=7, year=2001)  →  [223, 2001, 0, 0]   // Sun 12 Aug
apply_flag_snap(&mut buf, flag=5)    →  [222, 2001, 0, 0]   // Sat 11 Aug
```

The `apply_flag_snap` step rotates the packed date to the previous
occurrence of weekday `flag` (0=Mon..6=Sun). This is `FUN_00533eb0` in
the exe, called from inside `FUN_00533b50`. The snap direction is
determined by the shortest signed distance in [-3..3]: for round 0,
Sun→Sat is -1; for round 37 (input Sun 17 Mar 2002, flag=1 Tue), the
distance +5 rounds through to -2 and produces Tue 19 Mar 2002. Both
directions are captured live and reproduced by the Rust port.

## 6. FUN_00668890 reconstruction — CURRENT STATE

**Byte-exact port not yet complete.** Structural characterisation:

- 425-line decompile at `0x00668890`.
- Signature: `__fastcall (int *comp)` — takes a competition pointer;
  first field read is `*(short*)(comp + 0x3e)`.
- Uses `FUN_0066f280` (walker, §7) as an internal helper — the only
  caller of the walker in the entire executable.
- **Not invoked from FUN_0055f340.** Zero calls captured during the 46
  writer runs; verified twice, once with each hook script.
- 202 call sites total across the exe (see `find_callers.py` output).
  These live in the country-league constructors and cup-tick handlers.

Where team pairings actually get written into the schedule buffer is
therefore NOT FUN_00668890. Best hypotheses (each falsifiable via
Frida capture on a real save-init):

- (a) A separate per-comp "start-of-season" pass that reads
  `comp+0xba` (the date template) and writes pairings.
- (b) A per-round tick that fills the next round's pair fields lazily
  on the day it becomes visible.
- (c) FUN_00668890 IS called from `eng_second_ctor` via a slot I
  did not hook. Ruled out for the schedule-getter itself; ruled in for
  the ctor overall only after hooking the ctor return path.

Deciding among (a)/(b)/(c) requires driving a save fully through
init (via the UI, since direct-call bypasses the ctor). That is the
next runtime step.

## 7. FUN_0066f280 reconstruction — decoded

72-line decompile at `0x0066f280`. **Fully understood:**

```
walker(param_1, state_ptr, disc, p4, p5, p6, flag_byte):
    // Special-case: disc matches DAT_009bbba8 (magic const)
    if disc == DAT_009bbba8:
        if param_1 == 2: return 4
        if param_1 == 5: return 3
    else if p5 > 1 and p4 > 7:
        if flag_byte & 0x40:  return param_1 + 1
        if flag_byte & 0x80:  // Berger half-swap
            p1 = param_1 + 1
            if p1 == p6 / 2: return p6 - 1
            if p1 == p6:     return p6 / 2
            return p1
        state = *state_ptr
        if state == 1: *state_ptr = 2; return (param_1 - 1) with wrap to p6-1
        if state == 2: *state_ptr = 3; return (param_1 - 1) with wrap to p6-1
        if state == 3: *state_ptr = 0; return (param_1 + 3) mod p6
        // state == 0 fall-through
        if (param_1 < p6 - 5) and FUN_008fc4f0(4) > 1:
            *state_ptr = 1
            return (param_1 + 3) mod p6
        return (param_1 + 1) mod p6
    return param_1 + 1
```

This is an RNG-perturbed circular walker: normal mode is +1 mod
team-count, but with `FUN_008fc4f0(4)` (game RNG mod 4, > 1 ⇒ p=0.5)
it occasionally jumps by +3 and enters a 4-state cycle that produces a
short reverse-run before returning to +1. `flag_byte` bits 0x40 / 0x80
override with pure +1 or a Berger half-swap. This is exactly the
kind of primitive used in an RNG-derangement of the classical round-
robin, injecting non-uniformity without breaking the pairing invariant.

Not yet ported to Rust: waiting until FUN_00668890 is decoded so we
know exactly how the walker is used.

## 8. Original fixture-generation trace

`FUN_0055f340` (schedule-getter):
```
0x0055f340  vtable+0x3c entry, __thiscall
0x0055f34x  malloc(0xbae) = buf
0x0055f39x  (some inline init writes the 0xFF triples at +0x09..+0x0b?)
0x0055f3db  writer(buf, 0,  12,  7, 0,  5, 1, y2001, 0)
0x0055f404  writer(buf, 1,  19,  7, 0,  5, 1, y2001, 0)
...  [43 more writer calls, all with day_off = 0, y = 2001, flag ∈ {0,1,2,5}]
0x0055f8f8  writer(buf, 23, 22, 11, 0, -1, 1, y2001, 0)  ← flag=-1 begins
0x0055f922  writer(buf, 24, 26, 11, 0, -1, 1, y2001, 0)  ← Boxing Day
0x0055f94c  writer(buf, 25, 29, 11, 0, -1, 1, y2001, 0)
0x0055fb94  writer(buf, 26,  1,  0, 1, -1, 1, y2001, 0)  ← New Year; day_off flips to 1
0x0055fcc6  writer(buf, 27, 13,  0, 1,  5, 1, y2001, 0)  ← flag returns to 5
...
0x0055ffca  writer(buf, 45,  6,  4, 1,  6, 1, y2001, 0)  ← final round, Sun snap
0x0055????  return buf; ← writes comp+0xa9 = 46 (round count), comp+0xba = buf
```

Static return address ranges group into three sub-branches inside
FUN_0055f340:

- `0x0055f3db..0x0055f775` — pre-Christmas rounds 0..22 (flag ∈ {0,1,2,5})
- `0x0055f8f8..0x0055f94c` — holiday rounds 23..25 (flag = -1)
- `0x0055fb94..0x0055ffca` — post-New-Year rounds 26..45 (day_off = 1)

The three branches are dead-code selected by the arg1 discriminant
plus internal comp-year checks. Only one branch is live per invocation.

## 9. Remaining unknowns

- **Who writes 0xFF at +0x09..+0x0b in every round record?** Not
  FUN_0066f3b0 (which touches +0x00/+0x02/+0x04/+0x3d only). Not
  FUN_00668890 (never called). Not FUN_0066f280 (never called).
  Suspect: an inline `memset`-style loop between the malloc and the
  first writer call. To find: hook `memset`/`FUN_00933d40`/small
  `mov [buf + N], 0xFF` writes, or single-step the exe's memory-writes
  around `0x0055f39x`.
- **How team pairs (home_id, away_id) get filled in.** Presumably a
  separate pass reads `comp+0xba` (the date template) and writes team
  IDs. Ownership candidates in decreasing likelihood: FUN_00668890
  called from a per-comp start-of-season handler, cup-draw code, or a
  vtable slot I haven't traced.
- **Whether FUN_00533eb0 does anything else besides day-of-year rotation.**
  Its C++ EH frame suggests it may write additional data via a
  destructor path. All 42 live invocations produce a single doy shift
  and no other buffer changes; year/is_leap/reserved fields are
  identical before/after. So functionally: doy shift only. Formally:
  need to confirm the destructor pattern doesn't fire in some other
  edge case (e.g. year rollover across Dec 31).
- **Whether FUN_00668890 has more than one entry point.** Its 202
  callsites all reach the same VA; no evidence of tail-calls or jump
  aliases. Assumed unique entry.
- **Whether `flag_byte & 0x40 / 0x80` cases in the walker matter for
  English Second Division.** Depends on how FUN_00668890 calls the
  walker. Not yet observed.

## 10. Rust code changed

`crates/cm-domain/src/exe_date.rs` (see commit `[hash]`):

- **Added** `apply_flag_snap(&mut [i16; 4], flag: i32)` — byte-exact
  port of `FUN_00533eb0`. Justification: needed for the writer to
  produce buffer-matching bytes. Verified against all 42 flag-snap
  invocations in the runtime capture (`snap_all_42_runtime_calls`
  test).
- **Modified** `write_round_record` to call `apply_flag_snap`. Was
  incomplete before; now completes the exe's date-write chain.
  Justification: without it, `snap_all_42` fails and the full-buffer
  test fails.
- **Added** `ENG_SECOND_2001_TEMPLATE` — the 46-tuple `(day, month,
  day_off, flag, type)` sequence extracted verbatim from the runtime
  capture. Justification: this is the exe's real fixture template;
  encoding it as a const captures the recovered data without needing
  to keep the exe running.
- **Added** `build_eng_second_schedule(year)` — produces the 2990-byte
  buffer. Justification: gives cm-domain a callable that reproduces
  the exe's date/type template. Not yet wired into any game path per
  the user's explicit rule "do NOT wire the partially recovered dates
  into the game yet."
- **Added** `full_buffer_matches_runtime_capture` test — compares
  every FUN_0066f3b0-owned byte in the 2990-byte buffer against the
  runtime capture. Passes.

No changes anywhere else in the tree. `SeasonStart::resolve` in
`league_calendar.rs` is still the stub used by
`generate_double_round_robin`. The stub will be replaced only after
the team-pairing mechanism is recovered.

---

## Provenance summary (per rule J)

| Claim | Source |
|-------|--------|
| 46 writer calls per eng_second construction | `runtime/20260913_113106_direct_call.jsonl` (51 events, 46 = op:roundwriter) |
| 2990-byte buffer content | `runtime/20260913_113106_direct_buffer_0.bin` |
| flag = target weekday, snap = -iVar3 | `runtime/20260913_113814_v2_events.jsonl` (42 flagsnap pairs) |
| Round 0 = Sat 11 Aug 2001 | Same buffer, decoded by `decode_buffer.py` |
| FUN_00668890 never called | Same v2 capture (0 events op:rr_driver_enter) |
| FUN_0066f280 never called | Same v2 capture (0 events op:walker_enter) |
| 202 static callers of FUN_00668890 | `find_callers.py` E8-scan of .text |
| 1 static caller of FUN_0066f280 at 0x668ddd | Same scan |
| FUN_0066f280 walker mechanism | Ghidra decompile at `decompiled/0066f280.c` |
| Full-buffer byte match | `cargo test -p cm-domain full_buffer_matches_runtime_capture` |

No claim above depends on football-domain knowledge; football fixture
lists were consulted only for corroboration, never as source of truth.
