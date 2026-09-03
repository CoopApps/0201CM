//! Byte-diff test — render News through `screen_news::build_news_screen`
//! + the Layer 2 pipeline (`render_widget`), compare u16-for-u16 against
//! a captured after-framebuffer from `cm0102_GDI.exe`. This is the
//! forcing function that lands the "no wired screen without a capture"
//! rule (see [[coverage-vs-fidelity-antipattern]]).
//!
//! # Producing the fixture
//!
//! This test needs one binary file at
//! `fixtures/news_after.pixels.bin` — raw little-endian u16, exactly
//! `pitch_pixels * height * 2` bytes, matching the RGB555 framebuffer
//! `cm0102_GDI.exe` presents when the News screen is open.
//!
//! With the game running:
//!
//! ```text
//! # 1. Arm the streaming logger with framebuffer capture on.
//! D:/Python312/python.exe tools/gdi_capture/live_log.py \
//!     fixtures/news_capture_fb.jsonl.gz \
//!     --capture-framebuffers \
//!     --max-seconds 12 --silence-seconds 3
//!
//! # 2. In the game window, navigate away from News, then click News.
//! #    The recorder captures the full transition + several idle frames.
//!
//! # 3. Extract the last idle-News frame's raw pixels (present index N
//! #    depends on how many repaints landed — the analyser lists them).
//! D:/Python312/python.exe tools/gdi_capture/analyse_news_capture.py \
//!     fixtures/news_capture_fb.jsonl.gz \
//!     --dump-pixels 5 fixtures/news_after.pixels.bin
//! ```
//!
//! The Layer 2 pipeline is still converging; when this test starts
//! passing, News is byte-exact for real. Until then the diff report is
//! the score card that says how much drift is left.
//!
//! # No fuzz
//!
//! Any single differing u16 fails. The report names the first 10 diffs
//! (x, y, expected, got) so a mismatch is diagnosable at a glance.

use std::path::PathBuf;

use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::PixelFont;
use cm_render::packed_widget::{render_widget, Widget, WidgetGlobals};
use cm_render::packed_panel::PanelPalette;
use cm_render::screen_news::{build_news_screen, NewsItem, NewsScreenState};
use cm_render::widget_pool::GuiRecordPool;

const SCREEN_W: i32 = 800;
const SCREEN_H: i32 = 600;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .join("news_after.pixels.bin")
}

/// A News state that mirrors the shipped populated capture used in
/// `fixtures/news_capture.jsonl.gz` — same headlines and dates so
/// glyph positions and word-wrap boundaries line up when a real
/// after-buffer is dropped in.
fn news_state_matching_capture() -> NewsScreenState {
    NewsScreenState {
        header_title: "Christoph Olewicz News".into(),
        active_tab: 0,
        filter_text: String::new(),
        next_unread_enabled: true,
        items: vec![
            NewsItem { headline: "Andorno resumes full training".into(),
                       unread: true, date: "Tue 9th Oct AM".into() },
            NewsItem { headline: "Pro Vercelli march on".into(),
                       unread: true, date: "Mon 8th Oct EVE".into() },
            NewsItem { headline: "Genoa march on".into(),
                       unread: true, date: "Mon 8th Oct EVE".into() },
            NewsItem { headline: "Board reaction to Pro Sesto game".into(),
                       unread: false, date: "Sun 7th Oct PM".into() },
            NewsItem { headline: "World Cup 2002 qualifiers".into(),
                       unread: false, date: "Sat 6th Oct EVE".into() },
        ],
        selected_body: "Davide Andorno has begun full training \
                        following his thigh injury.".into(),
        back_disabled: false,
        next_enabled: true,
    }
}

/// Load raw u16 LE from disk. Returns None if the file is missing —
/// the caller is expected to `#[ignore]`-guard around that.
fn load_expected_after() -> Option<Vec<u16>> {
    let bytes = std::fs::read(fixture_path()).ok()?;
    let expected_bytes = (SCREEN_W as usize) * (SCREEN_H as usize) * 2;
    assert_eq!(
        bytes.len(), expected_bytes,
        "fixture size wrong: got {} bytes, expected {} (raw u16 LE, 800x600 RGB555)",
        bytes.len(), expected_bytes,
    );
    Some(bytes.chunks_exact(2)
         .map(|c| u16::from_le_bytes([c[0], c[1]]))
         .collect())
}

/// Render every widget in the pool onto `surface`. Areas are ignored
/// for now — the exe's area-background paint routine is a distinct
/// function (`FUN_005CE3E0` container repaint) that this port does not
/// yet drive from `render_widget`. When this test starts diverging in
/// area-background regions, that's the next port to land.
fn render_all_widgets(surface: &mut PackedSurface, pool: &mut GuiRecordPool,
                      font: &PixelFont) {
    let n = pool.widgets.len();
    for i in 0..n {
        // Widget → private Widget rec for render_widget.
        let d = pool.widgets[i].descriptor.clone();
        let mut w = Widget {
            frame_base: 0,
            flags: d.kind,
            x0: d.grid_x0,
            y0: d.grid_y0,
            x1: d.grid_x1,
            y1: d.grid_y1,
            style_byte: 0x30,          // panel bevel + solid fill (news default)
            text_style: 0x0c,          // wrapped-text style captured on news labels
            text_kern: -1,
            saved_bg: None,
            cached_text: None,
            colour_a: 0x0200,          // panel band ink — approximate
            colour_b: 0,
            label_ink: d.label_ink,
            pattern: 0,
            frame_idx: -1,
            label: {
                let mut b = d.text.as_bytes().to_vec();
                b.push(0);
                b
            },
            detached_glyph_cache: 0,
            alt_hover: 0,
        };
        let palette = PanelPalette { outer_highlight: 0x7fff, default_bevel: 0x39e7 };
        render_widget(surface, &mut w, Some(pool), font,
                      WidgetGlobals { panel_palette: palette }, true);
    }
}

fn report_first_diffs(actual: &[u16], expected: &[u16]) -> String {
    let mut out = String::new();
    let mut n_diff = 0usize;
    let mut first: Vec<(i32, i32, u16, u16)> = Vec::new();
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        if a != e {
            n_diff += 1;
            if first.len() < 10 {
                let x = (i as i32) % SCREEN_W;
                let y = (i as i32) / SCREEN_W;
                first.push((x, y, *e, *a));
            }
        }
    }
    out.push_str(&format!(
        "{} of {} pixels differ ({:.3}%)\n",
        n_diff, actual.len(),
        100.0 * n_diff as f64 / actual.len() as f64,
    ));
    out.push_str("first 10 diffs (x, y, expected_u16, got_u16):\n");
    for (x, y, e, a) in &first {
        out.push_str(&format!("  ({x}, {y})  expected {e:#06x}  got {a:#06x}\n"));
    }
    out
}

#[test]
#[ignore = "needs fixtures/news_after.pixels.bin — see module docs \
            for the capture command"]
fn news_screen_renders_byte_exact_against_exe_framebuffer() {
    let expected = load_expected_after().unwrap_or_else(|| {
        panic!("fixture missing: {} — see module docs for the capture \
                command that produces it", fixture_path().display());
    });

    let mut pool = GuiRecordPool::new();
    build_news_screen(&mut pool, &news_state_matching_capture())
        .expect("build_news_screen must not overflow the pool");

    let mut surface = PackedSurface::rgb555(SCREEN_W, SCREEN_H);
    let font = PixelFont::empty(12);
    render_all_widgets(&mut surface, &mut pool, &font);

    if surface.buf != expected {
        panic!("News pixel-diff mismatch:\n{}",
               report_first_diffs(&surface.buf, &expected));
    }
}

/// Sanity smoke test that runs unconditionally: the pipeline used by
/// the ignored diff test must at least not panic on a fresh pool.
/// Catches signature/regression breakage even without the fixture.
#[test]
fn pixel_diff_pipeline_does_not_panic_on_empty_fixture_path() {
    let mut pool = GuiRecordPool::new();
    build_news_screen(&mut pool, &news_state_matching_capture())
        .expect("build_news_screen");
    let mut surface = PackedSurface::rgb555(SCREEN_W, SCREEN_H);
    let font = PixelFont::empty(12);
    render_all_widgets(&mut surface, &mut pool, &font);
    // Surface should have been touched somewhere — at least one non-zero
    // pixel from a widget paint.
    let any_painted = surface.buf.iter().any(|&p| p != 0);
    assert!(any_painted, "render_all_widgets produced an empty framebuffer");
}
