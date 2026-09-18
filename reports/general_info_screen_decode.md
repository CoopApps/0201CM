# General Info screen (top tab #4) — GDI decode

Captured 2026-09-18 from cm0102_GDI.exe, **Chester City** (a club the
player does NOT manage) — the default page when you first click the tab.
`fixtures/general_info_screen/` (general_info.json, structure.txt,
reference.png). 1086 primitive calls.

The manager-owned version shows MORE (a second pass, deferred). This
records the not-managed default.

## Layout

Shares the club chrome (sidebar, kit title bar, 5 top tabs with General
Info active, 5 bottom tabs, Back/Next). Screen-specific middle:

| Element | Rect | Notes |
| --- | --- | --- |
| View button | (110,125)-(235,145) | gray `0x4210` bevel + "View" (font 1) |
| Title band | (110,150)-(780,185) | darkened; "General Info" yellow, centred, font ? |
| **Detail panel** | (110,190)-(780,332) | darkened. 6 rows, 21px pitch from y=198 |
| **Non-Playing Staff panel** | (110,337)-(780,500) | darkened |

Detail rows — label col x 112..301 (font **3**, grey 0x739c), value col
x 303..778 (font **2**, yellow 0x7fe0), labels carry two leading spaces:

```
  Nation           England
  Status           Professional
  Finances         Ok
  Stadium          The Deva Stadium, Chester
  Facilities       6000 (2700 seated)
  Training Ground  Adequate facilities
```

Non-Playing Staff block:
- Header "Non-Playing Staff" at (115,345) font 3, orange `0x7e00`, with an
  orange underline PANEL (115,367)-(267,386).
- Staff rows from y=388, 21px pitch: name col x=112 (font 3, grey) + role
  col x=371 (font 2, yellow):
  ```
    Terry Smith      Chairman
    Gordon Hill      Manager
    Tony MacDonald   Coach
    Adam Powell      Scout
    Mark Mason       Scout
  ```
- Scrollbar over the staff panel only: up (759,345)-(778,364), track
  (759,365)-(778,472), thumb (759,365)-(778,448), down (759,473)-(778,492).
  → `Scrollbar::new(345, 492)`.

## Fonts (same split as Next Match)
Labels = font_idx 3 (body/arial_14); values = font_idx 2
(arial_narrow_11); buttons/tabs = font_idx 1 (arial_narrow_10).

## Data sources for the port
- Nation: `ClubView::nation_id` → nation name.
- Stadium: home stadium name; Facilities: `capacity_total (seated)`.
- Non-playing staff: club's chairman/manager + staff at the club by role.
- Status / Finances / Training Ground: coarse descriptors — approximate,
  labelled, until the exact club status/facility bytes are decoded.

## UNKNOWN / UNCAPTURED
- The manager-owned version (more fields) — deferred second pass.
- The View dropdown's menu contents.

## Update 2026-09-18 — View dropdown + full staff list + job mapping

**View dropdown** (opened via the View button): green dropdown,
container PANEL (110,148)-(235,190) c=0x0200, two items each 20px:
- `General Info` (112,150)-(233,168) — checked (leading tick + spaces)
- `Stats` (112,170)-(233,188)
These are the tab's TWO pages. Same green dropdown style as the squad
View menu.

**Full Non-Playing Staff list** (user scrolled the original):
```
Terry Smith     Chairman
Gordon Hill     Manager
Tony MacDonald  Coach
Adam Powell     Scout
Mark Mason      Scout
Gary Stevens    Physio
Dean Spink      Player/Assistant Manager
```
7 rows → the staff panel scrolls; scrollbar (759,345)-(778,492).

**club_job byte → role** (derived: known-role staff cross-referenced
against rust-db `PlayerView::club_job()`):
- 1 → Chairman
- 5 → Manager
- 10 → Physio
- Coach / Scout / (Player/)Assistant Manager bytes: NOT yet resolved —
  Chester's coach + scouts are absent from our rust-db (a DB-fidelity
  gap the user confirmed), so their bytes can't be read from this club.
  Needs samples from a club whose non-playing staff ARE in rust-db, or a
  decode of the job-classification function.

**Known DB gap:** rust-db surfaces only 3 non-playing staff for Chester
(Chairman, Manager, Physio) vs the original's 7. The coach, scouts and
player/assistant-manager are missing from our staff→club data. The
screen can be built now; the staff-completeness fix is separate.

## Update 2 — authoritative club_job → role mapping (from FUN_00524850)

Rather than reverse the mapping screen-by-screen, it is the exe's own
role-name switch `FUN_00524850(job, …)` (DirectDraw decompile
`00524850.c`), cases 0..0xd. Transcribed verbatim:

| job | role | job | role |
| --- | --- | --- | --- |
| 0 | Unemployed | 7 | Reserve Team Manager |
| 1 | Chairman | 8 | Coach |
| 2 | Managing Director | 9 | Scout |
| 3 | General Manager | 10 | Physio |
| 4 | Director of Football | 11 | Player |
| 5 | Manager | 12 | Player/Manager |
| 6 | Assistant Manager | 13 | Player/Assistant Manager |

Confirmed against rust-db Burnley (id 1604): job 1 Barry Kilby
(Chairman), job 5 Stan Ternent (Manager), job 6 Sam Ellis (Assistant
Manager — his real 2001 role), job 8 ×4 (coaches: Docherty/Jepson/
Pashley/Robson), job 9 ×2 (scouts: Roberts/Catlow), job 10 ×2 (physios).

The General Info list excludes ordinary players (job 11); every other
job renders its real role. (Whether Managing Director / General Manager
appear in THIS list or elsewhere is unconfirmed — Chester had none; a
Burnley capture would settle it. They currently show, since they are
non-playing staff.)

## RESOLVED: "L. Catlow" → "Liz Catlow" (common-name override)

The pool was not corrupt and the importer was not at fault. The Burnley
scout's disk record (staff id 52277, club 1604, job 9) carries THREE
name ids:

- `first_name_id  = 12731` → `first_names[12731]  = "L."`
- `second_name_id = 19094` → `second_names[19094] = "Catlow"`
- `common_name_id = 1584`  → `common_names[1584]  = "Liz Catlow"`

When `common_name_id` is set (non-zero) the exe renders the common name
verbatim as the full known-as name — the same mechanism that shows
players as "Ronaldo" instead of "Ronaldo Nazário". Our resolver was
composing `first second` and ignoring the override, so it produced
"L. Catlow".

Fix: `World::person_display_name` (crates/cm-domain/src/lib.rs) now
returns the common name when set and non-empty, falling back to
`first second` otherwise. `next_match::person_full_name` and
`general_info_for` both route through it, so every player-visible name
honours the override. Regression test:
`general_info::common_name_tests::liz_catlow_uses_common_name` asserts
staff 52277 resolves to exactly "Liz Catlow".

## Stats page (View → Stats) — captured 2026-09-18 (GDI, Brighton)

Capture: `fixtures/general_info_screen/stats.json` (1051 calls). Club =
Brighton & Hove Albion (id 1507), Second Division, not managed. View
dropdown was left open, which re-confirms the bevelled container
(style 0x130 = P_SOLID_FILL|P_BEVEL, col 512) and the two flat rows
(112,150)-(233,168) col 512 / (112,170)-(233,188) col 576 hover.

### Frame (identical to General Info page)
- View button (110,125)-(235,145) grey 0x4210.
- Title band (110,190? no) — title band (110,150)-(780,185) P_DARKEN,
  text " Stats" font 3 yellow 0x7fe0, centred.
- ONE darkened content panel (110,190)-(780,500) P_DARKEN (no staff
  panel — that is General-Info-only).

### Rows — 16 fixed label/value pairs
Label col: panel+text at x=112, font 2, grey 0x739c (29596), 2 leading
spaces. Value col: x=446, font 2, yellow 0x7fe0 (32736). Each row is two
style-1 cells (112..444) + (446..778), height 16. Row tops (stride ~17.4,
alternating 17/18): 198,216,233,251,268,285,303,320,337,355,372,389,
407,424,441,459.

Labels (top→bottom) with Brighton values:
1.  Number Of Players — 26
2.  Number Of Players Injured — 0
3.  Average Age - First Team — -
4.  Average Age - Squad — 24.46
5.  Total Wage Bill - First Team (p/w) — £0
6.  Total Wage Bill - Squad (p/w) — £18.5K
7.  Average Wage (p/w) — £700
8.  Highest Wage (p/w) — £2K - Dirk Lehmann
9.  Lowest Wage (p/w) — £150 - Darren Trigg
10. Oldest Player — 36 - Paul Rogers
11. Youngest Player — 16 - Chris McPhee
12. Highest Valued Player — £350,000 - Bobby Zamora
13. Number Of Current International Players — 0
14. Number Of Current Under 21 Players — 0
15. Number Of Foreign Players — 2
16. Number Of Non-EU Players — 0

### CRITICAL: values are RUNTIME, not raw DB
The raw type6 fields do NOT reproduce these: shipped wage sum for the 26
players is £6,475 (capture: £18.5K), max raw value is Geoff Pitcher
£200K (capture: Zamora £350K), and youths carry dob_year=1900 sentinels
(capture: McPhee age 16). The exe computes wages/values/ages at boot
(valuation + contract pool + regen). The Stats page therefore aggregates
the SAME runtime feed the Squad screen uses:
- wage/value: `world.contracts.contract_for_staff(id)` → wage/value,
  falling back to `PlayerView::wage()/value()`.
- age: `DomainStaffType6::age_at(2001, day_of_year(2001,8,10))`.
This keeps Stats consistent with the Squad screen by construction.

### Formatting
Two money formats: wages use K/M abbreviation (£150, £700, £2K, £18.5K);
the Highest-Valued value uses full comma grouping (£350,000). "£X - Name"
for the four attributed rows. £ is CP1252 0xA3.

### Rows needing definition decode (not fabricated)
- First-Team avg age / wage bill: "-" / "£0" in the not-managed view
  (no selected XI). Wire to the selected first team once managed.
- Current International / Under-21 counts: current call-ups, not
  ever-capped. 0 for this Div-2 club; definition to confirm before
  trusting non-zero clubs.
- Non-EU: needs the per-nation EU/work-permit flag (no `is_eu` field in
  nations.json). 0 here (Lehmann = German = EU).
