# Accuracy Gate

This remake is a transplant first. Runtime systems must not be invented just
because a modern manager game needs them.

## Rule

No gameplay behavior enters the Rust runtime until it has one of these:

- `CODE_DERIVED`: read from CM0102 code via `D:/cm0102-carve`
- `STATIC_LIFT`: hardcoded address/offset lifted and waiting for verification
- `RUNTIME_CAPTURE`: debugger confirmation of an already-lifted claim

Anything else is `not implemented`.

## Current Verified Execution Model

- Entry: `WinMain 0x006725c0`
- Init / DB load: `0x005b6940 -> 0x005121a0`
- Main loop shell: `while 0x005b6920 { 0x00672770 }`
- Message pump / per-iteration shell: `0x00672770`
- Startup/setup flow: `0x005b6920 -> 0x005b6f10 -> 0x00803e00`
- Simulation loop frontier: `0x005b6a90`
- Date add-days helper: `0x00536190`
- Match-day frontier: `0x00699640 -> 0x0069aa70 -> 0x00699d90 -> 0x0069d950 -> 0x006c0f10`
- Save game: `0x004e24e0`
- Load game: `0x0089bd60`
- Match RNG: `0x008fc4f0`
- Match RNG initializer: `0x008fc5d0`
- MSVC rand: `0x00935a94`

Source: `D:/cm0102-carve/EXECUTION_MODEL.md`, `PROGRESS.md`, and `claims.json`.

## Binary Validation

Run this whenever we promote carve-derived behavior into Rust:

```powershell
D:\cm0102-rs\target\debug\cm-app.exe validate-original-binary D:\cm0102\cm0102.exe D:\cm0102-rs\reports\binary_validation.json
D:\cm0102-rs\target\debug\cm-app.exe validate-execution-model D:\cm0102\cm0102.exe D:\cm0102-rs\reports\execution_validation.json
D:\cm0102-rs\target\debug\cm-app.exe validate-simulation-frontier D:\cm0102\cm0102.exe D:\cm0102-rs\reports\simulation_frontier.json
D:\cm0102-rs\target\debug\cm-app.exe validate-runtime-simulation D:\cm0102-rs\rust-db D:\cm0102-rs\reports\runtime_simulation.json
D:\cm0102-rs\target\debug\cm-app.exe validate-rng D:\cm0102\cm0102.exe D:\cm0102-rs\reports\rng_validation.json
D:\cm0102-rs\target\debug\cm-app.exe extract-rng-table D:\cm0102\cm0102.exe D:\cm0102-rs\reports\rng_table.json
```

This parses the PE headers, maps verified virtual addresses into the original
binary, and checks key strings used by the carve evidence. It is not a substitute
for reading/decompiling behavior, but it catches accidental drift between the
carve and the binary we are rebuilding.

`validate-execution-model` keeps the runtime boundary honest. `0x00672770` is
validated as a Win32 message pump/per-iteration shell (`PeekMessageA`,
`GetMessageA`, `TranslateMessage`, `DispatchMessageA`) and not as the proven
day-advance implementation. The startup path `0x005b6920 -> 0x005b6f10 ->
0x00803e00` is validated separately, including the optional `-seed` route into
the RNG initializer.

`validate-simulation-frontier` records the first code-derived gameplay loop
shape. `0x005b6a90` compares current packed date globals against target date
state, builds per-date match-day queues via `0x00699640`, annotates those queues
via `0x0069aa70`, runs the match-day processor/setup dispatcher `0x00699d90`,
calls per-club callbacks, increments the phase global `DAT_00acde88`, and when
that phase exceeds `2` resets it to `0` and advances date state through
`0x00536190`. This proves a three-phase simulation frontier, not the full
semantics of each phase.

The current frontier report also classifies the major phase-2 call buckets:
`0x004cdef0` as a large date-sensitive data update frontier, `0x00595580` as a
fixture cleanup frontier, `0x005e4370` as a date/RNG-driven schedule frontier,
`0x00674c10` as a manager-manager frontier, and `0x00844940` as a stadium/date
cleanup frontier. `0x00844940` now has a code-derived frontier shape: a day-zero
setup call, a pre-2003 stadium slot clear/cache path, an exact 2003 restore path,
and a 30-day cleanup cadence over sorted 12-byte date records. The target record
array and mutation semantics remain frontier-only until their owning data model
is lifted.

The match-day path now has a code-derived setup boundary. `0x00699640` zeros the
builder state, gathers fixture rows into 0x18 competition groups, 0x54 match
groups, and 0x69 fixture snapshots, sorts fixture slices, and creates the
match-day scratch list. `0x0069aa70` walks those queues, checks human-manager
visibility and active staff records, and marks visible fixture/group flags.
`0x00699d90` allocates per-fixture 0x11d scratch state, scans 16 active staff
slots from 0x6e-byte records, assigns temporary match links, calls verified
`match_setup 0x0069d950`, pumps UI/messages, and frees the scratch state.
`0x0069d950` anchors the fixture at match-state `+0x4792`, configures team/player
arrays at `+0x4796` and `+0x6a6e` through `0x006c0f10`, uses verified
`match_random`, and queues 0x19-byte match incidents. `0x006c0f10` writes the
team-control header, links fixture/team/club inputs, copies a 0x18e3-byte team
block into match-state `+0x91e2 + team_index*0x18e3`, derives the visible squad
count from fixture byte `+0x4d`, and loads tactics through `0x008830a0`.
`0x008830a0` is tactics.cpp-attributed and copies one 0x91-byte tactics block
after resolving an index through `0x00882f60`. `0x006a1470` is a setup frontier
called from match setup: it scans 20 player slots per team from `+0x4796` with
0x1be stride, updates a team short at `+0x1ce + team_index*2`, samples verified
`match_random(20/7000/6)`, and calls deeper helpers `0x006d1780`/`0x006d46c0`.
`0x00882f60` resolves tactic indexes, including human/club-state checks through
`0x005ea590` and `0x0052a500`, a club tactic pointer at `+0xcf`, and a `-1`
blocked lookup result. `0x00882240` scans up to 20 slots in a 0x91-byte tactic
block and joins slot staff ids against 0x6e-byte staff records at
`DAT_00acd5c4`.
`0x006a91d0` and `0x006a9200` are tiny tactical flag readers: they read player
slot byte `+0x19`, side byte `+0x27`, then return a u16 from match-state pointer
tables `+0x8ebc` or `+0x8ec4`. `0x006d9ea0` seeds player-slot bytes
`+0x104..+0x10d` from verified `match_random` using bounds at `+0x10e..+0x117`.
`0x006d1a20` is now a code-derived player evaluation frontier: it derives short
`+0x3b`, writes many float fields including `+0x7d/+0x8d/+0x91/+0x95/+0x99`,
uses tactical flag readers, and applies random float jitter through `0x00935080`.
`0x006d46c0` is a code-derived player action-score frontier: it calls
`0x006d1a20`, resets/accumulates short `+0x37`, branches heavily on tactical flag
masks, adds deltas from linked player-data bytes `+0x17..+0x1a`, and can call
`0x006d9ea0`. `0x005a2c70` and `0x005a30d0` are formation.cpp-attributed mask
classifiers over formation tables `+0x12d/+0x14e`, using masks including
`0x880`, `0x40`, `0x20`, `0x10`, and `0x8`. `0x00935080` is a thin random-float
jitter wrapper into `0x009350a2`; it is used by player evaluation/action-score
frontiers, but the exact distribution is still not lifted. This is still
frontier-level: tactical decisions, exact player weighting formulas,
injury/fatigue semantics, minute ticks, and scoreline mutation are not
implemented until those formulas are fully lifted from the binary.

The next match-player action layer is also code-derived. `0x006b2cb0` walks a
candidate pointer list using count `param_3+side+0x58` and 0x2c stride, filters
candidate slots by `+0x2b` and `+0x19`, calls `0x006db630`, and clears
match-state bytes `+0x1a6/+0x1a5` on success. `0x006db580` reads player position
bytes `+0x102/+0x103`, side byte `+0x27`, match-state bounds `+0x8eaf/+0x8eb0`,
and dispatches to `0x006d63f0`. `0x006db630` is a match_pl.cpp action-attempt
frontier: it uses tactical flags, verified `match_random` thresholds, emits
event codes through `0x006bc8d0`, mutates match-state bytes
`+0x8ea7/+0x8ea8/+0x8eae/+0x8eb2`, and updates counters `+0x4782/+0x478a`.
`0x006d63f0` resolves movement/actions, increments player byte `+0x2b`, uses
seed bytes `+0x104/+0x107/+0x10a/+0x10c`, writes action/drift shorts
`+0x198/+0x19c`, emits events through `0x006bc8d0`/`0x00672320`, and can recurse
for chained action resolution. `0x006f99c0` selects player actions using
match-state bytes `+0x8ea7/+0x8ea8/+0x8ea9/+0x8eae` and action codes including
`0x68/0x69/0x6a/0x6b/0x76/0x100/0x105`. `0x006f63f0` dispatches match events by
switching on match-state byte `+0x8eb2`, clearing it, and calling event helpers
including `0x006ac3b0`, `0x006dfc50`, `0x006e65e0`, `0x006e7a60`, and
`0x006dfe90`.

The match event queue is now code-derived. `0x006bc8d0` is match_events.cpp
attributed and accepts event codes `8000..0x21e4`, normalises some through
`0x006bba10`/`0x006bb660`/`0x006bb6e0`, appends 0x0e-byte event slots at
`+0x30 + count*0x0e`, writes code/flags/participants/payload, mirrors selected
events to `+0x720`, maintains counters at `+6/+8/+0xa/+0xc/+0xe`, and recursively
emits follow-up codes including `0x21a0`, `0x219f`, `0x21e3`, `0x21c0`, and
`0x21bf`. `0x006dfc50` and `0x006dfe90` are follow-up frontiers: they increment
player byte `+0x2b`, use position bytes `+0x102/+0x103` and match-state action
bytes, call spatial/candidate helpers, emit `0x1f78`, and mutate match-state
action bytes including `+0x8ea7/+0x8ea8/+0x8eb2/+0x8eae` plus side bucket
`+0xf5ca`. `0x006e65e0` is a shot/action score frontier: it selects action bytes
including `0x16..0x1d`, `0x33`, `0x35`, `0x39`, and `0x3a`, writes event outputs
such as `0x1f7f/0x1f81`, computes player-slot score short `+0x39`, reads many
player short inputs around `+0x146..+0x180` plus `+0x198/+0x19c`, and uses
verified `match_random`. Event text meanings and score formulas remain
frontier-only.

The enclosing match step/control layer is now code-derived. `0x0069f2f0` is a
match_eng.cpp-attributed step controller: it reads the fixture through
match-state `+0x4792`, loops while control byte `+0x8eb4` is active, switches on
phase byte `+0x8eb3`, dispatches possession phases through `0x006a4020`,
resolves stored action scratch through `0x006a0550`, advances tick counters
`+0x8ed0/+0x8ed2`, writes fixture status byte `+0x43`, and emits event
codes including `0x217b/0x2002/0x2003/0x2004`. `0x006a4020` is the phase
possession controller: it resets scratch offsets `+0x475a..+0x4769`, selects
player slots from `+0x4796` using 0x1be stride, calls shot/action scoring
`0x006e65e0` and event resolution `0x006f63f0`, and writes fixture score bytes.
`0x006f5de0` is a pressure/action continuation frontier over candidate links at
player-slot `+0x1ae`/`+0x101`, verified RNG bounds, action short `+0x198`, and
dispatch through `0x006fa740/0x006f99c0/0x006f63f0`. `0x006a0550` is the
stored-action resolver over scratch bytes `+0x475a..+0x4769`, active pointers
`+0x4761/+0x4765`, stored-action event codes, and match-state bytes including
`+0x8ea7/+0x8ea8/+0x8eae/+0x8eb2/+0xf582/+0xf5ca`. These are still
frontier-level until their exact formulas and event text semantics are lifted.

The next helper layer under the match controller is also code-derived.
`0x006a1320` is a tiny action-scratch reset helper over `+0x475a..+0x4769`.
`0x006a3240` is a period transition frontier: it reads `+0x8ed4/+0x8ed0`,
handles thresholds `0x1ef/0x3de/0x483/0x528`, writes fixture bytes
`+0x43..+0x48`, emits `0x20f1/0x20f2/0x20f3`, and resets player slots through
`0x006db210`. `0x006b4510` is a match_eng.cpp player candidate selector that
scans 20 player slots from `+0x4796` with 0x1be stride, skips active pointers
`+0xf59e/+0xf5a2`, uses coordinates `+0x8ea7/+0x8ea8`, tactical table `+0x8ebc`,
role/spatial helpers, and verified `match_random`, then returns the
highest-scoring player pointer. `0x006aae20` is a per-tick tactical state updater:
it clears scratch/counters, derives a minute bucket from `+0x8ed0`, updates side
blocks around `+0x904d/+0x911d`, emits tactical/commentary codes including
`0x21cf/0x21c1/0x2137..0x213d`, can call `0x006a1470`, and branches on
match-state `+0x8eb2`. The selection scores, period semantics, and commentary
meanings remain frontier-only.

`0x004cdef0` now has a code-derived staff/contract date-renewal frontier shape.
It builds renewal windows from +7 days through +0x447 days using `0x00536190`,
shows "Updating game data" progress via `0x007ead30`, walks 0x6e-byte staff
records, maps 0x4f-byte staff side-state entries to 0x50-byte event/contract
records, resolves linked 0x245-byte club records, age-gates decisions through
database helper `0x005246e0`, and dispatches contract/status outcomes through
helpers including `0x004dc980`, `0x004dabd0`, and `0x004dcf60`. The exact
contract, news, and staff-state mutations remain frontier-only until their
record layouts are lifted.

`0x005e4370` now has a code-derived host-country/date/RNG schedule frontier
shape. It scans 34-byte records around current-date year offsets, resolves
pending status bytes, assigns unresolved slots with verified `match_random(3)`,
calls the host-country-attributed candidate helper `0x005e4840`, and emits
schedule/event records via `0x005e49e0`. The exact owning table and event record
semantics remain frontier-only until the surrounding data model is lifted.

`0x00595580` now has a code-derived fixture/news cleanup frontier shape. It
reads date state, builds short date offsets with `0x00536190`, walks news-like
records addressed as `DAT_00acd5bc + index * 0x245`, reattaches recent news via
`news.cpp` helpers `0x0076d7d0`/`0x0076e180`, filters human-manager-visible
records through `0x005ea590`, and can create paired dated fixture events through
`0x0050c8d0`. The exact fixture/news ownership and mutation semantics remain
frontier-only.

`0x00674c10` now has a code-derived manager-job lifecycle frontier shape. It
repairs missing jobs, expires dated manager-job entries, walks 0x245-byte
club/news records and 0x6e-byte manager records, clears/reassigns manager
relation status bytes, resets 0x49-byte per-club manager state blocks, uses
verified `match_random` for candidate and timing choices, and queues 0x26-byte
job/vacancy events. The appointment, contract, news, and manager-reputation
mutations remain frontier-only until their record layouts are fully lifted.

`0x0053fe40` is a code-derived current-date callback dispatcher. It walks 42
pointer slots and invokes each non-null object's vtable `+4` callback with the
current-date argument. The callback target types remain frontier-only.

`0x00614e90` is a code-derived staff role/competition drift frontier. It releases
scratch state, decrements per-entry counters, performs a day-180 refresh over
0x6e-byte staff records, samples staff through verified `match_random`, mutates
role/preference bytes, emits outcomes through `0x00616930`, and runs
`0x006176f0`/`0x006180c0` post-processing. The exact role/status byte semantics
remain frontier-only.

`0x005bfd90` is a code-derived season/calendar maintenance frontier. It
initializes a one-shot guard, scans 34 calendar buckets, compares bucket dates to
today and tomorrow, drives "Updating game data" progress, invokes bucket
callbacks, clears staff byte `+0x6d` for bucket 3, and rotates per-date scratch
state. The calendar bucket owners remain frontier-only.

`0x005c01d0` is a code-derived club rolling-metrics frontier. It walks 0x122-byte
club records, computes weighted double metrics from fields around `+0xb0` to
`+0xd8`, shifts rolling windows, resets current accumulators, and sorts 12-byte
ranking records. The exact metric meanings remain frontier-only.

`0x00449710` is a code-derived queued club-news dispatch cleanup. It drains
6-byte queued club/news items, resolves linked 0x245-byte club records, builds
news payloads, dispatches through human/non-human news helpers, frees the queue
when capacity exceeds 99, and resets the active count.

`0x00752d40` is a code-derived fixture/tie participant notification frontier. It
date-gates competition/tie lists, lazily builds scratch date state, walks two
fixture lists, filters fixture type codes, updates participant notification state
through `0x0075ee00`, prunes current-day entries, and runs a 70-day cleanup.

`0x00585ae0` is a code-derived club finance/stadium/status drift frontier. It
handles a special dated club/stadium reassignment path, walks 0x245-byte club
records with parallel 0x167-byte finance/status side blocks, uses verified
`match_random` for financial/status drift, updates linked records/news, and
applies season/date-gated club adjustments.

`0x00784290` is a code-derived byte-array clear frontier. It clears the byte
buffer at `param_1 + 8` for `param_1 + 4` entries. The owner of that buffer is
still unnamed.

The Rust runtime save now implements only the validated phase counter rollover,
packed date add-days, and a provenance trace (`RuntimePhaseTrace`) listing the
frontiers each phase would enter. Trace entries are audit data, not claims that
the listed subsystem mutations have been implemented.

The first headless runtime harness is also in place. It can create a
Rust-native save from `rust-db`, advance by days or to a target date without the
original `.dat` files, and records a `HeadlessRunReport` with days/phases
advanced, frontiers reached, milestones, and explicit blockers. Its current mode
is `VerifiedShell`. The save also records headless manager/session metadata and
command history, including optional selected club ids, but club-control effects
are frontier-only until original human-manager creation and control semantics are
lifted. This proves the headless delivery loop can run through the code-derived
phase/date shell, while match results, competition state, transfers/contracts,
and news/inbox mutations remain blocked until their exact CM0102 formulas and
record semantics are lifted.

`validate-runtime-simulation` checks that Rust's save runtime actually follows
that lifted slice: start date `2001-07-01` maps to CM day-of-year `182`, one day
executes phases `0 -> 1 -> 2 -> 0`, the final transition advances to
`2001-07-02`, and the phase-2 trace records the expected frontier list,
including the match-day builder, annotation helper, dispatcher, verified setup
boundary, team/player setup, player-risk setup frontier, and tactics block
loader. It also requires the deeper match tactical flag readers, player random
byte seeding frontier, player evaluation frontier, and action-score frontier to
be present in the trace, plus tactic index/slot lookups, formation mask
classifiers, the random-float jitter shim, and the match-player action selector,
movement/action resolver, action-attempt, candidate wrapper, adjacent-position
wrapper, event-resolution dispatcher, event queue writer, follow-up challenge,
directional follow-up, shot/action score, and match engine step/control
frontiers, plus the period transition, player candidate selector, per-tick
tactical updater, and scratch reset helpers. It also
checks target-date ticking: ticking to the current date is a no-op, while ticking
from `2001-07-01` to `2001-07-04` advances three days through nine traced CM
phase transitions and lands on CM day-of-year `185`. It also checks that a
two-day headless run advances six verified phases, records a run report, and
keeps unimplemented gameplay systems explicit as frontier-only blockers. It also
checks that a headless manager/session command is persisted with command history
without claiming club-control mutations.

`validate-rng` also proves the Rust MSVC `rand()` implementation against a known
answer, maps `match_random 0x008fc4f0`, maps the random table base
`0x00a8df38`, maps the initializer `0x008fc5d0`, and records the BSS-backed RNG
state globals.

The RNG initializer decompile (`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/008fc5d0.c`)
sets the start pointer as `base + rand() % 51000`, sets the inclusive wrap/end
pointer to `0x00abfc14`, and stores `seed16 = rand() % 0xffff`. Therefore the
canonical RNG table is `0x00a8df38..=0x00abfc14`: `204000` bytes, or `51000`
little-endian `i32` entries.

`extract-rng-table` writes the full provenance-stamped table by default. Supplying
an entry count writes a sample only, capped at the verified full length.

## Runtime Boundary

The Rust save/calendar code is infrastructure only. It proves that a modern app
can load `rust-db`, create a Rust-native save, persist it, and advance neutral
date state to either a day count or exact target date through the verified
three-phase frontier. It is not yet a claim that the phase subsystem mutations
inside the original day loop have been implemented.

The next accurate runtime work is to name the three `DAT_00acde88` phases,
classify each mutating subsystem call inside `0x005b6a90`, and only then replace
the placeholder Rust calendar tick with a CM-derived simulation step.
