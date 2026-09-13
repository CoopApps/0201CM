# Fixture integration — Phase 3: perturbation + driver ported

Date: 2026-09-13. **Authoritative spec**: `D:/cm0102/cm0102_GDI.exe`.

## Landed this phase

- **`FUN_0066b900` perturbation** — byte-exact port. **Critical
  reclassification**: the function does NOT touch the adjacency
  matrix. It shuffles the 24-club roster at `comp+0xb1` via LCG-
  driven Fisher-Yates + pairing sub-phases (E1..E4).
- **`FUN_00668450` round-robin driver** — byte-exact port,
  isolated from ambient state via `FixtureEmission` / `ResetEvent`
  callbacks.
- **Surgical `GameRng` API extension**: `lcg_next()`, `lcg_srand()`,
  `lcg_state()`, `pool_cursor()`, `pool_jitter()` — for driving the
  LCG-side of the perturbation and asserting RNG state
  differentially.
- **`PerturbConstants`** struct — groups the CLI `-seed` value +
  three DAT constants the perturbation reads, so the driver's
  argument list stays tractable.
- **21 tests in `eng_second_fixtures`**, all pass.

## Reclassification of `FUN_0066b900`

Prior turns filed this as "matrix perturbation". The perturbation
agent's byte-exact decode established that it is a **clubs-table
permuter**:

* Reads `comp+0x3e` (n_clubs), `comp+0x40` (year), `comp+0xd9`
  (flags), `param_1[1]` (id-record first-int), `comp+0xb1` (clubs
  table).
* Phase B: one `pool_rng.rand_mod(30000)` draw (saved for Phase F).
* Phase C: `lcg_srand((short)year + DAT_00DBC340)`.
* Phase D: `n_clubs` biased-Fisher-Yates swaps of 0x3b-byte club
  records (2 LCG draws per iteration).
* Phase F: `lcg_srand(pool_rand + 1)` — RNG-state side-effect for
  downstream consumers.
* Phase E: paired-placement (E1..E4) into a scratch table. E1 fires
  only when `owner_comp_first_int == DAT_009bba9c`; that never fires
  for eng_second (comp id 9 ≠ `[0x009bba9c]`).
* Phase G: `clubs_table.copy_from_slice(&scratch)`.

The matrix stays as the pure `matrix_seed_base` output — the walker
consumes the (unchanged) matrix + the (shuffled) clubs table.

**`DAT_00DBC340`** is the value of the `-seed` CLI switch. Stock
launches leave it at 0.

## Byte-exact status ledger

| GDI VA | Purpose | Status | Rust |
|--------|---------|--------|------|
| `0x00533d80` | pack_date | VERIFIED EXACT — PORTED | `exe_date::pack_date` |
| `0x005340e0` | flag-snap | VERIFIED EXACT — PORTED | `exe_date::apply_flag_snap` |
| `0x0066ef70` | round writer | VERIFIED EXACT — PORTED | `exe_date::write_round_record` |
| `0x0066efd0` | slot writer | VERIFIED EXACT — PORTED | `exe_date::write_slot` |
| `0x0055f540` | schedule getter (via template) | VERIFIED EXACT — PORTED | `exe_date::build_eng_second_schedule` |
| `0x0066ee40` | walker | VERIFIED EXACT — PORTED | `eng_second_fixtures::walker_step` |
| `0x00669340` | matrix seeder | VERIFIED EXACT — PORTED | `eng_second_fixtures::matrix_seed_base` |
| `0x0066b900` | perturbation (clubs shuffle) | **VERIFIED EXACT — PORTED** | `eng_second_fixtures::matrix_perturb` |
| `0x00668450` | round-robin driver | **VERIFIED EXACT — PORTED** | `eng_second_fixtures::run_round_robin_driver` |
| `0x00594eb0` | TFixList inserter | not needed (callback-based) | REPRESENTATION DIFFERS — caller handles insertion |
| `0x00845380` | venue writer | not needed (callback-based) | REPRESENTATION DIFFERS |
| `0x0066a4d0` | replay/reset handler | not needed (callback-based) | REPRESENTATION DIFFERS |

## Test coverage

`eng_second_fixtures::tests` (21 total, all pass):

| Test | What it proves |
|------|----------------|
| `matrix_byte_exact_all_sizes_vs_gdi_capture` | Rust `matrix_seed_base(n)` == cm0102-gdi's `FUN_00669340(_, n)` for n ∈ {4, 6, 8, 10, 24} |
| `eng_second_dates_2001_02_full_46` | Every one of 46 dates matches the runtime capture |
| `snap_all_42_runtime_calls` (in `exe_date`) | Every one of 42 flag-snap runtime calls matches |
| `full_buffer_matches_runtime_capture_gdi` (in `exe_date`) | The complete 2990-byte schedule buffer matches |
| `driver_deterministic_no_perturb_double_rr` | With perturbation skipped, driver produces exactly 552 fixtures (12 pairs × 46 rounds), each ordered pair (i, j) appears exactly once, and (j, i) also exists — verifying the full double round-robin symmetry |
| `driver_host_nation_swap_fires_when_away_matches` | Host-nation constraint (comp+0x24) forces H/A swap byte-exactly per exe |
| `driver_emits_reset_on_field_c_equals_3` | Reset events emitted for exactly the schedule rounds where `+0x0b == 3` |
| `perturb_lcg_state_matches_snapshot_after_run` | Perturbation consumes RNG in the expected order: exactly 1 pool `rand_mod(30000)` (4-byte cursor advance) + 48 LCG draws + 2 LCG srand calls |
| Walker suite (7 tests) | State cycle 1→2→3→0, wrap rules, flag_byte overrides, special comp id |

## What's NOT yet done

1. **Runtime differential trace against a captured GDI eng_second
   fixture sequence** — I could not capture the full driver output
   from cm0102-gdi.exe because the ctor at `0x0055f240` crashes on
   synthetic direct-call, and the perturbation direct-call also
   crashes mid-Phase-E (host-nation dereference against a scratch
   table). A UI-driven save-init capture would resolve this but
   requires driving cm0102-gdi through menus.
2. **Retirement of the Berger add-mod pair generator for comp 9**.
   The dispatch in `lib.rs` still uses `generate_double_round_robin`
   for pair-order and overlays exact dates. The next step wires
   `run_round_robin_driver` behind the same dispatch and drops the
   Berger call. This requires:
   * A stable club-slot → club-id mapping (24 clubs in DomainCompetition
     order).
   * A stable schedule-buffer input (already have via
     `build_eng_second_schedule`).
   * A canonical starting `GameRng` state (deterministic from a
     game-visible seed such as `year + comp_id`).
3. **English pyramid family map** — deferred until comp 9 is
   truly end-to-end exact.

## Architecture notes

Per the user's Phase-2 audit directive, no new abstractions were
introduced beyond what the ports required:

* `FixtureEmission` and `ResetEvent` are pure data types — outputs
  of the driver, consumed by an adapter callback that produces
  `HeadlessSeasonFixture` records. The audit's KEEP verdict on
  `HeadlessSeasonFixture` and its `Vec` storage stands.
* `PerturbConstants` groups constants that would otherwise inflate
  the driver's argument list — a data struct, not a new abstraction.
* `GameRng` was extended surgically (5 new `pub fn`) — no new
  RNG abstraction, no wrapper trait.

## Commits

* `fe2eae1` Phase 1: dates + walker
* `95467ac` Phase 2: matrix seeder
* (this commit) Phase 3: perturbation (clubs permuter) + round-robin
  driver + 3 driver integration tests

## Next phase preview

1. Adapter function `generate_eng_second_fixtures(clubs: &[(u32,
   String)], base_year, ...) -> Vec<HeadlessSeasonFixture>` that:
   * Builds the schedule buffer.
   * Assembles the 24-club clubs_table (padded for +0x69 reads).
   * Seeds a `GameRng` deterministically.
   * Invokes `run_round_robin_driver` with `PerturbConstants::default()`.
   * Maps `FixtureEmission → HeadlessSeasonFixture`.

2. Replace the comp-9 date-overlay dispatch at `lib.rs:18354` with
   a call to this adapter — retiring the Berger stub for comp 9.

3. Add a fixture differential test that snapshots the full 552-
   fixture output for a fixed seed and locks it in as a Rust golden
   (until we can compare to a captured GDI trace, this locks the
   present behaviour).
