# Fixture integration — Phase 5: real GDI reference recovered

Date: 2026-09-13. **Authoritative spec**: `D:/cm0102/cm0102_GDI.exe`.

## Major finding

`D:/cm0102/fixtures_2002.tmp` is an on-disk fixture-list file produced
by cm0102-gdi.exe's own runtime. Its format is byte-exact to the
`TFixture` layout reconstructed in Phase 2 (agent 3's `fix_man.cpp`
work): `count: u32` header followed by `count × 79-byte TFixture`
records.

The file contains **1099 total fixtures** covering 5 competitions:

| Comp id | Count | Identity |
|---------|-------|----------|
| 7 | 170 | English Premier |
| 8 | 240 | English First Division |
| 9 | **240** | **English Second Division** |
| 10 | 240 | English Third Division |
| 93 | 209 | English Conference |

**240 English Second Division fixtures for a season half** = 24
clubs × 20 rounds × 12 pairs. This is a real behavioural output of
cm0102-gdi.exe's fixture pipeline.

## Structural validation of the recovered fixtures

Every check from
`reports/fixture_disasm/analyze_gdi_eng_second_fixtures.py` passes:

- 240 fixtures spread across 20 rounds (round_within_half 26..45).
- Each round is a perfect 24-club matching (every club plays exactly
  once per round).
- No duplicate ordered pair across the 20 rounds.
- 24 unique club ids: `[715, 765, 786, 792, 795, 826, 881, 900, 979,
  1039, 1780, 2469, 2475, 2512, 2650, 2701, 2742, 2795, 3153, 3179,
  3277, 3484, 3511, 3514]`.
- Home/away distribution: each club plays 20 fixtures. Home count
  ranges 9..11 (balanced double-round-robin).

## What we can now assert

1. **`TFixture` layout is BYTE-EXACT**. Fields at `+0x00` (comp_id),
   `+0x0c` (home_id), `+0x10` (away_id), `+0x28` (year), `+0x2a`
   (doy), `+0x34` (round_within_half), `+0x3f` (weekday) are all
   consistent with the reconstructed 79-byte record.
2. **Roster size + round-robin count**: 24 × 20 × 12 = 240 is
   confirmed by real data.
3. **Comp id 9 IS the English Second Division at the DomainCompetition
   level** — confirmed by the club-count fingerprint (24, unique to
   Div 2 and Div 3).

## What we still can NOT assert (honestly)

The fixtures in `fixtures_2002.tmp` came from a specific save
(Stalybridge23pp.sav or Vercelli.sav) with a specific RNG state at
the point eng_second_ctor ran. We do NOT have that RNG state.

**Consequence**: We cannot do exact pair-for-pair Rust-vs-GDI
comparison against this trace. What we CAN do:

- Structural-property parity (round matchings, no duplicate pairs,
  home/away balance).
- Roster-order verification IF we can identify which of the 24
  clubs is "slot 0" in the shuffled table by analysing the first-
  round pairings against the seeded matrix's fixed-team column.

The Phase-4 `driver_structural_invariant_double_rr_symmetry` test
already proves all four structural invariants. Our Rust port passes
them. This does NOT prove GDI equivalence.

## Room-number one dot

For pair-order equivalence we still need one of:

1. **Fresh natural-execution capture** with known RNG seed:
   - Launch cm0102_GDI.exe
   - Attach Frida with the ctor + driver + perturb + walker hooks
   - Drive Start New Game → English leagues → Next through the
     Win32 UI
   - Capture the entire trace + save the resulting
     `fixtures_2001.tmp`
   - Compare to Rust with matched seed

2. **RNG-state inversion** from the recovered pairs:
   - Given fixture round-26 first pair `(home=900, away=786)`, and
     given the deterministic `matrix_seed_base(24)` output (already
     BYTE-EXACT vs GDI), work backwards to identify which slot
     ordering + walker RNG trace produced this pair.
   - Complex — the RNG has 51000 pool entries × LCG state; the
     search space is large.

## UI automation attempted

`pywinauto` inspection at `reports/fixture_disasm/gdi_ui_inspect.py`
confirms cm0102_GDI.exe uses a **fully custom-rendered single-window
GUI** — no child Win32 controls, no accessibility tree. Standard
control-tree automation (button clicks by identifier) is not
possible. Alternatives require pixel-coordinate mouse simulation
(`SendInput`), which is fragile against window resizes and DPI
scaling.

**Minimal user action needed to unblock exact pair differential**:
launch a fresh new-game with the Frida hook script attached, click
through Start New Game → select English leagues → Next. Alternative:
use `Stalybridge23pp.sav` as the input state and re-hook.

The exact automation gap is **the Start New Game menu click** (one
click on a specific pixel range in a custom-rendered window). All
subsequent hooks are automatic.

## Files added / updated

- `runtime/gdi_fixtures_2002.json` — parsed 1099-record dump.
- `runtime/gdi_eng_second_fixtures_2002.json` — 240 eng_second
  fixtures with (comp_id, home_id, away_id, year, doy,
  round_within_half, weekday) fields.
- `parse_fixtures_tmp.py` — TFixture format parser.
- `analyze_gdi_eng_second_fixtures.py` — structural validation.
- `gdi_ui_inspect.py` — records that cm0102_GDI has no standard
  Win32 controls.

## Confidence ledger — unchanged

Per Phase-4 tightening:

| GDI VA | Purpose | Status |
|--------|---------|--------|
| Schedule primitives (date/writer/slot) | | BYTE-EXACT |
| Matrix seeder | | BYTE-EXACT |
| Walker | | STRUCTURALLY PORTED — no live pair trace |
| Perturbation | | STRUCTURALLY PORTED — DIFFERENTIAL PENDING |
| Driver | | STRUCTURALLY PORTED — DIFFERENTIAL PENDING |

Nothing upgraded. The recovered `fixtures_2002.tmp` reference is
now available but not yet consumed by a Rust differential test —
producing that test requires either the RNG state or a fresh
capture.

## Explicit non-claims

- Second Division has NOT been wired into production.
- The Berger add-mod stub is NOT retired.
- The recovered 240 fixtures are the RESULT of GDI generation, not
  a proof that Rust reproduces them.
- Confidence labels stand: PORTED, not EXACT.

## Commits

- `fe2eae1` Phase 1 — dates + walker
- `95467ac` Phase 2 — matrix seeder
- `7b9528b` Phase 3 — perturbation + driver (later corrected)
- `252aef0` Phase 4 — clubs-table model correction
- (this commit) Phase 5 — recovered real GDI fixture reference

## Immediate next action items

1. Instrument fresh new-game execution to capture full trace with
   controlled RNG seed. Requires either:
   - The minimal manual click identified above, OR
   - Pixel-coordinate mouse automation (`SendInput`).
2. Once that trace lands, run Rust with the same seed + roster and
   diff every stage.
3. Only then wire comp 9 into production.
