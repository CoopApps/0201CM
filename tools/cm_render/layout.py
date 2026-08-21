"""Faithful port of area_rebuild_layout_tables (FUN_00403390) -- the engine that turns
a parent area's bounds + per-column/row weight bytes into concrete child rectangles.

Source: D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/00403390.c

The game stores the results as relative offset tables on the area record:
  col left[i]  = area+0x1c  + 4*i        col right[i] = area+0x10c + 4*i
  row top[i]   = area+0x94  + 4*i        row bottom[i]= area+0x184 + 4*i
A child in (col, row) then draws at:
  x0 = area.left + col_left[col] ,  x1 = area.left + col_right[col]
  y0 = area.top  + row_top[row] ,  y1 = area.top  + row_bottom[row]
"""

def _axis(extent, weights, inset):
    """One axis of FUN_00403390: relative (near,far) offset per cell.
    near[0]=inset ; near[i]=far[i-1]+2 ; far[i]=(cumsum_i*extent)//S -1 + inset ;
    far[last]=extent-1+inset."""
    n = len(weights)
    S = sum(weights) or 1
    near = [0] * n
    far = [0] * n
    cum = 0
    for i, w in enumerate(weights):
        cum += w
        near[i] = inset if i == 0 else far[i - 1] + 2
        if i < n - 1:
            far[i] = (cum * extent) // S - 1 + inset
        else:
            far[i] = extent - 1 + inset
    return near, far


def rebuild_layout(area, area_flags, col_weights, row_weights, scrollbar=False):
    """Return absolute (col_left, col_right, row_top, row_bottom) pixel tables.
    `area` = (left, top, right, bottom). Exact port of FUN_00403390's two loops."""
    left, top, right, bottom = area
    # insets: FUN_00403390 lines 51-60
    if area_flags & 1:
        col_inset = 0
        row_inset = 0
    else:
        col_inset = 2
        single_row = len(row_weights) <= 1
        row_inset = 2 if ((area_flags & 0x10) or single_row) else 8
    # horizontal extent E (iVar9); -0x15 if a scrollbar is present
    E = (right - 2 * col_inset - left) + 1
    if scrollbar:
        E -= 0x15
    # vertical extent (iVar1)
    V = (bottom - 2 * row_inset - top) + 1

    cl_rel, cr_rel = _axis(E, col_weights, col_inset)
    rt_rel, rb_rel = _axis(V, row_weights, row_inset)

    col_left = [left + o for o in cl_rel]
    col_right = [left + o for o in cr_rel]
    row_top = [top + o for o in rt_rel]
    row_bottom = [top + o for o in rb_rel]
    return col_left, col_right, row_top, row_bottom
