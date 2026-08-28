//! Main-window + DirectDraw initialization — direct port of
//! `FUN_005CC310` from cm0102.exe (7,949 bytes decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_populators/0x005cc310.c`.
//! Full decode: [`reports/gui_windows_msgbox_decode.md`](../../../../reports/gui_windows_msgbox_decode.md) §3.
//!
//! # Why this is a config struct
//!
//! The exe wraps Win32 + DirectDraw 1 primitives directly:
//! `RegisterClassA` / `CreateWindowExA` / `DirectDrawCreate` /
//! `SetCooperativeLevel` / `SetDisplayMode` / `CreateSurface` /
//! `IDirectDrawClipper::SetHWnd`. The Rust port models what the exe
//! CHOOSES (window style, cooperative-level flag, surface descriptors,
//! font pack) as a declarative [`WindowInitConfig`] — the host
//! renderer (winit + wgpu / softbuffer / SDL) executes the equivalent
//! calls on the target platform.
//!
//! Every literal is documented with its exe source line.

/// Companion of `FUN_005CC310(w, h, flags, font_pack, fullscreen, title)`.
/// Reproduces every decision the exe makes about window + surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInitConfig {
    /// `param_1` — logical canvas width (before fullscreen border adds).
    pub width: u32,
    /// `param_2` — logical canvas height.
    pub height: u32,
    /// `param_3` — feature flag bits. Bit `0x2` = allocate back buffer;
    /// other bits stashed in `DAT_00ACDF3C` for later reads.
    pub flags: u32,
    /// `param_5` — 0 = windowed, non-zero = fullscreen exclusive.
    pub fullscreen: bool,
    /// `param_6` — window title (used as `lpWindowName`).
    pub title: String,
    /// Cached system metrics captured in the first-call branch.
    pub metrics: WindowMetrics,
}

/// The four cached `GetSystemMetrics` values `FUN_005CC310` stores in
/// `DAT_00ACDF70/7C/34/88` + `DAT_00AD6BC8`. In Rust we ask the host
/// once at startup and cache here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowMetrics {
    /// `SM_CXBORDER` — cached to `DAT_00ACDF70`.
    pub cx_border: i32,
    /// `SM_CYBORDER` — cached to `DAT_00ACDF7C`.
    pub cy_border: i32,
    /// `SM_CYMENU + cy_border` — cached to `DAT_00ACDF34` (title-bar +
    /// menu adornment).
    pub cy_adornment: i32,
    /// Desktop width (from `GetWindowRect(GetDesktopWindow())` right).
    pub desktop_width: i32,
    /// Desktop height (bottom).
    pub desktop_height: i32,
}

/// The window style word the exe hands `CreateWindowExA` /
/// `SetWindowLongA(GWL_STYLE, ...)`. Values match exe literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowStyle {
    /// Windowed initial create: `0x000A0000`.
    WindowedInitial = 0x000A_0000,
    /// Windowed after mode-change: `0x00CA0000` = WS_CAPTION|WS_SYSMENU.
    Windowed = 0x00CA_0000,
    /// Fullscreen: `0x80080000` = WS_POPUP|WS_SYSMENU literal.
    Fullscreen = 0x8008_0000u32 as i32 as u32 as _,
}

/// DirectDraw cooperative level flag passed to `SetCooperativeLevel`.
/// Values match exe literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DDCoopLevel {
    /// `DDSCL_NORMAL = 0x08` — windowed.
    Normal = 0x08,
    /// `DDSCL_EXCLUSIVE | DDSCL_FULLSCREEN = 0x13`.
    ExclusiveFullscreen = 0x13,
}

/// Primary surface descriptor — matches the exe's stack-built
/// `DDSURFACEDESC` at `local_a4` (dwSize=0x6C, dwFlags=1, dwCaps=0x200).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimarySurfaceDesc {
    /// Always `0x6C` (108) — sizeof(DDSURFACEDESC) in DirectDraw 1.
    pub dw_size: u32,
    /// Always `1` — DDSD_CAPS.
    pub dw_flags: u32,
    /// Always `0x200` — DDSCAPS_PRIMARYSURFACE.
    pub dw_caps: u32,
}

impl Default for PrimarySurfaceDesc {
    fn default() -> Self { Self { dw_size: 0x6C, dw_flags: 1, dw_caps: 0x200 } }
}

/// Back surface descriptor — allocated only when `flags & 2`. Matches
/// the exe: dwSize=0x6C, dwFlags=7 (DDSD_CAPS|DDSD_HEIGHT|DDSD_WIDTH),
/// dwCaps=0x840 (DDSCAPS_OFFSCREENPLAIN|DDSCAPS_SYSTEMMEMORY).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackSurfaceDesc {
    pub dw_size: u32,
    pub dw_flags: u32,
    pub dw_width: u32,
    pub dw_height: u32,
    pub dw_caps: u32,
}

impl BackSurfaceDesc {
    pub fn for_dims(w: u32, h: u32) -> Self {
        Self { dw_size: 0x6C, dw_flags: 7, dw_width: w, dw_height: h,
               dw_caps: 0x840 }
    }
}

/// The plan the exe would execute — this is what a host implementation
/// consumes to bring up its own window + surfaces equivalently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInitPlan {
    pub style: WindowStyle,
    /// Effective canvas size the exe passes to CreateWindowExA (windowed
    /// mode adds 2*CXBORDER to width, CYMENU+CYBORDER to height).
    pub outer_width: u32,
    pub outer_height: u32,
    /// Top-left of the window on the desktop (centered in windowed
    /// mode; (0,0) for fullscreen).
    pub outer_x: i32,
    pub outer_y: i32,
    pub coop_level: DDCoopLevel,
    /// SetDisplayMode(w, h, 16) target — always 16-bit 5-6-5.
    pub display_mode_width: u32,
    pub display_mode_height: u32,
    pub display_mode_bpp: u32,
    pub primary_desc: PrimarySurfaceDesc,
    pub back_desc: Option<BackSurfaceDesc>,
    pub create_clipper: bool,
}

impl WindowInitConfig {
    /// Compute the derived plan — direct port of the branch structure
    /// inside `FUN_005CC310` (§3 of the decode).
    pub fn plan(&self) -> WindowInitPlan {
        let want_back = self.flags & 0x2 != 0;
        if self.fullscreen {
            // Fullscreen: dwStyle=0x80080000; full desktop dims from
            // GetSystemMetrics(0/1); pos (0, 0).
            WindowInitPlan {
                style: WindowStyle::Fullscreen,
                outer_width: self.metrics.desktop_width as u32,
                outer_height: self.metrics.desktop_height as u32,
                outer_x: 0, outer_y: 0,
                coop_level: DDCoopLevel::ExclusiveFullscreen,
                display_mode_width: self.width,
                display_mode_height: self.height,
                display_mode_bpp: 16,   // 5-6-5
                primary_desc: PrimarySurfaceDesc::default(),
                back_desc: want_back.then(|| BackSurfaceDesc::for_dims(self.width, self.height)),
                create_clipper: false,
            }
        } else {
            // Windowed: outer size = w + 2*CXBORDER × h + CYMENU + CYBORDER.
            let outer_w = self.width as i32 + 2 * self.metrics.cx_border;
            let outer_h = self.height as i32 + self.metrics.cy_adornment;
            // Center on desktop.
            let outer_x = (self.metrics.desktop_width - outer_w) / 2;
            let outer_y = (self.metrics.desktop_height - outer_h) / 2;
            WindowInitPlan {
                style: WindowStyle::WindowedInitial,
                outer_width: outer_w as u32,
                outer_height: outer_h as u32,
                outer_x, outer_y,
                coop_level: DDCoopLevel::Normal,
                display_mode_width: self.metrics.desktop_width as u32,
                display_mode_height: self.metrics.desktop_height as u32,
                display_mode_bpp: 16,
                primary_desc: PrimarySurfaceDesc::default(),
                back_desc: want_back.then(|| BackSurfaceDesc::for_dims(self.width, self.height)),
                create_clipper: true,     // clipper only in windowed mode
            }
        }
    }
}

/// Direct port of `FUN_005CC9B0` — window + DirectDraw teardown.
///
/// In the exe: restores app cursor → destroys custom cursors → releases
/// clipper → RestoreDisplayMode + Release IDirectDraw → hides + destroys
/// window → frees 7 widget-pointer tables (each ~244 slots × 0x14 stride
/// at `0x00ACE234..0xAD6BB9`).
///
/// The Rust port models this as an ordered checklist so the host can
/// execute the equivalent teardown for its platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeardownStep {
    RestoreAppCursor,
    DestroyCustomCursors,
    ReleaseClipper,
    RestoreDisplayModeAndReleaseDirectDraw,
    HideAndDestroyWindow,
    FreeWidgetPointerTables,
}

/// Exact ordered checklist of steps `FUN_005CC9B0` executes.
pub const TEARDOWN_ORDER: [TeardownStep; 6] = [
    TeardownStep::RestoreAppCursor,
    TeardownStep::DestroyCustomCursors,
    TeardownStep::ReleaseClipper,
    TeardownStep::RestoreDisplayModeAndReleaseDirectDraw,
    TeardownStep::HideAndDestroyWindow,
    TeardownStep::FreeWidgetPointerTables,
];

/// The seven widget-pointer-table sweeps `FUN_005CC9B0` runs at the end.
/// Each is a triple of `(start_va, end_va, stride_bytes)`. In the port
/// the host doesn't own raw VAs; this constant is retained so future
/// widget-pool array tests can assert we allocate the same number of
/// slots (each ~244 = `(end - start) / stride`).
pub const WIDGET_TABLE_SWEEPS: [(u32, u32, u32); 7] = [
    (0x00ACE234, 0x00ACF3A1, 0x14),
    (0x00ACF638, 0x00AD07A5, 0x14),
    (0x00AD0A3C, 0x00AD1BA9, 0x14),
    (0x00AD1E40, 0x00AD2FAD, 0x14),
    (0x00AD3244, 0x00AD43B1, 0x14),
    (0x00AD4648, 0x00AD57B5, 0x14),
    (0x00AD5A4C, 0x00AD6BB9, 0x14),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> WindowMetrics {
        WindowMetrics { cx_border: 3, cy_border: 3, cy_adornment: 22,
                        desktop_width: 1024, desktop_height: 768 }
    }

    #[test]
    fn fullscreen_plan_uses_exclusive_coop_and_16bpp() {
        let cfg = WindowInitConfig {
            width: 800, height: 600, flags: 0x2, fullscreen: true,
            title: "CM".into(), metrics: metrics(),
        };
        let p = cfg.plan();
        assert_eq!(p.coop_level, DDCoopLevel::ExclusiveFullscreen);
        assert_eq!(p.display_mode_bpp, 16);
        assert_eq!(p.display_mode_width, 800);
        assert_eq!(p.outer_x, 0);
        assert_eq!(p.outer_y, 0);
        assert!(!p.create_clipper);
    }

    #[test]
    fn windowed_plan_adds_border_metrics_and_centers() {
        let cfg = WindowInitConfig {
            width: 800, height: 600, flags: 0x2, fullscreen: false,
            title: "CM".into(), metrics: metrics(),
        };
        let p = cfg.plan();
        assert_eq!(p.coop_level, DDCoopLevel::Normal);
        // Windowed: outer_w = 800 + 2*3 = 806; outer_h = 600 + 22 = 622
        assert_eq!(p.outer_width, 806);
        assert_eq!(p.outer_height, 622);
        // Centered
        assert_eq!(p.outer_x, (1024 - 806) / 2);
        assert_eq!(p.outer_y, (768 - 622) / 2);
        assert!(p.create_clipper);
    }

    #[test]
    fn back_desc_omitted_when_flags_bit_2_clear() {
        let cfg = WindowInitConfig {
            width: 800, height: 600, flags: 0, fullscreen: true,
            title: "CM".into(), metrics: metrics(),
        };
        assert!(cfg.plan().back_desc.is_none());
    }

    #[test]
    fn primary_desc_exact_bytes_match_exe_literals() {
        let d = PrimarySurfaceDesc::default();
        assert_eq!(d.dw_size, 0x6C);
        assert_eq!(d.dw_flags, 1);
        assert_eq!(d.dw_caps, 0x200);
    }

    #[test]
    fn back_desc_exact_bytes_match_exe_literals() {
        let d = BackSurfaceDesc::for_dims(800, 600);
        assert_eq!(d.dw_size, 0x6C);
        assert_eq!(d.dw_flags, 7);
        assert_eq!(d.dw_caps, 0x840);
        assert_eq!(d.dw_width, 800);
        assert_eq!(d.dw_height, 600);
    }

    #[test]
    fn teardown_order_matches_exe() {
        assert_eq!(TEARDOWN_ORDER.len(), 6);
        assert_eq!(TEARDOWN_ORDER[0], TeardownStep::RestoreAppCursor);
        assert_eq!(TEARDOWN_ORDER[3],
                   TeardownStep::RestoreDisplayModeAndReleaseDirectDraw);
    }

    #[test]
    fn seven_widget_tables_each_hold_about_223_slots() {
        // (end - start) / stride ≈ 223 for each — verified against exe.
        // Actual range varies slightly between sweeps due to alignment.
        for (start, end, stride) in WIDGET_TABLE_SWEEPS {
            let slots = (end - start) / stride;
            assert!((220..=230).contains(&slots),
                    "sweep {start:#x}..{end:#x} → {slots} slots (want ~223)");
        }
    }

    #[test]
    fn windowed_style_word_matches_exe_literal() {
        // Post-modechange windowed uses 0x00CA0000 (WS_CAPTION|WS_SYSMENU).
        assert_eq!(WindowStyle::Windowed as u32, 0x00CA_0000);
        // Fullscreen literal.
        assert_eq!(WindowStyle::Fullscreen as u32, 0x8008_0000);
    }
}
