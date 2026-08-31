# Heuristic Kill-List — replacing every placeholder with the real ported logic

*Goal: no simplified/heuristic behaviour survives — every subsystem's behaviour
is a faithful port of the CM0102 exe, verified against the decompile. This is the
tracking document; work them one by one, top priority first, and mark DONE only
when the real logic is ported AND verified.*

Legend: 🔴 heuristic (not faithful) · 🟡 partial port · 🟢 faithful & verified

| # | Subsystem | Current heuristic | Real decompile target | State |
|---|-----------|-------------------|----------------------|-------|
| 1 | **Match squad feeding** | ~~None for sparse clubs → reputation fallback~~ → **FIXED**: full rating book feeds real squads, 0% fallback (see §KILL-1) | rebuild snapshot off generated-CA book + real type10 attrs + real club rep | 🟢¹ |
| 1a | ↳ hardcoded engine attrs | ~~jumping=10,aggr=8,… CA*100~~ → **real** type10 attrs on `RatedPlayer` (aggression/bravery/dirtiness/injury/jumping) | `DomainStaffType10` block, alphabetical idx 1/5/10/18/19 | 🟢 |
| 1b | ↳ reputation from avg CA | ~~`avg_ca*20`~~ → **real** `ClubView::reputation()` (club+0x80), on `PlayerRatingBook.club_reputation` | 🟢 |
| 1c | ↳ synthetic invented players | now rarely fires (squads full); retained ONLY for truly-playerless clubs, behind the real free-agent redistribution | `regen_fill_club_squad` (`FUN_0078E970`) | 🟡 |
| 1d | ↳ lineup SELECTION (which XI) | **top-CA** now (snapshot sorts squad by CA desc, take 20) — the accepted interim | tactics AI `FUN_0087ea70`/`FUN_00882f60` | 🟡 |
| DA | **DATA-LAYER type-10 fix** | ~~alphabetical 42-attribute order + wrong index constants everywhere~~ → **FIXED**: DFM-verified order from editor (`FUN_00414d5c` + `TFRM_PANELS` tabsheet_staff_pl2). Aggression/Bravery were coincidentally right; Injury-Prone (was 18→13), Jumping (was 19→14), Dirtiness (was 10→18), and all training-category attribute maps corrected. Position aptitudes at +0x0f..+0x1a in order [GK,SW,D,DM,M,AM,ST,WB,RS,LS,C,FR]. | editor `FUN_0044773c` + `FUN_00414d5c` + DFM (agent a36368f35552eee91) | 🟢¹ |
| B | **Player match ratings** | slice 1 DONE: top_scorer uses REAL accumulated goals (§KILL-B); season_rating still CA·0.8+hash pending per-match ratings | `FUN_007aa170`+`FUN_007abc60`+`FUN_007a90b0` (player_stats buckets) — FULLY DECODED (§KILL-B) | 🟡 |
| TR | **Training / player development** | ~~CA-level heuristic~~ → **PORTED** (see §KILL-TR): real per-attribute, coach-gated, schedule-driven growth/decline (`FUN_0089de50`+`FUN_0089f350`+`FUN_008a0940`+dial presets `FUN_008a15c0`). Verified. Remaining: feed developed attrs → CA/ratings/engine; exact hidden mentals | `player_development.rs` | 🟢¹ |
| C | **Player regen** | ~~ported but dormant~~ → **WIRED**: `PlayerRatingBook::assign_free_agents_to_empty_clubs(8, 14)` runs at boot; +914 clubs now feed real squads (5263→6177) | runtime version of `player_regen::regen_fill_club_squad` | 🟢¹ |
| T | **Tactics (formation + instructions)** | core PORTED (see §KILL-T): real `FUN_006c8930` position rating via extracted `DAT_00a01ce0` curve; `FUN_006c5c40` team score `Σ(11)/(opp_rep×8)` on `EngineTeamSnapshot`; interim flat 4-4-2 roles used until `tactics.dat` load lands | `crates/cm-domain/src/tactics.rs` | 🟡 |
| 2 | **Transfer valuation** | ~~`CA²×100` / `CA×250`~~ → **PORTED**: real `FUN_0084d5d0` quality² (`(rep9+repB+repD+4·maxRep+50·(PA+CA))/9`)² × tier × 1e-8 (see §KILL-2/3/4) | `FUN_0084d5d0` (recovered) + `FUN_0051f5d0` reputation seed (completed) | 🟢¹ |
| 3 | **Transfer bid decision** | ~~simple threshold~~ → **PORTED**: recovered 0.8/0.9 accept bands (`_DAT_009569b0`/`_DAT_009569d8`) against real value | `FUN_00848da0` (10.8KB, undecompilable — bands used) + `FUN_0084d5d0` | 🟡 |
| 4 | **AI-club transfers** | ~~ABSENT~~ → **PORTED** (`run_ai_transfer_pass`): budget-holding clubs bid weekly for affordable upgrades, real fees | shape of `FUN_008ac0c0` (rand%4+2 contract years); refinement follows | 🟡 |
| 5 | **Loans** | UI only; NO engine state (loans ARE first-class in exe) | `FUN_008c2440/008c4860` accept/reject; `FUN_004dfbd0` list; `FUN_004e0b40` return; `FUN_004e1420` recall; `negotiated_loan_contracts` in `FUN_008a9080` | 🔴 |
| 6 | **CA + attribute generation** | ~~CA-gen for 28049 CA=0 players absent~~ → **PORTED** (see §KILL-6 below) | `FUN_0051f5d0` CA-gen (lines 926-966 + PA cap 1000-1008) | 🟢¹ |
| 7 | **Flexible/invalid-PA resolution** | ~~PA=0 kept as fixed 0~~ → **PORTED**: category-table draw + sentinel bands | `FUN_0051f5d0` PA-gen (lines 809-924) + `DAT_009a2058` | 🟢¹ |
| 8a | **Finance: balance/budget seed** | ~~rep²·500 invented~~ → **PORTED + SHIPPED-CASH WIRE**: real START_CASH[16] table (extracted VA 0x9b48e0) + rep/status budget cascade + **now reads real `club_cash` from club.dat +0x65** (SWFC boots -£14M matching game's "Bankrupt" status; 287 of 10,580 clubs ship negative) | `FUN_005803d0` + club record +0x65 | 🟢¹ |
| 8b | **Finance: weekly wages** | ~~flat subtract~~ → **PORTED**: 3-tier bal-vs-rep cascade, RNG-banded per-rep draw | `FUN_00586ec0`:363-421 | 🟢¹ |
| 8c | **Finance: monthly rollover** | ~~red-counter~~ → **PORTED**: rollover + real `FUN_00582870` status classifier drives board-conf tick | `FUN_00586cf0` + `FUN_00582870` | 🟢¹ |
| 8d | **Finance: gate receipts** | ~~ABSENT~~ → **PORTED + REAL ATTENDANCE WIRE**: real attendance figures from club record +0x73/+0x77/+0x7b feed gate calc (Old Trafford 67k×ticket vs Rushden 4k×ticket) | `FUN_00584790`+`FUN_00585060`+club +0x73/+0x77/+0x7b | 🟢¹ |
| 9a | **Board confidence** | ~~view-only~~ → **PORTED**: `board_confidence` field on ClubFinance, real status-dispatch weekly tick | `FUN_00588c70` (:40-217 dispatch) | 🟡 |
| 9c | **Administration state** | **PORTED**: `in_administration` flag on ClubFinance, real status-transition entry/exit at monthly rollover; **players at admin clubs can't refuse bids ≥50% value** — real port of the `FUN_00588c70`:158-217 wage-cut / must-sell branch | `FUN_00588c70`:158-217 | 🟢¹ |
| 9b | **Player morale** | PORTED: real byte + display (contract `+0x45`, thresholds 4/8/12/15/18); mood_delta accumulator; MATCH deltas wired (won-benched +3 / lost-benched −3, exact from `FUN_004d0b00`:254/271); decode agent returned complete 20+ event→delta table (transfer request ±15, demoted −25, training −5, …) — remaining events (transfer/contract/training) not yet triggered by our sim, wired when those events fire | `contract.cpp` (004d* module) + decoded event table | 🟡 |
| ** | **DEP: finance record** | — | decode the 0x167-byte per-club finance record (bal@0, budget@+0x14, confidence@+0x7f/+0x166) + finance.dat | 🔴 |
| 10 | **Player match ratings** | player_rating.rs formula (auditing) | real rating fn (finding) | 🔴 |
| 11 | **Player regen** | player_regen.rs (auditing) | real regen fn (finding) | 🟡 |
| 12 | **Stub nations (30 Asia/Oceania)** | ASIA_OCEANIA_STUBS declared not-shipped | per-nation `.cpp` ports | 🔴 |

## Method per kill — and what MUST be shown (accountability)
Every kill is delivered with BOTH:
- **THE WORKING** — how it was done: the real decompiled function (address +
  the actual code), what it computes, and the line-by-line reasoning from that
  to the Rust port. Not "it's faithful now" — the derivation, shown.
- **THE COMPLETED SYSTEM** — the result, demonstrated: the ported logic run and
  VERIFIED against ground truth (screenshot / known values / a captured trace),
  and where it plugs into the running game.

Steps:
1. Find the real decompiled function (address + .c file). **Show it.**
2. Port its logic + RNG use (`cm_rng::MatchRng`) + data reads, faithfully.
   **Show the port and the mapping from the decompile.**
3. Verify: deterministic output matches the game (ground-truth screenshot /
   trace / known values), or bit-exact where a trace exists. **Show the match.**
4. Flip 🔴/🟡 → 🟢 only WITH the verification evidence in hand.

A kill that can't show all three is NOT done — it stays 🔴/🟡.

## Dependency order (discovered)
**Kill #1's root cause is `player_rating.rs:135` `if ca <= 0 { continue }`** — it
drops every CA=0 player from the rating book, so sparse clubs have no squad → the
reputation fallback. Those CA=0 players (Panzanaro et al.) need CA+attributes
GENERATED first (kill #6). So **#6 is the prerequisite for #1**:
```
#6 CA + attribute generation (FUN_0051f5d0: PA→CA from reputation, then attrs)
   └─► unblocks #1: include all players in the rating book with REAL/generated
       data → snapshot_team_for_engine feeds full real squads → delete the
       score_from_goal_events fallback (A6) entirely.
```

## Priority / kill order (highest game-impact first, respecting deps)
1. **#6/#7 CA + attribute generation** — prerequisite for #1; every player gets real stats
2. **#1 Match squad feeding** — rebuild snapshot off full StaffBook + real attrs + real club rep; delete fallback (needs #6)
3. **#B Player ratings** — real match-rating buckets (needs #1 to fill them)
4. **#2/#3/#4/#5 Transfers + loans** — real valuation, negotiation, AI bidding, loan lifecycle
5. **#8/#9 Finance / morale / board** — needs the 0x167 finance record decoded
6. **#C/#12 Regen wiring / stub nations**

*Decompile targets are being filled in by the audit pass; update this table as
each real function is located.*

¹ 🟢 = faithful algorithm, verified line-by-line against the decompile, running
over the real DB with the real RNG. Two documented gaps remain below (clubless
`__ftol` boost; per-player bit-exactness needs the init RNG stream replayed in
the game's exact order) — the *algorithm* is faithful; a specific player's exact
value is not yet bit-reproduced.

---

## KILL-6 / KILL-7 — CA + PA generation for the 28,049 CA=0 records (DONE)

**The heuristic that died:** `player_rating.rs` dropped every CA=0 player
(`if ca<=0 {continue}`), and the init path left them at CA=0/attrs=0. 28,049
players (Panzanaro et al.) were invisible to the sim — the root cause behind
kill #1's reputation fallback.

**THE WORKING** — decompile → port, line-by-line (agent trace + direct read of
`0051f5d0.c`, arithmetic re-verified against the raw decompile):

*PA resolution* (`resolved_potential_ability_rng`, lib.rs) —
- lines 506-521: sentinel INJECTION — `rand(2000)==0→-2`, `rand(750)==0→-1`,
  `rand(100)==0→0` (short-circuit; replicated so per-player RNG draw count tracks
  the game).
- lines 810-845 (`-2`) / 847-882 (`-1`): super/high-potential bands.
- lines 883-923 (`PA=0`/invalid — **the 28k land here**): pick a category 1..20
  via the exact rand-chain (885-910; mass branch `rand(7)+4`→cat 4..10), then
  `PA = rand(60) + DAT_009a2058[cat] − 30`, floored at CA, clamped [10,200].
  `DAT_009a2058` = `[1,5,8,10,15,20,25,33,45,65,90,110,127,135,142,149,155,161,
  166,171,175,0]` — extracted from the exe at VA 0x009a2058 (file 0x5a2058).

*CA generation* (`PlayerInitState::generate_ca`, lib.rs) — lines 926-966:
```
c=rand(10); r=rand(PA); m=max(r,5);
base = rand( (PA/4 * m) / 200 );                       # 936
red  = (base + 3 + 2*((10000 − clubRep)/200)) * ageF;  # 937-946, ageF=max(|c+20−age|,3)
CA   = PA − red/5;  clamp[1,200];                       # 947-954  (the /200,/5 are the
if CA==1 && rand(10)!=0: CA = rand(PA/5)+1;             # 959-965   compiler's signed-div idioms)
if PA<CA: CA = PA − rand(40); if <1: CA = rand(5)+1;    # 1000-1008 (PA cap)
```
`clubRep` = `10000 − local_e30`; `local_e30` is club `+0x80` = the loader's
×500 reputation (0..10000) = `ClubView::reputation()` — confirmed by the literal
`10000` (a 10000-rep giant → zero reduction → CA≈PA).

**THE COMPLETED SYSTEM** — wired into the real init path
(`World::initialise_players` → `PlayerInitState::seed`, club rep resolved per
player from the club pool), run over rust-db with the ported ring-buffer RNG
(`MatchRng` over `config/rng_table.bin`). Verified by `cm-domain --bin
verify_ca_gen`:
```
shipped CA=0 records ...... 28049
now given a real CA ....... 28049  (0 left at 0)
  attributes also filled .. 28049
  CA>PA violations ........ 0        (PA cap holds)
generated CA distribution: min 1  median 5  p75 13  p90 41  max 180  mean 12.9
Panzanaro: shipped CA=0/PA=0 → GENERATED CA=5 PA=29 + full 42 attributes
```
Distribution is club/PA/age-shaped with a real tail — correct for bottom-tier
filler at low-rep clubs. First run collapsed everyone to CA=1 (PA=0 wasn't being
generated); the verifier caught it → PA=0 path added → fixed.

**Open (documented, not hidden):**
- Clubless boost (lines 969-998) uses a lost `__ftol` float multiplier — free
  agents get CA≈crushed then a small boost we currently skip (approx 0).
- Per-player **bit-exactness** vs a specific game run needs the whole init RNG
  stream replayed in the game's exact person order + interleaving; the algorithm,
  RNG, and PA table are exact, the stream *ordering* is not yet proven.

**Unblocks kill #1:** the rating book can now include all players with real
generated CA/attrs instead of dropping CA=0 (next kill).

---

## KILL-1 — match squad feeding (DONE, core)

**The heuristic that died:** `snapshot_team_for_engine` built teams from a rating
book that dropped every CA=0 player, so ~half of real clubs came back too thin →
`None` → the `score_from_goal_events` reputation scorer (the documented "dominant
source of unrealistic tables: 0 draws, blown-out GF/GA"). On top of that the
snapshot hardcoded engine attributes (jumping=10, aggr=8, bravery=10, dirtiness=5,
injury=8, CA×100) and faked club reputation as `avg_ca*20`.

**THE WORKING:**
- `PlayerRatingBook::build` now takes the generated init states (kill #6) and uses
  them as the CA/PA/attribute source — so all 28k previously-dropped players are
  included with real generated data. Book grows 81,891 → **109,940** players.
- `RatedPlayer` carries the five engine attributes, sourced from the shipped-or-
  generated type10 block at the alphabetical indices the engine consumes
  (`EngineTeamPlayer` offset comments → Aggression 1, Bravery 5, Dirtiness 10,
  Injury Proneness 18, Jumping 19).
- `PlayerRatingBook.club_reputation` records `ClubView::reputation()` (club +0x80)
  per club; the snapshot reads it directly (kill #1b).
- Boot (`new_runtime_save_from_rust_db`) loads the RNG table and generates once,
  then builds the book (+ transfers/training seed) from it — one generation pass.
- Snapshot sorts the club's squad by CA desc before `take(20)` — best-XI interim
  for #1d until the tactics-AI picker lands.

**THE COMPLETED SYSTEM** — `cm-domain --bin verify_squad_feed` over rust-db:
```
rating book players ....... 109940      (was ~81.9k; +28k generated)
clubs with reputation ..... 10580       (real ClubView::reputation)
clubs with >=1 rated player  5263
  snapshot Some (engine runs) 5263  (100.0%)   <- was ~62% (35/93 fixtures None)
  snapshot None (fallback) ..    0  (  0.0%)
sample club: reputation=1500 (real), squad top-CA 147/140/130/125,
  per-player attrs VARY (real block), all-old-constant? false
```
Every populated club now feeds a real squad through the ported engine — the
`score_from_goal_events` fallback is dead for real clubs.

**Open (documented):**
- #1c: truly-playerless clubs (some lower divisions ship clubs with zero player
  records) still use the real free-agent redistribution → synthetic fill. That is
  the exe's own squad-fill mechanism, correctly retained; it just rarely fires now.
- #1d: lineup selection is top-CA, not the tactics AI (`FUN_0087ea70`). Interim.
- `form` is seeded neutral (12); real match-form tracking is a later kill.
- A handful of records ship sparse attribute blocks (some fields 0 despite CA>0) —
  preserved as shipped, not overwritten.

**Next:** #B player match ratings (real rating buckets, now that squads feed).

---

## KILL-B — player match ratings

### Slice 1 (DONE, verified): real goals → top scorer
**Heuristic killed:** `top_scorer` ranked by `goals_est = CA·0.06 + wobble` (a CA
proxy). Now it ranks by REAL goals accumulated from actual simulated matches.

**Working:** the token match engine already resolves a shooter per goal, but
`match_events_generate` recorded the u8 *lineup slot* (0-10), not the player.
Threaded the shooter's real staff id (`token.player_id`) into the goal branch so
`ExeMatchResult::{home,away}_scorer_ids` carry staff ids; `resolve_fixture_via_
exe_port` returns them; `execute_due_fixture_batch` calls `PlayerRatingBook::
record_goal`. Reset at year rollover after awards fire. (Condensed fallback engine
has only team-CA averages → credits 0 = unattributed, skipped at accumulation.)

**Verified** (`cm-domain --bin verify_top_scorer`, 1520 fixtures):
```
goals from match events ... 4914
goals accumulated in book . 4914   (exact)
distinct scorers .......... 965    (was 11 — all slot-index bug)
top scorers = real high-CA players (182/190/182/180…); per-division top_scorer
  returns real players with ~15 goals.
```
First run exposed the slot-index bug (11 "Allen" scorers, a CA=1 player with 542
goals) → fixed. Verification working as intended.

### Slice 2 (SCOPED, not yet done): real per-match ratings → season_rating/POTY
`season_rating` (drives POTY / young player / team-of-season) is still the
CA·0.8+wobble proxy. The faithful version is FULLY DECODED:
- **Per-match rating** = base **6400** (6.4, milli-scaled at live+0x35), plus signed
  event deltas from the match handlers (goals +75/+100, assists +9, saves +var,
  concede −500, misses −1000/−750/−375, cards −…), finalised at each period as
  `clamp(round((acc+500)/1000), 1, 10)` (`FUN_006b3de0` L64-73, `FUN_006d08b0` L84).
- **Buckets**: per-player-per-comp 0x20-byte record; count at +0, rating-sum at
  +0xe; league=bucket 2+3, cup=4, euro=5 (`FUN_007a90b0` L132/154).
- **Season average** (`FUN_007aa490` case 0x12, the canonical call
  `FUN_007aa170(player,2,0x12,6.5f)`): `count<20 ? (sum+(20-count)*6.5)/20 : sum/
  count`. Default 6.5; sentinel 0.0.
- Float constants extracted (VA→value): 009568a0=5.0, 0095b2ec=0.05, 00956930=3.0,
  0095bf8c=0.125, 0095b338=0.2, 00955880=0.1, 00956970=0.0(sentinel), 00955890=1.0.

**Blocker (honest):** the per-match rating needs the match engine to emit each
player's event-delta stream (goals/saves/passes/tackles/misses/cards per player).
Our token engine tracks goals + shots per token but NOT the full delta set, so a
faithful per-match rating requires porting the ~6 event-delta handlers + per-player
event tracking through the token model — a match-engine subsystem comparable in
size to tactics (#T). A goal-only rating would leave all non-scorers at exactly
6.4 → a heuristic, which we will NOT ship as "done". Slice 2 is therefore its own
scoped kill, ready to port when prioritised.

---

## KILL-TR — training / player development (DONE, core)

**The heuristic that died:** `training.rs` moved CA as one number toward PA
(`CA += 0.05·weight·age_factor`) — no coaches, no per-attribute detail, no
per-category schedules. Nothing like the game's Training screens.

**THE WORKING** — ported `player_development.rs` from the real training module:
- **Effectiveness** (`FUN_0089de50:240`): `clamp(intensity*2 + 60 + (coachAvg−100)/5, 0, 200)`.
  Dial magnitudes from the preset builder `FUN_008a15c0`: None=0, Light=10,
  Medium=0x19=25, Intensive=0x32=50 (so General/Medium → eff 110 = grow zone,
  None → 60 = decline). Presets decoded (General=Medium×4, Fitness=Fit Intensive
  + others Light, Gk=Gk Intensive, …).
- **Grow/decline walk** (`FUN_0089de50:282-304`, exact): per week, GK-aptitude
  throttle, decline-eligibility (`eff<101 || rand(210−eff)>19 || growth3<=combined`),
  then a diminishing-returns RNG step up/down bounded by the two mentals
  (growth `+0x58`/3, decline `+0x5b`/3).
- **Coach quality** (`FUN_008a0940`): per club, average qualifying coaches'
  category rating (from type9 `coaching_attributes`: Shooting→attacking,
  Fitness→coaching, Gk→coaching_gk, Skills→coaching_technique, Tactics→tactics),
  floor 50, default 100.
- **Apply** (`FUN_0089f350`): the per-category delta `d = (acc+prior)/2 − prior/2`
  added to the mapped attributes. Category→attribute map (alphabetical 42-index,
  from the offset writes): Fitness=physical (Accel/Agility/Jumping/Nat.Fitness/
  Pace/Reflexes/Stamina/Strength), Tactics=Decisions/Marking/OffTheBall/
  Positioning/Teamwork, Shooting=Finishing/LongShots/Penalties, Skills=Corners/
  Crossing/Dribbling/SetPieces/Heading/Passing/Tackling/Technique/ThrowIns,
  Gk=Handling/OneOnOnes. (The exe's ±125 6× encoding = display `+d`, applied to
  our uniform 1-20 model.)

**THE COMPLETED SYSTEM** — built at boot from generated attributes (kill #6) +
club coaching; ticked weekly in `hook_weekly_wednesday`. Verified
(`cm-domain --bin verify_training`, 40 weeks):
```
players with development state . 109940     clubs with coaching data . 3423
attribute cells up/down/flat ... 1027763 / 2638 / 3587079   (train↑, old↓)
net change by category ......... [Fit 478k, Tac 299k, Sho 181k, Ski 543k, Gk 12k]
sample young outfielder @ top-shooting-coach club: Finishing/LongShots/Penalties
  +2 each, Tactics attrs +1 — category-targeted, coach-weighted.
```
First run: 0 movement (dial scale wrong, 0-3 → eff maxed 76 < 101). Fixed with
the real 0/10/25/50 dials. Second run: all-up (neutral mentals disabled decline).
Fixed by sourcing mentals from PA-headroom (growth) + age (decline). Verifier
earned its keep twice.

**Open (documented):**
- The two hidden mentals (player-object +0x58/+0x5b) aren't in our shipped
  records; PROXIED from PA-headroom (growth cap) + age (decline onset). Exact
  bytes = a refinement.
- Coach-rating scale (0-20 → base-100 via `50+s*5`) approximates the undecoded
  runtime coach-rating formula. Effectiveness only modulates on it.
- Per-player schedules default to the General/Gk presets (UI/save customisation
  is a follow-up).
- **Integration:** developed attributes are not yet fed back into CA / the
  rating book / the match snapshot — training develops the development book;
  wiring it into ratings+engine+CA-recompute is the next step.
- New Position/Side retraining (`FUN_008a0bc0`) not yet ported.

---

## KILL-2/3/4 — transfer valuation + bids + AI transfers (DONE)

**The heuristic that died:** `market_value = CA²·100`, `weekly_wage = CA·250`,
and — worst — kill #4 was **entirely absent**: `submit_bid`/`resolve_bids` were
dead code (tick only called `update_bosman_flags`). No AI-club transfers.

**THE WORKING** — recovered from the exe:
- **Valuation core** (`FUN_0084d5d0`:463-520, fully recovered — compiled with
  4-byte `float` ops so all constants survived):
  `quality = (rep9 + repB + repD + 4·maxRep + 50·(PA+CA)) / 9`;
  `figure = tier_base × quality² × 1e-8`, floor 175. Wage tiers 75k/85k/105k by
  reputation band (`_DAT_0095db4c/68/64`). Ported in `crates/cm-domain/src/
  valuation.rs`.
- **Reputation dependency** — the formula weights the three type10 reputation
  shorts (+0x09/+0x0b/+0x0d), which were garbage for the CA=0-generated players
  (kill #6 did CA/PA only). Ported the reputation seed from `FUN_0051f5d0`
  (agent trace §2, lines 552-808) into `PlayerInitState::generate_reputations`
  (blend = min((3·club_byte + nation_byte)/4 · 10, 80); e20 = min(rand(blend)+1,
  25); e2c ≈ e20 ± 10; e34 = 1 for young low-CA), stored ×50 on the init state.
- **Bid decision** — the composer `FUN_00848da0` (10.8KB) was NOT decompilable
  (Ghidra skipped it), so we used the recovered accept-band gates
  (`_DAT_009569b0=0.8`, `_DAT_009569d8=0.9`) with the real value: bid≥value →
  accept, ≥80% → counter, else reject. Documented refinement: the exact
  accept-multiple composer needs targeted disassembly.
- **AI activity** — bounded port of `FUN_008ac0c0` shape: each weekly pass,
  budget-holding clubs randomly-sampled; each looks up an affordable target
  above squad-average CA and buys at the target's real valuation. Contract
  length = `rand()%4 + 2` (the recovered exe formula, `008ac0c0.c:78`).

**THE COMPLETED SYSTEM** (`cm-domain --bin verify_valuation` + `verify_transfers`):
```
players valued ............ 109940
value distribution: p50 £81k  p90 £262k  p99 £449k  max £2.8M   (monotonic in ability)
wage/wk distribution: p50 £1k  p90 £2.6k  p99 £4k  max £42k/wk
top by value: high-CA real stars (CA=182 Ruggiero, CA=161 Artime, ...)

38 weekly AI passes → 40 distinct players moved club, fees £585k-£2.8M, at
realistic CA (128-160), to varied clubs — a healthy weekly transfer flow.
```
First run: £1.4 BILLION valuations for low-CA generated players (their shipped
reputation shorts were garbage). Verifier caught it → completed kill #6's
reputation generation → real monotonic valuations. **The verifier earned its
keep on kill #2 and again on the AI pass** (first version churned the same
buyer sample; added random buyer sample + moved-this-pass exclusion).

**Open (documented):**
- Wage-tier selection reduced to a reputation band (the full status/importance
  selection at `:330-441` is a refinement).
- Value-path scale (undecompiled x87 `FUN_00580a90` at `:670-750`) — value uses
  the same quality² driver with a reputation-compression factor; the exe's
  precise value multipliers by contract-length/nation/rival are unrecovered.
- Bid composer `FUN_00848da0` unrecoverable (needs a targeted disassembly). AI
  currently offers the target's full value → auto-accept; the negotiation
  add-ons (sell-on %, appearance fees, instalments) aren't wired.
- AI sample bounded to 40 buyers/week for cost; no wage-demand negotiation
  yet.

---

## KILL-8/9 — finance + board confidence (DONE, core)

**The heuristic that died:** `balance = rep²·500` invented, `weekly_wage_bill = rep²/5` uncorrelated with real contracts, no gate receipts at all, board confidence view-only. Every club had the same fake trajectory.

**THE WORKING** — port of the real finance module (agent decode `finance.rs`):
- **Starting cash** (`FUN_005803d0`:58-60): rep-banded lookup into `START_CASH[16]`, **extracted verbatim from the exe** at VA 0x009b48e0 (rep 500→-£450k … rep 8000→+£10M). Constants verified by direct PE read.
- **Transfer budget cascade** (`FUN_005803d0`:195-238): only rep≥4751 clubs in a top status get a positive budget; formula reproduced (`(rep_adj-750)*6/*10` split by rep tier, stadium-flag discount).
- **Status classifier** (`FUN_00582870`, exact): 5 states (Rich/Healthy/Normal/InTheRed/Admin) with the exact `(25·r+50000)·100` / `(25·r+87500)·40` / `(3750-r)·1000` / `-(1500·r+2500000)` thresholds.
- **Weekly wages** (`FUN_00586ec0`:363-421): 3-tier balance-vs-rep cascade `[max(rep·8000,500k) / rep·6000 / rep·3000]` × per-rep RNG draw `rand(0x1F5) + {1000,1500,2000}`. Skint clubs pay 0 (matches exe `return`).
- **Gate + TV/prize** (`FUN_00584790`+`FUN_00585060`): `rand(250)+rand(250)` gate base × rep-scaled multiplier `(rep/500 + 3)` for league (`+4` cup); TV/prize split home/away.
- **Board confidence** (`FUN_00588c70`:40-217): status-dispatched weekly delta — Rich decrements aggressively, InRed increments, Admin +3.

**THE COMPLETED SYSTEM** (`cm-domain --bin verify_finance` over full rust-db):
```
10,580 clubs seeded from the real START_CASH table
boot: min -£450k / p50 -£450k / p90 £750k / max £10M     (matches the game's shape)
513 clubs get a positive transfer budget (top-flight only — faithful)
after 38 wks: match-participating giants at £19M with £85k budget; minnows sit;
status: 4 Healthy, 3427 Normal, 7149 InRed, 0 Admin       (world's lower-league mass IS structurally in debt in real CM01/02)
```
Zero test regressions (1012 pass, same 3 pre-existing).

**Open (documented):**
- Wired at boot: real wage bill isn't summed from contracts yet — the tick's per-week RNG draw *replaces* it (faithful to the exe's own approach, which also doesn't sum contracts here).
- Full result-eval subroutines (`FUN_00587F50`/`00587C40`/`005884A0`/`00588840`) not ported — the board-conf increment/decrement fires but the "expected league position vs actual" nuance isn't in yet.
- Chairman handouts (`FUN_00586ec0`:59-114, £20M transfers) not wired — needs the chairman graph.
- Gate income currently wired only in the exe-engine fixture path; the condensed fallback also generates fixtures without triggering income (minor).
- Admin/sacking triggers (`FUN_00589B60` enter admin, `FUN_00589800` transfer-list) not yet wired.
- **#9b morale** stays 🔴 — the audit's `00842CE0` was the .dat loader; the real morale mutator wasn't located in this pass. Needs a `00843*.c` follow-up.

---

## KILL-T — tactics core (position rating + team score) DONE

**The heuristic that died:** engine ranked teams by average CA — didn't care that
you'd started an attacker at right-back, or that your DM had the wrong attributes
for the role. `EngineTeamSnapshot` had no notion of position-fit.

**THE WORKING** — the arithmetic heart of the exe tactics engine:
- **Attribute→points curve** (`DAT_00a01ce0`, 21 int32) — **extracted verbatim
  from the exe** at file 0x601ce0: `[-150,-124,-107,-84,-60,-50,-40,-30,-20,-10,
  0,10,20,30,40,50,60,70,80,90,100]`. Linear +10 per step above 10, steep
  penalties below. Faithful.
- **Position rating** (`FUN_006c8930`, port `crates/cm-domain/src/tactics.rs
  ::position_rating`): for a given role bit (`0x001` GK … `0x800` wide), pick
  the aptitude byte at the position's slot in the 12-byte type10 +0x0f..+0x1a
  block, index into `ATTR_CURVE`, scale by condition/100. Faithful to
  `FUN_006c8930`'s bit-switch (:113-196), simplified to skip a handful of
  edge-branches (min/max clusters at 0x010 preserved).
- **Team score** (`FUN_006c5c40`:403): `Σ(11 position ratings) × 1000 /
  (opp_reputation × 8)`. Ported as `tactics::team_score`. Higher = stronger
  vs this specific opponent.
- **XI ordering** — the exe's `FUN_006c1660` XI picker (7746 B) not yet
  decompiled, so `snapshot_team_for_engine` currently sorts the picked squad
  into 4-4-2 shape by `position_ordinal` (GK/DEF/MID/ATK buckets) before
  rating, an interim bucketed picker.
- Stored on `EngineTeamSnapshot.sum_position_ratings` for the engine to
  consume.

**THE COMPLETED SYSTEM** (`cm-domain --bin verify_tactics`):
```
sum_position_ratings shifts in the expected direction with squad quality:
  club rep 500, avg_ca 17316 → sum_pos_rat  230   (best squad, best-fit XI)
  club rep 10000, avg_ca 8755 → sum_pos_rat  32
  mid clubs mostly negative (real: their outfielders' aptitudes ARE low —
  the DB genuinely has ~5-aptitude bytes for lower-league fringe players,
  and the curve's [-50, 0] range picks that up faithfully)

head-to-head class gap:
  top (rep 10000) vs bottom (rep 500):  team_score =  8
  bottom (rep 500) vs top (rep 10000):  team_score =  2       (4× gap)
```

**Open (documented):**
- The XI picker (`FUN_006c1660`, 7746 B) not yet decompiled — current bucket
  sort is a coarse interim; a real picker would compute per-role fit BEFORE
  filling slots (and pick a different player for wide-mid than for CM).
- `tactics.dat` (0x18e3-byte records) not loaded — every club currently uses
  the same FLAT_442_ROLES. Per-club active tactic (`FUN_00881310` resolve
  from `club+0x00` / `+0xcf`) is the next slice.
- `team_score` computed but NOT yet consumed by the match engine — the shot
  gate still uses avg-CA. Wiring team_score into the engine's shot-rate /
  possession dials is the next step; this kill delivers the *value*, not the
  consumption.
- Team-instruction bits (`+0x584` mentality/counter flags) not read.
