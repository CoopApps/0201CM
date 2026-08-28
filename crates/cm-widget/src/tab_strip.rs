//! Faithful port of the exe's shared tab-strip builder `FUN_005d7070`,
//! extracted from cm-ui-app/src/screens.rs so both the app and the
//! generated-screen renderer draw tabs through the SAME ported code.

use cm_render::font::Fonts;
use cm_render::layout::rebuild_layout;
use cm_render::panel::{F_BEVEL, F_SOLID_FILL};
use cm_render::Surface;

use crate::Palette;

/// A single tab record — mirrors FUN_005d7070's per-entry 0xd4-byte struct.
///
/// Field mapping to the exe record:
///   +0x00 short: `key` — widget key; if -1 the exe substitutes 1
///   +0x02 short: `text_color_override` — 0 means "use cached default fg"
///   +0x04 .. +0xcc: label text
///   +0x33 int:  `group` — group ptr (0 = ungrouped); tabs in the SAME group as
///                the selected tab render with same-group bg; other groups
///                render with cross-group bg and get widget flag 0x4002 (nav).
///   +0x34 int:  `id` — matched against `selected_id`
#[derive(Debug, Clone)]
pub struct TabRecord<'a> {
    pub key: i16,
    pub text_color_override: u16,
    pub group: i32,
    pub id: i32,
    pub label: &'a str,
}

impl<'a> TabRecord<'a> {
    /// Ungrouped, no color override, key auto-1. For simple contiguous strips.
    pub fn simple(id: i32, label: &'a str) -> Self {
        Self { key: 1, text_color_override: 0, group: 0, id, label }
    }
}

/// Faithful port of `FUN_005d7070` — the exe's shared tab-strip builder used by
/// News, history, and multiple club/comp screens.
///
/// Algorithm (matches the decompile line for line):
///
/// 1. **Two-pass reorder**: pass 1 finds the SELECTED tab's group ptr
///    (`local_418`); pass 2 stably packs tabs so all same-group entries come
///    first, in original order. The reorder is O(n²) in the exe (a nested loop
///    at 005d7300); we produce the same order with `sort_by_key`.
/// 2. **Split test**: if `n > 5`, split the strip into `n_left` (first half)
///    and `n - n_left` (second half). `param_6 == 0` → `n/2`, else `(n+1)/2`.
///    After each half, the exe advances `*left_x += 45` and `*right_x -= 45`
///    to leave a gutter. For `n ≤ 5` everything is a single half.
/// 3. **Per-half background panel**: `FUN_00549790` emits a thin 1px strip
///    (bevel guide) and a full-height panel that becomes the parent (`piVar2`)
///    of the tabs in that half.
/// 4. **Per-tab item widget**: `FUN_00549580` with subtype 2 (or 0x4002 if the
///    tab's group ≠ 0 AND ≠ selected group — this is the "cross-group nav"
///    flag). Flags = `0x30 | 0x800` for the selected tab, else `0x30`.
///    Background color:
///      - **DAT_00ad6bdc** if this is the selected tab (active)
///      - **DAT_00acdf74** if same-group as selected tab
///      - **DAT_00ad6bd8** if different group
///    Text color: `DAT_00af6c26` (cached `pack_rgb565(0x20,0,0x60,0)` at first
///    entry), overridden per-tab by `text_color_override` when non-zero.
///
/// The three BG colors map to our palette as:
///   `DAT_00ad6bdc` -> `pal.grey`         (active)
///   `DAT_00acdf74` -> `pal.title_fg`     (same-group)   — near-white
///   `DAT_00ad6bd8` -> `pal.btn_blue`     (other-group)  — navy
///
/// The cached fg `DAT_00af6c26` (packed rgb565 of 0x20/0/0x60/0) resolves to
/// a dark ink — we use `pal.dark_ink` so text on the light `title_fg`
/// inactive tabs stays legible. `text_color_override` shadows it.
///
/// `left_x` / `right_x` are `&mut` because the exe advances them (see step 2);
/// callers that draw a single strip pass distinct integers.
pub fn draw_tab_strip(
    s: &mut Surface,
    fonts: &mut Fonts,
    strip_rect: (i32, i32, i32, i32),
    tabs: &[TabRecord],
    selected_id: i32,
    left_x: &mut i32,
    right_x: &mut i32,
    split_up: bool,
    pal: &Palette,
) {
    if tabs.is_empty() || tabs.len() >= 0xd {
        return;
    }
    let (_l0, top, _r0, bot) = strip_rect;

    // ── Pass 1: find selected tab's group (local_418 in the exe). ──────────
    let selected_group: Option<i32> = tabs
        .iter()
        .find(|t| t.id == selected_id)
        .map(|t| t.group);

    // ── Pass 2: stable reorder — same-group first, then everything else. ──
    // Exe's nested copy at 005d7300 is a stable partition on group == selected.
    let mut order: Vec<usize> = (0..tabs.len()).collect();
    if let Some(sg) = selected_group {
        order.sort_by_key(|&i| if tabs[i].group == sg { 0 } else { 1 });
    }

    // ── Step 2: split. ─────────────────────────────────────────────────────
    let n = tabs.len();
    let n_left = if n > 5 {
        if !split_up { n / 2 } else { (n + 1) / 2 }
    } else {
        n
    };
    let n_right = n - n_left;

    // ── Step 3: background panels + per-tab layout. ────────────────────────
    // The exe emits one background panel per half; each half's tabs are laid
    // out inside that panel by the widget's own N-column auto-layout. We
    // replicate with `rebuild_layout`, one call per half.
    let left_rect  = (*left_x,               top, {
        // The exe leaves a 45px gap after the left half (advance +0x2d).
        // We compute the boundary as (right_x - n_right advance) so the two
        // halves stay symmetric around the strip midpoint.
        if n_right == 0 { *right_x } else { (*left_x + *right_x) / 2 - 22 }
    }, bot);
    let right_rect = ({
        if n_right == 0 { *left_x } else { (*left_x + *right_x) / 2 + 23 }
    }, top, *right_x, bot);

    // Draw a bevelled containing panel per non-empty half (parent of its tabs).
    if n_left > 0 {
        s.draw_panel(left_rect.0, left_rect.1, left_rect.2, left_rect.3,
                     F_SOLID_FILL | F_BEVEL, pal.title_bg);
    }
    if n_right > 0 {
        s.draw_panel(right_rect.0, right_rect.1, right_rect.2, right_rect.3,
                     F_SOLID_FILL | F_BEVEL, pal.title_bg);
    }

    let left_layout  = if n_left > 0 {
        Some(rebuild_layout(left_rect, 2, &vec![1; n_left], &[1], false))
    } else { None };
    let right_layout = if n_right > 0 {
        Some(rebuild_layout(right_rect, 2, &vec![1; n_right], &[1], false))
    } else { None };

    // Text font — FUN_005d7070 passes font slot 0xc (narrow, small; slot 1 in
    // our fonts registry, arial_narrow_10).
    // A strip where every tab is genuinely ungrouped (group==0 per the
    // struct doc comment: "group ptr, 0 = ungrouped") -- News's top/bottom
    // rows are built via TabRecord::simple, which always sets group: 0.
    // Real full-frame capture (this session's frida_capture_full_frame.py
    // scan, and independently the capture log's per-tab rflags: seq=9..12
    // all read the same fill regardless of selection) shows all 4 top-tab
    // cells render ONE flat fill, RGB(33,0,99) -- selection is shown only
    // by the gold border below, not a different background.
    let flat_ungrouped_strip = tabs.iter().all(|t| t.group == 0);

    for (slot, &tab_ix) in order.iter().enumerate() {
        let t = &tabs[tab_ix];
        let is_sel = t.id == selected_id;
        // group==0 is the documented "ungrouped" sentinel, not a real group
        // id -- two ungrouped tabs are NOT "the same group" as each other.
        // The unconditional `t.group == sg` below used to treat every
        // group-0 tab as matching every other group-0 tab, which is why an
        // ungrouped strip like News's got the same_group (near-white)
        // branch on every non-selected tab instead of its real flat fill.
        let same_group = selected_group.map_or(false, |sg| sg != 0 && t.group == sg);

        let (l, top_y, r, b) = if slot < n_left {
            left_layout.as_ref().unwrap().cell(slot, 0)
        } else {
            right_layout.as_ref().unwrap().cell(slot - n_left, 0)
        };

        let bg = if flat_ungrouped_strip {
            (33, 0, 99) // real measured flat fill, see flat_ungrouped_strip's comment
        } else if is_sel {
            pal.grey          // DAT_00ad6bdc
        } else if same_group {
            pal.title_fg      // DAT_00acdf74
        } else {
            pal.btn_blue      // DAT_00ad6bd8
        };
        s.draw_panel(l, top_y, r, b, F_SOLID_FILL | F_BEVEL, bg);

        // Text color: override (record+2) or the cached default fg. `dark_ink`
        // approximates DAT_00af6c26 (packed rgb565 of 0x20/0/0x60/0).
        let ink = if t.text_color_override != 0 {
            unpack_rgb565(t.text_color_override)
        } else if same_group && !is_sel {
            pal.dark_ink        // dark text on near-white same-group tabs
        } else {
            pal.near_white      // white text on navy other-group / grey active
        };
        let f = fonts.slot(1);
        s.draw_text_box(l, top_y, r, b, 0, f, ink, t.label);

        // Flag 0x800 → the exe renders a highlight border on the selected item.
        if is_sel {
            s.draw_hollow_rect(l - 1, top_y - 1, r + 1, b + 1, pal.highlight_fg);
        }
        let _ = t.key; // widget key (record[0]); consumed by cm-widget when routed
    }

    // Step 2 tail: advance the caller's x pointers by 45px each side (the exe
    // does `*param_4 += 0x2d; *param_5 -= 0x2d;` after emitting each half).
    if n_left > 0  { *left_x  += 45; }
    if n_right > 0 { *right_x -= 45; }
}

/// Unpack a 16-bit RGB565 back to 8-bit-per-channel tuple (matches the exe's
/// FUN_005ce4f0 pack in reverse). Used for `text_color_override` values that
/// arrive as 16-bit packed colors.
fn unpack_rgb565(c: u16) -> (u8, u8, u8) {
    let r5 = ((c >> 11) & 0x1f) as u8;
    let g6 = ((c >> 5)  & 0x3f) as u8;
    let b5 =  (c        & 0x1f) as u8;
    ((r5 << 3) | (r5 >> 2), (g6 << 2) | (g6 >> 4), (b5 << 3) | (b5 >> 2))
}
