# Handover: pinning down DirectDraw call sites in cm0102.exe

**Goal:** identify exactly where in `cm0102.exe`'s own code (not just inside
`ddraw.dll`) the game issues its DirectDraw rendering commands (`Blt`,
`Flip`, `Lock`, `Unlock`, surface creation) — the mechanism needed for
pixel-exact GUI capture/replay (see `[[frida-capture-replay]]` /
`[[emulation-pipeline]]` project memory). This is a fresh sub-thread off the
main match-engine work; read this file top to bottom before doing anything.

Paste this whole file into a new chat to resume with full context.

---

## 1. Why this is a different problem than everything else this project has done

Every other live-tracing target in this project (screen builders, match
engine, position decoder) is a **fixed static address inside cm0102.exe
itself** — you can `bp <addr>` directly from Ghidra's decompile.

`Blt`/`Flip`/`Lock` are **COM vtable methods** on an `IDirectDrawSurface`
object created *at runtime*. There is no fixed address to breakpoint —
you have to:
1. Catch the `IDirectDraw` device get created (`DirectDrawCreate`/`Ex`).
2. Catch it get upgraded to a newer interface (`QueryInterface` →
   `IDirectDraw4`/`IDirectDraw7`) — **confirmed this game does this**, see
   §3.
3. Catch `CreateSurface` on the *upgraded* interface to get a real
   `IDirectDrawSurface(N)` pointer.
4. Read *that* object's vtable to get the real, resolved code addresses of
   `Blt`/`Flip`/`Lock`/`Unlock`.
5. **Only then** can you `bp` those resolved addresses and find out which
   `cm0102.exe` function (not `ddraw.dll` function) is the caller —
   at that point this becomes a completely standard investigation, no
   different from anything else in this project (check the return address /
   call stack, map it via `functions.json`/`subsystem_map.json` like every
   other trace in this session).

So steps 1–4 are pure infrastructure to get to a normal address. Almost all
of the work done today is steps 1–4; step 5 (the actual interesting part —
mapping calls back to `cm0102.exe` source files) hasn't started yet.

## 2. Toolchain already proven safe and working today (reuse, don't rebuild)

- **Frida is the right tool for this specific job.** Not x64dbg. The
  earlier project-wide conclusion that Frida is unsafe (`[[frida-instrumentation-crash-evidence]]`
  memory) is specifically about `Interceptor.attach`'s 5-byte trampolines
  colliding in *cm0102.exe's own tightly-packed 2001-era code* when hooking
  thousands of functions at once. `ddraw.dll` is a normal, modern,
  well-aligned Windows system DLL — hooking a handful of its exports is
  Frida's single most common, best-supported use case. Confirmed today:
  spawning the game under Frida with these hooks active produced **zero
  crashes**, full normal play (menus, squad screens, everything) across
  several relaunches.
- Frida version installed: **17.17.0**. Note the v17 API change already
  hit and fixed today: `Module.findExportByName` (the old *static* method)
  no longer exists — use the **instance** method instead:
  ```js
  var mod = Process.findModuleByName('ddraw.dll');
  var addr = mod ? mod.findExportByName('DirectDrawCreate') : null;
  ```
- Don't force-load `ddraw.dll` with `Module.load()` — it may not be loaded
  yet at spawn time, and forcing it early could disturb the game's own init
  order. Use `Process.attachModuleObserver({ onAdded: ... })` and hook once
  it appears naturally (already implemented, see §4).
- x64dbg (`x32dbg.exe`/`headless.exe` under
  `D:\temp\claude\D--cm0102-rs\d6623250-6dc0-4547-be75-4e218aea4678\scratchpad\x64dbg\release\x32\`)
  is proven safe for **cm0102.exe's own static addresses** (the actual
  position-decode and match-engine debugging done earlier used it
  extensively, safely, at full scale — 4,905 breakpoints, zero crashes).
  It's the right tool for step 5 once you have a real resolved address to
  chase inside `cm0102.exe`. Key gotchas already worked out if you use it
  again:
  - Script *files* (`-cf`) need **unquoted** numeric args for
    `bpcond`/`SetBreakpointCommandCondition` (`bpcond ADDR, 0` not
    `bpcond ADDR, "0"`) — quoted works fine on the inline `-c` command line
    but silently breaks the file-based script parser.
  - Use `SetBreakpointSingleshoot` (fully-qualified name), not the
    `bpsingleshoot` alias — the alias fails specifically in file-mode
    scripts.
  - `-c`/`-cf` command execution simply **doesn't fire at all** in
    `-pid <attached>` mode — only in fresh **spawn** mode. If you need to
    attach to an already-running/already-loaded-save game process, you'll
    need a different injection path (a live command channel), not solved
    this session.
  - A fresh spawn always stops at the auto-set entry breakpoint
    (`00936A33`); either `bc 00936A33` before `run` in the script, or (GUI
    mode) send it a real F9 keystroke via `SendKeys` after
    `SetForegroundWindow` — the game will otherwise sit paused forever with
    `Responding: False`.
  - **Never `Stop-Process -Force` on `headless.exe`/`x32dbg.exe`while it's
    attached** — killing the debugger kills the debuggee with it (standard
    Win32 debug-API behavior). Detach cleanly (or accept the game will need
    reloading) before tearing down a session.

## 3. What's confirmed working right now (script:
`D:\temp\claude\D--cm0102-rs\d6623250-6dc0-4547-be75-4e218aea4678\scratchpad\frida_ddraw_capture.py`)

Run with: `D:/Python312/python.exe frida_ddraw_capture.py --spawn`
(spawns `D:\cm0102\cm0102.exe`, cwd `D:\cm0102`).

Confirmed live, this session:
- `ddraw.dll!DirectDrawCreate` hook fires, **succeeds** (`hresult: 0`).
- Real `IDirectDraw` device pointer captured (e.g. `0x120d420`).
- Its vtable dumped (24 slots) — mix of `DDRAW.dll` and, interestingly,
  **`apphelp.dll`** addresses (a Windows App Compatibility shim is
  intercepting some DirectDraw calls for this legacy app — potentially
  relevant context for later, not yet explored).
- `CreateSurface` (vtable slot 5, standard COM order) hooked on the base
  v1 interface — **never fires** even deep in-game. This is expected once
  you know why: see next point.
- **The game upgrades via `QueryInterface`** (vtable slot 0) to a newer
  interface — confirmed 3 distinct upgraded pointers captured per run
  (e.g. `0x119c460`, `0x119c5a0`, `0x119c520` in one run). `CreateSurface`
  hooked on *those* vtables **does fire repeatedly with `hresult: 0`**
  once you're in-game.

## 4. The bug blocking further progress — read this before continuing

`CreateSurface`'s output parameter (`args[2]`, the `LPDIRECTDRAWSURFACE*`
out-pointer) is being read successfully in the sense that it doesn't crash,
but the resulting "surface pointer" is **suspicious**:
- It's **byte-for-byte identical across every single `CreateSurface` call**,
  regardless of which of the 3 upgraded interfaces made the call.
- It lands inside `DDRAW.dll`'s own static/vtable data region (e.g.
  `0x6c80a0c0`, right next to the device's own vtable base
  `0x6c80a360` seen earlier), **not** a plausible per-instance heap
  allocation.
- Dereferencing *that* as a vtable produces slot values that look like raw
  x86 instruction bytes reinterpreted as pointers (e.g. `0x8bc04589` —
  `89 45 C0 8B` is a real, plausible instruction encoding) — i.e. we're
  reading into the middle of code, not a real vtable array.

**This means the `args[2]` read is wrong for whatever's actually being
hooked** — most likely one of:
- The vtable slot resolved via `vt.add(5 * 4)` isn't actually
  `CreateSurface` for *this* upgraded interface version (COM interface
  versioning is supposed to preserve earlier method order and only append
  new methods — that assumption needs verifying against the *actual* REFIID
  requested in the `QueryInterface` call, not assumed from vtable position
  alone).
- `args[2]` genuinely isn't the right stack index for this call — worth
  re-deriving the exact `IDirectDrawSurface(N)::CreateSurface` signature
  Microsoft actually shipped for whichever `IDirectDraw` version this game
  requests, rather than assuming.
- The output stack slot is being reused/stale-read between `onEnter` and
  `onLeave` in a way that isn't obvious yet — worth logging the *raw bytes
  at the exact moment of `onLeave`* rather than trusting a single
  `readPointer()`.

## 5. Concrete next steps, in order

1. **Log the REFIID requested in `QueryInterface`, not just the resulting
   pointer.** `QueryInterface(REFIID riid, void** ppv)` — `args[1]` is a
   pointer to a 16-byte GUID. Read those 16 bytes and compare against the
   well-known `IID_IDirectDraw2/4/7` GUID constants (findable via a quick
   web search or a `ddraw.h` header) to know **for certain** which
   interface version is actually in play, instead of assuming standard
   vtable-index math holds.
2. Once the real interface version is confirmed, **verify the real
   `CreateSurface` vtable index for that specific version** — don't
   re-assume index 5, look it up against the actual COM interface
   definition for that version.
3. Add a raw hex dump (`Memory.readByteArray(surfOutAddr, 16)`) at
   `onLeave`, printed alongside the `readPointer()` result, so a genuinely
   garbled/stale read is visually obvious rather than silently producing
   a plausible-looking-but-wrong pointer.
4. Once a real, distinct-per-call surface pointer is confirmed (sanity
   check: **different addresses across different `CreateSurface` calls** —
   the single clearest signal that step 1–3 succeeded), dump *its* vtable
   the same way already done for the device interface, and identify
   `Blt`/`Flip`/`Lock`/`Unlock` by COM position for that specific,
   now-confirmed interface version.
5. Hook those resolved addresses. In `onEnter`, capture
   `this.context.ecx`/the return address (`Thread.backtrace()` or just read
   the top-of-stack return address directly) to identify the **calling
   `cm0102.exe` function** — cross-reference that address against
   `functions.json`/`subsystem_map.json` exactly like every other trace
   this session (see `summarize_wide_hits.py` in the same scratchpad
   directory for the established pattern: map an address to its
   `subsystem_map.json` entry, pull `string_hits` for the real `.cpp`
   source file).
6. From there it's a completely standard investigation: which `cm0102.exe`
   source files/functions actually issue each kind of DirectDraw command,
   with what parameters (source/dest rects, surface identity) — the actual
   deliverable for pixel-exact capture/replay.

## 6. Files to know about

- `D:\temp\claude\D--cm0102-rs\d6623250-6dc0-4547-be75-4e218aea4678\scratchpad\frida_ddraw_capture.py`
  — the working script, continue editing this one directly.
- `D:\temp\claude\D--cm0102-rs\d6623250-6dc0-4547-be75-4e218aea4678\scratchpad\summarize_wide_hits.py`
  — the established address→source-file mapping pattern, reuse for step 5
  above.
- `D:\cm0102-carve\ghidra_out\cm0102.exe\functions.json` /
  `D:\cm0102-carve\newcarve\atlas\subsystem_map.json` — the same lookup
  tables used throughout this whole project.
- Note: the scratchpad directory above is **session-local** and may not
  survive into a new chat session unmodified — if it's gone, the Python
  scripts in this handover are short enough to reconstruct from the
  descriptions above, or ask to have them re-created from this file's
  content.

## 7. Explicitly out of scope for this thread (don't rabbit-hole)

- The match-engine work from earlier today (position decoding, shot
  conversion calibration) is **done and separate** — don't touch
  `match_engine_exe.rs` from this thread.
- The ghost-player virtual-id-pool writer (still unresolved from much
  earlier in the project) is unrelated — don't conflate.

## 8. RESOLVED (2026-08-27, follow-up session)

The goal in §1 is **achieved**: real `cm0102.exe` call sites for every
DirectDraw rendering call are captured and verified against the decompiled
source. Two wrong turns from §4/§5 above, plus two NEW indexing bugs found
this session, are all resolved below.

### What was wrong

1. **The "QI-upgrade → IDirectDraw4/7 → CreateSurface" theory (§4/§5) was
   flat wrong.** Logging the actual REFIID (as §5 step 1 recommended)
   showed it was `IID_IUnknown` (`00000000-0000-0000-C000-000000000046`),
   not a DirectDraw interface. The "upgraded" pointers were ordinary
   `QueryInterface(IID_IUnknown)` identity results. Indexing slot 5 of a
   3-method `IUnknown` vtable read garbage adjacent memory — that's why it
   produced a constant, nonsensical "surface" hundreds of times/sec. This
   whole code path was deleted, not fixed.
2. **A "renders via GDI" detour** (tried next) was disproven by dumping
   `cm0102.exe`'s own import table (`Module#enumerateImports()` — see
   `frida_dump_imports.py` in the scratchpad): its only `ddraw.dll` import
   is `DirectDrawCreate`, and its `GDI32.dll` imports
   (`StartDocA`/`EndDoc`/`StartPage`/`EndPage`/`CreateFontA`/`TextOutA`/
   `GetTextMetricsA`) are the **print-report feature** (printer DC), not
   screen rendering. No `BitBlt`/`StretchBlt`/`GetDC`/`CreateCompatibleDC`
   imported at all. Rendering is 100% through the DirectDraw COM vtable.
   (Aside: `Module#enumerateImports()`'s `.address`/`.slot` fields are
   unreliable on Windows in this Frida build — every import in a module
   reports the same slot address. Don't rely on them; dump once via a
   throwaway script and hook vtables instead.)
3. **Two vtable-order bugs of my own**, found by comparing live capture
   data against the decompiled source (ground truth beats memorized
   ddraw.h order):
   - `IDirectDraw` v1: **`CreatePalette` is slot 5, `CreateSurface` is
     slot 6** (`...CreateClipper(4), CreatePalette(5), CreateSurface(6),
     DuplicateSurface(7)...`) — I initially had these two swapped.
   - `IDirectDrawSurface` v1 has **36 methods (indices 0-35), and does
     NOT include `SetSurfaceDesc` at all** (that method was added in
     later `IDirectDrawSurface2+`). Including it shifted every index
     after it by one, so real `Unlock` (slot 32) was showing up mislabeled
     as `SetSurfaceDesc`. Confirmed via decompiled source: the call right
     after every `Lock` passes exactly one argument (`0`), matching
     `Unlock(lpSurfaceData)`'s real signature, not `SetSurfaceDesc`'s
     two-argument one.

### The technique that actually worked

Rather than continuing to guess/verify individual "interesting" vtable
slots, hook **every** real (unshimmed) method on the device vtable *and*
on any surface vtable reachable from a device method's out-param
(`CreateSurface`/`GetGDISurface`/`DuplicateSurface`), generically, with a
call-site recorder keyed by `(method name, caller return address)` — one
`new-callsite` event the first time a site is seen, then a periodic
(3s) count summary instead of one message per call (some fire hundreds of
times/sec). This is now the standing implementation in
`frida_ddraw_capture.py`. A vtable is only trusted enough to walk if its
slot 0 (`QueryInterface`) resolves into a real, known module — the lesson
from mistake #1.

Also load-bearing: `cm0102.exe` **self-relaunches into a fresh process
at startup** at least sometimes (not deterministically — one run had no
relaunch at all). `frida.spawn()`'s target can die out from under you with
zero events ever firing on it. `enable_spawn_gating()` is NOT supported on
Windows in this Frida build (`frida.NotSupportedError: not yet supported
on this OS`) — the working fix is a tight poll loop
(`device.enumerate_processes()`) watching for a brand-new `cm0102.exe` PID
right after resuming the initial spawn, attaching within milliseconds.
This is implemented as `--gated` mode in the script (despite the flag
name predating the pivot away from real spawn-gating).

### Confirmed real `cm0102.exe` call sites (static/fixed-base addresses —
this is a non-ASLR 2001 PE, so `cm0102.exe+0xOFFSET` from a live capture
equals Ghidra's `0x00OFFSET` directly, e.g. `cm0102.exe+0x1cc752` ==
`FUN_005cc310` at `0x005cc752`)

All from `FUN_005cc310` (the graphics-init routine, confirmed via its own
"Unable to initialise the graphic..." error strings) unless noted:

| Method | Caller address | Notes |
|---|---|---|
| `SetCooperativeLevel` | `0x005cc752` (also `0x005cc5c6` on re-entry) | fires again on window state changes |
| `SetDisplayMode` | `0x005cc771` | |
| `CreateSurface` | `0x005cc7c6` and `0x005cc80d` | two surfaces (likely primary + a second one, e.g. offscreen/back buffer) |
| `CreateClipper` | `0x005cc89b` | |
| `IDirectDrawSurface::GetSurfaceDesc` | `0x005cc84e` | |
| `IDirectDrawSurface::SetClipper` | `0x005cc90a` | |

From elsewhere (the actual per-frame rendering path, addresses in the
`0x005cd2xx`-`0x005d0dxx` range — NOT yet mapped to named functions/source
files, that's the natural next step):

| Method | Caller addresses |
|---|---|
| `IDirectDrawSurface::Blt` | `0x005ccd43`, `0x005cd9c3` |
| `IDirectDrawSurface::Lock` | `0x005cd252`, `0x005cd4aa`, `0x005cd542`, `0x005cdb83`, `0x005cdd45`, `0x005ce1d0`, `0x005cee0a`, `0x005cf9c5`, `0x005d0d99` (more likely exist — this list is from a short capture window, not exhaustive) |
| `IDirectDrawSurface::Unlock` | paired 1:1 with each `Lock` site above, a small offset later (e.g. `0x005cd252`→`0x005cd2e5`) |

The `Lock`→(manual pixel writes)→`Unlock` pattern at many distinct call
sites, versus only 2 `Blt` sites, suggests this engine does most of its
actual drawing by writing directly into the locked surface's pixel buffer
(software rendering, matching a 2001-era engine design) rather than
issuing many `Blt` calls — the 2 `Blt` sites are probably the
final present-to-primary / palette composite step. This lines up with
`[[gui-core-ported]]`'s already-known "dirty-rect blit" from the ported
GUI core cluster — that's very likely one of these exact `Lock` sites.

### §9. Call sites mapped to containing functions (2026-08-27, same session)

All addresses land in one tight, contiguous, unattributed cluster —
`0x005cc310`-`0x005d1c3e` — sitting alphabetically between `goldcup.cpp`
(ends `0x005cc02d`) and `gre_cup.cpp` (starts `0x005d2240`) per the
verified alphabetical `.text` layout ([[text-is-alphabetical]]). No
surviving `__FILE__` assert string pins the exact name, but a name like
`graphics.cpp` fits both the alphabetical slot and the content (DirectX
init, font loading, surface Lock/Blt) — treat as a strong hypothesis, not
confirmed. `subsystem_map.json` classifies the whole cluster `GUI`
(confidence 1.0 at the entry point, where the actual "Unable to initialise
the graphics..." strings hit).

Containing functions, read via Ghidra decompile (`D:\cm0102-carve\ghidra_out\cm0102.exe\decompiled\<addr>.c`):

| Caller addr | Containing fn | What it is |
|---|---|---|
| `0x005ccd43` | `FUN_005ccba0` | **The present routine.** Blits (or `BltFast`s in fullscreen) a dirty rect from the back buffer (`DAT_00ad6bd4`) onto the primary surface (`DAT_00acdf94`) — real 5-arg `Blt(lpDestRect, lpDDSrcSurface, lpSrcRect, dwFlags=DDBLT_WAIT, lpDDBltFx)` confirmed by argument shape. This is almost certainly what `[[gui-core-ported]]` already calls "dirty-rect blit." |
| `0x005cd9c3` | `FUN_005cd840` | **Rectangle fill/outline primitive.** Solid fill → direct `DDBLT_COLORFILL` blt (`dwFlags=0x1000400`, color in the `DDBLTFX`). Outline → calls `FUN_005cd420` (one of the confirmed `Lock` sites below) once per edge to draw 4 lines manually. |
| `0x005cd252`/`0x5cd2e5` | `FUN_005ccdd0` | **CORRECTED (was "sprite scaler", wrong):** a rect **fade transition effect**, taking a signed fade-amount as its last argument. Confirmed via its only 2 callers, both in `FUN_005b6f10` (`game.cpp`, the boot **splash-logo sequence**: loads `logo.rgn`/`eidos.rgn`(the Eidos publisher logo)/`kio.rgn`/`savechip.rgn` from the same `SI_DATA` container used for fonts) — draws each logo via `FUN_005cdcc0` (the "restore-background" primitive, reused generically as a blit-from-memory), then calls this function with `0x46` (+70) to fade in and `0xffffffce` (−50) to fade out over that same rect. Its `malloc`'d per-source-pixel cache (RGB→565/555 conversion inline) is keyed by fade step, not scale factor — it's memoizing the blended-toward-black color for each distinct source pixel value so the row loop doesn't redo the blend math per pixel. |
| `0x005cd4aa`/`0x5cd542`→`0x5cd420` | `FUN_005cd420` | Called by the rect-outline path above with `(x1,y1,x2,y2,style,color)` — a line-drawing primitive (Bresenham-style, given it's invoked once per rectangle edge). |
| `0x005cdb83` etc. | `FUN_005cdac0` | **Save-background.** Locks read-only (flags `0x11`), allocates (or reuses a cached, size-matched) buffer, memcpy's the locked region into it row-by-row (respecting surface pitch). Classic "save the area behind a popup" primitive. Returns the cache struct `{w, h, size, dataPtr}` for later restore. |
| `0x005cdd45` etc. | `FUN_005cdcc0` | **Restore-background** — the pair to the above. Locks write-only (flags `0x21`), memcpy's the cached buffer back onto the surface. |
| `0x005ce1d0` etc. | `FUN_005cdfd0` | **Darken/dim a rect.** Lazily builds a 65536-entry precomputed lookup table (one entry per possible 16-bit pixel value, each darkened to 60% brightness), then locks write-only and remaps every existing pixel in the rect through that table in place. Used for e.g. dimming the background behind a modal dialog. |
| `0x005cee0a` etc. | `FUN_005ced50` | **Single-line text/glyph renderer.** Takes a string, a font index (indexes a font-metrics table at `&DAT_00accb9c`), and an RGB555/565 color; unpacks 4-bit-per-pixel glyph bitmaps (`0xf`=solid text color, `0`=transparent/skip, else alpha-blended against the existing background pixel) and writes them into the locked buffer pixel-by-pixel. Also draws an underline via `FUN_005cd420` when requested. This is the renderer for the `.fnt` assets in [[no-pil-approximation]]. |
| `0x005cf9c5` etc. | `FUN_005cf8e0` | **Box/panel frame drawer.** Large flag-driven routine (dozens of `param_5` bit flags) selecting flat/gradient fills and single/beveled-3D/rounded borders — draws concentric rectangles with alternating light/dark edge colors for the classic raised/sunken Win95-era panel look. Built entirely on `FUN_005cd420` (line) and `FUN_005cd840` (rect fill). |
| `0x005d0d99` etc. | `FUN_005d0870` | **Word-wrapped multi-line text box.** Builds a wrapped-line list, supports password masking (`param_5&0x80` replaces every char with `*`), left/right/center alignment, and renders each line TWICE via `FUN_005ced50` — an offset shadow-color pass then the real color — i.e. drop-shadow text. This is the actual widget behind most on-screen labels/paragraphs. |

Also: `FUN_005ce750` (called from the init routine) loads the game's
fonts — confirmed via `string_hits`: `arial_narrow_10.fnt`,
`arial_narrow_11.fnt`, `arial_14.fnt`, `arial_16.fnt`, `arial_18.fnt`,
`trade_cond_24_bold.fnt`, `trade_cond_28_bold.fnt` (each paired with
`SI_DATA`) — this is the loader for the `.fnt` glyph assets already
decoded per [[no-pil-approximation]].

### The engine's complete low-level 2D drawing toolkit (now identified)

| Function | Role |
|---|---|
| `FUN_005ccba0` | Present (dirty-rect `Blt`/`BltFast` from back buffer to primary) |
| `FUN_005cd420` | Line drawer (called with `x1,y1,x2,y2,style,color`) |
| `FUN_005cd840` | Rectangle fill (`DDBLT_COLORFILL`) / outline (4× line) |
| `FUN_005ccdd0` | Rect fade transition (signed fade-amount arg; used for the boot splash-logo fade in/out) |
| `FUN_005cdac0` / `FUN_005cdcc0` | Save-background / restore-background pair — `...cc0` is also reused generically as "blit this raw pixel buffer to the surface" (e.g. drawing the splash-logo bitmaps) |
| `FUN_005cdfd0` | Darken-rect effect (precomputed 64K-entry LUT) |
| `FUN_005ced50` | Single-line alpha-blended glyph/text renderer |
| `FUN_005cf8e0` | Box/panel frame (flat/gradient fill, beveled/rounded borders) |
| `FUN_005d0870` | Word-wrapped multi-line text box (align, password mask, drop shadow) |

Every one of these is reached only via `Lock`→(manual pixel read/write)→`Unlock`
except the fill/present pair, which use real `Blt`. This is a complete,
verified answer to the original handover question (§1): the whole GUI is
software-rendered into a locked system-memory surface by this small
primitive library, then presented via two `Blt` call sites.

`FUN_005ccdd0` was traced to its only 2 callers (`functions.json` call
graph — both inside `FUN_005b6f10` in `game.cpp`) and is now correctly
understood as the fade-transition effect described above, not a scaler.

### §10. `cm-render` port status: complete (2026-08-27, same session)

Checked each of the 9 primitives against the `cm-render` crate — 7 were
**already ported** from earlier work on this project (before this
sub-thread started), which independently corroborates tonight's live
capture:

| Function | `cm-render` module | Status |
|---|---|---|
| `FUN_005ccba0` (present) | `blit.rs` | already ported |
| `FUN_005cd420` (line) | `line.rs` | already ported |
| `FUN_005cd840` (rect fill/outline) | `primitives.rs` / `panel.rs` | already ported |
| `FUN_005cdfd0` (darken/transparency) | `panel.rs::F_TRANSPARENT` / `dim_region` | already ported |
| `FUN_005ced50` (glyph blitter) | `glyph_blit.rs` | already ported |
| `FUN_005cf8e0` (box/panel frame) | `bevel.rs` / `panel.rs` | already ported |
| `FUN_005d0870` (word-wrap text box) | `drawstring.rs` / `font.rs` | already ported |
| `FUN_005cdac0` / `FUN_005cdcc0` (save/restore-background) | `background.rs` | **added this session** |
| `FUN_005ccdd0` (fade transition) | `fade.rs` | **added this session** |

`background.rs` ports the save/restore round-trip exactly (skips the
exe's `malloc`-cache-reuse micro-optimisation — irrelevant in Rust).
`fade.rs` ports the animation as a `Surface::fade_rect` method taking a
`present: impl FnMut(&Surface)` callback invoked once per step, matching
the exe's per-frame Lock→blend→Unlock→present loop; direction (fade
in/out) is inferred from the calling convention (see §9) rather than a
bit-for-bit trace of both decompiled branches — flag `INFERRED` if it
needs re-verification later. Both ship with unit tests following the
crate's existing convention (5 tests each); full suite is green at 316
tests (306 pre-existing + 10 new).

The engine's complete low-level 2D drawing primitive library is now
fully represented in `cm-render` — nothing left to port from this
investigation.

### §11. News screen: the exact-replica lesson (2026-08-27, same session)

After §8–10, `news`'s `dump_screen_geometry` output was declared
"PIXEL-EXACT (geometry) match" against `reports/screen_captures/news.json`.
That claim was **true but badly misleading** — shown a real screenshot of
the actual News page, the rendered output was unrecognizable. Root cause:
`news.json` was captured via Unicorn emulation
(`tools/capture_news.py`), which forces the unread-news count to 0 "to
skip the row loop" — because the emulator has no real game state to
iterate over. That one shortcut meant the capture **never observed** the
news list, filter bar, story panel, second tab row, or nav buttons at
all — it only ever saw the header, because that's the only thing built
before the (skipped) loop. `diff_screens.py` was genuinely comparing our
port against ground truth; the ground truth itself just wasn't the whole
screen. Two more specific misses layered on top: an "8 tabs, y:80→545"
capture record (a tab-strip builder's cursor state on entry) was modeled
as **8 full-height vertical columns** spanning most of the screen —
structurally implausible for a tab strip and never sanity-checked against
either general genre knowledge or the project's own memory note calling
it "a contiguous strip, not buttons" before being rendered and shown.

**The fix, once actually done properly:** run the **real, live, currently-
executing** exe (a real save loaded, actually on the News page) instead of
an emulator with fabricated state. Frida-hooked the 4 real widget/area
constructors (`GUIO=0x549580`, `AREA=0x549790`, `TABS=0x5D7070`,
`NAV=0x5D75B0`) globally, but only **recorded** a call while execution was
actually inside the draw callback itself (`0x00770170`, entry/exit
tracked via its own `Interceptor.attach`) — this needs zero fabricated
state and zero RET-stubbing, so every nested helper the real code calls
runs for real. Text pointers were dereferenced **live, at each widget's
own construction time** — a critical detail: many widgets share one
scratch text buffer that the *next* widget's construction overwrites, so
reading it after the fact only shows the last thing written there.

Real capture, one call to the draw callback: **617 `GUIO` + 65 `AREA`**
calls (vs. the 11+6 the static top-level disassembly could see — the rest
come from nested helper calls). Of the 65 areas, ~51 are zero-size
per-widget bookkeeping (`widget_pool.rs`'s own documented "dummy area"
pattern) and ~400 of the GUIOs are the **left sidebar's full cascading
menu tree** (Competitions→World Cup Quals→…, Nations A-I→Argentina→…) —
built once via the shared `FUN_00745540` sidebar dispatcher
(already-documented separately, see [[menu-command-tree]]), not part of
News itself, and deliberately excluded from `NewsView`.

**Real, verified News-specific structure** (12 top-level areas + real
labels), now in `cm_domain::NewsView` / `view_render.rs`'s
`impl RenderableView for NewsView` / `crates/cm-render/src/bin/render_news.rs`:

| Area | Rect | flags |
|---|---|---|
| Header | 100,10→790,70 | 48 |
| Header inner | 100,25→790,55 | 1 |
| Top tabs bg / container | 100,80→790,115 | 2 / 1 (4 children) |
| News list | 110,125→780,215 | 1 (2 cols × 5 rows) |
| Filter bar | 405,220→655,240 | 2 (2 children) |
| Selected headline strip | 110,245→780,280 | 2 |
| Story panel | 110,285→780,500 | 2 |
| Bottom tabs bg / container | 100,510→790,545 | 2 / 1 (4 children) |
| Nav row (Back/Next) | 100,555→790,590 | 1 (2 children) |

Real static labels (from the draw callback's own string table,
`subsystem_map.json` at `0x770170` — constant regardless of game state):
top tabs `All`/`Messages`/`Competitions`/`Injuries and Bans`; bottom tabs
`Contracts and Media`/`Transfers`/`Jobs`/`Records`; `Filter :`;
`Next Unread`. Real per-instance state captured live matching the actual
screenshot: header `"Christoph Olewicz News"`, 5 real list rows (date +
headline pairs, row 0 highlighted rflags=528), real selected-item
headline+body, `NAV: Back=enabled, Next=disabled`.

Rendered and visually confirmed against the real screenshot — structurally
correct (right text in the right place, correct row highlighted, correct
button states). A first render still discarded real per-widget detail:
every tab/button was hardcoded to one flag value instead of using its own
captured `rflags`. Fixed using data already sitting in the same capture
log but not yet mined: the selected top tab's real `rflags` is `2096`
(0x830, an extra "selected" bit over the unselected `48`) — renders as a
light/white box instead of navy; and enabled-vs-disabled ("Back" vs
"Next"/"Next Unread") turned out to be carried by `tflags` (12 vs 44), a
*separate* field from `rflags`, which stayed 48 (same filled+bevel box)
for both — the initial version had used `rflags` for enabled/disabled and
therefore gave disabled buttons the wrong box style entirely. Also added
the yellow story headline via a `fg_color` marker (screenshot-driven, not
independently verified against the palette).

**Round 2 fixes** (shown a real screenshot, still visibly off): text
wasn't centered anywhere (`Font::text_width()` was sitting unused in the
API the whole time — plain oversight) and the tab labels used the wrong
font (`3`/arial_14 instead of the real captured `1`/arial_narrow_10, a
value already sitting correctly in the earlier capture log and used
wrong anyway). Both fixed by actually reading data already on hand.

**Round 3 — solving colors properly, not per-widget guessing:**
`colP`/`colS` are opaque palette indices (`8204`, `33808`, …) with no
obvious decode. Rather than reverse-engineer what they mean, sidestepped
the question entirely: read the exe's own `DAT_00AD6BD4` (documented
back-buffer pointer, `blit.rs`'s globals table), called its real `Lock`
vtable method directly via Frida `NativeFunction` (same v1 `DDSURFACEDESC`
layout decoded earlier this session — `dwSize=0x6C`, `lpSurface@0x24`,
`lPitch@0x10`), and read real RGB565 pixels at known coordinates while
News was actually on screen — see
`scratchpad/frida_sample_news_pixels.py`. This is ground truth with zero
interpretation needed: whatever pixel value is sitting in memory is
exactly what the exe rendered. Confirmed real colors: header blue
`(0,48,165)`; tab fill `(33,0,99)` — **identical for selected AND
unselected tabs**, meaning the visible "selected" difference is a
border/highlight effect on top of the same fill, not a different fill
color (an assumption from round 1, now corrected); list-row grey
`(80,82,80)`; selected-row red `(132,0,0)`; filter-bar grey
`(78,75,74)`; **Back/Next buttons are grey `(132,130,132)`**, not the
blue used for header/tabs (a flat wrong guess in round 1). Applied all of
these in `render_news.rs`; full suite still green.

Remaining known gaps: exact selected-tab highlight/border color (used a
placeholder gold outline, the real effect isn't isolated yet), sample
points for the list rows may be landing slightly off (real screenshot
reads more blue-toned than the measured grey), no scrollbar, no
story-panel background photo.

**The actual lesson, independent of News specifically:** an automated
diff passing only proves the two things it compares agree with each
other — it says nothing about whether either one represents the real,
complete target. Sanity-check structure against genre knowledge (or a
real screenshot) *before* trusting a capture's completeness, especially
when a capture tool takes a shortcut (a forced constant, a narrow
RET-stub range) to avoid needing real state. And once shown a mismatch:
check whether the real answer is already sitting in data already
captured before reaching for a new guess — twice this session it was.
For colors specifically: reading the exe's own rendered pixels directly
beats reverse-engineering an opaque palette-index system — and once
that Lock-based pixel-sampling technique exists, it applies to every
future screen, not just News.

### Concrete next steps

1. Apply the same live-Frida-capture technique (hook constructors, scope
   to a screen's own draw-callback entry/exit, dereference text live) to
   other screens, now that it's proven — rather than extending the
   Unicorn/fake-state approach further.
2. Verify News's real colors/bevel styling against the exe (a further
   Frida capture of the actual `Blt`/`Lock` pixel output, or reading the
   palette table directly) — the current render's colors are placeholder.
3. Wire `fade.rs`/`background.rs` into whatever in `cm-ui-app` currently
   drives the boot splash-logo sequence and popup/dropdown overlays, once
   those screens are being built.
