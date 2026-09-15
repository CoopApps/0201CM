# C11.3A — date-tick + career-line findings

Prior: `a89c2dd` (per-class season-roll scope). This pass targeted
the actual guard condition and the career-line writer.

## Headline finding — the trigger is a slot scheduler, not a year check

The exe does NOT check `if (year_advanced)` per competition. It has
a **34-slot scheduler table at `DAT_00b4bc70`** (stride `0x48` bytes,
DirectDraw exe). Each slot has:

| Offset | Field | Meaning |
|--------|-------|---------|
| `+0x0c` | `count` (i32) | Number of competitions in this slot's pool |
| `+0x10` | `pool` (Comp**) | Base of this slot's competition-pointer array |
| `+0x15` | `trigger_day` (u8) | Day-of-year to fire on |
| `+0x35` | `last_processed_year` (u16) | Which year was last serviced |

Every day, the daily-tick driver `FUN_005b6a90` reaches phase 2 and
calls `FUN_005bfd90` (VA `0x005bfd90`). That function walks the 34
slots. **For each slot whose `trigger_day` equals today's
day-of-year, it iterates every comp in that slot's pool and invokes
vtable slot `+0x08`** (the season-roll method) after two per-comp
predicates (`+0xc` = "is-active", `+0x2c` = "already-processed").

Slot-year comparison against the global year only gates the
progress-string/announcement blocks — it does NOT gate the dispatch.
The dispatch is purely day-of-year driven.

## What this means for the English Second Division observation

Our runtime evidence (Frida `season_2001_02_v3_thiscall_fixed.jsonl`):
`Comp+0x40 = 2001` on boot → still `2001` on mid-season query → `2002`
on second regen. The `2001 → 2002` transition happened because
eng_second is registered in some slot with a trigger day that fell
between the two calls. Not because the year changed anywhere else
first.

The English Second Division doesn't advance its year "on Jan 1"
because Jan 1 is Jan 1 — it advances because **it lives in a slot
whose day-of-year happens to be around that time**. Different
competitions live in different slots. Brazilian league in a
December slot, English pyramid in whichever slot the exe registers
it into. **This solves the mystery of how calendar-year and
split-year competitions coexist without a per-comp year check.**

## Answering C11.3's remaining tranche points

| # | Item | Answer |
|---|------|--------|
| 4 | Year-turn caller | `FUN_005bfd90` (daily-tick's phase-2 slot dispatcher). Invokes vtable `+0x08` on every comp in the matching slot's pool |
| 5 | Guard condition source | **Slot table entry's `trigger_day` field** (u8 at `slot + 0x15`). Sourced from the exe's static slot registrations at boot |
| 8 | Exact guard form | `today.day_of_year == slot.trigger_day` — **NOT** year-diff, not month/day literal, not comp.year != global.year. Event-driven scheduler |
| 10 | Other league-class calls | Yes — same dispatcher loop hits every slot on its matching day. Prem/First/Second/Third/Conf almost certainly all in the same slot (or same-day slots) since they all had `year=2001` at boot and would advance in unison. Cannot confirm without dumping slot membership at runtime |

## Rust fix scope — reframed

**Not** a `hook_year_rollover` extension. **Not** a per-comp
year-difference check. What the exe does is:

```
struct SeasonRollScheduler {
    slots: [SlotEntry; 34],
}
struct SlotEntry {
    trigger_day: u8,
    last_processed_year: u16,
    comp_ids: Vec<u32>,
}

// Daily tick:
for slot in &mut scheduler.slots {
    if today.day_of_year != slot.trigger_day { continue }
    for comp_id in &slot.comp_ids {
        let comp = world.competition_mut(comp_id);
        if !comp.is_active() || comp.already_processed() { continue }
        comp.season_roll();  // year += 1, reset scratch, regenerate schedule
    }
    if slot.last_processed_year != today.year {
        emit_announcement(slot, today.year);
        slot.last_processed_year = today.year;
    }
}
```

The C15 `hook_year_rollover` in [lib.rs:20560](crates/cm-domain/src/lib.rs:20560)
that fires on `date.year != last_year_rollover` is **semantically
wrong** — it fires only once per Rust calendar year, whereas the
exe fires slot-triggered work on 34 different scheduled days.

## What's still missing before Rust changes

**The slot table's membership.** We know how the dispatcher works
but not which competition goes into which slot. Two paths:

**Path A — boot-time slot registration**. Somewhere at boot, each
competition registers itself into a slot. Likely called from the
comp constructor family. Static analysis can find those calls.

**Path B — dump `DAT_00b4bc70` at runtime via Frida.** Cheap, gives
the full slot membership post-boot in one shot, and confirms
Path A's findings.

Path B is faster; Path A confirms the mechanism. Recommend Path B
first for the data, then Path A only if the mechanism needs re-porting.

## Career-line writer (C15.1 §26.5)

Not conclusively identified in this pass. What IS established:

* Staff records at `DAT_00acd5bc` (stride `0x245`) carry **~13 history-list head pointers** at offsets `+0x53, +0x57, +0x5b, +0x60, +0x69, +0x6e, +0x83, +0x8b, +0x87, +0x8f, +0x93, +0x97, +0x9b, +0x9f, +0xa3`.
* Club records at `DAT_00acd5d8` (stride `0x6b`) carry **~4 similar heads** at `+0x59, +0x5d, +0x61, +0x65`.
* All 17 heads are enumerated by the save-writer `FUN_005176c0` — proven to be persistent state pools.
* Match-participant reputation update `FUN_005c0500` fires per-staff BEFORE the season-roll on each slot day — but it mutates a reputation counter, not a history list.
* The career-history push is **structurally per-comp per-staff scatter** inside the season-roll cascade (either the `+0x08` season-roll body itself or its `+0x8c`-reached child). Not a single flat writer.

**Big implication for C15.1**: our current Rust `staff_history: Vec<DomainHistory17>` (flat single list) is not going to match the exe's 13-list-per-person structure. C15.1 §26.5's "person history materialisation" is genuinely bigger than I estimated — it's not "queue a `RuntimeEvent`", it's "wire 13 typed history heads onto the staff pool and route each comp's contribution to the right head at year-roll time".

## Impact on the "how far off from other leagues" question

The slot scheduler is **generic across all 26 selectable leagues**.
Once we port it once, adding a foreign league becomes:

1. Register its comp into whichever slot has its year-turn day-of-year
2. Give its class a season-roll method (like `FUN_00575da0`)
3. Give it a reset-defaults+schedule-regen method (like `FUN_00560320`)

Steps 2 + 3 are per-league-family patterns already partially decoded.
Step 1 is a one-line registration. So worldwide expansion becomes
significantly cheaper once C11.3 lands.

## Next steps

**Immediate:**
1. One targeted Frida run to dump `DAT_00b4bc70` slot membership. Cheap; gives ground truth.
2. Then Rust: `SeasonRollScheduler` module + wire into `hook_year_rollover` replacement.
3. C15.1 §26.5 needs a data-model call: keep flat `staff_history` with a discriminator field, or port the 13-list structure faithfully. Pragmatically I'd say keep the flat vector with a `history_kind: u8` and a comment noting the exe's scatter — full faithful port would be a separate tranche.

**Path B Frida is simple**: extend `season_capture.js` with one boot-time snapshot function that reads `DAT_00b4bc70` (GDI equivalent VA — needs to be resolved but likely `DAT_00b4bc70 - 0xB0` = `0x00b4bbc0`) as a 34-entry byte array, decodes the count / trigger_day / pool_ptr / last_year fields, and emits one JSONL event. Then walks each pool reading comp identity strings.

## Files

* `reports/c11_3a_static_analysis.md` — prior class-hierarchy findings
* `reports/c11_3a_datetick_findings.md` — this file
* `fixtures/gdi_captures/season_2001_02_v3_thiscall_fixed.jsonl` — runtime evidence
