# Rust fixture-pipeline architecture audit — 2026-09-13

**Purpose**: Before integrating recovered cm0102-gdi.exe fixture
behaviour into the Rust port, map the existing architecture so we
can choose the minimum, least invasive change that reproduces the
GDI spec.

**Result**: KEEP `exe_date.rs` + `HeadlessSeasonFixture` + `Vec` fixture
storage. ADD the missing walker/matrix/driver as functions and a
per-comp dispatch. Full detail below.

## 1. Workspace crates

| Crate | Responsibility |
|-------|----------------|
| `cm-rng` | External-table RNG primitives (`CrtRand`, `MatchRng`). |
| `cm-data` | Raw `.dat` readers. |
| `cm-events` | Event/message types. |
| **`cm-domain`** | World, RuntimeSaveGame, competitions, fixtures. Owns fixture generation. |
| `cm-app` | Headless runner. |
| `cm-gui` | Legacy GUI shell. |
| `cm-render`, `cm-widget`, `cm-ui-app` | Faithful UI reproduction. |
| `cm-db`, `cm-import` | `.dat` → native db. |

Fixture generation lives in `cm-domain`.

## 2. Existing domain model

| Concept | Type | File:line |
|---------|------|-----------|
| Competition | `DomainCompetition` | `crates/cm-domain/src/lib.rs:394` |
| Fixture | `HeadlessSeasonFixture` | `crates/cm-domain/src/lib.rs:12249` |
| Fixture status | `HeadlessFixtureStatus` | `crates/cm-domain/src/lib.rs:12342` |
| Calendar date | `GameDate` | `crates/cm-domain/src/lib.rs:2564` |
| Packed date | `CmPackedDate` (1-indexed doy) | `crates/cm-domain/src/lib.rs:12215`, impl `:21080` |
| Season start (per country) | `SeasonStart` | `crates/cm-domain/src/league_calendar.rs:24` |
| RNG (byte-exact, embedded pool) | `game_rng::GameRng` | `crates/cm-domain/src/game_rng.rs:34` |
| Fixture storage | `HeadlessSeasonState.fixtures: Vec<HeadlessSeasonFixture>` | `crates/cm-domain/src/lib.rs:12240` |
| Fixture query | `World::club_fixtures_for`, `league_table_for`, `latest_scores` | all iterate `Vec` |

## 3. Fixture-generation call graph

```
cm-app::main
  → World::new_game_from_rust_db (lib.rs:16095)
      → league_calendar::earliest_start                 [→ save.date, NOT per-fixture]
      → generate_new_game_season (lib.rs:18289)
          for each comp with is_headless_league_like_competition
              AND id NOT IN LEAGUES_BUILT_BY_DEDICATED_ENGINES
              AND members.len() ∈ [2, 30]:
            start = league_calendar::season_start(country).resolve(base_year)
            generated = generate_double_round_robin(comp, members, row, &start)
            [PHASE-1 dispatch] if comp.id == 9 && members.len() == 24:
              overlay 46 exact dates from generate_eng_second_dates
              (patches fixture.date + fixture.source; pair-order unchanged)
            fixtures.extend(generated)
      save.season.fixtures = fixtures
```

## 4. Compatibility assessment

| GDI mechanism | Rust equivalent | Verdict | Notes |
|---------------|-----------------|---------|-------|
| `pack_date` (0x00533d80) | `exe_date::pack_date` | **KEEP** | byte-exact, tested |
| `apply_flag_snap` (0x005340e0) | `exe_date::apply_flag_snap` | **KEEP** | 42-case runtime sweep passes |
| 65-byte round record | `exe_date::write_round_record` + `write_slot` | **KEEP** | full 2990-byte capture match |
| `build_eng_second_schedule` | same | **KEEP** | authoritative GDI-capture test |
| Walker (0x0066ee40) | **added** as `eng_second_fixtures::walker_step` | **ADD** | required by driver |
| Matrix seeder (0x00669340) | not yet ported | **ADD (stub for now)** | small — 399 B |
| Matrix perturb (0x0066b900) | not yet ported | **ADD (stub for now)** | larger — 2481 B |
| Round-robin driver (0x00668450) | Berger stub `generate_double_round_robin` | **REPLACE for comp id 9** (after matrix seeder + runtime pair capture) |
| 79-byte TFixture | `HeadlessSeasonFixture` | **KEEP** | different repr, behaviour equivalent |
| TFixList container | `Vec<HeadlessSeasonFixture>` | **KEEP** | different repr, behaviour equivalent |
| RNG (0x008fbe20) | `game_rng::GameRng` | **KEEP** | byte-exact port with embedded pool |

## 5. Integration boundary (chosen)

- **Add** `crates/cm-domain/src/eng_second_fixtures.rs` — home for walker
  + matrix stubs + `generate_eng_second_dates` + eng-second constants.
- **Add** `pub mod eng_second_fixtures;` to lib.rs (one line).
- **Modify** the `generate_new_game_season` loop at `lib.rs:18354` —
  after generating fixtures, if comp.id == 9 and 24 clubs, overlay
  exact dates.
- **Delete** the orphan `crates/cm-domain/src/cm0102_gdi/` skeleton.

Total git-visible change: 3 files touched (+ 1 new report). No public
API changed. `HeadlessSeasonFixture` schema unchanged. Every existing
consumer (screens, save, standings, match pipeline) still works.

## 6. Generalisation path

`generate_eng_second_dates(base_year)` — the only competition-specific
public function. It internally consumes the generic
`build_eng_second_schedule` + `ENG_SECOND_2001_TEMPLATE`. To add
another English division:

1. Capture that comp's 2990-byte buffer (or its round count × 65).
2. Extract the `(day, month, day_off, flag, type)` tuple sequence
   into a new `ENG_XXX_2001_TEMPLATE` alongside `ENG_SECOND_2001_TEMPLATE`.
3. Add `generate_eng_xxx_dates(base_year)` alongside
   `generate_eng_second_dates(base_year)`.
4. Extend the dispatch match in `lib.rs:18354` with the new comp id.

Once four or five leagues share the pattern, extract a
`CompetitionScheduleTemplate` trait or a `&[(day, month, day_off, flag,
type)]` registry keyed by comp id. Do not create the registry before
that evidence lands.

## 7. Non-goals

- No new competition abstraction.
- No `CompetitionKind` enum.
- No new `Calendar` or `Fixture` types.
- No `HeadlessSeasonFixture` schema change.

## References

- `reports/fixture_disasm/gdi_va_map.json` — GDI address map
- `reports/fixture_disasm/GDI_CORRECTION_REPORT.md` — GDI vs DirectDraw
- `reports/fixture_disasm/FUN_00668890_DECODE.md` — driver reconstruction (still valid modulo relocations per the GDI overlay)
- `reports/fixture_disasm/TEAM_PAIRING_REPORT.md` — fixture pipeline overview
- `reports/fixture_disasm/INTEGRATION_PHASE_1.md` — this phase's changes
