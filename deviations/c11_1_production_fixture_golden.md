# C11.1 — production fixture golden deviation (open)

## What C11.1 delivered

Session `GameRng` bootstrap, explicit dispatch decision enum, hard error on roster-shape mismatch for 7/8/9/10/93, V4 negative-dispatch gate, 2001-only base-year policy, and P1 ordered-by-name match against captured GDI (`c11_1_p1_provenance`) — all clean.

## What is still open

Injecting the captured Prem `GameRngState` into `NewGameOptions.initial_game_rng_state`, then running the REAL production dispatch (`World::generate_new_game_season_with_rng`) against the shipped rust-db, does NOT yet produce byte-exact ordered fixtures vs the captured GDI insert trace.

Current diagnostic (see `tests/c11_1_production_fixture_golden.rs`):

* P1 ordered by name matches (verified separately in `c11_1_p1_provenance`).
* Captured Prem initial pool RNG state pins cleanly via `GameRng::from_state_snapshot`.
* Rust dispatches 7 → 8 → 9 → 10 → 93 through one shared RNG in exe boot order.
* Fisher-Yates output diverges: fixtures share the same club set for each round, but pairings differ starting from the first slot in every league.

## What is NOT the cause

* P1 order — verified 20/20 + 24/24 + 24/24 + 24/24 + 22/22 name matches.
* Initial RNG state — pinned via captured `GameRngState` for Prem.
* RNG continuity — the shared instance advances only through generated leagues.
* Stadium linkage for the sampled club — Rust-db Arsenal → stadium 2 → alt 119, matches captured exactly for that club.

## Likely cause (to investigate post-C11.1)

The archaeology helper `examples/five_league_diff.rs` reaches 0/N by (a) feeding captured LCG return values through the playback API and (b) building the resolver from the captured club_id list. Production uses:

* rust-db's shipped club_id numbering (different numeric ids than the running GDI process — the rust-db renumbering deviation);
* algorithmic RNG (`GameRng::from_state_snapshot` seeded from captured Prem initial state);
* a resolver built from rust-db stadium data.

The diff runner's 0/N result therefore does not directly translate. The perturb resolver's E2/E3 phases operate on club_ids (post-C10.7 fix — club-id keyed, not slot-keyed). Whether the rust-db renumbering interacts with the resolver's lookup keys in a way that produces different pairing decisions is the next thing to verify — likely a small mapping fix in `english_traditional::build_stadium_resolver` or its inputs.

## Not a regression

This is not a regression introduced by C11.1. C11 itself never asserted byte-exact production fixtures against captured GDI — only per-league fixture counts (380 / 552 / 552 / 552 / 462), which continue to pass. C11.1 added the necessary infrastructure (`GameRngState` injection through `NewGameOptions`) to eventually close this last gap; the archaeology has been narrowed to the stadium-graph / club-id-mapping interaction between rust-db and captured GDI.

## Scope decision

Kept in a diagnostic-mode test suite (`tests/c11_1_production_fixture_golden.rs`) that prints mismatch counts but does not fail the build. Freezing English fixture production per C11.1 point 15 waits on this being fully closed; C7/C8/C9 rollover work can proceed in parallel because it does NOT depend on production fixture identity being byte-exact — it depends on the dispatch shape, RNG ownership, and P1 provenance, all of which C11.1 established.
