# Screen structure + Back/history mechanism — decode

**Date:** 2026-09-18. Traced from the DirectDraw decompile
(`D:\cm0102-carve\ghidra_out\cm0102.exe\decompiled\`, files named
`<va>.c`); the GDI build has no decompiled source but the mechanism is
identical. VAs below are DirectDraw. Module string:
`C:\dev\CM3_00_01\si_code\scrman.*` ("screen manager").

We only need **observational equivalence** — do what it does, not byte
copy. This records what "what it does" is.

## 1. Screens are {generic frame + pluggable content}

The chrome is **shared builders**, not redrawn per screen:

| Chrome piece | Builder |
| --- | --- |
| 89px sidebar + ◀▶ history arrows + menu | `FUN_00745540` — arrows are buttons with command **-2** / **-3** |
| Back / Next bar | `FUN_005d75b0` — also commands **-2** / **-3** |
| Top / bottom tab strips | `FUN_005d7070` |

Each screen registers a **draw callback + event callback + leave
callback** with the screen manager:

```
FUN_007e6570(draw_cb, event_cb, arg, leave_cb, 0)
```

e.g. the club dashboard (`00454620.c:77`):
`FUN_007e6570(&LAB_004551c0, &LAB_00468d50, 0, FUN_0046b9b0, 0)`.
News registers `0x00770170` via `FUN_0076ffb0`.

A generic pump (`FUN_007e5bd0`) invokes the current node's draw callback
(`node+0`) and routes input to its event callback (`node+8`). Screen
bodies emit widgets (`FUN_00549580`) / areas (`FUN_00549790`) into a
pool the grid layout engine positions — bodies do not hardcode pixel
coordinates.

**Rust mapping:** this is exactly the generic-frame + pluggable-body
design. A screen = `{ draw(surface, state), handle_event(click) -> Nav }`.
The frame draws shared chrome; the body is the screen's own draw. No
widget-pool replica needed — just the same *shape*.

## 2. The Back / history stack

**A per-seat, per-page doubly-linked list of screen nodes** inside the
scrman object. "Page" = top-tab (16 of them); each page has its own
browser-style history list.

| Field | Meaning |
| --- | --- |
| `scrman+0x3056` | current page index (0..15) |
| `page = scrman + idx*0x300`; `page+0x06/+0x0a/+0x0e/+0x14` | head / tail / current / count |
| `node+0x1f4` / `node+0x1f8` | prev / next |

* **Push (navigate):** `FUN_007e6570` appends a node at the tail and
  makes it current — **with de-dup**: revisiting a screen already in the
  list just re-points `current`, it does not grow the stack
  (`007e6570.c:44-61`).
* **Pop (Back / ◀):** a screen's event handler returns **-9**; the pump
  walks `current = current.prev`, drops the tail, frees it
  (`007e5bd0.c:137-160`).
* **Enable gates:** Back (cmd -2) enabled iff `current.prev != 0`
  (`FUN_007e6b60`); Next (cmd -3) enabled iff `current.next != 0`
  (`FUN_007e6ab0`).

### It is NOT reset per phase (AM/PM/EVE)

The two clears are `FUN_007e6e00` (one page) and `FUN_007e4750` (all 16
pages). Neither is on the Continue/advance path. **Continue** (menu cmd
1000, `007491e0.c:677`) only *pushes* a "processing" screen — it never
clears history. The all-page clear runs only on full teardown: new game
/ load game / exit / fatal error (`FUN_00819a40`).

So the history persists across AM→PM→EVE→next-day advances. Back walks
back through the screens you visited, bounded only by the per-page list
and de-dup, until the screen is closed or the game session is torn down.

**Correction to the working assumption:** the history is NOT partitioned
per game phase. It is a persistent, de-duplicated, per-top-tab back-stack
for the whole session.

## 3. What the Rust port should do (observational equivalence)

* Keep a **back-stack of visited screens** with **de-dup** (revisiting a
  screen doesn't stack a duplicate).
* **Back** pops to the previous screen; enabled only when the stack has a
  previous entry.
* Do **not** clear the stack on Continue / phase advance. Clear it on new
  game / load / exit.
* A ▶ forward exists too (re-push after Back), enabled when there's a
  forward entry — lower priority than Back.
* Refinement (later): the exe keeps a *separate* history per top-tab
  page. Observable difference is subtle; a single session stack is a
  faithful first approximation, per-tab is the exact match.

Not yet implemented in the port — the sidebar ◀▶ currently show "Screen
history not yet implemented". This is the spec for building it.
