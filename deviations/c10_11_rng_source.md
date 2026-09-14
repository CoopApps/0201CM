# C10.11 — RNG-source exactness (in progress)

**Goal**: prove `GameRng::from_state(cursor, jitter, lcg_state)` reproduces
the GDI RNG stream for the full 5-English-league fixture-generation
chain, with NO captured playback queues.

## Method

Harness (`gdi_five_league_lineage.py`) was extended with `snapshotRng()`
which reads the three GDI RNG globals:

- `DAT_00dc7180` = pool cursor pointer (dword)
- `DAT_00dc717c` = pool jitter (word)
- `DAT_00ac2610` = LCG state (dword)

`cursor_off = cursor_ptr − POOL_BASE (0x00a8de80)` is the byte offset
into POOL that `GameRng::rand_mod` uses internally.

Snapshots are taken at each league's ctor entry (before any RNG
consumption for that league) and ctor leave (after all fixture-
generation RNG consumption is complete).

Diff runner (`five_league_diff.rs` — `full_chain_rng_diff`) seeds
`GameRng::from_state(rng_initial)`, runs `matrix_perturb` +
walker_step replay through that shared rng (no `queue_pool_returns`,
no `queue_lcg_returns`), and compares the resulting state to
`rng_final`.

## Result — capture 20260914_161202

### RNG stream continuity across leagues — PROVED

| League | initial cursor,lcg | final cursor,lcg | Δcursor |
|---|---|---|---|
| Prem  | 1992, 1726273615 | 2048, 23296 | 56 |
| First | 2048, 23296      | 2124, 23618 | 76 |
| Second| 2124, 23618      | 2212, 19966 | 88 |
| Third | 2212, 19966      | 2296, 24323 | 84 |
| Conf  | 2296, 24323      | 2368, 19983 | 72 |

`prem.rng_final == first.rng_initial`, and so on down the chain.
The GDI build does NOT reset the RNG per league; it consumes one
continuous stream. Prem's initial LCG `1726273615` reflects boot
entropy already consumed before Prem's ctor runs.

### Cursor and jitter reproduction — PROVED, all 5 leagues

Seeding `GameRng::from_state(init)` and running the algorithmic
perturb + walker chain produces the exact final `cursor_off` and
`jitter` values reported by the exe:

| League | rust.cursor | exe.cursor | rust.jitter | exe.jitter |
|---|---|---|---|---|
| Prem   | 2048 | 2048 | 26340 | 26340 |
| First  | 2124 | 2124 | 26340 | 26340 |
| Second | 2212 | 2212 | 26340 | 26340 |
| Third  | 2296 | 2296 | 26340 | 26340 |
| Conf   | 2368 | 2368 | 26340 | 26340 |

Every `rand_mod` call advances the cursor by exactly 4 bytes. Because
our final cursor matches the exe's exactly for every league, the port
performs exactly the same **number** of pool RNG calls as the exe.
This is a strong control-flow proof: the perturb + walker code paths
are byte-exact-equivalent in RNG-consumption count.

Also 0-diff: `perturb P2` for all 5 leagues (algorithmic RNG). Perturb
consumes only LCG, so it does not touch the pool.

### LCG state and pool-return values — DIVERGENT

| League | rust.lcg_final | exe.lcg_final | walkerΔ |
|---|---|---|---|
| Prem   |   7641 | 23296 | 6/38  |
| First  |   1420 | 23618 | 12/46 |
| Second |  22854 | 19966 | 11/46 |
| Third  |  25500 | 24323 | 12/46 |
| Conf   |  27125 | 19983 | 9/42  |

The walker retvals in the pure-algorithmic Rust path diverge from
captured returns for 6..12 of each league's ~40 walker calls.
Since **cursor matches exactly** and jitter is stable at 26340
(no pool wrap), the divergence is not in the number of pool reads,
which means the returned value differs at those positions. The
formula is `(jitter + POOL[cursor]) % n` for small `n`. Both `jitter`
and `n` (=4) match; only `POOL[cursor]` can differ.

**Conclusion**: the `crates/cm-domain/assets/game_rng_pool.bin`
asset does not byte-match the pool memory of the running GDI build
at these offsets. Same story for the LCG divergence — Rust's
algorithmic `msvc_rand()` matches the exe's when driven from the same
state, but perturb's `lcg_srand` reseeds mid-way, then a downstream
consumer we're missing keeps the exe's LCG on a different trajectory.

## What we've established

1. RNG stream is one continuous chain across all 5 English leagues.
2. Our port makes exactly the same number of pool RNG calls as the
   exe (5-league byte-exact cursor and jitter reproduction).
3. Perturb produces byte-exact P2 club_id lists from the algorithmic
   LCG stream, seeded from captured initial `lcg_state`.
4. Walker + driver produce byte-exact fixture emissions on all 5
   leagues **when fed captured pool returns** (playback path).

## What remains — pool asset byte-diff

The next action is not a code change; it is a data-diff. The harness
now (as of this commit) also emits `pool_slice` with 512 bytes of
POOL starting at each league's initial cursor. Comparing those slices
byte-for-byte against `assets/game_rng_pool.bin` at the same offsets
will pinpoint whether:

- our pool asset was extracted from `cm0102.exe` (DirectDraw) but
  differs in `cm0102_GDI.exe`, or
- the pool is dynamically initialized per boot and our snapshot is
  from a specific run, or
- there is a per-boot state (a header, a seed count) we are missing.

Once the pool asset is byte-matched, Rust's algorithmic path is
expected to reproduce every pool return and thus every walker retval
byte-exactly, closing C10.11 with full RNG-source exactness.

Playback queues stay in `GameRng` only as a diagnostic aid (they
are already used to prove the walker + driver algorithm layers are
byte-exact); they are not part of the final production wiring
promise.

## Confidence label — updated

- Prem perturb algorithm: `ByteExact` (from-state seed, no queues)
- First/Second/Third/Conf perturb algorithm: `ByteExact` (same)
- Walker algorithm: `BehaviourallyExact` when fed correct pool
  bytes; algorithmic pool reproduction pending pool-asset fix.
- Driver algorithm: `ByteExact` on 552-fixture emission when
  fed captured P2 + walker seq. Algorithmic reproduction pending
  the pool-asset fix above.
