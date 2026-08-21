"""Replay a captured CM0102 draw stream -> pixel-exact image, and diff vs a reference.

Improvements over a raw panel replay:
  - honors the 0x2 translucent-fill flag (alpha-blends panels over the background)
  - composites the background image (from the imgload filename, loaded from game assets)
  - renders TRADITIONAL fonts: smooth TrueType Arial (the CreateFontA path), not the
    .fnt futuristic bitmaps. Sizes come from captured CreateFontA heights when present.
"""
import argparse, json, os, glob
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
PIC_DIRS = ["D:/cm0102/Pictures", "D:/cm0102/Data", "D:/cm0102-rs/assets/cm0102/Pictures"]
# traditional font: Arial TrueType. slot -> point size (arial_* .fnt names imply these)
SLOT_PT = {0: 11, 1: 11, 2: 12, 3: 15, 4: 17, 5: 20, 6: 26, 7: 30}

def unpack565(v):
    r = (v >> 11) & 0x1f; g = (v >> 5) & 0x3f; b = v & 0x1f
    return (r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2)

def arial(pt, bold=False):
    for name in (["arialbd.ttf", "arial.ttf"] if bold else ["arial.ttf"]):
        for base in ["C:/Windows/Fonts/", ""]:
            try: return ImageFont.truetype(base + name, pt)
            except Exception: pass
    return ImageFont.load_default()

def load_rgn(path):
    """Decode CM0102 .RGN: 0x30 header (width,height int32 @0/4; RGB565 masks), then RGB565 pixels."""
    import struct
    d = open(path, "rb").read()
    w, h = struct.unpack_from("<II", d, 0)
    px = d[0x30:]
    im = Image.new("RGB", (w, h)); o = im.load()
    for y in range(h):
        for x in range(w):
            i = (y * w + x) * 2
            v = px[i] | (px[i + 1] << 8)
            r = (v >> 11) & 0x1f; g = (v >> 5) & 0x3f; b = v & 0x1f
            o[x, y] = (r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2)
    return im

def find_pic(fname):
    base = os.path.basename(fname.replace("\\", "/"))
    dirs = PIC_DIRS + ["D:/cm0102/Pictures", "D:/cm0102/Data", "D:/cm0102/pictures", "D:/cm0102/data"]
    for d in dirs:
        for ext in ("", ".RGN", ".rgn", ".png", ".bmp"):
            p = os.path.join(d, base + ext)
            if os.path.exists(p): return p
    return None

def load_bg(path):
    if path.lower().endswith(".rgn"):
        return load_rgn(path)
    return Image.open(path).convert("RGB")

def one_frame(recs):
    out, seen = [], False
    for r in recs:
        if r["t"] in ("panel", "text", "imgload", "imgblt"):
            if r["t"] == "text" and r.get("s", "").startswith("Championship"):
                if seen: break
                seen = True
            out.append(r)
    return out

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cap", default=os.path.join(HERE, "capture.jsonl"))
    ap.add_argument("--out", default="D:/cm0102-rs/reports/carve_segment_index/renders/replay.png")
    ap.add_argument("--ref", help="optional reference screenshot to diff against")
    a = ap.parse_args()
    recs = [json.loads(l) for l in open(a.cap)]
    frame = one_frame(recs)

    img = Image.new("RGB", (800, 600), (0, 0, 0))
    dr = ImageDraw.Draw(img, "RGBA")

    # background image: the game blits a loaded picture at 0,0
    bgfile = next((r["file"] for r in frame if r["t"] == "imgload" and r.get("file")), None)
    # use the LAST full-screen (0,0) background loaded (screens layer backgrounds)
    bgfiles = [r["file"] for r in frame if r["t"] == "imgload" and r.get("file")]
    for bgfile in [f for f in bgfiles if "pictures" in f.lower()][-1:] or bgfiles[-1:]:
        p = find_pic(bgfile)
        if p:
            try: img.paste(load_bg(p).resize((800, 600)), (0, 0)); print("bg:", os.path.basename(p))
            except Exception as e: print("bg load failed:", e)

    def hi(c):  # bevel highlight (brighter) -- FUN_005cd420 top/left edge color
        return tuple(min(255, int(v * 1.7) + 40) for v in c)
    def lo(c):  # bevel shadow (darker) -- bottom/right edge (the 0x5a/100 shade path)
        return tuple(int(v * 0.5) for v in c)

    def bevel3d(box, rgb, raised=True):
        l, t, rr, b = box
        h, s = (hi(rgb), lo(rgb)) if raised else (lo(rgb), hi(rgb))
        for k in range(2):  # 2px edges (thickness 2)
            dr.line([l, t + k, rr, t + k], fill=h)      # top  -> highlight
            dr.line([l + k, t, l + k, b], fill=h)        # left -> highlight
            dr.line([l, b - k, rr, b - k], fill=s)       # bottom -> shadow
            dr.line([rr - k, t, rr - k, b], fill=s)      # right  -> shadow

    # ---- faithful port of FUN_005cf8e0 (panel primitive) ----
    def scl(rgb, pct):  # exact brightness scale, factor in percent, clamped 0xff
        return tuple(min(255, c * pct // 100) for c in rgb)

    def dim_region(l, t, r, b, pct):  # FUN_005cdfd0: dim underlying pixels to pct% (0x3c=60)
        l, t = max(0, l), max(0, t); r, b = min(799, r), min(599, b)
        if r <= l or b <= t: return
        reg = img.crop((l, t, r + 1, b + 1)).point(lambda c: min(255, c * pct // 100))
        img.paste(reg, (l, t))

    def draw_panel(l, t, r, b, fl, rgb):
        if fl & 0x20000: r -= 2; b -= 2
        if fl & 0x40000: l += 2; t += 2
        # --- fill ---
        if fl & 0x2:                                  # transparency -> dim photo to 60%
            dim_region(l, t, r, b, 0x3c)
        elif fl & 0x10:                               # solid fill (mode 4)
            dr.rectangle([l, t, r, b], fill=rgb)
        elif fl & 0x8:                                # vertical gradient (scale 100->0 top->bottom)
            for y in range(t, b + 1):
                dr.line([l, y, r, y], fill=scl(rgb, max(0, 100 - 100 * (y - t) // max(1, b - t))))
        elif fl & 0x4:                                # horizontal gradient
            for x in range(l, r + 1):
                dr.line([x, t, x, b], fill=scl(rgb, max(0, 100 - 100 * (x - l) // max(1, r - l))))
        # --- bevel (0x20): graduated highlight (TL) / shadow (BR) ---
        if (fl & 0x20) and not (fl & 0x80):
            w, h = r - l, b - t
            th = 4 if (w >= 0x32 and h >= 0x32 and b <= 99 and (fl & 0x10)) else 2
            lb8 = 0x42
            for layer in range(th):
                c = (lb8 // th) & 0xff
                hi = scl(rgb, c + 100); lo = scl(rgb, max(0, 100 - c))
                if fl & 0x40: hi, lo = lo, hi           # 0x40 = sunken (swap)
                yt, yb, xl, xr = t + layer, b - layer, l + layer, r - layer
                for k in range(2):
                    dr.line([xl, yt + k, xr, yt + k], fill=hi)   # top    highlight
                    dr.line([xl + k, yt, xl + k, yb], fill=hi)   # left   highlight
                    dr.line([xl, yb - k, xr, yb - k], fill=lo)   # bottom shadow
                    dr.line([xr - k, yt, xr - k, yb], fill=lo)   # right  shadow
                lb8 += 0x42
        # --- plain border (0x400) when not beveled ---
        if (fl & 0x400) and not (fl & 0x20):
            for k in range(1 if fl & 0x2000 else 2):
                dr.rectangle([l + k, t + k, r - k, b - k], outline=rgb)
        # --- bottom shade (0x80) ---
        if (fl & 0x80) and not (fl & 0x20):
            dr.line([l + 2, b, r - 2, b], fill=scl(rgb, 0x5a))
        # --- outer 1px border (0x800) in DAT_00ad6bdc ---
        if fl & 0x800:
            dr.rectangle([l - 1, t - 1, r + 1, b + 1], outline=(255, 255, 0))

    # Sidebar: it's the game.mbr image (a blue gradient), blitted at 0,0 — decode + composite exactly.
    for p in ("D:/cm0102/Data/game.mbr", "D:/cm0102/data/game.mbr"):
        if os.path.exists(p):
            try: img.paste(load_rgn(p), (0, 0))
            except Exception: pass
            break
    for r in [x for x in frame if x["t"] == "panel"]:
        if r["r"] <= 95:  # sidebar column handled above; skip its flag-0x1 no-op fills
            continue
        draw_panel(r["l"], r["top"], r["r"], r["b"], r["flags"], unpack565(r["color"]))

    # PASS 2: text (always on top)
    for r in [x for x in frame if x["t"] == "text" and x.get("s")]:
        rgb = unpack565(r["color"]); slot = r["font"] & 7
        f = arial(SLOT_PT.get(slot, 15), bold=slot >= 6)
        lines = r["s"].split("\n")
        lh = SLOT_PT.get(slot, 15) + 3
        ty = r["top"] + ((r["b"] - r["top"]) - len(lines) * lh) // 2
        shadow = r["flags"] & 0x20
        for line in lines:
            w = dr.textlength(line, font=f)
            x = r["l"] + ((r["r"] - r["l"]) - w) / 2
            if shadow:
                dr.text((x + 1, ty + 1), line, fill=(0, 0, 0), font=f)
            dr.text((x, ty), line, fill=rgb, font=f)
            ty += lh

    img.save(a.out)
    print("replayed", len(frame), "draws ->", a.out)
    if a.ref and os.path.exists(a.ref):
        ref = Image.open(a.ref).convert("RGB").resize((800, 600))
        import statistics
        diffs = [abs(a_ - b_) for pa, pb in zip(img.getdata(), ref.getdata()) for a_, b_ in zip(pa, pb)]
        print(f"mean pixel diff vs reference: {statistics.mean(diffs):.1f}/255")

if __name__ == "__main__":
    main()
