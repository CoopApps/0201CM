# Fixture integration — Phase 7: natural GDI trace captured + first divergences measured

Date: 2026-09-13. **Authoritative spec**: `D:/cm0102/cm0102_GDI.exe`.

## Milestone

**Natural cm0102-gdi.exe execution of the English Second Division
constructor captured end-to-end.** After 6 phases of preparation the
trace lives at
`reports/fixture_disasm/runtime/20260913_175838_manual_*`.

Trace contents:
- Pre-perturb 24-club roster with Club* dereferenced
- Post-perturb 24-club roster
- 2990-byte schedule buffer
- 46 walker calls (full state trace)
- 19 pool RNG calls (1 in perturb, 18 in driver)
- 792 (!) fixture insertions
- `alt_pair_list = 0x0` — comp+0xea alt path is NOT reachable for eng_second

## Critical divergences measured

### 1. Fixture count: Rust 552 vs GDI 792

The single largest divergence. My structural driver port emits 552
fixtures (24 × 46 / 2 — the pure double round-robin count). Natural
execution emits **792**.

Round distribution:
- Rounds 0-25 (first half): 12 fixtures each = 312 ✓ matches Rust
- Rounds 26-45 (second half): **24 fixtures each** = 480 (Rust: 12 each)
- Total 792 vs Rust 552, diff 240

The second half emits double the fixtures per round. Not yet
explained. Candidates:
- Reset pass at end of driver (schedule[round]+0x0b == 3 rounds) may
  insert extra fixtures via `FUN_0066a4d0`.
- Reserve league fixtures may be interleaved with league fixtures in
  the same driver call.
- Second-half loop iterations may generate both forward AND reverse
  fixtures (rather than just the reverse).

**Investigation required** before wiring the Rust driver into
production.

### 2. Schedule buffer: differs at byte 474

Natural buffer vs prior direct-call GDI buffer (which matches Rust
byte-exact) differ at:

- Byte 474 = round 7, sub-slot 2 aux payload (offset +0x13)
- Natural: `0x02`; direct-call + Rust: `0x00`

This means during **natural** construction, additional writer calls
populate sub-slots 1..7 of certain rounds. My direct-call to the
schedule getter with arg1=0xFF only writes sub-slot 0. Natural
execution has additional data injected by some other code
(possibly the reset pass, possibly cup-schedule integration, possibly
part of the full ctor beyond just the schedule getter).

The date/type/prize fields (owned by round-writer) still match.

### 3. Walker flag_byte was 0x03, not 0

My hooks captured `flag_byte = 0x03` for every one of the 46 walker
calls. My port only tested the 0x40 and 0x80 bits; bits 0x01 and
0x02 don't affect the walker state machine per the decompile, so
this is likely fine. The walker output matched my port's expected
behavior for the first 5 calls (verified against captured trace).

### 4. Perturbation RNG (partial capture)

- **Phase B pool RNG**: `rand_mod(30000) → 7392` (single call as
  expected)
- LCG calls: **not captured** in this session — I only hooked pool
  RNG. Need to also hook the LCG (`FUN_00935a94` @ some GDI VA) for
  a complete perturbation RNG trace.

### 5. Pre-perturb roster recovered

24 club IDs, in this order:

```
slot  0: club_id=720  slot  1: 769   slot  2: 791   slot  3: 797
slot  4: 800          slot  5: 831   slot  6: 887   slot  7: 904
slot  8: 984          slot  9: 1046  slot 10: 1790  slot 11: 2485
slot 12: 2492         slot 13: 2533  slot 14: 2678  slot 15: 2734
slot 16: 2774         slot 17: 2830  slot 18: 3189  slot 19: 3213
slot 20: 3307         slot 21: 3511  slot 22: 3536  slot 23: 3539
```

These are the actual English Second Division clubs. IDs are
monotonically increasing — the pre-perturb order is likely alphabetical
by internal club name (that pattern is common). This is the input
Rust needs to feed into the perturbation.

### 6. Post-perturb roster (partial)

The captured post-perturb dump has slot indices > 24 which suggests
a hook-side issue during data serialization. The bytes-level buffer
is 30621 bytes long (expected 1416 = 24 × 0x3b) — likely capturing
past the end of the Club table into adjacent memory. Need to re-run
capture with tighter bounds. The `clubs_before_perturb_deref` was
clean.

## What we now know for certain

1. `comp+0xea = 0x0` for English Second Division → **alt path NOT
   reachable**. This resolves a Phase 3 open question. The Rust
   driver's `alt_pair_list: Option<&[u8]>` parameter can always be
   `None` for eng_second production wiring.
2. Comp id 9 IS English Second Division at the ctor level (24 clubs,
   46 rounds, year 2001).
3. The driver's outer loop DOES fire 46 walker calls total (matches
   Rust). The 792 fixture count comes from something INSIDE the
   inner loop, not from a longer outer loop.
4. The perturbation makes exactly ONE pool RNG call
   (`rand_mod(30000)`). Rust port has this correct.

## Files added

- `reports/fixture_disasm/runtime/20260913_175838_manual_natural.jsonl`
  — 18 events with all trace data
- `runtime/20260913_175838_manual_sched_buffer_0.bin` — 2990-byte
  schedule buffer (natural)
- `runtime/20260913_175838_manual_clubs_before_perturb_0.bin` — pre-
  perturb 1416-byte roster
- `runtime/20260913_175838_manual_clubs_after_perturb_0.bin` — post-
  perturb dump (needs re-capture with tighter bounds)
- `runtime/20260913_175838_manual_clubs_table_0.bin` — final roster
  after ctor
- `reports/fixture_disasm/gdi_wait_and_capture.py` — the Frida
  harness that produced this trace
- `reports/fixture_disasm/analyze_natural_trace.py` — parser +
  differential

## Confidence ledger

| Component | Status |
|-----------|--------|
| Schedule primitives (date, writer, slot writer) | BYTE-EXACT — reconfirmed |
| Matrix seeder | BYTE-EXACT — reconfirmed |
| Walker (0x0066ee40) | STRUCTURALLY PORTED. State machine matches for first 5 calls. Full 46-call diff pending. |
| Perturb (0x0066b900) pool RNG | STRUCTURALLY PORTED. Confirmed 1 rand_mod(30000). LCG capture pending. |
| Perturb (0x0066b900) output | UNKNOWN — post-perturb capture needs re-run with clean bounds. |
| Round-robin driver (0x00668450) | **DIVERGENT**. Emits 552 fixtures; GDI emits 792. |

## Non-claims

- Second Division is NOT wired into production.
- The Berger add-mod stub is NOT retired.
- The fixture-count discrepancy blocks wiring.
- The natural schedule buffer differs from my Rust construction at
  byte 474 (sub-slot 2 aux payload in round 7).

## Immediate next investigation

1. Understand the 792 vs 552 divergence. Options:
   - Add a hook on FUN_0066a4d0 (replay/reset handler) and count
     its fixture inserts.
   - Split driver body: is `FUN_00594eb0` called from the reset pass
     or the alt path or the main outer loop? (Answer: alt is
     disabled; reset pass may be firing.)
   - Cross-check with fixtures_2002.tmp: those 240 second-half
     fixtures had `home_id`, `away_id` — do the natural trace's
     duplicated second-half rounds have distinct pair sequences
     from the main 12?
2. Hook the LCG (need to find its GDI VA) to get the full perturbation
   RNG trace and match Rust against it.
3. Fix the post-perturb capture bounds.
4. Then run Rust perturbation with the captured pre-perturb roster +
   RNG seed and compare the post-perturb slot ordering.

## Commits

- `fe2eae1` Phase 1 — dates + walker
- `95467ac` Phase 2 — matrix seeder
- `7b9528b` Phase 3 — perturbation + driver (later corrected)
- `252aef0` Phase 4 — clubs-table model correction
- (Phase 5 commit) — fixtures_2002.tmp recovery
- (Phase 6 commit) — automation harness
- (this commit) — first natural cm0102-gdi capture of eng_second_ctor
  end-to-end. Fixture-count divergence surfaced.
