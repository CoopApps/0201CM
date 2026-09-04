# Unreferenced `build_screen_*` auto-fn routing audit (v2 — corrected)

Date: 2026-09-04 (branch `gdi-renderer-port`)
Supersedes: v1 of this file at commit `95c3b03` (see "What was wrong" below).

`crates/cm-render/src/screens_auto.rs` contains 40 `build_screen_*` fns
produced by `tools/screen_to_rust.py`. 14 are already reachable from a
ported dispatcher; **26 are dead-linked**. This report re-answers, for
each of the 26, which dispatcher would have to be ported next.

## What was wrong in v1

The v1 walker inferred a caller for each outer builder by taking the
first FUN that appeared in the outer's `xrefs.json:from` list. It did
not distinguish CALL (`FUN_004e2b80(...)`) from ADDRESS-OF
(`*param_5 = FUN_004e2b80;`). As a result, five auto-fns were tagged
`SCREEN_REGISTRY (FUN_0076ab10, news.cpp)` — which was the top
recommendation — but `FUN_0076ab10` never actually **calls** any of
them. It is a per-news-item action classifier that STORES their
addresses through `*param_5 = FUN_...;` writes; the classifier is
itself invoked from three news-list-item click handlers
(`FUN_0076c0a0`, `FUN_0076f0e0`, `FUN_0076f2f0`), which then invoke
the stored callback. So porting `FUN_0076ab10` unlocks nothing on its
own — the callback-invocation site lives elsewhere.

Two other v1 rows are also affected: v1's "AWARD_HANDLER
(`FUN_004176e0`)" and its multiple mentions of
`FUN_004e2af0` / `FUN_004ec3a0` / `FUN_008dfb10` as CONTRACT/TRANSFER
handlers — all four are themselves stored via
`*param_5 = FUN_...;` in `FUN_0076ab10` (lines 345, 469, 508, 728, 738, 759, 775
of `0076ab10.c`). They are not dispatchers; they are news-item modal
callbacks in the same layer as the five auto-fns above.

## Method (v2)

The auto-fn address (e.g. `0x00417870`) is a **callback VA** emitted
by the transliterator. Ghidra did not split most of these as their
own FUN entries; the exe reaches them by taking their address inside
a screen-setup call like

```c
FUN_007e6570(&LAB_00417870, FUN_00417e40, sVar1 == -1, 0, 0);
```

where `FUN_007e6570` is `SCRMAN_PUSH_SCREEN`, the first arg is the
widget-setup callback (our auto-fn), and the second is the event
handler. So the true **outer builder** is the FUN that contains that
`&LAB_<auto>` reference. The walker now:

1. Greps `decompiled/*.c` for `\b(?:LAB_|FUN_)<auto_va>\b`; the file
   containing that reference is the outer FUN (its name matches its
   filename).
2. Looks up the outer FUN's callers via `xrefs.json` and, for each
   call-site's containing FUN, reads its `.c` file and classifies
   every reference to the outer as CALL or ADDR-OF using regex
   `\bFUN_<outer>\s*\(` vs `\bFUN_<outer>\b(?!\s*\()`.
3. Only CALL edges are followed for higher-level tracing.

Every classification below cites `<file>.c:<line>` for verification.
Raw JSON: see `scratchpad/audit2.json` in this session.

## Per-auto-fn table

Legend for **Class**:
- `NEWS_ITEM_CALLBACK` — outer builder is only reached because its
  address is stored via `*param_5 = FUN_<outer>;` in
  `FUN_0076ab10` (news-item action classifier). Invoked by
  `FUN_0076c0a0` / `FUN_0076f0e0` / `FUN_0076f2f0`.
- `SCREEN_HANDLER_TABLE` — outer builder is called only from within a
  large handler function (e.g. `FUN_0088a850`, `FUN_00873040`) via
  what looks like a screen-id switch/jump-table.
- `DIRECT_DISPATCHER_CALL` — outer builder has a genuine CALL edge
  from a curated dispatcher (see v1 table).
- `JUMP_TABLE_ONLY` — outer builder's only caller sites live outside
  any Ghidra-identified FUN (jump-table thunks); no CALL chain
  recoverable from decompile alone.

| Auto fn | Outer FUN (via `&LAB_<auto>`) | Registration line | Level-1 CALL caller (with `.c:line`) | Level-1 ADDR-OF caller | Class |
|---|---|---|---|---|---|
| 00417870 | `FUN_00417780` | `00417780.c:20` `FUN_007e6570(&LAB_00417870,FUN_00417e40,...)` | `FUN_004176e0` @ `004176e0.c:21` `FUN_00417780(...)` | — | NEWS_ITEM_CALLBACK (004176e0 stored via `0076ab10.c:345`) |
| 00474760 | `FUN_0046ad30` | `0046ad30.c:350` `FUN_007e6570(&LAB_00474760,FUN_00474fb0,...)` | *(no CALL — only caller site 0x0046a367 lies outside any FUN — jump-table stub)* | — | JUMP_TABLE_ONLY |
| 004751b0 | `FUN_0046ad30` | `0046ad30.c:416` `FUN_007e6570(&LAB_004751b0,&LAB_004757c0,...)` | *(same as above)* | — | JUMP_TABLE_ONLY |
| 004e2c70 | `FUN_004e2b80` | `004e2b80.c:22` `FUN_007e6570(&LAB_004e2c70,&LAB_004e2f80,...)` | *(none)* | `FUN_0076ab10` @ `0076ab10.c:379` `*param_5 = FUN_004e2b80;` | NEWS_ITEM_CALLBACK |
| 004e38d0 | `FUN_004e3300` | `004e3300.c:69` `FUN_007e6570(&LAB_004e38d0,&LAB_004e42b0,...)` | `FUN_00873040` @ `00873040.c` (16×); `FUN_00874a10` @ `00874a10.c` (16×); `FUN_004e2af0`; `FUN_004ec3a0`; `FUN_008dfb10` | — | SCREEN_HANDLER_TABLE (873040/874a10 are staff-screen dispatchers; 4e2af0/4ec3a0/8dfb10 are themselves NEWS_ITEM_CALLBACKs stored in `0076ab10.c:469,508,728,759,775`) |
| 004e42b0 | `FUN_004e3300` | *(same setup call as 4e38d0 — second arg)* | *(same as 4e38d0)* | — | SCREEN_HANDLER_TABLE |
| 004e6680 | `FUN_004e4580` and `FUN_004e4960` | `004e4580.c:25`, `004e4960.c:26` `FUN_007e6570(FUN_004e4e50,&LAB_004e6680,...)` | `FUN_004e3300` @ `004e3300.c` calls both outers | — | SCREEN_HANDLER_TABLE (parent 4e3300 = same tree as 4e38d0) |
| 004ebfa0 | `FUN_004ebf60` | `004ebf60.c:7` `FUN_007e6570(&LAB_004ebfa0,&LAB_004ec240,...)` | `FUN_00873040`, `FUN_00874a10` (staff dispatchers) | — | SCREEN_HANDLER_TABLE |
| 00548560 | `FUN_00548170` | `00548170.c:40` `FUN_007e6570(&LAB_00548560,&LAB_005488f0,...)` | *(none)* | `FUN_0076ab10` (ADDR only) | NEWS_ITEM_CALLBACK |
| 005488f0 | `FUN_00548170` | *(same setup call — second arg)* | *(none)* | `FUN_0076ab10` | NEWS_ITEM_CALLBACK |
| 005dad10 | `FUN_005dab70` | `005dab70.c:7` `FUN_007e6570(&LAB_005dad10,&LAB_005db600,...)` | `FUN_00873040`, `FUN_00874a10` | — | SCREEN_HANDLER_TABLE |
| 005db600 | `FUN_005dab70` | *(same setup call — second arg)* | *(same)* | — | SCREEN_HANDLER_TABLE |
| 00697440 | `FUN_00697390` | `00697390.c:18` `FUN_007e6570(&LAB_00697440,FUN_006976a0,...)` | `FUN_006809d0` @ `006809d0.c` | — | DIRECT_DISPATCHER_CALL (6809d0 = MANAGER_SELECT_FLOW; itself called by `FUN_007491e0` GLOBAL_DISPATCHER at `007491e0.c` around `0x0074ae4c`) |
| 00697dc0 | `FUN_00697c30` | `00697c30.c:31` `FUN_007e6570(&LAB_00697dc0,FUN_00698020,...)` | *(none)* | `FUN_0076ab10` | NEWS_ITEM_CALLBACK |
| 006fd7b0 | `FUN_006fd6c0` | `006fd6c0.c:18` `FUN_007e6430(param_1,&LAB_006fd7b0,FUN_006fe420,...)` | `FUN_00699cd0`, `FUN_006fe420`, `FUN_0070d080` | — | DIRECT_DISPATCHER_CALL (0070d080 is called from `FUN_007491e0` GLOBAL_DISPATCHER at `007491e0.c` around `0x00749f67`) |
| 00701070 | `FUN_00700f20` | `00700f20.c:18` `FUN_007e6430(param_1,&LAB_00701070,&LAB_00701170,...)` | `FUN_00699d90`, `FUN_007491e0` @ `007491e0.c` | — | DIRECT_DISPATCHER_CALL (GLOBAL_DISPATCHER — **ALREADY PORTED**; wiring an arm here suffices) |
| 007cdc70 | `FUN_007cdb80` | `007cdb80.c:9` `FUN_007e6570(&LAB_007cdc70,"QSUVWj",1,FUN_0061d290,0)` | `FUN_00873040`, `FUN_00874a10` (staff dispatchers, 4× each) | — | SCREEN_HANDLER_TABLE |
| 007fb050 | `FUN_007faec0` | `007faec0.c:31` `FUN_007e6570(&LAB_007fb050,"QSUVWj",1,0,0)` | *(no callers in xrefs; site would be jump-table)* | — | JUMP_TABLE_ONLY |
| 00804020 | `FUN_00803e00` | `00803e00.c:51` `FUN_007e6430(0,&LAB_00804020,&LAB_00804340,...)` | `FUN_005b6f10` @ `005b6f10.c` | — | DIRECT_DISPATCHER_CALL (BOOT_INIT — partially ported) |
| 0088def0 | `FUN_0088a850` | `0088a850.c:497` `FUN_007e6570(&LAB_0088def0,FUN_00890360,...)` | *(none — only ADDR-OF)* | `FUN_00884700` @ `00884700.c` (4 ADDRs — tactics `FUN_0088a850` is stored, not called, by 00884700; the true invoker is inside 00884700's own dispatch table at `0x0088c254`) | SCREEN_HANDLER_TABLE (tactics — inner branch of the 13.6 KB `FUN_0088a850`; the exe reaches it by a state-id switch inside `FUN_0088a850` itself; wiring requires porting `FUN_0088a850`) |
| 00890e50 | `FUN_0088a850` | `0088a850.c:1226` `FUN_007e6570(&LAB_00890e50,&LAB_00893500,...)` | *(same as 88def0)* | *(same)* | SCREEN_HANDLER_TABLE |
| 00893500 | `FUN_0088a850` | *(same setup call — second arg)* | *(same)* | *(same)* | SCREEN_HANDLER_TABLE |
| 008a2180 | `FUN_008a20a0` | `008a20a0.c:16` `FUN_007e6570(&LAB_008a2180,&LAB_008a5030,1,0,&LAB_008a63e0)` | *(no callers in xrefs; site 0x008a61df lies outside any FUN — jump-table stub)* | — | JUMP_TABLE_ONLY |
| 008a6580 | `FUN_008a6420` | `008a6420.c:47` `FUN_007e6570(&LAB_008a6580,"QSUVWj",1,0,0)` | *(same — 008a61df external)* | — | JUMP_TABLE_ONLY |
| 008e01d0 | `FUN_008dfc20` | `008dfc20.c:21` `FUN_007e6570(&LAB_008e01d0,FUN_008e0560,...)` | *(none)* | `FUN_0076ab10` | NEWS_ITEM_CALLBACK |
| 008e9e60 | `FUN_008dfdf0` and `FUN_008e8590` | `008dfdf0.c:40`, `008e8590.c:32` `FUN_007e6570(FUN_008e8920,&LAB_008e9e60,...)` | `FUN_008e8590` is called by `FUN_00873040`, `FUN_00874a10`, `FUN_008e82b0` | `FUN_008dfdf0` addr-stored via `FUN_0076ab10` | SCREEN_HANDLER_TABLE (via the 8e8590 branch; the 8dfdf0 branch is NEWS_ITEM_CALLBACK) |

## Summary by corrected class

| Class | Auto-fn count | Path to a running screen |
|---|---:|---|
| **NEWS_ITEM_CALLBACK** (news-item modal, invoked from news-list click via `FUN_0076ab10` classifier) | 6 (417870, 4e2c70, 548560, 5488f0, 697dc0, 8e01d0) | Port `FUN_0076c0a0` (or `0076f0e0`/`0076f2f0`) + `FUN_0076ab10` — the classifier + one of its invokers. Each of the 6 outers is one arm of the classifier's per-news-type switch. |
| **SCREEN_HANDLER_TABLE** (called from a large screen dispatcher via a state-id switch) | 11 (4e38d0, 4e42b0, 4e6680, 4ebfa0, 5dad10, 5db600, 7cdc70, 88def0, 890e50, 893500, 8e9e60) | Port the outer dispatcher (`FUN_00873040` staff, `FUN_00874a10` staff-2, `FUN_0088a850` tactics, `FUN_004e3300` contract-detail). Each brings 2-3 auto-fns. |
| **DIRECT_DISPATCHER_CALL** (real CALL from an already-known routing dispatcher) | 4 (697440, 6fd7b0, 701070, 804020) | Add a dispatch arm to the ported dispatcher (`FUN_007491e0` for 701070 — cheapest; BOOT_INIT for 804020; matchday for 6fd7b0; manager-select for 697440). |
| **JUMP_TABLE_ONLY** (only caller sites live outside any Ghidra-detected FUN) | 5 (474760, 4751b0, 7fb050, 8a2180, 8a6580) | Requires jump-table reconstruction from disassembly of `0x0046a3xx` / `0x008a61xx` / `0x00469xxx` regions — not recoverable from decompile alone. |
| **Total** | 26 | |

Note: 4e38d0 / 4e42b0 / 4e6680 additionally show CALL edges from
`FUN_004e2af0`, `FUN_004ec3a0`, and `FUN_008dfb10` — but each of
those is itself a `NEWS_ITEM_CALLBACK` (address-only in `0076ab10.c`).
Their CALL edges up to a real dispatcher pass back through the news
classifier layer. Classifying them as SCREEN_HANDLER_TABLE keeps the
useful dispatcher edge (`FUN_00873040` / `FUN_00874a10`).

## Recommended next dispatcher to port

Two candidates lead:

1. **`FUN_00873040` (staff-screen dispatcher)** — a genuine CALL
   caller for **6** of the 26 auto-fns (4e38d0, 4e42b0, 4ebfa0,
   5dad10, 5db600, 7cdc70, 8e9e60 — verified via
   `00873040.c` grep hits above). Also brings its sibling
   `FUN_00874a10` for free (873040 calls 874a10 at
   `00873040.c` around `0x00873db5`). This is the highest single-fn
   unlock and does not require the news layer.

2. **`FUN_0076ab10` + one invoker (`FUN_0076c0a0`)** — unlocks the
   **6 NEWS_ITEM_CALLBACK** auto-fns (417870, 4e2c70, 548560,
   5488f0, 697dc0, 8e01d0). Requires porting two fns instead of one,
   but the fanout beyond the audit set is large (every news-response
   modal in the game routes through this layer).

Recommended pick: **`FUN_00873040` + `FUN_00874a10`** — one primary +
one trivially-adjacent, largest unlock without also having to port
the news classifier. Second-largest: news layer as (2). Cheapest
one-line change: **wire `701070` under `FUN_007491e0` GLOBAL_DISPATCHER
(already ported)** — this is the only auto-fn in the 26 whose
level-1 CALL caller is an already-ported dispatcher, and only needs a
new arm added.
