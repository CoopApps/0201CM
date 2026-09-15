# C11.3 Phase A.2 — the __thiscall diagnosis

**Prior tranche**: `66f9af3` (Phase A harness enrichment) + `28198ba`
(2-season findings). Both interpreted the schedule_getter's
`arg0 = -1` / `arg0 = 0` as sentinel values indicating comp
identity of "-1 = all leagues" or "0 = null query". Phase A ran a
new capture (`season_2001_02_v2_thiscall_diagnostic.jsonl`, 11,818
events, ~82 min wall clock, 2 seasons) with an enriched decoder
that added `return_addr` and `arg0` deref. Two things fell out:

1. **`return_addr` immediately pinned the caller**: 3 of 3 `-1`
   calls came from `0x5605a5`, both `0` calls from `0x5608c9`.
   Those addresses sit inside `sub_00560520` (146 bytes) and
   `sub_00560810` (480 bytes) respectively.

2. **Reading `sub_00560520`'s disassembly** revealed the harness
   had been interpreting the arguments wrong all along.

## The disasm proof

`sub_00560520`, first 30 lines of the linear-carved GDI asm
([reference](../../cm0102-carve/gdi_carve/functions/00004-PE_section_.text/02038_sub_00560520.asm)):

```
00560520  push    esi
00560521  mov     esi, ecx        ; save `this`
...
00560588  lea     ecx, [esi + 0x3a]     ; temp
0056058b  push    0                     ; arg 4 = 0
0056058d  lea     edx, [esi + 0xa9]     ; temp
0056058e  ...
00560596  mov     eax, dword ptr [esi]  ; vtable ptr
00560598  push    ecx                   ; arg 3
00560599  push    edx                   ; arg 2
0056059a  push    -1                    ; arg 1 = -1
0056059c  mov     ecx, esi              ; RESTORE `this` in ECX
0056059e  mov     byte ptr [esi + 0x49], 5
005605a2  call    dword ptr [eax + 0x3c] ; vtable[+0x3c] -> schedule_getter
005605a5  mov     dword ptr [esi + 0xba], eax  ; <-- return_addr we captured
```

The `mov ecx, esi` at `0056059c` immediately before `call [eax + 0x3c]`
is the Windows `__thiscall` convention: **the callee's `this`
pointer lives in ECX, not on the stack**. Stack args on entry to
`sub_0055F540` are:

* `arg0` (first stack arg) = **`mode`**: `-1` = regenerate,
  `0` = query
* `arg1`, `arg2` = pointers to two output buffers on `this`
* `arg3` = flag (0 here)

Frida's `args[0]` in `onEnter` maps to the first *stack* arg — so
what we've been calling "arg0" and interpreting as competition
identity is actually the **mode flag**. The competition identity
was in ECX all along and was never recorded.

## What this changes retroactively

* **The 5 schedule_getter calls in both captures had unknown comp
  identity.** All the analysis in `reports/season_capture_2001_02_findings.md`
  that claimed "same comp 0x6014fb0 in bursts 3 and 7" was
  incorrect — `0x6014fb0` was a **driver arg0** (different
  function, different call site), not a comp identity for the
  getter. The idea that mid-season regen targets one specific
  calendar-year league remains a good working hypothesis but is
  no longer supported by that specific piece of evidence.
* **The `sentinel: -1` records in Phase A output** aren't sentinels
  at all — they're the mode integer. The label was misleading and
  the retroactive interpretation of "no comp record" is a
  side-effect of the harness bug, not of the exe's behaviour.

## What we still know (survives the fix)

Independent of the arg-decoding bug:

* **Bi-modal behaviour confirmed**: 3 `-1` calls (regen) and 2 `0`
  calls (query) fired over 2 seasons. Both captures agree on the
  count and timing.
* **`-1` caller is `sub_00560520`**, `0` caller is `sub_00560810`.
  That's not affected by the arg-decoding bug — the `return_addr`
  hook reads the stack directly.
* **`sub_00560520` hardcodes `push -1`** and calls `vtable[+0x3c]`
  on ECX. Whatever object it's a method of, its schedule reset
  ALWAYS goes through `-1` mode.
* **`sub_00560520` is small (146 bytes) and looks like a
  competition "reset schedule" method** — writes many small
  byte-fields at `this + 0x3c..0xc7` (season config bytes), reads
  two globals at `0x9BB9C8` and `0x9BB9D0` (likely current
  calendar-year / season-start config).

## The A.2 fix

`season_capture.js` — the enriched `comp_and_caller` / `schedule_getter`
arg_shape now records:

* `stack_arg0_raw`, `stack_arg0_int` — the mode integer (renamed
  to make its meaning explicit; used to be `arg0_*`)
* `this_ptr` — pointer read from `this.context.ecx` at the
  Interceptor's `onEnter` (Frida exposes CPU registers via
  `context.ecx` on 32-bit processes)
* `this_record` — 128-byte hex dump if `this_ptr` is a plausible
  heap pointer, using the pool-record layout to extract id +
  name + three_letter + nation + reputation

`decode_schedule_getter_calls.py` — the decoder now accepts both
v1 (pre-A.2, `arg0_*` + `comp_record`) and v2 (post-A.2,
`stack_arg0_*` + `this_ptr` + `this_record`) records, so old
captures don't break.

Preflight verifier still passes: same 7 Group B hooks + G,
0 mid-function VAs.

## What A.2 does NOT do

* No re-run of the capture — that's the user's next step.
* No Rust changes.
* No claim about `0x6014fb0` any more — that was a mis-labelled
  finding.
* No claim of a specific trigger.

## Next: one more capture with the fixed harness

Same command:

```
D:/Python312/python.exe D:\cm0102-rs\tools\gdi_capture\season_capture.py --out D:\cm0102-rs\season_2001_02_v3.jsonl
```

Then:

```
D:/Python312/python.exe D:\cm0102-rs\tools\gdi_capture\decode_schedule_getter_calls.py D:\cm0102-rs\season_2001_02_v3.jsonl
```

The `this-> id=…  name=…` output on each schedule_getter call
will name the competition byte-exactly. That's point 1 of the
tranche and settles points 2, 3, 6 in one pass.

## Meta lesson

I encoded a specific interpretation of the record layout without
checking the calling convention. The reading of `arg0 = -1` as
"sentinel" was internally consistent but wrong. Ports of exe
data structures need calling-convention verification any time a
new interception point is added — not just a struct-layout
verification.

## Files changed

* `tools/gdi_capture/season_capture.js` (arg_shape body)
* `tools/gdi_capture/decode_schedule_getter_calls.py` (accepts both formats)
* `fixtures/gdi_captures/season_2001_02_v2_thiscall_diagnostic.jsonl`
  — keeps the v2 capture as the reference that revealed the bug
