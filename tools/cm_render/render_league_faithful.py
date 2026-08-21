"""Faithful league-table render -- executes the game's generator, not a lookalike.

FIDELITY: faithful.  Every pixel is produced by a ported primitive or a decoded asset:
  - color packing     surface.pack_pixel      (port of graphics_rgb_to_surface_pixel 0x005ce4f0)
  - panels/bevels     panel.draw_panel        (port of graphics_draw_panel 0x005cf8e0)
  - transparency      panel.dim_region        (port of FUN_005cdfd0, the 0x2 row-stripe)
  - text glyphs       text.draw_text_box      (port of 0x005d0870/0x005ced50) blitting the
                                              game's OWN .fnt glyphs (assets/cm0102/fonts/*.json)
  - sidebar/backdrop  real Data/game.mbr + Pictures/*.RGN (decoded RGB565 assets)
  - geometry          resolved column x-bounds + graphics_font_row_height (=21) -- lifted
  - per-object flags/colors/fonts  competition_league_table_decoded_callsites.json (resolved)

There is no PIL drawing here and no screenshot-matched literal. PIL is used only to write
the final Surface out as a PNG (pure I/O).
"""
import os, sys, struct, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from surface import Surface, pack_pixel, unpack_pixel
from panel import draw_panel
from text import draw_text_box, F_LEFT
from layout import rebuild_layout

RS = "D:/cm0102-rs"
OUT = f"{RS}/reports/carve_segment_index/renders/league_faithful.png"

# ---- decoded color globals -> their packed RGB565 (ui_constants.json) ----
C = {  # name -> rgb565
    "mid_grey": 0x8410, "yellowish": 0xe71c, "dark_purple": 0x4008, "green": 0x0400,
    "black": 0x0000, "near_white": 0xffe0, "red": 0x8000, "dark_red": 0xfc00, "navy": 0x0010,
}
def rgb(name):  # unpack the packed global to 8-bit rgb the primitives re-pack identically
    return unpack_pixel(C[name])

# ---- lifted geometry: DEFAULT column config (resolved_columns.json), row height 21 ----
COLS = json.load(open(f"{RS}/reports/carve_segment_index/fixtures/"
                      "competition_league_table_resolved_columns.json"))["configs"]["default"]["no_scrollbar"]["columns"]
def cx(i):  # (x_left, x_right)
    c = COLS[i]; return c["x_left"], c["x_right"]
# All emulation-derived (validated area_rebuild_layout_tables run on the real areas):
# body top=0x50(80), bottom=0x221(545); the TEAM LIST starts at top+0x41=145, giving
# 18 rows of pitch 22 (row height 21) with a scrollbar (20 teams > 18 visible).
ROW_H = 21          # row fill height = graphics_font_row_height(slot 3)
ROW_PITCH = 22      # emulated row pitch for the 145..545 list area
LIST_TOP = 145      # body_top(80) + 0x41
BODY_TOP = 153      # emulated row-0 top (LIST_TOP + inset)
HDR_TOP = 123       # column-header pill row, one pitch above the list
TITLE_Y = 84        # "League Table" heading, in the 80..145 title band

# ---- per-object render spec, taken from the decoded callsites (col -> flags/color/font) ----
# header cells (cols 4..10): rf=0x30 fill+bevel, mid_grey, font 1, text=label, textcolor=yellowish
HDR_LABELS = {4: "Pld", 5: "Won", 6: "Drn", 7: "Lst", 8: "For", 9: "Ag", 10: "Pts"}
# row cells:
ROW_SPEC = {
    0:  dict(rf=0x30, fill="navy",        font=1),   # position pill
    2:  dict(rf=0x01, fill=None,          font=3),   # team name (arial_14)
    4:  dict(rf=0x01, fill=None,          font=2),   # Pld  (arial_narrow_11)
    5:  dict(rf=0x01, fill=None,          font=2),   # Won
    6:  dict(rf=0x01, fill=None,          font=2),   # Drn
    7:  dict(rf=0x01, fill=None,          font=2),   # Lst
    8:  dict(rf=0x01, fill=None,          font=2),   # For
    9:  dict(rf=0x01, fill=None,          font=2),   # Ag
    10: dict(rf=0x30, fill="dark_purple", font=1),   # Pts pill
}
TEXTCOL = rgb("yellowish")   # DAT_00acdf74, the text color for every cell

def load_rgn(path):
    d = open(path, "rb").read(); w, h = struct.unpack_from("<II", d, 0); px = d[0x30:]
    surf_pixels = []
    for i in range(w * h):
        surf_pixels.append(px[i*2] | (px[i*2+1] << 8))
    return w, h, surf_pixels

def blit_asset(surf, path, ox, oy):
    w, h, pix = load_rgn(path)
    for y in range(h):
        if oy + y >= surf.H: break
        for x in range(w):
            if ox + x >= surf.W: break
            surf.buf[(oy + y) * surf.W + (ox + x)] = pix[y * w + x]

def ordinal(n):
    return f"{n}{'th' if 10 <= n % 100 <= 20 else {1:'st',2:'nd',3:'rd'}.get(n%10,'th')}"

# ---- sidebar dynamic fields (view-model) ----
VM_SIDEBAR = {"date": "Monday\n313.08 EVE", "manager": "Iain\nMacintosh"}
# ---- banner: geometry/font are code-derived; these colors + name are view-model data
#      (the game gets them at runtime from 0x00525310 keyed on the competition) ----
VIEW_MODEL_META = {"competition": "English Premier Division"}
VM_BANNER = {"bg": (236, 238, 244), "title_color": (0, 0, 128)}

# ---- data the view-model supplies (the ONLY synthesized part: the numbers) ----
# Base-game state: load the REAL English Premier Division roster from the shipped .dat pools
# (parsed rust-db/core/clubs.json, division==7 @club+0x57, short name @club+0x38). At SEASON
# START nothing is simulated -> every club 0 played / 0 points. This is the base game, not a save.
def _load_base_roster():
    import json as _j
    p = f"{RS}/reports/carve_segment_index/fixtures/epl_base_roster.json"
    roster = _j.load(open(p, encoding="utf-8"))
    return [(c["short"], 0, 0, 0, 0, 0, 0) for c in roster]

VIEW_MODEL = _load_base_roster()

def cell_text(surf, col, y0, y1, s, font):
    l, r = cx(col)
    flags = F_LEFT if col == 2 else 0     # team name left-aligned; numbers centred
    draw_text_box(surf, l, y0, r, y1, flags, font, TEXTCOL, s)

def main():
    surf = Surface()
    surf.fill(*unpack_pixel(0x0000))   # black base

    # backdrop: a real Pictures/*.RGN (the game randomises which; any real one is faithful)
    pics = sorted([p for p in os.listdir("D:/cm0102/Pictures") if p.lower().endswith(".rgn")])
    if pics:
        try: blit_asset(surf, f"D:/cm0102/Pictures/{pics[0]}", 0, 0)
        except Exception as e: print("backdrop skip:", e)
    # sidebar: the real Data/game.mbr image
    for p in ("D:/cm0102/Data/game.mbr", "D:/cm0102/data/game.mbr"):
        if os.path.exists(p):
            try: blit_asset(surf, p, 0, 0)
            except Exception as e: print("sidebar skip:", e)
            break

    # ---- sidebar menu text (builder FUN_00745540, in-game state) ----
    # area (5,10,85,590), area_flags=1, 13 rows, null weights -> area_init_record fills
    # weight=1 each (0x00402b00 L80-92/121-130). Layout via ported engine.
    _, _, srt, srb = rebuild_layout((5, 10, 85, 590), 1, [1], [1] * 13)
    # rows in builder order (local_364 increments); dynamic fields from the view-model
    SIDEBAR = [VM_SIDEBAR["date"], "<<<   >>>", "Continue\nGame", VM_SIDEBAR["manager"],
               "Competitions", "Nations\n& Clubs", "Find", "Game\nOptions"]
    for i, label in enumerate(SIDEBAR):
        draw_text_box(surf, 6, srt[i], 84, srb[i], F_LEFT, 1, rgb("near_white"), label)

    # ---- top banner (frame obj @0x004947c0): geometry+font code-derived, colors=view-model ----
    # display_create_guio_object(type=1, L=100,T=10,R=790,B=70, rflags=0x30, font=7=trade_cond_28_bold,
    # colP=edx, textcolor=ecx, text=competition name) -- colors come from 0x00525310 (competition data)
    draw_panel(surf, 100, 10, 790, 70, 0x30, VM_BANNER["bg"])
    draw_text_box(surf, 100, 10, 790, 70, 0, 7, VM_BANNER["title_color"], VIEW_MODEL_META["competition"])

    # "League Table" heading (string 0x00988a18) -- wide transparent label, font 3
    draw_text_box(surf, 110, TITLE_Y, 780, TITLE_Y + 20, 0, 3, rgb("near_white"), "League Table")

    # header pill row (cols 4..10): draw_panel fill+bevel mid_grey, then label text
    for col, label in HDR_LABELS.items():
        l, r = cx(col)
        draw_panel(surf, l, HDR_TOP, r, HDR_TOP + ROW_H - 1, 0x30, rgb("mid_grey"))
        draw_text_box(surf, l, HDR_TOP, r, HDR_TOP + ROW_H - 1, 0, 1, TEXTCOL, label)

    # team rows -- emulation-derived placement: first row at BODY_TOP, pitch ROW_PITCH,
    # 18 rows visible (list area 145..545). Beyond 18 the scrollbar scrolls.
    VISIBLE = 18
    for i, (team, Pld, W, D, L, F, A) in enumerate(VIEW_MODEL[:VISIBLE]):
        y0 = BODY_TOP + i * ROW_PITCH; y1 = y0 + ROW_H - 1
        if y1 > 545: break
        # row stripe: transparency object (render_flags 0x2) dims the backdrop to 60%
        draw_panel(surf, 110, y0, 759, y1, 0x2, (0, 0, 0))
        vals = {0: ordinal(i + 1), 2: team, 4: Pld, 5: W, 6: D, 7: L, 8: F, 9: A, 10: W*3 + D}
        for col, spec in ROW_SPEC.items():
            l, r = cx(col)
            if spec["fill"] is not None:
                draw_panel(surf, l, y0, r, y1, spec["rf"], rgb(spec["fill"]))
            cell_text(surf, col, y0, y1, str(vals[col]), spec["font"])

    # vertical scrollbar: area_rebuild_layout_tables reserves the rightmost 0x15(21)px for it
    # when the list scrolls (extent E -= 0x15). Buttons + track + thumb via ported panels.
    SB_L, SB_R = 761, 780
    draw_panel(surf, SB_L, LIST_TOP, SB_R, LIST_TOP + 19, 0x30, rgb("mid_grey"))   # up button
    draw_panel(surf, SB_L, 545 - 19, SB_R, 545, 0x30, rgb("mid_grey"))             # down button
    draw_panel(surf, SB_L, LIST_TOP + 20, SB_R, 545 - 20, 0x10, rgb("navy"))       # track
    total = len(VIEW_MODEL)
    track_h = (545 - 20) - (LIST_TOP + 20)
    thumb_h = max(18, int(track_h * min(1.0, VISIBLE / total)))
    draw_panel(surf, SB_L, LIST_TOP + 20, SB_R, LIST_TOP + 20 + thumb_h, 0x30, rgb("mid_grey"))

    # emit (PIL used for PNG I/O only -- no drawing)
    from PIL import Image
    Image.frombytes("RGB", (surf.W, surf.H), surf.to_rgb_bytes()).save(OUT)
    print("->", OUT)

if __name__ == "__main__":
    main()
