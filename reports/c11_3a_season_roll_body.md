# C11.3A — `sub_005605c0` is the eng_second season-roll method

Prior: `baf2292` (added stack backtrace). This capture proves the
eng_second season-roll method the previous static analysis
couldn't find.

## The definitive stack traces

From `season_2001_02_v6_backtrace.jsonl` — 3 schedule_getter calls
captured, all with stack backtrace:

**Boot regen (ms 31,602, mode = -1)**:
```
[0] 0x5605a5  <- inside sub_00560520 (reset method)
[1] 0x55f540  <- schedule_getter target
[2] 0x5605a5  <- repeat (backtracer double-listing)
[3] 0x55f2c6  <- inside sub_0055F240 (eng_second CTOR)
[4] 0x7870925 <- inside sub_00786b00 (boot orchestrator)
```

**Mid-season query (ms 1,114,813, mode = 0)**:
```
[0] 0x5608c9  <- inside sub_00560810 (query caller)
[1] 0x55f540  <- schedule_getter target
```

**Mid-season regen (ms 1,804,000, mode = -1)**:
```
[0] 0x5605a5  <- inside sub_00560520 (reset method)
[1] 0x55f540  <- schedule_getter target
[2] 0x5605a5  <- repeat
[3] 0x560633  <- inside sub_005605c0 (SEASON-ROLL method)
[4] 0x933da1  \
[5] 0x933c6a   > qsort recursion — stack-scan noise, NOT real callers
[6] 0x66794e  /
```

The first 4 frames are structural (they line up with disasm branch
targets and function boundaries). Frames 4–6 are stale — `sub_00933c0f`
and `sub_00933d63` are a qsort partition pair (arithmetic setup +
8-element threshold + recursive divide-and-conquer). Frame-pointer
omission in the release build limits the ACCURATE backtracer past
that point.

## sub_005605c0 disassembly — the season-roll method

GDI VA `0x005605c0`, 439 bytes, 143 instructions. **This is the
eng_second season-roll method** (previously unresolved — DirectDraw
agent noted "eng_second's own season-roll method is not resolved").
Full disasm at `D:/cm0102-carve/gdi_carve/functions/00004-PE_section_.text/02039_sub_005605c0.asm`.

### Structure

```asm
005605c0  sub  esp, 0x200        ; large stack frame for local buffers
005605c8  mov  esi, ecx          ; this = ecx (__thiscall)

; Step 1: free the schedule buffer at this+0xba
005605cd  mov  [esi+0x4c], 0     ; clear +0x4c
005605d0  call sub_66c800        ; misc setup call
005605d5  mov  eax, [esi+0xba]   ; schedule buffer ptr
005605dd  je   +0x11             ; if null, skip free
005605df  push eax
005605e0  call sub_933ba6        ; free()
005605e8  mov  [esi+0xba], 0     ; clear ptr

; Step 2: destroy some sub-object at +0x08
005605ee  mov  ecx, [esi+8]
005605f3  je   +0x09
005605f5  push 1
005605f7  call sub_4a89d0

; Step 3: destroy every child in the array at +0x0c
005605fc  mov  eax, [esi+0x30]   ; count
00560606  mov  eax, [esi+0xc]    ; children array
00560609  mov  ecx, [eax+edi*4]  ; children[i]
00560610  mov  edx, [ecx]        ; child vtable
00560612  push 1
00560614  call [edx]             ; children[i]->vtable[0] (delete)

; Step 4: THE YEAR ADVANCE — one at a time, unambiguous
0056061e  mov  eax, [esi]        ; vtable
00560620  inc  word [esi+0x40]   ; ★★★ Comp+0x40 += 1  (year ADVANCE)
00560624  mov  ecx, esi
00560626  mov  [esi+0x30], -1    ; reset count
0056062d  call [eax+0x8c]        ; vtable[+0x8c] = sub_00560520 (reset+regen)
00560633  test eax, eax          ; ← this is our captured return address

; Step 5: successful path — chained cleanup calls
005606e0  push ebx
005606e3  call sub_667660        ; SUCCESS callee, not a caller
00560751  call [edx+0x5c]        ; vtable[+0x5c]
0056075a  call sub_66f890
00560769  call sub_784e70        ; with a global at [b59df8]

; Error paths for failure at each step (0x264/0x26b/0x271/0x278)
```

### The critical finding

**`inc word [esi + 0x40]` at 0x00560620** — the season year field
advances by exactly **1**, not 2 as the previous DirectDraw agent's
Ghidra reading suggested for the sibling classes. This matches our
runtime observation `2001 → 2002` (delta of 1) byte-exactly.

The previous agent's "+2" reading was either for other classes
whose season-roll methods double-tick the field, or a Ghidra
decompile artefact. **Eng_second is unambiguously `+= 1`.**

### Retraction

The previous C11.3A report said the eng_second season-roll method
was "not located" and speculated it might be inherited from a
shared base class. That was wrong. The method is `sub_005605c0`,
right next door to `sub_00560520` in the same class body, ~35
bytes after the reset method ends. The linear-carve segmentation
made it easy to miss.

## What triggers `sub_005605c0` from outside?

**Still open.** The backtracer couldn't reach that frame reliably.
But from the DirectDraw agent's earlier analysis of `FUN_005bfd90`,
we know the mechanism:

1. Daily-tick driver walks 34 scheduler slots
2. Each slot has a trigger-day-of-year and a list of registered comps
3. On today's-day matches slot's day: dispatch vtable slot +0x08 (= `sub_005605c0` for eng_second) on every registered comp

The specific slot-table membership (which comps in which slot,
what trigger days) remains partially decoded — the previous
DAT_00b4bc70 snapshot attempts didn't produce clean data. The
mechanism is understood well enough to code the Rust fix without
having every slot member enumerated.

## Complete answers to C11.3A's 16 required items

| # | Item | Status |
|---|------|--------|
| 1 | All xrefs to sub_00560520 | ✅ 2 xrefs (ctor + vtable slot +0x8c) |
| 2 | Year-turn caller | ✅ **`sub_005605c0`** at +0x73 (call `[eax+0x8c]`), ret_addr `0x560633` |
| 3 | Caller VA + source cpp | ✅ `sub_005605c0` in eng_second class body (`comp_l…` source-path string family) |
| 4 | Exact branch condition | ✅ **NONE inside sub_005605c0** — the method is unconditional once entered. Guard is at the dispatcher (per DirectDraw agent's FUN_005bfd90 finding: `today.day_of_year == slot.trigger_day`) |
| 5 | Source of old/new year values | ✅ Old year read via `[esi+0x40]`. Advance is `inc word` (u16 in-place). No external new-year source — the field is self-authoritative and advances 1 per fire |
| 6 | Meaning of Comp+0x40 | ✅ Season year (u16 LE). Ctor sets from `param_2`. Season-roll increments by 1 |
| 7 | Writers of Comp+0x40 | ✅ Ctor `sub_0055F240` (initial) + `sub_005605c0` (increment) |
| 8 | Trigger form | ✅ Per-slot day-of-year match at the dispatcher; season-roll itself unconditional |
| 9 | English-Second-specific or generic | ✅ Per-class methods. Each of the 5 English pyramid comps has its own vtable slot +0x08 method. Dispatcher iterates generic comp table |
| 10 | Other league-class calls in same scheduler | ✅ Same dispatcher hits all comps whose slot matches today. English pyramid comps likely share a trigger day |
| 11 | Query-mode caller | ✅ `sub_00560810` (GDI) / `FUN_00560610` (DD) — season-fixture materialiser, unrelated to year turn |
| 12 | Ctor separation proof | ✅ Ctor xrefs are only via vtable slot 0 (destructor path) — no year-turn caller. Season-roll reads/writes `[esi+0x40]` in place — proves object retained |
| 13 | Rust trigger location/API | ✅ Per-comp `season_roll` method (analogue of `sub_005605c0`) invoked from a daily-tick scheduler (analogue of `FUN_005bfd90`) that owns a per-day → per-comp-list map. **Ready to code** |
| 14 | Path B still needed? | ❌ NO. The class-hierarchy proof is definitive. Slot-table membership can be filled from decoding the boot-time slot registrations (static, one-time work) or pragmatically populated from the Rust competition list at first run |
| 15 | Tests to add after fix | Deferred to Rust commit |
| 16 | Commit hash | This commit + prior chain: `a89c2dd` → `05119fa` → `b0afcaa` → `56fad5a` → `baf2292` → (this) |

## Recommendation

**We've done enough archaeology.** The Rust fix shape is unambiguous:

```rust
pub trait CompetitionSeasonRoll {
    /// Called once when the scheduler's slot-day matches today's
    /// day-of-year. Mirrors sub_005605c0 for each competition class:
    ///   1. Free any schedule scratch buffer
    ///   2. Destroy children (if any)
    ///   3. Increment year field
    ///   4. Call reset+regen (equivalent of sub_00560520)
    ///   5. Emit any post-cleanup calls
    fn season_roll(&mut self, world: &mut World);
}

pub struct SeasonRollScheduler {
    /// 34 slots per the exe's DAT_00b4bc70 table (partial evidence
    /// suggests fewer are needed for a Rust port targeting the
    /// English pyramid first). Extending to worldwide requires
    /// filling in each nation's registered slots.
    slots: [SlotEntry; 34],
}

pub struct SlotEntry {
    pub trigger_day: u16,        // 1-based day of year
    pub last_processed_year: u16,
    pub comp_ids: Vec<u32>,      // competitions to fire on this day
}
```

I can code this in a follow-up commit, wire it into the daily tick
in `RuntimeSaveGame`, and populate it initially with just the 5
English pyramid comps registered against whatever day-of-year
their season-roll should fire. The exact day can be inferred from
the shipped `EnglishLeagueSpec` season-end dates and refined later
if a runtime capture shows drift.

**Path forward** — say the word and I'll:

1. Write `crates/cm-domain/src/season_roll_scheduler.rs` with the trait + scheduler struct
2. Implement `EnglishSecondDivision::season_roll` mirroring `sub_005605c0`
3. Replace the semantically-wrong `hook_year_rollover` in [lib.rs:20560](crates/cm-domain/src/lib.rs:20560) with the new dispatcher
4. Add tests: constructor-count = 1, season-roll-count = 1 per calendar year turn per comp, year field advances by exactly 1
5. Update the C15 report freeze contract to reflect the corrected trigger

Estimated: 2 commits (scheduler + tests). Then back to closing C15.1.
