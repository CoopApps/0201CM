# Two-season capture findings

Capture: `season_2001_02.jsonl` (1.6 MB, 10,559 events over ~80 min
wall clock across 2 game seasons).

Hooks armed: Group B (fixture pipeline) + Group G (RNG sampler).
Group A (year-end) and C/D/E/F remain disabled pending GDI-address
archaeology.

## Bird's-eye

| Metric | Value |
| ------ | ----- |
| Wall clock | 4,809 s (~80 min) |
| Total events | 10,559 |
| Group-B events | 9,598 |
| Distinct fixture-engine bursts | 7 (gap threshold: 60 s) |
| `schedule_getter` calls | 5 (3× `-1`, 2× `0x0`) |
| `eng_second_ctor` calls | 2 (both at boot) |
| Shared-driver calls | 90 |
| No crashes; hooks preflight-verified | ✅ |

## `schedule_getter` semantic split — `arg0` decides

| # | ms | `arg0` | `rng_at_entry` | interpretation |
| - | -- | ------ | ------------- | ---------------- |
| 1 | 28,091 | `0xFFFFFFFF` | 25,779 | boot regen |
| 2 | 1,403,034 | `0x0` | 2,116,679,561 | in-season query |
| 3 | 2,149,275 | `0xFFFFFFFF` | 11,787 | mid-season 1 regen |
| 4 | 3,813,841 | `0x0` | 2,670,083,978 | in-season query |
| 5 | 4,685,859 | `0xFFFFFFFF` | 16,714 | mid-season 2 regen |

**Two distinct call modes**:

* `arg0 = -1` — regenerate. Fresh-seeded RNG state (tiny 5-digit
  values), followed by a burst of driver/walker/perturb activity
  producing a new schedule buffer.
* `arg0 = 0`  — query / read. Large random-looking RNG state at
  entry, no regeneration burst follows.

## The 7 bursts

| # | window | dur | events | RNG delta | getter | notable driver comp_ptr |
| - | ------ | --- | ------ | --------- | ------ | ----------------------- |
| 1 | 27,882 → 34,764 | 6.9 s | 2,194 | ~4.2 B | -1 | `0xb4bfa8` (5×), 13 distinct comps — BOOT |
| 2 | 224 k → 1,534 k | 22 min | 2,128 | ~3.1 B | 0 | `0x6007a40` (8×), 15 distinct — in-season gameplay |
| 3 | 2,149 k → 2,152 k | 3.0 s | 1,552 | 212 M | -1 | **`0x6014fb0` (4×)** — mid-season regen |
| 4 | 2,222 k → 2,302 k | 80 s | 118 | 3.8 B | – | `0x0` (1×) — small in-season |
| 5 | 2,417 k → 3,903 k | 25 min | 1,874 | 625 M | 0 | `0x6007a40` (8×), 11 distinct — long in-season |
| 6 | 3,977 k → 3,995 k | 18 s | 22 | 3.1 B | – | driver-only mini-burst |
| 7 | 4,685 k → 4,808 k | 123 s | 1,710 | 2.1 B | -1 | **`0x6014fb0` (4×)** — mid-season regen |

## Novel findings the harness could not have produced

1. **The exe pre-generates future-season schedules mid-current-season.**
   The `-1` schedule_getter calls fire NOT at year-end, but at
   ~36 min (burst 3) and ~78 min (burst 7) of wall-clock. Both
   bursts regenerate the same competition (`0x6014fb0`) at
   equivalent moments in each season. Consistent with calendar-year
   league rollover (Brazil, Finland, Japan, Norway, Russia,
   Sweden, USA per [league_calendar.rs:60-90](../crates/cm-domain/src/league_calendar.rs)):
   their season boundary is in the middle of the European calendar,
   and the exe regenerates their schedule as their end approaches.
   The Rust port's `hook_year_rollover` fires only at annual
   English rollover — it will miss the calendar-year regen event
   entirely.

2. **`sub_0055F240_eng_second_ctor` fires ONLY at boot** (2 calls,
   both in the first 30 s). The ctor is not re-run at year rollover
   — the exe reuses the same object and just re-fills it via the
   schedule getter. Memory `[[eng-second-ctor-identity-confirmed]]`
   describes the ctor as identity-confirmed but does not clarify
   its lifecycle; this capture nails it as one-shot per process.

3. **`schedule_getter arg0` is a mode discriminator, not just a
   pointer.** `-1` = generate-all; `0` = query. If a Rust caller
   passes `0` expecting generation, it gets a read instead. Worth
   pinning as a semantic in the fixture module.

4. **RNG budgets per regeneration are very different**:
   * Boot regen (many leagues at once): ~4.2 B RNG steps
   * Calendar-year mid-season regen (one league?): ~210 M steps
     (burst 3)
   * "Second season" mid-season regen: ~2.1 B (burst 7 — larger,
     but includes some in-season activity because the burst
     lasted longer)

   Boot's 4.2 B step count matches the memory `[[c10-11-rng-byte-exact]]`
   report where boot fixture generation exhausts a large slice of
   the RNG stream. Mid-season regens are order-of-magnitude smaller,
   suggesting they touch one comp not many.

## What this DOESN'T tell us

* **Which specific game date each burst maps to.** No date hook was
  installed. Needed to convert wall-clock into game-clock and
  confirm the "Jan 1" hypothesis. Fixable with one more hook in a
  follow-up run.
* **Whether the FA-cup-draw path uses the same getter.** No burst
  matches the shape of a single-comp cup-round injection. Either
  cup draws go through a different function, or they landed inside
  the noisy in-season bursts (2, 5).
* **Byte-exactness vs the Rust port.** The burst RNG deltas can be
  compared to Rust once we can drive its fixture generator with the
  same trigger sequence. Deferred to the differential comparer
  tranche.

## What the harness proved

* Frida capture is stable across 80 minutes / 2 seasons / 10 k
  events, no crashes.
* The 7 verified-GDI Group B VAs fire in coherent burst patterns
  matching real game state transitions.
* The verifier preflight (`verify_manifest_vas.py`) blocks the
  DirectDraw-address class of crash from ever happening again.

## Follow-ups this suggests

1. **Add a game-date read hook** (small, cheap) so future runs
   time-stamp each burst against `date.month/day/year` — settles
   the calendar-year / FA-cup / other trigger question definitively.
2. **Resolve GDI VAs for Group A** so a follow-up capture actually
   fills the C15.1 year-end differential. That's per-function
   cross-referencing of Ghidra decompiles of the two binaries.
3. **Extend the Rust fixture-engine driver to accept a captured
   burst as input** so we can produce a byte-exact differential
   against burst 3 or burst 7 (single-comp regenerations — small
   scope, fast to iterate on).
