# GDI port order (generated — call-graph driven)

Ranks the NOT_YET_PORTED backlog by **exe-reachability** from the live roots (per-day tick driver 0x005b6f10 + boot/new-game/message-pump) and depth from those roots. This is the order in which porting unblocks the running game.

> **Caveat.** The static call graph misses indirect/vtable/function-pointer calls, so `cg-unreachable` is a HINT, not proof of dead code. A 0-caller unreachable function is a candidate to review, not an automatic DEAD_OR_UNREACHABLE.

NOT_YET_PORTED: **1652** · reachable from live roots: **860** · unreachable: **792** · of those 0-caller (review as dead/indirect): **213**

## Reachable backlog by subsystem (reachable / total)

| subsystem | reachable | total |
|---|---|---|
| player-relationships | 109 | 139 |
| GUI | 76 | 129 |
| scouting | 75 | 108 |
| contracts | 57 | 99 |
| manager-ai | 49 | 89 |
| national-teams | 37 | 58 |
| regen | 35 | 43 |
| condition-fitness | 34 | 48 |
| tactics | 28 | 98 |
| tick | 27 | 27 |
| search | 27 | 64 |
| discipline | 26 | 42 |
| fixtures | 26 | 29 |
| finance | 22 | 34 |
| friendly | 21 | 52 |
| transfer | 21 | 31 |
| awards | 20 | 70 |
| training | 17 | 29 |
| news | 16 | 23 |
| manager-model | 15 | 38 |
| UNKNOWN | 13 | 34 |
| match | 13 | 21 |
| competition | 11 | 28 |
| staff_records | 11 | 30 |
| nation | 10 | 19 |
| international-comp | 9 | 107 |
| squad | 8 | 11 |
| records | 5 | 12 |
| RESOURCE | 4 | 4 |
| fifa_rankings | 4 | 4 |
| match-engine | 4 | 4 |
| officials | 4 | 4 |
| notes | 4 | 18 |
| date | 3 | 14 |
| english_cup | 3 | 22 |
| setup | 3 | 12 |
| cash | 2 | 5 |
| club_history | 2 | 9 |
| english_rules | 2 | 5 |
| fifa-rankings | 2 | 2 |
| injuries | 1 | 1 |
| transfer-rules | 1 | 3 |
| season-roll | 1 | 1 |
| transfers | 1 | 1 |
| month_ratings | 1 | 6 |

## Top 60 reachable targets (shallow depth, many callers first)

| DD VA | depth | callers | subsystem | semantic |
|---|---|---|---|---|
| 0x004c6ea0 | 1 | 39 | RESOURCE | FUN_004c6ea0 |
| 0x004c7010 | 1 | 38 | RESOURCE | FUN_004c7010 |
| 0x005274d0 | 1 | 26 | player-relationships | FUN_005274d0 |
| 0x00535600 | 1 | 25 | tick | news-manager pacing A |
| 0x007ead30 | 1 | 20 | tick | news-manager pacing B |
| 0x0052c290 | 1 | 14 | player-relationships | FUN_0052c290 |
| 0x00525ce0 | 1 | 11 | player-relationships | FUN_00525ce0 |
| 0x00594370 | 1 | 7 | fixtures | FUN_00594370 |
| 0x004c6f50 | 1 | 6 | GUI | FUN_004c6f50 |
| 0x007e4940 | 1 | 5 | tick | post-hotseat finalize |
| 0x00819a40 | 1 | 5 | setup | FUN_00819a40 |
| 0x008f2900 | 1 | 5 | tick | background subsystem E |
| 0x0078b4c0 | 1 | 4 | condition-fitness | FUN_0078b4c0 |
| 0x00511ce0 | 1 | 3 | player-relationships | FUN_00511ce0 |
| 0x00527690 | 1 | 3 | player-relationships | FUN_00527690 |
| 0x006691b0 | 1 | 3 | competition | FUN_006691b0 |
| 0x00699bc0 | 1 | 3 | match | FUN_00699bc0 |
| 0x00808a70 | 1 | 3 | tick | per-seat hotseat processing |
| 0x00823210 | 1 | 3 | tick | monthly hook (date%30) |
| 0x0082d870 | 1 | 3 | scouting | FUN_0082d870 |
| 0x004135c0 | 1 | 2 | RESOURCE | FUN_004135c0 |
| 0x00413980 | 1 | 2 | tick | background subsystem F |
| 0x00418fa0 | 1 | 2 | awards | FUN_00418fa0 |
| 0x004433a0 | 1 | 2 | club_history | club_history_registry_init |
| 0x004cdef0 | 1 | 2 | contracts | FUN_004cdef0 |
| 0x0051f490 | 1 | 2 | player-relationships | FUN_0051f490 |
| 0x005232a0 | 1 | 2 | player-relationships | FUN_005232a0 |
| 0x0053f0a0 | 1 | 2 | RESOURCE | FUN_0053f0a0 |
| 0x0053fe40 | 1 | 2 | tick | background subsystem D |
| 0x00585ae0 | 1 | 2 | tick | media/board mood pass B |
| 0x00595580 | 1 | 2 | tick | fixture/news cleanup |
| 0x00599cb0 | 1 | 2 | fixtures | FUN_00599cb0 |
| 0x005ac250 | 1 | 2 | friendly | FUN_005ac250 |
| 0x005e4370 | 1 | 2 | season-roll | season-roll driver companion |
| 0x00613de0 | 1 | 2 | condition-fitness | FUN_00613de0 |
| 0x00614e90 | 1 | 2 | tick | background subsystem C |
| 0x00652820 | 1 | 2 | nation | FUN_00652820 |
| 0x00672e40 | 1 | 2 | manager-ai | FUN_00672e40 |
| 0x00674c10 | 1 | 2 | tick | manager-job lifecycle A |
| 0x00752120 | 1 | 2 | national-teams | FUN_00752120 |
| 0x00752d40 | 1 | 2 | tick | tie-participant notification |
| 0x0076dac0 | 1 | 2 | news | FUN_0076dac0 |
| 0x0078d720 | 1 | 2 | regen | FUN_0078d720 |
| 0x007de900 | 1 | 2 | scouting | FUN_007de900 |
| 0x00823ad0 | 1 | 2 | tick | media/board mood pass A |
| 0x00844790 | 1 | 2 | squad | FUN_00844790 |
| 0x00844940 | 1 | 2 | tick | manager-job lifecycle B |
| 0x00851340 | 1 | 2 | staff_records | staffhistory_init_db |
| 0x00856d50 | 1 | 2 | tick | post-comp dispatch |
| 0x0087ea70 | 1 | 2 | tactics | tactics-AI XI selection (faithful picker) |
| 0x009123a0 | 1 | 2 | tick | background subsystem B |
| 0x00419c30 | 1 | 1 | tick | manager-job lifecycle C |
| 0x0050e9d0 | 1 | 1 | player-relationships | FUN_0050e9d0 |
| 0x00510420 | 1 | 1 | player-relationships | FUN_00510420 |
| 0x0051c970 | 1 | 1 | player-relationships | FUN_0051c970 |
| 0x00523630 | 1 | 1 | player-relationships | FUN_00523630 |
| 0x0052b9b0 | 1 | 1 | player-relationships | FUN_0052b9b0 |
| 0x0052e4c0 | 1 | 1 | player-relationships | FUN_0052e4c0 |
| 0x00553aa0 | 1 | 1 | tick | manager-job lifecycle D |
| 0x00586e70 | 1 | 1 | finance | FUN_00586e70 |
