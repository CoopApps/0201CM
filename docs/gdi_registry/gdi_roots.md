# GDI reachability roots (generated)

Named live-entry root sets used to compute exe-reachability. Indirect dispatch edges with a matching `root_hint` (PROVEN/STRONG) also seed their set. `indirect_edges.csv` currently holds **48** curated indirect edges.

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
| MATCH | 0x00699640 | match build | curated |
| MATCH | 0x00699d90 | match play | curated |
