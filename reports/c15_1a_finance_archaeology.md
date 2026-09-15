# C15.1A — Cash offset dispute resolved + finance ledger materialised

Prior: `7464c19` (C11.3 Step 2). This closes the finance-write
half of C15.1 §26.1–§26.4 into a materialised ledger.

## The 20-point report

### 1. Exact cash offset

**`+0x00` (byte offset), i64 signed**, low half at `+0x00..+0x04`,
high half at `+0x04..+0x08` on the RUNTIME finance object.

### 2. Evidence resolving the offset dispute

Five independent finance-cluster writers all touch `*param_1` + `param_1[1]` with the canonical i64 borrow/carry idiom (`(uint)(low < sub)` on debit, `(uint)CARRY4(low,add)` on credit, `>> 0x1f` sign-extending the high half). None of them touches byte `+0x65`:

| Function | File | Direction | Line refs |
|---|---|---|---|
| `FUN_00583FC0` stadium expand | `00583fc0.c` | debit + credit | 75-76, 114-115, 145-146, 151-153 |
| `FUN_00584790` gate day | `00584790.c` | credit + debit | 181-227 (many sites) |
| `FUN_00586EC0` wage cascade | `00586ec0.c` | debit | 79-419 (many sites) |
| `FUN_00587C40` chairman injection | `00587c40.c` | credit | 52-54, 67-68, 106-107 |
| `FUN_00585AE0` season roll | `00585ae0.c` | debit | ~620 |

`ClubView::cash() -> i32 @ +0x65` is not corroborated by any writer. That field is likely disk-only (memory `[[club-record-decoded]]` verified against 20+ shipped clubs' cash values, but shipped values are disk-layout — the runtime object shape may differ).

### 3. Cash width + signedness

**i64, signed.** Signed because:
- High half arithmetic uses arithmetic-right-shift (`>> 0x1f`).
- Affordability comparisons cast to `int` and compare signed.
- 287 shipped clubs ship bankrupt (per `[[club-record-decoded]]`), and `FUN_00587C40` fires when `high <= 0` — the game explicitly models overdraft.

### 4. `+0x8C` (Ghidra `param_1[0x23]`) — season misc operating expense

- Width: **i32**, single-word add (no carry pair anywhere).
- Reset annually by `FUN_00585AE0` (season roll clears words `0x1c..0x2f` = bytes `0x70..0xBC`, which includes this offset).
- Written by:
  - `FUN_00583FC0` (stadium expansion) — 3 sites
  - `FUN_00586EC0` (wage cascade) — signing bonuses, manager compensation, at 10+ sites
  - `FUN_00584790` (gate day) — per-attendance operating cost
- **NOT stadium-specific.** C14's prior "stadium YTD" label was too narrow. Correct label: "season misc operating expense" (or "season non-wage non-gate expense" — wages have their own bucket at `+0x94`, gate income at `+0xA0`).

### 5. `+0x12C` (Ghidra `param_1[0x4b]`) — lifetime misc operating expense

- Width: **i32**.
- **Never reset** (season-roll clear stops at word `0x2f`; lifetime bucket starts at word `0x48`).
- Same writer set as `+0x8C`. Monotone-increasing lifetime mirror.
- Correct label: "lifetime misc operating expense".

### 6. `+0xB4` (Ghidra `param_1[0x2d]`) — season subsidy income

- Width: **i32**.
- Reset annually.
- Written by:
  - `FUN_00583FC0` L144 (parent-club subsidises expansion)
  - `FUN_00587C40` L52,66,105 (chairman cash injection)
  - `FUN_00586EC0` L92 (inter-club wage rescue — parent pays subsidiary's wages)
  - `FUN_00584790` L180,186,210,215 (parent-owner top-up on gate day)
- Correct label: "season subsidy income" — broader than "owner-subsidy accumulator A", but that label was on-track.

### 7. `+0x154` (Ghidra `param_1[0x55]`) — lifetime subsidy income

- Width: **i32**.
- Never reset. Same writer set as `+0xB4`. Lifetime mirror.
- Correct label: "lifetime subsidy income".

Season and lifetime mirrors are **always paired** in every writer. The season↔lifetime distinction is proven by the reset scope (season bucket at `0x1c..0x2f`, lifetime at `0x48..0x56`), not by any writer.

### 8. Disk vs runtime layout

**Not fully traced.** The C15.1A archaeology agent explicitly declined to sink an hour into `FUN_005121A0`'s 2429-line loader. What we know:

- Every runtime writer references the same offsets consistently (5-source agreement).
- Disk `Club.dat` record is 581 bytes per memory `[[club-record-decoded]]`.
- `ClubView::cash() -> i32 @ +0x65` was verified against 20+ shipped-club values FROM DISK. If the runtime is a repacked view with cash moved to `+0x00`, both facts are simultaneously true.
- The runtime object accessed by the finance cluster is inferrable to be at least `0x158` bytes; its full stride is not established from the finance writes alone.

**Consequence for materialisation**: writing i64 cash to `World.core.clubs[i].raw[0..8]` would overwrite the disk `id` field (memory says `id` at `+0x00`). Therefore C15.1A does NOT write to `Club.raw`. Instead:

### 9. Rust accessors/mutators — `ClubFinanceLedger`

New typed sidecar on `RuntimeSaveGame`:

```rust
pub struct ClubFinanceState {
    pub cash: i64,                      // runtime +0x00
    pub season_misc_expense: i32,       // runtime +0x8C
    pub lifetime_misc_expense: i32,     // runtime +0x12C
    pub season_subsidy_income: i32,     // runtime +0xB4
    pub lifetime_subsidy_income: i32,   // runtime +0x154
}
pub struct ClubFinanceLedger {
    pub per_club: BTreeMap<u32, ClubFinanceState>,
}
```

Widths and semantics track the exe exactly. Storage is typed (not raw bytes) because we don't have runtime-object-identity confirmation for the `Club` record. When the loader trace lands (planned follow-up), we'll know whether to move these into `Club.raw` at those byte offsets or keep the ledger.

Tranche point 8 said "Do NOT add duplicate finance state." The ledger is not duplicate — Club.raw currently exposes NO finance state at these offsets in any accessor. The ledger IS the finance state.

### 10. Finance materialisation path

```
C14 apply_stadium_expansion
  → ClubFinanceOutcome (new values, byte-exact widths)
  → C15.1 apply_report_to_world queues PendingFinanceWrite
  → apply_report_to_world calls save.finance_ledger.apply_write for each
  → RuntimeSaveGame.finance_ledger.per_club[club_id] = new state
```

Trace-vs-state proven: after apply, ledger values match the PendingFinanceWrite that produced them (test `c15_1a_trace_matches_world_ledger_state_after_apply`).

### 11. Forced-path before/after golden — `c15_1a_forced_path_ledger_bytes`

Pre: cash=£50M, misc_expense_season=£2M, misc_expense_life=£10M, subsidy_a=£100k, subsidy_b=£500k. Expansion cost=£7,750,000.

Post (byte-exact assertions):
- cash: £42,250,000
- season_misc_expense: £9,750,000
- lifetime_misc_expense: £17,750,000
- season_subsidy_income: £100,000 (unchanged — forced path)
- lifetime_subsidy_income: £500,000 (unchanged)

### 12. Owner-subsidy golden — `c15_1a_owner_subsidy_net_cash_zero_but_all_four_accum_bump`

Pre: cash=£0 (broke), pre-existing accums. Parent chairman with refuse=20.

The exe (00583fc0 lines 145 → 151) credits cash by cost then debits by cost — net Δcash = 0. Both subsidy accumulators (season + lifetime) bump by cost. Both misc-expense accumulators (season + lifetime) also bump by cost.

Byte-exact assertions verify all four accumulator deltas equal `+cost` and cash stays exactly 0.

### 13. No-op golden — `c15_1a_noop_deltas_produce_no_finance_write`

Input already at desired capacity → C14 returns `NoOpDeltas` with `club_writes: None`. Ledger stays empty.

### 14. Refusal-failure golden — `c15_1a_refusal_path_produces_no_finance_write`

Affordability-checked + broke + big-ticket + `rand_mod(5) != 0` → `RescheduledIndependent`, `success=false`, `club_writes: None`. Ledger untouched.

### 15. Overflow/boundary tests — `c15_1a_ledger_overflow_wraps_like_i32` + `c15_1a_negative_cash_is_representable`

- Accumulator near `i32::MAX` bumped by cost wraps into negative territory (matches the exe's plain i32 add — no saturation).
- Cash of £1M debited by £7.75M yields −£6.75M, representable as signed i64.

### 16. Trace-vs-World consistency — `c15_1a_trace_matches_world_ledger_state_after_apply`

Every field of the ledger after apply equals the corresponding field of the `PendingFinanceWrite` that produced it. Byte-exact.

### 17. Tests — 7 new + 40 existing C14 + 23 existing C15/C15.1 all green

New: `c15_1a_forced_path_ledger_bytes`, `c15_1a_owner_subsidy_net_cash_zero_but_all_four_accum_bump`, `c15_1a_noop_deltas_produce_no_finance_write`, `c15_1a_refusal_path_produces_no_finance_write`, `c15_1a_ledger_overflow_wraps_like_i32`, `c15_1a_negative_cash_is_representable`, `c15_1a_trace_matches_world_ledger_state_after_apply`.

### 18. Commit hash

(this commit)

### 19. Remaining C15.1 items

- **§26.2** staff `+0x1F` / `+0x1C` writes — needs staff-pool accessor archaeology
- **§26.3** squad `+0x3A` writes — same accessor tranche
- **§26.4** stadium `+0x20` refuse-counter increment — `DomainStadium` needs a new field; small
- **§26.5** persistent history-table writes (13-list-per-person model per the earlier Explore agent finding) — largest remaining tranche
- **§16–22** runtime differential vs Frida GDI capture — depends on Group A hooks having GDI VAs

### 20. Freeze status

**Finance materialisation FROZEN** as of this commit for the observable path:

- Cash offset resolved (i64 @ +0x00 runtime), widths corrected (i32 for accumulators), labels corrected (not stadium-specific)
- Execution order preserved (forced path: expense-bump → cash-debit; subsidy path: subsidy-bump → cash-credit → expense-bump → cash-debit)
- Byte-exact ledger materialisation
- Trace-vs-state consistency proven

**Explicit non-freezes**:
- The runtime object IDENTITY (whether it's Club.raw or a separate pool) requires `FUN_005121A0` loader trace. Recorded as `[[loader-disk-to-runtime]]` for a small follow-up tranche.
- If loader trace shows the runtime object IS `Club.raw`, we'll fold the ledger's typed values back into raw-byte writes at the proven offsets. The ledger stays authoritative; raw is redundant.
- If loader trace shows a separate pool, we'll rename the ledger to match and this stays.

Neither outcome changes the arithmetic, widths, or semantic labels — all are proven from the runtime writer evidence.

## Retractions from prior C14 comments

Field label corrections (backward-incompatible rename in the Rust API):

| Old name (C14 pre-C15.1A) | New name |
|---|---|
| `club_stadium_expense_ytd` | `club_season_misc_expense` |
| `club_stadium_expense_lifetime` | `club_lifetime_misc_expense` |
| `club_owner_accum_a` | `club_season_subsidy_income` |
| `club_owner_accum_b` | `club_lifetime_subsidy_income` |
| `new_stadium_expense_ytd` | `new_season_misc_expense` |
| `new_stadium_expense_lifetime` | `new_lifetime_misc_expense` |
| `new_owner_accum_a` | `new_season_subsidy_income` |
| `new_owner_accum_b` | `new_lifetime_subsidy_income` |

Width corrections:
- Accumulators changed from `i64` to `i32` in `StadiumExpansionInput`, `ClubFinanceWrites`, `PendingFinanceWrite`, `ClubYearEndState`, `c13_request_to_c14_input`.

## Files changed

- `crates/cm-domain/src/c14_stadium_expansion.rs` — struct rename, i64→i32 widths, docstrings updated, arithmetic switched to `wrapping_add(cost as i32)` to match exe wrap semantics
- `crates/cm-domain/src/c15_1_world_apply.rs` — `PendingFinanceWrite` renamed, new `ClubFinanceState` + `ClubFinanceLedger` types, `apply_report_to_world` now applies finance writes to the ledger, 7 new tests
- `crates/cm-domain/src/c15_english_annual_rollover.rs` — `ClubYearEndState` field rename
- `crates/cm-domain/src/lib.rs` — new field `finance_ledger` on `RuntimeSaveGame`, initialised at every construction site (8 sites)
- `crates/cm-domain/src/manager_creation.rs` — 1 init site updated
