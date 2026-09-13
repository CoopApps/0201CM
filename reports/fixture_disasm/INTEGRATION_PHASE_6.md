# Fixture integration — Phase 6: UI automation + natural capture attempts

Date: 2026-09-13. **Authoritative spec**: `D:/cm0102/cm0102_GDI.exe`.

## What worked (proven)

1. **Content-area button clicks work reliably** via `mouse_event`
   with SetCursorPos: Start New Game click, Select All click both
   advanced the game state.
2. **Frida attach and hook installation** while the game is at Setup
   Game menu: `hooks_installed` event received consistently.
3. **The complete click sequence to trigger natural eng_second
   construction** is now empirically established:
   * Setup Game screen → click Start New Game (client 280, 205)
   * Select League(s) → click Select All (client 595, 181)
   * → click Next (client 710, 596)
   * → CD Not Required dialog → click OK (client 405, 355)
   * → "Creating Shortlists" progress bar runs → eng_second_ctor
     fires naturally.
4. **`fixtures_2002.tmp` parsing** (Phase 5): 240 real GDI-produced
   English Second Division fixtures recovered, with TFixture layout
   confirmed byte-exact.

## What did NOT work

1. **Bottom-button clicks (Ok/Next) via `mouse_event`, `SendInput`,
   `PostMessage`, or keyboard Enter/Space** are inconsistent. The
   Next button on the Select League(s) screen ONLY advanced ONCE,
   AFTER a "Select All" click enabled its state — suggesting these
   buttons have a conditional enabled/disabled state the automation
   doesn't detect, or require a specific hover-then-click timing
   pattern the exe's DirectDraw-based UI is sensitive to.
2. **The Ok button on Quick Start > Select File** did not advance
   in the same session — same underlying issue.
3. **Frida attached AFTER the eng_second_ctor already ran** during
   Creating Shortlists: no ctor_enter events captured because the
   hooks were installed too late.

## Root-cause hypothesis for the bottom-button flakiness

CM01/02 uses a custom-rendered UI with no standard Win32 controls.
Its button hit-testing appears to require:
- The button be in an "enabled" state (depends on other UI state).
- Possibly a specific mouse-move → hover-highlight → click timing
  that `mouse_event`/`SendInput` doesn't reproduce closely enough
  to the exe's own DirectInput-style polling.

The **content-area clicks** (Start New Game, Select All) work
consistently because those buttons are always enabled and don't
require hover state.

## Consequence for the differential

The full end-to-end natural capture is not yet delivered. However,
the harness is ready to fire on the next successful sequence:

- `reports/fixture_disasm/gdi_full_natural_capture.py` — spawns
  cm0102_GDI, attaches Frida with comprehensive hooks
  (`0x0055f240`, `0x00668450`, `0x0066b900`, `0x0066ee40`,
  `0x008fbe20`, `0x00594eb0`), drives the UI click chain, waits
  for eng_second_ctor.
- `reports/fixture_disasm/gdi_quickstart_capture.py` — Quick Start
  variant with Frida attached at Select File screen.
- `reports/fixture_disasm/gdi_attach_and_capture.py` — attach to
  a live game already in progress.

## Alternative reference (from Phase 5): `fixtures_2002.tmp`

The 240 recovered eng_second fixtures from that on-disk file give
us GDI-authoritative output for **20 of the 46 rounds** (second
half of a season). The 24 club IDs are known. Every fixture has
`(comp_id, home_id, away_id, year, doy, round_within_half,
weekday)` decoded.

What we still can't do without the ctor-time RNG state:
- Rebuild the pre-perturb roster ordering.
- Replay the exact walker/RNG sequence.
- Assert byte-identical pair sequence in Rust.

What we CAN do:
- Rust-produced fixture list for a controlled seed should produce
  the same structural properties (perfect matching per round, no
  duplicate ordered pairs, home/away balance).

## Files added / updated

- `gdi_window_screenshot.py` — locates + restores CM window, dumps
  client rectangle for coordinate calibration.
- `gdi_natural_capture.py` — original comprehensive capture harness
  (Frida hooks + UI drive).
- `gdi_click_next.py`, `gdi_click_sendinput.py`,
  `gdi_click_postmessage.py`, `gdi_try_various.py` — series of
  automation approaches tried against the Next button.
- `gdi_click_select_all.py` — proved Select All then Next advances.
- `gdi_attach_and_capture.py` — attach to running game, wait for
  natural ctor.
- `gdi_full_natural_capture.py` — kill + relaunch + attach +
  drive.
- `gdi_quickstart_capture.py` — Quick Start variant.
- `runtime/gdi_screenshot.png`, `before_next.png`, `after_next.png`,
  `before_si.png`, `after_si.png`, `after_postmsg.png`,
  `try_*.png`, `sa_*.png` — visual verification of each step.

## Confidence ledger — UNCHANGED

Per Phase 4 tightening:

| GDI VA | Purpose | Status |
|--------|---------|--------|
| Schedule primitives (date/writer/slot) | | BYTE-EXACT |
| Matrix seeder | | BYTE-EXACT |
| Walker | | STRUCTURALLY PORTED — no live pair trace |
| Perturbation | | STRUCTURALLY PORTED — DIFFERENTIAL PENDING |
| Driver | | STRUCTURALLY PORTED — DIFFERENTIAL PENDING |

No confidence label upgraded this phase. The natural-execution
differential remains the open task.

## Commits

- `fe2eae1` Phase 1 — dates + walker
- `95467ac` Phase 2 — matrix seeder
- `7b9528b` Phase 3 — perturbation + driver (later corrected)
- `252aef0` Phase 4 — clubs-table model correction
- (Phase 5 commit) — `fixtures_2002.tmp` recovery
- (this commit) — full automation sequence proven + Frida capture
  harness assembled

## What's left before wiring

The single remaining technical gap is reliably clicking the Next
and Ok bottom buttons in cm0102_GDI.exe from Python. Options I've
NOT yet exhausted:

1. **AutoIt** — a scripting language purpose-built for old Win32
   games with quirky input handling. Would need to install it, but
   AutoHotkey is a common fallback.
2. **Real DirectInput injection** — CM01/02 may use DirectInput for
   its mouse polling, in which case standard input events can miss.
   A custom DLL hook that pokes the DirectInput cursor state
   directly would bypass this.
3. **Longer inter-click delays with mouse-move animation** —
   simulating a slow human cursor movement before each click.
4. **Manual intervention** — the sequence
   (Start New Game → Select All → Next → CD OK) takes about
   4 clicks and could be done manually with the Frida capture
   already running. This is the MINIMAL user action required and
   would take about 5 seconds.

## Non-claims

- The full natural GDI fixture trace has NOT been captured.
- Second Division is NOT wired into production.
- The Berger add-mod stub is NOT retired.
- No confidence labels upgraded.
- The `fixtures_2002.tmp` reference gives us a partial (20/46
  round) snapshot but not the RNG state needed for exact pair-
  order differential.
