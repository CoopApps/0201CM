# How a new game is created: cm0102.exe vs cm0102-rs

Everything below about the exe was read out of the decompile at
`D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`. Function addresses are
given so any claim can be re-checked.

---

## 1. The original's flow

### Before Next

| Step | Function | What it does |
|---|---|---|
| Enter Select League(s) | `FUN_008053d0` | If pools aren't loaded, calls `FUN_0050e9b0` → `FUN_005121a0` |
| Load static pools | `FUN_005121a0` | "Loading database": mallocs and fills every pool from `index.dat` — clubs (581 B, `DAT_00acd5bc`), nations (290 B, `DAT_00acd5b0`), staff (`DAT_00acd5c4`), stadiums, competitions (107 B, `DAT_00acd5d8`), schedule templates |
| Seed default selection | `FUN_00811d80` | Zeroes every nation's flags byte at `+0x11c`, then sets the language-default nation to `2` (foreground). Sets counters `DAT_00acdf00` (selected), `DAT_00acdf04` (foreground), `DAT_00acdf08` (primary nation) |
| Build picker table | `FUN_006508e0` | Builds `DAT_00b4bc70`, one 0x48-byte entry per selectable league: nation pointer, per-league season start/end dates, handler label |
| User toggles a row | handler at `0x00806640` | Writes `2` (foreground) or `1` (background) into `nation + 0x11c` |
| Real Players Yes/No | events `0x39` / `0x3a` | `mov byte ptr [0x9a2051], 1` / `, 0` |
| Attribute Masking Yes/No | events `0x3b` / `0x3c` | `mov byte ptr [0x9b88a8], 1` / `, 0` |

**The selection flags live on the NATION record, not the competition record.**
That was a genuine correction during this work — the picker lists countries,
and its slot table holds nation pointers. `FUN_00811140` proves it by clearing
the byte while stepping the nation pool at its 0x122 stride. Bits: `1` =
background, `2` = foreground/playable, `4` = nation active.

### After Next — `FUN_008120d0`, "Initialising game data"

Runs 0x2c progress steps. In order:

1. **Schedule index** (`FUN_005232a0`) — per-competition fixture-date template index.
2. **Compact club-comp pool** (`FUN_00523630`) — physically drops records whose
   competition isn't selected. The first place background data is discarded.
3. **Division rebalance** (`FUN_00822cd0`) — forces affected divisions to exactly
   14 clubs, promoting/demoting to fit.
4. **Date normalisation** (`FUN_0081a020`) — reconciles the chosen start date
   against the foreground league's entry in the picker table.
5. **Runtime parallel arrays** — `DAT_00acdf0c` (0x4f bytes per staff member:
   retirement age, decline curve, career-history slots), `DAT_00acdf14`
   (6 bytes per club), `DAT_00acdf1c` (4 bytes per club).
6. **Squad backlinks** — for each club, walk its 50 squad slots at `club+0xd7`
   and set `player+0x39 = club`.
7. **Player initialisation** (`FUN_0051f5d0`) — 2199 lines, ~200 RNG calls:
   derives each person's starting form, condition, morale and reputation as of
   the start date.
8. **Fictionalisation** (`FUN_0051c970`) — runs **only when Real Players is No**.
   Replaces name pointers with generated names and scrambles international caps
   and goals.
9. **~35 subsystem constructors** — tactics, human managers, scouts, news,
   friendlies, UEFA and CONMEBOL competitions, staff and club records, match
   stats, player stats, discipline, per-nation rule engines, awards, injuries,
   national teams, squad manager, **fixture manager** (`FUN_00594370`, seeded
   with the chosen start year), finance, contract manager, transfer manager,
   regens, AI managers, **fog-of-war** (`FUN_00599cb0`), simulated stats,
   training, name generator.
10. **Club finance defaults** — per club: own-stadium flag, random attendance
    where unset, seating/capacity/balance derived from attendance² with
    league-dependent divisors.
11. **One priming tick** — calls the day-tick driver `FUN_005b6a90` once, then
    sets the game-running flag `DAT_00dbc3f4 = 1`.

### After init

- `FUN_00809ad0` creates the human manager's staff record (born 1 Jan 1966,
  `staff+0x5f = 5`) in a free slot at the tail of the staff pool.
- `FUN_00810f50` installs that manager at the chosen club: sacks the incumbent
  AI manager, sets wage from attendance (`attendance/10 + 3000`), sets manager
  reputation to 20, clears the club's AI-managed bit.

### Attribute masking

Never applied at initialisation — **attributes are never altered**. It is
enforced at *display* time: `FUN_0052cdd0` is the "is this attribute visible?"
predicate. It returns visible if masking is off or the fog-of-war object
`DAT_00acdc60` doesn't exist; otherwise `FUN_0059a970` computes a knowledge
level 0–3 (3 = full: same club, same division, or shortlisted) and only a
deterministic subset of attributes is revealed per level.

---

## 2. The Rust version's flow

| Step | Rust | Status |
|---|---|---|
| Load static data | `cm_db::Database::open` → `World::read_rust_db_dir` | **Done.** Reads `rust-db/`, never touches `.dat`. All 22 tables verified against shipping counts |
| Picker state | `SelectLeaguesState` in `cm-ui-app` | **Done.** Per-slot `selected` / `background_marker`, plus `use_real_players` / `attribute_masking` |
| Next pressed | `create_new_game` in `cm-ui-app/src/main.rs` | **Done.** Builds `NewGameOptions` and calls the domain |
| Initialise game | `World::new_game_from_rust_db` | **Partial** — see below |
| Persist | `RuntimeSaveGame::write_json_file` | **Done.** Native JSON save, no `.sav` container |

`new_game_from_rust_db` currently:

- resolves picker labels to nation ids (`nation_ids_for_picker_labels`),
- resolves those nations to competition ids (`competition_ids_for_nations`),
- restricts the season's fixtures, standings and generation proofs to them,
- sets the start date from the chosen season,
- records the options in the save so nothing about the player's choices is lost.

Verified against the real database: selecting England + Italy + Spain resolves
to nation ids `{60, 94, 171}` and 36 competitions, produces 570 fixtures across
three English divisions with 47 standings rows, and ticks 7 days playing 210
fixtures.

---

## 3. Honest gap list

Ordered by how much each blocks a faithful game.

1. **Competition → nation links are empty in the shipped data.** The field is
   decoded (`comp+0x5d`) but every record in `club_comp.dat` has it zero; the
   exe wires comps to nations during init, reachable through clubs
   (`club+0x53` = nation, `club+0x57/0x5b/0x60` = comps). Until that wiring pass
   is ported, `competition_ids_for_nations` falls back to matching the
   competition name against the nation name or its adjective ("English …" for
   England). The link field is consulted first, so this upgrades for free once
   the wiring lands.
2. **No club/player activation pass.** The exe compacts the club-comp pool
   (`FUN_00523630`) and rebalances divisions (`FUN_00822cd0`). We keep every
   club and player loaded and only filter fixtures.
3. **No per-staff or per-club runtime arrays.** The exe's `0x4f`-byte staff
   array (retirement age, decline) and `6`-byte club array have no Rust
   equivalent yet.
4. **No player initialisation pass.** `FUN_0051f5d0`'s form/condition/morale/
   reputation derivation is not ported, so people start with no derived state.
5. **Real Players = No does nothing.** `FUN_0051c970`'s fictionalisation
   (generated names, scrambled caps) is recorded in the options but not applied.
6. **Attribute masking does nothing.** Correctly a *display-time* concern, so it
   needs the fog-of-war knowledge model (`FUN_0059a970`) before the UI can
   honour it.
7. **Subsystems are a ledger, not an implementation.** The save records the ~35
   subsystems and their frontier status; the constructors themselves are not
   ported.
8. **No human manager.** `FUN_00809ad0` / `FUN_00810f50` (create manager record,
   install at club) have no Rust equivalent, so there is no "you" in the game.

---

## 4. Deliberate differences

These are not gaps — they are places where the port intentionally diverges.

- **Native database instead of `.dat` at runtime.** The original re-reads and
  re-swizzles `.dat` files on every start. We import once (`cm-import`) into
  `rust-db/` and the game opens that. The `.dat` files are only an import source.
- **IDs instead of pointers.** The exe swizzles ids into raw pointers at load
  (`FUN_0051b110`) and back on save (`FUN_005176c0`). Rust keeps ids and
  resolves by index, so there is no swizzle pass and no pointer-fixup bugs.
- **JSON save instead of the `.sav` container.** The original writes a section
  directory whose payloads are copies of the pool layouts. Ours is a typed,
  diffable snapshot that carries its own provenance notes.
- **Selection flags are not stored in a spare record byte.** The exe hides them
  at `nation+0x11c` inside the on-disk record (always `0` on disk, runtime-only).
  We keep them in `NewGameOptions`, which is explicit and serialisable.
