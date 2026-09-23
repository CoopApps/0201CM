# GDI port order (generated — call-graph driven)

Ranks the NOT_YET_PORTED backlog by **exe-reachability** from the live roots (per-day tick driver 0x005b6f10 + boot/new-game/message-pump) and depth from those roots. This is the order in which porting unblocks the running game.

> **Caveat.** The static call graph misses indirect/vtable/function-pointer calls, so `cg-unreachable` is a HINT, not proof of dead code. A 0-caller unreachable function is a candidate to review, not an automatic DEAD_OR_UNREACHABLE.

NOT_YET_PORTED: **1391** · reachable from live roots: **735** · unreachable: **656** · of those 0-caller (review as dead/indirect): **153**

## Reachable backlog by subsystem (reachable / total)

| subsystem | reachable | total |
|---|---|---|
| player-relationships | 101 | 127 |
| GUI | 76 | 129 |
| scouting | 59 | 73 |
| contracts | 53 | 87 |
| manager-ai | 46 | 85 |
| national-teams | 29 | 47 |
| tick | 27 | 27 |
| condition-fitness | 27 | 40 |
| tactics | 26 | 86 |
| regen | 26 | 30 |
| discipline | 23 | 38 |
| finance | 21 | 32 |
| search | 17 | 43 |
| fixtures | 16 | 19 |
| training | 16 | 26 |
| transfer | 16 | 25 |
| friendly | 15 | 35 |
| awards | 14 | 47 |
| UNKNOWN | 13 | 34 |
| manager-model | 13 | 27 |
| match | 13 | 21 |
| news | 12 | 17 |
| staff_records | 11 | 30 |
| competition | 8 | 23 |
| nation | 7 | 15 |
| squad | 6 | 9 |
| records | 5 | 10 |
| RESOURCE | 4 | 4 |
| fifa_rankings | 4 | 4 |
| match-engine | 4 | 4 |
| notes | 4 | 18 |
| date | 3 | 14 |
| english_cup | 3 | 22 |
| officials | 3 | 3 |
| cash | 2 | 5 |
| club_history | 2 | 9 |
| english_rules | 2 | 5 |
| fifa-rankings | 2 | 2 |
| setup | 2 | 11 |
| injuries | 1 | 1 |
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
| 0x00525ce0 | 1 | 11 | player-relationships | add person relationship link variant |
| 0x004c6f50 | 1 | 6 | GUI | FUN_004c6f50 |
| 0x007e4940 | 1 | 5 | tick | post-hotseat finalize |
| 0x008f2900 | 1 | 5 | tick | background subsystem E |
| 0x0078b4c0 | 1 | 4 | condition-fitness | FUN_0078b4c0 |
| 0x00527690 | 1 | 3 | player-relationships | FUN_00527690 |
| 0x006691b0 | 1 | 3 | competition | per-participant end-of-stage transfer-request plus news proc |
| 0x00699bc0 | 1 | 3 | match | FUN_00699bc0 |
| 0x00808a70 | 1 | 3 | tick | per-seat hotseat processing |
| 0x00823210 | 1 | 3 | tick | monthly hook (date%30) |
| 0x0082d870 | 1 | 3 | scouting | FUN_0082d870 |
| 0x004135c0 | 1 | 2 | RESOURCE | FUN_004135c0 |
| 0x00413980 | 1 | 2 | tick | background subsystem F |
| 0x004433a0 | 1 | 2 | club_history | club_history_registry_init |
| 0x0051f490 | 1 | 2 | player-relationships | FUN_0051f490 |
| 0x005232a0 | 1 | 2 | player-relationships | build per-person pref index table 0x24 records |
| 0x0053f0a0 | 1 | 2 | RESOURCE | FUN_0053f0a0 |
| 0x0053fe40 | 1 | 2 | tick | background subsystem D |
| 0x00585ae0 | 1 | 2 | tick | media/board mood pass B |
| 0x00595580 | 1 | 2 | tick | fixture/news cleanup |
| 0x005e4370 | 1 | 2 | season-roll | season-roll driver companion |
| 0x00613de0 | 1 | 2 | condition-fitness | injury engine ctor: CD check + load injury_history.tmp + see |
| 0x00614e90 | 1 | 2 | tick | background subsystem C |
| 0x00652820 | 1 | 2 | nation | per-nation season-boundary resolver |
| 0x00674c10 | 1 | 2 | tick | manager-job lifecycle A |
| 0x00752d40 | 1 | 2 | tick | tie-participant notification |
| 0x0078d720 | 1 | 2 | regen | regen-manager ctor + retire/regen driver init |
| 0x00823ad0 | 1 | 2 | tick | media/board mood pass A |
| 0x00844940 | 1 | 2 | tick | manager-job lifecycle B |
| 0x00851340 | 1 | 2 | staff_records | staffhistory_init_db |
| 0x00856d50 | 1 | 2 | tick | post-comp dispatch |
| 0x0087ea70 | 1 | 2 | tactics | tactics-AI XI selection (faithful picker) |
| 0x009123a0 | 1 | 2 | tick | background subsystem B |
| 0x00419c30 | 1 | 1 | tick | manager-job lifecycle C |
| 0x0050e9d0 | 1 | 1 | player-relationships | DB person-link relationship index build at load |
| 0x00510420 | 1 | 1 | player-relationships | person relationship link array builder |
| 0x0051c970 | 1 | 1 | player-relationships | assign favourite/disliked personnel names |
| 0x0052b9b0 | 1 | 1 | player-relationships | set has-contract flag and squad experience analysis |
| 0x0052e4c0 | 1 | 1 | player-relationships | prune squad relationship links to limit |
| 0x00553aa0 | 1 | 1 | tick | manager-job lifecycle D |
| 0x00586e70 | 1 | 1 | finance | all-clubs finance recompute driver |
| 0x0058fd80 | 1 | 1 | tick | manager-job lifecycle E |
| 0x0059a360 | 1 | 1 | scouting | FUN_0059a360 |
| 0x005b7f10 | 1 | 1 | tick | background subsystem A |
| 0x005b8390 | 1 | 1 | tick | event-drain predicate |
| 0x005b85b0 | 1 | 1 | tick | daily AI dispatcher (staff/transfers/AI) |
| 0x005c0d20 | 1 | 1 | tick | player-ranking summary B |
| 0x005c0f90 | 1 | 1 | tick | player-ranking summary A |
| 0x00614080 | 1 | 1 | condition-fitness | FUN_00614080 |
| 0x00699cd0 | 1 | 1 | match-engine | match pre-play pass |
| 0x0077d380 | 1 | 1 | notes | FUN_0077d380 |
| 0x0078db30 | 1 | 1 | regen | FUN_0078db30 |
| 0x0078dd80 | 1 | 1 | tick | media/scouting pass C |
| 0x007e08f0 | 1 | 1 | scouting | Weekly scout tick: evaluate watched players and emit reports |
