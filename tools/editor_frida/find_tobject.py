#!/usr/bin/env python3
"""Find one TObject of each entity type in the editor's memory dump,
then reverse-engineer its field layout.

The editor holds every DB record as a Delphi TObject with fields
(strings, ints, refs). Those TObjects — not the raw .dat bytes — are
the ground truth for semantic validation. Every field we can decode
here lets us verify one column of one table against rust-db.

Approach for each entity type:

  1. Pick an ANCHOR — a unique known ASCII string that appears in ONE
     specific record (e.g. "1.FC Bocholt" for club id 1, "Amber" for
     colour id 1). Ground-truth values for that record's OTHER fields
     come from either the shipped .dat file (parsed with our loader)
     or the editor's own displayed values.

  2. Search memory for the anchor. In Delphi, strings sit in the heap
     with a 4-byte header (refcount + length). Every hit is a candidate
     string body — look at bytes just before/after to find:
       - the string header (length matches, refcount = 1 or -1)
       - pointers TO this string body (they show up 4/8 bytes into a
         TObject as a PChar field)

  3. Walk backwards from each ptr-to-name found and probe candidate
     TObject headers (vptr at +0). Verify by:
       - vptr looks like a code segment address (0x00400000-0x00c00000)
       - other known fields (colour ids, rgb triple, etc.) sit at
         plausible offsets and match ground truth
       - the SAME shape works for a second record (e.g. colour id 2 =
         "Black" at rgb 0,0,0)

  4. Once shape is locked, walk every candidate TObject and dump.

Output: `tobject_shapes.json` — one entry per entity type with the
field offsets we've verified, plus a list of pool bases we found.

Usage:
    py find_tobject.py              # all entity types
    py find_tobject.py --kind colour
"""
import argparse, json, struct
from pathlib import Path

HERE = Path(__file__).parent


def load_ranges(dump_dir):
    idx = json.load((dump_dir / 'index.json').open())
    return [{
        'base': int(r['base'], 16),
        'file': r['file'],
        'size': r['size'],
        'data': (dump_dir / r['file']).read_bytes(),
    } for r in idx['ranges']]


def find_all_bytes(ranges, needle):
    """(range, offset_in_range) for every occurrence."""
    for r in ranges:
        d = r['data']; i = 0
        while True:
            j = d.find(needle, i)
            if j == -1: break
            yield (r, j)
            i = j + 1


def rd_u32(data, off):
    if off + 4 > len(data): return None
    return struct.unpack_from('<I', data, off)[0]


def rd_i32(data, off):
    if off + 4 > len(data): return None
    return struct.unpack_from('<i', data, off)[0]


def rd_u16(data, off):
    if off + 2 > len(data): return None
    return struct.unpack_from('<H', data, off)[0]


def find_pointers_to(ranges, target_addr):
    """Every location in memory that contains the 32-bit LE value target_addr."""
    needle = struct.pack('<I', target_addr & 0xffffffff)
    for r, off in find_all_bytes(ranges, needle):
        yield (r, off)


# -----------------------------------------------------------------------
# Colour probe — smallest, most tractable table. Ground truth: shipped
# `Amber` at rgb (255,170,0) with id=1, `Black` at rgb (0,0,0) with id=2.
# -----------------------------------------------------------------------

def probe_colour(ranges):
    anchor = b'Amber\x00'    # NUL-terminated so hits are the buffer end
    hits = list(find_all_bytes(ranges, anchor))
    print(f'[colour] found {len(hits)} "Amber\\0" locations')
    # For each hit — that's the ASCII "Amber\0" body. Look for a
    # pointer to this address anywhere in memory. If we find one, walk
    # back to a plausible TObject header.
    results = []
    for r, off in hits[:12]:
        addr = r['base'] + off
        ptrs = list(find_pointers_to(ranges, addr))
        print(f'  "Amber" at 0x{addr:08x} — {len(ptrs)} pointers to it')
        for pr, poff in ptrs[:6]:
            ptr_addr = pr['base'] + poff
            # Look at surrounding bytes for context.
            ctx_off = max(0, poff - 32)
            ctx = pr['data'][ctx_off: poff + 48]
            results.append({
                'anchor_addr':      f'0x{addr:08x}',
                'ptr_addr':         f'0x{ptr_addr:08x}',
                'ptr_offset_in_range': poff,
                'ptr_range':        f'0x{pr["base"]:08x}',
                'context_hex':      ctx.hex(),
                'context_ascii':    ''.join(c if 32 <= b < 127 else '.'
                                            for b in ctx
                                            for c in [chr(b)]),
            })
    return results


# -----------------------------------------------------------------------
# Club probe — anchor on "1.FC Bocholt" (club id 1).
# Ground truth from shipped club.dat: kit1_bg_color = 3 (Blue 3),
# kit1_fg_color = 27 (White) — user confirmed via editor screenshot.
# -----------------------------------------------------------------------

def probe_club(ranges):
    anchor = b'1.FC Bocholt\x00'
    hits = list(find_all_bytes(ranges, anchor))
    print(f'[club] found {len(hits)} "1.FC Bocholt\\0" locations')
    results = []
    for r, off in hits[:12]:
        addr = r['base'] + off
        ptrs = list(find_pointers_to(ranges, addr))
        print(f'  "1.FC Bocholt" at 0x{addr:08x} — {len(ptrs)} pointers')
        for pr, poff in ptrs[:6]:
            # The TObject is a few bytes BEFORE this pointer.
            # Delphi TObject.field convention: base+0 = vptr, then fields.
            # Scan candidate object starts from poff-64 backwards.
            for hdr_back in range(0, 128, 4):
                hdr_off = poff - hdr_back
                if hdr_off < 0: break
                vptr = rd_u32(pr['data'], hdr_off)
                if vptr is None: continue
                # vptr should look like a code-segment address.
                # Editor .exe is loaded at 0x00400000 typically.
                if 0x00400000 <= vptr < 0x00c00000:
                    # Plausible. Record it.
                    obj_addr = pr['base'] + hdr_off
                    fields_hex = pr['data'][hdr_off: hdr_off + 128].hex()
                    results.append({
                        'anchor_addr':  f'0x{addr:08x}',
                        'obj_addr':     f'0x{obj_addr:08x}',
                        'name_ptr_off': hdr_back,   # where name-ptr sits inside obj
                        'vptr':         f'0x{vptr:08x}',
                        'first_128B':   fields_hex,
                    })
                    break
    return results


# -----------------------------------------------------------------------
# Nation probe — anchor on "England".
# -----------------------------------------------------------------------

def probe_nation(ranges):
    anchor = b'England\x00'
    hits = list(find_all_bytes(ranges, anchor))
    print(f'[nation] found {len(hits)} "England\\0" locations')
    return [{'anchor_addr': f'0x{r["base"] + off:08x}'} for r, off in hits[:12]]


# -----------------------------------------------------------------------
# First-name probe — anchor on a very common one, but pick a unique one.
# -----------------------------------------------------------------------

def probe_first_name(ranges):
    anchor = b'Ryszard\x00'      # rare enough to be unique
    hits = list(find_all_bytes(ranges, anchor))
    print(f'[first_name] found {len(hits)} "Ryszard\\0" locations')
    return [{'anchor_addr': f'0x{r["base"] + off:08x}'} for r, off in hits[:8]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--dump', default=str(HERE / 'editor_dump'))
    ap.add_argument('--out',  default=str(HERE / 'tobject_shapes.json'))
    ap.add_argument('--kind', default=None,
                    help='colour | club | nation | first_name')
    args = ap.parse_args()

    ranges = load_ranges(Path(args.dump))
    print(f'[+] {len(ranges)} ranges loaded')
    print()

    probes = {
        'colour':      probe_colour,
        'club':        probe_club,
        'nation':      probe_nation,
        'first_name':  probe_first_name,
    }
    if args.kind:
        probes = {args.kind: probes[args.kind]}

    out = {}
    for name, fn in probes.items():
        print(f'=== {name} ===')
        out[name] = fn(ranges)
        print()

    Path(args.out).write_text(json.dumps(out, indent=2))
    print(f'[+] wrote {args.out}')


if __name__ == '__main__':
    main()
