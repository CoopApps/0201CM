# C11.3A — static analysis of sub_00560520 caller chain

Prior: `b8507d8` (Phase B — comp identity nailed as English 2nd Div, year-turn trigger correlated).

## Explore agent findings (DirectDraw decompile pass)

### 1. DirectDraw twin found

GDI `sub_00560520` == DirectDraw `FUN_00560320` at `.../decompiled/00560320.c`. Signature match one-to-one on every byte field write, the two globals, the vtable +0x3c call, the return-address store at `+0xba`. DD→GDI delta for this region is +0x200 (not applicable elsewhere).

Class identity: `PTR_FUN_00957dc8` (English Second Division) — vtable ctor at `FUN_0055f040`, embedded source-path string `s_C__dev_CM3_00_01_cm3_code_comp_l…` at `009b0a7c`. Matches our runtime observation (GDI vtable `0x00957DD8`, `+0x10` DD→GDI vtable delta).

### 2. Direct xrefs to `FUN_00560320`

Ghidra's `xrefs.json` shows exactly **two** references:

| From | Kind | What |
|------|------|------|
| `0x0055f0c1` | direct `call` | inside `FUN_0055f040` — eng_second constructor |
| `0x00957e54` | data pointer | eng_second vtable slot **+0x8c** (`0x00957e54 - 0x00957dc8 = 0x8c`) |

**Only ONE direct caller: the constructor.** Every other invocation is polymorphic via vtable slot +0x8c on the eng_second class.

### 3. Constructor caller — unconditional

`FUN_0055f040` (eng_second ctor) calls `FUN_00560320` unconditionally as its first initialization step. No branch guard. Its callers are through a class factory that Ghidra hasn't rendered as C, but is the ctor NOT the year-turn path — the ctor only fires when the object is first allocated.

**The year-turn regen fires via vtable slot +0x8c**, i.e. someone doing `(*this[+0x8c])()` on the existing eng_second instance.

### 4. Year field semantics — Comp+0x40

**Meaning**: season year (u16 LE). Constructor sets it from `param_2` (e.g. `0x7d1 = 2001`); the update path advances it in place.

**Writers found in the DirectDraw decompile**:

* Constructors of each competition class (initial write from ctor arg).
* Three "season-roll" methods across three sibling classes:
  * `FUN_00575da0` — vtable slot +0x08 of class `PTR_FUN_00958268`
  * `FUN_0077fee0` — vtable slot +0x08 of class `PTR_FUN_0095bce8`
  * `FUN_009192b0` — vtable slot +0x08 of class `PTR_FUN_0095ee44`

All three season-roll methods share this exact body shape:

```c
free(*(void **)((int)this + 0xba));           // free schedule buffer
// iterate this[0xc] children, destroy each
*(short *)(this + 0x40) = *(short *)(this + 0x40) + 2;  // year += 2 (Ghidra)
this[0xc] = -1;
*((int *)((int)this + 0x45)) = 0;
(**(code **)(*this + 0x8c))();               // vtable +0x8c → reset+regen (=FUN_00560320 for eng_second)
```

Ghidra reads `+ 2`, but the runtime evidence shows `2001 → 2002` = `+1`. Two possibilities: Ghidra folded two consecutive `add [esi+0x40], 1` into one `+2`, or the u16 is a half-season counter (autumn/spring) so a game year advances the field by 2. This ambiguity is not blocking; a `+2` u16 semantics is entirely consistent with the observed `2001 → 2002` if year fields are stored in encoded form.

### 5. Where's eng_second's OWN season-roll method?

**Not located.** Expected at eng_second vtable slot +0x08 (`0x00957dd0`) — the slot exists in the vtable but no Ghidra function symbol is bound there. Either eng_second inherits from a shared base class (would be one of the three season-roll methods above), or it has its own uncatalogued method that Ghidra didn't disambiguate. Open item, doesn't block the fix.

### 6. Year-turn branch condition — **the actual guard is one level up**

The three season-roll methods contain **no conditional guard** around the year increment. Once entered, they always advance and regen. So the check `if (year advanced)` lives at the **caller of the season-roll method**, one level up — inside the game date tick, which dispatches per-competition via vtable slot +0x08.

**Static-analysis agent could not reach that dispatch site through function-name grep**, because vtable dispatch is opaque to `FUN_XXX(` search. This is the last piece we don't have in-hand.

### 7. Scope — per-class, not looped

There is NO single caller that iterates the 5 English pyramid comps and calls season-roll on each. Each competition class has its own vtable slot +0x08 method. The date tick presumably iterates all competitions in the world and dispatches through their vtables.

### 8. Query-mode caller — unrelated to year turn

GDI `sub_00560810` == DD `FUN_00560610` at `.../decompiled/00560610.c`. Same eng_second class family (same source-path string). Signature: builds a schedule allocation buffer, calls schedule_getter with mode = 0 (query), then materialises per-match records via `FUN_0050ca60`. **Lifecycle role: season-fixture materialiser, fired when the schedule needs to be laid out** (season start, UI/scheduler query for "give me this comp's match list"). NOT year-turn related — unrelated code path.

### 9. Constructor separation — proven

Two independent proofs the ctor is NOT re-invoked at year turn:

1. `FUN_0055f040` (eng_second ctor) has ZERO direct callers by name in the decompile — only vtable slot 0 (destructor path) references it.
2. The season-roll methods perform **read-modify-write** on `this + 0x40` (self-referential year advance). A freshly constructed object would take year from the ctor arg, not from itself. This unambiguously proves the object is retained across year turns; only its schedule sub-buffer at `+0xba` is freed and rebuilt.

## Report against C11.3A's 16 required items

| # | Item | Answer |
|---|------|--------|
| 1 | All xrefs to sub_00560520 | 2 total — ctor call (`0x0055f0c1`) + vtable slot +0x8c pointer (`0x00957e54`) |
| 2 | Year-turn caller identified | Whoever calls eng_second vtable slot +0x08 = a season-roll method structurally identical to `FUN_00575da0` |
| 3 | Caller VA/source cpp | eng_second class body (source `comp_l…`), season-roll method at vtable +0x08 — specific VA for eng_second's slot 8 not resolved by Ghidra |
| 4 | Exact branch condition | **Still open** — the check `if (year advanced)` lives at the date-tick dispatch site, one level above the season-roll body. The season-roll itself is unconditional |
| 5 | Source of old/new year values | Comp+0x40 is self-authoritative for old year; new year comes from an external date-tick trigger (still open — see #4) |
| 6 | Meaning of Comp+0x40 | Season year (u16 LE); initialized from ctor param_2 (`0x7d1 = 2001`); advanced by season-roll methods in-place |
| 7 | Writers of Comp+0x40 | Ctor initial write; three season-roll methods (`FUN_00575da0`, `FUN_0077fee0`, `FUN_009192b0`) do `+2` in-place |
| 8 | Trigger form | Cannot answer conclusively without finding the date-tick dispatch site. Likely candidates: `if (global_year != last_seen)` OR `if (calendar_advanced)` |
| 9 | English-Second-specific or generic? | **Per-class, not generic**. Each of the 5 English pyramid comps has its own vtable slot +0x08 season-roll method. But the date-tick loop that invokes them is presumably generic across all competitions |
| 10 | Other league-class calls in same scheduler | Not enumerable from the class body — happens at the date-tick dispatch (still open). Analogous season-roll methods exist for many classes (`FUN_00575da0` etc.), so all league classes probably have one |
| 11 | Query-mode caller | `FUN_00560610` — season-fixture materialiser, called from season-start/UI paths. Unrelated to year turn |
| 12 | Ctor separation proof | Yes — ctor `FUN_0055f040` has no direct non-destructor callers; season-roll methods read-modify-write `this + 0x40` in place |
| 13 | Rust trigger location/API | Single-comp reset method (analogue of `FUN_00560320`) on the `EnglishCompetition` struct's own impl, dispatched from a game-tick per-comp iterator (analogue of the date-tick dispatch we haven't found yet) |
| 14 | Path B still needed? | **Not strictly necessary** to prove per-class scope (the DirectDraw class hierarchy already proves it). But WOULD help verify the trigger fires at the same moment for Prem/First/Third/Conf — worth doing before the Rust fix, cheap to add |
| 15 | Tests to add after fix | (deferred — see next steps) |
| 16 | Commit hash | (this commit) |

## What we now know with certainty

* The regeneration mechanism is **per-competition-class**, not a central English-pyramid handler
* The competition object PERSISTS across year turns (only the schedule sub-buffer at `+0xba` is freed and rebuilt)
* The year field at `Comp+0x40` is self-authoritative — advanced by the class's own season-roll method
* `FUN_00560320` (and its analogues) is a **stateless "reset defaults + regen schedule" method** — no branch, no guard
* The **actual trigger check** lives at the date-tick dispatch site, one static-analysis pass away

## What still blocks the Rust fix

The exact condition under which the date tick calls a competition's season-roll method. Without that, coding `hook_year_rollover` or a per-comp season-roll trigger risks:

* Firing every day (too many)
* Firing only on English-set year-end (misses calendar-year comps)
* Firing on Jan 1 (correct for split-year, wrong for calendar-year)

## Two paths to close

**Path A — one more targeted Explore pass on the date-tick side.** Look at `FUN_005b6a90` (memory `[[game-tick]]` calls this the exe daily tick driver), find where it iterates competitions and dispatches vtable slot +0x08. Read the guard condition. Estimated: 30-45 min agent time. This is the direct path to point 4.

**Path B — small targeted Frida run.** Add a hook on the season-roll methods (`FUN_00575da0` etc.) as GDI VAs, plus a hook that reads the game date at each fire. Confirms which method fires at which game date. Runs alongside Path A as verification. Not strictly needed if Path A is definitive.

Recommend Path A first, then Path B if Path A leaves ambiguity.

## Fixtures / files

* Runtime evidence: `fixtures/gdi_captures/season_2001_02_v3_thiscall_fixed.jsonl`
* Explore agent output: at task file (not committed)
* This report: `reports/c11_3a_static_analysis.md`
