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
| **Runtime Finance** (0x167 pool at `*DAT_00acdc38`) | `ClubFinanceLedger` on `RuntimeSaveGame.finance_ledger` (C15.1F) | WIRED | Cash i64 at +0x00 (widened from disk +0x65 seed via `__ftol`). Four accumulator DWORDs at +0x8C/+0xB4/+0x12C/+0x154. ~24 more DWORDs deferred. |
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
| 358 Isthmian Premier | Static data pool (feeder) | `LEAGUES_BUILT_BY_DEDICATED_ENGINES` exclusion | WIRED (correctly excluded) |
| 359 Southern Premier | Static feeder | same | WIRED |
| 360 Northern Premier | Static feeder | same | WIRED |
| 357 A Lower Division (multi-nation catch-all) | Shared sink | `generate_double_round_robin` `MAX_LEAGUE_CLUBS` = 30 gate skips it | WIRED |
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
| C15.1B contract clause writes | | same | Called by tick_days_bound | WIRED |
| C15.1C squad position writes | | same | Called by tick_days_bound | WIRED |
| C15.1D stadium refuse counter | | same | Called by tick_days_bound | WIRED |
| C15.1E person news mailbox | | same | Called by tick_days_bound | WIRED |
| C15.1F finance ledger | | same | Called by tick_days_bound | WIRED |
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
| Contract pool | `contract_init::initialise_all` | WIRED at new-game |
| Squad numbers | `World.squad_numbers` | WIRED at new-game |
| Finance ledger seed | `ClubFinanceLedger::seed_from_world` | **NOT WIRED — production never calls it** (only tests + the diff CLI). At runtime `save.finance_ledger` is empty, so `run_english_year_end` reads cash = 0 for every club. See §12 "TWO FINANCE STORES". |
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
| **TV / prize money** | Task #49 `in_progress`; no code | Not archaeologised → needs decode first | Large |
| **Weekly wage accumulator** | Referenced in transfer.rs but no tick action | Cascade fn `FUN_00586EC0` decoded, not wired to tick | Medium |
| **Board debt payment** | Referenced in finance.rs | `FUN_00587C40` decoded, not wired | Medium |
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

**Plan (separate commit):** `FinanceBook` becomes the single owner. Add the
4 accumulators to `ClubFinance`; expose `FinanceBook::year_end_state(club)
-> ClubFinanceState` (cash = balance) and `apply_year_end_write(&PendingFinanceWrite)`
(writes balance + accumulators); delete `RuntimeSaveGame.finance_ledger`
and its 9 constructor inits; repoint `run_english_year_end`,
`apply_report_to_world`, `YearEndSnapshot::from_apply`, the diff CLI, and
the C15.1F/G tests. `ClubFinanceState` survives as a value-type snapshot.
The C15.1A arithmetic is untouched — only the storage owner changes.

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
5. **TV / prize money archaeology + port** (needs new decode).
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
- Aging / retirement / TV / prize money — archaeology not done.
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
