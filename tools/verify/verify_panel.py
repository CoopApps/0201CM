"""Prove PackedSurface::draw_panel matches FUN_005cf570 byte-exact.

Panel is the biggest primitive by style-flag surface (20 flags decoded).
The test matrix covers the important combinations: solid fill, gradients
(H and V), bevel (thin and thick, raised and sunken), frames (solid and
dashed), outer highlight, sample-bg. Each case is drawn into a scratch
region with a known pre-fill and save-restored around the call.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import frida

SCRATCH = (0, 0, 199, 79)  # 200x80 (larger so bevel corners show)
W, H = SCRATCH[2]-SCRATCH[0]+1, SCRATCH[3]-SCRATCH[1]+1

# P_* flag values from packed_panel.rs
P_DARKEN         = 0x00000002
P_HGRADIENT      = 0x00000004
P_VGRADIENT      = 0x00000008
P_SOLID_FILL     = 0x00000010
P_BEVEL          = 0x00000020
P_BEVEL_INVERT   = 0x00000040
P_BOTTOM_SHADOW  = 0x00000080
P_SOLID_FRAME    = 0x00000200
P_RIGHT_EDGE     = 0x00000400
P_OUTER_HIGHLIGHT= 0x00000800
P_DASH_VARIANT   = 0x00002000
P_ALT_PRIMITIVE  = 0x00010000
P_MIDLINE_H      = 0x01000000

CASES = [
    # --- FUN_005d0ce0 circle paths (7th arg "pattern" = outline colour) ---
    {"label": "circle fill (alt)",          "x0": 10, "y0": 10, "x1": 60, "y1": 60, "style": P_SOLID_FILL|P_ALT_PRIMITIVE,               "colour": 0x7c00, "pattern": 0x0000, "prefill": None},
    {"label": "circle outline solid",       "x0": 70, "y0": 5,  "x1": 130,"y1": 65, "style": P_BEVEL|P_ALT_PRIMITIVE,                    "colour": 0x03e0, "pattern": 0x001f, "prefill": None},
    {"label": "circle outline dashed",      "x0": 140,"y0": 10, "x1": 195,"y1": 70, "style": P_BEVEL|P_ALT_PRIMITIVE|P_DASH_VARIANT,     "colour": 0x03e0, "pattern": 0x7fff, "prefill": None},
    {"label": "circle fill + outline",      "x0": 10, "y0": 10, "x1": 70, "y1": 70, "style": P_SOLID_FILL|P_BEVEL|P_ALT_PRIMITIVE,       "colour": 0x7c00, "pattern": 0x03e0, "prefill": None},
    {"label": "circle tall rect (x-clip)",  "x0": 10, "y0": 5,  "x1": 40, "y1": 75, "style": P_SOLID_FILL|P_BEVEL|P_ALT_PRIMITIVE,       "colour": 0x001f, "pattern": 0x7fff, "prefill": "fill_grey"},
    {"label": "circle wide rect",           "x0": 5,  "y0": 5,  "x1": 190,"y1": 40, "style": P_SOLID_FILL|P_ALT_PRIMITIVE,               "colour": 0x7fff, "pattern": 0x0000, "prefill": None},
    {"label": "circle even height dashed",  "x0": 100,"y0": 20, "x1": 150,"y1": 61, "style": P_BEVEL|P_ALT_PRIMITIVE|P_DASH_VARIANT,     "colour": 0x0000, "pattern": 0x7c00, "prefill": "fill_grey"},
    {"label": "circle tiny 5x5",            "x0": 3,  "y0": 3,  "x1": 7,  "y1": 7,  "style": P_SOLID_FILL|P_BEVEL|P_ALT_PRIMITIVE,       "colour": 0x7c00, "pattern": 0x03e0, "prefill": None},
    # rect (relative to scratch origin); coords are absolute in exe.
    {"label": "solid fill",            "x0": 10, "y0": 10, "x1": 60, "y1": 40, "style": P_SOLID_FILL,           "colour": 0x7c00, "prefill": None},
    {"label": "vgradient",             "x0": 70, "y0": 10, "x1": 130,"y1": 60, "style": P_VGRADIENT,            "colour": 0x7c00, "prefill": None},
    {"label": "hgradient",             "x0": 140,"y0": 10, "x1": 195,"y1": 40, "style": P_HGRADIENT,            "colour": 0x03e0, "prefill": None},
    {"label": "solid frame",           "x0": 10, "y0": 50, "x1": 60, "y1": 70, "style": P_BEVEL|P_SOLID_FRAME,  "colour": 0x001f, "prefill": None},
    {"label": "dashed frame",          "x0": 65, "y0": 50, "x1": 130,"y1": 70, "style": P_BEVEL|P_SOLID_FRAME|P_DASH_VARIANT, "colour": 0x7fff, "prefill": None},
    {"label": "bevel thin (2px)",      "x0": 5,  "y0": 5,  "x1": 40, "y1": 30, "style": P_BEVEL,                "colour": 0x4210, "prefill": "fill_grey"},
    {"label": "bevel thick (4px)",     "x0": 50, "y0": 5,  "x1": 130,"y1": 60, "style": P_BEVEL|P_SOLID_FILL,   "colour": 0x4210, "prefill": None},
    {"label": "bevel invert (sunken)", "x0": 5,  "y0": 40, "x1": 50, "y1": 70, "style": P_BEVEL|P_BEVEL_INVERT, "colour": 0x4210, "prefill": "fill_grey"},
    {"label": "outer highlight",       "x0": 20, "y0": 20, "x1": 60, "y1": 50, "style": P_SOLID_FILL|P_OUTER_HIGHLIGHT, "colour": 0x001f, "prefill": None},
    {"label": "midline horizontal",    "x0": 10, "y0": 10, "x1": 190,"y1": 20, "style": P_MIDLINE_H,            "colour": 0x7c00, "prefill": None},
    {"label": "bottom shadow",         "x0": 10, "y0": 30, "x1": 100,"y1": 50, "style": P_BOTTOM_SHADOW,        "colour": 0x7c00, "prefill": None},
    {"label": "right edge",            "x0": 100,"y0": 10, "x1": 150,"y1": 60, "style": P_RIGHT_EDGE,           "colour": 0x03e0, "prefill": None},
    {"label": "darken",                "x0": 10, "y0": 10, "x1": 60, "y1": 40, "style": P_DARKEN,               "colour": 0x0000, "prefill": "fill_red"},
]

SCRIPT_TMPL = """
'use strict';
const panelFn = new NativeFunction(ptr('0x005cf570'), 'void',
    ['int','int','int','int','uint','uint','uint'], 'mscdecl');
const rectFn = new NativeFunction(ptr('0x005cd730'), 'void',
    ['int','int','int','int','uint','uint16'], 'mscdecl');
const saveFn = new NativeFunction(ptr('0x005cd930'), 'pointer',
    ['int','int','int','int','pointer'], 'mscdecl');
const restoreFn = new NativeFunction(ptr('0x005cda90'), 'void',
    ['int','int','pointer'], 'mscdecl');
const B64='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
function b64(bytes){const v=new Uint8Array(bytes),n=v.length,out=new Array(Math.ceil(n/3)*4);let o=0,i=0;
for(;i+2<n;i+=3){const b1=v[i],b2=v[i+1],b3=v[i+2];out[o++]=B64[b1>>2];out[o++]=B64[((b1&3)<<4)|(b2>>4)];out[o++]=B64[((b2&15)<<2)|(b3>>6)];out[o++]=B64[b3&63];}
if(i<n){const b1=v[i],b2=(i+1<n)?v[i+1]:0;out[o++]=B64[b1>>2];out[o++]=B64[((b1&3)<<4)|(b2>>4)];if(i+1<n){out[o++]=B64[(b2&15)<<2];out[o++]='=';}else{out[o++]='=';out[o++]='=';}}
return out.join('');}
function readScratch(){
    const buf=ptr('0x00ad6b1c').readPointer(),pitch=ptr('0x00acdeb8').readU16();
    const bytes=new Uint8Array(__W__*__H__*2);
    for(let y=0;y<__H__;y++){const src=buf.add(((__Y0__+y)*pitch+__X0__)*2);bytes.set(new Uint8Array(src.readByteArray(__W__*2)),y*__W__*2);}
    return b64(bytes.buffer);
}
rpc.exports = {
    palette(){
        // The exe's palette globals used by draw_panel:
        //   DAT_00ad6b24 = outer highlight colour (P_OUTER_HIGHLIGHT)
        //   DAT_00ad6b3c = default bevel colour when caller passes colour=0
        return {
            outer_highlight: ptr('0x00ad6b24').readU16(),
            default_bevel:   ptr('0x00ad6b3c').readU16(),
        };
    },
    runCase(c){
        const orig=saveFn(__X0__,__Y0__,__X1__,__Y1__,NULL);
        // Clear to black.
        rectFn(__X0__,__Y0__,__X1__,__Y1__,0,0x0000);
        // Optional pre-fill.
        if (c.prefill === 'fill_grey') rectFn(__X0__,__Y0__,__X1__,__Y1__,0,0x4210);
        else if (c.prefill === 'fill_red') rectFn(__X0__,__Y0__,__X1__,__Y1__,0,0x7c00);
        const before = readScratch();
        panelFn(c.x0, c.y0, c.x1, c.y1, c.style, c.colour, c.pattern || 0);
        const after = readScratch();
        restoreFn(__X0__,__Y0__,orig);
        return {before, after};
    },
};
"""

def main():
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 \
        else Path("D:/cm0102-rs/fixtures/verify_panel.json")
    dev = frida.get_local_device()
    procs = [p for p in dev.enumerate_processes() if p.name.lower() == "cm0102_gdi.exe"]
    if not procs: print("no exe"); return 2
    s = dev.attach(procs[0].pid)
    src = SCRIPT_TMPL.replace("__X0__", str(SCRATCH[0])).replace("__Y0__", str(SCRATCH[1])) \
                     .replace("__X1__", str(SCRATCH[2])).replace("__Y1__", str(SCRATCH[3])) \
                     .replace("__W__", str(W)).replace("__H__", str(H))
    scr = s.create_script(src)
    scr.on("message", lambda m,_d: print(f"msg: {m}", flush=True))
    scr.load()
    api = scr.exports_sync
    palette = api.palette()
    print(f"palette: outer_highlight={palette['outer_highlight']:#06x} "
          f"default_bevel={palette['default_bevel']:#06x}", flush=True)
    out = []
    for c in CASES:
        r = api.run_case(c)
        out.append({**c, "before_b64": r["before"], "after_b64": r["after"]})
        print(f"  {c['label']}", flush=True)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps({
        "va": "0x005cf570",
        "palette": palette,
        "scratch": {"x0": SCRATCH[0], "y0": SCRATCH[1], "x1": SCRATCH[2], "y1": SCRATCH[3],
                    "width": W, "height": H},
        "cases": out,
    }))
    print(f"wrote {out_path}", flush=True)
    s.detach()

if __name__ == "__main__": sys.exit(main() or 0)
