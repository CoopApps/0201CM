// Frida agent for cm0102ed.exe — capture (a) raw .dat bytes as they load, and
// (b) every value that shows up in a Delphi control, so we can correlate
// (field_name shown) ↔ (bytes at that offset in the record) offline.
//
// Hook layer A: ReadFile — every time the editor reads a chunk of a data file,
//   we capture (file_handle, filename, offset, size, first N bytes).
// Hook layer B: SetWindowTextA/W — every time a Delphi control's text updates,
//   we capture (hwnd, class_name, control_name, text). This fires for every
//   TEdit / TLabel / TComboBox item change.
//
// The two streams together let a Python correlator identify:
//   "when SetWindowText(hwnd=X, text='6000') fired, the last club.dat read
//    contained the bytes for club 8371 (SWFC); scanning them for the value
//    6000 finds it at +0x80 → club_reputation is at record+0x80"
//
// Emits one JSON per event via send(). No page allocation, no expensive
// Interceptor tracing — just Native hooks with tight callbacks.

(function() {
    'use strict';

    const k32 = Module.load('kernel32.dll');
    const u32 = Module.load('user32.dll');
    const p_ReadFile = k32.getExportByName('ReadFile');
    const p_SetFilePointerEx = k32.getExportByName('SetFilePointerEx');
    const p_CreateFileW = k32.getExportByName('CreateFileW');
    const p_CreateFileA = k32.getExportByName('CreateFileA');
    const p_CreateFileMappingA = k32.getExportByName('CreateFileMappingA');
    const p_CreateFileMappingW = k32.getExportByName('CreateFileMappingW');
    const p_MapViewOfFile = k32.getExportByName('MapViewOfFile');
    const p_MapViewOfFileEx = k32.getExportByName('MapViewOfFileEx');
    const p_UnmapViewOfFile = k32.getExportByName('UnmapViewOfFile');
    const p_GetFinalPathNameByHandleW = k32.getExportByName('GetFinalPathNameByHandleW');
    const p_SetWindowTextA = u32.getExportByName('SetWindowTextA');
    const p_SetWindowTextW = u32.getExportByName('SetWindowTextW');
    const p_SendMessageA   = u32.getExportByName('SendMessageA');
    const p_SendMessageW   = u32.getExportByName('SendMessageW');
    const p_GetClassNameA = u32.getExportByName('GetClassNameA');
    const p_GetWindowTextA = u32.getExportByName('GetWindowTextA');

    const NativeGetFinalPathNameByHandleW = new NativeFunction(
        p_GetFinalPathNameByHandleW, 'uint32', ['pointer','pointer','uint32','uint32']);
    const NativeGetClassNameA = new NativeFunction(
        p_GetClassNameA, 'int32', ['pointer','pointer','int32']);
    const NativeGetWindowTextA = new NativeFunction(
        p_GetWindowTextA, 'int32', ['pointer','pointer','int32']);

    // Filename cache — we only care about .dat files.
    const filenameFor = new Map();
    function nameOfHandle(h) {
        const key = h.toString();
        if (filenameFor.has(key)) return filenameFor.get(key);
        const buf = Memory.alloc(1024);
        const n = NativeGetFinalPathNameByHandleW(h, buf, 512, 0);
        let name = null;
        if (n > 0 && n < 512) {
            name = buf.readUtf16String();
        }
        filenameFor.set(key, name);
        return name;
    }

    // Track the current file position per handle (approximate — we increment
    // by the byte count each successful ReadFile returns).
    const posFor = new Map();
    // Track destination memory ranges per file. For each .dat we watch, record
    // every (buf_addr, size, file_off) so we can later dump the RECEIVING
    // memory as a contiguous ground-truth snapshot — no UI walking needed.
    // Key = file basename (lowercased). Value = array of {addr, size, foff}.
    const writesFor = new Map();

    // Hook A: ReadFile(hFile, buf, size, &nread, &overlapped)
    Interceptor.attach(p_ReadFile, {
        onEnter: function(args) {
            this.h = args[0];
            this.buf = args[1];
            this.size = args[2].toInt32();
            this.pn = args[3];
        },
        onLeave: function(retval) {
            if (retval.toInt32() === 0) return;
            const name = nameOfHandle(this.h);
            if (!name || name.toLowerCase().indexOf('.dat') < 0) return;
            const n = this.pn.isNull() ? this.size : this.pn.readU32();
            if (n === 0) return;
            const key = this.h.toString();
            const off = posFor.get(key) || 0;
            posFor.set(key, off + n);
            // Cap bytes shipped to Python at 4096 per event to keep logs sane.
            const shipped = Math.min(n, 4096);
            let hex = '';
            try {
                const bytes = this.buf.readByteArray(shipped);
                const u = new Uint8Array(bytes);
                const chars = [];
                for (let i = 0; i < u.length; i++) chars.push(u[i].toString(16).padStart(2,'0'));
                hex = chars.join('');
            } catch (_) {}
            // Record destination buffer for post-import memory dump.
            const base = name.split(/[\\\/]/).pop().toLowerCase();
            let arr = writesFor.get(base);
            if (!arr) { arr = []; writesFor.set(base, arr); }
            arr.push({addr: this.buf.toString(), size: n, foff: off});
            send({t: 'read', file: name, off: off, n: n, hex: hex, buf: this.buf.toString()});
        }
    });

    // Hook A2: SetFilePointerEx — seeks. Update our position tracker.
    Interceptor.attach(p_SetFilePointerEx, {
        onEnter: function(args) {
            this.h = args[0];
            this.dist = args[1].toInt64(); // liDistanceToMove (i64)
            this.pnew = args[2];            // lpNewFilePointer
            this.method = args[3].toInt32();
        },
        onLeave: function(retval) {
            if (retval.toInt32() === 0) return;
            let np = 0;
            if (!this.pnew.isNull()) np = this.pnew.readU64().valueOf();
            else np = this.dist.valueOf();
            posFor.set(this.h.toString(), Number(np));
        }
    });

    // Hook A3: CreateFileW/A — reset the position cache on new open AND
    // remember the filename per handle (for MapViewOfFile correlation).
    const nameForHandle = new Map();  // fileHandle → filename
    Interceptor.attach(p_CreateFileW, {
        onEnter: function(args) { this.name = args[0].isNull() ? '' : args[0].readUtf16String(); },
        onLeave: function(retval) {
            const h = retval;
            if (h.toInt32() === -1) return;
            posFor.set(h.toString(), 0);
            if (this.name) nameForHandle.set(h.toString(), this.name);
        }
    });
    Interceptor.attach(p_CreateFileA, {
        onEnter: function(args) { this.name = args[0].isNull() ? '' : args[0].readCString(); },
        onLeave: function(retval) {
            const h = retval;
            if (h.toInt32() === -1) return;
            posFor.set(h.toString(), 0);
            if (this.name) nameForHandle.set(h.toString(), this.name);
        }
    });

    // File-mapping tracking. CreateFileMapping(h, ..., ..., ..., ..., name?)
    // returns a mapping handle. MapViewOfFile(mapping, ...) returns a memory
    // pointer. UnmapViewOfFile releases it. We chain: file → mapping → view.
    const fileForMapping = new Map();     // mappingHandle → filename
    const mappings = new Map();           // view_addr → {file, size, mapping}
    function attachCFM(fn) {
        Interceptor.attach(fn, {
            onEnter: function(args) { this.fh = args[0]; },
            onLeave: function(retval) {
                if (retval.isNull()) return;
                const fname = nameForHandle.get(this.fh.toString()) || '?';
                fileForMapping.set(retval.toString(), fname);
                send({t: 'mapping_created', file: fname, mapping: retval.toString()});
            }
        });
    }
    attachCFM(p_CreateFileMappingA);
    attachCFM(p_CreateFileMappingW);
    function attachMVO(fn) {
        Interceptor.attach(fn, {
            onEnter: function(args) { this.mh = args[0]; this.size = args[4].toInt32(); },
            onLeave: function(retval) {
                if (retval.isNull()) return;
                const fname = fileForMapping.get(this.mh.toString()) || '?';
                // If size argument was 0 (map whole file), we don't know the true
                // size here — leave 0; Python will VirtualQuery to determine.
                mappings.set(retval.toString(), {file: fname, size: this.size, mapping: this.mh.toString()});
                send({t: 'mapview', file: fname, addr: retval.toString(), size: this.size});
            }
        });
    }
    attachMVO(p_MapViewOfFile);
    attachMVO(p_MapViewOfFileEx);
    Interceptor.attach(p_UnmapViewOfFile, {
        onEnter: function(args) {
            const addr = args[0].toString();
            const info = mappings.get(addr);
            if (info) send({t: 'unmap', file: info.file, addr: addr});
            mappings.delete(addr);
        }
    });

    // Hook B: SetWindowTextA/W — capture displayed values.
    function classOf(hwnd) {
        const buf = Memory.alloc(128);
        const n = NativeGetClassNameA(hwnd, buf, 128);
        return n > 0 ? buf.readCString() : '?';
    }
    Interceptor.attach(p_SetWindowTextA, {
        onEnter: function(args) {
            const hwnd = args[0];
            const s = args[1].isNull() ? '' : args[1].readCString();
            if (!s || s.length === 0) return;
            const cls = classOf(hwnd);
            send({t: 'text', hwnd: hwnd.toInt32(), cls: cls, s: s});
        }
    });
    Interceptor.attach(p_SetWindowTextW, {
        onEnter: function(args) {
            const hwnd = args[0];
            const s = args[1].isNull() ? '' : args[1].readUtf16String();
            if (!s || s.length === 0) return;
            const cls = classOf(hwnd);
            send({t: 'text', hwnd: hwnd.toInt32(), cls: cls, s: s});
        }
    });
    // Delphi VCL controls often update text via SendMessage(WM_SETTEXT). Catch
    // that path too (WM_SETTEXT = 0x000C).
    const WM_SETTEXT = 0x000C;
    Interceptor.attach(p_SendMessageA, {
        onEnter: function(args) {
            if (args[1].toInt32() !== WM_SETTEXT) return;
            const hwnd = args[0]; const p = args[3];
            if (p.isNull()) return;
            let s = ''; try { s = p.readCString(); } catch (_) { return; }
            if (!s || s.length === 0 || s.length > 200) return;
            send({t: 'text', hwnd: hwnd.toInt32(), cls: classOf(hwnd), s: s, src: 'wmA'});
        }
    });
    Interceptor.attach(p_SendMessageW, {
        onEnter: function(args) {
            if (args[1].toInt32() !== WM_SETTEXT) return;
            const hwnd = args[0]; const p = args[3];
            if (p.isNull()) return;
            let s = ''; try { s = p.readUtf16String(); } catch (_) { return; }
            if (!s || s.length === 0 || s.length > 200) return;
            send({t: 'text', hwnd: hwnd.toInt32(), cls: classOf(hwnd), s: s, src: 'wmW'});
        }
    });

    // --- Auto-drive: post keystrokes to a target window without touching the
    // user's own keyboard or focus. Uses PostMessage(WM_KEYDOWN/UP) to the
    // hwnd Python discovers. VK_DOWN(0x28) / VK_NEXT(0x22 = PgDown) etc.
    const p_PostMessageA = u32.getExportByName('PostMessageA');
    const p_GetActiveWindow = u32.getExportByName('GetActiveWindow');
    const p_EnumWindows = u32.getExportByName('EnumWindows');
    const p_GetWindowThreadProcessId = u32.getExportByName('GetWindowThreadProcessId');
    const p_GetCurrentProcessId = k32.getExportByName('GetCurrentProcessId');
    const p_IsWindowVisible = u32.getExportByName('IsWindowVisible');
    const p_GetWindow = u32.getExportByName('GetWindow');
    const p_GetForegroundWindow = u32.getExportByName('GetForegroundWindow');

    const NativePostMessageA = new NativeFunction(p_PostMessageA, 'int32',
        ['pointer', 'uint32', 'pointer', 'pointer']);
    const NativeGetForegroundWindow = new NativeFunction(p_GetForegroundWindow, 'pointer', []);
    const NativeGetActiveWindow = new NativeFunction(p_GetActiveWindow, 'pointer', []);

    const WM_KEYDOWN = 0x0100, WM_KEYUP = 0x0101, WM_COMMAND = 0x0111;

    function postKey(hwnd, vk) {
        NativePostMessageA(hwnd, WM_KEYDOWN, ptr(vk), ptr(0));
        NativePostMessageA(hwnd, WM_KEYUP, ptr(vk), ptr(0xC0000001));
    }

    // Expose RPC callable from Python.
    rpc.exports = {
        listWindows: function() {
            const NativeEnumWindows = new NativeFunction(p_EnumWindows, 'int32', ['pointer','pointer']);
            const NativeGetWindowThreadPid = new NativeFunction(
                p_GetWindowThreadProcessId, 'uint32', ['pointer','pointer']);
            const NativeGetCurPid = new NativeFunction(p_GetCurrentProcessId, 'uint32', []);
            const NativeIsVisible = new NativeFunction(p_IsWindowVisible, 'int32', ['pointer']);
            const myPid = NativeGetCurPid();
            const results = [];
            const pidBuf = Memory.alloc(4);
            const clsBuf = Memory.alloc(128);
            const txtBuf = Memory.alloc(512);
            const cb = new NativeCallback(function(hwnd, _lp) {
                NativeGetWindowThreadPid(hwnd, pidBuf);
                if (pidBuf.readU32() !== myPid) return 1;
                if (NativeIsVisible(hwnd) === 0) return 1;
                NativeGetClassNameA(hwnd, clsBuf, 128);
                NativeGetWindowTextA(hwnd, txtBuf, 512);
                results.push({
                    hwnd: hwnd.toInt32(),
                    cls: clsBuf.readCString(),
                    title: txtBuf.readCString(),
                });
                return 1;
            }, 'int32', ['pointer','pointer']);
            NativeEnumWindows(cb, ptr(0));
            return results;
        },

        // Post `count` VK_DOWN presses to `hwnd`, one every `intervalMs`.
        // We schedule them with setTimeout so JS returns immediately.
        driveDown: function(hwnd, count, intervalMs) {
            const w = ptr(hwnd);
            let i = 0;
            function step() {
                if (i >= count) { send({t: 'drive_done', pressed: count}); return; }
                postKey(w, 0x28); // VK_DOWN
                i++;
                setTimeout(step, intervalMs);
            }
            step();
            return true;
        },

        // Post an arbitrary VK sequence — for scripted navigation.
        driveKeys: function(hwnd, vks, intervalMs) {
            const w = ptr(hwnd);
            let i = 0;
            function step() {
                if (i >= vks.length) { send({t: 'drive_done', pressed: vks.length}); return; }
                postKey(w, vks[i]); i++;
                setTimeout(step, intervalMs);
            }
            step(); return true;
        },

        // Post a WM_COMMAND (menu-item click) to hwnd.
        postCommand: function(hwnd, cmdId) {
            NativePostMessageA(ptr(hwnd), WM_COMMAND, ptr(cmdId), ptr(0));
            return true;
        },

        // Find a byte pattern in read-writable regions (for locating record
        // arrays by known content, e.g. "Sheffield Wednesday" as a Pascal
        // string). Returns first N hits with base addresses.
        findBytes: function(hexPattern, limit) {
            limit = limit || 8;
            const hits = [];
            Process.enumerateRanges({protection: 'r--', coalesce: true}).forEach(function(r) {
                if (hits.length >= limit) return;
                try {
                    const found = Memory.scanSync(r.base, r.size, hexPattern);
                    for (let i = 0; i < found.length && hits.length < limit; i++) {
                        hits.push({addr: found[i].address.toString(), size: found[i].size});
                    }
                } catch (_) {}
            });
            return hits;
        },

        // Dump `size` bytes at address `addrHex` as hex — for enumerating a
        // known record array by walking (base, base+stride, base+2*stride, …).
        readMem: function(addrHex, size) {
            try {
                const bytes = ptr(addrHex).readByteArray(size);
                const u = new Uint8Array(bytes);
                const chars = [];
                for (let i = 0; i < u.length; i++) chars.push(u[i].toString(16).padStart(2,'0'));
                return chars.join('');
            } catch (e) {
                return null;
            }
        },

        // Return a summary of what memory each .dat file was loaded into.
        // For each file: total bytes read, min/max addr, distinct buffer bases.
        // Python then uses this to dump each contiguous memory region back out
        // to disk — a byte-exact snapshot of the editor's in-memory records
        // without any UI interaction.
        loadedFiles: function() {
            const out = {};
            writesFor.forEach(function(arr, name) {
                let total = 0, minA = null, maxA = null;
                const bufs = {};
                for (let i = 0; i < arr.length; i++) {
                    const a = parseInt(arr[i].addr, 16);
                    const e = a + arr[i].size;
                    total += arr[i].size;
                    if (minA === null || a < minA) minA = a;
                    if (maxA === null || e > maxA) maxA = e;
                    // Bin by 64KB "region" to spot contiguous vs fragmented allocs.
                    const region = (a >>> 16) << 16;
                    bufs[region] = (bufs[region] || 0) + arr[i].size;
                }
                out[name] = {
                    total_bytes: total, min_addr: minA, max_addr: maxA,
                    span: (maxA - minA) || 0,
                    read_count: arr.length,
                    region_buckets: bufs,  // { hex_region_base: byte_count }
                };
            });
            return out;
        },

        // List every live file mapping this process holds. For each, try to
        // determine the mapped size via Process.findRangeByAddress. Result:
        //   [{file, addr, size, protection}, ...]
        listMappings: function() {
            const out = [];
            mappings.forEach(function(info, addrHex) {
                let realSize = info.size;
                let prot = '?';
                try {
                    const r = Process.findRangeByAddress(ptr(addrHex));
                    if (r) { realSize = realSize || r.size; prot = r.protection; }
                } catch (_) {}
                out.push({file: info.file, addr: addrHex, size: realSize, prot: prot});
            });
            return out;
        },

        // Given a base address + size, dump raw bytes to a big hex string.
        // Split calls if size > 512KB to avoid single-message blowup.
        dumpRegion: function(addrHex, size) {
            try {
                const bytes = ptr(addrHex).readByteArray(size);
                const u = new Uint8Array(bytes);
                let hex = '';
                for (let i = 0; i < u.length; i++) hex += u[i].toString(16).padStart(2,'0');
                return hex;
            } catch (e) {
                return null;
            }
        },
    };

    send({t: 'hello', msg: 'cm0102ed field-capture agent installed', rpc: ['listWindows','driveDown','driveKeys','postCommand','findBytes','readMem']});
})();
