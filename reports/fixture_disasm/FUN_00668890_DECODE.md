# FUN_00668890 byte-exact reconstruction

Date: 2026-09-13. Source: `D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/00668890.c` (425 lines).

All line numbers refer to that file. Stack offsets given both as
Ghidra's `local_XXX` (offset from EBP) and byte offset within the
79-byte fixture record (base = `local_274`).

---

## Function signature

```
undefined4 __fastcall FUN_00668890(int *param_1)   // param_1 = Comp* in ECX
```
- Calling convention: `__fastcall` — sole arg in ECX (line 2).
- Frame: ~0x290 bytes (lowest local `local_28c`, alignment).
- Return: **0** on early failure (lines 96/112), **1** on success (line 420).
- SEH frame: `puStack_8 = &LAB_0094b791`, `local_c = ExceptionList` (lines 75-76), unwound at 422.

## Comp field reads (init phase)

| Byte off from param_1 | Type | Line(s) | Meaning |
|-----------------------|------|---------|---------|
| +0x00 (`*param_1`)    | int vtable | 148 (`*(*param_1 + 0x7c)`) | vtable; slot 0x7c/4 = 31 called with fixture ptr — fills header bytes |
| +0x04 (`param_1[1]`)  | int* | 120, 124, 126 | comp id-record ptr; `*param_1[1]` = comp id at +0x00 |
| +0x3c (`u16*)(param_1+0xf)` | u16 | 160, 306, 308 | `matches_per_pair` (2 for eng_second) |
| +0x24 (`param_1[9]`)  | i32 | 247, 257, 285, 365 | host-nation constraint id (-1 = none) |
| +0x3a                 | u16 | 125 | → fixture +0x32 |
| +0x3e                 | i16 | 78, 82, 149, 178, 244, 304, 315, 318, 378 | `n_clubs` |
| +0x40 (`param_1[0x10]`) | i16 | 127, 152, 159, 193, 206, 226, 234, 288, 367, 411 | season year base |
| +0x44 (`param_1[0x11]` u8) | u8 | 132, 193, 206, 226, 234 | weekday/parity flag → fixture +0x42 |
| +0xa9                 | i16 | 149, 170, 291, 315, 370, 385, 405, 418 | `n_rounds` (46) |
| +0xab                 | u16 | 128 | → fixture +0x38 |
| +0xb1                 | Club[]* stride 0x3b (59) | 246, 258, 266, 334, 347, 366 | clubs table |
| +0xba                 | u8* stride 0x41 (65) | 150, 285, 287, 366, 408, 410 | schedule buffer |
| +0xc4                 | u8 | 130 | → fixture +0x3a |
| +0xd9                 | u8 | 171 | walker flag byte (`param_7` of FUN_0066f280) |
| +0xdb                 | u16 | 131 | → fixture +0x36 |
| +0xdd                 | u32 (write) | 88 | cleared to 0 at entry |
| +0xea                 | PairList* | 158, 327, 387, 392 | optional pre-baked pair list (alt path) |

Note: `param_1[N]` = Nth 4-byte word → byte offset `N*4`.

## Adjacency matrix

- **Spine** (line 89): `puVar1 = operator_new((n_even + 1) * 4)` where `n_even = n_clubs + (n_clubs & 1)`. Fail → return 0 (lines 90-97).
- **Rows** (99-117): for `i = 1..n_even`, `puVar1[i] = operator_new(n_even * 4)`. Fail → free spine and return 0 (lines 104-113).
- Layout: **1-indexed (n_even × n_even) int matrix**. `puVar1[0]` unused/guard.

### Seeding

- Line 119: `FUN_00669780(puVar1, n_even)` — **base pure round-robin seeder**.
- Lines 120-123: `if (param_1[1] == NULL || (*(int*)param_1[1] != DAT_009bbc5c && *(int*)param_1[1] != DAT_009bbc64))` → `FUN_0066bd40(n_even)` — **secondary perturbation**, skipped for two specific comp ids.

## Two execution paths

**MAIN PATH** (lines 160-311): fires when `comp+0xea == NULL || year mismatches` — generates fixtures from adjacency matrix.

**ALT PATH** (lines 314-392): fires when `comp+0xea != NULL && season_year matches pair-list year` — replays a pre-baked pair list (used for reschedules).

## MAIN PATH — outer loop

```
uVar13 = *(u16*)(comp+0x3c)                        # matches_per_pair
iVar14 = n_even - 1
iVar16 = matches_per_pair * (n_even - 1)           # total rounds (46 for eng_second)
iStack_280 = 1                                     # 1-based round idx
do {
    ... walker + inner loop ...
    iStack_280++
} while (iStack_280 <= iVar16)
```

### Walker call (per outer iteration, once)

```
local_288 = FUN_0066f280(
    local_288,                                     # prior walker column (init -1)
    &local_289,                                    # walker byte state (init 0)
    *(u32*)param_1[1],                             # comp id
    (i16)comp[+0x3e],                              # n_clubs
    matches_per_pair,                              # from uVar13
    (i16)comp[+0xa9],                              # n_rounds
    (u8)comp[+0xd9]);                              # walker flag byte
```

Column derivation:
```
iVar4 = iStack_280 % (n_even - 1)
if (iVar4 == 0) iVar4 = n_even - 1
sStack_240 = (i16)local_288                        # fixture +0x34 (round-within-half idx)
```

## MAIN PATH — inner loop

Iterate `iVar14 = 0..n_clubs-2` (23 iters for eng_second), row-1 = "row index":
```
piStack_284 = puVar1 + 1                           # start at matrix row 1
iVar14 = 0
do {
    row_ptr = *piStack_284
    cell    = *(int*)(row_ptr + iVar4 * 4)         # matrix[row][col]
    if (cell != 0) {
        # ... H/A pick + club resolve + venue + commit
    }
    piStack_284++
    iVar14++
} while (iVar14 + 1 < n_clubs)
```

### H/A flip (4-way switch, lines 185-241)

Based on `sign(cell)`, `second_half = ((iStack_280 - 1) / (n_even - 1)) % 2 == 1`, and `season_parity = ((year + weekday_flag) % 2 == 0)`.

Four canonical cases (first-half; second-half inverts season_parity):
- `cell > 0 && !season_parity` (LAB_00668e60): `iVar5 = puVar1[cell]; iVar16 = cell - 1; iVar10 = iVar14`
- `cell > 0 && season_parity`  (LAB_00668ee3): `iVar5 = puVar1[cell]; iVar16 = iVar14; iVar10 = cell - 1`
- `cell < 0 && season_parity` : same as case 2 with `cell = -cell`
- `cell < 0 && !season_parity`: `iVar16 = -cell - 1; iVar5 = puVar1[-cell]; iVar10 = iVar14`

Line 242: `*(u32*)(iVar5 + iVar4*4) = 0` — mark the mirror matrix slot consumed.

### Club-pointer resolution (lines 243-283)

Bounds: `0 <= iVar16 <= n_clubs-1 && 0 <= iVar10 <= n_clubs-1` (243-245).

```
clubs = comp+0xb1
if (comp+0x24 == -1) {                             # no host-nation constraint
    home = clubs + iVar16 * 0x3b
    away = clubs + iVar10 * 0x3b
} else {                                           # host-nation constrained
    away_candidate = clubs + iVar10 * 0x3b
    away_nation = *(int**)(*(int*)away_candidate + 0x69)
    if (away_nation != NULL && *away_nation == comp+0x24) {
        # away matches host → swap (host must play at home)
        home = away_candidate
        away = clubs + iVar16 * 0x3b
    } else {
        # keep original H/A
        home = clubs + iVar16 * 0x3b
        away = away_candidate
    }
}

fixture[+0x1c] = *(u32*)home       # Club* home
fixture[+0x20] = *(u32*)away       # Club* away
fixture[+0x40] = *(u8*)(home+0x38) == 0   # home has no manager
fixture[+0x41] = *(u8*)(away+0x38) == 0   # away has no manager
fixture[+0x0c] = home ? *(u32*)home_club : 0xffffffff   # home id
fixture[+0x10] = away ? *(u32*)away_club : 0xffffffff   # away id
```

### Venue writer (line 285)

```
FUN_00845380(
    *(u32*)(comp[+0xba] + local_288 * 0x41 + 0x3d),   # schedule[walker].prize
    local_274,                                         # fixture*
    comp+0x24);                                        # host_nation
```
Writes fixture `+0x24`/`+0x26` (venue shorts).

### Date fields (lines 287-290) — **indexed by walker return, not outer counter**

```
sched_rec = comp[+0xba] + local_288 * 0x41
fixture[+0x28] = *(i16*)(sched_rec + 2) + (i16)(comp+0x40)   # year
fixture[+0x2a] = *(u16*)(sched_rec + 0)                       # doy
fixture[+0x3f] = *(u8*)(sched_rec + 4)                        # weekday code
```

**IMPORTANT**: The schedule record consumed per fixture is
`schedule[local_288]` (walker column), NOT `schedule[iStack_280]`
(outer round counter). This is the specific insight that makes the
round-robin work: the walker traverses the 46 schedule rounds in
RNG-perturbed order.

### Last-round flag (lines 291-296)

```
if (sStack_240 == n_rounds - 1) {
    fixture[+0x4d] = (fixture[+0x4d] & 0xfcff) | 0x0800    # bit 0x0800
} else {
    fixture[+0x4d] = fixture[+0x4d] & 0xf4ff               # clear bits 0x0800, 0x0300
}
```

### Commit (line 297)

```
FUN_00594d00(local_274 /* fixture */, 1 /* mode */);
```
Return value stored in `iVar16` but not consumed. Files fixture into
TFixList[year][doy][slot][comp_idx].

## ALT PATH (lines 314-392) — reschedule from pair list

Fires when `comp+0xea != NULL && *(u16*)(comp[+0xea]+2) == comp+0x40`:

```
for round = 0..n_rounds-1:
    for pos = 0..n_clubs/2 - 1:
        pair = comp[+0xea] + ((n_clubs/2) * round + pos) * 0x10
        home_id = *(int*)(pair + 8)
        away_id = *(int*)(pair + 0xc)
        # linear scan comp+0xb1 for matching id → home_slot, away_slot
        # populate fixture (same as MAIN path from date/venue onward)
        # extra: field_c == 4 ALSO triggers last-round bit
        if (sStack_240 == n_rounds-1 || *(u8*)(sched+0x0b) == 4) {
            fixture[+0x4d] |= 0x0800
        }
        FUN_00594d00(fixture, 1)
# free pair list
FUN_0093534b(comp[+0xea], 0x10, *(u32*)(comp[+0xea]-4), FUN_00401bc0)
FUN_00933d24(comp[+0xea] - 4)
comp[+0xea] = 0
```

Pair record stride 0x10 (16 bytes), fields at +0x08 home_id, +0x0c away_id.

## Cleanup (394-403)

```
for i = 1..n_even:
    FUN_00933d24(puVar1[i])                        # free row
FUN_00933d24(puVar1)                               # free spine
```

## Reset/replay pass (404-419)

```
for round = 0..n_rounds-1:
    sched_rec = comp[+0xba] + round * 0x41
    if (*(u8*)(sched_rec + 0x0b) == 3) {
        FUN_00533ad0(*(u16*)sched_rec,
                     *(i16*)(sched_rec + 2) + comp+0x40)
        FUN_0066a910(aiStack_224)
    }
```

**FUN_0066a910 fires only when `schedule[round]+0x0b == 3`** —
confirms field_c domain [-1..4] where value 3 means "reset/replay this round".

## Return

- Line 96: `return 0` on spine alloc failure.
- Line 112: `return 0` on row alloc failure.
- Line 420: `uVar2 = 1` at end of success path.
- Line 422-423: `ExceptionList = local_c; return uVar2;`

Success ALWAYS runs through cleanup and reset pass; no early success return.

## RNG usage

- **FUN_00668890 itself**: zero direct FUN_008fc4f0 calls.
- **Via walker (line 167, once per outer round)**: `FUN_008fc4f0(4)`
  fires in FUN_0066f280 line 57 — only when walker `state == 0 &&
  local_288 < n_rounds - 5`. Determines forward-1 vs forward-3 with
  state:=1.

## Fixture record layout — CORRECTED

**Correction to TEAM_PAIRING_REPORT §3**: fields from +0x30 onward
were 2 bytes off. Correct mapping (base = `local_274`):

| Off | Size | Var | Source |
|-----|------|-----|--------|
| +0x00 | u32 | `local_274[0]` | `*param_1[1]` (line 126) — first int of comp id-record |
| +0x04 | u32 | `local_274[1]` | written by vtable-31 (line 148) |
| +0x08 | u32 | `local_26c` | 0xffffffff sentinel (line 144) |
| +0x0c | u32 | `uStack_268` | home id |
| +0x10 | u32 | `uStack_264` | away id |
| +0x14 | u32 | `local_260` | Comp id-record ptr = `param_1[1]` |
| +0x18 | u32 | — | untouched (vtable-31 fills or leaves 0) |
| +0x1c | u32 | `puStack_258` | home Club* |
| +0x20 | u32 | `puStack_254` | away Club* |
| +0x24 | u16 | `uStack_250` | venue short (per-fixture: FUN_00845380) |
| +0x26 | u16 | `uStack_24e` | venue short (per-fixture: FUN_00845380) |
| +0x28 | i16 | `sStack_24c` | year = `sched[walker]+0x02 + comp+0x40` |
| +0x2a | u16 | `uStack_24a` | doy = `sched[walker]+0x00` |
| +0x2c | u16 | `local_248` | season base year = `(u16)comp+0x40` |
| +0x2e | u16 | — | untouched |
| +0x30 | u16 | `local_244` | 0 |
| +0x32 | u16 | `local_242` | `*(u16*)(comp+0x3a)` |
| +0x34 | i16 | `sStack_240` | walker return |
| +0x36 | u16 | `local_23e` | `*(u16*)(comp+0xdb)` |
| +0x38 | u16 | `local_23c` | `*(u16*)(comp+0xab)` |
| +0x3a | u8  | `local_23a` | `(u8)comp+0xc4` |
| +0x3b | u8  | `local_239` | 0xff |
| +0x3c | u8  | — | untouched (vtable-31) |
| +0x3d | u8  | `local_237` | 0 |
| +0x3e | u8  | `local_236` | 0 |
| +0x3f | u8  | `uStack_235` | `*(u8*)(sched[walker]+0x04)` |
| +0x40 | u8  | `bStack_234` | `home_club[+0x38] == 0` (home has no manager) |
| +0x41 | u8  | `bStack_233` | `away_club[+0x38] == 0` (away has no manager) |
| +0x42 | u8  | `local_232` | `(u8)comp+0x44` |
| +0x43..+0x4c | 10×u8 | `local_231..local_228` | 0xff sentinels |
| +0x4d..+0x4e | u16 | `local_227` | flag word, bit 0x0800 = last round |

## Notable constants

| Symbol | Meaning |
|--------|---------|
| `DAT_009bbba8` | special comp id where walker returns 4 (from 2) or 3 (from 5) |
| `DAT_009bbc5c`, `DAT_009bbc64` | comp ids that skip FUN_0066bd40 matrix perturbation |
| Error line refs `0x3fc` (1020), `0x405` (1029) | `comp_league.cpp` diagnostic lines |
| `0x3b` (59) | Club record stride |
| `0x41` (65) | schedule record stride |
| `0x10` (16) | alt-path pair record stride |
| vtable slot `+0x7c` = slot 31 | virtual "fill fixture header" hook |

## Round/fixture count sanity check (eng_second)

- `n_clubs = 24` → `n_even = 24`
- Outer: `matches_per_pair * (n_even - 1) = 2 * 23 = 46` rounds ✓
- Inner: 23 iterations per round; base seeder writes `n_clubs/2 = 12`
  non-zero entries per column → **12 commits per round × 46 = 552 fixtures** ✓

## Rust translation plan

The function decomposes cleanly:

1. `alloc_matrix(n_even) -> Vec<Vec<i32>>` (spine of `n_even+1`, rows 1..=n_even).
2. `seed_matrix_base` = port of `FUN_00669780` (base pure round-robin).
3. `seed_matrix_perturb` = port of `FUN_0066bd40` (guarded by comp-id check).
4. `walker_step` = port of `FUN_0066f280` (mostly done; behavior verified).
5. `pick_ha(cell, second_half, season_parity) -> (row, col, mirror_row)` — 4-way switch.
6. `resolve_clubs(host_nation, clubs, i, j) -> (home*, away*, bool, bool)`.
7. `write_venue` = port of `FUN_00845380(u32 aux, &mut Fixture, i32 host_nation)`.
8. `commit` = port of `FUN_00594d00(&Fixture, 1)` — TFixList inserter.
9. `replay_reset` = port of `FUN_0066a910` (fires only on `field_c == 3`).
10. Fixture struct: exact byte layout above; `+0x18`, `+0x2e`, `+0x3c` are holes filled by vtable-31.

**Blockers remaining before byte-exact port**:
- FUN_00669780 (matrix base seeder) — small, likely a Cayley-table-style pure round-robin generator.
- FUN_0066bd40 (matrix perturbation) — unknown size.
- FUN_00845380 (venue writer, +0x24/+0x26 fixture fields) — unknown size.
- vtable-slot-31 (+0x7c) on the comp vtable — writes fixture +0x04
  (and possibly +0x18, +0x3c). For eng_second this is one specific
  virtual function to decode.
- FUN_00594d00 (TFixList inserter) — architecture known (agent 3
  report); needs byte-exact decode.
- FUN_0066a910 (replay/reset handler) — narrow; fires only on
  field_c == 3.
