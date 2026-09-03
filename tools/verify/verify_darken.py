"""Prove `PackedSurface::darken_rect` matches FUN_005cdd60 byte-exact.

The exe builds a 65536-entry LUT that ×60/100 darkens every packed
value (per-channel), then applies it. We pre-paint the scratch region
with a known gradient, save, darken via the exe, read, restore.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import frida

SCRATCH = (0, 0, 199, 39)  # 200x40
W = SCRATCH[2] - SCRATCH[0] + 1
H = SCRATCH[3] - SCRATCH[1] + 1

# We want a rich set of pre-existing pixels so the LUT gets exercised
# widely. Cases:
#   1. gradient across the scratch (each pixel has a unique packed value)
#   2. uniform pure-red / green / blue / white / black regions
CASES = [
    {"label": "gradient",  "prefill": "gradient"},
    {"label": "red",       "prefill": "colour", "colour": 0x7c00},
    {"label": "green",     "prefill": "colour", "colour": 0x03e0},
    {"label": "blue",      "prefill": "colour", "colour": 0x001f},
    {"label": "white",     "prefill": "colour", "colour": 0x7fff},
    {"label": "grey mid",  "prefill": "colour", "colour": 0x4210},
]

SCRIPT_TMPL = """
'use strict';
const darkenFn = new NativeFunction(ptr('0x005cdd60'), 'void',
    ['int','int','int','int'], 'mscdecl');
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
    const w=__W__,h=__H__;
    const bytes=new Uint8Array(w*h*2);
    for(let y=0;y<h;y++){const src=buf.add(((__Y0__+y)*pitch+__X0__)*2);bytes.set(new Uint8Array(src.readByteArray(w*2)),y*w*2);}
    return b64(bytes.buffer);
}
function writeScratchU16(u16arr){
    const buf=ptr('0x00ad6b1c').readPointer(),pitch=ptr('0x00acdeb8').readU16();
    const w=__W__,h=__H__;
    const bytes=new Uint8Array(w*h*2);
    for(let i=0;i<w*h;i++){bytes[i*2]=u16arr[i]&0xff;bytes[i*2+1]=(u16arr[i]>>8)&0xff;}
    for(let y=0;y<h;y++){const dst=buf.add(((__Y0__+y)*pitch+__X0__)*2);dst.writeByteArray(bytes.slice(y*w*2,(y+1)*w*2));}
}
rpc.exports = {
    runCase(c){
        const savedOrig = saveFn(__X0__,__Y0__,__X1__,__Y1__,NULL);
        // Pre-fill.
        if (c.prefill === 'gradient') {
            const arr = new Uint16Array(__W__ * __H__);
            for (let i = 0; i < arr.length; i++) arr[i] = i & 0xffff;
            writeScratchU16(Array.from(arr));
        } else {
            rectFn(__X0__,__Y0__,__X1__,__Y1__,0,c.colour);
        }
        const before = readScratch();
        darkenFn(__X0__,__Y0__,__X1__,__Y1__);
        const after = readScratch();
        restoreFn(__X0__,__Y0__,savedOrig);
        return {before, after};
    },
};
"""

def main():
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 \
        else Path("D:/cm0102-rs/fixtures/verify_darken.json")
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
    out = []
    for c in CASES:
        r = api.run_case(c)
        out.append({**c, "before_b64": r["before"], "after_b64": r["after"]})
        print(f"  {c['label']}", flush=True)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps({
        "va": "0x005cdd60",
        "scratch": {"x0": SCRATCH[0], "y0": SCRATCH[1], "x1": SCRATCH[2], "y1": SCRATCH[3],
                    "width": W, "height": H},
        "cases": out,
    }))
    print(f"wrote {out_path}", flush=True)
    s.detach()

if __name__ == "__main__": sys.exit(main() or 0)
