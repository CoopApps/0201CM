# GDI global data symbols (generated from data/globals.csv)

| address | meaning | Rust equivalent | lifetime | used by | confidence |
|---|---|---|---|---|---|
| DAT_00acd5bc | club record pool base (stride 0x245/581) | World.core.clubs Vec | loaded at boot | ClubView + staff_history club link | STRUCTURALLY_VERIFIED |
| DAT_00acd5c4 | person/staff pool base (stride 0x6e/110) | World.staff.type6 Vec | loaded at boot | staff_history person link + all staff views | STRUCTURALLY_VERIFIED |
| DAT_00acd5d4 | staff_history array base (stride 0x11/17) | World.references.staff_history | loaded at boot | records apps/goals attribution (FUN_007a8090) | STRUCTURALLY_VERIFIED |
| DAT_00acd57c | staff_history row count | references.staff_history.len() | loaded at boot | FUN_007a8090 boot seed loop | STRUCTURALLY_VERIFIED |
| DAT_00acd5d8 | competition display-record pool (stride 0x6b/107) | references.{club | staff | nation}_competitions | loaded at boot |
| DAT_00ac688c | club_comp pointer table (comp_id -> record) | references club_competitions + is_league | loaded at boot | FUN_004b6b30 is-league; FUN_0052e370 division gate | STRUCTURALLY_VERIFIED |
| DAT_00b4bc70 | per-nation season/calendar table (stride 0x48; +0x15 season-start day; +0x17 split-year flag) | league_calendar.rs (table builder ported; runtime resolver FUN_00652a00 NOT) | loaded at boot | season-year bucketing (FUN_00652cd0) | PARTIAL |
| DAT_00acde90 | global current date dword (low 16 = day-of-year) | RuntimeSaveGame.date / elapsed | per-tick | season key + many date gates | STRUCTURALLY_VERIFIED |
| DAT_00acde92 | global current YEAR (high 16 of the date dword) | RuntimeSaveGame.date.year | per-tick | season-year "this season" fallback | STRUCTURALLY_VERIFIED |
| DAT_00acd56c | real-record-count boundary / current game-day index | World record counts + date | boot + per-tick | virtual-id boundary (FUN_008FCBE0); date-indexed knowledge grid | PARTIAL |
| DAT_00A01E20 | distance float LUT (9-wide |  |dy|*9+|dx|) | match_engine_exe.rs DIST_LUT_00A01E20 | static | match token distance/position |
| DAT_00B4D7A4 | bearing/atan2 float LUT (dx*24+dy) | match_engine_exe.rs bearing_atan2 | static | match token bearing/reachability | BYTE_EXACT |
| DAT_009a4b28 | cumulative-days-before-month table (date pack) | exe_date.rs pack_date tables | static | date pack/unpack (FUN_00533d10) | BYTE_EXACT |
| DAT_009bbb70 | Spanish Lower Division comp id (is-league hardcode) | is_league special-case | static | FUN_004b6b30 | STRUCTURALLY_VERIFIED |
| DAT_009bba98 | French CFA comp id (is-league hardcode) | is_league special-case | static | FUN_004b6b30 | STRUCTURALLY_VERIFIED |
| DAT_009bba08 | French Lower Division comp id (is-league hardcode) | is_league special-case | static | FUN_004b6b30 | STRUCTURALLY_VERIFIED |
| dbc340 | session RNG / dbc340 context (with_session_rng closure arg) | GameRng session context | per-session | cup draws + season regen RNG | HYPOTHESIS |
