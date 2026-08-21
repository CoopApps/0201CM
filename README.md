# cm0102-rs - clean-room CM0102 reimplementation

A Rust workspace that reimplements the parts of Championship Manager 01/02 we have
verified from `D:/cm0102-carve`.

This is a clean-room project: we rebuild behavior and file formats from verified
reverse-engineering notes, not by copying the original code.

Status: the Rust side now has a real owned world database. It can import from a
live CM0102 install once, write a split Rust-owned database directory, load that
database without reading the old install, edit structured reference rows, create
and tick a Rust-native headless save shell, record backend mutation frontiers,
carry code-derived match/competition/transfer/news boundary maps,
define parity-gated contracts for the four exact gameplay mutators,
emit a machine-readable backend implementation plan,
and export `.dat` files only as a compatibility format. The exact gameplay
mutators are still ahead, but the data layer is no longer just a binary reader.

## Crates

| Crate | What | State |
|---|---|---|
| `cm-rng` | MSVC `rand()` plus the match RNG | Verified algorithm |
| `cm-data` | `.dat` containers, `index.dat`, save sections, typed reference tables | Real install loaders in place |
| `cm-domain` | Owned Rust world model plus snapshot/database I/O and audit support | Can load, serialize, re-read, and export a Rust DB |
| `cm-events` | `events_*.cfg` match commentary | Verified format |
| `cm-app` | Small executable baseline over a real install | Summary, export, inspect, and audit commands |

## Modern delivery shell

`D:/cm0102-rs/godot` is the first Godot 4 delivery shell. It follows the same
simple project shape as `D:/scoobydoo/godot`: a main scene plus scripts, with
autoloaded state. The important rule is that Godot is only the UI/input/assets
layer. The Rust API remains the source of truth for database records, saves,
headless ticks, parity status, and promotion gates.

```text
cargo run -p cm-app -- serve-rust-db D:/cm0102-rs/rust-db 8770
```

Then open `D:/cm0102-rs/godot` in Godot. The starter scene reads
`/api/promotion-control-room-cached`, `/api/backend-acceptance`,
`/api/runtime-save`, `/api/tables`, and individual `/api/table/<path>` datasets.
It can also call the Rust runtime/headless tick endpoints from buttons.

## Build and run

The workspace uses the `windows-gnu` toolchain pinned in
[`rust-toolchain.toml`](D:/cm0102-rs/rust-toolchain.toml:1).

```text
cargo test --workspace
cargo run -p cm-data --example tables -- D:/cm0102/Data
cargo run -p cm-data --example typed_tables -- D:/cm0102/Data
cargo run -p cm-domain --example export_world -- D:/cm0102 D:/cm0102-rs/world.json
cargo run -p cm-app -- D:/cm0102
cargo run -p cm-app -- export-world D:/cm0102 D:/cm0102-rs/world.json
cargo run -p cm-app -- inspect-world D:/cm0102-rs/world.json
cargo run -p cm-app -- audit-world D:/cm0102 D:/cm0102-rs/world.json
cargo run -p cm-app -- init-rust-db D:/cm0102 D:/cm0102-rs/rust-db
cargo run -p cm-app -- inspect-rust-db D:/cm0102-rs/rust-db
cargo run -p cm-app -- audit-rust-db D:/cm0102-rs/rust-db
cargo run -p cm-app -- backend-report D:/cm0102-rs/rust-db D:/cm0102-rs/reports/backend_readiness.json
cargo run -p cm-app -- backend-acceptance D:/cm0102-rs/rust-db D:/cm0102-rs/reports/backend_acceptance.json
cargo run -p cm-app -- exact-remake-report D:/cm0102-rs/rust-db D:/cm0102/cm0102.exe D:/cm0102-rs/reports/exact_remake.json
cargo run -p cm-app -- init-gameplay-parity-traces D:/cm0102-rs/rust-db D:/cm0102-rs/reports/parity_traces
cargo run -p cm-app -- export-rust-match-result-trace D:/cm0102-rs/rust-db D:/cm0102-rs/reports/parity_traces
cargo run -p cm-app -- export-rust-gameplay-candidate-traces D:/cm0102-rs/rust-db D:/cm0102-rs/reports/parity_traces
cargo run -p cm-app -- export-original-capture-templates D:/cm0102-rs/reports/parity_traces D:/cm0102-rs/reports/original_capture_templates
cargo run -p cm-app -- original-capture-status D:/cm0102-rs/reports/original_capture_templates D:/cm0102-rs/reports/original_capture_status.json
cargo run -p cm-app -- export-original-capture-workbench D:/cm0102-rs/reports/original_capture_templates D:/cm0102-rs/reports/original_capture_workbench
cargo run -p cm-app -- export-promotion-control-room D:/cm0102-rs/rust-db D:/cm0102-rs/reports/promotion_control_room
cargo run -p cm-app -- export-todo-attack-board D:/cm0102-rs/rust-db D:/cm0102-rs/reports/todo_attack_board
cargo run -p cm-app -- gameplay-parity-report D:/cm0102-rs/rust-db D:/cm0102-rs/reports/parity_traces D:/cm0102-rs/reports/gameplay_parity.json
cargo run -p cm-app -- gameplay-promotion-report D:/cm0102-rs/rust-db D:/cm0102-rs/reports/parity_traces D:/cm0102-rs/reports/gameplay_promotion.json
cargo run -p cm-app -- export-gameplay-lift-workbench D:/cm0102-rs/rust-db D:/cm0102-rs/reports/lift_workbench
cargo run -p cm-app -- export-gameplay-capture-pack D:/cm0102-rs/reports/parity_traces D:/cm0102-rs/reports/capture_pack
cargo run -p cm-app -- sync-gameplay-mutator-contracts D:/cm0102-rs/saves/new_game.json D:/cm0102-rs/reports/parity_traces
cargo run -p cm-app -- rename-rust-db D:/cm0102-rs/rust-db references.cities 0 Gent
cargo run -p cm-app -- rename-rust-db D:/cm0102-rs/rust-db core.continents 0 Africa
cargo run -p cm-app -- set-rust-db-text D:/cm0102-rs/rust-db core.nations 0 short AFG
cargo run -p cm-app -- set-rust-db-text D:/cm0102-rs/rust-db references.club_competitions 0 short_name "First Division"
cargo run -p cm-app -- set-staff-type10 D:/cm0102-rs/rust-db 0 ca 0
cargo run -p cm-app -- set-staff-attribute D:/cm0102-rs/rust-db 0 0 0
cargo run -p cm-app -- export-rust-db-world D:/cm0102-rs/rust-db D:/cm0102-rs/world.json
cargo run -p cm-app -- export-rust-db-viewer D:/cm0102-rs/rust-db D:/cm0102-rs/world_viewer.json
cargo run -p cm-app -- export-rust-db-viewer-tables D:/cm0102-rs/rust-db D:/cm0102-rs/viewer_tables
cargo run -p cm-app -- run-headless-campaign D:/cm0102-rs/saves/new_game.json 30 10 D:/cm0102-rs/reports/headless_campaign.json
cargo run -p cm-app -- serve-rust-db D:/cm0102-rs/rust-db 8770
cargo run -p cm-app -- export-rust-db-data D:/cm0102-rs/rust-db D:/cm0102-rs/rust-db-export
```

## Rust database workflow

The long-term migration path is now:

```text
old CM0102 install -> init-rust-db -> rust-db/ -> editor/game systems -> optional .dat export
```

`rust-db/` is the canonical Rust-owned world database. It is split into stable
table files such as `core/nations.json`, `references/cities.json`,
`staff/type10.json`, and `schema.json`, so tools can work with individual tables
without reparsing one giant snapshot. The old `.dat` files are now treated as an
import source and compatibility export target, not the truth the remake should
run from.

For exact gameplay promotion, use `reports/capture_pack/` as the original-binary
capture checklist. Each subsystem now has row-level artifacts:
`row-capture-plan.csv`, `row-capture-plan.json`, `x32dbg-row-watch-plan.txt`, and
`capture-session-checklist.md`. Those files map every Rust candidate mutation to
the original `cm0102.exe` function/offset/watch group that must be captured
before parity can turn green.

For the browser viewer, prefer the table-by-table export in `viewer_tables/`.
Serve `D:/cm0102-rs` and open `world_viewer.html`; the quick-load button first
tries the live Rust DB API at `http://127.0.0.1:8770/api/tables`, then falls back
to `viewer_tables/index.json`. `world_viewer.json` is still available as a
single-file fallback. The full `world.json` snapshot keeps machine-oriented binary
payloads and is much larger; it is not the browser viewer format.

The live API also exposes runtime backend state:

```text
GET http://127.0.0.1:8770/api/backend
GET http://127.0.0.1:8770/api/backend-acceptance
GET http://127.0.0.1:8770/api/exact-remake
GET http://127.0.0.1:8770/api/gameplay-parity
GET http://127.0.0.1:8770/api/promotion-control-room
GET http://127.0.0.1:8770/promotion-control-room
GET http://127.0.0.1:8770/api/promotion-control-room-cached
GET http://127.0.0.1:8770/promotion-control-room-cached
GET http://127.0.0.1:8770/api/original-capture-status
GET http://127.0.0.1:8770/api/original-capture-workbench
GET http://127.0.0.1:8770/original-capture-workbench
POST http://127.0.0.1:8770/api/original-capture/row
POST http://127.0.0.1:8770/api/original-capture/import-system
POST http://127.0.0.1:8770/api/original-capture/import-ready
GET http://127.0.0.1:8770/api/runtime-save/backend
GET http://127.0.0.1:8770/api/runtime-save/mutations
POST http://127.0.0.1:8770/api/headless/campaign
```

## What works now

- `cm-rng` reproduces the canonical MSVC `rand()` sequence.
- `cm-data` reads the verified fixed-record `.dat` files and the `index.dat` type map.
- `cm-data` parses the `.sav` section directory.
- `cm-data` bulk-loads a large reference slice of the install:
  cities, officials, names, stadiums, competitions, and history tables.
- `cm-domain` imports that data into an owned `World`, exports pretty JSON, and can load it back.
- `cm-domain` writes and reads a split Rust-owned database directory.
- `cm-domain` audits a Rust DB against its own manifest, schema, and verified core record sizes.
- `cm-domain` can create a Rust-native runtime save and run the verified three-phase
  CM day shell headlessly without reading `.dat` files.
- `cm-domain` records backend system mutation attempts for match results,
  competition state, transfers/contracts, and news/inbox during each headless tick.
- `cm-domain` carries code-derived backend boundary maps for fixture score writes,
  fixture notification state, transfer/contract processing, and news/inbox
  paired-event and queue handling.
- `cm-domain` carries a code-derived match-engine lift map for setup, step
  controller, phase/possession controller, period transition writer, and event
  queue boundaries.
- `cm-domain` carries a match-result mutator install plan that names the required
  original/Rust trace coverage and promotion rule before `implementation_present`
  can be enabled.
- `cm-domain` defines parity-gated mutator contracts for match results,
  competition state, transfers/contracts, and news/inbox, binding each to a phase,
  boundary map, trace file, and implementation hook.
- `cm-domain` defines install plans for all four exact mutator targets, naming
  required original/Rust coverage, required functions, promotion rules, and
  safety rules before any exact gameplay hook can be enabled.
- `cm-domain` carries promotion gates for all four exact gameplay targets, so a
  Rust mutator cannot be treated as live until original-binary capture, Rust
  mutation traces, exact ordered parity, and manual review blockers are cleared.
- `cm-domain` carries a gameplay lift workbench for the priority original-code
  functions that still block exact mutator bodies, including carve/decompile
  commands, required claims, trace files, and promotion targets.
- `cm-domain` carries code-derived match-result claims extracted from targeted
  decompile artifacts for fixture score writes, transition events, final result
  events, and event queue stride. These are evidence claims, not live gameplay
  mutations yet.
- `cm-app` can now export a Rust-side match-result parity trace candidate from
  those code-derived maps, producing fixture/event mutation rows ready to compare
  with the original `cm0102.exe` capture.
- `cm-app` can also export Rust-side candidate parity traces for all four
  gameplay systems, giving each subsystem a concrete Rust mutation proposal while
  keeping promotion blocked until original-binary equality is proven.
- `cm-app` reports original-capture readiness across those templates, including
  filled rows, placeholder rows, schema readiness, coverage blockers, and import
  readiness before a capture is allowed to affect parity traces.
- `cm-app` exports an original-capture workbench with CSVs and a browser
  dashboard at `reports/original_capture_workbench/dashboard.html`.
- `cm-domain` also carries targeted-decompile claims for competition fixture
  flags/cadence, transfer queue and contract-record strides, and news/inbox
  queue emission/removal offsets, each cited back to the original-binary carve.
- `cm-domain` defines disabled exact-mutator Rust skeletons for all four gameplay
  systems, so implementation bodies have named landing zones without mutating
  runtime state before parity promotion.
- The headless tick path dispatches gameplay attempts through those contracts and
  disabled exact-mutator skeleton entry points, then records contract status,
  trace file, boundary map, hook, parity gate, skeleton status, and zero emitted
  mutations in the mutation log.
- `cm-domain` emits a backend implementation plan that names each gameplay
  system's owned records, boundary map coverage, primary frontiers, missing lifts,
  and acceptance gate.
- `cm-domain` can run checkpointed headless campaigns that report days, phases,
  backend mutation attempts, and frontier blockers.
- Long headless campaigns retain the latest backend mutation entries while keeping
  total/dropped counters, so full-season validation does not grow saves unbounded.
- `cm-app` can initialize, inspect, audit, edit structured reference names, edit core text slots,
  edit competition long/short names, edit inferred `staff.type10` CA/PA/reputation fields,
  edit individual staff attribute bytes, export full machine snapshots, export compact
  browser snapshots, export table-by-table browser datasets, serve a live Rust DB JSON API,
  report backend readiness, and export compatibility `.dat` files from that database.
- `cm-app` loads a real install, reports the world summary, and audits snapshots against the install.

## Honest scope

- Strongest today: reference data, compatibility loading, and Rust-owned database I/O.
- Still partly inferred: many fixed record sizes and numeric slots beyond the safest
  text and identity fields.
- Editing support currently covers structured reference name tables, core table
  primary/secondary/short text slots, competition long/short names, inferred
  `staff.type10` CA/PA/reputation fields, and individual staff attribute bytes.
  Most numeric slots still need code-derived names before they should become named
  editor controls.
- Current backend gate: `backend-report` scores 80%. The Rust DB owns all known
  logical tables, the headless shell runs, and the backend mutation ledger is
  active. Match engine flow, all four exact mutator install gates, disabled Rust
  mutator skeletons, match result writes, competition fixture state,
  transfer/contract processing, and news/inbox emission boundaries are mapped
  from code, but exact match results, competition state, transfers/contracts, and
  news/inbox mutations are still frontier-only.
  `validate-runtime-simulation` also runs a 365-day full-year headless campaign gate.
- One-shot backend acceptance is available through `backend-acceptance`; it
  combines readiness, runtime validation, and the full-year campaign gate.
- One-shot exactness validation is available through `exact-remake-report`; it
  currently passes the Rust-foundation side but correctly fails one-for-one
  gameplay equivalence until the four exact gameplay mutators are implemented.
- Gameplay parity trace validation is available through `gameplay-parity-report`;
  it requires original-vs-Rust mutation traces for match results, competition
  state, transfers/contracts, and news/inbox before exactness can pass.
  `gameplay-promotion-report` reads the promotion gates plus those traces and
  explains why each exact mutator is still blocked or safe to enable.
  `export-gameplay-lift-workbench` writes the original-code lift queue into
  per-system command, claims, and decompile-artifact audit files for the next
  binary-derived implementation pass.
  `init-gameplay-parity-traces` writes pending trace templates from the current
  backend implementation plan without overwriting captured traces.
  `export-gameplay-capture-pack` expands those templates into per-system
  breakpoint, watched-write, Rust-hook, and quality-gate files.
  `sync-gameplay-mutator-contracts` updates a Rust save's mutator contract
  statuses from those traces. A contract only reaches `ParityVerified` when the
  trace passes and its trace declares `rust_implementation.present=true`; pending
  traces or verified original traces without an installed Rust mutator keep the
  contract disabled.
- Biggest remaining blocker: verified semantics for more `staff.dat`, club, nation,
  competition, and history fields, plus those living football systems built on top.

## Direction

The path to replacing the old `.dat` files is:

1. keep lifting more tables into owned Rust domain data
2. export, audit, and eventually edit that Rust-owned world directly
3. only keep binary import/export adapters at the edge
