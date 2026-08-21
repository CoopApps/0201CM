# cm0102-rs architecture

This repository is the clean-room rewrite of CM0102, driven by code-derived facts
from `D:/cm0102-carve`. The carve decides what is true; Rust code only implements
what has already been verified or is explicitly marked inferred.

## Rules

- Read the carve first. If a subsystem is still `UNKNOWN`, we do not invent it in Rust.
- Keep provenance close to code. Every subsystem should cite the carve address or document that justified it.
- Separate verified emulation from new features. We need a faithful core before we add product changes.
- Prefer vertical slices. A runnable asset loader or replayable RNG is better than a half-mapped mega-module.

## Runtime shape

The binary gives us a stable execution order:

1. Boot and load data.
2. Enter the main tick loop.
3. Advance simulation from that loop.
4. Save and load sectioned state.

The rewrite should mirror that with explicit Rust boundaries:

- `cm-app`
  The executable shell. Owns config, install discovery, save paths, feature flags, and eventually the game loop.
- `cm-data`
  Base data and save containers. Reads `index.dat`, fixed-record `.dat` files, and later `.sav` sections.
- `cm-events`
  Match commentary and event vocabulary from `events_*.cfg`.
- `cm-rng`
  Bit-exact RNG compatibility for deterministic replay and differential testing.
- future `cm-engine`
  Match setup, per-player slot state, event dispatch, and match resolution.
- future `cm-sim`
  Day advancement, competitions, transfers, contracts, finances, and long-horizon world state.
- future `cm-ui`
  Presentation only. UI should depend on engine and sim state, not own business rules.

## Near-term implementation order

1. Make the verified slices usable against a real install.
2. Add save-container reading so we can inspect `.sav` sections without touching engine logic.
3. Lift more record layouts from the carve and expose typed readers only where field meaning is proven.
4. Introduce match-engine structs once the carve closes enough of the slot setup and rating accumulator.
5. Add new features after the core data and simulation contracts stop moving.

## Updated features strategy

New features should sit above stable domain APIs, not inside reverse-engineered parsing code.
That keeps us free to modernize the product while preserving a trustworthy compatibility layer.

Good early feature candidates:

- richer data inspection and editor tooling
- deterministic replay and diff tooling against the original exe
- mod packs expressed as structured patches instead of hex edits
- modern UX around saves, scouting, filtering, and analytics

## What this repository should optimize for

- correctness first on verified subsystems
- explicit unknowns instead of implied certainty
- small testable crates
- deterministic behavior where the original game was deterministic
