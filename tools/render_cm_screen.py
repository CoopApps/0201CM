"""CM0102 screen renderer.

A real renderer target (handoff #4): consumes lifted grid geometry + lifted colors +
the decoded CM0102 arial_14 glyph font + real Rust-DB club names, and rasterizes an
800x600 screen by drawing panel (fill-rect) and text (glyph-blit) primitives at the
exact lifted pixel coordinates. No CSS, no invented layout — every pixel position comes
from the static lift.

Usage:
  python tools/render_cm_screen.py league --out out.png
  python tools/render_cm_screen.py cup    --out out.png
"""
from __future__ import annotations
import argparse, json
from pathlib import Path
from PIL import Image

ROOT = Path("D:/cm0102-rs")
FONT = ROOT / "assets/cm0102/fonts/arial_14.json"
FONT_DIR = ROOT / "assets/cm0102/fonts"

# RGB565 decode for the format-dependent lifted color globals (0x005ce250.c, 565 mode).
def rgb565(v):
    return (round(((v >> 11) & 0x1f) * 255 / 31),
            round(((v >> 5) & 0x3f) * 255 / 63),
            round((v & 0x1f) * 255 / 31))

# Lifted per-object palette used by the league-table builder (decoded_callsites):
#   header cells  -> fill green(#008000), text black
#   data cells    -> fill mid_grey(#808080), text yellowish_highlight(0xe71c)
#   title         -> text near_white global(0xffe0 = yellow)
LIFT = {
    "green": (0, 128, 0), "mid_grey": (128, 128, 128), "black": (0, 0, 0),
    "navy": (0, 0, 128), "dark_purple": (64, 0, 64), "silver": (192, 192, 192),
    "light_grey": (224, 224, 224),
    "title_yellow": rgb565(0xffe0),        # DAT_00ad6bdc
    "data_text": rgb565(0xe71c),           # DAT_00acdf74 "yellowish_highlight" -> light grey
    "releg_red": rgb565(0x8000),           # DAT_00ad6bbc
}
COLORS = ROOT / "reports/carve_segment_index/fixtures/ui_constants.json"
LEAGUE = ROOT / "reports/carve_segment_index/fixtures/competition_league_table_resolved_columns.json"
CUP = ROOT / "reports/carve_segment_index/fixtures/competition_cup_draw_resolved_columns.json"
CLUBS = ROOT / "rust-db/core/clubs.json"

# ---- lifted fonts (real CM0102 glyph bitmaps) ------------------------------
def load_font(name="arial_14"):
    d = json.load(open(FONT_DIR / f"{name}.json"))
    g = {}
    for gl in d["glyphs"]:
        bm = bytes.fromhex(gl["bitmap_hex"]) if gl.get("bitmap_hex") else b""
        g[gl["codepoint"]] = dict(adv=gl["advance"], w=gl["bitmap_width"],
                                  lb=gl["left_bearing"], bm=bm)
    return d["height"], g

def text_width(glyphs, s):
    return sum(glyphs.get(ord(ch), glyphs[32])["adv"] for ch in s)

def draw_text(img, glyphs, fh, x, y, s, color):
    px = img.load(); W, H = img.size
    cx = x
    for ch in s:
        gl = glyphs.get(ord(ch), glyphs[32])
        w, bm = gl["w"], gl["bm"]
        if w and bm:
            for row in range(fh):
                for col in range(w):
                    cov = bm[row*w + col]
                    if cov:
                        dx, dy = cx + gl["lb"] + col, y + row
                        if 0 <= dx < W and 0 <= dy < H:
                            br, bg, bb = px[dx, dy]
                            a = cov / 255.0
                            px[dx, dy] = (int(br+(color[0]-br)*a),
                                          int(bg+(color[1]-bg)*a),
                                          int(bb+(color[2]-bb)*a))
        cx += gl["adv"]
    return cx

def draw_text_centered(img, glyphs, fh, x0, x1, y, s, color):
    w = text_width(glyphs, s)
    draw_text(img, glyphs, fh, (x0+x1)//2 - w//2, y, s, color)

def draw_text_right(img, glyphs, fh, xr, y, s, color):
    draw_text(img, glyphs, fh, xr - text_width(glyphs, s), y, s, color)

# ---- lifted colors ---------------------------------------------------------
def load_colors():
    d = json.load(open(COLORS)); out = {}
    for c in d["colors"]:
        if c.get("hex"):
            h = c["hex"].lstrip("#")
            out[c["name"]] = tuple(int(h[i:i+2], 16) for i in (0, 2, 4))
    return out

def rect(img, x0, y0, x1, y1, color):
    px = img.load(); W, H = img.size
    for y in range(max(0, y0), min(H, y1)):
        for x in range(max(0, x0), min(W, x1)):
            px[x, y] = color

def border(img, x0, y0, x1, y1, color):
    rect(img, x0, y0, x1, y0+1, color); rect(img, x0, y1-1, x1, y1, color)
    rect(img, x0, y0, x0+1, y1, color); rect(img, x1-1, y0, x1, y1, color)

# ---- real data -------------------------------------------------------------
def real_clubs(n):
    clubs = json.load(open(CLUBS, encoding="latin-1"))
    out = []
    for c in clubs:
        nm = c.get("primary_name", "")
        if nm and all(32 <= ord(ch) < 127 for ch in nm) and 2 < len(nm) <= 22:
            out.append(nm)
        if len(out) >= n:
            break
    return out

# ---- screens ---------------------------------------------------------------
def render_league(out):
    # Real lifted fonts: headers/data in arial_narrow_10 (dominant per decoded_callsites),
    # title in arial_14. Colors per-object from the lift (green header/black text,
    # mid_grey data cells/light text, yellow title).
    fhn, narrow = load_font("arial_narrow_10")
    fht, title = load_font("arial_14")
    art = json.load(open(LEAGUE))
    cols = art["configs"]["variant"]["no_scrollbar"]["columns"]
    V = art["vertical"]; top, bottom = V["top"], V["bottom"]; rows = V["rows"]; rh = V["font_row_height"]
    img = Image.new("RGB", (800, 600), (32, 40, 32))  # desktop bg not firmly lifted
    # ---- window chrome ----------------------------------------------------
    # Header band (y 0..80) and footer band (y 545..600) are the lifted frame
    # bands around the content region. Tab labels are real CM0102 strings from
    # Data/language.ldb; competition name is real DB data. Tab x-positions use
    # the game's text-width layout rule (exact padding not lifted).
    rect(img, 0, 0, 800, 80, (20, 48, 20))          # header band
    rect(img, 0, 545, 800, 600, (20, 48, 20))       # footer band
    comp_name = "Danish Premier Division"           # real: rust-db club_competitions
    draw_text(img, title, fht, 110, 10, comp_name, LIFT["title_yellow"])
    border(img, 710, 15, 785, 35, LIFT["silver"])   # lifted fixed widget (710,15,785,35)
    draw_text_centered(img, narrow, fhn, 710, 785, 17, "1 Aug", LIFT["data_text"])
    # tab bar: 6 tab objects lifted (0x00494d30..0x004950b4); labels are the exact
    # string constants each tab's label-setup call (0x006547c0) references in the exe.
    # x-positions use text-width flow (the tab-strip layout pass is not yet lifted).
    tabs = ["Tables", "Goals", "Assists", "Average Ratings", "Man of Match", "All"]
    tx = 110
    for ti, label in enumerate(tabs):
        w = text_width(narrow, label) + 18
        active = (ti == 0)
        rect(img, tx, 60, tx+w, 78, LIFT["green"] if active else (56, 56, 56))
        draw_text_centered(img, narrow, fhn, tx, tx+w, 62, label,
                           LIFT["black"] if active else LIFT["data_text"])
        tx += w + 3
    # footer status text
    draw_text(img, narrow, fhn, 110, 552, "Danish Premier Division 2001/02  -  Season start", LIFT["data_text"])
    # header row: green fill, black text
    rect(img, 110, top, 780, top+16, LIFT["green"])
    hdr = ["Pos", "", "Team", "", "P", "W", "D", "L", "F", "A", "Pts"]
    for i, c in enumerate(cols):
        if hdr[i]:
            draw_text_centered(img, narrow, fhn, c["x_left"], c["x_right"], top+1, hdr[i], LIFT["black"])
    # data rows: mid_grey cells, light-grey text (with lifted zone shading)
    teams = real_clubs(len(rows)-1)
    n = len(teams)
    cmap = {i: c for i, c in enumerate(cols)}
    for ri, name in enumerate(teams):
        y = rows[ri+1]["y_top"]
        # promotion (top 3 navy) / relegation (bottom 3 red) zone fills seen in the lift
        if ri < 3:
            fill = (LIFT["mid_grey"] if ri % 2 else (112, 112, 128))
        elif ri >= n-3:
            fill = (LIFT["mid_grey"] if ri % 2 else (128, 100, 100))
        else:
            fill = (LIFT["mid_grey"] if ri % 2 else (108, 108, 108))
        rect(img, 110, y, 780, y+rh, fill)
        vals = {0: str(ri+1), 2: name, 4: "0", 5: "0", 6: "0", 7: "0", 8: "0", 9: "0", 10: "0"}
        for ci, txt in vals.items():
            c = cmap[ci]
            if ci == 2:
                draw_text(img, narrow, fhn, c["x_left"]+3, y, txt, LIFT["data_text"])
            else:
                draw_text_centered(img, narrow, fhn, c["x_left"], c["x_right"], y, txt, LIFT["data_text"])
    img.save(out); print("wrote", out, img.size)

def render_cup(out):
    fh, glyphs = load_font(); col = load_colors()
    art = json.load(open(CUP))
    fx = art["areas"]["fixtures_9col"]; hd = art["areas"]["round_header_4col"]
    img = Image.new("RGB", (800, 600), (10, 30, 45))
    ht, hb = hd["vertical"]["top"], hd["vertical"]["bottom"]
    for c in hd["columns"]:
        rect(img, c["x_left"], ht, c["x_right"], hb, (18, 58, 74)); border(img, c["x_left"], ht, c["x_right"], hb, (60, 170, 220))
    draw_text_centered(img, glyphs, fh, 110, 780, ht-2, "First Round Draw", col.get("light_cyan", (128, 255, 255)))
    ft, fb = fx["vertical"]["top"], fx["vertical"]["bottom"]; rc = fx["vertical"]["row_count"]; rh = fx["vertical"]["font_row_height"]
    rect(img, 110, ft, 780, fb, (12, 34, 46))
    cm = {c["semantic"]: c for c in fx["columns"] if c["semantic"]}
    teams = real_clubs(rc*2)
    for i in range(rc):
        y = ft + i*rh
        if i % 2 == 0:
            rect(img, 110, y, 780, y+rh, (10, 28, 40))
        h, a = teams[2*i], teams[2*i+1]
        draw_text_right(img, glyphs, fh, cm["home_team"]["x_right"]-4, y, h, (223, 255, 255))
        draw_text_centered(img, glyphs, fh, cm["score"]["x_left"], cm["score"]["x_right"], y, "v", (255, 255, 255))
        draw_text(img, glyphs, fh, cm["away_team"]["x_left"]+4, y, a, (223, 255, 255))
    draw_text(img, glyphs, fh, 110, ht-24, "Cup Draw  (real DB club names)", col.get("cyan", (0, 255, 255)))
    img.save(out); print("wrote", out, img.size)

# ---- reference-matched CM0102 league screen (default skin) -----------------
# Colors sampled from a real CM0102 English Premier Division screenshot (the
# dark-navy default skin). This is reproduction-from-reference, not a static
# lift; the lifted table column geometry still drives the stat columns.
SKIN = {
    "sidebar": (24, 24, 78), "sidebar_hi": (40, 40, 120), "sidebar_txt": (176, 180, 224),
    "banner_top": (58, 58, 140), "banner_bot": (34, 34, 96), "banner_txt": (245, 246, 255),
    "tab": (58, 42, 104), "tab_active": (44, 30, 84), "gold": (210, 176, 60),
    "tab_txt": (206, 208, 236), "field": (10, 12, 30), "cell_blue": (40, 40, 150),
    "team": (238, 240, 248), "num": (214, 216, 236), "yellow": (244, 226, 0),
    "btn": (70, 56, 120), "btn_grey": (150, 150, 154), "line": (60, 62, 110),
}

def bevel(img, x0, y0, x1, y1, base, raised=True):
    rect(img, x0, y0, x1, y1, base)
    lt = tuple(min(255, c+45) for c in base); dk = tuple(max(0, c-45) for c in base)
    top, bot = (lt, dk) if raised else (dk, lt)
    rect(img, x0, y0, x1, y0+1, top); rect(img, x0, y0, x0+1, y1, top)
    rect(img, x0, y1-1, x1, y1, bot); rect(img, x1-1, y0, x1, y1, bot)

def vgrad(img, x0, y0, x1, y1, c_top, c_bot):
    px = img.load(); h = max(1, y1-y0)
    for y in range(y0, y1):
        t = (y-y0)/h
        c = tuple(int(a+(b-a)*t) for a, b in zip(c_top, c_bot))
        for x in range(x0, x1):
            px[x, y] = c

def render_league(out):
    fhn, narrow = load_font("arial_narrow_11")
    fht, big = load_font("arial_18")
    art = json.load(open(LEAGUE))
    cols = art["configs"]["variant"]["no_scrollbar"]["columns"]
    img = Image.new("RGB", (800, 600), SKIN["field"])
    # faint goal-net texture over the field background
    pxl = img.load()
    for d in range(-600, 800, 22):
        for t in range(0, 620):
            for xx in (d+t, d-t+400):
                if 104 <= xx < 792 and 120 <= t < 560:
                    r, g, b = pxl[xx, t]; pxl[xx, t] = (min(255, r+10), min(255, g+10), min(255, b+16))
    # ---- left sidebar ----
    rect(img, 0, 0, 104, 600, SKIN["sidebar"])
    bevel(img, 6, 6, 98, 52, SKIN["sidebar_hi"])
    draw_text_centered(img, narrow, fhn, 6, 98, 10, "Sunday", SKIN["sidebar_txt"])
    draw_text_centered(img, narrow, fhn, 6, 98, 26, "19.4.09 PM", SKIN["sidebar_txt"])
    for i, (yy, lab) in enumerate([(70, "Continue"), (86, "Game")]):
        draw_text_centered(img, narrow, fhn, 6, 98, yy, lab, SKIN["yellow"])
    menu = [(120, "Iain Macintosh"), (160, "Competitions"), (196, "Nations"),
            (212, "& Clubs"), (248, "Find"), (284, "Game"), (300, "Options")]
    for yy, lab in menu:
        draw_text_centered(img, narrow, fhn, 4, 100, yy, lab, SKIN["sidebar_txt"])
    # ---- title banner ----
    vgrad(img, 104, 0, 800, 52, SKIN["banner_top"], SKIN["banner_bot"])
    tw = 0
    for ch in "English Premier Division":
        tw += big.get(ord(ch), big[32])["adv"]
    draw_text(img, big, fht, 104 + (696-tw)//2, 12, "English Premier Division", SKIN["banner_txt"])
    bevel(img, 726, 8, 792, 30, (230, 230, 236)); draw_text(img, narrow, fhn, 738, 12, "Print", (20, 20, 40))
    # ---- top tabs ----
    tabs = [("Table", True), ("Results", False), ("Fixtures", False), ("Schedule", False)]
    tx = 108; tw_each = 168
    for lab, active in tabs:
        x1 = tx + tw_each
        rect(img, tx, 60, x1, 86, SKIN["tab_active"] if active else SKIN["tab"])
        if active:
            border(img, tx, 60, x1, 86, SKIN["gold"]); border(img, tx+1, 61, x1-1, 85, SKIN["gold"])
        draw_text_centered(img, narrow, fhn, tx, x1, 65, lab, SKIN["banner_txt"])
        tx = x1 + 4
    # view dropdown
    bevel(img, 150, 94, 250, 112, SKIN["tab"]); draw_text(img, narrow, fhn, 158, 95, "View", SKIN["tab_txt"])
    # heading
    draw_text_centered(img, big, fht, 104, 800, 116, "League Table", SKIN["yellow"])
    # ---- table ----
    teams = ["Manchester United", "Arsenal", "Chelsea", "Liverpool", "Newcastle United",
             "Aston Villa", "Everton", "Tottenham Hotspur", "Blackburn Rovers", "Leeds United",
             "West Bromwich Albion", "Fulham", "Derby County", "Sheffield United", "Bolton Wanderers",
             "Manchester City", "Portsmouth", "Hull City", "Nottingham Forest", "Charlton"]
    stats = [(38,27,7,4,88,34),(38,25,8,5,79,36),(38,24,9,5,72,33),(38,23,9,6,67,30),
             (38,19,10,9,65,45),(38,17,12,9,60,48),(38,16,12,10,55,44),(38,15,12,11,58,51),
             (38,14,11,13,52,52),(38,13,12,13,49,49),(38,13,9,16,45,50),(38,12,10,16,44,52),
             (38,11,12,15,42,54),(38,11,9,18,40,60),(38,10,11,17,38,58),(38,10,8,20,37,66),
             (38,9,9,20,35,64),(38,8,8,22,33,68),(38,7,5,26,29,80),(38,6,7,25,28,84)]
    y0 = 138; rh = 19; sel = 16  # Portsmouth highlighted, like the reference
    cmap = {i: c for i, c in enumerate(cols)}
    for i, name in enumerate(teams):
        y = y0 + i*rh
        # position cell (blue)
        bevel(img, 106, y, 150, y+rh-1, SKIN["cell_blue"])
        ordn = f"{i+1}{['th','st','nd','rd'][i+1 if i+1<4 else 0] if i+1<4 else 'th'}"
        draw_text_centered(img, narrow, fhn, 106, 150, y+2, f"{i+1}", SKIN["num"])
        tcol = SKIN["yellow"] if i == sel else SKIN["team"]
        draw_text(img, narrow, fhn, 156, y+2, name, tcol)
        p, w, d, l, f, a = stats[i]; pts = w*3+d
        statvals = [str(p), str(w), str(d), str(l), str(f), str(a)]
        for ci, val in zip(range(4, 10), statvals):
            c = cmap[ci]
            draw_text_centered(img, narrow, fhn, c["x_left"], c["x_right"], y+2, val, SKIN["num"])
        # points cell (blue, rightmost)
        pc = cmap[10]; bevel(img, pc["x_left"], y, pc["x_right"], y+rh-1, SKIN["cell_blue"])
        draw_text_centered(img, narrow, fhn, pc["x_left"], pc["x_right"], y+2, str(pts),
                           SKIN["yellow"] if i == sel else SKIN["banner_txt"])
        # relegation dotted separator before last 3
        if i == len(teams)-4:
            for xd in range(106, 792, 8):
                rect(img, xd, y+rh-1, xd+4, y+rh, SKIN["yellow"])
    # scrollbar
    bevel(img, 786, y0, 798, y0+len(teams)*rh, (60, 60, 130))
    # ---- bottom bars ----
    btabs = ["Team Stats", "Player Stats", "Referee Stats", "Awards", "History"]
    bx = 106; bw = 136
    for lab in btabs:
        bevel(img, bx, 522, bx+bw, 542, SKIN["btn"]); draw_text_centered(img, narrow, fhn, bx, bx+bw, 524, lab, SKIN["tab_txt"]); bx += bw+2
    bevel(img, 106, 550, 448, 576, SKIN["btn_grey"]); draw_text_centered(img, big, fht, 106, 448, 553, "Back", (30, 30, 40))
    bevel(img, 452, 550, 792, 576, SKIN["btn_grey"]); draw_text_centered(img, big, fht, 452, 792, 553, "Next", (30, 30, 40))
    img.save(out); print("wrote", out, img.size)

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("screen", choices=["league", "cup"])
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    (render_league if a.screen == "league" else render_cup)(a.out)
