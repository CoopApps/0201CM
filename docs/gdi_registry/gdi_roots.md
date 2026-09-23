# GDI reachability roots (generated)

Named live-entry root sets used to compute exe-reachability. Indirect dispatch edges with a matching `root_hint` (PROVEN/STRONG) also seed their set. `indirect_edges.csv` currently holds **372** curated indirect edges.

| root set | address | semantic | evidence |
|---|---|---|---|
| BOOT | 0x00803e00 | startup / seed (CM3_QSTART) | reports:execution-model |
| BOOT | 0x005b6940 | init / DB-load entry | reports:execution-model |
| BOOT | 0x008120d0 | new-game init | memory:start-new-game-flow |
| BOOT | 0x005121a0 | load pools | memory:start-new-game-flow |
| DAILY_TICK | 0x005b6f10 | per-day tick driver | cg:calls all 9 tick-step hooks |
| AI | 0x005b85b0 | daily AI dispatcher (game.cpp step 10) | memory:sim-findings |
| UI | 0x007491e0 | menu/widget dispatcher | memory:menu-command-tree |
| UI | 0x0074bf60 | menu/widget sub-dispatcher | memory:menu-command-tree |
| UI | 0x007e5bd0 | scrman modal run-loop (calls registered screen builder slot+0 / handler slot+8) | agent-decode:edges_uireg (entered via message pump) |
| UI | 0x007e6570 | scrman open/push-screen registrar | agent-decode:edges_uireg |
| MATCH | 0x00699640 | match build | curated |
| MATCH | 0x00699d90 | match play | curated |
| COMPETITION | 0x005b6f10 | seeded by COMPETITION_DISPATCH dispatch edge | 0x005b6f10 daily-tick loops comp array DAT_00ac688c -> vtable+0x14 per-comp date |
| COMPETITION | 0x00509570 | seeded by VTABLE dispatch edge | per-comp tick 0x00509570 -> own vtable+0x28 round-progression -> 0x0061d290 |
| COMPETITION | 0x00558c80 | seeded by VTABLE dispatch edge | eng_fa_cup ctor 0x00558c80 -> vtable 0x00957b20 slot+0x3c 0x00957b5c -> draw bui |
| COMPETITION | 0x00555e80 | seeded by VTABLE dispatch edge | eng_cc_cup ctor 0x00555e80 -> vtable 0x0095792c slot+0x3c 0x00957968 -> draw bui |
| COMPETITION | 0x00554600 | seeded by VTABLE dispatch edge | eng_auto_cup ctor 0x00554600 -> vtable 0x0095788c slot+0x3c 0x009578c8 -> draw b |
| COMPETITION | 0x0055a8f0 | seeded by VTABLE dispatch edge | eng_fa_trophy ctor 0x0055a8f0 -> vtable 0x00957bc0 slot+0x3c 0x00957bfc -> draw  |
| COMPETITION | 0x0074d830 | seeded by VTABLE dispatch edge | mini_cup ctor 0x0074d830 (called from friendly.cpp 0x005afaf0) -> vtable 0x0095b |
| COMPETITION | 0x0050ba60 | seeded by COMPETITION_DISPATCH dispatch edge | all-divisions-ready check recurses +0x10 over child divisions |
| COMPETITION | 0x0064f4f0 | seeded by COMPETITION_DISPATCH dispatch edge | FA Cup season-init(+0x8c) calls draw/bracket builder +0x3c |
| COMPETITION | 0x00556ec0 | seeded by COMPETITION_DISPATCH dispatch edge | League Cup season-init(+0x8c) calls draw/bracket builder +0x3c |
| COMPETITION | 0x005552d0 | seeded by COMPETITION_DISPATCH dispatch edge | Auto Windscreens season-init(+0x8c) calls draw/bracket builder +0x3c |
| COMPETITION | 0x00557770 | seeded by COMPETITION_DISPATCH dispatch edge | FA Trophy season-init(+0x8c) calls draw/bracket builder +0x3c |
| COMPETITION | 0x006494a0 | seeded by COMPETITION_DISPATCH dispatch edge | mini_cup season-init(+0x8c) calls draw/bracket builder +0x3c |
| COMPETITION | 0x005098e0 | seeded by COMPETITION_DISPATCH dispatch edge | 0x005098e0 cup-base dispatch -> vtable 0x0095ef98+0x20 -> 0x0091e0b0 |
