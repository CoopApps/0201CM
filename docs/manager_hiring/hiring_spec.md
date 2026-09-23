# Manager Hiring AI — Porting Spec (`manager_manager.cpp`)

Target: `D:\cm0102-rs`, crate `cm-domain`. This spec is byte-precise where the
decompile permits and flags every place where Ghidra lost the x87 FPU math so
those are decoded from disassembly before implementation rather than guessed.

Decompile roots:
* Core fns: `D:\cm0102-carve\ghidra_out\cm0102.exe\decompiled\<addr8>.c`
* Dispatcher: `D:\cm0102-carve\decompiled\segment_01_006cbd50_callers\0x005b85b0.c`

---

## 0. Executive correction to the task premise

The daily AI dispatcher **`0x005b85b0`** (game.cpp "step 10", 30 333 bytes) does
**NOT** call any manager_manager function. A full symbol scan of its decompile
(`grep FUN_006*` over `0x005b85b0.c`) returns only `FUN_00615*` / `FUN_00618450`
/ `FUN_006ce0e0` / `FUN_00672270/90` — that dispatcher is the per-club *player
morale / board-mood / transfer-interest* pass. It touches the same job-security
table (see §2.3) but does not run hiring.

The manager-hiring subsystem is driven by **`FUN_00674c10`** (manager_manager
daily/weekly tick). Reverse call graph (verified by `grep -rl`):

```
FUN_008120d0 (game init)      FUN_005b6920 (game step)
      │                              │
FUN_005b6a90 ──┐              FUN_005b6f10 ──┐
               ▼                              ▼
            FUN_00674c10  (manager_manager::tick  — THE driver)   [005b6a90.c:133]
```

In the Rust tick this attaches to the already-stubbed hook
`hook_manager_job_lifecycle` (game.cpp step 10i), lib.rs:22405-22407, whose own
doc comment already names `FUN_00674c10`. That is the wiring point (see §7).

---

## 1. Control flow (who calls whom, with addresses)

```
FUN_00674c10  manager_manager::tick                              [lifecycle A / driver]
 ├─ FUN_00536b90  today() ; compares vs DAT_00b4d5b4 (last-run day)   674c10.c:63-86
 ├─ (day changed) per-club sweep DAT_00acd5bc stride 0x245:
 │    └─ FUN_00689eb0(club)   per-club daily housekeeping             674c10.c:74
 │    └─ FUN_006822d0()       rebuild-missing-jobs sweep              674c10.c:80  → §1a
 ├─ per-person "recent-form decay" sweep (DAT_00acd5c4 stride 0x6e)   674c10.c:87-98
 ├─ (DAT_00acde90 % 7 == 0)  FUN_0067ce90()  weekly news predicate    674c10.c:99-101
 ├─ FUN_006808a0(this)        vacancy-event queue drain               674c10.c:102 → §1b
 └─ event-queue loop  FUN_00672400()  (pops queued job events)        674c10.c:104-543
      For each popped event, resolve person `piVar1` (DAT_00acd5c4+id*0x6e)
      and club `piVar14` (DAT_00acd5bc+id*0x245), validate, then APPLY the
      appointment/vacate transition (job-status switch, §5):
        ├─ FUN_00684fc0(person,club)   build candidate-reaction news   674c10.c:135 → §1c
        ├─ FUN_00675ae0(person,club)   UNLINK person from old role     674c10.c:270… → §5.1
        ├─ FUN_00675980(person,club)   unlink (nation-job variant)     674c10.c:306…
        ├─ FUN_00688070(person,club,club2,mode)  fire appointment news 674c10.c:272…
        ├─ FUN_004d6f80(person)        refresh person→type10 caches    674c10.c:273…
        ├─ (on hire) club+0xcf = person ; person+0x39/0x24 = club      674c10.c:477-525 → §5.2
        └─ FUN_00679ed0(club)          init board confidence           674c10.c:532 → §5.3

FUN_006822d0  fix_missing_job sweep                                  [006822d0.c]
 └─ for each national-team club with a would-be manager but no job record:
      FUN_00680cc0(club, 0)   → run a hiring round for it             006822d0.c:45

FUN_006808a0  process vacancy-event queue                            [006808a0.c]
 └─ for each queued vacancy event (FUN_00672400 pops):
      iVar3 = club->manager (club+0xcf) ; iVar1 = club record
      ├─ FUN_006809e0(club, evt.byte0, evt.byte1, 0)  apply resignation/sack  6808a0.c:37
      ├─ FUN_0052e370(club) != 0  (pickable gate)                     6808a0.c:39
      └─ FUN_00680cc0(club, club->manager)   RUN A HIRING ROUND       6808a0.c:41 → §1d

FUN_00680cc0  hire_for_club  (THE candidate-selection round)          [00680cc0.c]
 │  param_1 = manager_manager `this` (holds job-table base at [*param_1])
 │  param_2 = club to fill ;  param_3 = outgoing manager (or 0)
 ├─ early-out gates (club rep / board-cash), §3.4                     680cc0.c:45-62
 ├─ reset this club's job-security table row to defaults              680cc0.c:63-99
 ├─ PASS 1 — pick best realistic candidate:
 │    for each person in DAT_00acd5c4 (stride 0x6e):
 │      FUN_004d5b00(p)==0 && FUN_00684ed0(p)=='\0' (not already queued)   680cc0.c:119-120
 │      FUN_00684a80(p, club, 0, 3.5=0x400c0000)  would-accept?           680cc0.c:128 → §3.3
 │      score = FUN_00682420(p, club, 0, 5)  ; keep argmax(local_40)      680cc0.c:135 → §3.1
 ├─ if none found → walk the pool for ANY eligible (fallback loop)         680cc0.c:145-229
 │      logs "MANAGER_MANAGER::add_job_vacancy" if still none              680cc0.c:168-176
 ├─ PASS 2 — "shortlist" other interested managers (news only):
 │    random sample of pool; FUN_00684a80(p,club,0.2=0x33333333,3.15=0x40093333) 680cc0.c:251
 │      && FUN_00536990(p+0x3e,p+0x42) > 0x95 (min tenure/age)             680cc0.c:253
 │      && score FUN_00682420 argmax → local_32                           680cc0.c:256
 ├─ build appointment news date via FUN_008fc4f0 jitter                    680cc0.c:270-326
 ├─ FUN_00672320(&local_34, 0x26)  enqueue the appointment event           680cc0.c:328 → §5
 └─ POACH pass — for clubs whose current mgr this club could poach:
       FUN_005e6d30 / FUN_005e8590 gates, then
       FUN_00681c70(club, other_club)   job-poaching decision        680cc0.c:341 → §3.2

FUN_00681c70  poach_decision                                          [00681c70.c]
 ├─ FUN_008fc4f0(4)==0 gate ; FUN_00525450 not-national gate          681c70.c:36-37
 ├─ FUN_00536990(mgr+0x3e,+0x42) tenure vs 0xb3(179) ; ambition +0x59  681c70.c:38-51
 ├─ FUN_00681a50(mgr, mgr->club, club)  (do the poach linkage)         681c70.c:55
 ├─ jitter the target job-table confidence rows (RNG __ftol)           681c70.c:58-102
 ├─ score = FUN_00682420(mgr, club, 0, 5)                              681c70.c:119
 └─ FUN_00672440 / FUN_00672320(...,0x26)  enqueue if score beats best  681c70.c:233-235

FUN_00678aa0  evaluate_targets / build candidate reaction list        [00678aa0.c]  → §1e
 (called by FUN_00689fb0 ← FUN_00763c30 news path)

FUN_0067fdf0  manager_ai_over_squad driver                            [0067fdf0.c]
 └─ FUN_006805a0(club_index)  "should manager leave & why" (code 0..0xb)  6805a0.c → §3.5
 └─ FUN_006817b0(club, mgr)  rate current club (compare vs a poach score)
```

### 1a. `FUN_006822d0` — fix missing jobs
National-team clubs (`FUN_005265e0(club)!=0` region) that pass `FUN_0052e370`
(pickable) and have `FUN_0052a5a0` roster but no job record run
`FUN_00680cc0(club,0)`. Frequency: every driver tick.

### 1b. `FUN_006808a0` — vacancy queue
Drains `this+4` event list; each event is `{u8 status, u8 subcode, u32 clubId(+2), …, u32 mgrId(+6)}` (the struct copied by `FUN_00672400`). For each it resolves
`club->manager = club+0xcf`; if non-null, applies `FUN_006809e0` (the
resign/sack mutation) then re-hires via `FUN_00680cc0`.

### 1c. `FUN_00684fc0` — candidate reaction news
Fires message `0x1776` with the appointment payload (`FUN_0076d730` writes
fields 0..7: club id, three colleague ids at mgr[1..3]+0x33, job-status
person+0x3d, mgr->club id, target person id, and a random field). Gated on
`FUN_008fc4f0(4)!=0` and a date-window check. Pure UI/news — port last.

### 1d. `FUN_00680cc0` frequency
Called once per club that becomes vacant (from `006808a0`), once per
national-team gap (from `006822d0`), and from `0067fdf0` indirectly. Not a full
pool sweep every tick — event-driven.

### 1e. `FUN_00678aa0` — evaluate_targets
Builds three candidate buckets (definite / maybe / reach) for a club from the
stack-passed pool, using `FUN_004d59d0` (person→type10), `FUN_0052e070`,
`FUN_008b9a70`, rep window `club+0x80 ± 0x2ee(750)`, and person+0x18<0x23(35)
age gate (678aa0.c:74-113). Returns count; used to size the news shortlist. It
does not itself appoint.

---

## 2. Data structures + field offsets touched

Three record pools, all indexed by the record's `id` field (`*rec` = `+0x00`):

| Pool | Global base | Stride | Count |
|------|-------------|--------|-------|
| Person/staff | `DAT_00acd5c4` | `0x6e` (110) | `DAT_00acd56c` |
| Club (& national teams) | `DAT_00acd5bc` | `0x245` (581) | `DAT_00acd564` |
| Job-security table | `[*param_1]` (manager_manager field 0) | `0x49` (73) | per club, indexed by club id |

### 2.1 Person record (stride 0x6e)   — the manager
| Off | Type | Meaning | Evidence |
|-----|------|---------|----------|
| +0x00 | u32 | person id (pool index) | 682420.c:78 `*param_1` |
| +0x18 | i8 | **age / world-standing gate** (`cVar6`; compared vs 60/38/45/35) | 682420.c:40,197,206; 684a80.c:41 |
| +0x1a | u32 | nation-of-birth link | 682420.c:326,334; 683dc0 club side |
| +0x22 | u8 | attribute (loyalty?) gate `<0x4c` | 684a80.c:62 |
| +0x24 | u32 | **nation-job link** (national team managed) | 682420.c (param_1[9]); 675ae0 |
| +0x28 | i8 | nation-job status code | 674380.c:31 |
| +0x39 | u32 | **club-job link** (club currently managed) | 682420.c:77; 674c10.c:480 |
| +0x3d | i8 | **club job-status code** (see §2.4) | 682420.c:41; 674c10 |
| +0x3e..+0x41 | date | contract/appointment date A | `FUN_00536990(+0x3e,+0x42)` = tenure days, 684a80.c:56,79 |
| +0x42..+0x45 | date | contract date B | same call |
| +0x59 | i8 | **ambition** (squared into score; +0x15 scaled ×10 vs tenure) | 682420.c:80; 684a80.c:57,80 |
| +0x5a | i8 | attribute (patience/adaptability) | 680cc0/682420 |
| +0x5b | i8 | attribute — base of `local_98 = +0x5b − 10` | 682420.c:232,260,306,315 |
| +0x61 | u32 | managed-club record ptr (secondary; `param_1+0x61`) | 684a80.c:31,63 |
| +0x69 | u32 | **manager standing record** ptr — **null ⇒ score −10000** | 682420.c:33-35,71 |
| +0x65 | u32 | competition-context ptr (checks +0x10/+0x14/+0x18) | 684a80.c:26-28 |

Manager standing record (`person+0x69` →):
`+0x04 short`, `+0x06 short`, `+0x08 short` (rep axis), `+0x0a short`
(expectation), `+0x0c short` (current-form rep, compared vs 4250/3250). See
682420.c:73-75, 679ed0.c:27-30.

### 2.2 Club record (stride 0x245) — also national teams
| Off | Type | Meaning | Evidence |
|-----|------|---------|----------|
| +0x00 | u32 | club id | everywhere |
| +0x53 | u32 | nation link (`+0x85` = nation reputation tier byte) | 682420.c:56,334-342 |
| +0x57 | u32 | primary competition link (`comp+0x69` short = comp rep) | 684a80.c:48-53 |
| +0x80 | i16 | **club reputation** (all thresholds compare vs this) | pervasive |
| +0x88 | i32 | board-cash / patience counter (`<0x3e9`, `<0x65`) | 680cc0.c:53,58 |
| +0x8e | i16 | reputation cap multiplier (×200) | 674380.c:72 |
| +0xbf | u32 | **assistant-manager** person ptr | 0067fdf0.c:69; 675ae0 case1 |
| +0xc3 | u32[3] | coach array | 675ae0 case2 |
| +0xcf | u32 | **MANAGER person ptr** (authoritative) | 674c10.c:477; 6805a0.c:90; 0067fdf0.c:40 |
| +0xd3 | u32 | physio / DoF ptr | 675ae0 case6 |
| +0xc7,+0xcb | u32 | other role slots (reset in 674380) | 674380.c:79-80 |
| +0x19f | u32[5] | scout array | 675ae0 case8 |
| +0x1b3 | u32[7] | array | 675ae0 case9 |
| +0x1cf | u32[3] | array | 675ae0 case10 |
| +0xd7 | u32[0x32] | **affiliated / player-manager list** (50 slots; hire appends here w/ status 0xc) | 674c10.c:487-509; 6805a0.c:27 |

### 2.3 Job-security table row (stride 0x49, base `[*param_1]`, indexed by clubId)
Address of row: `job_table_base + clubId*0x49`. Byte layout:
| Off | Type | Meaning | Evidence |
|-----|------|---------|----------|
| +0x00 | u8 | flag | 674380.c:100 |
| +0x02 | u8 | flag bits (bit1 processed) | 0067fdf0.c:26,199 |
| +0x03 | u8 | flag bits | 674380 |
| +0x04 | u8 | flag | 674380 |
| +0x05 | u8 | flag | 674380 |
| +0x06 | i16 | **board confidence** (clamp 0x186a..0x251c=6250..9500) | 679ed0.c:47-56 |
| +0x08 | i16 | chairman patience (`<3000` ⇒ vacancy in 683dc0) | 679ed0.c:60; 683dc0.c:10 |
| +0x0a | i16 | fans confidence (min 0x1482=5250) | 679ed0.c:72-79 |
| +0x0c | i16 | media/expectation (min 0x1676=5750) | 679ed0.c:84-91 |
| +0x0e | i16 | =0x10 default | 674380.c:115 |
| +0x10 | i16 | grievance A (`< −0x15e` ⇒ code 3) | 6805a0.c:72 |
| +0x12 | i16 | grievance B — results (`< −0x2ee` ⇒ code 5; `< −300` w/ finance ⇒ 10) | 6805a0.c:59,75 |
| +0x14 | i16 | grievance C (`< −0x2ee` ⇒ code 5) | 6805a0.c:75 |
| +0x16 | i16 | grievance D (`< −0x2ee` ⇒ code 4) | 6805a0.c:86 |
| +0x18..+0x1b | u32 | packed date A (with +0x1c → FUN_00536990) | 0067fdf0.c:223-224 |
| +0x1c..+0x1f | u32 | packed date B | same |
| +0x28 | u32 | zeroed on reset | 674380.c:130 |
| +0x30 | f64 | **expected** points/pos² = `(club_rep)² × _DAT_009569e0` | 674380.c:101-103 |
| +0x38 | f64 | **actual** performance (0 at reset; `actual<expected` ⇒ code 10) | 6805a0.c:58 |
| +0x40 | u8 | flags (low 3 bits kept, rest cleared) | 674380.c:133-135 |

### 2.4 Job-status codes (person+0x3d and person+0x28)
Matches `crates/cm-domain/src/human_manager.rs::job_status` already:
`0`=none, `5`=manager, `6`=assistant, `8`=coach, `0xb`=player, `0xc`=player-manager,
`0xd`=player+nat-assistant, `0xf`=player+nat. In 675ae0 the switch also handles
`1`(single mgr slot +0xbf), `2`(+0xc3), `9`(+0x1b3), `0xa`(+0x1cf) for the
non-manager staff roles. Additional lifecycle status values seen in 674c10
switch on `person+0x3d`: `5,6,8,0xf` (unlink both), `0xb,0xc` (demote to 0xb).

---

## 3. Scoring formulas, thresholds, constants

### 3.1 `FUN_00682420` — interest/rating score (the scoring core, mode `c`)
Signature: `int FUN_00682420(person *p, club *c, refClub *r, char mode)`.

**Recoverable integer skeleton** (cite 682420.c):
1. `if (p+0x69 == 0) return -10000;` (no standing record). :33-35
2. Resolve `r`: if `r==0 || r+0x69==0`, `r = c+0xbf` (club assistant); if still
   invalid, `r=0`. :36-39
3. `mode8e = p+0x3d; if 0 → p+0x28; if still 0 → 0x0b`. :41-44
4. **Base rating** `local_94 = FUN_0052a330(p, r_or_stack, 1)` — the
   manager↔club reputation-fit primitive (returns short). :63,68
5. Standing bonus: if `mode8e ∈ {0x05,0x0c}` and `c+0x80 > 0x109a(4250)` and
   `standing+0x0c > 0xcb2(3250)`: `local_94 += standing+0x04 *0x19 + standing+0x06*5`. :72-75
6. **Incumbency**: if `p+0x39 == c` (already manages it):
   `local_94 += (p+0x59)²` (ambition²) — or `+100` if club id in extinct range
   (`p.id ≥ DAT_00acd56c−0x10`). :77-85
7. `FUN_00531370/005313b0(p,c)` (affinity A) and `FUN_005313f0/00531420(p,r)`
   (affinity B) each trigger an `__ftol()` float re-weight of `local_94`. :86-93
8. **mode switch** (`param_4`, the caller's 5): per-status float terms
   (`__ftol()` chains) then `local_98 = p+0x5b − 10`. :94-320
9. **Nation-reputation gate**: when `c+0x80 > 0x128e(4750)` and manager's nation
   `p+0x1a` set, compares nation tier byte `nation+0x85`; if manager's nation
   rep < club-nation rep and `mode8e ∉ {0x05,0x0c}` → `return -10000`. :329-348
10. **National-team branch** `FUN_00525450(c)!=0`: rating scaled by
    `FUN_0052a330`; small-club early accept `return 1` paths; date-jitter via
    `FUN_00533b50(0x17,6,0x7d3,-1)`. :350-419
11. Final loyalty adjust `FUN_007aeb90(p,c,nation+0x77,1)` → `__ftol()`. :421-424
12. Return `local_94` (or `local_98`-adjusted variants per `c+0x53+0x7e` code). :425-461

**DECODE BLOCKER (float weights).** Every `__ftol()` at 682420.c:88,92,204,
208-231,239-320,347,398,403,411,419,423,459 is an x87 multiply whose coefficient
Ghidra dropped. The integer scaffold above is exact; the **magnitudes** of the
float re-weights must be recovered from the x87 disassembly of `0x00682420`
before this fn is STATE-EXACT. Port order: implement the integer skeleton +
`FUN_0052a330` first (behavioural), then backfill float coefficients.

Recoverable rep thresholds used as hard branch points (short vs `club+0x80`):
`0xcb2`=3250, `0x109a`=4250, `0x128e`=4750, `0x1482`=5250, `0x1676`=5750,
`0x186a`=6250, `0x1a5d`=6749, `0x1a5e`=6750, `0x1c51/0x1c52`=7249/7250,
`0x1c53`=7251, `0x203a`=8250.

### 3.2 `FUN_00681c70` — poaching decision
* `FUN_008fc4f0(4)==0` random gate AND target not national (`FUN_00525450==0`). :36-37
* tenure `FUN_00536990(mgr+0x3e,+0x42)`: if `≤0xb3(179)` days AND ambition
  `mgr+0x59 > 1` → decrement ambition, `FUN_008fc4f0(ambition/3)` gate;
  else `FUN_008fc4f0(ambition+5)` gate. :38-51
* On pass: `FUN_00681a50(mgr, mgr->club, targetClub)` performs the linkage +
  RNG-jitters the target club's confidence rows (`__ftol` at :62-97). :55-102
* Extinct-club floors on `mgr->club+0x69`: min 0x5dc/0x4e2/0xfa. :104-114
* `score = FUN_00682420(mgr, targetClub, 0, 5)`; if `−250000` sentinel when no
  standing record. :116-119
* Enqueue winning candidate via `FUN_00672440` + `FUN_00672320(&local_234,0x26)`
  when `score > best (local_222)` and identity differs. :224-235

### 3.3 `FUN_00684a80` — would-accept / want-job (returns bool in AL)
Signature: `uint FUN_00684a80(person*, club*, int scoreLo, double thresholdHi)`.
`param_3`(double) is assembled from the two caller ints, e.g. `(0,0x400c0000)`
= **3.5**, `(0x33333333,0x40093333)` = **3.15**.

Hard-reject cascade → `goto LAB_00684ec5` (return 0):
* no standing record `person+0x69==0` or `param_2==0`. :17-18
* club too big for manager: `standing+0x0c > 0x109a(4250)` && manager's nation
  ≠ club nation && `clubNation+0x85 < 0x0a(10)`. :19-21
* `standing+0x08 > 0x186a(6250)` && nation mismatch && status `5` &&
  `club+0x80 < 0x1676(5750)`. :22-24
* already contextually committed (`person+0x65` comp slots == club). :25-28
* player-manager (status `0x0b`) special path: club big (`comp+0x0b>0x128e`,
  `club+0x80<0x1a5e`), type10 form-rank `type10+0x4f>>4 ∈[1,3]`, tenure
  `FUN_00536990(type10+0x25,+0x29) ≥ 0xb4(180)`, morale bytes. :30-45

Accept branch (returns 1) after passing the window checks :47-64:
* small home-nation club, no current job → `return 1`. :65-69
* `FUN_00525450(c)!=0` national side & same nation → `return 1` (or with the
  6749/7750/8250 comparisons). :71-101
* Else the numeric decision: `fVar9 = FUN_0082dab0(p, c, 0, 5, 0)`
  (**attractiveness score** primitive); `if (fVar9 >= thresholdHi) return 1`. :103-110

**DECODE NOTE.** The final numeric accept is delegated to `FUN_0082dab0`
(attractiveness, separate fn) compared against the caller's double. The doubles
(3.5 for a genuine vacancy, 3.15 for the news-shortlist pass) are exact; the
`FUN_0082dab0` internals are a separate decode task (transfer-adjacent).

### 3.4 `FUN_00683dc0` — club-eligible-as-target predicate (returns 1/0)
```
eligible(this, club) =
    club != 0
 && FUN_0052e370(club) != 0          // pickable / in a selected league (already ported: club_is_pickable)
 && FUN_00525450(club) == 0          // NOT a national team
 && job_table[club.id].[+0x08] < 3000 // chairman patience low
 && (board = FUN_0058a490(club)) != 0 // has a board/chairman object
 && board[+4] < 1 && board[+4] < 0    // board confidence negative (wants change)
```
(683dc0.c:8-14. The `+0x08` read is `*(short*)(club.id*0x49 + 8 + [*this])`.)

### 3.5 `FUN_006805a0` — "should the manager leave, and why" (returns code 0..0xb)
Reads the job-table row + club + person; RNG-gated. Return codes (6805a0.c):
| Code | Trigger | Line |
|------|---------|------|
| 0 | stay | fallthrough |
| 9 | ≥3 affiliated managers w/ flag `type10+0x45 & 0x10000000` | :39-41 |
| 0xb | international job pull (board object vtable +0x9c, rep math) | :48-55 |
| 10 | `actual(+0x38) < expected(+0x30)` AND grievanceB(+0x12) < −300 | :57-61 |
| 2 | grievanceB < −0x2ee(750) AND not-first-season AND `FUN_008fc4f0(3)==0` | :64-67 |
| 7 | job flag `+0x02 & 8` set | :68-70 |
| 3 | grievanceA(+0x10) < −0x15e(350) | :72-74 |
| 5 | grievanceC(+0x14) < −0x2ee | :75-77 |
| 8 | fans(+0x0a) < board(+0x06) AND RNG loss | :78-85 |
| 4 | grievanceD(+0x16) < −0x2ee AND `FUN_008fc4f0(3)!=0` | :86-89 |
| 6 | current-mgr tenure `FUN_00536990(mgr+0x3e,+0x42)` > `FUN_008fc4f0(2000)+1000` | :90-97 |

---

## 4. RNG usage

**Every** random draw in this subsystem is `FUN_008fc4f0(n)` — the pool-based
bounded RNG. Decompile (008fc4f0.c):
```
int FUN_008fc4f0(int n){
  if (n==0) return 0;
  DAT_00dc7238 += 4;                       // advance 4-byte cursor
  if (DAT_00dc7238 > DAT_00dc7a70) {        // past pool end?
     DAT_00dc7238 = &DAT_00a8df38;          // wrap to pool start
     DAT_00dc7234 = FUN_00935a94() & 0xffff;// reroll 16-bit jitter
  }
  if (-0x10000 < n && n < 0x10000)
     return (DAT_00dc7234 + *DAT_00dc7238) % n;   // (jitter + pool[cursor]) % n
  ... // wide-n path recurses (n*10/0xffff)
}
```
This is **exactly** `GameRng::rand_mod(n)` already implemented at
`crates/cm-domain/src/game_rng.rs:292` (cursor +=4, wrap re-jitters via
`lcg_next`, return `(jitter + pool[cursor]) % n`). Confidence HIGH: the memory
note [[c10-11-rng-byte-exact]] verified this stream byte-exact for fixture gen.

Seeding: the subsystem does **not** seed its own RNG — it consumes the single
session pool (`save.session_rng_state` / the `GameRng` passed into
`tick_cm_phase_with_rng`). So the Rust port must take `&mut GameRng` and draw in
the **same order** as the exe (order matters for stream parity).

Other helpers (NOT RNG, but stream-adjacent):
* `FUN_00536990(dateA, dateB)` — integer **day-count between two packed dates**
  (age / tenure). Deterministic; no RNG. Port as a date-diff util.
* `FUN_00533b50(0x17,6/7,0x7d3/0x7b3,-1)` — builds a future news-fire date; wraps
  `FUN_008fc4f0` internally.
* `FUN_0082dab0`, `FUN_0052a330` — deterministic rating primitives (no RNG).

---

## 5. How appointment / sacking mutate state

### 5.1 `FUN_00675ae0` — unlink person from a club/nation role
`FUN_00675ae0(person p, entity e)` where `e` is the club (or nation). Two arms:

**Club arm** (`p+0x39 == e`), switch on `p+0x3d`:
| status | mutation |
|--------|----------|
| 1 | `e+0xbf = 0; p+0x39 = 0` |
| 2 | clear `p` from `e+0xc3[3]`; `p+0x39=0` |
| 5 | if `p == e+0xcf`: `e+0xcf = 0`; `p+0x39=0`  (**manager vacate**) |
| 6 | `e+0xd3 = 0; p+0x39=0` |
| 8 | clear `p` from `e+0x19f[5]`; `p+0x39=0` |
| 9 | clear `p` from `e+0x1b3[7]`; `p+0x39=0` |
| 10 | clear `p` from `e+0x1cf[3]`; `p+0x39=0` |
| 0xc | `p+0x3d = 0xb`; if `p==e+0xcf`: `e+0xcf=0` (player-mgr → player) |
| 0xd | `p+0x3d = 0xb`; `e+0xd3=0` |
| 0xf | `p+0x3d = 0xb`; clear `p` from `e+0x19f[5]` |

**Nation arm** (`p+0x24 == e`): mirror table keyed on `p+0x28`, same offset
map, sets `p+0x24 = 0` at the end. (675ae0.c:103-179)

Tail: always `FUN_0082d140(p,e,4,2)` (news) and, for live clubs,
`FUN_007df410`/`FUN_0089eb70` (UI refresh). :180-186

### 5.2 Appointment writes (in `FUN_00674c10`)
On a validated hire event (674c10.c:477-533):
```
club+0xcf = person                         // install manager           :477
if FUN_00525450(club)==0 {                 // regular club
    person+0x39 = club                     // club-job link              :480
    // optional player-manager path: append to club+0xd7[0..0x32],
    //   person+0x3d = 0x0c, and refresh via FUN_004d6f80 / FUN_00843970 :487-509
    else person+0x3d = 0x05                 // plain manager              :511
} else {                                    // national team
    person+0x24 = club                      // nation-job link (piVar1[9]):523
    person+0x3d = 0x05                                                    :525
}
FUN_00679ed0(club)                          // seed board confidence      :532
```
Also fires `FUN_00688070(person,club,club2,mode)` (appointment news, mode 1/2/3)
and `FUN_004d6f80(person)` (type10 cache rebuild) on each transition
(674c10.c:270-431).

### 5.3 `FUN_00679ed0` — init/randomise board confidence on appointment
`FUN_00679ed0(club c)` — writes the job-table row for `c` (all via
`FUN_008fc4f0`). Sequence (679ed0.c), **preserve draw order**:
1. `jobrow+0x02 |= 1`; `FUN_006889e0(c,1)` (mark dirty).                  :20-22
2. Seed the **manager's** standing expectation `mgr(+0xcf)+0x69 +10 / +8`
   from RNG, branching on whether manager's nation == club nation. :24-35
3. Extinct-club floor draws on `standing+10` and `club+0x20` (RNG, discarded). :37-43
4. `jobrow+0x06` (board confidence): `2×FUN_008fc4f0(500)` then `__ftol`,
   clamp `[0x186a(6250), 0x251c(9500)]`. :44-56
5. `jobrow+0x08` = `+0x06 − rand(500) + rand(500)`, clamp `[0x186a,10000]`. :57-68
6. `jobrow+0x0a` = `2×rand(500)` → `__ftol`, clamp `[0x1482(5250),10000]`. :69-79
7. `jobrow+0x0c` = `2×rand(500)` → `__ftol`, clamp `[0x1676(5750),10000]`. :81-91
8. Zero grievances `+0x10/+0x12/+0x14/+0x16`; clear flag bits at `+0x02`,
   dbls at `+0x38/+0x3c`, `+0x04`, `+0x28`, `+0x40` low bits. :93-109
9. `FUN_00852a50(mgr, c, 1)` (news); if live club, `FUN_008b19b0`,
   `FUN_00833d80`, and per-affiliated-manager `FUN_00881910` over `c+0xd7[50]`. :110-123
10. Employment flag byte `jobtable[c.id*0x49 + 1]` = `FUN_00582870()` if the
    club has a board, else 0. :124-130

**Same `__ftol` blocker** as §3.1: the confidence-axis *centre values* are x87
multiplies. The clamp ranges above are exact; the pre-clamp mean needs the x87
disassembly. Behavioural port: draw `2×rand(500)`, midpoint ≈ `club_rep`-scaled,
then clamp — then backfill exact coefficient.

### 5.4 `FUN_00674380` — rebuild manager→club job links (season reset)
Full re-link pass over both pools (674380.c). Person loop: normalises
`person+0x24/0x28/0x39/0x3d`. Club loop: **zeroes every staff back-link array**
(`+0xbf,+0xc3,+0xc7,+0xcb,+0xcf,+0xd3,+0x19f,+0x1a3,+0x1a7,+0x1ab,+0x1af,
+0x1b3[7],+0x1cf,+0x1d3,+0x1d7`) and re-seeds the job-table row defaults
(`+0x06..+0x0c` = 5000 for national teams w/o managers, else RNG-generated wage
demands `+0x06/+0x08` clamp `[0x1676,0x251c]`, `+0x0a/+0x0c` clamp `[0x157c,10000]`,
`+0x30 = club_rep² × _DAT_009569e0`). Runs at new-game / season roll, not daily.

---

## 6. Existing Rust vs missing

### Already present (extend, don't duplicate)
* `human_manager.rs` — `HumanSeatPool` / `SeatSlot` with `take_control`,
  `resign`, `sack`, `is_club_human_managed`, `is_managed_by_active`,
  `job_status` module (5/6/8/0xb/0xc/0xf) — the **human-side** seat pool. The
  offset map (SEAT_*, EF_*) and slot geometry are decoded.
* `manager_creation.rs` — `club_is_pickable` (= `FUN_0052e370`, the §3.4
  pickable gate), the creation screens, and `create_manager` (= `FUN_00810f50`
  human take-control).
* `lib.rs`:
  * `HumanManager { identity, club, nation, reputation }` (1593) — thin
    human-side model.
  * `install_manager_at_club` (19762) — human install (sets rep=20, promotes
    nation tier). This is the **human** analogue of §5.2's link writes.
  * `resign` (19805) — human resign.
  * `hook_manager_job_lifecycle` (22407) — **empty stub**, the wiring point.
  * `GameRng::rand_mod` (game_rng.rs:292) — `FUN_008fc4f0` byte-exact.

### Missing (this port must add)
1. **AI-manager model.** Nothing models the ~thousands of *non-human* managers
   as job-holders with the person-record fields (+0x59 ambition, +0x69 standing,
   +0x3d status). `HumanManager` only covers the ≤16 humans.
2. **Job-security table** (stride-0x49 row per club): board/fans/media
   confidence + 4 grievance axes + expected/actual dbls. Not modelled anywhere.
3. **The scoring core** `FUN_00682420` (rating), `FUN_0052a330` (rep-fit),
   `FUN_0082dab0` (attractiveness).
4. **Eligibility/accept predicates** `FUN_00683dc0`, `FUN_00684a80`.
5. **Hiring round** `FUN_00680cc0`, **vacancy queue** `FUN_006808a0`,
   **poaching** `FUN_00681c70`, **leave-reason** `FUN_006805a0`.
6. **Confidence init** `FUN_00679ed0`, **link rebuild** `FUN_00674380`,
   **unlink** `FUN_00675ae0`.
7. **The driver** `FUN_00674c10` and its event queue.
8. `FUN_00536990` date-diff util (may exist under another name — check
   `game_date` helpers before adding).

---

## 7. Proposed Rust module shape + tick wiring

New module `crates/cm-domain/src/manager_hiring.rs`. Operate on the **existing
opaque records** (`DomainOpaqueRecord.raw` for person/club, as
`manager_creation.rs` already does via `ClubView`/`NationView`) so offsets stay
byte-addressed — do **not** invent parallel structs for the person/club pools.

```rust
// ── Job-security table (stride 0x49 row per club) ────────────────────────
pub const JOB_ROW_STRIDE: usize = 0x49;
pub struct JobSecurityRow { pub raw: [u8; JOB_ROW_STRIDE] }   // typed accessors
impl JobSecurityRow {
    pub fn board_confidence(&self)->i16;   // +0x06
    pub fn chairman_patience(&self)->i16;  // +0x08
    pub fn fans(&self)->i16;               // +0x0a
    pub fn media(&self)->i16;              // +0x0c
    pub fn grievance(&self, i:usize)->i16; // +0x10 + i*2  (i:0..4)
    pub fn expected(&self)->f64;           // +0x30
    pub fn actual(&self)->f64;             // +0x38
    // setters mirror each
}
pub struct JobSecurityTable { pub rows: Vec<JobSecurityRow> } // indexed by club id

// ── Predicates ───────────────────────────────────────────────────────────
pub fn club_is_vacancy_target(club:&ClubView, row:&JobSecurityRow,
                              board_conf:i16) -> bool;         // FUN_00683dc0
pub fn manager_would_accept(mgr:&PersonView, club:&ClubView,
                            threshold:f64, rng:&mut GameRng) -> bool; // FUN_00684a80

// ── Scoring ────────────────────────────────────────────────────────────────
pub fn manager_club_repfit(mgr:&PersonView, club:&ClubView) -> i32;  // FUN_0052a330
pub fn manager_attractiveness(mgr:&PersonView, club:&ClubView) -> f64;// FUN_0082dab0
pub fn score_manager_for_job(mgr:&PersonView, club:&ClubView,
                             refclub:Option<&ClubView>, mode:u8) -> i32; // FUN_00682420

// ── Mutations ────────────────────────────────────────────────────────────
pub fn unlink_person_role(person:&mut [u8], entity:&mut [u8]);        // FUN_00675ae0
pub fn appoint_manager(person:&mut [u8], club:&mut [u8],
                       table:&mut JobSecurityTable, rng:&mut GameRng); // 674c10 write + 679ed0
pub fn init_board_confidence(club:&ClubView, mgr:&PersonView,
                             row:&mut JobSecurityRow, rng:&mut GameRng);// FUN_00679ed0
pub fn rebuild_job_links(persons:&mut[..], clubs:&mut[..],
                         table:&mut JobSecurityTable, rng:&mut GameRng);// FUN_00674380

// ── Round + leave decision ─────────────────────────────────────────────────
pub fn leave_reason(club_id:u32, row:&JobSecurityRow, /*ctx*/,
                    rng:&mut GameRng) -> u8;                          // FUN_006805a0
pub fn hire_for_club(club_id:u32, outgoing:Option<u32>, world:&World,
                     save:&mut RuntimeSaveGame, rng:&mut GameRng);    // FUN_00680cc0
pub fn poach_decision(mgr_id:u32, target_club:u32, /*..*/,
                      rng:&mut GameRng);                              // FUN_00681c70

// ── Driver ────────────────────────────────────────────────────────────────
pub fn manager_manager_tick(world:&World, save:&mut RuntimeSaveGame,
                            today:&GameDate, rng:&mut GameRng);       // FUN_00674c10
```

**Tick wiring point.** Fill the existing stub — it already cites `FUN_00674c10`:

```rust
// lib.rs:22405
fn hook_manager_job_lifecycle(&mut self, date: &GameDate) {
    // manager_manager::tick — FUN_00674c10. Runs only when the day advanced
    // (exe gates on DAT_00b4d5b4 = last-run day). Needs &mut GameRng, so the
    // real body runs from tick_cm_phase_with_rng where the session RNG exists.
}
```
Because the body needs the session RNG **and** `&World` (for the person/club
pools), invoke `manager_manager_tick` from `tick_cm_phase_with_rng` right after
`hook_evening_daily_ai` (step 10) — the same place the stub sits — passing the
`rng` already threaded there. Guard it with a "day changed since last run" latch
stored on `RuntimeSaveGame` (port of `DAT_00b4d5b4`) so it fires once per game
day, not once per phase. The `World`-bearing overload lives on the
`tick_days_bound` path (lib.rs:19862) which already holds `&mut World`.

**Ordering constraint.** All AI RNG draws pull from the one session pool. To keep
stream parity with the exe, `manager_manager_tick` must run at the exe's slot
(after the daily-AI dispatcher, inside phase 2 evening) and draw in the decompile
order (§4). Wiring it elsewhere desyncs every downstream RNG consumer.

---

## 8. Per-function registry rows

Format matches `docs/gdi_registry/` convention (`dd_va | semantic | rust_symbol
| confidence`). All are DIRECT_REACHABLE from `FUN_00674c10`.

| dd_va | semantic | proposed rust_symbol | confidence |
|-------|----------|----------------------|------------|
| 00674c10 | manager_manager daily driver / event-queue apply | `manager_hiring::manager_manager_tick` | HIGH (flow) / MED (float clamps) |
| 006808a0 | vacancy-event queue drain → hire | `manager_hiring::process_vacancy_queue` | HIGH |
| 006822d0 | fix-missing-jobs sweep (national teams) | `manager_hiring::fix_missing_jobs` | HIGH |
| 00680cc0 | hire_for_club — candidate selection round | `manager_hiring::hire_for_club` | HIGH (flow) / MED (float score) |
| 00682420 | score manager interest/rating (mode c=6) | `manager_hiring::score_manager_for_job` | MED — integer skeleton HIGH, x87 float weights UNDECODED |
| 0052a330 | manager↔club reputation-fit primitive | `manager_hiring::manager_club_repfit` | LOW — not yet decoded (dependency) |
| 0082dab0 | manager attractiveness score (double) | `manager_hiring::manager_attractiveness` | LOW — not yet decoded (dependency) |
| 00683dc0 | predicate: club eligible as AI target | `manager_hiring::club_is_vacancy_target` | HIGH |
| 00684a80 | manager would-accept / want-job | `manager_hiring::manager_would_accept` | MED — gate cascade HIGH, delegates numeric to 0082dab0 |
| 00684ed0 | predicate: person already in job queue | `manager_hiring::person_in_job_queue` | HIGH |
| 00684fc0 | build candidate-reaction news (msg 0x1776) | `manager_hiring::news_candidate_reaction` | MED (news payload) |
| 00681c70 | manager job-poaching decision | `manager_hiring::poach_decision` | MED (float jitter) |
| 00678aa0 | evaluate targets / bucket candidates | `manager_hiring::evaluate_targets` | MED |
| 006805a0 | leave-reason code (0..0xb) for a manager | `manager_hiring::leave_reason` | HIGH |
| 0067fdf0 | manager-AI-over-squad driver (calls 6805a0) | `manager_hiring::manager_ai_over_squad` | MED |
| 00675ae0 | unlink person from club/nation role | `manager_hiring::unlink_person_role` | HIGH |
| 00674380 | rebuild manager→club job links (season) | `manager_hiring::rebuild_job_links` | HIGH (flow) / MED (float seed) |
| 00679ed0 | init/randomise board confidence on hire | `manager_hiring::init_board_confidence` | HIGH (flow) / MED (float centre) |
| 006908e0 | predicate: does person manage a club | `manager_hiring::person_manages_club` | HIGH |
| 006933f0 | manager availability gate (type2 vs rep=0x14) | `manager_hiring::manager_available` | HIGH |
| 008fc4f0 | pool bounded RNG `(jitter+pool[cur])%n` | `GameRng::rand_mod` (EXISTS) | HIGH — byte-exact |
| 00536990 | day-count between two packed dates | `manager_hiring::date_diff_days` (check game_date first) | HIGH |

`006908e0` (person_manages_club) and `006933f0` (manager_available) confirmed:
* `006933f0(record, kind)`: `return !(kind==2 && record+0x7f == 0x14)` — a
  type-2 (assistant?) with attribute byte +0x7f == 20 is unavailable; else 1.
* `006908e0(club, personCtx)`: walks a 2×20 candidate grid (`FUN_00710e80`
  builds it), returns `0xff` if a matching entry has `entry+0x35 < 7`, `1` if
  `> 7`, else `0` — a tri-state "manages / could / no".

---

## 9. Recommended implementation order

1. `JobSecurityRow` / `JobSecurityTable` + `PersonView`/reuse `ClubView`
   accessors (pure offsets, §2). Testable immediately.
2. `FUN_00536990` date-diff, `person_manages_club`, `manager_available`,
   `person_in_job_queue`, `club_is_vacancy_target` (small, HIGH-confidence).
3. `unlink_person_role` + appointment writes (§5.1/§5.2) — deterministic.
4. `init_board_confidence` / `rebuild_job_links` — RNG draw-order first, float
   centres flagged TODO until x87 decode.
5. `manager_club_repfit` (`0052a330`) + `manager_attractiveness` (`0082dab0`)
   decode (blocks exact scoring).
6. `score_manager_for_job` integer skeleton, then float backfill.
7. `manager_would_accept`, `hire_for_club`, `process_vacancy_queue`,
   `poach_decision`, `leave_reason`.
8. `manager_manager_tick` + wire into `hook_manager_job_lifecycle` with the
   day-latch (port of `DAT_00b4d5b4`).

## 10. Known decode blockers (must resolve for STATE-EXACT, not behavioural)
* **x87 float coefficients** in `00682420`, `00679ed0`, `00674380`, `00681c70`
  — Ghidra emitted `__ftol()` with no multiplier. Recover from the raw
  disassembly of each fn (the `fmul`/`fld` constants). Integer scaffolds + RNG
  draw order in this spec are exact; only the float magnitudes are open.
* `FUN_0052a330` and `FUN_0082dab0` are undecoded rating primitives — decode
  before `score_manager_for_job` / `manager_would_accept` can be numeric.
