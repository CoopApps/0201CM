// Frida hook — cm0102_GDI.exe primitive-call capture.
//
// Attaches to a running cm0102_GDI.exe, hooks every primitive the packed
// pipeline replays (line/rect/darken/save/restore/glyph/panel/wrapped-
// text), and emits one fixture JSON per "capture window". A capture
// window is opened by sending {type:'start', screen:'name'} to send()
// and closed by {type:'stop'}; between the two, every primitive call is
// logged and the framebuffer is snapshotted at start ("before") and
// stop ("after").
//
// The fixture format is the one crates/cm-render/src/packed_capture.rs
// declares — the runner script at run_capture.py pumps fixtures out of
// stdout and writes them to disk for replay-and-byte-diff testing.
//
// IMPORTANT: heavy Frida hooking crashed the original exe twice (see
// [[frida-instrumentation-crash-evidence]]). Keep this script minimal:
// only the ~14 primitive VAs, no per-instruction stalking, no memory
// scans in a tight loop.

'use strict';

// -------- exe globals (from FUN_005cc4f0 init + the mask writes) --------
const GLOBALS = {
    BUF:        ptr('0x00ad6b1c'), // DAT_00ad6b1c — pointer to backbuffer
    PITCH:      ptr('0x00acdeb8'), // DAT_00acdeb8 — stride (u16 pixels)
    WIDTH:      ptr('0x00ad6b40'), // DAT_00ad6b40 — width
    HEIGHT:     ptr('0x00ad6b08'), // DAT_00ad6b08 — height
    RED_MASK:   ptr('0x00acdea8'),
    GREEN_MASK: ptr('0x00acdeac'),
    BLUE_MASK:  ptr('0x00acdeb0'),
};

// -------- primitive VAs (GDI variant, from gdi-renderer-is-ground-truth) --
const VA = {
    LINE:     ptr('0x005cd3e0'), // FUN_005cd3e0(x0,y0,x1,y1,style,colour)
    RECT:     ptr('0x005cd730'), // FUN_005cd730(x0,y0,x1,y1,style,colour)
    DARKEN:   ptr('0x005cdd60'), // FUN_005cdd60(x0,y0,x1,y1)
    SAVE:     ptr('0x005cd930'), // FUN_005cd930 — save-rect (state)
    RESTORE:  ptr('0x005cda90'), // FUN_005cda90(x, y, saved)
    GLYPH:    ptr('0x005ceaa0'), // FUN_005ceaa0(x, y, font, colour, string, underline_at)
    PANEL:    ptr('0x005cf570'), // FUN_005cf570(x0,y0,x1,y1,style,colour[,pattern])
    WRAPPED:  ptr('0x005d03a0'), // FUN_005d03a0(x0,y0,x1,y1,style,font,colour,string,kern)
};

let capturing = false;
let currentFixture = null;

// Read the current framebuffer as base64 raw little-endian u16 stream.
function snapshotFramebuffer() {
    const bufPtr = GLOBALS.BUF.readPointer();
    if (bufPtr.isNull()) return null;
    const pitch = GLOBALS.PITCH.readU16();
    const height = GLOBALS.HEIGHT.readS32();
    const bytes = bufPtr.readByteArray(pitch * height * 2);
    return b64encode(bytes);
}

function readMeta(screen) {
    return {
        exe: 'GDI',
        screen: screen,
        width: GLOBALS.WIDTH.readS32(),
        height: GLOBALS.HEIGHT.readS32(),
        pitch_pixels: GLOBALS.PITCH.readU16(),
        red_mask: GLOBALS.RED_MASK.readU16(),
        green_mask: GLOBALS.GREEN_MASK.readU16(),
        blue_mask: GLOBALS.BLUE_MASK.readU16(),
    };
}

// Minimal base64 encoder — Frida's Node buffer isn't available.
function b64encode(bytes) {
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    const view = new Uint8Array(bytes);
    let out = '';
    let i = 0;
    for (; i + 2 < view.length; i += 3) {
        const b1 = view[i], b2 = view[i + 1], b3 = view[i + 2];
        out += alphabet[b1 >> 2];
        out += alphabet[((b1 & 3) << 4) | (b2 >> 4)];
        out += alphabet[((b2 & 15) << 2) | (b3 >> 6)];
        out += alphabet[b3 & 63];
    }
    if (i < view.length) {
        const b1 = view[i];
        const b2 = i + 1 < view.length ? view[i + 1] : 0;
        out += alphabet[b1 >> 2];
        out += alphabet[((b1 & 3) << 4) | (b2 >> 4)];
        if (i + 1 < view.length) {
            out += alphabet[(b2 & 15) << 2];
            out += '=';
        } else {
            out += '==';
        }
    }
    return out;
}

function readAscii(ptr_) {
    if (ptr_.isNull()) return '';
    return ptr_.readCString();
}

// stdcall/cdecl on x86: args are on the stack; ESP+0 = return address,
// ESP+4 = arg1, ESP+8 = arg2, ... (`this.context.esp.add(4 + 4*n)`).
function argI32(args, n) { return args[n].toInt32(); }
function argU32(args, n) { return args[n].toUInt32(); }
function argU16(args, n) { return args[n].toUInt32() & 0xffff; }

Interceptor.attach(VA.LINE, {
    onEnter(args) {
        if (!capturing) return;
        currentFixture.calls.push({
            op: 'line',
            x0: argI32(args, 0), y0: argI32(args, 1),
            x1: argI32(args, 2), y1: argI32(args, 3),
            style: argU32(args, 4), colour: argU16(args, 5),
        });
    },
});

Interceptor.attach(VA.RECT, {
    onEnter(args) {
        if (!capturing) return;
        currentFixture.calls.push({
            op: 'rect',
            x0: argI32(args, 0), y0: argI32(args, 1),
            x1: argI32(args, 2), y1: argI32(args, 3),
            style: argU32(args, 4), colour: argU16(args, 5),
        });
    },
});

Interceptor.attach(VA.DARKEN, {
    onEnter(args) {
        if (!capturing) return;
        currentFixture.calls.push({
            op: 'darken',
            x0: argI32(args, 0), y0: argI32(args, 1),
            x1: argI32(args, 2), y1: argI32(args, 3),
        });
    },
});

Interceptor.attach(VA.PANEL, {
    onEnter(args) {
        if (!capturing) return;
        currentFixture.calls.push({
            op: 'panel',
            x0: argI32(args, 0), y0: argI32(args, 1),
            x1: argI32(args, 2), y1: argI32(args, 3),
            style: argU32(args, 4), colour: argU16(args, 5),
            outer_highlight: 0, default_bevel: 0, // populated from palette globals later
        });
    },
});

Interceptor.attach(VA.RESTORE, {
    onEnter(args) {
        if (!capturing) return;
        const savedPtr = args[2];
        const width  = savedPtr.readInt();
        const height = savedPtr.add(4).readInt();
        const dataPtr = savedPtr.add(12).readPointer();
        const bytes = dataPtr.readByteArray(width * height * 2);
        currentFixture.calls.push({
            op: 'restore',
            x: argI32(args, 0), y: argI32(args, 1),
            width, height,
            data_b64: b64encode(bytes),
        });
    },
});

// Text hooks capture the string but NOT the font contents (the font
// tables at DAT_00accae4 + font*0x1404 are ~5 KB each and 8 fonts total
// — the runner assembles a font sidecar the first time a font is seen
// and rewrites its reference into the fixture before persisting).
Interceptor.attach(VA.GLYPH, {
    onEnter(args) {
        if (!capturing) return;
        currentFixture.calls.push({
            op: 'text_pending_font',
            x: argI32(args, 0), y: argI32(args, 1),
            font_idx: argU16(args, 2), colour: argU16(args, 3),
            text: readAscii(args[4]),
            underline_at: argI32(args, 5),
        });
    },
});

Interceptor.attach(VA.WRAPPED, {
    onEnter(args) {
        if (!capturing) return;
        currentFixture.calls.push({
            op: 'wrapped_text_pending_font',
            x0: argI32(args, 0), y0: argI32(args, 1),
            x1: argI32(args, 2), y1: argI32(args, 3),
            style: argU32(args, 4),
            font_idx: argU16(args, 5),
            colour: argU16(args, 6),
            text: readAscii(args[7]),
            kern: argI32(args, 8),
        });
    },
});

rpc.exports = {
    start(screen) {
        if (capturing) return { ok: false, error: 'already capturing' };
        currentFixture = {
            meta: readMeta(screen),
            before_b64: snapshotFramebuffer(),
            after_b64: null,
            calls: [],
        };
        capturing = true;
        return { ok: true };
    },
    stop() {
        if (!capturing) return { ok: false, error: 'not capturing' };
        currentFixture.after_b64 = snapshotFramebuffer();
        capturing = false;
        const f = currentFixture;
        currentFixture = null;
        return { ok: true, fixture: f };
    },
    // Read a font table (5124 bytes) at DAT_00accae4 + font_idx*0x1404 —
    // the runner side calls this once per font_idx it sees in *_pending_font
    // calls, then rewrites those into `text`/`wrapped_text` with an inlined
    // FontDump.
    readFontTable(idx) {
        const base = ptr('0x00accae4').add(idx * 0x1404);
        return b64encode(base.readByteArray(0x1404));
    },
};

send({ type: 'ready', hooks: Object.keys(VA).length });
