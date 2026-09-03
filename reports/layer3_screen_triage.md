# Layer 3 screen-builder triage

Total TodoBuilder arms parsed: **59**

- **Class A (decomp-ready, ≤40000 bytes .c)**: 57
- **Class B (GDI asm only, no decomp)**: 0
- **Class C (skipped)**: 2


## Class A — port these first (57)

| cmd | scope | primary FUN_ | decomp .c | GDI asm | fn size | note |
|-----|-------|--------------|-----------|---------|---------|------|
| `0x426` | club | `007eb990` | 1412 | 0 | 337 | Network trigger (fires then falls through) |
| `0x7d1` | club | `00476df0` | 888 | 0 | 245 | Club overview / summary (club_s) |
| `0x7d4` | club | `00415010` | 662 | 0 | 196 | Club honours / Awards (award) |
| `0x7d5` | club | `00454620` | 5694 | 0 | 1424 | Club Dashboard (verified anchor; draw 004551c0) |
| `0x7d6` | club | `00454620` | 5694 | 0 | 1424 | Club Dashboard (other-club / non-managed) |
| `0x7d7` | club | `00859250` | 3381 | 0 | 929 | Squad â€” Senior |
| `0x7d8` | club | `00859250` | 3381 | 0 | 929 | Squad â€” Reserves |
| `0x7d9` | club | `00859250` | 3381 | 0 | 929 | Squad list mode 2 |
| `0x7da` | club | `00859250` | 3381 | 0 | 929 | Squad list mode 3 (youth/U21) |
| `0x7db` | club | `00859250` | 3381 | 0 | 929 | Squad list mode 4 |
| `0x7dc` | club | `00859250` | 3381 | 0 | 929 | Staff list page 2 |
| `0x7dd` | club | `00859250` | 3381 | 0 | 929 | Staff list |
| `0x7de` | club | `00859250` | 3381 | 0 | 929 | Staff list variant 2 (goto LAB_0074caec with uVar12=2) |
| `0x7df` | club | `00859250` | 3381 | 0 | 929 | Staff list variant 3 |
| `0x7e0` | club | `00935f4b` | 1410 | 0 | 220 | Network â€” falls out of this dispatcher into 007e4940 |
| `0x7e4` | club | `007f0020` | 993 | 0 | 336 | unresolved (network item) |
| `0x7e5` | club | `00493e10` | 2429 | 0 | 1076 | Player-in-club profile |
| `0x7e6` | club | `0046ba80` | 2237 | 0 | 865 | club/player detail |
| `0x7e7` | club | `00859250` | 3381 | 0 | 929 | Squad â€” Senior (variant 2) |
| `2000` | club | `00493e10` | 2429 | 0 | 1076 | Player profile (via FUN_007cf040 guard) |
| `0x3e9` | global | `0076ffb0` | 1519 | 0 | 438 | News subscribe/read |
| `0x3eb` | global | `007f0020` | 993 | 0 | 336 | network item |
| `0x3ec` | global | `00859250` | 3381 | 0 | 929 | Player & Staff Search (staff.c) |
| `0x3ef` | global | `006986a0` | 99 | 0 | 27 | manager/job screen |
| `0x3f0` | global | `006547c0` | 14002 | 0 | 1743 | Return from holiday? confirm |
| `0x3f2` | global | `00698140` | 100 | 0 | 27 | manager sub-screen |
| `0x3f4` | global | `0058a550` | 1217 | 2084 | 486 | shared list/table view (FUN_00652ca0 param) |
| `0x3f5` | global | `0058a550` | 1217 | 2084 | 486 | shared list/table view |
| `0x3f6` | global | `0058a550` | 1217 | 2084 | 486 | shared list/table view |
| `0x3f7` | global | `0058a550` | 1217 | 2084 | 486 | shared list/table view (variant) |
| `0x3f8` | global | `0058a550` | 1217 | 2084 | 486 | shared list/table view (variant) |
| `0x3f9` | global | `0058a550` | 1217 | 2084 | 486 | shared list/table view |
| `0x3fa` | global | `0058cde0` | 1448 | 0 | 544 | unresolved (takes widget payload) |
| `0x3fb` | global | `007e7790` | 653 | 945 | 141 | Add Manager (network) or Add Manager (local) |
| `0x3fc` | global | `00808a70` | 282 | 0 | 111 | Player Waiting screen |
| `0x3fd` | global | `007e7c40` | 568 | 0 | 103 | return-from-holiday redistribute + Please Confirm |
| `0x3fe` | global | `00822940` | 172 | 0 | 21 | Save Game (Enter File Name) |
| `0x402` | global | `007ebaf0` | 510 | 0 | 34 | Exit Game confirm |
| `0x414` | global | `00771810` | 416 | 0 | 109 | unresolved (takes widget payload) |
| `0x415` | global | `008e3700` | 538 | 0 | 218 | tactic/team |
| `0x418` | global | `00700f20` | 950 | 0 | 325 | Match/Latest Scores (match.c) |
| `0x41c` | global | `006809d0` | 88 | 0 | 14 | manager (club) |
| `0x41d` | global | `006809d0` | 88 | 0 | 14 | manager (nation) |
| `0x41e` | global | `00695e60` | 1035 | 0 | 244 | manager sub-screen |
| `0x41f` | global | `006977b0` | 603 | 0 | 171 | Manager screen (club) |
| `0x420` | global | `006977b0` | 603 | 0 | 171 | Manager screen (nation) |
| `0x421` | global | `007e6570` | 6290 | 0 | 1186 | Chat / scrman.c |
| `0x424` | global | `00822940` | 172 | 0 | 21 | Save â€” Enter File Name dialog |
| `0x425` | global | `005276f0` | 15122 | 0 | 1397 | Please Confirm dialog (club control) |
| `0x429` | global | `005dc5e0` | 313 | 0 | 117 | unresolved |
| `0x42a` | global | `0080fac0` | 229 | 0 | 79 | game options |
| `0x42c` | global | `004fd1b0` | 203 | 0 | 63 | settings sub-screen |
| `0x42e` | global | `00693410` | 959 | 0 | 251 | manager sub-screen |
| `0x42f` | global | `006547c0` | 14002 | 0 | 1743 | Restart the game? confirm |
| `0x431` | global | `008053d0` | 2441 | 0 | 515 | Start New Game / Select League(s) (Setup.c) |
| `0x433` | global | `005276f0` | 15122 | 0 | 1397 | Please Confirm dialog (nation control) |
| `0x434` | global | `007dfac0` | 1126 | 0 | 249 | Scouting (scout.c) |

## Class B — asm walk required (0)

| cmd | scope | primary FUN_ | decomp .c | GDI asm | fn size | note |
|-----|-------|--------------|-----------|---------|---------|------|

## Class C — skipped (unbounded / no source) (2)

| cmd | scope | primary FUN_ | decomp .c | GDI asm | fn size | note |
|-----|-------|--------------|-----------|---------|---------|------|
| `0x3f3` | global | `004a2190` | 0 | 0 | 98 | FIFA rankings (already ported as Screen::FifaRankings â€” Layer 3 wire |
| `0x40c` | global | `004a28c0` | 0 | 0 | 63 | competition sub-screen |
