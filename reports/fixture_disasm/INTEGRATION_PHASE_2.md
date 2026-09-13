# Fixture integration — Phase 2: byte-exact matrix seeder

Date: 2026-09-13. **Authoritative spec**: `D:/cm0102/cm0102_GDI.exe`.

## What landed

- **`FUN_00669340` matrix base seeder** — byte-exact port with
  differential test against a live Frida capture from
  `cm0102_GDI.exe`. n_even = 4, 6, 8, 10, and **24 (English Div 2)**
  all match byte-for-byte.
- **17 tests** in `eng_second_fixtures` module now, all pass.
- **`matrix_perturb` and driver stubs** marked as `unimplemented!()`
  / obvious-no-op so accidental production use fails immediately.

## Byte-exact status ledger

| GDI VA | Purpose | Status | Rust |
|--------|---------|--------|------|
| `0x00533d80` | pack_date | **VERIFIED EXACT — PORTED** | `exe_date::pack_date` |
| `0x005340e0` | flag-snap | **VERIFIED EXACT — PORTED** | `exe_date::apply_flag_snap` |
| `0x0066ef70` | round writer | **VERIFIED EXACT — PORTED** | `exe_date::write_round_record` |
| `0x0066efd0` | slot writer | **VERIFIED EXACT — PORTED** | `exe_date::write_slot` |
| `0x0055f540` | schedule getter (via template + build) | **VERIFIED EXACT — PORTED** | `exe_date::build_eng_second_schedule` |
| `0x0066ee40` | walker | **VERIFIED EXACT — PORTED** | `eng_second_fixtures::walker_step` |
| `0x00669340` | matrix seeder | **VERIFIED EXACT — PORTED** | `eng_second_fixtures::matrix_seed_base` |
| `0x0066b900` | matrix perturb | STRUCTURE VERIFIED — NOT YET PORTED | `eng_second_fixtures::matrix_perturb` (panics) |
| `0x00668450` | round-robin driver | STRUCTURE VERIFIED — NOT YET PORTED | `round_robin_driver_stub_returns_empty` |

## Differential runtime evidence

`runtime/20260913_143506_gdi_matrix_seed.json` — direct-call capture
from cm0102_GDI.exe of `FUN_00669340` for n_even ∈ {4, 6, 8, 10, 24}.
Rust `matrix_seed_base(n)` returns identical values for every
`(row, col)` cell. Compared exhaustively in
`matrix_byte_exact_all_sizes_vs_gdi_capture`.

**n_even = 4** (transcribed for readability):
```
[1]: [0,  2, -3,  4]
[2]: [0, -1,  4,  3]
[3]: [0, -4,  1, -2]
[4]: [0,  3, -2, -1]
```

Every non-zero cell obeys the pair-symmetry invariant: `matrix[i][c]
= ±j` ⇒ `matrix[|j|][c] = ±i`. Verified by
`matrix_pair_symmetry_n4` / `_n6`.

## Cross-verification with prior work

- Byte-exact schedule buffer (Phase 1) + byte-exact matrix seeder
  (Phase 2) means the **two independent pillars of the fixture
  pipeline are now proven**: dates come from the schedule buffer,
  pair connectivity comes from the adjacency matrix.
- The remaining unknowns are (a) the LCG-driven perturbation shuffle
  in `FUN_0066b900`, (b) the H/A pick + fixture record construction
  in `FUN_00668450`. Both consume state from the matrix + the walker
  — both are already byte-exact.

## Explicit non-implementations

Two functions have byte-exact target VAs but no ported body. To make
this loud:

- `matrix_perturb` — body is `unimplemented!(...)` so any accidental
  call panics with a source-annotated message.
- `round_robin_driver_stub_returns_empty` — the identifier itself
  flags the stub. Return type is `Vec<(i32, i32, i32)>` which does
  not match any expected fixture signature so no downstream code can
  silently accept its output.

Neither is invoked from any production path.

## Test counts

- `exe_date` module: 13 tests (unchanged, all pass)
- `eng_second_fixtures` module: 17 tests (up from 9 in Phase 1)
- Overall fixture-pipeline tests: 30, all pass

Pre-existing unrelated failures in `match_engine_exe`, `scouting`,
`tests::rust_db_save_executes_due_fixture_batches_*` are unchanged
(git-stash baseline confirmed).

## Provenance

- `reports/fixture_disasm/gdi_matrix_seed_capture.py` — Frida
  direct-call driver.
- `runtime/20260913_143506_gdi_matrix_seed.json` — captured matrices
  for n_even ∈ {4, 6, 8, 10, 24}.
- `D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/00669780.c` —
  DirectDraw decompile used as porting reference (byte-identical to
  GDI `0x00669340` modulo relocations per
  `GDI_CORRECTION_REPORT.md`).

## Not yet closed

1. **Matrix perturbation** (`FUN_0066b900`, 0x9b1 bytes) — RNG-heavy
   Fisher-Yates + additional mutation logic. Blocks driver.
2. **Round-robin driver** (`FUN_00668450`, 0x920 bytes) — consumes
   the seeder + perturbation + walker output to emit fixtures.
   Blocks retiring the Berger stub for comp 9.
3. **Full fixture differential** — waiting on driver port.
4. **English pyramid family map** — deferred to next phase.
5. **Schedule extraction automation** — deferred.
6. **GDI constructor runtime capture** (`0x0055f240`) — remains
   crashing on synthetic direct-call. Investigation deferred.

## Commits

* `fe2eae1` — Phase 1: eng_second dates + walker
* (this commit) — Phase 2: byte-exact matrix seeder + differential
  capture + classifier audit

## Rust-side production activation

Still: **comp id 9 date-overlay only**. The Berger add-mod pair
generator remains active for all competitions including comp 9 for
pair-order. That will only change when the perturbation + driver
ports land and produce zero-difference fixtures.

## Answers to the phase-2 checklist

1. **Audit/classification of `eng_second_fixtures.rs`** — done in
   the module header + this ledger.
2. **Completed GDI matrix seeder port** — **YES**, byte-exact vs
   Frida capture.
3. **Completed GDI matrix perturb port** — no, deferred (see §Not yet closed).
4. **GDI 0x00668450 driver implementation status** — stub only.
5. **Exact integration with existing Rust fixture types** — no schema
   change; date-overlay path documented in `RUST_ARCHITECTURE_AUDIT.md`.
6. **Removal of the old pairing stub from comp ID 9** — deferred to
   Phase 3, dependent on driver.
7. **Full GDI-vs-Rust English Second fixture differential** —
   deferred to Phase 3.
8. **RNG state differential** — deferred to Phase 3 (perturb + driver
   are the RNG consumers).
9. **Remaining English Second deviations** — pair order + H/A only.
10. **Cross-comparison of Premier/First/Second/Third/Conference** —
    deferred.
11. **Proposed generator-family architecture** — deferred pending
    evidence.
12. **Generic vs competition-specific values** — table in
    `INTEGRATION_PHASE_1.md` still current.
13. **Schedule extraction automation status** — deferred.
14. **Recommended next competition to activate** — none until comp 9
    is fully exact (Phase 3).
15. **Commits** — see §Commits.
