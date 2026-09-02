# GDI capture harness

Produces byte-exact fixtures for the ported packed-16bpp renderer by
hooking every primitive call inside `cm0102_GDI.exe` while a real game
is running. Companion to `crates/cm-render/src/packed_capture.rs` — the
Rust replay dispatcher that consumes them.

See [[gdi-renderer-is-ground-truth]] for the strategic pivot this
harness belongs to, and [[frida-instrumentation-crash-evidence]] for why
this script deliberately hooks a minimal surface (~14 primitive VAs)
rather than stalking the executable.

## Files

- **`capture.js`** — Frida script. Hooks `FUN_005cd3e0` (line),
  `FUN_005cd730` (rect), `FUN_005cdd60` (darken), `FUN_005cda90`
  (restore/blit), `FUN_005ceaa0` (glyph), `FUN_005cf570` (panel/bevel),
  `FUN_005d03a0` (wrapped text). Exposes `rpc.start(screen)`,
  `rpc.stop()` (returns the accumulated fixture) and
  `rpc.readFontTable(idx)`.
- **`run_capture.py`** — Python driver. Attaches to a running
  `cm0102_GDI.exe`, sends `start`, waits for the human to trigger a
  redraw, sends `stop`, rewrites `*_pending_font` calls into inlined
  `FontDump` payloads, and writes the fixture JSON.

## Recipe

```bash
# 1. Start the game (with the DAT files it expects).
"C:/path/to/cm0102_GDI.exe"

# 2. Navigate the game to the screen you want to capture; leave it there.

# 3. From another shell:
D:/Python312/python.exe D:/cm0102-rs/tools/gdi_capture/run_capture.py \
    news_home fixtures/news_home.json

# 4. When the script says "press Enter to start", press Enter, then
#    trigger a redraw inside the game (click something, scroll, hover),
#    then press Enter again to stop.
```

## Replay

```bash
cargo test -p cm-render --test capture_fixtures -- --nocapture
```

The replay test walks every `.json` in `fixtures/`, loads it via
`cm_render::packed_capture::replay`, and fails if a single u16 disagrees
between the replayed surface and the captured `after` framebuffer. That
IS the byte-exact contract.

## Adding a new primitive

1. Land the port in a `packed_*.rs` module with its own byte-exact
   unit tests.
2. Add a `Call` variant to `packed_capture.rs`.
3. Add an `Interceptor.attach` block in `capture.js` mirroring the exe
   ABI (stdcall/cdecl args on the stack, `argI32/U32/U16` helpers).
4. Ship a fixture that exercises it.

## Known limits (2026-09-03)

- The `readBytes` RPC needed to fully inline font glyphs isn't wired
  yet — text calls stay as `*_pending_font` on first capture; add
  `rpc.readBytes(ptr, n)` to `capture.js` when the first text-heavy
  fixture demands it. Kept minimal for now on the crash-evidence
  guidance.
- Only the GDI variant is targeted; the DirectDraw variant has the same
  VAs at different addresses and its framebuffer lives behind
  `IDirectDrawSurface::Lock`, which needs a separate hook.
- Palette globals (`DAT_00ad6b24` outer highlight, `DAT_00ad6b3c`
  default bevel) aren't sampled yet — panel captures serialise them as
  0 and the replay falls back to `PanelPalette::default()`. Wire them
  the first time a panel golden fails on an outer-highlight pixel.
