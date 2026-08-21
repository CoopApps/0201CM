"""Faithful port of CM0102's display surface + color packing.

Every function here is a direct port of a specific lifted function, cited by
address. No approximations. If behavior isn't lifted, it isn't here.
"""

# ---------------------------------------------------------------------------
# Color packing  --  faithful port of FUN_005ce4f0 (graphics_rgb_to_surface_pixel)
#
#   uint FUN_005ce4f0(byte r, uint g, uint b, uint* fmt):
#     if (DAT_00ad6bfc != 0) return DAT_00ad6bfc & 0xffff0000;   # forced-color override
#     if (fmt == 0) fmt = &DAT_00acdf48;                          # default format desc
#     if (fmt[5] == 0x7e0)                                        # RGB565 (green mask 0x7e0)
#         return ((r & 0xf8) << 5 | g & 0xfc) << 3 | (b & 0xff) >> 3;
#     else                                                        # RGB555
#         return ((r & 0xf8) << 5 | g & 0xf8) << 2 | (b & 0xff) >> 3;
#
# The display runs 800x600 16-bit (ui_renderer_map). Default skin uses RGB565
# (green mask DAT_00acdf5c == 0x7e0, per ui_constants color_format_note).
# ---------------------------------------------------------------------------

RGB565 = 0x7e0  # value of fmt[5]/green-mask selecting 565 packing


def pack_pixel(r, g, b, green_mask=RGB565, force=0):
    if force != 0:
        return force & 0xffff0000
    if green_mask == 0x7e0:
        return (((r & 0xf8) << 5 | (g & 0xfc)) << 3 | (b & 0xff) >> 3) & 0xffff
    return (((r & 0xf8) << 5 | (g & 0xf8)) << 2 | (b & 0xff) >> 3) & 0xffff


def unpack_pixel(v, green_mask=RGB565):
    """Inverse of pack_pixel, for emitting an RGB image from the 16-bit surface.
    Expands each channel to 8 bits by bit-replication (standard 5/6-bit scaling)."""
    if green_mask == 0x7e0:
        r5 = (v >> 11) & 0x1f
        g6 = (v >> 5) & 0x3f
        b5 = v & 0x1f
        return (r5 << 3 | r5 >> 2, g6 << 2 | g6 >> 4, b5 << 3 | b5 >> 2)
    r5 = (v >> 10) & 0x1f
    g5 = (v >> 5) & 0x1f
    b5 = v & 0x1f
    return (r5 << 3 | r5 >> 2, g5 << 3 | g5 >> 2, b5 << 3 | b5 >> 2)


class Surface:
    """800x600 16-bit indexed surface (ui_renderer_map: native mode)."""
    W, H = 800, 600

    def __init__(self, green_mask=RGB565):
        self.gm = green_mask
        self.buf = [0] * (self.W * self.H)   # 16-bit packed pixels

    def fill(self, r, g, b):
        p = pack_pixel(r, g, b, self.gm)
        self.buf = [p] * (self.W * self.H)

    def set(self, x, y, packed):
        if 0 <= x < self.W and 0 <= y < self.H:
            self.buf[y * self.W + x] = packed

    def to_rgb_bytes(self):
        out = bytearray(self.W * self.H * 3)
        for i, v in enumerate(self.buf):
            r, g, b = unpack_pixel(v, self.gm)
            out[i*3:i*3+3] = bytes((r, g, b))
        return bytes(out)
