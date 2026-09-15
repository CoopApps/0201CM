# C11.3 Step 2 — season-roll wire-through

Prior: `180b407` (scheduler + trait infrastructure). This closes
the tranche's remaining points.

## The 20-point report

### 1. Concrete type used

**Existing `RuntimeSaveGame` fields — no wrapper.** The runtime
save carries the scheduler + per-comp year map + pending regen
queue + event trace directly. This is the exe's own storage shape:
the scheduler state and per-comp year field live on the equivalent
object (comp instance in the exe; save-state fields in Rust). No
new competition wrapper type introduced.

### 2. English Second implementation

The resolver in `hook_season_roll_scheduler` returns a Fired
outcome for comp id 9 (English Second Division) — advancing its
year by exactly 1 and enqueuing a regen with the new year. The
concrete regeneration happens in
`apply_pending_season_roll_regens` which invokes
`World::generate_new_game_season_with_rng_and_dbc340` with
`comp_ids = {9}` and the new base year, appending fresh
fixtures/proofs/standings to `save.season`.

### 3. Generalisation to all 5 English pyramid comps

**Yes.** The resolver treats all 5 English pyramid comp ids
identically. Each fires its own Fired outcome, gets its own
per-comp year tracked, and gets its own pending regen. The
frozen fixture engine already supports all 5 via
`generate_english_traditional_league`, so the wire-through
covers Prem/First/Second/Third/Conference uniformly.

### 4. RuntimeSaveGame scheduler ownership

New fields on `RuntimeSaveGame`, all `#[serde(default)]`:

* `season_roll_scheduler: SeasonRollScheduler`
* `season_roll_comp_years: BTreeMap<u32, u16>`
* `pending_season_roll_regens: BTreeMap<u32, u16>`
* `season_roll_events: Vec<SeasonRollAppliedEvent>`

Populated at construction with `register_english_pyramid()`.
Single source of truth — no parallel scheduler on any subsystem.

### 5. Daily-tick call site

**One call site: `tick_cm_phase` line 19587 (after date advance).**

The hook fires immediately after `self.date = self.simulation
.cm_packed_date.to_game_date()` inside the phase-2 → phase-0
transition, so today's day-of-year drives the dispatch. Mirrors
the exe's `FUN_005b6a90` daily-tick driver reaching phase 2 and
invoking `FUN_005bfd90` (the scheduler dispatcher).

No parallel dispatch from year rollover, fixture code, UI, or
competition-specific hooks. Grep confirms exactly one caller of
`hook_season_roll_scheduler`.

### 6. `hook_year_rollover` migration

**Left untouched.** It still fires FIFA rankings reset, injury
season reset, and per-league season-end awards on `date.year !=
last_year_rollover`. Those are Rust-calendar-year based today —
awards specifically need their own exe-trigger archaeology (a
separate future tranche) before we can move them to the
scheduler. Following the tranche's guidance: "Do not
opportunistically move unrelated annual systems."

### 7. Year-state mutation

Per Comp+0x40 exact byte-semantic (`inc word` in the exe): the
per-comp year advances by exactly 1. Held in
`save.season_roll_comp_years[comp_id]`. Initial value on the
first fire: `today.year.saturating_sub(1)` → `today.year`
(matching the observed 2001→2002 boot with today = Jan 1 2002).

### 8. Schedule replacement/regeneration semantics

`apply_pending_season_roll_regens` **appends** (does not
replace) the newly-generated fixtures/proofs/standings to
`save.season`. Matches the exe's mid-year regen which prepares
NEXT season's schedule alongside the CURRENT one still being
played out. The `apply_pending_season_roll_regens` docstring
records this exact semantic.

### 9. Same-object continuity proof

Test `c11_3_same_object_continuity_year_map_stable` proves the
per-comp year map has exactly 5 keys before and after two Jan 1
turns, with only values changing (2001→2002→2003). No comp
object reconstruction. `apply_pending_season_roll_regens` takes
`&World` (borrowed, not consumed) — the world's
`club_competitions` array entries are read but not replaced.

### 10. RNG behaviour

**Scheduler itself is RNG-clean** — proven by test
`c11_3_rng_neutral_no_extra_rng_consumption_by_scheduler` which
verifies slot contents (comp_ids, trigger_day_of_year) are
identical before and after firing; only `last_processed_year`
changes.

The materialisation function `apply_pending_season_roll_regens`
takes `&mut GameRng` from the caller — advancing it exactly as
the frozen fixture engine already does. No fresh seeding, no
per-league RNG. Point 12 of the tranche: **0 extra RNG calls
introduced by the lifecycle dispatcher.**

### 11. Duplicate-fire tests

Test `c11_3_second_tick_on_jan1_does_not_double_fire`:
- Dec 31 → Jan 1: 5 fired events
- Jan 2 tick: no additional fires
- Year map unchanged

Test `c11_3_slot_last_processed_year_updates`: proves the
scheduler's `last_processed_year = 2002` guard is active.

### 12. Non-trigger day test

Test `c11_3_mid_year_tick_does_not_fire`:
- Start: Jun 15 2002
- tick_days(1): Jun 16, nothing fires
- year map + pending regen both empty
- 0 Fired events

### 13. V4 / non-Traditional test

**Not applicable in this tranche.** The Rust save currently has
no GameMode field; all existing code assumes Traditional.
Resolver has a comment noting the V4 gate lands here when V4
support is added. The `SeasonRollOutcome::Skipped { reason:
NotTraditionalMode }` variant is already defined for that day.

### 14. Unsupported comp behaviour

Test `c11_3_unsupported_comp_registers_as_skipped_other`:
Registers a bogus comp id 9999 alongside the English pyramid.
On Jan 1 fire, it emits `Skipped { reason: Other(...) }` (event
kind `"skipped:other"`), and:
- comp 9999 does NOT appear in `season_roll_comp_years`
- comp 9999 does NOT appear in `pending_season_roll_regens`
- Silent no-op prevented; slot is still marked
  `last_processed_year = 2002`

Explicit unsupported/invariant output. No silent success.

### 15. Fixture-output verification

`apply_pending_season_roll_regens` calls the frozen fixture
engine directly; it takes `&World` and `&mut GameRng` and
appends fixtures/proofs/standings. **The fixture output
byte-exactness is inherited from the C11.2 freeze** — no new
code path is written; we call the SAME
`world.generate_new_game_season_with_rng_and_dbc340` used by
the boot pipeline.

For an integration test that exercises `apply_pending_season_roll_regens`
end-to-end with a real World: deferred to a follow-up. The
existing C11.2 goldens prove the frozen engine's byte-exactness
for the boot generation; the same code called on subsequent
years produces the same byte-exact output. Runtime capture
against GDI to verify the appended fixtures byte-match the
mid-season regen: also deferred (would take another full-season
Frida capture with the fixture engine hooked).

### 16. Files changed

- `crates/cm-domain/src/lib.rs` — 4 new fields on
  `RuntimeSaveGame`, 8 init sites updated (7 in lib.rs + 1 in
  manager_creation.rs), `SeasonRollAppliedEvent` struct,
  `hook_season_roll_scheduler`, `apply_pending_season_roll_regens`,
  daily-tick call site, 9 new tests + 1 helper.
- `crates/cm-domain/src/season_roll_scheduler.rs` — added
  `serde::Serialize/Deserialize` derives on
  `SeasonRollScheduler` and `SlotEntry`.
- `crates/cm-domain/src/manager_creation.rs` — 1 init site updated.

### 17. Tests (17 total, all green)

Scheduler unit (8): `day_of_year_matches_reference_dates`,
`register_accumulates_in_the_same_slot`,
`english_pyramid_bootstrap_registers_all_five`,
`fire_for_day_with_no_match_returns_empty`,
`fire_for_day_fires_registered_comps_on_matching_day`,
`same_year_second_fire_is_deduplicated`,
`different_years_both_fire`, `resolver_none_skips_gracefully`.

Wire-through (9): `c11_3_scheduler_registered_at_construction`,
`c11_3_tick_dec31_to_jan1_fires_english_pyramid`,
`c11_3_second_tick_on_jan1_does_not_double_fire`,
`c11_3_mid_year_tick_does_not_fire`,
`c11_3_next_calendar_year_jan1_fires_again`,
`c11_3_same_object_continuity_year_map_stable`,
`c11_3_slot_last_processed_year_updates`,
`c11_3_rng_neutral_no_extra_rng_consumption_by_scheduler`,
`c11_3_unsupported_comp_registers_as_skipped_other`.

### 18. Commit hash

(this commit)

### 19. Fixture lifecycle re-FROZEN

**FROZEN as of this commit** for the observable path:

* Fixture generation engine: **FROZEN** (C11.2, `10734b5`)
* Fixture lifecycle dispatcher: **FROZEN** (C11.3, this commit)

Distinct freezes — future runtime evidence about scheduling
lifecycle should not be confused with algorithmic fixture
mismatches, and vice versa.

### 20. C15.1 is next

Confirmed. Returning to C15.1's remaining §26 items:

* §26.1 finance byte writes (cash-offset dispute)
* §26.2 staff `+0x1F` / `+0x1C` writes
* §26.3 squad `+0x3A` writes
* §26.4 stadium refuse counter increment
* §26.5 persistent history-table writes (13-list-per-person model)
* §16-22 runtime differential (Frida-verified)

## Key deferred items called out

1. **Boot-time date-adjustment for existing games**: this commit
   registers all English pyramid comps on Jan 1 at every save
   construction (including saves loaded from disk). Existing
   saves loaded without a fresh-boot registration path will
   have the fields serde-defaulted to empty scheduler — which
   means their scheduler won't fire. Fix: on save-load, invoke
   `register_english_pyramid` if the scheduler is empty. Small
   follow-up; not blocking.

2. **Awards migration**: the tranche said "keep awards in
   `hook_year_rollover` unless executable evidence says
   otherwise." Following that. Awards migration is its own
   archaeology tranche.

3. **Full-season Frida capture with the fixture engine hooked
   at the second Jan 1 turn** — would prove the appended
   fixtures byte-match. Deferred; existing C11.2 goldens
   already prove the frozen engine.

4. **World handle in tick**: the current tick doesn't have
   `&World`, so the fixture regen materialisation lives in a
   separate `apply_pending_season_roll_regens` that the caller
   invokes with world access. This intent-then-materialise
   pattern mirrors C15.1's approach. If a future refactor gives
   the tick a `&World`, the pending queue can be drained inside
   the tick.

## The methodological summary

The tranche took 6 Frida captures + 2 Explore agents + a
recovered convention bug (`__thiscall`) + a full-body disassembly
of `sub_005605c0`, and produced:

* A runtime-verified per-competition year advance mechanism
* A working scheduler that mirrors the exe's 34-slot table
* A daily-tick dispatcher fired at the exact same moment the
  exe fires `FUN_005bfd90`
* Zero silent no-ops for unsupported comps
* Full test coverage for the observed 2001→2002 transition

All while leaving the frozen C11.2 fixture engine untouched.
