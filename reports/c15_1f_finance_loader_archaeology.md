# C15.1F — Runtime Finance Object Loader Mapping

**Branch.** `gdi-renderer-port`.
**Predecessor.** C15.1E (`386acfa`).
**Scope.** Resolve the executable's runtime finance-object
mapping so `ClubFinanceLedger` is either frozen as canonical or
migrated to a proven canonical location, before the final
Traditional English differential (C15.1G).

## 1. `FUN_005121A0` — identity

`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/005121a0.c`,
2430 lines.

* Signature: `undefined4 FUN_005121a0(undefined4 param_1, int param_2, int param_3)`.
* `param_1` — opaque file/memory reader context.
* `param_2` — `DAT_00acd548` mode flag (0 = normal, non-0 =
  compact / handle-alloc via `GlobalAlloc`).
* `param_3` — 0 = disk load, 1 = memory / save-restore load.
* Return: 0 on error; non-zero on success.

**What it does.** Reads `index.dat` and dispatches over a
22-entry descriptor table (`piVar12` / `DAT_00acccbf`, 0x43
bytes each). For each descriptor `case 0..0x16` it `malloc`s
(or `GlobalAlloc`s) that pool and stores the base into the
matching global (`DAT_00acd5AC..DAT_00acd60C`). Pool table
(subset relevant to this tranche):

| case | base var | stride | role |
| ---: | -------- | -----: | ---- |
| 0/1 | `DAT_00acd5bc`/`DAT_00acd5c0` | 0x245 | Disk Club record |
| 6 | `DAT_00acd5c4` | 0x6e | Staff / Person |

`FUN_005121A0` does NOT allocate a 0x167-stride pool. **The
Runtime Finance record is not one of its pools.** Callers:
`FUN_0050E9B0` (disk, from new-game orchestrator
`008120d0.c:118` etc.) and `FUN_0050E970` (memory, from
`00814870.c:100` — save-restore).

## 2. Disk Club input provenance

Disk clubs are loaded into `DAT_00acd5bc` at stride `0x245`
(581 bytes). This preserves the full on-disk `Club` record —
including `Club+0x00 = club_id` and `Club+0x65 = i32 cash
seed`.

## 3. Runtime finance object — separate pool

The Runtime Finance record is allocated by `FUN_00584530`
(`005803d0.c` neighbour):

```
piVar4 = operator_new(DAT_00acd564 * 0x167 + 4);   // count * 0x167 + 4-byte header
FUN_0093543f(piVar8, 0x167, iVar7, &LAB_005803c0, FUN_0061d290);
```

Called from the new-game top-level `008120d0.c:1144` and the
save-restore top-level `00814870.c:1413`:

```
DAT_00acdc38 = FUN_00584530(0);
```

`DAT_00acdc38` is a wrapper; `*DAT_00acdc38` = pool base. Total
count = `DAT_00acd564` (the "case 0/1" Club count).

Per-record seed via `FUN_005803D0` (per-club ctor, called by
`FUN_0093543F` batch construction, walking the Club array at
stride `0x245`):

```
iVar5 = *(int *)(param_2 + 0x65);   // disk Club+0x65 (i32 cash seed)
FUN_008fc4f0(iVar5 / 10);           // seed the RNG (side effect)
*(u64*)param_1 = __ftol(iVar5);     // widen to i64, write runtime +0x00
*(u32*)(param_1 + 8) = *param_2;    // mirror club_id at finance +0x08
// … many zero-inits …
*(u8*)(param_1 + 0x164) = 3;        // status/tickdown bytes
*(u8*)(param_1 + 0x165) = 0;
*(u8*)(param_1 + 0x166) = 5|10|0xF; // variant seed
```

## 4. `FUN_00583FC0` arg1 provenance

Every call to `FUN_00583FC0` is preceded by
`FUN_0058A490(club_record_ptr)`, which returns the per-club
finance record address:

```c
// 0058a490.c:23
return *param_2 * 0x167 + *param_1;   // pool_base + club_id * 0x167
```

This exact idiom appears in 16 sites across the finance-cluster
files (`00584790.c`, `00585060.c`, `00585750.c`, `00585900.c`,
`00586ec0.c`, `00587c40.c`, `00588c70.c`, plus `00583fc0.c`
itself). Therefore `FUN_00583FC0`'s `param_1` is a pointer into
a **separate 0x167-stride pool**, not the disk Club record.

## 5. Reconciling the `+0x00` conflict

Both are true because there are **two distinct objects**, each
with its own `+0x00`:

* Disk `Club` record (`DAT_00acd5bc + club_id * 0x245`) —
  `+0x00 = u32 club_id`.
* Runtime Finance record (`*DAT_00acdc38 + club_id * 0x167`) —
  `+0x00 = i64 cash`, `+0x08 = u32 club_id` (mirrored so the
  finance code can invert to the Club record).

No pointer adjustment on a single object; no wrapper embedding.
Two independently-allocated parallel arrays keyed by the same
ordinal.

## 6. Identity mapping

**Same ordinal, keyed by `club_id`.** No lookup table. No
embedded pointer. `finance_record[i] = *DAT_00acdc38 + i * 0x167`.

## 7. Runtime finance record — layout & size

**Exactly 0x167 bytes (359 bytes) per record**, evidence
converged across three independent points:

1. Allocation size: `count * 0x167 + 4` (`FUN_00584530:42`).
2. Batch-ctor stride: `FUN_0093543F(pool, 0x167, …)`
   (`FUN_00584530:48`).
3. Save/load record size: `FUN_00921EA0(*pool, 0x167, count,
   fd)` (`FUN_005854D0:24`).
4. Max offset observed by any writer: `+0x166` (status byte,
   `FUN_005803D0:103` — fits within 0x167).

Field inventory used by year-end / stadium-expansion code:

| Offset | Type | Field | Rust field |
| -----: | ---- | ----- | ---------- |
| `+0x00` | i64 | Cash | `ClubFinanceState.cash` |
| `+0x08` | u32 | Mirrored `club_id` | (map key) |
| `+0x8C` | i32 | Season misc expense | `season_misc_expense` |
| `+0xB4` | i32 | Season subsidy income | `season_subsidy_income` |
| `+0x12C` | i32 | Lifetime misc expense | `lifetime_misc_expense` |
| `+0x154` | i32 | Lifetime subsidy income | `lifetime_subsidy_income` |
| `+0x14, +0x24, +0x2C, +0x34..+0x15C` | i32×24+ | Other accumulators (weekly wages, TV, prize money, gate day, attendance) | **NOT modelled this tranche** |
| `+0x164..+0x166` | u8×3 | Status / tickdown bytes | not modelled this tranche |

The five modelled fields are the only ones any of the
already-ported writers (`FUN_00583FC0`, `FUN_00587C40`,
`FUN_00586EC0`, `FUN_00584790`, `FUN_00585AE0`) touch. Adding
the other DWORDs is deferred to later tranches that port the
weekly-wages / TV-income / prize-money writers.

## 8. Serialisation ownership

The runtime finance pool is persisted as its own `finance.dat`
sub-file inside the `.sav` bundle
(`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/005854d0.c`):

```
FUN_00921770(param_2, "finance.dat", 1, 0x16, 4);       // open
FUN_00921ea0(*param_1, 0x167, DAT_00acd564, fd);        // read/write count * 0x167
```

Dispatcher `00818060.c:551` fires `FUN_005854D0` whenever
`DAT_00acdc38 != 0`. **The full 0x167-byte record per club is
round-tripped verbatim.** `Club+0x65` is the seed only; it
is NOT re-read at save-load. The Rust port therefore MUST
persist the ledger — which it already does via `serde` on
`RuntimeSaveGame.finance_ledger`.

## 9. `ClubView::cash` — actual role (RENAMED)

The Rust `typed_records::ClubView::cash()` reads
`Club.raw[0x65..0x69]` as `i32`. The C15.1F archaeology
confirms this is **the one-time new-game boot seed** — the only
reader in the exe is `FUN_005803D0` at lines 47/96/100. After
boot, live cash lives on the runtime finance pool at `+0x00`
(i64).

**Rename.** This tranche adds
`ClubView::initial_cash_seed() -> i32` as the correctly-named
accessor. The old `ClubView::cash()` is retained as a
`#[deprecated]` alias pointing to `initial_cash_seed()` so
existing importer / finance-seed callers continue to compile
while callers migrate.

Migrations landed this tranche:

* `crates/cm-domain/src/finance.rs:549` — seed helper switched.
* `crates/cm-import/src/bin/regen_clubs.rs:43,96` — importer +
  debug print switched.

## 10. The four accumulators — same object

All four accumulators (`+0x8C`, `+0xB4`, `+0x12C`, `+0x154`)
belong to the same 0x167-byte finance record. Cross-reference
across five independent writers proves object identity:

| Writer | Role | +0x00 (i64 cash) | +0x8C | +0xB4 | +0x12C | +0x154 |
| ------ | ---- | :---: | :---: | :---: | :---: | :---: |
| `FUN_00583FC0` (stadium expansion) | build cost + owner subsidy | ↓ | ↑ | ↑ | ↑ | ↑ |
| `FUN_00587C40` (chairman injection) | reputation cash gift | ↑ | – | ↑ | – | ↑ |
| `FUN_00586EC0` (inter-club wage rescue) | parent-owner wage top-up | via `*param_1 + *piVar15 * 0x167` | – | ✓ | – | ✓ |
| `FUN_00584790` (parent-owner gate-day) | gate-day subsidy | via same resolver | – | ✓ | – | ✓ |
| `FUN_00585AE0` (season roll reset) | zero season accumulators | – | reset | reset | – | – |

## 11. Chosen canonical Rust model — **Option B, sidecar promoted**

`ClubFinanceLedger` is confirmed as the canonical Rust carrier
for the exe's per-club Runtime Finance record. Freezing
rationale:

1. The exe treats it as a first-class serialisable pool
   (`finance.dat`). Sidecar semantics — a `.sav`-persisted
   `BTreeMap<club_id, ClubFinanceState>` — are the exact Rust
   analogue.
2. Same-ordinal indexing by `club_id` matches the exe
   byte-for-byte.
3. `Club` disk record stays clean — no field folding, no
   duplicate state.
4. Room to grow: `#[serde(default)]` on `per_club` and future
   fields via serde-default means backward-compatible extension
   as more writers land (weekly wages, TV, prize money, gate
   day) in later tranches.

## 12. Migrations / refactors made

1. `ClubFinanceState::from_disk_seed(disk_cash_seed: i32)` —
   ports `FUN_005803D0`'s i32→i64 widening seed.
2. `ClubFinanceLedger::seed_from_world(&World)` — walks
   `world.core.clubs`, seeds every ledger entry from
   `ClubView::initial_cash_seed()`. Ports the exe's
   `FUN_00584530 → FUN_005803D0` batch ctor.
3. `#[serde(default)]` on `ClubFinanceLedger.per_club` —
   old saves that never carried a ledger load as empty.
4. `ClubView::initial_cash_seed()` added; `ClubView::cash()`
   kept as `#[deprecated]` alias.
5. Extensive doc headers on `ClubFinanceState` and
   `ClubFinanceLedger` freezing them as canonical carriers.
6. Migrated `finance.rs:549` and `regen_clubs.rs:43,96` to
   the new accessor name.

No canonical location was pre-existing — the archaeology
confirms `ClubFinanceLedger` is the right home. No duplicate
state was introduced.

## 13. Initialization tests (this tranche)

* `c15_1f_from_disk_seed_widens_i32_to_i64` — cash widening,
  accumulator zero-init, negative + extreme seeds.
* `c15_1f_seed_from_world_populates_every_club` — world→ledger
  seed with a synthetic +0x65 byte pattern.
* `c15_1f_seed_from_world_is_idempotent` — calling twice is a
  no-op.

## 14. Save/load tests

* `c15_1f_serde_round_trip_preserves_state` — JSON
  serialise/deserialise a seeded+applied ledger; exact
  equality.
* `c15_1f_serde_default_lets_empty_json_deserialise` — old
  saves without the field still load.

## 15. Trace-vs-World tests

* `c15_1f_apply_write_overwrites_seed` — seed then apply
  overwrites correctly.
* `c15_1f_trace_vs_ledger_consistency` — apply a sequence of
  `PendingFinanceWrite`s and confirm the ledger reflects the
  last write per `club_id`.

## 16. Files changed

* `crates/cm-domain/src/c15_1_world_apply.rs` — canonical doc
  header + `from_disk_seed` + `seed_from_world` + 8 tests.
* `crates/cm-domain/src/typed_records.rs` —
  `initial_cash_seed` + deprecated `cash` alias.
* `crates/cm-domain/src/finance.rs` — internal callsite.
* `crates/cm-import/src/bin/regen_clubs.rs` — importer
  callsites.

## 17. Tests

8 new `c15_1f_*` cases, all green. Full C15.1x suite (A/B/C/D/E/F):
68 tests, all green. Full cm-domain lib: 1730 pass, 4
pre-existing baseline failures unchanged.

## 18. Commit hash

`<filled in after commit>` on `gdi-renderer-port`.

## 19. Remaining deviations

* **Only the five accumulators the year-end/stadium-expansion
  code touches are modelled.** The exe's 0x167-byte record
  carries at least 24 more DWORD slots at `+0x14, +0x24, +0x2C,
  +0x34..+0x15C` for weekly wages, TV, prize money, gate day,
  and attendance. Those land as later tranches port their
  writers. Extension is backward-compatible via
  `#[serde(default)]`.
* **Status / tickdown bytes at `+0x164..+0x166` not modelled.**
  Same rationale.
* **Runtime object mirror of `club_id` at `+0x08` not stored.**
  The Rust map key IS the club_id — no need for a mirror.
* **The mirror-cross-check `finance[0x08] == Club[0x00]`
  (implicit in `FUN_00583FC0:24` — resolves the Club record
  from a finance pointer via `DAT_00acd5bc + finance[0x08] *
  0x245`) is trivially satisfied by the map keying.**

## 20. Frozen?

**Yes — the finance World-mapping is now FROZEN.**
`ClubFinanceLedger` is the canonical Rust carrier for the exe's
per-club Runtime Finance record. Future extension is bounded
to adding fields to `ClubFinanceState` — the ledger's shape,
key, and location are locked.

---

## Precondition check for C15.1G

All World-storage mappings that year-end rollover touches are
now resolved:

| Subsystem | Rust canonical storage |
| --------- | ---------------------- |
| Club movement (`+0x57 / +0x5B / +0x37`) | `world.core.clubs[i].raw` |
| Contract clauses (`+0x1C / +0x1F`) | `world.contracts.records[]` (C15.1B) |
| Squad position (`+0x3A`) | `world.contracts.records[].position_code` (C15.1C) |
| Stadium capacity | `world.references.stadiums[]` (C15.1A/D) |
| Stadium refuse counter (`+0x20`) | `world.references.stadiums[].owner_refuse_counter` (C15.1D) |
| Person news / history | `world.person_news_mailboxes` (C15.1E) |
| **Finance** | **`save.finance_ledger`** (C15.1F — this tranche) |

C15.1G — authoritative GDI year-end runtime differential and
final Traditional English freeze — can now proceed.
