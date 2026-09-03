"""Capture one screen's primitive-call stream by posting a click into
the game (asynchronous — no deadlock like SendMessage). Also captures
before/after framebuffers so the fixture is complete.

Usage:
  python click_capture.py <label> <x> <y> [--out fixtures/X.json]

Default click at (700, 573) — Next button (bottom-right of any list
screen). Change --x --y to click something else. PostMessage returns
immediately; we sleep 500ms for the exe to process, then read the
after-buffer + snapshot the call list.
"""
from __future__ import annotations
import argparse, base64, json, sys, time
from pathlib import Path
import frida

SCRIPT = """
'use strict';
const B64='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
function b64(bytes){const v=new Uint8Array(bytes),n=v.length,out=new Array(Math.ceil(n/3)*4);let o=0,i=0;
for(;i+2<n;i+=3){const b1=v[i],b2=v[i+1],b3=v[i+2];out[o++]=B64[b1>>2];out[o++]=B64[((b1&3)<<4)|(b2>>4)];out[o++]=B64[((b2&15)<<2)|(b3>>6)];out[o++]=B64[b3&63];}
if(i<n){const b1=v[i],b2=(i+1<n)?v[i+1]:0;out[o++]=B64[b1>>2];out[o++]=B64[((b1&3)<<4)|(b2>>4)];if(i+1<n){out[o++]=B64[(b2&15)<<2];out[o++]='=';}else{out[o++]='=';out[o++]='=';}}
return out.join('');}
function snapshot(){
    const buf=ptr('0x00ad6b1c').readPointer();
    const w=ptr('0x00ad6b40').readS32(),h=ptr('0x00ad6b08').readS32();
    const pitch=ptr('0x00acdeb8').readU16();
    return { width:w, height:h, pitch, data:b64(buf.readByteArray(pitch*h*2)) };
}

// Hook every primitive; accumulate calls into a global.
const CALLS = [];
const VA = {
    LINE: ptr('0x005cd3e0'), RECT: ptr('0x005cd730'), DARKEN: ptr('0x005cdd60'),
    RESTORE: ptr('0x005cda90'), GLYPH: ptr('0x005ceaa0'),
    PANEL: ptr('0x005cf570'), WRAPPED: ptr('0x005d03a0'),
};
function readStr(p){ if(p.isNull())return ''; try{return p.readCString(256);}catch(_){return '';} }
Interceptor.attach(VA.LINE, {onEnter(a){CALLS.push({op:'line', x0:a[0].toInt32(), y0:a[1].toInt32(), x1:a[2].toInt32(), y1:a[3].toInt32(), style:a[4].toUInt32(), colour:a[5].toUInt32()&0xffff});}});
Interceptor.attach(VA.RECT, {onEnter(a){CALLS.push({op:'rect', x0:a[0].toInt32(), y0:a[1].toInt32(), x1:a[2].toInt32(), y1:a[3].toInt32(), style:a[4].toUInt32(), colour:a[5].toUInt32()&0xffff});}});
Interceptor.attach(VA.DARKEN, {onEnter(a){CALLS.push({op:'darken', x0:a[0].toInt32(), y0:a[1].toInt32(), x1:a[2].toInt32(), y1:a[3].toInt32()});}});
Interceptor.attach(VA.RESTORE, {onEnter(a){CALLS.push({op:'restore', x:a[0].toInt32(), y:a[1].toInt32(), saved_ptr:a[2].toString()});}});
Interceptor.attach(VA.GLYPH, {onEnter(a){CALLS.push({op:'glyph', x:a[0].toInt32(), y:a[1].toInt32(), font:a[2].toUInt32()&0xffff, colour:a[3].toUInt32()&0xffff, text:readStr(a[4]), underline:a[5].toInt32()});}});
Interceptor.attach(VA.PANEL, {onEnter(a){CALLS.push({op:'panel', x0:a[0].toInt32(), y0:a[1].toInt32(), x1:a[2].toInt32(), y1:a[3].toInt32(), style:a[4].toUInt32().toString(16), colour:a[5].toUInt32()&0xffff});}});
Interceptor.attach(VA.WRAPPED, {onEnter(a){CALLS.push({op:'wrapped', x0:a[0].toInt32(), y0:a[1].toInt32(), x1:a[2].toInt32(), y1:a[3].toInt32(), style:a[4].toUInt32().toString(16), font:a[5].toUInt32()&0xffff, colour:a[6].toUInt32()&0xffff, text:readStr(a[7]), underline:a[8].toInt32()});}});

const user32 = Process.getModuleByName('user32.dll');
const PostMessageA = new NativeFunction(user32.getExportByName('PostMessageA'), 'int', ['pointer','uint','uint','uint'], 'stdcall');
const InvalidateRect = new NativeFunction(user32.getExportByName('InvalidateRect'), 'int', ['pointer','pointer','int'], 'stdcall');

rpc.exports = {
    snapshot,
    reset(){ CALLS.length = 0; },
    getCalls(){ return CALLS; },
    click(hwnd_override, x, y){
        // Pass hwnd from the driver (Windows MainWindowHandle), not the
        // exe's DAT_00b4d4e8 which points at a different (inner?) window.
        const hwnd = ptr(hwnd_override);
        const lparam = ((y & 0xffff) << 16) | (x & 0xffff);
        PostMessageA(hwnd, 0x200, 0, lparam);       // WM_MOUSEMOVE
        PostMessageA(hwnd, 0x201, 1, lparam);       // WM_LBUTTONDOWN
        PostMessageA(hwnd, 0x202, 0, lparam);       // WM_LBUTTONUP
        return { hwnd: hwnd.toString(), x, y };
    },
    invalidate(){
        const hwnd = ptr('0x00b4d4e8').readPointer();
        InvalidateRect(hwnd, NULL, 1);
        return { hwnd: hwnd.toString() };
    },
    palette(){
        return {
            outer_highlight: ptr('0x00ad6b24').readU16(),
            default_bevel:   ptr('0x00ad6b3c').readU16(),
        };
    },
};
"""

def main():
    p = argparse.ArgumentParser()
    p.add_argument("label")
    p.add_argument("x", type=int, nargs='?', default=700)
    p.add_argument("y", type=int, nargs='?', default=573)
    p.add_argument("--out", type=Path, default=None)
    p.add_argument("--font-mode", choices=["traditional", "current"], default="traditional")
    args = p.parse_args()
    out = args.out or Path(f"D:/cm0102-rs/fixtures/screen_{args.label}.json")

    dev = frida.get_local_device()
    pid = next(pr.pid for pr in dev.enumerate_processes() if pr.name.lower() == "cm0102_gdi.exe")
    s = dev.attach(pid)
    scr = s.create_script(SCRIPT)
    scr.on("message", lambda m,_: print(f"msg: {m}", file=sys.stderr))
    scr.load()
    api = scr.exports_sync

    if args.font_mode == "traditional":
        # Force the traditional-bitmap path so my ported renderer is the oracle.
        scr.post({}); # keepalive
    palette = api.palette()
    before = api.snapshot()
    api.reset()
    print(f"before: {before['width']}x{before['height']} pitch={before['pitch']} palette={palette}", file=sys.stderr)
    print(f"clicking ({args.x}, {args.y}) via PostMessage...", file=sys.stderr)
    # Resolve the OS-reported main-window HWND via wmic; the exe's
    # DAT_00b4d4e8 pointer is off from what the OS treats as the game
    # window so clicks posted there go nowhere.
    import ctypes
    # EnumWindows + GetWindowThreadProcessId to find our pid's top-level HWND.
    user32 = ctypes.windll.user32
    hwnd_result = [0]
    def cb(h, _):
        p = ctypes.c_ulong()
        user32.GetWindowThreadProcessId(h, ctypes.byref(p))
        if p.value == pid and user32.IsWindowVisible(h):
            hwnd_result[0] = h
            return False
        return True
    EnumProc = ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
    user32.EnumWindows(EnumProc(cb), 0)
    hwnd = hwnd_result[0]
    if not hwnd:
        print("no window found", file=sys.stderr); return 4
    print(f"os hwnd = 0x{hwnd:x}", file=sys.stderr)
    r = api.click(hex(hwnd), args.x, args.y)
    print(f"click sent to hwnd={r['hwnd']}", file=sys.stderr)
    time.sleep(0.8)
    calls = api.get_calls()
    after = api.snapshot()
    print(f"captured {len(calls)} primitive calls", file=sys.stderr)

    fixture = {
        "meta": {
            "exe": "GDI",
            "screen": args.label,
            "width": before["width"], "height": before["height"],
            "pitch_pixels": before["pitch"],
            "red_mask": 0x7c00, "green_mask": 0x03e0, "blue_mask": 0x001f,
            "palette": palette,
        },
        "click": {"x": args.x, "y": args.y},
        "before_b64": before["data"],
        "after_b64": after["data"],
        "calls": calls,
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(fixture, indent=1))
    non_meta = [c for c in calls if c.get("text") or c["op"] in ("panel","rect","line","darken","restore")]
    text_calls = [c for c in calls if c["op"] in ("glyph","wrapped")]
    print(f"wrote {out} — {len(calls)} calls ({len(text_calls)} text)", file=sys.stderr)
    s.detach()

if __name__ == "__main__":
    sys.exit(main() or 0)
