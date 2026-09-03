"""Prove `PackedSurface::draw_rectangle` matches FUN_005cd730 byte-exact.

Save-restore around each test to preserve the game display. Three modes
to cover: style bit 0 = dashed frame (4 dashed lines), style bit 1 =
solid frame (4 solid lines), neither = FILL (horizontal solid lines
iterated).
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import frida

CASES = [
    # (label, x0, y0, x1, y1, style, colour)
    {"label": "fill small",         "x0": 5, "y0": 5, "x1": 30,  "y1": 25, "style": 0, "colour": 0x7c00},
    {"label": "fill 1x1",           "x0": 100,"y0": 20,"x1": 100,"y1": 20, "style": 0, "colour": 0x001f},
    {"label": "fill 1-pixel wide",  "x0": 40,"y0": 10,"x1": 40,  "y1": 30, "style": 0, "colour": 0x03e0},
    {"label": "fill 1-pixel tall",  "x0": 50,"y0": 15,"x1": 100, "y1": 15, "style": 0, "colour": 0x7fff},
    {"label": "fill wide",          "x0": 0, "y0": 0, "x1": 199, "y1": 39, "style": 0, "colour": 0x7c1f},
    {"label": "solid frame",        "x0": 5, "y0": 5, "x1": 30,  "y1": 25, "style": 2, "colour": 0x7fff},
    {"label": "solid frame small",  "x0": 60,"y0": 5, "x1": 65,  "y1": 10, "style": 2, "colour": 0x7c00},
    {"label": "dashed frame",       "x0": 80,"y0": 3, "x1": 130, "y1": 30, "style": 1, "colour": 0x03e0},
    {"label": "reversed endpoints", "x0": 150,"y0":30,"x1": 130, "y1": 5,  "style": 0, "colour": 0x7c1f},
    {"label": "solid frame big",    "x0": 0, "y0": 0, "x1": 199, "y1": 39, "style": 2, "colour": 0x001f},
]

SCRATCH = (0, 0, 199, 39)

SCRIPT_TMPL = """
'use strict';
const rectFn = new NativeFunction(ptr('0x005cd730'), 'void',
    ['int','int','int','int','uint','uint16'], 'mscdecl');
const saveFn = new NativeFunction(ptr('0x005cd930'), 'pointer',
    ['int','int','int','int','pointer'], 'mscdecl');
const restoreFn = new NativeFunction(ptr('0x005cda90'), 'void',
    ['int','int','pointer'], 'mscdecl');
const B64 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
function b64(bytes){const v=new Uint8Array(bytes),n=v.length,out=new Array(Math.ceil(n/3)*4);let o=0,i=0;
for(;i+2<n;i+=3){const b1=v[i],b2=v[i+1],b3=v[i+2];out[o++]=B64[b1>>2];out[o++]=B64[((b1&3)<<4)|(b2>>4)];out[o++]=B64[((b2&15)<<2)|(b3>>6)];out[o++]=B64[b3&63];}
if(i<n){const b1=v[i],b2=(i+1<n)?v[i+1]:0;out[o++]=B64[b1>>2];out[o++]=B64[((b1&3)<<4)|(b2>>4)];if(i+1<n){out[o++]=B64[(b2&15)<<2];out[o++]='=';}else{out[o++]='=';out[o++]='=';}}
return out.join('');}
function readScratch(){
    const buf=ptr('0x00ad6b1c').readPointer(),pitch=ptr('0x00acdeb8').readU16();
    const w=__X1__-__X0__+1,h=__Y1__-__Y0__+1;
    const bytes=new Uint8Array(w*h*2);
    for(let y=0;y<h;y++){const src=buf.add(((__Y0__+y)*pitch+__X0__)*2);bytes.set(new Uint8Array(src.readByteArray(w*2)),y*w*2);}
    return b64(bytes.buffer);
}
rpc.exports = {
    runCase(c){
        const saved=saveFn(__X0__,__Y0__,__X1__,__Y1__,NULL);
        const before=readScratch();
        rectFn(c.x0,c.y0,c.x1,c.y1,c.style,c.colour);
        const after=readScratch();
        restoreFn(__X0__,__Y0__,saved);
        return {before,after};
    },
};
"""

def main():
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 \
        else Path("D:/cm0102-rs/fixtures/verify_rect.json")
    dev = frida.get_local_device()
    procs = [p for p in dev.enumerate_processes() if p.name.lower() == "cm0102_gdi.exe"]
    if not procs: print("no exe"); return 2
    s = dev.attach(procs[0].pid)
    src = SCRIPT_TMPL.replace("__X0__", str(SCRATCH[0])).replace("__Y0__", str(SCRATCH[1])) \
                     .replace("__X1__", str(SCRATCH[2])).replace("__Y1__", str(SCRATCH[3]))
    scr = s.create_script(src)
    scr.on("message", lambda m,_d: print(f"msg: {m}", flush=True))
    scr.load()
    api = scr.exports_sync
    out = []
    for c in CASES:
        r = api.run_case(c)
        out.append({**c, "before_b64": r["before"], "after_b64": r["after"]})
        print(f"  {c['label']}", flush=True)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps({
        "va": "0x005cd730",
        "scratch": {"x0": SCRATCH[0], "y0": SCRATCH[1], "x1": SCRATCH[2], "y1": SCRATCH[3],
                    "width": SCRATCH[2]-SCRATCH[0]+1, "height": SCRATCH[3]-SCRATCH[1]+1},
        "cases": out,
    }))
    print(f"wrote {out_path}", flush=True)
    s.detach()

if __name__ == "__main__": sys.exit(main() or 0)
