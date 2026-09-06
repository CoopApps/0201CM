// Snapshot the editor's runtime memory image.
//
// Rationale: cm0102ed.exe loads index.dat + all Data/*.dat files into
// contiguous in-memory pools at fixed static offsets when it starts.
// Once the load is complete, the pools are just sitting there. If we
// walk every RW committed page and dump it, every record from every
// pool ends up in one blob. Offline we then search for known
// signatures ("1.FC Bocholt", "Blue 3", "Chester City", the shipped
// Data/*.dat file bytes, ...) to pinpoint each pool's base, stride and
// length.
//
// Two useful commands (RPC-exposed to Python):
//   list_ranges()                → JSON list of {base, size, protection}
//   dump_range(base, size)       → raw bytes (Python writes to disk)
//   find_bytes(pattern_hex)      → JSON list of matching addresses
//
// Bulk workflow: list_ranges → filter to RW/private/large → dump each
// → concatenate → offline signature search.

'use strict';

function rangeInfo(r) {
    return {
        base: r.base.toString(),
        size: r.size,
        protection: r.protection,
        file: r.file ? { path: r.file.path, offset: r.file.offset } : null,
    };
}

rpc.exports = {
    // Every committed page owned by this process.
    listRanges: function (protection) {
        // 'rw-' catches heap + BSS pool globals + Delphi private allocations.
        const prot = protection || 'rw-';
        return Process.enumerateRanges({ protection: prot, coalesce: true }).map(rangeInfo);
    },

    // Dump raw bytes at a given base (hex string) + size.
    dumpRange: function (baseHex, size) {
        const p = ptr(baseHex);
        const bytes = p.readByteArray(size);
        return bytes;   // Frida send/recv path returns as ArrayBuffer to Python
    },

    // Search ALL committed RW ranges for a byte pattern (ASCII/latin-1 hex).
    // Returns [{range_base, addr, offset_in_range}, ...].
    findBytes: function (patternHex) {
        const results = [];
        const ranges = Process.enumerateRanges({ protection: 'rw-', coalesce: true });
        for (const r of ranges) {
            let hits;
            try { hits = Memory.scanSync(r.base, r.size, patternHex); }
            catch (e) { continue; }
            for (const h of hits) {
                results.push({
                    range_base: r.base.toString(),
                    range_size: r.size,
                    addr: h.address.toString(),
                    offset_in_range: h.address.sub(r.base).toInt32(),
                });
                if (results.length >= 200) break;
            }
            if (results.length >= 200) break;
        }
        return results;
    },

    // Total size across RW ranges — sanity check before dumping.
    totalRwBytes: function () {
        let sum = 0;
        for (const r of Process.enumerateRanges({ protection: 'rw-', coalesce: true })) {
            sum += r.size;
        }
        return sum;
    },

    // Modules — .exe + loaded DLLs with base and range.
    listModules: function () {
        return Process.enumerateModules().map(m => ({
            name: m.name, base: m.base.toString(), size: m.size, path: m.path
        }));
    },
};

send({ src: 'dump_ready' });
