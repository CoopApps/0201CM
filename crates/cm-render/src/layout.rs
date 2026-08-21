//! Layout engine — faithful port of `area_rebuild_layout_tables` (FUN_00403390).
//!
//! Turns a parent area's bounds + per-column/row weight bytes into concrete child rectangles.
//! This is what positions the league-table columns, the tab strips, the menu grids — every
//! area-laid-out screen. Verified against the emulated original (reproduces the league column
//! geometry to the pixel; see the lock-in test).
//!
//! Per axis (from the decompile): `near[0]=inset`, `near[i]=far[i-1]+2`,
//! `far[i]=(cumsum_i*extent)/S - 1 + inset` for `i<n-1`, `far[last]=extent-1+inset`.
//! Insets: columns `2` (or `0` if area_flags&1); rows `2` if `area_flags&0x10` or single row, else `8`.
//! Extent `= span - 2*inset + 1`, minus `0x15` when a scrollbar is present.

/// One axis: relative (near, far) offsets for each cell.
fn axis(extent: i32, weights: &[i32], inset: i32) -> (Vec<i32>, Vec<i32>) {
    let n = weights.len();
    let s: i32 = weights.iter().sum::<i32>().max(1);
    let mut near = vec![0i32; n];
    let mut far = vec![0i32; n];
    let mut cum = 0i32;
    for i in 0..n {
        cum += weights[i];
        near[i] = if i == 0 { inset } else { far[i - 1] + 2 };
        far[i] = if i + 1 < n { cum * extent / s - 1 + inset } else { extent - 1 + inset };
    }
    (near, far)
}

/// Absolute cell rectangles for an area's children.
pub struct Layout {
    pub col_left: Vec<i32>,
    pub col_right: Vec<i32>,
    pub row_top: Vec<i32>,
    pub row_bottom: Vec<i32>,
}

impl Layout {
    /// Absolute rect of the cell at (col, row): (left, top, right, bottom).
    pub fn cell(&self, col: usize, row: usize) -> (i32, i32, i32, i32) {
        (self.col_left[col], self.row_top[row], self.col_right[col], self.row_bottom[row])
    }
}

/// Port of FUN_00403390. `area` = (left, top, right, bottom).
pub fn rebuild_layout(
    area: (i32, i32, i32, i32),
    area_flags: u32,
    col_weights: &[i32],
    row_weights: &[i32],
    scrollbar: bool,
) -> Layout {
    let (left, top, right, bottom) = area;
    let (col_inset, row_inset) = if area_flags & 1 != 0 {
        (0, 0)
    } else {
        let single_row = row_weights.len() <= 1;
        (2, if area_flags & 0x10 != 0 || single_row { 2 } else { 8 })
    };
    let mut e = right - 2 * col_inset - left + 1;
    if scrollbar {
        e -= 0x15;
    }
    let v = bottom - 2 * row_inset - top + 1;
    let (cl, cr) = axis(e, col_weights, col_inset);
    let (rt, rb) = axis(v, row_weights, row_inset);
    Layout {
        col_left: cl.iter().map(|o| left + o).collect(),
        col_right: cr.iter().map(|o| left + o).collect(),
        row_top: rt.iter().map(|o| top + o).collect(),
        row_bottom: rb.iter().map(|o| top + o).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::rebuild_layout;

    /// Locks the layout engine against the original: the English Premier Division league grid —
    /// area (110,80,780,545), area_flags=2, the VARIANT 11-column weights, a single row — must
    /// resolve to the exact column x-bounds emulated from cm0102.exe.
    #[test]
    fn league_columns_match_original() {
        let l = rebuild_layout((110, 80, 780, 545), 2, &[4, 2, 12, 4, 4, 4, 4, 4, 4, 4, 4], &[1], false);
        let cols: Vec<(i32, i32)> = (0..11).map(|i| (l.col_left[i], l.col_right[i])).collect();
        assert_eq!(
            cols,
            vec![
                (112, 164), (166, 191), (193, 351), (353, 404), (406, 457), (459, 511),
                (513, 564), (566, 617), (619, 671), (673, 724), (726, 778)
            ]
        );
    }

    /// Locks the Setup nav bar against the original (FUN_005d75b0): area (100,555,790,590),
    /// 2 columns weighted [3,1], flags=1, single row. Back is col 0 (wide), Next is col 1 (narrow);
    /// matches the emulated layout-engine output to the pixel.
    #[test]
    fn nav_back_next_match_original() {
        let l = rebuild_layout((100, 555, 790, 590), 1, &[3, 1], &[1], false);
        assert_eq!(l.cell(0, 0), (100, 555, 617, 590)); // Back  (width 518)
        assert_eq!(l.cell(1, 0), (619, 555, 790, 590)); // Next  (width 172)
    }

    /// Locks the pre-game sidebar rows against the original (FUN_00745540): area (5,10,85,590),
    /// 1 column, 13 rows, flags=1. Restart Game is row 3, Exit Game is row 4 (by `local_364`).
    #[test]
    fn sidebar_rows_match_original() {
        let l = rebuild_layout((5, 10, 85, 590), 1, &[1], &[1; 13], false);
        assert_eq!((l.row_top[0], l.row_bottom[0]), (10, 53)); // Version
        assert_eq!((l.row_top[1], l.row_bottom[1]), (55, 98)); // <<< / >>>
        assert_eq!((l.row_top[3], l.row_bottom[3]), (145, 187)); // Restart Game
        assert_eq!((l.row_top[4], l.row_bottom[4]), (189, 232)); // Exit Game
    }
}
