//! CM0102 History-screen bitmap (`.hsr`) decoder.
//!
//! `.hsr` files sit in `History/*.hsr` (248 files, all exactly 173,424 bytes = a
//! fixed 258×336 photo canvas) and are referenced from a `.his` article header
//! via a `{pic: <name>.hsr}` tag — the leading picture for a written-history
//! entry (managers, clubs, trophies, championship-final moments).
//!
//! ## Container
//!
//! The file is a raw dump of a DirectDraw surface with a 48-byte header — the
//! **same container** as `Pictures/*.RGN` (`crate::image::Image::load_rgn`).
//! Only the pixel format's channel masks vary from file to file:
//!
//! | off | field                  | notes                                    |
//! |-----|------------------------|------------------------------------------|
//! | +00 | width          `u32`   | 258 for every shipped `.hsr`             |
//! | +04 | height         `u32`   | 336 for every shipped `.hsr`             |
//! | +08 | data size      `u32`   | `w * h * 2` (16bpp pixels)               |
//! | +12 | surface ptr    `u32`   | runtime cache pointer, ignored on load   |
//! | +16 | dd flags       `u32`   | 0x20 = `DDSD_PIXELFORMAT`                |
//! | +20 | pf size        `u32`   | 0x40 — size of `DDPIXELFORMAT` struct    |
//! | +24 | pf flags       `u32`   | 0                                        |
//! | +28 | bit count      `u32`   | 16                                       |
//! | +32 | red mask       `u32`   | 0xF800 (RGB565) or 0x7C00 (XRGB1555)     |
//! | +36 | green mask     `u32`   | 0x07E0 (565) or 0x03E0 (1555)            |
//! | +40 | blue mask      `u32`   | 0x001F (both)                            |
//! | +44 | alpha mask     `u32`   | 0                                        |
//! | +48 | pixels                 | `w * h` little-endian `u16`, row-major   |
//!
//! The two 16bpp encodings are the two the DirectDraw setup path
//! (`FUN_005CE4F0`) targets — same choice the RGB565 vs RGB555 blit picks per
//! surface at runtime. We detect the format from the green mask exactly like
//! the exe: `0x07E0` → RGB565, otherwise RGB555.
//!
//! ## Callers
//!
//! `.hsr` filenames flow through `FUN_005DB8B0` (History.cpp `.his` parser),
//! which stashes the picture name — defaulting to `wcup.hsr` if none is
//! declared — for the History screen (`FUN_005DC660`) to blit. No other
//! subsystem references the extension in strings.json; kits, news thumbnails
//! and competition icons use the sibling `.RGN` container instead.

use std::io;

/// A decoded 32-bit RGBA image (row-major, top-down).
pub struct HsrImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug)]
pub enum HsrError {
    Truncated { got: usize, need: usize },
    UnsupportedBpp(u32),
    UnsupportedMasks { r: u32, g: u32, b: u32 },
}

impl std::fmt::Display for HsrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HsrError::Truncated { got, need } => {
                write!(f, ".hsr truncated: {} bytes, need {}", got, need)
            }
            HsrError::UnsupportedBpp(b) => write!(f, ".hsr unsupported bpp: {}", b),
            HsrError::UnsupportedMasks { r, g, b } => {
                write!(f, ".hsr unsupported channel masks r=0x{:X} g=0x{:X} b=0x{:X}", r, g, b)
            }
        }
    }
}

impl std::error::Error for HsrError {}

impl From<HsrError> for io::Error {
    fn from(e: HsrError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, e)
    }
}

const HEADER_LEN: usize = 48;

#[inline]
fn rd_u32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

/// Decode a `.hsr` blob to 8-bit RGBA. Detects RGB565 vs XRGB1555 from the
/// pixel-format channel masks in the header, then expands each 16-bit pixel
/// with the standard bit-replication (`v<<3 | v>>2` for a 5-bit channel,
/// `v<<2 | v>>4` for the 6-bit green in 565).
pub fn decode_hsr(bytes: &[u8]) -> Result<HsrImage, HsrError> {
    if bytes.len() < HEADER_LEN {
        return Err(HsrError::Truncated { got: bytes.len(), need: HEADER_LEN });
    }
    let width = rd_u32(bytes, 0);
    let height = rd_u32(bytes, 4);
    let bpp = rd_u32(bytes, 28);
    let rmask = rd_u32(bytes, 32);
    let gmask = rd_u32(bytes, 36);
    let bmask = rd_u32(bytes, 40);

    if bpp != 16 {
        return Err(HsrError::UnsupportedBpp(bpp));
    }
    let need = HEADER_LEN + (width as usize) * (height as usize) * 2;
    if bytes.len() < need {
        return Err(HsrError::Truncated { got: bytes.len(), need });
    }

    // Match the exe's own selection (green mask = 0x7E0 → RGB565).
    let is_565 = gmask == 0x07E0 && rmask == 0xF800 && bmask == 0x001F;
    let is_555 = gmask == 0x03E0 && rmask == 0x7C00 && bmask == 0x001F;
    if !is_565 && !is_555 {
        return Err(HsrError::UnsupportedMasks { r: rmask, g: gmask, b: bmask });
    }

    let n = (width as usize) * (height as usize);
    let src = &bytes[HEADER_LEN..HEADER_LEN + n * 2];
    let mut rgba = vec![0u8; n * 4];
    if is_565 {
        for i in 0..n {
            let v = u16::from_le_bytes([src[i * 2], src[i * 2 + 1]]);
            let r5 = ((v >> 11) & 0x1F) as u8;
            let g6 = ((v >> 5) & 0x3F) as u8;
            let b5 = (v & 0x1F) as u8;
            rgba[i * 4] = (r5 << 3) | (r5 >> 2);
            rgba[i * 4 + 1] = (g6 << 2) | (g6 >> 4);
            rgba[i * 4 + 2] = (b5 << 3) | (b5 >> 2);
            rgba[i * 4 + 3] = 0xFF;
        }
    } else {
        for i in 0..n {
            let v = u16::from_le_bytes([src[i * 2], src[i * 2 + 1]]);
            let r5 = ((v >> 10) & 0x1F) as u8;
            let g5 = ((v >> 5) & 0x1F) as u8;
            let b5 = (v & 0x1F) as u8;
            rgba[i * 4] = (r5 << 3) | (r5 >> 2);
            rgba[i * 4 + 1] = (g5 << 3) | (g5 >> 2);
            rgba[i * 4 + 2] = (b5 << 3) | (b5 >> 2);
            rgba[i * 4 + 3] = 0xFF;
        }
    }
    Ok(HsrImage { width, height, rgba })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(w: u32, h: u32, masks: (u32, u32, u32), pixels: &[u16]) -> Vec<u8> {
        let mut b = vec![0u8; HEADER_LEN + pixels.len() * 2];
        b[0..4].copy_from_slice(&w.to_le_bytes());
        b[4..8].copy_from_slice(&h.to_le_bytes());
        b[8..12].copy_from_slice(&((w * h * 2) as u32).to_le_bytes());
        b[16..20].copy_from_slice(&0x20u32.to_le_bytes());
        b[20..24].copy_from_slice(&0x40u32.to_le_bytes());
        b[28..32].copy_from_slice(&16u32.to_le_bytes());
        b[32..36].copy_from_slice(&masks.0.to_le_bytes());
        b[36..40].copy_from_slice(&masks.1.to_le_bytes());
        b[40..44].copy_from_slice(&masks.2.to_le_bytes());
        for (i, v) in pixels.iter().enumerate() {
            b[HEADER_LEN + i * 2..HEADER_LEN + i * 2 + 2].copy_from_slice(&v.to_le_bytes());
        }
        b
    }

    #[test]
    fn rgb565_pure_channels() {
        let px = [0xF800u16, 0x07E0, 0x001F, 0xFFFF];
        let img = decode_hsr(&make(2, 2, (0xF800, 0x07E0, 0x001F), &px)).unwrap();
        assert_eq!((img.width, img.height), (2, 2));
        assert_eq!(&img.rgba[0..4], &[0xFF, 0, 0, 0xFF]);
        assert_eq!(&img.rgba[4..8], &[0, 0xFF, 0, 0xFF]);
        assert_eq!(&img.rgba[8..12], &[0, 0, 0xFF, 0xFF]);
        assert_eq!(&img.rgba[12..16], &[0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn xrgb1555_pure_channels() {
        let px = [0x7C00u16, 0x03E0, 0x001F, 0x7FFF];
        let img = decode_hsr(&make(2, 2, (0x7C00, 0x03E0, 0x001F), &px)).unwrap();
        assert_eq!(&img.rgba[0..4], &[0xFF, 0, 0, 0xFF]);
        assert_eq!(&img.rgba[4..8], &[0, 0xFF, 0, 0xFF]);
        assert_eq!(&img.rgba[8..12], &[0, 0, 0xFF, 0xFF]);
        assert_eq!(&img.rgba[12..16], &[0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn rejects_short_header() {
        let b = vec![0u8; 10];
        assert!(matches!(decode_hsr(&b), Err(HsrError::Truncated { .. })));
    }

    #[test]
    fn rejects_unknown_masks() {
        let b = make(1, 1, (0x1234, 0x5678, 0x9ABC), &[0]);
        assert!(matches!(decode_hsr(&b), Err(HsrError::UnsupportedMasks { .. })));
    }
}
