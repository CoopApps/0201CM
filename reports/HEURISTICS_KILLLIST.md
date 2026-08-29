# Heuristic Kill-List — replacing every placeholder with the real ported logic

*Goal: no simplified/heuristic behaviour survives — every subsystem's behaviour
is a faithful port of the CM0102 exe, verified against the decompile. This is the
tracking document; work them one by one, top priority first, and mark DONE only
when the real logic is ported AND verified.*

Legend: 🔴 heuristic (not faithful) · 🟡 partial port · 🟢 faithful & verified

| # | Subsystem | Current heuristic | Real decompile target | State |
|---|-----------|-------------------|----------------------|-------|
| 1 | **Match squad feeding** | `snapshot_team_for_engine` built from thin CA>0 `player_ratings` book → None for sparse clubs → `score_from_goal_events` reputation fallback (lib.rs:17995,18171) | REAL CHAIN: `FUN_00834fc0` (simulated_stats) → `FUN_00882240` (tactics lineup: reads club's 20-slot XI from staff pool) → `FUN_0069d950`/`FUN_0069f2f0` (engine) → `FUN_007a90b0` (player stats). FIX: rebuild snapshot off full `StaffBook`+real type10 attrs+real club rep (club+0x80); wire `regen_fill_club_squad` at boot; delete the None path | 🔴 |
| 1a | ↳ hardcoded engine attrs | `lib.rs:18013` jumping=10,aggr=8,… CA*100 | wire real `DomainStaffType10` reads | 🔴 |
| 1b | ↳ reputation from avg CA | `lib.rs:17998` `avg_ca*20` | real club `+0x80` (ClubView::reputation) | 🔴 |
| 1c | ↳ synthetic invented players | `lib.rs:19329` | already-ported `regen_fill_club_squad` (`FUN_0078E970`), just wire at boot | 🔴 |
| 1d | ↳ lineup SELECTION (which XI) | top-CA (once real squads feed) | tactics AI `FUN_0087ea70`/`FUN_00882f60` | 🔴 |
| B | **Player match ratings** | `player_rating.rs` CA·0.8+hash proxy (:64,155) | `FUN_007aa170`+`FUN_007abc60`+`FUN_007a90b0` (player_stats buckets); storage `FUN_0074f760`/`FUN_00920420` | 🔴 |
| C | **Player regen** | ported but DORMANT (only in tests) + `FLOAT_TERM_BOUND=100`, stubbed flags | wire `regen_fill_club_squad` at boot; decode `FUN_00935080`; `DAT_00acdf0c`/club+0xcf | 🟡 |
| 2 | **Transfer valuation** | `CA²×100` (transfer.rs:136); wage `CA×250`; age-ladder length | `FUN_00580a90` (staff_club_valuation, 25 callers) + `FUN_004d7090` (offer value) | 🔴 |
| 3 | **Transfer bid decision** | threshold accept/counter/reject (transfer.rs:141) | `FUN_00848da0` + `FUN_0084d5d0` (accept/wage bands); `FUN_008d4a30/b10` offer totals; `FUN_004db1e0` negotiation | 🔴 |
| 4 | **AI-club transfers** | ABSENT — `resolve_bids/submit_bid` dead, tick only calls bosman (lib.rs:18861) | `FUN_008ac0c0` (AI offer gen) + `FUN_008ab360` (add_offer) + `FUN_004dd960` (listing/asking) | 🔴 |
| 5 | **Loans** | UI only; NO engine state (loans ARE first-class in exe) | `FUN_008c2440/008c4860` accept/reject; `FUN_004dfbd0` list; `FUN_004e0b40` return; `FUN_004e1420` recall; `negotiated_loan_contracts` in `FUN_008a9080` | 🔴 |
| 6 | **Attribute generation** | CA-anchored core only; missing position-weighting + Pass-2; and CA-gen for 24858 CA=0 players | `FUN_0051f5d0` (spec'd; partly ported) | 🟡 |
| 7 | **Flexible-PA resolution** | band midpoint, not RNG-exact draw (type10) | `FUN_0051f5d0` PA-gen (lines 809-924) | 🟡 |
| 8a | **Finance: balance/budget seed** | `finance.rs:96` rep²·500 (invented) | `FUN_005803d0` (005803d0.c:47-239); loader `FUN_005853c0`; ctor `FUN_00584530` | 🔴 |
| 8b | **Finance: weekly wages** | `finance.rs:134` flat subtract | `FUN_00586ec0` tail (00586ec0.c:363-422) — rep-banded + rand | 🔴 |
| 8c | **Finance: monthly rollover** | `finance.rs:144` red-counter | driver `FUN_00586cf0` + status classifier `FUN_00582870` | 🔴 |
| 8d | **Finance: gate receipts** | ABSENT | `FUN_00584790` (post-match income) + helper `FUN_00585060` | 🔴 |
| 9a | **Board confidence** | `screen_manager_batch.rs:126` view-only, no model | `FUN_00588c70` (confidence/budget at club +0x7f/+0x166) | 🔴 |
| 9b | **Player morale** | `lib.rs:783` inert byte, static seed 11 | TBD — match-result/squad_manager.cpp (0x00842ce0+) | 🔴 |
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

## Priority order (highest game-impact first)
1. #1 Match squad feeding — makes tables realistic (the sim's core output)
2. #6/#7 Attribute generation + PA — every player's real stats
3. #2/#3/#4/#5 Transfers + loans — the market feels alive
4. #8/#9/#10 Finance / morale / ratings
5. #11/#12 Regen / stub nations

*Decompile targets are being filled in by the audit pass; update this table as
each real function is located.*
