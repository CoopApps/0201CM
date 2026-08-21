"""Faithful Setup Game (main menu) render -- the accuracy-loop target.

FIDELITY: faithful.  Chrome geometry/fonts are emulation-derived (banner obj @0x8040cf
= 100,10,790,70 font 7; "Setup Game" title @0x804135 = 100,80,790,125 font 6). Colours
are the code globals (banner red rgb565 0xC000, text 0xe71c, mid-grey 0x8410, border blue).
Rendered via the ported primitives + real .fnt glyphs + real game.mbr. PIL only writes PNG.

Diffs the OPAQUE chrome regions (banner, sidebar, Back/Next bar) against the DirectDraw grab
dd_framebuffer.png -- the photo/button interiors are transparent (random photo) so excluded.
"""
import os, sys, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from surface import Surface, pack_pixel, unpack_pixel
from panel import draw_panel
from text import draw_text_box, F_LEFT, F_SHADOW
from layout import rebuild_layout

RS = "D:/cm0102-rs"
OUT = f"{RS}/reports/carve_segment_index/renders/setup_game.png"
GRAB = f"{RS}/reports/carve_segment_index/renders/dd_framebuffer.png"

RED = unpack_pixel(0xC000)        # banner fill (0,0 -> (198,0,0))
NEARW = unpack_pixel(0xe71c)      # near-white text
YELLOW = unpack_pixel(0xffe0)     # title yellow
MIDGREY = unpack_pixel(0x8410)    # Back/Next
BLUE = (0, 0, 214)                # menu button border

def load_rgn(path):
    d = open(path, "rb").read(); w, h = struct.unpack_from("<II", d, 0); px = d[0x30:]
    return w, h, [px[i*2] | (px[i*2+1] << 8) for i in range(w*h)]

def blit(surf, path, ox, oy):
    w, h, pix = load_rgn(path)
    for y in range(h):
        if oy+y >= surf.H: break
        for x in range(w):
            if ox+x >= surf.W: break
            surf.buf[(oy+y)*surf.W + ox+x] = pix[y*w+x]

def ctext(surf, box, s, slot, rgb, flags=0):
    draw_text_box(surf, box[0], box[1], box[2], box[3], flags, slot, rgb, s)

MENU = ["Start New Game", "Quick Start Game", "Restore Saved Game", "Delete Saved Game",
        "Network Play", "Game Settings", "Hall Of Fame", "Game Credits", "Web Sites"]

def main():
    surf = Surface(); surf.fill(*unpack_pixel(0))
    # backdrop: a real Pictures/*.RGN (game randomises it -- not a fidelity target)
    pics = sorted(p for p in os.listdir("D:/cm0102/Pictures") if p.lower().endswith(".rgn"))
    if pics: blit(surf, f"D:/cm0102/Pictures/{pics[0]}", 0, 0)
    # sidebar image + pre-game menu text
    for p in ("D:/cm0102/Data/game.mbr",):
        if os.path.exists(p): blit(surf, p, 0, 0)
    ctext(surf, (2, 8, 88, 40), "Version\n3.9.60", 1, YELLOW)
    ctext(surf, (2, 150, 88, 185), "Restart\nGame", 1, YELLOW)
    ctext(surf, (2, 195, 88, 220), "Exit\nGame", 1, YELLOW)

    # banner: fill+bevel red, near-white title (obj @0x8040cf: 100,10,790,70 font 7)
    draw_panel(surf, 100, 10, 790, 70, 0x30, RED)
    ctext(surf, (100, 10, 790, 70), "Championship Manager 2001/02", 7, NEARW)
    # "Setup Game" title (obj @0x804135: 100,80,790,125 font 6, transparent -> yellow text)
    ctext(surf, (100, 80, 790, 125), "Setup Game", 6, YELLOW)

    # 9 menu buttons: blue-bordered transparent boxes in a 2-col grid, laid out by the engine
    cl, cr, rt, rb = rebuild_layout((104, 148, 772, 474), 2, [1, 1], [1, 1, 1, 1, 1])
    for i, label in enumerate(MENU[:8]):
        col, row = i % 2, i // 2
        l, t, r, b = cl[col], rt[row], cr[col], rb[row]
        draw_panel(surf, l, t, r, b, 0x2, (0, 0, 0))          # transparency (photo shows through)
        draw_panel(surf, l, t, r, b, 0x400, BLUE)             # blue border
        ctext(surf, (l, t, r, b), label, 4, NEARW)
    # Web Sites -- centred single box on the 5th row
    l, t, r, b = cl[0], rt[4], cr[1], rb[4]
    wl = (l + r) // 2 - 160
    draw_panel(surf, wl, t, wl + 320, b, 0x2, (0, 0, 0))
    draw_panel(surf, wl, t, wl + 320, b, 0x400, BLUE)
    ctext(surf, (wl, t, wl + 320, b), "Web Sites", 4, NEARW)

    # Back / Next grey buttons
    for s, box in [("Back", (100, 552, 446, 588)), ("Next", (452, 552, 798, 588))]:
        draw_panel(surf, box[0], box[1], box[2], box[3], 0x30, MIDGREY)
        ctext(surf, box, s, 6, unpack_pixel(0))

    from PIL import Image
    img = Image.frombytes("RGB", (surf.W, surf.H), surf.to_rgb_bytes())
    img.save(OUT); print("->", OUT)

    # diff opaque chrome regions vs the grab
    if os.path.exists(GRAB):
        import statistics
        grab = Image.open(GRAB).convert("RGB")
        regions = {"banner": (100, 10, 789, 69), "sidebar": (0, 0, 88, 599),
                   "backnext": (100, 552, 798, 588)}
        for name, (x0, y0, x1, y1) in regions.items():
            a = img.crop((x0, y0, x1, y1)); b = grab.crop((x0, y0, x1, y1))
            diffs = [abs(p-q) for pa, pb in zip(a.getdata(), b.getdata()) for p, q in zip(pa, pb)]
            print(f"  {name:9} mean pixel diff vs grab: {statistics.mean(diffs):.1f}/255")

if __name__ == "__main__":
    main()
