# C11.3 Phase B — evidence-driven findings

Capture: `fixtures/gdi_captures/season_2001_02_v3_thiscall_fixed.jsonl`
(8,215 events, ~60 min wall clock; user Ctrl-C'd mid-second-season
so we see 3 schedule_getter calls not the full 5). Enhanced with
Phase A.2 `__thiscall` fix so `this` is read from ECX.

## The three schedule_getter events

| ms | mode | ret_addr | `this` | vtable | comp name at `+0x54` | year @ `+0x40` | rng in | rng out |
|----|------|----------|--------|--------|----------------------|---------------|--------|---------|
| 45,574 | **-1 regen** | 0x5605a5 | 0x629dc48 | 0x00957dd8 | `English Second Division_9.tmp` | **2001** | 1221 | 1221 |
| 1,326,057 | **0 query** | 0x5608c9 | 0x629dc48 | 0x00957dd8 | `English Second Division_9.tmp` | **2001** | 3,844,207,791 | 3,844,207,791 |
| 2,226,283 | **-1 regen** | 0x5605a5 | 0x629dc48 | 0x00957dd8 | `English Second Division_9.tmp` | **2002** | 3,256,114,486 | 3,256,114,486 |

## Points 1–10, evidence-based answers

### Point 1: comp `0x6014fb0` identity

**Retracted from prior tranches.** `0x6014fb0` in the v1 capture
was a **driver arg0**, mis-labelled as a competition id. The
actual competition instance the mid-season regen targets is
**`0x629dc48` = English Second Division (comp id 9)**.

Byte-exact identity comes from the 128-byte hex dump of `*this_ptr`:
* First 4 bytes = vtable pointer `0x00957DD8`
* Bytes at `+0x54..+0x71` = null-terminated `"English Second Division_9.tmp"`
  (Latin-1). The trailing `_9.tmp` is the shipped-data comp id
  suffix, matching `EnglishPyramidCompIds.second = 9` in
  `eng_second_fixtures.rs`.

### Point 2: exact regen dates

Not calendar dates in the strict sense — the exe stores a **year
field at `this + 0x40`** on the eng_second object. The regen
correlates with that field advancing.

* Boot regen: `+0x40 year = 2001`, ms 45,574 — initial construction
* Mid-season query: `+0x40 year = 2001`, ms 1,326,057 — the object
  is still tagged 2001, gameplay is reading fixtures
* Mid-season regen: `+0x40 year = 2002`, ms 2,226,283 — **the
  year field advanced from 2001 to 2002 between calls 2 and 3**,
  and the `-1` regen fired immediately after.

This aligns with the user's original hypothesis: the trigger is
the **calendar-year turn** (31 Dec 2001 → 1 Jan 2002), NOT a
foreign calendar-year league. The English Second Division
(a split-year 2001/02 season) responds to the January turn by
generating **next season's fixtures** while the current season
plays out.

### Point 3: are the two mid-season calls the same event?

In this v3 run only one mid-season regen was captured (user
stopped early). But comparing to the v1 run at `28198ba` where
two mid-season regens fired: the v1 finding still holds
structurally — one per calendar year turn. Under the new
interpretation:

* boot: year = 2001, generate 2001/02 season
* Jan 1 2002: year advances 2001 → 2002, regen for **2002/03**
* Jan 1 2003: year advances 2002 → 2003, regen for **2003/04**

Two mid-season regens across 2 game-seasons = one per Jan 1.
Consistent.

### Point 4: immediate caller of the `-1` call

**`sub_00560520` (146 bytes, GDI VA `0x00560520`).**
Return address `0x5605a5` (i.e. `0x00560520 + 0xa5`) fired 3/3
`-1` calls across v1, v2, v3. The function hardcodes `push -1`
right before its vtable call:

```asm
0056059a  push  -1                    ; mode = regenerate
0056059c  mov   ecx, esi              ; this = the eng_second obj
005605a2  call  dword ptr [eax+0x3c]  ; vtable[+0x3c] = schedule_getter
005605a5  mov   [esi + 0xba], eax     ; store result — ret_addr lands here
```

`sub_00560520` is a **vtable method on the same class as
schedule_getter**. It's a "reset schedule buffer + regenerate"
utility method. Whoever calls **`sub_00560520`** is the actual
lifecycle trigger — one level up.

**Still open**: what calls `sub_00560520`? It's an indirect vtable
call, so linear grep won't find it. Static analysis in the GDI
Ghidra decompile (which has proper xrefs) needs a follow-up pass.
Once identified, that caller's condition IS the trigger.

### Point 5: arg0 semantic proof

`-1` = regenerate. `0` = query. Byte-proven:

* Every `-1` came from `sub_00560520` (the "reset schedule" method)
* Every `0` came from `sub_00560810` (the "look up existing schedule" method)
* `sub_00560520` hardcodes `push -1` in its disassembly

### Point 6: generic across calendar-year leagues?

**Not observed in this capture** and the previous "calendar-year"
framing is now retracted. What we observed is the mechanism on
the **English Second Division only**. Whether the other 4 English
pyramid comps (Prem, Div 1, Div 3, Conference) and other nations'
leagues use the same year-turn trigger is:

* **Not yet observed** in this capture (no hooks fired for other comps' getters)
* **Very likely** by symmetry — all comps that inherit from the
  same class would carry the same `+0x40 year` field and the
  same regen path.

Answering point 6 rigorously would need enabling the hook on
other league objects' getters. Not urgent for the initial fix
(English pyramid alone matches the C11.3 tranche's scope).

### Point 7: no fixture algorithm changes

Held. Nothing in Phase B touches the frozen engine.

### Point 8: exact Rust trigger

Evidence-based candidate:

```rust
/// Fires exactly once per (eng_second_style_comp × calendar
/// year turn). Called by the daily tick when `old_date.year !=
/// new_date.year`.
pub fn on_calendar_year_advance(
    comp: &mut EnglishCompetitionSet,
    old_year: u16,
    new_year: u16,
) {
    if old_year == new_year { return }
    // Mirrors sub_00560520: reset schedule buffer bytes on the
    // comp instance, then invoke the schedule regen entry point.
    comp.reset_schedule_scratch_bytes();
    comp.regenerate_schedule_for_next_season();
}
```

**Not yet coded.** Two open items block writing it faithfully:

1. Which fields does `sub_00560520` reset before calling the
   getter? (I can transcribe from the disasm — 15+ byte writes at
   `this + 0x3c..+0xc7`.)
2. What is the caller of `sub_00560520`? If it's a per-comp
   iterator called from a global year-end hook, our Rust trigger
   should match that shape rather than firing on the daily tick
   directly.

### Point 9: don't reconstruct comp objects

Held. The exe reuses the same instance at `0x629dc48` for boot,
query, mid-season regen. The Rust equivalent should regenerate
the schedule on the existing `Competition` instance, not
reconstruct the whole object.

### Point 10: RNG continuity

**Zero RNG delta on all 3 schedule_getter calls.** The getter
itself is RNG-clean. RNG consumption during regeneration lives
in the downstream driver / walker / perturb functions (which the
tranche mandates unchanged). RNG continuity is therefore NOT a
lifecycle concern — the exe's sequence just needs the same
number of downstream driver calls.

## Points 11–17: still pending

Need Rust changes (point 8 above) before writing tests for
points 11–13. Once the caller of `sub_00560520` is nailed down
via Ghidra, I'll produce:

* 11: duplicate-regen test (year change fires trigger exactly once)
* 12: European negative control (English Premier / other split-year comps do NOT regen on year turn — or DO, if point 6 says so)
* 13: constructor vs getter separation test
* 14: fixture-lifecycle freeze contract updated

## The methodological correction

The retracted claims from prior tranches:

* "`0x6014fb0` = same league across mid-season regens" — that was
  a driver arg mis-labelled as comp identity. **Withdrawn.**
* "Calendar-year foreign leagues (Brazil / Finland / etc.) are
  the mid-season regen target" — that was inference from the
  Rust `league_calendar.rs` seeing 7 calendar-year countries.
  Wrong — the target is the English Second Division itself
  reacting to the calendar year turn (which happens mid-football-
  season). **Withdrawn.**

The user's earliest hypothesis — **"what happens between 31 Dec
and 1 Jan"** — turned out to be exactly right. My "calendar-year
foreign leagues" alternative was a wrong turn; the correct
interpretation is that the January turn triggers a year-field
advance on the English competition's own state.

## Files

* `fixtures/gdi_captures/season_2001_02_v3_thiscall_fixed.jsonl` — evidence
* `reports/c11_3_phase_b_findings.md` — this file
* `reports/c11_3_phase_a_2_thiscall.md` — the __thiscall bug fix (prior tranche 50d4cc0)

## Next step

Point 4 needs closure: identify the caller of `sub_00560520` in
the DirectDraw Ghidra decompile (which resolves xrefs). Its
condition IS the executable trigger. I can do that with an
Explore/agent pass over the decompile once you say go.

Alternatively: enable the schedule_getter hook temporarily on
the Premier / Div 1 / Div 3 / Conference class instances to
verify point 6 (is the year-turn trigger generic across the
English pyramid?). That's a smaller instrumentation add and
would give the negative-control data point 12 needs anyway.
