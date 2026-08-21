//! Panel/rect primitive — port of the flat-fill + border path of FUN_005cf8e0
//! (FUN_005cd840 solid fill, FUN_005cd420 edge line).

use crate::surface::{pack565, Surface};

pub fn fill_rect(surf: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, rgb: (u8, u8, u8)) {
    let p = pack565(rgb.0, rgb.1, rgb.2);
    for y in y0..=y1 {
        for x in x0..=x1 {
            surf.set(x, y, p);
        }
    }
}

pub fn border(surf: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, rgb: (u8, u8, u8)) {
    let p = pack565(rgb.0, rgb.1, rgb.2);
    for x in x0..=x1 {
        surf.set(x, y0, p);
        surf.set(x, y1, p);
    }
    for y in y0..=y1 {
        surf.set(x0, y, p);
        surf.set(x1, y, p);
    }
}
