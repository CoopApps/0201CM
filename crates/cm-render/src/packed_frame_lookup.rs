//! Direct port of `FUN_00403a20` — the panel/frame metrics lookup that
//! resolves an Area's outer bounds (and the child widget's inner text
//! rect) into two output values consumed by the widget draw pipeline.
//!
//! Source: `d:/cm0102-carve/gdi_carve/functions/00004-PE_section_.text/00036_sub_00403a20.asm`
//! (240 asm lines, `__thiscall` with two out-pointers, `ret 8`).
//!
//! Calling convention:
//! * `this`  = ecx = the current Area
//! * `arg1`  = `[esp+0x210]` (post-prologue) — first out slot (`out_a`)
//! * `arg2`  = `[esp+0x214]` (post-prologue) — second out slot (`out_b`)
//!
//! # High-level shape
//!
//! The function walks:
//!   `this_area → its child widget (via +0x20e) → back to that widget's
//!    "sibling" area (via widget +0x7a) → that sibling area's child widget`
//! and then branches on flag bits in `this_area.border_style` (+0x18)
//! and the sibling widget's `+0x10 / +0x18` slots to produce two frame
//! metrics.
//!
//! # Branch summary (bits are on `this_area.border_style`)
//!
//! * `child_widget_index == -1`          → error-log path (no outputs written)
//! * sibling area unreachable / has 0x100 → short-fallback (path 4/5/6)
//! * sibling widget's `flags < sibling.x0` (jge NOT taken):
//!     `out_a = child.flags + 3`, `out_b = child[+0x14]` or clamp
//! * sibling widget's `flags >= sibling.x0` (jge taken):
//!     `out_a = child[+0x10] - area.x1 + area.x0 - 3` (out_b unset here)
//! * short-fallback + bit `0x100` set (branch 2):
//!     `out_b` from `child[+0x1c]+3` or clamp; `out_a` from `child[+0x10]`
//!     or `child.flags - area.x1 + area.x0`
//! * short-fallback + bit `0x100` clear (branch 3):
//!     `out_b` from `child[+0x14]` or clamp — gated by 0x800000/0x400000;
//!     `out_a` gated by panel/0x200000/0x100000 → `child.flags+3` or
//!     `child[+0x10] - area.x1 + area.x0 - 3`.

use crate::widget_pool::{Area, GuiRecordPool, Widget};

/// The pair of frame metrics returned by [`frame_lookup`]. Each field is
/// `Option` so callers can distinguish "not written" (the exe leaves the
/// out slot untouched on error / on paths that only fill one) from a
/// legitimate zero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameMetrics {
    /// arg1 (`[esp+0x210]` on entry) — sometimes the panel-code, sometimes
    /// a derived x-metric.
    pub out_a: Option<i32>,
    /// arg2 (`[esp+0x214]` on entry) — a height/width metric relative to
    /// the frame's decorated bounds.
    pub out_b: Option<i32>,
}

/// Direct port of `FUN_00403a20`. Every asm branch is preserved.
///
/// Returns a [`FrameMetrics`] rather than writing through pointers; the
/// two `Option<i32>` fields mirror the exact set of out-writes each path
/// performs (the exe would leave the slot untouched on paths that don't
/// write it).
pub fn frame_lookup(this_area: &Area, pool: &GuiRecordPool) -> FrameMetrics {
    let mut out = FrameMetrics::default();

    // 00403a20  mov ax, [ecx + 0x20e]           ; this.child_widget_index
    // 00403a2d  cmp ax, 0xffff
    // 00403a34  jne 0x403aa5                    ; -1 → error-log path
    let cw_idx = this_area.child_widget_index;
    if cw_idx == -1 {
        // 00403a36..00403aa2 — error-log path. Elided calls, with cites:
        //   00403a4c  call 0x9335d9   ; format-string composer (varargs)
        //   00403a66  call 0x8fbf90   ; string builder / concat
        //   00403a76  call 0x933579   ; log dispatcher (0xacd620 = log ctx)
        //   00403a87  call 0x5d1670   ; error-message routing
        //   00403a8f  mov  [0xb4d4f0], 0   ; DAT_00b4d4f0 side-effect
        // The four callees are the CRT/game error-logging pipeline; we
        // do not port their side effects (log dispatch, message queue).
        // DAT_00b4d4f0: no readers in the decompiled corpus
        // (`grep -riE b4d4f0 D:/cm0102-carve/ghidra_out/` finds only
        // its data-symbol entry in `cm0102_GDI.exe/data_symbols.json`,
        // no code references), so the write to zero is a dead-global
        // side effect intentionally elided in the port. Match the asm's
        // observable behaviour: leave both out-slots untouched.
        return out;
    }

    // 00403aa5..00403ab8  child_widget_ptr = widget_pool_base + cw_idx*0x18C
    let child_widget = match pool.widgets.get(cw_idx as usize) {
        Some(w) => w,
        None => return out, // exe would read garbage; safe port bails.
    };

    // 00403abb  mov dx, [eax + 0x7a]            ; child_widget.parent_area_link
    // 00403abf  cmp dx, -1
    // 00403ac3  je  0x403bc5                    ; short-fallback
    let sibling_area_idx = child_widget.parent_area_link;
    if sibling_area_idx == -1 {
        return short_fallback(this_area, pool, child_widget, &mut out);
    }

    // 00403ac9..00403ad9  sibling_area_ptr = area_pool_base + parent_link*0xBF1
    let sibling_area = match pool.areas.get(sibling_area_idx as usize) {
        Some(a) => a,
        None => return out,
    };

    // 00403ae1  cmp word [edx + 0x20e], -1      ; sibling.child_widget_index
    // 00403ae9  je  0x403bc5                    ; short-fallback
    if sibling_area.child_widget_index == -1 {
        return short_fallback(this_area, pool, child_widget, &mut out);
    }
    // 00403aef  test dword [edx + 0x18], 0x100  ; sibling.border_style & 0x100
    // 00403af6  jne 0x403bc5                    ; short-fallback
    if sibling_area.border_style & 0x100 != 0 {
        return short_fallback(this_area, pool, child_widget, &mut out);
    }

    // ------------------------------------------------------------------
    // MAIN PATH (0x403afc..0x403bc2). All accesses here treat `this_area`
    // as the container and `child_widget` (the child at +0x20e) as the
    // frame source. The sibling-area walk we just did was a *reachability
    // gate* — the actual metric read is off `child_widget`.
    // ------------------------------------------------------------------

    // 00403afc  esi = [ecx + 0xc]               ; this.y1
    // 00403aff  edx = [ecx + 4]                 ; this.y0
    // 00403b02  edi = [eax + 0x14]              ; child_widget[+0x14] (max_columns slot)
    // 00403b05  ebx = esi - edx + edi           ; = (y1-y0) + child[+0x14]
    // 00403b0b  cmp ebx, 0x258
    let y1 = this_area.y1;
    let y0 = this_area.y0;
    let child14 = child_widget.max_columns_or_z_max; // +0x14
    let hsum = y1.wrapping_sub(y0).wrapping_add(child14);
    if hsum < 0x258 {
        // 00403b13..00403b1c   *out_b = edi
        out.out_b = Some(child14);
    } else {
        // 00403b1e..00403b36
        //   eax = child[+0x1c]                  ; unk_0x1c_right_edge
        //   eax = eax - esi + edx               ; - y1 + y0
        //   (branchless max(eax, 0)):
        //     xor edx,edx; setle dl; dec edx; and eax,edx
        let eax = child_widget
            .unk_0x1c_right_edge
            .wrapping_sub(y1)
            .wrapping_add(y0);
        let clamped = if eax > 0 { eax } else { 0 };
        out.out_b = Some(clamped);
    }

    // ------------------------------------------------------------------
    // 00403b38..00403b85  SECOND WALK: re-read child index, chase to
    // sibling area, then read the SIBLING'S child widget's flags and
    // compare with sibling_area.x0. This decides between the two
    // "main-path" tail computations.
    // ------------------------------------------------------------------

    // 00403b38  esi = sx([ecx + 0x20e])         ; child idx (again)
    // 00403b3f  edx = [ecx + 0x208]             ; widget pool base
    // 00403b52  eax = sx([widget[esi] + 0x7a])  ; parent_area_link
    // (identical to sibling_area_idx we already have)
    // 00403b57..00403b6a  esi = &area[eax]      ; sibling_area again
    // 00403b6c  edi = sx([esi + 0x20e])         ; sibling.child_widget_index
    // 00403b7d  edx = widget[edi].flags         ; sibling's child widget flags
    // 00403b81  eax = [esi]                     ; sibling.x0
    // 00403b83  cmp edx, eax
    // 00403b85  jge 0x403ba2
    let sibling_child_idx = sibling_area.child_widget_index;
    let sibling_child_widget = match pool.widgets.get(sibling_child_idx as usize) {
        Some(w) => w,
        None => return out,
    };
    let sibling_child_flags = sibling_child_widget.flags as i32; // signed cmp per jge
    let sibling_x0 = sibling_area.x0;

    if sibling_child_flags < sibling_x0 {
        // 00403b87..00403b9f
        //   eax = [ebx + 0x18]                  ; child_widget.flags (ebx=&child_widget)
        //   ecx = [esp + 0x210] = arg1
        //   eax += 3
        //   *arg1 = eax
        out.out_a = Some((child_widget.flags as i32).wrapping_add(3));
    } else {
        // 00403ba2..00403bc2  (jge taken)
        //   edx = [ebx + 0x10]                  ; child_widget.panel_code_or_z_min
        //   eax = [ecx + 8]                     ; this.x1
        //   edx -= eax                          ; panel - x1
        //   eax = [ecx]                         ; this.x0
        //   ecx = edx + eax - 3
        //   *arg1 = ecx
        let v = child_widget
            .panel_code_or_z_min
            .wrapping_sub(this_area.x1)
            .wrapping_add(this_area.x0)
            .wrapping_sub(3);
        out.out_a = Some(v);
    }
    out
}

/// Short-fallback path starting at label `0x403bc5`. Reached when the
/// sibling area is unreachable (`parent_area_link == -1`, sibling's
/// `child_widget_index == -1`, or sibling `border_style & 0x100 != 0`).
///
/// This path branches on `this_area.border_style` bit `0x100` (`bh & 1`
/// at asm line `00403bc8`):
///
/// * bit set     → branch 2 (`0x403bcb..0x403c62`)
/// * bit clear   → branch 3 (`0x403c65..0x403d21`)
fn short_fallback<'a>(
    this_area: &Area,
    pool: &GuiRecordPool,
    child_widget: &'a Widget,
    out: &mut FrameMetrics,
) -> FrameMetrics {
    // 00403bc5  ebx = [ecx + 0x18]              ; this.border_style
    let this_flags = this_area.border_style;
    // 00403bc8  test bh, 1                      ; bit 0x100
    // 00403bcb  je   0x403c65                   ; → branch 3
    if this_flags & 0x100 != 0 {
        branch2(this_area, pool, child_widget, out);
    } else {
        branch3(this_area, pool, child_widget, this_flags, out);
    }
    *out
}

/// Branch 2 — `0x100` bit SET on `this.border_style`. Covers
/// `0x403bcb..0x403c62`.
fn branch2(this_area: &Area, pool: &GuiRecordPool, child_widget: &Widget, out: &mut FrameMetrics) {
    // 00403bd1  edi = [eax + 0x1c]              ; child.unk_0x1c_right_edge
    // 00403bd4  edx = [ecx + 4]                 ; this.y0
    // 00403bd7  esi = [ecx + 0xc]               ; this.y1
    // 00403bda  ebx = edi
    // 00403bdc  sub ebx, edx
    // 00403bde  lea ebx, [ebx + esi + 3]        ; (edi - y0) + y1 + 3
    // 00403be2  cmp ebx, 0x258
    let edi = child_widget.unk_0x1c_right_edge;
    let y0 = this_area.y0;
    let y1 = this_area.y1;
    let hsum = edi
        .wrapping_sub(y0)
        .wrapping_add(y1)
        .wrapping_add(3);
    if hsum < 0x258 {
        // 00403bea..00403bf4
        //   *arg2 = edi + 3
        out.out_b = Some(edi.wrapping_add(3));
    } else {
        // 00403bf8..00403c08
        //   eax = [eax + 0x14]                  ; child[+0x14]
        //   eax -= esi                          ; - y1
        //   edx = eax + edx - 3                 ; + y0 - 3
        //   *arg2 = edx
        let v = child_widget
            .max_columns_or_z_max
            .wrapping_sub(y1)
            .wrapping_add(y0)
            .wrapping_sub(3);
        out.out_b = Some(v);
    }

    // 00403c0a..0x403c62 — the "walk again → panel test → out_a" tail.
    // Skip the redundant walk (we still have child_widget), but preserve
    // the read semantics: panel = child.panel_code_or_z_min.
    let _ = pool; // walk elided; child_widget is the same pointer.
    let panel = child_widget.panel_code_or_z_min;
    // 00403c27  cmp edx, 0x190
    // 00403c2d  jg  0x403c44
    if panel <= 0x190 {
        // 00403c2f..00403c41    *arg1 = panel
        out.out_a = Some(panel);
    } else {
        // 00403c44..00403c62
        //   edx = [eax + 0x18]                  ; child.flags
        //   esi = [ecx + 8]                     ; this.x1
        //   eax = [ecx]                         ; this.x0
        //   edx = child.flags - x1 + x0
        //   *arg1 = edx
        let v = (child_widget.flags as i32)
            .wrapping_sub(this_area.x1)
            .wrapping_add(this_area.x0);
        out.out_a = Some(v);
    }
}

/// Branch 3 — `0x100` bit CLEAR on `this.border_style`. Covers
/// `0x403c65..0x403d21`.
fn branch3(
    this_area: &Area,
    pool: &GuiRecordPool,
    child_widget: &Widget,
    this_flags: u32,
    out: &mut FrameMetrics,
) {
    // 00403c65  esi = [ecx + 0xc]               ; y1
    // 00403c68  edx = [ecx + 4]                 ; y0
    // 00403c6b  edi = [eax + 0x14]              ; child[+0x14]
    // 00403c6e  push ebp                        (register save; no logic)
    // 00403c6f  ebp = esi - edx + edi
    // 00403c75  cmp ebp, 0x258
    // 00403c7b  pop ebp
    // 00403c7c  jl  0x403c86                    ; ep < 600 → handle_400000_or_leq
    // 00403c7e  test ebx, 0x800000              ; else check 0x800000
    // 00403c84  je   0x403c99                   ; clear → compute_from_1c
    let y0 = this_area.y0;
    let y1 = this_area.y1;
    let edi = child_widget.max_columns_or_z_max; // +0x14
    let ep = y1.wrapping_sub(y0).wrapping_add(edi);

    let compute_from_1c = if ep < 0x258 {
        // handle_400000_or_leq (line 192):
        // 00403c86  test ebx, 0x400000
        // 00403c8c  jne  0x403c99               ; set → compute_from_1c
        // 00403c8e..0x403c97   *arg2 = edi; jmp branch3_common
        if this_flags & 0x400000 != 0 {
            true
        } else {
            out.out_b = Some(edi);
            false
        }
    } else if this_flags & 0x800000 == 0 {
        true
    } else {
        // hsum >= 0x258 AND 0x800000 set → fall through to handle_400000_or_leq
        if this_flags & 0x400000 != 0 {
            true
        } else {
            out.out_b = Some(edi);
            false
        }
    };

    if compute_from_1c {
        // 00403c99..0x403cb1
        //   eax = [eax + 0x1c]                  ; child.unk_0x1c_right_edge
        //   eax = eax - esi + edx               ; - y1 + y0
        //   branchless max(eax, 0)
        //   *arg2 = eax
        let eax = child_widget
            .unk_0x1c_right_edge
            .wrapping_sub(y1)
            .wrapping_add(y0);
        let clamped = if eax > 0 { eax } else { 0 };
        out.out_b = Some(clamped);
    }

    // 00403cb3..0x403d21 — branch3_common: walk again, panel test,
    // gated by 0x200000/0x100000 on this.border_style.
    let _ = pool;
    let panel = child_widget.panel_code_or_z_min;

    // 00403ccd  eax = [edx + 0x10]              ; panel
    // 00403cd0  cmp eax, 0x190
    // 00403cd5  jle 0x403ce0
    //   if panel > 0x190 → line 217:
    //     test [ecx+0x18], 0x200000
    //     je 0x403d04                           ; clear → compute_final_from_1c
    //   else (panel <= 0x190) → jump to 0x403ce0
    //     test [ecx+0x18], 0x100000
    //     jne 0x403d04                          ; set → compute_final_from_1c
    //     ; else use +0x18
    let compute_final_from_1c = if panel > 0x190 {
        if this_flags & 0x200000 == 0 {
            true
        } else {
            // fall through to 0x403ce0
            this_flags & 0x100000 != 0
        }
    } else {
        this_flags & 0x100000 != 0
    };

    if !compute_final_from_1c {
        // 00403ce9..0x403d01
        //   eax = [edx + 0x18]                  ; child.flags
        //   *arg1 = eax + 3
        out.out_a = Some((child_widget.flags as i32).wrapping_add(3));
    } else {
        // 00403d04..0x403d21
        //   edx = [ecx + 8]                     ; x1
        //   eax = panel - x1
        //   edx = [ecx]                         ; x0
        //   eax = eax + x0 - 3
        //   *arg1 = eax
        let v = panel
            .wrapping_sub(this_area.x1)
            .wrapping_add(this_area.x0)
            .wrapping_sub(3);
        out.out_a = Some(v);
    }
}

// ============================================================================
// Tests
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget_pool::{Widget, WidgetDescriptor};

    /// Build a pool with one Area + one Widget child. The Area's
    /// `child_widget_index` is set to 0 and the Widget's
    /// `parent_area_link` to -1 by default (unless overridden after).
    fn make_pool(area_flags: u32, widget: Widget) -> (Area, GuiRecordPool) {
        let mut pool = GuiRecordPool::new();
        pool.widgets.push(widget);
        let area = Area {
            x0: 10,
            y0: 20,
            x1: 100,
            y1: 200,
            border_style: area_flags,
            child_widget_index: 0,
            ..Default::default()
        };
        (area, pool)
    }

    #[test]
    fn child_widget_neg1_leaves_outputs_untouched() {
        // 00403a34 jne — the fast-bail. Both outputs remain None.
        let pool = GuiRecordPool::new();
        let area = Area {
            child_widget_index: -1,
            ..Default::default()
        };
        let got = frame_lookup(&area, &pool);
        assert_eq!(got, FrameMetrics::default());
    }

    #[test]
    fn short_fallback_bit_0x100_set_panel_le_190_sets_both_outs() {
        // parent_area_link == -1 → short_fallback; area.border_style bit
        // 0x100 set → branch 2; child.unk_0x10 <= 0x190 → out_a = panel.
        let child = Widget {
            parent_area_link: -1,
            panel_code_or_z_min: 0x50,
            unk_0x1c_right_edge: 30,
            max_columns_or_z_max: 40,
            flags: 0,
            ..Default::default()
        };
        let (area, pool) = make_pool(0x100, child);
        let got = frame_lookup(&area, &pool);
        // hsum for out_b: (30 - 20) + 200 + 3 = 213 < 600 → out_b = 30+3 = 33
        assert_eq!(got.out_b, Some(33));
        // panel 0x50 <= 0x190 → out_a = 0x50
        assert_eq!(got.out_a, Some(0x50));
    }

    #[test]
    fn short_fallback_bit_0x100_set_panel_gt_190_uses_x_calc_for_out_a() {
        // panel > 0x190 → out_a = child.flags - x1 + x0
        let child = Widget {
            parent_area_link: -1,
            panel_code_or_z_min: 0x200,
            unk_0x1c_right_edge: 30,
            max_columns_or_z_max: 40,
            flags: 200,
            ..Default::default()
        };
        let (area, pool) = make_pool(0x100, child);
        let got = frame_lookup(&area, &pool);
        assert_eq!(got.out_a, Some(200 - 100 + 10));
    }

    #[test]
    fn short_fallback_bit_0x100_clear_no_special_bits_uses_edi() {
        // border_style bit 0x100 clear → branch 3.
        // hsum = (y1-y0)+edi = 180+40 = 220 < 600 → fits.
        // 0x400000 clear → out_b = edi (child[+0x14]).
        // panel <= 0x190 → check 0x100000: clear → use +0x18: out_a = flags+3.
        let child = Widget {
            parent_area_link: -1,
            panel_code_or_z_min: 0x50,
            unk_0x1c_right_edge: 25,
            max_columns_or_z_max: 40,
            flags: 7,
            ..Default::default()
        };
        let (area, pool) = make_pool(0, child);
        let got = frame_lookup(&area, &pool);
        assert_eq!(got.out_b, Some(40));
        assert_eq!(got.out_a, Some(10));
    }

    #[test]
    fn short_fallback_branch3_0x400000_forces_1c_path() {
        // With 0x400000 set, even a small hsum diverts to compute_from_1c.
        // out_b = max(child.unk_0x1c - y1 + y0, 0)
        //       = max(25 - 200 + 20, 0) = 0
        let child = Widget {
            parent_area_link: -1,
            panel_code_or_z_min: 0x50,
            unk_0x1c_right_edge: 25,
            max_columns_or_z_max: 40,
            flags: 7,
            ..Default::default()
        };
        let (area, pool) = make_pool(0x400000, child);
        let got = frame_lookup(&area, &pool);
        assert_eq!(got.out_b, Some(0));
    }

    #[test]
    fn short_fallback_branch3_0x800000_gates_large_hsum_to_1c_path() {
        // Force hsum >= 0x258 by using a huge max_columns.
        // Without 0x800000: falls to handle_400000_or_leq → 0x400000 clear
        // → out_b = edi.
        // With 0x800000: falls to check 0x400000 (still clear) → out_b = edi.
        // Actually 0x800000 alone (with hsum large) leads to line 197
        // path via handle_400000_or_leq: yes 0x400000 clear → out_b = edi.
        // We test the raw 0x800000-set path.
        let child = Widget {
            parent_area_link: -1,
            panel_code_or_z_min: 0x10,
            unk_0x1c_right_edge: 5,
            max_columns_or_z_max: 1000, // huge → hsum >> 0x258
            flags: 0,
            ..Default::default()
        };
        let (area, pool) = make_pool(0x800000, child);
        let got = frame_lookup(&area, &pool);
        // hsum >= 0x258 and 0x800000 set → handle_400000_or_leq; 0x400000
        // clear → out_b = edi.
        assert_eq!(got.out_b, Some(1000));
    }

    #[test]
    fn short_fallback_branch3_0x200000_gates_out_a_when_panel_gt_190() {
        // panel > 0x190. Without 0x200000: jump to compute_final_from_1c.
        //   → out_a = panel - x1 + x0 - 3 = 0x200 - 100 + 10 - 3 = 419
        // With 0x200000 set: check 0x100000 (clear) → use +0x18: flags+3.
        let child_no = Widget {
            parent_area_link: -1,
            panel_code_or_z_min: 0x200,
            max_columns_or_z_max: 40,
            flags: 99,
            ..Default::default()
        };
        let child_yes = child_no.clone();
        let (area_no, pool_no) = make_pool(0, child_no);
        let (area_yes, pool_yes) = make_pool(0x200000, child_yes);

        let g_no = frame_lookup(&area_no, &pool_no);
        assert_eq!(g_no.out_a, Some(0x200 - 100 + 10 - 3));

        let g_yes = frame_lookup(&area_yes, &pool_yes);
        assert_eq!(g_yes.out_a, Some(99 + 3));
    }

    #[test]
    fn short_fallback_branch3_0x100000_forces_final_from_1c() {
        // panel <= 0x190 + 0x100000 set → compute_final_from_1c.
        //   out_a = panel - x1 + x0 - 3 = 0x50 - 100 + 10 - 3 = -13
        let child = Widget {
            parent_area_link: -1,
            panel_code_or_z_min: 0x50,
            max_columns_or_z_max: 40,
            flags: 99,
            ..Default::default()
        };
        let (area, pool) = make_pool(0x100000, child);
        let got = frame_lookup(&area, &pool);
        assert_eq!(got.out_a, Some(0x50 - 100 + 10 - 3));
    }

    #[test]
    fn main_path_jge_not_taken_writes_flags_plus_3_to_out_a() {
        // Build a pool where the sibling area's child widget's flags <
        // sibling.x0 → jge not taken → out_a = child.flags + 3.
        //
        // Chain: this_area.child_widget_index=0 → widget[0].parent_area_link=1
        //         → area[1].child_widget_index=1 → widget[1].flags < area[1].x0
        let mut pool = GuiRecordPool::new();
        // widget[0] = the "child_widget" of this_area
        pool.widgets.push(Widget {
            parent_area_link: 1,       // points to area[1]
            max_columns_or_z_max: 40,           // +0x14
            unk_0x1c_right_edge: 25,   // +0x1c
            flags: 12,                 // becomes out_a target
            ..Default::default()
        });
        // widget[1] = the sibling's own child widget; its flags are small.
        pool.widgets.push(Widget {
            flags: 5, // < sibling.x0 (below)
            ..Default::default()
        });
        // area[1] = sibling_area
        pool.areas.push(Area::default()); // dummy area[0] to keep indices sane? no, area[1] means push twice.
        pool.areas.push(Area {
            x0: 50,                     // > sibling_child.flags(5) → jge NOT taken
            y0: 0, y1: 0, x1: 0,
            child_widget_index: 1,      // → widget[1]
            border_style: 0,            // 0x100 clear
            ..Default::default()
        });
        let this = Area {
            x0: 10, y0: 20, x1: 100, y1: 200,
            child_widget_index: 0,      // → widget[0]
            border_style: 0,
            ..Default::default()
        };
        let got = frame_lookup(&this, &pool);
        // hsum = (200-20)+40 = 220 < 600 → out_b = 40
        assert_eq!(got.out_b, Some(40));
        // jge NOT taken → out_a = child.flags + 3 = 15
        assert_eq!(got.out_a, Some(15));
    }

    #[test]
    fn main_path_jge_taken_writes_panel_minus_x1_plus_x0_minus_3() {
        // sibling_child.flags >= sibling.x0 → jge taken →
        // out_a = child[+0x10] - this.x1 + this.x0 - 3
        let mut pool = GuiRecordPool::new();
        pool.widgets.push(Widget {
            parent_area_link: 1,
            panel_code_or_z_min: 500,
            max_columns_or_z_max: 40,
            unk_0x1c_right_edge: 25,
            flags: 0,
            ..Default::default()
        });
        pool.widgets.push(Widget {
            flags: 100, // >= sibling.x0
            ..Default::default()
        });
        pool.areas.push(Area::default());
        pool.areas.push(Area {
            x0: 50,
            child_widget_index: 1,
            border_style: 0,
            ..Default::default()
        });
        let this = Area {
            x0: 10, y0: 20, x1: 100, y1: 200,
            child_widget_index: 0,
            border_style: 0,
            ..Default::default()
        };
        let got = frame_lookup(&this, &pool);
        // out_a = 500 - 100 + 10 - 3 = 407
        assert_eq!(got.out_a, Some(407));
    }

    #[test]
    fn sibling_area_border_0x100_diverts_to_short_fallback() {
        // Sibling reachable but sibling.border_style & 0x100 set → jne to
        // short-fallback (branch 3 since this.border_style bit 0x100 clear).
        let mut pool = GuiRecordPool::new();
        pool.widgets.push(Widget {
            parent_area_link: 1,
            max_columns_or_z_max: 40,
            flags: 7,
            ..Default::default()
        });
        pool.areas.push(Area::default());
        pool.areas.push(Area {
            border_style: 0x100, // diverts
            child_widget_index: 0,
            ..Default::default()
        });
        let this = Area {
            x0: 10, y0: 20, x1: 100, y1: 200,
            child_widget_index: 0,
            border_style: 0,
            ..Default::default()
        };
        let got = frame_lookup(&this, &pool);
        // Branch 3, small hsum, no bits → out_b = edi (40), out_a = flags+3 (10).
        assert_eq!(got.out_b, Some(40));
        assert_eq!(got.out_a, Some(10));
    }

    #[test]
    fn short_fallback_bit_0x100_set_large_hsum_uses_max_columns_calc() {
        // Branch 2 with hsum >= 0x258 → out_b = child[+0x14] - y1 + y0 - 3
        let child = Widget {
            parent_area_link: -1,
            unk_0x1c_right_edge: 600, // makes hsum big
            max_columns_or_z_max: 250,
            panel_code_or_z_min: 0x10,
            ..Default::default()
        };
        // area y0=20, y1=200 → (600-20)+200+3 = 783 >= 600
        let (area, pool) = make_pool(0x100, child);
        let got = frame_lookup(&area, &pool);
        // out_b = 250 - 200 + 20 - 3 = 67
        assert_eq!(got.out_b, Some(67));
    }
}
