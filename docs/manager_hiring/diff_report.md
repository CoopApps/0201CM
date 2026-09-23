# Manager Hiring AI — differential verification (cm0102.exe vs Rust)

Harness: `tools/exe_diff/diff_manager_scoring.py` over the Unicorn emulator
`tools/cm-lift/cm_lift/emulate.py` (PE loader / memory mapper / call-ABI). Rust
side = the standalone `cm-scoring` probe (`target/debug/scoring_probe`), which is
the single canonical implementation cm-domain re-exports. Ground truth = the real
`D:\cm0102\cm0102.exe` (DirectDraw build, VAs match ghidra_out).

Technique: `hook_call` force-return isolates a target from its sub-calls — e.g.
force `FUN_0052a410`'s return to drive `FUN_0052a330` through every closeness
code, and force `FUN_005274d0` ("ref known in nation") to exercise every branch
of the classifier.

## Trust anchors (both PASS)

| function | rust symbol | corpus | mismatches | result |
|---|---|---|---|---|
| FUN_0052a330 manager_club_repfit | `cm_scoring::manager_club_repfit` | 600 (5 closeness codes × 2 modes × 60 random blocks, incl. negative/boundary reps) | 0 | **exe == Rust** |
| FUN_0052a410 closeness_class | `cm_scoring::closeness_class` | 65 (ref-null + all 64 branch combinations) | 0 | **exe == Rust** |

## Mismatches found & fixed during bring-up

- **Harness bug (not the port):** `test_repfit` left a permanent `hook_call` on
  `FUN_0052a410` that forced its return; when `test_closeness` then called
  `0052a410` directly it returned the stale forced value (4) instead of running.
  Fix: clear `_call_hooks` at the start of each test. After the fix, closeness
  went 55→0 mismatches.
- Reconciled arg order by disassembly: in `FUN_0052a410`, **arg1 = person,
  arg2 = ref** (arg2 is the null-tested operand). The Rust tree already matched
  this; only the test fixture's arg wiring was confirmed.

## Verified logic (from disassembly, docs/manager_hiring/rating_constants.md §7)

- `FUN_0052a330`: pick a `short` from the person's 3-slot rep vector by closeness
  code; codes 3/1 average two slots (`x/2 + y/2` on shorts). Pure integer.
- `FUN_0052a410`: same club → 0; same club-nation → 0; same person-nation → 2;
  `!known_in_person_nation` → 2; `!known_in_person_club_nation` → 3; regional
  rep sum ≥ 0x36b0 (14000) → 3; else 4.

## Still pending differential verification (not yet claimed exact)

- `FUN_00682420` score core — the ~180-term per-jobtype weight table (base
  re-weight + skeleton done; weights recovered in §5-6, pairing to be transcribed
  in verified chunks through this harness).
- `FUN_0082dab0` attractiveness, `FUN_00679ed0` board-confidence (RNG order),
  `FUN_00681c70` poach decision, and the `FUN_00674c10` driver.
