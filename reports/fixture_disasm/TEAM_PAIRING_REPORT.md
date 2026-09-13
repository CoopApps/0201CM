# English Second Division team-pairing pipeline — investigation report

Date: 2026-09-13. Under project ACCURACY CONTRACT.

The 11 items in the user's ordering. Confidence labels per revised
scheme (VERIFIED EXACT / VERIFIED STRUCTURE, SEMANTICS PARTIAL /
HYPOTHESIS / TEMPORARY STUB).

---

## 1. 65-byte round-record byte ownership map — VERIFIED EXACT

Established by combining the runtime capture with the static
decompiles of `FUN_0066f3b0` and the newly identified sibling
`FUN_0066f410`.

Each round record decomposes into three fixed parts and an
**8-sub-slot** middle section:

```
+0x00..+0x04  header       (5 bytes,  owned by FUN_0066f3b0)
+0x05..+0x3c  8 × 7-byte fixture sub-slots  (owned by FUN_0066f410)
+0x3d..+0x40  tail         (4 bytes,  owned by FUN_0066f3b0)
```

Header (+0x00..+0x04):

| Offset | Size | Field | Writer | Reader(s) |
|--------|------|-------|--------|-----------|
| +0x00  | i16  | day-of-year (0-idx, post-flag-snap) | FUN_0066f3b0 | FUN_00668890 (round-robin driver), FUN_00594d00 (fixture inserter) |
| +0x02  | i16  | year offset from season base year | FUN_0066f3b0 | FUN_00668890, FUN_00594d00 |
| +0x04  | u8   | round type (1=league, 2=midweek) | FUN_0066f3b0 | FUN_00668890 (reads +0x04 as weekday code copied to fixture+0x3c) |

Each fixture sub-slot (7 bytes at base + 0x05 + n*7, n ∈ 0..7):

| Offset in slot | Size | Field | Writer | Reader(s) |
|----------------|------|-------|--------|-----------|
| +0x00..+0x03 (record +0x05..+0x08 for slot 0) | u32 | aux payload | FUN_0066f410 (`param_7`) | FUN_00668890 line ~366 (this is the venue/prize field passed to FUN_00845380) |
| +0x04 (record +0x09 for slot 0) | i8 | field_a domain [-1..6] | FUN_0066f410 (`param_4`) | FUN_00668890 |
| +0x05 (record +0x0a for slot 0) | i8 | field_b domain [-1..2] | FUN_0066f410 (`param_5`) | FUN_00668890 |
| +0x06 (record +0x0b for slot 0) | i8 | field_c domain [-1..4] | FUN_0066f410 (`param_6`) | FUN_00668890 (checks for values 3 and 4 — likely "reset/replay round" markers) |

Tail (+0x3d..+0x40):

| Offset | Size | Field | Writer | Reader(s) |
|--------|------|-------|--------|-----------|
| +0x3d  | i32  | prize / round-int | FUN_0066f3b0 | FUN_00668890 |

Classification:
- Header +0x00..+0x04: **per-round, invariant** for a given comp/season.
- Sub-slots: **per-fixture**, but at construction only slot 0 is
  written to sentinels; slots 1..7 are uninitialised heap memory
  (observed 0x00 in the fresh capture only because malloc happened
  to return zero-filled pages).
- Tail +0x3d..+0x40: **per-round, invariant** at construction (0);
  possibly filled later by TV/prize allocator (see [[deferred-awards-engine]]).

Evidence in `crates/cm-domain/src/exe_date.rs::tests::full_buffer_matches_runtime_capture` — the Rust port now reproduces all 2990 bytes of the runtime buffer.

## 2. First post-schedule-getter mutations of the live buffer — VERIFIED STRUCTURE, SEMANTICS PARTIAL

Static evidence via decompile of FUN_00668890 (round-robin driver) —
confirmed by agent report 2 (a3ef5f2b) plus my inspection:

- `FUN_00668890` is called from `eng_second_ctor` at 0x0055f136,
  **immediately after** the roster populator `FUN_005601d0` returns.
- It **reads** `comp+0xba` (schedule buffer) — offsets +0x00, +0x02,
  +0x0b, +0x3d.
- It does **not** modify the schedule buffer directly. Its output
  goes into stack-local 79-byte fixture records that are then handed
  to `FUN_00594d00`.

**Runtime confirmation blocked**: my direct-call to FUN_0055f040
crashed the process (side effects on the global comp pool). This is
expected — the ctor calls `FUN_006674d0()` at line 20 which registers
the new comp with a global pool. Attaching to a running save's comp
pool with a synthetic comp record is unsafe. Confirmed runtime
capture would require driving the exe through the real save-init
menus.

Buffer mutations after schedule-getter are therefore:
- **read-only during round-robin** (proven statically).
- The 8 sub-slots per round remain at their construction-time state
  (slot 0 sentinels; slots 1..7 malloc-uninit). No later code
  observed to fill them; the sub-slot mechanism appears to be a
  cup-round overlap encoding used by non-league comps.

## 3. Actual fixture-object creation — VERIFIED STRUCTURE, SEMANTICS PARTIAL

**Fixture object**: 79 bytes (0x4f). Built on the stack inside
FUN_00668890 (`local_274 .. local_227`), then handed to
`FUN_00594d00(fixture, 1)` in `fix_man.cpp` for calendar insertion.

**Source file**: `fix_man.cpp` (Ghidra attributes via string
constant `s_C__dev_CM3_00_01_cm3_code_comp_f_009b8844` — 23
functions in range 0x00594370..0x00599b20).

**Container**: `TFixList` (0x1184 bytes), one per season year,
chained via `+0x112c`. Body is a `[day-of-year][slot 0..2][comp-idx]`
array of `TList<TFixture*>` heads. Look-up API is `FUN_00596590`.

**Save/load**: fixtures serialise to `fixtures.<year>.tmp`; save
writer `FUN_005966e0`, loader `FUN_00596a30`. Pointer fields
(club, comp, referee) round-trip as indices, swizzled to pointers on
load via `(idx * stride) + DAT_00acd5b*` where strides are:
- Club: 0x245 (581) — matches known club record size.
- Comp: 0x6b  (107) — matches known comp record size.
- Person: 0x4e (78).

### 79-byte fixture layout

| Offset | Size | Field | Evidence in decompile |
|--------|------|-------|-----------------------|
| +0x00 | i32 | first int of comp record | FUN_00668890 (`local_274[0] = *comp`) |
| +0x04 | i32 | referee slot (index→ptr on load) | FUN_00596a30 swizzle: `local_238 = DAT_00acd5b8 + local_24c * 0x4e` |
| +0x08 | i32 | result / match-id (initial 0xffffffff) | `local_26c = 0xffffffff` |
| +0x0c | i32 | home club index (id) | `uStack_268 = *home_club` |
| +0x10 | i32 | away club index (id) | `uStack_264 = *away_club` |
| +0x14 | u32 | Comp* | FUN_00668890 `local_260 = param_1[1]` |
| +0x18 | u32 | Person* referee | swizzled at load |
| +0x1c | u32 | Club* home | `puStack_258` in ctor |
| +0x20 | u32 | Club* away | `puStack_254` |
| +0x24 | i16 | venue short (city?) | via FUN_00845380 |
| +0x26 | i16 | venue short | via FUN_00845380 |
| +0x28 | i16 | year | `sStack_24c = doy + comp+0x40` |
| +0x2a | i16 | day-of-year | `uStack_24a = schedule[round].doy` |
| +0x2c | i16 | season year (comp+0x40) | `local_248 = param_1[0x10]` |
| +0x2e | i16 | (0 / goals-home slot ?) | `local_244 = 0` |
| +0x30 | i16 | comp+0x3a (cup round/week) | `local_242` |
| +0x32 | i16 | round index | `sStack_240 = (short)round_idx` |
| +0x34 | i16 | comp+0xdb | `local_23e` |
| +0x36 | i16 | comp+0xab flag | `local_23c` |
| +0x38 | u8 | comp+0x31 low byte | `local_23a` |
| +0x39 | u8 | 0xff sentinel | `local_239 = 0xff` |
| +0x3a | u8 | comp_type (1=league, 9=cup, ...) | `local_237`, gated `< 0x17` |
| +0x3b | u8 | comp-flag byte | `local_236` |
| +0x3c | u8 | weekday code from schedule+0x04 | `uStack_235 = *(u8*)(sched+2)` |
| +0x3d | u8 | home has no manager (bool) | `bStack_234 = (home+0x38 == 0)` |
| +0x3e | u8 | away has no manager (bool) | `bStack_233 = (away+0x38 == 0)` |
| +0x3f | u8 | weekday (0..6) | `local_232 = (u8)(comp+0x11)` |
| +0x40..+0x4a | 11 × 0xff | official/referee sub-list sentinels | 11 `local_XXX = 0xff` |
| +0x4b..+0x4c | 2 bytes | unused / part of local_26c | overlaps with 0xffffffff init |
| +0x4d | u16 | flag word (bit 0x800 = second-leg) | bit-masks in FUN_00668890 |

**Round-to-fixture math for English Div 2**:
- 24 clubs → 46 rounds × 12 pairings = **552 fixtures per season**.
- Outer loop bound `(n_clubs - 1) * matches_per_pair = 23 * 2 = 46`.
- Inner loop iterates `n_clubs / 2 = 12` pair-positions.
- Each iteration calls FUN_00594d00 → inserted into the calendar.

## 4. Backward trace for one concrete first-round fixture — HYPOTHESIS (needs live capture)

The exe stores the abstract mechanism, not concrete team pairs. For
round 0 on Sat 11 Aug 2001 the eng_second constructor will:

1. schedule-getter returns 2990-byte buffer with round-0 record at
   `+0` and doy=222, year=2001, type=1 at `+0x00..+0x04`.
2. FUN_005601d0 populates comp+0xb1 with 24 English Div-2 clubs (in
   the order determined by the clubs' `nation=England` +
   `chosen_division=Second` filter — order confirmed by other
   sim-findings work).
3. FUN_00668890 initialises an adjacency matrix (n_clubs × n_clubs),
   iterates 46 rounds, and for each round walks 12 pair positions
   using FUN_0066f280 as the rotation helper. For each pair it reads
   home = comp+0xb1[i], away = comp+0xb1[j] (with H/A flipped for
   the second half).
4. For each pair it constructs a 79-byte fixture with:
   home_club_ptr = &comp+0xb1[i], away = &comp+0xb1[j],
   date = 2001, doy = 222, weekday = 5 (Sat, from schedule+0x04).
5. FUN_00594d00 files that fixture into TFixList[year=2001][doy=222][slot][comp_idx=9].

Backward trace terminates at: **club-order in comp+0xb1** (established by roster populator) and **schedule buffer date** (established by schedule-getter). Both are recovered from static + runtime.

**Concrete pair sequence for round 0 is NOT yet captured**. Extracting it requires either:
- Driving the exe through the full save-init menus with FUN_00668890 hooked (safe, but manual).
- A fresh direct-call approach that first sets up a legal comp pool (unsafe without more work).

## 5. Relevant subset of FUN_00668890 callers — VERIFIED EXACT

From agent 2 report (a3ef5f2b), each of 109 comp-type source files
contributes ~2 callsites: one in the comp constructor, one in a
~59-byte reschedule helper.

English divisions (initial + reschedule callsites):

| Div | Ctor CS | Ctor VA | Reschedule helper | .cpp file |
|-----|---------|---------|-------------------|-----------|
| Conference | 0x0055789a | 0x005577a0 | 0x00558ba7 | eng_conf.cpp |
| First | 0x0055b436 | 0x0055b340 | 0x0055cbd7 | eng_first.cpp |
| Premier | 0x0055d01b | 0x0055cf20 | 0x0055e89a | eng_prm.cpp |
| **Second** | **0x0055f136** | **0x0055f040** | **0x005604a7** | **eng_second.cpp** |
| Third | 0x00560c48 | 0x00560b40 | 0x00561fc7 | eng_third.cpp |

**The eng_second Div-2 pairings for a fresh save originate at
`0x0055f136` inside FUN_0055f040.** Confirmed by comp id write
(`*(u8*)(param_1+0x14) = 9` at line 25 of the decompile).

The other 192 callsites are per-competition constructors and their
reschedule helpers — every league and cup in the exe follows the
same "self-generate fixtures at construction time" pattern. No
central master driver exists.

## 6. Evidence-backed status of FUN_0066f280 semantics — VERIFIED STRUCTURE, SEMANTICS PARTIAL

Prior claim of "RNG-perturbed circular walker, 4-state cycle + Berger
override" was TOO STRONG. Tightening per user instruction:

**VERIFIED EXACT**:
- State field: a single `char*` (param_2) holding one byte, values
  observed 0, 1, 2, 3.
- State transitions: 1→2, 2→3, 3→0 unconditional; 0→1 with RNG gate.
- RNG helper called: FUN_008fc4f0 with argument 4 (mod-4 game RNG).
- Wrap rule: on state==3 branch: `new = old + 3; if new >= n: new -= n`.
- Base rotation (default fall-through): `new = old + 1; if new >= n: new -= n`.
- Two special-case flag bits on `param_7`:
  - `& 0x40` → return `param_1 + 1` unmodified (no wrap check).
  - `& 0x80` → swap-based rotation: `p1 = old + 1; if p1 == n/2 return n-1; if p1 == n return n/2; else return p1`.

**VERIFIED STRUCTURE, SEMANTICS PARTIAL**:
- The state 0 branch does `if (old < n-5 && FUN_008fc4f0(4) > 1) shift by 3 with state:=1; else shift by 1`.
  So the RNG only fires on state 0 with roughly p=0.5 and only when
  distance from end is safe. This is NOT well-explained yet in terms
  of round-robin semantics — the `< n-5` guard suggests preserving
  final rounds from perturbation.

**HYPOTHESIS**:
- The `& 0x80` branch's shape (`p1 == n/2 → n-1`, `p1 == n → n/2`)
  is structurally **similar** to a Berger table's half-swap for
  ensuring balanced pairings when a team's own slot rolls over. I no
  longer claim this IS Berger; the resemblance is enough to note but
  not enough to name.

**Not yet observed**:
- The `DAT_009bbba8` special case (comp param==this constant returns
  4 or 3) — never fires in eng_second construction; the const
  corresponds to a specific comp id used only in cup-family
  competitions.

## 7. Source/mechanism of team pairing — VERIFIED STRUCTURE, SEMANTICS PARTIAL

**Mechanism**: Generic round-robin algorithm with RNG-perturbed
rotation via FUN_0066f280 walker.

- The competition-specific input is: `n_clubs` (24 for Div 2),
  `matches_per_pair` (2 for double round-robin), and the pre-computed
  schedule buffer at `comp+0xba` (dates only).
- The round-robin is executed at competition construction time,
  producing 552 stack-local 79-byte fixture records which are
  inserted into the TFixList calendar via FUN_00594d00.
- The pairings are NOT from a hard-coded table, NOT from precomputed
  permutations, NOT from database source data. They are generated at
  runtime from a deterministic starting arrangement (roster order)
  and perturbed by the game RNG through the walker.

**Not from a table**: Confirmed by absence of any large data blob
of 24-team pairings in `.rdata` and by the presence of both
FUN_00668890's adjacency matrix (`new (n_clubs * n_clubs)`) and the
per-round walker rotation.

**RNG-dependent**: Yes — FUN_008fc4f0 is called from the walker.
This means for the same club order and same initial RNG state, the
fixtures are deterministic. This matches the project's determinism
principle (see [[frida-capture-replay]] memory).

## 8. Round-to-fixture mapping for several rounds — VERIFIED EXACT (structural)

Every round record in the schedule buffer produces
`n_clubs / 2 = 12` fixtures for eng_second. The fixture-record's
`+0x2a` (doy), `+0x28` (year), `+0x3c` (weekday) fields are copied
from the schedule buffer at that round; the fixture's `+0x32` field
holds the round index. Multiple fixtures share one round record.

Concrete mappings for the requested rounds (structural — the exact
club pairings await runtime capture):

| Round | Schedule doy | Date | Type | Fixtures produced |
|-------|--------------|------|------|-------------------|
| 0 | 222, yr=2001 | Sat 11 Aug 2001 | 1 (league) | 12 pair-fixtures with weekday=Sat, +0x2c=2001 |
| 1 | 229, yr=2001 | Sat 18 Aug 2001 | 1 | 12 pair-fixtures Sat |
| 3 | 238, yr=2001 | Mon 27 Aug 2001 | 2 (midweek) | 12 pair-fixtures Mon (Bank Holiday) |
| 45 | 124, yr=2002 | Sun 5 May 2002 | 1 | 12 pair-fixtures Sun (final day) |

Round-to-record: 1:1 (round index N → schedule buffer offset N*0x41).
Round-to-fixtures: 1:12 for eng_second (in general 1:n_clubs/2).
Fixture stored at: `TFixList[year][doy][slot][comp_idx].list[home]`
via FUN_00596590.

## 9. Controlled perturbation results — NOT YET RUN

Perturbation tests deferred pending resolution of the "safe direct-
call of a comp constructor" problem (my ctor call crashed the exe).
Perturbations planned once safe:
- Change comp+0x40 (year) from 2001 to 2002 and observe date table
  shift.
- Change comp+0x3e (n_clubs) from 24 to 20 and observe round count
  drop and pair-count drop.
- Change comp+0xf (matches_per_pair) from 2 to 1 and observe round
  count halved.
- Force FUN_008fc4f0 return to a fixed value and confirm walker
  becomes deterministic pure round-robin.

## 10. Remaining unknowns — explicit list

- The **exact adjacency-matrix seeding** in FUN_00668890 (lines
  FUN_00669780 / FUN_0066bd40). This is the pure round-robin base
  before walker perturbation is applied.
- Whether the reschedule-helper callsite (0x005604a7) reproduces
  identical fixtures on subsequent seasons or perturbs anew.
- The meaning of the `< n-5` guard on state-0 walker perturbation.
- The 8 sub-slots per round in the schedule buffer: for eng_second
  only slot 0 is populated (with sentinels). For cup competitions
  or leagues with multi-comp scheduling, slots 1..7 may hold
  cup-round overlap references — unconfirmed.
- Whether `FUN_00594d00` mutates or wraps the fixture record before
  filing.
- Concrete pair sequence for round 0 (needs safe live capture).

## 11. Rust changes and why they do not prematurely constrain the final architecture

`crates/cm-domain/src/exe_date.rs` gained:

- **`apply_flag_snap`** — byte-exact port of FUN_00533eb0. Isolated
  primitive; consumers include the write chain but nothing else. No
  premature architecture.
- **`write_slot`** — byte-exact port of FUN_0066f410. Same
  characterisation — a primitive that receives byte-safe args and
  writes deterministic bytes. Does not commit to any team-slot
  semantics; the args are named neutrally (`field_a/b/c`) rather
  than `home_id/away_id/status`.
- **`build_eng_second_schedule`** — deterministic constructor for the
  2990-byte buffer only. Does NOT construct fixtures or team pairs.
  Justified because the buffer is a proven-invariant computable
  entity, and reproducing it exactly is a prerequisite to porting
  the round-robin driver.
- **`full_buffer_matches_runtime_capture`** — now compares all 2990
  bytes byte-for-byte. Passes.

Not added:
- Fixture object type — waiting until FUN_00668890 is decoded byte-
  exact.
- Round-robin driver logic — same.
- TFixList container — same.
- Wiring of the buffer into `generate_double_round_robin` — same.
  The generator remains a **TEMPORARY STUB — KNOWN INCORRECT** per
  the accuracy contract.

The Rust modules added are pure primitives whose signatures name
only bytes and structural fields (`round_idx`, `sub_slot`,
`field_a`), never `home_team`, `away_team`, `fixture`. This
deliberately leaves the higher-level architecture unchosen so the
byte-exact port of FUN_00668890 can define its own types
consistently with the exe.

---

## Provenance summary (rule J)

| Claim | Source |
|-------|--------|
| FUN_0066f410 writes +0x05..+0x0b | Static decompile `decompiled/0066f410.c` lines 39-43 + agent 1 report + my disasm at `disasm_prewriter.py` output |
| eng_second_ctor calls FUN_00668890 at 0x0055f136 | Static decompile `decompiled/0055f040.c` line 50 + agent 2 report |
| Fixture record = 79 bytes | `FUN_00672320(f, 0x4f)`, `FUN_00934d76(rec, 0x4f, 1, file)` at FUN_005966e0 (agent 3 report) |
| Fixture container = TFixList 0x1184 bytes | `operator_new(0x1184)` at FUN_00594370 (agent 3) |
| Save file name = `fixtures.<year>.tmp` | FUN_00596a30 filename constant at TFixList+0x1133 (agent 3) |
| Full 2990-byte buffer byte-exact | `cargo test full_buffer_matches_runtime_capture` |
| Walker semantics | `decompiled/0066f280.c` full read |
| 3 static-caller counts (0066f3b0=3499, 0066f410=5584, 00668890=202) | `find_callers.py`, `find_f410_callers.py` |

Nothing above depends on football-domain knowledge; football
fixture lists were consulted only for date corroboration, never as
source of truth.
