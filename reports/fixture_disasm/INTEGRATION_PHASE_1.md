# Fixture integration — Phase 1: exact dates + walker port

Date: 2026-09-13. **Authoritative spec**: `D:/cm0102/cm0102_GDI.exe`.

## Summary

Retired the flat `+7 days` stub for English Second Division (comp id 9)
only. Rust now emits the exact 46 GDI-derived matchdays for that
competition. Also ported the generic walker `FUN_0066ee40` byte-exact
with 8 unit tests. No competition other than English Second Division is
affected. All 22 new/existing fixture-pipeline tests pass.

## Audit-based decisions

Following the architecture audit at
`reports/fixture_disasm/RUST_ARCHITECTURE_AUDIT.md`:

- **Kept**: `exe_date.rs` (pack_date + apply_flag_snap + write_round_record + write_slot + template + build_eng_second_schedule) — already byte-exact.
- **Kept**: `HeadlessSeasonFixture` — every consumer (screens, save,
  standings, match pipeline) uses it. Overlaying the date on the
  Berger-produced fixtures avoids a schema change.
- **Kept**: `generate_double_round_robin` (Berger add-mod) as the
  pair-order producer for now. Retiring it depends on runtime
  capture of the exe's pair sequence + porting the round-robin
  driver (0x00668450) + matrix seeder (0x00669340) byte-exact.
- **Deleted**: `crates/cm-domain/src/cm0102_gdi/` orphan module (never
  wired; duplicated `exe_date.rs`).
- **Added**: `crates/cm-domain/src/eng_second_fixtures.rs`. Home for
  walker port, matrix stubs, `generate_eng_second_dates`, and the
  eng-second constants.

## Files changed

| File | Kind | Purpose |
|------|------|---------|
| `crates/cm-domain/src/eng_second_fixtures.rs` | added | walker port + `generate_eng_second_dates` + tests |
| `crates/cm-domain/src/lib.rs` | modified | `pub mod` declaration + comp-9 date-overlay dispatch |
| `crates/cm-domain/src/cm0102_gdi/*` | deleted | orphan skeleton |

## What's now VERIFIED EXACT — PORTED

| Component | GDI VA | Rust | Tests |
|-----------|--------|------|-------|
| `pack_date` | `0x00533d80` | `exe_date::pack_date` | 4 |
| `apply_flag_snap` | `0x005340e0` | `exe_date::apply_flag_snap` | 6 + 42-case sweep |
| Round writer | `0x0066ef70` | `exe_date::write_round_record` | 2 |
| Slot writer | `0x0066efd0` | `exe_date::write_slot` | 1 |
| 2990-byte schedule buffer | (composite) | `exe_date::build_eng_second_schedule` | 2 byte-exact vs GDI + DD captures |
| 46 exact eng_second dates | (composite) | `eng_second_fixtures::generate_eng_second_dates` | 2 (landmark + all-46 differential) |
| Walker | `0x0066ee40` | `eng_second_fixtures::walker_step` | 7 (pure, flag40, guard, state cycle, state1/3 wrap, special comp id) |

## What's STRUCTURE VERIFIED — SEMANTICS PARTIAL

| Component | GDI VA | Rust | Status |
|-----------|--------|------|--------|
| Matrix base seeder | `0x00669340` | `eng_second_fixtures::matrix_seed_base` | stub (empty vec) |
| Matrix perturbation | `0x0066b900` | `eng_second_fixtures::matrix_perturb` | stub |
| Round-robin driver | `0x00668450` | (Berger add-mod remains) | dispatch keeps existing `generate_double_round_robin` for pair generation until byte-exact port lands |

## What's REPRESENTATION DIFFERS, BEHAVIOUR VERIFIED

| GDI | Rust | Evidence |
|-----|------|----------|
| TFixList (0x1184 bytes/year container) | `Vec<HeadlessSeasonFixture>` on `HeadlessSeasonState.fixtures` | Every downstream consumer (`club_fixtures_for`, `league_table_for`, sidebar) queries by iteration. Insertion order matches Berger sequence; per-year segmentation is implicit in fixture dates. |

## Differential harness

`eng_second_fixtures::tests::eng_second_dates_2001_02_full_46` compares
all 46 Rust-generated dates against the canonical GDI runtime capture
(see `reports/fixture_disasm/runtime/20260913_131323_gdi_buffer_0.bin`).
Passes with **0 date differences**.

### First measured GDI-vs-Rust fixture comparison

| Stage | Metric | Result |
|-------|--------|--------|
| Schedule template (2990 B) | byte-diff vs GDI capture | **0** |
| 46 Round-record dates | date-diff vs GDI capture | **0** |
| Walker (state machine only, no RNG) | test-case behaviour | matches |
| Team pairing sequence | not yet compared | **pending** — needs runtime pair capture from `0x00668450` |
| Home/away assignment | not yet compared | **pending** — same |

## Not yet closed

1. **Team-pair order**: Rust still uses Berger add-mod, not the GDI
   driver's adjacency-matrix + walker sequence. Cannot produce
   zero-difference fixtures until (a) matrix seeder is ported byte-
   exact and (b) a runtime pair capture from `FUN_00668450` is
   available for differential testing.
2. **Walker + RNG**: `walker_step` accepts `Option<&mut GameRng>` but
   nothing wires it into `generate_new_game_season` yet; that plumbing
   is deferred until the driver is ready to consume it.
3. **GDI constructor runtime capture**: `0x0055f240` still crashes
   the exe on synthetic direct-call (side-effect at `0x00667090`).
   Investigation continues in parallel; not blocking date integration.

## Explicit non-goals for this phase

- Not porting Premier / First / Third / Conference. Comp id 9 only.
- Not extending `HeadlessSeasonFixture`. The schema stays.
- Not restructuring `exe_date.rs`. Byte-exact primitives stay where
  the audit found them.
- Not creating a `CompetitionTemplate` registry yet. That comes when
  the second English template lands.

## Generalisation ledger

Following the user's guidance on identifying reusable pieces vs
per-competition data:

| Value | Source in the exe | Classification |
|-------|-------------------|----------------|
| Round count (46) | schedule-getter body writes `0x2e` to `comp+0xa9` | **template data** (per competition) |
| Club count (24) | roster populator writes 24 slots | **template data** |
| `matches_per_pair` (2) | `comp+0x3c` | **competition field** |
| 46-tuple `(day, month, day_off, flag, type)` | schedule-getter body | **template data** |
| Weekday-snap flag semantics | `apply_flag_snap` | **generic invariant** |
| Round-record layout (65 B, 8 sub-slots × 7 B) | writers `0x0066ef70`, `0x0066efd0` | **generic invariant** |
| Walker state machine | `FUN_0066ee40` | **generic invariant** |
| Walker special-case comp id | `[0x009bbaf0]` global | **global constant** (passed as parameter) |
| Matrix-perturb skip comp ids | `[0x009bbba4]`, `[0x009bbbac]` | **global constants** (passed as parameters) |
| Adjacency matrix dimension | `n_clubs + (n_clubs & 1)` | **derived from template n_clubs** |
| H/A assignment rule | driver-body 4-way switch | **generic invariant** |

**Competition capability matrix** (initial, to be filled as leagues
are recovered):

| Competition | Template family | Pairing engine | Special behaviour |
|-------------|------------------|-----------------|-------------------|
| English Premier (id 7) | ? | round-robin driver + walker (shared) | midweek entries |
| English First (id 8) | ? | round-robin driver + walker (shared) | midweek entries |
| **English Second (id 9)** | **46-tuple recovered** | Berger stub → round-robin driver + walker (in progress) | Boxing Day + New Year hardcodes (flag = -1) |
| English Third (id 10) | ? | round-robin driver + walker (shared) | midweek entries |
| English Conference (id 93) | ? | round-robin driver + walker (shared) | fewer clubs? |

## Next steps

1. Port `FUN_00669340` (matrix base seeder) byte-exact — small (399 B).
2. Extract a schedule template from the other four English division
   schedule-getters via the same extract technique used for
   `ENG_SECOND_2001_TEMPLATE`. Fold them into a shared
   `EnglishScheduleTemplate` registry keyed by comp id.
3. Runtime-capture the pair sequence from `FUN_00668450` on a live
   save. Add a reference trace to
   `reports/fixture_disasm/runtime/`.
4. Wire `game_rng::GameRng` into the driver call so the walker's
   `rand_mod(4)` path fires deterministically.
5. Replace `generate_double_round_robin` for comp 9 with the walker-
   driven driver once (1)–(4) are done. Compare fixture-by-fixture
   against the runtime trace; require zero differences.

## Provenance

Every byte-exact claim above is traceable to:
- Static disassembly (`reports/fixture_disasm/gdi_disasm_probe.py`)
- Runtime buffer capture (`runtime/20260913_131323_gdi_buffer_0.bin`,
  SHA256 `682a5ea6…`)
- `reports/fixture_disasm/gdi_va_map.json` (address map)
- Prior FUN_00668890 reconstruction (structure) +
  `GDI_CORRECTION_REPORT.md` (GDI-specific overlay)
