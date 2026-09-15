# C15.1B — staff-record byte-state materialisation

Prior: `64af7bc` (C15.1A finance ledger). This closes C15.1 §26.2
into real byte writes on the canonical Rust storage.

## Executive summary

`+0x1C` and `+0x1F` live on the exe's **0x50-byte contract /
staff-employment record pool** (`DAT_00accad8`), indexed via the
`DAT_00acdf0c` person→staff-index table (0x4F stride, first int
= staff index, `-1` = no contract). This pool is already ported
as `contract_init::ContractPool` in Rust with typed fields on
`ContractRecord`:

- `+0x1C non_promotion: u8` — non-promotion release clause
- `+0x1F relegation: u8` — relegation release clause

The C13 walks' "staff+0x1C: 1 → 0" and "staff+0x1F: 1 → 0 / 1 → 2"
map cleanly to "clause armed / disarmed / tripped" on
`ContractRecord`. **No new Rust sidecar was introduced** —
canonical storage already exists.

## 22-point report

### 1. Exact staff structure owning `+0x1F`

**`ContractRecord`** (Rust) = the exe's 0x50-byte staff-employment
record pool at `DAT_00accad8`. Not type10 (70-byte, attributes),
not the club pool (0x245-byte). Resolved via
`DAT_00acdf0c[person_id * 0x4F]` (first int is the staff index;
`-1` sentinel = no contract).

### 2. `+0x1F` semantic

**Relegation clause state.** The exe treats it as a
3-state tag: `0` = clause absent / resolved, `1` = clause armed,
`2` = clause tripped. Cross-referenced in 4+ code paths:

- `004d3460.c:33-35` — relegation walk: if `+0x1F == 1` → write `2`; fire history `FUN_008D0D90(slot, old_comp, 3, staff)`
- `004d3550.c:45-47` — promotion walk clears `+0x1F: 1 → 0`
- `008d0d90.c:44-45` — history helper stamps siblings on the same record

### 3. Values 0/1/2

- `0` — idle / no pending clause. Terminal after promotion clears it.
- `1` — armed / active. Predicate guard for both walks.
- `2` — tripped — written only by relegation walk, paired with history event.

### 4. Exact staff structure owning `+0x1C`

**Same record.** Same 0x50-byte contract-record pool, same
resolver chain. Verified in `004d3550.c:48-50` (promotion clears)
and `004d3300.c:41,47-48` (a separate parallel walker writes
`1 → 2`).

### 5. `+0x1C` semantic

**Non-promotion clause state.** Parallel-but-independent 3-state
tag with a different event class than `+0x1F` (`kind=0` at
`004d3300` vs relegation's `kind=3` at `004d3460`). Same
0/1/2 pattern. Only touched by the promotion walk (`1 → 0`) and
by the `004d3300` walker (`1 → 2`); relegation walk never touches
it.

### 6. Values observed for `+0x1C`

- `0` — idle (terminal after promotion clear).
- `1` — armed.
- `2` — tripped (only written by the `004d3300` walker, not by
  this tranche's two walks).

### 7. `Club+0xD7 → staff record` resolution chain

Verbatim from `004d3550.c:38-44`:

```
piVar1 = (int*)*piVar5;                              // slot @ Club+0xD7+i*4 = pointer to Person record
if piVar1 == NULL                          → skip
iVar4 = *piVar1;                                     // person_id = person_record[0]
if iVar4 >= DAT_00acd56c                   → skip   // person-id upper bound
iVar4 = *(int*)(iVar4 * 0x4f + DAT_00acdf0c);        // staff index
if iVar4 < 0                               → skip   // -1 = no staff record
iVar4 = iVar4 * 0x50 + *param_1;                     // staff record base (0x50 stride)
if iVar4 == 0                              → skip
if *(int*)(iVar4+4) != *piVar6             → skip   // identity check: staff.club_id == current pass's club
```

- Slot contents: **raw pointer to a person record** (not id, not index).
- Empty-slot sentinel: **NULL pointer**.
- On any check failure: silently continue.

### 8. Promotion walk pseudocode (`FUN_004D3550`)

```
if (club == NULL) return

// Loop A: squad register (main club only) — writes staff+0x3A
// via FUN_00843970. That's C15.1C territory, not this tranche.
if (register_flag != 0) {
    for i in 0..50 { FUN_00843970(slot, club, 0) }
}

reserve = FUN_0052A5A0(club, &out_flag, 1)
if (reserve == NULL || out_flag != 0) {   // gate — port omits
    for pass in 0..2 {
        target = (pass==0) ? club : reserve
        for i in 0..50 {
            resolve slot → staff (chain from §7)
            if staff resolved:
                if staff+0x1F == 1: staff+0x1F = 0   // ORDER: +0x1F first
                if staff+0x1C == 1: staff+0x1C = 0   // then +0x1C
        }
    }
}
```

- Self-first, reserve-second.
- Both writes are ordered `+0x1F` → `+0x1C` and are INDEPENDENT
  (both may fire on one record; no combined predicate).
- **No history event** in either loop of `FUN_004D3550`.

### 9. Relegation walk pseudocode (`FUN_004D3460`)

```
if (club == NULL) return
reserve = FUN_0052A5A0(club, &out_flag, 1)
if (reserve != NULL and out_flag == 0) return

for pass in 0..2 {
    target = (pass==0) ? club : reserve
    for i in 0..50 {
        resolve slot → staff
        if staff+4 == *target AND staff+0x1F == 1:  // combined predicate
            staff+0x1F = 2                          // WRITE FIRST
            FUN_008D0D90(slot, old_comp, 3, staff)  // HISTORY AFTER
    }
}
```

- **Order: write `+0x1F = 2` FIRST, then fire history event.**
- **Never touches `+0x1C`.**
- History `kind` argument literal is **`3`**.

### 10. Duplicate-person behaviour

**Naturally idempotent.** The exe's `staff+4 == pass_target_club`
identity check gates all writes. A staff record has exactly one
`+4` (club id), so at most one pass's identity check succeeds.
Once the winning pass writes `+0x1F: 1 → 0` (promotion) or
`1 → 2` (relegation), the second visit finds `+0x1F != 1` and
skips. No dedup set required.

### 11. Rust canonical storage mapping

**`world.contracts: Option<ContractPool>`** (existing) →
`contracts.records[idx].{non_promotion, relegation}: u8`.

Look-up path: `contracts.by_staff_id[person_id as usize]` → `i32`
index into `records` (`-1` = no contract). New method
`contract_for_staff_mut` provides mutable access.

**No new sidecar** — canonical storage already exists.

### 12. Accessors / mutators added

- `ContractPool::contract_for_staff_mut(staff_id: u32) -> Option<&mut ContractRecord>` — mutable form of the existing `contract_for_staff`.
- `apply_contract_writes_from_report(&mut ContractPool, &AnnualRolloverReport, &mut WorldApplyReport)` — walks the report's Promotion and Relegation events, resolves each `PersonEffect` to a contract record, applies the byte writes matching the exe's predicates.
- `AppliedContractWrite { person_id, offset, old_value, new_value, kind }` — the trace-derived record emitted only after a real mutation.

### 13. Promotion `+0x1F` golden — `c15_1b_promotion_disarms_relegation_clause_when_armed`

Pre: `relegation = 1`, `non_promotion = 0`. Fire promotion.
Post: `relegation = 0`, `non_promotion = 0`. Trace: 1 write,
offset `0x1F`, kind `RelegationClauseDisarmed`.

### 14. Promotion `+0x1C` golden — `c15_1b_promotion_disarms_non_promotion_clause_when_armed`

Pre: `relegation = 0`, `non_promotion = 1`. Fire promotion.
Post: both `0`. Trace: 1 write, offset `0x1C`,
`NonPromotionClauseDisarmed`.

### 15. Combined-byte golden — `c15_1b_promotion_disarms_both_bytes_when_both_armed`

Pre: both `1`. Post: both `0`. Trace: 2 writes, **`+0x1F` first,
`+0x1C` second** — matches exe order (004d3550.c L45-47 then
L48-50).

### 16. Relegation golden — `c15_1b_relegation_trips_relegation_clause_when_armed`

Pre: `relegation = 1`, `non_promotion = 1`. Fire relegation.
Post: `relegation = 2` (tripped), **`non_promotion = 1`
untouched** (relegation walk never touches `+0x1C`). Trace:
1 write, kind `RelegationClauseTripped`, old=1, new=2.

### 17. Self/reserve golden — `c15_1b_duplicate_person_second_visit_is_naturally_noop`

Same person effect fired twice. First fire: `1 → 0`. Second
fire: current byte is `0`, predicate fails, no write. Ledger
unchanged after second fire. Matches the exe's natural dedup.

### 18. Invalid-record behaviour

Three test scenarios:

- `c15_1b_missing_contract_is_silent_skip` — person id 3000
  has no `by_staff_id` entry (out of bounds). Skip, no write.
- `c15_1b_negative_by_staff_id_sentinel_is_silent_skip` — sentinel
  `-1` in `by_staff_id`. Skip, no write.
- `c15_1b_no_write_when_*` (three tests) — pre-values `0`, `2`
  where predicate `== 1` fails. Skip, no write.

All match exe's silent-continue semantics. No panic.

### 19. Trace-vs-World consistency —
`c15_1b_trace_old_equals_pre_write_new_equals_post_write`

Reads the pool BEFORE apply; captures each byte. Applies. Reads
AFTER. Asserts `trace.old_value == pre_write` AND
`trace.new_value == post_write` for both offsets in trace order.

### 20. Tests — 11 total, all green

1. `c15_1b_promotion_disarms_relegation_clause_when_armed`
2. `c15_1b_promotion_disarms_non_promotion_clause_when_armed`
3. `c15_1b_promotion_disarms_both_bytes_when_both_armed`
4. `c15_1b_relegation_trips_relegation_clause_when_armed`
5. `c15_1b_no_write_when_relegation_clause_already_zero`
6. `c15_1b_no_write_when_relegation_clause_already_two`
7. `c15_1b_no_write_when_non_promotion_clause_zero`
8. `c15_1b_duplicate_person_second_visit_is_naturally_noop`
9. `c15_1b_missing_contract_is_silent_skip`
10. `c15_1b_negative_by_staff_id_sentinel_is_silent_skip`
11. `c15_1b_trace_old_equals_pre_write_new_equals_post_write`

### 21. Commit hash

(this commit)

### 22. Remaining C15.1 items

- **§26.3** squad `+0x3A` writes (SquadRecord) — next tranche C15.1C
- **§26.4** stadium `+0x20` refuse-counter increment — small; a new field on `DomainStadium`
- **§26.5** persistent history-table writes (13-list-per-person model) — largest remaining tranche
- **§16–22** runtime differential vs Frida GDI capture — depends on Group A hooks having GDI VAs

## Divergences the agent flagged in the existing C13 port

The agent noted 7 places where `walk_promotion_persons` and
`walk_relegation_persons` in `c13_promotion_apply.rs` diverge
from the exe. These are **C13-level bugs**, not C15.1B bugs.
Recorded here for a separate follow-up:

1. `walk_promotion_persons` Loop A fabricates `PersonHistoryEvent { kind: 1 }` — the exe's `FUN_00843970` is not a history call; it writes `staff+0x3A` (C15.1C territory).
2. Both walks omit the `FUN_0052A5A0` **out_flag gate**.
3. Both walks omit the `staff+4 == club_id` identity verify.
4. Both walks omit the `person_id < DAT_00acd56c` bound and the `-1` index-table sentinel skip.
5. Relegation history call collapses 4-arg to 2-arg (drops slot ptr, staff ptr).
6. `FUN_0052A5A0`'s `out_flag` collapses to a boolean-present model in the port.
7. Promotion Loop A emits an effect per non-None slot without the person-id upper-bound check.

**C15.1B's materialisation partially compensates** by re-verifying the current byte value (`== 1` predicate) before writing — so a C13 divergence that emits a write for a slot the exe would have rejected still no-ops at materialisation time. But the traces will differ (`PersonEffect` emitted vs `AppliedContractWrite` empty).

## Freeze status

**Byte-write semantics: FROZEN** — the exe's promotion (`1→0`) and relegation (`1→2`) mutations are byte-exact on the canonical Rust storage.

**World storage mapping: FROZEN** — writes hit `world.contracts.records[idx].{non_promotion, relegation}`, the exact canonical location.

**Runtime STATE-EXACT: NOT CLAIMED** — the tranche says "Do not claim StateExact solely from decompile-derived tests." No Frida runtime differential yet.

**C13 divergences: OPEN** — 7 issues flagged; C15.1B write-time re-verification masks most functional divergences but a strict trace-diff against captured GDI would fail. Documented as a follow-up.

## Files changed

- `crates/cm-domain/src/contract_init.rs` — added
  `contract_for_staff_mut`
- `crates/cm-domain/src/c15_1_world_apply.rs` —
  `AppliedContractWrite` type + `ContractWriteKind` enum +
  `apply_contract_writes_from_report` fn +
  `apply_contract_write_for_person` helper + integration into
  `apply_report_to_world` + 11 new tests
