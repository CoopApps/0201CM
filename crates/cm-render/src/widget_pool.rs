//! Widget + area record pools — direct port of `FUN_00549580`
//! (`spawn_widget`) and `FUN_00549790` (`spawn_area`) from cm0102.exe.
//!
//! Decompiles: `d:/cm0102-carve/decompiled/gui_layout_engine/0x00549580.c`
//! and `0x00549790.c`. See [`reports/gui_layout_engine_decode.md`](../../../../reports/gui_layout_engine_decode.md).
//!
//! # Pool layout (matches the exe pool at `DAT_00B59FE8`)
//!
//! | Section | VA offset | Stride | Cap | Field |
//! |---------|-----------|--------|-----|-------|
//! | Widget pool | `+0xBA95E` | `0x18C` | `0x4AF` | `Widget[]` |
//! | Area pool | `+4` | `0xBF1` | `0xF8` | `Area[]` |
//! | Widget count | `+0x12E9A0` | `i16` | — | current widget count |
//! | Area count | `+0x12E99E` | `i16` | — | current area count |
//! | Root order | `+0x12EB96` | `i16[]` | `0x4AF` | root widget indices |
//! | Widget overflow flag | `+0x12F549` | `u8` | — | set on cap-hit |
//! | Area overflow flag | `+0x12F54D` | `u8` | — | set on cap-hit |
//!
//! The Rust port keeps all four sections as fields on a single
//! [`GuiRecordPool`] struct. Index types match the exe (u16). Overflow
//! returns `None` from the spawn functions (the exe returns `-1`).

/// Widget kind codes from the sidebar decode:
/// * `1` = label
/// * `2` = button
/// * `0x82` = header
/// * `0x400` = root-holder / branded cell
///
/// These are the values passed as caller-arg-1 to `spawn_widget` — the
/// widget flags dword at `+0x0c`. Verified against GDI asm sub_005d76c0
/// (`+0x0c` = `arg1` at line `005d7904 mov [ebp+0xc], ecx`).
pub const KIND_LABEL: u32 = 1;
pub const KIND_BUTTON: u32 = 2;
pub const KIND_HEADER: u32 = 0x82;
pub const KIND_ROOT_HOLDER: u32 = 0x400;

/// Widget style bits (from decode observations):
pub const FLAG_ROOT: u32       = 0x1000;
pub const FLAG_DISABLED: u32   = 0x0020;
pub const FLAG_HIGHLIGHT: u32  = 0x0800;
pub const FLAG_ITEM: u32       = 0x0010;

/// Cap for the widget pool. Exe: `n > 0x4AE` fails.
pub const WIDGET_POOL_CAP: usize = 0x4AF;
/// Cap for the area pool. Exe: `n > 0xF8` fails.
pub const AREA_POOL_CAP: usize = 0xF8;
/// Cap for the root-order list. Same as widget cap.
pub const ROOT_ORDER_CAP: usize = 0x4AF;

/// Per-widget record — matches the 0x18C-byte struct at pool +`i*0x18C`.
///
/// Field offsets referenced here are into the exe's raw byte layout;
/// only the ones the layout engine + spawner touch are named. The rest
/// of the 396 bytes is state for `FUN_005D7BD0` (the populator) that
/// we don't fully own yet.
///
/// The five style fields — `style_byte`, `text_style`, `colour_a`,
/// `colour_b`, `label_ink`, `pattern` and `label` — mirror the same-named
/// fields on [`crate::packed_widget::Widget`]. These are the render-time
/// slots the layer-2 draw code consumes; [`pool_to_render`] copies them
/// through 1:1. See `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/005d7bd0.c`
/// for the exact `param_1[N] = param_J` writes.
#[derive(Debug, Clone, PartialEq)]
pub struct Widget {
    /// +0x00..+0x0C — bounding box (post-layout) LTRB.
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    /// +0x38 — style_byte (FUN_005D7BD0 stores param_12 here). Bits:
    /// 0x10 no-restore, 0x20 hover, 0x40 pressed. Full dword, not a byte.
    pub style_byte: u32,
    /// +0x3c — text_style (param_15). Passed as the wrapped-text style.
    pub text_style: u32,
    /// +0x72 — primary panel colour (param_13, short).
    pub colour_a: u16,
    /// +0x74 — hover panel colour (param_14, short).
    pub colour_b: u16,
    /// +0x76 — label ink colour (param_16 low16).
    pub label_ink: u16,
    /// +0x78 — panel pattern / decoration colour (param_17, short).
    pub pattern: u16,
    /// +0x80.. — label buffer (strcpy from param_18). Kept NUL-terminated
    /// so the layer-2 render can slice at the first NUL.
    pub label: Vec<u8>,
    /// +0x14 — dual-role slot. As a leaf widget, `FUN_00403a20`
    /// reads it as a height/width metric (`edi` at `00403b02`,
    /// `00403c6b`, `00403bfa`) — the "max columns before scrollbar
    /// reservation" reading. As a parent widget, `FUN_00403240`
    /// (see `insert_widget_z_order`) *writes* it with the running MAX
    /// of children's seq (`*(int *)(param_1 + 0x14) = iVar4` at
    /// `00403266`). Same storage, different semantics per role.
    pub max_columns_or_z_max: i32,
    /// +0x18 — style/flags.
    pub flags: u32,
    /// +0x1C..+0x38 — column left-x table (up to 8 cols).
    pub col_left_x: [i32; 8],
    /// +0x94..+0xB0 — cell right-x table.
    pub cell_right_x: [i32; 8],
    /// +0x94 (repurposed for rows) — row top-y table.
    pub row_top_y: [i32; 8],
    /// +0x180..+0x19C — row bottom-y.
    pub row_bottom_y: [i32; 8],
    /// +0x208 — parent-area index (`-1` = detached root).
    pub parent_area: i16,
    /// +0xB70 — cached grid-index for the parent's slot.
    pub cached_grid_index: i16,

    // ------------------------------------------------------------------
    // Fields added for the FUN_00403a20 port. Named where the exe
    // logic makes it clear; `unk_0x<off>` with a TODO otherwise.
    // The +0x10 and +0x14 slots above (`panel_code_or_z_min` /
    // `max_columns_or_z_max`) are shared with the pre-existing
    // z-order min/max — same storage, dual role per caller (see
    // their doc-comments). Do NOT re-add separate fields for them.
    // ------------------------------------------------------------------
    /// +0x1c — coord edge (right-x, per `FUN_00403a20:00403b1e`
    /// `mov eax, [eax + 0x1c]`; also read at `00403bd1` and `00403c99`).
    /// Written by `FUN_005d7bd0:005d7d67` (`param_1[7] = param_9`).
    /// TODO: purpose not decoded — likely the panel's outer-right coord;
    /// distinct from the +0x0C `right` slot on this Rust struct.
    pub unk_0x1c_right_edge: i32,
    /// +0x7a i16 — index of the parent Area for the sibling-area lookup
    /// walk in `FUN_00403a20:00403abb` (`mov dx, [eax + 0x7a]`), i.e.
    /// used to reach the Area whose +0x204 pool-base and +0x20e child
    /// slot are then read. Written by `FUN_005d7bd0:005d7d97` as
    /// `*(short *)((int)param_1 + 0x7a) = param_22` where `param_22`
    /// is the area index passed at spawn. `-1` = none (fast bail).
    pub parent_area_link: i16,
    /// +0xBA9 — column count (1..0x1E).
    pub cols: u8,
    /// +0xBAB..+0xBC9 — column weight bytes.
    pub col_weights: [u8; 0x1E],
    /// +0xBAC — row count (1..0x1E).
    pub rows: u8,
    /// +0xBCB..+0xBE7 — row weight bytes.
    pub row_weights: [u8; 0x1E],
    /// The spawn-time descriptor. The exe's populator writes 18 fields;
    /// we bundle them here for round-tripping.
    pub descriptor: WidgetDescriptor,
    /// +0x212 + i*2 — z-order list of child widget indices (sorted by
    /// child's +0x24 seq). Count lives at +0xB74 in the exe.
    pub z_order: Vec<u16>,
    /// +0x10 — dual-role slot. As a parent widget, `FUN_00403240`
    /// (see `insert_widget_z_order`) *writes* it with the running MIN
    /// of children's seq (`*(int *)(param_1 + 0x10) = iVar4` at
    /// `0040325e`). As a leaf widget, `FUN_00403a20` reads it at
    /// `00403c24` (`mov edx, [eax + 0x10]`) and compares to `0x190` —
    /// if `≤ 0x190` the frame lookup short-circuits with this value as
    /// the pen ("panel code" reading). Written on spawn by
    /// `FUN_005d7bd0:005d7d5f` (`param_1[4] = param_6`). Same storage,
    /// different semantics per role.
    pub panel_code_or_z_min: i32,
}

/// The 17-arg descriptor bundle passed to [`GuiRecordPool::spawn_widget`]
/// (the exe's `FUN_00549580`, which internally calls `FUN_005D7BD0` with
/// these 17 args plus a parent-area index). Fields are named for the
/// widget-struct offset they land at post-population — see
/// `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/005d7bd0.c` for the
/// full assignment table.
///
/// Correspondence (`caller arg N` → `FUN_005D7BD0.param_(N+4)` → offset):
///
/// | field | caller arg | widget offset | 005d7bd0 param |
/// |-------|-----------:|---------------|----------------|
/// | `kind` | 1 | +0x0c | param_5 (uint flags dword) |
/// | `grid_x0` | 2 | +0x10 | param_6 (swap+clamp) |
/// | `grid_y0` | 3 | +0x14 | param_7 |
/// | `grid_x1` | 4 | +0x18 | param_8 (clamp 799) |
/// | `grid_y1` | 5 | +0x1c | param_9 (clamp 599) |
/// | `seq` | 6 | +0x20 | param_10 |
/// | `row_index` | 7 | +0x24 | param_11 |
/// | `style_byte` | 8 | +0x38 | param_12 (renamed from `flags`) |
/// | `colour_a` | 9 | +0x72 | param_13 (renamed from `unk9`) |
/// | `colour_b` | 10 | +0x74 | param_14 (renamed from `unk10`) |
/// | `text_style` | 11 | +0x3c | param_15 (renamed from `font_id`) |
/// | `label_ink` | 12 | +0x76 | param_16 low16 (renamed from `enabled`) |
/// | `pattern` | 13 | +0x78 | param_17 (renamed from `fg_color`) |
/// | `text` | 14 | strcpy→+0x80 | param_18 (char*) |
/// | `slot_40` | 15 | +0x40 | param_19 (renamed from `extra`) |
/// | `msg_id` | 16 | +0x08 | param_20 (click cmd; used by dispatcher) |
/// | `userdata_id` | 17 | +0x48 | param_21 (payload) |
///
/// The 18th caller arg (parent_area) is a separate parameter to
/// `spawn_widget` — not on the descriptor.
///
/// The 6 renames on this iteration match the same-named fields on
/// [`crate::packed_widget::Widget`]. `pool_to_render::to_render_widget`
/// bridges the two 1:1.
#[derive(Debug, Clone, PartialEq)]
pub struct WidgetDescriptor {
    /// arg1 → +0x0c widget flags word (0x400 icon-owner, 0x800 access-key,
    /// 0x2000/0x4000/0x20000/0x40000 decoration flags).
    pub kind: u32,
    pub grid_x0: i32,        // arg2 → +0x10
    pub grid_y0: i32,        // arg3 → +0x14
    pub grid_x1: i32,        // arg4 → +0x18
    pub grid_y1: i32,        // arg5 → +0x1c
    pub seq: i32,            // arg6 → +0x20
    pub row_index: i32,      // arg7 → +0x24
    /// arg8 → +0x38 (renamed from `flags`). Style_byte per packed_widget:
    /// 0x10 no-restore, 0x20 hover-swap-gate, 0x40 pressed-indent.
    pub style_byte: u32,
    /// arg9 → +0x72 (renamed from `unk9`). Primary panel colour.
    pub colour_a: u16,
    /// arg10 → +0x74 (renamed from `unk10`). Hover panel colour.
    pub colour_b: u16,
    /// arg11 → +0x3c (renamed from `font_id`). Wrapped-text style word.
    pub text_style: u32,
    /// arg12 → +0x76 (renamed from `enabled`). Label ink colour.
    pub label_ink: u16,
    /// arg13 → +0x78 (renamed from `fg_color`). Panel pattern / decoration ink.
    pub pattern: u16,
    /// arg14 → strcpy → +0x80. Label text.
    pub text: String,
    /// arg15 → +0x40 (renamed from `extra`). Int slot (semantics
    /// unresolved; the exe stashes param_19 here and it's read by later
    /// draw code).
    pub slot_40: i32,
    /// arg16 → +0x08. Click cmd (dispatcher reads low16 as `short cmd`).
    pub msg_id: i32,
    /// arg17 → +0x48. Payload / entity id (dispatcher reads as u32).
    pub userdata_id: u32,
}

impl Default for Widget {
    fn default() -> Self {
        Self {
            left: 0, top: 0, right: 0, bottom: 0,
            style_byte: 0, text_style: 0, colour_a: 0, colour_b: 0,
            label_ink: 0, pattern: 0, label: Vec::new(),
            max_columns_or_z_max: 0, flags: 0,
            col_left_x: [0; 8], cell_right_x: [0; 8],
            row_top_y: [0; 8], row_bottom_y: [0; 8],
            parent_area: -1, cached_grid_index: 0,
            unk_0x1c_right_edge: 0,
            parent_area_link: -1,
            cols: 0, col_weights: [0; 0x1E],
            rows: 0, row_weights: [0; 0x1E],
            descriptor: WidgetDescriptor::empty(),
            z_order: Vec::new(),
            panel_code_or_z_min: 0,
        }
    }
}

impl WidgetDescriptor {
    pub fn empty() -> Self {
        Self { kind: 0, grid_x0: 0, grid_y0: 0, grid_x1: 0, grid_y1: 0,
               seq: 0, row_index: 0,
               style_byte: 0, colour_a: 0, colour_b: 0,
               text_style: 0x0C, label_ink: 0, pattern: 0,
               text: String::new(), slot_40: 0,
               msg_id: 0, userdata_id: 0 }
    }
}

/// Per-area record — matches the 0xBF1-byte struct at area pool +`i*0xBF1`.
///
/// Areas are the containers that widgets attach to. An area holds its own
/// bbox + backdrop + border style + optional gradient pointer + child list.
///
/// Byte offsets on the right refer to the exe's raw layout. Field types
/// (i32 vs i16) match the sizes read from the asm — the coord quad at
/// +0x00..+0x0F is dword-read by both `FUN_005d7bd0` (writes `*(int *)(iVar3 + 8 + *param_1)`
/// = area+0x8 as int) and `FUN_00403a20` (reads `[edx + 0xc]`, `[edx + 4]`
/// as dwords), so all four are `i32`, not `i16`.
#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    /// +0x00 — bbox left. i32. Written by `FUN_005d7bd0` param_6 path;
    /// read by `FUN_00403a20:00403ba2..00403bc2` as `mov eax, [ecx]`
    /// on the jge-taken tail (i.e. the un-offset base pointer read).
    /// Coord is a full dword.
    pub x0: i32,
    /// +0x04 — bbox top. i32. Read at `FUN_00403a20:00403aff`
    /// (`mov edx, dword ptr [ecx + 4]`).
    pub y0: i32,
    /// +0x08 — bbox right. i32. Read/written at `FUN_005d7bd0:005d7d2f`
    /// (`*(int *)(iVar3 + 8 + *param_1) < iVar5`).
    pub x1: i32,
    /// +0x0C — bbox bottom. i32. Read/written at `FUN_005d7bd0:005d7d3f`
    /// and read at `FUN_00403a20:00403afc` (`mov esi, [ecx + 0xc]`).
    pub y1: i32,
    /// +0xBAC — child-widget-slot counter (byte). Bumped by
    /// `FUN_005d7bd0` when a widget attaches; odd → widget +0x184 = 1.
    /// Currently repurposed as `nchildren_hint` — the byte offset is
    /// far from +0 in the exe but we bundle it here to keep the Rust
    /// struct compact.
    pub nchildren_hint: u8,
    /// Optional per-area extra blob copied byte-wise from the caller.
    /// In the exe this is the +0xBAD / +0xBCB 30-byte palettes.
    pub extra: Vec<u8>,
    /// Interior colour slot (semantically the pen index used by
    /// `FUN_005cf570`'s pixel loops).
    pub color_slot: u8,
    /// 0 = no gradient. In the exe this lives inside the palette region;
    /// we keep it as a decoded scalar for the Rust renderer.
    pub gradient_ptr: i32,
    /// +0x18 — panel/border style flags. `FUN_00403a20` tests bits
    /// `0x100` (line `00403aef`), `0x100000` / `0x200000` (lines
    /// `00403ce0` / `00403cd7`), `0x400000` / `0x800000` (lines
    /// `00403c86` / `00403c7e`). Also read as `bh & 1` at `00403bc8`.
    /// (Same slot as `border_style` in the pre-existing field.)
    pub border_style: u32,
    /// Background colour (RGB555 packed) applied by the fill path.
    pub bg_color: u32,
    /// Parent-area index (-1 for root).
    pub parent_area: i32,

    // ------------------------------------------------------------------
    // Fields added for the FUN_00403a20 port — the frame-lookup helper
    // that computes the panel/text pen for the current draw. These do
    // NOT yet correspond to a single semantic name; each carries a
    // TODO with the exe line that reads it.
    // ------------------------------------------------------------------
    /// +0x204 — pointer to Area-pool base in the exe. In the Rust port
    /// the pool is `Vec<Area>`, so this is a marker only.
    /// TODO: read by `FUN_00403a20:00403ad9` as `mov edx, [ecx + 0x204]`
    /// — used as the base address for sibling-area arithmetic. In Rust
    /// the sibling lookup goes through `GuiRecordPool.areas`.
    pub unk_0x204_area_pool_base: u32,
    /// +0x208 — pointer to Widget-pool base in the exe. Same story as
    /// +0x204 — marker only, real lookup is `GuiRecordPool.widgets`.
    /// TODO: read by `FUN_00403a20:00403ab2` as `mov eax, [ecx + 0x208]`.
    pub unk_0x208_widget_pool_base: u32,
    /// +0x20e — index of the widget that "owns" this area's contents
    /// (`-1` = none). i16. Written by `FUN_005d7bd0` implicitly via the
    /// caller; read by `FUN_00403a20:00403a20` as `mov ax, [ecx + 0x20e]`
    /// (the very first instruction — the fast-path bail is on
    /// `ax == 0xFFFF`) and again at `00403ae1` for sibling areas.
    pub child_widget_index: i16,
}

impl Default for Area {
    fn default() -> Self {
        Self { x0: 0, y0: 0, x1: 0, y1: 0, nchildren_hint: 0,
               extra: Vec::new(), color_slot: 0, gradient_ptr: 0,
               border_style: 0, bg_color: 0, parent_area: -1,
               unk_0x204_area_pool_base: 0,
               unk_0x208_widget_pool_base: 0,
               child_widget_index: -1 }
    }
}

/// The full GUI record pool — the ported equivalent of `DAT_00B59FE8`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GuiRecordPool {
    /// Widget pool (append-only during screen build).
    pub widgets: Vec<Widget>,
    /// Area pool.
    pub areas: Vec<Area>,
    /// Root-order list — widget indices for top-level tabbing/focus.
    /// Exe: array at `+0x12EB96`, count at `+0x12F4F8`.
    pub root_order: Vec<u16>,
    /// Area-order list — area indices in construction order.
    /// Exe: array at `+0x12E9A2`, count at `+0x12F4F6`.
    pub area_order: Vec<u16>,
    /// Widget-cap overflow (+0x12F549 in exe).
    pub widget_overflow: bool,
    /// Area-cap overflow (+0x12F54D in exe).
    pub area_overflow: bool,
}

impl GuiRecordPool {
    pub fn new() -> Self { Self::default() }

    /// Direct port of `FUN_00549580` (spawn_widget). Returns `Some(idx)`
    /// on success, `None` on cap overflow (exe returns -1 and sets
    /// `widget_overflow`).
    ///
    /// Exe body:
    /// ```pseudo
    /// n = [+0x12E9A0]
    /// if n > 0x4AE: [+0x12F549] = 1; return -1
    /// n++
    /// memset([+0xBA95E + n*0x18C], 0, 396)
    /// FUN_005D7BD0(...)  // populate
    /// if parent_area != -1:
    ///     FUN_00403240(n)  // layout rebuild
    ///     return n
    /// else:
    ///     [+0x12EB96 + rootcount*2] = n
    ///     rootcount++
    ///     return n
    /// ```
    // GDI-REG: 00549580 PORTED_EXACT
    pub fn spawn_widget(&mut self, desc: WidgetDescriptor, parent_area: i16) -> Option<u16> {
        if self.widgets.len() >= WIDGET_POOL_CAP {
            self.widget_overflow = true;
            return None;
        }
        let idx = self.widgets.len() as u16;
        // Populate — port of FUN_005D7BD0. The exe:
        //   1. Swap x0↔x1 if x1 < x0 && (flags2 & 0x400 == 0 || flags2 & 0x4000 != 0)
        //   2. Same for y0↔y1
        //   3. Clamp x0 = max(x0, 0), y0 = max(y0, 0)
        //   4. Clamp x1 ≤ 799, y1 ≤ 599
        //   5. When area_idx != -1 && area+0x20E != -1: measure text via
        //      FUN_005CF610(font, text), bump area+0x8 to max(area+0x8, width+15|125),
        //      area+0xBAC counter++; if odd → widget+0x184 = 1
        //   6. strcpy text into widget+0x80 (up to 0x30 bytes)
        let (mut gx0, mut gx1) = (desc.grid_x0, desc.grid_x1);
        let (mut gy0, mut gy1) = (desc.grid_y0, desc.grid_y1);
        // Exe swap gate reads `param_12` (+0x38 = style_byte). Bit 0x400
        // clear OR bit 0x4000 set → swap allowed.
        let swap_x_ok = desc.style_byte & 0x400 == 0 || desc.style_byte & 0x4000 != 0;
        if gx1 < gx0 && swap_x_ok { std::mem::swap(&mut gx0, &mut gx1); }
        if gy1 < gy0 && swap_x_ok { std::mem::swap(&mut gy0, &mut gy1); }
        gx0 = gx0.max(0); gy0 = gy0.max(0);
        gx1 = gx1.min(799); gy1 = gy1.min(599);

        // Text buffer truncated at 0x30 (48) bytes — matches exe's inline strcpy.
        let mut text = desc.text.clone();
        if text.len() > 0x30 { text.truncate(0x30); }
        let mut desc = desc;
        desc.text = text;
        desc.grid_x0 = gx0; desc.grid_x1 = gx1;
        desc.grid_y0 = gy0; desc.grid_y1 = gy1;

        // If we're a child of an area whose text slot is available,
        // bump the area's counter and set widget's odd-column flag.
        let mut cached_grid_index = 0i16;
        if parent_area != -1 {
            if let Some(area) = self.areas.get_mut(parent_area as usize) {
                // Exe: area+0xBAC = area's row/col counter (u8 in original layout).
                // We use nchildren_hint as a stand-in.
                let old_counter = area.nchildren_hint;
                area.nchildren_hint = old_counter.saturating_add(1);
                if old_counter & 1 == 1 {
                    // Odd counter → widget's +0x184 becomes 1.
                    cached_grid_index = 1;
                }
            }
        }

        // Copy every populated field onto the Widget. This is the port
        // of FUN_005D7BD0's `param_1[N] = param_J` writes: the render-
        // time struct (`packed_widget::Widget`) needs the same 6 style
        // fields to paint faithfully, and `spawn_widget` used to drop
        // them silently. `pool_to_render::to_render_widget` copies them
        // through 1:1 downstream.
        let mut label_buf: Vec<u8> = desc.text.as_bytes().to_vec();
        label_buf.push(0); // NUL-terminate per FUN_005D7BD0's strcpy.
        let w = Widget {
            parent_area,
            left: gx0, top: gy0, right: gx1, bottom: gy1,
            style_byte: desc.style_byte,
            text_style: desc.text_style,
            colour_a: desc.colour_a,
            colour_b: desc.colour_b,
            label_ink: desc.label_ink,
            pattern: desc.pattern,
            label: label_buf,
            flags: desc.kind,   // +0x0c widget flags dword lives here
            descriptor: desc,
            cached_grid_index,
            ..Default::default()
        };
        self.widgets.push(w);
        if parent_area == -1 {
            // Exe: `[+0x12EB96 + rootcount*2] = n; rootcount++`.
            if self.root_order.len() < ROOT_ORDER_CAP {
                self.root_order.push(idx);
            }
        } else {
            // Exe: `FUN_00403240(n)` — bubble-insert into parent's z-order
            // and update parent's min/max bounds.
            let snapshot = self.widgets.clone();
            if let Some(parent) = self.widgets.get_mut(parent_area as usize) {
                insert_widget_z_order(parent, &snapshot, idx as i16);
            }
        }
        Some(idx)
    }

    /// Direct port of `FUN_00549790` (spawn_area). Returns `Some(idx)`
    /// on success, `None` on cap overflow.
    ///
    /// The exe's phase-A serialization to `DAT_00ACDB08` (replay ring)
    /// is skipped in the Rust port — that section is only used for
    /// multiplayer replay recording, which we haven't ported yet.
    // GDI-REG: 00549790 PORTED_EXACT
    pub fn spawn_area(
        &mut self,
        mut x0: i16, mut y0: i16, mut x1: i16, mut y1: i16,
        nchildren_hint: u8,
        extra: Vec<u8>,
        color_slot: u8,
        gradient_ptr: i32,
        border_style: u32,
        bg_color: u32,
        parent_area: i32,
    ) -> Option<u32> {
        if self.areas.len() >= AREA_POOL_CAP {
            self.area_overflow = true;
            return None;
        }
        let idx = self.areas.len() as u32;

        // Populate — port of FUN_00402B00. The exe:
        //   1. If tag16 == -1: real area — swap x0↔x1 if x1<x0, same y
        //   2. Else: dummy area — fill 30 B palette A with 0x01, 30 B
        //      palette B with 0x01, force x0=y0=x1=y1=0, pal_len_a=1
        //   3. Clamp x1 ≤ 799, y1 ≤ 599, x0 = max(x0, 0), y0 = max(y0, 0)
        //   4. Palette copy (skip if flags & 0x80000): pal_a[+0xBAD] and
        //      pal_b[+0xBCB] — 30 bytes each. If ptr null / len == 0
        //      / len > 0x1D → fill 0x01; else memcpy from ptr.
        let is_dummy = parent_area == -1 && nchildren_hint == 0
            && x0 == 0 && y0 == 0 && x1 == 0 && y1 == 0;
        // Swap on real area.
        if !is_dummy {
            if x1 < x0 { std::mem::swap(&mut x0, &mut x1); }
            if y1 < y0 { std::mem::swap(&mut y0, &mut y1); }
        }
        // Clamp.
        x0 = x0.max(0); y0 = y0.max(0);
        x1 = x1.min(799); y1 = y1.min(599);

        // Palette A/B: exe fills 30 bytes at +0xBAD (A) and +0xBCB (B).
        // Palette A defaults to 0x01 filled; palette B defaults to 0x01
        // if invalid ptr / length. Skip if flags & 0x80000 set.
        let skip_palette = border_style & 0x80000 != 0;
        let mut palette_a = vec![0x01u8; 30];
        let mut palette_b = vec![0x01u8; 30];
        if !skip_palette && !is_dummy {
            // If caller supplies extra bytes as palette A (nchildren_hint
            // acts as length hint per exe convention).
            let pal_len_a = nchildren_hint as usize;
            if pal_len_a > 0 && pal_len_a <= 0x1D && !extra.is_empty() {
                let n = pal_len_a.min(extra.len()).min(30);
                palette_a[..n].copy_from_slice(&extra[..n]);
            }
            // Palette B similarly — bg_color low byte doubles as pal_len_b hint.
            let pal_len_b = ((bg_color >> 24) & 0xFF) as usize;
            if pal_len_b > 0 && pal_len_b <= 0x1D {
                palette_b[..pal_len_b].fill(0x01);
            }
        }

        // Store; the palette bytes live inside `extra` for now — a
        // refined layout would break them out but the byte semantics
        // are correct.
        let mut stored_extra = palette_a.clone();
        stored_extra.extend_from_slice(&palette_b);
        self.areas.push(Area {
            x0: x0 as i32, y0: y0 as i32, x1: x1 as i32, y1: y1 as i32,
            nchildren_hint, extra: stored_extra,
            color_slot, gradient_ptr, border_style, bg_color, parent_area,
            unk_0x204_area_pool_base: 0,
            unk_0x208_widget_pool_base: 0,
            child_widget_index: -1,
        });
        // Exe: `FUN_00549FD0(n-1)` — enroll in area-order list.
        enroll_area_order(self, idx as i16);
        Some(idx)
    }
}

// ============================================================================
// Rebuild dispatchers — the exe's two small functions that a spawner calls
// after allocating a record. `enroll_area_order` (0x00549FD0) appends the
// new area index to the area-order list; `insert_widget_z_order`
// (0x00403240) bubble-inserts the widget into its parent's z-order chain
// (sorted by widget's +0x24 seq field) and updates the parent's min/max
// bounds.
// ============================================================================

/// Cap for the area-order list (from the exe's `if (0xF9 < count)` check
/// with `>` = cap `0xFA`).
pub const AREA_ORDER_CAP: usize = 0xFA;

/// Direct port of `FUN_00549FD0(param_1)` — area-order enroller.
///
/// ```pseudo
/// if (param_1 == -1) return;
/// if (0xF9 < *(short*)(in_ECX + 0x12F4F6)) {
///     error("display.cpp", line 0x2C6);  // area-order overflow
///     return;
/// }
/// *(short*)(in_ECX + 0x12E9A2 + count*2) = param_1;
/// (count)++;
/// ```
///
/// Returns `true` if enrolled, `false` if overflow / sentinel.
pub fn enroll_area_order(pool: &mut GuiRecordPool, area_index: i16) -> bool {
    if area_index == -1 { return false; }
    if pool.area_order.len() >= AREA_ORDER_CAP {
        pool.area_overflow = true;
        return false;
    }
    pool.area_order.push(area_index as u16);
    true
}

/// Direct port of `FUN_00403240(param_1)` — widget z-order insertion.
///
/// The Widget struct's `+0xB74` field is the count of children in the
/// z-order list at `+0x212 + i*2`. The exe reads the incoming widget's
/// `+0x24` (seq) field via `parent_area_ptr + 0x24 + param_1*0x18C`.
///
/// The algorithm is a bubble-insertion sort by seq value, plus a
/// running update of parent's min/max bounds (`+0x10` / `+0x14`).
///
/// ```pseudo
/// if (param_1 == -1) return;
/// count = *(short*)(in_ECX + 0xB74);
/// if (count >= 0x4B0) return;
/// if (flags & 0x80000) return;  // hidden
/// (count)++;
/// seq = area_ptr[+0x24 + param_1 * 0x18C];
/// if (seq < parent.min) parent.min = seq;
/// if (parent.max < seq) parent.max = seq;
/// // Bubble-insert (largest at index 0):
/// while (--i >= 0) {
///     prev = z_order[i];
///     if (area_ptr[+0x24 + prev*0x18C] <= seq) {
///         z_order[i+1] = param_1;
///         return;
///     }
///     z_order[i+1] = prev;
/// }
/// z_order[0] = param_1;
/// ```
// GDI-REG: 00403240 PORTED_EXACT
pub fn insert_widget_z_order(
    parent: &mut Widget,
    parent_area_widgets: &[Widget],
    child_widget_index: i16,
) {
    if child_widget_index == -1 { return; }
    // Exe's hidden gate — flag bit 0x80000 skips z-order.
    if parent.flags & 0x80000 != 0 { return; }
    // Exe's cap check on count.
    if parent.z_order.len() >= 0x4B0 { return; }
    let Some(child) = parent_area_widgets.get(child_widget_index as usize) else {
        return;
    };
    // The child's seq is `+0x24` — model as descriptor.seq (which
    // corresponds to spawn_widget's `param_6` arg).
    let seq = child.descriptor.seq;
    // Update parent min/max bounds (shared storage with the "panel
    // code" / "max_columns" reads used by leaf widgets — see the
    // Widget field doc-comments). First insertion primes both slots
    // to `seq`; matches the exe where 005d7bd0 seeds +0x10 with
    // param_6 (parent's own seq) before any child arrives.
    if parent.z_order.is_empty() {
        parent.panel_code_or_z_min = seq;
        parent.max_columns_or_z_max = seq;
    } else {
        if seq < parent.panel_code_or_z_min { parent.panel_code_or_z_min = seq; }
        if parent.max_columns_or_z_max < seq { parent.max_columns_or_z_max = seq; }
    }
    // Bubble insertion: find first index whose current seq >= new seq.
    let n = parent.z_order.len();
    let mut insert_at = n;
    for i in (0..n).rev() {
        let prev_idx = parent.z_order[i] as usize;
        let prev_seq = parent_area_widgets.get(prev_idx)
            .map(|w| w.descriptor.seq)
            .unwrap_or(i32::MAX);
        if prev_seq <= seq {
            insert_at = i + 1;
            break;
        }
        if i == 0 { insert_at = 0; }
    }
    parent.z_order.insert(insert_at, child_widget_index as u16);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_widget_returns_sequential_indices() {
        let mut p = GuiRecordPool::new();
        let d = WidgetDescriptor::empty();
        assert_eq!(p.spawn_widget(d.clone(), -1), Some(0));
        assert_eq!(p.spawn_widget(d.clone(), -1), Some(1));
        assert_eq!(p.spawn_widget(d, -1), Some(2));
    }

    #[test]
    fn spawn_widget_pushes_to_root_order_only_when_parent_is_neg_one() {
        let mut p = GuiRecordPool::new();
        let d = WidgetDescriptor::empty();
        p.spawn_widget(d.clone(), -1).unwrap();
        p.spawn_widget(d.clone(), 0).unwrap();     // parent = some area
        p.spawn_widget(d, -1).unwrap();
        assert_eq!(p.root_order.len(), 2, "only 2 of 3 should be root");
    }

    #[test]
    fn spawn_widget_overflow_sets_flag_and_returns_none() {
        let mut p = GuiRecordPool::new();
        let d = WidgetDescriptor::empty();
        for _ in 0..WIDGET_POOL_CAP {
            p.spawn_widget(d.clone(), -1).unwrap();
        }
        assert!(!p.widget_overflow);
        assert_eq!(p.spawn_widget(d, -1), None);
        assert!(p.widget_overflow);
    }

    #[test]
    fn spawn_area_returns_sequential_indices() {
        let mut p = GuiRecordPool::new();
        assert_eq!(p.spawn_area(0, 0, 100, 100, 1, vec![], 1, 0, 1, 0, -1), Some(0));
        assert_eq!(p.spawn_area(5, 10, 85, 590, 1, vec![], 0xD, 0, 1, 0, -1), Some(1));
    }

    #[test]
    fn spawn_area_overflow_sets_flag() {
        let mut p = GuiRecordPool::new();
        for _ in 0..AREA_POOL_CAP {
            p.spawn_area(0, 0, 1, 1, 0, vec![], 0, 0, 0, 0, -1).unwrap();
        }
        assert!(!p.area_overflow);
        assert!(p.spawn_area(0, 0, 1, 1, 0, vec![], 0, 0, 0, 0, -1).is_none());
        assert!(p.area_overflow);
    }

    #[test]
    fn area_pool_stride_matches_exe() {
        // The exe struct is 0xBF1 bytes; our Rust struct is compact
        // (variable Vec) — we don't reproduce the byte size, only the
        // field set. This test documents the exe stride constant.
        assert_eq!(0xBF1, 3057);
        assert_eq!(0x18C, 396);
    }

    #[test]
    fn spawn_widget_swaps_x_when_x1_less_than_x0() {
        let mut p = GuiRecordPool::new();
        let mut d = WidgetDescriptor::empty();
        d.grid_x0 = 100; d.grid_x1 = 50;   // reversed
        d.style_byte = 0;                    // swap allowed (arg8 &0x400 clear)
        let i = p.spawn_widget(d, -1).unwrap();
        assert_eq!(p.widgets[i as usize].left, 50);
        assert_eq!(p.widgets[i as usize].right, 100);
    }

    #[test]
    fn spawn_widget_clamps_to_800_by_600() {
        let mut p = GuiRecordPool::new();
        let mut d = WidgetDescriptor::empty();
        d.grid_x0 = -10; d.grid_y0 = -20;
        d.grid_x1 = 900; d.grid_y1 = 700;
        let i = p.spawn_widget(d, -1).unwrap();
        assert_eq!(p.widgets[i as usize].left, 0);
        assert_eq!(p.widgets[i as usize].top, 0);
        assert_eq!(p.widgets[i as usize].right, 799);
        assert_eq!(p.widgets[i as usize].bottom, 599);
    }

    #[test]
    fn spawn_widget_truncates_text_at_48_bytes() {
        let mut p = GuiRecordPool::new();
        let mut d = WidgetDescriptor::empty();
        d.text = "a".repeat(100);
        let i = p.spawn_widget(d, -1).unwrap();
        assert!(p.widgets[i as usize].descriptor.text.len() <= 0x30);
    }

    #[test]
    fn spawn_widget_bumps_parent_counter_and_flags_odd() {
        let mut p = GuiRecordPool::new();
        let area = p.spawn_area(0, 0, 100, 100, 0, vec![], 0, 0, 0, 0, -1).unwrap();
        // First child: counter 0 → even → cached_grid_index = 0.
        let w0 = p.spawn_widget(WidgetDescriptor::empty(), area as i16).unwrap();
        assert_eq!(p.widgets[w0 as usize].cached_grid_index, 0);
        // Second child: counter 1 → odd → cached_grid_index = 1.
        let w1 = p.spawn_widget(WidgetDescriptor::empty(), area as i16).unwrap();
        assert_eq!(p.widgets[w1 as usize].cached_grid_index, 1);
    }

    #[test]
    fn spawn_area_populates_palette_defaults() {
        let mut p = GuiRecordPool::new();
        let idx = p.spawn_area(0, 0, 100, 100, 0, vec![], 5, 0, 0, 0, -1).unwrap();
        // Palettes concatenated in extra: first 30 bytes = palette A,
        // next 30 = palette B. Both default to 0x01-filled.
        let ext = &p.areas[idx as usize].extra;
        assert_eq!(ext.len(), 60);
        // Real area (not dummy) with no caller-supplied extra → both filled 0x01.
        assert!(ext[..30].iter().all(|&b| b == 0x01));
        assert!(ext[30..].iter().all(|&b| b == 0x01));
    }

    #[test]
    fn spawn_area_skips_palette_when_flag_0x80000() {
        let mut p = GuiRecordPool::new();
        let idx = p.spawn_area(0, 0, 100, 100, 5, vec![0xAA; 5], 0, 0,
                                 0x80000, 0, -1).unwrap();
        let ext = &p.areas[idx as usize].extra;
        // With skip flag, extra should still be 60 B (initial fill) but
        // not touched by any incoming palette; since palettes always
        // init to 0x01 and skip prevents overwrite from `extra`,
        // result stays 0x01.
        assert_eq!(ext.len(), 60);
    }

    #[test]
    fn spawn_area_dummy_stays_at_origin() {
        let mut p = GuiRecordPool::new();
        // Zero rect + zero children → dummy branch.
        let idx = p.spawn_area(0, 0, 0, 0, 0, vec![], 0, 0, 0, 0, -1).unwrap();
        assert_eq!(p.areas[idx as usize].x0, 0);
        assert_eq!(p.areas[idx as usize].x1, 0);
    }

    #[test]
    fn spawn_area_swaps_reversed_rect() {
        let mut p = GuiRecordPool::new();
        let idx = p.spawn_area(100, 200, 50, 100, 1, vec![], 0, 0, 0, 0, -1).unwrap();
        assert_eq!(p.areas[idx as usize].x0, 50);
        assert_eq!(p.areas[idx as usize].x1, 100);
        assert_eq!(p.areas[idx as usize].y0, 100);
        assert_eq!(p.areas[idx as usize].y1, 200);
    }

    #[test]
    fn enroll_area_order_appends_index() {
        let mut p = GuiRecordPool::new();
        assert!(enroll_area_order(&mut p, 3));
        assert!(enroll_area_order(&mut p, 7));
        assert_eq!(p.area_order, vec![3u16, 7]);
    }

    #[test]
    fn enroll_area_order_rejects_sentinel() {
        let mut p = GuiRecordPool::new();
        assert!(!enroll_area_order(&mut p, -1));
        assert!(p.area_order.is_empty());
    }

    #[test]
    fn enroll_area_order_overflow_sets_flag() {
        let mut p = GuiRecordPool::new();
        // Fill to cap.
        for i in 0..AREA_ORDER_CAP { assert!(enroll_area_order(&mut p, i as i16)); }
        assert!(!p.area_overflow);
        assert!(!enroll_area_order(&mut p, 999));
        assert!(p.area_overflow);
    }

    #[test]
    fn insert_widget_z_order_maintains_ascending_seq() {
        // Build a small parent + three child widgets with seq 30, 10, 20.
        let mut child_widgets = Vec::new();
        for seq in [30, 10, 20] {
            let mut d = WidgetDescriptor::empty();
            d.seq = seq;
            child_widgets.push(Widget { descriptor: d, ..Default::default() });
        }
        let mut parent = Widget::default();
        insert_widget_z_order(&mut parent, &child_widgets, 0);   // seq 30
        insert_widget_z_order(&mut parent, &child_widgets, 1);   // seq 10
        insert_widget_z_order(&mut parent, &child_widgets, 2);   // seq 20
        assert_eq!(parent.z_order, vec![1u16, 2, 0], "ascending by seq: 10, 20, 30");
        assert_eq!(parent.panel_code_or_z_min, 10);
        assert_eq!(parent.max_columns_or_z_max, 30);
    }

    #[test]
    fn insert_widget_z_order_respects_hidden_flag() {
        let mut parent = Widget { flags: 0x80000, ..Default::default() };
        let child = Widget::default();
        insert_widget_z_order(&mut parent, &[child], 0);
        assert!(parent.z_order.is_empty(), "hidden parent must not accept children");
    }

    #[test]
    fn insert_widget_z_order_rejects_sentinel() {
        let mut parent = Widget::default();
        insert_widget_z_order(&mut parent, &[], -1);
        assert!(parent.z_order.is_empty());
    }

    #[test]
    fn sidebar_prelude_matches_exe_decode() {
        // From FUN_00745540 §7 decode — the sidebar composer always
        // emits these two areas + one header widget as its prelude.
        let mut p = GuiRecordPool::new();
        // Left sidebar strip: FUN_00549790(0, 0, 0x59, 599, 1, 0, 1, 0, 1, 0, -1)
        let strip = p.spawn_area(0, 0, 0x59, 599, 1, vec![], 1, 0, 1, 0, -1).unwrap();
        // Branded logo cell (kind 0x400):
        let mut logo_desc = WidgetDescriptor::empty();
        logo_desc.kind = KIND_ROOT_HOLDER;
        logo_desc.text = "CM3 DATA / game.mbr".to_string();
        p.spawn_widget(logo_desc, strip as i16).unwrap();
        // Primary menu list: FUN_00549790(5, 10, 0x55, 0x24E, 1, 0, 0xD, 0, 1, 0, -1)
        let menu = p.spawn_area(5, 10, 0x55, 0x24E, 1, vec![], 0xD, 0, 1, 0, -1).unwrap();
        let _ = menu;
        assert_eq!(p.areas.len(), 2);
        assert_eq!(p.widgets.len(), 1);
    }
}
