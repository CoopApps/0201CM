"""Faithful port of CM0102's panel/rect primitive.

  FUN_005cf8e0  graphics_draw_panel  -> the object background/panel painter
    flag 0x20000 : right/bottom -= 2   (bevel inset)
    flag 0x40000 : left/top    += 2    (bevel inset)
    default fill : FUN_005cd840(l,t,r,b,4,color)      solid rect
    flag 0x400   : FUN_005cd420 rect edges (border, thickness 2 / 1 if 0x2000)
    flag 0x80    : shaded bottom edge = color * 0x5a/100 (90%) at (l+2,b)-(r-2,b)
    flag 8       : vertical gradient (per-row scale) -- NOT ported here
    flag 0x1000  : color sampled from a surface       -- NOT ported here

FUN_005cd840 (solid fill) and FUN_005cd420 (thick line) are raster primitives;
ported directly as fill_rect / edge_line.
"""
from surface import pack_pixel, unpack_pixel

F_INSET_BR = 0x20000
F_INSET_LT = 0x40000
F_BORDER = 0x400
F_SHADE_BOTTOM = 0x80
F_THIN_BORDER = 0x2000


def fill_rect(surf, x0, y0, x1, y1, rgb):
    """FUN_005cd840 solid fill (inclusive bounds, as the game passes them)."""
    p = pack_pixel(rgb[0], rgb[1], rgb[2], surf.gm)
    for y in range(max(0, y0), min(surf.H, y1 + 1)):
        base = y * surf.W
        for x in range(max(0, x0), min(surf.W, x1 + 1)):
            surf.buf[base + x] = p


def edge_line(surf, x0, y0, x1, y1, thick, rgb):
    """FUN_005cd420 thick line (horizontal or vertical)."""
    p = pack_pixel(rgb[0], rgb[1], rgb[2], surf.gm)
    for t in range(thick):
        if y0 == y1:
            yy = y0 + t
            for x in range(max(0, x0), min(surf.W, x1 + 1)):
                if 0 <= yy < surf.H:
                    surf.buf[yy * surf.W + x] = p
        else:
            xx = x0 + t
            for y in range(max(0, y0), min(surf.H, y1 + 1)):
                if 0 <= xx < surf.W:
                    surf.buf[y * surf.W + xx] = p


def _scale(rgb, pct):
    return tuple(min(255, c * pct // 100) for c in rgb)


F_SOLID_FILL = 0x10
F_BEVEL = 0x20
F_SUNKEN = 0x40
F_TRANSPARENT = 0x2   # dim underlying pixels instead of filling (FUN_005cdfd0)


def dim_region(surf, left, top, right, bottom, pct=0x3c):
    """Port of FUN_005cdfd0: scale every underlying pixel to pct% brightness
    (default 0x3c = 60%). Used by the F_TRANSPARENT (0x2) path -- this is the
    translucent row-stripe over the background photo."""
    for y in range(max(0, top), min(surf.H, bottom + 1)):
        base = y * surf.W
        for x in range(max(0, left), min(surf.W, right + 1)):
            r, g, b = unpack_pixel(surf.buf[base + x], surf.gm)
            surf.buf[base + x] = pack_pixel(min(255, r * pct // 100),
                                            min(255, g * pct // 100),
                                            min(255, b * pct // 100), surf.gm)


def _bevel(surf, left, top, right, bottom, flags, rgb):
    """Port of the graduated-bevel path of FUN_005cf8e0 (flag 0x20): per-layer
    highlight (top/left) and shadow (bottom/right), brightness graded by
    cVar = 0x42*layer/thickness. thickness=4 for large top-strip solids, else 2."""
    w, h = right - left, bottom - top
    th = 4 if (w >= 0x32 and h >= 0x32 and bottom <= 99 and (flags & F_SOLID_FILL)) else 2
    lb8 = 0x42
    # FUN_005cf8e0 draws INNERMOST layer first, OUTERMOST last (local_c0 counts th->1 while the
    # inset shrinks to 0), and c = local_b8/th grows each layer -> the strongest highlight/shadow
    # lands on the OUTER edge and wins the 2px overlaps. (Drawing outer-first inverts the bevel.)
    for layer in range(th):
        inset = th - 1 - layer
        c = (lb8 // th) & 0xff
        hi = _scale(rgb, c + 100)
        lo = _scale(rgb, max(0, 100 - c))
        if flags & F_SUNKEN:
            hi, lo = lo, hi
        yt, yb, xl, xr = top + inset, bottom - inset, left + inset, right - inset
        edge_line(surf, xl, yt, xr, yt, 2, hi)      # top    highlight
        edge_line(surf, xl, yt, xl, yb, 2, hi)      # left   highlight
        edge_line(surf, xl, yb, xr, yb, 2, lo)      # bottom shadow
        edge_line(surf, xr, yt, xr, yb, 2, lo)      # right  shadow
        lb8 += 0x42


def draw_panel(surf, left, top, right, bottom, flags, rgb):
    """Port of FUN_005cf8e0. Dispatch on the render flags exactly as the game does."""
    if flags & F_INSET_BR:
        right -= 2; bottom -= 2
    if flags & F_INSET_LT:
        left += 2; top += 2
    # transparency (0x2): dim underlying pixels to 60% instead of filling
    if flags & F_TRANSPARENT:
        dim_region(surf, left, top, right, bottom, 0x3c)
    # solid background fill (0x10, or the default no-flag branch)
    elif flags & F_SOLID_FILL:
        fill_rect(surf, left, top, right, bottom, rgb)
    # graduated bevel (0x20): highlight TL / shadow BR
    if flags & F_BEVEL:
        _bevel(surf, left, top, right, bottom, flags, rgb)
    # plain border edges (flag 0x400)
    if flags & F_BORDER:
        th = 1 if (flags & F_THIN_BORDER) else 2
        edge_line(surf, left, top, right, top, th, rgb)
        edge_line(surf, left, bottom, right, bottom, th, rgb)
        edge_line(surf, left, top, left, bottom, th, rgb)
        edge_line(surf, right, top, right, bottom, th, rgb)
    # shaded bottom bevel (flag 0x80): 90% of color along the bottom edge
    if flags & F_SHADE_BOTTOM:
        edge_line(surf, left + 2, bottom, right - 2, bottom, 1, _scale(rgb, 0x5a))
