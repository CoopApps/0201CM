//! In-engine modal MessageBox — direct port of `FUN_005D1C30` from
//! cm0102.exe (7,118 bytes decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_populators/0x005d1c30.c`.
//! Full decode: [`reports/gui_windows_msgbox_decode.md`](../../../../reports/gui_windows_msgbox_decode.md) §5.
//!
//! # Layout
//!
//! * y band fixed at `[0xF2 (242), 0x165 (357)]`.
//! * width computed from the body: `w = measure(font=1, body) + 0x18`,
//!   clamped to `[200, 600]`.
//! * `x0 = (800 - w) / 2`, `x1 = x0 + w - 1`.
//! * Button y: `[0x13E (318), 0x156 (342)]`, height 24, width `0x4A (74)`.
//! * Single button: OK centered — `ok_x = x0 + (w - 0x4B) / 2`.
//! * With quit: two buttons — `ok_x = x0 + (w - 0x8C) / 2`, quit at `[ok_x+0x55, ok_x+0x9F]`.
//!
//! Every literal is documented against the exe's source line.

/// One button rect emitted by the modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsgBoxButton {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Complete layout of a MessageBox as `FUN_005D1C30` computes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgBoxLayout {
    /// Outer frame LTRB.
    pub outer: (i32, i32, i32, i32),
    /// Title panel LTRB (inset 2px on x, y `[0xF4, 0x108]`).
    pub title_panel: (i32, i32, i32, i32),
    /// Title text rect (10-px inset on x).
    pub title_text: (i32, i32, i32, i32),
    /// Body text rect (12-px inset on x, y `[0x10A, 0x13B]`).
    pub body_text: (i32, i32, i32, i32),
    /// OK button rect.
    pub ok_button: MsgBoxButton,
    /// Optional Quit button rect.
    pub quit_button: Option<MsgBoxButton>,
}

/// The frame/panel color slots the exe reads at draw time. Names match
/// the decode's `DAT_` globals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsgBoxColors {
    /// `DAT_00ACDF6E` — outer frame color (bevel).
    pub frame: u16,
    /// `DAT_00ACDF74` — text color (title + body).
    pub text: u16,
    /// `DAT_00AD6BF4` — title-bar background.
    pub title_bg: u16,
}

/// Style flag literals the exe passes to `FUN_005CF8E0` (filled box).
pub mod style {
    /// Outer bevel frame — style word `0x30`.
    pub const OUTER_FRAME: u32 = 0x30;
    /// Inner title-panel inset — style word `0x210`.
    pub const TITLE_INSET: u32 = 0x210;
    /// Button normal state — `0x30`.
    pub const BUTTON_NORMAL: u32 = 0x30;
    /// Button pressed state — `0x70` (bevel inverted + offset+2).
    pub const BUTTON_PRESSED: u32 = 0x70;
}

/// Font slot literals from the exe (`FUN_005D0870` arg 5).
pub mod fonts {
    pub const TITLE: u8 = 9;
    pub const BODY: u8 = 5;
    pub const BUTTON_LABEL: u8 = 0x0C;
    /// Measure-string font used by `FUN_005CF610(1, body)`.
    pub const MEASURE: u8 = 1;
}

/// Direct port of the geometry block at the top of `FUN_005D1C30`.
///
/// Takes the pre-measured body-text width (what `FUN_005CF610(1, body)`
/// would return) so the port stays independent of the font renderer.
// GDI-REG: 005d1c30 PORTED_BEHAVIOURAL
pub fn compute_msgbox_layout(body_measured_width: i32, has_quit: bool) -> MsgBoxLayout {
    // exe: `w = measure + 0x18` (24-pixel padding).
    let w_raw = body_measured_width + 0x18;
    // exe: `if (w < 201) w = 200; else if (w > 599) w = 600`.
    let w = if w_raw < 201 { 200 } else if w_raw > 599 { 600 } else { w_raw };
    // exe: `x0 = (800 - w) / 2`; `x1 = x0 + w - 1`.
    let x0 = (800 - w) / 2;
    let x1 = x0 + w - 1;
    // exe: fixed y band.
    let y0 = 0xF2;
    let y1 = 0x165;

    // Button y band.
    let btn_top = 0x13E;
    let btn_bot = 0x156;

    // Button centring — same OK width 0x4A (74). Single: total 0x4B (75);
    // two-button: total 0x8C (140 = 0x4A + gap 0xB + 0x4A + trailing 5).
    let ok_button = if !has_quit {
        let ox = x0 + (w - 0x4B) / 2;
        MsgBoxButton { left: ox, top: btn_top, right: ox + 0x4A, bottom: btn_bot }
    } else {
        let ox = x0 + (w - 0x8C) / 2;
        MsgBoxButton { left: ox, top: btn_top, right: ox + 0x4A, bottom: btn_bot }
    };
    let quit_button = if has_quit {
        // exe: `[ok_x+0x55, 0x13E, ok_x+0x9F, 0x156]`.
        Some(MsgBoxButton {
            left: ok_button.left + 0x55,
            top: btn_top,
            right: ok_button.left + 0x9F,
            bottom: btn_bot,
        })
    } else { None };

    MsgBoxLayout {
        outer: (x0, y0, x1, y1),
        title_panel: (x0 + 2, 0xF4, x1 - 2, 0x108),
        title_text: (x0 + 10, 0xF4, x1 - 10, 0x108),
        body_text: (x0 + 12, 0x10A, x1 - 12, 0x13B),
        ok_button,
        quit_button,
    }
}

/// The re-entry guard flag from `DAT_00AF6C20`. When true, a second
/// call to the MessageBox is silently dropped (matches exe: if
/// `DAT_00AF6C20` is 1, the fallback path returns without recursing).
#[derive(Debug, Clone, Copy, Default)]
pub struct MsgBoxGuard { active: bool }

impl MsgBoxGuard {
    pub fn is_active(self) -> bool { self.active }
    pub fn enter(&mut self) -> bool {
        if self.active { false } else { self.active = true; true }
    }
    pub fn leave(&mut self) { self.active = false; }
}

/// Dismiss reasons the modal's event loop can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgBoxDismiss {
    /// User clicked OK.
    Ok,
    /// User pressed ENTER (`DAT_00B4D57C == 0xD`).
    Enter,
    /// User clicked Quit — the exe calls `FUN_005B6A10();
    /// FUN_0061D290(); FUN_009349C4(-1);` (noreturn). The Rust port
    /// signals this to the host so it can execute the shutdown chain.
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrow_body_clamps_to_min_200() {
        // Any body <= 176 wide (200 - 24) should clamp width to 200.
        let l = compute_msgbox_layout(50, false);
        assert_eq!(l.outer, ((800 - 200) / 2, 0xF2,
                             (800 - 200) / 2 + 199, 0x165));
    }

    #[test]
    fn wide_body_clamps_to_max_600() {
        let l = compute_msgbox_layout(1000, false);
        assert_eq!(l.outer.2 - l.outer.0 + 1, 600, "width clamped to 600");
    }

    #[test]
    fn ok_button_is_74_pixels_wide_and_24_tall() {
        let l = compute_msgbox_layout(300, false);
        assert_eq!(l.ok_button.right - l.ok_button.left, 0x4A);
        assert_eq!(l.ok_button.bottom - l.ok_button.top, 0x18);   // 342 - 318
    }

    #[test]
    fn two_button_layout_puts_quit_11px_after_ok() {
        let l = compute_msgbox_layout(400, true);
        let q = l.quit_button.unwrap();
        assert_eq!(q.left - l.ok_button.right, 0xB);
        assert_eq!(q.right - q.left, 0x4A);
    }

    #[test]
    fn no_quit_button_when_has_quit_false() {
        let l = compute_msgbox_layout(300, false);
        assert!(l.quit_button.is_none());
    }

    #[test]
    fn title_panel_and_body_rects_match_exe_y_bands() {
        let l = compute_msgbox_layout(300, false);
        assert_eq!(l.title_panel.1, 0xF4);
        assert_eq!(l.title_panel.3, 0x108);
        assert_eq!(l.body_text.1, 0x10A);
        assert_eq!(l.body_text.3, 0x13B);
    }

    #[test]
    fn reentry_guard_is_idempotent_and_reusable() {
        let mut g = MsgBoxGuard::default();
        assert!(g.enter());
        assert!(!g.enter(), "second entry rejected");
        g.leave();
        assert!(g.enter(), "leave allows re-entry");
    }

    #[test]
    fn font_slots_match_exe_literals() {
        assert_eq!(fonts::TITLE, 9);
        assert_eq!(fonts::BODY, 5);
        assert_eq!(fonts::BUTTON_LABEL, 0x0C);
        assert_eq!(fonts::MEASURE, 1);
    }

    #[test]
    fn style_words_match_exe_literals() {
        assert_eq!(style::OUTER_FRAME, 0x30);
        assert_eq!(style::TITLE_INSET, 0x210);
        assert_eq!(style::BUTTON_NORMAL, 0x30);
        assert_eq!(style::BUTTON_PRESSED, 0x70);
    }

    #[test]
    fn body_rect_x_uses_12_pixel_inset() {
        let l = compute_msgbox_layout(300, false);
        assert_eq!(l.body_text.0 - l.outer.0, 12);
        assert_eq!(l.outer.2 - l.body_text.2, 12);
    }
}
