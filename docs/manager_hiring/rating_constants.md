# Manager Hiring AI — RESOLVED x87 Float Constants (cm0102.exe, DirectDraw build)

**Status: DECODE BLOCKER CLEARED.** The real PE (`D:\cm0102\cm0102.exe`, ImageBase
0x00400000, VAs match `ghidra_out`) is now on disk, so every `.rdata` float the
Ghidra decompile dropped as bare `__ftol()` / `_DAT_0095xxxx` has been read
directly from the binary. This file supersedes the "UNKNOWN" columns in
`rating_primitives_spec.md` and the "float coefficient" blockers in
`manager_hiring_spec.md` §3.1/§5.3/§10.

Method: disassembled each target fn with capstone from `va2off(entry)` for
`functions.json` byte length; for every x87 instruction with an absolute memory
operand in `.rdata` (`>=0x00955000`) recorded {instr VA, mnemonic, operand VA,
access size, value}. `dword ptr` operands are IEEE-754 f32, `qword ptr` are f64.
Values verified against the named `_DAT_` roles in `rating_primitives_spec.md`
(all cross-checks pass — see §7). Helper `call 0x009346d0` = round-to-nearest-int
(the real body Ghidra rendered as `__ftol()`); `call 0x008fc4f0` = `GameRng::rand_mod`.

> NOTE on the f32 column in the tables: for a `qword ptr` (f64) site the f32
> reading of the first 4 bytes is meaningless (shown only for completeness in the
> raw dumps) — **use the size the instruction actually encodes.** Every scoring
> constant below is f64 except the board-confidence clamp floats at 0x0095ae6c–78
> and three literal-0/1 compares, which are genuinely f32.

---

## 1. Provenance / correction to earlier specs

`rating_primitives_spec.md` stated "No binary on disk … every `.rdata`
multiplier is UNKNOWN." That constraint is gone. Two earlier *guesses* are now
corrected by the disassembly:

* **`_DAT_00955888` is NOT an f32** (the spec speculated `float` at 0082dab0
  line 168). It is read as `qword ptr` everywhere → **f64 = 0.05**. Likewise
  `_DAT_00956968` = **0.5** (the line-168 addend).
* **Board confidence is NOT "2×rand(500)"** (manager_hiring_spec §5.3 step 4 was
  a placeholder). The real seed is a club-rep-ratio term scaled by 7500 then
  jittered — see §3.

Everything else in the two specs (integer skeletons, thresholds, RNG draw order,
field offsets) is confirmed intact.

---

## 2. MASTER CONSTANT DICTIONARY (every `_DAT_` resolved)

Distinct `.rdata` float constants referenced across the six target functions,
by operand VA. `refs` = number of instruction sites in the six fns. The `.rdata`
"float pool" is **non-contiguous** — these live interleaved with string/table
data between 0x00955878 and 0x0095d608, not as one packed array. `1/x` given
where the value is an exact reciprocal (these are `attribute/N` normalisers).
| operand VA (`_DAT_`) | size | value | 1/x | refs |
|---|---|---|---|---|
| 0x00955878 | f64 | 0.025 | 1/40 | 7 |
| 0x00955880 | f64 | 0.1 | 1/10 | 15 |
| 0x00955888 | f64 | 0.05 | 1/20 | 6 |
| 0x00955890 | f64 | 1.0 | 1/1 | 89 |
| 0x00955898 | f64 | 0.01 | 1/100 | 13 |
| 0x00956898 | f64 | 250.0 |  | 1 |
| 0x009568a0 | f64 | 5.0 |  | 2 |
| 0x009568a8 | f64 | 100.0 |  | 3 |
| 0x009568b0 | f64 | 10.0 |  | 13 |
| 0x009568e8 | f64 | 5000.0 |  | 3 |
| 0x009568f8 | f64 | 2000.0 |  | 2 |
| 0x00956908 | f64 | 1000.0 |  | 4 |
| 0x00956918 | f64 | 0.2 | 1/5 | 7 |
| 0x00956928 | f64 | 1.25 |  | 4 |
| 0x00956930 | f64 | 3.0 |  | 1 |
| 0x00956940 | f64 | 1.5 |  | 14 |
| 0x00956950 | f64 | 2.0 |  | 1 |
| 0x00956958 | f64 | 0.016666666666666666 | 1/60 | 2 |
| 0x00956968 | f64 | 0.5 | 1/2 | 15 |
| 0x00956970 | f32 | 0.0 |  | 2 |
| 0x00956978 | f64 | 1.2 |  | 3 |
| 0x009569a0 | f64 | 0.0 |  | 5 |
| 0x009569b0 | f64 | 0.8 |  | 5 |
| 0x009569c0 | f64 | 1.1 |  | 3 |
| 0x009569d0 | f64 | 0.95 |  | 4 |
| 0x009569d8 | f64 | 0.9 |  | 6 |
| 0x009569e0 | f64 | 0.25 | 1/4 | 6 |
| 0x00956a90 | f64 | 0.04 | 1/25 | 1 |
| 0x00956a98 | f64 | 0.02 | 1/50 | 9 |
| 0x00956d60 | f64 | 0.004 | 1/250 | 4 |
| 0x00956d78 | f64 | 6.666666666666667e-05 | 1/15000 | 2 |
| 0x00956da0 | f64 | 0.0006666666666666666 | 1/1500 | 1 |
| 0x00956dc0 | f64 | 1.4 |  | 2 |
| 0x00956e00 | f64 | 0.4 |  | 9 |
| 0x00956e18 | f64 | 2.5 |  | 1 |
| 0x00956e38 | f64 | 3.5 |  | 2 |
| 0x00956e40 | f64 | 0.7 |  | 2 |
| 0x00956e48 | f64 | 0.6 |  | 8 |
| 0x00956e68 | f32 | 1.0 | 1/1 | 1 |
| 0x00956e78 | f64 | 0.3 |  | 2 |
| 0x00956ee8 | f64 | 0.005 | 1/200 | 12 |
| 0x00956f40 | f64 | 150.0 |  | 2 |
| 0x00956f68 | f64 | 0.33 |  | 4 |
| 0x00956f70 | f64 | 0.67 |  | 2 |
| 0x00956f88 | f64 | 1.75 |  | 1 |
| 0x00957018 | f64 | 0.0025 | 1/400 | 3 |
| 0x00957030 | f64 | 0.75 |  | 17 |
| 0x00957038 | f64 | 0.002 | 1/500 | 21 |
| 0x00957500 | f64 | 0.35 |  | 1 |
| 0x00957510 | f64 | 0.85 |  | 4 |
| 0x00957518 | f64 | 0.15 |  | 1 |
| 0x00957580 | f64 | 1.35 |  | 1 |
| 0x00958348 | f64 | 0.001 | 1/1000 | 3 |
| 0x009585b0 | f64 | 0.0001 | 1/10000 | 2 |
| 0x009585c0 | f64 | 0.65 |  | 2 |
| 0x00958608 | f64 | 0.0008 | 1/1250 | 2 |
| 0x00958638 | f64 | 200.0 |  | 3 |
| 0x00958668 | f64 | 0.875 |  | 1 |
| 0x00958688 | f64 | 300.0 |  | 1 |
| 0x00958690 | f64 | 0.000125 | 1/8000 | 1 |
| 0x009586b0 | f64 | 0.45 |  | 1 |
| 0x00958f68 | f64 | 0.975 |  | 3 |
| 0x0095ae6c | f32 | 7500.0 |  | 1 |
| 0x0095ae70 | f32 | 500.0 |  | 2 |
| 0x0095ae74 | f32 | 6500.0 |  | 1 |
| 0x0095ae78 | f32 | 10000.0 |  | 1 |
| 0x0095ae98 | f64 | 0.006666666666666667 | 1/150 | 8 |
| 0x0095af00 | f64 | 5e-05 | 1/20000 | 2 |
| 0x0095af08 | f64 | 2.5e-05 | 1/40000 | 5 |
| 0x0095af10 | f32 | 0.75 |  | 1 |
| 0x0095af18 | f64 | 0.0013333333333333333 | 1/750 | 6 |
| 0x0095af20 | f64 | 0.0044444444444444444 | 1/225 | 2 |
| 0x0095af28 | f64 | 0.008 | 1/125 | 1 |
| 0x0095af30 | f64 | 0.0008333333333333334 | 1/1200 | 3 |
| 0x0095af38 | f64 | 0.475 |  | 1 |
| 0x0095b140 | f64 | 7000.0 |  | 4 |
| 0x0095b2b0 | f64 | 0.2857142857142857 |  | 4 |
| 0x0095b370 | f64 | 12.0 |  | 2 |
| 0x0095b3a8 | f64 | 0.225 |  | 1 |
| 0x0095b4a0 | f64 | 2.2 |  | 1 |
| 0x0095d5d8 | f64 | 0.00022222222222222223 | 1/4500 | 1 |
| 0x0095d5e0 | f64 | 0.08771929824561403 |  | 1 |
| 0x0095d5e8 | f64 | 12000.0 |  | 2 |
| 0x0095d5f0 | f64 | 1.45 |  | 2 |
| 0x0095d5f8 | f64 | 2.2222222222222223 |  | 1 |
| 0x0095d600 | f64 | 1.1764705882352942 |  | 1 |
| 0x0095d608 | f64 | 900.0 |  | 1 |

### 2.1 Named-role crosswalk (spec `_DAT_` label → resolved value)

| `_DAT_` label (from rating_primitives_spec) | resolved value | role in that spec |
|---|---|---|
| `_DAT_009569a0` | **0.0** | attractiveness "unattractive/ineligible" floor returned at `LAB_0082fb21` |
| `_DAT_00956e38` | **3.5** | neutral return when ref-club nation==0; also renegotiate numerator (line 907) |
| `_DAT_00955888` | **0.05** | line 168 manager-attribute (`person+0x57`) linear multiplier |
| `_DAT_00956968` | **0.5** | line 168 additive constant (also generic ×0.5) |
| `_DAT_0095d608` | **900.0** | vacancy age-term denominator addend `(clubRepHome + 900)` |
| `_DAT_0095b370` | **12.0** | vacancy age-term clamp ceiling |
| `_DAT_00956f68` | **0.33** | rivalry / league-identity multiplier |
| `_DAT_00956930` | **3.0** | `FUN_00531940(p,club)` factor (line 246) |
| `_DAT_00957510` | **0.85** | league/nation-identity factor |
| `_DAT_00956d78` | **6.667e-05** (=1/15000) | line 561 rep² divisor coefficient |
| `_DAT_0095d5e8` | **12000.0** | line 561 denominator base `(12000 − loyalty·200)` |
| `_DAT_00958638` | **200.0** | loyalty weight in that denominator |
| `_DAT_00955880` | **0.1** (=1/10) | loyalty-floor multiplier / generic ×0.1 |
| `_DAT_00956918` | **0.2** (=1/5) | line 561 additive `(dVar1 + 0.2)` |
| `_DAT_00957018` | **0.0025** (=1/400) | line 666 availability-term weight |
| `_DAT_00958690` | **0.000125** (=1/8000) | line 753 reputation-gap slope |
| `_DAT_00955890` | **1.0** | ubiquitous `±1.0` (builds `(1 ± a·w)` factors and clamps) |
| `_DAT_0095d5e0` | **0.0877192982…** (=5/57) | line 753 `person+0x57` slope in fVar24 |
| `_DAT_0095d5d8` | **0.000222…** (=1/4500) | line 753 final rep-gap slope |
| `_DAT_00956f88` | **1.75** | renegotiate trigger threshold (`fVar23 >= 1.75`) |
| `_DAT_00957030` | **0.75** | generic ×0.75 (very common) |
| `_DAT_00957038` | **0.002** (=1/500) | commonest per-attribute normaliser |

All 22 spot-checks agree with the spec's inferred roles → **recovery is
trustworthy**; the remaining constants in §2 slot into the same formulas.

---

## 3. FUN_00679ed0 — board-confidence init CENTRES (RESOLVED)

`call 0x008fc4f0` = `rand_mod(n)`; `call 0x009346d0` = round-to-int. `edi`→job
table base `[*this]`, `esi = clubId*0x49` (job row), `ebp` = club, `ebx` =
`person+0x69` (manager standing block during the seed phase).

**Manager-standing expectation seed** (0x00679f30–0x00679fd8, three sites all
×0.1): for standing shorts at `[ebx+0x08]`,`[+0x0a]`,`[+0x0c]` the exe blends
each with the club reputation `club+0x80`:
```
new = round( clubRep * 0.1 + standing_short )          // fild·fmul 0.1·fiadd
```
(`_DAT_00955880` = 0.1 confirmed; three consecutive applications, one per
standing axis. Matches spec §5.3 step 2.)

**jobrow+0x06 board confidence** (0x0067a02e–0x0067a119). Real formula
(replaces the "2×rand(500)" placeholder):
```
r     = rand_mod(1000) + 500                       // ∈ [500,1499]
denom = clubRep  if clubRep >= 500
        rand_mod(750)+250  if 0 < clubRep < 500     // ∈ [250,999]
        r        if clubRep == 0
centre = (r / denom) * 7500.0                       // f32 7500.0 @0x0095ae6c
centre = clamp(centre, 6500.0, 10000.0)             // f32 6500 @0x95ae74, 10000 @0x95ae78
jobrow[+0x06] = round( centre + rand_mod(500) − rand_mod(500) )
              // then hard clamp:
if < 0x186a(6250):  = 6250 + rand_mod(0x4e2=1250)
if > 0x251c(9500):  = 10000 − rand_mod(500)
```
**jobrow+0x08 chairman patience** (0x0067a167–0x0067a18c):
```
jobrow[+0x08] = round( jobrow[+0x06] − rand_mod(500) + rand_mod(500) )
if < 0x186a(6250): = 6250 + rand_mod(0x4e2=1250)
if > 0x2710(10000): = 10000
```
**jobrow+0x0a fans** (0x0067a1c8–0x0067a202):
```
jobrow[+0x0a] = round( jobrow[+0x06] − (rand_mod(500)+rand_mod(500)) )   // fadd then fsubr
if < 0x1482(5250): = 5250 + rand_mod(0xfa=250)
if > 0x2710(10000): = 10000
```
**jobrow+0x0c media/expectation** (0x0067a23e–0x0067a278): identical shape,
floor **0x1676(5750)** + `rand_mod(250)`, ceiling 10000.

Board-confidence CENTRES therefore are: base = `7500·r/denom` clamped
`[6500,10000]`, the other three rows derive from `+0x06` by `±rand(500)`
symmetric jitter with per-row floors 6250/6250/5250/5750. The five embedded
literals `0x459c4000=5000.0` (unused fallback), `0x461c4000=10000.0`,
`0x45cb2000=6500.0` are f32 immediates in the clamp path.

*(f32 reset value 5000.0 appears at 0x0067a01d as an initial load; the live
path uses the ratio term above.)*

---

## 4. FUN_00681c70 — poach jitter (RESOLVED)

When a poach linkage fires, the target/associated club's job-security
confidence rows are scaled down and rounded (`round(row * k)`). `edx/eax` index
the club's job row; offsets +6/+8/+0xa/+0xc = board/patience/fans/media.

| job row field | multiplier k | site |
|---|---|---|
| +0x06 board confidence | **0.9** | 0x00681d9b |
| +0x08 chairman patience | **0.75** | 0x00681dc8 |
| +0x0a fans | **0.9** | 0x00681df5 |
| +0x0c media | **0.9 then ×0.975** (two-step) | 0x00681e22, 0x00681e35 |
| second club +0x08 | **0.95** | 0x00681e62 |
| second club +0x0a | **0.975** | 0x00681e8f |
| second club +0x0c | **0.975** | 0x00681ebc |
```
row = round(row * k)          // k ∈ {0.9, 0.75, 0.95, 0.975}
```
`_DAT_009569d8`=0.9, `_DAT_00957030`=0.75, `_DAT_009569d0`=0.95,
`_DAT_00958f68`=0.975. No RNG in the jitter itself (the RNG gates are the
`rand_mod` acceptance checks in the enclosing decision, spec §3.2).

---

## 5. FUN_00682420 — score core: the `__ftol` sites RESOLVED

The two undecoded pieces were (a) the base re-weights (lines 88/92) and (b) the
per-jobtype "10-term" attribute blocks. Both are now pinned.

### 5.1 Base re-weight (lines 88 / 92)
`edi = person+0x69` standing block; the score accumulator `local_94` sits in
`[esp+0x14]`. Two symmetric gated rewrites:
```
// line 88 — gated by FUN_00531370 / FUN_005313b0(person, club):
if FUN_00531370(p,c):  L = max(L*1.1, L+2000.0)          // 1.1 @0x9569c0, 2000.0 @0x9568f8
elif FUN_005313b0(p,c): L = L*0.25                        // 0.25 @0x9569e0
L = round(L)
// line 92 — same shape, gated by FUN_005313f0 / FUN_00531420(person, otherclub):
if FUN_005313f0(p,o):  L = max(L*1.1, L+2000.0)
elif FUN_00531420(p,o): L = L*0.25
L = round(L)
```
(These are the "same-nationality / works-abroad" affinity re-weights the spec
flagged. `max()` is realised by the `fcom`+conditional at 0x00682600/0x00682660.)

### 5.2 Per-jobtype attribute blocks (the runs of `__ftol`)
Every discarded `__ftol` in the switch is one factor of a product applied to
`local_94`. The canonical shape (verified at 0x00682bc7 etc.):
```
a = (schar) standing_block[edi + off]        // off ∈ {0x0e,0x10,0x11,0x12,…}
L = round( L * (a * w + 1.0) )      // fadd 1.0  →  factor (1 + a·w)
L = round( L * (1.0 - a * w) )      // fsubr 1.0 →  factor (1 − a·w)
```
`w` is the per-attribute weight from the tables in §6, always a reciprocal
(1/500, 1/200, 1/100, 1/50, 1/150, 1/750, 1/1200, 1/225, …). A `fimul`/`fmul`
against the accumulator, then `round` (0x009346d0), commits each factor. The
per-case ordered weight list is the §6 table for FUN_00682420 read top-to-bottom
between the case boundaries below. Because each factor reads a **different
standing-record byte**, the operand-attribute pairing (which `edi+off` feeds
which weight) must be taken from the integer decompile lines listed — the
weights themselves are now all known.

Outer `switch(param_4 jobtype)` case boundaries (from decompile + jump table):
| jobtype (param_4) | case body VA range | decompile lines |
|---|---|---|
| 5 / 0xc (+ inner switch on preferred `local_8e`) | 0x006826da–0x006838xx | 95–232 |
| 6 / 7 / 0xd / 0xe | ~0x006831xx block | 234–306 (10+10 terms) |
| 8 / 0xf | " | 265–285 |
| 9 | " | 290–305 |
| 0xa | " | 309–316 |
| default | `goto` reject | 317 |

### 5.3 Complex sub-term at 0x006829e4–0x00682a36
One case computes a bounded ratio rather than a plain factor:
```
y = 1.25 − (standing_byte[edi+0x0e] * 0.025)      // 1.25 @0x956928, 0.025 @0x955878
// products y*100, y*150, y*300 built (100 @0x9568a8, 150 @0x956f40, 300 @0x958688)
term = round( (1.0 − someInt / (y*100|150|300)) * 1.5 )   // 1.0 @0x955890, 1.5 @0x956940
```
(Exact numerator selection follows the surrounding int ops; all constants known.)

### 5.4 Tail gates (lines 347–459) — final multipliers
Applied to `local_94`/return `iVar4` after the per-jobtype block:

| decompile line | guard | resolved op (nearest x87 site) |
|---|---|---|
| 347 | league-tier demotion (`club.rep>0x128e`, nation-tier fail, preferred 5/0xc) | `L = round(L*(tierGap*0.025 + 1.0))` @0x006838f8/fe |
| 365 | high-rep accept (`club.rep>=0x1c53`) | bonus via ×0.75/×0.35/×0.5 chain @0x00683904+ |
| 398/403 | national team (`FUN_00525450≠0`), extra at rep>0x186a | ×0.75 / ×0.35 / ×0.5 terms @0x006839be–0x00683a4c |
| 411 | rep-fit in `[0x109a,0x1a5e)` partial scaling | ×2.5e-05/×5e-05 + 0.5/0.75 @0x006839a8–0x00683c8c |
| 419 | national-team date-window tail | ×0.75/×0.05+0.5 @0x00683cc6–0x00683d47 |
| 423 | `FUN_007aeb90` language/adapt fit | final ×0.75 @0x00683d47 |
| 437/445/459 | `local_98[+0x7e]` class 2/other return scale | shares the same tail constants |

The dominant tail constants are 0.75, 0.35, 0.15, 0.5, 0.25, 0.05, 1.2, 2.5e-05,
5e-05 (see §6). No tail constant is unknown.

---

## 6. FULL ORDERED x87 CONSTANT TABLES (verbatim, per function)

For each function, every x87 instruction that touches a `.rdata` constant, in
address order: {instr VA, mnemonic, operand VA, access size, value, 1/x}. This
is the ground-truth stream to implement against. `fadd 1.0`/`fsubr 1.0` mark the
`(1 ± a·w)` factor boundaries; `fmul`/`fdiv`/`fadd`/`fsub`/`fsubr`/`fcom(p)` are
the arithmetic; the integer operand feeding each `fild`/`fimul` is a
person/club/standing record byte (not in `.rdata`) — read it from the decompile.

### FUN_00679ed0 board-confidence init  (0x00679ed0)

| instr VA | mnem | operand VA | size | value | 1/x |
|---|---|---|---|---|---|
| 0x00679f53 | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x00679f88 | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x00679fba | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x00679fe8 | fcomp | 0x00956970 | f32 | 0.0 |  |
| 0x0067a021 | fcomp | 0x0095ae70 | f32 | 500.0 |  |
| 0x0067a05b | fcom | 0x00956970 | f32 | 0.0 |  |
| 0x0067a070 | fcom | 0x0095ae70 | f32 | 500.0 |  |
| 0x0067a09f | fmul | 0x0095ae6c | f32 | 7500.0 |  |
| 0x0067a0af | fcomp | 0x0095ae78 | f32 | 10000.0 |  |
| 0x0067a0ca | fcomp | 0x0095ae74 | f32 | 6500.0 |  |

### FUN_00681c70 poach jitter  (0x00681c70)

| instr VA | mnem | operand VA | size | value | 1/x |
|---|---|---|---|---|---|
| 0x00681d9b | fmul | 0x009569d8 | f64 | 0.9 |  |
| 0x00681dc8 | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x00681df5 | fmul | 0x009569d8 | f64 | 0.9 |  |
| 0x00681e22 | fmul | 0x009569d8 | f64 | 0.9 |  |
| 0x00681e35 | fmul | 0x00958f68 | f64 | 0.975 |  |
| 0x00681e62 | fmul | 0x009569d0 | f64 | 0.95 |  |
| 0x00681e8f | fmul | 0x00958f68 | f64 | 0.975 |  |
| 0x00681ebc | fmul | 0x00958f68 | f64 | 0.975 |  |

### FUN_0082dab0 attractiveness  (0x0082dab0)

| instr VA | mnem | operand VA | size | value | 1/x |
|---|---|---|---|---|---|
| 0x0082dde1 | fmul | 0x00955878 | f64 | 0.025 | 1/40 |
| 0x0082ddeb | fsubr | 0x00956dc0 | f64 | 1.4 |  |
| 0x0082ddf5 | fmul | 0x00955878 | f64 | 0.025 | 1/40 |
| 0x0082de06 | fmul | 0x00956a98 | f64 | 0.02 | 1/50 |
| 0x0082de0c | fadd | 0x009569b0 | f64 | 0.8 |  |
| 0x0082de19 | fmul | 0x00955888 | f64 | 0.05 | 1/20 |
| 0x0082de1f | fadd | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082de3b | fadd | 0x00956e68 | f32 | 1.0 | 1/1 |
| 0x0082de48 | fcomp | 0x009569a0 | f64 | 0.0 |  |
| 0x0082dea2 | fld | 0x00956e38 | f64 | 3.5 |  |
| 0x0082e02b | fadd | 0x0095d608 | f64 | 900.0 |  |
| 0x0082e033 | fld | 0x0095b370 | f64 | 12.0 |  |
| 0x0082e044 | fld | 0x0095b370 | f64 | 12.0 |  |
| 0x0082e07c | fmul | 0x00956f68 | f64 | 0.33 |  |
| 0x0082e0df | fmul | 0x00957510 | f64 | 0.85 |  |
| 0x0082e0eb | fmul | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082e132 | fmul | 0x00956930 | f64 | 3.0 |  |
| 0x0082e1c7 | fmul | 0x00956958 | f64 | 0.016666666666666666 | 1/60 |
| 0x0082e1cd | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082e225 | fmul | 0x00955878 | f64 | 0.025 | 1/40 |
| 0x0082e22b | fsubr | 0x00957030 | f64 | 0.75 |  |
| 0x0082e242 | fmul | 0x00956958 | f64 | 0.016666666666666666 | 1/60 |
| 0x0082e248 | fsubr | 0x009569b0 | f64 | 0.8 |  |
| 0x0082e25f | fmul | 0x00955878 | f64 | 0.025 | 1/40 |
| 0x0082e265 | fsubr | 0x00957510 | f64 | 0.85 |  |
| 0x0082e27c | fmul | 0x00955878 | f64 | 0.025 | 1/40 |
| 0x0082e282 | fsubr | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082e29c | fcomp | 0x00956e40 | f64 | 0.7 |  |
| 0x0082e2e3 | fmul | 0x00955888 | f64 | 0.05 | 1/20 |
| 0x0082e359 | fmul | 0x00956f68 | f64 | 0.33 |  |
| 0x0082e3bd | fmul | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082e42c | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082e486 | fmul | 0x00957018 | f64 | 0.0025 | 1/400 |
| 0x0082e48c | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082e4a4 | fmul | 0x00957018 | f64 | 0.0025 | 1/400 |
| 0x0082e4aa | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082e4c4 | fcom | 0x00956940 | f64 | 1.5 |  |
| 0x0082e4dc | fcomp | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082e51a | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x0082e529 | fmul | 0x009586b0 | f64 | 0.45 |  |
| 0x0082e543 | fmul | 0x009585c0 | f64 | 0.65 |  |
| 0x0082e563 | fdivr | 0x009568b0 | f64 | 10.0 |  |
| 0x0082e57c | fdivr | 0x009568b0 | f64 | 10.0 |  |
| 0x0082e599 | fmul | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082e5a4 | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x0082e5d7 | fmul | 0x00955888 | f64 | 0.05 | 1/20 |
| 0x0082e5dd | fadd | 0x009569b0 | f64 | 0.8 |  |
| 0x0082e5f3 | fmul | 0x0095b4a0 | f64 | 2.2 |  |
| 0x0082e637 | fmul | 0x009568b0 | f64 | 10.0 |  |
| 0x0082e661 | fmul | 0x009568b0 | f64 | 10.0 |  |
| 0x0082e66c | fmul | 0x0095d600 | f64 | 1.1764705882352942 |  |
| 0x0082e677 | fmul | 0x0095d5f8 | f64 | 2.2222222222222223 |  |
| 0x0082e68f | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x0082e697 | fmul | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082e6ad | fmul | 0x00956e00 | f64 | 0.4 |  |
| 0x0082e6bf | fmul | 0x009569e0 | f64 | 0.25 | 1/4 |
| 0x0082e6c7 | fmul | 0x00956e00 | f64 | 0.4 |  |
| 0x0082e6df | fmul | 0x009569e0 | f64 | 0.25 | 1/4 |
| 0x0082e6e7 | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082e6ff | fmul | 0x00956e00 | f64 | 0.4 |  |
| 0x0082e762 | fmul | 0x00956dc0 | f64 | 1.4 |  |
| 0x0082e7da | fmul | 0x0095b2b0 | f64 | 0.2857142857142857 |  |
| 0x0082e822 | fmul | 0x0095b2b0 | f64 | 0.2857142857142857 |  |
| 0x0082e8f6 | fmul | 0x009569b0 | f64 | 0.8 |  |
| 0x0082e8fc | fcom | 0x0095b140 | f64 | 7000.0 |  |
| 0x0082e90b | fld | 0x0095b140 | f64 | 7000.0 |  |
| 0x0082e98e | fmul | 0x00956f70 | f64 | 0.67 |  |
| 0x0082e99c | fmul | 0x00956f68 | f64 | 0.33 |  |
| 0x0082e9bd | fmul | 0x00958608 | f64 | 0.0008 | 1/1250 |
| 0x0082e9c7 | fsubr | 0x009568a0 | f64 | 5.0 |  |
| 0x0082e9db | fmul | 0x00956908 | f64 | 1000.0 |  |
| 0x0082ea0a | fmul | 0x009585b0 | f64 | 0.0001 | 1/10000 |
| 0x0082ea14 | fadd | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082ea28 | fmul | 0x00956908 | f64 | 1000.0 |  |
| 0x0082ea9e | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x0082eac4 | fmul | 0x00956928 | f64 | 1.25 |  |
| 0x0082eaea | fmul | 0x00957510 | f64 | 0.85 |  |
| 0x0082eafd | fmul | 0x00956e40 | f64 | 0.7 |  |
| 0x0082eb2f | fmul | 0x0095d5f0 | f64 | 1.45 |  |
| 0x0082eb5d | fmul | 0x00956e00 | f64 | 0.4 |  |
| 0x0082eb67 | fmul | 0x00956e48 | f64 | 0.6 |  |
| 0x0082ebfe | fmul | 0x0095d5f0 | f64 | 1.45 |  |
| 0x0082ec74 | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x0082ec7a | fsub | 0x009568e8 | f64 | 5000.0 |  |
| 0x0082ecb1 | fmul | 0x00956d78 | f64 | 6.666666666666667e-05 | 1/15000 |
| 0x0082ece1 | fld | 0x009568b0 | f64 | 10.0 |  |
| 0x0082ece9 | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082ecf3 | fmul | 0x00958638 | f64 | 200.0 |  |
| 0x0082ecf9 | fsubr | 0x0095d5e8 | f64 | 12000.0 |  |
| 0x0082ed16 | fadd | 0x00956918 | f64 | 0.2 | 1/5 |
| 0x0082ed33 | fcomp | 0x00956e48 | f64 | 0.6 |  |
| 0x0082edd4 | fmul | 0x00957510 | f64 | 0.85 |  |
| 0x0082ede0 | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x0082edec | fmul | 0x009585c0 | f64 | 0.65 |  |
| 0x0082edf8 | fmul | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082ee04 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x0082ee23 | fmul | 0x00956e78 | f64 | 0.3 |  |
| 0x0082ee2b | fmul | 0x009569e0 | f64 | 0.25 | 1/4 |
| 0x0082ee33 | fmul | 0x0095b3a8 | f64 | 0.225 |  |
| 0x0082ee3b | fmul | 0x00956918 | f64 | 0.2 | 1/5 |
| 0x0082ee43 | fmul | 0x00958348 | f64 | 0.001 | 1/1000 |
| 0x0082ee51 | fmul | 0x00956e18 | f64 | 2.5 |  |
| 0x0082eefa | fmul | 0x00957018 | f64 | 0.0025 | 1/400 |
| 0x0082ef4d | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x0082ef53 | fsub | 0x009568e8 | f64 | 5000.0 |  |
| 0x0082efc6 | fmul | 0x0095b2b0 | f64 | 0.2857142857142857 |  |
| 0x0082f00d | fmul | 0x0095b2b0 | f64 | 0.2857142857142857 |  |
| 0x0082f0ef | fmul | 0x009569b0 | f64 | 0.8 |  |
| 0x0082f0f5 | fcom | 0x0095b140 | f64 | 7000.0 |  |
| 0x0082f104 | fld | 0x0095b140 | f64 | 7000.0 |  |
| 0x0082f12a | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x0082f149 | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x0082f192 | fmul | 0x00956f70 | f64 | 0.67 |  |
| 0x0082f1a0 | fmul | 0x00956f68 | f64 | 0.33 |  |
| 0x0082f1c1 | fmul | 0x00958608 | f64 | 0.0008 | 1/1250 |
| 0x0082f1cb | fsubr | 0x009568a0 | f64 | 5.0 |  |
| 0x0082f1df | fmul | 0x00956908 | f64 | 1000.0 |  |
| 0x0082f20e | fmul | 0x009585b0 | f64 | 0.0001 | 1/10000 |
| 0x0082f218 | fadd | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x0082f22c | fmul | 0x00956908 | f64 | 1000.0 |  |
| 0x0082f29f | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x0082f2c5 | fmul | 0x00956928 | f64 | 1.25 |  |
| 0x0082f2eb | fmul | 0x009569d8 | f64 | 0.9 |  |
| 0x0082f2fe | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x0082f330 | fmul | 0x00957580 | f64 | 1.35 |  |
| 0x0082f35e | fmul | 0x00956e00 | f64 | 0.4 |  |
| 0x0082f368 | fmul | 0x00956e48 | f64 | 0.6 |  |
| 0x0082f441 | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x0082f447 | fsub | 0x009568e8 | f64 | 5000.0 |  |
| 0x0082f48b | fmul | 0x00956d78 | f64 | 6.666666666666667e-05 | 1/15000 |
| 0x0082f4bb | fld | 0x009568b0 | f64 | 10.0 |  |
| 0x0082f4c3 | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082f4cd | fmul | 0x00958638 | f64 | 200.0 |  |
| 0x0082f4d3 | fsubr | 0x0095d5e8 | f64 | 12000.0 |  |
| 0x0082f4f0 | fadd | 0x00956918 | f64 | 0.2 | 1/5 |
| 0x0082f529 | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082f56b | fmul | 0x00958690 | f64 | 0.000125 | 1/8000 |
| 0x0082f571 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082f59c | fmul | 0x0095d5e0 | f64 | 0.08771929824561403 |  |
| 0x0082f5a2 | fadd | 0x00957030 | f64 | 0.75 |  |
| 0x0082f5ca | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x0082f5d2 | fcom | 0x009568b0 | f64 | 10.0 |  |
| 0x0082f5e1 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x0082f5e7 | fsubr | 0x009569c0 | f64 | 1.1 |  |
| 0x0082f5ef | fcomp | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082f5fe | fld | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082f608 | fcomp | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082f617 | fld | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082f63f | fcom | 0x009569a0 | f64 | 0.0 |  |
| 0x0082f650 | fcomp | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082f65f | fmul | 0x00956a98 | f64 | 0.02 | 1/50 |
| 0x0082f665 | fld | 0x009568a8 | f64 | 100.0 |  |
| 0x0082f676 | fld | 0x009568a8 | f64 | 100.0 |  |
| 0x0082f67c | fld | 0x00956f40 | f64 | 150.0 |  |
| 0x0082f68f | fmul | 0x0095ae98 | f64 | 0.006666666666666667 | 1/150 |
| 0x0082f69d | fcomp | 0x00956e48 | f64 | 0.6 |  |
| 0x0082f6d0 | fcom | 0x009569a0 | f64 | 0.0 |  |
| 0x0082f6ee | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082f6f4 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082f6fa | fcom | 0x00956918 | f64 | 0.2 | 1/5 |
| 0x0082f709 | fld | 0x00956918 | f64 | 0.2 | 1/5 |
| 0x0082f715 | fmul | 0x00956a90 | f64 | 0.04 | 1/25 |
| 0x0082f71b | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082f721 | fcom | 0x00956e00 | f64 | 0.4 |  |
| 0x0082f730 | fld | 0x00956e00 | f64 | 0.4 |  |
| 0x0082f73d | fmul | 0x0095d5d8 | f64 | 0.00022222222222222223 | 1/4500 |
| 0x0082f743 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082f74d | fcomp | 0x00958348 | f64 | 0.001 | 1/1000 |
| 0x0082f77a | fcomp | 0x00956e48 | f64 | 0.6 |  |
| 0x0082f78b | fcomp | 0x00956978 | f64 | 1.2 |  |
| 0x0082f7a8 | fcom | 0x00956928 | f64 | 1.25 |  |
| 0x0082f7e4 | fcomp | 0x00956940 | f64 | 1.5 |  |
| 0x0082f7f5 | fcomp | 0x00956978 | f64 | 1.2 |  |
| 0x0082f899 | fld | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x0082f8aa | fld | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x0082f8b2 | fld | 0x009568b0 | f64 | 10.0 |  |
| 0x0082f8c3 | fld | 0x009568b0 | f64 | 10.0 |  |
| 0x0082f8dd | fcomp | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082f8ea | fmul | 0x00956918 | f64 | 0.2 | 1/5 |
| 0x0082f921 | fmul | 0x00956e00 | f64 | 0.4 |  |
| 0x0082f945 | fcomp | 0x00956e48 | f64 | 0.6 |  |
| 0x0082f969 | fcom | 0x00956f88 | f64 | 1.75 |  |
| 0x0082f9c4 | fld | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082f9d5 | fld | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082f9e1 | fld | 0x00956e38 | f64 | 3.5 |  |
| 0x0082f9f3 | fcomp | 0x009569d8 | f64 | 0.9 |  |
| 0x0082fa46 | fcomp | 0x009569a0 | f64 | 0.0 |  |
| 0x0082fa7c | fcom | 0x009568b0 | f64 | 10.0 |  |
| 0x0082fa8b | fld | 0x009568b0 | f64 | 10.0 |  |
| 0x0082fa99 | fld | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x0082faa8 | fld | 0x009568b0 | f64 | 10.0 |  |
| 0x0082fab9 | fld | 0x009568b0 | f64 | 10.0 |  |
| 0x0082fadc | fcom | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0082fb21 | fld | 0x009569a0 | f64 | 0.0 |  |

### FUN_00682420 score core  (0x00682420)

| instr VA | mnem | operand VA | size | value | 1/x |
|---|---|---|---|---|---|
| 0x006825ec | fmul | 0x009569c0 | f64 | 1.1 |  |
| 0x006825f6 | fadd | 0x009568f8 | f64 | 2000.0 |  |
| 0x00682625 | fmul | 0x009569e0 | f64 | 0.25 | 1/4 |
| 0x0068264c | fmul | 0x009569c0 | f64 | 1.1 |  |
| 0x00682656 | fadd | 0x009568f8 | f64 | 2000.0 |  |
| 0x00682685 | fmul | 0x009569e0 | f64 | 0.25 | 1/4 |
| 0x006826e5 | fmul | 0x009569d0 | f64 | 0.95 |  |
| 0x006826ff | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x00682734 | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x00682758 | fmul | 0x00956e48 | f64 | 0.6 |  |
| 0x00682789 | fmul | 0x00956e78 | f64 | 0.3 |  |
| 0x006827b2 | fmul | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x006827d6 | fmul | 0x00956e00 | f64 | 0.4 |  |
| 0x006827fe | fmul | 0x0095af38 | f64 | 0.475 |  |
| 0x00682824 | fmul | 0x00956e48 | f64 | 0.6 |  |
| 0x00682850 | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x00682890 | fmul | 0x00956918 | f64 | 0.2 | 1/5 |
| 0x006828b0 | fmul | 0x00958348 | f64 | 0.001 | 1/1000 |
| 0x006828f1 | fmul | 0x00958668 | f64 | 0.875 |  |
| 0x00682908 | fmul | 0x009569d8 | f64 | 0.9 |  |
| 0x00682910 | fmul | 0x009569d0 | f64 | 0.95 |  |
| 0x00682934 | fmul | 0x00955888 | f64 | 0.05 | 1/20 |
| 0x0068293a | fsub | 0x009569d0 | f64 | 0.95 |  |
| 0x006829a1 | fld | 0x00956950 | f64 | 2.0 |  |
| 0x006829b0 | fmul | 0x00955880 | f64 | 0.1 | 1/10 |
| 0x006829ec | fmul | 0x00955878 | f64 | 0.025 | 1/40 |
| 0x006829f8 | fsubr | 0x00956928 | f64 | 1.25 |  |
| 0x00682a00 | fmul | 0x009568a8 | f64 | 100.0 |  |
| 0x00682a08 | fmul | 0x00956f40 | f64 | 150.0 |  |
| 0x00682a14 | fmul | 0x00958688 | f64 | 300.0 |  |
| 0x00682a26 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682a30 | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x00682a68 | fmul | 0x00956898 | f64 | 250.0 |  |
| 0x00682a70 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682a9b | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682aca | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682af7 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682b24 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682b51 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682b7c | fmul | 0x00958638 | f64 | 200.0 |  |
| 0x00682b86 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682bb3 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682bd3 | fmul | 0x0095af30 | f64 | 0.0008333333333333334 | 1/1200 |
| 0x00682bd9 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682bfc | fmul | 0x0095af30 | f64 | 0.0008333333333333334 | 1/1200 |
| 0x00682c02 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682c23 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00682c29 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682c4a | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00682c50 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682c71 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00682c77 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682c98 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00682c9e | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682cc8 | fmul | 0x0095af28 | f64 | 0.008 | 1/125 |
| 0x00682ce2 | fmul | 0x00956d60 | f64 | 0.004 | 1/250 |
| 0x00682cf6 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00682cfc | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682d1e | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x00682d24 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682d4a | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x00682d50 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682d74 | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x00682d7a | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682d9e | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x00682dd2 | fmul | 0x0095ae98 | f64 | 0.006666666666666667 | 1/150 |
| 0x00682dd8 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682de2 | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x00682e18 | fmul | 0x0095ae98 | f64 | 0.006666666666666667 | 1/150 |
| 0x00682e1e | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682e47 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x00682e4d | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682e76 | fmul | 0x0095af20 | f64 | 0.0044444444444444444 | 1/225 |
| 0x00682e7c | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682ea5 | fmul | 0x0095ae98 | f64 | 0.006666666666666667 | 1/150 |
| 0x00682eab | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682ed4 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x00682eda | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682f03 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x00682f09 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682f32 | fmul | 0x0095ae98 | f64 | 0.006666666666666667 | 1/150 |
| 0x00682f38 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682f61 | fmul | 0x00956d60 | f64 | 0.004 | 1/250 |
| 0x00682f67 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682f87 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00682f8d | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682fb0 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00682fb6 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682fd7 | fmul | 0x0095af18 | f64 | 0.0013333333333333333 | 1/750 |
| 0x00682fdd | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00682ffe | fmul | 0x0095af18 | f64 | 0.0013333333333333333 | 1/750 |
| 0x00683004 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683025 | fmul | 0x00956da0 | f64 | 0.0006666666666666666 | 1/1500 |
| 0x0068302b | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068304c | fmul | 0x0095af18 | f64 | 0.0013333333333333333 | 1/750 |
| 0x00683052 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683073 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00683079 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068309d | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x006830a3 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006830c7 | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x006830cd | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006830f1 | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x006830f7 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068311b | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x0068314f | fmul | 0x0095ae98 | f64 | 0.006666666666666667 | 1/150 |
| 0x00683155 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068315f | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x00683195 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x0068319b | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006831c4 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x006831ca | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006831f3 | fmul | 0x0095af20 | f64 | 0.0044444444444444444 | 1/225 |
| 0x006831f9 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683222 | fmul | 0x0095ae98 | f64 | 0.006666666666666667 | 1/150 |
| 0x00683228 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683251 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x00683257 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683280 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x00683286 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006832af | fmul | 0x0095ae98 | f64 | 0.006666666666666667 | 1/150 |
| 0x006832b5 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006832de | fmul | 0x00956d60 | f64 | 0.004 | 1/250 |
| 0x006832e4 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683304 | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x0068330a | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068332d | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x00683333 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683354 | fmul | 0x0095af18 | f64 | 0.0013333333333333333 | 1/750 |
| 0x0068335a | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068337b | fmul | 0x0095af18 | f64 | 0.0013333333333333333 | 1/750 |
| 0x00683381 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006833a2 | fmul | 0x0095af18 | f64 | 0.0013333333333333333 | 1/750 |
| 0x006833a8 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006833c9 | fmul | 0x0095af30 | f64 | 0.0008333333333333334 | 1/1200 |
| 0x006833cf | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006833f0 | fmul | 0x00956d60 | f64 | 0.004 | 1/250 |
| 0x006833f6 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068341a | fmul | 0x00956a98 | f64 | 0.02 | 1/50 |
| 0x00683420 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683444 | fmul | 0x00956a98 | f64 | 0.02 | 1/50 |
| 0x0068344a | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068346e | fmul | 0x00956a98 | f64 | 0.02 | 1/50 |
| 0x00683474 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006834c6 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x006834cc | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006834d6 | fmul | 0x00956940 | f64 | 1.5 |  |
| 0x0068350c | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00683512 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068353b | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00683541 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068356a | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00683570 | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683599 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x0068359f | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006835c8 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x006835ce | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006835f7 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x006835fd | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683626 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x0068362c | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683655 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x0068365b | fsubr | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068367b | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x00683681 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006836a4 | fmul | 0x00956ee8 | f64 | 0.005 | 1/200 |
| 0x006836aa | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006836ce | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x006836d4 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006836f8 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x006836fe | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683722 | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00683728 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x0068374c | fmul | 0x00957038 | f64 | 0.002 | 1/500 |
| 0x00683766 | fmul | 0x00955898 | f64 | 0.01 | 1/100 |
| 0x0068376c | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683792 | fmul | 0x00956a98 | f64 | 0.02 | 1/50 |
| 0x00683798 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006837bc | fmul | 0x00956a98 | f64 | 0.02 | 1/50 |
| 0x006837c2 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006837e6 | fmul | 0x00956a98 | f64 | 0.02 | 1/50 |
| 0x006837ec | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683810 | fmul | 0x00956a98 | f64 | 0.02 | 1/50 |
| 0x00683816 | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x006838f8 | fmul | 0x00955878 | f64 | 0.025 | 1/40 |
| 0x006838fe | fadd | 0x00955890 | f64 | 1.0 | 1/1 |
| 0x00683904 | fcom | 0x00957030 | f64 | 0.75 |  |
| 0x00683913 | fld | 0x0095af10 | f32 | 0.75 |  |
| 0x00683968 | fmul | 0x00956978 | f64 | 1.2 |  |
| 0x006839a8 | fmul | 0x0095af08 | f64 | 2.5e-05 | 1/40000 |
| 0x006839ae | fsubr | 0x00957030 | f64 | 0.75 |  |
| 0x006839be | fmul | 0x00957500 | f64 | 0.35 |  |
| 0x006839f9 | fmul | 0x0095af08 | f64 | 2.5e-05 | 1/40000 |
| 0x006839ff | fadd | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x00683a29 | fmul | 0x0095af00 | f64 | 5e-05 | 1/20000 |
| 0x00683a2f | fadd | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x00683a46 | fmul | 0x0095af08 | f64 | 2.5e-05 | 1/40000 |
| 0x00683a4c | fadd | 0x00957030 | f64 | 0.75 |  |
| 0x00683a9f | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x00683b0b | fmul | 0x00957030 | f64 | 0.75 |  |
| 0x00683b80 | fmul | 0x00957518 | f64 | 0.15 |  |
| 0x00683c11 | fmul | 0x0095af08 | f64 | 2.5e-05 | 1/40000 |
| 0x00683c4c | fmul | 0x0095af08 | f64 | 2.5e-05 | 1/40000 |
| 0x00683c52 | fadd | 0x00957030 | f64 | 0.75 |  |
| 0x00683c86 | fmul | 0x0095af00 | f64 | 5e-05 | 1/20000 |
| 0x00683c8c | fadd | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x00683cc6 | fmul | 0x00955888 | f64 | 0.05 | 1/20 |
| 0x00683ccc | fadd | 0x00956968 | f64 | 0.5 | 1/2 |
| 0x00683cfa | fmul | 0x00955888 | f64 | 0.05 | 1/20 |
| 0x00683d22 | fmul | 0x009569e0 | f64 | 0.25 | 1/4 |
| 0x00683d47 | fmul | 0x00957030 | f64 | 0.75 |  |

---

## 7. FUN_0052a330 / FUN_0052a410 — CONFIRMED integer, no x87

Disassembly of both (224 B / 184 B) contains **zero** x87 instructions with a
`.rdata` operand — no `fld`/`fmul`/`fadd`/`__ftol` at all. They are pure
integer: short loads, `movsx`, integer `/2` (arithmetic shift), compares,
and the `FUN_005274d0` calls. This confirms `rating_primitives_spec.md` §1:
`manager_club_repfit` returns a `short` selected/averaged from the person's
reputation vector with **no floating point**. Port as integer verbatim; the
only "averaging" is C integer `x/2 + y/2` on shorts. **No constants to recover.**

---

## 8. Remaining ambiguities (flagged for the implementer)

1. **Attribute↔weight pairing in the score-core blocks.** All *weights* (§6
   table for 0x00682420) are recovered, but which standing-record byte
   (`edi+off`) each weight multiplies must be read from the integer operands in
   `00682420.c` (the `movsx …, byte ptr [edi + off]` immediately preceding each
   `fild`). The `.rdata` recovery does not disambiguate the offsets — they are
   record fields, not constants. Same applies to the attractiveness switch
   blocks (0082dab0 §6 table).
2. **`fcom`/`fcomp` sites are comparisons, not weights** (e.g. score-core
   0x00683904 `fcom 0.75`, attractiveness 0x0082e4c4 `fcom 1.5`). They select a
   branch; the listed value is the threshold being compared against, which is
   still exact and needed.
3. **Board-confidence `denom` selection** (§3): the three-way pick
   (clubRep / rand(750)+250 / r) is reconstructed from the `fcom 0.0` /
   `fcom 500.0` branch tests at 0x0067a05b/0x0067a070 — verified, but exercise
   it against a live seed if a golden trace is available.
4. **Two f32 "0.0"/"1.0" literals** at 0x00956970 (f32 0.0) and 0x00956e68
   (f32 1.0) are compare/accumulate literals, not scoring weights.
5. Attractiveness (0082dab0) base seeds 1.5 / 2.25 are x86 **immediates**
   (`0x3ff8…`, `0x4002…`), not `.rdata` — unchanged from the spec; 3.5 is the
   `.rdata` double at 0x00956e38 (used for the current-club seed AND the
   neutral/renegotiate paths).

**Bottom line:** every previously-UNKNOWN `.rdata` float in the six functions is
now a concrete IEEE-754 value (§2, §6). The scoring cores can be implemented
byte-exact by pairing these weights with the integer skeletons already in
`manager_hiring_spec.md` §3.1 and `rating_primitives_spec.md` §2–3.
