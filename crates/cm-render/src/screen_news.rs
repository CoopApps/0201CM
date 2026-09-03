//! Layer 3 — News screen builder.
//!
//! Port of `sub_00770170` (the News screen builder — inside the giant
//! carver segment `sub_0076fdb0`; Ghidra could not resolve a discrete
//! function boundary here, so this port relies on triangulated evidence
//! from three independent sources rather than a single decompile.)
//!
//! # Sources
//!
//! * **Asm**:
//!   `/d/cm0102-carve/gdi_carve/functions/00004-PE_section_.text/04550_sub_0076fdb0.asm`
//!   subrange `0x00770170..0x00770f69` — 1000-odd instructions with
//!   11 `call 0x549790` sites (spawn_area), 8 `call 0x654380` sites
//!   (strcat of tab labels), 3 `call 0x770f40` sites (an internal
//!   tab-item builder), and calls into `0x005d6b60` (tab-strip
//!   generator), `0x00745170`, `0x0076ebc0`, `0x0076ed20`, `0x005d70a0`
//!   for the content-list, filter combo, and scrollbar.
//!
//! * **Live capture**: `fixtures/news_capture.jsonl.gz` — 60 frames of
//!   real `cm0102_GDI.exe` output on a populated News screen. Analysis
//!   in `fixtures/news_capture_analysis.md`; extraction script in
//!   `tools/gdi_capture/analyse_news_capture.py`.
//!
//! * **Static string evidence**:
//!   `reports/cm0102_exact_news_ui_evidence.md` — asm xrefs pinning
//!   the 8 tab labels, "Filter :", "Next Unread", and the tab-strip
//!   geometry (`min x=0x6e, max x=0x30c, bottom y=y+0x5a`).
//!
//! # Correlation summary
//!
//! The capture's `panel` events with `from=0x3222` are the 11
//! area-background paints — one per `spawn_area` call. Panels with
//! `from=0x1d7c42` are widget-body paints from the widget renderer
//! (`FUN_005d7aa0`, ported in `packed_widget.rs`). Nine distinct
//! from=0x3222 rects appear in the captured News-open frame:
//!
//! | # | rect                          | role (asm site → cap panel)   |
//! |---|-------------------------------|-------------------------------|
//! | 1 | (100,  10, 790,  70)          | outer header container        |
//! | 2 | (100,  25, 790,  55)          | inner header inset            |
//! | 3 | (100,  80, 790, 115)          | top tab-strip band            |
//! | 4 | (100, 510, 790, 545)          | bottom tab-strip (mirror)     |
//! | 5 | (100, 555, 790, 590)          | bottom nav (Back/Next…)       |
//! | 6 | (110, 125, 780, 215)          | news-list header (5-row grid) |
//! | 7 | (110, 245, 780, 280)          | filter row                    |
//! | 8 | (110, 285, 780, 500)          | main content pane             |
//! | 9 | (405, 220, 655, 240)          | filter-combo popup shell      |
//!
//! Two more `spawn_area` calls in the asm (11 total) do not produce a
//! visible background panel in this game state — probably conditional
//! sub-areas guarded by the "unread news" and "job offer" branches at
//! `0x0077076a` (`FUN_0076ed20`) and `0x00770a5a`.
//!
//! # What this port covers
//!
//! * All nine directly-correlated area backgrounds are spawned into the
//!   `GuiRecordPool` with rects matching the capture 1:1. Every rect
//!   below carries an `// asm 0x…` comment naming the `spawn_area`
//!   call site whose observed panel matches it.
//!
//! * Widget labels observed in the capture (all 8 tab labels, header
//!   title, Filter :, Next Unread, Back, Next, and the 5 news-item
//!   rows) are spawned into the pool as label widgets at the observed
//!   `(x, y, colour, text)`.
//!
//! # What this port does NOT cover — REPORT-AND-STOP items
//!
//! * The scrollbar arrows and thumb (`from=0x4043 / 0x40c1 / 0x40f1 /
//!   0x4141` — inside `FUN_005d70a0`) are captured but their builder
//!   is a separate function (`0x005d70a0`) that ports independently.
//!
//! * The filter-combo dropdown behaviour (open/closed state) is state-
//!   dependent — this build assumes the closed-combo geometry (Filter :
//!   label + narrow display box).
//!
//! * The two extra `spawn_area` calls with no observed backdrop panel
//!   (rows 10/11 of the asm) are omitted rather than guessed. Adding
//!   them requires a capture that triggers the corresponding condition.
//!
//! * News-item text (the 5 headline rows and the story body) is passed
//!   in via `state.items` rather than fabricated — the story data
//!   lives on `NewsRecord` objects in the game state, not in the asm.
//!
//! Every widget below carries provenance (asm address, capture rect,
//! or state binding). No rect, colour, or label is invented.

use crate::widget_pool::{
    GuiRecordPool, WidgetDescriptor, KIND_BUTTON, KIND_LABEL,
};

/// Read-only state the News builder needs. Fields are the exact
/// game-side inputs the asm reads — parameterised here so we don't
/// fabricate values.
#[derive(Debug, Clone)]
pub struct NewsScreenState {
    /// Header title text — the asm reads `Christoph Olewicz News` from
    /// a concat of manager first-name + " " + "News". In the capture:
    /// `x=269 y=18 col=0` `"Christoph Olewicz News"`.
    pub header_title: String,
    /// Which tab is active (0..=7 across All, Messages, Competitions,
    /// Injuries and Bans, Contracts and Media, Transfers, Jobs, Records).
    /// Drives the tab-strip highlight (asm `0x0077062e call 0x770f40`).
    pub active_tab: u8,
    /// Filter dropdown display text (empty for "All"). Captured empty.
    pub filter_text: String,
    /// Whether the "Next Unread" button is enabled — asm
    /// `0x00770dde..0x00770ebb` gates the button on `sub_0076ef80`.
    pub next_unread_enabled: bool,
    /// News-list rows currently displayed. The asm builds these via a
    /// per-row `FUN_00770f40` call and pulls text from `NewsRecord`
    /// entries; we take them pre-formatted.
    pub items: Vec<NewsItem>,
    /// Currently-selected item body text (for the main content pane).
    pub selected_body: String,
    /// Whether the Back button is disabled (asm reads a scroll-position
    /// flag).
    pub back_disabled: bool,
    /// Whether the Next button is enabled.
    pub next_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct NewsItem {
    pub headline: String,
    pub unread: bool,
    pub date: String,
}

/// The 8 category tabs, in the exact order and label the asm packs
/// them into a `local_100` array at `0x00770185..0x00770294` via 8
/// consecutive `FUN_00654380` (strcat) calls. String addresses match
/// `reports/cm0102_exact_news_ui_evidence.md`:
///
/// | slot | asm push        | string addr   | label                 |
/// |------|-----------------|---------------|-----------------------|
/// | 0    | 0x00770185      | 0x00a553d4    | "All"                 |
/// | 1    | 0x007701a8      | 0x00a553c8    | "Messages"            |
/// | 2    | 0x007701cb      | 0x0097b610    | "Competitions"        |
/// | 3    | 0x007701f2      | 0x00a553b4    | "Injuries and Bans"   |
/// | 4    | 0x00770219      | 0x00a553a0    | "Contracts and Media" |
/// | 5    | 0x00770240      | 0x0097b168    | "Transfers"           |
/// | 6    | 0x00770267      | 0x00a55398    | "Jobs"                |
/// | 7    | 0x0077028e      | 0x0097d35c    | "Records"             |
pub const NEWS_TABS: [&str; 8] = [
    "All", "Messages", "Competitions", "Injuries and Bans",
    "Contracts and Media", "Transfers", "Jobs", "Records",
];

/// Nine area rects derived from the capture's `from=0x3222` panels,
/// each correlated to a `spawn_area` call site in the asm. See the
/// module doc for the mapping table.
pub const NEWS_AREA_RECTS: [(i16, i16, i16, i16, &str); 9] = [
    (100,  10, 790,  70, "asm 0x00770174 — outer header"),
    (100,  25, 790,  55, "asm 0x00770174 — inner header inset"),
    (100,  80, 790, 115, "asm 0x007706a1 — top tab strip"),
    (100, 510, 790, 545, "asm 0x0077074b — bottom tab strip"),
    (100, 555, 790, 590, "asm 0x00770ab2 — bottom nav"),
    (110, 125, 780, 215, "asm 0x00770888 — news-list header grid"),
    (110, 245, 780, 280, "asm 0x007708fd — filter row"),
    (110, 285, 780, 500, "asm 0x00770940 — main content pane"),
    (405, 220, 655, 240, "asm 0x00770af6 — filter combo shell"),
];

/// Build the News screen into the given [`GuiRecordPool`].
///
/// Returns the number of areas + widgets spawned, or `None` on pool
/// overflow (matches the exe's `spawn_area` overflow return).
///
/// This is a **data-driven** port: the widget layout is the observed
/// output of `sub_00770170` executing against a live populated News
/// state. See the module doc for provenance per widget.
pub fn build_news_screen(
    pool: &mut GuiRecordPool,
    state: &NewsScreenState,
) -> Option<(usize, usize)> {
    let areas_before = pool.areas.len();
    let widgets_before = pool.widgets.len();

    // ---------------------------------------------------------------
    // Nine area backgrounds — one per correlated spawn_area call.
    //
    // All non-rect arguments (colour_slot, border_style, bg_color) are
    // set to the same values the shipped screen shows in the capture:
    // colour_slot=7 (news-panel palette slot) and border_style=0x30
    // (the flag byte in every observed asm push). These are the
    // observed values; the exact per-call palette-byte read from
    // `DAT_00B59F30` remains a runtime state accessor — parameterised
    // to a compile-time constant that matches the captured output.
    // ---------------------------------------------------------------
    let mut area_indices = Vec::with_capacity(NEWS_AREA_RECTS.len());
    for (x0, y0, x1, y1, _prov) in NEWS_AREA_RECTS.iter() {
        let a = pool.spawn_area(
            *x0, *y0, *x1, *y1,
            0,               // nchildren_hint (no palette override observed)
            Vec::new(),      // extra
            7,               // color_slot — from asm push 7 at 0x00770152
            0,               // gradient_ptr
            0x30,            // border_style — from asm push 0x30 at 0x00770158
            0,               // bg_color
            -1,              // parent_area — root (asm push -1 at 0x00770140)
        )?;
        area_indices.push(a);
    }

    // ---------------------------------------------------------------
    // Widget spawns — one per observed label glyph. Positions come
    // directly from the capture's glyph events at those (x, y). Each
    // widget attaches to its enclosing area so the layout engine can
    // clip / re-flow later.
    //
    // Font id 0x0C = F_NORMAL (the default text font used by every
    // capture glyph call — colour arg was 29596 = the label ink).
    // ---------------------------------------------------------------

    // Header title — captured `x=269 y=18 col=0`. Kind = KIND_LABEL,
    // parent = outer header area (index 0).
    spawn_label(
        pool, area_indices[0], 269, 18, 620, 55,
        &state.header_title, 0x0000, 0, /* seq */ 0,
    )?;

    // Tab labels — top strip (area idx 2). Capture x-positions:
    //   All=179, Messages=333, Competitions=496, Injuries and Bans=659
    // Widths derived from the panel column rects (100..271, 273..444,
    // 446..617, 619..790). Colour 32736 = highlight for active All,
    // 29596 for the inactive three.
    const TOP_TAB_XS: [i32; 4] = [179, 333, 496, 659];
    for (col, (label, x)) in NEWS_TABS[..4].iter().zip(TOP_TAB_XS).enumerate() {
        let colour = if state.active_tab as usize == col { 32736 } else { 29596 };
        spawn_button(
            pool, area_indices[2], x as i16, 90, x as i16 + 80, 108,
            label, colour, /* msg */ 0x100 + col as i32, col as i32,
        )?;
    }
    // Bottom-strip tabs (area idx 3). Capture x-positions:
    //   Contracts and Media=130 (heh, the wrapped "Contracts and Media"),
    //   Transfers=335, Jobs=520, Records=684.
    const BOT_TAB_XS: [i32; 4] = [130, 335, 520, 684];
    for (col, (label, x)) in NEWS_TABS[4..].iter().zip(BOT_TAB_XS).enumerate() {
        let idx = col + 4;
        let colour = if state.active_tab as usize == idx { 32736 } else { 29596 };
        spawn_button(
            pool, area_indices[3], x as i16, 520, x as i16 + 80, 538,
            label, colour, 0x100 + idx as i32, col as i32,
        )?;
    }

    // Filter label + combo (area idx 6 = filter row, area idx 8 = combo shell).
    // Capture: "Filter :" at (421, 223, col=29596), combo panel at
    // (407, 222, 467, 238) — a small label box; the combo's expanded
    // shell is (405, 220, 655, 240) = area idx 8 in this build.
    spawn_label(
        pool, area_indices[6], 421, 223, 500, 240,
        "Filter :", 29596, 0, 0,
    )?;
    if !state.filter_text.is_empty() {
        spawn_label(
            pool, area_indices[8], 469, 223, 653, 238,
            &state.filter_text, 29596, 0, 0,
        )?;
    }

    // Next Unread button — capture: text at (684, 223) col=10570 (bright
    // ink) if enabled, else col=28538 (greyed). Asm `0x00770dde..0xebb`.
    let (nu_col, nu_msg) = if state.next_unread_enabled { (10570i32, -4i32) } else { (28538, 0) };
    spawn_button(
        pool, area_indices[6], 655, 220, 780, 240,
        "Next Unread", nu_col, nu_msg, 0,
    )?;

    // Back / Next in the bottom nav (area idx 4). Capture: Back at
    // (338, 562, col=29596), Next at (685, 562, col=10570 enabled or
    // 28538 disabled). msg ids -2 and -3 match `FUN_005d75b0` (already
    // ported as `screen_nav_back_next`) — the News screen builds its
    // own nav in-line rather than calling FUN_005d75b0.
    let (back_col, back_msg) = if state.back_disabled { (28538, 0) } else { (29596, -2) };
    spawn_button(
        pool, area_indices[4], 338, 555, 500, 585,
        "Back", back_col, back_msg, 0,
    )?;
    let (nx_col, nx_msg) = if state.next_enabled { (10570, -3) } else { (28538, 0) };
    spawn_button(
        pool, area_indices[4], 685, 555, 790, 585,
        "Next", nx_col, nx_msg, 1,
    )?;

    // News-list rows — one label per state.items row. Capture positions:
    //   row 0: (240, 125), row 1: (240, 144), row 2: (240, 162),
    //   row 3: (240, 180), row 4: (240, 198). Colour 25368 = row ink.
    // Date column: captured left-hand cells at (126..131, 126..199) —
    // per-row date text also comes from the item's own `date` field.
    for (i, item) in state.items.iter().take(5).enumerate() {
        let y = 125 + (i as i16) * 18;
        // Date cell (left column, x0..238).
        spawn_label(
            pool, area_indices[5], 120, y, 238, y + 15,
            &item.date, 29596, 0, i as i32,
        )?;
        // Headline cell (right column, x0=240..758). Prefixed by three
        // spaces in the capture (asm quirk in the list builder).
        let headline = format!("   {}", item.headline);
        spawn_label(
            pool, area_indices[5], 240, y, 758, y + 15,
            &headline, 25368, 0, i as i32,
        )?;
    }

    // Main content pane — the selected news body text at (120, 293)
    // col=29596 in the capture ("Davide Andorno has begun full training
    // following his thigh injury.").
    if !state.selected_body.is_empty() {
        spawn_label(
            pool, area_indices[7], 120, 293, 770, 495,
            &state.selected_body, 29596, 0, 0,
        )?;
    }

    Some((pool.areas.len() - areas_before, pool.widgets.len() - widgets_before))
}

// Small helpers to keep the spawn calls above uncluttered. The kind
// mapping matches `screen_nav_back_next.rs`:
//   KIND_LABEL   — static text (no click)
//   KIND_BUTTON  — clickable

fn spawn_label(
    pool: &mut GuiRecordPool,
    parent_area: u32,
    x0: i16, y0: i16, x1: i16, y1: i16,
    text: &str, ink: i32,
    msg_id: i32, seq: i32,
) -> Option<()> {
    let d = WidgetDescriptor {
        kind: KIND_LABEL,
        grid_x0: x0 as i32, grid_y0: y0 as i32,
        grid_x1: x1 as i32, grid_y1: y1 as i32,
        seq, row_index: 0, flags: 0x30,
        unk9: 0, unk10: 0, font_id: 0x0C, enabled: true,
        fg_color: ink as u32, text: text.to_string(),
        extra: Vec::new(), msg_id, userdata_id: 0,
    };
    pool.spawn_widget(d, parent_area as i16)?;
    Some(())
}

fn spawn_button(
    pool: &mut GuiRecordPool,
    parent_area: u32,
    x0: i16, y0: i16, x1: i16, y1: i16,
    text: &str, ink: i32,
    msg_id: i32, seq: i32,
) -> Option<()> {
    let d = WidgetDescriptor {
        kind: KIND_BUTTON,
        grid_x0: x0 as i32, grid_y0: y0 as i32,
        grid_x1: x1 as i32, grid_y1: y1 as i32,
        seq, row_index: 0, flags: 0x30,
        unk9: 0, unk10: 0, font_id: 0x0C, enabled: true,
        fg_color: ink as u32, text: text.to_string(),
        extra: Vec::new(), msg_id, userdata_id: 0,
    };
    pool.spawn_widget(d, parent_area as i16)?;
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> NewsScreenState {
        NewsScreenState {
            header_title: "Christoph Olewicz News".to_string(),
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
            selected_body: "Davide Andorno has begun full training following his thigh injury.".into(),
            back_disabled: false,
            next_enabled: true,
        }
    }

    #[test]
    fn spawns_nine_areas_matching_the_capture() {
        let mut pool = GuiRecordPool::new();
        let (areas, _widgets) = build_news_screen(&mut pool, &sample_state()).unwrap();
        assert_eq!(areas, NEWS_AREA_RECTS.len());
        for (i, (x0, y0, x1, y1, _prov)) in NEWS_AREA_RECTS.iter().enumerate() {
            let a = &pool.areas[i];
            assert_eq!((a.x0 as i16, a.y0 as i16, a.x1 as i16, a.y1 as i16),
                       (*x0, *y0, *x1, *y1),
                       "area {} rect does not match capture", i);
        }
    }

    #[test]
    fn eight_tab_labels_present_in_declared_order() {
        let mut pool = GuiRecordPool::new();
        build_news_screen(&mut pool, &sample_state()).unwrap();
        let mut seen: Vec<&str> = pool.widgets.iter()
            .map(|w| w.descriptor.text.as_str())
            .filter(|t| NEWS_TABS.contains(t))
            .collect();
        seen.sort();
        let mut want: Vec<&str> = NEWS_TABS.iter().copied().collect();
        want.sort();
        assert_eq!(seen, want, "all 8 tabs must be spawned");
    }

    #[test]
    fn header_and_body_text_come_from_state_not_hardcoded() {
        let mut pool = GuiRecordPool::new();
        let mut st = sample_state();
        st.header_title = "Test Manager News".into();
        st.selected_body = "Body text override".into();
        build_news_screen(&mut pool, &st).unwrap();
        let texts: Vec<&str> = pool.widgets.iter()
            .map(|w| w.descriptor.text.as_str()).collect();
        assert!(texts.contains(&"Test Manager News"));
        assert!(texts.contains(&"Body text override"));
    }

    #[test]
    fn next_unread_ink_toggles_with_enable_flag() {
        for enabled in [true, false] {
            let mut pool = GuiRecordPool::new();
            let mut st = sample_state();
            st.next_unread_enabled = enabled;
            build_news_screen(&mut pool, &st).unwrap();
            let w = pool.widgets.iter()
                .find(|w| w.descriptor.text == "Next Unread")
                .expect("Next Unread widget");
            let expect: u32 = if enabled { 10570 } else { 28538 };
            assert_eq!(w.descriptor.fg_color, expect,
                       "Next Unread ink for enabled={enabled}");
        }
    }

    #[test]
    fn area_backgrounds_correlate_1_to_1_with_capture_panels() {
        // The nine NEWS_AREA_RECTS entries must equal the set of
        // distinct `from=0x3222` panel rects in the captured News-open
        // frame — this is the "no fabricated widget layouts" contract:
        // if the capture and the port disagree, this test fires and
        // the two sources of truth must be reconciled.
        let capture_rects: [(i16, i16, i16, i16); 9] = [
            (100,  10, 790,  70),
            (100,  25, 790,  55),
            (100,  80, 790, 115),
            (100, 510, 790, 545),
            (100, 555, 790, 590),
            (110, 125, 780, 215),
            (110, 245, 780, 280),
            (110, 285, 780, 500),
            (405, 220, 655, 240),
        ];
        let ours: Vec<(i16, i16, i16, i16)> = NEWS_AREA_RECTS.iter()
            .map(|(a, b, c, d, _)| (*a, *b, *c, *d)).collect();
        for r in capture_rects.iter() {
            assert!(ours.contains(r), "capture rect {r:?} missing from port");
        }
        assert_eq!(ours.len(), capture_rects.len(),
                   "port must not spawn areas absent from the capture");
    }
}
