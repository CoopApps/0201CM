# GDI port order (generated — executable-reachability driven)

Ranks the NOT_YET_PORTED backlog by **executable reachability**: whether the original GDI game can reach the function from a live root, via DIRECT calls or PROVEN/STRONG INDIRECT dispatch. Roots: docs/gdi_registry/gdi_roots.md. This is a NEW axis, separate from the Rust `reachable` field and from whether a Rust port exists.

> **Five reachability kinds:**
> - `DIRECT_REACHABLE` — reached by direct calls from a live root.
> - `INDIRECT_REACHABLE` — reached via a PROVEN/STRONG indirect dispatch edge (provenance recorded).
> - `POSSIBLE_INDIRECT` — address is taken (stored in a table or loaded in code) but invocation not yet proven; probably live, prioritize for decode — NOT auto-promoted.
> - `UNRESOLVED` — no reference found anywhere; can prove neither reachable nor dead (the static graph misses computed `call [reg]`).
> - `DEAD_OR_UNREACHABLE` — positive evidence of non-use; NEVER auto-assigned from mere absence of xrefs (curated only).

NOT_YET_PORTED: **1391**  ·  DIRECT **786**  ·  INDIRECT **0**  ·  POSSIBLE_INDIRECT **273**  ·  UNRESOLVED **332**  ·  DEAD_OR_UNREACHABLE **0**

## Backlog by subsystem (live / possible / unresolved / dead / total)

| subsystem | live | possible | unresolved | dead | total |
|---|---|---|---|---|---|
| player-relationships | 101 | 14 | 12 | 0 | 127 |
| GUI | 83 | 23 | 23 | 0 | 129 |
| scouting | 64 | 7 | 2 | 0 | 73 |
| contracts | 54 | 5 | 28 | 0 | 87 |
| manager-ai | 47 | 5 | 33 | 0 | 85 |
| national-teams | 30 | 7 | 10 | 0 | 47 |
| condition-fitness | 28 | 4 | 8 | 0 | 40 |
| tick | 27 | 0 | 0 | 0 | 27 |
| tactics | 27 | 36 | 23 | 0 | 86 |
| regen | 27 | 0 | 3 | 0 | 30 |
| discipline | 26 | 3 | 9 | 0 | 38 |
| finance | 21 | 1 | 10 | 0 | 32 |
| search | 18 | 3 | 22 | 0 | 43 |
| awards | 17 | 14 | 16 | 0 | 47 |
| manager-model | 17 | 2 | 8 | 0 | 27 |
| training | 17 | 1 | 8 | 0 | 26 |
| transfer | 17 | 6 | 2 | 0 | 25 |
| fixtures | 16 | 0 | 3 | 0 | 19 |
| friendly | 15 | 4 | 16 | 0 | 35 |
| UNKNOWN | 13 | 10 | 11 | 0 | 34 |
| match | 13 | 3 | 5 | 0 | 21 |
| english_cup | 12 | 10 | 0 | 0 | 22 |
| news | 12 | 4 | 1 | 0 | 17 |
| staff_records | 12 | 11 | 7 | 0 | 30 |
| nation | 10 | 5 | 0 | 0 | 15 |
| competition | 8 | 10 | 5 | 0 | 23 |
| squad | 7 | 0 | 2 | 0 | 9 |
| records | 5 | 3 | 2 | 0 | 10 |
| notes | 5 | 2 | 11 | 0 | 18 |
| RESOURCE | 4 | 0 | 0 | 0 | 4 |
| club_history | 4 | 1 | 4 | 0 | 9 |
| date | 4 | 6 | 4 | 0 | 14 |
| fifa_rankings | 4 | 0 | 0 | 0 | 4 |
| match-engine | 4 | 0 | 0 | 0 | 4 |
| officials | 3 | 0 | 0 | 0 | 3 |
| setup | 3 | 5 | 3 | 0 | 11 |
| cash | 2 | 2 | 1 | 0 | 5 |
| english_rules | 2 | 3 | 0 | 0 | 5 |
| fifa-rankings | 2 | 0 | 0 | 0 | 2 |
| month_ratings | 2 | 2 | 2 | 0 | 6 |
| injuries | 1 | 0 | 0 | 0 | 1 |
| season-roll | 1 | 0 | 0 | 0 | 1 |
| transfers | 1 | 0 | 0 | 0 | 1 |
| coach | 0 | 2 | 2 | 0 | 4 |
| ? | 0 | 2 | 0 | 0 | 2 |
| promotion | 0 | 0 | 1 | 0 | 1 |
| english_league | 0 | 17 | 0 | 0 | 17 |
| international-comp | 0 | 37 | 35 | 0 | 72 |
| transfer-rules | 0 | 2 | 0 | 0 | 2 |
| history | 0 | 1 | 0 | 0 | 1 |

## Top 30 live targets (reachability, root type, relevance, depth, callers)

| DD VA | kind | depth | roots | callers | subsystem | relevance | semantic |
|---|---|---|---|---|---|---|---|
| 0x0076f580 | DIRECT | 1 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 57 | news | PLAYER_VISIBLE | FUN_0076f580 |
| 0x00535600 | DIRECT | 1 | BOOT;DAILY_TICK;MATCH;COMPETITION | 25 | tick | PLAYER_VISIBLE | news-manager pacing A |
| 0x007ead30 | DIRECT | 1 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 20 | tick | PLAYER_VISIBLE | news-manager pacing B |
| 0x004df980 | DIRECT | 1 | DAILY_TICK;AI;COMPETITION | 1 | contracts | PLAYER_VISIBLE | contract news build (type 0xbc1) |
| 0x004e00c0 | DIRECT | 1 | DAILY_TICK;AI;COMPETITION | 1 | contracts | PLAYER_VISIBLE | contract news build (type 0xbc2) |
| 0x0077d380 | DIRECT | 1 | DAILY_TICK;COMPETITION | 1 | notes | PLAYER_VISIBLE | FUN_0077d380 |
| 0x0076e720 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 22 | news | PLAYER_VISIBLE | FUN_0076e720 |
| 0x0076e5e0 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 8 | news | PLAYER_VISIBLE | FUN_0076e5e0 |
| 0x005349f0 | DIRECT | 2 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 4 | date | PLAYER_VISIBLE | date_format_weekday_ordinal |
| 0x005952f0 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 4 | fixtures | PLAYER_VISIBLE | create/update fixture news record |
| 0x006a0550 | DIRECT | 2 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 4 | match-engine | PLAYER_VISIBLE | stored-action event resolver |
| 0x0076f450 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 4 | news | PLAYER_VISIBLE | FUN_0076f450 |
| 0x0076e800 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 3 | news | PLAYER_VISIBLE | FUN_0076e800 |
| 0x0079f7e0 | DIRECT | 2 | BOOT;DAILY_TICK;MATCH;COMPETITION | 3 | search | PLAYER_VISIBLE | PlayerSearch player-matches-target predi |
| 0x0041a630 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 2 | discipline | PLAYER_VISIBLE | AWOL news event builder |
| 0x007e1a30 | DIRECT | 2 | BOOT;DAILY_TICK;MATCH;COMPETITION | 2 | scouting | PLAYER_VISIBLE | Compile and sort scout reports then grou |
| 0x004e06d0 | DIRECT | 2 | BOOT;DAILY_TICK;COMPETITION | 1 | contracts | PLAYER_VISIBLE | contract news build/dispatch (type 0xbb9 |
| 0x005d8c90 | DIRECT | 2 | BOOT;DAILY_TICK;COMPETITION | 1 | records | PLAYER_VISIBLE | FUN_005d8c90 |
| 0x0067ce90 | DIRECT | 2 | BOOT;DAILY_TICK;COMPETITION | 1 | news | PLAYER_VISIBLE | news_weekly_predicate_cascade |
| 0x006aae20 | DIRECT | 2 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 1 | match-engine | PLAYER_VISIBLE | per-tick tactical/commentary updater |
| 0x0075f620 | DIRECT | 2 | BOOT;DAILY_TICK;COMPETITION | 1 | national-teams | PLAYER_VISIBLE | squad-selection news item builder (CA-so |
| 0x0077de40 | DIRECT | 2 | DAILY_TICK;COMPETITION | 1 | notes | PLAYER_VISIBLE | Populate note-edit screen state |
| 0x00791630 | DIRECT | 2 | DAILY_TICK;COMPETITION | 1 | regen | PLAYER_VISIBLE | retirement/international news generation |
| 0x0076e270 | DIRECT | 3 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 13 | news | PLAYER_VISIBLE | FUN_0076e270 |
| 0x00536df0 | DIRECT | 3 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 5 | date | PLAYER_VISIBLE | date_month_short_name |
| 0x0076d860 | DIRECT | 3 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 4 | news | PLAYER_VISIBLE | FUN_0076d860 |
| 0x007a5d10 | DIRECT | 3 | BOOT;DAILY_TICK;MATCH;COMPETITION | 4 | search | PLAYER_VISIBLE | Player attribute-comparison flag builder |
| 0x00796d10 | DIRECT | 3 | BOOT;DAILY_TICK;MATCH;COMPETITION | 3 | search | PLAYER_VISIBLE | PlayerSearch configure target label and  |
| 0x005b2500 | DIRECT | 3 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 2 | friendly | PLAYER_VISIBLE | friendly news item |
| 0x00685c00 | DIRECT | 3 | DAILY_TICK;COMPETITION | 2 | manager-ai | PLAYER_VISIBLE | news event emit type 0x1778 |

## POSSIBLE_INDIRECT (address-taken, unproven) — top decode targets to promote

273 functions whose address is stored in a table / loaded in code but whose invocation is not yet proven. Decoding their dispatch mechanism is what converts them to INDIRECT_REACHABLE.

| DD VA | subsystem | data_refs | code_refs | semantic |
|---|---|---|---|---|
| 0x004c0cf0 |  | 0 | 1 | FUN_004c0cf0 |
| 0x004c5c50 |  | 0 | 63 | FUN_004c5c50 |
| 0x00492b30 | GUI | 181 | 1 | FUN_00492b30 |
| 0x00503370 | GUI | 61 | 0 | FUN_00503370 |
| 0x00503570 | GUI | 52 | 1 | FUN_00503570 |
| 0x004025e0 | GUI | 1 | 0 | FUN_004025e0 |
| 0x0043fc60 | GUI | 1 | 0 | FUN_0043fc60 |
| 0x00401000 | GUI | 0 | 1 | FUN_00401000 |
| 0x004020d0 | GUI | 0 | 1 | FUN_004020d0 |
| 0x00491830 | GUI | 0 | 1 | FUN_00491830 |
| 0x00491d80 | GUI | 0 | 1 | FUN_00491d80 |
| 0x00492d50 | GUI | 0 | 2 | FUN_00492d50 |
| 0x004b6600 | GUI | 0 | 3 | FUN_004b6600 |
| 0x004b6f80 | GUI | 0 | 48 | FUN_004b6f80 |
| 0x004ba820 | GUI | 0 | 17 | FUN_004ba820 |
| 0x004ba990 | GUI | 0 | 48 | FUN_004ba990 |
| 0x004bac50 | GUI | 0 | 2 | FUN_004bac50 |
| 0x004c5e30 | GUI | 0 | 2 | FUN_004c5e30 |
| 0x004c6920 | GUI | 0 | 1 | FUN_004c6920 |
| 0x007a97f0 | GUI | 0 | 1 | FUN_007a97f0 |
| 0x007a99a0 | GUI | 0 | 2 | FUN_007a99a0 |
| 0x007a9e60 | GUI | 0 | 1 | FUN_007a9e60 |
| 0x007aa8f0 | GUI | 0 | 3 | FUN_007aa8f0 |
| 0x007aaa90 | GUI | 0 | 1 | FUN_007aaa90 |
| 0x008475a0 | GUI | 0 | 1 | FUN_008475a0 |
| 0x00491780 | UNKNOWN | 184 | 0 | FUN_00491780 |
| 0x0050ba60 | UNKNOWN | 61 | 0 | FUN_0050ba60 |
| 0x00413330 | UNKNOWN | 4 | 0 | FUN_00413330 |
| 0x00401230 | UNKNOWN | 1 | 0 | FUN_00401230 |
| 0x00401ae0 | UNKNOWN | 1 | 0 | FUN_00401ae0 |

## UNRESOLVED (no reference found): 332  ·  DEAD_OR_UNREACHABLE (curated positive-evidence only): 0

UNRESOLVED functions have no static caller and no address-taken evidence, but the static graph cannot see computed `call [reg]`, so absence is NOT proof of death — they are candidates for targeted decode, not deletion.
