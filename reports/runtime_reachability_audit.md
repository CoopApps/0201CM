# Runtime Reachability Audit — can a player actually reach it?

**Date:** 2026-09-17
**Scope:** the playable app is `crates/cm-ui-app` (binary `app`).
`crates/cm-app` is a dev/web inspection tool and is NOT the game — nothing
in it counts as player reachability.

**Governing contract (2026-09-17):** the target is *observational
equivalence* — see the fidelity contract at the head of
`full_gdi_integration_ledger.md`. Internals may differ where invisible;
player-visible behaviour, data and outcomes may not. That makes this
report the primary scorecard: reachability by a player IS the measure.

This report answers a different question from
`full_gdi_integration_ledger.md`. That ledger asks *"does production call
the proven implementation?"*. This one asks *"can a real player reach it,
trigger it, see the result, and keep it?"*.

A subsystem can be LIVE + EXACT in the integration ledger and still be
unreachable here — the year-end rollover is exactly that case.

## States used

| State | Meaning |
| --- | --- |
| REACHABLE + FUNCTIONAL | Player reaches it, real implementation runs, effect is visible |
| REACHABLE BUT APPROXIMATE | Reachable and runs, but behaviour is not exe-exact |
| REACHABLE BUT BROKEN | Player reaches it and it does the wrong thing |
| WIRED BUT UNREACHABLE | Production calls it, but no player can trigger it in practice |
| IMPLEMENTED BUT NOT WIRED | Code exists, no production caller |
| VISIBLE UI ONLY / NO WORKING BACKEND | Screen/control exists, data or action is fake |
| BACKEND WORKS / NO FRONT-END ACCESS | Simulation is live, player has no route to see it |
| BLOCKED BY MISSING DATA | Needs data the database does not carry |
| BLOCKED BY MISSING PERSISTENCE | Works in a session, cannot survive save/load |
| BLOCKED BY MODE/ROUTING | Needs a mode or route that does not exist |
| SUPERSEDED / DEAD | Replaced or unused |

## The seven questions

For each feature below: (1) can the player reach it, (2) does the live
runtime call the real implementation, (3) are the inputs present, (4)
does it mutate meaningful state, (5) can the player see the effect, (6)
does it persist, (7) exact or approximate.

---

## 1. THE HEADLINE FINDING — season end is unreachable in practice

Everything the last tranche fixed (year-end rollover, promotion and
relegation, play-offs, the Conference feeder edge, contract clause and
squad-position writes, person-news appends, year-end finance) fires from
`should_fire_english_year_end`, which requires **every English fixture to
be Played** and month >= 5.

A player reaches that only by advancing from the July start to the
following May — roughly **310 in-game days**, one click each via
`advance_active_day`.

The integration ledger records the measured cost of a single day as
**~199 s** (708 matches at ~0.28 s each; ledger §12b, flagged as a
pre-existing performance defect, not a correctness one).

310 days x 199 s ≈ **17 hours of continuous clicking** to see a single
season roll over.

So the entire year-end programme is **WIRED BUT UNREACHABLE**. It is
proven by tests that call `run_english_year_end` directly; no player can
get there. This makes per-day tick cost the single highest-value
reachability fix in the project — it unlocks more already-built, already-
exact work than any other change.

*(Cost figure is from the ledger's earlier measurement; a fresh
measurement against current code is pending and will be recorded here.)*

## 2. Finance — the whole subsystem has no UI

The string `finance` does not appear **anywhere** in `crates/cm-ui-app`
or `crates/cm-render`. There is no finances screen, no balance display,
no wage or transfer-budget readout.

Live and running every tick: `FinanceBook` seeded for all 10,580 clubs,
weekly wages (`pay_weekly_wages`), month-end rollover, per-match gate and
TV/prize income, the monthly board cascade (debt payment, takeover,
stadium share), and the year-end finance writes.

State: **BACKEND WORKS / NO FRONT-END ACCESS.**

## 3. Subsystems with zero UI consumers

Measured by searching `crates/cm-ui-app/src` + `crates/cm-render/src` for
each symbol:

| Backend state | Producer | UI consumers | State |
| --- | --- | --- | --- |
| `finance` (whole book) | tick: wages, match income, board cascade, year-end | **0** | BACKEND WORKS / NO FRONT-END ACCESS |
| `person_news_mailboxes` | `append_person_history_entry` (C15.1E) | **0** | BACKEND WORKS / NO FRONT-END ACCESS |
| `owner_refuse_counter` | C15.1D stadium refusal | **0** | BACKEND WORKS / NO FRONT-END ACCESS |
| `non_promotion` / `relegation` clauses | C15.1B contract writes | **0** | BACKEND WORKS / NO FRONT-END ACCESS |
| `injuries` | daily physio tick + per-match injury rolls | **0** | BACKEND WORKS / NO FRONT-END ACCESS |
| `scouts` | (deliberately unwired — non-faithful) | **0** | IMPLEMENTED BUT NOT WIRED |
| `club_records` | computed from played fixtures | **0** | BACKEND WORKS / NO FRONT-END ACCESS |
| `board_confidence`, `months_in_the_red`, `weekly_wage_bill` | finance tick | **0** | BACKEND WORKS / NO FRONT-END ACCESS |
| `position_code` (squad registration) | C15.1C | 2 | partial |
| `squad_numbers` | boot init | 3 | partial |
| `honours` | competition wins | 5 | partial |
| `fifa_rankings` | boot + year rollover | 20 | has a screen |
| `training` | weekly tick | 21 | partial |
| `transfers` | daily AI pass | 44 | has screens |

**Important caveat per the directive:** a zero here is only a defect if
the ORIGINAL GDI game exposes that state to the player. CM 01/02 does
have finance, injury and news screens, so those are genuine gaps.
Contract clause bytes and the stadium refuse counter are internal — they
should NOT get a new screen invented for them.

## 4. Persistence — nothing survives exit

`GameInstance.saved_path` is never assigned; the close handler literally
says *"once a Save Game screen exists, this will prompt to save; for now
it refuses and logs"*. There is no serializer call anywhere in the app.

Every subsystem below is therefore additionally
**BLOCKED BY MISSING PERSISTENCE**: fixtures, season-roll scheduler,
club competition/status, contracts, position codes, stadiums, refuse
counter, finance, person mailboxes, news, session RNG state, dbc340.

Note `season_roll_scheduler` is `#[serde(default)]`, so even once
save/load exists a loaded game would come back with an empty scheduler
and never fire Jan-1 unless registrations are reconstructed on load.

## 5. Core loop routes (traced)

| Route | State | Notes |
| --- | --- | --- |
| launch → Setup → new game | REACHABLE + FUNCTIONAL | `start_new_game` (main.rs), now runs `ensure_world_initialised` first |
| league selection → game data init | REACHABLE + FUNCTIONAL | `FUN_008120D0` port; was silently skipped before 2026-09-17 |
| new game → fixtures generated | REACHABLE + FUNCTIONAL, EXACT | exact English engine; 380/552/552/552/462 measured |
| club selection → install manager | REACHABLE + FUNCTIONAL | `add_manager` / `install_manager_at_club` |
| advance day | REACHABLE + FUNCTIONAL | `advance_active_day` → `tick_days_bound` (uses the World-bound tick) |
| play/simulate fixture | REACHABLE + FUNCTIONAL | `execute_due_fixture_batch` in phase 2 |
| season end → year-end rollover | **WIRED BUT UNREACHABLE** | see §1 — ~17 h of clicking |
| play-offs | **WIRED BUT UNREACHABLE** + REACHABLE BUT APPROXIMATE once reached | single-leg ties |
| promotion/relegation | **WIRED BUT UNREACHABLE** | gated behind season end |
| Conference feeder edge | **WIRED BUT UNREACHABLE** | gated behind season end |
| new season (Jan-1 regen) | **WIRED BUT UNREACHABLE** | ~180 days of clicking to reach Jan 1 |
| save | **BLOCKED BY MISSING PERSISTENCE** | does not exist |
| load | **BLOCKED BY MISSING PERSISTENCE** | does not exist |
| V4 mode | **BLOCKED BY MODE/ROUTING** | no mode field on save or options; two hardcoded `GameMode::Traditional` literals |
| stadium expansion at promotion | **BLOCKED BY MISSING DATA** | `Comp+0x43/0xE2/0xE4` absent from rust-db |

Screen-by-screen reachability is inventoried separately (see
`gdi_frontend_parity_ledger.md`), since a screen can be reachable but
show fake data, or show real data but look wrong.

## 6. Priority queues

### P0A — existing exact backend blocked from player use

0. **Standings need a `(competition_id, season)` key.** Not a
   reachability blocker but a *correctness* one that invalidates work
   already believed correct: cup ties currently award league points, and
   both the League Table screen and the year-end promotion/relegation
   read those rows. Also fixes the season-2 duplicate-row defect. This
   is the top correctness item in the project.
1. **Per-day tick cost (~199 s/day).** Unlocks the entire year-end
   programme: rollover, play-offs, promotion/relegation, Conference
   feeder, contract clauses, squad positions, person news, year-end
   finance. Highest leverage change in the project.
2. **Save/load.** Everything above is lost on exit even once reachable.
   All types already derive serde.
3. **Finance UI.** A whole live subsystem with no route to the player.
4. **Comp stadium template import** (`Comp+0x43/0xE2/0xE4`) — unblocks
   C14 stadium expansion, which is otherwise complete.

### P1B — reachable but approximate

1. Play-off ties resolve as one match; the exe plays two legs + final on
   dated May matchdays.
2. Seasons after 2001/02 reuse the 2001 round-date template.
3. Sticky statuses (3 / 5 / 0xFE / 0xFC) never carry into the next
   season — every table row is built `STATUS_IDLE`.
4. Standings accumulate duplicate rows per club from season 2.
5. Wage draw is a reputation-banded proxy; gate income layers an
   invented attendance model over the ported `FUN_00584790` form.

### P0B — reachable frontend showing fake or wrong state

Full detail in `gdi_frontend_parity_ledger.md` §3. Ranked:

1. **Squad screen Condition % and Age are hashes of the player id**
   (`render_new.rs:937`, `:1004`, `:1227`, `:1250`). The screen is
   reachable, looks right, and every one of those numbers is invented.
   Real values exist in `PlayerInitState`; they are simply not stored on
   `World` yet. Morale is the literal `"Ok"` everywhere.
2. **`FifaRankings` and `LatestScores` compute the real view and then
   discard it**, painting `::default()`. One-line fix each.
3. **~20 `AutoRoute` menu items** paint empty dispatcher substrates.
4. Season labels hardcoded `2001/2`; squad ages computed against a
   hardcoded 2001-08-10 rather than `save.date`.
5. Select Leagues secondary-league names are a static table in **three
   copies**; the 26 country rows are a hardcoded array that boot already
   cross-checks against the DB and only logs mismatches for.
6. A full set of hand-written transfer rows sits behind
   `SHOW_TRANSFER_DEMO_ROWS = false` — one constant from being visible.

### P2 — genuinely new work (kept separate, NOT part of this programme)

Aging/retirement, attendance/gate archaeology, friendlies AI, V4
implementation, new scouting decode, transfer-record loader.

---

## 6b. SILENT FAILURE AUDIT — plausible-but-wrong pathways

The directive's highest-priority class: the player sees a believable
value or outcome that is fabricated, skipped, or computed from the wrong
source. None of these log anything.

### TOP FINDING — cup ties award league points

`HeadlessSeasonStanding` (`lib.rs:12454`) has **no `competition_id`
field**, and `apply_fixture_to_standings` (`lib.rs:20846-20847`) is
called for *every* played fixture. Domestic cups, super cups and
play-off fixtures — all appended to the same `season.fixtures` — add
P/W/D/L/GF/GA/Pts into the club's single league row.

Two consequences, both severe:

1. The **League Table screen** (`league_table_for`, `lib.rs:1905`)
   renders those rows. The player sees a normal-looking table whose
   points include cup results.
2. **`run_english_year_end`'s `mk_table`** (`lib.rs:19580`) derives
   promotion, relegation and play-off places from the same rows. The
   promotions verified in the previous tranche are therefore computed
   from **polluted standings**.

This compounds the already-recorded defect that season-2 standings rows
are appended without a per-season key. The standings model needs
`(competition_id, season)` before the league table or the year-end can
be trusted.

### Other plausible-but-wrong findings

| Site | What silently happens | Why it matters |
| --- | --- | --- |
| `lib.rs:18365` | League position falls back to `unwrap_or(1)` | Dashboard tells a club with no standings row it is **1st** |
| `lib.rs:18402` | Dashboard condition hardcoded `INITIAL_CONDITION` | Every player shows full fitness forever, including ones `injuries.is_available()` rejects |
| `lib.rs:20765` | `record_match_income(..., is_cup: false)` hardcoded, though `fixture.competition_id` is in scope | Every cup tie priced as a league match; wrong gate/ticket formula, persisted to `balance` |
| `lib.rs:1755`, `18389` vs `transfer.rs:617` | Club membership has **two sources of truth**: static `type6 +0x35` vs `transfers.contracts` / `player_ratings` | The engine plays a transferred player for his NEW club while the Squad screen and Player Profile still list the OLD one |
| `player_development.rs:217` | Club link snapshotted once at boot | Transferred players keep developing under their former club's coaching |
| `player_development.rs:290` | `club_coaching … unwrap_or([100; 5])` | Clubs with no type9 coaches (about half the club table) coach as well as Manchester United |
| `player_development.rs:198` | `if st.attributes.len() < 42 { continue; }` | Those players never develop at all; no counter, no log |
| `lib.rs:20253` | Monthly hook gated on `day % 30 == 0` | **Player of the Month never fires in February**; 11 awards a year and nothing says so |
| `lib.rs:20309` | Weekly hook gated on `day % 7 == 3` ("rough 'every 7 days' trigger") | Wages, training, development, Bosman flags and the AI transfer pass run at 3–7 day intervals across month boundaries |
| `lib.rs:19674` | Play-off tie: `let (Some(h), Some(a)) = … else { return home }` | Awards the tie to the higher seed, then still pushes a "Play-off final — club #X win" news item and stamps the winner marker. Indistinguishable from a played tie |
| `transfer.rs:812-819` | Wage composer fed `player_reputation: 0, international_caps: 0, has_agent: false` and a placeholder squad status | The verified `contract_cost_readback` produces a believable wage from zeroed inputs; that wage is rendered and drives the refuse gate |
| `lib.rs:20480` | `form: 12` — every player enters every match at neutral form | Flattened input to the token model; scorelines shown everywhere |
| `finance.rs:937/997/1216`, `transfer.rs:745` | `club_reputation … unwrap_or(1000)` | A missing club is treated as mid-table for wage tiers and gate income (coverage unverified) |

### Front-end visible (obviously wrong rather than plausible)

| Site | Effect |
| --- | --- |
| `lib.rs:18401` + `player_development.rs:305` | CA growth is written to `PlayerDevelopmentBook` and `player_ratings`, **never back to `type10`** — so Player Profile renders shipped day-1 attributes forever. After ten seasons "nobody has improved" |
| `render_new.rs:917`, `948`, `971` | Squad Status, Apps, Av R and the Stats / More Stats tabs are unconditionally cleared, though `player_ratings` holds season goals, assists and ratings |
| `lib.rs:21672-21761` | Seven empty-body tick hooks, including `hook_manager_job_lifecycle` — **AI managers are never hired or sacked**, observable across seasons |
| `lib.rs:18401` | Missing `player_data_id` link renders CA `0` and sorts the player last |

### Backend-only / unreachable

* `lib.rs:20416` — Man of the Match computed then discarded (`FixtureOutcome` has no slot). Becomes visible if a match-report screen lands.
* `c15_1_world_apply.rs:591` — unresolvable persons are dropped from clause writes with no skip counter, so "all applied" and "none applied" look identical. Would be plausible-but-wrong if the miss rate is non-zero; unmeasured.
* `main.rs:1884` — `tick_days` fallback when `world` is `None`; believed unreachable, but there is no assert if the invariant breaks.

### Not audited

~30 `screen_batch*.rs` ported screen builders (~25 `unwrap_or_default()`
hits). Largest unexamined surface; mostly layout code.

Also noted, not classed as a silent failure: `seed_default_tactics`
(`lib.rs:18129`) gives every club `Tactic::flat_442()`, so AI clubs can
never have any other shape or team settings.

---

## 7. Part XVII summary counts

Runtime reachability (routes traced, not screens):

| # | Measure | Count |
| --- | --- | --- |
| 1 | REACHABLE + FUNCTIONAL | 6 (new game, league init, fixture generation, manager install, advance day, play fixture) |
| 2 | REACHABLE BUT APPROXIMATE | 1 (play-offs, once reachable) |
| 3 | REACHABLE BUT BROKEN | 0 traced so far |
| 4 | WIRED BUT UNREACHABLE | 5 (year-end rollover, play-offs, promotion/relegation, Conference feeder, Jan-1 regen) |
| 5 | IMPLEMENTED BUT NOT WIRED | 1 (scouting — deliberate) |
| 6 | VISIBLE UI ONLY / NO WORKING BACKEND | 3 screen classes (`FifaRankings`, `LatestScores`, `AutoRoute` ×~20) |
| 7 | BACKEND WORKS / NO FRONT-END ACCESS | 6 (finance, injuries, club records, person mailbox, board confidence group, honours partial) |
| 8 | BLOCKED BY MISSING DATA | 1 (stadium expansion — `Comp+0x43/0xE2/0xE4`) |
| 9 | BLOCKED BY MISSING PERSISTENCE | ALL of them (no save/load exists) |
| 10 | P0 runtime blockers | 4 — tick cost, save/load, finance UI, comp template import |

Cross-layer:

| # | Measure | Finding |
| --- | --- | --- |
| 24 | Backend exact, visuals not | Fixture generation (byte-exact) feeds a Fixtures tab whose body is empty and whose season label is hardcoded |
| 25 | Visuals exist, backend not faithful | Squad screen (Condition/Age/Morale invented), `FifaRankings`, `LatestScores`, all `AutoRoute` screens |
| 26 | Silent plausible-but-wrong pathways | Squad Condition/Age hashes; discarded live views; (backend sweep pending) |
| 27 | **Smallest change unlocking the most existing work** | **Per-day tick cost.** It alone gates the year-end rollover, play-offs, promotion/relegation, the Conference feeder edge, contract clauses, squad positions, person news and year-end finance — all already built and exact. |
| 28 | Recommended order | (a) standings `(comp, season)` key — corrects work already believed done, (b) tick cost, (c) save/load, (d) discard-the-live-view one-liners, (e) squad Condition/Age/Morale from real state, (f) finance screen, (g) canonical scrollbar, (h) screen-by-screen parity |

**Note on #26:** the silent-failure sweep found **16** distinct
plausible-but-wrong pathways (§6b), the worst being cup ties scoring
league points. That is a larger class than expected and should be
treated as a standing work queue, not a one-off cleanup.
