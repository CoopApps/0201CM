# GDI Front-End Parity Ledger

**Date:** 2026-09-17
**Subject:** `crates/cm-ui-app` (binary `app`) + `crates/cm-render`.

## The rule this ledger enforces

We are **not designing a UI that looks like CM 01/02**. We are
**transcribing the GDI executable's rendering and interaction behaviour
into Rust**. Fixed pixel coordinates are correct. Approximation is not.

Two independent questions per screen, never conflated:

* **A — does it look and behave like GDI?** (this ledger)
* **B — is it backed by real simulation state?** (also recorded here,
  because a screen can be visually right and semantically fake)

## Parity states

`UNMAPPED` · `REFERENCE CAPTURED` · `MEASURED` · `PARTIALLY IMPLEMENTED` ·
`VISUALLY APPROXIMATE` · `GEOMETRY EXACT / VISUALS NOT EXACT` ·
`VISUALLY EXACT / INTERACTION NOT EXACT` ·
`INTERACTION EXACT / DYNAMIC DATA NOT LIVE` · `GDI-PARITY CANDIDATE` ·
`FROZEN`

`UNKNOWN / UNCAPTURED` is a legitimate value for any per-state row. It
must NOT be replaced by extrapolation ("the other tabs probably behave
the same").

---

## 1. Existing parity infrastructure (found, not built)

The project already has the forensic loop the directive asks for. It
should be used rather than replaced:

| Piece | Path | What it does |
| --- | --- | --- |
| Capture corpus | `reports/screen_captures/` (119 entries) | Per-screen widget/geometry captures from the exe, plus `.render.json` counterparts |
| Generator | `tools/gen_screen_rs.py` | Emits a Rust screen module FROM a capture — geometry is never hand-written |
| Geometry dump | `dump_screen_geometry` bin | Emits the generated skeleton's geometry |
| Differ | `tools/diff_screens.py` | Diffs generated geometry against the exe capture |
| Drift gate | `tools/verify_screens.py` | One command: regenerate → build → dump → diff → PASS/FAIL table, exit 0 iff all match |
| Palette | `crates/cm-render/src/palette.rs` | **Byte-exact port of `FUN_005CE250`** — colours are derived from the exe's own packing routine, not eyeballed |
| Fonts | `Data/*.fnt` via `font_loader.rs` | Real shipped bitmap fonts |

Current state of the gate (`reports/screen_captures/DIFF_REPORT.md`):
**exact-match 16 · divergent 1 · no-port-annotation 27** across 44
tracked screens.

**Caveat, important:** this gate compares *widget slot geometry*, not
rendered pixels. A PASS means the screen's widget rectangles match the
exe's; it does NOT prove fonts, baselines, bevel shading, clipping or
interaction match. Pixel-level comparison is a separate, missing layer.

## 2. Screen inventory — reachability and data source

From tracing `Screen` assignment sites and renderers in
`crates/cm-ui-app/src/main.rs` and `render_new.rs`. "Live?" answers
question B only.

| # | Screen | Reachable by clicking? | Renderer | Live data? | Parity state |
| --- | --- | --- | --- | --- | --- |
| 1 | `Setup` (main menu) | yes — launch state | real, `screen_setup_faithful` | chrome only (correct) | MEASURED |
| 2 | `SelectLeagues` | yes — Setup btn 0 | real, `screen_leagues_faithful` | 34 hardcoded picker slots (matches exe's fixed list) | MEASURED |
| 3 | `StartSeason` | yes — but ONLY when >1 nation selected | real, `screen_season_faithful` | derived from picker | MEASURED |
| 4 | `EnterName` | yes | real, `screen_name_faithful` | manager name | MEASURED |
| 5 | `SelectNationality` | yes | real, `screen_nationality_faithful` | LIVE (world nations) | MEASURED |
| 6 | `SelectClub` | yes | real, `screen_team_faithful` | LIVE (manageable clubs) | MEASURED |
| 7 | `ClubPreview` | yes | real, `screen_club_preview_faithful` | LIVE | PARTIALLY IMPLEMENTED — body is a "full port in progress" placeholder panel |
| 8 | `ClubTransfers` | yes | real, `screen_club_transfers_faithful` | LIVE shell, **rows empty** pending transfer-history decode | PARTIALLY IMPLEMENTED |
| 9 | `ClubFixturesTab` | yes | real, `screen_club_fixtures_faithful` | **`viewed_season` hardcoded 2001**; body empty pending fixture-record loader | PARTIALLY IMPLEMENTED |
| 10 | `News` | yes — Take Control / menu | real | LIVE (`world.news_for`) | PARTIALLY IMPLEMENTED — bottom tabs/Back/Next inert |
| 11 | `Dashboard` | yes — menu | real | LIVE | PARTIALLY IMPLEMENTED |
| 12 | `LeagueTable` | yes — division link on Dashboard | real | LIVE | PARTIALLY IMPLEMENTED |
| 13 | `PlayerProfile` | yes — squad row | real | LIVE | PARTIALLY IMPLEMENTED |
| 14 | `ClubFixtures` | yes — next-fixture panel | real | LIVE | PARTIALLY IMPLEMENTED |
| 15 | `SelectedLeagues` | yes — Game Options | real | LIVE | PARTIALLY IMPLEMENTED |
| 16 | `LatestScores` | **NO — no menu item emits cmd 0x418**; env-var only | **stub** — dispatcher builds `LatestScoresView::default()`; the live-data renderer is dead code | **NO — live rows computed then discarded** | VISIBLE UI ONLY / NO WORKING BACKEND |
| 17 | `FifaRankings` | yes — Competitions menu | **stub** — dispatcher builds `FifaRankingsView::default()` | **NO — `fifa_view_from_save` built then ignored** | VISIBLE UI ONLY / NO WORKING BACKEND |
| 18 | `WidgetPoolDebug` | **NO** — env var only, self-documented "no user path" | real | mixed | dev tool, out of scope |
| 19 | `AutoRoute{cmd}` | yes — ~20 menu items | `try_render_via_pool`; dispatchers take **default** views | **NO** — empty substrates for every one | VISIBLE UI ONLY / NO WORKING BACKEND |

### Screens the directive lists that DO NOT EXIST

`finances` · `tactics` · `staff profile` · `competition view` ·
`stadium` · `preferences/options` · `save` · `load`.

`finance` does not appear anywhere in `cm-ui-app` or `cm-render`.

## 3. FRONTEND ELEMENTS NOT BACKED BY LIVE GAME STATE

Highest priority class per the directive: **plausible-but-wrong** — the
player sees a believable screen that is not the simulation.

### 3a. PLAUSIBLE-BUT-WRONG — fabricated values that look real

These are the worst class: the player reads a number and believes it.

| Screen | Field | Current source | Correct live source |
| --- | --- | --- | --- |
| Squad (Selection view) | **Condition %** | `render_new.rs:937-947` — deterministic **hash of `person.id`**, `70 + (h % 11)`, printed as `"84%"` | `PlayerInitState.condition` (seeded `INITIAL_CONDITION` = 156, `cm-domain/src/lib.rs:1272`) |
| Squad (Other Info view) | **Condition %** | `render_new.rs:1004-1009` — same hash, duplicated | same |
| Squad (Traditional, sort by Condition) | **Condition %** | `render_new.rs:1227-1232` → `screen_club_squad_faithful.rs:1032` | same |
| Squad (all views) | **Age** | `render_new.rs:1250-1256` — when DOB is missing, **hash of `person.id`**, `17 + (h2 % 19)`, shown indistinguishably from a real age | `PlayerInitState.age: Option<u8>`. Honest fallback is the `'-'` the rest of the screen already uses |
| Squad (all views) | **Morale** | literal `"Ok"` — `render_new.rs:936`, `:1001-1003`, `:1269` | `PlayerInitState.morale` / type10 `+0x45` form-morale byte |
| `FifaRankings` | entire table | dispatcher builds `FifaRankingsView::default()` | `screens::fifa_view_from_save(&save)` — **already computed at the call site and discarded** |
| `LatestScores` | entire table | `LatestScoresView::default()` | `cm_domain::latest_scores(&save, club)` — **already computed and discarded** |
| `AutoRoute` (~20 menu items) | whole body | dispatcher default views | varies; dispatchers do not yet accept live View payloads |

The Squad-screen hashes are the highest-priority item in this report:
the screen is reachable, looks right, and every Condition and Age value
on it is invented. The real data exists in `PlayerInitState` — it is
simply not stored on `World` yet.

### 3b. Stale-constant data (wrong, but not invented)

| Screen | Field | Current source | Correct live source |
| --- | --- | --- | --- |
| Squad | Age reference date | `render_new.rs:994` — hardcoded `age_at(2001, day_of_year(2001, 8, 10))` | `RuntimeSaveGame.date` (already plumbed at `main.rs:394`) |
| Club Transfers | subtitle `"… Season 2001/2"` | `render_new.rs:1501-1502` literal | `save.date.year` |
| Club Fixtures tab | subtitle `"Season 2001/2"` | `render_new.rs:1503-1504`; also ignores the screen's own `viewed_season`, itself hardcoded `2001` at `main.rs:1618-1629` | `save.date.year` |
| Select Leagues | secondary league name column | static `match country` table — **three separate copies**: `render_new.rs:455-464`, `render_new.rs:2072-2082`, `screens.rs:1760-1770` | `World.references.club_competitions` (already queried nearby for `short_name`) |
| Select Leagues | the 26 country rows | `game_state.rs:312-334` hardcoded `[&str; 26]`, `primary_comp_id: None` | `World.core.nations`. Boot already cross-checks these literals against the DB (`main.rs:2902-2910`) and only *logs* mismatches — display still uses the literals |
| Start Season | box labels `"England 01/02"` | `game_state.rs:253-273` — base year hardcoded to `1` (=2001) | the `start_year` already passed into game creation |
| Start Season | calendar-year vs split-season | `game_state.rs:242-247` static country list | the competition record's split-season flag (code cites `FUN_00807280:0x8073c8` and does not read it) |

### 3c. Dormant — one constant away from being player-visible

`render_new.rs:1619-1701` holds a full set of hand-written
`TransferRow`s ("Zubin Anklesaria"/"Reading"/"£150K", "Ashley Vincent",
"Martin Devaney", …) behind `const SHOW_TRANSFER_DEMO_ROWS: bool =
false`. Today the player sees an empty list, which is honest. Flipping
one constant would make all of it look like real transfer activity.
Should be deleted once the transfer-history loader (`LAB_004551c0`)
lands, not left as a switch.

### 3d. Judged CORRECT — not fake, do not "fix"

Column headers, button captions, menu labels and screen titles are fixed
chrome and are right to hardcode. Also correct: `"----"` / `"-"` in the
Av R / Apps / Form columns (documented as matching the exe at t=0),
`"Unattached"` for a clubless player, and blank nation-flag cells.

## 4. LIVE BACKEND STATE WITH NO FRONTEND CONSUMER

Measured by symbol search across `cm-ui-app/src` + `cm-render/src`.
**A zero is only a defect if the original GDI game exposes it.**

| Backend state | Producer | UI consumers | Does GDI expose it? | Verdict |
| --- | --- | --- | --- | --- |
| `finance` (balance, wages, budget, board confidence) | finance tick | **0** | **Yes** — CM 01/02 has finance screens | **Genuine gap — P0** |
| `injuries` | daily physio + per-match rolls | **0** | Yes — squad/player screens show injuries | Genuine gap |
| `person_news_mailboxes` | C15.1E | **0** | Partly — career history appears on person screens | Gap, lower priority |
| `club_records` | played fixtures | **0** | Yes — club records screen | Genuine gap |
| `honours` | competition wins | 5 | Yes | Partial |
| `training` | weekly tick | 21 | Yes | Partial |
| `owner_refuse_counter` | C15.1D | **0** | **No** — internal counter | Correctly hidden; do NOT invent a screen |
| `non_promotion` / `relegation` clause bytes | C15.1B | **0** | **No** — internal | Correctly hidden |
| `scouts` | deliberately unwired | **0** | Yes | Blocked on decode, not a UI gap |

## 5. Shared primitives — the freeze list

Per the directive these must be exact BEFORE screen-by-screen work.
Status is an initial classification; each needs its own capture-measure-
compare pass before it can be marked FROZEN.

| Primitive | Module | Status |
| --- | --- | --- |
| Palette / colours | `palette.rs` | **Byte-exact port of `FUN_005CE250`** — strongest primitive in the codebase |
| Fonts | `font.rs`, `font_loader.rs`, real `Data/*.fnt` | Real bitmap fonts loaded; per-style registry (baseline, spacing, truncation) NOT yet verified |
| Panels / bevels | `packed_panel.rs`, `bevel.rs` | Ported from the widget renderer; needs per-state verification |
| Packed widget renderer | `packed_widget.rs` (`FUN_005d7aa0`) | Hot path A–F+L byte-exact; **G–K stubs pending `FUN_00403a20` + 8 palette globals** |
| Glyph blit | `packed_glyph.rs`, `glyph_blit.rs` | Ported |
| Stipples | `packed_stipples.rs` | Ported |
| Menu / dropdown | `menu_dropdown.rs`, `screen_menu_bar.rs` | Ported |
| Message box | `msgbox.rs` | Ported |
| **Scrollbar** | — | **NO canonical module exists.** Only ad-hoc wheel handling in `main.rs` + `render_fixture_rows_scrolled`. Thumb drag unimplemented. Highest-risk primitive. |
| Table / list renderer | per-screen | No shared canonical renderer |
| Tabs / selected tab | per-screen | Not verified |
| Checkbox / radio / text field | — | UNMAPPED |
| Tooltips / status bar / hover | — | UNMAPPED — unknown whether GDI has hover states at all |

## 6. Interaction fidelity — known gaps

| Behaviour | State |
| --- | --- |
| Scrollbar thumb drag | **NOT IMPLEMENTED** (wheel only) |
| Scroll wheel | implemented on fixtures tab; unknown whether GDI supports it at all — **UNKNOWN / UNCAPTURED** |
| Screen history ◀ ▶ | "not yet implemented" status message |
| News bottom tabs / Back / Next | inert |
| Menu bar on `ClubPreview` / `ClubTransfers` / `ClubFixturesTab` / `SelectClub` / `SelectNationality` | **does not accept clicks at all** |
| `SelectClub` "Next" button | deliberately inert |
| `AutoRoute` screens | no click handling; only escape is the sidebar |
| Hover / mouse-over | UNMAPPED |
| Double click / right click / keyboard / tab focus | UNMAPPED |

## 7. Dead menu routes (visible but lead nowhere)

14 items dispatch to a `"… not yet implemented"` status line, including
**Save Game (0x3fe)**, Game Settings, Hall of Fame, Restart Game,
Written History, Player & Staff Search, Manager Stats, Job Information,
Transfers, Board/FA Confidence, Control All Teams, Control Reserve Team,
Compare Players.

A further ~20 route to `AutoRoute` and paint an empty substrate.

## 8. What must happen before screen work starts

Per the directive's fixing order, and consistent with what this audit
found:

1. **Scrollbar** — recover ONE canonical GDI scrollbar (arrow geometry,
   track, thumb min size, size/position formulas, page regions, drag
   offset, clamping, row mapping). It is the most approximation-prone
   primitive and currently does not exist as a shared component.

   *First hard evidence already in hand:* the layout engine decode
   (`reports/gui_layout_engine_decode.md` §1) shows the exe reserving a
   **0x15 = 21 px scrollbar gutter** (`inner_w -= 0x15`) after calling
   `FUN_00403640(pad_x, pad_y)`, and only when
   `cols < max_columns + 1`. So the gutter width is 21 px and its
   presence is conditional — both measured from the binary, not guessed.
   `FUN_00403640` is the entry point to decode next for arrow/track/thumb
   geometry.
2. **Font/text-style registry** — from observed styles only.
3. **Finish the packed widget renderer's G–K paths** (blocked on
   `FUN_00403a20` + 8 palette globals).
4. **Pixel-level comparison layer** — the current gate compares widget
   geometry only. Without rendered-pixel diffing, "exact" is unproven.
5. Only then screen-by-screen, in the directive's order.

## 9. Open questions that need capture, not judgement

Recorded as UNKNOWN rather than guessed:

* Does the GDI build support the mouse wheel?
* Are there hover / mouse-over states?
* Scrollbar thumb minimum size and drag-offset behaviour.
* Per-screen draw order where overlaps occur.
* Button repeat behaviour on scrollbar arrows.
* Disabled-control rendering.
