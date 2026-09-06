// Frida hooks for cm0102ed.exe — exposes ALL imported data by hooking every
// path the editor uses to surface DB records:
//
//   Tier 1  — File I/O:     ReadFile / CreateFileA / SetFilePointer / SetFilePointerEx
//                            → the raw record load stream (which .dat, what offsets,
//                              what byte ranges) → lets us align frida events to
//                              rust-db records by disk offset.
//
//   Tier 2  — Controls:      SendMessageA / SendMessageW / SetWindowTextA/W
//                            → every value pushed into a list-box, combo, edit or
//                              static (LB_ADDSTRING, CB_ADDSTRING, WM_SETTEXT, ...).
//                              This is the primary channel for editor cell data.
//
//   Tier 3  — GDI paint:     TextOutA / TextOutW / ExtTextOutA / ExtTextOutW /
//                            DrawTextA / DrawTextW
//                            → every string the editor paints. Redundant with Tier 2
//                              for control text but catches custom-drawn views.
//
// Output: each hook emits a `send({...})` JSON event; attach.py writes them to
// jsonl. Fields:
//   {"t": ts_ms, "src": "TextOutA", "text": "...", "x": 12, "y": 34, "hwnd": ...}
//   {"t": ts_ms, "src": "ReadFile", "handle": ..., "off": 0x123, "len": 0x46}
//   {"t": ts_ms, "src": "CreateFile", "path": "D:\\cm0102\\Data\\index.dat", "handle": ...}
//   {"t": ts_ms, "src": "SendMessage", "msg": "LB_ADDSTRING", "hwnd": ..., "text": "..."}

'use strict';

const TS = () => Date.now();

// Read a NUL-terminated ANSI (latin-1 for our purposes) buffer safely.
function readAnsi(p, maxLen) {
    if (p.isNull()) return null;
    try { return p.readCString(maxLen || -1); }   // Frida 17 API
    catch (e) { return null; }
}
function readUtf16(p, maxLen) {
    if (p.isNull()) return null;
    try { return p.readUtf16String(maxLen || -1); }
    catch (e) { return null; }
}

// -----------------------------------------------------------------------
// Tier 1 — File I/O
// -----------------------------------------------------------------------

// Track handle → path so ReadFile events name the file they came from.
const HANDLES = new Map();

const kernel32 = 'kernel32.dll';

const pCreateFileA = Module.findExportByName(kernel32, 'CreateFileA');
if (pCreateFileA) {
    Interceptor.attach(pCreateFileA, {
        onEnter: function (args) {
            this.path = readAnsi(args[0], 260);
        },
        onLeave: function (retval) {
            const h = retval.toInt32();
            if (h !== -1 && this.path) {
                HANDLES.set(h, this.path);
                send({ t: TS(), src: 'CreateFile', path: this.path, handle: h });
            }
        }
    });
}

const pCreateFileW = Module.findExportByName(kernel32, 'CreateFileW');
if (pCreateFileW) {
    Interceptor.attach(pCreateFileW, {
        onEnter: function (args) { this.path = readUtf16(args[0], 260); },
        onLeave: function (retval) {
            const h = retval.toInt32();
            if (h !== -1 && this.path) {
                HANDLES.set(h, this.path);
                send({ t: TS(), src: 'CreateFile', path: this.path, handle: h });
            }
        }
    });
}

const pReadFile = Module.findExportByName(kernel32, 'ReadFile');
if (pReadFile) {
    Interceptor.attach(pReadFile, {
        onEnter: function (args) {
            this.handle  = args[0].toInt32();
            this.buffer  = args[1];
            this.bytesToRead = args[2].toInt32();
            this.lpBytesRead = args[3];
        },
        onLeave: function (retval) {
            if (retval.toInt32() === 0) return;   // failed
            let got = this.bytesToRead;
            if (!this.lpBytesRead.isNull()) {
                try { got = Memory.readU32(this.lpBytesRead); } catch (e) {}
            }
            const path = HANDLES.get(this.handle) || null;
            // Only log the interesting files (skip stdout/consoles/etc).
            if (path && /\.(dat|idx|cfg)$/i.test(path)) {
                send({ t: TS(), src: 'ReadFile', path, handle: this.handle,
                       len: got });
            }
        }
    });
}

const pSetFilePointer = Module.findExportByName(kernel32, 'SetFilePointer');
if (pSetFilePointer) {
    Interceptor.attach(pSetFilePointer, {
        onEnter: function (args) {
            this.handle = args[0].toInt32();
            this.dist   = args[1].toInt32();
            this.method = args[3].toInt32();  // 0=BEGIN, 1=CURRENT, 2=END
        },
        onLeave: function (retval) {
            const off = retval.toInt32();
            const path = HANDLES.get(this.handle);
            if (path && /\.(dat|idx|cfg)$/i.test(path)) {
                send({ t: TS(), src: 'SetFilePointer', path, handle: this.handle,
                       dist: this.dist, method: this.method, off });
            }
        }
    });
}

// -----------------------------------------------------------------------
// Tier 2 — Windows control APIs
// -----------------------------------------------------------------------

const user32 = 'user32.dll';

// Message numbers for the control APIs we care about. See winuser.h.
const MSG_NAMES = {
    0x000C: 'WM_SETTEXT',
    0x000D: 'WM_GETTEXT',
    0x0180: 'LB_ADDSTRING',
    0x0181: 'LB_INSERTSTRING',
    0x0182: 'LB_DELETESTRING',
    0x018C: 'LB_SETCURSEL',
    0x0143: 'CB_ADDSTRING',
    0x014A: 'CB_INSERTSTRING',
    0x1004: 'LVM_INSERTITEMA',   // ListView (Common Controls)
    0x104D: 'LVM_INSERTITEMW',
    0x1006: 'LVM_SETITEMA',
    0x104C: 'LVM_SETITEMW',
    0x1200: 'TVM_INSERTITEMA',   // TreeView
    0x1232: 'TVM_INSERTITEMW',
};

function logMsg(hwnd, msg, wp, lp) {
    const name = MSG_NAMES[msg];
    if (!name) return;
    let text = null;
    // WM_SETTEXT + LB_ADDSTRING/CB_ADDSTRING carry a string ptr in lparam.
    if (msg === 0x000C || msg === 0x0180 || msg === 0x0181 ||
        msg === 0x0143 || msg === 0x014A) {
        text = readAnsi(lp, 512);
    }
    // LB_INSERTSTRING has index in wparam.
    const ev = {
        t: TS(), src: 'SendMessage', msg: name,
        hwnd: hwnd.toString(), wp: wp.toInt32(), lp: lp.toString()
    };
    if (text !== null) ev.text = text;
    send(ev);
}

const pSendMessageA = Module.findExportByName(user32, 'SendMessageA');
if (pSendMessageA) {
    Interceptor.attach(pSendMessageA, {
        onEnter: function (args) {
            logMsg(args[0], args[1].toInt32(), args[2], args[3]);
        }
    });
}
const pSendMessageW = Module.findExportByName(user32, 'SendMessageW');
if (pSendMessageW) {
    Interceptor.attach(pSendMessageW, {
        onEnter: function (args) {
            const msg = args[1].toInt32();
            const name = MSG_NAMES[msg];
            if (!name) return;
            let text = null;
            if (msg === 0x000C || msg === 0x0180 || msg === 0x0181 ||
                msg === 0x0143 || msg === 0x014A) {
                text = readUtf16(args[3], 512);
            }
            const ev = {
                t: TS(), src: 'SendMessage', msg: name,
                hwnd: args[0].toString(), wp: args[2].toInt32(),
                lp: args[3].toString()
            };
            if (text !== null) ev.text = text;
            send(ev);
        }
    });
}

const pSetWindowTextA = Module.findExportByName(user32, 'SetWindowTextA');
if (pSetWindowTextA) {
    Interceptor.attach(pSetWindowTextA, {
        onEnter: function (args) {
            send({ t: TS(), src: 'SetWindowText',
                   hwnd: args[0].toString(),
                   text: readAnsi(args[1], 512) });
        }
    });
}
const pSetWindowTextW = Module.findExportByName(user32, 'SetWindowTextW');
if (pSetWindowTextW) {
    Interceptor.attach(pSetWindowTextW, {
        onEnter: function (args) {
            send({ t: TS(), src: 'SetWindowText',
                   hwnd: args[0].toString(),
                   text: readUtf16(args[1], 512) });
        }
    });
}

// -----------------------------------------------------------------------
// Tier 3 — GDI paint
// -----------------------------------------------------------------------

const gdi32 = 'gdi32.dll';

const pTextOutA = Module.findExportByName(gdi32, 'TextOutA');
if (pTextOutA) {
    Interceptor.attach(pTextOutA, {
        onEnter: function (args) {
            const n = args[3].toInt32();
            let text = null;
            try { text = Memory.readAnsiString(args[2], n); } catch (e) {}
            if (text !== null) {
                send({ t: TS(), src: 'TextOutA',
                       hdc: args[0].toString(),
                       x: args[1].toInt32(), y: args[2].toInt32(),
                       text });
            }
        }
    });
}
const pTextOutW = Module.findExportByName(gdi32, 'TextOutW');
if (pTextOutW) {
    Interceptor.attach(pTextOutW, {
        onEnter: function (args) {
            const n = args[4].toInt32();
            let text = null;
            try { text = Memory.readUtf16String(args[3], n); } catch (e) {}
            if (text !== null) {
                send({ t: TS(), src: 'TextOutW',
                       hdc: args[0].toString(),
                       x: args[1].toInt32(), y: args[2].toInt32(),
                       text });
            }
        }
    });
}

const pExtTextOutA = Module.findExportByName(gdi32, 'ExtTextOutA');
if (pExtTextOutA) {
    Interceptor.attach(pExtTextOutA, {
        onEnter: function (args) {
            const n = args[6].toInt32();
            let text = null;
            try { text = Memory.readAnsiString(args[5], n); } catch (e) {}
            if (text !== null) {
                send({ t: TS(), src: 'ExtTextOutA',
                       hdc: args[0].toString(),
                       x: args[1].toInt32(), y: args[2].toInt32(),
                       text });
            }
        }
    });
}
const pExtTextOutW = Module.findExportByName(gdi32, 'ExtTextOutW');
if (pExtTextOutW) {
    Interceptor.attach(pExtTextOutW, {
        onEnter: function (args) {
            const n = args[6].toInt32();
            let text = null;
            try { text = Memory.readUtf16String(args[5], n); } catch (e) {}
            if (text !== null) {
                send({ t: TS(), src: 'ExtTextOutW',
                       hdc: args[0].toString(),
                       x: args[1].toInt32(), y: args[2].toInt32(),
                       text });
            }
        }
    });
}

// DrawText — used by common controls for owner-drawn cells.
const pDrawTextA = Module.findExportByName(user32, 'DrawTextA');
if (pDrawTextA) {
    Interceptor.attach(pDrawTextA, {
        onEnter: function (args) {
            const n = args[2].toInt32();
            const text = readAnsi(args[1], n < 0 ? 512 : n);
            if (text) {
                send({ t: TS(), src: 'DrawTextA',
                       hdc: args[0].toString(), text });
            }
        }
    });
}
const pDrawTextW = Module.findExportByName(user32, 'DrawTextW');
if (pDrawTextW) {
    Interceptor.attach(pDrawTextW, {
        onEnter: function (args) {
            const n = args[2].toInt32();
            const text = readUtf16(args[1], n < 0 ? 512 : n);
            if (text) {
                send({ t: TS(), src: 'DrawTextW',
                       hdc: args[0].toString(), text });
            }
        }
    });
}

send({ t: TS(), src: 'hook_ready',
       hooks: {
           CreateFileA: !!pCreateFileA, CreateFileW: !!pCreateFileW,
           ReadFile:    !!pReadFile,    SetFilePointer: !!pSetFilePointer,
           SendMessageA:!!pSendMessageA, SendMessageW: !!pSendMessageW,
           SetWindowTextA:!!pSetWindowTextA, SetWindowTextW: !!pSetWindowTextW,
           TextOutA:    !!pTextOutA,    TextOutW:    !!pTextOutW,
           ExtTextOutA: !!pExtTextOutA, ExtTextOutW: !!pExtTextOutW,
           DrawTextA:   !!pDrawTextA,   DrawTextW:   !!pDrawTextW,
       }});
