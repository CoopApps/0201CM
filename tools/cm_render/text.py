"""Faithful port of CM0102's text pipeline.

  FUN_005cf610  graphics_text_width   -> sum of .fnt glyph advances (fallback-font path
                                         via FUN_0059bed0; the .fnt files are what we decode)
  FUN_005d0870  graphics_draw_text_box-> box alignment/vertical-centering/shadow, then blits
                                         each line via FUN_005ced50
  FUN_005ced50  graphics_blit_string  -> per-glyph .fnt coverage composite at (x,y)

Font-slot fallback line heights (FUN_005d0870 switch / font metrics lift 0x005cf7b0):
  0,1->0x0f(15)  2->0x12(18)  3->0x15(21)  4->0x18(24)  5->0x1b(27)  6->0x27(39)  7->0x2d(45)
"""
import json
from pathlib import Path

FONT_DIR = Path("D:/cm0102-rs/assets/cm0102/fonts")
SLOT_FILE = {0: "arial_narrow_10", 1: "arial_narrow_10", 2: "arial_narrow_11",
             3: "arial_14", 4: "arial_16", 5: "arial_18",
             6: "trade_cond_24_bold", 7: "trade_cond_28_bold"}
SLOT_LINE_H = {0: 15, 1: 15, 2: 18, 3: 21, 4: 24, 5: 27, 6: 39, 7: 45}

# draw-flag bits used by FUN_005d0870 (param_5)
F_LEFT = 0x1       # left align (else centered)
F_NOVCENTER = 0x2  # top align (else vertical center)
F_SHADOW = 0x20    # draw darkened shadow at +1,+1 first
F_RIGHT = 0x40     # right align

_cache = {}
def load_glyphs(slot):
    name = SLOT_FILE.get(slot, "arial_14")
    if name in _cache:
        return _cache[name]
    d = json.load(open(FONT_DIR / f"{name}.json"))
    g = {gl["codepoint"]: dict(adv=gl["advance"], w=gl["bitmap_width"], lb=gl["left_bearing"],
                               rb=gl["right_bearing"],
                               bm=bytes.fromhex(gl["bitmap_hex"]) if gl.get("bitmap_hex") else b"")
         for gl in d["glyphs"]}
    base = g.get(32, {}).get("adv", 0)   # DAT_00acce20[slot] = space glyph's width (tracking base)
    _cache[name] = (d["height"], g, base)
    return _cache[name]


def _track(prev, cur, base):
    """Port of FUN_005cf840: inter-glyph kern = max(0, base*3//4 - cur.lb - prev.rb)."""
    if prev is None:
        return 0
    return max(0, (base * 3) // 4 - cur["lb"] - prev["rb"])

# NOTE (accuracy loop, Setup Game banner): the game's blit FUN_005ced50 advances the cursor by
# glyph_width + TRACKING, where tracking = FUN_005cf840 =
#     max(0, round(fontbase[slot]*3/4) - cur.field3 - prev.field4)
# a per-adjacent-pair kern. Omitting it makes title fonts (6/7 trade_cond) render ~25% too narrow
# (measured: banner text 357px vs the grab's 473px). To fix: carry the per-glyph metric fields
# (record is 5 ints/0x14B per glyph at 0xaccb9c+slot*0x1404) + fontbase (0xacce20+slot*0x1404)
# from the .fnt into the glyph JSON, then add the tracking here and in _blit_string.

def text_width(slot, s):
    """Width for the .fnt bitmap-font render path: sum of native .fnt glyph
    advances. NOTE: the decoder's `sample_widths` field uses a (height*3//4)
    inter-glyph spacing formula that is WRONG for bitmap rendering (it produces
    absurdly loose tracking); advance-only matches the tight text in real CM0102
    screenshots. The 16.16 fixed-point width fn FUN_0059bed0/FUN_0059c1d0 is a
    separate loaded-metrics mode, not the bitmap path used here."""
    _, g, base = load_glyphs(slot)
    w = 0; prev = None
    for ch in s:
        gl = g.get(ord(ch), g[32])
        w += _track(prev, gl, base) + gl["adv"]; prev = gl
    return w

def _blit_string(surf, x, y, slot, rgb, s):
    """Port of FUN_005ced50: composite each glyph's .fnt coverage in text color.

    The .fnt glyph bitmap is 4-bit coverage packed TWO pixels per byte (high nibble
    = left pixel, low nibble = right pixel); `w` is the byte-width, so the true pixel
    width is 2*w. Each nibble (0..15) scales to 0..255 by *17."""
    from surface import pack_pixel, unpack_pixel
    h, g, base = load_glyphs(slot)
    cx = x; prev = None
    for ch in s:
        gl = g.get(ord(ch), g[32]); w, bm = gl["w"], gl["bm"]
        cx += _track(prev, gl, base); prev = gl
        if w and bm:
            for row in range(h):
                for bcol in range(w):
                    byte = bm[row*w + bcol]
                    if not byte:
                        continue
                    for half in range(2):
                        nib = (byte >> 4) if half == 0 else (byte & 0xf)
                        if not nib:
                            continue
                        cov = nib * 17
                        dx, dy = cx + gl["lb"] + bcol*2 + half, y + row
                        if 0 <= dx < surf.W and 0 <= dy < surf.H:
                            br, bg, bb = unpack_pixel(surf.buf[dy*surf.W+dx], surf.gm)
                            a = cov/255.0
                            surf.set(dx, dy, pack_pixel(int(br+(rgb[0]-br)*a),
                                                        int(bg+(rgb[1]-bg)*a),
                                                        int(bb+(rgb[2]-bb)*a), surf.gm))
        cx += gl["adv"]

def draw_text_box(surf, left, top, right, bottom, flags, slot, rgb, text, shadow_rgb=(0, 0, 0)):
    """Port of FUN_005d0870: place `text` in the box per alignment/centering flags."""
    box_w = (right - left) + 1
    box_h = (bottom - top) + 1
    lh = SLOT_LINE_H.get(slot, 21)
    lines = text.split("\n")
    if flags & F_NOVCENTER:
        y = top
    else:
        y = (box_h - len(lines) * lh) // 2 + top
    for line in lines:
        tw = text_width(slot, line)
        if flags & F_LEFT:
            x = left
        elif flags & F_RIGHT:
            x = (right - tw) + 1
        else:
            x = (box_w - tw) // 2 + left
        if flags & F_SHADOW:
            _blit_string(surf, x + 1, y + 1, slot, shadow_rgb, line)
        _blit_string(surf, x, y, slot, rgb, line)
        y += lh
