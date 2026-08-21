//! CM0102 image assets: the `.RGN` / `.mbr` format — a 0x30-byte header (width int32 @0,
//! height int32 @4, then RGB565 channel masks) followed by width*height RGB565 pixels.
//! Used for the setup/menu background photos (Pictures/*.RGN, randomised per launch) and
//! any other blitted art. The blit is a straight copy into the RGB565 surface.

use crate::Surface;
use std::path::Path;

pub struct Image {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u16>, // RGB565, row-major
}

impl Image {
    pub fn load_rgn(path: impl AsRef<Path>) -> std::io::Result<Image> {
        let d = std::fs::read(path)?;
        let rd = |o: usize| u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]) as usize;
        let w = rd(0);
        let h = rd(4);
        let body = &d[0x30..];
        let n = (w * h).min(body.len() / 2);
        let px = (0..n).map(|i| u16::from_le_bytes([body[i * 2], body[i * 2 + 1]])).collect();
        Ok(Image { w, h, px })
    }
}

impl Surface {
    /// Blit an RGB565 image at (ox, oy), clipped to the surface.
    pub fn blit_image(&mut self, img: &Image, ox: i32, oy: i32) {
        for y in 0..img.h {
            let dy = oy + y as i32;
            if dy < 0 || dy as usize >= self.h {
                continue;
            }
            let drow = dy as usize * self.w;
            let srow = y * img.w;
            for x in 0..img.w {
                let dx = ox + x as i32;
                if dx < 0 || dx as usize >= self.w {
                    continue;
                }
                self.buf[drow + dx as usize] = img.px[srow + x];
            }
        }
    }
}
