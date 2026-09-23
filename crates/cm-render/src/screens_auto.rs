//! Auto-generated screen builders from `tools/screen_to_rust.py`.
//! DO NOT EDIT BY HAND — see the tool for source of truth.
//!
//! Each `build_screen_<addr>` fn is a direct transliteration of the
//! widget-analysis JSON for that address; args map per the
//! WidgetDescriptor field-map (see tool docstring).

use crate::widget_pool::{GuiRecordPool, WidgetDescriptor};

const COLOR_SLOT_PANEL: u8 = 7;
const BORDER_STYLE_PANEL: u32 = 0x30;
const LABEL_INK_STUB: u16 = 29596;

fn spawn_sidebar_placeholder(pool: &mut GuiRecordPool, _mode: i32) -> Option<()> {
    pool.spawn_area(0, 70, 99, 599, 0, Vec::new(),
                    COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1).map(|_| ())
}

fn spawn_navbar_stub(pool: &mut GuiRecordPool, parent: i16, back: bool, next: bool) -> Option<()> {
    let mk = |pool: &mut GuiRecordPool, x0, x1, text: &str, seq, msg| -> Option<()> {
        pool.spawn_widget(WidgetDescriptor {
            kind: 2, grid_x0: x0, grid_y0: 555, grid_x1: x1, grid_y1: 585,
            seq, row_index: 0, style_byte: 0x30, colour_a: 0, colour_b: 0,
            text_style: 0x0C, label_ink: LABEL_INK_STUB, pattern: 0,
            text: text.into(), slot_40: 0, msg_id: msg, userdata_id: 0,
        }, parent).map(|_| ())
    };
    if back { mk(pool, 338, 500, "Back", 0, -2)?; }
    if next { mk(pool, 685, 790, "Next", 1, -3)?; }
    Some(())
}

/// Auto-generated from `00701070.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_701070 / callback_va 0x701070 — 4 widgets.
// GDI-REG: 00701070 PORTED_BEHAVIOURAL
pub fn build_screen_701070(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // strcpy_scratch @ 0x7010b1: scratch := "Latest Scores<%s - COMMENT - single line>"
        // item @ 0x701104 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: "Latest Scores<%s - COMMENT - single line>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x70110d — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // nav_bar @ 0x701156 — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x701162
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `008596b0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_8596b0 / callback_va 0x8596b0 — 8 widgets.
// GDI-REG: 008596b0 PORTED_BEHAVIOURAL
pub fn build_screen_8596b0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // strcpy_scratch @ 0x859ba5: scratch := "<%s - player> ({}<%s - club>{})"
        // area @ 0x859bfa — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 10 as i16, 790 as i16, 70 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x859c4c — FUN_00549790
        let _area_2 = pool.spawn_area(
            100 as i16, 25 as i16, 790 as i16, 55 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // sprintf_scratch @ 0x85a00f: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // strcpy_scratch @ 0x85a4a8: scratch := "Add To Shortlist"
        // strcpy_scratch @ 0x85a4c5: scratch := "Action"
        // item @ 0x85a50d — FUN_00549580
        // TODO(unresolved-text pos=13): stack frame_offset=-80
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 660, grid_y0: 4,
            grid_x1: 785, grid_y1: 24,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (18i32) as u16,
            colour_b: (18i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (100i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x85a542 — FUN_00549790
        let _area_3 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00698160.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_698160 / callback_va 0x698160 — 10 widgets.
// GDI-REG: 00698160 PORTED_BEHAVIOURAL
pub fn build_screen_698160(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // sidebar @ 0x6981b6 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // item @ 0x698254 — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x698263: scratch := "Please Confirm"
        // item @ 0x6982ad — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Please Confirm".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x698339: scratch := "Retire from football ?"
        // area @ 0x6983a6 — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x6983b7: scratch := "No"
        // item @ 0x69840b — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "No".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x69841a: scratch := "Yes"
        // item @ 0x69846e — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Yes".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x698477
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `007719b0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_7719b0 / callback_va 0x7719b0 — 10 widgets.
// GDI-REG: 007719b0 PORTED_BEHAVIOURAL
pub fn build_screen_7719b0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x771a3b — FUN_00549580
        // TODO(unresolved-text pos=13): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x771a44 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x771a88: scratch := "Send Message To All"
        // item @ 0x771ad2 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Send Message To All".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x771b27 — FUN_00549580
        // TODO(unresolved-text pos=13): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 65560 as u32,
            grid_x0: 150, grid_y0: 310,
            grid_x1: 740, grid_y1: 370,
            seq: 0, row_index: 0,
            style_byte: 34 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (4i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 100,
            msg_id: 6,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x771b61 — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x771b72: scratch := "Cancel"
        // item @ 0x771bc7 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Cancel".into(),
            slot_40: 0,
            msg_id: 7,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x771bd6: scratch := "Send"
        // item @ 0x771c69 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 44 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Send".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x771c78
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `004150e0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_4150e0 / callback_va 0x4150e0 — 10 widgets.
// GDI-REG: 004150e0 PORTED_BEHAVIOURAL
pub fn build_screen_4150e0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // strcpy_scratch @ 0x4151e4: scratch := "International Awards"
        // item @ 0x415257 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: "International Awards".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x415407: scratch := "Overall"
        // sidebar @ 0x415446 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // item @ 0x415493 — FUN_00549580
        // TODO(unresolved-text pos=13): reg dword ptr [esi + 5] unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4154f2: scratch := "View"
        // item @ 0x415558 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 110, grid_y0: 145,
            grid_x1: 235, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "View".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x41558e — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // sprintf_scratch @ 0x4158ca: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // nav_bar @ 0x415a68 — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x415a7a
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00494640.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_494640 / callback_va 0x494640 — 25 widgets.
// GDI-REG: 00494640 PORTED_BEHAVIOURAL
pub fn build_screen_494640(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x4947c0 — FUN_00549580
        // TODO(unresolved-text pos=13): reg dword ptr [ebx + 4] unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x4947c9 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x494861: scratch := "Rounds"
        // strcpy_scratch @ 0x4948b6: scratch := "Results"
        // strcpy_scratch @ 0x4948d9: scratch := "Fixtures"
        // strcpy_scratch @ 0x494905: scratch := "Schedule<%s - COMMENT - competition schedule>"
        // strcpy_scratch @ 0x494bd3: scratch := "Print"
        // item @ 0x494c38 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 710, grid_y0: 15,
            grid_x1: 785, grid_y1: 35,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Print".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x494c6e — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x494c8f: scratch := "Tables"
        // item @ 0x494d30 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 33 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Tables".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x494d3f: scratch := "Goals"
        // item @ 0x494de3 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 33 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Goals".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x494df2: scratch := "Assists"
        // item @ 0x494e96 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 33 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Assists".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x494ea5: scratch := "Average Ratings"
        // item @ 0x494f49 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 33 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Average Ratings".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x494f58: scratch := "Man of Match"
        // item @ 0x494ffc — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 33 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Man of Match".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x495010: scratch := "All<%s - COMMENT - print out all>"
        // item @ 0x4950b4 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 33 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "All<%s - COMMENT - print out all>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4955be: scratch := "Date"
        // sprintf_scratch @ 0x4955d8: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // strcpy_scratch @ 0x49564e: scratch := "Date"
        // sprintf_scratch @ 0x495668: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `0046bdf0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_46bdf0 / callback_va 0x46bdf0 — 13 widgets.
// GDI-REG: 0046bdf0 PORTED_BEHAVIOURAL
pub fn build_screen_46bdf0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // strcpy_scratch @ 0x46bf25: scratch := "{}<%s - club name>{} History"
        // item @ 0x46bf6a — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (9718863i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: "{}<%s - club name>{} History".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x46bf72 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x46bfae: scratch := "Competitions"
        // strcpy_scratch @ 0x46bfda: scratch := "Landmarks"
        // strcpy_scratch @ 0x46bffd: scratch := "Records"
        // strcpy_scratch @ 0x46c024: scratch := "Positions"
        // strcpy_scratch @ 0x46c08a: scratch := "Attendances"
        // strcpy_scratch @ 0x46c0b1: scratch := "Results"
        // strcpy_scratch @ 0x46c0d8: scratch := "Sequences"
        // strcpy_scratch @ 0x46c0ff: scratch := "Players"
        // strcpy_scratch @ 0x46c126: scratch := "Transfers"
        // nav_bar @ 0x46c90b — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x46c94f
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00476ef0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_476ef0 / callback_va 0x476ef0 — 11 widgets.
// GDI-REG: 00476ef0 PORTED_BEHAVIOURAL
pub fn build_screen_476ef0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x476f9c — FUN_00549580
        // TODO(unresolved-text pos=13): reg edx unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x476fbf: scratch := "Invited Clubs"
        // item @ 0x477009 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Invited Clubs".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x477012 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // area @ 0x47715a — FUN_00549790
        let _area_1 = pool.spawn_area(
            110 as i16, 145 as i16, 780 as i16, 535 as i16,
            4 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // sprintf_scratch @ 0x477195: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x477222 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Invited Clubs".into(),
            slot_40: 0,
            msg_id: 2005,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4772c9 — FUN_00549580
        // TODO(unresolved-text pos=13): reg dword ptr [esi + 0x53] unread
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 2005,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x477394: scratch := "Waiting For Reply"
        // item @ 0x4773da — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 3, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Waiting For Reply".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // nav_bar @ 0x477573 — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x477582
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `007013d0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_7013d0 / callback_va 0x7013d0 — 21 widgets.
// GDI-REG: 007013d0 PORTED_BEHAVIOURAL
pub fn build_screen_7013d0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // sprintf_scratch @ 0x7016d4: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x70171d — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 444, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (4551152i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (7i32) as u16,
            pattern: (25i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x70175c — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 0, 'literal_hex': '0x0', 'literal_signed': 0}
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 444, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sprintf_scratch @ 0x701791: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x7017e4 — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 399, grid_y0: 15,
            grid_x1: 439, grid_y1: 65,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sprintf_scratch @ 0x701817: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x701863 — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 446, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (4551152i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (7i32) as u16,
            pattern: (25i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x7018a5 — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 0, 'literal_hex': '0x0', 'literal_signed': 0}
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 446, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sprintf_scratch @ 0x7018dc: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x70192e — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 745, grid_y0: 15,
            grid_x1: 785, grid_y1: 65,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x70194d: scratch := "Agg <%d - score1>-<%d - score2> "
        // item @ 0x701995 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 451, grid_y0: 55,
            grid_x1: 740, grid_y1: 65,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 64 as u32,
            label_ink: (1i32) as u16,
            pattern: (25i32) as u16,
            text: "Agg <%d - score1>-<%d - score2> ".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x70199e — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x7019b3: scratch := "Match Overview"
        // strcpy_scratch @ 0x7019d6: scratch := "Match Stats"
        // strcpy_scratch @ 0x7019fd: scratch := "Action Zones"
        // strcpy_scratch @ 0x701a24: scratch := "Match Report"
        // strcpy_scratch @ 0x701a58: scratch := "{}<%s - team>{} Stats"
        // strcpy_scratch @ 0x701a7f: scratch := "Player Ratings"
        // strcpy_scratch @ 0x701ab3: scratch := "{}<%s - team>{} Stats"
        // nav_bar @ 0x701b1f — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x701bfb
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `0058a740.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_58a740 / callback_va 0x58a740 — 7 widgets.
// GDI-REG: 0058a740 PORTED_BEHAVIOURAL
pub fn build_screen_58a740(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x58a84a — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x58a853 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x58aa85: scratch := "Continent"
        // area @ 0x58aaf0 — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x58ab28: scratch := "{}<%s - continent e.g. European>{} Nations"
        // sprintf_scratch @ 0x58ad3b: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // nav_bar @ 0x58aef1 — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x58af35
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `0058d000.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_58d000 / callback_va 0x58d000 — 7 widgets.
// GDI-REG: 0058d000 PORTED_BEHAVIOURAL
pub fn build_screen_58d000(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x58d0ab — FUN_00549580
        // TODO(unresolved-text pos=13): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x58d0b4 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x58d0c6: scratch := "Nation"
        // strcpy_scratch @ 0x58d0e9: scratch := "Club"
        // strcpy_scratch @ 0x58d110: scratch := "Non-Player"
        // strcpy_scratch @ 0x58d137: scratch := "Player"
        // nav_bar @ 0x58d26a — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x58d2ab
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00810ce0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_810ce0 / callback_va 0x810ce0 — 8 widgets.
// GDI-REG: 00810ce0 PORTED_BEHAVIOURAL
pub fn build_screen_810ce0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // sidebar @ 0x810cf8 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x810d0a: scratch := "Championship Manager 2001/02"
        // item @ 0x810d5a — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: "Championship Manager 2001/02".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x810d69: scratch := "No Memory"
        // item @ 0x810db3 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "No Memory".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x810de5: scratch := "There is not enough available system memory to continue the game. Please free up some more and t"
        // strcpy_scratch @ 0x810e27: scratch := "Ok"
        // item @ 0x810e7e — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 100, grid_y0: 555,
            grid_x1: 790, grid_y1: 590,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (4i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 52,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x810e84
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00417870.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_417870 / callback_va 0x417870 — 8 widgets.
// GDI-REG: 00417870 PORTED_BEHAVIOURAL
pub fn build_screen_417870(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x41799e — FUN_00549580
        // TODO(unresolved-text pos=13): reg dword ptr [esi + 5] unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x4179a6 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x4179b8: scratch := "Nominations"
        // item @ 0x4179fc — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Nominations".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x417a4f — FUN_00549790
        let _area_1 = pool.spawn_area(
            110 as i16, 145 as i16, 780 as i16, 535 as i16,
            3 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // sprintf_scratch @ 0x417b33: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x417be2 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Nominations".into(),
            slot_40: 0,
            msg_id: 2007,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // nav_bar @ 0x417e28 — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x417e38
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00474760.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_474760 / callback_va 0x474760 — 21 widgets.
// GDI-REG: 00474760 PORTED_BEHAVIOURAL
pub fn build_screen_474760(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // strcpy_scratch @ 0x4747d6: scratch := "Arrange Friendly Tour"
        // item @ 0x474833 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: "Arrange Friendly Tour".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x474842: scratch := "Select Nation"
        // item @ 0x47488c — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Select Nation".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x474895 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x4748a7: scratch := "Filter"
        // item @ 0x474912 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 655, grid_y0: 145,
            grid_x1: 780, grid_y1: 165,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Filter".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x474948 — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x474959: scratch := " Available Only"
        // item @ 0x4749fd — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: " Available Only".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (1i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x474a0c: scratch := " All Nations"
        // item @ 0x474aae — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: " All Nations".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x474b03 — FUN_00549790
        let _area_2 = pool.spawn_area(
            110 as i16, 170 as i16, 780 as i16, 535 as i16,
            4 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // sprintf_scratch @ 0x474cc2: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x474d8a — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " All Nations".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x474dd1 — FUN_00549580
        // TODO(unresolved-text pos=13): reg dword ptr [edi + 0x71] unread
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x474e8e — FUN_00549790
        let _area_3 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x474e9f: scratch := "Cancel"
        // item @ 0x474ef3 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Cancel".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x474f02: scratch := "Next"
        // item @ 0x474f91 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 44 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Next".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x474fa0
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `004751b0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_4751b0 / callback_va 0x4751b0 — 14 widgets.
// GDI-REG: 004751b0 PORTED_BEHAVIOURAL
pub fn build_screen_4751b0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // strcpy_scratch @ 0x475214: scratch := "Arrange Tour Of {}<%s - nation>{}"
        // item @ 0x475271 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: "Arrange Tour Of {}<%s - nation>{}".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x475280: scratch := "Select Club"
        // item @ 0x4752cb — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Select Club".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x4752d4 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // area @ 0x475311 — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x475322: scratch := "Cancel"
        // item @ 0x475376 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Cancel".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x475385: scratch := "Next"
        // item @ 0x475418 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 44 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Next".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x47546d — FUN_00549790
        let _area_2 = pool.spawn_area(
            110 as i16, 145 as i16, 780 as i16, 535 as i16,
            6 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // sprintf_scratch @ 0x4755ba: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x475645 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Next".into(),
            slot_40: 0,
            msg_id: 2005,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x47569a — FUN_00549580
        // TODO(unresolved-text pos=13): reg dword ptr [esi + 0x53] unread
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 2005,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x4757b2
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `004e2c70.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_4e2c70 / callback_va 0x4e2c70 — 13 widgets.
// GDI-REG: 004e2c70 PORTED_BEHAVIOURAL
pub fn build_screen_4e2c70(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // sidebar @ 0x4e2cd7 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x4e2d12: scratch := "<%s - player(eg Nick Barmby)>"
        // item @ 0x4e2d5d — FUN_00549580
        // TODO(unresolved-text pos=13): stack frame_offset=8
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (2i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4e2d6f: scratch := "Transfer Request"
        // item @ 0x4e2dbd — FUN_00549580
        // TODO(unresolved-text pos=13): stack frame_offset=2008
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4e2dd4: scratch := "Put <%s - player(eg Nick Barmby)> on the transfer list at his own request ?"
        // area @ 0x4e2e44 — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            3 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x4e2e55: scratch := "Back"
        // item @ 0x4e2ea9 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Back".into(),
            slot_40: 0,
            msg_id: 34,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4e2eb8: scratch := "No"
        // item @ 0x4e2f0c — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "No".into(),
            slot_40: 0,
            msg_id: 33,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4e2f1b: scratch := "Yes"
        // item @ 0x4e2f6f — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Yes".into(),
            slot_40: 0,
            msg_id: 32,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x4e2f7c
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `004e38d0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_4e38d0 / callback_va 0x4e38d0 — 15 widgets.
// GDI-REG: 004e38d0 PORTED_BEHAVIOURAL
pub fn build_screen_4e38d0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x4e3a55 — FUN_00549580
        // TODO(unresolved-text pos=13): stack frame_offset=-532
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4e3a64: scratch := "Set Role At Club"
        // item @ 0x4e3aaf — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Set Role At Club".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4e3af0 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 6 as u32,
            grid_x0: 125, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: "Set Role At Club".into(),
            slot_40: 0,
            msg_id: 2007,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x4e3b45 — FUN_00549790
        let _area_1 = pool.spawn_area(
            110 as i16, 140 as i16, 780 as i16, 545 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x4e3be6 — FUN_00549790
        let _area_2 = pool.spawn_area(
            110 as i16, 140 as i16, 780 as i16, 545 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x4e3e37 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Set Role At Club".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4e3e6a: scratch := "Job Description"
        // item @ 0x4e3eaf — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Job Description".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 5i16)?;
        // sidebar @ 0x4e4133 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // area @ 0x4e4173 — FUN_00549790
        let _area_3 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x4e4184: scratch := "Back"
        // item @ 0x4e41d8 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Back".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4e41ec: scratch := "Set<%s - COMMENT - set squad status>"
        // item @ 0x4e4299 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Set<%s - COMMENT - set squad status>".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x4e42a8
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `004e42b0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_4e42b0 / callback_va 0x4e42b0 — 0 widgets.
// GDI-REG: 004e42b0 PORTED_PARTIAL
pub fn build_screen_4e42b0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // ret @ 0x4e434c
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `004e6680.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_4e6680 / callback_va 0x4e6680 — 0 widgets.
// GDI-REG: 004e6680 PORTED_PARTIAL
pub fn build_screen_4e6680(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // ret @ 0x4e671c
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `004ebfa0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_4ebfa0 / callback_va 0x4ebfa0 — 10 widgets.
// GDI-REG: 004ebfa0 PORTED_BEHAVIOURAL
pub fn build_screen_4ebfa0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x4ec062 — FUN_00549580
        // TODO(unresolved-text pos=13): stack frame_offset=-304
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4ec071: scratch := "Terminate Contract"
        // item @ 0x4ec0bc — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Terminate Contract".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4ec0e8: scratch := "Terminate <%s - Staff Name(e.g.Derek Ferguson)>{s} contract ?"
        // sidebar @ 0x4ec122 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // area @ 0x4ec162 — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x4ec173: scratch := "No"
        // item @ 0x4ec1c7 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "No".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4ec1d6: scratch := "Yes"
        // item @ 0x4ec22a — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Yes".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x4ec238
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `004ec590.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_4ec590 / callback_va 0x4ec590 — 38 widgets.
// GDI-REG: 004ec590 PORTED_BEHAVIOURAL
pub fn build_screen_4ec590(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x4ec5f1 — FUN_00549580
        // TODO(unresolved-text pos=13): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x4ec5fa — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x4ec60c: scratch := "Game Credits"
        // item @ 0x4ec657 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Game Credits".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4ec666: scratch := "View"
        // item @ 0x4ec6cd — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 110, grid_y0: 145,
            grid_x1: 235, grid_y1: 165,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "View".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x4ec703 — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x4ec75b — FUN_00549790
        let _area_2 = pool.spawn_area(
            110 as i16, 170 as i16, 780 as i16, 535 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x4ec82c — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "View".into(),
            slot_40: 0,
            msg_id: 100,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4ec88f — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "View".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4ec8f1 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 1,
            style_byte: 50176 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 0 as u32,
            label_ink: (0i32) as u16,
            pattern: (0i32) as u16,
            text: "View".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4ec900: scratch := "Chairman"
        // item @ 0x4ec94c — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 440, grid_y1: 0,
            seq: 0, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Oliver Collyer".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4ec997: scratch := "Chairman"
        // strcpy_scratch @ 0x4eca25: scratch := "Managing Director<%s - COMMENT - non-football managing director>"
        // strcpy_scratch @ 0x4ecaae: scratch := "Design & Programming"
        // strcpy_scratch @ 0x4ecb38: scratch := "Design & Programming"
        // strcpy_scratch @ 0x4ecbc1: scratch := "Design & Programming"
        // strcpy_scratch @ 0x4ecc4a: scratch := "Design & Programming"
        // strcpy_scratch @ 0x4eccd4: scratch := "Design & Programming"
        // strcpy_scratch @ 0x4ecd5d: scratch := "Design & Programming"
        // strcpy_scratch @ 0x4ecde6: scratch := "Design & Programming"
        // strcpy_scratch @ 0x4ece70: scratch := "Research Co-ordination"
        // strcpy_scratch @ 0x4ecef9: scratch := "Research Co-ordination"
        // strcpy_scratch @ 0x4ecf82: scratch := "Internet Development"
        // strcpy_scratch @ 0x4ed00c: scratch := "Sound Effects"
        // strcpy_scratch @ 0x4ed095: scratch := "Manual & Written History"
        // strcpy_scratch @ 0x4ed15a: scratch := "Research"
        // strcpy_scratch @ 0x4ed23e: scratch := "Argentina"
        // strcpy_scratch @ 0x4ed2c8: scratch := "Australia"
        // strcpy_scratch @ 0x4ed351: scratch := "Austria"
        // strcpy_scratch @ 0x4ed3da: scratch := "Belgium"
        // strcpy_scratch @ 0x4ed464: scratch := "Belgium"
        // strcpy_scratch @ 0x4ed4ed: scratch := "Brazil"
        // strcpy_scratch @ 0x4ed576: scratch := "China"
        // strcpy_scratch @ 0x4ed600: scratch := "Croatia"
        // strcpy_scratch @ 0x4ed689: scratch := "Czech Republic"
        // strcpy_scratch @ 0x4ed712: scratch := "Denmark"
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `004fd1f0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_4fd1f0 / callback_va 0x4fd1f0 — 25 widgets.
// GDI-REG: 004fd1f0 PORTED_BEHAVIOURAL
pub fn build_screen_4fd1f0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x4fd24f — FUN_00549580
        // TODO(unresolved-text pos=13): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x4fd258 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x4fd26a: scratch := "Web Sites"
        // item @ 0x4fd2b5 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Web Sites".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x4fd307 — FUN_00549790
        let _area_1 = pool.spawn_area(
            110 as i16, 145 as i16, 780 as i16, 535 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x4fd390 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Web Sites".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd3f2 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 1,
            style_byte: 50176 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 0 as u32,
            label_ink: (0i32) as u16,
            pattern: (0i32) as u16,
            text: "Web Sites".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4fd401: scratch := "The developers"
        // item @ 0x4fd44d — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 760, grid_y1: 0,
            seq: 0, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "www.sigames.com".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd497 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 450, grid_y0: 0,
            grid_x1: 760, grid_y1: 0,
            seq: 0, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "The developers".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4fd4a6: scratch := "Official Messageboard"
        // item @ 0x4fd4f3 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 760, grid_y1: 0,
            seq: 0, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "community.sigames.com".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd53c — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 450, grid_y0: 0,
            grid_x1: 760, grid_y1: 0,
            seq: 0, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Official Messageboard".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd586 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 760, grid_y1: 0,
            seq: 0, row_index: 4,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Official Messageboard".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd60d — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 5,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Official Messageboard".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd66f — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 6,
            style_byte: 50176 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 0 as u32,
            label_ink: (0i32) as u16,
            pattern: (0i32) as u16,
            text: "Official Messageboard".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4fd67e: scratch := "The publishers"
        // item @ 0x4fd6ca — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 760, grid_y1: 0,
            seq: 0, row_index: 7,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "www.eidos.co.uk".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd714 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 450, grid_y0: 0,
            grid_x1: 760, grid_y1: 0,
            seq: 0, row_index: 7,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "The publishers".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd75e — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 760, grid_y1: 0,
            seq: 0, row_index: 8,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "The publishers".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd7d2 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 760, grid_y1: 0,
            seq: 0, row_index: 9,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "The publishers".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x4fd7e1: scratch := "Unofficial Sites"
        // item @ 0x4fd844 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 10,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Unofficial Sites".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x4fd8ab — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 130, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 11,
            style_byte: 50176 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 0 as u32,
            label_ink: (0i32) as u16,
            pattern: (0i32) as u16,
            text: "Unofficial Sites".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // nav_bar @ 0x4fda5f — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x4fda70
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00548560.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_548560 / callback_va 0x548560 — 11 widgets.
// GDI-REG: 00548560 PORTED_BEHAVIOURAL
pub fn build_screen_548560(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // sidebar @ 0x5485f2 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // item @ 0x548674 — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x548683: scratch := "Please Confirm"
        // item @ 0x5486ce — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Please Confirm".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sprintf_scratch @ 0x5486fd: stack frame_offset=-524
        // strcpy_scratch @ 0x54874b: scratch := "Appeal against the <%s - number (ie. 3)>{} game ban that has been placed on <%s - Staff Name (eg"
        // area @ 0x548816 — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x548827: scratch := "No"
        // item @ 0x54887b — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "No".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x54888a: scratch := "Yes"
        // item @ 0x5488de — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Yes".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x5488ed
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `005488f0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_5488f0 / callback_va 0x5488f0 — 0 widgets.
// GDI-REG: 005488f0 PORTED_PARTIAL
pub fn build_screen_5488f0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // ret @ 0x54895c
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `005dad10.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_5dad10 / callback_va 0x5dad10 — 15 widgets.
// GDI-REG: 005dad10 PORTED_BEHAVIOURAL
pub fn build_screen_5dad10(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // area @ 0x5dae8e — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 799 as i16, 599 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x5daecd — FUN_00549790
        let _area_2 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x5daf2f — FUN_00549580
        // TODO(unresolved-text pos=13): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (4i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x5daffc — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 11294452, 'literal_hex': '0xac56f4', 'literal_signed': 11294452}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 4,
            seq: 0, row_index: 0,
            style_byte: 97 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (2i32) as u16,
            pattern: (6i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x5db0f5 — FUN_00549580
        // TODO(unresolved-text pos=13): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1024 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (20i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 1,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x5db15f — FUN_00549580
        // TODO(unresolved-text pos=13): reg edx unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 50 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x5db1dc — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 11294452, 'literal_hex': '0xac56f4', 'literal_signed': 11294452}
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 32 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x5db20f — FUN_00549790
        let _area_3 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x5db282 — FUN_00549580
        // TODO(unresolved-text pos=13): global DAT_00b5d016
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00ad6bda
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 144 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x5db2db — FUN_00549790
        let _area_4 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x5db308 — FUN_00549790
        let _area_5 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x5db34d — FUN_00549790
        let _area_6 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x5db4f6 — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 11294452, 'literal_hex': '0xac56f4', 'literal_signed': 11294452}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 0i16)?;
        // strcpy_scratch @ 0x5db517: scratch := "Ok"
        // item @ 0x5db57b — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 20, grid_y0: 0,
            grid_x1: 21, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (4i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x5db58a
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `005db600.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_5db600 / callback_va 0x5db600 — 0 widgets.
// GDI-REG: 005db600 PORTED_PARTIAL
pub fn build_screen_5db600(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // ret @ 0x5db752
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00697440.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_697440 / callback_va 0x697440 — 10 widgets.
// GDI-REG: 00697440 PORTED_BEHAVIOURAL
pub fn build_screen_697440(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // sidebar @ 0x69746a — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // item @ 0x6974ec — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x6974fb: scratch := "Please Confirm"
        // item @ 0x697546 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Please Confirm".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x69755f: scratch := "Resign as manager of {}<%s - club>{} ?"
        // area @ 0x6975cb — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x6975dc: scratch := "No"
        // item @ 0x697630 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "No".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x69763f: scratch := "Yes"
        // item @ 0x697693 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Yes".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x69769c
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00697dc0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_697dc0 / callback_va 0x697dc0 — 10 widgets.
// GDI-REG: 00697dc0 PORTED_BEHAVIOURAL
pub fn build_screen_697dc0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // sidebar @ 0x697dea — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // item @ 0x697e6c — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x697e7b: scratch := "Please Confirm"
        // item @ 0x697ec6 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Please Confirm".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x697edf: scratch := "Accept the position as manager of {}<%s - club>{} ?"
        // area @ 0x697f4b — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x697f5c: scratch := "No"
        // item @ 0x697fb0 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "No".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x697fbf: scratch := "Yes"
        // item @ 0x698013 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Yes".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x69801c
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `006fd7b0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_6fd7b0 / callback_va 0x6fd7b0 — 13 widgets.
// GDI-REG: 006fd7b0 PORTED_BEHAVIOURAL
pub fn build_screen_6fd7b0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x6fd8e2 — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x6fd944 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x6fda76: scratch := "{}<%s - time of day>{} Results"
        // item @ 0x6fdaba — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "{}<%s - time of day>{} Results".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x6fdb04 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 105, grid_y0: 15,
            grid_x1: 120, grid_y1: 35,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "{}<%s - time of day>{} Results".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x6fdb2e — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x6fdd30 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "{}<%s - time of day>{} Results".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x6fdd62 — FUN_00549790
        let _area_2 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x6fdebd — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "{}<%s - time of day>{} Results".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x6fdff8 — FUN_00549790
        let _area_3 = pool.spawn_area(
            110 as i16, 145 as i16, 780 as i16, 535 as i16,
            9 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x6fe19e — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 115, grid_y0: 0,
            grid_x1: 753, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "{}<%s - time of day>{} Results".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x6fe1e9 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 115, grid_y0: 0,
            grid_x1: 753, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 50176 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 0 as u32,
            label_ink: (0i32) as u16,
            pattern: (0i32) as u16,
            text: "{}<%s - time of day>{} Results".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // nav_bar @ 0x6fe2bc — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x6fe41b
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `007cdc70.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_7cdc70 / callback_va 0x7cdc70 — 10 widgets.
// GDI-REG: 007cdc70 PORTED_BEHAVIOURAL
pub fn build_screen_7cdc70(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // sidebar @ 0x7cdcd8 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // item @ 0x7cdd47 — FUN_00549580
        // TODO(unresolved-text pos=13): stack frame_offset=-112
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x7cdd56: scratch := "Please Confirm"
        // item @ 0x7cdda1 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Please Confirm".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x7cddeb: scratch := "Are you sure that you wish to send <%s - Player Name (eg. Paul Di Canio)> on leave of absence fo"
        // area @ 0x7cde58 — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x7cde69: scratch := "No"
        // item @ 0x7cdebe — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "No".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x7cdecd: scratch := "Yes"
        // item @ 0x7cdf22 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Yes".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x7cdf2d
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `007fb050.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_7fb050 / callback_va 0x7fb050 — 43 widgets.
// GDI-REG: 007fb050 PORTED_BEHAVIOURAL
pub fn build_screen_7fb050(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // area @ 0x7fb315 — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 799 as i16, 599 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x7fb397 — FUN_00549790
        let _area_2 = pool.spawn_area(
            107 as i16, 600 as i16, 781 as i16, 0 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x7fb3c6: scratch := "Filters for <%s - search name>"
        // item @ 0x7fb417 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 117, grid_y0: 600,
            grid_x1: 771, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (4i32) as u16,
            pattern: (0i32) as u16,
            text: "Filters for <%s - search name>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x7fb44c — FUN_00549790
        let _area_3 = pool.spawn_area(
            117 as i16, 600 as i16, 367 as i16, 0 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x7fb462: scratch := "General<%s - COMMENT - search filters toggle>"
        // item @ 0x7fb4f3 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "General<%s - COMMENT - search filters toggle>".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (1i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x7fb507: scratch := "Attributes<%s - COMMENT - search filters toggle>"
        // item @ 0x7fb598 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Attributes<%s - COMMENT - search filters toggle>".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (2i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x7fb5ac: scratch := "Clear<%s - COMMENT - search filters>"
        // item @ 0x7fb65a — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 646, grid_y0: 600,
            grid_x1: 771, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Clear<%s - COMMENT - search filters>".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x7fb68f — FUN_00549790
        let _area_4 = pool.spawn_area(
            117 as i16, 0 as i16, 771 as i16, 0 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x7fb6a0: scratch := "Cancel"
        // item @ 0x7fb6f5 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Cancel".into(),
            slot_40: 0,
            msg_id: 8,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x7fb704: scratch := "Ok"
        // item @ 0x7fb759 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 9,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x7fb7d6 — FUN_00549790
        let _area_5 = pool.spawn_area(
            132 as i16, 600 as i16, 756 as i16, 0 as i16,
            7 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x7fb80f — FUN_00549790
        let _area_6 = pool.spawn_area(
            132 as i16, 600 as i16, 756 as i16, 0 as i16,
            7 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x7fb848 — FUN_00549790
        let _area_7 = pool.spawn_area(
            132 as i16, 600 as i16, 756 as i16, 0 as i16,
            7 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x7fb881 — FUN_00549790
        let _area_8 = pool.spawn_area(
            132 as i16, 600 as i16, 756 as i16, 0 as i16,
            7 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x7fb8e2 — FUN_00549790
        let _area_9 = pool.spawn_area(
            132 as i16, 600 as i16, 756 as i16, 0 as i16,
            9 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x7fb943 — FUN_00549790
        let _area_10 = pool.spawn_area(
            132 as i16, 600 as i16, 756 as i16, 0 as i16,
            9 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x7fb97c — FUN_00549790
        let _area_11 = pool.spawn_area(
            132 as i16, 600 as i16, 756 as i16, 0 as i16,
            7 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // sprintf_scratch @ 0x7fba31: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x7fbabe — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x7fbb6a — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sprintf_scratch @ 0x7fbb8d: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x7fbbef — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 18,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x7fbc9b — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 5,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x7fbce3: scratch := " Match<%s - attributes to match>"
        // item @ 0x7fbd2d — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): stack frame_offset=-1562
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 5, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: " Match<%s - attributes to match>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x7fbdb5 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): stack frame_offset=-1562
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 6, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: " Match<%s - attributes to match>".into(),
            slot_40: 0,
            msg_id: 6,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x7fbdce: scratch := "<%d - number1>/<%d - number2>"
        // item @ 0x7fbe14 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): stack frame_offset=-1562
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 7, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "<%d - number1>/<%d - number2>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sprintf_scratch @ 0x7fbec4: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x7fbf51 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "<%d - number1>/<%d - number2>".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (0i64) as u32,
        }, 1i16)?;
        // item @ 0x7fbffd — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "<%d - number1>/<%d - number2>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (0i64) as u32,
        }, 1i16)?;
        // sprintf_scratch @ 0x7fc020: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x7fc082 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "<%d - number1>/<%d - number2>".into(),
            slot_40: 0,
            msg_id: 18,
            userdata_id: (0i64) as u32,
        }, 1i16)?;
        // item @ 0x7fc12e — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "<%d - number1>/<%d - number2>".into(),
            slot_40: 0,
            msg_id: 5,
            userdata_id: (0i64) as u32,
        }, 1i16)?;
        // strcpy_scratch @ 0x7fc176: scratch := " Match<%s - attributes to match>"
        // item @ 0x7fc1c0 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): stack frame_offset=-1634
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 5, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: " Match<%s - attributes to match>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x7fc248 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): stack frame_offset=-1634
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 6, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: " Match<%s - attributes to match>".into(),
            slot_40: 0,
            msg_id: 6,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00804020.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_804020 / callback_va 0x804020 — 16 widgets.
// GDI-REG: 00804020 PORTED_BEHAVIOURAL
pub fn build_screen_804020(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // strcpy_scratch @ 0x80407e: scratch := "Championship Manager 2001/02"
        // item @ 0x8040cf — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: "Championship Manager 2001/02".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x8040d8 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x8040ea: scratch := "Setup Game"
        // item @ 0x804135 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Setup Game".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x804144: scratch := "Start New Game"
        // strcpy_scratch @ 0x804164: scratch := "Quick Start Game"
        // strcpy_scratch @ 0x804184: scratch := "Restore Saved Game"
        // strcpy_scratch @ 0x8041a4: scratch := "Delete Saved Game"
        // strcpy_scratch @ 0x8041c4: scratch := "Network Play"
        // strcpy_scratch @ 0x804209: scratch := "Game Settings"
        // strcpy_scratch @ 0x804229: scratch := "Hall Of Fame"
        // strcpy_scratch @ 0x804249: scratch := "Game Credits"
        // strcpy_scratch @ 0x804269: scratch := "Web Sites"
        // menu_list @ 0x80429b — FUN_005d6bf0 (TODO: dedicated port)
        // nav_bar @ 0x8042b7 — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x804338
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00808ae0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_808ae0 / callback_va 0x808ae0 — 15 widgets.
// GDI-REG: 00808ae0 PORTED_BEHAVIOURAL
pub fn build_screen_808ae0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // strcpy_scratch @ 0x808b0b: scratch := "Championship Manager 2001/02"
        // item @ 0x808b5b — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: "Championship Manager 2001/02".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x808b64 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // strcpy_scratch @ 0x808b76: scratch := "Manager Status"
        // item @ 0x808bc0 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Manager Status".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x808e25 — FUN_00549790
        let _area_1 = pool.spawn_area(
            110 as i16, 145 as i16, 780 as i16, 535 as i16,
            3 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // sprintf_scratch @ 0x808e84: unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        // item @ 0x808eca — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Manager Status".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x809256: scratch := "Waiting"
        // item @ 0x80929c — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Waiting".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x80937f — FUN_00549790
        let _area_2 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x809390: scratch := "Back"
        // item @ 0x8093d2 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 44 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Back".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x809412: scratch := "Finish"
        // nav_bar @ 0x80952a — FUN_005d75b0
        // TODO(unresolved-arg pos=0 back_flag): reg eax unread
        spawn_navbar_stub(pool, _root_area as i16, false, false)?;
        // ret @ 0x8096d5
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `0088def0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_88def0 / callback_va 0x88def0 — 45 widgets.
// GDI-REG: 0088def0 PORTED_BEHAVIOURAL
pub fn build_screen_88def0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // area @ 0x88dfaf — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 799 as i16, 599 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x88e011 — FUN_00549790
        let _area_2 = pool.spawn_area(
            170 as i16, 600 as i16, 719 as i16, 0 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x88e026: scratch := "Team Instructions"
        // item @ 0x88e07a — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 180, grid_y0: 600,
            grid_x1: 709, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (4i32) as u16,
            pattern: (0i32) as u16,
            text: "Team Instructions".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x88e0b5 — FUN_00549790
        let _area_3 = pool.spawn_area(
            180 as i16, 0 as i16, 709 as i16, 0 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x88e186: scratch := "Ok"
        // item @ 0x88e1df — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 180, grid_y0: 0,
            grid_x1: 709, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x88e288 — FUN_00549790
        let _area_4 = pool.spawn_area(
            195 as i16, 600 as i16, 694 as i16, 0 as i16,
            5 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x88e2a7: scratch := " Mentality"
        // item @ 0x88e2ee — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Mentality".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88e302: scratch := "Normal<%s - COMMENT - playing style>"
        // item @ 0x88e3e8 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Normal<%s - COMMENT - playing style>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88e3fc: scratch := "Defensive<%s - COMMENT - playing style>"
        // item @ 0x88e4e0 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Defensive<%s - COMMENT - playing style>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88e4f4: scratch := "Attacking<%s - COMMENT - playing style>"
        // item @ 0x88e5da — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 3, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Attacking<%s - COMMENT - playing style>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88e5e9: scratch := " Passing"
        // item @ 0x88e630 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Passing".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88e644: scratch := "Mixed<%s - COMMENT - passing instruction>"
        // item @ 0x88e724 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Mixed<%s - COMMENT - passing instruction>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88e738: scratch := "Short<%s - COMMENT - passing instruction>"
        // item @ 0x88e818 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Short<%s - COMMENT - passing instruction>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88e82c: scratch := "Direct<%s - COMMENT - passing instruction>"
        // item @ 0x88e90c — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 3, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Direct<%s - COMMENT - passing instruction>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88e920: scratch := "Long<%s - COMMENT - passing instruction>"
        // item @ 0x88ea00 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 4, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Long<%s - COMMENT - passing instruction>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88ea0f: scratch := " Tackling"
        // item @ 0x88ea56 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Tackling".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88ea6a: scratch := "Normal<%s - COMMENT - tackling>"
        // item @ 0x88eb4a — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Normal<%s - COMMENT - tackling>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88eb5e: scratch := "Easy<%s - COMMENT - tackling>"
        // item @ 0x88ec3e — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Easy<%s - COMMENT - tackling>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88ec52: scratch := "Hard<%s - COMMENT - tackling>"
        // item @ 0x88ed32 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 3, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Hard<%s - COMMENT - tackling>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88ed41: scratch := " Pressing"
        // item @ 0x88ed88 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Pressing".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88ed97: scratch := "No"
        // item @ 0x88ee7a — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "No".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88ee89: scratch := "Yes"
        // item @ 0x88ef6d — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Yes".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88ef7c: scratch := " Offside Trap"
        // item @ 0x88efc3 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 4,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Offside Trap".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88efd2: scratch := "No"
        // item @ 0x88f0b6 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 4,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "No".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x88f0c5: scratch := "Yes"
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00890e50.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_890e50 / callback_va 0x890e50 — 56 widgets.
// GDI-REG: 00890e50 PORTED_BEHAVIOURAL
pub fn build_screen_890e50(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // area @ 0x890f24 — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 799 as i16, 599 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x890f6a — FUN_00549790
        let _area_2 = pool.spawn_area(
            145 as i16, 55 as i16, 744 as i16, 544 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x890fd4 — FUN_00549580
        // TODO(unresolved-text pos=13): unknown {'literal': 14447164, 'literal_hex': '0xdc723c', 'literal_signed': 14447164}
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 155, grid_y0: 65,
            grid_x1: 734, grid_y1: 110,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (4i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x891153 — FUN_00549790
        let _area_3 = pool.spawn_area(
            155 as i16, 504 as i16, 734 as i16, 534 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x891164: scratch := "Cancel"
        // item @ 0x8911b8 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Cancel".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8911c7: scratch := "Ok"
        // item @ 0x89121b — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x891290 — FUN_00549790
        let _area_4 = pool.spawn_area(
            170 as i16, 135 as i16, 719 as i16, 479 as i16,
            6 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x8912ce: scratch := " Passing"
        // item @ 0x891314 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Passing".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891328: scratch := "Team<%s - COMMENT - passing instruction>"
        // item @ 0x8913b5 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Team<%s - COMMENT - passing instruction>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (1i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8913c9: scratch := "Mixed<%s - COMMENT - passing instruction>"
        // item @ 0x89145b — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Mixed<%s - COMMENT - passing instruction>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (2i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x89146f: scratch := "Short<%s - COMMENT - passing instruction>"
        // item @ 0x8914fc — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 3, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Short<%s - COMMENT - passing instruction>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (4i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891510: scratch := "Direct<%s - COMMENT - passing instruction>"
        // item @ 0x8915a2 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 4, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Direct<%s - COMMENT - passing instruction>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (8i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8915b6: scratch := "Long<%s - COMMENT - passing instruction>"
        // item @ 0x891643 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 5, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Long<%s - COMMENT - passing instruction>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (16i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891652: scratch := " Tackling"
        // item @ 0x891698 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Tackling".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8916ac: scratch := "Team<%s - COMMENT - tackling>"
        // item @ 0x89173f — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Team<%s - COMMENT - tackling>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (512i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891753: scratch := "Normal<%s - COMMENT - tackling>"
        // item @ 0x8917ea — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Normal<%s - COMMENT - tackling>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (1024i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8917fe: scratch := "Easy<%s - COMMENT - tackling>"
        // item @ 0x891891 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 3, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Easy<%s - COMMENT - tackling>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (2048i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8918a5: scratch := "Hard<%s - COMMENT - tackling>"
        // item @ 0x89193c — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 4, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Hard<%s - COMMENT - tackling>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (4096i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x89194b: scratch := " Pressing"
        // item @ 0x891992 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Pressing".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8919a6: scratch := "Team<%s - COMMENT - pressing>"
        // item @ 0x891a3d — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Team<%s - COMMENT - pressing>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (8192i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891a51: scratch := "No<%s - COMMENT - pressing>"
        // item @ 0x891ae4 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "No<%s - COMMENT - pressing>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (16384i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891af8: scratch := "Yes<%s - COMMENT - pressing>"
        // item @ 0x891b8f — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 3, row_index: 2,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Yes<%s - COMMENT - pressing>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (32768i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891b9e: scratch := " Pass To"
        // item @ 0x891be5 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Pass To".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891bf9: scratch := "R/L/C<%s - COMMENT - pass to>"
        // item @ 0x891c8b — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "R/L/C<%s - COMMENT - pass to>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (32i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891c9f: scratch := "Left<%s - COMMENT - pass to>"
        // item @ 0x891d2d — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Left<%s - COMMENT - pass to>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (64i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891d41: scratch := "Centre<%s - COMMENT - pass to>"
        // item @ 0x891dd8 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 3, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Centre<%s - COMMENT - pass to>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (128i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891dec: scratch := "Right<%s - COMMENT - pass to>"
        // item @ 0x891e7f — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 4, row_index: 3,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Right<%s - COMMENT - pass to>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (256i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891e8e: scratch := " Set Pieces (A)"
        // item @ 0x891ed4 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 4,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Set Pieces (A)".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891ee8: scratch := "Normal<%s - COMMENT - normal position for attacking set pieces>"
        // item @ 0x891f7b — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 4,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Normal<%s - COMMENT - normal position for attacking set pieces>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (65536i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x891f8f: scratch := "Back<%s - COMMENT - drop back for attacking set pieces>"
        // item @ 0x892026 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 4,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Back<%s - COMMENT - drop back for attacking set pieces>".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (1073741824i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x89203a: scratch := "Forward<%s - COMMENT - push forward for attacking set pieces>"
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `00893500.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_893500 / callback_va 0x893500 — 0 widgets.
// GDI-REG: 00893500 PORTED_PARTIAL
pub fn build_screen_893500(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // ret @ 0x8935c4
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `008a2180.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_8a2180 / callback_va 0x8a2180 — 37 widgets.
// GDI-REG: 008a2180 PORTED_BEHAVIOURAL
pub fn build_screen_8a2180(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // strcpy_scratch @ 0x8a242d: scratch := "{}<%s - Club Name(e.g.Falkirk)>{} Training"
        // item @ 0x8a2478 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (10i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (-1i32) as u16,
            text: "{}<%s - Club Name(e.g.Falkirk)>{} Training".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // sidebar @ 0x8a2481 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // area @ 0x8a268e — FUN_00549790
        let _area_1 = pool.spawn_area(
            110 as i16, 80 as i16, 485 as i16, 0 as i16,
            3 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x8a26a8: scratch := "Schedule<%s - COMMENT - training schedule>"
        // item @ 0x8a272d — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Schedule<%s - COMMENT - training schedule>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x8a2763 — FUN_00549790
        let _area_2 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x8a2774: scratch := "Load"
        // item @ 0x8a27c6 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Load".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a27d5: scratch := "Save"
        // item @ 0x8a289a — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 33 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Save".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a28a9: scratch := "Save As"
        // item @ 0x8a295f — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 33 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Save As".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a296e: scratch := "Delete"
        // item @ 0x8a29c0 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Delete".into(),
            slot_40: 0,
            msg_id: 15,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a2bdf: scratch := "View"
        // item @ 0x8a2c40 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "View".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 2i16)?;
        // area @ 0x8a2c76 — FUN_00549790
        let _area_3 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x8a2c8c: scratch := "  General<%s - COMMENT - club view menu>"
        // item @ 0x8a2d2d — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "  General<%s - COMMENT - club view menu>".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (1i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a2d3c: scratch := "  Attributes"
        // item @ 0x8a2ddd — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "  Attributes".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (4i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a2dfa: scratch := "Attributes"
        // item @ 0x8a2e58 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Attributes".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 2i16)?;
        // area @ 0x8a2e8e — FUN_00549790
        let _area_4 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x8a2e9f: scratch := "  Physical"
        // item @ 0x8a2f40 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "  Physical".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (16i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a2f4f: scratch := "  Mental"
        // item @ 0x8a2ff0 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "  Mental".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (32i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a2fff: scratch := "  Goalkeeping"
        // item @ 0x8a30a0 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "  Goalkeeping".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (64i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a30af: scratch := "  Defensive"
        // item @ 0x8a3153 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "  Defensive".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (128i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a3162: scratch := "  Attacking"
        // item @ 0x8a325c — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "  Attacking".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (256i64) as u32,
        }, -1i16)?;
        // area @ 0x8a3295 — FUN_00549790
        let _area_5 = pool.spawn_area(
            530 as i16, 1 as i16, 780 as i16, 0 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x8a3324: scratch := "Edit"
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `008a6580.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_8a6580 / callback_va 0x8a6580 — 37 widgets.
// GDI-REG: 008a6580 PORTED_BEHAVIOURAL
pub fn build_screen_8a6580(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // area @ 0x8a66ca — FUN_00549790
        let _area_1 = pool.spawn_area(
            0 as i16, 0 as i16, 799 as i16, 599 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x8a6710 — FUN_00549790
        let _area_2 = pool.spawn_area(
            160 as i16, 125 as i16, 729 as i16, 474 as i16,
            1 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x8a677a — FUN_00549580
        // TODO(unresolved-text pos=13): reg edx unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 170, grid_y0: 135,
            grid_x1: 719, grid_y1: 180,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (4i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x8a67b7 — FUN_00549790
        let _area_3 = pool.spawn_area(
            170 as i16, 434 as i16, 719 as i16, 464 as i16,
            2 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x8a67c8: scratch := "Cancel"
        // item @ 0x8a681c — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Cancel".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a682b: scratch := "Ok"
        // item @ 0x8a68bf — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Ok".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // area @ 0x8a6934 — FUN_00549790
        let _area_4 = pool.spawn_area(
            185 as i16, 205 as i16, 704 as i16, 409 as i16,
            4 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x8a6973 — FUN_00549790
        let _area_5 = pool.spawn_area(
            185 as i16, 205 as i16, 704 as i16, 409 as i16,
            5 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // area @ 0x8a69b2 — FUN_00549790
        let _area_6 = pool.spawn_area(
            185 as i16, 205 as i16, 704 as i16, 409 as i16,
            5 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x8a69c5: scratch := " Name"
        // item @ 0x8a6a0b — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Name".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x8a6a4b — FUN_00549580
        // TODO(unresolved-text pos=13): reg eax unread
        // TODO(unresolved-arg pos=17 area_handle): global DAT_00b5d016
        pool.spawn_widget(WidgetDescriptor {
            kind: 24 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 0,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 8,
            msg_id: 9,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x8a6adb — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " Name".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x8a6b99 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: " Name".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x8a6c39 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: " Name".into(),
            slot_40: 0,
            msg_id: 4,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x8a6cda — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 3, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: " Name".into(),
            slot_40: 0,
            msg_id: 5,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // item @ 0x8a6d7a — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 4, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: " Name".into(),
            slot_40: 0,
            msg_id: 6,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a6da1: scratch := " New Position"
        // item @ 0x8a6deb — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " New Position".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 0i16)?;
        // strcpy_scratch @ 0x8a6dff: scratch := "None<%s - COMMENT - new training position>"
        // item @ 0x8a6e95 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 1,
            style_byte: 545 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "None<%s - COMMENT - new training position>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 0i16)?;
        // strcpy_scratch @ 0x8a6ea9: scratch := "Specific<%s - COMMENT - new training position>"
        // item @ 0x8a6f3f — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Specific<%s - COMMENT - new training position>".into(),
            slot_40: 0,
            msg_id: 7,
            userdata_id: (1i64) as u32,
        }, 0i16)?;
        // item @ 0x8a6fa8 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 4, row_index: 1,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Specific<%s - COMMENT - new training position>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 0i16)?;
        // area @ 0x8a6fe7 — FUN_00549790
        let _area_7 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x8a7062 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Specific<%s - COMMENT - new training position>".into(),
            slot_40: 0,
            msg_id: 7,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8a7123: scratch := " New Side"
        // item @ 0x8a7169 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: " New Side".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 0i16)?;
        // strcpy_scratch @ 0x8a717d: scratch := "None<%s - COMMENT - new training side>"
        // item @ 0x8a7213 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 1,
            style_byte: 545 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "None<%s - COMMENT - new training side>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 0i16)?;
        // strcpy_scratch @ 0x8a7227: scratch := "Specific<%s - COMMENT - new training side>"
        // item @ 0x8a72c0 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 1,
            style_byte: 1 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (2i32) as u16,
            pattern: (0i32) as u16,
            text: "Specific<%s - COMMENT - new training side>".into(),
            slot_40: 0,
            msg_id: 8,
            userdata_id: (128i64) as u32,
        }, 0i16)?;
        // item @ 0x8a7329 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 4, row_index: 1,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Specific<%s - COMMENT - new training side>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, 0i16)?;
        // area @ 0x8a7368 — FUN_00549790
        let _area_8 = pool.spawn_area(
            0 as i16, 0 as i16, 0 as i16, 0 as i16,
            0 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // item @ 0x8a73e3 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 16 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 1 as u32,
            label_ink: (1i32) as u16,
            pattern: (0i32) as u16,
            text: "Specific<%s - COMMENT - new training side>".into(),
            slot_40: 0,
            msg_id: 8,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x8a749c
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `008e01d0.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_8e01d0 / callback_va 0x8e01d0 — 12 widgets.
// GDI-REG: 008e01d0 PORTED_BEHAVIOURAL
pub fn build_screen_8e01d0(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // item @ 0x8e02e6 — FUN_00549580
        // TODO(unresolved-text pos=13): reg dword ptr [ebp + 0x39] unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 10,
            grid_x1: 790, grid_y1: 70,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (2i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (7i32) as u16,
            pattern: (0i32) as u16,
            text: String::new(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8e030f: scratch := "Transfer Bid for <%s - Player Name(e.g.Kevin James)>"
        // item @ 0x8e0359 — FUN_00549580
        pool.spawn_widget(WidgetDescriptor {
            kind: 1 as u32,
            grid_x0: 100, grid_y0: 80,
            grid_x1: 790, grid_y1: 125,
            seq: 0, row_index: 0,
            style_byte: 2 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (6i32) as u16,
            pattern: (0i32) as u16,
            text: "Transfer Bid for <%s - Player Name(e.g.Kevin James)>".into(),
            slot_40: 0,
            msg_id: 0,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8e039b: scratch := "Accept a one week delay in the <%s - Player Name(e.g.Kevin James)>{} transfer from {}<%s - Club "
        // sidebar @ 0x8e03d7 — FUN_00745540(mode=4)
        spawn_sidebar_placeholder(pool, 4)?;
        // area @ 0x8e0414 — FUN_00549790
        let _area_1 = pool.spawn_area(
            100 as i16, 555 as i16, 790 as i16, 590 as i16,
            3 as u8, Vec::new(),                     // ncols + col_weights placeholder
            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg
            -1,                                              // parent
        )? as i16;
        // strcpy_scratch @ 0x8e0425: scratch := "Back"
        // item @ 0x8e047a — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 0, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Back".into(),
            slot_40: 0,
            msg_id: 1,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8e0489: scratch := "Reject"
        // item @ 0x8e04de — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 1, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Reject".into(),
            slot_40: 0,
            msg_id: 2,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // strcpy_scratch @ 0x8e04ed: scratch := "Accept"
        // item @ 0x8e0542 — FUN_00549580
        // TODO(unresolved-arg pos=17 area_handle): reg eax unread
        pool.spawn_widget(WidgetDescriptor {
            kind: 2 as u32,
            grid_x0: 0, grid_y0: 0,
            grid_x1: 0, grid_y1: 0,
            seq: 2, row_index: 0,
            style_byte: 48 as u32,
            colour_a: (0i32) as u16,
            colour_b: (0i32) as u16,
            text_style: 12 as u32,
            label_ink: (3i32) as u16,
            pattern: (0i32) as u16,
            text: "Accept".into(),
            slot_40: 0,
            msg_id: 3,
            userdata_id: (0i64) as u32,
        }, -1i16)?;
        // ret @ 0x8e0551
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


/// Auto-generated from `008e9e60.json` by
/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —
/// regenerate to pick up JSON schema fixes.
///
/// Source: FUN_8e9e60 / callback_va 0x8e9e60 — 0 widgets.
// GDI-REG: 008e9e60 PORTED_PARTIAL
pub fn build_screen_8e9e60(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {
    let a0 = pool.areas.len();
    let w0 = pool.widgets.len();

    // Root area — every screen implicitly has one canvas the
    // substrate widgets attach to (the exe reuses DAT_00B59FC0).
    let _root_area = pool.spawn_area(
        0, 0, 799, 599, 0, Vec::new(),
        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,
    )? as i16;

    {
        // ret @ 0x8e9f3e
    }

    Some((pool.areas.len() - a0, pool.widgets.len() - w0))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_build_screen_701070() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_701070(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_8596b0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_8596b0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_698160() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_698160(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_7719b0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_7719b0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_4150e0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_4150e0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_494640() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_494640(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_46bdf0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_46bdf0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_476ef0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_476ef0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_7013d0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_7013d0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_58a740() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_58a740(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_58d000() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_58d000(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_810ce0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_810ce0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_417870() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_417870(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_474760() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_474760(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_4751b0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_4751b0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_4e2c70() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_4e2c70(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_4e38d0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_4e38d0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_4e42b0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_4e42b0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_4e6680() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_4e6680(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_4ebfa0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_4ebfa0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_4ec590() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_4ec590(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_4fd1f0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_4fd1f0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_548560() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_548560(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_5488f0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_5488f0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_5dad10() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_5dad10(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_5db600() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_5db600(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_697440() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_697440(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_697dc0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_697dc0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_6fd7b0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_6fd7b0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_7cdc70() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_7cdc70(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_7fb050() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_7fb050(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_804020() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_804020(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_808ae0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_808ae0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_88def0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_88def0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_890e50() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_890e50(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_893500() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_893500(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_8a2180() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_8a2180(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_8a6580() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_8a6580(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_8e01d0() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_8e01d0(&mut pool).unwrap();
        assert!(a >= 1);
    }

    #[test]
    fn smoke_build_screen_8e9e60() {
        let mut pool = GuiRecordPool::new();
        let (a, _w) = build_screen_8e9e60(&mut pool).unwrap();
        assert!(a >= 1);
    }

}
