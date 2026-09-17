# cm0102-rs — Project Status Briefing

**Prepared:** 2026-09-17. Self-contained: assumes no prior knowledge.
**Purpose:** give an external reviewer enough grounding to write the next
work directive.

---

## 1. What this project is

A Rust reimplementation of **Championship Manager 01/02**, built by
reverse-engineering the original Windows executable.

* **Authoritative binary:** `cm0102_GDI.exe` (the GDI software-renderer
  build). The DirectDraw build `cm0102.exe` is corroborative only.
  Addresses differ between the two builds by **non-uniform** deltas —
  you cannot convert one to the other with a fixed offset.
* **Method:** the whole executable (9,155 functions) is decompiled to C
  in `ghidra_out`. Behaviour is recovered function-by-function and
  transcribed to Rust, then verified against captures taken from the
  running original via Frida.
* **Standard of fidelity:** byte-exact where provable. The project
  rejects "looks about right" — an approximation that is not labelled as
  one is treated as a defect.
* **Data:** the shipped game database has been decoded into `rust-db`
  (a native JSON/binary form). ~10,580 clubs, ~132,722 staff records.

**Repo:** `D:\cm0102-rs`, branch `gdi-renderer-port`.

### Crate layout (important — easy to get wrong)

| Crate | Role |
| --- | --- |
| `crates/cm-ui-app` | **THE PLAYABLE GAME** (binary `app`). Everything about player reachability refers to this. |
| `crates/cm-app` | A dev/web inspection tool. **Not the game.** Nothing in it counts as playability. |
| `crates/cm-domain` | The simulation: world model, tick, fixtures, finance, transfers, year-end. ~26k-line `lib.rs`. |
| `crates/cm-render` | The GDI rendering layer: primitives, palette, fonts, screen modules. |
| `crates/cm-db`, `cm-data`, `cm-rng`, `cm-import`, `cm-widget` | DB load, raw data, RNG, importers, widget pool. |

### Test baseline

`cargo test --workspace --no-fail-fast` → **103 targets, 2505 passed, 6
failed.** All six failures are pre-existing and understood:

1-2. two `match_engine_exe` shot-gate tests
3. `scouting::zero_coverage_fogs_to_full_1_20_range`
4. `rust_db_save_executes_due_fixture_batches_into_standings_and_news`
5. `every_fixture_replays_byte_exact` — capture-fixture schema drift
6. `every_screen_has_a_signed_contract…` — a deliberate "population
   gate" (92 of 95 screens unsigned)

Note: `cargo test` stops at the first failing binary, so `--no-fail-fast`
is mandatory or you measure only part of the suite.

---

## 2. What has just been completed

A two-part programme: (a) a **backend integration audit** asking whether
the playable runtime actually calls the proven implementations, and (b)
the beginning of a **front-end parity audit** against the original GDI
screens.

The audit's recurring discovery: **subsystems recorded as "done" were
unreachable in a real game, and every failure was silent** — no error,
no log, just nothing happening.

### Backend defects found and fixed

| Defect | Effect before the fix |
| --- | --- |
| Every tick subsystem built its own RNG from a per-call seed; `pay_weekly_wages` re-seeded the **same constant every week** | Identical wage draws for every club, every Wednesday, forever. Now one shared session `GameRng` threaded through the whole tick, persisted across ticks. |
| The app's game-data init (`run_start_game_init`, the exe's `FUN_008120D0`) was guarded on a `World` that is only loaded *later* | On a first new game the pass silently never ran: **no contract pool, no squad numbers** for the entire session. |
| The year-end passed `per_club_person_slots` empty | The per-person walk had nothing to iterate, so contract-clause, squad-position and person-news writes produced nothing. |
| **All 108,987 contracts carried `club_id == 0`** (written as a literal, with a note deferring it) | The identity gate `*(record+4) == *club` failed for every person, so three completed tranches could never fire. |
| Fixture dispatch gated on `EXACT_SUPPORTED_BASE_YEARS = [2001]`; the Jan-1 season roll asks for year+1 | **The game produced exactly one season of English fixtures and then nothing, ever** — silently. |
| The generic Berger builder only excluded comps the exact engine had *dispatched* | A declined English comp fell through and produced a plausible but non-exact season. |
| Award hooks read `simple_leagues`, which the English pyramid had left | **No English award had fired since** — the one pyramid the port simulates exactly. |
| `playoff_winner_marker` hardcoded `false` | Divisions 1/2/3 never promoted a fourth club; the C9 play-off helpers were unreachable. |
| All four Conference feeder inputs passed empty | The bottom of the pyramid never moved: no feeder promotion, no Conference relegation. |
| **`HeadlessSeasonStanding` had no `competition_id`**, and every played fixture fed it | **Cup ties awarded league points** — corrupting both the League Table the player reads and the promotion/relegation derived from it. |

All fixed, each with a test that drives the **real production entry
point** (load → init → new game → tick → year-end) rather than calling
the helper directly.

### Three ledgers now exist

| Report | Question it answers |
| --- | --- |
| `reports/full_gdi_integration_ledger.md` | What executable behaviour has been recovered, and does production call it? |
| `reports/runtime_reachability_audit.md` | Can a **player** reach it, trigger it, see it, and keep it? |
| `reports/gdi_frontend_parity_ledger.md` | Does the Rust screen render and behave **exactly** like GDI? |

---

## 3. Where the project actually stands

### 3a. The simulation is far ahead of the interface

This is the defining problem right now.

**Proven and working in the backend:**

* Traditional English fixture generation is **byte-exact** — verified
  against a captured GDI trace, 0 mismatches across 380/552/552/552/462
  fixtures for the five English leagues.
* The pyramid distinction is correct and measured: comps 7/8/9/10/93 are
  simulated; 357 (2,351 members), 358, 359, 360 are static pools that
  correctly get **zero** fixtures.
* Year-end rollover: status stamping, promotion/relegation across all
  four edges, play-offs, the Conference feeder swap, contract clauses,
  squad positions, person-news mailboxes, finance writes. All fire and
  mutate the live `World`.
* A full finance system: per-club balances seeded for all 10,580 clubs,
  weekly wages, month-end rollover, per-match gate/TV income, a monthly
  board cascade (debt payment, takeover, stadium share).
* Match engine, injuries, training, AI transfers, cups with decoded
  round dates.

**But in the playable app:**

| | Count |
| --- | --- |
| Screens that are real **and** show live data | ~13 |
| Screens that render but show **fabricated** data | 2 (`FifaRankings`, `LatestScores`) + ~20 menu items painting empty substrates |
| Menu items dead-ending in "not yet implemented" | 14, **including Save Game** |
| Real-game screens that **do not exist at all** | finances, tactics, staff profile, competition view, stadium, preferences, save, load |

Several "real" screens are shells: Club Preview's body is a
"port in progress" panel, Club Transfers has no rows, the Fixtures tab
body is empty with a hardcoded "2001" label.

### 3b. The year-end work is unreachable in practice

Everything in the rollover fires only when every English fixture is
Played and the month is ≥5 — about **310 in-game days**, one click each.
Measured tick cost is **~199 s/day** (708 matches at ~0.28 s each).

That is **~17 hours of continuous clicking to see one season roll over.**
So the entire year-end programme is proven by tests and unreachable by
players. Per-day tick cost is the single highest-leverage fix available.

### 3c. Nothing persists

There is **no serializer call anywhere in the playable app**. A game
exists only in memory and is lost on exit. The close handler literally
says *"once a Save Game screen exists, this will prompt to save; for now
it refuses and logs"*.

All the types already derive serde, so this is wiring, not archaeology.
Caveat: `season_roll_scheduler` is `#[serde(default)]`, so a loaded save
would come back with an empty scheduler and never fire Jan-1 unless
registrations are reconstructed on load.

### 3d. Sixteen "plausible-but-wrong" pathways

The worst class: the player sees a believable value that is fabricated.
None of these log anything. Examples:

* Squad screen **Condition % and Age are deterministic hashes of the
  player id**; Morale is the literal `"Ok"`.
* League position falls back to `unwrap_or(1)` — a club with no
  standings row is told it is **1st**.
* Player of the Month **never fires in February** (the hook is gated on
  `day % 30 == 0`).
* The weekly hook fires on `day % 7 == 3`, so wages, training and the AI
  transfer pass run at 3–7 day intervals across month boundaries.
* CA growth is written to two stores but **never back to `type10`**, so
  after ten simulated seasons every player profile still shows day-1
  attributes.
* AI managers are **never hired or sacked** (empty tick hook).
* A transferred player is played by the engine for his new club while
  the Squad screen still lists the old one — two sources of truth.
* `is_cup` is hardcoded `false` in match income, so every cup tie is
  priced as a league match.

### 3e. Front-end parity — better foundations than expected

The forensic loop already exists and should be **used, not rebuilt**:

* **119 screen captures**, a generator that emits Rust geometry *from* a
  capture (geometry is never hand-written), a differ, and a
  one-command drift gate (`tools/verify_screens.py`). Current gate
  state: 16 exact / 1 divergent / 27 unannotated.
* `crates/cm-render/src/palette.rs` is a **byte-exact port of
  `FUN_005CE250`** — colours are derived by the exe's own packing
  routine, not eyeballed.
* Real shipped bitmap fonts (`Data/*.fnt`) are loaded.
* **A pixel oracle exists**: eleven screens ship `paint.pixels.bin`, a
  raw 800×600 RGB555 framebuffer (exactly 960,000 bytes) captured from
  the original executable, alongside a `structure.txt` listing every
  draw primitive with exact coordinates, style and 16-bit colour.
* 3,429 further primitive draw bursts cover screens the fixtures never
  reached (Tactics, Team Selection, Contract, Loan bid, Save Game
  dialogs).
* Capture tooling can take **new per-state** references:
  `tools/gdi_capture/capture_screen.py` posts a click and captures
  before/after framebuffers.

**The two real gaps:**

1. The existing gate compares **widget geometry, not pixels** — so
   "exact" is unproven at the rendering layer. The oracle exists; no
   gate uses it. (A single-screen precedent already does exactly this
   comparison, so generalising it is small.)
2. **There is no canonical scrollbar.** Only ad-hoc wheel handling; no
   thumb drag. First hard evidence in hand: the layout engine reserves a
   **0x15 = 21 px scrollbar gutter** via `FUN_00403640`, conditionally.

### 3f. Known remaining approximations (labelled, not hidden)

* Play-off ties resolve as a single match; the exe plays two legs plus a
  final on dated May matchdays.
* Seasons after 2001/02 reuse the 2001 round-date template (the 2001
  path remains byte-exact).
* Sticky statuses (3 / 5 / 0xFE / 0xFC) never carry into the next season
  — every table row is built `STATUS_IDLE`.
* Standings have no season dimension, so from season 2 the Jan-1 regen's
  rows collide with the current season's.
* Wage draw is a reputation-banded proxy; gate income layers an invented
  attendance model over the ported formula.
* Squad regen scheduling is a boot sweep; the exe runs it daily.

### 3g. Blocked on data or routing

* **Stadium expansion at promotion** — needs `Comp+0x43/0xE2/0xE4`,
  which `rust-db` does not import (the competition tail is 3 bytes where
  ~0x90 are needed). Requires a `comp.dat` re-import, not a code change.
* **V4 mode** (the game's alternative pyramid) is **unreachable** —
  `GameMode` appears in production only as two hardcoded `Traditional`
  literals; there is no mode field on the save or the new-game options.

---

## 4. Working rules established on this project

Any new directive should respect these:

1. **The executable is the specification.** Where the binary gives exact
   coordinates or drawing order, prefer them over inference from
   screenshots. Screenshots are verification.
2. **Fixed pixel geometry is correct.** This is a 2001 GDI application;
   hardcoded coordinates are not bad architecture. Do not introduce
   responsive layout.
3. **Storage may differ; behaviour may not.** The original was built for
   25-year-old machines. Loading the whole DB into memory, richer
   indices, wider integer types and parallelism are all fine. RNG draw
   order, decision gates, thresholds, table sort tiebreaks and formulas
   are **not** free to differ — they produce a different game.
4. **No silent fallbacks.** A subsystem that cannot do the real thing
   must say so. A missing season is a visible bug; a fabricated one is
   not.
5. **"Unknown" means unknown.** If a screen state has not been observed
   in the original, it is marked `UNKNOWN / UNCAPTURED`, not invented.
6. **Exactness must be proven by the production path**, not by a test
   that calls a helper directly.
7. **One source of truth per concept.** No second finance store, no
   second RNG.

---

## 5. Candidate next steps (unordered — the reviewer should prioritise)

**Unlocking existing exact work**

* Reduce per-day tick cost (~199 s/day). Unlocks the entire year-end
  programme, which is already built and exact.
* Implement save/load. The menu item exists and dead-ends; all types
  derive serde.
* Re-import `comp.dat` fields to unblock stadium expansion.

**Making the interface show the real game**

* `FifaRankings` and `LatestScores` already compute the correct view and
  then discard it, painting `::default()` — one line each.
* Squad Condition/Age/Morale from `PlayerInitState` instead of hashes.
* Build a finances screen — the largest live subsystem with no UI at all.
* Give the ~20 `AutoRoute` dispatcher screens live View payloads.

**Front-end parity**

* Build the pixel gate on the existing `paint.pixels.bin` oracle.
* Recover one canonical GDI scrollbar and reuse it everywhere.
* Build a font/text-style registry from observed styles only.
* Finish the packed widget renderer's G–K paths (blocked on
  `FUN_00403a20` + 8 palette globals).

**Correctness**

* Add a season dimension to standings.
* Work through the 16 plausible-but-wrong pathways.
* Carry sticky statuses across seasons.

**Deliberately deferred (do not start before the above)**

Aging/retirement archaeology, attendance/gate archaeology, friendlies
AI, V4 implementation, new scouting decode.

---

## 6. What a good next directive would decide

The single open strategic question: **the port's simulation is years
ahead of its interface.** Roughly thirteen real screens sit on top of a
byte-exact fixture engine, a full finance system and a working year-end
rollover that a player cannot reach.

So the choice is between:

* **(A) Depth** — make the existing exact backend reachable and
  persistent (tick cost, save/load), so a player can actually play a
  season and keep it; or
* **(B) Breadth** — build out the missing screens (finances, tactics,
  save/load UI) so more of the simulation becomes visible; or
* **(C) Fidelity** — build the pixel gate and the canonical scrollbar,
  and make the screens that *do* exist exactly match the original.

These are not mutually exclusive, but sequencing matters, and the
project has a standing rule not to begin new archaeology while existing
recovered work remains unreachable.
