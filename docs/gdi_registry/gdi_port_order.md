# GDI port order (generated — executable-reachability driven)

Ranks the NOT_YET_PORTED backlog by **executable reachability**: whether the original GDI game can reach the function from a live root, via DIRECT calls or PROVEN/STRONG INDIRECT dispatch (function pointers, vtables, menu/AI/competition tables). Roots: docs/gdi_registry/gdi_roots.md.

> **Four reachability kinds** (a NEW axis, separate from the Rust `reachable` field and from whether a Rust port exists):
> - `DIRECT_REACHABLE` — reached by direct calls from a live root.
> - `INDIRECT_REACHABLE` — reached via a PROVEN/STRONG indirect dispatch edge (provenance recorded).
> - `UNRESOLVED_REACHABILITY` — address is taken (stored in a table or loaded in code) but no proven path yet; almost certainly live, not yet proven.
> - `PROBABLY_DEAD` — no direct caller and no address-taken evidence anywhere.

NOT_YET_PORTED: **1391**  ·  DIRECT **772**  ·  INDIRECT **0**  ·  UNRESOLVED **286**  ·  PROBABLY_DEAD **333**

## Backlog by subsystem (live / unresolved / probably-dead / total)

| subsystem | live | unresolved | dead | total |
|---|---|---|---|---|
| player-relationships | 101 | 14 | 12 | 127 |
| GUI | 79 | 27 | 23 | 129 |
| scouting | 64 | 7 | 2 | 73 |
| contracts | 54 | 5 | 28 | 87 |
| manager-ai | 47 | 5 | 33 | 85 |
| national-teams | 30 | 7 | 10 | 47 |
| condition-fitness | 28 | 4 | 8 | 40 |
| tick | 27 | 0 | 0 | 27 |
| tactics | 27 | 36 | 23 | 86 |
| regen | 27 | 0 | 3 | 30 |
| discipline | 26 | 3 | 9 | 38 |
| finance | 21 | 1 | 10 | 32 |
| search | 18 | 3 | 22 | 43 |
| awards | 17 | 14 | 16 | 47 |
| manager-model | 17 | 2 | 8 | 27 |
| training | 17 | 1 | 8 | 26 |
| transfer | 17 | 6 | 2 | 25 |
| fixtures | 16 | 0 | 3 | 19 |
| friendly | 15 | 4 | 16 | 35 |
| UNKNOWN | 13 | 10 | 11 | 34 |
| match | 13 | 3 | 5 | 21 |
| news | 12 | 4 | 1 | 17 |
| staff_records | 12 | 11 | 7 | 30 |
| nation | 10 | 5 | 0 | 15 |
| competition | 8 | 10 | 5 | 23 |
| squad | 7 | 0 | 2 | 9 |
| records | 5 | 3 | 2 | 10 |
| notes | 5 | 2 | 11 | 18 |
| RESOURCE | 4 | 0 | 0 | 4 |
| date | 4 | 6 | 4 | 14 |
| fifa_rankings | 4 | 0 | 0 | 4 |
| match-engine | 4 | 0 | 0 | 4 |
| club_history | 3 | 1 | 5 | 9 |
| english_cup | 3 | 19 | 0 | 22 |
| officials | 3 | 0 | 0 | 3 |
| setup | 3 | 5 | 3 | 11 |
| cash | 2 | 2 | 1 | 5 |
| english_rules | 2 | 3 | 0 | 5 |
| fifa-rankings | 2 | 0 | 0 | 2 |
| month_ratings | 2 | 2 | 2 | 6 |
| injuries | 1 | 0 | 0 | 1 |
| season-roll | 1 | 0 | 0 | 1 |
| transfers | 1 | 0 | 0 | 1 |
| coach | 0 | 2 | 2 | 4 |
| ? | 0 | 2 | 0 | 2 |
| promotion | 0 | 0 | 1 | 1 |
| english_league | 0 | 17 | 0 | 17 |
| international-comp | 0 | 37 | 35 | 72 |
| transfer-rules | 0 | 2 | 0 | 2 |
| history | 0 | 1 | 0 | 1 |

## Top 60 proven-reachable targets (direct first, then shallow indirect, many callers)

| DD VA | kind | depth | callers | subsystem | semantic | provenance |
|---|---|---|---|---|---|---|
| 0x005b85b0 | DIRECT | 0 | 1 | tick | daily AI dispatcher (staff/transfers/AI) | 0x005b85b0 |
| 0x005ea590 | DIRECT | 1 | 181 | transfers | related-club seniority gate | 0x005ea590 <- 0x005b85b0 |
| 0x0076f580 | DIRECT | 1 | 57 | news | FUN_0076f580 | 0x0076f580 <- 0x005b85b0 |
| 0x00615ae0 | DIRECT | 1 | 51 | condition-fitness | FUN_00615ae0 | 0x00615ae0 <- 0x005b85b0 |
| 0x004c6ea0 | DIRECT | 1 | 39 | RESOURCE | FUN_004c6ea0 | 0x004c6ea0 <- 0x008120d0 |
| 0x004c7010 | DIRECT | 1 | 38 | RESOURCE | FUN_004c7010 | 0x004c7010 <- 0x008120d0 |
| 0x004d7090 | DIRECT | 1 | 37 | contracts | exact staff wage formula | 0x004d7090 <- 0x005b85b0 |
| 0x007aa170 | DIRECT | 1 | 27 | GUI | FUN_007aa170 | 0x007aa170 <- 0x005b85b0 |
| 0x005274d0 | DIRECT | 1 | 26 | player-relationships | FUN_005274d0 | 0x005274d0 <- 0x008120d0 |
| 0x00531420 | DIRECT | 1 | 25 | player-relationships | FUN_00531420 | 0x00531420 <- 0x005b85b0 |
| 0x00535600 | DIRECT | 1 | 25 | tick | news-manager pacing A | 0x00535600 <- 0x005b6f10 |
| 0x00832ed0 | DIRECT | 1 | 25 | scouting | FUN_00832ed0 | 0x00832ed0 <- 0x005b85b0 |
| 0x0052df60 | DIRECT | 1 | 23 | injuries | physio rating (x87) | 0x0052df60 <- 0x005b85b0 |
| 0x0052e070 | DIRECT | 1 | 22 | player-relationships | FUN_0052e070 | 0x0052e070 <- 0x005b85b0 |
| 0x007ead30 | DIRECT | 1 | 20 | tick | news-manager pacing B | 0x007ead30 <- 0x008120d0 |
| 0x008506b0 | DIRECT | 1 | 20 | contracts | staff contract status-flag set + reaction di | 0x008506b0 <- 0x005b85b0 |
| 0x005316d0 | DIRECT | 1 | 19 | player-relationships | FUN_005316d0 | 0x005316d0 <- 0x005b85b0 |
| 0x005e8590 | DIRECT | 1 | 19 | manager-model | FUN_005e8590 | 0x005e8590 <- 0x007491e0 |
| 0x0075d410 | DIRECT | 1 | 17 | national-teams | international selection/retirement decision | 0x0075d410 <- 0x005b85b0 |
| 0x008506a0 | DIRECT | 1 | 17 | contracts | FUN_008506a0 | 0x008506a0 <- 0x005b85b0 |
| 0x00531940 | DIRECT | 1 | 15 | player-relationships | FUN_00531940 | 0x00531940 <- 0x005b85b0 |
| 0x00419ac0 | DIRECT | 1 | 14 | discipline | FUN_00419ac0 | 0x00419ac0 <- 0x005b85b0 |
| 0x0052c290 | DIRECT | 1 | 14 | player-relationships | FUN_0052c290 | 0x0052c290 <- 0x005121a0 |
| 0x005313b0 | DIRECT | 1 | 14 | player-relationships | FUN_005313b0 | 0x005313b0 <- 0x005b85b0 |
| 0x00531910 | DIRECT | 1 | 14 | player-relationships | FUN_00531910 | 0x00531910 <- 0x005b85b0 |
| 0x00790200 | DIRECT | 1 | 14 | regen | FUN_00790200 | 0x00790200 <- 0x005b85b0 |
| 0x00755580 | DIRECT | 1 | 13 | national-teams | FUN_00755580 | 0x00755580 <- 0x005b85b0 |
| 0x0084ff10 | DIRECT | 1 | 12 | contracts | staff wants-to-leave / unhappiness evaluator | 0x0084ff10 <- 0x005b85b0 |
| 0x00525ce0 | DIRECT | 1 | 11 | player-relationships | add person relationship link variant | 0x00525ce0 <- 0x005121a0 |
| 0x0082bf30 | DIRECT | 1 | 11 | scouting | Iterate all clubs to offer a player out / fi | 0x0082bf30 <- 0x005b85b0 |
| 0x00850490 | DIRECT | 1 | 11 | contracts | FUN_00850490 | 0x00850490 <- 0x005b85b0 |
| 0x00850510 | DIRECT | 1 | 11 | contracts | FUN_00850510 | 0x00850510 <- 0x005b85b0 |
| 0x005acc60 | DIRECT | 1 | 9 | friendly | FUN_005acc60 | 0x005acc60 <- 0x0074bf60 |
| 0x0041a3a0 | DIRECT | 1 | 7 | discipline | set player AWOL with morale/relationship hit | 0x0041a3a0 <- 0x005b85b0 |
| 0x00522710 | DIRECT | 1 | 7 | player-relationships | per-player attribute-bar computation | 0x00522710 <- 0x005b85b0 |
| 0x00595b90 | DIRECT | 1 | 7 | fixtures | build/fill a fixture record (date/teams/venu | 0x00595b90 <- 0x00699640 |
| 0x005e5820 | DIRECT | 1 | 7 | manager-model | FUN_005e5820 | 0x005e5820 <- 0x00699d90 |
| 0x005e5940 | DIRECT | 1 | 7 | manager-model | FUN_005e5940 | 0x005e5940 <- 0x00699d90 |
| 0x00615ab0 | DIRECT | 1 | 7 | condition-fitness | FUN_00615ab0 | 0x00615ab0 <- 0x005b85b0 |
| 0x007a9c90 | DIRECT | 1 | 7 | GUI | FUN_007a9c90 | 0x007a9c90 <- 0x005b85b0 |
| 0x00850680 | DIRECT | 1 | 7 | contracts | FUN_00850680 | 0x00850680 <- 0x005b85b0 |
| 0x008fcbe0 | DIRECT | 1 | 7 | transfer | FUN_008fcbe0 | 0x008fcbe0 <- 0x00699d90 |
| 0x004c6f50 | DIRECT | 1 | 6 | GUI | FUN_004c6f50 | 0x004c6f50 <- 0x005121a0 |
| 0x00531350 | DIRECT | 1 | 6 | player-relationships | FUN_00531350 | 0x00531350 <- 0x005b85b0 |
| 0x00531a50 | DIRECT | 1 | 6 | player-relationships | FUN_00531a50 | 0x00531a50 <- 0x005b85b0 |
| 0x004b6c70 | DIRECT | 1 | 5 | GUI | FUN_004b6c70 | 0x004b6c70 <- 0x0074bf60 |
| 0x00522fe0 | DIRECT | 1 | 5 | player-relationships | per-staff attribute computation | 0x00522fe0 <- 0x005b85b0 |
| 0x00531970 | DIRECT | 1 | 5 | player-relationships | FUN_00531970 | 0x00531970 <- 0x005b85b0 |
| 0x00531cd0 | DIRECT | 1 | 5 | player-relationships | FUN_00531cd0 | 0x00531cd0 <- 0x005b85b0 |
| 0x0075ce20 | DIRECT | 1 | 5 | national-teams | FUN_0075ce20 | 0x0075ce20 <- 0x005b85b0 |
| 0x007a4dd0 | DIRECT | 1 | 5 | search | Player value/price estimate | 0x007a4dd0 <- 0x005b85b0 |
| 0x007aed90 | DIRECT | 1 | 5 | GUI | FUN_007aed90 | 0x007aed90 <- 0x005b85b0 |
| 0x007e4940 | DIRECT | 1 | 5 | tick | post-hotseat finalize | 0x007e4940 <- 0x00803e00 |
| 0x008a0160 | DIRECT | 1 | 5 | training | effective training schedule query | 0x008a0160 <- 0x005b85b0 |
| 0x008f2900 | DIRECT | 1 | 5 | tick | background subsystem E | 0x008f2900 <- 0x005b6f10 |
| 0x0078b4c0 | DIRECT | 1 | 4 | condition-fitness | FUN_0078b4c0 | 0x0078b4c0 <- 0x00803e00 |
| 0x008224b0 | DIRECT | 1 | 4 | setup | count_network_waiting_players | 0x008224b0 <- 0x007491e0 |
| 0x00881c90 | DIRECT | 1 | 4 | tactics | register/unregister opponent tactic cache | 0x00881c90 <- 0x007491e0 |
| 0x008fc820 | DIRECT | 1 | 4 | transfer | FUN_008fc820 | 0x008fc820 <- 0x00699d90 |
| 0x00527690 | DIRECT | 1 | 3 | player-relationships | FUN_00527690 | 0x00527690 <- 0x008120d0 |

## PROBABLY_DEAD candidates (no caller, no address-taken) — review before pruning

333 functions. These have zero direct callers AND their address is never taken in code or data. Still a HINT (the static graph misses computed `call [reg]`), so review, don't auto-delete.

| DD VA | subsystem | semantic |
|---|---|---|
| 0x00401250 | GUI | FUN_00401250 |
| 0x004019f0 | GUI | FUN_004019f0 |
| 0x00401ee0 | GUI | FUN_00401ee0 |
| 0x00413340 | GUI | FUN_00413340 |
| 0x0043ff10 | GUI | FUN_0043ff10 |
| 0x00491c20 | GUI | FUN_00491c20 |
| 0x00492f00 | GUI | FUN_00492f00 |
| 0x004933b0 | GUI | FUN_004933b0 |
| 0x00493470 | GUI | FUN_00493470 |
| 0x004937b0 | GUI | FUN_004937b0 |
| 0x004c5d30 | GUI | FUN_004c5d30 |
| 0x004c6540 | GUI | FUN_004c6540 |
| 0x00503970 | GUI | FUN_00503970 |
| 0x00503a70 | GUI | FUN_00503a70 |
| 0x0050b3f0 | GUI | FUN_0050b3f0 |
| 0x0050b610 | GUI | FUN_0050b610 |
| 0x0050bea0 | GUI | FUN_0050bea0 |
| 0x005412e0 | GUI | FUN_005412e0 |
| 0x005454b0 | GUI | FUN_005454b0 |
| 0x00546820 | GUI | FUN_00546820 |
| 0x00546ca0 | GUI | FUN_00546ca0 |
| 0x005e4690 | GUI | FUN_005e4690 |
| 0x007ab3a0 | GUI | FUN_007ab3a0 |
| 0x0043fe90 | UNKNOWN | FUN_0043fe90 |
| 0x004915e0 | UNKNOWN | FUN_004915e0 |
| 0x004b62e0 | UNKNOWN | FUN_004b62e0 |
| 0x004b6440 | UNKNOWN | FUN_004b6440 |
| 0x004b6c90 | UNKNOWN | FUN_004b6c90 |
| 0x0050b790 | UNKNOWN | FUN_0050b790 |
| 0x0050b9c0 | UNKNOWN | FUN_0050b9c0 |
| 0x005467c0 | UNKNOWN | FUN_005467c0 |
| 0x00546f60 | UNKNOWN | FUN_00546f60 |
| 0x00546fc0 | UNKNOWN | FUN_00546fc0 |
| 0x005e4800 | UNKNOWN | FUN_005e4800 |
| 0x00414730 | awards | FUN_00414730 |
| 0x00414810 | awards | award vote processing from news event |
| 0x00418400 | awards | award shortlist append entry (player+rating+vote init) |
| 0x00418580 | awards | award shortlist finalize + news announcement |
| 0x004187b0 | awards | award shortlist vote simulation (RNG over staff) |
| 0x00418b40 | awards | FUN_00418b40 |
