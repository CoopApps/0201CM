//! Palette / pixel-format globals read by the widget renderer
//! (`FUN_005d7aa0`, `FUN_005ce2d0`, and the wrapped-text branch of
//! `FUN_005d03a0`) — dumped verbatim from `cm0102_GDI.exe` with `pefile`.
//!
//! Every `DAT_*` VA below is relative to `ImageBase = 0x00400000`. Values
//! that fall inside the exe's `.data` BSS tail (raw size 0) are read back
//! as zero — those are runtime-initialised slots the loader zeroes before
//! the game reaches them. All six BSS-zero slots below stay at 0 on the
//! main-menu path (verified by the prior static dump); the widget
//! renderer's port therefore treats zero as their initial-state value.
//!
//! Regenerate with `dump_globals.py` if the binary changes — do NOT
//! hand-edit.

/// `DAT_009b88f8` — scalable-font flag. Read by `FUN_005d7aa0` block L to
/// decide between the traditional-font path (via `PixelFont`) and the
/// futuristic scalable-font branch (`FUN_0059b550`). Ships as `1` in
/// `cm0102_GDI.exe`, meaning "traditional-font path" is the default.
pub const DAT_009B88F8: u32 = 0x0000_0001;

/// `DAT_00acdec8` — colour-key replacement word. `FUN_005d7aa0` block A
/// swaps `widget.colour_a` for this value when `colour_a == DAT_00ad6b22`.
/// BSS-zero on shipped exe.
pub const DAT_00ACDEC8: u16 = 0x0000;

/// `DAT_00ad6b22` — colour-key match word. `FUN_005d7aa0` block A compares
/// `widget.colour_a` against this; equality triggers the colour-key swap
/// with `DAT_00acdec8`. BSS-zero on shipped exe.
pub const DAT_00AD6B22: u16 = 0x0000;

/// `DAT_00ad6b0c` — palette word blitted by the widget renderer's
/// stipple-block-I and block-J branches (drawn through `draw_stipple`
/// at the pattern's paint step). BSS-zero on shipped exe.
pub const DAT_00AD6B0C: u16 = 0x0000;

/// `DAT_00acdee4` — palette word used by block K (the elongated stipple).
/// BSS-zero on shipped exe.
pub const DAT_00ACDEE4: u16 = 0x0000;

/// `DAT_00acdeac` — surface pixel-format code. `FUN_005ce2d0` (colour
/// scale) and `FUN_005d03a0` (wrapped-text shadow sampling) dispatch on
/// this field of the pixel-format record: `0x7e0 == RGB565`, anything else
/// is RGB555. BSS-zero on shipped exe (i.e. the format record isn't
/// installed until the DDraw init writes it — the software-only paths
/// use the surface's own `green_mask` instead).
pub const DAT_00ACDEAC: u16 = 0x0000;

/// `DAT_00ad6b44` — colour-scale early-exit gate. `FUN_005ce2d0` returns 0
/// immediately when this is non-zero (skipping the scale). Also gates
/// FUN_005d03a0 wrapped-text and FUN_005ceaa0 glyph blit. BSS-zero on
/// shipped exe → the gate lets rendering proceed.
pub const DAT_00AD6B44: u32 = 0x0000_0000;
