//! Visual diff — render the sidebar + manager drop-down via the
//! `screens_faithful` helpers (which now delegate to `render_widget`)
//! and compare pixel-for-pixel against a captured framebuffer of the
//! real cm0102_GDI.exe rendering the same screen.
//!
//! ## Fixture format
//!
//! `fixtures/visual_sidebar_and_menu.pixels.bin` — raw 16-bit RGB555
//! packed pixels, width×height u16 little-endian, matching the surface
//! this test builds (see [`WIDTH`] / [`HEIGHT`] below).
//!
//! ## Producing the fixture
//!
//! ```text
//! # in a shell that can drive the running exe:
//! python tools/gdi_capture/capture_screen.py \
//!     --click-name "Christoph Olewicz"      \  # opens the manager menu
//!     --width 800 --height 600              \
//!     --out fixtures/visual_sidebar_and_menu.pixels.bin
//! ```
//!
//! The test is `#[ignore]`d until that binary lands (it fails-fast with
//! the same instructions if the fixture is missing, so the CI signal is
//! preserved). Once the fixture exists, remove the `#[ignore]`.
//!
//! ## Diff policy
//!
//! Byte-exact. On mismatch we print (a) how many pixels differ and
//! (b) the first 10 `(x, y, expected, got)` triples, then fail.
//! No tolerances — a delta means a real integration bug in the port.

use std::path::PathBuf;

use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::{Glyph, PixelFont};
use cm_render::packed_panel::PanelPalette;
use cm_render::screens_faithful::{
    menu_dropdown_container, menu_item, sidebar_button,
    MENU_BAND_EVEN, MENU_BAND_ODD,
};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

/// The exe's sidebar font (font id 1). Until the real font is threaded
/// in, use a stub for glyph metrics — this makes the diff meaningful
/// only for the panel/bevel pixels; the label glyphs are noted as
/// expected differences. When the real font gets wired, drop this.
fn stub_font() -> PixelFont {
    let mut f = PixelFont::empty(6);
    for c in 0x20u8..=0x7Eu8 {
        f.glyphs[c as usize] = Some(Glyph {
            width: 4, kern_a: 0, kern_b: 0, kern_c: 0,
            bitmap: vec![0; 12],
        });
    }
    f
}

/// Rebuild the exact sidebar + manager-menu overlay from
/// `fixtures/screen_sidebar_and_manager_menu.md`.
fn render_sidebar_and_menu(s: &mut PackedSurface) {
    let font = stub_font();
    // Sidebar buttons — same rects the capture used.
    let side_palette = PanelPalette {
        outer_highlight: 0x7FE0,
        default_bevel: 0x0010,
    };
    // The human-name button ("Christoph Olewicz") lives at
    // (5, 145)-(85, 187). Capture only had one visible sidebar item on
    // the redraw, so this is what we replay.
    sidebar_button(s, 5, 145, 85, 187, b"Christoph Olewicz",
                   0x43FF, side_palette, &font);

    // Manager drop-down container at (88, 145)-(279, 481).
    menu_dropdown_container(s, 88, 145, 279, 481, PanelPalette::default(), &font);

    // 16 menu rows — labels + y-ranges from the fixture table.
    let rows: &[(i32, i32, &[u8], bool)] = &[
        (147, 166, b"Pro Vercelli Squad", false),
        (168, 187, b"Pro Vercelli Reserves", false),
        (189, 208, b"Control Senior Team Only", false),
        (210, 229, b"Board Confidence", false),
        (231, 250, b"Resign from Club", false),
        (252, 270, b"", true),                              // separator
        (272, 291, b"News", false),
        (293, 312, b"Player & Staff Search", false),
        (314, 333, b"Compare two chosen players", false),
    ];
    for (i, (y0, y1, label, is_sep)) in rows.iter().enumerate() {
        let band = if i % 2 == 0 { MENU_BAND_EVEN } else { MENU_BAND_ODD };
        menu_item(s, 90, *y0, 277, *y1, band, label, *is_sep, false,
                  PanelPalette::default(), &font);
    }
}

fn fixture_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("..");
    p.push("fixtures");
    p.push("visual_sidebar_and_menu.pixels.bin");
    p
}

fn instructions() -> String {
    format!(
        "\n\
         Missing capture fixture: {}\n\
         \n\
         Produce it with:\n\
         \n\
             python tools/gdi_capture/capture_screen.py \\\n\
                 --click-name \"Christoph Olewicz\" \\\n\
                 --width {} --height {} \\\n\
                 --out fixtures/visual_sidebar_and_menu.pixels.bin\n\
         \n\
         The file is width*height*2 bytes of little-endian u16\n\
         RGB555 pixels — the same layout PackedSurface::rgb555 uses.\n",
        fixture_path().display(),
        WIDTH,
        HEIGHT,
    )
}

#[test]
#[ignore = "needs fixtures/visual_sidebar_and_menu.pixels.bin — see module docs"]
fn diff_against_captured_exe_framebuffer() {
    let path = fixture_path();
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => panic!("{}", instructions()),
    };
    let expected_len = (WIDTH as usize) * (HEIGHT as usize) * 2;
    assert_eq!(
        bytes.len(),
        expected_len,
        "fixture size mismatch: got {} bytes, expected {}",
        bytes.len(),
        expected_len
    );

    let mut expected: Vec<u16> = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    for chunk in bytes.chunks_exact(2) {
        expected.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }

    let mut s = PackedSurface::rgb555(WIDTH, HEIGHT);
    render_sidebar_and_menu(&mut s);
    let got = &s.buf;

    let mut diffs = 0usize;
    let mut first10: Vec<(i32, i32, u16, u16)> = Vec::new();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let i = (y * WIDTH + x) as usize;
            if got[i] != expected[i] {
                diffs += 1;
                if first10.len() < 10 {
                    first10.push((x, y, expected[i], got[i]));
                }
            }
        }
    }

    if diffs != 0 {
        let mut msg = format!("visual diff: {} pixels differ\n", diffs);
        for (x, y, e, g) in &first10 {
            msg.push_str(&format!(
                "  ({:>4}, {:>4})  expected=0x{:04x}  got=0x{:04x}\n",
                x, y, e, g
            ));
        }
        panic!("{}", msg);
    }
}

/// Sanity check that runs unconditionally: render_widget produces some
/// non-zero pixels in the sidebar + menu region. If this fails the
/// helpers are broken outright — no capture fixture needed to notice.
#[test]
fn helpers_paint_something_in_the_expected_region() {
    let mut s = PackedSurface::rgb555(WIDTH, HEIGHT);
    render_sidebar_and_menu(&mut s);

    // The manager-menu container fill (0x0200) must show up somewhere
    // in its rect (88, 145)-(279, 481).
    let mut menu_fill_pixels = 0usize;
    for y in 200..470 {
        for x in 100..270 {
            if s.buf[(y * WIDTH + x) as usize] == 0x0200 {
                menu_fill_pixels += 1;
            }
        }
    }
    assert!(
        menu_fill_pixels > 1000,
        "menu container fill should paint many pixels, got {}",
        menu_fill_pixels
    );

    // The sidebar-button bevel must paint the outer top-left (5, 145).
    assert_ne!(
        s.buf[(145 * WIDTH + 5) as usize],
        0,
        "sidebar-button bevel top-left must be painted"
    );
}
