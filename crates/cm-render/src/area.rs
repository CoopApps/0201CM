//! The Area layout tree — port of `Area.cpp`
//! (`C:\dev\CM3 00-01\si\code\Area.cpp`, VA `0x00402b00..0x0040428f`).
//! Function decode + names: `reports/carve_rename_map.json` (`Area.cpp`).
//!
//! An `Area` is a rectangular region that subdivides into a grid of weighted
//! cells (rows × columns); each cell can hold a child `Area`, so screens are
//! nested layout trees. The per-cell geometry comes from the already-verified
//! [`crate::layout::rebuild_layout`] (`area_rebuild_layout_tables`,
//! `0x00403390`); this module adds the surrounding class: node construction,
//! recursive re-layout, the screen-bounds clamp, child ordering, and the
//! border-colour packer.
//!
//! Coordinates are the original's `(left, top, right, bottom)` in the 800×600
//! surface; the clamp bounds are the exe's literals `0x31f`=799 (x) and
//! `0x257`=599 (y).

use crate::layout::{rebuild_layout, Layout};

/// Inclusive max x (`0x31f` in `area_clamp_rect_to_bounds`).
pub const CLAMP_MAX_X: i32 = 0x31f;
/// Inclusive max y (`0x257`).
pub const CLAMP_MAX_Y: i32 = 0x257;

/// `(left, top, right, bottom)`.
pub type Rect = (i32, i32, i32, i32);

/// A child area placed in a parent grid cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaChild {
    /// Column index into the parent's `col_weights`.
    pub col: usize,
    /// Row index into the parent's `row_weights`.
    pub row: usize,
    pub area: Area,
}

/// A layout node. Field comments cite the original object offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Area {
    /// `+0x00..0x0c` — absolute bounds.
    pub rect: Rect,
    /// `+0x18` — layout flags (bit `0x1` = no inset; `0x10` = tight rows).
    pub flags: u32,
    /// `+0xbcb` weights, count `+0xbac` — column split.
    pub col_weights: Vec<i32>,
    /// `+0xbad` weights, count `+0xbab` — row split.
    pub row_weights: Vec<i32>,
    /// Whether a scrollbar steals `0x15` px from the content width.
    pub scrollbar: bool,
    /// `+0x20c` — packed border/highlight colour.
    pub border_colour: u16,
    /// Child areas (the `+0x208` pool, addressed here by cell).
    pub children: Vec<AreaChild>,
}

impl Area {
    /// `area_init` (`0x00402b00`): a leaf area with equal 1×1 weighting and no
    /// children. Weight arrays start filled with `1` (the exe memsets them to
    /// `0x01010101`).
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            flags: 0,
            col_weights: vec![1],
            row_weights: vec![1],
            scrollbar: false,
            border_colour: 0,
            children: Vec::new(),
        }
    }

    /// Builder: set the grid split (row and column weights).
    pub fn with_grid(mut self, col_weights: Vec<i32>, row_weights: Vec<i32>) -> Self {
        if !col_weights.is_empty() {
            self.col_weights = col_weights;
        }
        if !row_weights.is_empty() {
            self.row_weights = row_weights;
        }
        self
    }

    /// Builder: set layout flags.
    pub fn with_flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    /// Builder: place a child in cell `(col, row)`.
    pub fn child(mut self, col: usize, row: usize, area: Area) -> Self {
        self.insert_child_sorted(col, row, area);
        self
    }

    /// `area_insert_child_sorted` (`0x00403240`): add a child, keeping the child
    /// list ordered by grid position (row-major) as the exe keeps its `+0x212`
    /// index array sorted by cell extent.
    pub fn insert_child_sorted(&mut self, col: usize, row: usize, area: Area) {
        let key = (row, col);
        let pos = self
            .children
            .iter()
            .position(|c| (c.row, c.col) > key)
            .unwrap_or(self.children.len());
        self.children.insert(pos, AreaChild { col, row, area });
    }

    /// The cell grid for this area's own bounds.
    pub fn layout(&self) -> Layout {
        rebuild_layout(
            self.rect,
            self.flags,
            &self.col_weights,
            &self.row_weights,
            self.scrollbar,
        )
    }

    /// `area_recompute_layout` (`0x00402eb0` + `0x00403390`): assign every child
    /// the absolute rect of its cell, then recurse. Mutates the tree in place so
    /// each node's `rect` is its resolved screen position — exactly what the exe
    /// does when it walks the child pool writing geometry.
    pub fn recompute(&mut self) {
        let grid = self.layout();
        let ncol = self.col_weights.len();
        let nrow = self.row_weights.len();
        for child in &mut self.children {
            if child.col < ncol && child.row < nrow {
                child.area.rect = grid.cell(child.col, child.row);
            }
            child.area.recompute();
        }
    }

    /// Recompute, then collect every node's absolute rect with its tree depth
    /// (0 = this area). Handy for rendering or asserting geometry.
    pub fn flatten(&mut self) -> Vec<(usize, Rect)> {
        self.recompute();
        let mut out = Vec::new();
        self.collect(0, &mut out);
        out
    }

    fn collect(&self, depth: usize, out: &mut Vec<(usize, Rect)>) {
        out.push((depth, self.rect));
        for child in &self.children {
            child.area.collect(depth + 1, out);
        }
    }

    /// `area_clamp_rect_to_bounds` (`0x00403ab0`): translate the area so its
    /// top-left moves toward `(new_left, new_top)`, clamped so the rect stays
    /// within `[0, 0x31f] × [0, 0x257]`. Returns the applied `(dx, dy)`.
    ///
    /// The exe computes `dx = new_left - left`, clamps it low (`dx >= -left`)
    /// then high (`right + dx <= 0x31f`), same for `dy`, and only writes the
    /// rect when a non-zero delta results.
    pub fn move_to_clamped(&mut self, new_left: i32, new_top: i32) -> (i32, i32) {
        let (left, top, right, bottom) = self.rect;
        let mut dx = new_left - left;
        if left + dx < 0 {
            dx = -left;
        }
        if right + dx > CLAMP_MAX_X {
            dx = CLAMP_MAX_X - right;
        }
        let mut dy = new_top - top;
        if top + dy < 0 {
            dy = -top;
        }
        if bottom + dy > CLAMP_MAX_Y {
            dy = CLAMP_MAX_Y - bottom;
        }
        if dx != 0 || dy != 0 {
            self.rect = (left + dx, top + dy, right + dx, bottom + dy);
        }
        (dx, dy)
    }

    /// `area_pack_border_colour` (`0x00404210`): re-pack the stored border
    /// colour into the surface pixel format. `rgb565` selects the 5-6-5 masks
    /// (`0xf800`/`0x07e0`); otherwise the 5-5-5 masks (`0x7c00`/`0x03e0`). The
    /// channels are expanded by `unpack_channels` (the exact `FUN_005ce6e0`
    /// scaling) then re-packed to the surface's RGB565 via [`crate::pack565`]
    /// (the `device_colour_pack_rgb565` path, `0x005ce4f0`).
    pub fn pack_border_colour(&mut self, rgb565: bool) {
        let (r, g, b) = unpack_channels(self.border_colour, rgb565);
        self.border_colour = crate::pack565(r, g, b);
    }
}

/// Expand a packed colour to 8-bit channels. Exact port of `FUN_005ce6e0`
/// (16-bit path, `DAT_00ad6bfc == 0`): each channel is
/// `((mask & colour) << 8) / (mask + 1)` with the field left in place — NOT
/// bit-replication. `rgb565` picks the 5-6-5 masks (`0xf800`/`0x07e0`), else the
/// 5-5-5 masks (`0x7c00`/`0x03e0`); blue is `0x1f` in both.
fn unpack_channels(c: u16, rgb565: bool) -> (u8, u8, u8) {
    let (rmask, gmask): (u32, u32) = if rgb565 {
        (0xf800, 0x07e0)
    } else {
        (0x7c00, 0x03e0)
    };
    let bmask: u32 = 0x1f;
    let c = c as u32;
    let ch = |mask: u32| -> u8 { (((mask & c) << 8) / (mask + 1)) as u8 };
    (ch(rmask), ch(gmask), ch(bmask))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_layout_matches_rebuild_layout() {
        // A single-cell area returns its own bounds (minus insets) as cell 0.
        let mut a = Area::new((110, 80, 780, 545)).with_flags(2);
        a.col_weights = vec![4, 2, 12, 4, 4, 4, 4, 4, 4, 4, 4];
        let grid = a.layout();
        // col inset 2, single-row inset 2 (flags=2 has neither bit 0x1 nor 0x10).
        assert_eq!(grid.cell(0, 0), (112, 82, 164, 543));
    }

    #[test]
    fn nested_children_resolve_to_cells() {
        // Parent split into 2 columns [3,1], flags=1 (no inset), one row.
        // Two children, one per column — the Back/Next nav bar shape.
        let mut root = Area::new((100, 555, 790, 590))
            .with_flags(1)
            .with_grid(vec![3, 1], vec![1])
            .child(0, 0, Area::new((0, 0, 0, 0)))
            .child(1, 0, Area::new((0, 0, 0, 0)));
        let flat = root.flatten();
        // root + 2 children.
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0], (0, (100, 555, 790, 590)));
        // Child rects equal the nav-bar cells locked in layout.rs.
        assert_eq!(flat[1], (1, (100, 555, 617, 590))); // Back  (col 0)
        assert_eq!(flat[2], (1, (619, 555, 790, 590))); // Next  (col 1)
    }

    #[test]
    fn move_to_clamped_stays_on_screen() {
        // Move fully off the top-left: clamps to (0,0,..).
        let mut a = Area::new((100, 100, 200, 200));
        let d = a.move_to_clamped(-50, -50);
        assert_eq!(d, (-100, -100));
        assert_eq!(a.rect, (0, 0, 100, 100));

        // Move past the bottom-right: clamps so right<=799, bottom<=599.
        let mut b = Area::new((100, 100, 200, 200));
        b.move_to_clamped(750, 550);
        assert_eq!(b.rect, (699, 499, CLAMP_MAX_X, CLAMP_MAX_Y));

        // In-bounds move applies exactly.
        let mut c = Area::new((100, 100, 200, 200));
        assert_eq!(c.move_to_clamped(120, 130), (20, 30));
        assert_eq!(c.rect, (120, 130, 220, 230));
    }

    #[test]
    fn border_colour_565_round_trips() {
        // A pure red in 565 (0xf800) stays red through the packer.
        let mut a = Area::new((0, 0, 10, 10));
        a.border_colour = 0xf800;
        a.pack_border_colour(true);
        assert_eq!(a.border_colour, crate::pack565(255, 0, 0));
    }

    #[test]
    fn channel_scaling_is_ce6e0_divide_form() {
        // Green field 32 (0x0400): FUN_005ce6e0 gives (0x0400<<8)/(0x07e0+1)=129,
        // where naive bit-replication would give 130 — lock the divide form.
        let (_, g, _) = unpack_channels(0x0400, true);
        assert_eq!(g, 129);
        // Full white: red/green fields (high bits) reach 255, but the blue field
        // sits at the LSB so `(0x1f<<8)/0x20 = 248` — a real FUN_005ce6e0 quirk
        // that bit-replication would have masked.
        assert_eq!(unpack_channels(0xffff, true), (255, 255, 248));
    }

    #[test]
    fn insert_child_sorted_orders_row_major() {
        let mut root = Area::new((0, 0, 100, 100)).with_grid(vec![1, 1], vec![1, 1]);
        root.insert_child_sorted(1, 1, Area::new((0, 0, 0, 0)));
        root.insert_child_sorted(0, 0, Area::new((0, 0, 0, 0)));
        root.insert_child_sorted(1, 0, Area::new((0, 0, 0, 0)));
        let order: Vec<(usize, usize)> = root.children.iter().map(|c| (c.row, c.col)).collect();
        assert_eq!(order, vec![(0, 0), (0, 1), (1, 1)]);
    }
}
