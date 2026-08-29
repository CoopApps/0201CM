# Heuristic Kill-List — replacing every placeholder with the real ported logic

*Goal: no simplified/heuristic behaviour survives — every subsystem's behaviour
is a faithful port of the CM0102 exe, verified against the decompile. This is the
tracking document; work them one by one, top priority first, and mark DONE only
when the real logic is ported AND verified.*

Legend: 🔴 heuristic (not faithful) · 🟡 partial port · 🟢 faithful & verified

| # | Subsystem | Current heuristic | Real decompile target | State |
|---|-----------|-------------------|----------------------|-------|
| 1 | **Match squad feeding** | `snapshot_team_for_engine` returns None for ~1/3 clubs → `score_from_goal_events` (reputation-weighted) fallback (lib.rs) | team-selection / lineup builder (finding) | 🔴 |
| 2 | **Transfer valuation** | market value = `CA² × 100` (transfer.rs:138) | real player-valuation fn (finding) | 🔴 |
| 3 | **Transfer bid decision** | threshold: accept ≥ value & wage ≥ current; counter 80% (transfer.rs:141) | transfer_offer negotiation AI (finding) | 🔴 |
| 4 | **AI-club transfers** | only human bids? AI clubs may not trade | daily-AI transfer driver (finding) | 🔴 |
| 5 | **Loans** | screens only (screen_batch26/27); NO engine mechanic | loan lifecycle fn (finding) | 🔴 |
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
