# C15.1E — persistent person-history materialisation

**Branch.** `gdi-renderer-port`.
**Predecessor.** C15.1D (`a19fe8a`).
**Scope.** Recover the persistent storage the exe writes on
`FUN_004D3460 → FUN_008D0D90(..., old_comp, 3, contract_row)`
and materialise the C13 `PersonHistoryEvent` intent into that
storage.

## 1. Executive framing (CRITICAL — the tranche's premise is refuted)

The tranche directive presupposed a **13-list-per-person career
history** model, keyed such that `kind=3` selects list index 3.
The decompile shows this is **not what the exe does**.

`FUN_008D0D90` is a **news-item builder + per-person mailbox
dispatcher** (the surrounding file's Ghidra string constant is
`"C:/dev/CM3.00.01/cm3/code/news.c"`). It:

1. Allocates a 222-byte news item on the caller's stack.
2. Populates a fixed news category (`0xFBF`) and eight
   parameter slots via `FUN_0076D730(item, idx, val)`.
3. Calls `FUN_008BEE00` to collect up to 20 target persons.
4. Dispatches the item to each target's mailbox via
   `FUN_0076E180 → FUN_0076DCE0`.

`kind` (`param_3` on the caller side; `3` for relegation, `0`
for retirement) is stored **as data in slot 6 of the item**,
not used to index a list. There is no 13-list array anywhere
in the chain.

The persistent write lands in a **shared news slab** at
`DAT_00ACD5C4 + person_id * 0x6E → +0xCF → pool`, organised as
per-DOB-age-bucket ring buffers of 100 entries each. Records
are `0xDF` bytes on wire. Growth is by `+100 * 0xDF` slots at a
time via `FUN_009346F7` (`realloc`).

The tranche's persistent-storage owner is therefore the exe's
**per-person news mailbox pool**, not a career-history array.
This port materialises into that storage — modelled as a
`BTreeMap<person_id, Vec<PersonNewsItem>>` — with the
DOB-age-bucket routing and 100-entry ring overwrite left as
documented deviations (§14).

## 2. `FUN_008D0D90` — full signature & effect

`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/008d0d90.c`,
63 lines.

```c
void FUN_008d0d90(undefined4 *param_1, undefined4 *param_2,
                  char param_3, int param_4)
```

| Param | Semantic |
| ----- | -------- |
| `param_1` | Ptr to squad-slot / event context — `*param_1` is the subject person id. |
| `param_2` | Ptr to old competition id — `*param_2` is `old_comp_id`. |
| `param_3` | `kind` byte. Callers pass literal `3` (relegated) or `0` (retirement). |
| `param_4` | Ptr to the contract row — `*(u32*)(param_4 + 0x21)` is the staff id read out. |

Return: `void`.

Persistent writes performed by the function itself: **none.**
Every mutation is on the local stack buffer. Persistence
happens downstream via `FUN_0076E180 → FUN_0076DCE0` (the
mailbox dispatcher and pool writer).

## 3. Storage owner — the per-person news mailbox pool

`DAT_00ACD5C4 + person_id * 0x6E` is the **per-person mailbox
descriptor table** (stride `0x6E`, 110 bytes per person). The
descriptor's `+0xCF` field points into the shared 222-byte
news-item slab. The slab is bucketed by
`(person.dob - DAT_00ACD56C) + 0x10` (DOB-age index), each
bucket a 100-entry ring at stride `0xDF` (223 bytes per slot).
Ring overwrites once the cursor wraps.

**No 13-list-per-person model exists.** The bucket count
depends on the DOB range of the current save's person table,
not on `kind`.

## 4. `kind → list` mapping

There is no switch. `kind` is stored as data at record `+0x1D`
(slot 6). Both callers use the same news category `0xFBF`. The
downstream template renderer differentiates on the stored
`kind` byte at read time.

## 5. Record layout for one news item

Total footprint: 222 bytes (pool stride `0xDF`). Field
inventory (fields actually populated by
`FUN_004D3460 → FUN_008D0D90` with `kind=3`):

| Offset | Width | Source | Semantic |
| ------ | ----- | ------ | -------- |
| `+0x00` | u32 | `FUN_00763B90(item, 0xFBF, 0)` | News category id (`0xFBF`) |
| `+0x04` | u8  | init 0; overwritten by `FUN_0076CE50(person)` per dispatch | Per-person severity byte |
| `+0x05` | u32 | slot 0 — `*param_1` | Subject person id |
| `+0x09..+0x11` | u32×3 | slots 1..=3 — `*(u32*)(param_1[i]+0x33)` | Secondary pointer ids |
| `+0x15` | u32 | slot 4 | Club/nation base id |
| `+0x19` | u32 | slot 5 — `*param_2` | `old_comp_id` |
| `+0x1D` | u32 | slot 6 — `(int)param_3` | `kind` |
| `+0x21` | u32 | slot 7 — `*(u32*)(param_4 + 0x21)` | Staff row id |
| `+0xD5` | u8 | `DAT_00ACDE88` | Season byte (game year lo) |
| `+0xD6` | u32 | pool assigns via `param_1_ctx[4]++` | Monotonic news id |
| `+0xDA` | u32 | pool init 0 | Read/next flag |

There is **no explicit competition-id / club-id / date field
per se** — those are encoded into the numbered parameter slots
and pulled out again by downstream templates.

## 6. Append vs update

Read `FUN_0076DCE0` carefully:

* **Always append** to a per-age-bucket ring. No global dedupe.
* Duplicate suppression at line 65 tests `previous slot's
  news_id != incoming news_id` — because ids are pool-assigned
  and monotonic, this guard essentially always passes.
  Semantic dedupe on `(person, comp, kind, season)` does NOT
  exist.
* Ring wrap on cursor reaching `first + 100` overwrites the
  oldest entry silently.
* No sort. No merge. No coalesce.

## 7. Allocation / capacity

* Initial capacity per bucket: 100 slots.
* Growth: **linear**, `+100 * 0xDF = 22 300` bytes per
  first-use via `FUN_009346F7` (`realloc`).
* Failure: sets pool base to 0, pops an Error dialog, drops
  the item.
* Cap per bucket: **100 entries, ring-overwrite**.

## 8. Order of relegation write vs history append

`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/004d3460.c:33-36`:

```c
if (*(int *)(iVar3 + 4) == *param_1 &&
    *(char *)(iVar3 + 0x1f) == '\x01')
{
    *(undefined1 *)(iVar3 + 0x1f) = 2;   // (1) contract 1 → 2
    FUN_008d0d90(*piVar6, param_2, 3, iVar3);  // (2) news append
}
```

Order: **contract byte first, then news append.** The news
function does NOT read `+0x1F` back — it reads `+0x21`
(staff_id). So the write order is causal state-machine
sequencing, not a read-after-write dependency.

The Rust port preserves the same order: `apply_report_to_world_parts`
now runs (in this sequence)

1. Event walk (promotion/relegation effects, stadium, news).
2. C15.1B contract writes.
3. C15.1C squad-position writes.
4. C15.1E person-history append.

## 9. Caller matrix

Grep `FUN_008d0d90` across the decompile yields **only two
callers**:

| Caller | `kind` | Context | Guard byte | Exclusive to this path |
| ------ | ------ | ------- | ---------- | ---------------------- |
| `FUN_004D3300:48` | `0` | Retirement scan during rollover | `+0x1C` | Retire flag |
| `FUN_004D3460:35` | `3` | Relegation scan during rollover | `+0x1F` | Relegation flag |

Both walk the club's 50-slot `+0xD7` array. Neither reads back
the newly-written contract byte inside the news chain.

## 10. Existing Rust storage — none

Before this tranche:

* `AppliedPersonHistory { person_id, competition_id, year, kind }`
  in `WorldApplyReport.person_history` — a **flat per-apply
  log**, not persistent state.
* No `Vec` on a `PersonRecord`, no sidecar pool, no `NewsPool`
  keyed on person id.

## 11. New Rust types (this tranche)

`crates/cm-domain/src/person_news.rs`:

* `PersonNewsItem { category, severity, params: [u32; 8], year, news_id }`
* `PersonNewsMailboxPool { by_person: BTreeMap<u32, Vec<_>>, next_news_id }`
* `NEWS_CATEGORY_PERSON_CAREER = 0x0FBF`
* `kinds::{ RETIREMENT: 0, RELEGATED: 3 }`

`crates/cm-domain/src/lib.rs`:

* `World.person_news_mailboxes: PersonNewsMailboxPool` (with
  `#[serde(default)]` so older saves load unchanged).

`crates/cm-domain/src/c15_1_world_apply.rs`:

* `AppliedPersonHistoryWrite { person_id, category, kind, old_comp_id, staff_id, old_len, new_len, news_id }`
* `WorldApplyReport.applied_person_history_writes: Vec<_>`
* `apply_person_history_from_report` (public) — walks
  `Relegation` events, resolves each `PersonEffect.person_id`
  via the ContractPool (already updated by C15.1B), applies the
  identity gate (`record.club_id == effects.club_id`), builds
  a `PersonNewsItem`, appends via
  `world.person_news_mailboxes.push`, records the trace.

The applier is wired at the end of `apply_report_to_world_parts`
so BOTH entry points (`_parts` for tests, the full
`apply_report_to_world` for production) run the same pipeline
order.

## 12. C13 → World materialisation path

1. C13 emits `PersonHistoryEvent { old_comp_id, kind }` inside
   each `Relegation.person_effects[i].event_emit`.
2. `apply_report_to_world_parts` walks the event stream.
3. C15.1B `apply_contract_writes_from_report` transitions
   `contract.relegation 1 → 2` (identity-gated).
4. C15.1C `apply_squad_position_writes_from_report` is a
   no-op for `Relegation`.
5. C15.1E `apply_person_history_from_report` resolves the
   contract's `staff_id` and pushes a `PersonNewsItem` into
   `world.person_news_mailboxes.by_person[person_id]`.
6. Trace: one `AppliedPersonHistoryWrite` per landed append.

## 13. Identity semantics

The mailbox is keyed by **`person_id`** — matches the exe's
`DAT_00ACD5C4 + person_id * 0x6E` descriptor lookup. `staff_id`
is a separate value stored in slot 7 of the item, resolved via
the ContractPool.

## 14. Deviations from the exe (documented)

1. **DOB-age bucket routing not modelled.** The exe indexes
   the pool by `(person.dob - DAT_00ACD56C) + 0x10`. Rust
   simulation state does not carry person DOB at this
   granularity, so the port keys the mailbox directly by
   `person_id`. Retrieval consumers hit the per-person
   descriptor first anyway (`DAT_00ACD5C4 + person_id * 0x6E`)
   before the age index; the bucketing is a cross-person
   layout choice the port sidesteps.
2. **100-entry ring overwrite not modelled.** Rust uses an
   append-only `Vec`. Pinned by
   `c15_1e_boundary_no_max_size_is_an_intentional_deviation`.
   Safe at year-end scope: at most one entry per person per
   season. Revisit when per-day dispatches enter scope.
3. **Numbered param slots 1..=4 held at zero.** The C13/C15
   pipeline in Rust does not thread the exe's
   `param_1[1..=3]` pointer chains or the nation base id.
   Slots 0/5/6/7 (person_id / old_comp_id / kind / staff_id)
   — the values downstream templates for kind 0/3 actually
   consume — are populated faithfully.
4. **Per-person severity byte (`+0x04`) not overwritten.**
   The exe writes `FUN_0076CE50(person)` at dispatch time.
   Held at 0 in Rust.
5. **Stride-`0xDF` on-disk footprint not mirrored.** The Rust
   record only carries the fields actually populated (plus
   the `news_id` and `year` bookkeeping). Downstream readers
   consume through the accessor methods on `PersonNewsItem`
   rather than a raw byte view.
6. **Ring-overflow allocator failure not modelled.** No panic
   path exists; append is infallible.

## 15. Tests (11 total)

All in `crates/cm-domain/src/c15_1_world_apply.rs`, prefix
`c15_1e_`:

| Test | Coverage |
| ---- | -------- |
| `relegation_golden_end_to_end` | Contract 1→2 + mailbox append + trace symmetry |
| `multiple_persons_one_event_each_ordered` | Three armed people → three appends, monotonic news_ids |
| `duplicate_person_only_first_transition_writes_history` | Documents C15.1E's payload-driven semantic; C13 controls filter |
| `non_qualifying_contract_no_history` | Unknown person id → silent skip |
| `identity_mismatch_no_history` | Contract belongs to different club → silent skip |
| `no_contract_pool_is_silent_skip` | `World.contracts == None` → skip, no panic |
| `multi_season_appends` | Two apply passes across seasons → 2 entries, correct years |
| `kind_isolation_between_persons` | Two persons + different old_comp_ids → each mailbox gets only its own item |
| `boundary_no_max_size_is_an_intentional_deviation` | 150 appends → 150 entries (pins deviation §14.2) |
| `trace_matches_world_state` | For every trace entry, mailbox_for(pid).last() equals trace |
| `promotion_does_not_touch_mailbox` | Only Relegation variant consumes the path |

Result: **11 passed; 0 failed.** All 60 C15.1 tests green
(C15.1A/B/C/D/E). Full cm-domain lib: 1722 pass, 4 pre-existing
baseline failures (same as before this tranche).

## 16. Confidence

| Item | Level |
| ---- | ----- |
| `FUN_008D0D90` semantics | DIRECT — decompile-verified byte by byte |
| Mailbox storage layout | STRUCTURALLY PORTED — DOB bucket routing collapsed |
| Rust canonical mapping | FROZEN — `PersonNewsMailboxPool` and `PersonNewsItem` are the canonical carriers of `FUN_008D0D90`'s effect |
| World materialisation | STRUCTURALLY PORTED |
| STATE-EXACT parity with exe | Deferred to C15.1G runtime differential |

## 17. Frozen boundaries

* **C15.1A / B / C / D not touched.** Only wire-in for
  `apply_report_to_world_parts` gained the C15.1B/C/E passes.
* **Persistent history materialisation is FROZEN** at this
  layer: the mailbox owner and record schema will not change
  in later tranches without an explicit reopening.
* **C15.1F (finance loader `FUN_005121A0`) deferred.**
* **Frida differential (C15.1G) deferred.**

## 18. Commit hash

Landed at `<filled in after commit>` on `gdi-renderer-port`.
