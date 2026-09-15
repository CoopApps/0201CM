# C11.3 Phase A — enhanced capture harness

Prior tranche: `28198ba` (2-season capture findings — schedule
getter is bi-modal, mid-season regen exists, ctor is one-shot).

## Scope of Phase A

Extend the existing Frida harness so the NEXT capture records the
evidence C11.3 needs. **No Rust changes in Phase A.** No trigger
condition is invented — the executable's condition will be recovered
from the runtime data.

## What Phase A adds

**One new arg_shape**: `comp_and_caller` (and `schedule_getter`
which aliases it). Applied to two hooks:

* `sub_0055F540_schedule_getter` — already ships in Group B
* `FUN_00668450_shared_driver` — already ships in Group B

For every enter event on those hooks the JSONL now carries:

* `arg0_raw`  — the pointer as `"0x…"`
* `arg0_int`  — the same value as a signed int32 (so sentinels like
  `-1` are unambiguous)
* `comp_record` — one of:
   * `{ sentinel: N }` when arg0 is a small integer or `-1`
   * `{ id: <u32>, hex128: "…" }` when arg0 dereferences cleanly —
     the first 128 bytes of the record are hex-encoded so post-
     processing can extract id / name / three-letter / nation /
     reputation using the pool-record layout from memory
     `[[record-layouts-decoded]]`
   * `{ err: "…" }` when the pointer looks plausible but the read
     faulted
* `return_addr` — the immediate caller's return address on the
  stack, as `"0x…"`. **This is the piece the tranche's point 4
  needs — the "immediate caller of the -1 call".**

RNG state at enter is already emitted; the leave record captures
it again for the pre/post pair needed by point 10.

## What Phase A does NOT add

Deliberately kept minimal to avoid re-introducing the boot slow-down
we hit on the wide-hook run. **No game-date read hook** — reading
the game date requires a GDI VA we don't have pinned, and the
return-addr + Ghidra decompile chase is a cleaner path to the
same answer:

* We find the immediate caller VA from `return_addr`.
* We open that function in the GDI Ghidra dump.
* The date read is a call to whatever the game-date-reader is,
  visible in the caller's decompile — we identify BOTH the caller
  AND the date semantics from one static-analysis pass.

If the caller doesn't directly read the date (e.g. it's a per-day
tick that reads no date itself, just fires whenever), we may end up
needing a date hook — but the return-addr trace tells us that FIRST,
and cheaply.

## Preflight verification

Manifest passes `verify_manifest_vas.py` clean: every enabled hook
lands on a real GDI function start. Same 7 Group B hooks as the
previous stable capture; all Group A remains disabled.

## Files changed

* `tools/gdi_capture/season_capture.js`
  * New `readCompFromPtr(p)` helper: 128-byte hex dump + id
  * New arg_shape `schedule_getter` / `comp_and_caller`
  * `decodeArgs()` now receives the Interceptor context so it can
    read `return_addr`
* `tools/gdi_capture/season_capture_manifest.json`
  * `sub_0055F540_schedule_getter` → arg_shape `schedule_getter`
  * `FUN_00668450_shared_driver` → arg_shape `comp_and_caller`
  * Notes explaining the C11.3 rationale on both
* `tools/gdi_capture/decode_schedule_getter_calls.py` — new
  post-processor that decodes both getter and driver records,
  extracting comp identity + caller + RNG

## What Phase A needs to run

One user Frida capture with the enhanced manifest. Ideally two
seasons again (~80 min wall clock) so we see the mid-season regen
event twice. Preferably in the same game process (no restart) so
the runtime comp pointer stays stable, letting us map `0x6014fb0`
to one specific competition record byte-exactly.

```
D:/Python312/python.exe D:\cm0102-rs\tools\gdi_capture\season_capture.py --out D:\cm0102-rs\season_2001_02_v2.jsonl
```

Then:

```
D:/Python312/python.exe D:\cm0102-rs\tools\gdi_capture\decode_schedule_getter_calls.py D:\cm0102-rs\season_2001_02_v2.jsonl
```

The decoder output will name the competition behind `0x6014fb0`
and the return address of the mid-season caller. Both are the
gates for the remaining C11.3 points (2–16).

## What Phase A does not commit anyone to

* No Rust changes yet — no new triggers invented, no
  `hook_year_rollover` edits, no test skeletons for hypothetical
  triggers.
* No unfreezing of anything from C11.2.
* No claim of a fix.

Phase A is instrumentation only. Points 1–16 of the tranche's
report land after the enhanced capture data exists.

## Confidence tag

Fixture generation engine: **FROZEN** (C11.2, `10734b5`) —
unchanged by this tranche.

Fixture lifecycle dispatcher: **KNOWN BUG** (mid-season regen
missing) — evidence in `season_capture_2001_02_findings.md`.
Fix pending Phase B (Rust changes) which requires the enhanced
capture data.
