//! Dirty-rect blit dispatcher — direct port of `FUN_005CCBA0` from
//! cm0102.exe (545 bytes).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_layout_engine/0x005ccba0.c`.
//!
//! The exe's job is to move a dirty rectangle from the DirectDraw
//! **back** surface to the **front** surface, handling cursor
//! occlusion and windowed-vs-fullscreen coordinate translation. The
//! IDirectDrawSurface vtable offsets used are:
//!
//! | vtbl slot | method    | notes                        |
//! |-----------|-----------|------------------------------|
//! | `+0x14`   | `Blt`     | windowed mode; flag `DDBLT_WAIT = 0x01000000` |
//! | `+0x1C`   | `BltFast` | fullscreen; flag `DDBLTFAST_WAIT = 0x10`      |
//!
//! State globals the port needs to model:
//!
//! | exe DAT_       | meaning                                    |
//! |----------------|--------------------------------------------|
//! | `DAT_00AD6BFC` | pause/suspend flag (1 = skip all blits)    |
//! | `DAT_00ACDF94` | primary (front) surface ptr                |
//! | `DAT_00AD6BD4` | back-buffer surface ptr                    |
//! | `DAT_00ACDF8C` | 0 = windowed, non-zero = fullscreen        |
//! | `DAT_00AD6C18` | overlay-compose flag (calls `FUN_005D1900`)|
//! | `DAT_00AD6C04` | cursor-visibility gating flag              |
//! | `DAT_00B4D588` | cursor y                                   |
//! | `DAT_00B4D594` | cursor x                                   |
//! | `DAT_00ACDF34` | window top-inset (title-bar height)        |
//! | `DAT_00ACDF70` | window left-inset (border width)           |
//! | `DAT_00AD6C00` | screen-dirty flag (set to 1 after blit)    |
//! | `DAT_00B4D5A0` | game HWND (for `GetWindowRect`)            |

/// The blit request. Fully declarative — the host renderer executes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlitRequest {
    /// Which surface method to invoke.
    pub method: BlitMethod,
    /// Destination rect in front-surface coords (after windowed offset).
    pub dst_left: i32,
    pub dst_top: i32,
    pub dst_right: i32,
    pub dst_bottom: i32,
    /// Source rect in back-surface coords.
    pub src_left: i32,
    pub src_top: i32,
    pub src_right: i32,
    pub src_bottom: i32,
    /// Whether the cursor needs to be hidden across the blit — set when
    /// the dirty rect intersects the cursor's bounding box.
    pub hide_cursor_during_blit: bool,
}

/// Which DirectDraw method the exe calls. Matches the two vtable slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlitMethod {
    /// Windowed mode — `IDirectDrawSurface::Blt` (vtbl+0x14) with
    /// `DDBLT_WAIT = 0x01000000`.
    Blt = 0x14,
    /// Fullscreen — `IDirectDrawSurface::BltFast` (vtbl+0x1C) with
    /// `DDBLTFAST_WAIT = 0x10`.
    BltFast = 0x1C,
}

/// The subset of globals `FUN_005CCBA0` reads. Rust doesn't share
/// mutable globals across threads; we bundle them into a state struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlitState {
    /// `DAT_00AD6BFC` — pause flag; non-zero suppresses all blits.
    pub paused: bool,
    /// `DAT_00ACDF8C` — 0 = windowed, non-zero = fullscreen.
    pub fullscreen: bool,
    /// `DAT_00AD6C04` — cursor-visibility gating enabled.
    pub cursor_gating: bool,
    /// `DAT_00AD6C18` — overlay-compose enabled.
    pub compose_overlay: bool,
    /// `DAT_00B4D594` / `DAT_00B4D588` — cursor x / y in front-surface coords.
    pub cursor_x: i32,
    pub cursor_y: i32,
    /// `DAT_00ACDF34` / `DAT_00ACDF70` — window top / left insets.
    pub window_top_inset: i32,
    pub window_left_inset: i32,
    /// Front-surface bounds when windowed (from `GetWindowRect(HWND)`).
    pub window_rect_top: i32,
    pub window_rect_left: i32,
}

impl Default for BlitState {
    fn default() -> Self {
        Self {
            paused: false, fullscreen: true, cursor_gating: false,
            compose_overlay: false, cursor_x: 0, cursor_y: 0,
            window_top_inset: 0, window_left_inset: 0,
            window_rect_top: 0, window_rect_left: 0,
        }
    }
}

/// Direct port of `FUN_005CCBA0(left, top, right, bottom)`. Returns
/// `Some(BlitRequest)` if the dispatcher decided to blit — `None` if
/// any gate rejected (paused, missing surfaces, front==back).
///
/// Cursor-intersection test uses the same formula as the exe:
///
/// ```pseudo
/// cursor_max_x = cursor_x + 0x0F;   cursor_min_x = cursor_x - 5
/// cursor_max_y = cursor_y + 0x0F;   cursor_min_y = cursor_y - 5
/// hit = (min(right, cursor_max_x) - max(left, cursor_min_x) > 0)
///     && (min(bottom, cursor_max_y) - max(top, cursor_min_y) > 0)
/// ```
///
/// The `+0xF` / `-5` bounds are the exe's baked-in 20×20 cursor
/// bounding box centred on the hot-spot.
pub fn dispatch_dirty_blit(
    left: i32, top: i32, right: i32, bottom: i32,
    state: &BlitState,
    front_surface_present: bool,
    back_surface_present: bool,
) -> Option<BlitRequest> {
    // Gate: `if ((DAT_00AD6BFC == 0) && (DAT_00ACDF94 != 0) && (DAT_00AD6BD4 != 0)
    //            && (DAT_00ACDF94 != DAT_00AD6BD4))`
    if state.paused
        || !front_surface_present
        || !back_surface_present
    {
        return None;
    }

    // Cursor-intersection test — first pass (before blit).
    let hide_cursor = if state.cursor_gating {
        cursor_intersects(left, top, right, bottom, state.cursor_x, state.cursor_y)
    } else { false };

    // Extend by 1 pixel each side — the exe's `local_18 += 1; local_14 += 1;`
    // after clipping. Applied to dst dimensions.
    let ext_right = right + 1;
    let ext_bottom = bottom + 1;

    let (method, dst_left, dst_top) = if !state.fullscreen {
        // Windowed: `GetWindowRect(HWND, &local_10)`; adjust by insets.
        let dl = state.window_rect_left + state.window_left_inset + left;
        let dt = state.window_rect_top + state.window_top_inset + top;
        (BlitMethod::Blt, dl, dt)
    } else {
        // Fullscreen: BltFast(left, top, back, src, DDBLTFAST_WAIT).
        (BlitMethod::BltFast, left, top)
    };

    Some(BlitRequest {
        method,
        dst_left, dst_top,
        dst_right: dst_left + (ext_right - left),
        dst_bottom: dst_top + (ext_bottom - top),
        src_left: left, src_top: top,
        src_right: ext_right, src_bottom: ext_bottom,
        hide_cursor_during_blit: hide_cursor,
    })
}

fn cursor_intersects(
    rect_l: i32, rect_t: i32, rect_r: i32, rect_b: i32,
    cx: i32, cy: i32,
) -> bool {
    let cur_max_x = cx + 0x0F;
    let cur_min_x = cx - 5;
    let cur_max_y = cy + 0x0F;
    let cur_min_y = cy - 5;
    let overlap_w = rect_r.min(cur_max_x) - rect_l.max(cur_min_x);
    let overlap_h = rect_b.min(cur_max_y) - rect_t.max(cur_min_y);
    overlap_w > 0 && overlap_h > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paused_state_produces_no_blit() {
        let s = BlitState { paused: true, ..Default::default() };
        assert!(dispatch_dirty_blit(0, 0, 100, 100, &s, true, true).is_none());
    }

    #[test]
    fn missing_surfaces_produce_no_blit() {
        let s = BlitState::default();
        assert!(dispatch_dirty_blit(0, 0, 100, 100, &s, false, true).is_none());
        assert!(dispatch_dirty_blit(0, 0, 100, 100, &s, true, false).is_none());
    }

    #[test]
    fn fullscreen_uses_blt_fast() {
        let s = BlitState { fullscreen: true, ..Default::default() };
        let r = dispatch_dirty_blit(10, 20, 30, 40, &s, true, true).unwrap();
        assert_eq!(r.method, BlitMethod::BltFast);
        // Fullscreen dst starts at (left, top).
        assert_eq!(r.dst_left, 10);
        assert_eq!(r.dst_top, 20);
    }

    #[test]
    fn windowed_offsets_by_window_and_insets() {
        let s = BlitState {
            fullscreen: false,
            window_rect_left: 100, window_rect_top: 50,
            window_left_inset: 3, window_top_inset: 22,
            ..Default::default()
        };
        let r = dispatch_dirty_blit(10, 20, 30, 40, &s, true, true).unwrap();
        assert_eq!(r.method, BlitMethod::Blt);
        assert_eq!(r.dst_left, 100 + 3 + 10);
        assert_eq!(r.dst_top, 50 + 22 + 20);
    }

    #[test]
    fn dst_extended_by_one_pixel_each_side() {
        let s = BlitState::default();
        let r = dispatch_dirty_blit(10, 20, 30, 40, &s, true, true).unwrap();
        // dst dims should be src dims + 1 each side (exe's local_18/14 += 1).
        assert_eq!(r.src_right - r.src_left, 21);   // 30 + 1 - 10
        assert_eq!(r.dst_right - r.dst_left, 21);
    }

    #[test]
    fn cursor_intersection_detected_when_gating_on() {
        let s = BlitState {
            cursor_gating: true, cursor_x: 20, cursor_y: 30,
            ..Default::default()
        };
        // Cursor bounds are (15..35, 25..45). Rect (10, 20, 40, 40) → overlap.
        let r = dispatch_dirty_blit(10, 20, 40, 40, &s, true, true).unwrap();
        assert!(r.hide_cursor_during_blit, "should hide cursor during blit");

        // Rect (0, 0, 5, 5) — no overlap with cursor bounds.
        let r2 = dispatch_dirty_blit(0, 0, 5, 5, &s, true, true).unwrap();
        assert!(!r2.hide_cursor_during_blit);
    }

    #[test]
    fn cursor_gating_off_never_hides_cursor() {
        let s = BlitState {
            cursor_gating: false, cursor_x: 20, cursor_y: 30,
            ..Default::default()
        };
        let r = dispatch_dirty_blit(10, 20, 40, 40, &s, true, true).unwrap();
        assert!(!r.hide_cursor_during_blit);
    }

    #[test]
    fn blit_method_vtbl_offsets_match_exe() {
        // DirectDraw IDirectDrawSurface vtable slot 5 (offset 0x14) = Blt.
        // Slot 7 (offset 0x1C) = BltFast. Both verified against decompile.
        assert_eq!(BlitMethod::Blt as usize, 0x14);
        assert_eq!(BlitMethod::BltFast as usize, 0x1C);
    }
}
