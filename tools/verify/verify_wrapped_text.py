"""Prove PackedSurface wrapped_text matches FUN_005d03a0 byte-exact.

Same save-restore pattern. Wrap flags let us exercise centring,
left/right align, wrap-on-overflow, and multi-line.
"""
from __future__ import annotations

import base64
import json
import math
import struct
import sys
from pathlib import Path

import frida

SCRATCH = (0, 0, 399, 79)
W, H = SCRATCH[2]-SCRATCH[0]+1, SCRATCH[3]-SCRATCH[1]+1

CASES = [
    # (label, rect, style, font, colour, text, prefill)
    {"label": "centre",       "x0": 0,   "y0": 0,  "x1": 300, "y1": 30, "style": 0x0,   "font": 3, "colour": 0x7fff, "text": "Center me", "prefill": "black"},
    {"label": "left align",   "x0": 0,   "y0": 30, "x1": 300, "y1": 60, "style": 0x1,   "font": 3, "colour": 0x7c00, "text": "Left me",   "prefill": "black"},
    {"label": "right align",  "x0": 100, "y0": 0,  "x1": 300, "y1": 30, "style": 0x40,  "font": 3, "colour": 0x03e0, "text": "Right me",  "prefill": "black"},
    {"label": "wrap",         "x0": 0,   "y0": 0,  "x1": 100, "y1": 60, "style": 0x101, "font": 3, "colour": 0x7fff, "text": "This is a longer piece of text", "prefill": "black"},
    {"label": "single word",  "x0": 0,   "y0": 0,  "x1": 300, "y1": 30, "style": 0x2,   "font": 3, "colour": 0x7fff, "text": "Arsenal",   "prefill": "black"},
    # --- W_SHADOW (0x20): ink is sampled from the rect centre, not `colour` ---
    {"label": "shadow grey",  "x0": 0,   "y0": 0,  "x1": 300, "y1": 30, "style": 0x23,  "font": 3, "colour": 0x7fff, "text": "Shadow me", "prefill": "grey"},
    {"label": "shadow black", "x0": 0,   "y0": 30, "x1": 300, "y1": 60, "style": 0x20,  "font": 3, "colour": 0x7c00, "text": "Shadow black", "prefill": "black"},
    {"label": "shadow wrap",  "x0": 0,   "y0": 0,  "x1": 120, "y1": 79, "style": 0x120, "font": 3, "colour": 0x03e0, "text": "Wrapped shadow text here", "prefill": "grey"},
    {"label": "shadow right", "x0": 100, "y0": 20, "x1": 390, "y1": 60, "style": 0x60,  "font": 3, "colour": 0x0000, "text": "Right shadow", "prefill": "grey"},
]

SCRIPT_TMPL = """
'use strict';
const wrapFn = new NativeFunction(ptr('0x005d03a0'), 'void',
    ['int','int','int','int','uint','uint16','uint16','pointer','int'], 'mscdecl');
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
    fontTable(idx){const base=ptr('0x00accae4').add(idx*0x1404);return b64(base.readByteArray(0x1404));},
    readBytes(addr,n){return b64(ptr(addr).readByteArray(n));},
    runCase(c){
        const orig=saveFn(__X0__,__Y0__,__X1__,__Y1__,NULL);
        rectFn(__X0__,__Y0__,__X1__,__Y1__,0,c.prefill==='grey'?0x4210:0x0000);
        const before=readScratch();
        // Force traditional-bitmap font path — same as verify_glyph.py.
        const flagAddr=ptr('0x009b9d54');
        const saved=flagAddr.readU32(); flagAddr.writeU32(0);
        const txt=Memory.allocUtf8String(c.text);
        wrapFn(c.x0,c.y0,c.x1,c.y1,c.style,c.font,c.colour,txt,-1);
        flagAddr.writeU32(saved);
        const after=readScratch();
        restoreFn(__X0__,__Y0__,orig);
        return {before,after};
    },
};
"""


def parse_font_table(raw_b64, read_bytes_fn):
    raw = base64.b64decode(raw_b64)
    height = struct.unpack_from('<i', raw, 0)[0]
    glyphs = []
    for c in range(256):
        off = 4 + c * 0x14
        width, _a, _b, _c, ptr = struct.unpack_from('<iiiiI', raw, off)
        if width <= 0 or ptr == 0:
            glyphs.append({"width": width, "kern_a": _a, "kern_b": _b, "kern_c": _c, "bitmap_b64": ""})
            continue
        n_bytes = math.ceil(width / 2) * height  # row-padded (see verify_glyph.py note)
        try:
            bitmap_b64 = read_bytes_fn(ptr, n_bytes)
        except Exception:
            glyphs.append(None); continue
        glyphs.append({"width": width, "kern_a": _a, "kern_b": _b, "kern_c": _c, "bitmap_b64": bitmap_b64})
    return {"height": height, "glyphs": glyphs}


def main():
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 \
        else Path("D:/cm0102-rs/fixtures/verify_wrapped_text.json")
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
    font_ids = sorted({c["font"] for c in CASES})
    fonts = {}
    for fid in font_ids:
        raw = api.font_table(fid)
        fonts[str(fid)] = parse_font_table(raw, api.read_bytes)
    out_cases = []
    for c in CASES:
        r = api.run_case(c)
        out_cases.append({**c, "before_b64": r["before"], "after_b64": r["after"]})
        print(f"  {c['label']}", flush=True)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps({
        "va": "0x005d03a0",
        "scratch": {"x0": SCRATCH[0], "y0": SCRATCH[1], "x1": SCRATCH[2], "y1": SCRATCH[3],
                    "width": W, "height": H},
        "fonts": fonts,
        "cases": out_cases,
    }))
    print(f"wrote {out_path}", flush=True)
    s.detach()


if __name__ == "__main__": sys.exit(main() or 0)
