# C15.1G — Traditional English Year-End Runtime Differential

**Branch.** `gdi-renderer-port`.
**Predecessor.** C15.1F (`5695a1b`).
**Scope.** Establish the schema, tooling, and Rust-side
production goldens for a runtime differential between the
authoritative `cm0102_GDI.exe` year-end rollover and the Rust
production pipeline; freeze what the tests prove and clearly
scope what the executable-side capture must still verify.

## Executive summary

This tranche's authoritative Frida capture step is **executable
only on the user's Windows machine**. That is a physical
constraint: the capture requires `cm0102_GDI.exe` to be running
with a save queued at the season boundary, and no headless
replay of the exe exists. The Rust-side pieces that DO NOT
depend on a live capture — the semantic snapshot schema, the
production-path golden, the save/load round-trip, and the
diff comparator — are complete and tested; the capture
harness and its analyser are shipped and ready for the user to
run.

**Provisional freeze.** With the C15.1A–F World mappings all
resolved and the C15.1G production-path snapshot proven
deterministic across repeat runs and save/load, the Traditional
English 2001→2002 year-end core is **provisionally frozen at
the ported-semantics layer**. Full STATE-EXACT parity with the
exe is deferred to whichever session runs the Frida capture.
When that capture lands, the Rust-side differential tool will
either confirm zero divergences (upgrading confidence to
STATE-EXACT for the observed branches) or surface the first
divergence to trace.

## 1. Capture harness / hooks

`reports/fixture_disasm/gdi_year_end_capture.py` — Frida
script. Hooks the 12 functions in the tranche directive §1:

| Function | Role |
| -------- | ---- |
| `FUN_00669FA0` | status stamping |
| `FUN_0066EED0` | generic P/R swap |
| `FUN_00668380` | promotion install |
| `FUN_00668470` | Conference/fallback relegation install |
| `FUN_004D3460` | relegation contract/history walk |
| `FUN_004D3550` | promotion contract/squad walk |
| `FUN_00843970` | SquadRecord +0x3A write |
| `FUN_008D0D90` | person news mailbox append |
| `FUN_00583FC0` | stadium expansion |
| `FUN_0058A310` | stadium-expansion news |
| `FUN_0055EE90` | active English pyramid orchestrator |
| `FUN_0055EA00` | inactive-Conference fallback peer |

Each hook captures entry + leave state, resolving runtime
pointers to stable semantic ids (club_id, person_id,
stadium_id) using the finance-pool wrapper `DAT_00acdc38`,
person-index table `DAT_00acdf0c`, contract pool
`DAT_00accad8`, and mailbox descriptor base `DAT_00acd5c4`
(all identified in prior archaeology).

Usage from the user's Windows session:

```
python reports\fixture_disasm\gdi_year_end_capture.py \
    > fixtures\gdi_captures\year_end_2001_02.jsonl
```

## 2. Raw golden capture path

`fixtures/gdi_captures/year_end_2001_02.jsonl` — created by
running the harness against a save at the season boundary and
advancing one day inside the exe. **Not present in-tree yet**
— this tranche ships the harness; the capture itself is a
one-time user-executed step.

## 3. Semantic analyser

`tools/c15_1g/analyse_gdi_capture.py` — reads a raw JSONL and
emits a `YearEndSnapshot`-shaped JSON. The output deserialises
directly via `serde_json::from_str::<YearEndSnapshot>(...)` on
the Rust side, so the differential is a single equality check.

## 4. Captured major call order

Left blank pending the user's capture run. The analyser's
`events` array preserves emission order from Frida's `seq`
counter, so the first differential — call order — is a plain
vector-compare between the analyser's output and the Rust
`YearEndSnapshot.events`.

## 5. Rust production snapshot path

`crates/cm-domain/src/c15_1g_snapshot.rs::YearEndSnapshot::from_apply(year, world, ledger, applier)`
is the sole sanctioned constructor. It reads:

* Ordered event list from `WorldApplyReport` (the tracing
  vectors populated by C15.1B/C/D/E/F).
* Post-rollover club status from
  `WorldApplyReport.post_rollover_club_status`.
* Post-rollover finance from `ClubFinanceLedger.per_club`
  (C15.1F).
* Post-rollover mailboxes from
  `World.person_news_mailboxes.by_person` (C15.1E).
* Post-rollover stadiums from
  `World.references.stadiums` filtered to those the rollover
  touched (C15.1A/D).

Every event/state emission originates from the same production
apply pass a real rollover uses (`apply_report_to_world_parts`).

## 6–18. Differential layers

The Rust differential tool
`crates/cm-domain/src/c15_1g_snapshot.rs::diff_snapshots(gdi, rust)`
produces a `Vec<DifferentialLayerResult>` — one entry per
layer, each with `mismatches` count and `first_divergence`
prose.

| Layer | Comparator | Target |
| ----- | ---------- | ------ |
| A call_order | ordered event-kind sequence | 0 |
| B club_movement | `post_rollover_club_status` map | 0 |
| C contract | filtered `ContractClauseWrite` events | 0 |
| D squad_position | filtered `SquadPositionWrite` events | 0 |
| E history | mailbox contents keyed by `person_id` | 0 |
| F stadium | `stadiums` map | 0 |
| G finance | `finance` map | 0 |
| H news | filtered `NewsEmitted` events | 0 |
| I rng | (not modelled in snapshot) | — |

RNG is out of the current snapshot schema. If the user's
capture surfaces an RNG boundary that this tranche's semantic
tests do not exercise (specifically Conference sort/shuffle
and any affordability-checked stadium expansion), we will
extend `YearEndSnapshot` with RNG waypoints in a follow-up.
The forced-promotion expansion path is documented
RNG-clean.

**Verdict per layer is pending the user's Frida capture.**

## 19. C13 deviation resolutions

The seven C13-level trace differences documented in C15.1B are
masked by write-time re-verification (`record.relegation == 1`
gate). Until the runtime differential fires and the analyser
surfaces one, no re-classification is warranted. **Deferred
pending capture.**

## 20. C15.1C deviation resolutions

| Deviation | Classification |
| --------- | -------------- |
| Nation-affiliation identity clause not ported | UNEXERCISED IN COVERED TESTS. Route from `Club[+0xBF]` → nation base → cross-club match. Impact: a person whose contract's `club_id` differs but whose nation-record base matches would be missed. Real capture will surface any such case as a `SquadPositionWrite` divergence. Deferred pending capture. |
| Loop-A driven by `person_effects` (superset) | STATE-RELEVANT ONLY IF THE SUPERSET OVER-WRITES A NON-CLUB-A PERSON. Identity gate `record.club_id == club_id` (added retroactively in C15.1C) prevents over-write. Verified in `c15_1c_fifty_slot_walk_mixed_valid_missing_mismatch`. |
| Error dialog omission on out-of-range `param_3` | DIAGNOSTIC/UI ONLY. Not part of the observable state. Kept as documented behaviour. |

## 21. C15.1E representation deviations

Explicit list (per report §14 in
`reports/c15_1e_history_archaeology.md`):

| Deviation | Representation vs semantic |
| --------- | -------------------------- |
| DOB-age bucket routing | REPRESENTATION. Semantic append order per person is preserved. |
| 100-entry ring overwrite | REPRESENTATION. At year-end scope, at most one entry per person per season; never fires. |
| `+0x04` severity byte | REPRESENTATION. Held at 0; downstream template code overwrites at dispatch. |
| Stride-`0xDF` footprint | REPRESENTATION. Rust records only the populated slots. |
| Numbered slots 1..=4 | REPRESENTATION at this coverage level. If the capture surfaces a downstream template that reads those slots, upgrade. |
| Allocator failure path | REPRESENTATION. Not reachable at year-end scope. |

The C15.1E storage is **not** labelled BYTE-EXACT; the label
is STRUCTURALLY-PORTED with observable-semantics parity.

## 22. Confidence upgrades

Provisional. Upgrade only after the user's capture confirms
zero divergences on each layer.

| Component | Current | On clean capture (observed branches only) |
| --------- | ------- | ---------------------------------------- |
| C12 status stamping | STRUCTURALLY PORTED | STATE-EXACT |
| C13 promotion/relegation apply | STRUCTURALLY PORTED | STATE-EXACT |
| C13 contract walks | STRUCTURALLY PORTED | STATE-EXACT |
| C14 forced-promotion expansion | STRUCTURALLY PORTED | STATE-EXACT if observed |
| C14.5 squad reset | STRUCTURALLY PORTED | STATE-EXACT |
| C14.7 active/fallback path | STRUCTURALLY PORTED | STATE-EXACT for captured branch only |
| C15 composition | STATE-EXACT to captured semantics | – |
| C15.1A finance | STATE-EXACT to accumulator writes | – |
| C15.1B contract clauses | FROZEN | – |
| C15.1C squad +0x3A | FROZEN | Nation-affiliation deviation still open |
| C15.1D stadium refuse counter | FROZEN | Conference-fallback C14 gap still open |
| C15.1E mailbox | FROZEN (semantic) | – |
| C15.1F finance ledger | FROZEN | – |

## 23. Production final-snapshot golden

`c15_1g_production_end_to_end_relegation` — drives the real
production `apply_report_to_world_parts` pipeline against a
Relegation event and asserts:

* C15.1B contract clause tripped
* C15.1E mailbox append with correct category/kind/old_comp/staff_id
* C15.1F ledger carries the seeded cash
* News event queued

## 24. Post-rollover save/load golden

`c15_1g_post_rollover_save_load_round_trip` — apply a rollover,
serialise the `World` and the finance ledger, deserialise, and
assert semantic equality on both `person_news_mailboxes` and
`finance_ledger`. Confirms C15.1E and C15.1F are round-trip
safe under real `serde_json`.

## 25. Tests

Seven `c15_1g_*` tests, all green:

| Test | What it proves |
| ---- | -------------- |
| `snapshot_empty_diffs_zero` | Self-diff of empty snapshot is 0/0/0/... across all layers |
| `snapshot_serde_round_trip` | JSON round-trip of a populated snapshot is exact |
| `diff_flags_call_order_divergence` | Comparator surfaces first ordering divergence with prose |
| `diff_flags_finance_divergence_by_club` | Comparator surfaces finance mismatches by club id |
| `production_end_to_end_relegation` | Full production path fills `WorldApplyReport` correctly |
| `post_rollover_save_load_round_trip` | C15.1E mailbox + C15.1F ledger survive serde |
| `snapshot_is_deterministic` | Two identical rollover runs produce identical snapshots (self-diff clean) |

Full cm-domain suite: **1737 passed, 4 pre-existing baseline
failures unchanged, +7 from C15.1F.**

## 26. Commit hash

`<filled in after commit>` on `gdi-renderer-port`.

## 27. Remaining non-core deviations

* **Awards / player rating engine** — not part of Traditional
  English year-end core.
* **Non-2001 schedule templates** — the fixture subsystem is
  already frozen; other-season templates land as their data
  arrives.
* **History ring physical representation** (mailbox ring
  overflow) — semantic parity, not byte-exact. See §21.
* **Unobserved branches** — Conference-fallback C14 gap
  (documented in C15.1D report §5.1) and the owner-refusal
  branch of the stadium-expansion transaction. Both are
  UNEXERCISED IN THE PORTED PATH; STATE-EXACT status is
  contingent on capture.
* **Other nations** — the year-end pipeline for
  non-English pyramids is out of Traditional-English scope by
  design; each nation's year-end code path can freeze
  independently when its own capture is run.
* **RNG snapshot waypoints** — pending capture-side evidence
  that they need to be modelled in `YearEndSnapshot`.

## 28. Traditional English core FROZEN? — **provisionally yes**

At the ported-semantics layer: **YES**. Every World-mapping
required by year-end rollover is resolved (C15.1A–F), every
production entry point runs the same canonical pipeline, and
the deterministic snapshot golden holds across repeat runs and
save/load round-trips.

At the byte-exact STATE-EXACT layer: **PENDING the user's
Frida capture** — the capture harness and analyser are
shipped; running them and diffing against
`YearEndSnapshot::from_apply(...)` output is the one remaining
step. When that returns zero divergences, the freeze upgrades
from PROVISIONAL to STATE-EXACT for the observed branches.

Once the user has run the capture and merged its findings, the
tranche's next natural continuation is: awards engine
(deferred at C15.1E), or C15.2 (other nations' year-end
pipelines).
