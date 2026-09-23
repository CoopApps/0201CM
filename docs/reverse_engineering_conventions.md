# Reverse-engineering conventions & the GDI function registry

This document defines how recovered executable knowledge is permanently linked to
the Rust port. The authoritative bridge is the **GDI function registry**.

> **Location note.** The spec asked for `reports/gdi_function_registry.md`, but
> `/reports/` is gitignored in this repo (working scratch). The *committed*
> registry therefore lives under **`docs/gdi_registry/`**. The `tools/gdi_registry/`
> builder regenerates it.

## Files

| path | tracked | what |
|---|---|---|
| `docs/gdi_registry/gdi_function_registry.md` | ✅ | human-readable registry (generated) |
| `docs/gdi_registry/gdi_function_registry.csv` | ✅ | machine-readable registry (generated) |
| `docs/gdi_registry/gdi_function_registry.json` | ✅ | same, JSON (generated) |
| `docs/gdi_registry/gdi_function_coverage.md` | ✅ | coverage counts (generated) |
| `docs/gdi_registry/gdi_globals.md` | ✅ | executable global data symbols (generated) |
| `tools/gdi_registry/build_registry.py` | ✅ | regenerates all of the above |
| `tools/gdi_registry/query.py` | ✅ | address→Rust / Rust→address lookup |
| `tools/gdi_registry/validate.py` | ✅ | integrity checks: valid status; PORTED_* has a Rust link; NOT_YET not reachable; **curated `rust_symbol` still exists in its file (drift)**; **every in-code `// GDI-REG:` tag matches a registry row** |
| `tools/gdi_registry/inject_tags.py` | ✅ | idempotently writes `// GDI-REG: <addr> <status>` at the Rust item for each curated row (skips ambiguous/absent identifiers) so the link lives in the code |
| `tools/gdi_registry/data/*.csv` | ✅ | **curated** source rows (hand-authored) |

The builder merges three sources: (1) `functions.json` + the carver atlas
(`D:/cm0102-carve`, external — build-time only; the generated files are committed
so the registry is usable without it), (2) a live grep of `crates/**/*.rs` for
`FUN_`/`sub_` citations, and (3) the curated `data/*.csv` overlays. Curated status
always wins over auto-derived.

## Row schema (registry CSV/JSON)

```
dd_va            DirectDraw cm0102.exe virtual address (canonical key, e.g. 0x00669340)
gdi_va           GDI build address if known (VA deltas are non-uniform — see gdi-vs-directdraw-builds)
source_file      decompile .cpp (from the atlas string_hits / file attribution)
symbol           current Ghidra name (FUN_xxxxxxxx / sub_xxxxxxxx)
semantic_name    canonical human name of the function's role
subsystem        coarse subsystem tag
status           one primary status from the taxonomy below
rust_file        Rust file(s) implementing / representing it (";"-joined)
rust_symbol      Rust fn/const/type(s) (";"-joined)
reachable        production reachability: YES | NO | TEST_ONLY | INDIRECT | BLOCKED
relevance        observable relevance (";"-joined): PLAYER_VISIBLE | SIMULATION_RELEVANT |
                 PERSISTENCE_RELEVANT | UI_RENDERING | IMPLEMENTATION_ONLY
confidence       BYTE_EXACT | STATE_EXACT | BEHAVIOURALLY_EXACT | STRUCTURALLY_VERIFIED |
                 PARTIAL | HYPOTHESIS | UNVERIFIED
evidence         report/memory path(s) or capture id backing the classification
notes            free text
superseded_prev  previous (wrong) interpretation, if any
superseded_why   why it was superseded
```

## Status taxonomy (exactly one per function)

| status | meaning |
|---|---|
| `PORTED_EXACT` | observable semantics reproduced exactly, backed by executable evidence |
| `PORTED_BEHAVIOURAL` | Rust architecture differs; observable contract reproduced |
| `PORTED_PARTIAL` | some behaviour present; important executable semantics still absent |
| `REPLACED_BY_RUST` | exists due to old impl constraints (container growth, manual memory, pointer plumbing, file enumeration); Rust replaces it with a cleaner mechanism producing the same observable result |
| `NOT_YET_PORTED` | genuine game-logic gap |
| `BLOCKED_DEPENDENCY` | semantics known but cannot go live until another recovered subsystem exists |
| `FOREIGN_BREADTH` | valid game logic, outside the current England-first scope |
| `UI_GDI_DOMAIN` | GDI/widget/render machinery whose observable result is reproduced by `cm-render`, not function-for-function |
| `OUT_OF_SCOPE` | deliberate architectural replacement where exact internal impl is not required |
| `NON_USEFUL` | compiler/runtime plumbing (ctors/dtors/assignment/CRT thunks/`__ftol`/container plumbing/Win32 wrappers/static-init) |
| `DEAD_OR_UNREACHABLE` | executable function not part of the shipped/live path we reproduce |
| `UNKNOWN` | investigated insufficiently to classify (`notes` should say why, e.g. `NO_DECOMPILE`) |

Never use vague labels ("done", "probably covered", "not needed").

`REPLACED_BY_RUST` / `OUT_OF_SCOPE` / `UI_GDI_DOMAIN` are **not** licences for
behaviour to differ: the row's `notes` must state the observable contract Rust
preserves. Where observable parity is incomplete, say so and use
`confidence: PARTIAL`/`APPROXIMATE`.

## The four-way "untouched useful" distinction (permanent)

A function the port does not cite is **not** automatically "missing game logic".
It is one of: (1) covered-but-uncited → cite it; (2) NON_USEFUL plumbing; (3)
deliberately replaced / OUT_OF_SCOPE / UI_GDI_DOMAIN; (4) a genuine gap
(NOT_YET_PORTED / PORTED_PARTIAL / BLOCKED_DEPENDENCY / FOREIGN_BREADTH). Coverage
reports must not count (1)–(3) as unported game logic.

## Source annotation convention (in Rust)

Where Rust reproduces executable behaviour, add a concise doc comment — provenance
only, archaeology stays in the registry/reports:

```rust
/// GDI: 0x00669340 (`comp/fixture.cpp`) — PORTED_BEHAVIOURAL
/// Fixture matrix seed builder; ordering + RNG preserved.
/// See: docs/gdi_registry/gdi_function_registry.md
```

Durable machine-greppable tag (one convention, used consistently):

```rust
// GDI-REG: 00669340 PORTED_EXACT
```

When one Rust fn replaces several tiny helpers, or one exe fn decomposes into
several Rust fns, list them all in the comment and the registry (`;`-joined).
Do not create one-function-per-address wrappers just for bookkeeping. Do not add
`// TODO port this` without a registry row.

## Major-module provenance header

Top of a heavily reverse-engineered module:

```rust
//! Executable provenance
//! GDI functions represented here:
//!   0x00668450 outer fixture driver · 0x00669340 matrix seed · 0x0066B900 perturb
//! Full mapping: docs/gdi_registry/gdi_function_registry.md
```

## Contributor rules — when you port or investigate an executable function

1. add/update the registry row (a `data/*.csv` overlay line)
2. cite the DD address (and GDI address if known)
3. record the semantic name
4. record the Rust implementation (file + symbol)
5. record status (taxonomy above)
6. record confidence
7. record evidence (report/memory/capture)
8. add the concise source annotation when Rust code exists
9. update superseded interpretations if you corrected a prior belief
10. regenerate: `python tools/gdi_registry/build_registry.py` and run
    `python tools/gdi_registry/validate.py`

Keep source readable — detailed archaeology belongs in the registry and subsystem
reports, not inline.
