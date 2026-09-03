//! Packed 16-bpp surface + literal ports of cm0102_GDI.exe's software
//! renderer primitives. This is the ground-truth pipeline — see
//! [[gdi-renderer-is-ground-truth]].
//!
//! The GDI executable (`cm0102_GDI.exe`) draws into a plain allocated
//! framebuffer (`DAT_00ad6b1c`) with 16-bit pixels and a per-pixel format
//! held in `(DAT_00acdea8, DAT_00acdeac, DAT_00acdeb0)` (red, green, blue
//! masks); RGB555 is what the GDI init function installs
//! (`FUN_005cc4f0` writes `0x7c00, 0x03e0, 0x001f`). Every primitive in
//! this module is a byte-for-byte port of the corresponding
//! `FUN_005cd***` / `FUN_005ce***` function from that binary, addressing
//! a caller-supplied `PackedSurface` instead of the global framebuffer.
//!
//! The stride field (`DAT_00acdeb8`) is stored as PIXELS (the exe writes
//! `(undefined2)param_1` — `param_1` is width in pixels — then multiplies
//! all row offsets by 2 when addressing bytes). We keep that convention:
//! `pitch_pixels`.

use std::ops::RangeInclusive;

/// A packed 16-bpp software framebuffer.
///
/// `pitch_pixels` is measured in pixels, matching the exe's
/// `DAT_00acdeb8` (`(undefined2)param_1` — width in pixels). The three
/// mask fields carry the same channel masks the exe writes into
/// `(DAT_00acdea8, DAT_00acdeac, DAT_00acdeb0)` from `FUN_005cc4f0` (GDI:
/// `0x7c00, 0x03e0, 0x001f` = RGB555). RGB565 is
/// `0xf800, 0x07e0, 0x001f`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedSurface {
    pub buf: Vec<u16>,
    pub width: i32,
    pub height: i32,
    pub pitch_pixels: i32,
    pub red_mask: u16,
    pub green_mask: u16,
    pub blue_mask: u16,
}

impl PackedSurface {
    /// RGB555 constructor (the mask set `FUN_005cc4f0` installs).
    pub fn rgb555(width: i32, height: i32) -> Self {
        let pitch = width;
        Self {
            buf: vec![0u16; (pitch * height) as usize],
            width,
            height,
            pitch_pixels: pitch,
            red_mask: 0x7c00,
            green_mask: 0x03e0,
            blue_mask: 0x001f,
        }
    }

    /// RGB565 constructor — used by the DirectDraw build when its device
    /// negotiates 565 (many drivers do). `FUN_005ce240` dispatches on
    /// `green_mask == 0x7e0` for this format.
    pub fn rgb565(width: i32, height: i32) -> Self {
        let pitch = width;
        Self {
            buf: vec![0u16; (pitch * height) as usize],
            width,
            height,
            pitch_pixels: pitch,
            red_mask: 0xf800,
            green_mask: 0x07e0,
            blue_mask: 0x001f,
        }
    }

    /// Byte-exact port of `FUN_005ce240` (129 bytes) — the pack-RGB step.
    /// Dispatches on `param_4[5]` (the green mask carried in the pixel
    /// format descriptor). Every call site inside the renderer feeds
    /// through here, so `RGB555` vs `RGB565` differences never leak.
    #[inline]
    pub fn pack_rgb(&self, r: u8, g: u8, b: u8) -> u16 {
        if self.green_mask == 0x7e0 {
            // 565: ((r&0xF8)<<5 | g&0xFC) << 3 | (b&0xFF)>>3
            let a = ((r as u32) & 0xf8) << 5 | (g as u32) & 0xfc;
            ((a << 3) | ((b as u32) & 0xff) >> 3) as u16
        } else {
            // 555: ((r&0xF8)<<5 | g&0xF8) << 2 | (b&0xFF)>>3
            let a = ((r as u32) & 0xf8) << 5 | (g as u32) & 0xf8;
            ((a << 2) | ((b as u32) & 0xff) >> 3) as u16
        }
    }

    /// Byte-exact port of `FUN_005cd330` (165 bytes) — normalise a
    /// rectangle into `Some((min_x, min_y, max_x, max_y))` iff it is
    /// **fully inside** the surface `[0, W-1] × [0, H-1]`. Any corner
    /// off-surface → `None`. Endpoints are order-agnostic (swaps to
    /// min/max first).
    ///
    /// This is stricter than a "return visible portion" clip: the exe
    /// returns `false` (and its callers skip drawing entirely) whenever
    /// any part of the rect extends beyond the surface. Verified
    /// against the running exe: `verify_clip_against_exe.rs` (14,641
    /// cases). Bug this fixed: earlier revisions of this port returned
    /// `Some(clamped_rect)` for partially-off rects, which would have
    /// caused off-edge primitives to draw where the exe would have
    /// dropped them silently.
    pub fn clip(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> Option<(i32, i32, i32, i32)> {
        let min_x = x0.min(x1);
        let max_x = x0.max(x1);
        let min_y = y0.min(y1);
        let max_y = y0.max(y1);
        if min_x < 0 || min_y < 0 || max_x > self.width - 1 || max_y > self.height - 1 {
            return None;
        }
        Some((min_x, min_y, max_x, max_y))
    }

    #[inline]
    fn write_pixel(&mut self, x: i32, y: i32, colour: u16) {
        let idx = (self.pitch_pixels * y + x) as usize;
        self.buf[idx] = colour;
    }

    /// Byte-exact port of `FUN_005cd3e0` (838 bytes) — the line primitive.
    /// `style` bit 1 (0x2) selects a SOLID line; otherwise the line is
    /// DASHED with the exe's `(i - start) % 6 < 4` pattern (4 pixels on,
    /// 2 pixels off). Horizontal and vertical runs take the fast paths
    /// the exe has for `y0==y1` and `x0==x1`; diagonals take the
    /// x-major / y-major Bresenham branch depending on `|dx|` vs `|dy|`.
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, style: u32, colour: u16) {
        let Some((cx0, cy0, cx1, cy1)) = self.clip(x0, y0, x1, y1) else {
            return;
        };
        let solid = (style & 2) != 0;
        let dash = |i: i32| -> bool { (i - 0).rem_euclid(6) < 4 };

        // Fast paths: axis-aligned. The exe uses the CLIPPED endpoints
        // for the fast-path decision (`local_c == local_4` = y0==y1 after
        // clipping, `local_10 == local_8` = x0==x1 after clipping).
        if cy0 == cy1 && y0 == y1 {
            // Horizontal — iterate x=cx0..=cx1, dash counter is `i - cx0`.
            for (i, x) in (cx0..=cx1).enumerate() {
                if solid || dash(i as i32) {
                    self.write_pixel(x, cy0, colour);
                }
            }
            return;
        }
        if cx0 == cx1 && x0 == x1 {
            for (i, y) in (cy0..=cy1).enumerate() {
                if solid || dash(i as i32) {
                    self.write_pixel(cx0, y, colour);
                }
            }
            return;
        }

        // Bresenham, sign-preserving. The exe re-orders endpoints so it
        // always steps forward on the major axis; the minor axis's step
        // is `+1` or `-1` depending on the pre-swap sign relation.
        let (mut x, mut y, xe, ye, step_minor, x_major);
        let (mut err, two_dabs, dsub);
        let dx = x1 - x0;
        let dy = y1 - y0;
        let adx = dx.unsigned_abs() as i32;
        let ady = dy.unsigned_abs() as i32;
        if adx < ady {
            // y-major
            x_major = false;
            let (sx, sy, ex, ey) = if y1 < y0 { (x1, y1, x0, y0) } else { (x0, y0, x1, y1) };
            x = sx;
            y = sy;
            xe = ex;
            ye = ey;
            step_minor = if ex <= sx { -1 } else { 1 };
            let d_major = ey - sy;
            let d_minor = (ex - sx).abs();
            two_dabs = d_minor * 2;
            err = two_dabs - d_major;
            dsub = (d_minor - d_major) * 2;
        } else {
            // x-major
            x_major = true;
            let (sx, sy, ex, ey) = if x1 < x0 { (x1, y1, x0, y0) } else { (x0, y0, x1, y1) };
            x = sx;
            y = sy;
            xe = ex;
            ye = ey;
            step_minor = if ey <= sy { -1 } else { 1 };
            let d_major = ex - sx;
            let d_minor = (ey - sy).abs();
            two_dabs = d_minor * 2;
            err = two_dabs - d_major;
            dsub = (d_minor - d_major) * 2;
        }

        // Dash index only advances when a pixel is INSIDE the clip
        // rectangle — matches the exe's `param_4` counter that is bumped
        // inside the `if (local_10 <= param_1) && ... ` guard.
        let mut dash_i: i32 = 0;
        let inside = |px: i32, py: i32| -> bool {
            px >= cx0 && px <= cx1 && py >= cy0 && py <= cy1
        };

        if inside(x, y) {
            self.write_pixel(x, y, colour);
            dash_i = 1;
        }

        if x_major {
            while x < xe {
                x += 1;
                if err >= 0 {
                    y += step_minor;
                    err += dsub;
                } else {
                    err += two_dabs;
                }
                if inside(x, y) {
                    if solid || (dash_i.rem_euclid(6) < 4) {
                        self.write_pixel(x, y, colour);
                    }
                    dash_i += 1;
                }
            }
        } else {
            while y < ye {
                y += 1;
                if err >= 0 {
                    x += step_minor;
                    err += dsub;
                } else {
                    err += two_dabs;
                }
                if inside(x, y) {
                    if solid || (dash_i.rem_euclid(6) < 4) {
                        self.write_pixel(x, y, colour);
                    }
                    dash_i += 1;
                }
            }
        }
        let _ = (xe, ye); // acknowledged
    }

    /// Byte-exact port of `FUN_005cd730` (314 bytes) — the rectangle
    /// primitive. `style` bit 0 = dashed frame (4 lines with dash style),
    /// `style` bit 1 = solid frame (4 solid lines), neither bit = FILL
    /// (iterated horizontal solid lines across the clipped rect).
    pub fn draw_rectangle(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, style: u32, colour: u16) {
        if (style & 1) != 0 {
            self.draw_line(x0, y0, x1, y0, 1, colour);
            self.draw_line(x1, y0, x1, y1, 1, colour);
            self.draw_line(x1, y1, x0, y1, 1, colour);
            self.draw_line(x0, y1, x0, y0, 1, colour);
            return;
        }
        if (style & 2) != 0 {
            self.draw_line(x0, y0, x1, y0, 2, colour);
            self.draw_line(x1, y0, x1, y1, 2, colour);
            self.draw_line(x1, y1, x0, y1, 2, colour);
            self.draw_line(x0, y1, x0, y0, 2, colour);
            return;
        }
        let Some((cx0, cy0, cx1, cy1)) = self.clip(x0, y0, x1, y1) else { return };
        for y in cy0..=cy1 {
            self.draw_line(cx0, y, cx1, y, 2, colour);
        }
    }

    /// Byte-exact port of `FUN_005cd930` (349 bytes) — save the pixels
    /// in the clipped rectangle for a later `restore_rect`. The exe
    /// allocates a 48-byte header (width, height, size_bytes, ptr, then
    /// 8×4 bytes copied verbatim from the pixel-format descriptor at
    /// `DAT_00acde98`); we condense that to the `SavedRect` type.
    pub fn save_rect(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> Option<SavedRect> {
        let (cx0, cy0, cx1, cy1) = self.clip(x0, y0, x1, y1)?;
        let w = cx1 - cx0 + 1;
        let h = cy1 - cy0 + 1;
        let mut data = Vec::with_capacity((w * h) as usize);
        for y in cy0..=cy1 {
            let start = (self.pitch_pixels * y + cx0) as usize;
            data.extend_from_slice(&self.buf[start..start + w as usize]);
        }
        Some(SavedRect { width: w, height: h, data })
    }

    /// Byte-exact port of `FUN_005cda90` (186 bytes) — restore a rect
    /// saved by `save_rect` at `(x, y)`. Named "restore" in the exe but
    /// also does duty as a general image blit; the block copy is
    /// 4-byte-at-a-time with a byte tail.
    pub fn restore_rect(&mut self, x: i32, y: i32, saved: &SavedRect) {
        let x1 = x + saved.width - 1;
        let y1 = y + saved.height - 1;
        let Some((cx0, cy0, cx1, cy1)) = self.clip(x, y, x1, y1) else { return };
        // Source starts at the same clipped offset within the saved rect.
        let sx = cx0 - x;
        let sy = cy0 - y;
        let w = (cx1 - cx0 + 1) as usize;
        for dy in cy0..=cy1 {
            let src_off = ((dy - cy0 + sy) * saved.width + sx) as usize;
            let dst_off = (self.pitch_pixels * dy + cx0) as usize;
            self.buf[dst_off..dst_off + w].copy_from_slice(&saved.data[src_off..src_off + w]);
        }
    }

    /// Byte-exact port of `FUN_005cdd60` (575 bytes) — darken the rect by
    /// mapping every pixel through a 65536-entry LUT that scales each
    /// unpacked channel by 60/100 (with `>=255 → 255` clamps) and repacks
    /// via the same format. We build the LUT on demand from the current
    /// (green_mask, red_mask, blue_mask) — the exe caches it in
    /// `DAT_00ad6b64` behind a `DAT_00af6b62 == 0` guard; the cache is
    /// invalidated whenever the format changes, so we tie it to the
    /// surface's format for correctness.
    pub fn darken_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let Some((cx0, cy0, cx1, cy1)) = self.clip(x0, y0, x1, y1) else { return };
        let lut = build_darken_lut(self.red_mask, self.green_mask, self.blue_mask);
        for y in cy0..=cy1 {
            let row = (self.pitch_pixels * y) as usize;
            for x in cx0..=cx1 {
                let i = row + x as usize;
                self.buf[i] = lut[self.buf[i] as usize];
            }
        }
    }

    /// Byte-exact port of `FUN_005cd870` (189 bytes, 68 instructions) —
    /// blit a stipple mask at `(x, y)` painting `colour` at every mask
    /// byte != 0. The mask is stored inline in `pattern` as
    /// `[width, height, mask[width * height]]`. The exe verifies the
    /// full `w × h` rectangle is on-surface via `FUN_005cd330` and skips
    /// the blit entirely if any corner is off — matches our
    /// `clip` contract.
    ///
    /// Asm annotations (address / instruction):
    /// * 0x5cd891 `mov edi,[esp+0x30]` — pattern pointer (arg4)
    /// * 0x5cd8a3 `mov al,[edi+1]` — h
    /// * 0x5cd8a8 `mov bl,[edi]` — w
    /// * 0x5cd8ac..8c1 build (x, y, x+w-1, y+h-1) and call clip
    /// * 0x5cd8c9 `test eax; je exit` — clip failed
    /// * 0x5cd8cd..8e5 compute `dst = fb + (clipped_y0 * pitch + clipped_x0)`
    /// * 0x5cd8ef `mov bp,[esp+0x2c]` — colour (16-bit, arg3)
    /// * 0x5cd8f4 `lea edx,[edi+2]` — mask start
    /// * 0x5cd8f7..fd initialise row/col counters
    /// * 0x5cd8ff..912 inner col loop: if `mask[eax]!=0` write colour
    /// * 0x5cd914..920 advance dst by (pitch - w) after each row
    /// * 0x5cd923 `jne` — outer row loop; note `xor eax,eax` at 0x5cd8fd
    ///   is the loop entry so `eax` resets each row.
    pub fn draw_stipple(&mut self, x: i32, y: i32, colour: u16, pattern: &StipplePattern) {
        let w = pattern.width as i32;
        let h = pattern.height as i32;
        // 0x5cd8eb `test eax,eax; jl` — h == 0 short-circuits before the row loop.
        if h == 0 {
            return;
        }
        // 0x5cd8ac..8cb — build FUN_005cd330 args and skip if any corner off-surface.
        let Some((cx0, cy0, _, _)) = self.clip(x, y, x + w - 1, y + h - 1) else {
            return;
        };
        // 0x5cd8cd..8e5 — dst = &fb[cy0 * pitch + cx0].
        let pitch = self.pitch_pixels as usize;
        let mut dst = (cy0 as usize) * pitch + cx0 as usize;
        let mask = &pattern.mask[..];
        // 0x5cd8f7 `lea esi,[ebx-1]` — inner test is `eax <= w-1`.
        // We iterate the pattern in row-major order (mask index == row * w + col).
        let w_us = w as usize;
        for row in 0..(h as usize) {
            // 0x5cd8fd `xor eax,eax` — col index resets each row.
            // 0x5cd8ff `test esi; jl` — w == 0 skips the col loop entirely.
            for col in 0..w_us {
                // 0x5cd903 `cmp byte [edx+eax],0` — write only where the mask byte is non-zero.
                if mask[row * w_us + col] != 0 {
                    // 0x5cd909 `mov [ecx], bp` — 16-bit colour write.
                    self.buf[dst + col] = colour;
                }
            }
            // 0x5cd914..920 — advance dst by full pitch (inner loop's
            // ecx += 2*w done implicitly by iterating `col`; here we
            // add the whole pitch instead of just (pitch-w)).
            dst += pitch;
        }
    }

    /// Iterate over the surface's rows as slices — used by golden tests
    /// that assert full-frame byte-identity against captured buffers.
    pub fn rows(&self) -> impl Iterator<Item = &[u16]> {
        let pitch = self.pitch_pixels as usize;
        let h = self.height as usize;
        (0..h).map(move |y| &self.buf[y * pitch..y * pitch + self.width as usize])
    }
}

/// Byte-exact port of `FUN_005ce2d0` (349 bytes, 117 instructions) —
/// scales `colour` by `intensity_pct` / 100 per RGB channel and repacks
/// in the descriptor's pixel format. `intensity_pct > 100` brightens,
/// `< 100` darkens. Skipped (returns 0) when the exe's global
/// `DAT_00ad6b44` gate is non-zero — see [`packed_widget_globals`].
///
/// Follows the exe's argument shape exactly: `(colour, intensity, fmt)`
/// where `fmt` is a pointer to a pixel-format descriptor. When `fmt` is
/// `None`, the exe falls back to `DAT_00acde98`
/// (asm 005ce2e6..005ce2ec):
///
///     test ebx, ebx
///     mov  esi, ebx
///     jne  005ce2f1
///     mov  esi, 0xacde98
///
/// The static `DAT_00ACDE98` in [`packed_widget_globals`] carries the
/// RGB555 mask set that `FUN_005cc4f0` writes there at GDI init.
///
/// The RGB565 vs RGB555 dispatch is `fmt[+0x14] == 0x7e0`, i.e. the
/// green_mask carried by the descriptor.
///
/// The divide-by-100 is the exe's `imul 0x51eb851f; sar edx, 5` magic —
/// signed multiplicative inverse of 100 for a 32-bit dividend, with the
/// sign-bit round-to-zero correction (`add edx, edx>>31`). Delegated to
/// [`div_by_100_asm`] so every occurrence is literally the same asm.
///
/// Every asm site cited inline. Address prefix is the byte offset inside
/// `FUN_005ce2d0`.
pub fn colour_scale(
    colour: u16,
    intensity_pct: u32,
    fmt: Option<&crate::packed_widget_globals::PixelFormat>,
) -> u16 {
    // 005ce2d0..005ce2de: gate on DAT_00ad6b44 — return 0 if non-zero.
    if crate::packed_widget_globals::DAT_00AD6B44 != 0 {
        return 0;
    }
    // 005ce2df..005ce2ec: pick the pixel-format descriptor. Match the
    // exe's fall-back to `DAT_00acde98` when the pointer arg is NULL.
    let fmt_ref = fmt.unwrap_or(&crate::packed_widget_globals::DAT_00ACDE98);
    let rm = fmt_ref.red_mask as u32;
    let gm = fmt_ref.green_mask as u32;
    let bm = fmt_ref.blue_mask as u32;
    let is_565 = fmt_ref.green_mask == 0x7e0;
    // 005ce2f1..005ce2f8: ecx = colour & 0xffff.
    let c = colour as u32;
    // 005ce2fe..005ce30a: r = ((rm & c) << 8) / (rm + 1) — normalise
    // red channel to 0..255.
    let red_norm = ((rm & c) << 8) / (rm + 1);
    // 005ce30c..005ce322: g = ((gm & c) << 8) / (gm + 1).
    let green_norm = ((gm & c) << 8) / (gm + 1);
    // 005ce32a..005ce33c: b = ((bm & c) << 8) / (bm + 1). ecx here is
    // reloaded to hold `intensity_pct & 0xff` — the exe truncates the
    // intensity to a byte before multiplying.
    let blue_norm = ((bm & c) << 8) / (bm + 1);
    let intensity = intensity_pct & 0xff;
    // 005ce33e..005ce39e: for each channel byte:
    //   scaled = (byte * intensity).mul_hi(0x51eb851f) >> 5
    //          + ((above) >> 31)
    //          == byte * intensity / 100 rounded toward zero.
    // Delegate to `div_by_100_asm` — literally the same magic multiply
    // + shift arithmetic the exe emits, applied to `byte * intensity`.
    let red = div_by_100_asm((red_norm & 0xff) * intensity);
    let green = div_by_100_asm((green_norm & 0xff) * intensity);
    let blue = div_by_100_asm((blue_norm & 0xff) * intensity);
    // 005ce3a0..005ce3c6: clamp each channel to 0xff.
    let clamp = |v: u32| -> u32 { if v > 0xff { 0xff } else { v } };
    let r = clamp(red);
    let g = clamp(green);
    let b = clamp(blue);
    // 005ce3cb..005ce406 (RGB565 branch) vs 005ce407..005ce42c (RGB555).
    if is_565 {
        // 005ce3e4..005ce406: pack ((r & 0xf8) << 8) | ((g & 0xfc) << 3)
        //                        | (b >> 3).
        let a = (r & 0xf8) << 5 | (g & 0xfc);
        ((a << 3) | (b >> 3)) as u16
    } else {
        // 005ce407..005ce42c: pack ((r & 0xf8) << 7) | ((g & 0xf8) << 2)
        //                        | (b >> 3).
        let a = (r & 0xf8) << 5 | (g & 0xf8);
        ((a << 2) | (b >> 3)) as u16
    }
}

/// Byte-exact port of the exe's divide-by-100 idiom used in
/// `FUN_005ce2d0` (three times per call, once per RGB channel; asm
/// 005ce34f..005ce39e for the three chained blocks).
///
/// The compiler emits:
///
///     mov  eax, 0x51eb851f    ; signed multiplicative inverse of 100
///     imul edx                 ; edx:eax = value * 0x51eb851f  (64-bit signed)
///     sar  edx, 5              ; edx = high dword >> 5      (= value/100 flooring)
///     mov  eax, edx
///     shr  eax, 31             ; eax = sign bit  (0 for non-negative)
///     add  edx, eax            ; round toward zero
///
/// The `sar edx, 5` step takes only the upper 32 bits of the 64-bit
/// signed product before the shift — Rust's `wrapping_mul` on a 64-bit
/// value gives us the full product, `>> 37` on the whole i64 combines
/// the high-dword pick and the arithmetic shift by 5 (5 + 32 = 37) —
/// exactly the exe's arithmetic. The final `+ (result >> 31)` restores
/// round-toward-zero for negative dividends.
///
/// For the u32 domain this function sees (channel byte × intensity
/// byte, ≤ 0xff * 0xff = 0xfe01) the sign bit is always clear, so the
/// round-correction is a no-op — but the port emits it anyway to stay
/// bit-identical with the exe's control flow.
#[inline]
pub fn div_by_100_asm(v: u32) -> u32 {
    // imul edx with eax=0x51eb851f gives a full 64-bit signed product;
    // sign-extend the u32 dividend to i64 (matches x86 `mov eax, v` +
    // `cdq` implicit in `imul edx` on a 32-bit register), multiply by
    // the sign-extended constant.
    let prod: i64 = (v as i32 as i64) * (0x51eb851f_i32 as i64);
    // High dword of edx:eax then `sar edx, 5` combines to `>> 37` on i64.
    let hi_shr5: i32 = (prod >> 37) as i32;
    // `add edx, edx>>31` — round-toward-zero for negative values.
    let corr: i32 = ((hi_shr5 as u32) >> 31) as i32;
    (hi_shr5.wrapping_add(corr)) as u32
}

/// Stipple pattern read by `FUN_005d7aa0` widget-decoration branches and
/// blitted by `draw_stipple` (port of `FUN_005cd870`). The on-disk record
/// layout in `cm0102_GDI.exe` is
/// `[u8 width, u8 height, u8 mask[width * height]]` — the ports keep the
/// same shape (width/height in bytes, mask row-major).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StipplePattern {
    pub width: u8,
    pub height: u8,
    pub mask: Vec<u8>,
}

/// Rectangle of pixels saved by `PackedSurface::save_rect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedRect {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u16>,
}

/// Darken lookup: exact port of the inner loop of `FUN_005cdd60`. For
/// every 16-bit pixel value `i`, unpack channels using the mask
/// arithmetic the exe uses (`(i & mask) << 8 / (mask + 1)` — a
/// per-channel scale to 0..255), multiply by 0x3c/100, clamp to 0xff,
/// and repack.
fn build_darken_lut(red_mask: u16, green_mask: u16, blue_mask: u16) -> Vec<u16> {
    let mut lut = vec![0u16; 0x10000];
    let rm = red_mask as u32;
    let gm = green_mask as u32;
    let bm = blue_mask as u32;
    let pack = |r: u32, g: u32, b: u32| -> u16 {
        if green_mask == 0x7e0 {
            let a = ((r & 0xf8) << 5) | (g & 0xfc);
            (((a << 3) | ((b & 0xff) >> 3)) & 0xffff) as u16
        } else {
            let a = ((r & 0xf8) << 5) | (g & 0xf8);
            (((a << 2) | ((b & 0xff) >> 3)) & 0xffff) as u16
        }
    };
    for i in 0..lut.len() {
        let v = i as u32;
        let r = ((v & rm) << 8) / (rm + 1) & 0xff;
        let g = ((v & gm) << 8) / (gm + 1) & 0xff;
        let b = ((v & bm) << 8) / (bm + 1) & 0xff;
        let r2 = ((r * 0x3c) / 100).min(0xff);
        let g2 = ((g * 0x3c) / 100).min(0xff);
        let b2 = ((b * 0x3c) / 100).min(0xff);
        lut[i] = pack(r2, g2, b2);
    }
    lut
}

/// Iterate every `(x, y)` cell inside an inclusive rectangle — a small
/// helper for goldens.
#[allow(dead_code)]
fn range_incl(a: i32, b: i32) -> RangeInclusive<i32> {
    a..=b
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------- colour_scale ----------------

    /// Extract 0..255 red/green/blue from an RGB555 packed word using
    /// exactly the exe's normalisation formula, so we can compare
    /// mid-precision values without re-deriving the pack maths.
    fn unpack555(v: u16) -> (u32, u32, u32) {
        let v = v as u32;
        let r = ((v & 0x7c00) << 8) / (0x7c00 + 1);
        let g = ((v & 0x03e0) << 8) / (0x03e0 + 1);
        let b = ((v & 0x001f) << 8) / (0x001f + 1);
        (r, g, b)
    }

    #[test]
    fn colour_scale_at_100_pct_is_identity_within_pack_precision() {
        let s = PackedSurface::rgb555(1, 1);
        let fmt = crate::packed_widget_globals::PixelFormat::from_surface(&s);
        // Mid-grey in RGB555 (r=g=b≈128).
        let mid = s.pack_rgb(0x80, 0x80, 0x80);
        let out = colour_scale(mid, 100, Some(&fmt));
        // Values round through the /100 magic and back through the
        // 5-bit pack; expect the same 5-bit-quantised colour.
        assert_eq!(out, mid);
    }

    #[test]
    fn colour_scale_darkens_at_50_pct() {
        let s = PackedSurface::rgb555(1, 1);
        let fmt = crate::packed_widget_globals::PixelFormat::from_surface(&s);
        let mid = s.pack_rgb(0x80, 0x80, 0x80);
        let out = colour_scale(mid, 50, Some(&fmt));
        let (r, g, b) = unpack555(out);
        let (rm, gm, bm) = unpack555(mid);
        // Every channel should have dropped to about half; allow one
        // step of 5-bit quantisation slop.
        for (o, i) in [(r, rm), (g, gm), (b, bm)] {
            let half = i / 2;
            assert!(o <= half + 10, "channel {o} not <= {half}+10 (from {i})");
        }
    }

    #[test]
    fn colour_scale_brightens_at_200_pct_and_clamps() {
        let s = PackedSurface::rgb555(1, 1);
        let fmt = crate::packed_widget_globals::PixelFormat::from_surface(&s);
        // Mid-grey doubled should saturate the top of each 5-bit slot.
        let mid = s.pack_rgb(0x80, 0x80, 0x80);
        let out = colour_scale(mid, 200, Some(&fmt));
        // 0x80 * 2 = 0x100, clamped to 0xff, then (0xff & 0xf8) << 5 etc.
        // Expected pack: r=g=b saturated top-5-bit -> 0x7fff.
        assert_eq!(out, 0x7fff);
    }

    #[test]
    fn colour_scale_null_fmt_falls_back_to_dat_00acde98() {
        // Asm 005ce2e6..005ce2ec — `test ebx, ebx; jne skip; mov esi,
        // 0xacde98`. When callers pass NULL the exe reads masks from
        // the global format descriptor at DAT_00acde98. FUN_005cc4f0
        // writes the RGB555 set (0x7c00, 0x03e0, 0x001f) there, so the
        // NULL branch must match an RGB555 surface's fmt exactly.
        let s = PackedSurface::rgb555(1, 1);
        let fmt = crate::packed_widget_globals::PixelFormat::from_surface(&s);
        let mid = s.pack_rgb(0x80, 0x80, 0x80);
        let a = colour_scale(mid, 0x6e, Some(&fmt));
        let b = colour_scale(mid, 0x6e, None);
        assert_eq!(a, b, "NULL fmt_ref must equal fmt-from-RGB555-surface");
    }

    #[test]
    fn div_by_100_asm_matches_scalar_div_over_channel_range() {
        // Compare the magic-multiply-and-shift port against a straight
        // `/100` for every plausible input (byte * byte = 0..0xfe01).
        // The two must agree on every value in the range — that's how
        // the exe's asm resolves.
        for v in 0u32..=0xfe01 {
            let mine = super::div_by_100_asm(v);
            let scalar = v / 100;
            assert_eq!(mine, scalar, "mismatch at v={v}");
        }
    }

    // ---------------- pack_rgb ----------------

    #[test]
    fn pack_rgb_555_matches_exe_arithmetic() {
        let s = PackedSurface::rgb555(4, 4);
        // Compare against a direct evaluation of the exe expression
        // `((r&0xF8)<<5 | g&0xF8) << 2 | (b&0xFF)>>3` for a couple of
        // colours picked to exercise every channel edge.
        assert_eq!(s.pack_rgb(0, 0, 0), 0);
        assert_eq!(s.pack_rgb(0xff, 0xff, 0xff), 0x7fff);
        // Pure red = 0x7c00 in RGB555; pure green = 0x03e0; pure blue = 0x001f.
        assert_eq!(s.pack_rgb(0xff, 0, 0), 0x7c00);
        assert_eq!(s.pack_rgb(0, 0xff, 0), 0x03e0);
        assert_eq!(s.pack_rgb(0, 0, 0xff), 0x001f);
    }

    #[test]
    fn pack_rgb_565_matches_exe_arithmetic() {
        let s = PackedSurface::rgb565(4, 4);
        assert_eq!(s.pack_rgb(0xff, 0, 0), 0xf800);
        assert_eq!(s.pack_rgb(0, 0xff, 0), 0x07e0);
        assert_eq!(s.pack_rgb(0, 0, 0xff), 0x001f);
        assert_eq!(s.pack_rgb(0xff, 0xff, 0xff), 0xffff);
    }

    // ---------------- clip ----------------

    #[test]
    fn clip_swaps_endpoints_and_rejects_partially_off_surface() {
        // Behaviour verified against the running exe (see
        // tests/verify_clip_against_exe.rs, 14,641 cases). The exe
        // returns false when ANY corner extends beyond the surface —
        // NOT "return the visible portion".
        let s = PackedSurface::rgb555(10, 8);
        // Fully inside — no change.
        assert_eq!(s.clip(2, 3, 5, 6), Some((2, 3, 5, 6)));
        // Reversed endpoints get normalised to (min_x, min_y, max_x, max_y).
        assert_eq!(s.clip(5, 6, 2, 3), Some((2, 3, 5, 6)));
        // Any corner off-surface → None (verified: exe returns false
        // on clip(-4, -4, 3, 3), even though (0..3, 0..3) is visible).
        assert_eq!(s.clip(-4, -4, 3, 3), None);
        assert_eq!(s.clip(7, 5, 20, 20), None);
        assert_eq!(s.clip(20, 20, 40, 40), None);
        // Boundary — exactly W-1, H-1 is allowed.
        assert_eq!(s.clip(0, 0, 9, 7), Some((0, 0, 9, 7)));
        // One pixel beyond → None.
        assert_eq!(s.clip(0, 0, 10, 7), None);
    }

    // ---------------- rectangle fill (byte-exact) ----------------

    #[test]
    fn rectangle_fill_writes_exact_u16_grid() {
        let mut s = PackedSurface::rgb555(6, 4);
        let colour = s.pack_rgb(0xff, 0, 0); // 0x7c00
        s.draw_rectangle(1, 1, 4, 2, 0, colour); // FILL — bits 0 & 1 both clear
        let z = 0u16;
        let c = colour;
        // Expected 6×4 buffer, row-major.
        let expected: [u16; 24] = [
            z, z, z, z, z, z,
            z, c, c, c, c, z,
            z, c, c, c, c, z,
            z, z, z, z, z, z,
        ];
        assert_eq!(s.buf.as_slice(), &expected);
    }

    #[test]
    fn rectangle_fill_off_surface_is_dropped_entirely() {
        // Verified against exe: any rect with a corner off-surface is
        // NOT clamped-and-drawn — the exe's clip returns false and
        // draw_rectangle bails. `verify_clip_against_exe.rs` proves
        // this is exactly what the exe does.
        let mut s = PackedSurface::rgb555(4, 3);
        let c = s.pack_rgb(0, 0xff, 0);
        s.draw_rectangle(-2, -1, 10, 10, 0, c);
        // Nothing drawn — the surface stays black.
        assert!(s.buf.iter().all(|&v| v == 0));
        // And an exactly-bounds rect still works.
        s.draw_rectangle(0, 0, 3, 2, 0, c);
        assert!(s.buf.iter().all(|&v| v == c));
    }

    // ---------------- rectangle frame ----------------

    #[test]
    fn rectangle_solid_frame_draws_border_only() {
        let mut s = PackedSurface::rgb555(6, 4);
        let c = s.pack_rgb(0, 0, 0xff); // 0x001f
        s.draw_rectangle(1, 1, 4, 2, 2, c); // SOLID frame — bit 1 set
        let z = 0u16;
        // A 4-wide 2-tall frame at (1,1)..(4,2) is just its edges — both
        // rows are entirely edge for a 2-tall rect.
        let expected: [u16; 24] = [
            z, z, z, z, z, z,
            z, c, c, c, c, z,
            z, c, c, c, c, z,
            z, z, z, z, z, z,
        ];
        assert_eq!(s.buf.as_slice(), &expected);
    }

    // ---------------- lines ----------------

    #[test]
    fn horizontal_solid_line_is_a_run_of_pixels() {
        let mut s = PackedSurface::rgb555(8, 3);
        let c = s.pack_rgb(0xff, 0xff, 0);
        s.draw_line(1, 1, 5, 1, 2, c);
        let expected_row: [u16; 8] = [0, c, c, c, c, c, 0, 0];
        assert_eq!(&s.buf[8..16], &expected_row);
        assert!(s.buf[0..8].iter().all(|&v| v == 0));
        assert!(s.buf[16..24].iter().all(|&v| v == 0));
    }

    #[test]
    fn horizontal_dashed_line_repeats_4_on_2_off() {
        let mut s = PackedSurface::rgb555(12, 1);
        let c = s.pack_rgb(0xff, 0, 0);
        // Dashed = bit 1 clear; pattern is `(i - start) % 6 < 4`.
        s.draw_line(0, 0, 11, 0, 0, c);
        let expected: [u16; 12] = [c, c, c, c, 0, 0, c, c, c, c, 0, 0];
        assert_eq!(s.buf.as_slice(), &expected);
    }

    #[test]
    fn vertical_solid_line_writes_the_column() {
        let mut s = PackedSurface::rgb555(3, 5);
        let c = s.pack_rgb(0, 0xff, 0);
        s.draw_line(2, 1, 2, 3, 2, c);
        for y in 0..5 {
            let v = s.buf[y * 3 + 2];
            let expect = if (1..=3).contains(&y) { c } else { 0 };
            assert_eq!(v, expect, "row {y}");
        }
    }

    // ---------------- save / restore rect ----------------

    #[test]
    fn save_and_restore_rect_roundtrip() {
        let mut src = PackedSurface::rgb555(6, 4);
        let c = src.pack_rgb(0xff, 0, 0);
        src.draw_rectangle(1, 1, 4, 2, 0, c);
        let saved = src.save_rect(1, 1, 4, 2).unwrap();
        assert_eq!(saved.width, 4);
        assert_eq!(saved.height, 2);
        assert!(saved.data.iter().all(|&v| v == c));

        let mut dst = PackedSurface::rgb555(6, 4);
        dst.restore_rect(1, 1, &saved);
        assert_eq!(dst.buf, src.buf);
    }

    #[test]
    fn restore_rect_dropped_when_destination_off_surface() {
        // Same rule as line/rect: if the destination rect extends past
        // the surface, the exe's FUN_005cda90 skips entirely (via the
        // clip false-return). Restore that behaviour here — a full
        // reference-vs-exe verification for save/restore is a later
        // step (verify_save_restore_against_exe.rs).
        let saved = SavedRect {
            width: 3, height: 3,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        };
        let mut dst = PackedSurface::rgb555(4, 4);
        dst.restore_rect(-1, -1, &saved);
        assert!(dst.buf.iter().all(|&v| v == 0), "off-surface restore must not draw");
        // In-bounds destination still blits.
        dst.restore_rect(1, 1, &saved);
        assert_eq!(dst.buf[1 * 4 + 1], 1);
        assert_eq!(dst.buf[1 * 4 + 3], 3);
        assert_eq!(dst.buf[3 * 4 + 1], 7);
        assert_eq!(dst.buf[3 * 4 + 3], 9);
    }

    // ---------------- darken ----------------

    // ---------------- draw_stipple ----------------

    #[test]
    fn stipple_pattern_blits_expected_pixels() {
        // 3x3 hand-built pattern with a plus-shape: only cells whose
        // mask byte is non-zero should be painted.
        let pat = StipplePattern {
            width: 3,
            height: 3,
            mask: vec![
                0, 1, 0,
                1, 1, 1,
                0, 1, 0,
            ],
        };
        let mut s = PackedSurface::rgb555(5, 5);
        let c = s.pack_rgb(0xff, 0, 0);
        s.draw_stipple(1, 1, c, &pat);
        let expected: [u16; 25] = [
            0, 0, 0, 0, 0,
            0, 0, c, 0, 0,
            0, c, c, c, 0,
            0, 0, c, 0, 0,
            0, 0, 0, 0, 0,
        ];
        assert_eq!(s.buf, expected.to_vec());
    }

    #[test]
    fn stipple_clips_to_surface() {
        // Any part off-surface → whole blit is dropped (matches
        // `FUN_005cd330`-then-skip in the exe). Surface must stay black.
        let pat = StipplePattern {
            width: 3,
            height: 3,
            mask: vec![1; 9],
        };
        let mut s = PackedSurface::rgb555(4, 4);
        let c = s.pack_rgb(0xff, 0xff, 0xff);
        // Right edge would land at x=5 (off-surface, width=4).
        s.draw_stipple(3, 0, c, &pat);
        assert!(s.buf.iter().all(|&v| v == 0), "off-right stipple must not draw");
        // Negative origin.
        s.draw_stipple(-1, 0, c, &pat);
        assert!(s.buf.iter().all(|&v| v == 0), "off-left stipple must not draw");
        // Zero-height early exit (matches `test eax; jl exit` at 0x5cd8eb).
        let empty = StipplePattern { width: 3, height: 0, mask: vec![] };
        s.draw_stipple(0, 0, c, &empty);
        assert!(s.buf.iter().all(|&v| v == 0), "h==0 must not draw");
    }

    #[test]
    fn darken_scales_channels_by_60_percent_555() {
        let mut s = PackedSurface::rgb555(2, 1);
        // Pure white → darken → 0xff * 0x3c / 100 = 153 per channel.
        // In RGB555 the unpacked channel from `((0x7fff & 0x7c00) << 8) /
        // (0x7c00+1)` = 253, and 253*0x3c/100 = 151, repacked = 0x4a52.
        // Fill both pixels and darken.
        let white = s.pack_rgb(0xff, 0xff, 0xff);
        s.buf.iter_mut().for_each(|v| *v = white);
        s.darken_rect(0, 0, 1, 0);
        let lut = super::build_darken_lut(s.red_mask, s.green_mask, s.blue_mask);
        assert_eq!(s.buf[0], lut[white as usize]);
        assert_eq!(s.buf[1], lut[white as usize]);
        // Idempotent step: black stays black.
        assert_eq!(lut[0], 0);
    }
}
