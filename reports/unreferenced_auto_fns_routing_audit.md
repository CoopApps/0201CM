# Unreferenced `build_screen_*` auto-fn routing audit

Date: 2026-09-04 (branch `gdi-renderer-port`)

`crates/cm-render/src/screens_auto.rs` contains 40 `build_screen_*` fns produced by `tools/screen_to_rust.py` from `d:/cm0102-carve/analysis/screens/<addr>.json`. 14 are reachable from an already-ported dispatcher; **26 are dead-linked**. This report answers, for each of the 26, which dispatcher would have to be ported next for it to reach a caller.

## Method

For each `build_screen_<addr>` the entry address was resolved against `ghidra_out/cm0102.exe/functions.json` (containing function + offset) using the address→function bucketing derived from `functions.json` sizes. The containing function's direct callers (`xrefs.json`), then their callers up to 3 levels deep, were walked until a **known routing dispatcher** was hit (curated table below). Source-file attribution (`file_attribution.json`) is quoted verbatim when known; `?` means Ghidra did not attribute the address to a `.cpp`.

Known routing dispatchers used to classify:

| Entry | Class | Status |
|---|---|---|
| FUN_007491e0 | GLOBAL_DISPATCHER | ported (`dispatcher.rs`) |
| FUN_0074bf60 | CLUB_DISPATCHER | ported (`dispatcher.rs`) |
| FUN_00487210 / FUN_00487670 | CLUB_TOOLBAR | ported (`dispatcher_club_toolbar.rs`) |
| FUN_00745540 | MENU_BAR_BUILDER | not ported |
| FUN_007e6570 | SCRMAN_PUSH_SCREEN | primitive |
| FUN_007e6430 | SCRMAN_CB_REGISTER | primitive |
| FUN_0076ab10 | SCREEN_REGISTRY (news.cpp) | not ported |
| FUN_00873040 / FUN_00874a10 | STAFF_SCREEN_HANDLER | not ported |
| FUN_008dfb10..FUN_008e8920 | TRANSFER_HANDLER | not ported |
| FUN_00472bf0 / FUN_0046ad30 | CLUB_SCREEN_HANDLER | not ported |
| FUN_004e2af0 / FUN_004e4e50 / FUN_004ec3a0 | CONTRACT_HANDLER | not ported |
| FUN_00890360 / FUN_00894380 / FUN_0088a850 | TACTICS_HANDLER | not ported |
| FUN_008a20a0 / FUN_008a6420 | TRAINING_HANDLER | not ported |
| FUN_007faec0 | SEARCH_HANDLER | not ported |
| FUN_004176e0 | AWARD_HANDLER | not ported |
| FUN_00803e00 | SETUP_HANDLER (setup.cpp) | not ported |
| FUN_006809d0 | MANAGER_SELECT_FLOW | not ported |
| FUN_00699cd0 / FUN_00699d90 / FUN_0070d080 | MATCHDAY_FLOW | not ported |
| FUN_005b6f10 | BOOT_INIT | ported (partial) |

## Big finding

**None of the 26 unreferenced auto-fns are top-level screen entry points.** Every non-orphan case is a *mid-function offset* inside a larger `screen_screens.cpp` builder (`FUN_00xxxxxx+0xNNN`). This means the transliterator harvested inline widget-emission bursts from within already-known builders, not new screens. Wiring them requires either:

- (a) porting the OUTER builder in full — the auto-fn's block then becomes an inlined branch inside it, not a callable entry — or
- (b) restructuring the auto-fn to be dispatched independently and inserting a call into the outer builder at the correct offset.

Route (a) is what the shipped exe does. Route (b) is what the current `screens_auto.rs` shape suggests, but has no precedent in the exe's control flow.

## Per-fn routing table

| Auto fn | Entry | Containing fn (offset, .cpp) | Path to router | Router class | Port cost |
|---|---|---|---|---|---|
| build_screen_417870 | 0x00417870 | FUN_00417780+0xf0 @ award_screens.cpp | FUN_00417780 -> FUN_004176e0 | AWARD_HANDLER | M |
| build_screen_474760 | 0x00474760 | FUN_00473da0+0x9c0 @ club_screens.cpp | FUN_00473da0 -> FUN_00472bf0 | CLUB_SCREEN_HANDLER | M |
| build_screen_4751b0 | 0x004751b0 | FUN_00474fb0+0x200 @ ? | FUN_00474fb0 -> FUN_0046ad30 | CLUB_SCREEN_HANDLER | M |
| build_screen_4e2c70 | 0x004e2c70 | FUN_004e2b80+0xf0 @ ? | FUN_004e2b80 -> FUN_0076ab10 | SCREEN_REGISTRY | M |
| build_screen_4e38d0 | 0x004e38d0 | FUN_004e3300+0x5d0 @ contract_screens.cpp | FUN_004e3300 -> FUN_004e2af0 | CONTRACT_HANDLER | M |
| build_screen_4e42b0 | 0x004e42b0 | (no containing fn within 2 KB) | — | ORPHAN | S (audit) |
| build_screen_4e6680 | 0x004e6680 | FUN_004e6620+0x60 @ ? | FUN_004e6620 -> FUN_004e4e50 | CONTRACT_HANDLER | M |
| build_screen_4ebfa0 | 0x004ebfa0 | FUN_004ebf60+0x40 @ ? | FUN_004ebf60 -> FUN_00873040 | STAFF_SCREEN_HANDLER | M |
| build_screen_548560 | 0x00548560 | FUN_00548170+0x3f0 @ ? | FUN_00548170 -> FUN_0076ab10 | SCREEN_REGISTRY | M |
| build_screen_5488f0 | 0x005488f0 | FUN_00548170+0x780 @ ? | FUN_00548170 -> FUN_0076ab10 | SCREEN_REGISTRY | M |
| build_screen_5dad10 | 0x005dad10 | FUN_005dab70+0x1a0 @ ? | FUN_005dab70 -> FUN_00873040 | STAFF_SCREEN_HANDLER | M |
| build_screen_5db600 | 0x005db600 | (no containing fn within 2 KB) | — | ORPHAN | S (audit) |
| build_screen_697440 | 0x00697440 | FUN_00697390+0xb0 @ manager_screens.cpp | FUN_00697390 -> FUN_006809d0 | MANAGER_SELECT_FLOW | M |
| build_screen_697dc0 | 0x00697dc0 | FUN_00697c30+0x190 @ manager_screens.cpp | FUN_00697c30 -> FUN_0076ab10 | SCREEN_REGISTRY | M |
| build_screen_6fd7b0 | 0x006fd7b0 | FUN_006fd6c0+0xf0 @ match_screens.cpp | FUN_006fd6c0 -> FUN_00699cd0 | MATCHDAY_FLOW | M |
| build_screen_701070 | 0x00701070 | FUN_00700f20+0x150 @ match_screens.cpp | FUN_00700f20 -> FUN_00699d90 (and FUN_007491e0) | MATCHDAY_FLOW (also GLOBAL_DISPATCHER direct) | M |
| build_screen_7cdc70 | 0x007cdc70 | FUN_007cdb80+0xf0 @ ? | FUN_007cdb80 -> FUN_00873040 | STAFF_SCREEN_HANDLER | M |
| build_screen_7fb050 | 0x007fb050 | FUN_007faec0+0x190 @ search_screens.cpp | (outer has no discovered callers within 3 hops) | UNKNOWN (SEARCH_HANDLER-adjacent) | M |
| build_screen_804020 | 0x00804020 | FUN_00803e00+0x220 @ setup.cpp | FUN_00803e00 -> FUN_005b6f10 | BOOT_INIT (start-new-game path) | M |
| build_screen_88def0 | 0x0088def0 | FUN_0088dde0+0x110 @ ? | FUN_0088dde0 -> FUN_00890360 | TACTICS_HANDLER | M |
| build_screen_890e50 | 0x00890e50 | FUN_00890b30+0x320 @ ? | FUN_00890b30 -> FUN_00894380 | TACTICS_HANDLER | M |
| build_screen_893500 | 0x00893500 | (no containing fn within 2 KB) | — | ORPHAN | S (audit) |
| build_screen_8a2180 | 0x008a2180 | FUN_008a20a0+0xe0 @ ? | (outer has no discovered callers within 3 hops) | UNKNOWN (TRAINING_HANDLER-adjacent) | M |
| build_screen_8a6580 | 0x008a6580 | FUN_008a6420+0x160 @ training_screens.cpp | (outer has no discovered callers within 3 hops) | UNKNOWN (TRAINING_HANDLER-adjacent) | M |
| build_screen_8e01d0 | 0x008e01d0 | FUN_008dfdf0+0x3e0 @ transfer_screens.cpp | FUN_008dfdf0 -> FUN_0076ab10 | SCREEN_REGISTRY | M |
| build_screen_8e9e60 | 0x008e9e60 | FUN_008e8920+0x1540 @ transfer_screens.cpp | FUN_008e8920 -> FUN_008dfdf0 | TRANSFER_HANDLER | M |

Notes:
- Cmd column omitted throughout — none of these auto-fns dispatch on a top-level cmd code; they are inline branches picked by parent-builder state (screen id, mode enum, or list-row selection), so a cmd number does not apply until the outer builder is ported.
- `? ` in a `.cpp` cell means the address was not attributed by Ghidra's `file_attribution.json`; the containing-function decompile still lives under `ghidra_out/cm0102.exe/decompiled/<owner>.c`.
- The 3 `no-callers` UNKNOWN rows do have an owning function, but that owner has no xrefs — likely reachable only via vtable / callback registration through `FUN_007e6430` (`SCRMAN_CB_REGISTER`), not a direct call. Confirming that requires a callback-table sweep, not done here.

## Summary by router class

| Router class | Auto-fn count | Ported? |
|---|---:|---|
| SCREEN_REGISTRY (FUN_0076ab10, news.cpp) | 5 | no |
| STAFF_SCREEN_HANDLER (FUN_00873040 / FUN_00874a10) | 3 | no |
| ORPHAN (transliterator false positive) | 3 | n/a |
| UNKNOWN (outer builder has no discovered callers) | 3 | n/a |
| CLUB_SCREEN_HANDLER (FUN_00472bf0 / FUN_0046ad30) | 2 | no |
| CONTRACT_HANDLER (FUN_004e2af0 / FUN_004e4e50) | 2 | no |
| MATCHDAY_FLOW (FUN_00699cd0 / FUN_00699d90) | 2 | no |
| TACTICS_HANDLER (FUN_00890360 / FUN_00894380) | 2 | no |
| AWARD_HANDLER (FUN_004176e0) | 1 | no |
| MANAGER_SELECT_FLOW (FUN_006809d0) | 1 | no |
| BOOT_INIT (FUN_005b6f10) | 1 | partial |
| TRANSFER_HANDLER (FUN_008dfdf0) | 1 | no |
| **Total** | **26** | — |

## Recommended next dispatcher to port

**SCREEN_REGISTRY (`FUN_0076ab10`, news.cpp)** — highest unlock at **5 auto-fns** (`4e2c70`, `548560`, `5488f0`, `697dc0`, `8e01d0`), spread across contract, player, manager, and transfer screens. This is also the `news.cpp` screen callback registry seen as the level-1 caller of many other outer builders, so porting it also unlocks a large fanout of currently-unwired screens beyond these 26 (contract detail, player-search, transfer detail).

Runners-up:
- STAFF_SCREEN_HANDLER (3 fns) — `FUN_00873040` and `FUN_00874a10` are twin dispatchers in `staff_screens.cpp` — one port likely satisfies both.
- CONTRACT_HANDLER, TACTICS_HANDLER, MATCHDAY_FLOW, CLUB_SCREEN_HANDLER each unlock 2 fns.

## Follow-up flagged (not fixed here)

- **3 orphans** (`build_screen_4e42b0`, `5db600`, `893500`) — no containing function within 2 KB slack. Likely `screen_to_rust.py` scraped addresses that Ghidra did not identify as function-owned code (maybe data or padding, or a lifted jump-table target). Recommend either (a) rerun `capture_construct.py` with function-boundary validation, or (b) delete these stubs from `screens_auto.rs`.
- **3 UNKNOWN** (`build_screen_7fb050`, `8a2180`, `8a6580`) — owner has no direct xrefs; suspected callback-registered via `FUN_007e6430`. A follow-up sweep of the callback registry table is needed to confirm.
