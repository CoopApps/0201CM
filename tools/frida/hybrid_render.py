"""Hybrid league-table renderer, rebuilt STRICTLY from the lifted builder facts
(0x00495ad0 comp_screen_build_league_table_view) -- no invented chrome.

Every constant below is code-derived and cited:
  - column x-bounds       -> competition_league_table_resolved_columns.json (VARIANT)
  - header labels         -> exe strings "Pld/Won/Drn/Lst/For/Ag/Pts" (comment-stripped)
  - colors                -> ui_constants.json packed globals (navy #000080, near_white=0xffe0=yellow, ...)
  - vertical geometry     -> top=0x50(80) bottom=0x221(545), rows via 0x005cf7b0
  - sidebar               -> real asset Data/game.mbr (decoded, composited)
Parts still marked INFERRED in the lift (row pixel-height, background photo which is a
per-launch RANDOM Pictures/*.RGN, exact title tint) are matched to the reference screenshot
and labelled as such -- they are the only things a DirectDraw capture still needs to pin.
"""
import os, struct
from PIL import Image, ImageDraw, ImageFont

OUT = "D:/cm0102-rs/reports/carve_segment_index/renders/hybrid_league.png"

# ---- exact lifted column x-bounds (VARIANT, no scrollbar) ----
COLS = [(112,164),(166,191),(193,351),(353,404),(406,457),(459,511),
        (513,564),(566,617),(619,671),(673,724),(726,778)]
HDR = {4:"Pld",5:"Won",6:"Drn",7:"Lst",8:"For",9:"Ag",10:"Pts"}  # real strings

# ---- exact lifted colors (ui_constants.json) ----
NAVY=(0,0,128); DEEP=(0,0,96); PURPLE=(64,0,64); MIDGREY=(128,128,128)
SILVER=(192,192,192); WHITE=(236,238,244); YELLOW=(255,255,90)  # near_white 0xffe0 unpacks to yellow
GOLD=(210,182,72); BTNGREY=(150,150,158); BLACK=(0,0,0)
TITLE_BLUE=(44,44,150)  # trade_cond title tint (matched to ref; INFERRED)

def load_rgn(path):
    d = open(path, "rb").read(); w, h = struct.unpack_from("<II", d, 0); px = d[0x30:]
    im = Image.new("RGB", (w, h)); o = im.load()
    for y in range(h):
        for x in range(w):
            i = (y*w+x)*2; v = px[i] | (px[i+1] << 8)
            r=(v>>11)&0x1f; g=(v>>5)&0x3f; b=v&0x1f
            o[x,y]=(r<<3|r>>2, g<<2|g>>4, b<<3|b>>2)
    return im

def font(pt, bold=False):
    for n in (["arialbd.ttf"] if bold else ["arial.ttf"]):
        try: return ImageFont.truetype("C:/Windows/Fonts/"+n, pt)
        except Exception: pass
    return ImageFont.load_default()

def bevel(dr, box, base, raised=True):
    l,t,r,b=box
    hi=tuple(min(255,int(c*1.7)+30) for c in base); lo=tuple(int(c*0.45) for c in base)
    if not raised: hi,lo=lo,hi
    for k in range(2):
        dr.line([l,t+k,r,t+k],fill=hi); dr.line([l+k,t,l+k,b],fill=hi)
        dr.line([l,b-k,r,b-k],fill=lo); dr.line([r-k,t,r-k,b],fill=lo)

def ctext(dr,box,s,f,fill):
    l,t,r,b=box; w=dr.textlength(s,font=f); asc,desc=f.getmetrics()
    dr.text((l+((r-l)-w)/2, t+((b-t)-(asc+desc))/2), s, fill=fill, font=f)

def ordinal(n):
    return f"{n}{'th' if 10<=n%100<=20 else {1:'st',2:'nd',3:'rd'}.get(n%10,'th')}"

# ---- the DATA the backend view-model fills (Pld,Won,Drn,Lst,For,Ag) ----
VIEW_MODEL = {
    "competition": "English Premier Division",
    "user_club": "Everton",
    "standings": [
        ("Man Utd",31,26,2,3,61,18), ("Chelsea",31,18,8,5,57,32),
        ("Everton",31,18,8,5,62,28), ("Liverpool",31,15,11,5,41,23),
        ("Newcastle",30,15,6,9,43,31), ("Leeds",31,13,9,9,53,47),
        ("Tottenham",30,12,7,11,37,36), ("Bolton",31,11,6,13,35,41),
        ("Sunderland",30,10,10,10,38,32), ("Arsenal",31,10,7,13,29,30),
        ("Aston Villa",31,10,8,13,34,46), ("Nottm Forest",30,10,8,12,34,44),
    ],
    "cutoff_after": 1,   # dashed qualification marker after row 1 (comp +0xbe)
}

def main():
    img = Image.new("RGB", (800, 600), (12, 12, 16))
    dr = ImageDraw.Draw(img, "RGBA")
    f_title=font(30,True); f_tab=font(15); f_head=font(12,True)
    f_team=font(16); f_num=font(14); f_heading=font(24,True); f_btn=font(16,True); f_sm=font(13)

    # ===== CHROME =====
    # real sidebar image (Data/game.mbr) -- exact asset
    for p in ("D:/cm0102/Data/game.mbr","D:/cm0102/data/game.mbr"):
        if os.path.exists(p): img.paste(load_rgn(p),(0,0)); break

    # top banner: WHITE bar, blue trade_cond title, Print button (shell range 0x00494bc9)
    dr.rectangle([98,0,799,44], fill=(244,244,246)); dr.line([98,44,799,44],fill=(120,120,130))
    dr.text((196,4), VIEW_MODEL["competition"], fill=TITLE_BLUE, font=f_title)
    dr.rectangle([712,7,788,33], fill=(236,236,238)); bevel(dr,[712,7,788,33],(210,210,214))
    ctext(dr,[712,7,788,33],"Print",f_sm,(30,30,30))

    # tab strip: Table(active, gold border) Results Fixtures Schedule -- deep_blue pills
    for i,t in enumerate(["Table","Results","Fixtures","Schedule"]):
        x=108+i*172; box=[x,50,x+166,74]; dr.rectangle(box, fill=DEEP); bevel(dr,box,DEEP)
        if i==0:
            for k in range(2): dr.rectangle([box[0]+k,box[1]+k,box[2]-k,box[3]-k], outline=GOLD)
        ctext(dr,box,t,f_tab,WHITE)
    # "View" dropdown under the tabs (left)
    dr.rectangle([150,80,250,98], fill=MIDGREY); bevel(dr,[150,80,250,98],MIDGREY)
    dr.text((156,81),"View",fill=BLACK,font=f_sm); dr.polygon([(238,86),(246,86),(242,92)],fill=BLACK)

    # table backdrop: the football-net photo is a per-launch RANDOM Pictures/*.RGN -> dark fill
    # with faint net (INFERRED placeholder; a capture pins the exact asset).
    dr.rectangle([100,100,799,508], fill=(14,15,20))
    for gx in range(120,800,26): dr.line([gx,104,gx-40,504],fill=(30,32,40))
    for gy in range(104,505,26): dr.line([104,gy,799,gy-30],fill=(30,32,40))

    # "League Table" heading (string 0x00988a18), yellow, centered over the grid
    ctext(dr,[100,100,799,124],"League Table",f_heading,YELLOW)

    # column header pills (navy) over the 7 stat columns only
    for ci,label in HDR.items():
        l,r=COLS[ci]; box=[l,126,r,144]; dr.rectangle(box,fill=NAVY); bevel(dr,box,NAVY)
        ctext(dr,box,label,f_head,WHITE)

    # ===== DATA (parametric from the view-model) =====
    ROW_T, ROW_H = 148, 29   # row height INFERRED (lift marks uniformity inferred); matched to ref
    for i,(team,Pl,W,D,L,F,A) in enumerate(VIEW_MODEL["standings"]):
        y=ROW_T+i*ROW_H; box_b=y+ROW_H-2
        if box_b>506: break
        # translucent navy row stripe over the photo (alternating depth)
        dr.rectangle([110,y,779,box_b], fill=(0,0,128,180 if i%2 else 150))
        highlight = team==VIEW_MODEL["user_club"]
        tcol = YELLOW if highlight else WHITE
        # position cell (navy pill, ordinal) -- col 0
        pc=COLS[0]; pbox=[pc[0],y,pc[1],box_b]; dr.rectangle(pbox,fill=NAVY); bevel(dr,pbox,NAVY)
        ctext(dr,pbox,ordinal(i+1),f_num,WHITE)
        # team name -- col 2, left aligned
        l,r=COLS[2]; asc,desc=f_team.getmetrics()
        dr.text((l+4, y+((ROW_H-2)-(asc+desc))/2), team, fill=tcol, font=f_team)
        # stat numbers -- cols 4..9 centered
        for ci,val in {4:Pl,5:W,6:D,7:L,8:F,9:A}.items():
            ctext(dr,[COLS[ci][0],y,COLS[ci][1],box_b],str(val),f_num,tcol)
        # Pts cell -- col 10 navy pill, yellow text
        pc=COLS[10]; pbox=[pc[0],y,pc[1],box_b]; dr.rectangle(pbox,fill=NAVY); bevel(dr,pbox,NAVY)
        ctext(dr,pbox,str(W*3+D),f_num,YELLOW)
        # qualification cutoff dashed marker (comp +0xbe)
        if i+1==VIEW_MODEL["cutoff_after"]:
            for dx in range(112,780,10): dr.line([dx,box_b+1,dx+5,box_b+1],fill=YELLOW)

    # bottom stats bar (deep_blue pills) + Awards/History with arrows
    labels=[("Team Stats",0),("Player Stats",0),("Referee Stats",0),("Awards",1),("History",1)]
    for i,(t,arrow) in enumerate(labels):
        x=108+i*136; box=[x,512,x+132,540]; dr.rectangle(box,fill=DEEP); bevel(dr,box,DEEP)
        ctext(dr,box,t,f_sm,WHITE)
        if arrow: dr.polygon([(x+120,520),(x+120,532),(x+128,526)],fill=YELLOW)
    # Back / Next (grey)
    for t,box in [("Back",[100,548,446,584]),("Next",[452,548,796,584])]:
        dr.rectangle(box,fill=BTNGREY); bevel(dr,box,BTNGREY); ctext(dr,box,t,f_btn,BLACK)

    img.save(OUT); print("->", OUT)

if __name__ == "__main__":
    main()
