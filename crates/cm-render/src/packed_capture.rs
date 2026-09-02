//! Capture fixture format + replay dispatcher — the harness that
//! validates the packed-16bpp primitive layer against real output from
//! cm0102.exe / cm0102_GDI.exe. See [[gdi-renderer-is-ground-truth]].
//!
//! A fixture pairs a `before` framebuffer, an `after` framebuffer, and
//! the ordered sequence of primitive calls the exe made between them.
//! `replay(fixture)` loads `before` into a `PackedSurface`, dispatches
//! each call through the ported functions, and compares the resulting
//! buffer to `after`. A single differing u16 fails the fixture and
//! reports the first mismatch — that is the byte-exact contract.
//!
//! The Frida hook script that produces these fixtures lives at
//! `tools/gdi_capture/capture.js`; see its README for the run recipe.
//!
//! Fixture files are JSON. Framebuffers are base64-encoded raw little-
//! endian u16 streams (`width * height * 2` bytes each). Keeping the
//! format text-only avoids the binary-sidecar bookkeeping that has
//! historically bit-rotted this project.

use crate::packed::PackedSurface;
use crate::packed_glyph::{draw_glyph, draw_text, Glyph, PixelFont};
use crate::packed_panel::{draw_panel, PanelPalette};
use crate::packed_text::{draw_wrapped_text, W_LEFT, W_RIGHT, W_TOP};
use base64::Engine;
use serde::{Deserialize, Serialize};

/// Format + identity metadata for a capture. All fields correspond to
/// exe globals the Frida script reads once per capture:
/// - `pitch_pixels` = `DAT_00acdeb8`, width in pixels not bytes.
/// - `red/green/blue_mask` = `DAT_00acdea8 / eac / eb0`.
/// - `width` / `height` = `DAT_00ad6b40 / DAT_00ad6b08` (from init).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureMeta {
    /// `"GDI"` or `"DirectDraw"` — the binary the capture came from.
    pub exe: String,
    /// Short human-readable label for what screen was being drawn.
    pub screen: String,
    pub width: i32,
    pub height: i32,
    pub pitch_pixels: i32,
    pub red_mask: u16,
    pub green_mask: u16,
    pub blue_mask: u16,
}

/// One primitive call the exe made during the capture window. `args` are
/// preserved as-is in the exe's coordinate system; the replay dispatcher
/// applies no coordinate transformation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Call {
    /// `FUN_005cd3e0(x0, y0, x1, y1, style, colour)`.
    Line { x0: i32, y0: i32, x1: i32, y1: i32, style: u32, colour: u16 },
    /// `FUN_005cd730(x0, y0, x1, y1, style, colour)`.
    Rect { x0: i32, y0: i32, x1: i32, y1: i32, style: u32, colour: u16 },
    /// `FUN_005cdd60(x0, y0, x1, y1)`.
    Darken { x0: i32, y0: i32, x1: i32, y1: i32 },
    /// `FUN_005cf570(x0, y0, x1, y1, style, colour[, pattern])`.
    Panel {
        x0: i32, y0: i32, x1: i32, y1: i32,
        style: u32, colour: u16,
        #[serde(default)] outer_highlight: u16,
        #[serde(default)] default_bevel: u16,
    },
    /// `FUN_005ceaa0(x, y, font, colour, string, underline_at)`. `font`
    /// carries the height + glyphs inline so the fixture is self-
    /// contained (the capture reads the exe's font table by index).
    Text {
        x: i32, y: i32,
        colour: u16,
        text: Vec<u8>,
        font: FontDump,
    },
    /// `FUN_005d03a0(x0, y0, x1, y1, style, font, colour, string, kern)`.
    WrappedText {
        x0: i32, y0: i32, x1: i32, y1: i32,
        style: u32, colour: u16,
        text: Vec<u8>,
        font: FontDump,
    },
    /// `FUN_005cda90(x, y, saved_rect)` — image blit / restore.
    Restore {
        x: i32, y: i32,
        width: i32, height: i32,
        /// Base64 of the saved-rect pixels, little-endian u16.
        data_b64: String,
    },
}

/// Font contents inlined into a fixture (avoids a shared-font side
/// dependency during replay). Each glyph is `{width, bitmap_b64}` where
/// `bitmap_b64` is the 4-bit-nibble-packed row-major bitmap in base64.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontDump {
    pub height: i32,
    pub glyphs: Vec<Option<GlyphDump>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlyphDump {
    pub width: i32,
    pub bitmap_b64: String,
}

impl FontDump {
    fn into_font(&self) -> PixelFont {
        let mut f = PixelFont::empty(self.height);
        for (i, g) in self.glyphs.iter().enumerate() {
            if let Some(g) = g {
                let bitmap = base64::engine::general_purpose::STANDARD
                    .decode(&g.bitmap_b64)
                    .unwrap_or_default();
                if i < f.glyphs.len() {
                    f.glyphs[i] = Some(Glyph { width: g.width, bitmap });
                }
            }
        }
        f
    }
}

/// A whole capture — metadata, before/after framebuffers (base64 raw
/// little-endian u16), and the primitive calls between them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fixture {
    pub meta: FixtureMeta,
    pub before_b64: String,
    pub after_b64: String,
    pub calls: Vec<Call>,
}

/// A replay mismatch — the first differing pixel is enough to fail a
/// fixture; we carry a small context window for debugging.
#[derive(Debug, Clone)]
pub struct ReplayMismatch {
    pub first_diff_index: usize,
    pub x: i32,
    pub y: i32,
    pub expected: u16,
    pub actual: u16,
    pub total_diffs: usize,
}

impl std::fmt::Display for ReplayMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "first diff at ({x}, {y}) [idx {i}]: expected {e:#06x}, got {a:#06x} — {n} pixel(s) differ overall",
            x = self.x, y = self.y, i = self.first_diff_index,
            e = self.expected, a = self.actual, n = self.total_diffs,
        )
    }
}

/// Replay one fixture: build a `PackedSurface` from `meta` and `before`,
/// dispatch every `Call`, then compare to `after`. Returns the replayed
/// surface on success, or the first-diff report on failure.
pub fn replay(fixture: &Fixture) -> Result<PackedSurface, ReplayMismatch> {
    let meta = &fixture.meta;
    let before = decode_u16s(&fixture.before_b64);
    let after = decode_u16s(&fixture.after_b64);
    let expected_len = (meta.pitch_pixels * meta.height) as usize;
    assert_eq!(before.len(), expected_len, "before framebuffer length");
    assert_eq!(after.len(), expected_len, "after framebuffer length");
    let mut s = PackedSurface {
        buf: before,
        width: meta.width,
        height: meta.height,
        pitch_pixels: meta.pitch_pixels,
        red_mask: meta.red_mask,
        green_mask: meta.green_mask,
        blue_mask: meta.blue_mask,
    };
    for call in &fixture.calls {
        dispatch(&mut s, call);
    }
    compare(&s, &after)?;
    Ok(s)
}

fn dispatch(s: &mut PackedSurface, call: &Call) {
    match call {
        Call::Line { x0, y0, x1, y1, style, colour } => {
            s.draw_line(*x0, *y0, *x1, *y1, *style, *colour);
        }
        Call::Rect { x0, y0, x1, y1, style, colour } => {
            s.draw_rectangle(*x0, *y0, *x1, *y1, *style, *colour);
        }
        Call::Darken { x0, y0, x1, y1 } => {
            s.darken_rect(*x0, *y0, *x1, *y1);
        }
        Call::Panel { x0, y0, x1, y1, style, colour, outer_highlight, default_bevel } => {
            let pal = PanelPalette {
                outer_highlight: *outer_highlight,
                default_bevel: *default_bevel,
            };
            draw_panel(s, *x0, *y0, *x1, *y1, *style, *colour, pal);
        }
        Call::Text { x, y, colour, text, font } => {
            let f = font.into_font();
            draw_text(s, *x, *y, &f, text, *colour);
        }
        Call::WrappedText { x0, y0, x1, y1, style, colour, text, font } => {
            let f = font.into_font();
            draw_wrapped_text(s, *x0, *y0, *x1, *y1, &f, text, *colour, *style, 0);
        }
        Call::Restore { x, y, width, height, data_b64 } => {
            let data = decode_u16s(data_b64);
            let saved = crate::packed::SavedRect {
                width: *width,
                height: *height,
                data,
            };
            s.restore_rect(*x, *y, &saved);
        }
    }
    // Referenced style constants so the imports don't warn under
    // conditional codepaths that only appear in captured fixtures.
    let _ = (W_LEFT, W_RIGHT, W_TOP);
}

fn compare(s: &PackedSurface, after: &[u16]) -> Result<(), ReplayMismatch> {
    let mut first: Option<usize> = None;
    let mut total = 0usize;
    for (i, (&a, &b)) in s.buf.iter().zip(after.iter()).enumerate() {
        if a != b {
            if first.is_none() {
                first = Some(i);
            }
            total += 1;
        }
    }
    match first {
        None => Ok(()),
        Some(i) => Err(ReplayMismatch {
            first_diff_index: i,
            x: (i as i32) % s.pitch_pixels,
            y: (i as i32) / s.pitch_pixels,
            expected: after[i],
            actual: s.buf[i],
            total_diffs: total,
        }),
    }
}

/// Encode a u16 buffer as base64 little-endian bytes — the on-disk form
/// for `before_b64` / `after_b64` and `Restore.data_b64`. Exposed so the
/// Frida-side capture tooling on Windows can produce identical strings
/// by shelling out to the same encoder or replicating the transform.
pub fn encode_u16s(buf: &[u16]) -> String {
    let mut bytes = Vec::with_capacity(buf.len() * 2);
    for &v in buf {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Inverse of `encode_u16s`. Malformed base64 yields an empty vec so a
/// bad fixture surfaces as a length mismatch inside `replay`.
pub fn decode_u16s(b64: &str) -> Vec<u16> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .unwrap_or_default();
    bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect()
}

/// Encode a `Glyph` bitmap to the fixture's `bitmap_b64`.
pub fn encode_bitmap(bitmap: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bitmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end: build a synthetic fixture from primitives whose
    /// output we already know (via the byte-exact tests in packed.rs /
    /// packed_panel.rs), then round-trip through the replay dispatcher
    /// and assert the after buffer matches.
    #[test]
    fn synthetic_fixture_replays_byte_exact() {
        // Ground-truth build: a fresh 6×4 surface with a solid-red 4×2
        // rectangle in the middle — matches `rectangle_fill_writes_exact_u16_grid`.
        let mut expected = PackedSurface::rgb555(6, 4);
        let c = expected.pack_rgb(0xff, 0, 0);
        expected.draw_rectangle(1, 1, 4, 2, 0, c);

        let before = PackedSurface::rgb555(6, 4);
        let fixture = Fixture {
            meta: FixtureMeta {
                exe: "synthetic".into(),
                screen: "rectangle_fill".into(),
                width: 6, height: 4, pitch_pixels: 6,
                red_mask: 0x7c00, green_mask: 0x03e0, blue_mask: 0x001f,
            },
            before_b64: encode_u16s(&before.buf),
            after_b64: encode_u16s(&expected.buf),
            calls: vec![Call::Rect {
                x0: 1, y0: 1, x1: 4, y1: 2, style: 0, colour: c,
            }],
        };

        let replayed = replay(&fixture).expect("replay must match");
        assert_eq!(replayed.buf, expected.buf);
    }

    #[test]
    fn replay_reports_first_diff_when_a_call_is_wrong() {
        // Same setup but the fixture's after buffer says the rectangle
        // is blue when we replay red — the mismatch report must name
        // the first pixel that disagrees.
        let mut wrong = PackedSurface::rgb555(6, 4);
        let blue = wrong.pack_rgb(0, 0, 0xff);
        wrong.draw_rectangle(1, 1, 4, 2, 0, blue);
        let before = PackedSurface::rgb555(6, 4);
        let red = before.pack_rgb(0xff, 0, 0);
        let fixture = Fixture {
            meta: FixtureMeta {
                exe: "synthetic".into(),
                screen: "wrong_colour".into(),
                width: 6, height: 4, pitch_pixels: 6,
                red_mask: 0x7c00, green_mask: 0x03e0, blue_mask: 0x001f,
            },
            before_b64: encode_u16s(&before.buf),
            after_b64: encode_u16s(&wrong.buf),
            calls: vec![Call::Rect {
                x0: 1, y0: 1, x1: 4, y1: 2, style: 0, colour: red,
            }],
        };
        let err = replay(&fixture).expect_err("must fail");
        // First diff is at (1, 1) — pixel index 7 in a 6-wide surface.
        assert_eq!(err.first_diff_index, 7);
        assert_eq!((err.x, err.y), (1, 1));
        assert_eq!(err.expected, blue);
        assert_eq!(err.actual, red);
        assert_eq!(err.total_diffs, 8); // 4×2 rectangle
    }

    #[test]
    fn fixture_roundtrips_through_json() {
        // A fixture serialised to JSON and back deserialises identically
        // — protects the format from silent drift.
        let before = PackedSurface::rgb555(2, 1);
        let after = before.clone();
        let fixture = Fixture {
            meta: FixtureMeta {
                exe: "GDI".into(), screen: "noop".into(),
                width: 2, height: 1, pitch_pixels: 2,
                red_mask: 0x7c00, green_mask: 0x03e0, blue_mask: 0x001f,
            },
            before_b64: encode_u16s(&before.buf),
            after_b64: encode_u16s(&after.buf),
            calls: vec![],
        };
        let json = serde_json::to_string(&fixture).unwrap();
        let round: Fixture = serde_json::from_str(&json).unwrap();
        assert_eq!(round, fixture);
    }

    #[test]
    fn multi_call_fixture_dispatches_in_order() {
        // A rectangle fill followed by a solid frame in a contrasting
        // colour — order matters (frame paints over fill).
        let mut expected = PackedSurface::rgb555(6, 4);
        let fill = expected.pack_rgb(0xff, 0, 0);
        let frame = expected.pack_rgb(0, 0, 0xff);
        expected.draw_rectangle(0, 0, 5, 3, 0, fill);
        expected.draw_rectangle(0, 0, 5, 3, 2, frame);

        let before = PackedSurface::rgb555(6, 4);
        let fixture = Fixture {
            meta: FixtureMeta {
                exe: "synthetic".into(),
                screen: "fill_then_frame".into(),
                width: 6, height: 4, pitch_pixels: 6,
                red_mask: 0x7c00, green_mask: 0x03e0, blue_mask: 0x001f,
            },
            before_b64: encode_u16s(&before.buf),
            after_b64: encode_u16s(&expected.buf),
            calls: vec![
                Call::Rect { x0: 0, y0: 0, x1: 5, y1: 3, style: 0, colour: fill },
                Call::Rect { x0: 0, y0: 0, x1: 5, y1: 3, style: 2, colour: frame },
            ],
        };
        let replayed = replay(&fixture).expect("replay must match");
        assert_eq!(replayed.buf, expected.buf);
    }
}
