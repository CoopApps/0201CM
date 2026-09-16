# C15.1G.1 — Year-End Differential CLI

**Branch.** `gdi-renderer-port`.
**Predecessor.** C15.1G (`c54453a`).
**Scope.** Ship the one-command Rust-side differential runner
that consumes a GDI capture snapshot, runs the real production
year-end pipeline, and prints/exit-codes a layer-by-layer
verdict.

## 1. Binary + path

`crates/cm-import/src/bin/c15_year_end_diff.rs`, wired into
`crates/cm-import/Cargo.toml` as
`[[bin]] name = "c15_year_end_diff"`.

## 2. Invocation syntax

```bash
cargo run -p cm-import --bin c15_year_end_diff -- \
    --gdi <snapshot.json> \
    [--rust-input <config.json>] \
    [--json-output <verdict.json>] \
    [--verbose]
```

## 3. Required inputs

* `--gdi <path>` (required) — a `YearEndSnapshot`-shaped JSON
  produced by `tools/c15_1g/analyse_gdi_capture.py`.
* `--rust-input <path>` (optional, reserved) — will consume an
  `AnnualRolloverInputConfig` mirroring the GDI capture's
  pre-rollover state. **The schema is deliberately not yet
  defined**; the CLI refuses this flag with a clear error
  pointing the user at the demo path. When C15.1G.2 lands, the
  same CLI accepts real inputs.
* `--json-output <path>` (optional) — writes the same verdict
  as machine-readable JSON.
* `--verbose` (optional) — prints per-layer mismatch counts on
  failures.

When neither `--rust-input` nor a similar knob is present, the
CLI runs the built-in Traditional English demo pre-rollover
state through `apply_report_to_world_parts` — this is the
production path, but on synthetic data, and the output freeze
verdict is explicitly labelled `PASS_DEMO_ONLY`.

## 4. Production Rust path invoked

The CLI drives:

```
apply_report_to_world_parts(&mut world, ..., &date, 0, &report)
    → YearEndSnapshot::from_apply(year, &world, &ledger, &applier)
```

Same code path as `RuntimeSaveGame::apply_report_to_world`. No
new helpers; no synthetic-only entry points.

## 5. Top-level layer report

Printed as a compact fixed-width table:

```
  Call order .................... PASS|FAIL
  Club movement ................. PASS|FAIL
  Contract clauses .............. PASS|FAIL
  Squad position ................ PASS|FAIL
  Person history/news ........... PASS|FAIL
  Stadium ....................... PASS|FAIL
  Finance ....................... PASS|FAIL
  News .......................... PASS|FAIL
  RNG ........................... NOT CAPTURED
```

RNG is not hidden behind PASS — it appears with an explicit
`NOT CAPTURED` label until the capture schema adds RNG
waypoints.

## 6. Per-edge movement report

Prints a section:

```
  Per-edge movement:
        Premier ↔ First             gdi=  X rust=  Y mismatch=Z
        First ↔ Second              gdi=  X rust=  Y mismatch=Z
        Second ↔ Third              gdi=  X rust=  Y mismatch=Z
        Third ↔ Conference          gdi=  X rust=  Y mismatch=Z
        Conference feeder           gdi=  X rust=  Y mismatch=Z
```

When the GDI analyser doesn't yet emit `ClubMoved` events, the
CLI prints
`(edge breakdown NOT AVAILABLE — GDI analyser did not emit ClubMoved events)`
instead of fabricating zeros. Extending the analyser is
tracked as follow-up.

## 7. First-divergence formatting

For each `FAIL` layer, one line under the verdict:

```
        first divergence: <compact context>
```

Examples the diff comparator produces (verbatim):

```
first divergence: index 0: gdi=<end>, rust="ContractClauseWrite"
first divergence: key 100: gdi=<missing>, rust=FinanceSnapshotEntry { cash: 30000000, ... }
first divergence: index 2: gdi=ContractClauseWrite { person_id: 555, offset: 31, ... }, rust=SquadPositionWrite { ... }
```

## 8. Verbose / full-diff mode

`--verbose` adds `mismatch count: N` under any FAILing layer.
Full per-item dumping is not the default — the whole-run
`--json-output` file is the source of truth for the complete
diff.

## 9. JSON output mode

`--json-output <path>` writes:

```json
{
  "call_order":    { "layer": "Call order", "status": "PASS|FAIL", "mismatches": N, "first_divergence": ... },
  "club_movement": { … },
  "contracts":     { … },
  "squad":         { … },
  "history":       { … },
  "stadium":       { … },
  "finance":       { … },
  "news":          { … },
  "rng":           { "status": "NOT CAPTURED", … },
  "freeze":        { "status": "PASS|PASS_DEMO_ONLY|PASS_FOR_OBSERVED|FAIL",
                     "first_failing_layer": "…",
                     "unobserved_branches": [ … ] },
  "rust_source":   "demo|user_input",
  "representation_deviations": [ … ]
}
```

This is intended to be the CI artefact once the real capture
lands — a commit or PR can pin against this JSON.

## 10. Exit codes

* `0` — every captured semantic layer clean.
* `1` — semantic mismatch on one or more layers.
* `2` — invalid / missing input, setup error, or `--rust-input`
  used before its schema lands.

## 11. RNG not-captured handling

The RNG line always prints `NOT CAPTURED` verbatim, never
`PASS`. The verdict's `rng` JSON entry mirrors that. When a
future capture schema adds RNG waypoints, the CLI reads them
without changing the surrounding contract.

## 12. Representation-deviation handling

Printed under a fixed `Known representation deviations (not
compared)` section, always:

* history mailbox: DOB-age routing not modelled
* history mailbox: 100-entry ring overwrite not modelled
* history record: numbered slots 1..=4 held at 0
* history record: `+0x04` severity byte held at 0
* finance record: 24+ additional accumulator DWORDs deferred

These are documented facts, not layer verdicts.

## 13. Unobserved-branch handling

Printed under `Unobserved branches (state-exactness NOT
claimed)`:

* Conference fallback (`FUN_0055EA00` path)
* Owner-refusal stadium path (`FUN_00583FC0` refuse branch)
* Nation-affiliation identity clause (`FUN_00843970` second
  identity check)

These never contribute to PASS/FAIL; they surface as
free-standing metadata in the JSON output's `unobserved_branches`
array too.

## 14. Freeze verdict rules

* Any layer `FAIL` → `CORE FREEZE: FAIL`; JSON status `"FAIL"`;
  `first_failing_layer` set.
* Demo run + all captured layers pass → `CORE FREEZE: PASS
  (demo run — synthetic Rust input, NOT a final freeze
  differential)`; JSON status `"PASS_DEMO_ONLY"`.
* Real Rust input (once wired) + all captured layers pass →
  `CORE FREEZE: PASS FOR OBSERVED N→N+1 PATH`; JSON status
  `"PASS_FOR_OBSERVED"`, with `unobserved_branches` still
  listed as caveats.

## 15. Tests

4 unit tests in the bin:

| Test | What it proves |
| ---- | -------------- |
| `identical_snapshots_diff_clean` | Self-diff of a populated snapshot is 0/0/0 across every layer |
| `one_club_mismatch_flags_first_divergence` | Changing a single finance entry surfaces exactly one mismatch with prose |
| `representation_only_difference_is_not_a_semantic_mismatch` | Changing `news_id` on one side alone does NOT count against `history` — pins the semantic-vs-representation boundary |
| `demo_rollover_produces_deterministic_snapshot` | Two invocations of the demo pipeline produce identical snapshots (self-diff clean) — the CLI's `--demo` path is deterministic |

All 4 pass. Full cm-domain lib still 1737 pass / 4 pre-existing
baseline failures.

Smoke-run (with a stub GDI JSON) exercises every code path and
returns exit=1 on mismatch, exit=0 on clean.

## 16. Commit hash

`<filled in after commit>` on `gdi-renderer-port`.

## 17. User's next step

After running the GDI capture:

```powershell
# 1. Analyse the raw capture into a YearEndSnapshot JSON.
python tools\c15_1g\analyse_gdi_capture.py `
    fixtures\gdi_captures\year_end_2001_02.jsonl `
    > fixtures\gdi_captures\year_end_2001_02.gdi.json

# 2. Run the differential (demo Rust side; smoke-checks tooling).
cargo run -p cm-import --bin c15_year_end_diff -- `
    --gdi fixtures\gdi_captures\year_end_2001_02.gdi.json `
    --json-output fixtures\gdi_captures\year_end_2001_02.verdict.json

# Exit code: 0 = clean, 1 = mismatch, 2 = setup error.
```

The demo verdict is a smoke check of the CLI itself — expect
divergences because the demo state doesn't match your GDI
capture. The final freeze differential lands when the
`--rust-input` schema (C15.1G.2) accepts an
`AnnualRolloverInputConfig` mirroring your capture, at which
point the same CLI produces a real freeze verdict.

## 18. No final freeze claim

This commit only makes the final comparison **executable**. The
STATE-EXACT freeze happens after:

1. The user runs the Frida capture from C15.1G to produce
   `year_end_2001_02.jsonl`.
2. The analyser produces `year_end_2001_02.gdi.json`.
3. `--rust-input` accepts real pre-rollover state (C15.1G.2).
4. The CLI reports zero semantic differences.

Until then, the freeze remains **PROVISIONAL** at the
ported-semantics layer, as documented in the C15.1G report §28.
