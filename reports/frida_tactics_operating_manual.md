# Frida tactics decoding — operating manual

## What this closes

The remaining tactic-file decoders that can't be pinned from `.tct`
save-diffs alone:
- `per_slot_instr` u16 individual bits (Cross Ball / Try Through Balls /
  Long Shots / Run With Ball / Hold Up Ball / Free Role / Set Pieces D+A)
- 133-byte Unknown Region A at file `0x0079` (Playmaker + Free Kicks
  L/R + Corners L/R spinner-to-player-slot assignments)
- Per-tick engine consumption sites (which fn reads which byte)
- Setter cluster fns (which fn writes which bit when user clicks a toggle)

## Prerequisites

1. `cm0102.exe` running on Windows (native) OR under Wine on Linux/macOS
2. `frida-python` installed in the cm-lift venv (already in pyproject)
3. On Windows, no additional setup — Frida attaches to a running .exe
4. On Wine, run `frida-server-*.exe` inside Wine first, then attach from host

## Three-stage decode workflow

### Stage 1 — LOCATE the runtime tactic buffer

```bash
python -m cm_lift.frida_tactics locate
```

This attaches to `cm0102.exe` and hooks:
- `msvcrt!fread` — catches any 1476 or 1432-byte read (tactic file load)
- `FUN_00890b30` — the tactic-struct copy fn (from [prior decode](tactic_file_layout_audit.md))

**User action:** in-game, open **Load Formation**, pick any tactic (e.g.
`WWW2 Hard Tackling`), click OK.

**Expected output:**
```
  {"kind": "ready", "imgbase": "0x400000"}
  {"kind": "tactic_loaded", "buf": "0x1a2b3000", "size": 1476, "version": 10021982}
  {"kind": "tactic_copied", "dst": "0x1c5d0000", "src": "0x1a2b3000"}
```

**Record the `dst` address** from the `tactic_copied` event — that's the
runtime tactic-struct base in the game's slot pool. This is what stages 2
and 3 target.

### Stage 2 — WATCH writes during UI toggles

```bash
python -m cm_lift.frida_tactics writes 0x1c5d0000
```

(replace `0x1c5d0000` with the actual `dst` from Stage 1.)

Sets up a `MemoryAccessMonitor` over the 1476-byte struct.

**User action:** in-game, open **Team Instructions** or a per-player
tactics view. Toggle ONE setting. The Frida log fires a write event
naming:
- `offset` (0..1475) — which byte changed
- `value` — the new byte value  
- `pc` — the exe address of the fn that wrote the byte

Repeat for each unknown toggle. Each write event pins:
1. Which BYTE holds the setting → maps to struct field
2. Which FN wrote it → the setter to port

**Example decode session:**
```
User: toggles "Cross Ball" for player 5 from No to Yes
Log:  WRITE offset=0x010c val=0x02 pc=0x006a3f28
Interpretation:
  offset 0x010c = 0x0102 + 5*2 = per_slot_instr[5]
  bit 1 (0x02) = Cross Ball
  Setter fn at 0x006a3f28
```

### Stage 3 — WATCH reads during a match

```bash
python -m cm_lift.frida_tactics reads 0x1c5d0000
```

Same MemoryAccessMonitor, but the caller filters read events instead of
writes.

**User action:** start a match with the tactic loaded. Every per-tick
tactic-consumption site fires a read event. Groups of reads at
specific PCs identify the tactic-consumer fns.

**Payoff:** for each byte range read during a tick, we know:
- Which fn CONSUMES it (the calling PC)  
- WHEN in the tick it consumes it
- WHICH bit it masks

That's the ground truth the Rust port's consumption sites should mirror.

## Advisory

Per `[[frida-instrumentation-crash-evidence]]` in Claude's memory: heavy
Frida hooking crashed cm0102.exe twice identically in prior tests. Light
hooking on a different save didn't crash.

This module deliberately uses **surgical hooks only**:
- 2 fn interceptors in Stage 1 (fread + FUN_00890b30)
- 1 MemoryAccessMonitor region in Stages 2/3

Both should stay well under the crash threshold. If the exe crashes:
- Try a fresh game start
- Reduce the monitor region size (e.g. watch only bytes 0x100..0x150)
- Fall back to fewer hooks per stage

## Landing the results

Each Stage-2 write event produces a single-bit ground-truth anchor.
Format for the port:

```rust
// VERIFIED from Frida capture — writes at offset 0x010C when user
// toggles Cross Ball for player 5:
//   before: 0x0000
//   after:  0x0002
// Setter fn: FUN_006a3f28 (call this fn address `set_cross_ball`)
pub const PER_SLOT_INSTR_BIT_CROSS_BALL: u16 = 0x0002;
```

Every decoded bit → one commit + one test. Same pattern as the earlier
author-then-diff work (see commits `a6d3e50`, `879ad8d`).

## Non-Frida fallback if setup fails

If Frida can't attach for any reason (permissions, Wine setup, etc.),
the fallback is the author-then-save-then-diff loop from the earlier
`442.tct → 442-1 change.tct` methodology — slower but works from
save files alone.
