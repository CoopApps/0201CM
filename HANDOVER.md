# CM0102-RS — Project Handover

A from-scratch **Rust re-creation of Championship Manager 01/02** (`cm0102.exe`),
backed by a native database imported once from a real CM install. The north star:
**a 100% code-authentic port** — every behaviour and pixel traces to a specific
decompiled function/address in the original, never measured-by-eye or invented.

- **Repo:** `D:\cm0102-rs` · GitHub: `github.com/CoopApps/0201CM` (branch `main`)
- **Original game install:** `D:\cm0102\` (exe + `Data\*.dat` + `Data\*.fnt` fonts)
- **Reverse-engineering source of truth:** `D:\cm0102-carve\` (Ghidra decompile of all
  9155 functions under `ghidra_out\cm0102.exe\decompiled\`, plus `strings.json`,
  `functions.json`, `xrefs.json`, `data_symbols.json`)
- **Python:** `D:\Python312\python.exe` (capstone + unicorn available)
- **Build toolchain:** Windows GNU Rust; prepend `D:\msys64\mingw64\bin` to PATH.

---

## 1. The guiding principles (read these first)

1. **The simulation is the priority; the UI is a delivery detail.** Don't rabbit-hole
   on rendering pixels while the engine underneath is stubs.
2. **No heuristics, no approximations.** "Computed at runtime" ≠ unliftable — lift the
   computing code. Every screen coordinate, colour, formula, and decision must trace to
   the exe. When something can't be fully lifted yet, say so honestly and stub it with a
   named pointer to its `FUN_` address, rather than faking it.
3. **The master database is LOCKED.** A new game is an in-memory instance built from it;
   it only becomes a file on explicit save. Once a half-day ticks, the game is dirty and
   can't quit without saving (mirrors the exe).
4. **Data is final.** No membership caps or invented rules — the shipped world is
   authoritative (e.g. which leagues are manageable is a data property, not a heuristic).

---

## 2. How to build & run

```bash
# PATH for the GNU toolchain (once per shell)
export PATH="/d/msys64/mingw64/bin:$PATH"

# Build the app (package name is "app", in crates/cm-ui-app)
cargo build -p app

# ALWAYS kill a running instance before rebuilding — it locks the exe:
taskkill -F -IM app.exe 2>/dev/null; rm -f target/debug/app.exe

# Run the game (normal flow: Setup -> Select Leagues -> Enter Name -> Select Club -> News)
./target/debug/app.exe

# Dev shortcut — boot straight to the in-game News page (Arsenal / "Fergie"):
CM_BOOT=dashboard ./target/debug/app.exe

# Headless screen render to a PPM (for verification without a window):
./target/debug/app.exe --dump <out.ppm> <which>
#   which = setup | leagues | season | club | dashboard | news | spec:<va>
#   CM_MENU_OPEN=<n> opens sidebar menu n; CM_ADV_DAYS=<n> advances the tick first
# Convert PPM->PNG: D:/Python312/python.exe -c "from PIL import Image; Image.open('x.ppm').save('x.png')"
```

**Gotchas:** transient `rustc` ICEs — just retry. Stale `.git/index.lock` — `rm -f` it.
The domain crate is large (~19k lines) so a clean build of it is slow (~1–2 min).

---

## 3. Repo layout (Rust workspace, `crates/`)

| crate | role |
|---|---|
| **cm-domain** | The heart. ~19k lines: world model, typed records, new-game init, player seeding, the day-tick, match pipeline, news/menu view-models, save/overlay model. Start here. |
| **cm-db** | Reads the native `rust-db` database into a `World`. `Database::open`, `ClubView`/`NationView`/`PlayerView` typed accessors. |
| **cm-import** | One-time importer: reads an original install's `Data\*.dat` → regenerates `rust-db` (155 MB JSON, 22 tables) + `rng_table.bin` + config. Also holds verify/check bins. The game never touches `.dat` after this. |
| **cm-rng** | Bit-exact port of the game RNG (MSVC LCG + the 51000-entry ring buffer `FUN_008fc4f0`). |
| **cm-render** | Pixel layer: `Surface`, `draw_panel`/`fill_rect`, `.fnt` font decode + `draw_text_box`, `layout::rebuild_layout` (ports the exe's area layout engine). |
| **cm-widget** | **The spec evaluator.** Renders carved screen specs (`analysis/screens/<va>.json`) to pixels via a single shared evaluator. Handles area/item/sidebar/nav_bar/menu_list kinds, palette, the 0x800 selection highlight. **This is the intended path for all screens** (see §7). Has a signed-contract fidelity gate. |
| **cm-ui-app** | The runnable app (package name **`app`**). winit window, screen state machine, the hand-wired screens, menu bar, day-tick wiring, dev shortcuts, `--dump`. **This is the crate edited most this session.** |
| cm-app, cm-data, cm-events, cm-gui | Earlier/auxiliary crates; the live app is cm-ui-app. |

**External data (gitignored, live on disk):** `rust-db/` (the native DB), `assets/`
(copyrighted fonts/images), `saves/`, `reports/` (analysis outputs).

---

## 4. What works end-to-end today

The full new-game flow is playable:

**Setup → Select Leagues → (Select Start Season) → Enter Name → Select Club → News page**,
then **Continue** advances the game.

- **New-game init** (`FUN_005121a0`/`FUN_008120d0` ported): loads the whole world from
  `rust-db` in-memory, builds fixtures for foreground leagues (real double round-robin,
  no caps), seeds players (`FUN_0051f5d0` deterministic core: CA / resolved PA /
  condition / morale).
- **League tier model:** per-nation 3-state bitfield (`nation+0x11c`: neither /
  background / foreground). Whole world loads regardless; tier only gates manageability
  + sim detail. Foreground promotion is a runtime flag flip.
- **Manager model (multi-human):** `HumanManager { identity, club, nation }`; multiple
  humans, hotseat via `active_human` (= `DAT_00b5d016`). Take Control installs at a club
  (`install_manager_at_club`, ports `FUN_00810f50` + the nation tier promote); Resign
  vacates but stays unemployed.
- **The day tick** (`tick_cm_phase`, port of the driver `FUN_005b6a90`): 3 phases/day,
  rolls the calendar (`CmPackedDate::add_days` ports `FUN_00536190`), and on phase 2
  plays that day's due fixtures through a match pipeline that writes scores + standings +
  news events. Verified: advancing 30 days from a fresh Arsenal game moves the date,
  plays fixtures, and drops Arsenal 1st→10th. **Continue** (Enter/Space, or the sidebar
  item) drives it and dirties the game.
- **News page = the home screen** (the exe's `news.c`, callback `0x00770170`): titled
  "\<Manager\> News", the four filter tabs (All/Messages/Competitions/Injuries and Bans)
  as a contiguous blue tab strip, dated headline list ("Tue 7th Aug EVE"), Filter +
  Next Unread, headline+body, bottom tabs, Back/Next. Fed by `save.pending_events`.
- **Persistent menu bar** (`game_mbr` / `FUN_00745540`, the 89px left sidebar): date,
  ◀▶ nav, Continue Game, [Manager name]=Manager Options, Competitions, Nations & Clubs,
  Find, Game Options. Real command codes; drop-downs render yellow items; clicking
  dispatches (Continue/Squad/Add Manager/Resign/Exit wired, the rest show a
  "not-yet-implemented (cmd 0x…)" status naming their exe target).
- **Squad/Information screen** (the old "dashboard"): club squad overview, reachable via
  the "Squad" menu item (`0x7d5`).

---

## 5. Key decoded facts (all in memory + reports)

Persistent memory lives in `d:\claudetemp\projects\D--cm0102-rs\memory\` (indexed by
`MEMORY.md`). The load-bearing ones:

- **Record layouts** — verified field maps for staff (157B), clubs, nations,
  competitions/club_comp (nation@0x5d, reputation@0x69, three-letter@0x53). Selection
  flags are on the NATION record `+0x11c`.
- **Game tick** — the driver is `FUN_005b6a90`; `FUN_005246e0` is AGE (not condition);
  `FUN_00536190` is calendar add-days.
- **Menu/command tree** — distributed dispatch (no central switch); 2 shared routers
  (`FUN_007491e0` global, `FUN_0074bf60` club); menu bar `FUN_00745540`; club toolbar
  `FUN_00487210`/`FUN_00487670`. Full doc: `reports/menu_tree_scope.md`.
- **News screen geometry** — exact widget coords via emulation (`tools/capture_news.py`);
  tabs = `FUN_005d7070`, Back/Next = `FUN_005d75b0` (cols [3,1]), sidebar = `FUN_00745540`
  mode 4. Doc: `reports/news_screen`... (see memory `news-screen-geometry`).
- **News generation** — **how news is decided** (traced this session): weekly per-club
  predicate cascade `FUN_0067ce90` reads form vs expected position + streaks +
  reputation, RNG-gated, picks a type-code; formatter switch `FUN_00733610` (gated on
  record[0]==0xbdf) maps code→template. Catalogue of **4,849 templates** in
  `reports/news_templates.{md,json}`; decision logic in `reports/news_generation_logic.md`.
- **.sav format**, **player init decode**, **base-game data model**, **emulation
  pipeline** — see the corresponding memory files.

---

## 6. Tooling (no Ghidra needed to use these)

Under `D:\cm0102-rs\tools\` (Python):
- **`capture_construct.py` / `capture_news.py`** — **emulate a screen's real construct
  callback in Unicorn** to capture every widget's exact resolved geometry. This is the
  authoritative way to get screen coordinates (the binary computing its own values). The
  pattern generalises to any screen: set START/END + helper returns, hook the two
  constructors (item `0x549580`, area `0x549790`).
- `emu.py` — runs the exe's layout function in Unicorn (validated).
- Screen specs are pre-carved to `D:\cm0102-carve\analysis\screens\<va>.json` (98
  screens); `cm-widget` evaluates them.
- Disassembly recipe (used throughout): capstone with `va2off` where `RVA = VA - 0x400000`.

`D:\completed ai projects\structural_carver\` is a separate, from-scratch structural
carver (no Ghidra) — retro-game oriented; a handover to extend it with PE32 + 32-bit
x86 support is at `D:\cm0102-carve\PE32_CARVER_HANDOVER.md` (not yet implemented).

---

## 7. The architecture we're converging on (and the anti-pattern to avoid)

**Target pipeline:** `game state → view-model → carved ScreenSpec → cm-widget evaluator
→ pixels`. The evaluator already renders 98 screens faithfully and centralises the hard
parts (flags, palette, selection highlight, scrollbars).

**Anti-pattern (actively being retired):** hand-writing a per-screen Rust render
function with coordinates/colours reconstructed by eye. It re-introduces bugs the
evaluator already solved. The News and Squad screens are currently hand-written (with
*emulated* exact coords, so they're faithful) but the intended next step is to make them
spec-driven. New screens should go **straight to the spec-evaluator path**: load the
carved spec + write a ~50-line `StateProvider`, and extend the evaluator once where a
dynamic widget kind (tab_list, list_body) isn't yet handled — a fix that benefits all
screens.

---

## 8. Known stubs / limitations (be honest about these)

- **Match engine** is a pipeline that produces scores + a match report, but it's not the
  real match engine; results are placeholder-quality and the report body leaks internal
  markers (e.g. `(0x1f93)`).
- **Tick subsystems** dispatched each phase — finances, transfers, training, injuries,
  AI team selection, board confidence — are **named stubs** in the correct order, not
  ported bodies.
- **News** uses invented headlines ("produces 2 goal match report"); the *real*
  generator + templates are now fully decoded (§5) but **not yet wired in**. All match
  news is stamped EVE (phase 2) because fixtures only resolve in that phase.
- **News/Squad screens** are hand-rendered (faithful coords, but not yet spec-driven).
- **Sidebar ◀▶ arrows** are triangle stand-ins; pixel-exact needs a `Data\game.mbr` blit.

---

## 9. Recommended next steps (in priority order)

1. **Port the news generator** — lift `FUN_0067ce90`'s predicate cascade (form vs
   expected position + streaks + reputation, RNG-gated, weekly) → emit type-code → the
   `code→template` table, feeding `save.pending_events`. Everything needed is decoded
   (`reports/news_generation_logic.md` + `reports/news_templates.json`). This makes the
   news authentic and exercises real standings data.
2. **Fill a high-value tick subsystem** — the match engine itself, or transfers/finances,
   built headless with tests diffed against the original's behaviour.
3. **Move screens onto the spec evaluator** — add `tab_list`/`list_body` kinds to
   cm-widget, convert News + build a spec-driven Squad screen with a provider (§7).
4. **(Optional) PE32 carver** — implement `D:\cm0102-carve\PE32_CARVER_HANDOVER.md`.

---

## 10. Session workflow notes

- Commit/push are done only on request; commits are co-authored. `reports/` and
  `rust-db/` are gitignored (large/derived) so they stay local.
- Persistent memory (`d:\claudetemp\projects\D--cm0102-rs\memory\`) is the durable
  knowledge base — update it when you decode something non-obvious; verify a memory's
  file/function references against current code before relying on them.
- Verify screens headlessly with `--dump` + PPM→PNG rather than opening a window each
  time.
