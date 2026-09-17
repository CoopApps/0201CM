# Full GDI Integration Ledger — cm0102-rs

Cross-reference of every executable-derived subsystem: what was
decoded, where it lives in the Rust port, whether it is invoked
from the actual running game, and where the gap is.

**Provenance rule (always).** Authoritative binary =
`cm0102_GDI.exe`. `cm0102.exe` (DirectDraw) is corroborative
only. **VA deltas between builds are not uniform** — never
convert DD → GDI with a single offset. See memory
`[[gdi-vs-directdraw-builds]]`.

**Scope.** 434 total commits, 262 on `gdi-renderer-port`, 174
report files. This ledger is a **map**, not an exhaustive
per-commit tally. Each row cites the canonical Rust module and
records the most recent authoritative interpretation; earlier
superseded hypotheses live in the individual reports.

**Status legend:**

* **WIRED** — module invoked from real runtime path (new-game
  boot, tick, or on-demand from live game state).
* **WIRED-PARTIAL** — invoked, but with documented gaps.
* **PORTED-NOT-WIRED** — module exists and passes its tests,
  but no live-runtime caller.
* **PORTED-SEED-ONLY** — invoked at boot; no per-tick action.
* **NOT-PORTED** — decoded elsewhere (report only) but no
  Rust implementation.

## 0. PHASE A AUDIT — engine foundations vs the live runtime (2026-09-17)

State vocabulary (no vague "implemented"): **LIVE + EXACT**, **LIVE BUT
APPROXIMATE**, **EXACT HELPER EXISTS BUT NOT LIVE**, **TRACE-ONLY**,
**STATE MATERIALISED BUT CALLER NOT LIVE**, **SUPERSEDED / DEAD**,
**NOT YET PORTED**.

The question for every row is only: *does the playable runtime call the
proven implementation?*

| # | Foundation | State | Production callsite / evidence |
| --- | --- | --- | --- |
| A1 | Authoritative GDI data + assets | **LIVE + EXACT** | RNG pool is `include_bytes!("assets/game_rng_pool.bin")`, 204,000 bytes, SHA-256 pinned by `pool_sha256_matches_documented`. The GDI address shift (`POOL_ADDR_SHIFT_GDI` = 184) is applied inside `rand_mod` itself, so every production consumer gets the GDI-authoritative stream. Fonts/`Data/*.dat` load from `rust-db`. |
| A2 | Global GameRng ownership | **LIVE + EXACT** | `RuntimeSaveGame::with_session_rng` is the single owner; `tick_cm_phase` threads it to every subsystem. See §12c. |
| A3 | RNG persistence across new game / tick | **LIVE + EXACT** | `session_rng_state` persisted by `with_session_rng`; `new_game_from_rust_db` now saves the post-fixture-gen position so tick 1 continues rather than replaying. §12c. |
| A4 | `DAT_00DBC340` ownership / persistence | **LIVE + EXACT** | Carried on `GameRngState.dbc340_cli_seed`, preserved across every write-back (`GameRng::snapshot()` drops it; `with_session_rng` restores it). §12c. |
| A5 | Runtime-vs-disk object distinction | **LIVE + EXACT** | Disk `Club+0x65` is `ClubView::initial_cash_seed()`, consumed only by `ClubFinance::seed_from` at boot and by the `regen_clubs` dump tool. `ClubView::cash()` is `#[deprecated]`. No production code treats disk cash as live cash. |
| A6 | Club field semantics | **LIVE + EXACT** | `materialise_club_moves` writes `+0x57` / `+0x5B` / `+0x37` on `World.core.clubs[i].raw`, which is exactly what `ClubView::division_id()` and the league-table/club screens read back. Proven end-to-end by `live_year_end_moves_clubs_between_divisions` (real promotions observed, e.g. club 1953 comp 10 → 9). |
| A7 | Competition identity (current / previous) | **LIVE + EXACT** | Same write path: current comp `+0x57`, previous comp `+0x5B`, status `+0x37`. Test above asserts the observable move. |
| A8 | Stadium identity / references | **LIVE + EXACT** | `Club+0x69` → `DomainStadium`; `run_english_year_end` resolves each club's stadium via `ClubView::home_stadium_id()` into `world.references.stadiums` for the C14 affordability inputs. |
| A9 | ContractPool identity / indexing | **LIVE + EXACT (fixed §12d)** | Was **EXACT HELPER EXISTS BUT NOT LIVE** — pool never built in the app, and every record carried `club_id == 0` so the identity gate could not pass. Now built by `run_start_game_init` and keyed to the employing club. |
| A10 | Person / staff resolution | **LIVE + EXACT (fixed §12d)** | `ContractPool.by_staff_id` → `contract_for_staff`; the year-end now builds `per_club_person_slots` from the pool so the C13 per-person walk has real persons. |
| A11 | News / mailbox storage | **LIVE (write) + display gap** | `world.person_news_mailboxes` is appended by `append_person_history_entry` on the live path (§12d). **No screen reads it yet** — the playable News screen renders `save.pending_events`, which the apply layer does push to. So the *simulation* state is live; the mailbox has no reader. Tracked under Phase P. |
| A12 | Runtime finance store | **LIVE + EXACT** | One store: `RuntimeSaveGame.finance` (`FinanceBook`). Second store deleted (§12a); year-end seam is `year_end_state` / `apply_year_end_write`. |
| A13 | Date lifecycle | **LIVE + EXACT** | `simulation.cm_packed_date` canonical, `save.date` mirrors it, synced at boot (§12b). |
| A14 | Save / load reconstruction | **NOT YET PORTED** | There is no serializer call in the playable app at all. See §12e — blocks Phase R entirely. |

Phase A is closed except A14 (no save/load) and the A11 display gap.

## 0b. PHASE B AUDIT — fixture engine foundations (2026-09-17)

| Primitive | State | Evidence |
| --- | --- | --- |
| `matrix_seed_base` / `matrix_perturb` / `walker_step` / outer driver / P1-P2 ordering / schedule buffers / fixture insertion | **LIVE + EXACT** | All reached through `generate_english_traditional_league`, called from `generate_new_game_season_with_rng_and_dbc340` — the fn `new_game_from_rust_db` uses. `tests/c11_1_production_fixture_golden.rs` runs that production dispatch against the captured GDI trace: 0 mismatches across 380/552/552/552/462. |
| Club / stadium resolver | **LIVE + EXACT** | `EnglishClubEntry` built in the dispatch from `ClubView::stadium_id()` + `DomainStadium.alt_stadium_id`. |
| GameRng consumption | **LIVE + EXACT** | Fixture generation draws from the session `GameRng`; the post-generation position is now persisted so the tick continues the stream (§12c). |
| Stored-vs-generated | **LIVE + EXACT** | `simulate_season`'s `england_boot_wires_premiership` now asserts against `new_game_from_rust_db`'s stored output: comp 7 holds exactly 380 fixtures. |

**Old paths — all confirmed inactive for Traditional England:**

* Generic Berger (`generate_double_round_robin`) — excluded.
* `simple_league::from_teams` block for 7/8/9/10/93 — deleted in `2aa4aed`
  (it was generating a parallel season at `RUNTIME_BASE + id`, which is
  what made Cambridge show 94 fixtures and play Colchester twice in two
  days).
* `generate_eng_second_dates` overlay — obsolete, the exact engine
  carries native dates.

**Silent fallback found and closed.** The generic builder excluded only
comps the exact engine had *dispatched*
(`!english_dispatched_ids.contains(id)`). But
`english_dispatch_decision` returns `Skipped` for any base year outside
2001/02 — and those comps then dropped through to Berger pair
generation, producing a plausible-looking but non-exact English season
with no indication anything was wrong. Now the generic builder refuses
comps 7/8/9/10/93 unconditionally
(`!is_english_traditional_league(c.id)`), and a skip logs loudly and
leaves the league with no fixtures. A missing season is a visible bug;
a fake one is not. (Roster-shape failures already panicked rather than
falling back — that guard was correct and is untouched.)

**Superseded test corrected.**
`season_exclusion_tests::english_leagues_are_not_excluded_from_the_generic_builder`
asserted that 7/8/9/10/93 must NOT be excluded from the generic builder,
on the grounds that "nothing builds it". True pre-C11.2, false since.
Replaced by `english_leagues_are_never_built_by_the_generic_builder`.

**Workspace test targets did not compile.** `cargo test --workspace`
failed to build `app` (bin test) and `cm-render` (lib test) on stale
struct literals — `Screen::SelectClub` missing `selected`,
`NewGameOptions` missing `initial_game_rng_state`, `NationalityState`
missing `cursor_x/cursor_y`, `SquadState` missing 20 fields. Those
targets had therefore been running zero tests for some time. Fixed; the
workspace now builds and runs end to end.

## 0c. PHASE C AUDIT — the pyramid distinction, measured (2026-09-17)

Measured on a real England boot through `new_game_from_rust_db`:

| Comp | Members | Fixtures built | Verdict |
| --- | --- | --- | --- |
| 7 Premier | 20 | **380** | simulated, exact engine |
| 8 First | 24 | **552** | simulated, exact engine |
| 9 Second | 24 | **552** | simulated, exact engine |
| 10 Third | 24 | **552** | simulated, exact engine |
| 93 Conference | 22 | **462** | simulated, exact engine |
| 357 A Lower Division | 2,351 | **0** | shared sink — correctly unscheduled |
| 358 Isthmian Premier | 21 | **0** | static feeder — no fake season |
| 359 Southern Premier | 22 | **0** | static feeder — no fake season |
| 360 Northern Premier | 23 | **0** | static feeder — no fake season |

`save.simple_leagues` contains **no** entry for 357/358/359/360, so no
generic league state is fabricated for them either.

State: **LIVE + EXACT.** The runtime does distinguish the three classes,
and it does so through the manageable-league filter rather than the
constant this ledger used to cite — worth knowing, because the constant
would not have excluded them.

## 0d. PHASE D AUDIT — fixture lifecycle (2026-09-17)

Live route: `tick_cm_phase` day rollover → `hook_season_roll_scheduler`
(34-slot table, Jan-1 trigger) → `pending_season_roll_regens` →
`apply_pending_season_roll_regens` → exact engine → appended to
`save.season.fixtures` on the SAME `World.references.club_competitions`
entry (no competition object is rebuilt).

| Link | State | Notes |
| --- | --- | --- |
| Date advance → scheduler | **LIVE + EXACT** | Fires from the day-rollover branch of `tick_cm_phase`, after the date advances, as `FUN_005BFD90` does. |
| Scheduler registration | **LIVE + EXACT (new games)** | `register_english_pyramid` runs in the `RuntimeSaveGame` constructor. |
| Jan-1 trigger → pending regen | **LIVE + EXACT** | `season_roll_comp_years` mirrors `Comp+0x40`; slot guard prevents a double fire. |
| Pending regen → exact generator | **LIVE + EXACT (season 1), LIVE BUT APPROXIMATE (season 2+)** | See the defect below. |
| Install into existing competition | **LIVE + EXACT** | Appends; the current season keeps playing. |

### The game stopped after one season (FIXED)

`english_dispatch_decision` gated on `EXACT_SUPPORTED_BASE_YEARS`,
which is `[2001]`. The Jan-1 roll asks the engine for `year + 1`, so
from 2002 the decision was `Skipped`, the generator was never called,
and `apply_pending_season_roll_regens` hit `if fixtures.is_empty() {
continue }` — **silently**. A save could therefore never have a second
season of English fixtures, and nothing anywhere said so. The ledger
recorded this lifecycle as "FROZEN".

The gate conflated *unverified* with *unsupported*. Only 2001 has a GDI
capture to diff against, but the engine is year-parameterised and runs
for any season. Fixed:

* `EARLIEST_SUPPORTED_BASE_YEAR = 2001` is the dispatch gate (a year
  before the shipped database is a caller bug);
  `EXACT_SUPPORTED_BASE_YEARS` remains, re-documented as a statement
  about *evidence*, explicitly not a dispatch gate.
* An empty regen result now emits a `season_roll_error` event and logs,
  rather than being dropped.

**Fidelity note, deliberate:** seasons after 2001/02 reuse the 2001
round-date template for matchday dates, so they are **LIVE BUT
APPROXIMATE** — right shape, unverified calendar. The 2001 path is
untouched and still byte-exact against the capture. A game that stops
after one season is worse than one whose later matchday dates are not
capture-verified.

Proven by `tests/production_year_end_path.rs::
season_roll_produces_a_second_season_of_fixtures`, which drives the real
drain method: comp 7 goes from 380 to 760 fixtures.

### Awards were blind to the English pyramid (FIXED)

`hook_year_rollover` (end-of-season slate) and `hook_monthly` (Player of
the Month) both iterated `self.simple_leagues` for their league list.
Comps 7/8/9/10/93 stopped appearing there when the generic English block
was removed, so **no English award had fired since** — for the one
pyramid the port simulates exactly. Both now use
`RuntimeSaveGame::award_league_ids_and_names`, which merges
`simple_leagues` with the English comps taken from the live fixture list.

### Still open

Old-save scheduler reconstruction is moot until save/load exists
(§12e). Note `season_roll_scheduler` is `#[serde(default)]`, so a
deserialised save would load with an EMPTY scheduler and never fire
Jan-1 — this must be reconstructed on load when Phase R is built.

Also open: `apply_pending_season_roll_regens` extends
`save.season.standings` with the new season's rows while the old rows
remain, so from season 2 a club appears twice. `run_english_year_end`
builds its tables by filtering standings on club membership, so it would
see duplicate rows. Needs a per-comp-per-season key on
`HeadlessSeasonStanding` — tracked, not yet fixed.

## 0e. PHASE E/F AUDIT — playoffs and status stamping (2026-09-17, IN PROGRESS)

### Playoffs: EXACT HELPER EXISTS BUT NOT LIVE

`run_english_year_end` builds every `FinalTableRow` with
`playoff_winner_marker: false` and `current_status: STATUS_IDLE`,
hardcoded. In `compute_annual_rollover` step 2 the propagation is

```rust
let winner = table.rows.iter().find(|r| r.playoff_winner_marker);
let Some(w) = winner else { continue };
```

so with no marker the whole playoff step is skipped. **The fourth
promotion place in divisions 1/2/3 — the playoff slot — is never filled
in a real game.** `propagate_english_playoff_winner` and the C9
selection helpers are byte-exact and tested; nothing reaches them.

`advance_league_playoffs` in the tick is **not** this: it is the
Brazilian championship playoff (`bra_champ_cup.cpp`), keyed off
`league_playoffs` + `simple_leagues` by name. No English promotion
playoff competition is ever constructed, so there are no semi-finals or
final to produce a winner.

Closing this needs the playoff competition itself built between season
end and the year-end apply (4 clubs from the stamped status-3 rows →
2 semis → final → winner marked), not just a wiring change. Tracked as
the top Phase E item.

### Sticky statuses are never carried in

`current_status` is hardcoded `STATUS_IDLE` for every row, so the
sticky states (3 playoff participant, 5 playoff winner, 0xFE stadium
reprieve, 0xFC) from the previous season never reach the stamper, and
the "movement consumes to 0xFF" path has nothing to consume. The C12
stamper itself is correct and tested; the runtime feeds it a blank
slate each year.

Phase F's expected auto-promotion and relegation position bands are
implemented in `year_end_statuses` and exercised by the live path —
`live_year_end_moves_clubs_between_divisions` observes real moves across
all four edges — but positions alone drive them today, without the
playoff slot or sticky carry-in.

## 1. Foundations (pre-C10 and cross-cutting)

| System | Canonical Rust | Status | Notes |
| --- | --- | --- | --- |
| Executable provenance rules | (documented) `memory/gdi-vs-directdraw-builds.md` | WIRED (rule) | Prefix every citation with build; non-uniform deltas. |
| Game RNG (pool + jitter + LCG + `dbc340_cli_seed`) | `game_rng.rs` | WIRED | Pool asset is GDI-authoritative. `session_rng_state` persists across ticks (added this session). |
| `DAT_00DBC340` boot entropy | threaded through `NewGameOptions.initial_game_rng_state` + `english_rng_dbc340` param | WIRED | Central; not scattered. |
| Date model (`GameDate` + `CmPackedDate`) | `lib.rs` (embedded) | WIRED | Used everywhere. |

## 2. Core record layouts

| Record | Canonical Rust | Status | Notes |
| --- | --- | --- | --- |
| **Club** (stride 0x245) | `DomainOpaqueRecord.raw` accessed via `typed_records::ClubView` | WIRED | Settled offsets: +0x00 id, +0x37 status, +0x57 primary comp, +0x5B previous comp, +0x64 tier, +0x65 initial-cash-seed (renamed C15.1F from misleading `cash`), +0x69 stadium, +0xBF owner-parent, +0xD7 slot area. |
| **Competition** | `DomainCompetition` / `staff_competitions` / `club_competitions` / `nation_competitions` | WIRED | Loaded from rust-db. |
| **Stadium** (0x4B on-disk) | `DomainStadium` | WIRED | Includes runtime-only `owner_refuse_counter: i8` (C15.1D). |
| **Person / Type10** | `DomainStaffType10` | WIRED | Real attribute offsets past +0x24 corrected in memory `[[type10-real-attribute-offsets]]`. |
| **Contract** (0x50 shared pool at `DAT_00accad8`) | `contract_init::ContractRecord` | WIRED | Fields: staff_id, club_id (was mislabelled `person_id`), non_promotion (+0x1C), relegation (+0x1F), position_code (+0x3A). Dual-purpose = ContractRecord AND SquadRecord. |
| **Runtime Finance** (0x167 pool at `*DAT_00acdc38`) | `finance::FinanceBook` (`clubs: Vec<ClubFinance>`) on `RuntimeSaveGame.finance` — the ONE runtime store (see §12a) | WIRED | `balance` = cash i64 at +0x00 (disk +0x65 seed widened, `FUN_005803D0` START_CASH fallback). Four accumulator DWORDs at +0x8C/+0xB4/+0x12C/+0x154 (`season_/lifetime_misc_expense`, `season_/lifetime_subsidy_income`). Also `+0x165` in_administration, `+0x166` board_confidence, `+0x110` month block, `+0x6d`/`+0x82` latches. ~20 more DWORDs deferred. |
| **News mailbox** (222-byte item, per-person mailbox at `DAT_00ACD5C4 + person_id * 0x6E`) | `person_news::PersonNewsMailboxPool` on `World` (C15.1E) | WIRED | Semantic parity — DOB-age routing + 100-ring not modelled (deferred). |
| **Fixture** | `HeadlessSeasonFixture` | WIRED | |
| **News (UI)** | `NewsView` + `NewsItem` | WIRED | Display side, not the mailbox pool. |

## 3. Fixture engine primitives (pre-C10)

| Primitive | GDI VA | Canonical Rust | Status |
| --- | --- | --- | --- |
| Shared outer round-robin driver | 0x00668450..0x00668D70 | `eng_second_fixtures::run_round_robin_driver` | WIRED |
| `matrix_seed_base` | 0x00669340 | `eng_second_fixtures` | WIRED |
| `matrix_perturb` | 0x0066B900 | `eng_second_fixtures::matrix_perturb` | WIRED |
| `walker_step` | 0x0066EE40 | `eng_second_fixtures::walker_step` | WIRED |
| Schedule buffers / fixture insertion | `FUN_00594EB0` (recursive year-cache) | `eng_second_fixtures` | WIRED |
| Stadium/derby graph (Club+0x69 → Stadium; Stadium+0x48 → alt) | | Used inside `english_traditional::EnglishClubEntry` | WIRED (E2/E3 rules by CURRENT-club identity after Phase D shuffling — see `[[perturb-resolver-slot-vs-club-bug]]`) |
| Shared sort/shuffle (`FUN_004B6230`) | | Used by `english_traditional` + Conference feeder-swap | WIRED (single canonical impl) |

## 4. English pyramid — competitions

| Comp | Class | Rust wire point | Status |
| --- | --- | --- | --- |
| 7 Premier | Dynamic simulated | `english_traditional::ExactEnglish` | WIRED |
| 8 First | Dynamic simulated | `english_traditional::ExactEnglish` | WIRED |
| 9 Second | Dynamic simulated | `english_traditional::ExactEnglish` | WIRED |
| 10 Third | Dynamic simulated | `english_traditional::ExactEnglish` | WIRED |
| 93 Conference | Dynamic simulated | `english_traditional::ExactEnglish` | WIRED |
| 358 Isthmian Premier | Static data pool (feeder) | Excluded by `manageable_league_ids_for_nations` — a static pool is not a manageable league, so it never enters `comp_ids`. (The ledger previously credited `LEAGUES_BUILT_BY_DEDICATED_ENGINES`; 357-360 are **not** in that list. Mechanism corrected 2026-09-17.) | **LIVE + EXACT** — measured 21 members, 0 fixtures |
| 359 Southern Premier | Static feeder | same | **LIVE + EXACT** — 22 members, 0 fixtures |
| 360 Northern Premier | Static feeder | same | **LIVE + EXACT** — 23 members, 0 fixtures |
| 357 A Lower Division (multi-nation catch-all) | Shared sink | Same manageable-league filter; `MAX_LEAGUE_CLUBS` = 30 is a second guard behind it | **LIVE + EXACT** — 2,351 members, 0 fixtures |
| **Duplicate simple_league block for 7/8/9/10/93** (pre-C11.2 stale code) | REMOVED (`2aa4aed`) | — | Was causing Cambridge's 94-fixture duplicate season. |

## 5. Cups

| Cup | Comp ID | Round 0 date | Rounds 1-N dates | Status |
| --- | --- | --- | --- | --- |
| FA Cup | 351 | Oct 9 (WIRED) | **This session** — rounds 1-8 wired from `cup_round_schedules.md` via `CupState.round_dates` | WIRED |
| League Cup | 352 | Jul 23 (WIRED) | **This session** — rounds 1-6 wired | WIRED |
| FA Trophy | 94 | Nov 8 (WIRED) | Rounds NOT decoded in report | WIRED-PARTIAL |
| Vans Trophy | 354 | Nov 7 (WIRED) | Rounds NOT decoded | WIRED-PARTIAL |
| French Cup | 335 | Oct 10 (WIRED) | Rounds decoded but not attached | PORTED-NOT-WIRED (round dates) |
| Belgian FA Cup + others | | | Round 0 only | WIRED-PARTIAL |

## 6. Year-end + promotion/relegation

| Layer | GDI | Rust | Wire point | Status |
| --- | --- | --- | --- | --- |
| C7 generic P/R primitive | 0x0066EA90..0x0066EE3D | `year_end_statuses::stamp_league_end_of_season_statuses` | Called by C15 | WIRED |
| C8 English pyramid orchestrator | | `c15_english_annual_rollover::compute_annual_rollover` | Called by `RuntimeSaveGame::run_english_year_end` this session | WIRED |
| C9 playoff selection | | `year_end_statuses` playoff helpers | Called by C15 | WIRED |
| C12 status stamping (0/2/3/5/0xFC/0xFE/0xFF) | | `year_end_statuses` | Called by C15 | WIRED (canonical semantics; superseded hypotheses removed) |
| C13 promotion/relegation apply | | `c13_promotion_apply::{apply_promotion_install, apply_relegation_install}` | Called by C15 | WIRED |
| C14 stadium expansion | `FUN_00583FC0` | `c14_stadium_expansion::apply_stadium_expansion` | Called by C15 | WIRED |
| C14.5 squad manager | | `c14_5_squad_manager` | Called by C13 | WIRED |
| C14.7 Conference active feeder swap | | `year_end_statuses::conference_feeder_swap` | Called by C8/C15 | WIRED |
| C14.7 Conference-absent fallback | | `year_end_statuses::conference_fallback_promotion` | Called by C8/C15 | WIRED (mutually exclusive with feeder swap) |
| Stadium gate / 0xFE reprieve | | Applied inside conference fallback | Called by C15 | WIRED |
| C15 composition | | `c15_english_annual_rollover` | Called by `run_english_year_end` from `tick_days_bound` | WIRED (this session) |
| C15.1A finance materialisation | | `c15_1_world_apply::apply_report_to_world` | Called by tick_days_bound | WIRED |
| C15.1B contract clause writes | | same | Called by tick_days_bound | **LIVE + EXACT (since §12d)** — was EXACT HELPER EXISTS BUT NOT LIVE: three stacked defects (init pass skipped, no person slots passed, `club_id == 0`) made it impossible to fire in a real game. |
| C15.1C squad position writes | | same | Called by tick_days_bound | **LIVE + EXACT (since §12d)** — same three defects. |
| C15.1D stadium refuse counter | | same | Called by tick_days_bound | LIVE + EXACT (does not depend on the contract pool) |
| C15.1E person news mailbox | | same | Called by tick_days_bound | **LIVE + EXACT (since §12d)** — same three defects; `append_person_history_entry` resolves staff via the pool. |
| C15.1F finance writes | | `FinanceBook::apply_year_end_write` (via `apply_report_to_world`) | Called by tick_days_bound; reads `FinanceBook::year_end_state` | WIRED (single store since §12a) |
| C15.1G runtime differential | | `c15_1g_snapshot` + `c15_year_end_diff` bin + Frida harness | User-run tooling | PORTED-NOT-WIRED (needs runtime capture) |

## 7. Season lifecycle

| Trigger | GDI | Rust | Status |
| --- | --- | --- | --- |
| Jan-1 season roll (34-slot scheduler at `DAT_00b4bc70`) | `sub_005605c0` per-comp method | `season_roll_scheduler` fires via `hook_season_roll_scheduler` | WIRED |
| Regen queue drain | `sub_005605c0` calls +0x8C on vtable | `apply_pending_season_roll_regens` | WIRED (via `tick_days_bound` this session) |
| Year rollover (calendar year change) | | `hook_year_rollover` | WIRED (awards + season stats reset) |
| End-of-season detection | | `should_fire_english_year_end` (all English fixtures Played + month >= 5 + guard) | WIRED (this session) |

## 8. Match engine + per-match

| System | Rust | Status |
| --- | --- | --- |
| Match engine | `match_engine_exe.rs` | WIRED |
| Post-match writer (FUN_0069C6F0) | `match_engine_exe` batch writer | WIRED |
| Injury advance-day | `injury::InjuryBook::advance_day` | WIRED (daily tick) |
| Injury seasonal reset | `injury::InjuryBook::reset_season` | WIRED (hook_year_rollover) |

## 9. Boot-time seeding

| Subsystem | Rust | Status |
| --- | --- | --- |
| Rust-db loader | `World::read_rust_db_dir` | WIRED |
| Player init (FUN_0051f5d0) | `PlayerInitState` | WIRED at new-game |
| Contract pool | `contract_init::initialise_all` | **LIVE + EXACT (since §12d)** — built by `run_start_game_init`, NOT by `read_rust_db_dir`. Was never running in the app at all. |
| Squad numbers | `World.squad_numbers` | **LIVE + EXACT (since §12d)** — same skipped init pass; was empty for the whole session. |
| Finance seed | `FinanceBook::seed_from_clubs` in `new_runtime_save_from_rust_db` | WIRED — every one of the 10,580 clubs gets `balance` from disk `+0x65` (i32→i64) or the `FUN_005803D0` START_CASH-by-reputation fallback. (`ClubFinanceLedger::seed_from_world` is now test-only; the persisted ledger field was removed — §12a.) |
| Person news mailboxes | allocated on `World` | WIRED |
| FIFA rankings | `fifa_rankings.rs` | WIRED at boot + hook_year_rollover cache clear |
| Player regen — squad fill (`FUN_0078E970` driver / `FUN_0078F200` selector / `FUN_0078F4F0` scorer) | `player_regen::regen_fill_club_squad`, called from `new_game_from_rust_db` after fixture gen on the shared session RNG | **WIRED-PARTIAL** — the selector + scorer are byte-exact and now actually used (replaced the removed CA-ranked `assign_free_agents_to_empty_clubs` duplicate). The **scheduling is an approximation**: the port does a one-off boot sweep (clubs with scheduled fixtures, <8 real players → fill to 14). The exe does NOT do a boot sweep — `FUN_0078E970()` is called with no args once per day from the daily tick driver `FUN_005B6F10` (right after `FUN_0078DD80()`, before `FUN_0089DE30`); its "param_1" is one global regen context. `FUN_0078DD80` (323 lines) is the departure scheduler: scans date-gated staff, schedules departures 1–65 days ahead (`rand(0x41)+1+today` / `rand(0x23)`), and qsorts the 12-byte departure list (`FUN_009343C3` with comparator `FUN_00796590`). The fill target club (`ctx+0x1c`) and count (`ctx+0x20 − 2`) are set by a still-unidentified writer (`FUN_00790600` is the candidate). Follow-up to make scheduling exact: decode `FUN_0078DD80` + that writer and move the fill into the daily tick. RNG note: the boot sweep consumes pool RNG the exe would not consume at boot; the C11.2 fixture golden is unaffected (regen runs after fixture gen) and the Jan-1 state is already non-reproducible until the daily pool consumers (form rolls etc.) are ported. **Other known fidelity gaps (pre-existing in the port, now documented):** (1) the exe driver's opening loop drains a per-club *departure list* (`param_1+0x10`, 12-byte entries, `FUN_00793e10` release + compaction) — not an age-based retirement, and not modelled; (2) fill count comes from the caller as `param_1+0x20 − 2` — the port's 8/14 thin/target thresholds are inherited and NOT exe-verified (caller decode pending); (3) after each pick the exe links the staff record into club attach slots (`club+0xd3`, or one of 5 at `club+0x19f`, setting `staff+0x3d` = 0xf/0xd) and, when the pick has no type10 record, seeds a `+0x69` record with `rand(0x5dc)+1, rand(0x5dc)+1, rand(500)+1` — the port writes `staff+0x39` only and skips those rolls, so the session RNG stream diverges from the exe's after such a pick (fixture golden unaffected since regen runs after fixture gen). Picks propagate to `save.player_ratings` (persisted); `SaveWorldOverlay.staff_overrides` has no loader yet so is not written. |

## 10. Ported but NOT wired at runtime (integration gaps)

| Subsystem | Rust | Gap | Estimated wiring cost |
| --- | --- | --- | --- |
| **Friendlies / pre-season tour** | `friendly.rs` (primitive only) | Arranging AI (~35 exe helpers) + calendar slot not written | **Big** (multi-day port) |
| **Transfer window ticks** | `transfer.rs` (~4kloc) | Only 3 refs from lib.rs — load-time helpers only. No transfer-window tick, no bid/negotiate/accept AI in the daily loop. | **Big** |
| **Scouting reports** | `scouting.rs` | **PORTED-APPROXIMATE — deliberately NOT wired.** `ScoutBook::weekly_tick` exists but its own doc says it is not a faithful port: it uses invented 25/50/75/100 coverage thresholds (the exe scout cluster has no such literals) and an integrative weeks-watched model the exe doesn't have (the exe is snapshot-based). Wiring it would reintroduce a silent approximation. Blocked on decoding staff+0x113 semantics + `FUN_00489790` (see `reports/scout_knowledge_formula_decode.md`). Also note: fog is a human-display filter only — AI transfers see full CA/PA regardless. | Decode first (Medium), then wire |
| **Player regen at year-end** | `player_regen.rs` | Boot-time only; no annual youth intake | Medium |
| **Aging / retirement** | Task #48 says `in_progress`; no code | Not archaeologised → needs decode first | Large (archaeology + port + wire) |
| **Gate / TV / prize income per match** (`FUN_00584790` + `FUN_00585060`) | `FinanceBook::record_match_income(home, away, is_cup)`, fired per played fixture | **WIRED-APPROXIMATE — correction to an earlier row that said "no code".** The `FUN_00584790` gate form (`rand(250)+rand(250)` league / `rand(400|200)` cup, × `(rep/500+3|4)`) is ported but only as the *fallback*; the live path uses an invented attendance model ("60% avg + 40% draw between shipped min/max, × ticket = rep/500 + 15|20") layered over it, and it now draws from the shared session pool RNG (§12c). Which of the two is exe-exact needs `00584790.c` re-read; do not flip blindly. Month accumulators `month_gate` / `month_tv_prize` on `ClubFinance` mirror the exe's +0x110 block. | Decode-check `FUN_00584790` gate arithmetic (Small) |
| **Weekly wages** (`FUN_00586EC0:363-421`) | `finance::FinanceBook::pay_weekly_wages`, fired every Wednesday from `hook_weekly_wednesday` | **WIRED — correction to an earlier row that said "not wired".** It is a port: the exe draws a reputation-banded weekly amount (three tiers by balance-vs-reputation), it does NOT sum contract wages (see `reports/weekly_wage_bill_decode.md`), so a contract-sum replacement would be less faithful. Draws from the shared session pool RNG as of §12c (it previously re-seeded the same constant every week). Remaining deviation: `takeover_pending` stands in for the +0x82 chairman-boost flag. | — (§12c fixed) |
| **Board debt payment / takeover / stadium-share** (`FUN_00587C40`, `FUN_005884A0`, `FUN_00586EC0:56-114`) | `FinanceBook::board_debt_payment`, `takeover_check`, `stadium_share_transfers`, monthly `tick_month_board`; fired from the monthly cascade in the tick | **WIRED — correction to an earlier row.** Ported with line-cited gates. Draws from the shared session pool RNG as of §12c. Remaining deviation: the cascade order is "approximated with the ported fns available" (comment at the call site). | — (§12c fixed) |
| **Domestic cup rounds 1-8** | `domestic_cup.rs` | Fixed this session for FA Cup 351 + League Cup 352. FA Trophy 94, Vans 354, French Cup 335, etc. still need round dates. | Small each |
| **`FUN_005121A0` full loader trace** | | Partially traced in C15.1F; full loader-chain wiring deferred | Medium |

## 11. UI / screen infrastructure

| Layer | Rust | Status |
| --- | --- | --- |
| GDI primitives (line/rect/glyph/panel/…) | `cm-render` | WIRED |
| Packed widget renderer (`FUN_005d7aa0`) | `packed_widget.rs` | WIRED |
| Screen builders (117 View types) | `crates/cm-domain/src/screen_batch*.rs` | WIRED |
| Screen dispatcher (`FUN_007491e0`) | `render_new.rs` | WIRED |
| Menu bar (`FUN_00745540`) | `screens::menu_sidebar` | WIRED |
| Sidebar dispatch (`FUN_0076ab10`) | `render_new.rs` | WIRED |
| Layer 2 widget rendering fold | | WIRED |
| Real Data/*.fnt fonts | `Fonts` | WIRED |
| Cursor + dirty-rect blit | `cm-render` | WIRED |
| Fixtures scroll wheel (this session) | `main.rs` MouseWheel arm + `render_fixture_rows_scrolled` | WIRED |

## 12a. TWO FINANCE STORES — open consolidation defect (found 2026-09-17)

The directive's "no two finance sources of truth" rule is currently
violated. The port carries **two** runtime finance stores that both
model the exe's 0x167-byte per-club finance record:

| Store | Module | Fields | Seeded at boot | Mutated by |
| --- | --- | --- | --- | --- |
| `RuntimeSaveGame.finance: FinanceBook` (`clubs: Vec<ClubFinance>`) | `finance.rs` | `balance: i64`, `weekly_wage_bill`, `transfer_budget`, `months_in_the_red`, `board_confidence` (+0x166), `in_administration` (+0x165), `month_wages/gate/tv_prize` (+0x110 block), stadium-share latch (+0x6d), takeover latch (+0x82) | YES — `FinanceBook::seed_from_clubs` in `new_runtime_save_from_rust_db` | Every Wednesday (`pay_weekly_wages`), month-end (`end_of_month`), per match (`record_match_income`), monthly board cascade (`tick_month_board`), `board_debt_payment` / `takeover_check` / `stadium_share_transfers` |
| `RuntimeSaveGame.finance_ledger: ClubFinanceLedger` (`per_club: BTreeMap<u32, ClubFinanceState>`) | `c15_1_world_apply.rs` (C15.1A/F) | `cash: i64`, `season_misc_expense`, `lifetime_misc_expense`, `season_subsidy_income`, `lifetime_subsidy_income` (+0x8C/+0x12C/+0xB4/+0x154) | **NO** (`seed_from_world` never called in production) | Year-end only (`apply_write` from C14 stadium-expansion outputs) |

Consequences today: the tick debits `finance.balance`; `run_english_year_end`
reads `finance_ledger.cash` — which is never seeded and never sees a wage
payment — so stadium-expansion affordability at season end uses cash = 0.
C15.1F chose "promote the ledger" believing no canonical runtime finance
object existed; `finance.rs::ClubFinance` already was one (it even carries
+0x165/+0x166/+0x110 offsets).

**Resolved (2026-09-17):** `FinanceBook` is the single owner. The 4
accumulators now live on `ClubFinance`; `FinanceBook::year_end_state(club)
-> ClubFinanceState` (cash = balance), `year_end_states(ids)` and
`apply_year_end_write(&PendingFinanceWrite)` are the year-end seam;
`RuntimeSaveGame.finance_ledger` and its 9 constructor inits are deleted;
`run_english_year_end`, `apply_report_to_world`, the diff CLI and the
diagnostic bins read/write `save.finance`. `YearEndSnapshot::from_apply`
now takes a plain `BTreeMap<u32, ClubFinanceState>` so it is decoupled from
any store. `ClubFinanceLedger` survives only as a documented **value-type
helper** for the C15.1A/F arithmetic tests and hand-seeded snapshots — it
is no longer persisted anywhere. C15.1A arithmetic untouched.

## 12b. Boot date sync + tick cost (found 2026-09-17, boot_check evidence)

* **Date model (FIXED):** `new_game_from_rust_db` set `save.date` to the
  picker's normalised start (07-10) but not `simulation.cm_packed_date`
  (left at 07-01 by the runtime-save constructor); `tick_cm_phase` derives
  `date` from the packed form every step, so the picker's start was
  silently discarded on tick 1 (boot 07-10 → tick 1 = 07-02). Fixed by
  syncing the packed date at boot. The packed date is canonical;
  `save.date` mirrors it.
* **Tick cost (OPEN, pre-existing):** `tick_days(1)` took **199 s** on the
  England boot (708 matches ≈ 0.28 s each through the match batch). Any
  multi-season headless run is impractical until the per-match cost drops
  by ~2 orders of magnitude. Not a correctness defect; blocks the C15.1G
  full-season differential in practice.

## 12c. Tick RNG unified on the session pool (FIXED 2026-09-17)

Found: the directive's RNG rule ("use the shared session `GameRng`; do
not instantiate local RNGs") was violated by every tick-time subsystem.
Each built its own `MatchRng` from a per-call seed, so nothing shared a
stream and one of them repeated outright:

| Site | Was | Effect of the defect |
| --- | --- | --- |
| `pay_weekly_wages` | `MatchRng::new(0x0058_6ec0)` — the **same constant every call** | Identical wage draws every Wednesday, for every club, forever. |
| `record_match_income` | `MatchRng::new(0x0058_4790 ^ home<<16 ^ away)` | Same gate draw for a given pairing in every season. |
| `end_of_month` | `MatchRng::new(0x0058_84a0)` — constant | Same month-end draws every month. |
| monthly cascade / `tick_month_board` | `MatchRng::new(seed ^ 0xA5A5A5A5)` | Per-month deterministic; also fed a fake "date_short" out of the seed bits. |
| `run_ai_transfer_pass` (both call sites) | `MatchRng::new(date-folded seed)` | Also fed a fake `game_day` to `predict_wage`. |
| `development.weekly_tick` | `MatchRng::new(0x0089_de50 ^ …)` | Per-week deterministic. |

Fix, in order:

1. `game_rng::PoolRand` — the trait for "draws from the exe's one pool
   rand". `GameRng` implements it over `rand_mod`; `MatchRng` also
   implements it so per-match code and value-type tests still compile.
   Subsystems take `&mut impl PoolRand`; none constructs an RNG.
2. `RuntimeSaveGame::with_session_rng(f)` — loads `session_rng_state`,
   runs `f(save, rng, dbc340)`, persists the advanced state, preserving
   `dbc340_cli_seed` (`GameRng::snapshot()` drops it, so the old
   `tick_days_bound` write-back zeroed it after day one).
3. `tick_cm_phase` now wraps the whole phase in `with_session_rng` and
   passes `rng` down through `execute_due_fixture_batch`,
   `hook_evening_daily_ai`, `hook_monthly` and `hook_weekly_wednesday`.
   `tick_days_bound` no longer holds its own RNG.
4. `new_game_from_rust_db` persists the session RNG position after
   fixture generation + boot regen, so the first tick **continues** that
   stream rather than re-bootstrapping and replaying its draws.
5. Two fake values that were being folded out of the dead seeds are now
   real: `tick_month_board(elapsed_days, …)` (the exe's 16-bit day
   counter, compared against `generosity * 750`) and
   `run_ai_transfer_pass(…, game_day, …)`.

Also removed here: `scouts.weekly_tick()` was found **wired** into the
Wednesday hook despite the Scouting row recording it as deliberately not
wired. Unwired, with the reason inline at the call site.

This removes the repeating-sequence defect and gives the tick one
stream. It does NOT by itself make that stream exe-exact — that needs
every daily pool consumer (form rolls in `FUN_005B6F10`, regen
scheduling, …) ported and drawing in the exe's order.

## 12d. PHASE A — three stacked defects hid C15.1B/C/E from the live game (FIXED 2026-09-17)

Phase A of the integration audit asked the question the ledger had not:
**does the playable app reach the proven code?** For the contract-pool
tranches the answer was no, at three independent levels. Each one alone
was enough to make the feature invisible, and all three presented as
"no writes", indistinguishable from "nothing to write".

**Level 1 — the boot init pass never ran.** `World::run_start_game_init`
(the exe's post-league-selection `FUN_008120D0`: player init → squad
numbers → contract pool) had exactly one caller, `main.rs:1039`, guarded
by `if let Some(w) = self.world.as_mut()`. The only place `self.world`
is assigned is `start_new_game`, which runs *after* it. So on a first new
game the guard fell through silently and the pass never ran at all:
`world.contracts == None` and `world.squad_numbers` empty for the whole
session. Fixed by `App::ensure_world_initialised`, which loads the DB
*then* runs the pass; both call sites go through it.

**Level 2 — the year-end passed no person slots.** `run_english_year_end`
built its `AnnualRolloverInput` with `per_club_person_slots:
Default::default()`. The C13 per-person walk iterates that map, so it had
nothing to walk and produced zero effects regardless of the season.
Now built from the contract pool: for each club in the tables, its
contracted staff, carrying `+0x1F` relegation and `+0x1C` non-promotion,
capped at the exe's 50 slots (`Club+0xD7`).

**Level 3 — every contract had `club_id == 0`.** `contract_init::
initialise_all` wrote a literal `0`, with an inline note deferring the
real value to a disk→runtime loader trace. The identity gate
(`*(record+4) == *club`, FUN_00843970 / FUN_004D3550 / FUN_004D3460)
therefore failed for all 108,987 contracts, so even with levels 1 and 2
fixed nothing would write. Fixed by taking the employer already in scope
(the loop only emits a contract when the staff record has an employer,
and already prices it off that club's reputation). Confirmed: contracts
now carry 5,490 distinct clubs and all 20 comp-7 clubs resolve.

**Ledger rows corrected:** C15.1B, C15.1C and C15.1E were all recorded
as `WIRED`. True state before this fix was **EXACT HELPER EXISTS BUT NOT
LIVE** — the helpers are byte-exact and tested, and could not fire in a
real game.

**No-silent-failure follow-ups:**

* `WorldApplyReport.contract_pool_missing` distinguishes "no pool" from
  "no writes due"; `run_english_year_end` emits a `year_end_error` event
  and logs when it is set.
* `World.contracts`' doc claimed `read_rust_db_dir` populated it. It does
  not, and `tests/contract_pool_smoke.rs` asserted the false claim — that
  test had been failing. Doc corrected, test fixed to run the init pass.
* New `tests/production_year_end_path.rs` proves the sequence the app
  runs (load → init → new game → detector → year-end) rather than calling
  the C15 helpers directly.

**Baseline correction:** the previously reported "1744 pass / 4 known
failures" covered only the `--lib` target. `cargo test` stops at the
first failing binary, so the integration-test targets were never
reached. Full-suite figures now use `--no-fail-fast`.

**Still open from Phase A** (tracked, not silently accepted):

* `feeder_candidates`, `conference_marked_for_relegation`,
  `fallback_candidates`, `third_div_relegatees` and
  `comp_stadium_templates` are still passed empty from
  `run_english_year_end` — Phases H, I and M will close them.
* Save/load does not exist in the playable app (`GameInstance.saved_path`
  is never set; the quit guard says "once a Save Game screen exists").
  Phase R cannot run until it does. See §12e.
* The 50-slot `Club+0xD7` cap is preserved. Whether that is a real model
  limit or a memory-era artifact worth lifting is an open decision.

## 12e. PHASE A.14 — save/load does not exist (OPEN)

`App::GameInstance` carries `saved_path: Option<String>` and a `dirty`
flag, and the close handler refuses to quit a dirty game with "*once a
Save Game screen exists, this will prompt to save; for now it refuses
and logs*". There is no serializer call anywhere in `cm-ui-app`: a game
exists only in memory and is lost on exit.

State: **NOT YET PORTED**. This blocks Phase R (round-trip of fixtures,
scheduler, club comp/status, contracts, position codes, stadiums, refuse
counter, finance, mailboxes, news, RNG state, dbc340) in its entirety.

The pieces exist — `World`, `RuntimeSaveGame`, `ContractPool` and
`PersonNewsMailboxPool` all derive `Serialize`/`Deserialize`, and
`session_rng_state` was made serde-persistent in §12c — so this is a
wiring job, not an archaeology job.

## 12. Known deviations (semantic parity, not byte-exact)

* Person mailbox: DOB-age routing + 100-entry ring + 0xDF stride not modelled. Semantics preserved.
* C15.1C nation-affiliation identity clause not modelled (needs `DAT_00acd5bc` nation base + `Club+0xBF` chain wiring).
* Runtime finance record: 24+ accumulator DWORDs beyond the 5 modelled fields still deferred.
* Fixtures screen scrollbar-thumb drag: only wheel-scroll wired.

## 13. Confidence upgrades pending capture

The C15.1G Frida capture harness + snapshot comparator + diff
CLI are shipped and ready. Running a real GDI capture against a
2001→2002 save would upgrade Traditional English year-end from
PROVISIONAL to STATE-EXACT for observed branches. Not run
(three attempts crashed the exe — documented in
`[[frida-instrumentation-crash-evidence]]`).

## 14. Prioritised next wire-ups

1. **FA Trophy + Vans Trophy + French Cup round dates** (small each — same shape as this session's FA Cup / League Cup wiring).
2. **Player regen annual intake** at end-of-season (medium; the primitive is there).
3. **Scouting per-tick dispatch** (medium).
4. **Aging / retirement archaeology + port** (needs new decode).
5. **Match income**: already wired (`record_match_income`); remaining work is a decode-check of the `FUN_00584790` gate arithmetic vs the attendance override, the §12c RNG fix is done.
6. **Transfer window tick** (big — full AI port).
7. **Friendlies arranging AI** (big — ~35 helpers).

## 15. What is complete

- **Traditional English fixture generation**: FROZEN (byte-exact for 5 leagues).
- **Traditional English fixture lifecycle** (Jan-1 regen): FROZEN.
- **Traditional English year-end apply** (C15.1x): FROZEN; wired to runtime tick this session.
- **Core record layouts** for Club / Competition / Stadium / Contract / Finance / Person / Mailbox: settled.
- **UI rendering pipeline**: settled.

## 16. What is provisional / not started

- Transfer window logic.
- Scouting per-tick.
- Aging / retirement — archaeology not done. (TV/prize: wired, see §10; only the gate-arithmetic check remains.)
- Friendlies AI.
- Non-English nations' year-end pipelines (only Traditional English is wired end-to-end).
- The C15.1G runtime differential (Frida capture unstable).

---

## How to keep this current

When a new tranche lands:

1. Add or update the relevant row here.
2. Cite the canonical Rust module + wire point.
3. Move status labels (usually PORTED-NOT-WIRED → WIRED).
4. Move superseded hypotheses to per-subsystem reports; keep this doc as the single-source status map.
