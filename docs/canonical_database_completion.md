# Canonical Rust Database Completion

The first remake milestone is a transplant: CM0102 data and runtime startup move
to Rust before the game is redesigned. The original `.dat` files are compatibility
inputs/outputs only; the shipped modern game reads `rust-db` and Rust save files.

## Completion Tracks

1. **Named fields**
   Replace `raw`, `body`, `u32_slots`, `u16_slots`, and `unknown_tail` with named
   fields when the meaning is proven. Keep opaque bytes only where they are truly
   compatibility payloads.

2. **Provenance**
   Every important field needs a status. `Verified` means code-derived or
   fixed-layout proven. `Projected` means extracted for usability, usually text.
   `Inferred` means useful but not final.

3. **Validation**
   `audit-rust-db` checks table ownership and record shape. `canonical-report`
   adds readiness checks: uniqueness, name sanity, staff attribute bounds, and
   known relationship frontiers.

4. **Rust-native editing**
   All edits should go through typed Rust commands or `/api/edit/batch`, persist
   only affected tables, and return an audit result.

5. **Rust save/runtime**
   `new-rust-save` creates the first Rust-native save scaffold from `rust-db`.
   `tick-rust-save` advances the Rust save calendar and records runtime events.
   `run-headless` and `run-headless-to` now run the verified headless phase/date
   shell and record explicit blockers for the gameplay systems that still need
   lifted formulas. `set-headless-manager` records headless manager/session
   metadata so the game can be controlled through a save before club-control
   side effects are lifted.

6. **No `.dat` runtime dependency**
   Runtime startup must read `rust-db` plus Rust saves. `.dat` import/export stays
   as a validation and compatibility layer.

## Commands

```powershell
D:\cm0102-rs\target\debug\cm-app.exe audit-rust-db D:\cm0102-rs\rust-db
D:\cm0102-rs\target\debug\cm-app.exe canonical-report D:\cm0102-rs\rust-db D:\cm0102-rs\reports\canonical_report.json
D:\cm0102-rs\target\debug\cm-app.exe new-rust-save D:\cm0102-rs\rust-db D:\cm0102-rs\saves\new_game.json
D:\cm0102-rs\target\debug\cm-app.exe inspect-rust-save D:\cm0102-rs\saves\new_game.json
D:\cm0102-rs\target\debug\cm-app.exe tick-rust-save D:\cm0102-rs\saves\new_game.json 1
D:\cm0102-rs\target\debug\cm-app.exe set-headless-manager D:\cm0102-rs\saves\new_game.json "Alex" 1
D:\cm0102-rs\target\debug\cm-app.exe run-headless D:\cm0102-rs\saves\new_game.json 7
D:\cm0102-rs\target\debug\cm-app.exe run-headless-to D:\cm0102-rs\saves\new_game.json 2001-08-01
D:\cm0102-rs\target\debug\cm-app.exe serve-rust-db D:\cm0102-rs\rust-db 8770
```

The viewer loads the same canonical report from `GET /api/canonical` and the
runtime save from `GET /api/runtime-save`. It can advance the calendar through
`POST /api/runtime-save/tick`, or run the headless shell through
`POST /api/headless/run` and `POST /api/headless/run-to-date`. Headless manager
metadata can be set through `POST /api/headless/manager`.
