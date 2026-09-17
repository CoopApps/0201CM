# Next Work Plan — observational-equivalence programme

**Date:** 2026-09-17
**Supersedes:** the implicit "byte-exact internals" goal.

## 0. The fidelity contract (governs everything below)

The target is **observational equivalence**: the Rust port should look,
respond and play like CM 01/02 GDI from the player's seat. Internals may
differ freely where the difference is invisible.

**Free to differ:** `Vec` instead of C arrays, IDs instead of
reconstructed pointers, whole-DB-in-memory instead of paging, typed
structs instead of packed blobs, `i64` where the exe split `i32`, richer
indices, modern serialization, parallelism that preserves observable
ordering.

**Not free to differ:** RNG draw order and count where it affects
outcomes, decision gates, thresholds, formulas, sort order and
tiebreaks, promotion/relegation rules, fixture structure, match
outcomes, financial outcomes, contract effects, season lifecycle, screen
geometry, fonts, colours, draw order, clipping, control behaviour,
player-visible data.

> Do not reproduce **how** CM 01/02 was programmed. Reproduce **what** it
> does.

**Consequence for archaeology:** stop decoding internal plumbing once the
observable contract is known. If the exe walks
`Club* → person slot → staff table → contract record → identity gate`
and the proven observable rule is *"for every qualifying employee of the
club, if the clause is armed, transition it"*, implement that directly
against our indices. Decode further only to answer an observable
question.

---

## 1. Current measured daily tick cost and top cost centres

*Measurement in progress — `crates/cm-import/src/bin/tick_profile.rs`
drives the real production path (`tick_days_bound`) after the app's own
boot sequence, with `CM_TICK_PROFILE=1`. Results replace this section.*

The previously quoted ~199 s/day was measured on `run_headless_days`
(which calls `tick_days`, not the World-bound tick the app uses) and was
months old. It should not be trusted until re-measured.

**Structural waste already identified by reading the code** (independent
of the measurement):

1. **Two match pipelines run per fixture.** For every fixture the batch
   first runs the whole legacy scenario pipeline
   (`default_match_engine_runtime_scenario` → player evaluation → late
   branch → action selection → event queue → match events → goal
   events), then runs the real ported engine
   (`resolve_fixture_via_exe_port`). The legacy pipeline's **score is
   explicitly discarded** — an in-code comment says it only still runs
   "so its runtime-store frontiers stay coherent".
2. **O(n²) fixture lookup.** The batch collects due fixture *rows*, then
   for each row does `fixtures.iter().position(|f| f.row == row)` — a
   linear scan over the whole fixture list, per fixture. The list grows
   every season.
3. Per-fixture `clone()` of club names and repeated
   `apply_*_output_to_store` copies.

Spans are now instrumented at: `execute_due_fixture_batch`,
`fixture_row_lookup`, `legacy_scenario_pipeline`,
`resolve_fixture_via_exe_port`, `advance_competitions`,
`hook_evening_daily_ai`.

## 2. Minimum safe optimisation plan

Ordered by (expected gain ÷ risk). Each must preserve RNG order, mutation
order, sort/tiebreak behaviour, scheduling timing and outcomes.

| # | Change | Why it is observationally safe |
| --- | --- | --- |
| 1 | **Index fixtures by row** (build a `row → index` map once per batch, or iterate indices directly instead of rows) | Pure lookup change; same fixtures, same order |
| 2 | **Gate the legacy scenario pipeline** behind a diagnostics flag, defaulting off | Its score is already discarded. MUST first prove the `match_engine_runtime_store` "frontiers" feed nothing observable — if they feed a report or screen, keep it for those paths only. **Highest expected gain.** |
| 3 | Reuse scratch buffers across fixtures instead of allocating per fixture | Allocation only |
| 4 | Avoid `clone()` of club names; borrow or intern | Allocation only |
| 5 | Cache immutable per-club lookups (reputation, tactics, coaching) built once at boot | Values are immutable during a tick |
| 6 | Parallelise independent match simulation **only if** each match's RNG is already independent of the others | Needs proof that per-match RNG is seeded per fixture, not drawn from the shared stream. If matches draw from the session pool, parallelising changes draw order — **not allowed** |

Item 6 is deliberately last and conditional; items 1–5 are unconditional.

**Instrumentation:** `crates/cm-domain/src/tick_profile.rs`, enabled by
`CM_TICK_PROFILE=1`, zero-cost when off. Before/after timings recorded
here for every tranche.

**Target:** advancing a day should be fast enough that a season is
playable interactively. Concretely: a 310-day season in minutes, not
hours.

## 3. Save/load architecture

Use the state that already derives `Serialize`/`Deserialize`. The format
does **not** need to reproduce original CM save bytes (no compatibility
requirement has been stated); it needs to preserve everything required
for observationally identical continuation.

**Shape:**

* One versioned container: `{ format_version, world, save }`, written
  via `serde_json` (or a compact binary codec if size demands it).
* `World` and `RuntimeSaveGame` both already derive serde.
* Write to a temp file then rename, so an interrupted save cannot
  corrupt an existing one.
* On load: deserialize, then **reconstruct derived state** (see §4).

**Front-end routes to wire** (all currently dead-end in "not yet
implemented"):

* Save Game — menu cmd `0x3fe`
* Load Game
* Continue
* the quit guard, which today refuses to close a dirty game

**Tests:** `save → exit/reload → load → continue` end-to-end, asserting
that a tick after load produces the same state as a tick without the
round trip (the strongest available observational-equivalence check).

## 4. Exact list of state that must persist

**Serialized:**

| State | Home |
| --- | --- |
| World (clubs, staff, competitions, stadiums, nations) | `World` |
| Contract pool | `World.contracts` |
| Squad numbers | `World.squad_numbers` |
| Person news mailboxes | `World.person_news_mailboxes` |
| Fixtures + results + match reports | `save.season.fixtures` |
| Standings | `save.season.standings` |
| Session RNG state **and `dbc340_cli_seed`** | `save.session_rng_state` |
| Finance book (balances, accumulators, board state) | `save.finance` |
| Transfers, contracts, Bosman flags | `save.transfers` |
| Player ratings, season goals/assists | `save.player_ratings` |
| Training + development books | `save.training` |
| Injuries | `save.injuries` |
| Competition states (simple leagues, cups, playoffs, continental) | `save.*` |
| Season-roll scheduler + per-comp year | `save.season_roll_scheduler`, `save.season_roll_comp_years` |
| Year-end latch | `save.last_english_year_end_applied` |
| Current date + packed date + phase + elapsed days | `save.date`, `save.simulation` |
| Humans, active human, selected club | `save.humans`, `save.active_human` |
| New-game options (mode, selected nations) | `save.new_game` |
| Honours | `save.honours` |

**Reconstructed on load, not serialized:** FIFA-ranking cache,
`club_reputation` / `club_attendance` / `club_has_chairman` maps if
derivable, any `id_index` caches, the sidebar/menu view models.

**Known trap:** `season_roll_scheduler` is `#[serde(default)]`. A save
written before it existed — or any payload missing it — would load an
**empty** scheduler and then silently never fire Jan-1, so the game would
never regenerate fixtures. On load, if the scheduler has no
registrations, re-register the English pyramid (and log that it did).
Never treat an empty scheduler as valid.

## 5. Standings / multi-season defects to close

1. **Season dimension.** `HeadlessSeasonStanding` now carries
   `competition_id` (cup ties no longer award league points) but has no
   season. From season 2 the Jan-1 regen's appended rows collide with the
   current season's. Add a season key to both the fixture and the
   standing, and filter every consumer.
2. **Sticky statuses.** `run_english_year_end` builds every row with
   `current_status: STATUS_IDLE`, so statuses 3 / 5 / 0xFE / 0xFC never
   carry into the next season and the "movement consumes to 0xFF" path
   has nothing to consume. Carry the previous season's status in.
3. **Competition year advance.** Verify `season_roll_comp_years` advances
   exactly once per comp per season and that the guard cannot double-fire.
4. **Promoted/relegated clubs appear in the right division next season.**
   `Club+0x57` is written; assert end-to-end that the NEXT season's
   fixtures are generated from the NEW membership.
5. Later-season round dates reuse the 2001 template — keep the
   approximation **explicitly labelled**; do not block multi-season play
   on byte-verified future dates.

## 6. Exact `comp.dat` fields missing for stadium expansion

The C14/C15 stadium-expansion path is complete but receives no template
data, so its branch can never fire.

| Field | Offset | Used for |
| --- | --- | --- |
| Comp type byte | `Comp+0x43` | Gates the expansion branch (value `2`) |
| Stadium capacity template A | `Comp+0xE2` | `capacity_template_e2`; `-1` is the skip sentinel |
| Stadium capacity template B | `Comp+0xE4` | `capacity_template_e4` |

**Evidence of the gap:** `rust-db/references/club_competitions.json`
stores `unknown_tail` as **3 bytes** (e.g. `[255, 12, 0]`). The tail
begins at record offset `0x52`, so reaching `0xE2` needs ~0x90 bytes.
The data is simply not imported.

**Work:** extend the comp importer to carry the full record tail (or
named fields for `0x43`/`0xE2`/`0xE4`), re-generate `rust-db`, then
populate `comp_stadium_templates` in `run_english_year_end`. **Do not
invent template values.** Verify through the playable runtime that a real
promotion yields capacity change, seated change, peak update, finance
cost, refusal-counter behaviour and news.

## 7. Current plausible-but-wrong UI/runtime values

Full list in `runtime_reachability_audit.md` §6b. Sixteen pathways; the
ones to close first, each needing *real source → wire → test*, or an
explicit "unavailable" state if no faithful backend exists:

| Value | Currently | Real source |
| --- | --- | --- |
| Squad **Condition %** | hash of `person.id`, `70 + (h % 11)` | `PlayerInitState.condition` |
| Squad **Age** | hash of `person.id` when DOB missing | `PlayerInitState.age`; else show the screen's own `-` |
| Squad **Morale** | literal `"Ok"` | `transfers.contracts[].morale` (already mutated every match) |
| Squad **Age reference date** | hardcoded 2001-08-10 | `save.date` |
| League **position** | `unwrap_or(1)` → tells a club it is 1st | real standings row, else "unavailable" |
| Dashboard **condition** | hardcoded `INITIAL_CONDITION` | live condition |
| **Player of the Month** | gated `day % 30 == 0` → never fires in February | real month boundary |
| **Weekly hook** | gated `day % 7 == 3` → 3–7 day intervals across months | real day-of-week |
| **CA growth** | written to two stores, never back to `type10` | one source of truth for attributes |
| **AI manager hiring/sacking** | empty hook | implement or state it is absent |
| **Transferred player's club** | Squad screen reads static `type6`, engine uses `player_ratings` | single source of truth |
| **Cup income** | `is_cup` hardcoded `false` | `fixture.competition_id` |
| Season labels | hardcoded `2001/2` | `save.date.year` |

## 8. Backend systems that exist but need screen exposure

| System | Live? | UI consumers | Does GDI expose it? |
| --- | --- | --- | --- |
| **Finance** (balances, wages, budget, board confidence) | yes, every tick, 10,580 clubs | **0 — the word "finance" appears nowhere in the app or renderer** | Yes → build it |
| **Injuries** | yes | 0 | Yes → surface on squad/player |
| **Club records** | yes | 0 | Yes |
| **Person career mailbox** | yes | 0 | Partly |
| Honours | yes | 5 | Yes, partial |
| Training | yes | 21 | Yes, partial |
| Contract clause bytes, stadium refuse counter | yes | 0 | **No — internal. Do NOT invent a screen** |

## 9. Screens with a pixel oracle available

Eleven screens ship `paint.pixels.bin` — a raw 800×600 RGB555 frame
(exactly 960,000 bytes) captured from the original executable, beside a
`structure.txt` listing every draw primitive with exact coordinates,
style and 16-bit colour:

`setup / main menu` · `season select` · `name entry` · `leagues select` ·
`nationality select` · `nationality filter` · `club select` ·
`club preview` · `club squad` · `news` · `sidebar + manager menu (open,
with hover states)`

Plus 3,429 primitive draw bursts in `reports/screen_captures/draws/`
(37 named) covering Tactics, Team Selection, player profiles, Contract,
Loan bid, Save Game dialogs, Please Confirm.

## 10. Shared front-end primitives still missing

| Primitive | State |
| --- | --- |
| Palette / colours | **byte-exact port of `FUN_005CE250`** — strongest primitive present |
| Fonts | real `Data/*.fnt` loaded; per-style registry (baseline, spacing, truncation) NOT verified |
| Packed widget renderer | hot path A–F+L byte-exact; **G–K stubs** pending `FUN_00403a20` + 8 palette globals |
| **Scrollbar** | **does not exist** — ad-hoc wheel only, no thumb drag |
| Table / list renderer | no shared canonical renderer |
| Tabs, checkbox, radio, text field | UNMAPPED |
| Tooltips / hover / status bar | UNMAPPED — unknown whether GDI has them at all |

## 11. Canonical scrollbar archaeology target

**Known:** the layout engine reserves a **0x15 = 21 px scrollbar gutter**
(`inner_w -= 0x15`) after calling `FUN_00403640(pad_x, pad_y)`, and only
when `cols < max_columns + 1` — so the width is 21 px and its presence is
conditional. Both measured from the binary.

**Decode target:** `FUN_00403640` and its callees, for track geometry,
arrow-button geometry, thumb minimum size, thumb length and position
formulas, page-click regions, drag offset, clamping, wheel support (if
any), hitboxes and pressed states.

**Cross-check:** `club_screen` and `club_squad_screen` structure dumps
already contain the scrollbar's real draw calls at known coordinates.

Implement **one** shared primitive; never hand-build per screen.

## 12. First five screens to take to true GDI parity

Chosen for (a) pixel oracle present, (b) reachable, (c) primitive
coverage they unlock:

1. **Setup / main menu** — smallest (373 ops), exercises background,
   panels, bevels, buttons, fonts. Proves the pixel gate itself.
2. **Season select** — adds selection highlight + `darken`.
3. **Name entry** — adds text field and caret behaviour.
4. **Club select** — **first screen with a list and a scrollbar**; forces
   the canonical scrollbar and the shared list renderer.
5. **Club squad** — the densest table; multi-column layout, row states,
   and it is where the fabricated Condition/Age/Morale live, so it closes
   a visual and a data defect together.

News and club preview follow (both have oracles) once the primitives are
frozen.

## 13. Tests and gates that will prove each tranche

| Tranche | Gate |
| --- | --- |
| Tick performance | `tick_profile` before/after table committed with each change; plus the existing suite unchanged (any behavioural drift shows as a test failure) |
| RNG safety of an optimisation | A golden test asserting the session RNG snapshot after N days is unchanged by the optimisation |
| Save/load | `save → load → tick` equals `tick` without the round trip |
| Multi-season | production-path test: two consecutive season rolls, correct division membership, no duplicate standings rows |
| Stadium expansion | production-path test: a real promotion produces capacity/seated/peak/cost/news |
| Plausible-but-wrong | per-value production-path test asserting the UI value equals the live backend value |
| Pixel parity | per-screen framebuffer diff against `paint.pixels.bin`, exact or classified |
| Scrollbar | geometry test against the captured draw calls |

Existing gates to keep green: `cargo test --workspace --no-fail-fast`
(currently 103 targets, 2505 passed, 6 pre-existing failures) and
`tools/verify_screens.py`.

## 14. Reports and commits to update

* `reports/full_gdi_integration_ledger.md` — behaviour recovered vs
  production use
* `reports/runtime_reachability_audit.md` — player reachability
* `reports/gdi_frontend_parity_ledger.md` — visual/interaction parity
* `reports/next_work_plan.md` — this file
* **New:** a combined completion matrix (§11 of the directive) with both
  axes per feature/screen: playable route, backend exact, inputs
  complete, mutation exact, persistent, geometry exact, pixels verified,
  interaction exact, approximation remaining, frozen.

"Implemented" is not a terminal status in any of them.

---

## Priority order (strict)

```
P0  tick speed / ability to play a season
P0  save/load
P0  multi-season state correctness
P0  missing imported data blocking existing exact behaviour
P0  plausible-but-wrong live data

P1  expose existing backend systems in real screens
P1  pixel comparison gate
P1  canonical shared scrollbar
P1  screen-by-screen GDI parity

P2  known behavioural approximations (two-leg playoffs, later-season
    round dates, remaining scheduling detail)

P3  new archaeology (aging/retirement, attendance, friendlies, V4,
    scouting)
```

P3 does not start while P0/P1 work remains substantial.
