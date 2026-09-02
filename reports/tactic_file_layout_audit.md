# Tactic File Layout Audit

## Full 1476-byte `.tct` layout (v5E)

Every byte in a `WWW2 Hard Tackling.tct` file cross-referenced to a field
or an unknown region:

| Offset (hex) | Size | Field / status | Evidence |
|--:|--:|:--|:--|
| `0x0000` | 4 | `version` | Bit-inverted `.pct`, direct `.tct` |
| `0x0004` | 50 | `formation_name` (bit-inverted ASCII) | verified |
| `0x0036` | 3 | **unknown** | probably padding / preset flag |
| `0x0039` | 64 | `author` (ASCII) | verified |
| `0x0079` | 133 | **unknown** — see §Unknown Region A | probably setpiece-taker player-ids + preset metadata |
| `0x00FE` | 4 | `team_flags_1` | **byte 2 = mentality** (0x1F/0x15/0x9B verified) |
| `0x0102` | 22 | **NEW: `per_slot_instr[11]` (u16 each)** | verified WWW2 anchor tokens |
| `0x0118` | 11 | `aux_role[11]` (v5C+) | verified |
| `0x0123` | 22 | `depth[11]` (2 bytes each) | verified |
| `0x0139` | 1056 | `positional_grid[11]` (96B each = 2×3×4 × 4-byte PitchXY) | X=lateral, Y=depth (verified) |
| `0x0559` | 4 | `team_flags_2` | passing (fixed to Mixed/Short reversed), counter (fixed to bit 7), offside verified |
| `0x055D` | 88 | `slot_pair[11]` × 8 bytes (v5E) | movement_token nibbles decoded |
| `0x05B5` | 11 | `flag[11]` (v5E) | 3-state position category (verified) |
| `0x05C0` | 4 | **unknown tail** | value `f2 ff ff f2` — possibly checksum |

## Unknown Region A (0x0079..0x00FE — 133 bytes)

Structure visible in hex dump:
```
0079: ff ff d6 00 00 00 02 01 00 00 ee ea a1 01 6a 4d
0089: a2 01 00 00 d4 07 79 00 01 d4 07 79 00 01 01 ff
0099: ff ff 33 e8 60 00 00 00 84 6e 30 02 d3 07 00 00
00a9: 00 00 29 00 02 ff d4 cb 31 02 69 00 00 00 00 00
00b9: 00 00 00 00 00 00 ff ff ff ff ff ff ff ff ff ff
00c9: ff ff ff ff d8 06 00 00 2e 01 00 00 68 5d af 01
00d9: e6 af a2 01 00 00 d4 07 79 00 01 d4 07 79 00 01
00e9: 01 ff ff ff 33 e8 60 00 00 00 84 6e 30 02 d3 07
00f9: 00 00 00 00 29 81 aa 8e 10 01 00 08 08 88 00 04
```

The repeated 5-byte pattern `d4 07 79 00 01` at 0x0089/0x008E/0x00DD/
0x00E3 looks like per-slot spinner-selection records. `d4 07 = 2004`
plausible as year, `79 = 121` as player index? The two-block symmetric
structure (bytes at 0x0089..0x00A9 mirror bytes at 0x00DD..0x00F9)
suggests two teams stored — probably WITH-BALL and WITHOUT-BALL
tactic snapshots, or main + reserve tactic.

## Runtime struct layout (from FUN_00890b30 copy fn)

The runtime tactic struct is DIFFERENT from the file layout. `FUN_00890b30`
in the exe is a tactic-struct copy fn. Field offsets it accesses:

- `+0x00`, `+0x04` (u32 each) — 2 u32 header words
- `+0x08`, `+0x0A` (u16 each) — 2 u16 headers
- `+0x0C..0x16` — 5 × u16 array (10 bytes)  
- `+0x16..0x20` — 5 × u16 array (10 bytes)
- `+0x20..0x2A` — 5 × u16 array (10 bytes)
- `+0x2A..0x34` — 5 × u16 array (10 bytes)
- `+0x34`, `+0x38`, `+0x3C` (u32 each) — 3 u32 fields
- `+0x40..0x4C` — 3 × u32 array (12 bytes)
- `+0x4D` (u8), `+0x4D..0x53` (3 × u16) — small mixed fields
- `+0x53` (u32) — 1 u32
- `+0x57..0x11F` — 50 × u32 array (200 bytes)
- `+0x122` — start of 2 × 0x625-byte per-team-snapshot blocks

The 4 consecutive 5×u16 arrays at offsets 0x0C, 0x16, 0x20, 0x2A are
consistent with the 5 team-instruction spinners on the Team Instructions
dialog (Playmaker, Free Kicks L, Free Kicks R, Corners L, Corners R) —
each stored in 4 different context arrays (with-ball, without-ball,
main tactic, reserve tactic?).

## What's decoded

| Piece | Status | Evidence |
|:--|:--|:--|
| formation_name / author | verified | direct ASCII decode |
| team_flags_1 mentality byte | verified | 352 preset triple |
| team_flags_2 passing | fixed | WWW2 screenshot |
| team_flags_2 counter | fixed | WWW2 screenshot |
| team_flags_2 offside | verified | WWW2 screenshot |
| team_flags_2 mentality | verified | WWW2 screenshot |
| positional_grid axis semantics | verified | 352 attacking waypoint diff |
| positional_grid → SlotInstructions | verified | parse + iter |
| slot_pair.movement_token nibbles | 5-of-8 verified | 40-preset variance + 442 diffs |
| slot.flag position category | verified | 6-preset sweep |
| slot.per_slot_instr u16 | struct field only | WWW2 anchor values but individual bits TBD |

## What's still to decode

| Field | Where | Blocker |
|:--|:--|:--|
| team_flags_2 pressing/marking/tackling bit-to-label | tf2 bits 11-17 | screenshot pair needed |
| team_flags_2 men_behind_ball bit location | tf2 bits 18-19? | screenshot pair needed |
| per_slot_instr u16 individual bit mapping | per-slot 16 bits | controlled author-then-diff |
| Unknown Region A (133 bytes) | 0x0079..0x00FE | Playmaker/FK/Corner spinner selections |
| Unknown tail (4 bytes) | 0x05C0..0x05C4 | value pattern `f2 ff ff f2` — checksum? |

## Decoding methodology for remaining fields

To pin individual bit meanings without further screenshot evidence:

1. **Find the exe SETTER cluster**: the tactics-editor dialog is
   FUN_00xxxxxx (unidentified). When the user clicks "Cross Ball = Yes"
   the click handler writes to a specific bit in the runtime tactic
   struct. Finding this fn gives the byte offset + bit mask directly.
2. **Search for readers via string xrefs**: the 8 slider-name strings
   at 0x006779ed are UI labels. Their code refs (currently 0x886970,
   0x892cd2 for "Cross Ball") point to the render/setter fns.
3. **Compare shipped preset pairs**: only 40 `.pct` files ship — 4
   pairs found with 1-3 nibble diffs so far (yielded Forward Runs,
   Marking, Closing Down). More narrowly-differing pairs unlikely to
   surface without controlled author-then-diff.

## Session verification anchors

Every verified decoder has a Rust test using exact byte values from
shipped `.pct` files or the user's WWW2 `.tct`. See:
- `crates/cm-domain/src/tactic_file.rs` tests module
- `preset_token_constants_have_expected_nibble_patterns`
- `team_flags_1_mentality_byte_extracts_byte_2`
- `same_lateral_layout_verified_from_352_diff`
- `forward_runs_nibble_decode_from_442_striker_diff`
- `marking_closing_nibbles_from_442_defensive_midfielder_diff`
- `team_settings_www2_hard_tackling_ground_truth`
- `per_slot_instr_populated_from_www2_bytes`
