# C15.1C — SquadRecord+0x3A materialisation

**Branch.** `gdi-renderer-port`.
**Predecessor.** `203d663` (C15.1B).
**Scope.** Recover the canonical Rust storage for `SquadRecord+0x3A`
and materialise the promotion-side reset into `World`.

## 1. Objective

The exe's promotion path (`FUN_004D3550`) resets a per-person
"squad-registration position code" byte at record offset `+0x3A`
for every occupant of the promoted club's 50 squad slots. This
tranche identifies where that byte lives in the Rust model,
extends the model minimally to accommodate it, and materialises
the promotion reset onto `World.contracts` inside the C15 apply
pipeline.

## 2. Exe archaeology

### 2.1 The write helper — `FUN_00843970`

`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/00843970.c`,
41 lines. Signature (Ghidra):

```
void FUN_00843970(int param_1, int *param_2, char param_3)
```

* `param_1` — staff / person pointer (used as key to look up the
  contract record).
* `param_2` — club pointer (identity gate anchor).
* `param_3` — new position code byte to write.

Control flow:

1. **Nil-guard** (line 11). If either `param_1 == 0` or
   `param_2 == 0`, return without doing anything.
2. **Range gate** (line 12). If `param_3 < -0x32 || 0x32 < param_3`
   pop an Error dialog and clear `DAT_00b4d5a8 = 0`, then return
   without writing. `-0x32..=0x32` is `-50..=50` strict inclusive.
3. **Primary resolver** (line 20). `iVar1 = FUN_004D59D0(param_1)`.
4. **Primary write & short-circuit** (lines 21-27). If
   `iVar1 != 0` and identity gate passes (see 2.2), write
   `*(iVar1 + 0x3A) = param_3` and **return**.
5. **Secondary resolver** (line 28). `iVar1 = FUN_004D5B00(param_1)`.
6. **Secondary write** (lines 29-37). If `iVar1 != 0` and identity
   gate passes, write and return; else return without writing.

The identity gate combines two checks (either satisfies):

* Direct: `*(record + 4) == *param_2` — the record's `club_id`
  equals the club record's `+0x00` field.
* Nation-affiliation: `DAT_00acd5bc + *(record + 4) * 0x245 ==
  FUN_0052a5a0(param_2, 0, 1)` — a nation-indexed base-address
  equivalence that matches persons owned by an affiliate club of
  the same national federation.

Runtime constants:

* `DAT_00accad8` — 0x50-byte contract-record pool. Per session
  memory `[[contract-clauses-generated-at-boot]]`, this is the
  same pool that carries the `+0x1C` / `+0x1F` clause bytes that
  C15.1B writes.
* `DAT_00acdf0c` — the person-index table used by the primary
  resolver (0x4F stride).
* `DAT_00acd56c` — the person-count upper bound.
* `DAT_00acd5bc` — nation-affiliation table base.

### 2.2 The promotion walk — `FUN_004D3550`

`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/004d3550.c`,
61 lines. Signature (Ghidra):

```
void FUN_004d3550(int *param_1, int *param_2, int param_3)
```

* `param_1` — the pool base ptr (used by inner code that scans
  `by_staff_id`; not directly consumed here since
  `FUN_00843970` resolves via globals).
* `param_2` — club pointer.
* `param_3` — promotion flag (nonzero → do the +0x3A reset walk).

Two loops:

**Loop A (lines 15-26) — SquadRecord+0x3A reset walk.** Runs only
when `param_3 != 0` (i.e. actual promotion, not the shared
relegation entry point). Iterates
`piVar3 = param_2 + 0xd7; iVar7 = 0x32` — the 50 pointer-slots
at offset `+0xd7` on the club record. Each non-null occupant is
passed to `FUN_00843970(occupant, club, 0)` — resetting position
code to 0.

**Loop B (lines 27-58) — Contract flag clear (2 pass).** The
already-ported C15.1B path. Calls `FUN_0052a5a0(club, &param_2, 1)`
to obtain a second club reference (affiliate / reserve), then
loops over the same 50 squad-slot array on each of two clubs
(pass 0 = own club, pass 1 = affiliate). For each occupant,
looks up its contract record via `DAT_00acdf0c` and writes
`+0x1F`/`+0x1C` if armed.

### 2.3 The relegation walk — `FUN_004D3460`

`FUN_004D3460` never calls `FUN_00843970` — the `+0x3A` byte is
touched **only** on promotion. Relegation is a `+0x1F: 1 → 2`
byte write on the contract record (C15.1B).

## 3. Rust storage — what already exists

Before C15.1C, `crates/cm-domain/src/contract_init.rs` had the
0x50-byte contract-record model on `ContractRecord` with named
fields for the clause bytes (`+0x1C`, `+0x1F`), staff id
(`+0x00`), wage / value / etc. **`+0x3A` had no field.**

The archaeology confirms `SquadRecord+0x3A` and `ContractRecord`
alias the same 0x50-byte pool record (both indexed via
`DAT_00accad8`, both accessed via `DAT_00acdf0c[person * 0x4F]`).
Session memory `[[contract-clauses-generated-at-boot]]` had
already tagged this identity for the clause bytes.

**Decision.** Extend `ContractRecord` in-place with a new
`position_code: i8` field. No new sidecar was required — the
archaeology proves the byte lives on the same record.

## 4. Model changes

`contract_init.rs`:

* `ContractRecord`:
  * Renamed `person_id → club_id` (retroactive correction:
    the field at `+0x04` was mislabelled — every path in
    `FUN_00843970` and `FUN_004D3550` treats it as
    `club_id`, and every save-load path that populated it fed
    the club ID). `#[serde(alias = "person_id")]` preserves
    older saves.
  * Added `position_code: i8` at logical `+0x3A`.
* `ContractPool`:
  * Added `by_staff_id_secondary: Vec<i32>` with
    `#[serde(default)]` — the secondary resolver's index. Older
    saves that don't carry it deserialise as `Vec::new()`, which
    makes `contract_for_staff_secondary_mut` always miss (safe
    fallback identical to the "no secondary index yet" runtime
    state).
  * Added `contract_for_staff_secondary_mut` accessor.
* `initialise_all` populates the new fields with zero /
  `Vec::new()` defaults.

`c15_1_world_apply.rs`:

* `SquadRecordSlot { Primary, Secondary }` enum tagging which
  resolver hit.
* `AppliedSquadPreferenceWrite { person_id, record_slot,
  old_value, new_value }` trace entry.
* `WorldApplyReport.applied_squad_preference_writes:
  Vec<AppliedSquadPreferenceWrite>` — mutations-only log.
* `apply_squad_position_writes_from_report` walks
  `AnnualRolloverReport` events, calls `write_squad_position`
  for every `Promotion { effects }.person_effects` entry with
  `new_value = 0` (matches `FUN_004D3550` Loop A passing 0 to
  `FUN_00843970`).
* `write_squad_position` — byte-exact port of `FUN_00843970`:
  range gate → primary resolve → identity gate → primary
  short-circuit → secondary fallback.
* Wired into `apply_report_to_world` immediately after the
  C15.1B contract-clause writes (both operate on
  `world.contracts` if `Some`).

C15.1B retroactive fix:

* `apply_contract_write_for_person` gained a `club_id: u32`
  parameter and enforces
  `record.club_id == club_id` — the same identity gate the exe
  applies in `FUN_004D3550` Loop B. Both call sites in
  `apply_contract_writes_from_report` now pass
  `effects.club_id`. This does not change C15.1B semantics for
  the existing test corpus (all 11 tests still green) because
  every prior helper built the contract at the promotion
  event's `club_id`.

## 5. Deviations from the exe (explicit, bounded)

1. **Nation-affiliation identity check not ported.** The exe's
   secondary identity clause,
   `DAT_00acd5bc + record.club_id * 0x245 == FUN_0052a5a0(club, 0, 1)`,
   is not reproduced. `write_squad_position` uses only the
   direct `record.club_id == club_id` check. Impact: a person
   whose contract is on an affiliate club record whose nation
   base matches the promoted club would be missed. Deferred
   until the affiliate/nation base is modelled — no existing
   Rust code carries `DAT_00acd5bc`. **Failure mode is
   silent-skip, not a crash.**
2. **Loop-A walk driven by `person_effects` rather than club
   +0xd7 slots.** The Rust event carries a `Vec<PersonEffect>`
   the C13 pipeline produced. C15.1B uses the same set; C15.1C
   reuses it for consistency. This tranche does NOT model the
   club's 50-slot pointer array — that model is not yet in
   place, and the person set the report carries is a strict
   superset of the 50-slot occupants for the promotion event.
   Impact: the applier can theoretically process a person
   whose contract is on a different club, but the identity
   gate filters them out (silent skip). Verified in
   `c15_1c_fifty_slot_walk_mixed_valid_missing_mismatch`.
3. **Range-gate side effect (Error dialog + `DAT_00b4d5a8 = 0`)
   not reproduced.** The exe pops a modal Error dialog on
   out-of-range and clears a global. Rust silent-skips. The
   directive explicitly allowed this: "no panic unless the
   exe hard-fails" — the exe does not hard-fail (no crash, no
   assert), it soft-fails and continues.
4. **Applier ordering.** The exe runs Loop A (position writes)
   before Loop B (clause writes) inside `FUN_004D3550`. The
   Rust wire-in runs the C15.1B path first (already frozen)
   and the C15.1C path after. The two touch disjoint offsets
   (`0x1C` / `0x1F` vs `0x3A`) so no byte written by C15.1B is
   invalidated by re-visiting the record in C15.1C, and vice
   versa. Documented in the wire-in comment.

## 6. Tests (16 total)

All in `crates/cm-domain/src/c15_1_world_apply.rs` under the
`c15_1c_*` prefix:

| Test | Coverage |
| ---- | -------- |
| `c15_1c_primary_only_golden` | Primary resolver + identity match + trace entry |
| `c15_1c_secondary_only_when_primary_null` | Primary null → secondary hit path |
| `c15_1c_both_records_primary_wins_short_circuit` | Primary short-circuit prevents secondary write |
| `c15_1c_identity_mismatch_primary_falls_through_to_secondary` | Primary present but mismatch → secondary |
| `c15_1c_identity_mismatch_both_is_silent_skip` | Both mismatch → no write, no trace |
| `c15_1c_range_pass_minus_50` | `-50` accepted |
| `c15_1c_range_pass_plus_50` | `+50` accepted |
| `c15_1c_range_skip_minus_51` | `-51` silent-skip |
| `c15_1c_range_skip_plus_51` | `+51` silent-skip |
| `c15_1c_idempotent_second_fire_no_duplicate_trace` | Re-application does not re-write or re-trace |
| `c15_1c_missing_pool_index_is_silent_skip` | Person not in `by_staff_id` → no write |
| `c15_1c_relegation_does_not_touch_position_code` | Only `Promotion` events consume this path |
| `c15_1c_fifty_slot_walk_mixed_valid_missing_mismatch` | 50-slot walk with 30 valid + 10 mismatch + 10 unresolvable; exactly 30 writes |
| `c15_1c_trace_old_equals_pre_write_new_equals_post_write` | `old_value == pre`, `new_value == post` |
| `c15_1c_range_boundary_writes_are_recorded_faithfully` | `-50` and `+50` recorded with correct sign |
| `c15_1c_trace_vs_world_consistency` | For every trace entry, `pool.records[resolver_index].position_code == trace.new_value` |

Result: **16 passed; 0 failed.** C15.1B's 11 tests remain
**11 passed; 0 failed** after the retroactive identity-gate
addition.

## 7. Frozen boundaries

* C15.1B contract-clause writes are **frozen**. The identity
  gate added in this tranche is a strict tightening (no
  previously-written byte is now un-written, no
  previously-skipped byte is now written for the test corpus).
* No history events, news events, or stadium counter writes
  were touched.
* No Frida differential has been run — this port stands on the
  Ghidra decompile plus session-memory identity of the pool.
  The tranche does NOT claim STATE-EXACT parity with the exe;
  it claims that the ported control-flow and byte-write
  semantics match the decompile.

## 8. Follow-ups (out of scope for this tranche)

* Nation-affiliation identity check (deviation 1 above) once
  `DAT_00acd5bc` is modelled.
* 50-slot club squad pointer array (deviation 2). Deferred
  until other subsystems need it.
* Frida runtime differential of a promotion year-end vs the
  materialised `applied_squad_preference_writes` trace.
* Persistent-history capture of `+0x3A` transitions. The
  overarching directive keeps persistent history off until the
  bounded byte-write tranches are all closed. **This is one of
  them.** After C15.1D (stadium refuse counter), persistent
  history opens.
