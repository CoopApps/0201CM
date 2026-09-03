"""Prove PackedSurface save/restore match FUN_005cd930/FUN_005cda90.

Save-verify: fill scratch with a known u16 gradient, ask the exe to
save it, dereference the returned header to read back the saved
pixels, ship them alongside the input to the Rust test which asserts
our port produces the identical saved bytes.

Restore-verify: paint scratch with pattern A, save (via exe), overwrite
with pattern B, restore (via exe), read scratch, ship (A, B, restored)
where restored MUST equal A byte-for-byte. Our port's restore must do
the same.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import frida

SCRATCH = (0, 0, 99, 19)  # 100 x 20 for speed
W, H = 100, 20

SCRIPT_TMPL = """
'use strict';
const rectFn    = new NativeFunction(ptr('0x005cd730'), 'void',
    ['int','int','int','int','uint','uint16'], 'mscdecl');
const saveFn    = new NativeFunction(ptr('0x005cd930'), 'pointer',
    ['int','int','int','int','pointer'], 'mscdecl');
const restoreFn = new NativeFunction(ptr('0x005cda90'), 'void',
    ['int','int','pointer'], 'mscdecl');

const B64 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
function b64(bytes){const v=new Uint8Array(bytes),n=v.length,out=new Array(Math.ceil(n/3)*4);let o=0,i=0;
for(;i+2<n;i+=3){const b1=v[i],b2=v[i+1],b3=v[i+2];out[o++]=B64[b1>>2];out[o++]=B64[((b1&3)<<4)|(b2>>4)];out[o++]=B64[((b2&15)<<2)|(b3>>6)];out[o++]=B64[b3&63];}
if(i<n){const b1=v[i],b2=(i+1<n)?v[i+1]:0;out[o++]=B64[b1>>2];out[o++]=B64[((b1&3)<<4)|(b2>>4)];if(i+1<n){out[o++]=B64[(b2&15)<<2];out[o++]='=';}else{out[o++]='=';out[o++]='=';}}
return out.join('');}
function writeU16Grid(u16){
    const buf=ptr('0x00ad6b1c').readPointer(),pitch=ptr('0x00acdeb8').readU16();
    for(let y=0;y<__H__;y++){
        const dst=buf.add(((__Y0__+y)*pitch+__X0__)*2);
        const rowBytes=new Uint8Array(__W__*2);
        for(let x=0;x<__W__;x++){const v=u16[y*__W__+x];rowBytes[x*2]=v&0xff;rowBytes[x*2+1]=(v>>8)&0xff;}
        dst.writeByteArray(rowBytes);
    }
}
function readScratch(){
    const buf=ptr('0x00ad6b1c').readPointer(),pitch=ptr('0x00acdeb8').readU16();
    const bytes=new Uint8Array(__W__*__H__*2);
    for(let y=0;y<__H__;y++){const src=buf.add(((__Y0__+y)*pitch+__X0__)*2);bytes.set(new Uint8Array(src.readByteArray(__W__*2)),y*__W__*2);}
    return b64(bytes.buffer);
}
rpc.exports = {
    saveVerify(u16arr){
        const origSaved = saveFn(__X0__,__Y0__,__X1__,__Y1__,NULL);  // snapshot to restore later
        writeU16Grid(u16arr);
        const scratch_before = readScratch();
        // Ask exe to save the scratch region into a fresh buffer.
        const savedPtr = saveFn(__X0__,__Y0__,__X1__,__Y1__,NULL);
        // Header layout (FUN_005cd930): int[0]=width, int[1]=height,
        // int[2]=size_bytes, int[3]=data_ptr; data lives at data_ptr.
        const sw = savedPtr.readInt();
        const sh = savedPtr.add(4).readInt();
        const sz = savedPtr.add(8).readInt();
        const dp = savedPtr.add(12).readPointer();
        const savedBytes = b64(dp.readByteArray(sz));
        restoreFn(__X0__,__Y0__,origSaved);
        return { input: scratch_before, saved: savedBytes, sw, sh, sz };
    },
    restoreVerify(a_u16, b_u16){
        const origSaved = saveFn(__X0__,__Y0__,__X1__,__Y1__,NULL);
        writeU16Grid(a_u16);
        const savedA = saveFn(__X0__,__Y0__,__X1__,__Y1__,NULL);
        writeU16Grid(b_u16);
        const scratchB = readScratch();
        restoreFn(__X0__,__Y0__,savedA);
        const restored = readScratch();
        // Also dump savedA's raw pixels so the Rust test can construct
        // an equivalent SavedRect for its own port to restore from.
        const dp = savedA.add(12).readPointer();
        const sz = savedA.add(8).readInt();
        const savedBytes = b64(dp.readByteArray(sz));
        restoreFn(__X0__,__Y0__,origSaved);
        return { scratchB, restored, savedBytes };
    },
};
"""

def main():
    out_path = Path(sys.argv[1]) if len(sys.argv) > 1 \
        else Path("D:/cm0102-rs/fixtures/verify_save_restore.json")
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

    # Gradient of unique u16 values.
    gradient = [i & 0xffff for i in range(W * H)]
    # Reverse for pattern B.
    reverse = [(W*H - 1 - i) & 0xffff for i in range(W * H)]

    r1 = api.save_verify(gradient)
    r2 = api.restore_verify(gradient, reverse)

    out = {
        "va_save": "0x005cd930",
        "va_restore": "0x005cda90",
        "scratch": {"x0": SCRATCH[0], "y0": SCRATCH[1], "x1": SCRATCH[2], "y1": SCRATCH[3],
                    "width": W, "height": H},
        "save_case": {
            "input_b64":     r1["input"],
            "saved_b64":     r1["saved"],
            "saved_width":   r1["sw"],
            "saved_height":  r1["sh"],
            "saved_size":    r1["sz"],
        },
        "restore_case": {
            "pattern_a_b64":     json_gradient_b64(gradient),
            "pattern_b_b64":     r2["scratchB"],
            "restored_b64":      r2["restored"],
            "exe_saved_bytes_b64": r2["savedBytes"],
        },
    }
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(out))
    print(f"wrote {out_path}", flush=True)
    s.detach()


def json_gradient_b64(u16arr):
    import base64
    b = bytearray(len(u16arr) * 2)
    for i, v in enumerate(u16arr):
        b[i*2] = v & 0xff
        b[i*2+1] = (v >> 8) & 0xff
    return base64.b64encode(bytes(b)).decode('ascii')


if __name__ == "__main__": sys.exit(main() or 0)
