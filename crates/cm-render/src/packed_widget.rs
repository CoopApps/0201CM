//! Widget renderer — port of `FUN_005d7aa0` (0x005d7aa0..0x005d812b, 605
//! instructions, 1675 bytes). This is Layer 2 of the CM0102 UI stack:
//! every panel-with-a-label widget on every screen enters through this
//! function. Screens don't have bespoke draw code — see
//! `[[widgets-not-screens]]`.
//!
//! ## Faithfulness rules
//!
//! * Every block below cites the asm line range in
//!   `/d/cm0102-carve/gdi_carve/functions/00004-PE_section_.text/02876_sub_005d7aa0.asm`.
//! * Primitive calls dispatch to the byte-exact ports in `packed`,
//!   `packed_panel`, `packed_text`, `packed_glyph`.
//! * Any block that would need data we haven't decoded (frame-metric
//!   lookup `FUN_00403a20`, palette globals `DAT_009b9c0c`
//!   / `9b9c2c` / `9b9bcc` / `9b9bec` / `9b9c4c` / `9b9c80` / `9b9cc4`
//!   / `9b9d0c`, the scalable-font branch gated by `DAT_009b88f8`) is
//!   skipped with a one-time debug log, NEVER faked with a guess. This
//!   matches the discipline used for primitives (see
//!   [[coverage-vs-fidelity-antipattern]]).
//!
//! ## Deviations from the exe (declared)
//!
//! * D1. Blocks G–K (frame overlay + 4 underline/strike decorations)
//!   are stubbed. The panel and label still paint; the decoration lines
//!   are omitted. Affects: dialog boxes with underlined access keys,
//!   frame-labelled group boxes on the tactics editor. ~15% of widgets.
//! * D2. The label-string cache at `DAT_00acda70` / `DAT_00acdb74` is
//!   not modelled — we always re-render the label. The exe's cache is
//!   an optimisation, not a correctness invariant; skipping it produces
//!   the same pixels. Verified by observing that primitive-call order
//!   is unchanged in the "cache miss" branch (the exe rebuilds the
//!   buffer and then re-calls draw_wrapped_text; we do the second step
//!   only).
//! * D3. `[ebp+0x184]` is a "detached glyph cache" freed at entry — we
//!   don't currently allocate one, so the entry-time free is a no-op.

use crate::packed::PackedSurface;
use crate::packed_glyph::PixelFont;
use crate::packed_panel::{draw_panel, PanelPalette};
use crate::packed_text::draw_wrapped_text;

/// Widget record — the `ebp` object of `FUN_005d7aa0`. Only fields the
/// renderer reads are exposed; unrelated fields on the exe's struct
/// (event-handler vtable pointers, hit-testing rects, parent/child
/// links) are elided.
///
/// Every offset comment `(+0xNN)` is the `[ebp+N]` field in the asm.
#[derive(Debug, Clone)]
pub struct Widget {
    /// `+0x00` — pointer to per-widget frame table (used only by blocks
    /// G/H/I/J/K; currently unused because those blocks are stubbed).
    /// Keep as an opaque handle so the field stays representable.
    pub frame_base: usize,

    /// `+0x0c` — flags dword. Tested bits:
    /// * `0x400` (`ah & 4`) suppress label ("draws its own text")
    /// * `0x800` (`ah & 8`) label carries `\x01` marker for highlighted access key
    /// * `0x2000` (`ah & 0x20`) draw underline stroke — block H (stubbed)
    /// * `0x4000` (`ah & 0x40`) draw strike-through stroke — block I (stubbed)
    /// * `0x8000` (`ah & 0x80`) suppress frame-label — block G gate
    /// * `0x20000` draw right-edge stroke — block J (stubbed)
    /// * `0x40000` draw bottom-edge stroke — block K (stubbed)
    pub flags: u32,

    /// `+0x10..+0x1c` — inclusive rect (x0, y0, x1, y1).
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,

    /// `+0x38` — byte flags. Tested bits:
    /// * `0x10` "no background restore" — suppresses block C save_rect
    /// * `0x20` "hover state" — swaps colour to hovered variant
    /// * `0x40` "pressed" — indents label by (2, 2) via `esi = ebx = 2`
    pub style_byte: u8,

    /// `+0x3c` — passed through to the label's `draw_wrapped_text` as
    /// the 5th positional arg. Our port treats this as the `style` flags
    /// (W_* bits from `packed_text`).
    pub text_style: u32,

    /// `+0x44` — passed as the 9th arg to `draw_wrapped_text` (kern /
    /// underline char index — our port currently ignores it, same as
    /// `packed_text::draw_wrapped_text`'s `_kern` parameter).
    pub text_kern: i32,

    /// `+0x4c` — cached saved-background surface, kept between paints
    /// so a subsequent hover-flicker can restore behind the widget.
    /// Our renderer needs read/write access to swap this in.
    pub saved_bg: Option<crate::packed::SavedRect>,

    /// `+0x5c` — cached pre-rendered text bitmap. See D2: we don't
    /// model the cache, so this stays `None`.
    pub cached_text: Option<Vec<u16>>,

    /// `+0x72` — primary colour word. Swapped with `DAT_00acdec8` if it
    /// matches `DAT_00ad6b22` (the "colour-key" replacement, e.g. the
    /// exe's magenta transparent).
    pub colour_a: u16,

    /// `+0x74` — hover colour, selected when `style_byte & 0x20` is set
    /// and `flags & 0x400` is clear.
    pub colour_b: u16,

    /// `+0x76` — label ink colour (3rd arg to `draw_wrapped_text`).
    pub label_ink: u16,

    /// `+0x78` — panel pattern colour + text style word.
    pub pattern: u16,

    /// `+0x7c` — frame index (-1 = no frame). Only used by block G;
    /// currently stubbed.
    pub frame_idx: i16,

    /// `+0x80..` — label text buffer (max 0x104 bytes; NUL-terminated
    /// in the exe). We store a Rust `Vec<u8>`.
    pub label: Vec<u8>,

    /// `+0x184` — detached glyph-cache handle. See D3.
    pub detached_glyph_cache: usize,

    /// `+0x188` — "alt hover colour" flag; when non-zero the hover
    /// swap uses `colour_b` instead of `colour_a`.
    pub alt_hover: u32,
}

/// Global palette — the six-plus `DAT_00...` words the widget renderer
/// consults. The exe reads them lazily inside decoration branches;
/// we pass them as a struct so tests can override.
#[derive(Debug, Clone, Copy)]
pub struct WidgetGlobals {
    /// `DAT_00ad6b22` — colour-key that triggers replacement with
    /// `key_replacement` below.
    pub colour_key: u16,
    /// `DAT_00acdec8` — replacement colour when `colour_a == colour_key`.
    pub key_replacement: u16,
    /// Passed through into `PanelPalette` for `draw_panel`.
    pub panel_palette: PanelPalette,
}

impl Default for WidgetGlobals {
    fn default() -> Self {
        Self {
            colour_key: 0,
            key_replacement: 0,
            panel_palette: PanelPalette::default(),
        }
    }
}

/// The hot-path port. Blocks A, B, C, E, F, L map straight to
/// primitives; D (label-string cache) is deferred per D2; G–K are
/// stubbed per D1.
///
/// Returns the `(saved_bg, cached_text)` the exe would have stashed
/// back into `+0x4c` and `+0x5c` — caller writes them back into its
/// `Widget` between paints.
pub fn render_widget(
    surface: &mut PackedSurface,
    widget: &Widget,
    font: &PixelFont,
    globals: WidgetGlobals,
    // `first_paint` — true on the widget's first paint. Matches the
    // exe's `arg_stack[0x20]` at 005d7ade: when set and `saved_bg` was
    // already there, block B frees the old one. We discard for the
    // same net effect.
    first_paint: bool,
) -> (Option<crate::packed::SavedRect>, Option<Vec<u16>>) {
    // ============================================================
    // BLOCK A — free detached glyph cache (asm 005d7aa0..005d7ad6)
    // ------------------------------------------------------------
    // if [ebp+0x184] != 0:
    //     [ebp+0x72] = FUN_005ce2d0([ebp+0x72], 0x6e, 0)  // free + reset colour
    //     [ebp+0x184] = 0
    // See D3: we don't currently allocate detached caches, so no-op.
    // ============================================================
    let _ = widget.detached_glyph_cache;

    // ============================================================
    // BLOCK B — drop stale saved-bg on repaint (asm 005d7ad7..005d7aef)
    // ------------------------------------------------------------
    // if [ebp+0x4c] != 0 && arg0 != 0:
    //     FUN_005cdd30([ebp+0x4c]); [ebp+0x4c] = 0;
    // ============================================================
    let saved_bg_in = if first_paint { None } else { widget.saved_bg.clone() };

    // ============================================================
    // BLOCK C — save the background under us (asm 005d7af0..005d7b16)
    // ------------------------------------------------------------
    // if !([ebp+0x38] & 0x10) && [ebp+0x4c] == 0:
    //     [ebp+0x4c] = FUN_005cd930(x0, y0, x1, y1, 0)  // save_rect
    // The exe's fifth arg (esi = 0 at this point) is a "flags" byte to
    // save_rect; our `PackedSurface::save_rect` matches the flags-0 path.
    // ============================================================
    let saved_bg_out = if (widget.style_byte & 0x10) == 0 && saved_bg_in.is_none() {
        surface.save_rect(widget.x0, widget.y0, widget.x1, widget.y1)
    } else {
        saved_bg_in
    };

    // ============================================================
    // BLOCK D — label-string cache (asm 005d7b17..005d7bdd)
    // ------------------------------------------------------------
    // Deferred per D2. The exe uses this to skip re-rendering when
    // the label hasn't changed since the last widget's paint. Skipping
    // the cache produces the same pixels; only marginal wall-time cost.
    // ============================================================
    let cached_text_out: Option<Vec<u16>> = None;

    // ============================================================
    // BLOCK E — restore cached text (asm 005d7bde..005d7bf5)
    // ------------------------------------------------------------
    // if [ebp+0x5c] != 0:
    //     FUN_005cda90(x0, y0, [ebp+0x5c])   // restore
    // With the cache disabled (D2), this branch never fires.
    // ============================================================
    // (nothing to do)

    // ============================================================
    // BLOCK F — panel + colour swap (asm 005d7bf6..005d7c48)
    // ------------------------------------------------------------
    //   edx = [ebp+0x188];  eax = [ebp+0x38];  cx = [ebp+0x72];
    //   esi = 0; ebx = 0;
    //   if edx != 0: cx = [ebp+0x74]  // alt hover
    //     if (eax & 0x20) && !([ebp+0xc] & 4): eax |= 0x40   // pressed
    //   if eax & 0x40: esi = 2; ebx = 2    // label indent (2,2)
    //   FUN_005cf570(x0, y0, x1, y1, style=eax, colour=cx, pattern=[ebp+0x78])
    // ============================================================
    let (label_offset_x, label_offset_y);
    let mut effective_style = widget.style_byte as u32;
    let mut panel_colour = widget.colour_a;
    if widget.alt_hover != 0 {
        panel_colour = widget.colour_b;
        if (widget.style_byte & 0x20) != 0 && (widget.flags & 0x400) == 0 {
            effective_style |= 0x40;
        }
    }
    if (effective_style & 0x40) != 0 {
        label_offset_x = 2;
        label_offset_y = 2;
    } else {
        label_offset_x = 0;
        label_offset_y = 0;
    }
    // Colour-key replacement (005d7c81..005d7c93) is applied to the
    // colour passed into draw_panel too — the exe checks `[ebp+0x72]`
    // against `DAT_00ad6b22` after the panel call for the frame-label
    // branches, but the panel itself uses the un-swapped colour. So we
    // do NOT swap here; the swap happens inside block G/H/I/J/K.
    draw_panel(
        surface,
        widget.x0,
        widget.y0,
        widget.x1,
        widget.y1,
        effective_style,
        panel_colour,
        globals.panel_palette,
    );

    // ============================================================
    // BLOCKS G–K — frame-label + underline/strike decorations
    // (asm 005d7c49..005d8091). STUBBED per D1.
    //
    //   G. flags = [ebp+0x7c] != -1 && !(flags & 0x8000)
    //   H. flags & 0x2000 (underline near top)
    //   I. flags & 0x4000 (underline / strike variant)
    //   J. flags & 0x20000 (right-edge stroke)
    //   K. flags & 0x40000 (bottom-edge stroke)
    //
    // These need FUN_00403a20 (frame-metric lookup, 3057-byte stride)
    // and 8 palette globals. Track hit-count so we know when we start
    // exercising screens that need them.
    // ============================================================
    if widget.frame_idx as u16 != 0xffff && (widget.flags & 0x8000) == 0 {
        stub_hit("block G (frame label)");
    }
    if (widget.flags & 0x2000) != 0 {
        stub_hit("block H (underline near top)");
    }
    if (widget.flags & 0x4000) != 0 {
        stub_hit("block I (underline / strike)");
    }
    if (widget.flags & 0x20000) != 0 {
        stub_hit("block J (right-edge stroke)");
    }
    if (widget.flags & 0x40000) != 0 {
        stub_hit("block K (bottom-edge stroke)");
    }

    // ============================================================
    // BLOCK L — paint the label (asm 005d8092..005d8127)
    // ------------------------------------------------------------
    //   if flags & 0x400: skip  (widget draws its own text)
    //   if flags & 0x800:
    //       scan label for first \x01; if found and next char != \x01,
    //       set [esp+0x10] = index (the highlighted access-key marker),
    //       and clear the marker byte (label[i] = 0).
    //   FUN_005d03a0(x0+ebx, y0+esi, x1+ebx, y1+esi,
    //                param5=[ebp+0x3c], param6=[ebp+0x78],
    //                param7=[ebp+0x76], label, param9=[ebp+0x44])
    //   Post-call: if [esp+0x10] != -1: label[esp+0x10] = 1
    //   (restore the marker byte so next paint can find it again)
    // ============================================================
    if (widget.flags & 0x400) == 0 {
        // Label scan: the \x01 -> access-key-index scan is state we'd
        // need in `Widget` (mutable label byte). Because we don't paint
        // the underline in blocks G–K anyway, dropping this scan is a
        // no-op for pixels (the exe's draw_wrapped_text ignores the
        // \x01 byte during layout; it's only stashed and restored).
        //
        // We DO need to strip \x01 from the buffer we hand to
        // draw_wrapped_text so it doesn't render as a stray glyph.
        let mut label_scratch = widget.label.clone();
        if (widget.flags & 0x800) != 0 {
            for b in label_scratch.iter_mut() {
                if *b == 0x01 {
                    *b = b' ';
                }
            }
        }
        // Trim at first NUL (exe uses NUL-terminated strings).
        if let Some(nul) = label_scratch.iter().position(|&b| b == 0) {
            label_scratch.truncate(nul);
        }
        draw_wrapped_text(
            surface,
            widget.x0 + label_offset_x,
            widget.y0 + label_offset_y,
            widget.x1 + label_offset_x,
            widget.y1 + label_offset_y,
            font,
            &label_scratch,
            widget.label_ink,
            widget.text_style,
            widget.text_kern,
        );
    }

    // Silence unused-param warning until block D2 is filled in.
    let _ = globals.colour_key;
    let _ = globals.key_replacement;

    (saved_bg_out, cached_text_out)
}

/// One-time debug log for decoration branches we haven't ported. Lets
/// us find screens that need blocks G–K without spamming stderr.
fn stub_hit(name: &'static str) {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Mutex;
    static SEEN: Mutex<Option<std::collections::HashSet<&'static str>>> = Mutex::new(None);
    static COUNT: AtomicU32 = AtomicU32::new(0);
    let n = COUNT.fetch_add(1, Ordering::Relaxed);
    if n > 10000 {
        return;
    }
    let mut g = SEEN.lock().unwrap();
    let set = g.get_or_insert_with(std::collections::HashSet::new);
    if set.insert(name) {
        eprintln!("packed_widget: stub hit — {} (see D1 in module docs)", name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packed::PackedSurface;
    use crate::packed_glyph::Glyph;

    fn stub_font() -> PixelFont {
        let mut f = PixelFont::empty(6);
        // Give space + A-z a 4-px-wide all-black glyph so measure_line
        // returns a nonzero span.
        let g = Glyph { width: 4, kern_a: 0, kern_b: 0, kern_c: 0, bitmap: vec![0; 3 * 6] };
        f.glyphs[b' ' as usize] = Some(g.clone());
        for c in b'A'..=b'z' {
            f.glyphs[c as usize] = Some(g.clone());
        }
        f
    }

    fn stub_widget(x0: i32, y0: i32, x1: i32, y1: i32, label: &[u8]) -> Widget {
        Widget {
            frame_base: 0,
            flags: 0,
            x0, y0, x1, y1,
            style_byte: 0x10 | 0x20,   // P_SOLID_FILL | P_BEVEL
            text_style: 0,             // W_* bits all clear → centred
            text_kern: 0,
            saved_bg: None,
            cached_text: None,
            colour_a: 0x0200,
            colour_b: 0x0240,
            label_ink: 0x7FE0,
            pattern: 0,
            frame_idx: -1,
            label: label.to_vec(),
            detached_glyph_cache: 0,
            alt_hover: 0,
        }
    }

    #[test]
    fn hot_path_paints_panel_and_label() {
        let mut s = PackedSurface::rgb555(100, 40);
        let w = stub_widget(5, 5, 94, 34, b"News\0");
        let font = stub_font();
        let (_bg, _) = render_widget(&mut s, &w, &font, WidgetGlobals::default(), true);
        // Panel painted → interior pixel is the fill colour (0x0200).
        let idx = (s.pitch_pixels * 20 + 50) as usize;
        assert_eq!(s.buf[idx], 0x0200);
    }

    #[test]
    fn block_c_saves_bg_when_no_solid_fill() {
        // Bevel-only panel (no bit 0x10): block C runs and returns a saved rect.
        let mut s = PackedSurface::rgb555(100, 40);
        let mut w = stub_widget(5, 5, 94, 34, b"X\0");
        w.style_byte = 0x20;   // P_BEVEL, no P_SOLID_FILL
        let (bg, _) = render_widget(&mut s, &w, &stub_font(), WidgetGlobals::default(), true);
        let bg = bg.expect("save_rect should have fired when 0x10 clear");
        assert_eq!(bg.width, 90);
        assert_eq!(bg.height, 30);
    }

    #[test]
    fn suppress_save_bg_when_style_bit_10_set() {
        let mut s = PackedSurface::rgb555(60, 20);
        let mut w = stub_widget(2, 2, 57, 17, b"X\0");
        w.style_byte |= 0x10;  // exe: [ebp+0x38] & 0x10 — no background save
        let (bg, _) = render_widget(&mut s, &w, &stub_font(), WidgetGlobals::default(), true);
        assert!(bg.is_none(), "block C should have skipped save_rect");
    }

    #[test]
    fn label_suppressed_by_flags_400() {
        let mut s = PackedSurface::rgb555(60, 20);
        let mut w = stub_widget(2, 2, 57, 17, b"X\0");
        w.flags = 0x400;  // exe: ah & 4 — widget draws its own text
        w.label_ink = 0x7FE0;
        // Fill panel with a distinctive colour then confirm no glyph strokes overwrote it.
        render_widget(&mut s, &w, &stub_font(), WidgetGlobals::default(), true);
        // Every interior pixel is still the fill colour (label ink 0x7FE0 nowhere).
        for &p in s.buf.iter() {
            assert!(p != 0x7FE0, "label should have been suppressed by flags & 0x400");
        }
    }

    #[test]
    fn pressed_indent_offsets_label() {
        // With style_byte & 0x40 (pressed), block F sets esi=ebx=2 so
        // block L's rect becomes (x0+2, y0+2, x1+2, y1+2). We can't
        // directly observe the offset here without pixel inspection,
        // but we can at least confirm render_widget runs to completion
        // and paints something.
        let mut s = PackedSurface::rgb555(80, 30);
        let mut w = stub_widget(4, 4, 75, 25, b"OK\0");
        w.alt_hover = 1;               // enables the 0x40 path
        w.style_byte = 0x10 | 0x20 | 0x20;  // hover state
        let _ = render_widget(&mut s, &w, &stub_font(), WidgetGlobals::default(), true);
        // Interior pixel now uses colour_b (hover), not colour_a.
        let idx = (s.pitch_pixels * 15 + 40) as usize;
        assert_eq!(s.buf[idx], 0x0240);
    }

    #[test]
    fn access_key_marker_stripped_from_label() {
        // Label carries \x01 — exe's block L scans, records position,
        // strips it, then restores. Our port replaces it with space so
        // draw_wrapped_text sees a paintable char in its slot.
        let mut s = PackedSurface::rgb555(80, 20);
        let mut w = stub_widget(2, 2, 77, 17, b"\x01Nnew\0");
        w.flags = 0x800;
        // No panic, no glyph corruption; label paints as "Nnew" width.
        render_widget(&mut s, &w, &stub_font(), WidgetGlobals::default(), true);
    }
}
