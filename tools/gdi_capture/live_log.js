// Streaming primitive-call logger — attaches, logs every call as it
// happens with `send()`, exits when the driver detaches. Companion to
// live_log.py which prints and counts the events.
//
// Format upgrade (News Milestone C): at every PRESENT-boundary hook we
// ALSO snapshot the live framebuffer as a base64-encoded raw u16 LE
// stream and emit a distinct `present_fb` event. The pre-existing
// `--- PRESENT ---` marker is still emitted first so older analysers
// keep working. See tools/gdi_capture/live_log.py --capture-framebuffers.

'use strict';

// Base64 encoder — pure JS so the whole snapshot ships inside one
// `send()` payload (no data-channel bookkeeping). Same table + edge
// handling as capture_screen.py.
const B64='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
function b64(bytes) {
    const v = new Uint8Array(bytes), n = v.length;
    const out = new Array(Math.ceil(n / 3) * 4);
    let o = 0, i = 0;
    for (; i + 2 < n; i += 3) {
        const b1 = v[i], b2 = v[i + 1], b3 = v[i + 2];
        out[o++] = B64[b1 >> 2];
        out[o++] = B64[((b1 & 3) << 4) | (b2 >> 4)];
        out[o++] = B64[((b2 & 15) << 2) | (b3 >> 6)];
        out[o++] = B64[b3 & 63];
    }
    if (i < n) {
        const b1 = v[i], b2 = (i + 1 < n) ? v[i + 1] : 0;
        out[o++] = B64[b1 >> 2];
        out[o++] = B64[((b1 & 3) << 4) | (b2 >> 4)];
        if (i + 1 < n) { out[o++] = B64[(b2 & 15) << 2]; out[o++] = '='; }
        else { out[o++] = '='; out[o++] = '='; }
    }
    return out.join('');
}

// Snapshot the live u16 framebuffer via the same globals capture.js and
// capture_screen.py read.
function snapshotFramebuffer() {
    try {
        const buf = ptr('0x00ad6b1c').readPointer();
        const w = ptr('0x00ad6b40').readS32();
        const h = ptr('0x00ad6b08').readS32();
        const pitch = ptr('0x00acdeb8').readU16();
        const bytes = buf.readByteArray(pitch * h * 2);
        return { width: w, height: h, pitch: pitch, data_b64: b64(bytes) };
    } catch (e) {
        return { error: e.toString() };
    }
}

// Recv toggles at runtime so the driver can decide whether to pay the
// per-present snapshot cost (each frame is ~1 MB pre-base64).
let CAPTURE_FRAMEBUFFERS = false;
recv('config', function (msg) {
    if (typeof msg.capture_framebuffers === 'boolean') {
        CAPTURE_FRAMEBUFFERS = msg.capture_framebuffers;
    }
});

const VA = {
    LINE:    ptr('0x005cd3e0'),
    RECT:    ptr('0x005cd730'),
    DARKEN:  ptr('0x005cdd60'),
    RESTORE: ptr('0x005cda90'),
    GLYPH:   ptr('0x005ceaa0'),
    PANEL:   ptr('0x005cf570'),
    WRAPPED: ptr('0x005d03a0'),
    // Present-dirty-rect — natural batch boundary. Every draw batch ends
    // with one of these (the exe's "swap the region we just changed to
    // the screen" call).
    PRESENT: ptr('0x005cccd0'),
};

function i32(a) { return a.toInt32(); }
function u32(a) { return a.toUInt32(); }
function u16(a) { return a.toUInt32() & 0xffff; }

function readStr(p) {
    // readCString returns junk after the first non-NUL boundary when the
    // exe's text buffer isn't NUL-terminated within 256 bytes (some
    // callsites reuse a shared scratch buffer). Read as raw bytes and
    // slice at the first 0x00 ourselves — matches the C string convention
    // the exe's own draw fns follow.
    if (p.isNull()) return '';
    try {
        const bytes = p.readByteArray(256);
        if (!bytes) return '';
        const arr = new Uint8Array(bytes);
        let end = 0;
        while (end < arr.length && arr[end] !== 0) end++;
        return String.fromCharCode.apply(null, arr.subarray(0, end));
    } catch (_) { return ''; }
}

// Return-address in the caller's code — read from ESP on entry. Gives
// the exact instruction the primitive was called FROM, which is worth
// more than the primitive VA alone (which we already know).
function caller(ctx) {
    try { return ctx.esp.readPointer().sub(Process.getModuleByName('cm0102_GDI.exe').base).toString(); }
    catch (_) { return '?'; }
}

Interceptor.attach(VA.LINE, {
    onEnter(a) {
        send({ op: 'line', from: caller(this.context),
               x0: i32(a[0]), y0: i32(a[1]), x1: i32(a[2]), y1: i32(a[3]),
               style: u32(a[4]), colour: u16(a[5]) });
    },
});
Interceptor.attach(VA.RECT, {
    onEnter(a) {
        send({ op: 'rect', from: caller(this.context),
               x0: i32(a[0]), y0: i32(a[1]), x1: i32(a[2]), y1: i32(a[3]),
               style: u32(a[4]), colour: u16(a[5]) });
    },
});
Interceptor.attach(VA.DARKEN, {
    onEnter(a) {
        send({ op: 'darken', from: caller(this.context),
               x0: i32(a[0]), y0: i32(a[1]), x1: i32(a[2]), y1: i32(a[3]) });
    },
});
Interceptor.attach(VA.RESTORE, {
    onEnter(a) {
        send({ op: 'restore', from: caller(this.context),
               x: i32(a[0]), y: i32(a[1]) });
    },
});
Interceptor.attach(VA.GLYPH, {
    onEnter(a) {
        send({ op: 'glyph', from: caller(this.context),
               x: i32(a[0]), y: i32(a[1]), font: u16(a[2]), colour: u16(a[3]),
               text: readStr(a[4]) });
    },
});
Interceptor.attach(VA.PANEL, {
    onEnter(a) {
        send({ op: 'panel', from: caller(this.context),
               x0: i32(a[0]), y0: i32(a[1]), x1: i32(a[2]), y1: i32(a[3]),
               style: u32(a[4]).toString(16), colour: u16(a[5]), pattern: u16(a[6]) });
    },
});
Interceptor.attach(VA.WRAPPED, {
    onEnter(a) {
        send({ op: 'wrapped', from: caller(this.context),
               x0: i32(a[0]), y0: i32(a[1]), x1: i32(a[2]), y1: i32(a[3]),
               style: u32(a[4]).toString(16), font: u16(a[5]), colour: u16(a[6]),
               text: readStr(a[7]) });
    },
});
Interceptor.attach(VA.PRESENT, {
    onEnter(a) {
        // Marker first — old analysers key on this exact string.
        send({ op: '--- PRESENT ---',
               x0: i32(a[0]), y0: i32(a[1]), x1: i32(a[2]), y1: i32(a[3]) });
        // Then, only if the driver armed it, the fat framebuffer event.
        // Distinct op name ('present_fb') so old tools that don't know
        // about it treat it as an unrelated primitive and skip.
        if (CAPTURE_FRAMEBUFFERS) {
            const fb = snapshotFramebuffer();
            send(Object.assign({ op: 'present_fb' }, fb));
        }
    },
});

send({ ready: true, hooks: Object.keys(VA).length });
