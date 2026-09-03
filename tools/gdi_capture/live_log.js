// Streaming primitive-call logger — attaches, logs every call as it
// happens with `send()`, exits when the driver detaches. Companion to
// live_log.py which prints and counts the events. Deliberately narrower
// than capture.js (no fixture accumulation, no framebuffer snapshots)
// to keep the observation load tiny.

'use strict';

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
               style: u32(a[4]).toString(16), colour: u16(a[5]) });
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
        send({ op: '--- PRESENT ---',
               x0: i32(a[0]), y0: i32(a[1]), x1: i32(a[2]), y1: i32(a[3]) });
    },
});

send({ ready: true, hooks: Object.keys(VA).length });
