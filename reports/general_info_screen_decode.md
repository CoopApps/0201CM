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
