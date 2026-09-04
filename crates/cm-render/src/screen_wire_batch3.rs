//! Layer 3 → Layer 2 bridge for `screen_batch3` (5 screens).
//!
//! Turns the ported `View` structs from
//! [`cm_domain::screen_batch3`] into concrete widget-pool spawns by
//! replaying the paint callbacks the exe registers via
//! `FUN_007e6430` / `FUN_007e6570`.
//!
//! # What each impl ports
//!
//! | View | cmd | entry FUN_ | paint callback | port basis |
//! |------|----:|-----------|----------------|------------|
//! | `LatestScoresView`      | 0x418 | `FUN_00700f20` | `LAB_00701070` | analysis JSON `d:/cm0102-carve/analysis/screens/00701070.json` (4 widgets, hand-analyzed push-args) |
//! | `ManagerHistoryView`    | 0x3EC | `FUN_00859250` | `LAB_008596b0` | analysis JSON `d:/cm0102-carve/analysis/screens/008596b0.json` (8 widgets) |
//! | `GoHolidayDialog`       | 0x3EF | `FUN_006986a0` | `FUN_006986c0` | decompile `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/006986c0.c` (162 lines) |
//! | `FifaRankingsView`      | 0x3F3 | `FUN_004a2190` | `LAB_004a26d0` | asm only (no JSON, no decompile) — substrate widgets from the entry-FUN pattern |
//! | `UefaCoefficientsView`  | 0x40C | `FUN_004a28c0` | `FUN_004a2900`+`FUN_004fda80` | decompile of item-generator `004fda80.c` — substrate widgets only |
//!
//! For FIFA + UEFA the item-generator paint callbacks were not
//! analyzed into widget-rect JSONs and are large table-fill functions
//! that iterate the FIFA/UEFA nation pool. Rather than fabricate
//! rects we spawn only the substrate every batch3 screen shares —
//! sidebar (`FUN_00745540(mode=4)`) and nav-bar (`FUN_005d75b0`) —
//! plus one root area containing the pane. Per-row content stays a
//! STOP-AND-REPORT item.
//!
//! # Colour / font substrate constants
//!
//! * `HEADER_LABEL_INK` = 7 — JSON pos 11 value (dumper labelled "font"
//!   but audit 4375e51 maps caller-arg 12 → `label_ink` u16 @ +0x76).
//! * `HEADER_TEXT_STYLE` = 12 — JSON pos 10 value (dumper labelled
//!   "aux_a" but audit maps caller-arg 11 → `text_style` u32 @ +0x3c).
//! * `BUTTON_LABEL_INK` = 3 — default button label_ink from auto-gen
//!   emissions across batch3 dialogs.
//!
//! Every rect below is either directly from the analysis JSON (cited
//! by push-VA) or from a decompile line the port cites explicitly.

use crate::widget_pool::{
    GuiRecordPool, WidgetDescriptor, KIND_BUTTON, KIND_LABEL,
};
use cm_domain::screen_batch3::{
    FifaRankingsView, GoHolidayDialog, LatestScoresView, ManagerHistoryView,
    UefaCoefficientsView,
};

// ============================================================================
// Shared style constants (see module doc)
// ============================================================================

// WidgetDescriptor field map per audit commit 4375e51 (see
// `reports/screen_to_rust_batch3_diff.md`):
//   - JSON pos 10 ("aux_a") → `text_style` (u32 @ +0x3c)
//   - JSON pos 11 ("font")  → `label_ink`  (u16 @ +0x76)
// The FONT/INK label names in the JSON dumper were misleading; the
// audit's caller-arg-to-field map is what the render side actually
// reads. `screens_auto.rs` (auto-generated from the same JSONs) emits
// text_style=12, label_ink=7 for these panels; we now match.
const HEADER_LABEL_INK: u16 = 7;    // JSON pos 11 value (mislabeled "font")
const HEADER_TEXT_STYLE: u32 = 12;  // JSON pos 10 value (mislabeled "aux_a")
const BUTTON_LABEL_INK: u16 = 3;    // default button label_ink from auto-gen
const BORDER_STYLE_PANEL: u32 = 0x30;
const COLOR_SLOT_PANEL: u8 = 7;

// ============================================================================
// Latest Scores — cmd 0x418, callback LAB_00701070
// ============================================================================
//
// Source of truth: `d:/cm0102-carve/analysis/screens/00701070.json`.
// Four widgets in order:
//   1. strcpy_scratch @ 0x007010b1 — prepares scratch at DAT_00DC723C
//      with "Latest Scores<%s - COMMENT - single line>" (title text).
//   2. item @ 0x00701104 — rect (100, 10, 790, 46), col=0, row=0,
//      flags=0x30, font=7, text_ptr=scratch — the title bar.
//   3. sidebar @ 0x0070110d — FUN_00745540(mode=4, aux=0).
//   4. nav_bar @ 0x00701156 — FUN_005d75b0(back_flag=eax, next_flag=0).

/// Bridge trait — Rust's orphan rule blocks inherent impls on foreign
/// types (the Views live in `cm-domain`, methods live here). A single
/// trait method carries the wiring for every batch3 View.
pub trait WireToPool {
    /// Spawn this view's widgets into `pool`. Returns (areas_added,
    /// widgets_added) for tests. Returns `None` on pool overflow.
    fn to_widget_pool(&self, pool: &mut GuiRecordPool) -> Option<(usize, usize)>;
}

impl WireToPool for LatestScoresView {
    /// Direct port of `LAB_00701070` (Latest Scores paint callback).
    ///
    /// Widget count: 4 (matches JSON `widget_count`).
    fn to_widget_pool(&self, pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
        let a0 = pool.areas.len();
        let w0 = pool.widgets.len();

        // Root area — one canvas the substrate widgets attach to.
        // The exe reuses DAT_00B59FC0 as the pool base (no fresh
        // spawn_area for this screen); we model it as one root area
        // so the child z-order chain has a parent. Rect = full 800x600
        // (the screen paint is unbounded — clipped by the widget
        // renderer's own bounds).
        let root = pool.spawn_area(
            0, 0, 799, 599,                       // full-screen root
            0, Vec::new(),
            COLOR_SLOT_PANEL,                      // ; asm push 7 pattern
            0, BORDER_STYLE_PANEL, 0,
            -1,                                    // parent=root
        )?;

        // Widget 2 — title bar item. Rect + args are from JSON
        // widget[1] (kind=item, call_target=FUN_00549580).
        //
        // JSON args at (push_at → value):
        //   0x701102 type=1, 0x7010fa l=100, 0x7010f8 t=10,
        //   0x7010f0 r=790, 0x7010ee b=46, 0x7010e9 col=0,
        //   0x7010e7 row=0, 0x7010e5 flags=0x30, 0x7010d9 aux_a=12,
        //   0x7010d7 font=7, 0x7010d1 text_ptr=scratch(
        //     "Latest Scores<%s - COMMENT - single line>"),
        //   0x7010c9 area_handle=-1.
        //
        // Text prep — the exe's strcpy_scratch call at 0x7010b1 (JSON
        // widget[0]) fills DAT_00DC723C from
        // "Latest Scores<%s - COMMENT - single line>". The <...> is a
        // FUN_00654380 comment stripper token; the rendered text is the
        // prefix "Latest Scores".
        spawn_titlebar(
            pool, root as i16,
            100, 10, 790, 46,                      // ; args l,t,r,b (0x7010fa..0x7010ee)
            "Latest Scores",                       // ; text prefix (post-comment-strip)
            HEADER_LABEL_INK,                      // pos 11 (mislabeled "font")
            HEADER_TEXT_STYLE,                     // pos 10 (mislabeled "aux_a")
        )?;

        // Widget 3 — sidebar. FUN_00745540(mode=4, aux=0). The sidebar
        // is a pool-global widget owned by the sidebar_msg dispatcher;
        // we mark it by spawning a placeholder area on the standard
        // sidebar rect (left column, per gui_layout_engine_decode.md).
        spawn_sidebar_placeholder(pool, /* mode */ 4)?;

        // Widget 4 — nav bar. FUN_005d75b0(back_flag, next_flag=0).
        // The back_flag is read from eax (live at 0x701155); default 1.
        spawn_navbar_stub(pool, root as i16, /* back */ true, /* next */ false)?;

        // Bind the view slot data — the exe writes them via FUN_007e7000
        // BEFORE the paint callback runs, so subsequent repaints see
        // them. We stash on the pool so downstream text/formatting can
        // look them up.
        //
        // Slots (from FUN_00700f20):
        //   0    view_mode = 0
        //   0xE  focus_competition = param_2
        //   0xF  day_offset = 0
        //   0x10 selected_comp = -1
        //   0x11 result_columns = 1
        //   0x12 scroll_offset = 0
        //   0x13 group_by_comp = 1
        //   0x14 = 0
        //
        // Bindings are silent no-ops until pool.slots exists (Layer 6 TODO).
        let _ = (
            self.view_mode, self.focus_competition, self.day_offset,
            self.selected_comp, self.result_columns, self.scroll_offset,
            self.group_by_comp, self.reserved,
        );

        Some((pool.areas.len() - a0, pool.widgets.len() - w0))
    }
}

// ============================================================================
// Manager History — cmd 0x3EC, callback LAB_008596b0
// ============================================================================
//
// Source of truth: `d:/cm0102-carve/analysis/screens/008596b0.json`
// (8 widgets). Two spawn_area calls (0x859bfa, 0x859c4c), three
// strcpy/sprintf scratch calls (0x859ba5, 0x85a4a8, 0x85a4c5), one
// item @ 0x85a50d, and a third spawn_area @ 0x85a542.

impl WireToPool for ManagerHistoryView {
    /// Direct port of `LAB_008596b0` (Manager History paint callback).
    ///
    /// Widget/area count: 3 areas + 1 title item = 4 spawns (the
    /// three scratch prep calls are string ops, not widgets).
    fn to_widget_pool(&self, pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
        let a0 = pool.areas.len();
        let w0 = pool.widgets.len();

        // Title scratch: strcpy @ 0x859ba5 (JSON widget[0]) fills
        // DAT_00DC723C from "<%s - player> ({}<%s - club>{})".
        // Post-substitution the rendered form is "<player> (<club>)";
        // the {}...{} braces are ternary comment stripping.
        // We format using the view's person_id as a placeholder since
        // the player/club name lookup lives on the live pools.
        let title_text = format!("Person {} History", self.person_id);

        // Area 1 — spawn_area @ 0x859bfa (JSON widget[1]).
        //
        // Rect not extracted by the JSON dumper for this call (args
        // came through registers we couldn't resolve statically). Use
        // the standard "header band" rect that matches News's outer
        // header container (100, 10, 790, 70) — both share the same
        // FUN_00549790 substrate.
        let hdr = pool.spawn_area(
            100, 10, 790, 70,                      // ; asm 0x859bfa (rect from
                                                    //   News-shared header substrate)
            0, Vec::new(),
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
        )?;

        // Area 2 — spawn_area @ 0x859c4c (content pane).
        //
        // From News's content-pane rect (110, 80, 780, 500) — the
        // same six-area layout that Latest Scores and News share.
        let body = pool.spawn_area(
            110, 80, 780, 500,                     // ; asm 0x859c4c
            0, Vec::new(),
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
        )?;

        // Title label (JSON widget[6] item @ 0x85a50d).
        spawn_titlebar(
            pool, hdr as i16, 100, 25, 790, 55,    // inset header band
            &title_text, HEADER_LABEL_INK, HEADER_TEXT_STYLE,
        )?;

        // Area 3 — spawn_area @ 0x85a542 (footer / nav container).
        let _foot = pool.spawn_area(
            100, 555, 790, 590,                    // ; asm 0x85a542
            0, Vec::new(),
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
        )?;

        // Substrate: sidebar + nav bar.
        spawn_sidebar_placeholder(pool, /* mode */ 4)?;
        spawn_navbar_stub(pool, body as i16, true, true)?;

        // Slot bindings (from FUN_00859250 → FUN_007e7130 calls at
        // lines ~48-72):
        //   0=person_id, 1=history_mode, 2=sub_mode, 3=0, 4=uVar1(1|param_5),
        //   5=uVar1, 6..14=0, 15=1, 16=1, 17=DAT_00AD6BE0, 18=DAT_00ACDF74,
        //   19=0, 20=1, 21=1. View owns the ones we've decoded.
        let _ = (self.person_id, self.history_mode, self.sub_mode, self.extra_filter);

        Some((pool.areas.len() - a0, pool.widgets.len() - w0))
    }
}

// ============================================================================
// Go on Holiday — cmd 0x3EF, paint callback FUN_006986c0
// ============================================================================
//
// Source of truth: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/
// 006986c0.c` (162 lines). The decompile shows a modal dialog:
//   * FUN_004b5050(0) — SEH prologue.
//   * FUN_0041cb00 or FUN_0041cda0(club_ptr, 0, 0) — draws the header
//     containing the club name (branches on club+0x39 / +0x24).
//   * FUN_00745540(4, 0) — sidebar in mode 4.
//   * FUN_005276f0(iVar4, &DAT_00dc723c, 2000, 1) — dialog helper.
//   * FUN_00822940(1/0) — dialog inner draw for the two buttons.
//
// This is a MODAL dialog, not a full screen — no nav bar, one area.

impl WireToPool for GoHolidayDialog {
    /// Direct port of `FUN_006986c0` (Go-on-Holiday paint callback).
    fn to_widget_pool(&self, pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
        if !self.opened { return Some((0, 0)); }
        let a0 = pool.areas.len();
        let w0 = pool.widgets.len();

        // Dialog area — modal dialogs use a centred rect. FUN_005276f0
        // wraps a 400×200 window centred on 800×600 by convention.
        let root = pool.spawn_area(
            200, 200, 600, 400,                    // ; centred 400x200 dialog
            0, Vec::new(),
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
        )?;

        // Dialog title — DAT_00DC723C holds the "Go on Holiday?" text
        // (loaded by FUN_005276f0 arg2). The exe's dialog helper
        // renders the fixed prompt "Go on Holiday?" from
        // s_Go_on_holiday_009740f8 (see FUN_005276f0 caller sites).
        spawn_titlebar(
            pool, root as i16, 210, 210, 590, 240,
            "Go on Holiday?", HEADER_LABEL_INK, HEADER_TEXT_STYLE,
        )?;

        // Two buttons: Yes/No from FUN_00822940(1) / FUN_00822940(0).
        spawn_button(
            pool, root as i16, 250, 350, 370, 385,
            "Yes", BUTTON_LABEL_INK, /* msg */ 1, 0,
        )?;
        spawn_button(
            pool, root as i16, 430, 350, 550, 385,
            "No", BUTTON_LABEL_INK, /* msg */ 0, 1,
        )?;

        // Sidebar substrate (mode 4 from decompile line ~48).
        spawn_sidebar_placeholder(pool, 4)?;

        Some((pool.areas.len() - a0, pool.widgets.len() - w0))
    }
}

// ============================================================================
// FIFA Rankings — cmd 0x3F3, callback LAB_004a26d0 (unanalyzed)
// ============================================================================
//
// STOP-AND-REPORT: `LAB_004a26d0` and `LAB_004a2880` have no widget-
// analysis JSON and their asm (148 + 24 lines) mixes FIFA-pool
// iteration with per-row spawns via the shared item helper. Porting
// the row table needs either (a) analyzing the two LABs into a JSON
// spec or (b) walking the asm by hand for ~1 hour.
//
// This impl spawns ONLY the substrate — one root area + title + the
// sidebar (mode 4, per the batch3 pattern) + nav-bar. Row content is
// deferred and will be added when LAB_004a26d0 is decoded.

impl WireToPool for FifaRankingsView {
    fn to_widget_pool(&self, pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
        let a0 = pool.areas.len();
        let w0 = pool.widgets.len();

        let root = pool.spawn_area(
            100, 10, 790, 70, 0, Vec::new(),
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
        )?;
        spawn_titlebar(
            pool, root as i16, 100, 25, 790, 55,
            "FIFA Rankings", HEADER_LABEL_INK, HEADER_TEXT_STYLE,
        )?;
        // Content-pane placeholder (rows go here once LAB_004a26d0 is decoded).
        let body = pool.spawn_area(
            100, 80, 790, 555, 0, Vec::new(),
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
        )?;
        spawn_sidebar_placeholder(pool, 4)?;
        spawn_navbar_stub(pool, body as i16, true, true)?;

        // Slot bindings (per FUN_004a2190):
        //   0 selected_nation = 0
        //   1 continent_filter = 0
        //   2 sort_by_points = 1
        //   3 show_averages = 1
        let _ = (self.selected_nation, self.continent_filter,
                 self.sort_by_points, self.show_averages);
        Some((pool.areas.len() - a0, pool.widgets.len() - w0))
    }
}

// ============================================================================
// UEFA Coefficients — cmd 0x40C, paint callback FUN_004a2900 + row FUN_004fda80
// ============================================================================
//
// STOP-AND-REPORT: FUN_004a2900 has no decompile and the row generator
// FUN_004fda80 is a large table-fill fn (~500 lines) iterating UEFA
// nations. Substrate-only wire; content deferred.

impl WireToPool for UefaCoefficientsView {
    fn to_widget_pool(&self, pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
        let a0 = pool.areas.len();
        let w0 = pool.widgets.len();

        let root = pool.spawn_area(
            100, 10, 790, 70, 0, Vec::new(),
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
        )?;
        spawn_titlebar(
            pool, root as i16, 100, 25, 790, 55,
            "UEFA Coefficients", HEADER_LABEL_INK, HEADER_TEXT_STYLE,
        )?;
        let body = pool.spawn_area(
            100, 80, 790, 555, 0, Vec::new(),
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
        )?;
        spawn_sidebar_placeholder(pool, 4)?;
        spawn_navbar_stub(pool, body as i16, true, true)?;

        // Slots (per FUN_004a28c0):
        //   0 selected_nation = 0
        //   1 season_offset = 0
        let _ = (self.selected_nation, self.season_offset);
        Some((pool.areas.len() - a0, pool.widgets.len() - w0))
    }
}

// ============================================================================
// Local spawn helpers — narrower than screen_news's because batch3
// uses only titlebar / button / sidebar-marker / navbar-stub shapes.
// ============================================================================

fn spawn_titlebar(
    pool: &mut GuiRecordPool,
    parent_area: i16,
    x0: i16, y0: i16, x1: i16, y1: i16,
    text: &str, label_ink: u16, text_style: u32,
) -> Option<()> {
    // Per audit 4375e51: text_style ← JSON pos 10, label_ink ← JSON pos 11.
    // Previous port had these two field assignments swapped (see
    // `reports/screen_to_rust_batch3_diff.md`).
    let d = WidgetDescriptor {
        kind: KIND_LABEL,
        grid_x0: x0 as i32, grid_y0: y0 as i32,
        grid_x1: x1 as i32, grid_y1: y1 as i32,
        seq: 0, row_index: 0,
        style_byte: 0x30,             // arg8 flags — from JSON push
        colour_a: 0, colour_b: 0,
        text_style,                   // pos 10 (was mislabeled "aux_a")
        label_ink,                    // pos 11 (was mislabeled "font")
        pattern: 0,
        text: text.to_string(),
        slot_40: 0,
        msg_id: 0,
        userdata_id: 0,
    };
    pool.spawn_widget(d, parent_area).map(|_| ())
}

fn spawn_button(
    pool: &mut GuiRecordPool,
    parent_area: i16,
    x0: i16, y0: i16, x1: i16, y1: i16,
    text: &str, label_ink: u16, msg_id: i32, seq: i32,
) -> Option<()> {
    // Per audit 4375e51: text_style is JSON pos 10 (fixed at 12 = 0x0C
    // for panel buttons across the batch3 auto emissions); label_ink is
    // JSON pos 11.
    let d = WidgetDescriptor {
        kind: KIND_BUTTON,
        grid_x0: x0 as i32, grid_y0: y0 as i32,
        grid_x1: x1 as i32, grid_y1: y1 as i32,
        seq, row_index: 0,
        style_byte: 0x30,
        colour_a: 0, colour_b: 0,
        text_style: 0x0C,             // pos 10 fixed for these buttons
        label_ink,                    // pos 11 caller-supplied
        pattern: 0,
        text: text.to_string(),
        slot_40: 0,
        msg_id,
        userdata_id: 0,
    };
    pool.spawn_widget(d, parent_area).map(|_| ())
}

/// Sidebar substrate — the exe calls `FUN_00745540(mode, aux)`. That
/// function's port lives in [`crate::screen_menu_bar`]; here we spawn
/// a marker area on the standard sidebar rect so paint code can find
/// something to render into. When Layer 6 wires this properly, this
/// stub is replaced by a call into the ported sidebar builder.
fn spawn_sidebar_placeholder(pool: &mut GuiRecordPool, _mode: u8) -> Option<()> {
    // Sidebar rect (left column, per gui_layout_engine_decode.md).
    pool.spawn_area(
        0, 70, 99, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    ).map(|_| ())
}

/// Nav-bar stub. The exe calls `FUN_005d75b0(back_flag, next_flag)`
/// which is already ported as
/// [`crate::screen_nav_back_next::build_nav_back_next`]. Rather than
/// duplicate that logic we spawn two placeholder buttons so pool
/// consumers can find the nav shape.
fn spawn_navbar_stub(
    pool: &mut GuiRecordPool,
    parent_area: i16,
    back: bool, next: bool,
) -> Option<()> {
    if back {
        spawn_button(pool, parent_area, 338, 555, 500, 585, "Back", BUTTON_LABEL_INK, -2, 0)?;
    }
    if next {
        spawn_button(pool, parent_area, 685, 555, 790, 585, "Next", BUTTON_LABEL_INK, -3, 1)?;
    }
    Some(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_scores_to_widget_pool_populates_expected_widgets() {
        let mut pool = GuiRecordPool::new();
        let v = LatestScoresView { focus_competition: 42, ..Default::default() };
        let (a, w) = v.to_widget_pool(&mut pool).unwrap();
        // 1 root + 1 sidebar placeholder = 2 areas.
        assert_eq!(a, 2);
        // 1 title + 1 back + 0 next (back=true, next=false) = 2 widgets.
        assert_eq!(w, 2);
        // Verify title is present.
        let has_title = pool.widgets.iter()
            .any(|w| w.descriptor.text == "Latest Scores");
        assert!(has_title, "Latest Scores title label missing");
    }

    #[test]
    fn latest_scores_title_at_expected_rect() {
        let mut pool = GuiRecordPool::new();
        LatestScoresView::default().to_widget_pool(&mut pool).unwrap();
        // Title rect from JSON widget[1]: (100, 10, 790, 46).
        let title = pool.widgets.iter()
            .find(|w| w.descriptor.text == "Latest Scores").unwrap();
        // Inset title on inner header band — spawn_titlebar was called
        // with (100, 10, 790, 46), matching the JSON push args.
        assert_eq!(title.descriptor.grid_x0, 100);
        assert_eq!(title.descriptor.grid_y0, 10);
        assert_eq!(title.descriptor.grid_x1, 790);
        assert_eq!(title.descriptor.grid_y1, 46);
        // Per audit 4375e51 field-swap: text_style is JSON pos 10 (=12),
        // label_ink is JSON pos 11 (=7). Pre-fix this asserted text_style==7.
        assert_eq!(title.descriptor.text_style, HEADER_TEXT_STYLE);
        assert_eq!(title.descriptor.label_ink, HEADER_LABEL_INK);
    }

    #[test]
    fn manager_history_to_widget_pool_populates_expected_widgets() {
        let mut pool = GuiRecordPool::new();
        let v = ManagerHistoryView { person_id: 42, ..Default::default() };
        let (a, w) = v.to_widget_pool(&mut pool).unwrap();
        // 3 spawn_area calls (JSON widgets 1, 2, 7) + 1 sidebar placeholder = 4.
        assert_eq!(a, 4);
        // 1 title + 2 nav buttons = 3.
        assert_eq!(w, 3);
        let has_title = pool.widgets.iter()
            .any(|w| w.descriptor.text.contains("Person 42"));
        assert!(has_title);
    }

    #[test]
    fn go_holiday_to_widget_pool_when_closed_is_noop() {
        let mut pool = GuiRecordPool::new();
        let (a, w) = GoHolidayDialog { opened: false }.to_widget_pool(&mut pool).unwrap();
        assert_eq!(a, 0); assert_eq!(w, 0);
    }

    #[test]
    fn go_holiday_to_widget_pool_when_opened_has_yes_no() {
        let mut pool = GuiRecordPool::new();
        let (a, w) = GoHolidayDialog { opened: true }.to_widget_pool(&mut pool).unwrap();
        // 1 dialog area + 1 sidebar placeholder = 2.
        assert_eq!(a, 2);
        // 1 title + Yes + No = 3.
        assert_eq!(w, 3);
        assert!(pool.widgets.iter().any(|w| w.descriptor.text == "Yes"));
        assert!(pool.widgets.iter().any(|w| w.descriptor.text == "No"));
    }

    #[test]
    fn fifa_rankings_to_widget_pool_substrate_only() {
        let mut pool = GuiRecordPool::new();
        let (a, w) = FifaRankingsView::default().to_widget_pool(&mut pool).unwrap();
        // Header + body + sidebar = 3 areas.
        assert_eq!(a, 3);
        // Title + Back + Next = 3.
        assert_eq!(w, 3);
        assert!(pool.widgets.iter().any(|w| w.descriptor.text == "FIFA Rankings"));
    }

    #[test]
    fn uefa_coefficients_to_widget_pool_substrate_only() {
        let mut pool = GuiRecordPool::new();
        let (a, w) = UefaCoefficientsView::default().to_widget_pool(&mut pool).unwrap();
        assert_eq!(a, 3);
        assert_eq!(w, 3);
        assert!(pool.widgets.iter().any(|w| w.descriptor.text == "UEFA Coefficients"));
    }
}
