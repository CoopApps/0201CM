# Fixture integration — Phase 4: model correction + confidence tightening

Date: 2026-09-13. **Authoritative spec**: `D:/cm0102/cm0102_GDI.exe`.

## Phase-3 defects the user surfaced

1. **Clubs-table structure error**. My Phase-3 tests used a padded
   Rust allocation (`n_clubs * 0x3b + 256`) to hide out-of-bounds
   reads at `+0x69`. That was masking a wrong data model.
2. **Overclaiming "byte-exact"**. `run_round_robin_driver` and
   `matrix_perturb` were labelled VERIFIED EXACT despite the driver
   producing Rust callback events (not comparable to bytes) and the
   perturbation being validated only against structural properties
   plus one partial Frida capture that crashed mid-Phase-E.
3. **Missing pair trace**. No captured cm0102-gdi pair sequence had
   been differentially verified.

## What actually landed this phase

### 1. Clubs-table structure correctly modelled

Confirmed via `fun00668890.c:256` disassembly:

```c
piVar6 = *(int **)(*(int *)(iVar12 + iVar10 + iVar10 * 0x3a) + 0x69);
```

The addressing pattern is:

```text
clubs_base = *(int *)(comp + 0xb1)         // pointer to entry array
entry_i    = clubs_base + i * 0x3b         // 59-byte entry
club_ptr   = *(int *)entry_i               // first int is Club*
nation_id  = *(int *)(club_ptr + 0x69)     // Club + 0x69 — POINTED-TO object
```

`+0x69` is inside a *pointed-to* Club object (0x245 bytes), NOT
inside the 0x3b entry. A raw byte slice cannot reproduce the
dereference.

### 2. New `ClubResolver` abstraction

- `ClubSlotMeta { nation_id: Option<i32> }` — pre-resolved
  pointer-indirect state per slot.
- `ClubResolver` trait with `nation_of`, `e2_pair_shares_69`,
  `e3_pair_cross_linked` methods.
- `Vec<ClubSlotMeta>` and `[ClubSlotMeta]` impls.
- `NullResolver` for tests that isolate the algorithm from any
  pointer-indirect state (E2/E3 no-op, no host-nation match).

### 3. `run_round_robin_driver` refactored

- New `&dyn ClubResolver` parameter.
- Host-nation constraint routes `resolver.nation_of(away_slot)`
  instead of reading `clubs_table[slot*0x3b + 0x69..]`.
- No padded allocations in any test.

### 4. `matrix_perturb` refactored

- New `&dyn ClubResolver` parameter.
- E2 uses `resolver.e2_pair_shares_69(i, j)`.
- E3 is now IMPLEMENTED (previously a no-op) — uses
  `resolver.e3_pair_cross_linked(i, j)`; default trait impl
  returns false, matching the safe caller (no cross-link data).
- E1 gate unchanged (owner_comp_first_int comparison).

### 5. Confidence labels tightened

Per user's terminology directive:

| GDI VA | Purpose | Status |
|--------|---------|--------|
| `0x00533d80` | pack_date | **BYTE-EXACT** — verified vs runtime capture |
| `0x005340e0` | flag-snap | **BYTE-EXACT** — 42 runtime cases |
| `0x0066ef70` | round writer | **BYTE-EXACT** — 2990 B buffer match |
| `0x0066efd0` | slot writer | **BYTE-EXACT** — 2990 B buffer match |
| `0x0055f540` | schedule getter (composite) | **BYTE-EXACT** — buffer SHA match |
| `0x0066ee40` | walker | **STRUCTURALLY PORTED** — state machine unit-tested, no live pair trace |
| `0x00669340` | matrix seeder | **BYTE-EXACT** — n∈{4,6,8,10,24} cell-for-cell match vs GDI capture |
| `0x0066b900` | perturbation | **STRUCTURALLY PORTED — DIFFERENTIAL PENDING** — partial pool RNG capture; full RNG-state differential blocked on runtime crash mid-Phase-E |
| `0x00668450` | round-robin driver | **STRUCTURALLY PORTED — DIFFERENTIAL PENDING** — control flow translated line-by-line; no captured GDI pair sequence yet |

### 6. Test relabelling

- `driver_deterministic_no_perturb_double_rr` → `driver_structural_invariant_double_rr_symmetry`. Explicit STRUCTURAL INVARIANT label — proves math validity, NOT GDI equivalence.
- `driver_host_nation_swap_fires_when_away_matches` → `driver_host_nation_swap_via_resolver`. Verifies swap via the abstraction, not via byte-slice hack.
- All 21 tests pass.

## What's still not done

1. **Live GDI pair trace**. The Phase-3 report claimed one was next-
   phase work. It still is. Options being pursued:
   - UI-driven new-game capture (requires user interaction).
   - Attach + wait for FUN_00668450 to fire naturally.
   - Fabricate a valid Club-pool state so synthetic direct-call works.
2. **RNG-state differential for the full driver run**. Requires the
   pair trace above.
3. **`comp+0xea` reachability for eng_second**. Not yet proven
   impossible. Static analysis of eng_second_ctor + the comp
   struct at boot time will show this.
4. **Wiring `run_round_robin_driver` into `generate_new_game_season`
   for comp 9**. Cannot land until (1) and (2) complete without
   silently substituting Rust structural correctness for GDI
   fidelity.

## Explicit non-claims

- The driver is NOT wired into production.
- The Berger add-mod stub is NOT retired.
- The pair sequence produced by the current driver is a Rust-side
  invariant only — it is NOT proven identical to cm0102-gdi's
  sequence.
- The perturbation E3 branch is IMPLEMENTED as a code path but has
  NEVER been exercised against real data — the default resolver
  returns false, so E3 is a no-op in every current test.

## Commits

- `fe2eae1` Phase 1 — dates + walker
- `95467ac` Phase 2 — matrix seeder
- `7b9528b` Phase 3 — perturbation + driver (STRUCTURAL PORTS with
  provisional labels)
- (this commit) Phase 4 — model correction (ClubResolver +
  double-indirection removed byte-slice hack), tightened
  terminology, relabelled tests as structural invariants

## Answer to the 13-item user report

1. **Corrected club/roster structure**: See §1 above.
2. **Audited driver offsets/strides**:
   - `0x3b` entry stride: VERIFIED from `fun00668890.c:256`
     `iVar10 + iVar10 * 0x3a` idiom = `iVar10 * 0x3b`.
   - `0x69` offset: VERIFIED as inside a POINTED-TO Club object,
     not the 0x3b entry.
   - `comp+0xb1`: VERIFIED as a POINTER field (see `iVar12 = *(int
     *)((int)param_1 + 0xb1)` at the same call site) — its value is
     the base of the entry array.
   - Schedule `0x41` stride: VERIFIED byte-exact via Phase-1 capture.
   - Fixture `0x4f` (79-byte): STRUCTURE VERIFIED via prior agent
     report; NOT ported into the driver — driver emits
     `FixtureEmission` callbacks instead.
3. **Successful full GDI perturb capture**: NOT ACHIEVED. Partial
   capture only (one pool RNG value = 6181 for `rand_mod(30000)`;
   Phase E crashed on invalid Club pointer). Documented as pending.
4. **Exact Rust-vs-GDI perturb differential**: PENDING (3).
5. **Exact RNG initial/final differential**: STRUCTURAL only.
6. **Actual GDI pairing trace**: NOT ACHIEVED.
7. **Rust-vs-GDI pair-order/H-A differential**: PENDING (6).
8. **comp+0xea semantics/reachability**: NOT INVESTIGATED. Deferred.
9. **Fixture creation/insertion state requirements**: The current
   `FixtureEmission` carries `(walker_col, home_slot, away_slot,
   round_within_half, outer_round, is_last_round,
   host_nation_swap)`. Prior work established that
   `HeadlessSeasonFixture` fields (competition_id, competition_name,
   date, home/away ids/names, status, scores, match_packet) are
   sufficient for downstream consumers.
10. **Revised confidence classifications**: See §5 above.
11. **Whether Second Division is safe to wire**: **NO**.
    Structural port is complete; behavioural verification against
    cm0102-gdi is not.
12. **Production integration and Berger removal**: NOT DONE.
13. **English pyramid family comparison**: NOT STARTED — user
    directive says do not begin until Second Division is exact.

## Tests

21 tests in `eng_second_fixtures`, all pass. 13 `exe_date` tests
unchanged, all pass. No production code activated behind the new
abstraction.
