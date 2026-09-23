# GDI function coverage (generated)

Registered functions: **5031**  ·  cited in Rust: **1008**

**Complete-picture denominator.** The registry now ingests the entire useful game-logic universe (the `.text` span `0x00401000..0x00922a85`; CRT/runtime plumbing outside the span is excluded). Of **5031** useful functions, **1095** are classified and **3936** remain `UNKNOWN` (ingested, awaiting classification). Drive UNKNOWN to zero via mechanical bucketing (NON_USEFUL/FOREIGN_BREADTH/DEAD) + subsystem curation waves.

> **Read carefully.** A `PORTED_*` row derived automatically from a Rust citation is `confidence: UNVERIFIED` — it means *an exe address is cited in Rust without a 'not-implemented' marker*, NOT that the port was human-verified. Of **861** PORTED_* rows, only **303** are curated/verified; **558** are auto-cited and PENDING CURATION. Do not headline the auto number as real coverage (see the coverage-vs-fidelity antipattern).

## By status

| status | count |
|---|---|
| UNKNOWN | 3936 |
| PORTED_BEHAVIOURAL | 702 |
| NOT_YET_PORTED | 153 |
| PORTED_EXACT | 83 |
| PORTED_PARTIAL | 76 |
| OUT_OF_SCOPE | 24 |
| REPLACED_BY_RUST | 23 |
| NON_USEFUL | 22 |
| UI_GDI_DOMAIN | 9 |
| FOREIGN_BREADTH | 3 |

ported (EXACT+BEHAVIOURAL+PARTIAL): **861** · genuine missing (NOT_YET_PORTED): **153** · blocked: **0** · replaced/out-of-scope/ui: **56** · non-useful: **22** · foreign-breadth: **3** · unknown: **3936**

## By subsystem

| subsystem | total | ported | missing | out/ui/replaced | non-useful | unknown |
|---|---|---|---|---|---|---|
| GUI | 3336 | 420 | 105 | 24 | 7 | 2777 |
| UNKNOWN | 1036 | 35 | 0 | 0 | 0 | 1001 |
| ? | 160 | 96 | 0 | 0 | 0 | 64 |
| screens | 87 | 77 | 0 | 2 | 0 | 8 |
| RESOURCE | 47 | 6 | 0 | 0 | 0 | 41 |
| match-engine | 32 | 20 | 4 | 8 | 0 | 0 |
| tick | 28 | 1 | 27 | 0 | 0 | 0 |
| cups | 25 | 25 | 0 | 0 | 0 | 0 |
| records | 25 | 13 | 0 | 12 | 0 | 0 |
| render | 21 | 19 | 0 | 2 | 0 | 0 |
| transfers | 18 | 12 | 1 | 0 | 0 | 5 |
| fixtures | 13 | 12 | 0 | 0 | 0 | 1 |
| finance | 13 | 13 | 0 | 0 | 0 | 0 |
| comp-stats | 12 | 0 | 0 | 0 | 0 | 12 |
| widgets | 11 | 11 | 0 | 0 | 0 | 0 |
| promotion | 11 | 10 | 1 | 0 | 0 | 0 |
| date | 11 | 11 | 0 | 0 | 0 | 0 |
| tactics | 11 | 7 | 4 | 0 | 0 | 0 |
| plumbing | 10 | 0 | 0 | 0 | 10 | 0 |
| manager | 9 | 9 | 0 | 0 | 0 | 0 |
| SOUND | 9 | 1 | 0 | 0 | 0 | 8 |
| view-model | 8 | 7 | 0 | 0 | 0 | 1 |
| nation | 8 | 6 | 1 | 1 | 0 | 0 |
| news | 8 | 4 | 3 | 1 | 0 | 0 |
| competition | 6 | 2 | 0 | 0 | 0 | 4 |
| fifa-rankings | 6 | 4 | 2 | 0 | 0 | 0 |
| scouting | 5 | 4 | 1 | 0 | 0 | 0 |
| startup | 5 | 0 | 0 | 0 | 0 | 5 |
| training | 5 | 4 | 1 | 0 | 0 | 0 |
| crt | 5 | 0 | 0 | 0 | 5 | 0 |
| db-load | 4 | 2 | 0 | 2 | 0 | 0 |
| save-load | 4 | 0 | 0 | 4 | 0 | 0 |
| player-init | 4 | 4 | 0 | 0 | 0 | 0 |
| injuries | 4 | 3 | 1 | 0 | 0 | 0 |
| rng | 4 | 4 | 0 | 0 | 0 | 0 |
| editor | 3 | 0 | 0 | 0 | 0 | 3 |
| contracts | 3 | 2 | 1 | 0 | 0 | 0 |
| match-physics | 3 | 3 | 0 | 0 | 0 | 0 |
| regen | 3 | 3 | 0 | 0 | 0 | 0 |
| player-stats | 3 | 3 | 0 | 0 | 0 | 0 |
| player | 2 | 0 | 0 | 0 | 0 | 2 |
| stadium | 2 | 2 | 0 | 0 | 0 | 0 |
| season-roll | 2 | 1 | 1 | 0 | 0 | 0 |
| containers | 2 | 0 | 0 | 0 | 0 | 2 |
| discipline | 2 | 2 | 0 | 0 | 0 | 0 |
| national-caps | 2 | 2 | 0 | 0 | 0 | 0 |
| FILESYSTEM | 1 | 0 | 0 | 0 | 0 | 1 |
| REGISTRY | 1 | 0 | 0 | 0 | 0 | 1 |
| match | 1 | 1 | 0 | 0 | 0 | 0 |
