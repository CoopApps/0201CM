# GDI port order (generated — executable-reachability driven)

Ranks the NOT_YET_PORTED backlog by **executable reachability**: whether the original GDI game can reach the function from a live root, via DIRECT calls or PROVEN/STRONG INDIRECT dispatch. Roots: docs/gdi_registry/gdi_roots.md. This is a NEW axis, separate from the Rust `reachable` field and from whether a Rust port exists.

> **Five reachability kinds:**
> - `DIRECT_REACHABLE` — reached by direct calls from a live root.
> - `INDIRECT_REACHABLE` — reached via a PROVEN/STRONG indirect dispatch edge (provenance recorded).
> - `POSSIBLE_INDIRECT` — address is taken (stored in a table or loaded in code) but invocation not yet proven; probably live, prioritize for decode — NOT auto-promoted.
> - `UNRESOLVED` — no reference found anywhere; can prove neither reachable nor dead (the static graph misses computed `call [reg]`).
> - `DEAD_OR_UNREACHABLE` — positive evidence of non-use; NEVER auto-assigned from mere absence of xrefs (curated only).

NOT_YET_PORTED: **1391**  ·  DIRECT **790**  ·  INDIRECT **162**  ·  POSSIBLE_INDIRECT **213**  ·  UNRESOLVED **226**  ·  DEAD_OR_UNREACHABLE **0**

## Backlog by subsystem (live / possible / unresolved / dead / total)

| subsystem | live | possible | unresolved | dead | total |
|---|---|---|---|---|---|
| player-relationships | 107 | 9 | 11 | 0 | 127 |
| GUI | 92 | 16 | 21 | 0 | 129 |
| scouting | 70 | 2 | 1 | 0 | 73 |
| contracts | 67 | 4 | 16 | 0 | 87 |
| manager-ai | 53 | 4 | 28 | 0 | 85 |
| tactics | 51 | 29 | 6 | 0 | 86 |
| condition-fitness | 33 | 1 | 6 | 0 | 40 |
| national-teams | 33 | 5 | 9 | 0 | 47 |
| international-comp | 30 | 32 | 10 | 0 | 72 |
| discipline | 29 | 2 | 7 | 0 | 38 |
| search | 28 | 1 | 14 | 0 | 43 |
| tick | 27 | 0 | 0 | 0 | 27 |
| regen | 27 | 0 | 3 | 0 | 30 |
| manager-model | 24 | 2 | 1 | 0 | 27 |
| finance | 21 | 1 | 10 | 0 | 32 |
| awards | 19 | 14 | 14 | 0 | 47 |
| transfer | 19 | 6 | 0 | 0 | 25 |
| friendly | 18 | 4 | 13 | 0 | 35 |
| fixtures | 17 | 0 | 2 | 0 | 19 |
| training | 17 | 1 | 8 | 0 | 26 |
| notes | 16 | 0 | 2 | 0 | 18 |
| UNKNOWN | 15 | 9 | 10 | 0 | 34 |
| english_cup | 15 | 7 | 0 | 0 | 22 |
| nation | 15 | 0 | 0 | 0 | 15 |
| match | 13 | 3 | 5 | 0 | 21 |
| news | 12 | 4 | 1 | 0 | 17 |
| staff_records | 12 | 11 | 7 | 0 | 30 |
| competition | 10 | 9 | 4 | 0 | 23 |
| date | 7 | 4 | 3 | 0 | 14 |
| records | 7 | 2 | 1 | 0 | 10 |
| squad | 7 | 0 | 2 | 0 | 9 |
| setup | 5 | 4 | 2 | 0 | 11 |
| RESOURCE | 4 | 0 | 0 | 0 | 4 |
| cash | 4 | 1 | 0 | 0 | 5 |
| club_history | 4 | 1 | 4 | 0 | 9 |
| fifa_rankings | 4 | 0 | 0 | 0 | 4 |
| match-engine | 4 | 0 | 0 | 0 | 4 |
| officials | 3 | 0 | 0 | 0 | 3 |
| english_rules | 2 | 3 | 0 | 0 | 5 |
| fifa-rankings | 2 | 0 | 0 | 0 | 2 |
| transfer-rules | 2 | 0 | 0 | 0 | 2 |
| month_ratings | 2 | 2 | 2 | 0 | 6 |
| coach | 1 | 1 | 2 | 0 | 4 |
| ? | 1 | 1 | 0 | 0 | 2 |
| injuries | 1 | 0 | 0 | 0 | 1 |
| season-roll | 1 | 0 | 0 | 0 | 1 |
| transfers | 1 | 0 | 0 | 0 | 1 |
| promotion | 0 | 0 | 1 | 0 | 1 |
| english_league | 0 | 17 | 0 | 0 | 17 |
| history | 0 | 1 | 0 | 0 | 1 |

## Top 30 live targets (reachability, root type, relevance, depth, callers)

| DD VA | kind | depth | roots | callers | subsystem | relevance | semantic |
|---|---|---|---|---|---|---|---|
| 0x0076f580 | DIRECT | 1 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 57 | news | PLAYER_VISIBLE | FUN_0076f580 |
| 0x00535600 | DIRECT | 1 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 25 | tick | PLAYER_VISIBLE | news-manager pacing A |
| 0x007ead30 | DIRECT | 1 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 20 | tick | PLAYER_VISIBLE | news-manager pacing B |
| 0x004df980 | DIRECT | 1 | DAILY_TICK;AI;COMPETITION | 1 | contracts | PLAYER_VISIBLE | contract news build (type 0xbc1) |
| 0x004e00c0 | DIRECT | 1 | DAILY_TICK;AI;COMPETITION | 1 | contracts | PLAYER_VISIBLE | contract news build (type 0xbc2) |
| 0x0077d380 | DIRECT | 1 | DAILY_TICK;COMPETITION | 1 | notes | PLAYER_VISIBLE | FUN_0077d380 |
| 0x0076e720 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 22 | news | PLAYER_VISIBLE | FUN_0076e720 |
| 0x0076e5e0 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 8 | news | PLAYER_VISIBLE | FUN_0076e5e0 |
| 0x005349f0 | DIRECT | 2 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 4 | date | PLAYER_VISIBLE | date_format_weekday_ordinal |
| 0x005952f0 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 4 | fixtures | PLAYER_VISIBLE | create/update fixture news record |
| 0x006a0550 | DIRECT | 2 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 4 | match-engine | PLAYER_VISIBLE | stored-action event resolver |
| 0x0076d860 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 4 | news | PLAYER_VISIBLE | FUN_0076d860 |
| 0x0076f450 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 4 | news | PLAYER_VISIBLE | FUN_0076f450 |
| 0x005b0210 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 3 | friendly | PLAYER_VISIBLE | friendly result news |
| 0x0076e800 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 3 | news | PLAYER_VISIBLE | FUN_0076e800 |
| 0x0079f7e0 | DIRECT | 2 | BOOT;DAILY_TICK;MATCH;COMPETITION | 3 | search | PLAYER_VISIBLE | PlayerSearch player-matches-target predi |
| 0x0041a630 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 2 | discipline | PLAYER_VISIBLE | AWOL news event builder |
| 0x005afaf0 | DIRECT | 2 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 2 | friendly | PLAYER_VISIBLE | arrange friendly and emit news |
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
| 0x007a5d10 | DIRECT | 3 | BOOT;DAILY_TICK;UI;MATCH;COMPETITION | 4 | search | PLAYER_VISIBLE | Player attribute-comparison flag builder |
| 0x0076dfb0 | DIRECT | 3 | BOOT;DAILY_TICK;AI;UI;MATCH;COMPETITION | 3 | news | PLAYER_VISIBLE | FUN_0076dfb0 |

## POSSIBLE_INDIRECT (address-taken, unproven) — top decode targets to promote

213 functions whose address is stored in a table / loaded in code but whose invocation is not yet proven. Decoding their dispatch mechanism is what converts them to INDIRECT_REACHABLE.

| DD VA | subsystem | data_refs | code_refs | semantic |
|---|---|---|---|---|
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
| 0x004c6920 | GUI | 0 | 1 | FUN_004c6920 |
| 0x008475a0 | GUI | 0 | 1 | FUN_008475a0 |
| 0x00491780 | UNKNOWN | 184 | 0 | FUN_00491780 |
| 0x00413330 | UNKNOWN | 4 | 0 | FUN_00413330 |
| 0x00401230 | UNKNOWN | 1 | 0 | FUN_00401230 |
| 0x00401ae0 | UNKNOWN | 1 | 0 | FUN_00401ae0 |
| 0x004b6110 | UNKNOWN | 0 | 2 | FUN_004b6110 |
| 0x00503ca0 | UNKNOWN | 0 | 1 | FUN_00503ca0 |
| 0x007af0f0 | UNKNOWN | 0 | 1 | FUN_007af0f0 |
| 0x007af140 | UNKNOWN | 0 | 1 | FUN_007af140 |
| 0x008474a0 | UNKNOWN | 0 | 4 | FUN_008474a0 |
| 0x00418280 | awards | 0 | 1 | award shortlist serialize to save |
| 0x0074ed90 | awards | 0 | 18 | month-award result record (winner lookup) |
| 0x0074ee00 | awards | 0 | 29 | month-award winner apply + announce |
| 0x00750000 | awards | 0 | 22 | nation player-of-season award compute (year-rating |

## UNRESOLVED (no reference found): 226  ·  DEAD_OR_UNREACHABLE (curated positive-evidence only): 0

UNRESOLVED functions have no static caller and no address-taken evidence, but the static graph cannot see computed `call [reg]`, so absence is NOT proof of death — they are candidates for targeted decode, not deletion.
