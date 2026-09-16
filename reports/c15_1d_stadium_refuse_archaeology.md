# C15.1D — Stadium+0x20 owner-refuse counter materialisation

**Branch.** `gdi-renderer-port`.
**Predecessor.** C15.1C (`df5995c`).
**Scope.** Add runtime storage for the parent-stadium
`+0x20` owner-refuse counter, and materialise the year-end
increment through the C15 apply pipeline.

## 1. Objective

`FUN_00583FC0` (the exe's stadium-expansion transaction) writes
`parent_stadium.+0x20 += 1` on the owner-refuse branch when a
broke club's owner-parent-club subsidy is insufficient to cover
the expansion cost. The Rust C14 port already computes this
intent as `StadiumExpansionOutcome.refuse_counter_increment:
bool`, but no consumer applied it. This tranche threads it
through to `World.references.stadiums`.

## 2. Exe archaeology

### 2.1 The write site — `FUN_00583FC0`

`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/00583fc0.c`
lines 121-128 (owner-backed refuse branch):

```c
121  else {
122    if ((((int)param_1[1] <= iVar8) && (((int)param_1[1] < iVar8 || (*param_1 <= uVar12)))) &&
123       (cVar1 = *(char *)(iVar6 + 0x20), cVar1 * 500000 <= (int)uVar12)) {
124      if (cVar1 < '\x14') {
125        *(char *)(iVar6 + 0x20) = cVar1 + '\x01';
126      }
127      return 0;
128    }
```

* `iVar6` at line 125 is the **parent stadium ptr**, reached via
  `Club[+0xBF]` (owner-parent-club) → `+0x69` (that club's
  stadium). `+0x20` on the runtime stadium object is the
  owner-refuse counter.
* Predicate: `cVar1 * 500000 <= cost` (parent's counter times
  £500 000 does not cover the cost).
* Saturation gate at line 124: only increment while `cVar1 < 20`.
* `return 0` — the transaction refuses regardless.

### 2.2 Callers

Grep for `FUN_00583fc0` reveals three sites:

* `00668380.c:36` — promotion install, `param_5 = 0` (forced
  path; refuse branch unreachable).
* `0055ee90.c:34` — general expansion request.
* `0055ea00.c:49` — Conference champion-promotion stadium check,
  `param_5 = 1`. This is the year-end refuse-counter caller for
  the Conference-fallback path.

In the Rust port only the forced path (`00668380`) currently
runs C14; the affordability-checked Conference path
(`c15_english_annual_rollover.rs::ChampionFallback::StadiumFailed`)
skips C14 today and directly emits `StadiumFailReprieve`. That
gap is **noted, not closed** by this tranche.

## 3. Rust storage — additions

### 3.1 `DomainStadium.owner_refuse_counter: i8`

`crates/cm-domain/src/lib.rs`:

```rust
pub struct DomainStadium {
    // …
    #[serde(default)]
    pub owner_refuse_counter: i8,
}
```

* Runtime-only. Not part of the shipped 78-byte `stadium.dat`
  layout. `#[serde(default)]` keeps older `.sav` files loading.

### 3.2 `ClubYearEndState.parent_stadium_id: Option<u32>`

`crates/cm-domain/src/c15_english_annual_rollover.rs` — the C15
input state now carries the parent stadium id alongside the
existing `parent_stadium_refuse_counter`. Fixtures that don't
set it fall back to `None` (silent-skip in the applier).

### 3.3 `YearEndMutationEvent::StadiumExpansion.parent_stadium_id`

New optional field on the mutation event. Emitted by
`apply_annual_rollover` from `state.parent_stadium_id`;
consumed by `apply_stadium_expansion_outcome` in
`c15_1_world_apply.rs`.

### 3.4 `AppliedRefuseCounterWrite` + `WorldApplyReport.applied_refuse_counter_writes`

`crates/cm-domain/src/c15_1_world_apply.rs`:

```rust
pub struct AppliedRefuseCounterWrite {
    pub stadium_id: u32,
    pub old_value: i8,
    pub new_value: i8, // == old + 1
}
```

`WorldApplyReport.applied_refuse_counter_writes:
Vec<AppliedRefuseCounterWrite>` — mutation log (an entry is
pushed only when the byte actually changed).

## 4. Applier logic

Inside `apply_stadium_expansion_outcome`, before the
capacity-write branch:

```rust
if outcome.refuse_counter_increment {
    if let Some(psid) = parent_stadium_id {
        if let Some(parent_stadium) =
            find_stadium_mut(&mut world.references.stadiums, psid)
        {
            let old = parent_stadium.owner_refuse_counter;
            if old < 20 {
                parent_stadium.owner_refuse_counter = old + 1;
                out.applied_refuse_counter_writes.push(
                    AppliedRefuseCounterWrite {
                        stadium_id: psid,
                        old_value: old,
                        new_value: old + 1,
                    },
                );
            }
        }
    }
}
```

Silent skip on any of:

* `refuse_counter_increment == false`
* `parent_stadium_id == None`
* Parent stadium not present in `world.references.stadiums`
* Counter already `== 20` (matches exe line 124)

The bump fires INDEPENDENT of `outcome.success`: the exe write
is inside the refusal branch, so in practice
`refuse_counter_increment == true` implies `success == false`
(the C14 port only sets the flag on that path), but the
applier's logic is not gated on `success` — matching the exe's
control flow, which does `return 0` after the write.

## 5. Deviations from the exe

1. **Conference-fallback path doesn't run C14 in Rust.**
   `FUN_0055EA00` calls `FUN_00583fc0(..., 1)` and may hit the
   refuse branch on the exe. In Rust, `ConferenceRolloverDispatch::
   ChampionFallback::StadiumFailed` is pre-decided upstream and
   emits `StadiumFailReprieve` without an intervening
   `StadiumExpansion` event. Impact: on that path, no
   `refuse_counter_increment` reaches the applier. This is an
   **existing gap** documented in
   `reports/c15_1_status.md`, not introduced by this tranche.
2. **`parent_stadium_id` seeding is per-fixture.** No engine
   code today reads `Club[+0xBF]` to populate
   `ClubYearEndState.parent_stadium_id` — it defaults to
   `None`. Fixtures that need refuse-counter behaviour must
   seed both `parent_stadium_refuse_counter` and
   `parent_stadium_id` themselves. The applier silent-skips
   when either is missing, matching the exe's null-check
   behaviour when either pointer link is absent.
3. **Range/error path not reproduced.** `FUN_00583FC0`
   has null-club and null-stadium guards (return paths
   `NullClubRecord`, `NullStadium`) — those never reach the
   refuse-counter site by construction. The Rust
   `AppliedRefuseCounterWrite` never fires for them because
   `refuse_counter_increment` is `false` on those C14 branches.

## 6. Tests (10 total)

All in `crates/cm-domain/src/c15_1_world_apply.rs`, prefix
`c15_1d_*`:

| Test | Coverage |
| ---- | -------- |
| `c15_1d_refuse_bump_lands_on_parent_stadium` | Golden: 3 → 4 on parent 501, primary 500 untouched, trace entry |
| `c15_1d_saturates_at_20` | Counter at 20 → no write, no trace |
| `c15_1d_no_write_when_flag_false` | `refuse_counter_increment == false` → skip |
| `c15_1d_no_write_when_parent_stadium_id_none` | Missing parent id → skip |
| `c15_1d_no_write_when_parent_stadium_missing_in_world` | Unknown parent id → skip |
| `c15_1d_multiple_refuse_events_bump_additively` | Two events → +2 on same parent, ordered trace |
| `c15_1d_bump_fires_when_success_false` | Failure branch still writes counter |
| `c15_1d_saturation_streak_stops_writes` | 5 events from counter=18 → trace len 2, stops at 20 |
| `c15_1d_trace_isolates_by_stadium_id` | Two parents (501, 502) → correct per-id bumps |
| `c15_1d_trace_new_equals_pool_state` | Trace `new_value` matches post-state on world |

Result: **10 passed; 0 failed.** Adjacent C15.1A/B/C tests all
green (68/68 in the C15.1 + stadium-expansion filter).

## 7. Frozen boundaries

* C15.1A/B/C not touched — only the `StadiumExpansion` event
  pattern gained a field and its consumer signature grew a
  parameter. All prior tranche invariants preserved.
* No history events, no news events touched.
* Persistent history remains out of scope. Per the C15.1C
  report's "Follow-ups" section, C15.1D is the last of the
  bounded byte-write tranches — persistent history opens
  **after** this landing.

## 8. Follow-ups (out of scope)

* Wire `Club[+0xBF]` → parent club → parent stadium id into
  the ClubYearEndState seed so live promotion runs populate
  `parent_stadium_id` without per-fixture setup.
* Close the Conference-fallback C14 gap (item §5.1) so
  `FUN_0055EA00`-equivalent Rust paths actually run C14 and
  emit refuse-counter events when appropriate.
* Frida differential — capture a real promotion year-end that
  refuses on the owner path and compare
  `applied_refuse_counter_writes` byte-for-byte.
