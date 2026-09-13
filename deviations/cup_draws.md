# Deviation: Cup draw dispatch (fixtures appear pre-drawn at save init)

- **Rust sites**:
  - `crates/cm-domain/src/lib.rs` around 17627-17656 — cup init emits R1
    fixtures at save creation via `domestic_cup::generate_first_round`.
  - `crates/cm-domain/src/domestic_cup.rs` — bracket engine.
- **Executable functions / addresses**:
  - `FUN_00669a90` (`0x00669a90`) — per-competition daily tick; fires cup
    round draws when today matches `comp+0xd1` (next-draw date).
    Vtable dispatch at `:99-121` calls
    `**(code**)(*param_1 + 0x10 / 0x28 / 0x30)`.
  - `FUN_00443770`, `FUN_00447d20` — cup pairing routines (draw handlers).
  - `FUN_004b6b30` — cup eligibility scan.
  - `FUN_00845380` — fixture record writer (called from draw routines).
  - `FUN_00594d00` — link match into per-club `+0x2861` tree.
  - `FUN_00558f60` (`0x00558f60`) — English FA Cup schedule + hardcoded
    round dates (draw + match). Sibling `FUN_00556150` = English League Cup.

- **Discrepancy**:
  - Port creates first-round cup fixtures at save-init using bracket seeding.
  - Real exe writes cup fixtures only when the draw fires on the round's
    scheduled draw date via `FUN_00669a90` daily-tick vtable dispatch;
    until that date, the round's fixtures don't exist in the fixture pool.

- **Evidence**:
  - `FUN_00669a90` decompiled (`decompiled/00669a90.c:1-186`); draw dispatch
    confirmed at :99-121.
  - `FUN_00558f60` FA Cup schedule fully decoded — per-round `(day, month,
    year_offset)` triples for both draw (`FUN_0050bac0`) and match
    (`FUN_0050bb10`) calls.
  - Draw dates for 2001/02 in reports/cup_round_schedules.md.
  - Cup pairing routines FUN_00443770 / FUN_00447d20 NOT YET decompiled.
  - Eligibility scanner FUN_004b6b30 NOT YET decompiled.

- **Confidence**: TEMPORARY STUB (bracket seeding is pre-emptive, not
  daily-tick driven)

- **Work required**:
  1. Decompile FUN_00443770 / FUN_00447d20 / FUN_004b6b30 (cup pairing
     routines + eligibility scanner) into readable source.
  2. Decompile FUN_00845380 + FUN_00594d00 (fixture write helpers) fully.
  3. Port FUN_00669a90 into cm-domain per-comp daily tick.
  4. Wire cup ctors (English FA/League/etc.) to install their round-draw
     dates on `comp+0xd1` and vtable slots `+0x10/+0x28/+0x30`.
  5. Retire bracket pre-seeding in cm-domain/lib.rs — cup fixtures should
     appear in `save.season.fixtures` only after their draw date has ticked.
  6. Validate against Frida trace of the exe advancing day 1 → day 100
     comparing fixture pool state daily.

- **Player-visible**: YES (League Cup and FA Cup pairings visible at boot
  when the exe has none drawn yet)
- **Save-affecting**: YES (fixture pool composition and event order)

- **Opened**: 2026-09-13
- **Resolved**: —
