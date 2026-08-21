//! 800x600 16-bit surface + color packing.
//! Port of FUN_005ce4f0 (graphics_rgb_to_surface_pixel), verified 16/16 against
//! cm0102.exe's own lifted color globals. Default skin = RGB565 (green mask 0x7e0).

pub const W: usize = 800;
pub const H: usize = 600;

/// FUN_005ce4f0, RGB565 branch: ((r&0xf8)<<5 | g&0xfc)<<3 | (b&0xff)>>3
#[inline]
pub fn pack565(r: u8, g: u8, b: u8) -> u16 {
    ((((r as u32 & 0xf8) << 5 | (g as u32 & 0xfc)) << 3) | (b as u32 & 0xff) >> 3) as u16
}

/// Inverse, expanding 5/6-bit channels to 8-bit by bit-replication.
#[inline]
pub fn unpack565(v: u16) -> (u8, u8, u8) {
    let r5 = (v >> 11) & 0x1f;
    let g6 = (v >> 5) & 0x3f;
    let b5 = v & 0x1f;
    (((r5 << 3) | (r5 >> 2)) as u8, ((g6 << 2) | (g6 >> 4)) as u8, ((b5 << 3) | (b5 >> 2)) as u8)
}

pub struct Surface {
    pub buf: Vec<u16>,
}

impl Surface {
    pub fn new() -> Self {
        Surface { buf: vec![0; W * H] }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, p: u16) {
        if x >= 0 && (x as usize) < W && y >= 0 && (y as usize) < H {
            self.buf[y as usize * W + x as usize] = p;
        }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> u16 {
        if x >= 0 && (x as usize) < W && y >= 0 && (y as usize) < H {
            self.buf[y as usize * W + x as usize]
        } else {
            0
        }
    }

    pub fn fill(&mut self, r: u8, g: u8, b: u8) {
        let p = pack565(r, g, b);
        for v in self.buf.iter_mut() {
            *v = p;
        }
    }

    /// Emit a binary PPM (P6) — dependency-free, converts to PNG for viewing.
    pub fn to_ppm(&self) -> Vec<u8> {
        let mut out = format!("P6\n{} {}\n255\n", W, H).into_bytes();
        for &v in &self.buf {
            let (r, g, b) = unpack565(v);
            out.push(r);
            out.push(g);
            out.push(b);
        }
        out
    }
}
