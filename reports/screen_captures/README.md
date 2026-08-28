# Screen captures — Frida/Unicorn ground truth vs. cm-render port

The pipeline that made the **main menu** and **News** screens pixel-exact
(see memory: `frida-capture-replay`, `emulation-pipeline`, `news-screen-geometry`)
extended to the top-40 screens beyond the main menu.

## Workflow (per screen)

1. **Capture ground truth from the exe.** Emulates the screen's draw callback in
   Unicorn, RET-stubs every other function, and records every widget the
   binary constructs.
   ```
   python tools/capture_all_screens.py <name>
   # -> reports/screen_captures/<name>.json
   ```

2. **Dump the port's geometry.** The Rust binary walks the ported screen builder
   in `cm-render` / `cm-domain` and prints the widget geometry it produces, in
   the same schema.
   ```
   cargo run -p cm-render --bin dump_screen_geometry -- <name>
   # -> reports/screen_captures/<name>.render.json
   ```
   Pass a serialized View JSON on stdin when the screen's builder needs state.

3. **Diff.** Reports mismatched fields per widget; exits 0 iff the port matches
   the exe pixel-for-pixel (in geometry).
   ```
   python tools/diff_screens.py <name>
   ```

## Screen coverage — 40 top-priority targets

Grouped as they are in `tools/capture_all_screens.py` and the `dump_screen_geometry` bin.

| Family | Screens |
|---|---|
| Validated (green today) | `news`, `competition_league` |
| Club / dashboard | `dashboard`, `club_overview`, `club_squad`, `club_reserves`, `club_youth`, `club_staff`, `club_finance`, `club_history`, `club_transfers`, `club_fixtures` |
| Player | `player_overview`, `player_stats`, `player_history`, `player_contract` |
| Tactics / match | `tactics`, `team_talk`, `match_report`, `match_stats`, `fixture_history` |
| Competition | `competition_dashboard`, `competition_fixtures`, `competition_results`, `competition_cup_draw`, `competition_history`, `competition_stats` |
| World / rankings | `world_rankings`, `fifa_rankings`, `uefa_coefficients`, `hall_of_fame`, `manager_history` |
| Transfers / scouting | `transfers_incoming`, `transfers_outgoing`, `scout_reports`, `scout_shortlist`, `latest_scores` |
| Dialogs | `go_holiday`, `contract_offer`, `job_offer`, `select_league`, `select_club` |

`python tools/capture_all_screens.py --list` shows which entries have their
draw-callback address filled in and which still need one from the Ghidra
decompile (`reports/menu_tree_scope.md` lists the ~45 known screen-builder
addresses).

## Ground rules

- The exe is the oracle. If capture and render disagree, the render is wrong.
- Every widget's geometry must be **exact** (integer L/T/R/B match) — no
  "close enough". This is `no-runtime-computed-excuse` in memory: geometry that
  the exe *computes* is not licence for the port to *approximate*.
- New screens go in `SCREENS` in both `tools/capture_all_screens.py` and the
  `dump_screen_geometry` bin. Keep the lists in sync.

## Adding a screen

1. Find the draw callback address in the Ghidra decompile
   (`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`). The screen open
   helper is usually `FUN_007e6570`; the address it registers is the callback.
2. Fill in `SCREENS[<name>] = {"addr": 0x00XXXXXX, "size": 0xNNNN, "state": [...]}`.
   `state` writes seed the fake state page to steer branch selection inside the
   callback (see `capture_construct.py` and the `competition_league` entry).
3. Run steps 1–3 above. Iterate until the diff is empty.
