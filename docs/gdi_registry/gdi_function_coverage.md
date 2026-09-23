# GDI function coverage (generated)

Registered functions: **5031**  ·  cited in Rust: **1015**

**Complete-picture denominator.** The registry now ingests the entire useful game-logic universe (the `.text` span `0x00401000..0x00922a85`; CRT/runtime plumbing outside the span is excluded). Of **5031** useful functions, **4933** are classified and **98** remain `UNKNOWN` (ingested, awaiting classification). Drive UNKNOWN to zero via mechanical bucketing (NON_USEFUL/FOREIGN_BREADTH/DEAD) + subsystem curation waves.

> **Read carefully.** A `PORTED_*` row derived automatically from a Rust citation is `confidence: UNVERIFIED` — it means *an exe address is cited in Rust without a 'not-implemented' marker*, NOT that the port was human-verified. Of **887** PORTED_* rows, only **329** are curated/verified; **558** are auto-cited and PENDING CURATION. Do not headline the auto number as real coverage (see the coverage-vs-fidelity antipattern).

## Classification provenance

How each row's status was decided — a file-level default is a defensible per-`.cpp` guess (the audit's group default extended across the file), NOT a per-function verification.

| provenance | count |
|---|---|
| file-level default (UNVERIFIED) | 2043 |
| curated (hand-verified) | 1636 |
| auto-cited (UNVERIFIED) | 593 |
| frontier-audit (per-fn audited) | 588 |
| auto (UNVERIFIED) | 169 |
| UNKNOWN (uningested/awaiting) | 2 |

## By status

| status | count |
|---|---|
| NOT_YET_PORTED | 1384 |
| FOREIGN_BREADTH | 1338 |
| PORTED_BEHAVIOURAL | 716 |
| OUT_OF_SCOPE | 523 |
| NON_USEFUL | 411 |
| UI_GDI_DOMAIN | 325 |
| UNKNOWN | 98 |
| PORTED_PARTIAL | 86 |
| PORTED_EXACT | 85 |
| REPLACED_BY_RUST | 65 |

ported (EXACT+BEHAVIOURAL+PARTIAL): **887** · genuine missing (NOT_YET_PORTED): **1384** · blocked: **0** · replaced/out-of-scope/ui: **913** · non-useful: **411** · foreign-breadth: **1338** · unknown: **98**

## By subsystem

| subsystem | total | ported | missing | out/ui/replaced | non-useful | unknown |
|---|---|---|---|---|---|---|
| foreign-comp | 1218 | 0 | 0 | 0 | 0 | 0 |
| GUI | 578 | 420 | 129 | 0 | 0 | 29 |
| render | 226 | 19 | 0 | 207 | 0 | 0 |
| screens | 202 | 77 | 0 | 117 | 0 | 8 |
| transfer-ai | 185 | 0 | 0 | 185 | 0 | 0 |
| match-physics | 166 | 3 | 0 | 163 | 0 | 0 |
| competition | 143 | 2 | 23 | 1 | 16 | 4 |
| player-relationships | 137 | 0 | 125 | 0 | 12 | 0 |
| scouting | 112 | 4 | 73 | 8 | 27 | 0 |
| international-comp | 107 | 7 | 72 | 0 | 28 | 0 |
| tactics | 105 | 8 | 86 | 1 | 10 | 0 |
| records | 103 | 13 | 10 | 31 | 49 | 0 |
| contracts | 101 | 4 | 87 | 0 | 10 | 0 |
| ? | 99 | 96 | 2 | 0 | 0 | 1 |
| manager-ai | 93 | 8 | 81 | 1 | 3 | 0 |
| UNKNOWN | 74 | 35 | 34 | 0 | 0 | 5 |
| awards | 74 | 0 | 47 | 0 | 23 | 0 |
| media | 66 | 0 | 0 | 66 | 0 | 0 |
| search | 64 | 0 | 43 | 0 | 21 | 0 |
| friendly | 62 | 0 | 35 | 1 | 16 | 0 |
| national-teams | 58 | 0 | 47 | 0 | 11 | 0 |
| match | 49 | 1 | 21 | 27 | 0 | 0 |
| condition-fitness | 48 | 0 | 40 | 0 | 7 | 1 |
| finance | 47 | 13 | 32 | 0 | 2 | 0 |
| regen | 46 | 3 | 30 | 0 | 13 | 0 |
| discipline | 44 | 2 | 38 | 0 | 4 | 0 |
| fixtures | 42 | 12 | 19 | 6 | 4 | 1 |
| manager-model | 38 | 4 | 27 | 0 | 7 | 0 |
| english_cup | 35 | 0 | 22 | 0 | 13 | 0 |
| cups | 34 | 25 | 0 | 0 | 0 | 0 |
| staff_records | 33 | 0 | 30 | 0 | 3 | 0 |
| training | 33 | 4 | 26 | 0 | 3 | 0 |
| match-engine | 32 | 20 | 4 | 8 | 0 | 0 |
| transfer | 31 | 0 | 25 | 0 | 6 | 0 |
| comp-stats | 29 | 0 | 0 | 0 | 0 | 29 |
| date | 28 | 11 | 14 | 0 | 3 | 0 |
| english_league | 28 | 1 | 17 | 0 | 10 | 0 |
| news | 28 | 4 | 17 | 3 | 4 | 0 |
| tick | 27 | 1 | 26 | 0 | 0 | 0 |
| nation | 26 | 6 | 15 | 1 | 4 | 0 |
| util | 20 | 0 | 0 | 0 | 20 | 0 |
| io | 20 | 0 | 0 | 20 | 0 | 0 |
| officials | 18 | 0 | 3 | 0 | 15 | 0 |
| netcode | 18 | 0 | 0 | 18 | 0 | 0 |
| notes | 18 | 0 | 18 | 0 | 0 | 0 |
| player-stats | 18 | 3 | 0 | 15 | 0 | 0 |
| transfers | 17 | 11 | 1 | 0 | 0 | 5 |
| db-load | 15 | 2 | 0 | 13 | 0 | 0 |
| plumbing | 14 | 0 | 0 | 0 | 14 | 0 |
| setup | 13 | 0 | 11 | 1 | 1 | 0 |
| RESOURCE | 12 | 6 | 4 | 0 | 0 | 2 |
| widgets | 11 | 11 | 0 | 0 | 0 | 0 |
| promotion | 11 | 10 | 1 | 0 | 0 | 0 |
| month_ratings | 11 | 0 | 6 | 0 | 5 | 0 |
| squad | 11 | 0 | 9 | 1 | 1 | 0 |
| club_history | 10 | 0 | 9 | 0 | 1 | 0 |
| game-loop | 10 | 0 | 0 | 0 | 10 | 0 |
| manager | 9 | 9 | 0 | 0 | 0 | 0 |
| view-model | 8 | 7 | 0 | 0 | 0 | 1 |
| i18n | 8 | 0 | 0 | 0 | 8 | 0 |
| match_events | 8 | 0 | 0 | 8 | 0 | 0 |
| geography | 7 | 0 | 0 | 0 | 7 | 0 |
| cash | 7 | 1 | 5 | 0 | 1 | 0 |
| coach | 7 | 0 | 4 | 0 | 3 | 0 |
| config | 7 | 0 | 0 | 0 | 7 | 0 |
| fifa_rankings | 6 | 0 | 4 | 2 | 0 | 0 |
| fifa-rankings | 6 | 4 | 2 | 0 | 0 | 0 |
| startup | 6 | 0 | 0 | 0 | 1 | 5 |
| english_rules | 5 | 0 | 5 | 0 | 0 | 0 |
| crt | 5 | 0 | 0 | 0 | 5 | 0 |
| save-load | 4 | 0 | 0 | 4 | 0 | 0 |
| player-init | 4 | 4 | 0 | 0 | 0 | 0 |
| injuries | 4 | 3 | 1 | 0 | 0 | 0 |
| group_stage | 4 | 2 | 0 | 0 | 2 | 0 |
| training_schedule | 4 | 0 | 0 | 3 | 1 | 0 |
| rng | 4 | 4 | 0 | 0 | 0 | 0 |
| editor | 3 | 0 | 0 | 0 | 0 | 3 |
| transfer-rules | 3 | 1 | 2 | 0 | 0 | 0 |
| history | 3 | 0 | 1 | 2 | 0 | 0 |
| player | 2 | 0 | 0 | 0 | 0 | 2 |
| stadium | 2 | 2 | 0 | 0 | 0 | 0 |
| season-roll | 2 | 1 | 1 | 0 | 0 | 0 |
| containers | 2 | 0 | 0 | 0 | 0 | 2 |
| national-caps | 2 | 2 | 0 | 0 | 0 | 0 |
| SOUND | 1 | 1 | 0 | 0 | 0 | 0 |
